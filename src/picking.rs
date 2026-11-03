//! Editor picking of the currently rendered axis-aligned boxes.
use crate::{
    model::{Camera, Result, Scene, invalid},
    physics::Simulation,
};
use glam::{Mat4, Vec3};
use serde::Serialize;

pub(crate) const NEAR: f32 = 0.05;
pub(crate) const FAR: f32 = 30_000.;
fn projection(camera: &Camera, width: u32, height: u32) -> Mat4 {
    Mat4::perspective_rh(
        camera.fov_degrees.to_radians(),
        width as f32 / height as f32,
        NEAR,
        FAR,
    )
}
fn view(camera: &Camera) -> Mat4 {
    Mat4::look_at_rh(camera.position.into(), camera.target.into(), Vec3::Y)
}
pub fn view_projection(camera: &Camera, width: u32, height: u32) -> Mat4 {
    projection(camera, width, height) * view(camera)
}
pub(crate) fn view_projection_clip(
    camera: &Camera,
    width: u32,
    height: u32,
    clip: Option<crate::render::Clip>,
) -> Mat4 {
    clip.map_or_else(
        || view_projection(camera, width, height),
        |c| {
            Mat4::perspective_rh(
                camera.fov_degrees.to_radians(),
                width as f32 / height as f32,
                if c.reverse_z { c.far } else { c.near },
                if c.reverse_z { c.near } else { c.far },
            ) * view(camera)
        },
    )
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Hit {
    #[serde(skip)]
    pub entity_index: u16,
    pub position: [f32; 3],
    /// Distance from the authored camera, in world metres.
    pub distance: f32,
}
fn intersect(start: Vec3, delta: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for axis in 0..3 {
        let min = center[axis] - half[axis];
        let max = center[axis] + half[axis];
        if delta[axis] == 0. {
            if start[axis] < min || start[axis] > max {
                return None;
            }
        } else {
            let a = (min - start[axis]) / delta[axis];
            let b = (max - start[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
            if enter > exit {
                return None;
            }
        }
    }
    Some(enter)
}
pub(crate) fn ray(
    camera: &Camera,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
) -> Result<(Vec3, Vec3)> {
    if !(1..=16384).contains(&width)
        || !(1..=16384).contains(&height)
        || !x.is_finite()
        || !y.is_finite()
        || x < 0.
        || y < 0.
        || x >= width as f32
        || y >= height as f32
    {
        return Err(invalid(
            "Picking requires viewport [1,16384] and finite coordinates inside it",
        ));
    }
    // Unproject in camera space to avoid subtracting large world translations at the far plane.
    let near = projection(camera, width, height)
        .inverse()
        .project_point3(Vec3::new(
            2. * x / width as f32 - 1.,
            1. - 2. * y / height as f32,
            0.,
        ));
    let camera_to_world = view(camera).inverse();
    let start = camera_to_world.transform_point3(near);
    let end = camera_to_world.transform_point3(near * (FAR / NEAR));
    if !start.is_finite() || !end.is_finite() {
        return Err(invalid("Camera cannot produce a finite picking segment"));
    }
    Ok((start, end - start))
}
/// Resolve a cursor ray on an explicit horizontal plane, without selecting an entity.
pub fn point_on_plane(
    camera: &Camera,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    plane_y: f32,
) -> Result<[f32; 3]> {
    if !plane_y.is_finite() || plane_y.abs() > 10_000. {
        return Err(invalid("Aim plane must be finite within +/-10000m"));
    }
    let (start, delta) = ray(camera, x, y, width, height)?;
    if delta.y.abs() < 0.000001 {
        return Err(invalid("Cursor ray is parallel to aim plane"));
    }
    let t = (plane_y - start.y) / delta.y;
    if !(0.0..=1.0).contains(&t) {
        return Err(invalid("Aim plane is outside the forward camera segment"));
    }
    let position = (start + delta * t).to_array();
    crate::abilities::Aim::Point { position }.validate()?;
    Ok(position)
}
pub fn pick(
    scene: &Scene,
    simulation: &Simulation,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
) -> Result<Option<Hit>> {
    pick_camera(simulation, &scene.camera, x, y, width, height)
}
pub fn pick_camera(
    simulation: &Simulation,
    camera: &Camera,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
) -> Result<Option<Hit>> {
    let (start, delta) = ray(camera, x, y, width, height)?;
    let mut best: Option<(usize, f32)> = None;
    for (index, entity) in simulation.definitions().entities.iter().enumerate() {
        if entity.visible == Some(false) {
            continue;
        }
        let Some(position) = simulation.position(&entity.id) else {
            continue;
        };
        let contact = if entity.geometry.is_box() {
            intersect(
                start,
                delta,
                position.into(),
                Vec3::from(entity.transform.size) * 0.5,
            )
        } else {
            crate::geometry::Volume::entity(entity, position.into()).cast(
                start,
                delta,
                Vec3::ZERO,
                false,
                0.,
            )
        };
        let Some(t) = contact else {
            continue;
        };
        if best.is_none_or(|(old, old_t)| {
            t < old_t || (t == old_t && entity.id < simulation.definitions().entities[old].id)
        }) {
            best = Some((index, t));
        }
    }
    Ok(best.map(|(index, t)| {
        let position = start + delta * t;
        Hit {
            entity_index: index as u16,
            position: position.into(),
            distance: position.distance(camera.position.into()),
        }
    }))
}
