//! Optional deterministic utility policy over already eligible objectives.
//! No world access: personality cannot turn a preference into hidden knowledge.
use crate::{
    model::{Result, invalid},
    objectives::{Action, Objective},
    perception::Observation,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const SCALE: i32 = 10_000;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Personality {
    pub anxiety: f32,
    pub caution: f32,
    pub concern: f32,
    pub fear: f32,
    pub courage: f32,
    pub curiosity: f32,
    pub aggression: f32,
    pub sociability: f32,
    pub discipline: f32,
}
impl Default for Personality {
    fn default() -> Self {
        Self {
            anxiety: 0.5,
            caution: 0.5,
            concern: 0.5,
            fear: 0.5,
            courage: 0.5,
            curiosity: 0.5,
            aggression: 0.5,
            sociability: 0.5,
            discipline: 0.5,
        }
    }
}
impl Personality {
    fn values(self) -> [f32; 9] {
        [
            self.anxiety,
            self.caution,
            self.concern,
            self.fear,
            self.courage,
            self.curiosity,
            self.aggression,
            self.sociability,
            self.discipline,
        ]
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub intelligence: f32,
    pub personality: Personality,
    pub seed: u64,
    pub variation: f32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            intelligence: 0.5,
            personality: Personality::default(),
            seed: 0,
            variation: 0.,
        }
    }
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if self
            .personality
            .values()
            .into_iter()
            .chain([self.intelligence, self.variation])
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(&v))
        {
            return Err(invalid("NPC mind coefficients must be finite in [0,1]"));
        }
        Ok(())
    }
    pub fn resolve(self, instance: u64) -> Profile {
        let mut seed = self.seed ^ instance.wrapping_mul(0x9e3779b97f4a7c15);
        let mut quantize = |v: f32| {
            seed = seed.wrapping_add(0x9e3779b97f4a7c15);
            let mut x = seed;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            x ^= x >> 31;
            let delta = (x % 20_001) as i32 - SCALE;
            ((v * SCALE as f32).round() as i32 + (delta as f32 * self.variation).round() as i32)
                .clamp(0, SCALE) as u16
        };
        Profile {
            intelligence: quantize(self.intelligence),
            traits: self.personality.values().map(quantize),
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub intelligence: u16,
    pub traits: [u16; 9],
}
impl Profile {
    pub fn think_interval(self) -> u64 {
        1 + (10_000 - self.intelligence as u64) * 11 / 10_000
    }
    pub fn search_entries(self) -> usize {
        1 + self.intelligence as usize * 2 / 10_000
    }
    pub fn search_exits(self) -> usize {
        1 + self.intelligence as usize * 3 / 10_000
    }
    pub fn search_steps(self) -> u16 {
        600 + (self.intelligence as u32 * 1800 / 10_000) as u16
    }
    pub fn hysteresis(self) -> i32 {
        (self.traits[8] as i32 + self.traits[1] as i32) / 8 - self.traits[0] as i32 / 8
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Deliberated,
    Thinking,
    Hysteresis,
    NoEligible,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Decision {
    pub instance: u64,
    pub profile: Profile,
    pub scores: [Option<i32>; 8],
    pub chosen: Option<u8>,
    pub next_think_tick: u64,
    pub reason: Reason,
    pub injury: u16,
    pub threat: u16,
}
#[derive(Clone, Copy)]
pub struct Context {
    pub profile: Profile,
    pub instance: u64,
    pub health: Option<crate::vitality::Health>,
}
fn utility(profile: Profile, action: Action, injury: i32, threat: i32, uncertain: bool) -> i32 {
    let [
        anxiety,
        caution,
        concern,
        fear,
        courage,
        curiosity,
        aggression,
        sociability,
        discipline,
    ] = profile.traits.map(i32::from);
    let risk = threat + injury;
    match action {
        Action::Routine => {
            discipline + sociability / 2 + concern * injury / SCALE
                - (anxiety + fear) * threat / SCALE
        }
        Action::Tactical { .. } => {
            (caution + courage + curiosity) * threat / SCALE + fear * injury / SCALE
        }
        Action::Flee { .. } => {
            (fear + caution + anxiety) * risk / SCALE - courage * threat / SCALE - aggression / 2
        }
        Action::Respond => {
            (courage + aggression) * threat / SCALE + if uncertain { curiosity } else { 0 }
                - (fear + caution) * injury / SCALE
        }
        Action::UseAbility { .. } => {
            (courage + aggression) * threat / SCALE + concern * injury / SCALE
                - fear * injury / SCALE
        }
        Action::Patrol => discipline + curiosity / 2 - (anxiety + fear) * threat / SCALE,
        Action::Guard { .. } | Action::Guard3d { .. } => {
            discipline + caution / 2 + concern / 2 - fear * injury / SCALE
        }
        Action::Wait => {
            caution + discipline / 2 + sociability / 4 + anxiety * injury / SCALE - curiosity / 2
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub fn choose(
    state: &mut Option<Decision>,
    context: Context,
    objectives: &[Objective],
    eligible: u8,
    previous: Option<u8>,
    memory: Option<Observation>,
    visible: bool,
    tick: u64,
) -> Option<usize> {
    let d = state.get_or_insert_with(|| Decision {
        instance: context.instance,
        profile: context.profile,
        scores: [None; 8],
        chosen: None,
        next_think_tick: 0,
        reason: Reason::NoEligible,
        injury: 0,
        threat: 0,
    });
    if d.instance != context.instance {
        *state = None;
        return choose(
            state, context, objectives, eligible, previous, memory, visible, tick,
        );
    }
    let injury = context.health.map_or(0, |h| {
        ((h.maximum - h.units).max(0) as i128 * 10_000 / h.maximum.max(1) as i128) as i32
    });
    let threat = if visible {
        10_000
    } else if memory.is_some() {
        4_000
    } else {
        0
    };
    d.injury = injury as u16;
    d.threat = threat as u16;
    let mut best: Option<usize> = None;
    for (i, o) in objectives.iter().enumerate() {
        d.scores[i] = if eligible & (1 << i) != 0 {
            Some(utility(
                d.profile,
                o.action,
                injury,
                threat,
                memory.is_some_and(|m| !m.source.is_visual()),
            ))
        } else {
            None
        };
        if let Some(score) = d.scores[i]
            && best.is_none_or(|j| {
                o.priority > objectives[j].priority
                    || (o.priority == objectives[j].priority && score > d.scores[j].unwrap())
            })
        {
            best = Some(i);
        }
    }
    d.reason = Reason::Deliberated;
    if let Some(old) = previous
        .map(usize::from)
        .filter(|&i| i < objectives.len() && eligible & (1 << i) != 0)
        && let Some(new) = best
        && objectives[old].priority == objectives[new].priority
    {
        if tick < d.next_think_tick {
            best = Some(old);
            d.reason = Reason::Thinking;
        } else if new != old
            && d.scores[new].unwrap() - d.scores[old].unwrap() <= d.profile.hysteresis().max(0)
        {
            best = Some(old);
            d.reason = Reason::Hysteresis;
        }
    }
    if d.reason != Reason::Thinking {
        d.next_think_tick = tick.saturating_add(d.profile.think_interval());
    }
    d.chosen = best.map(|i| i as u8);
    if best.is_none() {
        d.reason = Reason::NoEligible;
    }
    best
}
