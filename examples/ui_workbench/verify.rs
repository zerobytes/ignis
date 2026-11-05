use super::tool::Tool;
use anyhow::{Result, ensure};
use egui::{Event, Pos2, Rect, Vec2};
use ignis::{
    editor_ui::Graphics,
    game_ui::{Edit, Operation, Text, Value},
};
use std::{path::Path, time::Instant};
struct Harness {
    tool: Tool,
    g: Graphics,
    size: [u32; 2],
    scale: f32,
    time: f64,
}
impl Harness {
    fn new(root: &Path) -> Result<Self> {
        Ok(Self {
            tool: Tool::new(root)?,
            g: Graphics::offscreen()?,
            size: [1280, 860],
            scale: 1.,
            time: 0.,
        })
    }
    fn raw(&mut self, events: Vec<Event>) -> egui::RawInput {
        self.time += 1. / 60.;
        let mut raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(self.size[0] as f32, self.size[1] as f32),
            )),
            events,
            time: Some(self.time),
            focused: true,
            ..Default::default()
        };
        raw.viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(self.scale);
        raw
    }
    fn target(&self) -> wgpu::Texture {
        self.g
            .renderer
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("current UI verification target"),
                size: wgpu::Extent3d {
                    width: (self.size[0] as f32 * self.scale) as u32,
                    height: (self.size[1] as f32 * self.scale) as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.g.renderer.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
    }
    fn frame(&mut self, events: Vec<Event>) -> Result<()> {
        let raw = self.raw(events);
        let out = self.tool.frame(&mut self.g, raw);
        let target = self.target();
        let start = Instant::now();
        self.g.paint(
            out,
            &target.create_view(&Default::default()),
            target.width(),
            target.height(),
        )?;
        self.tool.draw_ms = start.elapsed().as_secs_f64() * 1000.;
        Ok(())
    }
    fn settle(&mut self) -> Result<()> {
        for _ in 0..3 {
            self.frame(vec![])?;
        }
        Ok(())
    }
    fn position(&self, id: &str) -> Result<Pos2> {
        let r = self
            .tool
            .frame
            .hits
            .get(id)
            .or_else(|| self.tool.hits.get(id))
            .ok_or_else(|| anyhow::anyhow!("Missing hit {id}"))?;
        ensure!(r.interact_rect.is_positive(), "Hit {id} is clipped");
        Ok(r.interact_rect.center())
    }
    fn click(&mut self, id: &str) -> Result<()> {
        self.settle()?;
        let pos = self.position(id)?;
        self.frame(vec![Event::PointerMoved(pos)])?;
        for pressed in [true, false] {
            self.frame(vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ])?;
        }
        Ok(())
    }
    fn key(&mut self, key: egui::Key, modifiers: egui::Modifiers) -> Result<()> {
        for pressed in [true, false] {
            self.frame(vec![Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            }])?;
        }
        Ok(())
    }
    fn capture(&mut self, path: &Path) -> Result<serde_json::Value> {
        self.settle()?;
        let raw = self.raw(vec![]);
        let output = self.tool.frame(&mut self.g, raw);
        let target = self.target();
        let ppp = output.pixels_per_point;
        ensure!(
            (ppp - self.scale).abs() < 0.001,
            "DPI output must match requested native scale"
        );
        self.g.paint(
            output,
            &target.create_view(&Default::default()),
            target.width(),
            target.height(),
        )?;
        std::fs::write(
            path,
            self.g
                .renderer
                .read_texture_png(&target, target.width(), target.height())?,
        )?;
        Ok(
            serde_json::json!({"logical_size":self.size,"pixels_per_point":ppp,"physical_size":[target.width(),target.height()],"path":path}),
        )
    }
}
pub fn run(root: &Path) -> Result<()> {
    let output = root.join("artifacts/ui-workbench");
    std::fs::create_dir_all(&output)?;
    let mut h = Harness::new(root)?;
    let scene = serde_json::to_value(&h.tool.service.engine.scene)?;
    let scene_revision = h.tool.service.engine.revision;
    let result = (|| -> Result<()> {
        h.settle()?;
        h.click("name")?;
        ensure!(
            h.tool.frame.hits["name"].has_focus(),
            "Mouse must focus the authored text input"
        );
        ensure!(
            h.tool.capture.keyboard,
            "Text input must capture gameplay keys"
        );
        h.frame(vec![Event::Text("UI independente".into())])?;
        ensure!(
            h.tool.runtime.data().get("name") == Some(&Value::Text("UI independente".into())),
            "Typed text must update consumer data"
        );
        let focus_id = h.tool.frame.hits["name"].id;
        h.tool.command(
            Operation::Edit {
                edits: vec![Edit::Move {
                    id: "name".into(),
                    parent: "actions".into(),
                    index: 0,
                }],
            },
            Some(h.tool.service.game_ui.revision()),
        );
        h.settle()?;
        ensure!(
            h.tool.frame.hits["name"].id == focus_id && h.tool.frame.hits["name"].has_focus(),
            "Reparent/reorder must preserve element focus identity"
        );
        h.tool
            .command(Operation::Undo, Some(h.tool.service.game_ui.revision()));
        h.settle()?;
        h.key(egui::Key::Tab, Default::default())?;
        ensure!(
            h.tool.frame.hits["increment"].has_focus(),
            "Tab order should follow authored hierarchy"
        );
        h.key(egui::Key::Enter, Default::default())?;
        ensure!(
            h.tool.updates == 1,
            "Enter must activate focused authored button exactly once"
        );
        h.click("increment")?;
        ensure!(
            h.tool.updates == 2,
            "Pointer event must update data through consumer event"
        );
        h.click("disabled")?;
        ensure!(
            h.tool.frame.events.is_empty(),
            "Disabled widgets cannot emit an action"
        );
        h.click("modal")?;
        h.settle()?;
        ensure!(
            h.tool.modal && h.tool.capture.keyboard && h.tool.capture.pointer,
            "Modal must capture input"
        );
        h.key(egui::Key::Tab, Default::default())?;
        ensure!(
            h.g.context.memory(|m| m.focused()) == Some(h.tool.hits["close_modal"].id),
            "Focus must stay in modal"
        );
        let modal = h.capture(&output.join("modal.png"))?;
        let count = h.tool.updates;
        h.click("increment")?;
        ensure!(
            h.tool.updates == count,
            "Modal cannot leak activation to background"
        );
        // Outside click may dismiss an egui modal, but must never activate its backdrop.
        if !h.tool.modal {
            h.click("modal")?;
            h.settle()?;
        }
        h.key(egui::Key::Escape, Default::default())?;
        ensure!(!h.tool.modal, "Escape should close modal");
        // Edit through the actual manual property widget and apply dispatcher.
        h.click("property_text")?;
        let all = egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        };
        h.key(egui::Key::A, all)?;
        h.frame(vec![Event::Text("subtitle".into())])?;
        h.click("apply")?;
        ensure!(h.tool.error.is_empty(), "{}", h.tool.error);
        ensure!(
            h.tool
                .service
                .game_ui
                .document()
                .root
                .find("title")
                .unwrap()
                .text
                == Text::Key("subtitle".into()),
            "Manual property application must publish to service"
        );
        h.click("undo")?;
        ensure!(
            h.tool
                .service
                .game_ui
                .document()
                .root
                .find("title")
                .unwrap()
                .text
                == Text::Key("title".into()),
            "Manual undo must restore UI document"
        );
        h.click("save")?;
        let saved = h.tool.service.game_ui.document().clone();
        let rev = h.tool.service.game_ui.revision();
        h.tool.command(
            Operation::Edit {
                edits: vec![Edit::Remove {
                    id: "status".into(),
                }],
            },
            Some(rev),
        );
        h.click("load")?;
        ensure!(
            h.tool.service.game_ui.document() == &saved,
            "Manual save/load must round trip"
        );
        let wide = h.capture(&output.join("wide.png"))?;
        h.size = [820, 960];
        h.scale = 1.5;
        h.tool.language(true)?;
        let narrow = h.capture(&output.join("narrow.png"))?;
        let disabled = &h.tool.frame.hits["disabled"];
        let modal_button = &h.tool.frame.hits["modal"];
        ensure!(
            disabled.rect.top() > modal_button.rect.top() && disabled.rect.width() >= 170.,
            "Narrow rows must wrap preferred-width buttons instead of squeezing captions"
        );
        let name = &h.tool.frame.hits["name"];
        ensure!(
            name.rect.width() <= h.size[0] as f32 && name.interact_rect.is_positive(),
            "Input must fit the resized scroll viewport"
        );
        // Measure CPU composition and UI-only paint separately from the 3D renderer.
        let target = h.target();
        let mut layout = vec![];
        let mut ui_paint = vec![];
        for _ in 0..12 {
            let raw = h.raw(vec![]);
            let out = h.tool.frame(&mut h.g, raw);
            layout.push(h.tool.layout_ms);
            h.g.viewport = None;
            let start = Instant::now();
            h.g.paint(
                out,
                &target.create_view(&Default::default()),
                target.width(),
                target.height(),
            )?;
            ui_paint.push(start.elapsed().as_secs_f64() * 1000.);
        }
        ensure!(
            scene == serde_json::to_value(&h.tool.service.engine.scene)?
                && scene_revision == h.tool.service.engine.revision
                && h.tool.service.engine.simulation.tick == 0,
            "UI work must preserve the 3D world and gameplay clock"
        );
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let report = serde_json::json!({"verified":true,"harness":"same egui/Graphics/runtime with pointer, text and keyboard events; offscreen native GPU","checks":["pointer focus and typed data","stable focus after hierarchy edit","Tab/Enter order and single activation","disabled actions","modal capture, focus and no background activation","Escape close","manual property dispatcher + undo","manual save/load","resize and native DPI","unchanged scene/runtime"],"captures":[wide,narrow,modal],"cpu_samples":12,"ui_composition_mean_ms":mean(&layout),"ui_only_paint_mean_ms":mean(&ui_paint),"timing_limit":"CPU wall time; painting includes tessellation/upload/submit. 3D disabled in timing sample. No GPU timestamp/FPS claim.","adapter":format!("{:?}",h.g.renderer.adapter.get_info()),"desktop_interaction":false,"human_acceptance":false});
        std::fs::write(
            output.join("verification.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        Ok(())
    })();
    if result.is_err() {
        let _ = h.capture(&output.join("failure.png"));
    } else if output.join("failure.png").exists() {
        std::fs::remove_file(output.join("failure.png"))?;
    }
    result
}
