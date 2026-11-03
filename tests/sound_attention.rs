use ignis::{
    engine::{Command, Engine},
    model::{Awareness, RoomRecipe, Scene},
    noise::{Noise, NoiseOrigin},
    perception::{self, Sensors, Source, Stimuli},
    sound_attention::{Config, Reason},
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
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [0., 0.85, 5.];
    s.entities.push(serde_json::from_value(json!({"id":"guard","name":"guard","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[0,0.85,0],"size":[0.7,1.6,0.7]},"npc":{"surface":"floor","speed":1,"stop_distance":1.2,"enabled":true,"awareness":{"mode":"perceptive","vision_range":12,"fov_degrees":90,"initial_facing_degrees":0,"memory_ticks":30,"search_ticks":3,"search_turn_degrees_per_second":90,"hearing":{"range":10}},"sound_attention":{"commitment_ticks":3,"switch_margin":0.1,"repeat_radius":1,"visual_priority_ticks":3}}})).unwrap());
    s
}
fn cue(id: u64, tick: u64, x: f32) -> Noise {
    Noise {
        id,
        tick,
        position: [x, 0.85, 0.],
        radius: 10.,
        emitter_index: None,
        emitter_instance: None,
    }
}
fn sense(
    e: &mut Engine,
    p: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    manual: &[Noise],
    automatic: &[Noise],
) {
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
                manual,
                automatic,
                abilities: &[],
            },
        },
    );
}
#[test]
fn commitment_margin_and_repetition_preserve_memory_and_travel_budget() {
    let mut e = Engine::new(scene()).unwrap();
    let p = e.simulation.positions();
    sense(&mut e, &p, 0, &[cue(0, 0, 6.)], &[]);
    let first = e.simulation.npcs["guard"].memory;
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.reason,
        Reason::Acquired
    );
    e.simulation
        .npcs
        .get_mut("guard")
        .unwrap()
        .investigation_distance = 2.;
    sense(&mut e, &p, 1, &[cue(1, 1, -2.)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"]
            .sound_attention
            .commitment_rejections,
        1
    );
    sense(&mut e, &p, 2, &[cue(2, 2, 6.2)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"]
            .sound_attention
            .repeated_rejections,
        1
    );
    sense(&mut e, &p, 3, &[cue(3, 3, -6.5)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.margin_rejections,
        1
    );
    assert_eq!(e.simulation.npcs["guard"].memory, first);
    assert_eq!(e.simulation.npcs["guard"].investigation_distance, 2.);
    sense(&mut e, &p, 4, &[cue(4, 4, -2.)], &[]);
    let s = &e.simulation.npcs["guard"];
    assert_eq!(s.sound_attention.reason, Reason::ReplacedSound);
    assert_eq!(s.memory.unwrap().noise_id, Some(4));
    assert_eq!(s.memory.unwrap().actor_index, None);
    assert_eq!(s.investigation_distance, 0.);
}
#[test]
fn suppressed_strongest_does_not_hide_weaker_eligible_alternative_and_receipt_is_frozen() {
    let mut e = Engine::new(scene()).unwrap();
    let mut p = e.simulation.positions();
    sense(&mut e, &p, 0, &[cue(0, 0, 6.)], &[]);
    p.insert("guard".into(), [5., 0.85, 0.]);
    sense(&mut e, &p, 4, &[cue(1, 4, 6.05), cue(2, 4, 2.5)], &[]);
    let a = e.simulation.npcs["guard"].sound_attention;
    assert_eq!(a.strongest_audible_id, Some(1));
    assert_eq!(a.selected_noise_id, Some(2));
    assert_eq!(a.repeated_rejections, 1);
    assert_eq!(a.selectable_candidates, 1);
    assert_eq!(a.reason, Reason::ReplacedSound);
    assert!((a.active_strength.unwrap() - 0.75).abs() < 0.00001);
}
#[test]
fn visual_observation_wins_and_visual_memory_has_a_finite_priority_window() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .target = Some("player".into());
    let mut e = Engine::new(s).unwrap();
    let mut p = e.simulation.positions();
    p.insert("player".into(), [0., 0.85, -4.]);
    sense(&mut e, &p, 0, &[cue(0, 0, 1.)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.reason,
        Reason::VisualPriority
    );
    assert_eq!(
        e.simulation.npcs["guard"]
            .sound_attention
            .audible_candidates,
        0
    );
    let visual = e.simulation.npcs["guard"].memory;
    p.insert("player".into(), [0., 0.85, 5.]);
    sense(&mut e, &p, 1, &[cue(1, 1, 1.)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"]
            .sound_attention
            .visual_memory_rejections,
        1
    );
    assert_eq!(e.simulation.npcs["guard"].memory, visual);
    sense(&mut e, &p, 3, &[cue(2, 3, 1.)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.reason,
        Reason::ReplacedVisualMemory
    );
    assert_eq!(
        e.simulation.npcs["guard"].memory.unwrap().source,
        Source::Sound
    );
    p.insert("player".into(), [0., 0.85, -4.]);
    sense(&mut e, &p, 4, &[cue(3, 4, 1.)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.active_strength,
        None
    );
}
#[test]
fn expiry_search_completion_disable_and_distance_forget_clear_active_receipt() {
    for search in [false, true] {
        let mut e = Engine::new(scene()).unwrap();
        let p = e.simulation.positions();
        sense(&mut e, &p, 0, &[cue(0, 0, 1.)], &[]);
        if search {
            e.simulation
                .npcs
                .get_mut("guard")
                .unwrap()
                .search_started_tick = Some(0);
        }
        sense(
            &mut e,
            &p,
            if search { 3 } else { 30 },
            &[cue(1, if search { 3 } else { 30 }, 8.)],
            &[],
        );
        assert_eq!(
            e.simulation.npcs["guard"].sound_attention.reason,
            Reason::Acquired
        );
        assert_eq!(e.simulation.npcs["guard"].memory.unwrap().noise_id, Some(1));
        assert!(
            e.simulation
                .npcs
                .get_mut("guard")
                .unwrap()
                .record_travel([1., 0.], Some(0.5), 31)
        );
        assert_eq!(
            e.simulation.npcs["guard"].sound_attention.active_strength,
            None
        );
        sense(&mut e, &p, 32, &[cue(2, 32, 2.)], &[]);
        e.scene
            .entities
            .iter_mut()
            .find(|e| e.id == "guard")
            .unwrap()
            .npc
            .as_mut()
            .unwrap()
            .enabled = false;
        sense(&mut e, &p, 33, &[cue(3, 33, 2.)], &[]);
        assert_eq!(
            e.simulation.npcs["guard"].sound_attention.reason,
            Reason::Disabled
        );
        assert_eq!(
            e.simulation.npcs["guard"].sound_attention.active_strength,
            None
        );
    }
}
#[test]
fn legacy_equal_strength_order_and_automatic_self_exclusion_remain_intact() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .sound_attention = None;
    let mut e = Engine::new(s).unwrap();
    let p = e.simulation.positions();
    let mut own = cue(900, 0, 0.);
    own.emitter_index = Some(
        e.scene
            .entities
            .iter()
            .position(|e| e.id == "guard")
            .unwrap() as u16,
    );
    own.emitter_instance = e
        .simulation
        .actors
        .entity_reference(own.emitter_index.unwrap() as usize);
    sense(
        &mut e,
        &p,
        0,
        &[cue(1, 0, 4.), cue(2, 0, -4.)],
        &[own, cue(800, 0, 4.)],
    );
    assert_eq!(e.simulation.npcs["guard"].memory.unwrap().noise_id, Some(2));
    assert_eq!(
        e.simulation.npcs["guard"]
            .sound_attention
            .audible_candidates,
        3
    );
    sense(&mut e, &p, 1, &[cue(3, 1, 9.)], &[]);
    assert_eq!(e.simulation.npcs["guard"].memory.unwrap().noise_id, Some(3));
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.reason,
        Reason::LegacyStrongest
    );
}
#[test]
fn exact_margin_is_rejected_and_rejected_events_are_not_revisited_later() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .sound_attention = Some(Config {
        commitment_ticks: 0,
        switch_margin: 0.25,
        repeat_radius: 0.,
        visual_priority_ticks: 0,
    });
    let mut e = Engine::new(s).unwrap();
    let p = e.simulation.positions();
    sense(&mut e, &p, 0, &[cue(0, 0, 5.)], &[]);
    sense(&mut e, &p, 1, &[cue(1, 1, -2.5)], &[]);
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.margin_rejections,
        1
    );
    sense(&mut e, &p, 30, &[], &[]);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(
        e.simulation.npcs["guard"].sound_attention.reason,
        Reason::NoStimuli
    );
}
#[test]
fn edits_replay_inspection_and_authoring_order_preserve_acoustic_decisions() {
    let s = scene();
    let mut a = Engine::new(s.clone()).unwrap();
    let mut rev = s;
    rev.entities.reverse();
    let mut b = Engine::new(rev).unwrap();
    for e in [&mut a, &mut b] {
        for (x, ticks) in [(6., 1), (-2., 1), (6.1, 3), (-1., 3)] {
            e.execute(
                Command::EmitNoise {
                    origin: NoiseOrigin::Position {
                        position: [x, 0.85, 0.],
                    },
                    radius: 10.,
                }
                .into(),
            )
            .unwrap();
            e.advance(ticks).unwrap();
        }
    }
    assert_eq!(a.simulation.npcs, b.simulation.npcs);
    assert_eq!(a.simulation.positions(), b.simulation.positions());
    let r = a.execute(Command::InspectHearing.into()).unwrap();
    assert!(r["npcs"]["guard"]["attention"]["active_strength"].is_number());
    let replay = a.replay();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let prefix = Engine::replay_at(&replay, Some(2)).unwrap();
    assert_eq!(
        prefix.simulation.npcs["guard"]
            .sound_attention
            .commitment_rejections,
        1
    );
    let mut tampered = replay;
    tampered
        .expected_npcs
        .get_mut("guard")
        .unwrap()
        .sound_attention
        .active_strength = Some(0.);
    assert_eq!(
        Engine::verify_replay(&tampered).unwrap()["npcs_matched"],
        false
    );
    let mut guard = a
        .scene
        .entities
        .iter()
        .find(|e| e.id == "guard")
        .unwrap()
        .clone();
    guard.npc.as_mut().unwrap().sound_attention = None;
    a.execute(
        Command::Upsert {
            entity: guard.into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(
        a.simulation.npcs["guard"].sound_attention.active_noise_id,
        None
    );
    a.execute(Command::Undo.into()).unwrap();
    assert!(
        a.scene
            .entities
            .iter()
            .find(|e| e.id == "guard")
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .sound_attention
            .is_some()
    );
    a.execute(Command::Redo.into()).unwrap();
    assert!(
        a.scene
            .entities
            .iter()
            .find(|e| e.id == "guard")
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .sound_attention
            .is_none()
    );
}
#[test]
fn invalid_configs_are_atomic_and_legacy_schema_rejects_new_policy() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let before = e.snapshot();
    let base = Config {
        commitment_ticks: 0,
        switch_margin: 0.,
        repeat_radius: 0.,
        visual_priority_ticks: 0,
    };
    for config in [
        Config {
            commitment_ticks: 3601,
            ..base
        },
        Config {
            visual_priority_ticks: 3601,
            ..base
        },
        Config {
            switch_margin: 1.1,
            ..base
        },
        Config {
            switch_margin: f32::NAN,
            ..base
        },
        Config {
            repeat_radius: -0.1,
            ..base
        },
        Config {
            repeat_radius: 21.,
            ..base
        },
    ] {
        let mut g = e
            .scene
            .entities
            .iter()
            .find(|e| e.id == "guard")
            .unwrap()
            .clone();
        g.npc.as_mut().unwrap().sound_attention = Some(config);
        assert!(
            e.execute(Command::Upsert { entity: g.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut s = scene();
    s.version = 13;
    assert!(Engine::new(s).is_err());
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness = Some(Awareness::Omniscient);
    assert!(Engine::new(s).is_err());
}
