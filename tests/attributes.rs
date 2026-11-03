use ignis::{
    abilities::{Effect, Outcome, Recipient},
    attributes::{Context, Value},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::json;
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
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
    s.version = 38;
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"Target","body":"npc","color":[0.6,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    let buffs = json!([
        {"id":"ward","duration_ticks":10,"reapply":"independent","category":"protection","modifiers":[{"attribute":"ability.interruptible","operation":{"kind":"set","value":{"kind":"bool","value":false}}}]},
        {"id":"poison_guard","duration_ticks":10,"reapply":"refresh","modifiers":[{"attribute":"effect.dispellable","tags_all":["poison"],"operation":{"kind":"set","value":{"kind":"bool","value":false}}}]},
        {"id":"resist","duration_ticks":10,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_received.multiplier","tags_all":["ice"],"operation":{"kind":"multiply","basis_points":0}},{"attribute":"combat.damage_received.multiplier","tags_all":["fire"],"operation":{"kind":"multiply","basis_points":15000}}]},
        {"id":"immune","duration_ticks":10,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_received.multiplier","operation":{"kind":"multiply","basis_points":0}}]}
    ]);
    let mut abilities = Vec::new();
    for (id, buff) in [
        ("ward", "ward"),
        ("guard", "poison_guard"),
        ("resist", "resist"),
        ("immune", "immune"),
    ] {
        abilities.push(json!({"id":id,"selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"apply_buff","recipient":"target","buff":buff}]}));
    }
    abilities.extend([
        json!({"id":"long","selection":"self_actor","range":0,"cast_ticks":100,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"heal","recipient":"target","amount":5}]}),
        json!({"id":"poison","selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"tags":["poison"],"periodic":{"interval_ticks":100,"pulses":3,"reapply":"independent"},"periodic_category":"poison","costs":[],"effects":[{"kind":"damage","recipient":"target","amount":2}]}),
        json!({"id":"dispel","selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"dispel","recipient":"target","category":"poison","max_instances":1,"require_match":true}]}),
        json!({"id":"interrupt","selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"interrupt","recipient":"target","require_active":true}]})
    ]);
    for tag in ["fire", "ice", "physical"] {
        abilities.push(json!({"id":tag,"tags":[tag],"selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":10}]}));
    }
    let grants: Vec<_> = abilities
        .iter()
        .map(|a| a["id"].as_str().unwrap().to_string())
        .collect();
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"attributes":{"buffs":buffs},"abilities":abilities})).unwrap();
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.footsteps = None;
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":grants})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
    }
    entity(&mut s, "player").transform.position = [0., 0.8, -4.];
    s
}
fn index(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap()
}
fn hp(e: &Engine, id: &str) -> i64 {
    e.simulation.vitality.state.actors[index(e, id)]
        .unwrap()
        .units
}
fn energy(e: &Engine, id: &str) -> i64 {
    e.simulation.abilities.state.balances[index(e, id)][0]
}
fn cast(e: &mut Engine, actor: &str, slot: u8, target: &str) {
    e.execute(
        Command::ActivateAbility {
            actor: actor.into(),
            slot,
            target: Some(target.into()),
        }
        .into(),
    )
    .unwrap();
}
fn tick(e: &mut Engine) {
    e.advance(1).unwrap();
}
fn outcome(e: &Engine, outcome: Outcome) -> usize {
    e.simulation
        .abilities
        .state
        .receipts
        .iter()
        .filter(|r| r.outcome == outcome)
        .count()
}
fn verify(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn attr(e: &mut Engine, owner: &str, key: &str, ability: Option<&str>) -> serde_json::Value {
    e.execute(
        Command::InspectAttribute {
            entity: owner.into(),
            attribute: key.into(),
            ability: ability.map(Into::into),
            tags: vec![],
        }
        .into(),
    )
    .unwrap()
}
#[test]
fn ward_protects_current_cast_then_expires_at_exact_tick() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "target", 1, "target");
    tick(&mut e);
    cast(&mut e, "target", 5, "target");
    tick(&mut e);
    cast(&mut e, "player", 8, "target");
    tick(&mut e);
    assert_eq!(outcome(&e, Outcome::NoInterruptibleExecution), 1);
    assert_eq!(energy(&e, "player"), 100);
    let v = attr(&mut e, "target", "ability.interruptible", Some("long"));
    assert_eq!(v["effective"]["value"], false);
    assert_eq!(v["base"]["value"], true);
    assert_eq!(v["contributions"][0]["source"], "target");
    e.advance(7).unwrap();
    assert_eq!(
        attr(&mut e, "target", "ability.interruptible", Some("long"))["effective"]["value"],
        true
    );
    cast(&mut e, "player", 8, "target");
    tick(&mut e);
    assert_eq!(outcome(&e, Outcome::InterruptedByAbility), 1);
    assert_eq!(energy(&e, "player"), 95);
    verify(&e);
}
#[test]
fn overlapping_buffs_remove_only_their_own_contribution() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "player", 1, "player");
    e.advance(5).unwrap();
    cast(&mut e, "player", 1, "player");
    e.advance(5).unwrap();
    let v = attr(&mut e, "player", "ability.interruptible", None);
    assert_eq!(v["effective"]["value"], false);
    assert_eq!(v["contributions"].as_array().unwrap().len(), 1);
    e.advance(5).unwrap();
    assert_eq!(
        attr(&mut e, "player", "ability.interruptible", None)["effective"]["value"],
        true
    );
    assert_eq!(energy(&e, "player"), 90);
    verify(&e);
}
#[test]
fn refresh_renews_deadline_and_records_replacement() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].reapply = ignis::periodic::Reapply::Refresh;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "player");
    e.advance(5).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 1);
    assert_eq!(e.simulation.abilities.state.buffs.active[0].expires_at, 15);
    assert!(
        e.simulation
            .abilities
            .state
            .buffs
            .receipts
            .iter()
            .any(|r| r.outcome == ignis::attributes::End::Refreshed)
    );
    verify(&e);
}
#[test]
fn elemental_immunity_vulnerability_and_general_immunity_share_numeric_evaluator() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "target", 3, "target");
    tick(&mut e);
    cast(&mut e, "player", 10, "target");
    tick(&mut e);
    assert_eq!(hp(&e, "target"), 100);
    cast(&mut e, "player", 9, "target");
    tick(&mut e);
    assert_eq!(hp(&e, "target"), 85);
    let r = e
        .simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .effects[0]
        .unwrap();
    assert_eq!(
        (r.requested, r.adjusted_amount, r.applied),
        (10, Some(15), 15)
    );
    assert_eq!(r.vitality.unwrap().excess, 0);
    cast(&mut e, "player", 11, "target");
    tick(&mut e);
    assert_eq!(hp(&e, "target"), 75);
    cast(&mut e, "target", 4, "target");
    tick(&mut e);
    cast(&mut e, "player", 9, "target");
    tick(&mut e);
    assert_eq!(hp(&e, "target"), 75);
    e.advance(10).unwrap();
    cast(&mut e, "player", 10, "target");
    tick(&mut e);
    assert_eq!(hp(&e, "target"), 65);
    verify(&e);
}
#[test]
fn poison_protection_follows_author_not_recipient_and_covers_existing_effects() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "player", 6, "target");
    tick(&mut e);
    cast(&mut e, "player", 2, "player");
    tick(&mut e);
    cast(&mut e, "target", 7, "target");
    tick(&mut e);
    assert_eq!(outcome(&e, Outcome::NoMatchingEffect), 1);
    assert_eq!(energy(&e, "target"), 100);
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 1);
    let v = e.execute(Command::InspectAbilities.into()).unwrap();
    assert_eq!(v["periodic"]["active"][0]["dispellable"], false);
    e.advance(8).unwrap();
    cast(&mut e, "target", 7, "target");
    tick(&mut e);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    assert_eq!(energy(&e, "target"), 95);
    verify(&e);
}
#[test]
fn another_authors_poison_is_not_protected_by_recipient_buff() {
    let mut e = Engine::new(scene()).unwrap();
    tick(&mut e);
    cast(&mut e, "target", 6, "player");
    tick(&mut e);
    cast(&mut e, "player", 2, "player");
    tick(&mut e);
    cast(&mut e, "player", 7, "player");
    tick(&mut e);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    verify(&e);
}
#[test]
fn late_operator_failure_rolls_back_buff_damage_and_payment() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects.extend([
        Effect::Damage {
            recipient: Recipient::Target,
            amount: 10,
        },
        Effect::Dispel {
            recipient: Recipient::Target,
            category: "poison".into(),
            max_instances: 1,
            require_match: true,
        },
    ]);
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    assert_eq!(hp(&e, "player"), 100);
    assert_eq!(energy(&e, "player"), 100);
    verify(&e);
}
#[test]
fn buffs_become_visible_after_atomic_execution_not_mid_effect_list() {
    let mut s = scene();
    s.ability_catalog.abilities[3].effects.push(Effect::Damage {
        recipient: Recipient::Target,
        amount: 10,
    });
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 4, "player");
    tick(&mut e);
    assert_eq!(hp(&e, "player"), 90);
    cast(&mut e, "player", 11, "player");
    tick(&mut e);
    assert_eq!(hp(&e, "player"), 90);
    verify(&e);
}
#[test]
fn dispelling_one_buff_preserves_other_protection_and_static_non_dispellable() {
    let mut s = scene();
    s.ability_catalog.abilities[6].effects = vec![Effect::Dispel {
        recipient: Recipient::Target,
        category: "protection".into(),
        max_instances: 1,
        require_match: true,
    }];
    let mut e = Engine::new(s.clone()).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    cast(&mut e, "player", 7, "player");
    tick(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 1);
    assert_eq!(
        attr(&mut e, "player", "ability.interruptible", None)["effective"]["value"],
        false
    );
    verify(&e);
    s.ability_catalog.attributes.buffs[0].dispellable = false;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    cast(&mut e, "player", 7, "player");
    tick(&mut e);
    assert_eq!(outcome(&e, Outcome::NoMatchingEffect), 1);
    assert_eq!(energy(&e, "player"), 95);
    verify(&e);
}
#[test]
fn custom_attribute_uses_same_operations_bounds_and_entity_bases() {
    let mut s = scene();
    let cfg = &mut s.ability_catalog.attributes;
    cfg.definitions=serde_json::from_value(json!([{"key":"craft.temperature","default":{"kind":"number","units":10000},"minimum":0,"maximum":100000}])).unwrap();
    cfg.bases=serde_json::from_value(json!([{"entity":"player","attribute":"craft.temperature","value":{"kind":"number","units":20000}},{"entity":"floor","attribute":"craft.temperature","value":{"kind":"number","units":50000}}])).unwrap();
    cfg.buffs[0].modifiers=serde_json::from_value(json!([
        {"attribute":"craft.temperature","priority":0,"operation":{"kind":"add","units":10000}},
        {"attribute":"craft.temperature","priority":1,"operation":{"kind":"multiply","basis_points":20000}},
        {"attribute":"craft.temperature","priority":2,"operation":{"kind":"min","units":55000}},
        {"attribute":"craft.temperature","priority":3,"operation":{"kind":"max","units":56000}}
    ])).unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    let v = attr(&mut e, "player", "craft.temperature", None);
    assert_eq!(v["effective"]["units"], 56000);
    assert_eq!(v["contributions"].as_array().unwrap().len(), 4);
    assert_eq!(
        attr(&mut e, "floor", "craft.temperature", None)["effective"]["units"],
        50000
    );
    e.advance(10).unwrap();
    assert_eq!(
        attr(&mut e, "player", "craft.temperature", None)["effective"]["units"],
        20000
    );
    verify(&e);
}
#[test]
fn ability_and_tag_filters_apply_only_to_matching_context() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].modifiers[0].abilities = vec!["long".into()];
    s.ability_catalog.attributes.buffs[0].modifiers[0].tags_all = vec!["spell".into()];
    s.ability_catalog.abilities[4].tags = vec!["spell".into()];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    assert_eq!(
        attr(&mut e, "player", "ability.interruptible", Some("long"))["effective"]["value"],
        false
    );
    assert_eq!(
        attr(&mut e, "player", "ability.interruptible", Some("fire"))["effective"]["value"],
        true
    );
    assert_eq!(
        attr(&mut e, "player", "ability.interruptible", None)["effective"]["value"],
        true
    );
    verify(&e);
}
#[test]
fn source_death_does_not_remove_ally_buff_owner_death_does() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 100;
    s.ability_catalog.abilities[10].effects = vec![Effect::Damage {
        recipient: Recipient::Target,
        amount: 100,
    }];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "target");
    tick(&mut e);
    cast(&mut e, "player", 11, "player");
    tick(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 1);
    cast(&mut e, "target", 11, "target");
    tick(&mut e);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    verify(&e);
}
#[test]
fn registry_rejects_unknown_keys_types_bounds_duplicates_and_versions() {
    let base = scene();
    for case in 0..8 {
        let mut s = base.clone();
        match case {
            0 => s.version = 37,
            1 => s.ability_catalog.attributes.buffs[0].modifiers[0].attribute = "unknown".into(),
            2 => {
                s.ability_catalog.attributes.buffs[0].modifiers[0].operation =
                    ignis::attributes::Operation::Add { units: 1 }
            }
            3 => s.ability_catalog.attributes.buffs[0].duration_ticks = 0,
            4 => {
                s.ability_catalog.attributes.buffs[0].modifiers[0].abilities =
                    vec!["unknown".into()]
            }
            5 => {
                s.ability_catalog.attributes.definitions = serde_json::from_value(
                    json!([{"key":"ability.interruptible","default":{"kind":"bool","value":true}}]),
                )
                .unwrap()
            }
            6 => s.ability_catalog.attributes.definitions = serde_json::from_value(
                json!([{"key":"x","default":{"kind":"number","units":2},"minimum":4,"maximum":3}]),
            )
            .unwrap(),
            _ => s.ability_catalog.attributes.tags = (0..65).map(|i| format!("tag{i}")).collect(),
        }
        assert!(s.validate().is_err(), "case {case}");
    }
}
#[test]
fn direct_registry_can_modify_non_actor_owners_without_gameplay_specific_dispatch() {
    let s = scene();
    let registry = ignis::attributes::Registry::new(&s, &["protection".into()]);
    let mut state = ignis::attributes::State::default();
    let floor = s.entities.iter().position(|e| e.id == "floor").unwrap() as u16;
    registry.install(
        &mut state,
        ignis::attributes::Install {
            buff: 0,
            owner_instance: ignis::targeting::Catalog::new(&s)
                .entity_reference(floor as usize)
                .unwrap(),
            source_instance: ignis::targeting::Catalog::new(&s)
                .entity_reference(floor as usize)
                .unwrap(),
            owner: floor,
            source: floor,
            execution_id: 1,
        },
        0,
    );
    assert_eq!(
        registry.evaluate(
            &ignis::targeting::Catalog::new(&s),
            &state,
            floor,
            0,
            Value::boolean(true),
            Context::default(),
            0
        ),
        Value::boolean(false)
    );
    assert_eq!(
        registry.evaluate(
            &ignis::targeting::Catalog::new(&s),
            &state,
            floor,
            0,
            Value::boolean(true),
            Context::default(),
            10
        ),
        Value::boolean(true)
    );
}

#[test]
fn full_pool_rejects_new_instance_atomically_but_refresh_and_dispel_reuse_capacity() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 1000;
    s.ability_catalog.abilities[0].costs.clear();
    s.ability_catalog.abilities[6].effects = vec![
        Effect::Dispel {
            recipient: Recipient::Target,
            category: "protection".into(),
            max_instances: 1,
            require_match: true,
        },
        Effect::ApplyBuff {
            recipient: Recipient::Target,
            buff: "ward".into(),
        },
    ];
    let mut e = Engine::new(s).unwrap();
    for _ in 0..64 {
        cast(&mut e, "player", 1, "player");
        tick(&mut e);
    }
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 64);
    cast(&mut e, "player", 4, "player");
    tick(&mut e);
    assert_eq!(outcome(&e, Outcome::BuffBudget), 1);
    assert_eq!(energy(&e, "player"), 100);
    cast(&mut e, "player", 7, "player");
    tick(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 64);
    assert_eq!(energy(&e, "player"), 95);
    verify(&e);
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 1000;
    s.ability_catalog.abilities[0].costs.clear();
    s.ability_catalog.abilities[2].costs.clear();
    let mut e = Engine::new(s).unwrap();
    for _ in 0..63 {
        cast(&mut e, "player", 1, "player");
        tick(&mut e);
    }
    cast(&mut e, "player", 3, "player");
    tick(&mut e);
    cast(&mut e, "player", 3, "player");
    tick(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 64);
    assert_eq!(outcome(&e, Outcome::BuffBudget), 0);
    verify(&e);
}
#[test]
fn multiple_pending_installations_reserve_capacity_and_refresh_keys() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 1000;
    s.ability_catalog.abilities[0].costs.clear();
    s.ability_catalog.abilities[3].effects = vec![
        Effect::ApplyBuff {
            recipient: Recipient::Target,
            buff: "ward".into()
        };
        2
    ];
    let mut e = Engine::new(s).unwrap();
    for _ in 0..63 {
        cast(&mut e, "player", 1, "player");
        tick(&mut e);
    }
    cast(&mut e, "player", 4, "player");
    tick(&mut e);
    assert_eq!(outcome(&e, Outcome::BuffBudget), 1);
    assert_eq!(energy(&e, "player"), 100);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 63);
    verify(&e);
    let mut s = scene();
    s.ability_catalog.abilities[3].effects = vec![
        Effect::ApplyBuff {
            recipient: Recipient::Target,
            buff: "immune".into()
        };
        2
    ];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 4, "player");
    tick(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 1);
    assert_eq!(energy(&e, "player"), 95);
    verify(&e);
}
#[test]
fn projectile_applies_buff_to_physical_recipient_and_damage_uses_impact_time_resistance() {
    let mut s = scene();
    s.ability_catalog.abilities[0].selection = ignis::abilities::Selection::Spatial;
    s.ability_catalog.abilities[0].range = 0.;
    s.ability_catalog.abilities[0].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 12.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    s.ability_catalog.attributes.buffs[0].duration_ticks = 100;
    let mut e = Engine::new(s).unwrap();
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 1,
            aim: ignis::abilities::Aim::Direction {
                vector: [0., 0., 1.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(25).unwrap();
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 1);
    let b = e.simulation.abilities.state.buffs.active[0];
    assert_eq!(e.scene.entities[b.owner as usize].id, "target");
    assert!(b.applied_tick > 0);
    assert_eq!(b.expires_at, b.applied_tick + 100);
    verify(&e);
    let mut s = scene();
    s.ability_catalog.abilities[8].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 6.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    s.ability_catalog.attributes.buffs[3].duration_ticks = 100;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 9, "target");
    tick(&mut e);
    cast(&mut e, "target", 4, "target");
    e.advance(50).unwrap();
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(outcome(&e, Outcome::Impacted), 1);
    verify(&e);
}
#[test]
fn periodic_damage_rechecks_current_resistance_each_pulse() {
    let mut s = scene();
    s.ability_catalog.abilities[8].periodic = Some(ignis::periodic::Schedule {
        interval_ticks: 5,
        pulses: 3,
        reapply: ignis::periodic::Reapply::Refresh,
    });
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 9, "target");
    tick(&mut e);
    cast(&mut e, "target", 4, "target");
    e.advance(15).unwrap();
    assert_eq!(hp(&e, "target"), 90);
    assert_eq!(outcome(&e, Outcome::PeriodicPulse), 3);
    verify(&e);
}
#[test]
fn hidden_buff_target_is_not_available_to_npc_and_failed_cast_does_not_debit() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"Wall","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[0,1.5,-2],"size":[8,3,0.4]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    tick(&mut e);
    cast(&mut e, "target", 1, "player");
    tick(&mut e);
    assert_eq!(outcome(&e, Outcome::TargetUnavailable), 1);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    assert_eq!(energy(&e, "target"), 100);
    verify(&e);
}
#[test]
fn priority_overrides_and_numeric_saturation_are_deterministic() {
    let mut s = scene();
    let mut other = s.ability_catalog.attributes.buffs[0].clone();
    other.id = "override".into();
    other.modifiers[0].priority = 10;
    other.modifiers[0].operation = ignis::attributes::Operation::Set {
        value: Value::boolean(true),
    };
    s.ability_catalog.attributes.buffs.push(other);
    s.ability_catalog.abilities[3].effects = vec![Effect::ApplyBuff {
        recipient: Recipient::Target,
        buff: "override".into(),
    }];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 4, "player");
    tick(&mut e);
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    assert_eq!(
        attr(&mut e, "player", "ability.interruptible", None)["effective"]["value"],
        true
    );
    verify(&e);
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].modifiers=serde_json::from_value(json!([{"attribute":"combat.damage_received.multiplier","operation":{"kind":"add","units":1000000000000_i64}},{"attribute":"combat.damage_received.multiplier","operation":{"kind":"multiply","basis_points":1000000}}])).unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    assert_eq!(
        attr(&mut e, "player", "combat.damage_received.multiplier", None)["effective"]["units"],
        1000000
    );
    verify(&e);
}
#[test]
fn base_protection_and_custom_boolean_are_restored_after_expiry() {
    let mut s = scene();
    s.ability_catalog.abilities[4].interruptible = false;
    s.ability_catalog.attributes.definitions = serde_json::from_value(
        json!([{"key":"quest.allowed","default":{"kind":"bool","value":true}}]),
    )
    .unwrap();
    s.ability_catalog.attributes.buffs[0].modifiers.push(serde_json::from_value(json!({"attribute":"quest.allowed","operation":{"kind":"set","value":{"kind":"bool","value":false}}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "player");
    tick(&mut e);
    assert_eq!(
        attr(&mut e, "player", "quest.allowed", None)["effective"]["value"],
        false
    );
    e.advance(10).unwrap();
    assert_eq!(
        attr(&mut e, "player", "quest.allowed", None)["effective"]["value"],
        true
    );
    assert_eq!(
        attr(&mut e, "player", "ability.interruptible", Some("long"))["effective"]["value"],
        false
    );
    verify(&e);
}

#[test]
fn spatial_buff_impact_at_full_pool_fails_without_refunding_launch() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 1000;
    s.ability_catalog.abilities[0].costs.clear();
    s.ability_catalog.abilities[3].selection = ignis::abilities::Selection::Spatial;
    s.ability_catalog.abilities[3].range = 0.;
    s.ability_catalog.abilities[3].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 12.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    let mut e = Engine::new(s).unwrap();
    for _ in 0..64 {
        cast(&mut e, "player", 1, "player");
        tick(&mut e);
    }
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 4,
            aim: ignis::abilities::Aim::Direction {
                vector: [0., 0., 1.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(25).unwrap();
    assert_eq!(energy(&e, "player"), 95);
    assert_eq!(outcome(&e, Outcome::Launched), 1);
    assert_eq!(outcome(&e, Outcome::BuffBudget), 1);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 64);
    verify(&e);
}
#[test]
fn npc_objective_applies_buff_to_observed_target_with_shared_retry() {
    let mut s = scene();
    entity(&mut s,"target").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":30}},{"priority":0,"when":"always","action":{"kind":"wait"}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    assert_eq!(energy(&e, "target"), 95);
    let v = attr(&mut e, "player", "ability.interruptible", None);
    assert_eq!(v["effective"]["value"], false);
    assert_eq!(v["contributions"][0]["source"], "target");
    e.advance(20).unwrap();
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    assert_eq!(energy(&e, "target"), 95);
    verify(&e);
}
#[test]
fn buff_receipt_ring_is_bounded_and_reports_drops() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 1;
    s.ability_catalog.abilities[0].costs.clear();
    let mut e = Engine::new(s).unwrap();
    for _ in 0..140 {
        cast(&mut e, "player", 1, "player");
        tick(&mut e);
    }
    tick(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.receipts.len(), 256);
    assert_eq!(e.simulation.abilities.state.buffs.dropped_receipts, 24);
    verify(&e);
}
