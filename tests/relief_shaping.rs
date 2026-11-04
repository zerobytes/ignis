use ignis::{
    landforms::{self, CreasePlane, Document, Domain, Edit, Falloff, Ray, Shaping, Stamp},
    world_space::{Coordinates, Position},
};

fn source() -> Document {
    serde_json::from_value(
        serde_json::json!({"version":2,"space_version":1,"id":"plateau","revision":1,
        "origin":{"cell":["0","0","0"],"offset":[0,0,0]},"pieces":[{
            "id":"wall","anchor":[0,0,0],"axis":[0,1,0],"profile_x":[1,0,0],
            "profile":[[-30,-15],[30,-15],[30,15],[-30,15]],"inner_scale":null,
            "sections":[{"at":0,"offset":[0,0],"scale":1},{"at":20,"offset":[0,0],"scale":1}]}]}),
    )
    .unwrap()
}
fn stamp() -> Stamp {
    Stamp {
        center: [-7.5, 10., -15.],
        axis: [0., 0., -1.],
        profile_x: [1., 0., 0.],
        half_extents: [10., 7., 3.],
        displacement: [1., 1., -5.],
        flat_core: 0.25,
        falloff: Falloff::Linear,
    }
}
fn shaped() -> Document {
    landforms::edit(
        source(),
        1,
        vec![Edit::SetShaping {
            id: "wall".into(),
            shaping: Some(Shaping {
                subdivisions: 3,
                creases: vec![],
                stamps: vec![stamp()],
            }),
        }],
    )
    .unwrap()
}

#[test]
fn localized_3d_relief_changes_the_connected_surface_preserves_the_top_and_roundtrips() {
    let d = shaped();
    assert_eq!(d.version, 3);
    let bytes = landforms::encode(&d).unwrap();
    assert_eq!(landforms::decode(&bytes).unwrap(), d);
    let c = landforms::compile(&d).unwrap();
    let topology = ignis::mesh_topology::Topology::inspect(&c.geometry).unwrap();
    assert_eq!(topology.boundary_edges, 0);
    assert_eq!(topology.non_manifold_edges, 0);
    assert_eq!(c.parts.len(), 1);
    assert_eq!(c.geometry.triangles.len(), 12 * 64);
    assert_eq!(c.geometry.positions.len(), 386);
    assert!(c.geometry.positions.iter().any(|p| p[2] < -19.));
    assert!(c.geometry.positions.iter().all(|p| p[1] <= 20.));
    assert!(c.geometry.positions.iter().filter(|p| p[1] == 20.).count() > 20);
    assert!(
        c.geometry
            .positions
            .iter()
            .any(|p| p[1] > 10. && p[1] < 11.1 && p[2] < -18.)
    );
    let ray = Ray {
        origin: [-7.5, 10., -40.],
        direction: [0., 0., 1.],
        max_distance_meters: 100.,
    };
    let old = landforms::compile(&source()).unwrap();
    let old_hit = old.intersections(&ray).unwrap()[0].distance_meters;
    let new_hit = c.intersections(&ray).unwrap()[0].distance_meters;
    assert!(new_hit < old_hit - 3.);
    // The same shaped body at the pole/equator is a rotated radial placement.
    let zero = Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [0.; 3],
    })
    .unwrap();
    let mut north = d.clone();
    north.origin = zero.translated([0., 6_371_000., 0.]).unwrap();
    north.domain = Some(Domain::Sphere {
        center: zero,
        radius_meters: 6_371_000.,
        forward: [0., 0., 1.],
    });
    let n = landforms::compile(&north).unwrap();
    north.origin = zero.translated([6_371_000., 0., 0.]).unwrap();
    let e = landforms::compile(&north).unwrap();
    assert_eq!(n.geometry.triangles, e.geometry.triangles);
    for (a, b) in n.geometry.positions.iter().zip(&e.geometry.positions) {
        assert_eq!(*b, [a[1], -a[0], a[2]]);
    }
}

#[test]
fn limits_invalid_frames_folds_and_false_legacy_envelopes_are_rejected() {
    let mut d = shaped();
    d.version = 2;
    assert!(d.validate().is_err());
    d = shaped();
    d.pieces[0].shaping.as_mut().unwrap().stamps[0].axis = [1., 0., 0.];
    assert!(d.validate().is_err());
    d = shaped();
    d.pieces[0].shaping.as_mut().unwrap().stamps[0].half_extents[1] = 0.;
    assert!(d.validate().is_err());
    d = shaped();
    d.pieces[0].shaping.as_mut().unwrap().stamps[0].displacement = [0., -100., 0.];
    assert!(
        landforms::compile(&d).is_err(),
        "Do not publish a folded wall"
    );
    d = shaped();
    let mut middle = d.pieces[0].sections[0].clone();
    middle.at = 10.;
    d.pieces[0].sections.insert(1, middle);
    d.pieces = std::iter::repeat_with(|| d.pieces[0].clone())
        .take(32)
        .enumerate()
        .map(|(i, mut p)| {
            p.id = format!("p{i}");
            p.shaping.as_mut().unwrap().stamps = vec![stamp(); 32];
            p
        })
        .collect();
    assert!(
        d.validate().is_err(),
        "Charge refinement and stamp work before generating"
    );
    let mut raw = serde_json::to_value(shaped()).unwrap();
    raw["pieces"][0]["shaping"]["stamps"] = serde_json::json!(vec![stamp(); 33]);
    assert!(serde_json::from_value::<Document>(raw).is_err());
    let old = include_bytes!("../examples/landforms.ignis-landform");
    assert_eq!(
        landforms::encode(&landforms::decode(old).unwrap()).unwrap(),
        old
    );
}

#[test]
fn crease_tessellation_preserves_the_solid_and_aligns_actual_rock_facets() {
    use glam::DVec3;
    let center = [-7.5, 10., -15.];
    let mut planes = vec![
        CreasePlane {
            point: center,
            normal: [0.1, 1. / 7., 0.],
        },
        CreasePlane {
            point: center,
            normal: [0.1, -1. / 7., 0.],
        },
    ];
    for (point, normal) in [
        ([-17.5, 0., 0.], [1., 0., 0.]),
        ([2.5, 0., 0.], [1., 0., 0.]),
        ([0., 3., 0.], [0., 1., 0.]),
        ([0., 17., 0.], [0., 1., 0.]),
    ] {
        planes.push(CreasePlane { point, normal });
    }
    let mut d = source();
    d.version = 3;
    d.pieces[0].shaping = Some(Shaping {
        subdivisions: 0,
        creases: planes,
        stamps: vec![],
    });
    let cut = landforms::compile(&d).unwrap();
    let topology = ignis::mesh_topology::Topology::inspect(&cut.geometry).unwrap();
    assert_eq!(topology.boundary_edges, 0);
    assert_eq!(topology.non_manifold_edges, 0);
    assert!(topology.isolated_vertices.is_empty());
    assert!((cut.parts[0].signed_volume_cubic_meters - 36000.).abs() < 0.02);
    let mut s = stamp();
    s.displacement = [0., 0., -5.];
    s.flat_core = 0.;
    s.half_extents = [10., 7., 50.];
    d.pieces[0].shaping.as_mut().unwrap().stamps = vec![s];
    let c = landforms::compile(&d).unwrap();
    let expected = [
        [0., 0., -1.],
        [0.5, 0., -1.],
        [-0.5, 0., -1.],
        [0., 5. / 7., -1.],
        [0., -5. / 7., -1.],
    ]
    .map(|p| DVec3::from_array(p).normalize());
    let mut seen = [false; 5];
    for face in &c.geometry.triangles {
        let [a, b, c] =
            face.map(|i| DVec3::from_array(c.geometry.positions[i as usize].map(f64::from)));
        if [a, b, c].iter().all(|p| p.z <= -14.99) {
            let n = (b - a).cross(c - a).normalize();
            let (i, error) = expected
                .iter()
                .enumerate()
                .map(|(i, e)| (i, n.distance(*e)))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            assert!(
                error < 1e-4,
                "Face does not follow its authored plane: {n:?}"
            );
            seen[i] = true;
        }
    }
    assert!(seen.into_iter().all(|v| v));
}
