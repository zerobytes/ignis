//! Bounded, isolated verification scripts. Conditions read typed runtime state.
use crate::{
    engine::{Command, Engine},
    model::{Result, Scene, invalid},
    noise::{Noise, NoiseOrigin},
    patrol, perception,
    physics::Input,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MAX_ACTIONS: usize = 256;
pub const MAX_SCENARIO_TICKS: u32 = 3600;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub version: u32,
    pub name: String,
    pub budget_ticks: u32,
    pub steps: Vec<Action>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    SetInput {
        input: Input,
    },
    Step {
        ticks: u32,
    },
    EmitNoise {
        origin: NoiseOrigin,
        radius: f32,
    },
    Assert {
        condition: Condition,
    },
    Until {
        condition: Condition,
        max_ticks: u32,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    PositionWithin {
        id: String,
        center: [f32; 3],
        radius: f32,
    },
    NpcObserved {
        id: String,
        expected: bool,
    },
    NpcMemory {
        id: String,
        source: Option<perception::Source>,
    },
    NpcPhase {
        id: String,
        phase: perception::Phase,
    },
    PatrolPhase {
        id: String,
        phase: patrol::Phase,
    },
    PatrolVisits {
        id: String,
        at_least: u32,
    },
    TriggerOccupied {
        id: String,
        target: String,
        expected: bool,
    },
    TriggerEntries {
        id: String,
        at_least: u32,
    },
    Footsteps {
        min: u32,
        max: u32,
    },
}
impl Condition {
    fn validate(&self, scene: &Scene) -> Result<()> {
        let (id, role) = match self {
            Self::TriggerOccupied { id, target, .. } => {
                let config = trigger_config(scene, id)?;
                return if config.targets.contains(target) {
                    Ok(())
                } else {
                    Err(invalid(
                        "Trigger condition target is not selected by the volume",
                    ))
                };
            }
            Self::TriggerEntries { id, .. } => {
                trigger_config(scene, id)?;
                return Ok(());
            }
            Self::PositionWithin { id, center, radius } => {
                if center.iter().any(|n| !n.is_finite() || n.abs() > 10000.)
                    || !radius.is_finite()
                    || !(0.001..=10000.).contains(radius)
                {
                    return Err(invalid(
                        "Position condition requires finite centre within +/-10000m and radius [0.001,10000]m",
                    ));
                }
                (id, 0)
            }
            Self::NpcObserved { id, .. }
            | Self::NpcMemory { id, .. }
            | Self::NpcPhase { id, .. } => (id, 1),
            Self::PatrolPhase { id, .. } | Self::PatrolVisits { id, .. } => (id, 2),
            Self::Footsteps { min, max } => {
                return if min <= max {
                    Ok(())
                } else {
                    Err(invalid("Footstep condition requires min <= max"))
                };
            }
        };
        let entity = scene
            .entities
            .iter()
            .find(|e| e.id == *id)
            .ok_or_else(|| invalid(format!("Condition entity {id} does not exist")))?;
        if entity.body.is_none()
            || (role >= 1 && entity.npc.is_none())
            || (role == 2 && entity.npc.as_ref().is_none_or(|n| n.patrol.is_none()))
        {
            return Err(invalid(format!(
                "Condition entity {id} has no required body/NPC/patrol component"
            )));
        }
        Ok(())
    }
    fn satisfied(&self, e: &Engine) -> bool {
        match self {
            Self::PositionWithin { id, center, radius } => {
                e.simulation
                    .position(id)
                    .unwrap()
                    .iter()
                    .zip(center)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    <= radius * radius
            }
            Self::NpcObserved { id, expected } => {
                e.simulation.npcs[id].observed_this_tick == *expected
            }
            Self::NpcMemory { id, source } => {
                e.simulation.npcs[id].memory.as_ref().map(|m| m.source) == *source
            }
            Self::NpcPhase { id, phase } => e.simulation.npcs[id].phase == *phase,
            Self::PatrolPhase { id, phase } => e.simulation.patrols[id].phase == *phase,
            Self::PatrolVisits { id, at_least } => e.simulation.patrols[id].visited >= *at_least,
            Self::TriggerOccupied {
                id,
                target,
                expected,
            } => e.simulation.triggers.occupied(id, target) == Some(*expected),
            Self::TriggerEntries { id, at_least } => {
                e.simulation.triggers.volume_state(id).unwrap().entered >= *at_least
            }
            Self::Footsteps { min, max } => {
                (u64::from(*min)..=u64::from(*max)).contains(&e.simulation.footsteps.total_emitted)
            }
        }
    }
    fn actual(&self, e: &Engine) -> Value {
        match self {
            Self::PositionWithin { id, .. } => json!(e.simulation.position(id)),
            Self::NpcObserved { id, .. } => json!(e.simulation.npcs[id].observed_this_tick),
            Self::NpcMemory { id, .. } => {
                json!(e.simulation.npcs[id].memory.as_ref().map(|m| m.source))
            }
            Self::NpcPhase { id, .. } => json!(e.simulation.npcs[id].phase),
            Self::PatrolPhase { id, .. } => json!(e.simulation.patrols[id].phase),
            Self::PatrolVisits { id, .. } => json!(e.simulation.patrols[id].visited),
            Self::TriggerOccupied { id, target, .. } => {
                json!(e.simulation.triggers.occupied(id, target))
            }
            Self::TriggerEntries { id, .. } => {
                json!(e.simulation.triggers.volume_state(id).unwrap().entered)
            }
            Self::Footsteps { .. } => json!(e.simulation.footsteps.total_emitted),
        }
    }
}
fn trigger_config<'a>(scene: &'a Scene, id: &str) -> Result<&'a crate::trigger::Config> {
    scene
        .entities
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| e.trigger.as_ref())
        .ok_or_else(|| invalid(format!("Condition entity {id} has no trigger component")))
}
#[derive(Debug, Serialize)]
pub struct Check {
    pub step_index: usize,
    pub tick: u64,
    pub condition: Condition,
    pub passed: bool,
    pub actual: Value,
}
#[derive(Debug, Serialize)]
pub struct Failure {
    pub step_index: usize,
    pub tick: u64,
    pub code: String,
    pub message: String,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub version: u32,
    pub name: String,
    pub passed: bool,
    pub completed_steps: usize,
    pub ticks: u64,
    pub checks: Vec<Check>,
    pub failure: Option<Failure>,
    pub final_snapshot: Value,
}
pub struct Outcome {
    pub report: Report,
    pub engine: Engine,
}
impl Scenario {
    pub fn validate(&self, scene: &Scene) -> Result<()> {
        scene.validate()?;
        if !(1..=2).contains(&self.version)
            || self.name.is_empty()
            || self.name.len() > 128
            || self.steps.is_empty()
            || self.steps.len() > MAX_ACTIONS
            || !(1..=MAX_SCENARIO_TICKS).contains(&self.budget_ticks)
        {
            return Err(invalid(
                "Scenario v1/v2 requires name of 1..128 bytes, 1..256 actions and budget_ticks [1,3600]",
            ));
        }
        let mut ticks = 0_u64;
        let mut checks = 0;
        for action in &self.steps {
            if self.version < 2
                && matches!(
                    action,
                    Action::Assert {
                        condition: Condition::TriggerOccupied { .. }
                            | Condition::TriggerEntries { .. }
                    } | Action::Until {
                        condition: Condition::TriggerOccupied { .. }
                            | Condition::TriggerEntries { .. },
                        ..
                    }
                )
            {
                return Err(invalid("Trigger conditions require scenario version 2"));
            }
            match action {
                Action::SetInput { input } => Engine::validate_input(*input)?,
                Action::Step { ticks: n } => {
                    if *n == 0 {
                        return Err(invalid("Step ticks must be positive"));
                    }
                    ticks += u64::from(*n);
                }
                Action::EmitNoise { origin, radius } => {
                    let position = match origin {
                        NoiseOrigin::Position { position } => *position,
                        NoiseOrigin::Entity { id } => {
                            scene
                                .entities
                                .iter()
                                .find(|e| e.id == *id)
                                .ok_or_else(|| {
                                    invalid(format!("Noise entity {id} does not exist"))
                                })?
                                .transform
                                .position
                        }
                    };
                    Noise {
                        id: 0,
                        tick: 0,
                        position,
                        radius: *radius,
                        emitter_index: None,
                        emitter_instance: None,
                    }
                    .validate()?;
                }
                Action::Assert { condition } => {
                    condition.validate(scene)?;
                    checks += 1;
                }
                Action::Until {
                    condition,
                    max_ticks,
                } => {
                    condition.validate(scene)?;
                    checks += 1;
                    if *max_ticks == 0 {
                        return Err(invalid("Until max_ticks must be positive"));
                    }
                    ticks += u64::from(*max_ticks);
                }
            }
        }
        if ticks > u64::from(self.budget_ticks) || checks == 0 {
            return Err(invalid(
                "Scenario needs a check and the sum of step/until maxima must fit budget_ticks",
            ));
        }
        Ok(())
    }
    pub fn run(&self, scene: &Scene) -> Result<Outcome> {
        self.validate(scene)?;
        let mut engine = Engine::new(scene.clone())?;
        let mut checks = Vec::with_capacity(
            self.steps
                .iter()
                .filter(|a| matches!(a, Action::Assert { .. } | Action::Until { .. }))
                .count(),
        );
        let mut failure = None;
        let mut completed_steps = 0;
        for (index, action) in self.steps.iter().enumerate() {
            let result = match action {
                Action::SetInput { input } => engine
                    .execute(Command::SetInput { input: *input }.into())
                    .map(|_| ()),
                Action::Step { ticks } => engine.advance(*ticks),
                Action::EmitNoise { origin, radius } => engine
                    .execute(
                        Command::EmitNoise {
                            origin: origin.clone(),
                            radius: *radius,
                        }
                        .into(),
                    )
                    .map(|_| ()),
                Action::Assert { .. } | Action::Until { .. } => Ok(()),
            };
            if let Err(error) = result {
                failure = Some(Failure {
                    step_index: index,
                    tick: engine.simulation.tick,
                    code: error.code.into(),
                    message: error.message,
                });
                break;
            }
            let condition = match action {
                Action::Assert { condition } => Some(condition),
                Action::Until {
                    condition,
                    max_ticks,
                } => {
                    for _ in 0..*max_ticks {
                        if condition.satisfied(&engine) {
                            break;
                        }
                        engine.advance(1)?;
                    }
                    Some(condition)
                }
                _ => None,
            };
            if let Some(condition) = condition {
                let passed = condition.satisfied(&engine);
                checks.push(Check {
                    step_index: index,
                    tick: engine.simulation.tick,
                    condition: condition.clone(),
                    passed,
                    actual: condition.actual(&engine),
                });
                if !passed {
                    let code = if matches!(action, Action::Until { .. }) {
                        "condition_timeout"
                    } else {
                        "assertion_failed"
                    };
                    failure = Some(Failure {
                        step_index: index,
                        tick: engine.simulation.tick,
                        code: code.into(),
                        message:
                            "Condition was not satisfied; see check for expected and actual values"
                                .into(),
                    });
                    break;
                }
            }
            completed_steps += 1;
        }
        let report = Report {
            version: self.version,
            name: self.name.clone(),
            passed: failure.is_none(),
            completed_steps,
            ticks: engine.simulation.tick,
            checks,
            failure,
            final_snapshot: engine.snapshot(),
        };
        Ok(Outcome { report, engine })
    }
}
