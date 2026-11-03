//! Explicit visual object membership, independent of transforms, materials and physics.
use crate::{
    meshes::Document,
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub id: String,
    pub members: Vec<String>,
}
pub fn is_empty(objects: &Arc<Vec<Object>>) -> bool {
    objects.is_empty()
}
fn id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(invalid("Object IDs require 1..64 ASCII letters/digits/_/-"));
    }
    Ok(())
}
pub fn validate(doc: &Document) -> Result<()> {
    if doc.objects.is_empty() {
        return Ok(());
    }
    if doc.objects.len() > doc.instances.len() {
        return Err(invalid("More visual objects than mesh instances"));
    }
    let instances: BTreeSet<_> = doc.instances.iter().map(|i| i.id.as_str()).collect();
    let mut names = BTreeSet::new();
    let mut used = BTreeSet::new();
    for object in doc.objects.iter() {
        id(&object.id)?;
        if !names.insert(&object.id)
            || object.members.is_empty()
            || object.members.len() > doc.instances.len()
        {
            return Err(invalid(
                "Visual objects require unique IDs and nonempty members",
            ));
        }
        for member in &object.members {
            if !instances.contains(member.as_str()) || !used.insert(member) {
                return Err(invalid(
                    "Each object member must exist and belong to at most one visual object",
                ));
            }
        }
    }
    Ok(())
}
pub(crate) fn put(doc: &mut Document, object: Object) -> Result<()> {
    id(&object.id)?;
    // Validation belongs to the staged transaction; no caller sees half a group.
    let objects = Arc::make_mut(&mut doc.objects);
    if let Some(old) = objects.iter_mut().find(|o| o.id == object.id) {
        *old = object;
    } else {
        objects.push(object);
    }
    Ok(())
}
pub(crate) fn remove(doc: &mut Document, object: &str) -> Result<()> {
    let objects = Arc::make_mut(&mut doc.objects);
    let n = objects
        .iter()
        .position(|o| o.id == object)
        .ok_or_else(|| invalid("Visual object not found"))?;
    objects.remove(n);
    Ok(())
}
pub(crate) fn assign(doc: &mut Document, instance: &str, object: Option<String>) -> Result<()> {
    if !doc.instances.iter().any(|i| i.id == instance) {
        return Err(invalid("Mesh instance not found"));
    }
    if let Some(name) = &object {
        id(name)?;
    }
    if doc
        .objects
        .iter()
        .find(|g| g.members.iter().any(|m| m == instance))
        .map(|g| g.id.as_str())
        == object.as_deref()
    {
        return Ok(());
    }
    // Explicit transfer is one transaction, including removal of an emptied group.
    let objects = Arc::make_mut(&mut doc.objects);
    for group in objects.iter_mut() {
        group.members.retain(|m| m != instance);
    }
    if let Some(name) = object {
        if let Some(group) = objects.iter_mut().find(|g| g.id == name) {
            group.members.push(instance.to_owned());
        } else {
            objects.push(Object {
                id: name,
                members: vec![instance.to_owned()],
            });
        }
    }
    objects.retain(|g| !g.members.is_empty());
    Ok(())
}
pub(crate) fn forget(doc: &mut Document, instance: &str) {
    if doc.objects.is_empty() {
        return;
    }
    let objects = Arc::make_mut(&mut doc.objects);
    for object in objects.iter_mut() {
        object.members.retain(|m| m != instance);
    }
    objects.retain(|o| !o.members.is_empty());
}

pub fn inspect(doc: &Document, object: Option<&str>) -> Result<serde_json::Value> {
    let members = doc.objects.iter().map(|o| o.members.len()).sum::<usize>();
    let mut result = serde_json::json!({"objects":doc.objects.len(),"grouped_instances":members,
        "ungrouped_instances":doc.instances.len()-members,"transforms":"independent","physics":"unchanged"});
    if let Some(name) = object {
        let group = doc
            .objects
            .iter()
            .find(|o| o.id == name)
            .ok_or_else(|| invalid("Visual object not found"))?;
        result["object"] = serde_json::to_value(group).map_err(|e| invalid(e.to_string()))?;
    }
    Ok(result)
}
