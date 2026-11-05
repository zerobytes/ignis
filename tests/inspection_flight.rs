use ignis::{
    engine::{Command, Engine, ExecutionMode, inspection_flight::Config},
    model::{RoomRecipe, Scene},
    physics::Input,
};
fn engine() -> Engine {
    let mut s: Scene = RoomRecipe {
        width: 30.,
        depth: 30.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [0., 4., 0.];
    p.character_controller = Some(Default::default());
    p.footsteps = None;
    let mut e = Engine::new(s).unwrap();
    e.set_execution_mode(ExecutionMode::Continuous).unwrap();
    e.execute(serde_json::from_value(serde_json::json!({"command":{"op":"set_player_view","view":{"actor":"player","yaw_degrees":0,"pitch_degrees":0,"eye_height":1.4,"fov_degrees":70}}})).unwrap()).unwrap();
    e
}
#[test]
fn flight_hover_vertical_diagonal_release_and_persistence_boundary() {
    let mut e = engine();
    let authored = serde_json::to_value(&e.scene).unwrap();
    e.execute(
        Command::SetInspectionFlight {
            config: Some(Config {
                speed: 6.,
                fast_speed: 60.,
                ..Default::default()
            }),
        }
        .into(),
    )
    .unwrap();
    let start = e.simulation.position("player").unwrap();
    e.advance(60).unwrap();
    assert_eq!(start, e.simulation.position("player").unwrap());
    e.execute(Command::SetInspectionFlightVertical { value: 1. }.into())
        .unwrap();
    e.advance(60).unwrap();
    let up = e.simulation.position("player").unwrap();
    assert!((up[1] - start[1] - 6.).abs() < 0.001);
    e.execute(
        Command::SetInput {
            input: Input {
                x: 1.,
                z: 1.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    let p = e.simulation.position("player").unwrap();
    e.advance(60).unwrap();
    let q = e.simulation.position("player").unwrap();
    let d = glam::Vec3::from(q) - glam::Vec3::from(p);
    assert!((d.length() - 6.).abs() < 0.002);
    assert!(e.recorded_replay().is_err());
    assert!(e.set_execution_mode(ExecutionMode::Recorded).is_err());
    assert_eq!(authored, serde_json::to_value(&e.scene).unwrap());
    e.execute(Command::SetInspectionFlight { config: None }.into())
        .unwrap();
    let y = e.simulation.position("player").unwrap()[1];
    e.advance(30).unwrap();
    assert!(e.simulation.position("player").unwrap()[1] < y);
}
#[test]
fn reset_and_invalid_configuration_do_not_leave_flight_active() {
    let mut e = engine();
    e.execute(
        Command::SetInspectionFlight {
            config: Some(Default::default()),
        }
        .into(),
    )
    .unwrap();
    assert!(
        e.execute(Command::SetInspectionFlightVertical { value: f32::NAN }.into())
            .is_err()
    );
    assert!(
        e.execute(
            Command::SetInspectionFlight {
                config: Some(Config {
                    speed: -1.,
                    ..Default::default()
                })
            }
            .into()
        )
        .is_err()
    );
    assert!(e.inspection_flight_active());
    e.reset();
    assert!(!e.inspection_flight_active());
}

#[test]
fn noclip_crosses_a_wall_and_normal_controller_returns_after_disabling() {
    let mut e = engine();
    let mut scene = e.scene.clone();
    let mut wall = scene
        .entities
        .iter()
        .find(|e| e.body == Some(ignis::model::BodyKind::Fixed))
        .unwrap()
        .clone();
    wall.id = "inspection_wall".into();
    wall.transform.position = [0., 8., -3.];
    wall.transform.size = [20., 20., 1.];
    scene.entities.push(wall);
    e.replace(scene).unwrap();
    e.execute(serde_json::from_value(serde_json::json!({"command":{"op":"set_player_view","view":{"actor":"player","yaw_degrees":0,"pitch_degrees":0,"eye_height":1.4,"fov_degrees":70}}})).unwrap()).unwrap();
    e.execute(
        Command::SetInspectionFlight {
            config: Some(Config {
                speed: 6.,
                fast_speed: 60.,
                ..Default::default()
            }),
        }
        .into(),
    )
    .unwrap();
    e.execute(
        Command::SetPlayerViewInput {
            input: Input {
                x: 0.,
                z: -1.,
                ..Default::default()
            },
            control: None,
        }
        .into(),
    )
    .unwrap();
    e.advance(60).unwrap();
    assert!(e.simulation.position("player").unwrap()[2] < -5.9);
    e.execute(Command::SetInspectionFlight { config: None }.into())
        .unwrap();
    e.execute(
        Command::SetPlayerViewInput {
            input: Input {
                x: 0.,
                z: 1.,
                ..Default::default()
            },
            control: None,
        }
        .into(),
    )
    .unwrap();
    e.advance(60).unwrap();
    assert!(e.simulation.position("player").unwrap()[2] < -3.5);
}
