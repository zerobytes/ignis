//! Session-only character visuals. Physics, saves and received authority stay untouched.
use anyhow::{Result, ensure};
use glam::{Quat, Vec3};
use ignis::{
    materials::Material,
    meshes::{Document, Frame},
    render::{RenderFrame, Vertex},
};
use std::collections::BTreeMap;

struct Model {
    vertices: Vec<Vertex>,
    materials: Vec<Material>,
    min_y: f32,
    height: f32,
}
impl Model {
    fn new(document: Document) -> Result<Self> {
        document.validate()?;
        let mut frame = Frame::default();
        // Bake the existing native rest pose once, including all eye/skin parts.
        // Keeping expanded vertices here avoids recompiling or skinning per frame.
        frame.set_unique_static_residency(false);
        frame.update(&document, 0);
        ensure!(
            !frame.vertices.is_empty(),
            "Character model has no triangles"
        );
        let min_y = frame
            .vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = frame
            .vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        ensure!(max_y - min_y > 0.01, "Character model has no height");
        Ok(Self {
            vertices: frame.vertices,
            materials: frame.materials,
            min_y,
            height: max_y - min_y,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub actor: u64,
    pub life: u64,
    pub center: [f32; 3],
    pub foot_y: f32,
    pub selected_model: bool,
}
#[derive(Clone, Copy)]
struct Heading {
    life: u64,
    center: Vec3,
    yaw: f32,
}

pub struct Presentation {
    selected: Model,
    traveler: Option<Model>,
    headings: BTreeMap<u64, Heading>,
    pub placements: Vec<Placement>,
}
impl Presentation {
    pub fn new(selected: Document, traveler: Option<Document>) -> Result<Self> {
        Ok(Self {
            selected: Model::new(selected)?,
            traveler: traveler.map(Model::new).transpose()?,
            headings: BTreeMap::new(),
            placements: Vec::new(),
        })
    }
    pub fn apply(&mut self, frame: &mut RenderFrame, local: Option<u64>) -> Result<()> {
        // This game's session stage contains no authored mesh assets. Appending
        // after native frame preparation is a transient visual projection only.
        self.placements.clear();
        let mut removal = Vec::with_capacity(frame.player_primitives.len());
        for p in &frame.player_primitives {
            let b = frame
                .boxes
                .get(p.box_index)
                .ok_or_else(|| anyhow::anyhow!("Player primitive is absent"))?;
            ensure!(
                b.position.iter().chain(&b.size).all(|v| v.is_finite()) && b.size[1] > 0.,
                "Invalid player placement"
            );
            removal.push(p.box_index);
        }
        if removal.is_empty() {
            return Ok(());
        }
        ensure!(
            frame.meshes.resident_vertex_count() == 0,
            "This character stage cannot replace a world's authored mesh layer"
        );
        frame.meshes.materials.truncate(1);
        if frame.meshes.materials.is_empty() {
            frame.meshes.materials.push(Material::default());
        }
        let selected_offset = frame.meshes.materials.len() as u32 - 1;
        frame
            .meshes
            .materials
            .extend(self.selected.materials.iter().skip(1).cloned());
        let traveler_offset = frame.meshes.materials.len() as u32 - 1;
        if let Some(traveler) = &self.traveler {
            frame
                .meshes
                .materials
                .extend(traveler.materials.iter().skip(1).cloned());
        }
        for p in &frame.player_primitives {
            let b = &frame.boxes[p.box_index];
            let center = Vec3::from_array(b.position);
            let heading = self.headings.entry(p.actor).or_insert(Heading {
                life: p.life,
                center,
                yaw: 0.,
            });
            if heading.life != p.life {
                *heading = Heading {
                    life: p.life,
                    center,
                    yaw: 0.,
                };
            }
            let delta = center - heading.center;
            if delta.x * delta.x + delta.z * delta.z > 0.000001 {
                heading.yaw = (-delta.x).atan2(-delta.z);
            }
            heading.center = center;
            let selected_model = local == Some(p.actor);
            let (model, offset) = if selected_model || self.traveler.is_none() {
                (&self.selected, selected_offset)
            } else {
                (self.traveler.as_ref().unwrap(), traveler_offset)
            };
            let scale = b.size[1] / model.height;
            let foot_y = center.y - b.size[1] * 0.5;
            let anchor = Vec3::new(center.x, foot_y - model.min_y * scale, center.z);
            let rotation = Quat::from_rotation_y(heading.yaw);
            frame.meshes.vertices.extend(model.vertices.iter().map(|v| {
                let mut out = *v;
                out.position =
                    (anchor + rotation * (Vec3::from_array(v.position) * scale)).to_array();
                out.normal = (rotation * Vec3::from_array(v.normal)).to_array();
                if v.material != 0 {
                    out.material += offset;
                }
                out
            }));
            frame.meshes.drawn_avatars.push(p.actor);
            self.placements.push(Placement {
                actor: p.actor,
                life: p.life,
                center: b.position,
                foot_y,
                selected_model,
            });
            if selected_model {
                frame.camera.position = (center + Vec3::new(2., 1.4, -3.5)).to_array();
                frame.camera.target = (center + Vec3::new(0., 0.1, 0.)).to_array();
                frame.camera.fov_degrees = 48.;
            }
        }
        self.headings
            .retain(|actor, _| self.placements.iter().any(|p| p.actor == *actor));
        // Remove only explicitly identified Player primitives. Fixed scenery,
        // dynamic objects, projectiles and colliders are unaffected.
        removal.sort_unstable();
        for index in removal.into_iter().rev() {
            frame.boxes.remove(index);
        }
        frame.player_primitives.clear();
        Ok(())
    }
}
