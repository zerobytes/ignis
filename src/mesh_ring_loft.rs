//! Explicit longitudinal interpolation of existing section loops. No spatial fitting.
use crate::{
    mesh_preservation,
    mesh_topology::Topology,
    meshes::{self, Asset},
    model::{Result, invalid},
};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Interpolation {
    Linear,
    Cubic,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Loft {
    pub rings: Vec<Vec<u32>>,
    pub spans: u8,
    pub interpolation: Interpolation,
    /// Bound on the whole polynomial's displacement from its linear section loft.
    pub max_deviation: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub added_vertices: usize,
    pub replaced_faces: Vec<u32>,
    pub curve_deviation_bound: f64,
    pub sampled_deviation: f64,
    pub source_vertices_exact: bool,
}
#[derive(Debug, Serialize)]
pub struct Fitted {
    pub asset: Asset,
    pub report: Report,
    /// Endpoints in canonical source for each appended vertex, in append order.
    #[serde(skip)]
    pub(crate) origins: Vec<[u32; 2]>,
}
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
fn key(mut t: [u32; 3]) -> [u32; 3] {
    t.sort_unstable();
    t
}
fn forward(t: [u32; 3], ordered: [u32; 3]) -> bool {
    (0..3).any(|j| (0..3).all(|k| t[(j + k) % 3] == ordered[k]))
}
fn tangent(points: &[DVec3], times: &[f64], i: usize) -> DVec3 {
    if i == 0 {
        return (points[1] - points[0]) / (times[1] - times[0]);
    }
    if i + 1 == points.len() {
        return (points[i] - points[i - 1]) / (times[i] - times[i - 1]);
    }
    let a = times[i] - times[i - 1];
    let b = times[i + 1] - times[i];
    ((points[i] - points[i - 1]) * (b / a) + (points[i + 1] - points[i]) * (a / b)) / (a + b)
}
struct Patch {
    columns: Vec<Vec<DVec3>>,
    derivatives: Vec<Vec<DVec3>>,
    times: Vec<f64>,
    signs: Vec<bool>,
    normals: Vec<DVec3>,
}
pub fn fit(source: &Asset, lofts: &[Loft]) -> Result<Fitted> {
    source.validate()?;
    if lofts.is_empty() || lofts.len() > 128 {
        return Err(invalid("Ring loft requires1..128 patches"));
    }
    let topology = Topology::inspect(&source.geometry)?;
    if topology.non_manifold_edges != 0 {
        return Err(invalid("Ring loft requires manifold source edges"));
    }
    let g = &source.geometry;
    let mut face_map = BTreeMap::new();
    for (i, &t) in g.triangles.iter().enumerate() {
        if face_map.insert(key(t), i as u32).is_some() {
            return Err(invalid("Ring loft source has duplicate faces"));
        }
    }
    let mut removed = BTreeSet::new();
    let mut patches = Vec::with_capacity(lofts.len());
    let mut added = 0usize;
    let mut new_faces = 0usize;
    let mut bound = 0f64;
    for loft in lofts {
        let rows = loft.rings.len();
        let sides = loft.rings.first().map_or(0, Vec::len);
        if !(2..=128).contains(&rows)
            || !(3..=32).contains(&sides)
            || !(1..=16).contains(&loft.spans)
            || !loft.max_deviation.is_finite()
            || !(0.0..=10.).contains(&loft.max_deviation)
        {
            return Err(invalid(
                "Ring loft requires2..128 rings,3..32 sides,1..16 spans and deviation0..10m",
            ));
        }
        let mut used = BTreeSet::new();
        for ring in &loft.rings {
            if ring.len() != sides
                || ring
                    .iter()
                    .any(|&i| i as usize >= g.positions.len() || !used.insert(i))
            {
                return Err(invalid(
                    "Ring loft rings need equal sides and unique existing vertices",
                ));
            }
        }
        added = added
            .checked_add((rows - 1) * sides * (loft.spans as usize - 1))
            .ok_or_else(|| invalid("Ring loft count overflow"))?;
        new_faces += 2 * (rows - 1) * sides * loft.spans as usize;
        if g.positions.len() + added > meshes::MAX_VERTICES || new_faces > meshes::MAX_TRIANGLES {
            return Err(invalid("Ring loft exceeds mesh vertex/triangle budget"));
        }
        let mut times = Vec::with_capacity(rows);
        times.push(0.);
        let centres: Vec<_> = loft
            .rings
            .iter()
            .map(|ring| {
                ring.iter()
                    .map(|&i| v(g.positions[i as usize]))
                    .sum::<DVec3>()
                    / sides as f64
            })
            .collect();
        for w in centres.windows(2) {
            let distance = w[0].distance(w[1]);
            if distance < 1e-7 {
                return Err(invalid("Ring loft section centres must be distinct"));
            }
            times.push(times.last().unwrap() + distance.sqrt());
        }
        let columns: Vec<Vec<_>> = (0..sides)
            .map(|j| {
                loft.rings
                    .iter()
                    .map(|r| v(g.positions[r[j] as usize]))
                    .collect()
            })
            .collect();
        let derivatives: Vec<Vec<_>> = if loft.interpolation == Interpolation::Cubic {
            columns
                .iter()
                .map(|p| (0..rows).map(|i| tangent(p, &times, i)).collect())
                .collect()
        } else {
            Vec::new()
        };
        if loft.interpolation == Interpolation::Cubic {
            for j in 0..sides {
                for i in 0..rows - 1 {
                    let delta = columns[j][i + 1] - columns[j][i];
                    let h = times[i + 1] - times[i];
                    let envelope = ((derivatives[j][i] * h - delta) / 3.)
                        .length()
                        .max(((delta - derivatives[j][i + 1] * h) / 3.).length());
                    if envelope > f64::from(loft.max_deviation) {
                        return Err(invalid("Ring loft polynomial exceeds deviation envelope"));
                    }
                    bound = bound.max(envelope);
                }
            }
        }
        let mut signs = Vec::with_capacity((rows - 1) * sides);
        let mut normals = Vec::with_capacity((rows - 1) * sides);
        for i in 0..rows - 1 {
            for j in 0..sides {
                let [a, b, c, d] = [
                    loft.rings[i][j],
                    loft.rings[i][(j + 1) % sides],
                    loft.rings[i + 1][j],
                    loft.rings[i + 1][(j + 1) % sides],
                ];
                let candidates = [[[a, b, c], [b, d, c]], [[a, b, d], [a, d, c]]];
                let mut found = None;
                for pair in candidates {
                    if let (Some(&f), Some(&h)) =
                        (face_map.get(&key(pair[0])), face_map.get(&key(pair[1])))
                    {
                        let sign = forward(g.triangles[f as usize], pair[0]);
                        if forward(g.triangles[h as usize], pair[1]) != sign {
                            return Err(invalid("Ring loft cell has inconsistent winding"));
                        }
                        found = Some(([f, h], sign));
                        break;
                    }
                }
                let (pair, sign) = found
                    .ok_or_else(|| invalid("Ring loft cell must match two source triangles"))?;
                if pair.iter().any(|&f| !removed.insert(f)) {
                    return Err(invalid("Ring loft patches overlap faces"));
                }
                signs.push(sign);
                normals.push(
                    pair.iter()
                        .map(|&f| {
                            let [a, b, c] =
                                g.triangles[f as usize].map(|id| v(g.positions[id as usize]));
                            (b - a).cross(c - a)
                        })
                        .sum(),
                );
            }
        }
        if signs.windows(2).any(|w| w[0] != w[1]) {
            return Err(invalid("Ring loft columns or rings reverse orientation"));
        }
        patches.push(Patch {
            columns,
            derivatives,
            times,
            signs,
            normals,
        });
    }
    let triangles = g.triangles.len() - removed.len() + new_faces;
    if triangles > meshes::MAX_TRIANGLES {
        return Err(invalid("Ring loft exceeds mesh triangle budget"));
    }
    let mut positions = Vec::with_capacity(g.positions.len() + added);
    positions.extend_from_slice(&g.positions);
    let material_coordinates = g.material_coordinates.as_ref().map(|source| {
        let mut coordinates = Vec::with_capacity(source.len() + added);
        coordinates.extend_from_slice(source);
        coordinates
    });
    let mut geometry = meshes::Geometry {
        positions,
        material_coordinates,
        triangles: Vec::with_capacity(triangles),
    };
    geometry.triangles.extend(
        g.triangles
            .iter()
            .enumerate()
            .filter_map(|(i, &t)| (!removed.contains(&(i as u32))).then_some(t)),
    );
    let mut rig = source.rig.as_ref().map(|r| {
        let mut weights = Vec::with_capacity(r.weights.len() + added);
        weights.extend_from_slice(&r.weights);
        meshes::Rig {
            bones: r.bones.clone(),
            weights,
        }
    });
    let mut sampled = 0f64;
    let mut origins = Vec::with_capacity(added);
    for (loft, p) in lofts.iter().zip(&patches) {
        let sides = loft.rings[0].len();
        let spans = loft.spans as usize;
        for i in 0..loft.rings.len() - 1 {
            let first = geometry.positions.len();
            let h = p.times[i + 1] - p.times[i];
            for k in 1..spans {
                let t = k as f64 / spans as f64;
                let u = 1. - t;
                for j in 0..sides {
                    let a = p.columns[j][i];
                    let b = p.columns[j][i + 1];
                    let linear = a.lerp(b, t);
                    let point = match loft.interpolation {
                        Interpolation::Linear => linear,
                        Interpolation::Cubic => {
                            a * (2. * t * t * t - 3. * t * t + 1.)
                                + b * (-2. * t * t * t + 3. * t * t)
                                + p.derivatives[j][i] * (h * (t * t * t - 2. * t * t + t))
                                + p.derivatives[j][i + 1] * (h * (t * t * t - t * t))
                        }
                    };
                    let point = point.to_array().map(|x| x as f32);
                    sampled = sampled.max(v(point).distance(linear));
                    geometry.positions.push(point);
                    let [a, b] = [loft.rings[i][j] as usize, loft.rings[i + 1][j] as usize];
                    origins.push([a as u32, b as u32]);
                    if let Some(c) = &mut geometry.material_coordinates {
                        let src = g.material_coordinates.as_ref().unwrap();
                        c.push((v(src[a]) * u + v(src[b]) * t).to_array().map(|x| x as f32));
                    }
                    if let Some(r) = &mut rig {
                        r.weights.push(mesh_preservation::transfer_weight(
                            source.rig.as_ref().unwrap(),
                            [a, b, a],
                            [u, t, 0.],
                        )?);
                    }
                }
            }
            let index = |k: usize, j: usize| -> u32 {
                if k == 0 {
                    loft.rings[i][j]
                } else if k == spans {
                    loft.rings[i + 1][j]
                } else {
                    (first + (k - 1) * sides + j) as u32
                }
            };
            for k in 0..spans {
                for j in 0..sides {
                    let [a, b, c, d] = [
                        index(k, j),
                        index(k, (j + 1) % sides),
                        index(k + 1, j),
                        index(k + 1, (j + 1) % sides),
                    ];
                    for mut tri in [[a, b, c], [b, d, c]] {
                        if !p.signs[i * sides + j] {
                            tri.swap(1, 2);
                        }
                        let [x, y, z] = tri.map(|id| v(geometry.positions[id as usize]));
                        let n = (y - x).cross(z - x);
                        let reference = p.normals[i * sides + j];
                        if n.length_squared() < 1e-20
                            || n.dot(reference) <= 1e-12 * n.length() * reference.length()
                        {
                            return Err(invalid("Ring loft collapses or reverses a source cell"));
                        }
                        geometry.triangles.push(tri);
                    }
                }
            }
        }
    }
    let mut out = source.clone();
    out.geometry = Arc::new(geometry);
    out.rig = rig.map(Arc::new);
    out.validate()?;
    Ok(Fitted {
        asset: out,
        report: Report {
            added_vertices: added,
            replaced_faces: removed.into_iter().collect(),
            curve_deviation_bound: bound,
            sampled_deviation: sampled,
            source_vertices_exact: true,
        },
        origins,
    })
}
