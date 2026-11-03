//! Public visual/audio projection. Never supplies NPC knowledge or physics.
use crate::{
    identity::Reference,
    model::{Result, Scene, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const VOICES: usize = 64;
pub const RATE: u32 = 24000;
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Phase {
    #[default]
    Idle,
    Walk,
    Air,
    Cast,
    Dead,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub phase: Phase,
    #[serde(default)]
    pub clip: Option<String>,
    #[serde(default)]
    pub sound: Option<String>,
    #[serde(default)]
    pub weights: [i16; 9],
    #[serde(default)]
    pub looping: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub asset: String,
    /// Calibrates rig forward/up to the engine's -Z/+Y convention.
    #[serde(default = "identity", skip_serializing_if = "is_identity")]
    pub orientation: [f32; 4],
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ik: Vec<crate::avatar_ik::Chain>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_motion: Option<crate::avatar_motion::Root>,
    #[serde(default = "one")]
    pub scale: f32,
    #[serde(default)]
    pub offset: [f32; 3],
    #[serde(default = "yes")]
    pub hide_body: bool,
    #[serde(default = "blend")]
    pub blend_ticks: u16,
    #[serde(default = "steps")]
    pub step_ticks: u16,
    #[serde(default)]
    pub variants: Vec<Variant>,
}
fn identity() -> [f32; 4] {
    [0., 0., 0., 1.]
}
fn is_identity(q: &[f32; 4]) -> bool {
    *q == identity()
}
fn one() -> f32 {
    1.
}
fn yes() -> bool {
    true
}
fn blend() -> u16 {
    8
}
fn steps() -> u16 {
    30
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sound {
    pub id: String,
    pub samples: Vec<i16>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Audio {
    #[serde(default)]
    pub sounds: Vec<Sound>,
}
impl Audio {
    pub fn is_empty(&self) -> bool {
        self.sounds.is_empty()
    }
}
pub fn validate(scene: &Scene) -> Result<()> {
    let audio = &scene.avatar_audio;
    if audio.sounds.len() > 16
        || audio.sounds.iter().map(|s| s.samples.len()).sum::<usize>() > RATE as usize * 20
    {
        return Err(invalid("Avatar PCM budget:16sounds/20seconds mono24kHz"));
    }
    let mut ids = std::collections::BTreeSet::new();
    for s in &audio.sounds {
        if s.id.is_empty()
            || s.id.len() > 64
            || s.samples.is_empty()
            || s.samples.len() > RATE as usize * 4
            || !ids.insert(&s.id)
        {
            return Err(invalid("Invalid/duplicate avatar sound"));
        }
    }
    for e in scene.entities.iter().chain(&scene.npc_templates) {
        let Some(c) = &e.avatar else { continue };
        if scene.version < 95 || !matches!(e.body, Some(crate::model::BodyKind::Npc)) {
            return Err(invalid("Avatar requires actor/scene95"));
        }
        let a = scene
            .meshes
            .assets
            .iter()
            .find(|a| a.id == c.asset)
            .ok_or_else(|| invalid("Unknown avatar asset"))?;
        if !c.scale.is_finite()
            || !(0.001..=100.).contains(&c.scale)
            || c.offset.iter().any(|x| !x.is_finite() || x.abs() > 100.)
            || c.blend_ticks > 120
            || c.step_ticks < 6
            || c.step_ticks > 600
            || c.variants.len() > 40
        {
            return Err(invalid(
                "Invalid avatar transform/blend/step/variant budget",
            ));
        }
        let orientation = glam::Quat::from_array(c.orientation);
        if !orientation.is_finite() || (orientation.length_squared() - 1.).abs() > 0.001 {
            return Err(invalid("Avatar orientation requires a unit quaternion"));
        }
        if (!is_identity(&c.orientation) || !c.ik.is_empty() || c.root_motion.is_some())
            && scene.version < 99
        {
            return Err(invalid(
                "Avatar calibration/IK/root motion requires scene99",
            ));
        }
        if c.ik.len() > crate::avatar_ik::CHAINS {
            return Err(invalid("Avatar IK supports up to8 chains"));
        }
        if !c.ik.is_empty() || c.root_motion.is_some() {
            a.validate()?;
            let rig = a
                .rig
                .as_ref()
                .ok_or_else(|| invalid("IK/root motion requires rig"))?;
            let mut used = std::collections::BTreeSet::new();
            for chain in &c.ik {
                chain.validate(rig)?;
                for bone in chain.bones {
                    if !used.insert(bone) {
                        return Err(invalid("IK chains must not share joints"));
                    }
                }
            }
            if let Some(root) = c.root_motion {
                if e.character_controller.is_none() || used.contains(&root.bone) {
                    return Err(invalid(
                        "Root motion requires KCC and a root outside IK chains",
                    ));
                }
                root.validate(a, c)?;
            }
        }
        for phase in [
            Phase::Idle,
            Phase::Walk,
            Phase::Air,
            Phase::Cast,
            Phase::Dead,
        ] {
            if c.variants.iter().filter(|v| v.phase == phase).count() > 8 {
                return Err(invalid("Maximum8 avatar variants per phase"));
            }
        }
        for v in &c.variants {
            if v.weights.iter().any(|x| x.unsigned_abs() > 1000)
                || v.clip
                    .as_ref()
                    .is_some_and(|s| !a.clips.iter().any(|x| x.id == *s))
                || v.sound
                    .as_ref()
                    .is_some_and(|s| !audio.sounds.iter().any(|x| x.id == *s))
            {
                return Err(invalid("Unknown avatar clip/sound or invalid trait weight"));
            }
        }
    }
    if scene.version < 95 && !audio.is_empty() {
        return Err(invalid("Avatar audio requires scene95"));
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub instance: Reference,
    pub life: u64,
    pub phase: Phase,
    pub activity: u64,
    pub variant: Option<u8>,
    pub started: u64,
    pub animation_started: u64,
    pub previous_clip: Option<u8>,
    pub previous_tick: u32,
    pub next_step: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Voice {
    pub instance: Reference,
    pub life: u64,
    pub sound: u8,
    pub started: u64,
    pub position: [f32; 3],
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StateSet {
    pub actors: BTreeMap<String, State>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub motion: BTreeMap<String, crate::avatar_motion::Motion>,
    pub voices: Vec<Voice>,
    pub dropped_voices: u64,
}
impl StateSet {
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty()
            && self.motion.is_empty()
            && self.voices.is_empty()
            && self.dropped_voices == 0
    }
}
/// Explicit public pose; no goals, cues, abilities, routines or authoring data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    pub actor: u64,
    pub life: u64,
    pub asset: u32,
    pub clip: Option<u8>,
    pub clip_tick: u32,
    pub previous_clip: Option<u8>,
    pub previous_tick: u32,
    pub blend: u16,
    pub yaw: i16,
    pub pitch: i16,
    pub scale: f32,
    pub offset: [f32; 3],
    pub hide_body: bool,
    pub orientation: [f32; 4],
    pub strip_root: Option<u8>,
    pub joint_count: u8,
    pub ik_limited: u8,
    pub joints: [crate::avatar_ik::Joint; crate::avatar_ik::JOINTS],
}
impl Pose {
    pub fn validate(&self) -> Result<()> {
        if self.actor == 0
            || self
                .clip
                .is_some_and(|x| x as usize >= crate::meshes::MAX_CLIPS)
            || self
                .previous_clip
                .is_some_and(|x| x as usize >= crate::meshes::MAX_CLIPS)
            || self.clip_tick > crate::meshes::MAX_TICK
            || self.previous_tick > crate::meshes::MAX_TICK
            || self.blend > 10000
            || self.yaw.unsigned_abs() > 18000
            || self.pitch.unsigned_abs() > 9000
            || !self.scale.is_finite()
            || !(0.001..=100.).contains(&self.scale)
            || self.offset.iter().any(|x| !x.is_finite() || x.abs() > 100.)
        {
            return Err(invalid("Invalid avatar public pose"));
        }
        let q = glam::Quat::from_array(self.orientation);
        if !q.is_finite()
            || (q.length_squared() - 1.).abs() > 0.001
            || self.joint_count as usize > crate::avatar_ik::JOINTS
            || self.ik_limited as usize > crate::avatar_ik::CHAINS
            || self
                .strip_root
                .is_some_and(|b| b as usize >= crate::meshes::MAX_BONES)
        {
            return Err(invalid("Invalid avatar calibration/IK budget/root"));
        }
        let mut used = 0u32;
        for j in &self.joints[..self.joint_count as usize] {
            let q = glam::Quat::from_array(j.rotation);
            if j.bone as usize >= crate::meshes::MAX_BONES
                || used & (1u32 << j.bone) != 0
                || !q.is_finite()
                || (q.length_squared() - 1.).abs() > 0.001
                || j.translation
                    .iter()
                    .any(|x| !x.is_finite() || x.abs() > 10000.)
            {
                return Err(invalid("Invalid avatar joint override"));
            }
            used |= 1u32 << j.bone;
        }
        Ok(())
    }
    pub fn validate_asset(&self, asset: &crate::meshes::Asset) -> Result<()> {
        self.validate()?;
        if self.clip.is_some_and(|i| i as usize >= asset.clips.len())
            || self
                .previous_clip
                .is_some_and(|i| i as usize >= asset.clips.len())
        {
            return Err(invalid("Avatar clip absent in visual content"));
        }
        let bones = asset.rig.as_ref().map_or(0, |r| r.bones.len());
        if self.strip_root.is_some_and(|b| {
            b as usize >= bones
                || asset.rig.as_ref().unwrap().bones[b as usize]
                    .parent
                    .is_some()
        }) || self.joints[..self.joint_count as usize]
            .iter()
            .any(|j| j.bone as usize >= bones)
        {
            return Err(invalid("Avatar joint/root absent in visual rig"));
        }
        Ok(())
    }
}
fn clock(
    asset: &crate::meshes::Asset,
    variant: Option<&Variant>,
    elapsed: u64,
) -> (Option<u8>, u32) {
    let Some(v) = variant else { return (None, 0) };
    let Some((i, c)) = v
        .clip
        .as_ref()
        .and_then(|id| asset.clips.iter().enumerate().find(|(_, c)| c.id == *id))
    else {
        return (None, 0);
    };
    (
        Some(i as u8),
        if v.looping {
            elapsed % u64::from(c.duration_ticks)
        } else {
            elapsed.min(u64::from(c.duration_ticks))
        } as u32,
    )
}
pub(crate) fn select_variant(
    c: &Config,
    phase: Phase,
    traits: [u16; 9],
    instance: Reference,
    life: u64,
) -> Option<u8> {
    c.variants
        .iter()
        .enumerate()
        .filter(|(_, v)| v.phase == phase)
        .max_by_key(|(i, v)| {
            let score = v
                .weights
                .iter()
                .zip(traits)
                .map(|(w, t)| i64::from(*w) * i64::from(t))
                .sum::<i64>();
            let tie = instance
                .instance
                .wrapping_mul(0x9e3779b97f4a7c15)
                .rotate_left(*i as u32)
                ^ life;
            (score, tie)
        })
        .map(|(i, _)| i as u8)
}
impl crate::physics::Simulation {
    pub(crate) fn update_presentation(&mut self) {
        let tick = self.tick;
        let mut next = std::mem::take(&mut self.presentation);
        let defs = self.definitions();
        next.actors
            .retain(|id, s| self.reference_for_presentation(id) == Some((s.instance, s.life)));
        next.motion
            .retain(|id, m| self.reference_for_presentation(id) == Some((m.instance, m.life)));
        next.voices.retain(|v| {
            self.actors
                .actor_slot(v.instance)
                .is_some_and(|a| self.vitality.state.lives[a] == v.life)
                && defs
                    .avatar_audio
                    .sounds
                    .get(v.sound as usize)
                    .is_some_and(|s| tick.saturating_sub(v.started) * 400 < s.samples.len() as u64)
        });
        for (id, npc) in &self.npcs {
            let Some(slot) = self.actor_slot_by_id(id) else {
                continue;
            };
            let e = &defs.entities[self.actors.entity(slot)];
            let Some(c) = &e.avatar else { continue };
            let Some((instance, life)) = self.reference_for_presentation(id) else {
                continue;
            };
            let slot = self.actors.actor_slot(instance).unwrap();
            let velocity = self.velocity_for_presentation(id);
            let phase = if self.vitality.dead(slot) {
                Phase::Dead
            } else if self.abilities.state.casts[slot].is_some() {
                Phase::Cast
            } else if self
                .character_controllers
                .get(id)
                .is_some_and(|c| c.airborne)
                || self.mobility.states.get(id).is_some_and(|s| {
                    matches!(
                        s.motor_mode,
                        Some(
                            crate::mobility::Mode::Fly
                                | crate::mobility::Mode::Jump
                                | crate::mobility::Mode::Glide
                        )
                    )
                })
            {
                Phase::Air
            } else if next
                .motion
                .get(id)
                .is_some_and(|m| m.last_tick.checked_add(1) == Some(tick))
                || velocity[0] * velocity[0] + velocity[2] * velocity[2] > 0.0025
            {
                Phase::Walk
            } else {
                Phase::Idle
            };
            let activity = self.abilities.state.casts[slot].map_or(0, |c| c.id);
            let changed = next.actors.get(id).is_none_or(|s| {
                s.instance != instance
                    || s.life != life
                    || s.phase != phase
                    || s.activity != activity
                    || (phase == Phase::Walk
                        && next
                            .motion
                            .get(id)
                            .is_some_and(|m| s.animation_started != m.started))
            });
            if changed {
                let previous = next
                    .actors
                    .get(id)
                    .filter(|s| s.instance == instance && s.life == life);
                let a = defs.meshes.assets.iter().find(|a| a.id == c.asset).unwrap();
                let (previous_clip, previous_tick) = previous.map_or((None, 0), |s| {
                    clock(
                        a,
                        s.variant.map(|i| &c.variants[i as usize]),
                        tick.saturating_sub(s.animation_started),
                    )
                });
                let traits = npc
                    .mind_profile
                    .or_else(|| {
                        e.npc
                            .as_ref()
                            .and_then(|n| n.mind)
                            .map(|m| m.resolve(instance.instance))
                    })
                    .map_or([5000; 9], |p| p.traits);
                let variant = select_variant(c, phase, traits, instance, life);
                let animation_started = if phase == Phase::Walk {
                    next.motion.get(id).map_or(tick, |m| m.started)
                } else {
                    tick
                };
                next.actors.insert(
                    id.clone(),
                    State {
                        instance,
                        life,
                        phase,
                        activity,
                        variant,
                        started: tick,
                        animation_started,
                        previous_clip,
                        previous_tick,
                        next_step: tick,
                    },
                );
            }
            let s = next.actors.get_mut(id).unwrap();
            if changed || phase == Phase::Walk && tick >= s.next_step {
                s.next_step = tick.saturating_add(u64::from(c.step_ticks));
                if let Some(sound) = s
                    .variant
                    .and_then(|i| c.variants[i as usize].sound.as_ref())
                    .and_then(|id| defs.avatar_audio.sounds.iter().position(|x| x.id == *id))
                {
                    if next.voices.len() < VOICES {
                        next.voices.push(Voice {
                            instance,
                            life,
                            sound: sound as u8,
                            started: tick,
                            position: self.position(id).unwrap(),
                        });
                    } else {
                        next.dropped_voices = next.dropped_voices.saturating_add(1);
                    }
                }
            }
        }
        self.presentation = next;
    }
    pub fn presentation_poses(&self) -> Vec<Pose> {
        let d = self.definitions();
        let tick = self.tick;
        self.presentation
            .actors
            .iter()
            .filter_map(|(id, s)| {
                let slot = self.actors.actor_slot(s.instance)?;
                let e = &d.entities[self.actors.entity(slot as u16)];
                if e.visible == Some(false) {
                    return None;
                }
                let c = e.avatar.as_ref()?;
                let (asset, a) = d
                    .meshes
                    .assets
                    .iter()
                    .enumerate()
                    .find(|(_, a)| a.id == c.asset)?;
                let (clip, clip_tick) = clock(
                    a,
                    s.variant.map(|i| &c.variants[i as usize]),
                    tick.saturating_sub(s.animation_started),
                );
                let n = self.npcs.get(id)?;
                let mut pose = Pose {
                    actor: s.instance.instance,
                    life: s.life,
                    asset: asset as u32,
                    clip,
                    clip_tick,
                    previous_clip: s.previous_clip,
                    previous_tick: s.previous_tick,
                    blend: if c.blend_ticks == 0 {
                        10000
                    } else {
                        (tick.saturating_sub(s.started).min(u64::from(c.blend_ticks)) * 10000
                            / u64::from(c.blend_ticks))
                        .min(10000) as u16
                    },
                    yaw: (n.facing_degrees.clamp(-180., 180.) * 100.).round() as i16,
                    pitch: (n.facing_pitch_degrees.unwrap_or(0.).clamp(-90., 90.) * 100.).round()
                        as i16,
                    scale: c.scale,
                    offset: c.offset,
                    hide_body: c.hide_body,
                    orientation: c.orientation,
                    strip_root: c.root_motion.map(|r| r.bone),
                    joint_count: 0,
                    ik_limited: 0,
                    joints: [Default::default(); crate::avatar_ik::JOINTS],
                };
                self.correct_avatar_pose(id, c, &mut pose);
                Some(pose)
            })
            .collect::<Vec<_>>()
    }
}
pub fn pcm(
    audio: &Audio,
    voices: &[Voice],
    tick: u64,
    frames: u32,
    listener: [f32; 3],
    radius: f32,
) -> Result<Vec<i16>> {
    if voices.len() > VOICES
        || frames == 0
        || frames > RATE
        || !radius.is_finite()
        || radius <= 0.
        || radius > 10000.
        || listener.iter().any(|x| !x.is_finite() || x.abs() > 10000.)
    {
        return Err(invalid(
            "PCM capture requires1..24000frames and finite listener/radius",
        ));
    }
    let mut out = vec![0i32; frames as usize];
    for voice in voices {
        let Some(sound) = audio.sounds.get(voice.sound as usize) else {
            return Err(invalid("Invalid voice sound"));
        };
        let distance = glam::Vec3::from(voice.position).distance(listener.into());
        let gain = (1. - distance / radius).clamp(0., 1.);
        let start = tick.saturating_sub(voice.started) * 400;
        for (i, o) in out.iter_mut().enumerate() {
            if let Some(s) = sound.samples.get(start as usize + i) {
                *o += (f32::from(*s) * gain).round() as i32;
            }
        }
    }
    Ok(out
        .into_iter()
        .map(|x| x.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
        .collect())
}
pub fn wav(samples: &[i16]) -> Vec<u8> {
    let bytes = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(bytes as usize + 44);
    out.extend(b"RIFF");
    out.extend((bytes + 36).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(RATE.to_le_bytes());
    out.extend((RATE * 2).to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend(bytes.to_le_bytes());
    for s in samples {
        out.extend(s.to_le_bytes());
    }
    out
}
