//! Source9: the sclera remains an actual sphere. The lid wraps around it.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SphericalFit {
    /// Drawn socket-wall vertices, excluding the shared rim. They stay behind
    /// the globe's front surface; external skin stays in front of it.
    pub interior: Vec<u32>,
}
pub(super) struct Wrapped {
    pub center: Vec3,
    pub radius: f32,
    pub positions: Vec<(u32, Vec3)>,
}
impl SphericalFit {
    pub(super) fn wrap(
        &self,
        rim_ids: &[u32],
        rim: &[Vec3],
        samples: &[(u32, f32, Vec3)],
        base: (Vec3, f32),
        amount: f32,
        depth: f32,
    ) -> Result<Wrapped> {
        let (base, base_radius) = base;
        let inside: BTreeSet<_> = self.interior.iter().copied().collect();
        let rim_set: BTreeSet<_> = rim_ids.iter().copied().collect();
        let low = rim
            .iter()
            .fold(Vec3::splat(f32::INFINITY), |a, &b| a.min(b));
        let high = rim
            .iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), |a, &b| a.max(b));
        let mut fitted = (low + high) * 0.5;
        let fitted_radius = rim
            .iter()
            .map(|p| (p.truncate() - fitted.truncate()).length() * 1.06)
            .fold(base_radius, f32::max);
        if !fitted_radius.is_finite() || fitted_radius > 0.04 {
            return Err(invalid("Spherical aperture exceeds the40mm fitting domain"));
        }
        // Recess the centre enough to keep the spherical dome behind the
        // external facial surface. A mean fit could push it through the skin.
        fitted.z = f32::NEG_INFINITY;
        for &(id, _, p) in samples {
            let rr = (p.truncate() - fitted.truncate()).length_squared();
            if !inside.contains(&id) && rr < fitted_radius * fitted_radius {
                fitted.z = fitted
                    .z
                    .max(p.z + (fitted_radius * fitted_radius - rr).sqrt());
            }
        }
        if !fitted.z.is_finite() {
            return Err(invalid("Spherical contact has no external support"));
        }
        let center = base.lerp(fitted, amount);
        let radius = base_radius + (fitted_radius - base_radius) * amount;
        let front = |p: Vec3| {
            let rr = (p.truncate() - center.truncate()).length_squared();
            (rr < radius * radius).then(|| center.z - (radius * radius - rr).sqrt())
        };
        let corrections: Vec<_> = rim
            .iter()
            .map(|&p| front(p).map_or(0., |z| z - 0.00005 - p.z))
            .collect();
        let mut positions = Vec::with_capacity(samples.len());
        for &(id, weight, p) in samples {
            let mut numerator = 0.;
            let mut denominator = 0.;
            for (&r, &delta) in rim.iter().zip(&corrections) {
                let d = (p.truncate() - r.truncate()).length_squared() + 0.000001;
                let w = 1. / (d * d);
                numerator += w * delta;
                denominator += w;
            }
            let radial = (p.truncate() - center.truncate()).length();
            let t = ((radial - radius) / 0.006).clamp(0., 1.);
            let support = 1. - t * t * (3. - 2. * t);
            let delta = numerator / denominator;
            let mut q = p + Vec3::Z * (delta * weight * support * amount + depth * weight);
            if let Some(z) = front(p) {
                let z = z + depth;
                let target = if inside.contains(&id) {
                    q.z.max(z + 0.0002)
                } else if rim_set.contains(&id) {
                    z - 0.00005
                } else {
                    q.z.min(z - 0.00005)
                };
                q.z += (target - q.z) * amount;
            }
            positions.push((id, q));
        }
        Ok(Wrapped {
            center: center + Vec3::Z * depth,
            radius,
            positions,
        })
    }
}
