//! Bounded, deterministic locomotion stimuli. No audio-device or animation dependency.
use crate::{
    model::{FIXED_DT, FootstepConfig, Scene},
    noise::Noise,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub const RECENT_CAPACITY: usize = 256;
pub const AUTOMATIC_ID_BASE: u64 = 1 << 31;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FootstepState {
    pub entity_index: u16,
    pub surface_index: u16,
    pub distance_since_step: f32,
    pub grounded: bool,
    pub speed: f32,
    pub emitted: u64,
}
impl FootstepState {
    pub fn advance(
        &mut self,
        config: &FootstepConfig,
        displacement: [f32; 2],
        grounded: bool,
        intending: bool,
    ) -> bool {
        self.grounded = grounded;
        let distance = glam::Vec2::from(displacement).length();
        self.speed = distance / FIXED_DT;
        if !grounded {
            self.distance_since_step = 0.;
            return false;
        }
        if !intending || self.speed < config.min_speed {
            return false;
        }
        self.distance_since_step += distance;
        if self.distance_since_step < config.stride {
            return false;
        }
        // Coalesce exceptionally large physical pushes into one impulse in this tick.
        self.distance_since_step %= config.stride;
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Footsteps {
    pub states: BTreeMap<String, FootstepState>,
    pub pending: Vec<Noise>,
    pub recent: VecDeque<Noise>,
    pub total_emitted: u64,
    pub dropped_recent: u64,
}
impl Footsteps {
    pub(crate) fn install(&mut self, scene: &Scene, index: usize) {
        let e = &scene.entities[index];
        if let Some(c) = &e.footsteps {
            self.states.insert(
                e.id.clone(),
                FootstepState {
                    entity_index: index as u16,
                    surface_index: scene
                        .entities
                        .iter()
                        .position(|s| s.id == c.surface)
                        .unwrap() as u16,
                    distance_since_step: 0.,
                    grounded: false,
                    speed: 0.,
                    emitted: 0,
                },
            );
            self.pending.reserve(1);
            if self.recent.capacity() == 0 {
                self.recent.reserve(RECENT_CAPACITY);
            }
        }
    }

    pub fn new(scene: &Scene) -> Self {
        let states: BTreeMap<_, _> = scene
            .entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.footsteps.is_some())
            .map(|(index, e)| {
                (
                    e.id.clone(),
                    FootstepState {
                        entity_index: index as u16,
                        surface_index: scene
                            .entities
                            .iter()
                            .position(|s| s.id == e.footsteps.as_ref().unwrap().surface)
                            .unwrap() as u16,
                        distance_since_step: 0.,
                        grounded: false,
                        speed: 0.,
                        emitted: 0,
                    },
                )
            })
            .collect();
        let capacity = states.len();
        Self {
            states,
            pending: Vec::with_capacity(capacity),
            recent: VecDeque::with_capacity(if capacity == 0 { 0 } else { RECENT_CAPACITY }),
            total_emitted: 0,
            dropped_recent: 0,
        }
    }
}
