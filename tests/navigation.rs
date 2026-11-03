use ignis::{
    engine::{Command, Engine},
    model::{BodyKind, Entity, NpcConfig, RoomRecipe, Scene, Transform},
    physics::Input,
};

fn box_entity(id: &str, position: [f32; 3], size: [f32; 3]) -> Entity {
    Entity {
        avatar: None,
        geometry: Default::default(),
        id: id.into(),
        name: id.into(),
        transform: Transform { position, size },
        color: [0.8, 0.2, 0.2],
        body: Some(BodyKind::Fixed),
        npc: None,
        abilities: None,
        vitality: None,
        faction: None,
        blocks_vision: None,
        sound_transmission: None,
        footsteps: None,
        locomotion: None,
        character_controller: None,
        trigger: None,
        visible: None,
    }
}
fn pursuit() -> Scene {
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut npc = box_entity("hunter", [0., 0.85, -5.], [0.7, 1.6, 0.7]);
    npc.body = Some(BodyKind::Npc);
    npc.npc = Some(NpcConfig {
        routine: None,
        dialogue: None,
        cooperation: None,
        circulation: None,
        mind: None,
        communication: None,
        patrol: None,
        target: Some("player".into()),
        targeting: None,
        recognition: None,
        sound_attention: None,
        objectives: None,
        territory: None,
        navigation_recovery: None,
        passage_priority: None,
        mobility: None,
        unreachable_policy: None,
        local_connections: None,
        flight: None,
        spatial_vision: None,
        surface: "floor".into(),
        speed: 3.,
        stop_distance: 1.2,
        enabled: true,
        awareness: Some(ignis::model::Awareness::Omniscient),
    });
    scene.entities.push(npc);
    scene
}
fn horizontal_distance(e: &Engine) -> f32 {
    let p = e.simulation.positions();
    ((p["hunter"][0] - p["player"][0]).powi(2) + (p["hunter"][2] - p["player"][2]).powi(2)).sqrt()
}
#[test]
fn pursuer_detours_with_body_clearance_and_replays() {
    let mut scene = pursuit();
    scene
        .entities
        .push(box_entity("barrier", [0., 1., 0.], [6., 2., 0.6]));
    let mut e = Engine::new(scene).unwrap();
    let mut excursion = 0_f32;
    for _ in 0..540 {
        e.advance(1).unwrap();
        let p = e.simulation.positions()["hunter"];
        excursion = excursion.max(p[0].abs());
        assert!(
            p[0].abs() >= 3.33 || p[2].abs() >= 0.63,
            "body entered wall: {p:?}"
        );
    }
    assert!(excursion > 3.4, "no detour: {excursion}");
    assert!(horizontal_distance(&e) < 1.25, "{:?}", e.snapshot());
    assert_eq!(e.simulation.navigation["hunter"].status, "arrived");
    let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    assert_eq!(
        e.scene
            .entities
            .iter()
            .find(|x| x.id == "hunter")
            .unwrap()
            .transform
            .position,
        [0., 0.85, -5.]
    );
}
#[test]
fn sealed_wall_stops_then_removal_and_undo_are_observable() {
    let mut scene = pursuit();
    scene
        .entities
        .push(box_entity("barrier", [0., 1., 0.], [16., 2., 0.6]));
    let mut e = Engine::new(scene).unwrap();
    e.advance(120).unwrap();
    assert_eq!(e.simulation.navigation["hunter"].status, "unreachable");
    assert!((e.simulation.positions()["hunter"][2] + 5.).abs() < 0.01);
    e.execute(
        Command::Remove {
            id: "barrier".into(),
        }
        .into(),
    )
    .unwrap();
    e.advance(360).unwrap();
    assert!(horizontal_distance(&e) < 1.25);
    e.execute(Command::Undo.into()).unwrap();
    e.advance(12).unwrap();
    assert_eq!(e.simulation.navigation["hunter"].status, "unreachable");
}
#[test]
fn narrow_gap_rejects_body_even_when_center_could_fit() {
    let mut scene = pursuit();
    for (id, x) in [("left", -4.1), ("right", 4.1)] {
        scene
            .entities
            .push(box_entity(id, [x, 1., 0.], [7.7, 2., 0.6]));
    }
    let mut e = Engine::new(scene).unwrap();
    e.advance(60).unwrap();
    assert_eq!(e.simulation.navigation["hunter"].status, "unreachable");
}
#[test]
fn moving_player_is_reacquired_and_disabled_npc_stays_put() {
    let mut e = Engine::new(pursuit()).unwrap();
    e.advance(360).unwrap();
    assert!(horizontal_distance(&e) < 1.25);
    e.input = Input {
        x: 1.,
        z: 0.,
        ..Default::default()
    };
    e.advance(60).unwrap();
    e.input = Input::default();
    e.advance(180).unwrap();
    assert!(horizontal_distance(&e) < 1.25, "{:?}", e.snapshot());
    assert!(e.simulation.positions()["hunter"][0] > 2.5);
    let mut npc = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "hunter")
        .unwrap()
        .clone();
    npc.npc.as_mut().unwrap().enabled = false;
    e.execute(Command::Upsert { entity: npc.into() }.into())
        .unwrap();
    e.advance(120).unwrap();
    assert_eq!(e.simulation.navigation["hunter"].status, "disabled");
    assert!((e.simulation.positions()["hunter"][2] + 5.).abs() < 0.01);
}
#[test]
fn invalid_references_are_atomic_and_legacy_scene_migrates() {
    let mut e = Engine::new(pursuit()).unwrap();
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::Remove {
                id: "player".into()
            }
            .into()
        )
        .is_err()
    );
    assert!(
        e.execute(Command::Remove { id: "floor".into() }.into())
            .is_err()
    );
    let mut npc = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "hunter")
        .unwrap()
        .clone();
    npc.npc.as_mut().unwrap().speed = f32::NAN;
    assert!(
        e.execute(Command::Upsert { entity: npc.into() }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let old: Scene = serde_json::from_str(include_str!("../examples/lab.scene.json")).unwrap();
    assert_eq!(old.version, 1);
    assert_eq!(
        Engine::new(old).unwrap().scene.version,
        ignis::model::SCENE_VERSION
    );
}
#[test]
fn off_surface_target_stops_and_scene_order_does_not_change_result() {
    let mut s = pursuit();
    let mut a = Engine::new(s.clone()).unwrap();
    s.entities.reverse();
    let mut b = Engine::new(s.clone()).unwrap();
    a.advance(160).unwrap();
    b.advance(160).unwrap();
    assert_eq!(a.simulation.positions(), b.simulation.positions());
    assert_eq!(a.simulation.navigation, b.simulation.navigation);
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position[1] = 5.;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.navigation["hunter"].reason,
        "target_off_surface"
    );
}

#[test]
fn visible_player_touching_walls_is_reachable_with_body_clearance() {
    for (player, hunter) in [
        ([0., 0.85, 0.65], [0., 0.85, 5.]),
        ([3.35, 0.85, 0.], [6., 0.85, 0.]),
        ([7.45, 0.85, 4.], [3., 0.85, 4.]),
        ([7.45, 0.85, 6.45], [4., 0.85, 4.]),
    ] {
        for width in [0.7, 1.6] {
            let mut s = pursuit();
            s.entities
                .push(box_entity("barrier", [0., 1., 0.], [6., 2., 0.6]));
            s.entities
                .iter_mut()
                .find(|e| e.id == "player")
                .unwrap()
                .transform
                .position = player;
            let npc = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
            npc.transform.position = hunter;
            npc.transform.size = [width, 1.6, width];
            let mut e = Engine::new(s).unwrap();
            e.advance(240).unwrap();
            let n = &e.simulation.navigation["hunter"];
            assert_eq!(
                n.status,
                "arrived",
                "player={player:?}, width={width}, {:?}",
                e.snapshot()
            );
            assert!(horizontal_distance(&e) <= n.effective_stop_distance + 0.025);
            assert!(
                !e.simulation
                    .contacts()
                    .iter()
                    .any(|(a, b)| (a == "hunter" || b == "hunter")
                        && (a == "barrier"
                            || b == "barrier"
                            || a.starts_with("wall_")
                            || b.starts_with("wall_")))
            );
        }
    }
}

#[test]
fn close_target_across_wall_requires_detour_not_false_arrival() {
    for barrier_width in [6., 16.] {
        let mut s = pursuit();
        s.entities.push(box_entity(
            "barrier",
            [0., 1., 0.],
            [barrier_width, 2., 0.6],
        ));
        s.entities
            .iter_mut()
            .find(|e| e.id == "player")
            .unwrap()
            .transform
            .position = [0., 0.85, 0.65];
        let npc = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
        npc.transform.position = [0., 0.85, -0.85];
        npc.npc.as_mut().unwrap().stop_distance = 3.;
        let mut e = Engine::new(s).unwrap();
        let mut excursion = 0_f32;
        for _ in 0..540 {
            e.advance(1).unwrap();
            let p = e.simulation.positions()["hunter"];
            excursion = excursion.max(p[0].abs());
            assert!(p[0].abs() >= 3.33 || p[2].abs() >= 0.63);
            if p[0].abs() < 3. && p[2] < 0. {
                assert_ne!(e.simulation.navigation["hunter"].status, "arrived");
            }
        }
        if barrier_width == 6. {
            assert!(excursion > 3.4);
            assert_eq!(e.simulation.navigation["hunter"].status, "arrived");
        } else {
            assert_eq!(e.simulation.navigation["hunter"].status, "unreachable");
            assert!(excursion < 0.01);
        }
    }
}
