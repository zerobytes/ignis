//! Native GPU oracles: partitioning, depth convention and payload identity.
use super::*;
use crate::{
    engine::{Command, Engine, ExecutionMode},
    meshes::{Frame, SplitPosition},
    model::{Camera, RoomRecipe},
    render::{Clip, RenderFrame, Renderer},
};

fn pixels(png: &[u8]) -> Vec<[u8; 3]> {
    crate::image_similarity::Image::decode(png).unwrap().pixels
}
fn engine_at(direction: DVec3) -> Engine {
    let mut c = super::tests::config();
    c.spawn.direction = direction.to_array();
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene.entities.retain(|e| e.id == "player");
    let p = &mut scene.entities[0];
    p.character_controller = Some(Default::default());
    p.footsteps = None;
    p.transform.position = [0., 0.91, 0.];
    scene.light.atmosphere = Some(crate::atmosphere::Atmosphere {
        density: 0.02,
        sun_strength: 0.,
        ..Default::default()
    });
    let mut e = Engine::new_with_mode(scene, ExecutionMode::Continuous).unwrap();
    e.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(c)),
        }
        .into(),
    )
    .unwrap();
    e.advance(120).unwrap();
    e
}
fn welded(e: &Engine, original: &RenderFrame) -> RenderFrame {
    let mut frame = e.render_frame();
    let mut positions = vec![];
    let mut triangles = vec![];
    let mut rest = vec![];
    let mut normals = vec![];
    let mut split = vec![];
    let mut vertices: BTreeMap<[u32; 12], u32> = BTreeMap::new();
    let mut material = None;
    let mut color = [0.; 3];
    for draw in &original.meshes.draws {
        let Some(coords) = original.meshes.planet_coordinates[draw.asset].as_ref() else {
            continue;
        };
        let a = &original.meshes.source.assets[draw.asset];
        material = Some(a.material.clone());
        color = draw.color;
        let source_normals = original.meshes.normal_overrides[draw.asset]
            .as_ref()
            .unwrap();
        let mut remap = vec![];
        for (i, c) in coords.iter().enumerate() {
            let n = source_normals[i];
            let domain = a.geometry.material_coordinates.as_ref().unwrap()[i];
            let key = std::array::from_fn(|k| {
                if k < 3 {
                    c.high[k].to_bits()
                } else if k < 6 {
                    c.low[k - 3].to_bits()
                } else if k < 9 {
                    n[k - 6].to_bits()
                } else {
                    domain[k - 9].to_bits()
                }
            });
            let index = *vertices.entry(key).or_insert_with(|| {
                let index = positions.len() as u32;
                positions.push(c.high.map(|x| x * 4096.));
                rest.push(domain);
                normals.push(n);
                split.push(*c);
                index
            });
            remap.push(index);
        }
        triangles.extend(
            a.geometry
                .triangles
                .iter()
                .map(|t| t.map(|i| remap[i as usize])),
        );
    }
    let geometry = Geometry {
        positions,
        triangles,
        material_coordinates: Some(rest),
    };
    geometry.validate().unwrap();
    let asset = Asset {
        id: "welded".into(),
        geometry: Arc::new(geometry),
        shading: Shading::Smooth,
        material: material.unwrap(),
        rig: None,
        clips: vec![],
    };
    let doc = crate::meshes::Document {
        assets: vec![asset],
        instances: vec![Instance {
            id: "welded".into(),
            asset: "welded".into(),
            position: [0.; 3],
            rotation: [0., 0., 0., 1.],
            scale: 1.,
            color,
            playback: None,
            attachment: None,
        }],
        ..Default::default()
    };
    frame.meshes = Frame::default();
    frame.meshes.update(&doc, 0);
    frame
        .meshes
        .apply_smooth_normals(&[Arc::new(normals)])
        .unwrap();
    frame
        .meshes
        .apply_planet_coordinates(&[Some(Arc::new(split))])
        .unwrap();
    frame
        .meshes
        .apply_render_placements(&[Mat4::IDENTITY])
        .unwrap();
    frame.planet_air.as_mut().unwrap().terrain_materials = 1;
    frame
}
fn plane() -> Asset {
    let n = 32u32;
    let mut positions = vec![];
    let mut triangles = vec![];
    for y in 0..=n {
        for x in 0..=n {
            positions.push([
                2. * x as f32 / n as f32 - 1.,
                2. * y as f32 / n as f32 - 1.,
                0.,
            ]);
        }
    }
    for y in 0..n {
        for x in 0..n {
            let a = y * (n + 1) + x;
            triangles.extend([[a, a + 1, a + n + 1], [a + 1, a + n + 2, a + n + 1]]);
        }
    }
    Asset {
        id: "plane".into(),
        geometry: Arc::new(Geometry {
            positions,
            triangles,
            material_coordinates: None,
        }),
        shading: Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    }
}
fn instance(id: &str, color: [f32; 3]) -> Instance {
    Instance {
        id: id.into(),
        asset: "plane".into(),
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color,
        playback: None,
        attachment: None,
    }
}
fn empty() -> RenderFrame {
    let mut light = RoomRecipe::default().generate().unwrap().light;
    light.ambient = 1.;
    light.atmosphere = None;
    light.shadow = None;
    light.environment = None;
    let mut frame = crate::network::presentation::empty(
        Camera {
            position: [0.; 3],
            target: [0., 0., -1.],
            fov_degrees: 60.,
        },
        light,
    );
    frame.clip = Some(Clip {
        near: 0.05,
        far: 14_100_000.,
        reverse_z: true,
    });
    frame.suppress_diagnostic_grid = true;
    frame
}

#[test]
#[ignore = "requires native GPU; explicit offscreen precision acceptance"]
fn planet_orbit_gpu_precision_contracts() {
    std::fs::create_dir_all("artifacts/planet-zoom").unwrap();
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let mut seams = vec![];
    for (name, direction, yaw, tilt) in [
        ("edge", DVec3::new(1., 1., 0.25).normalize(), 0., -15.),
        ("edge-side", DVec3::new(1., 1., 0.25).normalize(), 90., -25.),
        ("corner", DVec3::ONE.normalize(), 45., -25.),
    ] {
        let mut e = engine_at(direction);
        for max_tiles in [6, 48] {
            e.execute(
                Command::SetPlanetView {
                    view: Some(Settings {
                        yaw_degrees: yaw,
                        tilt_degrees: tilt,
                        viewport: [640, 400],
                        max_tiles,
                        atmosphere_optical_depth: Some(0.),
                        space_color: [0.; 3],
                        ..Default::default()
                    }),
                }
                .into(),
            )
            .unwrap();
            let frame = e.render_frame();
            let reference = welded(&e, &frame);
            gpu.set_visibility_culling(false);
            let split = gpu.capture(&frame, 640, 400).unwrap();
            let merged = gpu.capture(&reference, 640, 400).unwrap();
            assert_eq!(
                pixels(&split),
                pixels(&merged),
                "partitioning seam at {name}/{max_tiles}"
            );
            assert!(
                pixels(&split).iter().filter(|p| p[1] > 30).count() > 1000,
                "empty fixture"
            );
            gpu.set_visibility_culling(true);
            let culled = gpu.capture(&frame, 640, 400).unwrap();
            assert_eq!(
                pixels(&split),
                pixels(&culled),
                "false planetary frustum rejection"
            );
            std::fs::write(
                format!("artifacts/planet-zoom/seam-{name}-{max_tiles}.png"),
                split,
            )
            .unwrap();
            seams.push(serde_json::json!({"case":name,"tiles_budget":max_tiles,"welded_rgb8_exact":true,"frustum_rgb8_exact":true}));
        }
    }
    let mut depths = vec![];
    for (near, far) in [(100_000f32, 101_000f32), (200_000., 205_000.), (0.9, 1.)] {
        for reverse_order in [false, true] {
            for green in [true, false] {
                let front = if green { [0., 1., 0.] } else { [1., 0., 0.] };
                let back = if green { [1., 0., 0.] } else { [0., 1., 0.] };
                let mut frame = empty();
                let mut instances =
                    vec![self::instance("back", back), self::instance("front", front)];
                let scale = near * 0.75;
                let mut matrices = vec![
                    Mat4::from_scale_rotation_translation(
                        glam::Vec3::splat(scale),
                        glam::Quat::IDENTITY,
                        glam::Vec3::new(0., 0., -far),
                    ),
                    Mat4::from_scale_rotation_translation(
                        glam::Vec3::splat(scale),
                        glam::Quat::IDENTITY,
                        glam::Vec3::new(0., 0., -near),
                    ),
                ];
                if reverse_order {
                    instances.reverse();
                    matrices.reverse();
                }
                let doc = crate::meshes::Document {
                    assets: vec![plane()],
                    instances,
                    ..Default::default()
                };
                frame.meshes.update(&doc, 0);
                frame.meshes.apply_render_placements(&matrices).unwrap();
                let image = gpu.capture(&frame, 640, 400).unwrap();
                let pixel = pixels(&image)[200 * 640 + 320];
                assert_eq!(
                    pixel,
                    if green { [0, 255, 0] } else { [255, 0, 0] },
                    "wrong depth winner at {near}/{far}/{reverse_order}"
                );
                depths.push(serde_json::json!({"near_meters":near,"far_meters":far,"reverse_draw_order":reverse_order,"front_green":green,"near_wins":true}));
            }
        }
    }
    let mut frame = empty();
    let asset = plane();
    let source = asset.geometry.clone();
    let normals = Arc::new(vec![[0., 0., 1.]; source.positions.len()]);
    let a = Arc::new(
        source
            .positions
            .iter()
            .map(|p| SplitPosition::new(DVec3::from(p.map(f64::from))))
            .collect::<Vec<_>>(),
    );
    let b = Arc::new(
        source
            .positions
            .iter()
            .map(|p| SplitPosition::new(DVec3::from(p.map(f64::from)) + DVec3::X * 0.5))
            .collect::<Vec<_>>(),
    );
    let doc = crate::meshes::Document {
        assets: vec![asset],
        instances: vec![self::instance("one", [0., 1., 0.])],
        ..Default::default()
    };
    frame.meshes.update(&doc, 0);
    frame.meshes.apply_smooth_normals(&[normals]).unwrap();
    frame
        .meshes
        .apply_render_placements(&[Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(1000.),
            glam::Quat::IDENTITY,
            glam::Vec3::new(0., 0., -2000.),
        )])
        .unwrap();
    frame.planet_air = Some(crate::render::PlanetAir {
        radius: 1000.,
        altitude: 1000.,
        radial: [0., 0., 1.],
        height: 100.,
        optical_depth: 0.,
        space_color: [0.; 3],
        terrain_materials: 1,
        eye: SplitPosition::new(DVec3::Z * 2.),
        axes: glam::Mat3::IDENTITY.to_cols_array_2d(),
    });
    let mut images = vec![];
    let mut uploads = vec![];
    for coordinates in [Some(a.clone()), Some(b), None, Some(a)] {
        frame
            .meshes
            .apply_planet_coordinates(&[coordinates])
            .unwrap();
        let image = gpu.capture(&frame, 640, 400).unwrap();
        images.push(pixels(&image));
        uploads.push(
            gpu.info()["instancing"]["geometry_uploads"]
                .as_u64()
                .unwrap(),
        );
        assert!(Arc::ptr_eq(
            &source,
            &frame.meshes.source.assets[0].geometry
        ));
    }
    assert_ne!(images[0], images[1]);
    assert_eq!(images[0], images[2]);
    assert_eq!(images[0], images[3]);
    assert!(
        uploads.windows(2).all(|p| p[1] > p[0]),
        "changed payload reused stale geometry"
    );
    frame.meshes.apply_planet_coordinates(&[None]).unwrap();
    frame.clip = None;
    frame.planet_air = None;
    assert_eq!(
        images[0],
        pixels(&gpu.capture(&frame, 640, 400).unwrap()),
        "legacy pipeline was not restored"
    );
    std::fs::write("artifacts/planet-zoom/precision.json",serde_json::to_vec_pretty(&serde_json::json!({"build_id":env!("IGNIS_BUILD_ID"),"seams":seams,"depths":depths,"override_cache_a_b_none_a":true,"legacy_same_frame_renderer_restored":true,"adapter":gpu.info()})).unwrap()).unwrap();
}

#[test]
#[ignore = "requires native GPU; explicit planetary material/layout acceptance"]
fn planet_orbit_gpu_material_layout_motion_resize() {
    let mut e = engine_at(DVec3::Y);
    e.execute(
        Command::SetPlanetView {
            view: Some(Settings {
                distance_meters: 4.,
                tilt_degrees: -35.,
                max_tiles: 48,
                atmosphere_optical_depth: Some(0.),
                atmosphere_optimization: false,
                ..Default::default()
            }),
        }
        .into(),
    )
    .unwrap();
    let mut frame = e.render_frame();
    let source = frame.meshes.source.clone();
    let normals = frame.meshes.normal_overrides.clone();
    let coords = frame.meshes.planet_coordinates.clone();
    let placements: Vec<_> = frame.meshes.draws.iter().map(|d| d.world).collect();
    let gpu_instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&gpu_instance, None)).unwrap();
    let pattern = serde_json::json!({"frequency":[2,3,4],"low":[0.01,0.02,0.03],"high":[0.8,1,0.9],"bump":0.01,"warp":0.3,"octaves":3,"seed":71,"contrast":1.2});
    let mut prior = None;
    for (case, material) in [
        ("plain", serde_json::json!({})),
        ("noise", serde_json::json!({"pattern":pattern})),
        (
            "detail",
            serde_json::json!({"pattern":pattern,"detail":{"plane":"xz","tint":[0.1,0.3,0.9],"opacity":1,"bump":0.02,"triplanar":{"sharpness":4},"segments":[{"start":[-1,-1],"end":[1,1],"widths":[0.5,0.5]}]}}),
        ),
        ("restored", serde_json::json!({})),
    ] {
        let mut doc = source.clone();
        for a in &mut doc.assets {
            a.material = serde_json::from_value(material.clone()).unwrap();
            a.material.validate().unwrap();
        }
        frame.meshes.update(&doc, 0);
        frame
            .meshes
            .apply_smooth_normals(
                &normals
                    .iter()
                    .map(|n| n.clone().unwrap())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        frame.meshes.apply_planet_coordinates(&coords).unwrap();
        frame.meshes.apply_render_placements(&placements).unwrap();
        let mut changed = false;
        for (width, height, light) in [(640, 400, [0., -1., 0.]), (701, 439, [0.4, -0.6, 0.2])] {
            frame.meshes.update(&doc, 0);
            frame
                .meshes
                .apply_smooth_normals(
                    &normals
                        .iter()
                        .map(|n| n.clone().unwrap())
                        .collect::<Vec<_>>(),
                )
                .unwrap();
            frame.meshes.apply_planet_coordinates(&coords).unwrap();
            frame.meshes.apply_render_placements(&placements).unwrap();
            frame.light.direction = light;
            frame.camera.target = [if width == 640 { 0. } else { 0.01 }, 0., -1.];
            gpu.set_geometry_indexing(false);
            gpu.set_compact_vertices(false);
            let expected = pixels(&gpu.capture(&frame, width, height).unwrap());
            assert!(
                expected
                    .iter()
                    .filter(|p| p.iter().any(|&c| c > 10))
                    .count()
                    > 1000,
                "fixture must have visible material/lighting"
            );
            if width == 640 {
                changed = prior.as_ref().is_some_and(|p| *p != expected);
                prior = Some(expected.clone());
            }
            frame.meshes.update_planet_canonical(
                &doc,
                &normals
                    .iter()
                    .map(|n| n.clone().unwrap())
                    .collect::<Vec<_>>(),
                &coords,
            );
            frame.meshes.apply_render_placements(&placements).unwrap();
            assert_eq!(
                expected,
                pixels(&gpu.capture(&frame, width, height).unwrap()),
                "canonical CPU fallback changed pixels in {case}/{width}"
            );
            gpu.set_geometry_indexing(true);
            gpu.set_compact_vertices(true);
            for _ in 0..2 {
                assert_eq!(
                    expected,
                    pixels(&gpu.capture(&frame, width, height).unwrap()),
                    "layout/index raster changed in {case}/{width}"
                );
            }
        }
        if case != "plain" {
            assert!(changed, "material change must be visible: {case}");
        }
    }
}
