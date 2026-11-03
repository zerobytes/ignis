use ignis::{
    engine::{Command, SceneEdit},
    model::RoomRecipe,
    service::Service,
};
use serde_json::json;
fn call(s: &mut Service, c: Command) -> serde_json::Value {
    s.execute(c.into()).unwrap().data
}
#[test]
fn live_updates_are_visible_atomic_and_share_only_their_own_undo() {
    let tmp = tempfile::tempdir().unwrap();
    let mut s = Service::new(tmp.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let original = s.engine.scene.clone();
    let mut p = original
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    for n in 1..=5 {
        p.transform.position[0] += 0.1;
        call(
            &mut s,
            Command::EditSceneLive {
                gesture: "drag1".into(),
                edits: vec![SceneEdit::Upsert {
                    entity: Box::new(p.clone()),
                }],
            },
        );
        assert_eq!(
            s.engine
                .scene
                .entities
                .iter()
                .find(|e| e.id == "player")
                .unwrap(),
            &p
        );
        assert_eq!(s.engine.snapshot()["history"]["undo"], 1);
        assert_eq!(s.engine.revision, n);
    }
    let state = s.engine.snapshot();
    let mut bad = p.clone();
    bad.transform.size[0] = -1.;
    assert!(
        s.execute(
            Command::EditSceneLive {
                gesture: "drag1".into(),
                edits: vec![SceneEdit::Upsert {
                    entity: Box::new(bad)
                }]
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(s.engine.snapshot(), state);
    call(&mut s, Command::Undo);
    assert_eq!(s.engine.scene, original);
    call(&mut s, Command::Redo);
    assert_eq!(
        s.engine
            .scene
            .entities
            .iter()
            .find(|e| e.id == "player")
            .unwrap(),
        &p
    );
    p.transform.position[0] += 1.;
    call(
        &mut s,
        Command::Upsert {
            entity: Box::new(p.clone()),
        },
    );
    let external = s.engine.scene.clone();
    p.transform.position[0] += 1.;
    call(
        &mut s,
        Command::EditSceneLive {
            gesture: "drag1".into(),
            edits: vec![SceneEdit::Upsert {
                entity: Box::new(p),
            }],
        },
    );
    call(&mut s, Command::Undo);
    assert_eq!(s.engine.scene, external);
}
#[test]
fn npc_creation_components_vertices_rig_and_overlays_use_shared_commands() {
    let tmp = tempfile::tempdir().unwrap();
    let mut s = Service::new(tmp.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    call(
        &mut s,
        Command::CreateEditorMesh {
            primitive: ignis::mesh_tools::Primitive::Plane,
            resolution: 8,
            size: 2.,
        },
    );
    let selected = s.selected_mesh.clone().unwrap();
    let asset = s.engine.scene.meshes.asset(&selected).unwrap().clone();
    let source = s.engine.scene.clone();
    let edit = ignis::meshes::Edit::TransformVertices {
        asset: asset.id.clone(),
        reference: s.engine.mesh_reference(),
        vertices: vec![40],
        transform: ignis::meshes::IslandTransform {
            pivot: [0.; 3],
            translation: [0., 0.1, 0.],
            rotation: [0., 0., 0., 1.],
            scale: 1.,
        },
    };
    call(&mut s, Command::EditMesh { edit: edit.clone() });
    let changed = s.engine.scene.meshes.asset(&asset.id).unwrap();
    for (i, (a, b)) in asset
        .geometry
        .positions
        .iter()
        .zip(&changed.geometry.positions)
        .enumerate()
    {
        if i == 40 {
            assert!((b[1] - a[1] - 0.1).abs() < 1e-6);
        } else {
            assert_eq!(a, b);
        }
    }
    assert_eq!(asset.geometry.triangles, changed.geometry.triangles);
    assert_eq!(
        changed.geometry.material_coordinates.as_ref().unwrap(),
        &asset.geometry.positions
    );
    let before = s.engine.snapshot();
    assert!(s.execute(Command::EditMesh { edit }.into()).is_err());
    assert_eq!(s.engine.snapshot(), before);
    call(&mut s, Command::Undo);
    assert_eq!(s.engine.scene, source);
    let before = s.engine.snapshot();
    call(
        &mut s,
        Command::SetEditorMeshView {
            view: ignis::editor::MeshView {
                edges: true,
                vertices: true,
                bones: true,
                xray: false,
            },
        },
    );
    assert_eq!(s.engine.snapshot(), before);
    assert_eq!(
        call(&mut s, Command::InspectEditor)["mesh_view"],
        json!({"edges":true,"vertices":true,"bones":true,"xray":false})
    );
    call(
        &mut s,
        Command::CreateSceneObject {
            kind: ignis::project::ObjectKind::Npc,
        },
    );
    assert!(s.selected_mesh.is_none());
    let id = s.engine.selected_id().unwrap();
    let npc = s.engine.scene.entities.iter().find(|e| e.id == id).unwrap();
    assert!(npc.npc.as_ref().unwrap().enabled);
}
