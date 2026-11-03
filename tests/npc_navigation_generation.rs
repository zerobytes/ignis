//! Critical generation must stay atomic, invalidate by physical source, and replay deterministically.
use ignis::{
    engine::{Command, Engine},
    mobility::Phase,
    model::{Entity, RoomRecipe},
};
use serde_json::json;
fn fixture() -> (
    ignis::model::Scene,
    ignis::npc_navigation_generation::Recipe,
) {
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
    s.entities.push(serde_json::from_value(json!({"id":"npc","name":"npc","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[-4,0.81,-4],"size":[0.6,1.6,0.6]},"character_controller":{},"npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,"mobility":{"walk":true},"local_connections":{"walk":true,"fly":false,"max_distance":2.5},"mind":{"intelligence":1},"objectives":[{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[4,0.8,4],"arrival_distance":0.2}}]}})).unwrap());
    s.entities.push(serde_json::from_value(json!({"id":"wall","name":"wall","body":"fixed","color":[0.3,0.6,0.5],"transform":{"position":[0,1.5,0],"size":[1,3,6]}})).unwrap());
    let recipe = serde_json::from_value(
        json!({"entity":"npc","min":[-4,0,-4],"max":[4,0,4],"cell_size":2,"connection_radius":2.1}),
    )
    .unwrap();
    (s, recipe)
}
#[test]
fn generated_layout_is_atomic_invalidated_by_geometry_and_preserved_in_replay() {
    let (s, recipe) = fixture();
    let mut e = Engine::new(s).unwrap();
    let before = e.snapshot();
    let preview = e
        .execute(
            Command::QueryNavigationGeneration {
                recipe: recipe.clone(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(before, e.snapshot());
    assert!(preview["report"]["nodes"].as_u64().unwrap() > 0);
    e.execute(
        Command::SetNavigationGeneration {
            recipe: Some(recipe.clone()),
        }
        .into(),
    )
    .unwrap();
    let original = e.scene.spatial_network.clone();
    assert_eq!(serde_json::to_value(&original).unwrap(), preview["network"]);
    let cache = e
        .scene
        .navigation_generation
        .as_ref()
        .unwrap()
        .cached
        .clone();
    e.advance(700).unwrap();
    assert_eq!(
        e.simulation.mobility.states["npc"].phase,
        Phase::Arrived,
        "{:?}",
        e.simulation.mobility.states["npc"]
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let before = e.snapshot();
    let mut bad = recipe.clone();
    bad.cell_size = 0.;
    assert!(
        e.execute(Command::SetNavigationGeneration { recipe: Some(bad) }.into())
            .is_err()
    );
    assert_eq!(before, e.snapshot());
    let mut actor = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "player")
        .unwrap()
        .clone();
    actor.transform.position = [8., 0.8, 7.];
    e.execute(
        Command::Upsert {
            entity: Box::new(actor),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.spatial_network, original);
    assert_eq!(
        e.scene.navigation_generation.as_ref().unwrap().cached,
        cache
    );
    let mut wall: Entity = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "wall")
        .unwrap()
        .clone();
    wall.transform.position[2] = 8.;
    e.execute(
        Command::Upsert {
            entity: Box::new(wall),
        }
        .into(),
    )
    .unwrap();
    assert_ne!(e.scene.spatial_network, original);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene.spatial_network, original);
    let serialized = serde_json::to_value(&e.scene).unwrap();
    assert!(serialized["navigation_generation"].get("cached").is_none());
    let roundtrip: ignis::model::Scene = serde_json::from_value(serialized).unwrap();
    let loaded = Engine::new(roundtrip).unwrap();
    assert_eq!(loaded.scene.spatial_network, e.scene.spatial_network);
    e.execute(Command::SetSpatialNetwork { network: original }.into())
        .unwrap();
    assert!(e.scene.navigation_generation.is_none());
    let mut old = e.scene.clone();
    old.version = 90;
    old.navigation_generation = Some(recipe);
    assert!(old.validate().is_err());
}
