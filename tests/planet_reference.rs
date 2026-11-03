//! Critical integration: global addresses cannot leak into GPU floats; source/view errors
//! must be rejected before allocating or writing a misleading sphere image.
use glam::DVec3;
use ignis::{
    planet::{Document, Pose, Relief, view::View},
    world_space::{Coordinates, Position},
};
fn doc() -> Document {
    Document {
        version: 1,
        space_version: 1,
        id: "earth".into(),
        revision: 1,
        center: Position::normalize(Coordinates {
            cell: [0; 3],
            offset: [0.; 3],
        })
        .unwrap(),
        diameter_meters: 12_742_000.,
        surface_gravity_mps2: 9.81,
        base_height_meters: 0.,
        operations: vec![],
    }
}
fn view(h: f64) -> View {
    View {
        pose: Pose {
            direction: [0., 1., 0.],
            forward: [0., 0., -1.],
            altitude_meters: h,
        },
        pitch_degrees: 0.,
        fov_degrees: 60.,
        diagnostic_mask: false,
    }
}
#[test]
fn global_translation_preserves_optical_frame_and_terrestrial_horizon() {
    let local = doc();
    let mut remote = doc();
    remote.center = Position::normalize(Coordinates {
        cell: [
            9_007_199_254_740_993,
            -9_007_199_254_740_993,
            9_007_199_254_740_993,
        ],
        offset: [0.125, 0.5, 0.25],
    })
    .unwrap();
    for altitude in [0.01, 2., 100., 10000., 400000., 10000000.] {
        let v = view(altitude);
        let a = v.prepare(&local, 1280, 800).unwrap();
        let b = v.prepare(&remote, 1280, 800).unwrap();
        assert_eq!(a.forward, b.forward);
        assert_eq!(a.camera_up, b.camera_up);
        assert_eq!(a.right, b.right);
        let address: Coordinates = b.camera_position.into();
        assert_eq!(address.cell[0], 9_007_199_254_740_993);
        assert_eq!(address.offset[0], 0.125);
        let down = -a.radial_up;
        assert!((a.hit_distance(down).unwrap() - altitude).abs() < 1e-7);
        assert!(a.hit_distance(a.forward).is_none());
        let dip = ((altitude * (2. * a.radius_meters + altitude)).sqrt()).atan2(a.radius_meters);
        for (angle, hit) in [(dip - 1e-7, false), (dip + 1e-7, true)] {
            let ray = DVec3::new(0., -angle.sin(), -angle.cos());
            assert_eq!(a.hit_distance(ray).is_some(), hit);
        }
    }
}
#[test]
fn invalid_source_view_and_dimensions_are_rejected_without_reinterpreting_relief() {
    let far = view(1e18).prepare(&doc(), 1280, 800).unwrap();
    assert!(far.hit_distance(-far.radial_up).is_some());
    let mut source = doc();
    let v = view(2.);
    for (w, h) in [(0, 800), (2049, 800), (1280, 63)] {
        assert!(v.prepare(&source, w, h).is_err());
    }
    let mut bad = v.clone();
    bad.pose.altitude_meters = 0.;
    assert!(bad.prepare(&source, 1280, 800).is_err());
    bad = v.clone();
    bad.pose.forward = [0., 1., 0.];
    assert!(bad.prepare(&source, 1280, 800).is_err());
    bad = v.clone();
    bad.fov_degrees = f64::NAN;
    assert!(bad.prepare(&source, 1280, 800).is_err());
    source.operations.push(Relief::Noise {
        seed: 1,
        wavelength_meters: 100000.,
        amplitude_meters: 100.,
        octaves: 3,
        persistence: 0.5,
    });
    assert!(v.prepare(&source, 1280, 800).is_err());
    source.operations.clear();
    source.base_height_meters = 1.;
    assert!(v.prepare(&source, 1280, 800).is_err());
    // Looking vertically down is valid; the explicit tangent frame avoids world-Y singularity.
    let mut down = v;
    down.pitch_degrees = -90.;
    assert!(down.prepare(&doc(), 1280, 800).is_ok());
}
