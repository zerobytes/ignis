//! Optional ground motor. Navigation/support policies remain separate from contact grounding.
use crate::model::{BodyKind, Entity, Result, invalid};
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub max_slope_degrees: f32,
    pub slide_slope_degrees: f32,
    pub step: Option<Step>,
    pub snap_distance: f32,
    pub offset: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub push_dynamic_bodies: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            max_slope_degrees: 30.,
            slide_slope_degrees: 35.,
            step: None,
            snap_distance: 0.15,
            offset: 0.01,
            gravity: 9.81,
            max_fall_speed: 50.,
            push_dynamic_bodies: false,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub max_height: f32,
    pub min_width: f32,
}
impl Config {
    pub fn validate(self, entity: &Entity, version: u32) -> Result<()> {
        let range = |x: f32, a: f32, b: f32| x.is_finite() && (a..=b).contains(&x);
        if version < 82
            || !matches!(entity.body, Some(BodyKind::Player | BodyKind::Npc))
            || entity.npc.as_ref().is_some_and(|n| {
                n.mobility.is_some_and(|m| {
                    version < 84
                        || !m.walk
                        || (version < 90
                            && (m.fly_speed.is_some()
                                || m.jump_speed.is_some()
                                || m.glide.is_some()))
                        || n.local_connections.is_some_and(|c| {
                            version < 85 || (!c.walk && version < 90) || (c.fly && version < 90)
                        })
                })
            })
        {
            return Err(invalid(
                "Character controller requires scene82; spatial use requires scene84 and walk-only before scene90; local walk requires scene85, hybrid modes require scene90",
            ));
        }
        if !range(self.max_slope_degrees, 0., 85.)
            || !range(self.slide_slope_degrees, self.max_slope_degrees, 85.)
            || !range(self.snap_distance, 0., 1.)
            || !range(self.offset, 0.001, 0.1)
            || self.offset
                > entity
                    .transform
                    .size
                    .into_iter()
                    .fold(f32::INFINITY, f32::min)
                    * 0.125
            || !range(self.gravity, 0.1, 100.)
            || !range(self.max_fall_speed, 1., 200.)
            || self
                .step
                .is_some_and(|s| !range(s.max_height, 0.01, 1.) || !range(s.min_width, 0.01, 4.))
        {
            return Err(invalid(
                "Invalid character controller slope/step/snap/offset/gravity/fall limits",
            ));
        }
        Ok(())
    }
    pub(crate) fn compile(self) -> KinematicCharacterController {
        KinematicCharacterController {
            max_slope_climb_angle: self.max_slope_degrees.to_radians(),
            min_slope_slide_angle: self.slide_slope_degrees.to_radians(),
            offset: CharacterLength::Absolute(self.offset),
            snap_to_ground: (self.snap_distance > 0.)
                .then_some(CharacterLength::Absolute(self.snap_distance)),
            autostep: self.step.map(|s| CharacterAutostep {
                max_height: CharacterLength::Absolute(s.max_height),
                min_width: CharacterLength::Absolute(s.min_width),
                include_dynamic_bodies: false,
            }),
            ..Default::default()
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub instance: crate::identity::Reference,
    pub entity_index: u16,
    pub active: bool,
    #[serde(default)]
    pub airborne: bool,
    pub grounded: bool,
    pub sliding: bool,
    pub vertical_velocity: f32,
    pub requested_velocity: [f32; 2],
    pub desired_translation: [f32; 3],
    pub corrected_translation: [f32; 3],
    pub collisions: u8,
}
impl State {
    pub(crate) fn new(
        instance: crate::identity::Reference,
        entity_index: u16,
        active: bool,
    ) -> Self {
        Self {
            instance,
            entity_index,
            active,
            airborne: false,
            grounded: false,
            sliding: false,
            vertical_velocity: 0.,
            requested_velocity: [0.; 2],
            desired_translation: [0.; 3],
            corrected_translation: [0.; 3],
            collisions: 0,
        }
    }
}
