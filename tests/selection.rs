use glam::Vec3;
use ignis::{
    engine::{Command, Engine},
    model::{Entity, Scene},
    physics::Input,
    picking,
};
use serde_json::{Value, json};
fn box_entity(id: &str, position: [f32; 3], size: [f32; 3]) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"transform":{"position":position,"size":size},"color":[1,0,0],"body":null})).unwrap()
}
fn visual() -> Engine {
    let mut s: Scene = serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap();
    s.entities = vec![
        box_entity("back", [0., 0., 0.], [1.; 3]),
        box_entity("front", [0., 0., 2.], [1.; 3]),
    ];
    s.camera.position = [0., 0., 5.];
    s.camera.target = [0.; 3];
    s.camera.fov_degrees = 60.;
    Engine::new(s).unwrap()
}
fn select(e: &mut Engine, id: Option<&str>) -> Value {
    e.execute(
        Command::SelectEntity {
            id: id.map(String::from),
        }
        .into(),
    )
    .unwrap()
}
fn click(e: &mut Engine, x: f32, y: f32, w: u32, h: u32) -> Value {
    e.execute(
        Command::SelectAt {
            x,
            y,
            width: w,
            height: h,
        }
        .into(),
    )
    .unwrap()
}
#[test]
fn closest_visible_box_hit_and_miss_are_session_only() {
    let mut e = visual();
    let before = e.snapshot();
    let replay = serde_json::to_value(e.replay()).unwrap();
    let hit = click(&mut e, 400., 300., 800, 600);
    assert_eq!(hit["selection"], "front");
    assert!((hit["hit"]["distance"].as_f64().unwrap() - 2.5).abs() < 1e-4);
    assert!((hit["hit"]["position"][2].as_f64().unwrap() - 2.5).abs() < 1e-4);
    assert!(hit["hit"].get("entity_index").is_none());
    let rev = e.view_revision;
    select(&mut e, Some("front"));
    assert_eq!(e.view_revision, rev);
    assert_eq!(serde_json::to_value(e.replay()).unwrap(), replay);
    let mut after = e.snapshot();
    after["selection"] = before["selection"].clone();
    after["view_revision"] = before["view_revision"].clone();
    assert_eq!(after, before);
    assert_eq!(click(&mut e, 0., 0., 800, 600)["selection"], Value::Null);
    e.scene
        .entities
        .iter_mut()
        .find(|x| x.id == "front")
        .unwrap()
        .visible = Some(false);
    // Authoring alone cannot mutate execution definitions.
    assert_eq!(click(&mut e, 400., 300., 800, 600)["selection"], "front");
    e = Engine::new(e.scene.clone()).unwrap();
    assert_eq!(click(&mut e, 400., 300., 800, 600)["selection"], "back");
    select(&mut e, Some("front"));
    assert_eq!(e.render_frame().boxes.len(), 1);
    assert!(e.render_frame().selection.is_some());
}
#[test]
fn projected_point_resizes_and_uses_current_physics() {
    let s = serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap();
    let mut e = Engine::new(s).unwrap();
    e.input = Input {
        x: 1.,
        ..Default::default()
    };
    e.advance(45).unwrap();
    let p = e.simulation.position("player").unwrap();
    let authored = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "player")
        .unwrap()
        .transform
        .position;
    assert!((p[0] - authored[0]).abs() > 1.);
    for (w, h) in [(800, 600), (1400, 500), (400, 900)] {
        let ndc = picking::view_projection(&e.scene.camera, w, h).project_point3(Vec3::from(p));
        let x = (ndc.x + 1.) * 0.5 * w as f32;
        let y = (1. - ndc.y) * 0.5 * h as f32;
        assert_eq!(click(&mut e, x, y, w, h)["selection"], "player");
    }
    let inspect = e.execute(Command::InspectSelection.into()).unwrap();
    assert_eq!(inspect["selection"]["world_position"], json!(p));
    assert!(inspect["selection"]["footsteps"].is_object());
}
#[test]
fn near_far_clipping_ties_and_parallel_segments() {
    let mut e = visual();
    e.scene.entities = vec![
        box_entity("z", [0., 0., 0.], [1.; 3]),
        box_entity("a", [0., 0., 0.], [1.; 3]),
    ];
    e = Engine::new(e.scene.clone()).unwrap();
    assert_eq!(click(&mut e, 400., 300., 800, 600)["selection"], "a");
    e.scene.entities.reverse();
    e = Engine::new(e.scene.clone()).unwrap();
    assert_eq!(click(&mut e, 400., 300., 800, 600)["selection"], "a");
    e.scene.camera.position = [0.; 3];
    e.scene.camera.target = [0., 0., -1.];
    e.scene.entities = vec![
        box_entity("near", [0., 0., -0.02], [0.05; 3]),
        box_entity("behind", [0., 0., 1.], [0.1; 3]),
        box_entity("side", [2., 0., -2.], [0.1; 3]),
    ];
    e = Engine::new(e.scene.clone()).unwrap();
    assert_eq!(
        click(&mut e, 400., 300., 800, 600)["selection"],
        Value::Null
    );
    // A diagonal view can put legal world positions beyond the far clipping plane.
    e.scene.camera.position = [10000.; 3];
    e.scene.camera.target = [-10000.; 3];
    e.scene.entities = vec![box_entity("far", [-10000.; 3], [1.; 3])];
    e = Engine::new(e.scene.clone()).unwrap();
    assert_eq!(
        click(&mut e, 400., 300., 800, 600)["selection"],
        Value::Null
    );
    e.scene.entities[0].transform.position = [0.; 3];
    e = Engine::new(e.scene.clone()).unwrap();
    assert_eq!(click(&mut e, 400., 300., 800, 600)["selection"], "far");
}
#[test]
fn invalid_requests_preserve_selection_and_live_state() {
    let mut e = visual();
    select(&mut e, Some("front"));
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::SelectEntity {
                id: Some("missing".into())
            }
            .into()
        )
        .is_err()
    );
    for (x, y, w, h) in [
        (f32::NAN, 0., 800, 600),
        (0., f32::INFINITY, 800, 600),
        (-1., 0., 800, 600),
        (800., 0., 800, 600),
        (0., 600., 800, 600),
        (0., 0., 0, 600),
        (0., 0., 800, 16385),
    ] {
        assert!(
            e.execute(
                Command::SelectAt {
                    x,
                    y,
                    width: w,
                    height: h
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
}
#[test]
fn reset_reindexes_existing_selection_and_removal_does_not_resurrect_on_undo() {
    let mut e = visual();
    select(&mut e, Some("front"));
    e.reset();
    assert_eq!(e.selected_id(), Some("front"));
    e.execute(
        Command::Upsert {
            entity: box_entity("aaa", [4., 0., 0.], [1.; 3]).into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.render_frame().selection.unwrap().position, [0., 0., 2.]);
    e.execute(Command::Remove { id: "front".into() }.into())
        .unwrap();
    assert_eq!(e.selected_id(), None);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.selected_id(), None);
    assert!(e.render_frame().selection.is_none());
    select(&mut e, Some("front"));
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.selected_id(), None);
}
#[test]
fn selected_components_inspect_npc_and_invisible_trigger() {
    let s = serde_json::from_str(include_str!("../examples/stealth-lab.scene.json")).unwrap();
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    select(&mut e, Some("hunter"));
    let npc = e.execute(Command::InspectSelection.into()).unwrap();
    assert!(npc["selection"]["npc"].is_object());
    assert!(npc["selection"]["navigation"].is_object());
    select(&mut e, Some("exit_zone"));
    let region = e.execute(Command::InspectSelection.into()).unwrap();
    assert_eq!(region["selection"]["entity"]["visible"], false);
    assert_eq!(region["selection"]["trigger"]["occupants"], json!([]));
    select(&mut e, None);
    assert_eq!(
        e.execute(Command::InspectSelection.into()).unwrap()["selection"],
        Value::Null
    );
}
