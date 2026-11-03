use ignis::{
    mesh_material_domain::{self as domain, Domain, Options},
    meshes::Geometry,
};
fn triangle() -> Geometry {
    Geometry {
        positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        triangles: vec![[0, 1, 2]],
        material_coordinates: None,
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-8 * b.abs().max(1.), "{a} != {b}");
}
#[test]
fn identity_fallback_explicit_isometry_and_uniform_scale_have_known_units() {
    let mut g = triangle();
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    assert_eq!(r.coordinates, "local_positions_fallback");
    close(r.local_area, 0.5);
    close(r.minimum_rate.value, 1.);
    close(r.maximum_rate.value, 1.);
    close(r.area_weighted_mean_anisotropy.unwrap(), 1.);
    g.material_coordinates = Some(vec![[3., 4., 5.], [3., 4., 7.], [5., 4., 5.]]);
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    assert_eq!(r.coordinates, "explicit");
    close(r.minimum_rate.value, 2.);
    close(r.maximum_rate.value, 2.);
    close(r.samples[0].area_rate, 4.);
    g.positions = vec![[0., 0., 0.], [0., 0., 3.], [3., 0., 0.]];
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    close(r.minimum_rate.value, 2. / 3.);
    close(r.maximum_anisotropy.unwrap().value, 1.);
}
#[test]
fn shear_and_axis_scale_use_principal_directions_not_only_edges() {
    let mut g = triangle();
    g.material_coordinates = Some(vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]]);
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    close(
        r.minimum_rate.value,
        (3_f64 - 5_f64.sqrt()).sqrt() / 2_f64.sqrt(),
    );
    close(
        r.maximum_rate.value,
        (3_f64 + 5_f64.sqrt()).sqrt() / 2_f64.sqrt(),
    );
    close(r.samples[0].area_rate, 1.);
    let r = domain::inspect(
        &triangle(),
        None,
        &Options {
            scale: [4., 2., 3.],
            ..Options::default()
        },
    )
    .unwrap();
    close(r.minimum_rate.value, 2.);
    close(r.maximum_rate.value, 4.);
    close(r.maximum_anisotropy.unwrap().value, 2.);
}
#[test]
fn planar_projections_and_collapsed_domains_are_observable_without_infinities() {
    let g = triangle();
    for (projection, rank) in [
        (Domain::Spatial, 2),
        (Domain::Xy, 2),
        (Domain::Xz, 1),
        (Domain::Yz, 1),
    ] {
        let r = domain::inspect(
            &g,
            None,
            &Options {
                domain: projection,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(r.samples[0].rank, rank);
        assert_eq!(r.rank_one_faces, usize::from(rank == 1));
        assert_eq!(r.maximum_anisotropy.is_none(), rank == 1);
        serde_json::to_value(&r).unwrap();
    }
    let mut g = g;
    g.material_coordinates = Some(vec![[7., 7., 7.]; 3]);
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    assert_eq!(r.rank_zero_faces, 1);
    assert_eq!(r.near_singular_faces, 1);
    assert_eq!(r.regular_area, 0.);
    assert_eq!(r.area_weighted_mean_anisotropy, None);
    assert_eq!(r.samples[0].rates, [0., 0.]);
    g.material_coordinates = Some(vec![[0., 0., 0.], [1., 0., 0.], [0., 1e-11, 0.]]);
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    assert_eq!(r.near_singular_faces, 1);
    assert_eq!(r.samples[0].rank, 2);
    assert!(r.samples[0].anisotropy.is_none());
    assert_eq!(r.area_weighted_mean_anisotropy, None);
}
#[test]
fn full_selection_statistics_are_area_weighted_even_with_zero_samples() {
    let mut g = triangle();
    g.positions
        .extend([[3., 0., 0.], [5., 0., 0.], [3., 1., 0.]]);
    g.triangles.push([3, 4, 5]);
    g.material_coordinates = Some(vec![
        [0., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        [0., 0., 0.],
        [4., 0., 0.],
        [0., 1., 0.],
    ]);
    let r = domain::inspect(
        &g,
        None,
        &Options {
            sample_limit: 0,
            ..Options::default()
        },
    )
    .unwrap();
    close(r.local_area, 1.5);
    close(r.area_weighted_mean_anisotropy.unwrap(), 5. / 3.);
    assert!(r.samples.is_empty());
    let r = domain::inspect(
        &g,
        Some(&[1, 0]),
        &Options {
            sample_limit: 1,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(r.samples.len(), 1);
    assert_eq!(r.samples[0].face, 1);
    assert_eq!(
        r,
        domain::inspect(
            &g,
            Some(&[0, 1]),
            &Options {
                sample_limit: 1,
                ..Options::default()
            }
        )
        .unwrap()
    );
    close(
        domain::inspect(&g, Some(&[0]), &Options::default())
            .unwrap()
            .local_area,
        0.5,
    );
}
#[test]
fn triangle_winding_basis_and_subdivision_do_not_change_affine_rates() {
    let mut g = triangle();
    g.positions = vec![[0., 0., 0.], [1., 0., 1.], [0., 1., 0.]];
    g.material_coordinates = Some(vec![[0., 0., 0.], [3., 0., 0.], [0., 2., 0.]]);
    let before = g.clone();
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    for indices in [[1, 2, 0], [2, 1, 0], [0, 2, 1]] {
        g.triangles[0] = indices;
        let a = domain::inspect(&g, None, &Options::default()).unwrap();
        close(a.minimum_rate.value, r.minimum_rate.value);
        close(a.maximum_rate.value, r.maximum_rate.value);
    }
    g = before.clone();
    g.positions.push([0.5, 0., 0.5]);
    g.material_coordinates.as_mut().unwrap().push([1.5, 0., 0.]);
    g.triangles = vec![[0, 3, 2], [3, 1, 2]];
    let a = domain::inspect(&g, None, &Options::default()).unwrap();
    close(
        a.area_weighted_mean_anisotropy.unwrap(),
        r.area_weighted_mean_anisotropy.unwrap(),
    );
    assert_eq!(before.positions.len(), 3);
}
#[test]
fn invalid_inputs_refuse_and_leave_sources_exact() {
    let g = triangle();
    let before = g.clone();
    for faces in [vec![], vec![0, 0], vec![1]] {
        assert!(domain::inspect(&g, Some(&faces), &Options::default()).is_err());
    }
    for o in [
        Options {
            sample_limit: 33,
            ..Options::default()
        },
        Options {
            scale: [0., 1., 1.],
            ..Options::default()
        },
        Options {
            scale: [-1., 1., 1.],
            ..Options::default()
        },
        Options {
            scale: [f32::NAN, 1., 1.],
            ..Options::default()
        },
        Options {
            scale: [f32::INFINITY, 1., 1.],
            ..Options::default()
        },
    ] {
        assert!(domain::inspect(&g, None, &o).is_err());
    }
    let mut bad = g.clone();
    bad.material_coordinates = Some(vec![]);
    assert!(domain::inspect(&bad, None, &Options::default()).is_err());
    assert_eq!(g, before);
}

#[test]
fn numerical_admission_is_relative_to_shape_not_absolute_mesh_scale() {
    let mut g = triangle();
    g.positions = vec![[0., 0., 0.], [1., 1., 0.], [2., 2., 1e-16]];
    let error = domain::inspect(&g, None, &Options::default()).unwrap_err();
    assert!(error.message.contains("face 0") && error.message.contains("ill-conditioned"));
    g = triangle();
    for p in &mut g.positions {
        for c in p {
            *c *= 1e-8;
        }
    }
    let r = domain::inspect(&g, None, &Options::default()).unwrap();
    close(r.minimum_rate.value, 1.);
    close(r.maximum_rate.value, 1.);
}
