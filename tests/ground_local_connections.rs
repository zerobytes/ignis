use ignis::{
    engine::{Command, Engine},
    mobility::Phase,
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::{Value, json};
fn box_(id: &str, p: [f32; 3], size: [f32; 3]) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":p,"size":size}})).unwrap()
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 30.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    player.transform.position = [-6., 0.8, 5.];
    player.footsteps = None;
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-1.5,0,0],[5.5,1.4558809,0]],"links":[{"from":0,"to":1,"mode":"walk"},{"from":1,"to":0,"mode":"walk"}]})).unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"npc","name":"npc","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[-1.5,0.81,0],"size":[0.6,1.6,0.6]},"character_controller":{"snap_distance":0.3,"step":{"max_height":0.3,"min_width":0.4}},"npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,"mobility":{"walk":true},"objectives":[{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[5.5,2.2558809,0],"arrival_distance":0.2}}]}})).unwrap());
    ramp(&mut s, 20.);
    npc_mut(&mut s).npc.as_mut().unwrap().local_connections =
        Some(serde_json::from_value(json!({"walk":true,"fly":false,"max_distance":8})).unwrap());
    s
}
fn ramp(s: &mut Scene, angle: f32) {
    let h = 4. * angle.to_radians().tan();
    let mut e = box_("ramp", [2., h / 2., 0.], [4., h, 4.]);
    e.geometry = serde_json::from_value(json!({"kind":"wedge","rise":"positive_x"})).unwrap();
    s.entities.retain(|e| e.id != "ramp" && e.id != "landing");
    s.entities.push(e);
    s.entities
        .push(box_("landing", [7., h / 2., 0.], [6., h, 4.]));
    let y = s.spatial_network.nodes[1][1];
    s.spatial_network.nodes[1][1] = h;
    let n = s
        .entities
        .iter_mut()
        .find(|e| e.id == "npc")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    if y != h {
        n.objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[5.5,h+0.8,0],"arrival_distance":0.2}}])).unwrap());
    }
}
fn npc_mut(s: &mut Scene) -> &mut Entity {
    s.entities.iter_mut().find(|e| e.id == "npc").unwrap()
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn flat() -> Scene {
    let mut s = scene();
    s.entities.retain(|e| e.id != "ramp" && e.id != "landing");
    s.spatial_network = Default::default();
    npc_mut(&mut s).transform.position = [-3., 0.81, 0.];
    npc_mut(&mut s).npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[3.,0.8,0.],"arrival_distance":0.2}}])).unwrap());
    s
}
fn arrived(e: &Engine) {
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?} {:?}",
        e.simulation.mobility.states["npc"],
        e.simulation.positions()
    );
}
#[test]
fn free_ramp_destination_without_graph_uses_physical_direct_connection() {
    let mut s = scene();
    s.spatial_network = Default::default();
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].connector.unwrap().kind,
        ignis::mobility::ConnectorKind::Direct
    );
    let mut prev = e.simulation.positions()["npc"];
    for _ in 0..300 {
        e.advance(1).unwrap();
        let now = e.simulation.positions()["npc"];
        assert!(glam::Vec3::from(now).distance(prev.into()) < 0.12);
        prev = now;
    }
    arrived(&e);
    let stats = e.simulation.mobility.states["npc"].ground_planning.unwrap();
    assert_eq!(stats.probes, 1);
    assert!(!stats.limited);
    replay(&e);
}
#[test]
fn physical_entry_graph_detour_and_exit_reach_off_node_destination() {
    let mut s = flat();
    npc_mut(&mut s).transform.position = [-4., 0.81, 0.];
    npc_mut(&mut s)
        .npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .max_distance = 2.85;
    s.entities.push(box_("wall", [0., 1.5, 0.], [1., 3., 2.5]));
    s.spatial_network = serde_json::from_value(
        json!({"nodes":[[-2,0,-2],[2,0,-2]],"links":[{"from":0,"to":1,"mode":"walk"}]}),
    )
    .unwrap();
    let mut e = Engine::new(s).unwrap();
    let mut kinds = std::collections::BTreeSet::new();
    let mut saw_edge = false;
    for _ in 0..600 {
        e.advance(1).unwrap();
        let state = &e.simulation.mobility.states["npc"];
        if let Some(c) = state.connector {
            kinds.insert(format!("{:?}", c.kind));
        }
        saw_edge |= state.edge.is_some();
    }
    arrived(&e);
    assert!(
        kinds.contains("Entry") && kinds.contains("Exit"),
        "{kinds:?}"
    );
    assert!(saw_edge);
    replay(&e);
}
#[test]
fn budget_is_visible_bounded_replayable_and_does_not_spin_while_waiting() {
    let mut s = flat();
    npc_mut(&mut s).npc.as_mut().unwrap().speed = 0.1;
    for i in 0..12 {
        s.spatial_network.nodes.push([-2. + i as f32 * 0.2, 0., 1.]);
    }
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let state = &e.simulation.mobility.states["npc"];
    let stats = state.ground_planning.unwrap();
    assert!(stats.limited);
    assert!(stats.probes <= 8 && stats.steps <= 2400);
    assert_eq!(state.reason, "ground_local_budget");
    let plans = state.plans;
    e.advance(20).unwrap();
    assert_eq!(e.simulation.mobility.states["npc"].plans, plans);
    replay(&e);
}
#[test]
fn complete_local_failure_is_distinct_from_candidate_truncation() {
    let mut s = flat();
    s.entities.push(box_("wall", [0., 1.5, 0.], [1., 3., 12.]));
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let state = &e.simulation.mobility.states["npc"];
    assert_eq!(state.reason, "no_local_route");
    assert!(!state.ground_planning.unwrap().limited);
}
#[test]
fn lock_cast_freezes_active_connector_deadline_then_resumes() {
    let mut s = scene();
    s.spatial_network = Default::default();
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[{"id":"hold","selection":"self_actor","range":0,"cast_ticks":150,"cooldown_ticks":0,"movement":{"mode":"lock"},"costs":[],"effects":[{"kind":"noise","radius":3}]}]})).unwrap();
    npc_mut(&mut s).abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["hold"]})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(95).unwrap();
    let state = &e.simulation.mobility.states["npc"];
    let connector = state.connector;
    let started = state.started_tick;
    assert!(connector.is_some());
    let p = e.simulation.positions()["npc"];
    run(
        &mut e,
        Command::ActivateAbility {
            actor: "npc".into(),
            slot: 1,
            target: None,
        },
    );
    e.advance(140).unwrap();
    let state = &e.simulation.mobility.states["npc"];
    assert_eq!(state.connector, connector);
    assert!(state.started_tick >= started + 139);
    assert!((e.simulation.positions()["npc"][0] - p[0]).abs() < 0.1);
    assert_eq!(
        e.simulation.character_controllers["npc"].requested_velocity,
        [0., 0.]
    );
    e.advance(400).unwrap();
    arrived(&e);
    replay(&e);
}
#[test]
fn old_format_and_local_flight_reject_without_mutation() {
    let mut s = scene();
    s.version = 84;
    assert!(s.validate().is_err());
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    let mut n = npc_mut(&mut flat()).clone();
    n.npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .fly = true;
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(n)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
}
#[test]
fn limits_steps_gaps_and_headroom_apply_to_connectors() {
    for (angle, limit, passes) in [(20., 30., true), (40., 30., false), (40., 45., true)] {
        let mut s = scene();
        ramp(&mut s, angle);
        s.spatial_network = Default::default();
        let c = npc_mut(&mut s).character_controller.as_mut().unwrap();
        c.max_slope_degrees = limit;
        c.slide_slope_degrees = 50.;
        let mut e = Engine::new(s).unwrap();
        e.advance(480).unwrap();
        assert_eq!(
            e.simulation.mobility.states["npc"].phase == Phase::Arrived,
            passes,
            "{:?}",
            e.simulation.mobility.states["npc"]
        );
    }
    for h in [0.2, 0.5] {
        let mut s = flat();
        s.entities.push(box_("step", [3., h / 2., 0.], [6., h, 4.]));
        npc_mut(&mut s).npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[3.,h+0.8,0.],"arrival_distance":0.2}}])).unwrap());
        let mut e = Engine::new(s).unwrap();
        e.advance(360).unwrap();
        assert_eq!(
            e.simulation.mobility.states["npc"].phase == Phase::Arrived,
            h < 0.3
        );
    }
}
#[test]
fn diagnostics_are_replay_checked_and_reset_and_undo_rebuild() {
    let mut e = Engine::new(flat()).unwrap();
    e.advance(30).unwrap();
    let replay0 = e.replay();
    let prefix = Engine::replay_at(&replay0, Some(10)).unwrap();
    assert!(
        prefix.simulation.mobility.states["npc"]
            .ground_planning
            .is_some()
    );
    let mut changed = serde_json::to_value(&replay0).unwrap();
    changed["expected_mobility"]["npc"]["ground_planning"]["probes"] = json!(8);
    let changed = serde_json::from_value(changed).unwrap();
    assert!(
        !Engine::verify_replay(&changed).unwrap()["matched"]
            .as_bool()
            .unwrap()
    );
    let mut wall = box_("wall", [0., 1.5, 0.], [1., 3., 12.]);
    wall.color = [0.5, 0.5, 0.5];
    run(
        &mut e,
        Command::Upsert {
            entity: Box::new(wall),
        },
    );
    assert!(
        e.simulation.mobility.states["npc"]
            .ground_planning
            .is_none()
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.mobility.states["npc"].reason, "no_local_route");
    run(&mut e, Command::Undo);
    e.advance(300).unwrap();
    arrived(&e);
    run(&mut e, Command::Reset);
    assert!(
        e.simulation.mobility.states["npc"]
            .ground_planning
            .is_none()
    );
}
#[test]
fn local_connectors_never_reveal_hidden_player() {
    let mut s = scene();
    let n = npc_mut(&mut s).npc.as_mut().unwrap();
    n.objectives = None;
    n.target = Some("player".into());
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":30,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":30,"search_turn_degrees_per_second":0})).unwrap());
    s.entities
        .push(box_("occlusion", [-4., 2., 2.5], [1., 4., 10.]));
    let mut e = Engine::new(s).unwrap();
    e.advance(180).unwrap();
    let state = &e.simulation.mobility.states["npc"];
    assert!(state.destination.is_none() && state.ground_planning.is_none());
    assert!(!e.simulation.npcs["npc"].observed_this_tick);
    replay(&e);
}
#[test]
fn pursuit_tracks_observed_player_outside_network_and_commits_current_segment() {
    let mut s = flat();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [3., 0.8, 0.];
    let n = npc_mut(&mut s).npc.as_mut().unwrap();
    n.objectives = None;
    n.target = Some("player".into());
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":30,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":30,"search_turn_degrees_per_second":0})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    let endpoint = e.simulation.mobility.states["npc"]
        .connector
        .unwrap()
        .endpoint;
    run(
        &mut e,
        Command::SetInput {
            input: ignis::physics::Input {
                x: 0.,
                z: 1.,
                ..Default::default()
            },
        },
    );
    e.advance(25).unwrap();
    let state = &e.simulation.mobility.states["npc"];
    assert_eq!(state.connector.unwrap().endpoint, endpoint);
    assert!(state.committed);
    run(
        &mut e,
        Command::SetInput {
            input: Default::default(),
        },
    );
    e.advance(500).unwrap();
    arrived(&e);
    assert!(e.simulation.mobility.states["npc"].destination.unwrap()[2] > 0.5);
    replay(&e);
}
#[test]
fn templates_reuse_geometry_but_never_connector_diagnostics_or_identity() {
    let mut s = flat();
    let mut t = npc_mut(&mut s).clone();
    t.id = "template".into();
    s.npc_templates = vec![t];
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        serde_json::from_value(
            json!({"op":"spawn_actor","template":"template","position":[-5,0.8,3],"target":null}),
        )
        .unwrap(),
    );
    e.advance(2).unwrap();
    let id = e
        .simulation
        .mobility
        .states
        .keys()
        .find(|id| id.as_str() != "npc")
        .unwrap()
        .clone();
    let old = e.simulation.character_controllers[&id].instance;
    assert!(e.simulation.mobility.states[&id].ground_planning.is_some());
    run(
        &mut e,
        serde_json::from_value(json!({"op":"despawn_actor","actor":id})).unwrap(),
    );
    e.advance(1).unwrap();
    assert!(!e.simulation.mobility.states.contains_key(&id));
    run(
        &mut e,
        serde_json::from_value(
            json!({"op":"spawn_actor","template":"template","position":[-5,0.8,3],"target":null}),
        )
        .unwrap(),
    );
    e.advance(1).unwrap();
    let new = e
        .simulation
        .mobility
        .states
        .keys()
        .find(|id| id.as_str() != "npc")
        .unwrap();
    assert_ne!(e.simulation.character_controllers[new].instance, old);
    assert!(e.simulation.mobility.states[new].plans <= 1);
    replay(&e);
}
#[test]
fn changing_known_target_does_not_bypass_failed_ground_search_cooldown() {
    let mut s = flat();
    let n = npc_mut(&mut s).npc.as_mut().unwrap();
    n.objectives = None;
    n.target = Some("player".into());
    n.awareness = Some(serde_json::from_value(json!({"mode":"omniscient"})).unwrap());
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [3., 0.8, 0.];
    s.entities.push(box_("wall", [0., 1.5, 0.], [1., 3., 12.]));
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    let plans = e.simulation.mobility.states["npc"].plans;
    let dest = e.simulation.mobility.states["npc"].destination;
    run(
        &mut e,
        Command::SetInput {
            input: ignis::physics::Input {
                x: 0.,
                z: 1.,
                ..Default::default()
            },
        },
    );
    e.advance(25).unwrap();
    let state = &e.simulation.mobility.states["npc"];
    assert_ne!(state.destination, dest);
    assert_eq!(state.plans, plans);
    replay(&e);
}
#[test]
fn body_height_and_missing_floor_reject_local_shortcuts() {
    for (height, passes) in [(1.6, true), (2.4, false)] {
        let mut s = flat();
        let n = npc_mut(&mut s);
        n.transform.size[1] = height;
        n.transform.position[1] = height / 2. + 0.01;
        n.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[3.,height/2.,0.],"arrival_distance":0.2}}])).unwrap());
        s.entities
            .push(box_("ceiling", [0., 2.3, 0.], [3., 0.2, 4.]));
        let mut e = Engine::new(s).unwrap();
        e.advance(360).unwrap();
        assert_eq!(
            e.simulation.mobility.states["npc"].phase == Phase::Arrived,
            passes
        );
    }
    let mut s = flat();
    let floor = s.entities.iter_mut().find(|e| e.id == "floor").unwrap();
    floor.transform.position[0] = -7.;
    floor.transform.size[0] = 12.;
    s.entities
        .push(box_("landing", [5., -0.25, 0.], [8., 0.5, 20.]));
    let mut e = Engine::new(s).unwrap();
    e.advance(240).unwrap();
    assert_eq!(e.simulation.mobility.states["npc"].completed, 0);
    assert!(e.simulation.positions()["npc"][0] < -2.9);
    replay(&e);
}
#[test]
fn anonymous_investigation_preserves_measured_distance_budget() {
    let mut s = flat();
    let n = npc_mut(&mut s).npc.as_mut().unwrap();
    n.objectives = None;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":0.1,"fov_degrees":90,"initial_facing_degrees":0,"memory_ticks":300,"search_ticks":30,"search_turn_degrees_per_second":0,"max_investigation_distance":3,"hearing":{"range":20}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    run(&mut e,serde_json::from_value(json!({"op":"emit_noise","origin":{"kind":"position","position":[3,0.8,0]},"radius":20})).unwrap());
    e.advance(2).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].reason,
        "insufficient_travel_budget"
    );
    assert_eq!(e.simulation.mobility.states["npc"].completed, 0);
    replay(&e);
}
#[test]
fn partial_connection_never_claims_arrival_at_blocked_goal() {
    let mut s = flat();
    s.entities.push(box_("wall", [0., 1.5, 0.], [1., 3., 12.]));
    s.spatial_network = serde_json::from_value(json!({"nodes":[[-1,0,0]],"links":[]})).unwrap();
    npc_mut(&mut s).npc.as_mut().unwrap().unreachable_policy = Some(
        serde_json::from_value(
            json!({"mode":"approach_reachable","min_progress":0.5,"max_path_ticks":1000}),
        )
        .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    e.advance(300).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::PartialReached,
        "{:?}",
        e.simulation.mobility.states["npc"]
    );
    let plans = e.simulation.mobility.states["npc"].plans;
    e.advance(100).unwrap();
    assert_eq!(e.simulation.mobility.states["npc"].plans, plans);
    replay(&e);
}
