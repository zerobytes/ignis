use ignis::{
    mesh_surface::{self, Options},
    mesh_topology::Topology,
    meshes::{self, Asset, Geometry},
};
use std::sync::Arc;

fn options() -> Options {
    Options {
        iterations: 1,
        strength: 0.5,
        max_displacement: 1.,
    }
}
fn asset(positions: Vec<[f32; 3]>, triangles: Vec<[u32; 3]>) -> Asset {
    let material_coordinates = Some(
        positions
            .iter()
            .map(|p| [p[0] + 3., p[1] * 2. + 1., p[2] - 2.])
            .collect(),
    );
    Asset {
        id: "surface".into(),
        geometry: Arc::new(Geometry {
            positions,
            triangles,
            material_coordinates,
        }),
        shading: meshes::Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    }
}
fn fan() -> Asset {
    asset(
        vec![
            [0., 0.5, 0.],
            [1., 0., 0.],
            [0., 0., 1.],
            [-1., 0., 0.],
            [0., 0., -1.],
        ],
        vec![[0, 2, 1], [0, 3, 2], [0, 4, 3], [0, 1, 4]],
    )
}
fn octahedron() -> Asset {
    asset(
        vec![
            [0., 1., 0.],
            [1., 0., 0.],
            [0., 0., 1.],
            [-1., 0., 0.],
            [0., 0., -1.],
            [0., -1., 0.],
        ],
        vec![
            [0, 2, 1],
            [0, 3, 2],
            [0, 4, 3],
            [0, 1, 4],
            [5, 1, 2],
            [5, 2, 3],
            [5, 3, 4],
            [5, 4, 1],
        ],
    )
}
#[test]
fn selected_peak_smooths_but_fixed_geometry_channels_and_source_are_exact() {
    let mut source = fan();
    Arc::make_mut(&mut source.geometry).positions[1][1] = -0.;
    let before = source.clone();
    let out = mesh_surface::smooth_region(&source, &[0], &[], &options()).unwrap();
    assert_eq!(source, before);
    assert_eq!(out.asset.geometry.positions[0], [0., 0.25, 0.]);
    assert_eq!(
        out.asset.geometry.positions[1][1].to_bits(),
        (-0_f32).to_bits()
    );
    assert_eq!(
        &out.asset.geometry.positions[1..],
        &source.geometry.positions[1..]
    );
    assert_eq!(out.asset.geometry.triangles, source.geometry.triangles);
    assert_eq!(
        out.asset.geometry.material_coordinates,
        source.geometry.material_coordinates
    );
    assert_eq!(out.report.movable_vertices, 1);
    assert_eq!(out.report.moved_vertices, 1);
    assert_eq!(out.report.max_displacement, 0.25);
    assert_eq!(out.report.rms_displacement, 0.25);
    assert!(out.report.bends_after.maximum_degrees < out.report.bends_before.maximum_degrees);
    assert!(out.report.bends_after.mean_degrees < out.report.bends_before.mean_degrees);
    assert_eq!(out.report.component_volumes.len(), 1);
    assert!(!out.report.component_volumes[0].closed);
    assert_eq!(out.report.component_volumes[0].signed_before, None);
    let again = mesh_surface::smooth_region(&source, &[0], &[], &options()).unwrap();
    assert_eq!(out.asset, again.asset);
    assert_eq!(out.report, again.report);
}
#[test]
fn real_boundaries_and_explicit_anchors_pin_and_input_order_does_not_matter() {
    let source = fan();
    let selected = [4, 0, 1, 3, 2];
    let out = mesh_surface::smooth_region(&source, &selected, &[], &options()).unwrap();
    assert_eq!(out.report.boundary_anchors, 4);
    assert_eq!(out.report.movable_vertices, 1);
    assert_eq!(
        &out.asset.geometry.positions[1..],
        &source.geometry.positions[1..]
    );
    let ordered = mesh_surface::smooth_region(&source, &[0, 1, 2, 3, 4], &[], &options()).unwrap();
    assert_eq!(out.asset, ordered.asset);
    assert_eq!(out.report, ordered.report);
    let pinned = mesh_surface::smooth_region(&source, &selected, &[0], &options()).unwrap();
    assert_eq!(pinned.asset, source);
    assert!(Arc::ptr_eq(&pinned.asset.geometry, &source.geometry));
    assert_eq!(pinned.report.moved_vertices, 0);
    assert_eq!(pinned.report.max_displacement, 0.);
    assert!(pinned.report.component_volumes.is_empty());
}
#[test]
fn total_envelope_is_checked_after_quantization_across_all_iterations() {
    let source = fan();
    let mut o = options();
    o.iterations = 32;
    o.strength = 1.;
    o.max_displacement = 0.1;
    let out = mesh_surface::smooth_region(&source, &[0], &[], &o).unwrap();
    assert!(out.report.max_displacement <= 0.100001);
    assert!(out.report.max_displacement > 0.09999);
    assert!((out.asset.geometry.positions[0][1] - 0.4).abs() < 1e-6);
}
#[test]
fn an_explicit_interior_anchor_stays_fixed_while_its_selected_neighbors_move() {
    let mut g = Geometry::grid(4, 2.).unwrap();
    g.positions[12][1] = 0.5;
    let source = asset(g.positions, g.triangles);
    let out = mesh_surface::smooth_region(&source, &[12, 6], &[6], &options()).unwrap();
    assert_eq!(
        out.asset.geometry.positions[6],
        source.geometry.positions[6]
    );
    assert_eq!(out.report.explicit_anchors, 1);
    assert_eq!(out.report.boundary_anchors, 0);
    assert_eq!(out.report.movable_vertices, 1);
    assert_eq!(out.report.moved_vertices, 1);
    assert!(out.asset.geometry.positions[12][1] < source.geometry.positions[12][1]);
}
#[test]
fn nearby_disconnected_island_is_never_smoothed_or_welded() {
    let mut source = fan();
    let mut g = (*source.geometry).clone();
    let other = g.clone();
    g.positions
        .extend(other.positions.iter().map(|p| [p[0], p[1] + 0.01, p[2]]));
    g.triangles
        .extend(other.triangles.iter().map(|t| t.map(|i| i + 5)));
    g.material_coordinates
        .as_mut()
        .unwrap()
        .extend_from_slice(other.material_coordinates.as_ref().unwrap());
    source.geometry = Arc::new(g);
    let out = mesh_surface::smooth_region(&source, &[0], &[], &options()).unwrap();
    assert_eq!(
        &out.asset.geometry.positions[1..],
        &source.geometry.positions[1..]
    );
    assert_eq!(out.asset.geometry.triangles, source.geometry.triangles);
    assert_eq!(out.report.component_volumes.len(), 1);
    let a = Topology::inspect(&source.geometry).unwrap();
    let b = Topology::inspect(&out.asset.geometry).unwrap();
    assert_eq!(a.edges, b.edges);
    assert_eq!(a.boundary_edges, b.boundary_edges);
}
#[test]
fn angles_include_neighbor_faces_across_edges_with_unselected_endpoints() {
    let mut source = fan();
    let g = Arc::make_mut(&mut source.geometry);
    g.positions
        .extend([[2., 0., 0.], [0., 0., 2.], [-2., 0., 0.], [0., 0., -2.]]);
    for j in 0..4 {
        let a = 1 + j;
        let b = 1 + (j + 1) % 4;
        g.triangles.extend([[a, b, b + 4], [a, b + 4, a + 4]]);
    }
    g.material_coordinates = None;
    let out = mesh_surface::smooth_region(&source, &[0], &[], &options()).unwrap();
    assert_eq!(out.report.affected_faces, 4);
    assert_eq!(out.report.affected_edges, 8);
    assert_eq!(
        &out.asset.geometry.positions[1..],
        &source.geometry.positions[1..]
    );
}
#[test]
fn closed_component_reports_real_shrinkage_instead_of_claiming_volume_preservation() {
    let source = octahedron();
    let mut o = options();
    o.strength = 0.1;
    let out = mesh_surface::smooth_region(&source, &[0, 1, 2, 3, 4, 5], &[], &o).unwrap();
    let volume = &out.report.component_volumes[0];
    assert!(volume.closed && volume.oriented);
    let before = volume.signed_before.unwrap();
    let after = volume.signed_after.unwrap();
    assert!((before - 4. / 3.).abs() < 1e-12);
    assert!((after / before - 0.9_f64.powi(3)).abs() < 1e-6);
    assert!(after < before);
    assert_eq!(out.report.bends_after.fold_edges, 0);
}
#[test]
fn invalid_selection_options_and_topology_leave_source_exact() {
    let source = fan();
    let before = source.clone();
    for case in 0..12 {
        let mut selected = vec![0];
        let mut anchors = vec![];
        let mut o = options();
        match case {
            0 => selected.clear(),
            1 => selected = vec![0, 0],
            2 => selected = vec![99],
            3 => anchors = vec![1],
            4 => anchors = vec![0, 0],
            5 => o.iterations = 0,
            6 => o.iterations = 33,
            7 => o.strength = 0.,
            8 => o.strength = 1.1,
            9 => o.strength = f32::NAN,
            10 => o.max_displacement = 0.,
            _ => o.max_displacement = f32::INFINITY,
        }
        assert!(
            mesh_surface::smooth_region(&source, &selected, &anchors, &o).is_err(),
            "case{case}"
        );
        assert_eq!(source, before);
    }
    let mut bad = source.clone();
    Arc::make_mut(&mut bad.geometry).triangles[0].swap(1, 2);
    assert!(
        mesh_surface::smooth_region(&bad, &[0], &[], &options())
            .unwrap_err()
            .to_string()
            .contains("winding")
    );
    let mut bad = source.clone();
    Arc::make_mut(&mut bad.geometry).triangles.push([0, 2, 1]);
    assert!(
        mesh_surface::smooth_region(&bad, &[0], &[], &options())
            .unwrap_err()
            .to_string()
            .contains("non-manifold")
    );
    let pinch = asset(
        vec![
            [0.; 3],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., 0., 1.],
            [-1., 0., 0.],
            [0., -1., 0.],
            [0., 0., -1.],
        ],
        vec![
            [0, 2, 1],
            [0, 1, 3],
            [0, 3, 2],
            [1, 2, 3],
            [0, 4, 5],
            [0, 6, 4],
            [0, 5, 6],
            [4, 6, 5],
        ],
    );
    assert!(
        mesh_surface::smooth_region(&pinch, &[0], &[], &options())
            .unwrap_err()
            .to_string()
            .contains("face fans")
    );
}
#[test]
fn strong_collapse_or_orientation_reversal_is_rejected() {
    let source = octahedron();
    let before = source.clone();
    let mut o = options();
    o.strength = 1.;
    o.max_displacement = 10.;
    assert!(
        mesh_surface::smooth_region(&source, &[0, 1, 2, 3, 4, 5], &[], &o)
            .unwrap_err()
            .to_string()
            .contains("degenerates")
    );
    assert_eq!(source, before);
    let mut source = fan();
    Arc::make_mut(&mut source.geometry).positions[0] = [2., 0., 0.2];
    assert!(
        mesh_surface::smooth_region(&source, &[0], &[], &o)
            .unwrap_err()
            .to_string()
            .contains("reverses")
    );
}
#[test]
fn isolated_dent_that_worsens_incident_peak_is_rejected() {
    let source = octahedron();
    let before = source.clone();
    let mut o = options();
    o.strength = 0.1;
    let error = mesh_surface::smooth_region(&source, &[0], &[], &o).unwrap_err();
    assert!(
        error.to_string().contains("worsens maximum or mean"),
        "{error}"
    );
    assert_eq!(source, before);
}
#[test]
fn rig_clip_channels_and_fixed_animated_poses_are_exact() {
    let mut source = fan();
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
            5
        ],
    }));
    source.clips.push(Arc::new(meshes::Clip {
        id: "turn".into(),
        duration_ticks: 120,
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
                    translation: [0.; 3],
                    rotation: [
                        0.,
                        0.,
                        std::f32::consts::FRAC_1_SQRT_2,
                        std::f32::consts::FRAC_1_SQRT_2,
                    ],
                },
            ],
        }],
    }));
    let out = mesh_surface::smooth_region(&source, &[0], &[], &options()).unwrap();
    assert!(Arc::ptr_eq(
        source.rig.as_ref().unwrap(),
        out.asset.rig.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(&source.clips[0], &out.asset.clips[0]));
    assert_eq!(
        out.asset.geometry.material_coordinates,
        source.geometry.material_coordinates
    );
    assert_eq!(out.asset.material, source.material);
    let instance = meshes::Instance {
        id: "instance".into(),
        asset: source.id.clone(),
        position: [1., 2., 3.],
        rotation: [0., 0., 0., 1.],
        scale: 2.,
        color: [1.; 3],
        playback: Some(meshes::Playback {
            clip: "turn".into(),
            offset_ticks: 0,
            looping: true,
        }),
        attachment: None,
    };
    let mut marker = asset(vec![[0.; 3], [0.1, 0., 0.], [0., 0.1, 0.]], vec![[0, 1, 2]]);
    marker.id = "marker".into();
    let mut child = instance.clone();
    child.id = "socket-child".into();
    child.asset = marker.id.clone();
    child.playback = None;
    child.position = [0.3, 0.2, 0.1];
    child.scale = 1.;
    child.attachment = Some(meshes::Attachment {
        parent: instance.id.clone(),
        bone: Some("root".into()),
    });
    let before = meshes::Document {
        objects: Default::default(),
        assets: vec![source, marker.clone()],
        instances: vec![instance.clone(), child.clone()],
    };
    let after = meshes::Document {
        objects: Default::default(),
        assets: vec![out.asset, marker],
        instances: vec![instance, child],
    };
    for tick in [0, 60] {
        let a = before.pose("instance", tick).unwrap();
        let b = after.pose("instance", tick).unwrap();
        assert_eq!(&a.positions[1..], &b.positions[1..]);
        assert_ne!(a.positions[0], b.positions[0]);
        assert_eq!(a.joints, b.joints);
        assert_eq!(a.world_transform, b.world_transform);
        let a = before.pose("socket-child", tick).unwrap();
        let b = after.pose("socket-child", tick).unwrap();
        assert_eq!(a.positions, b.positions);
        assert_eq!(a.world_transform, b.world_transform);
    }
}

#[test]
fn edge_ring_selection_is_exact_bounded_and_pins_outer_layers() {
    let g = Geometry::grid(4, 1.).unwrap();
    let before = g.clone();
    let one = mesh_surface::ring_region(&g, &[12], 1, 1).unwrap();
    assert_eq!(one.vertices, vec![7, 8, 11, 12, 13, 16, 17]);
    assert_eq!(one.anchors, vec![7, 8, 11, 13, 16, 17]);
    let two = mesh_surface::ring_region(&g, &[12], 2, 1).unwrap();
    assert_eq!(two.vertices.len(), 19);
    assert_eq!(two.anchors.len(), 12);
    let pinned = mesh_surface::ring_region(&g, &[12], 2, 2).unwrap();
    assert_eq!(pinned.vertices, two.vertices);
    assert_eq!(pinned.anchors.len(), 18);
    let expected = mesh_surface::ring_region(&g, &[12, 13], 2, 1).unwrap();
    let mut reversed = g.clone();
    reversed.triangles.reverse();
    assert_eq!(
        mesh_surface::ring_region(&reversed, &[13, 12], 2, 1).unwrap(),
        expected
    );
    assert_eq!(g, before);
}
#[test]
fn edge_rings_follow_indices_and_reject_ambiguous_or_unanchored_selections() {
    let mut g = Geometry::grid(4, 1.).unwrap();
    let original = g.clone();
    let n = g.positions.len() as u32;
    g.positions.extend_from_slice(&original.positions);
    g.triangles
        .extend(original.triangles.iter().map(|t| t.map(|i| i + n)));
    let region = mesh_surface::ring_region(&g, &[12], 2, 1).unwrap();
    assert!(region.vertices.iter().all(|&i| i < n));
    for (seeds, rings, pins) in [
        (vec![], 2, 1),
        (vec![12, 12], 2, 1),
        (vec![999], 2, 1),
        (vec![12], 0, 1),
        (vec![12], 17, 1),
        (vec![12], 2, 0),
        (vec![12], 2, 3),
        (vec![12], 16, 1),
        (vec![12], 5, 2),
    ] {
        assert!(mesh_surface::ring_region(&g, &seeds, rings, pins).is_err());
    }
}
