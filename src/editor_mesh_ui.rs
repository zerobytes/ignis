//! Direct mesh authoring. Draft strokes never mutate the document until release.
use super::*;
use crate::{mesh_tools, meshes};
use glam::{Mat4, Quat, Vec3};
#[derive(Default)]
pub(super) struct State {
    pub sculpt: bool,
    pub modeling: super::modeling_ui::State,
    pub components: bool,
    rig_mode: bool,
    component_kind: u8,
    vertices: Vec<u32>,
    component_selection: Option<crate::mesh_topology::Components>,
    reselect_after_edit: bool,
    component_error: String,
    marquee: Option<Marquee>,
    visibility: crate::mesh_screen::Visibility,
    component_revision: u64,
    component_source: Option<String>,
    seen_revision: u64,
    bone: usize,
    weight: f32,
    clip_tick: u32,
    clip_key: Option<(String, String, usize, u32)>,
    clip_translation: [f32; 3],
    clip_angles: [f32; 3],
    overlay_key: usize,
    overlay_edges: Vec<[u32; 2]>,
    pub input_blocked: bool,
    pub stroke: Option<Draft>,
    resolution: u16,
    size: f32,
    properties: Option<Properties>,
    inspect_object: bool,
    pub error: String,
    notice: String,
    hit_cache: Option<(HitKey, Option<mesh_tools::Hit>)>,
}
#[derive(PartialEq)]
struct HitKey {
    revision: u64,
    editor: u64,
    tick: u64,
    screen: [u32; 4],
    geometry: usize,
}
struct Marquee {
    start: Pos2,
    end: Pos2,
    revision: u64,
    editor_revision: u64,
    session_revision: u64,
    instance: String,
    view: Rect,
    additive: bool,
    kind: crate::mesh_screen::Kind,
}

struct Properties {
    object: String,
    object_original: String,
    object_members: usize,
    revision: u64,
    instance: meshes::Instance,
    material: crate::materials::Material,
    shading: meshes::Shading,
    attempted: Option<(
        meshes::Instance,
        crate::materials::Material,
        meshes::Shading,
    )>,
}
pub(super) struct Draft {
    pub document: meshes::Document,
    revision: u64,
    editor_revision: u64,
    last_screen: Option<Pos2>,
    instance: String,
    base: meshes::Asset,
    stroke: meshes::Stroke,
    valid: bool,
    adaptive_detail: bool,
    detail: mesh_tools::SculptDetail,
    effective_radius: f32,
    start_world: Vec3,
    plane_normal: Vec3,
    inverse: Mat4,
    path: StrokePath,
}
/// Arc-length sampling on the original surface, independent of redraw frequency.
#[derive(Default)]
struct StrokePath {
    previous: Option<Vec3>,
    remainder: f32,
}
impl StrokePath {
    fn advance(
        &mut self,
        point: Option<Vec3>,
        spacing: f32,
        out: &mut Vec<[f32; 3]>,
        limit: usize,
    ) {
        let Some(point) = point else {
            self.previous = None;
            self.remainder = 0.;
            return;
        };
        if let Some(previous) = self.previous {
            let delta = point - previous;
            let length = delta.length();
            let mut distance = spacing - self.remainder;
            while distance <= length && out.len() < limit {
                out.push((previous + delta * (distance / length)).to_array());
                distance += spacing;
            }
            self.remainder = (self.remainder + length) % spacing;
        } else if out.len() < limit {
            out.push(point.to_array());
        }
        self.previous = Some(point);
    }
}
impl State {
    pub(super) fn command_error(&mut self, text: &str) {
        if self.reselect_after_edit {
            self.component_error = text.into();
            self.reselect_after_edit = false;
        }
    }
    fn defaults(&mut self) {
        if self.resolution == 0 {
            self.resolution = 24;
            self.size = 2.;
            self.weight = 1.;
        }
    }
    pub fn cancel(&mut self) {
        self.modeling.cancel();
        self.stroke = None;
        self.marquee = None;
    }
    pub fn validate_session(&mut self, service: &Service) {
        let source = selected(service).map(|i| i.asset.clone());
        if source != self.component_source {
            self.vertices.clear();
            self.component_selection = None;
            self.reselect_after_edit = false;
            self.component_source = source;
            self.notice.clear();
            self.component_revision = service.engine.revision;
            self.component_error.clear();
        }
        if self.seen_revision != service.engine.revision {
            // Only an accepted edit advances the local selection revision.
            // Rejections leave both the engine revision and selection unchanged.
            if self.reselect_after_edit
                && self.component_revision.checked_add(1) == Some(service.engine.revision)
            {
                self.component_revision = service.engine.revision;
                self.component_error.clear();
            }
            if self.component_revision != service.engine.revision {
                self.reselect_after_edit = false;
                self.vertices.clear();
                self.component_selection = None;
                self.component_error.clear();
            }
            self.seen_revision = service.engine.revision;
        }

        if let Some(selection) = service
            .engine
            .selected_mesh_components()
            .filter(|s| self.component_source.as_ref() == Some(&s.asset))
        {
            if let Ok(asset) = service.engine.scene.meshes.asset(&selection.asset) {
                if self.component_selection.as_ref() != Some(&selection.components) {
                    self.component_error.clear();
                }
                self.component_selection = Some(selection.components.clone());
                self.vertices = component_vertices(&selection.components, &asset.geometry);
                self.component_revision = service.engine.revision;
                self.component_kind = match selection.components {
                    crate::mesh_topology::Components::Vertices { .. } => 0,
                    crate::mesh_topology::Components::Edges { .. } => 1,
                    crate::mesh_topology::Components::Faces { .. } => 2,
                };
            }
        } else if !self.reselect_after_edit {
            self.vertices.clear();
            self.component_selection = None;
        }
        if self.marquee.as_ref().is_some_and(|m| {
            m.revision != service.engine.revision
                || m.editor_revision != service.editor.revision
                || m.session_revision != service.session_revision
                || service.selected_mesh.as_ref() != Some(&m.instance)
                || service.engine.playing
                || !self.components
        }) {
            self.marquee = None;
        }

        if self.stroke.as_ref().is_some_and(|s| {
            s.revision != service.engine.revision
                || s.editor_revision != service.editor.revision
                || service.engine.playing
                || service.selected_mesh.as_ref() != Some(&s.instance)
        }) {
            self.cancel();
            self.error = "Traço cancelado: a cena ou a seleção mudou.".into();
        }
    }
}
pub(super) fn create_menu(ui: &mut egui::Ui, state: &mut State, commands: &mut Vec<Command>) {
    state.defaults();
    ui.label("Malha editável");
    ui.add(egui::Slider::new(&mut state.resolution, 4..=32).text("Resolução"));
    ui.add(egui::Slider::new(&mut state.size, 0.1..=10.).text("Tamanho (m)"));
    for (label, primitive) in [
        ("Esfera", mesh_tools::Primitive::Sphere),
        ("Cubo", mesh_tools::Primitive::Cube),
        ("Cilindro", mesh_tools::Primitive::Cylinder),
        ("Plano", mesh_tools::Primitive::Plane),
    ] {
        if ui.button(label).clicked() {
            commands.push(Command::CreateEditorMesh {
                primitive,
                resolution: state.resolution,
                size: state.size,
            });
            ui.close();
        }
    }
}
pub(super) fn selected(service: &Service) -> Option<&meshes::Instance> {
    let id = service.selected_mesh.as_ref()?;
    service
        .engine
        .scene
        .meshes
        .instances
        .iter()
        .find(|i| &i.id == id)
}
fn sculpt_reason(service: &Service, instance: &meshes::Instance) -> Option<&'static str> {
    let asset = service.engine.scene.meshes.asset(&instance.asset).ok()?;
    if service.engine.playing {
        Some("Pause a simulação para esculpir.")
    } else if asset.rig.is_some() || instance.playback.is_some() || instance.attachment.is_some() {
        Some(
            "Escultura direta disponível na base estática sem rig/animação/vínculo. A malha atual será preservada.",
        )
    } else if service
        .engine
        .scene
        .plants
        .entries
        .iter()
        .any(|e| e.definition.id == asset.id)
        || service
            .engine
            .scene
            .plants
            .connections
            .iter()
            .any(|c| c.child_instance == instance.id || c.parent_instance == instance.id)
    {
        Some(
            "Esta malha possui fonte paramétrica/conexão. Edite a fonte; escultura livre não desvincula automaticamente.",
        )
    } else {
        None
    }
}
fn mode_controls(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    let mesh_selected = selected(service).is_some();
    if design_ui::tab(
        ui,
        "Objeto",
        !state.sculpt && !state.components && !state.rig_mode,
        false,
    )
    .clicked()
    {
        state.cancel();
        state.sculpt = false;
        state.components = false;
        state.rig_mode = false;
    }
    ui.add_enabled_ui(mesh_selected, |ui| {
        if design_ui::tab(ui, "Rig", state.rig_mode, false).clicked() {
            state.rig_mode = true;
            state.components = false;
            state.sculpt = false;
            commands.push(Command::SetEditorMeshView {
                view: crate::editor::MeshView {
                    bones: true,
                    ..service.mesh_view
                },
            });
        }
        if design_ui::tab(ui, "Editar", state.components, false).clicked() {
            state.components = true;
            state.sculpt = false;
            state.rig_mode = false;
            state.vertices.clear();
            commands.push(Command::SetEditorMeshView {
                view: crate::editor::MeshView {
                    edges: true,
                    vertices: true,
                    bones: service.mesh_view.bones,
                    xray: service.mesh_view.xray,
                },
            });
        }
    })
    .response
    .on_disabled_hover_text(
        "Selecione uma malha editável para editar sua geometria ou criar um esqueleto.",
    );
    let available = selected(service).is_some_and(|i| sculpt_reason(service, i).is_none());
    ui.add_enabled_ui(available, |ui| {
        let mode = design_ui::tab(ui, "Escultura", state.sculpt, false);
        if mode.clicked() {
            state.cancel();
            state.sculpt = true;
            state.components = false;
            state.rig_mode = false;
            state.inspect_object = false;
        }
        if !available {
            mode.on_disabled_hover_text(
                selected(service)
                    .and_then(|i| sculpt_reason(service, i))
                    .unwrap_or("Selecione uma malha editável para esculpir."),
            );
        }
    });
}
pub(super) fn toolbar(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    state.defaults();
    state.validate_session(service);
    let mesh_selected = selected(service).is_some();
    if !mesh_selected {
        state.sculpt = false;
        state.components = false;
        state.rig_mode = false;
        state.cancel();
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.;
        let compact = ui.available_width() < 480.;
        if compact {
            ui.menu_button(if state.sculpt {"Escultura"} else if state.components {"Editar"} else if state.rig_mode {"Rig"} else {"Objeto"}, |ui| {
                let previous=(state.sculpt,state.components,state.rig_mode);
                mode_controls(ui,service,state,commands);
                if previous!=(state.sculpt,state.components,state.rig_mode) {ui.close();}
                if state.sculpt && ui.button("Restaurar pincéis").clicked() {
                    edit(commands,Action::Sculpt {settings:crate::editor::SculptSettings::default()});ui.close();
                }
            });
        } else { mode_controls(ui,service,state,commands); }
        if let Some(instance) = selected(service) {
            design_ui::divider(ui);
            if ui
                .small_button("F")
                .on_hover_text("Enquadrar seleção [F]")
                .clicked()
            {
                focus(service, instance, commands);
            }
            if ui.available_width() > 440. {
                ui.weak(design_ui::short_label(&instance.id, 16));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.menu_button("Vista", |ui| {
                let mut view = service.mesh_view;
                ui.checkbox(&mut view.edges, "Arestas / aramado");
                ui.checkbox(&mut view.vertices, "Vértices");
                ui.checkbox(&mut view.bones, "Esqueleto");
                ui.separator();
                ui.checkbox(&mut view.xray, "Raio X da malha");
                ui.small("Desligado: mostra e seleciona só a superfície visível. Ligado: inclui componentes ocultos. Ossos são guias independentes.");
                if view != service.mesh_view {
                    commands.push(Command::SetEditorMeshView { view });
                }
            });
            lighting_ui::controls(ui, service, commands);
            if state.sculpt {
                let mut settings = service.editor.sculpt.relative();
                ui.add_enabled_ui(!service.engine.playing && state.stroke.is_none(), |ui| {
                    ui.menu_button(topology_label(&settings), |ui| {
                        ui.set_min_width(270.);
                        topology_controls(ui, &mut settings);
                    });
                });
                if settings != service.editor.sculpt {
                    state.cancel();
                    edit(commands, Action::Sculpt { settings });
                }
            }
            if state.sculpt && !compact {
                ui.menu_button("Opções", |ui| {
                    if ui.button("Restaurar pincéis").clicked() {
                        edit(
                            commands,
                            Action::Sculpt {
                                settings: crate::editor::SculptSettings::default(),
                            },
                        );
                        ui.close();
                    }
                    ui.separator();
                    ui.label("Ctrl: cavar • Shift: suavizar");
                    ui.label("Esc: cancelar • ⌘Z: desfazer");
                    ui.label("Ctrl + roda: ajustar raio");
                });
            }
        });
    });
}
fn brush_label(brush: meshes::BrushKind) -> &'static str {
    match brush {
        meshes::BrushKind::Draw => "Volume",
        meshes::BrushKind::Grab => "Mover",
        meshes::BrushKind::Smooth => "Suavizar",
        meshes::BrushKind::Flatten => "Achatar",
        meshes::BrushKind::Pinch => "Pinçar",
        meshes::BrushKind::Crease => "Vinco",
        meshes::BrushKind::Scrape => "Raspar",
        meshes::BrushKind::Fill => "Preencher",
        meshes::BrushKind::Relax => "Relaxar",
    }
}
fn topology_label(settings: &crate::editor::SculptSettings) -> &'static str {
    if settings.adaptive_detail && settings.brush == meshes::BrushKind::Draw {
        "Dyntopo"
    } else {
        "Topologia fixa"
    }
}
fn topology_controls(ui: &mut egui::Ui, settings: &mut crate::editor::SculptSettings) {
    use mesh_tools::SculptDetail;
    let draw = settings.brush == meshes::BrushKind::Draw;
    ui.add_enabled_ui(draw, |ui| {
        ui.checkbox(
            &mut settings.adaptive_detail,
            "Dyntopo · topologia dinâmica",
        );
        ui.add_enabled_ui(settings.adaptive_detail, |ui| {
            ui.label("Refinamento: Subdividir arestas");
            ui.weak("Adiciona polígonos sob o traço; não reduz a densidade.");
            let mut brush_detail = matches!(settings.detail, SculptDetail::Brush { .. });
            let before = brush_detail;
            design_ui::field(ui, "Detalhamento", |ui| {
                egui::ComboBox::from_id_salt("sculpt_detail_mode")
                    .selected_text(if brush_detail {
                        "Detalhe do pincel"
                    } else {
                        "Detalhe constante"
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut brush_detail, true, "Detalhe do pincel");
                        ui.selectable_value(&mut brush_detail, false, "Detalhe constante");
                    });
            });
            if brush_detail != before {
                let edge = settings.detail.max_edge(settings.radius);
                settings.detail = if brush_detail {
                    SculptDetail::Brush {
                        percent: (100. * edge / settings.radius).clamp(3.125, 100.),
                    }
                } else {
                    SculptDetail::Constant {
                        edge_length: edge.clamp(0.00001, 100.),
                    }
                };
            }
            match &mut settings.detail {
                SculptDetail::Brush { percent } => {
                    design_ui::field(ui, "Tamanho", |ui| {
                        ui.add_sized(
                            [ui.available_width(), 26.],
                            design_ui::Scrub::new(percent)
                                .range(3.125..=100.)
                                .speed(0.125)
                                .suffix(" % do raio"),
                        );
                    });
                }
                SculptDetail::Constant { edge_length } => {
                    design_ui::field(ui, "Aresta alvo", |ui| {
                        ui.add_sized(
                            [ui.available_width(), 26.],
                            design_ui::Scrub::new(edge_length)
                                .range(0.00001..=100.)
                                .speed(0.001)
                                .fixed_decimals(5)
                                .suffix(" m"),
                        );
                    });
                }
            }
            ui.small(format!(
                "Aresta alvo: {:.5} m · espaço local",
                settings.detail.max_edge(settings.radius)
            ));
            ui.weak("Menor tamanho pede mais polígonos. Os limites da malha continuam valendo.");
        });
    });
    if !draw {
        ui.weak("Este pincel preserva a topologia. Dyntopo está disponível para Volume.");
    } else if !settings.adaptive_detail {
        ui.weak("Topologia fixa: move os vértices existentes sem adicionar polígonos.");
    }
}
fn brush_mark(ui: &egui::Ui, rect: Rect, brush: meshes::BrushKind, color: Color32) {
    let p = ui.painter();
    let origin = rect.center();
    let s = rect.width() * 0.42;
    let stroke = egui::Stroke::new(1.5_f32, color);
    match brush {
        meshes::BrushKind::Draw => {
            let points = (-12..=12)
                .map(|i| {
                    let x = i as f32 / 12.;
                    origin + Vec2::new(x * s, (0.35 - 0.9 * (1. - x * x).powi(2)) * s)
                })
                .collect();
            p.add(egui::Shape::line(points, stroke));
            p.line_segment(
                [
                    origin + Vec2::new(-s, s * 0.65),
                    origin + Vec2::new(s, s * 0.65),
                ],
                egui::Stroke::new(1.0_f32, design_ui::MUTED),
            );
            p.line_segment(
                [
                    origin + Vec2::new(0., -s * 0.9),
                    origin + Vec2::new(0., -s * 0.4),
                ],
                stroke,
            );
        }
        meshes::BrushKind::Grab => {
            p.line_segment(
                [
                    origin - Vec2::new(s * 0.8, 0.),
                    origin + Vec2::new(s * 0.8, 0.),
                ],
                stroke,
            );
            p.line_segment(
                [
                    origin - Vec2::new(0., s * 0.8),
                    origin + Vec2::new(0., s * 0.8),
                ],
                stroke,
            );
            for (axis, normal) in [
                (Vec2::X, Vec2::Y),
                (-Vec2::X, Vec2::Y),
                (Vec2::Y, Vec2::X),
                (-Vec2::Y, Vec2::X),
            ] {
                let tip = origin + axis * s * 0.85;
                p.add(egui::Shape::line(
                    vec![
                        tip - axis * s * 0.3 + normal * s * 0.25,
                        tip,
                        tip - axis * s * 0.3 - normal * s * 0.25,
                    ],
                    stroke,
                ));
            }
        }
        meshes::BrushKind::Flatten | meshes::BrushKind::Scrape | meshes::BrushKind::Fill => {
            p.line_segment(
                [origin - Vec2::new(s, 0.), origin + Vec2::new(s, 0.)],
                stroke,
            );
            if brush != meshes::BrushKind::Fill {
                p.arrow(origin - Vec2::new(0., s), Vec2::new(0., s * 0.7), stroke);
            }
            if brush != meshes::BrushKind::Scrape {
                p.arrow(origin + Vec2::new(0., s), Vec2::new(0., -s * 0.7), stroke);
            }
        }
        meshes::BrushKind::Pinch => {
            p.arrow(origin - Vec2::new(s, 0.), Vec2::new(s * 0.8, 0.), stroke);
            p.arrow(origin + Vec2::new(s, 0.), Vec2::new(-s * 0.8, 0.), stroke);
            p.line_segment(
                [
                    origin - Vec2::new(0., s * 0.7),
                    origin + Vec2::new(0., s * 0.7),
                ],
                stroke,
            );
        }
        meshes::BrushKind::Crease => {
            p.add(egui::Shape::line(
                vec![
                    origin + Vec2::new(-s, -s * 0.3),
                    origin + Vec2::new(-s * 0.2, 0.),
                    origin + Vec2::new(0., s * 0.8),
                    origin + Vec2::new(s * 0.2, 0.),
                    origin + Vec2::new(s, -s * 0.3),
                ],
                stroke,
            ));
        }
        meshes::BrushKind::Relax => {
            for y in [-0.7, 0., 0.7] {
                p.line_segment(
                    [origin + Vec2::new(-s, y * s), origin + Vec2::new(s, y * s)],
                    stroke,
                );
                for x in [-0.7, 0., 0.7] {
                    p.circle_filled(origin + Vec2::new(x * s, y * s), 1.8, color);
                }
            }
        }
        meshes::BrushKind::Smooth => {
            for offset in [-0.6, 0., 0.6] {
                let points = (-12..=12)
                    .map(|i| {
                        let x = i as f32 / 12.;
                        origin
                            + Vec2::new(
                                x * s,
                                (offset + (x * std::f32::consts::PI).sin() * 0.18) * s,
                            )
                    })
                    .collect();
                p.add(egui::Shape::line(points, stroke));
            }
        }
    }
}
fn brush_button(
    ui: &mut egui::Ui,
    kind: meshes::BrushKind,
    active: bool,
    width: f32,
) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::new(width, 58.), egui::Sense::click());
    let fill = if active {
        Color32::from_rgb(66, 49, 39)
    } else if r.hovered() {
        Color32::from_rgb(48, 50, 56)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 6, fill);
    if active || r.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            6,
            egui::Stroke::new(1.0_f32, EMBER),
            egui::StrokeKind::Inside,
        );
    }
    brush_mark(
        ui,
        Rect::from_center_size(rect.center() - Vec2::new(0., 8.), Vec2::splat(26.)),
        kind,
        if active { EMBER } else { design_ui::TEXT },
    );
    ui.painter().text(
        rect.center_bottom() - Vec2::new(0., 9.),
        egui::Align2::CENTER_CENTER,
        brush_label(kind),
        egui::FontId::proportional(11.),
        design_ui::TEXT,
    );
    r.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), brush_label(kind))
    });
    r.on_hover_text(match kind {
        meshes::BrushKind::Draw => "Volume [1]. Ctrl para cavar.",
        meshes::BrushKind::Grab => "Mover [2]. Puxa a região do pincel.",
        meshes::BrushKind::Smooth => "Suavizar [3]. Shift inicia suavização.",
        meshes::BrushKind::Flatten => {
            "Achatar [4]. Aproxima a superfície do plano médio do pincel."
        }
        meshes::BrushKind::Pinch => "Pinçar [5]. Concentra a superfície; Ctrl espalha.",
        meshes::BrushKind::Crease => "Vinco [6]. Escava e concentra; Ctrl cria uma crista.",
        meshes::BrushKind::Scrape => "Raspar [7]. Remove acima do plano; Ctrl preenche abaixo.",
        meshes::BrushKind::Fill => "Preencher [8]. Eleva abaixo do plano; Ctrl raspa acima.",
        meshes::BrushKind::Relax => {
            "Relaxar [9]. Distribui vértices no plano tangente sem suavização normal."
        }
    })
}
pub(super) fn tools(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    if !state.sculpt {
        return;
    }
    let mut settings = service.editor.sculpt.relative();
    ui.spacing_mut().item_spacing = Vec2::new(4., 4.);
    for brush in meshes::BrushKind::ALL {
        if brush_button(ui, brush, settings.brush == brush, 52.).clicked() {
            settings.brush = brush;
            state.inspect_object = false;
        }
    }
    if settings != service.editor.sculpt {
        state.cancel();
        edit(commands, Action::Sculpt { settings });
    }
}
pub(super) fn brush_properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) -> bool {
    if !state.sculpt {
        return false;
    }
    ui.horizontal(|ui| {
        if design_ui::tab(ui, "Pincel", !state.inspect_object, false).clicked() {
            state.inspect_object = false;
        }
        if design_ui::tab(ui, "Objeto", state.inspect_object, false).clicked() {
            state.inspect_object = true;
        }
    });
    ui.add_space(8.);
    if state.inspect_object {
        return false;
    }
    let mut settings = service.editor.sculpt.relative();
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(40.), egui::Sense::hover());
        ui.painter()
            .rect_filled(r, 6, Color32::from_rgb(44, 39, 35));
        brush_mark(ui, r.shrink(6.), settings.brush, EMBER);
        ui.vertical(|ui| {
            ui.heading(brush_label(settings.brush));
            ui.weak(match settings.brush {
                meshes::BrushKind::Draw => "Adicionar ou cavar relevo",
                meshes::BrushKind::Grab => "Deslocar a superfície",
                meshes::BrushKind::Smooth => "Regularizar a superfície",
                meshes::BrushKind::Flatten => "Aproximar do plano médio",
                meshes::BrushKind::Pinch => "Concentrar detalhes tangencialmente",
                meshes::BrushKind::Crease => "Criar sulcos ou cristas",
                meshes::BrushKind::Scrape => "Remover volumes acima do plano",
                meshes::BrushKind::Fill => "Preencher depressões abaixo do plano",
                meshes::BrushKind::Relax => "Regularizar a distribuição dos vértices",
            });
        });
    });
    ui.add_space(12.);
    ui.add_enabled_ui(!service.engine.playing && state.stroke.is_none(), |ui| {
        design_ui::field(ui, "Raio", |ui| {
            ui.add_sized(
                [ui.available_width(), 26.],
                design_ui::Scrub::new(&mut settings.radius)
                    .range(0.001..=100.)
                    .speed(0.005)
                    .suffix(" m"),
            );
        });
        ui.add_space(6.);
        design_ui::field(ui, "Força", |ui| {
            let kind = settings.brush;
            let value = settings.strength_mut(kind);
            let number = design_ui::Scrub::new(value)
                .range(0.001..=1.)
                .speed(0.005)
                .percent();
            ui.add_sized([ui.available_width(), 26.], number);
        });
        ui.add_space(6.);
        ui.small("Arraste os valores. Shift para ajuste fino.");
        design_ui::section(ui, "Topologia", |ui| topology_controls(ui, &mut settings));
        design_ui::section(ui, "Traçado", |ui| {
            if settings.brush == meshes::BrushKind::Draw {
                ui.label("Altura proporcional ao raio e à força");
                ui.weak("Repassar em um novo traço adiciona outra camada.");
            } else if settings.brush == meshes::BrushKind::Grab {
                ui.label("Desloca a região onde o arraste começa");
                ui.weak("Para desenhar um relevo contínuo, escolha Volume.");
            } else {
                ui.label("Aplicação contínua");
                ui.weak("Espaçamento: 15% do raio");
            }
            ui.small(
                if settings.brush.invertible() && settings.brush != meshes::BrushKind::Grab {
                    "Ctrl inverte · Shift suaviza"
                } else {
                    "Shift suaviza temporariamente"
                },
            );
            ui.small("Esc cancela · ⌘Z desfaz");
        });
        design_ui::section(ui, "Malha ativa", |ui| {
            if let Some(instance) = selected(service) {
                ui.label(&instance.id);
                let document = state
                    .stroke
                    .as_ref()
                    .map_or(&service.engine.scene.meshes, |draft| &draft.document);
                if let Ok(asset) = document.asset(&instance.asset) {
                    ui.small(format!(
                        "{} vértices · {} triângulos",
                        asset.geometry.positions.len(),
                        asset.geometry.triangles.len()
                    ));
                    if state
                        .stroke
                        .as_ref()
                        .is_some_and(|draft| draft.adaptive_detail)
                    {
                        ui.weak("Prévia do detalhe · confirme soltando o mouse");
                    }
                }
                if design_ui::action(ui, "Enquadrar seleção [F]").clicked() {
                    focus(service, instance, commands);
                }
            }
        });
    });
    if settings != service.editor.sculpt {
        state.cancel();
        edit(commands, Action::Sculpt { settings });
    }
    if !state.notice.is_empty() {
        ui.weak(&state.notice);
    }
    if !state.error.is_empty() {
        ui.colored_label(Color32::LIGHT_RED, &state.error);
    }
    true
}

fn focus(service: &Service, instance: &meshes::Instance, commands: &mut Vec<Command>) {
    if let Ok(pose) = service
        .engine
        .scene
        .meshes
        .pose(&instance.id, service.engine.simulation.tick)
    {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for p in pose.positions {
            let p = Vec3::from(p);
            min = min.min(p);
            max = max.max(p);
        }
        let center = (min + max) * 0.5;
        let distance = (max - min).length().max(0.2) * 1.5;
        let mut camera = service
            .editor
            .current()
            .camera
            .clone()
            .unwrap_or_else(|| service.engine.scene.camera.clone());
        let direction =
            (Vec3::from(camera.position) - Vec3::from(camera.target)).normalize_or_zero();
        camera.target = center.to_array();
        camera.position = (center + direction * distance).to_array();
        commands.push(Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera),
            },
            expected_editor_revision: Some(service.editor.revision),
        });
    }
}
pub(super) fn hierarchy(
    ui: &mut egui::Ui,
    service: &Service,
    search: &str,
    commands: &mut Vec<Command>,
) {
    let query = search.to_lowercase();
    if !service.engine.scene.meshes.instances.is_empty() {
        ui.add_space(8.);
        ui.small(egui::RichText::new("Malhas").strong());
    }
    for instance in &service.engine.scene.meshes.instances {
        if !query.is_empty() && !instance.id.to_lowercase().contains(&query) {
            continue;
        }
        if design_ui::list_row(
            ui,
            &instance.id,
            service.selected_mesh.as_ref() == Some(&instance.id),
        )
        .clicked()
        {
            commands.push(Command::SelectEditorMesh {
                instance: Some(instance.id.clone()),
            });
        }
    }
}
pub(super) fn properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) -> bool {
    let Some(instance) = selected(service) else {
        return false;
    };
    let asset = service.engine.scene.meshes.asset(&instance.asset).unwrap();
    if state
        .properties
        .as_ref()
        .is_none_or(|p| p.revision != service.engine.revision || p.instance.id != instance.id)
    {
        let owner = service
            .engine
            .scene
            .meshes
            .objects
            .iter()
            .find(|o| o.members.contains(&instance.id));
        let object = owner.map_or_else(String::new, |o| o.id.clone());
        state.properties = Some(Properties {
            object_original: object.clone(),
            object,
            object_members: owner.map_or(0, |o| o.members.len()),
            revision: service.engine.revision,
            instance: instance.clone(),
            material: asset.material.clone(),
            shading: asset.shading,
            attempted: None,
        });
    }
    if state.rig_mode {
        ui.heading("Rigging e animação");
        ui.add_enabled_ui(!service.engine.playing, |ui| {
            rig_properties(ui, service, state, commands, instance, asset);
        });
        return true;
    }
    if state.components {
        ui.add_enabled_ui(!service.engine.playing, |ui| {
            component_properties(ui, service, state, commands, instance, asset);
            rig_properties(ui, service, state, commands, instance, asset);
        });
        return true;
    }
    ui.heading(&instance.id);
    ui.add_enabled_ui(!service.engine.playing, |ui| {
        let mut enabled = service.engine.scene.collision_meshes.contains(&instance.id);
        if ui
            .checkbox(&mut enabled, "Colisão estática da malha completa")
            .changed()
        {
            commands.push(Command::EditScene {
                edits: vec![crate::engine::SceneEdit::SetMeshCollision {
                    instance: instance.id.clone(),
                    enabled,
                }],
            });
        }
    });
    ui.small(format!(
        "Asset {} · {} vértices · {} triângulos",
        asset.id,
        asset.geometry.positions.len(),
        asset.geometry.triangles.len()
    ));
    ui.add_enabled_ui(!service.engine.playing && state.stroke.is_none(), |ui| {
        let p = state.properties.as_mut().unwrap();
        design_ui::section(ui, "Objeto composto", |ui| {
            ui.weak("Peças do mesmo objeto compartilham uma identidade visual.");
            ui.text_edit_singleline(&mut p.object);
            if p.object_members > 0 {
                ui.small(format!("Vínculo atual: {} peças", p.object_members));
            }
            if ui
                .add_enabled(
                    p.object.trim() != p.object_original,
                    egui::Button::new("Aplicar vínculo"),
                )
                .clicked()
            {
                commands.push(Command::EditMesh {
                    edit: meshes::Edit::AssignObject {
                        instance: instance.id.clone(),
                        object: (!p.object.trim().is_empty()).then(|| p.object.trim().to_owned()),
                    },
                });
            }
            if !p.object_original.is_empty() && ui.button("Desvincular peça").clicked() {
                commands.push(Command::EditMesh {
                    edit: meshes::Edit::AssignObject {
                        instance: instance.id.clone(),
                        object: None,
                    },
                });
            }
            if !p.object_original.is_empty() && ui.button("Remover agrupamento").clicked() {
                commands.push(Command::EditMesh {
                    edit: meshes::Edit::RemoveObject {
                        id: p.object_original.clone(),
                    },
                });
            }
        });
        design_ui::section(ui, "Transformação", |ui| {
            ui.weak("Posição · m");
            vec3(ui, &mut p.instance.position, 0.05);
            let (x, y, z) = Quat::from_array(p.instance.rotation).to_euler(glam::EulerRot::XYZ);
            let mut angles = [x.to_degrees(), y.to_degrees(), z.to_degrees()];
            ui.weak("Rotação · °");
            if vec3(ui, &mut angles, 0.5) {
                p.instance.rotation = Quat::from_euler(
                    glam::EulerRot::XYZ,
                    angles[0].to_radians(),
                    angles[1].to_radians(),
                    angles[2].to_radians(),
                )
                .to_array();
            }
            design_ui::field(ui, "Escala", |ui| {
                ui.add(
                    design_ui::Scrub::new(&mut p.instance.scale)
                        .range(0.001..=100.)
                        .speed(0.01),
                );
            });
        });
        design_ui::section(ui, "Aparência", |ui| {
            design_ui::field(ui, "Cor base", |ui| {
                design_ui::color(ui, &mut p.instance.color)
            });
            design_ui::field(ui, "Superfície", |ui| {
                ui.selectable_value(&mut p.shading, meshes::Shading::Smooth, "Suave");
                ui.selectable_value(&mut p.shading, meshes::Shading::Flat, "Plano");
            });
            let mut procedural = p.material.pattern.is_some();
            if ui
                .checkbox(&mut procedural, "Material procedural")
                .changed()
            {
                p.material.pattern = procedural.then_some(crate::materials::Pattern {
                    frequency: [2.; 3],
                    low: [0.45; 3],
                    high: [1.; 3],
                    bump: 0.005,
                    warp: 0.1,
                    octaves: 3,
                    seed: 1,
                    contrast: 1.,
                    kind: crate::materials::PatternKind::Noise,
                });
            }
            if let Some(pattern) = &mut p.material.pattern {
                design_ui::field(ui, "Cor escura", |ui| {
                    design_ui::color(ui, &mut pattern.low)
                });
                design_ui::field(ui, "Cor clara", |ui| {
                    design_ui::color(ui, &mut pattern.high)
                });
                let mut frequency = pattern.frequency[0];
                if ui
                    .add(egui::Slider::new(&mut frequency, 0.1..=20.).text("Frequência"))
                    .changed()
                {
                    pattern.frequency = [frequency; 3];
                }
                ui.add(egui::Slider::new(&mut pattern.bump, 0.0..=0.05).text("Relevo visual"));
            }
        });
        ui.add_space(12.);
        let changed =
            p.instance != *instance || p.material != asset.material || p.shading != asset.shading;
        let unattempted = p
            .attempted
            .as_ref()
            .is_none_or(|(instance, material, shading)| {
                instance != &p.instance || material != &p.material || shading != &p.shading
            });
        if changed && unattempted {
            commands.push(live_edit(
                ui,
                vec![
                    crate::engine::SceneEdit::Mesh {
                        edit: meshes::Edit::PutInstance {
                            instance: p.instance.clone(),
                        },
                    },
                    crate::engine::SceneEdit::Mesh {
                        edit: meshes::Edit::SetShading {
                            asset: asset.id.clone(),
                            shading: p.shading,
                        },
                    },
                    crate::engine::SceneEdit::Mesh {
                        edit: meshes::Edit::SetMaterial {
                            asset: asset.id.clone(),
                            material: p.material.clone(),
                        },
                    },
                ],
            ));
            p.attempted = Some((p.instance.clone(), p.material.clone(), p.shading));
        }
        rig_properties(ui, service, state, commands, instance, asset);
        design_ui::section(ui, "Geometria", |ui| {
            if design_ui::action(ui, "Subdividir malha")
                .on_hover_text("Subdivisão linear: acrescenta resolução sem arredondar a forma.")
                .clicked()
            {
                commands.push(Command::EditMesh {
                    edit: meshes::Edit::Subdivide {
                        asset: asset.id.clone(),
                        reference: service.engine.mesh_reference(),
                        components: None,
                    },
                });
            }
        });
        design_ui::section(ui, "Instância", |ui| {
            if design_ui::action(ui, "Duplicar instância").clicked() {
                let mut next = instance.clone();
                if let Some(n) = (0..1024).find(|n| {
                    !service
                        .engine
                        .scene
                        .meshes
                        .instances
                        .iter()
                        .any(|i| i.id == format!("mesh_copy_{n}"))
                }) {
                    next.id = format!("mesh_copy_{n}");
                    next.position[0] += 1.;
                    commands.push(Command::EditMesh {
                        edit: meshes::Edit::PutInstance { instance: next },
                    });
                }
            }
            if ui
                .add_sized(
                    [ui.available_width(), 24.],
                    egui::Button::new(
                        egui::RichText::new("Excluir malha")
                            .color(Color32::from_rgb(234, 155, 148)),
                    ),
                )
                .clicked()
            {
                let mut edits = vec![crate::engine::SceneEdit::Mesh {
                    edit: meshes::Edit::RemoveInstance {
                        id: instance.id.clone(),
                    },
                }];
                if service
                    .engine
                    .scene
                    .meshes
                    .instances
                    .iter()
                    .filter(|i| i.asset == asset.id)
                    .count()
                    == 1
                    && !service
                        .engine
                        .scene
                        .plants
                        .entries
                        .iter()
                        .any(|e| e.definition.id == asset.id)
                {
                    edits.push(crate::engine::SceneEdit::Mesh {
                        edit: meshes::Edit::RemoveAsset {
                            id: asset.id.clone(),
                        },
                    });
                }
                commands.push(Command::EditScene { edits });
            }
        });
        ui.add_space(8.);
        ui.small("Escultura e material compartilhados entre instâncias do asset.");
    });
    true
}
fn vec3(ui: &mut egui::Ui, values: &mut [f32; 3], speed: f64) -> bool {
    design_ui::vector(ui, values, speed)
}

fn screen(view: Rect, pos: Pos2, scale: f32) -> (f32, f32, u32, u32) {
    let width = (view.width() * scale).round().clamp(1., 4096.) as u32;
    let height = (view.height() * scale).round().clamp(1., 4096.) as u32;
    (
        (pos.x - view.min.x) / view.width() * width as f32,
        (pos.y - view.min.y) / view.height() * height as f32,
        width,
        height,
    )
}
fn hit(
    service: &Service,
    document: &meshes::Document,
    camera: &crate::model::Camera,
    view: Rect,
    pos: Pos2,
    scale: f32,
) -> crate::model::Result<Option<mesh_tools::Hit>> {
    let (x, y, w, h) = screen(view, pos, scale);
    let mesh = mesh_tools::pick(
        document,
        camera,
        x,
        y,
        w,
        h,
        service.engine.simulation.tick,
        &service
            .engine
            .scene
            .plants
            .connections
            .iter()
            .map(|c| c.child_instance.as_str())
            .collect::<Vec<_>>(),
    )?;
    let actor = crate::picking::pick_camera(&service.engine.simulation, camera, x, y, w, h)?;
    Ok(mesh.filter(|m| actor.is_none_or(|a| m.distance <= a.distance)))
}
pub(super) fn viewport(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
    response: &egui::Response,
    view: Rect,
) {
    state.validate_session(service);
    let release_then_save = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
        && commands
            .iter()
            .all(|c| matches!(c, Command::SaveScene { .. }));
    if state.input_blocked
        || (!commands.is_empty() && !release_then_save)
        || ui.input(|i| !i.focused || i.key_pressed(egui::Key::Escape))
    {
        state.cancel();
        return;
    }
    if state.components
        && super::modeling_ui::viewport(ui, service, &mut state.modeling, commands, response, view)
    {
        return;
    }
    let mut settings = service.editor.sculpt.relative();
    let camera = service
        .editor
        .current()
        .camera
        .as_ref()
        .unwrap_or(&service.engine.scene.camera);
    if response.hovered()
        && super::viewport_keyboard(ui.ctx())
        && !ui.input(|i| i.modifiers.ctrl || i.modifiers.command || i.modifiers.alt)
    {
        if ui.input(|i| i.key_pressed(egui::Key::F))
            && let Some(instance) = selected(service)
        {
            focus(service, instance, commands);
        }
        for (key, kind) in [
            (egui::Key::Num1, meshes::BrushKind::Draw),
            (egui::Key::Num2, meshes::BrushKind::Grab),
            (egui::Key::Num3, meshes::BrushKind::Smooth),
            (egui::Key::Num4, meshes::BrushKind::Flatten),
            (egui::Key::Num5, meshes::BrushKind::Pinch),
            (egui::Key::Num6, meshes::BrushKind::Crease),
            (egui::Key::Num7, meshes::BrushKind::Scrape),
            (egui::Key::Num8, meshes::BrushKind::Fill),
            (egui::Key::Num9, meshes::BrushKind::Relax),
        ] {
            if ui.input(|i| i.key_pressed(key)) {
                settings.brush = kind;
                state.cancel();
            }
        }
    }
    if response.hovered() && state.sculpt && ui.input(|i| i.modifiers.ctrl) {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0. {
            state.cancel();
            settings.radius = (settings.radius * (scroll * 0.002).exp()).clamp(0.001, 100.);
        }
    }
    if settings != service.editor.sculpt {
        edit(commands, Action::Sculpt { settings });
        return;
    }
    overlays(ui, service, state, commands, response, view);
    if !state.sculpt || service.engine.playing {
        return;
    }
    let Some(instance) = selected(service) else {
        state.cancel();
        return;
    };
    if sculpt_reason(service, instance).is_some() {
        state.cancel();
        return;
    }
    let scale = ui.ctx().pixels_per_point();
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let pointer_in = pointer.filter(|p| view.contains(*p));
    if let Some(pos) = pointer_in {
        // Never chase the surface raised by this very gesture. Revisions cancel
        // the draft, so the authoring document remains its frozen reference.
        let document = &service.engine.scene.meshes;
        let (x, y, w, h) = screen(view, pos, scale);
        let key = HitKey {
            revision: service.engine.revision,
            editor: service.editor.revision,
            tick: service.engine.simulation.tick,
            screen: [x.to_bits(), y.to_bits(), w, h],
            geometry: document
                .asset(&instance.asset)
                .map_or(0, |a| std::sync::Arc::as_ptr(&a.geometry) as usize),
        };
        let current = if state.hit_cache.as_ref().is_some_and(|(old, _)| *old == key) {
            state.hit_cache.as_ref().unwrap().1.clone()
        } else {
            let result = hit(service, document, camera, view, pos, scale)
                .ok()
                .flatten();
            state.hit_cache = Some((key, result.clone()));
            result
        }
        .filter(|h| h.instance == instance.id);
        if let Some(h) = &current {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            let color =
                if ui.input(|i| i.modifiers.shift) || settings.brush == meshes::BrushKind::Smooth {
                    Color32::from_rgb(110, 210, 170)
                } else if settings.brush == meshes::BrushKind::Grab {
                    Color32::from_rgb(110, 180, 240)
                } else if ui.input(|i| i.modifiers.ctrl) {
                    Color32::from_rgb(245, 110, 100)
                } else {
                    EMBER
                };
            ring(
                ui,
                view,
                camera,
                h,
                state
                    .stroke
                    .as_ref()
                    .map_or(settings.radius, |draft| draft.effective_radius)
                    * instance.scale,
                scale,
                color,
            );
        }
        // A quick click/drag may be delivered with press and release in one pass.
        // Freeze the actual press location rather than the final pointer position.
        let press = ui.input(|i| {
            i.events.iter().find_map(|event| match event {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    ..
                } => Some(*pos),
                _ => None,
            })
        });
        let pressed_hit = press
            .filter(|p| view.contains(*p))
            .and_then(|p| {
                hit(
                    service,
                    &service.engine.scene.meshes,
                    camera,
                    view,
                    p,
                    scale,
                )
                .ok()
                .flatten()
            })
            .filter(|h| h.instance == instance.id);
        if response.hovered()
            && let Some(h) = &pressed_hit
        {
            let base = service
                .engine
                .scene
                .meshes
                .asset(&instance.asset)
                .unwrap()
                .clone();
            let kind = if ui.input(|i| i.modifiers.shift) {
                meshes::BrushKind::Smooth
            } else {
                settings.brush
            };
            let inverse = Mat4::from_cols_array_2d(
                &service
                    .engine
                    .scene
                    .meshes
                    .pose(&instance.id, 0)
                    .unwrap()
                    .world_transform,
            )
            .inverse();
            let strength = settings.strength(kind)
                * if kind.invertible() && ui.input(|i| i.modifiers.ctrl) {
                    -1.
                } else {
                    1.
                };
            state.stroke = Some(Draft {
                document: service.engine.scene.meshes.clone(),
                revision: service.engine.revision,
                editor_revision: service.editor.revision,
                last_screen: None,
                instance: instance.id.clone(),
                base,
                stroke: meshes::Stroke {
                    kind,
                    centers: vec![],
                    radius: settings.radius,
                    strength,
                    direction: [0., 1., 0.],
                },
                valid: true,
                adaptive_detail: settings.adaptive_detail && kind == meshes::BrushKind::Draw,
                detail: settings.detail,
                effective_radius: settings.radius,
                start_world: h.position.into(),
                plane_normal: (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize(),
                inverse,
                path: StrokePath::default(),
            });
            state.error.clear();
            state.notice.clear();
        }
        if ui.input(|i| {
            i.pointer.primary_down() || i.pointer.button_released(egui::PointerButton::Primary)
        }) && let Some(draft) = &mut state.stroke
            && (draft.last_screen != Some(pos)
                || ui.input(|i| {
                    i.events
                        .iter()
                        .any(|e| matches!(e, egui::Event::PointerMoved(_)))
                }))
        {
            draft.last_screen = Some(pos);
            let outcome = (|| -> crate::model::Result<()> {
                if draft.stroke.kind == meshes::BrushKind::Grab {
                    if draft.stroke.centers.is_empty() {
                        draft
                            .stroke
                            .centers
                            .push(draft.inverse.transform_point3(draft.start_world).to_array());
                    }
                    let (x, y, w, h) = screen(view, pos, scale);
                    let (start, delta) = crate::picking::ray(camera, x, y, w, h)?;
                    let den = delta.dot(draft.plane_normal);
                    if den.abs() < 1e-6 {
                        return Ok(());
                    }
                    let world =
                        start + delta * ((draft.start_world - start).dot(draft.plane_normal) / den);
                    let displacement = draft.inverse.transform_vector3(world - draft.start_world);
                    if displacement.length() < 1e-5 {
                        return Ok(());
                    }
                    draft.stroke.strength =
                        (displacement.length() * settings.grab_strength).min(1.);
                    draft.stroke.direction = displacement.normalize().to_array();
                    let (result, feedback) = crate::mesh_sculpt_shapes::interactive(
                        &draft.base,
                        &draft.stroke,
                        false,
                        draft.detail,
                        |v, t| {
                            service.engine.scene.meshes.check_replacement_counts(
                                &draft.base.id,
                                v,
                                t,
                            )
                        },
                    )?;
                    draft.effective_radius = feedback.effective_radius;
                    state.notice = feedback_message(feedback, draft.stroke.radius);
                    *draft
                        .document
                        .assets
                        .iter_mut()
                        .find(|a| a.id == draft.base.id)
                        .unwrap() = result;
                } else if draft.stroke.centers.len() < 128 {
                    // Preserve all delivered positions, including press/release
                    // coalesced into one frame, rather than just the final cursor.
                    let positions = ui.input(|i| {
                        let mut started = press.is_none();
                        let mut positions = Vec::new();
                        for event in &i.events {
                            match event {
                                egui::Event::PointerButton {
                                    pos,
                                    button: egui::PointerButton::Primary,
                                    pressed,
                                    ..
                                } => {
                                    started = true;
                                    positions.push(*pos);
                                    if !pressed {
                                        break;
                                    }
                                }
                                egui::Event::PointerMoved(pos) if started => positions.push(*pos),
                                _ => {}
                            }
                        }
                        positions
                    });
                    let mut centers = Vec::new();
                    let limit = 128 - draft.stroke.centers.len();
                    for position in positions {
                        if centers.len() == limit {
                            break;
                        }
                        let point = if view.contains(position) {
                            hit(service, document, camera, view, position, scale)?
                                .filter(|h| h.instance == instance.id)
                                .map(|h| Vec3::from(h.local_position))
                        } else {
                            None
                        };
                        draft
                            .path
                            .advance(point, draft.stroke.radius * 0.15, &mut centers, limit);
                    }
                    if !centers.is_empty() {
                        let mut sample = draft.stroke.clone();
                        sample.centers = if sample.kind.frozen_surface() {
                            sample.centers.extend_from_slice(&centers);
                            sample.centers.clone()
                        } else {
                            centers.clone()
                        };
                        let source = if sample.kind.frozen_surface() {
                            &draft.base
                        } else {
                            draft.document.asset(&instance.asset)?
                        };
                        let result = if sample.kind == meshes::BrushKind::Draw {
                            let (asset, feedback) = crate::mesh_sculpt_shapes::interactive(
                                &draft.base,
                                &sample,
                                draft.adaptive_detail,
                                draft.detail,
                                |v, t| {
                                    service.engine.scene.meshes.check_replacement_counts(
                                        &draft.base.id,
                                        v,
                                        t,
                                    )
                                },
                            )?;
                            draft.effective_radius = feedback.effective_radius;
                            state.notice = feedback_message(feedback, sample.radius);
                            asset
                        } else {
                            mesh_tools::sculpt_preview(source, &sample)?
                        };
                        *draft
                            .document
                            .assets
                            .iter_mut()
                            .find(|a| a.id == draft.base.id)
                            .unwrap() = result;
                        draft.stroke.centers.extend(centers);
                    }
                }
                Ok(())
            })();
            if let Err(e) = outcome {
                draft.valid = false;
                state.error = if e.message.starts_with("Sculpt would collapse or reverse") {
                    if draft.stroke.kind == meshes::BrushKind::Grab {
                        "O arraste dobraria a superfície. Recue o cursor ou aumente o raio.".into()
                    } else {
                        "O traço dobraria a superfície. Reduza a força ou subdivida a malha.".into()
                    }
                } else {
                    e.message
                };
                if draft.stroke.kind != meshes::BrushKind::Grab {
                    state.cancel();
                }
            } else {
                draft.valid = true;
                state.error.clear();
            }
        }
    } else if let Some(draft) = &mut state.stroke {
        draft.path = StrokePath::default();
        draft.last_screen = None;
    }
    if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
        && let Some(draft) = state.stroke.take()
        && draft.valid
        && pointer_in.is_some()
        && !draft.stroke.centers.is_empty()
        && draft.document.asset(&draft.base.id).ok() != Some(&draft.base)
    {
        // A save shortcut can share this input pass with mouse-up. Publish the
        // stroke first so the serialized scene includes the completed gesture.
        let edit = if matches!(
            draft.stroke.kind,
            meshes::BrushKind::Draw | meshes::BrushKind::Grab
        ) {
            meshes::Edit::SculptStroke {
                asset: draft.base.id,
                reference: service.engine.mesh_reference(),
                adaptive_detail: draft.adaptive_detail,
                detail: draft.detail,
                stroke: draft.stroke,
            }
        } else {
            meshes::Edit::Sculpt {
                asset: draft.base.id,
                stroke: draft.stroke,
            }
        };
        commands.insert(0, Command::EditMesh { edit });
    }
}
fn ring(
    ui: &egui::Ui,
    view: Rect,
    camera: &crate::model::Camera,
    hit: &mesh_tools::Hit,
    radius: f32,
    scale: f32,
    color: Color32,
) {
    let (_, _, w, h) = screen(view, view.center(), scale);
    let projection = crate::picking::view_projection(camera, w, h);
    let normal = Vec3::from(hit.normal);
    let axis = normal
        .cross(if normal.y.abs() < 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        })
        .normalize();
    let other = normal.cross(axis);
    let center = Vec3::from(hit.position);
    let mut points = Vec::with_capacity(49);
    for i in 0..=48 {
        let a = std::f32::consts::TAU * i as f32 / 48.;
        let p = projection.project_point3(center + radius * (axis * a.cos() + other * a.sin()));
        if !p.is_finite() {
            continue;
        }
        points.push(Pos2::new(
            view.min.x + (p.x + 1.) * 0.5 * view.width(),
            view.min.y + (1. - p.y) * 0.5 * view.height(),
        ));
    }
    ui.painter()
        .with_clip_rect(view)
        .add(egui::Shape::line(points, egui::Stroke::new(1.8_f32, color)));
}

fn component_properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
    instance: &meshes::Instance,
    asset: &meshes::Asset,
) {
    if super::modeling_ui::panel(ui, service, &mut state.modeling, commands) {
        return;
    }
    ui.heading("Editar geometria");
    let mut changed_kind = false;
    ui.horizontal_wrapped(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        for (i, label) in [(0, "Vértices"), (1, "Arestas"), (2, "Faces")] {
            if ui
                .selectable_value(&mut state.component_kind, i, label)
                .clicked()
            {
                state.vertices.clear();
                state.component_selection = None;
                changed_kind = true;
            }
        }
    });
    ui.small("Clique ou arraste um retângulo. Shift acrescenta; clique no vazio limpa. Edição na malha de repouso.");
    if changed_kind {
        commands.push(Command::SelectMeshComponents { selection: None });
        return;
    }
    if state.component_revision != service.engine.revision {
        state.vertices.clear();
        state.component_selection = None;
        state.component_revision = service.engine.revision;
    }
    super::modeling_ui::actions(ui, service, &mut state.modeling, commands);
    if state.modeling.active() {
        return;
    }
    if state.vertices.is_empty() {
        ui.weak("Selecione um componente para mover, subdividir, suavizar ou atribuir pesos.");
        return;
    }
    if state.reselect_after_edit && service.engine.selected_mesh_components().is_none() {
        commands.push(Command::SelectMeshComponents {
            selection: Some(crate::mesh_topology::Selection {
                asset: asset.id.clone(),
                reference: service.engine.mesh_reference(),
                components: state.component_selection.clone().unwrap(),
            }),
        });
    }
    state.reselect_after_edit = false;
    let description = match state.component_selection.as_ref().unwrap() {
        crate::mesh_topology::Components::Vertices { indices } => {
            format!("Vértices selecionados: {}", indices.len())
        }
        crate::mesh_topology::Components::Edges { vertices } => {
            format!("Arestas selecionadas: {}", vertices.len())
        }
        crate::mesh_topology::Components::Faces { indices } => {
            format!("Faces selecionadas: {}", indices.len())
        }
    };
    ui.strong(description);
    let center = state
        .vertices
        .iter()
        .map(|i| Vec3::from(asset.geometry.positions[*i as usize]))
        .sum::<Vec3>()
        / state.vertices.len() as f32;
    let mut next = center.to_array();
    ui.label("Centro da seleção · local (m)");
    if vec3(ui, &mut next, 0.001) {
        commands.push(live_edit(
            ui,
            vec![crate::engine::SceneEdit::Mesh {
                edit: meshes::Edit::TransformVertices {
                    asset: asset.id.clone(),
                    reference: service.engine.mesh_reference(),
                    vertices: state.vertices.clone(),
                    transform: meshes::IslandTransform {
                        pivot: center.to_array(),
                        translation: (Vec3::from(next) - center).to_array(),
                        rotation: [0., 0., 0., 1.],
                        scale: 1.,
                    },
                },
            }],
        ));
        state.reselect_after_edit = true;
    }
    if !state.component_error.is_empty() {
        ui.colored_label(Color32::LIGHT_RED, "Movimento não aplicado")
            .on_hover_text(&state.component_error);
        ui.small(if state.component_error.contains("collapses or reverses") {
            "O deslocamento inverteria ou colapsaria uma face. Use um valor menor ou Shift para ajuste fino."
        } else {
            "A última posição válida foi preservada. Passe o mouse sobre o aviso para ver o motivo."
        });
    }
    if design_ui::action(ui, "Subdividir seleção").clicked() {
        commands.push(Command::EditMesh {
            edit: meshes::Edit::Subdivide {
                asset: asset.id.clone(),
                reference: service.engine.mesh_reference(),
                components: state.component_selection.clone(),
            },
        });
    }
    if design_ui::action(ui, "Suavizar seleção").clicked() {
        commands.push(Command::EditMesh {
            edit: meshes::Edit::SmoothRegion {
                asset: asset.id.clone(),
                reference: service.engine.mesh_reference(),
                vertices: state.vertices.clone(),
                anchors: vec![],
                options: crate::mesh_surface::Options {
                    iterations: 2,
                    strength: 0.3,
                    max_displacement: 0.05,
                },
            },
        });
    }
    ui.small(format!(
        "Instância {} · edição compartilhada pelo asset {}",
        instance.id, asset.id
    ));
}
fn rig_properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
    instance: &meshes::Instance,
    asset: &meshes::Asset,
) {
    egui::CollapsingHeader::new("Esqueleto, pesos e animação").default_open(asset.rig.is_some()).open(state.rig_mode.then_some(true)).show(ui,|ui|{
        let Some(rig)=&asset.rig else {
            ui.small("Comece com um osso raiz; adicione filhos e atribua os vértices selecionados a cada osso.");
            if design_ui::action(ui,"Criar esqueleto").clicked(){
                let rig=meshes::Rig {bones:vec![meshes::Bone {name:"raiz".into(),parent:None,translation:[0.;3],rotation:[0.,0.,0.,1.]}],weights:vec![meshes::Weight { extra: None,joints:[0;4],weights:[1.,0.,0.,0.]};asset.geometry.positions.len()]};
                commands.push(Command::EditMesh {edit:meshes::Edit::SetRig {asset:asset.id.clone(),rig:Some(std::sync::Arc::new(rig))}});
                commands.push(Command::SetEditorMeshView {view:crate::editor::MeshView {bones:true,..service.mesh_view}});
            }
            return
        };
        state.bone=state.bone.min(rig.bones.len()-1);
        egui::ComboBox::from_id_salt("bone").selected_text(&rig.bones[state.bone].name).show_ui(ui,|ui|{
            for (i,bone) in rig.bones.iter().enumerate(){ui.selectable_value(&mut state.bone,i,&bone.name);}
        });
        let mut bone=rig.bones[state.bone].clone();
        ui.label("Nome do osso");ui.text_edit_singleline(&mut bone.name);
        ui.label("Posição de repouso · relativa ao pai");vec3(ui,&mut bone.translation,0.01);
        let (x,y,z)=Quat::from_array(bone.rotation).to_euler(glam::EulerRot::XYZ);let mut angles=[x.to_degrees(),y.to_degrees(),z.to_degrees()];
        ui.label("Rotação de repouso · °");if vec3(ui,&mut angles,0.5){bone.rotation=Quat::from_euler(glam::EulerRot::XYZ,angles[0].to_radians(),angles[1].to_radians(),angles[2].to_radians()).to_array();}
        if bone!=rig.bones[state.bone] {
            let mut next=(**rig).clone();next.bones[state.bone]=bone;
            commands.push(live_edit(ui,vec![crate::engine::SceneEdit::Mesh {edit:meshes::Edit::SetRig {asset:asset.id.clone(),rig:Some(std::sync::Arc::new(next))}}]));
        }
        if ui.add_enabled(rig.bones.len()<meshes::MAX_BONES,egui::Button::new("+ Osso filho")).clicked(){
            let mut next=(**rig).clone();next.bones.push(meshes::Bone {name:format!("osso_{}",next.bones.len()),parent:Some(state.bone as u8),translation:[0.,0.25,0.],rotation:[0.,0.,0.,1.]});
            commands.push(Command::EditMesh {edit:meshes::Edit::SetRig {asset:asset.id.clone(),rig:Some(std::sync::Arc::new(next))}});
        }
        ui.label("Peso deste osso na seleção · restante na raiz");
        ui.add_sized([ui.available_width(),26.],design_ui::Scrub::new(&mut state.weight).range(0.0..=1.).speed(0.01).percent());
        if ui.add_enabled(!state.vertices.is_empty(),egui::Button::new("Atribuir peso à seleção")).on_disabled_hover_text("Selecione vértices no modo Editar, depois volte a Rig.").clicked(){
            let mut next=(**rig).clone();for &i in &state.vertices {if let Some(w)=next.weights.get_mut(i as usize){*w=if state.bone==0 {meshes::Weight { extra: None,joints:[0;4],weights:[1.,0.,0.,0.]}} else {meshes::Weight { extra: None,joints:[state.bone as u8,0,0,0],weights:[state.weight,1.-state.weight,0.,0.]}};}}
            commands.push(Command::EditMesh {edit:meshes::Edit::SetRig {asset:asset.id.clone(),rig:Some(std::sync::Arc::new(next))}});
        }
        ui.small("Ossos editam o repouso. Pesos não são gerados automaticamente por proximidade.");
        ui.separator();ui.strong("Clipes de animação");
        for clip in &asset.clips {
            ui.horizontal(|ui|{ui.label(format!("{} · {} ticks · {} trilhas",clip.id,clip.duration_ticks,clip.tracks.len()));
                if ui.button("Usar clipe").clicked(){let mut next=instance.clone();next.playback=Some(meshes::Playback {clip:clip.id.clone(),offset_ticks:0,looping:true});commands.push(Command::EditMesh {edit:meshes::Edit::PutInstance {instance:next}});}
            });
        }
        if ui.button("Criar clipe vazio · 120 ticks").clicked(){
            let id=(0..1024).map(|i|format!("animacao_{i}")).find(|id|!asset.clips.iter().any(|c|c.id==*id)).unwrap();
            commands.push(Command::EditMesh {edit:meshes::Edit::PutClip {asset:asset.id.clone(),clip:std::sync::Arc::new(meshes::Clip {id,duration_ticks:120,tracks:vec![meshes::Track {bone:state.bone as u8,keys:vec![meshes::Keyframe {tick:0,translation:rig.bones[state.bone].translation,rotation:rig.bones[state.bone].rotation}]}]})}});
        }
        if let Some(clip)=asset.clips.first() {
            ui.label(format!("Editar chaves · {}",clip.id));
            ui.add(egui::Slider::new(&mut state.clip_tick,0..=clip.duration_ticks-1).text("Quadro (ticks)"));
            let key=(asset.id.clone(),clip.id.clone(),state.bone,state.clip_tick);
            if state.clip_key.as_ref()!=Some(&key) {
                let existing=clip.tracks.iter().find(|t|t.bone==state.bone as u8).and_then(|t|t.keys.iter().find(|k|k.tick==state.clip_tick));
                let (translation,rotation)=existing.map(|k|(k.translation,k.rotation)).unwrap_or((rig.bones[state.bone].translation,rig.bones[state.bone].rotation));
                state.clip_translation=translation;let (x,y,z)=Quat::from_array(rotation).to_euler(glam::EulerRot::XYZ);state.clip_angles=[x.to_degrees(),y.to_degrees(),z.to_degrees()];state.clip_key=Some(key);
            }
            ui.label("Translação local da chave");vec3(ui,&mut state.clip_translation,0.01);
            ui.label("Rotação da chave · °");vec3(ui,&mut state.clip_angles,0.5);
            if ui.button("Inserir / substituir chave do osso").clicked(){
                let mut next=(**clip).clone();
                if !next.tracks.iter().any(|t|t.bone==state.bone as u8){next.tracks.push(meshes::Track {bone:state.bone as u8,keys:vec![]});}
                let track=next.tracks.iter_mut().find(|t|t.bone==state.bone as u8).unwrap();
                track.keys.retain(|k|k.tick!=state.clip_tick);
                track.keys.push(meshes::Keyframe {tick:state.clip_tick,translation:state.clip_translation,rotation:Quat::from_euler(glam::EulerRot::XYZ,state.clip_angles[0].to_radians(),state.clip_angles[1].to_radians(),state.clip_angles[2].to_radians()).to_array()});
                track.keys.sort_by_key(|k|k.tick);
                commands.push(Command::EditMesh {edit:meshes::Edit::PutClip {asset:asset.id.clone(),clip:std::sync::Arc::new(next)}});
            }
        }
        ui.small("Atribua o clipe à instância e execute a simulação para reproduzir. Edição de repouso é separada das chaves.");
    });
}
fn overlays(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
    response: &egui::Response,
    view: Rect,
) {
    if !service.mesh_view.edges
        && !service.mesh_view.vertices
        && !service.mesh_view.bones
        && !state.components
    {
        return;
    }
    let Some(instance) = selected(service) else {
        return;
    };
    let Ok(asset) = service.engine.scene.meshes.asset(&instance.asset) else {
        return;
    };
    let Ok(pose) = service
        .engine
        .scene
        .meshes
        .pose(&instance.id, service.engine.simulation.tick)
    else {
        return;
    };
    let camera = service
        .editor
        .current()
        .camera
        .as_ref()
        .unwrap_or(&service.engine.scene.camera);
    let scale = ui.ctx().pixels_per_point();
    let (_, _, w, h) = screen(view, view.center(), scale);
    let projection = match crate::mesh_screen::Projection::new(camera, w, h) {
        Ok(p) => p,
        Err(e) => {
            state.error = e.to_string();
            return;
        }
    };
    if !service.mesh_view.xray
        && (service.mesh_view.edges || service.mesh_view.vertices || state.components)
        && let Err(e) = state.visibility.prepare(&service.engine)
    {
        state.error = e.to_string();
        return;
    }
    let project = |p: Vec3| {
        projection.project(p).map(|p| {
            Pos2::new(
                view.left() + p.x / w as f32 * view.width(),
                view.top() + p.y / h as f32 * view.height(),
            )
        })
    };
    let points: Vec<_> = pose
        .positions
        .iter()
        .map(|p| project(Vec3::from(*p)))
        .collect();
    let painter = ui.painter().with_clip_rect(view);
    if service.mesh_view.edges {
        let key = std::sync::Arc::as_ptr(&asset.geometry) as usize;
        if state.overlay_key != key {
            state.overlay_edges = crate::mesh_topology::Topology::inspect(&asset.geometry)
                .map(|t| t.edges.into_iter().map(|e| e.vertices).collect())
                .unwrap_or_default();
            state.overlay_key = key;
        }
        for &[a, b] in &state.overlay_edges {
            draw_visible_edge(
                &painter,
                view,
                &projection,
                &state.visibility,
                Vec3::from(pose.positions[a as usize]),
                Vec3::from(pose.positions[b as usize]),
                service.mesh_view.xray,
                [w, h],
                Color32::from_rgba_unmultiplied(175, 192, 210, 150),
            );
        }
    }
    if service.mesh_view.vertices || state.components {
        for (i, p) in points.iter().enumerate() {
            if let Some(p) = p
                && view.contains(*p)
                && (service.mesh_view.xray
                    || state
                        .visibility
                        .visible(&projection, Vec3::from(pose.positions[i])))
            {
                let selected = state.vertices.binary_search(&(i as u32)).is_ok();
                painter.circle_filled(
                    *p,
                    if selected { 3.5 } else { 1.6 },
                    if selected {
                        EMBER
                    } else {
                        Color32::from_rgb(130, 157, 180)
                    },
                );
            }
        }
    }
    if service.mesh_view.bones
        && let Some(rig) = &asset.rig
    {
        let world = Mat4::from_cols_array_2d(&pose.world_transform);
        for (i, bone) in rig.bones.iter().enumerate() {
            let p = world.transform_point3(
                Mat4::from_cols_array_2d(&pose.joints[i]).transform_point3(Vec3::ZERO),
            );
            if let Some(b) = project(p) {
                painter.circle_filled(b, 4., EMBER);
                if let Some(parent) = bone.parent {
                    let a = world.transform_point3(
                        Mat4::from_cols_array_2d(&pose.joints[parent as usize])
                            .transform_point3(Vec3::ZERO),
                    );
                    if let Some(a) = project(a) {
                        painter.line_segment([a, b], egui::Stroke::new(3_f32, EMBER));
                    }
                }
            }
        }
    }
    if !state.components || service.engine.playing {
        state.marquee = None;
        return;
    }
    if state.marquee.as_ref().is_some_and(|m| m.view != view) {
        state.marquee = None;
        return;
    }
    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(start) = ui
            .input(|i| i.pointer.press_origin())
            .filter(|p| view.contains(*p))
    {
        state.marquee = Some(Marquee {
            start,
            end: start,
            view,
            revision: service.engine.revision,
            editor_revision: service.editor.revision,
            session_revision: service.session_revision,
            instance: instance.id.clone(),
            additive: ui.input(|i| i.modifiers.shift),
            kind: match state.component_kind {
                0 => crate::mesh_screen::Kind::Vertices,
                1 => crate::mesh_screen::Kind::Edges,
                _ => crate::mesh_screen::Kind::Faces,
            },
        });
    }
    if let Some(m) = &mut state.marquee {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            m.end = pos.clamp(view.min, view.max);
        }
        let rect = Rect::from_two_pos(m.start, m.end).intersect(view);
        painter.rect_filled(rect, 0, Color32::from_rgba_unmultiplied(239, 158, 103, 22));
        painter.rect_stroke(
            rect,
            0,
            egui::Stroke::new(1.2_f32, EMBER),
            egui::StrokeKind::Inside,
        );
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            let m = state.marquee.take().unwrap();
            commands.push(Command::SelectEditorMeshRegion {
                instance: m.instance,
                reference: service.engine.mesh_reference(),
                component: m.kind,
                min: [
                    (rect.left() - view.left()) / view.width() * w as f32,
                    (rect.top() - view.top()) / view.height() * h as f32,
                ],
                max: [
                    (rect.right() - view.left()) / view.width() * w as f32,
                    (rect.bottom() - view.top()) / view.height() * h as f32,
                ],
                width: w,
                height: h,
                additive: m.additive,
            });
        }
        return;
    }
    if response.clicked_by(egui::PointerButton::Primary)
        && let Some(pos) = response.interact_pointer_pos()
    {
        let hit = hit(
            service,
            &service.engine.scene.meshes,
            camera,
            view,
            pos,
            scale,
        )
        .ok()
        .flatten()
        .filter(|h| h.instance == instance.id);
        let Some(hit) = hit else {
            if !ui.input(|i| i.modifiers.shift) {
                state.vertices.clear();
                state.component_selection = None;
                state.reselect_after_edit = false;
                commands.push(Command::SelectMeshComponents { selection: None });
            }
            return;
        };
        let triangle = asset.geometry.triangles[hit.face as usize];
        let mut ranked = triangle;
        ranked.sort_by(|a, b| {
            let d = |i: u32| {
                points[i as usize]
                    .map(|p| p.distance_sq(pos))
                    .unwrap_or(f32::INFINITY)
            };
            d(*a).total_cmp(&d(*b))
        });
        use crate::mesh_topology::Components;
        let mut components = match state.component_kind {
            0 => Components::Vertices {
                indices: vec![ranked[0]],
            },
            1 => {
                let edge = [
                    [triangle[0], triangle[1]],
                    [triangle[1], triangle[2]],
                    [triangle[2], triangle[0]],
                ]
                .into_iter()
                .min_by(|a, b| {
                    let distance = |[a, b]: [u32; 2]| {
                        let (Some(a), Some(b)) = (points[a as usize], points[b as usize]) else {
                            return f32::INFINITY;
                        };
                        let segment = b - a;
                        let t = ((pos - a).dot(segment)
                            / segment.length_sq().max(f32::MIN_POSITIVE))
                        .clamp(0., 1.);
                        pos.distance_sq(a + segment * t)
                    };
                    distance(*a).total_cmp(&distance(*b))
                })
                .unwrap();
                Components::Edges {
                    vertices: vec![[edge[0].min(edge[1]), edge[0].max(edge[1])]],
                }
            }
            _ => Components::Faces {
                indices: vec![hit.face],
            },
        };
        if ui.input(|i| i.modifiers.shift) {
            match (&mut components, &state.component_selection) {
                (
                    Components::Vertices { indices },
                    Some(Components::Vertices { indices: previous }),
                )
                | (Components::Faces { indices }, Some(Components::Faces { indices: previous })) => {
                    indices.extend(previous);
                    indices.sort_unstable();
                    indices.dedup();
                }
                (
                    Components::Edges { vertices },
                    Some(Components::Edges { vertices: previous }),
                ) => {
                    vertices.extend(previous);
                    vertices.sort_unstable();
                    vertices.dedup();
                }
                _ => {}
            }
        }
        state.vertices = component_vertices(&components, &asset.geometry);
        state.component_selection = Some(components.clone());
        state.component_revision = service.engine.revision;
        commands.push(Command::SelectMeshComponents {
            selection: Some(crate::mesh_topology::Selection {
                asset: asset.id.clone(),
                reference: service.engine.mesh_reference(),
                components,
            }),
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_visible_edge(
    painter: &egui::Painter,
    view: Rect,
    projection: &crate::mesh_screen::Projection,
    visibility: &crate::mesh_screen::Visibility,
    a: Vec3,
    b: Vec3,
    xray: bool,
    size: [u32; 2],
    color: Color32,
) {
    let project = |p: Vec3| {
        projection.project(p).map(|p| {
            Pos2::new(
                view.left() + p.x / size[0] as f32 * view.width(),
                view.top() + p.y / size[1] as f32 * view.height(),
            )
        })
    };
    let (Some(pa), Some(pb)) = (project(a), project(b)) else {
        return;
    };
    if !Rect::from_two_pos(pa, pb).expand(1.).intersects(view) {
        return;
    }
    let stroke = egui::Stroke::new(0.6_f32, color);
    if xray {
        painter.line_segment([pa, pb], stroke);
        return;
    }
    // Subdivide only for occlusion. Coalesce visible runs into one painted segment.
    let steps = (pa.distance(pb) / 2.).ceil().clamp(1., 2048.) as u32;
    let mut previous_visible = visibility.visible(projection, a);
    let mut previous = pa;
    let mut run = None;
    for i in 1..=steps {
        let p = a.lerp(b, i as f32 / steps as f32);
        let next = project(p).unwrap();
        let visible = visibility.visible(projection, p);
        let middle = a.lerp(b, (i as f32 - 0.5) / steps as f32);
        if previous_visible && visible && visibility.visible(projection, middle) {
            if run.is_none() {
                run = Some(previous);
            }
        } else if let Some(start) = run.take() {
            painter.line_segment([start, previous], stroke);
        }
        previous = next;
        previous_visible = visible;
    }
    if let Some(start) = run {
        painter.line_segment([start, pb], stroke);
    }
}

fn component_vertices(
    components: &crate::mesh_topology::Components,
    geometry: &meshes::Geometry,
) -> Vec<u32> {
    use crate::mesh_topology::Components;
    let mut vertices = match components {
        Components::Vertices { indices } => indices.clone(),
        Components::Edges { vertices } => vertices.iter().flatten().copied().collect(),
        Components::Faces { indices } => indices
            .iter()
            .flat_map(|i| geometry.triangles[*i as usize])
            .collect(),
    };
    vertices.sort_unstable();
    vertices.dedup();
    vertices
}

fn feedback_message(feedback: crate::mesh_sculpt_shapes::Feedback, radius: f32) -> String {
    let mut messages = Vec::new();
    if feedback.detail_limited {
        messages.push(
            "Limite de detalhe atingido; escultura continua na resolução disponível.".to_owned(),
        );
    }
    if feedback.effective_radius > radius * 1.01 {
        messages.push(format!(
            "Raio efetivo {:.3} m para a resolução desta região.",
            feedback.effective_radius
        ));
    }
    if feedback.fraction < 0.999 {
        messages.push("Movimento limitado para preservar a superfície.".to_owned());
    }
    messages.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        tempfile::TempDir,
        Service,
        super::super::UiState,
        egui::Context,
    ) {
        fixture_resolution(16)
    }
    fn fixture_resolution(
        resolution: u16,
    ) -> (
        tempfile::TempDir,
        Service,
        super::super::UiState,
        egui::Context,
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let mut service = Service::new(
            tmp.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        service
            .execute(
                Command::CreateEditorMesh {
                    primitive: mesh_tools::Primitive::Sphere,
                    resolution,
                    size: 2.,
                }
                .into(),
            )
            .unwrap();
        let mut camera = service.engine.scene.camera.clone();
        camera.position = [2., 1.05, 6.];
        camera.target = [2., 1.05, 0.];
        service
            .execute(
                Command::EditEditor {
                    action: Action::Camera {
                        camera: Some(camera),
                    },
                    expected_editor_revision: None,
                }
                .into(),
            )
            .unwrap();
        service.editor.sculpt.adaptive_detail = false;
        let mut state = super::super::UiState::default();
        state.project.dismissed = true;
        state.mesh.sculpt = true;
        (tmp, service, state, egui::Context::default())
    }
    fn pass(
        ctx: &egui::Context,
        service: &Service,
        state: &mut super::super::UiState,
        events: Vec<egui::Event>,
        focused: bool,
    ) -> (Rect, Vec<Command>) {
        pass_sized(
            ctx,
            service,
            state,
            events,
            focused,
            Default::default(),
            Vec2::new(1440., 900.),
        )
    }
    fn pass_sized(
        ctx: &egui::Context,
        service: &Service,
        state: &mut super::super::UiState,
        events: Vec<egui::Event>,
        focused: bool,
        modifiers: egui::Modifiers,
        size: Vec2,
    ) -> (Rect, Vec<Command>) {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            focused,
            modifiers,
            ..Default::default()
        });
        let mut view = None;
        let mut commands = vec![];
        super::super::draw_editor(
            ctx,
            service,
            state,
            &mut commands,
            egui::TextureId::User(0),
            &mut view,
        );
        let output = ctx.end_pass();
        fn field(shape: &egui::Shape) -> Option<Rect> {
            match shape {
                egui::Shape::Text(t) if t.galley.text().starts_with("X  ") => {
                    Some(Rect::from_min_size(t.pos, t.galley.size()))
                }
                egui::Shape::Vec(shapes) => shapes.iter().find_map(field),
                _ => None,
            }
        }
        let x_field = output.shapes.iter().find_map(|s| field(&s.shape));
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("test_component_x"), x_field));
        fn circles(shape: &egui::Shape) -> usize {
            match shape {
                egui::Shape::Circle(c)
                    if c.radius == 1.6 && c.fill == Color32::from_rgb(130, 157, 180) =>
                {
                    1
                }
                egui::Shape::Vec(shapes) => shapes.iter().map(circles).sum(),
                _ => 0,
            }
        }
        let count = output
            .shapes
            .iter()
            .map(|s| circles(&s.shape))
            .sum::<usize>();
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("test_overlay_circles"), count));
        (view.unwrap(), commands)
    }
    fn button(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }
    }
    #[test]
    fn component_numeric_rejection_preserves_fields_and_recovers_without_flicker() {
        let (_tmp, mut service, mut state, ctx) = fixture_resolution(24);
        state.mesh.sculpt = false;
        state.mesh.components = true;
        let asset = service.selected_mesh.clone().unwrap();
        service
            .execute(
                Command::SelectMeshComponents {
                    selection: Some(crate::mesh_topology::Selection {
                        asset,
                        reference: service.engine.mesh_reference(),
                        components: crate::mesh_topology::Components::Vertices {
                            indices: (0..47).collect(),
                        },
                    }),
                }
                .into(),
            )
            .unwrap();
        for _ in 0..3 {
            pass(&ctx, &service, &mut state, vec![], true);
        }
        let field = |ctx: &egui::Context| {
            ctx.data(|d| {
                d.get_temp::<Option<Rect>>(egui::Id::new("test_component_x"))
                    .flatten()
            })
        };
        let start = field(&ctx).unwrap().center();
        pass(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::PointerMoved(start), button(start, true)],
            true,
        );
        let before = service.engine.snapshot();
        let (_, commands) = pass(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::PointerMoved(start + Vec2::new(300., 0.))],
            true,
        );
        let mut rejected = false;
        for command in commands {
            if let Err(error) = service.execute(command.into()) {
                assert!(error.message.contains("reverses"), "{error:?}");
                state.mesh.command_error(&error.message);
                rejected = true;
                break;
            }
        }
        assert!(rejected, "Fixture must exercise a refused vertex transform");
        assert_eq!(service.engine.snapshot(), before);
        assert!(state.mesh.component_error.contains("reverses"));
        pass(
            &ctx,
            &service,
            &mut state,
            vec![button(start + Vec2::new(300., 0.), false)],
            true,
        );
        assert!(
            field(&ctx).is_some(),
            "Rejected drag must not remove the numeric fields for one frame"
        );
        for _ in 0..3 {
            let (_, commands) = pass(&ctx, &service, &mut state, vec![], true);
            assert!(commands.is_empty(), "Idle must not retry a rejected edit");
            assert!(field(&ctx).is_some());
            assert_eq!(state.mesh.vertices.len(), 47);
        }
        let start = field(&ctx).unwrap().center();
        pass(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::PointerMoved(start), button(start, true)],
            true,
        );
        let mut edits = 0;
        for dx in [7., 8., 9., 10.] {
            let (_, commands) = pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(start + Vec2::new(dx, 0.))],
                true,
            );
            for command in commands {
                if matches!(command, Command::EditSceneLive { .. }) {
                    edits += 1;
                }
                service.execute(command.into()).unwrap();
            }
            assert!(field(&ctx).is_some());
        }
        assert!(edits > 1, "Continuous drag must publish multiple updates");
        assert_ne!(service.engine.snapshot()["scene"], before["scene"]);
        assert_eq!(
            service.engine.snapshot()["history"]["undo"].as_u64(),
            Some(before["history"]["undo"].as_u64().unwrap() + 1)
        );
        service.execute(Command::Undo.into()).unwrap();
        assert_eq!(service.engine.snapshot()["scene"], before["scene"]);
    }
    #[test]
    fn rectangle_empty_click_shift_and_cancellation_use_real_pointer_input() {
        let (_tmp, mut service, mut state, ctx) = fixture();
        state.mesh.sculpt = false;
        state.mesh.components = true;
        let size = Vec2::new(1440., 900.);
        pass(&ctx, &service, &mut state, vec![], true);
        let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
        // Press/drag/release is one region command, not a world edit.
        let start = view.center() - Vec2::new(80., 60.);
        let end = view.center() + Vec2::new(80., 60.);
        pass(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::PointerMoved(start), button(start, true)],
            true,
        );
        pass(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::PointerMoved(end)],
            true,
        );
        assert!(state.mesh.marquee.is_some());
        let (_, commands) = pass(&ctx, &service, &mut state, vec![button(end, false)], true);
        assert!(matches!(
            commands.as_slice(),
            [Command::SelectEditorMeshRegion {
                additive: false,
                ..
            }]
        ));
        for command in commands {
            service.execute(command.into()).unwrap();
        }
        let selected = service.engine.selected_mesh_components().unwrap().clone();
        assert!(crate::mesh_screen::count(&selected.components) > 1);
        let authored = service.engine.scene.clone();
        let history = service.engine.snapshot()["history"].clone();
        let empty = view.left_top() + Vec2::splat(10.);
        let shift = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        pass_sized(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::PointerMoved(empty), button(empty, true)],
            true,
            shift,
            size,
        );
        let (_, commands) = pass_sized(
            &ctx,
            &service,
            &mut state,
            vec![button(empty, false)],
            true,
            shift,
            size,
        );
        assert!(commands.is_empty());
        assert_eq!(service.engine.selected_mesh_components(), Some(&selected));
        pass(&ctx, &service, &mut state, vec![button(empty, true)], true);
        let (_, commands) = pass(&ctx, &service, &mut state, vec![button(empty, false)], true);
        assert!(matches!(
            commands.as_slice(),
            [Command::SelectMeshComponents { selection: None }]
        ));
        for command in commands {
            service.execute(command.into()).unwrap();
        }
        for _ in 0..2 {
            assert!(pass(&ctx, &service, &mut state, vec![], true).1.is_empty());
        }
        assert!(
            state.mesh.vertices.is_empty() && service.engine.selected_mesh_components().is_none(),
            "Cleared selection must not be resurrected by the inspector"
        );
        for cancellation in 0..4 {
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(start), button(start, true)],
                true,
            );
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(end)],
                true,
            );
            assert!(state.mesh.marquee.is_some());
            let (events, focused, next_size) = match cancellation {
                0 => (
                    vec![egui::Event::Key {
                        key: egui::Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    }],
                    true,
                    size,
                ),
                1 => (vec![], false, size),
                2 => (vec![], true, Vec2::new(1000., 900.)),
                _ => {
                    let mut camera = service.editor.current().camera.clone().unwrap();
                    camera.position[0] += 0.1;
                    service
                        .execute(
                            Command::EditEditor {
                                action: Action::Camera {
                                    camera: Some(camera),
                                },
                                expected_editor_revision: None,
                            }
                            .into(),
                        )
                        .unwrap();
                    (vec![], true, size)
                }
            };
            let (_, commands) = pass_sized(
                &ctx,
                &service,
                &mut state,
                events,
                focused,
                Default::default(),
                next_size,
            );
            assert!(state.mesh.marquee.is_none());
            assert!(
                !commands
                    .iter()
                    .any(|c| matches!(c, Command::SelectEditorMeshRegion { .. }))
            );
            pass_sized(
                &ctx,
                &service,
                &mut state,
                vec![button(end, false)],
                true,
                Default::default(),
                next_size,
            );
            pass(&ctx, &service, &mut state, vec![], true);
        }
        assert_eq!(service.engine.scene, authored);
        assert_eq!(service.engine.snapshot()["history"], history);
    }
    #[test]
    fn surface_overlay_hides_back_vertices_and_foreign_occluders() {
        let (_tmp, mut service, mut state, ctx) = fixture();
        state.mesh.sculpt = false;
        state.mesh.components = true;
        pass(&ctx, &service, &mut state, vec![], true);
        pass(&ctx, &service, &mut state, vec![], true);
        let count = || {
            ctx.data(|d| {
                d.get_temp::<usize>(egui::Id::new("test_overlay_circles"))
                    .unwrap()
            })
        };
        let front = count();
        assert!(front > 20);
        service
            .execute(
                Command::SetEditorMeshView {
                    view: crate::editor::MeshView {
                        xray: true,
                        ..Default::default()
                    },
                }
                .into(),
            )
            .unwrap();
        pass(&ctx, &service, &mut state, vec![], true);
        let all = count();
        assert!(all > front);
        let mut wall = service
            .engine
            .scene
            .entities
            .iter()
            .find(|e| e.id == "floor")
            .unwrap()
            .clone();
        wall.id = "occluder".into();
        wall.transform.position = [2., 1.05, 3.];
        wall.transform.size = [4., 4., 0.1];
        service
            .execute(
                Command::Upsert {
                    entity: Box::new(wall),
                }
                .into(),
            )
            .unwrap();
        service
            .execute(
                Command::SetEditorMeshView {
                    view: crate::editor::MeshView::default(),
                }
                .into(),
            )
            .unwrap();
        pass(&ctx, &service, &mut state, vec![], true);
        assert_eq!(
            count(),
            0,
            "Another object must hide the overlay, not only backfacing normals"
        );
        service
            .execute(
                Command::SetEditorMeshView {
                    view: crate::editor::MeshView {
                        xray: true,
                        ..Default::default()
                    },
                }
                .into(),
            )
            .unwrap();
        pass(&ctx, &service, &mut state, vec![], true);
        assert_eq!(count(), all);
    }
    #[test]
    fn selecting_a_character_leaves_mesh_modes_and_restores_object_picking() {
        for rig in [false, true] {
            let (_tmp, mut service, mut state, ctx) = fixture();
            state.mesh.components = !rig;
            state.mesh.rig_mode = rig;
            state.mesh.sculpt = false;
            pass(&ctx, &service, &mut state, vec![], true);
            service
                .execute(
                    Command::CreateSceneObject {
                        kind: crate::project::ObjectKind::Npc,
                    }
                    .into(),
                )
                .unwrap();
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            assert!(!state.mesh.components && !state.mesh.rig_mode && !state.mesh.sculpt);
            let pos = view.center();
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(pos), button(pos, true)],
                true,
            );
            let (_, commands) = pass(&ctx, &service, &mut state, vec![button(pos, false)], true);
            assert!(
                commands
                    .iter()
                    .any(|c| matches!(c, Command::SelectEditorAt { .. })),
                "A previous mesh mode must not disable actor picking"
            );
        }
    }
    #[test]
    fn rejected_live_mesh_draft_waits_for_an_actual_change() {
        let (_tmp, mut service, mut state, ctx) = fixture();
        state.mesh.sculpt = false;
        pass(&ctx, &service, &mut state, vec![], true);
        pass(&ctx, &service, &mut state, vec![], true);
        state
            .mesh
            .properties
            .as_mut()
            .unwrap()
            .material
            .transmission = -1.;
        let before = service.engine.snapshot();
        let (_, commands) = pass(&ctx, &service, &mut state, vec![], true);
        assert_eq!(commands.len(), 1);
        assert!(
            service
                .execute(commands.into_iter().next().unwrap().into())
                .is_err()
        );
        for _ in 0..4 {
            assert!(pass(&ctx, &service, &mut state, vec![], true).1.is_empty());
        }
        assert_eq!(service.engine.snapshot(), before);
        state
            .mesh
            .properties
            .as_mut()
            .unwrap()
            .material
            .transmission = 0.2;
        let (_, commands) = pass(&ctx, &service, &mut state, vec![], true);
        assert_eq!(commands.len(), 1);
        service
            .execute(commands.into_iter().next().unwrap().into())
            .unwrap();
        assert!(pass(&ctx, &service, &mut state, vec![], true).1.is_empty());
    }
    #[test]
    fn viewport_preserves_edge_and_face_selection_for_refinement() {
        use crate::mesh_topology::Components;
        for kind in 0..=2 {
            let (_tmp, mut service, mut state, ctx) = fixture();
            state.mesh.sculpt = false;
            state.mesh.components = true;
            state.mesh.component_kind = kind;
            pass(&ctx, &service, &mut state, vec![], true);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let pos = view.center();
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(pos), button(pos, true)],
                true,
            );
            let (_, commands) = pass(&ctx, &service, &mut state, vec![button(pos, false)], true);
            let selection = commands
                .into_iter()
                .find_map(|command| match command {
                    Command::SelectMeshComponents {
                        selection: Some(selection),
                    } => Some(selection),
                    _ => None,
                })
                .expect("Surface click should select a component");
            match (kind, &selection.components) {
                (0, Components::Vertices { indices }) => assert_eq!(indices.len(), 1),
                (1, Components::Edges { vertices }) => assert_eq!(vertices.len(), 1),
                (2, Components::Faces { indices }) => assert_eq!(indices.len(), 1),
                _ => panic!("UI changed the selection domain"),
            }
            service
                .execute(
                    Command::SelectMeshComponents {
                        selection: Some(selection.clone()),
                    }
                    .into(),
                )
                .unwrap();
            pass(&ctx, &service, &mut state, vec![], true);
            assert_eq!(
                state.mesh.component_selection.as_ref(),
                Some(&selection.components)
            );
            if kind > 0 {
                let before = service
                    .engine
                    .scene
                    .meshes
                    .asset(&selection.asset)
                    .unwrap()
                    .geometry
                    .positions
                    .len();
                service
                    .execute(
                        Command::EditMesh {
                            edit: meshes::Edit::Subdivide {
                                asset: selection.asset.clone(),
                                reference: selection.reference,
                                components: state.mesh.component_selection.clone(),
                            },
                        }
                        .into(),
                    )
                    .unwrap();
                let after = service
                    .engine
                    .scene
                    .meshes
                    .asset(&selection.asset)
                    .unwrap()
                    .geometry
                    .positions
                    .len();
                assert_eq!(
                    after - before,
                    if kind == 1 { 1 } else { 3 },
                    "Refine only the selected edge/face, not every incident vertex edge"
                );
            }
        }
    }
    #[test]
    fn long_draw_on_default_sphere_is_a_ridge_independent_of_event_density() {
        let mut outputs = Vec::new();
        for pixels in [1, 8] {
            let (_tmp, mut service, mut state, ctx) = fixture_resolution(24);
            pass(&ctx, &service, &mut state, vec![], true);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let original = service.engine.scene.clone();
            let start = view.center() - Vec2::new(80., 0.);
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(start), button(start, true)],
                true,
            );
            for x in (pixels..=160).step_by(pixels) {
                pass(
                    &ctx,
                    &service,
                    &mut state,
                    vec![egui::Event::PointerMoved(start + Vec2::new(x as f32, 0.))],
                    true,
                );
            }
            let draft = state.mesh.stroke.as_ref().unwrap();
            let preview = draft.document.clone();
            let base = &draft.base.geometry;
            let geometry = &preview.asset(&draft.base.id).unwrap().geometry;
            let max_radius = geometry
                .positions
                .iter()
                .map(|p| Vec3::from(*p).length())
                .fold(0_f32, f32::max);
            let min_radius = geometry
                .positions
                .iter()
                .map(|p| Vec3::from(*p).length())
                .fold(f32::INFINITY, f32::min);
            let mut max_stretch = 0_f32;
            for face in &geometry.triangles {
                let p = face.map(|i| Vec3::from(geometry.positions[i as usize]));
                let q = face.map(|i| Vec3::from(base.positions[i as usize]));
                let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
                let original_normal = (q[1] - q[0]).cross(q[2] - q[0]).normalize();
                assert!(
                    normal.dot(original_normal) > 0.1,
                    "folded face at {pixels}px/event"
                );
                for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                    max_stretch = max_stretch.max(p[a].distance(p[b]) / q[a].distance(q[b]));
                }
            }
            println!(
                "draw {pixels}px/event: {} stamps, radius {min_radius}..{max_radius}, edge stretch {max_stretch}",
                draft.stroke.centers.len()
            );
            assert!(
                max_radius > 1.1 && max_radius < 1.5,
                "raised stroke without runaway"
            );
            assert!(min_radius > 0.999, "positive Draw must not cave in");
            assert!(max_stretch < 2.8, "no needle-like edges");
            assert!(
                draft.stroke.centers.len() > 16,
                "fast strokes must not truncate at 16 stamps"
            );
            let end = start + Vec2::new(160., 0.);
            let (_, commands) = pass(&ctx, &service, &mut state, vec![button(end, false)], true);
            assert_eq!(commands.len(), 1);
            service
                .execute(commands.into_iter().next().unwrap().into())
                .unwrap();
            assert_eq!(
                service.engine.scene.meshes, preview,
                "live preview = shared MCP kernel"
            );
            outputs.push(
                service.engine.scene.meshes.assets[0]
                    .geometry
                    .positions
                    .clone(),
            );
            service.execute(Command::Undo.into()).unwrap();
            assert_eq!(service.engine.scene, original);
            service.execute(Command::Redo.into()).unwrap();
            assert_eq!(service.engine.scene.meshes, preview);
        }
        let error = outputs[0]
            .iter()
            .zip(&outputs[1])
            .map(|(a, b)| Vec3::from(*a).distance(Vec3::from(*b)))
            .fold(0_f32, f32::max);
        println!("dense/sparse stroke maximum deviation: {error}m");
        assert!(error < 0.005, "event cadence changed the stroke: {error}m");
    }
    #[test]
    fn diagonal_pointer_strokes_in_both_directions_remain_unfolded() {
        for reverse in [false, true] {
            let (_tmp, service, mut state, ctx) = fixture_resolution(24);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let path: Vec<_> = (0..=40)
                .map(|i| view.center() + Vec2::new(-80. + i as f32 * 4., -35. + i as f32 * 1.75))
                .collect();
            let path: Vec<_> = if reverse {
                path.into_iter().rev().collect()
            } else {
                path
            };
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(path[0]), button(path[0], true)],
                true,
            );
            for p in &path[1..] {
                pass(
                    &ctx,
                    &service,
                    &mut state,
                    vec![egui::Event::PointerMoved(*p)],
                    true,
                );
            }
            let draft = state.mesh.stroke.as_ref().unwrap();
            let geometry = &draft.document.asset(&draft.base.id).unwrap().geometry;
            let camera = service
                .editor
                .current()
                .camera
                .as_ref()
                .unwrap_or(&service.engine.scene.camera);
            let local: Vec<_> = path
                .iter()
                .map(|p| {
                    Vec3::from(
                        hit(
                            &service,
                            &service.engine.scene.meshes,
                            camera,
                            view,
                            *p,
                            ctx.pixels_per_point(),
                        )
                        .unwrap()
                        .expect("diagonal must remain on the surface")
                        .local_position,
                    )
                })
                .collect();
            let length: f32 = local.windows(2).map(|p| p[0].distance(p[1])).sum();
            let expected = (1 + (length / (draft.stroke.radius * 0.15)).floor() as usize).min(128);
            assert_eq!(
                draft.stroke.centers.len(),
                expected,
                "entire surface path must be sampled, reverse={reverse}, length={length}"
            );
            for face in &geometry.triangles {
                let [a, b, c] = face.map(|i| Vec3::from(geometry.positions[i as usize]));
                let [p, q, r] = face.map(|i| Vec3::from(draft.base.geometry.positions[i as usize]));
                assert!(
                    (b - a)
                        .cross(c - a)
                        .normalize()
                        .dot((q - p).cross(r - p).normalize())
                        > 0.1,
                    "folded diagonal, reverse={reverse}"
                );
            }
        }
    }
    #[test]
    fn coalesced_curved_path_retains_all_moves_and_does_not_bridge_misses() {
        let mut outputs = Vec::new();
        for coalesced in [false, true] {
            let (_tmp, service, mut state, ctx) = fixture_resolution(24);
            pass(&ctx, &service, &mut state, vec![], true);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let start = view.center();
            let mut events = vec![egui::Event::PointerMoved(start), button(start, true)];
            // A complete loop ends at its start, so final-cursor-only handling loses it.
            for offset in [
                Vec2::new(80., 0.),
                Vec2::new(80., -60.),
                Vec2::new(0., -60.),
                Vec2::ZERO,
            ] {
                events.push(egui::Event::PointerMoved(start + offset));
            }
            if coalesced {
                pass(&ctx, &service, &mut state, events, true);
            } else {
                for event in events {
                    pass(&ctx, &service, &mut state, vec![event], true);
                }
            }
            let draft = state.mesh.stroke.as_ref().unwrap();
            assert!(draft.stroke.centers.len() > 20);
            outputs.push(draft.document.clone());
        }
        assert_eq!(
            outputs[0], outputs[1],
            "batching input events cannot change sculpt geometry"
        );

        let mut path = StrokePath::default();
        let mut samples = vec![];
        path.advance(Some(Vec3::ZERO), 0.1, &mut samples, 128);
        path.advance(None, 0.1, &mut samples, 128);
        path.advance(Some(Vec3::X), 0.1, &mut samples, 128);
        assert_eq!(
            samples,
            vec![[0., 0., 0.], [1., 0., 0.]],
            "surface misses split the path"
        );
        path.advance(Some(Vec3::X * 1000.), 0.1, &mut samples, 128);
        assert_eq!(samples.len(), 128, "sample budget remains bounded");
    }
    #[test]
    fn pointer_stroke_previews_live_and_commits_once_with_one_undo() {
        for brush in meshes::BrushKind::ALL {
            let (_tmp, mut service, mut state, ctx) = fixture();
            service.editor.sculpt.brush = brush;
            pass(&ctx, &service, &mut state, vec![], true);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let center = view.center();
            let original = service.engine.scene.clone();
            let revision = service.engine.revision;
            let (_, commands) = pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(center), button(center, true)],
                true,
            );
            assert!(commands.is_empty());
            let end = center + Vec2::new(45., 0.);
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(end)],
                true,
            );
            let draft = state.mesh.stroke.as_ref().expect("stroke is active");
            assert_ne!(
                draft.document.assets, original.meshes.assets,
                "live preview for {brush:?}"
            );
            let preview = draft.document.assets.clone();
            let samples = draft.stroke.centers.len();
            for _ in 0..5 {
                pass(&ctx, &service, &mut state, vec![], true);
            }
            assert_eq!(
                state.mesh.stroke.as_ref().unwrap().stroke.centers.len(),
                samples,
                "redraw never stamps more samples"
            );
            assert_eq!(service.engine.scene, original);
            assert_eq!(service.engine.revision, revision);
            let (_, commands) = pass(&ctx, &service, &mut state, vec![button(end, false)], true);
            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::EditMesh {
                    edit: meshes::Edit::Sculpt { .. } | meshes::Edit::SculptStroke { .. }
                }
            ));
            for command in commands {
                service.execute(command.into()).unwrap();
            }
            assert_eq!(service.engine.scene.meshes.assets, preview);
            service.execute(Command::Undo.into()).unwrap();
            assert_eq!(service.engine.scene, original);
        }
    }
    #[test]
    fn shared_radius_and_force_control_real_pointer_deformation() {
        let mut effects = vec![];
        for (radius, force) in [(0.25, 0.04), (0.25, 0.16), (0.9, 0.04)] {
            let (_tmp, mut service, mut state, ctx) = fixture();
            service
                .editor
                .apply(Action::Sculpt {
                    settings: crate::editor::SculptSettings {
                        radius,
                        draw_strength: force,
                        adaptive_detail: false,
                        ..Default::default()
                    },
                })
                .unwrap();
            pass(&ctx, &service, &mut state, vec![], true);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let center = view.center();
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(center), button(center, true)],
                true,
            );
            let draft = state.mesh.stroke.as_ref().unwrap();
            let base = &draft.base.geometry.positions;
            let changed = &draft.document.assets[0].geometry.positions;
            let displacements: Vec<f32> = base
                .iter()
                .zip(changed.iter())
                .map(|(a, b)| Vec3::from(*a).distance(Vec3::from(*b)))
                .collect();
            effects.push((
                displacements.iter().filter(|d| **d > 0.00001).count(),
                displacements.into_iter().fold(0_f32, f32::max),
            ));
        }
        assert_eq!(
            effects[0].0, effects[1].0,
            "force must not change footprint"
        );
        assert!(
            (effects[1].1 / effects[0].1 - 4.).abs() < 0.001,
            "four times the effective force must produce four times the displacement"
        );
        assert!(
            effects[2].0 > effects[0].0,
            "larger radius affects more vertices"
        );
    }
    #[test]
    fn dismissing_a_popup_over_the_mesh_never_starts_a_stroke() {
        let (_tmp, service, mut state, ctx) = fixture();
        pass(&ctx, &service, &mut state, vec![], true);
        let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
        egui::Popup::open_id(&ctx, egui::Id::new("settings-popup"));
        let center = view.center();
        let (_, commands) = pass(
            &ctx,
            &service,
            &mut state,
            vec![
                egui::Event::PointerMoved(center),
                button(center, true),
                button(center, false),
            ],
            true,
        );
        assert!(state.mesh.stroke.is_none());
        assert!(commands.is_empty(), "closing settings cannot edit the mesh");
    }
    #[test]
    fn quick_drag_press_and_release_in_one_pass_is_an_edit() {
        for brush in meshes::BrushKind::ALL {
            let (_tmp, mut service, mut state, ctx) = fixture();
            service.editor.sculpt.brush = brush;
            pass(&ctx, &service, &mut state, vec![], true);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let start = view.center();
            let end = start + Vec2::new(45., 0.);
            let original = service.engine.scene.clone();
            let (_, commands) = pass(
                &ctx,
                &service,
                &mut state,
                vec![
                    egui::Event::PointerMoved(start),
                    button(start, true),
                    egui::Event::PointerMoved(end),
                    button(end, false),
                ],
                true,
            );
            assert_eq!(commands.len(), 1, "quick {brush:?} gesture");
            service
                .execute(commands.into_iter().next().unwrap().into())
                .unwrap();
            assert_ne!(service.engine.scene, original);
            service.execute(Command::Undo.into()).unwrap();
            assert_eq!(service.engine.scene, original);
            assert!(state.mesh.stroke.is_none());
        }
    }
    #[test]
    fn release_and_save_in_one_input_pass_persists_the_completed_stroke() {
        let (tmp, mut service, mut state, ctx) = fixture();
        service.scene_path = Some("sculpt.scene.json".into());
        pass(&ctx, &service, &mut state, vec![], true);
        let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
        let start = view.center();
        let end = start + Vec2::new(45., 0.);
        let original = service.engine.scene.clone();
        let (_, commands) = pass(
            &ctx,
            &service,
            &mut state,
            vec![
                egui::Event::PointerMoved(start),
                button(start, true),
                egui::Event::PointerMoved(end),
                button(end, false),
                egui::Event::Key {
                    key: egui::Key::S,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
            ],
            true,
        );
        assert!(matches!(
            &commands[..],
            [Command::EditMesh { .. }, Command::SaveScene { .. }]
        ));
        for command in commands {
            service.execute(command.into()).unwrap();
        }
        assert_ne!(service.engine.scene, original);
        let saved: crate::model::Scene =
            serde_json::from_slice(&std::fs::read(tmp.path().join("sculpt.scene.json")).unwrap())
                .unwrap();
        assert_eq!(saved, service.engine.scene);
        assert!(!service.scene_dirty());
        service.execute(Command::Undo.into()).unwrap();
        assert_eq!(service.engine.scene, original);
    }
    #[test]
    fn save_shortcut_uses_the_open_scene_or_requests_a_destination() {
        for path in [None, Some("sculpt.scene.json".to_owned())] {
            let (_tmp, mut service, mut state, ctx) = fixture();
            service.scene_path = path.clone();
            pass(&ctx, &service, &mut state, vec![], true);
            let (_, commands) = pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::Key {
                    key: egui::Key::S,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                }],
                true,
            );
            if let Some(path) = path {
                assert!(matches!(&commands[..], [Command::SaveScene { path: p }] if p == &path));
            } else {
                assert!(commands.is_empty());
                assert_eq!(state.file_action, Some("save"));
            }
        }
    }
    #[test]
    fn escape_focus_loss_and_concurrent_revision_discard_the_draft() {
        for cancel in 0..3 {
            let (_tmp, mut service, mut state, ctx) = fixture();
            pass(&ctx, &service, &mut state, vec![], true);
            let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
            let center = view.center();
            let original = service.engine.scene.clone();
            pass(
                &ctx,
                &service,
                &mut state,
                vec![egui::Event::PointerMoved(center), button(center, true)],
                true,
            );
            assert!(state.mesh.stroke.is_some());
            if cancel == 2 {
                service
                    .execute(
                        Command::SetCamera {
                            camera: service.engine.scene.camera.clone(),
                        }
                        .into(),
                    )
                    .unwrap();
            }
            let events = if cancel == 0 {
                vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }]
            } else {
                vec![]
            };
            let (_, commands) = pass(&ctx, &service, &mut state, events, cancel != 1);
            assert!(commands.is_empty());
            assert!(state.mesh.stroke.is_none());
            assert_eq!(service.engine.scene, original);
            let (_, commands) = pass(
                &ctx,
                &service,
                &mut state,
                vec![button(center, false)],
                true,
            );
            assert!(commands.is_empty());
        }
    }
    #[test]
    fn focused_viewport_accepts_brush_shortcuts_and_text_fields_keep_their_keys() {
        let (_root, service, mut state, ctx) = fixture();
        let (view, _) = pass(&ctx, &service, &mut state, vec![], true);
        let key = || egui::Event::Key {
            key: egui::Key::Num4,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        };
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("viewport_image")));
        let (_, commands) = pass(
            &ctx,
            &service,
            &mut state,
            vec![egui::Event::PointerMoved(view.center()), key()],
            true,
        );
        assert!(commands.iter().any(|c|matches!(c,Command::EditEditor{action:Action::Sculpt{settings},..} if settings.brush==meshes::BrushKind::Flatten)));
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("test_numeric_field")));
        let (_, commands) = pass(&ctx, &service, &mut state, vec![key()], true);
        assert!(!commands.iter().any(|c| matches!(
            c,
            Command::EditEditor {
                action: Action::Sculpt { .. },
                ..
            }
        )));
    }
}
