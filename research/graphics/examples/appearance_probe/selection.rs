//! Screen-space geometric bound, not a bound on radiance or reconstruction error.
use super::{
    geometry::{TILE_COUNT, TileBounds},
    gpu::{Case, FOV_Y, NEAR},
};
use glam::{Mat4, Vec3};
use std::ops::Range;

pub const ENTER_SUMMARY: f32 = 0.75;
pub const RETURN_GEOMETRY: f32 = 1.0;

pub fn footprint(bound: TileBounds, case: Case, size: u32) -> f32 {
    projected_bound(
        bound,
        case.view_matrix(),
        size as f32 / (2.0 * (FOV_Y * 0.5).tan()),
    )
}

fn projected_bound(bound: TileBounds, view: Mat4, focal: f32) -> f32 {
    let mut depth = f32::INFINITY;
    let mut xy = [0.0_f32; 2];
    for x in [bound.min.x, bound.max.x] {
        for y in [bound.min.y, bound.max.y] {
            for z in [bound.min.z, bound.max.z] {
                let p = view.transform_point3(Vec3::new(x, y, z));
                depth = depth.min(-p.z);
                xy[0] = xy[0].max(p.x.abs());
                xy[1] = xy[1].max(p.y.abs());
            }
        }
    }
    if !depth.is_finite() || depth <= NEAR {
        return f32::INFINITY;
    }
    // Operator norm of the perspective projection Jacobian, bounded over the box.
    focal * bound.cell_diameter / depth
        * (1.0 + (xy[0] * xy[0] + xy[1] * xy[1]) / (depth * depth)).sqrt()
}

#[derive(Clone)]
pub struct Selector {
    pub detailed: [bool; TILE_COUNT],
}
impl Default for Selector {
    fn default() -> Self {
        Self {
            detailed: [true; TILE_COUNT],
        }
    }
}
impl Selector {
    pub fn update(
        &mut self,
        bounds: &[TileBounds; TILE_COUNT],
        case: Case,
        size: u32,
    ) -> [f32; TILE_COUNT] {
        let view = case.view_matrix();
        let focal = size as f32 / (2.0 * (FOV_Y * 0.5).tan());
        let pixels = bounds.map(|b| projected_bound(b, view, focal));
        for (detail, px) in self.detailed.iter_mut().zip(pixels) {
            if !px.is_finite() || px > RETURN_GEOMETRY {
                *detail = true;
            } else if px < ENTER_SUMMARY {
                *detail = false;
            }
        }
        pixels
    }
}

/// Coalesce adjacent regions without allocating or drawing unselected vertices.
pub fn ranges(mask: &[bool; TILE_COUNT], value: bool) -> impl Iterator<Item = Range<u32>> + '_ {
    let mut cursor = 0;
    std::iter::from_fn(move || {
        while cursor < TILE_COUNT && mask[cursor] != value {
            cursor += 1;
        }
        let start = cursor;
        while cursor < TILE_COUNT && mask[cursor] == value {
            cursor += 1;
        }
        (start < cursor).then_some(start as u32..cursor as u32)
    })
}

#[cfg(test)]
mod tests {
    use super::super::geometry;
    use super::*;
    fn case(distance: f32) -> Case {
        Case {
            distance,
            tilt: 35.0,
            light_angle: 25.0,
            exponent: 32.0,
            specular: 2.0,
            shift: 0.0,
        }
    }
    #[test]
    fn bound_covers_projected_cell_edges_and_scales_with_resolution() {
        let mesh = geometry::generate(32);
        for camera in [
            case(2.5),
            case(8.0),
            Case {
                tilt: 75.0,
                shift: 0.6,
                ..case(3.0)
            },
        ] {
            let view = camera.view_matrix();
            let focal = 256.0 / (2.0 * (FOV_Y * 0.5).tan());
            for tri in mesh.vertices.chunks_exact(3) {
                let b = mesh.bounds[tri[0].tile as usize];
                let limit = footprint(b, camera, 256);
                assert!((footprint(b, camera, 512) - 2.0 * limit).abs() < 1e-5);
                let projected = tri
                    .iter()
                    .map(|v| {
                        let p = view.transform_point3(Vec3::from_array(v.position));
                        glam::Vec2::new(p.x, p.y) * (focal / -p.z)
                    })
                    .collect::<Vec<_>>();
                for i in 0..3 {
                    assert!(projected[i].distance(projected[(i + 1) % 3]) <= limit + 1e-4);
                }
            }
        }
    }
    #[test]
    fn hysteresis_reversal_resize_and_near_plane() {
        let mesh = geometry::generate(128);
        let mut selector = Selector::default();
        selector.update(&mesh.bounds, case(2.5), 256);
        assert!(selector.detailed.iter().all(|v| *v));
        selector.update(&mesh.bounds, case(16.0), 256);
        assert!(selector.detailed.iter().all(|v| !*v));
        // Find a pose where a region is inside the dead band.
        let mid = (50..150)
            .map(|d| case(d as f32 / 10.0))
            .find(|c| {
                let px = footprint(mesh.bounds[0], *c, 256);
                px > ENTER_SUMMARY && px < RETURN_GEOMETRY
            })
            .unwrap();
        let mut from_near = Selector::default();
        from_near.update(&mesh.bounds, mid, 256);
        selector.update(&mesh.bounds, mid, 256);
        assert!(from_near.detailed[0]);
        assert!(!selector.detailed[0]);
        for _ in 0..20 {
            selector.update(&mesh.bounds, mid, 256);
            assert!(!selector.detailed[0]);
        }
        selector.update(&mesh.bounds, mid, 512);
        assert!(selector.detailed[0]);
        selector.update(&mesh.bounds, case(2.5), 256);
        assert!(selector.detailed.iter().all(|v| *v));
        assert!(footprint(mesh.bounds[36], case(0.05), 256).is_infinite());
    }
    #[test]
    fn contiguous_draws_cover_every_selected_tile_exactly_once() {
        for mask in [
            [true; TILE_COUNT],
            [false; TILE_COUNT],
            std::array::from_fn(|i| i % 3 == 0),
        ] {
            let mut hits = [0; TILE_COUNT];
            for value in [true, false] {
                for range in ranges(&mask, value) {
                    for i in range {
                        assert_eq!(mask[i as usize], value);
                        hits[i as usize] += 1;
                    }
                }
            }
            assert_eq!(hits, [1; TILE_COUNT]);
        }
        let mesh = geometry::generate(32);
        for (tile, vertices) in mesh
            .vertices
            .chunks_exact(mesh.vertices.len() / TILE_COUNT)
            .enumerate()
        {
            assert!(vertices.iter().all(|v| v.tile == tile as u32));
        }
        assert!(
            mesh.proxy
                .iter()
                .enumerate()
                .all(|(i, v)| v.tile == ((i / 6) as u32 | (1 << 31)))
        );
    }
}
