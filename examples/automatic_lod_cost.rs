//! Prior renderer versus itself + optional IGL1. Draw+wait is not GPU timestamps.
use anyhow::Result;
use ignis::{engine::Engine, mesh_lods, model::Scene, render::Renderer};
use serde_json::json;
use std::{fs, sync::Arc, time::Instant};
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("scene path");
    let fine = std::env::args().any(|a| a == "--fine-occlusion");
    let adaptive = std::env::args().any(|a| a == "--adaptive-occluders");
    let objects = std::env::args().any(|a| a == "--object-culling");
    let visibility_experiment = adaptive || fine || objects;
    let out = if objects {
        "artifacts/object-culling"
    } else if fine {
        "artifacts/fine-occlusion"
    } else if adaptive {
        "artifacts/adaptive-occluders"
    } else if std::env::args().any(|a| a == "--regional-lod") {
        "artifacts/regional-lod"
    } else {
        "artifacts/automatic-lod"
    };
    fs::create_dir_all(out)?;
    let scene: Scene = serde_json::from_slice(&fs::read(path)?)?;
    let started = Instant::now();
    let compiled = if objects {
        mesh_lods::Document::default()
    } else {
        mesh_lods::compile(&scene.meshes)?
    };
    let compile_ms = started.elapsed().as_secs_f64() * 1000.;
    let bytes = compiled.encode();
    let started = Instant::now();
    let loaded = mesh_lods::Document::decode(&bytes, &scene.meshes)?;
    let load_ms = started.elapsed().as_secs_f64() * 1000.;
    if !compiled.packs.is_empty() {
        fs::write(format!("{out}/compiled.ignis-lod"), &bytes)?;
    } else {
        let p = format!("{out}/compiled.ignis-lod");
        if std::path::Path::new(&p).exists() {
            fs::remove_file(p)?;
        }
    }
    let inspection = loaded.inspect();
    println!("compile {compile_ms:.1}ms, verify {load_ms:.1}ms: {inspection}");
    fs::write(
        format!("{out}/compiler.json"),
        serde_json::to_vec_pretty(
            &json!({"compile_ms":compile_ms,"verified_load_ms":load_ms,"sidecar":inspection}),
        )?,
    )?;
    if std::env::args().any(|a| a == "--compiler-only") {
        return Ok(());
    }
    let authored_camera = scene.camera.clone();
    let world_view = std::env::args().any(|a| a == "--world-player-view");
    let mut engine = Engine::new(scene)?;
    if world_view {
        engine.execute(
            ignis::engine::Command::SetPlayerView {
                view: Some(ignis::player_view::Config {
                    actor: "player".into(),
                    yaw_degrees: 0.,
                    pitch_degrees: 0.,
                    eye_height: 1.65,
                    fov_degrees: 75.,
                    mode: Default::default(),
                    third_person: Default::default(),
                }),
            }
            .into(),
        )?;
    }
    let mut frame = engine.render_frame();
    let compiled = Arc::new(loaded);
    frame.mesh_lods = compiled.clone();
    let gpu = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&gpu, None))?;
    let position = glam::Vec3::from(authored_camera.position);
    let target = glam::Vec3::from(authored_camera.target);
    let mut rows = vec![];
    for pose in (0..3).filter(|_| !std::env::args().any(|a| a == "--motion-only")) {
        let angle = (pose as f32 - 1.) * 0.32;
        let delta = glam::Quat::from_rotation_y(angle)
            * (position - target)
            * if pose == 1 { 0.55 } else { 1.0 };
        if pose == 1 && world_view {
            engine.execute(ignis::engine::Command::SetPlayerView { view: None }.into())?;
            engine.update_render_frame(&mut frame);
            frame.mesh_lods = compiled.clone();
        }
        if !(pose == 0 && world_view) {
            frame.camera = authored_camera.clone();
            frame.camera.position = (target + delta).into();
            frame.camera.target = target.into();
        }
        frame.light.direction = [0.65 - pose as f32 * 0.3, -1., -0.25 + pose as f32 * 0.2];
        for (width, height) in [(1280, 800), (1728, 1117)] {
            let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("LOD comparison"),
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
            let mut images = vec![];
            let mut info = vec![];
            let mut costs = [Vec::new(), Vec::new()];
            let mut cpu = [Vec::new(), Vec::new()];
            let mut wait = [Vec::new(), Vec::new()];
            for enabled in [false, true] {
                frame.mesh_lod_settings.enabled = enabled && !visibility_experiment;
                if objects {
                    renderer.set_object_culling(enabled);
                } else if fine {
                    renderer.set_fine_occlusion(enabled);
                } else if adaptive {
                    renderer.set_adaptive_occluders(enabled);
                }
                let png = renderer.capture(&frame, width, height)?;
                anyhow::ensure!(
                    png == renderer.capture(&frame, width, height)?,
                    "unstable capture {pose}/{enabled}"
                );
                if width == 1280 {
                    fs::write(
                        format!(
                            "{out}/pose{pose}-{}.png",
                            if enabled { "lod" } else { "previous" }
                        ),
                        &png,
                    )?;
                }
                images.push(ignis::image_similarity::Image::decode(&png)?);
                let mut snapshot = renderer.info();
                snapshot["gpu_survivors"] = renderer.inspect_occlusion_gpu()?;
                info.push(snapshot);
                for _ in 0..4 {
                    renderer.draw(&frame, &view, width, height)?;
                    renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                }
            }
            // Alternating blocks after warming both paths; no PNG/readback timed.
            let rounds = if objects { 8 } else { 4 };
            for round in 0..rounds {
                for index in if round % 2 == 0 { [0, 1] } else { [1, 0] } {
                    frame.mesh_lod_settings.enabled = index == 1 && !visibility_experiment;
                    if objects {
                        renderer.set_object_culling(index == 1);
                    } else if fine {
                        renderer.set_fine_occlusion(index == 1);
                    } else if adaptive {
                        renderer.set_adaptive_occluders(index == 1);
                    }
                    for _ in 0..4 {
                        let start = Instant::now();
                        renderer.draw(&frame, &view, width, height)?;
                        let submission_ms = start.elapsed().as_secs_f64() * 1000.;
                        renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                        let total = start.elapsed().as_secs_f64() * 1000.;
                        costs[index].push(total);
                        cpu[index].push(submission_ms);
                        wait[index].push(total - submission_ms);
                    }
                }
            }
            let errors: Vec<_> = images[0]
                .pixels
                .iter()
                .flatten()
                .zip(images[1].pixels.iter().flatten())
                .map(|(a, b)| a.abs_diff(*b))
                .collect();
            for c in &mut costs {
                c.sort_by(f64::total_cmp);
            }
            let median = costs.each_ref().map(|c| c[c.len() / 2]);
            for c in &mut cpu {
                c.sort_by(f64::total_cmp);
            }
            for c in &mut wait {
                c.sort_by(f64::total_cmp);
            }
            let cpu_median = cpu.each_ref().map(|c| c[c.len() / 2]);
            let wait_median = wait.each_ref().map(|c| c[c.len() / 2]);
            let row = json!({"experiment":if objects{"object_culling"}else if fine{"fine_occlusion"}else if adaptive{"adaptive_occluders"}else{"automatic_lod"},"build_id":env!("IGNIS_BUILD_ID"),"camera":frame.camera,"light":frame.light,"samples_per_variant":rounds*4,"draw_wait_samples_ms":costs,"cpu_submit_samples_ms":cpu,"pose":pose,"physical_width":width,"physical_height":height,"previous":info[0],"with_lod":info[1],"median_draw_wait_ms":median,"median_cpu_submit_wall_ms":cpu_median,"median_residual_gpu_wait_ms":wait_median,"speedup":median[0]/median[1],"max_rgb8":errors.iter().max(),"mean_rgb8":errors.iter().map(|&e|e as u64).sum::<u64>() as f64/errors.len() as f64,"method":"warm alternating draw+GPU wait; no readback; not GPU timestamps or gameplay FPS"});
            println!(
                "pose{pose} {width}x{height}: {}",
                row["median_draw_wait_ms"]
            );
            rows.push(row);
            if visibility_experiment {
                anyhow::ensure!(
                    errors.iter().all(|&e| e == 0),
                    "Adaptive occlusion changed pixels"
                );
            }
            fs::write(
                format!("{out}/comparison.json"),
                serde_json::to_vec_pretty(
                    &json!({"compiler":inspection,"rows":rows,"image_certificate":false}),
                )?,
            )?;
        }
    }
    if visibility_experiment && world_view {
        // Integration oracle: move an opaque occluder, small mesh targets, camera
        // and light. Compare against GPU occlusion OFF at the SAME instant.
        // This is render-only motion; it does not claim moving terrain physics.
        engine.execute(ignis::engine::Command::SetPlayerView { view: None }.into())?;
        engine.update_render_frame(&mut frame);
        let base = engine.scene.meshes.clone();
        let large = base
            .assets
            .iter()
            .find(|a| a.geometry.triangles.len() > 262_144)
            .map(|a| a.id.clone());
        let mut motion = vec![];
        for step in 0..8 {
            let mut doc = base.clone();
            let mut moved_large = false;
            let mut moved = 0;
            for i in &mut doc.instances {
                if !moved_large && Some(&i.asset) == large.as_ref() {
                    i.position[0] += step as f32 * 12.;
                    moved_large = true;
                } else if moved < 16
                    && doc.assets.iter().any(|a| {
                        a.id == i.asset
                            && a.rig.is_none()
                            && (16..=32768).contains(&a.geometry.triangles.len())
                    })
                {
                    i.position[0] += (step as f32 - 3.) * 25.;
                    i.position[2] += (step as f32 * 0.4).sin() * 40.;
                    moved += 1;
                }
            }
            frame.meshes.update(&doc, 0);
            frame.camera = authored_camera.clone();
            frame.camera.position = (target
                + glam::Quat::from_rotation_y(step as f32 * 0.04)
                    * (position - target)
                    * (0.55 + step as f32 * 0.035))
                .into();
            if step == 0 {
                let actor = engine
                    .scene
                    .entities
                    .iter()
                    .find(|e| e.id == "player")
                    .expect("world player");
                let p = actor.transform.position;
                let y = p[1] - actor.transform.size[1] * 0.5 - 0.2;
                frame.camera.position = [p[0], y, p[2]];
                frame.camera.target = [p[0], y, p[2] - 50.];
            }
            frame.light.direction = [0.65 - step as f32 * 0.06, -1., -0.25 + step as f32 * 0.04];
            frame.mesh_lod_settings.enabled = false;
            // Measure repeatability of the unculled reference, before AND after
            // the optimized draw. A one-LSB reference fluctuation is recorded,
            // not relabeled as exact equality or an image-error certificate.
            let mut references = vec![];
            if objects {
                renderer.set_object_culling(false);
            } else {
                renderer.set_occlusion_culling(false);
            }
            for _ in 0..3 {
                references.push(renderer.capture(&frame, 1280, 800)?);
            }
            renderer.set_object_culling(true);
            renderer.set_occlusion_culling(true);
            renderer.set_adaptive_occluders(true);
            renderer.set_fine_occlusion(true);
            let optimized = renderer.capture(&frame, 1280, 800)?;
            let gpu_info = renderer.inspect_occlusion_gpu()?;
            if objects {
                renderer.set_object_culling(false);
            } else {
                renderer.set_occlusion_culling(false);
            }
            for _ in 0..3 {
                references.push(renderer.capture(&frame, 1280, 800)?);
            }
            let decoded: Vec<_> = references
                .iter()
                .map(|p| ignis::image_similarity::Image::decode(p))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let actual = ignis::image_similarity::Image::decode(&optimized)?;
            let mut unstable = 0;
            let mut max_range = 0;
            let mut max_error = 0;
            let mut outside = 0;
            for (n, pixel) in actual.pixels.iter().enumerate() {
                let mut varies = false;
                let mut differs = false;
                for (channel, &value) in pixel.iter().enumerate() {
                    let lo = decoded.iter().map(|i| i.pixels[n][channel]).min().unwrap();
                    let hi = decoded.iter().map(|i| i.pixels[n][channel]).max().unwrap();
                    varies |= lo != hi;
                    max_range = max_range.max(hi - lo);
                    max_error = max_error.max(value.abs_diff(decoded[0].pixels[n][channel]));
                    differs |= value < lo || value > hi;
                }
                unstable += usize::from(varies);
                outside += usize::from(differs);
            }
            if step == 0 || step == 7 || outside > 0 {
                fs::write(format!("{out}/motion{step}-reference.png"), &references[0])?;
                fs::write(
                    format!("{out}/motion{step}-reference-repeat.png"),
                    &references[5],
                )?;
                fs::write(format!("{out}/motion{step}-optimized.png"), optimized)?;
            }
            motion.push(json!({"frame":step,"near_surface":step==0,"camera":frame.camera,"moving_occluder":moved_large,"moving_instances":moved,"pixel_exact":max_error==0,"max_rgb8_vs_first_reference":max_error,"reference_unstable_pixels":unstable,"reference_max_rgb8_range":max_range,"optimized_pixels_outside_reference_range":outside,"gpu":gpu_info}));
            anyhow::ensure!(
                outside == 0 && max_range <= 1,
                "Moving frame{step}: {outside} pixels outside repeated reference range (reference range {max_range})"
            );
            fs::write(
                format!("{out}/motion.json"),
                serde_json::to_vec_pretty(
                    &json!({"build_id":env!("IGNIS_BUILD_ID"),"physical_pixels":[1280,800],"render_only_motion":true,"frames":motion}),
                )?,
            )?;
        }
        println!(
            "8 moving frames: no differences outside measured reference repeatability; see motion.json"
        );
    }
    Ok(())
}
