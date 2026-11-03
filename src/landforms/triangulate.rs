//! Bounded simple-polygon validation and concave caps. No implicit convex fan.
use crate::model::{Result, invalid};
use glam::DVec2;

pub(super) struct Profile {
    pub caps: Vec<[usize; 3]>,
    pub star_shaped: bool,
    pub kernel_radius: f64,
}
fn cross(a: DVec2, b: DVec2, c: DVec2) -> f64 {
    (b - a).perp_dot(c - a)
}
fn on_segment(a: DVec2, b: DVec2, p: DVec2, e: f64) -> bool {
    p.x >= a.x.min(b.x) - e
        && p.x <= a.x.max(b.x) + e
        && p.y >= a.y.min(b.y) - e
        && p.y <= a.y.max(b.y) + e
}
fn intersects(a: DVec2, b: DVec2, c: DVec2, d: DVec2, e: f64) -> bool {
    let ab_c = cross(a, b, c);
    let ab_d = cross(a, b, d);
    let cd_a = cross(c, d, a);
    let cd_b = cross(c, d, b);
    if ((ab_c > e && ab_d < -e) || (ab_c < -e && ab_d > e))
        && ((cd_a > e && cd_b < -e) || (cd_a < -e && cd_b > e))
    {
        return true;
    }
    (ab_c.abs() <= e && on_segment(a, b, c, e))
        || (ab_d.abs() <= e && on_segment(a, b, d, e))
        || (cd_a.abs() <= e && on_segment(c, d, a, e))
        || (cd_b.abs() <= e && on_segment(c, d, b, e))
}
pub(super) fn prepare(points: &[[f64; 2]]) -> Result<Profile> {
    if !(3..=96).contains(&points.len())
        || points
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 1000.)
    {
        return Err(invalid(
            "Landform profile requires3..96finite points within1000m",
        ));
    }
    let p: Vec<_> = points.iter().map(|&p| DVec2::from_array(p)).collect();
    let n = p.len();
    let scale = p.iter().map(|p| p.abs().max_element()).fold(1., f64::max);
    let e = scale * scale * 1e-12;
    let mut area = 0.;
    let mut kernel = f64::INFINITY;
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        let edge = b - a;
        if edge.length_squared() < 1e-10 {
            return Err(invalid("Landform profile has a tiny/repeated edge"));
        }
        area += a.perp_dot(b);
        kernel = kernel.min(edge.perp_dot(-a) / edge.length());
        for j in i + 1..n {
            if a.distance_squared(p[j]) < 1e-10 {
                return Err(invalid("Landform profile repeats a vertex"));
            }
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if intersects(a, b, p[j], p[(j + 1) % n], e) {
                return Err(invalid("Landform profile crosses or touches itself"));
            }
        }
    }
    if area <= e * (n as f64) {
        return Err(invalid(
            "Landform profile must be simple CCW with nonzero area",
        ));
    }
    let mut remaining: Vec<_> = (0..n).collect();
    let mut caps = Vec::with_capacity(n - 2);
    while remaining.len() > 3 {
        let mut found = None;
        for i in 0..remaining.len() {
            let a = remaining[(i + remaining.len() - 1) % remaining.len()];
            let b = remaining[i];
            let c = remaining[(i + 1) % remaining.len()];
            if cross(p[a], p[b], p[c]) <= e {
                continue;
            }
            let blocked = remaining.iter().any(|&j| {
                j != a
                    && j != b
                    && j != c
                    && cross(p[a], p[b], p[j]) >= -e
                    && cross(p[b], p[c], p[j]) >= -e
                    && cross(p[c], p[a], p[j]) >= -e
            });
            if !blocked {
                found = Some((i, [a, b, c]));
                break;
            }
        }
        let (i, triangle) = found.ok_or_else(|| {
            invalid(
                "Landform cap could not be triangulated safely; simplify near-degenerate contour",
            )
        })?;
        caps.push(triangle);
        remaining.remove(i);
    }
    if cross(p[remaining[0]], p[remaining[1]], p[remaining[2]]) <= e {
        return Err(invalid("Degenerate final landform cap"));
    }
    caps.push([remaining[0], remaining[1], remaining[2]]);
    Ok(Profile {
        caps,
        star_shaped: kernel > scale * 1e-8,
        kernel_radius: kernel,
    })
}
