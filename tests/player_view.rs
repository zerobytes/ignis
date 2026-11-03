//! Critical integration: camera must follow the physical actor without mutating authoring,
//! and stale/foreign identities must never steer another life or player.
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::{RoomRecipe, Scene},
    physics::Input,
    player_view::Config,
};
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 30.,
        depth: 30.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [0., 0.91, 0.];
    p.transform.size = [0.6, 1.8, 0.6];
    p.footsteps = None;
    p.character_controller = Some(Default::default());
    s
}
fn view() -> Config {
    Config {
        actor: "player".into(),
        yaw_degrees: 90.,
        pitch_degrees: 0.,
        eye_height: 1.65,
        fov_degrees: 75.,
        mode: Default::default(),
        third_person: Default::default(),
    }
}
#[test]
fn view_tracks_physics_preserves_authoring_and_expires_on_reset() {
    let mut e = Engine::new_with_mode(scene(), ExecutionMode::Continuous).unwrap();
    let authored = serde_json::to_value(&e.scene).unwrap();
    e.execute(Command::SetPlayerView { view: Some(view()) }.into())
        .unwrap();
    e.execute(
        Command::SetPlayerViewInput {
            input: Input {
                z: -1.,
                ..Default::default()
            },
            control: None,
        }
        .into(),
    )
    .unwrap();
    e.advance(120).unwrap();
    let p = e.simulation.position("player").unwrap();
    assert!(p[0] > 2. && p[2].abs() < 0.01, "{p:?}");
    let c = e.player_camera().unwrap();
    assert_eq!(c.position[0], p[0]);
    assert!((c.position[1] - (p[1] + 0.75)).abs() < 0.0001);
    assert!(c.target[0] > c.position[0] + 9.);
    assert!(e.simulation.character_controllers["player"].grounded);
    assert_eq!(serde_json::to_value(&e.scene).unwrap(), authored);
    assert_eq!(e.render_frame().camera, c);
    let before = e.render_frame().boxes.len();
    e.execute(Command::SetPlayerView { view: None }.into())
        .unwrap();
    assert_eq!(e.render_frame().boxes.len(), before + 1);
    e.execute(Command::SetPlayerView { view: Some(view()) }.into())
        .unwrap();
    e.reset();
    assert!(e.player_camera().is_none());
    assert!(
        e.execute(
            Command::SetPlayerViewInput {
                input: Input::default(),
                control: None
            }
            .into()
        )
        .is_err()
    );
}
#[test]
fn invalid_view_and_foreign_control_fail_before_mutation() {
    let mut s = scene();
    let mut p = s
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    p.id = "other".into();
    p.transform.position[0] = -8.;
    s.entities.push(p);
    let mut e = Engine::new(s).unwrap();
    e.execute(Command::SetPlayerView { view: Some(view()) }.into())
        .unwrap();
    let before = e.player_camera().unwrap();
    let revision = e.view_revision;
    let mut bad = view();
    bad.pitch_degrees = 90.;
    assert!(
        e.execute(Command::SetPlayerView { view: Some(bad) }.into())
            .is_err()
    );
    assert_eq!(e.view_revision, revision);
    assert_eq!(e.player_camera().unwrap(), before);
    let token = e
        .execute(
            Command::BindControl {
                actor: "other".into(),
            }
            .into(),
        )
        .unwrap()["control"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        e.execute(
            Command::SetPlayerViewInput {
                input: Input {
                    z: -1.,
                    ..Default::default()
                },
                control: Some(token)
            }
            .into()
        )
        .is_err()
    );
    let token = e
        .execute(
            Command::BindControl {
                actor: "player".into(),
            }
            .into(),
        )
        .unwrap()["control"]
        .as_str()
        .unwrap()
        .to_string();
    e.execute(
        Command::SetPlayerViewInput {
            input: Input {
                z: -1.,
                ..Default::default()
            },
            control: Some(token),
        }
        .into(),
    )
    .unwrap();
    e.advance(60).unwrap();
    assert!(e.simulation.position("player").unwrap()[0] > 1.);
    assert!((e.simulation.position("other").unwrap()[0] + 8.).abs() < 0.01);
}

fn third() -> Config {
    let mut v = view();
    v.yaw_degrees = 0.;
    v.mode = ignis::player_view::Mode::ThirdPerson;
    v.third_person.shoulder_offset = 0.;
    v
}
fn state(e: &mut Engine) -> serde_json::Value {
    e.execute(Command::InspectPlayerView.into()).unwrap()
}
fn wall(s: &mut Scene, x: f32, z: f32, width: f32) {
    let mut w = s
        .entities
        .iter()
        .find(|e| e.id == "wall_north")
        .unwrap()
        .clone();
    w.id = "camera_wall".into();
    w.transform.position = [x, 1.5, z];
    w.transform.size = [width, 3., 0.4];
    s.entities.push(w);
}

/// Protects the tick0 integration boundary: a camera must not tunnel through
/// physical primitives/meshes or move the actor while resolving its own view.
#[test]
fn third_person_sphere_handles_tick_zero_meshes_grazing_and_overlap_read_only() {
    let mut s = scene();
    wall(&mut s, 0., 2.5, 4.);
    let mut e = Engine::new_with_mode(s, ExecutionMode::Continuous).unwrap();
    let authored = serde_json::to_value(&e.scene).unwrap();
    let position = e.simulation.position("player").unwrap();
    e.execute(
        Command::SetPlayerView {
            view: Some(third()),
        }
        .into(),
    )
    .unwrap();
    let report = state(&mut e);
    assert_eq!(report["resolved"]["obstructed"], true);
    assert_eq!(report["resolved"]["initial_overlap"], false);
    assert!((2.0..2.2).contains(&e.player_camera().unwrap().position[2]));
    assert_eq!(e.simulation.tick, 0);
    assert_eq!(e.simulation.position("player").unwrap(), position);
    assert_eq!(serde_json::to_value(&e.scene).unwrap(), authored);
    let mut bad = third();
    bad.third_person.distance_meters = f32::NAN;
    let revision = e.view_revision;
    assert!(
        e.execute(Command::SetPlayerView { view: Some(bad) }.into())
            .is_err()
    );
    assert_eq!(e.view_revision, revision);
    assert_eq!(state(&mut e), report);
    e.execute(
        Command::LookPlayerView {
            yaw_delta_degrees: 90.,
            pitch_delta_degrees: -10.,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(state(&mut e)["view"]["yaw_degrees"], 90.);
    let looked = state(&mut e);
    assert!(
        e.execute(
            Command::LookPlayerView {
                yaw_delta_degrees: f32::NAN,
                pitch_delta_degrees: 0.
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(state(&mut e), looked);
    assert_eq!(e.simulation.position("player").unwrap(), position);
    assert_eq!(e.simulation.tick, 0);

    let mut s = scene();
    // The centre ray x=0 misses x>=0.15, but the padded sphere must hit it.
    wall(&mut s, 0.45, 2.5, 0.6);
    let mut grazing = Engine::new(s).unwrap();
    grazing
        .execute(
            Command::SetPlayerView {
                view: Some(third()),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(state(&mut grazing)["resolved"]["obstructed"], true);

    let mut s = scene();
    let asset: ignis::meshes::Asset = serde_json::from_value(serde_json::json!({
        "id":"mesh_wall","geometry":{"positions":[[-3,0,2.5],[3,0,2.5],[-3,3,2.5],[3,3,2.5]],"triangles":[[0,2,1],[1,2,3]]},"shading":"smooth"
    })).unwrap();
    let instance = serde_json::from_value(serde_json::json!({"id":"mesh_wall","asset":"mesh_wall","position":[0,0,0],"rotation":[0,0,0,1],"scale":1,"color":[0.5,0.5,0.5]})).unwrap();
    s.meshes.assets.push(asset);
    s.meshes.instances.push(instance);
    s.collision_meshes.push("mesh_wall".into());
    let mut mesh = Engine::new(s).unwrap();
    mesh.execute(
        Command::SetPlayerView {
            view: Some(third()),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(state(&mut mesh)["resolved"]["obstructed"], true);
    assert!((2.2..2.4).contains(&mesh.player_camera().unwrap().position[2]));

    let mut ground = Engine::new(scene()).unwrap();
    let mut down = third();
    down.pitch_degrees = 70.;
    ground
        .execute(Command::SetPlayerView { view: Some(down) }.into())
        .unwrap();
    assert_eq!(state(&mut ground)["resolved"]["obstructed"], true);
    assert!(ground.player_camera().unwrap().position[1] >= 0.17);

    let mut s = scene();
    wall(&mut s, 0., 0., 4.);
    let mut overlap = Engine::new(s).unwrap();
    overlap
        .execute(
            Command::SetPlayerView {
                view: Some(third()),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(state(&mut overlap)["resolved"]["initial_overlap"], true);
    assert_eq!(state(&mut overlap)["resolved"]["hide_body"], true);
}

/// Perspective changes preserve the actor and other actors' avatars.
/// Player avatar authoring is a separate integration from camera control.
#[test]
fn perspective_switch_preserves_other_avatars_and_legacy_first_person_contract() {
    let s: Scene =
        serde_json::from_str(include_str!("../examples/npc-presentation.scene.json")).unwrap();
    let mut e = Engine::new(s).unwrap();
    let before = serde_json::to_value(&e.scene).unwrap();
    let count = e.render_frame().meshes.drawn_avatars.len();
    let mut v = third();
    v.eye_height = 1.44;
    e.execute(
        Command::SetPlayerView {
            view: Some(v.clone()),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.render_frame().meshes.drawn_avatars.len(), count);
    let bodies = e.render_frame().boxes.len();
    v.mode = ignis::player_view::Mode::FirstPerson;
    e.execute(Command::SetPlayerView { view: Some(v) }.into())
        .unwrap();
    assert_eq!(e.render_frame().meshes.drawn_avatars.len(), count);
    assert_eq!(e.render_frame().boxes.len(), bodies - 1);
    assert_eq!(serde_json::to_value(&e.scene).unwrap(), before);
    let old: Config = serde_json::from_value(serde_json::json!({"actor":"player","yaw_degrees":0,"pitch_degrees":0,"eye_height":1.44,"fov_degrees":75})).unwrap();
    assert_eq!(old.mode, ignis::player_view::Mode::FirstPerson);
    e.reset();
    assert_eq!(state(&mut e)["active"], false);
}
