//! Critical integration: a false occlusion loses visible surfaces; unstable
//! compaction changes coplanar priority. Compare real native GPU pixels exactly.
use ignis::{
    engine::Engine,
    meshes::{Asset, Geometry, Instance, Shading},
    model::{Camera, RoomRecipe, Scene},
    render::Renderer,
};
use std::sync::Arc;
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    for e in &mut s.entities {
        e.visible = Some(false);
    }
    let mut wall = Geometry::grid(4, 1.).unwrap();
    // A real opening in an otherwise broad opaque surface. A rectangle/AABB
    // must never become an occluder across this opening.
    wall.triangles = wall
        .triangles
        .chunks_exact(2)
        .enumerate()
        .filter(|(n, _)| !matches!(n, 5 | 6 | 9 | 10))
        .flat_map(|(_, t)| t.iter().copied())
        .collect();
    for (id, g) in [("wall", wall), ("surface", Geometry::grid(63, 1.).unwrap())] {
        s.meshes.assets.push(Asset {
            id: id.into(),
            geometry: Arc::new(g),
            shading: Shading::Flat,
            material: Default::default(),
            rig: None,
            clips: vec![],
        });
    }
    let rotation = glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2).to_array();
    s.meshes.instances.push(Instance {
        id: "occluder".into(),
        asset: "wall".into(),
        position: [0., 2., 4.],
        rotation,
        scale: 8.,
        color: [0.3, 0.4, 0.5],
        playback: None,
        attachment: None,
    });
    for n in 0..256 {
        // Duplicate coplanar pairs of different colors exercise stable order
        // within a compacted batch, including runs longer than32 lanes.
        let cell = n / 2;
        s.meshes.instances.push(Instance {
            id: format!("surface-{n}"),
            asset: "surface".into(),
            position: [
                (cell % 16) as f32 * 0.55 - 4.,
                (cell / 16) as f32 * 0.55,
                0.,
            ],
            rotation,
            scale: 0.5,
            color: if n % 2 == 0 {
                [0.8, 0.1, 0.2]
            } else {
                [0.1, 0.8, 0.2]
            },
            playback: None,
            attachment: None,
        });
    }
    s.camera = Camera {
        position: [0., 2., 12.],
        target: [0., 2., 0.],
        fov_degrees: 60.,
    };
    s
}
#[test]
#[ignore = "requires native GPU"]
fn occlusion_preserves_holes_order_motion_near_plane_resize_and_shadows() {
    let gpu = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&gpu, None)).unwrap();
    let source = scene();
    let mut evidence = vec![];
    for n in 0..13 {
        let mut s = source.clone();
        match n {
            1 => s.meshes.instances[0].position[0] = 8., // reveal on the very next frame
            2 => s.meshes.instances[0].position[2] = -3., // occluder behind the subjects
            3 => s.camera.position = [0., 2., 4.00001],  // near plane / inside bounds
            4 => s.camera.position = [7., 3., 8.],
            5 => {
                s.meshes.instances[15].position = [0., 2., 8.];
            } // moving object in front
            6 => {
                s.camera.fov_degrees = 35.;
            }
            7 => {
                s.light.shadow = Some(ignis::model::DirectionalShadow {
                    center: [0., 2., 0.],
                    extent: 12.,
                    resolution: 512,
                    bias: 0.0005,
                });
            }
            8 => {
                // Large world translation challenges cancellation in projection.
                s.camera.position[0] += 8000.;
                s.camera.target[0] += 8000.;
                for i in &mut s.meshes.instances {
                    i.position[0] += 8000.;
                }
            }
            10 => {
                // The same opaque asset also exercises the instanced occluder path.
                let mut other = s.meshes.instances[0].clone();
                other.id = "offscreen-wall".into();
                other.position[0] = 100.;
                s.meshes.instances.push(other);
            }
            11 | 12 => {
                // A broad transparent/cutout wall must not seed opaque depth.
                s.meshes.assets[0].material.surface = Some(ignis::materials::Surface {
                    mode: if n == 11 {
                        ignis::materials::BlendMode::Blend
                    } else {
                        ignis::materials::BlendMode::Mask
                    },
                    opacity: 0.25,
                    ..Default::default()
                });
            }
            _ => {}
        }
        let engine = Engine::new(s).unwrap();
        let mut frame = engine.render_frame();
        if n == 9 {
            frame.selection_mesh = Some("surface-254".into());
        }
        let (w, h) = if n % 2 == 0 { (640, 480) } else { (480, 640) };
        renderer.set_occlusion_culling(false);
        let a = renderer.capture(&frame, w, h).unwrap();
        renderer.set_occlusion_culling(true);
        let b = renderer.capture(&frame, w, h).unwrap();
        let a = ignis::image_similarity::Image::decode(&a).unwrap();
        let b = ignis::image_similarity::Image::decode(&b).unwrap();
        assert!(a.pixels == b.pixels, "occlusion changed pixels at pose{n}");
        let counts = renderer.inspect_occlusion_gpu().unwrap();
        if n == 0 {
            assert!(
                counts["rejected_instances"].as_u64().unwrap_or(0) > 0,
                "occlusion must actually eliminate work: {counts}, info: {}",
                renderer.info()
            );
        }
        evidence.push(
            serde_json::json!({"pose":n,"counts":counts,"occlusion":renderer.info()["occlusion"]}),
        );
    }
    eprintln!("{}", serde_json::to_string(&evidence).unwrap());
}
