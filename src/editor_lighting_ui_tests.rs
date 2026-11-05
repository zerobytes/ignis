use super::*;

struct Harness {
    _tmp: tempfile::TempDir,
    service: Service,
    ctx: egui::Context,
    output: egui::FullOutput,
    frame: u64,
}
impl Harness {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let service = Service::new(
            tmp.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        let ctx = egui::Context::default();
        design_ui::configure(&ctx);
        let mut h = Self {
            _tmp: tmp,
            service,
            ctx,
            output: Default::default(),
            frame: 0,
        };
        h.pass(vec![]);
        let button = h.text("Iluminação").center();
        h.click(button);
        h.idle();
        assert!(h.open());
        h
    }
    fn pass(&mut self, events: Vec<egui::Event>) -> Vec<Command> {
        self.frame += 1;
        let mut commands = vec![];
        self.output = self.ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 2000.),
                )),
                time: Some(self.frame as f64 / 60.),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                commands.clear();
                egui::TopBottomPanel::top("lighting_test_toolbar")
                    .show(ctx, |ui| controls(ui, &self.service, &mut commands));
            },
        );
        for command in &commands {
            self.service.execute(command.clone().into()).unwrap();
        }
        commands
    }
    fn idle(&mut self) {
        for _ in 0..3 {
            self.pass(vec![]);
        }
    }
    fn open(&self) -> bool {
        self.ctx
            .data(|d| d.get_temp::<bool>(egui::Id::new(PANEL_OPEN)))
            .unwrap_or(false)
    }
    fn text(&self, name: &str) -> egui::Rect {
        fn find(shape: &egui::Shape, name: &str) -> Option<egui::Rect> {
            match shape {
                egui::Shape::Text(t) if t.galley.job.text == name => Some(t.visual_bounding_rect()),
                egui::Shape::Vec(v) => v.iter().find_map(|s| find(s, name)),
                _ => None,
            }
        }
        self.output
            .shapes
            .iter()
            .find_map(|s| find(&s.shape, name))
            .unwrap_or_else(|| panic!("Missing lighting label: {name}"))
    }
    fn click(&mut self, p: egui::Pos2) -> Vec<Command> {
        let mut c = self.pass(vec![egui::Event::PointerMoved(p), button(p, true)]);
        c.extend(self.pass(vec![button(p, false)]));
        c
    }
    fn scene_section(&mut self) {
        self.click(self.text("Direção, intensidade e ambiente").center());
        self.idle();
    }
    fn field(&self, label: &str) -> egui::Pos2 {
        let r = self.text(label);
        // The shared two-column layout places the control after its104px label.
        egui::pos2(r.left() + 145., r.center().y)
    }
    fn picker(&self) -> egui::Rect {
        self.ctx
            .memory(|m| {
                m.areas()
                    .visible_layer_ids()
                    .iter()
                    .filter(|l| l.order == egui::Order::Foreground)
                    .find_map(|l| m.area_rect(l.id))
            })
            .expect("Color picker area must stay visible")
    }
}
fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}

#[test]
fn scene_slider_changes_effective_viewport_light_and_native_pixels() {
    let mut h = Harness::new();
    h.scene_section();
    assert!(h.service.editor.work_light.is_some());
    let old = h.service.engine.scene.light.direct_intensity;
    let commands = h.click(h.field("Intensidade direta"));
    h.idle();
    assert_ne!(
        h.service.engine.scene.light.direct_intensity, old,
        "Slider must submit its scene edit"
    );
    assert!(
        h.service.editor.work_light.is_none(),
        "Scene edits must be previewed without the studio override"
    );
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, Command::EditEditor { .. }))
            .count(),
        1,
        "One preview command avoids editor revision conflicts"
    );
    assert!(h.open());
    let mut graphics = crate::editor_ui::Graphics::offscreen().unwrap();
    let _ = graphics.build(
        &h.service,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 900.),
            )),
            ..Default::default()
        },
    );
    assert_eq!(
        graphics.custom_frame_mut().unwrap().light,
        h.service.engine.scene.light
    );
    let frame = h.service.engine.render_frame();
    let before = graphics.renderer.capture(&frame, 320, 240).unwrap();
    let mut light = h.service.engine.scene.light.clone();
    light.direct_intensity = 0.;
    h.service
        .execute(Command::SetLight { light }.into())
        .unwrap();
    let after = graphics
        .renderer
        .capture(&h.service.engine.render_frame(), 320, 240)
        .unwrap();
    assert_ne!(
        before, after,
        "Native illumination must actually change pixels"
    );
}
#[test]
fn color_picker_survives_frames_and_edits_without_closing_lighting() {
    let mut h = Harness::new();
    h.scene_section();
    h.click(h.field("Cor direta (RGB linear)"));
    h.idle();
    assert!(h.open());
    assert!(egui::Popup::is_any_open(&h.ctx));
    let picker = h.picker();
    let r = h
        .output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t)
                if t.galley.job.text.starts_with("R ")
                    && picker.contains(t.visual_bounding_rect().center()) =>
            {
                Some(t.visual_bounding_rect())
            }
            _ => None,
        })
        .expect("Color picker numeric R field");
    let old = h.service.engine.scene.light.direct_color;
    let start = r.center();
    let end = start - egui::vec2(40., 0.);
    h.pass(vec![egui::Event::PointerMoved(start), button(start, true)]);
    h.pass(vec![egui::Event::PointerMoved(end)]);
    h.pass(vec![button(end, false)]);
    h.idle();
    assert_ne!(h.service.engine.scene.light.direct_color, old);
    assert!(h.service.editor.work_light.is_none());
    assert!(
        h.open(),
        "Editing a color cannot dismiss the lighting panel"
    );
    assert!(
        egui::Popup::is_any_open(&h.ctx),
        "Editing must not recreate/dismiss the picker"
    );
    h.click(egui::pos2(1200., 1800.));
    h.idle();
    assert!(!egui::Popup::is_any_open(&h.ctx));
    assert!(h.open(), "Only the picker dismisses on an outside click");
}
#[test]
fn viewport_work_light_controls_remain_separate_from_saved_scene() {
    let mut h = Harness::new();
    let scene = h.service.engine.scene.light.clone();
    let old = h.service.editor.work_light.clone().unwrap();
    let panel = h
        .ctx
        .memory(|m| m.area_rect(egui::Id::new(PANEL_WINDOW)))
        .unwrap();
    h.pass(vec![
        egui::Event::PointerMoved(panel.center()),
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0., -10000.),
            modifiers: Default::default(),
        },
    ]);
    h.idle();
    h.click(h.field("Ambiente"));
    h.idle();
    assert_eq!(h.service.engine.scene.light, scene);
    assert_ne!(
        h.service.editor.work_light.as_ref().unwrap().ambient,
        old.ambient
    );
    assert!(h.open());
}

#[test]
fn disabled_work_light_controls_never_emit_idle_edits() {
    let mut h = Harness::new();
    h.service
        .execute(
            Command::EditEditor {
                action: Action::Lighting { light: None },
                expected_editor_revision: None,
            }
            .into(),
        )
        .unwrap();
    let mut light = h.service.engine.scene.light.clone();
    light.direction = [0., -1., 0.];
    h.service
        .execute(Command::SetLight { light }.into())
        .unwrap();
    let revision = h.service.editor.revision;
    let scene = h.service.engine.scene.clone();
    for _ in 0..4 {
        assert!(h.pass(vec![]).is_empty());
    }
    assert_eq!(h.service.editor.revision, revision);
    assert_eq!(h.service.engine.scene, scene);
}
