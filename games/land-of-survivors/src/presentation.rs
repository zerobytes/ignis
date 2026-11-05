//! Game appearance and locomotion use the native rig player over public physical poses.
use crate::{animation::Gait, controls::Controls};
use anyhow::{Result, ensure};
use glam::{Quat, Vec3};
use ignis::{
    materials::Material,
    meshes::{Asset, Document, Frame, Geometry, Rig, Shading, Weight},
    model::MovementMode,
    npc_presentation::Pose,
    render::{RenderFrame, Vertex},
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, PartialEq, Eq)]
struct PoseKey {
    clip: Option<u8>,
    tick: u32,
    previous: Option<u8>,
    previous_tick: u32,
    blend: u16,
}
struct Model {
    document: Document,
    styles: Vec<Vertex>,
    materials: Vec<Material>,
    height: f32,
    floor: f32,
    durations: [u32; 3],
    strides: [f32; 3],
    evaluated: Frame,
    pose_key: Option<PoseKey>,
}
fn pose(actor: u64, life: u64) -> Pose {
    Pose {
        actor,
        life,
        asset: 0,
        clip: None,
        clip_tick: 0,
        previous_clip: None,
        previous_tick: 0,
        blend: 10000,
        yaw: 0,
        pitch: 0,
        scale: 1.,
        offset: [0.; 3],
        hide_body: true,
        orientation: [0., 0., 0., 1.],
        strip_root: None,
        joint_count: 0,
        ik_limited: 0,
        joints: [Default::default(); ignis::avatar_ik::JOINTS],
    }
}
impl Model {
    fn new(source: Document, library: &ignis::animation::Document) -> Result<Self> {
        source.validate()?;
        let body_instance = source
            .instances
            .iter()
            .find(|i| i.id.ends_with("_body"))
            .ok_or_else(|| anyhow::anyhow!("Native character body is missing"))?;
        let body = source.asset(&body_instance.asset)?;
        let rig = body
            .rig
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Character has no rig"))?;
        let head =
            rig.bones
                .iter()
                .position(|b| b.name == "head")
                .ok_or_else(|| anyhow::anyhow!("Character has no head joint"))? as u8;
        let mut original = Frame::default();
        original.set_unique_static_residency(false);
        original.update(&source, 0);
        ensure!(
            !original.vertices.is_empty(),
            "Character model has no triangles"
        );
        let min_y = original
            .vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = original
            .vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        ensure!(max_y - min_y > 0.01, "Character model has no height");
        let rests = source.poses(0);
        let mut positions = Vec::new();
        let mut triangles = Vec::new();
        let mut weights = Vec::new();
        for instance in &source.instances {
            ensure!(
                instance.position == [0.; 3]
                    && instance.rotation == [0., 0., 0., 1.]
                    && instance.scale == 1.
                    && instance.attachment.is_none(),
                "Session character requires a canonical native placement"
            );
            let asset = source.asset(&instance.asset)?;
            ensure!(
                asset.shading == Shading::Smooth,
                "Session character requires native smooth parts"
            );
            let offset = positions.len() as u32;
            if instance.id == body_instance.id {
                positions.extend_from_slice(&asset.geometry.positions);
                weights.extend(rig.weights.iter().cloned());
            } else {
                // Rigid eye inserts follow the same posed head as the skin. This
                // keeps all components aligned during native clip cross-fades.
                let rest = rests.iter().find(|p| p.instance == instance.id).unwrap();
                positions.extend_from_slice(&rest.positions);
                weights.extend(rest.positions.iter().map(|_| Weight {
                    joints: [head, 0, 0, 0],
                    weights: [1., 0., 0., 0.],
                    extra: None,
                }));
            }
            triangles.extend(
                asset
                    .geometry
                    .triangles
                    .iter()
                    .map(|t| t.map(|i| i + offset)),
            );
        }
        let asset = Asset {
            id: "session_character".into(),
            geometry: Arc::new(Geometry {
                positions,
                triangles,
                material_coordinates: None,
            }),
            shading: Shading::Smooth,
            material: body.material.clone(),
            rig: Some(Arc::new(Rig {
                bones: rig.bones.clone(),
                weights,
            })),
            clips: library.clips_for(rig, &["idle", "walk", "run"])?,
        };
        let document = Document {
            assets: vec![asset],
            ..Default::default()
        };
        document.validate()?;
        // A native rest-pose oracle verifies the render-only packing preserved
        // geometry and every body influence before any animation is selected.
        let mut oracle = Frame::default();
        oracle.update(&document, 0);
        oracle.append_avatars(&document, &[pose(1, 0)], |_| Some(([0.; 3], [1.; 3])))?;
        ensure!(
            oracle.vertices.len() == original.vertices.len(),
            "Character component order changed"
        );
        ensure!(
            oracle
                .vertices
                .iter()
                .zip(&original.vertices)
                .all(
                    |(a, b)| Vec3::from_array(a.position).distance(Vec3::from_array(b.position))
                        < 0.00001
                ),
            "Native character packing changed its rest shape"
        );
        let durations = std::array::from_fn(|i| document.assets[0].clips[i].duration_ticks);
        let strides = ["idle", "walk", "run"].map(|n| library.motion(n).unwrap().stride);
        Ok(Self {
            floor: min_y,
            durations,
            strides,
            document,
            styles: original.vertices,
            materials: original.materials,
            height: max_y - min_y,
            evaluated: oracle,
            pose_key: None,
        })
    }
    fn append(
        &mut self,
        frame: &mut RenderFrame,
        center: [f32; 3],
        height: f32,
        p: Pose,
        material_offset: u32,
    ) -> Result<f32> {
        // Native rig playback owns evaluation and material projection caches.
        // Cache only the local pose, so idle travelers can share that result.
        let key = PoseKey {
            clip: p.clip,
            tick: p.clip_tick,
            previous: p.previous_clip,
            previous_tick: p.previous_tick,
            blend: p.blend,
        };
        if self.pose_key != Some(key) {
            self.evaluated.update(&self.document, 0);
            let mut local = pose(1, 0);
            local.clip = p.clip;
            local.clip_tick = p.clip_tick;
            local.previous_clip = p.previous_clip;
            local.previous_tick = p.previous_tick;
            local.blend = p.blend;
            self.evaluated
                .append_avatars(&self.document, &[local], |_| Some(([0.; 3], [1.; 3])))?;
            self.pose_key = Some(key);
        }
        let tail = &self.evaluated.vertices;
        ensure!(
            tail.len() == self.styles.len(),
            "Animated character topology changed"
        );
        let scale = height / self.height;
        let foot_y = center[1] - height * 0.5;
        let sole = self.floor;
        let rotation = Quat::from_rotation_y(-f32::from(p.yaw) * 0.01f32.to_radians());
        let anchor = Vec3::new(center[0], foot_y - sole * scale, center[2]);
        frame
            .meshes
            .vertices
            .extend(tail.iter().zip(&self.styles).map(|(v, style)| {
                let mut out = *v;
                out.position =
                    (anchor + rotation * (Vec3::from_array(v.position) * scale)).to_array();
                out.normal = (rotation * Vec3::from_array(v.normal)).to_array();
                out.color = style.color;
                out.material = if style.material == 0 {
                    0
                } else {
                    style.material + material_offset
                };
                out.rest_position = style.rest_position;
                out.material_projection_weights = style.material_projection_weights;
                out
            }));
        frame.meshes.drawn_avatars.push(p.actor);
        Ok(foot_y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub actor: u64,
    pub life: u64,
    pub center: [f32; 3],
    pub foot_y: f32,
    pub selected_model: bool,
    pub yaw_degrees: f32,
    pub gait: Gait,
    pub clip_tick: u32,
}
struct Motion {
    life: u64,
    center: Vec3,
    sampled: f64,
    last_moved: f64,
    yaw: f32,
    speed: f32,
    gait: Gait,
    phase: f64,
    previous: Option<(Gait, u32, f64)>,
}
impl Motion {
    fn new(life: u64, center: Vec3, clock: f64) -> Self {
        Self {
            life,
            center,
            sampled: clock,
            last_moved: clock - 1.,
            yaw: 0.,
            speed: 0.,
            gait: Gait::Idle,
            phase: 0.,
            previous: None,
        }
    }
    fn update(
        &mut self,
        center: Vec3,
        clock: f64,
        local: Option<(f32, MovementMode)>,
        durations: [u32; 3],
        strides: [f32; 3],
    ) -> Pose {
        let delta = center - self.center;
        let distance = delta.x.hypot(delta.z);
        let dt = clock - self.sampled;
        if distance > 0.00001 && dt > 0.000001 {
            self.speed = distance / dt as f32;
            self.last_moved = clock;
            self.yaw = delta.x.atan2(-delta.z).to_degrees().rem_euclid(360.);
        }
        if let Some((yaw, _)) = local {
            self.yaw = yaw;
        }
        let gait = if clock - self.last_moved >= 0.12 || self.speed < 0.05 {
            Gait::Idle
        } else if local.map_or(self.speed > 5., |(_, mode)| mode == MovementMode::Run) {
            Gait::Run
        } else {
            Gait::Walk
        };
        if gait != self.gait {
            self.previous = Some((
                self.gait,
                (self.phase * f64::from(durations[self.gait.index() as usize])) as u32,
                clock,
            ));
            self.gait = gait;
        }
        if gait == Gait::Idle {
            self.phase = (clock * 60. / f64::from(durations[gait.index() as usize])).fract();
        } else {
            self.phase =
                (self.phase + f64::from(distance / strides[gait.index() as usize])).fract();
        }
        self.center = center;
        self.sampled = clock;
        let mut p = pose(1, self.life);
        p.clip = Some(gait.index());
        p.clip_tick = (self.phase * f64::from(durations[gait.index() as usize])) as u32;
        p.yaw = (((self.yaw + 180.).rem_euclid(360.) - 180.) * 100.).round() as i16;
        if let Some((before, tick, started)) = self.previous {
            let elapsed = (clock - started).max(0.);
            p.previous_clip = Some(before.index());
            p.previous_tick = (f64::from(tick) + elapsed * 60.)
                .rem_euclid(f64::from(durations[before.index() as usize]))
                as u32;
            p.blend = (elapsed / 0.15 * 10000.).clamp(0., 10000.) as u16;
            if p.blend == 10000 {
                self.previous = None;
            }
        }
        p
    }
}

pub struct Presentation {
    selected: Model,
    traveler: Option<Model>,
    motions: BTreeMap<u64, Motion>,
    pub placements: Vec<Placement>,
}
impl Presentation {
    pub fn with_library(
        selected: Document,
        traveler: Option<Document>,
        library: &ignis::animation::Document,
    ) -> Result<Self> {
        crate::animation::validate_game(library)?;
        Ok(Self {
            selected: Model::new(selected, library)?,
            traveler: traveler.map(|d| Model::new(d, library)).transpose()?,
            motions: BTreeMap::new(),
            placements: Vec::new(),
        })
    }
    pub fn apply(
        &mut self,
        frame: &mut RenderFrame,
        local: Option<u64>,
        controls: &Controls,
        mode: MovementMode,
        clock: f64,
        remote: bool,
    ) -> Result<()> {
        self.placements.clear();
        let players = frame.player_primitives.clone();
        let mut removal = Vec::with_capacity(players.len());
        for p in &players {
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
        for player in &players {
            let b = frame.boxes[player.box_index];
            let center = Vec3::from_array(b.position);
            let motion = self
                .motions
                .entry(player.actor)
                .or_insert_with(|| Motion::new(player.life, center, clock));
            if motion.life != player.life || clock < motion.sampled {
                *motion = Motion::new(player.life, center, clock);
            }
            let selected_model = local == Some(player.actor);
            let (model, offset) = match (self.traveler.as_mut(), selected_model) {
                (Some(model), false) => (model, traveler_offset),
                _ => (&mut self.selected, selected_offset),
            };
            let mut p = motion.update(
                center,
                clock,
                selected_model.then_some((controls.heading_degrees, mode)),
                model.durations,
                model.strides.map(|s| s * b.size[1] / model.height),
            );
            p.actor = player.actor;
            let foot_y = model.append(frame, b.position, b.size[1], p, offset)?;
            self.placements.push(Placement {
                actor: player.actor,
                life: player.life,
                center: b.position,
                foot_y,
                selected_model,
                yaw_degrees: motion.yaw,
                gait: motion.gait,
                clip_tick: p.clip_tick,
            });
            if selected_model && remote {
                frame.camera = controls.view.unobstructed_camera(b.position, b.size[1])?;
            }
        }
        self.motions
            .retain(|id, _| self.placements.iter().any(|p| p.actor == *id));
        removal.sort_unstable();
        for index in removal.into_iter().rev() {
            frame.boxes.remove(index);
        }
        frame.player_primitives.clear();
        Ok(())
    }
}
