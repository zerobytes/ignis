//! Critical integration: rendered surface must be the floor, and remain so after save/edit.
use ignis::{
    engine::{Command, Engine, SceneEdit},
    meshes::{Asset, Geometry, Instance},
    model::{RoomRecipe, Scene},
    physics::Input,
};
use serde_json::json;
use std::sync::Arc;
fn run(e: &mut Engine, c: Command) -> serde_json::Value {
    e.execute(c.into()).unwrap()
}
fn mesh(
    s: &mut Scene,
    id: &str,
    positions: Vec<[f32; 3]>,
    triangles: Vec<[u32; 3]>,
    position: [f32; 3],
    rotation: [f32; 4],
    scale: f32,
) {
    s.meshes.assets.push(Asset {
        id: id.into(),
        geometry: Arc::new(Geometry {
            positions,
            triangles,
            material_coordinates: None,
        }),
        shading: Default::default(),
        material: Default::default(),
        rig: None,
        clips: vec![],
    });
    s.meshes.instances.push(Instance {
        id: id.into(),
        asset: id.into(),
        position,
        rotation,
        scale,
        color: [0.4, 0.6, 0.3],
        playback: None,
        attachment: None,
    });
    s.collision_meshes.push(id.into());
}
fn quad(s: &mut Scene, id: &str, x0: f32, x1: f32, z0: f32, z1: f32, slope: f32) {
    mesh(
        s,
        id,
        vec![
            [x0, 2. + slope * x0, z0],
            [x1, 2. + slope * x1, z0],
            [x0, 2. + slope * x0, z1],
            [x1, 2. + slope * x1, z1],
        ],
        vec![[0, 2, 1], [1, 2, 3]],
        [0.; 3],
        [0., 0., 0., 1.],
        1.,
    );
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.retain(|e| e.id == "player");
    let p = &mut s.entities[0];
    p.transform.position = [-6., 1.5, 0.];
    p.transform.size = [0.6, 1.6, 0.6];
    p.footsteps = None;
    p.character_controller = Some(Default::default());
    quad(&mut s, "left", -8., 0., -3., 3., 0.25);
    quad(&mut s, "right", 0., 8., -3., 3., 0.25);
    s
}
fn walk(e: &mut Engine, t: u32, x: f32, z: f32) {
    run(
        e,
        Command::SetInput {
            input: Input {
                x,
                z,
                ..Default::default()
            },
        },
    );
    e.advance(t).unwrap();
}
fn pos(e: &Engine) -> [f32; 3] {
    e.simulation.positions()["player"]
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn physical_slope_seam_fall_transforms_and_true_arch_opening() {
    let mut e = Engine::new(scene()).unwrap();
    walk(&mut e, 30, 0., 0.);
    let start = pos(&e);
    assert!(
        (start[1] - (2. + 0.25 * start[0] + 0.885)).abs() < 0.04,
        "{start:?}"
    );
    let report = run(
        &mut e,
        Command::QueryGroundTraversal {
            entity: "player".into(),
            start: [-6., 0.585, 0.],
            end: [6., 3.585, 0.],
        },
    );
    assert_eq!(report["report"]["status"], "arrived", "{report}");
    walk(&mut e, 180, 1., 0.);
    let up = pos(&e);
    assert!(up[0] > 4. && up[1] > 3.5, "{up:?}");
    assert!(e.simulation.character_controllers["player"].grounded);
    replay(&e);
    walk(&mut e, 180, -1., 0.);
    let down = pos(&e);
    assert!(down[0] < -4. && down[1] < 2., "{down:?}");
    walk(&mut e, 120, 0., 1.);
    assert!(pos(&e)[1] < -2.);
    replay(&e);
    // An actual landform arch leaves a walkable opening. Its bounds are not a solid.
    let mut s = scene();
    s.meshes = Default::default();
    s.collision_meshes.clear();
    quad(&mut s, "ground", -8., 8., -8., 8., 0.);
    let d:ignis::landforms::Document=serde_json::from_value(ignis::landforms::query(serde_json::from_value(json!({"kind":"preset","id":"arch","origin":{"cell":["0","0","0"],"offset":[0,0,0]},"preset":{"kind":"arch","outer_radius_meters":3,"inner_radius_meters":2,"leg_height_meters":2,"depth_meters":1,"segments":12}})).unwrap()).unwrap()["document"].clone()).unwrap();
    let c = ignis::landforms::compile(&d).unwrap();
    mesh(
        &mut s,
        "arch",
        c.geometry.positions.clone(),
        c.geometry.triangles.clone(),
        [0., 2., 0.],
        [0., 0., 0., 1.],
        1.,
    );
    s.entities[0].transform.position = [0., 2.9, -3.];
    let mut e = Engine::new(s.clone()).unwrap();
    walk(&mut e, 90, 0., 1.);
    assert!(pos(&e)[2] > 2., "{:?}", pos(&e));
    replay(&e);
    s.entities[0].transform.position = [2.5, 2.9, -3.];
    let mut blocked = Engine::new(s).unwrap();
    walk(&mut blocked, 90, 0., 1.);
    assert!(pos(&blocked)[2] < 0., "{:?}", pos(&blocked));
    // Translation, rotation and scale follow the same native instance.
    let mut s = scene();
    s.meshes = Default::default();
    s.collision_meshes.clear();
    let q = glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2).to_array();
    mesh(
        &mut s,
        "rotated",
        vec![
            [-2., 0., -0.5],
            [2., 0., -0.5],
            [-2., 0., 0.5],
            [2., 0., 0.5],
        ],
        vec![[0, 2, 1], [1, 2, 3]],
        [3., 4., 5.],
        q,
        2.,
    );
    s.entities[0].transform.position = [3., 5., 8.];
    s.entities.push(serde_json::from_value(json!({"id":"crate","name":"crate","body":"dynamic","color":[0.6,0.3,0.2],"transform":{"position":[3.,6.,6.5],"size":[0.5,0.5,0.5]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    walk(&mut e, 60, 0., 0.);
    assert!((pos(&e)[1] - 4.81).abs() < 0.03);
    assert!((e.simulation.positions()["crate"][1] - 4.25).abs() < 0.03);
    replay(&e);
}
#[test]
fn collision_edit_atomicity_full_footprint_and_native_portable_continuation() {
    let mut s = scene();
    let template=serde_json::from_value(json!({"id":"guard","name":"guard","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"character_controller":{},"npc":{"surface":"floor","speed":2,"stop_distance":1,"enabled":false,"mobility":{"walk":true},"local_connections":{"max_distance":8,"walk":true,"fly":false}}})).unwrap();
    // The legacy NPC surface anchor stays an authored primitive; physical support may be a mesh.
    s.entities.push(serde_json::from_value(json!({"id":"floor","name":"anchor","body":"fixed","color":[0.3,0.3,0.3],"transform":{"position":[0,-10,0],"size":[1,1,1]}})).unwrap());
    s.npc_templates = vec![template];
    let mut e = Engine::new(s).unwrap();
    let q = run(
        &mut e,
        Command::QuerySpawnSupport {
            template: "guard".into(),
            position: [3., 8., 0.],
            max_drop: 10.,
        },
    );
    assert_eq!(q["report"]["status"], "supported", "{q}");
    assert_eq!(q["surface_id"], "right");
    let spawn = run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: q["report"]["position"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap() as f32)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            target: None,
        },
    );
    assert!(!spawn.is_null());
    e.advance(1).unwrap();
    assert_eq!(e.simulation.npcs.len(), 1);
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::EditScene {
                edits: vec![
                    SceneEdit::SetMeshCollision {
                        instance: "left".into(),
                        enabled: false
                    },
                    SceneEdit::SetMeshCollision {
                        instance: "missing".into(),
                        enabled: true
                    }
                ]
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let bytes = ignis::checkpoint::Checkpoint::capture(&e)
        .unwrap()
        .encode()
        .unwrap();
    let mut stale = ignis::checkpoint::Checkpoint::capture(&e).unwrap();
    Arc::make_mut(&mut stale.scene.meshes.assets[0].geometry).positions[0][1] += 1.;
    assert!(e.load_checkpoint(&stale.encode().unwrap(), None).is_err());
    assert_eq!(e.snapshot(), before);
    let mut native = Engine::from_checkpoint(&bytes, None).unwrap();
    let bytes = ignis::portable_save::PortableSave::capture(&e)
        .unwrap()
        .encode()
        .unwrap();
    let mut portable = Engine::from_portable_save(&bytes, None).unwrap();
    for copy in [&mut native, &mut portable] {
        assert_eq!(copy.scene.collision_meshes, e.scene.collision_meshes);
        walk(copy, 45, 1., 0.);
        assert!(pos(copy)[1] > 1.);
        replay(copy);
    }
    let old = e.scene.clone();
    run(
        &mut e,
        Command::EditScene {
            edits: vec![SceneEdit::SetMeshCollision {
                instance: "left".into(),
                enabled: false,
            }],
        },
    );
    run(&mut e, Command::Undo);
    assert_eq!(e.scene, old);
    let mut asset = old.meshes.asset("left").unwrap().clone();
    for p in &mut Arc::make_mut(&mut asset.geometry).positions {
        p[1] -= 0.25;
    }
    run(
        &mut e,
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        },
    );
    walk(&mut e, 30, 0., 0.);
    assert!((pos(&e)[1] - 1.135).abs() < 0.04);
    run(&mut e, Command::Undo);
    walk(&mut e, 30, 0., 0.);
    assert!((pos(&e)[1] - 1.385).abs() < 0.04);
    // A center ray would accept this bridge, while the full footprint correctly refuses it.
    let mut s = scene();
    s.meshes = Default::default();
    s.collision_meshes.clear();
    quad(&mut s, "narrow", -8., 8., -0.1, 0.1, 0.);
    s.npc_templates = old.npc_templates.clone();
    s.entities.push(
        old.entities
            .iter()
            .find(|e| e.id == "floor")
            .unwrap()
            .clone(),
    );
    let mut e = Engine::new(s).unwrap();
    let q = run(
        &mut e,
        Command::QuerySpawnSupport {
            template: "guard".into(),
            position: [0., 8., 0.],
            max_drop: 8.,
        },
    );
    assert_eq!(q["report"]["status"], "insufficient_support", "{q}");
    // Center and all four corners have ground, but a small interior hole must stay a hole.
    let mut s = scene();
    s.meshes = Default::default();
    s.collision_meshes.clear();
    for (id, x0, x1, z0, z1) in [
        ("a", -8., 0.1, -3., 3.),
        ("b", 0.2, 8., -3., 3.),
        ("c", 0.1, 0.2, -3., 0.1),
        ("d", 0.1, 0.2, 0.2, 3.),
    ] {
        quad(&mut s, id, x0, x1, z0, z1, 0.);
    }
    s.npc_templates = old.npc_templates.clone();
    s.entities.push(
        old.entities
            .iter()
            .find(|e| e.id == "floor")
            .unwrap()
            .clone(),
    );
    let mut e = Engine::new(s).unwrap();
    let q = run(
        &mut e,
        Command::QuerySpawnSupport {
            template: "guard".into(),
            position: [0., 8., 0.],
            max_drop: 8.,
        },
    );
    assert_eq!(q["report"]["status"], "insufficient_support", "{q}");
}
