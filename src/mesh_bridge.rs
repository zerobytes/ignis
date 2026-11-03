//! Explicit face cuts and tangent-constrained bridges; no proximity welding.
use crate::{
    mesh_topology::Topology,
    meshes::{self, Asset, Geometry},
    model::{Result, invalid},
};
use glam::DVec3;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
#[derive(Debug, Serialize)]
pub struct Boundary {
    pub vertices: Vec<u32>,
    pub centre: [f64; 3],
}
pub fn boundaries(g: &Geometry) -> Result<Vec<Boundary>> {
    let t = Topology::inspect(g)?;
    if t.non_manifold_edges != 0 {
        return Err(invalid("Boundary cycles require a manifold mesh"));
    }
    let mut next = BTreeMap::new();
    let mut incoming = BTreeSet::new();
    for e in t.edges.iter().filter(|e| e.faces.len() == 1) {
        let [a, b] = e.vertices;
        let tri = g.triangles[e.faces[0] as usize];
        let (a, b) = if (0..3).any(|i| tri[i] == a && tri[(i + 1) % 3] == b) {
            (a, b)
        } else {
            (b, a)
        };
        if next.insert(a, b).is_some() || !incoming.insert(b) {
            return Err(invalid("Boundary cycles touch or branch at a vertex"));
        }
    }
    let mut out = Vec::new();
    while let Some((&start, _)) = next.first_key_value() {
        let mut vertices = Vec::new();
        let mut x = start;
        loop {
            vertices.push(x);
            x = next
                .remove(&x)
                .ok_or_else(|| invalid("Boundary chain is open or ambiguous"))?;
            if x == start {
                break;
            }
        }
        let centre = vertices
            .iter()
            .map(|&i| v(g.positions[i as usize]))
            .sum::<DVec3>()
            / vertices.len() as f64;
        out.push(Boundary {
            vertices,
            centre: centre.to_array(),
        });
    }
    Ok(out)
}
pub fn remove_faces(source: &Asset, faces: &[u32]) -> Result<Asset> {
    source.validate()?;
    let mut removed = BTreeSet::new();
    if faces.is_empty() || faces.len() >= source.geometry.triangles.len() {
        return Err(invalid("Face removal must leave at least one face"));
    }
    for &f in faces {
        if f as usize >= source.geometry.triangles.len() || !removed.insert(f) {
            return Err(invalid("Face removal requires unique current face IDs"));
        }
    }
    let triangles: Vec<_> = source
        .geometry
        .triangles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| (!removed.contains(&(i as u32))).then_some(*t))
        .collect();
    let mut used = vec![false; source.geometry.positions.len()];
    for t in &triangles {
        for &i in t {
            used[i as usize] = true;
        }
    }
    let mut map = vec![0; used.len()];
    let mut positions = Vec::new();
    let mut coordinates = source
        .geometry
        .material_coordinates
        .as_ref()
        .map(|_| Vec::new());
    let mut rig = source.rig.as_ref().map(|r| {
        let mut r = (**r).clone();
        r.weights.clear();
        r
    });
    for (i, &take) in used.iter().enumerate() {
        if take {
            map[i] = positions.len() as u32;
            positions.push(source.geometry.positions[i]);
            if let Some(c) = &mut coordinates {
                c.push(source.geometry.material_coordinates.as_ref().unwrap()[i]);
            }
            if let Some(r) = &mut rig {
                r.weights
                    .push(source.rig.as_ref().unwrap().weights[i].clone());
            }
        }
    }
    let mut out = source.clone();
    out.geometry = Arc::new(Geometry {
        positions,
        triangles: triangles
            .iter()
            .map(|t| t.map(|i| map[i as usize]))
            .collect(),
        material_coordinates: coordinates,
    });
    out.rig = rig.map(Arc::new);
    out.validate()?;
    Ok(out)
}
pub fn counts(source: &Asset, loops: &[Vec<u32>; 2], spans: u8) -> Result<(usize, usize)> {
    let n = loops[0].len();
    if !(3..=64).contains(&n) || loops[1].len() != n || !(2..=32).contains(&spans) {
        return Err(invalid(
            "Bridge requires two equal3..64vertex loops and2..32spans",
        ));
    }
    let vertices = source.geometry.positions.len() + n * (spans as usize - 1);
    let triangles = source.geometry.triangles.len() + 2 * n * spans as usize;
    if vertices > meshes::MAX_VERTICES || triangles > meshes::MAX_TRIANGLES {
        return Err(invalid("Bridge exceeds mesh vertex/triangle budget"));
    }
    Ok((vertices, triangles))
}
fn point(c: [DVec3; 4], t: f64) -> DVec3 {
    let u = 1. - t;
    c[0] * (u * u * u) + c[1] * (3. * u * u * t) + c[2] * (3. * u * t * t) + c[3] * (t * t * t)
}
fn derivative(c: [DVec3; 4], t: f64) -> DVec3 {
    let u = 1. - t;
    (c[1] - c[0]) * (3. * u * u) + (c[2] - c[1]) * (6. * u * t) + (c[3] - c[2]) * (3. * t * t)
}
// Spend finite samples near the constrained endpoints, where a long first
// chord otherwise loses the tangent even for a regular cubic loft.
fn parameter(row: usize, spans: u8) -> f64 {
    (1. - (std::f64::consts::PI * row as f64 / f64::from(spans)).cos()) * 0.5
}
pub fn bridge(
    source: &Asset,
    loops: &[Vec<u32>; 2],
    spans: u8,
    handles: [f32; 2],
) -> Result<Asset> {
    let (vertices, triangles) = counts(source, loops, spans)?;
    source.validate()?;
    if handles
        .iter()
        .any(|x| !x.is_finite() || !(0.0001..=2.).contains(x))
    {
        return Err(invalid("Bridge handles require0.0001..2metres"));
    }
    let t = Topology::inspect(&source.geometry)?;
    if t.non_manifold_edges != 0 {
        return Err(invalid("Bridge requires manifold source edges"));
    }
    let n = loops[0].len();
    let mut used = BTreeSet::new();
    for (side, lp) in loops.iter().enumerate() {
        for (j, &a) in lp.iter().enumerate() {
            if a as usize >= source.geometry.positions.len() || !used.insert(a) {
                return Err(invalid(
                    "Bridge loops must be disjoint unique current vertices",
                ));
            }
            let b = lp[(j + 1) % n];
            let edge = if a < b { [a, b] } else { [b, a] };
            let e = t
                .edges
                .binary_search_by_key(&edge, |e| e.vertices)
                .ok()
                .map(|i| &t.edges[i])
                .ok_or_else(|| invalid("Bridge loop edge not found"))?;
            if e.faces.len() != 1 {
                return Err(invalid("Bridge needs two open boundary cycles"));
            }
            let tri = source.geometry.triangles[e.faces[0] as usize];
            let forward = (0..3).any(|i| tri[i] == a && tri[(i + 1) % 3] == b);
            if forward != (side == 1) {
                return Err(invalid(
                    "Bridge first loop must reverse its boundary, second must follow its boundary",
                ));
            }
        }
    }
    let mut normals = vec![DVec3::ZERO; source.geometry.positions.len()];
    for tri in &source.geometry.triangles {
        let [a, b, c] = tri.map(|i| v(source.geometry.positions[i as usize]));
        let normal = (b - a).cross(c - a);
        for &i in tri {
            normals[i as usize] += normal;
        }
    }
    let mut controls = Vec::with_capacity(n);
    for j in 0..n {
        let mut ends = [DVec3::ZERO; 2];
        let mut tangents = [DVec3::ZERO; 2];
        for side in 0..2 {
            let i = loops[side][j] as usize;
            ends[side] = v(source.geometry.positions[i]);
            let along = v(source.geometry.positions[loops[side][(j + 1) % n] as usize])
                - v(source.geometry.positions[loops[side][(j + n - 1) % n] as usize]);
            let tangent = normals[i].cross(along);
            if !tangent.is_finite() || tangent.length_squared() < 1e-16 {
                return Err(invalid("Bridge boundary tangent is unresolved"));
            }
            tangents[side] = tangent.normalize();
        }
        controls.push([
            ends[0],
            ends[0] + tangents[0] * f64::from(handles[0]),
            ends[1] - tangents[1] * f64::from(handles[1]),
            ends[1],
        ]);
    }
    let mut out = source.clone();
    let mut g = (*source.geometry).clone();
    g.positions.reserve(vertices - g.positions.len());
    g.triangles.reserve(triangles - g.triangles.len());
    let mut rig = source.rig.as_ref().map(|r| (**r).clone());
    let mut rows = Vec::with_capacity(spans as usize + 1);
    rows.push(loops[0].clone());
    for row in 1..spans {
        let t = parameter(row as usize, spans);
        let mut indices = Vec::with_capacity(n);
        for (j, &control) in controls.iter().enumerate() {
            indices.push(g.positions.len() as u32);
            g.positions
                .push(point(control, t).to_array().map(|x| x as f32));
            if let Some(coordinates) = &mut g.material_coordinates {
                let a =
                    source.geometry.material_coordinates.as_ref().unwrap()[loops[0][j] as usize];
                let b =
                    source.geometry.material_coordinates.as_ref().unwrap()[loops[1][j] as usize];
                coordinates.push(std::array::from_fn(|i| {
                    (f64::from(a[i]) * (1. - t) + f64::from(b[i]) * t) as f32
                }));
            }
            if let Some(rig) = &mut rig {
                rig.weights.push(crate::mesh_preservation::transfer_weight(
                    source.rig.as_ref().unwrap(),
                    [
                        loops[0][j] as usize,
                        loops[1][j] as usize,
                        loops[0][j] as usize,
                    ],
                    [1. - t, t, 0.],
                )?);
            }
        }
        rows.push(indices);
    }
    rows.push(loops[1].clone());
    for row in 0..spans as usize {
        for j in 0..n {
            let k = (j + 1) % n;
            let [a, b, c, d] = [
                rows[row][j],
                rows[row][k],
                rows[row + 1][j],
                rows[row + 1][k],
            ];
            for (tri, blend) in [([a, b, c], 1. / 3.), ([b, d, c], 2. / 3.)] {
                let q = parameter(row, spans) * (1. - blend) + parameter(row + 1, spans) * blend;
                let transverse = point(controls[k], q) - point(controls[j], q);
                // Evaluate the longitudinal derivative at the actual triangle centroid.
                let forward =
                    derivative(controls[j], q) * (1. - blend) + derivative(controls[k], q) * blend;
                let jacobian = transverse.cross(forward);
                let [x, y, z] = tri.map(|i| v(g.positions[i as usize]));
                let normal = (y - x).cross(z - x);
                if !jacobian.is_finite()
                    || jacobian.length_squared() < 1e-18
                    || normal.dot(jacobian) <= 1e-18
                {
                    return Err(invalid(
                        "Bridge folds or has unresolved sampled Jacobian; change phase/handles/spans",
                    ));
                }
                g.triangles.push(tri);
            }
        }
    }
    let topo = Topology::inspect(&g)?;
    if topo.non_manifold_edges != 0 || topo.boundary_edges + 2 * n != t.boundary_edges {
        return Err(invalid(
            "Bridge must consume exactly the two chosen boundary cycles",
        ));
    }
    // A loft can fold together with its own sampled Jacobian. Check adjacent
    // triangles as well, including the retained surface at both seams. This
    // deliberately conservative smooth-bridge contract rejects bends >=90deg.
    for edge in &topo.edges {
        if edge.faces.len() == 2
            && edge
                .faces
                .iter()
                .any(|&f| f as usize >= source.geometry.triangles.len())
        {
            let normal = |f: u32| {
                let [a, b, c] = g.triangles[f as usize].map(|i| v(g.positions[i as usize]));
                (b - a).cross(c - a)
            };
            if normal(edge.faces[0]).dot(normal(edge.faces[1])) <= 0. {
                return Err(invalid(
                    "Bridge adjacent triangle bend reaches90 degrees; change phase/handles/spans",
                ));
            }
        }
    }
    out.geometry = Arc::new(g);
    out.rig = rig.map(Arc::new);
    out.validate()?;
    Ok(out)
}
