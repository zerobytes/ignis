//! Diagnostic runner: native scene unchanged, warm draws, independent CPU clocks.
//! Link the existing compiled ignis library to avoid rebuilding parallel work.
use anyhow::{Context, Result};
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::{Camera, RoomRecipe, Scene},
    render::Renderer,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::BufReader,
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.
}
fn clock_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
fn summary(values: &[f64]) -> Value {
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    json!({"n":v.len(), "median_ms":(v[(v.len()-1)/2]+v[v.len()/2])/2.,
           "p95_ms":v[((v.len()-1)*95).div_ceil(100)], "min_ms":v[0], "max_ms":v[v.len()-1]})
}
fn wait(r: &Renderer) -> Result<()> {
    r.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(60)),
    })?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let output = args.get(1).context("output JSON path")?;
    let trace = args.get(2).map(String::as_str);
    let out = Path::new("artifacts/botanical-valley/performance");
    fs::create_dir_all(out)?;
    let build_id = Engine::new(RoomRecipe::default().generate()?)?
        .replay()
        .build_id;
    let start = Instant::now();
    let scene: Scene = serde_json::from_reader(BufReader::new(fs::File::open(
        "artifacts/botanical-valley/world.scene.json",
    )?))?;
    let parse_ms = ms(start);
    let start = Instant::now();
    let mut engine = Engine::new_with_mode(scene, ExecutionMode::Continuous)?;
    let engine_ms = ms(start);
    engine.execute(
        Command::SetPlayerView {
            view: Some(ignis::player_view::Config {
                actor: "player".into(),
                yaw_degrees: 0.,
                pitch_degrees: -3.,
                eye_height: 1.65,
                fov_degrees: 65.,
                mode: ignis::player_view::Mode::FirstPerson,
                third_person: Default::default(),
            }),
        }
        .into(),
    )?;
    engine.advance(60)?;
    let start = Instant::now();
    let mut frame = engine.render_frame();
    let first_frame_ms = ms(start);
    let cameras: Value =
        serde_json::from_reader(fs::File::open("artifacts/botanical-valley/views.json")?)?;
    let player_camera = frame.camera.clone();
    let gpu = wgpu::Instance::default();
    let start = Instant::now();
    let mut renderer = pollster::block_on(Renderer::new(&gpu, None))?;
    let gpu_init_ms = ms(start);
    let authored_light = frame.light.clone();
    let mut cpu_rows = vec![];
    let mut rows = vec![];
    let mut cases: Vec<(String, Camera)> = vec![("player_spawn".into(), player_camera)];
    if trace.is_none() {
        cases.push((
            "valley".into(),
            serde_json::from_value(cameras["valley"].clone())?,
        ));
        cases.push((
            "panorama".into(),
            serde_json::from_value(cameras["panorama"].clone())?,
        ));
    }
    for (pose, camera) in cases {
        let mut step = vec![];
        let mut prepare = vec![];
        for _ in 0..24 {
            let t = Instant::now();
            engine.advance(1)?;
            step.push(ms(t));
            let t = Instant::now();
            engine.update_render_frame(&mut frame);
            prepare.push(ms(t));
        }
        frame.camera = camera.clone();
        // Measurements below use the settled, frozen frame. No camera/density edits.
        cpu_rows.push(json!({"pose":pose,"one_tick":summary(&step),"prepare_frame":summary(&prepare),"step_samples_ms":step,"prepare_samples_ms":prepare}));
        let sizes = if trace.is_some() {
            vec![(2200, 1520)]
        } else {
            vec![(2200, 1520), (1100, 760)]
        };
        for (width, height) in sizes {
            let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("botanical valley cost target"),
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
            let mut samples: [Vec<Value>; 2] = [vec![], vec![]];
            let mut info = [Value::Null, Value::Null];
            let mut cold = [Value::Null, Value::Null];
            let rounds = if trace.is_some() { 1 } else { 4 };
            for round in 0..rounds {
                let variants = match trace {
                    Some("full") => vec![0],
                    Some("no_shadows") => vec![1],
                    Some(_) => anyhow::bail!("trace variant must be full or no_shadows"),
                    None => {
                        if round % 2 == 0 {
                            vec![0, 1]
                        } else {
                            vec![1, 0]
                        }
                    }
                };
                for variant in variants {
                    frame.light = authored_light.clone();
                    if variant == 1 {
                        frame.light.shadow = None;
                    }
                    for warm in 0..3 {
                        let t = Instant::now();
                        renderer.draw(&frame, &view, width, height)?;
                        let cpu = ms(t);
                        wait(&renderer)?;
                        if warm == 0 && round == 0 {
                            cold[variant] = json!({"cpu_draw_submit_ms":cpu,"draw_wait_ms":ms(t)});
                        }
                    }
                    if trace.is_some() {
                        fs::write(
                            out.join("trace-ready.json"),
                            serde_json::to_vec(
                                &json!({"pid":std::process::id(),"variant":trace,"ready_unix_ns":clock_ns().to_string()}),
                            )?,
                        )?;
                        let gate = Instant::now();
                        while !out.join("trace-go").exists() {
                            anyhow::ensure!(
                                gate.elapsed() < Duration::from_secs(120),
                                "trace start timeout"
                            );
                            std::thread::sleep(Duration::from_millis(25));
                        }
                    }
                    let count = if trace.is_some() { 8 } else { 4 };
                    for index in 0..count {
                        let epoch = clock_ns();
                        let t = Instant::now();
                        renderer.draw(&frame, &view, width, height)?;
                        let cpu = ms(t);
                        wait(&renderer)?;
                        let total = ms(t);
                        samples[variant].push(json!({"round":round,"index":index,"start_unix_ns":epoch.to_string(),
                            "cpu_draw_submit_ms":cpu,"completion_wait_ms":total-cpu,"draw_wait_ms":total}));
                    }
                    info[variant] = renderer.info();
                    eprintln!(
                        "{pose} {width}x{height} {} round{round} done",
                        if variant == 0 { "full" } else { "no_shadows" }
                    );
                }
            }
            for (variant, name) in [(0, "full"), (1, "no_shadows")] {
                if samples[variant].is_empty() {
                    continue;
                }
                let metric = |key: &str| {
                    summary(
                        &samples[variant]
                            .iter()
                            .map(|s| s[key].as_f64().unwrap())
                            .collect::<Vec<_>>(),
                    )
                };
                rows.push(json!({"pose":pose,"camera":camera,"width":width,"height":height,"variant":name,
                    "cold":cold[variant],"cpu_draw_submit":metric("cpu_draw_submit_ms"),
                    "completion_wait":metric("completion_wait_ms"),"draw_wait":metric("draw_wait_ms"),
                    "samples":samples[variant],"renderer":info[variant]}));
            }
            let report = json!({"build_id":build_id,"trace_variant":trace,"cpu":cpu_rows,"rows":rows,
                "cold":{"parse_ms":parse_ms,"engine_ms":engine_ms,"first_frame_ms":first_frame_ms,"gpu_init_ms":gpu_init_ms},
                "method":"Warm alternating 4x4 measured frames per variant after3warmups/block, frozen frame, reused target, no PNG/readback/present in timed loop. CPU draw wall includes preparation/encoding/queue submission and possible backpressure; completion_wait is residual CPU wait, NOT pure GPU timestamp. Shadow-off is a controlled combined effect, not isolated depth-pass GPU cost. CPU tick/frame preparation sampled separately. First-person body hidden; all vegetation/terrain unchanged."});
            fs::write(output, serde_json::to_vec_pretty(&report)?)?;
        }
    }
    if trace.is_some() {
        let gate = Instant::now();
        while !out.join("trace-finish").exists() {
            anyhow::ensure!(
                gate.elapsed() < Duration::from_secs(120),
                "trace finish timeout"
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    Ok(())
}
