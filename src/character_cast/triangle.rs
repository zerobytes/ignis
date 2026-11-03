//! Bounded f64 SAT for the planetary cuboid motor. Large triangle/character
//! ratios made generic f32 support-map contacts return lateral floor normals.
//! Only the character query dispatcher opts in; other physics uses its backend.
use rapier3d::{
    math::{Isometry, Point, Real, Vector},
    na::{Unit, Vector3},
    parry::{
        query::{PointQuery, ShapeCastHit, ShapeCastOptions, ShapeCastStatus},
        shape::{Cuboid, Triangle},
    },
};

pub(super) fn cast(
    p: &Isometry<Real>,
    v: &Vector<Real>,
    triangle: &Triangle,
    cuboid: &Cuboid,
    options: ShapeCastOptions,
    continuous_heightfield: bool,
) -> Option<ShapeCastHit> {
    let inverse = p.cast::<f64>().inverse();
    let vertices = [triangle.a, triangle.b, triangle.c].map(|p| (inverse * p.cast::<f64>()).coords);
    let edges = [
        vertices[1] - vertices[0],
        vertices[2] - vertices[1],
        vertices[0] - vertices[2],
    ];
    let raw = edges[0].cross(&edges[1]);
    if raw.norm_squared() == 0. {
        return None;
    }
    let face = raw.normalize();
    let half = cuboid.half_extents.cast::<f64>();
    let velocity = inverse.rotation * v.cast::<f64>();
    let basis = [Vector3::x(), Vector3::y(), Vector3::z()];
    let mut axes = [Vector3::zeros(); 13];
    axes[..3].copy_from_slice(&basis);
    axes[3] = face;
    for (e, edge) in edges.iter().enumerate() {
        for (a, axis) in basis.iter().enumerate() {
            axes[4 + e * 3 + a] = edge.cross(axis);
        }
    }
    let mut enter = f64::NEG_INFINITY;
    let mut exit = f64::INFINITY;
    let mut normal = face;
    let mut inside = true;
    let mut actual_inside = true;
    let mut closest = f64::INFINITY;
    let mut escape = face;
    for axis in axes {
        let length = axis.norm();
        if length <= 1e-12 {
            continue;
        }
        let axis = axis / length;
        let min = vertices
            .iter()
            .map(|p| p.dot(&axis))
            .fold(f64::INFINITY, f64::min);
        let max = vertices
            .iter()
            .map(|p| p.dot(&axis))
            .fold(f64::NEG_INFINITY, f64::max);
        let radius = half.dot(&axis.abs());
        actual_inside &= min <= radius && max >= -radius;
        let radius = radius + f64::from(options.target_distance);
        inside &= min <= radius && max >= -radius;
        let lower = min - radius;
        let upper = max + radius;
        let depth = (-lower).min(upper);
        if depth < closest {
            closest = depth;
            escape = if -lower < upper { -axis } else { axis };
        }
        let speed = velocity.dot(&axis);
        if speed == 0. {
            if lower > 0. || upper < 0. {
                return None;
            }
            continue;
        }
        let a = lower / speed;
        let b = upper / speed;
        let start = a.min(b);
        let end = a.max(b);
        if start > enter {
            enter = start;
            normal = if speed > 0. { -axis } else { axis };
        }
        exit = exit.min(end);
        if enter > exit {
            return None;
        }
    }
    if exit < 0. || enter > f64::from(options.max_time_of_impact) {
        return None;
    }
    let time = enter.max(0.);
    let signed = face.dot(&(velocity * time - vertices[0]));
    let plane_normal = if signed >= 0. { face } else { -face };
    let plane_clearance = signed.abs() - half.dot(&face.abs());
    // A cuboid entirely on one side of this plane cannot be blocked tangentially
    // by an internal triangle edge. Existing penetration also recovers against
    // the actual face; an external solid edge retains its separating axis.
    // Planet residency guarantees a continuous heightfield beyond the motor's
    // entire sweep. Its triangle edges (including tile seams) are not solid
    // walls. SAT still clips against the finite triangle, but the contact uses
    // its surface normal. Detached finite meshes must retain their edge normal.
    if continuous_heightfield || plane_clearance >= 0. || (inside && actual_inside) {
        normal = plane_normal;
    } else if inside {
        normal = escape;
    }
    if inside && !options.stop_at_penetration && normal.dot(&velocity) >= 0. {
        return None;
    }
    let normal2 = Unit::new_normalize((-normal).cast::<Real>());
    let normal1 = Unit::new_normalize(p.rotation * (-*normal2));
    let support = Point::from(
        Vector3::new(
            if normal.x >= 0. { -half.x } else { half.x },
            if normal.y >= 0. { -half.y } else { half.y },
            if normal.z >= 0. { -half.z } else { half.z },
        )
        .cast::<Real>(),
    );
    let witness = p * support + v * (time as Real);
    let witness1 = triangle.project_local_point(&witness, false).point;
    Some(ShapeCastHit {
        time_of_impact: time as Real,
        normal1,
        normal2,
        witness1,
        witness2: support,
        status: if inside {
            ShapeCastStatus::PenetratingOrWithinTargetDist
        } else {
            ShapeCastStatus::Converged
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn large_floor_contacts_have_face_normals_and_finite_edges_still_block() {
        let floor = Triangle::new(
            Point::new(-6000., 0., -6000.),
            Point::new(0., 0., 6000.),
            Point::new(6000., 0., -6000.),
        );
        let body = Cuboid::new(Vector::new(0.3, 0.9, 0.3));
        let options = ShapeCastOptions {
            target_distance: 0.01,
            max_time_of_impact: 1.,
            stop_at_penetration: false,
            ..Default::default()
        };
        for y in [0.91, 0.9101, 0.8] {
            let hit = cast(
                &Isometry::translation(0., y, 0.),
                &Vector::new(0., -0.0027, -16.667),
                &floor,
                &body,
                options,
                false,
            )
            .unwrap();
            assert!(hit.normal1.y > 0.9999 && hit.time_of_impact <= 1.);
        }
        assert!(
            cast(
                &Isometry::translation(0., 0.91, 0.),
                &Vector::new(0., 1., 0.),
                &floor,
                &body,
                options,
                false
            )
            .is_none()
        );
        let wall = Triangle::new(
            Point::new(-6., -6., 0.),
            Point::new(6., -6., 0.),
            Point::new(0., 6., 0.),
        );
        let hit = cast(
            &Isometry::translation(0., 0., 10.),
            &Vector::new(0., 0., -1000.),
            &wall,
            &body,
            options,
            false,
        )
        .unwrap();
        assert!((hit.time_of_impact - 0.00969).abs() < 1e-6 && hit.normal1.z > 0.99);
        assert!(
            cast(
                &Isometry::translation(20., 0., 10.),
                &Vector::new(0., 0., -1000.),
                &wall,
                &body,
                options,
                false
            )
            .is_none()
        );
    }
}
