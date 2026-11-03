use glam::{Quat, Vec3};
use ignis::{
    editor_transform::{self as tr, Settings, Target},
    engine::Command,
    mesh_topology::Components,
    meshes::IslandTransform,
    model::RoomRecipe,
    service::Service,
};
fn fixture() -> (tempfile::TempDir, Service, String) {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: ignis::mesh_tools::Primitive::Sphere,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )
    .unwrap();
    let id = s.selected_mesh.clone().unwrap();
    (root, s, id)
}
fn identity(pivot: [f32; 3]) -> IslandTransform {
    IslandTransform {
        pivot,
        translation: [0.; 3],
        rotation: Quat::IDENTITY.to_array(),
        scale: 1.,
    }
}
fn inspect(s: &mut Service) -> serde_json::Value {
    s.execute(Command::Inspect.into()).unwrap().data
}
fn apply(s: &mut Service, target: Target, t: IslandTransform) {
    let reference = s.engine.mesh_reference();
    s.execute(
        Command::TransformEditor {
            target,
            reference,
            transform: t,
        }
        .into(),
    )
    .unwrap();
}
#[test]
fn world_mesh_transform_preview_commit_and_one_undo_keep_asset_unchanged() {
    let (_root, mut s, id) = fixture();
    let before = inspect(&mut s);
    let i = s.engine.scene.meshes.instances[0].clone();
    let mut t = identity([1., 0., 0.]);
    t.translation = [0.3, 0.1, 0.2];
    t.rotation = Quat::from_rotation_y(0.4).to_array();
    t.scale = 1.2;
    let target = Target::Mesh { instance: id };
    let preview = tr::preview(&s, &target, &s.engine.mesh_reference(), &t).unwrap();
    assert_eq!(inspect(&mut s), before);
    apply(&mut s, target, t.clone());
    assert_eq!(s.engine.scene, preview);
    let next = &s.engine.scene.meshes.instances[0];
    let expected = Vec3::from(t.pivot)
        + Quat::from_array(t.rotation) * ((Vec3::from(i.position) - Vec3::from(t.pivot)) * t.scale)
        + Vec3::from(t.translation);
    assert!(Vec3::from(next.position).distance(expected) < 1e-6);
    assert_eq!(
        s.engine.scene.meshes.assets[0].geometry,
        preview.meshes.assets[0].geometry
    );
    assert_eq!(
        inspect(&mut s)["history"]["undo"].as_u64().unwrap(),
        before["history"]["undo"].as_u64().unwrap() + 1
    );
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(inspect(&mut s)["scene"], before["scene"]);
    s.execute(Command::Redo.into()).unwrap();
    assert_eq!(s.engine.scene, preview);
}
#[test]
fn component_world_transform_converts_rotated_scaled_instance_and_preserves_selection() {
    let (_root, mut s, id) = fixture();
    let mut i = s.engine.scene.meshes.instances[0].clone();
    i.rotation = Quat::from_rotation_y(0.7).to_array();
    i.scale = 1.7;
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutInstance {
                instance: i.clone(),
            },
        }
        .into(),
    )
    .unwrap();
    // All vertices allow a rigid transform without folding the boundary.
    let indices =
        (0..s.engine.scene.meshes.assets[0].geometry.positions.len() as u32).collect::<Vec<_>>();
    let components = Components::Vertices {
        indices: indices.clone(),
    };
    let target = Target::Components {
        instance: id.clone(),
        components: components.clone(),
    };
    let original = s.engine.scene.meshes.assets[0].geometry.positions.clone();
    let mut t = identity(i.position);
    t.translation = [0.01, 0.02, -0.01];
    t.rotation = Quat::from_rotation_x(0.1).to_array();
    t.scale = 1.1;
    apply(&mut s, target, t.clone());
    for (old, new) in original
        .iter()
        .zip(&s.engine.scene.meshes.assets[0].geometry.positions)
    {
        let world =
            Vec3::from(i.position) + Quat::from_array(i.rotation) * (Vec3::from(*old) * i.scale);
        let expected = Vec3::from(t.pivot)
            + Quat::from_array(t.rotation) * ((world - Vec3::from(t.pivot)) * t.scale)
            + Vec3::from(t.translation);
        let actual =
            Vec3::from(i.position) + Quat::from_array(i.rotation) * (Vec3::from(*new) * i.scale);
        assert!(actual.distance(expected) < 1e-5);
    }
    let selection = s.engine.selected_mesh_components().unwrap();
    assert_eq!(selection.components, components);
    assert_eq!(selection.reference, s.engine.mesh_reference());
    assert_eq!(
        tr::selection(&s, true).unwrap().target,
        Target::Components {
            instance: id,
            components
        }
    );
}
#[test]
fn entity_move_and_uniform_scale_preserve_npc_and_reject_rotation_atomically() {
    let (_root, mut s, _) = fixture();
    s.execute(
        Command::CreateSceneObject {
            kind: ignis::project::ObjectKind::Npc,
        }
        .into(),
    )
    .unwrap();
    let e = s
        .engine
        .scene
        .entities
        .iter()
        .find(|e| e.npc.is_some())
        .unwrap()
        .clone();
    let target = Target::Entity { id: e.id.clone() };
    let before = inspect(&mut s);
    let mut t = identity(e.transform.position);
    t.rotation = Quat::from_rotation_y(0.1).to_array();
    assert!(tr::preview(&s, &target, &s.engine.mesh_reference(), &t).is_err());
    assert!(
        s.execute(
            Command::TransformEditor {
                target: target.clone(),
                reference: s.engine.mesh_reference(),
                transform: t
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(inspect(&mut s), before);
    let mut t = identity(e.transform.position);
    t.translation = [0.1, 0., 0.];
    t.scale = 0.9;
    apply(&mut s, target, t);
    let new = s
        .engine
        .scene
        .entities
        .iter()
        .find(|v| v.id == e.id)
        .unwrap();
    assert_eq!(new.npc, e.npc);
    assert!((new.transform.position[0] - e.transform.position[0] - 0.1).abs() < 1e-6);
}
#[test]
fn stale_reference_reset_bad_values_and_settings_leave_scene_runtime_history_unchanged() {
    let (_root, mut s, id) = fixture();
    let reference = s.engine.mesh_reference();
    s.execute(Command::Reset.into()).unwrap();
    let target = Target::Mesh { instance: id };
    let before = inspect(&mut s);
    assert!(
        s.execute(
            Command::TransformEditor {
                target: target.clone(),
                reference,
                transform: identity([0.; 3])
            }
            .into()
        )
        .is_err()
    );
    for t in [
        IslandTransform {
            scale: -1.,
            ..identity([0.; 3])
        },
        IslandTransform {
            rotation: [0.; 4],
            ..identity([0.; 3])
        },
        IslandTransform {
            translation: [f32::NAN, 0., 0.],
            ..identity([0.; 3])
        },
    ] {
        assert!(
            s.execute(
                Command::TransformEditor {
                    target: target.clone(),
                    reference: s.engine.mesh_reference(),
                    transform: t
                }
                .into()
            )
            .is_err()
        );
    }
    assert!(
        s.execute(
            Command::SetEditorTransform {
                settings: Settings {
                    move_step: 0.,
                    ..Default::default()
                }
            }
            .into()
        )
        .is_err()
    );
    s.execute(
        Command::SetEditorTransform {
            settings: Settings {
                snap: true,
                tool: tr::Tool::Move,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    assert_eq!(
        s.execute(Command::InspectEditor.into()).unwrap().data["transform"]["snap"],
        true
    );
    assert_eq!(inspect(&mut s), before);
    s.execute(Command::Play.into()).unwrap();
    let before = inspect(&mut s);
    assert!(tr::preview(&s, &target, &s.engine.mesh_reference(), &identity([0.; 3])).is_err());
    assert_eq!(inspect(&mut s), before);
}
