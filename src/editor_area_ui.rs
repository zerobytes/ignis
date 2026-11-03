//! Compact area actions. Text remains in tooltips, accessibility names and panel choices.
use super::*;
#[derive(Clone, Copy)]
enum Icon {
    Maximize,
    Restore,
    Side,
    Stack,
    Tab,
    Close,
}
pub(super) fn width(panel: Panel) -> f32 {
    if panel == Panel::Viewport { 94. } else { 116. }
}
fn button(ui: &mut egui::Ui, icon: Icon, name: &str) -> egui::Response {
    let response = ui.add(
        egui::Button::new("")
            .min_size(Vec2::new(20., 24.))
            .fill(Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE),
    );
    let r = Rect::from_center_size(response.rect.center(), Vec2::splat(13.));
    let painter = ui.painter();
    let stroke = egui::Stroke::new(
        1.4_f32,
        if response.hovered() || response.has_focus() {
            EMBER
        } else {
            design_ui::MUTED
        },
    );
    match icon {
        Icon::Close => {
            painter.line_segment([r.left_top(), r.right_bottom()], stroke);
            painter.line_segment([r.left_bottom(), r.right_top()], stroke);
        }
        Icon::Restore => {
            painter.rect_stroke(
                r.translate(Vec2::new(2., -2.)).shrink(1.),
                1.,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.rect_filled(r.translate(Vec2::new(-2., 2.)).shrink(1.), 1., PANEL);
            painter.rect_stroke(
                r.translate(Vec2::new(-2., 2.)).shrink(1.),
                1.,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
        Icon::Maximize => {
            painter.rect_stroke(r, 1., stroke, egui::StrokeKind::Inside);
        }
        Icon::Side | Icon::Stack => {
            painter.rect_stroke(r, 1., stroke, egui::StrokeKind::Inside);
            if matches!(icon, Icon::Side) {
                painter.line_segment([r.center_top(), r.center_bottom()], stroke);
            } else {
                painter.line_segment([r.left_center(), r.right_center()], stroke);
            }
        }
        Icon::Tab => {
            painter.rect_stroke(r, 1., stroke, egui::StrokeKind::Inside);
            painter.line_segment(
                [
                    r.left_top() + Vec2::new(0., 4.),
                    r.right_top() + Vec2::new(0., 4.),
                ],
                stroke,
            );
            let c = r.center() + Vec2::new(0., 2.);
            painter.line_segment([c - Vec2::new(2.5, 0.), c + Vec2::new(2.5, 0.)], stroke);
            painter.line_segment([c - Vec2::new(0., 2.5), c + Vec2::new(0., 2.5)], stroke);
        }
    }
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), name));
    response.on_hover_text(name)
}
pub(super) fn controls(
    ui: &mut egui::Ui,
    service: &Service,
    area: u32,
    active: Panel,
    commands: &mut Vec<Command>,
) {
    ui.spacing_mut().item_spacing.x = 2.;
    if active != Panel::Viewport && button(ui, Icon::Close, "Fechar aba").clicked() {
        edit(
            commands,
            Action::Close {
                area,
                panel: active,
            },
        );
    }
    let open = button(ui, Icon::Tab, "Abrir aba");
    egui::Popup::menu(&open).show(|ui| {
        for panel in Panel::ALL.into_iter().filter(|p| *p != Panel::Viewport) {
            if ui.button(panel.label()).clicked() {
                edit(commands, Action::Open { area, panel });
                ui.close();
            }
        }
    });
    for (axis, icon, name) in [
        (Axis::Vertical, Icon::Stack, "Dividir acima/abaixo"),
        (Axis::Horizontal, Icon::Side, "Dividir lado a lado"),
    ] {
        let split = button(ui, icon, name);
        egui::Popup::menu(&split).show(|ui| {
            for panel in Panel::ALL.into_iter().filter(|p| *p != Panel::Viewport) {
                if ui.button(panel.label()).clicked() {
                    edit(commands, Action::Split { area, axis, panel });
                    ui.close();
                }
            }
        });
    }
    let maximized = service.editor.current().maximized.is_some();
    if button(
        ui,
        if maximized {
            Icon::Restore
        } else {
            Icon::Maximize
        },
        if maximized {
            "Restaurar área"
        } else {
            "Maximizar área"
        },
    )
    .clicked()
    {
        edit(
            commands,
            Action::Maximize {
                area: (!maximized).then_some(area),
            },
        );
    }
}
