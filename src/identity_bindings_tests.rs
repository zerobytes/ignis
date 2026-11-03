//! Recycling is a test seam, not a population API. Verify consumers against a real reused slot.
use crate::{
    attributes::{self, Context, Install, Value},
    engine::{Command, Engine},
    model::Scene,
    perception::{NpcState, Observation, Source},
    targeting::Catalog,
};
use std::collections::BTreeMap;

fn scene(text: &str) -> Scene {
    serde_json::from_str(text).unwrap()
}
fn entity(s: &Scene, id: &str) -> u16 {
    s.entities.iter().position(|e| e.id == id).unwrap() as u16
}
fn actor(c: &Catalog, s: &Scene, id: &str) -> u16 {
    c.slot(entity(s, id) as usize)
}
fn quiet(mut s: Scene) -> Scene {
    for e in &mut s.entities {
        if let Some(n) = &mut e.npc {
            n.enabled = false;
        }
    }
    s
}
fn observation(c: &Catalog, a: u16) -> Observation {
    Observation {
        instance: Some(c.reference(a)),
        actor_index: Some(a),
        life: Some(0),
        position: [1., 0.8, 1.],
        tick: 0,
        source: Source::Vision,
        noise_id: None,
    }
}
#[test]
fn recycled_buff_owner_cannot_inherit_contributions_dispel_or_refresh() {
    let s = scene(include_str!("../examples/npc-attributes.scene.json"));
    let mut c = Catalog::new(&s);
    let r = attributes::Registry::new(&s, &["enhancement".into()]);
    let mut state = attributes::State::default();
    let owner = entity(&s, "player");
    let source = entity(&s, "cleanser");
    let p = Install {
        buff: 0,
        owner,
        source,
        owner_instance: c.entity_reference(owner as usize).unwrap(),
        source_instance: c.entity_reference(source as usize).unwrap(),
        execution_id: 1,
    };
    r.install(&mut state, p, 0);
    assert!(
        !r.evaluate(
            &c,
            &state,
            owner,
            attributes::INTERRUPTIBLE,
            Value::boolean(true),
            Context::default(),
            0
        )
        .as_bool()
    );
    let next = c.recycle_for_test(owner);
    assert_eq!(next.slot, p.owner_instance.slot);
    assert!(
        r.evaluate(
            &c,
            &state,
            owner,
            attributes::INTERRUPTIBLE,
            Value::boolean(true),
            Context::default(),
            0
        )
        .as_bool()
    );
    assert!(!r.removable(&c, &state.active[0], owner, 0));
    let old = state.inspect(&s, &c, 0);
    assert!(old["active"][0]["owner"].is_null());
    assert_eq!(old["active"][0]["owner_instance"], p.owner_instance.id());
    r.install(
        &mut state,
        Install {
            owner_instance: next,
            execution_id: 2,
            ..p
        },
        0,
    );
    assert_eq!(state.active.len(), 2);
    assert_eq!(state.receipts.len(), 2);
    assert!(
        !r.evaluate(
            &c,
            &state,
            owner,
            attributes::INTERRUPTIBLE,
            Value::boolean(true),
            Context::default(),
            0
        )
        .as_bool()
    );
}
#[test]
fn source_reuse_does_not_refresh_original_authorship_or_relabel_receipts() {
    let s = scene(include_str!("../examples/npc-attributes.scene.json"));
    let mut c = Catalog::new(&s);
    let r = attributes::Registry::new(&s, &["enhancement".into()]);
    let mut state = attributes::State::default();
    let owner = entity(&s, "player");
    let source = entity(&s, "cleanser");
    let p = Install {
        buff: 0,
        owner,
        source,
        owner_instance: c.entity_reference(owner as usize).unwrap(),
        source_instance: c.entity_reference(source as usize).unwrap(),
        execution_id: 1,
    };
    r.install(&mut state, p, 0);
    let new = c.recycle_for_test(source);
    r.install(
        &mut state,
        Install {
            source_instance: new,
            execution_id: 2,
            ..p
        },
        1,
    );
    assert_eq!(state.active.len(), 2);
    r.install(
        &mut state,
        Install {
            source_instance: new,
            execution_id: 3,
            ..p
        },
        2,
    );
    assert_eq!(state.active.len(), 2);
    let view = state.inspect(&s, &c, 2);
    assert!(view["active"][0]["source"].is_null());
    assert_eq!(view["active"][0]["source_instance"], p.source_instance.id());
    assert_eq!(view["receipts"][2]["responsible"]["instance_id"], new.id());
}
#[test]
fn dead_owner_cleanup_cannot_apply_to_another_individual() {
    let mut e = Engine::new(quiet(scene(include_str!(
        "../examples/npc-attributes.scene.json"
    ))))
    .unwrap();
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
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 1);
    let p = entity(&e.scene, "player");
    e.simulation.actors.recycle_for_test(p);
    e.advance(1).unwrap();
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    assert_eq!(
        e.simulation
            .abilities
            .state
            .buffs
            .receipts
            .back()
            .unwrap()
            .outcome,
        attributes::End::OwnerRemoved
    );
}
#[test]
fn new_individual_does_not_inherit_recognition_or_live_target_but_old_clue_survives() {
    let s = scene(include_str!("../examples/npc-radio.scene.json"));
    let mut c = Catalog::new(&s);
    let a = actor(&c, &s, "player");
    let old = observation(&c, a);
    let mut k = crate::recognition::Knowledge::default();
    let config = Some(crate::recognition::Config {
        acquire_ticks: 2,
        decay_ticks: 2,
    });
    assert!(!k.observe(config, old));
    assert!(k.observe(config, old));
    let npc = &s.entities[entity(&s, "sender") as usize];
    let mut state = NpcState::new(npc.npc.as_ref().unwrap());
    state.memory = Some(old);
    state.observed_this_tick = true;
    assert!(state.current_visual(a, 0, &c).is_some());
    c.recycle_for_test(entity(&s, "player"));
    assert!(state.current_visual(a, 0, &c).is_none());
    assert_eq!(state.memory, Some(old));
    let view = c.inspect_state(&s, &state);
    assert!(view["memory"]["actor_id"].is_null());
    assert_eq!(view["memory"]["position"], serde_json::json!(old.position));
    assert!(!k.observe(config, observation(&c, a)));
    assert!(!k.tracks[a as usize].confirmed);
}
#[test]
fn frozen_radio_rejects_reused_sender_or_subject_without_updating_memory() {
    for replace_sender in [false, true] {
        let s = scene(include_str!("../examples/npc-radio.scene.json"));
        let mut c = Catalog::new(&s);
        let positions: BTreeMap<_, _> = s
            .entities
            .iter()
            .map(|e| (e.id.clone(), e.transform.position))
            .collect();
        let mut states: BTreeMap<_, _> = s
            .entities
            .iter()
            .filter_map(|e| e.npc.as_ref().map(|n| (e.id.clone(), NpcState::new(n))))
            .collect();
        let a = actor(&c, &s, "player");
        let mut old = observation(&c, a);
        old.position = positions["player"];
        states.get_mut("sender").unwrap().memory = Some(old);
        states.get_mut("sender").unwrap().observed_this_tick = true;
        let reports = crate::communication::collect(&s, &positions, 1, &c, &mut states);
        let receive = |c: &Catalog| {
            let mut state = crate::communication::State::default();
            let got = crate::communication::receive(
                &reports,
                crate::communication::Listener {
                    scene: &s,
                    actors: c,
                    npc_index: entity(&s, "receiver") as usize,
                    position: positions["receiver"],
                    tick: 1,
                    active_memory: None,
                    known_dead: crate::actor_set::ActorSet::EMPTY,
                    known_lives: &[0; 9],
                    known_instances: &[None; 9],
                },
                &mut state,
            );
            (got, state)
        };
        assert!(receive(&c).0.is_some());
        c.recycle_for_test(entity(&s, if replace_sender { "sender" } else { "player" }));
        let (got, state) = receive(&c);
        assert!(got.is_none());
        assert!(state.rejected > 0);
        assert!(state.last_received.is_none());
    }
}
#[test]
fn reused_sound_emitter_is_audible_to_new_occupant_and_remains_anonymous() {
    let mut e = Engine::new(scene(include_str!(
        "../examples/npc-sound-attention.scene.json"
    )))
    .unwrap();
    let index = entity(&e.scene, "guard");
    let old = e
        .simulation
        .actors
        .entity_reference(index as usize)
        .unwrap();
    let mut position = e.simulation.position("guard").unwrap();
    position[0] += 1.;
    e.simulation.footsteps.pending.push(crate::noise::Noise {
        id: 1,
        tick: 0,
        position,
        radius: 10.,
        emitter_index: Some(index),
        emitter_instance: Some(old),
    });
    e.simulation.actors.recycle_for_test(index);
    e.advance(1).unwrap();
    let memory = e.simulation.npcs["guard"].memory.unwrap();
    assert_eq!(memory.source, Source::Sound);
    assert_eq!(memory.position, position);
    assert!(memory.instance.is_none());
    assert!(memory.actor_index.is_none());
}
#[test]
fn combat_and_vital_history_retain_original_participants_after_reuse() {
    let mut e = Engine::new(quiet(scene(include_str!(
        "../examples/npc-resurrection.scene.json"
    ))))
    .unwrap();
    let p = entity(&e.scene, "player");
    let v = entity(&e.scene, "patient");
    let original = e.simulation.actors.entity_reference(p as usize).unwrap();
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("patient".into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    e.simulation.actors.recycle_for_test(p);
    e.simulation.actors.recycle_for_test(v);
    let history = e.execute(Command::InspectAbilities.into()).unwrap();
    let last = history["receipts"].as_array().unwrap().last().unwrap();
    assert!(last["actor"].is_null());
    assert_eq!(last["actor_instance"], original.id());
    assert!(last["effects"][0]["actor"].is_null());
    assert_eq!(last["ability"], "strike");
    let life = e.execute(Command::InspectVitality.into()).unwrap();
    assert!(life["deaths"][0]["actor"].is_null());
    assert!(life["deaths"][0]["source"].is_null());
    assert_eq!(life["deaths"][0]["source_instance"], original.id());
}

#[test]
fn periodic_refresh_and_history_do_not_bind_to_replacement() {
    let mut e = Engine::new(quiet(scene(include_str!(
        "../examples/npc-periodic.scene.json"
    ))))
    .unwrap();
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("dummy".into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(31).unwrap();
    let old = e.simulation.abilities.state.periodic.active[0];
    let target = entity(&e.scene, "dummy");
    let next = e.simulation.actors.recycle_for_test(target);
    let mut new = old.cast;
    new.id += 1;
    new.life.target_instance = Some(next);
    let schedule = e.scene.ability_catalog.abilities[old.definition as usize]
        .periodic
        .unwrap();
    e.simulation.abilities.state.periodic.install(
        old.actor,
        old.definition,
        new,
        schedule,
        e.simulation.tick,
    );
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 2);
    let view = e.execute(Command::InspectAbilities.into()).unwrap();
    assert!(view["periodic"]["receipts"][0]["target"].is_null());
    assert_eq!(
        view["periodic"]["receipts"][0]["target_instance"],
        old.cast.life.target_instance.unwrap().id()
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 1);
    assert_eq!(
        e.simulation.abilities.state.periodic.active[0]
            .cast
            .life
            .target_instance,
        Some(next)
    );
}
#[test]
fn authored_trigger_binding_exits_without_adopting_replacement() {
    let mut s = crate::model::RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut zone = s
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    zone.id = "zone".into();
    zone.body = None;
    zone.trigger = Some(crate::trigger::Config {
        enabled: true,
        targets: vec!["player".into()],
    });
    zone.transform.size = [4.; 3];
    let point = zone.transform.position;
    s.entities.push(zone);
    s.validate().unwrap();
    let mut c = Catalog::new(&s);
    let mut runtime = crate::trigger::Runtime::new(&s, &c);
    runtime.update(0, &c, |_| point);
    let old = runtime.state.events[0].target_instance;
    c.recycle_for_test(entity(&s, "player"));
    runtime.update(1, &c, |_| panic!("removed target must not be queried"));
    assert_eq!(runtime.occupied("zone", "player"), Some(false));
    assert_eq!(runtime.state.events.len(), 2);
    assert_eq!(
        runtime.state.events[1].kind,
        crate::trigger::EventKind::Exit
    );
    assert_eq!(runtime.state.events[1].target_instance, old);
    runtime.update(2, &c, |_| panic!("removed target must not be queried"));
    assert_eq!(runtime.state.events.len(), 2);
}
