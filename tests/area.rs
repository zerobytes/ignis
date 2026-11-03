use ignis::{
    abilities::{Aim, Outcome, Selection},
    area::{Area, Status},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::{Value, json};
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
    for (id, x) in [("a", 0.), ("b", 2.)] {
        s.entities.push(serde_json::from_value(json!({"id":id,"name":id,"body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[x,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    }
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"attributes":{"tags":["fire","ice"],"buffs":[{"id":"resist","duration_ticks":600,"reapply":"refresh","category":"protection","modifiers":[{"attribute":"combat.damage_received.multiplier","tags_all":["ice"],"operation":{"kind":"multiply","basis_points":5000}}]}]},"abilities":[
        {"id":"area","selection":"spatial","range":12,"area":{"radius":3,"max_targets":9,"include_source":false,"occlusion":false},"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage_components","recipient":"target","components":[{"amount":40,"tags":["fire"]},{"amount":60,"tags":["ice"]}]}]},
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":1000}]}
    ]})).unwrap();
    for id in ["player", "a", "b"] {
        let e = entity(&mut s, id);
        e.footsteps = None;
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["area","kill"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":1000,"maximum":1000,"scale":1})).unwrap());
    }
    entity(&mut s, "player").transform.position = [-4., 0.8, 0.];
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
fn energy(e: &Engine) -> i64 {
    e.simulation.abilities.state.balances[slot(e, "player")][0]
}
fn aim(e: &mut Engine, p: [f32; 3]) {
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 1,
            aim: Aim::Point { position: p },
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
}
fn cast(e: &mut Engine) {
    aim(e, [0., 0.8, 0.]);
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn status(e: &Engine, id: &str) -> Status {
    e.simulation
        .abilities
        .state
        .areas
        .receipts
        .back()
        .unwrap()
        .candidates
        .iter()
        .flatten()
        .find(|c| c.actor as usize == slot(e, id))
        .unwrap()
        .status
}
fn effects(s: &mut Scene, v: Value) {
    s.ability_catalog.abilities[0].effects = serde_json::from_value(v).unwrap();
}
fn wall(s: &mut Scene, p: [f32; 3], size: [f32; 3]) {
    s.entities.push(serde_json::from_value(json!({"id":"cover","name":"cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":p,"size":size}})).unwrap());
}
#[test]
fn sphere_uses_xyz_and_component_resistance_and_one_cost() {
    let mut s = scene();
    s.ability_catalog.attributes.bases=serde_json::from_value(json!([{"entity":"b","attribute":"combat.damage_received.multiplier","value":{"kind":"number","units":5000}}])).unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!(
        (hp(&e, "a"), hp(&e, "b"), hp(&e, "player"), energy(&e)),
        (900, 950, 1000, 995)
    );
    let r = e.execute(Command::InspectAbilities.into()).unwrap();
    assert_eq!(
        r["receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| !r["costs"].as_array().unwrap().is_empty())
            .count(),
        1
    );
    assert_eq!(
        r["receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["outcome"] == "area_applied")
            .count(),
        2
    );
    replay(&e);
    let mut s = scene();
    entity(&mut s, "b").transform.position = [0., 4.8, 0.];
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!(hp(&e, "b"), 1000);
    assert_eq!(status(&e, "b"), Status::Outside);
    replay(&e);
}
#[test]
fn exact_radius_and_cap_use_distance_then_id_independent_of_authoring_order() {
    for reverse in [false, true] {
        let mut s = scene();
        entity(&mut s, "a").transform.position = [0., 0.8, -3.];
        entity(&mut s, "b").transform.position = [0., 0.8, 3.];
        s.ability_catalog.abilities[0]
            .area
            .as_mut()
            .unwrap()
            .max_targets = 1;
        if reverse {
            s.entities.reverse();
        }
        let mut e = Engine::new(s).unwrap();
        cast(&mut e);
        assert_eq!(hp(&e, "a"), 900);
        assert_eq!(hp(&e, "b"), 1000);
        assert_eq!(status(&e, "b"), Status::TargetLimit);
        replay(&e);
    }
}
#[test]
fn occlusion_uses_solid_geometry_independent_of_render_or_vision_flags() {
    for occlusion in [false, true] {
        let mut s = scene();
        wall(&mut s, [1., 1., 0.], [0.2, 2., 4.]);
        entity(&mut s, "cover").visible = Some(false);
        entity(&mut s, "cover").blocks_vision = Some(false);
        s.ability_catalog.abilities[0]
            .area
            .as_mut()
            .unwrap()
            .occlusion = occlusion;
        let mut e = Engine::new(s).unwrap();
        cast(&mut e);
        assert_eq!(hp(&e, "a"), 900);
        assert_eq!(hp(&e, "b"), if occlusion { 1000 } else { 900 });
        replay(&e);
    }
}
#[test]
fn point_range_revalidates_before_payment_but_radial_occlusion_is_not_placement() {
    let mut s = scene();
    wall(&mut s, [-2., 1., 0.], [0.2, 2., 4.]);
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .occlusion = true;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!(energy(&e), 995);
    assert_eq!(hp(&e, "a"), 900);
    aim(&mut e, [20., 0.8, 0.]);
    assert_eq!(energy(&e), 995);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::OutOfRange
    );
    replay(&e);
}
#[test]
fn empty_area_consumes_once_and_point_can_equal_source() {
    let mut e = Engine::new(scene()).unwrap();
    aim(&mut e, [-4., 0.8, 0.]);
    assert_eq!(energy(&e), 995);
    assert_eq!(hp(&e, "player"), 1000);
    assert_eq!(hp(&e, "a"), 1000);
    replay(&e);
}
#[test]
fn source_inclusion_is_explicit_and_relations_only_filter_other_actors() {
    for include_source in [false, true] {
        let mut s = scene();
        let d = &mut s.ability_catalog.abilities[0];
        d.selection = Selection::SelfActor;
        d.relation = Some(ignis::targeting::Attitude::Hostile);
        d.area = Some(Area {
            persistence: None,
            radius: 8.,
            max_targets: 9,
            include_source,
            occlusion: false,
        });
        entity(&mut s, "player").faction = Some("player".into());
        entity(&mut s, "a").faction = Some("enemy".into());
        s.faction_relations =
            serde_json::from_value(json!([{"from":"player","to":"enemy","attitude":"hostile"}]))
                .unwrap();
        let mut e = Engine::new(s).unwrap();
        e.execute(
            Command::ActivateAbility {
                actor: "player".into(),
                slot: 1,
                target: None,
            }
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
        assert_eq!(hp(&e, "a"), 900);
        assert_eq!(hp(&e, "b"), 1000);
        assert_eq!(hp(&e, "player"), if include_source { 900 } else { 1000 });
        replay(&e);
    }
}
#[test]
fn failed_recipient_rolls_back_all_its_effects_but_not_cost_or_other_targets() {
    let mut s = scene();
    entity(&mut s, "a").abilities = None;
    s.ability_catalog.abilities[0].effects.push(
        serde_json::from_value(
            json!({"kind":"resource","recipient":"target","resource":"energy","delta":-1}),
        )
        .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!((hp(&e, "a"), hp(&e, "b"), energy(&e)), (1000, 900, 995));
    assert_eq!(status(&e, "a"), Status::EffectFailed);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .areas
            .receipts
            .back()
            .unwrap()
            .candidates[0]
            .unwrap()
            .failure,
        Some(Outcome::MissingResource)
    );
    replay(&e);
}
#[test]
fn failed_slot_is_not_replaced_and_dead_actors_do_not_consume_cap() {
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .max_targets = 1;
    entity(&mut s, "a").vitality = None;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!(hp(&e, "b"), 1000);
    assert_eq!(status(&e, "a"), Status::EffectFailed);
    assert_eq!(status(&e, "b"), Status::TargetLimit);
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .max_targets = 1;
    let mut e = Engine::new(s).unwrap();
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: Some("a".into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    cast(&mut e);
    assert_eq!(status(&e, "a"), Status::Dead);
    assert_eq!(hp(&e, "b"), 900);
    replay(&e);
}
#[test]
fn cast_point_frozen_recipient_positions_selected_at_commit_and_cancel_is_free() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 10;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!(hp(&e, "a"), 1000);
    assert_eq!(energy(&e), 1000);
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: e.simulation.abilities.state.casts[slot(&e, "player")]
                .unwrap()
                .id,
        }
        .into(),
    )
    .unwrap();
    e.advance(20).unwrap();
    assert_eq!(energy(&e), 1000);
    cast(&mut e);
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "a"), 900);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .areas
            .receipts
            .back()
            .unwrap()
            .center,
        [0., 0.8, 0.]
    );
    replay(&e);
}
#[test]
fn projectile_explodes_on_wall_and_respects_occlusion_without_refund() {
    for occlusion in [false, true] {
        let mut s = scene();
        wall(&mut s, [0., 1., 0.], [0.2, 2., 4.]);
        entity(&mut s, "a").transform.position = [-0.8, 0.8, 1.];
        entity(&mut s, "b").transform.position = [0.8, 0.8, 1.];
        let d = &mut s.ability_catalog.abilities[0];
        d.range = 0.;
        d.area.as_mut().unwrap().occlusion = occlusion;
        d.delivery = serde_json::from_value(
            json!({"kind":"projectile","speed":10,"half_extent":0.08,"max_distance":20}),
        )
        .unwrap();
        let mut e = Engine::new(s).unwrap();
        cast(&mut e);
        assert_eq!(energy(&e), 995);
        assert_eq!(hp(&e, "a"), 1000);
        e.advance(30).unwrap();
        assert_eq!(hp(&e, "a"), 900);
        assert_eq!(hp(&e, "b"), if occlusion { 1000 } else { 900 });
        assert!(
            e.simulation
                .abilities
                .state
                .areas
                .receipts
                .back()
                .unwrap()
                .center[0]
                < 0.
        );
        replay(&e);
    }
}
#[test]
fn projectile_expiration_without_contact_does_not_create_area() {
    let mut s = scene();
    let d = &mut s.ability_catalog.abilities[0];
    d.range = 0.;
    d.delivery = serde_json::from_value(
        json!({"kind":"projectile","speed":10,"half_extent":0.08,"max_distance":1}),
    )
    .unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    e.advance(20).unwrap();
    assert_eq!(energy(&e), 995);
    assert!(e.simulation.abilities.state.areas.receipts.is_empty());
    replay(&e);
}
#[test]
fn buffs_dispel_and_healing_reuse_recipient_transactions() {
    let mut s = scene();
    effects(
        &mut s,
        json!([{"kind":"apply_buff","recipient":"target","buff":"resist"}]),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 2);
    replay(&e);
    let mut s = e.scene.clone();
    let mut dispel = s.ability_catalog.abilities[0].clone();
    dispel.id = "clean".into();
    dispel.effects=serde_json::from_value(json!([{"kind":"dispel","recipient":"target","category":"protection","max_instances":1,"require_match":true}])).unwrap();
    s.ability_catalog.abilities.push(dispel);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("clean".into());
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 3,
            aim: Aim::Point {
                position: [0., 0.8, 0.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    replay(&e);
    let mut s = scene();
    entity(&mut s, "a").vitality.as_mut().unwrap().initial = 800;
    entity(&mut s, "b").vitality.as_mut().unwrap().initial = 950;
    effects(
        &mut s,
        json!([{"kind":"heal","recipient":"target","amount":100}]),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!((hp(&e, "a"), hp(&e, "b")), (900, 1000));
    replay(&e);
}
#[test]
fn schema_rejects_unsupported_combinations_and_direction_without_mutation() {
    let base = serde_json::to_value(scene()).unwrap();
    for (key, value) in [
        ("radius", json!(0)),
        ("radius", json!(101)),
        ("max_targets", json!(0)),
        ("max_targets", json!(1025)),
    ] {
        let mut v = base.clone();
        v["ability_catalog"]["abilities"][0]["area"][key] = value;
        assert!(
            serde_json::from_value::<Scene>(v)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    for (key, value) in [
        (
            "channel",
            json!({"interval_ticks":1,"pulses":2,"cost_mode":"on_start"}),
        ),
        (
            "periodic",
            json!({"interval_ticks":1,"pulses":2,"reapply":"refresh"}),
        ),
        (
            "effects",
            json!([{"kind":"damage","recipient":"source","amount":1}]),
        ),
        ("effects", json!([{"kind":"noise","radius":2}])),
    ] {
        let mut v = base.clone();
        v["ability_catalog"]["abilities"][0][key] = value;
        assert!(
            serde_json::from_value::<Scene>(v)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    let mut s = scene();
    s.version = 40;
    assert!(s.validate().is_err());
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::AimAbility {
                actor: "player".into(),
                slot: 1,
                aim: Aim::Direction {
                    vector: [1., 0., 0.]
                }
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
#[test]
fn ring_is_bounded_and_guides_expire_without_changing_gameplay() {
    let mut e = Engine::new(scene()).unwrap();
    for _ in 0..70 {
        aim(&mut e, [-4., 0.8, 0.]);
    }
    assert_eq!(e.simulation.abilities.state.areas.receipts.len(), 64);
    assert_eq!(e.simulation.abilities.state.areas.dropped_receipts, 6);
    e.debug_view.npc_routes = true;
    let before = e.snapshot();
    let frame = e.render_frame();
    assert!(frame.route_guides.len() >= 8 * 96);
    assert_eq!(e.snapshot(), before);
    e.advance(61).unwrap();
    assert!(e.render_frame().route_guides.len() < frame.route_guides.len());
    replay(&e);
}
#[test]
fn direct_actor_center_preserves_existing_anchor_evidence_rules() {
    let mut s = scene();
    s.ability_catalog.abilities[0].selection = Selection::Actor;
    let mut e = Engine::new(s).unwrap();
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("a".into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!((hp(&e, "a"), hp(&e, "b")), (900, 900));
    replay(&e);
}

#[test]
fn independent_buff_capacity_failure_is_atomic_per_recipient() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].reapply =
        serde_json::from_value(json!("independent")).unwrap();
    effects(
        &mut s,
        json!([{"kind":"damage","recipient":"target","amount":1},{"kind":"apply_buff","recipient":"target","buff":"resist"}]),
    );
    let mut e = Engine::new(s).unwrap();
    for _ in 0..32 {
        cast(&mut e);
    }
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 64);
    assert_eq!(hp(&e, "a"), 968);
    cast(&mut e);
    assert_eq!((hp(&e, "a"), hp(&e, "b"), energy(&e)), (968, 968, 835));
    for c in e
        .simulation
        .abilities
        .state
        .areas
        .receipts
        .back()
        .unwrap()
        .candidates
        .iter()
        .flatten()
        .filter(|c| c.actor as usize != slot(&e, "player"))
    {
        assert_eq!(c.failure, Some(Outcome::BuffBudget));
    }
    replay(&e);
}
#[test]
fn npc_uses_observed_frozen_point_without_learning_secondary_targets() {
    let mut s = scene();
    entity(&mut s, "b").transform.position = [-3., 0.8, -1.];
    s.ability_catalog.abilities[0].cast_ticks = 60;
    s.ability_catalog.abilities[0].cooldown_ticks = 300;
    entity(&mut s,"a").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([
        {"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":15,"policy":{"spatial_aim":{"mode":"observed_point","range":12}}}},
        {"priority":0,"when":"always","action":{"kind":"wait"}}
    ])).unwrap());
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(62).unwrap();
    assert_eq!((hp(&e, "player"), hp(&e, "b")), (900, 900));
    assert_eq!(
        e.simulation.npcs["a"].memory.unwrap().actor_index,
        Some(slot(&e, "player") as u16)
    );
    replay(&e);
    let mut e = Engine::new(s).unwrap();
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
    e.advance(61).unwrap();
    assert_eq!(hp(&e, "player"), 1000);
    assert_eq!(hp(&e, "b"), 900);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .areas
            .receipts
            .back()
            .unwrap()
            .center,
        [-4., 0.8, 0.]
    );
    replay(&e);
}
#[test]
fn source_death_changes_later_recipient_context_but_does_not_cancel_committed_area() {
    let mut s = scene();
    entity(&mut s, "player").transform.position = [-1., 0.8, 0.];
    entity(&mut s, "player").vitality.as_mut().unwrap().initial = 50;
    let d = &mut s.ability_catalog.abilities[0];
    d.selection = Selection::SelfActor;
    d.area = Some(Area {
        persistence: None,
        radius: 5.,
        max_targets: 9,
        include_source: true,
        occlusion: false,
    });
    let mut boost = s.ability_catalog.abilities[1].clone();
    boost.id = "power".into();
    boost.selection = Selection::SelfActor;
    boost.effects =
        serde_json::from_value(json!([{"kind":"apply_buff","recipient":"target","buff":"resist"}]))
            .unwrap();
    s.ability_catalog.abilities.push(boost);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("power".into());
    s.ability_catalog.attributes.buffs[0].modifiers=serde_json::from_value(json!([{"attribute":"combat.damage_dealt.multiplier","operation":{"kind":"multiply","basis_points":20000}}])).unwrap();
    let mut e = Engine::new(s).unwrap();
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 3,
            target: None,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: None,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!((hp(&e, "player"), hp(&e, "a"), hp(&e, "b")), (0, 900, 900));
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    replay(&e);
}
#[test]
fn area_interrupts_multiple_casts_and_protected_recipient_fails_independently() {
    let mut s = scene();
    effects(
        &mut s,
        json!([{"kind":"interrupt","recipient":"target","require_active":true}]),
    );
    entity(&mut s, "b").transform.position = [2., 0.8, -1.5];
    s.ability_catalog.abilities[1].cast_ticks = 120;
    let mut protected = s.ability_catalog.abilities[1].clone();
    protected.id = "protected".into();
    protected.interruptible = false;
    s.ability_catalog.abilities.push(protected);
    entity(&mut s, "b")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("protected".into());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    for (actor, slot) in [("a", 2), ("b", 3)] {
        e.execute(
            Command::ActivateAbility {
                actor: actor.into(),
                slot,
                target: Some("player".into()),
            }
            .into(),
        )
        .unwrap();
    }
    e.advance(1).unwrap();
    assert!(e.simulation.abilities.state.casts[slot(&e, "a")].is_some());
    assert!(e.simulation.abilities.state.casts[slot(&e, "b")].is_some());
    cast(&mut e);
    assert!(e.simulation.abilities.state.casts[slot(&e, "a")].is_none());
    assert!(e.simulation.abilities.state.casts[slot(&e, "b")].is_some());
    assert_eq!(status(&e, "a"), Status::Applied);
    assert_eq!(status(&e, "b"), Status::EffectFailed);
    assert_eq!(energy(&e), 995);
    replay(&e);
}
#[test]
fn npc_direct_area_policy_cannot_request_more_range_than_placement_allows() {
    let mut s = scene();
    entity(&mut s,"a").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([
        {"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":15,"policy":{"spatial_aim":{"mode":"observed_point","range":15}}}}
    ])).unwrap());
    assert!(s.validate().is_err());
}

#[test]
fn floor_surface_does_not_occlude_but_point_inside_solid_does() {
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .occlusion = true;
    let mut e = Engine::new(s.clone()).unwrap();
    aim(&mut e, [0., 0., 0.]);
    assert_eq!((hp(&e, "a"), hp(&e, "b")), (900, 900));
    replay(&e);
    wall(&mut s, [0., 0.1, 0.], [1., 0.2, 1.]);
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, [0., 0.1, 0.]);
    assert_eq!(status(&e, "a"), Status::Occluded);
    assert_eq!(status(&e, "b"), Status::Occluded);
    replay(&e);
}
