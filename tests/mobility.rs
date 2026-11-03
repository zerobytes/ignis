use ignis::{
    engine::{Command, Engine},
    mobility::{Mode, Phase},
    model::{RoomRecipe, Scene},
};
use serde_json::json;
fn base(mode: Mode) -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities
        .retain(|e| e.body != Some(ignis::model::BodyKind::Player));
    let (a, b) = match mode {
        Mode::Jump => ([-2., 0., 0.], [2., 1., 0.]),
        Mode::Glide => ([-4., 4., 0.], [3., 0., 0.]),
        Mode::Fly => ([-3., 0., 0.], [3., 4., 0.]),
        Mode::Walk => ([-3., 0., 0.], [3., 0., 0.]),
    };
    if mode == Mode::Jump {
        s.entities.push(serde_json::from_value(json!({"id":"landing","name":"landing","body":"fixed","color":[0.2,0.5,0.7],"transform":{"position":[3,0.5,0],"size":[4,1,4]}})).unwrap());
    }
    s.spatial_network =
        serde_json::from_value(json!({"nodes":[a,b],"links":[{"from":0,"to":1,"mode":mode}]}))
            .unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"actor","name":"actor","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[a[0],a[1]+0.8,a[2]],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":4,"stop_distance":0.2,"enabled":true,"mobility":{"walk":true,"jump_speed":7,"fly_speed":3,"glide":{"speed":4,"sink_speed":1}},"objectives":[{"priority":1,"when":"always","action":{"kind":"guard_3d","point":[b[0],b[1]+0.8,b[2]],"arrival_distance":0.2}}]}})).unwrap());
    s
}
fn step(e: &mut Engine, n: u32) {
    e.execute(Command::Step { ticks: n }.into()).unwrap();
}
#[test]
fn physical_modes_reach_xyz_goals_and_replay() {
    for mode in [Mode::Walk, Mode::Jump, Mode::Glide, Mode::Fly] {
        let mut e = Engine::new(base(mode)).unwrap();
        assert!(
            e.simulation.mobility.compiled["actor"][0].is_some(),
            "{mode:?}"
        );
        let initial = e.simulation.position("actor").unwrap();
        let mut highest = initial[1];
        let mut lowest = initial[1];
        let mut previous = initial;
        for _ in 0..500 {
            step(&mut e, 1);
            let p = e.simulation.position("actor").unwrap();
            highest = highest.max(p[1]);
            lowest = lowest.min(p[1]);
            assert!(
                glam::Vec3::from(p).distance(previous.into()) < 0.2,
                "teleport {mode:?}"
            );
            if mode == Mode::Glide {
                assert!(p[1] <= previous[1] + 0.001);
            }
            previous = p;
        }
        let state = &e.simulation.mobility.states["actor"];
        assert_eq!(
            state.phase,
            Phase::Arrived,
            "{mode:?} {:?} {:?}",
            state,
            e.simulation.positions()
        );
        assert_eq!(state.completed, 1);
        if mode == Mode::Jump {
            assert!(highest > initial[1] + 2.);
        }
        if mode == Mode::Glide {
            assert!(lowest < initial[1] - 3.5);
        }
        if mode == Mode::Fly {
            assert!(highest > initial[1] + 3.5);
        }
        let replay = e.replay();
        let restored = Engine::replay_at(&replay, None).unwrap();
        assert_eq!(e.simulation.positions(), restored.simulation.positions());
        assert_eq!(
            e.simulation.mobility.states,
            restored.simulation.mobility.states
        );
    }
}
#[test]
fn absent_capability_static_obstacle_and_upward_glide_cannot_be_used() {
    let mut s = base(Mode::Jump);
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .mobility
        .as_mut()
        .unwrap()
        .jump_speed = None;
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 180);
    assert_eq!(
        e.simulation.mobility.states["actor"].phase,
        Phase::Unreachable
    );
    assert_eq!(e.simulation.mobility.states["actor"].completed, 0);
    let mut s = base(Mode::Fly);
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","color":[0,1,0],"transform":{"position":[0,3,0],"size":[1,6,5]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    assert!(e.simulation.mobility.compiled["actor"][0].is_none());
    step(&mut e, 180);
    assert_eq!(
        e.simulation.mobility.states["actor"].phase,
        Phase::Unreachable
    );
    let mut s = base(Mode::Glide);
    s.spatial_network.nodes.swap(0, 1);
    assert!(s.validate().is_err());
}
#[test]
fn shortest_compatible_route_uses_intermediate_height_and_stable_ties() {
    let mut s = base(Mode::Fly);
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-3,0,0],[-3,6,0],[3,6,0],[3,4,0]],"links":[{"from":0,"to":3,"mode":"walk"},{"from":0,"to":1,"mode":"fly"},{"from":1,"to":2,"mode":"fly"},{"from":2,"to":3,"mode":"fly"}]})).unwrap();
    // Direct walk is structurally invalid; use a jump that this actor cannot perform.
    s.spatial_network.links[0].mode = Mode::Jump;
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .mobility
        .as_mut()
        .unwrap()
        .jump_speed = None;
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 1);
    assert_eq!(e.simulation.mobility.states["actor"].route, vec![1, 2, 3]);
    let mut max_y = 0_f32;
    for _ in 0..500 {
        step(&mut e, 1);
        max_y = max_y.max(e.simulation.position("actor").unwrap()[1]);
    }
    assert!(max_y > 6.6);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 3);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
    s.entities.reverse();
    let mut reversed = Engine::new(s).unwrap();
    step(&mut reversed, 501);
    assert_eq!(e.simulation.positions(), reversed.simulation.positions());
    assert_eq!(
        e.simulation.mobility.states,
        reversed.simulation.mobility.states
    );
}
#[test]
fn invalid_network_is_atomic_and_reset_undo_restore_runtime() {
    let mut e = Engine::new(base(Mode::Fly)).unwrap();
    step(&mut e, 50);
    let before = e.snapshot();
    let mut bad = e.scene.spatial_network.clone();
    bad.links[0].to = 63;
    assert!(
        e.execute(Command::SetSpatialNetwork { network: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut network = e.scene.spatial_network.clone();
    network.links.clear();
    e.execute(Command::SetSpatialNetwork { network }.into())
        .unwrap();
    assert_eq!(e.simulation.tick, 0);
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].phase,
        Phase::Unreachable
    );
    e.execute(Command::Undo.into()).unwrap();
    assert!(e.simulation.mobility.compiled["actor"][0].is_some());
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.simulation.mobility.states["actor"].completed, 0);
    let mut old = base(Mode::Walk);
    old.version = 18;
    assert!(old.validate().is_err());
}
#[test]
fn committed_jump_finishes_after_goal_expires_without_reading_hidden_target() {
    let mut s = base(Mode::Jump);
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.objectives = None;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":10,"fov_degrees":90,"initial_facing_degrees":0,"search_turn_degrees_per_second":90,"memory_ticks":1,"search_ticks":1,"hearing":{"range":20}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.execute(
        serde_json::from_value::<Command>(
            json!({"op":"emit_noise","origin":{"kind":"position","position":[2,1.8,0]},"radius":20}),
        )
        .unwrap().into(),
    )
    .unwrap();
    step(&mut e, 1);
    assert!(
        e.simulation.mobility.states["actor"].edge.is_some(),
        "{:?}",
        e.snapshot()
    );
    step(&mut e, 10);
    assert!(e.simulation.mobility.states["actor"].committed);
    step(&mut e, 200);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 1);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Idle);
}
#[test]
fn dynamic_blocker_keeps_collision_and_times_out_without_teleport() {
    let mut s = base(Mode::Fly);
    s.entities.push(serde_json::from_value(json!({"id":"crate","name":"crate","body":"dynamic","color":[0.4,0.4,0.4],"transform":{"position":[0,5,0],"size":[2,10,10]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    assert!(e.simulation.mobility.compiled["actor"][0].is_some());
    step(&mut e, 500);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 0);
    assert_eq!(
        e.simulation.mobility.states["actor"].phase,
        Phase::Unreachable
    );
    assert!(e.simulation.position("actor").unwrap()[0] < 0.);
    assert!(
        e.simulation
            .events
            .iter()
            .any(|ev| (ev.a == "actor" && ev.b == "crate") || (ev.b == "actor" && ev.a == "crate"))
    );
}
#[test]
fn spatial_budget_counts_vertical_motion_and_prevents_unaffordable_link() {
    let mut s = base(Mode::Fly);
    s.spatial_network.nodes = vec![[-3., 0., 0.], [-3., 4., 0.]];
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.objectives = None;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":10,"fov_degrees":90,"initial_facing_degrees":0,"search_turn_degrees_per_second":90,"memory_ticks":600,"search_ticks":600,"max_investigation_distance":6,"hearing":{"range":20}})).unwrap());
    let mut e = Engine::new(s.clone()).unwrap();
    let noise: Command = serde_json::from_value(
        json!({"op":"emit_noise","origin":{"kind":"position","position":[-3,4.8,0]},"radius":20}),
    )
    .unwrap();
    e.execute(noise.into()).unwrap();
    step(&mut e, 60);
    assert!(e.simulation.npcs["actor"].investigation_distance > 2.8);
    if let Some(ignis::model::Awareness::Perceptive {
        max_investigation_distance,
        ..
    }) = &mut s
        .entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *max_investigation_distance = Some(2.);
    }
    let mut e = Engine::new(s).unwrap();
    let noise: Command = serde_json::from_value(
        json!({"op":"emit_noise","origin":{"kind":"position","position":[-3,4.8,0]},"radius":20}),
    )
    .unwrap();
    e.execute(noise.into()).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "insufficient_travel_budget"
    );
    assert_eq!(e.simulation.mobility.states["actor"].completed, 0);
}
#[test]
fn unsupported_planar_combinations_and_out_of_network_goals_are_explicit() {
    let mut s = base(Mode::Fly);
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .passage_priority = Some(0);
    assert!(s.validate().is_err());
    let mut s = base(Mode::Fly);
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .objectives
        .as_mut()
        .unwrap()[0]
        .action = ignis::objectives::Action::Guard3d {
        point: [7., 8., 0.],
        arrival_distance: 0.2,
    };
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].reason,
        "target_not_on_network"
    );
}
#[test]
fn flight_lands_before_switching_to_a_grounded_jump() {
    let mut s = base(Mode::Jump);
    s.entities.last_mut().unwrap().transform.position = [-3., 0.8, 0.];
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-3,0,0],[-3,3,0],[-2,0,0],[2,1,0]],"links":[{"from":0,"to":1,"mode":"fly"},{"from":1,"to":2,"mode":"fly"},{"from":2,"to":3,"mode":"jump"}]})).unwrap();
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 500);
    assert_eq!(
        e.simulation.mobility.states["actor"].completed,
        3,
        "{:?} {:?}",
        e.simulation.mobility.states,
        e.simulation.positions()
    );
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
}

fn partial_scene() -> Scene {
    let mut s = base(Mode::Walk);
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.mobility = Some(
        serde_json::from_value(
            json!({"walk":true,"jump_speed":null,"fly_speed":null,"glide":null}),
        )
        .unwrap(),
    );
    n.unreachable_policy = Some(ignis::mobility::UnreachablePolicy::ApproachReachable {
        min_progress: 0.25,
        max_path_ticks: 300,
    });
    n.objectives.as_mut().unwrap()[0].action = ignis::objectives::Action::Guard3d {
        point: [3., 2.8, 0.],
        arrival_distance: 0.2,
    };
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-3,0,0],[-4,0,2],[0,0,2],[1,0,0],[3,2,0]],"links":[{"from":0,"to":1,"mode":"walk"},{"from":1,"to":0,"mode":"walk"},{"from":1,"to":2,"mode":"walk"},{"from":2,"to":3,"mode":"walk"},{"from":3,"to":4,"mode":"fly"}]})).unwrap();
    s
}
#[test]
fn partial_route_keeps_detour_stops_without_success_or_replanning_and_replays() {
    let mut e = Engine::new(partial_scene()).unwrap();
    step(&mut e, 1);
    let s = &e.simulation.mobility.states["actor"];
    assert_eq!(s.route, vec![0, 2, 3]);
    assert_eq!(s.partial_target, Some(3));
    step(&mut e, 220);
    let s = &e.simulation.mobility.states["actor"];
    assert_eq!(s.completed, 3);
    assert_eq!(s.plans, 1);
    assert_eq!(s.phase, Phase::PartialReached);
    assert_eq!(s.destination, Some([3., 2., 0.]));
    assert_eq!(e.simulation.navigation["actor"].status, "unreachable");
    let plans = s.plans;
    let completed = s.completed;
    step(&mut e, 500);
    assert_eq!(e.simulation.mobility.states["actor"].plans, plans);
    assert_eq!(e.simulation.mobility.states["actor"].completed, completed);
    let replay = e.replay();
    let restored = Engine::replay_at(&replay, None).unwrap();
    assert_eq!(
        e.simulation.mobility.states,
        restored.simulation.mobility.states
    );
    assert_eq!(e.simulation.positions(), restored.simulation.positions());
    let prefix = Engine::replay_at(&replay, Some(50)).unwrap();
    assert_eq!(
        prefix.simulation.mobility.states["actor"].partial_target,
        Some(3)
    );
    assert_eq!(prefix.simulation.mobility.states["actor"].plans, 1);
}
#[test]
fn default_stop_and_complete_route_have_precedence_over_partial_policy() {
    for policy in [None, Some(ignis::mobility::UnreachablePolicy::Stop)] {
        let mut s = partial_scene();
        s.entities
            .last_mut()
            .unwrap()
            .npc
            .as_mut()
            .unwrap()
            .unreachable_policy = policy;
        let mut e = Engine::new(s).unwrap();
        step(&mut e, 120);
        assert_eq!(e.simulation.mobility.states["actor"].completed, 0);
        assert_eq!(
            e.simulation.mobility.states["actor"].phase,
            Phase::Unreachable
        );
    }
    let mut s = partial_scene();
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.mobility.as_mut().unwrap().fly_speed = Some(3.);
    n.unreachable_policy = Some(ignis::mobility::UnreachablePolicy::ApproachReachable {
        min_progress: 0.25,
        max_path_ticks: 1,
    });
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_eq!(e.simulation.mobility.states["actor"].partial_target, None);
    step(&mut e, 400);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 4);
}
#[test]
fn partial_progress_effort_and_ties_are_bounded_and_deterministic() {
    for (min_progress, max_path_ticks, expected) in
        [(0.25, 40, None), (4., 300, None), (0.25, 100, Some(2))]
    {
        let mut s = partial_scene();
        s.entities
            .last_mut()
            .unwrap()
            .npc
            .as_mut()
            .unwrap()
            .unreachable_policy = Some(ignis::mobility::UnreachablePolicy::ApproachReachable {
            min_progress,
            max_path_ticks,
        });
        let mut e = Engine::new(s).unwrap();
        step(&mut e, 1);
        assert_eq!(
            e.simulation.mobility.states["actor"].partial_target,
            expected
        );
        if expected.is_none() {
            assert_eq!(
                e.simulation.mobility.states["actor"].reason,
                "no_partial_progress"
            );
        }
    }
    let mut s = partial_scene();
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-3,0,0],[0,0,-1],[0,0,1]],"links":[{"from":0,"to":2,"mode":"walk"},{"from":0,"to":1,"mode":"walk"}]})).unwrap();
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.mobility.states["actor"].partial_target,
        Some(1)
    );
    s.entities.reverse();
    let mut other = Engine::new(s).unwrap();
    step(&mut other, 1);
    assert_eq!(
        e.simulation.mobility.states,
        other.simulation.mobility.states
    );
}
fn acoustic_partial(memory_ticks: u32) -> Scene {
    let mut s = partial_scene();
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.objectives = None;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":10,"fov_degrees":90,"initial_facing_degrees":0,"search_turn_degrees_per_second":90,"memory_ticks":memory_ticks,"search_ticks":1,"hearing":{"range":20}})).unwrap());
    s
}
fn point_noise(e: &mut Engine, position: [f32; 3]) {
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position { position },
            radius: 20.,
        }
        .into(),
    )
    .unwrap();
}
#[test]
fn partial_memory_expiry_finishes_only_current_link_and_new_cue_replaces_destination() {
    let mut e = Engine::new(acoustic_partial(3)).unwrap();
    point_noise(&mut e, [3., 2.8, 0.]);
    step(&mut e, 1);
    assert!(
        e.simulation.mobility.states["actor"]
            .partial_target
            .is_some()
    );
    step(&mut e, 5);
    assert!(e.simulation.mobility.states["actor"].committed);
    step(&mut e, 200);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 1);
    assert_eq!(e.simulation.mobility.states["actor"].partial_target, None);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Idle);
    let mut e = Engine::new(acoustic_partial(600)).unwrap();
    point_noise(&mut e, [3., 2.8, 0.]);
    step(&mut e, 5);
    point_noise(&mut e, [-3., 0.8, 0.]);
    step(&mut e, 1);
    assert!(e.simulation.mobility.states["actor"].committed);
    let mut saw_arrival = false;
    for _ in 0..200 {
        step(&mut e, 1);
        saw_arrival |= e.simulation.mobility.states["actor"].phase == Phase::Arrived;
    }
    assert!(saw_arrival);
    assert_eq!(e.simulation.mobility.states["actor"].completed, 2);
    assert_eq!(e.simulation.mobility.states["actor"].partial_target, None);
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Idle);
}
#[test]
fn partial_policy_edits_validation_legacy_and_reset_are_atomic() {
    let mut e = Engine::new(partial_scene()).unwrap();
    step(&mut e, 50);
    let before = e.snapshot();
    for (min_progress, max_path_ticks) in [(0., 300), (0.2, 0), (0.2, 3601), (f32::NAN, 300)] {
        let mut entity = e
            .scene
            .entities
            .iter()
            .find(|a| a.id == "actor")
            .unwrap()
            .clone();
        entity.npc.as_mut().unwrap().unreachable_policy =
            Some(ignis::mobility::UnreachablePolicy::ApproachReachable {
                min_progress,
                max_path_ticks,
            });
        assert!(
            e.execute(
                Command::Upsert {
                    entity: Box::new(entity)
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(before, e.snapshot());
    }
    let mut old = partial_scene();
    old.version = 19;
    assert!(old.validate().is_err());
    old.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .unreachable_policy = None;
    assert!(old.validate().is_ok());
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.simulation.mobility.states["actor"].partial_target, None);
    assert_eq!(e.simulation.mobility.states["actor"].plans, 0);
}
