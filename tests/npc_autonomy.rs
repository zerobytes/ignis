use ignis::{
    engine::{Command, Engine, SceneEdit},
    model::{Awareness, Entity, Hearing, RoomRecipe, SCENE_VERSION, Scene},
    navigation,
    noise::NoiseOrigin,
    perception::{self, Source},
};
use serde_json::json;
use std::collections::BTreeMap;
fn policy() -> Awareness {
    Awareness::Perceptive {
        vision_range: 20.,
        fov_degrees: 360.,
        initial_facing_degrees: 0.,
        memory_ticks: Some(60),
        search_ticks: Some(30),
        max_investigation_distance: None,
        area_search: None,
        hearing: Some(Hearing { range: 12. }),
        search_turn_degrees_per_second: 90.,
    }
}
fn actor(id: &str, target: Option<&str>, position: [f32; 3]) -> Entity {
    let mut e:Entity=serde_json::from_value(json!({"id":id,"name":id,"body":"npc","transform":{"position":position,"size":[0.7,1.6,0.7]},"color":[0.7,0.2,0.8],"npc":{"target":target,"surface":"floor","speed":2,"stop_distance":1.2,"enabled":true}})).unwrap();
    e.npc.as_mut().unwrap().awareness = Some(policy());
    e
}
fn room() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.retain(|e| e.id != "player");
    s
}
fn routine(e: &mut Entity) {
    e.npc.as_mut().unwrap().patrol=Some(serde_json::from_value(json!({"route":{"kind":"points","points":[[-3,2],[3,2]]},"traversal":"loop","wait_ticks":5,"travel_ticks":1200,"retry_ticks":60,"arrival_distance":0.2})).unwrap());
}
#[test]
fn no_player_or_target_needed_for_patrol_sound_investigation_and_resume() {
    let mut s = room();
    let mut guard = actor("guard", None, [0., 0.85, 2.]);
    routine(&mut guard);
    s.entities.push(guard);
    let mut e = Engine::new(s).unwrap();
    e.advance(20).unwrap();
    let index = e.simulation.patrols["guard"].point_index;
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(
        e.simulation.patrols["guard"].phase,
        ignis::patrol::Phase::Moving
    );
    e.execute(
        Command::EmitNoise {
            origin: NoiseOrigin::Position {
                position: [0., 0.8, 4.],
            },
            radius: 10.,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["guard"].memory.as_ref().unwrap().source,
        Source::Sound
    );
    assert!(!e.simulation.npcs["guard"].observed_this_tick);
    assert_eq!(
        e.simulation.patrols["guard"].phase,
        ignis::patrol::Phase::Suspended
    );
    e.advance(65).unwrap();
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(e.simulation.patrols["guard"].point_index, index);
    assert_eq!(
        e.simulation.patrols["guard"].phase,
        ignis::patrol::Phase::Moving
    );
    e.advance(500).unwrap();
    assert!(e.simulation.patrols["guard"].visited > 0);
    let replay: ignis::engine::Replay =
        serde_json::from_value(serde_json::to_value(e.replay()).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
}
#[test]
fn absent_visual_target_does_not_implicitly_select_player_even_when_omniscient() {
    for awareness in [policy(), Awareness::Omniscient] {
        let mut s = RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let mut guard = actor("guard", None, [0., 0.85, 2.]);
        guard.npc.as_mut().unwrap().awareness = Some(awareness);
        s.entities.push(guard);
        let mut e = Engine::new(s).unwrap();
        e.advance(120).unwrap();
        assert!(e.simulation.npcs["guard"].memory.is_none());
        assert_eq!(e.simulation.npcs["guard"].phase, perception::Phase::Unaware);
        assert_eq!(e.simulation.navigation["guard"].target_position, None);
    }
}
#[test]
fn mutual_targets_share_pre_step_observations_and_ignore_authoring_order() {
    let mut s = room();
    s.entities.extend([
        actor("a", Some("b"), [-3., 0.85, 0.]),
        actor("b", Some("a"), [3., 0.85, 0.]),
    ]);
    let mut a = Engine::new(s.clone()).unwrap();
    s.entities.reverse();
    let mut b = Engine::new(s).unwrap();
    for _ in 0..180 {
        let positions = a.simulation.positions();
        a.advance(1).unwrap();
        b.advance(1).unwrap();
        assert_eq!(
            a.simulation.npcs["a"].memory.as_ref().unwrap().position,
            positions["b"]
        );
        assert_eq!(
            a.simulation.npcs["b"].memory.as_ref().unwrap().position,
            positions["a"]
        );
        assert_eq!(a.simulation.positions(), b.simulation.positions());
        assert_eq!(a.simulation.npcs, b.simulation.npcs);
    }
    assert!(a.simulation.position("a").unwrap()[0] > -2.);
    assert_eq!(Engine::verify_replay(&a.replay()).unwrap()["matched"], true);
}
#[test]
fn hidden_npc_motion_cannot_change_frozen_memory_or_navigation() {
    let mut s = room();
    let hunter = actor("hunter", Some("prey"), [0., 0.85, -4.]);
    let mut prey = actor("prey", None, [0., 0.85, -2.]);
    prey.npc.as_mut().unwrap().enabled = false;
    s.entities.extend([hunter,prey,serde_json::from_value(json!({"id":"screen","name":"Screen","body":"fixed","transform":{"position":[0,1,0],"size":[6,2,0.3]},"color":[0.2,0.6,0.3]})).unwrap()]);
    let e = Engine::new(s.clone()).unwrap();
    let mut a = e.simulation.npcs.clone();
    let mut b = a.clone();
    let mut na = e.simulation.navigation.clone();
    let mut nb = na.clone();
    let mut pa: BTreeMap<_, _> = s
        .entities
        .iter()
        .map(|e| (e.id.clone(), e.transform.position))
        .collect();
    let mut pb = pa.clone();
    let (mut ga, mut gb, mut va, mut vb) = (
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    );
    let (mut sa, mut sb) = (
        navigation::NavigationScratch::default(),
        navigation::NavigationScratch::default(),
    );
    for tick in 0..90 {
        if tick > 0 {
            pa.insert("prey".into(), [-1., 0.85, 3.]);
            pb.insert("prey".into(), [1. + (tick % 10) as f32 * 0.03, 0.85, 4.]);
        }
        perception::update(&s, &pa, tick, &na, &mut a, &mut ga);
        perception::update(&s, &pb, tick, &nb, &mut b, &mut gb);
        navigation::update(&s, &pa, tick, &ga, &mut na, &mut va, &mut sa);
        navigation::update(&s, &pb, tick, &gb, &mut nb, &mut vb, &mut sb);
        assert_eq!(a, b);
        assert_eq!(na, nb);
        assert_eq!(va, vb);
        if let Some(memory) = &a["hunter"].memory {
            assert_eq!(memory.position, [0., 0.85, -2.]);
            assert_eq!(memory.tick, 0);
        }
    }
    assert!(a["hunter"].memory.is_none());
}
#[test]
fn optional_targets_are_versioned_and_invalid_references_are_atomic() {
    let old: Scene = serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap();
    let migrated = Engine::new(old.clone()).unwrap();
    assert_eq!(migrated.scene.version, SCENE_VERSION);
    assert_eq!(
        migrated
            .scene
            .entities
            .iter()
            .find(|e| e.id == "hunter")
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .target
            .as_deref(),
        Some("player")
    );
    let mut legacy = old;
    // MCP verifiers may resave the example with the current version. Pin the contract under test.
    legacy.version = 10;
    legacy
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .target = None;
    assert!(legacy.validate().is_err());
    let mut s = room();
    s.entities.extend([
        actor("hunter", Some("prey"), [-3., 0.85, 0.]),
        actor("prey", None, [3., 0.85, 0.]),
    ]);
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(20).unwrap();
    e.playing = true;
    let before = e.snapshot();
    for id in ["hunter", "floor", "absent"] {
        let mut bad = e
            .scene
            .entities
            .iter()
            .find(|x| x.id == "hunter")
            .unwrap()
            .clone();
        bad.npc.as_mut().unwrap().target = Some(id.into());
        assert!(
            e.execute(Command::Upsert { entity: bad.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    assert!(
        e.execute(Command::Remove { id: "prey".into() }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    s.version = 10;
    assert!(s.validate().is_err());
    let mut hunter = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "hunter")
        .unwrap()
        .clone();
    hunter.npc.as_mut().unwrap().target = None;
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Remove { id: "prey".into() },
                SceneEdit::Upsert {
                    entity: hunter.into(),
                },
            ],
        }
        .into(),
    )
    .unwrap();
    e.advance(10).unwrap();
    assert!(e.simulation.npcs["hunter"].memory.is_none());
    e.execute(Command::Undo.into()).unwrap();
    e.advance(1).unwrap();
    assert!(e.simulation.npcs["hunter"].observed_this_tick);
}
#[test]
fn anonymous_sound_navigation_does_not_depend_on_unobserved_target_dimensions() {
    let mut s = room();
    let hunter = actor("hunter", Some("prey"), [0., 0.85, 2.]);
    let mut prey = actor("prey", None, [0., 0.85, -4.]);
    prey.npc.as_mut().unwrap().enabled = false;
    s.entities.extend([hunter,prey,serde_json::from_value(json!({"id":"screen","name":"Screen","body":"fixed","transform":{"position":[0,2,0],"size":[8,4,0.3]},"color":[0.2,0.6,0.3]})).unwrap()]);
    let mut a = Engine::new(s.clone()).unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "prey").unwrap();
    p.transform.size[1] = 3.;
    p.transform.position[1] = 1.5;
    let mut b = Engine::new(s).unwrap();
    for e in [&mut a, &mut b] {
        e.execute(
            Command::EmitNoise {
                origin: NoiseOrigin::Position {
                    position: [2., 0.8, 3.],
                },
                radius: 10.,
            }
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
    }
    assert_eq!(a.simulation.npcs["hunter"], b.simulation.npcs["hunter"]);
    assert_eq!(
        a.simulation.navigation["hunter"],
        b.simulation.navigation["hunter"]
    );
}
