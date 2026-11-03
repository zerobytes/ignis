use glam::Vec3;
use ignis::{
    engine::{Command, Engine},
    model::{DirectionalShadow, RoomRecipe},
    render::shadow_view_projection,
};

fn settings() -> DirectionalShadow {
    DirectionalShadow {
        center: [0., 2., 0.],
        extent: 6.,
        resolution: 1024,
        bias: 0.0005,
    }
}
fn engine() -> Engine {
    Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn light_volume_is_finite_for_vertical_lights_and_maps_center_and_depth_direction() {
    let mut light = engine().scene.light;
    assert!(shadow_view_projection(&light).is_none());
    light.shadow = Some(settings());
    for direction in [
        [0., -1., 0.],
        [0., 1., 0.],
        [0.6, -1., 0.2],
        [0.01, -1., 0.01],
    ] {
        light.direction = direction;
        let matrix = shadow_view_projection(&light).unwrap();
        assert!(matrix.to_cols_array().iter().all(|x| x.is_finite()));
        let center = Vec3::from(settings().center);
        let p = matrix.project_point3(center);
        assert!(p.x.abs() < 0.00001 && p.y.abs() < 0.00001 && (p.z - 0.5).abs() < 0.001);
        let next = matrix.project_point3(center + Vec3::from(direction).normalize());
        assert!(next.z > p.z);
        assert!(next.x.abs() < 0.00001 && next.y.abs() < 0.00001);
    }
}

#[test]
fn explicit_extent_has_a_camera_independent_boundary() {
    let mut e = engine();
    e.scene.light.direction = [0., -1., 0.];
    e.scene.light.shadow = Some(settings());
    let vp = shadow_view_projection(&e.scene.light).unwrap();
    let center = Vec3::from(settings().center);
    assert!((vp.project_point3(center + Vec3::X * 6.).x.abs() - 1.).abs() < 0.00001);
    assert!(vp.project_point3(center + Vec3::X * 6.1).x.abs() > 1.);
    e.scene.camera.position = [30., 40., 50.];
    assert_eq!(shadow_view_projection(&e.scene.light).unwrap(), vp);
}

#[test]
fn shadow_limits_and_nonfinite_values_fail_atomically() {
    let mut e = engine();
    for kind in 0..8 {
        let mut light = e.scene.light.clone();
        let mut s = settings();
        match kind {
            0 => s.extent = 0.,
            1 => s.extent = 257.,
            2 => s.extent = f32::NAN,
            3 => s.resolution = 513,
            4 => s.resolution = 4096,
            5 => s.bias = -0.01,
            6 => s.bias = f32::INFINITY,
            _ => s.center[0] = f32::NAN,
        }
        light.shadow = Some(s);
        let before = e.snapshot();
        assert!(e.execute(Command::SetLight { light }.into()).is_err());
        assert_eq!(e.snapshot(), before);
    }
}

#[test]
fn opt_in_migrates_legacy_light_and_rejects_forged_old_scene() {
    let mut e = engine();
    let old = serde_json::json!({"direction":[0,-1,0],"ambient":0.2});
    let light = serde_json::from_value(old).unwrap();
    e.execute(Command::SetLight { light }.into()).unwrap();
    assert!(e.scene.light.shadow.is_none());
    let mut light = e.scene.light.clone();
    light.shadow = Some(settings());
    let original = e.scene.clone();
    e.execute(Command::SetLight { light }.into()).unwrap();
    e.scene.version = 52;
    assert!(
        e.scene
            .validate()
            .unwrap_err()
            .message
            .contains("version53")
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
}
