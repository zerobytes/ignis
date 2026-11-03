use ignis::{
    engine::{Command, Engine},
    model::{RoomRecipe, Scene},
    noise::Noise,
    perception,
    territory::{Config, Phase, State, Trigger},
};
use serde_json::json;
use std::collections::BTreeMap;
fn config() -> Config {
    Config {
        home: [0., 0.],
        radius: 5.,
        arrival_distance: 0.3,
        rearm_ticks: 6,
    }
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 24.,
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
        .position = [0., 0.85, -3.];
    let mut npc:ignis::model::Entity=serde_json::from_value(json!({"id":"guard","name":"Guard","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[1,0.85,0],"size":[0.7,1.6,0.7]},"npc":{"target":"player","surface":"floor","speed":2,"stop_distance":1.2,"enabled":true,"awareness":{"mode":"perceptive","vision_range":12,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":30,"search_turn_degrees_per_second":90,"hearing":{"range":12}},"objectives":[{"priority":10,"when":"evidence","action":{"kind":"respond"}},{"priority":1,"when":"always","action":{"kind":"patrol"}}],"patrol":{"route":{"kind":"points","points":[[0,2],[0,-2]]},"traversal":"loop","wait_ticks":10,"travel_ticks":600,"retry_ticks":60,"arrival_distance":0.3}}})).unwrap();
    npc.npc.as_mut().unwrap().territory = Some(config());
    s.entities.push(npc);
    s
}
fn sense(
    e: &mut Engine,
    p: &BTreeMap<String, [f32; 3]>,
    tick: u64,
    noises: &[Noise],
) -> BTreeMap<String, Option<ignis::navigation::NavGoal>> {
    let mut goals = BTreeMap::new();
    perception::update_with_noises(
        &e.scene,
        p,
        tick,
        &e.simulation.navigation,
        &mut e.simulation.npcs,
        &mut goals,
        noises,
    );
    goals
}
fn step(e: &mut Engine, ticks: u32) {
    e.execute(Command::Step { ticks }.into()).unwrap();
}
fn cue(id: u64, x: f32) -> Noise {
    Noise {
        id,
        tick: 0,
        position: [x, 0.85, 0.],
        radius: 12.,
        emitter_index: None,
        emitter_instance: None,
    }
}
#[test]
fn boundaries_cooldown_displacement_and_zero_wait_are_explicit() {
    let c = config();
    assert!(c.contains([3., 100., 4.]));
    assert!(!c.contains([3.001, 0., 4.]));
    let mut s = State::default();
    s.begin(Some(c), [6., 0., 0.], 10);
    assert_eq!(s.last_trigger, Some(Trigger::OwnOutside));
    assert_eq!(s.returns, 1);
    s.begin(Some(c), [0.31, 0., 0.], 11);
    assert_eq!(s.phase, Phase::CoolingDown);
    assert_eq!(s.ready_at_tick, Some(17));
    s.begin(Some(c), [0.5, 0., 0.], 12);
    assert_eq!(s.phase, Phase::Returning);
    assert_eq!(s.last_trigger, Some(Trigger::DisplacedFromHome));
    assert_eq!(s.ready_at_tick, None);
    s.begin(Some(c), [0., 0., 0.], 13);
    s.begin(Some(c), [0., 0., 0.], 18);
    assert!(s.blocked());
    s.begin(Some(c), [0., 0., 0.], 19);
    assert_eq!(s.phase, Phase::Active);
    s.start(Trigger::GoalOutside, 20);
    let mut zero = c;
    zero.rearm_ticks = 0;
    s.begin(Some(zero), [0., 0., 0.], 21);
    assert_eq!(s.phase, Phase::Active);
    s.begin(None, [100., 0., 0.], 22);
    assert_eq!(s, State::default());
}
#[test]
fn observed_target_departure_forces_return_ignores_new_cues_and_rearms_at_home() {
    let mut e = Engine::new(scene()).unwrap();
    let mut p = e.simulation.positions();
    p.insert("player".into(), [4., 0.85, 0.]);
    sense(&mut e, &p, 0, &[]);
    assert!(e.simulation.npcs["guard"].memory.is_some());
    p.insert("player".into(), [6., 0.85, 0.]);
    let goals = sense(&mut e, &p, 1, &[]);
    let n = &e.simulation.npcs["guard"];
    assert_eq!(n.territory.phase, Phase::Returning);
    assert_eq!(n.territory.last_trigger, Some(Trigger::TargetLeft));
    assert_eq!(n.territory.rejected_visual, 1);
    assert!(n.memory.is_none());
    assert_eq!(
        n.objectives.reason,
        ignis::objectives::Reason::TerritoryReturn
    );
    assert_eq!(goals["guard"].as_ref().unwrap().position, [0., 0.85, 0.]);
    p.insert("player".into(), [2., 0.85, 0.]);
    sense(&mut e, &p, 2, &[cue(0, 2.)]);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    p.insert("guard".into(), [0., 0.85, 0.]);
    sense(&mut e, &p, 3, &[]);
    assert_eq!(e.simulation.npcs["guard"].territory.ready_at_tick, Some(9));
    sense(&mut e, &p, 8, &[]);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    sense(&mut e, &p, 9, &[]);
    assert!(e.simulation.npcs["guard"].observed_this_tick);
    assert_eq!(e.simulation.npcs["guard"].territory.phase, Phase::Active);
}
#[test]
fn external_initial_target_cannot_acquire_and_hidden_departure_does_not_leak() {
    let mut e = Engine::new(scene()).unwrap();
    let mut p = e.simulation.positions();
    p.insert("player".into(), [6., 0.85, 0.]);
    sense(&mut e, &p, 0, &[]);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(e.simulation.npcs["guard"].territory.phase, Phase::Active);
    assert_eq!(
        e.simulation.npcs["guard"].targeting.reason,
        ignis::targeting::Reason::OutsideTerritory
    );
    let mut s = scene();
    if let Some(ignis::model::Awareness::Perceptive { fov_degrees, .. }) = &mut s
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *fov_degrees = 90.;
    }
    let mut a = Engine::new(s.clone()).unwrap();
    let mut b = Engine::new(s).unwrap();
    p = a.simulation.positions();
    sense(&mut a, &p, 0, &[]);
    sense(&mut b, &p, 0, &[]);
    p.insert("player".into(), [1., 0.85, 3.]);
    let ga = sense(&mut a, &p, 1, &[]);
    p.insert("player".into(), [1., 0.85, 7.]);
    let gb = sense(&mut b, &p, 1, &[]);
    assert_eq!(a.simulation.npcs["guard"], b.simulation.npcs["guard"]);
    assert_eq!(
        ga["guard"].as_ref().unwrap().position,
        gb["guard"].as_ref().unwrap().position
    );
    assert_eq!(a.simulation.npcs["guard"].territory.phase, Phase::Active);
}
#[test]
fn acoustic_filter_precedes_ranking_and_internal_visual_alternative_prevents_return() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .target = None;
    let mut e = Engine::new(s).unwrap();
    let mut p = e.simulation.positions();
    p.insert("guard".into(), [4., 0.85, 0.]);
    sense(&mut e, &p, 0, &[cue(0, -2.), cue(1, 5.1)]);
    let n = &e.simulation.npcs["guard"];
    assert_eq!(n.memory.unwrap().noise_id, Some(0));
    assert_eq!(n.territory.rejected_sound, 1);
    let mut s = scene();
    let guard = s.entities.iter_mut().find(|e| e.id == "guard").unwrap();
    guard.faction = Some("guards".into());
    let n = guard.npc.as_mut().unwrap();
    n.target = None;
    n.targeting = Some(ignis::targeting::Policy::HostileNearest { switch_margin: 0.2 });
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .faction = Some("hostiles".into());
    let mut other = s.entities.iter().find(|e| e.id == "guard").unwrap().clone();
    other.id = "other".into();
    other.faction = Some("hostiles".into());
    other.npc.as_mut().unwrap().enabled = false;
    s.entities.push(other);
    s.faction_relations = vec![ignis::targeting::Relation {
        from: "guards".into(),
        to: "hostiles".into(),
        attitude: ignis::targeting::Attitude::Hostile,
    }];
    let mut e = Engine::new(s).unwrap();
    let mut p = e.simulation.positions();
    p.insert("player".into(), [2., 0.85, -2.]);
    p.insert("other".into(), [-3., 0.85, 0.]);
    sense(&mut e, &p, 0, &[]);
    let old = e.simulation.npcs["guard"].memory.unwrap().actor_index;
    p.insert("player".into(), [6., 0.85, -2.]);
    sense(&mut e, &p, 1, &[]);
    let n = &e.simulation.npcs["guard"];
    assert!(n.territory.active_target_departed);
    assert_eq!(n.territory.phase, Phase::Active);
    assert_ne!(n.memory.unwrap().actor_index, old);
}
#[test]
fn physical_return_resumes_patrol_and_unreachable_home_never_teleports() {
    let mut s = scene();
    let g = s.entities.iter_mut().find(|e| e.id == "guard").unwrap();
    g.transform.position = [6., 0.85, 0.];
    g.npc.as_mut().unwrap().target = None;
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 1);
    assert_eq!(e.simulation.npcs["guard"].territory.phase, Phase::Returning);
    assert_eq!(
        e.simulation.patrols["guard"].phase,
        ignis::patrol::Phase::Suspended
    );
    assert!(e.simulation.positions()["guard"][0] > 5.9);
    step(&mut e, 200);
    assert_eq!(e.simulation.npcs["guard"].territory.phase, Phase::Active);
    assert_eq!(
        e.simulation.npcs["guard"].objectives.action,
        Some(ignis::objectives::Kind::Patrol)
    );
    assert_eq!(e.simulation.patrols["guard"].point_index, 0);
    s.entities.push(serde_json::from_value(json!({"id":"block","name":"Blocked home","body":"fixed","transform":{"position":[0,1,0],"size":[2,2,2]},"color":[0.2,0.5,0.3]})).unwrap());
    let mut blocked = Engine::new(s).unwrap();
    step(&mut blocked, 30);
    assert_eq!(
        blocked.simulation.npcs["guard"].territory.phase,
        Phase::Returning
    );
    assert_eq!(blocked.simulation.navigation["guard"].status, "unreachable");
    assert!(blocked.simulation.positions()["guard"][0] > 5.9);
}
#[test]
fn known_flee_destination_outside_returns_without_charging_investigation_and_disabled_clears() {
    let mut s = scene();
    let g = s.entities.iter_mut().find(|e| e.id == "guard").unwrap();
    let n = g.npc.as_mut().unwrap();
    n.target = None;
    n.objectives = Some(vec![ignis::objectives::Objective {
        priority: 1,
        when: ignis::objectives::Condition::Evidence,
        action: ignis::objectives::Action::Flee { distance: 10. },
    }]);
    let mut e = Engine::new(s).unwrap();
    let p = e.simulation.positions();
    sense(&mut e, &p, 0, &[cue(0, 2.)]);
    assert_eq!(
        e.simulation.npcs["guard"].territory.last_trigger,
        Some(Trigger::GoalOutside)
    );
    assert!(e.simulation.npcs["guard"].memory.is_none());
    e.scene
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .enabled = false;
    sense(&mut e, &p, 1, &[]);
    assert_eq!(e.simulation.npcs["guard"].territory.phase, Phase::Disabled);
    assert!(!e.simulation.npcs["guard"].territory.blocked());
}
#[test]
fn validation_history_replay_prefix_and_legacy_are_consistent() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .transform
        .position = [6., 0.85, 0.];
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 20);
    let replay = e.replay();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    assert_eq!(
        Engine::replay_at(&replay, Some(1)).unwrap().simulation.npcs["guard"]
            .territory
            .phase,
        Phase::Returning
    );
    let before = e.snapshot();
    let mut bad = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "guard")
        .unwrap()
        .clone();
    bad.npc.as_mut().unwrap().territory.as_mut().unwrap().radius = f32::NAN;
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
    for c in [
        Config {
            radius: 0.4,
            ..config()
        },
        Config {
            arrival_distance: 5.,
            ..config()
        },
        Config {
            home: [10001., 0.],
            ..config()
        },
        Config {
            rearm_ticks: 3601,
            ..config()
        },
        Config {
            radius: 1.,
            ..config()
        },
    ] {
        let mut bad = scene();
        bad.entities
            .iter_mut()
            .find(|e| e.id == "guard")
            .unwrap()
            .npc
            .as_mut()
            .unwrap()
            .territory = Some(c);
        assert!(Engine::new(bad).is_err());
    }
    let mut old = scene();
    old.version = 15;
    assert!(Engine::new(old).is_err());
    let mut legacy = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "guard")
        .unwrap()
        .clone();
    legacy.npc.as_mut().unwrap().territory = None;
    e.execute(
        Command::Upsert {
            entity: Box::new(legacy),
        }
        .into(),
    )
    .unwrap();
    step(&mut e, 1);
    assert_eq!(
        e.simulation.npcs["guard"].territory.phase,
        Phase::Unrestricted
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.simulation.npcs["guard"].territory, State::default());
    e.execute(Command::Redo.into()).unwrap();
    assert!(
        e.scene
            .entities
            .iter()
            .find(|e| e.id == "guard")
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .territory
            .is_none()
    );
}
#[test]
fn route_guides_show_boundary_and_home_without_affecting_simulation() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.replay();
    let frame = e.render_frame();
    assert!(frame.route_guides.is_empty());
    e.execute(
        Command::SetDebugView {
            npc_vision: None,
            npc_hearing: None,
            noise_events: None,
            npc_routes: Some(true),
        }
        .into(),
    )
    .unwrap();
    let frame = e.render_frame();
    assert!(frame.route_guides.len() >= 66);
    assert!(
        frame.route_guides[..64]
            .iter()
            .all(|l| l.color == [1., 0.65, 0.1])
    );
    assert_eq!(e.simulation.npcs, before.expected_npcs);
    e.simulation
        .npcs
        .get_mut("guard")
        .unwrap()
        .territory
        .start(Trigger::OwnOutside, 0);
    let frame = e.render_frame();
    assert!(
        frame.route_guides[..64]
            .iter()
            .all(|l| l.color == [1., 0.25, 0.2])
    );
}
