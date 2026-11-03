//! Author-time fire recipes compiled into the ordinary emitter/material contracts.
use super::*;
use crate::materials::recipe::{Fixed, Procedural, Recipe, Sample as Texel};
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Candle,
    #[default]
    Campfire,
    Torch,
    BlueJet,
    Inferno,
    Arcane,
}
impl Kind {
    pub const ALL: [Self; 6] = [
        Self::Candle,
        Self::Campfire,
        Self::Torch,
        Self::BlueJet,
        Self::Inferno,
        Self::Arcane,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Candle => "Vela",
            Self::Campfire => "Fogueira",
            Self::Torch => "Tocha",
            Self::BlueJet => "Jato azul",
            Self::Inferno => "Incêndio",
            Self::Arcane => "Arcano",
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::Candle => "candle",
            Self::Campfire => "campfire",
            Self::Torch => "torch",
            Self::BlueJet => "blue_jet",
            Self::Inferno => "inferno",
            Self::Arcane => "arcane",
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub kind: Kind,
    pub intensity: f32,
    pub height: f32,
    pub width: f32,
    pub turbulence: f32,
    pub seed: u32,
}
/// Immutable derivation evidence, rebuilt once at native recipe admission.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub settings: Settings,
    original: Settings,
    emitters: Arc<Vec<Emitter>>,
}
impl Binding {
    pub(super) fn new(settings: Settings, emitters: &[Emitter]) -> Self {
        Self {
            settings,
            original: settings,
            emitters: Arc::new(emitters.to_vec()),
        }
    }
    pub(super) fn matches(&self, source: &Source) -> bool {
        self.settings == self.original
            && source.seed == self.settings.seed
            && source.duration_ticks == 180
            && source.looping
            && source.emitters == *self.emitters
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self::for_kind(Kind::Campfire)
    }
}
impl Settings {
    pub fn for_kind(kind: Kind) -> Self {
        let (height, width, turbulence) = match kind {
            Kind::Candle => (0.7, 0.14, 0.12),
            Kind::Campfire => (2., 1.2, 0.7),
            Kind::Torch => (1.4, 0.35, 0.4),
            Kind::BlueJet => (2.4, 0.4, 0.15),
            Kind::Inferno => (3.5, 2.3, 1.5),
            Kind::Arcane => (2.2, 0.85, 1.2),
        };
        Self {
            kind,
            intensity: 1.,
            height,
            width,
            turbulence,
            seed: 7,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if !finite_range(&[self.intensity], 0., 3.)
            || !finite_range(&[self.height], 0.1, 8.)
            || !finite_range(&[self.width], 0.03, 5.)
            || !finite_range(&[self.turbulence], 0., 2.)
        {
            return Err(invalid(
                "Fire requires finite intensity0..3, height0.1..8m, width0.03..5m, turbulence0..2",
            ));
        }
        Ok(())
    }
    fn rate(self, base: f32) -> f32 {
        if self.intensity == 0. {
            0.
        } else {
            (base * self.intensity).max(0.001)
        }
    }
    /// Fully transparent map borders avoid wrapping seams. Coordinates are sprite XY.
    pub fn texel(self, uv: [f32; 2]) -> Texel {
        let x = uv[0];
        let y = uv[1];
        let t = ((y - 0.04) / 0.92).clamp(0., 1.);
        let phase = ((self.seed.wrapping_mul(1664525).wrapping_add(1013904223) >> 8) as f32
            / 16777216.)
            * TAU;
        let taper = match self.kind {
            Kind::Candle => 0.68,
            Kind::BlueJet => 0.28,
            Kind::Torch => 0.6,
            Kind::Inferno => 0.48,
            Kind::Arcane => 0.55,
            Kind::Campfire => 0.65,
        };
        let tongues = (t * 17. + phase).sin() * 0.065 + (t * 37. + phase * 1.7).sin() * 0.028;
        let center =
            0.5 + self.turbulence * (0.055 * (t * 8. + phase).sin() + tongues * t * 0.5) * t;
        let half = (0.34 * (1. - t).powf(taper) + 0.008) * (1. + tongues * self.turbulence * 2.);
        let distance = (x - center).abs() / half.max(0.004);
        let rim = 1. - smooth(0.65, 1.05, distance);
        let alpha = (smooth(0.03, 0.07, x)
            * (1. - smooth(0.93, 0.97, x))
            * rim
            * smooth(0.04, 0.18, y)
            * (1. - smooth(0.84, 0.96, y))
            * 0.62)
            .clamp(0., 1.);
        let heat = ((1. - distance).clamp(0., 1.) * 0.65 + (1. - t) * 0.35).powf(1.4);
        let (outer, inner) = match self.kind {
            Kind::BlueJet => ([0.008, 0.045, 0.9], [0.34, 0.88, 1.]),
            Kind::Arcane => ([0.34, 0.003, 0.9], [0.8, 0.48, 1.]),
            Kind::Candle => ([0.9, 0.15, 0.008], [1., 0.94, 0.48]),
            _ => ([0.95, 0.055, 0.002], [1., 0.91, 0.32]),
        };
        let gain = 1.45 + 0.12 * self.intensity;
        Texel {
            color_roughness: [0., 0., 0., 0.7],
            response: [0., 0.5, alpha, 1.],
            emission: [
                (outer[0] + (inner[0] - outer[0]) * heat) * gain,
                (outer[1] + (inner[1] - outer[1]) * heat) * gain,
                (outer[2] + (inner[2] - outer[2]) * heat) * gain,
                0.,
            ],
        }
    }
    pub fn material(self) -> Material {
        const N: usize = 32;
        let mut samples = Vec::with_capacity(crate::materials::recipe::sample_count(N as u16));
        for y in 0..N {
            for x in 0..N {
                let at = |dx: f32, dy: f32| {
                    self.texel([(x as f32 + dx) / N as f32, (y as f32 + dy) / N as f32])
                };
                samples.push(
                    at(0.25, 0.25)
                        .mix(at(0.75, 0.25), 0.5)
                        .mix(at(0.25, 0.75).mix(at(0.75, 0.75), 0.5), 0.5),
                );
            }
        }
        let (mut size, mut start) = (N, 0);
        while size > 1 {
            let next = size / 2;
            for y in 0..next {
                for x in 0..next {
                    let i = start + 2 * y * size + 2 * x;
                    samples.push(
                        samples[i]
                            .mix(samples[i + 1], 0.5)
                            .mix(samples[i + size].mix(samples[i + size + 1], 0.5), 0.5),
                    );
                }
            }
            start += size * size;
            size = next;
        }
        Material {
            surface: Some(Surface {
                mode: BlendMode::Blend,
                ..Default::default()
            }),
            procedural: Some(Arc::new(Procedural {
                version: 1,
                recipe: Recipe {
                    seed: self.seed,
                    plane: crate::materials::Plane::Xy,
                    tile_size: [1.; 2],
                    unique_instances: false,
                    height_m: 0.,
                    relief: crate::materials::recipe::Relief::Bump,
                    ..Default::default()
                },
                fixed: Some(Fixed {
                    resolution: N as u16,
                    samples: Arc::new(samples),
                }),
            })),
            ..Default::default()
        }
    }
    pub(super) fn emitters(self) -> Vec<Emitter> {
        let h = self.height;
        let w = self.width;
        let q = self.turbulence;
        let power = self.intensity;
        let (rate, rise, life, spread) = match self.kind {
            Kind::Candle => (28., 0.5, [18, 40], 4.),
            Kind::Campfire => (80., 0.8, [40, 70], 15.),
            Kind::Torch => (90., 1.1, [28, 52], 8.),
            Kind::BlueJet => (100., 2., [15, 34], 3.),
            Kind::Inferno => (150., 1., [38, 75], 22.),
            Kind::Arcane => (85., 0.8, [40, 65], 10.),
        };
        let velocity = h * rise * (0.9 + 0.1 * power);
        let mut body = Emitter {
            id: "flames".into(),
            rate_per_second: self.rate(rate),
            shape: Shape::Disc { radius: w * 0.2 },
            direction: [0., 1., 0.],
            spread_degrees: spread,
            speed: [velocity * 0.55, velocity],
            lifetime_ticks: life,
            size: [h * 0.45, h * 0.75],
            rotation_radians: [-0.12 * q, 0.12 * q],
            angular_velocity: [-0.2 * q, 0.2 * q],
            aspect: (w / h * 1.4).clamp(0.001, 100.),
            softness: 0.1,
            material: self.material(),
            modules: vec![
                Module::Offset {
                    value: [0., h * 0.2, 0.],
                },
                Module::Acceleration {
                    value: [0., h * 0.3, 0.],
                },
                Module::Drag { coefficient: 0.35 },
                Module::Wave {
                    amplitude: [w * 0.15 * q, h * 0.02 * q, w * 0.12 * q],
                    frequency_hz: [1.8, 2.6, 2.1],
                },
                Module::SizeOverLife {
                    keys: vec![
                        SizeKey {
                            time: 0.,
                            value: 0.45,
                        },
                        SizeKey {
                            time: 0.18,
                            value: 1.,
                        },
                        SizeKey {
                            time: 0.7,
                            value: 0.8,
                        },
                        SizeKey {
                            time: 1.,
                            value: 0.06,
                        },
                    ],
                },
                Module::ColorOverLife {
                    keys: vec![
                        ColorKey {
                            time: 0.,
                            value: [1., 1., 1., 0.],
                        },
                        ColorKey {
                            time: 0.12,
                            value: [1., 1., 1., 0.85],
                        },
                        ColorKey {
                            time: 0.65,
                            value: [0.9, 0.9, 0.9, 0.6],
                        },
                        ColorKey {
                            time: 1.,
                            value: [0.45, 0.45, 0.45, 0.],
                        },
                    ],
                },
            ],
            ..Default::default()
        };
        let mut tips = body.clone();
        tips.id = "tongues".into();
        tips.seed = 31;
        tips.rate_per_second = self.rate(rate * 0.4);
        tips.aspect *= 0.65;
        tips.speed = body.speed.map(|v| v * 1.3);
        tips.size = body.size.map(|v| v * 0.75);
        tips.shape = Shape::Disc { radius: w * 0.12 };
        tips.modules[0] = Module::Offset {
            value: [0., h * 0.45, 0.],
        };
        // A focused core has less positional spread; broad families expose more tongues.
        if matches!(self.kind, Kind::Candle | Kind::BlueJet) {
            body.rotation_radians = [-0.02, 0.02];
            body.angular_velocity = [0.; 2];
        }
        let mut out = vec![body, tips];
        if matches!(self.kind, Kind::Campfire | Kind::Torch | Kind::Inferno) {
            let mut smoke = Source::preset(Preset::Smoke).emitters.remove(0);
            smoke.id = "smoke".into();
            smoke.rate_per_second = self.rate(if self.kind == Kind::Inferno { 30. } else { 9. });
            smoke.size = [h * 0.2, h * 0.4];
            smoke.speed = [h * 0.25, h * 0.45];
            smoke.lifetime_ticks = [90, 200];
            smoke.modules.insert(
                0,
                Module::Offset {
                    value: [0., h * 0.65, 0.],
                },
            );
            for m in &mut smoke.modules {
                if let Module::ColorOverLife { keys } = m {
                    for k in keys {
                        k.value[3] *= 0.32;
                    }
                }
            }
            out.push(smoke);
            let mut sparks = Source::preset(Preset::Sparks).emitters.remove(0);
            sparks.id = "embers".into();
            sparks.burst = 0;
            sparks.rate_per_second = self.rate(if self.kind == Kind::Inferno { 28. } else { 8. });
            sparks.shape = Shape::Disc { radius: w * 0.15 };
            sparks.size = [0.018, 0.045];
            sparks.speed = [h * 0.9, h * 1.8];
            sparks.modules.insert(
                0,
                Module::Offset {
                    value: [0., h * 0.15, 0.],
                },
            );
            out.push(sparks);
        }
        out
    }
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
impl Source {
    pub fn fire(id: String, settings: Settings) -> Result<Self> {
        super::id(&id)?;
        settings.validate()?;
        let emitters = settings.emitters();
        let fire = Some(Arc::new(Binding::new(settings, &emitters)));
        Ok(Self {
            version: 2,
            id,
            seed: settings.seed,
            duration_ticks: 180,
            looping: true,
            emitters,
            fire,
        })
    }
}
