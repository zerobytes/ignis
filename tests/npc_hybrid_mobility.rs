//! Critical handoff: one physical body/identity must survive mode changes and replay.
use ignis::{
    engine::{Command, Engine},
    mobility::{Mode, Phase},
    model::{Entity, RoomRecipe},
};
use serde_json::json;
fn scene() -> ignis::model::Scene {
    let mut s = RoomRecipe {
        width: 30.,
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
        .position = [-10., 0.8, 7.];
    let platform:Entity=serde_json::from_value(json!({"id":"platform","name":"platform","body":"fixed","color":[0.2,0.7,0.5],"transform":{"position":[4.5,1,0],"size":[5,2,4]}})).unwrap();
    s.entities.push(platform);
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-5,0,0],[-3,0,0],[-3,3,0],[3,3,0],[3,2,0],[5,2,0]],"links":[{"from":0,"to":1,"mode":"walk"},{"from":1,"to":2,"mode":"fly"},{"from":2,"to":3,"mode":"fly"},{"from":3,"to":4,"mode":"fly"},{"from":4,"to":5,"mode":"walk"}]})).unwrap();
    s.entities.push(serde_json::from_value(json!({"id":"npc","name":"npc","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[-5,0.81,0],"size":[0.6,1.6,0.6]},"character_controller":{"snap_distance":0.3,"step":{"max_height":0.3,"min_width":0.4}},"npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,"mobility":{"walk":true,"fly_speed":3},"mind":{"intelligence":1,"personality":{"courage":1,"fear":0.2,"caution":0.5}},"objectives":[{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[5,2.8,0],"arrival_distance":0.2}}]}})).unwrap());
    s
}
#[test]
fn ground_air_ground_preserves_body_identity_physics_and_replay() {
    let mut s = scene();
    s.version = 89;
    assert!(s.validate().is_err());
    s.version = ignis::model::SCENE_VERSION;
    let mut e = Engine::new(s).unwrap();
    let identity = e.simulation.character_controllers["npc"].instance;
    let mut modes = std::collections::BTreeSet::new();
    let mut previous = e.simulation.positions()["npc"];
    for _ in 0..600 {
        e.advance(1).unwrap();
        let p = e.simulation.positions()["npc"];
        assert!(glam::Vec3::from(p).distance(previous.into()) < 0.2);
        previous = p;
        let c = e.simulation.character_controllers["npc"];
        assert_eq!(c.instance, identity);
        if c.airborne {
            assert!(!c.grounded);
        }
        if let Some(m) = e.simulation.mobility.states["npc"].motor_mode {
            modes.insert(format!("{m:?}"));
        }
    }
    assert!(modes.contains("Walk") && modes.contains("Fly"), "{modes:?}");
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?} {:?}",
        e.simulation.mobility.states["npc"],
        e.simulation.positions()
    );
    assert!(!e.simulation.character_controllers["npc"].airborne);
    assert!(e.simulation.character_controllers["npc"].grounded);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let prefix = Engine::replay_at(&e.replay(), Some(150)).unwrap();
    assert_eq!(
        prefix.simulation.character_controllers["npc"].instance,
        identity
    );
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.simulation.mobility.states["npc"].motor_mode, None);
    let mut absent = scene();
    absent
        .entities
        .iter_mut()
        .find(|e| e.id == "npc")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .mobility
        .as_mut()
        .unwrap()
        .fly_speed = None;
    let mut absent = Engine::new(absent).unwrap();
    absent.advance(200).unwrap();
    assert_ne!(
        absent.simulation.mobility.states["npc"].phase,
        Phase::Arrived
    );
    assert!(
        absent.simulation.mobility.states["npc"]
            .motor_mode
            .is_none_or(|m| m == Mode::Walk)
    );
    let mut local = scene();
    local
        .entities
        .iter_mut()
        .find(|e| e.id == "npc")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .local_connections =
        Some(serde_json::from_value(json!({"walk":true,"fly":true,"max_distance":8})).unwrap());
    let mut local = Engine::new(local).unwrap();
    local.advance(600).unwrap();
    assert_eq!(
        local.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?}",
        local.simulation.mobility.states["npc"]
    );
    assert_eq!(
        Engine::verify_replay(&local.replay()).unwrap()["matched"],
        true
    );
    // The same critical handoff also covers ballistic and descending motors.
    for mode in [Mode::Jump, Mode::Glide] {
        let mut s = scene();
        s.entities.retain(|e| e.id != "platform");
        let n = s.entities.iter_mut().find(|e| e.id == "npc").unwrap();
        n.npc.as_mut().unwrap().mobility = Some(
            serde_json::from_value(if mode == Mode::Jump {
                json!({"walk":true,"jump_speed":6})
            } else {
                json!({"walk":true,"glide":{"speed":2,"sink_speed":2}})
            })
            .unwrap(),
        );
        let (nodes, links, goal) = if mode == Mode::Jump {
            (
                json!([[-5, 0, 0], [-3, 0, 0], [-1, 0, 0], [1, 0, 0]]),
                json!([{"from":0,"to":1,"mode":"walk"},{"from":1,"to":2,"mode":"jump"},{"from":2,"to":3,"mode":"walk"}]),
                [1., 0.8, 0.],
            )
        } else {
            n.transform.position = [-3., 3.8, 0.];
            (
                json!([[-3, 3, 0], [0, 0, 0], [2, 0, 0]]),
                json!([{"from":0,"to":1,"mode":"glide"},{"from":1,"to":2,"mode":"walk"}]),
                [2., 0.8, 0.],
            )
        };
        n.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":goal,"arrival_distance":0.2}}])).unwrap());
        s.spatial_network = serde_json::from_value(json!({"nodes":nodes,"links":links})).unwrap();
        let mut e = Engine::new(s).unwrap();
        let mut saw = false;
        let mut last = e.simulation.positions()["npc"];
        for _ in 0..500 {
            e.advance(1).unwrap();
            saw |= e.simulation.mobility.states["npc"].motor_mode == Some(mode);
            let now = e.simulation.positions()["npc"];
            assert!(glam::Vec3::from(now).distance(last.into()) < 0.2);
            last = now;
        }
        assert!(saw, "{mode:?}");
        assert_eq!(
            e.simulation.mobility.states["npc"].phase,
            Phase::Arrived,
            "{mode:?} {:?}",
            e.simulation.mobility.states["npc"]
        );
        assert!(!e.simulation.character_controllers["npc"].airborne);
        assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    }
    // Death/return must reset the motor fence without replacing the individual.
    let mut life = scene();
    life.entities
        .iter_mut()
        .find(|e| e.id == "npc")
        .unwrap()
        .vitality =
        Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
    life.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["kill","revive"]})).unwrap());
    life.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[{"id":"kill","selection":"actor","range":30,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},{"id":"revive","selection":"actor","range":30,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":50}]}]})).unwrap();
    let mut e = Engine::new(life).unwrap();
    e.advance(90).unwrap();
    assert!(e.simulation.character_controllers["npc"].airborne);
    let id = e.simulation.character_controllers["npc"].instance;
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("npc".into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert!(!e.simulation.character_controllers["npc"].active);
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: Some("npc".into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    let c = e.simulation.character_controllers["npc"];
    assert_eq!(c.instance, id);
    assert!(c.active);
    assert!(!c.airborne);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
