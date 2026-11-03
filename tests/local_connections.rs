use ignis::{
    engine::{Command, Engine},
    mobility::{ConnectorKind, LocalConnections, Mode, Phase, UnreachablePolicy},
    model::{BodyKind, RoomRecipe, Scene},
};
use serde_json::json;
fn scene(fly: bool, goal: [f32; 3]) -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    s.entities.push(serde_json::from_value(json!({"id":"actor","name":"actor","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[-3,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":4,"stop_distance":0.2,"enabled":true,"mobility":{"walk":true,"jump_speed":null,"fly_speed":if fly {Some(3)}else{None},"glide":null},"local_connections":{"walk":!fly,"fly":fly,"max_distance":8},"objectives":[{"priority":1,"when":"always","action":{"kind":"guard_3d","point":goal,"arrival_distance":0.2}}]}})).unwrap());
    s
}
fn step(e: &mut Engine, n: u32) {
    e.execute(Command::Step { ticks: n }.into()).unwrap();
}
fn noise(e: &mut Engine, position: [f32; 3]) {
    e.execute(
        serde_json::from_value::<Command>(
            json!({"op":"emit_noise","origin":{"kind":"position","position":position},"radius":20}),
        )
        .unwrap()
        .into(),
    )
    .unwrap();
}
fn acoustic(s: &mut Scene, memory: u32, budget: Option<f32>) {
    let n = s
        .entities
        .iter_mut()
        .find(|e| e.id == "actor")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    n.objectives = None;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":0.1,"fov_degrees":90,"initial_facing_degrees":0,"search_turn_degrees_per_second":90,"memory_ticks":memory,"search_ticks":1,"max_investigation_distance":budget,"hearing":{"range":20}})).unwrap());
}
#[test]
fn direct_walk_and_free_xyz_flight_are_physical_and_replayable() {
    for fly in [false, true] {
        let mut e = Engine::new(scene(fly, [2., if fly { 3.8 } else { 0.8 }, 1.])).unwrap();
        step(&mut e, 1);
        let s = &e.simulation.mobility.states["actor"];
        assert_eq!(s.connector.unwrap().kind, ConnectorKind::Direct);
        assert_eq!(
            s.connector.unwrap().mode,
            if fly { Mode::Fly } else { Mode::Walk }
        );
        assert!(s.edge.is_none());
        let mut previous = e.simulation.position("actor").unwrap();
        for _ in 0..240 {
            step(&mut e, 1);
            let now = e.simulation.position("actor").unwrap();
            assert!(glam::Vec3::from(now).distance(previous.into()) < 0.2);
            previous = now;
        }
        assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
        assert_eq!(e.simulation.mobility.states["actor"].completed, 1);
        let replay = e.replay();
        let end = Engine::replay_at(&replay, None).unwrap();
        assert_eq!(end.simulation.mobility.states, e.simulation.mobility.states);
        assert_eq!(end.simulation.positions(), e.simulation.positions());
        assert!(
            Engine::replay_at(&replay, Some(20))
                .unwrap()
                .simulation
                .mobility
                .states["actor"]
                .connector
                .is_some()
        );
    }
}
#[test]
fn entry_authored_detour_and_exit_reach_an_off_network_goal() {
    let mut s = scene(false, [4., 0.8, 0.]);
    s.entities.last_mut().unwrap().transform.position = [-4., 0.8, 0.];
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .max_distance = 2.85;
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","color":[0.3,0.7,0.6],"transform":{"position":[0,1.5,0],"size":[1,3,2.5]}})).unwrap());
    s.spatial_network = serde_json::from_value(
        json!({"nodes":[[-2,0,-2],[2,0,-2]],"links":[{"from":0,"to":1,"mode":"walk"}]}),
    )
    .unwrap();
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"]
            .connector
            .unwrap()
            .kind,
        ConnectorKind::Entry
    );
    let mut exit = false;
    let mut network = false;
    for _ in 0..300 {
        step(&mut e, 1);
        let state = &e.simulation.mobility.states["actor"];
        exit |= state
            .connector
            .is_some_and(|c| c.kind == ConnectorKind::Exit);
        network |= state.edge.is_some();
    }
    assert!(
        exit && network,
        "exit={exit} network={network} {:?}",
        e.simulation.mobility.states["actor"]
    );
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 3);
    s.entities.reverse();
    let mut other = Engine::new(s).unwrap();
    step(&mut other, 301);
    assert_eq!(
        other.simulation.mobility.states,
        e.simulation.mobility.states
    );
}
#[test]
fn local_walk_can_connect_an_authored_jump_without_inventing_one() {
    let mut s = scene(false, [3., 1.8, 0.]);
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.mobility.as_mut().unwrap().jump_speed = Some(7.);
    n.local_connections.as_mut().unwrap().max_distance = 1.5;
    s.entities.push(serde_json::from_value(json!({"id":"landing","name":"landing","body":"fixed","color":[0.2,0.5,0.7],"transform":{"position":[3,0.5,0],"size":[4,1,4]}})).unwrap());
    s.spatial_network = serde_json::from_value(
        json!({"nodes":[[-2,0,0],[2,1,0]],"links":[{"from":0,"to":1,"mode":"jump"}]}),
    )
    .unwrap();
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 300);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 3);
    s.spatial_network.links.clear();
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "no_local_route"
    );
}
#[test]
fn range_wall_and_unsupported_ground_reject_local_paths() {
    let mut s = scene(false, [3., 0.8, 0.]);
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .max_distance = 2.;
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "no_local_route"
    );
    let mut s = scene(false, [3., 0.8, 0.]);
    s.entities.push(serde_json::from_value(json!({"id":"thin","name":"thin","body":"fixed","color":[0,1,0],"transform":{"position":[0,1,0],"size":[0.05,2,4]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "no_local_route"
    );
    let mut s = scene(false, [3., 0.8, 0.]);
    let floor = s.entities.iter_mut().find(|e| e.id == "floor").unwrap();
    floor.transform.position[0] = -3.;
    floor.transform.size[0] = 2.;
    let mut far = floor.clone();
    far.id = "far".into();
    far.transform.position[0] = 3.;
    s.entities.push(far);
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "no_local_route"
    );
}
#[test]
fn partial_path_retains_entry_and_detour_and_counts_entry_effort() {
    let mut s = scene(false, [3., 2.8, 0.]);
    let a = s.entities.last_mut().unwrap();
    a.transform.position = [-5., 0.8, 0.];
    let n = a.npc.as_mut().unwrap();
    n.local_connections.as_mut().unwrap().max_distance = 2.1;
    n.unreachable_policy = Some(UnreachablePolicy::ApproachReachable {
        min_progress: 0.25,
        max_path_ticks: 300,
    });
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-3,0,0],[-4,0,2],[0,0,2],[1,0,0]],"links":[{"from":0,"to":1,"mode":"walk"},{"from":1,"to":0,"mode":"walk"},{"from":1,"to":2,"mode":"walk"},{"from":2,"to":3,"mode":"walk"}]})).unwrap();
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"]
            .connector
            .unwrap()
            .kind,
        ConnectorKind::Entry
    );
    assert_eq!(
        e.simulation.mobility.states["actor"].partial_target,
        Some(3)
    );
    step(&mut e, 700);
    let a = &e.simulation.mobility.states["actor"];
    assert_eq!(a.phase, Phase::PartialReached);
    assert_eq!(a.completed, 4);
    assert_eq!(a.plans, 1);
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .unreachable_policy = Some(UnreachablePolicy::ApproachReachable {
        min_progress: 0.25,
        max_path_ticks: 20,
    });
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "no_partial_progress"
    );
}
#[test]
fn connector_commitment_expiry_budget_and_hidden_actor_do_not_change_known_goal() {
    let mut s = scene(false, [3., 0.8, 0.]);
    acoustic(&mut s, 2, None);
    let mut e = Engine::new(s).unwrap();
    noise(&mut e, [3., 0.8, 0.]);
    step(&mut e, 10);
    assert!(e.simulation.mobility.states["actor"].committed);
    step(&mut e, 200);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Idle);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 1);
    let mut s = scene(true, [-3., 4.8, 0.]);
    acoustic(&mut s, 600, Some(2.));
    let mut e = Engine::new(s).unwrap();
    noise(&mut e, [-3., 4.8, 0.]);
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "insufficient_travel_budget"
    );
    let mut s = scene(false, [3., 0.8, 0.]);
    acoustic(&mut s, 600, None);
    let mut e = Engine::new(s).unwrap();
    noise(&mut e, [3., 0.8, 0.]);
    step(&mut e, 5);
    noise(&mut e, [-3., 0.8, 0.]);
    step(&mut e, 1);
    assert!(e.simulation.mobility.states["actor"].committed);
    let mut arrived = false;
    for _ in 0..300 {
        step(&mut e, 1);
        arrived |= e.simulation.mobility.states["actor"].phase == Phase::Arrived;
    }
    assert!(arrived);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 2);
    let mut results = vec![];
    for x in [-5., 5.] {
        let mut s = scene(false, [3., 0.8, 0.]);
        acoustic(&mut s, 600, None);
        s.entities.push(serde_json::from_value(json!({"id":"hidden","name":"hidden","body":"player","color":[1,1,0],"transform":{"position":[x,0.8,5],"size":[0.6,1.6,0.6]}})).unwrap());
        s.entities
            .iter_mut()
            .find(|e| e.id == "actor")
            .unwrap()
            .npc
            .as_mut()
            .unwrap()
            .target = Some("hidden".into());
        let mut e = Engine::new(s).unwrap();
        noise(&mut e, [3., 0.8, 0.]);
        step(&mut e, 40);
        results.push((
            e.simulation.mobility.states.clone(),
            e.simulation.position("actor"),
        ));
    }
    assert_eq!(results[0], results[1]);
}
#[test]
fn dynamic_collision_times_out_and_modes_choose_least_cost() {
    let mut s = scene(true, [3., 4.8, 0.]);
    s.entities.push(serde_json::from_value(json!({"id":"crate","name":"crate","body":"dynamic","color":[0.4,0.4,0.4],"transform":{"position":[0,5,0],"size":[2,10,10]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 320);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 0);
    assert!(e.simulation.position("actor").unwrap()[0] < 0.);
    assert!(
        e.simulation
            .events
            .iter()
            .any(|ev| (ev.a == "actor" && ev.b == "crate") || (ev.b == "actor" && ev.a == "crate"))
    );
    for (speed, expected) in [(3., Mode::Walk), (4., Mode::Walk), (6., Mode::Fly)] {
        let mut s = scene(true, [3., 0.8, 0.]);
        let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
        n.local_connections.as_mut().unwrap().walk = true;
        n.mobility.as_mut().unwrap().fly_speed = Some(speed);
        let mut e = Engine::new(s).unwrap();
        step(&mut e, 1);
        assert_eq!(
            e.simulation.mobility.states["actor"]
                .connector
                .unwrap()
                .mode,
            expected
        );
    }
}
#[test]
fn configuration_atomicity_legacy_history_and_reset() {
    let mut e = Engine::new(scene(false, [3., 0.8, 0.])).unwrap();
    step(&mut e, 10);
    let before = e.snapshot();
    for config in [
        LocalConnections {
            walk: true,
            fly: false,
            max_distance: f32::NAN,
        },
        LocalConnections {
            walk: true,
            fly: false,
            max_distance: 8.1,
        },
        LocalConnections {
            walk: false,
            fly: false,
            max_distance: 2.,
        },
        LocalConnections {
            walk: false,
            fly: true,
            max_distance: 2.,
        },
    ] {
        let mut a = e.scene.entities.last().unwrap().clone();
        a.npc.as_mut().unwrap().local_connections = Some(config);
        assert!(
            e.execute(
                Command::Upsert {
                    entity: Box::new(a)
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut old = e.scene.clone();
    old.version = 20;
    assert!(old.validate().is_err());
    old.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .local_connections = None;
    old.spatial_network.nodes = vec![[-3., 0., 0.], [3., 0., 0.]];
    old.spatial_network.links = vec![];
    let mut legacy = Engine::new(old).unwrap();
    step(&mut legacy, 1);
    assert_eq!(
        legacy.simulation.mobility.states["actor"].reason,
        "no_compatible_route"
    );
    let mut a = e.scene.entities.last().unwrap().clone();
    a.npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .max_distance = 1.;
    e.execute(
        Command::Upsert {
            entity: Box::new(a),
        }
        .into(),
    )
    .unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "no_local_route"
    );
    e.execute(Command::Undo.into()).unwrap();
    step(&mut e, 1);
    assert!(e.simulation.mobility.states["actor"].connector.is_some());
    e.execute(Command::Redo.into()).unwrap();
    assert!(e.simulation.mobility.states["actor"].connector.is_none());
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.simulation.mobility.states["actor"].plans, 0);
}

#[test]
fn nearby_anchor_aligns_physically_before_a_range_limited_exit() {
    let mut s = scene(false, [1., 0.8, 0.]);
    s.entities.last_mut().unwrap().transform.position = [-3.1, 0.8, 0.];
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .max_distance = 4.;
    s.spatial_network.nodes = vec![[-3., 0., 0.]];
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"]
            .connector
            .unwrap()
            .kind,
        ConnectorKind::Entry
    );
    step(&mut e, 180);
    assert_eq!(
        e.simulation.mobility.states["actor"].phase,
        Phase::Arrived,
        "state={:?}; position={:?}",
        e.simulation.mobility.states["actor"],
        e.simulation.position("actor")
    );
    assert_eq!(e.simulation.mobility.states["actor"].completed, 2);

    // The shared 2 mm contact tolerance must not turn into a larger arrival-radius allowance.
    let mut outside = scene(false, [1.003, 0.8, 0.]);
    outside
        .entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .local_connections
        .as_mut()
        .unwrap()
        .max_distance = 4.;
    let mut outside = Engine::new(outside).unwrap();
    step(&mut outside, 1);
    assert_eq!(
        outside.simulation.mobility.states["actor"].phase,
        Phase::Unreachable
    );
}

#[test]
fn pursuit_connector_stops_at_goal_radius_before_touching_the_player() {
    let mut s = scene(false, [3., 0.8, 0.]);
    acoustic(&mut s, 600, None);
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.target = Some("player".into());
    n.stop_distance = 0.8;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":16,"fov_degrees":360,"initial_facing_degrees":0,"search_turn_degrees_per_second":90,"memory_ticks":600,"search_ticks":60})).unwrap());
    s.entities.push(serde_json::from_value(json!({"id":"player","name":"player","body":"player","color":[1,1,0],"transform":{"position":[3,0.8,0],"size":[0.6,1.6,0.6]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 140);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 1);
    assert!(e.simulation.position("actor").unwrap()[0] <= 2.27);
    assert_eq!(e.simulation.position("player").unwrap()[0], 3.);
    assert!(
        !e.simulation.events.iter().any(
            |ev| (ev.a == "actor" && ev.b == "player") || (ev.b == "actor" && ev.a == "player")
        )
    );
}
