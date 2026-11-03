//! Read-only camera sweep against the existing physical snapshot, never visual LOD.
use super::*;
use rapier3d::parry::{query::ShapeCastOptions, shape::Ball};

impl Simulation {
    pub(crate) fn camera_fraction(
        &self,
        owner: &str,
        pivot: [f32; 3],
        desired: [f32; 3],
        radius: f32,
    ) -> (f32, bool) {
        let Some(&body) = self.handles.get(owner) else {
            return (0., true);
        };
        let enabled = |_: ColliderHandle, c: &Collider| c.is_enabled();
        let query = self.broad.as_query_pipeline(
            self.narrow.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            QueryFilter::default()
                .exclude_rigid_body(body)
                .exclude_sensors()
                .predicate(&enabled),
        );
        let ball = Ball::new(radius);
        let pose = Isometry::translation(pivot[0], pivot[1], pivot[2]);
        if query.intersect_shape(pose, &ball).next().is_some() {
            return (0., true);
        }
        let velocity = vector![
            desired[0] - pivot[0],
            desired[1] - pivot[1],
            desired[2] - pivot[2]
        ];
        let options = ShapeCastOptions {
            max_time_of_impact: 1.,
            compute_impact_geometry_on_penetration: false,
            ..Default::default()
        };
        let fraction = query
            .cast_shape(&pose, &velocity, &ball, options)
            .map_or(1., |(_, hit)| hit.time_of_impact.clamp(0., 1.));
        (fraction, false)
    }
}
