//! First game-authored locomotion clips, evaluated/blended by Ignis' native rig player.
use anyhow::{Result, ensure};
use glam::Quat;
use ignis::meshes::{Clip, Keyframe, Rig, Track};
use std::{f32::consts::TAU, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gait {
    Idle,
    Walk,
    Run,
}
impl Gait {
    pub fn index(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Walk => 1,
            Self::Run => 2,
        }
    }
    pub fn duration(self) -> u32 {
        match self {
            Self::Idle => 180,
            Self::Walk => 60,
            Self::Run => 36,
        }
    }
    pub fn stride(self) -> f32 {
        match self {
            Self::Idle => 1.,
            Self::Walk => 2.2,
            Self::Run => 3.0,
        }
    }
}

pub fn clips(rig: &Rig) -> Result<Vec<Arc<Clip>>> {
    for name in [
        "spine03",
        "head",
        "upperarm01_L",
        "upperarm01_R",
        "lowerarm01_L",
        "lowerarm01_R",
        "upperleg01_L",
        "upperleg01_R",
        "lowerleg01_L",
        "lowerleg01_R",
        "foot_L",
        "foot_R",
    ] {
        ensure!(
            rig.bones.iter().any(|b| b.name == name),
            "Locomotion needs native joint {name}"
        );
    }
    let mut globals = Vec::with_capacity(rig.bones.len());
    for b in &rig.bones {
        let q = Quat::from_array(b.rotation);
        globals.push(b.parent.map_or(q, |p| globals[p as usize] * q));
    }
    let mut clips = Vec::new();
    for gait in [Gait::Idle, Gait::Walk, Gait::Run] {
        let mut tracks = Vec::new();
        for (index, b) in rig.bones.iter().enumerate() {
            if !(b.name.starts_with("upperarm01_")
                || b.name.starts_with("lowerarm01_")
                || b.name.starts_with("upperleg01_")
                || b.name.starts_with("lowerleg01_")
                || b.name.starts_with("foot_")
                || b.name == "spine03"
                || b.name == "spine01"
                || b.name == "head")
            {
                continue;
            }
            let mut keys = Vec::new();
            for sample in 0..=24 {
                let phase = sample as f32 / 24. * TAU;
                let side = if b.name.ends_with("_L") { 1. } else { -1. };
                let swing = phase.sin() * side;
                let moving = gait != Gait::Idle;
                let running = gait == Gait::Run;
                let hip = if moving {
                    swing * if running { 43. } else { 30. }
                } else {
                    0.
                };
                let knee = if moving {
                    -4. - swing.max(0.) * if running { 68. } else { 42. }
                } else {
                    -2.
                };
                let (x, y, z): (f32, f32, f32) = if b.name.starts_with("upperarm01_") {
                    (
                        if moving {
                            -swing * if running { 30. } else { 18. }
                        } else {
                            phase.sin() * 0.6
                        },
                        0.,
                        side * 34.,
                    )
                } else if b.name.starts_with("lowerarm01_") {
                    (
                        if running {
                            65. + swing * 6.
                        } else {
                            12. + swing * 3.
                        },
                        0.,
                        0.,
                    )
                } else if b.name.starts_with("upperleg01_") {
                    (hip, 0., 0.)
                } else if b.name.starts_with("lowerleg01_") {
                    (knee, 0., 0.)
                } else if b.name.starts_with("foot_") {
                    (-hip - knee, 0., 0.)
                } else if b.name == "spine03" {
                    (
                        if running { 7. } else { phase.sin() * 0.7 },
                        if moving { phase.sin() * 3. } else { 0. },
                        0.,
                    )
                } else if b.name == "spine01" {
                    (
                        0.,
                        0.,
                        if moving {
                            phase.sin() * 1.2
                        } else {
                            phase.sin() * 0.4
                        },
                    )
                } else {
                    (if moving { -1. } else { phase.sin() * 0.4 }, 0., 0.)
                };
                let world_delta = Quat::from_rotation_x(x.to_radians())
                    * Quat::from_rotation_y(y.to_radians())
                    * Quat::from_rotation_z(z.to_radians());
                let global: Quat = globals[index];
                let local_delta = global.inverse() * world_delta * global;
                let rotation = (Quat::from_array(b.rotation) * local_delta).normalize();
                keys.push(Keyframe {
                    tick: sample * gait.duration() / 24,
                    translation: b.translation,
                    rotation: rotation.to_array(),
                });
            }
            tracks.push(Track {
                bone: index as u8,
                keys,
            });
        }
        clips.push(Arc::new(Clip {
            id: match gait {
                Gait::Idle => "los_idle",
                Gait::Walk => "los_walk",
                Gait::Run => "los_run",
            }
            .into(),
            duration_ticks: gait.duration(),
            tracks,
        }));
    }
    Ok(clips)
}
