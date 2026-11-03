use ignis::{
    engine::{Command, Engine},
    ground_navigation::{Status, World},
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
#[test]
fn probe_and_execution_climb_descend_ramp_without_teleport() {
    let mut e = Engine::new(scene()).unwrap();
    let reports = e.simulation.mobility.ground_reports["npc"].clone();
    assert!(
        reports.iter().all(|r| r.status == Status::Arrived),
        "{reports:?}"
    );
    let mut previous = e.simulation.positions()["npc"];
    for _ in 0..300 {
        e.advance(1).unwrap();
        let p = e.simulation.positions()["npc"];
        assert!(glam::Vec3::from(p).distance(previous.into()) < 0.12);
        previous = p;
    }
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?} {:?}",
        e.simulation.mobility.states["npc"],
        e.simulation.positions()
    );
    replay(&e);
    let p = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "npc")
        .unwrap()
        .clone();
    let mut p = p;
    p.transform.position = [5.5, 2.2658808, 0.];
    p.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[-1.5,0.8,0],"arrival_distance":0.2}}])).unwrap());
    run(
        &mut e,
        Command::Upsert {
            entity: Box::new(p),
        },
    );
    e.advance(300).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?}",
        e.simulation.mobility.states["npc"]
    );
    assert!(e.simulation.positions()["npc"][1] < 0.9);
    replay(&e);
}
#[test]
fn steep_ramp_rejected_and_configuration_can_admit_it() {
    for (max, pass) in [(30., false), (45., true)] {
        let mut s = scene();
        ramp(&mut s, 40.);
        let c = npc_mut(&mut s).character_controller.as_mut().unwrap();
        c.max_slope_degrees = max;
        c.slide_slope_degrees = 50.;
        let mut e = Engine::new(s).unwrap();
        let r = e.simulation.mobility.ground_reports["npc"][0];
        assert_eq!(r.status == Status::Arrived, pass, "{r:?}");
        e.advance(480).unwrap();
        assert_eq!(
            e.simulation.mobility.states["npc"].phase == Phase::Arrived,
            pass,
            "{:?} {:?} {r:?}",
            e.simulation.mobility.states["npc"],
            e.simulation.positions()
        );
        replay(&e);
    }
}
#[test]
fn steps_ceiling_and_body_size_change_admission() {
    for (height, ceiling, pass) in [(0.2, false, true), (0.5, false, false), (0.2, true, false)] {
        let mut s = scene();
        s.entities.retain(|e| e.id != "ramp" && e.id != "landing");
        s.entities
            .push(box_("step", [4., height / 2., 0.], [8., height, 4.]));
        s.spatial_network.nodes[1] = [5.5, height, 0.];
        npc_mut(&mut s).npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[5.5,height+0.8,0],"arrival_distance":0.2}}])).unwrap());
        if ceiling {
            s.entities
                .push(box_("ceiling", [2., 1.95, 0.], [6., 0.4, 4.]));
        }
        let mut e = Engine::new(s).unwrap();
        let r = e.simulation.mobility.ground_reports["npc"][0];
        assert_eq!(
            r.status == Status::Arrived,
            pass,
            "{height} {ceiling} {r:?}"
        );
        e.advance(300).unwrap();
        assert_eq!(
            e.simulation.mobility.states["npc"].phase == Phase::Arrived,
            pass
        );
        replay(&e);
    }
}
#[test]
fn read_only_query_reports_bounds_overlap_end_support_and_budget() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    let r = run(
        &mut e,
        Command::QueryGroundTraversal {
            entity: "npc".into(),
            start: [-1.5, 0., 0.],
            end: [5.5, 1.4558809, 0.],
        },
    );
    assert_eq!(r["report"]["status"], "arrived");
    assert_eq!(e.snapshot(), before);
    assert!(
        e.execute(
            Command::QueryGroundTraversal {
                entity: "npc".into(),
                start: [0.; 3],
                end: [50.; 3]
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let world = World::new(&e.scene);
    let n = e.scene.entities.iter().find(|e| e.id == "npc").unwrap();
    assert_eq!(
        world.trace(n, [-1.5, 5., 0.], [5.5, 1.4558809, 0.]).status,
        Status::UnsupportedStart
    );
    assert_eq!(
        world.trace(n, [-1.5, 0., 0.], [5.5, 5., 0.]).status,
        Status::UnsupportedEnd
    );
    assert_eq!(
        world.trace(n, [2., 0., 0.], [5.5, 1.4558809, 0.]).status,
        Status::InitialOverlap
    );
    let mut slow = n.clone();
    slow.npc.as_mut().unwrap().speed = 0.1;
    let r = world.trace(&slow, [-1.5, 0., 0.], [5.5, 1.4558809, 0.]);
    assert_eq!(r.status, Status::BudgetExceeded);
    assert_eq!(r.ticks, 600);
}
#[test]
fn invalid_air_configuration_and_old_format_are_atomic() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    let source = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "npc")
        .unwrap()
        .clone();
    for mode in ["jump_speed", "fly_speed"] {
        let mut p = serde_json::to_value(&source).unwrap();
        p["npc"]["mobility"][mode] = json!(0);
        let entity = serde_json::from_value(p).unwrap();
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
    let mut s = scene();
    s.version = 83;
    assert!(s.validate().is_err());
}
#[test]
fn casts_preserve_in_progress_ground_segment_and_resume() {
    let mut s = scene();
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[{"id":"hold","selection":"self_actor","range":0,"cast_ticks":40,"cooldown_ticks":0,"movement":{"mode":"lock"},"costs":[],"effects":[{"kind":"noise","radius":3}]}]})).unwrap();
    npc_mut(&mut s).abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["hold"]})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(95).unwrap();
    let before = e.simulation.positions()["npc"];
    let edge = e.simulation.mobility.states["npc"].edge;
    run(
        &mut e,
        Command::ActivateAbility {
            actor: "npc".into(),
            slot: 1,
            target: None,
        },
    );
    e.advance(25).unwrap();
    let after = e.simulation.positions()["npc"];
    assert!((after[0] - before[0]).abs() < 0.001);
    assert_eq!(e.simulation.mobility.states["npc"].edge, edge);
    e.advance(300).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?}",
        e.simulation.mobility.states["npc"]
    );
    replay(&e);
}
#[test]
fn geometry_edits_invalidate_cache_undo_restores_and_replay_compares() {
    let mut e = Engine::new(scene()).unwrap();
    let mut ramp = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "ramp")
        .unwrap()
        .clone();
    ramp.transform.size[1] = 4.;
    ramp.transform.position[1] = 2.;
    run(
        &mut e,
        Command::Upsert {
            entity: Box::new(ramp),
        },
    );
    assert!(e.simulation.mobility.compiled["npc"][0].is_none());
    run(&mut e, Command::Undo);
    assert!(e.simulation.mobility.compiled["npc"][0].is_some());
    e.advance(80).unwrap();
    let prefix = e.snapshot();
    e.advance(200).unwrap();
    let r = e.replay();
    let restored = Engine::replay_at(&r, Some(80)).unwrap().snapshot();
    for key in ["runtime_positions", "mobility", "character_controllers"] {
        assert_eq!(restored[key], prefix[key], "{key}");
    }
    replay(&e);
    let mut bad = r;
    bad.expected_mobility.get_mut("npc").unwrap().completed += 1;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["matched"], false);
}
#[test]
fn planner_selects_detour_from_compiled_compatible_links() {
    let mut s = scene();
    let h = s.spatial_network.nodes[1][1];
    s.entities
        .push(box_("blocked_direct", [2., 2., 0.], [0.5, 4., 4.]));
    let mut r = s.entities.iter().find(|e| e.id == "ramp").unwrap().clone();
    r.id = "detour_ramp".into();
    r.transform.position[2] = 6.;
    s.entities.push(r);
    s.entities
        .push(box_("detour_landing", [7., h / 2., 6.], [6., h, 4.]));
    s.entities
        .push(box_("top_bridge", [5.5, h / 2., 3.], [3., h, 8.]));
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-1.5,0,0],[5.5,h,0],[-1.5,0,6],[5.5,h,6]],"links":[{"from":0,"to":1,"mode":"walk"},{"from":0,"to":2,"mode":"walk"},{"from":2,"to":3,"mode":"walk"},{"from":3,"to":1,"mode":"walk"}]})).unwrap();
    let mut e = Engine::new(s).unwrap();
    assert!(e.simulation.mobility.compiled["npc"][0].is_none());
    assert!(
        e.simulation.mobility.compiled["npc"][1..]
            .iter()
            .all(Option::is_some)
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.mobility.states["npc"].route, vec![1, 2, 3]);
    e.advance(800).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?} {:?}",
        e.simulation.mobility.states["npc"],
        e.simulation.positions()
    );
    replay(&e);
}
#[test]
fn gap_is_not_accepted_as_a_fall_to_the_destination() {
    let mut s = scene();
    s.entities.retain(|e| e.id != "ramp");
    let floor = s.entities.iter_mut().find(|e| e.id == "floor").unwrap();
    floor.transform.position[0] = -6.;
    floor.transform.size[0] = 10.;
    let mut e = Engine::new(s).unwrap();
    assert_eq!(
        e.simulation.mobility.ground_reports["npc"][0].status,
        Status::Unsupported
    );
    e.advance(300).unwrap();
    assert_eq!(e.simulation.mobility.states["npc"].completed, 0);
    assert!(e.simulation.positions()["npc"][0] < -1.);
    replay(&e);
}
#[test]
fn profiles_are_shared_for_template_births_and_cleared_on_retirement() {
    let mut s = scene();
    let mut template = npc_mut(&mut s).clone();
    template.id = "template".into();
    template.npc.as_mut().unwrap().enabled = false;
    s.npc_templates.push(template);
    let mut e = Engine::new(s).unwrap();
    let original = e.simulation.mobility.ground_reports["npc"].clone();
    run(
        &mut e,
        Command::SpawnActor {
            template: "template".into(),
            position: [-1.5, 0.8, 6.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    let id = e
        .simulation
        .mobility
        .ground_reports
        .keys()
        .find(|id| id.as_str() != "npc")
        .unwrap()
        .clone();
    assert!(std::sync::Arc::ptr_eq(
        &original,
        &e.simulation.mobility.ground_reports[&id]
    ));
    let old = e.simulation.character_controllers[&id].instance;
    run(&mut e, Command::DespawnActor { actor: id.clone() });
    e.advance(1).unwrap();
    assert!(!e.simulation.mobility.ground_reports.contains_key(&id));
    run(
        &mut e,
        Command::SpawnActor {
            template: "template".into(),
            position: [-1.5, 0.8, 6.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    let new = e
        .simulation
        .mobility
        .ground_reports
        .keys()
        .find(|id| id.as_str() != "npc")
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &original,
        &e.simulation.mobility.ground_reports[new]
    ));
    assert_ne!(e.simulation.character_controllers[new].instance, old);
    replay(&e);
}
#[test]
fn body_height_is_part_of_the_compiled_profile() {
    for (height, pass) in [(1.6, true), (2.4, false)] {
        let mut s = scene();
        s.entities.retain(|e| e.id != "ramp" && e.id != "landing");
        s.spatial_network.nodes[1][1] = 0.;
        let n = npc_mut(&mut s);
        n.transform.size[1] = height;
        n.transform.position[1] = height / 2. + 0.01;
        n.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[5.5,height/2.,0],"arrival_distance":0.2}}])).unwrap());
        s.entities
            .push(box_("ceiling", [2., 2.3, 0.], [6., 0.2, 4.]));
        let mut e = Engine::new(s).unwrap();
        assert_eq!(e.simulation.mobility.compiled["npc"][0].is_some(), pass);
        e.advance(300).unwrap();
        assert_eq!(
            e.simulation.mobility.states["npc"].phase == Phase::Arrived,
            pass
        );
        replay(&e);
    }
}
#[test]
fn compiled_geometry_never_provides_a_hidden_targets_position() {
    let mut s = scene();
    let n = npc_mut(&mut s).npc.as_mut().unwrap();
    n.objectives = None;
    n.target = Some("player".into());
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":30,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":30,"search_turn_degrees_per_second":0})).unwrap());
    s.entities
        .push(box_("occlusion", [-4., 2., 2.5], [1., 4., 10.]));
    let mut e = Engine::new(s).unwrap();
    assert!(e.simulation.mobility.compiled["npc"][0].is_some());
    e.advance(180).unwrap();
    assert!(e.simulation.mobility.states["npc"].destination.is_none());
    assert_eq!(e.simulation.mobility.states["npc"].completed, 0);
    assert!(!e.simulation.npcs["npc"].observed_this_tick);
    replay(&e);
}

#[test]
fn spatial_patrol_repeatedly_crosses_the_ramp_and_returns() {
    let mut s = scene();
    let h = s.spatial_network.nodes[1][1];
    let n = npc_mut(&mut s).npc.as_mut().unwrap();
    n.objectives = None;
    n.patrol=Some(serde_json::from_value(json!({"route":{"kind":"points_3d","points":[[5.5,h+0.8,0],[-1.5,0.8,0]]},"traversal":"loop","wait_ticks":20,"travel_ticks":600,"retry_ticks":60,"arrival_distance":0.2})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(1000).unwrap();
    assert!(
        e.simulation.patrols["npc"].visited >= 3,
        "{:?}",
        e.simulation.patrols["npc"]
    );
    assert_eq!(e.simulation.patrols["npc"].skipped, 0);
    replay(&e);
}
#[test]
fn death_revive_preserve_static_admissions_and_same_individual() {
    let mut s = scene();
    npc_mut(&mut s).vitality =
        Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[{"id":"kill","selection":"actor","range":30,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},{"id":"revive","selection":"actor","range":30,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":50}]}]})).unwrap();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["kill","revive"]})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(300).unwrap();
    let instance = e.simulation.character_controllers["npc"].instance;
    let compiled = e.simulation.mobility.ground_reports["npc"].clone();
    run(
        &mut e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("npc".into()),
        },
    );
    e.advance(1).unwrap();
    assert!(!e.simulation.character_controllers["npc"].active);
    e.advance(10).unwrap();
    run(
        &mut e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: Some("npc".into()),
        },
    );
    e.advance(3).unwrap();
    assert!(e.simulation.character_controllers["npc"].active);
    assert_eq!(e.simulation.character_controllers["npc"].instance, instance);
    assert!(std::sync::Arc::ptr_eq(
        &compiled,
        &e.simulation.mobility.ground_reports["npc"]
    ));
    assert_eq!(e.simulation.mobility.states["npc"].phase, Phase::Arrived);
    replay(&e);
}
#[test]
fn anonymous_sound_does_not_bypass_three_dimensional_travel_budget() {
    let mut s = scene();
    let h = s.spatial_network.nodes[1][1];
    let n = npc_mut(&mut s).npc.as_mut().unwrap();
    n.objectives = None;
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":0.1,"fov_degrees":90,"initial_facing_degrees":0,"memory_ticks":600,"search_ticks":1,"search_turn_degrees_per_second":0,"max_investigation_distance":3,"hearing":{"range":30}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [5.5, h + 0.8, 0.],
            },
            radius: 20.,
        },
    );
    e.advance(10).unwrap();
    assert!(e.simulation.positions()["npc"][0] < -1.4);
    assert_eq!(
        e.simulation.mobility.states["npc"].reason,
        "insufficient_travel_budget"
    );
    replay(&e);
}
#[test]
fn legacy_walker_cannot_inherit_the_new_inclined_link() {
    let mut s = scene();
    let mut legacy = npc_mut(&mut s).clone();
    legacy.id = "legacy".into();
    legacy.character_controller = None;
    legacy.transform.position[2] = -4.;
    legacy.npc.as_mut().unwrap().enabled = false;
    s.entities.push(legacy);
    let e = Engine::new(s).unwrap();
    assert!(e.simulation.mobility.compiled["npc"][0].is_some());
    assert!(e.simulation.mobility.compiled["legacy"][0].is_none());
}
#[test]
fn dynamic_blocker_is_not_reserved_by_static_probe_and_physical_motor_waits() {
    let mut s = scene();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [1., 1.3, 0.];
    let mut e = Engine::new(s).unwrap();
    assert!(e.simulation.mobility.compiled["npc"][0].is_some());
    e.advance(90).unwrap();
    assert!(
        e.simulation.positions()["npc"][0] < 0.6,
        "{:?}",
        e.simulation.positions()
    );
    assert_ne!(e.simulation.mobility.states["npc"].phase, Phase::Arrived);
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
    e.advance(30).unwrap();
    run(
        &mut e,
        Command::SetInput {
            input: Default::default(),
        },
    );
    e.advance(240).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?} {:?}",
        e.simulation.mobility.states["npc"],
        e.simulation.positions()
    );
    replay(&e);
}
