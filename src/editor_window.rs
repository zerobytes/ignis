//! Native editor host. The legacy playable window remains independent.
use crate::{
    editor_ui::Graphics, engine::Command, model::FIXED_DT, render::Renderer, service::Service,
    window::KeyboardControl,
};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

type ViewKey = (u64, u64, u64, u64, bool, u64);
pub fn run(service: Arc<Mutex<Service>>) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App {
        service,
        window: None,
        surface: None,
        graphics: None,
        input: None,
        config: None,
        keys: BTreeSet::new(),
        keyboard: KeyboardControl::default(),
        game_focus: false,
        cursor: None,
        last: Instant::now(),
        accumulator: 0.,
        presented: None,
        dirty: true,
        repaint: Instant::now(),
        occluded: false,
        unlimited: wgpu::PresentMode::Fifo,
        error: None,
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
    graphics: Option<Graphics>,
    input: Option<egui_winit::State>,
    config: Option<wgpu::SurfaceConfiguration>,
    keys: BTreeSet<KeyCode>,
    keyboard: KeyboardControl,
    game_focus: bool,
    cursor: Option<egui::Pos2>,
    last: Instant,
    accumulator: f32,
    presented: Option<ViewKey>,
    dirty: bool,
    repaint: Instant,
    occluded: bool,
    unlimited: wgpu::PresentMode,
    error: Option<String>,
}
fn view_key(service: &Service) -> ViewKey {
    (
        service.engine.revision,
        service.engine.view_revision,
        service.editor.revision,
        service.engine.simulation.tick,
        service.engine.playing,
        service.session_revision,
    )
}
impl App {
    fn release(&mut self) {
        if let Some(graphics) = &mut self.graphics {
            graphics.cancel_sculpt();
        }
        self.keys.clear();
        self.game_focus = false;
        let _ = self.keyboard.release(&mut self.service.lock().unwrap());
    }
    fn redraw(&mut self) -> anyhow::Result<()> {
        let window = self.window.as_ref().unwrap();
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            self.service.lock().unwrap().viewport_performance.pause();
            return Ok(());
        }
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f32().min(0.1);
        self.last = now;
        let graphics = self.graphics.as_mut().unwrap();
        let raw = self.input.as_mut().unwrap().take_egui_input(window);
        let (output, commands) = {
            let mut s = self.service.lock().unwrap();
            if !s.engine.playing {
                let _ = s.poll_network();
            }
            if s.engine.playing {
                self.accumulator += elapsed;
                while self.accumulator >= FIXED_DT {
                    if let Err(e) = s.advance(1) {
                        s.engine.playing = false;
                        graphics.message(e.message);
                        break;
                    }
                    self.accumulator -= FIXED_DT;
                }
            } else {
                self.accumulator = 0.;
                self.keys.clear();
                self.keyboard.release(&mut s)?;
            }
            let result = graphics.build(&s, raw);
            self.presented = Some(view_key(&s));
            result
        };
        self.repaint = now
            + output
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .map(|v| v.repaint_delay)
                .unwrap_or(Duration::from_secs(60))
                .min(Duration::from_secs(60));
        if let Some(viewport) = output.viewport_output.get(&egui::ViewportId::ROOT) {
            for command in &viewport.commands {
                match command {
                    egui::ViewportCommand::StartDrag => {
                        let _ = window.drag_window();
                    }
                    egui::ViewportCommand::Minimized(value) => window.set_minimized(*value),
                    egui::ViewportCommand::Maximized(value) => window.set_maximized(*value),
                    _ => {}
                }
            }
        }
        self.input
            .as_mut()
            .unwrap()
            .handle_platform_output(window, output.platform_output.clone());
        let config = self.config.as_mut().unwrap();
        let surface = self.surface.as_ref().unwrap();
        let settings = self.service.lock().unwrap().viewport_performance.settings;
        let desired = if !settings.enabled || settings.vsync {
            wgpu::PresentMode::Fifo
        } else {
            self.unlimited
        };
        if config.present_mode != desired {
            config.present_mode = desired;
            surface.configure(&graphics.renderer.device, config);
        }
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                surface.configure(&graphics.renderer.device, config);
                self.dirty = true;
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        graphics.paint(
            output,
            &frame.texture.create_view(&Default::default()),
            size.width,
            size.height,
        )?;
        frame.present();
        if settings.enabled && !self.occluded {
            let mut s = self.service.lock().unwrap();
            let optimized = s
                .engine
                .scene
                .light
                .atmosphere
                .is_some_and(|a| a.reduce_details);
            let mode = match config.present_mode {
                wgpu::PresentMode::Immediate => "Immediate",
                wgpu::PresentMode::Mailbox => "Mailbox",
                _ => "Fifo (VSync)",
            };
            if s.viewport_performance.presented(
                Instant::now(),
                now.elapsed().as_secs_f64() * 1000.,
                graphics.viewport_size(),
                mode,
                optimized,
            ) {
                s.viewport_performance.sample.renderer = graphics.renderer.info();
            }
        }
        self.dirty = false;
        if !commands.is_empty() {
            let mut service = self.service.lock().unwrap();
            let mut expected = self.presented.map(|key| key.0);
            for command in commands {
                let label = match &command {
                    Command::EditEditor { .. } | Command::EditSceneLive { .. } => None,
                    Command::Upsert { .. } => Some("Objeto atualizado"),
                    Command::SelectAt { .. }
                    | Command::SelectEditorAt { .. }
                    | Command::SelectEditorMeshRegion { .. }
                    | Command::SelectEntity { .. } => Some("Seleção atualizada"),
                    Command::SaveScene { .. } => Some("Cena salva"),
                    Command::LoadScene { .. } => Some("Cena carregada"),
                    Command::SaveEditor { .. } => Some("Layout salvo"),
                    Command::LoadEditor { .. } => Some("Layout carregado"),
                    Command::Undo => Some("Edição desfeita"),
                    Command::Redo => Some("Edição refeita"),
                    Command::Play => Some("Simulação em execução"),
                    Command::Pause => Some("Simulação pausada"),
                    Command::Reset => Some("Simulação reiniciada"),
                    _ => Some("Concluído"),
                };
                match service.execute(crate::engine::Request {
                    command,
                    expected_revision: expected,
                }) {
                    Ok(reply) => {
                        graphics.command_result(reply.data);
                        expected = Some(service.engine.revision);
                        if let Some(label) = label {
                            graphics.message(label.into());
                        }
                    }
                    Err(error) => {
                        graphics.command_error(format!("{}: {}", error.code, error.message));
                        break;
                    }
                }
            }
            self.dirty = true;
        }
        Ok(())
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let init = (|| -> anyhow::Result<()> {
            let attributes = Window::default_attributes()
                .with_title("Ignis — Editor")
                .with_inner_size(winit::dpi::LogicalSize::new(1440., 900.))
                .with_min_inner_size(winit::dpi::LogicalSize::new(900., 600.));
            #[cfg(target_os = "macos")]
            let attributes = {
                use winit::platform::macos::WindowAttributesExtMacOS;
                attributes
                    .with_titlebar_transparent(true)
                    .with_title_hidden(true)
                    .with_fullsize_content_view(true)
            };
            let window = Arc::new(event_loop.create_window(attributes)?);
            let instance = wgpu::Instance::new(&Default::default());
            let surface = instance.create_surface(window.clone())?;
            let renderer = pollster::block_on(Renderer::new(&instance, Some(&surface)))?;
            let size = window.inner_size();
            let mut config = surface
                .get_default_config(&renderer.adapter, size.width.max(1), size.height.max(1))
                .ok_or_else(|| anyhow::anyhow!("Unsupported editor surface"))?;
            config.format = renderer.format;
            let modes = surface.get_capabilities(&renderer.adapter).present_modes;
            self.unlimited = if modes.contains(&wgpu::PresentMode::Immediate) {
                wgpu::PresentMode::Immediate
            } else if modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            };
            config.present_mode = wgpu::PresentMode::AutoVsync;
            surface.configure(&renderer.device, &config);
            let graphics = Graphics::new(renderer);
            let input = egui_winit::State::new(
                graphics.context.clone(),
                egui::ViewportId::ROOT,
                window.as_ref(),
                Some(window.scale_factor() as f32),
                window.theme(),
                None,
            );
            self.window = Some(window);
            self.surface = Some(surface);
            self.graphics = Some(graphics);
            self.input = Some(input);
            self.config = Some(config);
            self.last = Instant::now();
            Ok(())
        })();
        if let Err(error) = init {
            self.error = Some(error.to_string());
            event_loop.exit();
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = &self.window else {
            return;
        };
        let response = self.input.as_mut().unwrap().on_window_event(window, &event);
        if response.repaint {
            self.dirty = true;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.release();
                let service = self.service.lock().unwrap();
                if service.scene_dirty() {
                    self.graphics.as_mut().unwrap().request_close(&service);
                    self.dirty = true;
                } else {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    let config = self.config.as_mut().unwrap();
                    config.width = size.width;
                    config.height = size.height;
                    self.surface
                        .as_ref()
                        .unwrap()
                        .configure(&self.graphics.as_ref().unwrap().renderer.device, config);
                    self.dirty = true;
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => self.dirty = true,
            WindowEvent::Occluded(value) => {
                self.occluded = value;
                self.service.lock().unwrap().viewport_performance.pause();
                self.dirty = true;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.graphics.as_ref().unwrap().context.pixels_per_point();
                self.cursor = Some(egui::pos2(
                    position.x as f32 / scale,
                    position.y as f32 / scale,
                ));
            }
            WindowEvent::CursorLeft { .. } => self.cursor = None,
            WindowEvent::Focused(false) => self.release(),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let graphics = self.graphics.as_ref().unwrap();
                let in_view = self
                    .cursor
                    .is_some_and(|p| graphics.viewport.is_some_and(|v| v.contains(p)));
                if !in_view {
                    self.release();
                } else {
                    self.game_focus = true;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if self.service.lock().unwrap().remote_client() {
                    // The authoring viewport still shows the local document.
                    // Only the playable window routes keys to the remote world.
                    self.release();
                    return;
                }
                if let PhysicalKey::Code(code) = event.physical_key {
                    let down = event.state == ElementState::Pressed;
                    let graphics = self.graphics.as_ref().unwrap();
                    let viewport_focus = graphics
                        .context
                        .memory(|m| m.focused() == Some(egui::Id::new("viewport_image")));
                    let eligible = self.game_focus
                        && !graphics.project_dialog_open()
                        && (!graphics.context.wants_keyboard_input() || viewport_focus)
                        && (!response.consumed || viewport_focus)
                        && self.service.lock().unwrap().engine.playing;
                    let movement = matches!(
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
                    );
                    let changed = if down && eligible && movement {
                        self.keys.insert(code)
                    } else if !down {
                        self.keys.remove(&code)
                    } else {
                        false
                    };
                    if changed {
                        let _ = self
                            .keyboard
                            .apply_keys(&mut self.service.lock().unwrap(), &self.keys);
                    }
                    if down && code == KeyCode::Escape {
                        self.release();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.redraw() {
                    self.error = Some(error.to_string());
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let service = self.service.lock().unwrap();
        if service.engine.exit_requested {
            event_loop.exit();
            return;
        }
        let now = Instant::now();
        let changed = self.presented != Some(view_key(&service));
        let measuring = service.viewport_performance.settings.enabled && !self.occluded;
        if (!self.occluded || service.engine.playing)
            && (measuring || service.engine.playing || changed || self.dirty || now >= self.repaint)
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
        event_loop.set_control_flow(if measuring {
            ControlFlow::Poll
        } else {
            ControlFlow::WaitUntil(now + Duration::from_millis(16))
        });
    }
}
