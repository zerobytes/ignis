use glam::DVec3;
use ignis::{
    engine::Command,
    mesh_modeling::{self as modeling, Operation},
    mesh_tools::{self, Primitive},
    meshes::{self, Asset},
    model::RoomRecipe,
    service::Service,
};
fn asset(kind: Primitive, resolution: u16) -> Asset {
    Asset {
        id: "model".into(),
        geometry: std::sync::Arc::new(mesh_tools::primitive(kind, resolution, 2.).unwrap()),
        shading: Default::default(),
        material: Default::default(),
        rig: None,
        clips: vec![],
    }
}
fn normal(a: &Asset, face: &[u32; 3]) -> DVec3 {
    let [p, q, r] =
        face.map(|i| DVec3::from_array(a.geometry.positions[i as usize].map(f64::from)));
    (q - p).cross(r - p).normalize()
}
fn top(a: &Asset) -> Vec<u32> {
    a.geometry
        .triangles
        .iter()
        .enumerate()
        .filter(|(_, f)| normal(a, f).y > 0.999)
        .map(|(i, _)| i as u32)
        .collect()
}
fn volume(a: &Asset) -> f64 {
    a.geometry
        .triangles
        .iter()
        .map(|f| {
            let [a, b, c] =
                f.map(|i| DVec3::from_array(a.geometry.positions[i as usize].map(f64::from)));
            a.dot(b.cross(c)) / 6.
        })
        .sum()
}
fn closed(a: &Asset) {
    let t = ignis::mesh_topology::Topology::inspect(&a.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);
    assert!(t.edges.iter().all(|e| e.faces.len() == 2));
    assert!(t.isolated_vertices.is_empty());
    assert!(volume(a) > 0.);
}
fn corner_edge(a: &Asset) -> [u32; 2] {
    ignis::mesh_topology::Topology::inspect(&a.geometry)
        .unwrap()
        .edges
        .into_iter()
        .find(|e| {
            let p = a.geometry.positions[e.vertices[0] as usize];
            let q = a.geometry.positions[e.vertices[1] as usize];
            p[0] > 0.999 && q[0] > 0.999 && p[1] > 0.999 && q[1] > 0.999
        })
        .unwrap()
        .vertices
}
#[test]
fn extrude_region_preserves_closed_solid_and_volume_without_old_cap() {
    for resolution in [4, 24] {
        let a = asset(Primitive::Cube, resolution);
        let faces = top(&a);
        let result = modeling::evaluate(
            &a,
            &Operation::Extrude {
                faces: faces.clone(),
                offset: [0., 0.3, 0.],
            },
        )
        .unwrap();
        closed(&result.asset);
        assert!((volume(&result.asset) - 9.2).abs() < 1e-5);
        assert_eq!(result.report.selected_faces.len(), faces.len());
        assert!(result.report.triangles > a.geometry.triangles.len());
        assert!(
            result
                .asset
                .geometry
                .triangles
                .iter()
                .enumerate()
                .filter(|(i, _)| result.report.selected_faces.contains(&(*i as u32)))
                .all(|(_, f)| f
                    .iter()
                    .all(|i| (result.asset.geometry.positions[*i as usize][1] - 1.3).abs() < 1e-5))
        );
        assert!(result.asset.geometry.triangles.iter().all(|f| {
            !f.iter()
                .all(|i| (result.asset.geometry.positions[*i as usize][1] - 1.).abs() < 1e-6)
        }));
    }
}
#[test]
fn inset_keeps_volume_and_constant_width_on_subdivided_face_and_can_be_extruded_again() {
    for resolution in [4, 24] {
        let a = asset(Primitive::Cube, resolution);
        let r = modeling::evaluate(
            &a,
            &Operation::Inset {
                faces: top(&a),
                width: 0.2,
            },
        )
        .unwrap();
        closed(&r.asset);
        assert!((volume(&r.asset) - 8.).abs() < 1e-6);
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for f in &r.report.selected_faces {
            for &i in &r.asset.geometry.triangles[*f as usize] {
                for axis in 0..3 {
                    min[axis] = min[axis].min(r.asset.geometry.positions[i as usize][axis]);
                    max[axis] = max[axis].max(r.asset.geometry.positions[i as usize][axis]);
                }
            }
        }
        for axis in [0, 2] {
            assert!((min[axis] + 0.8).abs() < 1e-6);
            assert!((max[axis] - 0.8).abs() < 1e-6);
        }
        let r = modeling::evaluate(
            &r.asset,
            &Operation::Extrude {
                faces: r.report.selected_faces,
                offset: [0., -0.25, 0.],
            },
        )
        .unwrap();
        closed(&r.asset);
        assert!((volume(&r.asset) - (8. - 1.6 * 1.6 * 0.25)).abs() < 1e-5);
    }
}
#[test]
fn bevel_explicitly_expands_collinear_chains_and_closes_shared_cut_intersections() {
    for resolution in [4, 24] {
        let a = asset(Primitive::Cube, resolution);
        let e = corner_edge(&a);
        assert!(
            modeling::evaluate(
                &a,
                &Operation::Bevel {
                    edges: vec![e],
                    width: 0.2,
                    extend_collinear: false
                }
            )
            .is_err()
        );
        let r = modeling::evaluate(
            &a,
            &Operation::Bevel {
                edges: vec![e],
                width: 0.2,
                extend_collinear: true,
            },
        )
        .unwrap();
        closed(&r.asset);
        assert_eq!(r.report.effective_edges.len(), resolution as usize);
        assert!(!r.report.selected_faces.is_empty());
        assert!((volume(&r.asset) - 7.96).abs() < 1e-5);
        assert!(
            r.asset
                .geometry
                .positions
                .iter()
                .all(|p| p[0] + p[1] <= 1.80001)
        );
    }
}
#[test]
fn adjacent_bevels_share_miters_and_oversized_or_flat_or_open_requests_fail() {
    let a = asset(Primitive::Cube, 4);
    let edges = ignis::mesh_topology::Topology::inspect(&a.geometry)
        .unwrap()
        .edges
        .into_iter()
        .filter(|e| {
            let n = normal(&a, &a.geometry.triangles[e.faces[0] as usize]);
            let m = normal(&a, &a.geometry.triangles[e.faces[1] as usize]);
            n.dot(m) < 0.99
        })
        .map(|e| e.vertices)
        .collect::<Vec<_>>();
    let r = modeling::evaluate(
        &a,
        &Operation::Bevel {
            edges: edges.clone(),
            width: 0.15,
            extend_collinear: false,
        },
    )
    .unwrap();
    closed(&r.asset);
    assert!(volume(&r.asset) < 8.);
    assert!(
        modeling::evaluate(
            &a,
            &Operation::Bevel {
                edges: edges.clone(),
                width: 3.,
                extend_collinear: false
            }
        )
        .is_err()
    );
    let plane = asset(Primitive::Plane, 4);
    assert!(
        modeling::evaluate(
            &plane,
            &Operation::Bevel {
                edges: vec![[0, 1]],
                width: 0.1,
                extend_collinear: true
            }
        )
        .is_err()
    );
    let flat = ignis::mesh_topology::Topology::inspect(&a.geometry)
        .unwrap()
        .edges
        .into_iter()
        .find(|e| {
            normal(&a, &a.geometry.triangles[e.faces[0] as usize])
                .dot(normal(&a, &a.geometry.triangles[e.faces[1] as usize]))
                > 0.999
        })
        .unwrap()
        .vertices;
    assert!(
        modeling::evaluate(
            &a,
            &Operation::Bevel {
                edges: vec![flat],
                width: 0.1,
                extend_collinear: true
            }
        )
        .is_err()
    );
}
#[test]
fn invalid_regions_widths_and_offsets_do_not_mutate_source() {
    let a = asset(Primitive::Cube, 4);
    let frozen = a.clone();
    for op in [
        Operation::Inset {
            faces: top(&a),
            width: 2.,
        },
        Operation::Inset {
            faces: vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
            ],
            width: 0.1,
        },
        Operation::Extrude {
            faces: vec![0, 0],
            offset: [0., 0.2, 0.],
        },
        Operation::Extrude {
            faces: top(&a),
            offset: [0.; 3],
        },
        Operation::Extrude {
            faces: vec![u32::MAX],
            offset: [0., 0.2, 0.],
        },
    ] {
        assert!(modeling::evaluate(&a, &op).is_err());
        assert_eq!(a, frozen);
    }
}
#[test]
fn shared_edit_guards_revision_and_preserves_history_undo_redo_and_materials() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let a = asset(Primitive::Cube, 4);
    s.execute(
        Command::EditMesh {
            edit: meshes::Edit::PutAsset { asset: a.clone() },
        }
        .into(),
    )
    .unwrap();
    let before = s.engine.snapshot();
    let reference = s.engine.mesh_reference();
    let edit = meshes::Edit::ModelSelection {
        asset: a.id.clone(),
        reference: reference.clone(),
        operation: Operation::Inset {
            faces: top(&a),
            width: 0.2,
        },
    };
    let preview = s
        .execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
        .unwrap()
        .data;
    assert_eq!(s.engine.snapshot(), before);
    s.execute(Command::EditMesh { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(
        serde_json::to_value(&s.engine.scene.meshes).unwrap(),
        preview["meshes"]
    );
    assert_eq!(s.engine.scene.meshes.assets[0].material, a.material);
    let after = s.engine.snapshot();
    assert!(s.execute(Command::EditMesh { edit }.into()).is_err());
    assert_eq!(s.engine.snapshot(), after);
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(s.engine.snapshot()["scene"], before["scene"]);
    s.execute(Command::Redo.into()).unwrap();
    assert_eq!(s.engine.snapshot()["scene"], after["scene"]);
}

#[test]
fn topology_modeling_preserves_rig_clips_and_interpolates_material_coordinates_without_pruning() {
    let mut a = asset(Primitive::Cube, 4);
    let count = a.geometry.positions.len();
    std::sync::Arc::make_mut(&mut a.geometry).material_coordinates = Some(
        a.geometry
            .positions
            .iter()
            .map(|p| p.map(|v| v * 2. + 0.25))
            .collect(),
    );
    let bones = (0..2)
        .map(|i| meshes::Bone {
            name: format!("bone_{i}"),
            parent: if i == 0 { None } else { Some(0) },
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
        })
        .collect();
    a.rig = Some(std::sync::Arc::new(meshes::Rig {
        bones,
        weights: a
            .geometry
            .positions
            .iter()
            .map(|p| {
                let x = (p[0] + 1.) * 0.5;
                meshes::Weight {
                    extra: None,
                    joints: [0, 1, 0, 0],
                    weights: [x, 1. - x, 0., 0.],
                }
            })
            .collect(),
    }));
    a.clips = vec![std::sync::Arc::new(meshes::Clip {
        id: "idle".into(),
        duration_ticks: 60,
        tracks: vec![meshes::Track {
            bone: 0,
            keys: vec![meshes::Keyframe {
                tick: 0,
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            }],
        }],
    })];
    let r = modeling::evaluate(
        &a,
        &Operation::Bevel {
            edges: vec![corner_edge(&a)],
            width: 0.2,
            extend_collinear: true,
        },
    )
    .unwrap();
    closed(&r.asset);
    assert_eq!(r.asset.clips, a.clips);
    assert_eq!(
        r.asset.rig.as_ref().unwrap().bones,
        a.rig.as_ref().unwrap().bones
    );
    assert_eq!(
        r.asset.rig.as_ref().unwrap().weights.len(),
        r.asset.geometry.positions.len()
    );
    for (i, p) in r.asset.geometry.positions.iter().enumerate() {
        for (axis, value) in p.iter().enumerate() {
            assert!(
                (r.asset.geometry.material_coordinates.as_ref().unwrap()[i][axis]
                    - (value * 2. + 0.25))
                    .abs()
                    < 1e-5
            );
        }
        let w = &r.asset.rig.as_ref().unwrap().weights[i];
        let expected = (p[0] + 1.) * 0.5;
        let actual = w
            .joints
            .iter()
            .zip(&w.weights)
            .filter(|(j, _)| **j == 0)
            .map(|(_, w)| *w)
            .sum::<f32>();
        assert!((actual - expected).abs() < 1e-5);
    }
    let rig = std::sync::Arc::make_mut(a.rig.as_mut().unwrap());
    rig.bones = (0..6)
        .map(|i| meshes::Bone {
            name: format!("bone_{i}"),
            parent: None,
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
        })
        .collect();
    rig.weights = (0..count)
        .map(|i| meshes::Weight {
            extra: None,
            joints: [(i % 6) as u8, 0, 0, 0],
            weights: [1., 0., 0., 0.],
        })
        .collect();
    let before = a.clone();
    let extended = modeling::evaluate(
        &a,
        &Operation::Bevel {
            edges: vec![corner_edge(&a)],
            width: 0.2,
            extend_collinear: true,
        },
    )
    .unwrap();
    assert!(
        extended
            .asset
            .rig
            .as_ref()
            .unwrap()
            .weights
            .iter()
            .any(|w| w.extra.is_some())
    );
    assert_eq!(a, before);
}
#[test]
fn editor_modeling_preview_commit_refreshes_selection_and_operation_errors_are_atomic() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: Primitive::Cube,
            resolution: 4,
            size: 2.,
        }
        .into(),
    )
    .unwrap();
    let i = s.engine.scene.meshes.instances[0].clone();
    let a = s.engine.scene.meshes.assets[0].clone();
    let reference = s.engine.mesh_reference();
    let op = Operation::Inset {
        faces: top(&a),
        width: 0.2,
    };
    let before = s.engine.snapshot();
    let preview = s
        .execute(
            Command::ModelEditorMesh {
                instance: i.id.clone(),
                reference: reference.clone(),
                operation: op.clone(),
                preview: true,
            }
            .into(),
        )
        .unwrap()
        .data;
    assert_eq!(s.engine.snapshot(), before);
    s.execute(
        Command::ModelEditorMesh {
            instance: i.id,
            reference,
            operation: op,
            preview: false,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.snapshot()["scene"], preview["scene"]);
    assert_eq!(
        s.engine.selected_mesh_components().unwrap().reference,
        s.engine.mesh_reference()
    );
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: Primitive::Cube,
            resolution: 26,
            size: 2.,
        }
        .into(),
    )
    .unwrap();
    let a = s.engine.scene.meshes.assets[0].clone();
    let original = s.engine.scene.meshes.instances[0].clone();
    for n in 1..16 {
        let mut i = original.clone();
        i.id = format!("copy_{n}");
        s.execute(
            Command::EditMesh {
                edit: meshes::Edit::PutInstance { instance: i },
            }
            .into(),
        )
        .unwrap();
    }
    let before = s.engine.snapshot();
    let edit = meshes::Edit::ModelSelection {
        asset: a.id.clone(),
        reference: s.engine.mesh_reference(),
        operation: Operation::Inset {
            faces: top(&a),
            // Scene103 removed the old aggregate draw ceiling. An invalid geometric
            // request still must preserve every shared instance and editor state.
            width: 10.0,
        },
    };
    assert!(s.execute(Command::EditMesh { edit }.into()).is_err());
    assert_eq!(s.engine.snapshot(), before);
}

#[test]
fn extrusion_preserves_holes_and_unrelated_isolated_vertices_while_inset_refuses_holes() {
    let mut a = asset(Primitive::Cube, 4);
    let faces: Vec<_> = top(&a)
        .into_iter()
        .filter(|i| {
            let center = a.geometry.triangles[*i as usize]
                .iter()
                .map(|j| DVec3::from_array(a.geometry.positions[*j as usize].map(f64::from)))
                .sum::<DVec3>()
                / 3.;
            center.x.abs() > 0.5 || center.z.abs() > 0.5
        })
        .collect();
    let result = modeling::evaluate(
        &a,
        &Operation::Extrude {
            faces: faces.clone(),
            offset: [0., 0.2, 0.],
        },
    )
    .unwrap();
    closed(&result.asset);
    assert!((volume(&result.asset) - 8.6).abs() < 1e-5);
    assert!(modeling::evaluate(&a, &Operation::Inset { faces, width: 0.05 }).is_err());
    let isolated = [7., 8., 9.];
    std::sync::Arc::make_mut(&mut a.geometry)
        .positions
        .push(isolated);
    let result = modeling::evaluate(
        &a,
        &Operation::Extrude {
            faces: top(&a),
            offset: [0., 0.2, 0.],
        },
    )
    .unwrap();
    assert!(result.asset.geometry.positions.contains(&isolated));
    assert_eq!(
        ignis::mesh_topology::Topology::inspect(&result.asset.geometry)
            .unwrap()
            .isolated_vertices
            .len(),
        1
    );
}

#[test]
fn concave_nonplanar_and_disconnected_insets_are_explicit_refusals() {
    let a = asset(Primitive::Cube, 4);
    let faces = top(&a);
    let concave: Vec<_> = faces
        .iter()
        .copied()
        .filter(|i| {
            !a.geometry.triangles[*i as usize].iter().all(|j| {
                let p = a.geometry.positions[*j as usize];
                p[0] >= 0. && p[2] >= 0.
            })
        })
        .collect();
    assert!(
        modeling::evaluate(
            &a,
            &Operation::Inset {
                faces: concave,
                width: 0.05
            }
        )
        .is_err()
    );
    let mut nonplanar = faces.clone();
    nonplanar.push(
        a.geometry
            .triangles
            .iter()
            .enumerate()
            .find(|(_, f)| normal(&a, f).x > 0.99)
            .unwrap()
            .0 as u32,
    );
    assert!(
        modeling::evaluate(
            &a,
            &Operation::Inset {
                faces: nonplanar,
                width: 0.05
            }
        )
        .is_err()
    );
    let first = faces[0];
    let distant = *faces.last().unwrap();
    assert!(
        modeling::evaluate(
            &a,
            &Operation::Extrude {
                faces: vec![first, distant],
                offset: [0., 0.1, 0.]
            }
        )
        .is_err()
    );
}
