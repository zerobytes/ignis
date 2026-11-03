use ignis::{
    abilities::{CastMovement, Effect, Outcome, Recipient, Selection},
    channel::{Channel, CostMode},
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
        {"id":"long","selection":"self_actor","range":0,"cast_ticks":100,"cooldown_ticks":0,"costs":[{"resource":"energy","units":10}],"effects":[{"kind":"heal","recipient":"target","amount":5}]},
        {"id":"interrupt","selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"interrupt","recipient":"target","require_active":true}]},
        {"id":"kill","selection":"actor","range":12,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]}
    ]})).unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"Target","body":"npc","color":[0.6,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.footsteps = None;
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["long","interrupt","kill"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":50,"maximum":100,"scale":1})).unwrap());
    }
    entity(&mut s, "player").transform.position = [0., 0.8, -4.];
    s
}
fn interrupt(required: bool) -> Effect {
    Effect::Interrupt {
        recipient: Recipient::Target,
        require_active: required,
    }
}
fn channel(s: &mut Scene, mode: CostMode) {
    s.ability_catalog.abilities[0].cast_ticks = 0;
    s.ability_catalog.abilities[0].channel = Some(Channel {
        interval_ticks: 10,
        pulses: 5,
        cost_mode: mode,
    });
}
fn projectile(s: &mut Scene) {
    s.ability_catalog.abilities[1].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 6.,
        half_extent: 0.1,
        max_distance: 20.,
    };
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
fn direct_interrupts_preparation_without_charging_victim_and_records_both_sides() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    let victim = active(&e, "target").unwrap().id;
    cast(&mut e, "player", 2, Some("target"));
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_none());
    assert_eq!(energy(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 95);
    let s = e.execute(Command::InspectAbilities.into()).unwrap();
    let r = s["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["outcome"] == "interrupted_by_ability")
        .unwrap();
    assert_eq!(r["execution_id"], victim);
    assert_eq!(r["interrupted_by"]["actor"], "player");
    let applied = s["receipts"].as_array().unwrap().last().unwrap();
    assert_eq!(applied["effects"][0]["interrupt"]["execution_id"], victim);
    assert_eq!(r["interrupted_by"]["execution_id"], applied["execution_id"]);
    e.advance(110).unwrap();
    assert_eq!(hp(&e, "target"), 50);
    verify(&e);
}
#[test]
fn required_absent_or_protected_fails_free_optional_is_paid_and_diagnostic() {
    for protected in [false, true] {
        for required in [false, true] {
            let mut s = scene();
            s.ability_catalog.abilities[0].interruptible = !protected;
            s.ability_catalog.abilities[1].effects = vec![interrupt(required)];
            let mut e = Engine::new(s).unwrap();
            if protected {
                cast(&mut e, "target", 1, None);
                e.advance(1).unwrap();
            }
            cast(&mut e, "player", 2, Some("target"));
            e.advance(1).unwrap();
            assert_eq!(energy(&e, "player"), if required { 100 } else { 95 });
            assert_eq!(active(&e, "target").is_some(), protected);
            if required {
                assert_eq!(receipts(&e, Outcome::NoInterruptibleExecution), 1);
            } else {
                let s = e.execute(Command::InspectAbilities.into()).unwrap();
                assert_eq!(
                    s["receipts"].as_array().unwrap().last().unwrap()["effects"][0]["interrupt"]["status"],
                    if protected {
                        "protected"
                    } else {
                        "not_casting"
                    }
                );
            }
            verify(&e);
        }
    }
}
#[test]
fn duplicate_required_rolls_back_and_optional_does_not_interrupt_twice() {
    for required in [false, true] {
        let mut s = scene();
        s.ability_catalog.abilities[1].effects = vec![interrupt(true), interrupt(required)];
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "target", 1, None);
        e.advance(1).unwrap();
        cast(&mut e, "player", 2, Some("target"));
        e.advance(1).unwrap();
        assert_eq!(active(&e, "target").is_some(), required);
        assert_eq!(energy(&e, "player"), if required { 100 } else { 95 });
        assert_eq!(
            receipts(&e, Outcome::InterruptedByAbility),
            usize::from(!required)
        );
        verify(&e);
    }
}
#[test]
fn failed_later_effect_preserves_cast_health_and_cost() {
    let mut s = scene();
    s.ability_catalog.abilities[1].effects.extend([
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
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    cast(&mut e, "player", 2, Some("target"));
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_some());
    assert_eq!(energy(&e, "player"), 100);
    assert_eq!(hp(&e, "target"), 50);
    assert_eq!(receipts(&e, Outcome::NoMatchingEffect), 1);
    verify(&e);
}
#[test]
fn combined_lethal_effect_attributes_interruption_once() {
    let mut s = scene();
    s.ability_catalog.abilities[1].effects.insert(
        0,
        Effect::Damage {
            recipient: Recipient::Target,
            amount: 100,
        },
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    cast(&mut e, "player", 2, Some("target"));
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "target"), 0);
    assert_eq!(receipts(&e, Outcome::InterruptedByAbility), 1);
    assert_eq!(receipts(&e, Outcome::CancelledByDeath), 0);
    verify(&e);
}
#[test]
fn channel_keeps_prior_pulses_and_payments() {
    for mode in [CostMode::OnStart, CostMode::PerPulse] {
        let mut s = scene();
        channel(&mut s, mode);
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "target", 1, None);
        e.advance(21).unwrap();
        assert_eq!(hp(&e, "target"), 60);
        cast(&mut e, "player", 2, Some("target"));
        e.advance(60).unwrap();
        assert_eq!(hp(&e, "target"), 60);
        assert_eq!(
            energy(&e, "target"),
            if mode == CostMode::OnStart { 90 } else { 80 }
        );
        assert_eq!(receipts(&e, Outcome::ChannelPulse), 2);
        verify(&e);
    }
}
#[test]
fn protection_still_allows_voluntary_movement_and_death_cancellation() {
    for cause in 0..3 {
        let mut s = scene();
        s.ability_catalog.abilities[0].interruptible = false;
        s.ability_catalog.abilities[0].movement = CastMovement::Interrupt {
            speed_tolerance: 0.2,
        };
        let mut e = Engine::new(s).unwrap();
        e.advance(60).unwrap();
        cast(&mut e, "player", 1, None);
        e.advance(1).unwrap();
        match cause {
            0 => cancel(&mut e, "player"),
            1 => input(&mut e, 1.),
            _ => cast(&mut e, "target", 3, Some("player")),
        };
        e.advance(1).unwrap();
        assert!(active(&e, "player").is_none());
        assert_eq!(
            receipts(
                &e,
                [
                    Outcome::Cancelled,
                    Outcome::InterruptedByMovement,
                    Outcome::CancelledByDeath
                ][cause]
            ),
            1
        );
        verify(&e);
    }
}
#[test]
fn preparing_interrupt_revalidates_current_execution_at_commit() {
    for replace in [false, true] {
        let mut s = scene();
        s.ability_catalog.abilities[1].cast_ticks = 10;
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "target", 1, None);
        e.advance(1).unwrap();
        cast(&mut e, "player", 2, Some("target"));
        e.advance(1).unwrap();
        cancel(&mut e, "target");
        e.advance(1).unwrap();
        if replace {
            cast(&mut e, "target", 1, None);
            e.advance(1).unwrap();
        }
        e.advance(11).unwrap();
        assert!(active(&e, "target").is_none());
        assert_eq!(energy(&e, "player"), if replace { 95 } else { 100 });
        assert_eq!(
            receipts(&e, Outcome::InterruptedByAbility),
            usize::from(replace)
        );
        verify(&e);
    }
}
#[test]
fn projectile_revalidates_on_impact_and_does_not_refund() {
    for replace in [false, true] {
        let mut s = scene();
        projectile(&mut s);
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "target", 1, None);
        e.advance(1).unwrap();
        cast(&mut e, "player", 2, Some("target"));
        e.advance(1).unwrap();
        assert_eq!(energy(&e, "player"), 95);
        cancel(&mut e, "target");
        e.advance(1).unwrap();
        if replace {
            cast(&mut e, "target", 1, None);
            e.advance(1).unwrap();
        }
        e.advance(50).unwrap();
        assert!(active(&e, "target").is_none());
        assert_eq!(energy(&e, "player"), 95);
        assert_eq!(
            receipts(&e, Outcome::InterruptedByAbility),
            usize::from(replace)
        );
        assert_eq!(
            receipts(&e, Outcome::NoInterruptibleExecution),
            usize::from(!replace)
        );
        verify(&e);
    }
}
#[test]
fn spatial_projectile_can_hit_a_protected_actor_without_inspecting_at_launch() {
    let mut s = scene();
    projectile(&mut s);
    s.ability_catalog.abilities[1].selection = Selection::Spatial;
    s.ability_catalog.abilities[1].range = 0.;
    s.ability_catalog.abilities[0].interruptible = false;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 2,
            aim: ignis::abilities::Aim::Direction {
                vector: [0., 0., 1.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(50).unwrap();
    assert!(active(&e, "target").is_some());
    assert_eq!(energy(&e, "player"), 95);
    assert_eq!(receipts(&e, Outcome::NoInterruptibleExecution), 1);
    verify(&e);
}
#[test]
fn hidden_target_cast_state_is_not_disclosed_to_npc() {
    for casting in [false, true] {
        let mut s = scene();
        s.entities.push(serde_json::from_value(json!({"id":"wall","name":"Wall","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[0,1.5,-2],"size":[8,3,0.4]}})).unwrap());
        let mut e = Engine::new(s).unwrap();
        if casting {
            cast(&mut e, "player", 1, None);
        }
        e.advance(1).unwrap();
        cast(&mut e, "target", 2, Some("player"));
        e.advance(1).unwrap();
        assert_eq!(receipts(&e, Outcome::TargetUnavailable), 1);
        assert_eq!(receipts(&e, Outcome::NoInterruptibleExecution), 0);
        assert_eq!(energy(&e, "target"), 100);
        verify(&e);
    }
}
#[test]
fn npc_cannot_restart_in_same_tick_and_honors_retry_after_interruption() {
    let mut s = scene();
    entity(&mut s,"target").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":10}},{"priority":0,"when":"always","action":{"kind":"wait"}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_some());
    cast(&mut e, "player", 2, Some("target"));
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_none());
    assert_eq!(
        e.simulation.abilities.state.retry_at[index(&e, "target")][0],
        11
    );
    e.advance(9).unwrap();
    assert!(active(&e, "target").is_none());
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_some());
    verify(&e);
}
#[test]
fn same_tick_external_start_is_busy_next_tick_is_allowed() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    cast(&mut e, "player", 2, Some("target"));
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_none());
    assert_eq!(receipts(&e, Outcome::Busy), 1);
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_some());
    verify(&e);
}
#[test]
fn due_pulse_precedes_new_direct_interrupt_but_prepared_interrupt_follows_actor_order() {
    for prepared in [false, true] {
        let mut s = scene();
        channel(&mut s, CostMode::PerPulse);
        if prepared {
            s.ability_catalog.abilities[1].cast_ticks = 9;
        }
        let mut e = Engine::new(s).unwrap();
        cast(&mut e, "target", 1, None);
        e.advance(1).unwrap();
        if !prepared {
            e.advance(9).unwrap();
        }
        cast(&mut e, "player", 2, Some("target"));
        e.advance(if prepared { 10 } else { 1 }).unwrap();
        assert!(active(&e, "target").is_none());
        assert_eq!(hp(&e, "target"), if prepared { 50 } else { 55 });
        verify(&e);
    }
}
#[test]
fn legacy_defaults_and_invalid_compositions() {
    let s = scene();
    assert!(s.ability_catalog.abilities[0].interruptible);
    assert!(
        serde_json::to_value(&s).unwrap()["ability_catalog"]["abilities"][0]
            .get("interruptible")
            .is_none()
    );
    for bad in 0..6 {
        let mut s = s.clone();
        match bad {
            0 => s.version = 36,
            1 => s.ability_catalog.abilities[1].selection = Selection::SelfActor,
            2 => {
                s.ability_catalog.abilities[1].effects = vec![Effect::Interrupt {
                    recipient: Recipient::Source,
                    require_active: true,
                }]
            }
            3 => {
                s.ability_catalog.abilities[1].channel = Some(Channel {
                    interval_ticks: 10,
                    pulses: 2,
                    cost_mode: CostMode::OnStart,
                })
            }
            4 => {
                s.ability_catalog.abilities[1].periodic = Some(ignis::periodic::Schedule {
                    interval_ticks: 10,
                    pulses: 2,
                    reapply: ignis::periodic::Reapply::Refresh,
                })
            }
            _ => {
                s.ability_catalog.abilities[1].effects = vec![Effect::Heal {
                    recipient: Recipient::Target,
                    amount: 5,
                }];
                s.ability_catalog.abilities[0].interruptible = false;
                s.version = 36;
            }
        }
        assert!(s.validate().is_err());
    }
    let mut legacy = s;
    legacy.ability_catalog.abilities[1].effects = vec![Effect::Heal {
        recipient: Recipient::Target,
        amount: 5,
    }];
    legacy.version = 36;
    assert!(legacy.validate().is_ok());
}

#[test]
fn interrupt_does_not_remove_already_installed_periodic_effects_or_launched_shots() {
    let mut s = scene();
    let mut poison = s.ability_catalog.abilities[2].clone();
    poison.id = "poison".into();
    poison.effects = vec![Effect::Damage {
        recipient: Recipient::Target,
        amount: 2,
    }];
    poison.periodic = Some(ignis::periodic::Schedule {
        interval_ticks: 20,
        pulses: 3,
        reapply: ignis::periodic::Reapply::Refresh,
    });
    let mut shot = poison.clone();
    shot.id = "damage_shot".into();
    shot.periodic = None;
    shot.delivery = ignis::projectiles::Delivery::Projectile {
        speed: 3.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    s.ability_catalog.abilities.extend([poison, shot]);
    entity(&mut s, "target")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .extend(["poison".into(), "damage_shot".into()]);
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    cast(&mut e, "target", 4, Some("player"));
    e.advance(1).unwrap();
    cast(&mut e, "target", 5, Some("player"));
    e.advance(1).unwrap();
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    cast(&mut e, "player", 2, Some("target"));
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 1);
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 1);
    e.advance(100).unwrap();
    assert_eq!(hp(&e, "player"), 42);
    assert_eq!(receipts(&e, Outcome::PeriodicPulse), 3);
    assert_eq!(receipts(&e, Outcome::Impacted), 1);
    verify(&e);
}
#[test]
fn projectile_uses_physical_interceptor_cast() {
    let mut s = scene();
    projectile(&mut s);
    // The intended actor is behind an interceptor. Spatial delivery permits launch despite occlusion.
    s.ability_catalog.abilities[1].selection = Selection::Spatial;
    s.ability_catalog.abilities[1].range = 0.;
    let mut interceptor = entity(&mut s, "target").clone();
    interceptor.id = "interceptor".into();
    interceptor.transform.position = [0., 0.8, -2.];
    s.entities.push(interceptor);
    let mut e = Engine::new(s).unwrap();
    for id in ["target", "interceptor"] {
        cast(&mut e, id, 1, None);
    }
    e.advance(1).unwrap();
    let victim = active(&e, "interceptor").unwrap().id;
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 2,
            aim: ignis::abilities::Aim::Direction {
                vector: [0., 0., 1.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(25).unwrap();
    assert!(active(&e, "target").is_some());
    assert!(active(&e, "interceptor").is_none());
    let r = e
        .simulation
        .abilities
        .state
        .receipts
        .iter()
        .find(|r| r.outcome == Outcome::InterruptedByAbility)
        .unwrap();
    assert_eq!(r.execution_id, Some(victim));
    verify(&e);
}
#[test]
fn insufficient_cost_does_not_interrupt_and_self_target_is_ineligible() {
    let mut s = scene();
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .initial = 4;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "target", 1, None);
    e.advance(1).unwrap();
    cast(&mut e, "player", 2, Some("target"));
    e.advance(1).unwrap();
    assert!(active(&e, "target").is_some());
    assert_eq!(energy(&e, "player"), 4);
    assert_eq!(receipts(&e, Outcome::InsufficientResource), 1);
    verify(&e);
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, "player", 2, Some("player"));
    e.advance(1).unwrap();
    assert_eq!(receipts(&e, Outcome::NoInterruptibleExecution), 1);
    assert_eq!(energy(&e, "player"), 100);
    verify(&e);
}
