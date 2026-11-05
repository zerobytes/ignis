//! IGT1 source-only diagnostic; compiled plane slopes, no shading normals.
use super::{compile, compiler, experiment, material, render::Gpu};
use anyhow::{Result, ensure};
use glam::DVec3;
use std::{path::Path, time::Instant};
const MODES: [&str; 4] = [
    "implicit",
    "analytic-lod",
    "geometric-lod",
    "geometric-grad",
];
const HEADER: usize = 40;
fn slopes(mesh: &compiler::Mesh) -> Result<Vec<[f32; 2]>> {
    mesh.faces
        .iter()
        .map(|f| {
            let a = DVec3::from_array(mesh.vertices[f[0] as usize].p.map(f64::from));
            let b = DVec3::from_array(mesh.vertices[f[1] as usize].p.map(f64::from));
            let c = DVec3::from_array(mesh.vertices[f[2] as usize].p.map(f64::from));
            let n = (b - a).cross(c - a);
            ensure!(
                n.is_finite() && n.z.abs() > 1e-20,
                "non-heightfield/degenerate face"
            );
            let s = [(-n.x / n.z) as f32, (-n.y / n.z) as f32];
            ensure!(s.iter().all(|v| v.is_finite()), "nonfinite slope");
            for edge in [b - a, c - a] {
                let error = (edge.z - f64::from(s[0]) * edge.x - f64::from(s[1]) * edge.y).abs();
                // Cancellation between x/y terms can make edge.z nearly zero.
                // Bound f32 rounding by the evaluated terms, not edge length.
                let magnitude = (f64::from(s[0]) * edge.x).abs()
                    + (f64::from(s[1]) * edge.y).abs()
                    + edge.z.abs();
                ensure!(
                    error <= f64::from(f32::EPSILON) * magnitude + 1e-12,
                    "slope quantization exceeds tolerance"
                );
            }
            Ok(s)
        })
        .collect()
}
fn encode(mesh: &compiler::Mesh) -> Result<Vec<u8>> {
    let values = slopes(mesh)?;
    let payload: Vec<u8> = values
        .iter()
        .flatten()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    let mut b = b"IGT1".to_vec();
    b.extend(1u32.to_le_bytes());
    b.extend((values.len() as u64).to_le_bytes());
    b.extend(compiler::hash(bytemuck::cast_slice(&mesh.vertices)).to_le_bytes());
    b.extend(compiler::hash(bytemuck::cast_slice(&mesh.faces)).to_le_bytes());
    b.extend(compiler::hash(&payload).to_le_bytes());
    b.extend(payload);
    Ok(b)
}
pub(super) fn decode(b: &[u8], mesh: &compiler::Mesh) -> Result<Vec<[f32; 2]>> {
    ensure!(
        b.len() >= HEADER && &b[..8] == b"IGT1\x01\0\0\0",
        "IGT1 header"
    );
    let u64_at = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    ensure!(
        u64_at(8) == mesh.faces.len() as u64 && b.len() == HEADER + mesh.faces.len() * 8,
        "IGT1 length"
    );
    ensure!(
        u64_at(16) == compiler::hash(bytemuck::cast_slice(&mesh.vertices))
            && u64_at(24) == compiler::hash(bytemuck::cast_slice(&mesh.faces)),
        "IGT1 source mismatch"
    );
    ensure!(
        u64_at(32) == compiler::hash(&b[HEADER..]),
        "IGT1 payload checksum"
    );
    let values: Vec<_> = b[HEADER..]
        .chunks_exact(8)
        .map(|v| {
            [
                f32::from_le_bytes(v[..4].try_into().unwrap()),
                f32::from_le_bytes(v[4..].try_into().unwrap()),
            ]
        })
        .collect();
    ensure!(
        values.iter().flatten().all(|v| v.is_finite()),
        "IGT1 finite"
    );
    Ok(values)
}
pub(super) fn source(dir: &Path) -> Result<compiler::Mesh> {
    super::build::verify_inputs(dir)?;
    let mut asset = compile::decode(&std::fs::read(dir.join("refined.igl"))?)?;
    asset.mesh.faces.truncate(asset.source_count as usize / 3);
    Ok(asset.mesh)
}
pub fn build(dir: &Path) -> Result<()> {
    let mesh = source(dir)?;
    let start = Instant::now();
    let b = encode(&mesh)?;
    decode(&b, &mesh)?;
    std::fs::write(dir.join("source.igt"), &b)?;
    println!(
        "IGT1 {} faces, {} bytes, {:.3}s compile+verify",
        mesh.faces.len(),
        b.len(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
pub fn run(dir: &Path) -> Result<()> {
    let mesh = source(dir)?;
    let compiled = std::fs::read(dir.join("source.igt"))?;
    let values = decode(&compiled, &mesh)?;
    let color = material::decode(&std::fs::read(dir.join("color.irm"))?)?;
    let normal = material::decode(&std::fs::read(dir.join("normal.irm"))?)?;
    let texture_bytes: usize = color
        .levels
        .iter()
        .chain(&normal.levels)
        .map(Vec::len)
        .sum();
    let full = vec![(0, mesh.faces.len() as u32 * 3)];
    let mut quality = vec![];
    let mut returns = vec![];
    let mut repeated = vec![];
    let mut timings = vec![];
    let mut adapters = vec![];
    let mut geometry_bytes = 0;
    for (width, height) in [(1728, 1117), (3456, 2234)] {
        let mut gpu = pollster::block_on(Gpu::new_features(
            width,
            height,
            wgpu::Features::SHADER_PRIMITIVE_INDEX,
        ))?;
        gpu.install_rich_material(color.size, &color.levels, normal.size, &normal.levels);
        gpu.install_rich_slopes(&values);
        let variants = MODES.map(|m| gpu.rich_variant(m));
        let uploaded = gpu.upload_rich(&mesh);
        geometry_bytes = uploaded.bytes;
        adapters.push(gpu.adapter.clone());
        // Alternate cached pipelines, no diagnostic MRT and no texture reuploads.
        for case in [0, 3] {
            let p = experiment::pose(case, 0, 8);
            let mut first = Vec::new();
            for variant in &variants {
                gpu.select_rich_variant(variant);
                first.push(gpu.capture_selected(&uploaded, p, Some(&full))?);
            }
            for turn in 0..32 {
                for mode in if turn % 2 == 0 {
                    [0, 1, 2, 3]
                } else {
                    [3, 2, 1, 0]
                } {
                    gpu.select_rich_variant(&variants[mode]);
                    let pixels = gpu.capture_selected(&uploaded, p, Some(&full))?;
                    let changed = pixels
                        .iter()
                        .zip(&first[mode])
                        .filter(|(a, b)| a != b)
                        .count();
                    repeated.push(serde_json::json!({"width":width,"case":case,"turn":turn,"mode":MODES[mode],"changed_pixels":changed,"quality":experiment::quality(&first[mode],&pixels)}));
                }
            }
        }
        for case in 0..5 {
            let steps = if case == 4 { 32 } else { 8 };
            let mut first = [0; 4];
            for frame in 0..=steps {
                let p = experiment::pose(case, frame, steps);
                let mut captures = Vec::new();
                for variant in &variants {
                    gpu.select_rich_variant(variant);
                    captures.push(gpu.capture_selected(&uploaded, p, Some(&full))?);
                }
                for (mode, pixels) in captures.iter().enumerate() {
                    let hash = compiler::hash(bytemuck::cast_slice(pixels));
                    if frame == 0 {
                        first[mode] = hash;
                    }
                    if frame == steps {
                        returns.push(serde_json::json!({"width":width,"case":case,"mode":MODES[mode],"pass":first[mode]==hash,"first_hash":first[mode].to_string(),"return_hash":hash.to_string()}));
                    }
                    experiment::png(
                        &dir.join(format!(
                            "tangent-{width}-{case}-{frame}-{}.png",
                            MODES[mode]
                        )),
                        pixels,
                        width,
                        height,
                    )?;
                }
                quality.push(serde_json::json!({"width":width,"height":height,"case":case,"frame":frame,"distance":p.distance,
                    "analytic-lod":experiment::quality(&captures[0],&captures[1]),
                    "geometric-lod":experiment::quality(&captures[0],&captures[2]),
                    "geometric-grad":experiment::quality(&captures[0],&captures[3]),
                    "geometry_vs_shading":experiment::quality(&captures[1],&captures[2]),
                    "gradient_vs_scalar":experiment::quality(&captures[2],&captures[3])}));
            }
            let poses = (0..32)
                .map(|i| experiment::pose(case, i, 32))
                .collect::<Vec<_>>();
            for round in 0..8 {
                for (slot, mode) in (if round % 2 == 0 {
                    [0, 1, 2, 3, 3, 2, 1, 0]
                } else {
                    [3, 2, 1, 0, 0, 1, 2, 3]
                })
                .into_iter()
                .enumerate()
                {
                    gpu.select_rich_variant(&variants[mode]);
                    let _ = gpu.batch_selected(&uploaded, &poses, |_| full.clone())?;
                    let timing = gpu.batch_selected(&uploaded, &poses, |_| full.clone())?;
                    timings.push(serde_json::json!({"width":width,"case":case,"round":round,"slot":slot,"mode":MODES[mode],"timing":timing}));
                }
            }
            println!("tangent {width} case{case} complete");
        }
    }
    let data = serde_json::json!({"adapters":adapters,"source_triangles":mesh.faces.len(),"geometry_bytes":geometry_bytes,"texture_bytes":texture_bytes,"slopes_bytes":values.len()*8,"compiled_file_bytes":compiled.len(),"payload_hash":compiler::hash(&compiled).to_string(),"quality":quality,"returns":returns,"repeated":repeated,"timings":timings});
    std::fs::write(
        dir.join("tangent-results.json"),
        serde_json::to_vec_pretty(&data)?,
    )?;
    ensure!(
        returns.iter().all(|r| r["pass"] == true)
            && repeated.iter().all(|r| r["changed_pixels"] == 0),
        "tangent return gate failed; data retained"
    );
    ensure!(
        quality
            .iter()
            .all(|r| r["geometric-lod"]["pass"] == true && r["geometric-grad"]["pass"] == true),
        "tangent quality gate failed; data retained"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn plane() -> compiler::Mesh {
        compiler::Mesh {
            vertices: [[0., 0., 1.], [1., 0., 3.], [0., 1., -2.]]
                .map(|p| compiler::Vertex { p, n: [0., 0., 1.] })
                .to_vec(),
            faces: vec![[0, 1, 2, 0]],
        }
    }
    #[test]
    fn codec_preserves_face_plane_and_rejects_stale_or_corrupt_data() {
        let mesh = plane();
        let b = encode(&mesh).unwrap();
        assert_eq!(decode(&b, &mesh).unwrap(), vec![[2., -3.]]);
        for i in [0, 4, 8, 16, 24, 32, 40] {
            let mut bad = b.clone();
            bad[i] ^= 1;
            assert!(decode(&bad, &mesh).is_err());
        }
        assert!(decode(&b[..39], &mesh).is_err());
        let mut other = plane();
        other.vertices[2].p[2] += 1.;
        assert!(decode(&b, &other).is_err());
        let mut extra = b.clone();
        extra.push(0);
        assert!(decode(&extra, &mesh).is_err());
        let mut bad = b.clone();
        bad[40..44].copy_from_slice(&f32::NAN.to_le_bytes());
        let h = compiler::hash(&bad[HEADER..]);
        bad[32..40].copy_from_slice(&h.to_le_bytes());
        assert!(decode(&bad, &mesh).is_err());
    }
    #[test]
    fn compile_rejects_degenerate_or_vertical_faces() {
        let mut mesh = plane();
        mesh.vertices[2].p = mesh.vertices[1].p;
        assert!(encode(&mesh).is_err());
        mesh.vertices[2].p = [1., 0., 7.];
        assert!(encode(&mesh).is_err());
    }
    #[test]
    fn quantization_bound_handles_cancelling_plane_terms() {
        let mut mesh = plane();
        mesh.vertices[0].p = [0., 0., 0.];
        mesh.vertices[1].p = [1., 1., 0.];
        mesh.vertices[2].p = [0.5, 0.50001, 1.];
        let b = encode(&mesh).unwrap();
        let s = decode(&b, &mesh).unwrap()[0];
        assert!((s[0] + s[1]).abs() < 0.01);
    }
}
