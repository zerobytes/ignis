use ignis::{
    mesh_chamfer::{self, Config},
    meshes::Geometry,
};
use std::collections::BTreeMap;
fn cube() -> Geometry {
    Geometry {
        positions: vec![
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
        ],
        triangles: vec![
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [3, 7, 6],
            [3, 6, 2],
            [0, 4, 7],
            [0, 7, 3],
            [1, 2, 6],
            [1, 6, 5],
        ],
        material_coordinates: None,
    }
}
#[test]
fn seeded_cuts_are_closed_inward_and_reproducible() {
    let g = cube();
    let c = Config {
        fraction: 0.18,
        variation: 0.65,
        seed: 2718,
    };
    let a = mesh_chamfer::apply(&g, &c).unwrap();
    let b = mesh_chamfer::apply(&g, &c).unwrap();
    assert_eq!(a.positions, b.positions);
    assert_eq!(a.triangles, b.triangles);
    assert!(a.triangles.len() > g.triangles.len());
    assert!(a.positions.iter().all(|p| p.iter().all(|v| v.abs() <= 1.)));
    let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
    for t in &a.triangles {
        for k in 0..3 {
            let (x, y) = (t[k], t[(k + 1) % 3]);
            let e = edges.entry((x.min(y), x.max(y))).or_default();
            e.0 += 1;
            e.1 += if x < y { 1 } else { -1 };
        }
    }
    assert!(edges.values().all(|&(n, d)| n == 2 && d == 0));
    let changed = mesh_chamfer::apply(&g, &Config { seed: 42, ..c }).unwrap();
    assert_ne!(a.positions, changed.positions);
    // Positive signed volume and less than original8: genuine removed corners.
    let volume = a
        .triangles
        .iter()
        .map(|t| {
            let p = t.map(|i| glam::Vec3::from_array(a.positions[i as usize]));
            p[0].dot(p[1].cross(p[2])) / 6.
        })
        .sum::<f32>();
    assert!(volume > 0. && volume < 8.);
}
#[test]
fn boundary_positions_stay_exact_and_invalid_parameters_refuse() {
    let g = Geometry {
        positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., 1.]],
        triangles: vec![[0, 2, 1]],
        material_coordinates: None,
    };
    let c = Config {
        fraction: 0.1,
        variation: 0.5,
        seed: 1,
    };
    let a = mesh_chamfer::apply(&g, &c).unwrap();
    for p in &g.positions {
        assert!(a.positions.contains(p));
    }
    assert_eq!(a.triangles.len(), 1);
    assert!(
        mesh_chamfer::apply(
            &g,
            &Config {
                fraction: f32::NAN,
                ..c
            }
        )
        .is_err()
    );
}
