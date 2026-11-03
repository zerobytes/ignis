//! Compiled population choices. Retries keep the chosen binding until a successful birth.
use crate::{
    model::{Result, Scene, invalid},
    targeting::Catalog,
    waves::Binding,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_CHOICES: usize = 16;
pub const MAX_WEIGHT: u32 = 1_000_000;
pub const SEED_SALT: u32 = 0x9e37_79b9;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub template: String,
    pub weight: u32,
}
pub fn validate(scene: &Scene, single: Option<&str>, choices: &[Choice]) -> Result<()> {
    if let Some(id) = single {
        if !choices.is_empty() || !scene.npc_templates.iter().any(|t| t.id == id) {
            return Err(invalid(
                "Choose either one known template or weighted templates",
            ));
        }
    } else if scene.version < 65
        || choices.is_empty()
        || choices.len() > MAX_CHOICES
        || choices.iter().enumerate().any(|(i, c)| {
            c.weight == 0
                || c.weight > MAX_WEIGHT
                || !scene.npc_templates.iter().any(|t| t.id == c.template)
                || choices[..i].iter().any(|p| p.template == c.template)
        })
    {
        return Err(invalid(
            "Weighted templates require scene65, 1..16 unique known templates and integer weights1..1000000",
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub binding: Binding,
    pub upper: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selection {
    Single { binding: Binding },
    Weighted { entries: Vec<Entry>, total: u32 },
}
impl Selection {
    fn compile(scene: &Scene, actors: &Catalog, single: Option<&str>, choices: &[Choice]) -> Self {
        if let Some(id) = single {
            return Self::Single {
                binding: Binding::compile(scene, actors, id),
            };
        }
        let mut total = 0;
        let entries = choices
            .iter()
            .map(|c| {
                total += c.weight;
                Entry {
                    binding: Binding::compile(scene, actors, &c.template),
                    upper: total,
                }
            })
            .collect();
        Self::Weighted { entries, total }
    }
    pub fn first(&self) -> Binding {
        match self {
            Self::Single { binding } => *binding,
            Self::Weighted { entries, .. } => entries[0].binding,
        }
    }
    pub fn select(&self, rng: &mut u32) -> (Binding, bool) {
        match self {
            Self::Single { binding } => (*binding, false),
            Self::Weighted { entries, total } => {
                let bucket =
                    ((u64::from(crate::spawn_region::draw(rng)) * u64::from(*total)) >> 32) as u32;
                (
                    entries.iter().find(|e| bucket < e.upper).unwrap().binding,
                    true,
                )
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sources {
    pub base: Selection,
    pub waves: Vec<Option<Selection>>,
}
impl Sources {
    pub fn compile(scene: &Scene, actors: &Catalog, g: &crate::generators::Config) -> Self {
        Self {
            base: Selection::compile(
                scene,
                actors,
                (!g.template.is_empty()).then_some(g.template.as_str()),
                &g.templates,
            ),
            waves: g
                .waves
                .iter()
                .map(|w| {
                    if w.template.is_none() && w.templates.is_empty() {
                        None
                    } else {
                        Some(Selection::compile(
                            scene,
                            actors,
                            w.template.as_deref(),
                            &w.templates,
                        ))
                    }
                })
                .collect(),
        }
    }
    pub fn current(&self, wave: Option<u8>) -> &Selection {
        wave.and_then(|i| self.waves[i as usize].as_ref())
            .unwrap_or(&self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weighted_bucket_covers_zero_boundary_and_max_without_gaps() {
        let selection = Selection::Weighted {
            entries: vec![
                Entry {
                    binding: Binding {
                        template: 0,
                        target: None,
                    },
                    upper: 3,
                },
                Entry {
                    binding: Binding {
                        template: 1,
                        target: None,
                    },
                    upper: 4,
                },
            ],
            total: 4,
        };
        // LCG preimages for draws0, 0xbfffffff, 0xc0000000 and u32::MAX.
        for (seed, draw, expected) in [
            (634785765, 0, 0),
            (3874862880, 0xbfffffff, 0),
            (3856011237, 0xc0000000, 1),
            (653637408, u32::MAX, 1),
        ] {
            let mut rng = seed;
            let (binding, weighted) = selection.select(&mut rng);
            assert!(weighted);
            assert_eq!(rng, draw);
            assert_eq!(binding.template, expected);
        }
    }
}
