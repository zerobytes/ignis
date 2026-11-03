use ignis::{
    engine::{Command, Engine, Replay},
    model::Scene,
    noise::NoiseOrigin,
    patrol, perception,
    physics::Input,
    scenario::{Action, Condition, Scenario},
    service::Service,
};
use serde_json::json;

fn scene() -> Scene {
    serde_json::from_str(include_str!("../examples/stealth.scene.json")).unwrap()
}
fn quiet() -> Scenario {
    serde_json::from_value(json!({"version":1,"name":"Quiet motion","budget_ticks":60,"steps":[
        {"op":"step","ticks":30},
        {"op":"set_input","input":{"x":1,"z":0,"mode":"sneak"}},
        {"op":"step","ticks":20},
        {"op":"assert","condition":{"kind":"npc_memory","id":"hunter","source":null}},
        {"op":"assert","condition":{"kind":"footsteps","min":1,"max":3}},
        {"op":"assert","condition":{"kind":"position_within","id":"player","center":[0.5,0.8,-2],"radius":0.1}},
        {"op":"until","condition":{"kind":"npc_observed","id":"hunter","expected":false},"max_ticks":1}
    ]})).unwrap()
}
#[test]
fn typed_scenario_is_repeatable_and_until_can_finish_without_a_tick() {
    let scenario = quiet();
    let a = scenario.run(&scene()).unwrap();
    let b = scenario.run(&scene()).unwrap();
    assert!(a.report.passed);
    assert_eq!(a.report.ticks, 50);
    assert_eq!(a.report.completed_steps, scenario.steps.len());
    assert_eq!(a.report.checks.last().unwrap().tick, 50);
    assert_eq!(
        serde_json::to_value(&a.report).unwrap(),
        serde_json::to_value(&b.report).unwrap()
    );
    assert_eq!(
        Engine::verify_replay(&a.engine.replay()).unwrap()["matched"],
        true
    );
}
#[test]
fn patrol_interruption_and_resume_are_verified_as_one_bounded_scenario() {
    let scene: Scene = serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap();
    let scenario = Scenario {
        version: 1,
        name: "Patrol resumption".into(),
        budget_ticks: 2400,
        steps: vec![
            Action::Step { ticks: 30 },
            Action::Assert {
                condition: Condition::PatrolPhase {
                    id: "hunter".into(),
                    phase: patrol::Phase::Moving,
                },
            },
            Action::EmitNoise {
                origin: NoiseOrigin::Position {
                    position: [0., 0.8, 4.],
                },
                radius: 12.,
            },
            Action::Step { ticks: 1 },
            Action::Assert {
                condition: Condition::NpcMemory {
                    id: "hunter".into(),
                    source: Some(perception::Source::Sound),
                },
            },
            Action::Assert {
                condition: Condition::NpcPhase {
                    id: "hunter".into(),
                    phase: perception::Phase::Investigating,
                },
            },
            Action::Assert {
                condition: Condition::PatrolPhase {
                    id: "hunter".into(),
                    phase: patrol::Phase::Suspended,
                },
            },
            Action::Until {
                condition: Condition::PatrolPhase {
                    id: "hunter".into(),
                    phase: patrol::Phase::Moving,
                },
                max_ticks: 1000,
            },
            Action::Assert {
                condition: Condition::NpcMemory {
                    id: "hunter".into(),
                    source: None,
                },
            },
            Action::Until {
                condition: Condition::PatrolVisits {
                    id: "hunter".into(),
                    at_least: 1,
                },
                max_ticks: 1000,
            },
        ],
    };
    let outcome = scenario.run(&scene).unwrap();
    assert!(outcome.report.passed, "{:?}", outcome.report.failure);
    assert_eq!(outcome.report.checks[4].tick, 922);
    assert!(outcome.report.ticks < 2400);
    assert_eq!(
        Engine::verify_replay(&outcome.engine.replay()).unwrap()["matched"],
        true
    );
}
#[test]
fn first_assertion_timeout_or_execution_error_stops_with_exact_evidence() {
    let mut s = quiet();
    s.steps = vec![
        Action::Assert {
            condition: Condition::Footsteps { min: 1, max: 10 },
        },
        Action::Step { ticks: 30 },
    ];
    let result = s.run(&scene()).unwrap();
    assert!(!result.report.passed);
    assert_eq!(result.report.ticks, 0);
    assert_eq!(result.report.completed_steps, 0);
    assert_eq!(result.report.checks[0].actual, 0);
    assert_eq!(result.report.failure.unwrap().code, "assertion_failed");
    s.steps[0] = Action::Until {
        condition: Condition::Footsteps { min: 1, max: 10 },
        max_ticks: 3,
    };
    let result = s.run(&scene()).unwrap();
    assert_eq!(result.report.ticks, 3);
    assert_eq!(result.report.checks[0].tick, 3);
    assert_eq!(result.report.failure.unwrap().code, "condition_timeout");
    s.steps = vec![
        Action::EmitNoise {
            origin: NoiseOrigin::Entity {
                id: "player".into()
            },
            radius: 8.
        };
        17
    ];
    s.steps.push(Action::Assert {
        condition: Condition::Footsteps { min: 0, max: 0 },
    });
    let result = s.run(&scene()).unwrap();
    let error = result.report.failure.unwrap();
    assert_eq!(error.step_index, 16);
    assert_eq!(error.tick, 0);
    assert_eq!(error.code, "invalid_argument");
    assert_eq!(result.report.completed_steps, 16);
    assert!(result.report.checks.is_empty());
}
#[test]
fn invalid_scripts_are_rejected_before_execution() {
    for invalid in [
        Action::SetInput {
            input: Input {
                x: f32::NAN,
                z: 0.,
                ..Default::default()
            },
        },
        Action::Step { ticks: 0 },
        Action::Step { ticks: u32::MAX },
        Action::Until {
            condition: Condition::Footsteps { min: 0, max: 0 },
            max_ticks: 0,
        },
        Action::Assert {
            condition: Condition::Footsteps { min: 4, max: 1 },
        },
        Action::Assert {
            condition: Condition::PositionWithin {
                id: "player".into(),
                center: [f32::INFINITY, 0., 0.],
                radius: 1.,
            },
        },
        Action::Assert {
            condition: Condition::NpcObserved {
                id: "player".into(),
                expected: false,
            },
        },
        Action::Assert {
            condition: Condition::PatrolVisits {
                id: "hunter".into(),
                at_least: 1,
            },
        },
        Action::EmitNoise {
            origin: NoiseOrigin::Entity {
                id: "absent".into(),
            },
            radius: 8.,
        },
    ] {
        let mut s = quiet();
        s.steps.insert(0, invalid);
        assert!(s.run(&scene()).is_err());
    }
    let mut s = quiet();
    s.version = 3;
    assert!(s.validate(&scene()).is_err());
    s.version = 1;
    s.budget_ticks = 3601;
    assert!(s.validate(&scene()).is_err());
    s.budget_ticks = 50;
    assert!(s.validate(&scene()).is_err()); // conservative maxima total51
    s.budget_ticks = 60;
    s.steps = vec![Action::Step { ticks: 1 }];
    assert!(s.validate(&scene()).is_err());
    s.steps = vec![
        Action::Assert {
            condition: Condition::Footsteps { min: 0, max: 0 }
        };
        257
    ];
    assert!(s.validate(&scene()).is_err());
}
#[test]
fn service_runs_from_authoring_preserves_live_state_and_saves_report_and_replay() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), scene()).unwrap();
    service
        .engine
        .execute(
            Command::SetInput {
                input: Input {
                    x: -1.,
                    z: 0.,
                    ..Default::default()
                },
            }
            .into(),
        )
        .unwrap();
    service.engine.advance(90).unwrap();
    service.engine.execute(Command::Play.into()).unwrap();
    let before = service.engine.snapshot();
    let replay = serde_json::to_value(service.engine.replay()).unwrap();
    service
        .execute(
            Command::SaveScenario {
                path: "test.scenario.json".into(),
                scenario: Box::new(quiet()),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(
        service
            .execute(
                Command::InspectScenario {
                    path: "test.scenario.json".into()
                }
                .into()
            )
            .unwrap()
            .data["scenario"]["name"],
        "Quiet motion"
    );
    let result = service
        .execute(
            Command::RunScenario {
                path: "test.scenario.json".into(),
                report_path: Some("artifacts/result.json".into()),
                replay_path: Some("artifacts/result.replay.json".into()),
            }
            .into(),
        )
        .unwrap()
        .data;
    assert_eq!(result["report"]["passed"], true);
    assert_eq!(result["report"]["ticks"], 50);
    assert_eq!(service.engine.snapshot(), before);
    assert_eq!(
        serde_json::to_value(service.engine.replay()).unwrap(),
        replay
    );
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("artifacts/result.json")).unwrap())
            .unwrap();
    assert_eq!(saved, result["report"]);
    let saved: Replay = serde_json::from_slice(
        &std::fs::read(dir.path().join("artifacts/result.replay.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(Engine::verify_replay(&saved).unwrap()["matched"], true);
    let file = std::fs::read(dir.path().join("test.scenario.json")).unwrap();
    let mut bad = quiet();
    bad.budget_ticks = 1;
    assert!(
        service
            .execute(
                Command::SaveScenario {
                    path: "test.scenario.json".into(),
                    scenario: Box::new(bad)
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(
        std::fs::read(dir.path().join("test.scenario.json")).unwrap(),
        file
    );
    for (report_path, replay_path) in [
        (Some("../outside.json"), None),
        (Some("test.scenario.json"), None),
        (Some("same.json"), Some("same.json")),
    ] {
        assert!(
            service
                .execute(
                    Command::RunScenario {
                        path: "test.scenario.json".into(),
                        report_path: report_path.map(str::to_string),
                        replay_path: replay_path.map(str::to_string)
                    }
                    .into()
                )
                .is_err()
        );
    }
    assert_eq!(service.engine.snapshot(), before);
}
#[cfg(unix)]
#[test]
fn output_aliases_cannot_overwrite_input_or_each_other() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), scene()).unwrap();
    service
        .execute(
            Command::SaveScenario {
                path: "data/test.json".into(),
                scenario: Box::new(quiet()),
            }
            .into(),
        )
        .unwrap();
    symlink(dir.path().join("data"), dir.path().join("alias")).unwrap();
    symlink(
        dir.path().join("data/test.json"),
        dir.path().join("input-link.json"),
    )
    .unwrap();
    for (source, report, replay) in [
        ("data/test.json", "alias/test.json", None),
        ("input-link.json", "data/test.json", None),
        ("data/test.json", "data/new.json", Some("alias/new.json")),
    ] {
        assert!(
            service
                .execute(
                    Command::RunScenario {
                        path: source.into(),
                        report_path: Some(report.into()),
                        replay_path: replay.map(str::to_string)
                    }
                    .into()
                )
                .is_err()
        );
    }
    assert!(!dir.path().join("data/new.json").exists());
}

#[test]
fn condition_met_on_last_allowed_tick_succeeds() {
    let scene: Scene = serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap();
    let scenario = Scenario {
        version: 1,
        name: "Inclusive deadline".into(),
        budget_ticks: 1,
        steps: vec![Action::Until {
            condition: Condition::PatrolPhase {
                id: "hunter".into(),
                phase: patrol::Phase::Moving,
            },
            max_ticks: 1,
        }],
    };
    let result = scenario.run(&scene).unwrap();
    assert!(result.report.passed);
    assert_eq!(result.report.ticks, 1);
}
