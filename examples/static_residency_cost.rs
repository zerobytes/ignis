//! Same scene/triangles/materials, old unique-static stream versus resident stream.
//! Native draw + completion; PNG, simulation and cold preparation excluded from timing.
use anyhow::{Result, ensure};
use ignis::{engine::Engine, image_similarity::Image, model::Scene, render::Renderer};
use serde_json::json;
use std::{fs, time::Instant};
// Offline readback at native viewport resolution. The product MCP capture
// envelope remains unchanged. This work stays outside all timed samples.
fn capture(
    renderer: &mut Renderer,
    scene: &ignis::render::RenderFrame,
    width: u32,
    height: u32,
) -> Result<Vec<u8>> {
    anyhow::ensure!(
        (64..=renderer.device.limits().max_texture_dimension_2d).contains(&width)
            && (64..=renderer.device.limits().max_texture_dimension_2d).contains(&height),
        "Offline capture exceeds device texture dimensions"
    );
    anyhow::ensure!(
        renderer.format == wgpu::TextureFormat::Rgba8UnormSrgb,
        "Capture requires RGBA renderer"
    );
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen capture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    renderer.draw(
        scene,
        &texture.create_view(&Default::default()),
        width,
        height,
    )?;
    read_texture_png(renderer, &texture, width, height)
}
fn read_texture_png(
    renderer: &Renderer,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Result<Vec<u8>> {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let padded = (width * 4).div_ceil(256) * 256;
    let buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("capture readback"),
        size: padded as u64 * height as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = renderer.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        size,
    );
    let submission = renderer.queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    renderer.device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(std::time::Duration::from_secs(20)),
    })?;
    rx.recv_timeout(std::time::Duration::from_secs(20))??;
    let mapped = buffer.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in mapped.chunks(padded as usize) {
        pixels.extend_from_slice(&row[..(width * 4) as usize]);
    }
    drop(mapped);
    buffer.unmap();
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&pixels)?;
    }
    Ok(png_bytes)
}
// The product image-comparison envelope is also 2048px. Decode only our own
// offline readbacks here so quality is measured at the actual timed resolution.
fn decode(bytes: &[u8]) -> Result<Image> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info()?;
    let mut buffer = vec![0; reader.output_buffer_size().expect("known capture size")];
    let info = reader.next_frame(&mut buffer)?;
    ensure!(
        info.color_type == png::ColorType::Rgba && info.bit_depth == png::BitDepth::Eight,
        "offline capture requires RGBA8"
    );
    Ok(Image {
        width: info.width,
        height: info.height,
        pixels: buffer[..info.buffer_size()]
            .chunks_exact(4)
            .map(|p| [p[0], p[1], p[2]])
            .collect(),
    })
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).expect("scene path");
    let out = args.get(2).expect("output directory");
    fs::create_dir_all(out)?;
    let mut scene: Scene = serde_json::from_slice(&fs::read(path)?)?;
    for entity in &mut scene.entities {
        entity.visible = Some(false);
    }
    let engine = Engine::new(scene)?;
    let gpu = wgpu::Instance::default();
    let mut variants = Vec::new();
    for enabled in [false, true] {
        let mut frame = engine.render_frame();
        frame.meshes = Default::default();
        let start = Instant::now();
        frame.meshes.set_unique_static_residency(enabled);
        frame.meshes.update(&engine.scene.meshes, 0);
        let prepare_ms = start.elapsed().as_secs_f64() * 1000.;
        let renderer = pollster::block_on(Renderer::new(&gpu, None))?;
        variants.push((frame, renderer, prepare_ms));
    }
    let authored = engine.scene.camera.clone();
    let poses = [
        authored.clone(),
        ignis::model::Camera {
            position: [650., 550., 800.],
            target: [650., 280., -1000.],
            fov_degrees: 65.,
        },
        ignis::model::Camera {
            position: [-2800., 1800., 4600.],
            target: [400., 250., -1200.],
            fov_degrees: 55.,
        },
    ];
    let mut rows = vec![];
    let pose_count = if args.iter().any(|a| a == "--authored-only") {
        1
    } else {
        3
    };
    for (pose, camera) in poses.into_iter().take(pose_count).enumerate() {
        for (width, height) in [(2784, 1560), (1280, 800)] {
            let mut reference = None;
            let mut image_diff = json!(null);
            for (variant, (frame, renderer, _)) in variants.iter_mut().enumerate() {
                frame.camera = camera.clone();
                frame.light.direction =
                    [0.65 - pose as f32 * 0.21, -1., -0.25 + pose as f32 * 0.17];
                let png = capture(renderer, frame, width, height)?;
                ensure!(
                    png == capture(renderer, frame, width, height)?,
                    "unstable capture"
                );
                fs::write(format!("{out}/{pose}-{width}-{variant}.png"), &png)?;
                let image = decode(&png)?;
                if let Some(before) = &reference {
                    let before: &Image = before;
                    let max = before
                        .pixels
                        .iter()
                        .flatten()
                        .zip(image.pixels.iter().flatten())
                        .map(|(a, b)| a.abs_diff(*b))
                        .max()
                        .unwrap();
                    let changed = before
                        .pixels
                        .iter()
                        .zip(&image.pixels)
                        .filter(|(a, b)| a != b)
                        .count();
                    image_diff =
                        json!({"max_rgb8":max,"changed_pixels":changed,"quality_gate":max<=2});
                } else {
                    reference = Some(image);
                }
            }
            for moving in [false, true] {
                for repeat in 0..3 {
                    for variant in if repeat % 2 == 0 { [0, 1] } else { [1, 0] } {
                        let (frame, renderer, prepare_ms) = &mut variants[variant];
                        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("static residency cost"),
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
                        let start = Instant::now();
                        renderer.draw(frame, &view, width, height)?;
                        renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                        let transition_ms = start.elapsed().as_secs_f64() * 1000.;
                        let mut samples = vec![];
                        for n in 0..28 {
                            frame.camera = camera.clone();
                            if moving {
                                let shift = (n as f32 * 0.35).sin() * 15.;
                                frame.camera.position[0] += shift;
                                frame.camera.target[0] += shift;
                            }
                            let start = Instant::now();
                            renderer.draw(frame, &view, width, height)?;
                            renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                            if n >= 8 {
                                samples.push(start.elapsed().as_secs_f64() * 1000.);
                            }
                        }
                        samples.sort_by(f64::total_cmp);
                        let row = json!({"pose":pose,"pixels":[width,height],"moving":moving,"repeat":repeat,"resident":variant==1,"median_ms":(samples[9]+samples[10])/2.,"p95_ms":samples[18],"transition_draw_wait_ms":transition_ms,"prepare_frame_ms":prepare_ms,"nominal_triangles":frame.meshes.triangle_count(),"legacy_vertices":frame.meshes.vertices.len(),"image":image_diff,"renderer":renderer.info()});
                        println!(
                            "pose{pose}/{width}/moving{moving}/{repeat}/resident{} {:.3}ms image{}",
                            variant == 1,
                            row["median_ms"],
                            image_diff
                        );
                        rows.push(row);
                        fs::write(
                            format!("{out}/cost.json"),
                            serde_json::to_vec_pretty(
                                &json!({"build_id":env!("IGNIS_BUILD_ID"),"scene":path,"method":"3 alternating repeats, 8 warmup +20 samples of native draw+GPU completion; cold frame preparation and transition separate; no PNG/readback/simulation/presentation in timed draws; identical camera motion for both paths","rows":rows}),
                            )?,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}
