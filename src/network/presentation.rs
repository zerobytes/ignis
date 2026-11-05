//! The same received public state feeds native rendering and MCP captures.
use super::client::State;
use crate::{
    model::{Camera, Light, Result, invalid},
    render::{RenderBox, RenderFrame},
};
pub fn empty(camera: Camera, light: Light) -> RenderFrame {
    RenderFrame {
        clip: None,
        planet_air: None,
        suppress_diagnostic_grid: false,
        meshes: Default::default(),
        particles: Default::default(),
        distant_meshes: Default::default(),
        mesh_lods: Default::default(),
        mesh_lod_settings: Default::default(),
        camera,
        light,
        boxes: Vec::new(),
        player_primitives: Vec::new(),
        vision_guides: Vec::new(),
        hearing_guides: Vec::new(),
        noise_guides: Vec::new(),
        route_guides: Vec::new(),
        truncated_navigation_paths: 0,
        selection: None,
        selection_mesh: None,
    }
}
pub fn update(state: &State, frame: &mut RenderFrame, camera: Option<&Camera>) -> Result<()> {
    let content = state
        .content
        .as_ref()
        .ok_or_else(|| invalid("Remote visual content is not ready"))?;
    let tick = state
        .replica
        .tick()
        .ok_or_else(|| invalid("Remote baseline is not ready"))?;
    frame.camera = camera.unwrap_or(&content.camera).clone();
    frame.light = content.light.clone();
    frame.particles.update(&content.particles, tick);
    frame.distant_meshes = content.distant_meshes.clone();
    frame.meshes.update_filtered(
        &content.meshes,
        tick,
        content.consumed.iter().map(String::as_str),
    );
    frame
        .meshes
        .append_avatars(&content.meshes, state.replica.avatars(), |id| {
            state
                .replica
                .actors()
                .get(&id)
                .map(|a| (a.position(), a.appearance.color))
        })?;
    frame.boxes.clear();
    frame.player_primitives.clear();
    frame
        .boxes
        .reserve(state.replica.actors().len() + state.replica.projectiles().len());
    frame.boxes.extend(
        state
            .replica
            .actors()
            .values()
            .filter(|a| {
                !state.replica.avatars().iter().any(|p| {
                    p.actor == a.id && p.hide_body && frame.meshes.drawn_avatars.contains(&a.id)
                })
            })
            .enumerate()
            .map(|(box_index, a)| {
                if a.appearance.kind == super::replica::Kind::Player {
                    frame
                        .player_primitives
                        .push(crate::render::PlayerPrimitive {
                            actor: a.id,
                            life: a.life,
                            box_index,
                        });
                }
                RenderBox {
                    geometry: a.appearance.geometry,
                    position: a.position(),
                    size: a.appearance.size,
                    color: if a.health.is_some_and(|h| h.units == 0) {
                        [0.22, 0.22, 0.24]
                    } else {
                        a.appearance.color
                    },
                }
            }),
    );
    frame
        .boxes
        .extend(state.replica.projectiles().iter().map(|p| RenderBox {
            geometry: Default::default(),
            position: p.millimeters.map(|v| v as f32 / 1000.),
            size: [p.half_extent * 2.; 3],
            color: [1., 0.72, 0.12],
        }));
    frame.vision_guides.clear();
    frame.hearing_guides.clear();
    frame.noise_guides.clear();
    frame.route_guides.clear();
    frame.truncated_navigation_paths = 0;
    frame.selection = None;
    Ok(())
}

pub fn pick(
    state: &State,
    camera: &Camera,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
) -> Result<Option<super::protocol::Target>> {
    let (start, delta) = crate::picking::ray(camera, x, y, width, height)?;
    let mut best = None;
    for actor in state.replica.actors().values() {
        let volume = crate::geometry::Volume {
            geometry: actor.appearance.geometry,
            center: actor.position().into(),
            size: actor.appearance.size.into(),
        };
        let Some(t) = volume.cast(start, delta, glam::Vec3::ZERO, false, 0.) else {
            continue;
        };
        if best.is_none_or(|(old, old_t): (&super::replica::Actor, f32)| {
            t < old_t || (t == old_t && actor.id < old.id)
        }) {
            best = Some((actor, t));
        }
    }
    Ok(best.and_then(|(actor, _)| {
        actor
            .appearance
            .kind
            .actor()
            .then_some(super::protocol::Target {
                actor: actor.id,
                life: actor.life,
            })
    }))
}
