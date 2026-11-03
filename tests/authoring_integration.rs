use ignis::{
    engine::{Engine, REPLAY_VERSION},
    model::{SCENE_VERSION, Scene},
};

fn combined_scene() -> Scene {
    let mut scene: Scene = serde_json::from_str(include_str!(
        "../examples/npc-generator-activation.scene.json"
    ))
    .unwrap();
    let authoring: Scene = serde_json::from_str(include_str!(
        "../examples/native-curved-cylinder.scene.json"
    ))
    .unwrap();
    scene.version = SCENE_VERSION;
    scene.meshes = authoring.meshes;
    scene.plants = authoring.plants;
    // Start inside the authored trigger to exercise production and authoring together.
    scene
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position[0] = 3.;
    scene
}

#[test]
fn plants_meshes_and_spatial_generators_survive_roundtrip_and_shared_replay() {
    let scene = combined_scene();
    scene.validate().unwrap();
    let loaded = serde_json::from_slice(&serde_json::to_vec(&scene).unwrap()).unwrap();
    let mut engine = Engine::new(loaded).unwrap();
    assert_eq!(engine.scene, scene);
    engine.advance(60).unwrap();
    assert!(engine.simulation.generators.generators[0].births > 0);
    let snapshot = engine.snapshot();
    assert!(snapshot.get("generators").is_some());
    assert!(snapshot.get("mesh_selection").is_some());
    let replay = engine.replay();
    assert_eq!(replay.version, REPLAY_VERSION);
    assert!(!replay.expected_meshes.is_empty());
    let parsed = serde_json::from_slice(&serde_json::to_vec(&replay).unwrap()).unwrap();
    let report = Engine::verify_replay(&parsed).unwrap();
    for key in [
        "matched",
        "meshes_matched",
        "generators_matched",
        "population_matched",
    ] {
        assert_eq!(report[key], true, "{key}");
    }
    let prefix = Engine::replay_at(&replay, Some(20)).unwrap();
    assert_eq!(prefix.scene.meshes, scene.meshes);
    assert_eq!(prefix.scene.plants, scene.plants);
    assert_eq!(prefix.scene.npc_generators, scene.npc_generators);
}

#[test]
fn replay_detects_each_front_independently_and_rejects_previous_format() {
    let mut engine = Engine::new(combined_scene()).unwrap();
    engine.advance(60).unwrap();
    let replay = engine.replay();
    let mut changed = replay.clone();
    changed.expected_meshes[0].positions[0][0] += 1.;
    let report = Engine::verify_replay(&changed).unwrap();
    assert_eq!(report["matched"], false);
    assert_eq!(report["meshes_matched"], false);
    assert_eq!(report["generators_matched"], true);
    let mut changed = replay.clone();
    changed.expected_generators.generators[0].births += 1;
    let report = Engine::verify_replay(&changed).unwrap();
    assert_eq!(report["matched"], false);
    assert_eq!(report["generators_matched"], false);
    assert_eq!(report["meshes_matched"], true);
    let mut previous = replay;
    previous.version = REPLAY_VERSION - 1;
    assert!(Engine::verify_replay(&previous).is_err());
}

#[test]
fn independent_players_ground_connectors_and_authored_meshes_share_one_replay() {
    use ignis::{engine::Command, physics::Input};
    let mut scene: Scene =
        serde_json::from_str(include_str!("../examples/npc-ground-local.scene.json")).unwrap();
    let authoring: Scene = serde_json::from_str(include_str!(
        "../examples/native-curved-cylinder.scene.json"
    ))
    .unwrap();
    scene.version = SCENE_VERSION;
    scene.meshes = authoring.meshes;
    scene.plants = authoring.plants;
    let mut second = scene
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    second.id = "second_player".into();
    second.transform.position = [-6., 0.81, 5.];
    scene.entities.push(second);
    let mut engine = Engine::new(scene).unwrap();
    let initial = engine.simulation.positions();
    for (actor, x) in [("player", 1.), ("second_player", -1.)] {
        let binding = engine
            .execute(
                Command::BindControl {
                    actor: actor.into(),
                }
                .into(),
            )
            .unwrap();
        engine
            .execute(
                Command::SetControlInput {
                    control: binding["control"].as_str().unwrap().into(),
                    input: Input {
                        x,
                        ..Default::default()
                    },
                }
                .into(),
            )
            .unwrap();
    }
    engine.advance(40).unwrap();
    let positions = engine.simulation.positions();
    assert!(positions["player"][0] > initial["player"][0] + 0.5);
    assert!(positions["second_player"][0] < initial["second_player"][0] - 0.5);
    assert!(
        engine.simulation.mobility.states["guard"]
            .ground_planning
            .is_some()
    );
    let replay = engine.replay();
    let result = Engine::verify_replay(&replay).unwrap();
    for key in [
        "matched",
        "controls_matched",
        "character_controllers_matched",
        "mobility_matched",
        "meshes_matched",
    ] {
        assert_eq!(result[key], true, "{key}: {result}");
    }
    let prefix = Engine::replay_at(&replay, Some(20)).unwrap();
    assert_eq!(prefix.simulation.tick, 20);
    assert_eq!(prefix.scene.meshes, engine.scene.meshes);
    assert_eq!(prefix.replay().expected_controls.bindings.len(), 2);
    let mut changed = replay.clone();
    changed
        .expected_character_controllers
        .values_mut()
        .next()
        .unwrap()
        .grounded ^= true;
    let result = Engine::verify_replay(&changed).unwrap();
    assert_eq!(result["matched"], false);
    assert_eq!(result["character_controllers_matched"], false);
    assert_eq!(result["controls_matched"], true);
    assert_eq!(result["meshes_matched"], true);
}
