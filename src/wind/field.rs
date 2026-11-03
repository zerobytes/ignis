use super::*;
use std::f64::consts::TAU;
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Field {
    pub mean: [f64; 3],
    pub gusts: Vec<Gust>,
    pub turbulence: Option<Turbulence>,
    pub jets: Vec<Jet>,
    pub vortices: Vec<Vortex>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Gust {
    pub amplitude: [f64; 3],
    pub period_seconds: f64,
    pub phase_radians: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Turbulence {
    pub seed: u32,
    pub modes: u16,
    pub rms_speed: f64,
    pub min_wavelength: f64,
    pub max_wavelength: f64,
    pub advection: [f64; 3],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Jet {
    pub origin: [f64; 3],
    pub direction: [f64; 3],
    pub transverse: [f64; 3],
    pub speed: f64,
    pub width: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Vortex {
    pub center: [f64; 3],
    pub axis: [f64; 3],
    pub velocity: [f64; 3],
    pub radius: f64,
    pub half_height: f64,
    pub peak_swirl_speed: f64,
    pub updraft_speed: f64,
    pub start_seconds: f64,
    /// None is persistent; Some has a compact C-infinity lifetime envelope.
    pub duration_seconds: Option<f64>,
    /// C-infinity onset duration for persistent events.
    #[serde(default = "fade_seconds")]
    pub fade_seconds: f64,
}
fn fade_seconds() -> f64 {
    1.
}
impl Field {
    pub fn sample_work(&self) -> usize {
        1 + self.gusts.len()
            + self.jets.len()
            + self.vortices.len()
            + self.turbulence.as_ref().map_or(0, |t| usize::from(t.modes))
    }
    pub fn validate(&self) -> Result<()> {
        if !vec(self.mean, 200.)
            || self.gusts.len() > 16
            || self.jets.len() > 16
            || self.vortices.len() > 32
        {
            return Err(invalid("Wind field values/event budget"));
        }
        for g in &self.gusts {
            if !vec(g.amplitude, 200.)
                || !bounded(g.period_seconds, 0.1, 1e8)
                || !bounded(g.phase_radians, -TAU, TAU)
            {
                return Err(invalid("Invalid gust amplitude, period or phase"));
            }
        }
        if let Some(t) = &self.turbulence
            && (!(1..=64).contains(&t.modes)
                || !bounded(t.rms_speed, 0., 100.)
                || !bounded(t.min_wavelength, 0.05, 1e6)
                || !bounded(t.max_wavelength, t.min_wavelength, 1e6)
                || !vec(t.advection, 200.))
        {
            return Err(invalid("Invalid turbulence spectrum or mode budget"));
        }
        for j in &self.jets {
            if !vec(j.origin, MAX_COORDINATE)
                || !direction(j.direction)
                || !direction(j.transverse)
                || DVec3::from_array(j.direction)
                    .dot(DVec3::from_array(j.transverse))
                    .abs()
                    > 1e-10
                || !bounded(j.speed, -200., 200.)
                || !bounded(j.width, 0.05, 1e6)
            {
                return Err(invalid(
                    "Jet requires perpendicular unit axes, finite width and speed",
                ));
            }
        }
        for v in &self.vortices {
            if !vec(v.center, MAX_COORDINATE)
                || !direction(v.axis)
                || !vec(v.velocity, 200.)
                || !bounded(v.radius, 0.05, 1e6)
                || !bounded(v.half_height, 0.05, 1e6)
                || !bounded(v.peak_swirl_speed, -200., 200.)
                || !bounded(v.updraft_speed, -200., 200.)
                || !bounded(v.start_seconds, 0., 1e9)
                || v.duration_seconds.is_some_and(|t| !bounded(t, 0.1, 1e9))
                || !bounded(v.fade_seconds, 0.01, 1e6)
            {
                return Err(invalid(
                    "Invalid compact vortex geometry, velocity or lifetime",
                ));
            }
        }
        // Conservative global envelope, including compact return flows. Reject
        // unsupported inputs instead of clipping velocities and breaking div(v)=0.
        let upper = DVec3::from_array(self.mean).length()
            + self
                .gusts
                .iter()
                .map(|g| DVec3::from_array(g.amplitude).length())
                .sum::<f64>()
            + self.jets.iter().map(|j| j.speed.abs()).sum::<f64>()
            + self
                .turbulence
                .as_ref()
                .map_or(0., |t| (2. * f64::from(t.modes)).sqrt() * t.rms_speed)
            + self
                .vortices
                .iter()
                .map(|v| {
                    v.peak_swirl_speed.abs()
                        + v.updraft_speed.abs()
                            * (1. + 3. * (1. + (v.radius / v.half_height).powi(2)).sqrt())
                })
                .sum::<f64>();
        if upper > 300. {
            return Err(invalid(
                "Wind conservative speed envelope exceeds300m/s; reduce the field amplitudes/aspect ratio",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
struct Mode {
    k: DVec3,
    b: DVec3,
    phase: f64,
    omega: f64,
}
#[derive(Clone, Default)]
pub struct CompiledField {
    source: Field,
    modes: Vec<Mode>,
}
impl CompiledField {
    pub fn new(source: &Field) -> Result<Self> {
        source.validate()?;
        let mut out = Self {
            source: source.clone(),
            modes: vec![],
        };
        // Admission accepts decimal unit vectors within tolerance; compile an
        // orthonormal basis so that input rounding cannot add jet divergence.
        for j in &mut out.source.jets {
            let transverse = DVec3::from_array(j.transverse).normalize();
            let side = DVec3::from_array(j.direction).cross(transverse).normalize();
            j.transverse = transverse.to_array();
            j.direction = transverse.cross(side).to_array();
        }
        for v in &mut out.source.vortices {
            v.axis = DVec3::from_array(v.axis).normalize().to_array();
        }
        if let Some(t) = &source.turbulence {
            let mut rng = u64::from(t.seed);
            let mut random = || {
                rng = rng.wrapping_add(0x9e3779b97f4a7c15);
                let mut v = rng;
                v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
                ((v ^ (v >> 31)) >> 11) as f64 / (1u64 << 53) as f64
            };
            let mut sum = 0.;
            for n in 0..t.modes {
                let z = 2. * random() - 1.;
                let angle = TAU * random();
                let radial = (1. - z * z).sqrt();
                let axis = DVec3::new(radial * angle.cos(), z, radial * angle.sin());
                let helper = if axis.y.abs() < 0.9 {
                    DVec3::Y
                } else {
                    DVec3::X
                };
                let a = axis.cross(helper).normalize();
                let b = axis.cross(a);
                let angle = TAU * random();
                let fraction = (f64::from(n) + 0.5) / f64::from(t.modes);
                let wavelength =
                    t.min_wavelength * (t.max_wavelength / t.min_wavelength).powf(fraction);
                // E(k) ~ k^-5/3: energy per logarithmic band ~ k^-2/3.
                let weight = wavelength.powf(1. / 3.);
                let k = axis * (TAU / wavelength);
                out.modes.push(Mode {
                    k,
                    b: (a * angle.cos() + b * angle.sin()) * weight,
                    phase: TAU * random(),
                    omega: -k.dot(DVec3::from_array(t.advection)),
                });
                sum += weight * weight;
            }
            let scale = t.rms_speed * (2. / sum).sqrt();
            for m in &mut out.modes {
                m.b *= scale;
            }
        }
        Ok(out)
    }
    /// Prepare time-only terms once for a batch of spatial queries. Stack-only.
    pub fn at_time(&self, seconds: f64) -> FieldSnapshot<'_> {
        let mut snapshot = FieldSnapshot {
            field: self,
            mean: DVec3::from_array(self.source.mean),
            phases: [0.; 64],
            events: [None; 32],
        };
        for g in &self.source.gusts {
            let phase = TAU * (seconds % g.period_seconds) / g.period_seconds + g.phase_radians;
            snapshot.mean += DVec3::from_array(g.amplitude) * phase.sin();
        }
        for (m, p) in self.modes.iter().zip(&mut snapshot.phases) {
            *p = (m.omega * seconds).rem_euclid(TAU) + m.phase;
        }
        for (v, out) in self.source.vortices.iter().zip(&mut snapshot.events) {
            let age = seconds - v.start_seconds;
            if age < 0. || v.duration_seconds.is_some_and(|d| age >= d) {
                continue;
            }
            let envelope = v.duration_seconds.map_or_else(
                || {
                    let t = (age / v.fade_seconds).min(1.);
                    if t <= 0. {
                        0.
                    } else if t >= 1. {
                        1.
                    } else {
                        let a = (-1. / t).exp();
                        let b = (-1. / (1. - t)).exp();
                        a / (a + b)
                    }
                },
                |d| {
                    let t = age / d;
                    (4. - 1. / (t * (1. - t))).exp()
                },
            );
            // For f(s)=exp[-s/(1-s)], the peak of sqrt(s) f(s)/(1-s)^2
            // is at s=1/sqrt(3). Normalize swirl speed to that analytic maximum.
            let peak = 1. / 3f64.sqrt();
            let swirl_scale = 0.5 * (1. - peak).powi(2) * (peak / (1. - peak)).exp() / peak.sqrt();
            *out = Some(PreparedVortex {
                center: DVec3::from_array(v.center) + DVec3::from_array(v.velocity) * age,
                axis: DVec3::from_array(v.axis),
                inverse_radius_squared: 1. / (v.radius * v.radius),
                inverse_height_squared: 1. / (v.half_height * v.half_height),
                potential_scale: v.peak_swirl_speed * v.radius * swirl_scale,
                updraft: v.updraft_speed,
                envelope,
            });
        }
        snapshot
    }
    /// Local scene coordinates in meters; no allocation or mutable random stream.
    /// For repeated queries at one instant, use `at_time` once instead.
    pub fn sample(&self, x: DVec3, seconds: f64) -> DVec3 {
        self.at_time(seconds).sample(x)
    }
}
#[derive(Clone, Copy)]
struct PreparedVortex {
    center: DVec3,
    axis: DVec3,
    inverse_radius_squared: f64,
    inverse_height_squared: f64,
    potential_scale: f64,
    updraft: f64,
    envelope: f64,
}
pub struct FieldSnapshot<'a> {
    field: &'a CompiledField,
    mean: DVec3,
    phases: [f64; 64],
    events: [Option<PreparedVortex>; 32],
}
impl FieldSnapshot<'_> {
    pub fn sample(&self, x: DVec3) -> DVec3 {
        let mut flow = self.mean;
        for (m, phase) in self.field.modes.iter().zip(&self.phases) {
            flow += m.b * (m.k.dot(x) + phase).sin();
        }
        for j in &self.field.source.jets {
            let d =
                (x - DVec3::from_array(j.origin)).dot(DVec3::from_array(j.transverse)) / j.width;
            flow += DVec3::from_array(j.direction) * (j.speed * (-d * d).exp());
        }
        for v in self.events.iter().flatten() {
            let r = x - v.center;
            let h = r.dot(v.axis);
            let radial = r - v.axis * h;
            let s = radial.length_squared() * v.inverse_radius_squared
                + h * h * v.inverse_height_squared;
            if s >= 1. {
                continue;
            }
            let f = (-s / (1. - s)).exp();
            let grad = -2. * f / (1. - s).powi(2)
                * (radial * v.inverse_radius_squared + v.axis * (h * v.inverse_height_squared));
            let swirl = grad.cross(v.axis) * v.potential_scale;
            // curl[.5 U×r f] = U f + .5 grad(f)×(U×r).
            let up = v.axis * (v.updraft * f) + grad.cross(v.axis.cross(r)) * (0.5 * v.updraft);
            flow += (swirl + up) * v.envelope;
        }
        flow
    }
}
