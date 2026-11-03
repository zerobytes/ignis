//! Contextual modeling preview. Release or explicit confirmation publishes one edit.
use super::*;
use crate::{
    mesh_modeling::{Operation, Report},
    mesh_topology::{Components, Reference, Selection},
    model::Scene,
};
#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Extrude,
    Inset,
    Bevel,
}
struct Drag {
    start: Pos2,
    last: Pos2,
    amount: f32,
    axis: Vec2,
    units: f32,
    direction: [f32; 3],
    fine: bool,
}
struct Draft {
    tool: Tool,
    drag: Option<Drag>,
    instance: String,
    reference: Reference,
    selection: Selection,
    editor: u64,
    session: u64,
    view_revision: u64,
    tick: u64,
    view: Option<(Rect, u32)>,
    amount: f32,
    normal: [f32; 3],
    offset: [f32; 3],
    use_normal: bool,
    extend: bool,
    operation: Operation,
    scene: Option<Scene>,
    report: Option<Report>,
    edges: Vec<[u32; 2]>,
    valid: bool,
    visibility: crate::mesh_screen::Visibility,
}
#[derive(Default)]
pub(super) struct State {
    draft: Option<Draft>,
    pending: Option<(Reference, Scene)>,
    suppress_release: bool,
    pub error: String,
}
impl State {
    pub(super) fn active(&self) -> bool {
        self.draft.is_some()
    }
    pub(super) fn preview(&self) -> Option<&Scene> {
        self.draft
            .as_ref()
            .and_then(|d| d.scene.as_ref())
            .or_else(|| self.pending.as_ref().map(|(_, s)| s))
    }
    pub(super) fn cancel(&mut self) {
        self.suppress_release |= self.draft.as_ref().is_some_and(|d| d.drag.is_some());
        self.draft = None;
        self.pending = None;
    }
    pub(super) fn validate(&mut self, service: &Service) {
        if self
            .pending
            .as_ref()
            .is_some_and(|(r, _)| *r != service.engine.mesh_reference())
        {
            self.pending = None;
        }
        if self.draft.as_ref().is_some_and(|d| {
            service.engine.playing
                || d.reference != service.engine.mesh_reference()
                || d.editor != service.editor.revision
                || d.session != service.session_revision
                || d.view_revision != service.engine.view_revision
                || d.tick != service.engine.simulation.tick
                || service.selected_mesh.as_ref() != Some(&d.instance)
                || service.engine.selected_mesh_components() != Some(&d.selection)
        }) {
            self.cancel();
        }
    }
    fn begin(&mut self, service: &Service, tool: Tool) -> crate::model::Result<()> {
        self.cancel();
        self.error.clear();
        let instance = service
            .selected_mesh
            .clone()
            .ok_or_else(|| crate::model::invalid("Selecione uma malha editável."))?;
        let selection = service
            .engine
            .selected_mesh_components()
            .cloned()
            .ok_or_else(|| crate::model::invalid("Selecione faces ou arestas no modo Editar."))?;
        let i = service
            .engine
            .scene
            .meshes
            .instances
            .iter()
            .find(|i| i.id == instance)
            .unwrap();
        if i.asset != selection.asset {
            return Err(crate::model::invalid("A seleção pertence a outra malha."));
        }
        let asset = service.engine.scene.meshes.asset(&selection.asset)?;
        let normal = if let Components::Faces { indices } = &selection.components {
            crate::mesh_modeling::region_normal(asset, indices)?
        } else {
            [0.; 3]
        };
        let operation = match (tool, &selection.components) {
            (Tool::Extrude, Components::Faces { indices }) => Operation::Extrude {
                faces: indices.clone(),
                offset: if normal == [0.; 3] {
                    [0., 0.1, 0.]
                } else {
                    normal.map(|x| x * 0.1)
                },
            },
            (Tool::Inset, Components::Faces { indices }) => Operation::Inset {
                faces: indices.clone(),
                width: 0.05,
            },
            (Tool::Bevel, Components::Edges { vertices }) => Operation::Bevel {
                edges: vertices.clone(),
                width: 0.05,
                extend_collinear: true,
            },
            _ => {
                return Err(crate::model::invalid(
                    "Extrusão/inset precisam de faces; bevel precisa de arestas.",
                ));
            }
        };
        let offset = if let Operation::Extrude { offset, .. } = &operation {
            *offset
        } else {
            [0.; 3]
        };
        let mut draft = Draft {
            tool,
            drag: None,
            instance,
            reference: service.engine.mesh_reference(),
            selection,
            editor: service.editor.revision,
            session: service.session_revision,
            view_revision: service.engine.view_revision,
            tick: service.engine.simulation.tick,
            view: None,
            amount: if tool == Tool::Extrude { 0.1 } else { 0.05 },
            normal,
            offset,
            use_normal: normal != [0.; 3],
            extend: true,
            operation,
            scene: None,
            report: None,
            edges: vec![],
            valid: false,
            visibility: Default::default(),
        };
        evaluate(service, &mut draft, &mut self.error);
        self.draft = Some(draft);
        Ok(())
    }
    fn apply(&mut self, commands: &mut Vec<Command>) {
        if self.draft.as_ref().is_some_and(|d| d.valid) {
            let d = self.draft.take().unwrap();
            self.suppress_release |= d.drag.is_some();
            commands.push(Command::ModelEditorMesh {
                instance: d.instance,
                reference: d.reference.clone(),
                operation: d.operation,
                preview: false,
            });
            self.pending = d.scene.map(|scene| (d.reference, scene));
        }
    }
}
fn update_operation(d: &mut Draft) {
    match &mut d.operation {
        Operation::Extrude { offset, .. } => {
            *offset = if d.use_normal {
                d.normal.map(|x| x * d.amount)
            } else {
                d.offset
            }
        }
        Operation::Inset { width, .. } => *width = d.amount,
        Operation::Bevel {
            width,
            extend_collinear,
            ..
        } => {
            *width = d.amount;
            *extend_collinear = d.extend;
        }
    }
}
fn evaluate(service: &Service, d: &mut Draft, error: &mut String) {
    match crate::editor_modeling::preview(service, &d.instance, &d.reference, &d.operation) {
        Ok((scene, report)) => {
            let mut frame = service.engine.render_frame();
            service
                .engine
                .update_render_frame_with_meshes(&mut frame, &scene.meshes);
            if let Err(e) = d.visibility.prepare_frame(&frame) {
                d.valid = false;
                *error = e.message;
                return;
            }
            let asset = scene
                .meshes
                .asset(&d.selection.asset)
                .expect("modeling preserves asset");
            let mut edges = std::collections::BTreeSet::new();
            for &f in &report.selected_faces {
                let [a, b, c] = asset.geometry.triangles[f as usize];
                for (a, b) in [(a, b), (b, c), (c, a)] {
                    edges.insert(if a < b { [a, b] } else { [b, a] });
                }
            }
            d.edges = edges.into_iter().collect();
            d.scene = Some(scene);
            d.report = Some(report);
            d.valid = true;
            error.clear();
        }
        Err(e) => {
            d.valid = false;
            *error = e.message;
        }
    }
}
fn start(state: &mut State, service: &Service, tool: Tool) {
    if let Err(e) = state.begin(service, tool) {
        state.error = e.message;
    }
}
pub(super) fn actions(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &[Command],
) {
    let selection = service.engine.selected_mesh_components();
    let faces = selection.is_some_and(|s| matches!(s.components, Components::Faces { .. }));
    let edges = selection.is_some_and(|s| matches!(s.components, Components::Edges { .. }));
    ui.horizontal_wrapped(|ui| {
        for (tool, label, enabled, help) in [
            (
                Tool::Extrude,
                "Extrudar",
                faces,
                "Extrusão topológica [Ctrl+E]. Selecione faces.",
            ),
            (
                Tool::Inset,
                "Inset",
                faces,
                "Inset de região plana convexa [I]. Selecione faces.",
            ),
            (
                Tool::Bevel,
                "Bevel",
                edges,
                "Bevel plano de sólido convexo fechado [Ctrl+B]. Selecione arestas.",
            ),
        ] {
            if ui
                .add_enabled(
                    enabled && !service.engine.playing && commands.is_empty(),
                    egui::Button::new(label),
                )
                .on_hover_text(help)
                .on_disabled_hover_text(help)
                .clicked()
            {
                start(state, service, tool);
            }
        }
    });
    if !state.error.is_empty() && !state.active() {
        ui.colored_label(Color32::LIGHT_RED, &state.error);
    }
}
pub(super) fn panel(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) -> bool {
    state.validate(service);
    let Some(d) = &mut state.draft else {
        return false;
    };
    ui.heading(match d.tool {
        Tool::Extrude => "Extrusão de faces",
        Tool::Inset => "Inset de faces",
        Tool::Bevel => "Bevel de arestas",
    });
    ui.weak("Prévia · arraste na vista ou ajuste os campos");
    ui.add_space(8.);
    let mut changed = false;
    if d.tool == Tool::Extrude {
        changed |= ui
            .add_enabled(
                d.normal != [0.; 3],
                egui::Checkbox::new(&mut d.use_normal, "Seguir normal média da seleção"),
            )
            .changed();
        if d.use_normal {
            ui.label("Distância local");
            changed |= ui
                .add_sized(
                    [ui.available_width(), 26.],
                    design_ui::Scrub::new(&mut d.amount)
                        .speed(0.001)
                        .range(-100.0..=100.)
                        .suffix(" m"),
                )
                .changed();
        } else {
            ui.label("Deslocamento local (m)");
            changed |= design_ui::vector(ui, &mut d.offset, 0.001);
        }
    } else {
        ui.label("Largura local");
        changed |= ui
            .add_sized(
                [ui.available_width(), 26.],
                design_ui::Scrub::new(&mut d.amount)
                    .speed(0.001)
                    .range(0.00001..=100.)
                    .suffix(" m"),
            )
            .changed();
    }
    if d.tool == Tool::Bevel {
        changed |= ui
            .checkbox(&mut d.extend, "Incluir cadeia reta completa")
            .changed();
        ui.small("Um segmento · sólido convexo fechado. Não inclui diagonais planas.");
    }
    if changed {
        update_operation(d);
        evaluate(service, d, &mut state.error);
    }
    if let Some(report) = &d.report {
        ui.small(format!(
            "Vértices: {} para {}",
            report.source_vertices, report.vertices
        ));
        ui.small(format!(
            "Triângulos: {} para {}",
            report.source_triangles, report.triangles
        ));
        if d.tool == Tool::Bevel {
            ui.small(format!(
                "{} segmentos de aresta incluídos",
                report.effective_edges.len()
            ));
        }
    }
    let valid = d.valid;
    if !state.error.is_empty() {
        ui.colored_label(Color32::LIGHT_RED, &state.error);
        ui.small("A cena não foi alterada. Ajuste o valor ou cancele.");
    }
    ui.add_space(8.);
    ui.horizontal(|ui| {
        if ui
            .add_enabled(valid, egui::Button::new("Confirmar operação"))
            .clicked()
        {
            state.apply(commands);
        }
        if ui.button("Cancelar operação").clicked() {
            state.cancel();
        }
    });
    ui.small("A edição modifica o asset compartilhado pelas suas instâncias.");
    true
}
pub(super) fn viewport(
    ui: &egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
    response: &egui::Response,
    view: Rect,
) -> bool {
    state.validate(service);
    if state.suppress_release {
        if ui.input(|i| {
            i.pointer.button_released(egui::PointerButton::Primary) || !i.pointer.primary_down()
        }) {
            state.suppress_release = false;
        }
        return true;
    }

    if !state.active()
        && response.hovered()
        && super::viewport_keyboard(ui.ctx())
        && !service.engine.playing
        && commands.is_empty()
    {
        let tool = ui.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::E) {
                Some(Tool::Extrude)
            } else if !i.modifiers.ctrl
                && !i.modifiers.command
                && !i.modifiers.alt
                && i.key_pressed(egui::Key::I)
            {
                Some(Tool::Inset)
            } else if i.modifiers.ctrl && i.key_pressed(egui::Key::B) {
                Some(Tool::Bevel)
            } else {
                None
            }
        });
        if let Some(tool) = tool {
            start(state, service, tool);
        }
    }
    let Some(d) = &mut state.draft else {
        return false;
    };
    if ui.input(|i| !i.focused || i.key_pressed(egui::Key::Escape)) || !commands.is_empty() {
        state.cancel();
        return true;
    }
    let key = (view, ui.ctx().pixels_per_point().to_bits());
    if d.view.is_some_and(|old| old != key) {
        state.cancel();
        return true;
    }
    d.view = Some(key);
    if d.drag.is_none()
        && super::viewport_keyboard(ui.ctx())
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
    {
        state.apply(commands);
        return true;
    }
    let camera = service
        .editor
        .current()
        .camera
        .as_ref()
        .unwrap_or(&service.engine.scene.camera);
    let ppp = ui.ctx().pixels_per_point();
    let size = [
        (view.width() * ppp).round().clamp(1., 4096.) as u32,
        (view.height() * ppp).round().clamp(1., 4096.) as u32,
    ];
    let Ok(projection) = crate::mesh_screen::Projection::new(camera, size[0], size[1]) else {
        return true;
    };

    let press = ui.input(|i| {
        i.events.iter().find_map(|e| match e {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                ..
            } if view.contains(*pos) => Some(*pos),
            _ => None,
        })
    });
    if d.drag.is_none()
        && response.hovered()
        && let Some(start) = press
    {
        let instance = service
            .engine
            .scene
            .meshes
            .instances
            .iter()
            .find(|i| i.id == d.instance)
            .unwrap();
        let asset = service
            .engine
            .scene
            .meshes
            .asset(&d.selection.asset)
            .unwrap();
        let vertices =
            crate::editor_transform::vertices(&asset.geometry, &d.selection.components).unwrap();
        let center = vertices
            .iter()
            .map(|i| glam::Vec3::from(asset.geometry.positions[*i as usize]))
            .sum::<glam::Vec3>()
            / vertices.len() as f32;
        let q = glam::Quat::from_array(instance.rotation);
        let world = glam::Vec3::from(instance.position) + q * (center * instance.scale);
        let direction = if d.tool == Tool::Extrude {
            if d.use_normal {
                d.normal
            } else {
                glam::Vec3::from(d.offset)
                    .try_normalize()
                    .unwrap_or(glam::Vec3::Y)
                    .to_array()
            }
        } else {
            [1., 0., 0.]
        };
        let forward =
            (glam::Vec3::from(camera.target) - glam::Vec3::from(camera.position)).normalize();
        let depth = (world - glam::Vec3::from(camera.position))
            .dot(forward)
            .max(crate::picking::NEAR);
        let units = 2. * depth * (camera.fov_degrees.to_radians() * 0.5).tan()
            / view.height()
            / instance.scale;
        let axis = if d.tool == Tool::Extrude {
            let normal = q * glam::Vec3::from(direction);
            let project = |p| {
                projection.project(p).map(|p| {
                    Vec2::new(
                        p.x / size[0] as f32 * view.width(),
                        p.y / size[1] as f32 * view.height(),
                    )
                })
            };
            if let (Some(a), Some(b)) = (
                project(world),
                project(world + normal * (units * instance.scale * 60.)),
            ) {
                let v = b - a;
                if v.length() > 12. {
                    v.normalized()
                } else {
                    Vec2::new(0., -1.)
                }
            } else {
                Vec2::new(0., -1.)
            }
        } else {
            Vec2::X
        };
        d.drag = Some(Drag {
            start,
            last: start,
            amount: if d.tool == Tool::Extrude && !d.use_normal {
                glam::Vec3::from(d.offset).length()
            } else {
                d.amount
            },
            axis,
            units,
            direction,
            fine: ui.input(|i| i.modifiers.shift),
        });
    }
    if let Some(drag) = &mut d.drag
        && let Some(pos) = ui.input(|i| i.pointer.latest_pos())
    {
        let fine = ui.input(|i| i.modifiers.shift);
        if pos != drag.last || fine != drag.fine {
            drag.last = pos;
            drag.fine = fine;
            let value = drag.amount
                + (pos - drag.start).dot(drag.axis) * drag.units * if fine { 0.1 } else { 1. };
            d.amount = if d.tool == Tool::Extrude {
                value
            } else {
                value.max(0.00001)
            };
            if !d.use_normal && d.tool == Tool::Extrude {
                d.offset = drag.direction.map(|x| x * value);
            }
            update_operation(d);
            evaluate(service, d, &mut state.error);
        }
        if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
            if d.valid {
                state.apply(commands);
            } else {
                state.cancel();
            }
            state.suppress_release = false;
            return true;
        }
    }
    let Some(scene) = &d.scene else {
        return true;
    };
    let a = scene.meshes.asset(&d.selection.asset).unwrap();
    let i = scene
        .meshes
        .instances
        .iter()
        .find(|i| i.id == d.instance)
        .unwrap();
    let world = |index: u32| {
        glam::Vec3::from(i.position)
            + glam::Quat::from_array(i.rotation)
                * (glam::Vec3::from(a.geometry.positions[index as usize]) * i.scale)
    };
    let painter = ui.painter().with_clip_rect(view);
    for &[a, b] in &d.edges {
        super::mesh_ui::draw_visible_edge(
            &painter,
            view,
            &projection,
            &d.visibility,
            world(a),
            world(b),
            service.mesh_view.xray,
            size,
            EMBER,
        );
    }
    painter.text(
        view.left_bottom() + Vec2::new(12., -12.),
        egui::Align2::LEFT_BOTTOM,
        "PRÉVIA · arraste e solte · Shift fino · Enter confirmar · Esc cancelar",
        egui::FontId::proportional(12.),
        EMBER,
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(
        tool: Tool,
    ) -> (
        tempfile::TempDir,
        Service,
        super::super::UiState,
        egui::Context,
    ) {
        let root = tempfile::tempdir().unwrap();
        let mut s = Service::new(
            root.path(),
            crate::model::RoomRecipe {
                obstacles: 0,
                ..Default::default()
            }
            .generate()
            .unwrap(),
        )
        .unwrap();
        s.execute(
            Command::CreateEditorMesh {
                primitive: crate::mesh_tools::Primitive::Cube,
                resolution: 4,
                size: 2.,
            }
            .into(),
        )
        .unwrap();
        let i = s.engine.scene.meshes.instances[0].clone();
        let a = s.engine.scene.meshes.asset(&i.asset).unwrap();
        let components = if tool == Tool::Bevel {
            let edge = crate::mesh_topology::Topology::inspect(&a.geometry)
                .unwrap()
                .edges
                .into_iter()
                .find(|e| {
                    e.vertices.iter().all(|i| {
                        let p = a.geometry.positions[*i as usize];
                        p[0] > 0.999 && p[1] > 0.999
                    })
                })
                .unwrap()
                .vertices;
            Components::Edges {
                vertices: vec![edge],
            }
        } else {
            Components::Faces {
                indices: a
                    .geometry
                    .triangles
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| {
                        f.iter()
                            .all(|i| a.geometry.positions[*i as usize][2] > 0.999)
                    })
                    .map(|(i, _)| i as u32)
                    .collect(),
            }
        };
        s.execute(
            Command::SelectMeshComponents {
                selection: Some(Selection {
                    asset: i.asset,
                    reference: s.engine.mesh_reference(),
                    components,
                }),
            }
            .into(),
        )
        .unwrap();
        let mut camera = s.engine.scene.camera.clone();
        camera.position = [5., 4., 7.];
        camera.target = [2., 1.05, 0.];
        s.execute(
            Command::EditEditor {
                action: Action::Camera {
                    camera: Some(camera),
                },
                expected_editor_revision: None,
            }
            .into(),
        )
        .unwrap();
        let mut state = super::super::UiState::default();
        state.project.dismissed = true;
        state.mesh.components = true;
        let ctx = egui::Context::default();
        design_ui::configure(&ctx);
        (root, s, state, ctx)
    }
    fn pass(
        s: &Service,
        state: &mut super::super::UiState,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        focused: bool,
    ) -> (Rect, Vec<Command>, Vec<(String, Rect)>) {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
            events,
            focused,
            ..Default::default()
        });
        let mut commands = vec![];
        let mut view = None;
        super::super::draw_editor(
            ctx,
            s,
            state,
            &mut commands,
            egui::TextureId::User(0),
            &mut view,
        );
        let output = ctx.end_pass();
        fn collect(shape: &egui::Shape, out: &mut Vec<(String, Rect)>) {
            match shape {
                egui::Shape::Text(t) => out.push((
                    t.galley.text().into(),
                    Rect::from_min_size(t.pos, t.galley.size()),
                )),
                egui::Shape::Vec(shapes) => {
                    for s in shapes {
                        collect(s, out)
                    }
                }
                _ => {}
            }
        }
        let mut labels = vec![];
        for s in output.shapes {
            collect(&s.shape, &mut labels);
        }
        (view.unwrap(), commands, labels)
    }
    fn button(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: Default::default(),
        }
    }
    fn label(labels: &[(String, Rect)], name: &str) -> Pos2 {
        labels
            .iter()
            .find(|(s, _)| s == name)
            .unwrap_or_else(|| {
                panic!(
                    "missing {name}; {:?}",
                    labels.iter().map(|(s, _)| s).collect::<Vec<_>>()
                )
            })
            .1
            .center()
    }
    fn click(
        s: &Service,
        state: &mut super::super::UiState,
        ctx: &egui::Context,
        pos: Pos2,
    ) -> Vec<Command> {
        pass(
            s,
            state,
            ctx,
            vec![egui::Event::PointerMoved(pos), button(pos, true)],
            true,
        );
        pass(s, state, ctx, vec![button(pos, false)], true).1
    }
    #[test]
    fn three_modeling_buttons_preview_without_mutation_and_confirm_one_shared_undo() {
        for tool in [Tool::Extrude, Tool::Inset, Tool::Bevel] {
            let (_root, mut s, mut state, ctx) = fixture(tool);
            let before = s.engine.snapshot();
            let (_, _, labels) = pass(&s, &mut state, &ctx, vec![], true);
            let at = label(
                &labels,
                match tool {
                    Tool::Extrude => "Extrudar",
                    Tool::Inset => "Inset",
                    Tool::Bevel => "Bevel",
                },
            );
            assert!(click(&s, &mut state, &ctx, at).is_empty());
            assert!(state.mesh.modeling.active());
            assert!(
                state.mesh.modeling.draft.as_ref().unwrap().valid,
                "{}",
                state.mesh.modeling.error
            );
            let preview = state.mesh.modeling.preview().unwrap().clone();
            assert_ne!(preview, s.engine.scene);
            assert_eq!(s.engine.snapshot(), before);
            let pointer = std::sync::Arc::as_ptr(&preview.meshes.assets[0].geometry);
            let mut confirm = None;
            for _ in 0..4 {
                let (_, commands, labels) = pass(&s, &mut state, &ctx, vec![], true);
                assert!(commands.is_empty());
                assert_eq!(
                    std::sync::Arc::as_ptr(
                        &state.mesh.modeling.preview().unwrap().meshes.assets[0].geometry
                    ),
                    pointer
                );
                confirm = Some(label(&labels, "Confirmar operação"));
            }
            let commands = click(&s, &mut state, &ctx, confirm.unwrap());
            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ModelEditorMesh { preview: false, .. }
            ));
            assert!(
                state.mesh.modeling.preview().is_some(),
                "preview survives release until dispatch"
            );
            for c in commands {
                s.execute(c.into()).unwrap();
            }
            assert_eq!(s.engine.scene, preview);
            assert_eq!(
                s.engine.snapshot()["history"]["undo"].as_u64(),
                Some(before["history"]["undo"].as_u64().unwrap() + 1)
            );
            assert!(matches!(
                s.engine.selected_mesh_components().unwrap().components,
                Components::Faces { .. }
            ));
            pass(&s, &mut state, &ctx, vec![], true);
            assert!(state.mesh.modeling.preview().is_none());
            s.execute(Command::Undo.into()).unwrap();
            assert_eq!(s.engine.snapshot()["scene"], before["scene"]);
        }
    }
    #[test]
    fn numeric_width_updates_cached_preview_and_escape_focus_or_revision_never_publish_it() {
        for reason in 0..3 {
            let (_root, mut s, mut state, ctx) = fixture(Tool::Inset);
            let (_, _, labels) = pass(&s, &mut state, &ctx, vec![], true);
            let start = label(&labels, "Inset");
            click(&s, &mut state, &ctx, start);
            let (_, _, labels) = pass(&s, &mut state, &ctx, vec![], true);
            let field = labels
                .iter()
                .find(|(s, _)| s.contains("0.050") && s.contains("m"))
                .unwrap()
                .1
                .center();
            let before = s.engine.snapshot();
            let original = state.mesh.modeling.preview().unwrap().clone();
            pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(field), button(field, true)],
                true,
            );
            pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(field + Vec2::new(14., 0.))],
                true,
            );
            pass(
                &s,
                &mut state,
                &ctx,
                vec![button(field + Vec2::new(14., 0.), false)],
                true,
            );
            assert_ne!(state.mesh.modeling.preview().unwrap(), &original);
            assert_eq!(s.engine.snapshot(), before);
            let events = if reason == 0 {
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
            if reason == 2 {
                s.execute(Command::Reset.into()).unwrap();
            }
            let before = s.engine.snapshot();
            let (_, commands, _) = pass(&s, &mut state, &ctx, events, reason != 1);
            assert!(commands.is_empty());
            assert!(!state.mesh.modeling.active());
            assert_eq!(s.engine.snapshot(), before);
        }
    }
    #[test]
    fn viewport_drag_is_pure_until_release_and_focus_loss_consumes_the_release() {
        for tool in [Tool::Extrude, Tool::Inset, Tool::Bevel] {
            let (_root, mut s, mut state, ctx) = fixture(tool);
            let (view, _, _) = pass(&s, &mut state, &ctx, vec![], true);
            state.mesh.modeling.begin(&s, tool).unwrap();
            pass(&s, &mut state, &ctx, vec![], true);
            let before = s.engine.snapshot();
            let original = state.mesh.modeling.preview().unwrap().clone();
            let start = view.center();
            pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(start), button(start, true)],
                true,
            );
            let axis = state
                .mesh
                .modeling
                .draft
                .as_ref()
                .unwrap()
                .drag
                .as_ref()
                .unwrap()
                .axis;
            let end = start + axis * 18.;
            let (_, commands, _) = pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(end)],
                true,
            );
            assert!(commands.is_empty());
            assert_ne!(state.mesh.modeling.preview().unwrap(), &original);
            assert_eq!(s.engine.snapshot(), before);
            let preview = state.mesh.modeling.preview().unwrap().clone();
            let (_, commands, _) = pass(&s, &mut state, &ctx, vec![button(end, false)], true);
            assert_eq!(commands.len(), 1);
            for command in commands {
                s.execute(command.into()).unwrap();
            }
            assert_eq!(s.engine.scene, preview);
            s.execute(Command::Undo.into()).unwrap();
            assert_eq!(s.engine.snapshot()["scene"], before["scene"]);
        }
        let (_root, s, mut state, ctx) = fixture(Tool::Inset);
        let (view, _, _) = pass(&s, &mut state, &ctx, vec![], true);
        state.mesh.modeling.begin(&s, Tool::Inset).unwrap();
        let start = view.center();
        pass(
            &s,
            &mut state,
            &ctx,
            vec![egui::Event::PointerMoved(start), button(start, true)],
            true,
        );
        let before = s.engine.snapshot();
        let (_, commands, _) = pass(&s, &mut state, &ctx, vec![], false);
        assert!(commands.is_empty());
        assert!(!state.mesh.modeling.active());
        let (_, commands, _) = pass(&s, &mut state, &ctx, vec![button(start, false)], true);
        assert!(
            commands.is_empty(),
            "canceled release must not deselect components"
        );
        assert_eq!(s.engine.snapshot(), before);
    }
}
