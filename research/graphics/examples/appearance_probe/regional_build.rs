//! Offline spatial/material partition and six local surface caches per region.
use super::*;
use std::collections::BTreeMap;
pub(super) const SIDE: usize = 16;
pub(super) const FACE_TEXELS: usize = 341; // 16²+8²+4²+2²+1²
pub(super) const AXES: [[Vec3; 3]; 6] = [
    [Vec3::Z, Vec3::Y, Vec3::X],
    [Vec3::NEG_Z, Vec3::Y, Vec3::NEG_X],
    [Vec3::X, Vec3::Z, Vec3::Y],
    [Vec3::X, Vec3::NEG_Z, Vec3::NEG_Y],
    [Vec3::NEG_X, Vec3::Y, Vec3::Z],
    [Vec3::X, Vec3::Y, Vec3::NEG_Z],
];
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Region {
    pub min: Vec3,
    pub max: Vec3,
    pub center: Vec3,
    pub radius: f32,
    pub edge: f32,
    pub first: u32,
    pub count: u32,
    pub material: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub(super) struct Texel {
    pub color: [f32; 4],
    pub normal_depth: [f32; 4],
}
#[derive(Serialize, Deserialize)]
pub(super) struct Manifest {
    pub signature: u64,
    pub checksum: u64,
    pub regions: Vec<Region>,
    pub indices: Vec<u32>,
    pub bake_seconds: f64,
}
pub(super) struct Asset {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub regions: Vec<Region>,
    pub cache: Vec<Texel>,
    pub cache_bytes: usize,
    pub bake_seconds: f64,
}
fn key(vs: &[Vertex], tri: &[u32]) -> [i32; 4] {
    let c = tri
        .iter()
        .map(|i| Vec3::from_array(vs[*i as usize].position))
        .sum::<Vec3>()
        / 3.;
    let q = ((c - Vec3::splat(-1.23)) / 0.3075).floor().as_ivec3();
    [q.x, q.y, q.z, vs[tri[0] as usize].tile as i32]
}
pub(super) fn partition(vs: &[Vertex], indices: &[u32]) -> (Vec<u32>, Vec<Region>) {
    let mut groups = BTreeMap::<[i32; 4], Vec<u32>>::new();
    for t in indices.chunks_exact(3) {
        groups.entry(key(vs, t)).or_default().extend_from_slice(t);
    }
    let mut ordered = Vec::with_capacity(indices.len());
    let mut regions = Vec::new();
    for (key, ids) in groups {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let mut edge = 0f32;
        for t in ids.chunks_exact(3) {
            for i in 0..3 {
                let a = Vec3::from_array(vs[t[i] as usize].position);
                let b = Vec3::from_array(vs[t[(i + 1) % 3] as usize].position);
                min = min.min(a);
                max = max.max(a);
                edge = edge.max(a.distance(b));
            }
        }
        let center = (min + max) * 0.5;
        let radius = ((max - min) * 0.5).max_element().max(0.001) * 1.08;
        regions.push(Region {
            min,
            max,
            center,
            radius,
            edge,
            first: ordered.len() as u32,
            count: ids.len() as u32,
            material: key[3] as u32,
        });
        ordered.extend(ids);
    }
    (ordered, regions)
}
fn bake_region(vs: &[Vertex], ids: &[u32], r: &Region) -> Vec<Texel> {
    let mut result = Vec::with_capacity(FACE_TEXELS * 6);
    for [u, v, w] in AXES {
        let n = SIDE * 2;
        let mut samples = vec![Texel::default(); n * n];
        let mut depths = vec![f32::NEG_INFINITY; n * n];
        for t in ids.chunks_exact(3) {
            let source = t.map_ref(vs);
            let p = source.map(|a| {
                let d = Vec3::from_array(a.position) - r.center;
                Vec3::new(
                    (d.dot(u) / r.radius * 0.5 + 0.5) * n as f32,
                    (d.dot(v) / r.radius * 0.5 + 0.5) * n as f32,
                    d.dot(w),
                )
            });
            let a = p[0].truncate();
            let ab = (p[1] - p[0]).truncate();
            let ac = (p[2] - p[0]).truncate();
            let det = ab.perp_dot(ac);
            if det.abs() < 1e-9 {
                continue;
            }
            let lo = p[0].min(p[1]).min(p[2]).truncate().floor().max(Vec2::ZERO);
            let hi = p[0]
                .max(p[1])
                .max(p[2])
                .truncate()
                .ceil()
                .min(Vec2::splat(n as f32));
            let geom = (Vec3::from_array(source[1].position)
                - Vec3::from_array(source[0].position))
            .cross(Vec3::from_array(source[2].position) - Vec3::from_array(source[0].position));
            let sign = if geom.dot(w) < 0. { -1. } else { 1. };
            for y in lo.y as usize..hi.y as usize {
                for x in lo.x as usize..hi.x as usize {
                    let q = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - a;
                    let b = q.perp_dot(ac) / det;
                    let c = ab.perp_dot(q) / det;
                    if b < 0. || c < 0. || b + c > 1. {
                        continue;
                    }
                    let weights = [1. - b - c, b, c];
                    let depth = p[0].z * weights[0] + p[1].z * b + p[2].z * c;
                    let at = y * n + x;
                    if depth <= depths[at] {
                        continue;
                    }
                    depths[at] = depth;
                    let mut color = Vec3::ZERO;
                    let mut normal = Vec3::ZERO;
                    for i in 0..3 {
                        color += Vec3::from_array(source[i].color) * weights[i];
                        normal += Vec3::from_array(source[i].normal) * weights[i];
                    }
                    samples[at] = Texel {
                        color: color.extend(1.).to_array(),
                        normal_depth: (normal.normalize_or_zero() * sign).extend(depth).to_array(),
                    };
                }
            }
        }
        let mut level = vec![Texel::default(); SIDE * SIDE];
        for y in 0..SIDE {
            for x in 0..SIDE {
                for sy in 0..2 {
                    for sx in 0..2 {
                        add(
                            &mut level[y * SIDE + x],
                            samples[(y * 2 + sy) * n + x * 2 + sx],
                            0.25,
                        );
                    }
                }
            }
        }
        let mut size = SIDE;
        loop {
            result.extend_from_slice(&level);
            if size == 1 {
                break;
            }
            let next = size / 2;
            let mut down = vec![Texel::default(); next * next];
            for y in 0..next {
                for x in 0..next {
                    for sy in 0..2 {
                        for sx in 0..2 {
                            add(
                                &mut down[y * next + x],
                                level[(y * 2 + sy) * size + x * 2 + sx],
                                0.25,
                            );
                        }
                    }
                }
            }
            level = down;
            size = next;
        }
    }
    result
}
// A fixed three-vertex gather, avoiding a heap allocation in the bake raster loop.
trait Triangle {
    fn map_ref<'a>(&self, vs: &'a [Vertex]) -> [&'a Vertex; 3];
}
impl Triangle for [u32] {
    fn map_ref<'a>(&self, vs: &'a [Vertex]) -> [&'a Vertex; 3] {
        [
            &vs[self[0] as usize],
            &vs[self[1] as usize],
            &vs[self[2] as usize],
        ]
    }
}
fn add(a: &mut Texel, b: Texel, w: f32) {
    for i in 0..4 {
        a.color[i] += b.color[i] * w;
        a.normal_depth[i] += b.normal_depth[i] * w;
    }
}
pub(super) fn assets(bake: bool) -> Result<Vec<Asset>> {
    let mut assets = Vec::new();
    for name in ["rock", "chest", "tree"] {
        let root = Path::new(ASSETS).join(name);
        let vb = fs::read(root.join("vertices.bin"))?;
        let ib = fs::read(root.join("indices.bin"))?;
        let (vertices, source) = decode(&vb, &ib)?;
        let signature = hash(&vb)
            ^ hash(&ib).rotate_left(13)
            ^ hash(include_bytes!("regional_build.rs")).rotate_left(31);
        let folder = Path::new(OUT).join("cache").join(name);
        fs::create_dir_all(&folder)?;
        if bake {
            let start = Instant::now();
            let (indices, regions) = partition(&vertices, &source);
            let mut cache = Vec::new();
            for r in &regions {
                cache.extend(bake_region(
                    &vertices,
                    &indices[r.first as usize..(r.first + r.count) as usize],
                    r,
                ));
            }
            let bytes = bytemuck::cast_slice(&cache);
            fs::write(folder.join("surfaces.bin"), bytes)?;
            let manifest = Manifest {
                signature,
                checksum: hash(bytes),
                regions,
                indices,
                bake_seconds: start.elapsed().as_secs_f64(),
            };
            fs::write(
                folder.join("manifest.json"),
                serde_json::to_vec_pretty(&manifest)?,
            )?;
        }
        let m: Manifest = serde_json::from_slice(
            &fs::read(folder.join("manifest.json")).context("run --bake-regional-scene first")?,
        )?;
        let bytes = fs::read(folder.join("surfaces.bin"))?;
        ensure!(
            m.signature == signature && m.checksum == hash(&bytes),
            "stale/corrupt regional cache for {name}"
        );
        ensure!(
            bytes.len() == m.regions.len() * 6 * FACE_TEXELS * 32,
            "invalid cache size"
        );
        let cache: Vec<Texel> = bytes
            .chunks_exact(32)
            .map(bytemuck::pod_read_unaligned)
            .collect();
        ensure!(
            cache
                .iter()
                .all(|t| t.color.iter().chain(&t.normal_depth).all(|v| v.is_finite())),
            "invalid cached values"
        );
        println!(
            "{name}: {} regions, {:0.2} MiB, {:0.2}s offline",
            m.regions.len(),
            bytes.len() as f64 / 1048576.,
            m.bake_seconds
        );
        assets.push(Asset {
            name: name.into(),
            vertices,
            indices: m.indices,
            regions: m.regions,
            cache_bytes: cache.len() * 32,
            cache,
            bake_seconds: m.bake_seconds,
        });
    }
    Ok(assets)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn vertex(p: [f32; 3], tile: u32) -> Vertex {
        Vertex {
            position: p,
            normal: [0., 0., 1.],
            color: [0.4, 0.6, 0.2],
            tile,
        }
    }
    #[test]
    fn partition_is_deterministic_and_owns_every_triangle_once() {
        let vs = [
            vertex([0., 0., 0.], 0),
            vertex([0.1, 0., 0.], 0),
            vertex([0., 0.1, 0.], 0),
            vertex([0.9, 0., 0.], 1),
            vertex([1., 0., 0.], 1),
            vertex([0.9, 0.1, 0.], 1),
        ];
        let (ids, rs) = partition(&vs, &[0, 1, 2, 3, 4, 5]);
        assert_eq!(ids.len(), 6);
        assert_eq!(rs.len(), 2);
        assert_eq!(ids, partition(&vs, &[0, 1, 2, 3, 4, 5]).0);
        let mut tris = ids.chunks_exact(3).map(|t| t.to_vec()).collect::<Vec<_>>();
        tris.sort();
        assert_eq!(tris, vec![vec![0, 1, 2], vec![3, 4, 5]]);
        for r in rs {
            for i in &ids[r.first as usize..(r.first + r.count) as usize] {
                let p = Vec3::from_array(vs[*i as usize].position);
                assert!(p.cmpge(r.min).all() && p.cmple(r.max).all());
                assert_eq!(vs[*i as usize].tile, r.material);
            }
        }
    }
    #[test]
    fn local_cache_uses_front_depth_and_preserves_premultiplied_mips() {
        let vs = [
            vertex([-0.1, -0.1, 0.1], 0),
            vertex([0.1, -0.1, 0.1], 0),
            vertex([0., 0.1, 0.1], 0),
            vertex([-0.1, -0.1, -0.1], 0),
            vertex([0.1, -0.1, -0.1], 0),
            vertex([0., 0.1, -0.1], 0),
        ];
        let r = Region {
            min: Vec3::splat(-0.1),
            max: Vec3::splat(0.1),
            center: Vec3::ZERO,
            radius: 0.12,
            edge: 0.2,
            first: 0,
            count: 6,
            material: 0,
        };
        let data = bake_region(&vs, &[0, 1, 2, 3, 4, 5], &r);
        assert_eq!(data.len(), 6 * FACE_TEXELS);
        let face = &data[4 * FACE_TEXELS..5 * FACE_TEXELS];
        let t = face[8 * 16 + 8];
        assert!(t.color[3] > 0.9);
        assert!((t.normal_depth[3] / t.color[3] - 0.1).abs() < 1e-5);
        let last = face[340];
        assert!(last.color[3] > 0. && last.color[3] < 1.);
        assert!((last.color[0] / last.color[3] - 0.4).abs() < 1e-5);
    }
}
