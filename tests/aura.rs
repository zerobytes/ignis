use ignis::{
    abilities::Selection,
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    periodic::Reapply,
    persistent_area::{Anchor, Schedule},
};
use serde_json::json;
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
        anchor: Anchor::Source,
        duration_ticks: 360,
        interval_ticks: 10,
        apply_on_spawn: false,
        reapply: Reapply::Refresh,
        end_on_source_death: false,
    });
    s.ability_catalog.abilities[0].selection = Selection::SelfActor;
    s.ability_catalog.abilities[0].range = 0.;
    s.ability_catalog.abilities[0].area.as_mut().unwrap().radius = 1.5;
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
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn center(e: &Engine) -> [f32; 3] {
    e.simulation.abilities.state.persistent_areas.active[0].center
}
#[test]
fn aura_moves_with_body_and_selects_current_neighbors_without_recharging() {
    for anchor in [Anchor::Fixed, Anchor::Source] {
        let mut s = scene();
        config(&mut s).anchor = anchor;
        let mut e = Engine::new(s).unwrap();
        activate(&mut e, "player", 1, "player");
        let initial = e.simulation.abilities.state.persistent_areas.active[0];
        movement(&mut e, 1., 0.);
        e.advance(50).unwrap();
        assert_eq!(energy(&e), 995);
        if anchor == Anchor::Source {
            assert_eq!(center(&e), e.simulation.position("player").unwrap());
            assert!(hp(&e, "a") < 1000);
        } else {
            assert_eq!(center(&e), initial.center);
            assert_eq!(hp(&e, "a"), 1000);
        }
        let now = e.simulation.abilities.state.persistent_areas.active[0];
        assert_eq!(
            (now.cast.id, now.created_tick, now.expires_at),
            (initial.cast.id, initial.created_tick, initial.expires_at)
        );
        movement(&mut e, 0., 1.);
        e.advance(60).unwrap();
        let left = hp(&e, "a");
        e.advance(30).unwrap();
        assert_eq!(hp(&e, "a"), left);
        replay(&e);
    }
}
#[test]
fn pulse_center_is_pre_physics_but_live_center_and_guide_are_post_physics() {
    let mut s = scene();
    config(&mut s).interval_ticks = 1;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, "player");
    movement(&mut e, 1., 0.);
    let before = e.simulation.position("player").unwrap();
    e.advance(1).unwrap();
    let r = e.simulation.abilities.state.areas.receipts.back().unwrap();
    assert_eq!(r.center, before);
    assert_ne!(center(&e), before);
    assert_eq!(center(&e), e.simulation.position("player").unwrap());
    e.debug_view.npc_routes = true;
    let state = e.simulation.abilities.state.clone();
    let frame = e.render_frame();
    assert!(
        frame
            .route_guides
            .iter()
            .any(|g| g.color == [0.35, 0.65, 1.])
    );
    assert_eq!(state, e.simulation.abilities.state);
    let inspect = e.execute(Command::InspectAbilities.into()).unwrap();
    assert_eq!(
        inspect["persistent_areas"]["active"][0]["anchor_actor"],
        "player"
    );
    let prefix = e.simulation.abilities.state.clone();
    let tick = e.simulation.tick;
    e.advance(30).unwrap();
    replay(&e);
    assert_eq!(
        Engine::replay_at(&e.replay(), Some(tick))
            .unwrap()
            .simulation
            .abilities
            .state,
        prefix
    );
}
#[test]
fn aerial_npc_aura_tracks_real_xyz_without_observing_secondary_targets() {
    let mut s = scene();
    let a = entity(&mut s, "a");
    a.transform.position = [0., 2.8, 0.];
    let n = a.npc.as_mut().unwrap();
    n.objectives = None;
    n.target = None;
    n.mobility = Some(serde_json::from_value(json!({"walk":false,"fly_speed":3})).unwrap());
    n.local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap());
    n.patrol=Some(serde_json::from_value(json!({"route":{"kind":"points_3d","points":[[0,2.8,0],[0,5.8,0],[4,5.8,0]]},"traversal":"loop","wait_ticks":12,"travel_ticks":1200,"retry_ticks":60,"arrival_distance":0.2})).unwrap());
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "a", 1, "a");
    let mut high = 0_f32;
    for _ in 0..180 {
        e.advance(1).unwrap();
        assert_eq!(center(&e), e.simulation.position("a").unwrap());
        high = high.max(center(&e)[1]);
    }
    assert!(high > 5.0, "aerial motor must actually rise");
    assert!(e.simulation.npcs["a"].memory.is_none());
    replay(&e);
}
#[test]
fn source_death_stops_or_continues_following_physical_corpse() {
    for stop in [false, true] {
        let mut s = scene();
        config(&mut s).end_on_source_death = stop;
        entity(&mut s, "player").transform.position = [-4., 5., 0.];
        let mut e = Engine::new(s).unwrap();
        activate(&mut e, "player", 1, "player");
        activate(&mut e, "a", 2, "player");
        assert_eq!(hp(&e, "player"), 0);
        e.advance(40).unwrap();
        if stop {
            assert!(
                e.simulation
                    .abilities
                    .state
                    .persistent_areas
                    .active
                    .is_empty()
            );
        } else {
            assert_eq!(center(&e), e.simulation.position("player").unwrap());
            assert!(center(&e)[1] < 4.);
        }
        replay(&e);
    }
}
#[test]
fn moving_does_not_refresh_cadence_but_recast_does() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, "player");
    movement(&mut e, 0., 1.);
    e.advance(24).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .areas
            .receipts
            .iter()
            .map(|r| r.tick)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    let old = e.simulation.abilities.state.persistent_areas.active[0]
        .cast
        .id;
    activate(&mut e, "player", 1, "player");
    let i = e.simulation.abilities.state.persistent_areas.active[0];
    assert_ne!(i.cast.id, old);
    assert_eq!(i.next_tick, 35);
    assert_eq!(i.created_tick, 25);
    assert_eq!(energy(&e), 990);
    replay(&e);
}
#[test]
fn independent_auras_share_position_not_clock_or_identity() {
    let mut s = scene();
    config(&mut s).reapply = Reapply::Independent;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, "player");
    activate(&mut e, "player", 1, "player");
    movement(&mut e, 0., 1.);
    e.advance(30).unwrap();
    let v = &e.simulation.abilities.state.persistent_areas.active;
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].center, v[1].center);
    assert_ne!(v[0].cast.id, v[1].cast.id);
    assert_eq!(v[1].next_tick - v[0].next_tick, 1);
    replay(&e);
}
#[test]
fn occlusion_changes_as_carrier_walks_around_wall() {
    let mut s = scene();
    s.ability_catalog.abilities[0].area.as_mut().unwrap().radius = 6.;
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .occlusion = true;
    s.entities.push(serde_json::from_value(json!({"id":"cover","name":"cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[-2,1,0],"size":[0.2,2,2]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, "player");
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "a"), 1000);
    movement(&mut e, 0., 1.);
    e.advance(50).unwrap();
    assert!(hp(&e, "a") < 1000);
    replay(&e);
}
#[test]
fn healed_target_and_installed_buff_do_not_require_remaining_in_aura() {
    let mut s = scene();
    s.ability_catalog.abilities[0].area.as_mut().unwrap().radius = 5.;
    s.ability_catalog.abilities[0].effects=serde_json::from_value(json!([{"kind":"heal","recipient":"target","amount":100},{"kind":"apply_buff","recipient":"target","buff":"resist"}])).unwrap();
    entity(&mut s, "a").vitality.as_mut().unwrap().initial = 500;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, "player");
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "a"), 600);
    movement(&mut e, -1., 0.);
    e.advance(90).unwrap();
    let health = hp(&e, "a");
    e.advance(20).unwrap();
    assert_eq!(hp(&e, "a"), health);
    assert!(!e.simulation.abilities.state.buffs.active.is_empty());
    replay(&e);
}
#[test]
fn source_anchor_rejects_ambiguous_delivery_and_old_version_atomically() {
    for selection in [Selection::Actor, Selection::Spatial] {
        let mut s = scene();
        s.ability_catalog.abilities[0].selection = selection;
        s.ability_catalog.abilities[0].range = 10.;
        assert!(Engine::new(s).is_err());
    }
    let mut s = scene();
    s.ability_catalog.abilities[0].delivery = serde_json::from_value(
        json!({"kind":"projectile","speed":10,"half_extent":0.1,"max_distance":20}),
    )
    .unwrap();
    assert!(Engine::new(s).is_err());
    let mut s = scene();
    s.version = 42;
    assert!(Engine::new(s).is_err());
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, "player");
    let before = e.snapshot();
    let mut catalog = e.scene.ability_catalog.clone();
    catalog.abilities[0].selection = Selection::Actor;
    assert!(
        e.execute(Command::SetAbilityCatalog { catalog }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    replay(&e);
}
#[test]
fn legacy_fixed_default_and_self_inclusion_are_explicit() {
    let mut s = scene();
    config(&mut s).anchor = Anchor::Fixed;
    s.version = 42;
    let value = serde_json::to_value(&s).unwrap();
    assert!(
        value["ability_catalog"]["abilities"][0]["area"]["persistence"]
            .get("anchor")
            .is_none()
    );
    let mut e = Engine::new(serde_json::from_value(value).unwrap()).unwrap();
    activate(&mut e, "player", 1, "player");
    movement(&mut e, 0., 1.);
    e.advance(30).unwrap();
    assert_eq!(center(&e), [-4., 0.8, 0.]);
    replay(&e);
    let mut s = scene();
    s.ability_catalog.abilities[0]
        .area
        .as_mut()
        .unwrap()
        .include_source = true;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, "player");
    movement(&mut e, 0., 1.);
    e.advance(30).unwrap();
    assert_eq!(hp(&e, "player"), 700);
    replay(&e);
}
#[test]
fn healing_aura_filters_allies_independently_of_source_self_selection() {
    let mut s = scene();
    entity(&mut s, "player").faction = Some("team".into());
    entity(&mut s, "a").faction = Some("team".into());
    entity(&mut s, "b").faction = Some("other".into());
    for id in ["a", "b"] {
        entity(&mut s, id).vitality.as_mut().unwrap().initial = 500;
    }
    let d = &mut s.ability_catalog.abilities[0];
    d.area.as_mut().unwrap().radius = 8.;
    d.relation = Some(ignis::targeting::Attitude::Allied);
    d.effects =
        serde_json::from_value(json!([{"kind":"heal","recipient":"target","amount":100}])).unwrap();
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, "player");
    e.advance(10).unwrap();
    assert_eq!((hp(&e, "a"), hp(&e, "b")), (600, 500));
    replay(&e);
}
