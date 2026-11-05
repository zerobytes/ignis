//! Offline certificate hierarchy. Geometry and the original IGC1 remain unchanged.
use super::{
    compiler::{self, Certificate, Compiled},
    render::{Bound, Pose},
};
use anyhow::{Result, ensure};
use glam::Vec3;
use serde::Serialize;
const LEAF: usize = 32;
const MAX_REGIONS: usize = 2_000_000;
const BUDGET: usize = 128;
#[derive(Clone, Copy)]
struct Node {
    bounds: [f32; 8], // low.xyz, position error, high.xyz, normal error
    end: u32,         // first node outside this subtree (stackless preorder)
    count: u32,       // number of certificate IDs in a leaf; zero in branches
}
impl Node {
    fn bound(self) -> Bound {
        Bound {
            low: Vec3::from_slice(&self.bounds[..3]),
            high: Vec3::from_slice(&self.bounds[4..7]),
            error: self.bounds[3],
            normal_error: self.bounds[7],
            reduces: true,
        }
    }
}
pub struct Hierarchy {
    nodes: Vec<Node>,
    reduces: bool,
}
#[derive(Serialize, Clone, Copy, Debug)]
pub struct Decision {
    pub proxy: bool,
    pub visits: usize,
    pub accepted_error_px: f32,
}
fn aggregate(regions: &[Certificate], ids: &[u32]) -> [f32; 8] {
    let mut a = [
        f32::INFINITY,
        f32::INFINITY,
        f32::INFINITY,
        0.,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
        0.,
    ];
    for &id in ids {
        let r = &regions[id as usize];
        for j in 0..3 {
            a[j] = a[j].min(r.low[j]);
            a[j + 4] = a[j + 4].max(r.high[j]);
        }
        a[3] = a[3].max(r.error);
        a[7] = a[7].max(r.normal_error);
    }
    a
}
fn join(a: [f32; 8], b: [f32; 8]) -> [f32; 8] {
    std::array::from_fn(|j| {
        if j < 3 {
            a[j].min(b[j])
        } else {
            a[j].max(b[j])
        }
    })
}
fn partition(regions: &[Certificate], ids: &mut [u32], nodes: &mut Vec<Node>) {
    let bounds = aggregate(regions, ids);
    let index = nodes.len();
    nodes.push(Node {
        bounds,
        end: 0,
        count: 0,
    });
    if ids.len() > LEAF {
        let axis = (0..3)
            .max_by(|&a, &b| (bounds[a + 4] - bounds[a]).total_cmp(&(bounds[b + 4] - bounds[b])))
            .unwrap();
        let middle = ids.len() / 2;
        ids.select_nth_unstable_by(middle, |&a, &b| {
            let a_region = &regions[a as usize];
            let b_region = &regions[b as usize];
            (a_region.low[axis] as f64 + a_region.high[axis] as f64)
                .total_cmp(&(b_region.low[axis] as f64 + b_region.high[axis] as f64))
                .then(a.cmp(&b))
        });
        let (left, right) = ids.split_at_mut(middle);
        partition(regions, left, nodes);
        partition(regions, right, nodes);
    } else {
        nodes[index].count = ids.len() as u32;
    }
    nodes[index].end = nodes.len() as u32;
}
/// Build once, serialize canonical LE fields; no platform-dependent struct layout.
pub fn build(c: &Compiled, package_hash: u64) -> Result<Vec<u8>> {
    ensure!(
        !c.regions.is_empty() && c.regions.len() <= MAX_REGIONS,
        "region limit"
    );
    let mut ids = (0..c.regions.len() as u32).collect::<Vec<_>>();
    let mut nodes = Vec::new();
    partition(&c.regions, &mut ids, &mut nodes);
    let mut payload = Vec::new();
    for n in &nodes {
        for v in n.bounds {
            payload.extend(v.to_le_bytes());
        }
        payload.extend(n.end.to_le_bytes());
        payload.extend(n.count.to_le_bytes());
    }
    for id in ids {
        payload.extend(id.to_le_bytes());
    }
    let mut bytes = b"ICH1".to_vec();
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(package_hash.to_le_bytes());
    bytes.extend(compiler::hash(&payload).to_le_bytes());
    bytes.extend((nodes.len() as u32).to_le_bytes());
    bytes.extend((c.regions.len() as u32).to_le_bytes());
    bytes.extend(payload);
    Hierarchy::load(&bytes, c, package_hash)?;
    Ok(bytes)
}
impl Hierarchy {
    pub fn load(bytes: &[u8], c: &Compiled, package_hash: u64) -> Result<Self> {
        ensure!(bytes.len() >= 32 && &bytes[..4] == b"ICH1", "ICH header");
        let u32_at = |i| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
        let u64_at = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        ensure!(
            u32_at(4) == 1 && u64_at(8) == package_hash,
            "ICH version/package mismatch"
        );
        let n = u32_at(24) as usize;
        let r = u32_at(28) as usize;
        ensure!(
            r > 0 && r <= MAX_REGIONS && r == c.regions.len() && n > 0 && n <= r * 2,
            "ICH counts"
        );
        ensure!(
            bytes.len() == 32 + n * 40 + r * 4 && compiler::hash(&bytes[32..]) == u64_at(16),
            "ICH length/checksum"
        );
        let nodes = (0..n)
            .map(|i| {
                let p = 32 + i * 40;
                Node {
                    bounds: std::array::from_fn(|j| f32::from_bits(u32_at(p + j * 4))),
                    end: u32_at(p + 32),
                    count: u32_at(p + 36),
                }
            })
            .collect::<Vec<_>>();
        let ids = (0..r)
            .map(|i| u32_at(32 + n * 40 + i * 4))
            .collect::<Vec<_>>();
        let mut seen = vec![false; r];
        for &id in &ids {
            ensure!((id as usize) < r && !seen[id as usize], "ICH permutation");
            seen[id as usize] = true;
        }
        fn validate(
            i: usize,
            depth: usize,
            nodes: &[Node],
            ids: &[u32],
            cursor: &mut usize,
            c: &Compiled,
        ) -> Result<usize> {
            ensure!(i < nodes.len() && depth <= 32, "ICH topology/depth");
            let node = nodes[i];
            let end = node.end as usize;
            ensure!(
                end > i && end <= nodes.len() && node.bounds.iter().all(|x| x.is_finite()),
                "ICH bounds/end"
            );
            let expected = if node.count > 0 {
                let count = node.count as usize;
                ensure!(
                    count <= LEAF && end == i + 1 && *cursor + count <= ids.len(),
                    "ICH leaf"
                );
                let a = aggregate(&c.regions, &ids[*cursor..*cursor + count]);
                *cursor += count;
                a
            } else {
                let right = validate(i + 1, depth + 1, nodes, ids, cursor, c)?;
                ensure!(right < end, "ICH missing child");
                ensure!(
                    validate(right, depth + 1, nodes, ids, cursor, c)? == end,
                    "ICH subtree"
                );
                join(nodes[i + 1].bounds, nodes[right].bounds)
            };
            ensure!(expected == node.bounds, "ICH certificate bound mismatch");
            Ok(end)
        }
        let mut cursor = 0;
        ensure!(
            validate(0, 0, &nodes, &ids, &mut cursor, c)? == n && cursor == r,
            "ICH unreferenced data"
        );
        Ok(Self {
            nodes,
            reduces: c.proxy.faces.len() < c.source.faces.len(),
        })
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn resident_bytes(&self) -> usize {
        self.nodes.len() * std::mem::size_of::<Node>()
    }
    pub fn decide(&self, p: Pose, height: u32) -> Decision {
        self.decide_budget(p, height, BUDGET)
    }
    fn decide_budget(&self, p: Pose, height: u32, budget: usize) -> Decision {
        let mut d = Decision {
            proxy: false,
            visits: 0,
            accepted_error_px: 0.,
        };
        if !self.reduces
            || !p.angle.is_finite()
            || !p.distance.is_finite()
            || p.distance <= 0.
            || height == 0
        {
            return d;
        }
        let mut i = 0;
        while i < self.nodes.len() {
            if d.visits >= budget {
                return d;
            }
            let n = self.nodes[i];
            let b = n.bound();
            d.visits += 1;
            let error = b.pixels(p, height);
            if b.normal_error <= 0.000101 && error <= 0.15 {
                d.accepted_error_px = d.accepted_error_px.max(error);
                i = n.end as usize;
            } else if n.count > 0 {
                return d;
            } else {
                i += 1;
            }
        }
        d.proxy = true;
        d
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Compiled {
        let vertex = compiler::Vertex {
            p: [0.; 3],
            n: [0., 0., 1.],
        };
        let mesh = compiler::Mesh {
            vertices: vec![vertex],
            faces: vec![[0; 4]; 128],
        };
        let proxy = compiler::Mesh {
            vertices: vec![vertex],
            faces: vec![[0; 4]; 1],
        };
        let regions = (0..128)
            .map(|i| {
                let x = (i % 8) as f32 * 0.1 - 0.4;
                let y = (i / 8) as f32 * 0.05 - 0.4;
                Certificate {
                    low: [x, y, -0.01],
                    high: [x + 0.01, y + 0.01, 0.01],
                    error: 0.00004,
                    normal_error: 0.00001,
                    counts: [1, 1, 0, 1],
                }
            })
            .collect();
        Compiled {
            source: mesh,
            proxy,
            regions,
            source_hash: 0,
        }
    }
    #[test]
    fn hierarchy_never_accepts_a_failing_certificate() -> Result<()> {
        let c = fixture();
        let bytes = build(&c, 123)?;
        let h = Hierarchy::load(&bytes, &c, 123)?;
        let mut accepted = 0;
        let mut fallback = 0;
        for distance in [0.05, 0.7, 1., 2.7, 64.] {
            for i in 0..64 {
                for height in [1117, 2234, 10000] {
                    let p = Pose {
                        angle: i as f32 * 0.1,
                        distance,
                        light: 0.,
                    };
                    let d = h.decide(p, height);
                    if d.proxy {
                        accepted += 1;
                        for r in &c.regions {
                            let b = Node {
                                bounds: [
                                    r.low[0],
                                    r.low[1],
                                    r.low[2],
                                    r.error,
                                    r.high[0],
                                    r.high[1],
                                    r.high[2],
                                    r.normal_error,
                                ],
                                end: 0,
                                count: 0,
                            }
                            .bound();
                            assert!(b.choose(p, height));
                        }
                    } else {
                        fallback += 1;
                    }
                    assert!(d.visits <= BUDGET);
                }
            }
        }
        assert!(accepted > 0 && fallback > 0);
        assert!(
            !h.decide_budget(
                Pose {
                    angle: 0.,
                    distance: 2.7,
                    light: 0.
                },
                2234,
                0
            )
            .proxy
        );
        assert!(
            !h.decide(
                Pose {
                    angle: f32::NAN,
                    distance: 2.7,
                    light: 0.
                },
                2234
            )
            .proxy
        );
        assert_eq!(h.resident_bytes(), h.node_count() * 40);
        Ok(())
    }
    #[test]
    fn exhausted_budget_cannot_accept_partial_coverage() -> Result<()> {
        let c = fixture();
        let bytes = build(&c, 123)?;
        let h = Hierarchy::load(&bytes, &c, 123)?;
        let mut refined = None;
        for i in 0..500 {
            let p = Pose {
                angle: 0.7,
                distance: 0.5 + i as f32 * 0.01,
                light: 0.,
            };
            let d = h.decide(p, 2234);
            if d.proxy && d.visits > 1 {
                refined = Some((p, d.visits));
                break;
            }
        }
        let (p, visits) = refined.expect("fixture must exercise successful refinement");
        let d = h.decide_budget(p, 2234, visits - 1);
        assert!(!d.proxy);
        assert_eq!(d.visits, visits - 1);
        Ok(())
    }
    #[test]
    fn unsafe_normals_and_nonreducing_meshes_keep_source() -> Result<()> {
        let mut c = fixture();
        c.regions[7].normal_error = 0.01;
        let bytes = build(&c, 123)?;
        let h = Hierarchy::load(&bytes, &c, 123)?;
        assert!(
            !h.decide(
                Pose {
                    angle: 0.,
                    distance: 64.,
                    light: 0.
                },
                2234
            )
            .proxy
        );
        c.regions[7].normal_error = 0.;
        c.proxy.faces = c.source.faces.clone();
        let bytes = build(&c, 123)?;
        let h = Hierarchy::load(&bytes, &c, 123)?;
        assert!(
            !h.decide(
                Pose {
                    angle: 0.,
                    distance: 64.,
                    light: 0.
                },
                2234
            )
            .proxy
        );
        Ok(())
    }
    #[test]
    fn rejects_corruption_and_wrong_package_even_with_recomputed_checksum() -> Result<()> {
        let c = fixture();
        let bytes = build(&c, 123)?;
        assert!(Hierarchy::load(&bytes, &c, 124).is_err());
        for length in [0, 4, 31, bytes.len() - 1] {
            assert!(Hierarchy::load(&bytes[..length], &c, 123).is_err());
        }
        // Root error, invalid escape, NaN, non-permutation IDs, unsupported version.
        for offset in [32 + 12, 32 + 32, 32, bytes.len() - 4, 4] {
            let mut bad = bytes.clone();
            bad[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            let sum = compiler::hash(&bad[32..]);
            bad[16..24].copy_from_slice(&sum.to_le_bytes());
            assert!(Hierarchy::load(&bad, &c, 123).is_err(), "offset {offset}");
        }
        let mut bad = bytes.clone();
        bad[44..48].copy_from_slice(&0f32.to_le_bytes());
        let sum = compiler::hash(&bad[32..]);
        bad[16..24].copy_from_slice(&sum.to_le_bytes());
        assert!(Hierarchy::load(&bad, &c, 123).is_err());
        Ok(())
    }
}

pub fn build_directory(input: &std::path::Path, output: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(output)?;
    let mut reports = vec![];
    for factor in [1, 2, 10, 100] {
        let package = std::fs::read(input.join(format!("stable-{factor}.igc")))?;
        let c = compiler::decode(&package)?;
        let hash = compiler::hash(&package);
        let start = std::time::Instant::now();
        let bytes = build(&c, hash)?;
        let build_ms = start.elapsed().as_secs_f64() * 1000.;
        let start = std::time::Instant::now();
        let h = Hierarchy::load(&bytes, &c, hash)?;
        let load_ms = start.elapsed().as_secs_f64() * 1000.;
        std::fs::write(output.join(format!("certificates-{factor}.ich")), &bytes)?;
        let bound = Bound::new(&c);
        let mut audit = vec![];
        for distance in [0.05, 0.7, 1., 1.5, 2., 2.7, 4., 64.] {
            for height in [1117, 2234] {
                for i in 0..64 {
                    let p = Pose {
                        angle: i as f32 * std::f32::consts::TAU / 64.,
                        distance,
                        light: 0.,
                    };
                    let d = h.decide(p, height);
                    // Independent exhaustive traversal verifies that pruning never hides a failing region.
                    let local = c.regions.iter().all(|r| {
                        Bound {
                            low: Vec3::from_array(r.low),
                            high: Vec3::from_array(r.high),
                            error: r.error,
                            normal_error: r.normal_error,
                            reduces: true,
                        }
                        .choose(p, height)
                    });
                    ensure!(
                        !d.proxy || local,
                        "hierarchy accepted a failing local certificate"
                    );
                    audit.push(serde_json::json!({"distance":distance,"height":height,"frame":i,"global":bound.choose(p,height),"all_local":local,"decision":d}));
                }
            }
        }
        reports.push(serde_json::json!({"factor":factor,"package_hash":format!("{hash:016x}"),"source_triangles":c.source.faces.len(),"proxy_triangles":c.proxy.faces.len(),"regions":c.regions.len(),"nodes":h.node_count(),"file_bytes":bytes.len(),"resident_bytes":h.resident_bytes(),"build_ms":build_ms,"load_ms":load_ms,"audit":audit}));
        println!(
            "factor{factor}: {} nodes, {} resident bytes, build {build_ms:.3}ms",
            h.node_count(),
            h.resident_bytes()
        );
    }
    std::fs::write(
        output.join("certificates.json"),
        serde_json::to_vec_pretty(&reports)?,
    )?;
    Ok(())
}
