//! Bounded visualization of authored routes and known navigation results.
use crate::{
    engine::Engine,
    patrol::Traversal,
    render::{LineGuide, RenderFrame},
};

pub const MAX_PATH_SEGMENTS: usize = 256;
static UNIT_CIRCLE: std::sync::LazyLock<[[f32; 2]; 64]> = std::sync::LazyLock::new(|| {
    std::array::from_fn(|i| {
        let (s, c) = (i as f32 * std::f32::consts::TAU / 64.).sin_cos();
        [c, s]
    })
});
fn projected(p: [f32; 3], height: f32) -> [f32; 3] {
    [p[0], height, p[2]]
}
fn marker(out: &mut Vec<LineGuide>, p: [f32; 3], size: f32, color: [f32; 3]) {
    for delta in [[size, 0., size], [size, 0., -size]] {
        out.push(LineGuide {
            start: [p[0] - delta[0], p[1], p[2] - delta[2]],
            end: [p[0] + delta[0], p[1], p[2] + delta[2]],
            color,
            dashed: false,
        });
    }
}
pub fn fill(engine: &Engine, frame: &mut RenderFrame) {
    frame.route_guides.clear();
    frame.truncated_navigation_paths = 0;
    if !engine.debug_view.npc_routes {
        return;
    }
    let scene = engine.simulation.definitions();
    for link in &scene.spatial_network.links {
        let color = match link.mode {
            crate::mobility::Mode::Walk => [0.7, 0.7, 0.7],
            crate::mobility::Mode::Jump => [1., 0.6, 0.1],
            crate::mobility::Mode::Glide => [0.2, 1., 0.4],
            crate::mobility::Mode::Fly => [0.2, 0.7, 1.],
        };
        let a = scene.spatial_network.nodes[link.from as usize];
        let b = scene.spatial_network.nodes[link.to as usize];
        frame.route_guides.push(LineGuide {
            start: a,
            end: b,
            color,
            dashed: true,
        });
        marker(&mut frame.route_guides, b, 0.15, color);
    }
    for (i, passage) in scene.passages.iter().enumerate() {
        let floor = scene
            .entities
            .iter()
            .find(|e| e.id == passage.surface)
            .unwrap();
        let height = floor.transform.position[1] + floor.transform.size[1] * 0.5 + 0.08;
        let state = &engine.simulation.passages.slots[i];
        let color = if state.occupied {
            [1., 0.2, 0.1]
        } else if state.owner.is_some() {
            [0.2, 1., 0.4]
        } else {
            [0.6, 0.6, 0.6]
        };
        let d = passage.direction();
        let perpendicular = glam::Vec2::new(-d.y, d.x) * passage.width * 0.5;
        let a = glam::Vec2::from_array(passage.a);
        let b = glam::Vec2::from_array(passage.b);
        let corners = [
            a - perpendicular,
            b - perpendicular,
            b + perpendicular,
            a + perpendicular,
        ];
        for j in 0..4 {
            let p = corners[j];
            let q = corners[(j + 1) % 4];
            frame.route_guides.push(LineGuide {
                start: [p.x, height, p.y],
                end: [q.x, height, q.y],
                color,
                dashed: true,
            });
        }
        for p in passage.wait_a.iter().chain(passage.wait_b.iter()) {
            marker(
                &mut frame.route_guides,
                [p[0], height, p[1]],
                0.18,
                [1., 0.8, 0.15],
            );
        }
    }
    for entity in &scene.entities {
        let Some(npc) = &entity.npc else { continue };
        if !npc.enabled || engine.simulation.position(&entity.id).is_none() {
            continue;
        }
        let floor = scene.entities.iter().find(|e| e.id == npc.surface).unwrap();
        let height = floor.transform.position[1] + floor.transform.size[1] * 0.5 + 0.07;
        let out = &mut frame.route_guides;
        if let Some(territory) = npc.territory {
            let color = if engine.simulation.npcs[&entity.id].territory.blocked() {
                [1., 0.25, 0.2]
            } else {
                [1., 0.65, 0.1]
            };
            let point = |i: usize| {
                let p = UNIT_CIRCLE[i % 64];
                [
                    territory.home[0] + p[0] * territory.radius,
                    height,
                    territory.home[1] + p[1] * territory.radius,
                ]
            };
            for i in 0..64 {
                out.push(LineGuide {
                    start: point(i),
                    end: point(i + 1),
                    color,
                    dashed: true,
                });
            }
            marker(
                out,
                [territory.home[0], height, territory.home[1]],
                0.35,
                color,
            );
        }

        if let Some(route) = engine.simulation.patrol_routes.get(&entity.id) {
            let color = [0.55, 0.5, 0.85];
            let project = |p| {
                if npc.patrol.as_ref().unwrap().route.is_spatial() {
                    p
                } else {
                    projected(p, height)
                }
            };
            for point in route {
                marker(out, project(*point), 0.12, color);
            }
            for pair in route.windows(2) {
                out.push(LineGuide {
                    start: project(pair[0]),
                    end: project(pair[1]),
                    color,
                    dashed: true,
                });
            }
            if route.len() > 1 && npc.patrol.as_ref().unwrap().traversal == Traversal::Loop {
                out.push(LineGuide {
                    start: project(*route.last().unwrap()),
                    end: project(route[0]),
                    color,
                    dashed: true,
                });
            }
        }
        if let Some(state) = engine.simulation.mobility.states.get(&entity.id)
            && let Some(edge) = state.edge
            && scene.spatial_network.links[edge as usize].mode == crate::mobility::Mode::Jump
            && let (Some(origin), Some(m)) = (state.launch_position, state.motion)
        {
            let origin = glam::Vec3::from(origin) + glam::Vec3::Y * entity.transform.size[1] * 0.5;
            let v = glam::Vec3::from(m.velocity);
            let mut previous = origin;
            for i in 1..=32 {
                let t = m.seconds * i as f32 / 32.;
                let next = origin + v * t - glam::Vec3::Y * (0.5 * 9.81 * t * t);
                out.push(LineGuide {
                    start: previous.into(),
                    end: next.into(),
                    color: [1., 0.6, 0.1],
                    dashed: false,
                });
                previous = next;
            }
        }
        if let Some(state) = engine.simulation.mobility.states.get(&entity.id) {
            let height = glam::Vec3::Y * entity.transform.size[1] * 0.5;
            for pair in state.flight_path.windows(2) {
                out.push(LineGuide {
                    start: (glam::Vec3::from(pair[0]) + height).into(),
                    end: (glam::Vec3::from(pair[1]) + height).into(),
                    color: [0.8, 0.4, 1.],
                    dashed: false,
                });
            }
            if let Some(flight) = state.flight {
                let origin = glam::Vec3::from(engine.simulation.position(&entity.id).unwrap());
                out.push(LineGuide {
                    start: origin.into(),
                    end: (origin + glam::Vec3::from(flight.heading) * 1.5).into(),
                    color: [1., 0.9, 0.3],
                    dashed: false,
                });
            }
        }
        if let Some(state) = engine.simulation.mobility.states.get(&entity.id)
            && let Some(node) = state.partial_target
        {
            let mut p = scene.spatial_network.nodes[node as usize];
            p[1] += entity.transform.size[1] * 0.5;
            marker(out, p, 0.3, [1., 0.65, 0.1]);
        }
        if let Some(state) = engine.simulation.mobility.states.get(&entity.id)
            && let Some(connector) = state.connector
        {
            let mut p = connector.endpoint;
            p[1] += entity.transform.size[1] * 0.5;
            marker(out, p, 0.2, [0.2, 1., 1.]);
        }
        let nav = &engine.simulation.navigation[&entity.id];
        if nav.path.len() > MAX_PATH_SEGMENTS {
            frame.truncated_navigation_paths += 1;
        }
        let project = |p| {
            if npc.mobility.is_some() {
                p
            } else {
                projected(p, height)
            }
        };
        let mut start = project(engine.simulation.position(&entity.id).unwrap());
        for point in nav.path.iter().take(MAX_PATH_SEGMENTS) {
            let end = project(*point);
            out.push(LineGuide {
                start,
                end,
                color: [1., 1., 1.],
                dashed: false,
            });
            start = end;
        }
        if let Some(goal) = nav.target_position {
            let color = if matches!(nav.status.as_str(), "unreachable" | "retry_wait") {
                [1., 0.1, 0.1]
            } else {
                [1., 1., 1.]
            };
            marker(out, project(goal), 0.25, color);
        }
    }
}
