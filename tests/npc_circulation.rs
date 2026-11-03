//! Critical: anonymous steering must preserve physics, private knowledge, identity and replay.
use ignis::{
    engine::{Command, Engine},
    model::{Entity, RoomRecipe},
};
use serde_json::json;
fn actor(id: &str, x: f32, goal: f32) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[x,0.81,0],"size":[0.6,1.6,0.6]},"character_controller":{"snap_distance":0.3},"npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,"mobility":{"walk":true},"local_connections":{"walk":true,"fly":false,"max_distance":3},"mind":{"intelligence":1},"circulation":{},"objectives":[{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[goal,0.8,0],"arrival_distance":0.2}}]}})).unwrap()
}
#[test]
fn anonymous_circulation_preserves_body_privacy_instance_and_replay() {
    let mut s = RoomRecipe {
        width: 24.,
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
        .position = [-8., 0.8, 7.];
    s.entities
        .extend([actor("a", -4., 4.), actor("b", 4., -4.)]);
    s.spatial_network=serde_json::from_value(json!({"nodes":[[-4,0,0],[4,0,0]],"links":[{"from":0,"to":1,"mode":"walk"},{"from":1,"to":0,"mode":"walk"}]})).unwrap();
    let mut template = actor("template", -4., 4.);
    template.transform.position[2] = 3.;
    s.npc_templates.push(template);
    let mut e = Engine::new(s).unwrap();
    let identity = e.simulation.character_controllers["a"].instance;
    let mut avoiding = false;
    for _ in 0..480 {
        let previous = e.simulation.positions();
        e.advance(1).unwrap();
        let now = e.simulation.positions();
        for id in ["a", "b"] {
            let state = &e.simulation.npcs[id];
            assert!(state.memory.is_none());
            let c = state.circulation.unwrap();
            avoiding |= c.reason == ignis::npc_circulation::Reason::Avoiding;
            assert!(glam::Vec3::from(c.chosen).with_y(0.).length() <= 2.001);
            assert!(glam::Vec3::from(now[id]).distance(previous[id].into()) < 0.1);
            assert_eq!(e.simulation.character_controllers["a"].instance, identity);
        }
    }
    assert!(
        avoiding,
        "{:?} {:?} {:?}",
        e.simulation.positions(),
        e.simulation.npcs["a"].circulation,
        e.simulation.mobility.states["a"]
    );
    assert!(
        e.simulation.positions()["a"][0] > 2.,
        "{:?} {:?}",
        e.simulation.positions(),
        e.simulation.npcs["a"].circulation
    );
    assert!(e.simulation.positions()["b"][0] < -2.);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut tampered = e.replay();
    tampered
        .expected_npcs
        .get_mut("a")
        .unwrap()
        .circulation
        .as_mut()
        .unwrap()
        .neighbors = 16;
    assert_eq!(Engine::verify_replay(&tampered).unwrap()["matched"], false);
    let before = e.snapshot();
    let mut bad = actor("a", -4., 4.);
    bad.npc
        .as_mut()
        .unwrap()
        .circulation
        .as_mut()
        .unwrap()
        .radius = 0.;
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
    let mut old = e.scene.clone();
    old.version = 91;
    assert!(old.validate().is_err());
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.simulation.npcs["a"].circulation.is_none());
    e.advance(1).unwrap();
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"despawn_actor","actor":"a"}))
            .unwrap()
            .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    e.execute(
        serde_json::from_value::<Command>(
            json!({"op":"spawn_actor","template":"template","position":[-4,0.81,3]}),
        )
        .unwrap()
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    let born = e
        .simulation
        .character_controllers
        .iter()
        .find(|(id, _)| id.as_str() != "b" && id.as_str() != "player")
        .unwrap();
    assert_ne!(born.1.instance, identity);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut hidden = e.scene.clone();
    hidden.npc_templates.clear();
    hidden
        .entities
        .iter_mut()
        .find(|e| e.id == "a")
        .unwrap()
        .transform
        .position = [-0.8, 0.81, 0.];
    hidden
        .entities
        .iter_mut()
        .find(|e| e.id == "b")
        .unwrap()
        .transform
        .position = [0.8, 0.81, 0.];
    hidden.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","transform":{"position":[0,2,0],"size":[0.4,4,8]},"color":[0.2,0.8,0.5]})).unwrap());
    let mut hidden = Engine::new(hidden).unwrap();
    hidden.advance(1).unwrap();
    assert_eq!(
        hidden.simulation.npcs["a"].circulation.unwrap().neighbors,
        0
    );
}
