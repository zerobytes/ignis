use ignis::{
    engine::{Command, Engine},
    model::{Awareness, Hearing, Scene},
    noise::NoiseOrigin,
    patrol::{Config, Failure, Phase, Route, Traversal},
    physics::Input,
};

fn config() -> Config {
    Config {
        route: Route::Points {
            points: vec![[-2., 2.], [2., 2.], [2., 4.]],
        },
        traversal: Traversal::Loop,
        wait_ticks: 10,
        travel_ticks: 1200,
        retry_ticks: 60,
        arrival_distance: 0.2,
    }
}
fn scene() -> Scene {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    s.version = 9;
    let hunter = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
    hunter.transform.position = [-2., 0.85, 2.];
    let npc = hunter.npc.as_mut().unwrap();
    npc.speed = 3.;
    npc.patrol = Some(config());
    npc.awareness = Some(Awareness::Perceptive {
        vision_range: 0.1,
        fov_degrees: 120.,
        initial_facing_degrees: 0.,
        memory_ticks: Some(20),
        search_ticks: Some(10),
        search_turn_degrees_per_second: 90.,
        max_investigation_distance: None,
        area_search: None,
        hearing: None,
    });
    s
}
fn input(e: &mut Engine, x: f32) {
    e.execute(
        Command::SetInput {
            input: Input {
                x,
                z: 0.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
}

#[test]
fn patrol_visits_waits_and_reverses_without_acquiring_knowledge() {
    for (traversal, expected) in [
        (Traversal::Loop, vec![0, 1, 2, 0, 1]),
        (Traversal::PingPong, vec![0, 1, 2, 1, 0]),
    ] {
        let mut s = scene();
        s.entities
            .iter_mut()
            .find(|e| e.id == "hunter")
            .unwrap()
            .npc
            .as_mut()
            .unwrap()
            .patrol
            .as_mut()
            .unwrap()
            .traversal = traversal;
        let mut e = Engine::new(s).unwrap();
        let mut visited = vec![];
        let mut last_count = 0;
        for _ in 0..900 {
            e.advance(1).unwrap();
            let state = &e.simulation.patrols["hunter"];
            assert!(e.simulation.npcs["hunter"].memory.is_none());
            if state.visited != last_count {
                visited.push(state.point_index);
                last_count = state.visited;
                assert_eq!(state.phase, Phase::Waiting);
                let position = e.simulation.positions()["hunter"];
                e.advance(5).unwrap();
                assert_eq!(e.simulation.patrols["hunter"].phase, Phase::Waiting);
                let after = e.simulation.positions()["hunter"];
                assert!(
                    (after[0] - position[0]).abs() < 0.001
                        && (after[2] - position[2]).abs() < 0.001
                );
                if visited.len() == expected.len() {
                    break;
                }
            }
        }
        assert_eq!(visited, expected);
        assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
        let mut bad = e.replay();
        bad.expected_patrols.get_mut("hunter").unwrap().visited += 1;
        assert_eq!(
            Engine::verify_replay(&bad).unwrap()["patrols_matched"],
            false
        );
    }
}

#[test]
fn sound_suspends_patrol_and_resumes_same_point_without_hidden_target_influence() {
    let mut s = scene();
    if let Some(Awareness::Perceptive { hearing, .. }) = &mut s
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *hearing = Some(Hearing { range: 10. });
    }
    let mut a = Engine::new(s.clone()).unwrap();
    let mut b = Engine::new(s).unwrap();
    for e in [&mut a, &mut b] {
        e.advance(30).unwrap();
        assert_eq!(e.simulation.patrols["hunter"].point_index, 1);
        e.execute(
            Command::EmitNoise {
                origin: NoiseOrigin::Position {
                    position: [-2., 0.8, 4.],
                },
                radius: 8.,
            }
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
        assert_eq!(e.simulation.patrols["hunter"].phase, Phase::Suspended);
        assert_eq!(e.simulation.patrols["hunter"].point_index, 1);
    }
    input(&mut a, 1.);
    input(&mut b, -1.);
    for _ in 0..45 {
        a.advance(1).unwrap();
        b.advance(1).unwrap();
        assert_eq!(a.simulation.patrols, b.simulation.patrols);
        assert_eq!(a.simulation.npcs, b.simulation.npcs);
        assert_eq!(a.simulation.navigation, b.simulation.navigation);
    }
    assert!(a.simulation.npcs["hunter"].memory.is_none());
    assert_eq!(a.simulation.patrols["hunter"].phase, Phase::Moving);
    assert_eq!(a.simulation.patrols["hunter"].point_index, 1);
    assert_eq!(
        a.simulation.navigation["hunter"].target_position,
        Some(a.simulation.patrol_routes["hunter"][1])
    );
    assert_eq!(Engine::verify_replay(&a.replay()).unwrap()["matched"], true);
}

#[test]
fn unreachable_route_cools_down_and_travel_deadline_bounds_stuck_attempts() {
    let mut s = scene();
    let patrol = s
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .patrol
        .as_mut()
        .unwrap();
    patrol.route = Route::Points {
        points: vec![[100., 100.], [101., 101.], [102., 102.]],
    };
    patrol.traversal = Traversal::PingPong;
    let mut e = Engine::new(s).unwrap();
    // Resume at the middle: a ping-pong repeat must not count as a distinct failed point.
    e.simulation.patrols.get_mut("hunter").unwrap().point_index = 1;
    for _ in 0..30 {
        e.advance(1).unwrap();
        if e.simulation.patrols["hunter"].phase == Phase::Cooldown {
            break;
        }
    }
    let state = e.simulation.patrols["hunter"].clone();
    assert_eq!(state.phase, Phase::Cooldown);
    assert_eq!(state.failed_points, 0b111);
    assert_eq!(state.skipped, 4); // 1,2,1,0
    assert_eq!(state.last_failure, Some(Failure::Unreachable));
    assert!(e.simulation.navigation["hunter"].target_position.is_none());
    e.advance(40).unwrap();
    assert_eq!(e.simulation.patrols["hunter"], state);
    e.advance(22).unwrap();
    assert!(e.simulation.patrols["hunter"].skipped > state.skipped);

    let mut s = scene();
    let p = s
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .patrol
        .as_mut()
        .unwrap();
    p.route = Route::Points {
        points: vec![[5., 4.]],
    };
    p.travel_ticks = 2;
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    assert_eq!(e.simulation.patrols["hunter"].phase, Phase::Cooldown);
    assert_eq!(
        e.simulation.patrols["hunter"].last_failure,
        Some(Failure::TravelTimeout)
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}

#[test]
fn procedural_routes_are_seeded_and_validation_history_and_legacy_are_atomic() {
    let mut s = scene();
    let hunter = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
    hunter.npc.as_mut().unwrap().patrol.as_mut().unwrap().route = Route::Ring {
        center: [0., 3.],
        radius: 1.,
        points: 4,
        seed: 37,
    };
    let mut e = Engine::new(s.clone()).unwrap();
    assert_eq!(
        e.simulation.patrol_routes,
        Engine::new(s.clone()).unwrap().simulation.patrol_routes
    );
    let hunter = s
        .entities
        .iter()
        .find(|e| e.id == "hunter")
        .unwrap()
        .clone();
    let mut rotated = hunter.clone();
    if let Route::Ring { seed, .. } =
        &mut rotated.npc.as_mut().unwrap().patrol.as_mut().unwrap().route
    {
        *seed = 38;
    }
    e.execute(
        Command::Upsert {
            entity: rotated.into(),
        }
        .into(),
    )
    .unwrap();
    assert_ne!(
        e.simulation.patrol_routes,
        Engine::new(s.clone()).unwrap().simulation.patrol_routes
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        e.simulation.patrol_routes,
        Engine::new(s.clone()).unwrap().simulation.patrol_routes
    );
    for route in [
        Route::Points { points: vec![] },
        Route::Points {
            points: vec![[0., 0.]; 17],
        },
        Route::Points {
            points: vec![[f32::NAN, 0.]],
        },
        Route::Ring {
            center: [0., 0.],
            radius: 0.,
            points: 4,
            seed: 0,
        },
    ] {
        let mut bad = hunter.clone();
        bad.npc.as_mut().unwrap().patrol.as_mut().unwrap().route = route;
        let before = e.snapshot();
        assert!(
            e.execute(Command::Upsert { entity: bad.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    s.version = 8;
    assert!(Engine::new(s).is_err());
    let mut disabled = hunter;
    disabled.npc.as_mut().unwrap().enabled = false;
    e.execute(
        Command::Upsert {
            entity: disabled.into(),
        }
        .into(),
    )
    .unwrap();
    e.advance(20).unwrap();
    assert_eq!(e.simulation.patrols["hunter"].phase, Phase::Disabled);
    assert_eq!(e.simulation.patrols["hunter"].visited, 0);
    e.execute(Command::Undo.into()).unwrap();
    e.advance(60).unwrap();
    e.reset();
    assert_eq!(e.simulation.patrols["hunter"], Default::default());
    let legacy: Scene =
        serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    assert!(Engine::new(legacy).unwrap().simulation.patrols.is_empty());
}

#[test]
fn real_vision_preempts_routine_and_patrol_steps_are_not_self_heard() {
    let mut s = scene();
    let hunter = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
    hunter.footsteps = Some(ignis::model::FootstepConfig {
        surface: "floor".into(),
        stride: 0.3,
        radius: 8.,
        min_speed: 0.1,
    });
    if let Some(Awareness::Perceptive { hearing, .. }) = &mut hunter.npc.as_mut().unwrap().awareness
    {
        *hearing = Some(Hearing { range: 10. });
    }
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(70).unwrap();
    assert!(e.simulation.footsteps.total_emitted > 0);
    assert!(e.simulation.npcs["hunter"].memory.is_none());
    assert_ne!(e.simulation.patrols["hunter"].phase, Phase::Suspended);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let hunter = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
    if let Some(Awareness::Perceptive {
        vision_range,
        fov_degrees,
        ..
    }) = &mut hunter.npc.as_mut().unwrap().awareness
    {
        *vision_range = 20.;
        *fov_degrees = 360.;
    }
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [2., 0.85, 3.];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert!(e.simulation.npcs["hunter"].observed_this_tick);
    assert_eq!(e.simulation.patrols["hunter"].phase, Phase::Suspended);
    assert_eq!(e.simulation.patrols["hunter"].visited, 0);
    assert_eq!(
        e.simulation.navigation["hunter"].target_position,
        e.simulation.npcs["hunter"]
            .memory
            .as_ref()
            .map(|m| m.position)
    );
}
