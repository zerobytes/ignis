use ignis::{
    abilities::{Effect, Outcome, Recipient},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    periodic::{Outcome as End, Reapply, Schedule},
};
use serde_json::json;
fn dispel(required: bool, max: u8) -> Effect {
    Effect::Dispel {
        recipient: Recipient::Target,
        category: "poison".into(),
        max_instances: max,
        require_match: required,
    }
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
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
 {"id":"poison","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":5}],"periodic":{"interval_ticks":60,"pulses":5,"reapply":"independent"},"periodic_category":"poison"},
 {"id":"regen","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"heal","recipient":"target","amount":5}],"periodic":{"interval_ticks":60,"pulses":5,"reapply":"independent"},"periodic_category":"regeneration"},
 {"id":"antidote","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"dispel","recipient":"target","category":"poison","max_instances":2,"require_match":true}]},
 {"id":"self_antidote","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"dispel","recipient":"target","category":"poison","max_instances":2,"require_match":true}]},
 {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
 {"id":"bolt","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"dispel","recipient":"target","category":"poison","max_instances":2,"require_match":true}],"delivery":{"kind":"projectile","speed":10,"half_extent":0.1,"max_distance":20}}
 ]})).unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"Target","body":"npc","color":[0.6,0.2,0.8],"transform":{"position":[3,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["poison","regen","antidote","self_antidote","kill","bolt"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
        e.footsteps = None;
    }
    entity(&mut s, "player").transform.position = [-3., 0.8, 0.];
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
fn hp(e: &Engine, id: &str) -> i64 {
    e.simulation.vitality.state.actors[slot(e, id)]
        .unwrap()
        .units
}
fn energy(e: &Engine, id: &str) -> i64 {
    e.simulation.abilities.state.balances[slot(e, id)][0]
}
fn cast(e: &mut Engine, actor: &str, n: u8, target: Option<&str>) {
    e.execute(
        Command::ActivateAbility {
            actor: actor.into(),
            slot: n,
            target: target.map(Into::into),
        }
        .into(),
    )
    .unwrap();
}
fn apply(e: &mut Engine, n: u8) {
    cast(e, "player", n, Some("target"));
    e.advance(1).unwrap();
}
fn verify(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn count(e: &Engine) -> usize {
    e.simulation.abilities.state.periodic.active.len()
}
#[test]
fn removes_only_matching_category_target_and_oldest_slots_with_provenance() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 2);
    apply(&mut e, 1);
    apply(&mut e, 1);
    cast(&mut e, "player", 1, Some("player"));
    e.advance(1).unwrap();
    let before = e.simulation.abilities.state.periodic.active.clone();
    apply(&mut e, 3);
    assert_eq!(count(&e), 3);
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .periodic
            .active
            .iter()
            .map(|a| a.cast.id)
            .collect::<Vec<_>>(),
        vec![before[1].cast.id, before[3].cast.id, before[4].cast.id]
    );
    let removed = e
        .simulation
        .abilities
        .state
        .periodic
        .receipts
        .iter()
        .filter(|r| r.outcome == End::Dispelled)
        .collect::<Vec<_>>();
    assert_eq!(removed.len(), 2);
    assert_eq!(removed[0].execution_id, before[0].cast.id);
    assert_eq!(removed[1].execution_id, before[2].cast.id);
    assert_eq!(
        removed[0].removed_by.unwrap().actor,
        slot(&e, "player") as u16
    );
    verify(&e);
}
#[test]
fn missing_required_effect_fails_without_cost_but_optional_consumes_cost() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 3);
    assert_eq!(energy(&e, "player"), 1000);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::NoMatchingEffect
    );
    let mut s = scene();
    s.ability_catalog.abilities[2].effects = vec![dispel(false, 2)];
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 3);
    assert_eq!(energy(&e, "player"), 995);
    let effect = e
        .simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .effects[0]
        .unwrap();
    assert_eq!(effect.dispel.unwrap().removed, 0);
    verify(&e);
}
#[test]
fn multiple_operators_never_select_same_instance_and_failure_rolls_back() {
    let mut s = scene();
    s.ability_catalog.abilities[2].effects = vec![dispel(true, 1), dispel(true, 1)];
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(count(&e), 1);
    assert_eq!(energy(&e, "player"), 1000);
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(count(&e), 0);
    assert_eq!(energy(&e, "player"), 995);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .effects[..2]
            .iter()
            .all(|a| a.unwrap().dispel.unwrap().removed == 1)
    );
    verify(&e);
}
#[test]
fn later_effect_failure_preserves_removals_health_and_cost() {
    let mut s = scene();
    entity(&mut s, "target").abilities = None;
    s.ability_catalog.abilities[2].effects.extend([
        Effect::Damage {
            recipient: Recipient::Target,
            amount: 20,
        },
        Effect::Resource {
            recipient: Recipient::Target,
            resource: "energy".into(),
            delta: 5,
        },
    ]);
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(count(&e), 1);
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 1000);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::MissingResource
    );
    verify(&e);
}
#[test]
fn expiry_during_cast_revalidates_without_cost_or_cooldown() {
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic = Some(Schedule {
        interval_ticks: 2,
        pulses: 1,
        reapply: Reapply::Independent,
    });
    s.ability_catalog.abilities[2].cast_ticks = 10;
    s.ability_catalog.abilities[2].cooldown_ticks = 100;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 3);
    let prefix = e.simulation.abilities.state.clone();
    e.advance(10).unwrap();
    assert_eq!(count(&e), 0);
    assert_eq!(hp(&e, "target"), 95);
    assert_eq!(energy(&e, "player"), 1000);
    assert_eq!(
        e.simulation.abilities.state.ready_at[slot(&e, "player")][2],
        0
    );
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::NoMatchingEffect
    );
    verify(&e);
    assert_eq!(
        Engine::replay_at(&e.replay(), Some(2))
            .unwrap()
            .simulation
            .abilities
            .state,
        prefix
    );
}
#[test]
fn cancel_at_deadline_does_not_dispel_or_charge() {
    let mut s = scene();
    s.ability_catalog.abilities[2].cast_ticks = 10;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 3);
    let id = e.simulation.abilities.state.casts[slot(&e, "player")]
        .unwrap()
        .id;
    e.advance(9).unwrap();
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: id,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(count(&e), 1);
    assert_eq!(energy(&e, "player"), 1000);
    verify(&e);
}
#[test]
fn due_pulse_happens_before_dispel_in_same_tick() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    e.advance(59).unwrap();
    apply(&mut e, 3);
    assert_eq!(hp(&e, "target"), 95);
    assert_eq!(count(&e), 0);
    e.advance(100).unwrap();
    assert_eq!(hp(&e, "target"), 95);
    verify(&e);
}
#[test]
fn refresh_keeps_selection_order_even_after_newer_application() {
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .periodic
        .as_mut()
        .unwrap()
        .reapply = Reapply::Refresh;
    s.ability_catalog.abilities[2].effects = vec![dispel(true, 1)];
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "target", 1, Some("target"));
    e.advance(1).unwrap();
    let other = e.simulation.abilities.state.periodic.active[1].cast.id;
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(count(&e), 1);
    assert_eq!(
        e.simulation.abilities.state.periodic.active[0].cast.id,
        other
    );
    verify(&e);
}
#[test]
fn unclassified_periodic_is_not_removed() {
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic_category = None;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(count(&e), 1);
    assert_eq!(energy(&e, "player"), 1000);
    verify(&e);
}
#[test]
fn projectile_resolves_actual_interceptor_and_leaves_intended_target() {
    let mut s = scene();
    let mut other = entity(&mut s, "target").clone();
    other.id = "interceptor".into();
    other.transform.position = [0., 0.8, 0.];
    other.blocks_vision = Some(false);
    s.entities.push(other);
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "player", 1, Some("interceptor"));
    e.advance(1).unwrap();
    apply(&mut e, 6);
    assert_eq!(count(&e), 2);
    e.advance(40).unwrap();
    assert_eq!(count(&e), 1);
    assert_eq!(
        e.simulation.abilities.state.periodic.active[0].cast.target,
        Some(slot(&e, "target") as u16)
    );
    assert_eq!(energy(&e, "player"), 995);
    verify(&e);
}
#[test]
fn effect_expires_in_flight_without_launch_refund() {
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic = Some(Schedule {
        interval_ticks: 2,
        pulses: 1,
        reapply: Reapply::Independent,
    });
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 6);
    e.advance(50).unwrap();
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .projectiles
            .receipts
            .back()
            .unwrap()
            .outcome,
        ignis::projectiles::Outcome::EffectFailed
    );
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::NoMatchingEffect
    );
    verify(&e);
}
#[test]
fn spatial_projectile_does_not_require_prior_target_or_effect() {
    let mut s = scene();
    s.ability_catalog.abilities[5].selection = ignis::abilities::Selection::Spatial;
    s.ability_catalog.abilities[5].range = 0.;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 6,
            aim: ignis::abilities::Aim::Direction {
                vector: [1., 0., 0.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(50).unwrap();
    assert_eq!(count(&e), 0);
    assert_eq!(energy(&e, "player"), 995);
    verify(&e);
}
#[test]
fn mask_covers_all_64_slots_and_reuses_capacity() {
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .periodic
        .as_mut()
        .unwrap()
        .interval_ticks = 3600;
    s.ability_catalog.abilities[2].effects = vec![dispel(true, 64)];
    let mut e = Engine::new(s).unwrap();
    for _ in 0..64 {
        apply(&mut e, 1);
    }
    assert_eq!(count(&e), 64);
    apply(&mut e, 3);
    assert_eq!(count(&e), 0);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .periodic
            .receipts
            .iter()
            .filter(|r| r.outcome == End::Dispelled)
            .count(),
        64
    );
    apply(&mut e, 1);
    assert_eq!(count(&e), 1);
    verify(&e);
}
#[test]
fn hidden_target_effects_do_not_change_npc_attempt_result() {
    for poisoned in [false, true] {
        let mut s = scene();
        s.entities.push(serde_json::from_value(json!({"id":"wall","name":"Wall","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[0,2,0],"size":[0.3,4,8]}})).unwrap());
        let mut e = Engine::new(s).unwrap();
        if poisoned {
            cast(&mut e, "player", 1, Some("player"));
        }
        e.advance(1).unwrap();
        cast(&mut e, "target", 3, Some("player"));
        e.advance(1).unwrap();
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
        assert_eq!(energy(&e, "target"), 1000);
        verify(&e);
    }
}
#[test]
fn npc_self_antidote_waits_for_effect_then_uses_shared_executor() {
    let mut s = scene();
    s.ability_catalog.abilities[3].cast_ticks = 5;
    entity(&mut s,"target").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":4,"retry_ticks":10}},{"priority":0,"when":"always","action":{"kind":"wait"}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(15).unwrap();
    assert_eq!(energy(&e, "target"), 1000);
    assert!(e.simulation.abilities.state.casts[slot(&e, "target")].is_none());
    apply(&mut e, 1);
    e.advance(20).unwrap();
    assert_eq!(count(&e), 0);
    assert_eq!(energy(&e, "target"), 995);
    e.advance(50).unwrap();
    assert_eq!(energy(&e, "target"), 995);
    verify(&e);
}
#[test]
fn validation_rejects_invalid_categories_versions_budgets_and_recursion() {
    for category in ["", "has space", "ç"] {
        let mut s = scene();
        s.ability_catalog.abilities[0].periodic_category = Some(category.into());
        assert!(s.validate().is_err());
    }
    for max in [0, 65] {
        let mut s = scene();
        s.ability_catalog.abilities[2].effects = vec![dispel(true, max)];
        assert!(s.validate().is_err());
    }
    let mut s = scene();
    s.version = 33;
    assert!(s.validate().is_err());
    let mut s = scene();
    s.ability_catalog.abilities[2].periodic_category = Some("poison".into());
    assert!(s.validate().is_err());
    let mut s = scene();
    s.ability_catalog.abilities[0].effects = vec![dispel(true, 1)];
    assert!(s.validate().is_err());
    let mut s = scene();
    let base = s.ability_catalog.abilities[2].clone();
    for i in 0..9 {
        let mut d = base.clone();
        d.id = format!("many{i}");
        d.effects = (0..8)
            .map(|j| Effect::Dispel {
                recipient: Recipient::Target,
                category: format!("class{i}_{j}"),
                max_instances: 1,
                require_match: false,
            })
            .collect();
        s.ability_catalog.abilities.push(d);
    }
    assert!(s.validate().is_err());
}
#[test]
fn direct_source_recipient_does_not_remove_target_effects() {
    let mut s = scene();
    s.ability_catalog.abilities[2].effects = vec![Effect::Dispel {
        recipient: Recipient::Source,
        category: "poison".into(),
        max_instances: 64,
        require_match: true,
    }];
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "player", 1, Some("player"));
    e.advance(1).unwrap();
    apply(&mut e, 3);
    assert_eq!(count(&e), 1);
    assert_eq!(
        e.simulation.abilities.state.periodic.active[0].cast.target,
        Some(slot(&e, "target") as u16)
    );
    verify(&e);
}

#[test]
fn concurrent_casts_revalidate_and_only_first_success_pays() {
    let mut s = scene();
    s.ability_catalog.abilities[2].cast_ticks = 5;
    s.ability_catalog.abilities[3].cast_ticks = 5;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "player", 3, Some("target"));
    cast(&mut e, "target", 4, None);
    e.advance(6).unwrap();
    assert_eq!(count(&e), 0);
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(energy(&e, "target"), 1000);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::NoMatchingEffect
    );
    verify(&e);
}
#[test]
fn source_death_after_projectile_launch_does_not_prevent_dispel() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 6);
    cast(&mut e, "target", 5, Some("player"));
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "player"), 0);
    e.advance(50).unwrap();
    assert_eq!(count(&e), 0);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .projectiles
            .receipts
            .back()
            .unwrap()
            .outcome,
        ignis::projectiles::Outcome::Hit
    );
    verify(&e);
}
#[test]
fn dispel_and_heal_are_one_successful_transaction_without_reversing_prior_damage() {
    let mut s = scene();
    s.ability_catalog.abilities[2].effects.push(Effect::Heal {
        recipient: Recipient::Target,
        amount: 3,
    });
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    e.advance(60).unwrap();
    assert_eq!(hp(&e, "target"), 95);
    apply(&mut e, 3);
    assert_eq!(count(&e), 0);
    assert_eq!(hp(&e, "target"), 98);
    assert_eq!(energy(&e, "player"), 995);
    verify(&e);
}

fn protected_scene() -> Scene {
    let mut s = scene();
    let mut protected = s.ability_catalog.abilities[0].clone();
    protected.id = "protected_poison".into();
    protected.dispellable = false;
    s.ability_catalog.abilities.push(protected);
    for id in ["player", "target"] {
        entity(&mut s, id)
            .abilities
            .as_mut()
            .unwrap()
            .grants
            .push("protected_poison".into());
    }
    s
}

#[test]
fn protected_instances_do_not_consume_limit_or_change_category_and_replay() {
    let mut e = Engine::new(protected_scene()).unwrap();
    for n in [7, 1, 7, 1, 1] {
        apply(&mut e, n);
    }
    let before = e.simulation.abilities.state.periodic.active.clone();
    apply(&mut e, 3);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .periodic
            .active
            .iter()
            .map(|a| a.cast.id)
            .collect::<Vec<_>>(),
        vec![before[0].cast.id, before[2].cast.id, before[4].cast.id]
    );
    let inspected =
        e.simulation
            .abilities
            .inspect(&e.scene, &e.simulation.actors, e.simulation.tick);
    let active = inspected["periodic"]["active"].as_array().unwrap();
    assert!(active.iter().all(|a| a["category"] == "poison"));
    assert_eq!(
        active
            .iter()
            .map(|a| a["dispellable"].as_bool().unwrap())
            .collect::<Vec<_>>(),
        vec![false, false, true]
    );
    assert_eq!(energy(&e, "player"), 995);
    verify(&e);
}

#[test]
fn only_protected_is_no_match_optional_pays_and_later_operator_rolls_back() {
    for required in [true, false] {
        let mut s = protected_scene();
        s.ability_catalog.abilities[2].effects = vec![dispel(required, 64)];
        let mut e = Engine::new(s).unwrap();
        apply(&mut e, 7);
        apply(&mut e, 3);
        assert_eq!(count(&e), 1);
        assert_eq!(energy(&e, "player"), if required { 1000 } else { 995 });
        assert_eq!(
            e.simulation
                .abilities
                .state
                .receipts
                .back()
                .unwrap()
                .outcome,
            if required {
                Outcome::NoMatchingEffect
            } else {
                Outcome::Completed
            }
        );
        verify(&e);
    }
    let mut s = protected_scene();
    s.ability_catalog.abilities[2].effects = vec![dispel(true, 1), dispel(true, 1)];
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 7);
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(count(&e), 2);
    assert_eq!(energy(&e, "player"), 1000);
    assert!(
        !e.simulation
            .abilities
            .state
            .periodic
            .receipts
            .iter()
            .any(|r| r.outcome == End::Dispelled)
    );
    verify(&e);
}

#[test]
fn protected_survivor_does_not_satisfy_cast_commit_or_projectile_impact() {
    for projectile in [false, true] {
        let mut s = protected_scene();
        s.ability_catalog.abilities[0].periodic = Some(Schedule {
            interval_ticks: 10,
            pulses: 1,
            reapply: Reapply::Independent,
        });
        s.ability_catalog.abilities[2].cast_ticks = 20;
        let mut e = Engine::new(s).unwrap();
        apply(&mut e, 7);
        apply(&mut e, 1);
        apply(&mut e, if projectile { 6 } else { 3 });
        e.advance(50).unwrap();
        assert_eq!(count(&e), 1);
        assert_eq!(
            e.simulation.abilities.state.periodic.active[0].definition,
            6
        );
        assert_eq!(energy(&e, "player"), if projectile { 995 } else { 1000 });
        assert_eq!(
            e.simulation
                .abilities
                .state
                .receipts
                .back()
                .unwrap()
                .outcome,
            Outcome::NoMatchingEffect
        );
        verify(&e);
    }
}

#[test]
fn npc_ignores_protected_only_but_cleans_normal_effect_without_removing_protected() {
    let mut s = protected_scene();
    entity(&mut s,"target").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":4,"retry_ticks":10}},{"priority":0,"when":"always","action":{"kind":"wait"}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 7);
    e.advance(25).unwrap();
    assert_eq!(count(&e), 1);
    assert_eq!(energy(&e, "target"), 1000);
    assert!(e.simulation.abilities.state.casts[slot(&e, "target")].is_none());
    apply(&mut e, 1);
    e.advance(25).unwrap();
    assert_eq!(count(&e), 1);
    assert_eq!(energy(&e, "target"), 995);
    assert_eq!(
        e.simulation.abilities.state.periodic.active[0].definition,
        6
    );
    verify(&e);
}

#[test]
fn protection_preserves_refresh_pulses_expiry_and_death_cleanup() {
    for kill in [false, true] {
        let mut s = protected_scene();
        s.ability_catalog.abilities[6].periodic = Some(Schedule {
            interval_ticks: 10,
            pulses: 2,
            reapply: Reapply::Refresh,
        });
        let mut e = Engine::new(s).unwrap();
        apply(&mut e, 7);
        e.advance(5).unwrap();
        apply(&mut e, 7);
        assert_eq!(count(&e), 1);
        assert!(
            e.simulation
                .abilities
                .state
                .periodic
                .receipts
                .iter()
                .any(|r| r.outcome == End::Refreshed)
        );
        if kill {
            apply(&mut e, 5);
        } else {
            e.advance(20).unwrap();
        }
        assert_eq!(count(&e), 0);
        assert_eq!(hp(&e, "target"), if kill { 0 } else { 90 });
        assert_eq!(
            e.simulation
                .abilities
                .state
                .periodic
                .receipts
                .back()
                .unwrap()
                .outcome,
            if kill { End::TargetDead } else { End::Finished }
        );
        verify(&e);
    }
}

#[test]
fn dispellable_default_legacy_schema_roundtrip_and_validation() {
    let mut s = scene();
    s.version = 34;
    assert!(s.ability_catalog.abilities.iter().all(|d| d.dispellable));
    assert!(s.validate().is_ok());
    let legacy = serde_json::to_value(&s).unwrap();
    assert!(
        legacy["ability_catalog"]["abilities"][0]
            .get("dispellable")
            .is_none()
    );
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(count(&e), 0);
    let s = protected_scene();
    let data = serde_json::to_value(&s).unwrap();
    assert_eq!(
        data["ability_catalog"]["abilities"][6]["dispellable"],
        false
    );
    assert_eq!(serde_json::from_value::<Scene>(data).unwrap(), s);
    let mut invalid = s.clone();
    invalid.version = 34;
    assert!(invalid.validate().is_err());
    let mut invalid = s.clone();
    invalid.ability_catalog.abilities[2].dispellable = false;
    assert!(invalid.validate().is_err());
    let mut no_category = s;
    no_category.ability_catalog.abilities[6].periodic_category = None;
    assert!(no_category.validate().is_ok());
    let mut bad = legacy;
    bad["ability_catalog"]["abilities"][0]["dispellable"] = json!("false");
    assert!(serde_json::from_value::<Scene>(bad).is_err());
}
