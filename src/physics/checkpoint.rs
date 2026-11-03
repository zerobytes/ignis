//! Native checkpoint primaries; transient executors/lookup caches are rebuilt.
use super::*;
use crate::model::{Result, invalid};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Saved {
    #[serde(default)]
    wind_handles: Vec<RigidBodyHandle>,
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
    islands: IslandManager,
    broad: BroadPhaseBvh,
    narrow: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse: ImpulseJointSet,
    multibody: MultibodyJointSet,
    handles: BTreeMap<String, RigidBodyHandle>,
    surface_bodies: BTreeMap<String, RigidBodyHandle>,
    names: HashMap<ColliderHandle, usize>,
    contact_names: Vec<String>,
    contact_instances: Vec<Option<crate::identity::Reference>>,
    contact_slots: Vec<usize>,
    players: Vec<PlayerDrive>,
    deactivated: crate::actor_set::ActorSet,
    contacts: Vec<(usize, usize)>,
    next_contacts: Vec<(usize, usize)>,
    events: VecDeque<ContactEvent>,
    dropped_events: u64,
    tick: u64,
}
impl Simulation {
    pub(crate) fn checkpoint(&self) -> Saved {
        Saved {
            wind_handles: self.wind_handles(),
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
            islands: self.islands.clone(),
            broad: self.broad.clone(),
            narrow: self.narrow.clone(),
            bodies: self.bodies.clone(),
            colliders: self.colliders.clone(),
            impulse: self.impulse.clone(),
            multibody: self.multibody.clone(),
            handles: self.handles.clone(),
            surface_bodies: self.surface_bodies.clone(),
            names: self.names.clone(),
            contact_names: self.contact_names.clone(),
            contact_instances: self.contact_instances.clone(),
            contact_slots: self.contact_slots.clone(),
            players: self.players.clone(),
            deactivated: self.deactivated_actors,
            contacts: self.contacts.clone(),
            next_contacts: self.next_contacts.clone(),
            events: self.events.clone(),
            dropped_events: self.dropped_events,
            tick: self.tick,
        }
    }
    pub(crate) fn from_checkpoint(author: &Scene, data: Saved) -> Result<Self> {
        data.validate(author)?;
        let mut out = Self::new(author);
        out.scene = data.world;
        let network = out.scene.spatial_network.clone();
        out.scene.regenerate_navigation();
        if out.scene.spatial_network != network {
            return Err(invalid(
                "Checkpoint generated navigation differs after validated rebuild",
            ));
        }
        out.actors = data.actors;
        out.abilities = crate::abilities::Runtime::with_actors(&out.scene, &out.actors);
        out.abilities
            .restore_checkpoint(data.abilities, &out.scene, &out.actors)?;
        let expanded = crate::templates::expand(author);
        out.interactions = crate::interactions::Runtime::new(&expanded, &out.actors);
        out.interactions.state = data.interactions;
        out.routines = crate::npc_routines::Runtime::new(&out.scene, &out.interactions);
        out.vitality.state = data.vitality;
        out.npcs = data.npcs;
        out.population = data.population;
        out.generators = data.generators;
        out.presentation = data.presentation;
        out.mobility = crate::mobility::Runtime::new(&out.scene);
        out.mobility.states = data.mobility;
        out.navigation = data.navigation;
        out.passages = data.passages;
        out.character_controllers = data.controllers;
        out.footsteps = data.footsteps;
        out.patrols = data.patrols;
        out.patrol_routes = data.patrol_routes;
        out.triggers = data.triggers;
        out.goals = data.goals;
        out.velocities = data.velocities;
        out.islands = data.islands;
        out.broad = data.broad;
        out.narrow = data.narrow;
        out.bodies = data.bodies;
        for (b, h) in out.wind.bodies.iter_mut().zip(data.wind_handles) {
            b.handle = h;
        }
        out.colliders = data.colliders;
        out.surface_bodies = data.surface_bodies;
        out.impulse = data.impulse;
        out.multibody = data.multibody;
        out.handles = data.handles;
        out.names = data.names;
        out.contact_names = data.contact_names;
        out.contact_instances = data.contact_instances;
        out.contact_slots = data.contact_slots;
        out.players = data.players;
        out.deactivated_actors = data.deactivated;
        out.contacts = data.contacts;
        out.next_contacts = data.next_contacts;
        out.events = data.events;
        out.dropped_events = data.dropped_events;
        out.tick = data.tick;
        out.ground_motors.clear();
        for (i, e) in out.scene.entities.iter().enumerate() {
            if out.actors.entity_reference(i).is_some()
                && let Some(c) = e.character_controller
            {
                out.ground_motors.insert(e.id.clone(), c.compile());
            }
        }
        out.positions_before_step = out.positions();
        out.combat_spatial
            .rebuild(&out.scene, &out.positions_before_step);
        Ok(out)
    }
}
impl Saved {
    pub(crate) fn tick(&self) -> u64 {
        self.tick
    }
    fn validate(&self, author: &Scene) -> Result<()> {
        self.actors.validate_checkpoint(self.world.definitions())?;
        self.world.validate_checkpoint(&self.actors, author)?;
        let defs = self.world.definitions();
        let entities = defs.entities.len();
        let actors = self.actors.actors.len();
        let contacts = self.contact_names.len();
        let invalid = || invalid("Checkpoint runtime dimensions/identity");
        if self.tick > crate::engine::MAX_RUNTIME_TICK
            || self.npcs.len() > author.npc_capacity as usize
            || self.vitality.actors.len() != actors
            || self.vitality.lives.len() != actors
            || self.contact_slots.len() != entities
            || self.contact_instances.len() != contacts
            || contacts > entities
            || self.handles.len() > entities
            || self.events.len() > 2048
            || self.players.len() > crate::targeting::MAX_PLAYERS
            || self.goals.len() > self.npcs.len()
            || self.velocities.len() > self.npcs.len()
            || self.generators.generators.len() != author.npc_generators.len()
        {
            return Err(invalid());
        }
        for (a, &i) in self.actors.actors.iter().enumerate() {
            let e = &defs.entities[i as usize];
            let r = self.actors.entity_reference(i as usize);
            if let Some(h) = self.vitality.actors[a]
                && (h.units < 0
                    || h.units > h.maximum
                    || h.maximum > crate::abilities::LIMIT
                    || h.maximum <= 0
                    || h.scale == 0)
            {
                return Err(invalid());
            }
            if let Some(s) = self.npcs.get(&e.id) {
                if r.is_none()
                    || e.npc.is_none()
                    || s.known_instances.len() != actors
                    || s.known_lives.len() != actors
                    || s.knowledge.tracks.len() > actors
                    || s.support.observations.len() > actors
                    || s.support.knowledge.tracks.len() > actors
                    || !self.navigation.contains_key(&e.id)
                {
                    return Err(invalid());
                }
            } else if r.is_some() && e.npc.is_some() {
                return Err(invalid());
            }
        }
        for (name, handle) in &self.handles {
            let body = self.bodies.get(*handle).ok_or_else(invalid)?;
            if !defs.entities.iter().any(|e| e.id == *name)
                || self
                    .world
                    .position(name, &self.actors, || Some((*body.translation()).into()))
                    .is_none()
                || body
                    .translation()
                    .iter()
                    .chain(body.linvel().iter())
                    .any(|x| !x.is_finite())
                || body
                    .colliders()
                    .iter()
                    .any(|h| self.colliders.get(*h).is_none())
            {
                return Err(invalid());
            }
        }
        let aero_colliders = super::wind::validate_saved_handles(
            author,
            &self.wind_handles,
            &self.wind,
            &self.bodies,
            &self.colliders,
        )?;
        let surfaces = crate::surfaces::prepare(defs);
        if self.surface_bodies.len() != surfaces.len()
            || self.bodies.len() != self.handles.len() + surfaces.len() + self.wind_handles.len()
        {
            return Err(invalid());
        }
        let mut surface_colliders = std::collections::HashSet::new();
        for s in surfaces {
            let body = self
                .surface_bodies
                .get(&s.id)
                .and_then(|h| self.bodies.get(*h))
                .ok_or_else(invalid)?;
            if !body.is_fixed()
                || !body.is_enabled()
                || *body.position() != Isometry::identity()
                || body.colliders().len() != 1
            {
                return Err(invalid());
            }
            let h = body.colliders()[0];
            let c = self.colliders.get(h).ok_or_else(invalid)?;
            let mesh = c.shape().as_trimesh().ok_or_else(invalid)?;
            let expected = s.shape.as_trimesh().unwrap();
            if !c.is_enabled()
                || c.is_sensor()
                || *c.position() != s.pose
                || mesh.vertices() != expected.vertices()
                || mesh.indices() != expected.indices()
                || !surface_colliders.insert(h)
            {
                return Err(invalid());
            }
        }
        for (h, c) in self.colliders.iter() {
            if c.parent().is_none_or(|b| self.bodies.get(b).is_none())
                || (self.names.get(&h).is_none_or(|i| *i >= contacts)
                    && !surface_colliders.contains(&h)
                    && !aero_colliders.contains(&h))
            {
                return Err(invalid());
            }
        }
        for p in &self.players {
            if p.actor >= actors || self.bodies.get(p.body).is_none() {
                return Err(invalid());
            }
        }
        if self
            .contacts
            .iter()
            .chain(&self.next_contacts)
            .any(|(a, b)| *a >= contacts || *b >= contacts)
        {
            return Err(invalid());
        }
        for (i, g) in self.generators.generators.iter().enumerate() {
            if g.members.len() != crate::templates::capacity(author)
                || g.births > author.npc_generators[i].max_births
                || g.members.iter().flatten().any(|m| {
                    m.actor as usize >= actors || m.template as usize >= author.npc_templates.len()
                })
            {
                return Err(invalid());
            }
        }
        if self.interactions.values.len() != author.interactions.values.len()
            || self.interactions.objects.len() != author.interactions.points.len()
            || self.interactions.anchors.len() != author.interactions.points.len()
            || self.interactions.sessions.len()
                != if author.interactions.dialogues.is_empty() {
                    0
                } else {
                    actors
                }
            || self.interactions.cooldowns.len()
                != if author.interactions.actions.is_empty() {
                    0
                } else {
                    actors
                }
        {
            return Err(invalid());
        }
        for (i, p) in author.interactions.points.iter().enumerate() {
            if p.mutable {
                let e = defs
                    .entities
                    .iter()
                    .find(|e| e.id == p.entity)
                    .ok_or_else(invalid)?;
                let state = self.interactions.objects[i];
                let body = self
                    .handles
                    .get(&e.id)
                    .and_then(|h| self.bodies.get(*h))
                    .ok_or_else(invalid)?;
                if e.body.is_some() != state.solid
                    || e.visible != Some(state.visible)
                    || e.blocks_vision != Some(state.blocks_vision)
                    || e.sound_transmission != Some(state.sound_transmission)
                    || body
                        .colliders()
                        .first()
                        .and_then(|h| self.colliders.get(*h))
                        .is_none_or(|c| c.is_enabled() != state.solid)
                {
                    return Err(invalid());
                }
            }
        }
        for s in self.interactions.sessions.iter().flatten() {
            let d = author
                .interactions
                .dialogues
                .get(s.dialogue as usize)
                .ok_or_else(invalid)?;
            if s.node as usize >= d.nodes.len()
                || s.serial >= self.interactions.next_session
                || s.anchor.entity as usize >= entities
            {
                return Err(invalid());
            }
        }
        for (id, s) in &self.controllers {
            let slot = self.actors.actor_slot(s.instance).ok_or_else(invalid)?;
            if self.actors.id(defs, slot as u16) != id
                || defs.entities[self.actors.entity(slot as u16)]
                    .character_controller
                    .is_none()
            {
                return Err(invalid());
            }
        }
        crate::avatar_motion::validate_saved(
            &self.presentation.motion,
            defs,
            &self.actors,
            &self.vitality.lives,
            self.tick,
        )?;
        for (id, s) in &self.presentation.actors {
            let a = self.actors.actor_slot(s.instance).ok_or_else(invalid)?;
            let c = defs.entities[self.actors.entity(a as u16)]
                .avatar
                .as_ref()
                .ok_or_else(invalid)?;
            if self.actors.id(defs, a as u16) != id
                || self.vitality.lives[a] != s.life
                || s.variant.is_some_and(|v| v as usize >= c.variants.len())
                || s.started > self.tick
                || s.animation_started > self.tick
            {
                return Err(invalid());
            }
        }
        if self.presentation.voices.len() > crate::npc_presentation::VOICES
            || self.presentation.voices.iter().any(|v| {
                v.sound as usize >= author.avatar_audio.sounds.len()
                    || v.started > self.tick
                    || v.position.iter().any(|x| !x.is_finite())
            })
        {
            return Err(invalid());
        }
        self.triggers.validate_checkpoint(entities)?;
        Ok(())
    }
}
