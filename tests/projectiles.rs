use ignis::{
    abilities::{Outcome, Recipient},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    physics::Input,
    projectiles::{Delivery, Outcome as Impact},
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
        {"id":"bolt","selection":"actor","range":20,"cast_ticks":10,"cooldown_ticks":30,"costs":[{"resource":"energy","units":5}],"delivery":{"kind":"projectile","speed":10,"half_extent":0.08,"max_distance":20},"effects":[{"kind":"damage","recipient":"target","amount":20}]},
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]}
    ]})).unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"target","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[4,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":30,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    for id in ["player", "target"] {
        let e = entity(&mut s, id);
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["bolt","kill"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
        e.footsteps = None;
    }
    entity(&mut s, "player").transform.position = [-4., 0.8, 0.];
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
fn health(e: &Engine, id: &str) -> i64 {
    e.simulation.vitality.state.actors[slot(e, id)]
        .unwrap()
        .units
}
fn energy(e: &Engine, id: &str) -> i64 {
    e.simulation.abilities.state.balances[slot(e, id)][0]
}
fn activate(e: &mut Engine, actor: &str, target: &str, slot: u8) {
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
fn outcome(e: &Engine) -> Outcome {
    e.simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .outcome
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn barrier(s: &mut Scene, id: &str, x: f32, visible: bool, blocks_vision: bool) {
    s.entities.push(serde_json::from_value(json!({"id":id,"name":id,"body":"fixed","visible":visible,"blocks_vision":blocks_vision,"color":[0.2,0.6,0.5],"transform":{"position":[x,1,0],"size":[0.05,2,2]}})).unwrap());
}
#[test]
fn launch_charges_once_impact_is_later_and_prefix_replays_active_flight() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(10).unwrap();
    assert_eq!(energy(&e, "player"), 1000);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Launched);
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(health(&e, "target"), 100);
    assert!(e.simulation.abilities.state.casts[slot(&e, "player")].is_none());
    assert_eq!(
        e.simulation.abilities.state.ready_at[slot(&e, "player")][0],
        40
    );
    let prefix = e.simulation.abilities.state.clone();
    e.advance(60).unwrap();
    assert_eq!(health(&e, "target"), 80);
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(outcome(&e), Outcome::Impacted);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .projectiles
            .receipts
            .back()
            .unwrap()
            .outcome,
        Impact::Hit
    );
    let restored = Engine::replay_at(&e.replay(), Some(11)).unwrap();
    assert_eq!(restored.simulation.abilities.state, prefix);
    replay(&e);
}
#[test]
fn direct_legacy_remains_immediate() {
    let mut s = scene();
    s.version = 29;
    s.ability_catalog.abilities[0].delivery = Delivery::Direct {};
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(11).unwrap();
    assert_eq!(health(&e, "target"), 80);
    assert_eq!(outcome(&e), Outcome::Completed);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    replay(&e);
}
#[test]
fn cancel_before_launch_free_after_launch_cannot_recall() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", "target", 1);
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
    e.advance(11).unwrap();
    assert_eq!(energy(&e, "player"), 1000);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    activate(&mut e, "player", "target", 1);
    e.advance(11).unwrap();
    let id = e.simulation.abilities.state.projectiles.active[0].cast.id;
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: id,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::NoActiveExecution);
    e.advance(60).unwrap();
    assert_eq!(health(&e, "target"), 80);
    replay(&e);
}
#[test]
fn sweep_hits_thin_invisible_wall_at_high_speed_not_target() {
    let mut s = scene();
    entity(&mut s, "player").transform.position = [-0.8, 0.8, 0.];
    entity(&mut s, "target").transform.position = [2., 0.8, 0.];
    barrier(&mut s, "thin", 0., false, false);
    s.ability_catalog.abilities[0].delivery = Delivery::Projectile {
        speed: 100.,
        half_extent: 0.01,
        max_distance: 20.,
    };
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(12).unwrap();
    assert_eq!(health(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 995);
    let r = e
        .simulation
        .abilities
        .state
        .projectiles
        .receipts
        .back()
        .unwrap();
    assert_eq!(r.outcome, Impact::Blocked);
    assert_eq!(e.scene.entities[r.entity.unwrap() as usize].id, "thin");
    assert!(r.position[0] <= 0.);
    replay(&e);
}
#[test]
fn visual_occlusion_rejects_launch_without_charge() {
    let mut s = scene();
    barrier(&mut s, "wall", 0., true, true);
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Occluded);
    assert_eq!(energy(&e, "player"), 1000);
    replay(&e);
}
#[test]
fn npc_shot_aim_stays_frozen_when_player_dodges() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "target", "player", 1);
    e.advance(11).unwrap();
    let heading = e.simulation.abilities.state.projectiles.active[0].direction;
    e.execute(
        Command::SetInput {
            input: Input {
                x: 0.,
                z: 1.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(30).unwrap();
    assert_eq!(
        e.simulation.abilities.state.projectiles.active[0].direction,
        heading
    );
    e.advance(120).unwrap();
    assert_eq!(health(&e, "player"), 100);
    assert_eq!(energy(&e, "target"), 995);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    replay(&e);
}
#[test]
fn range_expiration_stops_short_without_refund() {
    let mut s = scene();
    s.ability_catalog.abilities[0].delivery = Delivery::Projectile {
        speed: 10.,
        half_extent: 0.08,
        max_distance: 2.,
    };
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(40).unwrap();
    assert_eq!(health(&e, "target"), 100);
    assert_eq!(energy(&e, "player"), 995);
    let r = e
        .simulation
        .abilities
        .state
        .projectiles
        .receipts
        .back()
        .unwrap();
    assert_eq!(r.outcome, Impact::Expired);
    assert!((r.position[0] + 2.).abs() < 0.001);
    replay(&e);
}
#[test]
fn source_death_does_not_remove_launched_projectile() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(11).unwrap();
    activate(&mut e, "target", "player", 2);
    e.advance(1).unwrap();
    assert_eq!(health(&e, "player"), 0);
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 1);
    e.advance(60).unwrap();
    assert_eq!(health(&e, "target"), 80);
    replay(&e);
}
#[test]
fn impact_death_cancels_cast_before_deadline() {
    let mut s = scene();
    s.ability_catalog.abilities[0].effects = vec![ignis::abilities::Effect::Damage {
        recipient: Recipient::Target,
        amount: 100,
    }];
    s.ability_catalog.abilities[1].cast_ticks = 180;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(11).unwrap();
    activate(&mut e, "target", "player", 2);
    e.advance(1).unwrap();
    assert!(e.simulation.abilities.state.casts[slot(&e, "target")].is_some());
    e.advance(70).unwrap();
    assert_eq!(health(&e, "target"), 0);
    assert_eq!(health(&e, "player"), 100);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::CancelledByDeath)
    );
    assert_eq!(
        e.simulation.npcs["target"].phase,
        ignis::perception::Phase::Dead
    );
    replay(&e);
}
#[test]
fn bounded_pool_rejects_without_partial_cost_and_reset_clears() {
    let mut s = scene();
    let d = &mut s.ability_catalog.abilities[0];
    d.cast_ticks = 0;
    d.cooldown_ticks = 0;
    d.delivery = Delivery::Projectile {
        speed: 0.1,
        half_extent: 0.01,
        max_distance: 100.,
    };
    let mut e = Engine::new(s).unwrap();
    for _ in 0..64 {
        activate(&mut e, "player", "target", 1);
        e.advance(1).unwrap();
    }
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 64);
    let balance = energy(&e, "player");
    activate(&mut e, "player", "target", 1);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::ProjectileBudget);
    assert_eq!(energy(&e, "player"), balance);
    replay(&e);
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
}
#[test]
fn impact_effects_are_atomic_and_do_not_use_launch_snapshot() {
    let mut s = scene();
    s.ability_catalog
        .resources
        .push(ignis::abilities::Resource {
            id: "focus".into(),
            scale: 1,
        });
    entity(&mut s, "target")
        .abilities
        .as_mut()
        .unwrap()
        .resources
        .push(ignis::abilities::Pool {
            resource: "focus".into(),
            initial: 0,
            capacity: 100,
        });
    s.ability_catalog.abilities[0]
        .effects
        .push(ignis::abilities::Effect::Resource {
            recipient: Recipient::Target,
            resource: "focus".into(),
            delta: 10,
        });
    // A different actor physically intercepts the bolt, lacks focus, and must receive neither effect.
    let mut interceptor = entity(&mut s, "target").clone();
    interceptor.id = "interceptor".into();
    interceptor.name = "interceptor".into();
    interceptor.transform.position = [0., 0.8, 0.];
    interceptor.blocks_vision = Some(false);
    interceptor.abilities = None;
    s.entities.push(interceptor);
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(60).unwrap();
    assert_eq!(health(&e, "interceptor"), 100);
    assert_eq!(health(&e, "target"), 100);
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
        Impact::EffectFailed
    );
    assert_eq!(outcome(&e), Outcome::MissingResource);
    replay(&e);
}
#[test]
fn geometric_ties_use_entity_id_independent_of_scene_order() {
    let mut s = scene();
    barrier(&mut s, "z_barrier", 0., true, false);
    barrier(&mut s, "a_barrier", 0., true, false);
    let mut a = Engine::new(s.clone()).unwrap();
    s.entities.reverse();
    let mut b = Engine::new(s).unwrap();
    for e in [&mut a, &mut b] {
        activate(e, "player", "target", 1);
        e.advance(60).unwrap();
        let r = e
            .simulation
            .abilities
            .state
            .projectiles
            .receipts
            .back()
            .unwrap();
        assert_eq!(e.scene.entities[r.entity.unwrap() as usize].id, "a_barrier");
        replay(e);
    }
}
#[test]
fn invalid_parameters_self_target_and_source_effects_rejected() {
    let mut s = scene();
    s.version = 29;
    assert!(s.validate().is_err());
    s.version = 30;
    for speed in [0., 101., f32::INFINITY, f32::NAN] {
        s.ability_catalog.abilities[0].delivery = Delivery::Projectile {
            speed,
            half_extent: 0.1,
            max_distance: 20.,
        };
        assert!(s.validate().is_err());
    }
    s = scene();
    s.ability_catalog.abilities[0].effects = vec![ignis::abilities::Effect::Heal {
        recipient: Recipient::Source,
        amount: 10,
    }];
    assert!(s.validate().is_err());
    let mut e = Engine::new(scene()).unwrap();
    assert!(
        e.execute(
            Command::ActivateAbility {
                actor: "player".into(),
                slot: 1,
                target: Some("player".into())
            }
            .into()
        )
        .is_err()
    );
    assert!(serde_json::from_value::<Delivery>(json!({"kind":"direct","speed":1})).is_err());
}
#[test]
fn zero_direction_fails_without_spending() {
    let mut s = scene();
    entity(&mut s, "target").transform.position = [-4., 0.8, 0.];
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::InvalidAim);
    assert_eq!(energy(&e, "player"), 1000);
}
#[test]
fn friendly_interceptor_blocks_hostile_bolt_without_damage() {
    let mut s = scene();
    entity(&mut s, "player").faction = Some("blue".into());
    entity(&mut s, "target").faction = Some("red".into());
    s.faction_relations = vec![ignis::targeting::Relation {
        from: "blue".into(),
        to: "red".into(),
        attitude: ignis::targeting::Attitude::Hostile,
    }];
    s.ability_catalog.abilities[0].relation = Some(ignis::targeting::Attitude::Hostile);
    let mut friend = entity(&mut s, "target").clone();
    friend.id = "friend".into();
    friend.name = "friend".into();
    friend.faction = Some("blue".into());
    friend.transform.position = [0., 0.8, 0.];
    friend.blocks_vision = Some(false);
    s.entities.push(friend);
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(60).unwrap();
    assert_eq!(health(&e, "friend"), 100);
    assert_eq!(health(&e, "target"), 100);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .projectiles
            .receipts
            .back()
            .unwrap()
            .outcome,
        Impact::RelationDenied
    );
    replay(&e);
}
#[test]
fn vertical_aim_hits_stationary_hover_actor() {
    let mut s = scene();
    let target = entity(&mut s, "target");
    target.transform.position[1] = 4.8;
    target.npc.as_mut().unwrap().mobility = Some(
        serde_json::from_value(json!({"walk":false,"fly_speed":3,"jump_speed":null,"glide":null}))
            .unwrap(),
    );
    target.npc.as_mut().unwrap().local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":6})).unwrap());
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(11).unwrap();
    assert!(e.simulation.abilities.state.projectiles.active[0].direction[1] > 0.4);
    e.advance(80).unwrap();
    assert_eq!(health(&e, "target"), 80);
    replay(&e);
}
#[test]
fn impact_heal_reads_current_health_and_never_revives() {
    for kill in [false, true] {
        let mut s = scene();
        entity(&mut s, "target").vitality.as_mut().unwrap().initial = 50;
        s.ability_catalog.abilities[0].effects = vec![ignis::abilities::Effect::Heal {
            recipient: Recipient::Target,
            amount: 20,
        }];
        if !kill {
            s.ability_catalog.abilities[1].effects = vec![ignis::abilities::Effect::Damage {
                recipient: Recipient::Target,
                amount: 20,
            }];
        }
        let mut e = Engine::new(s).unwrap();
        activate(&mut e, "player", "target", 1);
        e.advance(11).unwrap();
        activate(&mut e, "player", "target", 2);
        e.advance(1).unwrap();
        assert_eq!(health(&e, "target"), if kill { 0 } else { 30 });
        e.advance(60).unwrap();
        assert_eq!(health(&e, "target"), if kill { 0 } else { 50 });
        assert_eq!(
            e.simulation
                .abilities
                .state
                .projectiles
                .receipts
                .back()
                .unwrap()
                .outcome,
            if kill {
                Impact::TargetDead
            } else {
                Impact::Hit
            }
        );
        replay(&e);
    }
}
#[test]
fn automatic_npc_uses_projectile_executor_and_cannot_launch_at_hidden_target() {
    for hidden in [false, true] {
        let mut s = scene();
        entity(&mut s,"target").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":30}}])).unwrap());
        if hidden {
            barrier(&mut s, "wall", 0., true, true);
        }
        let mut e = Engine::new(s).unwrap();
        e.advance(70).unwrap();
        assert_eq!(health(&e, "player"), if hidden { 100 } else { 80 });
        assert_eq!(energy(&e, "target"), if hidden { 1000 } else { 990 });
        replay(&e);
    }
}
#[test]
fn one_source_filling_its_quota_does_not_block_another_casts_commit() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 0;
    s.ability_catalog.abilities[0].cooldown_ticks = 0;
    s.ability_catalog.abilities[0].delivery = Delivery::Projectile {
        speed: 0.1,
        half_extent: 0.01,
        max_distance: 100.,
    };
    let mut delayed = s.ability_catalog.abilities[0].clone();
    delayed.id = "kill".into();
    delayed.cast_ticks = 2;
    s.ability_catalog.abilities[1] = delayed;
    let mut e = Engine::new(s).unwrap();
    for _ in 0..63 {
        activate(&mut e, "player", "target", 1);
        e.advance(1).unwrap();
    }
    let before = energy(&e, "player");
    activate(&mut e, "player", "target", 2);
    activate(&mut e, "target", "player", 2);
    e.advance(1).unwrap();
    assert!(e.simulation.abilities.state.casts[slot(&e, "player")].is_some());
    assert!(e.simulation.abilities.state.casts[slot(&e, "target")].is_some());
    e.advance(2).unwrap();
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 65);
    assert_eq!(energy(&e, "player"), before - 5);
    assert_eq!(energy(&e, "target"), 995);
    assert_ne!(outcome(&e), Outcome::ProjectileBudget);
    replay(&e);
}
#[test]
fn first_eligible_actor_receives_effect_instead_of_intended_target() {
    let mut s = scene();
    let mut interceptor = entity(&mut s, "target").clone();
    interceptor.id = "interceptor".into();
    interceptor.name = "interceptor".into();
    interceptor.transform.position = [0., 0.8, 0.];
    interceptor.blocks_vision = Some(false);
    s.entities.push(interceptor);
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "target", 1);
    e.advance(80).unwrap();
    assert_eq!(health(&e, "interceptor"), 80);
    assert_eq!(health(&e, "target"), 100);
    let r = e
        .simulation
        .abilities
        .state
        .projectiles
        .receipts
        .back()
        .unwrap();
    assert_eq!(
        e.scene.entities[r.entity.unwrap() as usize].id,
        "interceptor"
    );
    replay(&e);
}
