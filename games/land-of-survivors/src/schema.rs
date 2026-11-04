use ignis::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub version: u32,
    pub abilities: Vec<Ability>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub id: String,
    pub name_en: String,
    pub name_pt: String,
    pub description_en: String,
    pub description_pt: String,
    pub class_id: String,
    pub specialization: String,
    pub form: Form,
    pub focuses: Vec<Focus>,
    pub delivery: Delivery,
    pub cost: f64,
    pub cooldown: f64,
    pub cast: f64,
    pub range: f32,
    pub radius: f32,
    pub speed: f32,
    pub diameter: f32,
    pub duration: f64,
    pub interval: f64,
    pub periodic: bool,
    pub shots: u8,
    pub chain: u8,
    pub execute: bool,
    pub burn: f64,
    pub effects: Vec<Effect>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Focus {
    Dps,
    Healer,
    Tank,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    #[default]
    Humanoid,
    Bear,
    Feline,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    SelfActor,
    Target,
    Ground,
    Zone,
    Cone,
    Dash,
    Projectile,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    SelfActor,
    Ally,
    Enemy,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    Damage,
    Heal,
    Shield,
    Mitigation,
    Slow,
    Root,
    Cleanse,
    Mana,
    Taunt,
    Push,
    Form,
    Move,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub kind: EffectKind,
    pub relation: Relation,
    pub amount: f64,
    pub seconds: f64,
    #[serde(default)]
    pub form: Form,
}
pub fn ticks(seconds: f64) -> u64 {
    (seconds * 60.).round() as u64
}
impl Catalog {
    pub fn bundled() -> Result<Self> {
        let c: Self = serde_json::from_str(include_str!("../content/abilities.json"))
            .map_err(|e| invalid(e.to_string()))?;
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.abilities.is_empty() || self.abilities.len() > 1024 {
            return Err(invalid("Invalid game ability catalog version/count"));
        }
        let mut ids = BTreeSet::new();
        for a in &self.abilities {
            if a.id.is_empty()
                || a.id.len() > 80
                || !ids.insert(&a.id)
                || a.effects.is_empty()
                || a.effects.len() > 16
                || a.focuses.is_empty()
                || a.focuses.len() > 3
                || !["mage", "warrior", "archer", "druid"].contains(&a.class_id.as_str())
                || !["fire", "frost", "aether", "arms", "marksmanship", "nature"]
                    .contains(&a.specialization.as_str())
                || !a
                    .focuses
                    .iter()
                    .all(|f| a.focuses.iter().filter(|x| *x == f).count() == 1)
                || ![a.cost, a.cooldown, a.cast, a.duration, a.interval, a.burn]
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0. && *v <= 3600.)
                || ![a.range, a.radius, a.speed, a.diameter]
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0. && *v <= 100.)
                || a.interval < 1. / 60.
                || !(1..=16).contains(&a.shots)
                || !(1..=16).contains(&a.chain)
                || a.delivery == Delivery::Projectile
                    && (a.speed <= 0. || a.diameter <= 0. || a.range <= 0.)
                || a.effects.iter().any(|e| {
                    !e.amount.is_finite()
                        || e.amount < 0.
                        || e.amount > 1e6
                        || !e.seconds.is_finite()
                        || !(0. ..=3600.).contains(&e.seconds)
                        || matches!(e.kind, EffectKind::Slow | EffectKind::Mitigation)
                            && e.amount > 1.
                })
            {
                return Err(invalid(format!("Invalid ability {}", a.id)));
            }
        }
        Ok(())
    }
    pub fn get(&self, id: &str) -> Result<&Ability> {
        self.abilities
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| invalid(format!("Unknown ability {id}")))
    }
}
