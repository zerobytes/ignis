//! A warm resident catalog must match a cold GPU upload after rebase, edits and slot reuse.
use ignis::{
    engine::Engine,
    meshes::{Asset, Geometry, Instance, Shading},
    model::{Camera, RoomRecipe, Scene},
    render::Renderer,
};
use std::sync::Arc;

fn asset(id: &str, bend: f32) -> Asset {
    let mut geometry = Geometry::grid(32, 3.).unwrap();
    for p in &mut geometry.positions {
        p[1] = bend * (p[0] * p[0] + p[2] * p[2]);
    }
    Asset {
        id: id.into(),
        geometry: Arc::new(geometry),
        shading: Shading::Smooth,
        material: ignis::materials::Material {
            pattern: Some(ignis::materials::Pattern {
                frequency: [1.; 3],
                low: if id == "a" {
                    [1., 0.5, 0.25]
                } else {
                    [0.25, 1., 0.5]
                },
                high: if id == "a" {
                    [1., 0.5, 0.25]
                } else {
                    [0.25, 1., 0.5]
                },
                bump: 0.,
                warp: 0.,
                octaves: 1,
                seed: 7,
                contrast: 1.,
                kind: Default::default(),
            }),
            ..Default::default()
        },
        rig: None,
        clips: vec![],
    }
}
fn instance(id: &str, x: f32, color: [f32; 3]) -> Instance {
    Instance {
        id: id.into(),
        asset: id.into(),
        position: [x, 0., 0.],
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
    s.meshes.assets = vec![asset("a", 0.04), asset("b", -0.08)];
    s.meshes.instances = vec![
        instance("a", -1.8, [0.8, 0.3, 0.2]),
        instance("b", 1.8, [0.2, 0.7, 0.4]),
    ];
    s.camera = Camera {
        position: [0., 4., 10.],
        target: [0., 0., 0.],
        fov_degrees: 60.,
    };
    s
}
fn uploads(renderer: &Renderer) -> u64 {
    renderer.info()["instancing"]["geometry_uploads"]
        .as_u64()
        .unwrap()
}

#[test]
#[ignore = "requires native GPU"]
fn catalog_changes_preserve_survivors_and_never_reuse_stale_material_or_geometry() {
    let gpu = wgpu::Instance::default();
    let mut warm = pollster::block_on(Renderer::new(&gpu, None)).unwrap();
    let mut cold = pollster::block_on(Renderer::new(&gpu, None)).unwrap();
    for compact in [true, false] {
        warm.set_compact_vertices(compact);
        cold.set_compact_vertices(compact);
        let mut scene = source();
        let mut frame = Engine::new(scene.clone()).unwrap().render_frame();
        let mut previous = uploads(&warm);
        for step in 0..11 {
            match step {
                1 => {
                    let q = glam::Quat::from_rotation_y(0.3);
                    let t = glam::Vec3::new(20., 5., 8.);
                    for i in &mut scene.meshes.instances {
                        i.position = (q * glam::Vec3::from(i.position) + t).to_array();
                        i.rotation = (q * glam::Quat::from_array(i.rotation)).to_array();
                    }
                    scene.camera.position =
                        (q * glam::Vec3::from(scene.camera.position) + t).to_array();
                    scene.camera.target =
                        (q * glam::Vec3::from(scene.camera.target) + t).to_array();
                }
                2 => {
                    scene.meshes.assets.insert(0, asset("new", 0.1));
                    let mut i = scene.meshes.instances[0].clone();
                    i.id = "new".into();
                    i.asset = "new".into();
                    i.position[2] -= 4.;
                    scene.meshes.instances.push(i);
                }
                3 => {
                    scene.meshes.instances.retain(|i| i.asset != "new");
                    scene.meshes.assets.remove(0);
                }
                4 => scene.meshes.assets.swap(0, 1),
                5 => scene.meshes.assets[0].shading = Shading::Flat,
                6 => Arc::make_mut(&mut scene.meshes.assets[0].geometry).positions[100][1] += 0.8,
                7 => {
                    assert_ne!(
                        scene.meshes.assets[0].material,
                        scene.meshes.assets[1].material
                    );
                    scene.meshes.assets[0].material = scene.meshes.assets[1].material.clone();
                    scene.meshes.instances[0].color = [0.1, 0.2, 0.9];
                }
                8 => {
                    let mut alias = scene.meshes.assets[0].clone();
                    alias.id = "alias".into();
                    alias.material.pattern.as_mut().unwrap().low = [0.2, 0.2, 1.];
                    alias.material.pattern.as_mut().unwrap().high = [0.2, 0.2, 1.];
                    let mut i = scene.meshes.instances[1].clone();
                    i.id = "alias".into();
                    i.asset = "alias".into();
                    i.position[2] -= 4.;
                    scene.meshes.assets.push(alias);
                    scene.meshes.instances.push(i);
                }
                9 => {
                    let id = scene.meshes.assets[0].id.clone();
                    scene.meshes.assets[0] = asset(&id, 0.15);
                }
                10 => {
                    let p = scene.meshes.assets[1].material.pattern.as_mut().unwrap();
                    p.low = [0.8, 0.1, 0.8];
                    p.high = p.low;
                }
                _ => {}
            }
            frame.meshes.update(&scene.meshes, 0);
            frame.camera = scene.camera.clone();
            let (w, h) = if step % 2 == 0 {
                (320, 240)
            } else {
                (240, 320)
            };
            let a = warm.capture(&frame, w, h).unwrap();
            // Explicit diagnostic toggles force a cold geometry cache, with final
            // indexing policy unchanged; this is an independent upload oracle.
            cold.set_geometry_indexing(false);
            cold.set_geometry_indexing(true);
            let oracle = Engine::new(scene.clone()).unwrap().render_frame();
            let b = cold.capture(&oracle, w, h).unwrap();
            assert_eq!(
                ignis::image_similarity::Image::decode(&a).unwrap().pixels,
                ignis::image_similarity::Image::decode(&b).unwrap().pixels,
                "compact{compact} step{step}"
            );
            // Every subject must actually cover pixels. A background-only pair
            // cannot certify stale vertex/material/slot behavior.
            let mut background = scene.clone();
            background.meshes.instances.clear();
            let blank = ignis::image_similarity::Image::decode(
                &cold
                    .capture(&Engine::new(background).unwrap().render_frame(), w, h)
                    .unwrap(),
            )
            .unwrap();
            for subject in &scene.meshes.instances {
                let mut only = scene.clone();
                only.meshes.instances.retain(|i| i.id == subject.id);
                cold.set_geometry_indexing(false);
                cold.set_geometry_indexing(true);
                let pixels = ignis::image_similarity::Image::decode(
                    &cold
                        .capture(&Engine::new(only).unwrap().render_frame(), w, h)
                        .unwrap(),
                )
                .unwrap();
                let covered = pixels
                    .pixels
                    .iter()
                    .zip(&blank.pixels)
                    .filter(|(a, b)| a != b)
                    .count();
                assert!(
                    covered > 100,
                    "invisible subject {} compact{compact} step{step}: {covered}",
                    subject.id
                );
            }
            let count = uploads(&warm);
            if compact {
                let expected = match step {
                    0 => 2,
                    2 | 5 | 6 | 9 => 1,
                    _ => 0,
                };
                assert_eq!(
                    count - previous,
                    expected,
                    "incremental uploads at step{step}"
                );
            }
            previous = count;
        }
    }
}
