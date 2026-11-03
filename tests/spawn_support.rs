use ignis::{
    engine::{Command, Engine},
    generator_control::Action,
    model::{Entity, RoomRecipe, Scene},
    population::Outcome,
    spawn_support::Projection,
};
use serde_json::{Value, json};

fn box_at(id: &str, p: [f32; 3], size: [f32; 3]) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"fixed","color":[0.3,0.5,0.5],"transform":{"position":p,"size":size}})).unwrap()
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.npc_templates = vec![serde_json::from_value(json!({"id":"guard","name":"Guard","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":2,"stop_distance":1,"enabled":false},"vitality":{"initial":100,"maximum":100,"scale":1}})).unwrap()];
    s.npc_generators=vec![serde_json::from_value(json!({"id":"camp","template":"guard","points":[[-3,8,0],[0,8,0],[3,8,0]],"support_projection":{"max_drop":10},"seed":42,"target_alive":3,"interval_ticks":1,"attempts_per_cycle":8,"max_births":12,"retire_dead_after_ticks":0})).unwrap()];
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn query(e: &mut Engine, template: &str, p: [f32; 3], drop: f32) -> Value {
    run(
        e,
        Command::QuerySpawnSupport {
            template: template.into(),
            position: p,
            max_drop: drop,
        },
    )
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn remove_members(e: &mut Engine) {
    let refs: Vec<_> = e.simulation.generators.generators[0]
        .members
        .iter()
        .flatten()
        .map(|m| {
            e.simulation
                .actors
                .label(e.simulation.definitions(), m.instance)
                .unwrap()
                .to_owned()
        })
        .collect();
    for actor in refs {
        run(e, Command::DespawnActor { actor });
    }
    e.advance(1).unwrap();
}
#[test]
fn read_only_query_resolves_feet_and_body_without_advancing_or_recording() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    let recorded = serde_json::to_value(e.replay()).unwrap();
    for _ in 0..3 {
        let q = query(&mut e, "guard", [2., 8., 0.], 8.);
        assert_eq!(q["surface_id"], "floor");
        assert_eq!(q["report"]["status"], "supported");
        assert_eq!(q["report"]["feet"], json!([2., 0., 0.]));
        assert_eq!(q["report"]["position"], json!([2., 0.8f32, 0.]));
        assert_eq!(q["geometry_only"], true);
    }
    assert_eq!(e.snapshot(), before);
    assert_eq!(serde_json::to_value(e.replay()).unwrap(), recorded);
}
#[test]
fn first_top_and_inclusive_drop_ignore_higher_surfaces_and_tie_by_id() {
    let mut s = scene();
    s.entities
        .push(box_at("z_platform", [0., 1., 0.], [4., 2., 4.]));
    s.entities
        .push(box_at("a_platform", [0., 1., 0.], [4., 2., 4.]));
    for reverse in [false, true] {
        let mut copy = s.clone();
        if reverse {
            copy.entities.reverse();
        }
        let mut e = Engine::new(copy).unwrap();
        let q = query(&mut e, "guard", [0., 8., 0.], 6.);
        assert_eq!(q["surface_id"], "a_platform");
        assert_eq!(q["report"]["feet"][1], 2.);
        assert_eq!(
            query(&mut e, "guard", [0., 8., 0.], 5.999)["report"]["status"],
            "no_surface"
        );
        assert_eq!(
            query(&mut e, "guard", [0., 1., 0.], 2.)["surface_id"],
            "floor"
        );
        assert_eq!(
            query(&mut e, "guard", [0., -0.1, 0.], 2.)["report"]["status"],
            "no_surface"
        );
    }
}
#[test]
fn narrow_first_platform_never_falls_back_to_wide_lower_floor() {
    let mut s = scene();
    s.entities.push(box_at("thin", [0., 1., 0.], [0.2, 2., 2.]));
    let mut e = Engine::new(s).unwrap();
    let q = query(&mut e, "guard", [0., 8., 0.], 10.);
    assert_eq!(q["surface_id"], "thin");
    assert_eq!(q["report"]["status"], "insufficient_support");
    assert_eq!(q["report"]["position"][1], json!(2.8f32));
}
#[test]
fn modular_seam_supports_full_footprint_but_thin_gap_does_not() {
    let mut s = scene();
    s.entities.push(box_at("left", [-1., 1., 0.], [2., 2., 2.]));
    s.entities.push(box_at("right", [1., 1., 0.], [2., 2., 2.]));
    let mut e = Engine::new(s.clone()).unwrap();
    assert_eq!(
        query(&mut e, "guard", [0., 8., 0.], 10.)["report"]["status"],
        "supported"
    );
    s.entities
        .iter_mut()
        .find(|e| e.id == "right")
        .unwrap()
        .transform
        .position[0] += 0.005;
    let mut e = Engine::new(s).unwrap();
    assert_eq!(
        query(&mut e, "guard", [0., 8., 0.], 10.)["report"]["status"],
        "insufficient_support"
    );
}
#[test]
fn ceiling_clearance_depends_on_selected_template_height() {
    let mut s = scene();
    s.entities
        .push(box_at("ceiling", [0., 1.1, 0.], [4., 0.2, 4.]));
    let mut small = s.npc_templates[0].clone();
    small.id = "small".into();
    small.transform.size[1] = 0.8;
    s.npc_templates.push(small);
    let mut e = Engine::new(s).unwrap();
    assert_eq!(
        query(&mut e, "guard", [0., 0.9, 0.], 2.)["report"]["status"],
        "occupied"
    );
    let q = query(&mut e, "small", [0., 0.9, 0.], 2.);
    assert_eq!(q["report"]["status"], "supported");
    assert_eq!(q["report"]["position"][1], json!(0.4f32));
}
#[test]
fn visual_geometry_is_not_support_and_runtime_bodies_are_current() {
    let mut s = scene();
    let mut fake = box_at("fake", [0., 2., 0.], [4., 1., 4.]);
    fake.body = None;
    s.entities.push(fake);
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    player.transform.position = [0., 0.8, 0.];
    s.npc_generators.clear();
    let mut e = Engine::new(s).unwrap();
    let q = query(&mut e, "guard", [0., 8., 0.], 10.);
    assert_eq!(q["surface_id"], "floor");
    assert_eq!(q["report"]["status"], "occupied");
    run(
        &mut e,
        Command::SetInput {
            input: ignis::physics::Input {
                x: 1.,
                z: 0.,
                ..Default::default()
            },
        },
    );
    e.advance(90).unwrap();
    assert!(e.simulation.position("player").unwrap()[0] > 2.);
    assert_eq!(
        query(&mut e, "guard", [0., 8., 0.], 10.)["report"]["status"],
        "supported"
    );
    replay(&e);
}
#[test]
fn two_heights_use_template_size_and_successful_resolution_in_receipts() {
    let mut s = scene();
    s.entities
        .push(box_at("platform", [3., 1.5, 0.], [3., 3., 3.]));
    let mut tall = s.npc_templates[0].clone();
    tall.id = "tall".into();
    tall.transform.size[1] = 3.2;
    s.npc_templates.push(tall);
    let g = &mut s.npc_generators[0];
    g.template.clear();
    g.templates = serde_json::from_value(
        json!([{"template":"guard","weight":3},{"template":"tall","weight":1}]),
    )
    .unwrap();
    g.target_alive = 3;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators.generators[0].births, 3);
    for r in &e.simulation.generators.receipts {
        if r.outcome != Outcome::Spawned {
            continue;
        }
        let report = r.support.unwrap();
        let p = report.position.unwrap();
        let candidate = r.position.unwrap();
        assert_eq!(candidate[1], 8.);
        assert_eq!(candidate[0], p[0]);
        assert_eq!(candidate[2], p[2]);
        let top = if p[0] == 3. { 3. } else { 0. };
        let half = if r.template == Some(0) { 0.8 } else { 1.6 };
        assert_eq!(p[1], top + half);
    }
    remove_members(&mut e);
    assert_eq!(e.simulation.generators.generators[0].births, 6);
    replay(&e);
}
#[test]
fn failed_attempts_keep_template_quota_identity_and_bounded_receipts() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.template.clear();
    g.templates = serde_json::from_value(json!([{"template":"guard","weight":1}])).unwrap();
    g.points = vec![[40., 8., 40.]];
    g.attempts_per_cycle = 8;
    let mut e = Engine::new(s).unwrap();
    let ids = e.simulation.actors.instance_snapshot();
    e.advance(300).unwrap();
    let g = e.simulation.generators.generators[0].clone();
    assert_eq!(g.births, 0);
    assert_eq!(g.attempts, 300);
    assert_eq!(g.template_draws, 1);
    assert!(g.pending_template);
    assert_eq!(e.simulation.actors.instance_snapshot(), ids);
    assert_eq!(e.simulation.generators.receipts.len(), 256);
    assert_eq!(e.simulation.generators.dropped_receipts, 44);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::UnsupportedPosition
                && r.support.unwrap().status == ignis::spawn_support::Status::NoSurface)
    );
    run(
        &mut e,
        Command::ControlNpcGenerator {
            generator: "camp".into(),
            action: Action::Pause,
        },
    );
    e.advance(10).unwrap();
    let now = e.simulation.generators.generators[0].clone();
    assert_eq!(now.attempts, g.attempts);
    assert_eq!(now.rng, g.rng);
    assert_eq!(now.template_rng, g.template_rng);
    replay(&e);
}
#[test]
fn exact_legacy_stays_exact_and_projection_does_not_draw_more_random_values() {
    let mut s = scene();
    let mut exact = s.clone();
    exact.npc_generators[0].support_projection = None;
    let mut e = Engine::new(exact).unwrap();
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators.generators[0].births, 0);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.support.is_none())
    );
    s.npc_generators[0].region = Some(ignis::spawn_region::Region {
        min: [-5., 8., -2.],
        max: [5., 8., 2.],
    });
    s.npc_generators[0].points.clear();
    let mut legacy = s.clone();
    legacy.npc_generators[0].support_projection = None;
    let r = legacy.npc_generators[0].region.as_mut().unwrap();
    r.min[1] = 0.8;
    r.max[1] = 0.8;
    let mut p = Engine::new(s).unwrap();
    let mut l = Engine::new(legacy).unwrap();
    p.advance(1).unwrap();
    l.advance(1).unwrap();
    assert_eq!(
        p.simulation.generators.generators[0],
        l.simulation.generators.generators[0]
    );
    for (a, b) in p
        .simulation
        .generators
        .receipts
        .iter()
        .zip(&l.simulation.generators.receipts)
    {
        assert_eq!(a.outcome, b.outcome);
        assert_eq!(a.position.unwrap()[0], b.position.unwrap()[0]);
        assert_eq!(a.position.unwrap()[2], b.position.unwrap()[2]);
    }
    replay(&p);
    replay(&l);
}
#[test]
fn invalid_queries_and_edits_preserve_runtime_and_old_versions_reject_opt_in() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let before = e.snapshot();
    for (template, p, drop) in [
        ("missing", [0., 8., 0.], 10.),
        ("guard", [0., 8., 0.], 0.),
        ("guard", [0., 8., 0.], 20001.),
        ("guard", [10001., 8., 0.], 1.),
        ("guard", [f32::NAN, 8., 0.], 1.),
        ("guard", [0., 8., 0.], f32::INFINITY),
    ] {
        assert!(
            e.execute(
                Command::QuerySpawnSupport {
                    template: template.into(),
                    position: p,
                    max_drop: drop
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut bad = e.scene.npc_generators.clone();
    bad[0].support_projection = Some(Projection { max_drop: -1. });
    assert!(
        e.execute(Command::SetNpcGenerators { generators: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut old = scene();
    // Historical capability gate, independent of the combined output schema.
    old.version = 65;
    assert!(
        old.validate()
            .unwrap_err()
            .message
            .contains("Support projection requires scene66")
    );
    old.npc_generators[0].support_projection = None;
    assert!(old.validate().is_ok());
    old.npc_generators[0].support_projection = Some(Projection { max_drop: 10. });
    old.version = 66;
    assert!(old.validate().is_ok());
}
#[test]
fn edit_undo_reset_and_replay_compare_resolved_support_and_pending_inputs() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let first = e.simulation.generators.clone();
    let snapshot = e.snapshot();
    e.advance(10).unwrap();
    let r = e.replay();
    assert_eq!(
        Engine::replay_at(&r, Some(1)).unwrap().snapshot()["generators"],
        snapshot["generators"]
    );
    let mut corrupt = r.clone();
    corrupt.expected_generators.receipts[0]
        .support
        .as_mut()
        .unwrap()
        .position
        .as_mut()
        .unwrap()[1] += 1.;
    assert_eq!(Engine::verify_replay(&corrupt).unwrap()["matched"], false);
    let member = first.generators[0].members.iter().flatten().next().unwrap();
    let actor = e
        .simulation
        .actors
        .label(e.simulation.definitions(), member.instance)
        .unwrap()
        .to_owned();
    run(&mut e, Command::DespawnActor { actor });
    replay(&e);
    run(&mut e, Command::Reset);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators, first);
    run(
        &mut e,
        Command::SetNpcGenerators {
            generators: Vec::new(),
        },
    );
    run(&mut e, Command::Undo);
    assert_eq!(
        e.scene.npc_generators[0].support_projection,
        Some(Projection { max_drop: 10. })
    );
    assert_eq!(e.simulation.generators.generators[0].births, 0);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators, first);
    run(&mut e, Command::Redo);
    assert!(e.scene.npc_generators.is_empty());
}
#[test]
fn query_is_geometry_only_even_when_population_is_full() {
    let mut s = scene();
    s.npc_generators[0].points = (0..8).map(|i| [-7. + i as f32 * 2., 8., 0.]).collect();
    s.npc_generators[0].target_alive = 8;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators.generators[0].births, 8);
    assert_eq!(
        query(&mut e, "guard", [0., 8., 3.], 10.)["report"]["status"],
        "supported"
    );
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [0., 0.8, 3.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.population.receipts.back().unwrap().outcome,
        Outcome::Capacity
    );
    replay(&e);
}
#[test]
fn hover_template_preserves_exact_air_placement_but_projection_is_explicit() {
    let mut s = scene();
    let t = &mut s.npc_templates[0];
    t.npc.as_mut().unwrap().mobility =
        Some(serde_json::from_value(json!({"walk":false,"fly_speed":2})).unwrap());
    s.npc_templates[0].npc.as_mut().unwrap().local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap());
    s.npc_generators[0].support_projection = None;
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators.generators[0].births, 3);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.support.is_none())
    );
    s.npc_generators[0].support_projection = Some(Projection { max_drop: 10. });
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators.generators[0].births, 3);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.support.unwrap().position.unwrap()[1] == 0.8)
    );
    replay(&e);
}
#[test]
fn resolved_center_outside_world_bounds_is_reported_without_identity() {
    let mut s = scene();
    s.entities
        .push(box_at("high", [0., 9999., 0.], [4., 2., 4.]));
    let mut e = Engine::new(s).unwrap();
    let q = query(&mut e, "guard", [0., 10000., 0.], 1.);
    assert_eq!(q["surface_id"], "high");
    assert_eq!(q["report"]["status"], "out_of_bounds");
}

#[test]
fn complexity_limit_is_explicit_and_single_surface_fast_path_is_preserved() {
    let mut s = scene();
    s.npc_templates[0].transform.size[0] = 3.5;
    for i in 0..65 {
        s.entities.push(box_at(
            &format!("tile{i}"),
            [i as f32 * 0.05 - 1.6, 1., 0.],
            [0.06, 2., 2.],
        ));
    }
    let mut e = Engine::new(s.clone()).unwrap();
    assert_eq!(
        query(&mut e, "guard", [0., 8., 0.], 10.)["report"]["status"],
        "support_limit"
    );
    s.entities.push(box_at("whole", [0., 1., 0.], [4., 2., 2.]));
    let mut e = Engine::new(s).unwrap();
    assert_eq!(
        query(&mut e, "guard", [0., 8., 0.], 10.)["report"]["status"],
        "supported"
    );
}
#[test]
fn corpse_occupies_support_until_retired_and_slot_reuse_has_fresh_identity() {
    let mut s = scene();
    s.npc_generators[0].points = vec![[0., 8., 0.]];
    s.npc_generators[0].target_alive = 1;
    s.npc_templates[0].vitality.as_mut().unwrap().initial = 0;
    s.npc_generators[0].retire_dead_after_ticks = None;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let old = e.simulation.generators.generators[0].members[0].unwrap();
    assert_eq!(
        query(&mut e, "guard", [0., 8., 0.], 10.)["report"]["status"],
        "occupied"
    );
    let actor = e
        .simulation
        .actors
        .label(e.simulation.definitions(), old.instance)
        .unwrap()
        .to_owned();
    run(&mut e, Command::DespawnActor { actor });
    e.advance(1).unwrap();
    let new = e.simulation.generators.generators[0].members[0].unwrap();
    assert_ne!(old.instance, new.instance);
    assert_eq!(old.actor, new.actor);
    assert_eq!(e.simulation.generators.generators[0].births, 2);
    replay(&e);
}
#[test]
fn projection_does_not_override_missing_mobility_even_with_valid_support() {
    let mut s = scene();
    let n = s.npc_templates[0].npc.as_mut().unwrap();
    n.mobility = Some(serde_json::from_value(json!({"walk":false,"jump_speed":2})).unwrap());
    s.spatial_network.nodes = vec![[0., 0., 0.]];
    let mut e = Engine::new(s).unwrap();
    assert_eq!(
        query(&mut e, "guard", [0., 8., 0.], 10.)["report"]["status"],
        "supported"
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators.generators[0].births, 0);
}

#[test]
fn spatial_permission_manual_pause_and_wave_delays_preserve_projection() {
    for mode in ["once", "while_occupied"] {
        let mut s = scene();
        let mut zone = box_at("zone", [0., 1., 8.], [2., 3., 2.]);
        zone.body = None;
        zone.visible = Some(false);
        zone.trigger =
            Some(serde_json::from_value(json!({"enabled":true,"targets":["player"]})).unwrap());
        s.entities.push(zone);
        let g = &mut s.npc_generators[0];
        g.activation = Some(serde_json::from_value(json!({"volume":"zone","mode":mode})).unwrap());
        g.target_alive = 1;
        g.max_births = 2;
        g.waves = serde_json::from_value(
            json!([{"count":1,"delay_ticks":2},{"count":1,"delay_ticks":3}]),
        )
        .unwrap();
        let mut e = Engine::new(s).unwrap();
        e.advance(1).unwrap();
        assert_eq!(e.simulation.generators.generators[0].attempts, 0);
        e.advance(10).unwrap();
        assert_eq!(e.simulation.generators.generators[0].births, 1);
        run(
            &mut e,
            Command::ControlNpcGenerator {
                generator: "camp".into(),
                action: Action::Pause,
            },
        );
        remove_members(&mut e);
        e.advance(10).unwrap();
        assert_eq!(
            e.simulation.generators.generators[0].wave.unwrap().current,
            0
        );
        assert_eq!(e.simulation.generators.generators[0].births, 1);
        run(
            &mut e,
            Command::ControlNpcGenerator {
                generator: "camp".into(),
                action: Action::Activate,
            },
        );
        e.advance(5).unwrap();
        assert_eq!(e.simulation.generators.generators[0].births, 2);
        assert!(
            e.simulation
                .generators
                .receipts
                .iter()
                .filter(|r| r.outcome == Outcome::Spawned)
                .all(|r| r.support.unwrap().status == ignis::spawn_support::Status::Supported)
        );
        replay(&e);
    }
}
#[test]
fn absent_original_target_rejects_before_geometry_without_reroll_or_rebinding() {
    let mut s = scene();
    let mut target = s.npc_templates[0].clone();
    target.id = "target".into();
    target.transform.position = [0., 0.8, 4.];
    s.entities.push(target);
    s.npc_templates[0].npc.as_mut().unwrap().target = Some("target".into());
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::DespawnActor {
            actor: "target".into(),
        },
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators.generators[0].births, 0);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::InstanceUnavailable && r.support.is_none())
    );
    replay(&e);
}
