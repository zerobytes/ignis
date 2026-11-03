use glam::Vec3;
use ignis::{
    engine::Command,
    mesh_tools::{self, Primitive},
    meshes,
    model::RoomRecipe,
    service::Service,
};
#[test]
fn primitives_are_oriented_connected_bounded_and_all_resolutions_validate() {
    for kind in [
        Primitive::Sphere,
        Primitive::Cube,
        Primitive::Cylinder,
        Primitive::Plane,
    ] {
        for resolution in [4, 16, 32] {
            let geometry = mesh_tools::primitive(kind, resolution, 2.).unwrap();
            let topology = ignis::mesh_topology::Topology::inspect(&geometry).unwrap();
            assert_eq!(topology.non_manifold_edges, 0);
            assert!(topology.isolated_vertices.is_empty());
            assert!(geometry.positions.len() <= meshes::MAX_VERTICES);
            if kind != Primitive::Plane {
                assert_eq!(topology.boundary_edges, 0);
                let volume: f32 = geometry
                    .triangles
                    .iter()
                    .map(|face| {
                        let [a, b, c] = face.map(|i| Vec3::from(geometry.positions[i as usize]));
                        a.dot(b.cross(c)) / 6.
                    })
                    .sum();
                assert!(volume > 0.);
            }
        }
    }
    for (r, s) in [(3, 2.), (33, 2.), (8, f32::NAN), (8, 0.)] {
        assert!(mesh_tools::primitive(Primitive::Sphere, r, s).is_err());
    }
}
#[test]
fn create_pick_preview_atomic_sculpt_undo_save_load_and_replay() {
    let tmp = tempfile::tempdir().unwrap();
    let mut service = Service::new(tmp.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    service
        .execute(
            Command::CreateEditorMesh {
                primitive: Primitive::Sphere,
                resolution: 16,
                size: 2.,
            }
            .into(),
        )
        .unwrap();
    let id = service.selected_mesh.clone().unwrap();
    let original = service.engine.scene.clone();
    let rev = service.engine.revision;
    let mut camera = service.engine.scene.camera.clone();
    camera.position = [2., 1.05, 6.];
    camera.target = [2., 1.05, 0.];
    service
        .execute(
            Command::EditEditor {
                action: ignis::editor::Action::Camera {
                    camera: Some(camera.clone()),
                },
                expected_editor_revision: None,
            }
            .into(),
        )
        .unwrap();
    let hit = service.pick_mesh(400., 300., 800, 600).unwrap().unwrap();
    assert_eq!(hit.instance, id);
    assert!((hit.position[2] - 1.).abs() < 1e-4);
    assert!(service.pick_mesh(-1., 300., 800, 600).is_err());
    assert_eq!(service.engine.revision, rev);
    let stroke = meshes::Stroke {
        kind: meshes::BrushKind::Draw,
        centers: vec![hit.local_position, [0.15, 0., 1.]],
        radius: 0.6,
        strength: 0.04,
        direction: [0., 0., 1.],
    };
    let preview = mesh_tools::sculpt_preview(&original.meshes.assets[0], &stroke).unwrap();
    assert_eq!(service.engine.scene, original);
    service
        .execute(
            Command::EditMesh {
                edit: meshes::Edit::Sculpt {
                    asset: id.clone(),
                    stroke,
                },
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene.meshes.assets[0], preview);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, original);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene.meshes.assets[0], preview);
    service
        .execute(
            Command::SaveScene {
                path: "mesh.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    let mut independent =
        Service::new(tmp.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    independent
        .execute(
            Command::LoadScene {
                path: "mesh.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(independent.engine.scene, service.engine.scene);
    service.execute(Command::Step { ticks: 5 }.into()).unwrap();
    service
        .execute(
            Command::SaveReplay {
                path: "mesh.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert!(
        service
            .execute(
                Command::RunReplay {
                    path: "mesh.replay.json".into()
                }
                .into()
            )
            .unwrap()
            .data["matched"]
            .as_bool()
            .unwrap()
    );
    let before = service.engine.snapshot();
    let selected = service.selected_mesh.clone();
    assert!(
        service
            .execute(
                Command::SelectEditorMesh {
                    instance: Some("missing".into())
                }
                .into()
            )
            .is_err()
    );
    assert!(
        service
            .execute(
                Command::CreateEditorMesh {
                    primitive: Primitive::Sphere,
                    resolution: 33,
                    size: 2.
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(service.engine.snapshot(), before);
    assert_eq!(service.selected_mesh, selected);
    service
        .execute(
            Command::EditMesh {
                edit: meshes::Edit::RemoveInstance { id },
            }
            .into(),
        )
        .unwrap();
    assert!(service.selected_mesh.is_none());
}
#[test]
fn picking_respects_instance_rotation_scale_and_nearest_surface() {
    let geometry = mesh_tools::primitive(Primitive::Cube, 4, 2.).unwrap();
    let asset = meshes::Asset {
        id: "cube".into(),
        geometry: std::sync::Arc::new(geometry),
        shading: meshes::Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    };
    let instance = meshes::Instance {
        id: "near".into(),
        asset: asset.id.clone(),
        position: [0., 1., 0.],
        rotation: glam::Quat::from_rotation_y(0.2).to_array(),
        scale: 2.,
        color: [1.; 3],
        playback: None,
        attachment: None,
    };
    let mut far = instance.clone();
    far.id = "far".into();
    far.position[2] = -6.;
    let document = meshes::Document {
        objects: Default::default(),
        assets: vec![asset],
        instances: vec![far, instance],
    };
    document.validate().unwrap();
    let mut camera = RoomRecipe::default().generate().unwrap().camera;
    camera.position = [0., 1., 10.];
    camera.target = [0., 1., 0.];
    let hit = mesh_tools::pick(&document, &camera, 400., 300., 800, 600, 0, &[])
        .unwrap()
        .unwrap();
    assert_eq!(hit.instance, "near");
    assert!(hit.local_position[2].abs() <= 1.001);
    assert!(hit.normal[2] > 0.9);
    assert_eq!(
        mesh_tools::pick(&document, &camera, 400., 300., 800, 600, 0, &["near"])
            .unwrap()
            .unwrap()
            .instance,
        "far"
    );
}

/// A missed shared edge was painting through the cube and adding metres of
/// hidden path. Exercise screen diagonals, not pre-supplied3D brush centers.
#[test]
fn screen_diagonals_never_pick_through_shared_cube_edges_but_real_gaps_stay_open() {
    use std::sync::Arc;
    let tmp = tempfile::tempdir().unwrap();
    let mut s = Service::new(tmp.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: Primitive::Cube,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )
    .unwrap();
    let mut instance = s.engine.scene.meshes.instances[0].clone();
    instance.position = [-3., 1.05, 0.];
    s.execute(
        Command::EditMesh {
            edit: meshes::Edit::PutInstance { instance },
        }
        .into(),
    )
    .unwrap();
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [-7., 1.05, 0.];
    camera.target = [-3., 1.05, 0.];
    camera.fov_degrees = 48.;
    for slope in [-1., 1.] {
        for i in 0..=80 {
            let t = -180. + i as f32 * 4.5;
            let hit = mesh_tools::pick(
                &s.engine.scene.meshes,
                &camera,
                400. + t,
                400. + slope * t,
                800,
                800,
                0,
                &[],
            )
            .unwrap()
            .unwrap();
            assert!(
                (hit.local_position[0] + 1.).abs() < 1e-5,
                "front face lost at diagonal{slope}/sample{i}: {hit:?}"
            );
        }
    }
    // No expanding barycentric epsilon: an actual narrow opening must expose
    // the back plane rather than becoming a false bridge across the gap.
    let mut document = s.engine.scene.meshes.clone();
    let gap = 0.00001;
    Arc::make_mut(&mut document.assets[0].geometry).positions = vec![
        [-1., -1. + gap, -1.],
        [-1., 1. + gap, -1.],
        [-1., 1. + gap, 1.],
        [-1., -1. - gap, -1.],
        [-1., -1. - gap, 1.],
        [-1., 1. - gap, 1.],
        [1., -1., -1.],
        [1., 1., -1.],
        [1., 1., 1.],
        [1., -1., 1.],
    ];
    Arc::make_mut(&mut document.assets[0].geometry).triangles =
        vec![[0, 1, 2], [3, 4, 5], [6, 7, 8], [6, 8, 9]];
    document.validate().unwrap();
    let hit = mesh_tools::pick(&document, &camera, 400., 400., 800, 800, 0, &[])
        .unwrap()
        .unwrap();
    assert!(
        (hit.local_position[0] - 1.).abs() < 1e-5,
        "real opening was filled by picking tolerance"
    );
}
