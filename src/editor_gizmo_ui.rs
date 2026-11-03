//! Direct manipulation, with a validated authoring preview and one commit per gesture.
use super::*;
use crate::{
    editor_transform::{self as transform, Pivot, Selection, Settings, Space, Target, Tool},
    meshes::IslandTransform,
    model::{Camera, Scene},
};
use glam::{Quat, Vec3};

const COLORS: [Color32; 3] = [
    Color32::from_rgb(235, 101, 101),
    Color32::from_rgb(119, 199, 140),
    Color32::from_rgb(107, 163, 241),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Handle {
    Axis(usize),
    Plane(usize),
    Ring(usize),
    Screen,
    Uniform,
}
/// Logical UI rectangle plus the exact physical texture size used by Graphics::paint.
#[derive(Clone, Copy, PartialEq)]
struct View {
    rect: Rect,
    pixels: [u32; 2],
}
impl View {
    fn new(rect: Rect, ppp: f32) -> Self {
        Self {
            rect,
            pixels: [
                (rect.width() * ppp).round().clamp(1., 4096.) as u32,
                (rect.height() * ppp).round().clamp(1., 4096.) as u32,
            ],
        }
    }
}
impl std::ops::Deref for View {
    type Target = Rect;
    fn deref(&self) -> &Rect {
        &self.rect
    }
}
struct Draft {
    selection: Selection,
    settings: Settings,
    camera: Camera,
    view: View,
    session: u64,
    editor: u64,
    tick: u64,
    handle: Handle,
    pivot: Vec3,
    basis: [Vec3; 3],
    start: Pos2,
    last: Pos2,
    last_modifiers: egui::Modifiers,
    start_point: Vec3,
    vertices: Vec<u32>,
    transform: IslandTransform,
    preview: Option<Scene>,
    valid: bool,
}
struct SelectionCache {
    reference: crate::mesh_topology::Reference,
    view: u64,
    session: u64,
    components: bool,
    value: std::result::Result<std::sync::Arc<Selection>, String>,
}
#[derive(Default)]
pub(super) struct State {
    selection_cache: Option<SelectionCache>,
    draft: Option<Draft>,
    pending: Option<Draft>,
    pub error: String,
}
impl State {
    fn selection(
        &mut self,
        service: &Service,
        components: bool,
    ) -> std::result::Result<std::sync::Arc<Selection>, String> {
        let reference = service.engine.mesh_reference();
        if !self.selection_cache.as_ref().is_some_and(|c| {
            c.reference == reference
                && c.view == service.engine.view_revision
                && c.session == service.session_revision
                && c.components == components
        }) {
            self.selection_cache = Some(SelectionCache {
                reference,
                view: service.engine.view_revision,
                session: service.session_revision,
                components,
                value: transform::selection(service, components)
                    .map(std::sync::Arc::new)
                    .map_err(|e| e.message),
            });
        }
        self.selection_cache.as_ref().unwrap().value.clone()
    }
    pub(super) fn cancel(&mut self) {
        self.draft = None;
        self.pending = None;
    }
    pub(super) fn preview(&self) -> Option<&Scene> {
        self.draft
            .as_ref()
            .or(self.pending.as_ref())
            .and_then(|d| d.preview.as_ref())
    }
    /// RenderFrame deliberately has no IDs; resolve the existing filtered entity order.
    pub(super) fn preview_box(&self, service: &Service, frame: &mut RenderFrame) {
        let Some(d) = self.draft.as_ref().or(self.pending.as_ref()) else {
            return;
        };
        let (Some(scene), Target::Entity { id }) = (&d.preview, &d.selection.target) else {
            return;
        };
        let index = service
            .engine
            .simulation
            .definitions()
            .entities
            .iter()
            .filter(|e| {
                e.visible != Some(false) && service.engine.simulation.position(&e.id).is_some()
            })
            .position(|e| &e.id == id);
        if let Some(index) = index
            && let Some(b) = frame.boxes.get_mut(index)
        {
            let e = scene
                .entities
                .iter()
                .find(|e| &e.id == id)
                .expect("preview preserves entity");
            b.position = e.transform.position;
            b.size = e.transform.size;
            frame.selection = Some(*b);
        }
    }
}
fn pivot(s: &Selection, settings: &Settings) -> Vec3 {
    match settings.pivot {
        Pivot::Center => s.center.into(),
        Pivot::Origin => s.origin.into(),
        Pivot::Custom => settings.custom_pivot.into(),
    }
}
fn basis(s: &Selection, settings: &Settings) -> [Vec3; 3] {
    let q = if settings.space == Space::Local {
        Quat::from_array(s.orientation)
    } else {
        Quat::IDENTITY
    };
    [q * Vec3::X, q * Vec3::Y, q * Vec3::Z]
}
fn project(camera: &Camera, view: View, p: Vec3) -> Option<Pos2> {
    let projection =
        crate::mesh_screen::Projection::new(camera, view.pixels[0], view.pixels[1]).ok()?;
    projection.project(p).map(|p| {
        view.min
            + Vec2::new(
                p.x / view.pixels[0] as f32 * view.width(),
                p.y / view.pixels[1] as f32 * view.height(),
            )
    })
}
fn ray(camera: &Camera, view: View, p: Pos2) -> (Vec3, Vec3) {
    let forward = (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize();
    let right = forward.cross(Vec3::Y).normalize();
    let up = right.cross(forward);
    let h = (camera.fov_degrees.to_radians() * 0.5).tan();
    let x = (2. * (p.x - view.left()) / view.width() - 1.) * h * view.pixels[0] as f32
        / view.pixels[1] as f32;
    let y = (1. - 2. * (p.y - view.top()) / view.height()) * h;
    (
        camera.position.into(),
        (forward + right * x + up * y).normalize(),
    )
}
fn plane(camera: &Camera, view: View, pos: Pos2, pivot: Vec3, normal: Vec3) -> Option<Vec3> {
    let (eye, direction) = ray(camera, view, pos);
    let den = direction.dot(normal);
    if den.abs() < 0.0001 {
        return None;
    }
    let distance = (pivot - eye).dot(normal) / den;
    (distance > 0. && distance.is_finite()).then(|| eye + direction * distance)
}
fn normal(d: &Draft) -> Vec3 {
    match d.handle {
        Handle::Plane(i) | Handle::Ring(i) => d.basis[i],
        Handle::Axis(i) => {
            let view = (Vec3::from(d.camera.position) - d.pivot).normalize();
            (view - d.basis[i] * view.dot(d.basis[i])).normalize_or_zero()
        }
        _ => (Vec3::from(d.camera.position) - d.pivot).normalize(),
    }
}
fn snap(value: f32, step: f32, enabled: bool) -> f32 {
    if enabled {
        (value / step).round() * step
    } else {
        value
    }
}
fn delta(d: &Draft, pos: Pos2, modifiers: egui::Modifiers) -> Option<IslandTransform> {
    let mut out = IslandTransform {
        pivot: d.pivot.to_array(),
        translation: [0.; 3],
        rotation: Quat::IDENTITY.to_array(),
        scale: 1.,
    };
    let fine = if modifiers.shift { 0.1 } else { 1. };
    let snapping = d.settings.snap || modifiers.ctrl || modifiers.command;
    match d.handle {
        Handle::Uniform => {
            let value = ((pos.x - d.start.x - (pos.y - d.start.y)) * 0.005 * fine).exp() - 1.;
            out.scale = (1. + snap(value, d.settings.scale_step, snapping)).clamp(0.001, 100.);
        }
        handle => {
            let p = plane(&d.camera, d.view, pos, d.pivot, normal(d))?;
            let v = (p - d.start_point) * fine;
            let displacement = match handle {
                Handle::Axis(i) => {
                    d.basis[i] * snap(v.dot(d.basis[i]), d.settings.move_step, snapping)
                }
                Handle::Plane(i) => {
                    let a = (i + 1) % 3;
                    let b = (i + 2) % 3;
                    d.basis[a] * snap(v.dot(d.basis[a]), d.settings.move_step, snapping)
                        + d.basis[b] * snap(v.dot(d.basis[b]), d.settings.move_step, snapping)
                }
                Handle::Screen => {
                    // The free handle snaps world coordinates; constrained handles snap their basis.
                    Vec3::new(
                        snap(v.x, d.settings.move_step, snapping),
                        snap(v.y, d.settings.move_step, snapping),
                        snap(v.z, d.settings.move_step, snapping),
                    )
                }
                Handle::Ring(i) => {
                    let a = (d.start_point - d.pivot).normalize_or_zero();
                    let b = (p - d.pivot).normalize_or_zero();
                    if a == Vec3::ZERO || b == Vec3::ZERO {
                        return None;
                    }
                    let angle = a.cross(b).dot(d.basis[i]).atan2(a.dot(b)) * fine;
                    out.rotation = Quat::from_axis_angle(
                        d.basis[i],
                        snap(angle, d.settings.angle_step.to_radians(), snapping),
                    )
                    .to_array();
                    Vec3::ZERO
                }
                _ => unreachable!(),
            };
            out.translation = displacement.to_array();
        }
    }
    Some(out)
}
fn pressed(ui: &egui::Ui) -> Option<Pos2> {
    ui.input(|i| {
        i.events.iter().find_map(|event| match event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                ..
            } => Some(*pos),
            _ => None,
        })
    })
}
fn distance_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let v = b - a;
    let t = if v.length_sq() > 0. {
        (p - a).dot(v) / v.length_sq()
    } else {
        0.
    };
    p.distance(a + v * t.clamp(0., 1.))
}
fn length(camera: &Camera, view: View, pivot: Vec3) -> f32 {
    let forward = (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize();
    let depth = (pivot - Vec3::from(camera.position))
        .dot(forward)
        .max(crate::picking::NEAR);
    2. * depth * (camera.fov_degrees.to_radians() * 0.5).tan() * 78. / view.height()
}
fn ring_points(
    camera: &Camera,
    view: View,
    pivot: Vec3,
    basis: [Vec3; 3],
    i: usize,
    len: f32,
) -> Vec<Pos2> {
    (0..=64)
        .filter_map(|n| {
            let a = n as f32 / 64. * std::f32::consts::TAU;
            project(
                camera,
                view,
                pivot + (basis[(i + 1) % 3] * a.cos() + basis[(i + 2) % 3] * a.sin()) * len,
            )
        })
        .collect()
}
#[allow(clippy::too_many_arguments)]
fn draw(
    ui: &egui::Ui,
    camera: &Camera,
    view: View,
    pivot: Vec3,
    basis: [Vec3; 3],
    tool: Tool,
    active: Option<Handle>,
    can_rotate: bool,
) -> Option<Handle> {
    let center = project(camera, view, pivot)?;
    if !view.contains(center) {
        return None;
    }
    let p = ui.painter().with_clip_rect(view.rect);
    let len = length(camera, view, pivot);
    let pointer = pressed(ui)
        .or_else(|| ui.input(|i| i.pointer.hover_pos()))
        .filter(|q| view.contains(*q));
    let mut hit = None;
    if tool == Tool::Move {
        for i in 0..3 {
            let a = basis[(i + 1) % 3];
            let b = basis[(i + 2) % 3];
            let points = [
                a * 0.23 + b * 0.23,
                a * 0.43 + b * 0.23,
                a * 0.43 + b * 0.43,
                a * 0.23 + b * 0.43,
            ]
            .into_iter()
            .filter_map(|v| project(camera, view, pivot + v * len))
            .collect::<Vec<_>>();
            if points.len() != 4 {
                continue;
            }
            // Convex screen polygon, including edge-on rejection.
            let area = (points[1] - points[0])
                .rot90()
                .dot(points[3] - points[0])
                .abs();
            let hovered = area > 8.
                && pointer.is_some_and(|q| {
                    let signs = (0..4)
                        .map(|n| (points[(n + 1) % 4] - points[n]).rot90().dot(q - points[n]))
                        .collect::<Vec<_>>();
                    signs.iter().all(|s| *s >= 0.) || signs.iter().all(|s| *s <= 0.)
                });
            let color = if active == Some(Handle::Plane(i)) || hovered {
                EMBER
            } else {
                COLORS[i]
            };
            p.add(egui::Shape::convex_polygon(
                points,
                color.linear_multiply(0.2),
                egui::Stroke::new(1.0_f32, color),
            ));
            if hovered {
                hit = Some(Handle::Plane(i));
            }
        }
        for i in 0..3 {
            let Some(end) = project(camera, view, pivot + basis[i] * len) else {
                continue;
            };
            if end.distance(center) < 18. {
                continue;
            }
            let start = center + (end - center).normalized() * 13.;
            let hovered = pointer.is_some_and(|q| distance_segment(q, start, end) < 7.);
            let color = if active == Some(Handle::Axis(i)) || hovered {
                EMBER
            } else {
                COLORS[i]
            };
            p.arrow(start, end - start, egui::Stroke::new(2.0_f32, color));
            p.text(
                end + (end - center).normalized() * 11.,
                egui::Align2::CENTER_CENTER,
                ["X", "Y", "Z"][i],
                egui::FontId::proportional(12.),
                color,
            );
            if hovered {
                hit = Some(Handle::Axis(i));
            }
        }
        p.circle(
            center,
            7.,
            PANEL,
            egui::Stroke::new(
                1.8_f32,
                if active == Some(Handle::Screen) {
                    EMBER
                } else {
                    Color32::WHITE
                },
            ),
        );
        if pointer.is_some_and(|q| q.distance(center) < 9.) {
            hit = Some(Handle::Screen);
        }
    } else if tool == Tool::Rotate && can_rotate {
        // Draw back-to-front ordering is cosmetic; hit-test nearest eligible ring.
        let mut best = 8.;
        for i in 0..3 {
            let points = ring_points(camera, view, pivot, basis, i, len);
            if points.len() != 65 {
                continue;
            }
            let facing = (Vec3::from(camera.position) - pivot)
                .normalize()
                .dot(basis[i])
                .abs();
            let distance = pointer
                .map(|q| {
                    points
                        .windows(2)
                        .map(|w| distance_segment(q, w[0], w[1]))
                        .fold(f32::INFINITY, f32::min)
                })
                .unwrap_or(f32::INFINITY);
            let hovered = facing > 0.12 && distance < best;
            if hovered {
                hit = Some(Handle::Ring(i));
                best = distance;
            }
            let color = if active == Some(Handle::Ring(i)) || hovered {
                EMBER
            } else {
                COLORS[i]
            };
            p.add(egui::Shape::line(
                points,
                egui::Stroke::new(if facing > 0.12 { 2.0_f32 } else { 0.7_f32 }, color),
            ));
        }
        p.circle_filled(center, 3., Color32::WHITE);
    } else if tool == Tool::Scale {
        let end = center + Vec2::new(52., -52.);
        p.line_segment([center, end], egui::Stroke::new(2.0_f32, EMBER));
        for point in [center, end] {
            p.rect(
                Rect::from_center_size(point, Vec2::splat(12.)),
                2.,
                PANEL,
                egui::Stroke::new(1.8_f32, EMBER),
                egui::StrokeKind::Inside,
            );
        }
        p.text(
            center + Vec2::new(0., 20.),
            egui::Align2::CENTER_CENTER,
            "Uniforme",
            egui::FontId::proportional(11.),
            Color32::WHITE,
        );
        if pointer.is_some_and(|q| {
            q.distance(center) < 11.
                || q.distance(end) < 11.
                || distance_segment(q, center, end) < 7.
        }) {
            hit = Some(Handle::Uniform);
        }
    }
    hit
}
fn tool_button(ui: &mut egui::Ui, tool: Tool, current: Tool) -> egui::Response {
    let response = ui.add(
        egui::Button::new("")
            .min_size(Vec2::splat(34.))
            .selected(tool == current),
    );
    let c = response.rect.center();
    let p = ui.painter();
    let stroke = egui::Stroke::new(
        1.6_f32,
        if tool == current {
            EMBER
        } else {
            design_ui::TEXT
        },
    );
    match tool {
        Tool::Select => {
            p.add(egui::Shape::closed_line(
                vec![
                    c + Vec2::new(-6., -9.),
                    c + Vec2::new(-5., 8.),
                    c + Vec2::new(0., 3.),
                    c + Vec2::new(7., 3.),
                    c + Vec2::new(-6., -9.),
                ],
                stroke,
            ));
        }
        Tool::Move => {
            for a in [Vec2::X, Vec2::Y] {
                p.arrow(c - a * 8., a * 16., stroke);
                p.arrow(c + a * 8., -a * 16., stroke);
            }
        }
        Tool::Rotate => {
            p.circle_stroke(c, 8., stroke);
            p.arrow(c + Vec2::new(7., -4.), Vec2::new(-1., -6.), stroke);
        }
        Tool::Scale => {
            p.line_segment([c - Vec2::splat(6.), c + Vec2::splat(6.)], stroke);
            for point in [c - Vec2::splat(6.), c + Vec2::splat(6.)] {
                p.rect_stroke(
                    Rect::from_center_size(point, Vec2::splat(6.)),
                    0.,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
    let label = match tool {
        Tool::Select => "Selecionar [Q]",
        Tool::Move => "Mover [W]",
        Tool::Rotate => "Girar [E]",
        Tool::Scale => "Escala uniforme [R]",
    };
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            true,
            tool == current,
            label,
        )
    });
    response.on_hover_text(label)
}
pub(super) fn tools(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    components: bool,
    commands: &mut Vec<Command>,
) {
    let mut settings = service.transform.clone();
    let selection = state.selection(service, components);
    for tool in [Tool::Select, Tool::Move, Tool::Rotate, Tool::Scale] {
        let supported = tool == Tool::Select
            || selection
                .as_ref()
                .is_ok_and(|s| tool != Tool::Rotate || s.can_rotate);
        ui.add_enabled_ui(!service.engine.playing&&supported,|ui|{
            if tool_button(ui,tool,settings.tool).clicked() {settings.tool=tool;state.cancel();}
        }).response.on_disabled_hover_text(if tool==Tool::Rotate {"Rotação livre disponível para malhas e seus componentes. Caixas físicas permanecem alinhadas aos eixos."}else{"Selecione um objeto ou componentes e pause a simulação."});
    }
    ui.separator();
    ui.menu_button("···",|ui|{
        ui.set_min_width(236.);
        ui.strong("Transformação");
        ui.label("Orientação");
        ui.horizontal(|ui|{ui.selectable_value(&mut settings.space,Space::World,"Global");ui.selectable_value(&mut settings.space,Space::Local,"Local");});
        ui.label("Pivô");
        ui.horizontal_wrapped(|ui|{ui.selectable_value(&mut settings.pivot,Pivot::Center,"Centro");ui.selectable_value(&mut settings.pivot,Pivot::Origin,"Origem");ui.selectable_value(&mut settings.pivot,Pivot::Custom,"Personalizado");});
        if settings.pivot==Pivot::Custom {design_ui::vector(ui,&mut settings.custom_pivot,0.01);}
        ui.separator();ui.checkbox(&mut settings.snap,"Ajustar a incrementos");
        ui.horizontal(|ui|{ui.label("Posição");ui.add(design_ui::Scrub::new(&mut settings.move_step).speed(0.01).range(0.0001..=1000.).suffix(" m"));});
        ui.horizontal(|ui|{ui.label("Ângulo");ui.add(design_ui::Scrub::new(&mut settings.angle_step).speed(1.).range(0.1..=180.).suffix("°"));});
        ui.horizontal(|ui|{ui.label("Escala");ui.add(design_ui::Scrub::new(&mut settings.scale_step).speed(0.01).range(0.001..=10.));});
        ui.small("Ctrl/Cmd: snap · Shift: ajuste fino\nSoltar: confirmar · Esc: cancelar\nEscala uniforme preserva o contrato atual da malha.");
    });
    if settings != service.transform {
        state.cancel();
        commands.push(Command::SetEditorTransform { settings });
    }
}
/// Returns true when the gesture owns viewport input, including its release/cancel pass.
pub(super) fn viewport(
    ui: &egui::Ui,
    service: &Service,
    state: &mut State,
    components: bool,
    commands: &mut Vec<Command>,
    response: &egui::Response,
    view: Rect,
) -> bool {
    let view = View::new(view, ui.ctx().pixels_per_point());
    if state
        .pending
        .as_ref()
        .is_some_and(|d| d.selection.reference != service.engine.mesh_reference())
    {
        state.pending = None;
    }
    let camera = service
        .editor
        .current()
        .camera
        .as_ref()
        .unwrap_or(&service.engine.scene.camera);
    let selection = state.selection(service, components);
    if let Some(d) = &state.draft {
        let cancelled = ui.input(|i| {
            !i.focused
                || i.key_pressed(egui::Key::Escape)
                || i.pointer.button_pressed(egui::PointerButton::Secondary)
        }) || service.engine.playing
            || service.engine.mesh_reference() != d.selection.reference
            || service.session_revision != d.session
            || service.editor.revision != d.editor
            || service.engine.simulation.tick != d.tick
            || *camera != d.camera
            || view != d.view
            || selection
                .as_ref()
                .map_or(true, |s| s.target != d.selection.target)
            || !commands.is_empty();
        if cancelled {
            state.cancel();
            return true;
        }
    }
    if state.draft.is_none()
        && !service.engine.playing
        && response.hovered()
        && super::viewport_keyboard(ui.ctx())
        && !ui.input(|i| i.modifiers.ctrl || i.modifiers.command || i.modifiers.alt)
    {
        let mut settings = service.transform.clone();
        for (key, tool) in [
            (egui::Key::Q, Tool::Select),
            (egui::Key::W, Tool::Move),
            (egui::Key::E, Tool::Rotate),
            (egui::Key::R, Tool::Scale),
        ] {
            if ui.input(|i| i.key_pressed(key))
                && selection
                    .as_ref()
                    .is_ok_and(|s| tool != Tool::Rotate || s.can_rotate)
            {
                settings.tool = tool;
            }
        }
        if settings != service.transform {
            commands.push(Command::SetEditorTransform { settings });
            return true;
        }
    }
    let Ok(selection) = selection else {
        return false;
    };
    if service.transform.tool == Tool::Select || service.engine.playing {
        state.cancel();
        return false;
    }
    let (at, axes) = if let Some(d) = &state.draft {
        (d.pivot, d.basis)
    } else {
        (
            pivot(&selection, &service.transform),
            basis(&selection, &service.transform),
        )
    };
    let active = state.draft.as_ref().map(|d| d.handle);
    let display_pivot = if service.transform.tool == Tool::Move {
        at + state
            .draft
            .as_ref()
            .map_or(Vec3::ZERO, |d| Vec3::from(d.transform.translation))
    } else {
        at
    };
    let hit = draw(
        ui,
        camera,
        view,
        display_pivot,
        axes,
        service.transform.tool,
        active,
        selection.can_rotate,
    );
    if hit.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }
    if state.draft.is_none()
        && commands.is_empty()
        && pressed(ui).is_some_and(|p| view.contains(p))
        && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
        && let Some(handle) = hit
        && let Some(start) = pressed(ui)
    {
        let mut d = Draft {
            selection: (*selection).clone(),
            settings: service.transform.clone(),
            camera: camera.clone(),
            view,
            session: service.session_revision,
            editor: service.editor.revision,
            tick: service.engine.simulation.tick,
            handle,
            pivot: at,
            basis: axes,
            start,
            last: start,
            last_modifiers: ui.input(|i| i.modifiers),
            start_point: Vec3::ZERO,
            vertices: if let Target::Components {
                instance,
                components,
            } = &selection.target
            {
                let i = service
                    .engine
                    .scene
                    .meshes
                    .instances
                    .iter()
                    .find(|i| &i.id == instance)
                    .unwrap();
                transform::vertices(
                    &service
                        .engine
                        .scene
                        .meshes
                        .asset(&i.asset)
                        .unwrap()
                        .geometry,
                    components,
                )
                .unwrap_or_default()
            } else {
                vec![]
            },
            transform: IslandTransform {
                pivot: at.to_array(),
                translation: [0.; 3],
                rotation: Quat::IDENTITY.to_array(),
                scale: 1.,
            },
            preview: None,
            valid: false,
        };
        let Some(point) = plane(camera, view, start, at, normal(&d)) else {
            return true;
        };
        d.start_point = point;
        state.error.clear();
        state.draft = Some(d);
    }
    if let Some(d) = &mut state.draft {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
            let modifiers = ui.input(|i| i.modifiers);
            if pos != d.last || modifiers != d.last_modifiers {
                d.last = pos;
                d.last_modifiers = modifiers;
                if let Some(t) = delta(d, pos, modifiers) {
                    match transform::preview(
                        service,
                        &d.selection.target,
                        &d.selection.reference,
                        &t,
                    ) {
                        Ok(scene) => {
                            d.preview = Some(scene);
                            d.transform = t;
                            d.valid = true;
                            state.error.clear();
                        }
                        Err(e) => {
                            d.valid = false;
                            state.error = e.message;
                        }
                    }
                } else {
                    d.valid = false;
                    state.error = "Arraste paralelo ao plano; ajuste a vista.".into();
                }
            }
        }
        let t = &d.transform;
        let angle = Quat::from_array(t.rotation).to_axis_angle().1.to_degrees();
        let label = match d.settings.tool {
            Tool::Move => format!(
                "Mover  {:.3} · {:.3} · {:.3} m",
                t.translation[0], t.translation[1], t.translation[2]
            ),
            Tool::Rotate => format!("Girar  {angle:.1}°"),
            _ => format!("Escala  {:.3}×", t.scale),
        };
        ui.painter().text(
            view.left_bottom() + Vec2::new(16., -26.),
            egui::Align2::LEFT_BOTTOM,
            label,
            egui::FontId::monospace(13.),
            Color32::WHITE,
        );
        if let Some(scene) = &d.preview
            && let Target::Components { instance, .. } = &d.selection.target
            && let Some(i) = scene.meshes.instances.iter().find(|i| &i.id == instance)
            && let Ok(a) = scene.meshes.asset(&i.asset)
        {
            for &index in &d.vertices {
                let p = Vec3::from(i.position)
                    + Quat::from_array(i.rotation)
                        * (Vec3::from(a.geometry.positions[index as usize]) * i.scale);
                if let Some(pos) = project(camera, view, p) {
                    ui.painter()
                        .with_clip_rect(view.rect)
                        .circle_filled(pos, 3., EMBER);
                }
            }
        }
        if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
            let d = state.draft.take().unwrap();
            if d.valid
                && (d.transform.translation != [0.; 3]
                    || d.transform.rotation != Quat::IDENTITY.to_array()
                    || d.transform.scale != 1.)
            {
                commands.push(Command::TransformEditor {
                    target: d.selection.target.clone(),
                    reference: d.selection.reference.clone(),
                    transform: d.transform.clone(),
                });
                state.pending = Some(d);
            }
        }
        if !state.error.is_empty() {
            ui.painter().text(
                view.left_bottom() + Vec2::new(16., -8.),
                egui::Align2::LEFT_BOTTOM,
                "Limite da malha. Reduza o movimento; soltar cancela este gesto.",
                egui::FontId::proportional(12.),
                Color32::LIGHT_RED,
            );
        }
        return true;
    }
    false
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
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        s.execute(
            Command::CreateEditorMesh {
                primitive: crate::mesh_tools::Primitive::Sphere,
                resolution: 16,
                size: 2.,
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
        s.execute(
            Command::SetEditorTransform {
                settings: Settings {
                    tool,
                    ..Default::default()
                },
            }
            .into(),
        )
        .unwrap();
        let mut state = super::super::UiState::default();
        state.project.dismissed = true;
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
    ) -> (Rect, Vec<Command>) {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
            events,
            focused,
            ..Default::default()
        });
        let mut view = None;
        let mut commands = vec![];
        super::super::draw_editor(
            ctx,
            s,
            state,
            &mut commands,
            egui::TextureId::User(0),
            &mut view,
        );
        let _ = ctx.end_pass();
        (view.unwrap(), commands)
    }
    fn button(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: Default::default(),
        }
    }
    fn points(s: &Service, view: Rect, tool: Tool) -> (Pos2, Pos2) {
        let view = View::new(view, 1.);
        let camera = s.editor.current().camera.as_ref().unwrap();
        let selection = transform::selection(s, false).unwrap();
        let pivot = pivot(&selection, &s.transform);
        let basis = basis(&selection, &s.transform);
        let len = length(camera, view, pivot);
        match tool {
            Tool::Move => (
                project(camera, view, pivot + basis[0] * len * 0.8).unwrap(),
                project(camera, view, pivot + basis[0] * len * 1.1).unwrap(),
            ),
            Tool::Rotate => {
                let at = |a: f32| {
                    project(
                        camera,
                        view,
                        pivot + (basis[0] * a.cos() + basis[1] * a.sin()) * len,
                    )
                    .unwrap()
                };
                (at(0.8), at(1.1))
            }
            Tool::Scale => {
                let p = project(camera, view, pivot).unwrap() + Vec2::new(52., -52.);
                (p, p + Vec2::new(18., 0.))
            }
            _ => unreachable!(),
        }
    }
    #[test]
    fn native_drag_preview_release_uses_shared_command_and_exact_undo_for_three_tools() {
        for tool in [Tool::Move, Tool::Rotate, Tool::Scale] {
            let (_root, mut s, mut state, ctx) = fixture(tool);
            let before = s.engine.snapshot();
            let (view, _) = pass(&s, &mut state, &ctx, vec![], true);
            let (start, end) = points(&s, view, tool);
            pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(start)],
                true,
            );
            let (_, commands) = pass(&s, &mut state, &ctx, vec![button(start, true)], true);
            assert!(commands.is_empty(), "{tool:?}");
            assert!(state.gizmo.draft.is_some(), "{tool:?}");
            let (_, commands) = pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(end)],
                true,
            );
            assert!(commands.is_empty());
            assert!(
                state.gizmo.preview().is_some(),
                "{tool:?} {}",
                state.gizmo.error
            );
            assert_eq!(s.engine.snapshot(), before);
            let preview = state.gizmo.preview().unwrap().clone();
            let (_, commands) = pass(&s, &mut state, &ctx, vec![button(end, false)], true);
            assert_eq!(commands.len(), 1, "{tool:?}");
            assert!(matches!(commands[0], Command::TransformEditor { .. }));
            assert_eq!(
                state.gizmo.preview(),
                Some(&preview),
                "Release must retain preview until dispatcher accepts"
            );
            for c in commands {
                s.execute(c.into()).unwrap();
            }
            assert_eq!(s.engine.scene, preview);
            assert_eq!(
                s.engine.snapshot()["history"]["undo"].as_u64(),
                Some(before["history"]["undo"].as_u64().unwrap() + 1)
            );
            pass(&s, &mut state, &ctx, vec![], true);
            assert!(state.gizmo.preview().is_none());
            s.execute(Command::Undo.into()).unwrap();
            assert_eq!(s.engine.snapshot()["scene"], before["scene"]);
        }
    }
    #[test]
    fn escape_focus_revision_and_resize_cancel_without_touching_history_or_redo() {
        for reason in 0..5 {
            let (_root, mut s, mut state, ctx) = fixture(Tool::Move);
            // Create a real redo branch before the cancelled gesture.
            let mut e = s.engine.scene.entities[0].clone();
            e.name.push('a');
            s.execute(
                Command::Upsert {
                    entity: Box::new(e),
                }
                .into(),
            )
            .unwrap();
            s.execute(Command::Undo.into()).unwrap();
            let (view, _) = pass(&s, &mut state, &ctx, vec![], true);
            let (start, end) = points(&s, view, Tool::Move);
            pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(start)],
                true,
            );
            pass(&s, &mut state, &ctx, vec![button(start, true)], true);
            pass(
                &s,
                &mut state,
                &ctx,
                vec![egui::Event::PointerMoved(end)],
                true,
            );
            assert!(state.gizmo.preview().is_some());
            let events = match reason {
                0 => vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }],
                2 => {
                    s.execute(Command::Reset.into()).unwrap();
                    vec![]
                }
                3 => {
                    state.gizmo.draft.as_mut().unwrap().view.rect.max.x -= 1.;
                    vec![]
                }
                4 => {
                    s.execute(
                        Command::SetEditorTransform {
                            settings: Settings {
                                space: Space::Local,
                                ..s.transform.clone()
                            },
                        }
                        .into(),
                    )
                    .unwrap();
                    vec![]
                }
                _ => vec![],
            };
            let before = s.engine.snapshot();
            let (_, commands) = pass(&s, &mut state, &ctx, events, reason != 1);
            assert!(commands.is_empty());
            assert!(state.gizmo.preview().is_none(), "reason{reason}");
            assert_eq!(s.engine.snapshot(), before);
        }
    }
    #[test]
    fn world_local_axes_planes_snap_and_fine_adjustment_are_frame_independent() {
        let (_root, s, _, _) = fixture(Tool::Move);
        let view = View::new(
            Rect::from_min_size(Pos2::new(80., 140.), Vec2::new(800., 600.)),
            1.,
        );
        let camera = s.editor.current().camera.clone().unwrap();
        let mut selection = transform::selection(&s, false).unwrap();
        selection.orientation = Quat::from_rotation_y(0.8).to_array();
        let settings = Settings {
            space: Space::Local,
            snap: true,
            move_step: 0.1,
            ..s.transform.clone()
        };
        let at = pivot(&selection, &settings);
        let basis = basis(&selection, &settings);
        let start = project(&camera, view, at).unwrap();
        for handle in [
            Handle::Axis(0),
            Handle::Axis(1),
            Handle::Axis(2),
            Handle::Plane(0),
            Handle::Plane(1),
            Handle::Plane(2),
        ] {
            let mut d = Draft {
                selection: selection.clone(),
                settings: settings.clone(),
                camera: camera.clone(),
                view,
                session: 0,
                editor: 0,
                tick: 0,
                handle,
                pivot: at,
                basis,
                start,
                last: start,
                last_modifiers: Default::default(),
                start_point: Vec3::ZERO,
                vertices: vec![],
                transform: IslandTransform {
                    pivot: at.to_array(),
                    translation: [0.; 3],
                    rotation: Quat::IDENTITY.to_array(),
                    scale: 1.,
                },
                preview: None,
                valid: false,
            };
            d.start_point = plane(&camera, view, start, at, normal(&d)).unwrap();
            let movement = match handle {
                Handle::Axis(i) => basis[i] * 0.26,
                Handle::Plane(i) => basis[(i + 1) % 3] * 0.26 + basis[(i + 2) % 3] * 0.14,
                _ => unreachable!(),
            };
            let end = project(&camera, view, at + movement).unwrap();
            let t = delta(&d, end, Default::default()).unwrap();
            let v = Vec3::from(t.translation);
            match handle {
                Handle::Axis(i) => assert!((v.dot(basis[i]) - 0.3).abs() < 1e-5),
                Handle::Plane(i) => {
                    assert!((v.dot(basis[(i + 1) % 3]) - 0.3).abs() < 1e-5);
                    assert!((v.dot(basis[(i + 2) % 3]) - 0.1).abs() < 1e-5);
                    assert!(v.dot(basis[i]).abs() < 1e-5)
                }
                _ => unreachable!(),
            }
            d.settings.snap = false;
            let modifiers = egui::Modifiers {
                shift: true,
                ..Default::default()
            };
            let fine = Vec3::from(delta(&d, end, modifiers).unwrap().translation);
            assert!(fine.distance(movement * 0.1) < 1e-5);
        }
        assert_eq!(basis[0], Quat::from_array(selection.orientation) * Vec3::X);
        assert_eq!(
            super::basis(&selection, &Settings::default()),
            [Vec3::X, Vec3::Y, Vec3::Z]
        );
        assert_eq!(
            super::pivot(
                &selection,
                &Settings {
                    pivot: Pivot::Custom,
                    custom_pivot: [1., 2., 3.],
                    ..Default::default()
                }
            ),
            Vec3::new(1., 2., 3.)
        );
    }
    #[test]
    fn fast_drag_delivered_in_one_pass_is_not_downgraded_to_selection() {
        let (_root, mut s, mut state, ctx) = fixture(Tool::Move);
        let (view, _) = pass(&s, &mut state, &ctx, vec![], true);
        let (start, end) = points(&s, view, Tool::Move);
        let before = s.engine.scene.clone();
        let (_, commands) = pass(
            &s,
            &mut state,
            &ctx,
            vec![
                egui::Event::PointerMoved(start),
                button(start, true),
                egui::Event::PointerMoved(end),
                button(end, false),
            ],
            true,
        );
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Command::TransformEditor { .. }));
        for c in commands {
            s.execute(c.into()).unwrap();
        }
        assert_ne!(s.engine.scene, before);
    }
    #[test]
    fn idle_selection_does_not_rebuild_topology_or_duplicate_component_arrays() {
        let (_root, s, mut state, _) = fixture(Tool::Move);
        let first = state.gizmo.selection(&s, false).unwrap();
        let second = state.gizmo.selection(&s, false).unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &second));
    }
    #[test]
    fn hidpi_gizmo_projection_and_unbounded_drag_match_clamped_render_texture() {
        let (_root, s, _, _) = fixture(Tool::Move);
        let camera = s.editor.current().camera.as_ref().unwrap();
        for (size, ppp) in [
            (Vec2::new(3000., 2000.), 2.),
            (Vec2::new(1500., 900.), 2.),
            (Vec2::new(800., 600.), 1.5),
        ] {
            let view = View::new(Rect::from_min_size(Pos2::new(40., 90.), size), ppp);
            let projection =
                crate::mesh_screen::Projection::new(camera, view.pixels[0], view.pixels[1])
                    .unwrap();
            let world = Vec3::new(2.4, 1.3, 0.1);
            let pixel = projection.project(world).unwrap();
            let ui_point = project(camera, view, world).unwrap();
            assert!(
                (ui_point.x - view.left() - pixel.x / view.pixels[0] as f32 * view.width()).abs()
                    < 1e-4
            );
            let (eye, direction) = ray(camera, view, ui_point);
            assert!((world - eye).normalize().dot(direction) > 0.999999);
            let (start, picking_direction) =
                crate::picking::ray(camera, pixel.x, pixel.y, view.pixels[0], view.pixels[1])
                    .unwrap();
            assert!(picking_direction.normalize().dot(direction) > 0.999999);
            assert!(start.distance(eye) > 0.); // shared ray starts at near plane, gizmo at eye
        }
    }
    #[test]
    fn viewport_focus_enables_tool_shortcuts_but_text_focus_and_play_keep_them_disabled() {
        let (_root, mut s, mut state, ctx) = fixture(Tool::Select);
        let (view, _) = pass(&s, &mut state, &ctx, vec![], true);
        let key = || egui::Event::Key {
            key: egui::Key::W,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        };
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("viewport_image")));
        let (_, commands) = pass(
            &s,
            &mut state,
            &ctx,
            vec![egui::Event::PointerMoved(view.center()), key()],
            true,
        );
        assert!(commands.iter().any(
            |c| matches!(c,Command::SetEditorTransform{settings} if settings.tool==Tool::Move)
        ));
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("test_numeric_field")));
        let (_, commands) = pass(&s, &mut state, &ctx, vec![key()], true);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, Command::SetEditorTransform { .. }))
        );
        s.execute(Command::Play.into()).unwrap();
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("viewport_image")));
        let (_, commands) = pass(&s, &mut state, &ctx, vec![key()], true);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, Command::SetEditorTransform { .. }))
        );
    }
}
