use ignis::{
    abilities::{Aim, Outcome, Selection},
    ability_policy::{Phase, Policy, SpatialAim},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    physics::Input,
};
use serde_json::json;
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
        {"id":"bolt","selection":"spatial","range":0,"cast_ticks":30,"cooldown_ticks":300,"costs":[{"resource":"energy","units":5}],"delivery":{"kind":"projectile","speed":10,"half_extent":0.08,"max_distance":20},"effects":[{"kind":"damage","recipient":"target","amount":20}]},
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]}
    ]})).unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"target","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[4,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":30,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["bolt","kill"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
        e.footsteps = None;
    }
    entity(&mut s, "player").transform.position = [-4., 0.8, 0.];
    entity(&mut s,"target").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([
        {"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":15,"policy":{"spatial_aim":{"mode":"observed_point","range":12}}}},
        {"priority":0,"when":"always","action":{"kind":"wait"}}
    ])).unwrap());
    s
}
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
fn slot(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap()
}
fn health(e: &Engine, id: &str) -> i64 {
    e.simulation.vitality.state.actors[slot(e, id)]
        .unwrap()
        .units
}
fn energy(e: &Engine, id: &str) -> i64 {
    e.simulation.abilities.state.balances[slot(e, id)][0]
}

fn cast(e: &Engine) -> Option<ignis::abilities::Cast> {
    e.simulation.abilities.state.casts[slot(e, "target")]
}
fn policy(s: &mut Scene) -> &mut Policy {
    let action = &mut entity(s, "target")
        .npc
        .as_mut()
        .unwrap()
        .objectives
        .as_mut()
        .unwrap()[0]
        .action;
    match action {
        ignis::objectives::Action::UseAbility {
            policy: Some(p), ..
        } => p,
        _ => panic!("fixture"),
    }
}
fn phase(e: &Engine) -> Phase {
    e.simulation.npcs["target"].objectives.ability_decisions[0].phase
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn movement(e: &mut Engine, x: f32, z: f32) {
    e.execute(
        Command::SetInput {
            input: Input {
                x,
                z,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
}
fn wall(s: &mut Scene, position: [f32; 3], size: [f32; 3]) {
    s.entities.push(serde_json::from_value(json!({"id":"cover","name":"cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":position,"size":size}})).unwrap());
}
#[test]
fn observed_point_is_frozen_at_start_and_can_be_dodged_before_launch() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let original = cast(&e).unwrap();
    assert_eq!(
        original.aim,
        Some(Aim::Point {
            position: [-4., 0.8, 0.]
        })
    );
    assert_eq!(original.target, None);
    let evidence = original.aim_observation.unwrap();
    assert_eq!(evidence.actor, slot(&e, "player") as u16);
    assert_eq!(evidence.tick, 0);
    assert_eq!(evidence.source, ignis::perception::Source::Vision);
    movement(&mut e, 0., 1.);
    e.advance(20).unwrap();
    assert_eq!(cast(&e).unwrap().aim, original.aim);
    assert_eq!(cast(&e).unwrap().aim_observation, original.aim_observation);
    let prefix = e.simulation.abilities.state.clone();
    e.advance(10).unwrap();
    let shot = e.simulation.abilities.state.projectiles.active[0];
    assert_eq!(shot.cast.aim, original.aim);
    assert_eq!(shot.cast.aim_observation, original.aim_observation);
    assert_eq!(energy(&e, "target"), 995);
    assert!(shot.direction[2].abs() < 0.001);
    movement(&mut e, 0., 0.);
    e.advance(110).unwrap();
    assert_eq!(health(&e, "player"), 100);
    replay(&e);
    assert_eq!(
        Engine::replay_at(&e.replay(), Some(21))
            .unwrap()
            .simulation
            .abilities
            .state,
        prefix
    );
}
#[test]
fn still_target_is_hit_after_launch_and_cost_is_paid_once() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(31).unwrap();
    assert_eq!(health(&e, "player"), 100);
    assert_eq!(energy(&e, "target"), 995);
    e.advance(60).unwrap();
    assert_eq!(health(&e, "player"), 80);
    assert_eq!(energy(&e, "target"), 995);
    replay(&e);
}
#[test]
fn losing_sight_during_cast_cancels_without_spending() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 90;
    wall(&mut s, [0., 1.5, 0.75], [0.1, 3., 0.5]);
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert!(cast(&e).is_some());
    movement(&mut e, 0., 1.);
    for _ in 0..80 {
        e.advance(1).unwrap();
        if cast(&e).is_none() {
            break;
        }
    }
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e, "target"), 1000);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::Cancelled)
    );
    replay(&e);
}
#[test]
fn leaving_decision_range_cancels_a_cast_without_starting_approach_mid_cast() {
    let mut s = scene();
    policy(&mut s).spatial_aim = Some(SpatialAim::ObservedPoint { range: 8.1 });
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert!(cast(&e).is_some());
    movement(&mut e, -1., 0.);
    for _ in 0..10 {
        e.advance(1).unwrap();
        if cast(&e).is_none() {
            break;
        }
    }
    assert!(cast(&e).is_none());
    assert_eq!(phase(&e), Phase::OutOfRange);
    assert_eq!(energy(&e, "target"), 1000);
    replay(&e);
}
#[test]
fn approach_uses_policy_range_not_spatial_definition_zero_range() {
    let mut s = scene();
    let p = policy(&mut s);
    p.spatial_aim = Some(SpatialAim::ObservedPoint { range: 3. });
    p.approach = Some(ignis::ability_policy::Approach { margin: 0.4 });
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::Approaching);
    assert!(cast(&e).is_none());
    e.advance(200).unwrap();
    assert_eq!(health(&e, "player"), 80);
    assert_eq!(energy(&e, "target"), 995);
    assert!(e.simulation.position("target").unwrap()[0] < -1.3);
    replay(&e);
}
#[test]
fn hidden_position_and_anonymous_sound_never_create_automatic_aim() {
    let mut s = scene();
    wall(&mut s, [0., 1.5, 0.], [0.1, 3., 10.]);
    if let Some(ignis::model::Awareness::Perceptive { hearing, .. }) =
        &mut entity(&mut s, "target").npc.as_mut().unwrap().awareness
    {
        *hearing = Some(ignis::model::Hearing { range: 20. });
    }
    for z in [-1., 1.] {
        let mut changed = s.clone();
        entity(&mut changed, "player").transform.position[2] = z;
        let mut e = Engine::new(changed).unwrap();
        e.execute(
            Command::EmitNoise {
                origin: ignis::noise::NoiseOrigin::Position {
                    position: [3., 0.8, 0.],
                },
                radius: 12.,
            }
            .into(),
        )
        .unwrap();
        e.advance(90).unwrap();
        assert_eq!(phase(&e), Phase::NoVisibleTarget);
        assert!(cast(&e).is_none());
        assert!(e.simulation.abilities.state.receipts.is_empty());
        assert_eq!(energy(&e, "target"), 1000);
        replay(&e);
    }
}
#[test]
fn explicit_omniscience_is_preserved_and_labelled_in_provenance() {
    let mut s = scene();
    wall(&mut s, [0., 1.5, 0.], [0.1, 3., 10.]);
    entity(&mut s, "target").npc.as_mut().unwrap().awareness =
        Some(ignis::model::Awareness::Omniscient);
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        cast(&e).unwrap().aim_observation.unwrap().source,
        ignis::perception::Source::Omniscient
    );
    e.advance(100).unwrap();
    assert_eq!(health(&e, "player"), 100);
    assert_eq!(energy(&e, "target"), 995);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .projectiles
            .receipts
            .back()
            .unwrap()
            .outcome,
        ignis::projectiles::Outcome::Blocked
    );
    replay(&e);
}
#[test]
fn spatial_policy_validation_is_opt_in_versioned_and_atomic() {
    let s = scene();
    let mut bad = s.clone();
    bad.version = 31;
    assert!(bad.validate().is_err());
    for range in [0., 101., f32::NAN, f32::INFINITY] {
        let mut bad = s.clone();
        policy(&mut bad).spatial_aim = Some(SpatialAim::ObservedPoint { range });
        assert!(bad.validate().is_err());
    }
    let mut bad = s.clone();
    policy(&mut bad).spatial_aim = None;
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.ability_catalog.abilities[0].selection = Selection::Actor;
    bad.ability_catalog.abilities[0].range = 12.;
    assert!(bad.validate().is_err());
    let mut e = Engine::new(s).unwrap();
    e.advance(5).unwrap();
    let before = e.snapshot();
    let mut invalid = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "target")
        .unwrap()
        .clone();
    if let ignis::objectives::Action::UseAbility {
        policy: Some(p), ..
    } = &mut invalid.npc.as_mut().unwrap().objectives.as_mut().unwrap()[0].action
    {
        p.approach = Some(ignis::ability_policy::Approach { margin: 13. });
    }
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(invalid)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
#[test]
fn external_spatial_order_is_distinct_from_observation_based_automatic_cast() {
    let mut s = scene();
    wall(&mut s, [0., 1.5, 0.], [0.1, 3., 10.]);
    let mut e = Engine::new(s).unwrap();
    e.execute(
        Command::AimAbility {
            actor: "target".into(),
            slot: 1,
            aim: Aim::Direction {
                vector: [0., 1., 0.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert!(cast(&e).unwrap().aim_observation.is_none());
    e.advance(30).unwrap();
    assert_eq!(energy(&e, "target"), 995);
    assert_eq!(
        e.simulation.abilities.state.projectiles.active[0].direction,
        [0., 1., 0.]
    );
    replay(&e);
}
#[test]
fn source_death_and_explicit_cancel_do_not_pay_for_pending_cast() {
    for kill in [false, true] {
        let mut e = Engine::new(scene()).unwrap();
        e.advance(1).unwrap();
        let c = cast(&e).unwrap();
        let command = if kill {
            Command::ActivateAbility {
                actor: "player".into(),
                slot: 2,
                target: Some("target".into()),
            }
        } else {
            Command::CancelAbility {
                actor: "target".into(),
                execution_id: c.id,
            }
        };
        e.execute(command.into()).unwrap();
        e.advance(1).unwrap();
        assert!(cast(&e).is_none());
        assert_eq!(energy(&e, "target"), 1000);
        assert!(e.simulation.abilities.state.projectiles.active.is_empty());
        replay(&e);
    }
}
#[test]
fn decision_relation_and_vitality_threshold_gate_spatial_intent() {
    let mut s = scene();
    s.ability_catalog.abilities[0].relation = Some(ignis::targeting::Attitude::Hostile);
    let mut e = Engine::new(s).unwrap();
    e.advance(60).unwrap();
    assert_eq!(phase(&e), Phase::RelationMismatch);
    assert!(e.simulation.abilities.state.receipts.is_empty());
    let mut s = scene();
    policy(&mut s).vitality_below = Some(ignis::ability_policy::Threshold {
        subject: ignis::abilities::Recipient::Target,
        basis_points: 7000,
    });
    let mut e = Engine::new(s).unwrap();
    e.advance(60).unwrap();
    assert_eq!(phase(&e), Phase::VitalitySatisfied);
    assert!(e.simulation.abilities.state.receipts.is_empty());
}
#[test]
fn xyz_observation_drives_downward_shot_and_range_checks_height() {
    let mut s = scene();
    let n = entity(&mut s, "target");
    n.transform.position[1] = 4.8;
    n.npc.as_mut().unwrap().mobility = Some(
        serde_json::from_value(json!({"walk":false,"fly_speed":3,"jump_speed":null,"glide":null}))
            .unwrap(),
    );
    n.npc.as_mut().unwrap().local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":6})).unwrap());
    let mut short = s.clone();
    policy(&mut short).spatial_aim = Some(SpatialAim::ObservedPoint { range: 8.1 });
    let mut e = Engine::new(short).unwrap();
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::OutOfRange);
    assert!(cast(&e).is_none());
    let mut e = Engine::new(s).unwrap();
    e.advance(31).unwrap();
    assert!(e.simulation.abilities.state.projectiles.active[0].direction[1] < -0.4);
    e.advance(80).unwrap();
    assert_eq!(health(&e, "player"), 80);
    replay(&e);
}
#[test]
fn guidance_is_presentation_only_and_reset_clears_it() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(5).unwrap();
    let state = e.simulation.abilities.state.clone();
    e.execute(
        Command::SetDebugView {
            npc_routes: Some(true),
            npc_vision: None,
            npc_hearing: None,
            noise_events: None,
        }
        .into(),
    )
    .unwrap();
    let frame = e.render_frame();
    let markers = frame
        .boxes
        .iter()
        .filter(|b| b.color == [1., 0.72, 0.12] && b.position == [-4., 0.8, 0.])
        .count();
    assert_eq!(markers, 3);
    assert_eq!(e.simulation.abilities.state, state);
    e.execute(
        Command::SetDebugView {
            npc_routes: Some(false),
            npc_vision: None,
            npc_hearing: None,
            noise_events: None,
        }
        .into(),
    )
    .unwrap();
    assert!(
        !e.render_frame()
            .boxes
            .iter()
            .any(|b| b.color == [1., 0.72, 0.12])
    );
    e.execute(Command::Reset.into()).unwrap();
    assert!(cast(&e).is_none());
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
}

#[test]
fn switching_visible_candidate_cancels_instead_of_retargeting_cast() {
    let mut s = scene();
    entity(&mut s, "player").faction = Some("blue".into());
    let n = entity(&mut s, "target");
    n.faction = Some("red".into());
    n.npc.as_mut().unwrap().target = None;
    n.npc.as_mut().unwrap().targeting =
        Some(ignis::targeting::Policy::HostileNearest { switch_margin: 0. });
    let mut other = entity(&mut s, "target").clone();
    other.id = "other".into();
    other.faction = Some("blue".into());
    other.transform.position = [-3., 0.8, 4.];
    other.abilities = None;
    let n = other.npc.as_mut().unwrap();
    n.targeting = None;
    n.objectives = Some(
        serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"wait"}}]))
            .unwrap(),
    );
    s.entities.push(other);
    s.faction_relations = vec![ignis::targeting::Relation {
        from: "red".into(),
        to: "blue".into(),
        attitude: ignis::targeting::Attitude::Hostile,
    }];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let old = cast(&e).unwrap();
    assert_eq!(
        old.aim_observation.unwrap().actor,
        slot(&e, "player") as u16
    );
    movement(&mut e, -1., 0.);
    for _ in 0..15 {
        e.advance(1).unwrap();
        if cast(&e).is_none() {
            break;
        }
    }
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e, "target"), 1000);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.execution_id == Some(old.id) && r.outcome == Outcome::Cancelled)
    );
    replay(&e);
}
#[test]
fn observed_health_recovery_cancels_spatial_support_cast() {
    let mut s = scene();
    entity(&mut s, "player").vitality.as_mut().unwrap().initial = 40;
    policy(&mut s).vitality_below = Some(ignis::ability_policy::Threshold {
        subject: ignis::abilities::Recipient::Target,
        basis_points: 7000,
    });
    s.ability_catalog.abilities[1].selection = Selection::SelfActor;
    s.ability_catalog.abilities[1].effects = vec![ignis::abilities::Effect::Heal {
        recipient: ignis::abilities::Recipient::Target,
        amount: 100,
    }];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert!(cast(&e).is_some());
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: None,
        }
        .into(),
    )
    .unwrap();
    e.advance(2).unwrap();
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e, "target"), 1000);
    assert_eq!(phase(&e), Phase::VitalitySatisfied);
    replay(&e);
}
