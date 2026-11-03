//! Critical risks: source corruption, globe seams/precision, and atomic mesh publication.
use glam::DVec3;
use ignis::{
    engine::{Command, Engine},
    model::RoomRecipe,
    planet::{
        self, Document, Pose, Relief,
        mesh::{self, Area, Face, Projection},
    },
    world_space::{Coordinates, Position},
};
use std::collections::BTreeMap;
fn doc() -> Document {
    Document {
        version: 1,
        space_version: 1,
        id: "planet".into(),
        revision: 1,
        center: Position::normalize(Coordinates {
            cell: [0; 3],
            offset: [0.; 3],
        })
        .unwrap(),
        diameter_meters: 800.,
        surface_gravity_mps2: 9.81,
        base_height_meters: 0.,
        operations: vec![],
    }
}
fn pose() -> Pose {
    Pose {
        direction: [0., 1., 0.],
        forward: [0., 0., -1.],
        altitude_meters: 0.,
    }
}
#[test]
fn source_roundtrip_corruption_bounds_and_geometric_circumnavigation() {
    let mut d = doc();
    d.center = Position::normalize(Coordinates {
        cell: [9_007_199_254_740_993, -900, 10],
        offset: [0.125, 0.5, 0.25],
    })
    .unwrap();
    d.operations.push(Relief::Noise {
        seed: 42,
        wavelength_meters: 300.,
        amplitude_meters: 12.,
        octaves: 4,
        persistence: 0.5,
    });
    d.operations.push(Relief::Raise {
        direction: [1., 1., 1.],
        angular_radius_degrees: 25.,
        height_meters: 20.,
    });
    let sampled = planet::Sampler::new(&d)
        .unwrap()
        .sample([0., 1., 0.])
        .unwrap();
    let address: Coordinates = sampled.position.into();
    assert_eq!(address.cell[0], 9_007_199_254_740_993);
    assert_eq!(address.offset[0], 0.125);
    let bytes = planet::encode(&d).unwrap();
    assert_eq!(planet::decode(&bytes).unwrap(), d);
    assert_eq!(
        planet::encode(&planet::decode(&bytes).unwrap()).unwrap(),
        bytes
    );
    for i in [0, 8, 10, 18, bytes.len() - 1] {
        let mut b = bytes.clone();
        b[i] ^= 1;
        assert!(planet::decode(&b).is_err());
    }
    let mut b = bytes.clone();
    b.push(0);
    assert!(planet::decode(&b).is_err());
    for bad in [0., -1., f64::NAN, f64::INFINITY, 1e13] {
        let mut b = d.clone();
        b.diameter_meters = bad;
        assert!(b.validate().is_err());
    }
    let mut bad = d.clone();
    bad.operations = vec![d.operations[0].clone(); 33];
    assert!(bad.validate().is_err());
    // Large center cells never pass through f64. Local sub-meter offset survives sampling.
    let mut p = pose();
    let start = p;
    let circumference = std::f64::consts::PI * d.diameter_meters;
    for _ in 0..1024 {
        p = d.travel(p, circumference / 1024.).unwrap();
        let f = p.frame().unwrap();
        assert!(DVec3::from(f.up).dot(DVec3::from(f.forward)).abs() < 1e-12);
    }
    assert!(DVec3::from(p.direction).distance(DVec3::from(start.direction)) * d.radius() < 1e-8);
    assert!(DVec3::from(p.forward).distance(DVec3::from(start.forward)) < 1e-10);
    let one = d.travel(start, circumference).unwrap();
    assert!(DVec3::from(one.direction).distance(DVec3::from(start.direction)) < 1e-12);
    let backward = d.travel(d.travel(start, 100.).unwrap(), -100.).unwrap();
    assert!(DVec3::from(backward.direction).distance(DVec3::from(start.direction)) < 1e-12);
    assert!(
        d.travel(
            Pose {
                forward: [0., 1., 0.],
                ..start
            },
            1.
        )
        .is_err()
    );
}
#[test]
fn globe_is_closed_outward_and_local_earth_tiles_preserve_edges() {
    let mut d = doc();
    d.operations.push(Relief::Noise {
        seed: 712,
        wavelength_meters: 250.,
        amplitude_meters: 10.,
        octaves: 3,
        persistence: 0.5,
    });
    let c = mesh::compile(&d, &Area::Globe { subdivisions: 16 }, &Projection::Centered).unwrap();
    assert_eq!(c.geometry.positions.len(), 6 * 16 * 16 + 2);
    assert_eq!(c.geometry.triangles.len(), 12 * 16 * 16);
    let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
    for t in &c.geometry.triangles {
        let [a, b, c] = t.map(|i| DVec3::from(c.geometry.positions[i as usize].map(f64::from)));
        assert!((b - a).cross(c - a).dot(a + b + c) > 0.);
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let e = edges.entry((a.min(b), a.max(b))).or_default();
            e.0 += 1;
            e.1 += if a < b { 1 } else { -1 };
        }
    }
    assert!(edges.values().all(|v| *v == (2, 0)));
    d.operations.clear();
    d.diameter_meters = 12_742_000.;
    assert!(mesh::compile(&d, &Area::Globe { subdivisions: 16 }, &Projection::Centered).is_err());
    let projection = Projection::Surface { pose: pose() };
    let left = mesh::compile(
        &d,
        &Area::Tile {
            face: Face::PositiveY,
            level: 12,
            tile: [2047, 2048],
            subdivisions: 16,
        },
        &projection,
    )
    .unwrap();
    let right = mesh::compile(
        &d,
        &Area::Tile {
            face: Face::PositiveY,
            level: 12,
            tile: [2048, 2048],
            subdivisions: 16,
        },
        &projection,
    )
    .unwrap();
    for y in 0..17 {
        assert_eq!(
            left.geometry.positions[y * 17 + 16],
            right.geometry.positions[y * 17]
        );
    }
    // Tangent frame at the north pole: sphere bends below tangent plane, never a flat patch.
    assert!(left.geometry.positions.iter().any(|p| p[1] < -0.1));
    assert!(
        left.report["maximum_projection_error_meters"]
            .as_f64()
            .unwrap()
            < 0.001
    );
    let horizon = planet::query(planet::Query::Horizon {
        document: d,
        altitude_meters: 2.,
    })
    .unwrap();
    assert!((horizon["line_of_sight_meters"].as_f64().unwrap() - 5048.168).abs() < 1.);
}
#[test]
fn failed_publication_is_atomic_and_undo_replay_preserve_native_geometry() {
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    let original = e.scene.clone();
    let revision = e.revision;
    let mut p = mesh::Publication {
        document: doc(),
        area: Area::Globe { subdivisions: 8 },
        frame: Projection::Centered,
        asset: "planet_surface".into(),
        instance: "".into(),
        color: [0.3, 0.6, 0.2],
        material: Default::default(),
        collision: Some(true),
    };
    assert!(
        e.execute(
            Command::PublishPlanet {
                publication: Box::new(p.clone())
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.scene, original);
    assert_eq!(e.revision, revision);
    p.instance = "planet_surface".into();
    e.execute(
        Command::PublishPlanet {
            publication: Box::new(p),
        }
        .into(),
    )
    .unwrap();
    let published = e.scene.clone();
    assert!(
        published
            .collision_meshes
            .iter()
            .any(|s| s == "planet_surface")
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, published);
    let bytes = serde_json::to_vec(&e.scene).unwrap();
    assert_eq!(
        serde_json::from_slice::<ignis::model::Scene>(&bytes).unwrap(),
        published
    );
}
