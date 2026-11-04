use ignis::{
    landforms::{self, Document, Domain, Edit, Piece, Ray, Section},
    world_space::{Coordinates, Position},
};

fn origin() -> Position {
    Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [0.; 3],
    })
    .unwrap()
}
fn section(at: f64, scale: f64, radii: &[f64]) -> Section {
    Section {
        at,
        scale,
        offset: [0.; 2],
        radial_scales: radii.to_vec(),
    }
}
fn source() -> Document {
    Document {
        version: 2,
        space_version: 1,
        id: "strata".into(),
        revision: 1,
        origin: origin(),
        domain: None,
        pieces: vec![Piece {
            id: "cliff".into(),
            anchor: [0.; 3],
            axis: [0., 1., 0.],
            profile_x: [1., 0., 0.],
            profile: vec![[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]],
            inner_scale: None,
            sections: vec![
                section(0., 4., &[1.; 4]),
                section(4., 3., &[1., 0.8, 1.1, 0.9]),
                section(4., 5., &[1.; 4]),
                section(8., 5., &[0.9, 1.1, 1.2, 0.8]),
            ],
        }],
    }
}
fn ray(at: [f64; 3]) -> Ray {
    Ray {
        origin: at,
        direction: [0., -1., 0.],
        max_distance_meters: 100.,
    }
}
fn closed(d: &Document) -> landforms::Compiled {
    let c = landforms::compile(d).unwrap();
    let t = ignis::mesh_topology::Topology::inspect(&c.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);
    assert!(
        c.parts
            .iter()
            .all(|p| p.signed_volume_cubic_meters > 0. && p.closed_oriented)
    );
    c
}

#[test]
fn varying_contours_keep_actual_steps_undercuts_and_oriented_surface_metrics() {
    let d = source();
    let c = closed(&d);
    let h = c.intersections(&ray([4.3, 20., 0.])).unwrap();
    assert_eq!(h.len(), 2);
    assert!((h[0].altitude_meters - 8.).abs() < 1e-6);
    assert!((h[1].altitude_meters - 4.).abs() < 1e-6);
    assert!(h[0].slope_degrees < 1e-6 && h[1].slope_degrees > 179.999);
    let mut invalid = d.clone();
    invalid.pieces[0].sections[2].radial_scales = vec![0.3, 1., 1., 1.];
    assert!(
        invalid.validate().is_err(),
        "Crossing horizontal steps must fail"
    );
    invalid = d.clone();
    invalid.pieces[0].sections[1].radial_scales.pop();
    assert!(invalid.validate().is_err());
    invalid = d.clone();
    invalid.pieces[0].sections[0].radial_scales[0] = f64::NAN;
    assert!(invalid.validate().is_err());
    let mut hollow = d.clone();
    hollow.pieces[0].inner_scale = Some(0.4);
    let hollow = closed(&hollow);
    assert!(
        hollow
            .intersections(&ray([0., 20., 0.]))
            .unwrap()
            .is_empty()
    );
    assert_eq!(hollow.intersections(&ray([4.3, 20., 0.])).unwrap().len(), 2);
}

#[test]
fn endpoint_caps_follow_the_new_concave_outline_instead_of_the_original_triangulation() {
    let mut d = source();
    let n = 12;
    d.pieces[0].profile = (0..n)
        .map(|i| {
            let a = i as f64 * std::f64::consts::TAU / n as f64;
            [a.cos(), a.sin()]
        })
        .collect();
    let radii: Vec<_> = (0..n).map(|i| if i % 2 == 0 { 1. } else { 0.3 }).collect();
    d.pieces[0].sections = vec![section(0., 5., &radii), section(8., 5., &radii)];
    let c = closed(&d);
    let outline: Vec<_> = d.pieces[0]
        .profile
        .iter()
        .zip(&radii)
        .map(|(p, r)| [p[0] * r * 5., p[1] * r * 5.])
        .collect();
    let area = (0..n)
        .map(|j| {
            let a = outline[j];
            let b = outline[(j + 1) % n];
            a[0] * b[1] - a[1] * b[0]
        })
        .sum::<f64>()
        / 2.;
    assert!((c.parts[0].signed_volume_cubic_meters - area * 8.).abs() < 1e-4);
    let a = std::f64::consts::PI / 6.;
    assert!(
        c.intersections(&ray([4. * a.cos(), 20., -4. * a.sin()]))
            .unwrap()
            .is_empty()
    );
    assert_eq!(c.intersections(&ray([4., 20., 0.])).unwrap().len(), 2);
}

#[test]
fn same_authorship_has_precise_radial_placement_at_earth_and_large_radii() {
    let original = source();
    let plane = closed(&original);
    let mut explicit = original.clone();
    explicit.domain = Some(Domain::Plane);
    assert_eq!(plane.geometry, closed(&explicit).geometry);
    for radius in [1000., 6_371_000., 5e11] {
        let center = Position::normalize(Coordinates {
            cell: [9_007_199_254_740_993, -7, i64::MIN + 1000],
            offset: [0.125, 0.25, 0.5],
        })
        .unwrap();
        let mut d = original.clone();
        d.origin = center.translated([0., radius, 0.]).unwrap();
        d.domain = Some(Domain::Sphere {
            center,
            radius_meters: radius,
            forward: [0., 0., 1.],
        });
        let c = closed(&d);
        assert!(c.maximum_vertex_rounding_meters < 1e-5);
        let error = c.maximum_sampled_domain_error_meters.unwrap();
        assert!(error < 0.1);
        let h = c.intersections(&ray([0., 20., 0.])).unwrap();
        assert!((h[0].altitude_meters - 8.).abs() < error + 1e-5);
        assert!(h[0].slope_degrees < 1.);
        // Moving both origin and centre by gigantic cells preserves local geometry.
        let mut shifted = d.clone();
        shifted.origin = origin().translated([0., radius, 0.]).unwrap();
        shifted.domain = Some(Domain::Sphere {
            center: origin(),
            radius_meters: radius,
            forward: [0., 0., 1.],
        });
        assert_eq!(c.geometry, closed(&shifted).geometry);
        // The same chart at the equator rotates both radial up and tangent right.
        shifted.origin = origin().translated([radius, 0., 0.]).unwrap();
        let equator = closed(&shifted);
        assert_eq!(equator.geometry.triangles, c.geometry.triangles);
        for (north, equator) in c.geometry.positions.iter().zip(&equator.geometry.positions) {
            assert_eq!(*equator, [north[1], -north[0], north[2]]);
        }
    }
}

#[test]
fn legacy_bytes_versions_revision_and_invalid_domains_are_not_silently_reinterpreted() {
    let bytes = include_bytes!("../examples/landforms.ignis-landform");
    let old = landforms::decode(bytes).unwrap();
    assert_eq!(old.version, 1);
    assert!(old.domain.is_none());
    assert_eq!(landforms::encode(&old).unwrap(), bytes);
    let d = source();
    let bytes = landforms::encode(&d).unwrap();
    assert_eq!(landforms::decode(&bytes).unwrap(), d);
    let mut forged = bytes;
    forged[8..10].copy_from_slice(&1u16.to_le_bytes());
    assert!(landforms::decode(&forged).is_err());
    let mut invalid = d.clone();
    invalid.version = 1;
    assert!(invalid.validate().is_err());
    invalid = d.clone();
    invalid.domain = Some(Domain::Sphere {
        center: origin(),
        radius_meters: 1000.,
        forward: [0., 1., 0.],
    });
    assert!(invalid.validate().is_err());
    invalid.origin = origin().translated([0., 1000., 0.]).unwrap();
    assert!(invalid.validate().is_err());
    invalid.domain = Some(Domain::Sphere {
        center: origin(),
        radius_meters: 1000.,
        forward: [0., 0., 1.],
    });
    invalid.pieces[0].anchor[0] = 2000.;
    assert!(
        landforms::compile(&invalid).is_err(),
        "Do not fold a chart around the planet"
    );
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["pieces"][0]["sections"][0]["radial_scales"] = serde_json::json!(vec![1.; 97]);
    assert!(serde_json::from_value::<Document>(raw).is_err());
    let upgraded = landforms::edit(
        old.clone(),
        old.revision,
        vec![Edit::SetDomain {
            domain: Some(Domain::Plane),
        }],
    )
    .unwrap();
    assert_eq!(upgraded.version, 2);
    assert_eq!(upgraded.revision, old.revision + 1);
    assert!(
        landforms::edit(
            old.clone(),
            old.revision + 1,
            vec![Edit::SetDomain { domain: None }]
        )
        .is_err()
    );
}
