//! Stable initial cuboid contact for the readonly KCC queries; other queries delegate.
use rapier3d::{
    math::{Isometry, Real, Vector},
    parry::{
        query::{
            ClosestPoints, Contact, NonlinearRigidMotion, QueryDispatcher, ShapeCastHit,
            ShapeCastOptions, ShapeCastStatus, Unsupported, details,
        },
        shape::Shape,
    },
};
mod triangle;
pub(crate) struct StableContacts<'a>(pub &'a dyn QueryDispatcher, pub bool);
impl QueryDispatcher for StableContacts<'_> {
    fn intersection_test(
        &self,
        p: &Isometry<Real>,
        a: &dyn Shape,
        b: &dyn Shape,
    ) -> Result<bool, Unsupported> {
        self.0.intersection_test(p, a, b)
    }
    fn distance(
        &self,
        p: &Isometry<Real>,
        a: &dyn Shape,
        b: &dyn Shape,
    ) -> Result<Real, Unsupported> {
        self.0.distance(p, a, b)
    }
    fn contact(
        &self,
        p: &Isometry<Real>,
        a: &dyn Shape,
        b: &dyn Shape,
        prediction: Real,
    ) -> Result<Option<Contact>, Unsupported> {
        self.0.contact(p, a, b, prediction)
    }
    fn closest_points(
        &self,
        p: &Isometry<Real>,
        a: &dyn Shape,
        b: &dyn Shape,
        max_dist: Real,
    ) -> Result<ClosestPoints, Unsupported> {
        self.0.closest_points(p, a, b, max_dist)
    }
    fn cast_shapes(
        &self,
        p: &Isometry<Real>,
        v: &Vector<Real>,
        a: &dyn Shape,
        b: &dyn Shape,
        options: ShapeCastOptions,
    ) -> Result<Option<ShapeCastHit>, Unsupported> {
        if self.1 {
            if let (Some(mesh), Some(_)) = (a.as_trimesh(), b.as_cuboid()) {
                return Ok(details::cast_shapes_composite_shape_shape(
                    self, p, v, mesh, b, options,
                ));
            }
            if let (Some(_), Some(mesh)) = (a.as_cuboid(), b.as_trimesh()) {
                return Ok(details::cast_shapes_shape_composite_shape(
                    self, p, v, a, mesh, options,
                ));
            }
            if let (Some(t), Some(c)) = (a.as_triangle(), b.as_cuboid()) {
                return Ok(triangle::cast(p, v, t, c, options, true));
            }
            if let (Some(c), Some(t)) = (a.as_cuboid(), b.as_triangle()) {
                return Ok(triangle::cast(
                    &p.inverse(),
                    &-p.inverse_transform_vector(v),
                    t,
                    c,
                    options,
                    true,
                )
                .map(|h| h.swapped()));
            }
        }
        if let (Some(a), Some(b)) = (a.as_cuboid(), b.as_cuboid())
            && let Some(contact) = details::contact_cuboid_cuboid(p, a, b, options.target_distance)
        {
            if !options.stop_at_penetration && contact.normal1.dot(v) >= 0. {
                return Ok(None);
            }
            return Ok(Some(ShapeCastHit {
                time_of_impact: 0.,
                normal1: contact.normal1,
                normal2: contact.normal2,
                witness1: contact.point1,
                witness2: contact.point2,
                status: ShapeCastStatus::PenetratingOrWithinTargetDist,
            }));
        }
        self.0.cast_shapes(p, v, a, b, options)
    }
    fn cast_shapes_nonlinear(
        &self,
        m1: &NonlinearRigidMotion,
        a: &dyn Shape,
        m2: &NonlinearRigidMotion,
        b: &dyn Shape,
        start: Real,
        end: Real,
        stop: bool,
    ) -> Result<Option<ShapeCastHit>, Unsupported> {
        self.0.cast_shapes_nonlinear(m1, a, m2, b, start, end, stop)
    }
}
