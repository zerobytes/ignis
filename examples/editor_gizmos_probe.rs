//! GPU offscreen interaction proof for transformation gizmos; never touches the user's window.
use anyhow::{Result, ensure};
use egui::{Event, Pos2, Rect, Vec2};
use glam::Vec3;
use ignis::{
    editor::Action,
    editor_transform::{self as tr, Settings, Tool},
    editor_ui::Graphics,
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
fn frame(g: &mut Graphics, s: &mut Service, events: Vec<Event>, size: [u32; 2]) -> Result<()> {
    let (output, commands) = g.build(
        s,
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(size[0] as f32, size[1] as f32),
            )),
            events,
            ..Default::default()
        },
    );
    let target = g.renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("gizmo offscreen probe"),
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
    Ok(())
}
fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        pressed,
        button: egui::PointerButton::Primary,
        modifiers: Default::default(),
    }
}
fn main() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut s = Service::new(root.path(), RoomRecipe::default().generate()?)?;
    s.execute(
        Command::CreateEditorMesh {
            primitive: ignis::mesh_tools::Primitive::Sphere,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )?;
    // A saved scene avoids the project chooser without modifying product UI internals.
    s.execute(
        Command::SaveScene {
            path: "test.scene.json".into(),
        }
        .into(),
    )?;
    let mut camera = s.engine.scene.camera.clone();
    camera.position = [5., 4., 7.];
    camera.target = [2., 1.05, 0.];
    s.execute(
        Command::EditEditor {
            action: Action::Camera {
                camera: Some(camera.clone()),
            },
            expected_editor_revision: None,
        }
        .into(),
    )?;
    let mut g = Graphics::offscreen()?;
    let output = std::path::Path::new("artifacts/editor-gizmos");
    std::fs::create_dir_all(output)?;
    let size = [1440, 900];
    let mut checks = vec![];
    for tool in [Tool::Move, Tool::Rotate, Tool::Scale] {
        s.execute(
            Command::SetEditorTransform {
                settings: Settings {
                    tool,
                    ..Default::default()
                },
            }
            .into(),
        )?;
        frame(&mut g, &mut s, vec![], size)?;
        frame(&mut g, &mut s, vec![], size)?;
        let view = g.viewport.unwrap();
        let selection = tr::selection(&s, false)?;
        let pivot = Vec3::from(selection.center);
        let projection = ignis::mesh_screen::Projection::new(
            &camera,
            view.width().round() as u32,
            view.height().round() as u32,
        )?;
        let project = |p| {
            let p = projection.project(p).unwrap();
            view.min + Vec2::new(p.x, p.y)
        };
        let depth = (pivot - Vec3::from(camera.position))
            .dot((Vec3::from(camera.target) - Vec3::from(camera.position)).normalize());
        let length =
            2. * depth * (camera.fov_degrees.to_radians() * 0.5).tan() * 78. / view.height();
        let (start, end) = match tool {
            Tool::Move => (
                project(pivot + Vec3::X * length * 0.8),
                project(pivot + Vec3::X * length * 1.1),
            ),
            Tool::Rotate => {
                let at = |a: f32| project(pivot + (Vec3::X * a.cos() + Vec3::Y * a.sin()) * length);
                (at(0.8), at(1.1))
            }
            Tool::Scale => {
                let p = project(pivot) + Vec2::new(52., -52.);
                (p, p + Vec2::new(18., 0.))
            }
            _ => unreachable!(),
        };
        let before = s.engine.snapshot();
        frame(&mut g, &mut s, vec![Event::PointerMoved(start)], size)?;
        frame(&mut g, &mut s, vec![button(start, true)], size)?;
        frame(&mut g, &mut s, vec![Event::PointerMoved(end)], size)?;
        ensure!(s.engine.snapshot() == before, "Preview mutated engine");
        frame(&mut g, &mut s, vec![button(end, false)], size)?;
        ensure!(
            s.engine.snapshot()["scene"] != before["scene"],
            "{tool:?} did not transform"
        );
        ensure!(
            s.engine.snapshot()["history"]["undo"].as_u64().unwrap()
                == before["history"]["undo"].as_u64().unwrap() + 1
        );
        let png = g.capture(&s, size[0], size[1])?;
        std::fs::write(output.join(format!("{tool:?}.png")), png)?;
        s.execute(Command::Undo.into())?;
        ensure!(s.engine.snapshot()["scene"] == before["scene"]);
        checks.push(format!(
            "{tool:?}: mouse preview, GPU paint, commit and exact undo"
        ));
    }
    for width in [1000, 900, 800] {
        let png = g.capture(&s, width, 800)?;
        std::fs::write(output.join(format!("layout-{width}.png")), png)?;
    }
    let report = serde_json::json!({"passed":true,"build_id":s.engine.snapshot()["build_id"],"checks":checks,"widths":[1440,1000,900,800],"native_window":false});
    std::fs::write(
        output.join("probe.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
