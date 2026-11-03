use ignis::{
    editor::{Action, Area, Axis, Panel, State},
    engine::Command,
    model::RoomRecipe,
    service::Service,
};

#[test]
fn picking_uses_viewport_origin_and_physical_resolution() {
    use egui::{Rect, pos2, vec2};
    let view = Rect::from_min_size(pos2(100., 80.), vec2(600., 400.));
    let Some(Command::SelectEditorAt {
        x,
        y,
        width,
        height,
    }) = ignis::editor_ui::pick_command(view, pos2(250., 180.), 2.)
    else {
        panic!("pick")
    };
    assert_eq!((x, y, width, height), (300., 200., 1200, 800));
    assert!(ignis::editor_ui::pick_command(view, pos2(99., 80.), 2.).is_none());
    assert!(ignis::editor_ui::pick_command(view, pos2(700., 480.), 2.).is_none());
    assert!(ignis::editor_ui::pick_command(view, pos2(250., 180.), f32::NAN).is_none());
    let large = Rect::from_min_size(pos2(0., 0.), vec2(3000., 2000.));
    let Some(Command::SelectEditorAt {
        x,
        y,
        width,
        height,
    }) = ignis::editor_ui::pick_command(large, pos2(1500., 1000.), 2.)
    else {
        panic!("large pick")
    };
    assert_eq!((x, y, width, height), (2048., 2000., 4096, 4000));
}

#[test]
fn layout_transactions_preserve_scene_and_runtime_on_success_and_error() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(Command::Step { ticks: 8 }.into()).unwrap();
    let scene = serde_json::to_value(&s.engine.scene).unwrap();
    let replay = serde_json::to_value(s.engine.replay()).unwrap();
    for action in [
        Action::Resize {
            area: 0,
            ratio: 0.65,
        },
        Action::Split {
            area: 1,
            axis: Axis::Vertical,
            panel: Panel::Events,
        },
        Action::Maximize { area: Some(1) },
        Action::Maximize { area: None },
        Action::Move {
            from: 3,
            to: 4,
            panel: Panel::Templates,
        },
        Action::Duplicate {
            id: "custom".into(),
            name: "Meu trabalho".into(),
        },
        Action::Rename {
            name: "Outra distribuição".into(),
        },
        Action::Restore,
        Action::RemoveWorkspace,
        Action::Switch {
            workspace: "scene".into(),
        },
    ] {
        s.execute(
            Command::EditEditor {
                action,
                expected_editor_revision: Some(s.editor.revision),
            }
            .into(),
        )
        .unwrap();
    }
    assert_eq!(serde_json::to_value(&s.engine.scene).unwrap(), scene);
    assert_eq!(serde_json::to_value(s.engine.replay()).unwrap(), replay);
    assert_eq!(s.engine.simulation.tick, 8);
    for action in [
        Action::Switch {
            workspace: "missing".into(),
        },
        Action::Resize {
            area: 0,
            ratio: f32::NAN,
        },
        Action::Close {
            area: 1,
            panel: Panel::Viewport,
        },
        Action::Maximize { area: Some(0) },
        Action::Move {
            from: 4,
            to: 999,
            panel: Panel::Events,
        },
        Action::Duplicate {
            id: "scene".into(),
            name: "duplicate".into(),
        },
        Action::Rename { name: "\n".into() },
    ] {
        let before = s.editor.clone();
        assert!(
            s.execute(
                Command::EditEditor {
                    action,
                    expected_editor_revision: None
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(s.editor, before);
    }
    assert!(
        s.execute(
            Command::EditEditor {
                action: Action::Restore,
                expected_editor_revision: Some(999)
            }
            .into()
        )
        .is_err()
    );
}

#[test]
fn persistence_is_separate_atomic_and_confined() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.editor
        .apply(Action::Resize {
            area: 0,
            ratio: 0.7,
        })
        .unwrap();
    s.execute(
        Command::SaveEditor {
            path: "editor/layout.json".into(),
        }
        .into(),
    )
    .unwrap();
    let saved = s.editor.clone();
    s.editor
        .apply(Action::Switch {
            workspace: "inspect".into(),
        })
        .unwrap();
    let revision = s.editor.revision;
    s.execute(
        Command::LoadEditor {
            path: "editor/layout.json".into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.editor.workspaces, saved.workspaces);
    assert_eq!(s.editor.revision, revision + 1);
    std::fs::write(
        root.path().join("invalid.json"),
        r#"{"version":2,"revision":0,"active":"none","workspaces":[]}"#,
    )
    .unwrap();
    let before = s.editor.clone();
    assert!(
        s.execute(
            Command::LoadEditor {
                path: "invalid.json".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(s.editor, before);
    assert!(
        s.execute(
            Command::SaveEditor {
                path: "../escape.json".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(s.editor, before);
    s.execute(Command::Reset.into()).unwrap();
    assert_eq!(s.editor, before);
    s.execute(
        Command::SaveScene {
            path: "scene.json".into(),
        }
        .into(),
    )
    .unwrap();
    assert!(
        !std::fs::read_to_string(root.path().join("scene.json"))
            .unwrap()
            .contains("workspaces")
    );
}

#[test]
fn empty_areas_collapse_and_one_viewport_survives() {
    let mut s = State::default();
    for action in [
        Action::Move {
            from: 999,
            to: 999,
            panel: Panel::Events,
        },
        Action::Move {
            from: 1,
            to: 1,
            panel: Panel::Events,
        },
    ] {
        let before = s.clone();
        assert!(s.apply(action).is_err());
        assert_eq!(s, before);
    }
    s.apply(Action::Move {
        from: 1,
        to: 4,
        panel: Panel::Viewport,
    })
    .unwrap();
    s.validate().unwrap();
    assert!(s.current().layout.find(1).is_none());
    let before = s.clone();
    assert!(
        s.apply(Action::Close {
            area: 4,
            panel: Panel::Viewport
        })
        .is_err()
    );
    assert_eq!(s, before);
    assert!(
        s.apply(Action::Open {
            area: 3,
            panel: Panel::Viewport
        })
        .is_err()
    );
    assert_eq!(s, before);
    s.apply(Action::Move {
        from: 4,
        to: 3,
        panel: Panel::Viewport,
    })
    .unwrap();
    s.apply(Action::Activate {
        area: 3,
        panel: Panel::Hierarchy,
    })
    .unwrap();
    s.apply(Action::Activate {
        area: 3,
        panel: Panel::Viewport,
    })
    .unwrap();
    s.validate().unwrap();
}

#[test]
fn malformed_and_excessive_layouts_are_rejected() {
    let mut state = State::default();
    state.workspaces[0].layout = Area::Tabs {
        id: 1,
        tabs: vec![Panel::Viewport, Panel::Viewport],
        active: Panel::Viewport,
    };
    assert!(state.validate().is_err());
    let mut state = State::default();
    state.workspaces[0].maximized = Some(999);
    assert!(state.validate().is_err());
    let mut state = State::default();
    for n in 0..6 {
        state
            .apply(Action::Duplicate {
                id: format!("w{n}"),
                name: format!("Work{n}"),
            })
            .unwrap();
    }
    let before = state.clone();
    assert!(
        state
            .apply(Action::Duplicate {
                id: "ninth".into(),
                name: "Overflow".into()
            })
            .is_err()
    );
    assert_eq!(state, before);
    let mut state = State::default();
    let mut accepted = 0;
    loop {
        let before = state.clone();
        if state
            .apply(Action::Split {
                area: 1,
                axis: Axis::Horizontal,
                panel: Panel::Events,
            })
            .is_err()
        {
            assert_eq!(state, before);
            break;
        }
        accepted += 1;
        assert!(accepted <= 16);
    }
    assert!(accepted > 0);
    let mut state = State {
        revision: u64::MAX,
        ..Default::default()
    };
    assert!(state.apply(Action::Restore).is_err());
    assert_eq!(state.revision, u64::MAX);
}

#[test]
fn editor_camera_and_picking_do_not_edit_scene_or_replay() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let scene = s.engine.scene.clone();
    let replay = serde_json::to_value(s.engine.replay()).unwrap();
    let player = scene
        .entities
        .iter()
        .find(|e| e.body == Some(ignis::model::BodyKind::Player))
        .unwrap();
    let p = player.transform.position;
    let camera = ignis::model::Camera {
        position: [p[0] + 2., p[1] + 3., p[2] + 2.],
        target: p,
        fov_degrees: 48.,
    };
    s.editor
        .apply(Action::Camera {
            camera: Some(camera.clone()),
        })
        .unwrap();
    s.execute(
        Command::SelectEditorAt {
            x: 200.,
            y: 200.,
            width: 400,
            height: 400,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.selected_id(), Some(player.id.as_str()));
    assert_eq!(s.engine.scene, scene);
    assert_eq!(serde_json::to_value(s.engine.replay()).unwrap(), replay);
    let moved = ignis::editor_ui::navigate_camera(&camera, egui::vec2(30., 15.), 0., false);
    moved.validate().unwrap();
    assert_ne!(moved.position, camera.position);
    assert_eq!(moved.target, camera.target);
    let zoom = ignis::editor_ui::navigate_camera(&camera, egui::Vec2::ZERO, 100., false);
    zoom.validate().unwrap();
    assert_ne!(zoom.position, camera.position);
    let pan = ignis::editor_ui::navigate_camera(&camera, egui::vec2(30., 15.), 0., true);
    pan.validate().unwrap();
    assert_ne!(pan.target, camera.target);
    let before = s.editor.clone();
    let mut invalid = camera.clone();
    invalid.target = invalid.position;
    assert!(
        s.editor
            .apply(Action::Camera {
                camera: Some(invalid)
            })
            .is_err()
    );
    assert_eq!(s.editor, before);
    s.editor.apply(Action::Camera { camera: None }).unwrap();
    assert!(s.editor.current().camera.is_none());
}

#[test]
fn sculpt_and_work_light_persist_without_editing_scene_or_replay() {
    use ignis::editor::{SculptSettings, studio_light};
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(Command::Step { ticks: 12 }.into()).unwrap();
    let scene = s.engine.scene.clone();
    let replay = serde_json::to_value(s.engine.replay()).unwrap();
    let mut light = studio_light().unwrap();
    light.ambient = 0.15;
    light.direct_intensity = 0.7;
    light.environment = Some(ignis::environment::Environment {
        sky: [0.8, 0.9, 1.],
        horizon: [0.4; 3],
        ground: [0.1; 3],
        axis: [0., 1., 0.],
        intensity: 0.5,
    });
    light.direction = [-1., -0.5, 0.3];
    light.shadow.as_mut().unwrap().resolution = 2048;
    let settings = SculptSettings {
        brush: ignis::meshes::BrushKind::Grab,
        detail: ignis::mesh_tools::SculptDetail::Constant { edge_length: 0.05 },
        radius: 0.85,
        draw_strength: 0.12,
        grab_strength: 0.4,
        smooth_strength: 0.3,
        ..Default::default()
    };
    for action in [
        Action::Lighting {
            light: Some(light.clone()),
        },
        Action::Sculpt {
            settings: settings.clone(),
        },
    ] {
        s.execute(
            Command::EditEditor {
                action,
                expected_editor_revision: Some(s.editor.revision),
            }
            .into(),
        )
        .unwrap();
    }
    s.execute(
        Command::SaveEditor {
            path: "layout.json".into(),
        }
        .into(),
    )
    .unwrap();
    s.execute(
        Command::EditEditor {
            action: Action::Lighting { light: None },
            expected_editor_revision: None,
        }
        .into(),
    )
    .unwrap();
    s.execute(
        Command::LoadEditor {
            path: "layout.json".into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.editor.work_light, Some(light.clone()));
    assert_eq!(s.editor.sculpt, settings);
    let mut invalid_light = light.clone();
    invalid_light.shadow.as_mut().unwrap().resolution = 4096;
    let mut invalid_direct = light.clone();
    invalid_direct.direct_intensity = f32::NAN;
    let mut invalid_environment = light.clone();
    invalid_environment.environment.as_mut().unwrap().axis = [0.; 3];
    let mut zero_light = light;
    zero_light.direction = [0.; 3];
    let mut invalid_radius = settings.clone();
    invalid_radius.radius = 0.;
    let mut invalid_force = settings;
    invalid_force.smooth_strength = f32::NAN;
    for action in [
        Action::Lighting {
            light: Some(invalid_light),
        },
        Action::Lighting {
            light: Some(zero_light),
        },
        Action::Lighting {
            light: Some(invalid_direct),
        },
        Action::Lighting {
            light: Some(invalid_environment),
        },
        Action::Sculpt {
            settings: invalid_radius,
        },
        Action::Sculpt {
            settings: invalid_force,
        },
    ] {
        let before = s.editor.clone();
        assert!(
            s.execute(
                Command::EditEditor {
                    action,
                    expected_editor_revision: None
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(s.editor, before);
    }
    assert_eq!(s.engine.scene, scene);
    assert_eq!(serde_json::to_value(s.engine.replay()).unwrap(), replay);
    assert_eq!(s.engine.simulation.tick, 12);
}

#[test]
fn legacy_layouts_without_sculpt_or_light_load_with_visible_defaults() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    for version in [1, 2] {
        let mut legacy = serde_json::to_value(State::default()).unwrap();
        legacy["version"] = version.into();
        legacy.as_object_mut().unwrap().remove("sculpt");
        legacy.as_object_mut().unwrap().remove("work_light");
        // Version 1 predates the Project panel.
        if version == 1 {
            let mut state = State::default();
            state
                .apply(Action::Close {
                    area: 3,
                    panel: Panel::Project,
                })
                .unwrap();
            legacy["workspaces"] = serde_json::to_value(state.workspaces).unwrap();
        }
        std::fs::write(
            root.path().join("legacy.json"),
            serde_json::to_vec(&legacy).unwrap(),
        )
        .unwrap();
        s.execute(
            Command::LoadEditor {
                path: "legacy.json".into(),
            }
            .into(),
        )
        .unwrap();
        assert_eq!(s.editor.version, 3);
        assert_eq!(s.editor.sculpt, Default::default());
        assert_eq!(s.editor.work_light, ignis::editor::studio_light());
        assert_eq!(
            serde_json::to_value(&s.editor.workspaces).unwrap(),
            legacy["workspaces"]
        );
    }
}
