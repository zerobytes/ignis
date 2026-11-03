use ignis::{
    engine::{Command, Engine},
    model::{Awareness, RoomRecipe, Scene},
    perception::{self, Sensors, Source, Stimuli},
    recognition::{Config, Knowledge},
    targeting::Reason,
};
use serde_json::json;
use std::collections::BTreeMap;
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 20.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    player.transform.position = [-2., 0.85, -4.];
    player.faction = Some("hostiles".into());
    s.entities.push(serde_json::from_value(json!({"id":"guard","name":"guard","body":"npc","faction":"guards","color":[0.7,0.2,0.8],"transform":{"position":[0,0.85,0],"size":[0.7,1.6,0.7]},"npc":{"surface":"floor","speed":0.1,"stop_distance":1.2,"enabled":true,"targeting":{"kind":"hostile_nearest","switch_margin":0.2},"recognition":{"acquire_ticks":3,"decay_ticks":5},"awareness":{"mode":"perceptive","vision_range":15,"fov_degrees":120,"initial_facing_degrees":0,"memory_ticks":30,"search_ticks":10,"search_turn_degrees_per_second":90,"hearing":{"range":12}}}})).unwrap());
    s.entities.push(serde_json::from_value(json!({"id":"other","name":"other","body":"npc","blocks_vision":false,"faction":"hostiles","color":[0.9,0.1,0.1],"transform":{"position":[2,0.85,-4],"size":[0.7,1.6,0.7]},"npc":{"surface":"floor","speed":1,"stop_distance":1.2,"enabled":false}})).unwrap());
    s.faction_relations =
        serde_json::from_value(json!([{"from":"guards","to":"hostiles","attitude":"hostile"}]))
            .unwrap();
    s
}
fn sense(e: &mut Engine, p: &BTreeMap<String, [f32; 3]>, tick: u64) {
    perception::update_with_context(
        &e.scene,
        p,
        tick,
        &e.simulation.navigation,
        &mut e.simulation.npcs,
        &mut BTreeMap::new(),
        Sensors {
            lives: &[0; ignis::targeting::MAX_ACTORS],
            ability_policy: None,
            dead: ignis::actor_set::ActorSet::EMPTY,
            ability_ready: &[0; ignis::targeting::MAX_ACTORS],
            actors: &e.simulation.actors,
            stimuli: Stimuli {
                abilities: &[],
                manual: &[],
                automatic: &[],
            },
        },
    );
}
fn track(e: &Engine, id: &str) -> ignis::recognition::Track {
    e.simulation.npcs["guard"].knowledge.tracks[e
        .simulation
        .actors
        .slot(e.scene.entities.iter().position(|x| x.id == id).unwrap())
        as usize]
}
#[test]
fn exposure_threshold_is_exact_and_each_actor_has_independent_evidence() {
    let mut e = Engine::new(scene()).unwrap();
    let mut p = e.simulation.positions();
    p.insert("other".into(), [2., 0.85, 4.]);
    sense(&mut e, &p, 0);
    sense(&mut e, &p, 1);
    assert_eq!(track(&e, "player").evidence, 10);
    assert!(!track(&e, "player").confirmed);
    assert_eq!(track(&e, "other").evidence, 0);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(
        e.simulation.npcs["guard"].targeting.reason,
        Reason::AwaitingConfirmation
    );
    p.insert("other".into(), [0., 0.85, -1.]);
    sense(&mut e, &p, 2);
    assert!(track(&e, "player").confirmed);
    assert!(!track(&e, "other").confirmed);
    assert_eq!(
        e.execute(Command::InspectTargeting.into()).unwrap()["npcs"]["guard"]["selection"]["observed_actor"],
        "player"
    );
    sense(&mut e, &p, 3);
    sense(&mut e, &p, 4);
    assert!(track(&e, "other").confirmed);
    assert_eq!(
        e.execute(Command::InspectTargeting.into()).unwrap()["npcs"]["guard"]["selection"]["observed_actor"],
        "other"
    );
    e.scene
        .entities
        .iter_mut()
        .find(|x| x.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .enabled = false;
    sense(&mut e, &p, 5);
    let knowledge = &e.simulation.npcs["guard"].knowledge;
    assert_eq!(knowledge.threshold, 0);
    assert!(
        knowledge
            .tracks
            .iter()
            .all(|t| *t == ignis::recognition::Track::default())
    );
    assert!(e.simulation.npcs["guard"].memory.is_none());
}
#[test]
fn absence_decays_without_hidden_position_updates_and_confirmed_latches_until_zero() {
    let mut a = Engine::new(scene()).unwrap();
    let mut b = Engine::new(scene()).unwrap();
    let mut pa = a.simulation.positions();
    let mut pb = pa.clone();
    for tick in 0..3 {
        sense(&mut a, &pa, tick);
        sense(&mut b, &pb, tick);
    }
    let known = track(&a, "player").last_seen;
    let memory = a.simulation.npcs["guard"].memory;
    for tick in 3..8 {
        pa.insert("player".into(), [0., 0.85, 4.]);
        pa.insert("other".into(), [1., 0.85, 5.]);
        pb.insert("player".into(), [tick as f32, 0.85, 6.]);
        pb.insert("other".into(), [-4., 0.85, 8.]);
        sense(&mut a, &pa, tick);
        sense(&mut b, &pb, tick);
        assert_eq!(a.simulation.npcs["guard"], b.simulation.npcs["guard"]);
        assert_eq!(a.simulation.npcs["guard"].memory, memory);
        if tick < 7 {
            assert!(track(&a, "player").confirmed);
            assert_eq!(track(&a, "player").last_seen, known);
        }
    }
    assert_eq!(track(&a, "player"), Default::default());
    pa.insert("player".into(), [-2., 0.85, -4.]);
    sense(&mut a, &pa, 8);
    assert!(!track(&a, "player").confirmed);
    assert_eq!(track(&a, "player").evidence, 5);
    // Previous confirmed pursuit memory remains a separate last-known-location budget.
    assert_eq!(a.simulation.npcs["guard"].memory, memory);
}
#[test]
fn partial_exposure_recovers_and_fixed_point_extremes_do_not_drift() {
    let config = Config {
        acquire_ticks: 3,
        decay_ticks: 5,
    };
    let mut k = Knowledge::default();
    let o = perception::Observation {
        instance: None,
        life: Some(0),
        position: [0.; 3],
        tick: 0,
        source: Source::Vision,
        noise_id: None,
        actor_index: Some(0),
    };
    for _ in 0..2 {
        k.begin();
        assert!(!k.observe(Some(config), o));
        k.finish(Some(config));
    }
    k.begin();
    k.finish(Some(config));
    assert_eq!(k.tracks[0].evidence, 7);
    k.begin();
    assert!(!k.observe(Some(config), o));
    k.finish(Some(config));
    k.begin();
    assert!(k.observe(Some(config), o));
    for config in [
        Config {
            acquire_ticks: 1,
            decay_ticks: 1,
        },
        Config {
            acquire_ticks: 3600,
            decay_ticks: 3600,
        },
    ] {
        let mut k = Knowledge::default();
        for i in 1..=config.acquire_ticks {
            k.begin();
            assert_eq!(k.observe(Some(config), o), i == config.acquire_ticks);
            k.finish(Some(config));
        }
        assert_eq!(k.tracks[0].evidence, config.threshold());
        for _ in 0..config.decay_ticks {
            k.begin();
            k.finish(Some(config));
        }
        assert_eq!(k.tracks[0], Default::default());
    }
}
#[test]
fn explicit_target_waits_legacy_is_immediate_and_omniscience_bypasses_wait() {
    let mut s = scene();
    let n = s
        .entities
        .iter_mut()
        .find(|x| x.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    n.targeting = None;
    n.target = Some("player".into());
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(2).unwrap();
    assert!(e.simulation.npcs["guard"].memory.is_none());
    e.advance(1).unwrap();
    assert!(e.simulation.npcs["guard"].observed_this_tick);
    s.entities
        .iter_mut()
        .find(|x| x.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .recognition = None;
    let mut legacy = Engine::new(s.clone()).unwrap();
    legacy.advance(1).unwrap();
    assert!(legacy.simulation.npcs["guard"].observed_this_tick);
    assert_eq!(
        legacy.simulation.npcs["guard"].knowledge,
        Knowledge::default()
    );
    let n = s
        .entities
        .iter_mut()
        .find(|x| x.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    n.recognition = Some(Config {
        acquire_ticks: 300,
        decay_ticks: 300,
    });
    n.awareness = Some(Awareness::Omniscient);
    let mut omni = Engine::new(s).unwrap();
    omni.advance(1).unwrap();
    assert!(track(&omni, "player").confirmed);
    assert_eq!(
        omni.simulation.npcs["guard"].memory.unwrap().source,
        Source::Omniscient
    );
}
#[test]
fn occlusion_prevents_accumulation_and_sound_does_not_identify_or_confirm() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"screen","name":"screen","body":"fixed","color":[0.2,0.6,0.4],"transform":{"position":[0,1.5,-1],"size":[10,3,0.3]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(5).unwrap();
    let knowledge = &e.simulation.npcs["guard"].knowledge;
    assert_eq!(knowledge.threshold, 0);
    assert!(
        knowledge
            .tracks
            .iter()
            .all(|t| *t == ignis::recognition::Track::default())
    );
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Entity {
                id: "player".into(),
            },
            radius: 12.,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["guard"].memory.unwrap().source,
        Source::Sound
    );
    assert_eq!(e.simulation.npcs["guard"].memory.unwrap().actor_index, None);
    let knowledge = &e.simulation.npcs["guard"].knowledge;
    assert_eq!(knowledge.threshold, 0);
    assert!(
        knowledge
            .tracks
            .iter()
            .all(|t| *t == ignis::recognition::Track::default())
    );
    e.scene
        .entities
        .iter_mut()
        .find(|x| x.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .enabled = false;
    let p = e.simulation.positions();
    sense(&mut e, &p, 7);
    let knowledge = &e.simulation.npcs["guard"].knowledge;
    assert_eq!(knowledge.threshold, 0);
    assert!(
        knowledge
            .tracks
            .iter()
            .all(|t| *t == ignis::recognition::Track::default())
    );
    assert!(e.simulation.npcs["guard"].memory.is_none());
}
#[test]
fn patrol_continues_while_suspicious_and_suspends_after_confirmation() {
    let mut s = scene();
    let n = s
        .entities
        .iter_mut()
        .find(|x| x.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    n.recognition = Some(Config {
        acquire_ticks: 30,
        decay_ticks: 60,
    });
    if let Some(Awareness::Perceptive { fov_degrees, .. }) = &mut n.awareness {
        *fov_degrees = 360.;
    }
    n.patrol=Some(serde_json::from_value(json!({"route":{"kind":"points","points":[[3,0],[-3,0]]},"traversal":"loop","wait_ticks":0,"travel_ticks":600,"retry_ticks":30,"arrival_distance":0.4})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(29).unwrap();
    assert_eq!(
        e.simulation.patrols["guard"].phase,
        ignis::patrol::Phase::Moving
    );
    assert!(e.simulation.npcs["guard"].memory.is_none());
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.patrols["guard"].phase,
        ignis::patrol::Phase::Suspended
    );
    assert!(e.simulation.npcs["guard"].observed_this_tick);
}
#[test]
fn inspection_replay_order_reset_and_edits_preserve_contracts() {
    let s = scene();
    let mut a = Engine::new(s.clone()).unwrap();
    let mut reversed = s;
    reversed.entities.reverse();
    let mut b = Engine::new(reversed).unwrap();
    a.advance(20).unwrap();
    b.advance(20).unwrap();
    assert_eq!(a.simulation.npcs, b.simulation.npcs);
    let report = a.execute(Command::InspectKnowledge.into()).unwrap();
    assert_eq!(report["npcs"]["guard"]["tracks"][0]["progress"], 1.);
    assert_eq!(report["npcs"]["guard"]["tracks"][0]["actor_id"], "other");
    let replay = a.replay();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let prefix = Engine::replay_at(&replay, Some(2)).unwrap();
    assert!(!track(&prefix, "player").confirmed);
    assert_eq!(track(&prefix, "player").evidence, 10);
    let mut bad = replay;
    bad.expected_npcs.get_mut("guard").unwrap().knowledge.tracks[0].evidence += 1;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["npcs_matched"], false);
    let mut guard = a
        .scene
        .entities
        .iter()
        .find(|e| e.id == "guard")
        .unwrap()
        .clone();
    guard.npc.as_mut().unwrap().recognition = None;
    a.execute(
        Command::Upsert {
            entity: guard.into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(a.simulation.npcs["guard"].knowledge, Knowledge::default());
    a.execute(Command::Undo.into()).unwrap();
    a.advance(2).unwrap();
    assert!(!track(&a, "player").confirmed);
    a.execute(Command::Reset.into()).unwrap();
    assert_eq!(a.simulation.npcs["guard"].knowledge, Knowledge::default());
    a.execute(Command::Redo.into()).unwrap();
    a.advance(1).unwrap();
    assert!(a.simulation.npcs["guard"].observed_this_tick);
}
#[test]
fn validation_is_atomic_and_old_scenes_do_not_silently_enable_recognition() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(2).unwrap();
    let before = e.snapshot();
    for (acquire, decay) in [(0, 5), (3, 0), (3601, 5), (3, 3601)] {
        let mut guard = e
            .scene
            .entities
            .iter()
            .find(|x| x.id == "guard")
            .unwrap()
            .clone();
        guard.npc.as_mut().unwrap().recognition = Some(Config {
            acquire_ticks: acquire,
            decay_ticks: decay,
        });
        assert!(
            e.execute(
                Command::Upsert {
                    entity: guard.into()
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut old = scene();
    old.version = 12;
    assert!(Engine::new(old).is_err());
    let legacy: Scene =
        serde_json::from_str(include_str!("../examples/npc-relations.scene.json")).unwrap();
    let e = Engine::new(legacy).unwrap();
    assert_eq!(e.scene.version, ignis::model::SCENE_VERSION);
    assert!(
        e.scene
            .entities
            .iter()
            .filter_map(|e| e.npc.as_ref())
            .all(|n| n.recognition.is_none())
    );
}
