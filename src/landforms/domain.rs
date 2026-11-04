//! Compile-time placement of the same local relief in a plane or radial chart.
use super::*;
use crate::world_space::{CELL_METERS, Coordinates};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Domain {
    Plane,
    Sphere {
        #[serde(with = "crate::terrain::position_codec")]
        #[schemars(with = "Position")]
        center: Position,
        radius_meters: f64,
        /// Projected into the tangent plane once; cannot be parallel to radial up.
        forward: [f64; 3],
    },
}
#[derive(Debug, Clone)]
pub(super) struct Frame {
    sphere: Option<Sphere>,
}
#[derive(Debug, Clone)]
struct Sphere {
    radius: f64,
    up: DVec3,
    right: DVec3,
    forward: DVec3,
    datum: f64,
}
impl Frame {
    pub fn new(origin: Position, domain: Option<&Domain>) -> Result<Self> {
        let Some(Domain::Sphere {
            center,
            radius_meters: radius,
            forward,
        }) = domain
        else {
            return Ok(Self { sphere: None });
        };
        if !radius.is_finite() || !(5. ..=5e11).contains(radius) || !finite(forward, 10000.) {
            return Err(invalid(
                "Landform sphere needs radius5..5e11m and a finite forward",
            ));
        }
        let a: Coordinates = origin.into();
        let b: Coordinates = (*center).into();
        let mut delta = [0.; 3];
        for (i, value) in delta.iter_mut().enumerate() {
            let cells = i128::from(a.cell[i]) - i128::from(b.cell[i]);
            if cells.abs() > (2. * radius / CELL_METERS).ceil() as i128 + 1 {
                return Err(invalid("Landform origin is outside its reference sphere"));
            }
            *value = cells as f64 * CELL_METERS + (a.offset[i] - b.offset[i]);
        }
        let delta = DVec3::from_array(delta);
        let length = delta.length();
        if (length - radius).abs() > (radius * f64::EPSILON * 8.).max(1e-4) {
            return Err(invalid("Landform origin must lie on its reference sphere"));
        }
        let up = delta / length;
        let forward = DVec3::from_array(*forward);
        let forward = (forward - up * forward.dot(up))
            .try_normalize()
            .ok_or_else(|| invalid("Landform sphere forward must be tangent, not radial"))?;
        Ok(Self {
            sphere: Some(Sphere {
                radius: *radius,
                up,
                right: up.cross(forward),
                forward,
                datum: radius - length,
            }),
        })
    }
    pub fn spherical(&self) -> bool {
        self.sphere.is_some()
    }
    pub fn surface_data(&self, p: DVec3, normal: DVec3) -> (f64, f64) {
        let (height, up) = if let Some(s) = &self.sphere {
            let r0 = s.radius - s.datum;
            let radial = s.up * r0 + p;
            let height = (2. * r0 * s.up.dot(p) + p.length_squared() - s.datum * (r0 + s.radius))
                / (radial.length() + s.radius);
            (height, radial.normalize())
        } else {
            (p.y, DVec3::Y)
        };
        (height, normal.dot(up).clamp(-1., 1.).acos().to_degrees())
    }
    pub fn project(&self, p: DVec3) -> Result<DVec3> {
        let Some(s) = &self.sphere else {
            return Ok(p);
        };
        let rho = p.x.hypot(p.z);
        let theta = rho / s.radius;
        if theta >= std::f64::consts::FRAC_PI_2 || p.y <= -s.radius + 1e-4 {
            return Err(invalid(
                "Landform radial chart must fit a hemisphere above the planet centre",
            ));
        }
        let sinc = if theta == 0. { 1. } else { theta.sin() / theta };
        Ok(
            (s.right * p.x + s.forward * p.z) * (sinc * (1. + p.y / s.radius))
                + s.up
                    * (s.datum + p.y * theta.cos() - 2. * s.radius * (theta * 0.5).sin().powi(2)),
        )
    }
    /// Cofactor(J)*normal; avoids inversion and preserves normal orientation.
    pub fn normal(&self, p: DVec3, n: DVec3) -> DVec3 {
        let Some(s) = &self.sphere else {
            return n;
        };
        let rho = p.x.hypot(p.z);
        let theta = rho / s.radius;
        let (dx, dy, dz) = if rho < 1e-10 {
            (
                s.right * (1. + p.y / s.radius),
                s.up,
                s.forward * (1. + p.y / s.radius),
            )
        } else {
            let q = (s.right * p.x + s.forward * p.z) / rho;
            let sinc = theta.sin() / theta;
            let radial = s.up * theta.cos() + q * theta.sin();
            let change = q * (theta.cos() - sinc) - s.up * theta.sin();
            let gain = 1. + p.y / s.radius;
            (
                (s.right * sinc + change * (p.x / rho)) * gain,
                radial,
                (s.forward * sinc + change * (p.z / rho)) * gain,
            )
        };
        dy.cross(dz) * n.x + dz.cross(dx) * n.y + dx.cross(dy) * n.z
    }
}
