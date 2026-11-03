//! Deterministic bounded steering. Preview and runtime use the same discrete trajectory.
use crate::{
    mobility::Capabilities,
    model::{FIXED_DT, Result, invalid},
};
use glam::{Quat, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_TRAVEL_TICKS: u32 = 1800;
pub const MAX_ORBIT_TICKS: u32 = 3600;
pub const MAX_GUIDE_POINTS: usize = 128;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Profile {
    Straight,
    Curved {
        acceleration: f32,
        deceleration: f32,
        turn_degrees_per_second: f32,
        smoothing_seconds: f32,
        minimum_speed: f32,
        can_hover: bool,
        initial_direction: [f32; 3],
    },
}
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub acceleration: f32,
    pub deceleration: f32,
    pub turn: f32,
    pub smoothing: f32,
    pub minimum_speed: f32,
    pub can_hover: bool,
    pub initial_direction: Vec3,
}
impl Profile {
    pub fn settings(self) -> Option<Settings> {
        match self {
            Self::Straight => None,
            Self::Curved {
                acceleration,
                deceleration,
                turn_degrees_per_second,
                smoothing_seconds,
                minimum_speed,
                can_hover,
                initial_direction,
            } => Some(Settings {
                acceleration,
                deceleration,
                turn: turn_degrees_per_second.to_radians(),
                smoothing: smoothing_seconds,
                minimum_speed,
                can_hover,
                initial_direction: Vec3::from(initial_direction),
            }),
        }
    }
    pub fn validate(self, caps: Option<Capabilities>) -> Result<()> {
        let Some(max_speed) = caps.and_then(|c| c.fly_speed) else {
            return Err(invalid("Flight profile requires fly_speed"));
        };
        if let Self::Curved {
            acceleration,
            deceleration,
            turn_degrees_per_second,
            smoothing_seconds,
            minimum_speed,
            can_hover,
            initial_direction,
        } = self
        {
            let bounded = |x: f32, lo: f32, hi: f32| x.is_finite() && (lo..=hi).contains(&x);
            let d = Vec3::from(initial_direction);
            if !bounded(acceleration, 0.1, 30.)
                || !bounded(deceleration, 0.1, 30.)
                || !bounded(turn_degrees_per_second, 15., 360.)
                || !bounded(smoothing_seconds, 0., 2.)
                || !bounded(minimum_speed, if can_hover { 0. } else { 0.1 }, max_speed)
                || !d.is_finite()
                || !(0.01..=100.).contains(&d.length())
            {
                return Err(invalid(
                    "Curved flight requires acceleration/deceleration [0.1,30], turn [15,360]deg/s, smoothing [0,2]s, minimum_speed [0,fly_speed] (>=0.1 without hover), finite nonzero initial_direction",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub heading: [f32; 3],
    pub aim: [f32; 3],
    pub velocity: [f32; 3],
    pub holding: bool,
    pub orbit_axis: [f32; 3],
    pub orbit_angle_per_tick: f32,
    pub orbit_rotation: [f32; 4],
    pub orbit_speed: f32,
    pub expected_position: [f32; 3],
    pub ticks: u32,
    pub tracking_misses: u8,
    pub arrival_radius: f32,
}
impl State {
    pub fn new(c: Settings, p: Vec3, velocity: Vec3, radius: f32) -> Self {
        let heading = if velocity.length_squared() > 1e-8 {
            velocity.normalize()
        } else {
            c.initial_direction.normalize()
        };
        Self {
            heading: heading.into(),
            aim: heading.into(),
            velocity: velocity.into(),
            holding: false,
            orbit_axis: Vec3::Y.into(),
            orbit_angle_per_tick: 0.,
            orbit_rotation: Quat::IDENTITY.to_array(),
            orbit_speed: 0.,
            expected_position: p.into(),
            ticks: 0,
            tracking_misses: 0,
            arrival_radius: radius,
        }
    }
    pub fn tracks(&mut self, own: Vec3) -> bool {
        let error = own.distance(self.expected_position.into());
        self.tracking_misses = if error > 0.005 {
            self.tracking_misses.saturating_add(1)
        } else {
            0
        };
        error <= 0.15 && self.tracking_misses < 6
    }
    pub fn begin_holding(&mut self, c: Settings) -> std::result::Result<u32, &'static str> {
        let v = Vec3::from(self.velocity);
        let speed = v.length();
        if speed < 0.001 {
            return Err("flight_holding_requires_motion");
        }
        let dir = v / speed;
        let reference = if dir.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let axis = (reference - dir * reference.dot(dir)).normalize();
        // Half the limits leaves numerical room for the discrete circular integration.
        let turn = (c.turn * 0.5).min(c.acceleration.min(c.deceleration) / speed * 0.5);
        let ticks = (std::f32::consts::TAU / (turn * FIXED_DT)).ceil() as u32;
        if ticks > MAX_ORBIT_TICKS {
            return Err("flight_holding_preview_limit");
        }
        self.holding = true;
        self.orbit_axis = axis.into();
        // Integral number of steps closes the previewed orbit deterministically.
        self.orbit_angle_per_tick = std::f32::consts::TAU / ticks as f32;
        self.orbit_rotation = Quat::from_axis_angle(axis, self.orbit_angle_per_tick).to_array();
        self.orbit_speed = speed;
        Ok(ticks)
    }
    pub fn orbit(&mut self, own: Vec3) -> Vec3 {
        let direction =
            (Quat::from_array(self.orbit_rotation) * Vec3::from(self.heading)).normalize();
        let v = direction * self.orbit_speed;
        self.velocity = v.into();
        self.heading = direction.into();
        self.aim = self.heading;
        self.expected_position = (own + v * FIXED_DT).into();
        self.ticks = self.ticks.saturating_add(1);
        v
    }
    /// Returns true only after reaching the radius and (for hover) braking to rest.
    pub fn advance(&mut self, c: Settings, max_speed: f32, own: Vec3, end: Vec3) -> bool {
        self.advance_guided(c, max_speed, own, end, (end - own).normalize_or_zero(), 1.)
    }
    /// Local guidance is an input to the motor, never a correction to its output.
    pub(crate) fn advance_guided(
        &mut self,
        c: Settings,
        max_speed: f32,
        own: Vec3,
        end: Vec3,
        desired: Vec3,
        speed_scale: f32,
    ) -> bool {
        let delta = end - own;
        let distance = delta.length();
        let old = Vec3::from(self.velocity);
        let speed = old.length();
        if distance <= self.arrival_radius && (!c.can_hover || speed <= c.deceleration * FIXED_DT) {
            if c.can_hover {
                self.velocity = [0.; 3];
            }
            self.expected_position = own.into();
            return true;
        }
        let heading = Vec3::from(self.heading);
        let previous_aim = Vec3::from(self.aim);
        let alpha = FIXED_DT / (c.smoothing + FIXED_DT);
        let aim = if previous_aim.dot(desired) < -0.9999 {
            rotate_towards(previous_aim, desired, std::f32::consts::PI * alpha)
        } else {
            (previous_aim * (1. - alpha) + desired * alpha).normalize_or_zero()
        };
        self.aim = aim.into();
        let remaining = (distance - self.arrival_radius * 0.5).max(0.);
        let braking = (2. * c.deceleration * remaining).sqrt() * 0.65;
        let alignment = heading.dot(desired).max(0.);
        let mut target_speed = (max_speed * speed_scale).min(braking);
        if c.can_hover {
            target_speed = target_speed.min(max_speed * alignment);
            if distance > max_speed * max_speed / (2. * c.deceleration) + self.arrival_radius
                && alignment > 0.8
            {
                target_speed = target_speed.max(c.minimum_speed.min(max_speed * speed_scale));
            }
        } else {
            target_speed = target_speed.max(c.minimum_speed);
        }
        let rate = if target_speed > speed {
            c.acceleration
        } else if target_speed < speed {
            c.deceleration
        } else {
            c.acceleration.min(c.deceleration)
        };
        let budget = rate * FIXED_DT;
        let turning = heading.dot(aim) < 0.9999995;
        let speed_budget = budget * if turning { 0.5 } else { 1. };
        let next_speed = speed + (target_speed - speed).clamp(-speed_budget, speed_budget);
        let allowed = if speed * next_speed > 1e-10 {
            ((speed * speed + next_speed * next_speed - budget * budget)
                / (2. * speed * next_speed))
                .clamp(-1., 1.)
                .acos()
        } else {
            std::f32::consts::PI
        };
        let next_heading = rotate_towards(heading, aim, (c.turn * FIXED_DT).min(allowed));
        let v = next_heading * next_speed;
        self.heading = next_heading.into();
        self.velocity = v.into();
        self.expected_position = (own + v * FIXED_DT).into();
        self.ticks = self.ticks.saturating_add(1);
        false
    }
}
fn rotate_towards(from: Vec3, to: Vec3, limit: f32) -> Vec3 {
    if to.length_squared() < 0.5 {
        return from;
    }
    let angle = from.dot(to).clamp(-1., 1.).acos();
    if angle <= limit || angle < 1e-6 {
        return to;
    }
    let cross = from.cross(to);
    let axis = if cross.length_squared() > 1e-10 {
        cross.normalize()
    } else {
        from.cross(if from.y.abs() < 0.9 { Vec3::Y } else { Vec3::X })
            .normalize()
    };
    (Quat::from_axis_angle(axis, limit) * from).normalize()
}

pub struct Plan {
    pub initial: State,
    pub seconds: f32,
    pub distance: f32,
}
#[allow(clippy::too_many_arguments)]
pub fn preview(
    c: Settings,
    max_speed: f32,
    own: Vec3,
    velocity: Vec3,
    end: Vec3,
    radius: f32,
    path: &mut Vec<[f32; 3]>,
    clear: impl Fn(Vec3, Vec3) -> bool,
) -> std::result::Result<Plan, &'static str> {
    path.clear();
    if velocity.length() > max_speed + 0.001 {
        return Err("flight_initial_speed_exceeds_limit");
    }
    path.push(own.into());
    let initial = State::new(c, own, velocity, radius);
    let mut state = initial;
    let mut p = own;
    let mut distance = 0.;
    for tick in 0..MAX_TRAVEL_TICKS {
        if state.advance(c, max_speed, p, end) {
            path.push(p.into());
            if !c.can_hover {
                let steps = state.begin_holding(c)?;
                for i in 0..steps {
                    let next = p + state.orbit(p) * FIXED_DT;
                    if !clear(p, next) {
                        return Err("flight_holding_blocked");
                    }
                    p = next;
                    if i % steps.div_ceil(60) == 0 && path.len() < MAX_GUIDE_POINTS {
                        path.push(p.into());
                    }
                }
            }
            return Ok(Plan {
                initial,
                seconds: (tick.max(1) as f32) * FIXED_DT,
                distance,
            });
        }
        let next = p + Vec3::from(state.velocity) * FIXED_DT;
        if !clear(p, next) {
            return Err("flight_path_blocked");
        }
        distance += p.distance(next);
        p = next;
        if tick % 30 == 0 && path.len() < 65 {
            path.push(p.into());
        }
    }
    Err("flight_preview_limit")
}

pub fn validate_orbit(
    state: &mut State,
    c: Settings,
    own: Vec3,
    clear: impl Fn(Vec3, Vec3) -> bool,
) -> std::result::Result<(), &'static str> {
    let steps = state.begin_holding(c)?;
    let mut sample = *state;
    let mut p = own;
    for _ in 0..steps {
        let next = p + sample.orbit(p) * FIXED_DT;
        if !clear(p, next) {
            return Err("flight_holding_blocked");
        }
        p = next;
    }
    Ok(())
}
