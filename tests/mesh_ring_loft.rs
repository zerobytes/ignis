use ignis::{
    engine::{Command, Engine},
    mesh_ring_loft::{self, Interpolation, Loft},
    mesh_topology::Topology,
    meshes::{Asset, Geometry, Shading},
    model::RoomRecipe,
};
use std::sync::Arc;
fn fixture() -> (Asset, Loft) {
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    let mut rings = Vec::new();
    for (y, x) in [(0., 0.), (1., 0.1), (2., 0.55), (3., 0.6)] {
        let start = positions.len() as u32;
        rings.push((start..start + 8).collect());
        for j in 0..8 {
            let a = j as f32 * std::f32::consts::TAU / 8.;
            positions.push([x + 0.08 * a.cos(), y, 0.08 * a.sin()]);
        }
    }
    for i in 0..3 {
        for j in 0..8 {
            let a = i * 8 + j;
            let b = i * 8 + (j + 1) % 8;
            triangles.extend([[a, b, a + 8], [b, b + 8, a + 8]]);
        }
    }
    for j in 1..7 {
        triangles.extend([[0, j + 1, j], [24, 24 + j, 25 + j]]);
    }
    let weights: Vec<_> = (0..32)
        .map(|i| ignis::meshes::Weight {
            extra: None,
            joints: [0, 1, 0, 0],
            weights: [1. - i as f32 / 31., i as f32 / 31., 0., 0.],
        })
        .collect();
    let rig=serde_json::from_value(serde_json::json!({"bones":[{"name":"base","parent":null,"translation":[0,0,0],"rotation":[0,0,0,1]},{"name":"socket","parent":0,"translation":[0,1,0],"rotation":[0,0,0,1]}],"weights":weights})).unwrap();
    let source = Asset {
        id: "tube".into(),
        geometry: Arc::new(Geometry {
            material_coordinates: Some(positions.clone()),
            positions,
            triangles,
        }),
        shading: Shading::Smooth,
        material: Default::default(),
        rig: Some(Arc::new(rig)),
        clips: vec![],
    };
    (
        source,
        Loft {
            rings,
            spans: 12,
            interpolation: Interpolation::Cubic,
            max_deviation: 0.2,
        },
    )
}
#[test]
fn cubic_sections_round_silhouette_and_preserve_source_channels_and_closed_topology() {
    let (source, loft) = fixture();
    let before = source.clone();
    let out = mesh_ring_loft::fit(&source, std::slice::from_ref(&loft)).unwrap();
    let n = source.geometry.positions.len();
    assert_eq!(source, before);
    assert_eq!(
        &out.asset.geometry.positions[..n],
        &source.geometry.positions
    );
    assert_eq!(
        &out.asset.geometry.material_coordinates.as_ref().unwrap()[..n],
        source.geometry.material_coordinates.as_ref().unwrap()
    );
    assert_eq!(
        &out.asset.rig.as_ref().unwrap().weights[..n],
        &source.rig.as_ref().unwrap().weights
    );
    assert_eq!(
        out.asset.rig.as_ref().unwrap().bones,
        source.rig.as_ref().unwrap().bones
    );
    assert_eq!(out.asset.clips, source.clips);
    assert_eq!(out.report.added_vertices, 3 * 11 * 8);
    assert!(out.report.sampled_deviation > 0.01);
    assert!(out.report.sampled_deviation <= out.report.curve_deviation_bound + 1e-6);
    let topo = Topology::inspect(&out.asset.geometry).unwrap();
    assert_eq!(topo.boundary_edges, 0);
    assert_eq!(topo.non_manifold_edges, 0);
    assert!(topo.isolated_vertices.is_empty());
    assert_eq!(
        topo.vertices as i64 - topo.edges.len() as i64 + topo.faces.len() as i64,
        2
    );
    // A straight variant is available by intent; densification alone has no rounding.
    let linear = mesh_ring_loft::fit(
        &source,
        &[Loft {
            interpolation: Interpolation::Linear,
            ..loft
        }],
    )
    .unwrap();
    assert_eq!(linear.report.curve_deviation_bound, 0.);
    assert!(linear.report.sampled_deviation < 1e-6);
    assert_ne!(
        linear.asset.geometry.positions,
        out.asset.geometry.positions
    );
}
#[test]
fn malformed_topology_parameters_and_envelope_refuse_without_mutation() {
    let (source, loft) = fixture();
    let before = source.clone();
    for case in 0..7 {
        let mut bad = loft.clone();
        match case {
            0 => bad.spans = 0,
            1 => bad.rings[1][0] = bad.rings[0][0],
            2 => bad.rings[1].reverse(),
            3 => bad.max_deviation = 0.,
            4 => bad.rings[0][0] = 9999,
            5 => bad.rings[0].pop().map(|_| ()).unwrap(),
            _ => bad.spans = 17,
        };
        assert!(mesh_ring_loft::fit(&source, &[bad]).is_err(), "case{case}");
        assert_eq!(source, before);
    }
    assert!(mesh_ring_loft::fit(&source, &[loft.clone(), loft]).is_err());
}
#[test]
fn typed_dispatcher_preview_is_pure_and_stable() {
    let (source, loft) = fixture();
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    let state = e.snapshot();
    let command = Command::PreviewMeshRingLoft {
        source: Box::new(source),
        lofts: vec![loft],
    };
    let a = e.execute(command.clone().into()).unwrap();
    let b = e.execute(command.into()).unwrap();
    assert_eq!(a, b);
    assert_eq!(e.snapshot(), state);
}
