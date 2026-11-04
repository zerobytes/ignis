use ignis::{
    engine::{Command, Engine, SceneEdit},
    model::{DIRECT_COLOR_SCENE_VERSION, Light, RoomRecipe},
    network::content::Content,
};

fn engine() -> Engine {
    Engine::new(RoomRecipe::default().generate().unwrap()).unwrap()
}

#[test]
fn legacy_white_roundtrips_and_nonwhite_cannot_hide_in_an_old_scene() {
    let e = engine();
    let json = serde_json::to_value(&e.scene.light).unwrap();
    assert!(json.get("direct_color").is_none());
    let legacy: Light = serde_json::from_value(json).unwrap();
    assert_eq!(legacy.direct_color, [1.; 3]);
    let mut scene = e.scene.clone();
    scene.version = DIRECT_COLOR_SCENE_VERSION - 1;
    scene.validate().unwrap();
    scene.light.direct_color = [1., 0.8, 0.5];
    assert!(scene.validate().unwrap_err().message.contains("version120"));
    scene.version = DIRECT_COLOR_SCENE_VERSION;
    scene.validate().unwrap();
    let json = serde_json::to_value(&scene).unwrap();
    let restored: ignis::model::Scene = serde_json::from_value(json).unwrap();
    assert_eq!(restored, scene);
}

#[test]
fn invalid_linear_color_is_atomic_for_preview_and_commit() {
    let mut e = engine();
    e.execute(Command::Step { ticks: 3 }.into()).unwrap();
    let scene = e.scene.clone();
    let before = e.snapshot();
    for channel in 0..3 {
        for value in [-0.01, 1.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut light = e.scene.light.clone();
            light.direct_color[channel] = value;
            for command in [
                Command::PreviewSceneEdits {
                    edits: vec![SceneEdit::SetLight {
                        light: light.clone(),
                    }],
                },
                Command::SetLight {
                    light: light.clone(),
                },
            ] {
                assert!(e.execute(command.into()).is_err());
                assert_eq!(e.scene, scene);
                assert_eq!(e.snapshot(), before);
            }
        }
    }
    for color in [[0.; 3], [1.; 3], [1., 0.75, 0.3]] {
        let mut light = e.scene.light.clone();
        light.direct_color = color;
        light.validate().unwrap();
    }
}

#[test]
fn set_light_upgrades_atomically_and_undo_redo_replay_preserve_color() {
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.version = DIRECT_COLOR_SCENE_VERSION - 1;
    let mut e = Engine::new(scene).unwrap();
    // Admission upgrades legacy documents before preview/history begin.
    assert_eq!(e.scene.version, DIRECT_COLOR_SCENE_VERSION);
    let scene = e.scene.clone();
    let mut light = scene.light.clone();
    light.direct_color = [1., 0.8, 0.5];
    light.direct_intensity = 0.7;
    let preview = e
        .execute(
            Command::PreviewSceneEdits {
                edits: vec![SceneEdit::SetLight {
                    light: light.clone(),
                }],
            }
            .into(),
        )
        .unwrap();
    assert_eq!(e.scene, scene);
    assert_eq!(preview["scene"]["version"], DIRECT_COLOR_SCENE_VERSION);
    e.execute(Command::SetLight { light }.into()).unwrap();
    let published = e.scene.clone();
    assert_eq!(preview["scene"], serde_json::to_value(&published).unwrap());
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    assert_eq!(
        Engine::replay_at(&e.replay(), None).unwrap().scene,
        published
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, scene);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, published);
}

#[test]
fn public_content_and_binary_light_preserve_linear_channels_and_reject_old_content() {
    let mut e = engine();
    e.scene.light.direct_color = [0.125, 0.5, 1.];
    e.scene.light.direct_intensity = 2.;
    let content = Content::capture(&e, 31);
    let restored = Content::decode(31, &content.encode().unwrap()).unwrap();
    assert_eq!(restored.light, e.scene.light);
    let mut old = serde_json::to_value(&content).unwrap();
    old["version"] = (ignis::network::content::VERSION - 1).into();
    assert!(Content::decode(31, &serde_json::to_vec(&old).unwrap()).is_err());
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&e.scene.light, &mut bytes).unwrap();
    let binary: Light = ciborium::de::from_reader(bytes.as_slice()).unwrap();
    assert_eq!(binary, e.scene.light);
}

mod gpu {
    use super::*;
    use ignis::{
        materials::{BlendMode, Glass, Material, Reflectance, Surface},
        meshes::{Asset, Geometry, Instance, Shading},
        model::{Camera, Scene},
        render::Renderer,
    };
    use std::{io::Cursor, sync::Arc};

    fn source() -> Scene {
        let mut s = engine().scene;
        for e in &mut s.entities {
            e.visible = Some(false);
        }
        s.camera = Camera {
            position: [0., 1., 6.],
            target: [0., 1., 0.],
            fov_degrees: 45.,
        };
        s.light.direction = [0., 0., -1.];
        s.light.ambient = 0.;
        s.light.shadow = None;
        s.light.environment = None;
        s.light.atmosphere = None;
        s.meshes.assets.push(Asset {
            id: "plane".into(),
            geometry: Arc::new(Geometry {
                positions: vec![[-2., -1., 0.], [2., -1., 0.], [2., 3., 0.], [-2., 3., 0.]],
                triangles: vec![[0, 1, 2], [0, 2, 3]],
                material_coordinates: None,
            }),
            shading: Shading::Smooth,
            material: Material::default(),
            rig: None,
            clips: vec![],
        });
        s.meshes.instances.push(Instance {
            id: "plane".into(),
            asset: "plane".into(),
            position: [0.; 3],
            rotation: [0., 0., 0., 1.],
            scale: 1.,
            color: [0.25; 3],
            playback: None,
            attachment: None,
        });
        s
    }
    fn capture(gpu: &mut Renderer, s: &Scene) -> Vec<u8> {
        let png = gpu
            .capture(&Engine::new(s.clone()).unwrap().render_frame(), 128, 128)
            .unwrap();
        let mut reader = png::Decoder::new(Cursor::new(png)).read_info().unwrap();
        let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut bytes).unwrap();
        bytes.truncate(frame.buffer_size());
        bytes
    }
    fn center(bytes: &[u8]) -> [u8; 3] {
        bytes[(64 * 128 + 64) * 4..(64 * 128 + 64) * 4 + 3]
            .try_into()
            .unwrap()
    }
    fn srgb(v: f32) -> u8 {
        ((if v <= 0.0031308 {
            v * 12.92
        } else {
            1.055 * v.powf(1. / 2.4) - 0.055
        }) * 255.)
            .round() as u8
    }

    #[test]
    #[ignore = "requires native GPU"]
    fn direct_rgb_has_a_numeric_oracle_without_applying_intensity_twice() {
        let instance = wgpu::Instance::default();
        let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
        let mut s = source();
        s.light.direct_color = [1., 0.5, 0.25];
        s.light.direct_intensity = 2.;
        let actual = center(&capture(&mut gpu, &s));
        for (actual, expected) in actual.into_iter().zip([0.5, 0.25, 0.125]) {
            assert!(actual.abs_diff(srgb(expected)) <= 2, "{actual}, {expected}");
        }
        // Pure ambient/IBL/emission are independent of direct color.
        s.light.ambient = 1.;
        s.light.direct_color = [1.; 3];
        s.meshes.assets[0].material.surface = Some(Surface {
            emissive: [0.2, 0.1, 0.3],
            ..Default::default()
        });
        s.light.environment = Some(ignis::environment::Environment {
            sky: [0.5, 0.7, 1.],
            horizon: [0.3; 3],
            ground: [0.1; 3],
            axis: [0., 1., 0.],
            intensity: 0.5,
        });
        let white = capture(&mut gpu, &s);
        s.light.direct_color = [0., 1., 0.];
        assert_eq!(white, capture(&mut gpu, &s));
    }

    #[test]
    #[ignore = "requires native GPU"]
    fn dielectric_metal_glass_and_transmission_receive_direct_color_only() {
        let instance = wgpu::Instance::default();
        let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
        for material in [
            Material::default(),
            Material {
                reflectance: Some(Reflectance {
                    roughness: 0.5,
                    f0: 0.04,
                    metallic: 0.,
                }),
                ..Default::default()
            },
            Material {
                reflectance: Some(Reflectance {
                    roughness: 0.5,
                    f0: 0.04,
                    metallic: 1.,
                }),
                ..Default::default()
            },
            Material {
                surface: Some(Surface {
                    mode: BlendMode::Blend,
                    glass: Some(Glass {
                        roughness: 0.5,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Material {
                transmission: 0.7,
                ..Default::default()
            },
        ] {
            let mut s = source();
            s.meshes.assets[0].material = material;
            s.light.direct_intensity = 0.7;
            s.light.direct_color = [0.; 3];
            let dark = center(&capture(&mut gpu, &s));
            s.light.direct_color = [1., 0., 0.];
            let red = center(&capture(&mut gpu, &s));
            assert!(red[0] > dark[0] + 3, "{red:?} vs {dark:?}");
            assert_eq!(&red[1..], &dark[1..]);
            s.light.direct_color = [1.; 3];
            let white = center(&capture(&mut gpu, &s));
            assert_eq!(white[0], red[0]);
            assert!(white[1] > dark[1] + 3 && white[2] > dark[2] + 3);
        }
    }

    #[test]
    #[ignore = "requires native GPU"]
    fn diagnostic_field_consumes_the_same_directional_color_uniform() {
        use ignis::surface_field::{Level, Pack, SIDE, SampleVertex};
        let instance = wgpu::Instance::default();
        let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
        let mut s = source();
        s.meshes.assets.clear();
        s.meshes.instances.clear();
        let field = Pack {
            origin: [-2., -2.],
            spacing: 0.25,
            tiles: [1, 1],
            heights: vec![0.; (SIDE * SIDE) as usize],
            color: [0.25; 3],
            continuous: None,
            levels: std::array::from_fn(|_| Level {
                stride: 1,
                vertices: [[-0.1, 0., 0., 0.], [0.1, 0., 0., 0.], [0., 1., 0., 0.]]
                    .map(|position| SampleVertex {
                        position,
                        normal: [0., 0., 1., 0.],
                    })
                    .to_vec(),
            }),
        };
        gpu.set_surface_field(Some(Arc::new(field))).unwrap();
        s.light.direct_color = [0.; 3];
        let black = capture(&mut gpu, &s);
        s.light.direct_color = [1., 0., 0.];
        let red = capture(&mut gpu, &s);
        let mut changed = 0;
        for (a, b) in black.chunks_exact(4).zip(red.chunks_exact(4)) {
            assert_eq!(&a[1..], &b[1..]);
            changed += usize::from(b[0] > a[0]);
        }
        assert!(
            changed > 100,
            "Field color did not change visible blade pixels"
        );
    }
}
