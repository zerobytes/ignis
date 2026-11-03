//! Optional vital state, independent of ability grants. Mutations commit with an ability plan.
use crate::{
    model::{BodyKind, Result, Scene, invalid},
    targeting::Catalog,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub initial: i64,
    pub maximum: i64,
    pub scale: u32,
}
pub fn validate(scene: &Scene) -> Result<()> {
    for e in &scene.entities {
        if let Some(c) = e.vitality {
            if scene.version < 27 || !matches!(e.body, Some(BodyKind::Player | BodyKind::Npc)) {
                return Err(invalid("Vitality requires scene version 27 and an actor"));
            }
            if !(1..=crate::abilities::LIMIT).contains(&c.maximum)
                || !(0..=c.maximum).contains(&c.initial)
                || !(1..=1_000_000).contains(&c.scale)
            {
                return Err(invalid("Invalid vitality initial, maximum or scale"));
            }
        }
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Health {
    pub units: i64,
    pub maximum: i64,
    pub scale: u32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Death {
    pub actor_instance: crate::identity::Reference,
    pub source_instance: crate::identity::Reference,
    pub life: u64,
    pub tick: u64,
    pub actor: u16,
    pub source: u16,
    pub execution_id: u64,
    pub effect_index: u8,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Revival {
    pub actor_instance: crate::identity::Reference,
    pub source_instance: crate::identity::Reference,
    pub tick: u64,
    pub actor: u16,
    pub source: u16,
    pub execution_id: u64,
    pub life: u64,
    pub units: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub lives: Vec<u64>,
    pub died_this_tick: crate::actor_set::ActorSet,
    pub revived_this_tick: crate::actor_set::ActorSet,
    pub revivals: VecDeque<Revival>,
    pub dropped_revivals: u64,
    pub actors: Vec<Option<Health>>,
    pub deaths: VecDeque<Death>,
    pub dropped_deaths: u64,
}
/// Final health and lethal cause for one touched actor; never a world snapshot.
#[derive(Clone, Copy)]
pub(crate) struct ChangeSet {
    pub health: Option<Health>,
    pub cause: Option<u8>,
}
pub(crate) type Changes = crate::transaction::Writes<usize, ChangeSet, 2>;
pub struct Runtime {
    pub state: State,
}
impl Runtime {
    pub fn new(scene: &Scene) -> Self {
        let catalog = Catalog::new(scene);
        Self::with_actors(scene, &catalog)
    }
    pub(crate) fn with_actors(scene: &Scene, catalog: &Catalog) -> Self {
        let mut actors = vec![None; catalog.actors.len()];
        for (a, &i) in catalog.actors.iter().enumerate() {
            actors[a] = scene.entities[i as usize].vitality.map(|c| Health {
                units: c.initial,
                maximum: c.maximum,
                scale: c.scale,
            });
        }
        Self {
            state: State {
                lives: vec![0; actors.len()],
                died_this_tick: crate::actor_set::ActorSet::EMPTY,
                revived_this_tick: crate::actor_set::ActorSet::EMPTY,
                revivals: VecDeque::with_capacity(256),
                dropped_revivals: 0,
                actors,
                deaths: VecDeque::with_capacity(256),
                dropped_deaths: 0,
            },
        }
    }
    pub fn dead(&self, actor: usize) -> bool {
        self.state
            .actors
            .get(actor)
            .and_then(|h| *h)
            .is_some_and(|h| h.units == 0)
    }
    pub fn dead_mask(&self) -> crate::actor_set::ActorSet {
        let mut result = crate::actor_set::ActorSet::EMPTY;
        for (a, health) in self.state.actors.iter().enumerate() {
            if health.is_some_and(|h| h.units == 0) {
                result.insert(a);
            }
        }
        result
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn commit(
        &mut self,
        changes: &mut Changes,
        tick: u64,
        source: u16,
        execution_id: u64,
        identities: &Catalog,
        source_instance: crate::identity::Reference,
    ) {
        changes.sort_by_key(|a| identities.instance_order(a as u16));
        for &(a, change) in changes.iter() {
            let h = &change.health;
            if self.dead(a) && h.is_some_and(|h| h.units > 0) {
                self.state.lives[a] = self.state.lives[a]
                    .checked_add(1)
                    .expect("validated life budget");
                self.state.revived_this_tick.insert(a);
                if self.state.revivals.len() == 256 {
                    self.state.revivals.pop_front();
                    self.state.dropped_revivals += 1;
                }
                self.state.revivals.push_back(Revival {
                    actor_instance: identities.reference(a as u16),
                    source_instance,
                    tick,
                    actor: a as u16,
                    source,
                    execution_id,
                    life: self.state.lives[a],
                    units: h.unwrap().units,
                });
            }
            if !self.dead(a) && h.is_some_and(|h| h.units == 0) {
                self.state.died_this_tick.insert(a);
                if self.state.deaths.len() == 256 {
                    self.state.deaths.pop_front();
                    self.state.dropped_deaths += 1;
                }
                self.state.deaths.push_back(Death {
                    actor_instance: identities.reference(a as u16),
                    source_instance,
                    life: self.state.lives[a],
                    tick,
                    actor: a as u16,
                    source,
                    execution_id,
                    effect_index: change.cause.expect("lethal plan has damage cause"),
                });
            }
            self.state.actors[a] = change.health;
        }
    }
    pub fn inspect(&self, scene: &Scene, actors: &Catalog) -> serde_json::Value {
        use serde_json::json;
        let states:serde_json::Map<String,serde_json::Value>=actors.actors.iter().enumerate().filter_map(|(a,_)| self.state.actors[a].map(|h| (actors.id(scene,a as u16).into(),json!({"units":h.units,"maximum":h.maximum,"scale":h.scale,"life_index":self.state.lives[a],"life":if h.units==0 {"dead"} else {"alive"}})))).collect();
        let deaths:Vec<_>=self.state.deaths.iter().map(|d|json!({"tick":d.tick,"actor":actors.label(scene,d.actor_instance),"actor_instance":d.actor_instance.id(),"source":actors.label(scene,d.source_instance),"source_instance":d.source_instance.id(),"execution_id":d.execution_id,"effect_index":d.effect_index,"life_index":d.life})).collect();
        let revivals:Vec<_>=self.state.revivals.iter().map(|r|json!({"tick":r.tick,"actor":actors.label(scene,r.actor_instance),"actor_instance":r.actor_instance.id(),"source":actors.label(scene,r.source_instance),"source_instance":r.source_instance.id(),"execution_id":r.execution_id,"life_index":r.life,"units":r.units})).collect();
        json!({"actors":states,"deaths":deaths,"dropped_deaths":self.state.dropped_deaths,"revivals":revivals,"dropped_revivals":self.state.dropped_revivals})
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    Damage,
    Heal,
    Resurrect,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Applied {
    pub kind: Change,
    pub before: i64,
    pub after: i64,
    pub excess: i64,
}
