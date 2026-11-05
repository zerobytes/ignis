//! First game-authored locomotion clips, evaluated/blended by Ignis' native rig player.
use anyhow::{Result, ensure};
use glam::{Mat4, Quat, Vec3};
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
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Walk => "walk",
            Self::Run => "run",
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

pub const LIBRARY_PATH: &str = "games/land-of-survivors/assets/animations/locomotion.ignis-motion";

/// Seeds editable data from the original draft cycles. This is not an improved walk.
pub fn seed(source: &ignis::meshes::Document) -> Result<ignis::animation::Document> {
    let instance = source
        .instances
        .iter()
        .find(|i| i.id.ends_with("_body"))
        .ok_or_else(|| anyhow::anyhow!("Character body is missing"))?;
    let body = source.asset(&instance.asset)?;
    let rig = body
        .rig
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Character rig is missing"))?;
    let floor = body
        .geometry
        .positions
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);
    let ceiling = body
        .geometry
        .positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let joints = ignis::avatar_ik::pose_joints(body, &ignis::animation::pose(None, 0))?;
    let mut motions = Vec::new();
    for gait in [Gait::Idle, Gait::Walk, Gait::Run] {
        let mut feet = Vec::new();
        for side in ["L", "R"] {
            let tip = rig
                .bones
                .iter()
                .position(|b| b.name == format!("foot_{side}"))
                .unwrap();
            let matrix = Mat4::from_cols_array_2d(&joints[tip]);
            feet.push(ignis::animation::Foot {
                name: format!("foot_{side}"),
                bones: [
                    format!("upperleg01_{side}"),
                    format!("lowerleg01_{side}"),
                    format!("foot_{side}"),
                ],
                stance: if gait == Gait::Idle {
                    [0., 1.]
                } else if side == "L" {
                    [0.25, 0.75]
                } else {
                    [0.75, 0.25]
                },
                pole: [0., 0., -1.],
                normal_axis: (matrix.to_scale_rotation_translation().1.inverse() * Vec3::Y)
                    .to_array(),
                sole_offset: matrix.transform_point3(Vec3::ZERO).y - floor,
                max_correction: 0.5,
            });
        }
        motions.push(ignis::animation::Motion {
            name: gait.name().into(),
            clip: format!("los_{}", gait.name()),
            stride: if gait == Gait::Idle {
                0.
            } else {
                gait.stride() * (ceiling - floor) / 1.8
            },
            feet,
        });
    }
    let document = ignis::animation::Document {
        version: 1,
        floor_y: floor,
        bones: rig.bones.clone(),
        clips: clips(rig)?,
        motions,
    };
    validate_game(&document)?;
    Ok(document)
}

pub fn validate_game(d: &ignis::animation::Document) -> Result<()> {
    d.validate()?;
    let mut clips = std::collections::BTreeSet::new();
    for name in ["idle", "walk", "run"] {
        let m = d.motion(name)?;
        ensure!(clips.insert(&m.clip), "Game motions require distinct clips");
        ensure!(
            name == "idle" || m.stride > 0.001,
            "Moving cycles require a positive stride"
        );
    }
    Ok(())
}

pub fn authoring_scene(
    mut source: ignis::meshes::Document,
    library: &ignis::animation::Document,
    mut scene: ignis::model::Scene,
) -> Result<ignis::model::Scene> {
    use ignis::meshes::{Attachment, Playback};
    let body_instance = source
        .instances
        .iter()
        .find(|i| i.id.ends_with("_body"))
        .unwrap()
        .clone();
    let body = source.asset(&body_instance.asset)?;
    let rig = body.rig.as_ref().unwrap();
    let joints = ignis::avatar_ik::pose_joints(body, &ignis::animation::pose(None, 0))?;
    let head = rig.bones.iter().position(|b| b.name == "head").unwrap();
    let inverse = Mat4::from_cols_array_2d(&joints[head]).inverse();
    let clips = library.clips_for(rig, &["idle", "walk", "run"])?;
    let poses = source.poses(0);
    for asset in &mut source.assets {
        if asset.id == body_instance.asset {
            asset.clips = clips.clone();
        } else {
            let instance = source
                .instances
                .iter()
                .find(|i| i.asset == asset.id)
                .unwrap();
            let rest = poses.iter().find(|p| p.instance == instance.id).unwrap();
            Arc::make_mut(&mut asset.geometry).positions = rest
                .positions
                .iter()
                .map(|p| inverse.transform_point3((*p).into()).to_array())
                .collect();
            asset.rig = None;
            asset.clips.clear();
        }
    }
    let height = source
        .asset(&body_instance.asset)?
        .geometry
        .positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max)
        - library.floor_y;
    for instance in &mut source.instances {
        if instance.id == body_instance.id {
            instance.position = [0., -library.floor_y, 0.];
            instance.playback = Some(Playback {
                clip: clips[0].id.clone(),
                offset_ticks: 0,
                looping: true,
            });
        } else {
            instance.position = [0.; 3];
            instance.rotation = [0., 0., 0., 1.];
            instance.scale = 1.;
            instance.attachment = Some(Attachment {
                parent: body_instance.id.clone(),
                bone: Some("head".into()),
            });
            instance.playback = None;
        }
    }
    source.validate()?;
    scene.name = "Land of Survivors · animation authoring".into();
    scene.meshes = source;
    scene.characters.clear();
    scene.camera = ignis::model::Camera {
        position: [2.4, height * 0.7, -3.5],
        target: [0., height * 0.5, 0.],
        fov_degrees: 38.,
    };
    scene.light = serde_json::from_value(
        serde_json::json!({"direction":[0.5,-0.7,0.7],"ambient":0.34,"direct_intensity":1.1}),
    )?;
    Ok(scene)
}
