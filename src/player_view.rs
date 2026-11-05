//! Session-only local player camera; world-space inputs still use the existing controller.
use crate::{
    model::{Camera, Result, invalid},
    physics::Input,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    FirstPerson,
    ThirdPerson,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ThirdPerson {
    pub distance_meters: f32,
    pub shoulder_offset: f32,
    pub height_offset: f32,
    pub collision_radius: f32,
    pub collision_padding: f32,
    pub hide_body_below_meters: f32,
}
impl Default for ThirdPerson {
    fn default() -> Self {
        Self {
            distance_meters: 4.,
            shoulder_offset: 0.4,
            height_offset: 0.,
            collision_radius: 0.15,
            collision_padding: 0.03,
            hide_body_below_meters: 0.65,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Placement {
    pub pivot: [f32; 3],
    pub desired: [f32; 3],
    pub forward: [f32; 3],
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Resolved {
    pub camera: Camera,
    pub pivot: [f32; 3],
    pub desired_position: [f32; 3],
    pub fraction: f32,
    pub obstructed: bool,
    pub initial_overlap: bool,
    pub hide_body: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub actor: String,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
    pub eye_height: f32,
    pub fov_degrees: f32,
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub third_person: ThirdPerson,
}
impl Config {
    /// Project a third-person view from received public placement when no local
    /// authoritative physics snapshot is available. This does not sweep obstacles.
    /// Offline callers should use Engine::player_camera for physical obstruction.
    pub fn unobstructed_camera(&self, center: [f32; 3], height: f32) -> Result<Camera> {
        self.validate()?;
        if center.iter().any(|v| !v.is_finite()) || !height.is_finite() || height <= self.eye_height
        {
            return Err(invalid(
                "Camera requires finite placement and eyes inside the body",
            ));
        }
        Ok(self
            .resolve(self.placement(center, height), 1., false)
            .camera)
    }
    pub fn validate(&self) -> Result<()> {
        let t = &self.third_person;
        if !self.yaw_degrees.is_finite()
            || !self.pitch_degrees.is_finite()
            || !(-85.0..=85.0).contains(&self.pitch_degrees)
            || !self.eye_height.is_finite()
            || self.eye_height <= 0.
            || !self.fov_degrees.is_finite()
            || !(10.0..=120.0).contains(&self.fov_degrees)
            || !t.distance_meters.is_finite()
            || !(0.25..=100.).contains(&t.distance_meters)
            || !t.shoulder_offset.is_finite()
            || !(-10. ..=10.).contains(&t.shoulder_offset)
            || !t.height_offset.is_finite()
            || !(-10. ..=10.).contains(&t.height_offset)
            || !t.collision_radius.is_finite()
            || !(0.05..=2.).contains(&t.collision_radius)
            || !t.collision_padding.is_finite()
            || !(0. ..=0.5).contains(&t.collision_padding)
            || !t.hide_body_below_meters.is_finite()
            || !(0. ..=10.).contains(&t.hide_body_below_meters)
        {
            return Err(invalid(
                "Player view needs finite angles, pitch[-85,85], eyes inside body, FOV[10,120]; third-person distance[0.25,100], offsets[-10,10], radius[0.05,2], padding[0,0.5], hide distance[0,10]m",
            ));
        }
        Ok(())
    }
    pub fn camera(&self, center: [f32; 3], height: f32) -> Camera {
        self.resolve(self.placement(center, height), 1., false)
            .camera
    }
    pub(crate) fn placement(&self, center: [f32; 3], height: f32) -> Placement {
        let yaw = self.yaw_degrees.to_radians();
        let pitch = self.pitch_degrees.to_radians();
        let mut pivot = [
            center[0],
            center[1] - height * 0.5 + self.eye_height,
            center[2],
        ];
        let forward = [
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            -yaw.cos() * pitch.cos(),
        ];
        let mut desired = pivot;
        if self.mode == Mode::ThirdPerson {
            pivot[1] += self.third_person.height_offset;
            let right = [yaw.cos(), 0., yaw.sin()];
            desired = std::array::from_fn(|i| {
                pivot[i] - forward[i] * self.third_person.distance_meters
                    + right[i] * self.third_person.shoulder_offset
            });
        }
        Placement {
            pivot,
            desired,
            forward,
        }
    }
    pub(crate) fn resolve(&self, p: Placement, fraction: f32, initial_overlap: bool) -> Resolved {
        let position = std::array::from_fn(|i| p.pivot[i] + (p.desired[i] - p.pivot[i]) * fraction);
        let camera = Camera {
            position,
            target: [
                position[0] + p.forward[0] * 10.,
                position[1] + p.forward[1] * 10.,
                position[2] + p.forward[2] * 10.,
            ],
            fov_degrees: self.fov_degrees,
        };
        let distance = glam::Vec3::from(position).distance(p.pivot.into());
        Resolved {
            camera,
            pivot: p.pivot,
            desired_position: p.desired,
            fraction,
            obstructed: fraction < 1.,
            initial_overlap,
            hide_body: self.mode == Mode::FirstPerson
                || (self.third_person.hide_body_below_meters > 0.
                    && distance < self.third_person.hide_body_below_meters),
        }
    }
    pub fn world_input(&self, input: Input) -> Input {
        let (s, c) = self.yaw_degrees.to_radians().sin_cos();
        let scale = input.x.hypot(input.z).max(1.);
        Input {
            x: ((input.x * c - input.z * s) / scale).clamp(-1., 1.),
            z: ((input.x * s + input.z * c) / scale).clamp(-1., 1.),
            ..input
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct State {
    pub config: Config,
    pub reference: crate::identity::Reference,
    pub life: u64,
    pub serial: u64,
}
