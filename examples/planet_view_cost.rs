//! Same-view offscreen cost and lossless payload gates; excludes presentation/simulation.
use anyhow::{Result, ensure};
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::Scene,
    planet::{orbit::Settings, runtime::Config},
    player_view,
    render::{RenderFrame, Renderer},
};
use serde_json::json;
use std::{path::Path, time::Instant};
fn image(png: &[u8]) -> Vec<[u8; 3]> {
    ignis::image_similarity::Image::decode(png).unwrap().pixels
}
fn main() -> Result<()> {
    let out = Path::new("artifacts/planet-performance");
    std::fs::create_dir_all(out)?;
    let scene: Scene = serde_json::from_slice(&std::fs::read(
        "artifacts/planet-runtime/player.scene.json",
    )?)?;
    let config: Config =
        serde_json::from_slice(&std::fs::read("artifacts/planet-runtime/world.json")?)?;
    let mut engine = Engine::new_with_mode(scene, ExecutionMode::Continuous)?;
    engine.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(config.clone())),
        }
        .into(),
    )?;
    engine.advance(120)?;
    engine.execute(
        Command::SetPlayerView {
            view: Some(player_view::Config {
                actor: "player".into(),
                yaw_degrees: 0.,
                pitch_degrees: 0.,
                eye_height: 1.65,
                fov_degrees: 60.,
                mode: Default::default(),
                third_person: Default::default(),
            }),
        }
        .into(),
    )?;
    let instance = wgpu::Instance::default();
    let mut r = pollster::block_on(Renderer::new(&instance, None))?;
    let (width, height) = (1280, 800);
    let texture = r.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("planet cost"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: r.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target = texture.create_view(&Default::default());
    let only_total = std::env::args().any(|a| a == "--total");
    let report_name = if only_total {
        "totals.json"
    } else {
        "current.json"
    };
    let mut rows = vec![];
    let mut gates = vec![];
    for (case, distance, yaw, depth) in [
        ("surface", 0., 0., Some(0.3)),
        ("west", 0., 270., Some(0.3)),
        ("high", 100000., 0., Some(0.3)),
        (
            "globe",
            config.document.diameter_meters * 1.5,
            0.,
            Some(0.3),
        ),
        ("inherited", 0., 0., None),
    ] {
        let grid = 32;
        {
            engine.execute(
                Command::SetPlanetView {
                    view: Some(Settings {
                        distance_meters: distance,
                        yaw_degrees: yaw,
                        tile_subdivisions: grid,
                        atmosphere_optical_depth: depth,
                        atmosphere_optimization: false,
                        render_group_size: 1,
                        ..Default::default()
                    }),
                }
                .into(),
            )?;
            let frame: RenderFrame = engine.render_frame();
            r.set_geometry_indexing(false);
            r.set_compact_vertices(false);
            let reference = r.capture(&frame, width, height)?;
            let expected = image(&reference);
            std::fs::write(out.join(format!("{case}-{grid}-reference.png")), reference)?;
            for round in 0..if only_total { 0 } else { 4 } {
                for enabled in if round % 2 == 0 {
                    [false, true]
                } else {
                    [true, false]
                } {
                    r.set_geometry_indexing(enabled);
                    r.set_compact_vertices(enabled);
                    let mut times = vec![];
                    let mut transition = 0.;
                    for i in 0..36 {
                        let start = Instant::now();
                        r.draw(&frame, &target, width, height)?;
                        r.device.poll(wgpu::PollType::wait_indefinitely())?;
                        let elapsed = start.elapsed().as_secs_f64() * 1000.;
                        if i == 0 {
                            transition = elapsed;
                        }
                        if i >= 12 {
                            times.push(elapsed);
                        }
                    }
                    times.sort_by(f64::total_cmp);
                    rows.push(json!({"phase":"payload","case":case,"grid":grid,"round":round,"optimized":enabled,"transition_ms":transition,"median_ms":times[12],"p95_ms":times[22],"stats":r.info(),"world":engine.inspect_planet_view()}));
                }
            }
            for enabled in [false, true] {
                r.set_geometry_indexing(enabled);
                r.set_compact_vertices(enabled);
                for repeat in 0..2 {
                    let bytes = r.capture(&frame, width, height)?;
                    let actual = image(&bytes);
                    let max = actual
                        .iter()
                        .zip(&expected)
                        .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)))
                        .max()
                        .unwrap();
                    gates.push(json!({"case":case,"grid":grid,"optimized":enabled,"repeat":repeat,"maximum_rgb8":max}));
                    std::fs::write(out.join(format!("{case}-{grid}-optimized.png")), bytes)?;
                    ensure!(
                        max == 0,
                        "planet payload changed pixels: {case}/{grid}/{enabled}/{repeat} max={max}"
                    );
                }
            }
            engine.execute(
                Command::SetPlanetView {
                    view: Some(Settings {
                        distance_meters: distance,
                        yaw_degrees: yaw,
                        tile_subdivisions: grid,
                        atmosphere_optical_depth: depth,
                        atmosphere_optimization: false,
                        render_group_size: 8,
                        ..Default::default()
                    }),
                }
                .into(),
            )?;
            let grouped = engine.render_frame();
            r.set_geometry_indexing(true);
            r.set_compact_vertices(true);
            let bytes = r.capture(&grouped, width, height)?;
            let actual = image(&bytes);
            let maximum = actual
                .iter()
                .zip(&expected)
                .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)))
                .max()
                .unwrap();
            gates.push(json!({"case":case,"phase":"grouping","maximum_rgb8":maximum}));
            std::fs::write(out.join(format!("{case}-grouped.png")), bytes)?;
            ensure!(
                maximum == 0,
                "submission grouping changed pixels {case}: {maximum}"
            );
            for round in 0..if only_total { 0 } else { 4 } {
                for enabled in if round % 2 == 0 {
                    [false, true]
                } else {
                    [true, false]
                } {
                    let f = if enabled { &grouped } else { &frame };
                    let mut samples = vec![];
                    let mut transition = 0.;
                    for i in 0..36 {
                        let t = Instant::now();
                        r.draw(f, &target, width, height)?;
                        r.device.poll(wgpu::PollType::wait_indefinitely())?;
                        let elapsed = t.elapsed().as_secs_f64() * 1000.;
                        if i == 0 {
                            transition = elapsed;
                        }
                        if i >= 12 {
                            samples.push(elapsed);
                        }
                    }
                    samples.sort_by(f64::total_cmp);
                    rows.push(json!({"phase":"grouping","case":case,"grid":grid,"round":round,"optimized":enabled,"transition_ms":transition,"median_ms":samples[12],"p95_ms":samples[22],"stats":r.info()}));
                }
            }
            // Same geometry and optical model: enable finite contribution selection.
            engine.execute(
                Command::SetPlanetView {
                    view: Some(Settings {
                        distance_meters: distance,
                        yaw_degrees: yaw,
                        tile_subdivisions: grid,
                        atmosphere_optical_depth: depth,
                        atmosphere_optimization: true,
                        ..Default::default()
                    }),
                }
                .into(),
            )?;
            let finite = engine.render_frame();
            r.set_geometry_indexing(true);
            r.set_compact_vertices(true);
            let bytes = r.capture(&finite, width, height)?;
            let actual = image(&bytes);
            let maximum = actual
                .iter()
                .zip(&expected)
                .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)))
                .max()
                .unwrap();
            gates.push(json!({"case":case,"grid":grid,"phase":"finite_transport","maximum_rgb8":maximum,"world":engine.inspect_planet_view()}));
            std::fs::write(out.join(format!("{case}-{grid}-finite.png")), bytes)?;
            ensure!(
                maximum <= 2,
                "finite contribution error {case}/{grid}: {maximum}"
            );
            for round in 0..if only_total { 0 } else { 4 } {
                for enabled in if round % 2 == 0 {
                    [false, true]
                } else {
                    [true, false]
                } {
                    let f = if enabled { &finite } else { &grouped };
                    let mut samples = vec![];
                    let mut transition = 0.;
                    for i in 0..36 {
                        let t = Instant::now();
                        r.draw(f, &target, width, height)?;
                        r.device.poll(wgpu::PollType::wait_indefinitely())?;
                        let elapsed = t.elapsed().as_secs_f64() * 1000.;
                        if i == 0 {
                            transition = elapsed;
                        }
                        if i >= 12 {
                            samples.push(elapsed);
                        }
                    }
                    samples.sort_by(f64::total_cmp);
                    rows.push(json!({"phase":"finite_transport","case":case,"grid":grid,"round":round,"optimized":enabled,"transition_ms":transition,"median_ms":samples[12],"p95_ms":samples[22],"stats":r.info()}));
                }
            }
            if only_total {
                for round in 0..4 {
                    for enabled in if round % 2 == 0 {
                        [false, true]
                    } else {
                        [true, false]
                    } {
                        r.set_geometry_indexing(enabled);
                        r.set_compact_vertices(enabled);
                        let f = if enabled { &finite } else { &frame };
                        let uploads_before =
                            r.info()["instancing"]["geometry_uploads"].as_u64().unwrap();
                        let mut samples = vec![];
                        let mut transition = 0.;
                        for i in 0..36 {
                            let start = Instant::now();
                            r.draw(f, &target, width, height)?;
                            r.device.poll(wgpu::PollType::wait_indefinitely())?;
                            let elapsed = start.elapsed().as_secs_f64() * 1000.;
                            if i == 0 {
                                transition = elapsed;
                            }
                            if i >= 12 {
                                samples.push(elapsed);
                            }
                        }
                        samples.sort_by(f64::total_cmp);
                        rows.push(json!({"phase":"combined","case":case,"grid":grid,"round":round,"optimized":enabled,"transition_ms":transition,"geometry_upload_delta":r.info()["instancing"]["geometry_uploads"].as_u64().unwrap()-uploads_before,"median_ms":samples[12],"p95_ms":samples[22],"stats":r.info()}));
                    }
                }
            }
            std::fs::write(
                out.join(report_name),
                serde_json::to_vec_pretty(&json!({"rows":rows,"pixel_gates":gates}))?,
            )?;
        }
    }
    let report = json!({"build_id":env!("IGNIS_BUILD_ID"),"source_scene":"artifacts/planet-runtime/player.scene.json","source_planet":"artifacts/planet-runtime/world.json","method":"1280x800 native offscreen draw + GPU completion; 12 warmup + 24 samples; 4 alternating rounds; same source/camera/atmosphere within each grid. Not presented FPS. Grid32 unchanged; payload, grouping and finite contribution are compared separately.","rows":rows,"pixel_gates":gates});
    std::fs::write(out.join(report_name), serde_json::to_vec_pretty(&report)?)?;
    println!(
        "{}",
        json!({"report":format!("artifacts/planet-performance/{report_name}"),"pixel_gates":gates})
    );
    Ok(())
}
