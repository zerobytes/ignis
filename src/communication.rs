//! One-hop radio reports made from confirmed vision, never from hidden actor positions.
use crate::{
    model::{Awareness, Result, Scene, invalid},
    perception::{NpcState, Observation, Source},
    targeting::{Attitude, Catalog},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Config {
    Radio {
        channel: u16,
        range: f32,
        cooldown_ticks: u32,
        max_age_ticks: u32,
        send: bool,
        receive: bool,
    },
}
impl Config {
    pub fn validate(self) -> Result<()> {
        let Self::Radio {
            channel,
            range,
            cooldown_ticks,
            max_age_ticks,
            ..
        } = self;
        if channel == 0
            || !range.is_finite()
            || !(0.1..=100.).contains(&range)
            || !(1..=3600).contains(&cooldown_ticks)
            || !(1..=3600).contains(&max_age_ticks)
        {
            return Err(invalid(
                "Radio requires channel [1,65535], range [0.1,100]m and cooldown/max_age [1,3600]ticks",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub sender: u16,
    pub sender_instance: crate::identity::Reference,
    pub observation: Observation,
    pub received_tick: u64,
    pub expires_at: u64,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    #[default]
    Disabled,
    NoReport,
    DirectEvidence,
    ExistingEvidence,
    Filtered,
    Accepted,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub reason: Reason,
    pub received_this_tick: bool,
    pub sent_this_tick: bool,
    pub next_send_tick: u64,
    pub sent: u64,
    pub accepted: u64,
    pub rejected: u8,
    pub last_received: Option<Receipt>,
}
impl State {
    pub fn memory_expired(&self, memory: Observation, tick: u64) -> bool {
        memory.source == Source::Report && self.last_received.is_none_or(|r| tick >= r.expires_at)
    }
}

#[derive(Clone, Copy)]
pub struct Report {
    sender: u16,
    sender_instance: crate::identity::Reference,
    channel: u16,
    range: f32,
    position: [f32; 3],
    observation: Observation,
    expires_at: u64,
}
pub type Reports = Vec<Report>;

/// Freeze outgoing reports before receiver updates; logical instance IDs resolve reception ties.
pub fn collect(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    actors: &Catalog,
    states: &mut BTreeMap<String, NpcState>,
) -> Reports {
    let mut reports = Vec::with_capacity(states.len());
    collect_into(scene, positions, tick, actors, states, &mut reports);
    reports
}
pub(crate) fn collect_into(
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    actors: &Catalog,
    states: &mut BTreeMap<String, NpcState>,
    reports: &mut Reports,
) {
    reports.clear();
    for (slot, &index) in actors.actors.iter().enumerate() {
        let entity = &scene.entities[index as usize];
        let Some(npc) = &entity.npc else { continue };
        let Some(state) = states.get_mut(&entity.id) else {
            continue;
        };
        state.communication.sent_this_tick = false;
        state.communication.received_this_tick = false;
        state.communication.rejected = 0;
        state.communication.reason = Reason::Disabled;
        let Some(Config::Radio {
            channel,
            range,
            cooldown_ticks,
            max_age_ticks,
            send: true,
            ..
        }) = npc.communication
        else {
            continue;
        };
        if !npc.enabled
            || state.phase == crate::perception::Phase::Dead
            || state.territory.blocked()
            || npc
                .territory
                .is_some_and(|t| !t.contains(positions[&entity.id]))
            || tick < state.communication.next_send_tick
            || !state.observed_this_tick
        {
            continue;
        }
        let Some(observation) = state.memory.filter(|m| {
            m.source == Source::Vision
                && m.current_actor(actors).is_some()
                && m.tick.checked_add(1) == Some(tick)
        }) else {
            continue;
        };
        let expires_at = observation.tick + u64::from(max_age_ticks);
        if tick >= expires_at {
            continue;
        }
        reports.push(Report {
            sender: slot as u16,
            sender_instance: actors.reference(slot as u16),
            channel,
            range,
            position: positions[&entity.id],
            observation,
            expires_at,
        });
        state.communication.sent_this_tick = true;
        state.communication.sent = state.communication.sent.saturating_add(1);
        state.communication.next_send_tick = tick + u64::from(cooldown_ticks);
    }
}

pub struct Listener<'a> {
    pub scene: &'a Scene,
    pub actors: &'a Catalog,
    pub npc_index: usize,
    pub position: [f32; 3],
    pub tick: u64,
    pub active_memory: Option<Observation>,
    pub known_dead: crate::actor_set::ActorSet,
    pub known_instances: &'a [Option<crate::identity::Reference>],
    pub known_lives: &'a [u64],
}

pub fn receive(
    reports: &Reports,
    listener: Listener<'_>,
    state: &mut State,
) -> Option<Observation> {
    let Listener {
        scene,
        actors,
        npc_index,
        position,
        tick,
        active_memory,
        known_dead,
        known_lives,
        known_instances,
    } = listener;
    let npc = scene.entities[npc_index].npc.as_ref().unwrap();
    let Some(Config::Radio {
        channel,
        range,
        max_age_ticks,
        receive: true,
        ..
    }) = npc.communication
    else {
        return None;
    };
    state.reason = Reason::NoReport;
    if active_memory.is_some_and(|m| m.source != Source::Report) {
        state.reason = Reason::ExistingEvidence;
        return None;
    }
    let receiver = actors.slot(npc_index);
    let mut best: Option<Receipt> = None;
    for report in reports.iter() {
        if report.sender == receiver {
            continue;
        }
        let Some(target) = report.observation.current_actor(actors) else {
            state.rejected += 1;
            continue;
        };
        if !actors.matches(report.sender, report.sender_instance) {
            state.rejected += 1;
            continue;
        }
        let same_known = known_instances[target as usize] == report.observation.instance;
        let expires_at = report
            .expires_at
            .min(report.observation.tick + u64::from(max_age_ticks));
        let memory_limit = match npc.awareness.as_ref() {
            Some(Awareness::Perceptive { memory_ticks, .. }) => *memory_ticks,
            _ => None,
        };
        let eligible_target = npc
            .target
            .as_ref()
            .is_none_or(|id| id == actors.id(scene, target))
            && (npc.targeting.is_none() || actors.attitude(receiver, target) == Attitude::Hostile);
        let radius = range.min(report.range);
        if same_known
            && report
                .observation
                .life
                .is_some_and(|life| life < known_lives[target as usize])
            || same_known && known_dead.contains((target) as usize)
            || report.channel != channel
            || actors.attitude(receiver, report.sender) != Attitude::Allied
            || actors.attitude(report.sender, receiver) != Attitude::Allied
            || glam::Vec3::from(position).distance_squared(report.position.into()) > radius * radius
            || !eligible_target
            || target == receiver
            || tick >= expires_at
            || memory_limit
                .is_some_and(|m| tick.saturating_sub(report.observation.tick) >= u64::from(m))
            || npc
                .territory
                .is_some_and(|t| !t.contains(report.observation.position))
            || active_memory.is_some_and(|m| m.tick >= report.observation.tick)
        {
            state.rejected += 1;
            continue;
        }
        if best.is_none_or(|r| {
            report.observation.tick > r.observation.tick
                || (report.observation.tick == r.observation.tick
                    && report.sender_instance.instance < r.sender_instance.instance)
        }) {
            best = Some(Receipt {
                sender: report.sender,
                sender_instance: report.sender_instance,
                observation: Observation {
                    source: Source::Report,
                    ..report.observation
                },
                received_tick: tick,
                expires_at,
            });
        }
    }
    if let Some(receipt) = best {
        state.received_this_tick = true;
        state.accepted = state.accepted.saturating_add(1);
        state.reason = Reason::Accepted;
        state.last_received = Some(receipt);
        Some(receipt.observation)
    } else {
        if state.rejected > 0 {
            state.reason = Reason::Filtered;
        }
        None
    }
}
