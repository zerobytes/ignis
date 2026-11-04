//! Versioned, organ-local woody shape authoring. Metres are in source coordinates.
//! Star-shaped sections support concavity, not tunnels or arbitrary CSG.
use super::*;
#[path = "wood_surface_ui.rs"]
pub mod ui;
use std::f64::consts::TAU;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub id: String,
    pub u: f64,
    pub angle: f64,
    pub offset: [f64; 2],
    /// Evenly spaced periodic radial samples, in metres.
    pub radii: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub u: f64,
    /// Unwrapped radians, allowing a cord to wind around the axis.
    pub angle: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Relief {
    pub id: String,
    pub enabled: bool,
    pub path: Vec<Control>,
    pub width: f64,
    /// Signed metres: positive cord/bulge, negative valley.
    pub height: f64,
    pub end_fade: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub sections: Vec<Section>,
    pub relief: Vec<Relief>,
    pub sides: u8,
    pub samples: u16,
}
impl Default for Surface {
    fn default() -> Self {
        Self {
            sections: vec![
                Section {
                    id: "base".into(),
                    u: 0.,
                    angle: 0.,
                    offset: [0.; 2],
                    radii: vec![1.; 8],
                },
                Section {
                    id: "top".into(),
                    u: 1.,
                    angle: 0.,
                    offset: [0.; 2],
                    radii: vec![0.8; 8],
                },
            ],
            relief: vec![],
            sides: 32,
            samples: 48,
        }
    }
}
fn smooth(x: f64) -> f64 {
    let x = x.clamp(0., 1.);
    x * x * (3. - 2. * x)
}
fn span<T>(items: &[T], u: f64, at: impl Fn(&T) -> f64) -> (&T, &T, f64) {
    let i = items
        .partition_point(|p| at(p) < u)
        .saturating_sub(1)
        .min(items.len() - 2);
    let a = &items[i];
    let b = &items[i + 1];
    (a, b, smooth((u - at(a)) / (at(b) - at(a))))
}
impl Surface {
    pub fn validate(&self) -> Result<()> {
        if !(2..=32).contains(&self.sections.len())
            || self.relief.len() > 32
            || !(12..=32).contains(&self.sides)
            || !(8..=128).contains(&self.samples)
        {
            return Err(invalid("Wood surface: section/relief/resolution budget"));
        }
        let mut ids = BTreeSet::new();
        for p in &self.sections {
            if !identifier(&p.id)
                || !ids.insert(&p.id)
                || !p.u.is_finite()
                || !(0. ..=1.).contains(&p.u)
                || !p.angle.is_finite()
                || p.angle.abs() > 100.
                || p.offset.iter().any(|x| !x.is_finite())
                || !(8..=32).contains(&p.radii.len())
                || p.radii.len() > self.sides as usize
                || p.radii.iter().any(|r| !r.is_finite() || *r < 0.02)
            {
                return Err(invalid(format!(
                    "Wood section {}: invalid identity, contour or coordinates",
                    p.id
                )));
            }
        }
        if self.sections[0].u != 0.
            || self.sections.last().unwrap().u != 1.
            || self.sections.windows(2).any(|p| p[1].u - p[0].u < 0.005)
        {
            return Err(invalid(
                "Wood sections must be ordered, span0..1 and spaced at least0.005",
            ));
        }
        for r in &self.relief {
            if !identifier(&r.id)
                || !ids.insert(&r.id)
                || !(2..=32).contains(&r.path.len())
                || !r.width.is_finite()
                || r.width < 0.02
                || !r.height.is_finite()
                || !r.end_fade.is_finite()
                || !(0. ..=0.5).contains(&r.end_fade)
                || r.path.iter().any(|p| {
                    !p.u.is_finite()
                        || !(0. ..=1.).contains(&p.u)
                        || !p.angle.is_finite()
                        || p.angle.abs() > 100.
                })
                || r.path.windows(2).any(|p| p[1].u - p[0].u < 0.005)
            {
                return Err(invalid(format!(
                    "Wood relief {}: invalid identity/path/physical dimensions",
                    r.id
                )));
            }
        }
        // Sample the actual requested raster grid plus a denser diagnostic grid.
        // This detects radial collapse; it is not a global self-intersection certificate.
        for i in 0..=self.samples as usize * 2 {
            for j in 0..self.sides as usize * 2 {
                let u = i as f64 / (self.samples as f64 * 2.);
                let a = TAU * j as f64 / (self.sides as f64 * 2.);
                let radius = self.radius(u, a);
                if !radius.is_finite() || radius < 0.02 {
                    return Err(invalid(
                        "Wood relief overflows or collapses below0.02m; revise contour/relief",
                    ));
                }
            }
        }
        Ok(())
    }
    fn contour(p: &Section, a: f64) -> f64 {
        let x = ((a - p.angle) / TAU).rem_euclid(1.) * p.radii.len() as f64;
        let i = x.floor() as usize;
        let f = smooth(x - i as f64);
        p.radii[i % p.radii.len()] * (1. - f) + p.radii[(i + 1) % p.radii.len()] * f
    }
    pub fn radius(&self, u: f64, a: f64) -> f64 {
        let (p, q, t) = span(&self.sections, u, |p| p.u);
        let base = Self::contour(p, a) * (1. - t) + Self::contour(q, a) * t;
        self.relief
            .iter()
            .filter(|r| r.enabled && u >= r.path[0].u && u <= r.path.last().unwrap().u)
            .fold(base, |v, r| {
                let (p, q, t) = span(&r.path, u, |p| p.u);
                let angle = p.angle * (1. - t) + q.angle * t;
                let distance = 2. * base * ((a - angle) * 0.5).sin().abs();
                let profile = (1. - (distance / r.width).powi(2)).max(0.).powi(2);
                let start = r.path[0].u;
                let end = r.path.last().unwrap().u;
                let fade = if r.end_fade == 0. {
                    1.
                } else {
                    smooth((u - start) / r.end_fade) * smooth((end - u) / r.end_fade)
                };
                v + r.height * profile * fade
            })
    }
    pub fn offset(&self, u: f64) -> [f64; 2] {
        let (p, q, t) = span(&self.sections, u, |p| p.u);
        std::array::from_fn(|i| p.offset[i] * (1. - t) + q.offset[i] * t)
    }
    pub fn point(&self, frame: bake::Frame, u: f64, theta: f64, scale: f64) -> DVec3 {
        let r = self.radius(u, theta) * scale;
        let offset = self.offset(u);
        frame.p
            + frame.n * (offset[0] * scale + r * theta.cos())
            + frame.b() * (offset[1] * scale + r * theta.sin())
    }
}
