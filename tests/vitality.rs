use ignis::{
    abilities::*,
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    vitality::Config,
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
        {"id":"strike","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":3}],"effects":[{"kind":"damage","recipient":"target","amount":30}]},
        {"id":"heal","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":3}],"effects":[{"kind":"heal","recipient":"target","amount":30}]},
        {"id":"self","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"source","amount":100}]}
    ]})).unwrap();
    let p = entity(&mut s, "player");
    p.transform.position = [0., 0.8, -3.];
    p.footsteps = None;
    let npc:Entity=serde_json::from_value(json!({"id":"npc","name":"npc","body":"npc","color":[0.8,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","target":"player","speed":2,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":180,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap();
    s.entities.push(npc);
    for id in ["player", "npc"] {
        let e = entity(&mut s, id);
        e.vitality = Some(Config {
            initial: 50,
            maximum: 100,
            scale: 10,
        });
        e.abilities = Some(Loadout {
            resources: vec![Pool {
                resource: "energy".into(),
                initial: 100,
                capacity: 100,
            }],
            grants: vec!["strike".into(), "heal".into(), "self".into()],
        });
    }
    s
}
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
fn activate(e: &mut Engine, actor: &str, slot: u8, target: Option<&str>) {
    e.execute(
        Command::ActivateAbility {
            actor: actor.into(),
            slot,
            target: target.map(str::to_owned),
        }
        .into(),
    )
    .unwrap();
}
fn units(e: &Engine, id: &str) -> i64 {
    let a = e
        .simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap();
    e.simulation.vitality.state.actors[a].unwrap().units
}
fn outcome(e: &Engine) -> Outcome {
    e.simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .outcome
}
fn energy(e: &mut Engine, id: &str) -> i64 {
    e.execute(Command::InspectResources.into()).unwrap()["actors"][id]["resources"][0]["units"]
        .as_i64()
        .unwrap()
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn integer_damage_heal_receipts_and_full_heal_cost() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, Some("npc"));
    e.advance(1).unwrap();
    assert_eq!(units(&e, "npc"), 20);
    let receipt = e
        .simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .effects[0]
        .unwrap();
    assert_eq!((receipt.requested, receipt.applied), (30, 30));
    assert_eq!(receipt.vitality.unwrap().before, 50);
    for _ in 0..4 {
        activate(&mut e, "player", 2, Some("npc"));
        e.advance(1).unwrap();
    }
    assert_eq!(units(&e, "npc"), 100);
    assert_eq!(energy(&mut e, "player"), 85);
    let applied = e
        .simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .effects[0]
        .unwrap();
    assert_eq!(applied.applied, 0);
    assert_eq!(applied.vitality.unwrap().excess, 30);
    assert!(e.simulation.vitality.state.deaths.is_empty());
    replay(&e);
}
#[test]
fn lethal_plan_can_heal_before_commit_and_reports_last_zero_cause() {
    let mut s = scene();
    s.ability_catalog.abilities[2].effects.push(Effect::Heal {
        recipient: Recipient::Source,
        amount: 25,
    });
    let mut e = Engine::new(s.clone()).unwrap();
    activate(&mut e, "player", 3, None);
    e.advance(1).unwrap();
    assert_eq!(units(&e, "player"), 25);
    assert!(e.simulation.vitality.state.deaths.is_empty());
    s.ability_catalog.abilities[2].effects.push(Effect::Damage {
        recipient: Recipient::Source,
        amount: 100,
    });
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 3, None);
    e.advance(1).unwrap();
    assert_eq!(units(&e, "player"), 0);
    let death = e.simulation.vitality.state.deaths[0];
    assert_eq!(death.effect_index, 2);
    let effects = e
        .simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .effects;
    assert_eq!(effects[0].unwrap().vitality.unwrap().excess, 50);
    assert_eq!(effects[2].unwrap().vitality.unwrap().excess, 75);
    replay(&e);
}
#[test]
fn missing_vitality_rejects_entire_plan_including_cost_and_noise() {
    let mut s = scene();
    entity(&mut s, "npc").vitality = None;
    s.ability_catalog.abilities[0]
        .effects
        .insert(0, Effect::Noise { radius: 10. });
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, Some("npc"));
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::MissingVitality);
    assert_eq!(energy(&mut e, "player"), 100);
    assert_eq!(e.simulation.abilities.state.emitted, 0);
    assert_eq!(e.simulation.abilities.state.next_execution, 1);
}
#[test]
fn later_invalid_resource_rolls_back_lethal_damage() {
    let mut s = scene();
    entity(&mut s, "npc").abilities = None;
    s.ability_catalog.abilities[2].selection = Selection::Actor;
    s.ability_catalog.abilities[2].range = 20.;
    s.ability_catalog.abilities[2].effects = vec![
        Effect::Damage {
            recipient: Recipient::Target,
            amount: 100,
        },
        Effect::Resource {
            recipient: Recipient::Target,
            resource: "energy".into(),
            delta: 1,
        },
    ];
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 3, Some("npc"));
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::MissingResource);
    assert_eq!(units(&e, "npc"), 50);
    assert!(e.simulation.vitality.state.deaths.is_empty());
}
#[test]
fn dead_target_and_caster_reject_without_resurrection_or_debit() {
    let mut s = scene();
    entity(&mut s, "npc").vitality.as_mut().unwrap().initial = 0;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 2, Some("npc"));
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::TargetDead);
    activate(&mut e, "npc", 2, Some("npc"));
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::ActorDead);
    assert_eq!(energy(&mut e, "player"), 100);
    assert_eq!(energy(&mut e, "npc"), 100);
    assert_eq!(units(&e, "npc"), 0);
    assert!(e.simulation.vitality.state.deaths.is_empty());
    assert_eq!(
        e.simulation.npcs["npc"].phase,
        ignis::perception::Phase::Dead
    );
    replay(&e);
}
#[test]
fn lexical_instant_order_and_death_cancel_pending_cast() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects = vec![Effect::Damage {
        recipient: Recipient::Target,
        amount: 100,
    }];
    s.ability_catalog.abilities[1].cast_ticks = 10;
    let mut e = Engine::new(s.clone()).unwrap();
    activate(&mut e, "player", 2, Some("player"));
    e.advance(1).unwrap();
    activate(&mut e, "npc", 1, Some("player"));
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::CancelledByDeath);
    assert_eq!(units(&e, "player"), 0);
    assert_eq!(energy(&mut e, "player"), 100);
    e.advance(20).unwrap();
    assert_eq!(units(&e, "player"), 0);
    replay(&e);
    for reverse in [false, true] {
        if reverse {
            s.entities.reverse();
        }
        let mut e = Engine::new(s.clone()).unwrap();
        activate(&mut e, "player", 1, Some("npc"));
        activate(&mut e, "npc", 1, Some("player"));
        e.advance(1).unwrap();
        assert_eq!(units(&e, "player"), 0);
        assert_eq!(units(&e, "npc"), 50);
        assert_eq!(outcome(&e), Outcome::ActorDead);
        assert_eq!(energy(&mut e, "player"), 100);
        assert_eq!(energy(&mut e, "npc"), 97);
    }
}
#[test]
fn existing_cast_phase_precedes_new_instant_cast() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 1;
    s.ability_catalog.abilities[0].effects = vec![Effect::Damage {
        recipient: Recipient::Target,
        amount: 100,
    }];
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, Some("npc"));
    e.advance(1).unwrap();
    activate(&mut e, "npc", 3, None);
    e.advance(1).unwrap();
    assert_eq!(units(&e, "npc"), 0);
    assert_eq!(outcome(&e), Outcome::ActorDead);
    assert_eq!(e.simulation.vitality.state.deaths.len(), 1);
    replay(&e);
}
#[test]
fn player_motor_stops_on_same_tick_death_and_body_remains() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(30).unwrap();
    let before = e.simulation.position("player").unwrap();
    e.execute(
        Command::SetInput {
            input: ignis::physics::Input {
                x: 1.,
                z: 0.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    activate(&mut e, "player", 3, None);
    e.advance(30).unwrap();
    let after = e.simulation.position("player").unwrap();
    assert!((after[0] - before[0]).abs() < 0.001);
    assert!(
        e.simulation
            .contacts()
            .iter()
            .any(|(a, b)| a == "player" || b == "player")
    );
    assert!(
        e.execute(
            Command::EmitNoise {
                origin: ignis::noise::NoiseOrigin::Entity {
                    id: "player".into()
                },
                radius: 5.
            }
            .into()
        )
        .is_err()
    );
    replay(&e);
}
#[test]
fn flying_corpse_keeps_inertia_but_falls_without_motor() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-spatial-patrol.scene.json")).unwrap();
    s.version = 27;
    s.ability_catalog = scene().ability_catalog;
    let n = entity(&mut s, "flyer");
    n.vitality = Some(Config {
        initial: 50,
        maximum: 100,
        scale: 1,
    });
    n.abilities = Some(Loadout {
        resources: vec![],
        grants: vec!["self".into()],
    });
    let mut e = Engine::new(s).unwrap();
    for _ in 0..600 {
        e.advance(1).unwrap();
        if e.simulation.mobility.states["flyer"].phase == ignis::mobility::Phase::Traversing {
            break;
        }
    }
    let before = e.simulation.position("flyer").unwrap();
    let previous = e.simulation.mobility.states["flyer"].clone();
    assert_eq!(previous.phase, ignis::mobility::Phase::Traversing);
    activate(&mut e, "flyer", 1, None);
    e.advance(1).unwrap();
    let first = e.simulation.position("flyer").unwrap();
    assert_ne!(first, before);
    assert!(!e.simulation.mobility.states["flyer"].committed);
    assert!(e.simulation.mobility.states["flyer"].flight.is_none());
    e.advance(120).unwrap();
    assert!(e.simulation.position("flyer").unwrap()[1] < before[1] - 1.);
    assert_eq!(
        e.simulation.npcs["flyer"].phase,
        ignis::perception::Phase::Dead
    );
    assert!(e.simulation.npcs["flyer"].memory.is_none());
    replay(&e);
}
#[test]
fn author_validation_rejects_legacy_and_invalid_bounds_atomically() {
    let s = scene();
    for (initial, maximum, scale) in [
        (-1, 100, 1),
        (101, 100, 1),
        (0, 0, 1),
        (50, 100, 0),
        (50, 100, 1_000_001),
    ] {
        let mut bad = s.clone();
        entity(&mut bad, "player").vitality = Some(Config {
            initial,
            maximum,
            scale,
        });
        assert!(Engine::new(bad).is_err());
    }
    let mut bad = s.clone();
    bad.version = 26;
    assert!(Engine::new(bad).is_err());
    let mut bad = s.clone();
    entity(&mut bad, "floor").vitality = Some(Config {
        initial: 1,
        maximum: 1,
        scale: 1,
    });
    assert!(Engine::new(bad).is_err());
    for amount in [0, -1, LIMIT + 1] {
        let mut bad = s.clone();
        bad.ability_catalog.abilities[0].effects = vec![Effect::Damage {
            recipient: Recipient::Target,
            amount,
        }];
        assert!(Engine::new(bad).is_err());
    }
    let mut e = Engine::new(s.clone()).unwrap();
    activate(&mut e, "player", 1, Some("npc"));
    e.advance(1).unwrap();
    let before = e.snapshot();
    let mut changed = s.entities.iter().find(|e| e.id == "npc").unwrap().clone();
    changed.vitality.as_mut().unwrap().maximum = 0;
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(changed)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
#[test]
fn pending_prefix_final_replay_and_reset_preserve_vitality_contract() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, Some("npc"));
    let pending = e.replay();
    let mut restored = Engine::replay_at(&pending, None).unwrap();
    restored.advance(1).unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.vitality.state,
        restored.simulation.vitality.state
    );
    let snapshot = e.snapshot();
    activate(&mut e, "player", 1, Some("npc"));
    e.advance(1).unwrap();
    replay(&e);
    assert_eq!(
        Engine::replay_at(&e.replay(), Some(1)).unwrap().snapshot()["vitality"],
        snapshot["vitality"]
    );
    let mut corrupt = e.replay();
    corrupt.expected_vitality.actors[0].as_mut().unwrap().units = 77;
    assert_eq!(Engine::verify_replay(&corrupt).unwrap()["matched"], false);
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(units(&e, "npc"), 50);
    assert!(e.simulation.vitality.state.deaths.is_empty());
}

#[test]
fn target_dying_during_cast_revalidates_without_charging_caster() {
    let mut s = scene();
    s.ability_catalog.abilities[1].cast_ticks = 5;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 2, Some("npc"));
    e.advance(1).unwrap();
    activate(&mut e, "npc", 3, None);
    e.advance(1).unwrap();
    e.advance(4).unwrap();
    assert_eq!(outcome(&e), Outcome::TargetDead);
    assert_eq!(energy(&mut e, "player"), 100);
    assert_eq!(units(&e, "npc"), 0);
    replay(&e);
}
#[test]
fn dead_automatic_npc_has_no_actions_or_active_knowledge() {
    let mut s = scene();
    let n = entity(&mut s, "npc");
    n.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":2,"when":"always","action":{"kind":"use_ability","slot":3,"retry_ticks":1}},{"priority":1,"when":"always","action":{"kind":"respond"}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(units(&e, "npc"), 0);
    let count = e.simulation.abilities.state.receipts.len();
    e.advance(120).unwrap();
    assert_eq!(e.simulation.abilities.state.receipts.len(), count);
    assert!(e.simulation.npcs["npc"].memory.is_none());
    assert!(e.simulation.npcs["npc"].objectives.active.is_none());
    replay(&e);
}

#[test]
fn sliding_corpse_does_not_generate_footsteps() {
    let mut s = scene();
    entity(&mut s, "player").footsteps = Some(ignis::model::FootstepConfig {
        surface: "floor".into(),
        stride: 0.2,
        radius: 5.,
        min_speed: 0.1,
    });
    let mut e = Engine::new(s).unwrap();
    e.advance(30).unwrap();
    e.execute(
        Command::SetInput {
            input: ignis::physics::Input {
                x: 1.,
                z: 0.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(30).unwrap();
    let steps = e.simulation.footsteps.total_emitted;
    assert!(steps > 0);
    let before = e.simulation.position("player").unwrap();
    activate(&mut e, "player", 3, None);
    e.advance(1).unwrap();
    assert!(e.simulation.position("player").unwrap()[0] > before[0]);
    e.advance(60).unwrap();
    assert_eq!(e.simulation.footsteps.total_emitted, steps);
    assert!(e.simulation.footsteps.pending.is_empty());
    replay(&e);
}

#[test]
fn damage_provenance_does_not_create_sensory_knowledge() {
    let mut s = scene();
    entity(&mut s, "npc").npc.as_mut().unwrap().target = None;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, Some("npc"));
    e.advance(1).unwrap();
    assert_eq!(units(&e, "npc"), 20);
    assert!(e.simulation.npcs["npc"].memory.is_none());
    assert!(!e.simulation.npcs["npc"].observed_this_tick);
    replay(&e);
}
