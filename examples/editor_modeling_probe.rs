//! Offscreen GPU interaction proof. The human editor process is never accessed.
use anyhow::{Context, Result, ensure};
use egui::{Event, Pos2, Rect, Vec2};
use ignis::{
    editor::Action,
    editor_ui::Graphics,
    engine::Command,
    mesh_topology::{Components, Selection},
    model::RoomRecipe,
    service::Service,
};
fn frame(
    g: &mut Graphics,
    s: &mut Service,
    events: Vec<Event>,
    size: [u32; 2],
    ppp: f32,
    focused: bool,
) -> Result<Vec<(String, Rect)>> {
    let mut input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(size[0] as f32 / ppp, size[1] as f32 / ppp),
        )),
        events,
        focused,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .native_pixels_per_point = Some(ppp);
    let (output, commands) = g.build(s, input);
    fn collect(shape: &egui::Shape, out: &mut Vec<(String, Rect)>) {
        match shape {
            egui::Shape::Text(t) => out.push((
                t.galley.text().into(),
                Rect::from_min_size(t.pos, t.galley.size()),
            )),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, out)
                }
            }
            _ => {}
        }
    }
    let mut labels = vec![];
    for shape in &output.shapes {
        collect(&shape.shape, &mut labels)
    }
    let target = g.renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("modeling interaction probe"),
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
        match s.execute(command.into()) {
            Ok(reply) => g.command_result(reply.data),
            Err(error) => {
                g.command_error(error.message.clone());
                return Err(error.into());
            }
        }
    }
    Ok(labels)
}
fn button(p: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: p,
        pressed,
        button: egui::PointerButton::Primary,
        modifiers: Default::default(),
    }
}
fn at(labels: &[(String, Rect)], name: &str) -> Result<Pos2> {
    labels
        .iter()
        .find(|(s, _)| s == name)
        .map(|(_, r)| r.center())
        .with_context(|| format!("Missing {name}; {labels:?}"))
}
fn click(
    g: &mut Graphics,
    s: &mut Service,
    p: Pos2,
    size: [u32; 2],
    ppp: f32,
) -> Result<Vec<(String, Rect)>> {
    frame(
        g,
        s,
        vec![Event::PointerMoved(p), button(p, true)],
        size,
        ppp,
        true,
    )?;
    frame(g, s, vec![button(p, false)], size, ppp, true)
}
fn selection(s: &mut Service, bevel: bool) -> Result<()> {
    let instance = s.engine.scene.meshes.instances[0].clone();
    let a = s.engine.scene.meshes.asset(&instance.asset)?;
    let components = if bevel {
        let edge = ignis::mesh_topology::Topology::inspect(&a.geometry)?
            .edges
            .into_iter()
            .find(|e| {
                e.vertices.iter().all(|i| {
                    let p = a.geometry.positions[*i as usize];
                    p[0] > 0.999 && p[1] > 0.999
                })
            })
            .context("cube geometric edge")?
            .vertices;
        Components::Edges {
            vertices: vec![edge],
        }
    } else {
        Components::Faces {
            indices: a
                .geometry
                .triangles
                .iter()
                .enumerate()
                .filter(|(_, f)| {
                    f.iter()
                        .all(|i| a.geometry.positions[*i as usize][1] > 0.999)
                })
                .map(|(i, _)| i as u32)
                .collect(),
        }
    };
    s.execute(
        Command::SelectMeshComponents {
            selection: Some(Selection {
                asset: instance.asset,
                reference: s.engine.mesh_reference(),
                components,
            }),
        }
        .into(),
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut s = Service::new(
        root.path(),
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()?,
    )?;
    s.execute(
        Command::CreateEditorMesh {
            primitive: ignis::mesh_tools::Primitive::Cube,
            resolution: 4,
            size: 2.,
        }
        .into(),
    )?;
    s.execute(
        Command::SaveScene {
            path: "probe.scene.json".into(),
        }
        .into(),
    )?;
    let asset = s.engine.scene.meshes.assets[0].id.clone();
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::SetShading {
                asset,
                shading: ignis::meshes::Shading::Flat,
            },
        }
        .into(),
    )?;
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
    )?;
    let mut g = Graphics::offscreen()?;
    let size = [1440, 900];
    let dir = std::path::Path::new("artifacts/editor-modeling");
    std::fs::create_dir_all(dir)?;
    let mut labels = frame(&mut g, &mut s, vec![], size, 1., true)?;
    let edit = labels
        .iter()
        .find(|(t, r)| t == "Editar" && r.top() > 50.)
        .context("Edit mode")?
        .1
        .center();
    click(&mut g, &mut s, edit, size, 1.)?;
    let mut checks = vec![];
    for (name, bevel, axis) in [
        ("Extrudar", false, Vec2::new(0., -12.)),
        ("Inset", false, Vec2::new(12., 0.)),
        ("Bevel", true, Vec2::new(12., 0.)),
    ] {
        selection(&mut s, bevel)?;
        for _ in 0..2 {
            labels = frame(&mut g, &mut s, vec![], size, 1., true)?;
        }
        let before = s.engine.snapshot();
        click(&mut g, &mut s, at(&labels, name)?, size, 1.)?;
        labels = frame(&mut g, &mut s, vec![], size, 1., true)?;
        ensure!(
            labels.iter().any(|(t, _)| t == "Confirmar operação"),
            "{name} preview did not start"
        );
        ensure!(s.engine.snapshot() == before, "button mutated source");
        let start = g.viewport.unwrap().center();
        let end = start + axis;
        frame(
            &mut g,
            &mut s,
            vec![Event::PointerMoved(start), button(start, true)],
            size,
            1.,
            true,
        )?;
        frame(
            &mut g,
            &mut s,
            vec![Event::PointerMoved(end)],
            size,
            1.,
            true,
        )?;
        ensure!(s.engine.snapshot() == before, "drag mutated source");
        // capture() draws the same cached draft; no window or model mutation.
        std::fs::write(
            dir.join(format!("{name}-preview.png")),
            g.capture(&s, size[0], size[1])?,
        )?;
        frame(&mut g, &mut s, vec![button(end, false)], size, 1., true)?;
        ensure!(
            s.engine.snapshot()["scene"] != before["scene"],
            "release did not publish {name}"
        );
        ensure!(
            s.engine.snapshot()["history"]["undo"].as_u64()
                == Some(before["history"]["undo"].as_u64().unwrap() + 1)
        );
        ensure!(matches!(
            s.engine
                .selected_mesh_components()
                .context("result selection")?
                .components,
            Components::Faces { .. }
        ));
        std::fs::write(
            dir.join(format!("{name}.png")),
            g.capture(&s, size[0], size[1])?,
        )?;
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.snapshot()["scene"] == before["scene"]);
        checks.push(format!("{name}: actual Editar button, GPU preview, viewport drag/release, result selection and one exact undo"));
    }
    // Cancellation gates are exercised while a real viewport gesture is active.
    for reason in ["focus", "resize", "hidpi"] {
        selection(&mut s, false)?;
        for _ in 0..2 {
            labels = frame(&mut g, &mut s, vec![], size, 1., true)?;
        }
        click(&mut g, &mut s, at(&labels, "Inset")?, size, 1.)?;
        frame(&mut g, &mut s, vec![], size, 1., true)?;
        let before = s.engine.snapshot();
        let p = g.viewport.unwrap().center();
        frame(
            &mut g,
            &mut s,
            vec![Event::PointerMoved(p), button(p, true)],
            size,
            1.,
            true,
        )?;
        let current = if reason == "resize" {
            [1200, 800]
        } else {
            size
        };
        let ppp = if reason == "hidpi" { 2. } else { 1. };
        labels = frame(&mut g, &mut s, vec![], current, ppp, reason != "focus")?;
        ensure!(
            !labels.iter().any(|(t, _)| t
                == "PRÉVIA · arraste e solte · Shift fino · Enter confirmar · Esc cancelar"),
            "{reason} left a draft active"
        );
        frame(&mut g, &mut s, vec![button(p, false)], current, ppp, true)?;
        ensure!(
            s.engine.snapshot() == before,
            "{reason} cancellation changed source/selection"
        );
        frame(&mut g, &mut s, vec![], size, 1., true)?;
        checks.push(format!(
            "{reason}: draft canceled and release consumed without edit/deselection"
        ));
    }
    for width in [1440, 1000, 900, 800] {
        std::fs::write(
            dir.join(format!("layout-{width}.png")),
            g.capture(&s, width, 800)?,
        )?;
    }
    let report = serde_json::json!({"passed":true,"build_id":s.engine.snapshot()["build_id"],"checks":checks,"widths":[1440,1000,900,800],"native_window":false});
    std::fs::write(dir.join("probe.json"), serde_json::to_vec_pretty(&report)?)?;
    println!("{report}");
    Ok(())
}
