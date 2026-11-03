//! Critical integration: guidance must not bypass physical limits or lose identity/state on save.
use glam::Vec3;
use ignis::{
    checkpoint::Checkpoint,
    engine::{Engine, ExecutionMode},
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::json;

fn actor(id: &str, start: [f32; 3], end: [f32; 3], curved: bool) -> Entity {
    let direction = (Vec3::from(end) - Vec3::from(start)).normalize();
    let mut value = json!({"id":id,"name":id,"body":"npc","color":[0.7,0.2,0.8],
        "transform":{"position":start,"size":[0.6,0.6,0.6]},
        "character_controller":{},
        "npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,
        "mobility":{"walk":true,"fly_speed":2},"circulation":{"radius":6,"horizon_seconds":2,"personal_space":0.2},
        "mind":{"intelligence":1},
        "objectives":[{"priority":0,"when":"always","action":{"kind":"guard_3d","point":end,"arrival_distance":0.2}}]}});
    if curved {
        value["npc"]["flight"] = json!({"mode":"curved","acceleration":4,"deceleration":4,
        "turn_degrees_per_second":180,"smoothing_seconds":0.05,"minimum_speed":0,"can_hover":true,"initial_direction":direction.to_array()});
    }
    serde_json::from_value(value).unwrap()
}
fn scene(curved: bool, vertical: bool) -> Scene {
    let mut s = RoomRecipe {
        width: 30.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.retain(|e| e.id != "player");
    let (a, b) = if vertical {
        ([0., 2., 0.], [0., 8., 0.])
    } else {
        ([-4., 4., 0.], [4., 4., 0.])
    };
    s.entities
        .extend([actor("a", a, b, curved), actor("b", b, a, curved)]);
    s.entities.push(serde_json::from_value(json!({"id":"falling-crate","name":"crate","body":"dynamic","color":[0.4,0.6,0.3],"transform":{"position":[0,5,2],"size":[0.6,0.6,0.6]}})).unwrap());
    s.spatial_network = serde_json::from_value(json!({"nodes":[(Vec3::from(a)-Vec3::Y*0.3).to_array(),(Vec3::from(b)-Vec3::Y*0.3).to_array()],
        "links":[{"from":0,"to":1,"mode":"fly"},{"from":1,"to":0,"mode":"fly"}]})).unwrap();
    s
}
#[test]
fn xyz_crossing_preserves_motor_identity_checkpoint_and_baseline_replay() {
    for (curved, vertical, hover) in [
        (false, false, true),
        (true, false, true),
        (false, true, true),
        (true, true, true),
        (true, false, false),
    ] {
        let mut s = scene(curved, vertical);
        if !hover {
            for e in &mut s.entities {
                if let Some(n) = e.npc.as_mut()
                    && let Some(ignis::flight::Profile::Curved {
                        minimum_speed,
                        can_hover,
                        ..
                    }) = n.flight.as_mut()
                {
                    *can_hover = false;
                    *minimum_speed = 0.8;
                }
            }
        }
        let mut template = s.entities.iter().find(|e| e.id == "a").unwrap().clone();
        template.id = "air-template".into();
        s.npc_templates.push(template);
        let mut e = Engine::new(s).unwrap();
        let identity = e.simulation.character_controllers["a"].instance;
        let mut old_velocity = [Vec3::ZERO; 2];
        let mut avoided = false;
        let mut distance = f32::INFINITY;
        for tick in 0..720 {
            let before = e.simulation.positions();
            e.advance(1).unwrap();
            let now = e.simulation.positions();
            let d = (Vec3::from(now["a"]) - Vec3::from(now["b"])).abs();
            distance = distance.min(d.max_element());
            assert!(
                d.max_element() > 0.57,
                "overlap curved={curved} vertical={vertical} tick={tick}: {now:?}"
            );
            for (i, id) in ["a", "b"].iter().enumerate() {
                let c = e.simulation.npcs[*id].circulation.unwrap();
                if tick == 0 && !vertical {
                    assert_eq!(
                        c.neighbors, 1,
                        "Only the dynamic crate is within sensing radius at launch"
                    );
                }
                avoided |= c.air.is_some() && c.reason == ignis::npc_circulation::Reason::Avoiding;
                assert!(e.simulation.npcs[*id].memory.is_none());
                assert!(Vec3::from(now[*id]).distance(before[*id].into()) < 2. / 60. + 0.002);
                if let Some(f) = e.simulation.mobility.states[*id].flight {
                    let v = Vec3::from(f.velocity);
                    assert!(v.length() <= 2.001);
                    assert!(
                        v.distance(old_velocity[i]) <= 4. / 60. + 0.001,
                        "acceleration {curved} {vertical} {tick}: {:?}->{v:?}",
                        old_velocity[i]
                    );
                    if v.length() > 0.01 && old_velocity[i].length() > 0.01 {
                        assert!(
                            v.normalize()
                                .dot(old_velocity[i].normalize())
                                .clamp(-1., 1.)
                                .acos()
                                <= 180_f32.to_radians() / 60. + 0.002
                        );
                    }
                    old_velocity[i] = v;
                }
            }
            assert_eq!(e.simulation.character_controllers["a"].instance, identity);
            if tick == 90 {
                let bytes = Checkpoint::capture(&e).unwrap().encode().unwrap();
                let mut restored =
                    Engine::from_checkpoint(&bytes, Some(ExecutionMode::Recorded)).unwrap();
                assert_eq!(restored.simulation.npcs, e.simulation.npcs);
                for _ in 0..45 {
                    restored.advance(1).unwrap();
                    e.advance(1).unwrap();
                }
                assert_eq!(restored.simulation.positions(), e.simulation.positions());
                assert_eq!(
                    restored.simulation.mobility.states,
                    e.simulation.mobility.states
                );
                assert_eq!(restored.simulation.npcs, e.simulation.npcs);
                assert_eq!(
                    Engine::verify_replay(&restored.replay()).unwrap()["matched"],
                    true
                );
                old_velocity = ["a", "b"].map(|id| {
                    e.simulation.mobility.states[id]
                        .flight
                        .map_or(Vec3::ZERO, |f| f.velocity.into())
                });
            }
        }
        assert!(
            avoided,
            "no avoidance {curved} {vertical}: {:?}",
            e.simulation.positions()
        );
        for id in ["a", "b"] {
            assert_eq!(
                e.simulation.mobility.states[id].phase,
                ignis::mobility::Phase::Arrived,
                "{curved} {vertical} {id}: {:?} min={distance}",
                e.simulation.mobility.states[id]
            );
        }
        assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
        e.execute(
            serde_json::from_value::<ignis::engine::Command>(
                json!({"op":"despawn_actor","actor":"a"}),
            )
            .unwrap()
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
        e.execute(
            serde_json::from_value::<ignis::engine::Command>(
                json!({"op":"spawn_actor","template":"air-template","position":[-8,0.31,3]}),
            )
            .unwrap()
            .into(),
        )
        .unwrap();
        e.advance(1).unwrap();
        let (born, controller) = e
            .simulation
            .character_controllers
            .iter()
            .find(|(id, _)| id.as_str() != "b")
            .unwrap();
        assert_ne!(controller.instance, identity);
        assert!(e.simulation.mobility.states[born].flight.is_none());
        assert!(
            e.simulation.npcs[born]
                .circulation
                .is_none_or(|c| c.air.is_none())
        );
        assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    }
    // A newly installed KCC has no cached grounded contact yet. Admission must
    // use physical support, or the very first jump bypasses launch deferral.
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.retain(|e| e.id != "player");
    let mut jumper = actor("jumper", [-3., 0.31, 0.], [-1., 0.3, 0.], false);
    jumper.npc.as_mut().unwrap().mobility =
        Some(serde_json::from_value(json!({"walk":true,"jump_speed":6})).unwrap());
    let mut blocker = actor("blocker", [-2., 2.1, 0.], [-2., 2.1, 0.], false);
    blocker.character_controller = None;
    s.entities.extend([jumper, blocker]);
    s.spatial_network = serde_json::from_value(
        json!({"nodes":[[-3,0,0],[-1,0,0]],"links":[{"from":0,"to":1,"mode":"jump"}]}),
    )
    .unwrap();
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["jumper"].circulation.unwrap().reason,
        ignis::npc_circulation::Reason::LaunchDeferred
    );
    assert!(e.simulation.mobility.states["jumper"].motion.is_none());
    assert!(!e.simulation.character_controllers["jumper"].airborne);
    e.execute(
        serde_json::from_value::<ignis::engine::Command>(
            json!({"op":"despawn_actor","actor":"blocker"}),
        )
        .unwrap()
        .into(),
    )
    .unwrap();
    // Preserve the existing 60-tick planner retry budget after yielding.
    e.advance(65).unwrap();
    assert!(e.simulation.character_controllers["jumper"].airborne);
    assert_eq!(
        e.simulation.mobility.states["jumper"].motor_mode,
        Some(ignis::mobility::Mode::Jump)
    );
    e.advance(240).unwrap();
    assert_eq!(
        e.simulation.mobility.states["jumper"].phase,
        ignis::mobility::Phase::Arrived
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
