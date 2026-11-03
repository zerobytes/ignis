//! Bounded deterministic XZ navigation over a single flat box surface.
use crate::model::{Entity, FIXED_DT, Scene};
use crate::nav_recovery::{Config as RecoveryConfig, Failure, State as RecoveryState};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
};

const CELL: f32 = 0.5;
const MARGIN: f32 = 0.06;
const REPLAN_TICKS: u64 = 12;

/// Navigation knows a destination, not the current position of the target entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavGoal {
    pub position: [f32; 3],
    pub excluded_entity: Option<u16>,
    pub target_half_height: f32,
    pub stop_distance: f32,
    pub remaining_travel: Option<f32>,
    pub replan_on_target_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NavState {
    pub status: String,
    pub reason: String,
    pub path: Vec<[f32; 3]>,
    pub planned_tick: Option<u64>,
    pub decision_tick: u64,
    pub target_position: Option<[f32; 3]>,
    pub effective_stop_distance: f32,
    #[serde(default)]
    pub recovery: RecoveryState,
}
impl Default for NavState {
    fn default() -> Self {
        Self {
            status: "pending".into(),
            reason: "awaiting_step".into(),
            path: vec![],
            planned_tick: None,
            decision_tick: 0,
            target_position: None,
            effective_stop_distance: 0.,
            recovery: RecoveryState::default(),
        }
    }
}
impl NavState {
    pub(crate) fn idle(&mut self) {
        self.recovery = RecoveryState::default();
        self.target_position = None;
        self.effective_stop_distance = 0.;
        self.planned_tick = None;
        self.stop("idle", "no_known_goal");
    }
    fn failed(&mut self, config: Option<RecoveryConfig>, tick: u64, failure: Failure) {
        if let Some(config) = config {
            self.recovery.failed(config, tick, failure);
            self.planned_tick = None;
            self.stop("retry_wait", failure.reason());
        } else {
            self.stop("unreachable", failure.reason());
        }
    }
    fn stop(&mut self, status: &str, reason: &str) {
        status.clone_into(&mut self.status);
        reason.clone_into(&mut self.reason);
        self.path.clear();
    }
}
#[derive(Clone, Copy, Default)]
struct Rect {
    min: Vec2,
    max: Vec2,
}
impl Rect {
    fn contains(&self, p: Vec2) -> bool {
        p.cmpge(self.min).all() && p.cmple(self.max).all()
    }
    // Slab test of the whole swept body centre, not just its endpoint.
    fn intersects(&self, a: Vec2, b: Vec2) -> bool {
        let d = b - a;
        let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
        for axis in 0..2 {
            if d[axis].abs() < 1e-7 {
                if a[axis] < self.min[axis] || a[axis] > self.max[axis] {
                    return false;
                }
            } else {
                let t1 = (self.min[axis] - a[axis]) / d[axis];
                let t2 = (self.max[axis] - a[axis]) / d[axis];
                enter = enter.max(t1.min(t2));
                leave = leave.min(t1.max(t2));
                if enter > leave {
                    return false;
                }
            }
        }
        true
    }
}
fn xz(p: [f32; 3]) -> Vec2 {
    Vec2::new(p[0], p[2])
}
#[derive(Default)]
struct World {
    bounds: Rect,
    blocks: Vec<Rect>,
    sight_blocks: Vec<Rect>,
    surface: Rect,
    height: f32,
}
#[derive(Default)]
pub struct NavigationScratch {
    world: World,
    search: SearchScratch,
}
#[derive(Default)]
struct SearchScratch {
    free: Vec<bool>,
    cost: Vec<usize>,
    previous: Vec<usize>,
    queue: BinaryHeap<Reverse<(usize, usize, usize)>>,
}
impl World {
    /// Leave a contact margin without crossing a wall or consulting the hidden goal.
    /// Rapier may resolve resting contacts a few millimetres inside the ideal box.
    fn recovery(&self, start: Vec2) -> Option<Vec2> {
        const CONTACT_SLOP: f32 = 0.005;
        for direction in [
            Vec2::X,
            Vec2::NEG_X,
            Vec2::Y,
            Vec2::NEG_Y,
            Vec2::new(1., 1.),
            Vec2::new(1., -1.),
            Vec2::new(-1., 1.),
            Vec2::new(-1., -1.),
        ] {
            let end = start + direction.normalize() * 0.12;
            if !self.clear(end, end) {
                continue;
            }
            let escapes = self.blocks.iter().all(|r| {
                if !r.intersects(start, end) {
                    return true;
                }
                r.contains(start)
                    && (0..2).any(|axis| {
                        (start[axis] - r.min[axis] <= MARGIN + CONTACT_SLOP
                            && end[axis] < r.min[axis])
                            || (r.max[axis] - start[axis] <= MARGIN + CONTACT_SLOP
                                && end[axis] > r.max[axis])
                    })
            });
            // Surface-boundary contacts can also violate the conservative inset.
            let within_contact = start
                .cmpge(self.bounds.min - Vec2::splat(MARGIN + CONTACT_SLOP))
                .all()
                && start
                    .cmple(self.bounds.max + Vec2::splat(MARGIN + CONTACT_SLOP))
                    .all();
            if escapes && within_contact {
                return Some(end);
            }
        }
        None
    }
    fn clear(&self, a: Vec2, b: Vec2) -> bool {
        self.bounds.contains(a)
            && self.bounds.contains(b)
            && !self.blocks.iter().any(|r| r.intersects(a, b))
    }
    // Visibility uses physical obstacles, while locomotion uses body-expanded obstacles.
    fn visible(&self, from: Vec2, target: Vec2) -> bool {
        self.surface.contains(target)
            && !self.sight_blocks.iter().any(|r| r.intersects(from, target))
    }
    fn arrival(&self, p: Vec2, target: Vec2, radius: f32) -> bool {
        self.clear(p, p) && p.distance(target) <= radius && self.visible(p, target)
    }
    fn refresh(
        &mut self,
        scene: &Scene,
        npc: &Entity,
        positions: &BTreeMap<String, [f32; 3]>,
        excluded_entity: Option<u16>,
    ) {
        let config = npc.npc.as_ref().unwrap();
        let floor = scene
            .entities
            .iter()
            .find(|e| e.id == config.surface)
            .unwrap();
        let top = floor.transform.position[1] + floor.transform.size[1] / 2.;
        let half = xz(npc.transform.size) / 2. + Vec2::splat(MARGIN);
        let floor_half = xz(floor.transform.size) / 2.;
        let center = xz(floor.transform.position);
        let bounds = Rect {
            min: center - floor_half + half,
            max: center + floor_half - half,
        };
        self.blocks.clear();
        self.sight_blocks.clear();
        for (index, e) in scene.entities.iter().enumerate() {
            if Some(index as u16) == excluded_entity
                || e.id == npc.id
                || e.id == config.surface
                || config.target.as_ref() == Some(&e.id)
                || e.body.is_none()
            {
                continue;
            }
            let Some(p) = crate::runtime_world::position(positions, &e.id) else {
                continue;
            };
            if p[1] + e.transform.size[1] / 2. <= top + 0.02
                || p[1] - e.transform.size[1] / 2. >= top + npc.transform.size[1] + MARGIN
            {
                continue;
            }
            if !e.geometry.is_box() {
                if let Some((min, max)) = crate::geometry::Volume::entity(e, p.into())
                    .horizontal_bounds(top + 0.02, top + npc.transform.size[1] + MARGIN)
                {
                    self.sight_blocks.push(Rect { min, max });
                    self.blocks.push(Rect {
                        min: min - half,
                        max: max + half,
                    });
                }
                continue;
            }
            let obstacle_half = xz(e.transform.size) / 2.;
            self.sight_blocks.push(Rect {
                min: xz(p) - obstacle_half,
                max: xz(p) + obstacle_half,
            });
            let extent = obstacle_half + half;
            self.blocks.push(Rect {
                min: xz(p) - extent,
                max: xz(p) + extent,
            });
        }
        self.bounds = bounds;
        self.surface = Rect {
            min: center - floor_half,
            max: center + floor_half,
        };
        self.height = top + npc.transform.size[1] / 2.;
    }
    fn path(
        &self,
        start: Vec2,
        goal: Vec2,
        radius: f32,
        scratch: &mut SearchScratch,
        path: &mut Vec<[f32; 3]>,
    ) {
        path.clear();
        if !self.clear(start, start) || !self.surface.contains(goal) {
            return;
        }
        // Pursuit ends near the player, never at a point occupied by the player's body.
        let approach = goal + (start - goal).normalize_or_zero() * radius;
        if self.clear(start, approach) && self.visible(approach, goal) {
            path.push([approach.x, self.height, approach.y]);
            return;
        }
        let size = self.bounds.max - self.bounds.min;
        let nx = (size.x / CELL).floor() as usize + 1;
        let nz = (size.y / CELL).floor() as usize + 1;
        if nx == 0 || nz == 0 || nx * nz > 16_641 {
            return;
        }
        let point = |i: usize| self.bounds.min + Vec2::new((i % nx) as f32, (i / nx) as f32) * CELL;
        let SearchScratch {
            free,
            cost,
            previous,
            queue,
        } = scratch;
        free.clear();
        free.extend((0..nx * nz).map(|i| self.clear(point(i), point(i))));
        let connect = |p: Vec2| -> Option<usize> {
            let local = (p - self.bounds.min) / CELL;
            let (cx, cz) = (local.x.round() as isize, local.y.round() as isize);
            let mut best: Option<usize> = None;
            for dz in -2..=2 {
                for dx in -2..=2 {
                    let (x, z) = (cx + dx, cz + dz);
                    if x >= 0 && z >= 0 && x < nx as isize && z < nz as isize {
                        let i = z as usize * nx + x as usize;
                        if free[i]
                            && self.clear(p, point(i))
                            && best.is_none_or(|b| {
                                point(i)
                                    .distance_squared(p)
                                    .total_cmp(&point(b).distance_squared(p))
                                    .then(i.cmp(&b))
                                    .is_lt()
                            })
                        {
                            best = Some(i);
                        }
                    }
                }
            }
            best
        };
        let Some(source) = connect(start) else { return };
        // Multi-goal A*: any free cell in the stop region with line of sight is valid.
        // Distance to the region is an admissible lower bound in unit-cost grid steps.
        // A short swept connector can reach a small stop region between grid cells.
        let heuristic =
            |i: usize| ((point(i).distance(goal) - radius - CELL).max(0.) / CELL).floor() as usize;
        cost.resize(nx * nz, usize::MAX);
        cost.fill(usize::MAX);
        previous.resize(nx * nz, usize::MAX);
        previous.fill(usize::MAX);
        queue.clear();
        cost[source] = 0;
        queue.push(Reverse((heuristic(source), 0, source)));
        while let Some(Reverse((_, g, i))) = queue.pop() {
            if g != cost[i] {
                continue;
            }
            let p = point(i);
            let approach = goal + (p - goal).normalize_or_zero() * radius;
            let arrived = self.arrival(p, goal, radius);
            let connect_goal = !arrived
                && p.distance_squared(approach) <= CELL * CELL
                && self.clear(p, approach)
                && self.visible(approach, goal);
            if arrived || connect_goal {
                if connect_goal {
                    path.push([approach.x, self.height, approach.y]);
                }
                let mut node = i;
                loop {
                    let p = point(node);
                    path.push([p.x, self.height, p.y]);
                    if node == source {
                        break;
                    }
                    node = previous[node];
                }
                path.reverse();
                return;
            }
            let (x, z) = (i % nx, i / nx);
            for (dx, dz) in [(-1, 0), (0, -1), (1, 0), (0, 1)] {
                let (xx, zz) = (x as isize + dx, z as isize + dz);
                if xx < 0 || zz < 0 || xx >= nx as isize || zz >= nz as isize {
                    continue;
                }
                let j = zz as usize * nx + xx as usize;
                if free[j] && g + 1 < cost[j] && self.clear(point(i), point(j)) {
                    cost[j] = g + 1;
                    previous[j] = i;
                    queue.push(Reverse((g + 1 + heuristic(j), g + 1, j)));
                }
            }
        }
    }
}

pub fn update(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    goals: &BTreeMap<String, Option<NavGoal>>,
    states: &mut BTreeMap<String, NavState>,
    velocities: &mut BTreeMap<String, [f32; 2]>,
    scratch: &mut NavigationScratch,
) {
    velocities.retain(|id, _| states.contains_key(id));
    // All agents plan from the same snapshot; sorted state keys remove file-order dependence.
    for (id, state) in states {
        let npc = scene.entities.iter().find(|e| e.id == *id).unwrap();
        let config = npc.npc.as_ref().unwrap();
        let p = positions[id];
        state.decision_tick = tick;
        if !velocities.contains_key(id) {
            velocities.insert(id.clone(), [0., 0.]);
        }
        let velocity = velocities.get_mut(id).unwrap();
        *velocity = [0., 0.];
        if config.mobility.is_some() {
            continue;
        }
        if !config.enabled {
            state.recovery = RecoveryState::default();
            state.target_position = None;
            state.effective_stop_distance = 0.;
            state.planned_tick = None;
            state.stop("disabled", "disabled_by_configuration");
            continue;
        }
        let Some(destination) = goals.get(id).and_then(Option::as_ref) else {
            state.idle();
            continue;
        };
        if state.effective_stop_distance != destination.stop_distance
            || (destination.replan_on_target_change
                && state.target_position != Some(destination.position))
        {
            state.planned_tick = None;
        }
        let t = destination.position;
        let (start, goal) = (xz(p), xz(t));
        state.target_position = Some(t);
        state.effective_stop_distance = destination.stop_distance;
        if let Some(recovery) = config.navigation_recovery {
            if state.recovery.waiting(tick) {
                // Keep the latest already-known destination, without rebuilding the map.
                continue;
            }
            if state.recovery.stalled(recovery, tick, start) {
                state.failed(Some(recovery), tick, Failure::NoProgress);
                continue;
            }
        }
        let world = &mut scratch.world;
        world.refresh(scene, npc, positions, destination.excluded_entity);
        if (p[1] - world.height).abs() > 0.3 {
            state.failed(config.navigation_recovery, tick, Failure::NpcOffSurface);
            continue;
        }
        let target_y = world.height - npc.transform.size[1] / 2. + destination.target_half_height;
        if (t[1] - target_y).abs() > 0.3 {
            state.failed(config.navigation_recovery, tick, Failure::TargetOffSurface);
            continue;
        }
        if !world.clear(start, start) {
            state.planned_tick = None;
            state.path.clear();
            if let Some(exit) = world.recovery(start) {
                let delta = exit - start;
                let mut speed = config.speed.min(delta.length() / FIXED_DT);
                if let Some(remaining) = destination.remaining_travel {
                    speed = speed.min(remaining / FIXED_DT);
                }
                let v = delta.normalize_or_zero() * speed;
                *velocity = [v.x, v.y];
                state.stop("recovering", "leaving_contact_margin");
                state.recovery.movement_requested =
                    config.navigation_recovery.is_some() && speed > 0.;
            } else {
                state.failed(config.navigation_recovery, tick, Failure::NoClearPath);
            }
            continue;
        }
        if world.arrival(start, goal, state.effective_stop_distance + 0.02) {
            state.recovery.arrived();
            state.stop("arrived", "within_stop_distance");
            continue;
        }
        let replan = state
            .planned_tick
            .is_none_or(|last| tick - last >= REPLAN_TICKS)
            || state.status == "arrived"
            || state.status == "disabled";
        if replan {
            state.planned_tick = Some(tick);
            if config.navigation_recovery.is_some() {
                state.recovery.path_attempts = state.recovery.path_attempts.saturating_add(1);
            }
            world.path(
                start,
                goal,
                state.effective_stop_distance,
                &mut scratch.search,
                &mut state.path,
            );
        }
        // Smoothing only accepts a swept segment with clearance for the entire body.
        let next = state
            .path
            .iter()
            .enumerate()
            .rev()
            .find(|(_, q)| world.clear(start, xz(**q)))
            .map(|(i, q)| (i, xz(*q)));
        let Some((index, next)) = next else {
            state.failed(config.navigation_recovery, tick, Failure::NoClearPath);
            continue;
        };
        state.path.drain(..index);
        let delta = next - start;
        let distance = delta.length();
        let mut speed = config.speed.min(distance / FIXED_DT);
        if let Some(remaining) = destination.remaining_travel {
            speed = speed.min(remaining / FIXED_DT);
        }
        if world.visible(start, goal) {
            speed = speed
                .min(((start.distance(goal) - state.effective_stop_distance) / FIXED_DT).max(0.));
        }
        let v = delta.normalize_or_zero() * speed;
        *velocity = [v.x, v.y];
        state.recovery.movement_requested = config.navigation_recovery.is_some() && speed > 0.;
        "following".clone_into(&mut state.status);
        "clear_waypoint".clone_into(&mut state.reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_recovery_respects_corners_other_walls_and_deep_overlap() {
        let mut w = World {
            bounds: Rect {
                min: Vec2::splat(-5.),
                max: Vec2::splat(5.),
            },
            blocks: vec![Rect {
                min: Vec2::splat(-1.),
                max: Vec2::splat(1.),
            }],
            ..Default::default()
        };
        for start in [
            Vec2::new(0.94, 0.),
            Vec2::new(-0.94, 0.),
            Vec2::new(0., 0.94),
            Vec2::new(0., -0.94),
        ] {
            let end = w.recovery(start).unwrap();
            assert!(w.clear(end, end));
            assert!(end.length() > start.length());
        }
        assert!(w.recovery(Vec2::ZERO).is_none());
        // A second perpendicular contact requires a diagonal escape.
        w.blocks = vec![
            Rect {
                min: Vec2::new(-5., -5.),
                max: Vec2::new(1., 5.),
            },
            Rect {
                min: Vec2::new(-5., -5.),
                max: Vec2::new(5., 1.),
            },
        ];
        let end = w.recovery(Vec2::splat(0.94)).unwrap();
        assert!(end.x > 1. && end.y > 1.);
        // Even a thin obstacle beyond the contact must not be crossed.
        w.blocks = vec![
            Rect {
                min: Vec2::splat(-1.),
                max: Vec2::splat(1.),
            },
            Rect {
                min: Vec2::new(1.01, -5.),
                max: Vec2::new(1.02, 5.),
            },
        ];
        assert!(w.recovery(Vec2::new(0.94, 0.)).is_none());
        w.blocks.clear();
        assert!(w.recovery(Vec2::new(5.06, 0.)).unwrap().x < 5.);
        assert!(w.recovery(Vec2::new(5.2, 0.)).is_none());
    }
}
