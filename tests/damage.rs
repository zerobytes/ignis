use ignis::{
    abilities::{Effect, Outcome},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::{Value, json};
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
fn def(id: &str, effect: Value) -> Value {
    json!({"id":id,"selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[effect]})
}
fn packet(parts: Value) -> Value {
    json!({"kind":"damage_components","recipient":"target","components":parts})
}
fn mixed() -> Value {
    packet(json!([{"amount":40,"tags":["fire","magical"]},{"amount":60,"tags":["ice","magical"]}]))
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
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"Target","body":"npc","color":[0.6,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    let mut hit = def("mixed", mixed());
    hit["tags"] = json!(["theme"]);
    let mut buff = def(
        "resist",
        json!({"kind":"apply_buff","recipient":"target","buff":"resist"}),
    );
    buff["costs"] = json!([]);
    let mut immune = def(
        "immune",
        json!({"kind":"apply_buff","recipient":"target","buff":"immune"}),
    );
    immune["costs"] = json!([]);
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"attributes":{"tags":["fire","ice","magical","theme"],"buffs":[{"id":"resist","duration_ticks":600,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_received.multiplier","tags_all":["ice"],"operation":{"kind":"multiply","basis_points":5000}}]},{"id":"immune","duration_ticks":600,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_received.multiplier","operation":{"kind":"multiply","basis_points":0}}]}]},"abilities":[hit,buff,immune]})).unwrap();
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.footsteps = None;
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["mixed","resist","immune"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":1000,"maximum":1000,"scale":1})).unwrap());
    }
    entity(&mut s, "player").transform.position = [0., 0.8, -4.];
    s
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
    e.advance(1).unwrap();
}
fn hp(e: &Engine, id: &str) -> i64 {
    let a = e
        .simulation
        .actors
        .slot(e.scene.entities.iter().position(|v| v.id == id).unwrap());
    e.simulation.vitality.state.actors[a as usize]
        .unwrap()
        .units
}
fn energy(e: &Engine) -> i64 {
    let a = e.simulation.actors.slot(
        e.scene
            .entities
            .iter()
            .position(|v| v.id == "player")
            .unwrap(),
    );
    e.simulation.abilities.state.balances[a as usize][0]
}
fn report(e: &mut Engine) -> Value {
    e.execute(Command::InspectAbilities.into()).unwrap()
}
fn last_hit(e: &mut Engine) -> Value {
    report(e)["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|r| r["ability"] == "mixed" && !r["effects"].as_array().unwrap().is_empty())
        .unwrap()
        .clone()
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn effect(s: &mut Scene, value: Value) {
    s.ability_catalog.abilities[0].effects = vec![serde_json::from_value(value).unwrap()];
}
#[test]
fn mixed_damage_is_40_plus_30_with_one_effect_and_one_payment() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 930);
    assert_eq!(energy(&e), 95);
    let r = last_hit(&mut e);
    assert_eq!(r["effects"].as_array().unwrap().len(), 1);
    let x = &r["effects"][0];
    assert_eq!(x["requested"], 100);
    assert_eq!(x["adjusted_amount"], 70);
    assert_eq!(x["applied"], 70);
    assert_eq!(x["damage_components"][0]["adjusted"], 40);
    assert_eq!(x["damage_components"][1]["adjusted"], 30);
    assert!(x["damage_multiplier"].is_null());
    assert!(e.simulation.vitality.state.deaths.is_empty());
    replay(&e);
}
#[test]
fn permuting_components_preserves_total_and_emits_only_one_death() {
    for reverse in [false, true] {
        let mut s = scene();
        entity(&mut s, "target").vitality.as_mut().unwrap().initial = 50;
        if reverse
            && let Effect::DamageComponents { components, .. } =
                &mut s.ability_catalog.abilities[0].effects[0]
        {
            components.reverse();
        }
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "player", 2, "target");
        cast(&mut e, "player", 1, "target");
        assert_eq!(hp(&e, "target"), 0);
        assert_eq!(e.simulation.vitality.state.deaths.len(), 1);
        let r = last_hit(&mut e);
        assert_eq!(r["effects"][0]["applied"], 50);
        assert_eq!(r["effects"][0]["vitality"]["excess"], 20);
        assert_eq!(r["effects"][0]["damage_before_limit"], 70);
        replay(&e);
    }
}
#[test]
fn multiple_classifications_are_distinct_from_separate_quantities() {
    let mut s = scene();
    effect(
        &mut s,
        packet(json!([{"amount":100,"tags":["fire","ice"]}])),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 950);
    replay(&e);
}
#[test]
fn shared_magical_rule_and_element_rule_compose_for_each_component() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].modifiers.push(serde_json::from_value(json!({"attribute":"combat.damage_received.multiplier","tags_all":["magical"],"operation":{"kind":"multiply","basis_points":5000}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 965);
    replay(&e);
}
#[test]
fn ability_theme_does_not_leak_into_components_or_scalar_damage() {
    for scalar in [false, true] {
        let mut s = scene();
        s.ability_catalog.abilities[0].tags = vec!["ice".into()];
        if scalar {
            effect(
                &mut s,
                json!({"kind":"damage","recipient":"target","amount":100}),
            );
        } else {
            effect(&mut s, packet(json!([{"amount":100,"tags":["fire"]}])));
        }
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "player", 2, "target");
        cast(&mut e, "player", 1, "target");
        assert_eq!(hp(&e, "target"), 900);
        replay(&e);
    }
}
#[test]
fn inspect_damage_attribute_uses_explicit_component_context_and_ability_id() {
    let mut s = scene();
    s.ability_catalog.abilities[0].tags = vec!["ice".into()];
    s.ability_catalog.attributes.buffs[0].modifiers[0].abilities = vec!["mixed".into()];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    for (tags, expected) in [
        (vec![], 10000),
        (vec!["ice".into()], 5000),
        (vec!["fire".into()], 10000),
    ] {
        let v = e
            .execute(
                Command::InspectAttribute {
                    entity: "target".into(),
                    attribute: "combat.damage_received.multiplier".into(),
                    ability: Some("mixed".into()),
                    tags,
                }
                .into(),
            )
            .unwrap();
        assert_eq!(v["effective"]["units"], expected);
    }
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 930);
    replay(&e);
}
#[test]
fn unknown_and_duplicate_tags_are_rejected_at_all_reference_sites() {
    for case in 0..7 {
        let mut v = serde_json::to_value(scene()).unwrap();
        let a = &mut v["ability_catalog"];
        match case {
            0 => a["abilities"][0]["effects"][0]["components"][0]["tags"] = json!(["fier"]),
            1 => a["abilities"][0]["tags"] = json!(["fier"]),
            2 => a["attributes"]["buffs"][0]["modifiers"][0]["tags_all"] = json!(["fier"]),
            3 => a["attributes"]["tags"] = json!(["fire", "ice", "magical", "theme", "fire"]),
            4 => a["abilities"][0]["effects"][0]["components"][0]["tags"] = json!(["fire", "fire"]),
            5 => a["abilities"][0]["tags"] = json!(["theme", "theme"]),
            _ => a["attributes"]["buffs"][0]["modifiers"][0]["tags_all"] = json!(["ice", "ice"]),
        };
        assert!(
            serde_json::from_value::<Scene>(v)
                .unwrap()
                .validate()
                .is_err(),
            "{case}"
        );
    }
}
#[test]
fn component_amount_version_and_total_budgets_are_validated() {
    for case in 0..7 {
        let mut v = serde_json::to_value(scene()).unwrap();
        match case {
            0 => v["version"] = json!(38),
            1 => v["ability_catalog"]["abilities"][0]["effects"][0]["components"] = json!([]),
            2 => {
                v["ability_catalog"]["abilities"][0]["effects"][0]["components"] =
                    json!(vec![json!({"amount":1,"tags":[]}); 9])
            }
            3 => {
                v["ability_catalog"]["abilities"][0]["effects"][0]["components"][0]["amount"] =
                    json!(0)
            }
            4 => {
                v["ability_catalog"]["abilities"][0]["effects"][0]["components"][0]["amount"] =
                    json!(-1)
            }
            5 => {
                v["ability_catalog"]["abilities"][0]["effects"][0]["components"][0]["amount"] =
                    json!(1_000_000_000_000_i64)
            }
            _ => {
                v["ability_catalog"]["abilities"][0]["effects"] =
                    json!((0..5).map(|_| mixed()).collect::<Vec<_>>())
            }
        };
        assert!(
            serde_json::from_value::<Scene>(v)
                .unwrap()
                .validate()
                .is_err(),
            "{case}"
        );
    }
    assert!(serde_json::from_value::<Effect>(json!({"kind":"damage_components","recipient":"target","amount":100,"components":[{"amount":100,"tags":[]}]})).is_err());
}
#[test]
fn component_rounding_is_explicit_and_order_independent() {
    let mut s = scene();
    effect(
        &mut s,
        packet(json!([{"amount":1,"tags":["ice"]},{"amount":1,"tags":["ice"]}])),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 1000);
    let r = last_hit(&mut e);
    assert_eq!(r["effects"][0]["damage_before_limit"], 0);
    assert_eq!(r["effects"][0]["applied"], 0);
    replay(&e);
}
#[test]
fn numeric_saturation_reports_unclamped_sum_separately_from_overkill() {
    let mut s = scene();
    effect(
        &mut s,
        packet(
            json!([{"amount":500_000_000_000_i64,"tags":["ice"]},{"amount":500_000_000_000_i64,"tags":["ice"]}]),
        ),
    );
    s.ability_catalog.attributes.buffs[0].modifiers[0].operation =
        ignis::attributes::Operation::Multiply {
            basis_points: 1_000_000,
        };
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    let r = last_hit(&mut e);
    let x = &r["effects"][0];
    assert_eq!(x["damage_before_limit"], 100_000_000_000_000_i64);
    assert_eq!(x["adjusted_amount"], 1_000_000_000_000_i64);
    assert_eq!(x["damage_limit_excess"], 99_000_000_000_000_i64);
    assert_eq!(x["vitality"]["excess"], 999_999_999_000_i64);
    assert_eq!(e.simulation.vitality.state.deaths.len(), 1);
    replay(&e);
}
#[test]
fn late_failure_rolls_back_components_life_and_cost() {
    let mut s = scene();
    entity(&mut s, "target").abilities = None;
    s.ability_catalog.abilities[0].effects.push(
        serde_json::from_value(
            json!({"kind":"resource","recipient":"target","resource":"energy","delta":1}),
        )
        .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 1000);
    assert_eq!(energy(&e), 100);
    let r = e.simulation.abilities.state.receipts.back().unwrap();
    assert!(r.effects.iter().all(Option::is_none));
    assert!(r.damage_components.iter().all(Option::is_none));
    replay(&e);
}
#[test]
fn newly_applied_immunity_does_not_change_same_transaction_components() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects.insert(
        0,
        serde_json::from_value(json!({"kind":"apply_buff","recipient":"target","buff":"immune"}))
            .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 930);
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 930);
    replay(&e);
}
#[test]
fn projectile_uses_target_resistance_at_impact_and_pays_once() {
    let mut s = scene();
    s.ability_catalog.abilities[0].delivery = serde_json::from_value(
        json!({"kind":"projectile","speed":6,"half_extent":0.1,"max_distance":20}),
    )
    .unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "target");
    assert_eq!(energy(&e), 95);
    assert_eq!(hp(&e, "target"), 1000);
    cast(&mut e, "player", 2, "target");
    e.advance(60).unwrap();
    assert_eq!(hp(&e, "target"), 930);
    assert_eq!(energy(&e), 95);
    assert_eq!(last_hit(&mut e)["outcome"], "impacted");
    replay(&e);
}
#[test]
fn periodic_components_recheck_resistance_each_pulse_without_repaying() {
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic = Some(
        serde_json::from_value(json!({"interval_ticks":10,"pulses":3,"reapply":"independent"}))
            .unwrap(),
    );
    s.ability_catalog.attributes.buffs[0].duration_ticks = 12;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "target");
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "target"), 900);
    cast(&mut e, "player", 2, "target");
    e.advance(19).unwrap();
    assert_eq!(hp(&e, "target"), 730);
    assert_eq!(energy(&e), 95);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .filter(|r| r.outcome == Outcome::PeriodicPulse)
            .count(),
        3
    );
    replay(&e);
}
#[test]
fn channel_pulses_keep_component_transaction_and_payment_contract() {
    let mut s = scene();
    s.ability_catalog.abilities[0].channel = Some(
        serde_json::from_value(json!({"interval_ticks":10,"pulses":3,"cost_mode":"on_start"}))
            .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    e.advance(35).unwrap();
    assert_eq!(hp(&e, "target"), 790);
    assert_eq!(energy(&e), 95);
    assert_eq!(last_hit(&mut e)["effects"][0]["adjusted_amount"], 70);
    replay(&e);
}
#[test]
fn cancel_before_commit_preserves_life_and_payment() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 20;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "target");
    let id = report(&mut e)["actors"]["player"]["cast"]["execution_id"]
        .as_u64()
        .unwrap();
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: id,
        }
        .into(),
    )
    .unwrap();
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "target"), 1000);
    assert_eq!(energy(&e), 100);
    replay(&e);
}
#[test]
fn legacy_38_migrates_inferred_tags_and_preserves_tagged_scalar_damage() {
    let mut s = scene();
    s.version = 38;
    s.ability_catalog.attributes.tags.clear();
    s.ability_catalog.abilities[0].tags = vec!["ice".into(), "ice".into()];
    effect(
        &mut s,
        json!({"kind":"damage","recipient":"target","amount":100}),
    );
    let mut e = Engine::new(s).unwrap();
    assert_eq!(e.scene.version, ignis::model::SCENE_VERSION);
    e.scene.validate().unwrap();
    assert!(
        matches!(&e.scene.ability_catalog.abilities[0].effects[0],Effect::DamageComponents{components,..} if components[0].tags==vec!["ice"])
    );
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 950);
    let loaded: Scene = serde_json::from_str(&serde_json::to_string(&e.scene).unwrap()).unwrap();
    loaded.validate().unwrap();
    replay(&e);
}
#[test]
fn arbitrary_game_tags_have_no_builtin_element_meaning() {
    let mut s = scene();
    s.ability_catalog.attributes.tags.push("damage_x".into());
    s.ability_catalog.attributes.buffs[0].modifiers[0].tags_all = vec!["damage_x".into()];
    effect(&mut s, packet(json!([{"amount":100,"tags":["damage_x"]}])));
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 950);
    replay(&e);
}
#[test]
fn eight_components_share_receipt_capacity_and_keep_effect_indices() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects = vec![serde_json::from_value(mixed()).unwrap(); 4];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 720);
    let r = last_hit(&mut e);
    for i in 0..4 {
        assert_eq!(r["effects"][i]["effect_index"], i);
        assert_eq!(
            r["effects"][i]["damage_components"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(r["effects"][i]["applied"], 70);
    }
    replay(&e);
}
#[test]
fn npc_cannot_apply_components_to_an_unobserved_target() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"Wall","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[0,1.5,-2],"size":[8,3,0.4]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "target", 1, "player");
    assert_eq!(hp(&e, "player"), 1000);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::TargetUnavailable
    );
    replay(&e);
}
#[test]
fn source_recipient_uses_sources_resistance_and_vitality() {
    let mut s = scene();
    effect(
        &mut s,
        json!({"kind":"damage_components","recipient":"source","components":[{"amount":40,"tags":["fire"]},{"amount":60,"tags":["ice"]}]}),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, "player");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "player"), 930);
    assert_eq!(hp(&e, "target"), 1000);
    assert_eq!(last_hit(&mut e)["effects"][0]["actor"], "player");
    replay(&e);
}
#[test]
fn observed_npc_uses_same_packet_and_receipt_as_player() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "player", 2, "player");
    e.advance(1).unwrap();
    cast(&mut e, "target", 1, "player");
    assert_eq!(hp(&e, "player"), 930);
    let r = last_hit(&mut e);
    assert_eq!(r["actor"], "target");
    assert_eq!(r["effects"][0]["adjusted_amount"], 70);
    replay(&e);
}

// Outgoing damage shares component context and the pre-transaction buff snapshot.
fn outgoing(s: &mut Scene, units: i64, tags: &[&str]) {
    let buff = &mut s.ability_catalog.attributes.buffs[1];
    buff.id = "power".into();
    buff.category = Some("power".into());
    buff.modifiers = serde_json::from_value(json!([{"attribute":"combat.damage_dealt.multiplier","tags_all":tags,"operation":{"kind":"multiply","basis_points":units}}])).unwrap();
    s.ability_catalog.abilities[2].effects = vec![
        serde_json::from_value(json!({"kind":"apply_buff","recipient":"target","buff":"power"}))
            .unwrap(),
    ];
}
#[test]
fn outgoing_and_incoming_factors_have_separate_owners_and_component_context() {
    let mut s = scene();
    outgoing(&mut s, 12500, &["fire"]);
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "player");
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 920);
    let r = last_hit(&mut e);
    let c = &r["effects"][0]["damage_components"];
    assert_eq!(c[0]["dealt_multiplier"], 12500);
    assert_eq!(c[0]["received_multiplier"], 10000);
    assert_eq!(c[1]["dealt_multiplier"], 10000);
    assert_eq!(c[1]["received_multiplier"], 5000);
    assert_eq!(c[0]["adjusted"], 50);
    assert_eq!(c[1]["adjusted"], 30);
    replay(&e);
}
#[test]
fn outgoing_rounds_once_after_both_factors_and_preserves_incoming_alias() {
    let mut s = scene();
    outgoing(&mut s, 5000, &[]);
    effect(
        &mut s,
        json!({"kind":"damage","recipient":"target","amount":3}),
    );
    s.ability_catalog.attributes.buffs[0].modifiers[0]
        .tags_all
        .clear();
    s.ability_catalog.attributes.buffs[0].modifiers[0].operation =
        ignis::attributes::Operation::Multiply {
            basis_points: 20000,
        };
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "player");
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 997);
    let r = last_hit(&mut e);
    assert_eq!(r["effects"][0]["damage_multiplier"], 20000);
    assert_eq!(r["effects"][0]["damage_components"][0]["multiplier"], 20000);
    replay(&e);
}
#[test]
fn outgoing_filters_do_not_inherit_skill_themes_or_target_buffs() {
    for filter in ["theme", "fire"] {
        let mut s = scene();
        outgoing(&mut s, 20000, &[filter]);
        let mut e = Engine::new(s).unwrap();
        cast(
            &mut e,
            "player",
            3,
            if filter == "theme" {
                "player"
            } else {
                "target"
            },
        );
        cast(&mut e, "player", 1, "target");
        assert_eq!(hp(&e, "target"), 900);
        replay(&e);
    }
}
#[test]
fn outgoing_inspection_and_ability_filter_match_consumption() {
    let mut s = scene();
    outgoing(&mut s, 20000, &["fire"]);
    s.ability_catalog.attributes.buffs[1].modifiers[0].abilities = vec!["mixed".into()];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "player");
    for (ability, tags, expected) in [
        (Some("mixed"), vec!["fire"], 20000),
        (None, vec!["fire"], 10000),
        (Some("mixed"), vec![], 10000),
    ] {
        let v = e
            .execute(
                Command::InspectAttribute {
                    entity: "player".into(),
                    attribute: "combat.damage_dealt.multiplier".into(),
                    ability: ability.map(Into::into),
                    tags: tags.into_iter().map(Into::into).collect(),
                }
                .into(),
            )
            .unwrap();
        assert_eq!(v["effective"]["units"], expected);
    }
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 860);
    replay(&e);
}
#[test]
fn outgoing_extreme_product_is_bounded_without_intermediate_saturation() {
    let mut s = scene();
    outgoing(&mut s, 1_000_000, &[]);
    effect(
        &mut s,
        json!({"kind":"damage","recipient":"target","amount":1_000_000_000_000_i64}),
    );
    s.ability_catalog.attributes.buffs[0].modifiers[0]
        .tags_all
        .clear();
    s.ability_catalog.attributes.buffs[0].modifiers[0].operation =
        ignis::attributes::Operation::Multiply {
            basis_points: 1_000_000,
        };
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "player");
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "player", 1, "target");
    let r = last_hit(&mut e);
    assert_eq!(
        r["effects"][0]["damage_before_limit"],
        10_000_000_000_000_000_i64
    );
    assert_eq!(r["effects"][0]["adjusted_amount"], 1_000_000_000_000_i64);
    assert_eq!(e.simulation.vitality.state.deaths.len(), 1);
    replay(&e);
}
#[test]
fn outgoing_new_buff_is_visible_only_after_transaction_commit() {
    let mut s = scene();
    outgoing(&mut s, 20000, &[]);
    s.ability_catalog.abilities[0].effects.insert(
        0,
        serde_json::from_value(json!({"kind":"apply_buff","recipient":"source","buff":"power"}))
            .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 900);
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 700);
    replay(&e);
}
#[test]
fn outgoing_buff_and_damage_rollback_on_later_failure() {
    let mut s = scene();
    outgoing(&mut s, 20000, &[]);
    entity(&mut s, "target").abilities = None;
    s.ability_catalog.abilities[0].effects.insert(
        0,
        serde_json::from_value(json!({"kind":"apply_buff","recipient":"source","buff":"power"}))
            .unwrap(),
    );
    s.ability_catalog.abilities[0].effects.push(
        serde_json::from_value(
            json!({"kind":"resource","recipient":"target","resource":"energy","delta":1}),
        )
        .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 1000);
    assert_eq!(energy(&e), 100);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    replay(&e);
}
#[test]
fn outgoing_projectile_uses_buff_gained_after_launch_or_expired_before_impact() {
    for expires in [false, true] {
        let mut s = scene();
        outgoing(&mut s, 20000, &[]);
        if expires {
            s.ability_catalog.attributes.buffs[1].duration_ticks = 5;
        }
        s.ability_catalog.abilities[0].delivery = serde_json::from_value(
            json!({"kind":"projectile","speed":6,"half_extent":0.1,"max_distance":20}),
        )
        .unwrap();
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "player", 1, "target");
        cast(&mut e, "player", 3, "player");
        e.advance(60).unwrap();
        assert_eq!(hp(&e, "target"), if expires { 900 } else { 800 });
        assert_eq!(energy(&e), 95);
        replay(&e);
    }
}
#[test]
fn outgoing_periodic_and_channel_pulses_recheck_buff_expiration() {
    for channel in [false, true] {
        let mut s = scene();
        outgoing(&mut s, 20000, &[]);
        s.ability_catalog.attributes.buffs[1].duration_ticks = 15;
        if channel {
            s.ability_catalog.abilities[0].channel = Some(
                serde_json::from_value(
                    json!({"interval_ticks":10,"pulses":3,"cost_mode":"on_start"}),
                )
                .unwrap(),
            );
        } else {
            s.ability_catalog.abilities[0].periodic = Some(
                serde_json::from_value(
                    json!({"interval_ticks":10,"pulses":3,"reapply":"independent"}),
                )
                .unwrap(),
            );
        }
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "player", 3, "player");
        cast(&mut e, "player", 1, "target");
        e.advance(35).unwrap();
        assert_eq!(hp(&e, "target"), 600);
        assert_eq!(energy(&e), 95);
        replay(&e);
    }
}
#[test]
fn outgoing_persistent_damage_survives_source_death_without_source_buff() {
    let mut s = scene();
    outgoing(&mut s, 20000, &[]);
    s.ability_catalog.abilities[0].periodic = Some(
        serde_json::from_value(json!({"interval_ticks":10,"pulses":3,"reapply":"independent"}))
            .unwrap(),
    );
    s.ability_catalog.abilities[1].effects = vec![
        serde_json::from_value(json!({"kind":"damage","recipient":"source","amount":1000}))
            .unwrap(),
    ];
    // Static source power remains after its temporary buff is removed on death.
    s.ability_catalog.attributes.bases=serde_json::from_value(json!([{"entity":"player","attribute":"combat.damage_dealt.multiplier","value":{"kind":"number","units":15000}}])).unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "player");
    cast(&mut e, "player", 1, "target");
    cast(&mut e, "player", 2, "target");
    assert_eq!(hp(&e, "player"), 0);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    e.advance(35).unwrap();
    assert_eq!(hp(&e, "target"), 550);
    replay(&e);
}
#[test]
fn outgoing_dispel_keeps_start_snapshot_for_same_execution() {
    let mut s = scene();
    outgoing(&mut s, 20000, &[]);
    // Dispel is target-only, so use a self-targeted damage execution.
    s.ability_catalog.abilities[0].effects.insert(0,serde_json::from_value(json!({"kind":"dispel","recipient":"target","category":"power","max_instances":1,"require_match":false})).unwrap());
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "player");
    cast(&mut e, "player", 1, "player");
    assert_eq!(hp(&e, "player"), 800);
    cast(&mut e, "player", 1, "player");
    assert_eq!(hp(&e, "player"), 700);
    replay(&e);
}
#[test]
fn outgoing_npc_source_and_source_recipient_share_executor() {
    let mut s = scene();
    outgoing(&mut s, 20000, &["fire"]);
    if let Effect::DamageComponents { recipient, .. } =
        &mut s.ability_catalog.abilities[0].effects[0]
    {
        *recipient = ignis::abilities::Recipient::Source;
    }
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "target");
    cast(&mut e, "player", 2, "target");
    cast(&mut e, "target", 1, "player");
    assert_eq!(hp(&e, "target"), 890);
    assert_eq!(hp(&e, "player"), 1000);
    replay(&e);
}
#[test]
fn outgoing_zero_factor_does_not_block_heal_or_charge() {
    let mut s = scene();
    outgoing(&mut s, 0, &[]);
    entity(&mut s, "target").vitality.as_mut().unwrap().initial = 900;
    s.ability_catalog.abilities[0].effects.push(
        serde_json::from_value(json!({"kind":"heal","recipient":"target","amount":50})).unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 3, "player");
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 950);
    assert_eq!(energy(&e), 95);
    replay(&e);
}
#[test]
fn outgoing_reserved_key_and_old_version_reject_without_reinterpreting_custom_data() {
    let mut s = scene();
    s.version = 39;
    outgoing(&mut s, 20000, &[]);
    assert!(s.validate().unwrap_err().to_string().contains("version40"));
    let mut s = scene();
    s.version = 39;
    s.ability_catalog.attributes.definitions = serde_json::from_value(
        json!([{"key":"combat.damage_dealt.multiplier","default":{"kind":"bool","value":true}}]),
    )
    .unwrap();
    assert!(
        s.validate()
            .unwrap_err()
            .to_string()
            .contains("rename the custom key")
    );
}
#[test]
fn outgoing_addition_preserves_61_custom_keys_and_neutral_legacy_damage() {
    let mut s = scene();
    s.version = 39;
    s.ability_catalog.attributes.definitions = (0..61)
        .map(|i| {
            serde_json::from_value(
                json!({"key":format!("custom.{i}"),"default":{"kind":"number","units":0}}),
            )
            .unwrap()
        })
        .collect();
    let mut e = Engine::new(s).unwrap();
    assert_eq!(e.scene.version, ignis::model::SCENE_VERSION);
    e.scene.validate().unwrap();
    cast(&mut e, "player", 1, "target");
    assert_eq!(hp(&e, "target"), 900);
    replay(&e);
}
