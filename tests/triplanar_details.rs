use ignis::{
    engine::{Command, Engine},
    materials::{Detail, Triplanar},
    meshes::{Edit, Frame, Geometry},
    model::RoomRecipe,
};
use serde_json::json;
use std::sync::Arc;

fn detail() -> Detail {
    serde_json::from_value(json!({
        "tint":[0.2,0.3,0.4],"opacity":0.7,"bump":0.002,
        "segments":[{"start":[0,0],"end":[1,1],"widths":[0.1,0.01]}]
    }))
    .unwrap()
}

#[test]
fn optional_projection_is_strict_bounded_and_legacy_serialization_is_unchanged() {
    let mut d = detail();
    let old = serde_json::to_value(&d).unwrap();
    assert!(old.get("triplanar").is_none());
    let mut v = old.clone();
    v["triplanar"] = json!({});
    d = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(d.triplanar, Some(Triplanar { sharpness: 2. }));
    d.validate().unwrap();
    v["triplanar"]["unknown"] = json!(0);
    assert!(serde_json::from_value::<Detail>(v).is_err());
    for p in [f32::NAN, f32::INFINITY, 0., 0.999, 8.001] {
        d.triplanar.as_mut().unwrap().sharpness = p;
        assert!(d.validate().is_err());
    }
    for p in [1., 2., 4., 8.] {
        d.triplanar.as_mut().unwrap().sharpness = p;
        d.validate().unwrap();
    }
    d.triplanar = None;
    assert_eq!(serde_json::to_value(d).unwrap(), old);
    assert_eq!(std::mem::size_of::<ignis::render::Vertex>(), 64);
    assert_eq!(std::mem::offset_of!(ignis::render::Vertex, material), 48);
    assert_eq!(
        std::mem::offset_of!(ignis::render::Vertex, material_projection_weights),
        52
    );
}

#[test]
fn native_projection_publication_legacy_gate_and_replay_preserve_history() {
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::Grid {
                id: "sheet".into(),
                segments: 1,
                size: 1.,
            },
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    let scene = e.scene.clone();
    let before = e.snapshot();
    let mut d = detail();
    d.triplanar = Some(Default::default());
    let edit = Edit::SetMaterial {
        asset: "sheet".into(),
        material: ignis::materials::Material {
            detail: Some(Arc::new(d)),
            ..Default::default()
        },
    };
    e.execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(e.snapshot(), before);
    e.execute(Command::EditMesh { edit }.into()).unwrap();
    let projected = e.scene.clone();
    assert_ne!(projected, scene);
    let mut legacy = projected.clone();
    legacy.version = 69;
    assert!(legacy.validate().unwrap_err().message.contains("version70"));
    let roundtrip: ignis::model::Scene =
        serde_json::from_value(serde_json::to_value(&projected).unwrap()).unwrap();
    roundtrip.validate().unwrap();
    assert_eq!(roundtrip, projected);
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(Engine::replay_at(&replay, None).unwrap().scene, projected);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, scene);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, projected);
}

#[test]
fn frame_weights_follow_material_domain_and_survive_pose_material_and_identity_changes() {
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::Grid {
                id: "sheet".into(),
                segments: 1,
                size: 1.,
            },
        }
        .into(),
    )
    .unwrap();
    let mut doc = e.scene.meshes.clone();
    doc.instances.push(ignis::meshes::Instance {
        id: "one".into(),
        asset: "sheet".into(),
        position: [0., 2., 0.],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [0.5; 3],
        attachment: None,
        playback: None,
    });
    let mut f = Frame::default();
    f.update(&doc, 0);
    assert!(
        f.vertices
            .iter()
            .all(|v| v.material_projection_weights == [0., 1., 0.])
    );
    let weights = f
        .vertices
        .iter()
        .map(|v| v.material_projection_weights)
        .collect::<Vec<_>>();
    doc.instances[0].rotation = glam::Quat::from_rotation_z(0.7).to_array();
    doc.instances[0].position[0] = 3.;
    doc.assets[0].material.detail = Some(Arc::new(detail()));
    f.update(&doc, 8);
    assert_eq!(
        weights,
        f.vertices
            .iter()
            .map(|v| v.material_projection_weights)
            .collect::<Vec<_>>()
    );
    Arc::make_mut(&mut doc.assets[0].geometry).material_coordinates = Some(
        doc.assets[0]
            .geometry
            .positions
            .iter()
            .map(|p| [p[0], p[2], p[1]])
            .collect(),
    );
    f.update(&doc, 8);
    assert!(
        f.vertices
            .iter()
            .all(|v| v.material_projection_weights == [0., 0., 1.])
    );
    // Load an equal-valued Geometry with a new owning identity, at the same tick.
    let old = Arc::downgrade(&doc.assets[0].geometry);
    doc.assets[0].geometry = Arc::new((*doc.assets[0].geometry).clone());
    f.update(&doc, 8);
    // Frame's source may retain the last equal document; the projection cache
    // itself must still prepare/prune the new identity before this early return.
    Arc::make_mut(&mut doc.assets[0].geometry).material_coordinates = None;
    f.update(&doc, 8);
    assert!(old.upgrade().is_none());
    assert!(
        f.vertices
            .iter()
            .all(|v| v.material_projection_weights == [0., 1., 0.])
    );
    doc.assets.clear();
    doc.instances.clear();
    f.update(&doc, 0);
    assert!(f.vertices.is_empty());
    let g = Geometry::grid(1, 1.).unwrap();
    assert_eq!(
        ignis::mesh_projection::weights(&g).unwrap(),
        vec![[0., 1., 0.]; 4]
    );
}
