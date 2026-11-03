use super::*;
use crate::model::RoomRecipe;

pub(super) fn fixture(distance: f64) -> (super::super::runtime::Config, Atlas) {
    let c = super::tests::config();
    let anchor = Anchor::new(&c.document, c.spawn).unwrap();
    let mut light = RoomRecipe::default().generate().unwrap().light;
    light.atmosphere = Some(crate::atmosphere::Atmosphere {
        density: 0.002,
        sun_strength: 0.,
        ..Default::default()
    });
    let a = Atlas::new(
        &c,
        Settings {
            distance_meters: distance,
            atmosphere_geometry: true,
            max_tiles: 384,
            render_group_size: 1,
            ..Default::default()
        },
        anchor,
        ActorView {
            size: [0.6, 1.8, 0.6],
            color: [1.; 3],
            eye: 1.62,
        },
        &light,
        1.,
        Sampler::new(&c.document).unwrap(),
    )
    .unwrap();
    (c, a)
}
#[test]
fn planet_orbit_atmospheric_derivative_preserves_every_seam_attribute() {
    let (c, mut a) = fixture(1000.);
    let mut reduced = 0;
    for (k, selected) in &a.selected {
        let source = &a.resources[k];
        if Arc::ptr_eq(source, selected) {
            continue;
        }
        reduced += 1;
        assert!(selected.asset.geometry.triangles.len() < source.asset.geometry.triangles.len());
        assert_eq!(selected.asset.material, source.asset.material);
        let indices: BTreeMap<_, _> = source
            .keys
            .iter()
            .enumerate()
            .map(|(i, k)| (*k, i))
            .collect();
        for (i, key) in selected.keys.iter().enumerate() {
            let j = indices[key];
            assert_eq!(
                selected.coordinates.as_ref().unwrap()[i],
                source.coordinates.as_ref().unwrap()[j]
            );
            assert_eq!(selected.normals[i], source.normals[j]);
            assert_eq!(
                selected.asset.geometry.positions[i],
                source.asset.geometry.positions[j]
            );
            assert_eq!(
                selected
                    .asset
                    .geometry
                    .material_coordinates
                    .as_ref()
                    .unwrap()[i],
                source.asset.geometry.material_coordinates.as_ref().unwrap()[j]
            );
        }
        for (key, _) in &k.signature {
            assert!(selected.keys.contains(key));
        }
        // Every boundary edge is identical, in the same orientation.
        let edges = |r: &Resource| {
            let mut out: BTreeMap<(Key, Key), usize> = BTreeMap::new();
            for t in &r.asset.geometry.triangles {
                for e in 0..3 {
                    *out.entry((r.keys[t[e] as usize], r.keys[t[(e + 1) % 3] as usize]))
                        .or_default() += 1;
                }
            }
            out.iter()
                .filter(|((a, b), _)| !out.contains_key(&(*b, *a)))
                .map(|(k, n)| (*k, *n))
                .collect::<BTreeMap<_, _>>()
        };
        assert_eq!(edges(source), edges(selected));
    }
    assert!(
        reduced > 0,
        "fixture must exercise actual geometry reduction"
    );
    let sources: Vec<_> = a.resources.values().cloned().collect();
    let selected: Vec<_> = a.selected.values().cloned().collect();
    let before = a.geometry_cache.inspect(&a.resources, &a.selected);
    a.update(&c.document, &c, a.anchor, 1.62, 1.).unwrap();
    assert_eq!(before, a.geometry_cache.inspect(&a.resources, &a.selected));
    assert!(
        selected
            .iter()
            .zip(a.selected.values())
            .all(|(a, b)| Arc::ptr_eq(a, b))
    );
    a.settings.atmosphere_geometry = false;
    a.update(&c.document, &c, a.anchor, 1.62, 1.).unwrap();
    assert!(
        a.resources
            .values()
            .zip(&sources)
            .all(|(a, b)| Arc::ptr_eq(a, b))
    );
    assert!(
        a.selected
            .iter()
            .all(|(k, r)| Arc::ptr_eq(r, &a.resources[k]))
    );
}
#[test]
fn planet_orbit_atmospheric_geometry_rechecks_light_and_finite_vacuum() {
    let (c, mut a) = fixture(1000.);
    assert!(
        a.selected
            .iter()
            .any(|(k, r)| !Arc::ptr_eq(r, &a.resources[k]))
    );
    // Light may change without settings or source topology changing.
    let mut light = a.light.clone();
    light.atmosphere.as_mut().unwrap().density = 0.;
    a.set_light(&light);
    a.update(&c.document, &c, a.anchor, 1.62, 1.).unwrap();
    assert!(
        a.selected
            .iter()
            .all(|(k, r)| Arc::ptr_eq(r, &a.resources[k]))
    );
    light.atmosphere.as_mut().unwrap().density = 0.002;
    a.set_light(&light);
    // An external observer sees only a finite shell, never kilometers of vacuum.
    a.settings.distance_meters = c.document.radius() * 10.;
    a.settings.atmosphere_optical_depth = Some(0.01);
    a.update(&c.document, &c, a.anchor, 1.62, 1.).unwrap();
    assert!(
        a.selected
            .iter()
            .all(|(k, r)| Arc::ptr_eq(r, &a.resources[k]))
    );
}

#[test]
#[ignore = "native GPU color/coverage oracle"]
fn planet_orbit_gpu_atmospheric_geometry_motion_and_coverage() {
    use crate::{
        engine::{Command, Engine, ExecutionMode},
        render::Renderer,
    };
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene.entities.retain(|e| e.id == "player");
    scene.entities[0].character_controller = Some(Default::default());
    scene.entities[0].footsteps = None;
    scene.entities[0].transform.position = [0., 0.91, 0.];
    scene.light.atmosphere = Some(crate::atmosphere::Atmosphere {
        density: 0.002,
        sun_strength: 0.,
        ..Default::default()
    });
    let mut e = Engine::new_with_mode(scene, ExecutionMode::Continuous).unwrap();
    let mut config = super::tests::config();
    config.document.operations.push(Relief::Raise {
        direction: [0., 1., -0.002],
        angular_radius_degrees: 0.12,
        height_meters: 300.,
    });
    // Source datum/support at spawn stays explicit.
    config.spawn.altitude_meters = Sampler::new(&config.document).unwrap().height(DVec3::Y) + 0.91;
    e.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(config)),
        }
        .into(),
    )
    .unwrap();
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let mut rows = vec![];
    let mut exercised = 0;
    for (distance, yaw, tilt, width, height) in [
        (4., 0., 0., 640, 400),
        (1000., 0., 0., 640, 400),
        (1000., 12., 0., 800, 400),
        (1000., -12., -15., 400, 640),
        (20000., 0., 0., 640, 400),
        (100000., 0., 0., 640, 400),
    ] {
        let mut settings = Settings {
            atmosphere_geometry: true,
            distance_meters: distance,
            yaw_degrees: yaw,
            tilt_degrees: tilt,
            viewport: [width, height],
            max_tiles: 384,
            ..Default::default()
        };
        e.execute(
            Command::SetPlanetView {
                view: Some(settings.clone()),
            }
            .into(),
        )
        .unwrap();
        let current = e.render_frame();
        let info = e.inspect_planet_view();
        let reduced = info["atmospheric_geometry"]["reduced_tiles"]
            .as_u64()
            .unwrap();
        exercised += reduced;
        let actual = gpu.capture(&current, width, height).unwrap();
        let selected_stats = gpu.info();
        settings.atmosphere_geometry = false;
        e.execute(
            Command::SetPlanetView {
                view: Some(settings),
            }
            .into(),
        )
        .unwrap();
        let reference = e.render_frame();
        let expected = gpu.capture(&reference, width, height).unwrap();
        let reference_stats = gpu.info();
        let a = crate::image_similarity::Image::decode(&actual).unwrap();
        let b = crate::image_similarity::Image::decode(&expected).unwrap();
        let maximum = a
            .pixels
            .iter()
            .zip(&b.pixels)
            .flat_map(|(a, b)| (0..3).map(move |i| a[i].abs_diff(b[i])))
            .max()
            .unwrap();
        assert!(
            maximum <= 1,
            "distance{distance}, yaw{yaw}: maxRGB8={maximum}"
        );
        // Neutral terrain makes coverage differences visible independently of
        // the saturated color check; atmosphere optimization is already baked.
        let mut coverage_a = current;
        let mut coverage_b = reference;
        for f in [&mut coverage_a, &mut coverage_b] {
            f.light.atmosphere = None;
            f.planet_air.as_mut().unwrap().optical_depth = 0.;
            f.planet_air.as_mut().unwrap().space_color = [0.; 3];
            for m in &mut f.meshes.materials {
                *m = Default::default();
            }
        }
        let ma = crate::image_similarity::Image::decode(
            &gpu.capture(&coverage_a, width, height).unwrap(),
        )
        .unwrap();
        let mb = crate::image_similarity::Image::decode(
            &gpu.capture(&coverage_b, width, height).unwrap(),
        )
        .unwrap();
        // Compare terrain-vs-space masks, not relief shading/depth.
        let space = [0, 0, 0];
        let mask = |p: &[u8; 3]| p.iter().zip(space).any(|(a, b)| a.abs_diff(b) > 3);
        assert!(
            ma.pixels.iter().filter(|p| mask(p)).count() > ma.pixels.len() / 100,
            "coverage oracle must contain visible terrain"
        );
        assert!(
            ma.pixels
                .iter()
                .zip(&mb.pixels)
                .all(|(a, b)| mask(a) == mask(b)),
            "silhouette changed at {distance}"
        );
        rows.push(serde_json::json!({"distance":distance,"yaw":yaw,"reduced_tiles":reduced,"maximum_rgb8":maximum,"coverage_mask_equal":true,"atlas":info,"selected_renderer":selected_stats,"reference_renderer":reference_stats}));
    }
    assert!(exercised > 0, "GPU oracle must exercise reduced geometry");
    std::fs::create_dir_all("artifacts/planet-pacing").unwrap();
    std::fs::write("artifacts/planet-pacing/geometry.json",serde_json::to_vec_pretty(&serde_json::json!({"build_id":env!("IGNIS_BUILD_ID"),"rows":rows,"adapter":gpu.info()})).unwrap()).unwrap();
}
