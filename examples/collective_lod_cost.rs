//! Offline collective packet + native renderer A/B. Experimental, never enabled by default.
use anyhow::{Result, ensure};
use glam::Vec3;
use ignis::{
    collective_lod::{self, Pack},
    engine::{Command, Engine},
    image_similarity::Image,
    model::Scene,
    render::Renderer,
};
use serde_json::json;
use std::{fs, sync::Arc, time::Instant};
fn value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|n| args.get(n + 1))
        .map(String::as_str)
}
fn compare(a: &Image, b: &Image) -> serde_json::Value {
    let errors: Vec<_> = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)))
        .collect();
    let mut sorted = errors.clone();
    sorted.sort_unstable();
    json!({"max_rgb8":sorted.last(),"p99_rgb8":sorted[sorted.len()*99/100],"changed_pixels":a.pixels.iter().zip(&b.pixels).filter(|(a,b)|a!=b).count(),"mean_rgb8":errors.iter().map(|&e|e as f64).sum::<f64>()/errors.len() as f64,"rmse_rgb8":(errors.iter().map(|&e|(e as f64).powi(2)).sum::<f64>()/errors.len() as f64).sqrt(),"strict_image_gate":errors.iter().all(|&e|e<=2)})
}
fn focused(image: &Image, diameter: f32) -> Image {
    let half = (diameter * 0.5).ceil() as u32 + 2;
    let (x, y) = (image.width / 2 - half, image.height / 2 - half);
    Image {
        width: half * 2,
        height: half * 2,
        pixels: (y..y + half * 2)
            .flat_map(|row| {
                image.pixels
                    [(row * image.width + x) as usize..(row * image.width + x + half * 2) as usize]
                    .iter()
                    .copied()
            })
            .collect(),
    }
}
fn coverage(a: &Image, b: &Image) -> serde_json::Value {
    let white = |p: &&[u8; 3]| p.iter().all(|&c| c >= 250);
    let source = a.pixels.iter().filter(white).count();
    let actual = b.pixels.iter().filter(white).count();
    let changed = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .filter(|(a, b)| white(a) != white(b))
        .count();
    json!({"source_pixels":source,"collective_pixels":actual,"changed_coverage_pixels":changed,"strict_coverage_gate":changed==0,"nonempty_reference":source>0})
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).expect("scene path");
    let out = value(&args, "--output").unwrap_or("artifacts/collective-lod");
    fs::create_dir_all(out)?;
    let mut scene: Scene = serde_json::from_slice(&fs::read(path)?)?;
    let name = value(&args, "--object")
        .map(String::from)
        .unwrap_or_else(|| {
            scene
                .meshes
                .objects
                .first()
                .expect("explicit objects required")
                .id
                .clone()
        });
    let object = scene
        .meshes
        .objects
        .iter()
        .find(|o| o.id == name)
        .expect("object")
        .clone();
    let packet = value(&args, "--packet").unwrap_or("artifacts/collective-lod/object.igcb");
    if args.iter().any(|a| a == "--compile") {
        let start = Instant::now();
        let compiled = collective_lod::compile(&scene.meshes, &object, 8)?;
        let bytes = compiled.encode();
        fs::write(packet, &bytes)?;
        let compile_ms = start.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let loaded = Pack::decode(&bytes, &scene.meshes)?;
        println!(
            "compiled {compile_ms:.1}ms, {} bytes / {} source triangles",
            bytes.len(),
            loaded.source_triangles
        );
        fs::write(
            format!("{out}/compiler.json"),
            serde_json::to_vec_pretty(
                &json!({"packet":packet,"compile_ms":compile_ms,"verified_load_ms":started.elapsed().as_secs_f64()*1000.,"bytes":bytes.len(),"nodes":loaded.nodes.len(),"parts":loaded.parts.len(),"source_triangles":loaded.source_triangles,"quad_triangles":2,"coverage_is_sampled":false,"image_certificate":false,"runtime_payload":"containing hierarchy 32B/node + exact source triangles/normals 80B/face, threaded traversal per covered pixel; no view atlas or JSON"}),
            )?,
        )?;
    }
    if args.iter().any(|a| a == "--compiler-only") {
        return Ok(());
    }
    let pack = Arc::new(Pack::decode(&fs::read(packet)?, &scene.meshes)?);
    let quality_only = args.iter().any(|a| a == "--quality-only");
    let only_pose = value(&args, "--pose")
        .map(str::parse::<usize>)
        .transpose()?;
    let motion = args.iter().any(|a| a == "--motion");
    let seed_only = args.iter().any(|a| a == "--seed-only");
    let isolated = args.iter().any(|a| a == "--isolated");
    if isolated {
        scene
            .meshes
            .instances
            .retain(|i| object.members.contains(&i.id));
        scene.meshes.objects = Arc::new(vec![object]);
        scene.collision_meshes.clear();
        let factor = value(&args, "--scale-factor")
            .unwrap_or("1")
            .parse::<f32>()?;
        ensure!(factor.is_finite() && factor > 0., "Invalid isolated scale");
        for i in &mut scene.meshes.instances {
            i.position = [0., 50. * factor, 0.];
            i.scale *= factor;
            i.rotation = [0., 0., 0., 1.];
        }
        // Match the shared-geometry path used by the real forest. The second
        // placement is off-camera; it changes neither visible content nor shading.
        let duplicates: Vec<_> = scene
            .meshes
            .instances
            .iter()
            .enumerate()
            .map(|(n, i)| {
                let mut i = i.clone();
                i.id = format!("offscreen_reference_{n}");
                i.position = [10000., 50. * factor, 10000.];
                i
            })
            .collect();
        scene.meshes.instances.extend(duplicates);
        for e in &mut scene.entities {
            e.visible = Some(false);
        }
        scene.light.atmosphere = None;
        scene.light.shadow = None;
        scene.light.environment = None;
        scene.distant_meshes = Default::default();
    }
    let authored = scene.camera.clone();
    let mut e = Engine::new(scene)?;
    if !isolated {
        e.execute(
            Command::SetPlayerView {
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
    let mut frame = e.render_frame();
    let gpu = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&gpu, None))?;
    renderer.set_collective_packet(Some(pack.clone()))?;
    let error = value(&args, "--density").unwrap_or("16").parse::<f32>()?;
    renderer.set_collective_density(error)?;
    let raster_grid = value(&args, "--raster-grid")
        .unwrap_or("0")
        .parse::<u32>()?;
    renderer.set_collective_raster_grid(raster_grid)?;
    let native_depth_order = args.iter().any(|a| a == "--native-depth-order");
    renderer.set_collective_depth_order(native_depth_order)?;
    let max_pixels = value(&args, "--max-pixels").unwrap_or("8").parse::<f32>()?;
    let cost_guard = !args.iter().any(|a| a == "--no-cost-guard");
    renderer.set_collective_selection(max_pixels, cost_guard)?;
    let mut rows = vec![];
    let stationary = frame.camera.clone();
    let poses = if motion {
        8
    } else if isolated {
        18
    } else {
        3
    };
    for pose in 0..poses {
        if only_pose.is_some_and(|p| p != pose) {
            continue;
        }
        if motion {
            frame.camera = stationary.clone();
            let shift = Vec3::new(pose as f32 * 6., (pose as f32 * 0.7).sin() * 3., 0.);
            frame.camera.position = (Vec3::from(stationary.position) + shift).into();
            frame.camera.target = (Vec3::from(stationary.target) + shift).into();
            let mut moved = e.scene.meshes.clone();
            let indexes: std::collections::HashMap<_, _> = moved
                .instances
                .iter()
                .enumerate()
                .map(|(n, i)| (i.id.clone(), n))
                .collect();
            for (n, object) in moved.objects.iter().take(64).enumerate() {
                for member in &object.members {
                    let instance = &mut moved.instances[indexes[member]];
                    instance.position[0] += (pose as f32 * 0.9 + n as f32).sin() * 25.;
                    instance.rotation = glam::Quat::from_rotation_y(pose as f32 * 0.25).to_array();
                }
            }
            frame.meshes.update(&moved, 0);
        } else if isolated {
            let root = &e.scene.meshes.instances[0];
            let center = collective_lod::world(root).transform_point3(pack.center);
            let diameter = [16., 4., 2.][pose / 6];
            let azimuth = (pose % 6) as f32 * 0.29;
            let elevation = 0.17 + (pose % 3) as f32 * 0.11;
            let direction = Vec3::new(
                azimuth.sin() * elevation.cos(),
                elevation.sin(),
                azimuth.cos() * elevation.cos(),
            );
            let radius = pack.radius * root.scale;
            let focal = 384. / (2. * (75f32.to_radians() * 0.5).tan());
            frame.camera.position = (center + direction * (2. * radius * focal / diameter)).into();
            frame.camera.target = center.into();
            frame.camera.fov_degrees = 75.;
        } else if pose > 0 {
            e.execute(Command::SetPlayerView { view: None }.into())?;
            e.update_render_frame(&mut frame);
            let eye = Vec3::from(authored.position);
            let target = Vec3::from(authored.target);
            frame.camera = authored.clone();
            frame.camera.position = (target
                + glam::Quat::from_rotation_y(if pose == 1 { 0. } else { 0.32 })
                    * (eye - target)
                    * if pose == 1 { 0.55 } else { 1. })
            .into();
        }
        frame.light.direction = [
            0.65 - (pose % 6) as f32 * 0.18,
            -1.,
            -0.25 + (pose % 4) as f32 * 0.17,
        ];
        let resolutions = if motion {
            vec![if pose % 2 == 0 {
                (640, 400)
            } else {
                (800, 500)
            }]
        } else if isolated {
            vec![(512, 384)]
        } else {
            vec![(1280, 800), (1728, 1117)]
        };
        for (width, height) in resolutions {
            let mut images = vec![];
            let mut info = vec![];
            for enabled in [false, true] {
                renderer.set_collective_enabled(seed_only || enabled);
                renderer.set_collective_seed_culling(!seed_only || enabled);
                let png = renderer.capture(&frame, width, height)?;
                ensure!(
                    png == renderer.capture(&frame, width, height)?,
                    "Unstable same-state capture"
                );
                fs::write(
                    format!(
                        "{out}/{}-{pose}-{width}-{}.png",
                        if isolated { "isolated" } else { "world" },
                        if enabled { "collective" } else { "previous" }
                    ),
                    &png,
                )?;
                images.push(Image::decode(&png)?);
                info.push(renderer.info());
            }
            let difference = compare(&images[0], &images[1]);
            if seed_only {
                ensure!(
                    difference["max_rgb8"] == 0,
                    "Seed culling changes pixels at pose{pose}"
                );
                ensure!(
                    info[1]["collective_lod"]["seed_depth_active"] == true,
                    "Seed comparison did not exercise depth"
                );
            }
            if motion {
                rows.push(json!({"pose":pose,"physical_pixels":[width,height],"image":difference,"seed_only":seed_only,"collective":info[1],"build_id":env!("IGNIS_BUILD_ID")}));
                fs::write(
                    format!("{out}/motion.json"),
                    serde_json::to_vec_pretty(&rows)?,
                )?;
                continue;
            }
            let mut focus = serde_json::Value::Null;
            let mut mask_quality = serde_json::Value::Null;
            if isolated {
                let diameter = [16., 4., 2.][pose / 6];
                focus = compare(
                    &focused(&images[0], diameter),
                    &focused(&images[1], diameter),
                );
                let saved_light = frame.light.clone();
                let mut mask = e.scene.meshes.clone();
                for i in &mut mask.instances {
                    i.color = [1.; 3];
                }
                frame.meshes.update(&mask, 0);
                frame.light.ambient = 1.;
                let mut masks = vec![];
                for enabled in [false, true] {
                    renderer.set_collective_enabled(seed_only || enabled);
                    renderer.set_collective_seed_culling(!seed_only || enabled);
                    let png = renderer.capture(&frame, width, height)?;
                    fs::write(
                        format!(
                            "{out}/coverage-{pose}-{}.png",
                            if enabled { "collective" } else { "source" }
                        ),
                        &png,
                    )?;
                    masks.push(Image::decode(&png)?);
                }
                mask_quality = coverage(&masks[0], &masks[1]);
                frame.meshes.update(&e.scene.meshes, 0);
                frame.light = saved_light;
            }
            if quality_only {
                rows.push(json!({"pose":pose,"physical_pixels":[width,height],"camera":frame.camera,"light":frame.light,"image":difference,"focused_image":focus,"coverage":mask_quality,"collective":info[1],"raster_grid":raster_grid,"native_depth_order":native_depth_order,"max_pixels":max_pixels,"cost_guard":cost_guard,"build_id":env!("IGNIS_BUILD_ID")}));
                fs::write(
                    format!("{out}/{}.json", if isolated { "isolated" } else { "world" }),
                    serde_json::to_vec_pretty(&rows)?,
                )?;
                continue;
            }
            let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ICB1 timing"),
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
            let mut totals = [vec![], vec![]];
            let mut cpu = [vec![], vec![]];
            for round in 0..8 {
                for index in if round % 2 == 0 { [0, 1] } else { [1, 0] } {
                    renderer.set_collective_enabled(seed_only || index == 1);
                    renderer.set_collective_seed_culling(!seed_only || index == 1);
                    for warm in 0..5 {
                        let start = Instant::now();
                        renderer.draw(&frame, &view, width, height)?;
                        let submit = start.elapsed().as_secs_f64() * 1000.;
                        renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                        if warm > 0 {
                            totals[index].push(start.elapsed().as_secs_f64() * 1000.);
                            cpu[index].push(submit);
                        }
                    }
                }
            }
            for t in &mut totals {
                t.sort_by(f64::total_cmp);
            }
            for t in &mut cpu {
                t.sort_by(f64::total_cmp);
            }
            let row = json!({"build_id":env!("IGNIS_BUILD_ID"),"pose":pose,"physical_pixels":[width,height],"camera":frame.camera,"light":frame.light,"triangles_per_pixel_threshold":error,"image":difference,"focused_image":focus,"coverage":mask_quality,"previous":info[0],"collective":info[1],"median_draw_wait_ms":totals.each_ref().map(|t|t[t.len()/2]),"median_cpu_submit_wall_ms":cpu.each_ref().map(|t|t[t.len()/2]),"samples_per_variant":32,"method":"warm alternating draw+GPU wait, no readback; not FPS/GPU timestamps","image_certificate":false,"seed_only":seed_only,"raster_grid":raster_grid,"native_depth_order":native_depth_order,"max_pixels":max_pixels,"cost_guard":cost_guard});
            println!(
                "pose{pose} {width}px: {} selected {}, image {}",
                row["median_draw_wait_ms"],
                row["collective"]["collective_lod"]["selected_objects"],
                row["image"]
            );
            rows.push(row);
            fs::write(
                format!("{out}/{}.json", if isolated { "isolated" } else { "world" }),
                serde_json::to_vec_pretty(&rows)?,
            )?;
        }
    }
    Ok(())
}
