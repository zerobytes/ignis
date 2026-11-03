use glam::Vec3;
use ignis::{
    engine::{Command, Engine},
    model::{Awareness, RoomRecipe, Scene},
    perception::{self, NpcState, SpatialVision},
};
use serde_json::json;
use std::collections::BTreeMap;

fn scene(pitch: f32, target: [f32; 3]) -> Scene {
    let mut s = RoomRecipe {
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
        .position = target;
    s.entities.push(
        serde_json::from_value(json!({
            "id":"observer", "name":"Observer", "body":"npc", "color":[0.6,0.2,0.8],
            "transform":{"position":[0,3,0],"size":[0.6,1.6,0.6]},
            "npc":{"target":"player","surface":"floor","speed":2,"stop_distance":0.5,"enabled":true,
                "spatial_vision":{"initial_pitch_degrees":pitch,"follow_movement":false},
                "awareness":{"mode":"perceptive","vision_range":8,"fov_degrees":60,
                    "initial_facing_degrees":0,"memory_ticks":180,"search_ticks":60,
                    "search_turn_degrees_per_second":90}}
        }))
        .unwrap(),
    );
    s
}
fn npc(s: &mut Scene) -> &mut ignis::model::NpcConfig {
    s.entities
        .iter_mut()
        .find(|e| e.id == "observer")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
}
fn observed(s: Scene) -> bool {
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    e.simulation.npcs["observer"].observed_this_tick
}

#[test]
fn cone_filters_elevation_range_and_boundary_while_legacy_stays_horizontal() {
    for (pitch, target, expected) in [
        (0., [0., 7.56, 0.], false),
        (90., [0., 7.56, 0.], true),
        (-90., [0., 1.56, 0.], true),
        (-90., [0., 7.56, 0.], false),
        (90., [0., 12., 0.], false),
        (0., [0., 3.56, -4.], true),
        (0., [0., 3.56, 4.], false),
    ] {
        assert_eq!(
            observed(scene(pitch, target)),
            expected,
            "{pitch}: {target:?}"
        );
    }
    for (angle, expected) in [(29.9_f32, true), (30.1, false)] {
        let direction = perception::view_direction(0., angle);
        assert_eq!(
            observed(scene(
                0.,
                (Vec3::new(0., 3.56, 0.) + direction * 4.).to_array()
            )),
            expected
        );
    }
    let mut legacy = scene(0., [0., 7.56, 0.]);
    npc(&mut legacy).spatial_vision = None;
    legacy.version = 22;
    assert!(observed(legacy));
    let mut sphere = scene(0., [0., 7.56, 0.]);
    if let Some(Awareness::Perceptive { fov_degrees, .. }) = &mut npc(&mut sphere).awareness {
        *fov_degrees = 360.;
    }
    assert!(observed(sphere));
}

#[test]
fn ceiling_occludes_vertical_vision_and_omniscience_is_explicit() {
    let mut s = scene(90., [0., 7.56, 0.]);
    s.entities.push(serde_json::from_value(json!({"id":"ceiling","name":"ceiling","body":null,
        "blocks_vision":true,"transform":{"position":[0,5.5,0],"size":[3,0.3,3]},"color":[0.5,0.5,0.5]})).unwrap());
    assert!(!observed(s.clone()));
    let mut omniscient = s.clone();
    npc(&mut omniscient).awareness = Some(Awareness::Omniscient);
    assert!(observed(omniscient));
    s.entities
        .iter_mut()
        .find(|e| e.id == "ceiling")
        .unwrap()
        .blocks_vision = Some(false);
    assert!(observed(s));
}

#[test]
fn hidden_altitude_changes_do_not_leak_into_memory_or_goals() {
    let e = Engine::new(scene(0., [0., 3.56, -4.])).unwrap();
    let mut a = e.simulation.npcs.clone();
    let mut b = a.clone();
    let mut pa = e.simulation.positions();
    let mut pb = pa.clone();
    let (mut ga, mut gb) = (BTreeMap::new(), BTreeMap::new());
    for tick in 0..90 {
        if tick > 0 {
            pa.insert("player".into(), [0., 8., 0.]);
            pb.insert("player".into(), [1., 9. + (tick % 3) as f32, 0.]);
        }
        perception::update(
            &e.scene,
            &pa,
            tick,
            &e.simulation.navigation,
            &mut a,
            &mut ga,
        );
        perception::update(
            &e.scene,
            &pb,
            tick,
            &e.simulation.navigation,
            &mut b,
            &mut gb,
        );
        assert_eq!(a, b, "tick {tick}");
        let fields = |g: &Option<ignis::navigation::NavGoal>| {
            g.as_ref().map(|g| {
                (
                    g.position,
                    g.excluded_entity,
                    g.target_half_height,
                    g.stop_distance,
                    g.remaining_travel,
                    g.replan_on_target_change,
                )
            })
        };
        assert_eq!(
            fields(&ga["observer"]),
            fields(&gb["observer"]),
            "tick {tick}"
        );
        if tick > 0 {
            assert!(!a["observer"].observed_this_tick);
        }
    }
}

#[test]
fn orientation_follows_motor_xyz_retains_on_stop_and_can_be_independent() {
    let mut s = scene(20., [0., 3.56, -4.]);
    let config = npc(&mut s);
    let mut fixed = NpcState::new(config);
    fixed.face_spatial_velocity([3., 3., 0.]);
    assert_eq!(fixed.facing_pitch_degrees, Some(20.));
    assert_eq!(fixed.facing_degrees, 0.);
    config.spatial_vision.as_mut().unwrap().follow_movement = true;
    let mut state = NpcState::new(config);
    for velocity in [[0., 3., 0.], [3., 3., 0.], [0., -3., -3.], [0., 0., -2.]] {
        state.face_spatial_velocity(velocity);
        let dir =
            perception::view_direction(state.facing_degrees, state.facing_pitch_degrees.unwrap());
        assert!(dir.abs_diff_eq(Vec3::from(velocity).normalize(), 1e-6));
    }
    let before = state.clone();
    state.face_spatial_velocity([0.; 3]);
    assert_eq!(state, before);
}

#[test]
fn flight_updates_sensor_and_replay_prefix_matches_without_turning_body() {
    let mut s = scene(0., [5., 0.8, 5.]);
    let config = npc(&mut s);
    config.target = None;
    config.spatial_vision.as_mut().unwrap().follow_movement = true;
    config.mobility = Some(serde_json::from_value(json!({"walk":false,"fly_speed":3})).unwrap());
    config.local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap());
    config.objectives = Some(serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"guard_3d","point":[0,6,0],"arrival_distance":0.2}}])).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(20).unwrap();
    assert!(e.simulation.npcs["observer"].facing_pitch_degrees.unwrap() > 89.);
    let prefix = e.snapshot()["npcs"].clone();
    e.advance(90).unwrap();
    let replay = e.replay();
    assert_eq!(
        Engine::replay_at(&replay, Some(20)).unwrap().snapshot()["npcs"],
        prefix
    );
    assert_eq!(
        Engine::replay_at(&replay, None).unwrap().snapshot()["npcs"],
        e.snapshot()["npcs"]
    );
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.simulation.npcs["observer"].facing_pitch_degrees, Some(0.));
}

#[test]
fn invalid_config_is_atomic_and_history_restores_orientation() {
    let mut e = Engine::new(scene(30., [0., 7.56, 0.])).unwrap();
    let original = e.snapshot();
    for pitch in [91., -91., f32::NAN] {
        let mut entity = e
            .scene
            .entities
            .iter()
            .find(|e| e.id == "observer")
            .unwrap()
            .clone();
        entity.npc.as_mut().unwrap().spatial_vision = Some(SpatialVision {
            initial_pitch_degrees: pitch,
            follow_movement: true,
        });
        assert!(
            e.execute(
                Command::Upsert {
                    entity: Box::new(entity)
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), original);
    }
    let mut old = e.scene.clone();
    old.version = 22;
    assert!(Engine::new(old).is_err());
    let mut entity = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "observer")
        .unwrap()
        .clone();
    entity
        .npc
        .as_mut()
        .unwrap()
        .spatial_vision
        .as_mut()
        .unwrap()
        .initial_pitch_degrees = 90.;
    e.execute(
        Command::Upsert {
            entity: Box::new(entity),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(
        e.simulation.npcs["observer"].facing_pitch_degrees,
        Some(90.)
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        e.simulation.npcs["observer"].facing_pitch_degrees,
        Some(30.)
    );
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(
        e.simulation.npcs["observer"].facing_pitch_degrees,
        Some(90.)
    );
}
