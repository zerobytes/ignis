use glam::Vec3;
use ignis::{
    engine::{Command, Engine},
    model::{RoomRecipe, Scene},
    patrol::{self, Config, Phase, Route, Traversal},
};
use serde_json::json;
use std::collections::BTreeMap;

fn config(points: Vec<[f32; 3]>) -> Config {
    Config {
        route: Route::Points3d { points },
        traversal: Traversal::Loop,
        wait_ticks: 12,
        travel_ticks: 1200,
        retry_ticks: 60,
        arrival_distance: 0.2,
    }
}
fn base() -> Scene {
    let mut s = RoomRecipe {
        width: 32.,
        depth: 24.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities
        .retain(|e| e.body != Some(ignis::model::BodyKind::Player));
    s.entities.push(serde_json::from_value(json!({"id":"actor","name":"actor","body":"npc","color":[0.6,0.2,0.8],
        "transform":{"position":[-4,2.8,0],"size":[0.6,1.6,0.6]},
        "npc":{"surface":"floor","speed":4,"stop_distance":0.2,"enabled":true,
            "mobility":{"walk":false,"fly_speed":3},"local_connections":{"walk":false,"fly":true,"max_distance":8},
            "patrol":config(vec![[-4.,2.8,0.],[0.,4.8,0.],[4.,2.8,0.]])}})).unwrap());
    s
}
fn actor(s: &mut Scene) -> &mut ignis::model::Entity {
    s.entities.iter_mut().find(|e| e.id == "actor").unwrap()
}

#[test]
fn spatial_loops_pingpong_wait_and_replay_follow_real_heights() {
    for (traversal, expected) in [
        (Traversal::Loop, vec![0, 1, 2, 0, 1]),
        (Traversal::PingPong, vec![0, 1, 2, 1, 0]),
    ] {
        let mut s = base();
        actor(&mut s)
            .npc
            .as_mut()
            .unwrap()
            .patrol
            .as_mut()
            .unwrap()
            .traversal = traversal;
        let mut e = Engine::new(s).unwrap();
        let mut visits = vec![];
        let mut previous = 0;
        let mut waiting = None;
        for _ in 0..1600 {
            e.advance(1).unwrap();
            let state = &e.simulation.patrols["actor"];
            let position = Vec3::from(e.simulation.position("actor").unwrap());
            if state.visited > previous {
                visits.push(state.point_index);
                previous = state.visited;
                assert_eq!(state.phase, Phase::Waiting);
                let point = e.simulation.patrol_routes["actor"][state.point_index as usize];
                assert!(
                    position.distance(point.into()) < 0.203,
                    "{position:?} -> {point:?}"
                );
                waiting = Some(position);
            } else if state.phase == Phase::Waiting {
                assert!(position.distance(waiting.unwrap()) < 0.003);
            }
            if visits.len() == 5 {
                break;
            }
        }
        assert_eq!(visits, expected);
        assert_eq!(e.simulation.patrols["actor"].skipped, 0);
        assert!(e.simulation.npcs["actor"].memory.is_none());
        let replay = e.replay();
        assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    }
}

#[test]
fn tilted_procedural_ring_is_bounded_deterministic_and_flown() {
    let mut cfg = config(vec![]);
    let center = Vec3::new(0., 4., 0.);
    let normal = Vec3::new(0., 1., 1.).normalize();
    cfg.route = Route::Ring3d {
        center: center.to_array(),
        normal: normal.to_array(),
        radius: 2.,
        points: 6,
        seed: 27,
    };
    cfg.validate().unwrap();
    let route = cfg.resolve(-99.);
    assert_eq!(route, cfg.resolve(999.));
    for point in &route {
        let d = Vec3::from(*point) - center;
        assert!(d.dot(normal).abs() < 1e-5);
        assert!((d.length() - 2.).abs() < 1e-5);
    }
    assert!(route.iter().any(|p| p[1] > 4.5) && route.iter().any(|p| p[1] < 3.5));
    let mut changed = cfg.clone();
    if let Route::Ring3d { seed, .. } = &mut changed.route {
        *seed = 28;
    }
    assert_ne!(route, changed.resolve(0.));
    let mut s = base();
    actor(&mut s).transform.position = route[0];
    actor(&mut s).npc.as_mut().unwrap().patrol = Some(cfg);
    let mut e = Engine::new(s).unwrap();
    e.advance(1200).unwrap();
    assert!(
        e.simulation.patrols["actor"].visited >= 6,
        "{:?}",
        e.simulation.patrols
    );
    assert_eq!(e.simulation.patrols["actor"].skipped, 0);
}

#[test]
fn jump_and_glide_routes_keep_capability_limits_and_do_not_teleport() {
    for mode in ["jump", "glide"] {
        let mut s = base();
        let (a, b) = if mode == "jump" {
            ([-2., 0., 0.], [2., 1., 0.])
        } else {
            ([-4., 4., 0.], [3., 0., 0.])
        };
        if mode == "jump" {
            s.entities.push(serde_json::from_value(json!({"id":"landing","name":"landing","body":"fixed","color":[0.4,0.4,0.4],"transform":{"position":[3,0.5,0],"size":[4,1,4]}})).unwrap());
        }
        let mut links = vec![json!({"from":0,"to":1,"mode":mode})];
        if mode == "jump" {
            links.push(json!({"from":1,"to":0,"mode":mode}));
        }
        s.spatial_network = serde_json::from_value(json!({"nodes":[a,b],"links":links})).unwrap();
        let e = actor(&mut s);
        e.transform.position = [a[0], a[1] + 0.8, a[2]];
        let n = e.npc.as_mut().unwrap();
        n.local_connections = None;
        n.mobility = Some(
            serde_json::from_value(if mode == "jump" {
                json!({"walk":false,"jump_speed":10})
            } else {
                json!({"walk":false,"glide":{"speed":4,"sink_speed":2}})
            })
            .unwrap(),
        );
        n.patrol = Some(config(vec![
            [b[0], b[1] + 0.8, b[2]],
            [a[0], a[1] + 0.8, a[2]],
        ]));
        let mut e = Engine::new(s).unwrap();
        let mut previous = Vec3::from(e.simulation.position("actor").unwrap());
        for _ in 0..900 {
            e.advance(1).unwrap();
            let p = Vec3::from(e.simulation.position("actor").unwrap());
            assert!(p.distance(previous) < 0.3, "teleport");
            previous = p;
        }
        let state = &e.simulation.patrols["actor"];
        if mode == "jump" {
            assert!(state.visited >= 4, "{state:?}");
            assert_eq!(state.skipped, 0);
        } else {
            assert!(state.visited >= 1, "{state:?}");
            assert!(state.skipped > 0, "glider must not climb back");
            assert!(previous.y < 1.);
        }
    }
}

#[test]
fn acoustic_interruption_preserves_index_and_committed_flight_then_resumes() {
    for with_objectives in [false, true] {
        let mut s = base();
        let n = actor(&mut s).npc.as_mut().unwrap();
        n.patrol = Some(config(vec![[2., 5.8, 0.], [-4., 2.8, 0.]]));
        n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":0.1,"fov_degrees":90,"initial_facing_degrees":0,"memory_ticks":20,"search_ticks":1,"search_turn_degrees_per_second":0,"hearing":{"range":20}})).unwrap());
        if with_objectives {
            n.objectives = Some(
                serde_json::from_value(json!([
                    {"priority":10,"when":"evidence","action":{"kind":"respond"}},
                    {"priority":1,"when":"always","action":{"kind":"patrol"}}
                ]))
                .unwrap(),
            );
        }
        let mut e = Engine::new(s).unwrap();
        e.advance(15).unwrap();
        assert_eq!(e.simulation.patrols["actor"].point_index, 0);
        e.execute(serde_json::from_value::<Command>(json!({"op":"emit_noise","origin":{"kind":"position","position":[-4,2.8,3]},"radius":20})).unwrap().into()).unwrap();
        e.advance(1).unwrap();
        assert_eq!(e.simulation.patrols["actor"].phase, Phase::Suspended);
        assert!(e.simulation.mobility.states["actor"].committed);
        assert_eq!(e.simulation.patrols["actor"].visited, 0);
        e.advance(30).unwrap();
        assert!(e.simulation.npcs["actor"].memory.is_none());
        assert_eq!(e.simulation.patrols["actor"].point_index, 0);
        assert_eq!(e.simulation.patrols["actor"].phase, Phase::Moving);
        let prefix = e.snapshot()["patrols"].clone();
        let tick = e.simulation.tick;
        e.advance(700).unwrap();
        assert!(e.simulation.patrols["actor"].visited >= 2);
        let replay = e.replay();
        assert_eq!(
            Engine::replay_at(&replay, Some(tick)).unwrap().snapshot()["patrols"],
            prefix
        );
        assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    }
}

#[test]
fn spatial_feedback_rejects_stale_wrong_altitude_partial_and_accepts_roundtrip_tolerance() {
    let e = Engine::new(base()).unwrap();
    for (status, offset, stamp, visit, skip) in [
        ("arrived", 0., 9, false, false),
        ("arrived", 0.01, 10, false, false),
        ("arrived", 0.0006, 10, true, false),
        ("unreachable", 0., 10, false, true),
    ] {
        let mut states = e.simulation.patrols.clone();
        let state = states.get_mut("actor").unwrap();
        state.phase = Phase::Moving;
        state.leg_started_tick = Some(10);
        let mut nav = e.simulation.navigation.clone();
        let n = nav.get_mut("actor").unwrap();
        n.status = status.into();
        n.decision_tick = stamp;
        let mut p = e.simulation.patrol_routes["actor"][0];
        p[1] += offset;
        n.target_position = Some(p);
        let mut goals = BTreeMap::from([("actor".to_string(), None)]);
        patrol::update(
            &e.scene,
            &e.simulation.patrol_routes,
            11,
            &mut states,
            &e.simulation.npcs,
            &nav,
            &mut goals,
        );
        assert_eq!(states["actor"].visited, u32::from(visit));
        assert_eq!(states["actor"].skipped, u32::from(skip));
    }
    let mut s = base();
    actor(&mut s)
        .npc
        .as_mut()
        .unwrap()
        .patrol
        .as_mut()
        .unwrap()
        .travel_ticks = 1;
    actor(&mut s)
        .npc
        .as_mut()
        .unwrap()
        .patrol
        .as_mut()
        .unwrap()
        .route = Route::Points3d {
        points: vec![[2., 5., 0.]],
    };
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    assert_eq!(
        e.simulation.patrols["actor"].last_failure,
        Some(patrol::Failure::TravelTimeout)
    );
    assert_eq!(e.simulation.patrols["actor"].phase, Phase::Cooldown);
}

#[test]
fn non_hover_patrol_wait_is_orbiting_not_fake_stationary_flight() {
    let mut s = base();
    let n = actor(&mut s).npc.as_mut().unwrap();
    n.patrol = Some(config(vec![[1., 4.8, 0.]]));
    n.patrol.as_mut().unwrap().wait_ticks = 180;
    n.flight=Some(serde_json::from_value(json!({"mode":"curved","acceleration":3,"deceleration":3,"turn_degrees_per_second":90,"smoothing_seconds":0.2,"minimum_speed":1,"can_hover":false,"initial_direction":[1,0,0]})).unwrap());
    let mut e = Engine::new(s).unwrap();
    for _ in 0..1200 {
        e.advance(1).unwrap();
        if e.simulation.patrols["actor"].phase == Phase::Waiting {
            break;
        }
    }
    assert_eq!(
        e.simulation.patrols["actor"].phase,
        Phase::Waiting,
        "{:?}",
        e.snapshot()
    );
    let p = Vec3::from(e.simulation.position("actor").unwrap());
    e.advance(60).unwrap();
    assert!(Vec3::from(e.simulation.position("actor").unwrap()).distance(p) > 0.1);
    assert_eq!(e.simulation.patrols["actor"].phase, Phase::Waiting);
    assert!(
        e.simulation.mobility.states["actor"]
            .flight
            .unwrap()
            .holding
    );
}

#[test]
fn validation_history_and_guides_preserve_spatial_contract_and_legacy() {
    let mut e = Engine::new(base()).unwrap();
    let before = e.snapshot();
    for route in [
        json!({"kind":"points_3d","points":[]}),
        json!({"kind":"points_3d","points":[[0,10001,0]]}),
        json!({"kind":"ring_3d","center":[0,4,0],"normal":[0,0,0],"radius":2,"points":4,"seed":1}),
        json!({"kind":"ring_3d","center":[9999,4,0],"normal":[0,1,0],"radius":2,"points":4,"seed":1}),
    ] {
        let mut entity = serde_json::to_value(actor(&mut e.scene.clone())).unwrap();
        entity["npc"]["patrol"]["route"] = route;
        assert!(
            e.execute(
                serde_json::from_value::<Command>(json!({"op":"upsert","entity":entity}))
                    .unwrap()
                    .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut old = base();
    old.version = 23;
    assert!(Engine::new(old).is_err());
    let mut no_cap = base();
    actor(&mut no_cap).npc.as_mut().unwrap().mobility = None;
    actor(&mut no_cap).npc.as_mut().unwrap().local_connections = None;
    assert!(Engine::new(no_cap).is_err());
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_debug_view","npc_routes":true}))
            .unwrap()
            .into(),
    )
    .unwrap();
    let frame = e.render_frame();
    let authored: Vec<_> = frame
        .route_guides
        .iter()
        .filter(|g| g.color == [0.55, 0.5, 0.85])
        .collect();
    assert!(authored.iter().any(|g| g.start[1] > 4.));
    let mut entity = actor(&mut e.scene.clone()).clone();
    entity.npc.as_mut().unwrap().patrol.as_mut().unwrap().route = Route::Points {
        points: vec![[0., 0.]],
    };
    e.execute(
        Command::Upsert {
            entity: Box::new(entity),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.simulation.patrol_routes["actor"][0][1], 0.8);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.simulation.patrol_routes["actor"].len(), 3);
    e.advance(30).unwrap();
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.simulation.patrols["actor"].visited, 0);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.simulation.patrol_routes["actor"].len(), 1);
}
