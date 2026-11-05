use crate::{
    characters::{Draft, Store},
    front_end::{Action, Front, Page},
    front_ui::Ui,
};
use anyhow::{Result, ensure};
use egui::{Pos2, Rect, Vec2};
use ignis::{editor_ui::Graphics, physics::Input, render::Renderer};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

pub struct Composition {
    pub graphics: Graphics,
    pub ui: Ui,
}
impl Composition {
    fn new(renderer: Renderer) -> Result<Self> {
        let graphics = Graphics::new(renderer);
        let ui = Ui::new(&graphics.context)?;
        Ok(Self { graphics, ui })
    }
    fn frame(&mut self, front: &mut Front, raw: egui::RawInput) -> egui::FullOutput {
        let mut actions = vec![];
        let ui = &mut self.ui;
        let output = self.graphics.build_custom(raw, |ctx, texture| {
            let (view, a) = ui.draw(ctx, texture, front);
            actions = a;
            view
        });
        for action in actions {
            if let Err(e) = front.execute(action) {
                front.error = e.to_string();
            }
        }
        self.graphics.update_custom_scene(front.service());
        if let Some(frame) = self.graphics.custom_frame_mut()
            && let Err(e) = front.present(frame)
        {
            front.error = e.to_string();
        }
        output
    }
    fn raw() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 960.))),
            ..Default::default()
        }
    }
    fn target(&self) -> wgpu::Texture {
        self.graphics
            .renderer
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("game UI capture"),
                size: wgpu::Extent3d {
                    width: 1440,
                    height: 960,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.graphics.renderer.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
    }
    fn input(&mut self, front: &mut Front, events: Vec<egui::Event>) -> Result<()> {
        let mut raw = Self::raw();
        raw.events = events;
        let target = self.target();
        let output = self.frame(front, raw);
        self.graphics
            .paint(output, &target.create_view(&Default::default()), 1440, 960)
    }
    fn capture(&mut self, front: &mut Front, path: &Path) -> Result<()> {
        let target = self.target();
        for _ in 0..3 {
            let output = self.frame(front, Self::raw());
            self.graphics
                .paint(output, &target.create_view(&Default::default()), 1440, 960)?;
        }
        fs::write(
            path,
            self.graphics
                .renderer
                .read_texture_png(&target, 1440, 960)?,
        )?;
        Ok(())
    }
    fn click(&mut self, front: &mut Front, id: &str) -> Result<()> {
        // Areas need their sizing passes before they enter egui's hit-test map.
        for _ in 0..3 {
            self.input(front, vec![])?;
        }
        let pos = self
            .ui
            .hits
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Button {id} missing"))?
            .center();
        self.input(front, vec![egui::Event::PointerMoved(pos)])?;
        self.input(
            front,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        )?;
        self.input(
            front,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
        )?;
        ensure!(front.error.is_empty(), "{}", front.error);
        Ok(())
    }
}
struct App {
    front: Front,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    composition: Option<Composition>,
    input: Option<egui_winit::State>,
    keys: BTreeSet<KeyCode>,
    cursor: Option<[f64; 2]>,
    captured: bool,
    last: Instant,
    error: Option<String>,
}
impl App {
    fn pointer_blocked(&self) -> bool {
        let Some([x, y]) = self.cursor else {
            return true;
        };
        let Some(window) = &self.window else {
            return true;
        };
        let Some(g) = &self.composition else {
            return true;
        };
        let scale = window.scale_factor() as f32;
        let p = egui::pos2(x as f32 / scale, y as f32 / scale);
        !g.graphics.viewport.is_some_and(|r| r.contains(p)) || g.ui.blocks_pointer(p)
    }
    fn capture_cursor(&mut self) {
        let wanted = self
            .front
            .session
            .as_ref()
            .is_some_and(|s| s.controls.dragging());
        if wanted == self.captured {
            return;
        }
        if let Some(window) = &self.window {
            let result = if wanted {
                window
                    .set_cursor_grab(CursorGrabMode::Locked)
                    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
            } else {
                window.set_cursor_grab(CursorGrabMode::None)
            };
            self.captured = wanted && result.is_ok();
            window.set_cursor_visible(!self.captured);
        }
    }
    fn look(&mut self, dx: f64, dy: f64) {
        if let Err(e) = self.front.look(dx, dy) {
            self.front.error = e.to_string();
        }
    }
}
impl ApplicationHandler for App {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device: DeviceId,
        event: DeviceEvent,
    ) {
        if self.captured
            && let DeviceEvent::MouseMotion { delta: (x, y) } = event
        {
            self.look(x, y);
        }
    }
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let result = (|| -> Result<()> {
            let window = Arc::new(
                event_loop.create_window(
                    Window::default_attributes()
                        .with_title("Land of Survivors")
                        .with_inner_size(winit::dpi::LogicalSize::new(1440., 960.))
                        .with_min_inner_size(winit::dpi::LogicalSize::new(1040., 760.)),
                )?,
            );
            let instance = wgpu::Instance::new(&Default::default());
            let surface = instance.create_surface(window.clone())?;
            let renderer = pollster::block_on(Renderer::new(&instance, Some(&surface)))?;
            let size = window.inner_size();
            let mut config = surface
                .get_default_config(&renderer.adapter, size.width.max(1), size.height.max(1))
                .ok_or_else(|| anyhow::anyhow!("Unsupported game surface"))?;
            config.format = renderer.format;
            config.present_mode = wgpu::PresentMode::AutoVsync;
            surface.configure(&renderer.device, &config);
            let composition = Composition::new(renderer)?;
            let input = egui_winit::State::new(
                composition.graphics.context.clone(),
                egui::ViewportId::ROOT,
                window.as_ref(),
                Some(window.scale_factor() as f32),
                window.theme(),
                None,
            );
            self.window = Some(window);
            self.surface = Some(surface);
            self.config = Some(config);
            self.composition = Some(composition);
            self.input = Some(input);
            self.last = Instant::now();
            Ok(())
        })();
        if let Err(e) = result {
            self.error = Some(e.to_string());
            event_loop.exit();
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };
        let _ = self
            .input
            .as_mut()
            .unwrap()
            .on_window_event(&window, &event);
        match event {
            WindowEvent::CloseRequested => {
                if let Err(e) = self.front.release_controls() {
                    self.front.error = e.to_string();
                }
                self.capture_cursor();
                match self.front.leave() {
                    Ok(()) => event_loop.exit(),
                    Err(e) => self.front.error = e.to_string(),
                }
            }
            WindowEvent::Focused(false) => {
                self.keys.clear();
                self.cursor = None;
                if let Err(e) = self.front.release_controls() {
                    self.front.error = e.to_string();
                }
                self.capture_cursor();
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !self.captured
                    && let Some([x, y]) = self.cursor
                {
                    let scale = window.scale_factor();
                    self.look((position.x - x) / scale, (position.y - y) / scale);
                }
                self.cursor = Some([position.x, position.y]);
            }
            WindowEvent::CursorLeft { .. } => {
                if !self.captured {
                    self.cursor = None;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let blocked = self.pointer_blocked();
                if let Some(s) = &mut self.front.session {
                    s.controls
                        .button(button, state == ElementState::Pressed, blocked);
                }
                self.capture_cursor();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if self.front.session.is_some() && !self.pointer_blocked() {
                    let steps = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.,
                    };
                    if let Err(e) = self.front.zoom_camera(steps) {
                        self.front.error = e.to_string();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.keys.insert(code);
                    } else {
                        self.keys.remove(&code);
                    }
                    if code == KeyCode::Escape
                        && event.state == ElementState::Pressed
                        && !event.repeat
                    {
                        if self
                            .front
                            .session
                            .as_ref()
                            .is_some_and(|s| s.controls.dragging())
                        {
                            if let Err(e) = self.front.release_controls() {
                                self.front.error = e.to_string();
                            }
                            self.capture_cursor();
                            return;
                        }
                        let action = if self.front.page == Page::Session {
                            Some(Action::Leave)
                        } else if matches!(self.front.page, Page::Create | Page::Cosmetics) {
                            Some(Action::Cancel)
                        } else {
                            None
                        };
                        if let Some(action) = action
                            && let Err(e) = self.front.execute(action)
                        {
                            self.front.error = e.to_string();
                        }
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    let config = self.config.as_mut().unwrap();
                    config.width = size.width;
                    config.height = size.height;
                    self.surface.as_ref().unwrap().configure(
                        &self.composition.as_ref().unwrap().graphics.renderer.device,
                        config,
                    );
                }
            }
            WindowEvent::RedrawRequested => {
                let result = (|| -> Result<()> {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return Ok(());
                    }
                    let elapsed = self.last.elapsed().as_secs_f32().min(0.1);
                    self.last = Instant::now();
                    let g = self.composition.as_mut().unwrap();
                    let wants = g.graphics.context.wants_keyboard_input();
                    if wants {
                        self.keys.clear();
                    }
                    let held = |key| !wants && self.keys.contains(&key);
                    let input = Input {
                        x: i32::from(held(KeyCode::KeyD)) as f32
                            - i32::from(held(KeyCode::KeyA)) as f32,
                        z: i32::from(held(KeyCode::KeyS)) as f32
                            - i32::from(held(KeyCode::KeyW)) as f32,
                        mode: if held(KeyCode::ShiftLeft) || held(KeyCode::ShiftRight) {
                            ignis::model::MovementMode::Run
                        } else {
                            ignis::model::MovementMode::Walk
                        },
                    };
                    if let Err(e) = self.front.tick(elapsed, input) {
                        self.front.error = e.to_string();
                    }
                    let raw = self.input.as_mut().unwrap().take_egui_input(&window);
                    let output = g.frame(&mut self.front, raw);
                    self.input
                        .as_mut()
                        .unwrap()
                        .handle_platform_output(&window, output.platform_output.clone());
                    let surface = self.surface.as_ref().unwrap();
                    let frame = match surface.get_current_texture() {
                        Ok(f) => f,
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            surface.configure(
                                &g.graphics.renderer.device,
                                self.config.as_ref().unwrap(),
                            );
                            return Ok(());
                        }
                        Err(wgpu::SurfaceError::Timeout) => return Ok(()),
                        Err(e) => return Err(e.into()),
                    };
                    g.graphics.paint(
                        output,
                        &frame.texture.create_view(&Default::default()),
                        size.width,
                        size.height,
                    )?;
                    frame.present();
                    Ok(())
                })();
                if let Err(e) = result {
                    self.front.error = e.to_string();
                }
                if self.front.session.is_none() {
                    self.keys.clear();
                }
                self.capture_cursor();
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now()
                + Duration::from_millis(if self.front.page == Page::Session {
                    16
                } else {
                    33
                }),
        ));
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
fn verify(root: &Path, out: &Path) -> Result<()> {
    fs::create_dir_all(out)?;
    let dir = tempfile::tempdir()?;
    let profile = dir.path().join("characters.json");
    let mut f = Front::new(root.into(), profile.clone())?;
    let instance = wgpu::Instance::new(&Default::default());
    let renderer = pollster::block_on(Renderer::new(&instance, None))?;
    let mut graphics = Composition::new(renderer)?;
    graphics.capture(&mut f, &out.join("title.png"))?;
    graphics.click(&mut f, "begin")?;
    ensure!(f.page == Page::Roster, "Characters list did not open");
    graphics.capture(&mut f, &out.join("empty-roster.png"))?;
    graphics.click(&mut f, "new")?;
    ensure!(f.page == Page::Create, "New character did not open");
    graphics.click(&mut f, "name")?;
    graphics.input(&mut f, vec![egui::Event::Text("Aelara".into())])?;
    graphics.capture(&mut f, &out.join("creator.png"))?;
    graphics.click(&mut f, "confirm")?;
    ensure!(f.page == Page::Roster, "Confirmation failed: {}", f.error);
    let id = f.selected().unwrap().id.clone();
    let before = fs::read(&profile)?;
    graphics.click(&mut f, "new")?;
    graphics.click(&mut f, "cancel")?;
    ensure!(fs::read(&profile)? == before, "Cancel mutated roster");
    graphics.capture(&mut f, &out.join("characters.png"))?;
    let mut reopened = Front::new(root.into(), profile.clone())?;
    ensure!(
        reopened.selected().unwrap().id == id,
        "Identity changed on reopen"
    );
    reopened.execute(Action::Offline)?;
    for _ in 0..30 {
        reopened.tick(1. / 60., Input::default())?;
    }
    graphics.capture(&mut reopened, &out.join("character-in-scene.png"))?;
    let initial_vertices = graphics
        .graphics
        .custom_frame_mut()
        .unwrap()
        .meshes
        .vertices
        .len();
    ensure!(initial_vertices > 1000, "Player character was not drawn");
    {
        let frame = graphics.graphics.custom_frame_mut().unwrap();
        ensure!(
            frame.boxes.len() == 1,
            "Player box was not replaced or the floor was removed"
        );
        let s = reopened.session.as_ref().unwrap();
        ensure!(
            s.client.engine.scene.meshes.is_empty(),
            "Presentation contaminated the saved world"
        );
        let placed = &s.visuals.placements[0];
        let center = s.client.engine.simulation.position("player0").unwrap();
        ensure!(
            placed.center == center
                && placed.actor > 0
                && placed.life == s.client.engine.simulation.vitality.state.lives[0]
                && placed.selected_model,
            "Visual is not bound to the selected physical actor"
        );
        let foot = frame
            .meshes
            .vertices
            .iter()
            .map(|v| v.position[1])
            .fold(f32::INFINITY, f32::min);
        ensure!(
            (foot - placed.foot_y).abs() < 0.00001,
            "Character feet do not match the collider base"
        );
    }
    // The exact native camera/input path: left drag orbits without steering,
    // right drag steers in place, and UI/focus boundaries reject stale capture.
    let position_before_look = reopened
        .session
        .as_ref()
        .unwrap()
        .client
        .engine
        .simulation
        .positions();
    let camera_before_look = graphics.graphics.custom_frame_mut().unwrap().camera.clone();
    let heading_before = reopened.session.as_ref().unwrap().controls.heading_degrees;
    reopened
        .session
        .as_mut()
        .unwrap()
        .controls
        .button(MouseButton::Left, true, false);
    reopened.look(350., 50.)?;
    graphics.capture(&mut reopened, &out.join("camera-orbit.png"))?;
    ensure!(
        reopened
            .session
            .as_ref()
            .unwrap()
            .client
            .engine
            .simulation
            .positions()
            == position_before_look,
        "Left orbit moved the physical character"
    );
    ensure!(
        reopened.session.as_ref().unwrap().visuals.placements[0].yaw_degrees == heading_before,
        "Left orbit rotated the character"
    );
    ensure!(
        graphics
            .graphics
            .custom_frame_mut()
            .unwrap()
            .camera
            .position
            != camera_before_look.position,
        "Left orbit did not move the camera"
    );
    reopened
        .session
        .as_mut()
        .unwrap()
        .controls
        .button(MouseButton::Left, false, false);
    reopened
        .session
        .as_mut()
        .unwrap()
        .controls
        .button(MouseButton::Right, true, false);
    reopened.look(250., -25.)?;
    graphics.capture(&mut reopened, &out.join("character-turn.png"))?;
    {
        let session = reopened.session.as_ref().unwrap();
        ensure!(
            session.client.engine.simulation.positions() == position_before_look,
            "Right turn displaced the physical character"
        );
        ensure!(
            (session.visuals.placements[0].yaw_degrees - session.controls.view.yaw_degrees).abs()
                < 0.001,
            "Right drag did not align the character"
        );
        ensure!(
            graphics
                .graphics
                .custom_frame_mut()
                .unwrap()
                .camera
                .position
                == session.client.engine.player_camera().unwrap().position,
            "Offline camera bypassed the native physical camera"
        );
    }
    let hud = graphics.ui.hits["leave"].center();
    ensure!(
        graphics.ui.blocks_pointer(hud),
        "HUD does not intercept camera input"
    );
    reopened.release_controls()?;
    ensure!(
        !reopened.session.as_ref().unwrap().controls.dragging(),
        "Focus release kept the mouse captured"
    );
    reopened.zoom_camera(2.)?;
    ensure!(
        reopened
            .session
            .as_ref()
            .unwrap()
            .controls
            .view
            .third_person
            .distance_meters
            == 3.,
        "Wheel did not zoom"
    );
    reopened.session.as_mut().unwrap().controls = Default::default();
    reopened.sync_camera()?;
    graphics.capture(&mut reopened, &out.join("idle-a.png"))?;
    let idle_a: Vec<_> = graphics
        .graphics
        .custom_frame_mut()
        .unwrap()
        .meshes
        .vertices
        .iter()
        .step_by(7)
        .map(|v| v.position)
        .collect();
    for _ in 0..20 {
        reopened.tick(1. / 60., Input::default())?;
    }
    graphics.capture(&mut reopened, &out.join("idle-b.png"))?;
    ensure!(
        graphics
            .graphics
            .custom_frame_mut()
            .unwrap()
            .meshes
            .vertices
            .iter()
            .step_by(7)
            .zip(idle_a)
            .any(
                |(b, a)| glam::Vec3::from_array(b.position).distance(glam::Vec3::from_array(a))
                    > 0.0001
            ),
        "Idle clip did not animate the skeleton"
    );
    let mut concurrent = Front::new(root.into(), profile.clone())?;
    ensure!(
        concurrent.execute(Action::Offline).is_err(),
        "Concurrent offline save writer was admitted"
    );
    drop(concurrent);
    let before = reopened
        .session
        .as_ref()
        .unwrap()
        .client
        .engine
        .simulation
        .positions();
    for step in 0..120 {
        reopened.tick(
            1. / 60.,
            Input {
                x: 1.,
                z: 0.,
                mode: ignis::model::MovementMode::Walk,
            },
        )?;
        if step % 10 == 0 {
            graphics.input(&mut reopened, vec![])?;
        }
    }
    let mut after = reopened
        .session
        .as_ref()
        .unwrap()
        .client
        .engine
        .simulation
        .positions();
    ensure!(before != after, "Offline movement did not execute");
    graphics.capture(&mut reopened, &out.join("character-moving.png"))?;
    {
        let frame = graphics.graphics.custom_frame_mut().unwrap();
        ensure!(
            frame.meshes.vertices.len() == initial_vertices,
            "Visual geometry accumulated across frames"
        );
        let position = reopened
            .session
            .as_ref()
            .unwrap()
            .client
            .engine
            .simulation
            .position("player0")
            .unwrap();
        ensure!(
            reopened.session.as_ref().unwrap().visuals.placements[0].center == position,
            "Character did not follow movement"
        );
        let session = reopened.session.as_ref().unwrap();
        ensure!(
            frame.camera.position == session.client.engine.player_camera().unwrap().position,
            "Camera did not follow through the native view"
        );
        ensure!(
            session.visuals.placements[0].gait == crate::animation::Gait::Walk,
            "Walking did not select the walk clip"
        );
        ensure!(
            position[0] > before["player0"][0],
            "World movement direction changed unexpectedly"
        );
    }
    let walk_speed = (after["player0"][0] - before["player0"][0]).abs() / 2.;
    let walk_tick = reopened.session.as_ref().unwrap().visuals.placements[0].clip_tick;
    let run_start = after["player0"];
    for step in 0..60 {
        reopened.tick(
            1. / 60.,
            Input {
                x: 1.,
                z: 0.,
                mode: ignis::model::MovementMode::Run,
            },
        )?;
        if step % 10 == 0 {
            graphics.input(&mut reopened, vec![])?;
        }
    }
    graphics.capture(&mut reopened, &out.join("character-running.png"))?;
    {
        let session = reopened.session.as_ref().unwrap();
        let position = session
            .client
            .engine
            .simulation
            .position("player0")
            .unwrap();
        ensure!(
            (position[0] - run_start[0]).abs() > walk_speed * 1.35,
            "Shift did not increase physical speed"
        );
        ensure!(
            session.visuals.placements[0].gait == crate::animation::Gait::Run,
            "Running did not select the run clip"
        );
        ensure!(
            session.visuals.placements[0].clip_tick != walk_tick,
            "Locomotion phase did not advance"
        );
    }
    reopened.release_controls()?;
    for step in 0..30 {
        reopened.tick(1. / 60., Input::default())?;
        if step % 5 == 0 {
            graphics.input(&mut reopened, vec![])?;
        }
    }
    graphics.capture(&mut reopened, &out.join("character-stopped.png"))?;
    ensure!(
        reopened.session.as_ref().unwrap().visuals.placements[0].gait
            == crate::animation::Gait::Idle,
        "Stopping did not return to idle"
    );
    after = reopened
        .session
        .as_ref()
        .unwrap()
        .client
        .engine
        .simulation
        .positions();
    reopened.leave()?;
    ensure!(
        reopened.selected_save_path().unwrap().is_file(),
        "Character save missing"
    );
    reopened.execute(Action::Offline)?;
    let restored = reopened
        .session
        .as_ref()
        .unwrap()
        .client
        .engine
        .simulation
        .positions();
    ensure!(
        after == restored,
        "Offline character position was not restored"
    );
    reopened.leave()?;
    f.bind = "127.0.0.1:0".into();
    f.execute(Action::Host)?;
    let mut peer = Front::new(root.into(), dir.path().join("peer/characters.json"))?;
    let d = Draft {
        name: "Gorak".into(),
        people: crate::characters::People::Orc,
        body: crate::characters::Body::Masculine,
        ..Default::default()
    };
    let hash = peer.source_hash.clone();
    peer.store_mut()?.create(d, hash)?;
    let started = Instant::now();
    while f.session.as_ref().unwrap().invitation.is_empty()
        && started.elapsed() < Duration::from_secs(8)
    {
        f.tick(0.016, Input::default())?;
        std::thread::sleep(Duration::from_millis(10));
    }
    peer.invitation = f.session.as_ref().unwrap().invitation.clone();
    ensure!(!peer.invitation.is_empty(), "Host did not start");
    peer.execute(Action::Join)?;
    let start = Instant::now();
    let mut connected = false;
    while start.elapsed() < Duration::from_secs(10) {
        f.tick(0.016, Input::default())?;
        peer.tick(0.016, Input::default())?;
        if f.session.as_ref().unwrap().status["state"] == "connected"
            && peer.session.as_ref().unwrap().status["state"] == "connected"
        {
            connected = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    ensure!(
        connected,
        "Two real QUIC clients did not connect: {} / {}",
        f.session.as_ref().unwrap().status,
        peer.session.as_ref().unwrap().status
    );
    ensure!(
        f.session.as_ref().unwrap().status["actor"]
            != peer.session.as_ref().unwrap().status["actor"],
        "Clients share an actor lease"
    );
    let before_network = f
        .session
        .as_ref()
        .unwrap()
        .host
        .as_ref()
        .unwrap()
        .engine
        .simulation
        .positions();
    for _ in 0..120 {
        f.tick(0.016, Input::default())?;
        peer.tick(
            0.016,
            Input {
                x: 1.,
                z: 0.,
                mode: ignis::model::MovementMode::Walk,
            },
        )?;
        std::thread::sleep(Duration::from_millis(10));
    }
    let after_network = f
        .session
        .as_ref()
        .unwrap()
        .host
        .as_ref()
        .unwrap()
        .engine
        .simulation
        .positions();
    ensure!(
        after_network.iter().any(|(id, p)| before_network
            .get(id)
            .is_some_and(|old| (p[0] - old[0]).abs() > 0.5)),
        "Remote movement was not applied by authority"
    );
    graphics.capture(&mut f, &out.join("multiplayer.png"))?;
    ensure!(
        graphics.graphics.custom_frame_mut().unwrap().boxes.len() == 1,
        "Network Players still use primitive visuals"
    );
    {
        let session = f.session.as_ref().unwrap();
        let local = session.status["actor"]["actor"].as_u64().unwrap();
        ensure!(
            session.visuals.placements.len() == 8,
            "Network player mapping is incomplete"
        );
        ensure!(
            session
                .visuals
                .placements
                .iter()
                .filter(|p| p.selected_model)
                .count()
                == 1
                && session
                    .visuals
                    .placements
                    .iter()
                    .any(|p| p.actor == local && p.selected_model),
            "Selected model was assigned to the wrong network actor"
        );
    }
    graphics.capture(&mut peer, &out.join("multiplayer-peer.png"))?;
    {
        let session = peer.session.as_ref().unwrap();
        let local = session.status["actor"]["actor"].as_u64().unwrap();
        let placed = session
            .visuals
            .placements
            .iter()
            .find(|p| p.selected_model)
            .unwrap();
        ensure!(
            placed.actor == local && session.visuals.placements.len() == 8,
            "Peer model was assigned to the wrong actor"
        );
        let authority = &f
            .session
            .as_ref()
            .unwrap()
            .host
            .as_ref()
            .unwrap()
            .engine
            .simulation;
        let slot = *authority
            .actors
            .order()
            .iter()
            .find(|&&slot| authority.actors.instance_order(slot) == local)
            .unwrap();
        let entity = authority.actors.entity(slot);
        let name = &authority.definitions().entities[entity].id;
        ensure!(
            (placed.center[0] - after_network[name][0]).abs() < 0.25,
            "Received model did not follow authoritative movement"
        );
    }
    peer.leave()?;
    f.leave()?;
    fs::write(
        out.join("verification.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"native_ui_click_flow":true,"cancel_preserves_file":true,"identity_reopen":true,"offline_movement_and_portable_save":true,"concurrent_save_writer_rejected":true,"real_quic_clients":2,"remote_movement_authoritative":true,"logo_original":true,"native_player_model":true,"collider_foot_alignment":true,"model_follows_movement":true,"camera_follows_player":true,"visuals_excluded_from_saves":true,"network_model_lease_binding":true,"native_idle_walk_run":true,"native_rig_rest_shape_preserved":true,"left_orbit_preserves_heading_and_position":true,"right_drag_turns_in_place":true,"native_camera_collision_path":true,"wheel_zoom":true,"shift_increases_physical_speed":true,"focus_releases_controls":true,"ui_blocks_camera":true,"human_acceptance":false}),
        )?,
    )?;
    println!("Native creator, durable characters, offline save and two QUIC clients verified.");
    Ok(())
}
pub fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--verify") {
        return verify(&root, &root.join("games/land-of-survivors/artifacts/app"));
    }
    let mut front = Front::new(root, Store::default_path())?;
    if args.iter().any(|a| a == "--play") {
        if front.selected().is_some() {
            front.execute(Action::Offline)?;
        } else {
            front.execute(Action::New)?;
        }
    }
    let mut app = App {
        front,
        window: None,
        surface: None,
        config: None,
        composition: None,
        input: None,
        keys: BTreeSet::new(),
        cursor: None,
        captured: false,
        last: Instant::now(),
        error: None,
    };
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;
    if let Some(e) = app.error {
        anyhow::bail!(e);
    }
    Ok(())
}
