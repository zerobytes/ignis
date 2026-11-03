use ignis::{
    engine::{Command, Engine},
    materials::{
        library::{self, Execution},
        recipe::{Field, Layer, NormalFilter, Procedural, Recipe, Sample},
        rock::Family,
    },
    meshes::{Asset, Geometry, Instance, Shading},
    model::{Camera, RoomRecipe, Scene},
    render::Renderer,
};
use std::sync::Arc;
fn source(recipe: Recipe) -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    for e in &mut s.entities {
        e.visible = Some(false);
    }
    s.camera = Camera {
        position: [0.5, 0.5, 1.],
        target: [0.5, 0.5, 0.],
        fov_degrees: 53.130104,
    };
    s.light.ambient = 1.;
    s.meshes.assets = vec![Asset {
        id: "rock".into(),
        geometry: Arc::new(Geometry {
            positions: vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            material_coordinates: None,
        }),
        material: library::material(recipe, Execution::Runtime).unwrap(),
        shading: Shading::Smooth,
        rig: None,
        clips: vec![],
    }];
    s.meshes.instances = vec![Instance {
        id: "slab".into(),
        asset: "rock".into(),
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [1.; 3],
        attachment: None,
        playback: None,
    }];
    s
}
#[test]
fn rock_sources_are_periodic_distinct_and_lossless_with_meaningful_channels() {
    let mut fingerprints = std::collections::BTreeSet::new();
    for family in Family::ALL {
        let mut recipe = family.recipe();
        recipe.seed = u32::MAX;
        let p = Procedural::runtime(recipe.clone());
        p.validate().unwrap();
        let mut fingerprint = 0u64;
        let mut height_min = 1f32;
        let mut height_max = 0f32;
        for y in 0..32 {
            for x in 0..32 {
                let uv = [x as f32 / 32., y as f32 / 32.];
                let a = p.sample(uv, 0.);
                assert_eq!(a, p.sample([uv[0] + 1., uv[1] - 1.], 0.));
                assert_eq!(a.response[0], 0.);
                assert_eq!(a.response[2], 1.);
                assert_eq!(a.emission, [0.; 4]);
                assert!(
                    a.color_roughness
                        .iter()
                        .chain(&a.response)
                        .all(|v| v.is_finite())
                );
                assert!((0.2..=1.).contains(&a.color_roughness[3]));
                assert!(a.response.iter().all(|v| (0. ..=1.).contains(v)));
                height_min = height_min.min(a.response[1]);
                height_max = height_max.max(a.response[1]);
                for v in a.color_roughness.into_iter().chain(a.response) {
                    fingerprint = fingerprint.wrapping_mul(1099511628211) ^ u64::from(v.to_bits());
                }
            }
        }
        assert!(
            height_max - height_min > 0.2,
            "{family:?} lost its surface relief"
        );
        fingerprints.insert(fingerprint);
        for execution in [Execution::Runtime, Execution::Fixed { resolution: 64 }] {
            let m = library::material(recipe.clone(), execution).unwrap();
            assert_eq!(m, library::decode(&library::encode(&m).unwrap()).unwrap());
        }
        let mut unique = p.clone();
        unique.recipe.unique_instances = true;
        assert_eq!(unique.instance("a"), unique.instance("a"));
        assert_ne!(
            unique.instance("a").unwrap().sample([0.17, 0.31], 0.),
            unique.instance("b").unwrap().sample([0.17, 0.31], 0.)
        );
    }
    assert_eq!(fingerprints.len(), 4);
    for family in ignis::materials::marble::Family::ALL {
        let r = family.recipe();
        assert!(
            !serde_json::to_value(&r)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("rock")
        );
        assert!(library::material(r, Execution::Runtime).is_ok());
    }
}
#[test]
fn rock_validation_rejects_conflicts_invalid_controls_and_old_scenes_atomically() {
    let mut engine = Engine::new(source(Recipe::default())).unwrap();
    let before = engine.scene.clone();
    for case in 0..8 {
        let mut recipe = Family::Fractured.recipe();
        let rock = recipe.rock.as_mut().unwrap();
        match case {
            0 => rock.dark[0] = f32::NAN,
            1 => rock.blocks[0] = 0,
            2 => rock.layers = 129,
            3 => rock.direction = [0, 0],
            4 => rock.joint_width = 0.,
            5 => rock.roughness = f32::INFINITY,
            6 => rock.grain_frequency = 2049,
            _ => recipe.marble = Some(Default::default()),
        }
        let material = ignis::materials::Material {
            procedural: Some(Arc::new(Procedural::runtime(recipe))),
            ..Default::default()
        };
        assert!(
            engine
                .execute(
                    Command::EditMesh {
                        edit: ignis::meshes::Edit::SetMaterial {
                            asset: "rock".into(),
                            material
                        }
                    }
                    .into()
                )
                .is_err()
        );
        assert_eq!(engine.scene, before);
    }
    let mut old = source(Recipe {
        normal_filter: NormalFilter::Pixel,
        ..Default::default()
    });
    old.version = 116;
    assert!(Engine::new(old).is_err());
    let mut old = source(Family::Alpine.recipe());
    old.version = 116;
    assert!(Engine::new(old).is_err());
    let mut old = source(Recipe::default());
    old.version = 116;
    let mut engine = Engine::new(old).unwrap();
    engine
        .execute(
            Command::EditMesh {
                edit: ignis::meshes::Edit::SetMaterial {
                    asset: "rock".into(),
                    material: library::material(Family::Ochre.recipe(), Execution::Runtime)
                        .unwrap(),
                },
            }
            .into(),
        )
        .unwrap();
    assert!(engine.scene.version >= 117);
}
fn srgb(v: f32) -> u8 {
    let v = v.clamp(0., 1.);
    ((if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1. / 2.4) - 0.055
    }) * 255.)
        .round() as u8
}
fn rgba(bytes: &[u8]) -> Vec<u8> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut out = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut out).unwrap();
    out.truncate(info.buffer_size());
    out
}
#[test]
#[ignore = "requires native GPU"]
fn native_rock_cpu_gpu_layers_fixed_maps_and_lighting_agree() {
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let n = 512usize;
    for family in Family::ALL {
        let mut r = family.recipe();
        r.seed = u32::MAX;
        r.height_m = 0.;
        r.layers.push(Layer {
            field: Field::Waves,
            frequency: [3, 5],
            strength: 0.17,
            target: Sample {
                color_roughness: [0.12, 0.15, 0.18, 0.7],
                response: [0., 0.3, 1., 0.8],
                emission: [0.; 4],
            },
            ..Default::default()
        });
        let mut s = source(r.clone());
        let live = rgba(
            &gpu.capture(
                &Engine::new(s.clone()).unwrap().render_frame(),
                n as u32,
                n as u32,
            )
            .unwrap(),
        );
        let mut error = 0.;
        let mut count = 0;
        for y in (4..n - 4).step_by(3) {
            for x in (4..n - 4).step_by(3) {
                let a = r.evaluate(
                    [
                        (x as f32 + 0.5) / n as f32,
                        1. - (y as f32 + 0.5) / n as f32,
                    ],
                    1. / n as f32,
                );
                for (k, c) in a.color_roughness[..3].iter().enumerate() {
                    error +=
                        f64::from(live[4 * (y * n + x) + k].abs_diff(srgb(*c * a.response[3])));
                    count += 1;
                }
            }
        }
        let average = error / f64::from(count);
        println!("{family:?}: CPU/GPU RGB8 mean {average:.5}");
        assert!(average < 1.5, "{family:?} CPU/GPU {average}");
        s.meshes.assets[0].material =
            library::material(r, Execution::Fixed { resolution: 512 }).unwrap();
        let fixed = rgba(
            &gpu.capture(
                &Engine::new(s.clone()).unwrap().render_frame(),
                n as u32,
                n as u32,
            )
            .unwrap(),
        );
        let mean = live
            .iter()
            .zip(&fixed)
            .map(|(a, b)| f64::from(a.abs_diff(*b)))
            .sum::<f64>()
            / live.len() as f64;
        println!("{family:?}: runtime/fixed RGB8 mean {mean:.5}");
        assert!(mean < 6., "{family:?} fixed/runtime {mean}");
        let uploads = gpu.info()["procedural_materials"]["uploads"].clone();
        assert_eq!(
            fixed,
            rgba(
                &gpu.capture(&Engine::new(s).unwrap().render_frame(), n as u32, n as u32)
                    .unwrap()
            )
        );
        assert_eq!(uploads, gpu.info()["procedural_materials"]["uploads"]);
    }
    let mut r = Family::Fractured.recipe();
    let mut s = source(r.clone());
    s.light.ambient = 0.12;
    s.light.direction = [-0.5, -0.7, -1.];
    let a = rgba(
        &gpu.capture(
            &Engine::new(s.clone()).unwrap().render_frame(),
            n as u32,
            n as u32,
        )
        .unwrap(),
    );
    r.normal_filter = NormalFilter::Screen;
    s.meshes.assets[0].material = library::material(r.clone(), Execution::Runtime).unwrap();
    let screen = rgba(
        &gpu.capture(
            &Engine::new(s.clone()).unwrap().render_frame(),
            n as u32,
            n as u32,
        )
        .unwrap(),
    );
    let different = a
        .chunks_exact(4)
        .zip(screen.chunks_exact(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    assert!(
        different > n * n / 30,
        "Per-pixel normals must independently change steep relief"
    );
    r.height_m = 0.;
    s.meshes.assets[0].material = library::material(r, Execution::Runtime).unwrap();
    let flat = rgba(
        &gpu.capture(
            &Engine::new(s.clone()).unwrap().render_frame(),
            n as u32,
            n as u32,
        )
        .unwrap(),
    );
    assert_ne!(a, flat);
    s.light.direction = [0.7, -0.25, -0.8];
    let b = rgba(
        &gpu.capture(&Engine::new(s).unwrap().render_frame(), n as u32, n as u32)
            .unwrap(),
    );
    assert_ne!(flat, b);
}
