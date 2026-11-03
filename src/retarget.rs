//! Author-time rest-relative retarget. Produces ordinary native clips, never runtime rig aliases.
use crate::{
    meshes::{Asset, Clip, Keyframe, MAX_BONES, Track},
    model::{Result, invalid},
};
use glam::{Quat, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub source: String,
    pub target: String,
    /// Changes axes of the source rest-relative delta into target local axes.
    #[serde(default = "identity")]
    pub basis: [f32; 4],
    /// Zero preserves target rest lengths. Explicitly enable for root travel/stretch.
    #[serde(default)]
    pub translation_scale: f32,
}
fn identity() -> [f32; 4] {
    [0., 0., 0., 1.]
}

/// Both assets must be valid. Missing animated mappings and incompatible parent
/// relations are errors. Unmapped target branches remain in their native rest pose.
pub fn bake(
    source: &Asset,
    target: &Asset,
    clip: &str,
    output: String,
    mappings: &[Mapping],
) -> Result<Clip> {
    source.validate()?;
    target.validate()?;
    let sr = source
        .rig
        .as_ref()
        .ok_or_else(|| invalid("Retarget source requires rig"))?;
    let tr = target
        .rig
        .as_ref()
        .ok_or_else(|| invalid("Retarget target requires rig"))?;
    let clip = source
        .clips
        .iter()
        .find(|c| c.id == clip)
        .ok_or_else(|| invalid("Unknown retarget source clip"))?;
    if mappings.is_empty() || mappings.len() > MAX_BONES {
        return Err(invalid("Retarget requires1..256 bone mappings"));
    }
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut resolved = Vec::with_capacity(mappings.len());
    for m in mappings {
        let si = sr
            .bones
            .iter()
            .position(|b| b.name == m.source)
            .ok_or_else(|| invalid(format!("Unknown source bone:{}", m.source)))?;
        let ti = tr
            .bones
            .iter()
            .position(|b| b.name == m.target)
            .ok_or_else(|| invalid(format!("Unknown target bone:{}", m.target)))?;
        let basis = Quat::from_array(m.basis);
        if !sources.insert(si)
            || !targets.insert(ti)
            || !basis.is_finite()
            || (basis.length_squared() - 1.).abs() > 0.001
            || !m.translation_scale.is_finite()
            || !(0.0..=100.).contains(&m.translation_scale)
        {
            return Err(invalid(
                "Retarget needs unique bones, unit basis and translation_scale0..100",
            ));
        }
        resolved.push((si, ti, m));
    }
    for &(si, ti, _) in &resolved {
        let sp = sr.bones[si].parent.map(usize::from);
        let tp = tr.bones[ti].parent.map(usize::from);
        if sp.is_some() != tp.is_some()
            || sp
                .zip(tp)
                .is_some_and(|(s, t)| !resolved.iter().any(|(a, b, _)| *a == s && *b == t))
        {
            return Err(invalid(
                "Retarget requires corresponding direct parents; map ancestors or bake hierarchy conversion explicitly",
            ));
        }
    }
    let mut tracks = Vec::with_capacity(clip.tracks.len());
    for track in &clip.tracks {
        let &(si, ti, m) = resolved
            .iter()
            .find(|(si, _, _)| *si == track.bone as usize)
            .ok_or_else(|| invalid("Retarget cannot silently discard an animated source bone"))?;
        let source_rest = Quat::from_array(sr.bones[si].rotation).normalize();
        let target_rest = Quat::from_array(tr.bones[ti].rotation).normalize();
        let basis = Quat::from_array(m.basis).normalize();
        let keys = track
            .keys
            .iter()
            .map(|k| {
                let delta = source_rest.inverse() * Quat::from_array(k.rotation).normalize();
                Keyframe {
                    tick: k.tick,
                    rotation: (target_rest * basis * delta * basis.inverse())
                        .normalize()
                        .to_array(),
                    translation: (Vec3::from(tr.bones[ti].translation)
                        + basis
                            * (Vec3::from(k.translation) - Vec3::from(sr.bones[si].translation))
                            * m.translation_scale)
                        .to_array(),
                }
            })
            .collect();
        tracks.push(Track {
            bone: ti as u8,
            keys,
        });
    }
    tracks.sort_unstable_by_key(|t| t.bone);
    let clip = Clip {
        id: output,
        duration_ticks: clip.duration_ticks,
        tracks,
    };
    // Validate the full result before the staged document is changed.
    let mut candidate = target.clone();
    candidate.clips = vec![std::sync::Arc::new(clip.clone())];
    candidate.validate()?;
    Ok(clip)
}
