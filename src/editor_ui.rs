//! Shared native/offscreen editor composition; all mutations leave through Command.
use crate::{
    editor::{Action, Area, Axis, Panel},
    engine::Command,
    model::Entity,
    render::{RenderFrame, Renderer},
    service::Service,
};
use egui::{Color32, Pos2, Rect, Vec2};
use std::collections::VecDeque;

#[path = "editor_area_ui.rs"]
mod area_ui;

#[path = "editor_authoring_ui.rs"]
mod authoring_ui;
#[path = "editor_botany_ui.rs"]
mod botany_ui;
#[path = "editor_character_gizmo.rs"]
mod character_gizmo;
#[path = "editor_character_ui.rs"]
mod character_ui;
#[path = "editor_gizmo_ui.rs"]
mod gizmo_ui;
#[path = "editor_lighting_ui.rs"]
mod lighting_ui;
#[path = "editor_mesh_ui.rs"]
mod mesh_ui;
#[path = "editor_modeling_ui.rs"]
mod modeling_ui;
#[path = "editor_particles_ui.rs"]
mod particles_ui;
#[path = "editor_project_ui.rs"]
mod project_ui;
#[path = "editor_wind_ui.rs"]
mod wind_ui;

#[path = "editor_design_ui.rs"]
mod design_ui;
use design_ui::{BG, EMBER, LINE, PANEL};

#[derive(Clone)]
struct DragTab {
    from: u32,
    panel: Panel,
}
#[derive(Default)]
struct UiState {
    wind: wind_ui::State,
    character: character_ui::State,
    botany: botany_ui::State,
    particles: particles_ui::State,
    character_gizmo: character_gizmo::State,
    authoring: authoring_ui::State,
    project: project_ui::State,
    mesh: mesh_ui::State,
    gizmo: gizmo_ui::State,
    draft: Option<Entity>,
    draft_key: Option<(String, u64)>,
    draft_attempt: Option<Entity>,
    search: String,
    path: String,
    lod_path: String,
    file_action: Option<&'static str>,
    workspace_name: String,
    workspace_dialog: bool,
    rename: bool,
    messages: VecDeque<String>,
}
impl UiState {
    fn message(&mut self, text: String) {
        if self.messages.len() == 64 {
            self.messages.pop_front();
        }
        self.messages.push_back(text);
    }
}
pub struct Graphics {
    pub renderer: Renderer,
    pub context: egui::Context,
    painter: egui_wgpu::Renderer,
    viewport_texture: wgpu::Texture,
    viewport_id: egui::TextureId,
    pub viewport: Option<Rect>,
    ui: UiState,
    frame: Option<RenderFrame>,
}
impl Graphics {
    pub fn viewport_size(&self) -> [u32; 2] {
        [
            self.viewport_texture.width(),
            self.viewport_texture.height(),
        ]
    }
    pub fn offscreen() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&Default::default());
        Ok(Self::new(pollster::block_on(Renderer::new(
            &instance, None,
        ))?))
    }
    pub fn new(renderer: Renderer) -> Self {
        let context = egui::Context::default();
        design_ui::configure(&context);
        let mut painter = egui_wgpu::Renderer::new(
            &renderer.device,
            renderer.format,
            egui_wgpu::RendererOptions::PREDICTABLE,
        );
        let texture = viewport_texture(&renderer, 64, 64);
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(renderer.format.remove_srgb_suffix()),
            ..Default::default()
        });
        let viewport_id =
            painter.register_native_texture(&renderer.device, &view, wgpu::FilterMode::Linear);
        Self {
            renderer,
            context,
            painter,
            viewport_texture: texture,
            viewport_id,
            viewport: None,
            ui: UiState::default(),
            frame: None,
        }
    }
    pub fn cancel_sculpt(&mut self) {
        self.ui.character_gizmo.cancel();
        self.ui.mesh.cancel();
        self.ui.gizmo.cancel();
        self.context.request_repaint();
    }
    pub fn request_close(&mut self, service: &Service) {
        self.cancel_sculpt();
        if service.scene_dirty() {
            self.ui.project.pending = Some(Command::Exit);
            self.context.request_repaint();
        }
    }
    pub fn command_result(&mut self, data: serde_json::Value) {
        project_ui::command_result(&mut self.ui.project, &data);
        if self.ui.authoring.awaiting {
            self.ui.authoring.result = Some(data);
            self.ui.authoring.awaiting = false;
        }
    }
    pub fn message(&mut self, text: String) {
        project_ui::error(&mut self.ui.project, &text);
        if self.ui.authoring.awaiting {
            self.ui.authoring.error = text.clone();
            self.ui.authoring.awaiting = false;
        }
        self.ui.message(text);
    }
    pub fn command_error(&mut self, text: String) {
        self.ui.mesh.command_error(&text);
        self.ui.gizmo.cancel();
        self.ui.mesh.modeling.cancel();
        self.message(text);
    }
    pub fn project_dialog_open(&self) -> bool {
        self.ui.project.modal()
    }
    /// Compose a game's UI with the native scene renderer, without editor panels.
    /// The caller owns its UI state and applies actions after this read-only pass.
    pub fn build_custom(
        &mut self,
        input: egui::RawInput,
        draw: impl FnOnce(&egui::Context, egui::TextureId) -> Option<Rect>,
    ) -> egui::FullOutput {
        self.context.begin_pass(input);
        self.viewport = draw(&self.context, self.viewport_id);
        self.context.end_pass()
    }
    /// Refresh the scene after applying a game's actions and before painting.
    pub fn update_custom_scene(&mut self, service: &Service) {
        if let Some(frame) = &mut self.frame {
            service.update_game_frame(frame);
        } else {
            self.frame = Some(service.game_frame());
        }
    }
    /// Decorate the last native snapshot with game-owned presentation only.
    pub fn custom_frame_mut(&mut self) -> Option<&mut RenderFrame> {
        self.frame.as_mut()
    }
    pub fn build(
        &mut self,
        service: &Service,
        input: egui::RawInput,
    ) -> (egui::FullOutput, Vec<Command>) {
        let mut commands = Vec::new();
        self.viewport = None;
        self.ui.mesh.modeling.validate(service);
        self.ui.character_gizmo.validate(service);
        self.context.begin_pass(input);
        draw_editor(
            &self.context,
            service,
            &mut self.ui,
            &mut commands,
            self.viewport_id,
            &mut self.viewport,
        );
        let catalog_start = commands.len();
        authoring_ui::show(
            &self.context,
            service,
            &mut self.ui.authoring,
            &mut commands,
        );
        for command in commands.split_off(catalog_start) {
            if matches!(
                command,
                Command::Generate { .. }
                    | Command::LoadScene { .. }
                    | Command::CreateProject { .. }
                    | Command::OpenProject { .. }
                    | Command::NewProjectScene { .. }
                    | Command::OpenProjectScene { .. }
                    | Command::Exit
            ) {
                project_ui::request(service, &mut self.ui.project, command, &mut commands);
            } else {
                commands.push(command);
            }
        }
        for command in &mut commands {
            if let Command::EditEditor {
                expected_editor_revision,
                ..
            } = command
            {
                *expected_editor_revision = Some(service.editor.revision);
            }
        }
        let output = self.context.end_pass();
        if let Some(frame) = &mut self.frame {
            if let Some(scene) = self.ui.character_gizmo.preview() {
                service
                    .engine
                    .update_render_frame_with_meshes(frame, &scene.meshes);
            } else if let Some(scene) = self.ui.mesh.modeling.preview() {
                service
                    .engine
                    .update_render_frame_with_meshes(frame, &scene.meshes);
            } else if let Some(scene) = self.ui.gizmo.preview() {
                service
                    .engine
                    .update_render_frame_with_meshes(frame, &scene.meshes);
            } else if let Some(draft) = &self.ui.mesh.stroke {
                service
                    .engine
                    .update_render_frame_with_meshes(frame, &draft.document);
            } else if service.botany.revision == Some(service.engine.revision)
                && let Some(overlay) = &service.botany.overlay
            {
                service
                    .engine
                    .update_render_frame_with_meshes(frame, overlay);
            } else {
                service.engine.update_render_frame(frame);
            }
        } else {
            let mut frame = service.engine.render_frame();
            if service.botany.revision == Some(service.engine.revision)
                && let Some(overlay) = &service.botany.overlay
            {
                service
                    .engine
                    .update_render_frame_with_meshes(&mut frame, overlay);
            }
            self.frame = Some(frame);
        }
        if !service.engine.planet_active() {
            if let Some(camera) = &service.editor.current().camera {
                self.frame.as_mut().unwrap().camera = camera.clone();
            }
            if let Some(light) = &service.editor.work_light {
                // Atmosphere belongs to the scene even while inspecting with a work light.
                let frame = self.frame.as_mut().unwrap();
                let atmosphere = frame.light.atmosphere;
                frame.light = light.clone();
                frame.light.atmosphere = atmosphere;
            }
        }
        if service.transform.tool == crate::editor_transform::Tool::Select
            && !self.ui.mesh.sculpt
            && !self.ui.mesh.components
            && !service.mesh_view.edges
            && !service.mesh_view.bones
            && let Some(instance) = mesh_ui::selected(service)
        {
            self.frame.as_mut().unwrap().selection = None;
            self.frame.as_mut().unwrap().selection_mesh = Some(instance.id.clone());
        }
        self.ui
            .gizmo
            .preview_box(service, self.frame.as_mut().unwrap());
        (output, commands)
    }
    pub fn paint(
        &mut self,
        output: egui::FullOutput,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let ppp = output.pixels_per_point;
        if let Some(rect) = self.viewport {
            let w = (rect.width() * ppp).round().clamp(1., 4096.) as u32;
            let h = (rect.height() * ppp).round().clamp(1., 4096.) as u32;
            if self.viewport_texture.width() != w || self.viewport_texture.height() != h {
                self.viewport_texture = viewport_texture(&self.renderer, w, h);
                let view = self
                    .viewport_texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        format: Some(self.renderer.format.remove_srgb_suffix()),
                        ..Default::default()
                    });
                self.painter.update_egui_texture_from_wgpu_texture(
                    &self.renderer.device,
                    &view,
                    wgpu::FilterMode::Linear,
                    self.viewport_id,
                );
            }
            self.renderer.draw(
                self.frame.as_ref().unwrap(),
                &self.viewport_texture.create_view(&Default::default()),
                w,
                h,
            )?;
        }
        let jobs = self.context.tessellate(output.shapes, ppp);
        for (id, delta) in &output.textures_delta.set {
            self.painter
                .update_texture(&self.renderer.device, &self.renderer.queue, *id, delta);
        }
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: ppp,
        };
        let mut encoder = self
            .renderer
            .device
            .create_command_encoder(&Default::default());
        let buffers = self.painter.update_buffers(
            &self.renderer.device,
            &self.renderer.queue,
            &mut encoder,
            &jobs,
            &screen,
        );
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("editor composition"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.painter
                .render(&mut pass.forget_lifetime(), &jobs, &screen);
        }
        self.renderer
            .queue
            .submit(buffers.into_iter().chain([encoder.finish()]));
        for id in &output.textures_delta.free {
            self.painter.free_texture(id);
        }
        Ok(())
    }
    pub fn capture(
        &mut self,
        service: &Service,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Vec<u8>> {
        let target = self
            .renderer
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("editor capture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.renderer.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
        for _ in 0..2 {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(width as f32, height as f32),
                )),
                time: Some(0.),
                ..Default::default()
            };
            let (output, commands) = self.build(service, input);
            debug_assert!(commands.is_empty());
            self.paint(
                output,
                &target.create_view(&Default::default()),
                width,
                height,
            )?;
        }
        self.renderer.read_texture_png(&target, width, height)
    }
}
fn viewport_texture(renderer: &Renderer, width: u32, height: u32) -> wgpu::Texture {
    renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("editor scene viewport"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[renderer.format.remove_srgb_suffix()],
    })
}
fn edit(commands: &mut Vec<Command>, action: Action) {
    commands.push(Command::EditEditor {
        action,
        expected_editor_revision: None,
    });
}
fn open_panel(service: &Service, commands: &mut Vec<Command>, panel: Panel) {
    let mut leaves = Vec::new();
    service.editor.current().layout.leaves(&mut leaves);
    let area = service
        .editor
        .current()
        .layout
        .panel_area(panel)
        .unwrap_or_else(|| {
            leaves
                .iter()
                .find(|(_, p)| *p != Panel::Viewport)
                .unwrap_or(&leaves[0])
                .0
        });
    edit(commands, Action::Open { area, panel });
}
/// Viewport keyboard focus belongs to editor tools; text fields keep their own input.
fn viewport_keyboard(ctx: &egui::Context) -> bool {
    !ctx.wants_keyboard_input()
        || ctx.memory(|m| m.focused() == Some(egui::Id::new("viewport_image")))
}
fn draw_editor(
    ctx: &egui::Context,
    service: &Service,
    state: &mut UiState,
    commands: &mut Vec<Command>,
    texture: egui::TextureId,
    viewport: &mut Option<Rect>,
) {
    // Retain the opening state: closing a popup must consume its viewport click.
    state.mesh.input_blocked = egui::Popup::is_any_open(ctx) || state.project.modal();
    if !state.project.modal()
        && (!ctx.wants_keyboard_input()
            || ctx.memory(|m| m.focused() == Some(egui::Id::new("viewport_image"))))
    {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            if let Some(path) = &service.scene_path {
                commands.push(Command::SaveScene { path: path.clone() });
            } else {
                state.file_action = Some("save");
            }
        }
        if ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Z,
            )
        }) {
            commands.push(Command::Redo);
        } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
            commands.push(Command::Undo);
        }
    }
    if ctx.input(|i| {
        i.pointer.any_pressed() || i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Tab)
    }) {
        ctx.data_mut(|d| {
            let id = egui::Id::new("live_edit_epoch");
            let n = d.get_temp::<u64>(id).unwrap_or(0);
            d.insert_temp(id, n.wrapping_add(1));
        });
    }
    egui::TopBottomPanel::top("global_menu")
        .exact_height(29.)
        .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if cfg!(target_os = "macos") {
                    ui.add_space(78.);
                }
                ui.label(egui::RichText::new("IGNIS").strong().color(EMBER));
                ui.add_space(9.);
                ui.menu_button("Arquivo", |ui| {
                    if ui.button("Novo projeto…").clicked() {
                        project_ui::open(&mut state.project, project_ui::Dialog::Create);
                        ui.close();
                    }
                    if ui.button("Abrir projeto…").clicked() {
                        project_ui::open(&mut state.project, project_ui::Dialog::Open);
                        ui.close();
                    }
                    if ui
                        .add_enabled(service.project.is_some(), egui::Button::new("Nova cena…"))
                        .clicked()
                    {
                        project_ui::open(&mut state.project, project_ui::Dialog::Scene);
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            service.scene_path.is_some(),
                            egui::Button::new("Salvar cena    ⌘S"),
                        )
                        .clicked()
                    {
                        commands.push(Command::SaveScene {
                            path: service.scene_path.clone().unwrap(),
                        });
                        ui.close();
                    }
                    ui.separator();
                    for (label, action) in [
                        ("Abrir cena…", "open"),
                        ("Salvar cena…", "save"),
                        ("Salvar layout…", "save_layout"),
                        ("Carregar layout…", "load_layout"),
                    ] {
                        if ui.button(label).clicked() {
                            state.file_action = Some(action);
                            state.path = service
                                .scene_path
                                .clone()
                                .unwrap_or_else(|| "examples/editor.scene.json".into());
                            if action.contains("layout") {
                                state.path = "editor.layout.json".into();
                            }
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Sair").clicked() {
                        project_ui::request(service, &mut state.project, Command::Exit, commands);
                        ui.close();
                    }
                });
                ui.menu_button("Editar", |ui| {
                    for (label, command) in [
                        ("Desfazer    ⌘Z", Command::Undo),
                        ("Refazer    ⇧⌘Z", Command::Redo),
                    ] {
                        if ui.button(label).clicked() {
                            commands.push(command);
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Exibir", |ui| {
                    let mut settings = service.viewport_performance.settings;
                    let mut changed = ui.checkbox(&mut settings.enabled, "Medir FPS continuamente").changed();
                    changed |= ui.checkbox(&mut settings.vsync, "VSync na medição").changed();
                    if changed { commands.push(Command::SetViewportPerformance { settings }); }
                    ui.separator();
                    ui.menu_button("LOD compilado (experimental)",|ui| {
                        let mut settings=service.engine.mesh_lod_settings;
                        let mut changed=ui.checkbox(&mut settings.enabled,"Usar LOD de geometria").changed();
                        changed|=ui.add(egui::Slider::new(&mut settings.error_px,0.0..=2.0).text("Erro em pixels físicos")).changed();
                        changed|=ui.add(egui::Slider::new(&mut settings.color_error,0.0..=0.1).text("Erro de resposta")).changed();
                        if changed {commands.push(Command::SetMeshLodSettings{settings});}
                        if state.lod_path.is_empty(){state.lod_path="artifacts/mesh-lods.ignis-lod".into();}
                        ui.text_edit_singleline(&mut state.lod_path);
                        if ui.add_enabled(!service.engine.playing,egui::Button::new("Compilar em build time")).clicked(){commands.push(Command::CompileMeshLods{path:state.lod_path.clone()});}
                        if ui.add_enabled(!service.engine.playing,egui::Button::new("Carregar pacote")).clicked(){commands.push(Command::LoadMeshLods{path:state.lod_path.clone()});}
                        ui.label("Fonte/colisão preservadas. Qualidade da imagem precisa de validação.");
                    });
                    ui.separator();
                    if ui.button("Restaurar câmera da cena").clicked() {
                        edit(commands, Action::Camera { camera: None });
                        ui.close();
                    }
                    for panel in Panel::ALL {
                        if ui.button(panel.label()).clicked() {
                            open_panel(service, commands, panel);
                            ui.close();
                        }
                    }
                    ui.separator();
                    let d = service.engine.debug_view;
                    for (label, command) in [
                        (
                            "Guias de visão",
                            Command::SetDebugView {
                                npc_vision: Some(!d.npc_vision),
                                npc_hearing: None,
                                noise_events: None,
                                npc_routes: None,
                            },
                        ),
                        (
                            "Guias de rotas",
                            Command::SetDebugView {
                                npc_vision: None,
                                npc_hearing: None,
                                noise_events: None,
                                npc_routes: Some(!d.npc_routes),
                            },
                        ),
                    ] {
                        if ui.button(label).clicked() {
                            commands.push(command);
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Adicionar", |ui| {
                    ui.menu_button("Malha editável", |ui| {
                        mesh_ui::create_menu(ui, &mut state.mesh, commands)
                    });
                    ui.separator();
                    for (label, kind) in [
                        ("Cubo visual", crate::project::ObjectKind::Visual),
                        ("Bloco sólido", crate::project::ObjectKind::Fixed),
                        ("Caixa física", crate::project::ObjectKind::Dynamic),
                        ("NPC / personagem", crate::project::ObjectKind::Npc),
                    ] {
                        if ui.button(label).clicked() {
                            commands.push(Command::CreateSceneObject { kind });
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Simulação", |ui| {
                    for (label, command) in [
                        ("Executar", Command::Play),
                        ("Pausar", Command::Pause),
                        ("Avançar um passo", Command::Step { ticks: 1 }),
                        ("Reiniciar", Command::Reset),
                    ] {
                        if ui.button(label).clicked() {
                            commands.push(command);
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Janela", |ui| {
                    if ui.button("Duplicar área de trabalho…").clicked() {
                        state.workspace_name = format!("{} cópia", service.editor.current().name);
                        state.rename = false;
                        state.workspace_dialog = true;
                        ui.close();
                    }
                    if ui.button("Renomear área de trabalho…").clicked() {
                        state.workspace_name = service.editor.current().name.clone();
                        state.rename = true;
                        state.workspace_dialog = true;
                        ui.close();
                    }
                    if ui.button("Restaurar distribuição").clicked() {
                        edit(commands, Action::Restore);
                        ui.close();
                    }
                    if ui.button("Excluir área de trabalho").clicked() {
                        edit(commands, Action::RemoveWorkspace);
                        ui.close();
                    }
                    if service.editor.current().maximized.is_some()
                        && ui.button("Restaurar área maximizada").clicked()
                    {
                        edit(commands, Action::Maximize { area: None });
                        ui.close();
                    }
                });
                if ui.button("Ferramentas…").clicked() {
                    state.authoring.open = true;
                }
                ui.menu_button("Ajuda", |ui| {
                    ui.label("Editor de projetos e cenas");
                    ui.label("Arraste abas para outros cabeçalhos.");
                    ui.label("Ícones no canto direito de cada região: maximizar, dividir e abrir/fechar abas.");
                    ui.label("Edição autoral reinicia a simulação.");
                    ui.label("Projeto contém as cenas do jogo.");
                    ui.label("Área de trabalho organiza os painéis.");
                    ui.label("Uma cena ativa por sessão.");
                });
                let drag = ui.allocate_response(
                    Vec2::new(ui.available_width(), 22.),
                    egui::Sense::click_and_drag(),
                );
                if drag.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                        !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
                    ));
                }
            });
        });
    egui::TopBottomPanel::top("workspaces")
        .exact_height(33.)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for w in &service.editor.workspaces {
                    if design_ui::tab(ui, &w.name, w.id == service.editor.active, false).clicked() {
                        edit(
                            commands,
                            Action::Switch {
                                workspace: w.id.clone(),
                            },
                        );
                    }
                }
                design_ui::divider(ui);
                project_ui::context(ui, service, &mut state.project, commands);
                design_ui::divider(ui);
                if design_ui::tab(ui, "Personagem", service.character_editor.open, false).clicked()
                {
                    open_panel(service, commands, Panel::Properties);
                    commands.push(Command::OpenCharacterPanel {
                        open: !service.character_editor.open,
                        prefix: None,
                    });
                }
                if design_ui::tab(ui, "Partículas", service.particle_panel_open, false).clicked() {
                    open_panel(service, commands, Panel::Properties);
                    commands.push(Command::Particles {
                        operation: Box::new(crate::particles::Operation::Open {
                            open: !service.particle_panel_open,
                        }),
                    });
                }
                if design_ui::tab(ui, "Vento", service.wind_panel_open, false).clicked() {
                    open_panel(service, commands, Panel::Properties);
                    commands.push(Command::Wind {
                        operation: Box::new(crate::wind::Operation::Open {
                            open: !service.wind_panel_open,
                        }),
                    });
                }
                if design_ui::tab(ui, "Vegetação", service.botany.open, false).clicked() {
                    open_panel(service, commands, Panel::Properties);
                    commands.push(Command::Botany {
                        operation: Box::new(crate::botany::Operation::Open {
                            open: !service.botany.open,
                            prefix: None,
                        }),
                    });
                }
                if ui
                    .button(if service.engine.playing {
                        "Ⅱ Pausar"
                    } else {
                        "▶ Executar"
                    })
                    .clicked()
                {
                    commands.push(if service.engine.playing {
                        Command::Pause
                    } else {
                        Command::Play
                    });
                }
                if ui
                    .add_enabled(!service.engine.playing, egui::Button::new("Passo"))
                    .clicked()
                {
                    commands.push(Command::Step { ticks: 1 });
                }
                if ui.button("Reiniciar").clicked() {
                    commands.push(Command::Reset);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(if service.engine.playing {
                        "Executando"
                    } else {
                        "Em edição"
                    });
                });
            });
        });
    egui::TopBottomPanel::bottom("status")
        .exact_height(25.)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.small_button("Eventos").clicked() {
                    open_panel(service, commands, Panel::Events);
                }
                if ui.small_button("Projeto").clicked() {
                    open_panel(service, commands, Panel::Project);
                }
                ui.separator();
                ui.small(
                    state
                        .messages
                        .back()
                        .map(String::as_str)
                        .unwrap_or("Pronto"),
                );
                if service.viewport_performance.settings.enabled {
                    let sample = &service.viewport_performance.sample;
                    ui.separator();
                    ui.strong(sample.fps.map_or_else(
                        || "FPS: medindo…".into(),
                        |fps| format!("{fps:.1} FPS · {:.2} ms", sample.frame_ms.unwrap_or(0.)),
                    ));
                    ui.small(format!(
                        "{}×{} · {}",
                        sample.viewport[0], sample.viewport[1], sample.present_mode
                    ));
                    if let Some(air) = service.engine.scene.light.atmosphere {
                        let mut optimized = air.reduce_details;
                        if ui.checkbox(&mut optimized, "Otimização").changed() {
                            let mut light = service.engine.scene.light.clone();
                            light.atmosphere.as_mut().unwrap().reduce_details = optimized;
                            commands.push(Command::SetLight { light });
                        }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(if service.scene_dirty() {
                        "Alterações não salvas"
                    } else {
                        "Sem alterações"
                    })
                    .on_hover_text(format!(
                        "Tick {} · revisão {}",
                        service.engine.simulation.tick, service.engine.revision
                    ));
                });
            });
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let workspace = service.editor.current();
            let area = workspace
                .maximized
                .and_then(|id| workspace.layout.find(id))
                .unwrap_or(&workspace.layout);
            draw_area(
                ui,
                ui.max_rect(),
                area,
                service,
                state,
                commands,
                texture,
                viewport,
            );
        });
    project_ui::draw(ctx, service, &mut state.project, commands);
    if let Some(action) = state.file_action {
        egui::Window::new(match action {
            "open" => "Abrir cena",
            "save" => "Salvar cena",
            "save_layout" => "Salvar layout",
            _ => "Carregar layout",
        })
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Caminho relativo à pasta do projeto");
            ui.add(egui::TextEdit::singleline(&mut state.path).desired_width(410.));
            ui.horizontal(|ui| {
                if ui.button("Confirmar").clicked() {
                    let path = state.path.clone();
                    let command = match action {
                        "open" => Command::OpenProjectScene {
                            path,
                            discard_changes: false,
                        },
                        "save" => Command::SaveScene { path },
                        "save_layout" => Command::SaveEditor { path },
                        _ => Command::LoadEditor { path },
                    };
                    if action == "open" {
                        project_ui::request(service, &mut state.project, command, commands);
                    } else {
                        commands.push(command);
                    }
                    state.file_action = None;
                }
                if ui.button("Cancelar").clicked() {
                    state.file_action = None;
                }
            });
        });
    }
    if state.workspace_dialog {
        egui::Window::new("Área de trabalho")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.text_edit_singleline(&mut state.workspace_name);
                ui.horizontal(|ui| {
                    if ui.button("Salvar").clicked() {
                        let name = std::mem::take(&mut state.workspace_name);
                        state.workspace_dialog = false;
                        edit(
                            commands,
                            if state.rename {
                                Action::Rename { name }
                            } else {
                                Action::Duplicate {
                                    id: (0..=8)
                                        .map(|n| format!("workspace_{n}"))
                                        .find(|id| {
                                            !service.editor.workspaces.iter().any(|w| w.id == *id)
                                        })
                                        .unwrap(),
                                    name,
                                }
                            },
                        );
                    }
                    if ui.button("Cancelar").clicked() {
                        state.workspace_name.clear();
                        state.workspace_dialog = false;
                    }
                });
            });
    }
}
#[allow(clippy::too_many_arguments)]
fn draw_area(
    ui: &mut egui::Ui,
    rect: Rect,
    area: &Area,
    service: &Service,
    state: &mut UiState,
    commands: &mut Vec<Command>,
    texture: egui::TextureId,
    viewport: &mut Option<Rect>,
) {
    match area {
        Area::Split {
            id,
            axis,
            ratio,
            first,
            second,
        } => {
            let horizontal = *axis == Axis::Horizontal;
            let extent = if horizontal {
                rect.width()
            } else {
                rect.height()
            };
            let size = (extent - 5.).max(1.);
            let point = size * ratio;
            let (a, divider, b) = if horizontal {
                (
                    Rect::from_min_max(rect.min, Pos2::new(rect.min.x + point, rect.max.y)),
                    Rect::from_min_max(
                        Pos2::new(rect.min.x + point, rect.min.y),
                        Pos2::new(rect.min.x + point + 5., rect.max.y),
                    ),
                    Rect::from_min_max(Pos2::new(rect.min.x + point + 5., rect.min.y), rect.max),
                )
            } else {
                (
                    Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + point)),
                    Rect::from_min_max(
                        Pos2::new(rect.min.x, rect.min.y + point),
                        Pos2::new(rect.max.x, rect.min.y + point + 5.),
                    ),
                    Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + point + 5.), rect.max),
                )
            };
            ui.painter().rect_filled(divider, 0, BG);
            ui.painter().line_segment(
                if horizontal {
                    [divider.center_top(), divider.center_bottom()]
                } else {
                    [divider.left_center(), divider.right_center()]
                },
                egui::Stroke::new(1.0_f32, LINE),
            );
            let response = ui
                .interact(divider, egui::Id::new(("split", id)), egui::Sense::drag())
                .on_hover_cursor(if horizontal {
                    egui::CursorIcon::ResizeHorizontal
                } else {
                    egui::CursorIcon::ResizeVertical
                });
            if response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
            {
                let offset = if horizontal {
                    pos.x - rect.min.x
                } else {
                    pos.y - rect.min.y
                };
                edit(
                    commands,
                    Action::Resize {
                        area: *id,
                        ratio: (offset / size).clamp(0.1, 0.9),
                    },
                );
            }
            draw_area(ui, a, first, service, state, commands, texture, viewport);
            draw_area(ui, b, second, service, state, commands, texture, viewport);
        }
        Area::Tabs { id, tabs, active } => {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(("area", id))
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            child.set_clip_rect(rect);
            child.painter().rect_filled(rect, 0, BG);
            let head = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 32.));
            child.painter().rect_filled(head, 0, PANEL);
            let mut header = child.new_child(
                egui::UiBuilder::new()
                    .id_salt("header")
                    .max_rect(head.shrink2(Vec2::new(8., 3.)))
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            let mut tab_ui = header.new_child(
                egui::UiBuilder::new()
                    .id_salt("tabs")
                    .max_rect(Rect::from_min_max(
                        header.max_rect().min,
                        Pos2::new(
                            if *active == Panel::Viewport {
                                header.max_rect().min.x
                                    + if rect.width() < 760. { 66. } else { 180. }
                            } else {
                                (header.max_rect().max.x - area_ui::width(*active))
                                    .max(header.max_rect().min.x)
                            },
                            header.max_rect().max.y,
                        ),
                    ))
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            tab_ui.set_clip_rect(tab_ui.max_rect());
            egui::ScrollArea::horizontal()
                .id_salt("tab_scroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(&mut tab_ui, |ui| {
                    ui.horizontal(|header| {
                        for p in tabs {
                            let caption = if *p == Panel::Viewport {
                                service
                                    .scene_path
                                    .as_deref()
                                    .and_then(|p| std::path::Path::new(p).file_name())
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Cena sem arquivo")
                            } else {
                                p.label()
                            };
                            let label = if *p == Panel::Viewport && rect.width() < 760. {
                                "Cena".to_owned()
                            } else if *p == Panel::Properties && state.mesh.sculpt {
                                "Ferramenta".to_owned()
                            } else {
                                design_ui::short_label(
                                    caption,
                                    if *active == Panel::Viewport { 20 } else { 28 },
                                )
                            };
                            let response = design_ui::tab(header, &label, p == active, true)
                                .on_hover_text(caption);
                            if response.clicked() {
                                edit(
                                    commands,
                                    Action::Activate {
                                        area: *id,
                                        panel: *p,
                                    },
                                );
                            }
                            response.dnd_set_drag_payload(DragTab {
                                from: *id,
                                panel: *p,
                            });
                        }
                    });
                });
            if *active == Panel::Viewport {
                let mut tools = header.new_child(
                    egui::UiBuilder::new()
                        .id_salt("viewport_tools")
                        .max_rect(Rect::from_min_max(
                            Pos2::new(tab_ui.max_rect().max.x + 4., head.min.y + 3.),
                            Pos2::new(head.max.x - area_ui::width(*active) - 4., head.max.y - 3.),
                        ))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                tools.set_clip_rect(tools.max_rect());
                mesh_ui::toolbar(&mut tools, service, &mut state.mesh, commands);
            }
            header.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                area_ui::controls(ui, service, *id, *active, commands);
            });
            let drop = child.interact(head, egui::Id::new(("drop", id)), egui::Sense::hover());
            if drop.dnd_hover_payload::<DragTab>().is_some() {
                child.painter().rect_stroke(
                    head.shrink(1.),
                    0,
                    egui::Stroke::new(1.0_f32, EMBER),
                    egui::StrokeKind::Inside,
                );
            }
            if let Some(payload) = drop.dnd_release_payload::<DragTab>() {
                edit(
                    commands,
                    Action::Move {
                        from: payload.from,
                        to: *id,
                        panel: payload.panel,
                    },
                );
            }
            let body = Rect::from_min_max(Pos2::new(rect.min.x, head.max.y), rect.max);
            let mut content = child.new_child(
                egui::UiBuilder::new()
                    .id_salt("body")
                    .max_rect(body.shrink(8.))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            content.set_clip_rect(body);
            match active {
                Panel::Viewport => {
                    let rail = if state.mesh.sculpt { 64. } else { 48. };
                    let image = Rect::from_min_max(body.min + Vec2::new(rail, 0.), body.max);
                    if rail > 0. {
                        let rect = Rect::from_min_size(body.min, Vec2::new(rail, body.height()));
                        content.painter().rect_filled(rect, 0, PANEL);
                        content.painter().line_segment(
                            [rect.right_top(), rect.right_bottom()],
                            egui::Stroke::new(1.0_f32, LINE),
                        );
                        let mut tools = content.new_child(
                            egui::UiBuilder::new()
                                .id_salt("sculpt_tools")
                                .max_rect(rect.shrink2(Vec2::new(6., 8.)))
                                .layout(egui::Layout::top_down(egui::Align::Center)),
                        );
                        tools.set_clip_rect(rect);
                        egui::ScrollArea::vertical().show(&mut tools, |ui| {
                            if state.mesh.sculpt {
                                state.gizmo.cancel();
                                mesh_ui::tools(ui, service, &mut state.mesh, commands);
                            } else {
                                gizmo_ui::tools(
                                    ui,
                                    service,
                                    &mut state.gizmo,
                                    state.mesh.components,
                                    commands,
                                );
                            }
                        });
                    }
                    if image.is_positive() {
                        *viewport = Some(image);
                        content.painter().image(
                            texture,
                            image,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                            Color32::WHITE,
                        );
                        let response = content.interact(
                            image,
                            egui::Id::new("viewport_image"),
                            egui::Sense::click_and_drag(),
                        );
                        let character_mode = character_gizmo::enabled(service);
                        let character_blocked = character_gizmo::viewport(
                            &content,
                            service,
                            &mut state.character_gizmo,
                            commands,
                            &response,
                            image,
                        );
                        let gizmo_blocked = if !character_mode
                            && !state.mesh.sculpt
                            && !state.mesh.input_blocked
                            && !state.mesh.modeling.active()
                        {
                            gizmo_ui::viewport(
                                &content,
                                service,
                                &mut state.gizmo,
                                state.mesh.components,
                                commands,
                                &response,
                                image,
                            )
                        } else {
                            state.gizmo.cancel();
                            false
                        } || character_blocked;
                        if !gizmo_blocked && !character_mode {
                            mesh_ui::viewport(
                                &mut content,
                                service,
                                &mut state.mesh,
                                commands,
                                &response,
                                image,
                            );
                        }
                        if !gizmo_blocked
                            && !character_mode
                            && !state.mesh.sculpt
                            && !state.mesh.components
                            && !state.mesh.input_blocked
                            && response.clicked()
                            && let Some(pos) = response.interact_pointer_pos()
                            && let Some(command) =
                                pick_command(image, pos, ui.ctx().pixels_per_point())
                        {
                            commands.push(command);
                        }
                        let camera = service
                            .editor
                            .current()
                            .camera
                            .as_ref()
                            .unwrap_or(&service.engine.scene.camera);
                        if !gizmo_blocked
                            && !state.mesh.input_blocked
                            && (response.dragged_by(egui::PointerButton::Secondary)
                                || response.dragged_by(egui::PointerButton::Middle))
                        {
                            let delta = ui.input(|i| i.pointer.delta());
                            let pan = response.dragged_by(egui::PointerButton::Middle)
                                || ui.input(|i| i.modifiers.shift);
                            edit(
                                commands,
                                Action::Camera {
                                    camera: Some(navigate_camera(camera, delta, 0., pan)),
                                },
                            );
                        } else if !gizmo_blocked && !state.mesh.input_blocked && response.hovered()
                        {
                            let scroll = ui.input(|i| i.raw_scroll_delta.y);
                            if scroll != 0.
                                && !(state.mesh.sculpt && ui.input(|i| i.modifiers.ctrl))
                            {
                                edit(
                                    commands,
                                    Action::Camera {
                                        camera: Some(navigate_camera(
                                            camera,
                                            Vec2::ZERO,
                                            scroll,
                                            false,
                                        )),
                                    },
                                );
                            }
                        }
                        response.on_hover_text("Botão direito: orbitar · meio/Shift+direito: deslocar · roda: aproximar");
                    }
                }
                Panel::Project => {
                    project_ui::panel(&mut content, service, &mut state.project, commands)
                }
                Panel::Hierarchy => {
                    egui::ScrollArea::vertical()
                        .show(&mut content, |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut state.search)
                                    .hint_text("Filtrar objetos")
                                    .desired_width(f32::INFINITY),
                            );
                            mesh_ui::hierarchy(ui, service, &state.search, commands);
                            ui.add_space(8.);
                            ui.small(egui::RichText::new("Objetos").strong());
                            for entity in &service.engine.simulation.definitions().entities {
                                if service.engine.simulation.position(&entity.id).is_none() {
                                    continue;
                                }
                                if !state.search.is_empty()
                                    && !entity
                                        .name
                                        .to_lowercase()
                                        .contains(&state.search.to_lowercase())
                                    && !entity.id.contains(&state.search)
                                {
                                    continue;
                                }
                                let response = design_ui::list_row(
                                    ui,
                                    &entity.name,
                                    service.engine.selected_id() == Some(entity.id.as_str()),
                                );
                                if response.clicked() {
                                    commands.push(Command::SelectEntity {
                                        id: Some(entity.id.clone()),
                                    });
                                }
                                response.on_hover_text(&entity.id);
                            }
                        })
                        .inner
                }
                Panel::Properties => properties(&mut content, service, state, commands),
                Panel::Templates => {
                    egui::ScrollArea::vertical().show(&mut content, |ui| {
                        ui.weak("Modelos de NPC da cena");
                        if service.engine.scene.npc_templates.is_empty() {
                            ui.label("Esta cena não possui templates.");
                        }
                        for entity in &service.engine.scene.npc_templates {
                            ui.label(&entity.name);
                            ui.small(&entity.id);
                            ui.separator();
                        }
                    });
                }
                Panel::Events => {
                    egui::ScrollArea::both().show(&mut content, |ui| {
                        for message in &state.messages {
                            ui.label(message);
                        }
                        ui.separator();
                        for event in service.engine.simulation.events.iter().rev().take(64) {
                            ui.monospace(format!("{event:?}"));
                        }
                    });
                }
            }
        }
    }
}
pub fn pick_command(view: Rect, pos: Pos2, scale: f32) -> Option<Command> {
    if !view.is_positive() || !view.contains(pos) || !scale.is_finite() || scale <= 0. {
        return None;
    }
    let width = (view.width() * scale).round().clamp(1., 4096.) as u32;
    let height = (view.height() * scale).round().clamp(1., 4096.) as u32;
    let x = (pos.x - view.min.x) / view.width() * width as f32;
    let y = (pos.y - view.min.y) / view.height() * height as f32;
    if x >= width as f32 || y >= height as f32 {
        return None;
    }
    Some(Command::SelectEditorAt {
        x,
        y,
        width,
        height,
    })
}
pub fn navigate_camera(
    camera: &crate::model::Camera,
    delta: Vec2,
    scroll: f32,
    pan: bool,
) -> crate::model::Camera {
    use glam::Vec3;
    let position = Vec3::from(camera.position);
    let target = Vec3::from(camera.target);
    let offset = position - target;
    let distance = offset.length();
    let mut next = camera.clone();
    if pan {
        let forward = -offset / distance;
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward);
        let shift = (right * (-delta.x) + up * delta.y) * distance * 0.0015;
        next.position = (position + shift).to_array();
        next.target = (target + shift).to_array();
    } else {
        let yaw = offset.x.atan2(offset.z) - delta.x * 0.008;
        let pitch = ((offset.y / distance).asin() + delta.y * 0.008).clamp(-1.5, 1.5);
        let radius = (distance * (-scroll * 0.002).exp()).clamp(0.05, 10000.);
        next.position = (target
            + Vec3::new(
                yaw.sin() * pitch.cos(),
                pitch.sin(),
                yaw.cos() * pitch.cos(),
            ) * radius)
            .to_array();
    }
    next
}
fn live_edit(ui: &egui::Ui, edits: Vec<crate::engine::SceneEdit>) -> Command {
    let id = egui::Id::new("live_edit_epoch");
    let epoch = ui.ctx().data_mut(|d| d.get_temp::<u64>(id).unwrap_or(0));
    let focus = ui.memory(|m| m.focused());
    Command::EditSceneLive {
        edits,
        gesture: format!("ui/{epoch}/{focus:?}"),
    }
}
fn properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut UiState,
    commands: &mut Vec<Command>,
) {
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    if service.wind_panel_open {
        wind_ui::properties(ui, service, &mut state.wind, commands);
        return;
    }
    if service.particle_panel_open {
        particles_ui::properties(ui, service, &mut state.particles, commands);
        return;
    }
    if service.botany.open {
        botany_ui::properties(ui, service, &mut state.botany, commands);
        return;
    }
    if service.character_editor.open {
        character_ui::properties(ui, service, &mut state.character, commands);
        return;
    }
    if mesh_ui::selected(service).is_some() {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if !mesh_ui::brush_properties(ui, service, &mut state.mesh, commands) {
                mesh_ui::properties(ui, service, &mut state.mesh, commands);
            }
        });
        return;
    }
    let Some(id) = service.engine.selected_id() else {
        ui.heading("Comece pela cena");
        ui.label("Crie um objeto, selecione-o e ajuste suas propriedades aqui. As alterações aparecem na hora.");
        if design_ui::action(ui, "+ NPC / personagem").clicked() {
            commands.push(Command::CreateSceneObject {
                kind: crate::project::ObjectKind::Npc,
            });
        }
        if design_ui::action(ui, "+ Esfera para modelar").clicked() {
            commands.push(Command::CreateEditorMesh {
                primitive: crate::mesh_tools::Primitive::Sphere,
                resolution: 24,
                size: 2.,
            });
        }
        if design_ui::action(ui, "Explorar ferramentas…").clicked() {
            state.authoring.open = true;
        }
        ui.separator();
        ui.small("NPC: configure alvo, percepção e patrulha nos componentes. Malha: escolha Editar ou Escultura na viewport.");
        return;
    };
    let authored = service.engine.scene.entities.iter().find(|e| e.id == id);
    let Some(entity) = authored else {
        ui.label(id);
        ui.weak("Instância nascida durante a execução.");
        if let Some(position) = service.engine.simulation.position(id) {
            ui.label(format!("Posição {position:?}"));
        }
        return;
    };
    let key = (id.to_owned(), service.engine.revision);
    if state.draft_key.as_ref() != Some(&key) {
        state.draft = Some(entity.clone());
        state.draft_key = Some(key);
        state.draft_attempt = None;
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        let draft = state.draft.as_mut().unwrap();
        ui.heading(&draft.name);
        ui.small(&draft.id);
        ui.separator();
        ui.add_enabled_ui(!service.engine.playing, |ui| {
            design_ui::field(ui, "Nome", |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut draft.name).desired_width(ui.available_width()),
                )
            });
            egui::CollapsingHeader::new("Transformação")
                .default_open(entity.npc.is_none())
                .show(ui, |ui| {
                    for (label, values) in [
                        ("Posição (m)", &mut draft.transform.position),
                        ("Dimensões (m)", &mut draft.transform.size),
                    ] {
                        ui.weak(label);
                        design_ui::vector(ui, values, 0.05);
                    }
                });
            egui::CollapsingHeader::new("Aparência")
                .default_open(entity.npc.is_none())
                .show(ui, |ui| {
                    design_ui::field(ui, "Cor base", |ui| design_ui::color(ui, &mut draft.color));
                    let mut visible = draft.visible.unwrap_or(true);
                    if ui.checkbox(&mut visible, "Visível").changed() {
                        draft.visible = Some(visible);
                    }
                });
            authoring_ui::components(ui, draft, service, &mut state.authoring);
            if draft != entity && state.draft_attempt.as_ref() != Some(draft) {
                commands.push(live_edit(
                    ui,
                    vec![crate::engine::SceneEdit::Upsert {
                        entity: Box::new(draft.clone()),
                    }],
                ));
                state.draft_attempt = Some(draft.clone());
            }
        });
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !service.engine.playing && entity.body != Some(crate::model::BodyKind::Player),
                    egui::Button::new("Duplicar"),
                )
                .clicked()
            {
                commands.push(Command::DuplicateSceneObject { id: id.into() });
            }
            if ui
                .add_enabled(!service.engine.playing, egui::Button::new("Excluir"))
                .clicked()
            {
                commands.push(Command::Remove { id: id.into() });
            }
        });
        if let Some(position) = service.engine.simulation.position(id) {
            ui.separator();
            ui.label("Execução · posição atual");
            ui.monospace(format!(
                "{:.2}  {:.2}  {:.2}",
                position[0], position[1], position[2]
            ));
        }
        if let Some(npc) = &entity.npc {
            ui.separator();
            ui.label("NPC");
            ui.label(format!("Velocidade: {} m/s", npc.speed));
            ui.label(format!(
                "Alvo: {}",
                npc.target.as_deref().unwrap_or("política / rotina")
            ));
        }
    });
}

#[cfg(test)]
mod input_tests {
    use super::*;
    fn pass(
        ctx: &egui::Context,
        service: &Service,
        state: &mut UiState,
        events: Vec<egui::Event>,
    ) -> (Option<Rect>, Vec<Command>) {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
            events,
            time: Some(0.),
            ..Default::default()
        });
        let mut view = None;
        let mut commands = Vec::new();
        draw_editor(
            ctx,
            service,
            state,
            &mut commands,
            egui::TextureId::User(0),
            &mut view,
        );
        let _ = ctx.end_pass();
        (view, commands)
    }
    fn click(pos: Pos2) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ]
    }
    #[test]
    fn rejected_live_entity_draft_waits_for_an_actual_change() {
        let root = tempfile::tempdir().unwrap();
        let mut service = Service::new(
            root.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        service
            .execute(
                Command::SelectEntity {
                    id: Some("player".into()),
                }
                .into(),
            )
            .unwrap();
        let ctx = egui::Context::default();
        let mut state = UiState::default();
        state.project.dismissed = true;
        pass(&ctx, &service, &mut state, vec![]);
        pass(&ctx, &service, &mut state, vec![]);
        state.draft.as_mut().unwrap().transform.size[0] = -1.;
        let before = service.engine.snapshot();
        let (_, commands) = pass(&ctx, &service, &mut state, vec![]);
        assert_eq!(commands.len(), 1);
        assert!(
            service
                .execute(commands.into_iter().next().unwrap().into())
                .is_err()
        );
        for _ in 0..4 {
            assert!(pass(&ctx, &service, &mut state, vec![]).1.is_empty());
        }
        assert_eq!(service.engine.snapshot(), before);
        state.draft.as_mut().unwrap().transform.size[0] = 0.9;
        let (_, commands) = pass(&ctx, &service, &mut state, vec![]);
        assert_eq!(commands.len(), 1);
        service
            .execute(commands.into_iter().next().unwrap().into())
            .unwrap();
        assert!(pass(&ctx, &service, &mut state, vec![]).1.is_empty());
    }
    #[test]
    fn scene_clicks_are_local_and_panel_typing_never_selects_or_drives_gameplay() {
        let root = tempfile::tempdir().unwrap();
        let service = Service::new(
            root.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        let ctx = egui::Context::default();
        let mut state = UiState::default();
        state.project.dismissed = true;
        pass(&ctx, &service, &mut state, vec![]);
        let (view, commands) = pass(&ctx, &service, &mut state, vec![]);
        assert!(commands.is_empty());
        let view = view.unwrap();
        let (_, commands) = pass(&ctx, &service, &mut state, click(view.center()));
        assert!(matches!(
            commands.as_slice(),
            [Command::SelectEditorAt { .. }]
        ));
        let side = Pos2::new(view.max.x + 35., 120.);
        let (_, commands) = pass(&ctx, &service, &mut state, click(side));
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, Command::SelectEditorAt { .. } | Command::SetInput { .. }))
        );
        let (_, commands) = pass(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::Text("wasd".into())],
        );
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, Command::SelectEditorAt { .. } | Command::SetInput { .. }))
        );
        assert_eq!(service.engine.simulation.tick, 0);
    }
}
