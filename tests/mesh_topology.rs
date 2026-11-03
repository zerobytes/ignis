use glam::Vec3;
use ignis::{
    engine::{Command, Engine, SceneEdit},
    mesh_topology::{Components, Plan, Reference, Selection, Topology},
    meshes,
    model::RoomRecipe,
    plants,
};
use std::sync::Arc;

fn engine() -> Engine {
    Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap()
}
fn asset(id: &str, geometry: meshes::Geometry) -> meshes::Asset {
    meshes::Asset {
        material: Default::default(),
        shading: Default::default(),
        id: id.into(),
        geometry: Arc::new(geometry),
        rig: None,
        clips: vec![],
    }
}
fn edit(e: &mut Engine, edit: meshes::Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}
fn grid(e: &mut Engine, n: u16) {
    edit(
        e,
        meshes::Edit::Grid {
            id: "surface".into(),
            segments: n,
            size: 2.,
        },
    );
}
fn reference(e: &mut Engine, id: &str) -> Reference {
    serde_json::from_value(
        e.execute(Command::InspectMeshTopology { asset: id.into() }.into())
            .unwrap()["reference"]
            .clone(),
    )
    .unwrap()
}
fn subdivision(e: &mut Engine, id: &str, components: Option<Components>) -> meshes::Edit {
    meshes::Edit::Subdivide {
        asset: id.into(),
        reference: reference(e, id),
        components,
    }
}
fn selection(e: &mut Engine, components: Components) -> Selection {
    Selection {
        asset: "surface".into(),
        reference: reference(e, "surface"),
        components,
    }
}
fn close(a: [f32; 3], b: [f32; 3]) {
    assert!(
        Vec3::from(a).distance(Vec3::from(b)) < 2e-5,
        "{a:?} != {b:?}"
    );
}
fn triangle() -> meshes::Geometry {
    meshes::Geometry {
        material_coordinates: None,
        positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        triangles: vec![[0, 1, 2]],
    }
}

#[test]
fn every_edge_split_pattern_preserves_area_and_orientation() {
    let source = triangle();
    for mask in 1u8..8 {
        let components = Components::Edges {
            vertices: [[0, 1], [1, 2], [0, 2]]
                .into_iter()
                .enumerate()
                .filter_map(|(i, e)| (mask & (1 << i) != 0).then_some(e))
                .collect(),
        };
        let plan = Plan::new(&source, Some(&components)).unwrap();
        let out = plan.geometry(&source).unwrap();
        assert_eq!(out.triangles.len(), 1 + mask.count_ones() as usize);
        assert_eq!(out.positions.len(), 3 + mask.count_ones() as usize);
        let mut area = 0.;
        for t in &out.triangles {
            let [a, b, c] = t.map(|i| Vec3::from(out.positions[i as usize]));
            let cross = (b - a).cross(c - a);
            assert!(cross.z > 0.);
            area += cross.length() * 0.5;
        }
        assert!((area - 0.5).abs() < 1e-6);
    }
}
#[test]
fn full_subdivision_shares_midpoints_and_retains_old_vertices() {
    let base = meshes::Geometry::grid(1, 2.).unwrap();
    let plan = Plan::new(&base, None).unwrap();
    let refined = plan.geometry(&base).unwrap();
    assert_eq!((refined.positions.len(), refined.triangles.len()), (9, 8));
    assert_eq!(&refined.positions[..4], &base.positions);
    let topology = Topology::inspect(&refined).unwrap();
    assert_eq!(topology.boundary_edges, 8);
    assert_eq!(topology.non_manifold_edges, 0);
    assert_eq!(plan.parents(), vec![0, 0, 0, 0, 1, 1, 1, 1]);
}
#[test]
fn selected_face_refines_neighbor_conformantly_without_t_junction() {
    let base = meshes::Geometry::grid(1, 2.).unwrap();
    let plan = Plan::new(&base, Some(&Components::Faces { indices: vec![0] })).unwrap();
    let out = plan.geometry(&base).unwrap();
    assert_eq!((out.positions.len(), out.triangles.len()), (7, 6));
    let middle = 4 + plan
        .split_edges()
        .iter()
        .position(|e| *e == [1, 2])
        .unwrap() as u32;
    let topology = Topology::inspect(&out).unwrap();
    assert!(!topology.edges.iter().any(|e| e.vertices == [1, 2]));
    for endpoints in [[1, middle], [2, middle]] {
        assert_eq!(
            topology
                .edges
                .iter()
                .find(|e| e.vertices == endpoints)
                .unwrap()
                .faces
                .len(),
            2
        );
    }
}
#[test]
fn vertex_and_edge_selections_split_incident_edges() {
    let base = meshes::Geometry::grid(1, 2.).unwrap();
    let vertex = Plan::new(&base, Some(&Components::Vertices { indices: vec![0] })).unwrap();
    assert_eq!((vertex.vertex_count(), vertex.triangles().len()), (6, 4));
    let edge = Plan::new(
        &base,
        Some(&Components::Edges {
            vertices: vec![[2, 1]],
        }),
    )
    .unwrap();
    assert_eq!((edge.vertex_count(), edge.triangles().len()), (5, 4));
}
#[test]
fn seams_nonmanifold_edges_and_isolated_vertices_are_explicit() {
    let mut seam = triangle();
    seam.positions.extend(triangle().positions);
    seam.triangles.push([3, 4, 5]);
    let topology = Topology::inspect(&seam).unwrap();
    assert_eq!(topology.boundary_edges, 6);
    assert_eq!(Plan::new(&seam, None).unwrap().vertex_count(), 12);
    let nonman = meshes::Geometry {
        material_coordinates: None,
        positions: vec![
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., -1., 0.],
            [0., 0., 1.],
            [9., 9., 9.],
        ],
        triangles: vec![[0, 1, 2], [1, 0, 3], [0, 1, 4]],
    };
    let topology = Topology::inspect(&nonman).unwrap();
    assert_eq!(topology.non_manifold_edges, 1);
    assert_eq!(topology.isolated_vertices, vec![5]);
    assert!(Plan::new(&nonman, Some(&Components::Vertices { indices: vec![5] })).is_err());
    let refined = Plan::new(&nonman, None).unwrap().geometry(&nonman).unwrap();
    assert_eq!(Topology::inspect(&refined).unwrap().non_manifold_edges, 2);
}
#[test]
fn bad_components_plans_and_growth_limits_reject() {
    let base = triangle();
    let topology = Topology::inspect(&base).unwrap();
    for c in [
        Components::Faces { indices: vec![] },
        Components::Faces { indices: vec![1] },
        Components::Vertices {
            indices: vec![1, 1],
        },
        Components::Edges {
            vertices: vec![[0, 1], [1, 0]],
        },
        Components::Edges {
            vertices: vec![[0, 0]],
        },
    ] {
        assert!(topology.canonicalize(&c).is_err());
    }
    let plan = Plan::new(&base, None).unwrap();
    let mut changed = base.clone();
    changed.triangles[0].swap(1, 2);
    assert!(plan.geometry(&changed).is_err());
    let max = meshes::Geometry::grid(63, 2.).unwrap();
    let refined = Plan::new(&max, None).unwrap().geometry(&max).unwrap();
    assert_eq!(refined.positions.len(), 16129);
    assert!(Plan::new(&refined, None).is_err());
}

#[test]
fn selection_is_canonical_session_state_without_gameplay_or_history_changes() {
    let mut e = engine();
    grid(&mut e, 1);
    e.execute(Command::Play.into()).unwrap();
    let before = e.execute(Command::Inspect.into()).unwrap();
    let selected = selection(
        &mut e,
        Components::Vertices {
            indices: vec![3, 0],
        },
    );
    let result = e
        .execute(
            Command::SelectMeshComponents {
                selection: Some(selected),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(
        result["selection"]["components"]["indices"],
        serde_json::json!([0, 3])
    );
    let mut after = e.execute(Command::Inspect.into()).unwrap();
    let mut expected = before.clone();
    for value in [&mut after, &mut expected] {
        value.as_object_mut().unwrap().remove("mesh_selection");
        value.as_object_mut().unwrap().remove("view_revision");
    }
    assert_eq!(after, expected);
    let selected: Selection = serde_json::from_value(result["selection"].clone()).unwrap();
    assert_eq!(
        e.execute(
            Command::SelectMeshComponents {
                selection: Some(selected)
            }
            .into()
        )
        .unwrap(),
        result
    );
    assert!(
        serde_json::to_value(&e.scene)
            .unwrap()
            .get("mesh_selection")
            .is_none()
    );
    let mut bad = selection(&mut e, Components::Faces { indices: vec![999] });
    assert!(
        e.execute(
            Command::SelectMeshComponents {
                selection: Some(bad.clone())
            }
            .into()
        )
        .is_err()
    );
    bad.reference.scope = "different-session".into();
    assert_eq!(
        e.execute(
            Command::SelectMeshComponents {
                selection: Some(bad)
            }
            .into()
        )
        .unwrap_err()
        .code,
        "topology_conflict"
    );
    assert_eq!(
        e.execute(Command::InspectMeshSelection.into()).unwrap(),
        result
    );
    e.execute(Command::SelectMeshComponents { selection: None }.into())
        .unwrap();
    assert!(e.execute(Command::InspectMeshSelection.into()).unwrap()["selection"].is_null());
}
#[test]
fn old_references_fail_after_reset_edit_undo_and_new_engine() {
    let mut e = engine();
    grid(&mut e, 1);
    let selected = selection(&mut e, Components::Faces { indices: vec![0] });
    e.execute(
        Command::SelectMeshComponents {
            selection: Some(selected.clone()),
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.execute(Command::InspectMeshSelection.into()).unwrap()["selection"].is_null());
    assert_eq!(
        e.execute(
            Command::SelectMeshComponents {
                selection: Some(selected.clone())
            }
            .into()
        )
        .unwrap_err()
        .code,
        "topology_conflict"
    );
    let mut fresh = Engine::new(e.scene.clone()).unwrap();
    assert_eq!(
        fresh
            .execute(
                Command::SelectMeshComponents {
                    selection: Some(selected)
                }
                .into()
            )
            .unwrap_err()
            .code,
        "topology_conflict"
    );
    let old = subdivision(&mut e, "surface", None);
    edit(&mut e, old.clone());
    let before = e.execute(Command::Inspect.into()).unwrap();
    assert_eq!(
        e.execute(Command::EditMesh { edit: old.clone() }.into())
            .unwrap_err()
            .code,
        "topology_conflict"
    );
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    e.execute(Command::Undo.into()).unwrap();
    assert!(e.execute(Command::EditMesh { edit: old }.into()).is_err());
}
#[test]
fn subdivision_is_atomic_and_undo_redo_restore_exact_assets() {
    let mut e = engine();
    grid(&mut e, 1);
    let before = e.scene.clone();
    let selected = selection(
        &mut e,
        Components::Edges {
            vertices: vec![[1, 2]],
        },
    );
    e.execute(
        Command::SelectMeshComponents {
            selection: Some(selected.clone()),
        }
        .into(),
    )
    .unwrap();
    edit(
        &mut e,
        meshes::Edit::Subdivide {
            asset: selected.asset,
            reference: selected.reference,
            components: Some(selected.components),
        },
    );
    assert!(e.execute(Command::InspectMeshSelection.into()).unwrap()["selection"].is_null());
    assert_eq!(e.scene.meshes.assets[0].geometry.triangles.len(), 4);
    let after = e.scene.clone();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, after);
    let restored = serde_json::from_str(&serde_json::to_string(&e.scene).unwrap()).unwrap();
    assert_eq!(Engine::new(restored).unwrap().scene, after);
    let snapshot = e.execute(Command::Inspect.into()).unwrap();
    let bad = subdivision(
        &mut e,
        "surface",
        Some(Components::Edges {
            vertices: vec![[0, 3]],
        }),
    );
    assert!(e.execute(Command::EditMesh { edit: bad }.into()).is_err());
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), snapshot);
}
#[test]
fn batches_reject_stale_intra_batch_references_and_can_refine_two_assets() {
    let mut e = engine();
    grid(&mut e, 1);
    let op = subdivision(&mut e, "surface", None);
    let before = e.execute(Command::Inspect.into()).unwrap();
    let result = e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Mesh {
                    edit: meshes::Edit::Grid {
                        id: "surface".into(),
                        segments: 2,
                        size: 2.,
                    },
                },
                SceneEdit::Mesh { edit: op },
            ],
        }
        .into(),
    );
    assert!(result.is_err());
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    edit(
        &mut e,
        meshes::Edit::Grid {
            id: "other".into(),
            segments: 1,
            size: 2.,
        },
    );
    let a = subdivision(&mut e, "surface", None);
    let b = subdivision(&mut e, "other", None);
    e.execute(
        Command::EditScene {
            edits: vec![SceneEdit::Mesh { edit: a }, SceneEdit::Mesh { edit: b }],
        }
        .into(),
    )
    .unwrap();
    assert!(
        e.scene
            .meshes
            .assets
            .iter()
            .all(|a| a.geometry.positions.len() == 9)
    );
    e.execute(Command::Undo.into()).unwrap();
    assert!(
        e.scene
            .meshes
            .assets
            .iter()
            .all(|a| a.geometry.positions.len() == 4)
    );
}
#[test]
fn subdivision_is_not_limited_by_other_assets_or_instance_count() {
    let mut e = engine();
    let base = meshes::Geometry::grid(44, 2.).unwrap();
    let large = Plan::new(&base, None).unwrap().geometry(&base).unwrap();
    for i in 0..4 {
        edit(
            &mut e,
            meshes::Edit::PutAsset {
                asset: asset(&format!("large{i}"), large.clone()),
            },
        );
    }
    grid(&mut e, 21);
    let op = subdivision(&mut e, "surface", None);
    e.execute(Command::EditMesh { edit: op }.into()).unwrap();
    let mut e = engine();
    grid(&mut e, 24);
    //1152triangles ×4children ×29instances=133632, above131072.
    for i in 0..29 {
        edit(
            &mut e,
            meshes::Edit::PutInstance {
                instance: meshes::Instance {
                    id: format!("i{i}"),
                    asset: "surface".into(),
                    position: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                    scale: 1.,
                    color: [1.; 3],
                    attachment: None,
                    playback: None,
                },
            },
        );
    }
    let op = subdivision(&mut e, "surface", None);
    e.execute(Command::EditMesh { edit: op }.into()).unwrap();
    assert_eq!(e.render_frame().meshes.triangle_count(), 133632);
}
#[test]
fn original_vertices_keep_animated_poses_and_clips_are_shared() {
    let mut source = asset("surface", meshes::Geometry::grid(1, 2.).unwrap());
    source.rig = Some(Arc::new(meshes::Rig {
        bones: vec![meshes::Bone {
            name: "root".into(),
            parent: None,
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
        }],
        weights: vec![
            meshes::Weight {
                extra: None,
                joints: [0; 4],
                weights: [1., 0., 0., 0.]
            };
            4
        ],
    }));
    source.clips = vec![Arc::new(meshes::Clip {
        id: "bend".into(),
        duration_ticks: 60,
        tracks: vec![meshes::Track {
            bone: 0,
            keys: vec![
                meshes::Keyframe {
                    tick: 0,
                    translation: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                },
                meshes::Keyframe {
                    tick: 60,
                    translation: [0.1, 0., 0.],
                    rotation: [0., 0., 0.38268343, 0.9238795],
                },
            ],
        }],
    })];
    let plan = Plan::new(&source.geometry, None).unwrap();
    let target = plan.asset(&source).unwrap();
    assert!(Arc::ptr_eq(&source.clips[0], &target.clips[0]));
    assert_eq!(
        source.rig.as_ref().unwrap().bones,
        target.rig.as_ref().unwrap().bones
    );
    let instance = meshes::Instance {
        id: "actor".into(),
        asset: "surface".into(),
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [1.; 3],
        attachment: None,
        playback: Some(meshes::Playback {
            clip: "bend".into(),
            offset_ticks: 0,
            looping: false,
        }),
    };
    let old = meshes::Document {
        objects: Default::default(),
        assets: vec![source],
        instances: vec![instance.clone()],
    }
    .pose("actor", 30)
    .unwrap();
    let new = meshes::Document {
        objects: Default::default(),
        assets: vec![target],
        instances: vec![instance],
    }
    .pose("actor", 30)
    .unwrap();
    assert_eq!(&new.positions[..4], &old.positions);
    for (i, &[a, b]) in plan.split_edges().iter().enumerate() {
        close(
            new.positions[4 + i],
            ((Vec3::from(old.positions[a as usize]) + Vec3::from(old.positions[b as usize])) * 0.5)
                .to_array(),
        );
    }
}
#[test]
fn extra_bone_influences_transfer_without_changing_source() {
    let mut source = asset("surface", triangle());
    source.rig = Some(Arc::new(meshes::Rig {
        bones: (0..6)
            .map(|i| meshes::Bone {
                name: format!("b{i}"),
                parent: None,
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            })
            .collect(),
        weights: vec![
            meshes::Weight {
                extra: None,
                joints: [0, 1, 2, 3],
                weights: [0.25; 4],
            },
            meshes::Weight {
                extra: None,
                joints: [2, 3, 4, 5],
                weights: [0.25; 4],
            },
            meshes::Weight {
                extra: None,
                joints: [0; 4],
                weights: [1., 0., 0., 0.],
            },
        ],
    }));
    let original = source.clone();
    let refined = Plan::new(&source.geometry, None)
        .unwrap()
        .asset(&source)
        .unwrap();
    assert!(
        refined
            .rig
            .as_ref()
            .unwrap()
            .weights
            .iter()
            .any(|w| w.extra.is_some())
    );
    meshes::Document {
        objects: Default::default(),
        assets: vec![refined],
        instances: vec![],
    }
    .validate()
    .unwrap();
    assert_eq!(source, original);
}

#[test]
fn reused_plan_rejects_an_incompatible_rigged_asset_before_weight_access() {
    let grid = meshes::Geometry::grid(1, 2.).unwrap();
    let plan = Plan::new(&grid, None).unwrap();
    let mut source = asset("surface", triangle());
    source.rig = Some(Arc::new(meshes::Rig {
        bones: vec![meshes::Bone {
            name: "root".into(),
            parent: None,
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
        }],
        weights: vec![
            meshes::Weight {
                extra: None,
                joints: [0; 4],
                weights: [1., 0., 0., 0.]
            };
            3
        ],
    }));
    let before = source.clone();
    assert!(
        plan.asset(&source)
            .unwrap_err()
            .message
            .contains("source topology changed")
    );
    assert_eq!(source, before);
}

fn definition(levels: u8) -> plants::Definition {
    plants::Definition {
        trajectory: None,
        id: "limb".into(),
        profile: plants::Rectangle {
            id: "outline".into(),
            width: 2.,
            height: 4.,
        },
        depth: 0.5,
        segments: [1; 3],
        sections: None,
        subdivision_levels: levels,
        sampling_tolerance: 1. / 1024.,
    }
}
fn put(e: &mut Engine, definition: plants::Definition) -> ignis::model::Result<serde_json::Value> {
    e.execute(
        Command::EditPlant {
            edit: plants::Edit::Put {
                definition,
                detail_policy: ignis::mesh_preservation::DisplacementPolicy::Physical,
                animation_translation_policy:
                    ignis::mesh_preservation::DisplacementPolicy::Physical,
            },
        }
        .into(),
    )
}
#[test]
fn all_refined_plant_vertices_have_surface_provenance_and_correct_barycentrics() {
    for from in 0..=2 {
        for to in 0..=2 {
            let mut a = definition(from);
            a.segments = [1, 2, 1];
            let source = a.sample().unwrap();
            let mut b = definition(to);
            b.segments = [2, 3, 2];
            let target = b.sample().unwrap();
            assert_eq!(source.surface.len(), source.geometry.positions.len());
            assert_eq!(target.surface.len(), target.geometry.positions.len());
            assert_eq!(
                a.counts().unwrap(),
                (
                    source.geometry.positions.len(),
                    source.geometry.triangles.len()
                )
            );
            for (&point, &surface) in target.material.iter().zip(&target.surface) {
                let (triangle, bary) = source.correspondence(surface as usize, point);
                let actual = source.geometry.triangles[triangle as usize]
                    .into_iter()
                    .zip(bary)
                    .fold(Vec3::ZERO, |sum, (i, w)| {
                        sum + Vec3::from(source.material[i as usize]) * w
                    });
                close(actual.to_array(), point);
                assert!(bary.iter().all(|w| *w >= -1e-5 && *w <= 1. + 1e-5));
            }
            for triangle in &source.geometry.triangles {
                let [a, b, c] = triangle.map(|i| Vec3::from(source.geometry.positions[i as usize]));
                assert!((b - a).cross(c - a).dot(a + b + c) > 0.);
                assert!(
                    triangle.iter().all(
                        |i| source.surface[*i as usize] == source.surface[triangle[0] as usize]
                    )
                );
            }
        }
    }
}
#[test]
fn uniform_subdivision_of_linked_plant_keeps_canonical_source_and_undo() {
    let mut e = engine();
    put(&mut e, definition(0)).unwrap();
    let source = e.scene.plants.entries[0].source.clone();
    let original = e.scene.clone();
    for level in 1..=2 {
        let op = subdivision(&mut e, "limb", None);
        edit(&mut e, op);
        assert_eq!(
            e.scene.plants.entries[0].definition.subdivision_levels,
            level
        );
        assert!(Arc::ptr_eq(&source, &e.scene.plants.entries[0].source));
        e.scene.validate().unwrap();
    }
    assert_eq!(
        e.scene
            .meshes
            .asset("limb")
            .unwrap()
            .geometry
            .positions
            .len(),
        150
    );
    put(&mut e, definition(0)).unwrap();
    assert_eq!(e.scene.meshes, original.meshes);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene.plants.entries[0].definition.subdivision_levels, 2);
    let bad = subdivision(&mut e, "limb", Some(Components::Faces { indices: vec![0] }));
    let before = e.scene.clone();
    assert!(e.execute(Command::EditMesh { edit: bad }.into()).is_err());
    assert_eq!(e.scene, before);
    let local = subdivision(&mut e, "limb", Some(Components::Faces { indices: vec![0] }));
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Plant {
                    edit: plants::Edit::Detach { id: "limb".into() },
                },
                SceneEdit::Mesh { edit: local },
            ],
        }
        .into(),
    )
    .unwrap();
    assert!(e.scene.plants.is_empty());
}
#[test]
fn detail_only_on_a_new_midpoint_cannot_disappear_during_coarsening() {
    let mut e = engine();
    put(&mut e, definition(1)).unwrap();
    let sample = definition(1).sample().unwrap();
    let index = sample
        .material
        .iter()
        .enumerate()
        .find(|(i, p)| *i >= 24 && sample.surface[*i] == 5 && p[0] == 0. && p[1] == 0.)
        .unwrap()
        .0;
    let point = sample.geometry.positions[index];
    edit(
        &mut e,
        meshes::Edit::Sculpt {
            asset: "limb".into(),
            stroke: meshes::Stroke {
                kind: meshes::BrushKind::Grab,
                centers: vec![point],
                radius: 0.01,
                strength: 0.05,
                direction: [0., 0., 1.],
            },
        },
    );
    assert_eq!(e.scene.plants.entries[0].reference.subdivision_levels, 1);
    let before = e.scene.clone();
    let error = put(&mut e, definition(0)).unwrap_err();
    assert!(error.message.contains("resolution insufficient"));
    assert_eq!(e.scene, before);
    let mut finer = definition(2);
    finer.profile.width = 3.;
    finer.profile.height = 6.;
    finer.depth = 1.;
    put(&mut e, finer).unwrap();
    put(&mut e, definition(1)).unwrap();
    assert_eq!(e.scene.meshes, before.meshes);
    let mut coarse = definition(0);
    coarse.sampling_tolerance = 0.1;
    put(&mut e, coarse).unwrap();
    assert_eq!(e.scene.plants.entries[0].reference.subdivision_levels, 1);
    let mut old = e.scene.clone();
    old.version = 49;
    assert!(old.validate().unwrap_err().message.contains("version50"));
    let loaded = serde_json::from_str(&serde_json::to_string(&e.scene).unwrap()).unwrap();
    assert_eq!(Engine::new(loaded).unwrap().scene, e.scene);
}
#[test]
fn plant_refinement_budgets_and_hidden_old_versions_reject() {
    for level in [6, 7, u8::MAX] {
        assert!(definition(level).sample().is_err());
    }
    let mut d = definition(1);
    d.segments = [u32::MAX; 3];
    assert!(d.sample().is_err());
    let mut e = engine();
    put(&mut e, definition(1)).unwrap();
    let mut old = e.scene.clone();
    old.version = 49;
    assert!(old.validate().is_err());
    let before = e.scene.clone();
    let mut huge = definition(5);
    huge.segments = [63; 3];
    assert!(put(&mut e, huge).is_err());
    assert_eq!(e.scene, before);
}
