use ignis::{
    engine::{Command, Engine},
    meshes,
    model::Scene,
    network::replica::{Frame, Replica},
    npc_presentation::Phase,
    service::Service,
};
fn call(s: &mut Service, c: Command) {
    s.execute(c.into()).unwrap();
}
fn move_to(s: &mut Service, actor: &str) {
    call(s,Command::OrderNpc{actor:actor.into(),order:serde_json::from_value(serde_json::json!({"kind":"move","position":[3,0.81,-2],"arrival_distance":0.2,"for_ticks":600,"formation":"offsets","spacing":1,"facing_degrees":0})).unwrap()});
}
// Protects movement ownership, dependency/identity persistence and atomic public
// decode, all of which can corrupt gameplay if an animation applies twice.
#[test]
fn root_contacts_cast_lifecycle_checkpoint_and_wire_share_one_body() {
    let mut scene: Scene = serde_json::from_str(include_str!(
        "../examples/npc-presentation-complete.scene.json"
    ))
    .unwrap();
    scene.entities.push(serde_json::from_value(serde_json::json!({"id":"root_blocker","name":"Moving physical blocker","body":"dynamic","transform":{"position":[1,4,-2],"size":[0.6,2,3]},"color":[0.4,0.4,0.4]})).unwrap());
    scene
        .entities
        .iter_mut()
        .find(|e| e.id == "a")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .circulation = None;
    scene
        .entities
        .iter_mut()
        .find(|e| e.id == "a")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .speed = 2.;
    let mut template = scene.entities.iter().find(|e| e.id == "a").unwrap().clone();
    template.id = "root_template".into();
    template.npc.as_mut().unwrap().objectives = serde_json::from_value(
        serde_json::json!([{"priority":0,"when":"always","action":{"kind":"wait"}}]),
    )
    .unwrap();
    scene.npc_templates = vec![template];
    let mut value = serde_json::to_value(&scene).unwrap();
    let meditate = value["ability_catalog"]["abilities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|a| a["id"] == "meditate")
        .unwrap();
    meditate["movement"] = serde_json::json!({"mode":"lock"});
    scene = serde_json::from_value(value).unwrap();
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), scene).unwrap();
    assert_eq!(
        s.execute(Command::InspectNpcAudioOutput.into())
            .unwrap()
            .data["enabled"],
        false
    );
    move_to(&mut s, "a");
    call(&mut s, Command::Step { ticks: 30 });
    let position = s.engine.simulation.position("a").unwrap();
    let elapsed = s.engine.simulation.tick - s.engine.simulation.presentation.motion["a"].started;
    let expected_distance = elapsed as f32 * 1.5 / 60.;
    assert!(elapsed >= 20);
    assert!(
        (position[0] + 3. - expected_distance).abs() < 0.005,
        "physical root distance:{position:?}, elapsed:{elapsed}, motion:{:?}",
        s.engine.simulation.presentation.motion["a"]
    );
    let poses = s.engine.simulation.presentation_poses();
    let pose = *poses
        .iter()
        .find(|p| {
            p.actor
                == s.engine.simulation.presentation.actors["a"]
                    .instance
                    .instance
        })
        .unwrap();
    let drive = s.engine.simulation.presentation.motion["a"];
    assert!(
        glam::Vec2::from(drive.intended).length() - glam::Vec2::from(drive.command).length() > 0.4
    );
    assert_eq!(pose.strip_root, Some(0));
    assert!(pose.joint_count >= 3);
    assert!(pose.ik_limited < pose.joint_count / 3);
    assert_eq!(pose.clip_tick, (elapsed % 60) as u32);
    let document = &s.engine.scene.meshes;
    let mut placed = meshes::Frame::default();
    placed.update(document, 30);
    placed
        .append_avatars(document, &[pose], |_| Some((position, [1.; 3])))
        .unwrap();
    let mut wrong = pose;
    wrong.strip_root = None;
    let mut doubled = meshes::Frame::default();
    doubled.update(document, 30);
    doubled
        .append_avatars(document, &[wrong], |_| Some((position, [1.; 3])))
        .unwrap();
    let delta = glam::Vec3::from(placed.vertices[0].position)
        - glam::Vec3::from(doubled.vertices[0].position);
    assert!(
        (delta.length() - pose.clip_tick as f32 * 1.5 / 60.).abs() < 0.001,
        "strip must remove exactly one visual root displacement:{delta:?}"
    );
    let motion = s.engine.simulation.presentation.motion.clone();
    let saved_poses = s.engine.simulation.presentation_poses();
    call(
        &mut s,
        Command::SaveCheckpoint {
            path: "state.ignis-save".into(),
        },
    );
    call(&mut s, Command::Step { ticks: 20 });
    let continued = s.engine.simulation.presentation.clone();
    let end = s.engine.simulation.presentation_poses();
    call(
        &mut s,
        Command::LoadCheckpoint {
            path: "state.ignis-save".into(),
            mode: None,
        },
    );
    assert_eq!(s.engine.simulation.presentation.motion, motion);
    assert_eq!(s.engine.simulation.presentation_poses(), saved_poses);
    call(&mut s, Command::Step { ticks: 20 });
    assert_eq!(s.engine.simulation.presentation, continued);
    assert_eq!(s.engine.simulation.presentation_poses(), end);
    assert_eq!(
        Engine::verify_replay(&s.engine.replay()).unwrap()["matched"],
        true
    );
    // Content-specific validation must run before full-frame clear or actor patches.
    let frame = Frame::capture(&s.engine, 11, None).unwrap();
    let mut replica = Replica::new(11);
    replica.apply(&frame.encode(None).unwrap()).unwrap();
    assert_eq!(replica.avatars(), frame.avatars);
    let original = replica.clone();
    let mut invalid = frame.clone();
    invalid.tick += 1;
    invalid.avatars[0].joint_count = 1;
    invalid.avatars[0].joints[0] = ignis::avatar_ik::Joint {
        bone: 31,
        translation: [0.; 3],
        rotation: [0., 0., 0., 1.],
    };
    assert!(
        replica
            .apply_with_avatars(&invalid.encode(None).unwrap(), |poses| {
                for p in poses {
                    p.validate_asset(&s.engine.scene.meshes.assets[p.asset as usize])?;
                }
                Ok(())
            })
            .is_err()
    );
    assert_eq!(replica, original);
    call(&mut s, Command::Step { ticks: 120 });
    let blocked = s.engine.simulation.position("a").unwrap();
    let controller = &s.engine.simulation.character_controllers["a"];
    assert!(
        blocked[0] < -0.85,
        "root tunneled through a falling physical blocker:{blocked:?}"
    );
    assert!(
        controller.desired_translation[0] > 0. && controller.corrected_translation[0].abs() < 0.001
    );
    assert!(s.engine.simulation.presentation.motion.contains_key("a"));
    let before = s.engine.simulation.position("a").unwrap();
    call(
        &mut s,
        Command::ActivateAbility {
            actor: "a".into(),
            slot: 1,
            target: None,
        },
    );
    call(&mut s, Command::Step { ticks: 1 });
    assert_eq!(
        s.engine.simulation.presentation.actors["a"].phase,
        Phase::Cast
    );
    assert!(!s.engine.simulation.presentation.motion.contains_key("a"));
    assert!((s.engine.simulation.position("a").unwrap()[0] - before[0]).abs() < 0.001);
    let old = s.engine.simulation.presentation.actors["a"];
    call(
        &mut s,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("a".into()),
        },
    );
    call(&mut s, Command::Step { ticks: 1 });
    assert_eq!(
        s.engine.simulation.presentation.actors["a"].phase,
        Phase::Dead
    );
    call(
        &mut s,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: Some("a".into()),
        },
    );
    call(&mut s, Command::Step { ticks: 1 });
    assert_eq!(
        s.engine.simulation.presentation.actors["a"].life,
        old.life + 1
    );
    move_to(&mut s, "a");
    call(&mut s, Command::Step { ticks: 1 });
    assert_eq!(
        s.engine.simulation.presentation.motion["a"].life,
        old.life + 1
    );
    call(&mut s, Command::DespawnActor { actor: "a".into() });
    call(
        &mut s,
        Command::SpawnActor {
            template: "root_template".into(),
            position: [-3., 0.81, -2.],
            target: None,
        },
    );
    call(&mut s, Command::Step { ticks: 1 });
    assert!(
        s.engine
            .simulation
            .presentation
            .motion
            .values()
            .all(|m| m.instance != old.instance)
    );
    let name = s
        .engine
        .simulation
        .presentation
        .actors
        .iter()
        .find(|(_, a)| {
            a.instance != old.instance
                && a.instance.instance
                    != s.engine.simulation.presentation.actors["b"]
                        .instance
                        .instance
                && a.instance.instance
                    != s.engine.simulation.presentation.actors["flyer"]
                        .instance
                        .instance
        })
        .unwrap()
        .0
        .clone();
    let tick = s.engine.simulation.tick;
    move_to(&mut s, &name);
    call(&mut s, Command::Step { ticks: 1 });
    let new = s.engine.simulation.presentation.motion[&name];
    assert_ne!(new.instance, old.instance);
    assert!(new.started >= tick);
    assert_eq!(new.life, 0);
    assert_eq!(
        Engine::verify_replay(&s.engine.replay()).unwrap()["matched"],
        true
    );
}
