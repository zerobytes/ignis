//! Native/MCP modeling session boundary. Geometry and validation remain in the shared engine.
use crate::{
    engine::Command,
    mesh_modeling::{Operation, Report},
    mesh_topology::{Components, Reference, Selection},
    meshes,
    model::{Result, Scene, invalid},
    service::Service,
};
fn edit(
    service: &Service,
    instance: &str,
    reference: &Reference,
    operation: &Operation,
) -> Result<meshes::Edit> {
    if service.engine.playing {
        return Err(invalid("Pause simulation before component modeling"));
    }
    let i = service
        .engine
        .scene
        .meshes
        .instances
        .iter()
        .find(|i| i.id == instance)
        .ok_or_else(|| invalid("Modeling mesh instance not found"))?;
    if i.playback.is_some() || i.attachment.is_some() {
        return Err(invalid(
            "Edit the rest mesh without active animation/attachment before modeling",
        ));
    }
    Ok(meshes::Edit::ModelSelection {
        asset: i.asset.clone(),
        reference: reference.clone(),
        operation: operation.clone(),
    })
}
pub fn preview(
    service: &Service,
    instance: &str,
    reference: &Reference,
    operation: &Operation,
) -> Result<(Scene, Report)> {
    let (scene, report) = service
        .engine
        .prepare_mesh_edit(&edit(service, instance, reference, operation)?)?;
    Ok((scene, report.expect("model operation returns a report")))
}
pub fn execute(
    service: &mut Service,
    instance: &str,
    reference: &Reference,
    operation: &Operation,
    preview_only: bool,
) -> Result<serde_json::Value> {
    if preview_only {
        let (scene, report) = preview(service, instance, reference, operation)?;
        return Ok(serde_json::json!({"scene":scene,"modeling_report":report}));
    }
    let command = Command::EditMesh {
        edit: edit(service, instance, reference, operation)?,
    };
    let mut data = service.engine.execute(command.into())?;
    let report: Report = serde_json::from_value(data["modeling_report"].clone())
        .expect("engine returns typed modeling report");
    let asset = service
        .engine
        .scene
        .meshes
        .instances
        .iter()
        .find(|i| i.id == instance)
        .expect("modeling preserves instance")
        .asset
        .clone();
    service.select_mesh(Some(instance.into()))?;
    service.engine.execute(
        Command::SelectMeshComponents {
            selection: Some(Selection {
                asset,
                reference: service.engine.mesh_reference(),
                components: Components::Faces {
                    indices: report.selected_faces,
                },
            }),
        }
        .into(),
    )?;
    data["selection"] = serde_json::to_value(service.engine.selected_mesh_components())
        .expect("selection serializes");
    Ok(data)
}
