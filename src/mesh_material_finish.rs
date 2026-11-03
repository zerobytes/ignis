//! Material-only extension on a final junction with explicit transient provenance.
use crate::{
    mesh_material_domain as domain,
    mesh_topology::Topology,
    meshes::{Asset, Geometry},
    model::{Result, invalid},
};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub retained_vertices: u32,
    pub retained_faces: u32,
    pub iso_vertices: u32,
    /// [retained source, generated guard], covering every final guard exactly once.
    pub pairs: Vec<[u32; 2]>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Affine {
    /// Material = column0*position.x + column1*position.y + column2*position.z + offset.
    pub columns: [[f32; 3]; 3],
    pub offset: [f32; 3],
}
impl Affine {
    fn at(&self, p: [f32; 3]) -> DVec3 {
        v(self.offset)
            + v(self.columns[0]) * f64::from(p[0])
            + v(self.columns[1]) * f64::from(p[1])
            + v(self.columns[2]) * f64::from(p[2])
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub reference: Affine,
    pub iterations: u8,
    pub tolerance: f32,
    pub max_change: f32,
    pub max_anisotropy: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct Regions {
    pub source: domain::Report,
    pub guard: domain::Report,
    pub zipper: Option<domain::Report>,
    pub iso: Option<domain::Report>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub iterations: u8,
    pub residual_max: f64,
    pub max_change: f64,
    pub changed_guards: usize,
    pub changed_interior: usize,
    pub max_guard_fit_disagreement: f64,
    pub max_guard_quantization_error: f64,
    pub before: Regions,
    pub after: Regions,
}
#[derive(Debug, Serialize)]
pub struct Finished {
    pub asset: Asset,
    pub report: Report,
}
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
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
        let mut next = offsets[..count].to_vec();
        let mut adjacent = vec![0; offsets[count]];
        for e in &topo.edges {
            let [a, b] = e.vertices.map(|i| i as usize);
            adjacent[next[a]] = b;
            next[a] += 1;
            adjacent[next[b]] = a;
            next[b] += 1;
        }
        Self { offsets, adjacent }
    }
    fn at(&self, i: usize) -> &[usize] {
        &self.adjacent[self.offsets[i]..self.offsets[i + 1]]
    }
}
struct Layout {
    source: Vec<u32>,
    guard: Vec<u32>,
    zipper: Vec<u32>,
    iso: Vec<u32>,
}
impl Layout {
    fn reports(&self, g: &Geometry) -> Result<Regions> {
        let inspect = |faces: &[u32]| domain::inspect(g, Some(faces), &domain::Options::default());
        Ok(Regions {
            source: inspect(&self.source)?,
            guard: inspect(&self.guard)?,
            zipper: (!self.zipper.is_empty())
                .then(|| inspect(&self.zipper))
                .transpose()?,
            iso: (!self.iso.is_empty())
                .then(|| inspect(&self.iso))
                .transpose()?,
        })
    }
}
fn layout(g: &Geometry, plan: &Plan) -> Result<Layout> {
    let old = plan.retained_vertices as usize;
    let end = old
        .checked_add(plan.iso_vertices as usize)
        .ok_or_else(|| invalid("Material plan range overflow"))?;
    let faces = plan.retained_faces as usize;
    if old < 3
        || end >= g.positions.len()
        || faces == 0
        || faces >= g.triangles.len()
        || plan.pairs.len() != g.positions.len() - end
    {
        return Err(invalid(
            "Material plan must cover retained prefixes, iso and all final guards",
        ));
    }
    let mut r = Layout {
        source: vec![],
        guard: vec![],
        zipper: vec![],
        iso: vec![],
    };
    for (i, t) in g.triangles.iter().enumerate() {
        let mut mask = 0;
        for &v in t {
            mask |= if (v as usize) < old {
                1
            } else if (v as usize) < end {
                2
            } else {
                4
            };
        }
        match (i < faces, mask) {
            (true, 1) => r.source.push(i as u32),
            (false, 2) => r.iso.push(i as u32),
            (false, 5) => r.guard.push(i as u32),
            (false, 6) => r.zipper.push(i as u32),
            _ => {
                return Err(invalid(format!(
                    "Material plan face {i} has inconsistent provenance"
                )));
            }
        }
    }
    if r.guard.is_empty() {
        return Err(invalid("Material plan has no source-to-guard faces"));
    }
    Ok(r)
}
fn anchored(graph: &Graph, old: usize, end: usize) -> Result<()> {
    let mut seen = vec![false; end - old];
    let mut queue = Vec::with_capacity(end - old);
    for first in old..end {
        if seen[first - old] {
            continue;
        }
        queue.clear();
        queue.push(first);
        seen[first - old] = true;
        let mut anchor = false;
        let mut head = 0;
        while head < queue.len() {
            let i = queue[head];
            head += 1;
            for &j in graph.at(i) {
                if !(old..end).contains(&j) {
                    anchor = true;
                } else if !seen[j - old] {
                    seen[j - old] = true;
                    queue.push(j);
                }
            }
        }
        if !anchor {
            return Err(invalid(format!(
                "Material interior component at vertex {first} has no Dirichlet anchor"
            )));
        }
    }
    Ok(())
}
pub fn finish(source: &Asset, plan: &Plan, options: &Options) -> Result<Finished> {
    source.validate()?;
    let g = source.geometry.as_ref();
    let coords = g
        .material_coordinates
        .as_deref()
        .ok_or_else(|| invalid("Material finish requires explicit material coordinates"))?;
    if !(1..=64).contains(&options.iterations)
        || !options.tolerance.is_finite()
        || !(1e-9..=1.).contains(&options.tolerance)
        || !options.max_change.is_finite()
        || !(1e-6..=10000.).contains(&options.max_change)
        || !options.max_anisotropy.is_finite()
        || !(1. ..=1e8).contains(&options.max_anisotropy)
        || options
            .reference
            .columns
            .iter()
            .flatten()
            .chain(options.reference.offset.iter())
            .any(|v| !v.is_finite())
    {
        return Err(invalid(
            "Material finish options require finite reference, iterations1..64, tolerance1e-9..1, max_change1e-6..10000 and max_anisotropy1..1e8",
        ));
    }
    let regions = layout(g, plan)?;
    let old = plan.retained_vertices as usize;
    let end = old + plan.iso_vertices as usize;
    let topo = Topology::inspect(g)?;
    if topo.non_manifold_edges > 0 {
        return Err(invalid("Material finish requires manifold edges"));
    }
    let mut boundary = vec![0usize; old];
    for e in &topo.edges {
        if e.faces.iter().filter(|&&f| f < plan.retained_faces).count() == 1 {
            for i in e.vertices {
                if (i as usize) < old {
                    boundary[i as usize] += 1;
                }
            }
        }
        if e.faces.len() == 2 {
            let forward = |face: u32| {
                let t = g.triangles[face as usize];
                (0..3).any(|i| t[i] == e.vertices[0] && t[(i + 1) % 3] == e.vertices[1])
            };
            if forward(e.faces[0]) == forward(e.faces[1]) {
                return Err(invalid(
                    "Material finish requires consistent triangle winding",
                ));
            }
        }
    }
    let mut pairs = plan.pairs.clone();
    pairs.sort_unstable_by_key(|p| p[1]);
    let mut paired = vec![None; old];
    for (index, &[s, t]) in pairs.iter().enumerate() {
        if s as usize >= old
            || t as usize != end + index
            || paired[s as usize].is_some()
            || boundary[s as usize] != 2
        {
            return Err(invalid(
                "Material source/guard pairs must be unique, source-boundary members and cover final guards",
            ));
        }
        let edge = [s.min(t), s.max(t)];
        if topo
            .edges
            .binary_search_by_key(&edge, |e| e.vertices)
            .is_err()
        {
            return Err(invalid(
                "Material source-to-guard pair has no real mesh edge",
            ));
        }
        paired[s as usize] = Some(index);
    }
    let graph = Graph::new(&topo, g.positions.len());
    anchored(&graph, old, end)?;
    let before = regions.reports(g)?;
    let mut sums = vec![DVec3::ZERO; pairs.len()];
    let mut squares = vec![0.; pairs.len()];
    let mut weights = vec![0.; pairs.len()];
    for &face in &regions.source {
        let t = g.triangles[face as usize];
        if t.iter().all(|&s| paired[s as usize].is_none()) {
            continue;
        }
        let metric = domain::sample(g, coords, face, &domain::Options::default())?;
        if metric.near_singular {
            return Err(invalid(format!(
                "Material source face {face} is singular; no transverse direction can be invented"
            )));
        }
        let [a, b, c] = t.map(|i| v(g.positions[i as usize]));
        let e = b - a;
        let f = c - a;
        let n = e.cross(f);
        let reciprocal1 = f.cross(n) / n.length_squared();
        let reciprocal2 = n.cross(e) / n.length_squared();
        let [ma, mb, mc] = t.map(|i| v(coords[i as usize]));
        for s in t {
            if let Some(i) = paired[s as usize] {
                let delta = v(g.positions[pairs[i][1] as usize]) - v(g.positions[s as usize]);
                let predicted =
                    (mb - ma) * reciprocal1.dot(delta) + (mc - ma) * reciprocal2.dot(delta);
                sums[i] += predicted * metric.local_area;
                squares[i] += predicted.length_squared() * metric.local_area;
                weights[i] += metric.local_area;
            }
        }
    }
    let base: Vec<_> = g
        .positions
        .iter()
        .map(|&p| options.reference.at(p))
        .collect();
    let mut corrections: Vec<_> = coords.iter().zip(&base).map(|(&p, &b)| v(p) - b).collect();
    let mut targets = vec![DVec3::ZERO; pairs.len()];
    let mut disagreement: f64 = 0.;
    for (i, &[s, t]) in pairs.iter().enumerate() {
        if weights[i] == 0. {
            return Err(invalid(format!(
                "Material source vertex {s} has no retained incident face"
            )));
        }
        let delta = sums[i] / weights[i];
        targets[i] = v(coords[s as usize]) + delta;
        if !targets[i].is_finite() {
            return Err(invalid("Material guard extrapolation is not finite"));
        }
        disagreement = disagreement.max(
            (squares[i] / weights[i] - delta.length_squared())
                .max(0.)
                .sqrt(),
        );
        corrections[t as usize] = targets[i] - base[t as usize];
    }
    let count = end - old;
    let mut x = vec![DVec3::ZERO; count];
    let mut residual = vec![DVec3::ZERO; count];
    for i in old..end {
        residual[i - old] = graph
            .at(i)
            .iter()
            .filter(|&&j| !(old..end).contains(&j))
            .map(|&j| corrections[j])
            .sum();
    }
    let mut direction = residual.clone();
    let mut product = vec![DVec3::ZERO; count];
    let mut norm = residual.iter().map(|r| r.length_squared()).sum::<f64>();
    let mut iterations = 0;
    while iterations < options.iterations {
        let maximum = residual
            .iter()
            .enumerate()
            .map(|(i, r)| r.length() / graph.at(i + old).len() as f64)
            .fold(0., f64::max);
        if maximum <= f64::from(options.tolerance) * 0.25 {
            break;
        }
        for i in old..end {
            let ns = graph.at(i);
            product[i - old] = direction[i - old] * ns.len() as f64
                - ns.iter()
                    .filter(|&&j| (old..end).contains(&j))
                    .map(|&j| direction[j - old])
                    .sum::<DVec3>();
        }
        let denominator = direction
            .iter()
            .zip(&product)
            .map(|(a, b)| a.dot(*b))
            .sum::<f64>();
        if !norm.is_finite() || !denominator.is_finite() || denominator <= 0. {
            return Err(invalid("Material Dirichlet system is unresolved"));
        }
        let alpha = norm / denominator;
        for i in 0..count {
            x[i] += direction[i] * alpha;
            residual[i] -= product[i] * alpha;
        }
        let next = residual.iter().map(|r| r.length_squared()).sum::<f64>();
        for i in 0..count {
            direction[i] = residual[i] + direction[i] * (next / norm);
        }
        norm = next;
        iterations += 1;
    }
    let mut output = g.clone();
    let material = output.material_coordinates.as_mut().unwrap();
    let mut quantization: f64 = 0.;
    for (i, &[_, t]) in pairs.iter().enumerate() {
        material[t as usize] = targets[i].to_array().map(|x| x as f32);
        quantization = quantization.max(v(material[t as usize]).distance(targets[i]));
    }
    for i in old..end {
        material[i] = (base[i] + x[i - old]).to_array().map(|x| x as f32);
    }
    output.validate()?;
    let material = output.material_coordinates.as_ref().unwrap();
    for i in 0..corrections.len() {
        corrections[i] = v(material[i]) - base[i];
    }
    let mut residual_max: f64 = 0.;
    for i in old..end {
        let mean =
            graph.at(i).iter().map(|&j| corrections[j]).sum::<DVec3>() / graph.at(i).len() as f64;
        residual_max = residual_max.max(corrections[i].distance(mean));
    }
    if residual_max > f64::from(options.tolerance) {
        return Err(invalid(format!(
            "Material final f32 residual {residual_max} exceeds tolerance {}",
            options.tolerance
        )));
    }
    let max_change = (old..coords.len())
        .map(|i| v(material[i]).distance(v(coords[i])))
        .fold(0., f64::max);
    if max_change > f64::from(options.max_change) + 1e-6 {
        return Err(invalid(format!(
            "Material change {max_change} exceeds envelope {}",
            options.max_change
        )));
    }
    let after = regions.reports(&output)?;
    for (name, r) in [
        ("guard", Some(&after.guard)),
        ("iso", after.iso.as_ref()),
        ("zipper", after.zipper.as_ref()),
    ] {
        if let Some(r) = r {
            if r.near_singular_faces > 0 {
                return Err(invalid(format!(
                    "Material {name} still has {} singular faces",
                    r.near_singular_faces
                )));
            }
            if let Some(e) = r
                .maximum_anisotropy
                .as_ref()
                .filter(|e| e.value > f64::from(options.max_anisotropy))
            {
                return Err(invalid(format!(
                    "Material {name} face {} anisotropy {} exceeds anisotropy cap {}",
                    e.face, e.value, options.max_anisotropy
                )));
            }
        }
    }
    let report = Report {
        iterations,
        residual_max,
        max_change,
        changed_guards: (end..coords.len())
            .filter(|&i| coords[i] != material[i])
            .count(),
        changed_interior: (old..end).filter(|&i| coords[i] != material[i]).count(),
        max_guard_fit_disagreement: disagreement,
        max_guard_quantization_error: quantization,
        before,
        after,
    };
    let mut asset = source.clone();
    asset.geometry = Arc::new(output);
    Ok(Finished { asset, report })
}
