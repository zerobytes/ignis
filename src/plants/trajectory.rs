//! Certified monotone cubic centreline and explicit normal-section frame.
use crate::{
    mesh_preservation::MAPPING_TOLERANCE,
    model::{Result, invalid},
};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Trajectory {
    CubicBezier {
        id: String,
        /// Local metres X/Z; longitudinal Y is linear over profile.height.
        control_points_xz: [[f32; 2]; 4],
        #[serde(default = "default_surface_tolerance")]
        surface_tolerance: f32,
    },
}
fn default_surface_tolerance() -> f32 {
    0.0002
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub inverse_derivative_margin: f64,
    pub sampled_surface_error: f64,
    pub surface_tolerance: f32,
    pub surface_probes: usize,
}

pub(super) struct Compiled {
    coefficients: [[f64; 4]; 2],
    height: f64,
    pub(super) tolerance: f32,
    ranges: [f64; 2],
    accelerations: [f64; 2],
}
pub(super) struct Frame {
    pub(super) center: DVec3,
    pub(super) tangent: DVec3,
    pub(super) normal: DVec3,
    pub(super) binormal: DVec3,
}
impl Frame {
    pub(super) fn position(&self, x: f64, z: f64) -> [f64; 3] {
        (self.center + self.normal * x + self.binormal * z).to_array()
    }
}
impl Trajectory {
    pub(super) fn id(&self) -> &str {
        match self {
            Self::CubicBezier { id, .. } => id,
        }
    }
    pub(super) fn compile(&self, height: f64) -> Compiled {
        let Self::CubicBezier {
            control_points_xz: p,
            surface_tolerance,
            ..
        } = self;
        let coefficients = std::array::from_fn(|k| {
            let [a, b, c, d] = p.map(|p| f64::from(p[k]));
            [
                -a + 3. * b - 3. * c + d,
                3. * a - 6. * b + 3. * c,
                3. * (b - a),
                a,
            ]
        });
        let ranges = std::array::from_fn(|k| {
            let p = p.map(|p| f64::from(p[k]));
            p.into_iter().fold(f64::NEG_INFINITY, f64::max)
                - p.into_iter().fold(f64::INFINITY, f64::min)
        });
        let accelerations =
            coefficients.map(|[a, b, _, _]| (2. * b).abs().max((6. * a + 2. * b).abs()));
        Compiled {
            coefficients,
            height,
            tolerance: *surface_tolerance,
            ranges,
            accelerations,
        }
    }
    pub(super) fn validate(&self, height: f64, radius: f64) -> Result<()> {
        let Self::CubicBezier {
            id,
            control_points_xz,
            surface_tolerance,
        } = self;
        if !super::valid_id(id)
            || control_points_xz
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 100.)
            || !surface_tolerance.is_finite()
            || !(0.000001..=1.).contains(surface_tolerance)
        {
            return Err(invalid(
                "Trajectory requires a stable ID, finite controls within100m and surface tolerance0.000001..1m",
            ));
        }
        let c = self.compile(height);
        if c.margin(radius) <= height * height * 1e-6 {
            return Err(invalid(
                "Trajectory/section volume cannot certify a unique inverse; reduce bend or thickness, or increase height",
            ));
        }
        Ok(())
    }
}
impl Compiled {
    pub(super) fn margin(&self, radius: f64) -> f64 {
        let radius = radius * (1. + 4. * MAPPING_TOLERANCE);
        self.height * self.height
            - (self.ranges[0] + radius) * self.accelerations[0]
            - (self.ranges[1] + radius) * self.accelerations[1]
    }
    fn curve(&self, t: f64) -> (DVec3, DVec3) {
        let xz = self
            .coefficients
            .map(|[a, b, c, d]| ((a * t + b) * t + c) * t + d);
        let derivative = self
            .coefficients
            .map(|[a, b, c, _]| (3. * a * t + 2. * b) * t + c);
        (
            DVec3::new(xz[0], self.height * (t - 0.5), xz[1]),
            DVec3::new(derivative[0], self.height, derivative[1]),
        )
    }
    pub(super) fn frame(&self, t: f64) -> Frame {
        let (center, derivative) = self.curve(t);
        let tangent = derivative.normalize();
        let q = 1. + tangent.y;
        let normal = DVec3::new(
            1. - tangent.x * tangent.x / q,
            -tangent.x,
            -tangent.x * tangent.z / q,
        );
        let binormal = DVec3::new(
            -tangent.x * tangent.z / q,
            -tangent.z,
            1. - tangent.z * tangent.z / q,
        );
        Frame {
            center,
            tangent,
            normal,
            binormal,
        }
    }
    pub(super) fn inverse(&self, position: [f64; 3]) -> Result<[f64; 3]> {
        let p = DVec3::from_array(position);
        if !p.is_finite() {
            return Err(invalid("Trajectory point must be finite"));
        }
        let field = |t| {
            let (c, d) = self.curve(t);
            (p - c).dot(d)
        };
        let t = if field(0.) <= 0. {
            0.
        } else if field(1.) >= 0. {
            1.
        } else {
            let (mut low, mut high) = (0., 1.);
            for _ in 0..48 {
                let mid = (low + high) * 0.5;
                if field(mid) > 0. {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            (low + high) * 0.5
        };
        let frame = self.frame(t);
        let delta = p - frame.center;
        if delta.dot(frame.tangent).abs() > self.height * MAPPING_TOLERANCE {
            return Err(invalid(
                "Plant bind origin lies beyond the trajectory end planes",
            ));
        }
        Ok([delta.dot(frame.normal), t, delta.dot(frame.binormal)])
    }
}
