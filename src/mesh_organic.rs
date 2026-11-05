//! Bounded three-collar junctions. New interior faces have no implicit source map.
use crate::{
    mesh_topology::{Components, Plan, Topology},
    meshes::Geometry,
    model::{Result, invalid},
};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Collar {
    pub id: String,
    pub center: [f32; 3],
    /// Unit exterior normal; its plane must strictly support all other collars.
    pub normal: [f32; 3],
    pub radial: [f32; 3],
    pub radius: f32,
    pub samples: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    pub refinement_levels: u8,
    pub relaxation_steps: u8,
    pub relaxation_factor: f32,
    pub max_volume_loss: f32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            refinement_levels: 1,
            relaxation_steps: 2,
            relaxation_factor: 0.25,
            max_volume_loss: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Boundary {
    pub id: String,
    /// Original CCW sample order. Adjacent surfaces must reuse these indices.
    pub vertices: Vec<u32>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub vertices: usize,
    pub triangles: usize,
    pub boundary_edges: usize,
    pub euler_characteristic: i64,
    /// Volume with the three auxiliary exterior caps; caps are not output faces.
    pub initial_volume: f64,
    pub final_volume: f64,
    pub volume_loss: f64,
    pub max_interior_movement: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<FieldReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldReport {
    pub fixed_vertices: usize,
    pub max_residual: f64,
    pub max_face_rotation_degrees: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_lengths: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Fusion {
    pub center: [f32; 3],
    pub center_radius: f32,
    pub blend_radius: f32,
    pub guard_length: f32,
    /// Metres after each collar's guard, in input order. All zero is legacy.
    pub transition_lengths: [f32; 3],
    pub max_displacement: f32,
    pub tolerance: f32,
    pub max_volume_loss: f32,
    pub max_face_rotation_degrees: f32,
}
impl Default for Fusion {
    fn default() -> Self {
        Self {
            center: [0.; 3],
            center_radius: 0.4,
            blend_radius: 0.12,
            guard_length: 0.12,
            transition_lengths: [0.; 3],
            max_displacement: 0.75,
            tolerance: 0.0001,
            max_volume_loss: 0.5,
            max_face_rotation_degrees: 80.,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Junction {
    pub geometry: Geometry,
    pub boundaries: [Boundary; 3],
    /// Only this prefix has boundary provenance; generated interior vertices do not.
    pub collar_vertices: usize,
    pub report: Report,
}
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}

fn samples(collars: &[Collar; 3]) -> Result<(Geometry, [Boundary; 3], Vec<DVec3>, f64)> {
    let (geometry, boundaries, points, epsilon) = samples_many(collars)?;
    Ok((
        geometry,
        boundaries
            .try_into()
            .map_err(|_| invalid("Three collars required"))?,
        points,
        epsilon,
    ))
}
fn samples_many(collars: &[Collar]) -> Result<(Geometry, Vec<Boundary>, Vec<DVec3>, f64)> {
    let mut ids = BTreeSet::new();
    let mut geometry = Geometry {
        material_coordinates: None,
        positions: vec![],
        triangles: vec![],
    };
    let mut points = Vec::new();
    let mut boundaries: Vec<Boundary> = collars
        .iter()
        .map(|c| Boundary {
            id: c.id.clone(),
            vertices: vec![],
        })
        .collect();
    for (c, b) in collars.iter().zip(&mut boundaries) {
        if c.id.is_empty()
            || c.id.len() > 64
            || !c
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            || !ids.insert(&c.id)
            || c.center
                .iter()
                .chain(&c.normal)
                .chain(&c.radial)
                .any(|x| !x.is_finite())
            || !c.radius.is_finite()
            || c.radius < 0.001
            || !(3..=32).contains(&c.samples)
        {
            return Err(invalid(
                "Organic collars require unique ASCII IDs, finite frames, radius>=0.001 and3..32 samples",
            ));
        }
        let normal = v(c.normal);
        let radial = v(c.radial);
        if (normal.length() - 1.).abs() > 1e-5
            || (radial.length() - 1.).abs() > 1e-5
            || normal.dot(radial).abs() > 1e-5
        {
            return Err(invalid(
                "Organic collar frame must have perpendicular unit normal and radial",
            ));
        }
        let normal = normal.normalize();
        let radial = (radial - normal * normal.dot(radial)).normalize();
        let other = normal.cross(radial);
        for j in 0..c.samples {
            let theta = std::f64::consts::TAU * f64::from(j) / f64::from(c.samples);
            let p =
                v(c.center) + f64::from(c.radius) * (radial * theta.cos() + other * theta.sin());
            if !p.is_finite() || p.abs().max_element() > 10000. {
                return Err(invalid(
                    "Organic collar exceeds mesh coordinate domain +/-10000m",
                ));
            }
            b.vertices.push(geometry.positions.len() as u32);
            points.push(p);
            geometry.positions.push(p.to_array().map(|x| x as f32));
        }
    }
    let minimum = geometry
        .positions
        .iter()
        .copied()
        .map(v)
        .fold(DVec3::splat(f64::INFINITY), DVec3::min);
    let maximum = geometry
        .positions
        .iter()
        .copied()
        .map(v)
        .fold(DVec3::splat(f64::NEG_INFINITY), DVec3::max);
    let magnitude = maximum.abs().max(minimum.abs()).max_element();
    let epsilon =
        (maximum - minimum).max_element() * 1e-7 + magnitude * f64::from(f32::EPSILON) * 2. + 1e-10;
    if collars.iter().any(|c| f64::from(c.radius) <= epsilon * 32.) {
        return Err(invalid(
            "Organic collar radius is unresolved at the coordinate precision",
        ));
    }
    for (i, c) in collars.iter().enumerate() {
        for (j, b) in boundaries.iter().enumerate() {
            if i != j {
                for &p in &b.vertices {
                    if v(c.normal).dot(v(geometry.positions[p as usize]) - v(c.center))
                        >= -epsilon * 8.
                    {
                        return Err(invalid(format!(
                            "Organic collar {} is not a separated support plane; move collars or use another junction domain",
                            c.id
                        )));
                    }
                }
            }
        }
    }
    Ok((geometry, boundaries, points, epsilon))
}

fn hull(
    geometry: &mut Geometry,
    boundaries: &[Boundary],
    points: &[DVec3],
    epsilon: f64,
) -> Result<()> {
    let mut facets: BTreeMap<Vec<u32>, DVec3> = BTreeMap::new();
    for a in 0..points.len() {
        for b in a + 1..points.len() {
            for c in b + 1..points.len() {
                let n = (points[b] - points[a]).cross(points[c] - points[a]);
                if n.length() <= epsilon * epsilon {
                    continue;
                }
                let mut normal = n.normalize();
                let mut positive = false;
                let mut negative = false;
                for &p in points {
                    let d = normal.dot(p - points[a]);
                    positive |= d > epsilon;
                    negative |= d < -epsilon;
                    if positive && negative {
                        break;
                    }
                }
                if positive && negative {
                    continue;
                }
                if !positive && !negative {
                    return Err(invalid(
                        "Organic collars are coplanar or numerically unresolved",
                    ));
                }
                if positive {
                    normal = -normal;
                }
                let face: Vec<_> = points
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &p)| {
                        (normal.dot(p - points[a]).abs() <= epsilon).then_some(i as u32)
                    })
                    .collect();
                facets.entry(face).or_insert(normal);
            }
        }
    }
    for (face, normal) in facets {
        if boundaries.iter().any(|b| b.vertices == face) {
            continue;
        }
        let center = face.iter().map(|&i| points[i as usize]).sum::<DVec3>() / face.len() as f64;
        let u = (points[face[0] as usize] - center).normalize();
        let w = normal.cross(u);
        let mut polygon = face.clone();
        polygon.sort_by(|&a, &b| {
            let a = points[a as usize] - center;
            let b = points[b as usize] - center;
            a.dot(w)
                .atan2(a.dot(u))
                .total_cmp(&b.dot(w).atan2(b.dot(u)))
        });
        let first = polygon
            .iter()
            .position(|&i| i == face[0])
            .expect("facet vertex");
        polygon.rotate_left(first);
        for j in 1..polygon.len() - 1 {
            let tri = [polygon[0], polygon[j], polygon[j + 1]];
            let [a, b, c] = tri.map(|i| points[i as usize]);
            if (b - a).cross(c - a).dot(normal) <= epsilon * epsilon {
                return Err(invalid(
                    "Organic hull facet has collinear or interior collar samples",
                ));
            }
            geometry.triangles.push(tri);
        }
    }
    Ok(())
}

fn topology(geometry: &Geometry, boundaries: &[Boundary]) -> Result<Topology> {
    let t = Topology::inspect(geometry)?;
    let mut boundary = BTreeSet::new();
    for b in boundaries {
        for i in 0..b.vertices.len() {
            boundary.insert([b.vertices[(i + 1) % b.vertices.len()], b.vertices[i]]);
        }
    }
    if !t.isolated_vertices.is_empty()
        || t.non_manifold_edges != 0
        || t.boundary_edges != boundary.len()
        || t.vertices as i64 - t.edges.len() as i64 + t.faces.len() as i64
            != 2 - boundaries.len() as i64
    {
        return Err(invalid(format!(
            "Organic junction requires exact boundaries and Euler2-N: boundary{} expected{}, nonmanifold{}, isolated{}, Euler{}",
            t.boundary_edges,
            boundary.len(),
            t.non_manifold_edges,
            t.isolated_vertices.len(),
            t.vertices as i64 - t.edges.len() as i64 + t.faces.len() as i64
        )));
    }
    let mut adjacent = vec![vec![]; geometry.positions.len()];
    for e in &t.edges {
        let [a, b] = e.vertices;
        let directed: Vec<_> = e
            .faces
            .iter()
            .map(|&f| {
                let tri = geometry.triangles[f as usize];
                if (0..3).any(|i| tri[i] == a && tri[(i + 1) % 3] == b) {
                    [a, b]
                } else {
                    [b, a]
                }
            })
            .collect();
        if (directed.len() == 1 && !boundary.contains(&directed[0]))
            || (directed.len() == 2 && directed[0] == directed[1])
        {
            return Err(invalid(
                "Organic junction boundary or interior edge has inconsistent orientation",
            ));
        }
        adjacent[a as usize].push(b);
        adjacent[b as usize].push(a);
    }
    let mut seen = vec![false; geometry.positions.len()];
    let mut stack = vec![0u32];
    seen[0] = true;
    while let Some(i) = stack.pop() {
        for &j in &adjacent[i as usize] {
            if !seen[j as usize] {
                seen[j as usize] = true;
                stack.push(j);
            }
        }
    }
    if seen.iter().any(|&s| !s) {
        return Err(invalid("Organic junction is disconnected"));
    }
    Ok(t)
}
fn volume(geometry: &Geometry, boundaries: &[Boundary]) -> f64 {
    // Translate to a local reference to reduce cancellation for distant objects.
    let origin = v(geometry.positions[0]);
    let tetra = |[a, b, c]: [u32; 3]| {
        let [a, b, c] = [a, b, c].map(|i| v(geometry.positions[i as usize]) - origin);
        a.dot(b.cross(c)) / 6.
    };
    let mut sum: f64 = geometry.triangles.iter().copied().map(tetra).sum();
    for b in boundaries {
        for j in 1..b.vertices.len() - 1 {
            sum += tetra([b.vertices[0], b.vertices[j], b.vertices[j + 1]]);
        }
    }
    sum
}

/// Returns a pair-of-pants surface with exact collar boundaries. Inputs never mutate.
#[derive(Debug, Clone)]
pub struct MultiJunction {
    pub geometry: Geometry,
    pub boundaries: Vec<Boundary>,
    pub collar_vertices: usize,
    pub report: Report,
}
impl From<Junction> for MultiJunction {
    fn from(j: Junction) -> Self {
        Self {
            geometry: j.geometry,
            boundaries: j.boundaries.into_iter().collect(),
            collar_vertices: j.collar_vertices,
            report: j.report,
        }
    }
}
pub fn junction(collars: &[Collar; 3], options: Options) -> Result<Junction> {
    let j = junction_many(collars, options)?;
    Ok(Junction {
        geometry: j.geometry,
        boundaries: j
            .boundaries
            .try_into()
            .map_err(|_| invalid("Three collars required"))?,
        collar_vertices: j.collar_vertices,
        report: j.report,
    })
}
pub fn junction_many(collars: &[Collar], options: Options) -> Result<MultiJunction> {
    if !(2..=13).contains(&collars.len())
        || collars.iter().map(|c| c.samples as usize).sum::<usize>() > 160
    {
        return Err(invalid(
            "Multicollar junction requires2..13 collars and at most160 boundary samples",
        ));
    }
    if options.refinement_levels > 3
        || options.relaxation_steps > 8
        || !options.relaxation_factor.is_finite()
        || !(0.0..=0.5).contains(&options.relaxation_factor)
        || !options.max_volume_loss.is_finite()
        || !(0.0..=0.25).contains(&options.max_volume_loss)
    {
        return Err(invalid(
            "Organic options require0..3 refinement levels,0..8 relaxations, factor0..0.5 and volume loss0..0.25",
        ));
    }
    let (mut geometry, boundaries, points, epsilon) = samples_many(collars)?;
    let collar_vertices = geometry.positions.len();
    let magnitude = points
        .iter()
        .map(|p| p.abs().max_element())
        .fold(0f64, f64::max);
    let hull_epsilon = epsilon * 0.001 + magnitude * f64::EPSILON * 64.;
    hull(&mut geometry, &boundaries, &points, hull_epsilon)?;
    topology(&geometry, &boundaries)?;
    let initial_volume = volume(&geometry, &boundaries);
    if initial_volume <= epsilon.powi(3) || !initial_volume.is_finite() {
        return Err(invalid(
            "Organic junction has unresolved or nonpositive enclosed volume",
        ));
    }
    for _ in 0..options.refinement_levels {
        let t = topology(&geometry, &boundaries)?;
        let components = Components::Edges {
            vertices: t
                .edges
                .iter()
                .filter(|e| e.faces.len() == 2)
                .map(|e| e.vertices)
                .collect(),
        };
        geometry = Plan::new(&geometry, Some(&components))?.geometry(&geometry)?;
    }
    let baseline = geometry.positions.clone();
    let t = topology(&geometry, &boundaries)?;
    let mut adjacent = vec![vec![]; geometry.positions.len()];
    for e in &t.edges {
        let [a, b] = e.vertices;
        adjacent[a as usize].push(b as usize);
        adjacent[b as usize].push(a as usize);
    }
    let mut max_interior_movement: f64 = 0.;
    for _ in 0..options.relaxation_steps {
        let mut next = geometry.positions.clone();
        for i in collar_vertices..geometry.positions.len() {
            let mean = adjacent[i]
                .iter()
                .map(|&j| v(geometry.positions[j]))
                .sum::<DVec3>()
                / adjacent[i].len() as f64;
            let p = v(geometry.positions[i]).lerp(mean, f64::from(options.relaxation_factor));
            next[i] = p.to_array().map(|x| x as f32);
            max_interior_movement = max_interior_movement.max(p.distance(v(baseline[i])));
        }
        for &tri in &geometry.triangles {
            let [a, b, c] = tri.map(|i| v(geometry.positions[i as usize]));
            let [x, y, z] = tri.map(|i| v(next[i as usize]));
            if (b - a).cross(c - a).dot((y - x).cross(z - x)) <= 0. {
                return Err(invalid(
                    "Organic relaxation inverts or collapses a local face",
                ));
            }
        }
        geometry.positions = next;
    }
    let t = topology(&geometry, &boundaries)?;
    let final_volume = volume(&geometry, &boundaries);
    let volume_loss = (initial_volume - final_volume) / initial_volume;
    if !final_volume.is_finite()
        || final_volume <= 0.
        || volume_loss > f64::from(options.max_volume_loss) + 1e-6
        || volume_loss < -1e-5
    {
        return Err(invalid(format!(
            "Organic relaxation volume loss{volume_loss} exceeds envelope{}",
            options.max_volume_loss
        )));
    }
    Ok(MultiJunction {
        report: Report {
            vertices: geometry.positions.len(),
            triangles: geometry.triangles.len(),
            boundary_edges: t.boundary_edges,
            euler_characteristic: 2 - boundaries.len() as i64,
            initial_volume,
            final_volume,
            volume_loss,
            max_interior_movement,
            field: None,
        },
        geometry,
        boundaries,
        collar_vertices,
    })
}

fn fusion_field(point: DVec3, collars: &[Collar; 3], fusion: Fusion, order: [usize; 3]) -> f64 {
    let center = v(fusion.center);
    let mut values = [0.; 3];
    for (i, c) in collars.iter().enumerate() {
        let axis = v(c.center) - center;
        let t = ((point - center).dot(axis) / axis.length_squared()).clamp(0., 1.);
        let radius = f64::from(fusion.center_radius) * (1. - t) + f64::from(c.radius) * t;
        values[i] = (point - center - axis * t).length() - radius;
    }
    let k = f64::from(fusion.blend_radius);
    let base = order
        .into_iter()
        .map(|i| values[i])
        .reduce(|a, b| {
            let h = (0.5 + 0.5 * (b - a) / k).clamp(0., 1.);
            b * (1. - h) + a * h - k * h * (1. - h)
        })
        .expect("three fields");
    if fusion.transition_lengths == [0.; 3] {
        return base;
    }
    order.into_iter().fold(base, |field, i| {
        let c = &collars[i];
        let normal = v(c.normal).normalize();
        let relative = point - v(c.center);
        let axial = relative.dot(normal);
        let u = ((-axial - f64::from(fusion.guard_length))
            / f64::from(fusion.transition_lengths[i]))
        .clamp(0., 1.);
        let weight = u.powi(3) * (10. + u * (-15. + 6. * u));
        let cylinder = (relative - normal * axial).length() - f64::from(c.radius);
        cylinder * (1. - weight) + field * weight
    })
}

/// Fit a star-shaped field while protecting the exact collars and cylindrical
/// guard bands. Generated interior vertices have no authored-source transfer.
pub fn fused_junction(collars: &[Collar; 3], options: Options, fusion: Fusion) -> Result<Junction> {
    if options.refinement_levels == 0 {
        return Err(invalid("Organic fusion requires interior refinement"));
    }
    if fusion.center.iter().any(|v| !v.is_finite())
        || [
            fusion.center_radius,
            fusion.blend_radius,
            fusion.guard_length,
            fusion.max_displacement,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v < 0.001)
        || !fusion.tolerance.is_finite()
        || !(0.000001..=0.01).contains(&fusion.tolerance)
        || !fusion.max_volume_loss.is_finite()
        || !(0.0..=0.75).contains(&fusion.max_volume_loss)
        || !fusion.max_face_rotation_degrees.is_finite()
        || !(0.1..=85.).contains(&fusion.max_face_rotation_degrees)
        || fusion.blend_radius > fusion.center_radius
    {
        return Err(invalid(
            "Invalid organic fusion center, radii, guards, tolerance or envelopes",
        ));
    }
    let (outer, boundaries, _, epsilon) = samples(collars)?;
    let center = v(fusion.center);
    let transition = fusion.transition_lengths != [0.; 3];
    if transition {
        for (j, c) in collars.iter().enumerate() {
            let length = fusion.transition_lengths[j];
            let normal = v(c.normal).normalize();
            if !length.is_finite()
                || length < 0.001
                || f64::from(fusion.guard_length + length)
                    >= normal.dot(v(c.center) - center) - epsilon
            {
                return Err(invalid(
                    "Organic collar transition escapes its support domain",
                ));
            }
            for (i, other) in collars.iter().enumerate() {
                if i == j {
                    continue;
                }
                let other_normal = v(other.normal).normalize();
                let cosine = normal.dot(other_normal).clamp(-1., 1.);
                let radial_extent = f64::from(other.radius) * (1. - cosine * cosine).sqrt();
                let axial_min = (cosine * f64::from(fusion.guard_length)).min(0.);
                let clearance =
                    normal.dot(v(c.center) - v(other.center)) - radial_extent + axial_min;
                if clearance <= f64::from(fusion.guard_length + length) + epsilon * 8. {
                    return Err(invalid(
                        "Organic collar transition overlaps another fixed guard",
                    ));
                }
            }
        }
    }
    let mut inner = collars.clone();
    for c in &mut inner {
        let normal = v(c.normal).normalize();
        let depth = normal.dot(v(c.center) - center);
        if f64::from(fusion.center_radius + fusion.blend_radius + fusion.guard_length) >= depth
            || f64::from(fusion.guard_length) >= depth * 0.25
        {
            return Err(invalid(
                "Organic fusion center or guard escapes collar support domain",
            ));
        }
        c.center = (v(c.center) - normal * f64::from(fusion.guard_length))
            .to_array()
            .map(|p| p as f32);
    }
    let core = junction(&inner, options)?;
    let n = outer.positions.len();
    let fixed = n * 2;
    let mut geometry = outer;
    geometry
        .positions
        .extend_from_slice(&core.geometry.positions);
    geometry.triangles.extend(
        core.geometry
            .triangles
            .iter()
            .map(|t| t.map(|i| i + n as u32)),
    );
    for b in &boundaries {
        for j in 0..b.vertices.len() {
            let a = b.vertices[j];
            let z = b.vertices[(j + 1) % b.vertices.len()];
            geometry
                .triangles
                .extend([[a + n as u32, z + n as u32, z], [a + n as u32, z, a]]);
        }
    }
    topology(&geometry, &boundaries)?;
    let initial_volume = volume(&geometry, &boundaries);
    if !initial_volume.is_finite() || initial_volume <= epsilon.powi(3) {
        return Err(invalid("Organic fusion initial volume is unresolved"));
    }
    let baseline = geometry.positions.clone();
    let mut order = [0, 1, 2];
    order.sort_by(|&a, &b| collars[a].id.cmp(&collars[b].id));
    let field = |p| fusion_field(p, collars, fusion, order);
    let mut max_residual: f64 = 0.;
    let mut max_displacement: f64 = 0.;
    for (i, &p) in baseline.iter().enumerate().skip(fixed) {
        let original = v(p);
        let ray = original - center;
        let distance = ray.length();
        if distance <= epsilon {
            return Err(invalid(
                "Organic fusion vertex coincides with its radial origin",
            ));
        }
        let ray = ray / distance;
        let mut high = distance;
        if field(original) < 0. {
            high += f64::from(fusion.max_displacement);
            for c in collars {
                let normal = v(c.normal).normalize();
                let denominator = normal.dot(ray);
                if denominator > 1e-12 {
                    high =
                        high.min((normal.dot(v(c.center) - center) - epsilon * 8.) / denominator);
                }
            }
        }
        if high <= 0. || field(center + ray * high) < 0. {
            return Err(invalid(
                "Organic fusion zero lies outside its radial displacement or collar domain",
            ));
        }
        let mut low = 0.;
        for _ in 0..40 {
            let mid = (low + high) * 0.5;
            if field(center + ray * mid) > 0. {
                high = mid;
            } else {
                low = mid;
            }
        }
        let next = center + ray * ((low + high) * 0.5);
        let movement = next.distance(original);
        max_displacement = max_displacement.max(movement);
        if movement > f64::from(fusion.max_displacement) {
            return Err(invalid("Organic fusion displacement exceeds envelope"));
        }
        geometry.positions[i] = next.to_array().map(|p| p as f32);
        let residual = field(v(geometry.positions[i])).abs();
        max_residual = max_residual.max(residual);
        if !residual.is_finite() || residual > f64::from(fusion.tolerance) {
            return Err(invalid(
                "Organic fusion field tolerance is unresolved after quantization",
            ));
        }
    }
    let mut max_rotation: f64 = 0.;
    for &tri in &geometry.triangles {
        let [a, b, c] = tri.map(|i| v(baseline[i as usize]));
        let before = (b - a).cross(c - a);
        let [x, y, z] = tri.map(|i| v(geometry.positions[i as usize]));
        let after = (y - x).cross(z - x);
        if before.dot((a + b + c) / 3. - center) <= epsilon.powi(3)
            || after.dot((x + y + z) / 3. - center) <= epsilon.powi(3)
        {
            return Err(invalid(
                "Organic fusion requires radial orientation without folded faces",
            ));
        }
        let cosine = before.normalize().dot(after.normalize()).clamp(-1., 1.);
        let angle = cosine.acos().to_degrees();
        max_rotation = max_rotation.max(angle);
        if cosine <= 0. || angle > f64::from(fusion.max_face_rotation_degrees) {
            return Err(invalid("Organic fusion face rotation exceeds envelope"));
        }
    }
    let t = topology(&geometry, &boundaries)?;
    let final_volume = volume(&geometry, &boundaries);
    let loss = (initial_volume - final_volume) / initial_volume;
    if !final_volume.is_finite()
        || final_volume <= 0.
        || loss < -1e-5
        || loss > f64::from(fusion.max_volume_loss) + 1e-6
    {
        return Err(invalid(format!(
            "Organic fusion volume loss{loss} exceeds envelope{}",
            fusion.max_volume_loss
        )));
    }
    Ok(Junction {
        report: Report {
            vertices: geometry.positions.len(),
            triangles: geometry.triangles.len(),
            boundary_edges: t.boundary_edges,
            euler_characteristic: -1,
            initial_volume,
            final_volume,
            volume_loss: loss,
            max_interior_movement: max_displacement,
            field: Some(FieldReport {
                fixed_vertices: fixed,
                max_residual,
                max_face_rotation_degrees: max_rotation,
                transition_lengths: transition.then_some(fusion.transition_lengths),
            }),
        },
        geometry,
        boundaries,
        collar_vertices: n,
    })
}

#[cfg(test)]
mod field_tests {
    use super::*;

    #[test]
    fn transition_field_has_radial_normal_at_guard_and_recovers_original_field() {
        let s = 0.5f32.sqrt();
        let normals = [[0., -1., 0.], [-s, s, 0.], [s, s, 0.]];
        let collars = std::array::from_fn(|i| Collar {
            id: format!("c{i}"),
            center: normals[i].map(|x| 1.2 * x),
            normal: normals[i],
            radial: [0., 0., 1.],
            radius: [0.45, 0.35, 0.3][i],
            samples: 8,
        });
        let fusion = Fusion {
            transition_lengths: [0.4; 3],
            ..Default::default()
        };
        let field = |p| fusion_field(p, &collars, fusion, [0, 1, 2]);
        for c in &collars {
            let n = v(c.normal).normalize();
            let r = v(c.radial);
            for angle in [0., 0.3, 1.7, 3.2, 5.4] {
                let radial = r * f64::cos(angle) + n.cross(r) * f64::sin(angle);
                let p =
                    v(c.center) - n * f64::from(fusion.guard_length) + radial * f64::from(c.radius);
                assert!(field(p).abs() < 1e-12);
                let step = 1e-5;
                let axial_derivative = (field(p + n * step) - field(p - n * step)) / (2. * step);
                let radial_derivative =
                    (field(p + radial * step) - field(p - radial * step)) / (2. * step);
                assert!(axial_derivative.abs() < 1e-7, "{axial_derivative}");
                assert!((radial_derivative - 1.).abs() < 1e-8);
            }
        }
        // All transitions are inactive well inside their inner support planes.
        for p in [DVec3::ZERO, DVec3::new(0., -0.3, 0.2)] {
            let legacy = fusion_field(p, &collars, Fusion::default(), [0, 1, 2]);
            assert_eq!(field(p), legacy);
        }
    }
}
