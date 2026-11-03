use glam::{Quat, Vec3};
use ignis::{
    engine::{Command, Engine},
    meshes::{Asset, Bone, Clip, Edit, Keyframe, Rig, Track},
    model::Scene,
    retarget::Mapping,
};
use std::sync::Arc;

fn setup() -> (Engine, Vec<Mapping>) {
    let mut scene: Scene =
        serde_json::from_str(include_str!("../examples/npc-presentation.scene.json")).unwrap();
    let mut source = scene.meshes.assets[0].clone();
    source.id = "source_rig".into();
    let mut rig = (*source.rig.as_ref().unwrap().as_ref()).clone();
    rig.bones.push(Bone {
        name: "branch".into(),
        parent: Some(0),
        translation: [0., 1., 0.],
        rotation: Quat::from_rotation_z(0.3).to_array(),
    });
    source.rig = Some(Arc::new(rig));
    source.clips = vec![Arc::new(Clip {
        id: "action".into(),
        duration_ticks: 60,
        tracks: vec![
            Track {
                bone: 0,
                keys: vec![
                    Keyframe {
                        tick: 0,
                        translation: [0.; 3],
                        rotation: Quat::IDENTITY.to_array(),
                    },
                    Keyframe {
                        tick: 60,
                        translation: [0., 0., 2.],
                        rotation: Quat::from_rotation_y(0.4).to_array(),
                    },
                ],
            },
            Track {
                bone: 1,
                keys: vec![
                    Keyframe {
                        tick: 0,
                        translation: [0., 1., 0.],
                        rotation: Quat::from_rotation_z(0.3).to_array(),
                    },
                    Keyframe {
                        tick: 60,
                        translation: [0., 1., 0.],
                        rotation: Quat::from_rotation_z(0.8).to_array(),
                    },
                ],
            },
        ],
    })];
    let mut target: Asset = source.clone();
    target.id = "target_rig".into();
    target.clips.clear();
    let rig: &mut Rig = Arc::make_mut(target.rig.as_mut().unwrap());
    rig.bones[0].name = "thorax".into();
    rig.bones[0].rotation = Quat::from_rotation_x(0.2).to_array();
    rig.bones[1].name = "tentacle".into();
    rig.bones[1].translation = [0., 2., 0.];
    rig.bones[1].rotation = Quat::from_rotation_x(-0.4).to_array();
    scene.meshes.assets.extend([source, target]);
    (
        Engine::new(scene).unwrap(),
        vec![
            Mapping {
                source: "body".into(),
                target: "thorax".into(),
                basis: Quat::IDENTITY.to_array(),
                translation_scale: 2.,
            },
            Mapping {
                source: "branch".into(),
                target: "tentacle".into(),
                basis: Quat::from_rotation_y(0.5).to_array(),
                translation_scale: 0.,
            },
        ],
    )
}
fn edit(mappings: Vec<Mapping>) -> Command {
    Command::EditMesh {
        edit: Edit::RetargetClip {
            asset: "target_rig".into(),
            source: "source_rig".into(),
            clip: "action".into(),
            output: "baked".into(),
            mappings,
        },
    }
}

// Critical dependency/atomicity test: a bad map must not publish a partial clip,
// and the baked result must remain independent after source removal and replay.
#[test]
fn retarget_rest_relative_transaction_and_native_persistence() {
    let (mut e, mappings) = setup();
    let before = e.scene.clone();
    let mut bad = mappings.clone();
    bad[1].source = "missing".into();
    assert!(e.execute(edit(bad).into()).is_err());
    assert_eq!(e.scene, before);
    assert!(e.execute(edit(mappings[..1].to_vec()).into()).is_err());
    assert_eq!(e.scene, before);
    e.execute(edit(mappings.clone()).into()).unwrap();
    let target = e.scene.meshes.assets.last().unwrap();
    let baked = &target.clips[0];
    assert_eq!(baked.tracks[0].keys[1].translation, [0., 0., 4.]);
    assert_eq!(baked.tracks[1].keys[1].translation, [0., 2., 0.]);
    let rest = Quat::from_array(target.rig.as_ref().unwrap().bones[1].rotation);
    let basis = Quat::from_array(mappings[1].basis);
    let expected = rest * basis * Quat::from_rotation_z(0.5) * basis.inverse();
    let got = Quat::from_array(baked.tracks[1].keys[1].rotation);
    assert!(got.abs_diff_eq(expected, 0.00001) || got.abs_diff_eq(-expected, 0.00001));
    assert!((got * Vec3::Y - expected * Vec3::Y).length() < 0.00001);
    let serialized = serde_json::to_string(&e.scene).unwrap();
    let saved: Scene = serde_json::from_str(&serialized).unwrap();
    saved.validate().unwrap();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
    e.execute(edit(mappings).into()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::RemoveAsset {
                id: "source_rig".into(),
            },
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.meshes.assets.last().unwrap().clips[0].id, "baked");
    e.advance(2).unwrap();
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
