//! Pure, bounded surface fairing. Position edits never weld disconnected pieces.
use crate::{
    mesh_topology::Topology,
    meshes::{Asset, Geometry},
    model::{Result, invalid},
};
use glam::{DVec3, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub iterations: u8,
    pub strength: f32,
    pub max_displacement: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BendStats {
    pub maximum_degrees: f64,
    pub mean_degrees: f64,
    pub fold_edges: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComponentVolume {
    pub seed_vertex: u32,
    pub closed: bool,
    pub oriented: bool,
    pub signed_before: Option<f64>,
    pub signed_after: Option<f64>,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    pub iterations: u8,
    pub selected_vertices: usize,
    pub explicit_anchors: usize,
    pub boundary_anchors: usize,
    pub movable_vertices: usize,
    pub moved_vertices: usize,
    pub affected_faces: usize,
    pub affected_edges: usize,
    pub max_displacement: f64,
    pub rms_displacement: f64,
    pub bends_before: BendStats,
    pub bends_after: BendStats,
    pub component_volumes: Vec<ComponentVolume>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Filtered {
    pub asset: Asset,
    pub report: Report,
}
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
fn ids(values: &[u32], count: usize, empty: bool) -> Result<Vec<usize>> {
    if values.len() > count || (!empty && values.is_empty()) {
        return Err(invalid(
            "Surface selection must be nonempty and within vertex budget",
        ));
    }
    let mut result: Vec<_> = values.iter().map(|&i| i as usize).collect();
    result.sort_unstable();
    if result.last().is_some_and(|&i| i >= count) || result.windows(2).any(|w| w[0] == w[1]) {
        return Err(invalid(
            "Surface selection/anchors contain missing or duplicate vertices",
        ));
    }
    Ok(result)
}
struct Csr {
    offsets: Vec<usize>,
    values: Vec<usize>,
}
impl Csr {
    fn from_pairs(count: usize, pairs: impl Clone + Iterator<Item = (usize, usize)>) -> Self {
        let mut offsets = vec![0; count + 1];
        for (i, _) in pairs.clone() {
            offsets[i + 1] += 1;
        }
        for i in 1..=count {
            offsets[i] += offsets[i - 1];
        }
        let mut values = vec![0; offsets[count]];
        let mut next = offsets[..count].to_vec();
        for (i, j) in pairs {
            values[next[i]] = j;
            next[i] += 1;
        }
        Self { offsets, values }
    }
    fn at(&self, i: usize) -> &[usize] {
        &self.values[self.offsets[i]..self.offsets[i + 1]]
    }
}
/// A bounded region measured in shortest paths along unique mesh edges.
/// The outer `fixed_rings` layers are anchors; disconnected geometry is untouched.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RingRegion {
    pub vertices: Vec<u32>,
    pub anchors: Vec<u32>,
}
pub fn ring_region(g: &Geometry, seeds: &[u32], rings: u8, fixed_rings: u8) -> Result<RingRegion> {
    if !(1..=16).contains(&rings) || fixed_rings == 0 || fixed_rings > rings {
        return Err(invalid(
            "Ring region requires1..16rings and1..rings fixed outer layers",
        ));
    }
    let seeds = ids(seeds, g.positions.len(), false)?;
    let topo = Topology::inspect(g)?;
    let graph = Csr::from_pairs(
        g.positions.len(),
        topo.edges.iter().flat_map(|e| {
            let [a, b] = e.vertices.map(|i| i as usize);
            [(a, b), (b, a)]
        }),
    );
    let mut distance = vec![u8::MAX; g.positions.len()];
    let mut queue = Vec::with_capacity(g.positions.len());
    for seed in seeds {
        distance[seed] = 0;
        queue.push(seed);
    }
    let mut head = 0;
    while head < queue.len() {
        let i = queue[head];
        head += 1;
        if distance[i] == rings {
            continue;
        }
        for &j in graph.at(i) {
            if distance[j] == u8::MAX {
                distance[j] = distance[i] + 1;
                queue.push(j);
            }
        }
    }
    let mut result = RingRegion {
        vertices: Vec::with_capacity(queue.len()),
        anchors: vec![],
    };
    let mut reached_outer = false;
    for (i, d) in distance.into_iter().enumerate() {
        reached_outer |= d == rings;
        if d <= rings {
            result.vertices.push(i as u32);
            if d > rings - fixed_rings {
                result.anchors.push(i as u32);
            }
        }
    }
    if result.anchors.is_empty() || !reached_outer {
        return Err(invalid("Ring region did not reach its fixed outer layers"));
    }
    Ok(result)
}
fn forward(g: &Geometry, f: u32, e: [u32; 2]) -> bool {
    let t = g.triangles[f as usize];
    (0..3).any(|i| t[i] == e[0] && t[(i + 1) % 3] == e[1])
}
fn normal(points: &[DVec3], t: [u32; 3]) -> DVec3 {
    let [a, b, c] = t.map(|i| points[i as usize]);
    (b - a).cross(c - a)
}
fn stats(cosines: &[f64]) -> BendStats {
    let mut result = BendStats {
        maximum_degrees: 0.,
        mean_degrees: 0.,
        fold_edges: 0,
    };
    for &c in cosines {
        let d = c.clamp(-1., 1.).acos().to_degrees();
        result.maximum_degrees = result.maximum_degrees.max(d);
        result.mean_degrees += d;
        result.fold_edges += usize::from(c <= 0.);
    }
    if !cosines.is_empty() {
        result.mean_degrees /= cosines.len() as f64;
    }
    result
}
fn volumes(
    g: &Geometry,
    topo: &Topology,
    graph: &Csr,
    points: &[DVec3],
    movable: &[usize],
) -> Vec<ComponentVolume> {
    let mut labels = vec![usize::MAX; points.len()];
    let mut seeds = Vec::new();
    let mut pending = Vec::new();
    for i in 0..points.len() {
        if labels[i] != usize::MAX {
            continue;
        }
        labels[i] = seeds.len();
        seeds.push(i);
        pending.push(i);
        while let Some(j) = pending.pop() {
            for &k in graph.at(j) {
                if labels[k] == usize::MAX {
                    labels[k] = labels[i];
                    pending.push(k);
                }
            }
        }
    }
    let mut touched = vec![false; seeds.len()];
    for &i in movable {
        touched[labels[i]] = true;
    }
    let mut reports: Vec<_> = seeds
        .iter()
        .map(|&i| ComponentVolume {
            seed_vertex: i as u32,
            closed: true,
            oriented: true,
            signed_before: None,
            signed_after: None,
        })
        .collect();
    for e in &topo.edges {
        let r = &mut reports[labels[e.vertices[0] as usize]];
        if e.faces.len() != 2 {
            r.closed = false;
        } else if forward(g, e.faces[0], e.vertices) == forward(g, e.faces[1], e.vertices) {
            r.oriented = false;
        }
    }
    let mut before = vec![0.; seeds.len()];
    let mut after = vec![0.; seeds.len()];
    for &t in &g.triangles {
        let label = labels[t[0] as usize];
        let r = &reports[label];
        if !touched[label] || !r.closed || !r.oriented {
            continue;
        }
        let origin = v(g.positions[seeds[label]]);
        let [a, b, c] = t.map(|i| v(g.positions[i as usize]) - origin);
        before[label] += a.dot(b.cross(c)) / 6.;
        let [a, b, c] = t.map(|i| points[i as usize] - origin);
        after[label] += a.dot(b.cross(c)) / 6.;
    }
    reports
        .into_iter()
        .enumerate()
        .filter_map(|(i, mut r)| {
            if !touched[i] {
                return None;
            }
            if r.closed && r.oriented {
                r.signed_before = Some(before[i]);
                r.signed_after = Some(after[i]);
            }
            Some(r)
        })
        .collect()
}

/// Smooth selected vertices in asset-local rest coordinates. Anchors must belong
/// to the selection. All other vertices and selected real boundary vertices pin.
/// The caller owns revision guards, canonical-source policy and publication.
pub fn smooth_region(
    source: &Asset,
    vertices: &[u32],
    anchors: &[u32],
    options: &Options,
) -> Result<Filtered> {
    source.validate()?;
    if !(1..=32).contains(&options.iterations)
        || !options.strength.is_finite()
        || options.strength <= 0.
        || options.strength > 1.
        || !options.max_displacement.is_finite()
        || !(0.000001..=100.).contains(&options.max_displacement)
    {
        return Err(invalid(
            "Surface options require1..32 iterations,strength(0,1],displacement1e-6..100m",
        ));
    }
    let g = &source.geometry;
    let count = g.positions.len();
    let selected = ids(vertices, count, false)?;
    let anchors = ids(anchors, count, true)?;
    let mut selected_mask = vec![false; count];
    for &i in &selected {
        selected_mask[i] = true;
    }
    if anchors.iter().any(|&i| !selected_mask[i]) {
        return Err(invalid("Surface anchors must belong to the selection"));
    }
    let topo = Topology::inspect(g)?;
    let mut pinned = vec![false; count];
    let mut boundary = vec![false; count];
    for e in &topo.edges {
        if e.faces.len() == 1 {
            for i in e.vertices {
                boundary[i as usize] = true;
            }
        }
    }
    for &i in &anchors {
        pinned[i] = true;
    }
    let boundary_anchors = selected.iter().filter(|&&i| boundary[i]).count();
    let movable: Vec<_> = selected
        .iter()
        .copied()
        .filter(|&i| !pinned[i] && !boundary[i])
        .collect();
    if movable.is_empty() {
        return Ok(Filtered {
            asset: source.clone(),
            report: Report {
                iterations: options.iterations,
                selected_vertices: selected.len(),
                explicit_anchors: anchors.len(),
                boundary_anchors,
                movable_vertices: 0,
                moved_vertices: 0,
                affected_faces: 0,
                affected_edges: 0,
                max_displacement: 0.,
                rms_displacement: 0.,
                bends_before: stats(&[]),
                bends_after: stats(&[]),
                component_volumes: vec![],
            },
        });
    }
    let mut moving = vec![false; count];
    for &i in &movable {
        moving[i] = true;
    }
    let faces: Vec<_> = g
        .triangles
        .iter()
        .enumerate()
        .filter_map(|(f, t)| t.iter().any(|&i| moving[i as usize]).then_some(f))
        .collect();
    let mut incident = vec![false; g.triangles.len()];
    for &f in &faces {
        incident[f] = true;
    }
    let graph = Csr::from_pairs(
        count,
        topo.edges.iter().flat_map(|e| {
            let [a, b] = e.vertices.map(|i| i as usize);
            [(a, b), (b, a)].into_iter()
        }),
    );
    let star = Csr::from_pairs(
        count,
        g.triangles
            .iter()
            .enumerate()
            .flat_map(|(f, t)| t.iter().map(move |&i| (i as usize, f))),
    );
    let mut across = vec![[u32::MAX; 3]; g.triangles.len()];
    let mut edges = Vec::new();
    for (index, e) in topo.edges.iter().enumerate() {
        if !e.faces.iter().any(|&f| incident[f as usize]) {
            continue;
        }
        if e.faces.len() > 2 {
            return Err(invalid("Surface region touches a non-manifold edge"));
        }
        if e.faces.len() == 2 {
            if forward(g, e.faces[0], e.vertices) == forward(g, e.faces[1], e.vertices) {
                return Err(invalid("Surface region requires consistent source winding"));
            }
            edges.push(index);
            for k in 0..2 {
                let f = e.faces[k] as usize;
                let t = g.triangles[f];
                for j in 0..3 {
                    if (t[j] == e.vertices[0] && t[(j + 1) % 3] == e.vertices[1])
                        || (t[j] == e.vertices[1] && t[(j + 1) % 3] == e.vertices[0])
                    {
                        across[f][j] = e.faces[1 - k];
                    }
                }
            }
        }
    }
    // Edge-manifold alone misses two closed fans touching at a single vertex.
    let mut seen = vec![0; g.triangles.len()];
    let mut pending = Vec::new();
    for &i in &movable {
        let star = star.at(i);
        if star.is_empty() {
            return Err(invalid("Surface selected vertex has no incident surface"));
        }
        let stamp = i + 1;
        seen[star[0]] = stamp;
        pending.push(star[0]);
        let mut visited = 0;
        while let Some(f) = pending.pop() {
            visited += 1;
            let t = g.triangles[f];
            for j in 0..3 {
                let other = across[f][j];
                if (t[j] as usize == i || t[(j + 1) % 3] as usize == i)
                    && other != u32::MAX
                    && seen[other as usize] != stamp
                {
                    seen[other as usize] = stamp;
                    pending.push(other as usize);
                }
            }
        }
        if visited != star.len() {
            return Err(invalid(
                "Surface selected vertex has disconnected face fans",
            ));
        }
    }
    let original: Vec<_> = g.positions.iter().copied().map(v).collect();
    let mut points = original.clone();
    let original_normals: Vec<_> = g
        .triangles
        .iter()
        .map(|&t| normal(&original, t).normalize())
        .collect();
    let mut normals = original_normals.clone();
    let edge_cosines = |normals: &[DVec3], out: &mut [f64]| {
        for (&e, c) in edges.iter().zip(out) {
            let e = &topo.edges[e];
            *c = normals[e.faces[0] as usize].dot(normals[e.faces[1] as usize]);
        }
    };
    let mut before_cosines = vec![0.; edges.len()];
    edge_cosines(&normals, &mut before_cosines);
    let bends_before = stats(&before_cosines);
    let mut after_cosines = before_cosines.clone();
    let mut updates = vec![DVec3::ZERO; movable.len()];
    let limit = f64::from(options.max_displacement);
    for iteration in 0..options.iterations {
        for (&i, target) in movable.iter().zip(&mut updates) {
            let ns = graph.at(i);
            let average = ns.iter().map(|&j| points[j]).sum::<DVec3>() / ns.len() as f64;
            let next = points[i] + (average - points[i]) * f64::from(options.strength);
            let d = next - original[i];
            *target = original[i] + d * (limit / d.length().max(limit));
        }
        for (&i, &point) in movable.iter().zip(&updates) {
            points[i] = v(point.to_array().map(|x| x as f32));
            if !points[i].is_finite() || points[i].distance(original[i]) > limit + 1e-6 {
                return Err(invalid(
                    "Surface displacement exceeds envelope after f32 quantization",
                ));
            }
        }
        for &f in &faces {
            let t = g.triangles[f];
            let n = normal(&points, t);
            let [a, b, c] =
                t.map(|i| Vec3::from_array(points[i as usize].to_array().map(|x| x as f32)));
            if !n.is_finite()
                || (b - a).cross(c - a).length_squared() <= 0.
                || n.dot(original_normals[f]) <= 0.
            {
                return Err(invalid(format!(
                    "Surface iteration{} reverses or degenerates incident face{f}",
                    iteration + 1
                )));
            }
            normals[f] = n.normalize();
        }
        edge_cosines(&normals, &mut after_cosines);
        for (index, (&before, &after)) in before_cosines.iter().zip(&after_cosines).enumerate() {
            if !after.is_finite()
                || (before > 0. && after <= 0.)
                || (before <= 0. && after < before - 1e-12)
            {
                return Err(invalid(format!(
                    "Surface iteration{} introduces or worsens incident fold at edge{:?}: {:.6}->{:.6} degrees",
                    iteration + 1,
                    topo.edges[edges[index]].vertices,
                    before.clamp(-1., 1.).acos().to_degrees(),
                    after.clamp(-1., 1.).acos().to_degrees()
                )));
            }
        }
    }
    let bends_after = stats(&after_cosines);
    if bends_after.maximum_degrees > bends_before.maximum_degrees + 1e-6
        || bends_after.mean_degrees > bends_before.mean_degrees + 1e-6
    {
        return Err(invalid(format!(
            "Surface filter worsens maximum or mean incident bend: peak {:.6}->{:.6}, mean {:.6}->{:.6} degrees",
            bends_before.maximum_degrees,
            bends_after.maximum_degrees,
            bends_before.mean_degrees,
            bends_after.mean_degrees
        )));
    }
    let mut max = 0_f64;
    let mut sum = 0.;
    let mut moved = 0;
    for &i in &movable {
        let d = points[i].distance_squared(original[i]);
        max = max.max(d);
        sum += d;
        moved += usize::from(d > 0.);
    }
    let component_volumes = volumes(g, &topo, &graph, &points, &movable);
    let mut geometry = (**g).clone();
    for &i in &movable {
        geometry.positions[i] = points[i].to_array().map(|x| x as f32);
    }
    let mut asset = source.clone();
    asset.geometry = Arc::new(geometry);
    asset.validate()?;
    Ok(Filtered {
        asset,
        report: Report {
            iterations: options.iterations,
            selected_vertices: selected.len(),
            explicit_anchors: anchors.len(),
            boundary_anchors,
            movable_vertices: movable.len(),
            moved_vertices: moved,
            affected_faces: faces.len(),
            affected_edges: edges.len(),
            max_displacement: max.sqrt(),
            rms_displacement: if movable.is_empty() {
                0.
            } else {
                (sum / movable.len() as f64).sqrt()
            },
            bends_before,
            bends_after,
            component_volumes,
        },
    })
}
