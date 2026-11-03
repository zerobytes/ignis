use ignis::{
    communication::{Config, Reason},
    engine::{Command, Engine},
    model::{Awareness, Entity, RoomRecipe, Scene},
    navigation::{NavGoal, NavState},
    perception::{self, NpcState, Source},
};
use serde_json::json;
use std::collections::BTreeMap;

fn actor(id: &str, x: f32, vision: f32) -> Entity {
    serde_json::from_value(json!({
        "id":id,"name":id,"body":"npc","faction":"guards","color":[0.3,0.4,0.9],
        "transform":{"position":[x,0.8,0],"size":[0.6,1.6,0.6]},
        "npc":{"target":"player","surface":"floor","speed":2,"stop_distance":0.5,"enabled":true,
            "communication":{"mode":"radio","channel":1,"range":8,"cooldown_ticks":30,"max_age_ticks":120,"send":true,"receive":true},
            "awareness":{"mode":"perceptive","vision_range":vision,"fov_degrees":90,
            "initial_facing_degrees":0,"memory_ticks":180,"search_ticks":60,"search_turn_degrees_per_second":0}}
    })).unwrap()
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
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    player.transform.position = [-4., 0.8, -3.];
    player.footsteps = None;
    s.entities
        .extend([actor("sender", -4., 8.), actor("receiver", 3., 0.1)]);
    s
}
fn config<'a>(s: &'a mut Scene, id: &str) -> &'a mut ignis::model::NpcConfig {
    s.entities
        .iter_mut()
        .find(|e| e.id == id)
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
}
struct Harness {
    scene: Scene,
    positions: BTreeMap<String, [f32; 3]>,
    states: BTreeMap<String, NpcState>,
    navigation: BTreeMap<String, NavState>,
    goals: BTreeMap<String, Option<NavGoal>>,
}
impl Harness {
    fn new(scene: Scene) -> Self {
        scene.validate().unwrap();
        let positions = scene
            .entities
            .iter()
            .map(|e| (e.id.clone(), e.transform.position))
            .collect();
        let states = scene
            .entities
            .iter()
            .filter_map(|e| e.npc.as_ref().map(|n| (e.id.clone(), NpcState::new(n))))
            .collect();
        let navigation = scene
            .entities
            .iter()
            .filter(|e| e.npc.is_some())
            .map(|e| (e.id.clone(), NavState::default()))
            .collect();
        Self {
            scene,
            positions,
            states,
            navigation,
            goals: Default::default(),
        }
    }
    fn step(&mut self, tick: u64) {
        perception::update(
            &self.scene,
            &self.positions,
            tick,
            &self.navigation,
            &mut self.states,
            &mut self.goals,
        );
    }
}

#[test]
fn frozen_report_preserves_identity_age_and_hidden_position_privacy() {
    let mut a = Harness::new(scene());
    a.step(0);
    assert!(a.states["sender"].observed_this_tick);
    assert!(a.states["receiver"].memory.is_none());
    let mut b = Harness::new(scene());
    b.step(0);
    a.positions.insert("player".into(), [-4., 0.8, 4.]);
    b.positions.insert("player".into(), [-7., 0.8, 5.]);
    for tick in 1..10 {
        a.step(tick);
        b.step(tick);
        assert_eq!(a.states["receiver"], b.states["receiver"]);
        assert_eq!(
            a.goals["receiver"].as_ref().map(|g| g.position),
            b.goals["receiver"].as_ref().map(|g| g.position)
        );
    }
    let state = &a.states["receiver"];
    let memory = state.memory.unwrap();
    assert_eq!(memory.source, Source::Report);
    assert_eq!(memory.position, [-4., 0.8, -3.]);
    assert_eq!(memory.tick, 0);
    assert!(!state.observed_this_tick && !state.heard_this_tick);
    assert_eq!(state.communication.accepted, 1);
    assert_eq!(state.communication.last_received.unwrap().received_tick, 1);
    assert!(state.confidence < 1.);
}

#[test]
fn radio_filters_channels_alliances_range_height_and_target() {
    for mode in 0..8 {
        let mut s = scene();
        match mode {
            0 => {
                let Config::Radio { channel, .. } =
                    config(&mut s, "receiver").communication.as_mut().unwrap();
                *channel = 2;
            }
            1 => {
                s.entities
                    .iter_mut()
                    .find(|e| e.id == "receiver")
                    .unwrap()
                    .faction = Some("other".into())
            }
            2 => {
                let Config::Radio { range, .. } =
                    config(&mut s, "sender").communication.as_mut().unwrap();
                *range = 6.;
            }
            3 => {
                let Config::Radio { receive, .. } =
                    config(&mut s, "receiver").communication.as_mut().unwrap();
                *receive = false;
            }
            4 => {
                let Config::Radio { send, .. } =
                    config(&mut s, "sender").communication.as_mut().unwrap();
                *send = false;
            }
            5 => {
                s.entities
                    .iter_mut()
                    .find(|e| e.id == "receiver")
                    .unwrap()
                    .transform
                    .position[1] = 7.
            }
            6 => config(&mut s, "receiver").target = Some("sender".into()),
            _ => {
                s.entities
                    .iter_mut()
                    .find(|e| e.id == "receiver")
                    .unwrap()
                    .faction = Some("other".into());
                s.faction_relations = serde_json::from_value(
                    json!([{"from":"guards","to":"other","attitude":"allied"}]),
                )
                .unwrap();
            }
        }
        let mut h = Harness::new(s);
        h.step(0);
        h.step(1);
        assert!(h.states["receiver"].memory.is_none(), "mode {mode}");
    }
}

#[test]
fn no_relay_no_order_dependence_and_cooldown_uses_fresh_vision_only() {
    let mut s = scene();
    s.entities.push(actor("relay", 9., 0.1));
    let mut reverse = s.clone();
    reverse.entities.reverse();
    let mut a = Harness::new(s);
    let mut b = Harness::new(reverse);
    for tick in 0..63 {
        a.step(tick);
        b.step(tick);
        assert_eq!(a.states, b.states);
        assert!(a.states["relay"].memory.is_none());
    }
    assert_eq!(a.states["sender"].communication.sent, 3);
    assert_eq!(a.states["receiver"].communication.sent, 0);
    assert_eq!(a.states["receiver"].memory.unwrap().tick, 60);
}

#[test]
fn report_expires_on_original_tick_without_rebroadcast_or_budget_renewal() {
    let mut s = scene();
    let Config::Radio { max_age_ticks, .. } =
        config(&mut s, "receiver").communication.as_mut().unwrap();
    *max_age_ticks = 5;
    let mut h = Harness::new(s);
    h.step(0);
    h.positions.insert("player".into(), [-4., 0.8, 4.]);
    for tick in 1..5 {
        h.step(tick);
        assert_eq!(h.states["receiver"].memory.unwrap().tick, 0);
    }
    h.step(5);
    assert!(h.states["receiver"].memory.is_none());
    assert_eq!(h.states["receiver"].reason, "report_expired");
    h.step(6);
    assert_eq!(h.states["sender"].communication.sent, 1);
    let mut s = scene();
    let Config::Radio { max_age_ticks, .. } =
        config(&mut s, "sender").communication.as_mut().unwrap();
    *max_age_ticks = 1;
    let mut h = Harness::new(s);
    h.step(0);
    h.step(1);
    assert!(h.states["receiver"].memory.is_none());
}

#[test]
fn recognition_direct_sensors_and_objectives_do_not_confuse_report_with_vision() {
    let mut s = scene();
    config(&mut s, "sender").recognition =
        Some(serde_json::from_value(json!({"acquire_ticks":3,"decay_ticks":1})).unwrap());
    let mut h = Harness::new(s);
    h.step(0);
    h.step(1);
    h.step(2);
    assert!(h.states["receiver"].memory.is_none());
    h.step(3);
    assert_eq!(h.states["receiver"].memory.unwrap().source, Source::Report);
    let mut s = scene();
    config(&mut s, "receiver").objectives = Some(
        serde_json::from_value(json!([
            {"priority":100,"when":"visual_memory","action":{"kind":"wait"}},
            {"priority":10,"when":"evidence","action":{"kind":"respond"}}
        ]))
        .unwrap(),
    );
    config(&mut s,"receiver").sound_attention=Some(serde_json::from_value(json!({"commitment_ticks":60,"switch_margin":0.1,"repeat_radius":5,"visual_priority_ticks":300})).unwrap());
    if let Some(Awareness::Perceptive { hearing, .. }) = &mut config(&mut s, "receiver").awareness {
        *hearing = Some(ignis::model::Hearing { range: 12. });
    }
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    assert_eq!(e.simulation.npcs["receiver"].objectives.eligible_mask, 2);
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [2., 0.8, 0.],
            },
            radius: 12.,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["receiver"].memory.unwrap().source,
        Source::Sound
    );
    assert_eq!(
        e.simulation.npcs["receiver"].sound_attention.reason,
        ignis::sound_attention::Reason::ReplacedReport
    );
    let mut s = scene();
    if let Some(Awareness::Perceptive {
        vision_range,
        fov_degrees,
        ..
    }) = &mut config(&mut s, "receiver").awareness
    {
        *vision_range = 20.;
        *fov_degrees = 360.;
    }
    let mut h = Harness::new(s);
    h.step(0);
    h.step(1);
    assert_eq!(h.states["receiver"].memory.unwrap().source, Source::Vision);
    assert_eq!(
        h.states["receiver"].communication.reason,
        Reason::DirectEvidence
    );
}

#[test]
fn version_validation_atomic_edits_reset_and_replay_keep_provenance() {
    let mut old = scene();
    old.version = 24;
    assert!(old.validate().is_err());
    for e in &mut old.entities {
        if let Some(n) = &mut e.npc {
            n.communication = None;
        }
    }
    assert!(old.validate().is_ok());
    let mut e = Engine::new(scene()).unwrap();
    e.advance(2).unwrap();
    let before = e.snapshot();
    let mut bad = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "receiver")
        .unwrap()
        .clone();
    let Config::Radio { channel, .. } = bad.npc.as_mut().unwrap().communication.as_mut().unwrap();
    *channel = 0;
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
    e.advance(60).unwrap();
    let replay = e.replay();
    assert_eq!(
        Engine::replay_at(&replay, Some(2)).unwrap().snapshot()["npcs"],
        before["npcs"]
    );
    assert_eq!(
        Engine::replay_at(&replay, None).unwrap().snapshot()["npcs"],
        e.snapshot()["npcs"]
    );
    assert_eq!(
        before["npcs"]["receiver"]["communication"]["last_received"]["sender_id"],
        "sender"
    );
    e.execute(Command::Reset.into()).unwrap();
    assert!(
        e.simulation.npcs["receiver"]
            .communication
            .last_received
            .is_none()
    );
}

#[test]
fn territorial_disabled_and_hostility_policies_filter_before_accepting() {
    for mode in 0..5 {
        let mut s = scene();
        match mode {
            0 => config(&mut s, "sender").enabled = false,
            1 => config(&mut s, "receiver").enabled = false,
            2 => {
                config(&mut s, "receiver").territory = Some(ignis::territory::Config {
                    home: [3., 0.],
                    radius: 2.,
                    arrival_distance: 0.2,
                    rearm_ticks: 0,
                })
            }
            _ => {
                let npc = config(&mut s, "receiver");
                npc.target = None;
                npc.targeting =
                    Some(ignis::targeting::Policy::HostileNearest { switch_margin: 0.1 });
                if mode == 4 {
                    s.entities
                        .iter_mut()
                        .find(|e| e.id == "player")
                        .unwrap()
                        .faction = Some("intruders".into());
                    s.faction_relations = serde_json::from_value(json!([
                        {"from":"guards","to":"intruders","attitude":"hostile"}
                    ]))
                    .unwrap();
                }
            }
        }
        let mut h = Harness::new(s);
        h.step(0);
        h.step(1);
        assert_eq!(
            h.states["receiver"].memory.is_some(),
            mode == 4,
            "mode {mode}"
        );
    }
}

#[test]
fn direct_memory_has_priority_and_equal_reports_use_stable_sender_id() {
    for source in [Source::Vision, Source::Sound] {
        let mut h = Harness::new(scene());
        h.step(0);
        let memory = ignis::perception::Observation {
            instance: None,
            life: Some(0),
            position: [2., 0.8, 2.],
            tick: 0,
            noise_id: None,
            source,
            actor_index: None,
        };
        h.states.get_mut("receiver").unwrap().memory = Some(memory);
        h.step(1);
        assert_eq!(h.states["receiver"].memory, Some(memory));
        assert_eq!(
            h.states["receiver"].communication.reason,
            Reason::ExistingEvidence
        );
        assert_eq!(h.states["receiver"].communication.accepted, 0);
    }
    let mut s = scene();
    s.entities.push(actor("a_sender", -3., 8.));
    config(&mut s, "receiver").recognition =
        Some(serde_json::from_value(json!({"acquire_ticks":3,"decay_ticks":1})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    let inspected = e.execute(Command::InspectNpcs.into()).unwrap();
    assert_eq!(
        inspected["npcs"]["receiver"]["communication"]["last_received"]["sender_id"],
        "a_sender"
    );
    assert!(!e.simulation.npcs["receiver"].observed_this_tick);
    assert!(
        e.simulation.npcs["receiver"]
            .knowledge
            .tracks
            .iter()
            .all(|t| t.evidence == 0 && !t.confirmed && t.last_seen.is_none())
    );
}
