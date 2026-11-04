//! Finite convex subtraction on cold authoring paths, with fail-closed publication.
use super::*;
use boolmesh::prelude::{Manifold, OpType, compute_boolean};
use rapier3d::{
    na::Point3,
    parry::{
        bounding_volume::BoundingVolume,
        shape::{TriMesh, TriMeshFlags},
        transformation::try_convex_hull,
    },
};

mod validation;

const MAX_INPUT_TRIANGLES: usize = 8192;
const MAX_PAIR_WORK: usize = 524_288;
const EPS: f64 = 2e-6;
const _: () = assert!(std::mem::size_of::<boolmesh::Real>() == 8);

pub(super) fn decode_points<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<[f32; 3]>, D::Error> {
    crate::terrain::limited::<D, [f32; 3], 32>(d)
}

fn points(g: &Geometry) -> Vec<Point3<f32>> {
    g.positions.iter().map(|p| Point3::from(*p)).collect()
}
fn mesh(g: &Geometry) -> Result<TriMesh> {
    TriMesh::with_flags(
        points(g),
        g.triangles.clone(),
        TriMeshFlags::HALF_EDGE_TOPOLOGY | TriMeshFlags::ORIENTED,
    )
    .map_err(|e| invalid(format!("Subtraction topology: {e}")))
}
fn volume(g: &Geometry) -> f64 {
    let pivot = v(g.positions[0]);
    let mut sum = 0.;
    let mut compensation = 0.;
    for f in &g.triangles {
        let [a, b, c] = f.map(|i| v(g.positions[i as usize]) - pivot);
        let value = a.dot(b.cross(c)) / 6. - compensation;
        let next = sum + value;
        compensation = (next - sum) - value;
        sum = next;
    }
    sum
}

fn closed(g: &Geometry) -> Result<()> {
    let t = Topology::new(g)?;
    if t.edges.values().any(|uses| uses.len() != 2) {
        return Err(invalid(
            "Subtraction requires closed oriented manifold edges",
        ));
    }
    let mut adjacent = vec![[usize::MAX; 3]; g.triangles.len()];
    let mut degree = vec![0; g.triangles.len()];
    let mut incident = vec![Vec::new(); g.positions.len()];
    for (f, tri) in g.triangles.iter().enumerate() {
        for &i in tri {
            incident[i as usize].push(f);
        }
    }
    for uses in t.edges.values() {
        for (a, b) in [(uses[0].face, uses[1].face), (uses[1].face, uses[0].face)] {
            adjacent[a][degree[a]] = b;
            degree[a] += 1;
        }
    }
    let mut seen = vec![false; g.triangles.len()];
    let mut queue = VecDeque::from([0]);
    seen[0] = true;
    while let Some(f) = queue.pop_front() {
        for &next in &adjacent[f] {
            if !seen[next] {
                seen[next] = true;
                queue.push_back(next);
            }
        }
    }
    if seen.iter().any(|x| !*x) || incident.iter().any(Vec::is_empty) {
        return Err(invalid(
            "Subtraction requires one connected surface without isolated vertices or enclosed cavities",
        ));
    }
    // A closed edge manifold can still have a pinched vertex. Walk each vertex fan.
    for (i, faces) in incident.iter().enumerate() {
        let mut previous = usize::MAX;
        let mut at = faces[0];
        let mut count = 0;
        loop {
            count += 1;
            let next = adjacent[at]
                .iter()
                .copied()
                .find(|&f| f != previous && g.triangles[f].contains(&(i as u32)))
                .ok_or_else(|| invalid("Subtraction requires a disk-like vertex fan"))?;
            previous = at;
            at = next;
            if at == faces[0] {
                break;
            }
            if count > faces.len() {
                return Err(invalid("Subtraction vertex fan does not close"));
            }
        }
        if count != faces.len() {
            return Err(invalid("Subtraction refuses pinched vertices"));
        }
    }
    if volume(g) <= EPS.powi(3) {
        return Err(invalid(
            "Subtraction requires positive outward-oriented volume",
        ));
    }
    Ok(())
}

fn self_intersections(g: &Geometry, m: &TriMesh) -> Result<()> {
    let mut work = 0;
    for (a, b) in m
        .bvh()
        .leaf_pairs(m.bvh(), |a, b| a.aabb().intersects(&b.aabb()))
    {
        if a >= b {
            continue;
        }
        work += 1;
        if work > MAX_PAIR_WORK {
            return Err(invalid(
                "Subtraction surface-validation work budget exceeded",
            ));
        }
        if validation::forbidden_intersection(g, a as usize, b as usize) {
            return Err(invalid(
                "Subtraction refuses self-intersecting or ambiguously touching surfaces",
            ));
        }
    }
    Ok(())
}

pub(super) fn evaluate(source: &Asset, cutter: &[[f32; 3]], limits: Limits) -> Result<Evaluated> {
    let g = &source.geometry;
    if source.rig.is_some() || !source.clips.is_empty() {
        return Err(invalid(
            "Subtraction currently requires a static unrigged mesh",
        ));
    }
    if g.material_coordinates
        .as_ref()
        .is_some_and(|c| c != &g.positions)
    {
        return Err(invalid(
            "Subtraction refuses custom material coordinates; use local procedural mapping",
        ));
    }
    if g.triangles.len() > MAX_INPUT_TRIANGLES || g.positions.len() > 8192 {
        return Err(invalid(
            "Subtraction input budget is 8192 vertices/triangles",
        ));
    }
    if !(4..=32).contains(&cutter.len())
        || cutter
            .iter()
            .flatten()
            .any(|x| !x.is_finite() || x.abs() > 10000.)
    {
        return Err(invalid(
            "Subtraction requires 4..32 finite local cutter points within +/-10000",
        ));
    }
    let min = g
        .positions
        .iter()
        .map(|p| v(*p))
        .fold(DVec3::splat(f64::INFINITY), DVec3::min);
    let max = g
        .positions
        .iter()
        .map(|p| v(*p))
        .fold(DVec3::splat(f64::NEG_INFINITY), DVec3::max);
    let center = (min + max) * 0.5;
    let scale = (max - min).max_element();
    if scale < 1e-4 {
        return Err(invalid("Subtraction source span is too small"));
    }
    let normalize = |p| array((v(p) - center) / scale);
    // Match simplification to the precision of the document we will publish.
    // f64 topology alone can retain slivers that become exactly collinear in f32.
    let packing_epsilon = f64::from(f32::EPSILON) * (1. + center.abs().max_element() / scale);
    if packing_epsilon > 0.001 {
        return Err(invalid(
            "Subtraction source requires a closer local origin for f32 precision",
        ));
    }
    let normalized = Geometry {
        positions: g.positions.iter().map(|p| normalize(*p)).collect(),
        triangles: g.triangles.clone(),
        material_coordinates: None,
    };
    closed(&normalized)?;
    let original_mesh = mesh(&normalized)?;
    self_intersections(&normalized, &original_mesh)?;
    let cut_points: Vec<_> = cutter.iter().map(|p| Point3::from(normalize(*p))).collect();
    if cut_points.iter().any(|p| p.coords.abs().max() > 4.) {
        return Err(invalid(
            "Subtraction cutter must remain within four source spans of its center",
        ));
    }
    let (cut_positions, cut_triangles) = try_convex_hull(&cut_points)
        .map_err(|e| invalid(format!("Subtraction cutter hull: {e:?}")))?;
    let hull = Geometry {
        positions: cut_positions.iter().map(|p| [p.x, p.y, p.z]).collect(),
        triangles: cut_triangles,
        material_coordinates: None,
    };
    closed(&hull)?;
    if volume(&hull) < 1e-9 {
        return Err(invalid(
            "Subtraction cutter is too thin at the source scale",
        ));
    }
    if normalized
        .triangles
        .len()
        .saturating_mul(hull.triangles.len())
        > MAX_PAIR_WORK
    {
        return Err(invalid("Subtraction Boolean pair-work budget exceeded"));
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let manifold = |g: &Geometry| {
            let mut m = Manifold::new(
                &g.positions
                    .iter()
                    .flatten()
                    .map(|&x| f64::from(x))
                    .collect::<Vec<_>>(),
                &g.triangles
                    .iter()
                    .flatten()
                    .map(|&i| i as usize)
                    .collect::<Vec<_>>(),
            )?;
            // Reviewed public numerical fields in pinned Boolmesh0.2.2. Vertex,
            // face sorting and collider contents are not modified.
            m.eps = packing_epsilon;
            m.tol = packing_epsilon;
            Ok::<_, String>(m)
        };
        compute_boolean(&manifold(&normalized)?, &manifold(&hull)?, OpType::Subtract)
    }))
    .map_err(|_| invalid("Subtraction kernel refused an ambiguous intersection"))?
    .map_err(|e| invalid(format!("Subtraction kernel: {e}")))?;
    check_counts(result.ps.len(), result.hs.len() / 3, limits)?;
    if result.ps.len() < 4 || result.hs.is_empty() {
        return Err(invalid("Subtraction would remove the entire solid"));
    }
    let mut out = Geometry {
        positions: result
            .ps
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32])
            .collect(),
        triangles: result
            .hs
            .chunks_exact(3)
            .map(|h| [h[0].tail as u32, h[1].tail as u32, h[2].tail as u32])
            .collect(),
        material_coordinates: None,
    };
    if out
        .positions
        .iter()
        .any(|p| v(*p).abs().max_element() > 0.5 + packing_epsilon * 4.)
    {
        return Err(invalid(
            "Subtraction result exceeds its source bounding cube",
        ));
    }
    closed(&out)?;
    self_intersections(&out, &mesh(&out)?)?;
    let before = volume(&normalized);
    let after = volume(&out);
    // Roundoff floor in the normalized unit bounding cube, not a minimum fraction
    // of the whole object's volume: small local chips must remain possible.
    let volume_floor =
        64. * f64::EPSILON * (normalized.triangles.len() + out.triangles.len()) as f64;
    if before - after <= volume_floor || after >= before {
        return Err(invalid(
            "Subtraction must remove a measurable positive volume",
        ));
    }
    // Source vertices survive bit-exactly; only new intersections are denormalized.
    let surviving: BTreeMap<_, _> = normalized
        .positions
        .iter()
        .zip(&g.positions)
        .map(|(p, old)| (p.map(f32::to_bits), *old))
        .collect();
    for p in &mut out.positions {
        *p = surviving
            .get(&p.map(f32::to_bits))
            .copied()
            .unwrap_or_else(|| array(v(*p) * scale + center));
    }
    // Stable vertex/face ordering is independent of the kernel's spatial/hash traversal.
    let mut order: Vec<_> = (0..out.positions.len()).collect();
    order.sort_by(|&a, &b| {
        for axis in 0..3 {
            let c = out.positions[a][axis].total_cmp(&out.positions[b][axis]);
            if c != std::cmp::Ordering::Equal {
                return c;
            }
        }
        a.cmp(&b)
    });
    let mut mapping = vec![0; order.len()];
    for (new, &old) in order.iter().enumerate() {
        mapping[old] = new as u32;
    }
    out.positions = order.iter().map(|&i| out.positions[i]).collect();
    for f in &mut out.triangles {
        *f = f.map(|i| mapping[i as usize]);
        let min = (0..3).min_by_key(|&i| f[i]).unwrap();
        f.rotate_left(min);
    }
    out.triangles.sort_unstable();
    // f32 denormalization can collapse features: validate the actual published geometry.
    closed(&out)?;
    let actual = Geometry {
        positions: out.positions.iter().map(|p| normalize(*p)).collect(),
        triangles: out.triangles.clone(),
        material_coordinates: None,
    };
    actual.validate()?;
    if before - volume(&actual) <= volume_floor {
        return Err(invalid(
            "Subtraction loses measurable removal at publication precision",
        ));
    }
    self_intersections(&actual, &mesh(&actual)?)?;
    let mut selected_faces = vec![];
    for (f, triangle) in actual.triangles.iter().enumerate() {
        let vertices = triangle.map(|i| v(actual.positions[i as usize]));
        let n = (vertices[1] - vertices[0])
            .cross(vertices[2] - vertices[0])
            .normalize();
        if hull.triangles.iter().any(|face| {
            let [a, b, c] = face.map(|i| v(hull.positions[i as usize]));
            let hn = (b - a).cross(c - a).normalize();
            n.dot(hn) < -0.99
                && vertices
                    .iter()
                    .all(|p| hn.dot(*p - a).abs() < EPS.max(packing_epsilon) * 4.)
        }) {
            selected_faces.push(f as u32);
        }
    }
    if selected_faces.is_empty() {
        return Err(invalid("Subtraction produced no exposed cut surface"));
    }
    if g.material_coordinates.is_some() {
        out.material_coordinates = Some(out.positions.clone());
    }
    let mut asset = source.clone();
    asset.geometry = Arc::new(out);
    meshes::Document {
        objects: Default::default(),
        assets: vec![asset.clone()],
        instances: vec![],
    }
    .validate()?;
    Ok(Evaluated {
        report: Report {
            source_vertices: g.positions.len(),
            source_triangles: g.triangles.len(),
            vertices: asset.geometry.positions.len(),
            triangles: asset.geometry.triangles.len(),
            selected_faces,
            effective_edges: vec![],
            numerical_tolerance_meters: Some(packing_epsilon * scale),
        },
        asset,
    })
}
