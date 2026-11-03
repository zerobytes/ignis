//! Reconstructible cosmetic effects. Neither actor state nor a physics solver.
use crate::{
    materials::{BlendMode, Material, Surface},
    model::{Result, invalid},
    world_space::{Coordinates, Position},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};
mod evaluate;
pub mod fire;
pub const FIRE_SCENE_VERSION: u32 = 118;
pub use evaluate::{Frame, Sample, Stats, trajectory};
pub fn document_empty(d: &Arc<Document>) -> bool {
    d.is_empty()
}
pub const VERSION: u16 = 1;
pub const SCENE_VERSION: u32 = 113;
pub const MAX_BYTES: usize = 8 * 1024 * 1024;
pub fn origin() -> Position {
    Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [0.; 3],
    })
    .unwrap()
}
fn id(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid(
            "Particle identifiers require1..64 ASCII letters/digits/_/-",
        ));
    }
    Ok(())
}
fn one() -> f32 {
    1.
}
fn yes() -> bool {
    true
}
fn period() -> u32 {
    300
}
fn budget() -> u32 {
    8192
}
fn emitter_budget() -> u32 {
    4096
}
fn rotation() -> [f32; 4] {
    [0., 0., 0., 1.]
}
fn white() -> [f32; 4] {
    [1.; 4]
}
fn default_material() -> Material {
    Material {
        surface: Some(Surface {
            mode: BlendMode::Blend,
            ..Default::default()
        }),
        ..Default::default()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u16,
    #[serde(default = "budget")]
    pub max_particles: u32,
    #[serde(default)]
    pub sources: Vec<Arc<Source>>,
    #[serde(default)]
    pub instances: Vec<Placement>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: VERSION,
            max_particles: budget(),
            sources: vec![],
            instances: vec![],
        }
    }
}
#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq)]
#[serde(try_from = "SourceWire")]
#[schemars(with = "SourceWire")]
pub struct Source {
    pub version: u16,
    pub id: String,
    pub seed: u32,
    pub duration_ticks: u32,
    pub looping: bool,
    pub emitters: Vec<Emitter>,
    pub fire: Option<Arc<fire::Binding>>,
}
// Recipe-bound fire persists the compact recipe; native admission rebuilds maps.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SourceWire {
    version: u16,
    id: String,
    seed: u32,
    #[serde(default = "period")]
    duration_ticks: u32,
    #[serde(default = "yes")]
    looping: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    emitters: Option<Vec<Emitter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fire: Option<fire::Settings>,
}
impl TryFrom<SourceWire> for Source {
    type Error = crate::model::Error;
    fn try_from(w: SourceWire) -> Result<Self> {
        let source = if let Some(settings) = w.fire {
            let source = Self::fire(w.id, settings)?;
            if w.seed != source.seed
                || w.duration_ticks != source.duration_ticks
                || w.looping != source.looping
                || w.version != 2
                || w.emitters.as_ref().is_some_and(|e| e != &source.emitters)
            {
                return Err(invalid(
                    "Fire source recipe and serialized derivation disagree",
                ));
            }
            source
        } else {
            Self {
                version: w.version,
                id: w.id,
                seed: w.seed,
                duration_ticks: w.duration_ticks,
                looping: w.looping,
                emitters: w
                    .emitters
                    .ok_or_else(|| invalid("Particle source requires emitters"))?,
                fire: None,
            }
        };
        source.validate()?;
        Ok(source)
    }
}
impl Serialize for Source {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        if self.fire.is_some() {
            self.validate().map_err(serde::ser::Error::custom)?;
        }
        SourceWire {
            version: self.version,
            id: self.id.clone(),
            seed: self.seed,
            duration_ticks: self.duration_ticks,
            looping: self.looping,
            emitters: self.fire.is_none().then(|| self.emitters.clone()),
            fire: self.fire.as_ref().map(|r| r.settings),
        }
        .serialize(serializer)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub id: String,
    pub source: String,
    #[serde(default)]
    pub seed: u32,
    #[serde(default = "origin")]
    pub position: Position,
    #[serde(default = "rotation")]
    pub rotation: [f32; 4],
    #[serde(default = "one")]
    pub scale: f32,
    #[serde(default = "white")]
    pub tint: [f32; 4],
    #[serde(default)]
    pub start_tick: u64,
    #[serde(default = "yes")]
    pub enabled: bool,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Shape {
    #[default]
    Point,
    Sphere {
        radius: f32,
    },
    Box {
        half_extents: [f32; 3],
    },
    Disc {
        radius: f32,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ColorKey {
    pub time: f32,
    pub value: [f32; 4],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SizeKey {
    pub time: f32,
    pub value: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Module {
    Offset {
        value: [f32; 3],
    },
    Acceleration {
        value: [f32; 3],
    },
    Drag {
        coefficient: f32,
    },
    Wave {
        amplitude: [f32; 3],
        frequency_hz: [f32; 3],
    },
    ColorOverLife {
        keys: Vec<ColorKey>,
    },
    SizeOverLife {
        keys: Vec<SizeKey>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Emitter {
    pub id: String,
    #[serde(default)]
    pub seed: u32,
    #[serde(default)]
    pub delay_ticks: u32,
    #[serde(default)]
    pub rate_per_second: f32,
    #[serde(default)]
    pub burst: u32,
    #[serde(default = "emitter_budget")]
    pub max_particles: u32,
    #[serde(default)]
    pub shape: Shape,
    pub direction: [f32; 3],
    pub spread_degrees: f32,
    pub speed: [f32; 2],
    pub lifetime_ticks: [u32; 2],
    pub size: [f32; 2],
    #[serde(default)]
    pub rotation_radians: [f32; 2],
    #[serde(default)]
    pub angular_velocity: [f32; 2],
    #[serde(default = "white")]
    pub color_min: [f32; 4],
    #[serde(default = "white")]
    pub color_max: [f32; 4],
    #[serde(default = "one")]
    pub aspect: f32,
    #[serde(default = "one")]
    pub softness: f32,
    #[serde(default)]
    pub align_velocity: bool,
    #[serde(default = "default_material")]
    pub material: Material,
    #[serde(default)]
    pub modules: Vec<Module>,
}
fn finite_range(v: &[f32], lo: f32, hi: f32) -> bool {
    v.iter().all(|v| v.is_finite() && (lo..=hi).contains(v))
}
fn ordered(v: [f32; 2], lo: f32, hi: f32) -> bool {
    finite_range(&v, lo, hi) && v[0] <= v[1]
}
fn color(v: [f32; 4]) -> bool {
    finite_range(&v[..3], 0., 16.) && finite_range(&v[3..], 0., 1.)
}
fn times(v: impl Iterator<Item = f32>) -> bool {
    let v: Vec<_> = v.collect();
    (2..=16).contains(&v.len())
        && v[0] == 0.
        && v[v.len() - 1] == 1.
        && v.windows(2).all(|w| w[0].is_finite() && w[0] < w[1])
}
impl Document {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || !(1..=262144).contains(&self.max_particles)
            || self.sources.len() > 128
            || self.instances.len() > 256
        {
            return Err(invalid(
                "Particles1 requires budget1..262144, <=128 sources and <=256 placements",
            ));
        }
        let mut ids = BTreeSet::new();
        for s in &self.sources {
            s.validate()?;
            if !ids.insert(&s.id) {
                return Err(invalid("Duplicate particle source ID"));
            }
        }
        let mut placements = BTreeSet::new();
        for p in &self.instances {
            id(&p.id)?;
            if !placements.insert(&p.id)
                || !ids.contains(&p.source)
                || !finite_range(&p.rotation, -1., 1.)
                || glam::Quat::from_array(p.rotation).length() < 0.01
                || !finite_range(&[p.scale], 0.001, 1000.)
                || !color(p.tint)
            {
                return Err(invalid(
                    "Invalid particle placement, source, rotation, scale or tint",
                ));
            }
        }
        Ok(())
    }
}
impl Source {
    pub fn validate(&self) -> Result<()> {
        id(&self.id)?;
        if !(1..=2).contains(&self.version)
            || (self.version == 1
                && (self.fire.is_some()
                    || self
                        .emitters
                        .iter()
                        .any(|e| e.modules.iter().any(|m| matches!(m, Module::Offset { .. })))))
            || !(1..=36000).contains(&self.duration_ticks)
            || self.emitters.is_empty()
            || self.emitters.len() > 32
        {
            return Err(invalid(
                "Effect1 needs period1..36000ticks and1..32emitters",
            ));
        }
        let mut ids = BTreeSet::new();
        for e in &self.emitters {
            id(&e.id)?;
            e.material.validate()?;
            if !ids.insert(&e.id)
                || e.delay_ticks >= self.duration_ticks
                || !finite_range(&[e.rate_per_second], 0., 100000.)
                || (e.rate_per_second > 0. && e.rate_per_second < 0.001)
                || e.burst > 65536
                || !(1..=65536).contains(&e.max_particles)
                || !finite_range(&e.direction, -1., 1.)
                || glam::Vec3::from(e.direction).length() < 0.01
                || !finite_range(&[e.spread_degrees], 0., 180.)
                || !ordered(e.speed, 0., 50000.)
                || e.lifetime_ticks[0] == 0
                || e.lifetime_ticks[0] > e.lifetime_ticks[1]
                || e.lifetime_ticks[1] > 3600
                || !ordered(e.size, 0.001, 100.)
                || !ordered(e.rotation_radians, -100., 100.)
                || !ordered(e.angular_velocity, -100., 100.)
                || !color(e.color_min)
                || !color(e.color_max)
                || e.color_min.iter().zip(e.color_max).any(|(a, b)| *a > b)
                || !finite_range(&[e.aspect], 0.001, 100.)
                || !finite_range(&[e.softness], 0., 1.)
                || e.modules.len() > 16
                || !e.material.transparent()
                || e.material
                    .surface
                    .as_ref()
                    .is_some_and(|s| s.glass.is_some())
            {
                return Err(invalid(
                    "Invalid emitter birth/sprite/budget parameters; particles require blend/additive, no glass",
                ));
            }
            match e.shape {
                Shape::Point => {}
                Shape::Sphere { radius } | Shape::Disc { radius } => {
                    if !finite_range(&[radius], 0., 10000.) {
                        return Err(invalid("Spawn radius requires0..10000m"));
                    }
                }
                Shape::Box { half_extents } => {
                    if !finite_range(&half_extents, 0., 10000.) {
                        return Err(invalid("Spawn half extents require0..10000m"));
                    }
                }
            }
            let mut extent = f64::from(e.size[1]) * f64::from(e.aspect.max(1.));
            for m in &e.modules {
                let valid = match m {
                    Module::Offset { value } => finite_range(value, -10000., 10000.),
                    Module::Acceleration { value } => finite_range(value, -10000., 10000.),
                    Module::Drag { coefficient } => finite_range(&[*coefficient], 0., 64.),
                    Module::Wave {
                        amplitude,
                        frequency_hz,
                    } => {
                        finite_range(amplitude, -100., 100.) && finite_range(frequency_hz, 0., 16.)
                    }
                    Module::ColorOverLife { keys } => {
                        times(keys.iter().map(|k| k.time)) && keys.iter().all(|k| color(k.value))
                    }
                    Module::SizeOverLife { keys } => {
                        times(keys.iter().map(|k| k.time))
                            && keys.iter().all(|k| finite_range(&[k.value], 0., 100.))
                    }
                };
                if !valid {
                    return Err(invalid(
                        "Invalid particle module or strictly ordered0..1 curve endpoints",
                    ));
                }
                if let Module::SizeOverLife { keys } = m {
                    extent *= f64::from(keys.iter().map(|k| k.value).fold(0., f32::max));
                }
            }
            if extent > 1_000_000. {
                return Err(invalid(
                    "Combined particle size/aspect/curves exceed the conservative1,000km source extent",
                ));
            }
        }
        if let Some(recipe) = &self.fire {
            recipe.settings.validate()?;
            if !recipe.matches(self) {
                return Err(invalid(
                    "Fire recipe and derived emitters differ; apply fire settings or clear the recipe before manual emitter editing",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Inspect,
    DetachFire {
        id: String,
    },
    Fire {
        id: String,
        settings: fire::Settings,
    },
    Preset {
        preset: Preset,
        id: String,
    },
    Replace {
        document: Arc<Document>,
    },
    PutSource {
        source: Arc<Source>,
    },
    RemoveSource {
        id: String,
    },
    PutInstance {
        instance: Placement,
    },
    RemoveInstance {
        id: String,
    },
    Preview {
        tick: Option<u64>,
    },
    Open {
        open: bool,
    },
    SaveSource {
        id: String,
        path: String,
    },
    LoadSource {
        path: String,
    },
}
impl Default for Emitter {
    fn default() -> Self {
        Self {
            id: "emitter".into(),
            seed: 0,
            delay_ticks: 0,
            rate_per_second: 30.,
            burst: 0,
            max_particles: 4096,
            shape: Shape::Point,
            direction: [0., 1., 0.],
            spread_degrees: 20.,
            speed: [1., 2.],
            lifetime_ticks: [60, 120],
            size: [0.1, 0.2],
            rotation_radians: [0.; 2],
            angular_velocity: [0.; 2],
            color_min: [1.; 4],
            color_max: [1.; 4],
            aspect: 1.,
            softness: 1.,
            align_velocity: false,
            material: default_material(),
            modules: vec![],
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Sparks,
    Smoke,
}
impl Source {
    pub fn preset(preset: Preset) -> Self {
        let mut e = Emitter::default();
        match preset {
            Preset::Sparks => {
                e.id = "embers".into();
                e.rate_per_second = 70.;
                e.burst = 40;
                e.speed = [3., 8.];
                e.spread_degrees = 48.;
                e.lifetime_ticks = [30, 110];
                e.size = [0.08, 0.2];
                e.aspect = 0.16;
                e.softness = 0.4;
                e.align_velocity = true;
                e.material = Material {
                    surface: Some(Surface {
                        mode: BlendMode::Additive,
                        emissive: [4.; 3],
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                e.modules = vec![
                    Module::Acceleration {
                        value: [0., -6., 0.],
                    },
                    Module::Drag { coefficient: 0.25 },
                    Module::ColorOverLife {
                        keys: vec![
                            ColorKey {
                                time: 0.,
                                value: [1., 0.75, 0.2, 1.],
                            },
                            ColorKey {
                                time: 0.5,
                                value: [1., 0.18, 0.02, 0.8],
                            },
                            ColorKey {
                                time: 1.,
                                value: [0.3, 0.01, 0., 0.],
                            },
                        ],
                    },
                ];
            }
            Preset::Smoke => {
                e.id = "plume".into();
                e.rate_per_second = 30.;
                e.speed = [0.7, 1.2];
                e.spread_degrees = 15.;
                e.lifetime_ticks = [140, 300];
                e.size = [0.3, 0.55];
                e.softness = 1.;
                e.shape = Shape::Disc { radius: 0.2 };
                e.rotation_radians = [-std::f32::consts::PI, std::f32::consts::PI];
                e.angular_velocity = [-0.4, 0.4];
                e.modules = vec![
                    Module::Acceleration {
                        value: [0.08, 0.15, 0.],
                    },
                    Module::Drag { coefficient: 0.15 },
                    Module::Wave {
                        amplitude: [0.18, 0.05, 0.13],
                        frequency_hz: [0.3, 0.15, 0.23],
                    },
                    Module::SizeOverLife {
                        keys: vec![
                            SizeKey {
                                time: 0.,
                                value: 0.5,
                            },
                            SizeKey {
                                time: 1.,
                                value: 3.5,
                            },
                        ],
                    },
                    Module::ColorOverLife {
                        keys: vec![
                            ColorKey {
                                time: 0.,
                                value: [0.18, 0.2, 0.22, 0.],
                            },
                            ColorKey {
                                time: 0.15,
                                value: [0.22, 0.25, 0.28, 0.5],
                            },
                            ColorKey {
                                time: 1.,
                                value: [0.4, 0.43, 0.46, 0.],
                            },
                        ],
                    },
                ];
            }
        }
        Self {
            version: VERSION,
            id: match preset {
                Preset::Sparks => "sparks",
                Preset::Smoke => "smoke",
            }
            .into(),
            seed: 7,
            duration_ticks: 180,
            looping: true,
            emitters: vec![e],
            fire: None,
        }
    }
}
