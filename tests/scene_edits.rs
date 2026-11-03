use ignis::{
    engine::{Command, Engine, MAX_SCENE_EDITS, Request, SceneEdit},
    model::{BodyKind, Scene},
    physics::Input,
};
use serde_json::json;
fn scene() -> Scene {
    serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap()
}
fn rename(s: &Scene) -> Vec<SceneEdit> {
    let mut hero = s
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    hero.id = "hero".into();
    let mut hunter = s
        .entities
        .iter()
        .find(|e| e.id == "hunter")
        .unwrap()
        .clone();
    hunter.npc.as_mut().unwrap().target = Some("hero".into());
    vec![
        SceneEdit::Remove {
            id: "player".into(),
        },
        SceneEdit::Upsert {
            entity: hunter.into(),
        },
        SceneEdit::Upsert {
            entity: hero.into(),
        },
    ]
}
#[test]
fn references_resolve_at_commit_and_whole_batch_is_one_undo() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.scene.clone();
    e.input = Input {
        x: 1.,
        ..Default::default()
    };
    e.advance(30).unwrap();
    e.playing = true;
    e.execute(
        Command::EditScene {
            edits: rename(&e.scene),
        }
        .into(),
    )
    .unwrap();
    let after = e.scene.clone();
    assert_eq!(e.revision, 1);
    assert_eq!(e.simulation.tick, 0);
    assert!(!e.playing);
    assert_eq!(e.input, Input::default());
    assert_eq!(e.snapshot()["history"], json!({"undo":1,"redo":0}));
    assert!(!e.scene.entities.iter().any(|e| e.id == "player"));
    assert_eq!(
        e.scene
            .entities
            .iter()
            .find(|e| e.id == "hunter")
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .target,
        Some("hero".into())
    );
    e.advance(30).unwrap();
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
    assert_eq!(e.simulation.tick, 0);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, after);
    assert_eq!(e.snapshot()["history"]["undo"], 1);
}
#[test]
fn failed_local_final_or_stale_edit_preserves_live_world_and_redo() {
    let mut e = Engine::new(scene()).unwrap();
    e.execute(
        Command::SetCamera {
            camera: e.scene.camera.clone(),
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Undo.into()).unwrap();
    e.input = Input {
        x: 1.,
        ..Default::default()
    };
    e.advance(20).unwrap();
    e.playing = true;
    let before = e.snapshot();
    let replay = serde_json::to_value(e.replay()).unwrap();
    let mut camera = e.scene.camera.clone();
    camera.position[0] += 1.;
    let local = e
        .execute(
            Command::EditScene {
                edits: vec![
                    SceneEdit::SetCamera { camera },
                    SceneEdit::Remove {
                        id: "missing".into(),
                    },
                ],
            }
            .into(),
        )
        .unwrap_err();
    assert_eq!(local.code, "not_found");
    assert!(local.message.starts_with("Edit 1:"));
    assert_eq!(e.snapshot(), before);
    let dangling = e.execute(
        Command::EditScene {
            edits: vec![SceneEdit::Remove {
                id: "player".into(),
            }],
        }
        .into(),
    );
    assert!(dangling.is_err());
    let mut invalid = e.scene.camera.clone();
    invalid.target = invalid.position;
    assert!(
        e.execute(
            Command::EditScene {
                edits: vec![SceneEdit::SetCamera { camera: invalid }]
            }
            .into()
        )
        .is_err()
    );
    assert!(
        e.execute(Request {
            expected_revision: Some(0),
            command: Command::EditScene {
                edits: rename(&e.scene)
            }
        })
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    assert_eq!(serde_json::to_value(e.replay()).unwrap(), replay);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.snapshot()["history"]["redo"], 0);
}
#[test]
fn bounds_and_authoring_only_schema_reject_before_mutation() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    let op = SceneEdit::SetLight {
        light: e.scene.light.clone(),
    };
    for n in [0, MAX_SCENE_EDITS + 1] {
        assert!(
            e.execute(
                Command::EditScene {
                    edits: vec![op.clone(); n]
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    e.execute(
        Command::EditScene {
            edits: vec![op; MAX_SCENE_EDITS],
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.revision, 1);
    assert_eq!(e.snapshot()["history"]["undo"], 1);
    for op in [
        json!({"op":"step","ticks":1}),
        json!({"op":"save_scene","path":"x.json"}),
        json!({"op":"edit_scene","edits":[]}),
    ] {
        assert!(
            serde_json::from_value::<Request>(json!({"command":{"op":"edit_scene","edits":[op]}}))
                .is_err()
        );
    }
}
#[test]
fn staging_can_temporarily_exceed_final_entity_limit_without_building_partial_world() {
    let mut s = scene();
    let mut template = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Fixed))
        .unwrap()
        .clone();
    template.body = None;
    template.blocks_vision = Some(false);
    template.id = "visual".into();
    for i in s.entities.len()..512 {
        let mut e = template.clone();
        e.id = format!("visual_{i:03}");
        s.entities.push(e);
    }
    let removed: Vec<_> = s
        .entities
        .iter()
        .filter(|e| e.id.starts_with("visual_"))
        .take(256)
        .map(|e| e.id.clone())
        .collect();
    let mut edits = Vec::new();
    for i in 0..256 {
        let mut e = template.clone();
        e.id = format!("replacement_{i:03}");
        edits.push(SceneEdit::Upsert { entity: e.into() });
    }
    for id in &removed {
        edits.push(SceneEdit::Remove { id: id.clone() });
    }
    let mut e = Engine::new(s).unwrap();
    e.execute(Command::EditScene { edits }.into()).unwrap();
    assert_eq!(e.scene.entities.len(), 512);
    assert_eq!(e.snapshot()["history"]["undo"], 1);
    assert!(
        removed
            .iter()
            .all(|id| !e.scene.entities.iter().any(|e| e.id == *id))
    );
    e.execute(Command::Undo.into()).unwrap();
    assert!(
        removed
            .iter()
            .all(|id| e.scene.entities.iter().any(|e| e.id == *id))
    );
}
#[test]
fn batch_and_individual_commands_share_final_scene_and_runtime_behavior() {
    let mut batch = Engine::new(scene()).unwrap();
    let mut singles = Engine::new(scene()).unwrap();
    let mut player = batch
        .scene
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    player.color = [0.2, 0.4, 0.8];
    let mut camera = batch.scene.camera.clone();
    camera.position[0] += 1.;
    let edits = vec![
        SceneEdit::Upsert {
            entity: player.clone().into(),
        },
        SceneEdit::SetCamera {
            camera: camera.clone(),
        },
    ];
    batch.execute(Command::EditScene { edits }.into()).unwrap();
    singles
        .execute(
            Command::Upsert {
                entity: player.into(),
            }
            .into(),
        )
        .unwrap();
    singles
        .execute(Command::SetCamera { camera }.into())
        .unwrap();
    assert_eq!(batch.scene, singles.scene);
    assert_eq!(batch.revision, 1);
    assert_eq!(singles.revision, 2);
    batch.advance(90).unwrap();
    singles.advance(90).unwrap();
    assert_eq!(
        serde_json::to_value(batch.replay()).unwrap(),
        serde_json::to_value(singles.replay()).unwrap()
    );
}
