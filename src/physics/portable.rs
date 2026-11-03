//! Canonical Ignis primaries. This payload contains no physics backend type.
use super::*;
use crate::model::{Result, invalid};
use rapier3d::na::{Quaternion, UnitQuaternion};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Saved {
    #[serde(default)]
    wind: Vec<super::wind::Primary>,
    world: crate::runtime_world::RuntimeWorld,
    actors: crate::targeting::Catalog,
    abilities: crate::abilities::SavedRuntime,
    vitality: crate::vitality::State,
    npcs: BTreeMap<String, crate::perception::NpcState>,
    population: crate::population::State,
    generators: crate::generators::State,
    interactions: crate::interactions::State,
    presentation: crate::npc_presentation::StateSet,
    navigation: BTreeMap<String, crate::navigation::NavState>,
    passages: crate::passages::Runtime,
    mobility: BTreeMap<String, crate::mobility::State>,
    controllers: BTreeMap<String, crate::character_controller::State>,
    footsteps: crate::footsteps::Footsteps,
    patrols: BTreeMap<String, crate::patrol::State>,
    patrol_routes: BTreeMap<String, Vec<[f32; 3]>>,
    triggers: crate::trigger::Runtime,
    goals: BTreeMap<String, Option<crate::navigation::NavGoal>>,
    velocities: BTreeMap<String, [f32; 2]>,
    contact_names: Vec<String>,
    contact_instances: Vec<Option<crate::identity::Reference>>,
    contact_slots: Vec<Option<u16>>,
    deactivated: crate::actor_set::ActorSet,
    contacts: Vec<(u16, u16)>,
    next_contacts: Vec<(u16, u16)>,
    events: VecDeque<ContactEvent>,
    dropped_events: u64,
    tick: u64,
    physical: Vec<Body>,
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Motor {
    Fixed,
    Dynamic,
    KinematicPosition,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Body {
    id: String,
    instance: crate::identity::Reference,
    motor: Motor,
    position: [f32; 3],
    rotation: [f32; 4],
    next_position: [f32; 3],
    next_rotation: [f32; 4],
    linear_velocity: [f32; 3],
    angular_velocity: [f32; 3],
    force: [f32; 3],
    torque: [f32; 3],
    gravity: f32,
    ccd: bool,
    enabled: bool,
    collider_enabled: bool,
    sleeping: bool,
}
impl Simulation {
    pub(crate) fn portable_checkpoint(&self) -> Result<Saved> {
        if !self.impulse.is_empty() || self.multibody.iter().next().is_some() {
            return Err(invalid(
                "Portable save: joints have no canonical contract yet",
            ));
        }
        let mut physical = Vec::with_capacity(self.handles.len());
        for (id, handle) in &self.handles {
            let b = &self.bodies[*handle];
            if b.colliders().len() != 1 {
                return Err(invalid(format!(
                    "Portable save: {id}: expected one canonical collider"
                )));
            }
            let motor = match b.body_type() {
                RigidBodyType::Fixed => Motor::Fixed,
                RigidBodyType::Dynamic => Motor::Dynamic,
                RigidBodyType::KinematicPositionBased => Motor::KinematicPosition,
                _ => return Err(invalid(format!("Portable save: {id}: unsupported motor"))),
            };
            physical.push(Body {
                id: id.clone(),
                instance: self
                    .scene
                    .reference(id)
                    .ok_or_else(|| invalid("Portable body identity"))?,
                motor,
                position: (*b.translation()).into(),
                rotation: b.rotation().coords.into(),
                next_position: b.next_position().translation.vector.into(),
                next_rotation: b.next_position().rotation.coords.into(),
                linear_velocity: (*b.linvel()).into(),
                angular_velocity: (*b.angvel()).into(),
                force: b.user_force().into(),
                torque: b.user_torque().into(),
                gravity: b.gravity_scale(),
                ccd: b.is_ccd_enabled(),
                enabled: b.is_enabled(),
                collider_enabled: self.colliders[b.colliders()[0]].is_enabled(),
                sleeping: b.is_sleeping(),
            });
        }
        Ok(Saved {
            wind: self.wind_primaries(),
            world: self.scene.clone(),
            actors: self.actors.clone(),
            abilities: self.abilities.checkpoint(),
            vitality: self.vitality.state.clone(),
            npcs: self.npcs.clone(),
            population: self.population.clone(),
            generators: self.generators.clone(),
            interactions: self.interactions.state.clone(),
            presentation: self.presentation.clone(),
            navigation: self.navigation.clone(),
            passages: self.passages.clone(),
            mobility: self.mobility.states.clone(),
            controllers: self.character_controllers.clone(),
            footsteps: self.footsteps.clone(),
            patrols: self.patrols.clone(),
            patrol_routes: self.patrol_routes.clone(),
            triggers: self.triggers.clone(),
            goals: self.goals.clone(),
            velocities: self.velocities.clone(),
            contact_names: self.contact_names.clone(),
            contact_instances: self.contact_instances.clone(),
            contact_slots: self
                .contact_slots
                .iter()
                .map(|&c| (c != usize::MAX).then_some(c as u16))
                .collect(),
            deactivated: self.deactivated_actors,
            contacts: self
                .contacts
                .iter()
                .map(|&(a, b)| (a as u16, b as u16))
                .collect(),
            next_contacts: self
                .next_contacts
                .iter()
                .map(|&(a, b)| (a as u16, b as u16))
                .collect(),
            events: self.events.clone(),
            dropped_events: self.dropped_events,
            tick: self.tick,
            physical,
        })
    }
}
impl Saved {
    pub(crate) fn tick(&self) -> u64 {
        self.tick
    }
    pub(crate) fn restore(self, author: &Scene) -> Result<Simulation> {
        self.actors.validate_checkpoint(self.world.definitions())?;
        self.world.validate_checkpoint(&self.actors, author)?;
        let entities = self.world.entities.len();
        let actors = self.actors.actors.len();
        if self.physical.len() > entities
            || self.tick > crate::engine::MAX_RUNTIME_TICK
            || self.npcs.len() > author.npc_capacity as usize
            || self.vitality.actors.len() != actors
            || self.vitality.lives.len() != actors
            || self.contact_slots.len() != entities
            || self.contact_instances.len() != self.contact_names.len()
            || self.contact_names.len() > entities
            || self.events.len() > 2048
            || self.goals.len() > self.npcs.len()
            || self.velocities.len() > self.npcs.len()
            || self.generators.generators.len() != author.npc_generators.len()
        {
            return Err(invalid("Portable save dimensions/identity budget"));
        }
        let mut out = Simulation::new(author);
        out.scene = self.world;
        out.actors = self.actors;
        out.vitality.state = self.vitality;
        out.abilities = crate::abilities::Runtime::with_actors(&out.scene, &out.actors);
        out.abilities
            .restore_checkpoint(self.abilities, &out.scene, &out.actors)?;
        let expanded = crate::templates::expand(author);
        out.interactions = crate::interactions::Runtime::new(&expanded, &out.actors);
        out.interactions.state = self.interactions;
        out.routines = crate::npc_routines::Runtime::new(&out.scene, &out.interactions);
        out.mobility = crate::mobility::Runtime::new(&out.scene);
        out.mobility.states = self.mobility;
        out.npcs = self.npcs;
        out.population = self.population;
        out.generators = self.generators;
        out.presentation = self.presentation;
        out.navigation = self.navigation;
        out.passages = self.passages;
        out.character_controllers = self.controllers;
        out.footsteps = self.footsteps;
        out.patrols = self.patrols;
        out.patrol_routes = self.patrol_routes;
        out.triggers = self.triggers;
        out.goals = self.goals;
        out.velocities = self.velocities;
        out.contact_names = self.contact_names;
        out.contact_instances = self.contact_instances;
        out.contact_slots = self
            .contact_slots
            .into_iter()
            .map(|c| c.map_or(usize::MAX, usize::from))
            .collect();
        out.deactivated_actors = self.deactivated;
        out.contacts = self
            .contacts
            .into_iter()
            .map(|(a, b)| (usize::from(a), usize::from(b)))
            .collect();
        out.next_contacts = self
            .next_contacts
            .into_iter()
            .map(|(a, b)| (usize::from(a), usize::from(b)))
            .collect();
        out.events = self.events;
        out.dropped_events = self.dropped_events;
        out.tick = self.tick;
        out.bodies = RigidBodySet::new();
        out.colliders = ColliderSet::new();
        out.islands = IslandManager::new();
        out.broad = BroadPhaseBvh::new();
        out.narrow = NarrowPhase::new();
        out.impulse = ImpulseJointSet::new();
        out.multibody = MultibodyJointSet::new();
        out.handles.clear();
        out.names.clear();
        out.players.clear();
        let mut seen = BTreeSet::new();
        for primary in self.physical {
            primary.install(&mut out, author, &mut seen)?;
        }
        for (index, e) in out.scene.entities.iter().enumerate() {
            let expected = out.actors.entity_reference(index).is_some()
                && (e.body.is_some()
                    || author
                        .interactions
                        .points
                        .iter()
                        .any(|p| p.mutable && p.entity == e.id));
            if expected != out.handles.contains_key(&e.id) {
                return Err(invalid(format!("Portable save body coverage: {}", e.id)));
            }
        }
        out.insert_surfaces();
        out.insert_aerodynamics();
        out.restore_wind_primaries(self.wind)?;
        // Queries (including IK at prefix zero) must work before the first tick.
        // Publish bounds only; no simulation step, contacts or gameplay events.
        let params = IntegrationParameters {
            dt: FIXED_DT,
            ..Default::default()
        };
        for (handle, collider) in out.colliders.iter() {
            if collider.is_enabled() {
                out.broad.set_aabb(&params, handle, collider.compute_aabb());
            }
        }
        // Reuse full domain validation and derived-cache reconstruction. This snapshot
        // exists only in memory on the destination; no native bytes are read from file.
        Simulation::from_checkpoint(author, out.checkpoint())
    }
}
impl Body {
    fn install(
        self,
        out: &mut Simulation,
        author: &Scene,
        seen: &mut BTreeSet<String>,
    ) -> Result<()> {
        let fail = || {
            invalid(format!(
                "Portable save body {}: invalid identity/primaries",
                self.id
            ))
        };
        let index = out
            .scene
            .entities
            .iter()
            .position(|e| e.id == self.id)
            .ok_or_else(fail)?;
        let e = &out.scene.entities[index];
        let fixed = e.body == Some(BodyKind::Fixed)
            || author
                .interactions
                .points
                .iter()
                .any(|p| p.mutable && p.entity == e.id);
        if !seen.insert(self.id.clone())
            || out.actors.entity_reference(index) != Some(self.instance)
            || (self.motor == Motor::Fixed) != fixed
            || (self.motor == Motor::KinematicPosition && e.character_controller.is_none())
            || self
                .position
                .iter()
                .chain(&self.next_position)
                .any(|x| !x.is_finite())
            || self
                .linear_velocity
                .iter()
                .chain(&self.angular_velocity)
                .chain(&self.force)
                .chain(&self.torque)
                .any(|x| !x.is_finite())
            || !self.gravity.is_finite()
            || !(0. ..=1.).contains(&self.gravity)
        {
            return Err(fail());
        }
        let rotation = |q: [f32; 4]| -> Result<UnitQuaternion<f32>> {
            let q = Quaternion::new(q[3], q[0], q[1], q[2]);
            if !q.norm_squared().is_finite() || (q.norm_squared() - 1.).abs() > 0.00001 {
                return Err(fail());
            }
            Ok(UnitQuaternion::new_unchecked(q))
        };
        let pose = Isometry::from_parts(self.position.into(), rotation(self.rotation)?);
        let next = Isometry::from_parts(self.next_position.into(), rotation(self.next_rotation)?);
        let builder = match self.motor {
            Motor::Fixed => RigidBodyBuilder::fixed(),
            Motor::Dynamic => RigidBodyBuilder::dynamic()
                .lock_rotations()
                .ccd_enabled(true),
            Motor::KinematicPosition => {
                RigidBodyBuilder::kinematic_position_based().lock_rotations()
            }
        };
        let mut body = builder
            .pose(pose)
            .linvel(self.linear_velocity.into())
            .angvel(self.angular_velocity.into())
            .ccd_enabled(self.ccd)
            .gravity_scale(self.gravity)
            .enabled(self.enabled)
            .build();
        body.add_force(self.force.into(), false);
        body.add_torque(self.torque.into(), false);
        if self.motor == Motor::KinematicPosition {
            body.set_next_kinematic_position(next);
        }
        let handle = out.bodies.insert(body);
        let size = e.transform.size;
        let shape = if e.geometry.is_box() {
            ColliderBuilder::cuboid(size[0] / 2., size[1] / 2., size[2] / 2.)
        } else {
            let vertices = e.geometry.vertices(size).map(|p| point![p.x, p.y, p.z]);
            ColliderBuilder::convex_hull(&vertices).ok_or_else(fail)?
        };
        let collider = out.colliders.insert_with_parent(
            shape
                .friction(
                    if matches!(e.body, Some(BodyKind::Player | BodyKind::Npc)) {
                        0.
                    } else {
                        0.7
                    },
                )
                .restitution(0.)
                .enabled(self.collider_enabled)
                .build(),
            handle,
            &mut out.bodies,
        );
        if self.sleeping {
            out.bodies[handle].sleep();
        }
        let contact = *out.contact_slots.get(index).ok_or_else(fail)?;
        if out.contact_names.get(contact) != Some(&self.id)
            || out.contact_instances.get(contact) != Some(&Some(self.instance))
        {
            return Err(fail());
        }
        out.names.insert(collider, contact);
        out.handles.insert(self.id.clone(), handle);
        if e.body == Some(BodyKind::Player) {
            out.players.push(PlayerDrive {
                body: handle,
                actor: out.actors.slot(index) as usize,
                locomotion: e.locomotion.unwrap_or_default(),
            });
        }
        Ok(())
    }
}
