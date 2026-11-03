use glam::Vec3;
use ignis::{
    abilities::Aim,
    engine::{Command, Engine},
    geometry::{Geometry, Rise, Volume},
    model::{BodyKind, Entity, RoomRecipe, Scene},
};
use rapier3d::prelude::*;
use serde_json::{Value, json};

fn wedge() -> Entity {
    serde_json::from_value(json!({"id":"ramp","name":"Ramp","geometry":{"kind":"wedge","rise":"positive_x"},"body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[0,1,0],"size":[6,2,4]},"sound_transmission":0})).unwrap()
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.push(wedge());
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn query(e: &mut Engine, start: [f32; 3], end: [f32; 3], half: [f32; 3], interior: bool) -> Value {
    run(
        e,
        Command::QueryGeometry {
            entity: "ramp".into(),
            start,
            end,
            half_extent: half,
            interior,
        },
    )
}
fn actor(id: &str, x: f32, z: f32) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[x,0.8,z],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","target":"player","speed":2,"stop_distance":0.3,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]},"vitality":{"initial":100,"maximum":100,"scale":1}})).unwrap()
}
fn combat(x: f32) -> Scene {
    let mut s = scene();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [x, 0.8, -4.];
    p.abilities = Some(
        serde_json::from_value(json!({"resources":[],"grants":["direct","shot","area"]})).unwrap(),
    );
    s.entities.push(actor("observer", x, 4.));
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[
 {"id":"direct","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":20}]},
 {"id":"shot","selection":"spatial","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"delivery":{"kind":"projectile","speed":30,"half_extent":0.05,"max_distance":20},"effects":[{"kind":"damage","recipient":"target","amount":20}]},
 {"id":"area","selection":"spatial","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"area":{"radius":9,"max_targets":9,"include_source":false,"occlusion":true},"effects":[{"kind":"damage","recipient":"target","amount":20}]}
 ]})).unwrap();
    s
}
fn hp(e: &Engine) -> i64 {
    let a = e
        .simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.simulation.definitions().entities[i as usize].id == "observer")
        .unwrap();
    e.simulation.vitality.state.actors[a].unwrap().units
}

#[test]
fn four_wedges_match_rapier_point_and_ray_queries_including_void() {
    for rise in [
        Rise::PositiveX,
        Rise::NegativeX,
        Rise::PositiveZ,
        Rise::NegativeZ,
    ] {
        let g = Geometry::Wedge { rise };
        let size = [6., 2., 4.];
        let vertices = g.vertices(size).map(|p| point![p.x, p.y, p.z]);
        let shape = SharedShape::convex_hull(&vertices).unwrap();
        let pose = Isometry::translation(1., 3., -2.);
        let v = Volume {
            geometry: g,
            center: Vec3::new(1., 3., -2.),
            size: size.into(),
        };
        for ix in -7..8 {
            for iy in -5..6 {
                for iz in -5..6 {
                    let p = Vec3::new(
                        ix as f32 * 0.47 + 0.031,
                        iy as f32 * 0.31 + 0.023,
                        iz as f32 * 0.49 + 0.017,
                    ) + v.center;
                    assert_eq!(
                        v.cast(p, Vec3::ZERO, Vec3::ZERO, false, 0.).is_some(),
                        shape.contains_point(&pose, &point![p.x, p.y, p.z]),
                        "{rise:?} {p}"
                    );
                }
            }
        }
        for x in [-2., 0., 2.] {
            for z in [-1., 1.] {
                let start = Vec3::new(x, 5., z) + v.center;
                let delta = Vec3::new(0., -10., 0.);
                let ray = Ray::new(
                    point![start.x, start.y, start.z],
                    vector![delta.x, delta.y, delta.z],
                );
                let expected = shape.cast_ray(&pose, &ray, 1., true).unwrap();
                let found = v.cast(start, delta, Vec3::ZERO, false, 0.).unwrap();
                assert!((found - expected).abs() < 1e-6);
            }
        }
    }
}
#[test]
fn expanded_wedge_matches_rapier_cuboid_overlap() {
    for rise in [
        Rise::PositiveX,
        Rise::NegativeX,
        Rise::PositiveZ,
        Rise::NegativeZ,
    ] {
        let g = Geometry::Wedge { rise };
        let size = [6., 2., 4.];
        let vertices = g.vertices(size).map(|p| point![p.x, p.y, p.z]);
        let shape = SharedShape::convex_hull(&vertices).unwrap();
        let v = Volume {
            geometry: g,
            center: Vec3::ZERO,
            size: size.into(),
        };
        let half = Vec3::new(0.37, 0.81, 0.23);
        let cube = Cuboid::new(vector![half.x, half.y, half.z]);
        for ix in -10..11 {
            for iy in -6..7 {
                let p = Vec3::new(ix as f32 * 0.37 + 0.013, iy as f32 * 0.29 + 0.017, 0.031);
                let expected = rapier3d::parry::query::intersection_test(
                    &Isometry::identity(),
                    shape.as_ref(),
                    &Isometry::translation(p.x, p.y, p.z),
                    &cube,
                )
                .unwrap();
                assert_eq!(
                    v.cast(p, Vec3::ZERO, half, false, 0.).is_some(),
                    expected,
                    "{rise:?} {p}"
                );
            }
        }
    }
}
#[test]
fn boundary_policies_sweeps_and_height_projection_are_explicit() {
    let e = wedge();
    let v = Volume::entity(&e, e.transform.position.into());
    let a = Vec3::new(-2., 1.5, -3.);
    let delta = Vec3::new(0., 0., 6.);
    assert!(v.cast(a, delta, Vec3::ZERO, false, 0.).is_none());
    assert!(
        v.cast(Vec3::new(2., 1., -3.), delta, Vec3::ZERO, false, 0.)
            .is_some()
    );
    assert!(
        v.cast(Vec3::new(-2., 0.6, -3.), delta, Vec3::splat(0.4), false, 0.)
            .is_some()
    );
    let p = Vec3::new(0., 1., 0.);
    assert!(v.cast(p, Vec3::ZERO, Vec3::ZERO, false, 0.).is_some());
    assert!(v.cast(p, Vec3::ZERO, Vec3::ZERO, true, 0.).is_none());
    assert!((v.top(-2., 0.).unwrap() - 1. / 3.).abs() < 1e-6);
    assert!(v.top(4., 0.).is_none());
    let (min, max) = v.horizontal_bounds(1.5, 2.).unwrap();
    assert!((min.x - 1.5).abs() < 1e-6);
    assert_eq!(max.x, 3.);
}
#[test]
fn query_is_read_only_and_actor_trigger_and_surface_combinations_are_atomic() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    assert!(query(&mut e, [-2., 1.5, -3.], [-2., 1.5, 3.], [0.; 3], false)["fraction"].is_null());
    assert_eq!(e.snapshot(), before);
    for half in [[-1., 0., 0.], [f32::NAN, 0., 0.], [501., 0., 0.]] {
        assert!(
            e.execute(
                Command::QueryGeometry {
                    entity: "ramp".into(),
                    start: [0.; 3],
                    end: [1.; 3],
                    half_extent: half,
                    interior: false
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    for body in [BodyKind::Player, BodyKind::Npc, BodyKind::Dynamic] {
        let mut bad = wedge();
        bad.body = Some(body);
        assert!(
            e.execute(
                Command::Upsert {
                    entity: Box::new(bad)
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut old = scene();
    old.version = 74;
    assert!(old.validate().is_err());
    let mut s = scene();
    s.entities.push(actor("observer", -2., 4.));
    s.entities.last_mut().unwrap().npc.as_mut().unwrap().surface = "ramp".into();
    assert!(s.validate().is_err());
    let mut bad = wedge();
    bad.trigger =
        Some(serde_json::from_value(json!({"enabled":true,"targets":["player"]})).unwrap());
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(bad)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
#[test]
fn physical_fall_lands_on_real_slope_and_replay_and_edit_restore_geometry() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-1.5, 5., 0.];
    let mut e = Engine::new(s).unwrap();
    e.advance(90).unwrap();
    let y = e.simulation.position("player").unwrap()[1];
    assert!(y > 1.1 && y < 1.7, "{y}");
    replay(&e);
    let r = e.replay();
    let prefix = Engine::replay_at(&r, Some(20)).unwrap();
    assert!(
        !prefix
            .scene
            .entities
            .iter()
            .find(|e| e.id == "ramp")
            .unwrap()
            .geometry
            .is_box()
    );
    let mut corrupt = r.clone();
    corrupt
        .scene
        .entities
        .iter_mut()
        .find(|e| e.id == "ramp")
        .unwrap()
        .geometry = Geometry::Box;
    assert_eq!(Engine::verify_replay(&corrupt).unwrap()["matched"], false);
    let mut b = wedge();
    b.geometry = Geometry::Box;
    run(
        &mut e,
        Command::Upsert {
            entity: Box::new(b),
        },
    );
    run(&mut e, Command::Undo);
    assert!(
        !e.scene
            .entities
            .iter()
            .find(|e| e.id == "ramp")
            .unwrap()
            .geometry
            .is_box()
    );
    run(&mut e, Command::Reset);
    e.advance(90).unwrap();
    assert!((e.simulation.position("player").unwrap()[1] - y).abs() < 1e-6);
}
#[test]
fn perception_and_sound_ignore_void_and_block_solid() {
    for (x, seen) in [(-2., true), (2., false)] {
        let mut e = Engine::new(combat(x)).unwrap();
        e.advance(1).unwrap();
        assert_eq!(e.simulation.npcs["observer"].observed_this_tick, seen);
        replay(&e);
        let mut s = combat(x);
        if let Some(ignis::model::Awareness::Perceptive {
            vision_range,
            hearing,
            ..
        }) = &mut s
            .entities
            .iter_mut()
            .find(|e| e.id == "observer")
            .unwrap()
            .npc
            .as_mut()
            .unwrap()
            .awareness
        {
            *vision_range = 0.1;
            *hearing = Some(serde_json::from_value(json!({"range":20})).unwrap());
        }
        let mut e = Engine::new(s).unwrap();
        run(
            &mut e,
            Command::EmitNoise {
                origin: ignis::noise::NoiseOrigin::Position {
                    position: [x, 0.8, -4.],
                },
                radius: 20.,
            },
        );
        e.advance(1).unwrap();
        assert_eq!(e.simulation.npcs["observer"].heard_this_tick, seen);
        replay(&e);
    }
}
#[test]
fn direct_projectile_and_area_use_the_same_solid() {
    for (x, clear) in [(-2., true), (2., false)] {
        for slot in 1..=3 {
            let mut e = Engine::new(combat(x)).unwrap();
            if slot == 1 {
                run(
                    &mut e,
                    Command::ActivateAbility {
                        actor: "player".into(),
                        slot,
                        target: Some("observer".into()),
                    },
                );
            } else {
                run(
                    &mut e,
                    Command::AimAbility {
                        actor: "player".into(),
                        slot,
                        aim: Aim::Point {
                            position: if slot == 2 {
                                [x, 0.8, 4.]
                            } else {
                                [x, 0.8, -4.]
                            },
                        },
                    },
                );
            }
            e.advance(40).unwrap();
            assert_eq!(hp(&e), if clear { 80 } else { 100 }, "x{x} slot{slot}");
            replay(&e);
        }
    }
}
#[test]
fn sloped_support_is_explicit_and_hover_birth_uses_empty_volume() {
    let mut s = scene();
    let mut t = actor("flyer", 0., 0.);
    t.npc.as_mut().unwrap().target = None;
    t.npc.as_mut().unwrap().enabled = false;
    t.npc.as_mut().unwrap().mobility =
        Some(serde_json::from_value(json!({"walk":false,"fly_speed":2})).unwrap());
    t.npc.as_mut().unwrap().local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap());
    t.transform.size = [0.2, 0.2, 0.2];
    s.npc_templates.push(t);
    let mut e = Engine::new(s).unwrap();
    let q = run(
        &mut e,
        Command::QuerySpawnSupport {
            template: "flyer".into(),
            position: [-2., 3., 0.],
            max_drop: 5.,
        },
    );
    assert_eq!(q["report"]["status"], "sloped_surface");
    assert!((q["report"]["feet"][1].as_f64().unwrap() - 1. / 3.).abs() < 1e-6);
    run(
        &mut e,
        Command::SpawnActor {
            template: "flyer".into(),
            position: [-2., 1.5, 0.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.population.receipts.back().unwrap().outcome,
        ignis::population::Outcome::Spawned
    );
    replay(&e);
}
#[test]
fn flight_clearance_crosses_void_but_cannot_cross_solid() {
    for (x, can_pass) in [(-2., true), (2., false)] {
        let mut s = scene();
        s.entities.retain(|e| e.id != "player");
        let mut a = actor("flyer", x, -3.);
        a.transform.size = [0.2, 0.2, 0.2];
        a.transform.position[1] = 1.2;
        let n = a.npc.as_mut().unwrap();
        n.target = None;
        n.mobility = Some(serde_json::from_value(json!({"walk":false,"fly_speed":2})).unwrap());
        n.local_connections = Some(
            serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap(),
        );
        n.objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[x,1.2,3],"arrival_distance":0.2}}])).unwrap());
        s.entities.push(a);
        let mut e = Engine::new(s).unwrap();
        e.advance(240).unwrap();
        assert_eq!(e.simulation.position("flyer").unwrap()[2] > 2.7, can_pass);
        replay(&e);
    }
}

#[test]
fn planar_route_goes_around_solid_without_treating_slope_as_floor() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-8., 0.8, 8.];
    let mut a = actor("walker", 2., -4.);
    a.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard","point":[2,4],"arrival_distance":0.2}}])).unwrap());
    s.entities.push(a);
    let mut e = Engine::new(s).unwrap();
    let v = Volume::entity(&wedge(), Vec3::new(0., 1., 0.));
    let mut detour = false;
    for _ in 0..480 {
        e.advance(1).unwrap();
        let p = Vec3::from(e.simulation.position("walker").unwrap());
        detour |= p.x > 3.3;
        assert!(
            v.cast(p, Vec3::ZERO, Vec3::new(0.27, 0.77, 0.27), true, 0.)
                .is_none(),
            "{p}"
        );
    }
    let p = Vec3::from(e.simulation.position("walker").unwrap());
    assert!(detour);
    assert!(p.distance(Vec3::new(2., 0.8, 4.)) < 0.4, "{p}");
    replay(&e);
}
