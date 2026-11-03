use ignis::{
    abilities::{Aim, Outcome, Selection},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    projectiles::Outcome as Impact,
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
        {"id":"bolt","selection":"spatial","range":0,"cast_ticks":10,"cooldown_ticks":30,"costs":[{"resource":"energy","units":5}],"delivery":{"kind":"projectile","speed":10,"half_extent":0.08,"max_distance":20},"effects":[{"kind":"damage","recipient":"target","amount":20}]},
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
fn aim(e: &mut Engine, actor: &str, aim: Aim) {
    e.execute(
        Command::AimAbility {
            actor: actor.into(),
            slot: 1,
            aim,
        }
        .into(),
    )
    .unwrap();
}
fn direction() -> Aim {
    Aim::Direction {
        vector: [2., 0., 0.],
    }
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
fn free_direction_launches_without_selection_and_hits_only_on_contact() {
    let mut e = Engine::new(scene()).unwrap();
    aim(&mut e, "player", direction());
    replay(&e);
    e.advance(11).unwrap();
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(health(&e, "target"), 100);
    let shot = e.simulation.abilities.state.projectiles.active[0];
    assert_eq!(shot.direction, [1., 0., 0.]);
    assert_eq!(shot.cast.target, None);
    e.advance(60).unwrap();
    assert_eq!(health(&e, "target"), 80);
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(outcome(&e), Outcome::Impacted);
    replay(&e);
    let restored = Engine::replay_at(&e.replay(), Some(11)).unwrap();
    assert_eq!(
        &*restored.simulation.abilities.state.projectiles.active,
        &[shot]
    );
}
#[test]
fn point_is_a_frozen_direction_reference_not_a_stop_or_actor() {
    let mut e = Engine::new(scene()).unwrap();
    aim(
        &mut e,
        "player",
        Aim::Point {
            position: [-3., 0.8, 0.],
        },
    );
    e.advance(80).unwrap();
    assert_eq!(health(&e, "target"), 80); // travelled beyond the point
    replay(&e);
}
#[test]
fn vertical_miss_expires_and_is_paid_without_source_vitality() {
    let mut s = scene();
    entity(&mut s, "player").vitality = None;
    let mut e = Engine::new(s).unwrap();
    aim(
        &mut e,
        "player",
        Aim::Direction {
            vector: [0., 1., 0.],
        },
    );
    e.advance(140).unwrap();
    assert_eq!(energy(&e, "player"), 995);
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
        Impact::Expired
    );
    replay(&e);
}
#[test]
fn point_frozen_during_cast_but_launch_origin_follows_actual_motion() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 30;
    let mut e = Engine::new(s).unwrap();
    let point = [4., 0.8, 0.];
    aim(&mut e, "player", Aim::Point { position: point });
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
    e.advance(31).unwrap();
    let shot = e.simulation.abilities.state.projectiles.active[0];
    assert!(shot.position[2] > 0.5);
    let expected = (glam::Vec3::from(point) - glam::Vec3::from(shot.position)).normalize();
    assert!(glam::Vec3::from(shot.direction).distance(expected) < 0.00001);
    assert_eq!(shot.cast.aim, Some(Aim::Point { position: point }));
    replay(&e);
}
#[test]
fn direction_frozen_even_when_source_moves() {
    let mut e = Engine::new(scene()).unwrap();
    aim(&mut e, "player", direction());
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
    e.advance(11).unwrap();
    assert_eq!(
        e.simulation.abilities.state.projectiles.active[0].direction,
        [1., 0., 0.]
    );
    replay(&e);
}
#[test]
fn invalid_inputs_are_atomic_and_selection_modes_do_not_mix() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    for aim in [
        Aim::Direction { vector: [0.; 3] },
        Aim::Direction {
            vector: [f32::NAN, 0., 0.],
        },
        Aim::Direction {
            vector: [0.0001, 0., 0.],
        },
        Aim::Point {
            position: [f32::INFINITY, 0., 0.],
        },
        Aim::Point {
            position: [10001., 0., 0.],
        },
    ] {
        assert!(
            e.execute(
                Command::AimAbility {
                    actor: "player".into(),
                    slot: 1,
                    aim
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    for command in [
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("target".into()),
        },
        Command::AimAbility {
            actor: "player".into(),
            slot: 2,
            aim: direction(),
        },
        Command::AimAbility {
            actor: "player".into(),
            slot: 0,
            aim: direction(),
        },
    ] {
        assert!(e.execute(command.into()).is_err());
        assert_eq!(e.snapshot(), before);
    }
}
#[test]
fn coincident_point_is_runtime_failure_without_debit() {
    let mut e = Engine::new(scene()).unwrap();
    aim(
        &mut e,
        "player",
        Aim::Point {
            position: [-4., 0.8, 0.],
        },
    );
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::InvalidAim);
    assert_eq!(energy(&e, "player"), 1000);
    replay(&e);
}
#[test]
fn physical_cover_blocks_free_fire_without_visual_acquisition() {
    let mut s = scene();
    barrier(&mut s, "cover", 0., false, true);
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, "player", direction());
    e.advance(90).unwrap();
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
        Impact::Blocked
    );
    replay(&e);
}
#[test]
fn missing_impact_component_does_not_prevent_launch_or_refund_it() {
    let mut s = scene();
    entity(&mut s, "target").vitality = None;
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, "player", direction());
    e.advance(90).unwrap();
    assert_eq!(energy(&e, "player"), 995);
    assert_eq!(outcome(&e), Outcome::MissingVitality);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    replay(&e);
}
#[test]
fn spatial_definition_requires_projectile_version_and_explicit_intent() {
    let s = scene();
    let mut bad = s.clone();
    bad.version = 30;
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.ability_catalog.abilities[0].delivery = Default::default();
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    entity(&mut bad,"target").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":30}}])).unwrap());
    assert!(bad.validate().is_err());
    let mut old = s;
    old.version = 30;
    old.ability_catalog.abilities[0].selection = Selection::Actor;
    assert!(old.validate().is_ok());
}
#[test]
fn npc_explicit_point_does_not_track_unobserved_player() {
    let mut s = scene();
    entity(&mut s, "target").npc.as_mut().unwrap().target = None;
    let mut other = s.clone();
    entity(&mut other, "player").transform.position = [-4., 0.8, 4.];
    let mut a = Engine::new(s).unwrap();
    let mut b = Engine::new(other).unwrap();
    for e in [&mut a, &mut b] {
        aim(
            e,
            "target",
            Aim::Point {
                position: [-4., 0.8, 0.],
            },
        );
        e.advance(11).unwrap();
    }
    assert_eq!(
        a.simulation.abilities.state.projectiles.active,
        b.simulation.abilities.state.projectiles.active
    );
    assert_eq!(
        a.simulation.abilities.state.projectiles.active[0]
            .cast
            .target,
        None
    );
    replay(&a);
    replay(&b);
}
#[test]
fn disabled_npc_and_insufficient_funds_still_refuse_spatial_cast() {
    let mut s = scene();
    entity(&mut s, "target").npc.as_mut().unwrap().enabled = false;
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .initial = 4;
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, "target", direction());
    aim(&mut e, "player", direction());
    e.advance(1).unwrap();
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    assert_eq!(energy(&e, "player"), 4);
    let r = &e.simulation.abilities.state.receipts;
    assert!(r.iter().any(|r| r.outcome == Outcome::ActorUnavailable));
    assert!(r.iter().any(|r| r.outcome == Outcome::InsufficientResource));
    replay(&e);
}
#[test]
fn screen_aim_resolves_shared_world_point_and_replay_does_not_need_viewport() {
    let mut s = scene();
    s.camera.target = [0., 0.8, 0.];
    s.camera.position = [8., 10., 12.];
    let mut e = Engine::new(s).unwrap();
    let result = e
        .execute(
            Command::AimAbilityAt {
                actor: "player".into(),
                slot: 1,
                x: 400.,
                y: 300.,
                width: 800,
                height: 600,
                plane_y: 0.8,
            }
            .into(),
        )
        .unwrap();
    let point: [f32; 3] = serde_json::from_value(result["aim_point"].clone()).unwrap();
    assert!(glam::Vec3::from(point).distance(glam::Vec3::new(0., 0.8, 0.)) < 0.00001);
    replay(&e);
    e.advance(80).unwrap();
    assert_eq!(health(&e, "target"), 80);
    replay(&e);
}
#[test]
fn screen_aim_rejects_bad_viewport_parallel_and_behind_plane_atomically() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::AimAbilityAt {
                actor: "player".into(),
                slot: 1,
                x: 800.,
                y: 300.,
                width: 800,
                height: 600,
                plane_y: 0.8
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut camera = e.scene.camera.clone();
    camera.position = [0., 2., 10.];
    camera.target = [0., 2., 0.];
    assert!(ignis::picking::point_on_plane(&camera, 400., 300., 800, 600, 0.8).is_err());
    camera.target = [0., 0., 0.];
    assert!(ignis::picking::point_on_plane(&camera, 400., 300., 800, 600, 3.).is_err());
}
#[test]
fn pending_aim_cancellation_and_reset_do_not_spend() {
    let mut e = Engine::new(scene()).unwrap();
    aim(&mut e, "player", direction());
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
    e.advance(20).unwrap();
    assert_eq!(energy(&e, "player"), 1000);
    replay(&e);
    aim(&mut e, "player", direction());
    e.execute(Command::Reset.into()).unwrap();
    e.advance(20).unwrap();
    assert_eq!(energy(&e, "player"), 1000);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
}

#[test]
fn point_becoming_coincident_at_commit_fails_without_cost_or_cooldown() {
    let input = ignis::physics::Input {
        x: 0.,
        z: 1.,
        ..Default::default()
    };
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 30;
    let mut probe = Engine::new(s.clone()).unwrap();
    probe.execute(Command::SetInput { input }.into()).unwrap();
    probe.advance(30).unwrap();
    let point = probe.simulation.position("player").unwrap();
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, "player", Aim::Point { position: point });
    e.execute(Command::SetInput { input }.into()).unwrap();
    e.advance(31).unwrap();
    assert_eq!(outcome(&e), Outcome::InvalidAim);
    assert_eq!(energy(&e, "player"), 1000);
    assert_eq!(
        e.simulation.abilities.state.ready_at[slot(&e, "player")][0],
        0
    );
    replay(&e);
}
#[test]
fn spatial_cast_respects_movement_interruption() {
    let mut s = scene();
    s.ability_catalog.abilities[0].movement = ignis::abilities::CastMovement::Interrupt {
        speed_tolerance: 0.1,
    };
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, "player", direction());
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
    e.advance(20).unwrap();
    assert_eq!(outcome(&e), Outcome::InterruptedByMovement);
    assert_eq!(energy(&e, "player"), 1000);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    replay(&e);
}
