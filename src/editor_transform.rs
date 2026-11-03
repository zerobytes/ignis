//! World-space authoring transformations shared by native gizmos and MCP.
use crate::{
    engine::{Command, SceneEdit},
    mesh_topology::{Components, Reference},
    meshes::{self, IslandTransform},
    model::{Result, Scene, invalid},
    service::Service,
};
use glam::{Quat, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    #[default]
    Select,
    Move,
    Rotate,
    Scale,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Space {
    #[default]
    World,
    Local,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Pivot {
    #[default]
    Center,
    Origin,
    Custom,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub tool: Tool,
    pub space: Space,
    pub pivot: Pivot,
    pub custom_pivot: [f32; 3],
    pub snap: bool,
    pub move_step: f32,
    pub angle_step: f32,
    pub scale_step: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            tool: Tool::Select,
            space: Space::World,
            pivot: Pivot::Center,
            custom_pivot: [0.; 3],
            snap: false,
            move_step: 0.1,
            angle_step: 15.,
            scale_step: 0.1,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !Vec3::from(self.custom_pivot).is_finite()
            || self.custom_pivot.iter().any(|v| v.abs() > 10_000.)
            || !self.move_step.is_finite()
            || !(0.0001..=1000.).contains(&self.move_step)
            || !self.angle_step.is_finite()
            || !(0.1..=180.).contains(&self.angle_step)
            || !self.scale_step.is_finite()
            || !(0.001..=10.).contains(&self.scale_step)
        {
            return Err(invalid("Invalid transform pivot or snapping steps"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Entity {
        id: String,
    },
    Mesh {
        instance: String,
    },
    Components {
        instance: String,
        components: Components,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct Selection {
    pub target: Target,
    pub reference: Reference,
    pub center: [f32; 3],
    pub origin: [f32; 3],
    pub orientation: [f32; 4],
    pub can_rotate: bool,
}

pub fn vertices(geometry: &meshes::Geometry, components: &Components) -> Result<Vec<u32>> {
    let canonical = crate::mesh_topology::Topology::inspect(geometry)?.canonicalize(components)?;
    let mut out = match canonical {
        Components::Vertices { indices } => indices,
        Components::Edges { vertices } => vertices.into_iter().flatten().collect(),
        Components::Faces { indices } => indices
            .into_iter()
            .flat_map(|i| geometry.triangles[i as usize])
            .collect(),
    };
    out.sort_unstable();
    out.dedup();
    Ok(out)
}
fn instance<'a>(scene: &'a Scene, id: &str) -> Result<&'a meshes::Instance> {
    let instance = scene
        .meshes
        .instances
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| invalid("Transform mesh instance not found"))?;
    if instance.attachment.is_some() || instance.playback.is_some() {
        return Err(invalid(
            "Transform requires an unlinked instance without active animation",
        ));
    }
    Ok(instance)
}
pub fn selection(service: &Service, components: bool) -> Result<Selection> {
    let target = if let Some(id) = &service.selected_mesh {
        if components {
            let s = service
                .engine
                .selected_mesh_components()
                .ok_or_else(|| invalid("Selecione vértices, arestas ou faces para transformar."))?;
            let i = instance(&service.engine.scene, id)?;
            if s.asset != i.asset || s.reference != service.engine.mesh_reference() {
                return Err(invalid("Mesh selection reference is stale"));
            }
            Target::Components {
                instance: id.clone(),
                components: s.components.clone(),
            }
        } else {
            Target::Mesh {
                instance: id.clone(),
            }
        }
    } else {
        Target::Entity {
            id: service
                .engine
                .selected_id()
                .ok_or_else(|| invalid("Selecione um objeto para transformar."))?
                .into(),
        }
    };
    describe(service, target)
}
pub fn describe(service: &Service, target: Target) -> Result<Selection> {
    let (center, origin, orientation, can_rotate) = match &target {
        Target::Entity { id } => {
            let e = service
                .engine
                .scene
                .entities
                .iter()
                .find(|e| &e.id == id)
                .ok_or_else(|| invalid("Transform entity not found"))?;
            (
                e.transform.position,
                e.transform.position,
                [0., 0., 0., 1.],
                false,
            )
        }
        Target::Mesh { instance: id } | Target::Components { instance: id, .. } => {
            let i = instance(&service.engine.scene, id)?;
            let a = service.engine.scene.meshes.asset(&i.asset)?;
            let indices = if let Target::Components { components, .. } = &target {
                Some(vertices(&a.geometry, components)?)
            } else {
                None
            };
            let center = if let Some(indices) = indices {
                indices
                    .iter()
                    .map(|n| Vec3::from(a.geometry.positions[*n as usize]))
                    .sum::<Vec3>()
                    / indices.len() as f32
            } else {
                let (min, max) = a.geometry.positions.iter().fold(
                    (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                    |(min, max), p| (min.min(Vec3::from(*p)), max.max(Vec3::from(*p))),
                );
                (min + max) * 0.5
            };
            (
                (Vec3::from(i.position) + Quat::from_array(i.rotation) * (center * i.scale))
                    .to_array(),
                i.position,
                i.rotation,
                true,
            )
        }
    };
    Ok(Selection {
        target,
        reference: service.engine.mesh_reference(),
        center,
        origin,
        orientation,
        can_rotate,
    })
}
pub fn edits(
    service: &Service,
    target: &Target,
    reference: &Reference,
    t: &IslandTransform,
) -> Result<Vec<SceneEdit>> {
    if service.engine.playing {
        return Err(invalid("Pause simulation before transforming"));
    }
    if reference != &service.engine.mesh_reference() {
        return Err(invalid("Transform reference is stale"));
    }
    let q = Quat::from_array(t.rotation);
    if !Vec3::from(t.pivot).is_finite()
        || !Vec3::from(t.translation).is_finite()
        || !q.is_finite()
        || (q.length_squared() - 1.).abs() > 0.001
        || !t.scale.is_finite()
        || !(0.001..=100.).contains(&t.scale)
    {
        return Err(invalid(
            "Transform needs finite values, unit quaternion and scale0.001..100",
        ));
    }
    let q = q.normalize();
    let apply = |p: [f32; 3]| {
        (Vec3::from(t.pivot)
            + q * ((Vec3::from(p) - Vec3::from(t.pivot)) * t.scale)
            + Vec3::from(t.translation))
        .to_array()
    };
    let edit = match target {
        Target::Entity { id } => {
            if q.xyz().length() > 0.000001 {
                return Err(invalid(
                    "Axis-aligned physics entities do not support arbitrary rotation",
                ));
            }
            let mut e = service
                .engine
                .scene
                .entities
                .iter()
                .find(|e| &e.id == id)
                .cloned()
                .ok_or_else(|| invalid("Transform entity not found"))?;
            e.transform.position = apply(e.transform.position);
            e.transform.size = (Vec3::from(e.transform.size) * t.scale).to_array();
            SceneEdit::Upsert {
                entity: Box::new(e),
            }
        }
        Target::Mesh { instance: id } => {
            let mut i = instance(&service.engine.scene, id)?.clone();
            i.position = apply(i.position);
            i.rotation = (q * Quat::from_array(i.rotation)).normalize().to_array();
            i.scale *= t.scale;
            SceneEdit::Mesh {
                edit: meshes::Edit::PutInstance { instance: i },
            }
        }
        Target::Components {
            instance: id,
            components,
        } => {
            let i = instance(&service.engine.scene, id)?;
            let rotation = Quat::from_array(i.rotation).normalize();
            let inverse = rotation.conjugate();
            let pivot = (inverse * (Vec3::from(t.pivot) - Vec3::from(i.position))) / i.scale;
            let translation = (inverse * Vec3::from(t.translation)) / i.scale;
            let transform = IslandTransform {
                pivot: pivot.to_array(),
                translation: translation.to_array(),
                rotation: (inverse * q * rotation).normalize().to_array(),
                scale: t.scale,
            };
            SceneEdit::Mesh {
                edit: meshes::Edit::TransformVertices {
                    asset: i.asset.clone(),
                    reference: reference.clone(),
                    vertices: vertices(
                        &service.engine.scene.meshes.asset(&i.asset)?.geometry,
                        components,
                    )?,
                    transform,
                },
            }
        }
    };
    Ok(vec![edit])
}
/// No history/runtime mutation. The commit uses the identical edit planner and validators.
pub fn preview(
    service: &Service,
    target: &Target,
    reference: &Reference,
    transform: &IslandTransform,
) -> Result<Scene> {
    service
        .engine
        .preview_edits(edits(service, target, reference, transform)?)
}
pub fn commit(
    service: &mut Service,
    target: &Target,
    reference: &Reference,
    transform: &IslandTransform,
) -> Result<serde_json::Value> {
    let edits = edits(service, target, reference, transform)?;
    let reply = service
        .engine
        .execute(Command::EditScene { edits }.into())?;
    if let Target::Components {
        instance: id,
        components,
    } = target
    {
        let asset = instance(&service.engine.scene, id)?.asset.clone();
        service.engine.execute(
            Command::SelectMeshComponents {
                selection: Some(crate::mesh_topology::Selection {
                    asset,
                    reference: service.engine.mesh_reference(),
                    components: components.clone(),
                }),
            }
            .into(),
        )?;
    }
    Ok(reply)
}
