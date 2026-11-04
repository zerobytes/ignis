//! Conservative f64 surface-contact guard around the normalized f32 kernel.
use super::*;

fn inside(p: DVec3, tri: &[DVec3; 3], n: DVec3) -> bool {
    (0..3).all(|i| {
        let e = tri[(i + 1) % 3] - tri[i];
        n.dot(e.cross(p - tri[i])) >= -EPS * e.length()
    })
}
fn near_segment(p: DVec3, a: DVec3, b: DVec3) -> bool {
    let e = b - a;
    let t = (p - a).dot(e) / e.length_squared();
    p.distance(a + e * t.clamp(0., 1.)) <= EPS * 4.
}
pub(super) fn forbidden_intersection(g: &Geometry, a: usize, b: usize) -> bool {
    let fa = g.triangles[a];
    let fb = g.triangles[b];
    let ta = fa.map(|i| v(g.positions[i as usize]));
    let tb = fb.map(|i| v(g.positions[i as usize]));
    let mut shared = [DVec3::ZERO; 3];
    let mut count = 0;
    for i in fa {
        if fb.contains(&i) {
            shared[count] = v(g.positions[i as usize]);
            count += 1;
        }
    }
    if count == 3 {
        return true;
    }
    let allowed = |p: DVec3| match &shared[..count] {
        [a] => p.distance(*a) <= EPS * 4.,
        [a, b] => near_segment(p, *a, *b),
        _ => false,
    };
    let na = (ta[1] - ta[0]).cross(ta[2] - ta[0]).normalize();
    let nb = (tb[1] - tb[0]).cross(tb[2] - tb[0]).normalize();
    let da = tb.map(|p| na.dot(p - ta[0]));
    let db = ta.map(|p| nb.dot(p - tb[0]));
    let separated = |d: [f64; 3]| d.iter().all(|&x| x > EPS) || d.iter().all(|&x| x < -EPS);
    if separated(da) || separated(db) {
        return false;
    }
    // All intersections of non-coplanar triangles occur on an edge of one of them.
    for (t, other, n) in [(ta, tb, nb), (tb, ta, na)] {
        for i in 0..3 {
            let p = t[i];
            let q = t[(i + 1) % 3];
            let dp = n.dot(p - other[0]);
            let dq = n.dot(q - other[0]);
            if dp.abs() <= EPS && inside(p, &other, n) && !allowed(p) {
                return true;
            }
            if dp * dq < 0. {
                let hit = p + (q - p) * (dp / (dp - dq));
                if inside(hit, &other, n) && !allowed(hit) {
                    return true;
                }
            }
        }
    }
    if da.iter().all(|d| d.abs() <= EPS) && db.iter().all(|d| d.abs() <= EPS) {
        // Coplanar triangles can overlap by crossing edges with no contained vertex.
        let drop = na.abs().max_position();
        let project = |p: DVec3| match drop {
            0 => DVec2::new(p.y, p.z),
            1 => DVec2::new(p.x, p.z),
            _ => DVec2::new(p.x, p.y),
        };
        for i in 0..3 {
            let p = project(ta[i]);
            let e = project(ta[(i + 1) % 3]) - p;
            for j in 0..3 {
                let q = project(tb[j]);
                let f = project(tb[(j + 1) % 3]) - q;
                let determinant = e.perp_dot(f);
                if determinant.abs() <= 1e-14 {
                    continue;
                }
                let t = (q - p).perp_dot(f) / determinant;
                let u = (q - p).perp_dot(e) / determinant;
                if (0. ..=1.).contains(&t) && (0. ..=1.).contains(&u) {
                    let hit = ta[i] + (ta[(i + 1) % 3] - ta[i]) * t;
                    if !allowed(hit) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
