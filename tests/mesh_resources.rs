//! Critical boundaries: large documents retain identity/atomicity and share
//! storage; the GPU comparison protects appearance across the new representation.
use ignis::{
    engine::{Command, Engine, SceneEdit},
    meshes::{Asset, Document, Edit, Frame, Geometry, Instance, Shading},
    model::{RoomRecipe, Scene},
};
use std::sync::Arc;
fn asset(id: &str, g: Geometry) -> Asset {
    Asset {
        id: id.into(),
        geometry: Arc::new(g),
        shading: Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    }
}
fn instance(n: usize, asset: &str) -> Instance {
    Instance {
        id: format!("tree-{n}"),
        asset: asset.into(),
        position: [(n % 100) as f32 * 2., 0., (n / 100) as f32 * 2.],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [0.2, 0.5, 0.1],
        playback: None,
        attachment: None,
    }
}
// Persistence must not refuse a valid large catalog, truncate its tail, or
// replace the current world when a streamed candidate is malformed.
#[test]
fn large_local_scene_streams_atomically_and_reopens_through_project_paths() {
    use ignis::service::Service;
    fn assert_scene_equal(actual: &Scene, expected: &Scene) {
        if actual != expected {
            let a = serde_json::to_value(actual).unwrap();
            let b = serde_json::to_value(expected).unwrap();
            for (key, value) in a.as_object().unwrap() {
                if value != &b[key] {
                    eprintln!("Different scene field: {key}");
                    if key != "meshes" {
                        eprintln!("actual={value}, expected={}", b[key]);
                    }
                }
            }
            panic!("Scene roundtrip changed values");
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    // LoadScene canonicalizes entity order; start this persistence fixture in
    // that order so the equality check isolates serialized values.
    scene.entities.sort_by(|a, b| a.id.cmp(&b.id));
    scene.meshes = Document {
        objects: Default::default(),
        assets: vec![asset("tree", Geometry::grid(1, 1.).unwrap())],
        instances: (0..60000).map(|n| instance(n, "tree")).collect(),
    };
    let mut service = Service::new(dir.path(), scene).unwrap();
    let path = "scenes/large.scene.json";
    service
        .execute(Command::SaveScene { path: path.into() }.into())
        .unwrap();
    let file = dir.path().join(path);
    assert!(std::fs::metadata(&file).unwrap().len() > 8 * 1024 * 1024);
    let expected = service.engine.scene.clone();
    service
        .execute(Command::LoadScene { path: path.into() }.into())
        .unwrap();
    assert_scene_equal(&service.engine.scene, &expected);
    service
        .execute(
            Command::OpenProjectScene {
                path: path.into(),
                discard_changes: false,
            }
            .into(),
        )
        .unwrap();
    assert_scene_equal(&service.engine.scene, &expected);
    std::fs::write(
        dir.path().join("ignis.project.json"),
        serde_json::to_vec(&serde_json::json!({
            "version":1,"name":"Large","entry_scene":path,"scenes":[path]
        }))
        .unwrap(),
    )
    .unwrap();
    service
        .execute(
            Command::OpenProject {
                path: dir.path().to_string_lossy().into(),
                discard_changes: false,
            }
            .into(),
        )
        .unwrap();
    assert_scene_equal(&service.engine.scene, &expected);
    let revision = service.engine.revision;
    // Valid complete scene followed by junk must be rejected, including at EOF.
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(&file)
        .unwrap()
        .write_all(b" trailing")
        .unwrap();
    assert!(
        service
            .execute(Command::LoadScene { path: path.into() }.into())
            .is_err()
    );
    assert_scene_equal(&service.engine.scene, &expected);
    assert_eq!(service.engine.revision, revision);
    // The replay transport/resource envelope is independent and remains bounded.
    let err = service
        .execute(Command::RunReplay { path: path.into() }.into())
        .err()
        .unwrap();
    assert!(err.message.contains("8 MiB"), "{}", err.message);
    service
        .execute(Command::SaveScene { path: path.into() }.into())
        .unwrap();
    service
        .execute(Command::LoadScene { path: path.into() }.into())
        .unwrap();
    assert_scene_equal(&service.engine.scene, &expected);
    // Failed replacement must leave the previous file and session intact.
    service.project = None;
    let original = std::fs::read(&file).unwrap();
    std::fs::create_dir(dir.path().join("scenes/blocked")).unwrap();
    let prior_path = service.scene_path.clone();
    assert!(
        service
            .execute(
                Command::SaveScene {
                    path: "scenes/blocked".into()
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(std::fs::read(&file).unwrap(), original);
    assert_eq!(service.scene_path, prior_path);
}
#[test]
fn large_mesh_catalog_and_ten_thousand_instances_roundtrip_and_rollback_without_expansion() {
    let n = 130u32;
    let g = Geometry {
        positions: (0..=n)
            .flat_map(|z| (0..=n).map(move |x| [x as f32 / 130., 0., z as f32 / 130.]))
            .collect(),
        triangles: (0..n)
            .flat_map(|z| {
                (0..n).flat_map(move |x| {
                    let a = z * (n + 1) + x;
                    [[a, a + n + 1, a + 1], [a + 1, a + n + 1, a + n + 2]]
                })
            })
            .collect(),
        material_coordinates: None,
    };
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene.meshes = Document {
        objects: Default::default(),
        assets: vec![asset("tree", g)],
        instances: (0..10000).map(|n| instance(n, "tree")).collect(),
    };
    for n in 0..20 {
        scene
            .meshes
            .assets
            .push(asset(&format!("other-{n}"), Geometry::grid(1, 1.).unwrap()));
    }
    scene.validate().unwrap();
    let encoded = serde_json::to_vec(&scene).unwrap();
    let decoded: Scene = serde_json::from_slice(&encoded).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded, scene);
    let mut legacy = scene.clone();
    legacy.version = 102;
    assert!(
        legacy
            .validate()
            .unwrap_err()
            .message
            .contains("version103")
    );
    let mut frame = Frame::default();
    let start = std::time::Instant::now();
    frame.update(&scene.meshes, 0);
    assert_eq!(frame.triangle_count(), 338_000_000);
    assert_eq!(frame.resident_vertex_count(), 33_800 * 3);
    assert!(frame.vertices.is_empty());
    eprintln!(
        "10k instances /338M submitted triangles: {} resident vertices, {:.2}ms preparation",
        frame.resident_vertex_count(),
        start.elapsed().as_secs_f64() * 1000.
    );
    let mut engine = Engine::new(scene).unwrap();
    let before = engine.scene.clone();
    let mut bad = instance(10001, "absent");
    bad.id = "invalid".into();
    assert!(
        engine
            .execute(
                Command::EditScene {
                    edits: vec![
                        SceneEdit::Mesh {
                            edit: Edit::PutInstance {
                                instance: instance(10000, "tree")
                            }
                        },
                        SceneEdit::Mesh {
                            edit: Edit::PutInstance { instance: bad }
                        }
                    ]
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(engine.scene, before);
    assert_eq!(engine.render_frame().meshes.triangle_count(), 338_000_000);
}
fn visual_scene(shared: bool) -> Scene {
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    for e in &mut scene.entities {
        e.visible = Some(false);
    }
    scene.camera = ignis::model::Camera {
        position: [0., 8., 12.],
        target: [0., 0., 0.],
        fov_degrees: 55.,
    };
    scene
        .meshes
        .assets
        .push(asset("tree", Geometry::grid(4, 1.).unwrap()));
    scene.meshes.assets[0].material.pattern = Some(ignis::materials::Pattern {
        frequency: [5., 2., 5.],
        low: [0.3; 3],
        high: [1.2; 3],
        bump: 0.002,
        warp: 0.5,
        octaves: 3,
        seed: 42,
        contrast: 1.5,
        kind: ignis::materials::PatternKind::Ridged,
    });
    for n in 0..40 {
        let mut i = instance(n, "tree");
        i.position = [
            (n % 8) as f32 * 1.25 - 4.5,
            (n % 3) as f32 * 0.25,
            (n / 8) as f32 * 1.5 - 3.,
        ];
        i.color = [0.1 + (n % 7) as f32 * 0.1, 0.2 + (n % 5) as f32 * 0.1, 0.3];
        i.rotation = glam::Quat::from_rotation_y(n as f32 * 0.13).to_array();
        if !shared {
            let mut a = scene.meshes.assets[0].clone();
            a.id = format!("asset-{n}");
            i.asset = a.id.clone();
            scene.meshes.assets.push(a);
        }
        scene.meshes.instances.push(i);
    }
    scene
}
#[test]
#[ignore = "Requires a native GPU; run explicitly with --ignored --nocapture"]
fn shared_gpu_matches_expanded_geometry_materials_shadows_and_selection() {
    let out = std::path::Path::new("artifacts/mesh-resources");
    std::fs::create_dir_all(out).unwrap();
    let gpu = wgpu::Instance::new(&Default::default());
    let mut renderer = pollster::block_on(ignis::render::Renderer::new(&gpu, None)).unwrap();
    let mut results = Vec::new();
    for mode in 0..6 {
        let mut images = Vec::new();
        let mut stats = Vec::new();
        for shared in [false, true] {
            let mut scene = visual_scene(shared);
            if mode == 5 {
                // Exercise indexed neutral surfaces as well as the procedural
                // stream fallback, with the same instancing/depth contract.
                for asset in &mut scene.meshes.assets {
                    asset.material = Default::default();
                }
            }
            if mode == 1 || mode == 5 {
                scene.light.shadow = Some(ignis::model::DirectionalShadow {
                    center: [0.; 3],
                    extent: 12.,
                    resolution: 512,
                    bias: 0.001,
                    receiver_plane_bias: false,
                });
            }
            if mode == 2 {
                scene.light.atmosphere = Some(ignis::atmosphere::Atmosphere {
                    density: 0.5,
                    color_error: 0.02,
                    silhouette_error_px: 0.1,
                    ..Default::default()
                });
                scene.distant_meshes = ignis::distant::compile(&scene.meshes).unwrap();
            }
            if mode == 4 {
                // Coplanar mixed paths: keep authored depth-tie priority.
                let mut a = scene.meshes.assets[0].clone();
                a.id = "single".into();
                scene.meshes.assets.push(a);
                let mut i = scene.meshes.instances[0].clone();
                i.id = "overlap".into();
                i.asset = "single".into();
                i.color = [1., 0., 0.];
                scene.meshes.instances.insert(1, i);
            }
            let engine = Engine::new(scene).unwrap();
            let mut frame = engine.render_frame();
            if mode == 3 {
                frame.selection_mesh = Some("tree-0".into());
            }
            let bytes = renderer.capture(&frame, 640, 480).unwrap();
            std::fs::write(
                out.join(format!(
                    "mode-{mode}-{}.png",
                    if shared { "shared" } else { "expanded" }
                )),
                &bytes,
            )
            .unwrap();
            images.push(ignis::image_similarity::Image::decode(&bytes).unwrap());
            stats.push(renderer.info());
            if mode == 2 && shared {
                assert!(
                    renderer.info()["instancing"]["cheap_triangles"]
                        .as_u64()
                        .unwrap()
                        > 0
                );
            }
            if shared {
                let before = renderer.info()["instancing"]["geometry_uploads"].clone();
                let repeated = renderer.capture(&frame, 640, 480).unwrap();
                assert_eq!(bytes, repeated, "unchanged shared capture mode{mode}");
                assert_eq!(renderer.info()["instancing"]["geometry_uploads"], before);
            }
        }
        let errors: Vec<_> = images[0]
            .pixels
            .iter()
            .zip(&images[1].pixels)
            .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)))
            .collect();
        let max = *errors.iter().max().unwrap();
        let changed = errors.iter().filter(|&&e| e > 0).count();
        eprintln!("mode{mode}: maxRGB8={max} changed channels={changed}");
        results.push(serde_json::json!({"mode":mode,"max_rgb8":max,"changed_channels":changed,"expanded":stats[0],"shared":stats[1]}));
        std::fs::write(
            out.join("comparison.json"),
            serde_json::to_vec_pretty(&results).unwrap(),
        )
        .unwrap();
        assert!(max <= 2, "mode{mode}: max error {max}");
    }
}

/// Critical renderer integration: visibility rejection must not erase visible
/// geometry or off-camera shadow casters, including after camera/pose changes.
#[test]
#[ignore = "Requires a native GPU; run explicitly with --ignored --nocapture"]
fn visibility_preserves_pixels_across_views_shadows_resize_and_motion() {
    let gpu = wgpu::Instance::default();
    let mut renderer = pollster::block_on(ignis::render::Renderer::new(&gpu, None)).unwrap();
    let mut rejected = 0;
    let mut saved = 0;
    for shadow in [false, true] {
        let mut scene = visual_scene(true);
        if shadow {
            scene.light.shadow = Some(ignis::model::DirectionalShadow {
                center: [0., 0., 0.],
                extent: 12.,
                resolution: 512,
                bias: 0.001,
                receiver_plane_bias: false,
            });
        }
        // Include static shared, unique, coplanar tie and extreme off-screen objects.
        let mut a = scene.meshes.assets[0].clone();
        a.id = "unique".into();
        scene.meshes.assets.push(a);
        let mut item = scene.meshes.instances[0].clone();
        item.id = "tie".into();
        item.asset = "unique".into();
        item.color = [1., 0., 0.];
        scene.meshes.instances.insert(1, item);
        for n in 0..8 {
            let mut item = scene.meshes.instances[0].clone();
            item.id = format!("outside-{n}");
            item.position = [if n % 2 == 0 { 500. } else { -500. }, 0., n as f32 * 80.];
            scene.meshes.instances.push(item);
        }
        let member = scene.meshes.instances[0].id.clone();
        let mut parent = scene.meshes.instances[0].clone();
        parent.id = "fallback-parent".into();
        let mut child = parent.clone();
        child.id = "fallback-child".into();
        child.attachment = Some(ignis::meshes::Attachment {
            parent: parent.id.clone(),
            bone: None,
        });
        scene.meshes.instances.extend([parent, child]);
        scene.meshes.objects = Arc::new(vec![
            ignis::mesh_objects::Object {
                id: "mixed".into(),
                members: vec![member, "tie".into()],
            },
            ignis::mesh_objects::Object {
                id: "outside".into(),
                members: vec!["outside-0".into(), "outside-2".into()],
            },
            ignis::mesh_objects::Object {
                id: "attachment-fallback".into(),
                members: vec!["fallback-parent".into(), "fallback-child".into()],
            },
        ]);
        for (pose, (width, height)) in [
            (0, (320, 240)),
            (1, (640, 360)),
            (2, (240, 320)),
            (3, (320, 240)),
            (0, (320, 240)),
        ] {
            let mut s = scene.clone();
            match pose {
                1 => {
                    s.camera.position = [0., 2., -10.];
                    s.camera.target = [0., 0., 0.];
                }
                2 => {
                    s.camera.position = [0., 0.1, 0.1];
                    s.camera.target = [0., 0., -2.];
                }
                3 => {
                    s.camera.position = [500., 3., 8.];
                    s.camera.target = [500., 0., 0.];
                    s.meshes.instances[0].position = [500., 0., 0.];
                }
                _ => {}
            }
            s.light.direction = [0.3 + pose as f32 * 0.2, -1., -0.4];
            s.meshes.instances[0].rotation =
                glam::Quat::from_rotation_y(pose as f32 * 0.4).to_array();
            let e = Engine::new(s).unwrap();
            let frame = e.render_frame();
            renderer.set_visibility_culling(true);
            renderer.set_object_culling(false);
            let old = renderer.capture(&frame, width, height).unwrap();
            renderer.set_object_culling(true);
            let grouped = renderer.capture(&frame, width, height).unwrap();
            assert_eq!(
                old, grouped,
                "group membership changed render at pose{pose}"
            );
            assert_eq!(renderer.info()["visibility"]["object_groups"], 2);
            saved += renderer.info()["visibility"]["object_camera_tests_saved"]
                .as_u64()
                .unwrap();
            renderer.set_visibility_culling(false);
            let reference = ignis::image_similarity::Image::decode(
                &renderer.capture(&frame, width, height).unwrap(),
            )
            .unwrap();
            renderer.set_visibility_culling(true);
            let image = ignis::image_similarity::Image::decode(
                &renderer.capture(&frame, width, height).unwrap(),
            )
            .unwrap();
            assert!(
                image.pixels == reference.pixels,
                "visibility changed pixels: shadow={shadow},pose={pose}"
            );
            rejected += renderer.info()["visibility"]["rejected_mesh_instances"]
                .as_u64()
                .unwrap();
        }
    }
    assert!(rejected > 0, "test must exercise actual rejection");
    assert!(saved > 0, "test must exercise compound object rejection");
}

/// Page splitting must preserve instance order, including coplanar depth ties.
/// Otherwise a later object's first page can cover an earlier object's last page.
#[test]
#[ignore = "Requires a native GPU; run explicitly with --ignored --nocapture"]
fn indexed_pages_preserve_coplanar_instance_priority() {
    let mut scene = visual_scene(true);
    let n = 256u32;
    let geometry = Geometry {
        positions: (0..=n)
            .flat_map(|y| (0..=n).map(move |x| [x as f32 / 8. - 16., y as f32 / 8. - 16., 0.]))
            .collect(),
        triangles: (0..n)
            .flat_map(|y| {
                (0..n).flat_map(move |x| {
                    let a = y * (n + 1) + x;
                    [[a, a + n + 1, a + 1], [a + 1, a + n + 1, a + n + 2]]
                })
            })
            .collect(),
        material_coordinates: None,
    };
    let mut first = instance(0, "sheet");
    first.position = [0.; 3];
    first.color = [1., 0., 0.];
    let mut second = first.clone();
    second.id = "second".into();
    second.color = [0., 1., 0.];
    second.rotation = [0., 0., 1., 0.];
    scene.meshes = Document {
        objects: Default::default(),
        assets: vec![asset("sheet", geometry)],
        instances: vec![first, second],
    };
    scene.camera = ignis::model::Camera {
        position: [0., 0., 30.],
        target: [0.; 3],
        fov_degrees: 60.,
    };
    scene.light.ambient = 1.;
    let frame = Engine::new(scene).unwrap().render_frame();
    let gpu = wgpu::Instance::new(&Default::default());
    let mut renderer = pollster::block_on(ignis::render::Renderer::new(&gpu, None)).unwrap();
    renderer.set_geometry_indexing(false);
    let reference = renderer.capture(&frame, 512, 512).unwrap();
    renderer.set_geometry_indexing(true);
    let indexed = renderer.capture(&frame, 512, 512).unwrap();
    assert!(
        renderer.info()["instancing"]["selected_indices"]
            .as_u64()
            .unwrap()
            > 0
    );
    let a = ignis::image_similarity::Image::decode(&reference).unwrap();
    let b = ignis::image_similarity::Image::decode(&indexed).unwrap();
    let changed = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(changed, 0, "paging changed coplanar priority");
    assert_eq!(indexed, renderer.capture(&frame, 512, 512).unwrap());
}

/// Changing a material without changing geometry must not reuse a layout that
/// discarded its coordinates. Exercise both eligibility transitions in one cache.
#[test]
#[ignore = "requires native GPU"]
fn consumed_attributes_preserve_pixels_and_material_cache_transitions() {
    let gpu = wgpu::Instance::default();
    let mut original = pollster::block_on(ignis::render::Renderer::new(&gpu, None)).unwrap();
    let mut optimized = pollster::block_on(ignis::render::Renderer::new(&gpu, None)).unwrap();
    original.set_minimal_vertices(false);
    optimized.set_minimal_vertices(true);
    // Isolate layout invalidation from the existing index-policy invalidation.
    original.set_geometry_indexing(false);
    optimized.set_geometry_indexing(false);
    let source = visual_scene(true);
    let pattern = source.meshes.assets[0].material.pattern;
    let mut frame = Engine::new(source.clone()).unwrap().render_frame();
    let mut document = source.meshes.clone();
    for mode in 0..9 {
        document.assets[0].material = Default::default();
        match mode {
            2 => document.assets[0].material.pattern = pattern,
            4 => document.assets[0].material.secondary_pattern = pattern,
            6 => {
                document.assets[0].material.detail = Some(
                    serde_json::from_value(serde_json::json!({
                        "plane":"xz","tint":[0.3,0.8,0.4],"opacity":0.7,"bump":0.01,
                        "segments":[{"start":[-0.4,0.0],"end":[0.4,0.0],"widths":[0.1,0.1]}]
                    }))
                    .unwrap(),
                )
            }
            7 => {
                document.assets[0].material.transmission = 0.4;
                document.assets[0].material.reflectance = Some(ignis::materials::Reflectance {
                    roughness: 0.6,
                    f0: 0.06,
                    metallic: 0.,
                });
            }
            _ => {}
        }
        document.validate().unwrap();
        frame.meshes.update(&document, 0);
        frame.light.direction = [0.3 + mode as f32 * 0.08, -1., 0.2];
        frame.camera.position[0] = mode as f32 * 0.25;
        frame.selection_mesh = if mode == 8 {
            Some("tree-0".into())
        } else {
            None
        };
        if mode == 1 {
            frame.light.shadow = Some(ignis::model::DirectionalShadow {
                center: [0.; 3],
                extent: 12.,
                resolution: 512,
                bias: 0.001,
                receiver_plane_bias: false,
            });
        }
        let a = original.capture(&frame, 640, 480).unwrap();
        let b = optimized.capture(&frame, 640, 480).unwrap();
        assert_eq!(a, b, "material transition {mode} changed pixels");
        assert_eq!(
            b,
            optimized.capture(&frame, 640, 480).unwrap(),
            "repeat changed pixels"
        );
        let count = optimized.info()["instancing"]["minimal_instances"]
            .as_u64()
            .unwrap();
        assert_eq!(
            count > 0,
            !matches!(mode, 2 | 4 | 6),
            "wrong layout at {mode}"
        );
    }
}

/// A unique resident asset must not disappear or keep stale geometry/materials
/// after edits or switching back to the source oracle. This protects the new
/// route's integration/cache invalidation, including shadows and transforms.
#[test]
#[ignore = "requires native GPU"]
fn unique_static_residency_preserves_edits_transforms_and_shadows() {
    let mut scene = visual_scene(true);
    let pattern = scene.meshes.assets[0].material.pattern;
    let mut geometry = Geometry::grid(24, 4.).unwrap();
    for p in &mut geometry.positions {
        p[1] = (p[0] * 2.).sin() * (p[2] * 2.).cos() * 0.2;
    }
    scene.meshes = Document {
        objects: Default::default(),
        assets: vec![asset("unique", geometry)],
        instances: vec![instance(0, "unique")],
    };
    scene.camera = ignis::model::Camera {
        position: [4., 3., 5.],
        target: [0.; 3],
        fov_degrees: 60.,
    };
    let gpu = wgpu::Instance::default();
    let mut renderer = pollster::block_on(ignis::render::Renderer::new(&gpu, None)).unwrap();
    let engine = Engine::new(scene.clone()).unwrap();
    let mut frame = engine.render_frame();
    for mode in 0..6 {
        let mut doc = scene.meshes.clone();
        doc.instances[0].position = [mode as f32 * 0.013, 0., 0.];
        doc.instances[0].rotation = glam::Quat::from_rotation_y(mode as f32 * 0.21).to_array();
        doc.instances[0].scale = 1. + mode as f32 * 0.09;
        if mode % 2 == 1 {
            doc.assets[0].material.pattern = pattern;
        }
        if mode == 3 {
            Arc::make_mut(&mut doc.assets[0].geometry).positions[10][1] += 0.6;
        }
        frame.light.direction = [0.3 + mode as f32 * 0.08, -1., 0.2];
        if mode >= 4 {
            frame.light.shadow = Some(ignis::model::DirectionalShadow {
                center: [0.; 3],
                extent: 8.,
                resolution: 256,
                bias: 0.0005,
                receiver_plane_bias: false,
            });
        }
        let (w, h) = if mode == 5 { (480, 320) } else { (384, 384) };
        frame.meshes.set_unique_static_residency(false);
        frame.meshes.update(&doc, 0);
        assert!(!frame.meshes.vertices.is_empty());
        let source = renderer.capture(&frame, w, h).unwrap();
        frame.meshes.set_unique_static_residency(true);
        frame.meshes.update(&doc, 0);
        assert!(frame.meshes.vertices.is_empty());
        let resident = renderer.capture(&frame, w, h).unwrap();
        assert_eq!(resident, renderer.capture(&frame, w, h).unwrap());
        let a = ignis::image_similarity::Image::decode(&source).unwrap();
        let b = ignis::image_similarity::Image::decode(&resident).unwrap();
        let max = a
            .pixels
            .iter()
            .flatten()
            .zip(b.pixels.iter().flatten())
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert!(max <= 2, "mode{mode} maxRGB8={max}");
    }
}
