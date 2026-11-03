use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SCENE_VERSION: u32 = 116;
pub const FIXED_DT: f32 = 1.0 / 60.0;
pub const MAX_ENTITIES: usize = 4096;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    pub code: &'static str,
    pub message: String,
}
pub type Result<T> = std::result::Result<T, Error>;
pub fn invalid(message: impl Into<String>) -> Error {
    Error {
        code: "invalid_argument",
        message: message.into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Transform {
    pub position: [f32; 3],
    /// Axis-aligned full dimensions in metres. Rotation is a future schema extension.
    pub size: [f32; 3],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BodyKind {
    Fixed,
    Dynamic,
    Player,
    Npc,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<crate::npc_presentation::Config>,
    pub id: String,
    pub name: String,
    pub transform: Transform,
    #[serde(default, skip_serializing_if = "crate::geometry::Geometry::is_box")]
    pub geometry: crate::geometry::Geometry,
    /// Linear RGB, each channel in [0,1].
    pub color: [f32; 3],
    /// None produces a visual-only box. Box meshes are the first supported primitive.
    pub body: Option<BodyKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npc: Option<NpcConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abilities: Option<crate::abilities::Loadout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vitality: Option<crate::vitality::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction: Option<String>,
    /// None inherits body presence; explicit values decouple sight from collision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks_vision: Option<bool>,
    /// Multiplier of audible radius across this box; omitted means transparent (1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound_transmission: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footsteps: Option<FootstepConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locomotion: Option<Locomotion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_controller: Option<crate::character_controller::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<crate::trigger::Config>,
    /// None means visible. Visibility does not change physics, sight or hearing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MovementMode {
    #[default]
    Walk,
    Sneak,
    Run,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MovementProfile {
    pub speed: f32,
    pub noise_multiplier: f32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Locomotion {
    pub walk: MovementProfile,
    pub sneak: MovementProfile,
    pub run: MovementProfile,
}
impl Default for Locomotion {
    fn default() -> Self {
        Self {
            walk: MovementProfile {
                speed: 4.,
                noise_multiplier: 1.,
            },
            sneak: MovementProfile {
                speed: 1.5,
                noise_multiplier: 0.2,
            },
            run: MovementProfile {
                speed: 6.,
                noise_multiplier: 1.8,
            },
        }
    }
}
impl Locomotion {
    pub fn profile(self, mode: MovementMode) -> MovementProfile {
        match mode {
            MovementMode::Walk => self.walk,
            MovementMode::Sneak => self.sneak,
            MovementMode::Run => self.run,
        }
    }
    fn validate(self) -> Result<()> {
        for p in [self.walk, self.sneak, self.run] {
            if !p.speed.is_finite()
                || !(0.1..=1000.).contains(&p.speed)
                || !p.noise_multiplier.is_finite()
                || !(0. ..=4.).contains(&p.noise_multiplier)
            {
                return Err(invalid(
                    "Locomotion profiles require speed [0.1,1000]m/s and noise_multiplier [0,4]",
                ));
            }
        }
        Ok(())
    }
}

/// Optional point-noise emitter driven by supported physical locomotion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FootstepConfig {
    pub surface: String,
    pub stride: f32,
    pub radius: f32,
    pub min_speed: f32,
}

/// Pursuit on one flat, fixed box surface. Distances are in metres.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NpcConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routine: Option<crate::npc_routines::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dialogue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooperation: Option<crate::npc_cooperation::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circulation: Option<crate::npc_circulation::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mind: Option<crate::npc_mind::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub communication: Option<crate::communication::Config>,
    /// Explicit visual objective; absent uses optional targeting, otherwise routine/acoustic cues.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targeting: Option<crate::targeting::Policy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recognition: Option<crate::recognition::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound_attention: Option<crate::sound_attention::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objectives: Option<Vec<crate::objectives::Objective>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub territory: Option<crate::territory::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation_recovery: Option<crate::nav_recovery::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passage_priority: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobility: Option<crate::mobility::Capabilities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unreachable_policy: Option<crate::mobility::UnreachablePolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_connections: Option<crate::mobility::LocalConnections>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flight: Option<crate::flight::Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_vision: Option<crate::perception::SpatialVision>,
    pub surface: String,
    pub speed: f32,
    pub stop_distance: f32,
    pub enabled: bool,
    /// Omitted/null resolves to perceptive for v3+, omniscient for legacy v2 scenes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awareness: Option<Awareness>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patrol: Option<crate::patrol::Config>,
}

/// Optional listener; gameplay noise is separate from device audio.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Hearing {
    pub range: f32,
}

/// Bounded procedural ring around the last observation, not a hidden-target query.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AreaSearch {
    pub radius: f32,
    pub points: u8,
    pub scan_ticks: u32,
    pub seed: u32,
}
impl AreaSearch {
    fn validate(&self) -> Result<()> {
        if !self.radius.is_finite()
            || !(0.5..=20.).contains(&self.radius)
            || !(1..=8).contains(&self.points)
            || !(1..=3600).contains(&self.scan_ticks)
        {
            return Err(invalid(
                "Area search requires radius [0.5,20]m, points [1,8], scan_ticks [1,3600]",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Awareness {
    Omniscient,
    Perceptive {
        vision_range: f32,
        fov_degrees: f32,
        initial_facing_degrees: f32,
        memory_ticks: Option<u32>,
        search_ticks: Option<u32>,
        search_turn_degrees_per_second: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_investigation_distance: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        area_search: Option<AreaSearch>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hearing: Option<Hearing>,
    },
}
impl Default for Awareness {
    fn default() -> Self {
        Self::Perceptive {
            vision_range: 12.,
            fov_degrees: 120.,
            initial_facing_degrees: 0.,
            memory_ticks: Some(600),
            search_ticks: Some(180),
            search_turn_degrees_per_second: 90.,
            max_investigation_distance: None,
            area_search: None,
            hearing: None,
        }
    }
}
impl Awareness {
    fn validate(&self) -> Result<()> {
        if let Self::Perceptive {
            vision_range,
            fov_degrees,
            initial_facing_degrees,
            memory_ticks,
            search_ticks,
            search_turn_degrees_per_second,
            max_investigation_distance,
            ..
        } = self
            && (!vision_range.is_finite()
                || !(0.1..=100.).contains(vision_range)
                || !fov_degrees.is_finite()
                || !(1. ..=360.).contains(fov_degrees)
                || !initial_facing_degrees.is_finite()
                || !(-180. ..=180.).contains(initial_facing_degrees)
                || memory_ticks.is_some_and(|t| !(1..=36000).contains(&t))
                || search_ticks.is_some_and(|t| !(1..=3600).contains(&t))
                || max_investigation_distance
                    .is_some_and(|d| !d.is_finite() || !(0.05..=10000.).contains(&d))
                || (memory_ticks.is_none()
                    && search_ticks.is_none()
                    && max_investigation_distance.is_none())
                || !search_turn_degrees_per_second.is_finite()
                || !(0. ..=360.).contains(search_turn_degrees_per_second))
        {
            return Err(invalid(
                "Perception requires range [0.1,100]m, FOV [1,360], facing [-180,180], optional memory [1,36000] ticks, search [1,3600] ticks, travel [0.05,10000]m; at least one budget; turn speed [0,360] degrees/s",
            ));
        }
        if let Self::Perceptive {
            area_search: Some(search),
            ..
        } = self
        {
            search.validate()?;
        }
        if let Self::Perceptive {
            hearing: Some(hearing),
            ..
        } = self
            && (!hearing.range.is_finite() || !(0.1..=100.).contains(&hearing.range))
        {
            return Err(invalid("Hearing range must be in [0.1,100] metres"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov_degrees: f32,
}
impl Camera {
    pub fn validate(&self) -> Result<()> {
        finite(&self.position, "camera position")?;
        finite(&self.target, "camera target")?;
        let forward = Vec3::from(self.target) - Vec3::from(self.position);
        if forward.length() < 0.01 || forward.normalize().cross(Vec3::Y).length() < 0.01 {
            return Err(invalid(
                "Camera direction must be nonzero and not parallel to world up",
            ));
        }
        if !self.fov_degrees.is_finite() || !(10.0..=120.0).contains(&self.fov_degrees) {
            return Err(invalid("Camera FOV must be in [10,120] degrees"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Light {
    /// Direction in which light travels, not necessarily normalized.
    pub direction: [f32; 3],
    pub ambient: f32,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub direct_intensity: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<crate::environment::Environment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atmosphere: Option<crate::atmosphere::Atmosphere>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<DirectionalShadow>,
}

impl Light {
    pub fn validate(&self) -> Result<()> {
        finite(&self.direction, "light direction")?;
        if !self.direct_intensity.is_finite() || !(0. ..=4.).contains(&self.direct_intensity) {
            return Err(invalid("Direct light intensity requires finite0..4"));
        }
        if let Some(atmosphere) = &self.atmosphere {
            atmosphere.validate()?;
        }
        if let Some(environment) = &self.environment {
            environment.validate()?;
        }
        if let Some(shadow) = &self.shadow {
            finite(&shadow.center, "shadow center")?;
            if !shadow.extent.is_finite()
                || !(0.25..=256.).contains(&shadow.extent)
                || !matches!(shadow.resolution, 512 | 1024 | 2048)
                || !shadow.bias.is_finite()
                || !(0.0..=0.01).contains(&shadow.bias)
            {
                return Err(invalid(
                    "Shadow requires extent0.25..256m, resolution512/1024/2048 and bias0..0.01",
                ));
            }
        }
        if Vec3::from(self.direction).length() < 0.01
            || !self.ambient.is_finite()
            || !(0.0..=1.0).contains(&self.ambient)
        {
            return Err(invalid(
                "Light needs a nonzero direction and ambient in [0,1]",
            ));
        }
        Ok(())
    }
}

fn one() -> f32 {
    1.
}
fn is_one(value: &f32) -> bool {
    *value == 1.
}
fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DirectionalShadow {
    pub center: [f32; 3],
    /// Orthographic half-width/half-height in world metres.
    pub extent: f32,
    pub resolution: u32,
    /// Comparison bias in normalized light depth.
    pub bias: f32,
    /// Correct PCF comparison depth using the local geometric receiver plane.
    #[serde(default, skip_serializing_if = "is_false")]
    pub receiver_plane_bias: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoomRecipe {
    pub width: f32,
    pub depth: f32,
    pub seed: u32,
    pub obstacles: u32,
}
impl Default for RoomRecipe {
    fn default() -> Self {
        Self {
            width: 16.0,
            depth: 14.0,
            seed: 42,
            obstacles: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    #[serde(default, skip_serializing_if = "crate::wind::document_empty")]
    pub wind: std::sync::Arc<crate::wind::Document>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub botany: Vec<crate::botany::Recipe>,
    /// Editor recipes link separately stored sources to their baked meshes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characters: Vec<crate::character_editor::Binding>,
    /// Explicit static physical surfaces; geometry and transforms come from meshes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collision_meshes: Vec<String>,
    #[serde(
        default = "crate::templates::default_capacity",
        skip_serializing_if = "crate::templates::is_default_capacity"
    )]
    pub npc_capacity: u16,
    #[serde(
        default,
        skip_serializing_if = "crate::npc_presentation::Audio::is_empty"
    )]
    pub avatar_audio: crate::npc_presentation::Audio,
    #[serde(
        default,
        skip_serializing_if = "crate::interactions::Document::is_empty"
    )]
    pub interactions: crate::interactions::Document,
    #[serde(default, skip_serializing_if = "crate::plants::Document::is_empty")]
    pub plants: crate::plants::Document,
    #[serde(default, skip_serializing_if = "crate::meshes::Document::is_empty")]
    pub meshes: crate::meshes::Document,
    #[serde(default, skip_serializing_if = "crate::particles::document_empty")]
    pub particles: std::sync::Arc<crate::particles::Document>,
    #[serde(default, skip_serializing_if = "crate::distant::is_empty")]
    pub distant_meshes: std::sync::Arc<Vec<crate::distant::Pack>>,
    pub version: u32,
    pub name: String,
    pub camera: Camera,
    pub light: Light,
    pub recipe: Option<RoomRecipe>,
    pub entities: Vec<Entity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub npc_templates: Vec<Entity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub npc_generators: Vec<crate::generators::Config>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub faction_relations: Vec<crate::targeting::Relation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passages: Vec<crate::passages::Config>,
    #[serde(default)]
    pub spatial_network: crate::mobility::Network,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation_generation: Option<crate::npc_navigation_generation::Recipe>,
    #[serde(default, skip_serializing_if = "crate::abilities::Catalog::is_empty")]
    pub ability_catalog: crate::abilities::Catalog,
}

fn finite(values: &[f32], label: &str) -> Result<()> {
    if values.iter().any(|v| !v.is_finite() || v.abs() > 10_000.0) {
        return Err(invalid(format!(
            "{label}: values must be finite and within ±10000"
        )));
    }
    Ok(())
}

impl Scene {
    /// Called only after validation, before the scene enters a running engine.
    pub fn normalize(&mut self) {
        crate::attributes::migrate(self);
        for e in self
            .entities
            .iter_mut()
            .chain(self.npc_templates.iter_mut())
        {
            if let Some(n) = &mut e.npc {
                n.awareness.get_or_insert_with(|| {
                    if self.version == 2 {
                        Awareness::Omniscient
                    } else {
                        Awareness::default()
                    }
                });
            }
        }
        // Compiled substitutes are derived data. Undo retains the old Scene; the
        // current document does not carry an unusable packet after a source edit.
        if self
            .distant_meshes
            .iter()
            .any(|p| !self.meshes.assets.iter().any(|a| p.matches(a)))
        {
            self.distant_meshes = std::sync::Arc::new(
                self.distant_meshes
                    .iter()
                    .filter(|p| self.meshes.assets.iter().any(|a| p.matches(a)))
                    .cloned()
                    .collect(),
            );
        }
        self.version = SCENE_VERSION;
        if let Some(mut generation) = self.navigation_generation.take() {
            generation.rebuild(self);
            self.navigation_generation = Some(generation);
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.npc_capacity == 0
            || self.npc_capacity as usize > crate::templates::MAX_NPCS
            || self.version < 96 && self.npc_capacity != 8
        {
            return Err(invalid(
                "NPC capacity1..128 requires scene96; legacy default8",
            ));
        }
        self.validate_inner(true)?;
        if self.version < 94 && !self.interactions.is_empty() {
            return Err(invalid("Interactions require scene94"));
        }
        self.interactions.validate(self)?;
        crate::npc_presentation::validate(self)?;
        // Visual assets are independent of per-template entity probes: validate once.
        self.meshes.validate()?;
        self.particles.validate()?;
        if self.version < crate::particles::SCENE_VERSION && !self.particles.is_empty() {
            return Err(invalid("Particle effects require scene113"));
        }
        if self.version < 107 && !self.meshes.objects.is_empty() {
            return Err(invalid("Visual object membership requires scene107"));
        }
        crate::character_editor::validate(self)?;
        crate::botany::validate_scene(self)?;
        self.wind.validate_scene(self)?;
        if self.version < 105
            && self.meshes.assets.iter().any(|a| {
                a.rig.as_ref().is_some_and(|r| {
                    r.bones.len() > 32 || r.weights.iter().any(|w| w.extra.is_some())
                })
            })
        {
            return Err(invalid(
                "Extended rigs and skin influences require scene105",
            ));
        }
        crate::surfaces::validate(self)?;
        if self.version < 103 && self.meshes.exceeds_legacy_limits() {
            return Err(invalid(
                "Meshes beyond legacy resource ceilings require scene version103",
            ));
        }

        if self.version < 102
            && self
                .light
                .atmosphere
                .is_some_and(|a| a.edge_softness.is_some())
        {
            return Err(invalid(
                "Atmospheric edge softness requires scene version102",
            ));
        }
        if self.version < 101
            && (self.light.atmosphere.is_some_and(|a| a.layers.is_some())
                || self
                    .distant_meshes
                    .iter()
                    .any(|p| p.data.starts_with(b"ATM3")))
        {
            return Err(invalid("Atmospheric layers/ATM3 require scene version101"));
        }
        if self.version < 100 && !self.distant_meshes.is_empty() {
            return Err(invalid("Compiled distant meshes require scene version100"));
        }
        crate::distant::validate(&self.distant_meshes, &self.meshes)?;
        if self.version < 72 && !self.plants.connections.is_empty() {
            return Err(invalid("Plant connections require scene version72"));
        }
        if self.version < 100 && self.light.atmosphere.is_some() {
            return Err(invalid("Atmosphere requires scene version100"));
        }
        if self.version < 76
            && (self.light.environment.is_some() || self.light.direct_intensity != 1.)
        {
            return Err(invalid(
                "Environment/direct intensity require scene version76",
            ));
        }
        if self.version < crate::materials::marble::SCENE_VERSION {
            let marble = |a: &crate::meshes::Asset| {
                a.material
                    .procedural
                    .as_ref()
                    .is_some_and(|p| p.recipe.marble.is_some())
            };
            if self.meshes.assets.iter().any(marble)
                || self.plants.entries.iter().any(|e| {
                    marble(&e.source)
                        || e.assembly.as_ref().is_some_and(|a| {
                            marble(&a.context) || a.context_grafts.iter().any(|g| marble(&g.source))
                        })
                })
                || self
                    .particles
                    .sources
                    .iter()
                    .flat_map(|s| &s.emitters)
                    .any(|s| {
                        s.material
                            .procedural
                            .as_ref()
                            .is_some_and(|p| p.recipe.marble.is_some())
                    })
            {
                return Err(invalid("Marble programs require scene115"));
            }
        }
        if self.version < 112 {
            let procedural = |a: &crate::meshes::Asset| a.material.procedural.is_some();
            if self.meshes.assets.iter().any(procedural)
                || self.plants.entries.iter().any(|e| {
                    procedural(&e.source)
                        || e.assembly.as_ref().is_some_and(|a| {
                            procedural(&a.context)
                                || a.context_grafts.iter().any(|g| procedural(&g.source))
                        })
                })
            {
                return Err(invalid(
                    "Procedural material recipes require scene version112",
                ));
            }
        }
        if self.version < 111
            && self
                .light
                .shadow
                .as_ref()
                .is_some_and(|s| s.receiver_plane_bias)
        {
            return Err(invalid(
                "Receiver-plane shadow bias requires scene version111",
            ));
        }
        if self.version < 110 {
            let surface = |a: &crate::meshes::Asset| a.material.surface.is_some();
            if self.meshes.assets.iter().any(surface)
                || self.plants.entries.iter().any(|e| {
                    surface(&e.source)
                        || e.assembly.as_ref().is_some_and(|a| {
                            surface(&a.context)
                                || a.context_grafts.iter().any(|g| surface(&g.source))
                        })
                })
            {
                return Err(invalid("Surface composition requires scene version110"));
            }
        }
        let metallic =
            |a: &crate::meshes::Asset| a.material.reflectance.is_some_and(|r| r.metallic > 0.);
        if self.version < 74
            && (self.meshes.assets.iter().any(metallic)
                || self.plants.entries.iter().any(|e| {
                    metallic(&e.source)
                        || e.assembly.as_ref().is_some_and(|a| {
                            metallic(&a.context)
                                || a.context_grafts.iter().any(|g| metallic(&g.source))
                        })
                }))
        {
            return Err(invalid("Metallic response requires scene version74"));
        }
        let layered = |a: &crate::meshes::Asset| a.material.secondary_pattern.is_some();
        if self.version < 73
            && (self.meshes.assets.iter().any(layered)
                || self.plants.entries.iter().any(|e| {
                    layered(&e.source)
                        || e.assembly.as_ref().is_some_and(|a| {
                            layered(&a.context)
                                || a.context_grafts.iter().any(|g| layered(&g.source))
                        })
                }))
        {
            return Err(invalid(
                "Secondary material patterns require scene version73",
            ));
        }
        let new_detail = |a: &crate::meshes::Asset| {
            a.material
                .detail
                .as_ref()
                .is_some_and(|d| d.profile != crate::materials::Profile::Band || d.repeat.is_some())
        };
        if self.version < 71
            && (self.meshes.assets.iter().any(new_detail)
                || self.plants.entries.iter().any(|e| {
                    new_detail(&e.source)
                        || e.assembly.as_ref().is_some_and(|a| {
                            new_detail(&a.context)
                                || a.context_grafts.iter().any(|g| new_detail(&g.source))
                        })
                }))
        {
            return Err(invalid(
                "Detail profiles and repetition require scene version71",
            ));
        }
        let triplanar = |a: &crate::meshes::Asset| {
            a.material
                .detail
                .as_ref()
                .is_some_and(|d| d.triplanar.is_some())
        };
        if self.version < 70
            && (self.meshes.assets.iter().any(triplanar)
                || self.plants.entries.iter().any(|e| {
                    triplanar(&e.source)
                        || e.assembly.as_ref().is_some_and(|a| {
                            triplanar(&a.context)
                                || a.context_grafts.iter().any(|g| triplanar(&g.source))
                        })
                }))
        {
            return Err(invalid("Triplanar surface detail requires scene version70"));
        }
        if self.version < 70
            && self.plants.entries.iter().any(|e| {
                e.assembly.as_ref().is_some_and(|a| {
                    a.plant_material_frame.is_some()
                        || a.material_finish.is_some()
                        || a.context_grafts.iter().any(|g| {
                            g.source_material_frame.is_some() || g.material_finish.is_some()
                        })
                })
            })
        {
            return Err(invalid(
                "Plant material frames and finishing require scene version70",
            ));
        }
        self.plants.validate(&self.meshes)?;
        if self.version < 69
            && self.plants.entries.iter().any(|e| {
                e.assembly
                    .as_ref()
                    .is_some_and(|a| a.context_grafts.iter().any(|g| g.collar_surface.is_some()))
            })
        {
            return Err(invalid("Graft collar surfaces require scene version69"));
        }
        if self.version < 68
            && self.plants.entries.iter().any(|e| {
                e.assembly
                    .as_ref()
                    .is_some_and(|a| !a.context_grafts.is_empty())
            })
        {
            return Err(invalid("Context grafts require scene version68"));
        }
        if self.version < 67
            && (self
                .meshes
                .assets
                .iter()
                .chain(self.plants.entries.iter().map(|e| e.source.as_ref()))
                .chain(
                    self.plants
                        .entries
                        .iter()
                        .filter_map(|e| e.assembly.as_ref().map(|a| a.context.as_ref())),
                )
                .any(|a| a.geometry.positions.len() > 8192 || a.geometry.triangles.len() > 16384)
                || self
                    .meshes
                    .assets
                    .iter()
                    .map(|a| a.geometry.positions.len())
                    .sum::<usize>()
                    > 16384
                || self
                    .meshes
                    .assets
                    .iter()
                    .map(|a| a.geometry.triangles.len())
                    .sum::<usize>()
                    > 32768
                || self
                    .plants
                    .entries
                    .iter()
                    .map(|e| {
                        e.source.geometry.positions.len()
                            + e.assembly
                                .as_ref()
                                .map_or(0, |a| a.context.geometry.positions.len())
                    })
                    .sum::<usize>()
                    > 16384
                || self
                    .plants
                    .entries
                    .iter()
                    .map(|e| {
                        e.source.geometry.triangles.len()
                            + e.assembly
                                .as_ref()
                                .map_or(0, |a| a.context.geometry.triangles.len())
                    })
                    .sum::<usize>()
                    > 32768
                || self.plants.entries.iter().any(|e| {
                    e.assembly.as_ref().is_some_and(|a| {
                        !a.context_lofts.is_empty()
                            || a.context_surface.is_some()
                            || a.junction_surface.is_some()
                    })
                }))
        {
            return Err(invalid("Expanded mesh storage requires scene version67"));
        }
        if self.version < 66
            && self
                .meshes
                .assets
                .iter()
                .chain(self.plants.entries.iter().map(|e| e.source.as_ref()))
                .any(|a| a.geometry.positions.len() > 4096 || a.geometry.triangles.len() > 8192)
        {
            return Err(invalid("Extended mesh assets require scene version66"));
        }
        if self.version < 66 && self.plants.entries.iter().any(|e| e.assembly.is_some()) {
            return Err(invalid("Plant assemblies require scene version66"));
        }
        if self.version < 65
            && self
                .plants
                .entries
                .iter()
                .any(|e| e.reference.trajectory.is_none() && e.definition.trajectory.is_some())
        {
            return Err(invalid("Trajectory adoption requires scene version65"));
        }
        if self.version < 64
            && self
                .plants
                .entries
                .iter()
                .any(|e| e.definition.trajectory.is_some() || e.reference.trajectory.is_some())
        {
            return Err(invalid("Plant trajectories require scene version64"));
        }
        if self.version < 60
            && (self
                .meshes
                .assets
                .iter()
                .any(|a| a.material.reflectance.is_some())
                || self
                    .plants
                    .entries
                    .iter()
                    .any(|e| e.source.material.reflectance.is_some()))
        {
            return Err(invalid("Dielectric reflectance requires scene version60"));
        }
        if self.version < 59
            && (self
                .meshes
                .assets
                .iter()
                .any(|a| a.material.detail.is_some())
                || self
                    .plants
                    .entries
                    .iter()
                    .any(|e| e.source.material.detail.is_some()))
        {
            return Err(invalid("Surface detail requires scene version59"));
        }
        if self.version < 58 && self.meshes.instances.iter().any(|i| i.attachment.is_some()) {
            return Err(invalid("Mesh attachments require scene version58"));
        }
        if self.version < 57
            && self.plants.entries.iter().any(|e| {
                e.definition.requires_smooth_format() || e.reference.requires_smooth_format()
            })
        {
            return Err(invalid(
                "Smooth section interpolation requires scene version57",
            ));
        }
        if self.version < 56
            && self.plants.entries.iter().any(|e| {
                e.definition.requires_curved_format() || e.reference.requires_curved_format()
            })
        {
            return Err(invalid(
                "Curved and elliptical sections require scene version56",
            ));
        }
        if self.version < 65 {
            let draw = self.meshes.instances.iter().try_fold(0usize, |total, i| {
                self.meshes
                    .asset(&i.asset)
                    .map(|a| total + a.geometry.triangles.len())
            })?;
            if draw > 65536 {
                return Err(invalid(
                    "Drawing more than65536 mesh triangles requires scene version65",
                ));
            }
        }
        if self.version < 55 {
            let draw: usize = self
                .meshes
                .instances
                .iter()
                .map(|i| {
                    self.meshes
                        .asset(&i.asset)
                        .expect("validated mesh instance")
                        .geometry
                        .triangles
                        .len()
                })
                .sum();
            if draw > 32768 {
                return Err(invalid(
                    "Drawing more than32768 mesh triangles requires scene version55",
                ));
            }
        }
        if self.version < 54
            && (self.meshes.assets.iter().any(|a| {
                a.material != Default::default() || a.geometry.material_coordinates.is_some()
            }) || self.plants.entries.iter().any(|e| {
                e.source.material != Default::default()
                    || e.source.geometry.material_coordinates.is_some()
            }))
        {
            return Err(invalid("Procedural materials require scene version54"));
        }
        if self.version < 53 && self.light.shadow.is_some() {
            return Err(invalid("Directional shadows require scene version53"));
        }
        if self.version < 52
            && (self
                .meshes
                .assets
                .iter()
                .any(|a| a.shading != crate::meshes::Shading::Flat)
                || self
                    .plants
                    .entries
                    .iter()
                    .any(|e| e.source.shading != crate::meshes::Shading::Flat))
        {
            return Err(invalid("Smooth mesh shading requires scene version52"));
        }
        if self.version < 51
            && self
                .plants
                .entries
                .iter()
                .any(|e| e.definition.sections.is_some() || e.reference.sections.is_some())
        {
            return Err(invalid("Section profiles require scene version51"));
        }
        if self.version < 50
            && self.plants.entries.iter().any(|e| {
                e.definition.subdivision_levels != 0 || e.reference.subdivision_levels != 0
            })
        {
            return Err(invalid(
                "Plant subdivision sampling requires scene version50",
            ));
        }
        if self.version < 49 && !self.plants.is_empty() {
            return Err(invalid("Parametric plants require scene version49"));
        }
        if self.version < 48 && !self.meshes.is_empty() {
            return Err(invalid("Native meshes require scene version48"));
        }
        crate::templates::validate(self)?;
        crate::generators::validate(self)
    }
    pub(crate) fn validate_inner(&self, population_limit: bool) -> Result<()> {
        self.spatial_network.validate(self)?;
        if let Some(recipe) = &self.navigation_generation {
            recipe.validate(self)?;
        }
        if !(1..=SCENE_VERSION).contains(&self.version) {
            return Err(Error {
                code: "unsupported_version",
                message: format!(
                    "Scene version {} is unsupported; expected {SCENE_VERSION}",
                    self.version
                ),
            });
        }
        crate::passages::validate(self)?;
        crate::abilities::validate(self)?;
        crate::vitality::validate(self)?;
        if self.name.is_empty() || self.name.len() > 120 {
            return Err(invalid("Scene name must contain 1–120 bytes"));
        }
        if population_limit && self.entities.len() > MAX_ENTITIES {
            return Err(invalid("Scene exceeds 4096 entities"));
        }
        if self.version < 10
            && self
                .entities
                .iter()
                .any(|e| e.trigger.is_some() || e.visible.is_some())
        {
            return Err(invalid(
                "Triggers and explicit visibility require scene version 10",
            ));
        }
        if self.entities.iter().filter(|e| e.trigger.is_some()).count()
            > crate::trigger::MAX_VOLUMES
        {
            return Err(invalid("At most 32 trigger volumes are supported"));
        }
        if self.version < 9
            && self
                .entities
                .iter()
                .any(|e| e.npc.as_ref().is_some_and(|n| n.patrol.is_some()))
        {
            return Err(invalid("Patrol routines require scene version 9"));
        }
        if self.version < 8 && self.entities.iter().any(|e| e.locomotion.is_some()) {
            return Err(invalid("Locomotion profiles require scene version 8"));
        }
        if self.version < 7 && self.entities.iter().any(|e| e.footsteps.is_some()) {
            return Err(invalid("Footsteps require scene version 7"));
        }
        if self.version < 6
            && self.entities.iter().any(|e| {
                e.sound_transmission.is_some()
                    || matches!(
                        e.npc.as_ref().and_then(|n| n.awareness.as_ref()),
                        Some(Awareness::Perceptive {
                            hearing: Some(_),
                            ..
                        })
                    )
            })
        {
            return Err(invalid(
                "Hearing and sound_transmission require scene version 6",
            ));
        }
        if self.version < 5
            && self.entities.iter().any(|e| {
                matches!(
                    e.npc.as_ref().and_then(|n| n.awareness.as_ref()),
                    Some(Awareness::Perceptive {
                        area_search: Some(_),
                        ..
                    })
                )
            })
        {
            return Err(invalid("Area search requires scene version 5"));
        }
        if self.version < 4 && self.entities.iter().any(|e| {
            matches!(e.npc.as_ref().and_then(|n| n.awareness.as_ref()),
                Some(Awareness::Perceptive { memory_ticks, search_ticks, max_investigation_distance, .. })
                if memory_ticks.is_none() || search_ticks.is_none() || max_investigation_distance.is_some())
        }) {
            return Err(invalid("Optional time budgets and travel budget require scene version 4"));
        }
        if self.version < 3
            && self.entities.iter().any(|e| {
                e.blocks_vision.is_some() || e.npc.as_ref().is_some_and(|n| n.awareness.is_some())
            })
        {
            return Err(invalid(
                "Awareness and blocks_vision require scene version 3",
            ));
        }
        if self.version == 1
            && self
                .entities
                .iter()
                .any(|e| e.npc.is_some() || e.body == Some(BodyKind::Npc))
        {
            return Err(invalid("NPCs require scene version 2"));
        }
        let mut ids = BTreeSet::new();
        let mut players = 0;
        for e in &self.entities {
            if e.id.is_empty()
                || e.id.len() > 80
                || !e
                    .id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
            {
                return Err(invalid(
                    "Entity id must contain 1–80 ASCII letters, digits, _ or -",
                ));
            }
            if !ids.insert(&e.id) {
                return Err(invalid(format!("Duplicate entity id: {}", e.id)));
            }
            if e.name.is_empty() || e.name.len() > 120 {
                return Err(invalid("Entity name must contain 1–120 bytes"));
            }
            finite(&e.transform.position, "position")?;
            finite(&e.transform.size, "size")?;
            e.geometry.validate(e, self.version)?;
            if e.transform.size.iter().any(|v| *v < 0.05 || *v > 1000.0) {
                return Err(invalid("Box dimensions must be in [0.05,1000] metres"));
            }
            if e.sound_transmission
                .is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
            {
                return Err(invalid("Sound transmission must be in [0,1]"));
            }
            finite(&e.color, "color")?;
            if e.color.iter().any(|v| !(0.0..=1.0).contains(v)) {
                return Err(invalid("Color channels must be in [0,1]"));
            }
            if e.body == Some(BodyKind::Player) {
                players += 1;
            }
        }
        if players > 1 && self.version < 77 {
            return Err(invalid("Multiple players require scene version77"));
        }
        let npcs = self
            .entities
            .iter()
            .filter(|e| e.body == Some(BodyKind::Npc))
            .count();
        if population_limit
            && self.version < 87
            && (self.entities.len() > 512 || players + npcs > 9)
        {
            return Err(invalid(
                "Larger actor/entity storage requires scene version87",
            ));
        }
        if population_limit && players > crate::targeting::MAX_PLAYERS {
            return Err(invalid(
                "At most1000 players are supported by actor storage",
            ));
        }
        if population_limit && players + npcs > crate::targeting::MAX_ACTORS {
            return Err(invalid(
                "Actor storage supports at most1024 players and NPCs combined",
            ));
        }
        if population_limit
            && self.entities.iter().filter(|e| e.npc.is_some()).count() > self.npc_capacity as usize
        {
            return Err(invalid("NPC count exceeds authored capacity"));
        }
        if self.version < 87
            && self.entities.iter().any(|e| e.npc.is_some())
            && self.entities.iter().filter(|e| e.body.is_some()).count() > 128
        {
            return Err(invalid(
                "NPC navigation supports at most 128 physical bodies",
            ));
        }
        crate::targeting::validate(self)?;
        for e in &self.entities {
            if (e.body == Some(BodyKind::Npc)) != e.npc.is_some() {
                return Err(invalid(
                    "Npc body and npc configuration must be present together",
                ));
            }
            if let Some(trigger) = &e.trigger {
                trigger.validate(e, self)?;
            }
            if let Some(controller) = e.character_controller {
                controller.validate(e, self.version)?;
            }
            if let Some(locomotion) = e.locomotion {
                if e.body != Some(BodyKind::Player) {
                    return Err(invalid(
                        "Locomotion profiles currently require a player body",
                    ));
                }
                locomotion.validate()?;
            }
            if let Some(f) = &e.footsteps {
                if !matches!(e.body, Some(BodyKind::Player | BodyKind::Npc))
                    || !f.stride.is_finite()
                    || !(0.2..=10.).contains(&f.stride)
                    || !f.radius.is_finite()
                    || !(0.1..=100.).contains(&f.radius)
                    || !f.min_speed.is_finite()
                    || !(0.01..=10.).contains(&f.min_speed)
                {
                    return Err(invalid(
                        "Footsteps require player/npc body, stride [0.2,10]m, radius [0.1,100]m and min_speed [0.01,10]m/s",
                    ));
                }
                if !self.entities.iter().any(|s| {
                    s.id == f.surface && s.body == Some(BodyKind::Fixed) && s.geometry.is_box()
                }) {
                    return Err(invalid("Footstep surface must reference a fixed body"));
                }
            }
            if let Some(n) = &e.npc {
                if let Some(routine) = &n.routine {
                    if self.version < 94 {
                        return Err(invalid("NPC routines require scene94"));
                    }
                    routine.validate(self, e)?;
                }
                if n.objectives.as_ref().is_some_and(|os| {
                    os.iter()
                        .any(|o| matches!(o.action, crate::objectives::Action::Routine))
                }) && (n.routine.is_none() || self.version < 94)
                {
                    return Err(invalid(
                        "Routine objective requires scene94 and routine configuration",
                    ));
                }
                if n.dialogue.as_ref().is_some_and(|id| {
                    self.version < 94 || !self.interactions.dialogues.iter().any(|d| d.id == *id)
                }) {
                    return Err(invalid(
                        "NPC dialogue requires scene94 and registered graph",
                    ));
                }
                if let Some(c) = n.cooperation {
                    if self.version < 93 {
                        return Err(invalid("NPC cooperation requires scene93"));
                    }
                    c.validate()?;
                }
                if self.version < 93
                    && n.objectives.as_ref().is_some_and(|os| {
                        os.iter()
                            .any(|o| matches!(o.action, crate::objectives::Action::Tactical { .. }))
                    })
                {
                    return Err(invalid("NPC tactics require scene93"));
                }
                if let Some(c) = n.circulation {
                    if self.version < 92 {
                        return Err(invalid("NPC circulation requires scene92"));
                    }
                    c.validate()?;
                }
                if let Some(mind) = n.mind {
                    if self.version < 89 {
                        return Err(invalid("NPC mind requires scene89"));
                    }
                    mind.validate()?;
                }
                if let Some(radio) = n.communication {
                    if self.version < 25 || matches!(n.awareness, Some(Awareness::Omniscient)) {
                        return Err(invalid(
                            "Radio requires scene version 25 and perceptive awareness",
                        ));
                    }
                    radio.validate()?;
                }
                if let Some(vision) = n.spatial_vision {
                    if self.version < 23 {
                        return Err(invalid("Spatial vision requires scene version 23"));
                    }
                    vision.validate()?;
                }
                if let Some(recovery) = n.navigation_recovery {
                    if self.version < 17 {
                        return Err(invalid("Navigation recovery requires scene version 17"));
                    }
                    recovery.validate(n.speed)?;
                }
                if let Some(profile) = n.flight {
                    if self.version < 22 {
                        return Err(invalid("Flight profile requires scene version 22"));
                    }
                    profile.validate(n.mobility)?;
                }
                if let Some(config) = n.local_connections {
                    if self.version < 21 {
                        return Err(invalid("Local connections require scene version 21"));
                    }
                    config.validate(n.mobility)?;
                }
                if let Some(policy) = n.unreachable_policy {
                    if self.version < 20 || n.mobility.is_none() {
                        return Err(invalid(
                            "Unreachable policy requires spatial mobility and scene version 20",
                        ));
                    }
                    policy.validate()?;
                }
                if let Some(mobility) = n.mobility {
                    mobility.validate(n)?;
                    if (self.spatial_network.nodes.is_empty()
                        && n.local_connections.is_none()
                        && self.navigation_generation.is_none())
                        || e.footsteps.is_some()
                    {
                        return Err(invalid(
                            "Spatial mobility requires a network or local connections; multi-surface footsteps are not yet supported",
                        ));
                    }
                }
                if n.objectives.as_ref().is_some_and(|os| {
                    os.iter()
                        .any(|o| matches!(o.action, crate::objectives::Action::Guard3d { .. }))
                }) && (n.mobility.is_none() || self.version < 19)
                {
                    return Err(invalid(
                        "guard_3d requires spatial mobility and scene version 19",
                    ));
                }
                if let Some(territory) = n.territory {
                    if self.version < 16 {
                        return Err(invalid("Territory requires scene version 16"));
                    }
                    territory.validate(n)?;
                }
                if let Some(objectives) = &n.objectives {
                    if self.version < 15 {
                        return Err(invalid("Objectives require scene version 15"));
                    }
                    crate::objectives::validate(objectives, n.patrol.is_some())?;
                }

                if let Some(attention) = n.sound_attention {
                    if self.version < 14 {
                        return Err(invalid("Sound attention requires scene version 14"));
                    }
                    if !matches!(
                        n.awareness.as_ref(),
                        Some(Awareness::Perceptive {
                            hearing: Some(_),
                            ..
                        })
                    ) {
                        return Err(invalid("Sound attention requires perceptive hearing"));
                    }
                    attention.validate()?;
                }
                if let Some(recognition) = n.recognition {
                    if self.version < 13 {
                        return Err(invalid("Recognition requires scene version 13"));
                    }
                    recognition.validate()?;
                }
                if let Some(patrol) = &n.patrol {
                    if n.mobility.is_some() && self.version < 24 {
                        return Err(invalid(
                            "Patrol with spatial mobility requires scene version 24",
                        ));
                    }
                    if patrol.route.is_spatial() && (self.version < 24 || n.mobility.is_none()) {
                        return Err(invalid(
                            "Spatial patrol requires scene version 24 and spatial mobility",
                        ));
                    }
                    patrol.validate()?;
                }
                if let Some(a) = &n.awareness {
                    a.validate()?;
                }
                if let Some(target_id) = &n.target {
                    let target = self
                        .entities
                        .iter()
                        .find(|t| t.id == *target_id)
                        .ok_or_else(|| invalid(format!("NPC {}: target does not exist", e.id)))?;
                    if target.id == e.id
                        || !matches!(target.body, Some(BodyKind::Player | BodyKind::Npc))
                    {
                        return Err(invalid("NPC target must be a distinct player or npc body"));
                    }
                    if self.version < 11 && target.body != Some(BodyKind::Player) {
                        return Err(invalid("NPC targets require scene version 11"));
                    }
                } else if self.version < 11 {
                    return Err(invalid("Optional NPC targets require scene version 11"));
                }
                let surface = self
                    .entities
                    .iter()
                    .find(|t| t.id == n.surface)
                    .ok_or_else(|| invalid(format!("NPC {}: surface does not exist", e.id)))?;
                if surface.body != Some(BodyKind::Fixed)
                    || !surface.geometry.is_box()
                    || surface.transform.size[0] > 64.
                    || surface.transform.size[2] > 64.
                    || e.transform.size[0] > 4.
                    || e.transform.size[2] > 4.
                {
                    return Err(invalid(
                        "NPC needs a fixed surface up to 64x64m and a body up to 4x4m",
                    ));
                }
                if !n.speed.is_finite()
                    || !(0.1..=10.).contains(&n.speed)
                    || !n.stop_distance.is_finite()
                    || !(0.2..=5.).contains(&n.stop_distance)
                {
                    return Err(invalid(
                        "NPC speed must be in [0.1,10]; stop_distance in [0.2,5]",
                    ));
                }
            }
        }
        self.camera.validate()?;
        self.light.validate()?;
        if let Some(r) = &self.recipe {
            r.validate()?;
        }
        Ok(())
    }
}

impl RoomRecipe {
    pub fn validate(&self) -> Result<()> {
        if !self.width.is_finite()
            || !self.depth.is_finite()
            || !(8.0..=80.0).contains(&self.width)
            || !(8.0..=80.0).contains(&self.depth)
            || self.obstacles > 12
        {
            return Err(invalid(
                "Room width/depth must be in [8,80]; obstacles in [0,12]",
            ));
        }
        Ok(())
    }
    pub fn generate(&self) -> Result<Scene> {
        self.validate()?;
        let mut entities = Vec::new();
        let mut add = |id: &str, p, size, color, body| {
            entities.push(Entity {
                avatar: None,
                geometry: Default::default(),
                id: id.into(),
                name: id.replace('_', " "),
                transform: Transform { position: p, size },
                color,
                body: Some(body),
                npc: None,
                abilities: None,
                vitality: None,
                faction: None,
                blocks_vision: None,
                sound_transmission: None,
                footsteps: None,
                locomotion: None,
                character_controller: None,
                trigger: None,
                visible: None,
            })
        };
        let (w, d) = (self.width, self.depth);
        add(
            "floor",
            [0., -0.25, 0.],
            [w + 0.5, 0.5, d + 0.5],
            [0.13, 0.19, 0.25],
            BodyKind::Fixed,
        );
        add(
            "wall_north",
            [0., 1., -d / 2.],
            [w, 2., 0.4],
            [0.2, 0.36, 0.42],
            BodyKind::Fixed,
        );
        add(
            "wall_south",
            [0., 0.35, d / 2.],
            [w, 0.7, 0.4],
            [0.2, 0.36, 0.42],
            BodyKind::Fixed,
        );
        add(
            "wall_west",
            [-w / 2., 1., 0.],
            [0.4, 2., d],
            [0.23, 0.31, 0.43],
            BodyKind::Fixed,
        );
        add(
            "wall_east",
            [w / 2., 0.35, 0.],
            [0.4, 0.7, d],
            [0.23, 0.31, 0.43],
            BodyKind::Fixed,
        );
        add(
            "player",
            [0., 0.85, d / 2. - 2.],
            [0.7, 1.6, 0.7],
            [1., 0.5, 0.08],
            BodyKind::Player,
        );
        // A versioned integer PRNG: deterministic across machines; no global RNG state.
        let mut state = self.seed as u64 + 1;
        for i in 0..self.obstacles {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let height = 0.7 + ((state >> 32) % 100) as f32 / 100.;
            let col = (i % 4) as f32;
            let row = (i / 4) as f32;
            let x = (col - 1.5) * (w - 4.) / 4.;
            let z = -d / 2. + 2. + row * (d - 5.) / 3.;
            add(
                &format!("obstacle_{i}"),
                [x, height / 2., z],
                [0.9, height, 0.9],
                [0.1, 0.55 + height * 0.1, 0.42],
                BodyKind::Fixed,
            );
        }
        let scene = Scene {
            collision_meshes: Vec::new(),
            npc_capacity: 8,
            avatar_audio: Default::default(),
            interactions: Default::default(),
            meshes: Default::default(),
            particles: Default::default(),
            characters: Vec::new(),
            botany: Vec::new(),
            wind: Default::default(),
            distant_meshes: Default::default(),
            plants: Default::default(),
            npc_templates: Vec::new(),
            npc_generators: Vec::new(),
            faction_relations: Vec::new(),
            passages: Vec::new(),
            spatial_network: Default::default(),
            navigation_generation: None,
            ability_catalog: Default::default(),
            version: SCENE_VERSION,
            name: "Procedural laboratory".into(),
            camera: Camera {
                position: [w * 0.85, w.max(d) * 0.95, d * 1.15],
                target: [0., 0., 0.],
                fov_degrees: 48.,
            },
            light: Light {
                environment: None,
                atmosphere: None,
                direct_intensity: 1.,
                shadow: None,
                direction: [-0.5, -1., -0.3],
                ambient: 0.32,
            },
            recipe: Some(self.clone()),
            entities,
        };
        scene.validate()?;
        Ok(scene)
    }
}
