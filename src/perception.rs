//! Sensor observations and bounded pursuit memory. Navigation only receives known goals.
use crate::{
    model::{AreaSearch, Awareness, Entity, FIXED_DT, NpcConfig, Scene},
    navigation::{NavGoal, NavState},
};
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Opt-in spherical field of view. Yaw/FOV/range reuse the awareness policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpatialVision {
    pub initial_pitch_degrees: f32,
    pub follow_movement: bool,
}
impl SpatialVision {
    pub fn validate(self) -> crate::model::Result<()> {
        if !self.initial_pitch_degrees.is_finite()
            || !(-90. ..=90.).contains(&self.initial_pitch_degrees)
        {
            return Err(crate::model::invalid(
                "Spatial vision pitch must be in [-90,90] degrees",
            ));
        }
        Ok(())
    }
}

pub fn view_direction(yaw: f32, pitch: f32) -> Vec3 {
    let (sy, cy) = yaw.to_radians().sin_cos();
    let (sp, cp) = pitch.to_radians().sin_cos();
    Vec3::new(sy * cp, sp, -cy * cp)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Unaware,
    Pursuing,
    Investigating,
    Exploring,
    Searching,
    Lost,
    Disabled,
    Dead,
    Suspended,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Vision,
    Omniscient,
    Sound,
    Report,
}
impl Source {
    pub fn is_visual(self) -> bool {
        matches!(self, Self::Vision | Self::Omniscient)
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    pub instance: Option<crate::identity::Reference>,
    pub life: Option<u64>,
    pub position: [f32; 3],
    pub tick: u64,
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_id: Option<u64>,
    /// Slot in the ID-sorted actor catalog; decoded to an ID only during inspection.
    pub actor_index: Option<u16>,
}
impl Observation {
    pub fn current_actor(self, actors: &crate::targeting::Catalog) -> Option<u16> {
        self.actor_index
            .filter(|&a| self.instance.is_some_and(|r| actors.matches(a, r)))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routine: Option<crate::npc_routines::State>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<crate::npc_cooperation::State>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tactics: Option<crate::npc_tactics::State>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circulation: Option<crate::npc_circulation::State>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mind_profile: Option<crate::npc_mind::Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mind_search_points: Option<u8>,
    pub corpse_visual: Option<Observation>,
    pub known_instances: Vec<Option<crate::identity::Reference>>,
    pub known_lives: Vec<u64>,
    pub locomotion_locked: bool,
    #[serde(default)]
    pub known_dead: crate::actor_set::ActorSet,
    #[serde(default)]
    pub communication: crate::communication::State,
    pub phase: Phase,
    pub targeting: crate::targeting::Selection,
    pub knowledge: crate::recognition::Knowledge,
    #[serde(default)]
    pub support: crate::ability_policy::Support,
    pub sound_attention: crate::sound_attention::State,
    pub objectives: crate::objectives::State,
    pub territory: crate::territory::State,
    pub reason: String,
    pub decision_tick: u64,
    /// True only for a selected, confirmed visual/omniscient observation; raw sightings are in knowledge.
    pub observed_this_tick: bool,
    pub heard_this_tick: bool,
    pub memory: Option<Observation>,
    /// Linear recency weight, not a statistically calibrated probability.
    pub confidence: f32,
    pub facing_degrees: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facing_pitch_degrees: Option<f32>,
    #[serde(default)]
    pub vision_follows_movement: bool,
    pub search_started_tick: Option<u64>,
    pub search_point: Option<[f32; 3]>,
    pub search_point_index: u8,
    pub skipped_search_points: u8,
    pub point_scan_started_tick: Option<u64>,
    /// Actual XZ distance after losing sight; retained on give-up for diagnostics.
    pub investigation_distance: f32,
    pub distance_accounted_tick: Option<u64>,
}
impl NpcState {
    pub fn new(config: &NpcConfig) -> Self {
        let facing = match config.awareness.clone().unwrap_or_default() {
            Awareness::Perceptive {
                initial_facing_degrees,
                ..
            } => initial_facing_degrees,
            Awareness::Omniscient => 0.,
        };
        Self {
            mind_profile: None,
            routine: None,
            order: None,
            tactics: None,
            circulation: None,
            mind_search_points: None,
            corpse_visual: None,
            known_instances: Vec::new(),
            known_lives: Vec::new(),
            locomotion_locked: false,
            communication: Default::default(),
            phase: Phase::Unaware,
            known_dead: crate::actor_set::ActorSet::EMPTY,
            targeting: Default::default(),
            knowledge: Default::default(),
            support: Default::default(),
            sound_attention: Default::default(),
            objectives: Default::default(),
            territory: Default::default(),
            reason: "awaiting_step".into(),
            decision_tick: 0,
            observed_this_tick: false,
            heard_this_tick: false,
            memory: None,
            confidence: 0.,
            facing_degrees: facing,
            facing_pitch_degrees: config.spatial_vision.map(|v| v.initial_pitch_degrees),
            vision_follows_movement: config.spatial_vision.is_some_and(|v| v.follow_movement),
            search_started_tick: None,
            search_point: None,
            search_point_index: 0,
            skipped_search_points: 0,
            point_scan_started_tick: None,
            investigation_distance: 0.,
            distance_accounted_tick: None,
        }
    }
    pub(crate) fn with_actor_storage(config: &NpcConfig, count: usize) -> Self {
        let mut state = Self::new(config);
        state.size_actor_storage(count);
        state
    }
    fn size_actor_storage(&mut self, count: usize) {
        self.known_instances.resize(count, None);
        self.known_lives.resize(count, 0);
    }
    pub fn die(&mut self) {
        if let Some(r) = &mut self.routine {
            r.suspend_life();
        }
        self.order = None;
        self.tactics = None;
        self.circulation = None;
        self.corpse_visual = None;
        if self.phase == Phase::Dead {
            return;
        }
        self.locomotion_locked = false;
        self.forget(Phase::Dead, "actor_dead");
        self.observed_this_tick = false;
        self.heard_this_tick = false;
        self.objectives = Default::default();
        self.targeting = Default::default();
        self.knowledge = Default::default();
        self.support = Default::default();
        self.sound_attention = Default::default();
        self.communication = Default::default();
    }
    pub fn current_visual(
        &self,
        actor: u16,
        tick: u64,
        actors: &crate::targeting::Catalog,
    ) -> Option<Observation> {
        self.memory
            .filter(|o| {
                self.observed_this_tick
                    && o.tick == tick
                    && o.source.is_visual()
                    && o.current_actor(actors) == Some(actor)
            })
            .or_else(|| {
                self.support
                    .observations
                    .get(actor as usize)
                    .copied()
                    .flatten()
                    .filter(|o| {
                        o.tick == tick
                            && o.source.is_visual()
                            && o.current_actor(actors) == Some(actor)
                    })
            })
    }
    fn recognize_death(
        &mut self,
        observation: Observation,
        dead: crate::actor_set::ActorSet,
    ) -> bool {
        let Some(a) = observation.actor_index else {
            return false;
        };
        self.known_instances[a as usize] = observation.instance;
        self.known_lives[a as usize] = observation.life.unwrap_or(0);
        if !dead.contains(a as usize) {
            self.known_dead.remove(a as usize);
            return false;
        }
        self.known_dead.insert(a as usize);
        if self
            .memory
            .is_some_and(|m| m.instance == observation.instance)
        {
            self.forget(Phase::Lost, "observed_target_dead");
        }
        true
    }
    fn territory_return(&mut self) {
        self.support = Default::default();
        self.forget(Phase::Suspended, "territory_return");
        self.observed_this_tick = false;
        self.heard_this_tick = false;
        if self.knowledge.threshold != 0 {
            self.knowledge = Default::default();
        }
        self.targeting = crate::targeting::Selection {
            reason: crate::targeting::Reason::TerritoryReturn,
            ..Default::default()
        };
        self.sound_attention = crate::sound_attention::State {
            reason: crate::sound_attention::Reason::TerritoryReturn,
            ..Default::default()
        };
        self.objectives.territory_suspend();
    }
    fn forget(&mut self, phase: Phase, reason: &str) {
        self.phase = phase;
        reason.clone_into(&mut self.reason);
        self.memory = None;
        self.sound_attention.synchronize(None);
        self.confidence = 0.;
        self.search_started_tick = None;
        self.search_point = None;
        self.point_scan_started_tick = None;
    }
    fn unexpired_memory(&self, policy: &Awareness, tick: u64) -> Option<Observation> {
        let memory = self.memory?;
        if self.communication.memory_expired(memory, tick) {
            return None;
        }
        if let Awareness::Perceptive {
            memory_ticks,
            search_ticks,
            ..
        } = policy
            && (memory_ticks
                .is_some_and(|limit| tick.saturating_sub(memory.tick) >= u64::from(limit))
                || self.search_started_tick.is_some_and(|start| {
                    search_ticks.is_some_and(|limit| tick.saturating_sub(start) >= u64::from(limit))
                }))
        {
            return None;
        }
        Some(memory)
    }
    pub fn face_velocity(&mut self, velocity: [f32; 2]) {
        self.face_spatial_velocity([velocity[0], 0., velocity[1]]);
    }
    /// Commanded motor orientation, not a hidden-target lookup or body/mesh rotation.
    pub fn face_spatial_velocity(&mut self, velocity: [f32; 3]) {
        if self.facing_pitch_degrees.is_some() && !self.vision_follows_movement {
            return;
        }
        let horizontal = Vec2::new(velocity[0], velocity[2]);
        if self.facing_pitch_degrees.is_some() && Vec3::from(velocity).length_squared() > 1e-8 {
            self.facing_pitch_degrees = Some(velocity[1].atan2(horizontal.length()).to_degrees());
        }
        if horizontal.length_squared() > 1e-8 {
            // Yaw 0 faces -Z; +90 faces +X. Bodies remain axis-aligned.
            self.facing_degrees = velocity[0].atan2(-velocity[2]).to_degrees();
        }
    }
    /// Account resolved physical displacement, not requested speed or straight-line goal distance.
    pub fn record_travel(&mut self, displacement: [f32; 2], limit: Option<f32>, tick: u64) -> bool {
        self.record_spatial_travel([displacement[0], 0., displacement[1]], limit, tick)
    }
    pub fn record_spatial_travel(
        &mut self,
        displacement: [f32; 3],
        limit: Option<f32>,
        tick: u64,
    ) -> bool {
        if !matches!(
            self.phase,
            Phase::Investigating | Phase::Exploring | Phase::Searching
        ) {
            return false;
        }
        self.investigation_distance += Vec3::from(displacement).length();
        self.distance_accounted_tick = Some(tick);
        // One millimetre: contact friction can cancel the tiny final capped velocity.
        // Preserve measured travel rather than rounding it up to the configured limit.
        if limit.is_some_and(|d| self.investigation_distance >= d - 0.001) {
            self.forget(Phase::Lost, "travel_budget_exhausted");
            self.decision_tick = tick;
            return true;
        }
        false
    }
}

pub(crate) fn segment_intersects_box(a: Vec3, b: Vec3, center: Vec3, half: Vec3) -> bool {
    let delta = b - a;
    let (mut enter, mut leave) = (0_f32, 1_f32);
    for axis in 0..3 {
        let (min, max) = (center[axis] - half[axis], center[axis] + half[axis]);
        if delta[axis].abs() < 1e-7 {
            if a[axis] < min || a[axis] > max {
                return false;
            }
        } else {
            let t1 = (min - a[axis]) / delta[axis];
            let t2 = (max - a[axis]) / delta[axis];
            enter = enter.max(t1.min(t2));
            leave = leave.min(t1.max(t2));
            if enter > leave {
                return false;
            }
        }
    }
    true
}

pub(crate) fn segment_intersects_entity(a: Vec3, b: Vec3, e: &Entity, center: Vec3) -> bool {
    if e.geometry.is_box() {
        segment_intersects_box(a, b, center, Vec3::from(e.transform.size) * 0.5)
    } else {
        crate::geometry::Volume::entity(e, center)
            .cast(a, b - a, Vec3::ZERO, false, 1e-7)
            .is_some()
    }
}

/// This is the only stage allowed to read the live target position for perceptive NPCs.
#[allow(clippy::too_many_arguments)]
fn observe(
    scene: &Scene,
    npc: &Entity,
    target_id: &str,
    positions: &BTreeMap<String, [f32; 3]>,
    orientation: &NpcState,
    tick: u64,
    policy: &Awareness,
    spatial: Option<&crate::combat_spatial::Index>,
) -> Option<Observation> {
    let position = crate::runtime_world::position(positions, target_id)?;
    let source = match policy {
        Awareness::Omniscient => Source::Omniscient,
        Awareness::Perceptive {
            vision_range,
            fov_degrees,
            ..
        } => {
            // Eye at 85% of NPC body height; sample the target centre.
            let eye = Vec3::from(positions[&npc.id]) + Vec3::Y * npc.transform.size[1] * 0.35;
            let target = Vec3::from(position);
            let offset = target - eye;
            if offset.length() > *vision_range {
                return None;
            }
            if *fov_degrees < 360. {
                let limit = (fov_degrees * 0.5).to_radians().cos();
                let outside = if let Some(pitch) = orientation.facing_pitch_degrees {
                    offset.length_squared() > 1e-8
                        && view_direction(orientation.facing_degrees, pitch).dot(offset.normalize())
                            + 1e-6
                            < limit
                } else {
                    let horizontal = Vec2::new(offset.x, offset.z);
                    let yaw = orientation.facing_degrees.to_radians();
                    horizontal.length_squared() > 1e-8
                        && Vec2::new(yaw.sin(), -yaw.cos()).dot(horizontal.normalize()) + 1e-6
                            < limit
                };
                if outside {
                    return None;
                }
            }
            let blocked = |index: usize| {
                let e = &scene.entities[index];
                e.id != npc.id
                    && e.id != target_id
                    && e.blocks_vision.unwrap_or(e.body.is_some())
                    && positions.contains_key(&e.id)
                    && segment_intersects_entity(
                        eye,
                        target,
                        e,
                        spatial
                            .and_then(|s| s.position(index))
                            .unwrap_or_else(|| positions[&e.id].into()),
                    )
            };
            let occluded = spatial.map_or_else(
                || (0..scene.entities.len()).any(blocked),
                |s| s.sight_candidates(eye, target).any(|(i, _)| blocked(i)),
            );
            if occluded {
                return None;
            }
            Source::Vision
        }
    };
    Some(Observation {
        instance: None,
        life: None,
        position,
        tick,
        source,
        noise_id: None,
        actor_index: None,
    })
}

#[derive(Clone, Copy)]
pub struct Stimuli<'a> {
    pub manual: &'a [crate::noise::Noise],
    pub automatic: &'a [crate::noise::Noise],
    pub abilities: &'a [crate::noise::Noise],
}

/// Filter audible impulses before selecting the strongest; ties prefer the later command.
/// Reads the listener/occluders, never the source entity or hidden target.
#[allow(clippy::too_many_arguments)]
fn hear(
    scene: &Scene,
    listener_instance: crate::identity::Reference,
    npc: &Entity,
    positions: &BTreeMap<String, [f32; 3]>,
    policy: &Awareness,
    stimuli: Stimuli<'_>,
    state: &mut NpcState,
    active_memory: Option<Observation>,
    spatial: Option<&crate::combat_spatial::Index>,
) -> Option<Observation> {
    use crate::sound_attention::Reason;
    if stimuli.manual.is_empty() && stimuli.automatic.is_empty() && stimuli.abilities.is_empty() {
        state.sound_attention.reason = Reason::NoStimuli;
        return None;
    }
    let Awareness::Perceptive {
        hearing: Some(hearing),
        ..
    } = policy
    else {
        state.sound_attention.reason = Reason::HearingDisabled;
        return None;
    };
    let config = npc.npc.as_ref().unwrap().sound_attention;
    let listener = Vec3::from(positions[&npc.id]);
    let mut best = None;
    let mut best_strength = -1_f32;
    // Automatic impulses were generated by the previous physical step; manual commands
    // at this tick happened afterwards and therefore win exact intensity ties.
    for noise in stimuli
        .automatic
        .iter()
        .chain(stimuli.abilities)
        .chain(stimuli.manual)
    {
        if noise.emitter_instance == Some(listener_instance) {
            continue;
        }
        let origin = Vec3::from(noise.position);
        let distance_squared = listener.distance_squared(origin);
        let mut radius = noise.radius.min(hearing.range);
        if distance_squared > radius * radius {
            continue;
        }
        let mut indexed = spatial.map(|s| s.ordered_candidates(listener, origin));
        let mut linear = 0..scene.entities.len();
        while let Some(index) = indexed
            .as_mut()
            .map_or_else(|| linear.next(), Iterator::next)
        {
            let obstacle = &scene.entities[index];
            let transmission = obstacle.sound_transmission.unwrap_or(1.);
            if transmission < 1.
                && obstacle.id != npc.id
                && positions.contains_key(&obstacle.id)
                && segment_intersects_entity(
                    listener,
                    origin,
                    obstacle,
                    Vec3::from(positions[&obstacle.id]),
                )
            {
                radius *= transmission;
                if radius == 0. || distance_squared > radius * radius {
                    break;
                }
            }
        }
        if radius == 0. || distance_squared > radius * radius {
            continue;
        }
        if npc
            .npc
            .as_ref()
            .unwrap()
            .territory
            .is_some_and(|c| !c.contains(noise.position))
        {
            state.territory.rejected_sound += 1;
            continue;
        }
        let strength = 1. - distance_squared.sqrt() / radius;
        let observation = Observation {
            instance: None,
            life: None,
            position: noise.position,
            tick: noise.tick,
            source: Source::Sound,
            noise_id: Some(noise.id),
            actor_index: None,
        };
        if state.sound_attention.consider(
            config,
            active_memory,
            state.decision_tick,
            observation,
            strength,
        ) && strength >= best_strength
        {
            best_strength = strength;
            best = Some(observation);
        }
    }
    if let Some(observation) = best {
        state
            .sound_attention
            .accept(config, active_memory, observation, best_strength);
    } else {
        state.sound_attention.reason = if state.sound_attention.audible_candidates == 0 {
            if state.territory.rejected_sound > 0 {
                Reason::OutsideTerritory
            } else {
                Reason::NoAudibleCue
            }
        } else {
            Reason::Rejected
        };
    }
    best
}

pub fn update(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    navigation: &BTreeMap<String, NavState>,
    states: &mut BTreeMap<String, NpcState>,
    goals: &mut BTreeMap<String, Option<NavGoal>>,
) {
    update_with_noises(scene, positions, tick, navigation, states, goals, &[]);
}

pub fn update_with_noises(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    navigation: &BTreeMap<String, NavState>,
    states: &mut BTreeMap<String, NpcState>,
    goals: &mut BTreeMap<String, Option<NavGoal>>,
    noises: &[crate::noise::Noise],
) {
    update_with_stimuli(
        scene,
        positions,
        tick,
        navigation,
        states,
        goals,
        Stimuli {
            manual: noises,
            automatic: &[],
            abilities: &[],
        },
    );
}

pub fn update_with_stimuli(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    navigation: &BTreeMap<String, NavState>,
    states: &mut BTreeMap<String, NpcState>,
    goals: &mut BTreeMap<String, Option<NavGoal>>,
    stimuli: Stimuli<'_>,
) {
    let actors = crate::targeting::Catalog::new(scene);
    let lives = vec![0; actors.actors.len()];
    let ability_ready = vec![0; actors.actors.len()];
    update_with_context(
        scene,
        positions,
        tick,
        navigation,
        states,
        goals,
        Sensors {
            lives: &lives,
            stimuli,
            actors: &actors,
            ability_ready: &ability_ready,
            ability_policy: None,
            dead: crate::actor_set::ActorSet::EMPTY,
        },
    );
}
pub struct AbilityContext<'a> {
    pub runtime: &'a crate::abilities::Runtime,
    pub health: &'a [Option<crate::vitality::Health>],
}
pub struct Sensors<'a> {
    pub lives: &'a [u64],
    pub ability_policy: Option<AbilityContext<'a>>,
    pub dead: crate::actor_set::ActorSet,
    pub stimuli: Stimuli<'a>,
    pub actors: &'a crate::targeting::Catalog,
    pub ability_ready: &'a [u16],
}
pub fn update_with_context(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    navigation: &BTreeMap<String, NavState>,
    states: &mut BTreeMap<String, NpcState>,
    goals: &mut BTreeMap<String, Option<NavGoal>>,
    sensors: Sensors<'_>,
) {
    let mut reports = Vec::with_capacity(states.len());
    update_with_spatial(
        scene,
        positions,
        tick,
        navigation,
        states,
        goals,
        sensors,
        None,
        &mut reports,
    );
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_with_spatial(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    navigation: &BTreeMap<String, NavState>,
    states: &mut BTreeMap<String, NpcState>,
    goals: &mut BTreeMap<String, Option<NavGoal>>,
    sensors: Sensors<'_>,
    spatial: Option<&crate::combat_spatial::Index>,
    reports: &mut crate::communication::Reports,
) {
    let Sensors {
        lives,
        stimuli,
        actors,
        ability_ready,
        ability_policy,
        dead,
    } = sensors;
    for (a, &index) in actors.actors.iter().enumerate() {
        if dead.contains(a)
            && let Some(state) = states.get_mut(&scene.entities[index as usize].id)
        {
            state.die();
        }
    }
    crate::communication::collect_into(scene, positions, tick, actors, states, reports);
    let empty_navigation = NavState::default();
    goals.retain(|id, _| states.contains_key(id));
    for (id, state) in states {
        if !goals.contains_key(id) {
            goals.insert(id.clone(), None);
        }
        let slot = goals.get_mut(id).unwrap();
        *slot = None;
        let (npc_index, npc) = scene
            .entities
            .iter()
            .enumerate()
            .find(|(_, e)| e.id == *id)
            .unwrap();
        let config = npc.npc.as_ref().unwrap();
        let policy = config.awareness.clone().unwrap_or_default();
        state.size_actor_storage(actors.actors.len());
        state.decision_tick = tick;
        state.support.observations.fill(None);
        state.corpse_visual = None;

        state.observed_this_tick = false;
        state.heard_this_tick = false;
        if state.phase == Phase::Dead {
            continue;
        }
        if !config.enabled {
            state.support = Default::default();
            state.territory = crate::territory::State {
                phase: crate::territory::Phase::Disabled,
                ..Default::default()
            };
            state.objectives = crate::objectives::State {
                reason: crate::objectives::Reason::Disabled,
                ..Default::default()
            };
            state.sound_attention = crate::sound_attention::State {
                reason: crate::sound_attention::Reason::Disabled,
                ..Default::default()
            };
            if state.knowledge.threshold != 0 {
                state.knowledge = Default::default();
            }
            state.targeting = crate::targeting::Selection {
                reason: crate::targeting::Reason::Disabled,
                ..Default::default()
            };
            state.forget(Phase::Disabled, "disabled_by_configuration");
            state.search_point_index = 0;
            state.skipped_search_points = 0;
            state.investigation_distance = 0.;
            state.distance_accounted_tick = None;
            continue;
        }
        state.territory.begin(config.territory, positions[id], tick);
        if state.territory.blocked() {
            state.support = Default::default();
            state.territory_return();
            *slot = state.territory.return_goal(
                config.territory.unwrap(),
                positions[id],
                npc.transform.size[1] * 0.5,
            );
            continue;
        }
        if config.recognition.is_some() {
            state
                .knowledge
                .tracks
                .resize(actors.actors.len(), crate::recognition::Track::default());
            state.knowledge.begin();
        }
        observe_support(
            scene, npc_index, positions, tick, &policy, state, actors, dead, lives, spatial,
        );
        let active_memory = state.unexpired_memory(&policy, tick);
        state.sound_attention.begin(active_memory);
        let visual = select_observation(
            scene, npc_index, positions, tick, &policy, state, actors, dead, lives, spatial,
        );
        // A confirmed corpse can invalidate memory; hidden deaths cannot.
        let active_memory = state.unexpired_memory(&policy, tick);
        if visual.is_none() && state.territory.active_target_departed {
            state
                .territory
                .start(crate::territory::Trigger::TargetLeft, tick);
            state.territory_return();
            *slot = state.territory.return_goal(
                config.territory.unwrap(),
                positions[id],
                npc.transform.size[1] * 0.5,
            );
            continue;
        }
        let direct = if let Some(visual) = visual {
            state.sound_attention.reason = crate::sound_attention::Reason::VisualPriority;
            Some(visual)
        } else {
            hear(
                scene,
                actors.entity_reference(npc_index).unwrap(),
                npc,
                positions,
                &policy,
                stimuli,
                state,
                active_memory,
                spatial,
            )
        };
        let observation = direct.or_else(|| {
            crate::communication::receive(
                reports,
                crate::communication::Listener {
                    scene,
                    actors,
                    npc_index,
                    position: positions[id],
                    tick,
                    active_memory,
                    known_dead: state.known_dead,
                    known_lives: &state.known_lives,
                    known_instances: &state.known_instances,
                },
                &mut state.communication,
            )
        });
        if direct.is_some() && config.communication.is_some() {
            state.communication.reason = crate::communication::Reason::DirectEvidence;
        }
        state.knowledge.finish(config.recognition);
        let actor = actors.slot(npc_index);
        let mut eligible = u8::MAX;
        if let Some(objectives) = &config.objectives {
            for (i, objective) in objectives.iter().enumerate() {
                let previous = state.objectives.ability_decisions[i];
                state.objectives.ability_decisions[i] = Default::default();
                if let crate::objectives::Action::UseAbility {
                    slot: grant,
                    policy: decision_policy,
                    ..
                } = objective.action
                {
                    if ability_ready[actor as usize] & (1 << (grant - 1)) == 0 {
                        eligible &= !(1 << i);
                        if decision_policy.is_some() {
                            state.objectives.ability_decisions[i].phase =
                                crate::ability_policy::Phase::Unavailable;
                        }
                        continue;
                    }
                    if let Some(p) = decision_policy {
                        let decision = if let Some(context) = &ability_policy {
                            let evidence = crate::ability_policy::Evidence {
                                actor,
                                position: positions[id],
                                health: context.health[actor as usize],
                                target: observation
                                    .filter(|o| {
                                        o.tick == tick
                                            && o.source.is_visual()
                                            && o.current_actor(actors).is_some()
                                    })
                                    .and_then(|o| {
                                        o.actor_index.map(|target| crate::ability_policy::Target {
                                            actor: target,
                                            position: o.position,
                                            health: context.health[target as usize],
                                            relation: actors.attitude(actor, target),
                                        })
                                    }),
                            };
                            let offer = context.runtime.policy_offer(actor as usize, grant);
                            let matches_effects = |target| {
                                !p.require_dispel_match
                                    || context.runtime.policy_dispel_match(
                                        scene, actors, tick, actor, grant, target,
                                    )
                            };
                            if p.allies.is_some() {
                                let candidates = actors
                                    .order()
                                    .iter()
                                    .copied()
                                    .take(actors.actors.len())
                                    .map(usize::from)
                                    .filter_map(|a| {
                                        let own = a == actor as usize;
                                        let position = if own {
                                            positions[id]
                                        } else {
                                            state.support.observations[a]?.position
                                        };
                                        if dead.contains(a) {
                                            return None;
                                        }
                                        Some(crate::ability_policy::Target {
                                            actor: a as u16,
                                            position,
                                            health: context.health[a],
                                            relation: actors.attitude(actor, a as u16),
                                        })
                                    });
                                crate::ability_policy::choose_ally(
                                    p,
                                    &offer,
                                    &evidence,
                                    previous,
                                    candidates,
                                    matches_effects,
                                )
                            } else {
                                let mut d =
                                    crate::ability_policy::evaluate(p, &offer, &evidence, previous);
                                if d.eligible() && !matches_effects(d.target.unwrap()) {
                                    d.phase = crate::ability_policy::Phase::NoMatchingEffect;
                                }
                                d
                            }
                        } else {
                            crate::ability_policy::Decision {
                                phase: crate::ability_policy::Phase::Unavailable,
                                target: None,
                            }
                        };
                        state.objectives.ability_decisions[i] = decision;
                        if !decision.eligible() {
                            eligible &= !(1 << i);
                        }
                    }
                }
            }
        }
        let mind_context = config.mind.map(|mind| {
            let instance = actors.reference(actor).instance;
            let profile = *state
                .mind_profile
                .get_or_insert_with(|| mind.resolve(instance));
            crate::npc_mind::Context {
                profile,
                instance,
                health: ability_policy
                    .as_ref()
                    .and_then(|c| c.health[actor as usize]),
            }
        });
        let action = state.objectives.select_minded(
            config.objectives.as_deref(),
            observation.or(active_memory),
            observation.is_some_and(|o| o.source.is_visual()),
            tick,
            eligible,
            mind_context,
        );
        let respond = config.objectives.is_none()
            || matches!(action, Some(crate::objectives::Action::Respond));
        let feedback = if state.objectives.changed_this_tick || state.territory.changed_this_tick {
            &empty_navigation
        } else {
            &navigation[id]
        };
        let goal = decide(&policy, observation, tick, feedback, state, respond);
        if let Some(memory) = state.memory.filter(|m| m.source == Source::Report)
            && let Some(receipt) = state.communication.last_received
        {
            let lifetime = (receipt.expires_at - memory.tick).max(1);
            let recency = 1. - tick.saturating_sub(memory.tick) as f32 / lifetime as f32;
            state.confidence = state.confidence.min(recency.max(0.));
        }
        if config.objectives.is_some() && !respond {
            *slot = state.objectives.goal(
                action,
                state.memory,
                positions[id],
                npc.transform.size[1] * 0.5,
            );
        }
        state.sound_attention.synchronize(state.memory);
        if let Some((position, remembered)) = goal {
            let actor_index = state.memory.as_ref().and_then(|m| m.current_actor(actors));
            // Actor identity comes from direct vision or an explicit frozen report.
            let target = actor_index
                .map(|i| &scene.entities[actors.entity(i)])
                .unwrap_or(npc);
            let radius =
                |e: &Entity| Vec2::new(e.transform.size[0], e.transform.size[2]).length() * 0.5;
            *slot = Some(NavGoal {
                excluded_entity: actor_index.map(|i| actors.entity(i) as u16),
                position,
                replan_on_target_change: remembered,
                target_half_height: target.transform.size[1] * 0.5,
                remaining_travel: if remembered {
                    match policy {
                        Awareness::Perceptive {
                            max_investigation_distance,
                            ..
                        } => max_investigation_distance
                            .map(|d| (d - state.investigation_distance).max(0.)),
                        Awareness::Omniscient => None,
                    }
                } else {
                    None
                },
                stop_distance: if remembered {
                    0.4
                } else {
                    config
                        .stop_distance
                        .max(radius(npc) + radius(target) + 0.06)
                },
            });
        }
        if let Some(crate::objectives::Action::UseAbility {
            slot: grant,
            policy: Some(p),
            ..
        }) = action
            && let Some(i) = state.objectives.active
            && state.objectives.ability_decisions[i as usize].phase
                == crate::ability_policy::Phase::Approaching
            && let Some(context) = &ability_policy
            && let Some(target) = state.objectives.ability_decisions[i as usize].target
            && let Some(observation) = if p.allies.is_some() {
                state.support.observations[target as usize]
            } else {
                observation.filter(|o| {
                    o.tick == tick && o.source.is_visual() && o.current_actor(actors).is_some()
                })
            }
        {
            let range = p.range(context.runtime.policy_range(actor as usize, grant))
                - p.approach.unwrap().margin;
            // Ground navigation measures XZ; account for centre-height separation.
            let stop = if config.mobility.is_none() {
                let dy = positions[id][1] - observation.position[1];
                (range * range - dy * dy).max(0.).sqrt()
            } else {
                range
            };
            // Existing navigation has a 2cm arrival tolerance. Stay inside policy range.
            *slot = Some(NavGoal {
                position: observation.position,
                excluded_entity: Some(actors.entity(target) as u16),
                target_half_height: scene.entities[actors.entity(target)].transform.size[1] * 0.5,
                stop_distance: (stop - 0.04).max(0.01),
                remaining_travel: None,
                replan_on_target_change: false,
            });
            state.objectives.destination = Some(observation.position);
            state.objectives.evidence_tick = Some(observation.tick);
        } else if matches!(
            action,
            Some(crate::objectives::Action::UseAbility {
                policy: Some(_),
                ..
            })
        ) {
            state.objectives.destination = None;
            state.objectives.evidence_tick = None;
        }
        if let Some(c) = config.territory
            && slot.as_ref().is_some_and(|g| !c.contains(g.position))
        {
            state
                .territory
                .start(crate::territory::Trigger::GoalOutside, tick);
            state.territory_return();
            *slot = state
                .territory
                .return_goal(c, positions[id], npc.transform.size[1] * 0.5);
        }
    }
}

/// One bounded support sensor pass per NPC, shared by every support objective.
#[allow(clippy::too_many_arguments)]
fn observe_support(
    scene: &Scene,
    npc_index: usize,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    policy: &Awareness,
    state: &mut NpcState,
    actors: &crate::targeting::Catalog,
    dead: crate::actor_set::ActorSet,
    lives: &[u64],
    spatial: Option<&crate::combat_spatial::Index>,
) {
    let npc = &scene.entities[npc_index];
    let config = npc.npc.as_ref().unwrap();
    let enabled = config.objectives.as_ref().is_some_and(|os| {
        os.iter().any(|o| {
            matches!(o.action,
        crate::objectives::Action::UseAbility { policy: Some(p), .. } if p.allies.is_some())
        })
    });
    if !enabled {
        return;
    }
    state.support.observations.resize(actors.actors.len(), None);
    if config.recognition.is_some() {
        state
            .support
            .knowledge
            .tracks
            .resize(actors.actors.len(), crate::recognition::Track::default());
    }
    state.support.knowledge.begin();
    let own = actors.slot(npc_index);
    for (a, &index) in actors.actors.iter().enumerate() {
        if a == own as usize || actors.attitude(own, a as u16) != crate::targeting::Attitude::Allied
        {
            continue;
        }
        let Some(mut o) = observe(
            scene,
            npc,
            &scene.entities[index as usize].id,
            positions,
            state,
            tick,
            policy,
            spatial,
        ) else {
            continue;
        };
        o.actor_index = Some(a as u16);
        o.instance = Some(actors.reference(a as u16));
        o.life = Some(lives[a]);
        if config.territory.is_some_and(|c| !c.contains(o.position)) {
            continue;
        }
        if !state.support.knowledge.observe(config.recognition, o) {
            continue;
        }
        if state.recognize_death(o, dead) {
            continue;
        }
        state.support.observations[a] = Some(o);
    }
    state.support.knowledge.finish(config.recognition);
}

/// Only sensor-accepted observations are compared. Hidden positions never enter ranking.
#[allow(clippy::too_many_arguments)]
fn select_observation(
    scene: &Scene,
    npc_index: usize,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    policy: &Awareness,
    state: &mut NpcState,
    actors: &crate::targeting::Catalog,
    dead: crate::actor_set::ActorSet,
    lives: &[u64],
    spatial: Option<&crate::combat_spatial::Index>,
) -> Option<Observation> {
    use crate::targeting::{Attitude, Reason, Selection};
    let npc = &scene.entities[npc_index];
    let config = npc.npc.as_ref().unwrap();
    state.targeting = Selection::default();
    if let Some(target) = &config.target {
        let prior_memory = state.unexpired_memory(policy, tick);
        let observation = observe(scene, npc, target, positions, state, tick, policy, spatial)
            .map(|mut o| {
                let index = scene.entities.iter().position(|e| e.id == *target).unwrap();
                o.actor_index = Some(actors.slot(index));
                o.instance = Some(actors.reference(actors.slot(index)));
                o.life = Some(lives[actors.slot(index) as usize]);
                o
            })
            .filter(|o| {
                state
                    .territory
                    .allow_visual(config.territory, *o, prior_memory)
            });
        let confirmed = observation.is_some_and(|o| state.knowledge.observe(config.recognition, o));
        if let Some(o) = observation.filter(|_| confirmed)
            && state.recognize_death(o, dead)
        {
            state.corpse_visual = Some(o);
            state.targeting.reason = Reason::ObservedDead;
            return None;
        }
        state.targeting = Selection {
            reason: if observation.is_some() && !confirmed {
                Reason::AwaitingConfirmation
            } else if observation.is_some() {
                Reason::ExplicitObserved
            } else if state.territory.rejected_visual > 0 {
                Reason::OutsideTerritory
            } else {
                Reason::ExplicitUnobserved
            },
            eligible_candidates: 1,
            observed_candidates: u16::from(observation.is_some()),
            confirmed_candidates: u16::from(confirmed),
            observed_actor: observation
                .filter(|_| confirmed)
                .and_then(|o| o.actor_index),
            observed_distance_squared: observation
                .filter(|_| confirmed)
                .as_ref()
                .map(|o| Vec3::from(positions[&npc.id]).distance_squared(o.position.into())),
            incumbent_distance_squared: None,
            nearest_distance_squared: observation
                .filter(|_| confirmed)
                .as_ref()
                .map(|o| Vec3::from(positions[&npc.id]).distance_squared(o.position.into())),
        };
        return observation.filter(|_| confirmed);
    }
    let Some(targeting) = config.targeting else {
        state.targeting.reason = Reason::NoVisualTarget;
        return None;
    };
    let prior = state.memory.as_ref().and_then(|m| m.current_actor(actors));
    let mut best: Option<(Observation, f32)> = None;
    let mut incumbent: Option<(Observation, f32)> = None;
    let own = actors.slot(npc_index);
    let own_position = Vec3::from(positions[&npc.id]);
    for slot in actors.order().iter().copied().take(actors.actors.len()) {
        let index = actors.entity(slot);
        if own == slot || actors.attitude(own, slot) != Attitude::Hostile {
            continue;
        }
        state.targeting.eligible_candidates += 1;
        let candidate = &scene.entities[index];
        let Some(mut observation) = observe(
            scene,
            npc,
            &candidate.id,
            positions,
            state,
            tick,
            policy,
            spatial,
        ) else {
            continue;
        };
        observation.actor_index = Some(slot);
        observation.instance = Some(actors.reference(slot));
        observation.life = Some(lives[slot as usize]);
        let prior_memory = state.unexpired_memory(policy, tick);
        if !state
            .territory
            .allow_visual(config.territory, observation, prior_memory)
        {
            continue;
        }
        state.targeting.observed_candidates += 1;
        if !state.knowledge.observe(config.recognition, observation) {
            continue;
        }
        state.targeting.confirmed_candidates += 1;
        if state.recognize_death(observation, dead) {
            continue;
        }
        let distance = own_position.distance_squared(observation.position.into());
        if Some(slot) == prior {
            incumbent = Some((observation, distance));
        }
        // Logical instance order is independent of recycled slots, so equality keeps the first candidate.
        if best.as_ref().is_none_or(|(_, d)| distance < *d) {
            best = Some((observation, distance));
        }
    }
    let Some((mut chosen, distance)) = best else {
        state.targeting.reason = if state.targeting.confirmed_candidates > 0 {
            Reason::ObservedDead
        } else if state.targeting.observed_candidates > 0 {
            Reason::AwaitingConfirmation
        } else if state.territory.rejected_visual > 0 {
            Reason::OutsideTerritory
        } else {
            Reason::NoObservedHostile
        };
        return None;
    };
    state.targeting.reason = if prior.is_none() {
        Reason::Acquired
    } else {
        Reason::PreviousNotObserved
    };
    state.targeting.nearest_distance_squared = Some(distance);
    state.targeting.observed_distance_squared = Some(distance);
    if let Some((previous, prior_distance)) = incumbent {
        state.targeting.incumbent_distance_squared = Some(prior_distance);
        let ratio = 1. - targeting.margin();
        if chosen.actor_index != previous.actor_index && distance < prior_distance * ratio * ratio {
            state.targeting.reason = Reason::SwitchedCloser;
        } else {
            chosen = previous;
            state.targeting.observed_distance_squared = Some(prior_distance);
            state.targeting.reason = Reason::Retained;
        }
    }
    state.targeting.observed_actor = chosen.actor_index;
    Some(chosen)
}

/// Decisions have no access to the world snapshot or the hidden target.
fn decide(
    policy: &Awareness,
    observation: Option<Observation>,
    tick: u64,
    navigation: &NavState,
    state: &mut NpcState,
    execute: bool,
) -> Option<([f32; 3], bool)> {
    if let Some(observation) = observation {
        let heard = observation.source == Source::Sound;
        state.observed_this_tick = observation.source.is_visual();
        state.heard_this_tick = heard;
        state.phase = if !state.observed_this_tick {
            Phase::Investigating
        } else {
            Phase::Pursuing
        };
        let reason = match observation.source {
            Source::Vision => "visual_observation",
            Source::Omniscient => "omniscient_policy",
            Source::Sound => "sound_cue",
            Source::Report => "radio_report",
        };
        reason.clone_into(&mut state.reason);
        state.confidence = 1.;
        state.search_started_tick = None;
        state.search_point = None;
        state.search_point_index = 0;
        state.skipped_search_points = 0;
        state.point_scan_started_tick = None;
        state.investigation_distance = 0.;
        state.distance_accounted_tick = None;
        let position = observation.position;
        state.memory = Some(observation);
        if !execute {
            state.phase = Phase::Suspended;
            "objective_owns_movement".clone_into(&mut state.reason);
            return None;
        }
        return Some((position, !state.observed_this_tick));
    }
    let Some(memory) = &state.memory else {
        if state.phase != Phase::Lost {
            state.phase = Phase::Unaware;
            "no_observation".clone_into(&mut state.reason);
        }
        return None;
    };
    if state.communication.memory_expired(*memory, tick) {
        state.forget(Phase::Lost, "report_expired");
        return None;
    }
    let Awareness::Perceptive {
        memory_ticks,
        search_ticks,
        search_turn_degrees_per_second,
        area_search,
        ..
    } = policy
    else {
        unreachable!("enabled omniscient policy always supplies an observation")
    };
    let age = tick.saturating_sub(memory.tick);
    if memory_ticks.is_some_and(|limit| age >= u64::from(limit)) {
        state.forget(Phase::Lost, "memory_expired");
        return None;
    }
    state.confidence = memory_ticks.map_or(1., |limit| 1. - age as f32 / limit as f32);
    if state
        .search_started_tick
        .is_some_and(|start| search_ticks.is_some_and(|limit| tick - start >= u64::from(limit)))
    {
        state.forget(Phase::Lost, "search_budget_exhausted");
        return None;
    }
    if !execute {
        state.phase = Phase::Suspended;
        "objective_owns_movement".clone_into(&mut state.reason);
        return None;
    }

    if state.phase == Phase::Investigating
        && (navigation.status == "arrived"
            || (area_search.is_some() && navigation.status == "unreachable"))
        && navigation.target_position == Some(memory.position)
    {
        state.search_started_tick = Some(tick);
    }
    if let Some(started) = state.search_started_tick {
        if search_ticks.is_some_and(|limit| tick - started >= u64::from(limit)) {
            state.forget(Phase::Lost, "search_budget_exhausted");
        } else if let Some(search) = area_search {
            return explore(
                search,
                memory.position,
                *search_turn_degrees_per_second,
                tick,
                navigation,
                state,
            );
        } else {
            state.phase = Phase::Searching;
            "scan_last_known_position".clone_into(&mut state.reason);
            state.facing_degrees =
                (state.facing_degrees + search_turn_degrees_per_second * FIXED_DT + 180.)
                    .rem_euclid(360.)
                    - 180.;
        }
        return None;
    }
    state.phase = Phase::Investigating;
    (match memory.source {
        Source::Sound => "last_heard_position",
        Source::Report => "last_reported_position",
        _ => "last_known_position",
    })
    .clone_into(&mut state.reason);
    Some((memory.position, true))
}

/// Generate one candidate at a time. Seed rotates a regular ring; it is not a probabilistic model.
fn explore(
    search: &AreaSearch,
    center: [f32; 3],
    turn: f32,
    tick: u64,
    nav: &NavState,
    state: &mut NpcState,
) -> Option<([f32; 3], bool)> {
    if let Some(point) = state.search_point {
        if let Some(started) = state.point_scan_started_tick {
            if tick - started < u64::from(search.scan_ticks) {
                state.phase = Phase::Searching;
                "scan_search_point".clone_into(&mut state.reason);
                state.facing_degrees =
                    (state.facing_degrees + turn * FIXED_DT + 180.).rem_euclid(360.) - 180.;
                return None;
            }
            state.search_point_index += 1;
            state.search_point = None;
            state.point_scan_started_tick = None;
        } else if nav.target_position == Some(point) {
            if nav.status == "arrived" {
                state.point_scan_started_tick = Some(tick);
                state.phase = Phase::Searching;
                "scan_search_point".clone_into(&mut state.reason);
                return None;
            }
            if nav.status == "unreachable" {
                state.skipped_search_points += 1;
                state.search_point_index += 1;
                state.search_point = None;
            }
        }
    }
    let point_limit = state.mind_profile.map_or(search.points, |profile| {
        let fraction = (2500 + profile.intelligence as i32 / 2 + profile.traits[5] as i32 / 4
            - profile.traits[3] as i32 / 5
            + profile.traits[4] as i32 / 10)
            .clamp(1000, 10000);
        (search.points as i32 * fraction / 10000).clamp(1, search.points as i32) as u8
    });
    state.mind_search_points = state.mind_profile.map(|_| point_limit);
    if state.search_point_index >= point_limit {
        state.forget(Phase::Lost, "search_points_exhausted");
        return None;
    }
    let point = *state.search_point.get_or_insert_with(|| {
        let angle = ((search.seed % 360) as f32
            + f32::from(state.search_point_index) * 360. / f32::from(search.points))
        .to_radians();
        [
            center[0] + angle.sin() * search.radius,
            center[1],
            center[2] - angle.cos() * search.radius,
        ]
    });
    state.phase = Phase::Exploring;
    "investigate_search_point".clone_into(&mut state.reason);
    Some((point, true))
}
