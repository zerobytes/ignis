//! Optional noncircular root sections and bounded continuous junction deformation.
use super::bake::{Frame, Pose};
use super::*;
#[derive(Clone, Debug)]
pub struct Section {
    pub form: roots::Buttress,
    pub base: DVec3,
    pub stem_radius: f64,
    pub variation: f64,
}
#[derive(Clone, Debug)]
pub struct Flutes {
    pub form: roots::Buttress,
    pub base: DVec3,
    pub radius: f64,
    pub angles: Vec<f64>,
}
impl Flutes {
    pub fn active(&self, point: DVec3, radius: f64) -> bool {
        point.y - radius < self.base.y + self.radius * (self.form.height + 1.) * 2.
    }
    pub fn warp(&self, point: DVec3) -> DVec3 {
        let d = point - self.base;
        let height = d.y.max(0.);
        if height >= self.radius * (self.form.height + 1.) * 2. {
            return point;
        }
        let fade = (1. - height / (self.radius * (self.form.height + 1.) * 2.))
            .clamp(0., 1.)
            .powi(2);
        let az = d.z.atan2(d.x);
        let lobes = self
            .angles
            .iter()
            .map(|a| (-((2. * ((az - a) * 0.5).sin()) / 0.32).powi(2)).exp())
            .fold(0., f64::max);
        let radial = (d.x * d.x + d.z * d.z) / self.radius.powi(2);
        let gain = 1. + self.form.fluting * fade * lobes / (1. + radial);
        self.base + DVec3::new(d.x * gain, d.y, d.z * gain)
    }
}
pub fn shaped(p: &Pose) -> bool {
    p.section.is_some() || p.wood_surface.is_some()
}
pub fn material_section(p: &Pose, u: f64) -> bool {
    shaped(p)
        || p.flutes
            .as_ref()
            .is_some_and(|f| p.width == 0. && f.active(p.at(u).p, p.radius))
}

fn smooth(x: f64) -> f64 {
    let x = x.clamp(0., 1.);
    x * x * (3. - 2. * x)
}
impl Section {
    pub fn point(&self, frame: Frame, radius: f64, theta: f64) -> DVec3 {
        let up = super::bake::normal(frame.t, DVec3::Y);
        let side = up.cross(frame.t);
        let direction = frame.n * theta.cos() + frame.b() * theta.sin();
        let x = direction.dot(side);
        let y = direction.dot(up);
        let distance = (frame.p - self.base).with_y(0.).length();
        let fade = (-(distance / (self.stem_radius * self.form.reach)).powi(2)).exp();
        let height = self.stem_radius * self.form.height * self.variation * fade;
        let width = self.form.wall_width
            + (self.form.crest_width - self.form.wall_width) * smooth(y.max(0.));
        let horizontal = radius * x * (1. + (width - 1.) * fade);
        let vertical = radius * y * (1. + (self.form.crest_width - 1.) * fade)
            + height * smooth((y + 0.15) / 0.3);
        frame.p + side * horizontal + up * vertical
    }
}
pub fn sides(p: &Pose, recipe: &Recipe) -> usize {
    if let Some(surface) = &p.wood_surface {
        surface.sides as usize
    } else if shaped(p) {
        (recipe.settings.sides as usize).max(12)
    } else {
        recipe.settings.sides as usize
    }
}
pub fn point(p: &Pose, source: &Source, u: f64, theta: f64) -> DVec3 {
    let f = p.at(u);
    let (a, b) = p.radius_at(source, u);
    if let Some(surface) = &p.wood_surface {
        surface.point(f, u, theta, p.surface_scale)
    } else if let Some(section) = &p.section {
        section.point(f, a, theta)
    } else {
        f.p + f.n * (a * theta.cos()) + f.b() * (b * theta.sin())
    }
}
pub fn area_bound(p: &Pose, source: &Source, u: f64) -> (f64, f64) {
    let f = p.at(u);
    let mut area = DVec3::ZERO;
    let mut bound: f64 = 0.;
    let count = p.wood_surface.as_ref().map_or(24, |s| 2 * s.sides as usize);
    let points: Vec<DVec3> = (0..count).map(|i| {
        let q = point(p, source, u, std::f64::consts::TAU * i as f64 / count as f64);
        p.flutes.as_ref().map_or(q, |f| f.warp(q)) - f.p
    }).collect();
    for i in 0..points.len() {
        area += points[i].cross(points[(i + 1) % points.len()]);
        bound = bound.max(points[i].length());
    }
    (area.dot(f.t).abs() * 0.5, bound)
}
pub fn center(p: &Pose, source: &Source, u: f64) -> DVec3 {
    if let Some(surface) = &p.wood_surface {
        let frame = p.at(u);
        let offset = surface.offset(u);
        return frame.p + (frame.n * offset[0] + frame.b() * offset[1]) * p.surface_scale;
    }
    if !shaped(p) {
        return p.at(u).p;
    }
    (0..24)
        .map(|i| point(p, source, u, std::f64::consts::TAU * i as f64 / 24.))
        .sum::<DVec3>()
        / 24.
}
/// Dirichlet displacement extension: collars exact, connectivity unchanged.
/// Homotopy steps refuse collapsed/reversed local faces; no global CSG claim.
pub fn deform(mesh: &mut crate::mesh_organic::MultiJunction, targets: &[Vec<DVec3>]) -> Result<()> {
    if targets.len() != mesh.boundaries.len() || targets.iter().flatten().any(|p| !p.is_finite()) {
        return Err(invalid(
            "Buttress junction requires finite targets for every collar",
        ));
    }
    let old: Vec<_> = mesh
        .geometry
        .positions
        .iter()
        .map(|p| DVec3::from_array(p.map(f64::from)))
        .collect();
    let mut displacement = vec![DVec3::ZERO; old.len()];
    for (boundary, target) in mesh.boundaries.iter().zip(targets) {
        if boundary.vertices.len() != target.len() {
            return Err(invalid("Root collar sample mismatch"));
        }
        for (&i, p) in boundary.vertices.iter().zip(target) {
            displacement[i as usize] = *p - old[i as usize];
        }
    }
    let mut adjacent = vec![BTreeSet::new(); old.len()];
    for &[a, b, c] in &mesh.geometry.triangles {
        for (a, b) in [(a, b), (b, c), (c, a)] {
            adjacent[a as usize].insert(b as usize);
            adjacent[b as usize].insert(a as usize);
        }
    }
    let mut next = displacement.clone();
    for _ in 0..160 {
        for i in mesh.collar_vertices..old.len() {
            next[i] = adjacent[i].iter().map(|&j| displacement[j]).sum::<DVec3>()
                / adjacent[i].len() as f64;
        }
        std::mem::swap(&mut displacement, &mut next);
    }
    for step in 1..=32 {
        let a = (step - 1) as f64 / 32.;
        let b = step as f64 / 32.;
        for tri in &mesh.geometry.triangles {
            let [p, q, r] = tri.map(|i| old[i as usize] + displacement[i as usize] * a);
            let [x, y, z] = tri.map(|i| old[i as usize] + displacement[i as usize] * b);
            let n = (q - p).cross(r - p);
            let m = (y - x).cross(z - x);
            if !m.is_finite() || m.length_squared() < 1e-24 || n.dot(m) <= 0. {
                return Err(invalid(
                    "Buttress junction deformation collapses or reverses a local face",
                ));
            }
        }
    }
    for (i, p) in mesh.geometry.positions.iter_mut().enumerate() {
        *p = (old[i] + displacement[i]).as_vec3().to_array();
    }
    Ok(())
}
