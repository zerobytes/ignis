use ignis::{
    engine::{Command, Engine, SceneEdit},
    mesh_preservation::DisplacementPolicy,
    meshes,
    model::RoomRecipe,
    plants::{Definition, Edit, Rectangle},
};
use std::sync::Arc;
fn definition() -> Definition {
    Definition {
        trajectory: None,
        id: "limb".into(),
        profile: Rectangle {
            id: "outline".into(),
            width: 2.,
            height: 4.,
        },
        depth: 0.5,
        segments: [2, 4, 1],
        sections: None,
        subdivision_levels: 0,
        sampling_tolerance: 1. / 1024.,
    }
}
fn put(definition: Definition) -> Command {
    Command::EditPlant {
        edit: Edit::Put {
            definition,
            detail_policy: DisplacementPolicy::Physical,
            animation_translation_policy: DisplacementPolicy::Physical,
        },
    }
}
fn engine() -> Engine {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    e.execute(put(definition()).into()).unwrap();
    e
}

#[test]
fn shading_metadata_preserves_the_canonical_reference_after_regeneration() {
    let mut e = engine();
    decorate(&mut e);
    let original = e.scene.meshes.asset("limb").unwrap().clone();
    let mut grown = definition();
    grown.profile.width *= 1.5;
    e.execute(put(grown).into()).unwrap();
    let before = e.scene.plants.entries[0].clone();
    mesh(
        &mut e,
        meshes::Edit::SetShading {
            asset: "limb".into(),
            shading: meshes::Shading::Smooth,
        },
    );
    let after = &e.scene.plants.entries[0];
    assert_eq!(after.reference, before.reference);
    assert_eq!(after.definition, before.definition);
    assert!(Arc::ptr_eq(&after.source.geometry, &before.source.geometry));
    assert_eq!(after.source.rig, before.source.rig);
    assert_eq!(after.source.clips, before.source.clips);
    assert_eq!(after.source.shading, meshes::Shading::Smooth);
    e.execute(put(definition()).into()).unwrap();
    let returned = e.scene.meshes.asset("limb").unwrap();
    assert_eq!(returned.geometry, original.geometry);
    assert_eq!(returned.rig, original.rig);
    assert_eq!(returned.clips, original.clips);
    assert_eq!(returned.shading, meshes::Shading::Smooth);
}
fn mesh(e: &mut Engine, edit: meshes::Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}
fn decorate(e: &mut Engine) {
    mesh(
        e,
        meshes::Edit::Sculpt {
            asset: "limb".into(),
            stroke: meshes::Stroke {
                kind: meshes::BrushKind::Grab,
                centers: vec![[0., 0., 0.25]],
                radius: 0.7,
                strength: 0.08,
                direction: [0., 0., 1.],
            },
        },
    );
    let asset = e.scene.meshes.asset("limb").unwrap();
    let weights = asset
        .geometry
        .positions
        .iter()
        .map(|p| {
            let t = ((p[1] + 2.) / 2.).clamp(0., 2.);
            let low = (t.floor() as u8).min(1);
            let f = t - low as f32;
            meshes::Weight {
                extra: None,
                joints: [low, low + 1, 0, 0],
                weights: [1. - f, f, 0., 0.],
            }
        })
        .collect();
    let bones = (0..3)
        .map(|i| meshes::Bone {
            name: format!("bone{i}"),
            parent: if i == 0 { None } else { Some(i - 1) },
            translation: [0., if i == 0 { -2. } else { 2. }, 0.],
            rotation: [0., 0., 0., 1.],
        })
        .collect();
    mesh(
        e,
        meshes::Edit::SetRig {
            asset: "limb".into(),
            rig: Some(Arc::new(meshes::Rig { bones, weights })),
        },
    );
    mesh(
        e,
        meshes::Edit::PutClip {
            asset: "limb".into(),
            clip: Arc::new(meshes::Clip {
                id: "bend".into(),
                duration_ticks: 60,
                tracks: vec![meshes::Track {
                    bone: 1,
                    keys: vec![
                        meshes::Keyframe {
                            tick: 0,
                            translation: [0., 2., 0.],
                            rotation: [0., 0., 0., 1.],
                        },
                        meshes::Keyframe {
                            tick: 60,
                            translation: [0., 2., 0.],
                            rotation: [0., 0., 0.38268343, 0.9238795],
                        },
                    ],
                }],
            }),
        },
    );
}
#[test]
fn dimensional_change_preserves_canonical_detail_rig_and_local_motion() {
    let mut e = engine();
    decorate(&mut e);
    let source = e.scene.plants.entries[0].source.clone();
    let mut d = definition();
    d.profile.width = 3.;
    d.profile.height = 6.;
    d.depth = 1.;
    e.execute(put(d.clone()).into()).unwrap();
    assert!(Arc::ptr_eq(&source, &e.scene.plants.entries[0].source));
    let asset = e.scene.meshes.asset("limb").unwrap();
    assert_eq!(
        asset.rig.as_ref().unwrap().bones[0].translation,
        [0., -3., 0.]
    );
    assert_eq!(
        asset.rig.as_ref().unwrap().bones[1].translation,
        [0., 3., 0.]
    );
    assert_eq!(asset.clips[0].tracks[0].keys[0].translation, [0., 3., 0.]);
    let old_base = definition().sample().unwrap();
    let new_base = d.sample().unwrap();
    for i in 0..asset.geometry.positions.len() {
        for axis in 0..3 {
            let delta = source.geometry.positions[i][axis] - old_base.geometry.positions[i][axis];
            assert!(
                (asset.geometry.positions[i][axis] - new_base.geometry.positions[i][axis] - delta)
                    .abs()
                    < 1e-5
            );
        }
    }
    e.execute(put(definition()).into()).unwrap();
    let original = e.scene.meshes.asset("limb").unwrap().clone();
    for _ in 0..5 {
        e.execute(put(d.clone()).into()).unwrap();
        e.execute(put(definition()).into()).unwrap();
    }
    assert_eq!(e.scene.meshes.asset("limb").unwrap(), &original);
}
#[test]
fn retessellation_resamples_from_source_and_roundtrips_without_drift() {
    let mut e = engine();
    decorate(&mut e);
    let before = e.scene.clone();
    let mut d = definition();
    d.segments = [4, 8, 2];
    e.execute(put(d).into()).unwrap();
    let asset = e.scene.meshes.asset("limb").unwrap();
    assert!(asset.geometry.positions.len() > before.meshes.assets[0].geometry.positions.len());
    assert_eq!(
        asset.rig.as_ref().unwrap().weights.len(),
        asset.geometry.positions.len()
    );
    e.scene.validate().unwrap();
    let saved = serde_json::to_vec(&e.scene).unwrap();
    let loaded = serde_json::from_slice(&saved).unwrap();
    assert_eq!(Engine::new(loaded).unwrap().scene, e.scene);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
    e.execute(Command::Redo.into()).unwrap();
    e.execute(put(definition()).into()).unwrap();
    assert_eq!(e.scene.meshes, before.meshes);
}
#[test]
fn conflicts_and_cache_tampering_are_rejected_atomically() {
    let mut e = engine();
    decorate(&mut e);
    let before = e.execute(Command::Inspect.into()).unwrap();
    let mut d = definition();
    d.profile.id = "different_region".into();
    assert!(e.execute(put(d).into()).is_err());
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    assert!(
        e.execute(
            Command::EditMesh {
                edit: meshes::Edit::RemoveAsset { id: "limb".into() }
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    let mut forged = e.scene.clone();
    Arc::make_mut(&mut forged.meshes.assets[0].geometry).positions[0][0] += 0.1;
    assert!(forged.validate().is_err());
    let mut forged = e.scene.clone();
    forged.version = 48;
    assert!(forged.validate().is_err());
    let mut coarse = definition();
    coarse.segments = [1; 3];
    assert!(e.execute(put(coarse).into()).is_err());
    let mut d = definition();
    d.segments = [63; 3];
    assert!(e.execute(put(d).into()).is_err());
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
}
#[test]
fn preview_queries_do_not_mutate_and_detach_is_explicit() {
    let mut e = engine();
    let before = e.execute(Command::Inspect.into()).unwrap();
    let mut d = definition();
    d.depth = 2.;
    let Command::EditPlant { edit } = put(d) else {
        unreachable!()
    };
    e.execute(Command::PreviewPlant { edit }.into()).unwrap();
    e.execute(Command::InspectPlant { id: "limb".into() }.into())
        .unwrap();
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Plant {
                    edit: Edit::Detach { id: "limb".into() },
                },
                SceneEdit::Mesh {
                    edit: meshes::Edit::RemoveAsset { id: "limb".into() },
                },
            ],
        }
        .into(),
    )
    .unwrap();
    assert!(e.scene.plants.is_empty());
    assert!(e.scene.meshes.is_empty());
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        e.execute(Command::Inspect.into()).unwrap()["scene"],
        before["scene"]
    );
}
