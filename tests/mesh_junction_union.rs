use ignis::{
    engine::{Command, Engine},
    mesh_junction_union::{self, Options},
    mesh_topology::Topology,
    meshes::{Asset, Geometry, Shading},
    model::RoomRecipe,
};
use serde::Deserialize;
use std::sync::Arc;
#[derive(Deserialize)]
struct Fixture {
    source: Asset,
    ports: [Vec<u32>; 3],
    root: Option<String>,
    options: Options,
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../examples/mesh-junction-union.input.json")).unwrap()
}
#[test]
fn frozen_three_ports_close_with_retained_data_and_rigid_root_exact() {
    let f = fixture();
    let before = f.source.clone();
    let out = mesh_junction_union::fit(&f.source, &f.ports, f.root.as_deref(), &f.options).unwrap();
    assert_eq!(f.source, before);
    let n = f.source.geometry.positions.len();
    let t = f.source.geometry.triangles.len();
    assert_eq!(
        &out.asset.geometry.positions[..n],
        &f.source.geometry.positions
    );
    assert_eq!(
        &out.asset.geometry.triangles[..t],
        &f.source.geometry.triangles
    );
    assert_eq!(
        &out.asset.geometry.material_coordinates.as_ref().unwrap()[..n],
        f.source.geometry.material_coordinates.as_ref().unwrap()
    );
    assert_eq!(
        &out.asset.rig.as_ref().unwrap().weights[..n],
        &f.source.rig.as_ref().unwrap().weights
    );
    assert_eq!(
        out.asset.rig.as_ref().unwrap().bones,
        f.source.rig.as_ref().unwrap().bones
    );
    assert_eq!(out.asset.clips, f.source.clips);
    let exported = serde_json::to_value(&out).unwrap();
    let plan: ignis::mesh_material_finish::Plan =
        serde_json::from_value(exported["material_plan"].clone()).unwrap();
    assert_eq!(plan.retained_vertices as usize, n);
    assert_eq!(plan.retained_faces as usize, t);
    let iso_end = n + plan.iso_vertices as usize;
    assert_eq!(
        plan.pairs.len(),
        out.asset.geometry.positions.len() - iso_end
    );
    let guards: std::collections::BTreeSet<_> = plan.pairs.iter().map(|p| p[1]).collect();
    assert_eq!(guards.len(), plan.pairs.len());
    for [source, guard] in plan.pairs {
        assert!((source as usize) < n);
        assert!((iso_end..out.asset.geometry.positions.len()).contains(&(guard as usize)));
    }
    assert_eq!(out.asset.material, f.source.material);
    assert_eq!(out.asset.shading, f.source.shading);
    assert_eq!(out.asset.geometry.positions.len(), 3995);
    assert_eq!(out.asset.geometry.triangles.len(), 7986);
    assert!(out.report.max_bend_degrees < 60.);
    assert!(out.report.mean_bend_degrees < 6.);
    assert!(out.report.max_field_error < 1e-6);
    let topology = Topology::inspect(&out.asset.geometry).unwrap();
    assert_eq!(topology.boundary_edges, 0);
    assert_eq!(topology.non_manifold_edges, 0);
    assert!(topology.isolated_vertices.is_empty());
    assert_eq!(
        topology.vertices as isize - topology.edges.len() as isize + topology.faces.len() as isize,
        2
    );
    let again =
        mesh_junction_union::fit(&f.source, &f.ports, f.root.as_deref(), &f.options).unwrap();
    assert_eq!(out.asset, again.asset);
}
#[test]
fn errors_reject_orientation_mixed_roots_options_and_budgets_without_source_change() {
    let f = fixture();
    let original = f.source.clone();
    for case in 0..8 {
        let mut lp = f.ports.clone();
        let mut options = f.options.clone();
        let mut source = f.source.clone();
        let mut root = f.root.clone();
        match case {
            0 => lp[0].reverse(),
            1 => lp[1] = lp[0].clone(),
            2 => options.hub = [100.; 3],
            3 => options.cell_size = 0.001,
            4 => options.collar_length = 0.,
            5 => root = None,
            6 => {
                let rig = Arc::make_mut(source.rig.as_mut().unwrap());
                rig.weights[lp[1][0] as usize] = ignis::meshes::Weight {
                    extra: None,
                    joints: [0, 1, 0, 0],
                    weights: [0.5, 0.5, 0., 0.],
                };
            }
            _ => options.relaxation_steps = 9,
        }
        assert!(
            mesh_junction_union::fit(&source, &lp, root.as_deref(), &options).is_err(),
            "case{case}"
        );
        assert_eq!(f.source, original);
    }
}
fn generic_source() -> (Asset, [Vec<u32>; 3], Options) {
    use glam::Vec3;
    let centres = [
        Vec3::new(0., -0.65, 0.),
        Vec3::new(0.55, 0.5, 0.),
        Vec3::new(-0.1, 0.8, 0.),
    ];
    let normals = [-Vec3::Y, Vec3::new(0.7, 0.7, 0.).normalize(), Vec3::Y];
    let radii = [0.18, 0.11, 0.13];
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    let mut ports = [vec![], vec![], vec![]];
    for k in 0..3 {
        let base = positions.len() as u32;
        let n = normals[k];
        let x = n.cross(Vec3::Z).normalize();
        let y = n.cross(x);
        for row in 0..2 {
            for j in 0..16 {
                let a = j as f32 * std::f32::consts::TAU / 16.;
                positions.push(
                    (centres[k] + n * (row as f32 * 0.5) + (x * a.cos() + y * a.sin()) * radii[k])
                        .to_array(),
                );
            }
        }
        ports[k] = (base..base + 16).collect();
        for j in 0..16 {
            let a = base + j;
            let b = base + (j + 1) % 16;
            triangles.extend([[a, b, a + 16], [b, b + 16, a + 16]]);
        }
        for j in 1..15 {
            triangles.push([base + 16, base + 16 + j, base + 17 + j]);
        }
    }
    let source = Asset {
        id: "three-tubes".into(),
        geometry: Arc::new(Geometry {
            positions,
            triangles,
            material_coordinates: None,
        }),
        shading: Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    };
    (
        source,
        ports,
        Options {
            hub: [0.; 3],
            hub_radius: 0.15,
            blend: 0.04,
            cell_size: 0.05,
            guard_length: 0.006,
            collar_length: 0.04,
            relaxation_steps: 3,
            material_offset: [0.; 3],
            seam_relaxation_steps: 0,
            seam_max_displacement: 0.,
            material_relaxation_steps: 0,
        },
    )
}
#[test]
fn generic_unrigged_tubes_use_same_kernel_without_material_domain() {
    let (a, p, o) = generic_source();
    let out = mesh_junction_union::fit(&a, &p, None, &o).unwrap();
    assert!(out.asset.rig.is_none());
    assert!(out.asset.geometry.material_coordinates.is_none());
    assert!(out.report.max_bend_degrees < 90.);
    assert_eq!(
        &out.asset.geometry.positions[..a.geometry.positions.len()],
        &a.geometry.positions
    );
}
#[test]
fn dispatcher_query_preserves_runtime_history_and_reports_errors_atomically() {
    let f = fixture();
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(Command::Step { ticks: 9 }.into()).unwrap();
    let before = e.snapshot();
    let command = Command::PreviewMeshJunctionUnion {
        source: Box::new(f.source),
        ports: Box::new(f.ports),
        root: f.root,
        options: f.options,
    };
    let r = e.execute(command.clone().into()).unwrap();
    assert!(r["report"]["max_bend_degrees"].as_f64().unwrap() < 60.);
    assert_eq!(e.snapshot(), before);
    let mut bad = command;
    if let Command::PreviewMeshJunctionUnion { options, .. } = &mut bad {
        options.blend = f32::NAN;
    }
    assert!(e.execute(bad.into()).is_err());
    assert_eq!(e.snapshot(), before);
}

#[test]
fn finite_seam_and_domain_refinement_keep_source_and_first_guard_exact() {
    let f = fixture();
    let base =
        mesh_junction_union::fit(&f.source, &f.ports, f.root.as_deref(), &f.options).unwrap();
    let old = f.source.geometry.positions.len();
    let iso = old + base.report.extracted_vertices;
    let mut o = f.options.clone();
    o.seam_max_displacement = 0.025;
    o.material_relaxation_steps = 64;
    for steps in [1, 2, 4] {
        o.seam_relaxation_steps = steps;
        let fit = mesh_junction_union::fit(&f.source, &f.ports, f.root.as_deref(), &o).unwrap();
        let g = &fit.asset.geometry;
        assert_eq!(&g.positions[..old], &base.asset.geometry.positions[..old]);
        assert_eq!(&g.positions[iso..], &base.asset.geometry.positions[iso..]);
        assert_eq!(g.triangles, base.asset.geometry.triangles);
        assert_eq!(fit.asset.rig, base.asset.rig);
        assert_eq!(fit.asset.clips, base.asset.clips);
        assert_eq!(
            &g.material_coordinates.as_ref().unwrap()[..old],
            f.source.geometry.material_coordinates.as_ref().unwrap()
        );
        let mut guard = iso;
        for lp in &f.ports {
            for &source in lp.iter().rev() {
                assert_eq!(
                    g.material_coordinates.as_ref().unwrap()[guard],
                    g.material_coordinates.as_ref().unwrap()[source as usize]
                );
                guard += 1;
            }
        }
        assert_eq!(
            fit.report.source_seam_max_degrees,
            base.report.source_seam_max_degrees
        );
        assert!(fit.report.max_bend_degrees < base.report.max_bend_degrees);
        assert!(fit.report.mean_bend_degrees < base.report.mean_bend_degrees);
        assert!(fit.report.refinement.max_seam_displacement <= 0.025001);
        assert_eq!(fit.report.refinement.seam_selected_vertices, 352);
        assert_eq!(fit.report.refinement.material_guard_jump_after, 0.);
        assert!(fit.report.refinement.material_guard_jump_before > 3.);
        assert!(
            fit.report.refinement.material_residual_max < 1e-4,
            "{:?}",
            fit.report.refinement
        );
        assert!(fit.report.max_field_error < 1e-6);
        assert!(fit.report.max_field_error_after_seam > 1e-3);
        let r = &fit.report.refinement;
        assert!(
            ((r.signed_volume_after - r.signed_volume_before) / r.signed_volume_before).abs()
                < 0.001
        );
        if steps == 4 {
            assert!(fit.report.max_bend_degrees < 44.);
            assert!(fit.report.mean_bend_degrees < 5.);
            let again =
                mesh_junction_union::fit(&f.source, &f.ports, f.root.as_deref(), &o).unwrap();
            assert_eq!(fit.asset, again.asset);
        }
    }
}

#[test]
fn refinement_limits_and_worsening_candidate_fail_without_source_mutation() {
    let f = fixture();
    let original = f.source.clone();
    for case in 0..7 {
        let mut source = f.source.clone();
        let mut o = f.options.clone();
        match case {
            0 => o.seam_relaxation_steps = 9,
            1 => {
                o.seam_relaxation_steps = 1;
                o.seam_max_displacement = 0.;
            }
            2 => o.seam_max_displacement = 0.04,
            3 => o.seam_max_displacement = f32::NAN,
            4 => o.material_relaxation_steps = 65,
            5 => {
                o.material_relaxation_steps = 1;
                Arc::make_mut(&mut source.geometry).material_coordinates = None;
            }
            _ => {
                o.seam_relaxation_steps = 8;
                o.seam_max_displacement = 0.025;
            }
        }
        let err = mesh_junction_union::fit(&source, &f.ports, f.root.as_deref(), &o).unwrap_err();
        if case == 6 {
            assert!(err.to_string().contains("worsens"));
        }
        assert_eq!(f.source, original);
    }
}

#[test]
fn material_zero_rhs_preserves_nonzero_fixed_domain_without_rig() {
    let (mut source, ports, mut o) = generic_source();
    let base = mesh_junction_union::fit(&source, &ports, None, &o).unwrap();
    let old = source.geometry.positions.len();
    let iso = old + base.report.extracted_vertices;
    let mut coords = source.geometry.positions.clone();
    let mut guard = iso;
    for lp in &ports {
        for &src in lp.iter().rev() {
            coords[src as usize] = base.asset.geometry.positions[guard];
            guard += 1;
        }
    }
    Arc::make_mut(&mut source.geometry).material_coordinates = Some(coords);
    o.material_relaxation_steps = 64;
    let fit = mesh_junction_union::fit(&source, &ports, None, &o).unwrap();
    let g = &fit.asset.geometry;
    let coords = g.material_coordinates.as_ref().unwrap();
    assert_eq!(
        &coords[..old],
        source.geometry.material_coordinates.as_ref().unwrap()
    );
    assert_eq!(&coords[old..iso], &g.positions[old..iso]);
    assert_eq!(fit.report.refinement.material_residual_max, 0.);
    assert_eq!(fit.report.refinement.material_guard_jump_after, 0.);
    assert!(coords[iso..].iter().any(|p| *p != [0.; 3]));
    assert_eq!(g.positions, base.asset.geometry.positions);
}
