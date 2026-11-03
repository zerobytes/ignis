use ignis::{
    engine::{Command, Engine},
    materials::{
        library::{self, Execution},
        marble::Family,
        recipe::{Fixed, Procedural, Recipe, Sample},
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
        id: "marble".into(),
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
        asset: "marble".into(),
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
fn families_are_distinct_periodic_and_exact_in_bounded_library2_files() {
    let mut fingerprints = std::collections::BTreeSet::new();
    for family in Family::ALL {
        let mut r = family.recipe();
        r.seed = u32::MAX;
        let p = Procedural::runtime(r);
        p.validate().unwrap();
        let mut h = 0u64;
        for y in 0..12 {
            for x in 0..12 {
                let uv = [x as f32 / 16., y as f32 / 16.];
                let a = p.sample(uv, 0.);
                assert_eq!(a, p.sample([uv[0] + 1., uv[1] - 1.], 0.));
                assert!(
                    a.color_roughness
                        .iter()
                        .chain(&a.response)
                        .all(|v| v.is_finite())
                );
                for v in a.color_roughness {
                    h = h.wrapping_mul(1099511628211) ^ u64::from(v.to_bits());
                }
            }
        }
        fingerprints.insert(h);
        let m = library::material(p.recipe.clone(), Execution::Fixed { resolution: 32 }).unwrap();
        let bytes = library::encode(&m).unwrap();
        assert!(bytes.starts_with(b"IGNISMAT2"));
        assert_eq!(m, library::decode(&bytes).unwrap());
        let mut corrupt = bytes.clone();
        corrupt[13..15].copy_from_slice(&4096u16.to_le_bytes());
        assert!(library::decode(&corrupt).is_err());
        assert!(library::decode(&bytes[..bytes.len() / 2]).is_err());
    }
    assert_eq!(fingerprints.len(), Family::ALL.len());
    assert!(library::decode(br#"{"version":1,"material":{"transmission":0}}"#).is_ok());
}
#[test]
fn invalid_mineral_parameters_reject_atomically_and_require_a_new_scene() {
    let mut engine = Engine::new(source(Family::VerdeBreccia.recipe())).unwrap();
    let before = engine.scene.clone();
    for invalid in 0..5 {
        let mut recipe = Family::EmeraldGold.recipe();
        let m = recipe.marble.as_mut().unwrap();
        match invalid {
            0 => m.dark[0] = f32::NAN,
            1 => m.direction = [0, 0],
            2 => m.network = 0,
            3 => m.gold_width = 0.,
            _ => m.gold_flow = 1.1,
        };
        let material = ignis::materials::Material {
            procedural: Some(Arc::new(Procedural::runtime(recipe))),
            ..Default::default()
        };
        assert!(
            engine
                .execute(
                    Command::EditMesh {
                        edit: ignis::meshes::Edit::SetMaterial {
                            asset: "marble".into(),
                            material
                        }
                    }
                    .into()
                )
                .is_err()
        );
        assert_eq!(before, engine.scene);
    }
    let mut old = before;
    old.version = 114;
    assert!(Engine::new(old).is_err());
    let good = Family::Carrara.recipe();
    let mut old = source(Recipe::default());
    old.version = 114;
    let mut engine = Engine::new(old).unwrap();
    engine
        .execute(
            Command::EditMesh {
                edit: ignis::meshes::Edit::SetMaterial {
                    asset: "marble".into(),
                    material: library::material(good, Execution::Runtime).unwrap(),
                },
            }
            .into(),
        )
        .unwrap();
    assert!(engine.scene.version >= 115);
}
#[test]
fn high_resolution_bake_keeps_the_complete_mip_chain_and_cold_bytes_bounded() {
    let p = Procedural::runtime(Family::Carrara.recipe());
    let f = p.bake(512).unwrap();
    f.validate().unwrap();
    assert_eq!(f.fixed.as_ref().unwrap().samples.len(), 349525);
    let mat = ignis::materials::Material {
        procedural: Some(Arc::new(f)),
        ..Default::default()
    };
    let bytes = library::encode(&mat).unwrap();
    assert_eq!(mat, library::decode(&bytes).unwrap());
    let mut invalid = (*mat.procedural.unwrap()).clone();
    invalid.fixed = Some(Fixed {
        resolution: 2048,
        samples: Arc::new(vec![Sample::default()]),
    });
    assert!(invalid.validate().is_err());
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
fn native_mineral_program_matches_the_cpu_channels_and_fixed_mode() {
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    for family in [
        Family::VerdeBreccia,
        Family::EmeraldWhiteGold,
        Family::EmeraldGold,
        Family::Carrara,
    ] {
        let mut r = family.recipe();
        r.seed = u32::MAX;
        let mut s = source(r.clone());
        let n = 512usize;
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
                let uv = [
                    (x as f32 + 0.5) / n as f32,
                    1. - (y as f32 + 0.5) / n as f32,
                ];
                let c = r.evaluate(uv, 1. / n as f32).color_roughness;
                for k in 0..3 {
                    error += f64::from(live[4 * (y * n + x) + k].abs_diff(srgb(c[k])));
                    count += 1;
                }
            }
        }
        let average = error / f64::from(count);
        println!("{family:?}: CPU/GPU RGB8 mean {average:.4}");
        assert!(average < 1.5, "{family:?} CPU/GPU RGB8 average {average}");
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
        println!("{family:?}: runtime/fixed RGB8 mean {mean:.4}");
        assert!(mean < 5., "{family:?} runtime/fixed RGB8 average {mean}");
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
}
