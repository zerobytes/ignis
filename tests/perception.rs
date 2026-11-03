use ignis::{
    engine::{Command, Engine},
    model::{Awareness, RoomRecipe, Scene},
    navigation,
    perception::{self, Phase, Source},
};
use std::collections::BTreeMap;

fn scene() -> Scene {
    let mut value = serde_json::to_value(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    let entities = value["entities"].as_array_mut().unwrap();
    entities.iter_mut().find(|e| e["id"] == "player").unwrap()["transform"]["position"] =
        serde_json::json!([0, 0.8, -4]);
    entities.push(serde_json::json!({"id":"hunter","name":"Hunter","body":"npc","transform":{"position":[0,0.8,0],"size":[0.7,1.6,0.7]},"color":[0.7,0.2,0.8],"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":1.2,"enabled":true}}));
    serde_json::from_value(value).unwrap()
}
fn wall(s: &mut Scene, height: f32, body: serde_json::Value, blocks: Option<bool>) {
    s.entities.push(serde_json::from_value(serde_json::json!({"id":"screen","name":"Screen","body":body,"transform":{"position":[0,height/2.,-2],"size":[6,height,0.3]},"color":[0.2,0.6,0.3],"blocks_vision":blocks})).unwrap());
}
fn policy(s: &mut Scene) -> &mut Option<Awareness> {
    &mut s
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
}
fn config(memory_ticks: u32, search_ticks: u32) -> Awareness {
    Awareness::Perceptive {
        vision_range: 20.,
        fov_degrees: 360.,
        initial_facing_degrees: 0.,
        memory_ticks: Some(memory_ticks),
        search_ticks: Some(search_ticks),
        max_investigation_distance: None,
        area_search: None,
        hearing: None,
        search_turn_degrees_per_second: 90.,
    }
}

#[test]
fn vision_range_fov_and_3d_occlusion_are_independent_of_collision() {
    let mut cases = Vec::new();
    cases.push((scene(), true));
    let mut behind = scene();
    behind
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position[2] = 4.;
    cases.push((behind, false));
    let mut far = scene();
    let mut short_range = Awareness::default();
    if let Awareness::Perceptive { vision_range, .. } = &mut short_range {
        *vision_range = 2.;
    }
    *policy(&mut far) = Some(short_range);
    cases.push((far, false));
    for (height, body, blocks, expected) in [
        (2., serde_json::json!("fixed"), None, false),
        (0.5, serde_json::json!("fixed"), None, true),
        (2., serde_json::json!("fixed"), Some(false), true),
        (2., serde_json::Value::Null, Some(true), false),
        (2., serde_json::Value::Null, None, true),
    ] {
        let mut s = scene();
        wall(&mut s, height, body, blocks);
        cases.push((s, expected));
    }
    for (s, expected) in cases {
        let mut e = Engine::new(s).unwrap();
        e.advance(1).unwrap();
        assert_eq!(
            e.simulation.npcs["hunter"].observed_this_tick,
            expected,
            "{:?}",
            e.snapshot()
        );
        assert_eq!(e.simulation.npcs["hunter"].memory.is_some(), expected);
    }
}

#[test]
fn hidden_target_motion_cannot_change_memory_decisions_or_navigation() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .transform
        .position = [0., 0.8, -4.];
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [0., 0.8, -2.];
    wall(&mut s, 2., serde_json::json!("fixed"), None);
    s.entities
        .iter_mut()
        .find(|e| e.id == "screen")
        .unwrap()
        .transform
        .position[2] = 0.;
    *policy(&mut s) = Some(config(600, 30));
    let e = Engine::new(s.clone()).unwrap();
    let mut a = e.simulation.npcs.clone();
    let mut b = a.clone();
    let mut na = e.simulation.navigation.clone();
    let mut nb = na.clone();
    let mut pa: BTreeMap<_, _> = s
        .entities
        .iter()
        .map(|e| (e.id.clone(), e.transform.position))
        .collect();
    let mut pb = pa.clone();
    let mut phases = Vec::new();
    let (mut ga, mut gb, mut va, mut vb) = (
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    );
    let (mut sa, mut sb) = (
        navigation::NavigationScratch::default(),
        navigation::NavigationScratch::default(),
    );
    for tick in 0..160 {
        if tick > 0 {
            pa.insert("player".into(), [-1., 0.8, 3.]);
            pb.insert("player".into(), [1. + (tick % 10) as f32 * 0.03, 0.8, 4.]);
        }
        perception::update(&s, &pa, tick, &na, &mut a, &mut ga);
        perception::update(&s, &pb, tick, &nb, &mut b, &mut gb);
        navigation::update(&s, &pa, tick, &ga, &mut na, &mut va, &mut sa);
        navigation::update(&s, &pb, tick, &gb, &mut nb, &mut vb, &mut sb);
        assert_eq!(a, b, "knowledge diverged at {tick}");
        assert_eq!(na, nb, "route diverged at {tick}");
        assert_eq!(va, vb);
        if tick > 0
            && let Some(memory) = &a["hunter"].memory
        {
            assert_eq!(memory.position, [0., 0.8, -2.]);
            assert_eq!(memory.tick, 0);
        }
        phases.push(a["hunter"].phase);
        for (p, states, velocity) in [(&mut pa, &mut a, &va), (&mut pb, &mut b, &vb)] {
            let v = velocity["hunter"];
            states.get_mut("hunter").unwrap().face_velocity(v);
            let pos = p.get_mut("hunter").unwrap();
            pos[0] += v[0] / 60.;
            pos[2] += v[1] / 60.;
        }
    }
    for phase in [
        Phase::Pursuing,
        Phase::Investigating,
        Phase::Searching,
        Phase::Lost,
    ] {
        assert!(phases.contains(&phase), "missing {phase:?}: {phases:?}");
    }
    assert_eq!(a["hunter"].reason, "search_budget_exhausted");
    pa.insert("player".into(), [0., 0.8, -4.]);
    perception::update(&s, &pa, 160, &na, &mut a, &mut ga);
    assert_eq!(a["hunter"].phase, Phase::Pursuing);
    assert_eq!(a["hunter"].memory.as_ref().unwrap().tick, 160);
}

#[test]
fn memory_expires_even_when_last_known_destination_is_unreachable() {
    let mut s = scene();
    *policy(&mut s) = Some(config(3, 30));
    let e = Engine::new(s.clone()).unwrap();
    let mut state = e.simulation.npcs.clone();
    let mut nav = e.simulation.navigation.clone();
    nav.get_mut("hunter").unwrap().status = "unreachable".into();
    let mut p = e.simulation.positions();
    let mut goals = BTreeMap::new();
    perception::update(&s, &p, 0, &nav, &mut state, &mut goals);
    p.insert("player".into(), [100., 0.8, 0.]);
    for tick in 1..=3 {
        perception::update(&s, &p, tick, &nav, &mut state, &mut goals);
    }
    assert_eq!(state["hunter"].phase, Phase::Lost);
    assert_eq!(state["hunter"].reason, "memory_expired");
    assert!(state["hunter"].memory.is_none());
}

#[test]
fn migration_defaults_validation_history_and_replay_include_knowledge() {
    let old: Scene = serde_json::from_str(include_str!("../examples/npc.scene.json")).unwrap();
    let mut legacy = Engine::new(old).unwrap();
    assert_eq!(legacy.scene.version, ignis::model::SCENE_VERSION);
    assert!(matches!(
        legacy
            .scene
            .entities
            .iter()
            .find(|e| e.id == "hunter")
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .awareness,
        Some(Awareness::Omniscient)
    ));
    legacy.advance(1).unwrap();
    assert_eq!(
        legacy.simulation.npcs["hunter"]
            .memory
            .as_ref()
            .unwrap()
            .source,
        Source::Omniscient
    );
    let mut s = scene();
    wall(&mut s, 2., serde_json::json!("fixed"), None);
    let mut e = Engine::new(s).unwrap();
    e.advance(10).unwrap();
    assert_eq!(e.simulation.npcs["hunter"].phase, Phase::Unaware);
    let original = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "hunter")
        .unwrap()
        .clone();
    let before = e.snapshot();
    let mut invalid = original.clone();
    invalid.npc.as_mut().unwrap().awareness = Some(config(0, 30));
    assert!(
        e.execute(
            Command::Upsert {
                entity: invalid.into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    let mut omni = original;
    omni.npc.as_mut().unwrap().awareness = Some(Awareness::Omniscient);
    e.execute(
        Command::Upsert {
            entity: omni.into(),
        }
        .into(),
    )
    .unwrap();
    e.advance(30).unwrap();
    assert_eq!(e.simulation.npcs["hunter"].phase, Phase::Pursuing);
    e.execute(Command::Undo.into()).unwrap();
    e.advance(10).unwrap();
    assert_eq!(e.simulation.npcs["hunter"].phase, Phase::Unaware);
    e.execute(Command::Redo.into()).unwrap();
    e.advance(30).unwrap();
    let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
    assert_eq!(
        Engine::verify_replay(&replay).unwrap()["npcs_matched"],
        true
    );
    let mut tampered = e.replay();
    tampered.expected_npcs.get_mut("hunter").unwrap().memory = None;
    assert_eq!(Engine::verify_replay(&tampered).unwrap()["matched"], false);
    let saved = serde_json::to_string(&e.scene).unwrap();
    assert_eq!(
        Engine::new(serde_json::from_str(&saved).unwrap())
            .unwrap()
            .scene,
        e.scene
    );
    e.reset();
    assert!(e.simulation.npcs["hunter"].memory.is_none());
}

fn budget_scene(memory: Option<u32>, search: Option<u32>, distance: Option<f32>) -> Scene {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-2., 0.85, 4.];
    let npc = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
    npc.transform.position = [-5., 0.85, 4.];
    npc.npc.as_mut().unwrap().speed = 1.;
    npc.npc.as_mut().unwrap().awareness = Some(Awareness::Perceptive {
        vision_range: 4.,
        fov_degrees: 360.,
        initial_facing_degrees: 90.,
        memory_ticks: memory,
        search_ticks: search,
        max_investigation_distance: distance,
        area_search: None,
        hearing: None,
        search_turn_degrees_per_second: 90.,
    });
    s
}

#[test]
fn distance_only_and_combined_budgets_stop_on_first_limit_and_replay() {
    for (memory, distance, expected) in [
        (None, 1., "travel_budget_exhausted"),
        (Some(600), 1., "travel_budget_exhausted"),
        (Some(5), 100., "memory_expired"),
    ] {
        let mut e = Engine::new(budget_scene(memory, None, Some(distance))).unwrap();
        e.advance(1).unwrap();
        e.input = ignis::physics::Input {
            x: 1.,
            z: 0.,
            ..Default::default()
        };
        for _ in 0..135 {
            e.advance(1).unwrap();
        }
        e.input = Default::default();
        let state = &e.simulation.npcs["hunter"];
        assert_eq!(state.reason, expected, "{state:?}");
        assert_eq!(state.phase, Phase::Lost);
        assert!(state.investigation_distance > 0.);
        assert!(state.investigation_distance <= distance + 0.0001);
        if expected == "travel_budget_exhausted" {
            assert!((state.investigation_distance - distance).abs() < 0.001);
        }
        assert!(state.memory.is_none());
        assert!(e.simulation.navigation["hunter"].target_position.is_none());
        let frozen = state.investigation_distance;
        e.advance(30).unwrap();
        assert_eq!(e.simulation.npcs["hunter"].investigation_distance, frozen);
        let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
        assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
        let mut tampered = e.replay();
        tampered
            .expected_npcs
            .get_mut("hunter")
            .unwrap()
            .investigation_distance += 1.;
        assert_eq!(
            Engine::verify_replay(&tampered).unwrap()["npcs_matched"],
            false
        );
        // A new actual observation starts a new episode, without preserving an exhausted budget.
        e.input = ignis::physics::Input {
            x: -1.,
            z: 0.,
            ..Default::default()
        };
        e.advance(150).unwrap();
        assert_eq!(e.simulation.npcs["hunter"].phase, Phase::Pursuing);
        assert_eq!(e.simulation.npcs["hunter"].investigation_distance, 0.);
        e.reset();
        assert_eq!(e.simulation.npcs["hunter"].investigation_distance, 0.);
    }
}

#[test]
fn travel_budget_measures_path_not_displacement_and_stationary_search_costs_zero() {
    let s = budget_scene(None, None, Some(1.1));
    let npc = s.entities.iter().find(|e| e.id == "hunter").unwrap();
    let mut state = perception::NpcState::new(npc.npc.as_ref().unwrap());
    state.phase = Phase::Investigating;
    assert!(!state.record_travel([0.3, 0.4], Some(1.1), 1));
    assert!(!state.record_travel([0.3, -0.4], Some(1.1), 2));
    assert!((state.investigation_distance - 1.).abs() < 1e-6); // net distance from origin is only 0.6m.
    state.phase = Phase::Searching;
    for tick in 3..600 {
        assert!(!state.record_travel([0., 0.], Some(1.1), tick));
    }
    assert_eq!(state.phase, Phase::Searching);
    assert!((state.investigation_distance - 1.).abs() < 1e-6);
    assert!(state.record_travel([0.1, 0.], Some(1.1), 600));
    assert_eq!(state.reason, "travel_budget_exhausted");
    assert_eq!(state.distance_accounted_tick, Some(600));
}

#[test]
fn optional_budgets_validate_atomically_and_v3_keeps_its_time_limits() {
    let mut e = Engine::new(budget_scene(Some(600), Some(180), None)).unwrap();
    let before = e.snapshot();
    for distance in [
        None,
        Some(0.),
        Some(-1.),
        Some(f32::NAN),
        Some(f32::INFINITY),
        Some(10001.),
    ] {
        let bad = budget_scene(None, None, distance);
        let npc = bad
            .entities
            .iter()
            .find(|e| e.id == "hunter")
            .unwrap()
            .clone();
        assert!(
            e.execute(Command::Upsert { entity: npc.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut legacy = budget_scene(Some(600), Some(180), None);
    legacy.version = 3;
    let loaded =
        Engine::new(serde_json::from_str(&serde_json::to_string(&legacy).unwrap()).unwrap())
            .unwrap();
    assert_eq!(loaded.scene.version, ignis::model::SCENE_VERSION);
    assert_eq!(
        loaded
            .scene
            .entities
            .iter()
            .find(|e| e.id == "hunter")
            .unwrap()
            .npc,
        legacy
            .entities
            .iter()
            .find(|e| e.id == "hunter")
            .unwrap()
            .npc
    );
    let mut incompatible = budget_scene(None, None, Some(1.));
    incompatible.version = 3;
    assert!(Engine::new(incompatible).is_err());
}

fn area_scene(radius: f32) -> Scene {
    let mut s = budget_scene(None, None, Some(100.));
    if let Some(Awareness::Perceptive { area_search, .. }) = policy(&mut s) {
        *area_search = Some(ignis::model::AreaSearch {
            radius,
            points: 4,
            scan_ticks: 12,
            seed: 90,
        });
    }
    s
}

#[test]
fn area_search_visits_points_skips_unreachable_and_replays() {
    for radius in [2., 20.] {
        let mut e = Engine::new(area_scene(radius)).unwrap();
        e.advance(1).unwrap();
        e.input = ignis::physics::Input {
            x: 1.,
            z: 0.,
            ..Default::default()
        };
        e.advance(135).unwrap();
        e.input = Default::default();
        let mut explored = false;
        let mut scanned = false;
        for _ in 0..2500 {
            e.advance(1).unwrap();
            let state = &e.simulation.npcs["hunter"];
            explored |= state.phase == Phase::Exploring;
            scanned |= state.reason == "scan_search_point";
            if state.phase == Phase::Lost {
                break;
            }
        }
        let state = &e.simulation.npcs["hunter"];
        assert_eq!(state.reason, "search_points_exhausted", "{state:?}");
        assert!(explored);
        assert_eq!(state.search_point_index, 4);
        assert_eq!(
            state.skipped_search_points,
            if radius == 20. { 4 } else { 0 }
        );
        assert_eq!(scanned, radius == 2.);
        assert!(state.search_point.is_none() && state.memory.is_none());
        let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
        assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
        let mut tampered = e.replay();
        tampered
            .expected_npcs
            .get_mut("hunter")
            .unwrap()
            .search_point_index = 0;
        assert_eq!(Engine::verify_replay(&tampered).unwrap()["matched"], false);
        e.reset();
        assert_eq!(e.simulation.npcs["hunter"].search_point_index, 0);
    }
}

#[test]
fn area_search_does_not_leak_hidden_motion_and_reacquisition_clears_episode() {
    let s = area_scene(2.);
    let e = Engine::new(s.clone()).unwrap();
    let (mut a, mut b) = (e.simulation.npcs.clone(), e.simulation.npcs.clone());
    let (mut na, mut nb) = (
        e.simulation.navigation.clone(),
        e.simulation.navigation.clone(),
    );
    let (mut pa, mut pb) = (e.simulation.positions(), e.simulation.positions());
    let (mut ga, mut gb, mut va, mut vb) = (
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    );
    let (mut sa, mut sb) = (
        navigation::NavigationScratch::default(),
        navigation::NavigationScratch::default(),
    );
    let mut exploring_tick = None;
    for tick in 0..1200 {
        if tick > 0 {
            pa.insert("player".into(), [50., 0.85, 0.]);
            pb.insert("player".into(), [60. + (tick % 10) as f32, 0.85, 2.]);
        }
        perception::update(&s, &pa, tick, &na, &mut a, &mut ga);
        perception::update(&s, &pb, tick, &nb, &mut b, &mut gb);
        navigation::update(&s, &pa, tick, &ga, &mut na, &mut va, &mut sa);
        navigation::update(&s, &pb, tick, &gb, &mut nb, &mut vb, &mut sb);
        assert_eq!(a, b);
        assert_eq!(na, nb);
        assert_eq!(va, vb);
        if a["hunter"].phase == Phase::Exploring {
            exploring_tick.get_or_insert(tick);
        }
        if exploring_tick.is_some_and(|start| tick - start >= 60) {
            break;
        }
        for (p, v) in [(&mut pa, &va), (&mut pb, &vb)] {
            let pos = p.get_mut("hunter").unwrap();
            pos[0] += v["hunter"][0] / 60.;
            pos[2] += v["hunter"][1] / 60.;
        }
    }
    let tick = exploring_tick.expect("must begin spatial search") + 60;
    let first_point = a["hunter"].search_point.unwrap();
    assert!((first_point[0] - 0.).abs() < 0.001); // initial memory x=-2, radius=2, seed=90.
    let mut other = a.clone();
    other.get_mut("hunter").unwrap().search_point = None;
    let mut other_scene = s.clone();
    if let Some(Awareness::Perceptive {
        area_search: Some(search),
        ..
    }) = policy(&mut other_scene)
    {
        search.seed = 180;
    }
    perception::update(&other_scene, &pa, tick + 1, &na, &mut other, &mut gb);
    assert_ne!(other["hunter"].search_point, Some(first_point));
    // An unreachable last-known region may still have reachable nearby search points.
    let blocked = other.get_mut("hunter").unwrap();
    blocked.phase = Phase::Investigating;
    blocked.search_started_tick = None;
    blocked.search_point = None;
    nb.get_mut("hunter").unwrap().status = "unreachable".into();
    nb.get_mut("hunter").unwrap().target_position = Some(blocked.memory.as_ref().unwrap().position);
    perception::update(&s, &pb, tick + 2, &nb, &mut other, &mut gb);
    assert_eq!(other["hunter"].phase, Phase::Exploring);
    assert_eq!(other["hunter"].search_started_tick, Some(tick + 2));
    pa.insert("player".into(), pa["hunter"]);
    perception::update(&s, &pa, tick + 1, &na, &mut a, &mut ga);
    assert_eq!(a["hunter"].phase, Phase::Pursuing);
    assert!(a["hunter"].search_point.is_none());
    assert_eq!(a["hunter"].search_point_index, 0);
    assert_eq!(a["hunter"].investigation_distance, 0.);
}

#[test]
fn area_config_is_atomic_versioned_and_budgets_still_apply() {
    let mut e = Engine::new(area_scene(2.)).unwrap();
    let before = e.snapshot();
    for (radius, points, scan_ticks) in [
        (0., 4, 12),
        (21., 4, 12),
        (2., 0, 12),
        (2., 9, 12),
        (2., 4, 0),
        (2., 4, 3601),
    ] {
        let mut bad = area_scene(radius);
        if let Some(Awareness::Perceptive {
            area_search: Some(search),
            ..
        }) = policy(&mut bad)
        {
            search.points = points;
            search.scan_ticks = scan_ticks;
        }
        let entity = bad.entities.into_iter().find(|e| e.id == "hunter").unwrap();
        assert!(
            e.execute(
                Command::Upsert {
                    entity: entity.into()
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut old = budget_scene(Some(600), Some(180), None);
    old.version = 4;
    assert_eq!(
        Engine::new(old).unwrap().scene.version,
        ignis::model::SCENE_VERSION
    );
    let mut invalid_old = area_scene(2.);
    invalid_old.version = 4;
    assert!(Engine::new(invalid_old).is_err());
    for time in [false, true] {
        let mut s = area_scene(2.);
        if let Some(Awareness::Perceptive {
            search_ticks,
            max_investigation_distance,
            ..
        }) = policy(&mut s)
        {
            *search_ticks = time.then_some(5);
            *max_investigation_distance = Some(if time { 100. } else { 4.5 });
        }
        let mut e = Engine::new(s).unwrap();
        e.advance(1).unwrap();
        e.input = ignis::physics::Input {
            x: 1.,
            z: 0.,
            ..Default::default()
        };
        e.advance(135).unwrap();
        e.input = Default::default();
        let mut explored = false;
        for _ in 0..1200 {
            e.advance(1).unwrap();
            explored |= e.simulation.npcs["hunter"].phase == Phase::Exploring;
            if e.simulation.npcs["hunter"].phase == Phase::Lost {
                break;
            }
        }
        assert!(explored);
        assert_eq!(
            e.simulation.npcs["hunter"].reason,
            if time {
                "search_budget_exhausted"
            } else {
                "travel_budget_exhausted"
            }
        );
    }
}
