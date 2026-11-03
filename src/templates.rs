//! Authored NPC prototypes, compiled into bounded runtime storage without spawning them.
use crate::model::{BodyKind, Entity, Result, Scene, invalid};
use std::collections::BTreeSet;
pub const MAX_TEMPLATES: usize = 16;
pub const MAX_NPCS: usize = 128;
pub fn default_capacity() -> u16 {
    8
}
pub fn is_default_capacity(n: &u16) -> bool {
    *n == 8
}
pub const PREFIX: &str = "_ignis_";
/// Transitional shared actor budget; player count does not create hidden extra NPC slots.
pub fn capacity(scene: &Scene) -> usize {
    (if scene.version < 87 {
        9
    } else {
        crate::targeting::MAX_ACTORS
    })
    .saturating_sub(
        scene
            .entities
            .iter()
            .filter(|e| e.body == Some(BodyKind::Player))
            .count(),
    )
    .min(scene.npc_capacity as usize)
}
pub(crate) fn vacant(e: &Entity) -> bool {
    e.id.starts_with("_ignis_vacant_")
}

pub fn validate(scene: &Scene) -> Result<()> {
    if scene.entities.iter().any(|e| e.id.starts_with(PREFIX)) {
        return Err(invalid(
            "Entity IDs beginning _ignis_ are reserved for runtime instances",
        ));
    }
    if scene.npc_templates.is_empty() {
        return Ok(());
    }
    if scene.version < 47 || scene.npc_templates.len() > MAX_TEMPLATES {
        return Err(invalid(
            "NPC templates require scene47 and at most16 definitions",
        ));
    }
    let npcs = scene
        .entities
        .iter()
        .filter(|e| e.body == Some(BodyKind::Npc))
        .count();
    let capacity = capacity(scene);
    if npcs > capacity || scene.entities.len() + capacity - npcs > crate::model::MAX_ENTITIES {
        return Err(invalid(
            "Scene needs actor and entity storage for its reserved NPC capacity",
        ));
    }
    let mut names: BTreeSet<_> = scene.entities.iter().map(|e| e.id.as_str()).collect();
    for t in &scene.npc_templates {
        if !names.insert(t.id.as_str())
            || t.id.starts_with(PREFIX)
            || t.body != Some(BodyKind::Npc)
            || t.npc.is_none()
            || t.trigger.is_some()
        {
            return Err(invalid(
                "Template requires a unique nonreserved ID, NPC body/config and no trigger",
            ));
        }
        // Reuse component validation at the authoring boundary, never on the simulation tick.
        let mut probe = scene.clone();
        probe.npc_templates.clear();
        for e in &mut probe.entities {
            if let Some(n) = &mut e.npc {
                n.passage_priority = None;
            }
        }
        probe.entities.push(t.clone());
        probe
            .ability_catalog
            .attributes
            .bases
            .retain(|b| probe.entities.iter().any(|e| e.id == b.entity));
        probe.validate_inner(false)?;
    }
    Ok(())
}
/// Reserved slots have no identity/body/state; initial entities preserve their original indices.
pub(crate) fn expand(scene: &Scene) -> Scene {
    let mut result = scene.clone();
    if scene.npc_templates.is_empty() {
        return result;
    }
    let count = scene
        .entities
        .iter()
        .filter(|e| e.body == Some(BodyKind::Npc))
        .count();
    for i in count..capacity(scene) {
        let mut e = scene.npc_templates[0].clone();
        e.id = format!("_ignis_vacant_{i}");
        e.abilities = None;
        e.vitality = None;
        e.footsteps = None;
        let n = e.npc.as_mut().unwrap();
        n.enabled = false;
        n.target = None;
        n.patrol = None;
        n.mobility = None;
        n.passage_priority = None;
        result.entities.push(e);
    }
    result
}
