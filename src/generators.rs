//! Bounded deterministic population policy. Birth and retirement use the same runtime transactions.
use crate::{
    identity::Reference,
    model::{Result, Scene, invalid},
    population::Outcome,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

pub const MAX_GENERATORS: usize = 8;
pub const MAX_POINTS: usize = 32;
pub const RECEIPTS: usize = 256;
fn default_active() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub template: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates: Vec<crate::template_selection::Choice>,
    #[serde(default)]
    pub points: Vec<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<crate::spawn_region::Region>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_projection: Option<crate::spawn_support::Projection>,
    pub seed: u32,
    pub target_alive: u8,
    pub interval_ticks: u32,
    pub attempts_per_cycle: u8,
    pub max_births: u32,
    #[serde(default = "default_active")]
    pub initially_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation: Option<crate::generator_activation::Config>,
    #[serde(default)]
    pub initial_delay_ticks: u32,
    #[serde(default)]
    pub retire_dead_after_ticks: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waves: Vec<crate::waves::Config>,
}
pub fn validate(scene: &Scene) -> Result<()> {
    let mut names = BTreeSet::new();
    if scene.npc_generators.len() > MAX_GENERATORS
        || (!scene.npc_generators.is_empty() && scene.version < 60)
    {
        return Err(invalid(
            "Generators require scene60 and at most8 definitions",
        ));
    }
    for g in &scene.npc_generators {
        if g.id.is_empty()
            || g.id.len() > 64
            || !names.insert(&g.id)
            || !g
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            || (g.points.is_empty() && g.region.is_none())
            || (!g.points.is_empty() && g.region.is_some())
            || g.points.len() > MAX_POINTS
            || g.points
                .iter()
                .flatten()
                .any(|p| !p.is_finite() || p.abs() > 10000.)
            || g.target_alive == 0
            || g.target_alive as usize > crate::templates::capacity(scene)
            || !(1..=8).contains(&g.attempts_per_cycle)
            || !(1..=36000).contains(&g.interval_ticks)
            || !(1..=36000).contains(&g.max_births)
            || g.initial_delay_ticks > 36000
            || g.retire_dead_after_ticks.is_some_and(|t| t > 36000)
        {
            return Err(invalid(
                "Invalid generator ID, template, points, population or tick/birth budget",
            ));
        }
        if !g.initially_active && scene.version < 62 {
            return Err(invalid("Inactive generators require scene62"));
        }
        crate::waves::validate(scene, g)?;
        crate::template_selection::validate(
            scene,
            (!g.template.is_empty()).then_some(g.template.as_str()),
            &g.templates,
        )?;
        if let Some(region) = g.region {
            region.validate(scene.version)?;
        }
        if let Some(projection) = g.support_projection {
            if scene.version < 66 {
                return Err(invalid("Support projection requires scene66"));
            }
            projection.validate()?;
        }
        if let Some(activation) = &g.activation {
            activation.validate(scene)?;
        }
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Member {
    pub template: u8,
    pub actor: u16,
    pub instance: Reference,
    pub dead_since: Option<u64>,
    pub observed_life: u64,
    pub wave: Option<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Generator {
    pub template_rng: u32,
    pub template_draws: u32,
    pub pending_template: bool,
    pub activation: Option<crate::generator_activation::State>,
    pub status: crate::generator_control::Status,
    pub paused_since: Option<u64>,
    pub cancelled_tick: Option<u64>,
    pub template: u8,
    pub target: Option<Reference>,
    pub rng: u32,
    pub next_attempt_tick: u64,
    pub births: u32,
    pub attempts: u32,
    #[serde(deserialize_with = "read_members")]
    pub members: Vec<Option<Member>>,
    pub wave: Option<crate::waves::Progress>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub support: Option<crate::spawn_support::Report>,
    pub template: Option<u8>,
    pub tick: u64,
    pub generator: u8,
    pub point: Option<u8>,
    pub position: Option<[f32; 3]>,
    pub wave: Option<u8>,
    pub outcome: Outcome,
    pub instance: Option<Reference>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub generators: Vec<Generator>,
    pub receipts: VecDeque<Receipt>,
    pub dropped_receipts: u32,
    pub template_sources: Vec<crate::template_selection::Sources>,
    pub wave_events: VecDeque<crate::waves::Event>,
    pub dropped_wave_events: u32,
}
impl State {
    pub fn new(scene: &Scene, actors: &crate::targeting::Catalog) -> Self {
        let template_sources: Vec<_> = scene
            .npc_generators
            .iter()
            .map(|g| crate::template_selection::Sources::compile(scene, actors, g))
            .collect();
        Self {
            generators: scene
                .npc_generators
                .iter()
                .enumerate()
                .map(|(i, g)| {
                    let binding = template_sources[i]
                        .current((!g.waves.is_empty()).then_some(0))
                        .first();
                    let wave = crate::waves::Progress::new(g);
                    Generator {
                        template_rng: g.seed ^ crate::template_selection::SEED_SALT,
                        template_draws: 0,
                        pending_template: false,
                        activation: g
                            .activation
                            .as_ref()
                            .map(|c| crate::generator_activation::State::new(c, scene)),
                        status: if g.initially_active {
                            crate::generator_control::Status::Active
                        } else {
                            crate::generator_control::Status::Paused
                        },
                        paused_since: (!g.initially_active || g.activation.is_some()).then_some(0),
                        cancelled_tick: None,
                        template: binding.template,
                        target: binding.target,
                        rng: g.seed,
                        next_attempt_tick: wave
                            .map_or(g.initial_delay_ticks as u64, |w| w.due_tick),
                        births: 0,
                        attempts: 0,
                        members: vec![None; crate::templates::capacity(scene)],
                        wave,
                    }
                })
                .collect(),
            receipts: VecDeque::with_capacity(if scene.npc_generators.is_empty() {
                0
            } else {
                RECEIPTS
            }),
            dropped_receipts: 0,
            template_sources,
            wave_events: VecDeque::with_capacity(
                if scene.npc_generators.iter().any(|g| !g.waves.is_empty()) {
                    crate::waves::EVENTS
                } else {
                    0
                },
            ),
            dropped_wave_events: 0,
        }
    }
    fn wave_event(&mut self, event: crate::waves::Event) {
        if self.wave_events.len() == crate::waves::EVENTS {
            self.wave_events.pop_front();
            self.dropped_wave_events += 1;
        }
        self.wave_events.push_back(event);
    }
    fn record(&mut self, receipt: Receipt) {
        if self.receipts.len() == RECEIPTS {
            self.receipts.pop_front();
            self.dropped_receipts += 1;
        }
        self.receipts.push_back(receipt);
    }
}
impl crate::physics::Simulation {
    pub(crate) fn step_generators(&mut self) {
        // Author order is an explicit capacity priority; no random global ordering or retries until success.
        for i in 0..self.generators.generators.len() {
            let members = std::mem::take(&mut self.generators.generators[i].members);
            let mut state = self.generators.generators[i].clone();
            state.members = members;
            let config = &self.definitions().npc_generators[i];
            let (target_alive, interval, attempts, max_births, retire) = (
                config.target_alive,
                config.interval_ticks,
                config.attempts_per_cycle,
                config.max_births,
                config.retire_dead_after_ticks,
            );
            let mut alive = 0;
            let mut wave_alive = 0;
            let mut wave_present = 0;
            let current_wave = state.wave.map(|w| w.current);
            for member in &mut state.members {
                let Some(m) = member.as_mut() else {
                    continue;
                };
                if !self.actors.matches(m.actor, m.instance) {
                    *member = None;
                    continue;
                }
                let life = self.vitality.state.lives[m.actor as usize];
                if life != m.observed_life {
                    m.observed_life = life;
                    m.dead_since = None;
                }
                if self.vitality.dead(m.actor as usize) {
                    let since = *m.dead_since.get_or_insert(self.tick);
                    if retire.is_some_and(|delay| self.tick - since >= u64::from(delay)) {
                        self.remove_entity(self.actors.entity(m.actor));
                        self.generators.record(Receipt {
                            template: Some(m.template),
                            tick: self.tick,
                            generator: i as u8,
                            point: None,
                            position: None,
                            support: None,
                            wave: m.wave,
                            outcome: Outcome::Despawned,
                            instance: Some(m.instance),
                        });
                        *member = None;
                        continue;
                    }
                } else {
                    m.dead_since = None;
                    alive += 1;
                }
                if current_wave.is_some() && m.wave == current_wave {
                    wave_present += 1;
                    if !self.vitality.dead(m.actor as usize) {
                        wave_alive += 1;
                    }
                }
            }
            if let Some(activation) = &mut state.activation {
                activation.sample(
                    self.triggers
                        .occupied_present(activation.volume, &self.actors),
                    self.tick,
                );
            }
            state.sync_production_clock(self.tick, max_births);
            if !state.production_enabled() {
                self.generators.generators[i] = state;
                continue;
            }
            if let Some(wave) = &mut state.wave {
                let event = wave.advance(
                    &self.definitions().npc_generators[i].waves,
                    self.tick,
                    wave_alive,
                    wave_present,
                );
                if let Some((index, kind)) = event {
                    self.generators.wave_event(crate::waves::Event {
                        tick: self.tick,
                        generator: i as u8,
                        wave: index,
                        kind,
                    });
                    let binding = self.generators.template_sources[i]
                        .current(Some(wave.current))
                        .first();
                    state.pending_template = false;
                    state.template = binding.template;
                    state.target = binding.target;
                    state.next_attempt_tick = wave.due_tick;
                }
            }
            let can_spawn = state
                .wave
                .is_none_or(|w| w.phase == crate::waves::Phase::Spawning);
            if can_spawn
                && self.tick >= state.next_attempt_tick
                && alive < target_alive
                && state.births < max_births
            {
                state.next_attempt_tick = self.tick + u64::from(interval);
                let point_count = self.definitions().npc_generators[i].points.len();
                let region = self.definitions().npc_generators[i].region;
                // Preserve the legacy point sequence; regions draw only for candidates actually attempted.
                let (start, count) = if region.is_some() {
                    (0, usize::from(attempts))
                } else {
                    let rng = crate::spawn_region::draw(&mut state.rng);
                    (
                        ((u64::from(rng) * point_count as u64) >> 32) as usize,
                        usize::from(attempts).min(point_count),
                    )
                };
                for offset in 0..count {
                    if alive >= target_alive
                        || state.births >= max_births
                        || state
                            .wave
                            .is_some_and(|w| w.phase != crate::waves::Phase::Spawning)
                    {
                        break;
                    }
                    let Some(slot) = state.members.iter().position(Option::is_none) else {
                        break;
                    };
                    if !state.pending_template {
                        let (binding, drew) = self.generators.template_sources[i]
                            .current(state.wave.map(|w| w.current))
                            .select(&mut state.template_rng);
                        state.template = binding.template;
                        state.target = binding.target;
                        state.template_draws += u32::from(drew);
                        state.pending_template = true;
                    }
                    let (point, position) = if let Some(region) = region {
                        (None, region.sample(&mut state.rng))
                    } else {
                        let point = (start + offset) % point_count;
                        (
                            Some(point as u8),
                            self.definitions().npc_generators[i].points[point],
                        )
                    };
                    state.attempts += 1;
                    let target_available = state
                        .target
                        .is_none_or(|r| self.actors.label(self.definitions(), r).is_some());
                    let support = if target_available {
                        self.definitions().npc_generators[i]
                            .support_projection
                            .map(|projection| {
                                self.query_spawn_support(state.template, position, projection)
                            })
                    } else {
                        None
                    };
                    let result = if !target_available {
                        Err(Outcome::InstanceUnavailable)
                    } else if let Some(report) = support {
                        match report.accepted() {
                            Some(resolved) => self.spawn(state.template, resolved, state.target),
                            None => Err(report.rejection()),
                        }
                    } else {
                        self.spawn(state.template, position, state.target)
                    };
                    let (outcome, instance) = match result {
                        Ok((actor, instance)) => {
                            state.members[slot] = Some(Member {
                                template: state.template,
                                actor,
                                instance,
                                dead_since: None,
                                observed_life: self.vitality.state.lives[actor as usize],
                                wave: state.wave.map(|w| w.current),
                            });
                            state.births += 1;
                            state.pending_template = false;
                            if let Some(wave) = &mut state.wave {
                                wave.births += 1;
                                if wave.births
                                    == self.definitions().npc_generators[i].waves
                                        [wave.current as usize]
                                        .count
                                {
                                    wave.phase = crate::waves::Phase::WaitingForClear;
                                }
                            }
                            if !self.vitality.dead(actor as usize) {
                                alive += 1;
                            }
                            (Outcome::Spawned, Some(instance))
                        }
                        Err(reason) => (reason, None),
                    };
                    self.generators.record(Receipt {
                        template: Some(state.template),
                        tick: self.tick,
                        generator: i as u8,
                        point,
                        position: Some(position),
                        support,
                        wave: state.wave.map(|w| w.current),
                        outcome,
                        instance,
                    });
                }
            }
            self.generators.generators[i] = state;
        }
    }
}

fn read_members<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Option<Member>>, D::Error> {
    struct Bounded;
    impl<'de> serde::de::Visitor<'de> for Bounded {
        type Value = Vec<Option<Member>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "up to128generator members")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out =
                Vec::with_capacity(a.size_hint().unwrap_or(8).min(crate::templates::MAX_NPCS));
            while let Some(m) = a.next_element()? {
                if out.len() == crate::templates::MAX_NPCS {
                    return Err(serde::de::Error::custom("Generator member budget"));
                }
                out.push(m);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Bounded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;
    #[test]
    fn repeated_death_after_revival_between_samples_gets_new_retirement_deadline() {
        let mut scene: Scene =
            serde_json::from_str(include_str!("../examples/npc-spawn.scene.json")).unwrap();
        scene.version = crate::model::SCENE_VERSION;
        scene.npc_generators = vec![Config {
            id: "camp".into(),
            template: "guard".into(),
            templates: Vec::new(),
            points: vec![[0., 0.8, 4.]],
            region: None,
            support_projection: None,
            seed: 0,
            target_alive: 1,
            interval_ticks: 1,
            attempts_per_cycle: 1,
            max_births: 2,
            initially_active: true,
            activation: None,
            initial_delay_ticks: 0,
            retire_dead_after_ticks: Some(2),
            waves: Vec::new(),
        }];
        let mut e = Engine::new(scene).unwrap();
        e.advance(1).unwrap();
        let m = e.simulation.generators.generators[0].members[0].unwrap();
        // Exercise real vitality commits between population samples, including same-tick revive/death.
        let commit = |e: &mut Engine, units: i64| {
            let s = &mut e.simulation;
            let mut changes = crate::vitality::Changes::default();
            let mut health = s.vitality.state.actors[m.actor as usize].unwrap();
            health.units = units;
            changes.get_or_insert(m.actor as usize, || crate::vitality::ChangeSet {
                health: Some(health),
                cause: Some(0),
            });
            s.vitality
                .commit(&mut changes, s.tick, m.actor, 1, &s.actors, m.instance);
        };
        commit(&mut e, 0);
        e.advance(1).unwrap();
        assert_eq!(
            e.simulation.generators.generators[0].members[0]
                .unwrap()
                .dead_since,
            Some(1)
        );
        commit(&mut e, 40);
        commit(&mut e, 0);
        assert_eq!(e.simulation.vitality.state.lives[m.actor as usize], 1);
        e.advance(1).unwrap();
        assert_eq!(
            e.simulation.generators.generators[0].members[0]
                .unwrap()
                .dead_since,
            Some(2)
        );
        e.advance(1).unwrap();
        assert!(e.simulation.actors.matches(m.actor, m.instance));
        e.advance(1).unwrap();
        assert!(!e.simulation.actors.matches(m.actor, m.instance));
    }
}
