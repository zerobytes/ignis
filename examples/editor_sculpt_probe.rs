//! Exercise real editor pointer input and capture the resulting raised stroke.
//! All state and PNGs are isolated; no desktop window or user scene is touched.
use anyhow::{Result, ensure};
use egui::{Event, Pos2, Rect, Vec2, epaint::Shape};
use glam::Vec3;
use ignis::{
    editor::Action, editor_ui::Graphics, engine::Command, mesh_tools::Primitive, model::RoomRecipe,
    service::Service,
};

fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}
fn find_text(shape: &Shape, text: &str) -> Option<Pos2> {
    match shape {
        Shape::Text(t) if t.galley.text() == text => Some(t.pos + t.galley.size() * 0.5),
        Shape::Vec(shapes) => shapes.iter().find_map(|s| find_text(s, text)),
        _ => None,
    }
}
fn paint(g: &mut Graphics, output: egui::FullOutput) -> Result<()> {
    let target = g.renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sculpt pointer verification"),
        size: wgpu::Extent3d {
            width: 1440,
            height: 900,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: g.renderer.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    g.paint(output, &target.create_view(&Default::default()), 1440, 900)
}
fn frame(g: &mut Graphics, s: &mut Service, events: Vec<Event>) -> Result<Option<Pos2>> {
    let (output, commands) = g.build(
        s,
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
            events,
            focused: true,
            ..Default::default()
        },
    );
    let sculpt = output
        .shapes
        .iter()
        .find_map(|s| find_text(&s.shape, "Escultura"));
    paint(g, output)?;
    for c in commands {
        s.execute(c.into())?;
    }
    Ok(sculpt)
}
fn main() -> Result<()> {
    let root = std::env::current_dir()?.join("artifacts/editor-sculpt-quality");
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
    s.execute(
        Command::CreateEditorMesh {
            primitive: Primitive::Sphere,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )?;
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [2., 1.05, 6.];
    camera.target = [2., 1.05, 0.];
    camera.fov_degrees = 48.;
    s.execute(
        Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera.clone()),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    s.execute(
        Command::SaveScene {
            path: "probe.scene.json".into(),
        }
        .into(),
    )?;
    // Keep the established fixed-topology brush regressions, then exercise detailed Volume separately.
    let mut legacy = s.editor.sculpt.clone();
    legacy.adaptive_detail = false;
    s.execute(
        Command::EditEditor {
            action: Action::Sculpt { settings: legacy },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let original = s.engine.scene.clone();
    let mut g = Graphics::offscreen()?;
    let mut tab = None;
    for _ in 0..3 {
        tab = frame(&mut g, &mut s, vec![])?;
    }
    let tab = tab.expect("sculpt mode tab");
    frame(
        &mut g,
        &mut s,
        vec![
            Event::PointerMoved(tab),
            button(tab, true),
            button(tab, false),
        ],
    )?;
    frame(&mut g, &mut s, vec![])?;
    let view = g.viewport.expect("viewport");
    let start = view.center() - Vec2::new(110., 0.);
    let mut command = None;
    frame(
        &mut g,
        &mut s,
        vec![Event::PointerMoved(start), button(start, true)],
    )?;
    for x in (8..=216).step_by(8) {
        frame(
            &mut g,
            &mut s,
            vec![Event::PointerMoved(
                start
                    + Vec2::new(
                        x as f32,
                        -25. * (x as f32 / 216. * std::f32::consts::PI).sin(),
                    ),
            )],
        )?;
    }
    let end = start + Vec2::new(216., 0.);
    let (output, commands) = g.build(
        &s,
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
            events: vec![button(end, false)],
            focused: true,
            ..Default::default()
        },
    );
    paint(&mut g, output)?;
    ensure!(commands.len() == 1, "a pointer gesture must commit once");
    for c in commands {
        command = Some(serde_json::to_value(&c)?);
        s.execute(c.into())?;
    }
    let sculpted = s.engine.scene.clone();
    let base = &original.meshes.assets[0].geometry;
    let after = &sculpted.meshes.assets[0].geometry;
    let maximum = base
        .positions
        .iter()
        .zip(&after.positions)
        .map(|(a, b)| Vec3::from(*a).distance(Vec3::from(*b)))
        .fold(0_f32, f32::max);
    let changed = base
        .positions
        .iter()
        .zip(&after.positions)
        .filter(|(a, b)| Vec3::from(**a).distance(Vec3::from(**b)) > 1e-5)
        .count();
    ensure!(
        maximum > 0.1 && maximum < 0.5 && changed > 50,
        "unexpected ridge: {maximum}m/{changed} vertices"
    );
    s.execute(Command::Undo.into())?;
    ensure!(
        s.engine.scene == original,
        "undo did not restore the original"
    );
    s.execute(Command::Redo.into())?;
    ensure!(
        s.engine.scene == sculpted,
        "redo did not restore the stroke"
    );
    // Demonstrate MCP parity using precisely the command produced by pointer input.
    s.execute(Command::Undo.into())?;
    let mut replayed: Command = serde_json::from_value(command.clone().unwrap())?;
    // Undo intentionally expires component references. Reinspect before replaying
    // the identical authoring parameters through the shared dispatcher.
    if let Command::EditMesh {
        edit: ignis::meshes::Edit::SculptStroke { reference, .. },
    } = &mut replayed
    {
        *reference = s.engine.mesh_reference();
    }
    s.execute(replayed.into())?;
    ensure!(
        s.engine.scene == sculpted,
        "serialized shared command differs from UI geometry"
    );
    std::fs::write(
        root.join("pointer-command.json"),
        serde_json::to_vec_pretty(&command)?,
    )?;
    std::fs::write(
        root.join("pointer.scene.json"),
        serde_json::to_vec(&sculpted)?,
    )?;
    camera.position = [2., 1.05, 3.5];
    s.execute(
        Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut light = s.editor.work_light.clone().unwrap();
    light.direction = [0.7, -0.6, -0.8];
    s.execute(
        Command::EditEditor {
            action: Action::Lighting { light: Some(light) },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    std::fs::write(root.join("pointer-after.png"), g.capture(&s, 1440, 900)?)?;
    let mut viewport = s.engine.render_frame();
    viewport.camera = s.editor.current().camera.clone().unwrap();
    viewport.light = s.editor.work_light.clone().unwrap();
    std::fs::write(
        root.join("pointer-viewport.png"),
        g.renderer.capture(&viewport, 1000, 750)?,
    )?;
    s.execute(Command::Undo.into())?;
    std::fs::write(root.join("pointer-before.png"), g.capture(&s, 1440, 900)?)?;
    let mut shaping_results = vec![];
    for brush in [
        ignis::meshes::BrushKind::Flatten,
        ignis::meshes::BrushKind::Pinch,
        ignis::meshes::BrushKind::Crease,
        ignis::meshes::BrushKind::Scrape,
        ignis::meshes::BrushKind::Fill,
        ignis::meshes::BrushKind::Relax,
    ] {
        let mut settings = s.editor.sculpt.clone();
        settings.brush = brush;
        s.execute(
            Command::EditEditor {
                action: Action::Sculpt { settings },
                expected_editor_revision: None,
            }
            .into(),
        )?;
        frame(&mut g, &mut s, vec![])?;
        let view = g.viewport.unwrap();
        let start = view.center() - Vec2::new(20., 10.);
        let end = start + Vec2::new(40., 15.);
        let before = s.engine.snapshot();
        let (output, commands) = g.build(
            &s,
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
                events: vec![
                    Event::PointerMoved(start),
                    button(start, true),
                    Event::PointerMoved(end),
                    button(end, false),
                ],
                focused: true,
                ..Default::default()
            },
        );
        paint(&mut g, output)?;
        ensure!(commands.len() == 1, "{brush:?} must commit once");
        let Command::EditMesh { edit } = commands[0].clone() else {
            anyhow::bail!("expected a real sculpt operation");
        };
        let preview = s.execute(Command::PreviewMeshEdit { edit }.into())?.data;
        for c in commands {
            s.execute(c.into())?;
        }
        ensure!(
            serde_json::to_value(&s.engine.scene.meshes)? == preview["meshes"],
            "{brush:?} preview/commit differs"
        );
        ensure!(
            s.engine.snapshot()["scene"] != before["scene"],
            "{brush:?} was inert"
        );
        ensure!(
            s.engine.snapshot()["history"]["undo"].as_u64().unwrap()
                == before["history"]["undo"].as_u64().unwrap() + 1
        );
        std::fs::write(
            root.join(format!("brush-{brush:?}.png")),
            g.capture(&s, 1440, 900)?,
        )?;
        shaping_results.push(serde_json::json!({"brush":brush,"changed":true,"pointer_command":true,"preview_commit_identical":true}));
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.snapshot()["scene"] == before["scene"]);
    }
    std::fs::write(
        root.join("shaping-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"native_window":false,"build_id":s.engine.snapshot()["build_id"],"brushes":shaping_results}),
        )?,
    )?;
    let report = serde_json::json!({"passed":true,"native_window":false,"vertices":after.positions.len(),"changed_vertices":changed,"max_displacement_m":maximum,"command":command,"checks":["real pointer input","one commit per gesture","exact undo/redo","serialized shared command parity","before/after GPU capture"]});
    std::fs::write(
        root.join("pointer-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    critical_first_strokes(&mut g, &mut s, &root)?;
    detailed_diagonals(&mut g, &mut s, &root)?;
    saturated_volume(&mut g, &mut s, &root)?;
    crossed_diagonals(&mut g, &mut s, &root)?;
    detail_configuration(&mut g, &mut s, &root)?;
    println!(
        "pointer sculpt quality: {changed} vertices, {maximum}m; undo/redo, command parity, offscreen PNGs passed"
    );
    Ok(())
}

// Direct desktop-flow check without a window: operate the real controls, then
// confirm the pointer command freezes exactly the displayed resolution.
fn detail_configuration(g: &mut Graphics, s: &mut Service, root: &std::path::Path) -> Result<()> {
    fn locate(g: &mut Graphics, s: &mut Service, text: &str) -> Result<Pos2> {
        let (output, commands) = g.build(
            s,
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
                focused: true,
                ..Default::default()
            },
        );
        let p = output
            .shapes
            .iter()
            .find_map(|shape| find_text(&shape.shape, text));
        paint(g, output)?;
        for c in commands {
            s.execute(c.into())?;
        }
        p.ok_or_else(|| anyhow::anyhow!("control not visible: {text}"))
    }
    fn click(g: &mut Graphics, s: &mut Service, p: Pos2) -> Result<()> {
        frame(
            g,
            s,
            vec![Event::PointerMoved(p), button(p, true), button(p, false)],
        )?;
        Ok(())
    }
    let original = s.engine.scene.clone();
    let checkbox = locate(g, s, "Dyntopo · topologia dinâmica")?;
    click(g, s, checkbox)?;
    ensure!(!s.editor.sculpt.adaptive_detail);
    locate(g, s, "Topologia fixa")?;
    std::fs::write(root.join("topology-fixed-ui.png"), g.capture(s, 1440, 900)?)?;
    click(g, s, checkbox)?;
    ensure!(s.editor.sculpt.adaptive_detail);
    let combo = locate(g, s, "Detalhe do pincel")?;
    click(g, s, combo)?;
    let constant = locate(g, s, "Detalhe constante")?;
    click(g, s, constant)?;
    let ignis::mesh_tools::SculptDetail::Constant {
        edge_length: initial,
    } = s.editor.sculpt.detail
    else {
        anyhow::bail!("native detailing selector did not change shared preferences");
    };
    let value = locate(g, s, &format!("{initial:.5} m"))?;
    frame(g, s, vec![Event::PointerMoved(value), button(value, true)])?;
    frame(g, s, vec![Event::PointerMoved(value + Vec2::new(10., 0.))])?;
    let end = value + Vec2::new(30., 0.);
    frame(g, s, vec![Event::PointerMoved(end)])?;
    frame(g, s, vec![button(end, false)])?;
    let detail = s.editor.sculpt.detail;
    ensure!(
        matches!(detail, ignis::mesh_tools::SculptDetail::Constant { edge_length } if edge_length > initial)
    );
    ensure!(
        s.engine.scene == original,
        "preferences changed scene geometry"
    );
    frame(g, s, vec![])?;
    let view = g.viewport.unwrap();
    let start = view.center() - Vec2::new(80., 0.);
    frame(g, s, vec![Event::PointerMoved(start), button(start, true)])?;
    for i in 1..=20 {
        frame(
            g,
            s,
            vec![Event::PointerMoved(start + Vec2::new(i as f32 * 8., 0.))],
        )?;
    }
    let (output, commands) = g.build(
        s,
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
            events: vec![button(start + Vec2::new(160., 0.), false)],
            focused: true,
            ..Default::default()
        },
    );
    paint(g, output)?;
    ensure!(commands.len() == 1);
    let Command::EditMesh {
        edit: ignis::meshes::Edit::SculptStroke { detail: sent, .. },
    } = &commands[0]
    else {
        anyhow::bail!("configured stroke did not use shared mesh dispatcher");
    };
    ensure!(*sent == detail);
    let Command::EditMesh { edit } = commands[0].clone() else {
        unreachable!()
    };
    let preview = s.execute(Command::PreviewMeshEdit { edit }.into())?.data;
    for c in commands {
        s.execute(c.into())?;
    }
    ensure!(serde_json::to_value(&s.engine.scene.meshes)? == preview["meshes"]);
    std::fs::write(
        root.join("dyntopo-constant-ui.png"),
        g.capture(s, 1440, 900)?,
    )?;
    s.execute(Command::Undo.into())?;
    ensure!(s.engine.scene == original);
    std::fs::write(
        root.join("detail-configuration-report.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "passed":true,"native_window":false,"build_id":s.engine.snapshot()["build_id"],"detail":detail,
            "checks":["native toggle","native detail selector","horizontal numeric drag","pure preferences","pointer command parity","preview=commit","one undo"]
        }))?,
    )?;
    Ok(())
}

// Critical reproduction: first strokes on a clean Cube24 and invalid Grab publication/recovery.
fn critical_first_strokes(g: &mut Graphics, s: &mut Service, root: &std::path::Path) -> Result<()> {
    let id = s.selected_mesh.clone().unwrap();
    let mut asset = s.engine.scene.meshes.asset(&id)?.clone();
    asset.geometry = std::sync::Arc::new(ignis::mesh_tools::primitive(Primitive::Cube, 24, 2.)?);
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        }
        .into(),
    )?;
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [2., 1.05, 6.];
    camera.target = [2., 1.05, 0.];
    camera.fov_degrees = 48.;
    s.execute(
        Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut settings = s.editor.sculpt.clone();
    settings.brush = ignis::meshes::BrushKind::Grab;
    settings.radius = 0.241;
    settings.grab_strength = 0.7;
    s.execute(
        Command::EditEditor {
            action: Action::Sculpt {
                settings: settings.clone(),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    for _ in 0..3 {
        frame(g, s, vec![])?;
    }
    let start = g.viewport.unwrap().center();
    for sign in [-1., 1.] {
        let end = start + Vec2::new(120. * sign, 0.);
        let before = s.engine.snapshot();
        frame(g, s, vec![Event::PointerMoved(start), button(start, true)])?;
        frame(g, s, vec![Event::PointerMoved(end)])?;
        ensure!(s.engine.snapshot() == before, "Grab preview changed source");
        std::fs::write(root.join("grab-limited.png"), g.capture(s, 1440, 900)?)?;
        frame(g, s, vec![button(end, false)])?;
        ensure!(
            s.engine.snapshot()["scene"] != before["scene"],
            "limited Grab was swallowed"
        );
        let a = s.engine.scene.meshes.asset(&id)?;
        let original = &s.engine.scene;
        for face in &a.geometry.triangles {
            let [p, q, r] = face.map(|i| Vec3::from(a.geometry.positions[i as usize]));
            ensure!(
                (q - p).cross(r - p).length_squared() > 0.,
                "limited Grab collapsed a face"
            );
        }
        let _ = original;
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.snapshot()["scene"] == before["scene"]);
        // Recede after crossing the unsafe region: only the recovered candidate may commit.
        frame(g, s, vec![Event::PointerMoved(start), button(start, true)])?;
        frame(
            g,
            s,
            vec![Event::PointerMoved(start + Vec2::new(10. * sign, 0.))],
        )?;
        frame(g, s, vec![Event::PointerMoved(end)])?;
        let valid = start + Vec2::new(10. * sign, 0.);
        frame(g, s, vec![Event::PointerMoved(valid)])?;
        frame(g, s, vec![button(valid, false)])?;
        ensure!(
            s.engine.snapshot()["scene"] != before["scene"],
            "safe Grab failed to recover"
        );
        ensure!(
            s.engine.snapshot()["history"]["undo"].as_u64()
                == Some(before["history"]["undo"].as_u64().unwrap() + 1)
        );
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.snapshot()["scene"] == before["scene"]);
    }
    settings.brush = ignis::meshes::BrushKind::Draw;
    settings.draw_strength = 0.6;
    s.execute(
        Command::EditEditor {
            action: Action::Sculpt { settings },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut outputs = vec![];
    for reverse in [false, true] {
        frame(g, s, vec![])?;
        let view = g.viewport.unwrap();
        let mut path: Vec<_> = (0..=40)
            .map(|i| view.center() + Vec2::new(-70. + i as f32 * 3.5, -45. + i as f32 * 2.25))
            .collect();
        if reverse {
            path.reverse();
        }
        let before = s.engine.scene.clone();
        frame(
            g,
            s,
            vec![Event::PointerMoved(path[0]), button(path[0], true)],
        )?;
        for p in &path[1..] {
            frame(g, s, vec![Event::PointerMoved(*p)])?;
        }
        frame(g, s, vec![button(*path.last().unwrap(), false)])?;
        let a = before.meshes.asset(&id)?;
        let b = s.engine.scene.meshes.asset(&id)?;
        ensure!(a.geometry != b.geometry, "first Draw was inert");
        for face in &a.geometry.triangles {
            let [a0, a1, a2] = face.map(|i| Vec3::from(a.geometry.positions[i as usize]));
            let [b0, b1, b2] = face.map(|i| Vec3::from(b.geometry.positions[i as usize]));
            ensure!(
                (a1 - a0).cross(a2 - a0).dot((b1 - b0).cross(b2 - b0)) > 0.,
                "first Draw folded a face, reverse={reverse}"
            );
        }
        outputs.push(b.geometry.positions.clone());
        std::fs::write(
            root.join(if reverse {
                "cube-first-reverse.png"
            } else {
                "cube-first-forward.png"
            }),
            g.capture(s, 1440, 900)?,
        )?;
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.scene == before);
    }
    let deviation = outputs[0]
        .iter()
        .zip(&outputs[1])
        .map(|(a, b)| Vec3::from(*a).distance(Vec3::from(*b)))
        .fold(0f32, f32::max);
    ensure!(
        deviation < 0.04,
        "reverse path differed by more than a full-strength dab: {deviation}"
    );
    let report = serde_json::json!({"passed":true,"build_id":s.engine.snapshot()["build_id"],"native_window":false,"first_draw_forward_reverse_max_difference_m":deviation,"first_grab_limited_both_directions":true,"limited_release_is_one_undo":true,"grab_recede_recover_one_undo":true});
    std::fs::write(
        root.join("direction-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}

fn detailed_diagonals(g: &mut Graphics, s: &mut Service, root: &std::path::Path) -> Result<()> {
    let id = s.selected_mesh.clone().unwrap();
    let mut instance = s.engine.scene.meshes.instances[0].clone();
    instance.position = [-3., 1.05, 0.];
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutInstance { instance },
        }
        .into(),
    )?;
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [-6.5, 1.05, 0.];
    camera.target = [-3., 1.05, 0.];
    camera.fov_degrees = 48.;
    s.execute(
        Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera.clone()),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut settings = s.editor.sculpt.clone();
    settings.brush = ignis::meshes::BrushKind::Draw;
    settings.radius = 0.241;
    settings.draw_strength = 0.6;
    settings.adaptive_detail = true;
    s.execute(
        Command::EditEditor {
            action: Action::Sculpt { settings },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut cases = vec![];
    for slope in [-1., 1.] {
        for _ in 0..2 {
            frame(g, s, vec![])?;
        }
        let view = g.viewport.unwrap();
        let path: Vec<_> = (0..=48)
            .map(|i| {
                view.center() + Vec2::new(-120. + i as f32 * 5., slope * (-120. + i as f32 * 5.))
            })
            .collect();
        let before = s.engine.snapshot();
        let old = s.engine.scene.meshes.asset(&id)?.geometry.positions.len();
        frame(
            g,
            s,
            vec![Event::PointerMoved(path[0]), button(path[0], true)],
        )?;
        for pos in &path[1..] {
            frame(g, s, vec![Event::PointerMoved(*pos)])?;
        }
        ensure!(
            s.engine.snapshot() == before,
            "adaptive preview mutated source"
        );
        let (output, commands) = g.build(
            s,
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
                events: vec![button(*path.last().unwrap(), false)],
                focused: true,
                ..Default::default()
            },
        );
        paint(g, output)?;
        ensure!(
            commands.len() == 1,
            "adaptive stroke did not commit exactly once"
        );
        let Command::EditMesh {
            edit: ignis::meshes::Edit::SculptStroke { .. },
        } = &commands[0]
        else {
            anyhow::bail!("native detailed Volume must use shared scoped operator");
        };
        let Command::EditMesh { edit } = commands[0].clone() else {
            unreachable!()
        };
        let preview = s.execute(Command::PreviewMeshEdit { edit }.into())?.data;
        for command in commands {
            s.execute(command.into())?;
        }
        ensure!(
            serde_json::to_value(&s.engine.scene.meshes)? == preview["meshes"],
            "detail preview/commit differs"
        );
        let asset = s.engine.scene.meshes.asset(&id)?;
        let vertices = asset.geometry.positions.len();
        let triangles = asset.geometry.triangles.len();
        ensure!(vertices > old, "native detailed Volume failed to refine");
        ensure!(
            s.engine.snapshot()["history"]["undo"].as_u64()
                == Some(before["history"]["undo"].as_u64().unwrap() + 1)
        );
        let name = if slope < 0. {
            "detail-diagonal-minus"
        } else {
            "detail-diagonal-plus"
        };
        std::fs::write(
            root.join(format!("{name}-ui.png")),
            g.capture(s, 1440, 900)?,
        )?;
        let mut scene = s.engine.render_frame();
        scene.camera = camera.clone();
        scene.light = s.editor.work_light.clone().unwrap();
        scene.light.shadow = None;
        std::fs::write(
            root.join(format!("{name}.png")),
            g.renderer.capture(&scene, 1200, 1200)?,
        )?;
        cases.push(serde_json::json!({"slope":slope,"vertices":vertices,"triangles":triangles,"one_undo":true,"preview_commit_identical":true}));
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.snapshot()["scene"] == before["scene"]);
    }
    let result = serde_json::json!({"passed":true,"build_id":s.engine.snapshot()["build_id"],"cases":cases,"native_window":false});
    std::fs::write(
        root.join("detail-diagonals.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}

// Critical native saturation: near-full planar tessellation and graduated pressure.
fn saturated_volume(g: &mut Graphics, s: &mut Service, root: &std::path::Path) -> Result<()> {
    let id = s.selected_mesh.clone().unwrap();
    let mut asset = s.engine.scene.meshes.asset(&id)?.clone();
    let mut grid = ignis::meshes::Geometry::grid(63, 2.)?;
    for p in &mut grid.positions {
        *p = [-1., p[0], p[2]];
    }
    asset.geometry = std::sync::Arc::new(grid);
    asset = ignis::mesh_topology::Plan::new(&asset.geometry, None)?.asset(&asset)?;
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        }
        .into(),
    )?;
    let mut results = vec![];
    let mut previous = 0.;
    for force in [0.1, 0.3, 0.6] {
        let mut settings = s.editor.sculpt.clone();
        settings.brush = ignis::meshes::BrushKind::Draw;
        settings.radius = 0.08;
        settings.draw_strength = force;
        settings.draw_strength_is_relative = true;
        settings.adaptive_detail = true;
        s.execute(
            Command::EditEditor {
                action: Action::Sculpt { settings },
                expected_editor_revision: None,
            }
            .into(),
        )?;
        for _ in 0..2 {
            frame(g, s, vec![])?;
        }
        let view = g.viewport.unwrap();
        let before = s.engine.snapshot();
        let start = view.center() - Vec2::new(80., 0.);
        frame(g, s, vec![Event::PointerMoved(start), button(start, true)])?;
        for i in 1..=40 {
            frame(
                g,
                s,
                vec![Event::PointerMoved(start + Vec2::new(i as f32 * 4., 0.))],
            )?;
        }
        ensure!(
            s.engine.snapshot() == before,
            "saturation preview mutated source"
        );
        frame(g, s, vec![button(start + Vec2::new(160., 0.), false)])?;
        let a = s.engine.scene.meshes.asset(&id)?;
        let max = a
            .geometry
            .positions
            .iter()
            .map(|p| -p[0] - 1.)
            .fold(0f32, f32::max);
        ensure!(
            max > previous && max <= 0.08 * force * 0.5 + 1e-6,
            "saturation pressure failed: {force}/{max}"
        );
        ensure!(a.geometry.positions.len() <= ignis::meshes::MAX_VERTICES);
        ensure!(
            s.engine.snapshot()["history"]["undo"].as_u64()
                == Some(before["history"]["undo"].as_u64().unwrap() + 1)
        );
        std::fs::write(
            root.join(format!("saturation-{}.png", (force * 100.) as u32)),
            g.capture(s, 1440, 900)?,
        )?;
        results.push(serde_json::json!({"force":force,"max_displacement":max,"vertices":a.geometry.positions.len(),"triangles":a.geometry.triangles.len(),"one_undo":true}));
        previous = max;
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.snapshot()["scene"] == before["scene"]);
    }
    std::fs::write(
        root.join("saturation-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"build_id":s.engine.snapshot()["build_id"],"native_window":false,"cases":results}),
        )?,
    )?;
    Ok(())
}

// Crossing gestures must stay on the visible front, including exact triangle seams.
fn crossed_diagonals(g: &mut Graphics, s: &mut Service, root: &std::path::Path) -> Result<()> {
    let id = s.selected_mesh.clone().unwrap();
    let mut asset = s.engine.scene.meshes.asset(&id)?.clone();
    asset.geometry = std::sync::Arc::new(ignis::mesh_tools::primitive(Primitive::Cube, 24, 2.)?);
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        }
        .into(),
    )?;
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [-7., 1.05, 0.];
    camera.target = [-3., 1.05, 0.];
    camera.fov_degrees = 48.;
    s.execute(
        Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera.clone()),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut settings = s.editor.sculpt.clone();
    settings.brush = ignis::meshes::BrushKind::Draw;
    settings.radius = 0.241;
    settings.draw_strength = 0.6;
    settings.draw_strength_is_relative = true;
    settings.adaptive_detail = true;
    s.execute(
        Command::EditEditor {
            action: Action::Sculpt { settings },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut report = vec![];
    for order in [[-1., 1.], [1., -1.]] {
        let before = s.engine.scene.clone();
        let name = if order[0] < 0. { "cross-a" } else { "cross-b" };
        let mut counts = vec![];
        for slope in order {
            for _ in 0..2 {
                frame(g, s, vec![])?;
            }
            let view = g.viewport.unwrap();
            let path: Vec<_> = (0..=80)
                .map(|i| {
                    let t = -180. + i as f32 * 4.5;
                    view.center() + Vec2::new(t, slope * t)
                })
                .collect();
            let snapshot = s.engine.snapshot();
            frame(
                g,
                s,
                vec![Event::PointerMoved(path[0]), button(path[0], true)],
            )?;
            for &p in &path[1..] {
                frame(g, s, vec![Event::PointerMoved(p)])?;
            }
            ensure!(s.engine.snapshot() == snapshot, "X preview mutated source");
            let (output, commands) = g.build(
                s,
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 900.))),
                    events: vec![button(*path.last().unwrap(), false)],
                    focused: true,
                    ..Default::default()
                },
            );
            paint(g, output)?;
            ensure!(commands.len() == 1, "X stroke must commit once");
            let Command::EditMesh {
                edit: ignis::meshes::Edit::SculptStroke { stroke, .. },
            } = &commands[0]
            else {
                anyhow::bail!("X must use shared stroke command");
            };
            ensure!(
                stroke.centers.iter().all(|p| p[0] < -0.99),
                "X painted through the front seam: {:?}",
                stroke.centers
            );
            ensure!(
                stroke.centers.len() < 80,
                "unexpected depth jump inflated/truncated X path"
            );
            counts.push(stroke.centers.len());
            let Command::EditMesh { edit } = commands[0].clone() else {
                unreachable!()
            };
            let preview = s.execute(Command::PreviewMeshEdit { edit }.into())?.data;
            for c in commands {
                s.execute(c.into())?;
            }
            ensure!(serde_json::to_value(&s.engine.scene.meshes)? == preview["meshes"]);
            ensure!(
                s.engine.snapshot()["history"]["undo"].as_u64()
                    == Some(snapshot["history"]["undo"].as_u64().unwrap() + 1)
            );
        }
        std::fs::write(
            root.join(format!("{name}-ui.png")),
            g.capture(s, 1440, 900)?,
        )?;
        let mut render = s.engine.render_frame();
        render.camera = camera.clone();
        render.light = s.editor.work_light.clone().unwrap();
        std::fs::write(
            root.join(format!("{name}-shadow.png")),
            g.renderer.capture(&render, 1200, 1200)?,
        )?;
        render.light.shadow = None;
        std::fs::write(
            root.join(format!("{name}-no-shadow.png")),
            g.renderer.capture(&render, 1200, 1200)?,
        )?;
        render.camera.position = [-6.2, 1.7, 1.4];
        std::fs::write(
            root.join(format!("{name}-oblique.png")),
            g.renderer.capture(&render, 1200, 1200)?,
        )?;
        let a = s.engine.scene.meshes.asset(&id)?;
        report.push(serde_json::json!({"order":order,"samples":counts,"front_only":true,"vertices":a.geometry.positions.len(),"triangles":a.geometry.triangles.len(),"one_undo_per_stroke":true}));
        s.execute(Command::Undo.into())?;
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.scene == before);
    }
    std::fs::write(
        root.join("crossed-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"build_id":s.engine.snapshot()["build_id"],"native_window":false,"cases":report}),
        )?,
    )?;
    Ok(())
}
