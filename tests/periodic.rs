use ignis::{
    abilities::Outcome,
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    periodic::{Reapply, Schedule},
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
        {"id":"poison","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":10}],"periodic":{"interval_ticks":10,"pulses":3,"reapply":"refresh"}},
        {"id":"regen","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"heal","recipient":"target","amount":10}],"periodic":{"interval_ticks":10,"pulses":3,"reapply":"independent"}},
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
        {"id":"bolt","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":10}],"periodic":{"interval_ticks":10,"pulses":3,"reapply":"refresh"},"delivery":{"kind":"projectile","speed":10,"half_extent":0.1,"max_distance":20}}
    ]})).unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"Target","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[3,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["poison","regen","kill","bolt"]})).unwrap());
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
#[test]
fn delayed_pulses_exact_end_and_single_charge_with_prefix_replay() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 995);
    let prefix = e.simulation.abilities.state.clone();
    e.advance(9).unwrap();
    assert_eq!(hp(&e, "target"), 100);
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "target"), 90);
    e.advance(20).unwrap();
    assert_eq!(hp(&e, "target"), 70);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    let receipts = &e.simulation.abilities.state.receipts;
    assert_eq!(
        receipts
            .iter()
            .filter(|r| r.outcome == Outcome::PeriodicPulse)
            .map(|r| r.tick)
            .collect::<Vec<_>>(),
        vec![10, 20, 30]
    );
    assert_eq!(energy(&e, "player"), 995);
    verify(&e);
    assert_eq!(
        Engine::replay_at(&e.replay(), Some(1))
            .unwrap()
            .simulation
            .abilities
            .state,
        prefix
    );
}
#[test]
fn refresh_restarts_delay_replaces_execution_without_stacking() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    let first = e.simulation.abilities.state.periodic.active[0].cast.id;
    e.advance(4).unwrap();
    apply(&mut e, 1);
    let state = &e.simulation.abilities.state.periodic;
    assert_eq!(state.active.len(), 1);
    assert_ne!(state.active[0].cast.id, first);
    assert_eq!(state.active[0].next_tick, 15);
    assert!(
        state
            .receipts
            .iter()
            .any(|r| r.execution_id == first && r.outcome == ignis::periodic::Outcome::Refreshed)
    );
    e.advance(9).unwrap();
    assert_eq!(hp(&e, "target"), 100);
    e.advance(21).unwrap();
    assert_eq!(hp(&e, "target"), 70);
    assert_eq!(energy(&e, "player"), 990);
    verify(&e);
}
#[test]
fn independent_reapplications_each_keep_their_schedule() {
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .periodic
        .as_mut()
        .unwrap()
        .reapply = Reapply::Independent;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 1);
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 2);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "target"), 40);
    assert_eq!(energy(&e, "player"), 990);
    verify(&e);
}
#[test]
fn refresh_identity_includes_source_and_target() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "target", 1, Some("player"));
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 2);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "player"), 70);
    assert_eq!(hp(&e, "target"), 70);
    verify(&e);
}
#[test]
fn source_death_does_not_remove_applied_effect_or_grant_vision() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "target", 3, Some("player"));
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "player"), 0);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "target"), 70);
    verify(&e);
}
#[test]
fn target_death_removes_all_schedules_and_regeneration_cannot_revive() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "target", 2, None);
    e.advance(1).unwrap();
    apply(&mut e, 1);
    apply(&mut e, 3);
    assert_eq!(hp(&e, "target"), 0);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    e.advance(50).unwrap();
    assert_eq!(hp(&e, "target"), 0);
    assert_eq!(e.simulation.vitality.state.deaths.len(), 1);
    verify(&e);
}
#[test]
fn pulse_before_cast_deadline_cancels_victim_without_charging() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects[0] =
        serde_json::from_value(json!({"kind":"damage","recipient":"target","amount":100})).unwrap();
    s.ability_catalog.abilities[1].cast_ticks = 9;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "target", 2, None);
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "target"), 0);
    assert_eq!(energy(&e, "target"), 1000);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::CancelledByDeath)
    );
    verify(&e);
}
#[test]
fn pulse_transaction_damage_then_heal_commits_death_only_at_end() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects=serde_json::from_value(json!([{"kind":"damage","recipient":"target","amount":100},{"kind":"heal","recipient":"target","amount":25}])).unwrap();
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "target"), 25);
    assert!(e.simulation.vitality.state.deaths.is_empty());
    verify(&e);
}
#[test]
fn heal_and_resource_pulses_saturate_without_recharging() {
    let mut s = scene();
    entity(&mut s, "player").vitality.as_mut().unwrap().initial = 85;
    s.ability_catalog.abilities[1].effects.push(
        serde_json::from_value(
            json!({"kind":"resource","recipient":"target","resource":"energy","delta":3}),
        )
        .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 2, None);
    e.advance(31).unwrap();
    assert_eq!(hp(&e, "player"), 100);
    assert_eq!(energy(&e, "player"), 1000);
    verify(&e);
}
#[test]
fn missing_component_rejects_entire_delivery_without_cost_or_schedule() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects.push(
        serde_json::from_value(
            json!({"kind":"resource","recipient":"target","resource":"energy","delta":3}),
        )
        .unwrap(),
    );
    entity(&mut s, "target").abilities = None;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 1000);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
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
}
#[test]
fn full_pool_rejects_direct_without_debit_but_refresh_can_replace() {
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic = Some(Schedule {
        interval_ticks: 3600,
        pulses: 2,
        reapply: Reapply::Independent,
    });
    let mut e = Engine::new(s).unwrap();
    for _ in 0..64 {
        apply(&mut e, 1);
    }
    assert_eq!(energy(&e, "player"), 680);
    apply(&mut e, 1);
    assert_eq!(energy(&e, "player"), 680);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::PeriodicBudget
    );
    verify(&e);
    // Fill with independent instances after installing one refreshable schedule.
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .periodic
        .as_mut()
        .unwrap()
        .interval_ticks = 3600;
    s.ability_catalog.abilities[1]
        .periodic
        .as_mut()
        .unwrap()
        .interval_ticks = 3600;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    for _ in 0..63 {
        cast(&mut e, "player", 2, None);
        e.advance(1).unwrap();
    }
    apply(&mut e, 1);
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 64);
    assert_eq!(energy(&e, "player"), 675);
    verify(&e);
}
#[test]
fn projectile_installs_on_contact_and_uses_actual_interceptor() {
    let mut s = scene();
    let mut other = entity(&mut s, "target").clone();
    other.id = "interceptor".into();
    // Transparent to sight, solid to the projectile: launch is authorized first.
    other.blocks_vision = Some(false);
    other.transform.position = [0., 0.8, 0.];
    s.entities.push(other);
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 4);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    for _ in 0..40 {
        e.advance(1).unwrap();
        if !e.simulation.abilities.state.periodic.active.is_empty() {
            break;
        }
    }
    let effect = e.simulation.abilities.state.periodic.active[0];
    assert_eq!(effect.cast.target, Some(slot(&e, "interceptor") as u16));
    assert_eq!(hp(&e, "interceptor"), 100);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "interceptor"), 70);
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 995);
    verify(&e);
}
#[test]
fn projectile_pool_failure_does_not_refund_launch() {
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic = Some(Schedule {
        interval_ticks: 3600,
        pulses: 2,
        reapply: Reapply::Independent,
    });
    let mut e = Engine::new(s).unwrap();
    for _ in 0..64 {
        apply(&mut e, 1);
    }
    apply(&mut e, 4);
    e.advance(50).unwrap();
    assert_eq!(energy(&e, "player"), 675);
    assert_eq!(hp(&e, "target"), 100);
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
        Outcome::PeriodicBudget
    );
    verify(&e);
}
#[test]
fn authoring_validation_version_and_unknown_fields_are_strict() {
    let base = scene();
    for (interval, pulses) in [(0, 1), (1, 0), (3601, 1), (1, 3601), (3600, 61)] {
        let mut s = base.clone();
        s.ability_catalog.abilities[0].periodic = Some(Schedule {
            interval_ticks: interval,
            pulses,
            reapply: Reapply::Refresh,
        });
        assert!(s.validate().is_err());
    }
    let mut s = base.clone();
    s.version = 32;
    assert!(s.validate().is_err());
    for effect in [
        json!({"kind":"noise","radius":4}),
        json!({"kind":"heal","recipient":"source","amount":5}),
    ] {
        let mut s = base.clone();
        s.ability_catalog.abilities[0].effects = serde_json::from_value(json!([effect])).unwrap();
        assert!(s.validate().is_err());
    }
    assert!(
        serde_json::from_value::<Schedule>(
            json!({"interval_ticks":10,"pulses":2,"reapply":"refresh","unused":1})
        )
        .is_err()
    );
}
#[test]
fn schedule_ring_is_bounded_and_cast_cancellation_does_not_dispel() {
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic = Some(Schedule {
        interval_ticks: 1,
        pulses: 1,
        reapply: Reapply::Independent,
    });
    s.ability_catalog.abilities[0].effects =
        serde_json::from_value(json!([{"kind":"heal","recipient":"target","amount":1}])).unwrap();
    s.ability_catalog.abilities[0].costs.clear();
    let mut e = Engine::new(s).unwrap();
    for _ in 0..150 {
        apply(&mut e, 1);
    }
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.periodic.receipts.len(), 256);
    assert_eq!(e.simulation.abilities.state.periodic.dropped_receipts, 44);
    verify(&e);
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    let id = e.simulation.abilities.state.periodic.active[0].cast.id;
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: id,
        }
        .into(),
    )
    .unwrap();
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "target"), 70);
    verify(&e);
}

#[test]
fn cancel_preparation_and_insufficient_energy_never_install() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 10;
    let mut e = Engine::new(s.clone()).unwrap();
    apply(&mut e, 1);
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
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    assert_eq!(energy(&e, "player"), 1000);
    verify(&e);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .initial = 4;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1);
    assert_eq!(energy(&e, "player"), 4);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
}
#[test]
fn refresh_same_tick_as_pulse_applies_due_pulse_before_replacement() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    e.advance(9).unwrap();
    apply(&mut e, 1);
    assert_eq!(hp(&e, "target"), 90);
    assert_eq!(
        e.simulation.abilities.state.periodic.active[0].next_tick,
        20
    );
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "target"), 60);
    verify(&e);
}
#[test]
fn different_sources_same_target_do_not_refresh_each_other() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    cast(&mut e, "target", 1, Some("target"));
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 2);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "target"), 40);
    verify(&e);
}
#[test]
fn npc_installed_effect_survives_loss_of_sight_without_renewing_memory() {
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .periodic
        .as_mut()
        .unwrap()
        .interval_ticks = 60;
    s.entities.push(serde_json::from_value(json!({"id":"cover","name":"Cover","body":"fixed","color":[0.2,0.5,0.5],"transform":{"position":[0,1.5,2],"size":[0.2,3,3]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "target", 1, Some("player"));
    e.advance(1).unwrap();
    e.execute(
        Command::SetInput {
            input: ignis::physics::Input {
                x: 0.,
                z: 1.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(90).unwrap();
    e.execute(
        Command::SetInput {
            input: Default::default(),
        }
        .into(),
    )
    .unwrap();
    assert!(!e.simulation.npcs["target"].observed_this_tick);
    let memory = e.simulation.npcs["target"].memory;
    e.advance(31).unwrap();
    assert_eq!(hp(&e, "player"), 80);
    assert!(!e.simulation.npcs["target"].observed_this_tick);
    if let Some(current) = e.simulation.npcs["target"].memory {
        assert_eq!(Some(current.tick), memory.map(|m| m.tick));
    }
    verify(&e);
}
#[test]
fn guide_toggle_is_presentation_only_and_reset_discards_schedules() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1);
    let before = e.simulation.abilities.state.clone();
    e.execute(
        Command::SetDebugView {
            npc_vision: None,
            npc_hearing: None,
            noise_events: None,
            npc_routes: Some(true),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.simulation.abilities.state, before);
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    assert!(e.simulation.abilities.state.periodic.receipts.is_empty());
}
