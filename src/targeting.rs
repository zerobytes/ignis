//! Authored relations compiled once per world; selection consumes sensor evidence only.
use crate::model::{BodyKind, Entity, Result, Scene, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const MAX_ACTORS: usize = 1024;
pub const MAX_PLAYERS: usize = 1000;
pub const MAX_FACTIONS: usize = 32;
pub const MAX_RELATIONS: usize = 64;
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Attitude {
    Allied,
    #[default]
    Neutral,
    Hostile,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub attitude: Attitude,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Policy {
    HostileNearest { switch_margin: f32 },
}
impl Policy {
    pub fn margin(self) -> f32 {
        match self {
            Self::HostileNearest { switch_margin } => switch_margin,
        }
    }
}
fn valid_label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}
fn actor(e: &Entity) -> bool {
    matches!(e.body, Some(BodyKind::Player | BodyKind::Npc))
}
pub fn validate(scene: &Scene) -> Result<()> {
    if scene.version < 12
        && (!scene.faction_relations.is_empty()
            || scene.entities.iter().any(|e| {
                e.faction.is_some() || e.npc.as_ref().is_some_and(|n| n.targeting.is_some())
            }))
    {
        return Err(invalid(
            "Factions, relations and automatic targeting require scene version 12",
        ));
    }
    if scene.faction_relations.len() > MAX_RELATIONS {
        return Err(invalid(
            "At most 64 directional faction relations are supported",
        ));
    }
    let mut labels = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    for e in scene.entities.iter().chain(&scene.npc_templates) {
        if let Some(faction) = &e.faction {
            if !actor(e) || !valid_label(faction) {
                return Err(invalid(
                    "Faction requires player/npc and 1..80 ASCII identifier bytes",
                ));
            }
            labels.insert(faction.as_str());
        }
        if let Some(n) = &e.npc
            && let Some(policy) = n.targeting
            && (n.target.is_some()
                || !policy.margin().is_finite()
                || !(0.0..=0.9).contains(&policy.margin()))
        {
            return Err(invalid(
                "Automatic targeting excludes explicit target and requires switch_margin [0,0.9]",
            ));
        }
    }
    for r in &scene.faction_relations {
        if !valid_label(&r.from) || !valid_label(&r.to) || !pairs.insert((&r.from, &r.to)) {
            return Err(invalid(
                "Faction relations need valid labels and unique directed pairs",
            ));
        }
        labels.insert(&r.from);
        labels.insert(&r.to);
    }
    if labels.len() > MAX_FACTIONS {
        return Err(invalid("At most 32 faction labels are supported"));
    }
    Ok(())
}
/// Initial actors are ID-sorted; reserved slots stay stable, while order() follows logical instances.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    instances: crate::identity::Registry,
    references: Vec<Option<crate::identity::Reference>>,
    pub actors: Vec<u16>,
    slots: Vec<u16>,
    labels: Vec<String>,
    factions: Vec<Option<u8>>,
    order: Vec<u16>,
    faction_attitudes: [[Attitude; MAX_FACTIONS]; MAX_FACTIONS],
}
impl Catalog {
    pub fn new(scene: &Scene) -> Self {
        let mut actors: Vec<_> = scene
            .entities
            .iter()
            .enumerate()
            .filter(|(_, e)| actor(e))
            .map(|(i, _)| i as u16)
            .collect();
        actors.sort_by(|&a, &b| {
            crate::templates::vacant(&scene.entities[a as usize])
                .cmp(&crate::templates::vacant(&scene.entities[b as usize]))
                .then_with(|| {
                    scene.entities[a as usize]
                        .id
                        .cmp(&scene.entities[b as usize].id)
                })
        });
        let mut instances =
            crate::identity::Registry::new(scene.entities.len().max(1)).expect("bounded catalog");
        let mut references = vec![None; scene.entities.len()];
        let mut entities: Vec<_> = (0..scene.entities.len()).collect();
        entities.sort_by(|&a, &b| scene.entities[a].id.cmp(&scene.entities[b].id));
        for i in entities {
            if crate::templates::vacant(&scene.entities[i]) {
                continue;
            }
            references[i] = Some(instances.insert(i as u16).expect("validated entity limit"));
        }
        let mut labels = BTreeSet::new();
        for e in scene.entities.iter().chain(&scene.npc_templates) {
            if let Some(f) = &e.faction {
                labels.insert(f.as_str());
            }
        }
        for r in &scene.faction_relations {
            labels.insert(r.from.as_str());
            labels.insert(r.to.as_str());
        }
        let labels: Vec<_> = labels.into_iter().collect();
        let mut faction_attitudes = [[Attitude::Neutral; MAX_FACTIONS]; MAX_FACTIONS];
        for (i, from) in labels.iter().enumerate() {
            for (j, to) in labels.iter().enumerate() {
                faction_attitudes[i][j] = scene
                    .faction_relations
                    .iter()
                    .find(|r| r.from == *from && r.to == *to)
                    .map_or(
                        if i == j {
                            Attitude::Allied
                        } else {
                            Attitude::Neutral
                        },
                        |r| r.attitude,
                    );
            }
        }
        let mut s = Self {
            instances,
            references,
            actors: actors.clone(),
            slots: vec![u16::MAX; scene.entities.len()],
            labels: labels.iter().map(|s| s.to_string()).collect(),
            factions: vec![None; actors.len()],
            order: Vec::with_capacity(actors.len()),
            faction_attitudes,
        };
        for (slot, &index) in s.actors.iter().enumerate() {
            s.slots[index as usize] = slot as u16;
            s.factions[slot] = scene.entities[index as usize]
                .faction
                .as_deref()
                .map(|f| labels.iter().position(|x| *x == f).unwrap() as u8);
        }
        s.rebuild_order();
        s
    }
    pub fn faction(&self, actor: u16) -> Option<u8> {
        self.factions[actor as usize]
    }
    pub fn attitude_from(&self, faction: Option<u8>, target: u16) -> Attitude {
        match (faction, self.faction(target)) {
            (Some(from), Some(to)) => self.faction_attitudes[from as usize][to as usize],
            _ => Attitude::Neutral,
        }
    }
    pub(crate) fn remove(&mut self, reference: crate::identity::Reference) {
        let entity = self
            .instances
            .remove(reference)
            .expect("validated present reference");
        self.references[entity as usize] = None;
        self.rebuild_order();
    }
    #[cfg(test)]
    pub(crate) fn remove_for_test(&mut self, entity: u16) {
        if let Some(reference) = self.references[entity as usize] {
            self.remove(reference);
        }
    }
    pub fn reference(&self, actor: u16) -> crate::identity::Reference {
        self.entity_reference(self.entity(actor))
            .expect("actor instance")
    }
    pub fn entity_reference(&self, entity: usize) -> Option<crate::identity::Reference> {
        self.references.get(entity).copied().flatten()
    }
    pub fn matches(&self, actor: u16, reference: crate::identity::Reference) -> bool {
        self.actors
            .get(actor as usize)
            .is_some_and(|&entity| self.instances.resolve(reference) == Some(entity))
    }
    pub fn contains(&self, reference: crate::identity::Reference) -> bool {
        self.instances.resolve(reference).is_some()
    }
    /// Resolve a logical instance directly to its current actor slot, never to a recycled occupant.
    pub fn actor_slot(&self, reference: crate::identity::Reference) -> Option<usize> {
        let entity = self.instances.resolve(reference)?;
        let slot = self.slots[entity as usize] as usize;
        (slot < self.actors.len()).then_some(slot)
    }
    pub fn entity_matches(&self, entity: u16, reference: crate::identity::Reference) -> bool {
        self.instances.resolve(reference) == Some(entity)
    }
    /// Diagnostic name only when the original individual is still present.
    pub fn label<'a>(
        &self,
        scene: &'a Scene,
        reference: crate::identity::Reference,
    ) -> Option<&'a str> {
        self.instances
            .resolve(reference)
            .and_then(|e| scene.entities.get(e as usize))
            .map(|e| e.id.as_str())
    }
    #[cfg(test)]
    pub(crate) fn recycle_for_test(&mut self, entity: u16) -> crate::identity::Reference {
        self.remove_for_test(entity);
        let next = self.instances.insert(entity).unwrap();
        self.references[entity as usize] = Some(next);
        self.rebuild_order();
        next
    }
    pub(crate) fn insert(
        &mut self,
        actor: u16,
        faction: Option<&str>,
    ) -> Result<crate::identity::Reference> {
        let entity = self.entity(actor);
        let reference = self.instances.insert(entity as u16)?;
        self.references[entity] = Some(reference);
        self.factions[actor as usize] = faction.map(|f| {
            self.labels
                .iter()
                .position(|x| x == f)
                .expect("validated faction") as u8
        });
        self.rebuild_order();
        Ok(reference)
    }
    /// Include vacant slots last so stale queued requests still receive a rejection.
    pub fn order(&self) -> &[u16] {
        &self.order
    }
    fn rebuild_order(&mut self) {
        self.order.clear();
        self.order.extend((0..self.actors.len()).map(|a| a as u16));
        let references = &self.references;
        let actors = &self.actors;
        self.order.sort_unstable_by_key(|&a| {
            references[actors[a as usize] as usize].map_or(u64::MAX, |r| r.instance)
        });
    }
    pub(crate) fn slot_for_instance(&self, instance: u64) -> Option<u16> {
        let i = self
            .order
            .binary_search_by_key(&instance, |&a| self.instance_order(a))
            .ok()?;
        let actor = self.order[i];
        self.entity_reference(self.entity(actor))
            .filter(|r| r.instance == instance)
            .map(|_| actor)
    }
    pub fn instance_order(&self, actor: u16) -> u64 {
        self.entity_reference(self.entity(actor))
            .map_or(u64::MAX, |r| r.instance)
    }
    pub fn instance_snapshot(&self) -> crate::identity::Snapshot {
        self.instances.snapshot()
    }
    pub fn slot(&self, entity: usize) -> u16 {
        self.slots[entity]
    }
    pub fn entity(&self, slot: u16) -> usize {
        self.actors[slot as usize] as usize
    }
    pub fn attitude(&self, from: u16, to: u16) -> Attitude {
        self.attitude_from(self.faction(from), to)
    }
    pub fn id<'a>(&self, scene: &'a Scene, slot: u16) -> &'a str {
        &scene.entities[self.entity(slot)].id
    }
    pub fn inspect(&self, scene: &Scene) -> Value {
        let actors: Vec<_> = self
            .actors
            .iter()
            .filter(|&&i| self.entity_reference(i as usize).is_some())
            .map(|&i| {
                let e = &scene.entities[i as usize];
                json!({"id":e.id,"faction":e.faction})
            })
            .collect();
        let mut resolved = Vec::new();
        for a in 0..self.actors.len() {
            for b in 0..self.actors.len() {
                if a != b
                    && self.entity_reference(self.entity(a as u16)).is_some()
                    && self.entity_reference(self.entity(b as u16)).is_some()
                {
                    resolved.push(json!({"from":self.id(scene,a as u16),"to":self.id(scene,b as u16),"attitude":self.attitude(a as u16,b as u16)}));
                }
            }
        }
        json!({"actors":actors,"relations":scene.faction_relations,"resolved":resolved})
    }
    pub fn inspect_knowledge(&self, scene: &Scene, state: &crate::perception::NpcState) -> Value {
        let tracks: Vec<_> = state.knowledge.tracks.iter().filter_map(|t| {
            let o=t.last_seen?;
            Some(json!({"actor_id":o.instance.and_then(|r|self.label(scene,r)),"instance_id":o.instance.map(|r|r.id()),"evidence":t.evidence,"progress":f64::from(t.evidence)/f64::from(state.knowledge.threshold.max(1)),"confirmed":t.confirmed,"visible":t.visible,"life_index":o.life,"last_seen_tick":o.tick,"last_seen_position":o.position,"source":o.source}))
        }).collect();
        json!(tracks)
    }
    pub fn inspect_state(&self, scene: &Scene, state: &crate::perception::NpcState) -> Value {
        let mut value = serde_json::to_value(state).unwrap();
        if let Some(pitch) = state.facing_pitch_degrees {
            value["vision_direction"] =
                json!(crate::perception::view_direction(state.facing_degrees, pitch).to_array());
        }
        if let Some(memory) = value["memory"].as_object_mut() {
            memory.remove("actor_index");
            memory.insert(
                "actor_id".into(),
                json!(
                    state
                        .memory
                        .as_ref()
                        .and_then(|m| m.instance)
                        .and_then(|r| self.label(scene, r))
                ),
            );
        }
        value["corpse_visual"] = json!(state.corpse_visual.map(|o| json!({
            "actor_id":o.instance.and_then(|r|self.label(scene,r)),"instance_id":o.instance.map(|r|r.id()), "life_index":o.life,
            "position":o.position,"tick":o.tick,"source":o.source,
        })));
        value["known_lives"] = json!(
            self.actors
                .iter()
                .enumerate()
                .map(|(a, _)| (
                    self.id(scene, a as u16),
                    if state
                        .known_instances
                        .get(a)
                        .copied()
                        .flatten()
                        .is_some_and(|r| self.matches(a as u16, r))
                    {
                        state.known_lives[a]
                    } else {
                        0
                    }
                ))
                .collect::<std::collections::BTreeMap<_, _>>()
        );
        for (i, decision) in state.objectives.ability_decisions.iter().enumerate() {
            value["objectives"]["ability_decisions"][i]["target_id"] =
                json!(decision.target.map(|target| self.id(scene, target)));
        }
        value["support"]["observed_allies"] = json!(state.support.observations.iter().flatten().map(|o| json!({
            "actor_id": o.instance.and_then(|r|self.label(scene,r)), "instance_id":o.instance.map(|r|r.id()), "life_index":o.life, "position":o.position, "tick":o.tick, "source":o.source
        })).collect::<Vec<_>>());
        value["knowledge"] = self.inspect_knowledge(scene, state);
        value["known_dead"] = json!(
            self.actors
                .iter()
                .enumerate()
                .filter(|(a, _)| state.known_dead.contains(*a)
                    && state
                        .known_instances
                        .get(*a)
                        .copied()
                        .flatten()
                        .is_some_and(|r| self.matches(*a as u16, r)))
                .map(|(a, _)| self.id(scene, a as u16))
                .collect::<Vec<_>>()
        );
        if let Some(receipt) = state.communication.last_received {
            value["communication"]["last_received"] = json!({
                "sender_id": self.label(scene, receipt.sender_instance),
                "sender_instance": receipt.sender_instance.id(),
                "target_id": receipt.observation.instance.and_then(|r|self.label(scene,r)),
                "target_instance":receipt.observation.instance.map(|r|r.id()),
                "position": receipt.observation.position,
                "observed_tick": receipt.observation.tick,
                "life_index": receipt.observation.life,
                "received_tick": receipt.received_tick,
                "expires_at": receipt.expires_at,
                "source": "vision"
            });
        }
        let selection = value["targeting"].as_object_mut().unwrap();
        selection.remove("observed_actor");
        selection.insert(
            "observed_actor".into(),
            json!(state.targeting.observed_actor.map(|i| self.id(scene, i))),
        );
        value
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    ObservedDead,
    #[default]
    AwaitingStep,
    AwaitingConfirmation,
    Disabled,
    TerritoryReturn,
    OutsideTerritory,
    NoVisualTarget,
    ExplicitObserved,
    ExplicitUnobserved,
    NoObservedHostile,
    Acquired,
    Retained,
    SwitchedCloser,
    PreviousNotObserved,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Selection {
    pub reason: Reason,
    pub eligible_candidates: u16,
    pub observed_candidates: u16,
    pub confirmed_candidates: u16,
    pub observed_actor: Option<u16>,
    pub observed_distance_squared: Option<f32>,
    pub incumbent_distance_squared: Option<f32>,
    pub nearest_distance_squared: Option<f32>,
}

impl Catalog {
    pub(crate) fn validate_checkpoint(&self, scene: &Scene) -> Result<()> {
        let n = scene.entities.len();
        self.instances.validate_checkpoint(n)?;
        if self.references.len() != n
            || self.slots.len() != n
            || self.actors.len() > MAX_ACTORS
            || self.factions.len() != self.actors.len()
            || self.labels.len() > MAX_FACTIONS
            || self.order.len() != self.actors.len()
        {
            return Err(invalid("Checkpoint actor dimensions"));
        }
        let mut seen = vec![false; n];
        for (slot, &index) in self.actors.iter().enumerate() {
            let i = index as usize;
            if i >= n
                || seen[i]
                || self.slots[i] as usize != slot
                || !actor(&scene.entities[i])
                || self.factions[slot].is_some_and(|f| f as usize >= self.labels.len())
            {
                return Err(invalid("Checkpoint actor mapping/faction"));
            }
            seen[i] = true;
        }
        for (i, was_seen) in seen.iter().copied().enumerate() {
            if self.references[i] != self.instances.reference(i as u16)
                || actor(&scene.entities[i]) != was_seen
            {
                return Err(invalid("Checkpoint actor/reference mismatch"));
            }
        }
        let mut order = (0..self.actors.len()).map(|a| a as u16).collect::<Vec<_>>();
        order.sort_unstable_by_key(|a| self.instance_order(*a));
        if order != self.order {
            return Err(invalid("Checkpoint actor order"));
        }
        Ok(())
    }
}
impl Catalog {
    pub(crate) fn historical(&self, r: crate::identity::Reference) -> bool {
        self.instances.historical(r)
    }
}
