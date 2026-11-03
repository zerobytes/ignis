use ignis::{
    engine::{Command, Engine, SceneEdit},
    model::{Awareness, Entity, Hearing, RoomRecipe, Scene},
    navigation::NavGoal,
    noise::NoiseOrigin,
    perception::{self, Sensors, Source, Stimuli},
    targeting::{Attitude, Catalog, Policy, Reason, Relation},
};
use serde_json::json;
use std::collections::BTreeMap;
fn actor(id: &str, faction: Option<&str>, position: [f32; 3]) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"npc","faction":faction,"blocks_vision":false,"color":[0.7,0.2,0.8],"transform":{"position":position,"size":[0.7,1.6,0.7]},"npc":{"surface":"floor","speed":2,"stop_distance":1.2,"enabled":false}})).unwrap()
}
fn relation(from: &str, to: &str, attitude: Attitude) -> Relation {
    Relation {
        from: from.into(),
        to: to.into(),
        attitude,
    }
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 20.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut guard = actor("guard", Some("guards"), [0., 0.85, 0.]);
    let n = guard.npc.as_mut().unwrap();
    n.enabled = true;
    n.targeting = Some(Policy::HostileNearest { switch_margin: 0.2 });
    n.awareness = Some(Awareness::Perceptive {
        vision_range: 15.,
        fov_degrees: 120.,
        initial_facing_degrees: 0.,
        memory_ticks: Some(30),
        search_ticks: Some(10),
        search_turn_degrees_per_second: 90.,
        max_investigation_distance: None,
        area_search: None,
        hearing: Some(Hearing { range: 12. }),
    });
    s.entities.extend([
        guard,
        actor("a", Some("raiders"), [-3., 0.85, -4.]),
        actor("b", Some("raiders"), [3., 0.85, -4.]),
        actor("friend", Some("guards"), [0., 0.85, -1.]),
        actor("neutral", Some("town"), [0., 0.85, -2.]),
    ]);
    s.faction_relations = vec![relation("guards", "raiders", Attitude::Hostile)];
    s
}
fn attitude(e: &Engine, from: &str, to: &str) -> Attitude {
    let slot = |id| {
        e.simulation
            .actors
            .slot(e.scene.entities.iter().position(|x| x.id == id).unwrap())
    };
    e.simulation.actors.attitude(slot(from), slot(to))
}
fn sense(
    e: &mut Engine,
    positions: &BTreeMap<String, [f32; 3]>,
    tick: u64,
) -> BTreeMap<String, Option<NavGoal>> {
    let mut goals = BTreeMap::new();
    perception::update_with_context(
        &e.scene,
        positions,
        tick,
        &e.simulation.navigation,
        &mut e.simulation.npcs,
        &mut goals,
        Sensors {
            lives: &[0; ignis::targeting::MAX_ACTORS],
            ability_policy: None,
            dead: ignis::actor_set::ActorSet::EMPTY,
            ability_ready: &[0; ignis::targeting::MAX_ACTORS],
            stimuli: Stimuli {
                abilities: &[],
                manual: &[],
                automatic: &[],
            },
            actors: &e.simulation.actors,
        },
    );
    goals
}
fn observed(e: &Engine) -> Option<&str> {
    e.simulation.npcs["guard"]
        .targeting
        .observed_actor
        .map(|i| e.simulation.actors.id(&e.scene, i))
}
#[test]
fn relations_are_directional_default_neutral_and_explicitly_override_same_faction() {
    let mut e = Engine::new(scene()).unwrap();
    assert_eq!(attitude(&e, "guard", "a"), Attitude::Hostile);
    assert_eq!(attitude(&e, "a", "guard"), Attitude::Neutral);
    assert_eq!(attitude(&e, "guard", "friend"), Attitude::Allied);
    assert_eq!(attitude(&e, "guard", "neutral"), Attitude::Neutral);
    assert_eq!(attitude(&e, "guard", "player"), Attitude::Neutral);
    e.execute(
        Command::SetFactionRelations {
            relations: vec![
                relation("guards", "guards", Attitude::Neutral),
                relation("town", "guards", Attitude::Hostile),
            ],
        }
        .into(),
    )
    .unwrap();
    assert_eq!(attitude(&e, "guard", "friend"), Attitude::Neutral);
    assert_eq!(attitude(&e, "guard", "a"), Attitude::Neutral);
    assert_eq!(attitude(&e, "neutral", "guard"), Attitude::Hostile);
    assert_eq!(attitude(&e, "guard", "neutral"), Attitude::Neutral);
    let report = e.execute(Command::InspectRelations.into()).unwrap();
    assert_eq!(report["actors"].as_array().unwrap().len(), 6);
    assert_eq!(report["resolved"].as_array().unwrap().len(), 30);
}
#[test]
fn acquisition_ties_by_id_and_margin_retains_until_strictly_better_visible_candidate() {
    let mut e = Engine::new(scene()).unwrap();
    let mut p = e.simulation.positions();
    sense(&mut e, &p, 0);
    assert_eq!(observed(&e), Some("a"));
    assert_eq!(e.simulation.npcs["guard"].targeting.observed_candidates, 2);
    p.insert("b".into(), [0., 0.85, -4.5]);
    sense(&mut e, &p, 1);
    assert_eq!(observed(&e), Some("a"));
    assert_eq!(
        e.simulation.npcs["guard"].targeting.reason,
        Reason::Retained
    );
    assert_eq!(
        e.simulation.npcs["guard"]
            .targeting
            .nearest_distance_squared,
        Some(20.25)
    );
    p.insert("b".into(), [0., 0.85, -4.]);
    sense(&mut e, &p, 2);
    assert_eq!(observed(&e), Some("a"));
    p.insert("b".into(), [0., 0.85, -3.]);
    sense(&mut e, &p, 3);
    assert_eq!(observed(&e), Some("b"));
    assert_eq!(
        e.simulation.npcs["guard"].targeting.reason,
        Reason::SwitchedCloser
    );
    let stats = e.simulation.npcs["guard"].targeting;
    assert_eq!(stats.observed_distance_squared, Some(9.));
    assert_eq!(stats.incumbent_distance_squared, Some(25.));
    // Loss is not a hysteresis lock: the visible alternative wins immediately.
    p.insert("b".into(), [0., 0.85, 4.]);
    sense(&mut e, &p, 4);
    assert_eq!(observed(&e), Some("a"));
    assert_eq!(
        e.simulation.npcs["guard"].targeting.reason,
        Reason::PreviousNotObserved
    );
    let inspection = e.execute(Command::InspectTargeting.into()).unwrap();
    assert_eq!(inspection["npcs"]["guard"]["memory"]["actor_id"], "a");
    assert!(
        inspection["npcs"]["guard"]["memory"]
            .get("actor_index")
            .is_none()
    );
}
#[test]
fn hidden_candidates_cannot_affect_choice_or_frozen_memory_but_omniscience_is_explicit() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"screen","name":"Screen","body":"fixed","color":[0.2,0.6,0.3],"transform":{"position":[0,1.5,2],"size":[10,3,0.3]}})).unwrap());
    if let Some(Awareness::Perceptive { fov_degrees, .. }) = &mut s
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *fov_degrees = 360.;
    }
    let mut a = Engine::new(s.clone()).unwrap();
    let mut b = Engine::new(s.clone()).unwrap();
    let mut pa = a.simulation.positions();
    let mut pb = pa.clone();
    sense(&mut a, &pa, 0);
    sense(&mut b, &pb, 0);
    let frozen = a.simulation.npcs["guard"].memory;
    for tick in 1..35 {
        pa.insert("a".into(), [-1., 0.85, 4.]);
        pa.insert("b".into(), [1., 0.85, 4.]);
        pb.insert("a".into(), [-2. + tick as f32 * 0.01, 0.85, 5.]);
        pb.insert("b".into(), [0., 0.85, 6.]);
        let ga = sense(&mut a, &pa, tick);
        let gb = sense(&mut b, &pb, tick);
        assert_eq!(a.simulation.npcs["guard"], b.simulation.npcs["guard"]);
        assert_eq!(
            ga["guard"].as_ref().map(|g| g.position),
            gb["guard"].as_ref().map(|g| g.position)
        );
        if tick < 30 {
            assert_eq!(a.simulation.npcs["guard"].memory, frozen);
        } else {
            assert!(a.simulation.npcs["guard"].memory.is_none());
        }
    }
    // These points are inside a 360-degree FOV but blocked by the wall.
    for e in &mut s.entities {
        if let Some(n) = &mut e.npc
            && e.id == "guard"
        {
            n.awareness = Some(Awareness::Omniscient);
        }
    }
    let mut omni = Engine::new(s).unwrap();
    sense(&mut omni, &pa, 0);
    assert!(observed(&omni).is_some());
    assert_eq!(
        omni.simulation.npcs["guard"]
            .memory
            .as_ref()
            .unwrap()
            .source,
        Source::Omniscient
    );
}
#[test]
fn range_fov_and_wall_filter_before_ranking_even_when_hostile_is_closer() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"screen","name":"Screen","body":"fixed","color":[0.2,0.6,0.3],"transform":{"position":[0,1.5,-1],"size":[1,3,0.2]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    let mut p = e.simulation.positions();
    p.insert("a".into(), [0., 0.85, -2.]);
    sense(&mut e, &p, 0);
    assert_eq!(observed(&e), Some("b"));
    assert_eq!(e.simulation.npcs["guard"].targeting.observed_candidates, 1);
    p.insert("b".into(), [0., 0.85, 1.]);
    sense(&mut e, &p, 1);
    assert_eq!(observed(&e), None);
    p.insert("b".into(), [3., 0.85, -40.]);
    sense(&mut e, &p, 2);
    assert_eq!(observed(&e), None);
}
#[test]
fn sound_remains_anonymous_and_disabled_state_clears_selection_and_memory() {
    let mut s = scene();
    for e in &mut s.entities {
        if e.id == "a" || e.id == "b" {
            e.transform.position[2] = 4.;
        }
    }
    let mut e = Engine::new(s).unwrap();
    e.execute(
        Command::EmitNoise {
            origin: NoiseOrigin::Entity { id: "a".into() },
            radius: 10.,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    let state = &e.simulation.npcs["guard"];
    assert_eq!(state.memory.as_ref().unwrap().source, Source::Sound);
    assert_eq!(state.memory.as_ref().unwrap().actor_index, None);
    assert_eq!(state.targeting.observed_actor, None);
    let snap = e.snapshot();
    assert!(snap["npcs"]["guard"]["memory"]["actor_id"].is_null());
    assert!(
        !snap["npcs"]["guard"]["observed_this_tick"]
            .as_bool()
            .unwrap()
    );
    let mut guard = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "guard")
        .unwrap()
        .clone();
    guard.npc.as_mut().unwrap().enabled = false;
    e.execute(
        Command::Upsert {
            entity: guard.into(),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(
        e.simulation.npcs["guard"].targeting.reason,
        Reason::Disabled
    );
}
#[test]
fn edits_rebuild_catalog_and_undo_replay_and_authoring_order_are_consistent() {
    let s = scene();
    let mut a = Engine::new(s.clone()).unwrap();
    let mut reversed = s;
    reversed.entities.reverse();
    reversed.faction_relations.reverse();
    let mut b = Engine::new(reversed).unwrap();
    a.advance(90).unwrap();
    b.advance(90).unwrap();
    assert_eq!(a.simulation.npcs, b.simulation.npcs);
    assert_eq!(a.simulation.positions(), b.simulation.positions());
    let replay: ignis::engine::Replay =
        serde_json::from_value(serde_json::to_value(a.replay()).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let mut bad = replay;
    bad.expected_npcs.get_mut("guard").unwrap().targeting.reason = Reason::Disabled;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["npcs_matched"], false);
    a.execute(Command::SetFactionRelations { relations: vec![] }.into())
        .unwrap();
    a.advance(1).unwrap();
    assert_eq!(observed(&a), None);
    a.execute(Command::Undo.into()).unwrap();
    a.advance(1).unwrap();
    assert_eq!(observed(&a), Some("a"));
    a.execute(Command::Redo.into()).unwrap();
    a.advance(1).unwrap();
    assert_eq!(observed(&a), None);
    // A batch can change faction and rules together, with one rebuilt catalog.
    let mut guard = a
        .scene
        .entities
        .iter()
        .find(|x| x.id == "guard")
        .unwrap()
        .clone();
    guard.faction = Some("new_guards".into());
    a.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Upsert {
                    entity: guard.into(),
                },
                SceneEdit::SetFactionRelations {
                    relations: vec![relation("new_guards", "raiders", Attitude::Hostile)],
                },
            ],
        }
        .into(),
    )
    .unwrap();
    a.advance(1).unwrap();
    assert_eq!(observed(&a), Some("a"));
}
#[test]
fn validation_bounds_and_legacy_fields_are_atomic() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(10).unwrap();
    e.playing = true;
    let before = e.snapshot();
    for rules in [
        vec![relation("guards", "raiders", Attitude::Hostile); 2],
        vec![relation("", "raiders", Attitude::Hostile)],
        (0..65)
            .map(|i| relation("guards", &format!("f{i}"), Attitude::Hostile))
            .collect(),
        (0..32)
            .map(|i| relation("extra", &format!("f{i}"), Attitude::Hostile))
            .collect(),
    ] {
        assert!(
            e.execute(Command::SetFactionRelations { relations: rules }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    for margin in [-0.01, 1., f32::NAN] {
        let mut bad = e
            .scene
            .entities
            .iter()
            .find(|x| x.id == "guard")
            .unwrap()
            .clone();
        bad.npc.as_mut().unwrap().targeting = Some(Policy::HostileNearest {
            switch_margin: margin,
        });
        assert!(
            e.execute(Command::Upsert { entity: bad.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut conflicting = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "guard")
        .unwrap()
        .clone();
    conflicting.npc.as_mut().unwrap().target = Some("player".into());
    assert!(
        e.execute(
            Command::Upsert {
                entity: conflicting.into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut floor = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "floor")
        .unwrap()
        .clone();
    floor.faction = Some("guards".into());
    assert!(
        e.execute(
            Command::Upsert {
                entity: floor.into()
            }
            .into()
        )
        .is_err()
    );
    let mut legacy = scene();
    legacy.version = 11;
    assert!(legacy.validate().is_err());
    let old: Scene =
        serde_json::from_str(include_str!("../examples/npc-autonomy.scene.json")).unwrap();
    assert!(Engine::new(old).is_ok());
}
#[test]
fn maximum_catalog_is_bounded_and_explicit_target_ignores_automatic_hostility_filter() {
    let mut s = scene();
    for i in 0..3 {
        s.entities.push(actor(
            &format!("extra{i}"),
            Some("raiders"),
            [6., 0.85, -4. + i as f32],
        ));
    }
    let e = Engine::new(s).unwrap();
    assert_eq!(e.simulation.actors.actors.len(), 9);
    let c = Catalog::new(&e.scene);
    assert_eq!(c.actors.len(), 9);
    let mut s = scene();
    let g = s
        .entities
        .iter_mut()
        .find(|x| x.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    g.targeting = None;
    g.target = Some("neutral".into());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(observed(&e), Some("neutral"));
    assert_eq!(
        e.simulation.npcs["guard"].targeting.reason,
        Reason::ExplicitObserved
    );
}
