//! Resource-driven needs and tick-based schedules over explicitly known points.
use crate::{
    model::{Entity, Result, Scene, invalid},
    npc_mind::Profile,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
fn cycle() -> u32 {
    3600
}
fn interval() -> u32 {
    60
}
fn below() -> u16 {
    2500
}
fn satisfied() -> u16 {
    9000
}
fn depletion() -> i64 {
    -1
}
fn priority() -> u8 {
    100
}
fn trip() -> u32 {
    1800
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "trip")]
    pub max_trip_ticks: u32,
    #[serde(default = "cycle")]
    pub cycle_ticks: u32,
    #[serde(default)]
    pub offset_ticks: u32,
    #[serde(default)]
    pub phase_variation: u32,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub auto_dialogue: bool,
    #[serde(default)]
    pub needs: Vec<Need>,
    #[serde(default)]
    pub schedule: Vec<Schedule>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Need {
    pub resource: String,
    pub tag: String,
    pub action: String,
    #[serde(default = "below")]
    pub below: u16,
    #[serde(default = "satisfied")]
    pub satisfied: u16,
    #[serde(default = "priority")]
    pub priority: u8,
    #[serde(default = "interval")]
    pub interval_ticks: u32,
    #[serde(default = "depletion")]
    pub delta: i64,
    #[serde(default)]
    pub personality_weights: [i16; 9],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    pub start: u32,
    pub end: u32,
    pub point: String,
    pub action: String,
    #[serde(default)]
    pub priority: u8,
}
impl Config {
    pub fn validate(&self, scene: &Scene, e: &Entity) -> Result<()> {
        if !(60..=36000).contains(&self.max_trip_ticks)
            || !(60..=86_400_000).contains(&self.cycle_ticks)
            || self.offset_ticks >= self.cycle_ticks
            || self.phase_variation >= self.cycle_ticks
            || self.needs.len() > 8
            || self.schedule.len() > 8
        {
            return Err(invalid(
                "Routine cycle [60,86400000], offsets<cycle and up to8needs/schedule entries",
            ));
        }
        let known = |p: &crate::interactions::Point| {
            scene
                .entities
                .iter()
                .find(|e| e.id == p.entity)
                .is_some_and(|e| {
                    !matches!(
                        e.body,
                        Some(
                            crate::model::BodyKind::Npc
                                | crate::model::BodyKind::Player
                                | crate::model::BodyKind::Dynamic
                        )
                    )
                })
        };
        for n in &self.needs {
            if n.below >= n.satisfied
                || n.satisfied > 10000
                || !(1..=36000).contains(&n.interval_ticks)
                || !(-crate::abilities::LIMIT..=crate::abilities::LIMIT).contains(&n.delta)
                || n.personality_weights
                    .iter()
                    .any(|v| v.unsigned_abs() > 1000)
                || !scene
                    .ability_catalog
                    .resources
                    .iter()
                    .any(|r| r.id == n.resource)
                || !e.abilities.as_ref().is_some_and(|l| {
                    l.resources
                        .iter()
                        .any(|p| p.resource == n.resource && p.capacity > 0)
                })
                || !scene
                    .interactions
                    .points
                    .iter()
                    .any(|p| known(p) && p.tags.contains(&n.tag) && p.actions.contains(&n.action))
            {
                return Err(invalid(
                    "Need requires registered own resource, thresholds, bounded rate/weights and known static station/action",
                ));
            }
        }
        for s in &self.schedule {
            if s.start >= s.end
                || s.end > self.cycle_ticks
                || !scene
                    .interactions
                    .points
                    .iter()
                    .any(|p| p.id == s.point && known(p) && p.actions.contains(&s.action))
            {
                return Err(invalid(
                    "Schedule requires ordered window and known static point/action",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    Need { index: u8 },
    Schedule { index: u8, cycle: u64 },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Selection {
    pub source: Source,
    pub point: u8,
    pub action: u8,
    pub position: [f32; 3],
    pub contact: [f32; 3],
    pub score: i32,
    pub started: u64,
    pub awaiting: Option<u64>,
    pub reached: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Selected,
    Thinking,
    Suspended,
    NoDemand,
    NoStation,
    Approaching,
    Interacting,
    Dialogue,
    Completed,
    Rejected,
    ActionBudget,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub schedule_mask: u8,
    pub active_needs: u8,
    pub current: Option<Selection>,
    pub next_needs: [u64; 8],
    pub completed_cycles: [Option<u64>; 8],
    pub failures: [Option<(u8, u64)>; 8],
    pub search_cursor: [u8; 8],
    pub phase_offset: u32,
    pub next_think_tick: u64,
    pub next_action_tick: u64,
    pub next_dialogue_tick: u64,
    pub dialogue_cursor: u8,
    pub dialogue_scores: [Option<i32>; 8],
    pub reason: Reason,
    pub samples: u16,
    pub limited: bool,
    pub resource_updates: u64,
}
impl State {
    fn new(config: &Config, instance: u64, tick: u64) -> Self {
        let x = (instance ^ config.seed).wrapping_mul(0x9e3779b97f4a7c15);
        let phase_offset = ((u64::from(config.offset_ticks)
            + x % u64::from(config.phase_variation + 1))
            % u64::from(config.cycle_ticks)) as u32;
        Self {
            schedule_mask: 0,
            active_needs: 0,
            current: None,
            next_needs: std::array::from_fn(|i| {
                config.needs.get(i).map_or(u64::MAX, |n| {
                    tick.saturating_add(u64::from(n.interval_ticks))
                })
            }),
            completed_cycles: [None; 8],
            failures: [None; 8],
            search_cursor: [0; 8],
            phase_offset,
            next_think_tick: tick,
            next_action_tick: tick,
            next_dialogue_tick: tick,
            dialogue_cursor: 0,
            dialogue_scores: [None; 8],
            reason: Reason::NoDemand,
            samples: 0,
            limited: false,
            resource_updates: 0,
        }
    }
    pub fn suspend_life(&mut self) {
        self.current = None;
        self.failures = [None; 8];
        self.next_needs = [0; 8];
        self.next_think_tick = 0;
        self.next_action_tick = 0;
        self.next_dialogue_tick = 0;
        self.reason = Reason::Suspended;
    }
}
struct CompiledNeed {
    resource: u8,
    action: u8,
    points: Vec<u8>,
}
struct CompiledSchedule {
    point: u8,
    action: u8,
}
pub(crate) struct Compiled {
    configuration: Config,
    needs: Vec<CompiledNeed>,
    schedule: Vec<CompiledSchedule>,
}
#[derive(Default)]
pub struct Runtime {
    profiles: std::collections::BTreeMap<String, Compiled>,
}
impl Runtime {
    pub(crate) fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
    pub fn new(scene: &Scene, interactions: &crate::interactions::Runtime) -> Self {
        let mut r = Self::default();
        for e in &scene.entities {
            if !crate::templates::vacant(e) {
                r.install(scene, e, interactions);
            }
        }
        r
    }
    pub(crate) fn install(
        &mut self,
        scene: &Scene,
        e: &Entity,
        interactions: &crate::interactions::Runtime,
    ) {
        let Some(config) = e.npc.as_ref().and_then(|n| n.routine.as_ref()) else {
            return;
        };
        let action = |id: &str| {
            interactions
                .document
                .actions
                .iter()
                .position(|a| a.id == id)
                .unwrap() as u8
        };
        let needs = config
            .needs
            .iter()
            .map(|n| {
                let tag = interactions.tag(&n.tag).unwrap();
                let action = action(&n.action);
                CompiledNeed {
                    resource: scene
                        .ability_catalog
                        .resources
                        .iter()
                        .position(|r| r.id == n.resource)
                        .unwrap() as u8,
                    action,
                    points: interactions
                        .points
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            p.stationary
                                && p.tags & (1u64 << tag) != 0
                                && p.actions & (1u32 << action) != 0
                        })
                        .map(|(i, _)| i as u8)
                        .collect(),
                }
            })
            .collect();
        let schedule = config
            .schedule
            .iter()
            .map(|s| CompiledSchedule {
                point: interactions
                    .document
                    .points
                    .iter()
                    .position(|p| p.id == s.point)
                    .unwrap() as u8,
                action: action(&s.action),
            })
            .collect();
        self.profiles.insert(
            e.id.clone(),
            Compiled {
                configuration: config.clone(),
                needs,
                schedule,
            },
        );
    }
    pub(crate) fn remove(&mut self, id: &str) {
        self.profiles.remove(id);
    }
    pub(crate) fn drift(
        &self,
        id: &str,
        instance: u64,
        actor: usize,
        npc: &mut crate::perception::NpcState,
        abilities: &mut crate::abilities::Runtime,
        tick: u64,
    ) {
        let Some(c) = self.profiles.get(id) else {
            return;
        };
        let state = npc
            .routine
            .get_or_insert_with(|| State::new(&c.configuration, instance, tick));
        for (i, n) in c.needs.iter().enumerate() {
            if state.next_needs[i] == 0 {
                state.next_needs[i] =
                    tick.saturating_add(u64::from(c.configuration.needs[i].interval_ticks));
            }
            if tick < state.next_needs[i] {
                continue;
            }
            let (before, capacity) = abilities
                .resource_level(actor, n.resource as usize)
                .expect("validated own need resource");
            abilities.state.balances[actor][n.resource as usize] = before
                .saturating_add(c.configuration.needs[i].delta)
                .clamp(0, capacity);
            state.next_needs[i] =
                tick.saturating_add(u64::from(c.configuration.needs[i].interval_ticks));
            state.resource_updates = state.resource_updates.saturating_add(1);
        }
    }
}

pub(crate) struct Context<'a> {
    pub scene: &'a Scene,
    pub interactions: &'a crate::interactions::Runtime,
    pub abilities: &'a crate::abilities::Runtime,
    pub entity: &'a Entity,
    pub actor: usize,
    pub position: [f32; 3],
    pub profile: Option<Profile>,
    pub instance: u64,
    pub navigation: &'a crate::navigation::NavState,
    pub selected: bool,
    pub tick: u64,
}
#[derive(Default)]
pub(crate) struct Update {
    pub goal: Option<crate::navigation::NavGoal>,
    pub use_point: Option<(u8, u8)>,
    pub close_dialogue: Option<u64>,
}
fn ratio(abilities: &crate::abilities::Runtime, actor: usize, resource: u8) -> u16 {
    let (value, capacity) = abilities
        .resource_level(actor, resource as usize)
        .expect("compiled need resource");
    (value as u64 * 10000 / capacity.max(1) as u64) as u16
}
fn fail(state: &mut State, point: u8, tick: u64, profile: Profile) {
    let until = tick.saturating_add(60 + (10000 - u64::from(profile.intelligence)) * 180 / 10000);
    let at = state
        .failures
        .iter()
        .position(|v| v.is_none_or(|(_, end)| end <= tick))
        .unwrap_or_else(|| {
            state
                .failures
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| v.unwrap().1)
                .unwrap()
                .0
        });
    state.failures[at] = Some((point, until));
    state.current = None;
    state.next_think_tick = tick;
    state.reason = Reason::Rejected;
}
impl Runtime {
    pub(crate) fn update(&self, context: Context<'_>, state: &mut State) -> Update {
        let c = &self.profiles[&context.entity.id];
        let Context {
            scene,
            interactions,
            abilities,
            entity,
            actor,
            position: own,
            profile,
            instance,
            navigation,
            selected,
            tick,
        } = context;
        let profile = profile.unwrap_or(Profile {
            intelligence: 5000,
            traits: [5000; 9],
        });
        let session = interactions.state.sessions.get(actor).copied().flatten();
        if !selected {
            state.reason = Reason::Suspended;
            return Update {
                close_dialogue: state.current.and_then(|s| s.awaiting).filter(|&serial| {
                    session.is_some_and(|s| {
                        s.serial == serial && s.phase == crate::interactions::SessionPhase::Active
                    })
                }),
                ..Default::default()
            };
        }
        let old_needs = state.active_needs;
        for (i, n) in c.needs.iter().enumerate() {
            let value = ratio(abilities, actor, n.resource);
            if value < c.configuration.needs[i].below {
                state.active_needs |= 1 << i;
            }
            if value >= c.configuration.needs[i].satisfied {
                state.active_needs &= !(1 << i);
            }
        }
        let cycle = u64::from(c.configuration.cycle_ticks);
        let shifted = tick % cycle + u64::from(state.phase_offset);
        let day = tick / cycle + shifted / cycle;
        let phase = (shifted % cycle) as u32;
        let schedule_mask = c
            .configuration
            .schedule
            .iter()
            .enumerate()
            .fold(0, |mask, (i, s)| {
                if s.start <= phase && phase < s.end {
                    mask | 1 << i
                } else {
                    mask
                }
            });
        if state.active_needs != old_needs || schedule_mask != state.schedule_mask {
            state.next_think_tick = tick;
        }
        state.schedule_mask = schedule_mask;
        if let Some(current) = state.current {
            let valid = match current.source {
                Source::Need { index } => state.active_needs & (1 << index) != 0,
                Source::Schedule { index, cycle } => {
                    cycle == day
                        && c.configuration.schedule[index as usize].start <= phase
                        && phase < c.configuration.schedule[index as usize].end
                }
            };
            if !valid {
                state.current = None;
                state.next_think_tick = tick;
            } else if let Some(serial) = current.awaiting {
                match session.filter(|s| s.serial == serial).map(|s| s.phase) {
                    Some(crate::interactions::SessionPhase::Active) => {
                        state.reason = Reason::Dialogue;
                        return Update::default();
                    }
                    Some(crate::interactions::SessionPhase::Closed) => {
                        if let Source::Schedule { index, cycle } = current.source {
                            state.completed_cycles[index as usize] = Some(cycle);
                        }
                        state.current.as_mut().unwrap().awaiting = None;
                    }
                    _ => fail(state, current.point, tick, profile),
                }
            } else if (!current.reached
                && tick.saturating_sub(current.started) > u64::from(c.configuration.max_trip_ticks))
                || (navigation.status == "unreachable"
                    && navigation.decision_tick > current.started
                    && navigation.target_position.is_some_and(|p| {
                        glam::Vec3::from(p).distance(current.position.into()) < 0.3
                    }))
            {
                fail(state, current.point, tick, profile);
            }
        }
        if tick >= state.next_think_tick {
            let old = state.current;
            state.samples = 0;
            state.limited = false;
            let mut best: Option<(u8, Selection)> = None;
            let mut consider = |priority: u8,
                                source: Source,
                                point: u8,
                                action: u8,
                                base: i32,
                                state: &mut State| {
                state.samples += 1;
                if state
                    .failures
                    .iter()
                    .flatten()
                    .any(|&(p, end)| p == point && tick < end)
                {
                    return;
                }
                let p = &interactions.points[point as usize];
                let Some(_reference) = interactions.state.anchors[point as usize] else {
                    return;
                };
                if !p.stationary {
                    return;
                }
                let contact =
                    glam::Vec3::from(scene.entities[p.entity as usize].transform.position)
                        + glam::Vec3::from(p.offset);
                let flying = entity
                    .npc
                    .as_ref()
                    .unwrap()
                    .mobility
                    .is_some_and(|m| m.fly_speed.is_some());
                let destination = interactions.document.points[point as usize]
                    .approach
                    .unwrap_or_else(|| {
                        if flying {
                            contact.to_array()
                        } else {
                            contact.with_y(own[1]).to_array()
                        }
                    });
                let distance = glam::Vec3::from(own).distance(destination.into()).min(256.);
                let cost = (distance * 100.) as i32
                    * (2000 + i32::from(profile.traits[1]) + i32::from(profile.traits[3])
                        - i32::from(profile.traits[4]) / 2)
                        .max(300)
                    / 10000;
                let score = base - cost;
                let existing =
                    old.filter(|s| s.source == source && s.point == point && s.action == action);
                let selection = Selection {
                    source,
                    point,
                    action,
                    position: destination,
                    contact: contact.into(),
                    score,
                    started: existing.map_or(tick, |s| s.started),
                    awaiting: existing.and_then(|s| s.awaiting),
                    reached: existing.is_some_and(|s| s.reached),
                };
                if best.is_none_or(|(p, s)| priority > p || (priority == p && score > s.score)) {
                    best = Some((priority, selection));
                }
            };
            for (i, n) in c.needs.iter().enumerate() {
                if state.active_needs & (1 << i) == 0 {
                    continue;
                }
                let definition = &c.configuration.needs[i];
                let value = i32::from(ratio(abilities, actor, n.resource));
                let urgency = 10000 - value;
                let bias = definition
                    .personality_weights
                    .iter()
                    .zip(profile.traits)
                    .map(|(&w, v)| i32::from(w) * i32::from(v) / 1000)
                    .sum::<i32>();
                let base = urgency
                    + (i32::from(profile.traits[0]) + i32::from(profile.traits[3])) * urgency
                        / 10000
                    + i32::from(profile.traits[2]) / 4
                    + bias;
                let budget = 4 + usize::from(profile.intelligence) * 60 / 10000;
                state.limited |= n.points.len() > budget;
                let start = (instance
                    .wrapping_add(c.configuration.seed)
                    .wrapping_add(u64::from(state.search_cursor[i])))
                    as usize;
                for offset in 0..budget.min(n.points.len()) {
                    let point = n.points[(start + offset) % n.points.len()];
                    consider(
                        definition.priority,
                        Source::Need { index: i as u8 },
                        point,
                        n.action,
                        base,
                        state,
                    );
                }
                state.search_cursor[i] =
                    state.search_cursor[i].wrapping_add(budget.min(n.points.len()) as u8);
            }
            for (i, s) in c.schedule.iter().enumerate() {
                let definition = &c.configuration.schedule[i];
                if definition.start <= phase && phase < definition.end {
                    let base = i32::from(profile.traits[8])
                        + i32::from(profile.traits[7]) / 2
                        + i32::from(profile.traits[5]) / 4
                        - i32::from(profile.traits[0]) / 4;
                    consider(
                        definition.priority,
                        Source::Schedule {
                            index: i as u8,
                            cycle: day,
                        },
                        s.point,
                        s.action,
                        base,
                        state,
                    );
                }
            }
            state.current = best.map(|(_, s)| s);
            state.next_think_tick =
                tick.saturating_add(if best.is_some() || state.active_needs == 0 {
                    profile.think_interval()
                } else {
                    60
                });
            state.reason = if best.is_some() {
                Reason::Selected
            } else if state.active_needs != 0 {
                Reason::NoStation
            } else {
                Reason::NoDemand
            };
        } else if state.current.is_some() {
            state.reason = Reason::Thinking;
        }
        let Some(current) = state.current else {
            return Update::default();
        };
        let action = &interactions.document.actions[current.action as usize];
        let point = &interactions.document.points[current.point as usize];
        let flying = entity
            .npc
            .as_ref()
            .unwrap()
            .mobility
            .is_some_and(|m| m.fly_speed.is_some());
        let margin = (action.range - 0.08).max(0.01);
        let dy = own[1] - current.contact[1];
        let stop = if point.approach.is_some() {
            0.1
        } else if flying {
            margin
        } else {
            (margin * margin - dy * dy).max(0.0001).sqrt()
        };
        let complete = matches!(current.source,Source::Schedule {index,cycle} if state.completed_cycles[index as usize]==Some(cycle));
        let close = glam::Vec3::from(own).distance(current.contact.into()) <= action.range;
        let ready = tick >= state.next_action_tick
            && tick >= interactions.state.cooldowns[actor][current.action as usize];
        if complete {
            state.reason = Reason::Completed;
        } else if close && ready {
            state.reason = Reason::Interacting;
        } else {
            state.reason = Reason::Approaching;
        }
        Update {
            goal: Some(crate::navigation::NavGoal {
                position: current.position,
                excluded_entity: Some(interactions.points[current.point as usize].entity),
                target_half_height: entity.transform.size[1] * 0.5,
                stop_distance: stop,
                remaining_travel: None,
                replan_on_target_change: true,
            }),
            use_point: (!complete && close && ready).then_some((current.point, current.action)),
            close_dialogue: None,
        }
    }
    pub(crate) fn feedback(
        &self,
        state: &mut State,
        session: Option<crate::interactions::Session>,
        outcome: crate::interactions::Outcome,
        tick: u64,
        profile: Option<Profile>,
    ) {
        let profile = profile.unwrap_or(Profile {
            intelligence: 5000,
            traits: [5000; 9],
        });
        state.next_action_tick = tick.saturating_add(profile.think_interval());
        let Some(mut current) = state.current else {
            return;
        };
        match outcome {
            crate::interactions::Outcome::Applied | crate::interactions::Outcome::Closed => {
                current.reached = true;
                current.awaiting = None;
                if let Source::Schedule { index, cycle } = current.source {
                    state.completed_cycles[index as usize] = Some(cycle);
                }
                state.current = Some(current);
                state.reason = Reason::Completed;
            }
            crate::interactions::Outcome::Opened | crate::interactions::Outcome::Advanced => {
                current.reached = true;
                current.awaiting = session.map(|s| s.serial);
                state.current = Some(current);
                state.reason = Reason::Dialogue;
            }
            crate::interactions::Outcome::Cooldown | crate::interactions::Outcome::Busy => {
                state.reason = Reason::Rejected;
            }
            _ => fail(state, current.point, tick, profile),
        }
    }
}
