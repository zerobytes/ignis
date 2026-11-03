use glam::{Quat, Vec3};
use ignis::{
    editor::MeshView,
    engine::Command,
    mesh_screen::{self, Kind},
    mesh_topology::Components,
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
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [2., 1.05, 6.];
    camera.target = [2., 1.05, 0.];
    s.execute(
        Command::EditEditor {
            action: ignis::editor::Action::Camera {
                camera: Some(camera),
            },
            expected_editor_revision: None,
        }
        .into(),
    )
    .unwrap();
    (root, s, id)
}
fn select(s: &mut Service, id: &str, kind: Kind, min: [f32; 2], max: [f32; 2], additive: bool) {
    s.execute(
        Command::SelectEditorMeshRegion {
            instance: id.into(),
            reference: s.engine.mesh_reference(),
            component: kind,
            min,
            max,
            width: 800,
            height: 600,
            additive,
        }
        .into(),
    )
    .unwrap();
}
fn count(s: &Service) -> usize {
    s.engine
        .selected_mesh_components()
        .map(|s| mesh_screen::count(&s.components))
        .unwrap_or(0)
}
#[test]
fn region_receipt_includes_clearing_a_previously_selected_actor() {
    let (_root, mut s, id) = fixture();
    s.execute(
        Command::SelectEntity {
            id: Some("player".into()),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.selected_id(), Some("player"));
    let before = s.engine.snapshot();
    let command = |min| Command::SelectEditorMeshRegion {
        instance: id.clone(),
        reference: s.engine.mesh_reference(),
        component: Kind::Vertices,
        min,
        max: [800., 600.],
        width: 800,
        height: 600,
        additive: false,
    };
    let invalid = command([-1., 0.]);
    let valid = command([0., 0.]);
    assert!(s.execute(invalid.into()).is_err());
    assert_eq!(s.engine.snapshot(), before);
    let reply = s.execute(valid.into()).unwrap().data;
    let after = s.engine.snapshot();
    assert_eq!(reply["view_revision"], after["view_revision"]);
    assert!(after["view_revision"].as_u64().unwrap() > before["view_revision"].as_u64().unwrap());
    assert!(s.engine.selected_id().is_none());
    assert_eq!(s.selected_mesh.as_deref(), Some(id.as_str()));
    assert!(count(&s) > 100);
    assert_eq!(after["scene"], before["scene"]);
    assert_eq!(after["history"], before["history"]);
}
#[test]
fn rectangle_visibility_xray_additive_clear_and_stale_are_session_only() {
    let (_root, mut s, id) = fixture();
    let scene = s.engine.scene.clone();
    let history = s.engine.snapshot()["history"].clone();
    select(&mut s, &id, Kind::Vertices, [0., 0.], [800., 600.], false);
    let front = count(&s);
    assert!(front > 100);
    let selection = s.engine.selected_mesh_components().unwrap();
    let Components::Vertices { indices } = &selection.components else {
        panic!()
    };
    let pose = s.engine.scene.meshes.pose(&id, 0).unwrap();
    assert!(indices.iter().all(|i| pose.positions[*i as usize][2] > 0.));
    let previous = selection.clone();
    select(&mut s, &id, Kind::Vertices, [0., 0.], [10., 10.], true);
    assert_eq!(s.engine.selected_mesh_components(), Some(&previous));
    select(&mut s, &id, Kind::Vertices, [0., 0.], [10., 10.], false);
    assert_eq!(count(&s), 0);
    s.execute(
        Command::SetEditorMeshView {
            view: MeshView {
                xray: true,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    select(&mut s, &id, Kind::Vertices, [0., 0.], [400., 600.], false);
    let left = count(&s);
    select(&mut s, &id, Kind::Vertices, [400., 0.], [800., 600.], true);
    let all = s
        .engine
        .scene
        .meshes
        .asset(&id)
        .unwrap()
        .geometry
        .positions
        .len();
    assert!(left > front / 2 && left < all);
    assert_eq!(count(&s), all);
    select(&mut s, &id, Kind::Vertices, [0., 0.], [800., 600.], true);
    assert_eq!(count(&s), all);
    assert_eq!(s.engine.scene, scene);
    assert_eq!(s.engine.snapshot()["history"], history);
    let before = s.engine.snapshot();
    let mut reference = s.engine.mesh_reference();
    reference.revision += 1;
    assert!(
        s.execute(
            Command::SelectEditorMeshRegion {
                instance: id.clone(),
                reference,
                component: Kind::Vertices,
                min: [0., 0.],
                max: [800., 600.],
                width: 800,
                height: 600,
                additive: false
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(s.engine.snapshot(), before);
    for (min, max, width) in [
        ([-1., 0.], [800., 600.], 800),
        ([20., 0.], [10., 600.], 800),
        ([0., 0.], [800., 600.], 0),
    ] {
        assert!(
            s.execute(
                Command::SelectEditorMeshRegion {
                    instance: id.clone(),
                    reference: s.engine.mesh_reference(),
                    component: Kind::Vertices,
                    min,
                    max,
                    width,
                    height: 600,
                    additive: false
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(s.engine.snapshot(), before);
    }
}
#[test]
fn visibility_respects_other_meshes_boxes_wedges_and_camera_changes() {
    let (_root, mut s, id) = fixture();
    let camera = s.editor.current().camera.clone().unwrap();
    let projection = mesh_screen::Projection::new(&camera, 800, 600).unwrap();
    let mut visibility = mesh_screen::Visibility::default();
    visibility.prepare(&s.engine).unwrap();
    assert!(visibility.visible(&projection, Vec3::new(2., 1.05, 1.)));
    assert!(!visibility.visible(&projection, Vec3::new(2., 1.05, -1.)));
    let mut rear = camera.clone();
    rear.position = [2., 1.05, -6.];
    let rear = mesh_screen::Projection::new(&rear, 800, 600).unwrap();
    assert!(visibility.visible(&rear, Vec3::new(2., 1.05, -1.)));
    assert!(!visibility.visible(&rear, Vec3::new(2., 1.05, 1.)));
    let mut obstacle = s
        .engine
        .scene
        .entities
        .iter()
        .find(|e| e.id == "floor")
        .unwrap()
        .clone();
    obstacle.id = "occluder".into();
    obstacle.name = "occluder".into();
    obstacle.transform.position = [2., 1.05, 3.];
    obstacle.transform.size = [4., 4., 0.1];
    s.execute(
        Command::Upsert {
            entity: Box::new(obstacle.clone()),
        }
        .into(),
    )
    .unwrap();
    visibility.prepare(&s.engine).unwrap();
    assert!(!visibility.visible(&projection, Vec3::new(2., 1.85, 0.6)));
    obstacle.geometry = ignis::geometry::Geometry::Wedge {
        rise: ignis::geometry::Rise::PositiveX,
    };
    s.execute(
        Command::Upsert {
            entity: Box::new(obstacle),
        }
        .into(),
    )
    .unwrap();
    visibility.prepare(&s.engine).unwrap();
    assert!(
        visibility.visible(&projection, Vec3::new(2., 1.85, 0.6)),
        "Empty wedge half must not occlude"
    );
    assert!(
        !visibility.visible(&projection, Vec3::new(2., 0.25, 0.6)),
        "Solid wedge half occludes"
    );
    s.execute(
        Command::Remove {
            id: "occluder".into(),
        }
        .into(),
    )
    .unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: ignis::mesh_tools::Primitive::Plane,
            resolution: 4,
            size: 4.,
        }
        .into(),
    )
    .unwrap();
    let mut plane = s.engine.scene.meshes.instances.last().unwrap().clone();
    plane.position = [2., 1.05, 3.];
    plane.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2).to_array();
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutInstance { instance: plane },
        }
        .into(),
    )
    .unwrap();
    visibility.prepare(&s.engine).unwrap();
    assert!(!visibility.visible(&projection, Vec3::new(2., 1.05, 1.)));
    select(&mut s, &id, Kind::Vertices, [0., 0.], [800., 600.], false);
    assert_eq!(count(&s), 0);
    s.execute(
        Command::SetEditorMeshView {
            view: MeshView {
                xray: true,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    select(&mut s, &id, Kind::Vertices, [0., 0.], [800., 600.], false);
    assert_eq!(count(&s), 1106);
}
#[test]
fn rectangle_preserves_edge_and_face_domains_in_both_visibility_modes() {
    let (_root, mut s, id) = fixture();
    for kind in [Kind::Edges, Kind::Faces] {
        s.execute(
            Command::SetEditorMeshView {
                view: MeshView::default(),
            }
            .into(),
        )
        .unwrap();
        select(&mut s, &id, kind, [0., 0.], [800., 600.], false);
        let front = count(&s);
        assert!(front > 100);
        s.execute(
            Command::SetEditorMeshView {
                view: MeshView {
                    xray: true,
                    ..Default::default()
                },
            }
            .into(),
        )
        .unwrap();
        select(&mut s, &id, kind, [0., 0.], [800., 600.], false);
        let all = count(&s);
        assert!(all > front);
        assert!(matches!(
            (
                kind,
                &s.engine.selected_mesh_components().unwrap().components
            ),
            (Kind::Edges, Components::Edges { .. }) | (Kind::Faces, Components::Faces { .. })
        ));
    }
}
