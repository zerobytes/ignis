//! CPU frame pacing along real KCC movement. No GPU, window, or presentation.
use anyhow::Result;
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::{MovementMode, Scene},
    physics::Input,
    planet::{orbit::Settings, runtime::Config},
    player_view,
};
use serde_json::json;
use std::{path::Path, time::Instant};

fn main() -> Result<()> {
    let out = Path::new("artifacts/planet-pacing");
    std::fs::create_dir_all(out)?;
    let reference = std::env::args().any(|a| a == "--reference");
    let scene: Scene = serde_json::from_slice(&std::fs::read(
        "artifacts/planet-runtime/player.scene.json",
    )?)?;
    let config: Config =
        serde_json::from_slice(&std::fs::read("artifacts/planet-runtime/world.json")?)?;
    let mut e = Engine::new_with_mode(scene, ExecutionMode::Continuous)?;
    e.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(config)),
        }
        .into(),
    )?;
    e.advance(120)?;
    e.execute(
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
    e.execute(
        Command::SetPlanetView {
            view: Some(Settings {
                distance_meters: 4.,
                viewport: [2248, 1580],
                reuse_vertex_samples: !reference,
                canonical_cpu_stream: !reference,
                ..Default::default()
            }),
        }
        .into(),
    )?;
    let mut frame = e.render_frame();
    let mut samples = vec![];
    for (name, count) in [("idle", 12), ("run", 240), ("pan", 24)] {
        e.execute(
            Command::SetPlayerViewInput {
                input: Input {
                    x: 0.,
                    z: if name == "run" { -1. } else { 0. },
                    mode: MovementMode::Run,
                },
                control: None,
            }
            .into(),
        )?;
        for index in 0..count {
            let start = Instant::now();
            if name == "pan" {
                e.execute(
                    Command::LookPlanetView {
                        yaw_delta_degrees: 2.,
                        tilt_delta_degrees: 0.,
                    }
                    .into(),
                )?;
            }
            let control_ms = start.elapsed().as_secs_f64() * 1000.;
            let step = Instant::now();
            e.advance(1)?;
            let step_ms = step.elapsed().as_secs_f64() * 1000.;
            let render = Instant::now();
            e.update_render_frame(&mut frame);
            let frame_ms = render.elapsed().as_secs_f64() * 1000.;
            let elapsed = start.elapsed().as_secs_f64() * 1000.;
            let row = json!({"segment":name,"index":index,"total_cpu_ms":elapsed,"control_ms":control_ms,
                "step_ms":step_ms,"frame_prepare_ms":frame_ms,"view":e.inspect_planet_view(),
                "world":e.inspect_planet_runtime(),"cpu_corner_vertices":frame.meshes.resident_vertex_count(),"triangles":frame.meshes.triangle_count()});
            if elapsed > 30. {
                println!("{name} {index}: {elapsed:.2}ms step{step_ms:.2} frame{frame_ms:.2}");
            }
            samples.push(row);
        }
    }
    let mut summary = json!({});
    for segment in ["idle", "run", "pan"] {
        let mut values: Vec<_> = samples
            .iter()
            .filter(|s| s["segment"] == segment)
            .map(|s| s["total_cpu_ms"].as_f64().unwrap())
            .collect();
        values.sort_by(f64::total_cmp);
        let n = values.len();
        summary[segment] = json!({"samples":n,"p50_ms":values[n/2],"p95_ms":values[(n-1)*95/100],
            "p99_ms":values[(n-1)*99/100],"max_ms":values[n-1],"over_33ms":values.iter().filter(|&&v|v>33.333).count()});
    }
    let path = out.join(if reference {
        "cpu-reference.json"
    } else {
        "cpu-current.json"
    });
    std::fs::write(
        path,
        serde_json::to_vec_pretty(
            &json!({"build_id":env!("IGNIS_BUILD_ID"),"reference":reference,"method":"one native tick plus frame preparation; no GPU/present/IPC", "summary":summary,"samples":samples}),
        )?,
    )?;
    println!("{summary:#}");
    Ok(())
}
