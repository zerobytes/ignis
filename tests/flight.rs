use glam::Vec3;
use ignis::{
    engine::{Command, Engine},
    flight::{Profile, preview},
    mobility::Phase,
    model::{BodyKind, RoomRecipe, Scene},
};
use serde_json::json;
fn profile(hover: bool) -> Profile {
    serde_json::from_value(json!({"mode":"curved","acceleration":3,"deceleration":3,"turn_degrees_per_second":90,"smoothing_seconds":0.2,"minimum_speed":if hover {0.} else {1.},"can_hover":hover,"initial_direction":[0,0,-1]})).unwrap()
}
fn scene(hover: bool) -> Scene {
    let mut s = RoomRecipe {
        width: 32.,
        depth: 32.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    s.entities.push(serde_json::from_value(json!({"id":"flyer","name":"flyer","body":"npc","color":[0.3,0.5,0.9],"transform":{"position":[-5,3.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":3,"stop_distance":0.3,"enabled":true,"mobility":{"walk":false,"jump_speed":null,"fly_speed":3,"glide":null},"flight":profile(hover),"local_connections":{"walk":false,"fly":true,"max_distance":8},"objectives":[{"priority":1,"when":"always","action":{"kind":"guard_3d","point":[2,5.8,0],"arrival_distance":0.3}}]}})).unwrap());
    s
}
fn step(e: &mut Engine, ticks: u32) {
    e.execute(Command::Step { ticks }.into()).unwrap();
}
#[test]
fn curved_flight_has_bounded_velocity_acceleration_turn_and_stops_physically() {
    let mut e = Engine::new(scene(true)).unwrap();
    let mut old = Vec3::ZERO;
    let mut lateral = 0_f32;
    for _ in 0..1200 {
        step(&mut e, 1);
        let s = &e.simulation.mobility.states["flyer"];
        assert_ne!(s.phase, Phase::Unreachable, "{s:?}");
        let v = s.flight.map(|f| Vec3::from(f.velocity)).unwrap_or_default();
        assert!(v.length() <= 3.0001, "{v:?}");
        assert!(
            v.distance(old) <= 3. / 60. + 0.0002,
            "accel {old:?} -> {v:?}"
        );
        if v.length() > 0.01 && old.length() > 0.01 {
            assert!(
                v.normalize().dot(old.normalize()).clamp(-1., 1.).acos()
                    <= 90_f32.to_radians() / 60. + 0.001,
                "turn"
            );
        }
        old = v;
        lateral = lateral.max(e.simulation.position("flyer").unwrap()[2].abs());
        if s.phase == Phase::Arrived {
            break;
        }
    }
    assert_eq!(
        e.simulation.mobility.states["flyer"].phase,
        Phase::Arrived,
        "{:?}",
        e.simulation.mobility.states["flyer"]
    );
    assert!(lateral > 0.3, "lateral={lateral}");
    let p = e.simulation.position("flyer").unwrap();
    step(&mut e, 60);
    assert_eq!(e.simulation.position("flyer").unwrap(), p);
    let replay = e.replay();
    assert_eq!(
        Engine::replay_at(&replay, None)
            .unwrap()
            .simulation
            .mobility
            .states,
        e.simulation.mobility.states
    );
}
#[test]
fn non_hover_flyer_passes_goal_then_keeps_a_bounded_validated_orbit() {
    let mut e = Engine::new(scene(false)).unwrap();
    let mut reached = false;
    let mut points = vec![];
    for _ in 0..1200 {
        step(&mut e, 1);
        let s = &e.simulation.mobility.states["flyer"];
        assert_ne!(s.phase, Phase::Unreachable, "{s:?}");
        if let Some(f) = s.flight
            && f.holding
        {
            reached = true;
            assert!(Vec3::from(f.velocity).length() >= 0.99);
            points.push(Vec3::from(e.simulation.position("flyer").unwrap()));
        }
    }
    assert!(reached);
    assert_eq!(e.simulation.mobility.states["flyer"].completed, 1);
    assert!(
        points.iter().any(|p| p.distance(points[0]) > 0.5),
        "orbit samples={points:?}"
    );
    assert!(points.iter().all(|p| p.distance(points[0]) < 8.));
    assert_eq!(
        Engine::replay_at(&e.replay(), None)
            .unwrap()
            .simulation
            .mobility
            .states,
        e.simulation.mobility.states
    );
}
#[test]
fn curve_obstruction_is_rejected_even_when_the_straight_segment_is_clear() {
    let mut s = scene(true);
    let mut cfg = serde_json::to_value(profile(true)).unwrap();
    cfg["turn_degrees_per_second"] = json!(35);
    let curved_profile: Profile = serde_json::from_value(cfg).unwrap();
    s.entities.last_mut().unwrap().npc.as_mut().unwrap().flight = Some(curved_profile);
    let mut path = vec![];
    preview(
        curved_profile.settings().unwrap(),
        3.,
        Vec3::new(-5., 3., 0.),
        Vec3::ZERO,
        Vec3::new(2., 5., 0.),
        0.3,
        &mut path,
        |_, _| true,
    )
    .unwrap();
    let point = path
        .into_iter()
        .min_by(|a, b| a[2].total_cmp(&b[2]))
        .unwrap();
    assert!(point[2] < -0.5, "{point:?}");
    s.entities.push(serde_json::from_value(json!({"id":"block","name":"block","body":"fixed","color":[1,0,0],"transform":{"position":[point[0],point[1]+0.8,point[2]],"size":[0.3,0.3,0.3]}})).unwrap());
    let mut curved = Engine::new(s.clone()).unwrap();
    step(&mut curved, 1);
    assert_eq!(
        curved.simulation.mobility.states["flyer"].reason,
        "flight_path_blocked"
    );
    s.entities
        .iter_mut()
        .find(|e| e.id == "flyer")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .flight = Some(Profile::Straight);
    let mut straight = Engine::new(s).unwrap();
    step(&mut straight, 200);
    assert_eq!(
        straight.simulation.mobility.states["flyer"].phase,
        Phase::Arrived
    );
}
#[test]
fn flight_parameters_change_the_path_and_invalid_profiles_are_atomic() {
    let c = profile(true).settings().unwrap();
    let mut a = vec![];
    let mut b = vec![];
    let p = Vec3::new(-5., 3., 0.);
    let q = Vec3::new(2., 5., 0.);
    preview(c, 3., p, Vec3::ZERO, q, 0.3, &mut a, |_, _| true).unwrap();
    let mut d = c;
    d.turn = 35_f32.to_radians();
    d.smoothing = 0.8;
    preview(d, 3., p, Vec3::ZERO, q, 0.3, &mut b, |_, _| true).unwrap();
    assert_ne!(a, b);
    let mut smoothed = c;
    smoothed.smoothing = 1.2;
    let mut smooth_path = vec![];
    preview(
        smoothed,
        3.,
        p,
        Vec3::ZERO,
        q,
        0.3,
        &mut smooth_path,
        |_, _| true,
    )
    .unwrap();
    assert_ne!(a, smooth_path);
    let mut e = Engine::new(scene(true)).unwrap();
    step(&mut e, 20);
    let before = e.snapshot();
    for (key, value) in [
        ("acceleration", json!(0)),
        ("deceleration", json!(-1)),
        ("turn_degrees_per_second", json!(500)),
        ("smoothing_seconds", json!(3)),
        ("minimum_speed", json!(4)),
        ("initial_direction", json!([0, 0, 0])),
    ] {
        let mut entity =
            serde_json::to_value(e.scene.entities.iter().find(|e| e.id == "flyer").unwrap())
                .unwrap();
        entity["npc"]["flight"][key] = value;
        let cmd: Command = serde_json::from_value(json!({"op":"upsert","entity":entity})).unwrap();
        assert!(e.execute(cmd.into()).is_err());
        assert_eq!(e.snapshot(), before);
    }
    let mut old = scene(true);
    old.version = 21;
    assert!(Engine::new(old).is_err());
}
fn acoustic(s: &mut Scene, memory: u32, budget: Option<f32>) {
    let n = s
        .entities
        .iter_mut()
        .find(|e| e.id == "flyer")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    n.objectives = None;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":0.1,"fov_degrees":90,"initial_facing_degrees":0,"search_turn_degrees_per_second":90,"memory_ticks":memory,"search_ticks":1,"max_investigation_distance":budget,"hearing":{"range":20}})).unwrap());
}
fn noise(e: &mut Engine, p: [f32; 3]) {
    e.execute(
        serde_json::from_value::<Command>(
            json!({"op":"emit_noise","origin":{"kind":"position","position":p},"radius":20}),
        )
        .unwrap()
        .into(),
    )
    .unwrap();
}
#[test]
fn curved_network_legs_reach_nodes_without_teleport_and_replay_prefix() {
    let mut s = scene(true);
    let actor = s.entities.last_mut().unwrap();
    actor
        .npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .max_distance = 3.;
    actor.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"guard_3d","point":[5,5.8,0],"arrival_distance":0.3}}])).unwrap());
    s.spatial_network = serde_json::from_value(
        json!({"nodes":[[-3,3,0],[3,5,0]],"links":[{"from":0,"to":1,"mode":"fly"}]}),
    )
    .unwrap();
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 60);
    let prefix = e.snapshot();
    step(&mut e, 1200);
    assert_eq!(
        e.simulation.mobility.states["flyer"].phase,
        Phase::Arrived,
        "{:?}",
        e.simulation.mobility.states["flyer"]
    );
    assert_eq!(e.simulation.mobility.states["flyer"].completed, 3);
    assert_eq!(
        Engine::replay_at(&e.replay(), Some(60)).unwrap().snapshot(),
        prefix
    );
    s.entities.reverse();
    let mut reversed = Engine::new(s).unwrap();
    step(&mut reversed, 1260);
    assert_eq!(reversed.simulation.positions(), e.simulation.positions());
}
#[test]
fn expired_or_changed_cues_finish_the_frozen_curve_and_budget_uses_curved_distance() {
    for hover in [true, false] {
        let mut s = scene(hover);
        acoustic(&mut s, 5, None);
        let mut e = Engine::new(s).unwrap();
        noise(&mut e, [2., 5.8, 0.]);
        step(&mut e, 20);
        assert!(e.simulation.mobility.states["flyer"].committed);
        assert_eq!(e.simulation.mobility.states["flyer"].completed, 0);
        step(&mut e, 700);
        assert_eq!(e.simulation.mobility.states["flyer"].phase, Phase::Idle);
        assert_eq!(e.simulation.mobility.states["flyer"].completed, 1);
        assert_eq!(
            e.simulation.mobility.states["flyer"]
                .flight
                .is_some_and(|f| f.holding),
            !hover
        );
    }
    let mut s = scene(true);
    acoustic(&mut s, 2000, None);
    let mut e = Engine::new(s.clone()).unwrap();
    noise(&mut e, [2., 5.8, 0.]);
    step(&mut e, 20);
    let endpoint = e.simulation.mobility.states["flyer"]
        .connector
        .unwrap()
        .endpoint;
    noise(&mut e, [2., 5.8, 4.]);
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["flyer"]
            .connector
            .unwrap()
            .endpoint,
        endpoint
    );
    let mut new_leg = false;
    for _ in 0..1000 {
        step(&mut e, 1);
        if e.simulation.mobility.states["flyer"]
            .connector
            .is_some_and(|c| c.endpoint[2] == 4.)
        {
            new_leg = true;
            break;
        }
    }
    assert!(new_leg);
    let mut path = vec![];
    let plan = preview(
        profile(true).settings().unwrap(),
        3.,
        Vec3::new(-5., 3., 0.),
        Vec3::ZERO,
        Vec3::new(2., 5., 0.),
        0.3,
        &mut path,
        |_, _| true,
    )
    .unwrap();
    assert!(plan.distance > Vec3::new(7., 2., 0.).length() - 0.3);
    let mut reference = Engine::new(s.clone()).unwrap();
    noise(&mut reference, [2., 5.8, 0.]);
    step(&mut reference, 1);
    let actual_distance = reference.simulation.mobility.states["flyer"]
        .motion
        .unwrap()
        .distance;
    acoustic(&mut s, 2000, Some(actual_distance + 0.01));
    let mut limited = Engine::new(s).unwrap();
    noise(&mut limited, [2., 5.8, 0.]);
    step(&mut limited, 1);
    assert_eq!(
        limited.simulation.mobility.states["flyer"].reason,
        "insufficient_travel_budget"
    );
}
#[test]
fn dynamic_collision_stops_safely_and_straight_profile_preserves_legacy_motion() {
    let mut s = scene(true);
    s.entities.push(serde_json::from_value(json!({"id":"crate","name":"crate","body":"dynamic","color":[0.4,0.4,0.4],"transform":{"position":[0,5,0],"size":[2,10,12]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    let mut failed = false;
    let mut p = Vec3::from(e.simulation.position("flyer").unwrap());
    for _ in 0..700 {
        step(&mut e, 1);
        let now = Vec3::from(e.simulation.position("flyer").unwrap());
        assert!(now.distance(p) < 0.1);
        p = now;
        if e.simulation.mobility.states["flyer"].reason == "flight_tracking_interrupted" {
            failed = true;
            break;
        }
    }
    assert!(failed, "{:?}", e.simulation.mobility.states["flyer"]);
    assert!(p.x < 0.);
    assert!(
        e.simulation
            .events
            .iter()
            .any(|v| v.a == "crate" || v.b == "crate")
    );
    let mut a = scene(true);
    a.entities.last_mut().unwrap().npc.as_mut().unwrap().flight = None;
    let mut b = a.clone();
    b.entities.last_mut().unwrap().npc.as_mut().unwrap().flight = Some(Profile::Straight);
    let mut a = Engine::new(a).unwrap();
    let mut b = Engine::new(b).unwrap();
    step(&mut a, 180);
    step(&mut b, 180);
    assert_eq!(a.simulation.positions(), b.simulation.positions());
    assert_eq!(a.simulation.mobility.states, b.simulation.mobility.states);
}
#[test]
fn flight_history_reset_limits_and_non_hover_unreachable_goal_preserve_contracts() {
    let mut e = Engine::new(scene(true)).unwrap();
    step(&mut e, 50);
    let mut actor = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "flyer")
        .unwrap()
        .clone();
    actor.npc.as_mut().unwrap().flight = Some(Profile::Straight);
    e.execute(
        Command::Upsert {
            entity: Box::new(actor),
        }
        .into(),
    )
    .unwrap();
    assert!(e.simulation.mobility.states["flyer"].flight.is_none());
    e.execute(Command::Undo.into()).unwrap();
    step(&mut e, 1);
    assert!(e.simulation.mobility.states["flyer"].flight.is_some());
    e.execute(Command::Redo.into()).unwrap();
    step(&mut e, 1);
    assert!(e.simulation.mobility.states["flyer"].flight.is_none());
    let mut s = scene(false);
    acoustic(&mut s, 2000, None);
    let mut e = Engine::new(s).unwrap();
    noise(&mut e, [2., 5.8, 0.]);
    for _ in 0..1000 {
        step(&mut e, 1);
        if e.simulation.mobility.states["flyer"]
            .flight
            .is_some_and(|f| f.holding)
        {
            break;
        }
    }
    assert!(
        e.simulation.mobility.states["flyer"]
            .flight
            .unwrap()
            .holding
    );
    noise(&mut e, [14., 5.8, 0.]);
    step(&mut e, 20);
    assert!(
        e.simulation.mobility.states["flyer"]
            .flight
            .unwrap()
            .holding
    );
    assert_eq!(
        e.simulation.mobility.states["flyer"].phase,
        Phase::Unreachable
    );
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.simulation.mobility.states["flyer"].flight.is_none());
    assert!(e.simulation.mobility.states["flyer"].flight_path.is_empty());
    let mut cfg = serde_json::to_value(profile(false)).unwrap();
    cfg["minimum_speed"] = json!(0);
    let invalid: Profile = serde_json::from_value(cfg).unwrap();
    assert!(
        invalid
            .validate(
                e.scene
                    .entities
                    .iter()
                    .find(|e| e.id == "flyer")
                    .unwrap()
                    .npc
                    .as_ref()
                    .unwrap()
                    .mobility
            )
            .is_err()
    );
}

#[test]
fn holding_space_is_required_and_prediction_work_is_bounded() {
    let mut s = scene(false);
    let mut path = vec![];
    preview(
        profile(false).settings().unwrap(),
        3.,
        Vec3::new(-5., 3., 0.),
        Vec3::ZERO,
        Vec3::new(2., 5., 0.),
        0.3,
        &mut path,
        |_, _| true,
    )
    .unwrap();
    let point = *path.iter().max_by(|a, b| a[0].total_cmp(&b[0])).unwrap();
    assert!(point[0] > 3.);
    s.entities.push(serde_json::from_value(json!({"id":"orbit_block","name":"orbit_block","body":"fixed","color":[1,0,0],"transform":{"position":[point[0],point[1]+0.8,point[2]],"size":[0.3,0.3,0.3]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["flyer"].reason,
        "flight_holding_blocked"
    );
    assert_eq!(e.simulation.mobility.states["flyer"].completed, 0);
    let mut c = profile(true).settings().unwrap();
    c.acceleration = 0.1;
    let first_count = std::cell::Cell::new(0);
    let mut guide = vec![];
    // A blocked path must stop the preview immediately, not keep enumerating candidates.
    let result = preview(
        c,
        3.,
        Vec3::new(-5., 3., 0.),
        Vec3::ZERO,
        Vec3::new(2., 5., 0.),
        0.3,
        &mut guide,
        |_, _| {
            first_count.set(first_count.get() + 1);
            false
        },
    );
    assert!(result.is_err());
    assert!(guide.len() <= ignis::flight::MAX_GUIDE_POINTS);
    assert_eq!(first_count.get(), 1);
    // Impossibly long travel is bounded even in an unobstructed world.
    let counter = std::cell::Cell::new(0);
    let result = preview(
        c,
        0.1,
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::X * 8.,
        0.1,
        &mut guide,
        |_, _| {
            counter.set(counter.get() + 1);
            true
        },
    );
    assert!(result.is_err());
    assert!(counter.get() <= ignis::flight::MAX_TRAVEL_TICKS);
    assert!(guide.len() <= ignis::flight::MAX_GUIDE_POINTS);
}
