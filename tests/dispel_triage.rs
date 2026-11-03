use ignis::{
    abilities::{Effect, Outcome, Recipient, Selection},
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
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"attributes":{"buffs":[
        {"id":"guard","duration_ticks":30,"reapply":"refresh","modifiers":[{"attribute":"effect.dispellable","operation":{"kind":"set","value":{"kind":"bool","value":false}}}]},
        {"id":"hex","duration_ticks":120,"reapply":"independent","category":"ailment","modifiers":[{"attribute":"combat.damage_received.multiplier","operation":{"kind":"multiply","basis_points":12000}}]}
    ]},"abilities":[
        {"id":"cleanse","selection":"actor","relation":"allied","range":8,"cast_ticks":12,"cooldown_ticks":30,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"dispel","recipient":"target","category":"ailment","max_instances":1,"require_match":false}]},
        {"id":"poison","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"periodic":{"interval_ticks":60,"pulses":5,"reapply":"independent"},"periodic_category":"ailment","effects":[{"kind":"damage","recipient":"target","amount":1}]},
        {"id":"protected","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"dispellable":false,"periodic":{"interval_ticks":60,"pulses":5,"reapply":"independent"},"periodic_category":"ailment","effects":[{"kind":"damage","recipient":"target","amount":1}]},
        {"id":"restore","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"dispel","recipient":"target","category":"ailment","max_instances":64,"require_match":false}]},
        {"id":"guard","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"apply_buff","recipient":"target","buff":"guard"}]},
        {"id":"hex","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"apply_buff","recipient":"target","buff":"hex"}]},
        {"id":"regen","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"periodic":{"interval_ticks":60,"pulses":5,"reapply":"independent"},"periodic_category":"regeneration","effects":[{"kind":"heal","recipient":"target","amount":1}]}
    ]})).unwrap();
    let p = entity(&mut s, "player");
    p.transform.position = [0., 0.8, -6.];
    p.footsteps = None;
    p.faction = Some("foes".into());
    p.vitality = Some(ignis::vitality::Config {
        initial: 100,
        maximum: 100,
        scale: 1,
    });
    p.abilities = Some(
        serde_json::from_value(
            json!({"resources":[],"grants":["poison","protected","restore","guard","hex","regen"]}),
        )
        .unwrap(),
    );
    for (id, pos, hp) in [
        ("healer", [0., 0.8, 0.], 100),
        ("ally_a", [-2., 0.8, -2.], 20),
        ("ally_b", [3., 0.8, -2.], 100),
    ] {
        s.entities.push(serde_json::from_value(json!({"id":id,"name":id,"body":"npc","faction":"team","color":[0.2,0.6,1],"blocks_vision":false,"transform":{"position":pos,"size":[0.6,1.6,0.6]},"vitality":{"initial":hp,"maximum":100,"scale":1},"npc":{"surface":"floor","speed":3,"stop_distance":1,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap());
    }
    let h = entity(&mut s, "healer");
    h.npc.as_mut().unwrap().target = Some("player".into());
    h.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["cleanse"]})).unwrap());
    h.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([
        {"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":1,"policy":{"allies":{"ranking":"lowest_health_ratio","include_self":false},"require_dispel_match":true}}},
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
fn cast(e: &Engine) -> Option<String> {
    e.simulation.abilities.state.casts[slot(e, "healer")]
        .and_then(|c| c.target)
        .map(|a| e.simulation.actors.id(&e.scene, a).to_string())
}
fn energy(e: &Engine) -> i64 {
    e.simulation.abilities.state.balances[slot(e, "healer")][0]
}
fn issue(e: &mut Engine, n: u8, target: &str) {
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
fn apply(e: &mut Engine, n: u8, target: &str) {
    issue(e, n, target);
    e.advance(1).unwrap();
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn filters_before_health_rank_and_preserves_enemy_memory() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    assert!(cast(&e).is_none());
    apply(&mut e, 1, "ally_b");
    e.advance(1).unwrap();
    assert_eq!(cast(&e).as_deref(), Some("ally_b"));
    assert_eq!(
        e.simulation.npcs["healer"].memory.unwrap().actor_index,
        Some(slot(&e, "player") as u16)
    );
    e.advance(12).unwrap();
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    assert_eq!(energy(&e), 95);
    replay(&e);
}
#[test]
fn protected_and_wrong_category_do_not_motivate_cast() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 2, "ally_a");
    apply(&mut e, 6, "ally_b");
    e.advance(5).unwrap();
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e), 100);
    apply(&mut e, 1, "ally_b");
    e.advance(14).unwrap();
    assert_eq!(energy(&e), 95);
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 2);
    replay(&e);
}
#[test]
fn already_removed_effect_cancels_optional_dispel_without_payment() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(1).unwrap();
    assert!(cast(&e).is_some());
    apply(&mut e, 3, "ally_b");
    e.advance(1).unwrap();
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e), 100);
    replay(&e);
}
#[test]
fn expiry_and_dynamic_author_protection_invalidate_cast() {
    let mut s = scene();
    s.ability_catalog.abilities[1]
        .periodic
        .as_mut()
        .unwrap()
        .interval_ticks = 3;
    s.ability_catalog.abilities[1]
        .periodic
        .as_mut()
        .unwrap()
        .pulses = 1;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(6).unwrap();
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e), 100);
    replay(&e);
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(1).unwrap();
    apply(&mut e, 4, "player");
    e.advance(1).unwrap();
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e), 100);
    e.advance(45).unwrap();
    assert_eq!(energy(&e), 95);
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    replay(&e);
}
#[test]
fn hidden_allies_never_supply_status() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","color":[0.2,0.7,0.5],"transform":{"position":[1.5,1,-1],"size":[0.4,2,1.5]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(20).unwrap();
    assert!(cast(&e).is_none());
    assert_eq!(energy(&e), 100);
    assert!(e.simulation.npcs["healer"].support.observations[slot(&e, "ally_b")].is_none());
    replay(&e);
}
#[test]
fn removable_buffs_are_candidates_without_vitality() {
    let mut s = scene();
    policy(&mut s).allies.as_mut().unwrap().ranking = ignis::ability_policy::Ranking::Nearest;
    entity(&mut s, "ally_b").vitality = None;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 5, "ally_b");
    e.advance(14).unwrap();
    assert_eq!(energy(&e), 95);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    replay(&e);
}
#[test]
fn multiple_operators_share_masks_and_required_matches() {
    let mut s = scene();
    let effect = Effect::Dispel {
        recipient: Recipient::Target,
        category: "ailment".into(),
        max_instances: 1,
        require_match: true,
    };
    s.ability_catalog.abilities[0].effects = vec![effect.clone(), effect];
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(2).unwrap();
    assert!(cast(&e).is_none());
    apply(&mut e, 5, "ally_b");
    e.advance(14).unwrap();
    assert_eq!(energy(&e), 95);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    assert!(e.simulation.abilities.state.periodic.active.is_empty());
    replay(&e);
}
#[test]
fn stable_recipient_and_self_support_are_explicit() {
    let mut e = Engine::new(scene()).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(1).unwrap();
    apply(&mut e, 1, "ally_a");
    assert_eq!(cast(&e).as_deref(), Some("ally_b"));
    e.advance(11).unwrap();
    assert_eq!(energy(&e), 95);
    replay(&e);
    let mut s = scene();
    policy(&mut s).allies.as_mut().unwrap().include_self = true;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "healer");
    e.advance(14).unwrap();
    assert_eq!(energy(&e), 95);
    replay(&e);
}
#[test]
fn explicit_target_and_self_policy_use_same_filter() {
    let mut s = scene();
    policy(&mut s).allies = None;
    entity(&mut s, "healer").npc.as_mut().unwrap().target = Some("ally_b".into());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["healer"].objectives.ability_decisions[0].phase,
        Phase::NoMatchingEffect
    );
    apply(&mut e, 1, "ally_b");
    e.advance(14).unwrap();
    assert_eq!(energy(&e), 95);
    replay(&e);
    let mut s = scene();
    policy(&mut s).allies = None;
    s.ability_catalog.abilities[0].selection = Selection::SelfActor;
    s.ability_catalog.abilities[0].range = 0.;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "healer");
    e.advance(14).unwrap();
    assert_eq!(energy(&e), 95);
    replay(&e);
}
#[test]
fn removal_at_deadline_revalidated_before_charge() {
    let mut s = scene();
    entity(&mut s, "healer").id = "z_healer".into();
    s.ability_catalog.abilities[3].cast_ticks = 1; // player commits before this actor
    // Remaining helpers use healer; use the catalog slot directly in this fixture.
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(11).unwrap();
    apply(&mut e, 3, "ally_b");
    e.advance(1).unwrap();
    let a = slot(&e, "z_healer");
    assert_eq!(e.simulation.abilities.state.balances[a][0], 100);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.actor == a as u16 && r.outcome == Outcome::NoMatchingEffect)
    );
    replay(&e);
}
#[test]
fn policy_validation_is_versioned_atomic_and_legacy_optional_still_pays() {
    let mut s = scene();
    s.version = 44;
    assert!(Engine::new(s).is_err());
    let mut s = scene();
    s.ability_catalog.abilities[0].effects = vec![Effect::Heal {
        recipient: Recipient::Target,
        amount: 1,
    }];
    assert!(Engine::new(s).is_err());
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    let mut bad = e.scene.ability_catalog.clone();
    bad.abilities[0].effects = vec![Effect::Heal {
        recipient: Recipient::Target,
        amount: 1,
    }];
    assert!(
        e.execute(Command::SetAbilityCatalog { catalog: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut s = scene();
    policy(&mut s).require_dispel_match = false;
    let mut e = Engine::new(s).unwrap();
    e.advance(14).unwrap();
    assert_eq!(energy(&e), 95);
    replay(&e);
}

#[test]
fn launched_projectile_is_not_recalled_when_need_disappears() {
    let mut s = scene();
    s.ability_catalog.abilities[0].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 2.,
        half_extent: 0.1,
        max_distance: 20.,
    };
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "ally_b");
    e.advance(13).unwrap();
    assert_eq!(energy(&e), 95);
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 1);
    apply(&mut e, 3, "ally_b");
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 1);
    e.advance(150).unwrap();
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    assert_eq!(energy(&e), 95);
    replay(&e);
}

#[test]
fn sight_loss_cancels_status_support_even_while_memory_remains() {
    let mut s = scene();
    let p = entity(&mut s, "player");
    p.faction = Some("team".into());
    p.transform.position = [0., 0.8, -2.];
    let h = entity(&mut s, "healer").npc.as_mut().unwrap();
    if let Some(ignis::model::Awareness::Perceptive { fov_degrees, .. }) = &mut h.awareness {
        *fov_degrees = 60.;
    }
    s.ability_catalog.abilities[0].cast_ticks = 100;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 1, "player");
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
    assert_eq!(energy(&e), 100);
    assert!(e.simulation.npcs["healer"].memory.is_some());
    replay(&e);
}

#[test]
fn expired_buff_is_excluded_before_ranking_and_intention() {
    let mut s = scene();
    s.ability_catalog.attributes.buffs[1].duration_ticks = 1;
    let mut e = Engine::new(s).unwrap();
    apply(&mut e, 5, "ally_b");
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["healer"].objectives.ability_decisions[0].phase,
        Phase::NoEligibleAlly
    );
    let healer = slot(&e, "healer") as u16;
    assert!(
        !e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.actor == healer)
    );
    assert_eq!(energy(&e), 100);
    replay(&e);
}
