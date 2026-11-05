use crate::{
    engine::{Command, Request},
    model::FIXED_DT,
    physics::Input,
    render::{RenderFrame, Renderer},
    service::Service,
};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

pub fn run(service: Arc<Mutex<Service>>) -> anyhow::Result<()> {
    {
        let mut s = service.lock().unwrap();
        let settings = crate::viewport_performance::Settings {
            enabled: true,
            ..s.viewport_performance.settings
        };
        s.execute(Command::SetViewportPerformance { settings }.into())?;
    }
    let local_context = service.lock().unwrap().engine.player_view_context();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        service,
        window: None,
        surface: None,
        renderer: None,
        config: None,
        keys: BTreeSet::new(),
        cursor: None,
        keyboard: KeyboardControl::default(),
        input_context: None,
        local_context,
        looking: false,
        last: Instant::now(),
        accumulator: 0.,
        error: None,
        frame: None,
        title_state: None,
        presented_state: None,
        occluded: false,
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        anyhow::bail!(error);
    }
    Ok(())
}

struct App {
    service: Arc<Mutex<Service>>,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    renderer: Option<Renderer>,
    config: Option<wgpu::SurfaceConfiguration>,
    keys: BTreeSet<KeyCode>,
    cursor: Option<[f32; 2]>,
    keyboard: KeyboardControl,
    input_context: Option<Option<crate::network::controls::Context>>,
    local_context: Option<(u64, u64, u64, u64)>,
    looking: bool,
    last: Instant,
    accumulator: f32,
    error: Option<String>,
    frame: Option<RenderFrame>,
    title_state: Option<(bool, u64, u64, u64, crate::model::MovementMode, bool)>,
    presented_state: Option<(bool, u64, u64, u64, crate::model::MovementMode, bool)>,
    occluded: bool,
}
impl App {
    fn look_capture(&mut self, enabled: bool) {
        if let Some(w) = &self.window {
            let result = if enabled {
                w.set_cursor_grab(CursorGrabMode::Locked)
                    .or_else(|_| w.set_cursor_grab(CursorGrabMode::Confined))
            } else {
                w.set_cursor_grab(CursorGrabMode::None)
            };
            self.looking = enabled && result.is_ok();
            w.set_cursor_visible(!self.looking);
        }
    }
    fn command(&self, command: Command) {
        if let Err(e) = self.service.lock().unwrap().execute(Request::from(command)) {
            eprintln!("{}: {}", e.code, e.message);
        }
    }
    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f32().min(0.1);
        self.last = now;
        let mut service = self.service.lock().unwrap();
        if !service.engine.playing {
            let _ = service.poll_network();
        }
        if service.engine.playing {
            self.accumulator += elapsed;
            while self.accumulator >= FIXED_DT {
                if let Err(e) = service.advance(1) {
                    service.engine.playing = false;
                    eprintln!("{}", e.message);
                    break;
                }
                self.accumulator -= FIXED_DT;
            }
        } else {
            self.accumulator = 0.;
        }
        if let Some(config) = &self.config {
            service
                .engine
                .prepare_planet_viewport(config.width.max(64), config.height.max(64))?;
        }
        if let Some(frame) = &mut self.frame {
            service.update_game_frame(frame);
        } else {
            self.frame = Some(service.game_frame());
        }
        let status = if service.engine.playing {
            "PLAY"
        } else {
            "PAUSED"
        };
        let remote = service.remote_stamp();
        let title_state = remote.map(|s| (s.0, s.1, s.2, s.3, s.4, true)).unwrap_or((
            service.engine.playing,
            service.engine.simulation.tick,
            service.engine.revision,
            service.engine.view_revision,
            service.engine.input.mode,
            false,
        ));
        let guides = service.engine.debug_view;
        let metrics = if service.viewport_performance.settings.enabled {
            let sample = &service.viewport_performance.sample;
            match (sample.fps, sample.frame_ms) {
                (Some(fps), Some(ms)) => format!("{fps:.1} FPS · {ms:.2} ms | "),
                _ => "FPS: medindo | ".to_string(),
            }
        } else {
            String::new()
        };
        let title = if self.title_state != Some(title_state) {
            self.title_state = Some(title_state);
            if remote.is_some() {
                Some(format!(
                    "Ignis Multiplayer | {metrics}{} | tick {} | clique alvo | WASD | Shift correr | Ctrl furtivo |1-6 habilidade | X cancelar | R reconectar | Esc fechar",
                    if title_state.0 {
                        "conectado"
                    } else {
                        "conectando ou desconectado"
                    },
                    title_state.1
                ))
            } else {
                let help = if service.engine.inspection_flight_active() {
                    "VOO · WASD mover · Espaço subir · C descer · Shift rápido · Ctrl devagar · clique olhar · Tab 1ª/3ª · Esc soltar"
                } else if service.engine.planet_active() {
                    "WASD mover · Shift correr · roda: superfície/globo · G globo/voltar · F superfície · Tab 1ª/3ª · clique olhar · Esc soltar · P pausa"
                } else if guides.npc_routes {
                    "J: rotas | branco: caminho/destino | lilas: patrulha | vermelho: inacessivel"
                } else if guides.npc_hearing || guides.noise_events {
                    "H: audicao | roxo: alcance nominal | laranja: ruido | V: visao | N: emitir"
                } else if guides.npc_vision {
                    "V: guias | FOV nominal | verde: viu | amarelo: memoria | azul: sem pista"
                } else {
                    "WASD mover · Shift correr · F câmera · Tab 1ª/3ª · roda zoom · clique olhar · Esc soltar/fechar · P play/pause · R reset"
                };
                let mode = match title_state.4 {
                    crate::model::MovementMode::Walk => "caminhada",
                    crate::model::MovementMode::Sneak => "furtivo",
                    crate::model::MovementMode::Run => "corrida",
                };
                Some(format!(
                    "Ignis | {metrics}{status} | {mode} | tick {} | selecionado: {}{} | F3 FPS | {help}",
                    title_state.1,
                    service.engine.selected_id().unwrap_or("nenhum"),
                    if service.engine.scene.npc_generators.len() == 1 {
                        let g = &service.engine.simulation.generators.generators[0];
                        format!(
                            " | gerador {}: {:?}{} [B retomar/O pausar/K cancelar]",
                            service.engine.scene.npc_generators[0].id,
                            g.status,
                            if g.finished(service.engine.scene.npc_generators[0].max_births) {
                                " concluido"
                            } else if g.activation.is_some_and(|a| !a.open) {
                                " aguardando regiao"
                            } else {
                                ""
                            }
                        )
                    } else {
                        String::new()
                    }
                ))
            }
        } else {
            None
        };
        drop(service);
        if let Some(title) = title {
            self.window.as_ref().unwrap().set_title(&title);
        }
        let surface = self.surface.as_ref().unwrap();
        let renderer = self.renderer.as_mut().unwrap();
        let config = self.config.as_mut().unwrap();
        let settings = self.service.lock().unwrap().viewport_performance.settings;
        let desired = if settings.enabled && !settings.vsync {
            wgpu::PresentMode::AutoNoVsync
        } else {
            wgpu::PresentMode::Fifo
        };
        if config.present_mode != desired {
            config.present_mode = desired;
            surface.configure(&renderer.device, config);
        }
        if self.window.as_ref().unwrap().inner_size().width == 0
            || self.window.as_ref().unwrap().inner_size().height == 0
        {
            return Ok(());
        }
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                surface.configure(&renderer.device, config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        renderer.draw(
            self.frame.as_ref().unwrap(),
            &frame.texture.create_view(&Default::default()),
            config.width,
            config.height,
        )?;
        frame.present();
        if settings.enabled && !self.occluded {
            let mut s = self.service.lock().unwrap();
            let optimized = self
                .frame
                .as_ref()
                .unwrap()
                .light
                .atmosphere
                .is_some_and(|a| a.reduce_details);
            let mode = if settings.vsync {
                "Fifo (VSync)"
            } else {
                "AutoNoVsync (requested)"
            };
            if s.viewport_performance.presented(
                Instant::now(),
                now.elapsed().as_secs_f64() * 1000.,
                [config.width, config.height],
                mode,
                optimized,
            ) {
                s.viewport_performance.sample.renderer = renderer.info();
                self.title_state = None;
            }
        }
        self.presented_state = Some(title_state);
        Ok(())
    }
}
impl ApplicationHandler for App {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if self.looking
            && let DeviceEvent::MouseMotion { delta: (x, y) } = event
        {
            let mut service = self.service.lock().unwrap();
            if service.engine.player_view_context().is_some() {
                let owned = self.keyboard.revision == Some(service.engine.input_revision);
                if service
                    .execute(
                        Command::LookPlayerView {
                            yaw_delta_degrees: x as f32 * 0.12,
                            pitch_delta_degrees: -y as f32 * 0.12,
                        }
                        .into(),
                    )
                    .is_ok()
                    && owned
                {
                    let _ = self.keyboard.apply_keys(&mut service, &self.keys);
                }
            }
        }
    }
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let init = (|| -> anyhow::Result<()> {
            let window = Arc::new(
                event_loop.create_window(
                    Window::default_attributes()
                        .with_title("Ignis Engine — 3D Laboratory")
                        .with_inner_size(winit::dpi::LogicalSize::new(1100., 760.)),
                )?,
            );
            let instance = wgpu::Instance::new(&Default::default());
            let surface = instance.create_surface(window.clone())?;
            let renderer = pollster::block_on(Renderer::new(&instance, Some(&surface)))?;
            let size = window.inner_size();
            let mut config = surface
                .get_default_config(&renderer.adapter, size.width.max(1), size.height.max(1))
                .ok_or_else(|| anyhow::anyhow!("Unsupported window surface"))?;
            config.format = renderer.format;
            config.present_mode = wgpu::PresentMode::AutoVsync;
            surface.configure(&renderer.device, &config);
            eprintln!("Graphics: {}", renderer.info());
            self.window = Some(window);
            self.surface = Some(surface);
            self.renderer = Some(renderer);
            self.config = Some(config);
            self.last = Instant::now();
            Ok(())
        })();
        if let Err(e) = init {
            self.error = Some(e.to_string());
            event_loop.exit();
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Some([position.x as f32, position.y as f32]);
            }
            WindowEvent::CursorLeft { .. } => self.cursor = None,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let first_person = {
                    let s = self.service.lock().unwrap();
                    !s.remote_client() && s.engine.player_view_context().is_some()
                };
                if first_person {
                    self.look_capture(true);
                    return;
                }
                let remote_client = { self.service.lock().unwrap().remote_client() };
                if remote_client {
                    if let (Some([x, y]), Some(window)) = (self.cursor, &self.window) {
                        let size = window.inner_size();
                        self.command(Command::PickMultiplayer {
                            x,
                            y,
                            width: size.width,
                            height: size.height,
                        });
                    }
                    return;
                }
                if let (Some([x, y]), Some(window)) = (self.cursor, &self.window) {
                    let size = window.inner_size();
                    if size.width > 0 && size.height > 0 {
                        self.command(Command::SelectAt {
                            x,
                            y,
                            width: size.width,
                            height: size.height,
                        });
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let mut service = self.service.lock().unwrap();
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => f64::from(y),
                    MouseScrollDelta::PixelDelta(p) => p.y / 40.,
                };
                if service.engine.planet_active() {
                    if let Err(e) = service.execute(
                        Command::ZoomPlanetView {
                            factor: (-amount.clamp(-20., 20.) * 0.35).exp(),
                        }
                        .into(),
                    ) {
                        eprintln!("{}: {}", e.code, e.message);
                    }
                } else if !service.remote_client()
                    && let Some(mut view) = service.engine.player_view_config()
                    && view.mode == crate::player_view::Mode::ThirdPerson
                {
                    view.third_person.distance_meters =
                        (view.third_person.distance_meters - amount as f32 * 0.5).clamp(0.25, 100.);
                    if let Err(e) =
                        service.execute(Command::SetPlayerView { view: Some(view) }.into())
                    {
                        eprintln!("{}: {}", e.code, e.message);
                    }
                }
            }
            WindowEvent::CloseRequested => {
                self.command(Command::Exit);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.presented_state = None;
                if size.width > 0
                    && size.height > 0
                    && let (Some(surface), Some(renderer), Some(config)) =
                        (&self.surface, &self.renderer, &mut self.config)
                {
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&renderer.device, config);
                }
            }
            WindowEvent::Focused(false) => {
                self.service.lock().unwrap().viewport_performance.pause();
                self.look_capture(false);
                self.keys.clear();
                if let Err(e) = self.keyboard.release(&mut self.service.lock().unwrap()) {
                    eprintln!("{}: {}", e.code, e.message);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && !event.repeat
                    && event.physical_key == PhysicalKey::Code(KeyCode::F3)
                {
                    let mut settings = self.service.lock().unwrap().viewport_performance.settings;
                    settings.enabled = !settings.enabled;
                    self.command(Command::SetViewportPerformance { settings });
                    self.title_state = None;
                    return;
                }
                if event.state == ElementState::Pressed
                    && !event.repeat
                    && event.physical_key == PhysicalKey::Code(KeyCode::F4)
                {
                    let mut settings = self.service.lock().unwrap().engine.mesh_lod_settings;
                    settings.enabled = !settings.enabled;
                    self.command(Command::SetMeshLodSettings { settings });
                    self.title_state = None;
                    return;
                }
                let (remote_client, context, local_context) = {
                    let service = self.service.lock().unwrap();
                    (
                        service.remote_client(),
                        service
                            .remote_client()
                            .then(|| service.remote_input_context()),
                        service.engine.player_view_context(),
                    )
                };
                if self.local_context != local_context {
                    self.keys.clear();
                    let _ = self.keyboard.release(&mut self.service.lock().unwrap());
                    self.local_context = local_context;
                    self.look_capture(false);
                    return;
                }
                if self.input_context != context {
                    self.keys.clear();
                    let _ = self.keyboard.release(&mut self.service.lock().unwrap());
                    self.input_context = context;
                    // The first queued event observing a new context cannot own it.
                    return;
                }
                if let PhysicalKey::Code(code) = event.physical_key {
                    let down = event.state == ElementState::Pressed;
                    // OS repeats belong to the old hold. A cleared life/connection
                    // context needs a new physical keydown before it can move.
                    if (remote_client || self.local_context.is_some()) && down && event.repeat {
                        return;
                    }
                    let changed = if down {
                        self.keys.insert(code)
                    } else {
                        self.keys.remove(&code)
                    };
                    if remote_client {
                        if down && !event.repeat {
                            match code {
                                KeyCode::Escape => {
                                    self.command(Command::Exit);
                                    event_loop.exit();
                                }
                                KeyCode::KeyR => self.command(Command::ReconnectMultiplayer),
                                KeyCode::KeyX => self.command(Command::CancelMultiplayerAbility),
                                KeyCode::Digit1
                                | KeyCode::Digit2
                                | KeyCode::Digit3
                                | KeyCode::Digit4
                                | KeyCode::Digit5
                                | KeyCode::Digit6 => {
                                    let slot = match code {
                                        KeyCode::Digit1 => 1,
                                        KeyCode::Digit2 => 2,
                                        KeyCode::Digit3 => 3,
                                        KeyCode::Digit4 => 4,
                                        KeyCode::Digit5 => 5,
                                        _ => 6,
                                    };
                                    let spatial = self.service.lock().unwrap().remote_grant(slot)
                                        == Some(crate::abilities::Selection::Spatial);
                                    if spatial {
                                        if let (Some([x, y]), Some(window)) =
                                            (self.cursor, &self.window)
                                        {
                                            let size = window.inner_size();
                                            self.command(Command::UseMultiplayerAbilityAt {
                                                slot,
                                                x,
                                                y,
                                                width: size.width,
                                                height: size.height,
                                                plane_y: None,
                                            });
                                        }
                                    } else {
                                        self.command(Command::UseMultiplayerAbility {
                                            slot,
                                            target: None,
                                            aim: None,
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                        if changed
                            && matches!(
                                code,
                                KeyCode::KeyW
                                    | KeyCode::KeyA
                                    | KeyCode::KeyS
                                    | KeyCode::KeyD
                                    | KeyCode::ArrowUp
                                    | KeyCode::ArrowDown
                                    | KeyCode::ArrowLeft
                                    | KeyCode::ArrowRight
                                    | KeyCode::ShiftLeft
                                    | KeyCode::ShiftRight
                                    | KeyCode::ControlLeft
                                    | KeyCode::ControlRight
                                    | KeyCode::Space
                                    | KeyCode::KeyC
                            )
                            && let Err(e) = self
                                .keyboard
                                .apply_keys(&mut self.service.lock().unwrap(), &self.keys)
                        {
                            eprintln!("{}: {}", e.code, e.message);
                        }
                        return;
                    }
                    if down && !event.repeat {
                        match code {
                            KeyCode::KeyB | KeyCode::KeyO | KeyCode::KeyK => {
                                let generator = {
                                    let service = self.service.lock().unwrap();
                                    (service.engine.scene.npc_generators.len() == 1)
                                        .then(|| service.engine.scene.npc_generators[0].id.clone())
                                };
                                if let Some(generator) = generator {
                                    use crate::generator_control::Action;
                                    self.command(Command::ControlNpcGenerator {
                                        generator,
                                        action: match code {
                                            KeyCode::KeyB => Action::Activate,
                                            KeyCode::KeyO => Action::Pause,
                                            _ => Action::Cancel,
                                        },
                                    });
                                }
                            }
                            KeyCode::KeyP => {
                                let playing = self.service.lock().unwrap().engine.playing;
                                self.command(if playing {
                                    Command::Pause
                                } else {
                                    Command::Play
                                });
                            }
                            KeyCode::KeyC => {
                                let mode = self.service.lock().unwrap().engine.execution_mode();
                                self.command(Command::SetExecutionMode {
                                    mode: match mode {
                                        crate::engine::ExecutionMode::Recorded => {
                                            crate::engine::ExecutionMode::Continuous
                                        }
                                        crate::engine::ExecutionMode::Continuous => {
                                            crate::engine::ExecutionMode::Recorded
                                        }
                                    },
                                });
                            }
                            KeyCode::KeyG
                                if self.service.lock().unwrap().engine.planet_active() =>
                            {
                                let mut s = self.service.lock().unwrap();
                                if let Ok(mut view) = s.engine.planet_view_settings() {
                                    let radius = s
                                        .engine
                                        .simulation
                                        .planet
                                        .as_ref()
                                        .unwrap()
                                        .config
                                        .document
                                        .radius();
                                    let command = if view.distance_meters > radius {
                                        view.distance_meters = 0.;
                                        view.tilt_degrees = 0.;
                                        Command::SetPlanetView { view: Some(view) }
                                    } else {
                                        Command::FramePlanetView
                                    };
                                    if let Err(e) = s.execute(command.into()) {
                                        eprintln!("{}: {}", e.code, e.message);
                                    }
                                }
                            }
                            KeyCode::KeyF
                                if self.service.lock().unwrap().engine.planet_active() =>
                            {
                                let mut s = self.service.lock().unwrap();
                                if let Ok(mut view) = s.engine.planet_view_settings() {
                                    view.distance_meters = 0.;
                                    view.tilt_degrees = 0.;
                                    let _ = s.execute(
                                        Command::SetPlanetView { view: Some(view) }.into(),
                                    );
                                }
                            }
                            KeyCode::KeyF => {
                                let view = {
                                    let s = self.service.lock().unwrap();
                                    if s.engine.player_view_context().is_some() {
                                        None
                                    } else {
                                        s.engine
                                            .scene
                                            .entities
                                            .iter()
                                            .find(|e| {
                                                e.body == Some(crate::model::BodyKind::Player)
                                            })
                                            .map(|e| {
                                                let c = &s.engine.scene.camera;
                                                crate::player_view::Config {
                                                    actor: e.id.clone(),
                                                    yaw_degrees: (c.target[0] - c.position[0])
                                                        .atan2(c.position[2] - c.target[2])
                                                        .to_degrees(),
                                                    pitch_degrees: 0.,
                                                    eye_height: (e.transform.size[1] * 0.9)
                                                        .min(1.65),
                                                    fov_degrees: 75.,
                                                    mode: Default::default(),
                                                    third_person: Default::default(),
                                                }
                                            })
                                    }
                                };
                                self.look_capture(false);
                                self.keys.clear();
                                let _ = self.keyboard.release(&mut self.service.lock().unwrap());
                                self.command(Command::SetPlayerView { view });
                            }
                            KeyCode::Tab if self.service.lock().unwrap().engine.planet_active() => {
                                let mut s = self.service.lock().unwrap();
                                if let Ok(mut view) = s.engine.planet_view_settings() {
                                    view.distance_meters =
                                        if view.distance_meters < 1. { 4. } else { 0. };
                                    let _ = s.execute(
                                        Command::SetPlanetView { view: Some(view) }.into(),
                                    );
                                }
                            }
                            KeyCode::Tab => {
                                let mut s = self.service.lock().unwrap();
                                if let Some(mut view) = s.engine.player_view_config() {
                                    view.mode = match view.mode {
                                        crate::player_view::Mode::FirstPerson => {
                                            crate::player_view::Mode::ThirdPerson
                                        }
                                        crate::player_view::Mode::ThirdPerson => {
                                            crate::player_view::Mode::FirstPerson
                                        }
                                    };
                                    if let Err(e) = s
                                        .execute(Command::SetPlayerView { view: Some(view) }.into())
                                    {
                                        eprintln!("{}: {}", e.code, e.message);
                                    }
                                }
                            }
                            KeyCode::KeyV => {
                                let enabled =
                                    self.service.lock().unwrap().engine.debug_view.npc_vision;
                                self.command(Command::SetDebugView {
                                    npc_vision: Some(!enabled),
                                    npc_hearing: None,
                                    noise_events: None,
                                    npc_routes: None,
                                });
                            }
                            KeyCode::KeyH => {
                                let enabled =
                                    self.service.lock().unwrap().engine.debug_view.npc_hearing;
                                self.command(Command::SetDebugView {
                                    npc_vision: None,
                                    npc_hearing: Some(!enabled),
                                    noise_events: Some(!enabled),
                                    npc_routes: None,
                                });
                            }
                            KeyCode::KeyJ => {
                                let enabled =
                                    self.service.lock().unwrap().engine.debug_view.npc_routes;
                                self.command(Command::SetDebugView {
                                    npc_vision: None,
                                    npc_hearing: None,
                                    noise_events: None,
                                    npc_routes: Some(!enabled),
                                });
                            }
                            KeyCode::KeyN => {
                                let player = self
                                    .service
                                    .lock()
                                    .unwrap()
                                    .engine
                                    .scene
                                    .entities
                                    .iter()
                                    .find(|e| e.body == Some(crate::model::BodyKind::Player))
                                    .map(|e| e.id.clone());
                                if let Some(id) = player {
                                    self.command(Command::EmitNoise {
                                        origin: crate::noise::NoiseOrigin::Entity { id },
                                        radius: 12.,
                                    });
                                }
                            }
                            KeyCode::Digit1
                            | KeyCode::Digit2
                            | KeyCode::Digit3
                            | KeyCode::Digit4
                            | KeyCode::Digit5
                            | KeyCode::Digit6
                            | KeyCode::KeyX => {
                                let command = {
                                    let service = self.service.lock().unwrap();
                                    let e = &service.engine;
                                    e.scene
                                        .entities
                                        .iter()
                                        .find(|a| a.body == Some(crate::model::BodyKind::Player))
                                        .and_then(|player| {
                                            if code == KeyCode::KeyX {
                                                let entity = e
                                                    .scene
                                                    .entities
                                                    .iter()
                                                    .position(|a| a.id == player.id)
                                                    .unwrap();
                                                let slot = e.simulation.actors.slot(entity);
                                                e.simulation.abilities.state.casts[slot as usize]
                                                    .map(|c| Command::CancelAbility {
                                                        actor: player.id.clone(),
                                                        execution_id: c.id,
                                                    })
                                            } else {
                                                let slot = match code {
                                                    KeyCode::Digit1 => 1,
                                                    KeyCode::Digit2 => 2,
                                                    KeyCode::Digit3 => 3,
                                                    KeyCode::Digit4 => 4,
                                                    KeyCode::Digit5 => 5,
                                                    _ => 6,
                                                };
                                                let id = player
                                                    .abilities
                                                    .as_ref()?
                                                    .grants
                                                    .get(slot - 1)?;
                                                let definition = e
                                                    .scene
                                                    .ability_catalog
                                                    .abilities
                                                    .iter()
                                                    .find(|d| d.id == *id)?;
                                                if definition.selection
                                                    == crate::abilities::Selection::Spatial
                                                {
                                                    let [x, y] = self.cursor?;
                                                    let size = self.window.as_ref()?.inner_size();
                                                    return Some(Command::AimAbilityAt {
                                                        actor: player.id.clone(),
                                                        slot: slot as u8,
                                                        x,
                                                        y,
                                                        width: size.width,
                                                        height: size.height,
                                                        plane_y: e
                                                            .simulation
                                                            .position(&player.id)?[1],
                                                    });
                                                }
                                                Some(Command::ActivateAbility {
                                                    actor: player.id.clone(),
                                                    slot: slot as u8,
                                                    target: if definition.selection
                                                        == crate::abilities::Selection::SelfActor
                                                    {
                                                        None
                                                    } else {
                                                        e.selected_id().map(str::to_owned)
                                                    },
                                                })
                                            }
                                        })
                                };
                                if let Some(c) = command {
                                    self.command(c);
                                }
                            }
                            KeyCode::KeyR => self.command(Command::Reset),
                            KeyCode::Escape => {
                                if self.looking {
                                    self.look_capture(false);
                                    return;
                                }
                                self.command(Command::Exit);
                                event_loop.exit();
                            }
                            _ => (),
                        }
                    }
                    if changed
                        && matches!(
                            code,
                            KeyCode::KeyW
                                | KeyCode::KeyS
                                | KeyCode::KeyA
                                | KeyCode::KeyD
                                | KeyCode::ArrowUp
                                | KeyCode::ArrowDown
                                | KeyCode::ArrowLeft
                                | KeyCode::ArrowRight
                                | KeyCode::ShiftLeft
                                | KeyCode::ShiftRight
                                | KeyCode::ControlLeft
                                | KeyCode::ControlRight
                                | KeyCode::Space
                                | KeyCode::KeyC
                        )
                        && let Err(e) = self
                            .keyboard
                            .apply_keys(&mut self.service.lock().unwrap(), &self.keys)
                    {
                        eprintln!("{}: {}", e.code, e.message);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.redraw() {
                    self.error = Some(e.to_string());
                    event_loop.exit();
                }
            }
            WindowEvent::Occluded(value) => {
                self.occluded = value;
                self.service.lock().unwrap().viewport_performance.pause();
                self.title_state = None;
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut service = self.service.lock().unwrap();
        let _ = service.poll_network();
        let local_context = service.engine.player_view_context();
        let release_mouse = self.local_context != local_context;
        if release_mouse {
            self.keys.clear();
            let _ = self.keyboard.release(&mut service);
            self.local_context = local_context;
        }
        let input_context = service
            .remote_client()
            .then(|| service.remote_input_context());
        if self.input_context != input_context {
            self.keys.clear();
            let _ = self.keyboard.release(&mut service);
            self.input_context = input_context;
        }
        if service.engine.exit_requested {
            event_loop.exit();
            return;
        }
        let remote = service.remote_stamp();
        let state = remote.map(|s| (s.0, s.1, s.2, s.3, s.4, true)).unwrap_or((
            service.engine.playing,
            service.engine.simulation.tick,
            service.engine.revision,
            service.engine.view_revision,
            service.engine.input.mode,
            false,
        ));
        let measuring = service.viewport_performance.settings.enabled && !self.occluded;
        drop(service);
        if release_mouse {
            self.look_capture(false);
        }
        if !state.0 {
            self.last = Instant::now();
            self.accumulator = 0.;
        }
        // MCP can edit while paused. Poll its stamp without submitting unchanged GPU frames.
        event_loop.set_control_flow(if measuring {
            ControlFlow::Poll
        } else {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16))
        });
        if (measuring || remote.is_none() && state.0 || self.presented_state != Some(state))
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }
}

#[derive(Default)]
pub(crate) struct KeyboardControl {
    revision: Option<u64>,
    remote_control: Option<String>,
}
impl KeyboardControl {
    pub(crate) fn apply_keys(
        &mut self,
        service: &mut Service,
        keys: &BTreeSet<KeyCode>,
    ) -> crate::model::Result<()> {
        if service.engine.inspection_flight_active() {
            service.execute(
                Command::SetInspectionFlightVertical {
                    value: f32::from(u8::from(keys.contains(&KeyCode::Space)))
                        - f32::from(u8::from(keys.contains(&KeyCode::KeyC))),
                }
                .into(),
            )?;
        }
        self.apply(service, keyboard_input(keys))
    }
    pub(crate) fn apply(
        &mut self,
        service: &mut Service,
        input: Input,
    ) -> crate::model::Result<()> {
        if service.remote_client() {
            let reply = service.execute(Command::SetMultiplayerInput { input }.into())?;
            self.remote_control = reply.data["control"].as_str().map(str::to_owned);
            self.revision = None;
        } else {
            let command = if service.engine.player_view_context().is_some() {
                Command::SetPlayerViewInput {
                    input,
                    control: None,
                }
            } else {
                Command::SetInput { input }
            };
            service.execute(command.into())?;
            self.revision = Some(service.engine.input_revision);
            self.remote_control = None;
        }
        Ok(())
    }
    pub(crate) fn release(&mut self, service: &mut Service) -> crate::model::Result<()> {
        if let Some(control) = self.remote_control.take()
            && service.remote_client()
        {
            service.execute(Command::ReleaseMultiplayerInput { control }.into())?;
        }
        if self.revision.take() == Some(service.engine.input_revision) {
            if service.engine.inspection_flight_active() {
                service.execute(Command::SetInspectionFlightVertical { value: 0. }.into())?;
            }
            service.execute(
                Command::SetInput {
                    input: Input::default(),
                }
                .into(),
            )?;
        }
        Ok(())
    }
}

pub(crate) fn keyboard_input(keys: &BTreeSet<KeyCode>) -> Input {
    let held = |a, b| keys.contains(&a) || keys.contains(&b);
    Input {
        x: u8::from(held(KeyCode::KeyD, KeyCode::ArrowRight)) as f32
            - u8::from(held(KeyCode::KeyA, KeyCode::ArrowLeft)) as f32,
        z: u8::from(held(KeyCode::KeyS, KeyCode::ArrowDown)) as f32
            - u8::from(held(KeyCode::KeyW, KeyCode::ArrowUp)) as f32,
        mode: if held(KeyCode::ControlLeft, KeyCode::ControlRight) {
            crate::model::MovementMode::Sneak
        } else if held(KeyCode::ShiftLeft, KeyCode::ShiftRight) {
            crate::model::MovementMode::Run
        } else {
            crate::model::MovementMode::Walk
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MovementMode;

    #[test]
    fn modifiers_release_without_losing_movement_and_control_has_priority() {
        let mut keys = BTreeSet::from([KeyCode::KeyW, KeyCode::ShiftLeft]);
        assert_eq!(
            keyboard_input(&keys),
            Input {
                x: 0.,
                z: -1.,
                mode: MovementMode::Run
            }
        );
        keys.insert(KeyCode::ControlRight);
        assert_eq!(keyboard_input(&keys).mode, MovementMode::Sneak);
        keys.remove(&KeyCode::ControlRight);
        assert_eq!(keyboard_input(&keys).mode, MovementMode::Run);
        keys.remove(&KeyCode::ShiftLeft);
        assert_eq!(keyboard_input(&keys).mode, MovementMode::Walk);
        assert_eq!(keyboard_input(&keys).z, -1.);
        keys.insert(KeyCode::ArrowDown);
        assert_eq!(keyboard_input(&keys).z, 0.);
        keys.clear();
        assert_eq!(keyboard_input(&keys), Input::default());
    }
    #[test]
    fn focus_release_clears_only_input_still_owned_by_keyboard() {
        let dir = tempfile::tempdir().unwrap();
        let mut service = Service::new(
            dir.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        let mut keyboard = KeyboardControl::default();
        let input = Input {
            x: 1.,
            ..Default::default()
        };
        service.execute(Command::SetInput { input }.into()).unwrap();
        keyboard.release(&mut service).unwrap();
        assert_eq!(service.engine.input, input);
        keyboard.apply(&mut service, input).unwrap();
        keyboard.release(&mut service).unwrap();
        assert_eq!(service.engine.input, Input::default());
        for next in [
            input,
            Input {
                z: -1.,
                ..Default::default()
            },
        ] {
            keyboard.apply(&mut service, input).unwrap();
            service
                .execute(Command::SetInput { input: next }.into())
                .unwrap();
            let revision = service.engine.input_revision;
            keyboard.release(&mut service).unwrap();
            assert_eq!(service.engine.input, next);
            assert_eq!(service.engine.input_revision, revision);
        }
        keyboard.apply(&mut service, input).unwrap();
        service.execute(Command::Reset.into()).unwrap();
        let revision = service.engine.input_revision;
        keyboard.release(&mut service).unwrap();
        assert_eq!(service.engine.input_revision, revision);
        keyboard.apply(&mut service, input).unwrap();
        assert!(
            service
                .execute(
                    Command::SetInput {
                        input: Input {
                            x: f32::NAN,
                            ..Default::default()
                        }
                    }
                    .into()
                )
                .is_err()
        );
        keyboard.release(&mut service).unwrap();
        assert_eq!(service.engine.input, Input::default());
    }

    #[test]
    fn former_keyboard_owner_cannot_release_explicit_control() {
        let dir = tempfile::tempdir().unwrap();
        let mut service = Service::new(
            dir.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        let mut keyboard = KeyboardControl::default();
        keyboard
            .apply(
                &mut service,
                Input {
                    x: 1.,
                    ..Default::default()
                },
            )
            .unwrap();
        let control = service
            .execute(
                Command::BindControl {
                    actor: "player".into(),
                }
                .into(),
            )
            .unwrap()
            .data["control"]
            .as_str()
            .unwrap()
            .to_owned();
        service
            .execute(
                Command::SetControlInput {
                    control,
                    input: Input {
                        z: 1.,
                        ..Default::default()
                    },
                }
                .into(),
            )
            .unwrap();
        let before = service.engine.snapshot();
        keyboard.release(&mut service).unwrap();
        assert_eq!(before, service.engine.snapshot());
        assert!(
            keyboard
                .apply(
                    &mut service,
                    Input {
                        x: -1.,
                        ..Default::default()
                    }
                )
                .is_err()
        );
        assert_eq!(before, service.engine.snapshot());
    }
}
