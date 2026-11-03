//! Discrete, axis-aligned gameplay regions. Physics supplies positions; this module emits data only.
use crate::model::{Entity, Result, Scene, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MAX_VOLUMES: usize = 32;
pub const MAX_TARGETS: usize = 32;
pub const EVENT_CAPACITY: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub enabled: bool,
    pub targets: Vec<String>,
}
impl Config {
    pub fn validate(&self, owner: &Entity, scene: &Scene) -> Result<()> {
        if owner.body.is_some() || self.targets.is_empty() || self.targets.len() > MAX_TARGETS {
            return Err(invalid(
                "Trigger requires no physical body and 1..32 target IDs",
            ));
        }
        let mut seen = BTreeSet::new();
        for id in &self.targets {
            if !seen.insert(id)
                || !scene
                    .entities
                    .iter()
                    .any(|e| e.id == *id && e.body.is_some())
            {
                return Err(invalid(
                    "Trigger targets must be unique existing physical bodies",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Enter,
    Exit,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventReason {
    Overlap,
    Removed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub volume_instance: crate::identity::Reference,
    pub target_instance: crate::identity::Reference,
    pub tick: u64,
    pub kind: EventKind,
    pub reason: EventReason,
    /// Indices into lexicographically sorted volume IDs and its sorted target IDs.
    pub volume: u8,
    pub target: u8,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolumeState {
    pub occupancy: u32,
    pub entered: u32,
    pub exited: u32,
    pub last_change_tick: Option<u64>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub volumes: Vec<VolumeState>,
    pub events: VecDeque<Event>,
    pub total_events: u32,
    pub dropped_events: u32,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Actor {
    instance: crate::identity::Reference,
    id: String,
    half: [f32; 3],
    position: [f32; 3],
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Volume {
    instance: crate::identity::Reference,
    id: String,
    enabled: bool,
    center: [f32; 3],
    half: [f32; 3],
    targets: Vec<u16>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    actors: Vec<Actor>,
    volumes: Vec<Volume>,
    pub state: State,
}
impl Runtime {
    /// Last post-physics occupancy, filtered against current identities after external removals.
    pub(crate) fn occupied_present(
        &self,
        index: u8,
        identities: &crate::targeting::Catalog,
    ) -> bool {
        let volume = &self.volumes[index as usize];
        if !volume.enabled || !identities.contains(volume.instance) {
            return false;
        }
        let mask = self.state.volumes[index as usize].occupancy;
        volume.targets.iter().enumerate().any(|(slot, &actor)| {
            mask & (1u32 << slot) != 0 && identities.contains(self.actors[actor as usize].instance)
        })
    }
    pub fn new(scene: &Scene, identities: &crate::targeting::Catalog) -> Self {
        let ids: BTreeSet<_> = scene
            .entities
            .iter()
            .filter_map(|e| e.trigger.as_ref())
            .flat_map(|t| t.targets.iter())
            .collect();
        let actors: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let (index, e) = scene
                    .entities
                    .iter()
                    .enumerate()
                    .find(|(_, e)| e.id == *id)
                    .unwrap();
                Actor {
                    instance: identities.entity_reference(index).unwrap(),
                    id: id.clone(),
                    half: e.transform.size.map(|v| v * 0.5),
                    position: e.transform.position,
                }
            })
            .collect();
        let mut volumes: Vec<_> = scene
            .entities
            .iter()
            .enumerate()
            .filter_map(|(index, e)| {
                e.trigger.as_ref().map(|c| {
                    let mut targets: Vec<_> = c
                        .targets
                        .iter()
                        .map(|id| actors.binary_search_by(|a| a.id.cmp(id)).unwrap() as u16)
                        .collect();
                    targets.sort_unstable();
                    Volume {
                        instance: identities.entity_reference(index).unwrap(),
                        id: e.id.clone(),
                        enabled: c.enabled,
                        center: e.transform.position,
                        half: e.transform.size.map(|v| v * 0.5),
                        targets,
                    }
                })
            })
            .collect();
        volumes.sort_by(|a, b| a.id.cmp(&b.id));
        let state = State {
            volumes: vec![VolumeState::default(); volumes.len()],
            events: VecDeque::with_capacity(if volumes.is_empty() {
                0
            } else {
                EVENT_CAPACITY
            }),
            ..Default::default()
        };
        Self {
            actors,
            volumes,
            state,
        }
    }
    pub fn update(
        &mut self,
        tick: u64,
        identities: &crate::targeting::Catalog,
        mut position: impl FnMut(&str) -> [f32; 3],
    ) {
        if self.volumes.is_empty() {
            return;
        }
        for actor in &mut self.actors {
            if identities.contains(actor.instance) {
                actor.position = position(&actor.id);
            }
        }
        for (index, volume) in self.volumes.iter().enumerate() {
            if !volume.enabled {
                continue;
            }
            let volume_present = identities.contains(volume.instance);
            let state = &mut self.state.volumes[index];
            for (slot, &actor) in volume.targets.iter().enumerate() {
                let actor = &self.actors[actor as usize];
                let inside = identities.contains(actor.instance)
                    && volume_present
                    && (0..3).all(|axis| {
                        (actor.position[axis] - volume.center[axis]).abs()
                            < actor.half[axis] + volume.half[axis]
                    });
                let mask = 1u32 << slot;
                if inside == (state.occupancy & mask != 0) {
                    continue;
                }
                state.occupancy ^= mask;
                state.last_change_tick = Some(tick);
                let kind = if inside {
                    state.entered += 1;
                    EventKind::Enter
                } else {
                    state.exited += 1;
                    EventKind::Exit
                };
                if self.state.events.len() == EVENT_CAPACITY {
                    self.state.events.pop_front();
                    self.state.dropped_events += 1;
                }
                self.state.events.push_back(Event {
                    volume_instance: volume.instance,
                    target_instance: actor.instance,
                    tick,
                    kind,
                    reason: if !volume_present || !identities.contains(actor.instance) {
                        EventReason::Removed
                    } else {
                        EventReason::Overlap
                    },
                    volume: index as u8,
                    target: slot as u8,
                });
                self.state.total_events += 1;
            }
        }
    }
    fn volume_index(&self, id: &str) -> Option<usize> {
        self.volumes
            .binary_search_by(|v| v.id.as_str().cmp(id))
            .ok()
    }
    pub fn occupied(&self, id: &str, target: &str) -> Option<bool> {
        let index = self.volume_index(id)?;
        let volume = &self.volumes[index];
        let slot = volume
            .targets
            .binary_search_by(|&actor| self.actors[actor as usize].id.as_str().cmp(target))
            .ok()?;
        Some(self.state.volumes[index].occupancy & (1u32 << slot) != 0)
    }
    pub fn volume_state(&self, id: &str) -> Option<&VolumeState> {
        self.volume_index(id).map(|i| &self.state.volumes[i])
    }
    /// ID strings and JSON are materialized only on explicit inspection.
    pub fn inspect(&self) -> Value {
        let volumes: BTreeMap<_,_> = self.volumes.iter().enumerate().map(|(index,v)| {
            let state=&self.state.volumes[index];
            let targets:Vec<_>=v.targets.iter().map(|&i| self.actors[i as usize].id.as_str()).collect();
            let occupants:Vec<_>=targets.iter().enumerate().filter(|(i,_)|state.occupancy&(1u32<<i)!=0).map(|(_,id)|*id).collect();
            (v.id.as_str(),json!({"enabled":v.enabled,"targets":targets,"occupants":occupants,
                "entered":state.entered,"exited":state.exited,"last_change_tick":state.last_change_tick}))
        }).collect();
        let events:Vec<_>=self.state.events.iter().map(|event| {
            let v=self.volumes.get(event.volume as usize).filter(|v|v.instance==event.volume_instance);
            let target=v.and_then(|v|v.targets.get(event.target as usize)).and_then(|&a|self.actors.get(a as usize)).filter(|a|a.instance==event.target_instance);
            json!({"tick":event.tick,"kind":event.kind,"reason":event.reason,"volume":v.map(|v|v.id.as_str()),"volume_instance":event.volume_instance.id(),"target":target.map(|a|a.id.as_str()),"target_instance":event.target_instance.id()})
        }).collect();
        json!({"volumes":volumes,"events":events,"event_capacity":EVENT_CAPACITY,
            "total_events":self.state.total_events,"dropped_events":self.state.dropped_events})
    }
}

impl Runtime {
    pub(crate) fn validate_checkpoint(&self, entities: usize) -> crate::model::Result<()> {
        if self.actors.len() > MAX_VOLUMES * MAX_TARGETS
            || self.volumes.len() > 32
            || self.state.volumes.len() != self.volumes.len()
            || self.state.events.len() > EVENT_CAPACITY
            || self.actors.iter().any(|a| {
                a.instance.slot as usize >= entities
                    || a.position
                        .iter()
                        .chain(a.half.iter())
                        .any(|x| !x.is_finite())
            })
            || self.volumes.iter().any(|v| {
                v.targets.len() > 32 || v.targets.iter().any(|i| *i as usize >= self.actors.len())
            })
        {
            return Err(crate::model::invalid("Checkpoint trigger dimensions"));
        }
        Ok(())
    }
}
