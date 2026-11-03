//! Authorial rest curvature, composed with the source curve before child placement.
use super::{
    bake::{Frame, normal},
    *,
};
use glam::DQuat;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Deformation {
    /// Curvature toward the material normal/binormal, rad/m, at base and tip.
    pub base: [f64; 2],
    pub tip: [f64; 2],
    pub twist: f64,
    pub sag: f64,
    pub tropism: f64,
    pub irregularity: f64,
    pub frequency: f64,
}
impl Default for Deformation {
    fn default() -> Self {
        Self {
            base: [0.; 2],
            tip: [0.; 2],
            twist: 0.,
            sag: 0.,
            tropism: 0.,
            irregularity: 0.,
            frequency: 2.,
        }
    }
}
impl Deformation {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
    fn active(&self) -> bool {
        self.base != [0.; 2]
            || self.tip != [0.; 2]
            || self.twist != 0.
            || self.sag != 0.
            || self.tropism != 0.
            || self.irregularity != 0.
    }
    pub fn validate(&self) -> Result<()> {
        if self
            .base
            .iter()
            .chain(&self.tip)
            .chain([&self.twist, &self.sag, &self.tropism, &self.irregularity])
            .any(|x| !x.is_finite() || x.abs() > 10.)
            || !self.frequency.is_finite()
            || !(0.1..=16.).contains(&self.frequency)
        {
            return Err(invalid("Invalid botanical curvature field"));
        }
        Ok(())
    }
}
pub fn apply(base: Vec<Frame>, d: &Deformation, seed: u64, id: &str) -> Result<Vec<Frame>> {
    d.validate()?;
    if !d.active() {
        return Ok(base);
    }
    // Refine the existing material polyline, rather than resampling the source
    // spline: enabling a field must not change its rest length or physical mass.
    let subdivisions = 128usize.div_ceil(base.len() - 1);
    let mut refined = Vec::with_capacity((base.len() - 1) * subdivisions + 1);
    refined.push(base[0]);
    for pair in base.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        for k in 1..=subdivisions {
            let f = k as f64 / subdivisions as f64;
            let t = a.t.lerp(b.t, f).normalize();
            refined.push(Frame {
                p: a.p.lerp(b.p, f),
                t,
                n: normal(t, a.n.lerp(b.n, f)),
            });
        }
    }
    let base = refined;
    let mut hash = seed;
    for b in id.bytes() {
        hash = (hash ^ u64::from(b)).wrapping_mul(0x100000001b3);
    }
    let phase = (hash >> 11) as f64 / (1u64 << 53) as f64 * std::f64::consts::TAU;
    let mut result = Vec::with_capacity(base.len());
    result.push(base[0]);
    let mut q = DQuat::IDENTITY;
    let mut p = base[0].p;
    for i in 1..base.len() {
        let u = (i as f64 - 0.5) / (base.len() - 1) as f64;
        let s = u * u * (3. - 2. * u);
        let original = base[i - 1];
        let t = q * original.t;
        let n = q * original.n;
        let b = t.cross(n);
        let ripple = std::f64::consts::TAU * d.frequency * u + phase;
        let kn = d.base[0] * (1. - s) + d.tip[0] * s + d.irregularity * ripple.sin();
        let kb = d.base[1] * (1. - s) + d.tip[1] * s + d.irregularity * (ripple * 0.73 + 1.7).sin();
        let up = DVec3::Y - t * t.y;
        let down = -up;
        let derivative = n * kn + b * kb + up * d.tropism + down * (d.sag * (1. - u));
        let omega = t.cross(derivative) + t * d.twist;
        let segment = base[i].p - original.p;
        let ds = segment.length();
        if omega.length() * ds > 0.35 {
            return Err(invalid(
                "Curvature exceeds integration resolution; reduce field strength or organ length",
            ));
        }
        let half = DQuat::from_scaled_axis(omega * (ds * 0.5));
        p += half * q * segment;
        q = (DQuat::from_scaled_axis(omega * ds) * q).normalize();
        let t = q * base[i].t;
        let n = normal(t, q * base[i].n);
        if !p.is_finite() || p.abs().max_element() > 10000. {
            return Err(invalid("Botanical deformation exceeded coordinate bounds"));
        }
        result.push(Frame { p, t, n });
    }
    Ok(result)
}
