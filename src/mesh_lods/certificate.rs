//! Conservative surface/normal correspondence bounds. Candidate generation is not trusted.
//! Distance to ONE convex triangle is convex, so its maximum on a source triangle
//! is bounded by the three vertex distances. Subdivision tightens this union bound.
use super::V;
use crate::model::{Result, invalid};
use glam::DVec3;
use rapier3d::{
    na::Point3,
    parry::{
        query::PointQuery,
        shape::{FeatureId, TriMesh},
    },
};
use std::collections::BTreeMap;
struct Surface {
    v: Vec<V>,
    t: Vec<[u32; 3]>,
    bvh: TriMesh,
    adjacent: Vec<Vec<usize>>,
}
impl Surface {
    fn new(vertices: &[V], triangles: &[[u32; 3]]) -> Result<Self> {
        let mut remap = BTreeMap::new();
        let mut v = Vec::new();
        let mut t = Vec::new();
        for face in triangles {
            let mut out = [0; 3];
            for (i, &id) in face.iter().enumerate() {
                out[i] = *remap.entry(id).or_insert_with(|| {
                    let n = v.len() as u32;
                    v.push(vertices[id as usize]);
                    n
                });
            }
            t.push(out);
        }
        let points = v
            .iter()
            .map(|v| Point3::new(v.p.x as f32, v.p.y as f32, v.p.z as f32))
            .collect();
        let bvh = TriMesh::new(points, t.clone()).map_err(|e| invalid(e.to_string()))?;
        let mut adjacent = vec![Vec::new(); v.len()];
        for (i, f) in t.iter().enumerate() {
            for &n in f {
                adjacent[n as usize].push(i);
            }
        }
        Ok(Self {
            v,
            t,
            bvh,
            adjacent,
        })
    }
    fn nearest(&self, p: DVec3) -> usize {
        let (_, feature) = self
            .bvh
            .project_local_point_and_get_feature(&Point3::new(p.x as f32, p.y as f32, p.z as f32));
        match feature {
            FeatureId::Face(i) => i as usize,
            _ => 0,
        }
    }
}
fn closest(p: DVec3, t: [V; 3]) -> V {
    let [a, b, c] = t;
    let e = b.p - a.p;
    let f = c.p - a.p;
    let d = p - a.p;
    let (ee, ff, ef) = (e.dot(e), f.dot(f), e.dot(f));
    let determinant = ee * ff - ef * ef;
    if determinant > 1e-30 {
        let u = (d.dot(e) * ff - d.dot(f) * ef) / determinant;
        let v = (d.dot(f) * ee - d.dot(e) * ef) / determinant;
        if u >= 0. && v >= 0. && u + v <= 1. {
            return V {
                p: a.p + e * u + f * v,
                n: a.n + (b.n - a.n) * u + (c.n - a.n) * v,
            };
        }
    }
    let mut best = a;
    let mut distance = p.distance_squared(a.p);
    for (a, b) in [(a, b), (b, c), (c, a)] {
        let e = b.p - a.p;
        let u = ((p - a.p).dot(e) / e.length_squared().max(1e-30)).clamp(0., 1.);
        let q = V {
            p: a.p + e * u,
            n: a.n + (b.n - a.n) * u,
        };
        let dist = p.distance_squared(q.p);
        if dist < distance {
            distance = dist;
            best = q;
        }
    }
    best
}
fn correspondence(t: [V; 3], other: [V; 3]) -> (f64, f64) {
    let projected = t.map(|v| closest(v.p, other));
    let error = t
        .iter()
        .zip(projected)
        .map(|(v, q)| v.p.distance(q.p))
        .fold(0., f64::max);
    let raw = t
        .iter()
        .zip(projected)
        .map(|(v, q)| (v.n - q.n).length())
        .fold(0., f64::max);
    // One common positive half-space bounds the ENTIRE segment between both
    // interpolated normals away from zero. Normalize's Jacobian norm is1/r,
    // so integrating along that segment gives raw/min (not2*raw/min).
    let axis = t
        .iter()
        .chain(&projected)
        .map(|v| v.n)
        .sum::<DVec3>()
        .normalize_or_zero();
    let min = t
        .iter()
        .chain(&projected)
        .map(|v| v.n.dot(axis))
        .fold(1., f64::min);
    let normal = if min > 1e-6 && raw.is_finite() {
        (raw / min).min(2.)
    } else {
        2.
    };
    (error, normal)
}
fn node(
    t: [V; 3],
    target: &Surface,
    depth: u8,
    goal: (f64, f64),
    budget: &mut usize,
) -> Result<(f64, f64)> {
    *budget = budget
        .checked_sub(1)
        .ok_or_else(|| invalid("LOD certificate work budget exceeded"))?;
    let center = V {
        p: (t[0].p + t[1].p + t[2].p) / 3.,
        n: (t[0].n + t[1].n + t[2].n) / 3.,
    };
    let nearest = target.nearest(center.p);
    let mut result = correspondence(t, target.t[nearest].map(|i| target.v[i as usize]));
    let score = |r: (f64, f64)| (r.0 / goal.0).max(r.1 / goal.1);
    // A geometric nearest triangle can have an unrelated normal at a seam or
    // junction. Any convex target triangle gives a valid bound; inspect nearby
    // topology and retain the strongest proved correspondence, never a sample
    // error masquerading as a bound. Cap work; nearest remains a safe fallback.
    let mut tried = 0;
    if score(result) > 1. {
        'neighbors: for &v in &target.t[nearest] {
            for &face in &target.adjacent[v as usize] {
                if face == nearest {
                    continue;
                }
                let r = correspondence(t, target.t[face].map(|i| target.v[i as usize]));
                if score(r) < score(result) {
                    result = r;
                }
                tried += 1;
                if tried >= 32 || score(result) <= 1. {
                    break 'neighbors;
                }
            }
        }
    }
    let (error, normal) = result;
    // Map the source corners to three points of ONE convex target triangle.
    // Barycentric interpolation defines an affine correspondence throughout the
    // source triangle. Both positional and raw-normal differences are affine,
    // so their norm is bounded by the three corner differences. This does not
    // require the interior mapping to be nearest-point projection.
    if depth == 4 || (error <= goal.0 && normal <= goal.1) {
        return Ok((error, normal));
    }
    let mid = |a: V, b: V| V {
        p: (a.p + b.p) * 0.5,
        n: (a.n + b.n) * 0.5,
    };
    let [a, b, c] = t;
    let (ab, bc, ca) = (mid(a, b), mid(b, c), mid(c, a));
    let mut result = (0f64, 0f64);
    for child in [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]] {
        let p = node(child, target, depth + 1, goal, budget)?;
        result.0 = result.0.max(p.0);
        result.1 = result.1.max(p.1);
    }
    Ok(result)
}
pub(super) fn bounds(
    vertices: &[V],
    a: &[[u32; 3]],
    b: &[[u32; 3]],
    position_goal: f64,
    normal_goal: f64,
    budget: &mut usize,
) -> Result<(f32, f32)> {
    if a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            a.iter().zip(b).all(|(&a, &b)| {
                vertices[a as usize].p == vertices[b as usize].p
                    && vertices[a as usize].n == vertices[b as usize].n
            })
        })
    {
        return Ok((0., 0.));
    }
    let source = Surface::new(vertices, a)?;
    let target = Surface::new(vertices, b)?;
    let mut result = (0f64, 0f64);
    for (s, t) in [(&source, &target), (&target, &source)] {
        for f in &s.t {
            let e = node(
                f.map(|i| s.v[i as usize]),
                t,
                0,
                (position_goal, normal_goal),
                budget,
            )?;
            result.0 = result.0.max(e.0);
            result.1 = result.1.max(e.1);
        }
    }
    let magnitude = a
        .iter()
        .chain(b)
        .flatten()
        .map(|&i| vertices[i as usize].p.abs().max_element())
        .fold(1., f64::max);
    Ok((
        ((result.0 + 16. * f32::EPSILON as f64 * magnitude) as f32).next_up(),
        ((result.1 + 0.00001).min(2.) as f32).next_up().min(2.),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Critical certificate boundary: an underestimated baked normal bound can
    // admit a visually invalid sidecar. Exercise normalized interpolated fields,
    // including cancellation where a positive common cone does not exist.
    #[test]
    fn bound_contains_interpolated_normal_field_and_fails_open_at_cancellation() {
        let points = [DVec3::ZERO, DVec3::X, DVec3::Y];
        let normals = [
            DVec3::new(0.1, 0.2, 1.),
            DVec3::new(-0.2, 0.1, 1.),
            DVec3::new(0.2, -0.1, 1.),
        ]
        .map(DVec3::normalize);
        let source = std::array::from_fn(|i| V {
            p: points[i],
            n: normals[i],
        });
        let rotation = glam::DQuat::from_rotation_y(0.17);
        let target = source.map(|v| V {
            n: rotation * v.n,
            ..v
        });
        let (_, bound) = correspondence(source, target);
        assert!(bound < 0.3);
        for x in 0..=32 {
            for y in 0..=32 - x {
                let weights = [x as f64 / 32., y as f64 / 32., (32 - x - y) as f64 / 32.];
                let interpolate = |t: [V; 3]| {
                    t.into_iter()
                        .zip(weights)
                        .map(|(v, w)| v.n * w)
                        .sum::<DVec3>()
                        .normalize()
                };
                assert!(interpolate(source).distance(interpolate(target)) <= bound + 1e-12);
            }
        }
        assert_eq!(
            correspondence(source, source.map(|v| V { n: -v.n, ..v })).1,
            2.
        );
    }
}
