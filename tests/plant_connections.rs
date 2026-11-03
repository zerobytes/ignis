use ignis::{
    engine::{Command, Engine},
    mesh_topology::Topology,
    meshes::Frame,
    model::RoomRecipe,
};
use serde_json::{Value, json};
fn run(e: &mut Engine, v: Value) -> Value {
    e.execute(serde_json::from_value::<Command>(v).unwrap().into())
        .unwrap()
}
fn put(e: &mut Engine, d: Value) {
    run(
        e,
        json!({"op":"edit_plant","edit":{"kind":"put","definition":d,"detail_policy":"physical","animation_translation_policy":"physical"}}),
    );
}
fn input() -> Value {
    serde_json::from_str(include_str!("../examples/plant-connection.input.json")).unwrap()
}
fn recipe() -> Value {
    input()["connection"].clone()
}
fn connect(e: &mut Engine, r: Value) -> Value {
    run(
        e,
        json!({"op":"edit_plant","edit":{"kind":"put_connection","connection":r}}),
    )
}
fn setup() -> Engine {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    for id in ["parent", "child"] {
        put(
            &mut e,
            input()["definitions"][usize::from(id == "child")].clone(),
        );
        let n = e.scene.meshes.asset(id).unwrap().geometry.positions.len();
        let mut bones =
            vec![json!({"name":"root","parent":null,"translation":[0,0,0],"rotation":[0,0,0,1]})];
        if id == "parent" {
            bones.push(
                json!({"name":"socket","parent":0,"translation":[0,-0.12,0],"rotation":[0,0,0,1]}),
            );
        }
        run(
            &mut e,
            json!({"op":"edit_mesh","edit":{"kind":"set_rig","asset":id,"rig":{"bones":bones,"weights":vec![json!({"joints":[0,0,0,0],"weights":[1,0,0,0]});n]}}}),
        );
    }
    run(
        &mut e,
        json!({"op":"edit_mesh","edit":{"kind":"put_instance","instance":{"id":"parent-instance","asset":"parent","position":[0,2,0],"rotation":[0,0,0,1],"scale":1,"color":[0.5,0.3,0.1]}}}),
    );
    run(
        &mut e,
        json!({"op":"edit_mesh","edit":{"kind":"put_instance","instance":{"id":"child-instance","asset":"child","position":[0.38971144,0.225,0],"rotation":[0,0,-0.5,0.8660254],"scale":1,"color":[0.5,0.3,0.1],"attachment":{"parent":"parent-instance","bone":"socket"}}}}),
    );
    e
}
#[test]
fn regenerate_preserve_unlink_and_frame_invalidation() {
    let mut e = setup();
    let original = e.scene.clone();
    connect(&mut e, recipe());
    let connected = e.scene.clone();
    assert_eq!(original.plants.entries, connected.plants.entries);
    assert_eq!(original.meshes.instances, connected.meshes.instances);
    assert_eq!(
        original.meshes.asset("child").unwrap(),
        connected.meshes.asset("child").unwrap()
    );
    let asset = connected.meshes.asset("parent").unwrap();
    let t = Topology::inspect(&asset.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);
    assert!(t.isolated_vertices.is_empty());
    let reports = e.scene.plants.connection_reports(&e.scene.meshes).unwrap();
    assert!(reports[0].skin.residual_max <= 1e-6);
    let mut f = Frame::default();
    f.update(&connected.meshes, 0);
    let full = f.vertices.len();
    f.update_filtered(&connected.meshes, 0, std::iter::once("child-instance"));
    assert_eq!(
        full - f.vertices.len(),
        connected
            .meshes
            .asset("child")
            .unwrap()
            .geometry
            .triangles
            .len()
            * 3
    );
    f.update(&connected.meshes, 0);
    assert_eq!(full, f.vertices.len());
    for id in ["parent", "child"] {
        let d = serde_json::to_value(&e.scene.plants.entry(id).unwrap().definition).unwrap();
        let mut moved = d.clone();
        moved["trajectory"]["control_points_xz"][1][0] = json!(0.004);
        put(&mut e, moved);
        assert_ne!(e.scene.meshes.asset("parent").unwrap(), asset);
        assert_eq!(
            e.scene.plants.entry(id).unwrap().source,
            original.plants.entry(id).unwrap().source
        );
        put(&mut e, d);
        assert_eq!(e.scene, connected);
    }
    run(
        &mut e,
        json!({"op":"edit_plant","edit":{"kind":"remove_connection","id":"join"}}),
    );
    assert_eq!(e.scene, original);
    run(&mut e, json!({"op":"undo"}));
    assert_eq!(e.scene, connected);
    run(&mut e, json!({"op":"redo"}));
    assert_eq!(e.scene, original);
}
#[test]
fn invalid_dependencies_solver_versions_and_caches_are_atomic() {
    let mut e = setup();
    connect(&mut e, recipe());
    let original = e.scene.clone();
    for case in 0..10 {
        let mut r = recipe();
        match case {
            0 => r["child_instance"] = json!("absent"),
            1 => r["child_instance"] = r["parent_instance"].clone(),
            2 => r["parent_sections"] = json!(["upper", "lower"]),
            3 => r["parent_sections"][0] = json!("start"),
            4 => r["child_section"] = json!("missing"),
            5 => r["parent_root"] = json!("socket"),
            6 => r["skin"]["iterations"] = json!(1),
            7 => r["shape"]["cell_size"] = json!(0.001),
            8 => r["skin"]["tolerance"] = json!(0),
            _ => r["id"] = json!("another"),
        };
        let cmd: Command = serde_json::from_value(
            json!({"op":"edit_plant","edit":{"kind":"put_connection","connection":r}}),
        )
        .unwrap();
        assert!(e.execute(cmd.into()).is_err(), "case {case}");
        assert_eq!(e.scene, original);
    }
    let mut old = original.clone();
    old.version = 71;
    assert!(old.validate().unwrap_err().message.contains("version72"));
    old.plants.entries.clear();
    assert!(old.validate().is_err());
    let mut bad = original.clone();
    std::sync::Arc::make_mut(&mut bad.meshes.assets[0].geometry).positions[0][0] += 0.001;
    assert!(bad.validate().is_err());
    let encoded = serde_json::to_string(&original).unwrap();
    let decoded = serde_json::from_str::<ignis::model::Scene>(&encoded).unwrap();
    decoded.validate().unwrap();
    assert_eq!(original, decoded);
    assert_eq!(e.replay().version, ignis::engine::REPLAY_VERSION);
    assert!(Engine::replay_at(&e.replay(), Some(0)).is_ok());
}
