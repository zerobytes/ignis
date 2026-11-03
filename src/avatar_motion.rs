//! Optional steered planar root translation. The KCC remains the only owner of
//! the body, including collision correction, gravity, steps and arrival clamps.
use crate::{
    identity::Reference,
    meshes::Asset,
    model::{Result, invalid},
    npc_presentation::{Config, Phase},
};
use glam::Vec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Root {
    pub bone: u8,
    pub distance_scale: f32,
    /// Unit forward axis of root translation in asset-space XZ. Navigation steers that axis.
    pub forward_axis: [f32; 2],
}
impl Root {
    pub fn validate(&self, asset: &Asset, config: &Config) -> Result<()> {
        asset.validate()?;
        let rig = asset
            .rig
            .as_ref()
            .ok_or_else(|| invalid("Root motion requires rig"))?;
        let axis = Vec2::from(self.forward_axis);
        if self.bone as usize >= rig.bones.len()
            || rig.bones[self.bone as usize].parent.is_some()
            || !self.distance_scale.is_finite()
            || !(0.001..=100.).contains(&self.distance_scale)
            || !axis.is_finite()
            || (axis.length_squared() - 1.).abs() > 0.001
        {
            return Err(invalid(
                "Root motion requires root bone, scale0.001..100 and unit planar forward",
            ));
        }
        let variants: Vec<_> = config
            .variants
            .iter()
            .filter(|v| v.phase == Phase::Walk)
            .collect();
        if variants.is_empty() {
            return Err(invalid("Root motion requires walk variants"));
        }
        for v in variants {
            let clip = v
                .clip
                .as_ref()
                .and_then(|id| asset.clips.iter().find(|c| c.id == *id))
                .ok_or_else(|| invalid("Root walk requires a clip"))?;
            let track = clip
                .tracks
                .iter()
                .find(|t| t.bone == self.bone)
                .ok_or_else(|| invalid("Root walk clip requires a root track"))?;
            if !v.looping
                || track.keys.first().unwrap().tick != 0
                || track.keys.last().unwrap().tick != clip.duration_ticks
                || track.keys.windows(2).any(|k| {
                    let d = Vec2::new(
                        k[1].translation[0] - k[0].translation[0],
                        k[1].translation[2] - k[0].translation[2],
                    );
                    d.dot(axis) < -0.00001 || (d - axis * d.dot(axis)).length() > 0.0001
                })
            {
                return Err(invalid(
                    "Steered root locomotion needs looping, complete, monotone planar forward travel",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    pub instance: Reference,
    pub life: u64,
    pub clip: u8,
    pub started: u64,
    pub last_tick: u64,
    pub intended: [f32; 2],
    pub command: [f32; 2],
}
/// One interval only, including exact loop seam. Never reads a future physical state.
pub(crate) fn velocity(
    root: Root,
    asset: &Asset,
    clip: u8,
    elapsed: u64,
    intended: Vec2,
    scale: f32,
) -> Vec2 {
    let clip = &asset.clips[clip as usize];
    let track = clip
        .tracks
        .iter()
        .find(|t| t.bone == root.bone)
        .expect("validated root track");
    let from = (elapsed % u64::from(clip.duration_ticks)) as u32;
    let a = crate::meshes::sample(track, from).w_axis;
    let b = crate::meshes::sample(track, from + 1).w_axis;
    let distance = Vec2::new(b.x - a.x, b.z - a.z).dot(Vec2::from(root.forward_axis));
    let speed = (distance * root.distance_scale * scale / crate::model::FIXED_DT)
        .max(0.)
        .min(intended.length());
    intended.normalize_or_zero() * speed
}

/// Used by checkpoint adapters before publishing a candidate. Source files of
/// those adapters are owned by the save front; this invariant stays with motion.
pub(crate) fn validate_saved(
    motions: &std::collections::BTreeMap<String, Motion>,
    scene: &crate::model::Scene,
    actors: &crate::targeting::Catalog,
    lives: &[u64],
    tick: u64,
) -> Result<()> {
    if motions.len() > crate::targeting::MAX_ACTORS {
        return Err(invalid("Root motion state exceeds actor budget"));
    }
    for (id, m) in motions {
        let slot = actors
            .actor_slot(m.instance)
            .ok_or_else(|| invalid("Root motion owner is absent"))?;
        let e = &scene.entities[actors.entity(slot as u16)];
        let config = e
            .avatar
            .as_ref()
            .ok_or_else(|| invalid("Root motion owner has no avatar"))?;
        let asset = scene
            .meshes
            .assets
            .iter()
            .find(|a| a.id == config.asset)
            .ok_or_else(|| invalid("Root asset is absent"))?;
        let clip = asset
            .clips
            .get(m.clip as usize)
            .ok_or_else(|| invalid("Root motion clip is absent"))?;
        if e.id != *id
            || e.character_controller.is_none()
            || config.root_motion.is_none()
            || lives.get(slot) != Some(&m.life)
            || m.started > m.last_tick
            || m.last_tick > tick
            || !config
                .variants
                .iter()
                .any(|v| v.phase == Phase::Walk && v.clip.as_ref() == Some(&clip.id))
            || m.intended
                .iter()
                .chain(&m.command)
                .any(|x| !x.is_finite() || x.abs() > 10000.)
            || Vec2::from(m.command).length() > Vec2::from(m.intended).length() + 0.001
        {
            return Err(invalid(
                "Invalid root clock, identity/life, clip or bounded movement",
            ));
        }
    }
    Ok(())
}
