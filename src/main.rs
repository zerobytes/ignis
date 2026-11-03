use ignis::{
    engine::{Command, Request},
    model::RoomRecipe,
    service::Service,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

fn main() -> anyhow::Result<()> {
    let mut root = default_root()?;
    let mut scene = None;
    let mut mcp = false;
    let mut window = false;
    let mut editor = false;
    let mut server = None;
    let mut command = None;
    let mut continuous = false;
    let mut planet_runtime = None;
    let mut planet_view_distance = None;
    let mut first_person = None;
    let mut third_person = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                root = PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--root requires a directory"))?,
                )
            }
            "--scene" => {
                scene = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--scene requires a relative path"))?,
                )
            }
            "--mcp" => mcp = true,
            "--server" => {
                server = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--server requires IP:port"))?,
                )
            }
            "--window" => window = true,
            "--continuous" => continuous = true,
            "--planet-runtime" => {
                planet_runtime = Some(args.next().ok_or_else(|| {
                    anyhow::anyhow!(
                        "--planet-runtime requires a project-relative configuration path"
                    )
                })?)
            }
            "--planet-view-distance" => {
                planet_view_distance = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--planet-view-distance requires metres"))?
                        .parse::<f64>()?,
                );
            }
            "--first-person" => {
                first_person = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--first-person requires a player ID"))?,
                )
            }
            "--third-person" => {
                third_person = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--third-person requires a player ID"))?,
                );
            }
            "--editor" => {
                editor = true;
                window = true;
            }
            "--command" => {
                command = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--command requires a JSON request"))?,
                )
            }
            "--help" | "-h" => {
                println!(
                    "ignis [--root DIRECTORY] [--scene RELATIVE_JSON] [--window | --editor] [--mcp]\nignis [--root DIRECTORY] [--scene RELATIVE_JSON] --command '{{\"command\":{{\"op\":\"inspect\"}}}}'\n\n--server IP:port: authoritative headless QUIC host; default roster comes from --scene.\n--mcp: newline-delimited MCP stdio. Combine with --window to see the same world.\n--editor: native editor with menus, workspaces and dockable areas.\n--continuous: unbounded offline runtime without recording input history.\n--planet-runtime CONFIG: offline single-player transported surface runtime (requires --continuous).\n--planet-view-distance METRES: surface-to-globe session camera (requires planet runtime).\n--first-person PLAYER: session camera at the player eyes; click to look, Esc releases mouse.\n--third-person PLAYER: orbital player camera with physical obstruction handling.\nNative planet: wheel continuous zoom; G globe/surface; mouse look.\nNative: F toggles player view; Tab first/third; wheel zooms third; C execution mode at paused tick0.\nDefault: native playable window, paused. WASD/arrows move; P play/pause; R reset; Esc close.\nAll game controls use the same command API exposed by MCP.\nFile paths are confined to --root. Closing stdin does not close an open window."
                );
                return Ok(());
            }
            _ => anyhow::bail!("Unknown argument: {arg}"),
        }
    }
    anyhow::ensure!(
        command.is_none() || (!mcp && !window && server.is_none()),
        "--command cannot be combined with --mcp or --window"
    );
    let mut service = Service::new(&root, RoomRecipe::default().generate()?)?;
    if editor && let Some(path) = ignis::project_browser::default_file() {
        if let Err(error) = service.execute(
            Command::UseProjectLibrary {
                path: path.to_string_lossy().into_owned(),
            }
            .into(),
        ) {
            service.project_library.warning =
                Some(format!("Biblioteca não carregada: {}", error.message));
        } else if let Err(error) = service.execute(
            Command::DiscoverProjects {
                paths: ignis::project_browser::discovery_locations(),
            }
            .into(),
        ) {
            service.project_library.warning = Some(format!(
                "Não foi possível procurar projetos: {}",
                error.message
            ));
        }
    }
    if let Some(path) = scene {
        service.execute(Command::LoadScene { path }.into())?;
    }
    if continuous {
        service.execute(
            Command::SetExecutionMode {
                mode: ignis::engine::ExecutionMode::Continuous,
            }
            .into(),
        )?;
    }
    if let Some(path) = planet_runtime {
        anyhow::ensure!(
            !editor && server.is_none(),
            "Planet runtime currently requires offline gameplay"
        );
        service.execute(Command::LoadPlanetRuntime { path }.into())?;
    }
    anyhow::ensure!(
        first_person.is_none() || third_person.is_none(),
        "Choose either --first-person or --third-person"
    );
    let perspective = if third_person.is_some() {
        ignis::player_view::Mode::ThirdPerson
    } else {
        ignis::player_view::Mode::FirstPerson
    };
    if let Some(actor) = first_person.or(third_person) {
        anyhow::ensure!(
            !editor && server.is_none(),
            "Player views require local gameplay, not editor/server"
        );
        let e = service
            .engine
            .scene
            .entities
            .iter()
            .find(|e| e.id == actor)
            .ok_or_else(|| anyhow::anyhow!("Player not found"))?;
        let c = &service.engine.scene.camera;
        let view = ignis::player_view::Config {
            actor,
            yaw_degrees: (c.target[0] - c.position[0])
                .atan2(c.position[2] - c.target[2])
                .to_degrees(),
            pitch_degrees: 0.,
            eye_height: (e.transform.size[1] * 0.9).min(1.65),
            fov_degrees: 75.,
            mode: perspective,
            third_person: Default::default(),
        };
        service.execute(Command::SetPlayerView { view: Some(view) }.into())?;
    }
    if let Some(distance_meters) = planet_view_distance {
        let mut view = service.engine.planet_view_settings()?;
        view.distance_meters = distance_meters;
        service.execute(Command::SetPlanetView { view: Some(view) }.into())?;
    }
    if let Some(address) = &server {
        anyhow::ensure!(!window, "--server runs headless; use MCP to inspect it");
        service.execute(
            Command::StartMultiplayerHost {
                address: address.clone(),
                capacity: ignis::targeting::MAX_PLAYERS,
            }
            .into(),
        )?;
    }
    if let Some(command) = command {
        let reply = service.execute(serde_json::from_str::<Request>(&command)?)?;
        println!("{}", serde_json::to_string_pretty(&reply.data)?);
        return Ok(());
    }
    if !mcp && server.is_none() {
        window = true;
    }
    let service = Arc::new(Mutex::new(service));
    if window {
        if mcp {
            let shared = service.clone();
            std::thread::spawn(move || {
                if let Err(e) =
                    ignis::mcp::serve(std::io::stdin().lock(), std::io::stdout().lock(), shared)
                {
                    eprintln!("MCP transport: {e}");
                }
            });
        }
        if editor {
            ignis::editor_window::run(service)?;
        } else {
            ignis::window::run(service)?;
        }
    } else {
        let stop = Arc::new(AtomicBool::new(false));
        let shared = service.clone();
        let finished = stop.clone();
        let clock = std::thread::spawn(move || {
            let mut last = Instant::now();
            let mut accumulator = 0.;
            while !finished.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(4));
                let now = Instant::now();
                let elapsed = now.duration_since(last).as_secs_f32().min(0.1);
                last = now;
                let mut s = shared.lock().unwrap();
                if !s.engine.playing {
                    let _ = s.poll_network();
                }
                if s.engine.playing {
                    accumulator += elapsed;
                    while accumulator >= ignis::model::FIXED_DT {
                        if s.advance(1).is_err() {
                            s.engine.playing = false;
                            break;
                        }
                        accumulator -= ignis::model::FIXED_DT;
                    }
                } else {
                    accumulator = 0.;
                }
            }
        });
        let result = if server.is_some() && !mcp {
            let mut announced = false;
            loop {
                std::thread::sleep(Duration::from_millis(50));
                let mut s = service.lock().unwrap();
                let state = s.execute(Command::InspectMultiplayerHost.into())?.data;
                let status = state["transport"]["state"].as_str().unwrap_or("");
                if status == "failed" {
                    break Err(anyhow::anyhow!(
                        "QUIC host: {}",
                        state["transport"]["error"]
                    ));
                }
                if status == "listening" && !announced {
                    eprintln!("{}", serde_json::to_string(&state["transport"])?);
                    announced = true;
                }
                if s.engine.exit_requested {
                    break Ok(());
                }
            }
        } else {
            ignis::mcp::serve(std::io::stdin().lock(), std::io::stdout().lock(), service)
                .map_err(anyhow::Error::from)
        };
        stop.store(true, Ordering::Relaxed);
        let _ = clock.join();
        result?;
    }
    Ok(())
}

fn default_root() -> std::io::Result<PathBuf> {
    let executable = std::env::current_exe()?;
    // The local .app lives in artifacts/. Keep launch-from-Finder scoped to this project.
    if executable
        .parent()
        .and_then(|p| p.file_name())
        .is_some_and(|n| n == "MacOS")
        && let Some(project) = executable.ancestors().nth(5)
        && project.join("Cargo.toml").is_file()
    {
        return Ok(project.to_path_buf());
    }
    std::env::current_dir()
}
