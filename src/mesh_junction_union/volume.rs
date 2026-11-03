use super::{Options, Port, Surface};
use crate::model::{Result, invalid};
use glam::DVec3;
use std::collections::{BTreeMap, BTreeSet};
const MAX_GRID_POINTS: usize = 65_536;
const MAX_RAW_VERTICES: usize = 16_384;
const MAX_RAW_TRIANGLES: usize = 32_768;
struct Segment {
    a: DVec3,
    ab: DVec3,
    length_squared: f64,
    radius: f64,
    delta_radius: f64,
}
struct Field {
    arms: Vec<Vec<Segment>>,
    blend: f64,
    epsilon: f64,
}
impl Field {
    fn value(&self, p: DVec3) -> f64 {
        let mut result = f64::INFINITY;
        for arm in &self.arms {
            let mut distance = f64::INFINITY;
            for s in arm {
                let u = ((p - s.a).dot(s.ab) / s.length_squared).clamp(0., 1.);
                let f = (p - s.a - s.ab * u).length() - s.radius - s.delta_radius * u;
                distance = distance.min(f);
            }
            result = if result == f64::INFINITY {
                distance
            } else {
                let h = (0.5 + 0.5 * (distance - result) / self.blend).clamp(0., 1.);
                distance + (result - distance) * h - self.blend * h * (1. - h)
            };
        }
        result
    }
    fn gradient(&self, p: DVec3) -> DVec3 {
        let e = self.epsilon;
        DVec3::new(
            self.value(p + DVec3::X * e) - self.value(p - DVec3::X * e),
            self.value(p + DVec3::Y * e) - self.value(p - DVec3::Y * e),
            self.value(p + DVec3::Z * e) - self.value(p - DVec3::Z * e),
        ) / (2. * e)
    }
}
pub(super) struct Extracted {
    pub surface: Surface,
    pub grid_points: usize,
    pub max_error: f64,
    field: Field,
}
impl Extracted {
    pub fn field_error(&self, points: &[[f32; 3]]) -> f64 {
        points
            .iter()
            .map(|&p| self.field.value(super::v(p)).abs())
            .fold(0., f64::max)
    }
}
fn iso_edge(
    a: usize,
    b: usize,
    values: &[f64],
    positions: &[DVec3],
    cache: &mut BTreeMap<[usize; 2], u32>,
    points: &mut Vec<DVec3>,
) -> Result<u32> {
    let key = if a < b { [a, b] } else { [b, a] };
    if let Some(&i) = cache.get(&key) {
        return Ok(i);
    }
    if points.len() >= MAX_RAW_VERTICES {
        return Err(invalid("Union intermediate vertex budget exceeded"));
    }
    let u = values[a] / (values[a] - values[b]);
    let p = positions[a] + (positions[b] - positions[a]) * u;
    if !p.is_finite() || !(0.0..=1.0).contains(&u) {
        return Err(invalid("Union iso edge is unresolved"));
    }
    let index = points.len() as u32;
    points.push(p);
    cache.insert(key, index);
    Ok(index)
}
pub(super) fn extract(ports: &[Port], options: &Options) -> Result<Extracted> {
    let hub = DVec3::from_array(options.hub.map(f64::from));
    let rh = f64::from(options.hub_radius);
    let mut arms = Vec::new();
    let mut low = DVec3::splat(f64::INFINITY);
    let mut high = DVec3::splat(f64::NEG_INFINITY);
    for p in ports {
        let length = p.centre.distance(hub);
        let controls = [
            hub,
            hub + (p.centre - hub) * 0.4,
            p.centre - p.normal * length * 0.35,
            p.centre,
        ];
        let mut axis = Vec::with_capacity(26);
        let mut radii = Vec::with_capacity(26);
        for i in 0..=24 {
            let t = i as f64 / 24.;
            let u = 1. - t;
            let point = controls[0] * u.powi(3)
                + controls[1] * (3. * u * u * t)
                + controls[2] * (3. * u * t * t)
                + controls[3] * t.powi(3);
            axis.push(point);
            radii.push(rh + (p.radius - rh) * (3. * t * t - 2. * t * t * t));
        }
        axis.push(p.centre + p.normal * p.radius * 1.5);
        radii.push(p.radius);
        for &p in &axis {
            low = low.min(p);
            high = high.max(p);
        }
        let mut segments = Vec::with_capacity(25);
        for i in 0..axis.len() - 1 {
            let ab = axis[i + 1] - axis[i];
            let length_squared = ab.length_squared();
            if length_squared < 1e-16 {
                return Err(invalid("Union guide has an unresolved segment"));
            }
            segments.push(Segment {
                a: axis[i],
                ab,
                length_squared,
                radius: radii[i],
                delta_radius: radii[i + 1] - radii[i],
            });
        }
        arms.push(segments);
    }
    let margin = (rh * 1.8).max(
        ports
            .iter()
            .map(|p| p.radius + f64::from(options.blend))
            .fold(0., f64::max),
    );
    low -= DVec3::splat(margin);
    high += DVec3::splat(margin);
    let h = f64::from(options.cell_size);
    let dims = (high - low) / h;
    let mut ns = [0_usize; 3];
    let mut count = 1_usize;
    for k in 0..3 {
        let n = dims[k].ceil() + 1.;
        if !n.is_finite() || !(2.0..=MAX_GRID_POINTS as f64).contains(&n) {
            return Err(invalid("Union grid extent exceeds budget"));
        }
        ns[k] = n as usize;
        count = count
            .checked_mul(ns[k])
            .ok_or_else(|| invalid("Union grid extent overflows"))?;
    }
    if count > MAX_GRID_POINTS {
        return Err(invalid("Union grid exceeds65536point authoring budget"));
    }
    let field = Field {
        arms,
        blend: f64::from(options.blend),
        epsilon: (h / 7000.).clamp(1e-7, 1e-4),
    };
    let mut positions = Vec::with_capacity(count);
    let mut values = Vec::with_capacity(count);
    for x in 0..ns[0] {
        for y in 0..ns[1] {
            for z in 0..ns[2] {
                let p = low + DVec3::new(x as f64, y as f64, z as f64) * h;
                let value = field.value(p);
                if !value.is_finite() {
                    return Err(invalid("Union field is not finite"));
                }
                if (x == 0
                    || y == 0
                    || z == 0
                    || x + 1 == ns[0]
                    || y + 1 == ns[1]
                    || z + 1 == ns[2])
                    && value <= 0.
                {
                    return Err(invalid("Union surface touches its extraction envelope"));
                }
                positions.push(p);
                values.push(value);
            }
        }
    }
    let mut surface = Surface {
        points: Vec::new(),
        triangles: Vec::new(),
    };
    let mut cache = BTreeMap::new();
    let index = |x: usize, y: usize, z: usize| (x * ns[1] + y) * ns[2] + z;
    const TETS: [[usize; 4]; 6] = [
        [0, 1, 3, 7],
        [0, 3, 2, 7],
        [0, 2, 6, 7],
        [0, 6, 4, 7],
        [0, 4, 5, 7],
        [0, 5, 1, 7],
    ];
    for x in 0..ns[0] - 1 {
        for y in 0..ns[1] - 1 {
            for z in 0..ns[2] - 1 {
                let ids: [usize; 8] = std::array::from_fn(|i| {
                    index(x + (i & 1), y + ((i >> 1) & 1), z + ((i >> 2) & 1))
                });
                if ids.iter().all(|&i| values[i] > 0.) || ids.iter().all(|&i| values[i] < 0.) {
                    continue;
                }
                for tet in TETS {
                    let mut inside = [0_usize; 4];
                    let mut outside = [0_usize; 4];
                    let (mut inside_count, mut outside_count) = (0, 0);
                    for i in tet {
                        if values[ids[i]] < 0. {
                            inside[inside_count] = ids[i];
                            inside_count += 1;
                        } else {
                            outside[outside_count] = ids[i];
                            outside_count += 1;
                        }
                    }
                    let inside = &inside[..inside_count];
                    let outside = &outside[..outside_count];
                    if inside.is_empty() || outside.is_empty() {
                        continue;
                    }
                    let direction = outside.iter().map(|&i| positions[i]).sum::<DVec3>()
                        / outside.len() as f64
                        - inside.iter().map(|&i| positions[i]).sum::<DVec3>() / inside.len() as f64;
                    let mut edge =
                        |a, b| iso_edge(a, b, &values, &positions, &mut cache, &mut surface.points);
                    let (faces, n) = if inside.len() == 2 {
                        let (a, b, c, d) = (inside[0], inside[1], outside[0], outside[1]);
                        let p = [edge(a, c)?, edge(a, d)?, edge(b, d)?, edge(b, c)?];
                        ([[p[0], p[1], p[2]], [p[0], p[2], p[3]]], 2)
                    } else {
                        let (a, others) = if inside.len() == 1 {
                            (inside[0], outside)
                        } else {
                            (outside[0], inside)
                        };
                        let p = [
                            edge(a, others[0])?,
                            edge(a, others[1])?,
                            edge(a, others[2])?,
                        ];
                        ([p, [0; 3]], 1)
                    };
                    for mut t in faces.into_iter().take(n) {
                        if surface.triangles.len() >= MAX_RAW_TRIANGLES {
                            return Err(invalid("Union intermediate triangle budget exceeded"));
                        }
                        let [a, b, c] = t.map(|i| surface.points[i as usize]);
                        if (b - a).cross(c - a).dot(direction) < 0. {
                            t.swap(1, 2);
                        }
                        surface.triangles.push(t);
                    }
                }
            }
        }
    }
    if surface.triangles.is_empty() {
        return Err(invalid("Union field has no resolved surface"));
    }
    let mut adjacent = vec![BTreeSet::new(); surface.points.len()];
    for &[a, b, c] in &surface.triangles {
        adjacent[a as usize].extend([b, c]);
        adjacent[b as usize].extend([a, c]);
        adjacent[c as usize].extend([a, b]);
    }
    for _ in 0..options.relaxation_steps {
        let mut next = Vec::with_capacity(surface.points.len());
        for (i, neighbors) in adjacent.iter().enumerate() {
            if neighbors.is_empty() {
                return Err(invalid("Union iso surface has an isolated point"));
            }
            let mut point = surface.points[i] * 0.5
                + neighbors
                    .iter()
                    .map(|&j| surface.points[j as usize])
                    .sum::<DVec3>()
                    * (0.5 / neighbors.len() as f64);
            for _ in 0..4 {
                let f = field.value(point);
                let gradient = field.gradient(point);
                let norm = gradient.length_squared();
                if !norm.is_finite() || norm < 1e-12 {
                    return Err(invalid("Union projection gradient is unresolved"));
                }
                let correction = gradient * (f / norm);
                if !correction.is_finite() || correction.length() > h {
                    return Err(invalid(
                        "Union projection exceeds its local displacement envelope",
                    ));
                }
                point -= correction;
            }
            next.push(point);
        }
        surface.points = next;
    }
    let max_error = surface
        .points
        .iter()
        .map(|&p| field.value(super::v(p.to_array().map(|x| x as f32))).abs())
        .fold(0., f64::max);
    if !max_error.is_finite() || max_error > h * 0.05 {
        return Err(invalid(
            "Union projected surface exceeds field error tolerance",
        ));
    }
    for t in &surface.triangles {
        let [a, b, c] =
            t.map(|i| super::v(surface.points[i as usize].to_array().map(|x| x as f32)));
        let normal = (b - a).cross(c - a);
        if !normal.is_finite()
            || normal.length_squared() < 1e-24
            || normal.dot(field.gradient((a + b + c) / 3.)) <= 0.
        {
            return Err(invalid("Union extracted face reverses its field gradient"));
        }
    }
    let keep: Vec<_> = surface
        .points
        .iter()
        .map(|p| {
            ports.iter().all(|port| {
                let centre = port.centre - port.normal * f64::from(options.collar_length)
                    + port.normal * port.radius * 1.5;
                p.distance(centre) > port.radius * 3.25_f64.sqrt()
            })
        })
        .collect();
    surface
        .triangles
        .retain(|t| t.iter().all(|&i| keep[i as usize]));
    let mut used = vec![false; surface.points.len()];
    for t in &surface.triangles {
        for &i in t {
            used[i as usize] = true;
        }
    }
    let mut map = vec![0_u32; used.len()];
    let mut points = Vec::new();
    for (i, &yes) in used.iter().enumerate() {
        if yes {
            map[i] = points.len() as u32;
            points.push(surface.points[i]);
        }
    }
    surface.points = points;
    for t in &mut surface.triangles {
        *t = t.map(|i| map[i as usize]);
    }
    if surface.points.is_empty() {
        return Err(invalid("Union trimming removed the entire surface"));
    }
    let mut neighbors = vec![Vec::new(); surface.points.len()];
    for &[a, b, c] in &surface.triangles {
        neighbors[a as usize].extend([b, c]);
        neighbors[b as usize].extend([a, c]);
        neighbors[c as usize].extend([a, b]);
    }
    let mut visited = vec![false; neighbors.len()];
    let mut pending = vec![0_usize];
    visited[0] = true;
    while let Some(i) = pending.pop() {
        for &j in &neighbors[i] {
            if !visited[j as usize] {
                visited[j as usize] = true;
                pending.push(j as usize);
            }
        }
    }
    if visited.iter().any(|v| !*v) {
        return Err(invalid("Union trimming produced disconnected components"));
    }
    Ok(Extracted {
        surface,
        grid_points: count,
        max_error,
        field,
    })
}
