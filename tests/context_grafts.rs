use ignis::{
    engine::{Command, Engine},
    mesh_bridge,
    mesh_topology::Topology,
    meshes,
    model::Scene,
    plants,
};
use std::{collections::BTreeSet, sync::Arc};
const ID: &str = "branch-editable-1";
fn scene() -> Scene {
    serde_json::from_str(include_str!("../examples/native-tree-study.scene.json")).unwrap()
}
fn recipe(s: &Scene) -> plants::Assembly {
    s.plants.entry(ID).unwrap().assembly.clone().unwrap()
}
fn edit(a: plants::Assembly) -> Command {
    Command::EditPlant {
        edit: plants::Edit::SetAssembly {
            id: ID.into(),
            assembly: Some(a),
        },
    }
}
#[test]
fn exact_source_frame_detail_channels_root_and_closed_topology_survive_graft() {
    let s = scene();
    let mut e = s.plants.entry(ID).unwrap().clone();
    let before = e.clone();
    // Nonzero authored relief in a retained child point, beyond the fixed port.
    let graft = &mut e.assembly.as_mut().unwrap().context_grafts[0];
    Arc::make_mut(&mut Arc::make_mut(&mut graft.source).geometry).positions[14][0] += 0.001;
    let canonical = graft.clone();
    let fitted = e.evaluate().unwrap();
    assert_eq!(e.source, before.source);
    assert_eq!(e.reference, before.reference);
    assert_eq!(
        e.assembly.as_ref().unwrap().context,
        before.assembly.as_ref().unwrap().context
    );
    assert_eq!(e.assembly.as_ref().unwrap().context_grafts[0], canonical);
    let source = mesh_bridge::remove_faces(&canonical.source, &canonical.source_faces).unwrap();
    let out = &fitted.asset;
    let domain = out.geometry.material_coordinates.as_ref().unwrap();
    let pairs: BTreeSet<_> = out
        .geometry
        .positions
        .iter()
        .zip(domain)
        .map(|(p, m)| (p.map(f32::to_bits), m.map(f32::to_bits)))
        .collect();
    let mut found = 0;
    for (p, m) in source
        .geometry
        .positions
        .iter()
        .zip(source.geometry.material_coordinates.as_ref().unwrap())
    {
        let translated = std::array::from_fn::<_, 3, _>(|i| p[i] + canonical.source_translation[i]);
        assert!(pairs.contains(&(translated.map(f32::to_bits), m.map(f32::to_bits))));
        found += 1;
    }
    assert_eq!(found, 91);
    let rig = out.rig.as_ref().unwrap();
    assert_eq!(rig.bones, before.source.rig.as_ref().unwrap().bones);
    assert_eq!(out.clips, before.source.clips);
    let report = fitted.report.assembly.unwrap();
    let graft = &report.context_grafts[0];
    assert_eq!(graft.retained_source_vertices, 91);
    assert!(graft.junction.retained_vertices_exact && graft.junction.retained_triangles_exact);
    assert!(graft.surface.as_ref().unwrap().moved_vertices > 0);
    let topology = Topology::inspect(&out.geometry).unwrap();
    assert_eq!(topology.boundary_edges, 0);
    assert_eq!(topology.non_manifold_edges, 0);
    assert!(topology.isolated_vertices.is_empty());
    assert!(mesh_bridge::boundaries(&out.geometry).unwrap().is_empty());
}
#[test]
fn graft_preview_publication_parameter_return_undo_and_version_gate() {
    let mut e = Engine::new(scene()).unwrap();
    let initial = e.scene.clone();
    let a = recipe(&initial);
    let state = e.snapshot();
    let value = e
        .execute(
            Command::PreviewPlant {
                edit: plants::Edit::SetAssembly {
                    id: ID.into(),
                    assembly: Some(a.clone()),
                },
            }
            .into(),
        )
        .unwrap();
    assert_eq!(e.snapshot(), state);
    let expected: meshes::Asset = serde_json::from_value(
        value["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == ID)
            .unwrap()
            .clone(),
    )
    .unwrap();
    e.execute(edit(a).into()).unwrap();
    assert_eq!(*e.scene.meshes.asset(ID).unwrap(), expected);
    let entry = e.scene.plants.entry(ID).unwrap().clone();
    let mut changed = entry.definition.clone();
    let plants::Trajectory::CubicBezier {
        control_points_xz, ..
    } = changed.trajectory.as_mut().unwrap();
    control_points_xz[3][0] += 0.001;
    for d in [changed, entry.definition.clone()] {
        e.execute(
            Command::EditPlant {
                edit: plants::Edit::Put {
                    definition: d,
                    detail_policy: ignis::mesh_preservation::DisplacementPolicy::Physical,
                    animation_translation_policy:
                        ignis::mesh_preservation::DisplacementPolicy::Physical,
                },
            }
            .into(),
        )
        .unwrap();
    }
    assert_eq!(e.scene, initial);
    e.execute(Command::Undo.into()).unwrap();
    assert_ne!(e.scene, initial);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, initial);
    // Change the parent's tessellation: masks and later cuts must not keep
    // ephemeral evaluated IDs from the previous loft/graft.
    let mut parent = recipe(&e.scene);
    parent.context_lofts[0].spans = 4;
    e.execute(edit(parent).into()).unwrap();
    assert_ne!(
        e.scene.meshes.asset(ID).unwrap().geometry,
        expected.geometry
    );
    e.execute(edit(recipe(&initial)).into()).unwrap();
    assert_eq!(e.scene, initial);
    let mut old = initial;
    old.version = 68;
    assert!(old.validate().unwrap_err().message.contains("version69"));
}
#[test]
fn invalid_and_overlapping_grafts_are_atomic_in_preview_and_execution() {
    let mut e = Engine::new(scene()).unwrap();
    e.execute(Command::Step { ticks: 11 }.into()).unwrap();
    let source = recipe(&e.scene);
    for case in 0..12 {
        let mut a = source.clone();
        let g = &mut a.context_grafts[0];
        match case {
            0 => g.context_vertices.push(g.context_vertices[0]),
            1 => g
                .context_vertices
                .push(a.context.geometry.positions.len() as u32),
            2 => g.context_loops[0][0] = u32::MAX,
            3 => g.source_loop[0] = u32::MAX,
            4 => g.source_faces.push(u32::MAX),
            5 => g.source_translation[0] = f32::INFINITY,
            6 => g.options.cell_size = 0.001,
            7 => {
                let mut rig = (**e
                    .scene
                    .plants
                    .entry(ID)
                    .unwrap()
                    .source
                    .rig
                    .as_ref()
                    .unwrap())
                .clone();
                rig.weights.resize(
                    g.source.geometry.positions.len(),
                    meshes::Weight {
                        extra: None,
                        joints: [0; 4],
                        weights: [1., 0., 0., 0.],
                    },
                );
                Arc::make_mut(&mut g.source).rig = Some(Arc::new(rig));
                meshes::Document {
                    objects: Default::default(),
                    assets: vec![(*g.source).clone()],
                    ..Default::default()
                }
                .validate()
                .unwrap();
            }
            8 => a.context_grafts = vec![g.clone(), g.clone()],
            9 => a.context_grafts = vec![g.clone(); 9],
            10 => {
                let selected: BTreeSet<_> = g.context_vertices.iter().copied().collect();
                let face = a
                    .context
                    .geometry
                    .triangles
                    .iter()
                    .position(|t| t.iter().all(|i| selected.contains(i)))
                    .unwrap();
                a.context_faces.push(face as u32);
            }
            _ => a.context_loops[0][0] = a.context.geometry.positions.len() as u32,
        }
        let before = e.snapshot();
        let pe = plants::Edit::SetAssembly {
            id: ID.into(),
            assembly: Some(a.clone()),
        };
        assert!(
            e.execute(Command::PreviewPlant { edit: pe }.into())
                .is_err(),
            "case{case}"
        );
        assert_eq!(e.snapshot(), before);
        assert!(e.execute(edit(a).into()).is_err(), "case{case}");
        assert_eq!(e.snapshot(), before);
    }
}
#[test]
fn appearance_updates_graft_metadata_without_rebasing_sources() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.scene.clone();
    for command in [
        Command::EditMesh {
            edit: meshes::Edit::SetMaterial {
                asset: ID.into(),
                material: Default::default(),
            },
        },
        Command::EditMesh {
            edit: meshes::Edit::SetShading {
                asset: ID.into(),
                shading: meshes::Shading::Flat,
            },
        },
    ] {
        let prior = e.scene.clone();
        e.execute(command.into()).unwrap();
        let a = recipe(&e.scene);
        let b = recipe(&prior);
        assert_eq!(a.context.geometry, b.context.geometry);
        assert_eq!(
            a.context_grafts[0].source.geometry,
            b.context_grafts[0].source.geometry
        );
        assert_eq!(a.context.material, a.context_grafts[0].source.material);
        assert_eq!(a.context.shading, a.context_grafts[0].source.shading);
        e.execute(Command::Undo.into()).unwrap();
        assert_eq!(e.scene, prior);
    }
    assert_eq!(e.scene, before);
}
#[test]
fn all_hidden_graft_sources_and_consumed_context_vertices_are_validated_before_evaluation() {
    let mut s = scene();
    let a = s.plants.entries[0].assembly.as_mut().unwrap();
    let mut extra = a.context_grafts[0].clone();
    let source = Arc::make_mut(&mut extra.source);
    let g = Arc::make_mut(&mut source.geometry);
    g.positions.resize(5000, g.positions[0]);
    let material = g.material_coordinates.as_mut().unwrap();
    material.resize(5000, material[0]);
    // Scene103 removed aggregate mesh storage ceilings. One larger hidden source
    // remains valid; repeatedly consuming the same context ports must still fail.
    a.context_grafts = vec![extra.clone()];
    s.validate().unwrap();
    let a = s.plants.entries[0].assembly.as_mut().unwrap();
    a.context_grafts = (0..8)
        .map(|i| {
            let mut graft = extra.clone();
            Arc::make_mut(&mut graft.source).id = format!("hidden-{i}");
            graft
        })
        .collect();
    let error = s.validate().unwrap_err();
    assert!(
        error.message.contains("context vertex"),
        "{}",
        error.message
    );
    let mut s = scene();
    let a = s.plants.entries[0].assembly.as_mut().unwrap();
    Arc::make_mut(&mut Arc::make_mut(&mut a.context_grafts[0].source).geometry).positions[0][0] =
        f32::NAN;
    assert!(s.validate().is_err());
    let mut s = scene();
    let a = s.plants.entries[0].assembly.as_mut().unwrap();
    a.context_grafts.push(a.context_grafts[0].clone());
    let error = s.validate().unwrap_err();
    assert!(error.message.contains("Duplicate"), "{}", error.message);
}

#[test]
fn collar_is_an_explicit_post_filter_with_exact_sources_channels_and_protected_ports() {
    let s = scene();
    let entry = s.plants.entry(ID).unwrap();
    let mut before = entry.clone();
    before.assembly.as_mut().unwrap().context_grafts[0].collar_surface = None;
    let original = before.clone();
    let base = before.evaluate().unwrap();
    let mut legacy = s.clone();
    legacy.plants.entries[0] = before.clone();
    *legacy
        .meshes
        .assets
        .iter_mut()
        .find(|a| a.id == ID)
        .unwrap() = base.asset.clone();
    legacy.version = 68;
    legacy.validate().unwrap();
    let migrated = Engine::new(legacy.clone()).unwrap();
    assert_eq!(*migrated.scene.meshes.asset(ID).unwrap(), base.asset);
    legacy.version = 67;
    assert!(legacy.validate().unwrap_err().message.contains("version68"));
    let mut after = before.clone();
    after.assembly.as_mut().unwrap().context_grafts[0].collar_surface =
        Some(plants::CollarSurface {
            ports: vec![0, 1],
            rings: 6,
            fixed_rings: 2,
            options: ignis::mesh_surface::Options {
                iterations: 4,
                strength: 0.3,
                max_displacement: 0.01,
            },
        });
    let fitted = after.evaluate().unwrap();
    assert_eq!(before, original);
    assert_eq!(after.source, before.source);
    assert_eq!(after.reference, before.reference);
    let a = after.assembly.as_ref().unwrap();
    let b = before.assembly.as_ref().unwrap();
    assert_eq!(a.context, b.context);
    assert_eq!(a.context_grafts[0].source, b.context_grafts[0].source);
    assert_eq!(
        fitted.asset.geometry.triangles,
        base.asset.geometry.triangles
    );
    assert_eq!(
        fitted.asset.geometry.material_coordinates,
        base.asset.geometry.material_coordinates
    );
    assert_eq!(fitted.asset.rig, base.asset.rig);
    assert_eq!(fitted.asset.clips, base.asset.clips);
    let report = fitted.report.assembly.as_ref().unwrap();
    let collar = report.context_grafts[0].collar_surface.as_ref().unwrap();
    assert!(collar.moved_retained_context > 0 && collar.moved_generated > 0);
    assert_eq!(collar.moved_retained_source, 0);
    let moved = fitted
        .asset
        .geometry
        .positions
        .iter()
        .zip(&base.asset.geometry.positions)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(moved, collar.filter.moved_vertices);
    assert!(collar.filter.bends_after.maximum_degrees < collar.filter.bends_before.maximum_degrees);
    assert!(collar.filter.bends_after.mean_degrees < collar.filter.bends_before.mean_degrees);
    assert!(collar.filter.max_displacement <= 0.010001);
    // The unselected third port's seven guards are the last context vertices.
    let n = report.retained_context_vertices;
    assert_eq!(
        fitted.asset.geometry.positions[n - 7..n],
        base.asset.geometry.positions[n - 7..n]
    );
    let roundtrip: plants::Entry =
        serde_json::from_str(&serde_json::to_string(&before).unwrap()).unwrap();
    assert_eq!(roundtrip, before);
    assert!(
        !serde_json::to_string(&before)
            .unwrap()
            .contains("collar_surface")
    );
}
#[test]
fn malformed_and_overworked_collar_requests_are_atomic() {
    let mut e = Engine::new(scene()).unwrap();
    let source = recipe(&e.scene);
    for case in 0..8 {
        let mut a = source.clone();
        let mut collar = plants::CollarSurface {
            ports: vec![0, 1],
            rings: 6,
            fixed_rings: 2,
            options: ignis::mesh_surface::Options {
                iterations: 4,
                strength: 0.3,
                max_displacement: 0.01,
            },
        };
        match case {
            0 => collar.ports.clear(),
            1 => collar.ports = vec![0, 0],
            2 => collar.ports = vec![3],
            3 => collar.rings = 17,
            4 => collar.fixed_rings = 0,
            5 => collar.fixed_rings = 7,
            6 => collar.options.strength = f32::NAN,
            _ => {
                collar.options.iterations = 32;
                collar.options.max_displacement = 0.03;
            }
        }
        a.context_grafts[0].collar_surface = Some(collar);
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::PreviewPlant {
                    edit: plants::Edit::SetAssembly {
                        id: ID.into(),
                        assembly: Some(a.clone())
                    }
                }
                .into()
            )
            .is_err(),
            "case{case}"
        );
        assert_eq!(e.snapshot(), before);
        assert!(e.execute(edit(a).into()).is_err(), "case{case}");
        assert_eq!(e.snapshot(), before);
    }
}
