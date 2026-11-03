use ignis::{
    abilities::{Aim, Outcome, Selection},
    area::Status,
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    periodic::Reapply,
    persistent_area::{Outcome as End, Schedule},
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
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .persistence = Some(Schedule {
        anchor: ignis::persistent_area::Anchor::Fixed,
        duration_ticks: 30,
        interval_ticks: 10,
        apply_on_spawn: false,
        reapply: Reapply::Refresh,
        end_on_source_death: false,
    });
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
fn config(s: &mut Scene) -> &mut Schedule {
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .persistence
        .as_mut()
        .unwrap()
}
fn activate(e: &mut Engine, actor: &str, n: u8, target: &str) {
    e.execute(
        Command::ActivateAbility {
            actor: actor.into(),
            slot: n,
            target: Some(target.into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
}
fn movement(e: &mut Engine, x: f32, z: f32) {
    e.execute(
        Command::SetInput {
            input: ignis::physics::Input {
                x,
                z,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
}
#[test]
fn cadence_expiration_excludes_deadline_single_cost_and_prefix() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e);
    let prefix = e.simulation.abilities.state.clone();
    assert_eq!((hp(&e, "a"), energy(&e)), (1000, 995));
    e.advance(9).unwrap();
    assert_eq!(hp(&e, "a"), 1000);
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "a"), 900);
    e.advance(20).unwrap();
    assert_eq!((hp(&e, "a"), energy(&e)), (800, 995));
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .is_empty()
    );
    assert_eq!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .receipts
            .back()
            .unwrap()
            .outcome,
        End::Expired
    );
    assert_eq!(
        e.simulation
            .abilities
            .state
            .areas
            .receipts
            .iter()
            .map(|r| (r.tick, r.pulse))
            .collect::<Vec<_>>(),
        vec![(10, Some(1)), (20, Some(2))]
    );
    replay(&e);
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
fn initial_pulse_optional_and_no_double_pulse() {
    let mut s = scene();
    config(&mut s).apply_on_spawn = true;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert_eq!(hp(&e, "a"), 900);
    e.advance(9).unwrap();
    assert_eq!(hp(&e, "a"), 900);
    e.advance(21).unwrap();
    assert_eq!(hp(&e, "a"), 700);
    assert_eq!(energy(&e), 995);
    replay(&e);
}
#[test]
fn refresh_relocates_restarts_cadence_and_links_executions() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e);
    let id = e.simulation.abilities.state.persistent_areas.active[0]
        .cast
        .id;
    e.advance(4).unwrap();
    aim(&mut e, [4., 0.8, 0.]);
    let state = &e.simulation.abilities.state.persistent_areas;
    assert_eq!(state.active.len(), 1);
    assert_eq!(state.active[0].next_tick, 15);
    assert_eq!(state.active[0].center, [4., 0.8, 0.]);
    assert!(state.receipts.iter().any(|r| r.execution_id == id
        && r.outcome == End::Refreshed
        && r.replaced_by == Some(state.active[0].cast.id)));
    e.advance(9).unwrap();
    assert_eq!(hp(&e, "b"), 1000);
    e.advance(1).unwrap();
    assert_eq!((hp(&e, "a"), hp(&e, "b"), energy(&e)), (1000, 900, 990));
    replay(&e);
}
#[test]
fn independent_areas_overlap_and_keep_separate_clocks() {
    let mut s = scene();
    config(&mut s).reapply = Reapply::Independent;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    cast(&mut e);
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        2
    );
    e.advance(30).unwrap();
    assert_eq!((hp(&e, "a"), energy(&e)), (600, 990));
    replay(&e);
}
#[test]
fn refresh_identity_includes_source_and_definition_but_not_anchor() {
    let mut s = scene();
    s.ability_catalog.abilities[0].selection = Selection::Actor;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, "a");
    activate(&mut e, "player", 1, "b");
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        1
    );
    activate(&mut e, "a", 1, "player");
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        2
    );
    replay(&e);
}
#[test]
fn source_death_policy_and_posthumous_pulses() {
    for stop in [false, true] {
        let mut s = scene();
        config(&mut s).end_on_source_death = stop;
        let mut e = Engine::new(s).unwrap();
        cast(&mut e);
        activate(&mut e, "a", 2, "player");
        assert_eq!(hp(&e, "player"), 0);
        assert_eq!(
            e.simulation
                .abilities
                .state
                .persistent_areas
                .active
                .is_empty(),
            stop
        );
        e.advance(25).unwrap();
        assert_eq!(hp(&e, "a"), if stop { 1000 } else { 800 });
        if stop {
            assert_eq!(
                e.simulation
                    .abilities
                    .state
                    .persistent_areas
                    .receipts
                    .back()
                    .unwrap()
                    .outcome,
                End::SourceDead
            );
        }
        replay(&e);
    }
}
#[test]
fn source_killed_by_initial_pulse_finishes_pulse_then_removes_volume() {
    let mut s = scene();
    config(&mut s).apply_on_spawn = true;
    config(&mut s).end_on_source_death = true;
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .include_source = true;
    effects(
        &mut s,
        json!([{"kind":"damage","recipient":"target","amount":1000}]),
    );
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, [-2., 0.8, 0.]);
    assert_eq!((hp(&e, "a"), hp(&e, "player")), (0, 0));
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .is_empty()
    );
    replay(&e);
}
#[test]
fn direct_capacity_is_atomic_and_releases_exactly_at_expiration() {
    let mut s = scene();
    config(&mut s).reapply = Reapply::Independent;
    config(&mut s).duration_ticks = 100;
    config(&mut s).interval_ticks = 50;
    let mut e = Engine::new(s).unwrap();
    for _ in 0..16 {
        aim(&mut e, [0., 7., 0.]);
    }
    assert_eq!(energy(&e), 920);
    aim(&mut e, [0., 7., 0.]);
    assert_eq!(energy(&e), 920);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::AreaBudget
    );
    e.advance(83).unwrap();
    assert_eq!(e.simulation.tick, 100);
    aim(&mut e, [0., 7., 0.]);
    assert_eq!(energy(&e), 915);
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        16
    );
    replay(&e);
}
#[test]
fn full_pool_allows_refresh() {
    let mut s = scene();
    config(&mut s).duration_ticks = 1000;
    config(&mut s).interval_ticks = 500;
    let mut other = s.ability_catalog.abilities[0].clone();
    other.id = "other".into();
    other
        .area
        .as_mut()
        .unwrap()
        .persistence
        .as_mut()
        .unwrap()
        .reapply = Reapply::Independent;
    s.ability_catalog.abilities.push(other);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("other".into());
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    for _ in 0..15 {
        e.execute(
            Command::AimAbility {
                actor: "player".into(),
                slot: 3,
                aim: ignis::abilities::Aim::Point {
                    position: [0., 7., 0.],
                },
            }
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
    }
    cast(&mut e);
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        16
    );
    assert_eq!(energy(&e), 915);
    replay(&e);
}
#[test]
fn movement_changes_membership_at_pulses_and_volume_does_not_follow_source() {
    let mut s = scene();
    config(&mut s).duration_ticks = 121;
    config(&mut s).interval_ticks = 10;
    let area = s.ability_catalog.abilities[0].area.as_mut().unwrap();
    area.include_source = true;
    area.radius = 0.7;
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, [-2.5, 0.8, 0.]);
    movement(&mut e, 1., 0.);
    e.advance(38).unwrap();
    let after = hp(&e, "player");
    assert!(after < 1000, "player must enter and receive pulses");
    movement(&mut e, 0., 1.);
    e.advance(50).unwrap();
    let outside = hp(&e, "player");
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "player"), outside);
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active[0].center,
        [-2.5, 0.8, 0.]
    );
    replay(&e);
}
#[test]
fn crossing_between_pulses_does_not_trigger_entry_damage() {
    let mut s = scene();
    config(&mut s).duration_ticks = 61;
    config(&mut s).interval_ticks = 60;
    let area = s.ability_catalog.abilities[0].area.as_mut().unwrap();
    area.include_source = true;
    area.radius = 0.4;
    let mut e = Engine::new(s).unwrap();
    aim(&mut e, [-2.5, 0.8, 0.]);
    movement(&mut e, 1., 0.);
    e.advance(60).unwrap();
    assert_eq!(hp(&e, "player"), 1000);
    assert_eq!(status(&e, "player"), Status::Outside);
    replay(&e);
}
#[test]
fn occlusion_xyz_and_per_recipient_failure_recomputed_each_pulse() {
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .occlusion = true;
    wall(&mut s, [1., 0.8, 0.], [0.2, 2., 3.]);
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    e.advance(20).unwrap();
    assert_eq!(hp(&e, "a"), 800);
    assert_eq!(hp(&e, "b"), 1000);
    assert_eq!(status(&e, "b"), Status::Occluded);
    replay(&e);
    let mut s = scene();
    entity(&mut s, "b").transform.position = [0., 4.8, 0.];
    entity(&mut s, "a").abilities = None;
    effects(
        &mut s,
        json!([{"kind":"damage","recipient":"target","amount":100},{"kind":"resource","recipient":"target","resource":"energy","delta":-1}]),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    e.advance(20).unwrap();
    assert_eq!(hp(&e, "a"), 1000);
    assert_eq!(status(&e, "a"), Status::EffectFailed);
    assert_eq!(energy(&e), 995);
    replay(&e);
}
#[test]
fn buffs_are_evaluated_at_each_pulse_and_heal_remains_transactional() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[0].duration_ticks = 15;
    let buff=serde_json::from_value(json!({"id":"protect","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"apply_buff","recipient":"target","buff":"resist"}]})).unwrap();
    s.ability_catalog.abilities.push(buff);
    entity(&mut s, "b")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("protect".into());
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "b", 3, "b");
    cast(&mut e);
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "b"), 930);
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "b"), 830);
    replay(&e);
    let mut s = scene();
    entity(&mut s, "a").vitality.as_mut().unwrap().initial = 500;
    effects(
        &mut s,
        json!([{"kind":"heal","recipient":"target","amount":100}]),
    );
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "a"), 700);
    replay(&e);
}
#[test]
fn cancel_after_install_does_not_revoke_committed_area_and_cast_cancel_is_free() {
    let mut e = Engine::new(scene()).unwrap();
    cast(&mut e);
    let id = e.simulation.abilities.state.persistent_areas.active[0]
        .cast
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
    assert_eq!(hp(&e, "a"), 800);
    assert_eq!(energy(&e), 995);
    replay(&e);
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 10;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
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
    assert_eq!(energy(&e), 1000);
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .is_empty()
    );
    replay(&e);
}
#[test]
fn projectile_travels_then_starts_lifetime_at_contact_not_launch() {
    let mut s = scene();
    s.ability_catalog.abilities[0].range = 0.;
    s.ability_catalog.abilities[0].delivery = serde_json::from_value(
        json!({"kind":"projectile","speed":10,"half_extent":0.1,"max_distance":20}),
    )
    .unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .is_empty()
    );
    assert_eq!(energy(&e), 995);
    e.advance(10).unwrap();
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .is_empty()
    );
    while e
        .simulation
        .abilities
        .state
        .persistent_areas
        .active
        .is_empty()
    {
        e.advance(1).unwrap();
        assert!(e.simulation.tick < 50);
    }
    let i = e.simulation.abilities.state.persistent_areas.active[0];
    assert!(i.created_tick > 10);
    assert_eq!(i.expires_at - i.created_tick, 30);
    assert_eq!(hp(&e, "a"), 1000);
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "a"), 900);
    assert_eq!(energy(&e), 995);
    replay(&e);
}
#[test]
fn projectile_impact_respects_source_death_policy() {
    for stop in [false, true] {
        let mut s = scene();
        config(&mut s).end_on_source_death = stop;
        s.ability_catalog.abilities[0].range = 0.;
        s.ability_catalog.abilities[0].delivery = serde_json::from_value(
            json!({"kind":"projectile","speed":10,"half_extent":0.1,"max_distance":20}),
        )
        .unwrap();
        let mut e = Engine::new(s).unwrap();
        cast(&mut e);
        activate(&mut e, "a", 2, "player");
        e.advance(35).unwrap();
        assert_eq!(hp(&e, "a"), if stop { 1000 } else { 900 });
        assert_eq!(energy(&e), 995);
        assert_eq!(
            e.simulation
                .abilities
                .state
                .projectiles
                .receipts
                .back()
                .unwrap()
                .outcome,
            if stop {
                ignis::projectiles::Outcome::EffectFailed
            } else {
                ignis::projectiles::Outcome::Hit
            }
        );
        replay(&e);
    }
}
#[test]
fn invalid_schedules_versions_and_combinations_rejected() {
    for change in [
        json!({"duration_ticks":0}),
        json!({"duration_ticks":216001}),
        json!({"interval_ticks":0}),
        json!({"interval_ticks":3601}),
        json!({"duration_ticks":10,"interval_ticks":10}),
    ] {
        let mut value = serde_json::to_value(scene()).unwrap();
        for (k, v) in change.as_object().unwrap() {
            value["ability_catalog"]["abilities"][0]["area"]["persistence"][k] = v.clone();
        }
        assert!(Engine::new(serde_json::from_value(value).unwrap()).is_err());
    }
    let mut s = scene();
    s.version = 41;
    assert!(Engine::new(s).is_err());
    let mut s = scene();
    s.ability_catalog.abilities[0].periodic = Some(ignis::periodic::Schedule {
        interval_ticks: 10,
        pulses: 3,
        reapply: Reapply::Refresh,
    });
    assert!(Engine::new(s).is_err());
    let mut s = scene();
    config(&mut s).apply_on_spawn = true;
    config(&mut s).duration_ticks = 1;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "a"), 900);
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .is_empty()
    );
    replay(&e);
}
#[test]
fn lifecycle_ring_guides_and_inspection_are_bounded_without_simulation_mutation() {
    let mut s = scene();
    config(&mut s).duration_ticks = 2;
    config(&mut s).interval_ticks = 1;
    let mut e = Engine::new(s).unwrap();
    for _ in 0..90 {
        aim(&mut e, [0., 7., 0.]);
        e.advance(2).unwrap();
    }
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.receipts.len(),
        128
    );
    assert_eq!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .dropped_receipts,
        52
    );
    e.debug_view.npc_routes = true;
    let empty = e.render_frame().route_guides.len();
    aim(&mut e, [0., 7., 0.]);
    let before = e.simulation.abilities.state.clone();
    assert_eq!(e.render_frame().route_guides.len(), empty + 96);
    assert_eq!(e.simulation.abilities.state, before);
    let inspect = e.execute(Command::InspectAbilities.into()).unwrap();
    assert_eq!(inspect["persistent_areas"]["active"][0]["ability"], "area");
    e.advance(2).unwrap();
    assert_eq!(e.render_frame().route_guides.len(), empty);
    replay(&e);
}
#[test]
fn another_source_does_not_fill_a_pending_casts_area_quota() {
    let mut s = scene();
    config(&mut s).duration_ticks = 1000;
    config(&mut s).interval_ticks = 500;
    config(&mut s).reapply = Reapply::Independent;
    let mut other = s.ability_catalog.abilities[0].clone();
    other.id = "other".into();
    s.ability_catalog.abilities.push(other);
    s.ability_catalog.abilities[0].cast_ticks = 20;
    s.ability_catalog.abilities[0].cooldown_ticks = 100;
    for id in ["player", "a"] {
        entity(&mut s, id)
            .abilities
            .as_mut()
            .unwrap()
            .grants
            .push("other".into());
    }
    let mut e = Engine::new(s).unwrap();
    for _ in 0..15 {
        e.execute(
            Command::AimAbility {
                actor: "player".into(),
                slot: 3,
                aim: Aim::Point {
                    position: [0., 7., 0.],
                },
            }
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
    }
    cast(&mut e);
    assert!(e.simulation.abilities.state.casts[slot(&e, "player")].is_some());
    e.execute(
        Command::AimAbility {
            actor: "a".into(),
            slot: 3,
            aim: Aim::Point {
                position: [0., 7., 0.],
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(21).unwrap();
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        17
    );
    assert_eq!(energy(&e), 920);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::Completed
    );
    assert!(e.simulation.abilities.state.ready_at[slot(&e, "player")][0] > e.simulation.tick);
    replay(&e);
}
#[test]
fn projectile_does_not_reserve_area_capacity_and_impact_failure_is_paid() {
    let mut s = scene();
    config(&mut s).duration_ticks = 1000;
    config(&mut s).interval_ticks = 500;
    config(&mut s).reapply = Reapply::Independent;
    let mut other = s.ability_catalog.abilities[0].clone();
    other.id = "other".into();
    s.ability_catalog.abilities.push(other);
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("other".into());
    s.ability_catalog.abilities[0].range = 0.;
    s.ability_catalog.abilities[0].delivery = serde_json::from_value(
        json!({"kind":"projectile","speed":1,"half_extent":0.1,"max_distance":20}),
    )
    .unwrap();
    let mut e = Engine::new(s).unwrap();
    cast(&mut e);
    for _ in 0..16 {
        e.execute(
            Command::AimAbility {
                actor: "player".into(),
                slot: 3,
                aim: Aim::Point {
                    position: [0., 7., 0.],
                },
            }
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
    }
    e.advance(230).unwrap();
    assert_eq!(energy(&e), 915);
    assert_eq!(hp(&e, "a"), 1000);
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        16
    );
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
        Outcome::AreaBudget
    );
    replay(&e);
}
#[test]
fn npc_point_stays_frozen_and_secondary_hits_do_not_become_observations() {
    let mut s = scene();
    entity(&mut s, "b").transform.position = [-3., 0.8, -1.];
    config(&mut s).duration_ticks = 120;
    config(&mut s).interval_ticks = 60;
    s.ability_catalog.abilities[0].cast_ticks = 10;
    s.ability_catalog.abilities[0].cooldown_ticks = 300;
    entity(&mut s,"a").npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([
        {"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":15,"policy":{"spatial_aim":{"mode":"observed_point","range":12}}}},
        {"priority":0,"when":"always","action":{"kind":"wait"}}
    ])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(11).unwrap();
    movement(&mut e, 0., 1.);
    e.advance(60).unwrap();
    assert_eq!((hp(&e, "player"), hp(&e, "b")), (1000, 900));
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active[0].center,
        [-4., 0.8, 0.]
    );
    assert_eq!(
        e.simulation.npcs["a"].memory.unwrap().actor_index,
        Some(slot(&e, "player") as u16)
    );
    replay(&e);
}
