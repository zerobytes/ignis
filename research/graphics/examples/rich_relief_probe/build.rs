use super::{
    certificates,
    compile::{Asset, Level, Patch},
    compiler::{self, Mesh, Vertex},
    material,
};
use anyhow::{Result, ensure};
use glam::{DVec2, Vec3};
use std::{path::Path, time::Instant};
pub const GRID: usize = 768;
pub const PATCH: usize = 64;
fn cross(a: DVec2, b: DVec2) -> f64 {
    a.x * b.y - a.y * b.x
}
fn clip(a: [DVec2; 3], b: [DVec2; 3]) -> Vec<DVec2> {
    let mut poly = a.to_vec();
    for j in 0..3 {
        let origin = b[j];
        let edge = b[(j + 1) % 3] - origin;
        let mut next = vec![];
        if poly.is_empty() {
            break;
        }
        let mut prev = *poly.last().unwrap();
        let mut dp = cross(edge, prev - origin);
        for &cur in &poly {
            let dc = cross(edge, cur - origin);
            if (dp >= 0.) != (dc >= 0.) {
                next.push(prev + (cur - prev) * (dp / (dp - dc)));
            }
            if dc >= 0. {
                next.push(cur);
            }
            prev = cur;
            dp = dc;
        }
        poly = next;
    }
    poly
}
fn sample(q: [DVec2; 3], z: [f64; 3], p: DVec2) -> f64 {
    let den = cross(q[1] - q[0], q[2] - q[0]);
    let b = cross(p - q[0], q[2] - q[0]) / den;
    let c = cross(q[1] - q[0], p - q[0]) / den;
    z[0] * (1. - b - c) + z[1] * b + z[2] * c
}
fn q(id: u32, n: usize) -> DVec2 {
    DVec2::new(
        (id as usize % (n + 1)) as f64,
        (id as usize / (n + 1)) as f64,
    )
}
fn grid_faces(n: usize, x: usize, y: usize, step: usize) -> [[u32; 4]; 2] {
    let a = (y * (n + 1) + x) as u32;
    let b = a + step as u32;
    let d = a + (step * (n + 1)) as u32;
    let c = d + step as u32;
    [[a, b, c, 0], [a, c, d, 0]]
}
pub fn source(n: usize) -> Mesh {
    let mut vertices = vec![];
    for y in 0..=n {
        for x in 0..=n {
            let px = x as f64 / n as f64 * 2. - 1.;
            let py = y as f64 / n as f64 * 2. - 1.;
            vertices.push(Vertex {
                p: [px as f32, py as f32, material::height(px, py) as f32],
                n: [0., 0., 1.],
            });
        }
    }
    let mut faces = vec![];
    for y in 0..n {
        for x in 0..n {
            faces.extend(grid_faces(n, x, y, 1));
        }
    }
    let mut normals = vec![Vec3::ZERO; vertices.len()];
    for f in &faces {
        let [a, b, c] = [f[0], f[1], f[2]].map(|i| Vec3::from_array(vertices[i as usize].p));
        let normal = (b - a).cross(c - a);
        for &i in &f[..3] {
            normals[i as usize] += normal;
        }
    }
    for (v, n) in vertices.iter_mut().zip(normals) {
        v.n = n.normalize().to_array();
    }
    Mesh { vertices, faces }
}
fn candidate(n: usize, ox: usize, oy: usize, size: usize, step: usize) -> Vec<[u32; 4]> {
    let id = |x, y| (y * (n + 1) + x) as u32;
    let mut faces = vec![];
    for y in (oy..oy + size).step_by(step) {
        for x in (ox..ox + size).step_by(step) {
            if step == 1 || !(x == ox || y == oy || x + step == ox + size || y + step == oy + size)
            {
                faces.extend(grid_faces(n, x, y, step));
                continue;
            }
            let mut ring = vec![id(x, y)];
            if y == oy {
                for dx in 1..step {
                    ring.push(id(x + dx, y));
                }
            }
            ring.push(id(x + step, y));
            if x + step == ox + size {
                for dy in 1..step {
                    ring.push(id(x + step, y + dy));
                }
            }
            ring.push(id(x + step, y + step));
            if y + step == oy + size {
                for dx in (1..step).rev() {
                    ring.push(id(x + dx, y + step));
                }
            }
            ring.push(id(x, y + step));
            if x == ox {
                for dy in (1..step).rev() {
                    ring.push(id(x, y + dy));
                }
            }
            let center = id(x + step / 2, y + step / 2);
            for j in 0..ring.len() {
                faces.push([center, ring[j], ring[(j + 1) % ring.len()], 0]);
            }
        }
    }
    faces
}
/// Exact extrema of the difference of two piecewise affine heightfields occur
/// at vertices of their triangle intersections, not only at source vertices.
fn certify(mesh: &Mesh, n: usize, faces: &[[u32; 4]], physical: bool) -> Result<f32> {
    let mut max_error = 0f64;
    let coordinate = |i: u32| {
        if physical {
            let p = mesh.vertices[i as usize].p;
            DVec2::new(p[0] as f64, p[1] as f64)
        } else {
            q(i, n)
        }
    };
    for f in faces {
        let bounds = [f[0], f[1], f[2]].map(|i| q(i, n));
        let b = [f[0], f[1], f[2]].map(coordinate);
        let bz = [f[0], f[1], f[2]].map(|i| mesh.vertices[i as usize].p[2] as f64);
        let lo = bounds[0].min(bounds[1]).min(bounds[2]);
        let hi = bounds[0].max(bounds[1]).max(bounds[2]);
        let mut covered = 0.;
        for y in lo.y as usize..hi.y as usize {
            for x in lo.x as usize..hi.x as usize {
                for t in grid_faces(n, x, y, 1) {
                    let a = [t[0], t[1], t[2]].map(coordinate);
                    let az = [t[0], t[1], t[2]].map(|i| mesh.vertices[i as usize].p[2] as f64);
                    let poly = clip(a, b);
                    if poly.len() < 3 {
                        continue;
                    }
                    for j in 1..poly.len() - 1 {
                        covered += cross(poly[j] - poly[0], poly[j + 1] - poly[0]) * 0.5;
                    }
                    for p in poly {
                        max_error = max_error.max((sample(a, az, p) - sample(b, bz, p)).abs());
                    }
                }
            }
        }
        let expected = cross(b[1] - b[0], b[2] - b[0]) * 0.5;
        ensure!(
            expected > 0. && (covered - expected).abs() < expected * 1e-7 + 1e-14,
            "chart coverage"
        );
    }
    // Round upward; retain a 2e-6 numerical guard matching prior regional certificates.
    Ok(((max_error + if physical { 1e-10 } else { 2e-6 }) as f32).next_up())
}
pub fn regional(src: &Mesh, n: usize, size: usize) -> Result<Asset> {
    regional_kind(src, n, size, false)
}
struct LocalNode {
    faces: Vec<[u32; 4]>,
    error: f32,
    children: Vec<LocalNode>,
}
impl LocalNode {
    fn build(src: &Mesh, n: usize, x: usize, y: usize, size: usize) -> Result<Self> {
        if size == 2 {
            return Ok(Self {
                faces: candidate(n, x, y, size, 1),
                error: 0.,
                children: vec![],
            });
        }
        // All leaf perimeters keep native samples, including internal neighbors
        // at different scales. This avoids balancing/stitching work at runtime.
        let faces = candidate(n, x, y, size, size);
        let error = certify(src, n, &faces, false)?;
        ensure!(
            certify(src, n, &faces, true)? <= error,
            "local physical certificate"
        );
        let half = size / 2;
        let children = [(x, y), (x + half, y), (x, y + half), (x + half, y + half)]
            .into_iter()
            .map(|(x, y)| Self::build(src, n, x, y, half))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            faces,
            error,
            children,
        })
    }
    fn choose(&self, budget: f32, out: &mut Vec<[u32; 4]>) -> f32 {
        if self.error <= budget || self.children.is_empty() {
            out.extend_from_slice(&self.faces);
            self.error
        } else {
            self.children
                .iter()
                .map(|c| c.choose(budget, out))
                .fold(0f32, f32::max)
        }
    }
}
fn regional_kind(src: &Mesh, n: usize, size: usize, adaptive: bool) -> Result<Asset> {
    ensure!(
        !adaptive || size.is_power_of_two(),
        "adaptive chart power of two"
    );
    ensure!(
        n.is_multiple_of(size) && size.is_multiple_of(16),
        "grid partition"
    );
    let mut faces = vec![];
    let mut patches = vec![];
    let mut variants = vec![];
    for oy in (0..n).step_by(size) {
        for ox in (0..n).step_by(size) {
            let native = candidate(n, ox, oy, size, 1);
            let mut low = Vec3::splat(f32::INFINITY);
            let mut high = Vec3::splat(f32::NEG_INFINITY);
            let mut cone = 0f32;
            for y in oy..=oy + size {
                for x in ox..=ox + size {
                    let p = Vec3::from_array(src.vertices[y * (n + 1) + x].p);
                    low = low.min(p);
                    high = high.max(p);
                }
            }
            let start = faces.len() as u32 * 3;
            faces.extend(native);
            let mut levels = vec![];
            let tree = adaptive
                .then(|| LocalNode::build(src, n, ox, oy, size))
                .transpose()?;
            for (li, step) in [2, 4, 8, 16].into_iter().enumerate() {
                let (reduced, error) = if let Some(tree) = &tree {
                    let mut faces = vec![];
                    let error =
                        tree.choose([0.000008, 0.000032, 0.000128, 0.000512][li], &mut faces);
                    (faces, error)
                } else {
                    let faces = candidate(n, ox, oy, size, step);
                    let error = certify(src, n, &faces, false)?;
                    (faces, error)
                };
                if error == 0. {
                    let mut native = candidate(n, ox, oy, size, 1);
                    let mut actual = reduced.clone();
                    native.sort_unstable();
                    actual.sort_unstable();
                    ensure!(actual == native, "zero-error level must be native");
                } else {
                    let checked = certify(src, n, &reduced, true)?;
                    ensure!(
                        checked <= error,
                        "physical-coordinate certificate {checked} exceeds {error}"
                    );
                }
                for t in &reduced {
                    let [a, b, c] =
                        [t[0], t[1], t[2]].map(|i| Vec3::from_array(src.vertices[i as usize].p));
                    let normal = (b - a).cross(c - a).normalize();
                    cone = cone.max((1. - normal.z * normal.z).max(0.).sqrt());
                }
                levels.push((reduced, error));
            }
            for t in &faces[start as usize / 3..] {
                let [a, b, c] =
                    [t[0], t[1], t[2]].map(|i| Vec3::from_array(src.vertices[i as usize].p));
                let normal = (b - a).cross(c - a).normalize();
                cone = cone.max((1. - normal.z * normal.z).max(0.).sqrt());
            }
            patches.push(Patch {
                low: low.to_array(),
                high: high.to_array(),
                cone: [0., 0., 1., (cone + 1e-5).min(1.)],
                levels: vec![Level {
                    start,
                    count: (size * size * 6) as u32,
                    error: 0.,
                    normal: 0.,
                }],
            });
            variants.push(levels);
        }
    }
    let source_count = faces.len() as u32 * 3;
    for level in 0..4 {
        for (i, levels) in variants.iter().enumerate() {
            let (tris, error) = &levels[level];
            if adaptive {
                let native = &patches[i].levels[0];
                if tris.len() * 3 >= native.count as usize {
                    // No reduction: reference the native range, do not store a duplicate.
                    let native = native.clone();
                    patches[i].levels.push(native);
                    continue;
                }
                if let Some(same) = patches[i]
                    .levels
                    .iter()
                    .find(|l| {
                        l.count as usize == tris.len() * 3
                            && &faces[l.start as usize / 3..(l.start + l.count) as usize / 3]
                                == tris.as_slice()
                    })
                    .cloned()
                {
                    patches[i].levels.push(same);
                    continue;
                }
            }
            let start = faces.len() as u32 * 3;
            faces.extend_from_slice(tris);
            patches[i].levels.push(Level {
                start,
                count: tris.len() as u32 * 3,
                error: *error,
                normal: 0.,
            });
        }
    }
    let result = Asset {
        mesh: Mesh {
            vertices: src.vertices.clone(),
            faces,
        },
        source_count,
        patches,
    };
    compiler::validate(&result.mesh)?;
    Ok(result)
}
pub fn refine(dir: &Path) -> Result<()> {
    verify_inputs(dir)?;
    let source = compiler::decode_mesh(&std::fs::read(dir.join("source.igm"))?)?;
    let start = Instant::now();
    let asset = regional_kind(&source, GRID, PATCH, true)?;
    let bytes = super::compile::encode(&asset);
    let loaded = super::compile::decode(&bytes)?;
    ensure!(
        super::compile::encode(&loaded) == bytes,
        "refined roundtrip"
    );
    std::fs::write(dir.join("refined.igl"), &bytes)?;
    let report = serde_json::json!({"compile_ms":start.elapsed().as_secs_f64()*1000.,"patches":asset.patches.len(),"level_triangles":(0..5).map(|l|asset.patches.iter().map(|p|p.levels[l].count as usize/3).sum::<usize>()).collect::<Vec<_>>(),"file_bytes":bytes.len(),"gpu_geometry_bytes":asset.mesh.vertices.len()*12+asset.mesh.faces.len()*12});
    std::fs::write(
        dir.join("refined-build.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
pub fn run(dir: &Path) -> Result<()> {
    let start = Instant::now();
    let src = source(GRID);
    compiler::validate(&src)?;
    std::fs::write(dir.join("source.igm"), compiler::encode_mesh(&src))?;
    println!("source {}T", src.faces.len());
    let colors = material::build_color(2048);
    let normals = material::build_normal(1024, &src, GRID);
    std::fs::write(dir.join("color.irm"), material::encode(&colors))?;
    std::fs::write(dir.join("normal.irm"), material::encode(&normals))?;
    let authored_ms = start.elapsed().as_secs_f64() * 1000.;
    let start = Instant::now();
    let (prior, _) = compiler::compile_borders(
        &src,
        compiler::hash(&compiler::encode_mesh(&src)),
        true,
        true,
    )?;
    let bytes = compiler::encode(&prior);
    let hierarchy = certificates::build(&prior, compiler::hash(&bytes))?;
    std::fs::write(dir.join("previous.igc"), &bytes)?;
    std::fs::write(dir.join("previous.ich"), hierarchy)?;
    let prior_ms = start.elapsed().as_secs_f64() * 1000.;
    println!("prior {}T", prior.proxy.faces.len());
    let start = Instant::now();
    let asset = regional(&src, GRID, PATCH)?;
    let bin = super::compile::encode(&asset);
    super::compile::decode(&bin)?;
    std::fs::write(dir.join("regional.igl"), &bin)?;
    let report = serde_json::json!({"grid":GRID,"patch_size":PATCH,"source_triangles":src.faces.len(),"previous_triangles":prior.proxy.faces.len(),"patches":asset.patches.len(),"level_triangles":(0..5).map(|l|asset.patches.iter().map(|p|p.levels[l].count as usize/3).sum::<usize>()).collect::<Vec<_>>(),"source_gpu_bytes":src.vertices.len()*12+src.faces.len()*12,"regional_gpu_bytes":asset.mesh.vertices.len()*12+asset.mesh.faces.len()*12,"regional_file_bytes":bin.len(),"texture_gpu_bytes":colors.levels.iter().chain(normals.levels.iter()).map(Vec::len).sum::<usize>(),"authoring_ms":authored_ms,"previous_compile_ms":prior_ms,"regional_compile_ms":start.elapsed().as_secs_f64()*1000.});
    std::fs::write(dir.join("build.json"), serde_json::to_vec_pretty(&report)?)?;
    println!("{report}");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    fn boundary(faces: &[[u32; 4]]) -> Vec<(u32, u32)> {
        let mut e = HashMap::new();
        for f in faces {
            for j in 0..3 {
                let a = f[j];
                let b = f[(j + 1) % 3];
                *e.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        let mut b = e
            .into_iter()
            .filter_map(|(e, n)| (n == 1).then_some(e))
            .collect::<Vec<_>>();
        b.sort_unstable();
        b
    }
    #[test]
    fn omitted_interior_peak_is_in_the_error_bound() -> Result<()> {
        let mut src = source(32);
        for v in &mut src.vertices {
            v.p[2] = 0.;
        }
        src.vertices[2 * 33 + 1].p[2] = 0.1;
        let coarse = candidate(32, 0, 0, 32, 16);
        let err = certify(&src, 32, &coarse, false)?;
        assert!(err >= 0.1 && err < 0.10001);
        assert!(certify(&src, 32, &coarse, true)? <= err);
        Ok(())
    }
    #[test]
    fn all_levels_share_the_exact_boundary_and_certificate_detects_real_relief() -> Result<()> {
        let src = source(32);
        let a = regional(&src, 32, 16)?;
        for p in &a.patches {
            let slice =
                |l: &Level| &a.mesh.faces[l.start as usize / 3..(l.start + l.count) as usize / 3];
            let original = boundary(slice(&p.levels[0]));
            for l in &p.levels[1..] {
                assert_eq!(boundary(slice(l)), original);
                assert!(l.error > 2e-6);
                assert!(l.count < p.levels[0].count);
            }
        }
        let bytes = super::super::compile::encode(&a);
        assert_eq!(
            bytes,
            super::super::compile::encode(&super::super::compile::decode(&bytes)?)
        );
        Ok(())
    }
    #[test]
    fn interior_crossing_is_certified() {
        let a = [DVec2::new(0., 0.), DVec2::new(1., 0.), DVec2::new(1., 1.)];
        let b = [DVec2::new(0., 0.), DVec2::new(1., 0.), DVec2::new(0., 1.)];
        let poly = clip(a, b);
        assert!(poly.contains(&DVec2::new(0.5, 0.5)));
    }
    #[test]
    fn local_refinement_preserves_spike_and_all_shared_edges() -> Result<()> {
        let mut src = source(32);
        for v in &mut src.vertices {
            v.p[2] = 0.;
        }
        src.vertices[11 * 33 + 7].p[2] = 0.1;
        let asset = regional_kind(&src, 32, 32, true)?;
        let patch = &asset.patches[0];
        let native = &asset.mesh.faces[..asset.source_count as usize / 3];
        let perimeter = boundary(native);
        for l in &patch.levels[1..] {
            let tris = &asset.mesh.faces[l.start as usize / 3..(l.start + l.count) as usize / 3];
            assert_eq!(boundary(tris), perimeter);
            assert!(l.count < asset.source_count);
            assert!(certify(&src, 32, tris, true)? <= l.error);
            let mut edges = HashMap::new();
            for t in tris {
                for j in 0..3 {
                    let a = t[j];
                    let b = t[(j + 1) % 3];
                    *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                }
            }
            assert!(edges.values().all(|&n| n == 1 || n == 2));
        }
        let bytes = super::super::compile::encode(&asset);
        assert_eq!(
            bytes,
            super::super::compile::encode(&super::super::compile::decode(&bytes)?)
        );
        // Flat areas with one isolated peak are shared across equal budgets.
        assert!(asset.mesh.faces.len() < native.len() * 2);
        Ok(())
    }
}

/// Validate the saved source independently of regional face ordering.
pub fn verify_inputs(dir: &Path) -> Result<()> {
    let src = compiler::decode_mesh(&std::fs::read(dir.join("source.igm"))?)?;
    let old = compiler::decode(&std::fs::read(dir.join("previous.igc"))?)?;
    let a = super::compile::decode(&std::fs::read(dir.join("regional.igl"))?)?;
    ensure!(
        bytemuck::cast_slice::<_, u8>(&src.vertices)
            == bytemuck::cast_slice::<_, u8>(&a.mesh.vertices),
        "regional source vertices changed"
    );
    // The prior compiler welds and sorts vertices/faces. Compare their exact
    // values and oriented faces after remapping, not their storage order.
    let bits = |v: &Vertex| {
        let mut key = [0u32; 6];
        for (out, value) in key.iter_mut().zip(v.p.into_iter().chain(v.n)) {
            *out = if value == 0. { 0 } else { value.to_bits() };
        }
        key
    };
    let lookup = src
        .vertices
        .iter()
        .enumerate()
        .map(|(i, v)| (bits(v), i as u32))
        .collect::<std::collections::HashMap<_, _>>();
    let remap = old
        .source
        .vertices
        .iter()
        .map(|v| {
            lookup
                .get(&bits(v))
                .copied()
                .ok_or_else(|| anyhow::anyhow!("previous source vertex changed"))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut old_faces = old
        .source
        .faces
        .iter()
        .map(|t| {
            [
                remap[t[0] as usize],
                remap[t[1] as usize],
                remap[t[2] as usize],
                t[3],
            ]
        })
        .collect::<Vec<_>>();
    old_faces.sort_unstable();
    let mut expected = src.faces;
    expected.sort_unstable();
    ensure!(expected == old_faces, "previous source faces changed");
    let mut actual = a.mesh.faces[..a.source_count as usize / 3].to_vec();
    actual.sort_unstable();
    ensure!(expected == actual, "regional source faces changed");
    Ok(())
}
