//! Pure three-port tubular unions. Retained data keep their exact indices;
//! the implicit interior is explicitly new geometry, with one rigid root.
mod refine;
mod volume;
use crate::{
    mesh_bridge,
    mesh_topology::Topology,
    meshes::{self, Asset, Geometry},
    model::{Result, invalid},
};
use glam::DVec3;
pub use refine::Report as RefinementReport;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
pub(super) fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub hub: [f32; 3],
    pub hub_radius: f32,
    pub blend: f32,
    pub cell_size: f32,
    pub guard_length: f32,
    pub collar_length: f32,
    pub relaxation_steps: u8,
    /// New interior domain only. Retained material coordinates are never changed.
    pub material_offset: [f32; 3],
    #[serde(default)]
    pub seam_relaxation_steps: u8,
    #[serde(default)]
    pub seam_max_displacement: f32,
    #[serde(default)]
    pub material_relaxation_steps: u8,
}
#[derive(Debug, Clone, Serialize)]
pub struct PortReport {
    pub source_vertices: usize,
    pub extracted_vertices: usize,
    pub phase: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub grid_points: usize,
    pub extracted_vertices: usize,
    pub extracted_triangles: usize,
    pub added_vertices: usize,
    pub added_triangles: usize,
    pub max_bend_degrees: f64,
    pub mean_bend_degrees: f64,
    pub source_seam_max_degrees: f64,
    pub extracted_seam_max_degrees: f64,
    pub interior_max_degrees: f64,
    pub max_field_error: f64,
    pub max_field_error_after_seam: f64,
    pub refinement: RefinementReport,
    pub ports: Vec<PortReport>,
    pub retained_vertices_exact: bool,
    pub retained_triangles_exact: bool,
}
#[derive(Debug, Serialize)]
pub struct Fitted {
    /// Transient exact source/guard provenance, also useful for explicit skinning.
    pub material_plan: crate::mesh_material_finish::Plan,
    pub asset: Asset,
    pub report: Report,
}
pub(super) struct Port {
    pub points: Vec<u32>,
    pub centre: DVec3,
    pub normal: DVec3,
    pub radius: f64,
}
pub(super) struct Surface {
    pub points: Vec<DVec3>,
    pub triangles: Vec<[u32; 3]>,
}
pub(super) fn normals(p: &[DVec3], t: &[[u32; 3]]) -> Result<Vec<DVec3>> {
    let mut ns = vec![DVec3::ZERO; p.len()];
    for tri in t {
        let [a, b, c] = tri.map(|i| p[i as usize]);
        let n = (b - a).cross(c - a);
        for &i in tri {
            ns[i as usize] += n;
        }
    }
    for n in &mut ns {
        if !n.is_finite() || n.length_squared() < 1e-24 {
            return Err(invalid("Union surface has an unresolved vertex normal"));
        }
        *n = n.normalize();
    }
    Ok(ns)
}
fn oriented(g: &Geometry) -> Result<()> {
    let t = Topology::inspect(g)?;
    if t.non_manifold_edges != 0 || !t.isolated_vertices.is_empty() {
        return Err(invalid("Union requires manifold used vertices"));
    }
    for e in t.edges.iter().filter(|e| e.faces.len() == 2) {
        let forward = |f: u32| {
            let tri = g.triangles[f as usize];
            (0..3).any(|j| tri[j] == e.vertices[0] && tri[(j + 1) % 3] == e.vertices[1])
        };
        if forward(e.faces[0]) == forward(e.faces[1]) {
            return Err(invalid(
                "Union source or result has inconsistent face winding",
            ));
        }
    }
    Ok(())
}
fn unit_bisector(p: &[DVec3], lp: &[u32], j: usize) -> Result<DVec3> {
    let n = lp.len();
    let a = p[lp[j] as usize];
    let before = a - p[lp[(j + n - 1) % n] as usize];
    let after = p[lp[(j + 1) % n] as usize] - a;
    if before.length_squared() < 1e-20 || after.length_squared() < 1e-20 {
        return Err(invalid("Union boundary has unresolved edges"));
    }
    let along = before.normalize() + after.normalize();
    if along.length_squared() < 1e-16 {
        return Err(invalid("Union boundary has a reversing tangent"));
    }
    Ok(along)
}
fn boundary_normals(p: &[DVec3], triangles: &[[u32; 3]]) -> BTreeMap<[u32; 2], DVec3> {
    let mut edges: BTreeMap<[u32; 2], (usize, DVec3)> = BTreeMap::new();
    for t in triangles {
        let [a, b, c] = t.map(|i| p[i as usize]);
        let normal = (b - a).cross(c - a).normalize();
        for j in 0..3 {
            let a = t[j];
            let b = t[(j + 1) % 3];
            let e = if a < b { [a, b] } else { [b, a] };
            let entry = edges.entry(e).or_insert((0, normal));
            entry.0 += 1;
        }
    }
    edges
        .into_iter()
        .filter_map(|(e, (n, v))| (n == 1).then_some((e, v)))
        .collect()
}
fn triangle_cost(
    p: &[DVec3],
    ns: &[DVec3],
    boundary: &BTreeMap<[u32; 2], DVec3>,
    t: [u32; 3],
) -> f64 {
    let [a, b, c] = t.map(|i| p[i as usize]);
    let normal = (b - a).cross(c - a);
    let area = normal.length();
    if area < 1e-10 {
        return f64::INFINITY;
    }
    let expected = ns[t[0] as usize] + ns[t[1] as usize] + ns[t[2] as usize];
    let cosine = normal.dot(expected) / (area * expected.length());
    if !cosine.is_finite() || cosine < 0.05 {
        return f64::INFINITY;
    }
    let mut seam = 0.;
    for j in 0..3 {
        let a = t[j];
        let b = t[(j + 1) % 3];
        let key = if a < b { [a, b] } else { [b, a] };
        if let Some(n) = boundary.get(&key) {
            let c = normal.dot(*n) / area;
            if c < 0.5 {
                return f64::INFINITY;
            }
            seam += 1. - c;
        }
    }
    (a - b).length_squared() / area
        + (b - c).length_squared() / area
        + (c - a).length_squared() / area
        + 8. * (1. - cosine)
        + 12. * seam
}
fn zipper(
    p: &[DVec3],
    ns: &[DVec3],
    boundary: &BTreeMap<[u32; 2], DVec3>,
    a: &[u32],
    b: &[u32],
) -> Option<(f64, Vec<[u32; 3]>)> {
    let (na, nb) = (a.len(), b.len());
    let stride = nb + 1;
    let mut costs = vec![f64::INFINITY; (na + 1) * stride];
    let mut from = vec![0_u8; costs.len()];
    costs[0] = 0.;
    for i in 0..=na {
        for j in 0..=nb {
            let cost = costs[i * stride + j];
            if !cost.is_finite() {
                continue;
            }
            if i < na {
                let cc = triangle_cost(p, ns, boundary, [a[i % na], a[(i + 1) % na], b[j % nb]]);
                let index = (i + 1) * stride + j;
                if cost + cc < costs[index] {
                    costs[index] = cost + cc;
                    from[index] = 1;
                }
            }
            if j < nb {
                let cc = triangle_cost(p, ns, boundary, [a[i % na], b[(j + 1) % nb], b[j % nb]]);
                let index = i * stride + j + 1;
                if cost + cc < costs[index] {
                    costs[index] = cost + cc;
                    from[index] = 2;
                }
            }
        }
    }
    let cost = costs[na * stride + nb];
    if !cost.is_finite() {
        return None;
    }
    let (mut i, mut j) = (na, nb);
    let mut triangles = Vec::with_capacity(na + nb);
    while i != 0 || j != 0 {
        match from[i * stride + j] {
            1 => {
                i -= 1;
                triangles.push([a[i % na], a[(i + 1) % na], b[j % nb]]);
            }
            2 => {
                j -= 1;
                triangles.push([a[i % na], b[(j + 1) % nb], b[j % nb]]);
            }
            _ => return None,
        }
    }
    Some((cost, triangles))
}
pub fn fit(
    source: &Asset,
    loops: &[Vec<u32>; 3],
    root: Option<&str>,
    options: &Options,
) -> Result<Fitted> {
    source.validate()?;
    oriented(&source.geometry)?;
    let finite = options
        .hub
        .iter()
        .chain(&options.material_offset)
        .all(|v| v.is_finite() && v.abs() <= 10000.);
    if !finite
        || !options.hub_radius.is_finite()
        || !(0.001..=100.).contains(&options.hub_radius)
        || !options.blend.is_finite()
        || !(0.0001..=10.).contains(&options.blend)
        || !options.cell_size.is_finite()
        || !(0.001..=10.).contains(&options.cell_size)
        || !options.guard_length.is_finite()
        || !(0.0001..=1.).contains(&options.guard_length)
        || !options.collar_length.is_finite()
        || options.collar_length <= 2. * options.guard_length
        || options.collar_length > 2.
        || options.relaxation_steps > 8
        || options.seam_relaxation_steps > 8
        || options.material_relaxation_steps > 64
        || !options.seam_max_displacement.is_finite()
        || options.seam_max_displacement < 0.
        || options.seam_max_displacement > options.cell_size * 0.5
        || options.seam_max_displacement > options.collar_length
        || (options.seam_relaxation_steps > 0 && options.seam_max_displacement == 0.)
        || (options.material_relaxation_steps > 0 && source.geometry.material_coordinates.is_none())
    {
        return Err(invalid("Union options outside finite bounded ranges"));
    }
    let source_points: Vec<_> = source.geometry.positions.iter().copied().map(v).collect();
    let boundaries = mesh_bridge::boundaries(&source.geometry)?;
    if boundaries.len() != 3 {
        return Err(invalid(
            "Union requires exactly three real source boundaries",
        ));
    }
    let mut used = BTreeSet::new();
    let mut ports = Vec::new();
    for lp in loops {
        if !(4..=32).contains(&lp.len())
            || lp.iter().any(|i| !used.insert(*i))
            || !boundaries.iter().any(|b| {
                b.vertices.len() == lp.len()
                    && (0..lp.len())
                        .any(|s| (0..lp.len()).all(|j| lp[j] == b.vertices[(j + s) % lp.len()]))
            })
        {
            return Err(invalid(
                "Union ports must be disjoint oriented4..32vertex boundaries",
            ));
        }
        let centre = lp.iter().map(|&i| source_points[i as usize]).sum::<DVec3>() / lp.len() as f64;
        let area = (0..lp.len())
            .map(|j| {
                (source_points[lp[j] as usize] - centre)
                    .cross(source_points[lp[(j + 1) % lp.len()] as usize] - centre)
            })
            .sum::<DVec3>();
        if area.length_squared() < 1e-18 {
            return Err(invalid("Union port has no resolved plane"));
        }
        let normal = area.normalize();
        let radius = lp
            .iter()
            .map(|&i| source_points[i as usize].distance(centre))
            .sum::<f64>()
            / lp.len() as f64;
        if !(0.001..=100.).contains(&radius)
            || f64::from(options.cell_size) > radius * 0.8
            || f64::from(options.guard_length) > radius * 0.25
            || normal.dot(centre - v(options.hub)) <= f64::from(options.guard_length) * 2.
        {
            return Err(invalid(
                "Union hub, grid or guard does not fit the tubular ports",
            ));
        }
        if lp.iter().any(|&i| {
            let d = source_points[i as usize] - centre;
            d.dot(normal).abs() > radius * 0.2 || (d.length() - radius).abs() > radius * 0.25
        }) {
            return Err(invalid(
                "Union currently requires approximately round planar ports",
            ));
        }
        ports.push(Port {
            points: lp.clone(),
            centre,
            normal,
            radius,
        });
    }
    let root_index = match (&source.rig, root) {
        (Some(r), Some(name)) => Some(
            r.bones
                .iter()
                .position(|b| b.name == name && b.parent.is_none())
                .ok_or_else(|| invalid("Union requires an explicit root bone"))? as u8,
        ),
        (None, None) => None,
        _ => return Err(invalid("Union root and source rig disagree")),
    };
    if let Some(index) = root_index {
        let rig = source.rig.as_ref().unwrap();
        for p in &ports {
            for &i in &p.points {
                let w = &rig.weights[i as usize];
                if w.influences().any(|(j, a)| a > 0. && j != index) {
                    return Err(invalid(
                        "Union boundaries require one rigid root; multibone interpolation is not inferred",
                    ));
                }
            }
        }
    }
    let extracted = volume::extract(&ports, options)?;
    let geometry = Geometry {
        positions: extracted
            .surface
            .points
            .iter()
            .map(|p| p.to_array().map(|x| x as f32))
            .collect(),
        triangles: extracted.surface.triangles.clone(),
        material_coordinates: None,
    };
    oriented(&geometry)?;
    let topology = Topology::inspect(&geometry)?;
    let mut extracted_loops = mesh_bridge::boundaries(&geometry)?;
    if extracted_loops.len() != 3
        || geometry.positions.len() as isize - topology.edges.len() as isize
            + geometry.triangles.len() as isize
            != -1
    {
        return Err(invalid(
            "Union extraction must form a three-boundary pair of pants",
        ));
    }
    let guard_count: usize = ports.iter().map(|p| p.points.len()).sum();
    let vertex_count = source_points.len() + extracted.surface.points.len() + guard_count;
    let triangle_count = source.geometry.triangles.len()
        + extracted.surface.triangles.len()
        + 3 * guard_count
        + extracted_loops
            .iter()
            .map(|b| b.vertices.len())
            .sum::<usize>();
    if vertex_count > meshes::MAX_VERTICES || triangle_count > meshes::MAX_TRIANGLES {
        return Err(invalid("Union output exceeds mesh vertex/triangle budget"));
    }
    let mut points = source_points;
    let mut triangles = source.geometry.triangles.clone();
    let old_vertices = points.len();
    let old_triangles = triangles.len();
    let iso_end = old_triangles + extracted.surface.triangles.len();
    let mut ns = normals(&points, &triangles)?;
    let iso_normals = normals(&extracted.surface.points, &extracted.surface.triangles)?;
    points.extend_from_slice(&extracted.surface.points);
    ns.extend(iso_normals);
    triangles.extend(
        extracted
            .surface
            .triangles
            .iter()
            .map(|t| t.map(|i| i + old_vertices as u32)),
    );
    let mut port_reports = Vec::new();
    let mut guard_sources = Vec::with_capacity(guard_count);
    for port in &ports {
        let which = (0..extracted_loops.len())
            .min_by(|&a, &b| {
                let ca = DVec3::from_array(extracted_loops[a].centre);
                let cb = DVec3::from_array(extracted_loops[b].centre);
                ca.distance_squared(port.centre)
                    .total_cmp(&cb.distance_squared(port.centre))
            })
            .unwrap();
        let inner = extracted_loops.remove(which);
        if !(4..=128).contains(&inner.vertices.len())
            || DVec3::from_array(inner.centre).distance(port.centre) > port.radius * 2.
        {
            return Err(invalid(
                "Union extraction port is not resolved near its source",
            ));
        }
        let mut a = port.points.clone();
        a.reverse();
        let b: Vec<_> = inner
            .vertices
            .iter()
            .map(|&i| i + old_vertices as u32)
            .collect();
        let mut guard = Vec::new();
        let mut guard_normals = Vec::new();
        for (j, &i) in a.iter().enumerate() {
            let along = unit_bisector(&points, &a, j)?;
            let tangent = ns[i as usize].cross(along).normalize_or_zero();
            if tangent.length_squared() < 0.5 {
                return Err(invalid("Union source tangent is unresolved"));
            }
            guard.push(points[i as usize] + tangent * f64::from(options.guard_length));
            guard_normals.push(ns[i as usize]);
        }
        let ids: Vec<_> = (points.len() as u32..points.len() as u32 + a.len() as u32).collect();
        guard_sources.extend(ids.iter().copied().zip(a.iter().copied()));
        points.extend(guard);
        ns.extend(guard_normals);
        for j in 0..a.len() {
            let k = (j + 1) % a.len();
            triangles.extend([[a[j], a[k], ids[j]], [a[k], ids[k], ids[j]]]);
        }
        let boundary = boundary_normals(&points, &triangles);
        let centre = a.iter().map(|&i| points[i as usize]).sum::<DVec3>() / a.len() as f64;
        let radial = (points[ids[0] as usize] - centre).normalize();
        let bc = b.iter().map(|&i| points[i as usize]).sum::<DVec3>() / b.len() as f64;
        let mut phases: Vec<_> = (0..b.len()).collect();
        phases.sort_by(|&i, &j| {
            let a = (points[b[i] as usize] - bc).normalize() - radial;
            let b = (points[b[j] as usize] - bc).normalize() - radial;
            a.length_squared()
                .total_cmp(&b.length_squared())
                .then(i.cmp(&j))
        });
        let mut best: Option<(f64, Vec<[u32; 3]>, usize)> = None;
        for &phase in phases.iter().take(8) {
            let mut lp = b.clone();
            lp.rotate_left(phase);
            if let Some((cost, faces)) = zipper(&points, &ns, &boundary, &ids, &lp)
                && best.as_ref().is_none_or(|old| cost < old.0)
            {
                best = Some((cost, faces, phase));
            }
        }
        let (_, faces, phase) = best.ok_or_else(|| {
            invalid("Union cannot connect its real contour within the conservative seam limit")
        })?;
        triangles.extend(faces);
        port_reports.push(PortReport {
            source_vertices: a.len(),
            extracted_vertices: b.len(),
            phase,
        });
    }
    let mut g = (*source.geometry).clone();
    g.positions = points
        .iter()
        .map(|p| p.to_array().map(|x| x as f32))
        .collect();
    g.triangles = triangles;
    if let Some(coords) = &mut g.material_coordinates {
        coords.extend(points[old_vertices..].iter().map(|p| {
            (*p + v(options.material_offset))
                .to_array()
                .map(|x| x as f32)
        }));
    }
    oriented(&g)?;
    let topo = Topology::inspect(&g)?;
    if topo.boundary_edges != 0 {
        return Err(invalid("Union failed to close the source boundaries"));
    }
    let (refinement, angles) = refine::apply(
        &mut g,
        &topo,
        old_vertices,
        old_vertices + extracted.surface.points.len(),
        old_triangles,
        iso_end,
        &guard_sources,
        options,
    )?;
    let max_field_error_after_seam = extracted
        .field_error(&g.positions[old_vertices..old_vertices + extracted.surface.points.len()]);
    let mut out = source.clone();
    out.geometry = Arc::new(g);
    if let Some(index) = root_index {
        let mut rig = (**source.rig.as_ref().unwrap()).clone();
        rig.weights.extend(std::iter::repeat_n(
            meshes::Weight {
                extra: None,
                joints: [index; 4],
                weights: [1., 0., 0., 0.],
            },
            points.len() - old_vertices,
        ));
        out.rig = Some(Arc::new(rig));
    }
    out.validate()?;
    Ok(Fitted {
        material_plan: crate::mesh_material_finish::Plan {
            retained_vertices: old_vertices as u32,
            retained_faces: old_triangles as u32,
            iso_vertices: extracted.surface.points.len() as u32,
            pairs: guard_sources
                .into_iter()
                .map(|(guard, source)| [source, guard])
                .collect(),
        },
        asset: out,
        report: Report {
            grid_points: extracted.grid_points,
            extracted_vertices: extracted.surface.points.len(),
            extracted_triangles: extracted.surface.triangles.len(),
            added_vertices: points.len() - old_vertices,
            added_triangles: triangle_count - old_triangles,
            max_bend_degrees: angles.max,
            mean_bend_degrees: angles.mean,
            source_seam_max_degrees: angles.source,
            extracted_seam_max_degrees: angles.extracted,
            interior_max_degrees: angles.interior,
            max_field_error: extracted.max_error,
            max_field_error_after_seam,
            refinement,
            ports: port_reports,
            retained_vertices_exact: true,
            retained_triangles_exact: true,
        },
    })
}
