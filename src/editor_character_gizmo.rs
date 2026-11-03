//! Facial direct manipulation. Preview is disposable; release submits one shared command.
#[cfg(test)]
#[path = "editor_character_gizmo_tests.rs"]
mod tests;
use super::*;
use crate::{
    mesh_topology::Reference,
    meshes::character::HandlePose,
    model::{Camera, Scene},
};
use glam::Vec3;

const AXES: [Color32; 3] = [
    Color32::from_rgb(235, 100, 100),
    Color32::from_rgb(110, 215, 145),
    Color32::from_rgb(110, 165, 250),
];
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
    fn projection(self, camera: &Camera) -> Option<crate::mesh_screen::Projection> {
        crate::mesh_screen::Projection::new(camera, self.pixels[0], self.pixels[1]).ok()
    }
    fn project(self, projection: &crate::mesh_screen::Projection, p: [f32; 3]) -> Option<Pos2> {
        projection.project(p.into()).map(|v| {
            self.rect.min
                + Vec2::new(
                    v.x / self.pixels[0] as f32 * self.rect.width(),
                    v.y / self.pixels[1] as f32 * self.rect.height(),
                )
        })
    }
}
struct Draft {
    prefix: String,
    handle: HandlePose,
    axis: Option<usize>,
    axis_screen: Vec2,
    axis_meters: f32,
    scale: f32,
    start: Pos2,
    last: Pos2,
    offset: [f32; 3],
    reference: Reference,
    camera: Camera,
    view: View,
    editor: u64,
    session: u64,
    view_revision: u64,
    handle_view: crate::character_editor::HandleView,
    preview: Option<Scene>,
    poses: Vec<HandlePose>,
    valid: bool,
    limited: bool,
}
#[derive(Default)]
pub(super) struct State {
    draft: Option<Draft>,
    pending: Option<Draft>,
    visibility: crate::mesh_screen::Visibility,
    consume_release: bool,
    pub error: String,
}
impl State {
    pub(super) fn preview(&self) -> Option<&Scene> {
        self.draft
            .as_ref()
            .or(self.pending.as_ref())
            .and_then(|d| d.preview.as_ref())
    }
    pub(super) fn cancel(&mut self) {
        self.consume_release |= self.draft.is_some();
        self.draft = None;
        self.pending = None;
    }
    pub(super) fn validate(&mut self, service: &Service) {
        let invalid = self
            .draft
            .as_ref()
            .or(self.pending.as_ref())
            .is_some_and(|d| {
                !enabled(service)
                    || d.reference != service.engine.mesh_reference()
                    || d.editor != service.editor.revision
                    || d.session != service.session_revision
                    || d.view_revision != service.engine.view_revision
                    || d.handle_view != service.character_editor.handle_view
                    || &d.camera != camera(service)
                    || service.character_editor.selected.as_deref() != Some(d.prefix.as_str())
            });
        if invalid {
            self.cancel();
        }
    }
}
pub(super) fn enabled(service: &Service) -> bool {
    service.character_editor.open
        && service.character_editor.handle_view.enabled
        && service.character_editor.ready
        && !service.engine.playing
}
fn camera(service: &Service) -> &Camera {
    service
        .editor
        .current()
        .camera
        .as_ref()
        .unwrap_or(&service.engine.scene.camera)
}
fn press(ui: &egui::Ui) -> Option<Pos2> {
    ui.input(|i| {
        i.events.iter().find_map(|e| match e {
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
fn metres_per_pixel(camera: &Camera, view: View, p: Vec3) -> f32 {
    let forward = (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize();
    let depth = (p - Vec3::from(camera.position))
        .dot(forward)
        .max(crate::picking::NEAR);
    2. * depth * (camera.fov_degrees.to_radians() * 0.5).tan() / view.rect.height()
}
fn ray(camera: &Camera, view: View, p: Pos2) -> Vec3 {
    let forward = (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize();
    let right = forward.cross(Vec3::Y).normalize();
    let up = right.cross(forward);
    let h = (camera.fov_degrees.to_radians() * 0.5).tan();
    let x = (2. * (p.x - view.rect.left()) / view.rect.width() - 1.) * h * view.pixels[0] as f32
        / view.pixels[1] as f32;
    let y = (1. - 2. * (p.y - view.rect.top()) / view.rect.height()) * h;
    (forward + right * x + up * y).normalize()
}
fn plane(camera: &Camera, view: View, pos: Pos2, pivot: Vec3) -> Option<Vec3> {
    let normal = (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize();
    let eye = Vec3::from(camera.position);
    let direction = ray(camera, view, pos);
    let denominator = normal.dot(direction);
    if denominator.abs() < 0.0001 {
        return None;
    }
    let distance = (pivot - eye).dot(normal) / denominator;
    (distance > 0. && distance.is_finite()).then(|| eye + direction * distance)
}
fn endpoint(
    view: View,
    projection: &crate::mesh_screen::Projection,
    camera: &Camera,
    h: &HandlePose,
    axis: usize,
) -> Option<(Pos2, Vec2, f32)> {
    let center = view.project(projection, h.position)?;
    let metres = metres_per_pixel(camera, view, h.position.into()) * 54.;
    let end = view.project(
        projection,
        (Vec3::from(h.position) + Vec3::from(h.basis[axis]) * metres).to_array(),
    )?;
    let vector = end - center;
    // A camera-aligned depth axis remains reachable as a labelled screen stub.
    let vector = if vector.length() < 14. {
        Vec2::new(28., -36.)
    } else {
        vector
    };
    Some((center + vector, vector, metres))
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
pub(super) fn viewport(
    ui: &egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
    response: &egui::Response,
    rect: Rect,
) -> bool {
    state.validate(service);
    let active = enabled(service);
    if !active {
        state.cancel();
    }
    let released = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary));
    if state.consume_release {
        if released || !ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
            state.consume_release = false;
        }
        return true;
    }
    if !active {
        return false;
    }
    let view = View::new(rect, ui.ctx().pixels_per_point());
    if state.draft.as_ref().is_some_and(|d| d.view != view) {
        state.cancel();
    }
    if ui.input(|i| !i.focused || i.key_pressed(egui::Key::Escape)) {
        state.cancel();
    }
    let released = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary));
    if state.consume_release {
        if released || !ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
            state.consume_release = false;
        }
        return true;
    }
    let camera = camera(service);
    let Some(projection) = view.projection(camera) else {
        return false;
    };
    if let Err(error) = state.visibility.prepare(&service.engine) {
        state.error = error.message;
        return false;
    }
    let painter = ui.painter().with_clip_rect(rect);
    let pointer = press(ui)
        .or_else(|| ui.input(|i| i.pointer.hover_pos()))
        .filter(|p| rect.contains(*p));
    let poses = state
        .draft
        .as_ref()
        .filter(|d| !d.poses.is_empty())
        .map_or(service.character_editor.handles.as_slice(), |d| {
            d.poses.as_slice()
        });
    let selected = state
        .draft
        .as_ref()
        .map(|d| d.handle.name.as_str())
        .or(service.character_editor.handle_view.selected.as_deref());
    let mut hovered = None;
    let mut best = 10.;
    for h in poses.iter().filter(|h| {
        service.character_editor.handle_view.group.is_none()
            || h.group == service.character_editor.handle_view.group
    }) {
        let Some(p) = view.project(&projection, h.position) else {
            continue;
        };
        if !rect.contains(p) {
            continue;
        }
        let active = selected == Some(h.name.as_str());
        if !active && !state.visibility.visible(&projection, h.position.into()) {
            continue;
        }
        let distance = pointer.map_or(f32::INFINITY, |q| q.distance(p));
        if distance < best {
            best = distance;
            hovered = Some((h.name.clone(), None, Vec2::ZERO, 0.));
        }
        painter.circle_filled(
            p,
            if active { 6. } else { 4. },
            if active {
                Color32::from_rgb(110, 235, 235)
            } else {
                Color32::from_rgb(235, 182, 112)
            },
        );
        painter.circle_stroke(
            p,
            if active { 6. } else { 4. },
            egui::Stroke::new(1.0_f32, Color32::from_gray(25)),
        );
    }
    if let Some(h) = selected.and_then(|name| {
        poses.iter().find(|h| {
            h.name == name
                && (service.character_editor.handle_view.group.is_none()
                    || h.group == service.character_editor.handle_view.group)
        })
    }) && let Some(center) = view.project(&projection, h.position)
    {
        for (axis, color) in AXES.into_iter().enumerate() {
            if axis == 0 && h.midline && service.character_editor.handle_view.symmetry {
                continue;
            }
            if let Some((end, vector, metres)) = endpoint(view, &projection, camera, h, axis) {
                painter.arrow(center, vector, egui::Stroke::new(2.0_f32, color));
                painter.circle_filled(end, 4., color);
                painter.text(
                    end + Vec2::new(3., -6.),
                    egui::Align2::LEFT_BOTTOM,
                    ["X", "Y", "Z"][axis],
                    egui::FontId::proportional(12.),
                    color,
                );
                if let Some(pointer) = pointer
                    && (pointer.distance(end) < 9.
                        || pointer.distance(center) > 9.
                            && distance_segment(pointer, center, end) < 5.)
                {
                    hovered = Some((h.name.clone(), Some(axis), vector, metres));
                }
            }
        }
        painter.text(
            center + Vec2::new(10., 8.),
            egui::Align2::LEFT_TOP,
            &h.label,
            egui::FontId::proportional(13.),
            Color32::WHITE,
        );
    }
    if hovered.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }
    if let Some(start) = press(ui).filter(|p| rect.contains(*p))
        && state.draft.is_none()
        && let Some((name, axis, axis_screen, axis_meters)) = hovered
    {
        let h = service
            .character_editor
            .handles
            .iter()
            .find(|h| h.name == name)
            .unwrap()
            .clone();
        if let Some(binding) = service.character_editor.binding(&service.engine.scene) {
            state.pending = None;
            state.error.clear();
            state.draft = Some(Draft {
                prefix: binding.recipe.placement.prefix.clone(),
                offset: h.offset,
                handle: h,
                axis,
                axis_screen,
                axis_meters,
                scale: binding.recipe.placement.scale,
                start,
                last: start,
                reference: service.engine.mesh_reference(),
                camera: camera.clone(),
                view,
                editor: service.editor.revision,
                session: service.session_revision,
                view_revision: service.engine.view_revision,
                handle_view: service.character_editor.handle_view.clone(),
                preview: None,
                poses: Vec::new(),
                valid: true,
                limited: false,
            });
        }
    }
    let Some(d) = state.draft.as_mut() else {
        return response.clicked_by(egui::PointerButton::Primary);
    };
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    if let Some(pos) = ui.input(|i| i.pointer.interact_pos())
        && pos != d.last
    {
        d.last = pos;
        let world_delta = if let Some(axis) = d.axis {
            let t = (pos - d.start).dot(d.axis_screen) / d.axis_screen.length_sq().max(1.);
            Some(Vec3::from(d.handle.basis[axis]) * t * d.axis_meters)
        } else {
            plane(&d.camera, view, pos, d.handle.position.into())
                .zip(plane(&d.camera, view, d.start, d.handle.position.into()))
                .map(|(p, q)| p - q)
        };
        if let Some(delta) = world_delta {
            let requested: [f32; 3] = std::array::from_fn(|i| {
                d.handle.offset[i] + delta.dot(d.handle.basis[i].into()) / d.scale
            });
            d.offset = std::array::from_fn(|i| {
                requested[i].clamp(-d.handle.limits[i], d.handle.limits[i])
            });
            if d.handle.midline && d.handle_view.symmetry {
                d.offset[0] = 0.;
            }
            d.limited = d.offset != requested;
            let result = service
                .character_editor
                .prepare_handle(
                    &service.engine,
                    &d.prefix,
                    &d.handle.name,
                    d.offset,
                    d.handle_view.symmetry,
                    &d.reference,
                )
                .and_then(|prepared| {
                    let poses = service
                        .character_editor
                        .source
                        .as_ref()
                        .unwrap()
                        .handle_poses(
                            &prepared.recipe.configuration,
                            &prepared.recipe.placement,
                            &prepared.document,
                        )?;
                    let preview = prepared.preview(&service.engine)?;
                    Ok((preview, poses))
                });
            match result {
                Ok((preview, poses)) => {
                    d.preview = Some(preview);
                    d.poses = poses;
                    d.valid = true;
                    state.error.clear();
                }
                Err(error) => {
                    d.preview = None;
                    d.poses.clear();
                    d.valid = false;
                    state.error = error.message;
                }
            }
        }
    }
    painter.text(
        rect.left_top() + Vec2::new(12., 12.),
        egui::Align2::LEFT_TOP,
        if !d.valid {
            state.error.as_str()
        } else if d.limited {
            "Limite deste ponto · solte para aplicar · Esc cancela"
        } else {
            "Prévia · solte para aplicar · Esc cancela"
        },
        egui::FontId::proportional(13.),
        if d.valid {
            Color32::WHITE
        } else {
            Color32::LIGHT_RED
        },
    );
    if released {
        let d = state.draft.take().unwrap();
        if d.valid && d.offset != d.handle.offset {
            commands.push(Command::SetCharacterHandle {
                prefix: d.prefix.clone(),
                handle: d.handle.name.clone(),
                offset: d.offset,
                symmetry: d.handle_view.symmetry,
                reference: d.reference.clone(),
                gesture: None,
            });
            state.pending = Some(d);
        } else if d.valid {
            let mut view = service.character_editor.handle_view.clone();
            view.selected = Some(d.handle.name.clone());
            commands.push(Command::SetCharacterHandleView { view });
        }
    }
    true
}
