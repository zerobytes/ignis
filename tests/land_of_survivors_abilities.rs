#[path = "../games/land-of-survivors/src/combat.rs"]
mod combat;
#[path = "../games/land-of-survivors/src/lab.rs"]
mod lab;
use combat::*;
fn game(spec: &str, focus: Focus) -> Game {
    let class = match spec {
        "arms" => "warrior",
        "marksmanship" => "archer",
        "nature" => "druid",
        _ => "mage",
    };
    let mut g = lab::game(class, spec, focus).unwrap();
    for f in g.state.fighters.values_mut() {
        f.health = 500.;
        f.mana = 300.;
    }
    g
}
fn cast(g: &mut Game, id: &str) {
    let a = g.catalog.get(id).unwrap().clone();
    let aim = lab::aim(g, &a);
    g.cast("player", id, aim).unwrap();
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-6, "{a} != {b}");
}
#[test]
fn full_catalog_has_six_twenty_choice_kits_and_two_ten_choice_forms() {
    let c = Catalog::bundled().unwrap();
    assert_eq!(c.abilities.len(), 284);
    for spec in ["fire", "frost", "aether", "arms", "marksmanship", "nature"] {
        for focus in [Focus::Dps, Focus::Healer, Focus::Tank] {
            let g = game(spec, focus);
            assert_eq!(g.available("player").len(), 20);
            assert_eq!(g.bar("player").len(), 10);
        }
    }
    for form in [Form::Bear, Form::Feline] {
        let mut g = game("nature", Focus::Tank);
        g.state.fighters.get_mut("player").unwrap().form = form;
        assert_eq!(g.available("player").len(), 10);
    }
}
#[test]
fn every_definition_executes_in_native_physics_and_has_a_functional_effect() {
    for a in Catalog::bundled().unwrap().abilities {
        let mut g = game(&a.specialization, a.focuses[0]);
        g.state.fighters.get_mut("player").unwrap().form = a.form;
        lab::place_for_ability(&mut g, &a).unwrap();
        let old = g.position("player");
        cast(&mut g, &a.id);
        g.step((ticks(a.cast + a.duration) + 240).min(3600) as u32)
            .unwrap();
        assert!(
            g.state
                .events
                .iter()
                .any(|e| e.ability == a.id && e.kind == "committed"),
            "{} did not commit",
            a.id
        );
        assert!(
            g.state
                .events
                .iter()
                .any(|e| e.ability == a.id && e.target.is_some() && e.amount > 0.)
                || a.effects.iter().any(|e| e.kind == EffectKind::Form)
                    && g.state.fighters["player"].form == a.effects[0].form
                || a.delivery == Delivery::Dash && old.distance(g.position("player")) > 0.01,
            "{} had no effect",
            a.id
        );
        assert_eq!(g.engine.simulation.tick, g.state.tick);
    }
}
#[test]
fn fireball_cast_cost_flight_explosion_and_eight_burn_pulses_are_distinct() {
    let mut g = game("fire", Focus::Dps);
    g.cast_slot(
        "player",
        1,
        Aim {
            target: Some("enemy".into()),
            ..Default::default()
        },
    )
    .unwrap();
    g.step(119).unwrap();
    close(g.state.fighters["player"].mana, 300.);
    close(g.state.fighters["enemy"].health, 500.);
    g.step(1).unwrap();
    close(g.state.fighters["player"].mana, 285.);
    assert_eq!(g.state.shots.len(), 1);
    close(g.state.fighters["enemy"].health, 500.);
    // Raise enemy life so all eight burn pulses can be observed after the600 burst.
    for id in ["enemy", "enemy2", "enemy3"] {
        g.state.fighters.get_mut(id).unwrap().health = 1000.;
    }
    g.step(30).unwrap();
    close(g.state.fighters["enemy"].health, 400.);
    close(g.state.fighters["ally"].health, 500.);
    g.step(480).unwrap();
    close(g.state.fighters["enemy"].health, 320.);
    assert!(g.state.shots.is_empty());
    assert!(g.state.pulses.is_empty());
    assert!(
        g.cast(
            "player",
            "los.mage.fire.fireball",
            Aim {
                direction: Some([0., 0., -1.]),
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn cancelled_and_rejected_casts_do_not_debit_or_start_cooldown() {
    let mut g = game("fire", Focus::Dps);
    cast(&mut g, "los.mage.fire.fireball");
    g.step(119).unwrap();
    g.cancel("player").unwrap();
    g.step(2).unwrap();
    close(g.state.fighters["player"].mana, 300.);
    assert!(g.state.fighters["player"].cooldowns.is_empty());
    assert!(g.state.shots.is_empty());
    assert!(
        g.cast("player", "los.mage.fire.fire_circle", Aim::default())
            .is_err()
    );
    let before = g.state.clone();
    g.state.fighters.get_mut("player").unwrap().mana = 0.;
    assert!(
        g.cast("player", "los.mage.fire.fireball", Aim::default())
            .is_err()
    );
    assert_eq!(g.state.events, before.events);
    assert!(
        g.cast("player", "los.mage.frost.ice_bolt", Aim::default())
            .is_err()
    );
}
#[test]
fn circle_reselects_recipients_and_applies_damage_and_healing_separately() {
    let mut g = game("fire", Focus::Dps);
    cast(&mut g, "los.mage.fire.fire_circle");
    g.step(36).unwrap();
    close(g.state.fighters["enemy"].health, 489.);
    close(g.state.fighters["ally"].health, 514.);
    g.movement("enemy", [1., 0.]).unwrap();
    g.step(120).unwrap();
    g.movement("enemy", [0., 0.]).unwrap();
    let hp = g.state.fighters["enemy"].health;
    g.step(360).unwrap();
    close(g.state.fighters["enemy"].health, hp);
    close(g.state.fighters["ally"].health, 612.);
}
#[test]
fn ground_root_and_cleanse_control_actual_character_motion() {
    let mut g = game("frost", Focus::Dps);
    cast(&mut g, "los.mage.frost.frozen_ground");
    g.step(36).unwrap();
    let p = g.position("enemy");
    g.movement("enemy", [1., 0.]).unwrap();
    g.step(60).unwrap();
    assert!((g.position("enemy").x - p.x).abs() < 0.001);
    assert!(
        g.state.fighters["enemy"]
            .statuses
            .iter()
            .any(|s| s.kind == EffectKind::Root)
    );
    // A same-team healer can dispel the movement impairment immediately.
    g.state.fighters.get_mut("ally").unwrap().team = "hostiles".into();
    g.cast(
        "ally",
        "los.mage.frost.thaw",
        Aim {
            target: Some("enemy".into()),
            ..Default::default()
        },
    )
    .unwrap();
    g.step(30).unwrap();
    assert!(g.position("enemy").x - p.x > 2.8);
    assert!(
        !g.state.fighters["enemy"]
            .statuses
            .iter()
            .any(|s| s.kind == EffectKind::Root)
    );
}
#[test]
fn shields_mitigation_armor_and_execution_change_actual_health() {
    let mut g = game("fire", Focus::Tank);
    cast(&mut g, "los.mage.fire.ember_ward");
    close(g.state.fighters["player"].shield(), 70.);
    let actual = g.damage("enemy", "player", "test", 100., false);
    close(actual, 30.);
    close(g.state.fighters["player"].health, 470.);
    // An explicit mitigation ability is independent from shields and armor.
    let a = g
        .catalog
        .abilities
        .iter()
        .find(|a| {
            a.specialization == "fire"
                && a.focuses == [Focus::Tank]
                && a.effects.iter().any(|e| e.kind == EffectKind::Mitigation)
                && a.delivery == Delivery::SelfActor
        })
        .unwrap()
        .clone();
    cast(&mut g, &a.id);
    let m = a
        .effects
        .iter()
        .find(|e| e.kind == EffectKind::Mitigation)
        .unwrap()
        .amount;
    close(
        g.damage("enemy", "player", "test", 100., false),
        100. * (1. - m),
    );
    g.state.fighters.get_mut("enemy").unwrap().health = 250.;
    close(g.damage("player", "enemy", "test", 68., true), 136.);
    g.state.fighters.get_mut("enemy").unwrap().health = 251.;
    close(g.damage("player", "enemy", "test", 68., true), 68.);
}
#[test]
fn projectiles_hit_physical_obstructions_and_never_track_a_moving_target() {
    let mut g = game("frost", Focus::Dps);
    cast(&mut g, "los.mage.frost.ice_bolt");
    g.step(51).unwrap();
    g.movement("enemy", [1., 0.]).unwrap();
    g.step(150).unwrap();
    close(g.state.fighters["enemy"].health, 500.);
    assert!(g.state.shots.is_empty());
    let mut g = game("frost", Focus::Dps);
    let wall:ignis::model::Entity=serde_json::from_value(serde_json::json!({"id":"wall","name":"wall","body":"fixed","color":[0.5,0.5,0.5],"transform":{"position":[0,1,-2],"size":[4,2,0.2]}})).unwrap();
    let mut s = g.engine.scene.clone();
    s.entities.push(wall);
    g = Game::new(s, g.catalog, g.state.fighters).unwrap();
    assert!(
        g.cast(
            "player",
            "los.mage.frost.ice_bolt",
            Aim {
                target: Some("enemy".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
    g.cast(
        "player",
        "los.mage.frost.ice_bolt",
        Aim {
            direction: Some([0., 0., -1.]),
            ..Default::default()
        },
    )
    .unwrap();
    g.step(120).unwrap();
    close(g.state.fighters["enemy"].health, 500.);
    assert!(
        g.state
            .events
            .iter()
            .any(|e| e.kind == "impact" && e.target.as_deref() == Some("wall"))
    );
}
#[test]
fn chain_jumps_to_distinct_targets_and_multishot_launches_real_projectiles() {
    let mut g = game("nature", Focus::Dps);
    cast(&mut g, "los.druid.nature.dps_08");
    for id in ["enemy", "enemy2", "enemy3", "enemy4"] {
        close(g.state.fighters[id].health, 464.);
    }
    let mut g = game("marksmanship", Focus::Dps);
    let a = g
        .catalog
        .abilities
        .iter()
        .find(|a| a.specialization == "marksmanship" && a.shots == 3)
        .unwrap()
        .clone();
    cast(&mut g, &a.id);
    g.step(ticks(a.cast) as u32).unwrap();
    assert_eq!(g.state.shots.len(), 3);
    g.step(30).unwrap();
    assert_eq!(
        g.state
            .events
            .iter()
            .filter(|e| e.ability == a.id && e.kind == "impact")
            .count(),
        3
    );
}
#[test]
fn cones_obey_direction_and_channels_cancel_remaining_pulses() {
    let mut g = game("frost", Focus::Dps);
    let id = "los.mage.frost.cold_front";
    g.cast(
        "player",
        id,
        Aim {
            direction: Some([0., 0., 1.]),
            ..Default::default()
        },
    )
    .unwrap();
    g.step(240).unwrap();
    close(g.state.fighters["enemy"].health, 500.);
    let mut g = game("frost", Focus::Dps);
    cast(&mut g, id);
    close(g.state.fighters["enemy"].health, 475.);
    g.cancel("player").unwrap();
    g.step(240).unwrap();
    close(g.state.fighters["enemy"].health, 475.);
}
#[test]
fn forms_replace_bar_without_healing_and_death_restores_humanoid() {
    let mut g = game("nature", Focus::Dps);
    let before = g.bar("player");
    cast(&mut g, "los.druid.nature.bear_form");
    assert_eq!(g.state.fighters["player"].form, Form::Bear);
    assert_ne!(g.bar("player"), before);
    assert_eq!(g.bar("player").len(), 10);
    close(g.state.fighters["player"].health, 500.);
    assert!(g.set_mounted("player", true).is_err());
    assert!(g.build("player", "nature", Focus::Tank).is_err());
    cast(&mut g, "los.druid.nature.bear_10");
    assert_eq!(g.bar("player"), before);
    let mana = g.state.fighters["player"].mana;
    assert!(mana <= 300.);
    cast(&mut g, "los.druid.nature.cat_form");
    assert_eq!(g.state.fighters["player"].speed_factor(g.state.tick), 1.3);
    g.damage("enemy", "player", "test", 10000., false);
    assert_eq!(g.state.fighters["player"].form, Form::Humanoid);
    assert!(
        g.cast(
            "player",
            "los.druid.nature.renewal",
            Aim {
                target: Some("player".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn build_switch_retains_per_build_bar_mana_health_and_global_cooldown() {
    let mut g = game("fire", Focus::Dps);
    let mut bar = g.bar("player");
    bar.reverse();
    g.equip("player", bar.clone()).unwrap();
    cast(&mut g, "los.mage.fire.ember_ward");
    assert!(g.build("player", "frost", Focus::Healer).is_err());
    g.step(480).unwrap();
    let mana = g.state.fighters["player"].mana;
    let hp = g.state.fighters["player"].health;
    let cd = g.state.fighters["player"].cooldowns.clone();
    g.build("player", "frost", Focus::Healer).unwrap();
    close(g.state.fighters["player"].mana, mana);
    close(g.state.fighters["player"].health, hp);
    assert_eq!(g.state.fighters["player"].cooldowns, cd);
    g.build("player", "fire", Focus::Dps).unwrap();
    assert_eq!(g.bar("player"), bar);
}
#[test]
fn taunt_overrides_hostile_ai_attention_and_dash_sweeps_a_thin_wall() {
    let mut g = game("fire", Focus::Tank);
    g.state.fighters.get_mut("enemy").unwrap().ai = true;
    g.state
        .fighters
        .get_mut("enemy")
        .unwrap()
        .threat
        .insert("ally".into(), 10000.);
    let a = g
        .catalog
        .abilities
        .iter()
        .find(|a| {
            a.specialization == "fire"
                && a.focuses == [Focus::Tank]
                && a.delivery == Delivery::Target
                && a.effects.iter().any(|e| e.kind == EffectKind::Taunt)
        })
        .unwrap()
        .clone();
    cast(&mut g, &a.id);
    let before = g.position("enemy");
    g.step(20).unwrap();
    let moved = g.position("enemy") - before;
    assert!(moved.z > 1.15 && moved.x.abs() < 0.1, "{moved:?}");
    let mut g = game("aether", Focus::Dps);
    let wall:ignis::model::Entity=serde_json::from_value(serde_json::json!({"id":"wall","name":"wall","body":"fixed","color":[0.5,0.5,0.5],"transform":{"position":[0,1,-2],"size":[4,2,0.1]}})).unwrap();
    let mut s = g.engine.scene.clone();
    s.entities.push(wall);
    g = Game::new(s, g.catalog, g.state.fighters).unwrap();
    g.step(2).unwrap();
    cast(&mut g, "los.mage.aether.blink");
    assert!(
        g.position("player").z > -1.7 && g.position("player").z < -1.,
        "{:?}",
        g.position("player")
    );
}
#[test]
fn save_load_preserves_cast_flight_statuses_mana_and_physical_clock() {
    let mut g = game("fire", Focus::Dps);
    cast(&mut g, "los.mage.fire.ember_ward");
    cast(&mut g, "los.mage.fire.fireball");
    g.step(120).unwrap();
    let bytes = g.save().unwrap();
    let mut restored = Game::load(&bytes).unwrap();
    assert_eq!(g.state, restored.state);
    g.step(150).unwrap();
    restored.step(150).unwrap();
    assert_eq!(g.state, restored.state);
    for id in g.state.fighters.keys() {
        assert!((g.position(id) - restored.position(id)).length() < 0.0001);
    }
    let mut bad: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    bad["state"]["tick"] = serde_json::json!(9999);
    assert!(Game::load(&serde_json::to_vec(&bad).unwrap()).is_err());
}
#[test]
fn native_slow_factor_is_not_only_diagnostic() {
    let mut g = game("frost", Focus::Dps);
    let a = "los.mage.frost.ice_bolt";
    cast(&mut g, a);
    g.step(80).unwrap();
    assert!(g.state.fighters["enemy"].speed_factor(g.state.tick) < 1.);
    let p = g.position("enemy");
    g.movement("enemy", [1., 0.]).unwrap();
    g.step(30).unwrap();
    let moved = g.position("enemy") - p;
    assert!((moved.x - 2.1).abs() < 0.03, "{moved:?}");
}
#[test]
fn target_is_revalidated_at_commit_and_cannot_fund_its_own_cost() {
    let mut g = game("nature", Focus::Healer);
    let a = "los.druid.nature.healer_01";
    cast(&mut g, a);
    g.movement("ally", [1., 0.]).unwrap();
    g.state.fighters.get_mut("ally").unwrap().spawn = [99., 0.81, 0.];
    // Move the physical target before commit; metadata alone must not affect range.
    g.engine
        .simulation
        .displace_character("ally", [25., 0., 0.])
        .unwrap();
    g.step(27).unwrap();
    close(g.state.fighters["player"].mana, 300.);
    assert!(g.state.fighters["player"].cooldowns.is_empty());
    g.state.fighters.get_mut("player").unwrap().mana = 0.;
    assert!(
        g.cast(
            "player",
            "los.druid.nature.healer_11",
            Aim {
                target: Some("player".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn animal_self_heal_is_periodic_and_survives_movement() {
    let mut g = game("nature", Focus::Dps);
    cast(&mut g, "los.druid.nature.bear_form");
    cast(&mut g, "los.druid.nature.bear_09");
    close(g.state.fighters["player"].health, 514.);
    g.movement("player", [1., 0.]).unwrap();
    g.step(240).unwrap();
    close(g.state.fighters["player"].health, 570.);
    assert!(g.state.pulses.is_empty());
}
#[test]
fn malformed_aim_and_invalid_build_never_panic_or_replace_the_game() {
    let mut g = game("fire", Focus::Dps);
    let before = g.state.clone();
    assert!(
        g.cast(
            "player",
            "los.mage.fire.fire_circle",
            Aim {
                target: Some("missing".into()),
                point: Some([0., 0., -4.]),
                direction: None
            }
        )
        .is_err()
    );
    assert_eq!(g.state, before);
    assert!(lab::game("mage", "nature", Focus::Dps).is_err());
}
#[test]
fn area_lifetime_is_independent_from_the_actor_used_to_choose_its_point() {
    let mut g = game("fire", Focus::Dps);
    cast(&mut g, "los.mage.fire.fire_circle");
    g.step(36).unwrap();
    g.damage("player", "enemy", "test", 10000., false);
    g.step(420).unwrap();
    close(g.state.fighters["ally"].health, 612.);
}
