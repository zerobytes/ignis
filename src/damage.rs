//! Game-authored damage classifications; component math precedes one vitality change.
use crate::{
    abilities::{Effect, LIMIT},
    model::{Result, Scene, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const COMPONENTS: usize = 8;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub amount: i64,
    #[serde(default)]
    pub tags: Vec<String>,
}
#[derive(Clone, Copy)]
pub struct CompiledComponent {
    pub amount: i64,
    pub tags: u64,
}
pub struct Packet {
    pub components: Vec<CompiledComponent>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Breakdown {
    pub effect_index: u8,
    pub component_index: u8,
    pub tags: u64,
    pub requested: i64,
    /// Incoming factor; retained for existing inspection clients.
    pub multiplier: i64,
    pub dealt_multiplier: i64,
    pub adjusted: i64,
}
pub fn validate(scene: &Scene) -> Result<()> {
    for ability in &scene.ability_catalog.abilities {
        let mut count = 0;
        for effect in &ability.effects {
            match effect {
                Effect::Damage { .. } => count += 1,
                Effect::DamageComponents { components, .. } => {
                    if scene.version < 39 || components.is_empty() || components.len() > COMPONENTS
                    {
                        return Err(invalid(
                            "Damage components require version39 and 1..8 entries",
                        ));
                    }
                    let mut total = 0_i128;
                    for c in components {
                        if !(1..=LIMIT).contains(&c.amount) || c.tags.len() > 64 {
                            return Err(invalid("Invalid damage component amount or tags"));
                        }
                        total += i128::from(c.amount);
                    }
                    if total > i128::from(LIMIT) {
                        return Err(invalid("Damage packet total exceeds limit"));
                    }
                    count += components.len();
                }
                _ => {}
            }
        }
        if count > COMPONENTS {
            return Err(invalid(
                "At most8 damage components across all ability effects",
            ));
        }
    }
    Ok(())
}
