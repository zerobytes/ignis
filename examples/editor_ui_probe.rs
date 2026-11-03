//! Offscreen UI interaction and layout check, without touching a desktop session.
//! Run: cargo run --locked --release --example editor_ui_probe
use anyhow::{Context, Result, ensure};
use egui::{Event, Pos2, Rect, Vec2, epaint::Shape};
use ignis::{
    editor::Action, editor_ui::Graphics, engine::Command, mesh_tools::Primitive, model::RoomRecipe,
    service::Service,
};

fn texts(shape: &Shape, out: &mut Vec<(String, Rect)>) {
    match shape {
        Shape::Text(t) => out.push((
            t.galley.text().to_owned(),
            Rect::from_min_size(t.pos, t.galley.size()),
        )),
        Shape::Vec(shapes) => shapes.iter().for_each(|s| texts(s, out)),
        _ => {}
    }
}
fn frame(
    g: &mut Graphics,
    service: &mut Service,
    size: [u32; 2],
    events: Vec<Event>,
) -> Result<Vec<(String, Rect)>> {
    frame_mod(g, service, size, events, Default::default())
}
fn frame_mod(
    g: &mut Graphics,
    service: &mut Service,
    size: [u32; 2],
    events: Vec<Event>,
    modifiers: egui::Modifiers,
) -> Result<Vec<(String, Rect)>> {
    let (output, commands) = g.build(
        service,
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(size[0] as f32, size[1] as f32),
            )),
            time: Some(0.),
            events,
            modifiers,
            ..Default::default()
        },
    );
    let mut labels = vec![];
    for shape in &output.shapes {
        texts(&shape.shape, &mut labels);
    }
    let target = g.renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("UI verification"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: g.renderer.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    g.paint(
        output,
        &target.create_view(&Default::default()),
        size[0],
        size[1],
    )?;
    for command in commands {
        service.execute(command.into())?;
    }
    Ok(labels)
}
fn label(labels: &[(String, Rect)], text: &str) -> Result<Rect> {
    labels
        .iter()
        .find(|(s, _)| s == text)
        .map(|(_, r)| *r)
        .with_context(|| format!("missing label {text}"))
}
fn click(
    g: &mut Graphics,
    s: &mut Service,
    size: [u32; 2],
    pos: Pos2,
) -> Result<Vec<(String, Rect)>> {
    frame(
        g,
        s,
        size,
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    )?;
    frame(
        g,
        s,
        size,
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
    )?;
    frame(g, s, size, vec![])
}
fn drag(
    g: &mut Graphics,
    s: &mut Service,
    size: [u32; 2],
    start: Pos2,
    delta: Vec2,
    fine: bool,
) -> Result<Vec<(String, Rect)>> {
    let modifiers = egui::Modifiers {
        shift: fine,
        ..Default::default()
    };
    frame_mod(
        g,
        s,
        size,
        vec![
            Event::PointerMoved(start),
            Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers,
            },
        ],
        modifiers,
    )?;
    frame_mod(
        g,
        s,
        size,
        vec![Event::PointerMoved(start + delta)],
        modifiers,
    )?;
    frame_mod(
        g,
        s,
        size,
        vec![Event::PointerButton {
            pos: start + delta,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers,
        }],
        modifiers,
    )
}

fn main() -> Result<()> {
    let root = std::env::current_dir()?;
    std::fs::create_dir_all(root.join("artifacts/editor-studio"))?;
    let scratch = tempfile::tempdir()?;
    let mut s = Service::new(
        scratch.path(),
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()?,
    )?;
    s.execute(
        Command::CreateEditorMesh {
            primitive: Primitive::Sphere,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )?;
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [4.5, 3., 5.];
    camera.target = [2., 1.05, 0.];
    s.execute(
        Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    s.execute(
        Command::SaveScene {
            path: "review.scene.json".into(),
        }
        .into(),
    )?;
    let original = s.engine.scene.clone();
    let mut g = Graphics::offscreen()?;
    let size = [1440, 900];
    let mut labels = vec![];
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    labels = click(&mut g, &mut s, size, label(&labels, "Escultura")?.center())?;
    std::fs::write(
        root.join("artifacts/editor-studio/initial.png"),
        g.capture(&s, size[0], size[1])?,
    )?;
    let inspector_object = labels
        .iter()
        .find(|(text, r)| text == "Objeto" && r.left() > g.viewport.unwrap().right())
        .unwrap()
        .1;
    labels = click(&mut g, &mut s, size, inspector_object.center())?;
    std::fs::write(
        root.join("artifacts/editor-studio/object.png"),
        g.capture(&s, size[0], size[1])?,
    )?;
    ensure!(
        label(&labels, "Aplicar propriedades").is_err() && s.engine.scene == original,
        "properties must apply automatically without idle edits"
    );
    labels = click(&mut g, &mut s, size, label(&labels, "Pincel")?.center())?;
    ensure!(
        label(&labels, "Raio").is_ok() && label(&labels, "Força").is_ok(),
        "sculpt controls absent"
    );
    let base_settings = s.editor.sculpt.clone();
    let radius = label(&labels, "0.400 m")?.center();
    labels = drag(&mut g, &mut s, size, radius, Vec2::new(40., 0.), false)?;
    ensure!(
        (s.editor.sculpt.radius - 0.6).abs() < 1e-5,
        "horizontal field drag failed"
    );
    let radius = label(&labels, "0.600 m")?.center();
    labels = drag(&mut g, &mut s, size, radius, Vec2::new(40., 0.), true)?;
    ensure!(
        (s.editor.sculpt.radius - 0.62).abs() < 1e-5,
        "Shift fine field drag failed"
    );
    let revision = s.editor.revision;
    let radius = label(&labels, "0.620 m")?.center();
    labels = drag(&mut g, &mut s, size, radius, Vec2::new(0., 30.), false)?;
    ensure!(
        s.editor.revision == revision,
        "vertical field drag published an edit"
    );
    for (name, brush) in [
        ("Mover", ignis::meshes::BrushKind::Grab),
        ("Suavizar", ignis::meshes::BrushKind::Smooth),
        ("Volume", ignis::meshes::BrushKind::Draw),
    ] {
        let target = label(&labels, name)?.center();
        labels = click(&mut g, &mut s, size, target)?;
        ensure!(
            s.editor.sculpt.brush == brush,
            "brush rail did not select {name}"
        );
        ensure!(
            s.engine.scene == original,
            "tool input sculpted through the rail/inspector"
        );
    }
    s.execute(
        Command::EditEditor {
            action: Action::Sculpt {
                settings: base_settings,
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    labels = frame(&mut g, &mut s, size, vec![])?;
    std::fs::write(
        root.join("artifacts/editor-design-wide.png"),
        g.capture(&s, size[0], size[1])?,
    )?;
    for narrow in [[1000, 760], [800, 650]] {
        for _ in 0..3 {
            labels = frame(&mut g, &mut s, narrow, vec![])?;
        }
        for text in [
            "Escultura",
            "Volume",
            "Mover",
            "Suavizar",
            "Raio",
            "Força",
            "Iluminação",
            "Vista",
        ] {
            let r = label(&labels, text)?;
            ensure!(
                r.left() >= 0. && r.right() <= narrow[0] as f32 && r.bottom() <= narrow[1] as f32,
                "{text} overflows at {}: {r:?}",
                narrow[0]
            );
            let view = g.viewport.unwrap();
            if matches!(text, "Volume" | "Mover" | "Suavizar") {
                ensure!(
                    r.right() <= view.left(),
                    "brush tile is not in the tool rail: {text}"
                );
            } else if matches!(text, "Raio" | "Força") {
                ensure!(
                    r.left() >= view.right(),
                    "parameter is not in the inspector: {text}"
                );
            } else {
                ensure!(
                    r.bottom() <= view.top(),
                    "contextual toolbar overflows: {text}"
                );
            }
        }
        // Compact headers keep modes/options accessible through the mode menu.
        let view = g.viewport.unwrap();
        if labels
            .iter()
            .any(|(t, r)| t == "Objeto" && r.bottom() <= view.top())
        {
            for text in ["Objeto", "Opções"] {
                let r = label(&labels, text)?;
                ensure!(
                    r.bottom() <= view.top() && r.left() >= 0. && r.right() <= narrow[0] as f32,
                    "Inline mode/option clipped: {text}"
                );
            }
        } else {
            let mode = labels
                .iter()
                .find(|(t, r)| t == "Escultura" && r.bottom() <= view.top())
                .context("Compact mode menu absent")?
                .1
                .center();
            labels = click(&mut g, &mut s, narrow, mode)?;
            std::fs::write(
                root.join("artifacts/editor-studio/compact-popup.png"),
                g.capture(&s, narrow[0], narrow[1])?,
            )?;
            std::fs::write(
                root.join("artifacts/editor-studio/compact-popup-labels.txt"),
                format!("mode={mode:?}, view={view:?}\n{labels:#?}"),
            )?;
            for text in ["Objeto", "Rig", "Editar", "Restaurar pincéis"] {
                let r = labels
                    .iter()
                    .find(|(t, r)| t == text && r.left() < view.right() && r.top() > mode.y)
                    .with_context(|| format!("Compact mode option absent: {text}"))?
                    .1;
                ensure!(
                    r.left() >= 0.
                        && r.right() <= narrow[0] as f32
                        && r.bottom() <= narrow[1] as f32,
                    "Compact option overflows: {text}"
                );
            }
            let object = labels
                .iter()
                .find(|(t, r)| t == "Objeto" && r.left() < view.right() && r.top() > mode.y)
                .unwrap()
                .1
                .center();
            labels = click(&mut g, &mut s, narrow, object)?;
            let object_menu = labels
                .iter()
                .find(|(t, r)| t == "Objeto" && r.bottom() <= g.viewport.unwrap().top())
                .context("Object mode did not become active")?
                .1
                .center();
            labels = click(&mut g, &mut s, narrow, object_menu)?;
            let sculpt = labels
                .iter()
                .find(|(t, r)| {
                    t == "Escultura"
                        && r.left() < g.viewport.unwrap().right()
                        && r.top() > object_menu.y
                })
                .context("Sculpt mode option absent")?
                .1
                .center();
            labels = click(&mut g, &mut s, narrow, sculpt)?;
            ensure!(
                label(&labels, "Raio").is_ok(),
                "Compact mode failed to restore sculpt inspector"
            );
        }
        for (text, rect) in &labels {
            if (matches!(text.as_str(), "X" | "Y" | "Z")
                || text.parse::<f32>().is_ok()
                || ["X  ", "Y  ", "Z  "]
                    .iter()
                    .any(|prefix| text.starts_with(prefix)))
                && rect.top() < narrow[1] as f32 - 24.
            {
                ensure!(
                    rect.right() <= narrow[0] as f32,
                    "numeric field {text} clipped at {}: {rect:?}",
                    narrow[0]
                );
            }
        }
        std::fs::write(
            root.join(format!("artifacts/editor-design-{}.png", narrow[0])),
            g.capture(&s, narrow[0], narrow[1])?,
        )?;
    }
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    labels = click(&mut g, &mut s, size, label(&labels, "Iluminação")?.center())?;
    let ambient = label(&labels, "Ambiente")?;
    let before_light = s.editor.work_light.clone();
    // The slider starts just after its aligned label column. The popup must stay
    // open and emit an editor-only command, without sculpting through the form.
    click(
        &mut g,
        &mut s,
        size,
        Pos2::new(ambient.left() + 104. + 100., ambient.center().y),
    )?;
    ensure!(
        s.editor.work_light != before_light,
        "ambient slider did not change light"
    );
    ensure!(s.engine.scene == original, "UI controls edited the scene");
    ensure!(
        egui::Popup::is_any_open(&g.context),
        "slider dismissed its form"
    );
    labels = frame(&mut g, &mut s, size, vec![])?;
    click(
        &mut g,
        &mut s,
        size,
        label(&labels, "Restaurar contraste")?.center(),
    )?;
    std::fs::write(
        root.join("artifacts/editor-design-light.png"),
        g.capture(&s, size[0], size[1])?,
    )?;
    let compact = [900, 600];
    frame(
        &mut g,
        &mut s,
        compact,
        vec![Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    )?;
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, compact, vec![])?;
    }
    labels = click(
        &mut g,
        &mut s,
        compact,
        label(&labels, "Projeto…")?.center(),
    )?;
    labels = click(
        &mut g,
        &mut s,
        compact,
        label(&labels, "Criar projeto…")?.center(),
    )?;
    for text in ["Criar projeto", "Cancelar", "Nome do jogo"] {
        let r = label(&labels, text)?;
        ensure!(
            r.left() >= 0. && r.right() <= 900. && r.top() >= 0. && r.bottom() <= 600.,
            "project control {text} clipped: {r:?}"
        );
    }
    std::fs::write(
        root.join("artifacts/editor-design-project.png"),
        g.capture(&s, compact[0], compact[1])?,
    )?;
    ensure!(
        s.engine.scene == original && s.project.is_none(),
        "project form mutated authored data"
    );
    println!(
        "{{\"passed\":true,\"widths\":[1440,1000,900,800],\"popup_interaction\":true,\"project_form\":true,\"scene_unchanged\":true,\"horizontal_drag\":true,\"shift_fine\":true,\"vertical_no_edit\":true,\"brush_rail\":true,\"desktop_window\":false}}"
    );
    Ok(())
}
