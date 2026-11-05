//! Revision-scoped, bounded component modeling; no implicit weld, detach or weight pruning.
use crate::{
    meshes::{self, Asset, Geometry, Weight},
    model::{Result, invalid},
};
use glam::{DVec2, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};
mod bevel;
mod subtract;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    /// Add the finite convex hull of asset-local points to a static solid.
    UnionConvex {
        #[serde(deserialize_with = "subtract::decode_points")]
        #[schemars(length(min = 4, max = 32))]
        points: Vec<[f32; 3]>,
    },
    /// Remove the finite convex hull of asset-local points from a static solid.
    SubtractConvex {
        #[serde(deserialize_with = "subtract::decode_points")]
        #[schemars(length(min = 4, max = 32))]
        points: Vec<[f32; 3]>,
    },
    Extrude {
        faces: Vec<u32>,
        offset: [f32; 3],
    },
    Inset {
        faces: Vec<u32>,
        width: f32,
    },
    Bevel {
        edges: Vec<[u32; 2]>,
        width: f32,
        #[serde(default)]
        extend_collinear: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub source_vertices: usize,
    pub source_triangles: usize,
    pub vertices: usize,
    pub triangles: usize,
    pub selected_faces: Vec<u32>,
    pub effective_edges: Vec<[u32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numerical_tolerance_meters: Option<f64>,
}
pub struct Evaluated {
    pub asset: Asset,
    pub report: Report,
}
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
fn array(p: DVec3) -> [f32; 3] {
    p.to_array().map(|x| x as f32)
}
fn edge(a: u32, b: u32) -> [u32; 2] {
    if a < b { [a, b] } else { [b, a] }
}
fn normal(g: &Geometry, f: usize) -> DVec3 {
    let [a, b, c] = g.triangles[f].map(|i| v(g.positions[i as usize]));
    (b - a).cross(c - a)
}
#[derive(Clone, Copy)]
struct Use {
    face: usize,
    a: u32,
    b: u32,
}
struct Topology {
    edges: BTreeMap<[u32; 2], Vec<Use>>,
}
impl Topology {
    fn new(g: &Geometry) -> Result<Self> {
        g.validate()?;
        let mut edges: BTreeMap<[u32; 2], Vec<Use>> = BTreeMap::new();
        for (face, &[a, b, c]) in g.triangles.iter().enumerate() {
            for (a, b) in [(a, b), (b, c), (c, a)] {
                edges
                    .entry(edge(a, b))
                    .or_default()
                    .push(Use { face, a, b });
            }
        }
        if edges
            .values()
            .any(|uses| uses.len() > 2 || (uses.len() == 2 && uses[0].a == uses[1].a))
        {
            return Err(invalid(
                "Modeling requires consistently oriented manifold edges",
            ));
        }
        Ok(Self { edges })
    }
}
struct Region {
    faces: Vec<usize>,
    vertices: Vec<u32>,
    boundary: Vec<[u32; 2]>,
    normal: DVec3,
}
fn region(g: &Geometry, faces: &[u32]) -> Result<Region> {
    if faces.is_empty() || faces.len() > 4096 {
        return Err(invalid("Modeling region requires 1..4096 faces"));
    }
    let mut faces: Vec<usize> = faces.iter().map(|i| *i as usize).collect();
    faces.sort_unstable();
    if faces.last().is_none_or(|i| *i >= g.triangles.len())
        || faces.windows(2).any(|w| w[0] == w[1])
    {
        return Err(invalid(
            "Modeling faces contain missing or duplicate indices",
        ));
    }
    let topology = Topology::new(g)?;
    let mut selected = vec![false; g.triangles.len()];
    for &f in &faces {
        selected[f] = true;
    }
    let mut adjacent = vec![Vec::new(); g.triangles.len()];
    let mut boundary = vec![];
    for uses in topology.edges.values() {
        let inside: Vec<_> = uses.iter().filter(|u| selected[u.face]).collect();
        if inside.len() == 1 {
            boundary.push([inside[0].a, inside[0].b]);
        }
        if inside.len() == 2 {
            adjacent[inside[0].face].push(inside[1].face);
            adjacent[inside[1].face].push(inside[0].face);
        }
    }
    let mut seen = vec![false; g.triangles.len()];
    let mut queue = VecDeque::from([faces[0]]);
    seen[faces[0]] = true;
    let mut visited = 0;
    while let Some(f) = queue.pop_front() {
        visited += 1;
        for &next in &adjacent[f] {
            if !seen[next] {
                seen[next] = true;
                queue.push_back(next);
            }
        }
    }
    if visited != faces.len() || boundary.is_empty() {
        return Err(invalid(
            "Select one edge-connected face region with a boundary",
        ));
    }
    let vertices: Vec<_> = faces
        .iter()
        .flat_map(|f| g.triangles[*f])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let normal = faces
        .iter()
        .map(|f| normal(g, *f))
        .sum::<DVec3>()
        .normalize_or_zero();
    Ok(Region {
        faces,
        vertices,
        boundary,
        normal,
    })
}
/// Area-weighted normal for the native editor; the extrusion command itself uses an explicit offset.
pub fn region_normal(asset: &Asset, faces: &[u32]) -> Result<[f32; 3]> {
    Ok(array(region(&asset.geometry, faces)?.normal))
}
#[derive(Clone)]
struct Vertex {
    position: DVec3,
    material: DVec3,
    weight: Option<Weight>,
}
#[derive(Clone, Copy)]
struct Face {
    vertices: [u32; 3],
    selected: bool,
    group: Option<usize>,
}
#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub vertices: usize,
    pub triangles: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            vertices: meshes::MAX_VERTICES,
            triangles: meshes::MAX_TRIANGLES,
        }
    }
}
struct Builder<'a> {
    source: &'a Asset,
    limits: Limits,
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
    effective_edges: Vec<[u32; 2]>,
}
fn blended_weight(parts: impl Iterator<Item = (Weight, f64)>) -> Result<Weight> {
    let mut sums = [0.; meshes::MAX_BONES];
    for (weight, factor) in parts {
        let sum: f64 = weight.influences().map(|(_, w)| f64::from(w)).sum();
        for (joint, w) in weight.influences() {
            sums[joint as usize] += f64::from(w) / sum * factor;
        }
    }
    Weight::from_sums(&sums)
}
impl<'a> Builder<'a> {
    fn new(source: &'a Asset, limits: Limits) -> Self {
        Self {
            source,
            limits,
            vertices: source
                .geometry
                .positions
                .iter()
                .enumerate()
                .map(|(i, p)| Vertex {
                    position: v(*p),
                    material: v(source
                        .geometry
                        .material_coordinates
                        .as_ref()
                        .map_or(*p, |c| c[i])),
                    weight: source.rig.as_ref().map(|r| r.weights[i].clone()),
                })
                .collect(),
            faces: vec![],
            effective_edges: vec![],
        }
    }
    fn push(&mut self, vertex: Vertex) -> Result<u32> {
        if self.vertices.len() >= self.limits.vertices {
            return Err(invalid("Modeling vertex budget exceeded"));
        }
        let i = self.vertices.len() as u32;
        self.vertices.push(vertex);
        Ok(i)
    }
    fn triangle(&mut self, vertices: [u32; 3], selected: bool, group: Option<usize>) -> Result<()> {
        if self.faces.len() >= self.limits.triangles {
            return Err(invalid("Modeling triangle budget exceeded"));
        }
        self.faces.push(Face {
            vertices,
            selected,
            group,
        });
        Ok(())
    }
    fn compact(&mut self) {
        let mut keep = vec![false; self.vertices.len()];
        for f in &self.faces {
            for i in f.vertices {
                keep[i as usize] = true;
            }
        }
        // Isolated vertices that existed before this operation are authored data.
        let mut source_used = vec![false; self.source.geometry.positions.len()];
        for face in &self.source.geometry.triangles {
            for &i in face {
                source_used[i as usize] = true;
            }
        }
        // Original IDs remain in their original slots until the first compaction.
        // Subsequent bevel compactions have no pre-existing isolated vertices.
        if self.vertices.len() >= source_used.len() {
            for (i, used) in source_used.iter().enumerate() {
                if !used {
                    keep[i] = true;
                }
            }
        }
        let mut mapping = vec![u32::MAX; self.vertices.len()];
        let mut next = 0;
        for (i, &keep) in keep.iter().enumerate() {
            if keep {
                mapping[i] = next;
                next += 1;
            }
        }
        let mut i = 0;
        self.vertices.retain(|_| {
            let value = keep[i];
            i += 1;
            value
        });
        for f in &mut self.faces {
            f.vertices = f.vertices.map(|i| mapping[i as usize]);
        }
    }
    fn finish(mut self) -> Result<Evaluated> {
        self.compact();
        let geometry = Geometry {
            positions: self.vertices.iter().map(|v| array(v.position)).collect(),
            triangles: self.faces.iter().map(|f| f.vertices).collect(),
            material_coordinates: Some(self.vertices.iter().map(|v| array(v.material)).collect()),
        };
        geometry.validate()?;
        Topology::new(&geometry)?;
        let mut asset = self.source.clone();
        asset.geometry = Arc::new(geometry);
        if let Some(rig) = &self.source.rig {
            let mut next = (**rig).clone();
            next.weights = self
                .vertices
                .iter()
                .map(|v| {
                    v.weight
                        .clone()
                        .expect("rig weights carried by every vertex")
                })
                .collect();
            asset.rig = Some(Arc::new(next));
        }
        meshes::Document {
            objects: Default::default(),
            assets: vec![asset.clone()],
            instances: vec![],
        }
        .validate()?;
        let report = Report {
            source_vertices: self.source.geometry.positions.len(),
            source_triangles: self.source.geometry.triangles.len(),
            vertices: asset.geometry.positions.len(),
            triangles: asset.geometry.triangles.len(),
            selected_faces: self
                .faces
                .iter()
                .enumerate()
                .filter_map(|(i, f)| f.selected.then_some(i as u32))
                .collect(),
            effective_edges: self.effective_edges,
            numerical_tolerance_meters: None,
        };
        Ok(Evaluated { asset, report })
    }
}
fn check_counts(v: usize, t: usize, limits: Limits) -> Result<()> {
    if v > limits.vertices || t > limits.triangles {
        return Err(invalid("Modeling exceeds vertex/triangle budget"));
    }
    Ok(())
}
fn cap_and_rim<'a>(
    source: &'a Asset,
    r: &Region,
    new_positions: impl Fn(u32) -> Result<DVec3>,
    limits: Limits,
) -> Result<Builder<'a>> {
    let selected_faces: BTreeSet<_> = r.faces.iter().copied().collect();
    let mut duplicate: BTreeSet<_> = r.boundary.iter().flatten().copied().collect();
    for (f, vertices) in source.geometry.triangles.iter().enumerate() {
        if !selected_faces.contains(&f) {
            for &i in vertices {
                if r.vertices.binary_search(&i).is_ok() {
                    duplicate.insert(i);
                }
            }
        }
    }
    check_counts(
        source.geometry.positions.len() + duplicate.len(),
        source.geometry.triangles.len() + r.boundary.len() * 2,
        limits,
    )?;
    let mut builder = Builder::new(source, limits);
    let mut mapping = vec![u32::MAX; source.geometry.positions.len()];
    for &old in &r.vertices {
        let position = new_positions(old)?;
        if duplicate.contains(&old) {
            let mut next = builder.vertices[old as usize].clone();
            next.position = position;
            mapping[old as usize] = builder.push(next)?;
        } else {
            builder.vertices[old as usize].position = position;
            mapping[old as usize] = old;
        }
    }
    let selected: BTreeSet<_> = r.faces.iter().copied().collect();
    for (i, &f) in source.geometry.triangles.iter().enumerate() {
        builder.triangle(
            if selected.contains(&i) {
                f.map(|i| mapping[i as usize])
            } else {
                f
            },
            selected.contains(&i),
            None,
        )?;
    }
    for &[a, b] in &r.boundary {
        let [c, d] = [mapping[a as usize], mapping[b as usize]];
        builder.triangle([a, b, d], false, None)?;
        builder.triangle([a, d, c], false, None)?;
    }
    Ok(builder)
}
pub fn evaluate(source: &Asset, operation: &Operation) -> Result<Evaluated> {
    evaluate_limited(source, operation, Limits::default())
}
pub(crate) fn evaluate_limited(
    source: &Asset,
    operation: &Operation,
    limits: Limits,
) -> Result<Evaluated> {
    meshes::Document {
        objects: Default::default(),
        assets: vec![source.clone()],
        instances: vec![],
    }
    .validate()?;
    match operation {
        Operation::SubtractConvex { points } => subtract::evaluate(source, points, limits),
        Operation::UnionConvex { points } => subtract::evaluate_union(source, points, limits),
        Operation::Extrude { faces, offset } => {
            let offset = v(*offset);
            if !offset.is_finite()
                || offset.length() < 0.00001
                || offset.abs().max_element() > 10_000.
            {
                return Err(invalid(
                    "Extrusion offset requires finite nonzero local displacement",
                ));
            }
            let r = region(&source.geometry, faces)?;
            cap_and_rim(
                source,
                &r,
                |i| Ok(v(source.geometry.positions[i as usize]) + offset),
                limits,
            )?
            .finish()
        }
        Operation::Inset { faces, width } => inset(source, faces, *width, limits),
        Operation::Bevel {
            edges,
            width,
            extend_collinear,
        } => bevel::evaluate(source, edges, *width, *extend_collinear, limits),
    }
}
fn boundary_loop(r: &Region) -> Result<Vec<u32>> {
    if r.boundary.len() > 256 {
        return Err(invalid("Inset boundary exceeds 256 vertices"));
    }
    let mut next = BTreeMap::new();
    let mut incoming = BTreeSet::new();
    for &[a, b] in &r.boundary {
        if next.insert(a, b).is_some() || !incoming.insert(b) {
            return Err(invalid("Inset requires one simple boundary loop"));
        }
    }
    let start = *next.keys().next().unwrap();
    let mut out = vec![];
    let mut at = start;
    loop {
        out.push(at);
        at = *next
            .get(&at)
            .ok_or_else(|| invalid("Inset boundary is open"))?;
        if at == start {
            break;
        }
        if out.len() >= next.len() {
            return Err(invalid("Inset requires one boundary without holes"));
        }
    }
    if out.len() != next.len() {
        return Err(invalid(
            "Inset does not fill holes or disconnected boundaries",
        ));
    }
    Ok(out)
}
fn inset(source: &Asset, faces: &[u32], width: f32, limits: Limits) -> Result<Evaluated> {
    if !width.is_finite() || !(0.00001..=100.).contains(&width) {
        return Err(invalid("Inset width requires 0.00001..100 local metres"));
    }
    let r = region(&source.geometry, faces)?;
    let n = r.normal;
    if n == DVec3::ZERO {
        return Err(invalid("Inset requires a planar region"));
    }
    let boundary = boundary_loop(&r)?;
    let origin = v(source.geometry.positions[boundary[0] as usize]);
    let u = (v(source.geometry.positions[boundary[1] as usize]) - origin).normalize();
    let axis = n.cross(u);
    let project = |p: DVec3| DVec2::new((p - origin).dot(u), (p - origin).dot(axis));
    let extent = r
        .vertices
        .iter()
        .map(|i| v(source.geometry.positions[*i as usize]).distance(origin))
        .fold(0., f64::max)
        .max(0.001);
    let eps = extent * 1e-7;
    if r.vertices.iter().any(|i| {
        (v(source.geometry.positions[*i as usize]) - origin)
            .dot(n)
            .abs()
            > extent * 1e-5
    }) || r
        .faces
        .iter()
        .any(|f| normal(&source.geometry, *f).normalize().dot(n) < 1. - 1e-5)
    {
        return Err(invalid(
            "Inset region must be planar with consistent face orientation",
        ));
    }
    let points: Vec<_> = boundary
        .iter()
        .map(|i| project(v(source.geometry.positions[*i as usize])))
        .collect();
    let mut corners = vec![];
    for i in 0..points.len() {
        let a = (points[i] - points[(i + points.len() - 1) % points.len()]).normalize();
        let b = (points[(i + 1) % points.len()] - points[i]).normalize();
        let cross = a.perp_dot(b);
        if cross < -1e-7 {
            return Err(invalid("Inset currently requires a convex region"));
        }
        if cross > 1e-7 {
            corners.push(points[i]);
        }
    }
    if corners.len() < 3 {
        return Err(invalid("Inset region has no polygonal area"));
    }
    let mut inner = vec![];
    for i in 0..corners.len() {
        let prev = (corners[i] - corners[(i + corners.len() - 1) % corners.len()]).normalize();
        let next = (corners[(i + 1) % corners.len()] - corners[i]).normalize();
        let a = DVec2::new(-prev.y, prev.x);
        let b = DVec2::new(-next.y, next.x);
        let den = 1. + a.dot(b);
        if den < 1e-8 {
            return Err(invalid("Inset corner is singular"));
        }
        inner.push(corners[i] + (a + b) * (f64::from(width) / den));
    }
    // Every inner point must remain strictly inside every original edge, and the
    // offset polygon must preserve ordering. Oversized requests are refused, not clamped.
    for i in 0..corners.len() {
        let a = corners[i];
        let edge = corners[(i + 1) % corners.len()] - a;
        if inner
            .iter()
            .any(|p| edge.perp_dot(*p - a) <= eps * edge.length())
        {
            return Err(invalid("Inset width collapses or crosses the boundary"));
        }
        let prev = inner[i] - inner[(i + inner.len() - 1) % inner.len()];
        let next = inner[(i + 1) % inner.len()] - inner[i];
        if prev.perp_dot(next) <= eps * eps || prev.length() <= eps || next.length() <= eps {
            return Err(invalid("Inset width collapses a corner"));
        }
    }
    let map = |point: DVec2| -> Result<DVec2> {
        for i in 0..corners.len() {
            let a = corners[i];
            let b = corners[(i + 1) % corners.len()];
            let edge = b - a;
            let t = (point - a).dot(edge) / edge.length_squared();
            if (0.0..=1.0).contains(&t) && edge.perp_dot(point - a).abs() <= eps * edge.length() {
                return Ok(inner[i].lerp(inner[(i + 1) % inner.len()], t));
            }
        }
        let vectors: Vec<_> = corners.iter().map(|p| *p - point).collect();
        let distances: Vec<_> = vectors.iter().map(|p| p.length()).collect();
        let mut tangents = vec![];
        for i in 0..vectors.len() {
            let j = (i + 1) % vectors.len();
            let den = distances[i] * distances[j] + vectors[i].dot(vectors[j]);
            if den <= 0. {
                return Err(invalid("Inset mean-value map is singular"));
            }
            tangents.push(vectors[i].perp_dot(vectors[j]) / den);
        }
        let mut total = 0.;
        let mut result = DVec2::ZERO;
        for i in 0..vectors.len() {
            let w =
                (tangents[(i + vectors.len() - 1) % vectors.len()] + tangents[i]) / distances[i];
            total += w;
            result += inner[i] * w;
        }
        if !total.is_finite() || total <= 0. {
            return Err(invalid("Inset mean-value map is invalid"));
        }
        Ok(result / total)
    };
    let builder = cap_and_rim(
        source,
        &r,
        |i| {
            let p = map(project(v(source.geometry.positions[i as usize])))?;
            Ok(origin + u * p.x + axis * p.y)
        },
        limits,
    )?;
    for (i, f) in builder.faces.iter().enumerate() {
        if i < source.geometry.triangles.len() && !f.selected {
            continue;
        }
        let [a, b, c] = f.vertices.map(|i| builder.vertices[i as usize].position);
        if (b - a).cross(c - a).dot(n) <= eps * eps {
            return Err(invalid("Inset would collapse or reverse a face"));
        }
    }
    builder.finish()
}
