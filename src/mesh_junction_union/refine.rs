use super::{Options, v};
use crate::{
    mesh_topology::Topology,
    meshes::Geometry,
    model::{Result, invalid},
};
use glam::DVec3;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub seam_steps: u8,
    pub seam_selected_vertices: usize,
    pub max_seam_displacement: f64,
    pub signed_volume_before: f64,
    pub signed_volume_after: f64,
    pub material_steps: u8,
    pub material_guard_jump_before: f64,
    pub material_guard_jump_after: f64,
    pub material_residual_max: f64,
}
#[derive(Clone, Copy)]
pub(super) struct Angles {
    pub max: f64,
    pub mean: f64,
    pub source: f64,
    pub extracted: f64,
    pub interior: f64,
}
fn face_normals(g: &Geometry) -> Vec<DVec3> {
    g.triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| v(g.positions[i as usize]));
            (b - a).cross(c - a)
        })
        .collect()
}
pub(super) fn angles(g: &Geometry, topo: &Topology, old: usize, iso: usize) -> Result<Angles> {
    let ns: Vec<_> = face_normals(g).into_iter().map(DVec3::normalize).collect();
    let mut r = Angles {
        max: 0.,
        mean: 0.,
        source: 0.,
        extracted: 0.,
        interior: 0.,
    };
    let mut count = 0;
    for e in &topo.edges {
        if e.faces.len() != 2 || e.faces.iter().all(|&i| (i as usize) < old) {
            continue;
        }
        let [a, b] = [e.faces[0] as usize, e.faces[1] as usize];
        let cosine = ns[a].dot(ns[b]);
        if !cosine.is_finite() || cosine <= 0. {
            return Err(invalid("Union adjacent triangle bend reaches90 degrees"));
        }
        let angle = cosine.clamp(-1., 1.).acos().to_degrees();
        if a.min(b) < old {
            r.source = r.source.max(angle);
        } else if a.min(b) < iso && a.max(b) >= iso {
            r.extracted = r.extracted.max(angle);
        } else {
            r.interior = r.interior.max(angle);
        }
        r.max = r.max.max(angle);
        r.mean += angle;
        count += 1;
    }
    r.mean /= count as f64;
    Ok(r)
}
fn volume(g: &Geometry) -> f64 {
    g.triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| v(g.positions[i as usize]));
            a.dot(b.cross(c)) / 6.
        })
        .sum()
}
struct Graph {
    offsets: Vec<usize>,
    adjacent: Vec<usize>,
}
impl Graph {
    fn new(topo: &Topology, count: usize) -> Self {
        let mut offsets = vec![0; count + 1];
        for e in &topo.edges {
            for i in e.vertices {
                offsets[i as usize + 1] += 1;
            }
        }
        for i in 1..=count {
            offsets[i] += offsets[i - 1];
        }
        let mut adjacent = vec![0; offsets[count]];
        let mut next = offsets[..count].to_vec();
        for e in &topo.edges {
            let [a, b] = e.vertices.map(|i| i as usize);
            adjacent[next[a]] = b;
            next[a] += 1;
            adjacent[next[b]] = a;
            next[b] += 1;
        }
        Self { offsets, adjacent }
    }
    fn neighbors(&self, i: usize) -> &[usize] {
        &self.adjacent[self.offsets[i]..self.offsets[i + 1]]
    }
    fn average(&self, p: &[DVec3], i: usize) -> DVec3 {
        let ns = self.neighbors(i);
        ns.iter().map(|&j| p[j]).sum::<DVec3>() / ns.len() as f64
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn apply(
    g: &mut Geometry,
    topo: &Topology,
    old: usize,
    iso: usize,
    old_faces: usize,
    iso_faces: usize,
    guards: &[(u32, u32)],
    options: &Options,
) -> Result<(Report, Angles)> {
    let before = angles(g, topo, old_faces, iso_faces)?;
    let mut final_angles = before;
    let initial_volume = volume(g);
    let mut r = Report {
        seam_steps: options.seam_relaxation_steps,
        seam_selected_vertices: 0,
        max_seam_displacement: 0.,
        signed_volume_before: initial_volume,
        signed_volume_after: initial_volume,
        material_steps: options.material_relaxation_steps,
        material_guard_jump_before: 0.,
        material_guard_jump_after: 0.,
        material_residual_max: 0.,
    };
    if options.seam_relaxation_steps == 0 && options.material_relaxation_steps == 0 {
        return Ok((r, before));
    }
    let graph = Graph::new(topo, g.positions.len());
    if options.seam_relaxation_steps > 0 {
        let original: Vec<_> = g.positions.iter().copied().map(v).collect();
        let original_normals = face_normals(g);
        let mut points = original.clone();
        let mut distance = vec![u8::MAX; points.len()];
        for t in &g.triangles[iso_faces..] {
            for &i in t {
                let i = i as usize;
                if (old..iso).contains(&i) {
                    distance[i] = 0;
                }
            }
        }
        for ring in 1..3 {
            // Each pass only reads the completed preceding ring.
            for i in old..iso {
                if distance[i] == u8::MAX
                    && graph.neighbors(i).iter().any(|&j| distance[j] == ring - 1)
                {
                    distance[i] = ring;
                }
            }
        }
        let selected: Vec<_> = (old..iso).filter(|&i| distance[i] < 3).collect();
        r.seam_selected_vertices = selected.len();
        let mut updates = vec![DVec3::ZERO; selected.len()];
        let limit = f64::from(options.seam_max_displacement);
        for _ in 0..options.seam_relaxation_steps {
            for (&i, target) in selected.iter().zip(&mut updates) {
                let factor = [1., 0.65, 0.25][distance[i] as usize] * 0.3;
                let next = points[i] + (graph.average(&points, i) - points[i]) * factor;
                let delta = next - original[i];
                *target = original[i] + delta * (limit / delta.length().max(limit)).min(1.);
            }
            for (&i, &point) in selected.iter().zip(&updates) {
                g.positions[i] = point.to_array().map(|x| x as f32);
                points[i] = v(g.positions[i]);
                // Account for f32 quantization without claiming an exact real envelope.
                if points[i].distance(original[i]) > limit + 1e-6 {
                    return Err(invalid(
                        "Union seam relaxation exceeds displacement envelope",
                    ));
                }
            }
            for (t, n0) in g.triangles[old_faces..]
                .iter()
                .zip(&original_normals[old_faces..])
            {
                let [a, b, c] = t.map(|i| points[i as usize]);
                let n = (b - a).cross(c - a);
                if !n.is_finite() || n.length_squared() < 1e-24 || n.dot(*n0) <= 0. {
                    return Err(invalid(
                        "Union seam relaxation reverses or degenerates a face",
                    ));
                }
            }
        }
        let after = angles(g, topo, old_faces, iso_faces)?;
        if after.max > before.max + 1e-6 || after.mean > before.mean + 1e-6 {
            return Err(invalid(
                "Union seam relaxation worsens the maximum or mean bend",
            ));
        }
        final_angles = after;
        r.max_seam_displacement = selected
            .iter()
            .map(|&i| points[i].distance(original[i]))
            .fold(0., f64::max);
        r.signed_volume_after = volume(g);
    }
    if options.material_relaxation_steps > 0 {
        let coords = g
            .material_coordinates
            .as_mut()
            .ok_or_else(|| invalid("Union material relaxation requires explicit coordinates"))?;
        let base: Vec<_> = g
            .positions
            .iter()
            .map(|&p| v(p) + v(options.material_offset))
            .collect();
        let mut corrections = vec![DVec3::ZERO; base.len()];
        for &(guard, source) in guards {
            let (guard, source) = (guard as usize, source as usize);
            r.material_guard_jump_before = r
                .material_guard_jump_before
                .max(v(coords[guard]).distance(v(coords[source])));
            coords[guard] = coords[source];
            corrections[guard] = v(coords[source]) - base[guard];
        }
        // Solve only the material correction, never geometry. The fixed
        // guards remain outside the unknown vector, including zero RHS cases.
        let count = iso - old;
        let mut x = vec![DVec3::ZERO; count];
        let mut residual = vec![DVec3::ZERO; count];
        for i in old..iso {
            residual[i - old] = graph
                .neighbors(i)
                .iter()
                .filter(|&&j| !(old..iso).contains(&j))
                .map(|&j| corrections[j])
                .sum();
        }
        let mut direction = residual.clone();
        let mut product = vec![DVec3::ZERO; count];
        let mut norm = residual.iter().map(|p| p.length_squared()).sum::<f64>();
        for _ in 0..options.material_relaxation_steps {
            if norm < 1e-28 {
                break;
            }
            for i in old..iso {
                let ns = graph.neighbors(i);
                product[i - old] = direction[i - old] * ns.len() as f64
                    - ns.iter()
                        .filter(|&&j| (old..iso).contains(&j))
                        .map(|&j| direction[j - old])
                        .sum::<DVec3>();
            }
            let denominator = direction
                .iter()
                .zip(&product)
                .map(|(a, b)| a.dot(*b))
                .sum::<f64>();
            if !denominator.is_finite() || denominator <= 0. {
                return Err(invalid(
                    "Union material correction has unresolved Dirichlet system",
                ));
            }
            let alpha = norm / denominator;
            for i in 0..count {
                x[i] += direction[i] * alpha;
                residual[i] -= product[i] * alpha;
            }
            let next = residual.iter().map(|p| p.length_squared()).sum::<f64>();
            for i in 0..count {
                direction[i] = residual[i] + direction[i] * (next / norm);
            }
            norm = next;
        }
        corrections[old..iso].copy_from_slice(&x);
        for i in old..iso {
            coords[i] = (base[i] + corrections[i]).to_array().map(|x| x as f32);
        }
        for i in old..g.positions.len() {
            corrections[i] = v(coords[i]) - base[i];
        }
        for i in old..iso {
            r.material_residual_max = r
                .material_residual_max
                .max(corrections[i].distance(graph.average(&corrections, i)));
        }
        for &(guard, source) in guards {
            r.material_guard_jump_after = r
                .material_guard_jump_after
                .max(v(coords[guard as usize]).distance(v(coords[source as usize])));
        }
    }
    Ok((r, final_angles))
}
