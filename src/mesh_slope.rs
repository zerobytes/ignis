//! Cold, disjoint slope partition of a native static mesh. No overlapping grass shell.
use crate::{
    engine::SceneEdit,
    meshes::{self, Geometry},
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Style {
    pub instance: String,
    pub wall_asset: String,
    pub top_asset: String,
    pub top_instance: String,
    pub min_up: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chamfer: Option<crate::mesh_chamfer::Config>,
    pub walls: crate::materials::Material,
    pub top: crate::materials::Material,
    pub wall_color: [f32; 3],
    pub top_color: [f32; 3],
}
pub fn plan(scene: &crate::model::Scene, s: Style) -> Result<(Vec<SceneEdit>, serde_json::Value)> {
    if !s.min_up.is_finite() || !(0. ..=1.).contains(&s.min_up) {
        return Err(invalid("Slope threshold requires0..1"));
    }
    let instance = scene
        .meshes
        .instances
        .iter()
        .find(|i| i.id == s.instance)
        .ok_or_else(|| invalid("Unknown slope instance"))?;
    let asset = scene
        .meshes
        .assets
        .iter()
        .find(|a| a.id == instance.asset)
        .unwrap();
    if asset.rig.is_some() || !asset.clips.is_empty() || instance.attachment.is_some() {
        return Err(invalid(
            "Slope partition requires an unattached static mesh",
        ));
    }
    if s.wall_asset == s.top_asset
        || scene
            .meshes
            .assets
            .iter()
            .any(|a| a.id == s.wall_asset || a.id == s.top_asset)
        || scene
            .meshes
            .instances
            .iter()
            .any(|i| i.id == s.top_instance)
    {
        return Err(invalid("Slope output IDs must be distinct and unused"));
    }
    let refined = s
        .chamfer
        .as_ref()
        .map(|c| crate::mesh_chamfer::apply(&asset.geometry, c))
        .transpose()?;
    let g = refined.as_ref().unwrap_or(&asset.geometry);
    if g.positions.len() > 1_048_576 || g.triangles.len() > 2_000_000 {
        return Err(invalid(
            "Slope partition exceeds cold-operation budget; partition the source into smaller authored assets",
        ));
    }
    let mut outputs = [
        Geometry {
            positions: vec![],
            triangles: vec![],
            material_coordinates: Some(vec![]),
        },
        Geometry {
            positions: vec![],
            triangles: vec![],
            material_coordinates: Some(vec![]),
        },
    ];
    let mut maps = [
        vec![u32::MAX; g.positions.len()],
        vec![u32::MAX; g.positions.len()],
    ];
    let rotation = glam::Quat::from_array(instance.rotation);
    for &face in &g.triangles {
        let p = face.map(|i| glam::Vec3::from_array(g.positions[i as usize]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let group = usize::from((rotation * normal).y >= s.min_up);
        let out = &mut outputs[group];
        let map = &mut maps[group];
        let triangle = face.map(|v| {
            let slot = &mut map[v as usize];
            if *slot == u32::MAX {
                *slot = out.positions.len() as u32;
                out.positions.push(g.positions[v as usize]);
                if let Some(uv) = out.material_coordinates.as_mut() {
                    uv.push(g.material_coordinates.as_ref().map_or_else(
                        || {
                            (rotation
                                * (glam::Vec3::from_array(g.positions[v as usize])
                                    * instance.scale)
                                + glam::Vec3::from_array(instance.position))
                            .to_array()
                        },
                        |uv| uv[v as usize],
                    ));
                }
            }
            *slot
        });
        out.triangles.push(triangle);
    }
    if outputs.iter().any(|o| o.triangles.is_empty()) {
        return Err(invalid(
            "Slope split requires both top and wall faces at the selected threshold",
        ));
    }
    for o in &outputs {
        o.validate()?;
    }
    let counts = outputs.each_ref().map(|o| o.triangles.len());
    let [walls, top] = outputs;
    let mut wall_asset = asset.clone();
    wall_asset.id = s.wall_asset.clone();
    wall_asset.geometry = Arc::new(walls);
    wall_asset.material = s.walls;
    let mut top_asset = asset.clone();
    top_asset.id = s.top_asset.clone();
    top_asset.geometry = Arc::new(top);
    top_asset.material = s.top;
    let mut wall_instance = instance.clone();
    wall_instance.asset = s.wall_asset;
    wall_instance.color = s.wall_color;
    let mut top_instance = instance.clone();
    top_instance.id = s.top_instance.clone();
    top_instance.asset = s.top_asset;
    top_instance.color = s.top_color;
    let mut edits = vec![
        SceneEdit::Mesh {
            edit: meshes::Edit::PutAsset { asset: wall_asset },
        },
        SceneEdit::Mesh {
            edit: meshes::Edit::PutAsset { asset: top_asset },
        },
        SceneEdit::Mesh {
            edit: meshes::Edit::PutInstance {
                instance: wall_instance,
            },
        },
        SceneEdit::Mesh {
            edit: meshes::Edit::PutInstance {
                instance: top_instance,
            },
        },
    ];
    if scene.collision_meshes.contains(&s.instance) {
        edits.push(SceneEdit::SetMeshCollision {
            instance: s.top_instance,
            enabled: true,
        });
    }
    if scene
        .meshes
        .instances
        .iter()
        .filter(|i| i.asset == asset.id)
        .count()
        == 1
    {
        edits.push(SceneEdit::Mesh {
            edit: meshes::Edit::RemoveAsset {
                id: asset.id.clone(),
            },
        });
    }
    Ok((
        edits,
        serde_json::json!({"wall_triangles":counts[0],"top_triangles":counts[1],"source_triangles":g.triangles.len(),"disjoint":true,"geometry_displaced":s.chamfer.is_some(),"input_triangles":asset.geometry.triangles.len(),"chamfer":s.chamfer}),
    ))
}
