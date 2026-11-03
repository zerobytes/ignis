use ignis::{
    engine::{Command, Engine},
    mesh_bridge,
    meshes::*,
    model::RoomRecipe,
};
use std::sync::Arc;
fn source() -> Asset {
    let n = 8;
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    for (part, ys) in [[0., 1.], [2., 3.]].into_iter().enumerate() {
        let start = part * 16;
        for y in ys {
            for j in 0..n {
                let a = j as f32 * std::f32::consts::TAU / n as f32;
                positions.push([a.sin(), y, a.cos()]);
            }
        }
        for j in 0..n {
            let [a, b, c, d] = [
                start + j,
                start + (j + 1) % n,
                start + 8 + j,
                start + 8 + (j + 1) % n,
            ]
            .map(|i| i as u32);
            triangles.extend([[a, b, c], [b, d, c]]);
        }
        for j in 1..n - 1 {
            triangles.push([start as u32, (start + j + 1) as u32, (start + j) as u32]);
            triangles.push([
                (start + 8) as u32,
                (start + 8 + j) as u32,
                (start + 8 + j + 1) as u32,
            ]);
        }
    }
    let coordinates = positions
        .iter()
        .map(|p| [p[0] + 7., p[1] * 2., p[2] - 3.])
        .collect();
    Asset {
        id: "tube".into(),
        geometry: Arc::new(Geometry {
            positions,
            triangles,
            material_coordinates: Some(coordinates),
        }),
        shading: Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    }
}
fn cut_faces() -> Vec<u32> {
    (0..6).flat_map(|i| [17 + i * 2, 44 + i * 2]).collect()
}
fn loops() -> [Vec<u32>; 2] {
    [(8..16).collect(), (16..24).collect()]
}
#[test]
fn real_cuts_and_bridge_form_closed_component_preserve_points_and_material_domain() {
    let original = source();
    Document {
        objects: Default::default(),
        assets: vec![original.clone()],
        instances: vec![],
    }
    .validate()
    .unwrap();
    let cut = mesh_bridge::remove_faces(&original, &cut_faces()).unwrap();
    assert_eq!(cut.geometry.positions, original.geometry.positions);
    assert_eq!(cut.geometry.triangles.len(), 44);
    let boundaries = mesh_bridge::boundaries(&cut.geometry).unwrap();
    assert_eq!(boundaries.len(), 2);
    assert!(boundaries.iter().all(|b| b.vertices.len() == 8));
    let out = mesh_bridge::bridge(&cut, &loops(), 4, [0.3, 0.3]).unwrap();
    assert_eq!(out.geometry.positions.len(), 56);
    assert_eq!(out.geometry.triangles.len(), 108);
    assert_eq!(&out.geometry.positions[..32], &original.geometry.positions);
    assert_eq!(
        &out.geometry.material_coordinates.as_ref().unwrap()[..32],
        original.geometry.material_coordinates.as_ref().unwrap()
    );
    let t = ignis::mesh_topology::Topology::inspect(&out.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);
    assert!(t.isolated_vertices.is_empty());
    assert_eq!(
        t.vertices as i64 - t.edges.len() as i64 + t.faces.len() as i64,
        2
    );
    for row in 1..4 {
        for j in 0..8 {
            let p = out.geometry.positions[32 + (row - 1) * 8 + j];
            assert!((p[0] - original.geometry.positions[8 + j][0]).abs() < 1e-6);
            assert!((p[2] - original.geometry.positions[8 + j][2]).abs() < 1e-6);
            assert!((1.0..2.0).contains(&p[1]));
        }
    }
}
#[test]
fn invalid_loops_handles_budget_and_face_ids_leave_source_intact() {
    let original = source();
    for f in [vec![], vec![999], vec![1, 1], (0..56).collect()] {
        assert!(mesh_bridge::remove_faces(&original, &f).is_err());
    }
    let cut = mesh_bridge::remove_faces(&original, &cut_faces()).unwrap();
    let snapshot = cut.clone();
    for case in 0..6 {
        let mut lp = loops();
        let mut h = [0.3, 0.3];
        let mut spans = 4;
        match case {
            0 => lp[0].reverse(),
            1 => lp[1] = lp[0].clone(),
            2 => {
                lp[0][0] = 999;
            }
            3 => h[0] = f32::NAN,
            4 => spans = 33,
            _ => {
                lp[1].pop();
            }
        }
        assert!(mesh_bridge::bridge(&cut, &lp, spans, h).is_err());
        assert_eq!(cut, snapshot);
    }
    assert_eq!(original, source());
    let error = mesh_bridge::bridge(&cut, &loops(), 8, [2.; 2]).unwrap_err();
    assert!(error.to_string().contains("reaches90"));
    let mut large = cut.clone();
    let g = Arc::make_mut(&mut large.geometry);
    g.positions.resize(MAX_VERTICES - 8, [4., 4., 4.]);
    g.material_coordinates
        .as_mut()
        .unwrap()
        .resize(MAX_VERTICES - 8, [1.; 3]);
    assert!(mesh_bridge::counts(&large, &loops(), 4).is_err());
}
#[test]
fn rig_and_clips_survive_new_weights_without_silent_influence_reduction() {
    let mut a = source();
    let bones = (0..8)
        .map(|i| Bone {
            name: format!("b{i}"),
            parent: None,
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
        })
        .collect();
    a.rig = Some(Arc::new(Rig {
        bones,
        weights: vec![
            Weight {
                extra: None,
                joints: [0; 4],
                weights: [1., 0., 0., 0.]
            };
            32
        ],
    }));
    let cut = mesh_bridge::remove_faces(&a, &cut_faces()).unwrap();
    let out = mesh_bridge::bridge(&cut, &loops(), 4, [0.3, 0.3]).unwrap();
    assert_eq!(out.rig.as_ref().unwrap().weights.len(), 56);
    assert_eq!(
        &out.rig.as_ref().unwrap().weights[..32],
        &a.rig.as_ref().unwrap().weights
    );
    let mut mixed = cut;
    for i in 8..24 {
        Arc::make_mut(mixed.rig.as_mut().unwrap()).weights[i] = Weight {
            extra: None,
            joints: if i < 16 { [0, 1, 2, 3] } else { [4, 5, 6, 7] },
            weights: [0.25; 4],
        };
    }
    let bridged = mesh_bridge::bridge(&mixed, &loops(), 4, [0.3, 0.3]).unwrap();
    let weights = &bridged.rig.as_ref().unwrap().weights;
    assert_eq!(&weights[..32], &mixed.rig.as_ref().unwrap().weights);
    assert!(
        weights[32..]
            .iter()
            .any(|w| w.influences().filter(|(_, v)| *v > 0.).count() == 8)
    );
}
#[test]
fn dispatcher_preview_queries_and_errors_preserve_execution_history_and_plants() {
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::PutAsset { asset: source() },
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Step { ticks: 9 }.into()).unwrap();
    let before = e.snapshot();
    let reference = serde_json::from_value(
        e.execute(
            Command::InspectMeshBoundaries {
                asset: "tube".into(),
            }
            .into(),
        )
        .unwrap()["reference"]
            .clone(),
    )
    .unwrap();
    let edit = Edit::RemoveFaces {
        asset: "tube".into(),
        reference,
        faces: cut_faces(),
    };
    e.execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(e.snapshot(), before);
    e.execute(Command::EditMesh { edit: edit.clone() }.into())
        .unwrap();
    assert!(e.execute(Command::EditMesh { edit }.into()).is_err());
    let reference = serde_json::from_value(
        e.execute(
            Command::InspectMeshBoundaries {
                asset: "tube".into(),
            }
            .into(),
        )
        .unwrap()["reference"]
            .clone(),
    )
    .unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::BridgeBoundaryLoops {
                asset: "tube".into(),
                reference,
                loops: loops(),
                spans: 4,
                handles: [0.3, 0.3],
            },
        }
        .into(),
    )
    .unwrap();
    let final_scene = e.scene.clone();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        mesh_bridge::boundaries(&e.scene.meshes.assets[0].geometry)
            .unwrap()
            .len(),
        2
    );
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, final_scene);
    let replay = e.replay();
    assert_eq!(Engine::replay_at(&replay, None).unwrap().scene, e.scene);
}

#[test]
fn cuts_compact_unreferenced_points_and_weights_in_original_order() {
    let mut a = source();
    let mut g = (*a.geometry).clone();
    g.positions
        .extend([[5., 0., 0.], [6., 0., 0.], [5., 1., 0.]]);
    g.material_coordinates
        .as_mut()
        .unwrap()
        .extend([[7.; 3], [8.; 3], [9.; 3]]);
    g.triangles.push([32, 33, 34]);
    a.geometry = Arc::new(g);
    let out = mesh_bridge::remove_faces(&a, &[56]).unwrap();
    assert_eq!(out, source());
    // A cyclic phase may obey edge winding while its finite loft still folds.
    let cut = mesh_bridge::remove_faces(&source(), &cut_faces()).unwrap();
    let mut crossed = loops();
    crossed[1].rotate_left(4);
    assert!(mesh_bridge::bridge(&cut, &crossed, 2, [0.0001; 2]).is_err());
}
