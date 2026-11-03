//! Removal/reuse is an internal seam until the population transaction is public.
use crate::{
    abilities::{Effect, Recipient},
    engine::{Command, Engine},
    model::{RoomRecipe, Scene},
};
use serde_json::json;
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        width: 24.,
        depth: 20.,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [-4., 0.8, 0.];
    p.footsteps = None;
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"target","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[2,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","surface":"floor","speed":3,"stop_distance":0.5,"enabled":false}})).unwrap());
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"attributes":{"buffs":[{"id":"power","duration_ticks":12,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_dealt.multiplier","operation":{"kind":"multiply","basis_points":20000}},{"attribute":"effect.dispellable","operation":{"kind":"set","value":{"kind":"bool","value":false}}}]}]},"abilities":[
        {"id":"bolt","selection":"actor","range":20,"delivery":{"kind":"projectile","speed":12,"half_extent":0.1,"max_distance":20},"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":10}]},
        {"id":"dot","selection":"actor","range":20,"periodic":{"interval_ticks":5,"pulses":3,"reapply":"independent"},"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":10}]},
        {"id":"power","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"apply_buff","recipient":"target","buff":"power"}]}
    ]})).unwrap();
    for id in ["player", "target"] {
        let e = s.entities.iter_mut().find(|e| e.id == id).unwrap();
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["bolt","dot","power"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":1000,"maximum":1000,"scale":1})).unwrap());
    }
    s
}
fn actor(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap()
}
fn cast(e: &mut Engine, slot: u8) {
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot,
            target: if slot == 3 {
                None
            } else {
                Some("target".into())
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
}
fn hp(e: &Engine) -> i64 {
    e.simulation.vitality.state.actors[actor(e, "target")]
        .unwrap()
        .units
}
fn reserved(e: &Engine) -> usize {
    e.simulation
        .abilities
        .state
        .origins
        .slots()
        .iter()
        .filter(|s| s.reference.is_some())
        .count()
}
#[test]
fn detached_periodic_keeps_source_contributions_and_original_deadlines() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, 3);
    cast(&mut e, 2);
    let source = e.simulation.actors.reference(actor(&e, "player") as u16);
    e.simulation.remove_for_test("player");
    assert_eq!(reserved(&e), 1);
    assert!(e.simulation.abilities.state.origins.slots()[0].detached);
    e.advance(5).unwrap();
    assert_eq!(hp(&e), 980);
    let v = e.execute(Command::InspectAbilities.into()).unwrap();
    assert_eq!(v["periodic"]["active"][0]["dispellable"], false);
    e.advance(5).unwrap();
    assert_eq!(hp(&e), 960);
    e.advance(5).unwrap();
    assert_eq!(hp(&e), 950);
    assert_eq!(reserved(&e), 0);
    let r = e.simulation.abilities.state.receipts.back().unwrap();
    assert_eq!(r.actor_instance, source);
    assert!(!e.simulation.abilities.state.receipts.is_empty());
}
#[test]
fn launched_projectile_uses_retained_origin_and_releases_context() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 100;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, 1);
    cast(&mut e, 3);
    e.simulation.remove_for_test("player");
    e.advance(40).unwrap();
    assert_eq!(hp(&e), 980);
    assert_eq!(reserved(&e), 0);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .projectiles
            .receipts
            .back()
            .unwrap()
            .outcome,
        crate::projectiles::Outcome::Hit
    );
}
#[test]
fn expired_retained_buff_does_not_boost_late_impact() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, 3);
    cast(&mut e, 1);
    e.simulation.remove_for_test("player");
    e.advance(40).unwrap();
    assert_eq!(hp(&e), 990);
}
#[test]
fn absent_source_mutation_rolls_back_whole_delivery_without_refund() {
    for effect in [
        Effect::Resource {
            recipient: Recipient::Source,
            resource: "energy".into(),
            delta: 10,
        },
        Effect::Noise { radius: 5. },
    ] {
        let mut s = scene();
        s.ability_catalog.abilities[0].effects.push(effect);
        // Authored projectiles currently forbid source effects. Exercise the runtime
        // guard independently so extending delivery composition cannot bypass it.
        let mut e = Engine::new(scene()).unwrap();
        e.simulation = crate::physics::Simulation::new(&s);
        e.scene = s;
        cast(&mut e, 1);
        let a = actor(&e, "player");
        e.simulation.remove_for_test("player");
        e.advance(40).unwrap();
        assert_eq!(hp(&e), 1000);
        assert_eq!(e.simulation.abilities.state.balances[a][0], 995);
        assert!(
            e.simulation
                .abilities
                .state
                .receipts
                .iter()
                .any(|r| r.outcome == crate::abilities::Outcome::SourceUnavailable)
        );
        assert_eq!(reserved(&e), 0);
    }
}
#[test]
fn world_absence_preserves_authorship_and_reset_recreates_runtime() {
    let mut e = Engine::new(scene()).unwrap();
    let authored = e.scene.clone();
    let count = e.simulation.positions().len();
    e.simulation.remove_for_test("target");
    e.advance(2).unwrap();
    assert_eq!(e.scene, authored);
    assert_eq!(e.simulation.positions().len(), count - 1);
    assert_eq!(e.simulation.position("target"), None);
    let v = e.execute(Command::InspectWorld.into()).unwrap();
    assert!(v["positions"].get("target").is_none());
    assert!(
        e.execute(
            Command::EmitNoise {
                origin: crate::noise::NoiseOrigin::Entity {
                    id: "target".into()
                },
                radius: 3.
            }
            .into()
        )
        .is_err()
    );
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.simulation.position("target").is_some());
}
#[test]
fn several_deliveries_share_one_context_and_last_delivery_releases_it() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, 1);
    cast(&mut e, 2);
    cast(&mut e, 2);
    assert_eq!(reserved(&e), 1);
    e.simulation.remove_for_test("player");
    e.advance(16).unwrap();
    assert_eq!(reserved(&e), 1);
    e.advance(30).unwrap();
    assert_eq!(reserved(&e), 0);
}
#[test]
fn source_snapshot_buffers_are_reserved_before_removal() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, 3);
    cast(&mut e, 2);
    let old = &e.simulation.abilities.state.origins.slots()[0].attributes;
    let pointers = (old.bases.as_ptr(), old.contributions.as_ptr());
    e.simulation.remove_for_test("player");
    let new = &e.simulation.abilities.state.origins.slots()[0].attributes;
    assert_eq!(pointers, (new.bases.as_ptr(), new.contributions.as_ptr()));
    assert_eq!(new.contributions.len(), 1);
}

#[test]
fn reused_source_slot_does_not_change_old_effect_or_definition() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 100;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, 3);
    cast(&mut e, 2);
    let a = actor(&e, "player");
    let entity = e.simulation.actors.entity(a as u16) as u16;
    let old = e.simulation.actors.reference(a as u16);
    e.simulation.remove_for_test("player");
    let next = e.simulation.actors.recycle_for_test(entity);
    assert_eq!(old.slot, next.slot);
    assert_ne!(old, next);
    e.simulation.abilities.state.buffs.active.clear();
    e.simulation.vitality.state.actors[a] = Some(crate::vitality::Health {
        units: 1,
        maximum: 1,
        scale: 1,
    });
    e.simulation.vitality.state.lives[a] = 999;
    e.advance(5).unwrap();
    assert_eq!(hp(&e), 980);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .actor_instance,
        old
    );
    assert_eq!(e.simulation.position("player"), None);
}
#[test]
fn fixed_area_survives_absence_but_source_anchor_ends() {
    use crate::{
        abilities::{Aim, Selection},
        periodic::Reapply,
        persistent_area::{Anchor, Schedule},
    };
    for anchor in [Anchor::Fixed, Anchor::Source] {
        let mut s = scene();
        let d = &mut s.ability_catalog.abilities[0];
        d.selection = if anchor == Anchor::Source {
            Selection::SelfActor
        } else {
            Selection::Spatial
        };
        d.range = if anchor == Anchor::Source { 0. } else { 20. };
        d.delivery = Default::default();
        d.area = Some(crate::area::Area {
            radius: 10.,
            max_targets: 9,
            include_source: false,
            occlusion: false,
            persistence: Some(Schedule {
                anchor,
                duration_ticks: 20,
                interval_ticks: 5,
                apply_on_spawn: false,
                reapply: Reapply::Refresh,
                end_on_source_death: true,
            }),
        });
        let mut e = Engine::new(s).unwrap();
        if anchor == Anchor::Fixed {
            e.execute(
                Command::AimAbility {
                    actor: "player".into(),
                    slot: 1,
                    aim: Aim::Point {
                        position: [2., 0.8, 0.],
                    },
                }
                .into(),
            )
            .unwrap();
        } else {
            e.execute(
                Command::ActivateAbility {
                    actor: "player".into(),
                    slot: 1,
                    target: None,
                }
                .into(),
            )
            .unwrap();
        }
        e.advance(1).unwrap();
        e.simulation.remove_for_test("player");
        e.advance(5).unwrap();
        assert_eq!(hp(&e), if anchor == Anchor::Fixed { 990 } else { 1000 });
        assert_eq!(reserved(&e), usize::from(anchor == Anchor::Fixed));
        e.advance(20).unwrap();
        assert_eq!(reserved(&e), 0);
    }
}
#[test]
fn removed_cover_is_not_a_projectile_or_render_ghost() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"cover","name":"cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[0,0.8,0],"size":[0.2,2,3]}})).unwrap());
    // Allow launch through cover so collision independently verifies removal.
    s.ability_catalog.abilities[0].selection = crate::abilities::Selection::Spatial;
    s.ability_catalog.abilities[0].range = 0.;
    let mut e = Engine::new(s).unwrap();
    let n = e.render_frame().boxes.len();
    e.simulation.remove_for_test("cover");
    assert_eq!(e.render_frame().boxes.len(), n - 1);
    e.execute(
        Command::AimAbility {
            actor: "player".into(),
            slot: 1,
            aim: crate::abilities::Aim::Direction {
                vector: [1., 0., 0.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(40).unwrap();
    assert_eq!(hp(&e), 990);
}
#[test]
fn full_context_pool_rejects_admission_before_cost_and_cooldown() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e, 2);
    let template = e.simulation.abilities.state.periodic.active[0];
    e.simulation.abilities.state.periodic.active.clear();
    e.simulation.abilities.state.origins.collect([]);
    for i in 0..crate::origins::CAPACITY {
        let mut r = template.cast.life.source_instance;
        r.instance += 100 + i as u64;
        let lease = e
            .simulation
            .abilities
            .state
            .origins
            .reserve(template.actor, r)
            .unwrap();
        e.simulation
            .abilities
            .state
            .origins
            .source_mut(r)
            .unwrap()
            .detached = true;
        let mut effect = template;
        effect.next_tick = 10000;
        effect.cast.origin = Some(lease);
        effect.cast.life.source_instance = r;
        e.simulation.abilities.state.periodic.active.push(effect);
    }
    let a = actor(&e, "player");
    let before = e.simulation.abilities.state.balances[a];
    let ready = e.simulation.abilities.state.ready_at[a];
    cast(&mut e, 1);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        crate::abilities::Outcome::OriginBudget
    );
    assert_eq!(e.simulation.abilities.state.balances[a], before);
    assert_eq!(e.simulation.abilities.state.ready_at[a], ready);
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
}
#[test]
fn origin_pool_reuses_only_unreferenced_slots_and_rejects_old_lease() {
    let mut p = crate::origins::State::default();
    let r = crate::identity::Reference {
        slot: 0,
        generation: 1,
        instance: 1,
    };
    assert_eq!(p.reserve(0, r), Some(0));
    assert_eq!(p.reserve(0, r), Some(0));
    p.collect([(0, r)]);
    assert!(p.get(Some(0), r).is_some());
    p.collect([]);
    let next = crate::identity::Reference { instance: 2, ..r };
    assert_eq!(p.reserve(0, next), Some(0));
    assert!(p.get(Some(0), r).is_none());
    eprintln!(
        "origin layout: slot={} instance={} base={} upfront_payload={}",
        std::mem::size_of::<crate::origins::Slot>(),
        std::mem::size_of::<crate::attributes::Instance>(),
        std::mem::size_of::<(u8, crate::attributes::Value)>(),
        crate::origins::CAPACITY
            * (std::mem::size_of::<crate::origins::Slot>()
                + 64 * std::mem::size_of::<(u8, crate::attributes::Value)>()
                + 64 * std::mem::size_of::<crate::attributes::Instance>())
    );
}

#[test]
fn removed_observer_and_subject_do_not_leave_live_guides_or_observations() {
    let s: Scene = serde_json::from_str(include_str!("../examples/npc-radio.scene.json")).unwrap();
    let mut e = Engine::new(s).unwrap();
    e.debug_view.npc_routes = true;
    e.debug_view.npc_vision = true;
    e.debug_view.npc_hearing = true;
    e.advance(13).unwrap();
    assert!(e.simulation.npcs["sender"].memory.is_some());
    e.simulation.remove_for_test("player");
    e.advance(2).unwrap();
    assert!(!e.simulation.npcs["sender"].observed_this_tick);
    e.simulation.remove_for_test("sender");
    e.advance(2).unwrap();
    let _ = e.render_frame();
    let v = e.execute(Command::Inspect.into()).unwrap();
    assert!(v["runtime_positions"].get("sender").is_none());
    assert!(
        e.execute(
            Command::SelectEntity {
                id: Some("sender".into())
            }
            .into()
        )
        .is_err()
    );
    assert!(
        e.execute(
            Command::ActivateAbility {
                actor: "sender".into(),
                slot: 1,
                target: None
            }
            .into()
        )
        .is_err()
    );
}
#[test]
fn source_removal_cancels_uncommitted_cast_without_payment() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 100;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, 1);
    let a = actor(&e, "player");
    assert!(e.simulation.abilities.state.casts[a].is_some());
    e.simulation.remove_for_test("player");
    e.advance(110).unwrap();
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    assert_eq!(e.simulation.abilities.state.balances[a][0], 1000);
    assert_eq!(reserved(&e), 0);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == crate::abilities::Outcome::SourceUnavailable)
    );
}

#[test]
fn retained_dispel_protection_expires_and_does_not_charge_failed_dispel() {
    let mut s = scene();
    s.ability_catalog.abilities[1].periodic_category = Some("curse".into());
    s.ability_catalog.abilities.push(serde_json::from_value(json!({"id":"clean","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"dispel","recipient":"target","category":"curse","max_instances":1,"require_match":true}]})).unwrap());
    s.entities
        .iter_mut()
        .find(|e| e.id == "target")
        .unwrap()
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("clean".into());
    let n = s
        .entities
        .iter_mut()
        .find(|e| e.id == "target")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    n.enabled = true;
    n.objectives = Some(
        serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"wait"}}]))
            .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, 3);
    cast(&mut e, 2);
    e.simulation.remove_for_test("player");
    e.advance(5).unwrap();
    let target = actor(&e, "target");
    let purge = |e: &mut Engine| {
        e.execute(
            Command::ActivateAbility {
                actor: "target".into(),
                slot: 4,
                target: None,
            }
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
    };
    purge(&mut e);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        crate::abilities::Outcome::NoMatchingEffect
    );
    assert_eq!(e.simulation.abilities.state.balances[target][0], 1000);
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 1);
    e.advance(5).unwrap();
    purge(&mut e);
    assert_eq!(e.simulation.abilities.state.balances[target][0], 995);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    assert_eq!(reserved(&e), 0);
}
#[test]
fn detached_lethal_projectile_preserves_original_source_in_death_receipt() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "target")
        .unwrap()
        .vitality
        .as_mut()
        .unwrap()
        .initial = 10;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, 1);
    let source = e.simulation.actors.reference(actor(&e, "player") as u16);
    e.simulation.remove_for_test("player");
    e.advance(40).unwrap();
    assert_eq!(hp(&e), 0);
    assert_eq!(
        e.simulation.vitality.state.deaths[0].source_instance,
        source
    );
    let v = e.execute(Command::InspectVitality.into()).unwrap();
    assert!(v["deaths"][0]["source"].is_null());
    assert_eq!(v["deaths"][0]["source_instance"], source.id());
}
