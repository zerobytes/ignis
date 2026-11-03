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
        receiver_plane_bias: false,
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

#[test]
fn receiver_plane_bias_is_explicit_versioned_and_round_trips() {
    let mut e = engine();
    let mut light = e.scene.light.clone();
    light.shadow = Some(settings());
    let legacy = serde_json::to_value(&light).unwrap();
    assert!(legacy["shadow"].get("receiver_plane_bias").is_none());
    assert!(
        !serde_json::from_value::<ignis::model::Light>(legacy)
            .unwrap()
            .shadow
            .unwrap()
            .receiver_plane_bias
    );
    light.shadow.as_mut().unwrap().receiver_plane_bias = true;
    e.scene.version = 110;
    e.execute(Command::SetLight { light }.into()).unwrap();
    assert_eq!(e.scene.version, 111);
    let restored: ignis::model::Scene =
        serde_json::from_slice(&serde_json::to_vec(&e.scene).unwrap()).unwrap();
    restored.validate().unwrap();
    assert!(restored.light.shadow.unwrap().receiver_plane_bias);
    let content = ignis::network::content::Content::capture(&e, 208);
    let decoded =
        ignis::network::content::Content::decode(208, &content.encode().unwrap()).unwrap();
    assert!(decoded.light.shadow.unwrap().receiver_plane_bias);
    e.scene.version = 110;
    assert!(
        e.scene
            .validate()
            .unwrap_err()
            .message
            .contains("version111")
    );
    e.execute(Command::Undo.into()).unwrap();
    assert!(e.scene.light.shadow.is_none());
}

#[test]
#[ignore = "requires native GPU"]
fn receiver_plane_gpu_removes_planar_acne_and_keeps_cast_shadows() {
    use ignis::{image_similarity::Image, render::Renderer};
    let mut scene = engine().scene;
    scene.entities.clear();
    scene.meshes = serde_json::from_value(serde_json::json!({
        "assets":[{"id":"floor","geometry":{"positions":[[-1,0,-1],[1,0,-1],[-1,0,1],[1,0,1]],"triangles":[[0,2,1],[1,2,3]]}}],
        "instances":[{"id":"floor","asset":"floor","position":[0,0,0],"rotation":[0,0,0,1],"scale":1,"color":[0.5,0.5,0.5]}]
    })).unwrap();
    scene.camera = ignis::model::Camera {
        position: [1.6, 1.8, 2.5],
        target: [0., 0., 0.],
        fov_degrees: 48.,
    };
    scene.light.ambient = 0.05;
    scene.light.environment = None;
    let mut frame = Engine::new(scene).unwrap().render_frame();
    frame.suppress_diagnostic_grid = true;
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let error = |a: &[u8], b: &[u8]| {
        let a = Image::decode(a).unwrap();
        let b = Image::decode(b).unwrap();
        a.pixels
            .iter()
            .zip(&b.pixels)
            .filter(|(a, b)| a.iter().zip(b.iter()).any(|(a, b)| a.abs_diff(*b) > 2))
            .count()
    };
    for direction in [[0.6, -1., -0.8], [0.7, -0.12, 0.2], [0., -1., 0.]] {
        frame.light.direction = direction;
        frame.light.shadow = None;
        let lit = renderer.capture(&frame, 384, 384).unwrap();
        frame.light.shadow = Some(DirectionalShadow {
            center: [0., 0., 0.],
            extent: 1.6,
            resolution: 512,
            bias: 0.000001,
            receiver_plane_bias: false,
        });
        let old = renderer.capture(&frame, 384, 384).unwrap();
        frame.light.shadow.as_mut().unwrap().receiver_plane_bias = true;
        let corrected = renderer.capture(&frame, 384, 384).unwrap();
        let before = error(&lit, &old);
        let after = error(&lit, &corrected);
        println!("direction{direction:?}: self-shadow pixels {before}->{after}");
        assert!(
            after <= before / 10 + 8,
            "receiver bias did not remove acne"
        );
        assert_eq!(corrected, renderer.capture(&frame, 384, 384).unwrap());
    }
    // A separate elevated plate must still cast a shadow; correcting the
    // receiver must not merely turn all comparison samples into light.
    let mut scene = engine().scene;
    scene.entities.clear();
    scene.meshes=serde_json::from_value(serde_json::json!({
        "assets":[{"id":"plate","geometry":{"positions":[[-1,0,-1],[1,0,-1],[-1,0,1],[1,0,1]],"triangles":[[0,2,1],[1,2,3]]}}],
        "instances":[{"id":"floor","asset":"plate","position":[0,0,0],"rotation":[0,0,0,1],"scale":1,"color":[0.5,0.5,0.5]},
                     {"id":"caster","asset":"plate","position":[0,0.3,0],"rotation":[0,0,0,1],"scale":0.25,"color":[0.5,0.5,0.5]}]
    })).unwrap();
    scene.camera = ignis::model::Camera {
        position: [1.6, 1.8, 2.5],
        target: [0., 0., 0.],
        fov_degrees: 48.,
    };
    scene.light.direction = [0.6, -1., -0.8];
    scene.light.ambient = 0.05;
    scene.light.environment = None;
    let mut frame = Engine::new(scene).unwrap().render_frame();
    frame.suppress_diagnostic_grid = true;
    let lit = renderer.capture(&frame, 384, 384).unwrap();
    frame.light.shadow = Some(DirectionalShadow {
        center: [0., 0., 0.],
        extent: 1.6,
        resolution: 512,
        bias: 0.000001,
        receiver_plane_bias: true,
    });
    let shadowed = renderer.capture(&frame, 384, 384).unwrap();
    assert!(error(&lit, &shadowed) > 250, "cast shadow disappeared");
}
