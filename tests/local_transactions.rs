use ignis::{
    abilities::{Effect, Outcome, Recipient},
    engine::{Command, Engine},
    model::{BodyKind, RoomRecipe, Scene},
    physics::Input,
};
use serde_json::json;

fn scene(players: usize, effects: serde_json::Value) -> Scene {
    let mut scene = RoomRecipe {
        width: 32.,
        depth: 28.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene.ability_catalog = serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],
        "attributes":{"buffs":[{"id":"boost","duration_ticks":60,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_dealt.multiplier","operation":{"kind":"multiply","basis_points":20000}}]}]},
        "abilities":[{"id":"mixed","selection":"actor","range":30,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":7}],"effects":effects}]})).unwrap();
    let player = scene
        .entities
        .iter_mut()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap();
    player.transform.position = [-4., 0.8, 0.];
    player.footsteps = None;
    player.vitality =
        Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
    player.abilities = Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["mixed"]})).unwrap());
    let original = player.clone();
    for i in 1..players {
        let mut partner = original.clone();
        partner.id = format!("partner{i}");
        partner.name = partner.id.clone();
        partner.transform.position = [-4. + 3. * i as f32, 0.8, 0.];
        scene.entities.push(partner);
    }
    scene
}
fn slot(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap()
}
fn cast(e: &mut Engine, target: &str) {
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some(target.into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
}
fn verify(e: &Engine) {
    let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let mut prefix = Engine::replay_at(&replay, Some(e.simulation.tick - 1)).unwrap();
    prefix.advance(1).unwrap();
    assert_eq!(
        prefix.simulation.abilities.state,
        e.simulation.abilities.state
    );
    assert_eq!(
        prefix.simulation.vitality.state,
        e.simulation.vitality.state
    );
}

#[test]
fn self_alias_cost_and_repeated_effects_share_one_write_without_overwriting_other_players() {
    let mut e = Engine::new(scene(
        3,
        json!([
            {"kind":"resource","recipient":"source","resource":"energy","delta":20},
            {"kind":"resource","recipient":"target","resource":"energy","delta":-11},
            {"kind":"damage","recipient":"source","amount":40},
            {"kind":"heal","recipient":"target","amount":10}
        ]),
    ))
    .unwrap();
    cast(&mut e, "player");
    let player = slot(&e, "player");
    assert_eq!(e.simulation.abilities.state.balances[player][0], 89);
    assert_eq!(
        e.simulation.vitality.state.actors[player].unwrap().units,
        70
    );
    for id in ["partner1", "partner2"] {
        assert_eq!(e.simulation.abilities.state.balances[slot(&e, id)][0], 100);
        assert_eq!(
            e.simulation.vitality.state.actors[slot(&e, id)]
                .unwrap()
                .units,
            100
        );
    }
    let receipt = e.simulation.abilities.state.receipts.back().unwrap();
    assert_eq!(receipt.effects[0].unwrap().applied, 7);
    assert_eq!(receipt.effects[1].unwrap().applied, -11);
    verify(&e);
}

#[test]
fn dual_lethal_changes_commit_in_instance_order_and_leave_bystander_cast_running() {
    let mut s = scene(
        3,
        json!([
            {"kind":"damage","recipient":"target","amount":100},
            {"kind":"damage","recipient":"source","amount":100}
        ]),
    );
    let mut long = s.ability_catalog.abilities[0].clone();
    long.id = "long".into();
    long.cast_ticks = 100;
    long.effects = vec![Effect::Heal {
        recipient: Recipient::Target,
        amount: 1,
    }];
    s.ability_catalog.abilities.push(long);
    for p in s
        .entities
        .iter_mut()
        .filter(|e| e.body == Some(BodyKind::Player))
    {
        p.abilities.as_mut().unwrap().grants.push("long".into());
    }
    let mut e = Engine::new(s).unwrap();
    for actor in ["partner1", "partner2"] {
        e.execute(
            Command::ActivateAbility {
                actor: actor.into(),
                slot: 2,
                target: Some(actor.into()),
            }
            .into(),
        )
        .unwrap();
    }
    e.advance(1).unwrap();
    cast(&mut e, "partner1");
    let deaths = &e.simulation.vitality.state.deaths;
    assert_eq!(deaths.len(), 2);
    assert!(deaths[0].actor_instance.id() < deaths[1].actor_instance.id());
    let causes: Vec<_> = deaths.iter().map(|d| (d.actor, d.effect_index)).collect();
    assert!(causes.contains(&(slot(&e, "partner1") as u16, 0)));
    assert!(causes.contains(&(slot(&e, "player") as u16, 1)));
    assert!(e.simulation.abilities.state.casts[slot(&e, "partner1")].is_none());
    assert!(e.simulation.abilities.state.casts[slot(&e, "partner2")].is_some());
    verify(&e);
}

#[test]
fn late_missing_target_resource_discards_cost_health_buff_noise_and_resource_writes() {
    let mut s = scene(
        3,
        json!([
            {"kind":"resource","recipient":"source","resource":"energy","delta":-11},
            {"kind":"damage","recipient":"target","amount":100},
            {"kind":"apply_buff","recipient":"source","buff":"boost"},
            {"kind":"noise","radius":4},
            {"kind":"resource","recipient":"target","resource":"energy","delta":10}
        ]),
    );
    s.entities
        .iter_mut()
        .find(|e| e.id == "partner1")
        .unwrap()
        .abilities = None;
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "partner1");
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::MissingResource
    );
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "player")][0],
        100
    );
    assert_eq!(
        e.simulation.vitality.state.actors[slot(&e, "partner1")]
            .unwrap()
            .units,
        100
    );
    assert!(e.simulation.vitality.state.deaths.is_empty());
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    assert_eq!(e.simulation.abilities.state.buffs.next_id, 1);
    assert_eq!(e.simulation.abilities.state.emitted, 0);
    verify(&e);
}

#[test]
fn eight_sparse_costs_and_eight_distinct_target_writes_fit_the_operation_budget() {
    let mut s = scene(3, json!([]));
    let resources: Vec<_> = (0..32)
        .map(|i| json!({"id":format!("r{i}"),"scale":1}))
        .collect();
    s.ability_catalog.resources = serde_json::from_value(json!(resources)).unwrap();
    s.ability_catalog.abilities[0].costs = serde_json::from_value(json!(
        (24..32)
            .map(|i| json!({"resource":format!("r{i}"),"units":1}))
            .collect::<Vec<_>>()
    ))
    .unwrap();
    s.ability_catalog.abilities[0].effects = (0..8)
        .map(|i| Effect::Resource {
            recipient: Recipient::Target,
            resource: format!("r{i}"),
            delta: 1,
        })
        .collect();
    for p in s
        .entities
        .iter_mut()
        .filter(|e| e.body == Some(BodyKind::Player))
    {
        p.abilities.as_mut().unwrap().resources = serde_json::from_value(json!(
            (0..8)
                .chain(24..32)
                .map(|i| json!({"resource":format!("r{i}"),"initial":100,"capacity":200}))
                .collect::<Vec<_>>()
        ))
        .unwrap();
    }
    let mut e = Engine::new(s).unwrap();
    cast(&mut e, "partner1");
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
    let source = &e.simulation.abilities.state.balances[slot(&e, "player")];
    assert!(source[24..].iter().all(|v| *v == 99));
    assert!(source[..8].iter().all(|v| *v == 100));
    assert!(source[8..24].iter().all(|v| *v == 0));
    let target = &e.simulation.abilities.state.balances[slot(&e, "partner1")];
    assert!(target[..8].iter().all(|v| *v == 101));
    assert!(target[8..24].iter().all(|v| *v == 0));
    assert!(target[24..].iter().all(|v| *v == 100));
    let bystander = &e.simulation.abilities.state.balances[slot(&e, "partner2")];
    assert!(
        bystander[..8]
            .iter()
            .chain(&bystander[24..])
            .all(|v| *v == 100)
    );
    assert!(bystander[8..24].iter().all(|v| *v == 0));
    verify(&e);
}

#[test]
fn offline_single_player_legacy_and_explicit_control_share_simulation_and_combat() {
    let s = scene(
        1,
        json!([
            {"kind":"damage","recipient":"target","amount":40},
            {"kind":"resource","recipient":"source","resource":"energy","delta":3}
        ]),
    );
    let mut local = Engine::new(s.clone()).unwrap();
    let mut controlled = Engine::new(s).unwrap();
    let binding = controlled
        .execute(
            Command::BindControl {
                actor: "player".into(),
            }
            .into(),
        )
        .unwrap();
    let input = Input {
        x: 1.,
        z: 0.,
        ..Default::default()
    };
    local.execute(Command::SetInput { input }.into()).unwrap();
    controlled
        .execute(
            Command::SetControlInput {
                control: binding["control"].as_str().unwrap().into(),
                input,
            }
            .into(),
        )
        .unwrap();
    local.advance(15).unwrap();
    controlled.advance(15).unwrap();
    cast(&mut local, "player");
    cast(&mut controlled, "player");
    assert_eq!(
        local.simulation.positions(),
        controlled.simulation.positions()
    );
    assert_eq!(
        local.simulation.abilities.state,
        controlled.simulation.abilities.state
    );
    assert_eq!(
        local.simulation.vitality.state,
        controlled.simulation.vitality.state
    );
    verify(&local);
    verify(&controlled);
}
