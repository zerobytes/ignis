use ignis::{
    landmark_projection::{Landmark, Status, inspect},
    model::Camera,
};
fn camera() -> Camera {
    Camera {
        position: [0., 0., 5.],
        target: [0., 0., 0.],
        fov_degrees: 90.,
    }
}
fn point(id: &str, world: [f32; 3], reference_pixel: Option<[f32; 2]>) -> Landmark {
    Landmark {
        id: id.into(),
        world,
        reference_pixel,
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 0.002, "{a} != {b}");
}
#[test]
fn perspective_pixels_use_top_left_and_reciprocate_picking_plane() {
    let camera = camera();
    let points = [
        point("center", [0., 0., 0.], None),
        point("upper-right", [1., 1., 0.], None),
        point("farther", [1., 1., -5.], None),
    ];
    let report = inspect(&camera, 800, 600, &points).unwrap();
    for (sample, expected) in report
        .samples
        .iter()
        .zip([[400., 300.], [460., 240.], [430., 270.]])
    {
        let pixel = sample.pixel.unwrap();
        close(pixel[0] as f64, expected[0]);
        close(pixel[1] as f64, expected[1]);
        // Camera ray intersects this world point's horizontal plane, with a tilted camera.
        assert_eq!(sample.status, Status::InViewport);
    }
    let tilted = Camera {
        position: [3., 4., 8.],
        target: [0., 0., 0.],
        ..camera
    };
    let points = [
        point("ground", [1., 0., 1.], None),
        point("upper", [-1., 2., 0.], None),
    ];
    let report = inspect(&tilted, 800, 600, &points).unwrap();
    for sample in report.samples {
        let p = sample.pixel.unwrap();
        let world =
            ignis::picking::point_on_plane(&tilted, p[0], p[1], 800, 600, sample.world[1]).unwrap();
        for (a, b) in world.into_iter().zip(sample.world) {
            close(a as f64, b as f64);
        }
    }
}
#[test]
fn clipping_and_offscreen_residuals_do_not_hide_errors() {
    let camera = camera();
    let points = [
        point("zero", [0., 0., 0.], Some([400., 300.])),
        point("offset", [0., 0., 0.], Some([397., 296.])),
        point("outside", [100., 0., 0.], Some([400., 300.])),
        point("behind", [0., 0., 6.], Some([1., 1.])),
        point("near", [0., 0., 4.99], None),
        point("eye", [0., 0., 5.], None),
    ];
    let r = inspect(&camera, 800, 600, &points).unwrap();
    assert_eq!(
        (
            r.requested,
            r.projectable,
            r.in_viewport,
            r.correspondences,
            r.compared,
            r.excluded
        ),
        (6, 3, 2, 4, 3, 1)
    );
    assert_eq!(r.samples[2].status, Status::OutsideViewport);
    assert_eq!(r.samples[3].status, Status::BehindCamera);
    assert_eq!(r.samples[4].status, Status::BeforeNear);
    assert_eq!(r.samples[5].status, Status::BehindCamera);
    close(r.samples[1].distance_pixels.unwrap(), 5.);
    close(r.maximum_pixels.unwrap(), 6000.);
    close(r.rmse_pixels.unwrap(), ((36_000_000_f64 + 25.) / 3.).sqrt());
    let far_camera = Camera {
        position: [10000.; 3],
        target: [-10000.; 3],
        ..camera
    };
    let far = [point("far", [-10000.; 3], Some([1., 1.]))];
    let r = inspect(&far_camera, 800, 600, &far).unwrap();
    assert_eq!(r.samples[0].status, Status::AfterFar);
    assert_eq!(
        (r.rmse_pixels, r.maximum_pixels, r.compared, r.excluded),
        (None, None, 0, 1)
    );
}
#[test]
fn bounded_input_rejects_invalid_points_ids_camera_and_reference() {
    let camera = camera();
    let valid = point("a", [0.; 3], None);
    assert!(inspect(&camera, 800, 600, &[]).is_err());
    assert!(inspect(&camera, 800, 600, &vec![valid.clone(); 129]).is_err());
    assert!(inspect(&camera, 800, 600, &[valid.clone(), valid.clone()]).is_err());
    for id in ["", "space here", "ç", &"x".repeat(65)] {
        assert!(inspect(&camera, 800, 600, &[point(id, [0.; 3], None)]).is_err());
    }
    for world in [
        [f32::NAN, 0., 0.],
        [f32::INFINITY, 0., 0.],
        [10000.1, 0., 0.],
    ] {
        assert!(inspect(&camera, 800, 600, &[point("a", world, None)]).is_err());
    }
    for pixel in [
        [800., 0.],
        [0., 600.],
        [-1., 0.],
        [f32::NAN, 0.],
        [0., f32::INFINITY],
    ] {
        assert!(inspect(&camera, 800, 600, &[point("a", [0.; 3], Some(pixel))]).is_err());
    }
    for (w, h) in [(0, 600), (800, 0), (16385, 1)] {
        assert!(inspect(&camera, w, h, std::slice::from_ref(&valid)).is_err());
    }
    assert!(
        inspect(
            &Camera {
                target: camera.position,
                ..camera.clone()
            },
            800,
            600,
            std::slice::from_ref(&valid)
        )
        .is_err()
    );
    assert!(
        inspect(
            &Camera {
                fov_degrees: 0.,
                ..camera.clone()
            },
            800,
            600,
            std::slice::from_ref(&valid)
        )
        .is_err()
    );
    let maximum: Vec<_> = (0..128)
        .map(|i| point(&format!("a-{i}"), [0.; 3], None))
        .collect();
    assert_eq!(
        inspect(&camera, 16384, 16384, &maximum).unwrap().requested,
        128
    );
}
