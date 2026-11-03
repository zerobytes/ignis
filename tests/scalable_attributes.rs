//! Critical scale contracts: shared recipients, budget/rollback, retained source
//! composition, format gates and buffers after serialized state restoration.
use ignis::{
    abilities::{Aim, Outcome},
    attributes::{self, Context, Value},
    engine::{Command, Engine},
    model::{BodyKind, RoomRecipe, Scene},
};
use serde_json::json;
fn id(a: usize) -> String {
    format!("p{a:04}")
}
fn run(e: &mut Engine, c: Command) -> serde_json::Value {
    e.execute(c.into()).unwrap()
}
fn cast(e: &mut Engine, actor: usize, slot: u8, target: Option<String>) {
    run(
        e,
        Command::ActivateAbility {
            actor: id(actor),
            slot,
            target,
        },
    );
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 63.,
        depth: 63.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut prototype = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap()
        .clone();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    prototype.footsteps = None;
    prototype.transform.size = [0.05, 1.6, 0.05];
    prototype.vitality = Some(
        serde_json::from_value(json!({"initial":1000000,"maximum":1000000,"scale":1})).unwrap(),
    );
    prototype.abilities = Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["mark","clear","late","group","bolt"]})).unwrap());
    for a in 0..1000 {
        let mut p = prototype.clone();
        p.id = id(a);
        p.name = p.id.clone();
        let angle = a as f32 * std::f32::consts::TAU / 1000.;
        p.transform.position = [20. * angle.cos(), 0.8, 20. * angle.sin()];
        s.entities.push(p);
    }
    s.entities.push(serde_json::from_value(json!({"id":"boss","name":"boss","body":"npc","color":[0.8,0.2,0.1],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"omniscient"},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]},"vitality":{"initial":1000000,"maximum":1000000,"scale":1},"abilities":{"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["bolt"]}})).unwrap());
    s.ability_catalog = serde_json::from_value(json!({"resources":[{"id":"energy","scale":1},{"id":"focus","scale":1}],"attributes":{"buffs":[{"id":"ward","duration_ticks":3600,"reapply":"independent","category":"ward","modifiers":[{"attribute":"combat.damage_dealt.multiplier","operation":{"kind":"add","units":10}}]}]},"abilities":[
        {"id":"mark","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":1}],"effects":[{"kind":"apply_buff","recipient":"target","buff":"ward"}]},
        {"id":"clear","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":7}],"effects":[{"kind":"dispel","recipient":"target","category":"ward","max_instances":64,"require_match":true}]},
        {"id":"late","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":7}],"effects":[{"kind":"dispel","recipient":"target","category":"ward","max_instances":64,"require_match":true},{"kind":"apply_buff","recipient":"target","buff":"ward"},{"kind":"damage","recipient":"target","amount":1},{"kind":"resource","recipient":"target","resource":"focus","delta":1}]},
        {"id":"group","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"apply_buff","recipient":"target","buff":"ward"}],"area":{"radius":100,"max_targets":1001,"include_source":true,"occlusion":false}},
        {"id":"bolt","selection":"spatial","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"delivery":{"kind":"projectile","speed":100,"half_extent":0.01,"max_distance":100},"effects":[{"kind":"damage","recipient":"target","amount":10}]}
    ]})).unwrap();
    for n in 0..61 {
        s.ability_catalog.attributes.definitions.push(
            serde_json::from_value(
                json!({"key":format!("stat.{n}"),"default":{"kind":"number","units":0}}),
            )
            .unwrap(),
        );
    }
    for a in 0..1000 {
        for (key, value) in [
            ("ability.interruptible", Value::boolean(true)),
            ("effect.dispellable", Value::boolean(true)),
            (
                "combat.damage_received.multiplier",
                Value::number(attributes::SCALE),
            ),
            (
                "combat.damage_dealt.multiplier",
                Value::number(attributes::SCALE),
            ),
        ] {
            s.ability_catalog.attributes.bases.push(
                serde_json::from_value(json!({"entity":id(a),"attribute":key,"value":value}))
                    .unwrap(),
            );
        }
        for n in 0..61 {
            s.ability_catalog.attributes.bases.push(serde_json::from_value(json!({"entity":id(a),"attribute":format!("stat.{n}"),"value":{"kind":"number","units":a}})).unwrap());
        }
    }
    s
}
fn slot(e: &Engine, id: &str) -> u16 {
    let entity = e
        .simulation
        .definitions()
        .entities
        .iter()
        .position(|entity| entity.id == id)
        .unwrap();
    e.simulation.actors.slot(entity)
}
fn effective(e: &Engine, actor: &str, key: u8) -> i64 {
    let actor = slot(e, actor);
    e.simulation
        .abilities
        .attributes
        .evaluate(
            &e.simulation.actors,
            &e.simulation.abilities.state.buffs,
            e.simulation.actors.entity(actor) as u16,
            key,
            Value::number(attributes::SCALE),
            Context::default(),
            e.simulation.tick,
        )
        .units()
}
#[test]
fn thousand_sources_shared_recipient_and_group_keep_budgets_snapshots_and_replay() {
    let s = scene();
    assert_eq!(s.ability_catalog.attributes.bases.len(), 65000);
    let mut old = s.clone();
    old.version = 87;
    assert!(
        Engine::new(old)
            .err()
            .unwrap()
            .message
            .contains("Attribute limits")
    );
    let mut e = Engine::new(s.clone()).unwrap();
    assert_eq!(effective(&e, &id(999), 64), 999);
    let boss = slot(&e, "boss");
    let p0 = slot(&e, &id(0)) as usize;
    let old_boss = e.simulation.actors.reference(boss);
    run(
        &mut e,
        Command::AimAbility {
            actor: "boss".into(),
            slot: 1,
            aim: Aim::Direction {
                vector: [1., 0., 0.],
            },
        },
    );
    e.advance(1).unwrap();
    for a in 0..1000 {
        cast(&mut e, a, 1, Some("boss".into()));
    }
    e.advance(1).unwrap();
    let owner = e.simulation.actors.entity(boss) as u16;
    assert_eq!(
        e.simulation.abilities.state.buffs.active.owner_count(owner),
        1000
    );
    assert_eq!(effective(&e, "boss", attributes::DAMAGE_DEALT), 20000);
    let encoded = serde_json::to_value(&e.simulation.abilities.state).unwrap();
    let restored: ignis::abilities::State = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(restored, e.simulation.abilities.state);
    e.simulation.abilities.state = restored;
    let lease = e.simulation.abilities.state.projectiles.active[0]
        .cast
        .origin
        .unwrap();
    let before = e
        .simulation
        .abilities
        .state
        .origins
        .get(Some(lease), old_boss)
        .unwrap()
        .attributes
        .contributions
        .as_ptr();
    run(
        &mut e,
        Command::DespawnActor {
            actor: "boss".into(),
        },
    );
    e.advance(1).unwrap();
    let retained = e
        .simulation
        .abilities
        .state
        .origins
        .get(Some(lease), old_boss)
        .unwrap();
    assert!(retained.detached);
    assert_eq!(retained.attributes.contributions.len(), 1000);
    assert_eq!(retained.attributes.contributions.as_ptr(), before);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    e.advance(26).unwrap();
    assert_eq!(
        e.simulation.vitality.state.actors[p0].unwrap().units,
        999980
    );
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::Impacted && r.actor_instance == old_boss)
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut bad = encoded.clone();
    bad["buffs"]["next_id"] = json!(1);
    assert!(serde_json::from_value::<ignis::abilities::State>(bad).is_err());
    let mut bad = encoded;
    bad["buffs"]["active"][1]["id"] = bad["buffs"]["active"][0]["id"].clone();
    assert!(serde_json::from_value::<ignis::abilities::State>(bad).is_err());

    let mut b = Engine::new(s.clone()).unwrap();
    for a in 0..1000 {
        cast(&mut b, a, 1, Some("boss".into()));
    }
    b.advance(1).unwrap();
    cast(&mut b, 0, 1, Some("boss".into()));
    b.advance(1).unwrap();
    assert_eq!(
        b.simulation.abilities.state.buffs.active.owner_count(owner),
        1001
    );
    let balance = b.simulation.abilities.state.balances[p0];
    cast(&mut b, 0, 1, Some("boss".into()));
    b.advance(1).unwrap();
    assert_eq!(
        b.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::BuffBudget
    );
    assert_eq!(b.simulation.abilities.state.balances[p0], balance);
    cast(&mut b, 0, 1, Some(id(0)));
    b.advance(1).unwrap();
    let before = b.simulation.abilities.state.buffs.clone();
    let balance = b.simulation.abilities.state.balances[p0];
    let hp = b.simulation.vitality.state.actors[p0];
    cast(&mut b, 0, 3, Some(id(0)));
    b.advance(1).unwrap();
    assert_eq!(
        b.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::MissingResource
    );
    assert_eq!(b.simulation.abilities.state.buffs, before);
    assert_eq!(b.simulation.abilities.state.balances[p0], balance);
    assert_eq!(b.simulation.vitality.state.actors[p0], hp);
    let own = b.simulation.actors.entity(p0 as u16) as u16;
    assert!(
        b.simulation
            .abilities
            .state
            .buffs
            .active
            .iter()
            .find(|i| i.owner == own)
            .unwrap()
            .id
            > 1000
    );
    cast(&mut b, 0, 2, Some(id(0)));
    b.advance(1).unwrap();
    assert_eq!(
        b.simulation.abilities.state.buffs.active.owner_count(own),
        0
    );
    let selected: Vec<_> = b
        .simulation
        .abilities
        .state
        .buffs
        .active
        .iter()
        .filter(|i| i.owner == owner)
        .take(64)
        .map(|i| i.id)
        .collect();
    cast(&mut b, 0, 2, Some("boss".into()));
    b.advance(1).unwrap();
    let removed: Vec<_> = b
        .simulation
        .abilities
        .state
        .buffs
        .receipts
        .iter()
        .filter(|r| r.outcome == attributes::End::Dispelled && selected.contains(&r.instance.id))
        .map(|r| r.instance.id)
        .collect();
    assert_eq!(selected, removed);
    assert_eq!(effective(&b, "boss", attributes::DAMAGE_DEALT), 19370);
    assert_eq!(Engine::verify_replay(&b.replay()).unwrap()["matched"], true);

    // A valid serialized counter can be exhausted; reject before resources or effects.
    b.simulation.abilities.state.buffs.next_id = u64::MAX;
    let before = b.simulation.abilities.state.buffs.clone();
    let balance = b.simulation.abilities.state.balances[p0];
    cast(&mut b, 0, 1, Some(id(0)));
    b.advance(1).unwrap();
    assert_eq!(
        b.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::BuffBudget
    );
    assert_eq!(b.simulation.abilities.state.buffs, before);
    assert_eq!(b.simulation.abilities.state.balances[p0], balance);

    let mut group = Engine::new(s).unwrap();
    cast(&mut group, 0, 4, None);
    group.advance(1).unwrap();
    assert_eq!(group.simulation.abilities.state.buffs.active.len(), 1001);
    let source = group.simulation.actors.entity(p0 as u16) as u16;
    assert_eq!(
        group
            .simulation
            .abilities
            .state
            .buffs
            .active
            .source_count(source),
        1001
    );
    assert_eq!(
        group
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
            .filter(|c| c.status == ignis::area::Status::Applied)
            .count(),
        1001
    );
    let balance = group.simulation.abilities.state.balances[p0];
    cast(&mut group, 0, 1, Some(id(0)));
    group.advance(1).unwrap();
    assert_eq!(
        group
            .simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::BuffBudget
    );
    assert_eq!(group.simulation.abilities.state.balances[p0], balance);
    cast(&mut group, 1, 1, Some(id(1)));
    group.advance(1).unwrap();
    assert_eq!(group.simulation.abilities.state.buffs.active.len(), 1002);
    assert_eq!(
        Engine::verify_replay(&group.replay()).unwrap()["matched"],
        true
    );
}
