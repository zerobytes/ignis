//! Conservative full-footprint coverage. Polygon subtraction preserves holes.
use super::Surface;
use crate::model::{BodyKind, Entity, Scene};
use glam::{DVec2, DVec3, Vec3};
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Support {
    Supported,
    Gap,
    Limit,
}
type Poly = Vec<DVec2>;
fn cross(a: DVec2, b: DVec2) -> f64 {
    a.x * b.y - a.y * b.x
}
fn clip(p: &Poly, a: DVec2, b: DVec2, inside: bool) -> Poly {
    let mut out = Vec::new();
    for j in 0..p.len() {
        let v = p[j];
        let w = p[(j + 1) % p.len()];
        let d = cross(b - a, v - a);
        let e = cross(b - a, w - a);
        let vin = if inside { d >= 0. } else { d <= 0. };
        let win = if inside { e >= 0. } else { e <= 0. };
        if vin {
            out.push(v);
        }
        if vin != win {
            out.push(v + (w - v) * (d / (d - e)));
        }
    }
    out
}
fn area(p: &Poly) -> f64 {
    (0..p.len())
        .map(|j| cross(p[j], p[(j + 1) % p.len()]))
        .sum::<f64>()
        .abs()
        * 0.5
}
fn remove(uncovered: &mut Vec<Poly>, cover: &Poly, work: &mut usize) -> Option<()> {
    let old = std::mem::take(uncovered);
    for mut p in old {
        for j in 0..cover.len() {
            *work += p.len();
            if *work > 65536 || uncovered.len() > 512 {
                return None;
            }
            let outside = clip(&p, cover[j], cover[(j + 1) % cover.len()], false);
            if area(&outside) > 1e-12 {
                uncovered.push(outside);
            }
            p = clip(&p, cover[j], cover[(j + 1) % cover.len()], true);
            if p.len() < 3 {
                break;
            }
        }
    }
    Some(())
}
fn triangle_poly(
    t: [[f32; 3]; 3],
    feet: Vec3,
    half: Vec3,
    min_y: f32,
    max_y: f32,
    min_normal: f32,
) -> Option<Poly> {
    let t = t.map(|p| DVec3::from(Vec3::from(p) - feet));
    let n = (t[1] - t[0]).cross(t[2] - t[0]).normalize_or_zero();
    if n.y < min_normal as f64 {
        return None;
    }
    let mut p = t.map(|v| DVec2::new(v.x, v.z)).to_vec();
    // XZ projection of an upward triangle is clockwise.
    p.reverse();
    for (a, b) in [
        ([-half.x, -half.z], [half.x, -half.z]),
        ([half.x, -half.z], [half.x, half.z]),
        ([half.x, half.z], [-half.x, half.z]),
        ([-half.x, half.z], [-half.x, -half.z]),
    ] {
        p = clip(
            &p,
            DVec2::from(Vec3::new(a[0], a[1], 0.).truncate()),
            DVec2::from(Vec3::new(b[0], b[1], 0.).truncate()),
            true,
        );
    }
    if p.len() < 3 {
        return None;
    }
    let height = |v: DVec2| t[0].y - (n.x * (v.x - t[0].x) + n.z * (v.y - t[0].z)) / n.y;
    if p.iter()
        .any(|&v| height(v) < min_y as f64 || height(v) > max_y as f64)
    {
        return None;
    }
    Some(p)
}
/// Full horizontal footprint, not sampled corner rays. Refuses excess complexity.
pub(crate) fn footprint(
    scene: &Scene,
    surfaces: &[Surface],
    actor: &Entity,
    feet: Vec3,
) -> Support {
    let half =
        (Vec3::from(actor.transform.size) * 0.5 - Vec3::splat(0.001)).max(Vec3::splat(0.0001));
    let slope = actor
        .character_controller
        .map_or(0., |c| c.max_slope_degrees);
    let drop = 2. * half.x.hypot(half.z) * slope.to_radians().tan() + 0.06;
    let offset = actor.character_controller.map_or(0., |c| c.offset);
    let hi = 0.06;
    let lo = -drop - offset;
    let mut uncovered = vec![vec![
        DVec2::new(-half.x as f64, -half.z as f64),
        DVec2::new(half.x as f64, -half.z as f64),
        DVec2::new(half.x as f64, half.z as f64),
        DVec2::new(-half.x as f64, half.z as f64),
    ]];
    let mut work = 0;
    // Preserve composition with existing flat primitive floors, without inventing boxes for meshes.
    for e in &scene.entities {
        if e.body != Some(BodyKind::Fixed) || !e.geometry.is_box() {
            continue;
        }
        let c = Vec3::from(e.transform.position) - feet;
        let h = Vec3::from(e.transform.size) * 0.5;
        if (c.y + h.y).abs() > 0.06 {
            continue;
        }
        let a = c - h;
        let b = c + h;
        let p = vec![
            DVec2::new(a.x as f64, a.z as f64),
            DVec2::new(b.x as f64, a.z as f64),
            DVec2::new(b.x as f64, b.z as f64),
            DVec2::new(a.x as f64, b.z as f64),
        ];
        if remove(&mut uncovered, &p, &mut work).is_none() {
            return Support::Limit;
        }
        if uncovered.is_empty() {
            return Support::Supported;
        }
    }
    let min = feet + Vec3::new(-half.x, lo, -half.z);
    let max = feet + Vec3::new(half.x, hi, half.z);
    let mut count = 0;
    for s in surfaces {
        let Some(triangles) = s.triangles(min, max) else {
            return Support::Limit;
        };
        count += triangles.len();
        if count > 4096 {
            return Support::Limit;
        }
        for t in triangles {
            if let Some(p) = triangle_poly(t, feet, half, lo, hi, slope.to_radians().cos() - 1e-6) {
                if remove(&mut uncovered, &p, &mut work).is_none() {
                    return Support::Limit;
                }
                if uncovered.is_empty() {
                    return Support::Supported;
                }
            }
        }
    }
    Support::Gap
}
/// Highest point under the footprint within the explicit downward search.
/// Full coverage and body clearance are separate mandatory admission gates.
pub(crate) fn project(
    surfaces: &[Surface],
    actor: &Entity,
    origin: Vec3,
    max_drop: f32,
) -> Result<Option<(String, f32)>, ()> {
    let half = Vec3::from(actor.transform.size) * 0.5;
    let mut best = None;
    let mut count = 0;
    for s in surfaces {
        let ts = s
            .triangles(
                origin - Vec3::new(half.x, max_drop, half.z),
                origin + Vec3::new(half.x, 0., half.z),
            )
            .ok_or(())?;
        count += ts.len();
        if count > 4096 {
            return Err(());
        }
        for t in ts {
            let p = t.map(Vec3::from);
            let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
            if n.y
                < actor
                    .character_controller
                    .map_or(1., |c| c.max_slope_degrees.to_radians().cos())
                    - 1e-6
            {
                continue;
            }
            if let Some(poly) = triangle_poly(t, origin, half, -max_drop, 0., n.y - 1e-6) {
                for v in poly {
                    let x = origin.x + v.x as f32;
                    let z = origin.z + v.y as f32;
                    let y = p[0].y - (n.x * (x - p[0].x) + n.z * (z - p[0].z)) / n.y;
                    if best
                        .as_ref()
                        .is_none_or(|(id, old): &(String, f32)| y > *old || y == *old && s.id < *id)
                    {
                        best = Some((s.id.clone(), y));
                    }
                }
            }
        }
    }
    Ok(best)
}
