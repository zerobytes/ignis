use ignis::{
    engine::{Command, Engine},
    mesh_preservation::DisplacementPolicy,
    mesh_topology::Topology,
    meshes::{self, Asset, Geometry},
    model::RoomRecipe,
    plants::{Assembly, Definition, Edit},
};
use std::sync::Arc;
fn definition() -> Definition {
    serde_json::from_value(serde_json::json!({
"id":"branch","profile":{"id":"profile","width":0.16,"height":0.9},"depth":0.16,"segments":[6,3,6],"sampling_tolerance":0.002,
"sections":{"id":"sections","cross_section":"ellipse","stations":[{"id":"start","at":0,"width":1,"depth":1},{"id":"socket","at":0.45,"offset":[0.27,0],"width":1,"depth":1},{"id":"end","at":1,"offset":[0.6,0],"width":1,"depth":1}]}
})).unwrap()
}
fn target() -> Definition {
    let mut d = definition();
    for s in &mut d.sections.as_mut().unwrap().stations {
        s.offset = [0., 0.];
    }
    d.trajectory=Some(serde_json::from_value(serde_json::json!({"kind":"cubic_bezier","id":"axis","control_points_xz":[[0,0],[0.2,0],[0.4,0],[0.6,0]],"surface_tolerance":0.002})).unwrap());
    d
}
fn mesh(e: &mut Engine, edit: meshes::Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}
fn plant(e: &mut Engine, edit: Edit) {
    e.execute(Command::EditPlant { edit }.into()).unwrap();
}
fn setup() -> (Engine, Assembly) {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    plant(
        &mut e,
        Edit::Put {
            definition: definition(),
            detail_policy: DisplacementPolicy::Physical,
            animation_translation_policy: DisplacementPolicy::Physical,
        },
    );
    let source = e.scene.meshes.assets[0].clone();
    mesh(
        &mut e,
        meshes::Edit::SetMaterialCoordinates {
            asset: "branch".into(),
            coordinates: Some(source.geometry.positions.clone()),
        },
    );
    mesh(
        &mut e,
        meshes::Edit::SetRig {
            asset: "branch".into(),
            rig: Some(Arc::new(meshes::Rig {
                bones: vec![
                    meshes::Bone {
                        name: "root".into(),
                        parent: None,
                        translation: [0., -0.45, 0.],
                        rotation: [0., 0., 0., 1.],
                    },
                    meshes::Bone {
                        name: "socket".into(),
                        parent: Some(0),
                        translation: [0.27, 0.405, 0.],
                        rotation: [0., 0., 0., 1.],
                    },
                ],
                weights: vec![
                    meshes::Weight {
                        extra: None,
                        joints: [0; 4],
                        weights: [1., 0., 0., 0.]
                    };
                    source.geometry.positions.len()
                ],
            })),
        },
    );
    mesh(
        &mut e,
        meshes::Edit::PutClip {
            asset: "branch".into(),
            clip: Arc::new(meshes::Clip {
                id: "wind".into(),
                duration_ticks: 60,
                tracks: vec![meshes::Track {
                    bone: 1,
                    keys: vec![
                        meshes::Keyframe {
                            tick: 0,
                            translation: [0.27, 0.405, 0.],
                            rotation: [0., 0., 0., 1.],
                        },
                        meshes::Keyframe {
                            tick: 60,
                            translation: [0.27, 0.405, 0.],
                            rotation: [0., 0., 0.04_f32.sin(), 0.04_f32.cos()],
                        },
                    ],
                }],
            }),
        },
    );
    let center = *source
        .geometry
        .positions
        .iter()
        .find(|p| p[1] > 0.3)
        .unwrap();
    mesh(
        &mut e,
        meshes::Edit::Sculpt {
            asset: "branch".into(),
            stroke: meshes::Stroke {
                kind: meshes::BrushKind::Grab,
                centers: vec![center],
                radius: 0.025,
                strength: 0.001,
                direction: [0., 0., 1.],
            },
        },
    );
    plant(
        &mut e,
        Edit::AdoptTrajectory {
            definition: target(),
            detail_policy: DisplacementPolicy::Physical,
            animation_translation_policy: DisplacementPolicy::Physical,
        },
    );
    let mut d = definition();
    d.id = "context".into();
    d.profile.width = 0.25;
    d.profile.height = 1.;
    d.depth = 0.25;
    d.segments = [4, 16, 4];
    d.sections.as_mut().unwrap().stations.remove(1);
    for s in &mut d.sections.as_mut().unwrap().stations {
        s.offset = [0., 0.];
    }
    let mut g = d.sample().unwrap().geometry;
    g.material_coordinates = Some(g.positions.clone());
    let faces = g
        .triangles
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            t.iter().all(|&i| {
                g.positions[i as usize][1] >= -0.250001 && g.positions[i as usize][1] <= 0.250001
            })
        })
        .map(|(i, _)| i as u32)
        .collect();
    let loops = [-0.25_f32, 0.25].map(|y| {
        g.positions
            .iter()
            .enumerate()
            .filter(|(_, p)| (p[1] - y).abs() < 1e-6)
            .map(|(i, _)| i as u32)
            .collect()
    });
    let context = Asset {
        id: "context".into(),
        geometry: Arc::new(g),
        material: Default::default(),
        shading: meshes::Shading::Flat,
        rig: None,
        clips: vec![],
    };
    let assembly=Assembly{plant_material_frame:None,material_finish:None,context_grafts:vec![],context_lofts:vec![],context_surface:None,junction_surface:None,context:Arc::new(context),context_faces:faces,context_loops:loops,station:"socket".into(),context_bone:Some("root".into()),options:serde_json::from_value(serde_json::json!({"hub":[0,-0.1,0],"hub_radius":0.115,"blend":0.03,"cell_size":0.04,"guard_length":0.004,"collar_length":0.03,"relaxation_steps":3,"material_offset":[0,0,0]})).unwrap()};
    (e, assembly)
}
#[test]
fn canonical_context_mask_expands_through_loft_and_finishing_survives_regeneration() {
    let (mut e, mut recipe) = setup();
    // Authored high-frequency relief to remove; do not expect a local smoother
    // to improve the mean curvature of an already regular cylindrical strip.
    let context = Arc::make_mut(&mut recipe.context);
    for p in &mut Arc::make_mut(&mut context.geometry).positions {
        if (p[1] + 0.375).abs() < 1e-6 {
            let factor = 1. + 0.08 * (5. * p[2].atan2(p[0])).cos();
            p[0] *= factor;
            p[2] *= factor;
        }
    }
    let source = recipe.context.clone();
    let g = &source.geometry;
    let coordinates = g.material_coordinates.as_ref().unwrap();
    let mut rings = Vec::new();
    for y in [-0.5_f32, -0.4375, -0.375, -0.3125, -0.25] {
        let mut ring: Vec<u32> = g
            .positions
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                (p[1] - y).abs() < 1e-6
                    && (coordinates[*i][0] * coordinates[*i][0]
                        + coordinates[*i][2] * coordinates[*i][2])
                        .sqrt()
                        > 0.1249
            })
            .map(|(i, _)| i as u32)
            .collect();
        ring.sort_by(|&a, &b| {
            let a = g.positions[a as usize];
            let b = g.positions[b as usize];
            a[2].atan2(a[0]).total_cmp(&b[2].atan2(b[0]))
        });
        assert!(ring.len() >= 4);
        rings.push(ring);
    }
    recipe.context_lofts = vec![ignis::mesh_ring_loft::Loft {
        rings: rings.clone(),
        spans: 3,
        interpolation: ignis::mesh_ring_loft::Interpolation::Cubic,
        max_deviation: 0.01,
    }];
    recipe.context_surface = Some(ignis::plants::ContextSurface {
        vertices: rings.iter().flatten().copied().collect(),
        anchors: rings[0]
            .iter()
            .chain(&rings[1])
            .chain(&rings[3])
            .chain(&rings[4])
            .copied()
            .collect(),
        options: ignis::mesh_surface::Options {
            iterations: 2,
            strength: 0.15,
            max_displacement: 0.003,
        },
    });
    recipe.junction_surface = Some(ignis::mesh_surface::Options {
        iterations: 2,
        strength: 0.1,
        max_displacement: 0.001,
    });
    let original = e.scene.plants.entry("branch").unwrap().clone();
    plant(
        &mut e,
        Edit::SetAssembly {
            id: "branch".into(),
            assembly: Some(recipe.clone()),
        },
    );
    let entry = e.scene.plants.entry("branch").unwrap();
    assert_eq!(entry.source, original.source);
    assert_eq!(entry.reference, original.reference);
    assert_eq!(entry.assembly.as_ref().unwrap().context, source);
    let evaluated = entry.evaluate().unwrap();
    let report = evaluated.report.assembly.unwrap();
    let surface = report.context_surface.unwrap();
    assert!(surface.selected_vertices > rings.iter().map(Vec::len).sum());
    assert!(surface.moved_vertices > 0);
    assert!(report.junction_surface.unwrap().moved_vertices > 0);
    let output = e.scene.clone();
    let mut d = entry.definition.clone();
    // Authored curve changes and returns preserve canonical layers and final output.
    d.profile.height += 0.001;
    plant(
        &mut e,
        Edit::Put {
            definition: d,
            detail_policy: DisplacementPolicy::Physical,
            animation_translation_policy: DisplacementPolicy::Physical,
        },
    );
    plant(
        &mut e,
        Edit::Put {
            definition: original.definition,
            detail_policy: DisplacementPolicy::Physical,
            animation_translation_policy: DisplacementPolicy::Physical,
        },
    );
    assert_eq!(e.scene, output);
    let mut old = output;
    old.version = 66;
    assert!(old.validate().unwrap_err().message.contains("version67"));
}
#[test]
fn assembly_keeps_sources_pose_channels_and_rebuilds_without_baking() {
    let (mut e, assembly) = setup();
    let entry = e.scene.plants.entry("branch").unwrap().clone();
    let original = e.scene.clone();
    let edit = Edit::SetAssembly {
        id: "branch".into(),
        assembly: Some(assembly.clone()),
    };
    let before = e.snapshot();
    let predicted = e
        .execute(
            Command::PreviewSceneEdits {
                edits: vec![ignis::engine::SceneEdit::Plant { edit: edit.clone() }],
            }
            .into(),
        )
        .unwrap();
    assert_eq!(e.snapshot(), before);
    assert_eq!(
        serde_json::from_value::<ignis::model::Scene>(predicted["scene"].clone())
            .unwrap()
            .plants
            .entries[0]
            .assembly,
        Some(assembly)
    );
    assert!(
        e.execute(Command::PreviewSceneEdits { edits: vec![] }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let state = e.snapshot();
    let preview = e
        .execute(Command::PreviewPlant { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(e.snapshot(), state);
    plant(&mut e, edit);
    let expected: Asset = serde_json::from_value(preview["assets"][0].clone()).unwrap();
    assert_eq!(e.scene.meshes.assets[0], expected);
    let linked = e.scene.plants.entry("branch").unwrap();
    assert!(Arc::ptr_eq(&entry.source, &linked.source));
    assert_eq!(entry.reference, linked.reference);
    let evaluated = linked.evaluate().unwrap();
    assert!(evaluated.report.preservation.max_detail_displacement > 0.00099);
    let n = evaluated
        .report
        .assembly
        .as_ref()
        .unwrap()
        .retained_context_vertices;
    let sample = target().sample().unwrap();
    let kept: std::collections::BTreeSet<_> = original.meshes.assets[0]
        .geometry
        .triangles
        .iter()
        .filter(|t| t.iter().all(|&i| sample.material[i as usize][1] >= -1e-6))
        .flatten()
        .copied()
        .collect();
    let old = &original.meshes.assets[0];
    assert_eq!(
        expected.geometry.positions[n..n + kept.len()],
        kept.iter()
            .map(|&i| old.geometry.positions[i as usize])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        expected.geometry.material_coordinates.as_ref().unwrap()[n..n + kept.len()],
        kept.iter()
            .map(|&i| old.geometry.material_coordinates.as_ref().unwrap()[i as usize])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        expected.rig.as_ref().unwrap().weights[n..n + kept.len()],
        kept.iter()
            .map(|&i| old.rig.as_ref().unwrap().weights[i as usize].clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(linked.evaluate().unwrap().asset, expected);
    let t = Topology::inspect(&expected.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);
    assert_eq!(
        t.vertices as isize - t.edges.len() as isize + t.faces.len() as isize,
        2
    );
    assert_eq!(
        expected.rig.as_ref().unwrap().bones,
        original.meshes.assets[0].rig.as_ref().unwrap().bones
    );
    assert_eq!(expected.clips, original.meshes.assets[0].clips);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene.meshes.assets[0], expected);
    for _ in 0..2 {
        let mut d = target();
        if let Some(ignis::plants::Trajectory::CubicBezier {
            control_points_xz, ..
        }) = &mut d.trajectory
        {
            control_points_xz[3][0] += 0.001;
        }
        plant(
            &mut e,
            Edit::Put {
                definition: d,
                detail_policy: DisplacementPolicy::Physical,
                animation_translation_policy: DisplacementPolicy::Physical,
            },
        );
        plant(
            &mut e,
            Edit::Put {
                definition: target(),
                detail_policy: DisplacementPolicy::Physical,
                animation_translation_policy: DisplacementPolicy::Physical,
            },
        );
        assert_eq!(e.scene.meshes.assets[0], expected);
        assert_eq!(e.scene.plants.entry("branch").unwrap().source, entry.source);
    }
    let scene = e.scene.clone();
    let mut old = scene.clone();
    old.version = 65;
    assert!(old.validate().unwrap_err().message.contains("version66"));
    let reloaded =
        Engine::new(serde_json::from_slice(&serde_json::to_vec(&scene).unwrap()).unwrap()).unwrap();
    assert_eq!(reloaded.scene, scene);
    plant(
        &mut e,
        Edit::SetAssembly {
            id: "branch".into(),
            assembly: None,
        },
    );
    assert_eq!(e.scene.meshes.assets[0], original.meshes.assets[0]);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, scene);
}
#[test]
fn invalid_recipe_and_direct_rebase_preserve_execution_and_canonical_sources() {
    let (mut e, a) = setup();
    for case in 0..6 {
        let mut bad = a.clone();
        match case {
            0 => bad.station = "missing".into(),
            1 => bad.context_loops[1] = bad.context_loops[0].clone(),
            2 => bad.context_bone = Some("missing".into()),
            3 => bad.context_faces.push(bad.context_faces[0]),
            4 => {
                Arc::make_mut(&mut bad.context).geometry = Arc::new(Geometry::grid(1, 1.).unwrap())
            }
            _ => bad.options.cell_size = 0.001,
        };
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::EditPlant {
                    edit: Edit::SetAssembly {
                        id: "branch".into(),
                        assembly: Some(bad)
                    }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    plant(
        &mut e,
        Edit::SetAssembly {
            id: "branch".into(),
            assembly: Some(a),
        },
    );
    let before = e.snapshot();
    let rig = (**e.scene.meshes.assets[0].rig.as_ref().unwrap()).clone();
    assert!(
        e.execute(
            Command::EditMesh {
                edit: meshes::Edit::SetRig {
                    asset: "branch".into(),
                    rig: Some(Arc::new(rig))
                }
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
