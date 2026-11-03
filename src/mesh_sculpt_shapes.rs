//! Frozen-surface shaping brushes. Never sample the result of an earlier stamp.
use crate::{
    meshes::{BrushKind, Geometry, Stroke},
    model::{Result, invalid},
};
use glam::{DVec3, Vec3};

fn weight(point: Vec3, center: [f32; 3], radius: f32) -> f64 {
    let d = point.distance(Vec3::from(center)) / radius;
    if d < 1. {
        (1. - d * d).powi(2) as f64
    } else {
        0.
    }
}
/// Critical authoring guard, evaluated before publishing a sculpt candidate.
/// This conservatively refuses orientation reversal relative to the source face.
pub(crate) fn check_orientation(
    triangles: &[[u32; 3]],
    before: &[[f32; 3]],
    after: &[[f32; 3]],
) -> Result<()> {
    for face in triangles {
        let [a, b, c] = face.map(|i| Vec3::from(before[i as usize]));
        let [p, q, r] = face.map(|i| Vec3::from(after[i as usize]));
        if (b - a).cross(c - a).dot((q - p).cross(r - p)) <= 0. {
            return Err(invalid(
                "Sculpt would collapse or reverse a face; reduce displacement/strength or subdivide",
            ));
        }
    }
    Ok(())
}
/// Draw displaces each dab along its area normal. Independent vertex normals
/// behave like Inflate and can split an already curved patch into sharp ridges.
fn draw_positions(
    geometry: &Geometry,
    stroke: &Stroke,
    relative_height: Option<f32>,
) -> Vec<[f32; 3]> {
    let mut normals = vec![Vec3::ZERO; geometry.positions.len()];
    for face in &geometry.triangles {
        let [a, b, c] = face.map(|i| Vec3::from(geometry.positions[i as usize]));
        let normal = (b - a).cross(c - a);
        for &i in face {
            normals[i as usize] += normal;
        }
    }
    for normal in &mut normals {
        *normal = normal.normalize_or_zero();
    }
    let mut directions = [DVec3::ZERO; 128];
    for (sample, &center) in stroke.centers.iter().enumerate() {
        let mut normal = DVec3::ZERO;
        for (p, n) in geometry.positions.iter().zip(&normals) {
            normal += n.as_dvec3() * weight(Vec3::from(*p), center, stroke.radius);
        }
        directions[sample] = normal.normalize_or_zero();
    }
    let mut positions = geometry.positions.clone();
    for (p, result) in geometry.positions.iter().zip(&mut positions) {
        let point = Vec3::from(*p);
        let mut delta = DVec3::ZERO;
        let mut envelope = 0f64;
        for (sample, &center) in stroke.centers.iter().enumerate() {
            let w = weight(point, center, stroke.radius);
            delta += directions[sample] * w;
            envelope = envelope.max(w);
        }
        *result = if let Some(height) = relative_height {
            (point
                + (delta.normalize_or_zero() * envelope * f64::from(height * stroke.strength))
                    .as_vec3())
            .to_array()
        } else {
            (point + (delta * f64::from(stroke.strength)).as_vec3()).to_array()
        };
    }
    positions
}
pub(crate) fn draw(geometry: &mut Geometry, stroke: &Stroke) -> Result<()> {
    let positions = draw_positions(geometry, stroke, None);
    check_orientation(&geometry.triangles, &geometry.positions, &positions)?;
    let previous = std::mem::replace(&mut geometry.positions, positions);
    if let Err(error) = geometry.validate() {
        geometry.positions = previous;
        return Err(error);
    }
    Ok(())
}

pub(crate) fn apply(geometry: &mut Geometry, stroke: &Stroke) -> Result<()> {
    let n = geometry.positions.len();
    let mut normals = vec![Vec3::ZERO; n];
    let mut neighbors = vec![
        Vec3::ZERO;
        if stroke.kind == BrushKind::Relax {
            n
        } else {
            0
        }
    ];
    let mut counts = vec![0u32; neighbors.len()];
    for &[a, b, c] in &geometry.triangles {
        let ids = [a as usize, b as usize, c as usize];
        let [a, b, c] = ids.map(|i| Vec3::from(geometry.positions[i]));
        let normal = (b - a).cross(c - a);
        for i in ids {
            normals[i] += normal;
        }
        if stroke.kind == BrushKind::Relax {
            for (i, sum) in [(ids[0], b + c), (ids[1], a + c), (ids[2], a + b)] {
                neighbors[i] += sum;
                counts[i] += 2;
            }
        }
    }
    for normal in &mut normals {
        *normal = normal.normalize_or_zero();
    }
    // Weighted surface plane for each stamp, built from the original positions/normals.
    let mut planes = [(Vec3::ZERO, Vec3::ZERO); 128];
    for (sample, &center) in stroke.centers.iter().enumerate() {
        let mut total = 0.;
        let mut position = DVec3::ZERO;
        let mut normal = DVec3::ZERO;
        for (i, p) in geometry.positions.iter().enumerate() {
            let p = Vec3::from(*p);
            let w = weight(p, center, stroke.radius);
            total += w;
            position += p.as_dvec3() * w;
            normal += normals[i].as_dvec3() * w;
        }
        if total > 0. {
            planes[sample] = (
                (position / total).as_vec3(),
                normal.normalize_or_zero().as_vec3(),
            );
        }
    }
    let mut positions = geometry.positions.clone();
    for (i, p) in geometry.positions.iter().enumerate() {
        let point = Vec3::from(*p);
        let mut total = 0.;
        let mut change = DVec3::ZERO;
        for (sample, &center) in stroke.centers.iter().enumerate() {
            let w = weight(point, center, stroke.radius);
            if w == 0. {
                continue;
            }
            let (origin, normal) = planes[sample];
            if normal == Vec3::ZERO {
                continue;
            }
            let height = (point - origin).dot(normal);
            let towards = Vec3::from(center) - point;
            let tangent = towards - normal * towards.dot(normal);
            let sign = stroke.strength.signum();
            let delta = match stroke.kind {
                BrushKind::Flatten => -normal * height,
                BrushKind::Pinch => tangent * sign,
                BrushKind::Crease => {
                    tangent * 0.35 + normal * (-height - stroke.radius * 0.12 * sign)
                }
                BrushKind::Scrape => {
                    -normal
                        * if sign >= 0. {
                            height.max(0.)
                        } else {
                            height.min(0.)
                        }
                }
                BrushKind::Fill => {
                    -normal
                        * if sign >= 0. {
                            height.min(0.)
                        } else {
                            height.max(0.)
                        }
                }
                BrushKind::Relax if counts[i] > 0 => {
                    let v = neighbors[i] / counts[i] as f32 - point;
                    v - normals[i] * v.dot(normals[i])
                }
                BrushKind::Relax => Vec3::ZERO,
                _ => return Err(invalid("Expected a shaping brush")),
            };
            total += w;
            change += delta.as_dvec3() * w;
        }
        if total > 0. {
            let alpha = (1. - (-(stroke.strength.abs() as f64) * total).exp()).min(0.95);
            positions[i] = (point + (change / total * alpha).as_vec3()).to_array();
        }
    }
    // The operation is a deformation, not a topological edit. Refuse folds/collapse
    // relative to the frozen surface instead of silently creating inside-out faces.
    for &[a, b, c] in &geometry.triangles {
        let [a, b, c] = [a as usize, b as usize, c as usize];
        let old = (Vec3::from(geometry.positions[b]) - Vec3::from(geometry.positions[a]))
            .cross(Vec3::from(geometry.positions[c]) - Vec3::from(geometry.positions[a]));
        let new = (Vec3::from(positions[b]) - Vec3::from(positions[a]))
            .cross(Vec3::from(positions[c]) - Vec3::from(positions[a]));
        if new.dot(old) <= 0. {
            return Err(invalid(
                "Sculpt shaping would collapse or reverse a face; reduce strength/radius",
            ));
        }
    }
    let previous = std::mem::replace(&mut geometry.positions, positions);
    if let Err(error) = geometry.validate() {
        geometry.positions = previous;
        return Err(error);
    }
    Ok(())
}

/// Conforming detail in the frozen brush footprint, followed by one Draw.
/// Caller checks aggregate storage/draw budgets before each geometry/rig allocation.
pub(crate) fn detailed(
    source: &crate::meshes::Asset,
    stroke: &Stroke,
    max_edge: f32,
    mut check: impl FnMut(usize, usize) -> Result<()>,
) -> Result<crate::meshes::Asset> {
    stroke.validate()?;
    if stroke.kind != BrushKind::Draw
        || !max_edge.is_finite()
        || !(0.00001..=stroke.radius).contains(&max_edge)
    {
        return Err(invalid(
            "Detailed sculpt requires Draw and max_edge0.00001..radius",
        ));
    }
    // A batch may have staged a PutAsset that has not reached scene validation.
    // Validate its indices and channels before footprint queries or subdivision.
    crate::meshes::Document {
        objects: Default::default(),
        assets: vec![source.clone()],
        instances: vec![],
    }
    .validate()?;
    let mut result = source.clone();
    check(
        result.geometry.positions.len(),
        result.geometry.triangles.len(),
    )?;
    for level in 0..=4 {
        let edges = detail_edges(&result.geometry, stroke, max_edge);
        if edges.is_empty() {
            let geometry = std::sync::Arc::make_mut(&mut result.geometry);
            if geometry.material_coordinates.is_none() {
                geometry.material_coordinates = Some(geometry.positions.clone());
            }
            geometry.sculpt(stroke)?;
            return Ok(result);
        }
        if level == 4 {
            return Err(invalid(
                "Sculpt detail cannot reach max_edge within4 passes; increase brush radius",
            ));
        }
        let components = crate::mesh_topology::Components::Edges { vertices: edges };
        let plan = crate::mesh_topology::Plan::new(&result.geometry, Some(&components))?;
        check(plan.vertex_count(), plan.triangles().len())?;
        result = plan.asset(&result)?;
    }
    unreachable!()
}
fn detail_edges(geometry: &Geometry, stroke: &Stroke, max_edge: f32) -> Vec<[u32; 2]> {
    use rapier3d::{
        na::Point3,
        parry::{query::PointQuery, shape::Triangle},
    };
    use std::collections::BTreeSet;
    let radius = stroke.radius + max_edge * 2.;
    let r2 = radius * radius;
    let centers: Vec<_> = stroke.centers.iter().copied().map(Vec3::from).collect();
    let lo = centers
        .iter()
        .copied()
        .fold(Vec3::splat(f32::INFINITY), Vec3::min)
        - Vec3::splat(radius);
    let hi = centers
        .iter()
        .copied()
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max)
        + Vec3::splat(radius);
    let mut marked = BTreeSet::new();
    for face in &geometry.triangles {
        let [a, b, c] = face.map(|i| Vec3::from(geometry.positions[i as usize]));
        let min = a.min(b).min(c);
        let max = a.max(b).max(c);
        if min.cmpgt(hi).any() || max.cmplt(lo).any() {
            continue;
        }
        let long: [bool; 3] = [
            (a - b).length_squared(),
            (b - c).length_squared(),
            (c - a).length_squared(),
        ]
        .map(|len| len > max_edge * max_edge * 1.0001);
        if !long.iter().any(|x| *x) {
            continue;
        }
        let triangle = Triangle::new(
            Point3::from(a.to_array()),
            Point3::from(b.to_array()),
            Point3::from(c.to_array()),
        );
        if !centers.iter().any(|p| {
            p.distance_squared(p.clamp(min, max)) <= r2
                && triangle.distance_to_local_point(&Point3::from(p.to_array()), false) <= radius
        }) {
            continue;
        }
        for i in 0..3 {
            if long[i] {
                let (a, b) = (face[i], face[(i + 1) % 3]);
                marked.insert(if a < b { [a, b] } else { [b, a] });
            }
        }
    }
    marked.into_iter().collect()
}

/// Interactive sculpting keeps detail and displacement within available budgets.
/// Strict Sculpt/SculptDetailed retain their original contracts.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Feedback {
    pub detail_limited: bool,
    pub fraction: f32,
    pub effective_radius: f32,
}
pub(crate) fn interactive(
    source: &crate::meshes::Asset,
    stroke: &Stroke,
    adaptive_detail: bool,
    detail: crate::mesh_tools::SculptDetail,
    mut check: impl FnMut(usize, usize) -> Result<()>,
) -> Result<(crate::meshes::Asset, Feedback)> {
    stroke.validate()?;
    detail.validate()?;
    if !matches!(stroke.kind, BrushKind::Draw | BrushKind::Grab)
        || (stroke.kind == BrushKind::Grab && stroke.centers.len() != 1)
    {
        return Err(invalid(
            "Interactive sculpt requires Draw or single-center Grab",
        ));
    }
    crate::meshes::Document {
        objects: Default::default(),
        assets: vec![source.clone()],
        instances: vec![],
    }
    .validate()?;
    check(
        source.geometry.positions.len(),
        source.geometry.triangles.len(),
    )?;
    let mut result = source.clone();
    let mut feedback = Feedback {
        detail_limited: false,
        fraction: 1.,
        effective_radius: stroke.radius,
    };
    if adaptive_detail && stroke.kind == BrushKind::Draw {
        let target = detail.max_edge(stroke.radius);
        for level in 0..=4 {
            let edges = detail_edges(&result.geometry, stroke, target);
            if edges.is_empty() {
                break;
            }
            if level == 4 {
                feedback.detail_limited = true;
                break;
            }
            // Preflight generated incidence counts before Plan can reject a full
            // pass, so expected budget saturation is not an operation failure.
            let marked: std::collections::BTreeSet<_> = edges.iter().copied().collect();
            let extra: usize = result
                .geometry
                .triangles
                .iter()
                .map(|f| {
                    (0..3)
                        .filter(|&i| {
                            let (a, b) = (f[i], f[(i + 1) % 3]);
                            marked.contains(&if a < b { [a, b] } else { [b, a] })
                        })
                        .count()
                })
                .sum();
            if check(
                result.geometry.positions.len() + edges.len(),
                result.geometry.triangles.len() + extra,
            )
            .is_err()
            {
                feedback.detail_limited = true;
                break;
            }
            let plan = crate::mesh_topology::Plan::new(
                &result.geometry,
                Some(&crate::mesh_topology::Components::Edges { vertices: edges }),
            )?;
            result = plan.asset(&result)?;
        }
    }
    let geometry = std::sync::Arc::make_mut(&mut result.geometry);
    if geometry.material_coordinates.is_none() {
        geometry.material_coordinates = Some(geometry.positions.clone());
    }
    // A brush smaller than the remaining tessellation must not silently miss
    // every vertex. Enlarge only its footprint on the surface, not its height.
    feedback.effective_radius = supported_radius(geometry, stroke);
    let mut adjusted = stroke.clone();
    adjusted.radius = feedback.effective_radius;
    let positions = if stroke.kind == BrushKind::Draw {
        draw_positions(geometry, &adjusted, Some(stroke.radius * 0.5))
    } else {
        let direction = Vec3::from(stroke.direction).normalize_or_zero();
        geometry
            .positions
            .iter()
            .map(|p| {
                (Vec3::from(*p)
                    + direction
                        * (stroke.strength
                            * weight(Vec3::from(*p), stroke.centers[0], adjusted.radius) as f32))
                    .to_array()
            })
            .collect()
    };
    feedback.fraction = publish_limited(geometry, &positions)?;
    Ok((result, feedback))
}
fn supported_radius(geometry: &Geometry, stroke: &Stroke) -> f32 {
    use rapier3d::{
        na::Point3,
        parry::{query::PointQuery, shape::Triangle},
    };
    let mut radius = stroke.radius;
    for &center in &stroke.centers {
        let point = Vec3::from(center);
        let mut nearest = [f32::INFINITY; 3];
        let mut within = 0;
        for &p in &geometry.positions {
            let distance = point.distance_squared(Vec3::from(p));
            if distance < stroke.radius * stroke.radius {
                within += 1;
            }
            if within >= 3 {
                break;
            }
            if distance < nearest[2] {
                nearest[2] = distance;
                nearest.sort_by(f32::total_cmp);
            }
        }
        if within < 3
            && geometry.triangles.iter().any(|face| {
                let [a, b, c] = face.map(|i| Point3::from(geometry.positions[i as usize]));
                Triangle::new(a, b, c).distance_to_local_point(&Point3::from(center), false)
                    <= stroke.radius
            })
        {
            radius = radius.max(nearest[2].sqrt() / 0.8).min(100.);
        }
    }
    radius
}
/// First orientation boundary along linear vertex displacement, including a
/// safety margin. Checking only the endpoint can miss a fold followed by unfold.
fn safe_fraction(geometry: &Geometry, positions: &[[f32; 3]]) -> f32 {
    let mut fraction = 1f64;
    for face in &geometry.triangles {
        let [p, q, r] =
            face.map(|i| DVec3::from_array(geometry.positions[i as usize].map(f64::from)));
        let [u, v, w] = face.map(|i| DVec3::from_array(positions[i as usize].map(f64::from)));
        let e = q - p;
        let f = r - p;
        let de = (v - u) - e;
        let df = (w - u) - f;
        let normal = e.cross(f);
        let norm = normal.length_squared();
        let a = normal.dot(de.cross(df)) / norm;
        let b = normal.dot(de.cross(f) + e.cross(df)) / norm;
        let mut boundary = 1f64;
        if a.abs() < 1e-12 {
            if b < 0. {
                let root = -0.95 / b;
                if root > 0. && root <= 1. {
                    boundary = root * 0.98;
                }
            }
        } else {
            let disc = b * b - 4. * a * 0.95;
            if disc >= 0. {
                let q = -0.5 * (b + disc.sqrt().copysign(b));
                for root in [q / a, 0.95 / q] {
                    if root > 0. && root <= 1. {
                        boundary = boundary.min(root * 0.98);
                    }
                }
            }
        }
        fraction = fraction.min(boundary);
    }
    for (before, after) in geometry.positions.iter().zip(positions) {
        for (&p, &q) in before.iter().zip(after) {
            let d = f64::from(q) - f64::from(p);
            if d != 0. {
                let boundary = (10000f64.copysign(d) - f64::from(p)) / d;
                if boundary < 1. {
                    fraction = fraction.min(boundary.max(0.) * 0.98);
                }
            }
        }
    }
    fraction as f32
}
fn publish_limited(geometry: &mut Geometry, target: &[[f32; 3]]) -> Result<f32> {
    let original = geometry.positions.clone();
    let mut fraction = safe_fraction(geometry, target);
    for _ in 0..8 {
        let candidate: Vec<_> = original
            .iter()
            .zip(target)
            .map(|(a, b)| {
                (Vec3::from(*a) + (Vec3::from(*b) - Vec3::from(*a)) * fraction).to_array()
            })
            .collect();
        if check_orientation(&geometry.triangles, &original, &candidate).is_ok() {
            geometry.positions = candidate;
            if geometry.validate().is_ok() {
                return Ok(fraction);
            }
            geometry.positions = original.clone();
        }
        fraction *= 0.5;
    }
    geometry.positions = original;
    Ok(0.)
}
