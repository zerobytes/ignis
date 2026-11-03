//! Version7 basal skin: swept cuffs and screened harmonic valleys.
use super::bake::Pose;
use super::*;
use std::f64::consts::TAU;
fn smooth(x: f64) -> f64 {
    let x = x.clamp(0., 1.);
    x * x * (3. - 2. * x)
}
fn angle(a: f64, b: f64) -> f64 {
    ((a - b + std::f64::consts::PI).rem_euclid(TAU) - std::f64::consts::PI).abs()
}
struct Lobe {
    az: f64,
    reach: f64,
    width: f64,
    crest: f64,
}
struct Guide {
    base: DVec3,
    top: DVec3,
    radius: f64,
    top_radius: f64,
    lobes: Vec<Lobe>,
}
impl Guide {
    fn project(&self, p: DVec3) -> DVec3 {
        let t = smooth((p.y - self.base.y) / (self.top.y - self.base.y).max(1e-12));
        let axis = self.base.lerp(self.top, t);
        let d = (p - axis).with_y(0.);
        let length = d.length();
        if length < 1e-12 {
            return p;
        }
        let az = d.z.atan2(d.x);
        let trunk = self.radius + (self.top_radius - self.radius) * t;
        let extension = self
            .lobes
            .iter()
            .map(|l| {
                let vertical = 1. - smooth((p.y - l.crest) / (self.top.y - l.crest).max(1e-12));
                (l.reach - trunk).max(0.) * (-(angle(az, l.az) / l.width).powi(2)).exp() * vertical
            })
            .fold(0., f64::max);
        let q = axis + d * ((trunk + extension) / length);
        q.with_y(p.y)
    }
}
pub fn refine(
    mesh: &mut crate::mesh_organic::MultiJunction,
    seed: &[[f32; 3]],
    poses: &[Pose],
    ports: &[(usize, f64)],
    recipe: &Recipe,
) -> Result<f64> {
    let (parent, u) = ports[0];
    let stem = &poses[parent];
    let base = stem.samples[0].p;
    let top = stem.at(u).p;
    let mut lobes = vec![];
    for (k, (i, _)) in ports.iter().enumerate().skip(1) {
        let points: Vec<_> = mesh.boundaries[k]
            .vertices
            .iter()
            .map(|&j| DVec3::from_array(mesh.geometry.positions[j as usize].map(f64::from)))
            .collect();
        let center = points.iter().sum::<DVec3>() / points.len() as f64;
        let d = (center - base).with_y(0.);
        let az = d.z.atan2(d.x);
        let width = points
            .iter()
            .map(|p| angle((p.z - base.z).atan2(p.x - base.x), az))
            .fold(0., f64::max)
            * 2.5;
        let crest = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        if crest >= top.y {
            return Err(invalid("Root crest reaches above its stem collar"));
        }
        let _ = i;
        lobes.push(Lobe {
            az,
            reach: d.length(),
            width: width.clamp(0.08, 0.45),
            crest,
        });
    }
    for i in 0..lobes.len() {
        let spacing = lobes
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, l)| angle(l.az, lobes[i].az))
            .fold(TAU, f64::min);
        lobes[i].width = lobes[i].width.min(spacing * 0.38).max(0.04);
    }
    let guide = Guide {
        base,
        top,
        radius: stem.radius,
        top_radius: stem.radius_at(&recipe.source, u).0,
        lobes,
    };
    let old: Vec<_> = mesh
        .geometry
        .positions
        .iter()
        .map(|p| DVec3::from_array(p.map(f64::from)))
        .collect();
    let mut adjacent = vec![BTreeSet::new(); old.len()];
    for &[a, b, c] in &mesh.geometry.triangles {
        for (a, b) in [(a, b), (b, c), (c, a)] {
            adjacent[a as usize].insert(b as usize);
            adjacent[b as usize].insert(a as usize);
        }
    }
    let mut owner = vec![None; old.len()];
    for (k, boundary) in mesh.boundaries.iter().enumerate() {
        for &i in &boundary.vertices {
            owner[i as usize] = Some(k);
        }
    }
    let mut fixed: Vec<_> = (0..old.len())
        .map(|i| (i < mesh.collar_vertices).then_some(old[i]))
        .collect();
    // The first row follows the sweep derivative instead of retaining the
    // convex hull shoulder. Angular coordinates come from the undeformed cage.
    for i in mesh.collar_vertices..old.len() {
        let owners: BTreeSet<_> = adjacent[i].iter().filter_map(|&j| owner[j]).collect();
        if owners.len() != 1 {
            continue;
        }
        let k = *owners.first().unwrap();
        let (organ, u) = ports[k];
        let pose = &poses[organ];
        let frame = pose.at(u);
        let delta = DVec3::from_array(seed[i].map(f64::from)) - frame.p;
        let depth = -delta.dot(frame.t);
        if depth <= 1e-10 {
            continue;
        }
        let theta = delta.dot(frame.b()).atan2(delta.dot(frame.n));
        let epsilon = 1e-4_f64.min(u * 0.25).min((1. - u) * 0.25);
        if epsilon <= 1e-12 {
            return Err(invalid("Insufficient crown cuff interval"));
        }
        let a = super::root_sections::point(pose, &recipe.source, u - epsilon, theta);
        let b = super::root_sections::point(pose, &recipe.source, u + epsilon, theta);
        let derivative = (b - a) / (2. * epsilon);
        let speed = (pose.at(u + epsilon).p - pose.at(u - epsilon).p).dot(frame.t) / (2. * epsilon);
        if speed <= 1e-10 {
            return Err(invalid("Crown cuff requires a forward sweep"));
        }
        let angular = super::root_sections::point(pose, &recipe.source, u, theta + epsilon)
            - super::root_sections::point(pose, &recipe.source, u, theta - epsilon);
        let outward = angular.cross(frame.t).normalize_or_zero();
        let radial = delta - frame.t * delta.dot(frame.t);
        let radius = pose.radius_at(&recipe.source, u).0;
        let cuff_depth = depth.min(radius * 0.2);
        // Preserve the chart's second-order radial separation. Dropping it
        // can map distinct first-row vertices to the same sweep coordinate.
        let second_order =
            (radial.length() - radius) * (cuff_depth / radius).min(0.5) * (cuff_depth / depth);
        fixed[i] = Some(
            super::root_sections::point(pose, &recipe.source, u, theta)
                - derivative * (cuff_depth / speed)
                + outward * second_order,
        );
    }
    let mut best = None;
    for screening in [0.8, 0.4, 0.15] {
        let mut points = old.clone();
        let mut next = old.clone();
        for _ in 0..192 {
            for i in mesh.collar_vertices..points.len() {
                let mean = adjacent[i].iter().map(|&j| points[j]).sum::<DVec3>()
                    / adjacent[i].len() as f64;
                next[i] = if let Some(p) = fixed[i] {
                    mean.lerp(p, 0.5)
                } else {
                    mean.lerp(guide.project(mean), screening)
                };
            }
            std::mem::swap(&mut points, &mut next);
        }
        // Preserve local orientation while selecting a bounded fairing amount.
        for amount in [1., 0.75, 0.5] {
            let mut valid = true;
            'path: for step in 1..=64 {
                let a = amount * (step - 1) as f64 / 64.;
                let b = amount * step as f64 / 64.;
                for tri in &mesh.geometry.triangles {
                    let [p, q, r] = tri.map(|i| old[i as usize].lerp(points[i as usize], a));
                    let [x, y, z] = tri.map(|i| old[i as usize].lerp(points[i as usize], b));
                    let n = (q - p).cross(r - p);
                    let m = (y - x).cross(z - x);
                    if !m.is_finite() || m.length_squared() < 1e-24 || n.dot(m) <= 0. {
                        valid = false;
                        break 'path;
                    }
                }
            }
            if valid {
                if amount >= 1. {
                    for (p, q) in mesh.geometry.positions.iter_mut().zip(&points) {
                        *p = q.as_vec3().to_array();
                    }
                    return Ok(1.);
                }
                if best.as_ref().is_none_or(|(quality, _)| amount > *quality) {
                    best = Some((amount, points.clone()));
                }
                break;
            }
        }
    }
    if let Some((amount, points)) = best {
        for (i, p) in mesh.geometry.positions.iter_mut().enumerate() {
            *p = old[i].lerp(points[i], amount).as_vec3().to_array();
        }
        return Ok(amount);
    }
    Err(invalid("Valley crown collapses or reverses a local face"))
}
