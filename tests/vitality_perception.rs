use ignis::{
    model::{Awareness, Entity, RoomRecipe, Scene},
    navigation::{NavGoal, NavState},
    perception::{self, NpcState},
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
    fn step(&mut self, tick: u64, dead: u16) {
        self.step_lives(tick, dead, &[0; ignis::targeting::MAX_ACTORS]);
    }
    fn step_lives(&mut self, tick: u64, dead: u16, lives: &[u64; ignis::targeting::MAX_ACTORS]) {
        let actors = ignis::targeting::Catalog::new(&self.scene);
        perception::update_with_context(
            &self.scene,
            &self.positions,
            tick,
            &self.navigation,
            &mut self.states,
            &mut self.goals,
            perception::Sensors {
                lives,
                ability_policy: None,
                actors: &actors,
                dead: dead.into(),
                ability_ready: &[0; ignis::targeting::MAX_ACTORS],
                stimuli: perception::Stimuli {
                    manual: &[],
                    automatic: &[],
                    abilities: &[],
                },
            },
        );
    }
}

#[test]
fn hidden_death_preserves_memory_until_confirmed_observation() {
    let mut s = scene();
    s.entities.retain(|e| e.id != "receiver");
    let mut a = Harness::new(s.clone());
    let mut b = Harness::new(s);
    a.step(0, 0);
    b.step(0, 0);
    assert!(a.states["sender"].observed_this_tick);
    let actors = ignis::targeting::Catalog::new(&a.scene);
    let player = actors
        .actors
        .iter()
        .position(|&i| a.scene.entities[i as usize].id == "player")
        .unwrap();
    let dead = 1 << player;
    a.positions.insert("player".into(), [-4., 0.8, 4.]);
    b.positions.insert("player".into(), [-6., 0.8, 5.]);
    a.step(1, dead);
    b.step(1, 0);
    assert_eq!(a.states["sender"], b.states["sender"]);
    assert!(a.states["sender"].known_dead.is_empty());
    assert!(a.states["sender"].memory.is_some());
    a.positions.insert("player".into(), [-4., 0.8, -3.]);
    a.step(2, dead);
    assert_eq!(a.states["sender"].known_dead, dead.into());
    assert!(a.states["sender"].memory.is_none());
    assert!(a.goals["sender"].is_none());
}
#[test]
fn confirmed_corpse_cannot_be_reacquired_from_same_tick_stale_radio() {
    let mut s = scene();
    config(&mut s, "sender").communication = Some(ignis::communication::Config::Radio {
        channel: 1,
        range: 20.,
        cooldown_ticks: 1,
        max_age_ticks: 120,
        send: true,
        receive: true,
    });
    config(&mut s, "receiver").awareness = Some(Awareness::Perceptive {
        vision_range: 20.,
        fov_degrees: 360.,
        initial_facing_degrees: 0.,
        memory_ticks: Some(180),
        max_investigation_distance: None,
        search_ticks: Some(60),
        search_turn_degrees_per_second: 0.,
        area_search: None,
        hearing: None,
    });
    let mut h = Harness::new(s);
    h.step(0, 0);
    let actors = ignis::targeting::Catalog::new(&h.scene);
    let p = actors
        .actors
        .iter()
        .position(|&i| h.scene.entities[i as usize].id == "player")
        .unwrap();
    h.step(1, 1 << p);
    assert!(h.states["sender"].communication.sent_this_tick);
    assert!(h.states["receiver"].memory.is_none());
    assert!(h.states["receiver"].communication.rejected > 0);
    h.positions.insert("player".into(), [10., 0.8, 9.]);
    h.step(2, 1 << p);
    assert!(h.states["receiver"].memory.is_none());
}
#[test]
fn dead_sender_never_broadcasts_previous_living_observation() {
    let mut h = Harness::new(scene());
    h.step(0, 0);
    let actors = ignis::targeting::Catalog::new(&h.scene);
    let a = actors
        .actors
        .iter()
        .position(|&i| h.scene.entities[i as usize].id == "sender")
        .unwrap();
    h.step(1, 1 << a);
    assert_eq!(h.states["sender"].phase, perception::Phase::Dead);
    assert!(!h.states["sender"].communication.sent_this_tick);
    assert!(h.states["receiver"].memory.is_none());
    assert!(h.goals["sender"].is_none());
}
#[test]
fn corpse_recognition_requires_exposure_but_omniscient_is_immediate() {
    for omniscient in [false, true] {
        let mut s = scene();
        s.entities.retain(|e| e.id != "receiver");
        config(&mut s, "sender").recognition = Some(ignis::recognition::Config {
            acquire_ticks: 3,
            decay_ticks: 6,
        });
        if omniscient {
            config(&mut s, "sender").communication = None;
            config(&mut s, "sender").awareness = Some(Awareness::Omniscient);
        }
        let mut h = Harness::new(s);
        let actors = ignis::targeting::Catalog::new(&h.scene);
        let p = actors
            .actors
            .iter()
            .position(|&i| h.scene.entities[i as usize].id == "player")
            .unwrap();
        h.step(0, 1 << p);
        assert_eq!(!h.states["sender"].known_dead.is_empty(), omniscient);
        h.step(1, 1 << p);
        h.step(2, 1 << p);
        assert_eq!(h.states["sender"].known_dead, (1_u16 << p).into());
        assert!(h.states["sender"].memory.is_none());
    }
}

#[test]
fn automatic_selection_skips_confirmed_corpse_for_living_hostile() {
    let mut s = scene();
    s.entities.retain(|e| e.id != "receiver");
    let mut other = actor("other", -2., 8.);
    other.transform.position = [-2., 0.8, -4.];
    other.npc.as_mut().unwrap().communication = None;
    other.faction = Some("foes".into());
    s.entities.push(other);
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .faction = Some("foes".into());
    s.faction_relations.push(ignis::targeting::Relation {
        from: "guards".into(),
        to: "foes".into(),
        attitude: ignis::targeting::Attitude::Hostile,
    });
    let n = config(&mut s, "sender");
    n.target = None;
    n.communication = None;
    n.targeting = Some(ignis::targeting::Policy::HostileNearest { switch_margin: 0.2 });
    let mut h = Harness::new(s);
    let actors = ignis::targeting::Catalog::new(&h.scene);
    let p = actors
        .actors
        .iter()
        .position(|&i| h.scene.entities[i as usize].id == "player")
        .unwrap();
    let other = actors
        .actors
        .iter()
        .position(|&i| h.scene.entities[i as usize].id == "other")
        .unwrap();
    h.step(0, 0);
    assert_eq!(
        h.states["sender"].memory.unwrap().actor_index,
        Some(p as u16)
    );
    h.step(1, 1 << p);
    assert_eq!(h.states["sender"].known_dead, (1_u16 << p).into());
    assert_eq!(
        h.states["sender"].memory.unwrap().actor_index,
        Some(other as u16)
    );
}

#[test]
fn hidden_revival_preserves_belief_until_new_confirmation() {
    let mut s = scene();
    s.entities.retain(|e| e.id != "receiver");
    config(&mut s, "sender").communication = None;
    config(&mut s, "sender").recognition = Some(ignis::recognition::Config {
        acquire_ticks: 3,
        decay_ticks: 6,
    });
    let mut a = Harness::new(s.clone());
    let mut b = Harness::new(s);
    let actors = ignis::targeting::Catalog::new(&a.scene);
    let p = actors
        .actors
        .iter()
        .position(|&i| a.scene.entities[i as usize].id == "player")
        .unwrap();
    for t in 0..3 {
        a.step(t, 1 << p);
        b.step(t, 1 << p);
    }
    a.positions.insert("player".into(), [-4., 0.8, 4.]);
    b.positions.insert("player".into(), [-6., 0.8, 5.]);
    let mut lives = [0; ignis::targeting::MAX_ACTORS];
    lives[p] = 1;
    for t in 3..7 {
        a.step_lives(t, 0, &lives);
        b.step(t, 1 << p);
        assert_eq!(a.states["sender"], b.states["sender"]);
    }
    a.positions.insert("player".into(), [-4., 0.8, -3.]);
    a.step_lives(7, 0, &lives);
    assert!(a.states["sender"].known_dead.contains(p));
    assert!(!a.states["sender"].observed_this_tick);
    a.step_lives(8, 0, &lives);
    a.step_lives(9, 0, &lives);
    assert!(!a.states["sender"].known_dead.contains(p));
    assert_eq!(a.states["sender"].memory.unwrap().life, Some(1));
}

#[test]
fn radio_from_previous_life_cannot_override_newer_confirmed_identity() {
    let mut h = Harness::new(scene());
    let actors = ignis::targeting::Catalog::new(&h.scene);
    let p = actors
        .actors
        .iter()
        .position(|&i| h.scene.entities[i as usize].id == "player")
        .unwrap();
    h.positions.insert("player".into(), [10., 0.8, 9.]);
    let sender = h.states.get_mut("sender").unwrap();
    sender.memory = Some(perception::Observation {
        instance: Some(actors.reference(p as u16)),
        life: Some(0),
        position: [-4., 0.8, -3.],
        tick: 9,
        source: perception::Source::Vision,
        noise_id: None,
        actor_index: Some(p as u16),
    });
    sender.observed_this_tick = true;
    h.states
        .get_mut("receiver")
        .unwrap()
        .known_lives
        .resize(actors.actors.len(), 0);
    h.states
        .get_mut("receiver")
        .unwrap()
        .known_instances
        .resize(actors.actors.len(), None);
    h.states.get_mut("receiver").unwrap().known_lives[p] = 1;
    h.states.get_mut("receiver").unwrap().known_instances[p] = Some(actors.reference(p as u16));
    let mut lives = [0; ignis::targeting::MAX_ACTORS];
    lives[p] = 1;
    h.step_lives(10, 0, &lives);
    assert!(h.states["sender"].communication.sent_this_tick);
    assert!(h.states["receiver"].communication.rejected > 0);
    assert!(h.states["receiver"].memory.is_none());
}
