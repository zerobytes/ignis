//! Local, execution-scoped control bindings. Network authentication is a separate boundary.
use crate::{
    identity::Reference,
    model::{BodyKind, Result, Scene, invalid},
    physics::Input,
    targeting::Catalog,
    vitality,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_EVENTS: usize = 4096;
pub const PER_TICK: usize = 4096;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Bind {
        control: u64,
        actor: Reference,
        life: u64,
    },
    Input {
        control: u64,
        input: Input,
        life: u64,
    },
    Release {
        control: u64,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub sequence: u64,
    pub tick: u64,
    pub operation: Operation,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub control: u64,
    pub actor: Reference,
    pub life: u64,
    pub input: Input,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub next_control: u64,
    pub bindings: BTreeMap<u64, Binding>,
}
pub struct Runtime {
    pub state: State,
    inputs: Vec<Option<Input>>,
}
impl Runtime {
    pub fn new(actors: usize) -> Self {
        Self {
            state: State {
                next_control: 1,
                bindings: BTreeMap::new(),
            },
            inputs: vec![None; actors],
        }
    }
    pub fn binding(&self, control: u64) -> Result<&Binding> {
        self.state
            .bindings
            .get(&control)
            .ok_or_else(|| invalid("Unknown or released control"))
    }
    pub fn owns(&self, actor: Reference) -> bool {
        self.state.bindings.values().any(|b| b.actor == actor)
    }
    pub fn apply(
        &mut self,
        op: Operation,
        actors: &Catalog,
        scene: &Scene,
        vitality: &vitality::Runtime,
    ) -> Result<()> {
        match op {
            Operation::Bind {
                control,
                actor,
                life,
            } => {
                let slot = actors
                    .actor_slot(actor)
                    .ok_or_else(|| invalid("Unknown controlled instance"))?;
                if scene.entities[actors.entity(slot as u16)].body != Some(BodyKind::Player)
                    || vitality.dead(slot)
                    || vitality.state.lives[slot] != life
                {
                    return Err(invalid(
                        "Control requires a living player instance and current life",
                    ));
                }
                let next = self
                    .state
                    .next_control
                    .checked_add(1)
                    .ok_or_else(|| invalid("Control identity exhausted"))?;
                if control != self.state.next_control || self.owns(actor) {
                    return Err(invalid(
                        "Control sequence mismatch or player already controlled",
                    ));
                }
                self.state.bindings.insert(
                    control,
                    Binding {
                        control,
                        actor,
                        life,
                        input: Input::default(),
                    },
                );
                self.state.next_control = next;
            }
            Operation::Input {
                control,
                input,
                life,
            } => {
                crate::engine::Engine::validate_input(input)?;
                let binding = self.binding(control)?;
                let slot = actors
                    .actor_slot(binding.actor)
                    .ok_or_else(|| invalid("Unknown controlled instance"))?;
                if vitality.dead(slot) || vitality.state.lives[slot] != life {
                    return Err(invalid(
                        "Input requires a living controlled instance and current life",
                    ));
                }
                let binding = self.state.bindings.get_mut(&control).unwrap();
                binding.input = input;
                binding.life = life;
            }
            Operation::Release { control } => {
                self.state
                    .bindings
                    .remove(&control)
                    .ok_or_else(|| invalid("Unknown or released control"))?;
            }
        }
        Ok(())
    }
    /// Cached actor slots give O(P) preparation, without per-tick allocation or name lookup.
    pub fn inputs(&mut self, actors: &Catalog, vitality: &vitality::Runtime) -> &[Option<Input>] {
        self.inputs.fill(None);
        for b in self.state.bindings.values() {
            if let Some(a) = actors.actor_slot(b.actor) {
                self.inputs[a] = Some(if vitality.dead(a) || vitality.state.lives[a] != b.life {
                    Input::default()
                } else {
                    b.input
                });
            }
        }
        &self.inputs
    }
    /// Death and death/revival in one tick both clear held explicit input.
    pub fn observe_lifecycle(&mut self, actors: &Catalog, vitality: &vitality::Runtime) {
        for b in self.state.bindings.values_mut() {
            if let Some(a) = actors.actor_slot(b.actor) {
                if vitality.dead(a)
                    || vitality.state.lives[a] != b.life
                    || vitality.state.died_this_tick.contains(a)
                {
                    b.input = Input::default();
                    b.life = vitality.state.lives[a];
                }
            } else {
                b.input = Input::default();
            }
        }
    }
}

pub fn validate_order(events: &[Entry], ticks: usize) -> Result<()> {
    if events.len() > MAX_EVENTS {
        return Err(invalid("Too many control events"));
    }
    let mut previous = None;
    let mut count = 0;
    for (i, event) in events.iter().enumerate() {
        if event.sequence != i as u64
            || event.tick > ticks as u64
            || event.tick >= crate::engine::MAX_TICKS as u64
            || previous.is_some_and(|tick| event.tick < tick)
        {
            return Err(invalid("Invalid control event sequence or tick"));
        }
        count = if previous == Some(event.tick) {
            count + 1
        } else {
            1
        };
        if count > PER_TICK {
            return Err(invalid("Too many control events in tick"));
        }
        if let Operation::Input { input, .. } = event.operation {
            crate::engine::Engine::validate_input(input)?;
        }
        previous = Some(event.tick);
    }
    Ok(())
}
