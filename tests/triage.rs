use ignis::{
    abilities::{Outcome, Selection},
    ability_policy::{Phase, Policy},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::json;
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
fn policy(s: &mut Scene) -> &mut Policy {
    let ignis::objectives::Action::UseAbility {
        policy: Some(p), ..
    } = &mut entity(s, "healer")
        .npc
        .as_mut()
        .unwrap()
        .objectives
        .as_mut()
        .unwrap()[0]
        .action
    else {
        unreachable!()
    };
    p
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
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
        {"id":"heal","selection":"actor","range":8,"relation":"allied","cast_ticks":12,"cooldown_ticks":30,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"heal","recipient":"target","amount":30}]},
        {"id":"injure","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":20}]},
        {"id":"restore","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"heal","recipient":"target","amount":1000}]}
    ]})).unwrap();
    let p = entity(&mut s, "player");
    p.transform.position = [0., 0.8, -6.];
    p.footsteps = None;
    p.faction = Some("foes".into());
    p.abilities = Some(
        serde_json::from_value(json!({"resources":[],"grants":["injure","restore"]})).unwrap(),
    );
    for (id, pos, hp, max) in [
        ("healer", [0., 0.8, 0.], 100, 100),
        ("ally_a", [-2., 0.8, -2.], 40, 100),
        ("ally_b", [2., 0.8, -2.], 60, 200),
    ] {
        let e:Entity=serde_json::from_value(json!({"id":id,"name":id,"body":"npc","faction":"team","color":[0.2,0.6,1],"blocks_vision":false,"transform":{"position":pos,"size":[0.6,1.6,0.6]},"vitality":{"initial":hp,"maximum":max,"scale":1},"npc":{"surface":"floor","speed":3,"stop_distance":1,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap();
        s.entities.push(e);
    }
    let h = entity(&mut s, "healer");
    h.npc.as_mut().unwrap().target = Some("player".into());
    h.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["heal"]})).unwrap());
    h.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([
        {"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":1,"policy":{"allies":{"ranking":"lowest_health_ratio","include_self":false},"vitality_below":{"subject":"target","basis_points":7000}}}},
        {"priority":0,"when":"always","action":{"kind":"wait"}}
    ])).unwrap());
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
fn cast(e: &Engine) -> Option<String> {
    e.simulation.abilities.state.casts[slot(e, "healer")]
        .and_then(|c| c.target)
        .map(|a| e.simulation.actors.id(&e.scene, a).to_string())
}
fn phase(e: &Engine) -> Phase {
    e.simulation.npcs["healer"].objectives.ability_decisions[0].phase
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn use_on(e: &mut Engine, n: u8, target: &str) {
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: n,
            target: Some(target.into()),
        }
        .into(),
    )
    .unwrap();
}
#[test]
fn ranks_exact_ratios_and_keeps_enemy_memory_separate() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("ally_b"));
    assert_eq!(
        e.simulation.npcs["healer"].memory.unwrap().actor_index,
        Some(slot(&e, "player") as u16)
    );
    e.advance(12).unwrap();
    assert_eq!(hp(&e, "ally_b"), 90);
    assert_eq!(hp(&e, "ally_a"), 40);
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "healer")][0],
        95
    );
    replay(&e);
}
#[test]
fn stable_cast_despite_more_urgent_candidate_then_retriage() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    use_on(&mut e, 1, "ally_a");
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "ally_a"), 20);
    assert_eq!(cast(&e).as_deref(), Some("ally_b"));
    e.advance(11).unwrap();
    assert_eq!(hp(&e, "ally_b"), 90);
    e.advance(30).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("ally_a"));
    replay(&e);
}
#[test]
fn healed_candidate_cancels_without_charging_or_switching_midcast() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    use_on(&mut e, 2, "ally_b");
    e.advance(2).unwrap();
    assert!(cast(&e).is_none());
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "healer")][0],
        100
    );
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::Cancelled)
    );
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("ally_a"));
    replay(&e);
}
#[test]
fn visibility_confirmation_and_hidden_health_are_required() {
    let mut s = scene();
    entity(&mut s, "healer").npc.as_mut().unwrap().recognition = Some(ignis::recognition::Config {
        acquire_ticks: 3,
        decay_ticks: 6,
    });
    s.entities.push(serde_json::from_value(json!({"id":"screen","name":"wall","body":"fixed","color":[0.2,0.7,0.5],"transform":{"position":[1,1,-1],"size":[0.3,2,1.5]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    assert_eq!(phase(&e), Phase::NoEligibleAlly);
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("ally_a"));
    assert!(e.simulation.npcs["healer"].support.observations[slot(&e, "ally_b")].is_none());
    replay(&e);
}
#[test]
fn eligible_before_rank_and_xyz_range() {
    let mut s = scene();
    entity(&mut s, "ally_b").transform.position = [2., 12., -2.];
    s.ability_catalog.abilities[0].range = 3.;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("ally_a"));
    replay(&e);
}
#[test]
fn nearest_then_id_and_optional_self() {
    let mut s = scene();
    policy(&mut s).allies.as_mut().unwrap().ranking = ignis::ability_policy::Ranking::Nearest;
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("ally_a"));
    policy(&mut s).allies.as_mut().unwrap().include_self = true;
    entity(&mut s, "healer").vitality.as_mut().unwrap().initial = 10;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("healer"));
    e.advance(12).unwrap();
    assert_eq!(hp(&e, "healer"), 40);
    replay(&e);
}
#[test]
fn approach_uses_support_evidence_instead_of_enemy_position() {
    let mut s = scene();
    s.ability_catalog.abilities[0].range = 2.;
    policy(&mut s).approach = Some(ignis::ability_policy::Approach { margin: 0.3 });
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::Approaching);
    assert_eq!(
        e.simulation.npcs["healer"].objectives.destination,
        Some([2., 0.8, -2.])
    );
    e.advance(10).unwrap();
    assert!(e.simulation.position("healer").unwrap()[0] > 0.1);
    e.advance(80).unwrap();
    assert!(hp(&e, "ally_b") > 60);
    replay(&e);
}
#[test]
fn territory_and_dead_allies_excluded_without_global_health_triage() {
    let mut s = scene();
    entity(&mut s, "healer").npc.as_mut().unwrap().territory = Some(ignis::territory::Config {
        home: [0., 0.],
        radius: 3.,
        arrival_distance: 0.4,
        rearm_ticks: 10,
    });
    entity(&mut s, "ally_b").transform.position = [4., 0.8, -2.];
    entity(&mut s, "ally_a").vitality.as_mut().unwrap().initial = 0;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::NoEligibleAlly);
    assert!(cast(&e).is_none());
    replay(&e);
}
#[test]
fn same_tick_restore_is_revalidated_before_commit_payment() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 0;
    // Rename healer so player commits first in lexicographic actor order.
    entity(&mut s, "healer").id = "z_healer".into();
    let mut e = Engine::new(s).unwrap();
    use_on(&mut e, 2, "ally_b");
    e.advance(1).unwrap();
    assert_eq!(hp(&e, "ally_b"), 200);
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "z_healer")][0],
        100
    );
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::ConditionFailed)
    );
    replay(&e);
}
#[test]
fn invalid_version_selection_and_atomic_edit() {
    let mut projectile = scene();
    projectile.ability_catalog.abilities[0].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 10.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    assert!(projectile.validate().is_ok());
    policy(&mut projectile)
        .allies
        .as_mut()
        .unwrap()
        .include_self = true;
    assert!(projectile.validate().is_err());
    let mut s = scene();
    s.version = 43;
    assert!(s.validate().is_err());
    s.version = 44;
    s.ability_catalog.abilities[0].selection = Selection::SelfActor;
    s.ability_catalog.abilities[0].range = 0.;
    assert!(s.validate().is_err());
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let before = e.snapshot();
    let mut bad = entity(&mut e.scene.clone(), "healer").clone();
    let mut value = serde_json::to_value(&bad).unwrap();
    value["npc"]["objectives"][0]["action"]["policy"]["allies"]["ranking"] =
        json!("global_hidden_health");
    assert!(serde_json::from_value::<Entity>(value).is_err());
    bad.abilities = None;
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(bad)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    replay(&e);
}
#[test]
fn leaving_fov_cancels_even_with_unexpired_memory() {
    let mut s = scene();
    s.ability_catalog.abilities[0].cast_ticks = 120;
    let p = entity(&mut s, "player");
    p.faction = Some("team".into());
    p.transform.position = [0., 0.8, -2.];
    p.vitality = Some(ignis::vitality::Config {
        initial: 10,
        maximum: 100,
        scale: 1,
    });
    let h = entity(&mut s, "healer").npc.as_mut().unwrap();
    if let Some(ignis::model::Awareness::Perceptive { fov_degrees, .. }) = &mut h.awareness {
        *fov_degrees = 60.;
    }
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("player"));
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_input","input":{"x":1,"z":0}}))
            .unwrap()
            .into(),
    )
    .unwrap();
    e.advance(35).unwrap();
    assert!(cast(&e).is_none());
    assert!(e.simulation.npcs["healer"].memory.is_some());
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "healer")][0],
        100
    );
    replay(&e);
}
#[test]
fn anonymous_sound_never_populates_support_observations() {
    let mut value = serde_json::to_value(scene()).unwrap();
    for e in value["entities"].as_array_mut().unwrap() {
        if e["id"] == "healer" {
            e["npc"]["target"] = json!(null);
            e["npc"]["awareness"]["initial_facing_degrees"] = json!(180);
            e["npc"]["awareness"]["fov_degrees"] = json!(60);
            e["npc"]["awareness"]["hearing"] = json!({"range":20});
        }
    }
    let mut e = Engine::new(serde_json::from_value(value).unwrap()).unwrap();
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Entity {
                id: "ally_b".into(),
            },
            radius: 20.,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    let n = &e.simulation.npcs["healer"];
    assert!(n.heard_this_tick);
    assert_eq!(n.memory.unwrap().source, ignis::perception::Source::Sound);
    assert!(n.support.observations.iter().all(Option::is_none));
    assert_eq!(phase(&e), Phase::NoEligibleAlly);
    replay(&e);
}
