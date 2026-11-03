//! One-segment convex edge-chain chamfer by indexed half-space clipping.
use super::*;
#[derive(Clone, Copy)]
struct Plane {
    normal: DVec3,
    k: f64,
}
fn plane_matches(a: Plane, b: Plane, eps: f64) -> bool {
    a.normal.dot(b.normal) > 1. - 1e-8 && (a.k - b.k).abs() <= eps
}
pub(super) fn evaluate(
    source: &Asset,
    edges: &[[u32; 2]],
    width: f32,
    extend: bool,
    limits: Limits,
) -> Result<Evaluated> {
    let g = &source.geometry;
    if edges.is_empty()
        || edges.len() > 4096
        || !width.is_finite()
        || !(0.00001..=100.).contains(&width)
    {
        return Err(invalid(
            "Bevel requires 1..4096 edges and width 0.00001..100",
        ));
    }
    if g.positions.len() > 4096 || g.triangles.len() > 8192 {
        return Err(invalid(
            "Convex bevel input budget is 4096 vertices/8192 triangles",
        ));
    }
    let topology = Topology::new(g)?;
    if topology.edges.values().any(|uses| uses.len() != 2) {
        return Err(invalid("Bevel requires a closed oriented solid"));
    }
    let mut used = vec![false; g.positions.len()];
    for face in &g.triangles {
        for &i in face {
            used[i as usize] = true;
        }
    }
    if used.iter().any(|x| !*x) {
        return Err(invalid("Bevel refuses isolated vertices"));
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
    let eps = (max - min).length().max(0.001) * 1e-7;
    let mut planes = vec![];
    let mut face_groups = vec![];
    for f in 0..g.triangles.len() {
        let n = normal(g, f).normalize();
        let plane = Plane {
            normal: n,
            k: n.dot(v(g.positions[g.triangles[f][0] as usize])),
        };
        let group = if let Some(i) = planes.iter().position(|p| plane_matches(*p, plane, eps)) {
            i
        } else {
            if planes.len() >= 128 {
                return Err(invalid("Bevel input exceeds 128 geometric face planes"));
            }
            planes.push(plane);
            planes.len() - 1
        };
        face_groups.push(group);
    }
    if planes.iter().any(|p| {
        g.positions
            .iter()
            .any(|vtx| p.normal.dot(v(*vtx)) - p.k > eps * 4.)
    }) {
        return Err(invalid("Bevel currently requires a convex solid"));
    }
    // A consistently oriented convex surface can still contain disconnected or
    // coincident components. Reject those instead of beveling hidden components.
    let mut adjacent = vec![vec![]; g.triangles.len()];
    for uses in topology.edges.values() {
        adjacent[uses[0].face].push(uses[1].face);
        adjacent[uses[1].face].push(uses[0].face);
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
    if seen.iter().any(|v| !*v) {
        return Err(invalid("Bevel requires one connected convex solid"));
    }
    let mut selected = BTreeSet::new();
    let mut groups = BTreeSet::new();
    for &[a, b] in edges {
        let key = edge(a, b);
        if a == b || !selected.insert(key) {
            return Err(invalid("Bevel contains duplicate or collapsed edges"));
        }
        let uses = topology
            .edges
            .get(&key)
            .ok_or_else(|| invalid("Bevel edge does not exist"))?;
        let [a, b] = [face_groups[uses[0].face], face_groups[uses[1].face]];
        if a == b {
            return Err(invalid(
                "Choose an edge between different face planes, not a flat triangulation diagonal",
            ));
        }
        groups.insert(if a < b { [a, b] } else { [b, a] });
    }
    if groups.len() > 32 {
        return Err(invalid("Bevel exceeds 32 geometric edge chains"));
    }
    let mut effective = vec![];
    for (edge, uses) in &topology.edges {
        let [a, b] = [face_groups[uses[0].face], face_groups[uses[1].face]];
        let pair = if a < b { [a, b] } else { [b, a] };
        if groups.contains(&pair) {
            if !extend && !selected.contains(edge) {
                return Err(invalid(
                    "Select the complete collinear edge chain or explicitly enable extension",
                ));
            }
            effective.push(*edge);
        }
    }
    let mut cuts = vec![];
    for [a, b] in groups {
        let p = planes[a];
        let q = planes[b];
        let dot = p.normal.dot(q.normal).clamp(-1., 1.);
        let n = (p.normal + q.normal).normalize_or_zero();
        if n == DVec3::ZERO {
            return Err(invalid("Bevel edge has opposite face planes"));
        }
        let source_edge = effective
            .iter()
            .find(|e| {
                let uses = &topology.edges[*e];
                let x = face_groups[uses[0].face];
                let y = face_groups[uses[1].face];
                (x == a && y == b) || (x == b && y == a)
            })
            .unwrap();
        let cut = Plane {
            normal: n,
            k: n.dot(v(g.positions[source_edge[0] as usize]))
                - f64::from(width) * ((1. - dot) * 0.5).sqrt(),
        };
        if !cuts.iter().any(|p| plane_matches(*p, cut, eps)) {
            cuts.push(cut);
        }
    }
    let mut builder = Builder::new(source, limits);
    builder.effective_edges = effective;
    for (i, &f) in g.triangles.iter().enumerate() {
        builder.triangle(f, false, Some(face_groups[i]))?;
    }
    for cut in cuts {
        clip(&mut builder, cut, eps)?;
        builder.compact();
    }
    let covered: BTreeSet<_> = builder.faces.iter().filter_map(|f| f.group).collect();
    if covered.len() != planes.len() {
        return Err(invalid(
            "Bevel width consumes an original face; reduce width",
        ));
    }
    let result = builder.finish()?;
    if Topology::new(&result.asset.geometry)?
        .edges
        .values()
        .any(|e| e.len() != 2)
    {
        return Err(invalid("Bevel would open the solid"));
    }
    Ok(result)
}
fn interpolate(builder: &mut Builder<'_>, a: u32, b: u32, t: f64) -> Result<u32> {
    let va = &builder.vertices[a as usize];
    let vb = &builder.vertices[b as usize];
    let next = Vertex {
        position: va.position.lerp(vb.position, t),
        material: va.material.lerp(vb.material, t),
        weight: match (&va.weight, &vb.weight) {
            (Some(a), Some(b)) => Some(blended_weight(
                [(a.clone(), 1. - t), (b.clone(), t)].into_iter(),
            )?),
            _ => None,
        },
    };
    builder.push(next)
}
fn intersection(
    builder: &mut Builder<'_>,
    cache: &mut BTreeMap<[u32; 2], u32>,
    cut: Plane,
    a: u32,
    b: u32,
    eps: f64,
) -> Result<u32> {
    let da = cut.k - cut.normal.dot(builder.vertices[a as usize].position);
    let db = cut.k - cut.normal.dot(builder.vertices[b as usize].position);
    if da.abs() <= eps {
        return Ok(a);
    }
    if db.abs() <= eps {
        return Ok(b);
    }
    let key = edge(a, b);
    if let Some(&i) = cache.get(&key) {
        return Ok(i);
    }
    let t = (da / (da - db)).clamp(0., 1.);
    let i = interpolate(builder, a, b, t)?;
    let p = &mut builder.vertices[i as usize].position;
    *p -= cut.normal * (cut.normal.dot(*p) - cut.k);
    cache.insert(key, i);
    Ok(i)
}
fn clip(builder: &mut Builder<'_>, cut: Plane, eps: f64) -> Result<()> {
    let old = std::mem::take(&mut builder.faces);
    let mut cache = BTreeMap::new();
    let mut caps = BTreeSet::new();
    let mut removed = false;
    for face in old {
        let mut polygon = vec![];
        for j in 0..3 {
            let a = face.vertices[j];
            let b = face.vertices[(j + 1) % 3];
            let da = cut.k - cut.normal.dot(builder.vertices[a as usize].position);
            let db = cut.k - cut.normal.dot(builder.vertices[b as usize].position);
            let (inside_a, inside_b) = (da >= -eps, db >= -eps);
            if inside_a {
                polygon.push(a);
            } else {
                removed = true;
            }
            if inside_a != inside_b {
                polygon.push(intersection(builder, &mut cache, cut, a, b, eps)?);
            }
        }
        polygon.dedup();
        if polygon.len() > 1 && polygon.first() == polygon.last() {
            polygon.pop();
        }
        if polygon.len() < 3 {
            continue;
        }
        for &i in &polygon {
            if (cut.k - cut.normal.dot(builder.vertices[i as usize].position)).abs() <= eps * 2. {
                caps.insert(i);
            }
        }
        for j in 1..polygon.len() - 1 {
            let tri = [polygon[0], polygon[j], polygon[j + 1]];
            let [a, b, c] = tri.map(|i| builder.vertices[i as usize].position);
            if (b - a).cross(c - a).length() > eps * eps {
                builder.triangle(tri, face.selected, face.group)?;
            }
        }
    }
    if !removed || builder.faces.is_empty() || caps.len() < 3 {
        return Err(invalid("Bevel cut has no valid bounded cross section"));
    }
    let mut ring: Vec<_> = caps.into_iter().collect();
    let center = ring
        .iter()
        .map(|i| builder.vertices[*i as usize].position)
        .sum::<DVec3>()
        / ring.len() as f64;
    let axis = if cut.normal.x.abs() < 0.8 {
        DVec3::X
    } else {
        DVec3::Y
    };
    let u = cut.normal.cross(axis).normalize();
    let w = cut.normal.cross(u);
    ring.sort_by(|a, b| {
        let a = builder.vertices[*a as usize].position - center;
        let b = builder.vertices[*b as usize].position - center;
        a.dot(w)
            .atan2(a.dot(u))
            .total_cmp(&b.dot(w).atan2(b.dot(u)))
            .then_with(|| a.length_squared().total_cmp(&b.length_squared()))
    });
    let material = ring
        .iter()
        .map(|i| builder.vertices[*i as usize].material)
        .sum::<DVec3>()
        / ring.len() as f64;
    let weight = if builder.source.rig.is_some() {
        Some(blended_weight(ring.iter().map(|i| {
            (
                builder.vertices[*i as usize].weight.clone().unwrap(),
                1. / ring.len() as f64,
            )
        }))?)
    } else {
        None
    };
    let center = builder.push(Vertex {
        position: center,
        material,
        weight,
    })?;
    for j in 0..ring.len() {
        builder.triangle([ring[j], ring[(j + 1) % ring.len()], center], true, None)?;
    }
    Ok(())
}
