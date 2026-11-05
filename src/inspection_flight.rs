//! Session-only, offline inspection flight. Not NPC/gameplay movement or persistence.
use super::*;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub speed: f32,
    pub fast_speed: f32,
    pub follow_pitch: bool,
    pub near_clip: f32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            speed: 100.,
            fast_speed: 600.,
            follow_pitch: true,
            near_clip: 0.5,
        }
    }
}
#[derive(Clone)]
pub(crate) struct State {
    pub config: Config,
    pub reference: crate::identity::Reference,
    pub life: u64,
    pub vertical: f32,
}
#[derive(Clone, Copy)]
pub(crate) struct Motion {
    pub actor: crate::identity::Reference,
    pub target: [f32; 3],
}
impl Engine {
    pub fn inspection_flight_active(&self) -> bool {
        self.inspection_flight.as_ref().is_some_and(|f| {
            self.player_view_actor().is_some_and(|a| {
                self.simulation.actors.reference(a as u16) == f.reference
                    && self.simulation.vitality.state.lives[a] == f.life
            })
        })
    }
    pub(crate) fn inspection_clip(&self) -> Option<crate::render::Clip> {
        self.inspection_flight_active()
            .then(|| crate::render::Clip {
                near: self.inspection_flight.as_ref().unwrap().config.near_clip,
                far: 30000.,
                reverse_z: false,
            })
    }
    pub(super) fn set_inspection_flight(&mut self, config: Option<Config>) -> Result<Value> {
        if let Some(c) = config {
            if self.execution_mode != ExecutionMode::Continuous || self.planet_active() {
                return Err(invalid(
                    "Inspection flight requires offline continuous local-world execution",
                ));
            }
            if !c.speed.is_finite()
                || !(0.1..=5000.).contains(&c.speed)
                || !c.fast_speed.is_finite()
                || !(c.speed..=10000.).contains(&c.fast_speed)
                || !c.near_clip.is_finite()
                || !(0.01..=10.).contains(&c.near_clip)
            {
                return Err(invalid(
                    "Invalid inspection flight speed/fast speed or near clip",
                ));
            }
            let a = self
                .player_view_actor()
                .ok_or_else(|| invalid("Inspection flight requires a living viewed player"))?;
            let e = self.simulation.actors.entity(a as u16);
            let entity = &self.simulation.definitions().entities[e];
            if entity.body != Some(crate::model::BodyKind::Player)
                || entity.character_controller.is_none()
            {
                return Err(invalid(
                    "Inspection flight requires a player with a character controller",
                ));
            }
            self.inspection_flight = Some(State {
                config: c,
                reference: self.simulation.actors.reference(a as u16),
                life: self.simulation.vitality.state.lives[a],
                vertical: 0.,
            });
        } else {
            self.inspection_flight = None;
        }
        self.input = Input::default();
        self.input_revision += 1;
        self.view_revision += 1;
        Ok(self.inspect_inspection_flight())
    }
    pub(super) fn inspect_inspection_flight(&self) -> Value {
        json!({"active":self.inspection_flight_active(),"config":self.inspection_flight.as_ref().map(|f|f.config),"vertical":self.inspection_flight.as_ref().map(|f|f.vertical),"session_only":true,"collision":"noclip","gameplay_flight":false})
    }
    pub(crate) fn inspection_motion(&mut self) -> Option<Motion> {
        if !self.inspection_flight_active() {
            self.inspection_flight = None;
            return None;
        }
        let a = self.player_view_actor()?;
        let view = &self.player_view.as_ref()?.config;
        let f = self.inspection_flight.as_ref()?;
        let mut velocity = glam::Vec3::new(self.input.x, f.vertical, self.input.z);
        if f.config.follow_pitch {
            let yaw = view.yaw_degrees.to_radians();
            let pitch = view.pitch_degrees.to_radians();
            let forward = glam::Vec3::new(yaw.sin(), 0., -yaw.cos());
            let amount = velocity.dot(forward);
            velocity += forward * amount * (pitch.cos() - 1.);
            velocity.y += amount * pitch.sin();
        }
        velocity /= velocity.length().max(1.);
        let speed = match self.input.mode {
            crate::model::MovementMode::Run => f.config.fast_speed,
            crate::model::MovementMode::Sneak => f.config.speed * 0.02,
            _ => f.config.speed,
        };
        let entity = self.simulation.actors.entity(a as u16);
        let id = &self.simulation.definitions().entities[entity].id;
        let p = glam::Vec3::from_array(self.simulation.position(id)?);
        let target = p + velocity * speed * crate::model::FIXED_DT;
        Some(Motion {
            actor: f.reference,
            target: target.to_array(),
        })
    }
}
