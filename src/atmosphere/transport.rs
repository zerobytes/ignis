//! Conservative homogeneous-medium contribution; vacuum has no extinction.
//! Sphere is the convex volume used by the renderer, ending at opaque terrain.
use glam::DVec3;
#[derive(Clone, Copy)]
pub(crate) struct Envelope {
    pub corners: [DVec3; 8],
    pub radius_upper: f64,
}
#[derive(Clone, Copy)]
pub(crate) enum Medium {
    #[cfg(test)]
    Infinite,
    Sphere {
        radius: f64,
    },
}
impl Medium {
    pub fn minimum_path(self, eye: DVec3, envelope: Envelope, shift: f64, guard: f64) -> f64 {
        let bounds = envelope.corners;
        let lo = bounds
            .iter()
            .fold(DVec3::splat(f64::INFINITY), |a, b| a.min(*b));
        let hi = bounds
            .iter()
            .fold(DVec3::splat(f64::NEG_INFINITY), |a, b| a.max(*b));
        let nearest = ((eye - eye.clamp(lo, hi)).length() - shift - guard).max(0.);
        match self {
            #[cfg(test)]
            Self::Infinite => nearest,
            Self::Sphere { radius } => {
                let endpoint_max = envelope.radius_upper + shift + guard;
                if endpoint_max >= radius {
                    return 0.;
                }
                if eye.length() + guard < radius {
                    nearest
                } else if eye.length() - guard > radius {
                    (radius - endpoint_max).max(0.)
                } else {
                    0.
                }
            }
        }
    }
    pub fn transmission_upper(
        self,
        density: f64,
        eye: DVec3,
        envelope: Envelope,
        shift: f64,
        guard: f64,
    ) -> f64 {
        (-density.max(0.) * self.minimum_path(eye, envelope, shift, guard)).exp()
    }
}
// The same hard endpoint displacement limit is enforced in WGSL. This is a
// quality policy, not an inferred bound on floating point root error.
pub(crate) fn correction_limit(radius: f64, altitude: f64) -> f64 {
    (altitude.abs() * 0.1).max(32.).min(radius * 0.125)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn b(p: DVec3) -> Envelope {
        Envelope {
            corners: [p; 8],
            radius_upper: p.length(),
        }
    }
    #[test]
    fn finite_volume_never_counts_vacuum() {
        let m = Medium::Sphere { radius: 10. };
        assert_eq!(m.minimum_path(DVec3::Z * 1e6, b(DVec3::Z * 9.), 0., 0.), 1.);
        assert_eq!(m.minimum_path(DVec3::Z * 8., b(DVec3::Z * 9.), 0., 0.), 1.);
        assert_eq!(m.minimum_path(DVec3::Z * 10., b(DVec3::Z * 9.), 0., 0.), 0.);
        assert_eq!(m.minimum_path(DVec3::ZERO, b(DVec3::Z * 11.), 0., 0.), 0.);
    }
    #[test]
    fn shift_and_clear_air_are_conservative() {
        assert_eq!(
            Medium::Infinite.minimum_path(DVec3::ZERO, b(DVec3::X * 100.), 5., 2.),
            93.
        );
        assert_eq!(
            Medium::Sphere { radius: 1000. }.transmission_upper(
                0.,
                DVec3::ZERO,
                b(DVec3::X * 100.),
                5.,
                2.
            ),
            1.
        );
    }
}
