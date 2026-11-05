//! Editable native motion libraries; independent from world/simulation history.
use crate::{
    avatar_ik,
    meshes::{Asset, Bone, Clip, Geometry, Rig, Weight},
    model::{Result, invalid},
};
use glam::{Mat4, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};
pub const MAX_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Foot {
    pub name: String,
    pub bones: [String; 3],
    /// Normalized cycle interval; start>end wraps through phase zero.
    pub stance: [f32; 2],
    pub pole: [f32; 3],
    pub normal_axis: [f32; 3],
    pub sole_offset: f32,
    pub max_correction: f32,
}
impl Foot {
    pub fn active(&self, phase: f32) -> bool {
        let [a, b] = self.stance;
        if a <= b {
            phase >= a && phase < b
        } else {
            phase >= a || phase < b
        }
    }
    pub fn chain(&self, rig: &Rig) -> Result<avatar_ik::Chain> {
        let indices: Vec<_> = self
            .bones
            .iter()
            .map(|name| {
                rig.bones
                    .iter()
                    .position(|b| &b.name == name)
                    .ok_or_else(|| invalid(format!("Unknown contact joint {name}")))
            })
            .collect::<Result<_>>()?;
        let c = avatar_ik::Chain {
            bones: [indices[0] as u8, indices[1] as u8, indices[2] as u8],
            pole: self.pole,
            normal_axis: self.normal_axis,
            probe_up: 0.25,
            probe_down: 0.5,
            max_correction: self.max_correction,
            support_radius: 0.,
            support_tolerance: 0.03,
            sole_offset: self.sole_offset,
            max_slope_degrees: 45.,
        };
        c.validate_ancestry(rig)?;
        Ok(c)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    pub name: String,
    pub clip: String,
    /// Distance per full cycle, in the reference rig's source metres. Zero=idle.
    pub stride: f32,
    pub feet: Vec<Foot>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u16,
    #[serde(default)]
    pub floor_y: f32,
    pub bones: Vec<Bone>,
    pub clips: Vec<Arc<Clip>>,
    pub motions: Vec<Motion>,
}
pub fn pose(clip: Option<u8>, tick: u32) -> crate::npc_presentation::Pose {
    crate::npc_presentation::Pose {
        actor: 1,
        life: 0,
        asset: 0,
        clip,
        clip_tick: tick,
        previous_clip: None,
        previous_tick: 0,
        blend: 10000,
        yaw: 0,
        pitch: 0,
        scale: 1.,
        offset: [0.; 3],
        hide_body: false,
        orientation: [0., 0., 0., 1.],
        strip_root: None,
        joint_count: 0,
        ik_limited: 0,
        joints: [Default::default(); avatar_ik::JOINTS],
    }
}
fn skeleton_asset(bones: Vec<Bone>, clips: Vec<Arc<Clip>>) -> Asset {
    Asset {
        id: "motion_reference".into(),
        geometry: Arc::new(Geometry {
            positions: vec![[0.; 3], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2]],
            material_coordinates: None,
        }),
        shading: Default::default(),
        material: Default::default(),
        rig: Some(Arc::new(Rig {
            bones,
            weights: vec![
                Weight {
                    joints: [0; 4],
                    weights: [1., 0., 0., 0.],
                    extra: None
                };
                3
            ],
        })),
        clips,
    }
}
impl Document {
    pub fn asset(&self) -> Asset {
        skeleton_asset(self.bones.clone(), self.clips.clone())
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !self.floor_y.is_finite()
            || self.floor_y.abs() > 10000.
            || self.motions.is_empty()
            || self.motions.len() > 32
            || serde_json::to_vec(self)
                .map_err(|e| invalid(e.to_string()))?
                .len()
                > MAX_BYTES
        {
            return Err(invalid("Motion library version/count/byte limit exceeded"));
        }
        let asset = self.asset();
        crate::meshes::Document {
            assets: vec![asset.clone()],
            ..Default::default()
        }
        .validate()?;
        // Locomotion remains in-place in X/Z; the physical controller owns travel.
        // Root Y translation is retained for authored weight shift and bounce.
        for clip in &self.clips {
            for track in &clip.tracks {
                let bone = &self.bones[track.bone as usize];
                if bone.parent.is_none()
                    && track.keys.iter().any(|k| {
                        (k.translation[0] - bone.translation[0]).abs() > 0.00001
                            || (k.translation[2] - bone.translation[2]).abs() > 0.00001
                    })
                {
                    return Err(invalid(
                        "Motion libraries require in-place root X/Z translation",
                    ));
                }
            }
        }
        let mut names = BTreeSet::new();
        for m in &self.motions {
            if m.name.is_empty()
                || m.name.len() > 64
                || !names.insert(&m.name)
                || !self.clips.iter().any(|c| c.id == m.clip)
                || !m.stride.is_finite()
                || !(0.0..=100.).contains(&m.stride)
                || m.feet.len() > avatar_ik::CHAINS
            {
                return Err(invalid("Invalid motion name/clip/stride/contact count"));
            }
            let mut joints = BTreeSet::new();
            let mut feet = BTreeSet::new();
            for f in &m.feet {
                if f.name.is_empty()
                    || f.name.len() > 64
                    || !feet.insert(&f.name)
                    || f.stance
                        .iter()
                        .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
                    || f.stance[0] == f.stance[1]
                {
                    return Err(invalid("Invalid foot name or stance interval"));
                }
                for b in f.chain(asset.rig.as_ref().unwrap())?.bones {
                    if !joints.insert(b) {
                        return Err(invalid("Foot contacts must not share joints"));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn motion(&self, name: &str) -> Result<&Motion> {
        self.motions
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| invalid(format!("Unknown motion {name}")))
    }
    pub fn sample(
        &self,
        name: &str,
        phase: f32,
        targets: &[avatar_ik::Target],
    ) -> Result<serde_json::Value> {
        if !phase.is_finite() || !(0.0..=1.).contains(&phase) {
            return Err(invalid("Phase must be 0..1"));
        }
        let m = self.motion(name)?;
        let index = self.clips.iter().position(|c| c.id == m.clip).unwrap();
        let c = &self.clips[index];
        let a = self.asset();
        let p = pose(
            Some(index as u8),
            (phase * c.duration_ticks as f32).floor() as u32 % c.duration_ticks,
        );
        let solved = avatar_ik::constrain_pose(&a, &p, targets)?;
        let joints = avatar_ik::pose_joints(&a, &solved)?;
        let feet=m.feet.iter().map(|f| {
            let chain=f.chain(a.rig.as_ref().unwrap())?;
            let matrix=Mat4::from_cols_array_2d(&joints[chain.bones[2] as usize]);
            let point=matrix.transform_point3(Vec3::ZERO);
            Ok(serde_json::json!({"name":f.name,"stance":f.active(phase.rem_euclid(1.)),"point":point.to_array(),
                "normal":matrix.transform_vector3(f.normal_axis.into()).normalize().to_array(),
                "floor_gap":point.y-f.sole_offset-self.floor_y}))
        }).collect::<Result<Vec<_>>>()?;
        Ok(
            serde_json::json!({"motion":name,"phase":phase,"pose":solved,
            "bones":self.bones.iter().zip(&joints).map(|(b,j)|serde_json::json!({"name":b.name,"matrix":j})).collect::<Vec<_>>(),"feet":feet}),
        )
    }

    pub fn calibrate(&self, name: &str) -> Result<f32> {
        let m = self.motion(name)?;
        let a = self.asset();
        let rig = a.rig.as_ref().unwrap();
        let index = self.clips.iter().position(|c| c.id == m.clip).unwrap();
        let c = &self.clips[index];
        let mut distances = Vec::new();
        for f in &m.feet {
            let tip = f.chain(rig)?.bones[2] as usize;
            let point = |phase: f32| -> Result<Vec3> {
                let tick = (phase * c.duration_ticks as f32).floor() as u32 % c.duration_ticks;
                let joints = avatar_ik::pose_joints(&a, &pose(Some(index as u8), tick))?;
                Ok(Mat4::from_cols_array_2d(&joints[tip]).transform_point3(Vec3::ZERO))
            };
            let span = (f.stance[1] - f.stance[0]).rem_euclid(1.);
            if span < 0.05 {
                continue;
            }
            let d = (point(f.stance[1])? - point(f.stance[0])?).z / span;
            if d.is_finite() && d > 0.01 {
                distances.push(d);
            }
        }
        if distances.is_empty() {
            return Err(invalid(
                "No forward stance travel to calibrate; edit contacts/clip first",
            ));
        }
        Ok(distances.iter().sum::<f32>() / distances.len() as f32)
    }
    pub fn clips_for(&self, rig: &Rig, names: &[&str]) -> Result<Vec<Arc<Clip>>> {
        let source = self.asset();
        let target = skeleton_asset(rig.bones.clone(), vec![]);
        let mappings = self
            .bones
            .iter()
            .map(|b| crate::retarget::Mapping {
                source: b.name.clone(),
                target: b.name.clone(),
                basis: [0., 0., 0., 1.],
                translation_scale: if b.parent.is_none() { 1. } else { 0. },
            })
            .collect::<Vec<_>>();
        names
            .iter()
            .map(|name| {
                let m = self.motion(name)?;
                Ok(Arc::new(crate::retarget::bake(
                    &source,
                    &target,
                    &m.clip,
                    m.clip.clone(),
                    &mappings,
                )?))
            })
            .collect()
    }
    /// Bake world-stationary supports for a straight in-place cycle. The virtual
    /// actor advances along -Z by stride per cycle; no gameplay body is moved.
    pub fn bake_supports(&self, name: &str, samples: u16) -> Result<Self> {
        if !(4..=240).contains(&samples) {
            return Err(invalid("Support bake requires 4..240 samples"));
        }
        let motion = self.motion(name)?;
        if motion.feet.is_empty() {
            return Err(invalid("Support bake requires contact definitions"));
        }
        let asset = self.asset();
        let rig = asset.rig.as_ref().unwrap();
        let index = self.clips.iter().position(|c| c.id == motion.clip).unwrap();
        let mut output = (*self.clips[index]).clone();
        let original = &self.clips[index];
        let mut anchors = Vec::new();
        for f in &motion.feet {
            let chain = f.chain(rig)?;
            let tick =
                (f.stance[0] * original.duration_ticks as f32) as u32 % original.duration_ticks;
            let joints = avatar_ik::pose_joints(&asset, &pose(Some(index as u8), tick))?;
            let mut point = Mat4::from_cols_array_2d(&joints[chain.bones[2] as usize])
                .transform_point3(Vec3::ZERO);
            point.y = self.floor_y + f.sole_offset;
            anchors.push((chain, point));
        }
        let count = u32::from(samples).min(original.duration_ticks);
        for sample in 0..=count {
            let tick = sample * original.duration_ticks / count;
            let phase = tick as f32 / original.duration_ticks as f32;
            let evaluation_tick = tick % original.duration_ticks;
            let mut targets = Vec::new();
            for (foot, (chain, start)) in motion.feet.iter().zip(&anchors) {
                let phase = phase.rem_euclid(1.);
                if !foot.active(phase) {
                    continue;
                }
                let elapsed = (phase - foot.stance[0]).rem_euclid(1.);
                let span = if foot.stance == [0., 1.] {
                    1.
                } else {
                    (foot.stance[1] - foot.stance[0]).rem_euclid(1.)
                };
                let edge = (elapsed.min(span - elapsed) / 0.04).clamp(0., 1.);
                targets.push(avatar_ik::Target {
                    chain: chain.clone(),
                    point: (*start + Vec3::Z * motion.stride * elapsed).to_array(),
                    normal: [0., 1., 0.],
                    weight: edge * edge * (3. - 2. * edge),
                });
            }
            let corrected = avatar_ik::constrain_pose(
                &asset,
                &pose(Some(index as u8), evaluation_tick),
                &targets,
            )?;
            let local = crate::meshes::avatar_locals(&asset, &corrected);
            for bone in anchors.iter().flat_map(|(chain, _)| chain.bones) {
                let (_, rotation, translation) =
                    local[bone as usize].to_scale_rotation_translation();
                let joint = avatar_ik::Joint {
                    bone,
                    rotation: rotation.to_array(),
                    translation: translation.to_array(),
                };
                let slot = output
                    .tracks
                    .iter()
                    .position(|t| t.bone == joint.bone)
                    .unwrap_or_else(|| {
                        output.tracks.push(crate::meshes::Track {
                            bone: joint.bone,
                            keys: vec![],
                        });
                        output.tracks.len() - 1
                    });
                let track = &mut output.tracks[slot];
                track.keys.retain(|k| k.tick != tick);
                track.keys.push(crate::meshes::Keyframe {
                    tick,
                    translation: joint.translation,
                    rotation: joint.rotation,
                });
                track.keys.sort_by_key(|k| k.tick);
            }
        }
        output.tracks.sort_by_key(|t| t.bone);
        let mut candidate = self.clone();
        candidate.clips[index] = Arc::new(output);
        candidate.validate()?;
        Ok(candidate)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Inspect,
    Replace {
        document: Box<Document>,
    },
    PutClip {
        clip: Arc<Clip>,
    },
    SetMotion {
        motion: Motion,
    },
    ImportAsset {
        asset: String,
        motions: Vec<Motion>,
    },
    ExportClips {
        asset: String,
    },
    Sample {
        motion: String,
        phase: f32,
        #[serde(default)]
        targets: Vec<avatar_ik::Target>,
    },
    CalibrateStride {
        motion: String,
    },
    BakeSupports {
        motion: String,
        samples: u16,
    },
    Save {
        path: String,
    },
    Load {
        path: String,
    },
    Undo,
    Redo,
}
#[derive(Clone, Default)]
pub struct State {
    document: Option<Arc<Document>>,
    revision: u64,
    undo: Vec<Option<Arc<Document>>>,
    redo: Vec<Option<Arc<Document>>>,
}
impl State {
    pub fn document(&self) -> Result<&Document> {
        self.document
            .as_deref()
            .ok_or_else(|| invalid("No motion library loaded"))
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn guard(&self, r: Option<u64>) -> Result<()> {
        if r.is_some_and(|r| r != self.revision) {
            Err(invalid("Stale animation revision"))
        } else {
            Ok(())
        }
    }
    pub fn replace(&mut self, d: Document) -> Result<()> {
        d.validate()?;
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Animation revision overflow"))?;
        self.undo.push(self.document.replace(Arc::new(d)));
        if self.undo.len() > 16 {
            self.undo.remove(0);
        }
        self.redo.clear();
        Ok(())
    }
    pub fn history(&mut self, redo: bool) -> Result<()> {
        let (from, to) = if redo {
            (&mut self.redo, &mut self.undo)
        } else {
            (&mut self.undo, &mut self.redo)
        };
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Animation revision overflow"))?;
        let next = from.pop().ok_or_else(|| invalid("No animation history"))?;
        self.revision = revision;
        to.push(std::mem::replace(&mut self.document, next));
        Ok(())
    }
}
