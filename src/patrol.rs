//! Authored routines provide goals only when perception has no live clue.
use crate::{
    model::{Entity, Result, Scene, invalid},
    navigation::{NavGoal, NavState},
    perception::NpcState,
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Route {
    Points {
        points: Vec<[f32; 2]>,
    },
    Ring {
        center: [f32; 2],
        radius: f32,
        points: u8,
        seed: u32,
    },
    /// Body-center coordinates, independent of the configured legacy floor.
    #[serde(rename = "points_3d")]
    Points3d {
        points: Vec<[f32; 3]>,
    },
    #[serde(rename = "ring_3d")]
    Ring3d {
        center: [f32; 3],
        normal: [f32; 3],
        radius: f32,
        points: u8,
        seed: u32,
    },
}
impl Route {
    pub fn is_spatial(&self) -> bool {
        matches!(self, Self::Points3d { .. } | Self::Ring3d { .. })
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Traversal {
    Loop,
    PingPong,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub route: Route,
    pub traversal: Traversal,
    pub wait_ticks: u32,
    pub travel_ticks: u32,
    pub retry_ticks: u32,
    pub arrival_distance: f32,
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        let coordinates = |p: &[f32; 2]| p.iter().all(|n| n.is_finite() && n.abs() <= 10000.);
        let valid = match &self.route {
            Route::Points { points } => {
                (1..=16).contains(&points.len()) && points.iter().all(coordinates)
            }
            Route::Ring {
                center,
                radius,
                points,
                ..
            } => {
                coordinates(center)
                    && radius.is_finite()
                    && (0.5..=32.).contains(radius)
                    && (1..=16).contains(points)
            }
            Route::Points3d { points } => {
                (1..=16).contains(&points.len())
                    && points
                        .iter()
                        .flatten()
                        .all(|n| n.is_finite() && n.abs() <= 10000.)
            }
            Route::Ring3d {
                center,
                normal,
                radius,
                points,
                ..
            } => {
                radius.is_finite()
                    && (0.5..=32.).contains(radius)
                    && center
                        .iter()
                        .all(|n| n.is_finite() && n.abs() + radius <= 10000.)
                    && normal.iter().all(|n| n.is_finite())
                    && (0.0001..=10000.).contains(&Vec3::from(*normal).length_squared())
                    && (1..=16).contains(points)
            }
        };
        if !valid
            || self.wait_ticks > 3600
            || !(1..=36000).contains(&self.travel_ticks)
            || !(1..=3600).contains(&self.retry_ticks)
            || !self.arrival_distance.is_finite()
            || !(0.1..=2.).contains(&self.arrival_distance)
        {
            return Err(invalid(
                "Patrol requires 1..16 finite XZ/XYZ points within +/-10000m or ring radius [0.5,32]m; ring3d center plus radius within +/-10000m and normal length [0.01,100]; wait [0,3600], travel [1,36000], retry [1,3600] ticks; arrival [0.1,2]m",
            ));
        }
        Ok(())
    }
    pub fn resolve(&self, height: f32) -> Vec<[f32; 3]> {
        match &self.route {
            Route::Points3d { points } => points.clone(),
            Route::Ring3d {
                center,
                normal,
                radius,
                points,
                seed,
            } => {
                let normal = Vec3::from(*normal).normalize();
                let projected = Vec3::NEG_Z - normal * normal.dot(Vec3::NEG_Z);
                let u = if projected.length_squared() > 1e-6 {
                    projected.normalize()
                } else {
                    (Vec3::X - normal * normal.x).normalize()
                };
                let v = u.cross(normal);
                (0..*points)
                    .map(|i| {
                        let (sin, cos) = (((*seed % 360) as f32
                            + f32::from(i) * 360. / f32::from(*points))
                        .to_radians())
                        .sin_cos();
                        (Vec3::from(*center) + (u * cos + v * sin) * *radius).to_array()
                    })
                    .collect()
            }
            Route::Points { points } => points.iter().map(|p| [p[0], height, p[1]]).collect(),
            Route::Ring {
                center,
                radius,
                points,
                seed,
            } => (0..*points)
                .map(|i| {
                    let angle = ((*seed % 360) as f32 + f32::from(i) * 360. / f32::from(*points))
                        .to_radians();
                    [
                        center[0] + angle.sin() * radius,
                        height,
                        center[1] - angle.cos() * radius,
                    ]
                })
                .collect(),
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Ready,
    Moving,
    Waiting,
    Suspended,
    Disabled,
    Cooldown,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Failure {
    Unreachable,
    TravelTimeout,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub phase: Phase,
    pub point_index: u8,
    pub direction: i8,
    pub leg_started_tick: Option<u64>,
    pub wait_started_tick: Option<u64>,
    pub retry_at_tick: Option<u64>,
    pub visited: u32,
    pub skipped: u32,
    pub consecutive_failures: u8,
    pub failed_points: u16,
    pub last_failure: Option<Failure>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Ready,
            point_index: 0,
            direction: 1,
            leg_started_tick: None,
            wait_started_tick: None,
            retry_at_tick: None,
            visited: 0,
            skipped: 0,
            consecutive_failures: 0,
            failed_points: 0,
            last_failure: None,
        }
    }
}
impl State {
    fn interrupt(&mut self, phase: Phase) {
        self.phase = phase;
        self.leg_started_tick = None;
        self.wait_started_tick = None;
        self.retry_at_tick = None;
        self.consecutive_failures = 0;
        self.failed_points = 0;
    }
    fn advance(&mut self, count: usize, traversal: Traversal) {
        self.point_index = match traversal {
            Traversal::Loop => (usize::from(self.point_index) + 1).rem_euclid(count) as u8,
            Traversal::PingPong if count == 1 => 0,
            Traversal::PingPong => {
                if usize::from(self.point_index) == count - 1 {
                    self.direction = -1;
                } else if self.point_index == 0 {
                    self.direction = 1;
                }
                (self.point_index as i8 + self.direction) as u8
            }
        };
        self.phase = Phase::Ready;
        self.leg_started_tick = None;
        self.wait_started_tick = None;
    }
}

pub fn routes(scene: &Scene) -> BTreeMap<String, Vec<[f32; 3]>> {
    scene
        .entities
        .iter()
        .filter_map(|e| {
            let npc = e.npc.as_ref()?;
            let config = npc.patrol.as_ref()?;
            let floor = scene.entities.iter().find(|f| f.id == npc.surface).unwrap();
            let height = floor.transform.position[1]
                + floor.transform.size[1] * 0.5
                + e.transform.size[1] * 0.5;
            Some((e.id.clone(), config.resolve(height)))
        })
        .collect()
}

/// No target position or world snapshot is available to this routine.
pub fn update(
    scene: &Scene,
    routes: &BTreeMap<String, Vec<[f32; 3]>>,
    tick: u64,
    states: &mut BTreeMap<String, State>,
    perception: &BTreeMap<String, NpcState>,
    navigation: &BTreeMap<String, NavState>,
    goals: &mut BTreeMap<String, Option<NavGoal>>,
) {
    for (id, state) in states {
        let npc = scene.entities.iter().find(|e| e.id == *id).unwrap();
        let config = npc.npc.as_ref().unwrap();
        if !config.enabled || perception[id].phase == crate::perception::Phase::Dead {
            state.interrupt(Phase::Disabled);
            continue;
        }
        let selected = if config.objectives.is_some() {
            perception[id].objectives.action == Some(crate::objectives::Kind::Patrol)
        } else {
            perception[id].memory.is_none()
        };
        if !selected || perception[id].territory.blocked() || perception[id].locomotion_locked {
            state.interrupt(Phase::Suspended);
            continue;
        }
        let settings = config.patrol.as_ref().unwrap();
        *goals.get_mut(id).unwrap() =
            decide(npc, settings, &routes[id], tick, &navigation[id], state);
    }
}
fn decide(
    npc: &Entity,
    config: &Config,
    points: &[[f32; 3]],
    tick: u64,
    nav: &NavState,
    state: &mut State,
) -> Option<NavGoal> {
    if state.phase == Phase::Cooldown {
        if tick < state.retry_at_tick.unwrap() {
            return None;
        }
        state.retry_at_tick = None;
        state.consecutive_failures = 0;
        state.failed_points = 0;
        state.phase = Phase::Ready;
    }
    if state.phase == Phase::Waiting {
        if tick - state.wait_started_tick.unwrap() >= u64::from(config.wait_ticks) {
            state.advance(points.len(), config.traversal);
        }
        return None;
    }
    let point = points[usize::from(state.point_index)];
    if state.phase == Phase::Moving {
        let started = state.leg_started_tick.unwrap();
        // A previous pursuit can have the same destination; only consume this leg's result.
        // Spatial navigation converts centers to feet and back; keep its physical tolerance.
        // A partial result remains unreachable and must never count as a visit.
        let same_goal = nav.target_position.is_some_and(|p| {
            if npc.npc.as_ref().unwrap().mobility.is_some() {
                Vec3::from(p).abs_diff_eq(Vec3::from(point), crate::mobility::NODE_ALIGNMENT)
            } else {
                p == point
            }
        });
        let own_result = nav.decision_tick >= started && same_goal;
        if own_result && nav.status == "arrived" {
            state.visited += 1;
            state.consecutive_failures = 0;
            state.failed_points = 0;
            state.phase = Phase::Waiting;
            state.wait_started_tick = Some(tick);
            state.leg_started_tick = None;
            if config.wait_ticks == 0 {
                state.advance(points.len(), config.traversal);
            }
            return None;
        }
        let failure = if own_result && nav.status == "unreachable" {
            Some(Failure::Unreachable)
        } else if tick - started >= u64::from(config.travel_ticks) {
            Some(Failure::TravelTimeout)
        } else {
            None
        };
        if let Some(failure) = failure {
            state.skipped += 1;
            state.consecutive_failures += 1;
            state.failed_points |= 1 << state.point_index;
            state.last_failure = Some(failure);
            state.advance(points.len(), config.traversal);
            if state.failed_points.count_ones() as usize == points.len() {
                state.phase = Phase::Cooldown;
                state.retry_at_tick = Some(tick + u64::from(config.retry_ticks));
            }
            return None;
        }
    } else {
        state.phase = Phase::Moving;
        state.leg_started_tick = Some(tick);
    }
    Some(NavGoal {
        excluded_entity: None,
        position: point,
        target_half_height: npc.transform.size[1] * 0.5,
        stop_distance: config.arrival_distance,
        remaining_travel: None,
        replan_on_target_change: true,
    })
}
