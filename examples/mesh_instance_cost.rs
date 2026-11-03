//! Native draw+GPU completion, excluding readback/PNG/presentation/simulation.
//! The reference concatenates single-instance legacy frames of the SAME asset.
//! Both variants have identical material tables and draw all authored triangles.
use anyhow::Result;
use ignis::{
    engine::Engine,
    meshes::{Asset, Geometry, Instance, Shading},
    model::RoomRecipe,
    render::Renderer,
};
use serde_json::json;
use std::{sync::Arc, time::Instant};
fn main() -> Result<()> {
    let out = std::path::Path::new("artifacts/mesh-resources");
    std::fs::create_dir_all(out)?;
    let gpu = wgpu::Instance::default();
    let (width, height) = (1728, 1117);
    let mut rows = Vec::new();
    for count in [1000usize, 10000] {
        let mut reference = None;
        for repeat in 0..3 {
            for shared in if repeat % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let mut scene = RoomRecipe {
                    obstacles: 0,
                    ..Default::default()
                }
                .generate()?;
                for e in &mut scene.entities {
                    e.visible = Some(false);
                }
                scene.camera.position = [100., 180., 210.];
                scene.camera.target = [100., 0., 100.];
                scene.camera.fov_degrees = 65.;
                let geometry = Arc::new(Geometry::grid(8, 1.5)?);
                for n in 0..1 {
                    scene.meshes.assets.push(Asset {
                        id: format!("asset-{n}"),
                        geometry: geometry.clone(),
                        shading: Shading::Smooth,
                        material: Default::default(),
                        rig: None,
                        clips: vec![],
                    });
                }
                for n in 0..count {
                    scene.meshes.instances.push(Instance {
                        id: format!("instance-{n}"),
                        asset: "asset-0".into(),
                        position: [
                            (n % 100) as f32 * 2.,
                            (n % 5) as f32 * 0.25,
                            (n / 100) as f32 * 2.,
                        ],
                        rotation: [0., 0., 0., 1.],
                        scale: 1.,
                        color: [0.1 + (n % 7) as f32 * 0.1, 0.4, 0.2],
                        playback: None,
                        attachment: None,
                    });
                }
                let engine = Engine::new(scene)?;
                let mut frame = engine.render_frame();
                if !shared {
                    let mut expanded = ignis::meshes::Frame::default();
                    expanded.materials = frame.meshes.materials.clone();
                    let mut single = ignis::meshes::Document {
                        objects: Default::default(),
                        assets: engine.scene.meshes.assets.clone(),
                        instances: vec![],
                    };
                    let mut piece = ignis::meshes::Frame::default();
                    for instance in &engine.scene.meshes.instances {
                        single.instances.clear();
                        single.instances.push(instance.clone());
                        piece.update(&single, 0);
                        expanded.vertices.extend_from_slice(&piece.vertices);
                    }
                    frame.meshes = expanded;
                }
                let mut renderer = pollster::block_on(Renderer::new(&gpu, None))?;
                let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("instance cost target"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: renderer.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                let png = renderer.capture(&frame, width, height)?;
                let pixels = ignis::image_similarity::Image::decode(&png)?;
                let mut max = 0;
                if shared {
                    let before: &ignis::image_similarity::Image = reference.as_ref().unwrap();
                    max = before
                        .pixels
                        .iter()
                        .zip(&pixels.pixels)
                        .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)))
                        .max()
                        .unwrap();
                    anyhow::ensure!(max <= 2, "shared reference differs by {max}");
                } else {
                    reference = Some(pixels);
                }
                std::fs::write(
                    out.join(format!(
                        "{count}-{}.png",
                        if shared { "shared" } else { "expanded" }
                    )),
                    png,
                )?;
                let mut samples = Vec::new();
                for i in 0..44 {
                    let start = Instant::now();
                    renderer.draw(&frame, &view, width, height)?;
                    renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                    if i >= 12 {
                        samples.push(start.elapsed().as_secs_f64() * 1000.);
                    }
                }
                samples.sort_by(f64::total_cmp);
                let row = json!({"repeat":repeat,"instances":count,"shared":shared,"triangles":frame.meshes.triangle_count(),"resident_vertices":frame.meshes.resident_vertex_count(),"draw_wait_median_ms":samples[16],"draw_wait_p95_ms":samples[30],"max_rgb8":max,"adapter":renderer.info()});
                println!("{}", row);
                rows.push(row);
                std::fs::write(
                    out.join("cost.json"),
                    serde_json::to_vec_pretty(
                        &json!({"width":width,"height":height,"method":"3 alternating rounds, 12 warmup +32 native draw/GPU-completion samples per case, no PNG/presentation/simulation in timing; 128 triangles per instance, no culling or atmosphere", "rows":rows}),
                    )?,
                )?;
            }
        }
    }
    Ok(())
}
