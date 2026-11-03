//! Finite, observed wave progression; no knowledge is supplied to individual NPCs.
use crate::{
    identity::Reference,
    model::{Result, Scene, invalid},
    targeting::Catalog,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_WAVES: usize = 16;
pub const EVENTS: usize = 256;
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Completion {
    #[default]
    DefeatedOrRemoved,
    Removed,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub count: u32,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates: Vec<crate::template_selection::Choice>,
    #[serde(default)]
    pub delay_ticks: u32,
    #[serde(default)]
    pub completion: Completion,
}
pub fn validate(scene: &Scene, generator: &crate::generators::Config) -> Result<()> {
    if generator.waves.is_empty() {
        return Ok(());
    }
    if scene.version < 61 || generator.waves.len() > MAX_WAVES {
        return Err(invalid(
            "Waves require scene61 and at most16 entries per generator",
        ));
    }
    let mut total = 0u32;
    for wave in &generator.waves {
        if !(1..=36000).contains(&wave.count)
            || wave.delay_ticks > 36000
            || wave
                .template
                .as_ref()
                .is_some_and(|id| !scene.npc_templates.iter().any(|e| &e.id == id))
        {
            return Err(invalid(
                "Wave requires count1..36000, delay0..36000 and a known template",
            ));
        }
        if wave.template.is_some() || !wave.templates.is_empty() {
            crate::template_selection::validate(scene, wave.template.as_deref(), &wave.templates)?;
        }
        total += wave.count;
    }
    if total > generator.max_births {
        return Err(invalid("Wave counts exceed generator max_births"));
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Binding {
    pub template: u8,
    pub target: Option<Reference>,
}
impl Binding {
    pub fn compile(scene: &Scene, actors: &Catalog, id: &str) -> Self {
        let template = scene.npc_templates.iter().position(|t| t.id == id).unwrap();
        let target = scene.npc_templates[template]
            .npc
            .as_ref()
            .unwrap()
            .target
            .as_ref()
            .and_then(|id| scene.entities.iter().position(|e| &e.id == id))
            .and_then(|i| actors.entity_reference(i));
        Self {
            template: template as u8,
            target,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Delay,
    Spawning,
    WaitingForClear,
    Completed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress {
    pub current: u8,
    pub births: u32,
    pub phase: Phase,
    pub due_tick: u64,
    pub completed_tick: Option<u64>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Started,
    Cleared,
    Completed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub tick: u64,
    pub generator: u8,
    pub wave: u8,
    pub kind: EventKind,
}
impl Progress {
    pub fn new(config: &crate::generators::Config) -> Option<Self> {
        config.waves.first().map(|w| Self {
            current: 0,
            births: 0,
            phase: Phase::Delay,
            due_tick: u64::from(config.initial_delay_ticks) + u64::from(w.delay_ticks),
            completed_tick: None,
        })
    }
    /// At most one transition event per tick. Completion is latched even if a corpse later revives.
    pub fn advance(
        &mut self,
        configs: &[Config],
        tick: u64,
        alive: usize,
        present: usize,
    ) -> Option<(u8, EventKind)> {
        let current = self.current;
        match self.phase {
            Phase::Delay if tick >= self.due_tick => {
                self.phase = Phase::Spawning;
                Some((current, EventKind::Started))
            }
            Phase::WaitingForClear
                if match configs[current as usize].completion {
                    Completion::DefeatedOrRemoved => alive == 0,
                    Completion::Removed => present == 0,
                } =>
            {
                if current as usize + 1 == configs.len() {
                    self.phase = Phase::Completed;
                    self.completed_tick = Some(tick);
                    Some((current, EventKind::Completed))
                } else {
                    self.current += 1;
                    self.births = 0;
                    self.phase = Phase::Delay;
                    self.due_tick = tick + u64::from(configs[self.current as usize].delay_ticks);
                    Some((current, EventKind::Cleared))
                }
            }
            _ => None,
        }
    }
}
