//! Explicit game-owned displacement through the existing physical character motor.
use super::*;
impl Simulation {
    /// Move a local ground character with the same collision shape/controller as locomotion.
    /// Session gameplay owns the intent; this does not edit the authored scene or reset time.
    pub fn displace_character(
        &mut self,
        id: &str,
        delta: [f32; 3],
    ) -> crate::model::Result<[f32; 3]> {
        if self.planet.is_some()
            || delta.iter().any(|v| !v.is_finite())
            || glam::Vec3::from(delta).length() > 100.
        {
            return Err(crate::model::invalid(
                "Local displacement requires finite delta <=100m and no transported planet",
            ));
        }
        let handle = *self
            .handles
            .get(id)
            .ok_or_else(|| crate::model::invalid("Unknown character"))?;
        let controller = self
            .ground_motors
            .get(id)
            .ok_or_else(|| crate::model::invalid("Displacement requires a character controller"))?;
        let body = &self.bodies[handle];
        let collider = &self.colliders[body.colliders()[0]];
        let enabled = |_: ColliderHandle, c: &Collider| c.is_enabled();
        let query = self.broad.as_query_pipeline(
            self.narrow.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            QueryFilter::default()
                .exclude_rigid_body(handle)
                .exclude_sensors()
                .predicate(&enabled),
        );
        let movement = controller.move_shape(
            crate::model::FIXED_DT,
            &query,
            collider.shape(),
            body.position(),
            delta.into(),
            |_| {},
        );
        let next = *body.translation() + movement.translation;
        let body = &mut self.bodies[handle];
        body.set_translation(next, true);
        body.set_next_kinematic_translation(next);
        self.bodies
            .propagate_modified_body_positions_to_colliders(&mut self.colliders);
        Ok(movement.translation.into())
    }
}
