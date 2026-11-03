//! Shared visual tokens and compact authoring controls.
use egui::{Color32, Response, RichText, Stroke, Ui, Vec2};
pub(super) const BG: Color32 = Color32::from_rgb(24, 25, 28);
pub(super) const PANEL: Color32 = Color32::from_rgb(31, 32, 36);
pub(super) const LINE: Color32 = Color32::from_rgb(53, 55, 62);
pub(super) const EMBER: Color32 = Color32::from_rgb(239, 158, 103);
pub(super) const TEXT: Color32 = Color32::from_rgb(230, 231, 235);
pub(super) const MUTED: Color32 = Color32::from_rgb(165, 169, 179);
pub(super) const CONTROL_HEIGHT: f32 = 26.;

pub(super) fn configure(ctx: &egui::Context) {
    let mut s = egui::Style {
        visuals: egui::Visuals::dark(),
        animation_time: 0.,
        explanation_tooltips: false,
        ..Default::default()
    };
    s.visuals.panel_fill = BG;
    s.visuals.window_fill = PANEL;
    s.visuals.extreme_bg_color = Color32::from_rgb(18, 21, 25);
    s.visuals.override_text_color = Some(TEXT);
    s.visuals.weak_text_color = Some(MUTED);
    s.visuals.selection.bg_fill = Color32::from_rgb(67, 50, 40);
    s.visuals.selection.stroke = Stroke::new(1.0_f32, EMBER);
    s.visuals.slider_trailing_fill = true;
    let w = &mut s.visuals.widgets;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, LINE);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.inactive.bg_fill = Color32::from_rgb(44, 45, 51);
    w.inactive.weak_bg_fill = Color32::from_rgb(39, 40, 45);
    w.inactive.bg_stroke = Stroke::new(1.0_f32, LINE);
    w.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    w.hovered.bg_fill = Color32::from_rgb(58, 60, 67);
    w.hovered.weak_bg_fill = Color32::from_rgb(48, 50, 57);
    w.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(99, 102, 113));
    w.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    w.active.bg_fill = Color32::from_rgb(67, 50, 40);
    w.active.bg_stroke = Stroke::new(1.0_f32, EMBER);
    w.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    w.open.weak_bg_fill = Color32::from_rgb(42, 43, 48);
    w.open.bg_stroke = Stroke::new(1.0_f32, LINE);
    for v in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        v.corner_radius = egui::CornerRadius::same(4);
    }
    s.visuals.window_corner_radius = egui::CornerRadius::same(8);
    s.spacing.window_margin = egui::Margin::same(12);
    s.spacing.item_spacing = Vec2::new(8., 6.);
    s.spacing.button_padding = Vec2::new(8., 4.);
    s.spacing.interact_size.y = CONTROL_HEIGHT;
    s.spacing.combo_width = 112.;
    s.spacing.slider_width = 148.;
    s.drag_value_text_style = egui::TextStyle::Monospace;
    for (kind, size) in [
        (egui::TextStyle::Body, 13.),
        (egui::TextStyle::Button, 13.),
        (egui::TextStyle::Small, 11.),
        (egui::TextStyle::Heading, 18.),
    ] {
        s.text_styles.insert(kind, egui::FontId::proportional(size));
    }
    s.text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(12.));
    ctx.set_style(s);
}

pub(super) fn tab(ui: &mut Ui, label: &str, active: bool, drag: bool) -> Response {
    let r = ui.add(
        egui::Button::new(RichText::new(label).color(if active { TEXT } else { MUTED }))
            .selected(active)
            .fill(if active {
                Color32::from_rgb(43, 45, 51)
            } else {
                Color32::TRANSPARENT
            })
            .stroke(Stroke::NONE)
            .sense(if drag {
                egui::Sense::click_and_drag()
            } else {
                egui::Sense::click()
            }),
    );
    if r.hovered() || r.has_focus() {
        ui.painter().rect_stroke(
            r.rect.shrink(0.5),
            3,
            Stroke::new(1.0_f32, if r.has_focus() { EMBER } else { LINE }),
            egui::StrokeKind::Inside,
        );
    }
    if active && !drag {
        ui.painter().line_segment(
            [r.rect.left_bottom(), r.rect.right_bottom()],
            Stroke::new(1.5_f32, EMBER),
        );
    }
    r
}

pub(super) fn section(ui: &mut Ui, title: &str, contents: impl FnOnce(&mut Ui)) {
    ui.add_space(8.);
    egui::CollapsingHeader::new(RichText::new(title).strong().color(TEXT))
        .default_open(true)
        .show(ui, contents);
}

/// Retain egui's text editing, focus and keyboard semantics, but scrub only X.
/// The native drag widget also uses Y; correcting a local draft avoids changing
/// global pointer input or publishing false edits from vertical mouse movement.
pub(super) struct Scrub<'a> {
    value: &'a mut f32,
    speed: f64,
    range: std::ops::RangeInclusive<f32>,
    decimals: usize,
    suffix: &'a str,
    prefix: &'a str,
    percent: bool,
}
impl<'a> Scrub<'a> {
    pub fn new(value: &'a mut f32) -> Self {
        Self {
            value,
            speed: 0.01,
            range: f32::MIN..=f32::MAX,
            decimals: 3,
            suffix: "",
            prefix: "",
            percent: false,
        }
    }
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }
    pub fn range(mut self, range: std::ops::RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }
    pub fn fixed_decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }
    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }
    pub fn percent(mut self) -> Self {
        self.percent = true;
        self
    }
}
impl egui::Widget for Scrub<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let old = *self.value;
        let mut draft = old;
        let mut widget = egui::DragValue::new(&mut draft)
            .speed(self.speed)
            .range(self.range.clone())
            .clamp_existing_to_range(false)
            .fixed_decimals(self.decimals)
            .suffix(self.suffix)
            .prefix(self.prefix)
            .update_while_editing(false);
        if self.percent {
            widget = widget
                .custom_formatter(|v, _| format!("{:.0}%", v * 100.))
                .custom_parser(|s| {
                    s.trim_end_matches('%')
                        .replace(',', ".")
                        .parse::<f64>()
                        .ok()
                        .map(|v| v / 100.)
                });
        }
        let mut response = ui.add(widget);
        let memory = response.id.with("horizontal_value");
        if response.dragged() {
            let precise = if response.drag_started() {
                old as f64
            } else {
                ui.data(|d| d.get_temp::<(f64, f32)>(memory))
                    .filter(|(_, published)| *published == old)
                    .map_or(old as f64, |(precise, _)| precise)
            };
            let factor = if ui.input(|i| i.modifiers.shift) {
                0.1
            } else {
                1.
            };
            let next = (precise + response.drag_delta().x as f64 * self.speed * factor)
                .clamp(*self.range.start() as f64, *self.range.end() as f64);
            *self.value = next as f32;
            ui.data_mut(|d| d.insert_temp(memory, (next, *self.value)));
        } else {
            *self.value = draft;
            if response.drag_stopped() {
                ui.data_mut(|d| d.remove::<(f64, f32)>(memory));
            }
        }
        response
            .flags
            .set(egui::response::Flags::CHANGED, *self.value != old);
        response.widget_info(|| egui::WidgetInfo::drag_value(ui.is_enabled(), *self.value as f64));
        response.on_hover_text("Arraste na horizontal para ajustar. Shift: ajuste fino. Clique para digitar; Enter confirma.")
    }
}

pub(super) fn field<R>(ui: &mut Ui, label: &str, contents: impl FnOnce(&mut Ui) -> R) -> R {
    if ui.available_width() < 270. {
        return ui
            .vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.;
                ui.label(RichText::new(label).color(MUTED));
                contents(ui)
            })
            .inner;
    }
    ui.horizontal(|ui| {
        let width = (ui.available_width() * 0.3).clamp(64., 104.);
        ui.add_sized(
            [width, CONTROL_HEIGHT],
            egui::Label::new(RichText::new(label).color(MUTED))
                .halign(egui::Align::Min)
                .truncate(),
        );
        contents(ui)
    })
    .inner
}

pub(super) fn vector(ui: &mut Ui, values: &mut [f32; 3], speed: f64) -> bool {
    let mut changed = false;
    if ui.available_width() < 230. {
        for (axis, value) in ["X  ", "Y  ", "Z  "].into_iter().zip(values) {
            changed |= ui
                .add_sized(
                    [ui.available_width(), CONTROL_HEIGHT],
                    Scrub::new(value).prefix(axis).speed(speed),
                )
                .changed();
        }
    } else {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 4.;
            ui.columns(3, |columns| {
                for ((column, axis), value) in
                    columns.iter_mut().zip(["X  ", "Y  ", "Z  "]).zip(values)
                {
                    column.set_clip_rect(column.max_rect().intersect(column.clip_rect()));
                    changed |= column
                        .add_sized(
                            [column.available_width(), CONTROL_HEIGHT],
                            Scrub::new(value).prefix(axis).speed(speed),
                        )
                        .changed();
                }
            });
        });
    }
    changed
}

pub(super) fn primary(label: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(label).color(Color32::from_rgb(30, 24, 19)))
        .fill(EMBER)
        .stroke(Stroke::NONE)
}

pub(super) fn action(ui: &mut Ui, label: &str) -> Response {
    ui.add_sized(
        [ui.available_width(), CONTROL_HEIGHT],
        egui::Button::new(label),
    )
}

pub(super) fn list_row(ui: &mut Ui, label: &str, selected: bool) -> Response {
    ui.add_sized(
        [ui.available_width(), CONTROL_HEIGHT],
        egui::Button::selectable(selected, (label, egui::Atom::grow())).truncate(),
    )
}

pub(super) fn short_label(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let left: String = text.chars().take(limit.saturating_sub(7)).collect();
    let right: String = text
        .chars()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{left}…{right}")
}

pub(super) fn divider(ui: &mut Ui) {
    let (_, r) = ui.allocate_space(Vec2::new(8., CONTROL_HEIGHT));
    let x = r.center().x;
    ui.painter().line_segment(
        [egui::pos2(x, r.min.y + 4.), egui::pos2(x, r.max.y - 4.)],
        Stroke::new(1.0_f32, LINE),
    );
}

// The egui picker converts RGB through HSV even when idle. Keep authored values
// exact until a user interaction reports a change, so merely opening an inspector
// never produces a pending edit.
pub(super) fn color(ui: &mut Ui, value: &mut [f32; 3]) -> Response {
    let mut draft = *value;
    let response = ui.color_edit_button_rgb(&mut draft);
    if response.changed() {
        *value = draft;
    }
    response
}

#[cfg(test)]
mod tests {
    fn pass(
        ctx: &egui::Context,
        value: &mut f32,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
    ) -> (egui::Rect, bool) {
        let mut result = None;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400., 200.),
                )),
                events,
                modifiers,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let r = ui.add_sized([280., 26.], super::Scrub::new(value).speed(0.1));
                    result = Some((r.rect, r.changed()));
                });
            },
        );
        result.unwrap()
    }
    fn button(pos: egui::Pos2, pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers,
        }
    }
    #[test]
    fn numeric_fields_scrub_horizontally_with_fine_control_and_no_vertical_edits() {
        for (delta, fine, expected) in [
            (egui::vec2(40., 0.), false, 14.),
            (egui::vec2(0., 40.), false, 10.),
            (egui::vec2(40., 40.), false, 14.),
            (egui::vec2(40., 0.), true, 10.4),
            (egui::vec2(-40., 0.), false, 6.),
        ] {
            let ctx = egui::Context::default();
            super::configure(&ctx);
            let mut value = 10.;
            let modifiers = egui::Modifiers {
                shift: fine,
                ..Default::default()
            };
            let (r, _) = pass(&ctx, &mut value, vec![], modifiers);
            assert!(r.width() >= 270., "the whole field must be draggable");
            let start = r.center();
            pass(
                &ctx,
                &mut value,
                vec![
                    egui::Event::PointerMoved(start),
                    button(start, true, modifiers),
                ],
                modifiers,
            );
            let (_, changed) = pass(
                &ctx,
                &mut value,
                vec![egui::Event::PointerMoved(start + delta)],
                modifiers,
            );
            assert_eq!(
                changed,
                delta.x != 0.,
                "vertical movement must not publish false changes"
            );
            pass(
                &ctx,
                &mut value,
                vec![button(start + delta, false, modifiers)],
                modifiers,
            );
            assert!(
                (value - expected).abs() < 1e-5,
                "{delta:?}/fine={fine}: {value} != {expected}"
            );
        }
    }
    #[test]
    fn numeric_drag_rebases_when_an_external_edit_changes_the_value() {
        let ctx = egui::Context::default();
        super::configure(&ctx);
        let mut value = 10.;
        let start = pass(&ctx, &mut value, vec![], Default::default())
            .0
            .center();
        pass(
            &ctx,
            &mut value,
            vec![
                egui::Event::PointerMoved(start),
                button(start, true, Default::default()),
            ],
            Default::default(),
        );
        pass(
            &ctx,
            &mut value,
            vec![egui::Event::PointerMoved(start + egui::vec2(20., 0.))],
            Default::default(),
        );
        assert_eq!(value, 12.);
        value = 30.; // A shared command or undo changed the displayed draft.
        pass(
            &ctx,
            &mut value,
            vec![egui::Event::PointerMoved(start + egui::vec2(40., 0.))],
            Default::default(),
        );
        assert_eq!(
            value, 32.,
            "a stale drag cache must not overwrite the external edit"
        );
    }
    #[test]
    fn numeric_field_retains_text_entry_keyboard_and_idle_precision() {
        let ctx = egui::Context::default();
        super::configure(&ctx);
        let mut value = 0.1234567;
        let (r, changed) = pass(&ctx, &mut value, vec![], Default::default());
        assert!(!changed);
        assert_eq!(value, 0.1234567);
        pass(
            &ctx,
            &mut value,
            vec![
                egui::Event::PointerMoved(r.center()),
                button(r.center(), true, Default::default()),
                button(r.center(), false, Default::default()),
            ],
            Default::default(),
        );
        pass(
            &ctx,
            &mut value,
            vec![egui::Event::Text("12.34567".into())],
            Default::default(),
        );
        assert_eq!(value, 0.1234567, "typing is a draft until confirmation");
        pass(
            &ctx,
            &mut value,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            Default::default(),
        );
        assert_eq!(value, 12.34567);
        assert!(!pass(&ctx, &mut value, vec![], Default::default()).1);
    }
    #[test]
    fn inspecting_color_does_not_create_a_pending_edit() {
        let ctx = egui::Context::default();
        super::configure(&ctx);
        let original = [0.68, 0.72, 0.78];
        let mut color = original;
        for _ in 0..3 {
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    assert!(!super::color(ui, &mut color).changed());
                });
            });
            assert_eq!(color, original);
        }
    }
}
