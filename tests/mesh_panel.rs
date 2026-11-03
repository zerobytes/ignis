use glam::DVec3;
use ignis::{
    engine::{Command, Engine},
    mesh_panel::{Opening, build},
    mesh_topology::Topology,
    meshes::Geometry,
    model::RoomRecipe,
};
use std::collections::BTreeMap;

fn opening(min: [f32; 2], max: [f32; 2]) -> Opening {
    Opening { min, max }
}
fn verify(g: &Geometry, expected_volume: f64) {
    let t = Topology::inspect(g).unwrap();
    assert_eq!((t.boundary_edges, t.non_manifold_edges), (0, 0));
    assert!(t.isolated_vertices.is_empty());
    let mut edges = BTreeMap::<(u32, u32), i32>::new();
    let mut volume = 0.;
    for &[a, b, c] in &g.triangles {
        for (u, v) in [(a, b), (b, c), (c, a)] {
            *edges.entry((u.min(v), u.max(v))).or_default() += if u < v { 1 } else { -1 };
        }
        let [a, b, c] =
            [a, b, c].map(|i| DVec3::from_array(g.positions[i as usize].map(f64::from)));
        volume += a.dot(b.cross(c)) / 6.;
    }
    assert!(edges.values().all(|&sum| sum == 0));
    assert!(
        (volume - expected_volume).abs() < expected_volume * 1e-5,
        "volume {volume} vs {expected_volume}"
    );
}
// Independent ray/triangle intersection, along Z from an arbitrary XY point.
fn hits(g: &Geometry, x: f64, y: f64) -> usize {
    let o = DVec3::new(x, y, -200.);
    let d = DVec3::Z;
    g.triangles
        .iter()
        .filter(|face| {
            let [a, b, c] = face.map(|i| DVec3::from_array(g.positions[i as usize].map(f64::from)));
            let e1 = b - a;
            let e2 = c - a;
            let h = d.cross(e2);
            let det = e1.dot(h);
            if det.abs() < 1e-12 {
                return false;
            }
            let s = o - a;
            let u = s.dot(h) / det;
            let q = s.cross(e1);
            let v = d.dot(q) / det;
            u >= 0. && v >= 0. && u + v <= 1. && e2.dot(q) / det > 0.
        })
        .count()
}
#[test]
fn material_solid_with_door_and_window_has_real_voids() {
    let g = build(
        [6., 4., 0.3],
        &[
            opening([-2., -2.], [-1., 0.5]),
            opening([0., -0.5], [1.5, 1.]),
        ],
    )
    .unwrap();
    verify(&g, (24. - 2.5 - 2.25) * 0.3);
    assert_eq!(hits(&g, -1.4, -1.3), 0);
    assert_eq!(hits(&g, 0.61, 0.27), 0);
    assert_eq!(hits(&g, 2.17, 0.21), 2);
    assert_eq!(hits(&g, -1.43, 1.21), 2); // lintel above door
}
#[test]
fn same_operation_handles_different_panels_and_full_grid_extremes() {
    verify(&build([0.01, 0.01, 0.01], &[]).unwrap(), 1e-6);
    verify(
        &build([2., 8., 0.8], &[opening([-0.6, -2.], [0.7, 2.])]).unwrap(),
        (16. - 5.2) * 0.8,
    );
    let holes: Vec<_> = (0..16)
        .map(|i| {
            let x = -7.5 + i as f32 * 0.9;
            let y = -7.6 + i as f32 * 0.8;
            opening([x, y], [x + 0.3, y + 0.4])
        })
        .collect();
    let g = build([20., 20., 0.02], &holes).unwrap();
    verify(&g, (400. - 16. * 0.12) * 0.02);
    assert!(g.positions.len() <= 2312);
    assert_eq!(
        build([6., 4., 0.3], &[]).unwrap().positions,
        build([6., 4., 0.3], &[]).unwrap().positions
    );
}
#[test]
fn contradictory_or_degenerate_requests_are_rejected() {
    for dimensions in [[0., 4., 1.], [1., f32::NAN, 1.], [1., 1., 101.]] {
        assert!(build(dimensions, &[]).is_err());
    }
    for holes in [
        vec![opening([0., 0.], [0., 1.])],
        vec![opening([-3., 0.], [-2., 1.])], // side perimeter
        vec![opening([0., 1.], [1., 2.])],   // top perimeter
        vec![opening([0., -3.], [1., 1.])],
        vec![opening([0., 0.], [1., 1.]), opening([0.5, 0.5], [1.5, 1.5])],
        vec![opening([0., 0.], [1., 1.]), opening([1., 0.], [2., 1.])],
        vec![opening([f32::NAN, 0.], [1., 1.])],
        vec![opening([0., 0.], [1., 1.]); 17],
    ] {
        assert!(build([6., 4., 0.3], &holes).is_err());
    }
}
#[test]
fn shared_dispatcher_preview_undo_and_invalid_edit_preserve_state() {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    let command = |op: &str| {
        serde_json::from_value::<Command>(serde_json::json!({"op":op,"edit":{"kind":"panel","id":"wall","dimensions":[6,4,0.3],"openings":[{"min":[-1,-2],"max":[1,0.5]}]}})).unwrap()
    };
    let before = e.execute(Command::Inspect.into()).unwrap();
    let preview = e.execute(command("preview_mesh_edit").into()).unwrap();
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    e.execute(command("edit_mesh").into()).unwrap();
    let asset = e
        .execute(
            Command::InspectMesh {
                asset: "wall".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(asset["asset"], preview["meshes"]["assets"][0]);
    let snapshot = e.execute(Command::Inspect.into()).unwrap();
    let bad=serde_json::from_value::<Command>(serde_json::json!({"op":"edit_mesh","edit":{"kind":"panel","id":"wall","dimensions":[0,4,0.3]}})).unwrap();
    assert!(e.execute(bad.into()).is_err());
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), snapshot);
    e.execute(Command::Undo.into()).unwrap();
    assert!(
        e.execute(
            Command::InspectMesh {
                asset: "wall".into()
            }
            .into()
        )
        .is_err()
    );
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(
        e.execute(
            Command::InspectMesh {
                asset: "wall".into()
            }
            .into()
        )
        .unwrap(),
        asset
    );
}
