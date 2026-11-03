//! Critical contract: personality must not bypass evidence/validation and must replay by identity.
use ignis::{
    engine::{Command, Engine},
    model::{Entity, RoomRecipe},
};
use serde_json::json;
#[test]
fn mind_preserves_private_evidence_atomic_edits_and_instance_replay() {
    let mut scene = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [3., 0.8, 0.];
    let actor:Entity=serde_json::from_value(json!({"id":"guard","name":"guard","body":"npc","color":[0.6,0.2,0.8],"transform":{"position":[-3,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":2,"stop_distance":0.6,"enabled":true,"target":"player","awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"memory_ticks":120,"search_ticks":60,"initial_facing_degrees":0,"search_turn_degrees_per_second":0},"mind":{"intelligence":0.8,"variation":0.1,"seed":42,"personality":{"fear":1,"courage":0,"anxiety":0.8,"caution":0.9,"aggression":0}},"objectives":[{"priority":1,"when":"evidence","action":{"kind":"respond"}},{"priority":1,"when":"evidence","action":{"kind":"flee","distance":4}}]}})).unwrap();
    scene.entities.push(actor.clone());
    let mut template = actor.clone();
    template.id = "template".into();
    scene.npc_templates.push(template);
    let mut engine = Engine::new(scene).unwrap();
    engine.advance(2).unwrap();
    let state = &engine.simulation.npcs["guard"];
    assert_eq!(state.objectives.active, Some(1));
    let profile = state.mind_profile.unwrap();
    assert_eq!(
        Engine::verify_replay(&engine.replay()).unwrap()["matched"],
        true
    );
    let mut tampered = engine.replay();
    tampered
        .expected_npcs
        .get_mut("guard")
        .unwrap()
        .mind_profile
        .as_mut()
        .unwrap()
        .intelligence = 0;
    assert_eq!(Engine::verify_replay(&tampered).unwrap()["matched"], false);
    let before = engine.snapshot();
    let mut invalid = actor.clone();
    invalid
        .npc
        .as_mut()
        .unwrap()
        .mind
        .as_mut()
        .unwrap()
        .intelligence = 2.;
    assert!(
        engine
            .execute(
                Command::Upsert {
                    entity: Box::new(invalid)
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(before, engine.snapshot());
    let mut old = engine.scene.clone();
    old.version = 88;
    assert!(old.validate().is_err());
    let mut hidden = engine.scene.clone();
    hidden.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","color":[0.3,0.5,0.4],"transform":{"position":[0,2,0],"size":[1,4,16]}})).unwrap());
    let mut hidden = Engine::new(hidden).unwrap();
    hidden.advance(30).unwrap();
    assert!(hidden.simulation.npcs["guard"].memory.is_none());
    assert_eq!(hidden.simulation.npcs["guard"].objectives.active, None);
    engine.execute(Command::Reset.into()).unwrap();
    engine.advance(2).unwrap();
    assert_eq!(engine.simulation.npcs["guard"].mind_profile, Some(profile));
    engine.execute(serde_json::from_value::<Command>(json!({"op":"spawn_actor","template":"template","position":[-5,0.8,4],"target":"player"})).unwrap().into()).unwrap();
    engine.advance(2).unwrap();
    let id = engine
        .simulation
        .npcs
        .keys()
        .find(|id| id.as_str() != "guard")
        .unwrap();
    assert_ne!(engine.simulation.npcs[id].mind_profile, Some(profile));
    assert_eq!(
        Engine::verify_replay(&engine.replay()).unwrap()["matched"],
        true
    );
}
