use glam::Vec3;
use ignis::{
    engine::{Command, Engine},
    mobility::Phase,
    model::{BodyKind, Entity, RoomRecipe, Scene},
    walk_support::{Support, query},
};
use serde_json::json;

fn surface(id: &str, x: f32, z: f32, width: f32, depth: f32) -> Entity {
    serde_json::from_value(
        json!({"id":id,"name":id,"body":"fixed","color":[0.4,0.5,0.6],
        "transform":{"position":[x,1,z],"size":[width,2,depth]}}),
    )
    .unwrap()
}
fn scene(local: bool) -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    for (id, x) in [("left", -3.), ("bridge", 0.), ("right", 3.)] {
        s.entities.push(surface(id, x, 0., 3., 2.));
    }
    let mut n = json!({"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,
        "mobility":{"walk":true},
        "objectives":[{"priority":1,"when":"always","action":{"kind":"guard_3d","point":[3,2.8,0],"arrival_distance":0.2}}]});
    if local {
        n["local_connections"] = json!({"walk":true,"fly":false,"max_distance":8});
    }
    s.entities.push(
        serde_json::from_value(
            json!({"id":"actor","name":"actor","body":"npc","color":[0.7,0.2,0.8],
        "transform":{"position":[-3,2.8,0],"size":[0.6,1.6,0.6]},"npc":n}),
        )
        .unwrap(),
    );
    if !local {
        s.spatial_network = serde_json::from_value(
            json!({"nodes":[[-3,2,0],[3,2,0]],"links":[{"from":0,"to":1,"mode":"walk"}]}),
        )
        .unwrap();
    }
    s
}
fn actor(s: &Scene) -> &Entity {
    s.entities.iter().find(|e| e.id == "actor").unwrap()
}
fn supported(s: &Scene, a: [f32; 3], b: [f32; 3]) -> Support {
    query(s, actor(s), a.into(), b.into())
}

#[test]
fn composed_support_is_order_independent_and_handles_point_seams_and_both_axes() {
    let mut s = scene(false);
    assert_eq!(
        supported(&s, [-3., 2., 0.], [3., 2., 0.]),
        Support::Continuous
    );
    assert_eq!(
        supported(&s, [1.5, 2., 0.], [1.5, 2., 0.]),
        Support::Continuous
    );
    s.entities.reverse();
    assert_eq!(
        supported(&s, [3., 2., 0.], [-3., 2., 0.]),
        Support::Continuous
    );
    for e in &mut s.entities {
        e.transform.position.swap(0, 2);
        e.transform.size.swap(0, 2);
    }
    assert_eq!(
        supported(&s, [0., 2., -3.], [0., 2., 3.]),
        Support::Continuous
    );
}

#[test]
fn full_swept_footprint_rejects_tiny_holes_and_accepts_diagonal_tile_union() {
    let mut s = scene(false);
    s.entities.retain(|e| e.npc.is_some() || e.id == "floor");
    // Four quadrants meet at a corner. A diagonal walk is covered across the shared seam.
    for (id, x, z) in [
        ("a", -1., -1.),
        ("b", 1., -1.),
        ("c", -1., 1.),
        ("d", 1., 1.),
    ] {
        s.entities.push(surface(id, x, z, 2., 2.));
    }
    assert_eq!(
        supported(&s, [-1., 2., -1.], [1., 2., 1.]),
        Support::Continuous
    );
    s.entities.retain(|e| e.npc.is_some() || e.id == "floor");
    for (id, c) in [("a", -1.), ("b", 0.), ("c", 1.)] {
        s.entities.push(surface(id, c, c, 2.2, 2.2));
    }
    // Support covers the diagonal sweep, not its entire axis-aligned bounding rectangle.
    assert_eq!(
        supported(&s, [-1.5, 2., -1.5], [1.5, 2., 1.5]),
        Support::Continuous
    );
    s.entities.retain(|e| e.npc.is_some() || e.id == "floor");
    // A 1cm hole at x=[0,.01],z=[.10,.11] misses center/corner ray samples.
    s.entities.extend([
        surface("left", -1.5, 0., 3., 2.),
        surface("right", 1.505, 0., 2.99, 2.),
        surface("below", 0.005, -0.45, 0.01, 1.1),
        surface("above", 0.005, 0.555, 0.01, 0.89),
    ]);
    assert_eq!(supported(&s, [-1., 2., 0.], [1., 2., 0.]), Support::Gap);
    s.entities
        .push(surface("patch", 0.005, 0.105, 0.012, 0.012));
    assert_eq!(
        supported(&s, [-1., 2., 0.], [1., 2., 0.]),
        Support::Continuous
    );
}

#[test]
fn gaps_width_height_and_non_fixed_support_are_not_traversable() {
    for kind in ["gap", "narrow", "lower", "visual", "dynamic"] {
        let mut s = scene(false);
        let b = s.entities.iter_mut().find(|e| e.id == "bridge").unwrap();
        match kind {
            "gap" => b.transform.size[0] = 2.99,
            "narrow" => b.transform.size[2] = 0.4,
            "lower" => b.transform.position[1] -= 0.2,
            "visual" => b.body = None,
            "dynamic" => b.body = Some(BodyKind::Dynamic),
            _ => unreachable!(),
        }
        assert_eq!(
            supported(&s, [-3., 2., 0.], [3., 2., 0.]),
            Support::Gap,
            "{kind}"
        );
        let mut e = Engine::new(s).unwrap();
        e.advance(300).unwrap();
        assert_eq!(
            e.simulation.mobility.states["actor"].phase,
            Phase::Unreachable,
            "{kind}"
        );
        assert_eq!(
            e.simulation.mobility.rejection_reasons["actor"][0],
            Some("walk_support_gap")
        );
        assert!(e.simulation.position("actor").unwrap()[0] < -2.9);
    }
    let s = scene(false);
    assert_eq!(
        supported(&s, [-3., 2., 0.], [3., 2.1, 0.]),
        Support::HeightMismatch
    );
}

#[test]
fn support_work_is_bounded_without_disabling_single_surface_fast_path() {
    let mut s = scene(false);
    s.entities.retain(|e| e.npc.is_some() || e.id == "floor");
    for i in 0..65 {
        s.entities.push(surface(
            &format!("tile{i}"),
            i as f32 * 0.1 - 3.2,
            0.,
            0.12,
            2.,
        ));
    }
    assert_eq!(
        supported(&s, [-2.8, 2., 0.], [2.8, 2., 0.]),
        Support::Continuous
    );
    assert_eq!(
        supported(&s, [-3., 2., 0.], [3., 2., 0.]),
        Support::ComplexityLimit
    );
    s.entities.reverse();
    assert_eq!(
        supported(&s, [-3., 2., 0.], [3., 2., 0.]),
        Support::ComplexityLimit
    );
    s.entities.push(surface("whole", 0., 0., 8., 2.));
    assert_eq!(
        supported(&s, [-3., 2., 0.], [3., 2., 0.]),
        Support::Continuous
    );
}

#[test]
fn links_and_local_connections_walk_across_physical_seams_and_replay() {
    for local in [false, true] {
        let mut e = Engine::new(scene(local)).unwrap();
        let mut previous = Vec3::from(e.simulation.position("actor").unwrap());
        let mut prefix = None;
        for tick in 1..=240 {
            e.advance(1).unwrap();
            let now = Vec3::from(e.simulation.position("actor").unwrap());
            assert!(
                now.distance(previous) < 0.05 && now.y > 2.7,
                "{local}: {now:?}"
            );
            previous = now;
            if tick == 90 {
                prefix = Some(e.snapshot()["runtime_positions"].clone());
            }
        }
        assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
        assert!(previous.x > 2.79);
        let r = e.replay();
        assert_eq!(Engine::verify_replay(&r).unwrap()["matched"], true);
        assert_eq!(
            Engine::replay_at(&r, Some(90)).unwrap().snapshot()["runtime_positions"],
            prefix.unwrap()
        );
    }
}

#[test]
fn seam_destination_bridge_removal_and_undo_preserve_physical_contract() {
    let mut s = scene(true);
    let a = s.entities.iter_mut().find(|e| e.id == "actor").unwrap();
    if let ignis::objectives::Action::Guard3d { point, .. } =
        &mut a.npc.as_mut().unwrap().objectives.as_mut().unwrap()[0].action
    {
        point[0] = 1.5;
    }
    let mut e = Engine::new(s).unwrap();
    e.advance(240).unwrap();
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
    e.execute(
        Command::Remove {
            id: "bridge".into(),
        }
        .into(),
    )
    .unwrap();
    e.advance(240).unwrap();
    assert_eq!(
        e.simulation.mobility.states["actor"].phase,
        Phase::Unreachable
    );
    e.execute(Command::Undo.into()).unwrap();
    e.advance(240).unwrap();
    assert_eq!(e.simulation.mobility.states["actor"].phase, Phase::Arrived);
}
