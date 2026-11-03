//! Offscreen UI interaction and layout check, without touching a desktop session.
//! Run: cargo run --locked --release --example editor_workflows_probe
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
        match service.execute(command.into()) {
            Ok(reply) => g.command_result(reply.data),
            Err(error) => {
                g.command_error(format!("{}: {}", error.code, error.message));
                return Err(error.into());
            }
        }
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
    let root = std::env::current_dir()?.join("artifacts/editor-workflows");
    std::fs::create_dir_all(&root)?;
    let scratch = tempfile::tempdir()?;
    let mut s = Service::new(
        scratch.path(),
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()?,
    )?;
    let mut g = Graphics::offscreen()?;
    let size = [1440, 900];
    let mut labels = vec![];
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    area_features(&mut g, &mut s, size)?;
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    labels = click(
        &mut g,
        &mut s,
        size,
        label(&labels, "+ NPC / personagem")?.center(),
    )?;
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    let npc = s.engine.selected_id().unwrap().to_owned();
    ensure!(
        s.engine
            .scene
            .entities
            .iter()
            .find(|e| e.id == npc)
            .unwrap()
            .npc
            .is_some(),
        "NPC creation UI failed"
    );
    std::fs::write(root.join("npc.png"), g.capture(&s, size[0], size[1])?)?;
    labels = click(&mut g, &mut s, size, label(&labels, "+ Alvo")?.center())?;
    for _ in 0..2 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    ensure!(
        s.engine
            .scene
            .entities
            .iter()
            .find(|e| e.id == npc)
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .target
            .as_deref()
            == Some("player"),
        "NPC target did not apply live"
    );
    s.execute(
        Command::CreateEditorMesh {
            primitive: Primitive::Sphere,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )?;
    let instance = s.selected_mesh.clone().unwrap();
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
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    labels = click(&mut g, &mut s, size, label(&labels, "Rig")?.center())?;
    if label(&labels, "Criar esqueleto").is_err() {
        std::fs::write(root.join("rig-mode.png"), g.capture(&s, size[0], size[1])?)?;
        std::fs::write(root.join("rig-mode-labels.txt"), format!("{labels:#?}"))?;
    }
    labels = click(
        &mut g,
        &mut s,
        size,
        label(&labels, "Criar esqueleto")?.center(),
    )?;
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    ensure!(
        s.engine.scene.meshes.asset(&instance)?.rig.is_some(),
        "Rig creation UI failed"
    );
    labels = click(
        &mut g,
        &mut s,
        size,
        label(&labels, "+ Osso filho")?.center(),
    )?;
    for _ in 0..2 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    ensure!(
        s.engine
            .scene
            .meshes
            .asset(&instance)?
            .rig
            .as_ref()
            .unwrap()
            .bones
            .len()
            == 2,
        "Bone UI failed"
    );
    std::fs::write(root.join("rig.png"), g.capture(&s, size[0], size[1])?)?;
    let edit_mode = labels
        .iter()
        .find(|(t, r)| t == "Editar" && r.top() > 50.)
        .context("Viewport editing mode absent")?
        .1
        .center();
    labels = click(&mut g, &mut s, size, edit_mode)?;
    for _ in 0..2 {
        frame(&mut g, &mut s, size, vec![])?;
    }
    let center = g.viewport.unwrap().center();
    click(&mut g, &mut s, size, center)?;
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    std::fs::write(
        root.join("components.png"),
        g.capture(&s, size[0], size[1])?,
    )?;
    ensure!(
        s.engine.selected_mesh_components().is_some(),
        "Viewport vertex selection absent"
    );
    let before = s.engine.scene.clone();
    let undo = s.engine.snapshot()["history"]["undo"].as_u64().unwrap();
    let field = labels
        .iter()
        .find(|(t, r)| t.starts_with("X  ") && r.left() > g.viewport.unwrap().right())
        .context("Vertex X field missing")?
        .1
        .center();
    drag(&mut g, &mut s, size, field, Vec2::new(8., 0.), false)?;
    ensure!(s.engine.scene != before, "Vertex drag did not apply live");
    ensure!(
        s.engine.snapshot()["history"]["undo"].as_u64() == Some(undo + 1),
        "Drag polluted history"
    );
    std::fs::write(root.join("vertices.png"), g.capture(&s, size[0], size[1])?)?;
    s.execute(Command::Undo.into())?;
    ensure!(s.engine.scene == before, "Vertex undo failed");
    for _ in 0..3 {
        frame(&mut g, &mut s, size, vec![])?;
    }
    // The selection regression also runs after a revision change caused by undo.
    // The original vertex edit is already proven above.
    selection_features(&mut g, &mut s, size, &root)?;
    numeric_component_features(&mut g, &mut s, size, &root)?;
    labels = frame(&mut g, &mut s, size, vec![])?;
    labels = click(
        &mut g,
        &mut s,
        size,
        label(&labels, "Ferramentas…")?.center(),
    )?;
    for _ in 0..3 {
        labels = frame(&mut g, &mut s, size, vec![])?;
    }
    ensure!(
        label(&labels, "Executar operação").is_ok(),
        "Typed tool form absent"
    );
    std::fs::write(root.join("tools.png"), g.capture(&s, size[0], size[1])?)?;
    println!(
        "{{\"passed\":true,\"npc_creation_and_live_target\":true,\"rig_and_child_bone\":true,\"vertex_picking_and_live_drag_undo\":true,\"typed_catalog\":true,\"area_icons\":true,\"rectangle_xray_clear_shift\":true,\"numeric_47_vertices_reject_recover_xyz\":true,\"native_window\":false}}"
    );
    Ok(())
}

fn numeric_component_features(
    g: &mut Graphics,
    s: &mut Service,
    size: [u32; 2],
    root: &std::path::Path,
) -> Result<()> {
    let asset = s.selected_mesh.clone().context("Mesh absent")?;
    s.execute(
        Command::SelectMeshComponents {
            selection: Some(ignis::mesh_topology::Selection {
                asset: asset.clone(),
                reference: s.engine.mesh_reference(),
                components: ignis::mesh_topology::Components::Vertices {
                    indices: (0..47).collect(),
                },
            }),
        }
        .into(),
    )?;
    let viewport_right = g.viewport.unwrap().right();
    let field = |labels: &[(String, Rect)], axis: &str| -> Result<Pos2> {
        labels
            .iter()
            .find(|(t, r)| t.starts_with(axis) && r.left() > viewport_right)
            .map(|(_, r)| r.center())
            .context("Selection coordinate absent")
    };
    let mut labels = vec![];
    for _ in 0..3 {
        labels = frame(g, s, size, vec![])?;
    }
    let start = field(&labels, "X  ")?;
    let before = s.engine.snapshot();
    let rejected = drag(g, s, size, start, Vec2::new(300., 0.), false).unwrap_err();
    ensure!(
        rejected.to_string().contains("reverses"),
        "Unexpected error: {rejected}"
    );
    ensure!(
        s.engine.snapshot() == before,
        "Rejected transform mutated authoring or history"
    );
    labels = frame(
        g,
        s,
        size,
        vec![Event::PointerButton {
            pos: start + Vec2::new(300., 0.),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    )?;
    ensure!(
        label(&labels, "Vértices selecionados: 47").is_ok(),
        "Rejected edit hid selection controls"
    );
    field(&labels, "X  ")?;
    ensure!(
        labels.iter().any(|(t, _)| t == "Movimento não aplicado"),
        "Native command error absent beside fields"
    );
    std::fs::write(
        root.join("component-edit-rejected.png"),
        g.capture(s, size[0], size[1])?,
    )?;
    for _ in 0..3 {
        frame(g, s, size, vec![])?;
    }
    ensure!(
        s.engine.snapshot() == before,
        "Idle retried a rejected edit"
    );
    for (index, axis) in ["X  ", "Y  ", "Z  "].into_iter().enumerate() {
        labels = frame(g, s, size, vec![])?;
        let start = field(&labels, axis)?;
        let old = s
            .engine
            .scene
            .meshes
            .asset(&asset)?
            .geometry
            .positions
            .clone();
        let history = s.engine.snapshot()["history"]["undo"].as_u64().unwrap();
        frame(
            g,
            s,
            size,
            vec![
                Event::PointerMoved(start),
                Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        )?;
        for dx in [7., 8., 9., 10.] {
            labels = frame(
                g,
                s,
                size,
                vec![Event::PointerMoved(start + Vec2::new(dx, 0.))],
            )?;
            label(&labels, "Vértices selecionados: 47")?;
        }
        frame(
            g,
            s,
            size,
            vec![Event::PointerButton {
                pos: start + Vec2::new(10., 0.),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        )?;
        for _ in 0..3 {
            frame(g, s, size, vec![])?;
        }
        let new = &s.engine.scene.meshes.asset(&asset)?.geometry.positions;
        ensure!(
            (new[0][index] - old[0][index] - 0.01).abs() < 1e-5,
            "{axis} numeric value and geometry disagree: old={} new={} delta={}",
            old[0][index],
            new[0][index],
            new[0][index] - old[0][index]
        );
        ensure!(new[47..] == old[47..], "Unselected vertices moved");
        ensure!(
            s.engine.snapshot()["history"]["undo"].as_u64() == Some(history + 1),
            "Continuous drag created more than one undo"
        );
        s.execute(Command::Undo.into())?;
        ensure!(
            s.engine.scene.meshes.asset(&asset)?.geometry.positions == old,
            "Undo is not exact"
        );
        s.execute(
            Command::SelectMeshComponents {
                selection: Some(ignis::mesh_topology::Selection {
                    asset: asset.clone(),
                    reference: s.engine.mesh_reference(),
                    components: ignis::mesh_topology::Components::Vertices {
                        indices: (0..47).collect(),
                    },
                }),
            }
            .into(),
        )?;
    }
    Ok(())
}

fn area_features(g: &mut Graphics, s: &mut Service, size: [u32; 2]) -> Result<()> {
    use ignis::editor::{Area, Axis, Panel};
    fn count(area: &Area, axis: Axis) -> usize {
        match area {
            Area::Split {
                axis: a,
                first,
                second,
                ..
            } => usize::from(*a == axis) + count(first, axis) + count(second, axis),
            _ => 0,
        }
    }
    let scene = s.engine.scene.clone();
    let area = s
        .editor
        .current()
        .layout
        .panel_area(Panel::Viewport)
        .unwrap();
    let header = |g: &Graphics, offset: f32| {
        let v = g.viewport.unwrap();
        Pos2::new(v.right() - offset, v.top() - 16.)
    };
    click(g, s, size, header(g, 84.))?;
    ensure!(
        s.editor.current().maximized == Some(area),
        "Maximize icon failed"
    );
    click(g, s, size, header(g, 84.))?;
    ensure!(
        s.editor.current().maximized.is_none(),
        "Restore icon failed"
    );
    for (axis, offset) in [(Axis::Horizontal, 62.), (Axis::Vertical, 40.)] {
        let before = count(&s.editor.current().layout, axis);
        let top = g.viewport.unwrap().top();
        let labels = click(g, s, size, header(g, offset))?;
        let target = labels
            .iter()
            .find(|(t, r)| t == "Projeto" && r.top() >= top && r.top() < top + 100.)
            .context("Split panel choice absent")?
            .1
            .center();
        click(g, s, size, target)?;
        ensure!(
            count(&s.editor.current().layout, axis) == before + 1,
            "Split icon used the wrong axis"
        );
        s.execute(
            Command::EditEditor {
                action: Action::Restore,
                expected_editor_revision: None,
            }
            .into(),
        )?;
        for _ in 0..3 {
            frame(g, s, size, vec![])?;
        }
    }
    let top = g.viewport.unwrap().top();
    let bounds = g.viewport.unwrap();
    let labels = click(g, s, size, header(g, 18.))?;
    let target = labels
        .iter()
        .find(|(t, r)| t == "Templates" && r.top() >= top && r.top() < top + 200.)
        .context("Open tab choice absent")?
        .1
        .center();
    click(g, s, size, target)?;
    ensure!(
        matches!(
            s.editor.current().layout.find(area),
            Some(Area::Tabs {
                active: Panel::Templates,
                ..
            })
        ),
        "Open tab icon failed"
    );
    click(
        g,
        s,
        size,
        Pos2::new(bounds.right() - 18., bounds.top() - 16.),
    )?;
    ensure!(
        matches!(
            s.editor.current().layout.find(area),
            Some(Area::Tabs {
                active: Panel::Viewport,
                ..
            })
        ),
        "Close tab icon failed"
    );
    s.execute(
        Command::EditEditor {
            action: Action::Restore,
            expected_editor_revision: None,
        }
        .into(),
    )?;
    for _ in 0..3 {
        frame(g, s, size, vec![])?;
    }
    ensure!(s.engine.scene == scene, "Area icons changed authoring");
    Ok(())
}
fn selection_features(
    g: &mut Graphics,
    s: &mut Service,
    size: [u32; 2],
    root: &std::path::Path,
) -> Result<()> {
    let source = s.engine.scene.clone();
    let history = s.engine.snapshot()["history"].clone();
    let count = |s: &Service| {
        s.engine
            .selected_mesh_components()
            .map(|s| ignis::mesh_screen::count(&s.components))
            .unwrap_or(0)
    };
    let view = g.viewport.unwrap();
    let start = view.center() - Vec2::new(60., 50.);
    let end = view.center() + Vec2::new(60., 50.);
    drag(g, s, size, start, end - start, false)?;
    let front = count(s);
    ensure!(front > 1, "Rectangle did not select multiple vertices");
    let empty = view.left_top() + Vec2::splat(10.);
    let shift = egui::Modifiers {
        shift: true,
        ..Default::default()
    };
    for pressed in [true, false] {
        frame_mod(
            g,
            s,
            size,
            vec![
                Event::PointerMoved(empty),
                Event::PointerButton {
                    pos: empty,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: shift,
                },
            ],
            shift,
        )?;
    }
    ensure!(count(s) == front, "Shift+empty click cleared selection");
    click(g, s, size, empty)?;
    frame(g, s, size, vec![])?;
    ensure!(
        count(s) == 0,
        "Empty click did not clear or inspector resurrected selection"
    );
    let start = view.left_top() + Vec2::splat(5.);
    let end = view.right_bottom() - Vec2::splat(5.);
    drag(g, s, size, start, end - start, false)?;
    let surface = count(s);
    ensure!(surface > front, "Full surface rectangle failed");
    std::fs::write(
        root.join("selection-surface.png"),
        g.capture(s, size[0], size[1])?,
    )?;
    let labels = frame(g, s, size, vec![])?;
    let labels = click(g, s, size, label(&labels, "Vista")?.center())?;
    click(g, s, size, label(&labels, "Raio X da malha")?.center())?;
    ensure!(
        s.mesh_view.xray,
        "X-ray checkbox did not use shared command"
    );
    for pressed in [true, false] {
        frame(
            g,
            s,
            size,
            vec![Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            }],
        )?;
    }
    drag(g, s, size, start, end - start, false)?;
    ensure!(
        count(s) > surface,
        "X-ray rectangle omitted hidden vertices"
    );
    let all = count(s);
    drag(g, s, size, start, end - start, true)?;
    ensure!(count(s) == all, "Shift rectangle duplicated components");
    std::fs::write(
        root.join("selection-xray.png"),
        g.capture(s, size[0], size[1])?,
    )?;
    click(g, s, size, empty)?;
    frame(g, s, size, vec![])?;
    ensure!(count(s) == 0, "Empty clear failed in X-ray");
    ensure!(
        s.engine.scene == source && s.engine.snapshot()["history"] == history,
        "Selection changed the scene/history"
    );
    Ok(())
}
