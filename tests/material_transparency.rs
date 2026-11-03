use ignis::{
    engine::{Command, Engine},
    materials::{BlendMode, Coverage, Glass, Material, Surface},
    meshes::{Asset, Edit, Geometry, Instance, Shading},
    model::{Camera, RoomRecipe, Scene},
    render::Renderer,
};
use std::{io::Cursor, sync::Arc};

fn material(mode: BlendMode, opacity: f32) -> Material {
    Material {
        surface: Some(Surface {
            mode,
            opacity,
            ..Default::default()
        }),
        ..Default::default()
    }
}
fn plane(id: &str, material: Material) -> Asset {
    Asset {
        id: id.into(),
        material,
        shading: Shading::Smooth,
        rig: None,
        clips: vec![],
        geometry: Arc::new(Geometry {
            positions: vec![[-2., -1., 0.], [2., -1., 0.], [2., 3., 0.], [-2., 3., 0.]],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            material_coordinates: None,
        }),
    }
}
fn instance(id: &str, z: f32, color: [f32; 3]) -> Instance {
    Instance {
        id: id.into(),
        asset: id.into(),
        position: [0., 0., z],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color,
        playback: None,
        attachment: None,
    }
}
fn source() -> Scene {
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
        position: [0., 1., 6.],
        target: [0., 1., 0.],
        fov_degrees: 45.,
    };
    s.light.ambient = 1.;
    s.meshes.assets = vec![
        plane("back", Material::default()),
        plane("front", material(BlendMode::Blend, 0.25)),
    ];
    s.meshes.instances = vec![
        instance("back", -1., [0.2, 0.4, 0.7]),
        instance("front", 0., [0.8, 0.1, 0.3]),
    ];
    s
}

#[test]
fn surface_contract_is_atomic_versioned_and_public_content_round_trips() {
    let mut engine = Engine::new(source()).unwrap();
    let original = engine.snapshot();
    for invalid in [
        Surface {
            version: 2,
            ..Default::default()
        },
        Surface {
            opacity: f32::NAN,
            ..Default::default()
        },
        Surface {
            opacity: 0.5,
            ..Default::default()
        },
        Surface {
            mode: BlendMode::Blend,
            glass: Some(Glass {
                ior: 0.9,
                ..Default::default()
            }),
            ..Default::default()
        },
        Surface {
            mode: BlendMode::Blend,
            glass: Some(Glass {
                attenuation_distance_m: 0.,
                ..Default::default()
            }),
            ..Default::default()
        },
        Surface {
            mode: BlendMode::Mask,
            coverage: Some(Coverage {
                high: 1.1,
                ..Default::default()
            }),
            ..Default::default()
        },
    ] {
        assert!(
            engine
                .execute(
                    Command::EditMesh {
                        edit: Edit::SetMaterial {
                            asset: "front".into(),
                            material: Material {
                                surface: Some(invalid),
                                ..Default::default()
                            }
                        }
                    }
                    .into()
                )
                .is_err()
        );
        assert_eq!(engine.snapshot(), original);
    }
    let mut glass = material(BlendMode::Blend, 1.);
    glass.surface.as_mut().unwrap().glass = Some(Glass::default());
    let mut conflicting = glass.clone();
    conflicting.transmission = 0.3;
    assert!(conflicting.validate().is_err());
    engine
        .execute(
            Command::EditMesh {
                edit: Edit::SetMaterial {
                    asset: "front".into(),
                    material: glass.clone(),
                },
            }
            .into(),
        )
        .unwrap();
    assert_eq!(engine.scene.version, ignis::model::SCENE_VERSION);
    let saved = serde_json::to_vec(&engine.scene).unwrap();
    let restored = Engine::new(serde_json::from_slice(&saved).unwrap()).unwrap();
    assert_eq!(restored.scene.meshes.assets[1].material, glass);
    let content = ignis::network::content::Content::capture(&engine, 123);
    let decoded =
        ignis::network::content::Content::decode(123, &content.encode().unwrap()).unwrap();
    assert_eq!(decoded.meshes.assets[1].material, glass);
    let mut forged = restored.scene.clone();
    forged.version = 109;
    assert!(forged.validate().is_err());
    let mut hidden: Scene =
        serde_json::from_str(include_str!("../examples/native-tree-study.scene.json")).unwrap();
    hidden.version = 109;
    Arc::make_mut(&mut hidden.plants.entries[0].source).material = glass;
    assert!(
        hidden
            .validate()
            .unwrap_err()
            .message
            .contains("version110")
    );
    assert!(
        !serde_json::to_string(&Material::default())
            .unwrap()
            .contains("surface")
    );
}

fn rgba(png: &[u8]) -> Vec<u8> {
    let mut reader = png::Decoder::new(Cursor::new(png)).read_info().unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    bytes.truncate(info.buffer_size());
    bytes
}
fn capture(gpu: &mut Renderer, scene: &Scene) -> Vec<u8> {
    rgba(
        &gpu.capture(
            &Engine::new(scene.clone()).unwrap().render_frame(),
            256,
            256,
        )
        .unwrap(),
    )
}
fn pixel(bytes: &[u8]) -> [u8; 3] {
    bytes[(128 * 256 + 128) * 4..(128 * 256 + 128) * 4 + 3]
        .try_into()
        .unwrap()
}
fn srgb(v: f32) -> u8 {
    ((if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1. / 2.4) - 0.055
    })
    .clamp(0., 1.)
        * 255.)
        .round() as u8
}
fn expect_pixel(bytes: &[u8], linear: [f32; 3]) {
    let p = pixel(bytes);
    for i in 0..3 {
        assert!(
            p[i].abs_diff(srgb(linear[i])) <= 2,
            "{p:?} expected {:?}",
            linear.map(srgb)
        );
    }
}
fn close(a: &[u8], b: &[u8], tolerance: u8) {
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(b).all(|(a, b)| a.abs_diff(*b) <= tolerance));
}

#[test]
#[ignore = "requires native GPU"]
fn depth_colored_glass_and_order_independent_composition_have_numeric_oracles() {
    let gpu_instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&gpu_instance, None)).unwrap();
    let mut s = source();
    expect_pixel(&capture(&mut gpu, &s), [0.35, 0.325, 0.6]);
    assert_eq!(gpu.info()["transparency"]["active"], true);
    assert_eq!(gpu.info()["transparency"]["image_bytes"], 256 * 256 * 28);
    s.meshes.instances[1].position[2] = -2.;
    expect_pixel(&capture(&mut gpu, &s), [0.2, 0.4, 0.7]);
    s.meshes.instances[1].position[2] = 0.;
    s.meshes.assets[1].material = material(BlendMode::Blend, 1.);
    s.meshes.instances[1].color = [1.; 3];
    s.meshes.assets[1].material.surface.as_mut().unwrap().glass = Some(Glass {
        ior: 1.,
        ..Default::default()
    });
    let clear = capture(&mut gpu, &s);
    expect_pixel(&clear, [0.2, 0.4, 0.7]);
    s.meshes.instances[1].rotation =
        glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_3).to_array();
    expect_pixel(&capture(&mut gpu, &s), [0.2, 0.4, 0.7]);
    s.meshes.assets[1]
        .material
        .surface
        .as_mut()
        .unwrap()
        .glass
        .as_mut()
        .unwrap()
        .ior = 1.5;
    expect_pixel(&capture(&mut gpu, &s), [0.2 * 0.93, 0.4 * 0.93, 0.7 * 0.93]);
    s.meshes.instances[1].rotation = [0., 0., 0., 1.];
    let g = s.meshes.assets[1]
        .material
        .surface
        .as_mut()
        .unwrap()
        .glass
        .as_mut()
        .unwrap();
    g.ior = 1.5;
    g.thickness_m = 0.5;
    g.attenuation_color = [0.25, 0.64, 1.];
    expect_pixel(
        &capture(&mut gpu, &s),
        [0.2 * 0.96 * 0.5, 0.4 * 0.96 * 0.8, 0.7 * 0.96],
    );
    let g = s.meshes.assets[1]
        .material
        .surface
        .as_mut()
        .unwrap()
        .glass
        .as_mut()
        .unwrap();
    g.attenuation_color = [0., 1., 1.];
    expect_pixel(&capture(&mut gpu, &s), [0., 0.4 * 0.96, 0.7 * 0.96]);
    s.meshes.assets[1]
        .material
        .surface
        .as_mut()
        .unwrap()
        .glass
        .as_mut()
        .unwrap()
        .thickness_m = 0.;
    expect_pixel(&capture(&mut gpu, &s), [0.2 * 0.96, 0.4 * 0.96, 0.7 * 0.96]);
    let g = s.meshes.assets[1]
        .material
        .surface
        .as_mut()
        .unwrap()
        .glass
        .as_mut()
        .unwrap();
    g.attenuation_color = [0.25, 0.64, 1.];
    g.thickness_m = 0.5;
    s.meshes
        .assets
        .push(plane("other", material(BlendMode::Blend, 0.5)));
    s.meshes
        .instances
        .push(instance("other", 0.2, [0.1, 0.8, 0.4]));
    let order = capture(&mut gpu, &s);
    s.meshes.instances.reverse();
    close(&order, &capture(&mut gpu, &s), 1);
    s.meshes.instances.retain(|i| i.id != "other");
    s.meshes.assets[1].material = material(BlendMode::Opaque, 1.);
    s.meshes
        .instances
        .iter_mut()
        .find(|i| i.id == "front")
        .unwrap()
        .color = [0.; 3];
    s.meshes.assets[1]
        .material
        .surface
        .as_mut()
        .unwrap()
        .emissive = [0.12, 0.3, 0.5];
    expect_pixel(&capture(&mut gpu, &s), [0.12, 0.3, 0.5]);
    s.meshes.assets[1].material = material(BlendMode::Additive, 0.5);
    s.meshes
        .instances
        .iter_mut()
        .find(|i| i.id == "front")
        .unwrap()
        .color = [0.; 3];
    s.meshes.assets[1]
        .material
        .surface
        .as_mut()
        .unwrap()
        .emissive = [0.2, 0.1, 0.3];
    expect_pixel(&capture(&mut gpu, &s), [0.3, 0.45, 0.85]);
    s.meshes.assets[1].material = Material::default();
    capture(&mut gpu, &s);
    assert_eq!(gpu.info()["transparency"]["active"], false);
    assert_eq!(gpu.info()["transparency"]["image_bytes"], 0);
}

#[test]
#[ignore = "requires native GPU"]
fn material_edits_coverage_and_legacy_geometry_match_fresh_uploads() {
    let gpu_instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&gpu_instance, None)).unwrap();
    let mut cold = pollster::block_on(Renderer::new(&gpu_instance, None)).unwrap();
    let mut s = source();
    let baseline = capture(&mut gpu, &s);
    for mode in [BlendMode::Mask, BlendMode::Blend, BlendMode::Opaque] {
        s.meshes.assets[1].material = material(mode, 1.);
        if mode != BlendMode::Opaque {
            s.meshes.assets[1]
                .material
                .surface
                .as_mut()
                .unwrap()
                .coverage = Some(Coverage::default());
        }
        let current = capture(&mut gpu, &s);
        close(&current, &capture(&mut cold, &s), 0);
        assert_ne!(current, baseline);
        let mut frame = Engine::new(s.clone()).unwrap().render_frame();
        frame.meshes.set_unique_static_residency(false);
        frame.meshes.update(&s.meshes, 0);
        let legacy = rgba(&gpu.capture(&frame, 256, 256).unwrap());
        close(&current, &legacy, 1);
    }
    s.meshes.assets[1].material = material(BlendMode::Mask, 0.);
    s.light = serde_json::from_value(serde_json::json!({
        "direction":[0,0,-1], "ambient":0.2,
        "shadow":{"center":[0,1,-0.5],"extent":6,"resolution":512,"bias":0.0001}
    }))
    .unwrap();
    let hidden = capture(&mut gpu, &s);
    s.meshes.instances.retain(|i| i.id != "front");
    close(&hidden, &capture(&mut gpu, &s), 0);
}

#[test]
#[ignore = "requires native GPU"]
fn presentation_tails_keep_surface_routing_after_atmospheric_stream_compaction() {
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    for atmospheric in [false, true] {
        let mut s = source();
        if atmospheric {
            s.light.atmosphere = Some(ignis::atmosphere::Atmosphere {
                density: 1.,
                color: [0.4, 0.5, 0.6],
                sun_strength: 0.,
                ..Default::default()
            });
            s.distant_meshes = ignis::distant::compile(&s.meshes).unwrap();
        }
        let mut control = Engine::new(s.clone()).unwrap().render_frame();
        control.meshes.set_unique_static_residency(false);
        control.meshes.update(&s.meshes, 0);
        let reference = rgba(&gpu.capture(&control, 256, 256).unwrap());
        s.meshes.instances.retain(|i| i.id != "front");
        let mut tail = Engine::new(s.clone()).unwrap().render_frame();
        tail.meshes.set_unique_static_residency(false);
        tail.meshes.update(&s.meshes, 0);
        // Public frame vertex tails are the interface used by actor presentation.
        let geometry = &s.meshes.assets[1].geometry;
        for triangle in &geometry.triangles {
            for &v in triangle {
                tail.meshes.vertices.push(ignis::render::Vertex {
                    position: geometry.positions[v as usize],
                    rest_position: geometry.positions[v as usize],
                    normal: [0., 0., 1.],
                    color: [0.8, 0.1, 0.3],
                    material: 2,
                    material_projection_weights: [0., 0., 1.],
                });
            }
        }
        close(&reference, &rgba(&gpu.capture(&tail, 256, 256).unwrap()), 1);
        if atmospheric {
            assert!(
                gpu.info()["distant_selection"]["substituted_instances"]
                    .as_u64()
                    .unwrap()
                    > 0
            );
        }
    }
}
