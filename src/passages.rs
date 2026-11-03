//! Explicit cooperative bottlenecks. This scheduler never writes sensory knowledge.
use crate::{
    model::{BodyKind, Result, Scene, invalid},
    navigation::{NavGoal, NavState},
    perception::NpcState,
};
use glam::Vec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub id: String,
    pub surface: String,
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub width: f32,
    pub wait_a: Vec<[f32; 2]>,
    pub wait_b: Vec<[f32; 2]>,
    pub lease_ticks: u32,
    pub retry_ticks: u32,
    pub aging_ticks: u32,
}
impl Config {
    pub fn direction(&self) -> Vec2 {
        // Validation guarantees a nonzero axis-aligned segment. No per-tick sqrt.
        if self.a[0] != self.b[0] {
            Vec2::new((self.b[0] - self.a[0]).signum(), 0.)
        } else {
            Vec2::new(0., (self.b[1] - self.a[1]).signum())
        }
    }
    fn length(&self) -> f32 {
        (self.b[0] - self.a[0]).abs() + (self.b[1] - self.a[1]).abs()
    }
    fn local(&self, p: Vec2) -> Vec2 {
        let d = self.direction();
        let q = p - Vec2::from_array(self.a);
        Vec2::new(q.dot(d), q.dot(Vec2::new(-d.y, d.x)))
    }
    fn waits(&self, side: i8) -> &[[f32; 2]] {
        if side < 0 { &self.wait_a } else { &self.wait_b }
    }
    fn request(&self, p: Vec2, g: Vec2, half: f32) -> Option<i8> {
        let p = self.local(p);
        let g = self.local(g);
        let middle = self.length() * 0.5;
        let side = if p.x < middle { -1 } else { 1 };
        if (side < 0 && g.x <= self.length() + half + 0.2) || (side > 0 && g.x >= -half - 0.2) {
            return None;
        }
        let t = (middle - p.x) / (g.x - p.x);
        if !(0. ..=1.).contains(&t) || (p.y + (g.y - p.y) * t).abs() > self.width * 0.5 {
            return None;
        }
        Some(side)
    }
    fn exit(&self, side: i8, half: f32) -> Vec2 {
        if side < 0 {
            Vec2::from_array(self.b) + self.direction() * (half + 0.25)
        } else {
            Vec2::from_array(self.a) - self.direction() * (half + 0.25)
        }
    }
    fn passed(&self, side: i8, p: Vec2, half: f32) -> bool {
        let x = self.local(p).x;
        if side < 0 {
            x >= self.length() + half + 0.05
        } else {
            x <= -half - 0.05
        }
    }
}
fn point(p: [f32; 3]) -> Vec2 {
    Vec2::new(p[0], p[2])
}

pub fn validate(scene: &Scene) -> Result<()> {
    if scene.passages.len() > 8
        || (scene.version < 18
            && (!scene.passages.is_empty()
                || scene
                    .entities
                    .iter()
                    .any(|e| e.npc.as_ref().is_some_and(|n| n.passage_priority.is_some()))))
    {
        return Err(invalid("Passages require scene18 and at most8 passages"));
    }
    for (i, c) in scene.passages.iter().enumerate() {
        let delta = Vec2::from_array(c.b) - Vec2::from_array(c.a);
        if c.id.is_empty()
            || c.id.len() > 64
            || scene.passages[..i].iter().any(|p| p.id == c.id)
            || !c
                .a
                .iter()
                .chain(c.b.iter())
                .all(|x| x.is_finite() && x.abs() <= 10000.)
            || !((delta.x == 0.) ^ (delta.y == 0.))
            || !(2. ..=16.).contains(&delta.length())
            || !c.width.is_finite()
            || !(0.8..=8.).contains(&c.width)
            || !(30..=3600).contains(&c.lease_ticks)
            || !(1..=3600).contains(&c.retry_ticks)
            || !(1..=3600).contains(&c.aging_ticks)
        {
            return Err(invalid(
                "Passage requires unique id, finite axis-aligned endpoints 2..16m apart, width0.8..8m, lease30..3600 and retry/aging1..3600ticks",
            ));
        }
        let floor = scene
            .entities
            .iter()
            .find(|e| e.id == c.surface && e.body == Some(BodyKind::Fixed) && e.geometry.is_box())
            .ok_or_else(|| invalid("Passage surface must reference fixed body"))?;
        let participants = scene.entities.iter().filter(|e| {
            e.npc
                .as_ref()
                .is_some_and(|n| n.passage_priority.is_some() && n.surface == c.surface)
        });
        let count = participants.clone().count().max(1);
        let half = participants
            .clone()
            .map(|e| e.transform.size[0].max(e.transform.size[2]) * 0.5)
            .fold(0.35, f32::max);
        if c.width < half * 2. + 0.12 {
            return Err(invalid(
                "Passage width must fit participating bodies and navigation margin",
            ));
        }
        for (side, waits) in [(-1, &c.wait_a), (1, &c.wait_b)] {
            if waits.len() < count || waits.len() > 8 {
                return Err(invalid(
                    "Each passage side needs 1..8 waiting slots, at least one per participant on its surface",
                ));
            }
            for (j, p) in waits.iter().enumerate() {
                let local = c.local(Vec2::from_array(*p));
                if !p.iter().all(|x| x.is_finite() && x.abs() <= 10000.)
                    || (side < 0 && local.x > -half - 0.3)
                    || (side > 0 && local.x < c.length() + half + 0.3)
                    || local.y.abs() < c.width * 0.5 + half + 0.2
                    || waits[..j].iter().any(|q| {
                        Vec2::from_array(*p).distance_squared(Vec2::from_array(*q))
                            < (2. * half + 0.2).powi(2)
                    })
                {
                    return Err(invalid(
                        "Waiting slots must be distinct, outside the passage exit lane on their own side",
                    ));
                }
            }
        }
        for p in c
            .wait_a
            .iter()
            .chain(c.wait_b.iter())
            .copied()
            .chain([c.exit(-1, half).to_array(), c.exit(1, half).to_array()])
        {
            let relative = Vec2::from_array(p) - point(floor.transform.position);
            let bound = point(floor.transform.size) * 0.5 - Vec2::splat(half + 0.06);
            if !relative.abs().cmple(bound).all() {
                return Err(invalid("Passage exits/waiting slots must fit the surface"));
            }
            if participants.clone().any(|e| {
                e.npc
                    .as_ref()
                    .unwrap()
                    .territory
                    .is_some_and(|t| !t.contains([p[0], 0., p[1]]))
            }) {
                return Err(invalid(
                    "Passage waypoints must fit participant territories",
                ));
            }
        }
        let lo = Vec2::from_array(c.a).min(Vec2::from_array(c.b)) - Vec2::splat(c.width * 0.5);
        let hi = Vec2::from_array(c.a).max(Vec2::from_array(c.b)) + Vec2::splat(c.width * 0.5);
        if scene.passages[..i]
            .iter()
            .filter(|p| p.surface == c.surface)
            .any(|p| {
                let pl =
                    Vec2::from_array(p.a).min(Vec2::from_array(p.b)) - Vec2::splat(p.width * 0.5);
                let ph =
                    Vec2::from_array(p.a).max(Vec2::from_array(p.b)) + Vec2::splat(p.width * 0.5);
                lo.cmplt(ph).all() && hi.cmpgt(pl).all()
            })
        {
            return Err(invalid(
                "Passage envelopes on the same surface must not overlap",
            ));
        }
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Idle,
    Waiting,
    Crossing,
    Cooldown,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    #[default]
    NoRequest,
    Queue,
    Granted,
    Occupied,
    LeaseExpired,
    Completed,
    Cancelled,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    pub entity_index: u16,
    pub passage: Option<u8>,
    pub side: i8,
    pub slot: u8,
    pub phase: Phase,
    pub reason: Reason,
    pub requested_tick: u64,
    pub retry_at_tick: u64,
    pub effective_priority: u64,
    pub original_goal: Option<[f32; 3]>,
    pub waypoint: Option<[f32; 3]>,
    pub grants: u32,
    pub completions: u32,
    pub timeouts: u32,
}
impl Agent {
    fn cancel(&mut self) {
        self.passage = None;
        self.phase = Phase::Idle;
        self.reason = Reason::Cancelled;
        self.original_goal = None;
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    pub owner: Option<u16>,
    pub granted_tick: u64,
    pub occupied: bool,
    pub grants: u32,
    pub timeouts: u32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Runtime {
    pub agents: BTreeMap<String, Agent>,
    pub slots: Vec<Slot>,
}
impl Runtime {
    pub(crate) fn release_actor(&mut self, id: &str) {
        if let Some(agent) = self.agents.get_mut(id) {
            for slot in &mut self.slots {
                if slot.owner == Some(agent.entity_index) {
                    slot.owner = None;
                }
            }
            agent.cancel();
            agent.retry_at_tick = 0;
            agent.waypoint = None;
        }
    }
    pub fn new(scene: &Scene) -> Self {
        Self {
            agents: scene
                .entities
                .iter()
                .enumerate()
                .filter(|(_, e)| e.npc.as_ref().is_some_and(|n| n.passage_priority.is_some()))
                .map(|(i, e)| {
                    (
                        e.id.clone(),
                        Agent {
                            entity_index: i as u16,
                            ..Default::default()
                        },
                    )
                })
                .collect(),
            slots: vec![Slot::default(); scene.passages.len()],
        }
    }
    pub fn yielding(&self, id: &str) -> bool {
        self.agents
            .get(id)
            .is_some_and(|a| matches!(a.phase, Phase::Waiting | Phase::Cooldown))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        scene: &Scene,
        actors: &crate::targeting::Catalog,
        positions: &BTreeMap<String, [f32; 3]>,
        tick: u64,
        npcs: &BTreeMap<String, NpcState>,
        goals: &mut BTreeMap<String, Option<NavGoal>>,
        navigation: &mut BTreeMap<String, NavState>,
    ) {
        // Resolve intent first for everyone; grants never depend on scene file order.
        for (id, a) in &mut self.agents {
            let e = &scene.entities[a.entity_index as usize];
            let n = e.npc.as_ref().unwrap();
            let goal = goals.get(id).and_then(Option::as_ref);
            let p = point(positions[id]);
            let half = e.transform.size[0].max(e.transform.size[2]) * 0.5;
            if !n.enabled
                || goal.is_none()
                || npcs[id].objectives.changed_this_tick
                || npcs[id].territory.changed_this_tick
            {
                a.cancel();
            }
            if !n.enabled || goal.is_none() {
                continue;
            }
            let g = goal.unwrap();
            if let Some(index) = a.passage {
                let c = &scene.passages[index as usize];
                let local = c.local(point(g.position));
                if c.passed(a.side, p, half) {
                    if a.phase == Phase::Crossing {
                        a.completions = a.completions.saturating_add(1);
                    }
                    a.cancel();
                    a.reason = Reason::Completed;
                } else if (a.side < 0 && local.x <= c.length() + half + 0.2)
                    || (a.side > 0 && local.x >= -half - 0.2)
                {
                    a.cancel();
                }
            }
            if a.passage.is_none() {
                // Nearest declared crossing on the known straight intent segment.
                let mut best: Option<(usize, i8, f32)> = None;
                for (i, c) in scene
                    .passages
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.surface == n.surface)
                {
                    if let Some(side) = c.request(p, point(g.position), half) {
                        let d = p.distance_squared(
                            (Vec2::from_array(c.a) + Vec2::from_array(c.b)) * 0.5,
                        );
                        if best.is_none_or(|(j, _, old)| {
                            d < old || (d == old && c.id < scene.passages[j].id)
                        }) {
                            best = Some((i, side, d));
                        }
                    }
                }
                if let Some((i, side, _)) = best {
                    a.passage = Some(i as u8);
                    a.side = side;
                    a.phase = Phase::Waiting;
                    a.requested_tick = tick;
                    a.retry_at_tick = 0;
                    a.slot = u8::MAX;
                }
            }
            a.original_goal = Some(g.position);
            if let Some(i) = a.passage {
                let c = &scene.passages[i as usize];
                a.effective_priority = u64::from(n.passage_priority.unwrap())
                    + tick.saturating_sub(a.requested_tick) / u64::from(c.aging_ticks);
                if a.phase == Phase::Cooldown && tick >= a.retry_at_tick {
                    a.phase = Phase::Waiting;
                }
            }
        }
        // Assign stable waiting slots once; capacities were validated at authoring time.
        for index in 0..self.slots.len() {
            for side in [-1, 1] {
                let mut used = 0u16;
                for a in self
                    .agents
                    .values()
                    .filter(|a| a.passage == Some(index as u8) && a.side == side && a.slot < 8)
                {
                    used |= 1 << a.slot;
                }
                for a in self.agents.values_mut().filter(|a| {
                    a.passage == Some(index as u8) && a.side == side && a.slot == u8::MAX
                }) {
                    let slot = used.trailing_ones() as u8;
                    debug_assert!(slot < 8);
                    a.slot = slot;
                    used |= 1 << slot;
                }
            }
        }
        for (i, slot) in self.slots.iter_mut().enumerate() {
            let c = &scene.passages[i];
            if let Some(owner) = slot.owner {
                let id = &scene.entities[owner as usize].id;
                let a = &mut self.agents.get_mut(id).unwrap();
                if a.passage != Some(i as u8) || a.phase != Phase::Crossing {
                    slot.owner = None;
                } else if tick.saturating_sub(slot.granted_tick) >= u64::from(c.lease_ticks) {
                    a.phase = Phase::Cooldown;
                    a.reason = Reason::LeaseExpired;
                    a.retry_at_tick = tick + u64::from(c.retry_ticks);
                    a.timeouts = a.timeouts.saturating_add(1);
                    slot.timeouts = slot.timeouts.saturating_add(1);
                    slot.owner = None;
                }
            }
            // Physical occupancy is a safety gate, including non-participants and disabled NPCs.
            let floor = scene.entities.iter().find(|e| e.id == c.surface).unwrap();
            let top = floor.transform.position[1] + floor.transform.size[1] * 0.5;
            slot.occupied = scene.entities.iter().enumerate().any(|(index, e)| {
                if Some(index as u16) == slot.owner
                    || !matches!(e.body, Some(BodyKind::Player | BodyKind::Npc))
                {
                    return false;
                }
                let Some(pos) = crate::runtime_world::position(positions, &e.id) else {
                    return false;
                };
                if pos[1] + e.transform.size[1] * 0.5 <= top
                    || pos[1] - e.transform.size[1] * 0.5 >= top + 3.
                {
                    return false;
                }
                let p = c.local(point(pos));
                let half = e.transform.size[0].max(e.transform.size[2]) * 0.5;
                p.x + half > 0. && p.x - half < c.length() && p.y.abs() < c.width * 0.5 + half
            });
            if slot.owner.is_none() && !slot.occupied {
                let mut best: Option<(u16, u64)> = None;
                for a in self.agents.values() {
                    if a.passage == Some(i as u8)
                        && a.phase == Phase::Waiting
                        && best.is_none_or(|(old, score)| {
                            a.effective_priority > score
                                || (a.effective_priority == score
                                    && actors.instance_order(actors.slot(a.entity_index as usize))
                                        < actors.instance_order(actors.slot(old as usize)))
                        })
                    {
                        best = Some((a.entity_index, a.effective_priority));
                    }
                }
                if let Some((index, _)) = best {
                    let a = self
                        .agents
                        .get_mut(&scene.entities[index as usize].id)
                        .unwrap();
                    a.phase = Phase::Crossing;
                    a.reason = Reason::Granted;
                    a.grants = a.grants.saturating_add(1);
                    slot.owner = Some(a.entity_index);
                    slot.granted_tick = tick;
                    slot.grants = slot.grants.saturating_add(1);
                }
            }
        }
        for (id, a) in &mut self.agents {
            let previous = a.waypoint;
            if a.phase == Phase::Waiting {
                a.reason = if self.slots[a.passage.unwrap() as usize].occupied {
                    Reason::Occupied
                } else {
                    Reason::Queue
                };
            }
            a.waypoint = a.passage.map(|i| {
                let e = &scene.entities[a.entity_index as usize];
                let c = &scene.passages[i as usize];
                let half = e.transform.size[0].max(e.transform.size[2]) * 0.5;
                let p = if a.phase == Phase::Crossing {
                    c.exit(a.side, half)
                } else {
                    Vec2::from_array(c.waits(a.side)[a.slot as usize])
                };
                [p.x, positions[id][1], p.y]
            });
            // Ground settling must not invalidate paths every frame; only XZ/ownership changes do.
            if previous.map(point) != a.waypoint.map(point) {
                navigation.get_mut(id).unwrap().idle();
            }
            if let Some(p) = a.waypoint {
                let remaining_travel = if a.phase == Phase::Crossing {
                    goals
                        .get(id)
                        .and_then(Option::as_ref)
                        .and_then(|g| g.remaining_travel)
                } else {
                    None
                };
                *goals.get_mut(id).unwrap() = Some(NavGoal {
                    position: p,
                    excluded_entity: None,
                    target_half_height: scene.entities[a.entity_index as usize].transform.size[1]
                        * 0.5,
                    stop_distance: 0.12,
                    remaining_travel,
                    replan_on_target_change: false,
                });
            }
        }
    }
}

impl Runtime {
    /// Stop a planned sweep into an unreserved lane, including incidental A* detours.
    /// Bodies already inside may retreat toward their entry; collision pushes remain physical.
    pub fn gate(
        &self,
        scene: &Scene,
        positions: &BTreeMap<String, [f32; 3]>,
        velocities: &mut BTreeMap<String, [f32; 2]>,
        navigation: &mut BTreeMap<String, NavState>,
    ) {
        for (id, a) in &self.agents {
            let e = &scene.entities[a.entity_index as usize];
            let n = e.npc.as_ref().unwrap();
            let Some(v) = velocities.get_mut(id) else {
                continue;
            };
            for (i, c) in scene
                .passages
                .iter()
                .enumerate()
                .filter(|(_, c)| c.surface == n.surface)
            {
                if self.slots[i].owner == Some(a.entity_index) {
                    continue;
                }
                let p = c.local(point(positions[id]));
                let q =
                    c.local(point(positions[id]) + Vec2::from_array(*v) * crate::model::FIXED_DT);
                let half = e.transform.size[0].max(e.transform.size[2]) * 0.5 + 0.06;
                let min = Vec2::new(-half, -c.width * 0.5 - half);
                let max = Vec2::new(c.length() + half, c.width * 0.5 + half);
                let inside = p.cmpge(min).all() && p.cmple(max).all();
                let blocked = if inside {
                    let side = if a.passage == Some(i as u8) {
                        a.side
                    } else if p.x < c.length() * 0.5 {
                        -1
                    } else {
                        1
                    };
                    (q.x - p.x) * f32::from(side) < -0.000001
                } else {
                    let delta = q - p;
                    let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
                    let mut hit = true;
                    for axis in 0..2 {
                        if delta[axis].abs() < 1e-7 {
                            if p[axis] < min[axis] || p[axis] > max[axis] {
                                hit = false;
                                break;
                            }
                        } else {
                            let t1 = (min[axis] - p[axis]) / delta[axis];
                            let t2 = (max[axis] - p[axis]) / delta[axis];
                            enter = enter.max(t1.min(t2));
                            leave = leave.min(t1.max(t2));
                            if enter > leave {
                                hit = false;
                                break;
                            }
                        }
                    }
                    hit
                };
                if blocked {
                    *v = [0., 0.];
                    let nav = navigation.get_mut(id).unwrap();
                    "waiting_passage".clone_into(&mut nav.status);
                    "reservation_required".clone_into(&mut nav.reason);
                    nav.recovery.movement_requested = false;
                    break;
                }
            }
        }
    }
}
