//! A neutral UI authoring consumer. All authoring uses Command::Ui, as MCP does.
#[path = "ui_workbench/tool.rs"]
mod tool;
#[path = "ui_workbench/verify.rs"]
mod verify;
use anyhow::Result;
use ignis::{editor_ui::Graphics, render::Renderer};
use std::{
    path::Path,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
type Pending = (
    ignis::engine::Request,
    mpsc::SyncSender<(ignis::model::Result<ignis::service::Reply>, bool)>,
);
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    match std::env::args().nth(1).as_deref() {
        Some("--verify") => verify::run(root),
        Some("--help") => {
            println!(
                "ui_workbench [--verify | --mcp]\nNative authoring tool; --verify writes current GPU captures and a JSON report under artifacts/ui-workbench."
            );
            Ok(())
        }
        None | Some("--mcp") => {
            let (tx, rx) = mpsc::sync_channel::<Pending>(8);
            if std::env::args().any(|a| a == "--mcp") {
                std::thread::spawn(move || {
                    let result = ignis::mcp::serve_with(
                        std::io::stdin().lock(),
                        std::io::stdout().lock(),
                        move |request| {
                            let (reply_tx, reply_rx) = mpsc::sync_channel(1);
                            if tx.send((request, reply_tx)).is_ok()
                                && let Ok(reply) = reply_rx.recv()
                            {
                                return reply;
                            }
                            (Err(ignis::model::invalid("UI window closed")), true)
                        },
                    );
                    if let Err(e) = result {
                        eprintln!("UI MCP: {e}");
                    }
                });
            }
            let mut app = App {
                tool: tool::Tool::new(root)?,
                window: None,
                surface: None,
                config: None,
                graphics: None,
                input: None,
                error: None,
                requests: if std::env::args().any(|a| a == "--mcp") {
                    Some(rx)
                } else {
                    None
                },
                repaint: Instant::now(),
            };
            EventLoop::new()?.run_app(&mut app)?;
            if let Some(e) = app.error {
                anyhow::bail!(e);
            }
            Ok(())
        }
        Some(a) => anyhow::bail!("Unknown argument {a}"),
    }
}
struct App {
    tool: tool::Tool,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    graphics: Option<Graphics>,
    input: Option<egui_winit::State>,
    error: Option<String>,
    requests: Option<mpsc::Receiver<Pending>>,
    repaint: Instant,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let result = (|| -> Result<()> {
            let window = Arc::new(
                el.create_window(
                    Window::default_attributes()
                        .with_title("Ignis · UI Workbench")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280., 860.))
                        .with_min_inner_size(winit::dpi::LogicalSize::new(640., 480.)),
                )?,
            );
            let instance = wgpu::Instance::new(&Default::default());
            let surface = instance.create_surface(window.clone())?;
            let renderer = pollster::block_on(Renderer::new(&instance, Some(&surface)))?;
            let sz = window.inner_size();
            let mut config = surface
                .get_default_config(&renderer.adapter, sz.width.max(1), sz.height.max(1))
                .ok_or_else(|| anyhow::anyhow!("UI surface unsupported"))?;
            config.format = renderer.format;
            config.present_mode = wgpu::PresentMode::AutoVsync;
            surface.configure(&renderer.device, &config);
            let g = Graphics::new(renderer);
            let input = egui_winit::State::new(
                g.context.clone(),
                egui::ViewportId::ROOT,
                window.as_ref(),
                Some(window.scale_factor() as f32),
                window.theme(),
                None,
            );
            self.input = Some(input);
            self.graphics = Some(g);
            self.surface = Some(surface);
            self.config = Some(config);
            self.window = Some(window);
            Ok(())
        })();
        if let Err(e) = result {
            self.error = Some(e.to_string());
            el.exit();
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let response = self.input.as_mut().unwrap().on_window_event(window, &event);
        if response.repaint {
            window.request_redraw();
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(sz) if sz.width > 0 && sz.height > 0 => {
                let c = self.config.as_mut().unwrap();
                c.width = sz.width;
                c.height = sz.height;
                self.surface
                    .as_ref()
                    .unwrap()
                    .configure(&self.graphics.as_ref().unwrap().renderer.device, c);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let sz = window.inner_size();
                if sz.width == 0 || sz.height == 0 {
                    return;
                }
                let result = (|| -> Result<()> {
                    let g = self.graphics.as_mut().unwrap();
                    let raw = self.input.as_mut().unwrap().take_egui_input(window);
                    let out = self.tool.frame(g, raw);
                    self.repaint = Instant::now()
                        + out
                            .viewport_output
                            .get(&egui::ViewportId::ROOT)
                            .map(|v| v.repaint_delay)
                            .unwrap_or(Duration::from_secs(60))
                            .min(Duration::from_secs(60));
                    self.input
                        .as_mut()
                        .unwrap()
                        .handle_platform_output(window, out.platform_output.clone());
                    let surface = self.surface.as_ref().unwrap();
                    let target = match surface.get_current_texture() {
                        Ok(t) => t,
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            surface.configure(&g.renderer.device, self.config.as_ref().unwrap());
                            window.request_redraw();
                            return Ok(());
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            window.request_redraw();
                            return Ok(());
                        }
                        Err(e) => return Err(e.into()),
                    };
                    let paint_started = Instant::now();
                    g.paint(
                        out,
                        &target.texture.create_view(&Default::default()),
                        sz.width,
                        sz.height,
                    )?;
                    self.tool.draw_ms = paint_started.elapsed().as_secs_f64() * 1000.;
                    target.present();
                    Ok(())
                })();
                if let Err(e) = result {
                    self.error = Some(e.to_string());
                    el.exit();
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        while let Some(Ok((request, reply))) = self.requests.as_ref().map(|r| r.try_recv()) {
            let result = self.tool.service.execute(request);
            let exit = self.tool.service.engine.exit_requested;
            let _ = reply.send((result, exit));
            if exit {
                el.exit();
            }
            self.repaint = Instant::now();
        }
        // Poll only the bounded command mailbox; redraw when egui requests it.
        let now = Instant::now();
        el.set_control_flow(ControlFlow::WaitUntil(if self.requests.is_some() {
            (now + Duration::from_millis(33)).min(self.repaint)
        } else {
            self.repaint
        }));
        if now >= self.repaint
            && let Some(w) = &self.window
        {
            w.request_redraw();
        }
    }
}
