mod avatar_support;
mod checkpoint;
mod gameplay;
mod planet;
mod player_view;
mod portable;
pub(crate) mod wind;
use crate::model::{BodyKind, FIXED_DT, Scene};
pub(crate) use checkpoint::Saved as SavedSimulation;
pub(crate) use portable::Saved as PortableSimulation;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub x: f32,
    pub z: f32,
    #[serde(default)]
    pub mode: crate::model::MovementMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactEvent {
    pub a_instance: crate::identity::Reference,
    pub b_instance: crate::identity::Reference,
    pub tick: u64,
    pub kind: String,
    pub a: String,
    pub b: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayerDrive {
    body: RigidBodyHandle,
    actor: usize,
    locomotion: crate::model::Locomotion,
}

fn player_input(legacy: Input, controls: &[Option<Input>], actor: usize, players: usize) -> Input {
    controls.get(actor).copied().flatten().unwrap_or_else(|| {
        if players == 1 {
            legacy
        } else {
            Input::default()
        }
    })
}

pub struct Simulation {
    pub(crate) wind: wind::Runtime,
    pub(crate) planet: Option<Box<crate::planet::runtime::World>>,
    pub(crate) physical_surfaces: Vec<crate::surfaces::Surface>,
    surface_bodies: BTreeMap<String, RigidBodyHandle>,
    report_scratch: crate::communication::Reports,
    pub presentation: crate::npc_presentation::StateSet,
    pub routines: crate::npc_routines::Runtime,
    pub interactions: crate::interactions::Runtime,
    interaction_geometry_dirty: bool,
    scene: crate::runtime_world::RuntimeWorld,
    positions_before_step: BTreeMap<String, [f32; 3]>,
    combat_spatial: crate::combat_spatial::Index,
    circulation_velocities: Vec<glam::Vec3>,
    circulation_enabled: bool,
    avatar_motion_enabled: bool,
    goals: BTreeMap<String, Option<crate::navigation::NavGoal>>,
    velocities: BTreeMap<String, [f32; 2]>,
    navigation_scratch: crate::navigation::NavigationScratch,
    pub navigation: BTreeMap<String, crate::navigation::NavState>,
    pub passages: crate::passages::Runtime,
    pub mobility: crate::mobility::Runtime,
    pub character_controllers: BTreeMap<String, crate::character_controller::State>,
    ground_motors: BTreeMap<String, rapier3d::control::KinematicCharacterController>,
    character_collisions: Vec<rapier3d::control::CharacterCollision>,
    pub npcs: BTreeMap<String, crate::perception::NpcState>,
    pub actors: crate::targeting::Catalog,
    pub population: crate::population::State,
    pub generators: crate::generators::State,
    pub abilities: crate::abilities::Runtime,
    pub vitality: crate::vitality::Runtime,
    pub footsteps: crate::footsteps::Footsteps,
    pub patrols: BTreeMap<String, crate::patrol::State>,
    pub triggers: crate::trigger::Runtime,
    pub patrol_routes: BTreeMap<String, Vec<[f32; 3]>>,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad: BroadPhaseBvh,
    narrow: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse: ImpulseJointSet,
    multibody: MultibodyJointSet,
    ccd: CCDSolver,
    handles: BTreeMap<String, RigidBodyHandle>,
    names: HashMap<ColliderHandle, usize>,
    contact_names: Vec<String>,
    contact_instances: Vec<Option<crate::identity::Reference>>,
    contact_slots: Vec<usize>,
    players: Vec<PlayerDrive>,
    ability_ready: Vec<u16>,
    deactivated_actors: crate::actor_set::ActorSet,
    contacts: Vec<(usize, usize)>,
    next_contacts: Vec<(usize, usize)>,
    pub events: VecDeque<ContactEvent>,
    pub dropped_events: u64,
    pub tick: u64,
}

impl Simulation {
    pub fn new(scene: &Scene) -> Self {
        let expanded = crate::templates::expand(scene);
        let scene = &expanded;
        let patrol_routes = crate::patrol::routes(scene);
        let patrols = patrol_routes
            .keys()
            .map(|id| (id.clone(), Default::default()))
            .collect();
        let actors = crate::targeting::Catalog::new(scene);
        let actor_count = actors.actors.len();
        let triggers = crate::trigger::Runtime::new(scene, &actors);
        let world = crate::runtime_world::RuntimeWorld::new(scene, &actors);
        let abilities = crate::abilities::Runtime::with_actors(scene, &actors);
        let vitality = crate::vitality::Runtime::with_actors(scene, &actors);
        let interactions = crate::interactions::Runtime::new(scene, &actors);
        let routines = crate::npc_routines::Runtime::new(scene, &interactions);
        let mut s = Self {
            wind: Default::default(),
            planet: None,
            physical_surfaces: crate::surfaces::prepare(scene),
            surface_bodies: BTreeMap::new(),
            report_scratch: Vec::with_capacity(
                scene
                    .entities
                    .iter()
                    .filter(|e| e.npc.is_some())
                    .count()
                    .max(crate::templates::capacity(scene)),
            ),
            presentation: Default::default(),
            routines,
            interactions,
            interaction_geometry_dirty: false,
            generators: crate::generators::State::new(scene, &actors),
            actors,
            population: Default::default(),
            abilities,
            vitality,
            passages: crate::passages::Runtime::new(scene),
            mobility: crate::mobility::Runtime::new(scene),
            character_controllers: BTreeMap::new(),
            ground_motors: BTreeMap::new(),
            character_collisions: Vec::new(),
            triggers,
            patrol_routes,
            patrols,
            scene: world,
            positions_before_step: BTreeMap::new(),
            combat_spatial: Default::default(),
            circulation_velocities: Vec::new(),
            circulation_enabled: scene
                .entities
                .iter()
                .chain(&scene.npc_templates)
                .any(|e| e.npc.as_ref().is_some_and(|n| n.circulation.is_some())),
            avatar_motion_enabled: scene
                .entities
                .iter()
                .chain(&scene.npc_templates)
                .any(|e| e.avatar.as_ref().is_some_and(|a| a.root_motion.is_some())),
            goals: BTreeMap::new(),
            velocities: BTreeMap::new(),
            navigation_scratch: Default::default(),
            navigation: scene
                .entities
                .iter()
                .filter(|e| e.npc.is_some() && !crate::templates::vacant(e))
                .map(|e| (e.id.clone(), crate::navigation::NavState::default()))
                .collect(),
            npcs: scene
                .entities
                .iter()
                .filter_map(|e| {
                    if crate::templates::vacant(e) {
                        return None;
                    }
                    e.npc.as_ref().map(|n| {
                        (
                            e.id.clone(),
                            crate::perception::NpcState::with_actor_storage(n, actor_count),
                        )
                    })
                })
                .collect(),
            footsteps: crate::footsteps::Footsteps::new(scene),
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad: BroadPhaseBvh::new(),
            narrow: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse: ImpulseJointSet::new(),
            multibody: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            handles: BTreeMap::new(),
            names: HashMap::new(),
            contact_names: Vec::new(),
            contact_instances: Vec::new(),
            contact_slots: vec![usize::MAX; scene.entities.len()],
            players: Vec::with_capacity(
                scene
                    .entities
                    .iter()
                    .filter(|e| e.body == Some(BodyKind::Player))
                    .count(),
            ),
            ability_ready: vec![0; actor_count],
            deactivated_actors: crate::actor_set::ActorSet::EMPTY,
            contacts: Vec::new(),
            next_contacts: Vec::new(),
            events: VecDeque::new(),
            dropped_events: 0,
            tick: 0,
        };
        let mut entities: Vec<_> = scene.entities.iter().enumerate().collect();
        entities.sort_by_key(|(_, e)| &e.id);
        for (index, e) in entities {
            if e.body.is_none() {
                continue;
            }
            s.contact_slots[index] = s.contact_names.len();
            s.contact_names.push(e.id.clone());
            s.contact_instances.push(s.actors.entity_reference(index));
            if s.actors.entity_reference(index).is_some() {
                s.insert_body(index, e.transform.position);
            }
        }
        s.insert_surfaces();
        s.insert_aerodynamics();
        for point in 0..s.interactions.points.len() {
            if !s.interactions.document.points[point].mutable {
                continue;
            }
            let index = s.interactions.points[point].entity as usize;
            let value = s.interactions.state.objects[point];
            let body = &s.bodies[s.handles[&s.scene.entities[index].id]];
            s.colliders[body.colliders()[0]].set_enabled(value.solid);
            if !value.solid {
                s.interaction_geometry_dirty = true;
            }
            s.scene.set_object(index, value);
        }
        s.flush_interaction_geometry();
        s.positions_before_step = s.positions();
        for (a, &index) in s.actors.actors.iter().enumerate() {
            if s.vitality.dead(a)
                && let Some(n) = s.npcs.get_mut(&scene.entities[index as usize].id)
            {
                n.die();
            }
        }
        s.update_presentation();
        s
    }
    fn insert_surfaces(&mut self) {
        self.surface_bodies.clear();
        for s in &self.physical_surfaces {
            let body = self.bodies.insert(RigidBodyBuilder::fixed());
            let h = self
                .colliders
                .insert_with_parent(s.collider(), body, &mut self.bodies);
            self.broad.set_aabb(
                &IntegrationParameters::default(),
                h,
                self.colliders[h].compute_aabb(),
            );
            self.surface_bodies.insert(s.id.clone(), body);
        }
    }
    fn insert_body(&mut self, index: usize, p: [f32; 3]) {
        let e = &self.scene.entities[index];
        let kind = e.body.unwrap();
        let size = e.transform.size;
        let rb = if kind == BodyKind::Fixed {
            RigidBodyBuilder::fixed()
        } else if e.character_controller.is_some()
            && !self.vitality.dead(self.actors.slot(index) as usize)
        {
            RigidBodyBuilder::kinematic_position_based().lock_rotations()
        } else {
            RigidBodyBuilder::dynamic()
                .lock_rotations()
                .ccd_enabled(true)
        };
        let handle = self
            .bodies
            .insert(rb.translation(vector![p[0], p[1], p[2]]).build());
        let shape = if e.geometry.is_box() {
            ColliderBuilder::cuboid(size[0] / 2., size[1] / 2., size[2] / 2.)
        } else {
            let vertices = e.geometry.vertices(size).map(|p| point![p.x, p.y, p.z]);
            ColliderBuilder::convex_hull(&vertices).expect("Validated nondegenerate wedge")
        };
        let collider = self.colliders.insert_with_parent(
            shape
                .friction(if matches!(kind, BodyKind::Player | BodyKind::Npc) {
                    0.
                } else {
                    0.7
                })
                .restitution(0.)
                .build(),
            handle,
            &mut self.bodies,
        );
        // Queries, including a paused camera at tick0, must see newborn solids.
        self.broad.set_aabb(
            &IntegrationParameters::default(),
            collider,
            self.colliders[collider].compute_aabb(),
        );
        if let Some(config) = e.character_controller {
            if self.character_collisions.capacity() < 20 {
                self.character_collisions.reserve_exact(20);
            }
            let instance = self
                .actors
                .entity_reference(index)
                .expect("present controller owner");
            self.character_controllers.insert(
                e.id.clone(),
                crate::character_controller::State::new(
                    instance,
                    index as u16,
                    !self.vitality.dead(self.actors.slot(index) as usize),
                ),
            );
            self.ground_motors.insert(e.id.clone(), config.compile());
        }
        self.handles.insert(e.id.clone(), handle);
        let c = self.contact_slots[index];
        self.names.insert(collider, c);
        self.contact_names[c] = e.id.clone();
        self.contact_instances[c] = self.actors.entity_reference(index);
        if kind == BodyKind::Player {
            self.players.push(PlayerDrive {
                body: handle,
                actor: self.actors.slot(index) as usize,
                locomotion: e.locomotion.unwrap_or_default(),
            });
        }
    }
    fn can_close_object(&self, index: u16) -> bool {
        let e = &self.scene.entities[index as usize];
        let Some(&handle) = self.handles.get(&e.id) else {
            return false;
        };
        let object = &self.bodies[handle];
        let collider = &self.colliders[object.colliders()[0]];
        !self.bodies.iter().any(|(h, body)| {
            if h == handle || !(body.is_dynamic() || body.is_kinematic()) {
                return false;
            }
            body.colliders().iter().any(|&h| {
                let other = &self.colliders[h];
                let Some(shape) = other.shape().as_cuboid() else {
                    return true;
                };
                let inset = rapier3d::parry::shape::Cuboid::new(
                    shape.half_extents.map(|v| (v - 0.005).max(0.001)),
                );
                rapier3d::parry::query::intersection_test(
                    collider.position(),
                    collider.shape(),
                    other.position(),
                    &inset,
                )
                .unwrap_or(true)
            })
        })
    }
    fn interaction_world<'a>(
        &'a self,
        actor: u16,
        position: &'a dyn Fn(u16) -> Option<[f32; 3]>,
        can_close: &'a dyn Fn(u16) -> bool,
    ) -> crate::interactions::World<'a> {
        crate::interactions::World {
            scene: &self.scene,
            actors: &self.actors,
            vitality: &self.vitality,
            abilities: &self.abilities,
            position,
            can_close,
            actor,
            tick: self.tick,
        }
    }
    pub(crate) fn validate_interaction(
        &self,
        actor: u16,
        request: crate::interactions::Request,
    ) -> crate::model::Result<()> {
        let position = |i: u16| {
            self.scene
                .entities
                .get(i as usize)
                .and_then(|e| self.position(&e.id))
        };
        let can_close = |i| self.can_close_object(i);
        self.interactions.validate_request(
            &self.interaction_world(actor, &position, &can_close),
            request,
        )
    }
    pub(crate) fn inspect_dialogue(&self, actor: u16) -> serde_json::Value {
        let position = |i: u16| {
            self.scene
                .entities
                .get(i as usize)
                .and_then(|e| self.position(&e.id))
        };
        let can_close = |i| self.can_close_object(i);
        self.interactions
            .inspect_dialogue(&self.interaction_world(actor, &position, &can_close))
    }
    pub(crate) fn perform_interaction(
        &mut self,
        actor: u16,
        request: crate::interactions::Request,
        id: Option<u64>,
    ) -> crate::interactions::Outcome {
        let result = {
            let position = |i: u16| {
                self.scene
                    .entities
                    .get(i as usize)
                    .and_then(|e| self.position(&e.id))
            };
            let can_close = |i| self.can_close_object(i);
            self.interactions.prepare(
                &self.interaction_world(actor, &position, &can_close),
                request,
            )
        };
        let (outcome, events) = match result {
            Err(outcome) => (outcome, [None; 8]),
            Ok(plan) => {
                for &(resource, value) in plan.resources.iter() {
                    self.abilities.state.balances[actor as usize][resource as usize] = value;
                }
                for &(point, value) in plan.objects.iter() {
                    let index = self.interactions.points[point as usize].entity as usize;
                    let old = self.interactions.state.objects[point as usize];
                    if old.solid != value.solid {
                        let body = &self.bodies[self.handles[&self.scene.entities[index].id]];
                        self.colliders[body.colliders()[0]].set_enabled(value.solid);
                        self.interaction_geometry_dirty = true;
                    }
                    self.scene.set_object(index, value);
                }
                self.interactions.commit(actor, &plan);
                (plan.outcome, plan.events)
            }
        };
        self.interactions.record(
            self.tick,
            id,
            self.actors.reference(actor),
            request,
            outcome,
            events,
        );
        outcome
    }
    fn flush_interaction_geometry(&mut self) {
        if !self.interaction_geometry_dirty {
            return;
        }
        let previous = self.scene.spatial_network.clone();
        self.scene.regenerate_navigation();
        self.mobility.refresh_geometry(&self.scene, &previous);
        for state in self.navigation.values_mut() {
            state.idle();
        }
        for state in self.npcs.values_mut().filter_map(|n| n.routine.as_mut()) {
            state.failures = [None; 8];
            state.next_think_tick = 0;
        }
        self.interaction_geometry_dirty = false;
    }
    fn process_routines(&mut self, positions: &BTreeMap<String, [f32; 3]>) {
        if self.routines.is_empty() {
            return;
        }
        let count = self.actors.order().len();
        if count == 0 {
            return;
        }
        let start = usize::from(self.interactions.state.auto_cursor) % count;
        let mut actions = 0;
        for offset in 0..count {
            let ordinal = (start + offset) % count;
            let actor = self.actors.order()[ordinal];
            let index = self.actors.entity(actor);
            if self.vitality.dead(actor as usize) {
                continue;
            }
            let Some(npc) = self.scene.entities[index]
                .npc
                .as_ref()
                .filter(|n| n.enabled && n.routine.is_some())
            else {
                continue;
            };
            let auto_dialogue = npc.routine.as_ref().unwrap().auto_dialogue;
            let Some(&own) = positions.get(&self.scene.entities[index].id) else {
                continue;
            };
            let (update, profile) = {
                let e = &self.scene.entities[index];
                let state = self.npcs.get_mut(&e.id).unwrap();
                let profile = state.mind_profile;
                let selected = state.objectives.action == Some(crate::objectives::Kind::Routine)
                    && !state.territory.blocked();
                let Some(routine) = &mut state.routine else {
                    continue;
                };
                let mut update = self.routines.update(
                    crate::npc_routines::Context {
                        scene: &self.scene,
                        interactions: &self.interactions,
                        abilities: &self.abilities,
                        entity: e,
                        actor: actor as usize,
                        position: own,
                        profile,
                        instance: self.actors.reference(actor).instance,
                        navigation: &self.navigation[&e.id],
                        selected,
                        tick: self.tick,
                    },
                    routine,
                );
                if selected {
                    *self.goals.get_mut(&e.id).unwrap() = update.goal.take();
                }
                (update, profile)
            };
            let mut request = update
                .close_dialogue
                .map(|session| crate::interactions::Request::Close { session });
            if request.is_none() && auto_dialogue {
                let routine = self.npcs[&self.scene.entities[index].id].routine.unwrap();
                if self.tick >= routine.next_dialogue_tick
                    && self.npcs[&self.scene.entities[index].id].objectives.action
                        == Some(crate::objectives::Kind::Routine)
                {
                    let p = profile.unwrap_or(crate::npc_mind::Profile {
                        intelligence: 5000,
                        traits: [5000; 9],
                    });
                    let (choice, scores, samples) = {
                        let position = |i: u16| {
                            self.scene
                                .entities
                                .get(i as usize)
                                .and_then(|e| self.position(&e.id))
                        };
                        let close = |i| self.can_close_object(i);
                        self.interactions.choose_dialogue(
                            &self.interaction_world(actor, &position, &close),
                            p,
                            routine.dialogue_cursor,
                        )
                    };
                    let state = self
                        .npcs
                        .get_mut(&self.scene.entities[index].id)
                        .unwrap()
                        .routine
                        .as_mut()
                        .unwrap();
                    state.dialogue_scores = scores;
                    state.dialogue_cursor = state.dialogue_cursor.wrapping_add(samples);
                    state.next_dialogue_tick = self.tick.saturating_add(p.think_interval());
                    request = choice;
                }
            }
            if request.is_none()
                && let Some((point, action)) = update.use_point
                && let Some(instance) = self.interactions.state.anchors[point as usize]
            {
                let life = self
                    .actors
                    .actor_slot(instance)
                    .map(|a| self.vitality.state.lives[a]);
                request = Some(crate::interactions::Request::Use {
                    point,
                    action,
                    instance,
                    life,
                });
            }
            if let Some(request) = request {
                if actions == 16 {
                    self.npcs
                        .get_mut(&self.scene.entities[index].id)
                        .unwrap()
                        .routine
                        .as_mut()
                        .unwrap()
                        .reason = crate::npc_routines::Reason::ActionBudget;
                    continue;
                }
                let outcome = self.perform_interaction(actor, request, None);
                actions += 1;
                self.interactions.state.auto_cursor = ((ordinal + 1) % count) as u16;
                let session = self
                    .interactions
                    .state
                    .sessions
                    .get(actor as usize)
                    .copied()
                    .flatten();
                let state = self
                    .npcs
                    .get_mut(&self.scene.entities[index].id)
                    .unwrap()
                    .routine
                    .as_mut()
                    .unwrap();
                self.routines
                    .feedback(state, session, outcome, self.tick, profile);
            }
        }
        self.flush_interaction_geometry();
    }
    pub(crate) fn validate_population(
        &self,
        entry: crate::population::Entry,
    ) -> crate::model::Result<()> {
        use crate::{model::invalid, population::Operation};
        match entry.operation {
            Operation::Interact {
                actor,
                instance,
                life,
                request,
            } => {
                if !self.actors.matches(actor, instance)
                    || self.vitality.dead(actor as usize)
                    || self.vitality.state.lives[actor as usize] != life
                {
                    return Err(invalid("Interaction requires a current living actor life"));
                }
                self.validate_interaction(actor, request)?;
            }
            Operation::OrderNpc {
                actor,
                instance,
                life,
                order,
            } => {
                order.validate()?;
                if !self.actors.matches(actor, instance)
                    || self.vitality.dead(actor as usize)
                    || self.vitality.state.lives[actor as usize] != life
                    || self.scene.entities[self.actors.entity(actor)].body != Some(BodyKind::Npc)
                {
                    return Err(invalid(
                        "Order requires a current living NPC instance and life",
                    ));
                }
            }
            Operation::ControlGenerator { generator, .. } => {
                if generator as usize >= self.generators.generators.len() {
                    return Err(invalid("Unknown NPC generator"));
                }
            }
            Operation::Despawn { actor, instance } => {
                if !self.actors.matches(actor, instance) {
                    return Err(invalid("Unknown actor instance"));
                }
                if self.scene.entities[self.actors.entity(actor)].body != Some(BodyKind::Npc) {
                    return Err(invalid("Despawn supports NPCs only"));
                }
            }
            Operation::Spawn {
                template,
                position,
                target,
            } => {
                let e = self
                    .scene
                    .npc_templates
                    .get(template as usize)
                    .ok_or_else(|| invalid("Unknown NPC template"))?;
                if position.iter().any(|v| !v.is_finite() || v.abs() > 10000.) {
                    return Err(invalid("Spawn position must be finite within +/-10000"));
                }
                if let Some(r) = target {
                    if !self
                        .actors
                        .actors
                        .iter()
                        .any(|&e| self.actors.entity_matches(e, r))
                    {
                        return Err(invalid("Unknown spawn target instance"));
                    }
                    if e.npc.as_ref().unwrap().targeting.is_some() {
                        return Err(invalid(
                            "Explicit spawn target excludes automatic targeting",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    fn apply_population(&mut self, entries: &[crate::population::Entry]) {
        use crate::population::{Operation, Outcome};
        for &entry in entries {
            let (outcome, result) = if self.validate_population(entry).is_err() {
                (Outcome::InstanceUnavailable, None)
            } else {
                match entry.operation {
                    Operation::Interact {
                        actor,
                        instance,
                        request,
                        ..
                    } => {
                        let outcome =
                            self.perform_interaction(actor, request, Some(entry.sequence));
                        let applied = matches!(
                            outcome,
                            crate::interactions::Outcome::Applied
                                | crate::interactions::Outcome::Opened
                                | crate::interactions::Outcome::Advanced
                                | crate::interactions::Outcome::Closed
                        );
                        (
                            if applied {
                                Outcome::InteractionApplied
                            } else {
                                Outcome::InteractionRejected
                            },
                            Some((actor, instance)),
                        )
                    }
                    Operation::OrderNpc {
                        actor,
                        instance,
                        life,
                        order,
                    } => {
                        let entity = &self.scene.entities[self.actors.entity(actor)];
                        let npc = self.npcs.get_mut(&entity.id).unwrap();
                        let profile = npc.mind_profile.or_else(|| {
                            entity
                                .npc
                                .as_ref()
                                .unwrap()
                                .mind
                                .map(|m| m.resolve(instance.instance))
                        });
                        npc.mind_profile = profile;
                        npc.order = Some(crate::npc_cooperation::State::new(
                            entry.sequence,
                            instance,
                            life,
                            order,
                            self.tick,
                            profile,
                        ));
                        self.navigation.get_mut(&entity.id).unwrap().idle();
                        (Outcome::OrderApplied, Some((actor, instance)))
                    }
                    Operation::ControlGenerator { generator, action } => {
                        let config = &self.scene.npc_generators[generator as usize];
                        (
                            self.generators.generators[generator as usize].control(
                                action,
                                self.tick,
                                config.max_births,
                            ),
                            None,
                        )
                    }
                    Operation::Despawn { actor, instance } => {
                        self.remove_entity(self.actors.entity(actor));
                        (Outcome::Despawned, Some((actor, instance)))
                    }
                    Operation::Spawn {
                        template,
                        position,
                        target,
                    } => match self.spawn(template, position, target) {
                        Ok(r) => (Outcome::Spawned, Some(r)),
                        Err(reason) => (reason, None),
                    },
                }
            };
            self.population.record(entry, outcome, result);
        }
    }
    /// Same strict AABB overlap used by spawn; reads current rigid bodies, including corpses.
    pub(crate) fn spawn_position_occupied(&self, template: u8, p: glam::Vec3) -> bool {
        let half =
            glam::Vec3::from(self.scene.npc_templates[template as usize].transform.size) * 0.5;
        let primitives = self.scene.entities.iter().any(|e| {
            let Some(handle) = self.handles.get(&e.id) else {
                return false;
            };
            let other: [f32; 3] = (*self.bodies[*handle].translation()).into();
            if !e.geometry.is_box() {
                return crate::geometry::Volume::entity(e, other.into())
                    .cast(
                        p,
                        glam::Vec3::ZERO,
                        half - glam::Vec3::splat(0.0001),
                        true,
                        0.,
                    )
                    .is_some();
            }
            let separation = (p - glam::Vec3::from(other)).abs();
            let sum = half + glam::Vec3::from(e.transform.size) * 0.5;
            separation.x < sum.x - 0.0001
                && separation.y < sum.y - 0.0001
                && separation.z < sum.z - 0.0001
        });
        primitives
            || (!self.physical_surfaces.is_empty() && {
                let pose = Isometry::translation(p.x, p.y, p.z);
                let inset = (half - glam::Vec3::splat(0.0001)).max(glam::Vec3::splat(0.00001));
                let shape = rapier3d::parry::shape::Cuboid::new(vector![inset.x, inset.y, inset.z]);
                let surfaces = |h: ColliderHandle, c: &Collider| {
                    c.is_enabled() && !self.names.contains_key(&h)
                };
                self.broad
                    .as_query_pipeline(
                        self.narrow.query_dispatcher(),
                        &self.bodies,
                        &self.colliders,
                        QueryFilter::default().predicate(&surfaces),
                    )
                    .intersect_shape(pose, &shape)
                    .next()
                    .is_some()
            })
    }
    pub(crate) fn spawn(
        &mut self,
        template: u8,
        position: [f32; 3],
        target: Option<crate::identity::Reference>,
    ) -> std::result::Result<(u16, crate::identity::Reference), crate::population::Outcome> {
        use crate::population::Outcome;
        let actor = self
            .actors
            .actors
            .iter()
            .position(|&i| {
                self.scene.entities[i as usize].body == Some(BodyKind::Npc)
                    && self.actors.entity_reference(i as usize).is_none()
            })
            .ok_or(Outcome::Capacity)? as u16;
        let index = self.actors.entity(actor);
        let prototype = &self.scene.npc_templates[template as usize];
        let p = glam::Vec3::from(position);
        let half = glam::Vec3::from(prototype.transform.size) * 0.5;
        if self.spawn_position_occupied(template, p) {
            return Err(Outcome::Occupied);
        }
        let n = prototype.npc.as_ref().unwrap();
        let feet = p - glam::Vec3::Y * half.y;
        let supported = crate::walk_support::query(&self.scene, prototype, feet, feet).continuous()
            || (!self.physical_surfaces.is_empty()
                && crate::surfaces::footprint(
                    &self.scene,
                    &self.physical_surfaces,
                    prototype,
                    feet,
                ) == crate::surfaces::Support::Supported);
        let can_walk = n.mobility.is_none_or(|m| m.walk);
        let can_hover = n.mobility.is_some_and(|m| m.fly_speed.is_some())
            && n.flight
                .unwrap_or(crate::flight::Profile::Straight)
                .settings()
                .is_none_or(|s| s.can_hover);
        if !(supported && can_walk || can_hover) {
            return Err(Outcome::UnsupportedPosition);
        }
        if n.passage_priority.is_some() {
            let participants = self
                .passages
                .agents
                .values()
                .filter(|a| {
                    self.scene.entities[a.entity_index as usize]
                        .npc
                        .as_ref()
                        .unwrap()
                        .surface
                        == n.surface
                })
                .count()
                + 1;
            if self
                .scene
                .passages
                .iter()
                .filter(|p| p.surface == n.surface)
                .any(|p| p.wait_a.len() < participants || p.wait_b.len() < participants)
            {
                return Err(Outcome::PassageCapacity);
            }
        }
        let mut entity = prototype.clone();
        entity.transform.position = position;
        entity.npc.as_mut().unwrap().target =
            target.and_then(|r| self.actors.label(&self.scene, r).map(str::to_owned));
        let reference = self
            .actors
            .insert(actor, entity.faction.as_deref())
            .map_err(|_| Outcome::IdentityExhausted)?;
        entity.id = format!("_ignis_spawn_{}", reference.id());
        self.scene.install(index, entity, reference);
        let e = &self.scene.entities[index];
        let id = e.id.clone();
        let n = e.npc.as_ref().unwrap();
        self.routines.install(&self.scene, e, &self.interactions);
        self.npcs.insert(
            id.clone(),
            crate::perception::NpcState::with_actor_storage(n, self.actors.actors.len()),
        );
        self.navigation.insert(id.clone(), Default::default());
        self.goals.insert(id.clone(), None);
        self.vitality.state.lives[actor as usize] = 0;
        self.vitality.state.actors[actor as usize] = e.vitality.map(|c| crate::vitality::Health {
            units: c.initial,
            maximum: c.maximum,
            scale: c.scale,
        });
        if self.vitality.dead(actor as usize) {
            self.npcs.get_mut(&id).unwrap().die();
        }
        self.deactivated_actors.remove(actor as usize);
        if let Some(c) = &n.patrol {
            let floor = self
                .scene
                .entities
                .iter()
                .find(|e| e.id == n.surface)
                .unwrap();
            let y = floor.transform.position[1]
                + floor.transform.size[1] * 0.5
                + e.transform.size[1] * 0.5;
            self.patrol_routes.insert(id.clone(), c.resolve(y));
            self.patrols.insert(id.clone(), Default::default());
        }
        if n.passage_priority.is_some() {
            self.passages.agents.insert(
                id.clone(),
                crate::passages::Agent {
                    entity_index: index as u16,
                    ..Default::default()
                },
            );
        }
        self.mobility.install(&self.scene, e);
        self.footsteps.install(&self.scene, index);
        self.abilities
            .install_actor(&self.scene, &self.actors, actor, template as usize);
        self.insert_body(index, position);
        self.positions_before_step.insert(id, position);
        Ok((actor, reference))
    }
    #[cfg(test)]
    pub(crate) fn remove_for_test(&mut self, id: &str) {
        let index = self.scene.entities.iter().position(|e| e.id == id).unwrap();
        self.remove_entity(index);
    }
    pub(crate) fn remove_entity(&mut self, index: usize) {
        let actor = self.actors.slot(index);
        if (actor as usize) < crate::targeting::MAX_ACTORS {
            self.abilities
                .detach_origin(&self.actors, &self.vitality, actor, self.tick);
            self.vitality.state.actors[actor as usize] = None;
            self.deactivated_actors.remove(actor as usize);
        }
        self.actors
            .remove(self.actors.entity_reference(index).expect("present entity"));
        let contact = self.contact_slots[index];
        for &(a, b) in &self.contacts {
            if a == contact || b == contact {
                self.events.push_back(ContactEvent {
                    a_instance: self.contact_instances[a].unwrap(),
                    b_instance: self.contact_instances[b].unwrap(),
                    tick: self.tick + 1,
                    kind: "contact_ended".into(),
                    a: self.contact_names[a].clone(),
                    b: self.contact_names[b].clone(),
                });
            }
        }
        self.contacts.retain(|&(a, b)| a != contact && b != contact);
        while self.events.len() > 2048 {
            self.events.pop_front();
            self.dropped_events += 1;
        }
        let id = self.scene.entities[index].id.as_str();
        if let Some(handle) = self.handles.remove(id) {
            self.players.retain(|p| p.body != handle);
            self.bodies.remove(
                handle,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse,
                &mut self.multibody,
                true,
            );
            self.names.retain(|h, _| self.colliders.contains(*h));
        }
        self.positions_before_step.remove(id);
        self.npcs.remove(id);
        self.routines.remove(id);
        self.navigation.remove(id);
        self.goals.remove(id);
        self.velocities.remove(id);
        self.character_controllers.remove(id);
        self.ground_motors.remove(id);
        self.mobility.states.remove(id);
        self.mobility.compiled.remove(id);
        self.mobility.rejection_reasons.remove(id);
        self.mobility.ground_reports.remove(id);
        self.patrol_routes.remove(id);
        self.patrols.remove(id);
        self.footsteps.states.remove(id);
        self.passages.release_actor(id);
        self.passages.agents.remove(id);
    }
    fn move_ground_characters(&mut self) {
        if !self
            .character_controllers
            .values()
            .any(|state| state.active)
        {
            return;
        }
        // Publish initial/new collider AABBs without advancing time or consuming pipeline events.
        // Rapier set_aabb propagates immediately and preserves change detection for physics.
        let params = IntegrationParameters {
            dt: FIXED_DT,
            ..Default::default()
        };
        for (handle, collider) in self.colliders.iter() {
            if !collider.is_enabled() {
                continue;
            }
            self.broad
                .set_aabb(&params, handle, collider.compute_aabb());
        }
        // All movement queries see the same pre-physics positions, including other characters.
        for (id, state) in &mut self.character_controllers {
            if !state.active || state.airborne {
                continue;
            }
            let body_handle = self.handles[id];
            let body = &self.bodies[body_handle];
            let collider = &self.colliders[body.colliders()[0]];
            let config = self.scene.entities[state.entity_index as usize]
                .character_controller
                .unwrap();
            let controller = &self.ground_motors[id];
            let gravity = self.planet.as_ref().map_or(config.gravity, |w| {
                let r = w.config.document.radius();
                (w.config.document.surface_gravity_mps2
                    * (r / (r + w.actor_pose.altitude_meters)).powi(2)) as f32
            });
            state.vertical_velocity =
                (state.vertical_velocity - gravity * FIXED_DT).max(-config.max_fall_speed);
            let desired = vector![
                state.requested_velocity[0],
                state.vertical_velocity,
                state.requested_velocity[1]
            ] * FIXED_DT;
            self.character_collisions.clear();
            let enabled = |_: ColliderHandle, c: &Collider| c.is_enabled();
            let filter = QueryFilter::default()
                .exclude_rigid_body(body_handle)
                .exclude_sensors()
                .predicate(&enabled);
            let movement = controller.move_shape(
                FIXED_DT,
                &self.broad.as_query_pipeline(
                    &crate::character_cast::StableContacts(
                        self.narrow.query_dispatcher(),
                        self.planet.is_some(),
                    ),
                    &self.bodies,
                    &self.colliders,
                    filter,
                ),
                collider.shape(),
                body.position(),
                desired,
                |collision| {
                    if self.character_collisions.len() < 20 {
                        self.character_collisions.push(collision);
                    }
                },
            );
            state.grounded = movement.grounded;
            state.sliding = movement.is_sliding_down_slope;
            if state.grounded {
                state.vertical_velocity = 0.;
            }
            state.desired_translation = desired.into();
            state.corrected_translation = movement.translation.into();
            state.collisions = self.character_collisions.len() as u8;
            let next = *body.translation() + movement.translation;
            if config.push_dynamic_bodies {
                let shape = collider.shared_shape().clone();
                // Unit-density reference mass, independent of kinematic inverse mass (zero).
                let mass = shape.mass_properties(1.).mass();
                controller.solve_character_collision_impulses(
                    FIXED_DT,
                    &mut self.broad.as_query_pipeline_mut(
                        &crate::character_cast::StableContacts(
                            self.narrow.query_dispatcher(),
                            self.planet.is_some(),
                        ),
                        &mut self.bodies,
                        &mut self.colliders,
                        filter,
                    ),
                    shape.as_ref(),
                    mass,
                    &self.character_collisions,
                );
            }
            self.bodies[body_handle].set_next_kinematic_translation(next);
        }
    }
    pub fn step(&mut self, input: Input) {
        self.step_with_noises(input, &[]);
    }
    pub fn step_with_noises(&mut self, input: Input, noises: &[crate::noise::Noise]) {
        self.step_with_commands(input, noises, &[]);
    }
    pub fn step_with_commands(
        &mut self,
        input: Input,
        noises: &[crate::noise::Noise],
        entries: &[crate::abilities::Entry],
    ) {
        self.step_with_population(input, noises, entries, &[]);
    }
    pub(crate) fn step_with_population(
        &mut self,
        input: Input,
        noises: &[crate::noise::Noise],
        entries: &[crate::abilities::Entry],
        population: &[crate::population::Entry],
    ) {
        self.step_with_controls(input, &[], noises, entries, population);
    }
    pub(crate) fn step_with_controls(
        &mut self,
        input: Input,
        controls: &[Option<Input>],
        noises: &[crate::noise::Noise],
        entries: &[crate::abilities::Entry],
        population: &[crate::population::Entry],
    ) {
        self.interactions
            .cleanup(&self.actors, &self.vitality, self.tick);
        self.apply_population(population);
        self.flush_interaction_geometry();
        self.step_generators();
        self.interactions
            .cleanup(&self.actors, &self.vitality, self.tick);
        self.vitality.state.died_this_tick = crate::actor_set::ActorSet::EMPTY;
        self.vitality.state.revived_this_tick = crate::actor_set::ActorSet::EMPTY;
        if !self.routines.is_empty() {
            for (actor, &index) in self.actors.actors.iter().enumerate() {
                if self.vitality.dead(actor) {
                    continue;
                }
                let e = &self.scene.entities[index as usize];
                if let Some(state) = self.npcs.get_mut(&e.id) {
                    self.routines.drift(
                        &e.id,
                        self.actors.reference(actor as u16).instance,
                        actor,
                        state,
                        &mut self.abilities,
                        self.tick,
                    );
                }
            }
        }
        let mut position_snapshot = std::mem::take(&mut self.positions_before_step);
        for (id, p) in &mut position_snapshot {
            if let Some(handle) = self.handles.get(id) {
                *p = (*self.bodies[*handle].translation()).into();
            }
        }
        self.combat_spatial.rebuild(&self.scene, &position_snapshot);
        if self.circulation_enabled {
            self.circulation_velocities.clear();
            self.circulation_velocities
                .extend(self.scene.entities.iter().map(|e| {
                    self.handles.get(&e.id).map_or(glam::Vec3::ZERO, |h| {
                        glam::Vec3::from_array((*self.bodies[*h].linvel()).into())
                    })
                }));
        }
        let positions = &position_snapshot;
        self.abilities
            .state
            .motion
            .fill(crate::abilities::MotionSample::default());
        for (a, &index) in self.actors.actors.iter().enumerate() {
            if !positions.contains_key(&self.scene.entities[index as usize].id) {
                continue;
            }
            let entity = &self.scene.entities[index as usize];
            self.abilities.state.motion[a].speed_squared = self.bodies[self.handles[&entity.id]]
                .linvel()
                .norm_squared();
            let drive = player_input(input, controls, a, self.players.len());
            self.abilities.state.motion[a].requested =
                entity.body == Some(BodyKind::Player) && (drive.x != 0. || drive.z != 0.);
            self.abilities.state.motion[a].lock_supported = self
                .mobility
                .states
                .get(&entity.id)
                .is_none_or(|m| m.can_lock_movement(&self.scene.spatial_network));
        }
        let mut ready = std::mem::take(&mut self.ability_ready);
        self.abilities.fill_ready_masks(self.tick, &mut ready);
        crate::perception::update_with_spatial(
            &self.scene,
            positions,
            self.tick,
            &self.navigation,
            &mut self.npcs,
            &mut self.goals,
            crate::perception::Sensors {
                lives: &self.vitality.state.lives,
                actors: &self.actors,
                ability_ready: &ready,
                ability_policy: Some(crate::perception::AbilityContext {
                    runtime: &self.abilities,
                    health: &self.vitality.state.actors,
                }),
                dead: self.vitality.dead_mask(),
                stimuli: crate::perception::Stimuli {
                    manual: noises,
                    automatic: &self.footsteps.pending,
                    abilities: &self.abilities.state.noises,
                },
            },
            Some(&self.combat_spatial),
            &mut self.report_scratch,
        );
        self.ability_ready = ready;
        self.abilities.step(
            crate::abilities::World {
                scene: &self.scene,
                spatial: Some(&self.combat_spatial),
                vitality: &mut self.vitality,
                actors: &self.actors,
                positions,
                npcs: &self.npcs,
                tick: self.tick,
            },
            entries,
        );
        // Consume every lifecycle transition, including death followed by revival in this tick.
        let changed = self.vitality.state.died_this_tick | self.vitality.state.revived_this_tick;
        for (a, &index) in self.actors.actors.iter().enumerate() {
            if !positions.contains_key(&self.scene.entities[index as usize].id) {
                continue;
            }
            if !changed.contains(a) {
                continue;
            }
            let e = &self.scene.entities[index as usize];
            let id = &e.id;
            self.deactivated_actors.remove(a);
            if let Some(state) = self.character_controllers.get_mut(id) {
                let active = !self.vitality.dead(a);
                *state = crate::character_controller::State::new(state.instance, index, active);
                self.bodies[self.handles[id]].set_body_type(
                    if active {
                        RigidBodyType::KinematicPositionBased
                    } else {
                        RigidBodyType::Dynamic
                    },
                    true,
                );
            }
            self.bodies[self.handles[id]].set_gravity_scale(1., true);
            if let Some(config) = &e.npc {
                let state = self.npcs.get_mut(id).unwrap();
                if self.vitality.dead(a) {
                    state.die();
                } else {
                    let routine = state.routine.map(|mut r| {
                        r.suspend_life();
                        r
                    });
                    let profile = state.mind_profile;
                    *state = crate::perception::NpcState::with_actor_storage(
                        config,
                        self.actors.actors.len(),
                    );
                    state.routine = routine;
                    state.mind_profile = profile;
                }
                *self.goals.get_mut(id).unwrap() = None;
                self.navigation.get_mut(id).unwrap().idle();
                if let Some(m) = self.mobility.states.get_mut(id) {
                    m.revive();
                }
                if let Some(p) = self.patrols.get_mut(id) {
                    *p = Default::default();
                }
                self.passages.release_actor(id);
            }
            if let Some(f) = self.footsteps.states.get_mut(id) {
                f.distance_since_step = 0.;
                f.speed = 0.;
                f.grounded = false;
            }
        }
        self.interactions
            .cleanup(&self.actors, &self.vitality, self.tick);
        self.process_routines(positions);
        let locks =
            self.abilities.movement_locks() | self.interactions.movement_locks(&self.actors);
        for (a, &index) in self.actors.actors.iter().enumerate() {
            if !positions.contains_key(&self.scene.entities[index as usize].id) {
                continue;
            }
            let entity = &self.scene.entities[index as usize];
            if let Some(npc) = self.npcs.get_mut(&entity.id) {
                npc.locomotion_locked = locks.contains(a);
                if npc.locomotion_locked {
                    *self.goals.get_mut(&entity.id).unwrap() = None;
                    self.navigation.get_mut(&entity.id).unwrap().idle();
                    if let Some(m) = self.mobility.states.get_mut(&entity.id) {
                        if entity.character_controller.is_some()
                            && entity.npc.as_ref().is_some_and(|n| n.mobility.is_some())
                        {
                            m.pause_ground_for_cast();
                        } else {
                            m.hold_for_cast();
                        }
                    }
                }
            }
        }
        for player in &self.players {
            if self.vitality.dead(player.actor) {
                continue;
            }
            let drive = if controls.get(player.actor).is_some_and(Option::is_some)
                && self.vitality.state.died_this_tick.contains(player.actor)
            {
                Input::default()
            } else {
                player_input(input, controls, player.actor, self.players.len())
            };
            let profile = player.locomotion.profile(drive.mode);
            let movement = if locks.contains(player.actor) {
                Input::default()
            } else {
                drive
            };
            let n = (movement.x * movement.x + movement.z * movement.z)
                .sqrt()
                .max(1.);
            let id = &self.scene.entities[self.actors.actors[player.actor] as usize].id;
            if let Some(state) = self.character_controllers.get_mut(id) {
                state.requested_velocity = [
                    movement.x / n * profile.speed,
                    movement.z / n * profile.speed,
                ];
            }
            let body = &mut self.bodies[player.body];
            body.set_linvel(
                vector![
                    movement.x / n * profile.speed,
                    body.linvel().y,
                    movement.z / n * profile.speed
                ],
                true,
            );
        }
        for (a, &index) in self.actors.actors.iter().enumerate() {
            if !positions.contains_key(&self.scene.entities[index as usize].id) {
                continue;
            }
            if !self.vitality.dead(a) || self.deactivated_actors.contains(a) {
                continue;
            }
            self.deactivated_actors.insert(a);
            let id = &self.scene.entities[index as usize].id;
            if let Some(state) = self.character_controllers.get_mut(id) {
                state.active = false;
                state.grounded = false;
                state.sliding = false;
                state.requested_velocity = [0.; 2];
                state.desired_translation = [0.; 3];
                state.corrected_translation = [0.; 3];
                state.collisions = 0;
                self.bodies[self.handles[id]].set_body_type(RigidBodyType::Dynamic, true);
            }
            self.bodies[self.handles[id]].set_gravity_scale(1., true);
            if let Some(state) = self.npcs.get_mut(id) {
                state.die();
                *self.goals.get_mut(id).unwrap() = None;
                self.navigation.get_mut(id).unwrap().idle();
                if let Some(m) = self.mobility.states.get_mut(id) {
                    m.die();
                }
            }
        }
        for (id, state) in &self.npcs {
            if state.objectives.changed_this_tick || state.territory.changed_this_tick {
                self.navigation.get_mut(id).unwrap().idle();
            }
        }
        crate::patrol::update(
            &self.scene,
            &self.patrol_routes,
            self.tick,
            &mut self.patrols,
            &self.npcs,
            &self.navigation,
            &mut self.goals,
        );
        for &index in &self.actors.actors {
            let entity = &self.scene.entities[index as usize];
            let Some(npc) = &entity.npc else { continue };
            let Some(state) = self.npcs.get_mut(&entity.id) else {
                continue;
            };
            let action = state
                .objectives
                .active
                .and_then(|i| npc.objectives.as_ref().and_then(|os| os.get(i as usize)))
                .map(|o| o.action);
            let Some(crate::objectives::Action::Tactical { configuration }) = action else {
                state.tactics = None;
                continue;
            };
            if state.locomotion_locked {
                if let Some(t) = &mut state.tactics {
                    t.reason = crate::npc_tactics::Reason::MovementLocked;
                }
                continue;
            }
            if state.territory.blocked() {
                state.tactics = None;
                continue;
            }
            let Some(evidence) = state.memory else {
                state.tactics = None;
                continue;
            };
            let Some(&position) = positions.get(&entity.id) else {
                continue;
            };
            let actor = self.actors.slot(index as usize) as usize;
            let previous = state
                .tactics
                .filter(|_| !state.objectives.changed_this_tick);
            let decision = crate::npc_tactics::choose(
                crate::npc_tactics::Context {
                    scene: &self.scene,
                    index: &self.combat_spatial,
                    ground: self.mobility.ground.as_ref(),
                    entity,
                    position: position.into(),
                    health: self.vitality.state.actors[actor],
                    profile: state.mind_profile,
                    tick: self.tick,
                },
                configuration,
                evidence,
                previous,
            );
            if let Some(choice) = decision
                .choice
                .filter(|choice| npc.territory.is_none_or(|t| t.contains(choice.position)))
            {
                let remaining_travel = if !state.observed_this_tick {
                    match npc.awareness {
                        Some(crate::model::Awareness::Perceptive {
                            max_investigation_distance,
                            ..
                        }) => max_investigation_distance
                            .map(|d| (d - state.investigation_distance).max(0.)),
                        _ => None,
                    }
                } else {
                    None
                };
                *self.goals.get_mut(&entity.id).unwrap() = Some(crate::navigation::NavGoal {
                    position: choice.position,
                    excluded_entity: None,
                    target_half_height: entity.transform.size[1] * 0.5,
                    stop_distance: 0.15,
                    remaining_travel,
                    replan_on_target_change: true,
                });
                state.objectives.destination = Some(choice.position);
                state.objectives.evidence_tick = Some(decision.evidence.tick);
            }
            state.tactics = Some(decision);
        }
        for &index in &self.actors.actors {
            let entity = &self.scene.entities[index as usize];
            let Some(npc) = &entity.npc else { continue };
            let Some(state) = self.npcs.get_mut(&entity.id) else {
                continue;
            };
            let Some(order) = &mut state.order else {
                continue;
            };
            let actor = self.actors.slot(index as usize);
            if !self.actors.matches(actor, order.instance)
                || self.vitality.dead(actor as usize)
                || self.vitality.state.lives[actor as usize] != order.life
            {
                state.order = None;
                continue;
            }
            use crate::npc_cooperation::{Intent, Phase};
            if matches!(order.order.intent, Intent::Cancel) {
                order.phase = Phase::Cancelled;
                continue;
            }
            if self.tick >= order.expires_at {
                order.phase = Phase::Expired;
                continue;
            }
            if self.tick < order.starts_at {
                order.phase = Phase::Reacting;
                continue;
            }
            if state.locomotion_locked || state.territory.blocked() || !npc.enabled {
                order.phase = Phase::Suspended;
                continue;
            }
            let goal = self.goals.get_mut(&entity.id).unwrap();
            match order.order.intent {
                Intent::Hold { .. } => {
                    *goal = None;
                    order.phase = Phase::Holding;
                }
                Intent::Move {
                    position,
                    arrival_distance,
                    ..
                } => {
                    order.phase = if glam::Vec3::from(positions[&entity.id])
                        .distance(position.into())
                        <= arrival_distance
                    {
                        Phase::Arrived
                    } else {
                        Phase::Moving
                    };
                    *goal = Some(crate::navigation::NavGoal {
                        position,
                        excluded_entity: None,
                        target_half_height: entity.transform.size[1] * 0.5,
                        stop_distance: arrival_distance,
                        remaining_travel: None,
                        replan_on_target_change: true,
                    });
                }
                Intent::Cancel => {}
            }
        }
        self.passages.update(
            &self.scene,
            &self.actors,
            positions,
            self.tick,
            &self.npcs,
            &mut self.goals,
            &mut self.navigation,
        );
        crate::navigation::update(
            &self.scene,
            positions,
            self.tick,
            &self.goals,
            &mut self.navigation,
            &mut self.velocities,
            &mut self.navigation_scratch,
        );
        self.passages.gate(
            &self.scene,
            positions,
            &mut self.velocities,
            &mut self.navigation,
        );
        for (id, velocity) in &self.velocities {
            if self.npcs[id].phase == crate::perception::Phase::Dead {
                continue;
            }
            self.npcs.get_mut(id).unwrap().face_velocity(*velocity);
            if let Some(state) = self.character_controllers.get_mut(id) {
                state.requested_velocity = *velocity;
            }
            let body = &mut self.bodies[self.handles[id]];
            body.set_linvel(vector![velocity[0], body.linvel().y, velocity[1]], true);
        }
        for (entity_index, e) in self.scene.entities.iter().enumerate() {
            if !positions.contains_key(&e.id) {
                continue;
            }
            if e.npc.as_ref().is_none_or(|n| n.mobility.is_none())
                || self
                    .npcs
                    .get(&e.id)
                    .is_some_and(|s| s.phase == crate::perception::Phase::Dead)
            {
                continue;
            }
            let body = &mut self.bodies[self.handles[&e.id]];
            if self.npcs[&e.id].locomotion_locked {
                // Only walking/stationary hover can reach this branch; committed air travel was rejected.
                let hover = e
                    .npc
                    .as_ref()
                    .unwrap()
                    .mobility
                    .unwrap()
                    .fly_speed
                    .is_some();
                body.set_gravity_scale(if hover { 0. } else { 1. }, true);
                if let Some(state) = self.character_controllers.get_mut(&e.id) {
                    state.requested_velocity = [0.; 2];
                }
                body.set_linvel(
                    vector![0., if hover { 0. } else { body.linvel().y }, 0.],
                    true,
                );
                continue;
            }
            let c = self.mobility.control_with_circulation(
                &self.scene,
                e,
                self.goals.get(&e.id).and_then(Option::as_ref),
                self.navigation.get_mut(&e.id).unwrap(),
                self.tick,
                glam::Vec3::from_array((*body.translation()).into()),
                glam::Vec3::from_array((*body.linvel()).into()),
                self.character_controllers
                    .get(&e.id)
                    .filter(|s| !s.airborne)
                    .map(|s| s.grounded),
                self.npcs[&e.id].mind_profile,
                e.npc
                    .as_ref()
                    .unwrap()
                    .circulation
                    .map(|_| crate::npc_circulation::Air {
                        index: &self.combat_spatial,
                        velocities: &self.circulation_velocities,
                        entity: entity_index,
                        profile: self.npcs[&e.id].mind_profile,
                        tick: self.tick,
                        decision: &mut self.npcs.get_mut(&e.id).unwrap().circulation,
                    }),
            );
            self.npcs
                .get_mut(&e.id)
                .unwrap()
                .face_spatial_velocity(c.velocity.to_array());
            body.set_gravity_scale(c.gravity, true);
            if let Some(state) = self.character_controllers.get_mut(&e.id) {
                let airborne = c.mode != crate::mobility::Mode::Walk;
                if state.airborne != airborne {
                    state.vertical_velocity = body.linvel().y;
                    body.set_body_type(
                        if airborne {
                            RigidBodyType::Dynamic
                        } else {
                            RigidBodyType::KinematicPositionBased
                        },
                        true,
                    );
                }
                state.airborne = airborne;
                if airborne {
                    state.grounded = false;
                    state.sliding = false;
                    state.desired_translation = [0.; 3];
                    state.corrected_translation = [0.; 3];
                    state.collisions = 0;
                }
                state.requested_velocity = [c.velocity.x, c.velocity.z];
            }
            body.set_linvel(vector![c.velocity.x, c.velocity.y, c.velocity.z], true);
        }
        self.apply_avatar_motion();
        for (a, &index) in self.actors.actors.iter().enumerate() {
            if !positions.contains_key(&self.scene.entities[index as usize].id) {
                continue;
            }
            let entity = &self.scene.entities[index as usize];
            if let Some(npc) = &entity.npc {
                if npc.circulation.is_some() && !self.vitality.dead(a) {
                    let body = &mut self.bodies[self.handles[&entity.id]];
                    let mut velocity = glam::Vec3::from_array((*body.linvel()).into());
                    if let Some(controller) = self.character_controllers.get(&entity.id)
                        && !controller.airborne
                    {
                        // Position-based bodies expose the previous physical velocity;
                        // their new intention lives in the KCC, not in set_linvel.
                        velocity.x = controller.requested_velocity[0];
                        velocity.z = controller.requested_velocity[1];
                    }
                    let state = self.npcs.get_mut(&entity.id).unwrap();
                    let air = self
                        .mobility
                        .states
                        .get(&entity.id)
                        .and_then(|s| s.motor_mode)
                        .is_some_and(|m| m != crate::mobility::Mode::Walk);
                    let decision = if let Some(decision) = state
                        .circulation
                        .filter(|d| d.tick == self.tick && d.air.is_some())
                    {
                        decision
                    } else if air {
                        crate::npc_circulation::State::inactive(
                            self.tick,
                            if velocity.length_squared() < 1e-8 {
                                crate::npc_circulation::Reason::Stationary
                            } else {
                                crate::npc_circulation::Reason::CommittedAir
                            },
                            velocity,
                        )
                    } else if state.locomotion_locked {
                        crate::npc_circulation::State::inactive(
                            self.tick,
                            crate::npc_circulation::Reason::Stationary,
                            velocity,
                        )
                    } else {
                        crate::npc_circulation::steer(
                            &self.scene,
                            &self.combat_spatial,
                            &self.circulation_velocities,
                            index as usize,
                            glam::Vec3::from_array((*body.translation()).into()),
                            velocity,
                            state.mind_profile,
                            self.mobility.ground.as_ref(),
                            self.tick,
                        )
                    };
                    let chosen = decision.chosen;
                    state.circulation = Some(decision);
                    if !air {
                        state.face_velocity([chosen[0], chosen[2]]);
                        body.set_linvel(vector![chosen[0], chosen[1], chosen[2]], true);
                        if let Some(controller) = self.character_controllers.get_mut(&entity.id) {
                            controller.requested_velocity = [chosen[0], chosen[2]];
                        }
                        if let Some(v) = self.velocities.get_mut(&entity.id) {
                            *v = [chosen[0], chosen[2]];
                        }
                    }
                }
                let v = self.bodies[self.handles[&entity.id]].linvel();
                self.abilities.state.motion[a].requested = (self.avatar_motion_enabled
                    && self
                        .presentation
                        .motion
                        .get(&entity.id)
                        .is_some_and(|m| m.last_tick == self.tick && m.intended != [0.; 2]))
                    || if let Some(state) = self.character_controllers.get(&entity.id) {
                        if state.airborne {
                            v.x != 0. || v.y != 0. || v.z != 0.
                        } else {
                            state.requested_velocity != [0.; 2]
                        }
                    } else if npc.mobility.is_some() {
                        v.x != 0. || v.z != 0.
                    } else {
                        self.velocities
                            .get(&entity.id)
                            .is_some_and(|v| v[0] != 0. || v[1] != 0.)
                    };
            }
        }
        self.abilities
            .interrupt_movement(&self.scene, &self.actors, self.tick);
        self.move_ground_characters();
        let positions = &position_snapshot;
        let substeps = if self.wind.bodies.is_empty() {
            1
        } else {
            self.scene.wind.substeps
        };
        self.prepare_wind_kinematics(substeps);
        for substep in 0..substeps {
            if !self.wind.bodies.is_empty() {
                self.advance_wind_kinematics(substep, substeps);
                self.apply_aerodynamics(
                    (self.tick as f64 + f64::from(substep) / f64::from(substeps)) / 60.,
                    f64::from(FIXED_DT) / f64::from(substeps),
                );
            }
            self.pipeline.step(
                &vector![0., -9.81, 0.],
                &IntegrationParameters {
                    dt: FIXED_DT / f32::from(substeps),
                    ..Default::default()
                },
                &mut self.islands,
                &mut self.broad,
                &mut self.narrow,
                &mut self.bodies,
                &mut self.colliders,
                &mut self.impulse,
                &mut self.multibody,
                &mut self.ccd,
                &(),
                &(),
            );
        }
        self.tick += 1;
        for (a, &index) in self.actors.actors.iter().enumerate() {
            if !positions.contains_key(&self.scene.entities[index as usize].id) {
                continue;
            }
            let entity = &self.scene.entities[index as usize];
            let body = &self.bodies[self.handles[&entity.id]];
            let after: [f32; 3] = (*body.translation()).into();
            let displacement = glam::Vec3::from(after) - glam::Vec3::from(positions[&entity.id]);
            self.abilities.state.motion[a].speed_squared = body
                .linvel()
                .norm_squared()
                .max(displacement.length_squared() / (FIXED_DT * FIXED_DT));
        }
        self.abilities
            .interrupt_movement(&self.scene, &self.actors, self.tick);
        self.abilities.state.persistent_areas.sync_centers(|actor| {
            let id = self.actors.id(&self.scene, actor);
            (*self.bodies[self.handles[id]].translation()).into()
        });
        self.triggers.update(self.tick, &self.actors, |id| {
            (*self.bodies[self.handles[id]].translation()).into()
        });
        // Reuse the pre-step snapshot and body handles: no extra snapshot/allocation for budgets.
        for entity in &self.scene.entities {
            if !positions.contains_key(&entity.id) {
                continue;
            }
            let Some(config) = &entity.npc else { continue };
            if self.npcs[&entity.id].phase == crate::perception::Phase::Dead {
                continue;
            }
            let limit = match config.awareness.as_ref() {
                Some(crate::model::Awareness::Perceptive {
                    max_investigation_distance,
                    ..
                }) => *max_investigation_distance,
                _ => None,
            };
            let body = &mut self.bodies[self.handles[&entity.id]];
            let after = body.translation();
            let before = positions[&entity.id];
            let displacement = [
                after.x - before[0],
                if config.mobility.is_some() {
                    after.y - before[1]
                } else {
                    0.
                },
                after.z - before[2],
            ];
            if !self.passages.yielding(&entity.id)
                && self
                    .npcs
                    .get_mut(&entity.id)
                    .unwrap()
                    .record_spatial_travel(displacement, limit, self.tick)
            {
                if config.mobility.is_none() {
                    body.set_linvel(vector![0., body.linvel().y, 0.], true);
                }
                let navigation = self.navigation.get_mut(&entity.id).unwrap();
                navigation.idle();
                navigation.decision_tick = self.tick;
            }
        }
        self.footsteps.pending.clear();
        for (id, state) in &mut self.footsteps.states {
            let entity = &self.scene.entities[state.entity_index as usize];
            let config = entity.footsteps.as_ref().unwrap();
            let body = &self.bodies[self.handles[id]];
            let floor = &self.bodies[self.handles[&config.surface]];
            let surface = &self.scene.entities[state.surface_index as usize];
            let grounded = self.character_controllers.get(id).map_or_else(
                || {
                    self.narrow
                        .contact_pair(body.colliders()[0], floor.colliders()[0])
                        .is_some_and(|pair| pair.has_any_active_contact)
                        && (body.translation().y
                            - entity.transform.size[1] * 0.5
                            - floor.translation().y
                            - surface.transform.size[1] * 0.5)
                            .abs()
                            <= 0.05
                },
                |motor| {
                    motor.active
                        && motor.grounded
                        && (body.translation().y
                            - entity.transform.size[1] * 0.5
                            - floor.translation().y
                            - surface.transform.size[1] * 0.5)
                            .abs()
                            <= entity.character_controller.unwrap().offset + 0.05
                },
            );
            let actor = self.actors.slot(state.entity_index as usize) as usize;
            let drive = player_input(input, controls, actor, self.players.len());
            let intending = !self.vitality.dead(actor)
                && match entity.body {
                    Some(BodyKind::Player) => {
                        !locks.contains(actor)
                            && (drive.x != 0. || drive.z != 0.)
                            && !(controls.get(actor).is_some_and(Option::is_some)
                                && self.vitality.state.died_this_tick.contains(actor))
                    }
                    Some(BodyKind::Npc) => self
                        .velocities
                        .get(id)
                        .is_some_and(|v| v[0] != 0. || v[1] != 0.),
                    _ => false,
                };
            let after = body.translation();
            let before = positions[id];
            if state.advance(
                config,
                [after.x - before[0], after.z - before[2]],
                grounded,
                intending,
            ) {
                let multiplier = if entity.body == Some(BodyKind::Player) {
                    entity
                        .locomotion
                        .unwrap_or_default()
                        .profile(drive.mode)
                        .noise_multiplier
                } else {
                    1.
                };
                if multiplier == 0. {
                    continue;
                }
                if self.footsteps.total_emitted >= crate::noise::MAX_SERIAL {
                    self.footsteps.dropped_recent = self.footsteps.dropped_recent.saturating_add(1);
                    continue;
                }
                state.emitted += 1;
                let noise = crate::noise::Noise {
                    emitter_instance: self.actors.entity_reference(state.entity_index as usize),
                    id: crate::noise::event_id(
                        crate::noise::Domain::Footstep,
                        self.footsteps.total_emitted,
                    ),
                    tick: self.tick,
                    position: [after.x, after.y, after.z],
                    radius: (config.radius * multiplier).clamp(0.1, 100.),
                    emitter_index: Some(state.entity_index),
                };
                self.footsteps.total_emitted += 1;
                self.footsteps.pending.push(noise);
                if self.footsteps.recent.len() == crate::footsteps::RECENT_CAPACITY {
                    self.footsteps.recent.pop_front();
                    self.footsteps.dropped_recent += 1;
                }
                self.footsteps.recent.push_back(noise);
            }
        }
        self.next_contacts.clear();
        for pair in self
            .narrow
            .contact_pairs()
            .filter(|p| p.has_any_active_contact)
        {
            if let (Some(a), Some(b)) = (
                self.names.get(&pair.collider1),
                self.names.get(&pair.collider2),
            ) {
                self.next_contacts
                    .push(if a < b { (*a, *b) } else { (*b, *a) });
            }
        }
        // Numeric indices follow sorted IDs; event order stays lexicographic.
        self.next_contacts.sort_unstable();
        self.next_contacts.dedup();
        for (current, previous, kind) in [
            (&self.next_contacts, &self.contacts, "contact_started"),
            (&self.contacts, &self.next_contacts, "contact_ended"),
        ] {
            for &(a, b) in current {
                if previous.binary_search(&(a, b)).is_err() {
                    self.events.push_back(ContactEvent {
                        a_instance: self.contact_instances[a].unwrap(),
                        b_instance: self.contact_instances[b].unwrap(),
                        tick: self.tick,
                        kind: kind.into(),
                        a: self.contact_names[a].clone(),
                        b: self.contact_names[b].clone(),
                    });
                }
            }
        }
        while self.events.len() > 2048 {
            self.events.pop_front();
            self.dropped_events += 1;
        }
        std::mem::swap(&mut self.contacts, &mut self.next_contacts);
        self.positions_before_step = position_snapshot;
        self.update_presentation();
    }
    pub fn movement_profile(
        &self,
        mode: crate::model::MovementMode,
    ) -> crate::model::MovementProfile {
        self.players
            .first()
            .filter(|_| self.players.len() == 1)
            .map(|p| p.locomotion)
            .unwrap_or_default()
            .profile(mode)
    }
    pub fn player_count(&self) -> usize {
        self.players.len()
    }
    pub fn definitions(&self) -> &Scene {
        self.scene.definitions()
    }
    pub(crate) fn reference_for_presentation(
        &self,
        id: &str,
    ) -> Option<(crate::identity::Reference, u64)> {
        let a = self.actor_slot_by_id(id)? as usize;
        Some((
            self.actors.reference(a as u16),
            self.vitality.state.lives[a],
        ))
    }
    pub(crate) fn velocity_for_presentation(&self, id: &str) -> [f32; 3] {
        self.handles
            .get(id)
            .map_or([0.; 3], |h| (*self.bodies[*h].linvel()).into())
    }
    pub fn position(&self, id: &str) -> Option<[f32; 3]> {
        self.scene.position(id, &self.actors, || {
            self.handles
                .get(id)
                .map(|h| (*self.bodies[*h].translation()).into())
        })
    }
    pub(crate) fn actor_slot_by_id(&self, id: &str) -> Option<u16> {
        self.scene
            .reference(id)
            .and_then(|r| self.actors.actor_slot(r))
            .map(|a| a as u16)
    }
    pub fn positions(&self) -> BTreeMap<String, [f32; 3]> {
        self.scene.positions(&self.actors, |id| {
            self.handles
                .get(id)
                .map(|h| (*self.bodies[*h].translation()).into())
        })
    }
    pub fn contacts(&self) -> BTreeSet<(String, String)> {
        self.contacts
            .iter()
            .map(|&(a, b)| (self.contact_names[a].clone(), self.contact_names[b].clone()))
            .collect()
    }
}
