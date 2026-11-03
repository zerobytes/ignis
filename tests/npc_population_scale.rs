#[path = "../examples/support/npc_scene.rs"]
mod fixture;
use ignis::{
    engine::{Command, Engine},
    perception::{AbilityContext, Sensors, Stimuli},
};
#[test]
fn large_npc_slots_match_linear_sensors_and_reuse_generators_without_state_transfer() {
    let mut e = Engine::new(fixture::scene()).unwrap();
    let mut expected = e.simulation.npcs.clone();
    let positions = e.simulation.positions();
    let mut goals = Default::default();
    let ready = e.simulation.abilities.ready_masks(0);
    ignis::perception::update_with_context(
        e.simulation.definitions(),
        &positions,
        0,
        &e.simulation.navigation,
        &mut expected,
        &mut goals,
        Sensors {
            lives: &e.simulation.vitality.state.lives,
            ability_policy: Some(AbilityContext {
                runtime: &e.simulation.abilities,
                health: &e.simulation.vitality.state.actors,
            }),
            dead: e.simulation.vitality.dead_mask(),
            stimuli: Stimuli {
                manual: &[],
                automatic: &[],
                abilities: &[],
            },
            actors: &e.simulation.actors,
            ability_ready: &ready,
        },
    );
    e.advance(1).unwrap();
    for (id, n) in &expected {
        let actual = &e.simulation.npcs[id];
        assert_eq!(actual.memory, n.memory, "{id}");
        assert_eq!(actual.targeting, n.targeting, "{id}");
        assert_eq!(actual.known_instances, n.known_instances, "{id}");
    }
    assert_eq!(e.simulation.npcs.len(), 100);
    assert_eq!(e.simulation.actors.actors.len(), 628);
    assert!(
        e.simulation
            .actors
            .actors
            .iter()
            .enumerate()
            .filter(|(_, i)| e.simulation.definitions().entities[**i as usize]
                .npc
                .is_some())
            .take(100)
            .all(|(a, _)| a >= 500)
    );
    e.advance(2).unwrap();
    assert!(
        e.simulation
            .npcs
            .values()
            .filter(|n| n.communication.sent > 0)
            .count()
            >= 40
    );
    let before = e.simulation.actors.instance_snapshot();
    assert!(
        e.execute(Command::SetNpcCapacity { capacity: 8 }.into())
            .is_err()
    );
    assert_eq!(e.simulation.actors.instance_snapshot(), before);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    // A producer really owns >8 member slots, not merely a widened authored limit.
    let mut s = fixture::scene();
    s.npc_capacity = 32;
    s.entities.retain(|e| {
        e.body != Some(ignis::model::BodyKind::Npc)
            && (!matches!(e.body, Some(ignis::model::BodyKind::Player))
                || e.id == fixture::player(0))
    });
    s.npc_generators=serde_json::from_value(serde_json::json!([{"id":"population","template":"npc_template","points":(0..32).map(|i|[-12.+(i%8)as f32*3.,0.81,4.+(i/8)as f32*3.]).collect::<Vec<_>>(),"seed":42,"target_alive":24,"interval_ticks":1,"attempts_per_cycle":8,"max_births":64,"retire_dead_after_ticks":0}])).unwrap();
    let mut g = Engine::new(s).unwrap();
    g.advance(16).unwrap();
    assert_eq!(g.simulation.npcs.len(), 24);
    assert_eq!(g.simulation.generators.generators[0].members.len(), 32);
    let member = g.simulation.generators.generators[0]
        .members
        .iter()
        .flatten()
        .max_by_key(|m| m.actor)
        .unwrap()
        .to_owned();
    let name = g
        .simulation
        .actors
        .id(g.simulation.definitions(), member.actor)
        .to_owned();
    g.execute(
        Command::ActivateAbility {
            actor: name.clone(),
            slot: 1,
            target: Some(fixture::player(0)),
        }
        .into(),
    )
    .unwrap();
    g.advance(1).unwrap();
    assert_eq!(
        g.simulation.abilities.state.balances[member.actor as usize][0],
        999999
    );
    g.execute(
        Command::ActivateAbility {
            actor: fixture::player(0),
            slot: 2,
            target: Some(name),
        }
        .into(),
    )
    .unwrap();
    g.advance(16).unwrap();
    assert_eq!(g.simulation.npcs.len(), 24);
    assert!(!g.simulation.actors.contains(member.instance));
    assert_eq!(
        g.simulation.abilities.state.balances[member.actor as usize][0],
        1000000
    );
    assert!(g.simulation.generators.generators[0].births > 24);
    assert_eq!(Engine::verify_replay(&g.replay()).unwrap()["matched"], true);
    let mut old = g.scene.clone();
    old.version = 95;
    assert!(old.validate().is_err());
    let mut oversized = g.scene.clone();
    oversized.npc_capacity = 129;
    assert!(oversized.validate().is_err());
}
