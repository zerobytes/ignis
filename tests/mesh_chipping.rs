use glam::DVec3;
use ignis::{
    mesh_modeling::{self, Operation},
    mesh_tools::{self, Primitive},
    meshes::Asset,
};
use std::sync::Arc;

fn cube() -> Asset {
    Asset {
        id: "stone".into(),
        geometry: Arc::new(mesh_tools::primitive(Primitive::Cube, 4, 2.).unwrap()),
        shading: Default::default(),
        material: Default::default(),
        rig: None,
        clips: vec![],
    }
}
fn box_points(min: [f32; 3], max: [f32; 3]) -> Vec<[f32; 3]> {
    (0..8)
        .map(|i| std::array::from_fn(|a| if i & (1 << a) == 0 { min[a] } else { max[a] }))
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
#[test]
fn localized_subtraction_and_repeated_concave_cut_have_analytic_volumes() {
    let a = cube();
    let operation = Operation::SubtractConvex {
        points: box_points([0.2, -0.7, 0.4], [1.2, 0.6, 1.3]),
    };
    let r = mesh_modeling::evaluate(&a, &operation).unwrap();
    assert!(
        (volume(&r.asset) - (8. - 0.8 * 1.3 * 0.6)).abs() < 1e-5,
        "{}",
        volume(&r.asset)
    );
    let r2 = mesh_modeling::evaluate(&a, &operation).unwrap();
    assert_eq!(r.asset.geometry, r2.asset.geometry);
    assert!(!r.report.selected_faces.is_empty());
    // Untouched corners remain exact; the kernel may simplify coplanar interiors.
    for p in &a.geometry.positions {
        if p.iter().all(|x| x.abs() == 1.) && (p[0] == -1. || p[2] == -1.) {
            assert!(r.asset.geometry.positions.contains(p));
        }
    }
    let second = Operation::SubtractConvex {
        points: box_points([-0.8, -0.4, 0.7], [0.6, 0.3, 1.4]),
    };
    let s = mesh_modeling::evaluate(&r.asset, &second).unwrap();
    // Overlap with first removed box: [0.2,0.6] x [-0.4,0.3] x [0.7,1].
    let expected = 8. - 0.8 * 1.3 * 0.6 - 1.4 * 0.7 * 0.3 + 0.4 * 0.7 * 0.3;
    assert!(
        (volume(&s.asset) - expected).abs() < 2e-5,
        "{} != {expected}",
        volume(&s.asset)
    );
    let t = ignis::mesh_topology::Topology::inspect(&s.asset.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);

    // A ray through the recess now reaches its back plane at z=0.4;
    // an untouched region still hits the original front at z=1.
    use rapier3d::{
        na::{Point3, Vector3},
        parry::{
            query::{Ray, RayCast},
            shape::TriMesh,
        },
    };
    let mesh = TriMesh::new(
        r.asset
            .geometry
            .positions
            .iter()
            .map(|p| Point3::from(*p))
            .collect(),
        r.asset.geometry.triangles.clone(),
    )
    .unwrap();
    for (x, expected) in [(0.5, 1.6), (-0.5, 1.)] {
        let hit = mesh
            .cast_local_ray_and_get_normal(
                &Ray::new(Point3::new(x, 0., 2.), Vector3::new(0., 0., -1.)),
                4.,
                false,
            )
            .unwrap();
        assert!((hit.time_of_impact - expected).abs() < 1e-5);
        assert!(hit.normal.z > 0.99);
    }
}

#[test]
fn rejects_unsupported_or_nonlocal_ambiguous_operations() {
    let a = cube();
    for points in [
        vec![[0.; 3]; 4],
        vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        box_points([1., -0.5, -0.5], [2., 0.5, 0.5]), // tangency/no removal
        box_points([2., 2., 2.], [3., 3., 3.]),       // disjoint
        box_points([-2.; 3], [2.; 3]),                // empty
        box_points([-0.2; 3], [0.2; 3]),              // enclosed cavity
        box_points([-0.1, -2., -2.], [0.1, 2., 2.]),  // two fragments
    ] {
        assert!(mesh_modeling::evaluate(&a, &Operation::SubtractConvex { points }).is_err());
    }
    let mut open = a.clone();
    Arc::make_mut(&mut open.geometry).triangles.pop();
    let op = Operation::SubtractConvex {
        points: box_points([0.2, -0.7, 0.4], [1.2, 0.6, 1.3]),
    };
    assert!(mesh_modeling::evaluate(&open, &op).is_err());
    let mut custom = a.clone();
    Arc::make_mut(&mut custom.geometry).material_coordinates = Some(
        custom
            .geometry
            .positions
            .iter()
            .map(|p| p.map(|x| x * 2.))
            .collect(),
    );
    assert!(mesh_modeling::evaluate(&custom, &op).is_err());

    let mut crossing = a.clone();
    let index = crossing
        .geometry
        .positions
        .iter()
        .position(|p| *p == [-1., -1., -1.])
        .unwrap();
    Arc::make_mut(&mut crossing.geometry).positions[index] = [1.5, 1.2, 1.4];
    let error = mesh_modeling::evaluate(&crossing, &op)
        .err()
        .expect("crossed closed surface must fail");
    assert!(
        error.message.contains("self-intersect"),
        "{}",
        error.message
    );
    let mut large = a.clone();
    large.geometry = Arc::new(mesh_tools::primitive(Primitive::Cube, 32, 2.).unwrap());
    assert!(
        mesh_modeling::evaluate(&large, &op)
            .err()
            .unwrap()
            .message
            .contains("budget")
    );
    let oversized = serde_json::json!({"kind":"subtract_convex","points":vec![[0.,0.,0.];33]});
    assert!(serde_json::from_value::<Operation>(oversized).is_err());
}

#[test]
fn normalized_kernel_preserves_translated_source_and_original_material() {
    let mut a = cube();
    for p in &mut Arc::make_mut(&mut a.geometry).positions {
        *p = [p[0] * 3. + 3500., p[1] * 3. - 1200., p[2] * 3. + 900.];
    }
    let operation = Operation::SubtractConvex {
        points: box_points([3500.6, -1202.1, 901.2], [3503.6, -1198.2, 903.9]),
    };
    let r = mesh_modeling::evaluate(&a, &operation).unwrap();
    assert_eq!(r.asset.material, a.material);
    assert!(r.asset.geometry.positions.contains(&[3497., -1203., 897.]));
    // Translation quantizes f32 input; use an independent local-coordinate volume.
    let mut local = r.asset.clone();
    for p in &mut Arc::make_mut(&mut local.geometry).positions {
        *p = [p[0] - 3500., p[1] + 1200., p[2] - 900.];
    }
    assert!((volume(&local) - (8. - 0.8 * 1.3 * 0.6) * 27.).abs() < 0.005);
}

#[test]
fn a_small_corner_chip_is_not_rejected_as_a_fraction_of_the_entire_mass() {
    let a = cube();
    let r = mesh_modeling::evaluate(
        &a,
        &Operation::SubtractConvex {
            points: box_points([0.99, 0.99, 0.99], [1.01, 1.01, 1.01]),
        },
    )
    .unwrap();
    assert!((8. - volume(&r.asset) - 1e-6).abs() < 1e-10);
    assert!(r.report.numerical_tolerance_meters.unwrap() < 1e-6);
}

#[test]
fn convex_union_adds_real_closed_volume_and_supports_repeated_nonconvex_results() {
    let a = cube();
    let op = Operation::UnionConvex {
        points: box_points([0.4, -0.7, 0.4], [1.6, 0.6, 1.3]),
    };
    let r = mesh_modeling::evaluate(&a, &op).unwrap();
    let expected = 8. + 1.2 * 1.3 * 0.9 - 0.6 * 1.3 * 0.6;
    assert!((volume(&r.asset) - expected).abs() < 2e-5);
    assert!(!r.report.selected_faces.is_empty());
    assert_eq!(
        r.asset.geometry,
        mesh_modeling::evaluate(&a, &op).unwrap().asset.geometry
    );
    let topology = ignis::mesh_topology::Topology::inspect(&r.asset.geometry).unwrap();
    assert_eq!(topology.boundary_edges, 0);
    assert_eq!(topology.non_manifold_edges, 0);
    let r2 = mesh_modeling::evaluate(
        &r.asset,
        &Operation::UnionConvex {
            points: box_points([-1.3, -0.6, 0.3], [-0.4, 0.3, 1.4]),
        },
    )
    .unwrap();
    assert!(volume(&r2.asset) > volume(&r.asset));
    for points in [
        box_points([-0.2; 3], [0.2; 3]),
        box_points([2.; 3], [3.; 3]),
        box_points([1.; 3], [2.; 3]),
    ] {
        assert!(mesh_modeling::evaluate(&a, &Operation::UnionConvex { points }).is_err());
    }
}
