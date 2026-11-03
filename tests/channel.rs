use ignis::{
    abilities::{CastMovement, Effect, Outcome, Recipient},
    channel::{Channel, CostMode},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    physics::Input,
};
use serde_json::json;
fn scene(mode: CostMode) -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
        {"id":"channel","selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":80,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":5}]},
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
        {"id":"drain","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resource","recipient":"target","resource":"energy","delta":-100}]}
    ]})).unwrap();
    s.ability_catalog.abilities[0].channel = Some(Channel {
        interval_ticks: 10,
        pulses: 3,
        cost_mode: mode,
    });
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"Target","body":"npc","color":[0.6,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.footsteps = None;
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["channel","kill","drain"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
    }
    entity(&mut s, "player").transform.position = [0., 0.8, -4.];
    s
}
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
fn index(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap()
}
fn energy(e: &Engine, id: &str) -> i64 {
    e.simulation.abilities.state.balances[index(e, id)][0]
}
fn hp(e: &Engine, id: &str) -> i64 {
    e.simulation.vitality.state.actors[index(e, id)]
        .unwrap()
        .units
}
fn active(e: &Engine, id: &str) -> Option<ignis::abilities::Cast> {
    e.simulation.abilities.state.casts[index(e, id)]
}
fn cast(e: &mut Engine, actor: &str, slot: u8, target: Option<&str>) {
    e.execute(
        Command::ActivateAbility {
            actor: actor.into(),
            slot,
            target: target.map(Into::into),
        }
        .into(),
    )
    .unwrap();
}
fn cancel(e: &mut Engine, id: &str) {
    e.execute(
        Command::CancelAbility {
            actor: id.into(),
            execution_id: active(e, id).unwrap().id,
        }
        .into(),
    )
    .unwrap();
}
fn input(e: &mut Engine, x: f32) {
    e.execute(
        Command::SetInput {
            input: Input {
                x,
                z: 0.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
}
fn verify(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn receipts(e: &Engine, kind: Outcome) -> usize {
    e.simulation
        .abilities
        .state
        .receipts
        .iter()
        .filter(|r| r.outcome == kind)
        .count()
}

#[test]
fn upfront_channel_delays_pulses_charges_once_and_records_stages() {
    let mut s = scene(CostMode::OnStart);
    s.ability_catalog.abilities[0].cast_ticks = 5;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(5).unwrap();
    assert_eq!(energy(&e, "player"), 100);
    assert!(active(&e, "player").unwrap().channel.is_none());
    e.advance(1).unwrap();
    assert_eq!(energy(&e, "player"), 95);
    assert_eq!(hp(&e, "target"), 100);
    let ch = active(&e, "player").unwrap().channel.unwrap();
    assert_eq!(
        (ch.started, ch.next_tick, ch.finishes_at, ch.remaining),
        (5, 15, 35, 3)
    );
    e.advance(9).unwrap();
    assert_eq!(hp(&e, "target"), 100);
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "target"), 95);
    e.advance(20).unwrap();
    assert_eq!(hp(&e, "target"), 85);
    assert!(active(&e, "player").is_none());
    assert_eq!(energy(&e, "player"), 95);
    assert_eq!(receipts(&e, Outcome::ChannelStarted), 1);
    assert_eq!(receipts(&e, Outcome::ChannelPulse), 3);
    assert_eq!(receipts(&e, Outcome::Completed), 1);
    assert_eq!(
        e.simulation.abilities.state.ready_at[index(&e, "player")][0],
        85
    );
    let snapshot = e.execute(Command::InspectAbilities.into()).unwrap();
    let charged: Vec<_> = snapshot["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| !r["costs"].as_array().unwrap().is_empty())
        .collect();
    assert_eq!(charged.len(), 1);
    assert_eq!(charged[0]["outcome"], "channel_started");
    verify(&e);
}
#[test]
fn per_pulse_costs_stop_atomically_when_balance_runs_out() {
    let mut s = scene(CostMode::PerPulse);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .initial = 11;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(1).unwrap();
    assert_eq!(energy(&e, "player"), 11);
    e.advance(30).unwrap();
    assert_eq!(energy(&e, "player"), 1);
    assert_eq!(hp(&e, "target"), 90);
    assert!(active(&e, "player").is_none());
    assert_eq!(receipts(&e, Outcome::InsufficientResource), 1);
    assert_eq!(receipts(&e, Outcome::ChannelPulse), 2);
    let snapshot = e.execute(Command::InspectAbilities.into()).unwrap();
    let charged: Vec<_> = snapshot["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| !r["costs"].as_array().unwrap().is_empty())
        .collect();
    assert_eq!(charged.len(), 2);
    assert!(charged.iter().all(|r| r["outcome"] == "channel_pulse"));
    verify(&e);
}
#[test]
fn cancellation_before_start_is_free_and_deadline_cancels_future_pulse_only() {
    for before in [false, true] {
        for mode in [CostMode::OnStart, CostMode::PerPulse] {
            let mut s = scene(mode);
            s.ability_catalog.abilities[0].cast_ticks = 5;
            let mut e = Engine::new(s).unwrap();
            cast(&mut e, "player", 1, Some("target"));
            e.advance(if before { 5 } else { 25 }).unwrap();
            cancel(&mut e, "player");
            e.advance(1).unwrap();
            assert_eq!(hp(&e, "target"), if before { 100 } else { 95 });
            assert_eq!(energy(&e, "player"), if before { 100 } else { 95 });
            assert_eq!(
                e.simulation.abilities.state.ready_at[index(&e, "player")][0],
                if before { 0 } else { 85 }
            );
            assert!(active(&e, "player").is_none());
            verify(&e);
        }
    }
}
#[test]
fn moving_interrupts_channel_and_keeps_previous_payment() {
    let mut s = scene(CostMode::OnStart);
    s.ability_catalog.abilities[0].movement = CastMovement::Interrupt {
        speed_tolerance: 0.2,
    };
    let mut e = Engine::new(s).unwrap();
    e.advance(60).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(11).unwrap();
    assert_eq!(hp(&e, "target"), 95);
    input(&mut e, 1.);
    e.advance(1).unwrap();
    assert!(active(&e, "player").is_none());
    assert_eq!(receipts(&e, Outcome::InterruptedByMovement), 1);
    assert_eq!(energy(&e, "player"), 95);
    verify(&e);
}
#[test]
fn movement_lock_lasts_through_channel_and_releases_on_cancel() {
    let mut s = scene(CostMode::OnStart);
    s.ability_catalog.abilities[0].movement = CastMovement::Lock {};
    let mut e = Engine::new(s).unwrap();
    e.advance(60).unwrap();
    let start = e.simulation.position("player").unwrap();
    cast(&mut e, "player", 1, Some("target"));
    input(&mut e, 1.);
    e.advance(21).unwrap();
    assert!((e.simulation.position("player").unwrap()[0] - start[0]).abs() < 0.01);
    assert!(active(&e, "player").is_some());
    cancel(&mut e, "player");
    e.advance(20).unwrap();
    assert!(e.simulation.position("player").unwrap()[0] > start[0] + 0.1);
    verify(&e);
}
#[test]
fn free_channel_moves_and_range_loss_interrupts_between_pulses() {
    let mut s = scene(CostMode::OnStart);
    s.ability_catalog.abilities[0].range = 4.2;
    s.ability_catalog.abilities[0]
        .channel
        .as_mut()
        .unwrap()
        .interval_ticks = 120;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(1).unwrap();
    input(&mut e, 1.);
    e.advance(60).unwrap();
    assert!(e.simulation.position("player").unwrap()[0] > 1.);
    assert!(active(&e, "player").is_none());
    assert_eq!(receipts(&e, Outcome::OutOfRange), 1);
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 95);
    verify(&e);
}
#[test]
fn npc_loses_actual_sight_and_stops_before_next_pulse() {
    let mut s = scene(CostMode::OnStart);
    s.ability_catalog.abilities[0]
        .channel
        .as_mut()
        .unwrap()
        .interval_ticks = 120;
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"Wall","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[2,1.5,-2],"size":[0.4,3,2]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    cast(&mut e, "target", 1, Some("player"));
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_some());
    input(&mut e, 1.);
    e.advance(80).unwrap();
    assert!(active(&e, "target").is_none());
    assert!(receipts(&e, Outcome::TargetUnavailable) + receipts(&e, Outcome::Occluded) > 0);
    assert_eq!(hp(&e, "player"), 100);
    assert_eq!(energy(&e, "target"), 95);
    verify(&e);
}
#[test]
fn death_cancels_source_channel_and_target_death_ends_remaining_pulses() {
    for source in [false, true] {
        let mut e = Engine::new(scene(CostMode::OnStart)).unwrap();
        cast(&mut e, "player", 1, Some("target"));
        e.advance(11).unwrap();
        if source {
            cast(&mut e, "target", 2, Some("player"));
        } else {
            cast(&mut e, "target", 2, Some("target"));
        }
        e.advance(20).unwrap();
        assert!(active(&e, "player").is_none());
        assert_eq!(energy(&e, "player"), 95);
        assert_eq!(receipts(&e, Outcome::ChannelPulse), 1);
        assert_eq!(hp(&e, if source { "player" } else { "target" }), 0);
        verify(&e);
    }
}
#[test]
fn self_lethal_pulse_does_not_restore_cancelled_cast() {
    let mut s = scene(CostMode::OnStart);
    entity(&mut s, "player").vitality.as_mut().unwrap().initial = 5;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, Some("player"));
    e.advance(11).unwrap();
    assert_eq!(hp(&e, "player"), 0);
    assert!(active(&e, "player").is_none());
    assert_eq!(receipts(&e, Outcome::CancelledByDeath), 1);
    verify(&e);
}
#[test]
fn npc_prepaid_channel_continues_with_zero_energy_and_policy() {
    let mut s = scene(CostMode::OnStart);
    s.ability_catalog.abilities[0].selection = ignis::abilities::Selection::SelfActor;
    s.ability_catalog.abilities[0].range = 0.;
    s.ability_catalog.abilities[0].effects = vec![Effect::Heal {
        recipient: Recipient::Target,
        amount: 5,
    }];
    let n = entity(&mut s, "target");
    n.vitality.as_mut().unwrap().initial = 50;
    n.abilities.as_mut().unwrap().resources[0].initial = 5;
    n.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":10,"policy":{"vitality_below":{"subject":"source","basis_points":10000}}}},{"priority":0,"when":"always","action":{"kind":"wait"}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(31).unwrap();
    assert_eq!(energy(&e, "target"), 0);
    assert_eq!(hp(&e, "target"), 65);
    assert_eq!(receipts(&e, Outcome::ChannelPulse), 3);
    verify(&e);
}
#[test]
fn payment_is_not_financed_by_same_pulse_and_failed_late_effect_rolls_back() {
    let mut s = scene(CostMode::PerPulse);
    s.ability_catalog.abilities[0]
        .effects
        .push(Effect::Resource {
            recipient: Recipient::Source,
            resource: "energy".into(),
            delta: 5,
        });
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(1).unwrap();
    cast(&mut e, "target", 3, Some("player"));
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 0);
    assert_eq!(receipts(&e, Outcome::InsufficientResource), 1);
    verify(&e);
    let mut s = scene(CostMode::PerPulse);
    s.ability_catalog.abilities[0].effects.push(Effect::Dispel {
        recipient: Recipient::Target,
        category: "poison".into(),
        max_instances: 1,
        require_match: true,
    });
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(31).unwrap();
    assert_eq!(hp(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 100);
    assert_eq!(receipts(&e, Outcome::NoMatchingEffect), 1);
    verify(&e);
}
#[test]
fn channel_is_busy_even_after_cooldown_and_cancel_does_not_cancel_next_execution() {
    let mut s = scene(CostMode::OnStart);
    s.ability_catalog.abilities[0].cooldown_ticks = 0;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(1).unwrap();
    let old = active(&e, "player").unwrap().id;
    cast(&mut e, "player", 2, Some("target"));
    e.advance(1).unwrap();
    assert_eq!(receipts(&e, Outcome::Busy), 1);
    cancel(&mut e, "player");
    e.advance(1).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(1).unwrap();
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: old,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert!(active(&e, "player").is_some());
    assert_eq!(receipts(&e, Outcome::NoActiveExecution), 1);
    verify(&e);
}
#[test]
fn channel_validation_limits_and_legacy_omission() {
    let s = scene(CostMode::OnStart);
    for (interval, pulses) in [(0, 1), (1, 0), (3601, 1), (1, 3601), (3600, 61)] {
        let mut invalid = s.clone();
        let channel = invalid.ability_catalog.abilities[0]
            .channel
            .as_mut()
            .unwrap();
        channel.interval_ticks = interval;
        channel.pulses = pulses;
        assert!(invalid.validate().is_err());
    }
    let mut invalid = s.clone();
    invalid.version = 35;
    assert!(invalid.validate().is_err());
    let mut invalid = s.clone();
    invalid.ability_catalog.abilities[0].periodic = Some(ignis::periodic::Schedule {
        interval_ticks: 10,
        pulses: 3,
        reapply: ignis::periodic::Reapply::Refresh,
    });
    assert!(invalid.validate().is_err());
    let mut invalid = s.clone();
    invalid.ability_catalog.abilities[0].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 10.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    assert!(invalid.validate().is_err());
    let mut legacy = s;
    legacy.ability_catalog.abilities[0].channel = None;
    legacy.version = 35;
    assert!(legacy.validate().is_ok());
    assert!(
        serde_json::to_value(&legacy).unwrap()["ability_catalog"]["abilities"][0]
            .get("channel")
            .is_none()
    );
}

#[test]
fn pulse_budget_failure_preserves_cost_and_damage_of_that_pulse() {
    let mut s = scene(CostMode::PerPulse);
    s.ability_catalog.abilities[0].effects = vec![Effect::Noise { radius: 2. }; 8];
    s.ability_catalog.abilities[0].effects[0] = Effect::Damage {
        recipient: Recipient::Target,
        amount: 5,
    };
    let mut other = entity(&mut s, "target").clone();
    other.id = "third".into();
    other.name = "Third".into();
    other.transform.position = [4., 0.8, 0.];
    s.entities.push(other);
    let mut e = Engine::new(s.clone()).unwrap();
    for id in ["player", "target", "third"] {
        cast(&mut e, id, 1, Some(id));
    }
    e.advance(11).unwrap();
    assert_eq!(receipts(&e, Outcome::ChannelPulse), 3);
    assert_eq!(receipts(&e, Outcome::NoiseBudget), 0);
    assert_eq!(hp(&e, "third"), 95);
    assert_eq!(energy(&e, "third"), 95);
    assert!(active(&e, "third").is_some());
    assert_eq!(e.simulation.abilities.state.noises.len(), 21);
    verify(&e);
    // Inject serial exhaustion: only two pulses fit. The third remains atomic.
    let mut e = Engine::new(s).unwrap();
    e.simulation.abilities.state.emitted = ignis::noise::MAX_SERIAL - 14;
    for id in ["player", "target", "third"] {
        cast(&mut e, id, 1, Some(id));
    }
    e.advance(11).unwrap();
    assert_eq!(receipts(&e, Outcome::ChannelPulse), 2);
    assert_eq!(receipts(&e, Outcome::NoiseBudget), 1);
    assert_eq!(hp(&e, "third"), 100);
    assert_eq!(energy(&e, "third"), 100);
    assert!(active(&e, "third").is_none());
    assert_eq!(e.simulation.abilities.state.noises.len(), 14);
    assert_eq!(
        e.simulation.abilities.state.emitted,
        ignis::noise::MAX_SERIAL
    );
}

#[test]
fn channel_effects_can_expire_before_pulse_without_partial_damage_or_payment() {
    let mut s = scene(CostMode::PerPulse);
    let mut poison = s.ability_catalog.abilities[0].clone();
    poison.id = "poison".into();
    poison.channel = None;
    poison.costs.clear();
    poison.cooldown_ticks = 0;
    poison.periodic = Some(ignis::periodic::Schedule {
        interval_ticks: 5,
        pulses: 1,
        reapply: ignis::periodic::Reapply::Refresh,
    });
    poison.periodic_category = Some("poison".into());
    s.ability_catalog.abilities.push(poison);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("poison".into());
    s.ability_catalog.abilities[0].effects.push(Effect::Dispel {
        recipient: Recipient::Target,
        category: "poison".into(),
        max_instances: 1,
        require_match: true,
    });
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "player", 4, Some("target"));
    e.advance(1).unwrap();
    cast(&mut e, "player", 1, Some("target"));
    e.advance(11).unwrap();
    assert_eq!(receipts(&e, Outcome::ChannelStarted), 1);
    assert_eq!(receipts(&e, Outcome::NoMatchingEffect), 1);
    assert_eq!(hp(&e, "target"), 95);
    assert_eq!(energy(&e, "player"), 100);
    assert_eq!(receipts(&e, Outcome::ChannelPulse), 0);
    verify(&e);
}

#[test]
fn npc_policy_condition_interrupts_without_waiting_for_next_pulse() {
    let mut s = scene(CostMode::PerPulse);
    s.ability_catalog.abilities[0].selection = ignis::abilities::Selection::SelfActor;
    s.ability_catalog.abilities[0].range = 0.;
    s.ability_catalog.abilities[0].effects = vec![Effect::Heal {
        recipient: Recipient::Target,
        amount: 20,
    }];
    let n = entity(&mut s, "target");
    n.vitality.as_mut().unwrap().initial = 50;
    n.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":10,"policy":{"vitality_below":{"subject":"source","basis_points":7000}}}},{"priority":0,"when":"always","action":{"kind":"wait"}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(12).unwrap();
    assert_eq!(hp(&e, "target"), 70);
    assert_eq!(energy(&e, "target"), 95);
    assert!(active(&e, "target").is_none());
    assert_eq!(receipts(&e, Outcome::ChannelPulse), 1);
    verify(&e);
}
