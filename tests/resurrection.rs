use ignis::{
    abilities::{Effect, Outcome, Recipient},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::json;
fn actor(id: &str, x: f32, initial: i64) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"npc","color":[0.3,0.8,0.4],"blocks_vision":false,
        "transform":{"position":[x,0.8,0],"size":[0.6,1.6,0.6]},"vitality":{"initial":initial,"maximum":100,"scale":1},
        "abilities":{"resources":[{"resource":"energy","initial":73,"capacity":100}],"grants":["kill","revive","heal","burn","aura","end_aura"]},
        "npc":{"surface":"floor","target":"victim","speed":2,"stop_distance":1,"enabled":true,
            "awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":60,"search_turn_degrees_per_second":0},
            "objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap()
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
    let def = |id, kind, amount, cast| {
        json!({"id":id,"selection":"actor","range":20,"cast_ticks":cast,"cooldown_ticks":120,
        "costs":[{"resource":"energy","units":5}],"effects":[{"kind":kind,"recipient":"target","amount":amount}]})
    };
    let mut burn = def("burn", "damage", 1, 0);
    burn["periodic"] = json!({"interval_ticks":60,"pulses":5,"reapply":"independent"});
    let mut aura = def("aura", "damage", 1, 0);
    aura["selection"] = json!("self_actor");
    aura["range"] = json!(0);
    aura["area"] = json!({"radius":3,"max_targets":9,"include_source":false,"occlusion":false,
        "persistence":{"duration_ticks":600,"interval_ticks":60,"apply_on_spawn":false,"reapply":"independent","end_on_source_death":false,"anchor":"source"}});
    let mut end = aura.clone();
    end["id"] = json!("end_aura");
    end["area"]["persistence"]["end_on_source_death"] = json!(true);
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[def("kill","damage",100,0),def("revive","resurrect",40,0),def("heal","heal",10,5),burn,aura,end]})).unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [0., 0.8, -4.];
    p.footsteps = None;
    p.blocks_vision = Some(false);
    p.vitality = Some(ignis::vitality::Config {
        initial: 100,
        maximum: 100,
        scale: 1,
    });
    p.abilities = actor("unused", 0., 100).abilities;
    let mut v = actor("victim", 0., 100);
    v.npc.as_mut().unwrap().target = Some("player".into());
    s.entities.push(v);
    s
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
fn life(e: &Engine, id: &str) -> u64 {
    e.simulation.vitality.state.lives[slot(e, id)]
}
fn use_(e: &mut Engine, a: &str, s: u8, t: Option<&str>) {
    e.execute(
        Command::ActivateAbility {
            actor: a.into(),
            slot: s,
            target: t.map(str::to_owned),
        }
        .into(),
    )
    .unwrap();
}
fn step_use(e: &mut Engine, a: &str, s: u8, t: Option<&str>) {
    use_(e, a, s, t);
    e.advance(1).unwrap();
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
#[test]
fn same_body_preserves_resources_cooldowns_and_supports_repeated_death() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cooldown_ticks = 0;
    s.ability_catalog.abilities[1].cooldown_ticks = 0;
    let mut e = Engine::new(s).unwrap();
    step_use(&mut e, "victim", 5, None);
    let a = slot(&e, "victim");
    let balance = e.simulation.abilities.state.balances[a];
    let deadlines = e.simulation.abilities.state.ready_at[a];
    let authored = e.scene.clone();
    for n in 1..=3 {
        step_use(&mut e, "player", 1, Some("victim"));
        assert_eq!(hp(&e, "victim"), 0);
        let position = e.simulation.positions()["victim"];
        step_use(&mut e, "player", 2, Some("victim"));
        assert_eq!(hp(&e, "victim"), 40);
        assert_eq!(life(&e, "victim"), n);
        assert_eq!(e.scene, authored);
        assert!((e.simulation.positions()["victim"][0] - position[0]).abs() < 0.001);
        assert_eq!(e.simulation.abilities.state.balances[a], balance);
        assert_eq!(e.simulation.abilities.state.ready_at[a], deadlines);
        assert_ne!(
            e.simulation.npcs["victim"].phase,
            ignis::perception::Phase::Dead
        );
    }
    assert_eq!(e.simulation.vitality.state.deaths.len(), 3);
    assert_eq!(e.simulation.vitality.state.revivals.len(), 3);
    replay(&e);
}
#[test]
fn alive_missing_vitality_and_failed_composition_never_charge() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.simulation.abilities.state.balances.clone();
    step_use(&mut e, "player", 2, Some("victim"));
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::TargetAlive
    );
    assert_eq!(before, e.simulation.abilities.state.balances);
    let mut s = scene();
    entity(&mut s, "victim").vitality = None;
    let mut e = Engine::new(s).unwrap();
    step_use(&mut e, "player", 2, Some("victim"));
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::MissingVitality
    );
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    s.ability_catalog
        .resources
        .push(ignis::abilities::Resource {
            id: "absent".into(),
            scale: 1,
        });
    s.ability_catalog.abilities[1]
        .effects
        .push(Effect::Resource {
            recipient: Recipient::Source,
            resource: "absent".into(),
            delta: 1,
        });
    assert!(Engine::new(s).is_err()); // Missing source pool is rejected before simulation exists.
}
#[test]
fn two_observing_npcs_compete_without_double_payment() {
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    s.ability_catalog.abilities[1].cast_ticks = 2;
    for (id, x) in [("a_healer", -3.), ("b_healer", 3.)] {
        s.entities.push(actor(id, x, 100));
    }
    let mut e = Engine::new(s).unwrap();
    use_(&mut e, "a_healer", 2, Some("victim"));
    use_(&mut e, "b_healer", 2, Some("victim"));
    e.advance(3).unwrap();
    assert_eq!(hp(&e, "victim"), 40);
    assert_eq!(life(&e, "victim"), 1);
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "a_healer")][0],
        68
    );
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "b_healer")][0],
        73
    );
    replay(&e);
}
#[test]
fn death_and_revival_same_tick_clean_old_effects_and_invalidate_cast() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 2;
    s.entities.push(actor("a_killer", -3., 100));
    s.entities.push(actor("z_healer", 3., 100));
    let mut e = Engine::new(s).unwrap();
    step_use(&mut e, "victim", 6, None);
    step_use(&mut e, "player", 4, Some("victim"));
    use_(&mut e, "a_killer", 1, Some("victim"));
    use_(&mut e, "z_healer", 3, Some("victim"));
    e.advance(2).unwrap();
    use_(&mut e, "player", 2, Some("victim"));
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "victim"), 40);
    assert_eq!(life(&e, "victim"), 1);
    assert_eq!(
        e.simulation.vitality.state.deaths.back().unwrap().tick,
        e.simulation.vitality.state.revivals.back().unwrap().tick
    );
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .is_empty()
    );
    e.advance(4).unwrap();
    assert_eq!(hp(&e, "victim"), 40);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.actor as usize == slot(&e, "z_healer")
                && matches!(r.outcome, Outcome::LifeChanged | Outcome::TargetUnavailable))
    );
    replay(&e);
}
#[test]
fn cancellation_at_deadline_and_insufficient_resources_leave_corpse_unchanged() {
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    s.ability_catalog.abilities[1].cast_ticks = 2;
    let mut e = Engine::new(s.clone()).unwrap();
    step_use(&mut e, "player", 2, Some("victim"));
    e.advance(1).unwrap();
    let id = e.simulation.abilities.state.casts[slot(&e, "player")]
        .unwrap()
        .id;
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: id,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "victim"), 0);
    replay(&e);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .initial = 4;
    let mut e = Engine::new(s).unwrap();
    step_use(&mut e, "player", 2, Some("victim"));
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::InsufficientResource
    );
    assert_eq!(hp(&e, "victim"), 0);
}
#[test]
fn living_rediscovered_only_after_new_visual_confirmation() {
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    s.entities.push(actor("observer", -3., 100));
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let victim_slot = slot(&e, "victim");
    assert!(
        e.simulation.npcs["observer"]
            .known_dead
            .contains(victim_slot)
    );
    step_use(&mut e, "player", 2, Some("victim"));
    assert!(
        e.simulation.npcs["observer"]
            .known_dead
            .contains(victim_slot)
    );
    e.advance(1).unwrap();
    assert!(
        !e.simulation.npcs["observer"]
            .known_dead
            .contains(victim_slot)
    );
    assert_eq!(e.simulation.npcs["observer"].memory.unwrap().life, Some(1));
    replay(&e);
}
#[test]
fn old_queued_request_does_not_rebind_to_new_life() {
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    s.entities.push(actor("a_healer", -3., 100));
    let mut e = Engine::new(s).unwrap();
    use_(&mut e, "a_healer", 2, Some("victim"));
    use_(&mut e, "player", 2, Some("victim"));
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::LifeChanged
    );
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "player")][0],
        73
    );
    replay(&e);
}
#[test]
fn resurrection_schema_rejects_ambiguous_delivery_and_target_effects() {
    let base = scene();
    for variant in 0..6 {
        let mut s = base.clone();
        let d = &mut s.ability_catalog.abilities[1];
        match variant {
            0 => d.selection = ignis::abilities::Selection::SelfActor,
            1 => {
                d.delivery = ignis::projectiles::Delivery::Projectile {
                    speed: 3.,
                    half_extent: 0.1,
                    max_distance: 20.,
                }
            }
            2 => d.effects.push(Effect::Heal {
                recipient: Recipient::Target,
                amount: 10,
            }),
            3 => {
                d.effects = vec![Effect::Resurrect {
                    recipient: Recipient::Target,
                    amount: 0,
                }]
            }
            4 => d.effects.push(d.effects[0].clone()),
            _ => {
                d.periodic = Some(
                    serde_json::from_value(
                        json!({"interval_ticks":60,"pulses":2,"reapply":"refresh"}),
                    )
                    .unwrap(),
                )
            }
        };
        assert!(s.validate().is_err(), "variant {variant}");
    }
}

#[test]
fn projectile_collision_acquires_current_life_after_resurrection() {
    let mut s = scene();
    let mut bolt = s.ability_catalog.abilities[0].clone();
    bolt.id = "bolt".into();
    bolt.effects = vec![Effect::Damage {
        recipient: Recipient::Target,
        amount: 10,
    }];
    bolt.delivery = ignis::projectiles::Delivery::Projectile {
        speed: 10.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    s.ability_catalog.abilities.push(bolt);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("bolt".into());
    let mut e = Engine::new(s).unwrap();
    step_use(&mut e, "player", 7, Some("victim"));
    step_use(&mut e, "player", 1, Some("victim"));
    step_use(&mut e, "player", 2, Some("victim"));
    assert_eq!(hp(&e, "victim"), 40);
    e.advance(40).unwrap();
    assert_eq!(hp(&e, "victim"), 30);
    replay(&e);
}
#[test]
fn epoch_budget_rejects_without_overflow_or_payment() {
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    let mut e = Engine::new(s).unwrap();
    let a = slot(&e, "victim");
    e.simulation.vitality.state.lives[a] = u64::MAX;
    step_use(&mut e, "player", 2, Some("victim"));
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::LifeBudget
    );
    assert_eq!(hp(&e, "victim"), 0);
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "player")][0],
        73
    );
}

#[test]
fn flying_body_reactivates_existing_motor_without_teleport_or_new_capability() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-flight.scene.json")).unwrap();
    s.version = ignis::model::SCENE_VERSION;
    let template = scene();
    s.ability_catalog = template.ability_catalog.clone();
    entity(&mut s, "player").abilities = entity(&mut template.clone(), "player").abilities.clone();
    let id = s
        .entities
        .iter()
        .find(|e| {
            e.npc
                .as_ref()
                .is_some_and(|n| n.mobility.is_some_and(|m| m.fly_speed.is_some()))
        })
        .unwrap()
        .id
        .clone();
    for e in &mut s.entities {
        e.blocks_vision = Some(false);
    }
    entity(&mut s, &id).vitality = Some(ignis::vitality::Config {
        initial: 0,
        maximum: 100,
        scale: 1,
    });
    s.ability_catalog.abilities[0].range = 100.;
    s.ability_catalog.abilities[1].range = 100.;
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    let pos = e.simulation.positions()[&id];
    step_use(&mut e, "player", 2, Some(&id));
    let now = e.simulation.positions()[&id];
    assert!((now[0] - pos[0]).abs() < 0.2 && (now[2] - pos[2]).abs() < 0.2);
    e.advance(60).unwrap();
    assert_ne!(e.simulation.positions()[&id], now);
    assert_ne!(e.simulation.npcs[&id].phase, ignis::perception::Phase::Dead);
    step_use(&mut e, "player", 1, Some(&id));
    assert_eq!(e.simulation.npcs[&id].phase, ignis::perception::Phase::Dead);
    let dead_y = e.simulation.positions()[&id][1];
    e.advance(20).unwrap();
    assert!(e.simulation.positions()[&id][1] <= dead_y + 0.01);
    replay(&e);
}

#[test]
fn later_noise_budget_failure_rolls_back_resurrection_and_cost() {
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    s.ability_catalog.abilities[1]
        .effects
        .push(Effect::Noise { radius: 1. });
    let mut noisy = s.ability_catalog.abilities[1].clone();
    noisy.id = "noisy".into();
    noisy.selection = ignis::abilities::Selection::SelfActor;
    noisy.range = 0.;
    noisy.costs.clear();
    noisy.effects = vec![Effect::Noise { radius: 1. }; 8];
    s.ability_catalog.abilities.push(noisy);
    for (id, x) in [("a_noise", -3.), ("b_noise", 3.)] {
        let mut n = actor(id, x, 100);
        n.abilities.as_mut().unwrap().grants.push("noisy".into());
        s.entities.push(n);
    }
    let mut e = Engine::new(s).unwrap();
    // Distinct sources no longer share a quota of 16. Prove the ordinary
    // execution/replay, then exhaust the global serial space for late rollback.
    use_(&mut e, "a_noise", 7, None);
    use_(&mut e, "b_noise", 7, None);
    use_(&mut e, "player", 2, Some("victim"));
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "victim"), 40);
    replay(&e);
    e.reset();
    e.simulation.abilities.state.emitted = ignis::noise::MAX_SERIAL - 16;
    use_(&mut e, "a_noise", 7, None);
    use_(&mut e, "b_noise", 7, None);
    use_(&mut e, "player", 2, Some("victim"));
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::NoiseBudget
    );
    assert_eq!(hp(&e, "victim"), 0);
    assert_eq!(life(&e, "victim"), 0);
    assert!(e.simulation.vitality.state.revivals.is_empty());
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "player")][0],
        73
    );
    assert_eq!(
        e.simulation.abilities.state.emitted,
        ignis::noise::MAX_SERIAL
    );
}
#[test]
fn externally_commanded_npc_cannot_resurrect_through_occlusion() {
    let mut s = scene();
    entity(&mut s, "victim").vitality.as_mut().unwrap().initial = 0;
    s.entities.push(actor("healer", -3., 100));
    s.entities.push(serde_json::from_value(json!({"id":"screen","name":"screen","body":"fixed","color":[0.3,0.6,0.7],"transform":{"position":[-1.5,1.5,0],"size":[0.4,3,4]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    step_use(&mut e, "healer", 2, Some("victim"));
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
    assert_eq!(hp(&e, "victim"), 0);
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "healer")][0],
        73
    );
    replay(&e);
}

#[test]
fn revival_history_is_bounded_and_reset_restores_authored_life() {
    let mut s = scene();
    for d in &mut s.ability_catalog.abilities {
        d.costs.clear();
        d.cooldown_ticks = 0;
    }
    let mut e = Engine::new(s).unwrap();
    for _ in 0..257 {
        step_use(&mut e, "player", 1, Some("victim"));
        step_use(&mut e, "player", 2, Some("victim"));
    }
    assert_eq!(life(&e, "victim"), 257);
    assert_eq!(e.simulation.vitality.state.revivals.len(), 256);
    assert_eq!(e.simulation.vitality.state.dropped_revivals, 1);
    assert_eq!(e.simulation.vitality.state.deaths.len(), 256);
    replay(&e);
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(hp(&e, "victim"), 100);
    assert_eq!(life(&e, "victim"), 0);
    assert!(e.simulation.vitality.state.revivals.is_empty());
}

#[test]
fn resurrection_keeps_instance_and_external_reference_but_advances_life() {
    let mut e = Engine::new(scene()).unwrap();
    let instances = e.execute(Command::InspectInstances.into()).unwrap();
    let token = |id: &str| {
        instances["instances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["authored_id"] == id)
            .unwrap()["reference"]
            .as_str()
            .unwrap()
    };
    let p = token("player");
    let v = token("victim");
    let reference = e.simulation.actors.reference(slot(&e, "victim") as u16);
    step_use(&mut e, p, 1, Some(v));
    assert_eq!(hp(&e, "victim"), 0);
    step_use(&mut e, p, 2, Some(v));
    assert_eq!(hp(&e, "victim"), 40);
    assert_eq!(life(&e, "victim"), 1);
    assert_eq!(
        e.simulation.actors.reference(slot(&e, "victim") as u16),
        reference
    );
    let after = e.execute(Command::InspectInstances.into()).unwrap();
    let row = after["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["authored_id"] == "victim")
        .unwrap();
    assert_eq!(row["reference"], v);
    assert_eq!(row["life_index"], 1);
    replay(&e);
}

#[test]
fn stale_cast_source_or_target_instance_cannot_spend_or_apply() {
    for source in [true, false] {
        let mut s = scene();
        s.ability_catalog.abilities[0].cast_ticks = 5;
        let mut e = Engine::new(s).unwrap();
        use_(&mut e, "player", 1, Some("victim"));
        e.advance(1).unwrap();
        let a = slot(&e, "player");
        let balances = e.simulation.abilities.state.balances.clone();
        let cooldowns = e.simulation.abilities.state.ready_at.clone();
        let cast = e.simulation.abilities.state.casts[a].as_mut().unwrap();
        if source {
            cast.life.source_instance.generation += 1;
        } else {
            cast.life.target_instance.as_mut().unwrap().instance += 1;
        }
        e.advance(6).unwrap();
        assert_eq!(hp(&e, "victim"), 100);
        assert_eq!(e.simulation.abilities.state.balances, balances);
        assert_eq!(e.simulation.abilities.state.ready_at, cooldowns);
        assert_eq!(
            e.simulation
                .abilities
                .state
                .receipts
                .back()
                .unwrap()
                .outcome,
            Outcome::InstanceChanged
        );
    }
}
