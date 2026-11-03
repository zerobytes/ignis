//! NPC intent over current accepted evidence; never an alternative ability executor.
use crate::{
    abilities::{Recipient, Selection},
    model::{Result, invalid},
    targeting::Attitude,
    vitality::Health,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub require_dispel_match: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allies: Option<Allies>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_aim: Option<SpatialAim>,
    #[serde(default)]
    pub vitality_below: Option<Threshold>,
    #[serde(default)]
    pub approach: Option<Approach>,
}
/// Support selection is independent from pursuit memory.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Allies {
    pub ranking: Ranking,
    #[serde(default)]
    pub include_self: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Ranking {
    LowestHealthRatio,
    Nearest,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Support {
    pub knowledge: crate::recognition::Knowledge,
    pub observations: Vec<Option<crate::perception::Observation>>,
}
/// How a spatial ability obtains an automatic aim. No actor tracking in flight.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpatialAim {
    ObservedPoint { range: f32 },
}
impl SpatialAim {
    pub fn range(self) -> f32 {
        match self {
            Self::ObservedPoint { range } => range,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Threshold {
    pub subject: Recipient,
    /// Strictly below; 10000 basis points = 100 percent.
    pub basis_points: u16,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Approach {
    pub margin: f32,
}
impl Policy {
    pub fn range(self, actor_range: f32) -> f32 {
        self.spatial_aim.map_or(actor_range, SpatialAim::range)
    }
    pub fn validate(self, version: u32, selection: Selection, range: f32) -> Result<()> {
        if version < 28 {
            return Err(invalid("Ability policy requires scene version 28"));
        }
        if (selection == Selection::Spatial) != self.spatial_aim.is_some()
            || self.spatial_aim.is_some_and(|a| {
                version < 32 || !a.range().is_finite() || !(0.1..=100.).contains(&a.range())
            })
        {
            return Err(invalid(
                "Automatic spatial abilities require version32 and spatial_aim observed_point range0.1..100; other selections cannot use spatial_aim",
            ));
        }
        if self.allies.is_some() && (version < 44 || selection != Selection::Actor) {
            return Err(invalid("Ally triage requires scene44 and actor selection"));
        }
        if self.require_dispel_match && (version < 45 || selection == Selection::Spatial) {
            return Err(invalid(
                "Dispel policy requires scene45 and actor/self_actor selection",
            ));
        }
        let range = self.range(range);
        if self
            .vitality_below
            .is_some_and(|t| !(1..=10000).contains(&t.basis_points))
        {
            return Err(invalid("Vitality threshold requires 1..10000 basis points"));
        }
        if self.approach.is_some_and(|a| {
            selection == Selection::SelfActor
                || !a.margin.is_finite()
                || a.margin < 0.1
                || a.margin > range - 0.1
        }) {
            return Err(invalid(
                "Approach requires actor/spatial selection and margin between 0.1 and range minus 0.1 metres",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Unconfigured,
    Unavailable,
    InsufficientResources,
    NoVisibleTarget,
    NoEligibleAlly,
    NoMatchingEffect,
    RelationMismatch,
    MissingVitality,
    VitalitySatisfied,
    OutOfRange,
    TargetChanged,
    Approaching,
    Ready,
    Casting,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Decision {
    pub phase: Phase,
    pub target: Option<u16>,
}
impl Decision {
    pub fn eligible(self) -> bool {
        matches!(
            self.phase,
            Phase::Approaching | Phase::Ready | Phase::Casting
        )
    }
    pub fn execute(self) -> bool {
        matches!(self.phase, Phase::Ready | Phase::Casting)
    }
}
pub struct Offer {
    pub selection: Selection,
    pub range: f32,
    pub relation: Option<Attitude>,
    pub affordable: bool,
    pub cast_target: Option<u16>,
}
/// A target is supplied only after this tick's confirmed direct observation.
pub struct Target {
    pub actor: u16,
    pub position: [f32; 3],
    pub health: Option<Health>,
    pub relation: Attitude,
}
pub struct Evidence {
    pub actor: u16,
    pub position: [f32; 3],
    pub health: Option<Health>,
    pub target: Option<Target>,
}
pub fn evaluate(
    policy: Policy,
    offer: &Offer,
    evidence: &Evidence,
    previous: Decision,
) -> Decision {
    let mut decision = Decision {
        phase: Phase::Ready,
        target: None,
    };
    if !offer.affordable {
        decision.phase = Phase::InsufficientResources;
        return decision;
    }
    let (target, position, health, relation) = match offer.selection {
        Selection::SelfActor => (
            evidence.actor,
            evidence.position,
            evidence.health,
            Attitude::Allied,
        ),
        Selection::Actor | Selection::Spatial => {
            let Some(t) = &evidence.target else {
                decision.phase = Phase::NoVisibleTarget;
                return decision;
            };
            (t.actor, t.position, t.health, t.relation)
        }
    };
    decision.target = Some(target);
    if offer.selection != Selection::SelfActor && offer.relation.is_some_and(|r| r != relation) {
        decision.phase = Phase::RelationMismatch;
        return decision;
    }
    if let Some(t) = policy.vitality_below {
        let Some(h) = (if t.subject == Recipient::Source {
            evidence.health
        } else {
            health
        }) else {
            decision.phase = Phase::MissingVitality;
            return decision;
        };
        if h.units * 10000 >= h.maximum * i64::from(t.basis_points) {
            decision.phase = Phase::VitalitySatisfied;
            return decision;
        }
    }
    let distance_squared = glam::Vec3::from(evidence.position).distance_squared(position.into());
    let effective_range = policy.range(offer.range);
    if offer.selection == Selection::Spatial
        && offer.cast_target.is_some()
        && distance_squared > effective_range * effective_range
    {
        decision.phase = Phase::OutOfRange;
        return decision;
    }
    if let Some(cast_target) = offer.cast_target {
        decision.phase = if cast_target == target {
            Phase::Casting
        } else {
            Phase::TargetChanged
        };
        return decision;
    }
    let approaching = previous.phase == Phase::Approaching && previous.target == Some(target);
    let entry_range = effective_range - policy.approach.map_or(0., |a| a.margin);
    let range = if approaching {
        entry_range
    } else {
        effective_range
    };
    if distance_squared > range * range {
        decision.phase = if policy.approach.is_some() {
            Phase::Approaching
        } else {
            Phase::OutOfRange
        };
    }
    decision
}

/// Receives only current confirmed observations (plus explicitly allowed self knowledge).
/// Caller iterates actor IDs in catalog order, so equal scores preserve the lower ID.
pub fn choose_ally(
    policy: Policy,
    offer: &Offer,
    own: &Evidence,
    previous: Decision,
    candidates: impl Iterator<Item = Target>,
    mut matches_effects: impl FnMut(u16) -> bool,
) -> Decision {
    let config = policy.allies.expect("triage policy");
    if !offer.affordable {
        return Decision {
            phase: Phase::InsufficientResources,
            target: None,
        };
    }
    let mut best: Option<(Decision, Option<Health>, f32)> = None;
    for target in candidates {
        if target.relation != Attitude::Allied
            || (target.actor == own.actor && !config.include_self)
            || offer.cast_target.is_some_and(|a| a != target.actor)
            || (config.ranking == Ranking::LowestHealthRatio && target.health.is_none())
        {
            continue;
        }
        let distance = glam::Vec3::from(own.position).distance_squared(target.position.into());
        let health = target.health;
        let evidence = Evidence {
            actor: own.actor,
            position: own.position,
            health: own.health,
            target: Some(target),
        };
        let decision = evaluate(policy, offer, &evidence, previous);
        if !decision.eligible() || !matches_effects(decision.target.unwrap()) {
            continue;
        }
        let better = best.as_ref().is_none_or(|(_, h, d)| {
            let order = match config.ranking {
                Ranking::LowestHealthRatio => {
                    let a = health.unwrap();
                    let b = h.unwrap();
                    (i128::from(a.units) * i128::from(b.maximum))
                        .cmp(&(i128::from(b.units) * i128::from(a.maximum)))
                }
                Ranking::Nearest => std::cmp::Ordering::Equal,
            };
            order.is_lt() || (order.is_eq() && distance < *d)
        });
        if better {
            best = Some((decision, health, distance));
        }
    }
    best.map_or(
        Decision {
            phase: Phase::NoEligibleAlly,
            target: None,
        },
        |b| b.0,
    )
}
