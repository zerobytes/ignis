use super::*;
use egui::{Color32, Rect, Response, RichText, Stroke, Vec2};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    Text(String),
    Number(f32),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Activated {
        node: String,
        action: String,
    },
    Changed {
        node: String,
        binding: String,
        value: Value,
    },
}
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct Capture {
    pub pointer: bool,
    pub keyboard: bool,
}
impl Capture {
    /// Read after composing every UI layer and before routing input to gameplay.
    /// Clear held gameplay inputs when capture starts or the window loses focus.
    pub fn read(ctx: &egui::Context, modal_open: bool) -> Self {
        Self {
            pointer: modal_open || ctx.wants_pointer_input(),
            keyboard: modal_open || ctx.wants_keyboard_input(),
        }
    }
}
#[derive(Default)]
pub struct Frame {
    pub events: Vec<Event>,
    pub hits: BTreeMap<String, Response>,
    pub viewport: Option<Rect>,
}
/// Consumer data only: never serialized into an authored document or undo stack.
pub struct Runtime {
    instance: String,
    locale: String,
    data: BTreeMap<String, Value>,
}
impl Runtime {
    pub fn new(instance: &str) -> Result<Self> {
        key(instance)?;
        Ok(Self {
            instance: instance.into(),
            locale: "en".into(),
            data: BTreeMap::new(),
        })
    }
    pub fn set_locale(&mut self, locale: &str) -> Result<()> {
        key(locale)?;
        self.locale = locale.into();
        Ok(())
    }
    pub fn data(&self) -> &BTreeMap<String, Value> {
        &self.data
    }
    pub fn set_value(&mut self, binding: &str, value: Value) -> Result<()> {
        key(binding)?;
        if !self.data.contains_key(binding) && self.data.len() >= 128 {
            return Err(invalid("UI runtime exceeds128 bindings"));
        }
        match &value {
            Value::Text(s) if s.len() > MAX_TEXT => {
                return Err(invalid("UI runtime text exceeds4096 bytes"));
            }
            Value::Number(n) if !n.is_finite() => return Err(invalid("UI data must be finite")),
            _ => {}
        }
        self.data.insert(binding.into(), value);
        Ok(())
    }
    fn text(&self, d: &Document, t: &Text) -> String {
        match t {
            Text::Literal(s) => s.clone(),
            Text::Binding(k) => match self.data.get(k) {
                Some(Value::Text(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                None => format!("[{k}]"),
            },
            Text::Key(k) => d
                .translations
                .get(&self.locale)
                .and_then(|t| t.get(k))
                .or_else(|| d.translations.get("en").and_then(|t| t.get(k)))
                .cloned()
                .unwrap_or_else(|| format!("[{k}]")),
        }
    }
    /// Only accepts published, validated authoring state. egui memory belongs to
    /// (runtime instance, document ID, node ID), independent of text/data/order.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &State,
        texture: Option<egui::TextureId>,
    ) -> Frame {
        let d = state.document();
        let mut frame = Frame::default();
        let instance = self.instance.clone();
        ui.push_id(("ignis_ui", instance, &d.id), |ui| {
            ui.scope(|ui| {
                theme(ui, &d.theme);
                egui::ScrollArea::vertical()
                    .id_salt("document_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.node(ui, d, &d.root, texture, &mut frame);
                    });
            });
        });
        frame
    }
    fn node(
        &mut self,
        ui: &mut egui::Ui,
        d: &Document,
        n: &Node,
        texture: Option<egui::TextureId>,
        frame: &mut Frame,
    ) {
        if !n.visible {
            return;
        }
        ui.scope_builder(
            egui::UiBuilder::new().id(("ignis_ui_node", &self.instance, &d.id, &n.id)),
            |ui| {
                ui.add_enabled_ui(n.enabled, |ui| {
                    if let Some(w) = n.layout.max_width {
                        ui.set_max_width(ui.available_width().min(w));
                    }
                    ui.set_min_height(n.layout.min_height);
                    let padding = n
                        .layout
                        .padding
                        .min((ui.available_width() - 1.).max(0.) / 2.);
                    let r = egui::Frame::new().inner_margin(padding).show(ui, |ui| {
                        let width = ui.available_width().max(1.);
                        let text = self.text(d, &n.text);
                        match n.kind {
                            Kind::Column => {
                                ui.vertical(|ui| {
                                    for c in &n.children {
                                        self.node(ui, d, c, texture, frame);
                                    }
                                });
                                None
                            }
                            Kind::Row => {
                                ui.horizontal_wrapped(|ui| {
                                    for c in &n.children {
                                        if !c.visible {
                                            continue;
                                        }
                                        // Preserve a child's preferred width before laying it
                                        // out. Shrinking it into the remaining row space can
                                        // turn a button caption into a narrow vertical strip.
                                        if let Some(width) = c.layout.max_width
                                            && ui.cursor().left() > ui.max_rect().left() + 1.
                                            && ui.available_rect_before_wrap().width()
                                                < width.min(ui.max_rect().width())
                                        {
                                            ui.end_row();
                                        }
                                        self.node(ui, d, c, texture, frame);
                                    }
                                });
                                None
                            }
                            Kind::Label => Some(ui.add(egui::Label::new(text).wrap())),
                            Kind::Heading => Some(
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(text).size(d.theme.font_size + 10.).strong(),
                                    )
                                    .wrap(),
                                ),
                            ),
                            Kind::Button => {
                                let r = ui.add(egui::Button::new(text).wrap().min_size(Vec2::new(
                                    width.min(180.),
                                    n.layout.min_height.max(36.),
                                )));
                                if r.clicked() {
                                    frame.events.push(Event::Activated {
                                        node: n.id.clone(),
                                        action: n.action.clone().unwrap(),
                                    });
                                }
                                Some(r)
                            }
                            Kind::TextInput => {
                                let binding = n.binding.as_ref().unwrap();
                                let mut value = match self.data.get(binding) {
                                    Some(Value::Text(s)) => s.clone(),
                                    _ => String::new(),
                                };
                                let r = ui.add(
                                    egui::TextEdit::singleline(&mut value)
                                        .id_salt("input")
                                        .hint_text(text)
                                        .desired_width(width)
                                        .char_limit(MAX_TEXT),
                                );
                                if r.changed()
                                    && value.len() <= MAX_TEXT
                                    && self.set_value(binding, Value::Text(value.clone())).is_ok()
                                {
                                    frame.events.push(Event::Changed {
                                        node: n.id.clone(),
                                        binding: binding.clone(),
                                        value: Value::Text(value),
                                    });
                                }
                                Some(r)
                            }
                            Kind::Progress => {
                                let value = match self.data.get(n.binding.as_ref().unwrap()) {
                                    Some(Value::Number(n)) => n.clamp(0., 1.),
                                    _ => 0.,
                                };
                                Some(
                                    ui.add(
                                        egui::ProgressBar::new(value)
                                            .desired_width(width)
                                            .text(text),
                                    ),
                                )
                            }
                            Kind::Viewport => {
                                let (rect, r) = ui.allocate_exact_size(
                                    Vec2::new(width, (width / 1.8).max(n.layout.min_height)),
                                    egui::Sense::hover(),
                                );
                                if let Some(t) = texture.filter(|_| ui.is_rect_visible(rect)) {
                                    ui.painter().image(
                                        t,
                                        rect,
                                        Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                                        Color32::WHITE,
                                    );
                                    frame.viewport = Some(rect);
                                } else {
                                    ui.painter().rect_filled(rect, 4., color(d.theme.surface));
                                }
                                Some(r)
                            }
                        }
                    });
                    let response = r.inner.unwrap_or(r.response);
                    if response.has_focus() {
                        ui.painter().rect_stroke(
                            response.rect.expand(2.),
                            d.theme.rounding,
                            Stroke::new(2.0_f32, color(d.theme.focus)),
                            egui::StrokeKind::Inside,
                        );
                    }
                    frame.hits.insert(n.id.clone(), response);
                });
            },
        );
    }
}
pub fn color(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}
fn theme(ui: &mut egui::Ui, t: &Theme) {
    let s = ui.style_mut();
    s.visuals = egui::Visuals::dark();
    s.override_font_id = Some(egui::FontId::proportional(t.font_size));
    s.spacing.item_spacing = Vec2::splat(t.spacing);
    s.spacing.button_padding = Vec2::new(14., 9.);
    s.visuals.override_text_color = Some(color(t.text));
    s.visuals.panel_fill = color(t.background);
    s.visuals.extreme_bg_color = color(t.surface);
    s.visuals.selection.bg_fill = color(t.pressed);
    s.visuals.selection.stroke = Stroke::new(1.0_f32, color(t.focus));
    s.visuals.widgets.inactive.bg_fill = color(t.normal);
    s.visuals.widgets.inactive.weak_bg_fill = color(t.normal);
    s.visuals.widgets.hovered.bg_fill = color(t.hover);
    s.visuals.widgets.hovered.weak_bg_fill = color(t.hover);
    s.visuals.widgets.active.bg_fill = color(t.pressed);
    s.visuals.widgets.active.weak_bg_fill = color(t.pressed);
    s.visuals.widgets.noninteractive.bg_fill = color(t.disabled);
    s.visuals.widgets.noninteractive.weak_bg_fill = color(t.disabled);
    for w in [
        &mut s.visuals.widgets.inactive,
        &mut s.visuals.widgets.hovered,
        &mut s.visuals.widgets.active,
        &mut s.visuals.widgets.noninteractive,
    ] {
        w.corner_radius = t.rounding.into();
        w.fg_stroke = Stroke::new(1.0_f32, color(t.text));
    }
    s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, color(t.focus));
    s.visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, color(t.focus));
    s.visuals.widgets.noninteractive.fg_stroke =
        Stroke::new(1.0_f32, color(t.text).gamma_multiply(0.4));
    s.visuals.selection.bg_fill = color(t.accent);
}
