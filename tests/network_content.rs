//! Critical public-data boundary and atomic visual-format failures.
#[path = "support/network.rs"]
mod support;
use ignis::{
    engine::Engine,
    network::{
        content::Content,
        replica::{Applied, Frame, Kind, MAX_PROJECTILE_VISUALS, Projectile, Replica},
    },
};
use serde_json::json;
#[test]
fn public_content_and_nonactor_projection_exclude_private_scene_and_validate_before_publication() {
    let mut scene = support::scene(2);
    scene.name = "private_world_label".into();
    let mut serialized = serde_json::to_string(&scene).unwrap();
    serialized = serialized.replace("energy", "secret_energy");
    scene = serde_json::from_str(&serialized).unwrap();
    for entity in [
        json!({"id":"ramp","name":"Ramp","body":"fixed","geometry":{"kind":"wedge","rise":"positive_x"},"transform":{"position":[0,1,4],"size":[3,2,3]},"color":[0.2,0.3,0.7]}),
        json!({"id":"crate","name":"Crate","body":"dynamic","transform":{"position":[0,3,0],"size":[1,1,1]},"color":[0.8,0.5,0.2]}),
    ] {
        scene.entities.push(serde_json::from_value(entity).unwrap());
    }
    scene.meshes=serde_json::from_value(json!({"assets":[{"id":"decoration","geometry":{"positions":[[0,0,0],[2,0,0],[0,1,0]],"triangles":[[0,1,2]]}}],"instances":[{"id":"decoration_instance","asset":"decoration","position":[0,2,0],"rotation":[0,0,0,1],"scale":1,"color":[0.2,0.8,0.3],"playback":null}]})).unwrap();
    let engine = Engine::new(scene).unwrap();
    let content = Content::capture(&engine, 42);
    let bytes = content.encode().unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    for private in [
        "secret_energy",
        "private_world_label",
        "npc_templates",
        "ability_catalog",
        "character_controller",
        "spatial_network",
    ] {
        assert!(!text.contains(private), "{private}");
    }
    let decoded = Content::decode(42, &bytes).unwrap();
    assert_eq!(decoded.meshes, content.meshes);
    assert!(Content::decode(43, &bytes).is_err());
    let mut poisoned = serde_json::to_value(&content).unwrap();
    poisoned["npc_policy"] = json!({"hidden_target":"player"});
    assert!(Content::decode(42, &serde_json::to_vec(&poisoned).unwrap()).is_err());
    let mut bad = content.clone();
    bad.camera.fov_degrees = f32::NAN;
    assert!(bad.validate(42).is_err());
    bad = content.clone();
    bad.meshes.assets[0].geometry = std::sync::Arc::new(
        serde_json::from_value(json!({"positions":[[0,0,0]],"triangles":[[0,1,2]]})).unwrap(),
    );
    assert!(bad.encode().is_err());
    assert!(Content::decode(42, &vec![b' '; ignis::network::content::MAX_BYTES + 1]).is_err());
    let initial = Frame::capture(&engine, 42, None).unwrap();
    assert!(
        initial
            .actors
            .iter()
            .any(|a| a.appearance.kind == Kind::Dynamic)
    );
    assert!(
        initial
            .actors
            .iter()
            .any(|a| !a.appearance.geometry.is_box())
    );
    for a in &initial.actors {
        let index = engine
            .simulation
            .definitions()
            .entities
            .iter()
            .position(|e| e.id == a.appearance.label)
            .unwrap();
        assert_eq!(
            a.id,
            engine
                .simulation
                .actors
                .entity_reference(index)
                .unwrap()
                .instance
        );
    }
    let mut replica = Replica::new(42);
    replica.apply(&initial.encode(None).unwrap()).unwrap();
    assert_eq!(replica.actor_count(), 2);
    assert_eq!(replica.actors().len(), initial.actors.len());
    let mut next = initial.clone();
    next.tick += 1;
    next.projectiles = vec![Projectile {
        execution: 9,
        millimeters: [1000, 2000, 3000],
        half_extent: 0.1,
    }];
    let valid = next.encode(Some(&initial)).unwrap();
    let mut invalid = valid.clone();
    let n = invalid.len();
    invalid[n - 4..].copy_from_slice(&f32::NAN.to_le_bytes());
    let before = replica.clone();
    assert!(replica.apply(&invalid).is_err());
    assert_eq!(replica, before);
    assert_eq!(replica.apply(&valid).unwrap(), Applied::Current);
    assert_eq!(replica.projectiles().len(), 1);
    next.projectiles = vec![next.projectiles[0]; MAX_PROJECTILE_VISUALS + 1];
    assert!(next.encode(None).is_err());
}
