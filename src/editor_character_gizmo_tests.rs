use super::*;
use crate::meshes::character::*;

fn service(root: &std::path::Path) -> Service {
    let names = ["x", "y", "z"].map(str::to_owned);
    let source = Source {
        version: 2,
        id: "face".into(),
        provenance: "Synthetic transaction fixture".into(),
        positions: vec![
            [-0.1, -0.1, 0.],
            [-0.1, 0.1, 0.],
            [0.1, 0.1, 0.],
            [0.1, -0.1, 0.],
        ],
        parameters: names
            .iter()
            .map(|name| Parameter {
                name: name.clone(),
                minimum: -0.03,
                maximum: 0.03,
                default: 0.,
            })
            .collect(),
        targets: names
            .iter()
            .enumerate()
            .map(|(axis, name)| {
                let mut offset = [0.; 3];
                offset[axis] = 0.03;
                Target {
                    name: format!("move_{name}"),
                    gain: 1.,
                    factors: vec![Factor {
                        parameter: name.clone(),
                        curve: vec![[-0.03, -1.], [0.03, 1.]],
                    }],
                    deltas: vec![Delta { vertex: 2, offset }],
                }
            })
            .collect(),
        joints: vec![Joint {
            name: "head".into(),
            parent: None,
            head: Anchor { vertices: vec![0] },
            tail: Anchor { vertices: vec![1] },
        }],
        parts: vec![Part {
            name: "skin".into(),
            vertices: vec![0, 1, 2, 3],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            weights: vec![
                crate::meshes::Weight {
                    joints: [0; 4],
                    weights: [1., 0., 0., 0.],
                    extra: None
                };
                4
            ],
            socket: None,
            color: [0.5; 3],
            material: Default::default(),
        }],
        handles: vec![Handle {
            group: None,
            name: "cheek".into(),
            label: "Cheek".into(),
            part: "skin".into(),
            anchor: Anchor { vertices: vec![2] },
            parameters: names,
            limits: [0.03; 3],
            mirror: None,
            frame_joint: 0,
            midline: false,
        }],
        warps: vec![],
        refinements: vec![],
        surfaces: vec![],
    };
    std::fs::write(
        root.join("face.ignis-character"),
        serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    let mut scene = crate::model::RoomRecipe::default().generate().unwrap();
    for e in &mut scene.entities {
        e.visible = Some(false);
    }
    scene.camera = Camera {
        position: [0., 0., -1.],
        target: [0., 0., 0.],
        fov_degrees: 45.,
    };
    let mut service = Service::new(root, scene).unwrap();
    service
        .execute(
            Command::PublishCharacter {
                path: "face.ignis-character".into(),
                configuration: Default::default(),
                placement: Placement {
                    prefix: "face".into(),
                    position: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                    scale: 1.,
                },
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::OpenCharacterPanel {
                open: true,
                prefix: None,
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::SetCharacterHandleView {
                view: crate::character_editor::HandleView {
                    group: None,
                    enabled: true,
                    symmetry: true,
                    selected: Some("cheek".into()),
                },
            }
            .into(),
        )
        .unwrap();
    service
}
fn pointer(p: Pos2, down: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed: down,
        modifiers: Default::default(),
    }
}
fn step(
    ctx: &egui::Context,
    state: &mut State,
    service: &Service,
    events: Vec<egui::Event>,
) -> (Vec<Command>, bool) {
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800., 600.));
    let mut commands = Vec::new();
    let mut blocked = false;
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(rect),
        events,
        focused: true,
        ..Default::default()
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        let response = ui.interact(
            rect,
            egui::Id::new("viewport_image"),
            egui::Sense::click_and_drag(),
        );
        blocked = viewport(ui, service, state, &mut commands, &response, rect);
    });
    let _ = ctx.end_pass();
    (commands, blocked)
}
// Critical integration risk: an abandoned/stale native drag must not commit a
// preview, leak its mouse release to scene picking, or split undo transactions.
#[test]
fn native_face_preview_cancel_hidpi_and_release_commit_are_atomic() {
    let root = tempfile::tempdir().unwrap();
    let mut service = service(root.path());
    let ctx = egui::Context::default();
    let mut state = State::default();
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800., 600.));
    let view = View::new(rect, 1.);
    let point = view
        .project(
            &view.projection(camera(&service)).unwrap(),
            service.character_editor.handles[0].position,
        )
        .unwrap();
    let moved = point + Vec2::new(18., -6.);
    let original = service.engine.scene.clone();
    step(&ctx, &mut state, &service, vec![]);
    let (commands, _) = step(
        &ctx,
        &mut state,
        &service,
        vec![egui::Event::PointerMoved(point), pointer(point, true)],
    );
    assert!(commands.is_empty());
    step(
        &ctx,
        &mut state,
        &service,
        vec![egui::Event::PointerMoved(moved)],
    );
    assert!(state.preview().is_some());
    assert_eq!(service.engine.scene, original);
    let escape = egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    };
    let (commands, blocked) = step(&ctx, &mut state, &service, vec![escape]);
    assert!(blocked && commands.is_empty());
    assert!(state.preview().is_none());
    let (commands, blocked) = step(&ctx, &mut state, &service, vec![pointer(moved, false)]);
    assert!(blocked && commands.is_empty());
    assert_eq!(service.engine.scene, original);
    step(
        &ctx,
        &mut state,
        &service,
        vec![egui::Event::PointerMoved(point), pointer(point, true)],
    );
    step(
        &ctx,
        &mut state,
        &service,
        vec![egui::Event::PointerMoved(moved)],
    );
    assert!(state.preview().is_some());
    ctx.set_pixels_per_point(2.);
    let (commands, blocked) = step(&ctx, &mut state, &service, vec![]);
    assert!(blocked && commands.is_empty());
    assert!(state.preview().is_none());
    step(&ctx, &mut state, &service, vec![pointer(moved, false)]);
    assert_eq!(service.engine.scene, original);
    step(
        &ctx,
        &mut state,
        &service,
        vec![egui::Event::PointerMoved(point), pointer(point, true)],
    );
    step(
        &ctx,
        &mut state,
        &service,
        vec![egui::Event::PointerMoved(moved)],
    );
    let (commands, blocked) = step(&ctx, &mut state, &service, vec![pointer(moved, false)]);
    assert!(blocked);
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0],
        Command::SetCharacterHandle { gesture: None, .. }
    ));
    for command in commands {
        service.execute(command.into()).unwrap();
    }
    assert_ne!(service.engine.scene, original);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, original);
}
