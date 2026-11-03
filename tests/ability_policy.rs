use ignis::{
    abilities::{Outcome, Recipient, Selection},
    ability_policy::{self, Approach, Decision, Evidence, Offer, Phase, Policy, Target, Threshold},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    vitality::{Config, Health},
};
use serde_json::{Value, json};
fn scene(policy: Value) -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
        {"id":"heal","selection":"actor","range":3,"cast_ticks":5,"cooldown_ticks":10,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"heal","recipient":"target","amount":30}]},
        {"id":"self_heal","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"heal","recipient":"target","amount":100}]}
    ]})).unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [0., 0.8, -6.];
    p.footsteps = None;
    p.vitality = Some(Config {
        initial: 40,
        maximum: 100,
        scale: 1,
    });
    p.abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["self_heal"]})).unwrap());
    let npc:Entity=serde_json::from_value(json!({"id":"npc","name":"npc","body":"npc","color":[0.5,0.4,1],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"vitality":{"initial":40,"maximum":100,"scale":1},"abilities":{"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["heal","self_heal"]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":1,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":10,"policy":policy}},{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap();
    s.entities.push(npc);
    s
}
fn config() -> Value {
    json!({"vitality_below":{"subject":"target","basis_points":7000},"approach":{"margin":0.4}})
}
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
fn phase(e: &Engine) -> Phase {
    e.simulation.npcs["npc"].objectives.ability_decisions[0].phase
}
fn hp(e: &Engine, id: &str) -> i64 {
    let a = e
        .simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap();
    e.simulation.vitality.state.actors[a].unwrap().units
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn activate(e: &mut Engine, actor: &str, slot: u8) {
    e.execute(
        Command::ActivateAbility {
            actor: actor.into(),
            slot,
            target: None,
        }
        .into(),
    )
    .unwrap();
}

#[test]
fn approaches_without_failed_attempts_then_heals_and_waits_at_threshold() {
    let mut e = Engine::new(scene(config())).unwrap();
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::Approaching);
    assert!(e.simulation.abilities.state.receipts.is_empty());
    e.advance(240).unwrap();
    assert_eq!(hp(&e, "player"), 70);
    assert_eq!(phase(&e), Phase::VitalitySatisfied);
    assert_eq!(e.simulation.abilities.state.receipts.len(), 2);
    let p = e.simulation.position("npc").unwrap();
    assert!(p[2] < -3.2 && p[2] > -4.);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::Completed
    );
    replay(&e);
}
#[test]
fn no_approach_means_out_of_range_without_attempt_or_motion() {
    let mut e = Engine::new(scene(json!({}))).unwrap();
    e.advance(90).unwrap();
    assert_eq!(phase(&e), Phase::OutOfRange);
    assert!(e.simulation.abilities.state.receipts.is_empty());
    assert!(e.simulation.position("npc").unwrap()[2].abs() < 0.001);
    replay(&e);
}
#[test]
fn legacy_without_policy_keeps_retry_behavior() {
    let mut e = Engine::new(scene(Value::Null)).unwrap();
    e.advance(25).unwrap();
    assert_eq!(phase(&e), Phase::Unconfigured);
    assert_eq!(e.simulation.abilities.state.receipts.len(), 3);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::OutOfRange)
    );
    replay(&e);
}
#[test]
fn cost_availability_and_missing_vitality_have_no_failed_executions() {
    for missing in [false, true] {
        let mut s = scene(config());
        entity(&mut s, "player").transform.position[2] = -2.;
        if missing {
            entity(&mut s, "player").vitality = None;
        } else {
            entity(&mut s, "npc").abilities.as_mut().unwrap().resources[0].initial = 4;
        }
        let mut e = Engine::new(s).unwrap();
        e.advance(30).unwrap();
        assert_eq!(
            phase(&e),
            if missing {
                Phase::MissingVitality
            } else {
                Phase::InsufficientResources
            }
        );
        assert!(e.simulation.abilities.state.receipts.is_empty());
        replay(&e);
    }
}
#[test]
fn self_health_can_trigger_without_a_target_or_vision() {
    let mut s = scene(json!({"vitality_below":{"subject":"source","basis_points":5000}}));
    let n = entity(&mut s, "npc").npc.as_mut().unwrap();
    n.target = None;
    if let ignis::objectives::Action::UseAbility { slot, .. } =
        &mut n.objectives.as_mut().unwrap()[0].action
    {
        *slot = 2;
    }
    let mut e = Engine::new(s).unwrap();
    e.advance(11).unwrap();
    assert_eq!(hp(&e, "npc"), 100);
    assert_eq!(phase(&e), Phase::VitalitySatisfied);
    replay(&e);
}
#[test]
fn healing_during_cast_cancels_unneeded_action_without_debit() {
    let mut s = scene(config());
    entity(&mut s, "player").transform.position[2] = -2.;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    activate(&mut e, "player", 1);
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "player"), 100);
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::VitalitySatisfied);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::Cancelled)
    );
    let n = e
        .simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == "npc")
        .unwrap();
    assert_eq!(e.simulation.abilities.state.balances[n][0], 100);
    replay(&e);
}
#[test]
fn same_slot_objectives_have_independent_conditions_and_keep_cast_occupied() {
    let mut s = scene(config());
    entity(&mut s, "player").transform.position[2] = -2.;
    let os = entity(&mut s, "npc")
        .npc
        .as_mut()
        .unwrap()
        .objectives
        .as_mut()
        .unwrap();
    let mut first = os[0];
    first.priority = 3;
    if let ignis::objectives::Action::UseAbility {
        policy: Some(p), ..
    } = &mut first.action
    {
        p.vitality_below.as_mut().unwrap().basis_points = 3000;
    }
    os.insert(0, first);
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    let state = &e.simulation.npcs["npc"].objectives;
    assert_eq!(state.active, Some(1));
    assert_eq!(state.ability_decisions[0].phase, Phase::VitalitySatisfied);
    assert_eq!(state.ability_decisions[1].phase, Phase::Casting);
    e.advance(3).unwrap();
    assert_eq!(hp(&e, "player"), 70);
    replay(&e);
}
#[test]
fn obstacles_block_navigation_without_granting_omniscience() {
    let mut s = scene(config());
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","blocks_vision":false,"color":[0.2,0.7,0.5],"transform":{"position":[0,1.5,-3],"size":[5,3,0.5]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    let mut detour = false;
    for _ in 0..600 {
        e.advance(1).unwrap();
        if e.simulation.position("npc").unwrap()[0].abs() > 2.5 {
            detour = true;
        }
    }
    assert!(detour);
    assert_eq!(hp(&e, "player"), 70);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .all(|r| matches!(r.outcome, Outcome::Started | Outcome::Completed))
    );
    replay(&e);
}
#[test]
fn hidden_health_and_position_never_change_the_npc_decision() {
    let mut a = scene(config());
    let mut b = a.clone();
    for (s, x, health) in [(&mut a, -2., 20), (&mut b, 2., 90)] {
        let p = entity(s, "player");
        p.transform.position = [x, 0.8, -6.];
        p.vitality.as_mut().unwrap().initial = health;
        s.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","color":[0.2,0.7,0.5],"transform":{"position":[0,1.5,-3],"size":[12,3,0.5]}})).unwrap());
    }
    let mut a = Engine::new(a).unwrap();
    let mut b = Engine::new(b).unwrap();
    a.advance(30).unwrap();
    b.advance(30).unwrap();
    assert_eq!(a.simulation.npcs["npc"], b.simulation.npcs["npc"]);
    assert_eq!(phase(&a), Phase::NoVisibleTarget);
    assert!(a.simulation.abilities.state.receipts.is_empty());
    replay(&a);
    replay(&b);
}
#[test]
fn losing_sight_cancels_and_falls_back_to_memory_not_target_tracking() {
    let mut s = scene(config());
    entity(&mut s, "player").transform.position[2] = -2.;
    s.ability_catalog.abilities[0].cast_ticks = 120;
    let n = entity(&mut s, "npc").npc.as_mut().unwrap();
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":2.6,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":30,"search_turn_degrees_per_second":0})).unwrap());
    n.objectives.as_mut().unwrap()[1].action = ignis::objectives::Action::Respond;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_input","input":{"x":0,"z":-1}}))
            .unwrap()
            .into(),
    )
    .unwrap();
    e.advance(20).unwrap();
    assert_eq!(phase(&e), Phase::NoVisibleTarget);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::Cancelled)
    );
    let n = &e.simulation.npcs["npc"];
    assert!(!n.observed_this_tick);
    assert_ne!(
        n.memory.unwrap().position,
        e.simulation.position("player").unwrap()
    );
    replay(&e);
}
#[test]
fn xyz_range_and_missing_flight_do_not_become_success() {
    let mut s = scene(config());
    let p = entity(&mut s, "player");
    p.transform.position = [0., 6.8, 0.];
    s.entities.push(serde_json::from_value(json!({"id":"platform","name":"platform","body":"fixed","color":[0.2,0.7,0.5],"blocks_vision":false,"transform":{"position":[0,5.5,0],"size":[3,1,3]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(90).unwrap();
    assert_eq!(phase(&e), Phase::Approaching);
    assert_eq!(hp(&e, "player"), 40);
    assert!(e.simulation.abilities.state.receipts.is_empty());
    assert_eq!(e.simulation.navigation["npc"].reason, "target_off_surface");
    replay(&e);
}
#[test]
fn validates_version_threshold_and_approach_atomically() {
    for p in [
        json!({"vitality_below":{"subject":"target","basis_points":0}}),
        json!({"vitality_below":{"subject":"source","basis_points":10001}}),
        json!({"approach":{"margin":3}}),
        json!({"approach":{"margin":0}}),
    ] {
        assert!(Engine::new(scene(p)).is_err());
    }
    let mut s = scene(config());
    s.version = 27;
    assert!(s.validate().is_err());
    let mut s = scene(config());
    s.ability_catalog.abilities[0].selection = Selection::SelfActor;
    assert!(s.validate().is_err());
}
#[test]
fn integer_threshold_boundary_and_range_hysteresis() {
    let policy = Policy {
        require_dispel_match: false,
        allies: None,
        spatial_aim: None,
        vitality_below: Some(Threshold {
            subject: Recipient::Target,
            basis_points: 7000,
        }),
        approach: Some(Approach { margin: 0.4 }),
    };
    let mut offer = Offer {
        selection: Selection::Actor,
        range: 3.,
        relation: None,
        affordable: true,
        cast_target: None,
    };
    let mut e = Evidence {
        actor: 0,
        position: [0.; 3],
        health: None,
        target: Some(Target {
            actor: 1,
            position: [0., 0., -4.],
            health: Some(Health {
                units: 699_999_999_999,
                maximum: 1_000_000_000_000,
                scale: 1000,
            }),
            relation: ignis::targeting::Attitude::Neutral,
        }),
    };
    let approaching = ability_policy::evaluate(policy, &offer, &e, Decision::default());
    assert_eq!(approaching.phase, Phase::Approaching);
    e.target.as_mut().unwrap().position[2] = -2.8;
    assert_eq!(
        ability_policy::evaluate(policy, &offer, &e, approaching).phase,
        Phase::Approaching
    );
    e.target.as_mut().unwrap().position[2] = -2.5;
    let ready = ability_policy::evaluate(policy, &offer, &e, approaching);
    assert_eq!(ready.phase, Phase::Ready);
    e.target.as_mut().unwrap().position[2] = -2.8;
    assert_eq!(
        ability_policy::evaluate(policy, &offer, &e, ready).phase,
        Phase::Ready
    );
    e.target.as_mut().unwrap().health.as_mut().unwrap().units = 700_000_000_000;
    assert_eq!(
        ability_policy::evaluate(policy, &offer, &e, ready).phase,
        Phase::VitalitySatisfied
    );
    e.target.as_mut().unwrap().health.as_mut().unwrap().units = 1;
    offer.cast_target = Some(1);
    assert_eq!(
        ability_policy::evaluate(policy, &offer, &e, ready).phase,
        Phase::Casting
    );
    offer.cast_target = Some(2);
    assert_eq!(
        ability_policy::evaluate(policy, &offer, &e, ready).phase,
        Phase::TargetChanged
    );
    e.target = None;
    assert_eq!(
        ability_policy::evaluate(policy, &offer, &e, ready).phase,
        Phase::NoVisibleTarget
    );
}

#[test]
fn spatial_flyer_approaches_an_elevated_target_with_existing_motor() {
    let mut s = scene(config());
    entity(&mut s, "player").transform.position = [0., 6.8, -5.];
    s.entities.push(serde_json::from_value(json!({"id":"platform","name":"platform","body":"fixed","color":[0.2,0.7,0.5],"blocks_vision":false,"transform":{"position":[0,5.5,-5],"size":[3,1,3]}})).unwrap());
    // The straight segment crosses the platform side. Author a legal route above it.
    s.spatial_network = serde_json::from_value(
        json!({"nodes":[[0,6,0],[0,6,-5]],"links":[{"from":0,"to":1,"mode":"fly"}]}),
    )
    .unwrap();
    let n = entity(&mut s, "npc").npc.as_mut().unwrap();
    n.mobility = Some(
        serde_json::from_value(json!({"walk":true,"jump_speed":null,"fly_speed":3,"glide":null}))
            .unwrap(),
    );
    n.local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(360).unwrap();
    assert_eq!(
        hp(&e, "player"),
        70,
        "{:?} {:?} {:?}",
        e.simulation.mobility.states["npc"],
        e.simulation.npcs["npc"].objectives,
        e.simulation.position("npc")
    );
    assert!(e.simulation.position("npc").unwrap()[1] > 3.);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .all(|r| matches!(r.outcome, Outcome::Started | Outcome::Completed))
    );
    replay(&e);
}
#[test]
fn territory_and_relation_filter_before_approach() {
    for territory in [false, true] {
        let mut s = scene(config());
        if territory {
            entity(&mut s, "npc").npc.as_mut().unwrap().territory = Some(
                serde_json::from_value(
                    json!({"home":[0,0],"radius":3,"arrival_distance":0.3,"rearm_ticks":10}),
                )
                .unwrap(),
            );
        } else {
            s.ability_catalog.abilities[0].relation = Some(ignis::targeting::Attitude::Hostile);
        }
        let mut e = Engine::new(s).unwrap();
        e.advance(30).unwrap();
        assert!(e.simulation.position("npc").unwrap()[2].abs() < 0.001);
        assert!(e.simulation.abilities.state.receipts.is_empty());
        assert_eq!(
            phase(&e),
            if territory {
                Phase::NoVisibleTarget
            } else {
                Phase::RelationMismatch
            }
        );
        replay(&e);
    }
}
