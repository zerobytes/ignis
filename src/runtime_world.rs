//! Execution definitions and instance-bound placements, separate from editing history.
use crate::{
    identity::Reference,
    model::{Entity, Scene},
    targeting::Catalog,
};
use std::{collections::BTreeMap, ops::Deref};

pub type Positions = BTreeMap<String, [f32; 3]>;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    reference: Reference,
    initial: [f32; 3],
    physical: bool,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeWorld {
    navigation_source: Option<Box<Entity>>,
    definitions: Scene,
    placements: BTreeMap<String, Placement>,
}
impl RuntimeWorld {
    pub fn new(scene: &Scene, actors: &Catalog) -> Self {
        Self {
            navigation_source: scene
                .navigation_generation
                .as_ref()
                .and_then(|r| {
                    scene
                        .entities
                        .iter()
                        .chain(&scene.npc_templates)
                        .find(|e| e.id == r.entity)
                })
                .map(|e| Box::new(e.clone())),
            definitions: scene.clone(),
            placements: scene
                .entities
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    let reference = actors.entity_reference(i)?;
                    Some((
                        e.id.clone(),
                        Placement {
                            reference,
                            initial: e.transform.position,
                            physical: e.body.is_some(),
                        },
                    ))
                })
                .collect(),
        }
    }
    pub(crate) fn install(&mut self, index: usize, entity: Entity, reference: Reference) {
        self.placements.remove(&self.definitions.entities[index].id);
        self.placements.insert(
            entity.id.clone(),
            Placement {
                reference,
                initial: entity.transform.position,
                physical: true,
            },
        );
        self.definitions.entities[index] = entity;
    }
    pub(crate) fn reference(&self, id: &str) -> Option<Reference> {
        self.placements.get(id).map(|p| p.reference)
    }
    pub fn definitions(&self) -> &Scene {
        &self.definitions
    }
    pub(crate) fn set_object(&mut self, index: usize, state: crate::interactions::ObjectState) {
        let e = &mut self.definitions.entities[index];
        e.body = state.solid.then_some(crate::model::BodyKind::Fixed);
        e.visible = Some(state.visible);
        e.blocks_vision = Some(state.blocks_vision);
        e.sound_transmission = Some(state.sound_transmission);
    }
    pub(crate) fn regenerate_navigation(&mut self) {
        if let Some(mut recipe) = self.definitions.navigation_generation.take() {
            recipe.rebuild_with_source(
                &mut self.definitions,
                self.navigation_source
                    .as_deref()
                    .expect("frozen navigation profile"),
            );
            self.definitions.navigation_generation = Some(recipe);
        }
    }
    pub fn position(
        &self,
        id: &str,
        actors: &Catalog,
        body: impl FnOnce() -> Option<[f32; 3]>,
    ) -> Option<[f32; 3]> {
        let p = self.placements.get(id)?;
        if !actors.contains(p.reference) {
            return None;
        }
        if p.physical { body() } else { Some(p.initial) }
    }
    pub fn positions(
        &self,
        actors: &Catalog,
        mut body: impl FnMut(&str) -> Option<[f32; 3]>,
    ) -> Positions {
        self.placements
            .keys()
            .filter_map(|id| {
                self.position(id, actors, || body(id))
                    .map(|p| (id.clone(), p))
            })
            .collect()
    }
}
// Read-only access to execution definitions; templates remain immutable; only population installation replaces an instance definition.
impl Deref for RuntimeWorld {
    type Target = Scene;
    fn deref(&self) -> &Scene {
        self.definitions()
    }
}
/// Missing membership is absence, never an invitation to use the authored placement.
pub fn position(positions: &Positions, id: &str) -> Option<[f32; 3]> {
    positions.get(id).copied()
}

impl RuntimeWorld {
    pub(crate) fn validate_checkpoint(
        &self,
        actors: &Catalog,
        authored: &Scene,
    ) -> crate::model::Result<()> {
        if self.definitions.entities.len() != crate::templates::expand(authored).entities.len()
            || self.definitions.entities.len() > crate::model::MAX_ENTITIES
            || self.definitions.ability_catalog != authored.ability_catalog
            || self.definitions.meshes != authored.meshes
            || self.definitions.collision_meshes != authored.collision_meshes
            || self.definitions.interactions != authored.interactions
            || self.definitions.npc_templates != authored.npc_templates
            || self.definitions.avatar_audio != authored.avatar_audio
            || self.placements.len() > self.definitions.entities.len()
        {
            return Err(crate::model::invalid(
                "Checkpoint world/catalog differs from authoring contract",
            ));
        }
        let mut recipe = self.definitions.navigation_generation.clone();
        let mut original_recipe = authored.navigation_generation.clone();
        for r in [&mut recipe, &mut original_recipe].into_iter().flatten() {
            r.cached = None;
        }
        let original_source = authored.navigation_generation.as_ref().and_then(|r| {
            authored
                .entities
                .iter()
                .chain(&authored.npc_templates)
                .find(|e| e.id == r.entity)
        });
        if recipe != original_recipe
            || self.navigation_source.as_deref() != original_source
            || self.definitions.npc_capacity != authored.npc_capacity
            || self.definitions.version != authored.version
        {
            return Err(crate::model::invalid(
                "Checkpoint navigation recipe/profile changed",
            ));
        }
        let expanded = crate::templates::expand(authored);
        for (i, original) in expanded.entities.iter().enumerate() {
            let current = &self.definitions.entities[i];
            if original.body != Some(crate::model::BodyKind::Npc) {
                let mut comparable = current.clone();
                if authored
                    .interactions
                    .points
                    .iter()
                    .any(|p| p.mutable && p.entity == original.id)
                {
                    comparable.body = original.body;
                    comparable.visible = original.visible;
                    comparable.blocks_vision = original.blocks_vision;
                    comparable.sound_transmission = original.sound_transmission;
                }
                if comparable != *original {
                    return Err(crate::model::invalid(
                        "Checkpoint changed immutable world definition",
                    ));
                }
            } else {
                let canonical = |e: &Entity| {
                    let mut c = e.clone();
                    c.id = String::new();
                    c.name = String::new();
                    c.transform.position = [0.; 3];
                    if let Some(n) = &mut c.npc {
                        n.target = None;
                    }
                    c
                };
                if canonical(current) != canonical(original)
                    && !authored
                        .npc_templates
                        .iter()
                        .any(|t| canonical(t) == canonical(current))
                {
                    return Err(crate::model::invalid(
                        "Checkpoint actor is not an authored individual/template",
                    ));
                }
            }
        }
        let mut probe = self.definitions.clone();
        probe.entities.retain(|e| !crate::templates::vacant(e));
        if let Some(source) = self.navigation_source.as_deref()
            && !probe
                .entities
                .iter()
                .chain(&probe.npc_templates)
                .any(|e| e.id == source.id)
        {
            probe.entities.push(source.clone());
        }
        let names: std::collections::BTreeSet<_> =
            probe.entities.iter().map(|e| e.id.clone()).collect();
        for e in &mut probe.entities {
            if let Some(t) = &mut e.trigger {
                t.targets.retain(|id| names.contains(id));
                if t.targets.is_empty() {
                    e.trigger = None;
                }
            }
            if let Some(n) = &mut e.npc
                && n.target.as_ref().is_some_and(|t| !names.contains(t))
            {
                n.target = None;
            }
        }
        probe.ability_catalog.attributes.bases.retain(|b| {
            names.contains(&b.entity) || probe.npc_templates.iter().any(|t| t.id == b.entity)
        });
        probe.validate_inner(false)?;
        for (id, p) in &self.placements {
            let index = self
                .definitions
                .entities
                .iter()
                .position(|e| e.id == *id)
                .ok_or_else(|| crate::model::invalid("Checkpoint placement unknown"))?;
            if p.initial.iter().any(|x| !x.is_finite())
                || !actors.entity_matches(index as u16, p.reference)
                    && !(actors.entity_reference(index).is_none()
                        && self.definitions.entities[index].body
                            == Some(crate::model::BodyKind::Npc)
                        && actors.historical(p.reference))
            {
                return Err(crate::model::invalid("Checkpoint placement identity"));
            }
        }
        for (i, e) in self.definitions.entities.iter().enumerate() {
            if let Some(r) = actors.entity_reference(i)
                && self.placements.get(&e.id).is_none_or(|p| p.reference != r)
            {
                return Err(crate::model::invalid(
                    "Checkpoint presence/placement mismatch",
                ));
            }
        }
        if self.navigation_source.is_some() != self.definitions.navigation_generation.is_some() {
            return Err(crate::model::invalid(
                "Checkpoint frozen navigation source missing",
            ));
        }
        Ok(())
    }
}
