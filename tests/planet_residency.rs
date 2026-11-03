use glam::DVec3;
use ignis::planet::{
    self, Document, Pose,
    mesh::Face,
    residency::{self, Anchor, FACES, Request, Tile},
};
use std::collections::BTreeSet;

fn document() -> Document {
    serde_json::from_value(serde_json::json!({"version":1,"space_version":1,"id":"earth","revision":1,
        "center":{"cell":["9007199254740993","-9007199254740993","0"],"offset":[0.125,0.5,0.25]},
        "diameter_meters":12742000.,"surface_gravity_mps2":9.81,"base_height_meters":0.,"operations":[]})).unwrap()
}
fn pose(direction: DVec3) -> Pose {
    let forward = if direction.dot(DVec3::Z).abs() < 0.9 {
        DVec3::Z
    } else {
        DVec3::X
    };
    Pose {
        direction: direction.to_array(),
        forward: forward.to_array(),
        altitude_meters: 2.,
    }
}

#[test]
fn residency_has_no_face_or_polar_boundary_and_keeps_physics_subset() {
    let d = document();
    let mut seen = BTreeSet::new();
    // Face centers, twelve edges and eight corners, including both poles.
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let n = DVec3::new(f64::from(x), f64::from(y), f64::from(z));
                if n == DVec3::ZERO {
                    continue;
                }
                let n = n.normalize();
                let q = Request {
                    pose: pose(n),
                    level: 13,
                    visual_radius_meters: 4000.,
                    physical_radius_meters: 300.,
                };
                let p = residency::plan(&d, &q).unwrap();
                assert!(!p.resources_installed && !p.physical.is_empty());
                assert!(p.physical.iter().all(|t| p.visual.contains(t)));
                // Every adjacent face owning this boundary must be resident.
                for face in FACES {
                    let axis = (Tile {
                        face,
                        level: 0,
                        xy: [0, 0],
                    })
                    .direction([0.5, 0.5]);
                    if axis.dot(n) + 1e-12
                        >= [n.x.abs(), n.y.abs(), n.z.abs()]
                            .into_iter()
                            .fold(0., f64::max)
                    {
                        assert!(p.physical.iter().any(|t| t.face == face), "{n:?} {face:?}");
                    }
                }
                seen.extend(p.physical.iter().map(|t| t.face));
                let same = residency::plan(&d, &q).unwrap();
                assert_eq!(p.visual, same.visual);
            }
        }
    }
    assert_eq!(seen.len(), 6);
}

#[test]
fn transported_frame_preserves_points_velocities_and_submeter_height() {
    let d = document();
    let initial = Anchor::new(&d, pose(DVec3::Y)).unwrap();
    let local = DVec3::new(0.125, 1.65, -0.25);
    let (unit, h) = initial.unproject(local);
    assert!(initial.project(unit, h).distance(local) < 1e-9);
    let mut current = initial;
    let mut p = local;
    let mut velocity = DVec3::new(1., 2., 3.);
    let mut travel = initial.pose;
    for _ in 0..512 {
        travel = d
            .travel(travel, std::f64::consts::TAU * d.radius() / 512.)
            .unwrap();
        let next = Anchor::new(&d, travel).unwrap();
        p = current.point_in(next, p);
        velocity = current.rotation_to(next) * velocity;
        current = next;
    }
    assert!(p.distance(local) < 1e-6, "{p:?}");
    assert!(velocity.distance(DVec3::new(1., 2., 3.)) < 1e-10);
    // A tile's immutable local geometry can use only an instance transform after a rebase.
    let moved = initial
        .transported(&d, DVec3::new(0.001, 1., 0.002).normalize(), 23.)
        .unwrap();
    let arbitrary = DVec3::new(1024., 18., -800.);
    let (n, h) = initial.unproject(arbitrary);
    assert!(
        initial
            .point_in(moved, arbitrary)
            .distance(moved.project(n, h))
            < 1e-8
    );
}

#[test]
fn invalid_or_unbounded_requests_do_not_return_partial_coverage() {
    let d = document();
    let mut q = Request {
        pose: pose(DVec3::Y),
        level: 13,
        visual_radius_meters: 4000.,
        physical_radius_meters: 300.,
    };
    q.visual_radius_meters = 0.;
    assert!(residency::plan(&d, &q).is_err());
    q.visual_radius_meters = f64::NAN;
    assert!(residency::plan(&d, &q).is_err());
    q.visual_radius_meters = d.diameter_meters;
    assert!(residency::plan(&d, &q).is_err());
    assert!(
        Tile {
            face: Face::PositiveY,
            level: 13,
            xy: [8192, 0]
        }
        .validate()
        .is_err()
    );
    q.level = 0;
    let globe = residency::plan(&d, &q).unwrap();
    assert_eq!(globe.visual.len(), 6);
    // Tiny and Jupiter-sized worlds use identical addressing and cone math.
    for diameter in [800., 139820000.] {
        let mut other = d.clone();
        other.diameter_meters = diameter;
        let q = Request {
            pose: pose(DVec3::Y),
            level: 0,
            visual_radius_meters: diameter * 2.,
            physical_radius_meters: diameter * 2.,
        };
        assert_eq!(residency::plan(&other, &q).unwrap().visual.len(), 6);
    }
    // The MCP query reports proposals, never fabricated backend readiness.
    let result = planet::query(planet::Query::PlanResidence {
        document: d,
        request: q,
    })
    .unwrap();
    assert_eq!(result["resources_installed"], false);
}
