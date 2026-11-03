use super::*;
use glam::DMat3;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub vertices: [[f64; 3]; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct SurfacePoint {
    pub position: DVec3,
    pub axes: [DVec3; 3],
    pub area: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct Mass {
    pub mass: f64,
    pub center: DVec3,
    pub inertia: DMat3,
}
fn outer(v: DVec3) -> DMat3 {
    DMat3::from_cols(v * v.x, v * v.y, v * v.z)
}
pub fn compile_surfaces(
    body: &Body,
    scale: f64,
) -> Result<(Mass, Vec<SurfacePoint>, Vec<[f32; 3]>)> {
    let mut area = 0.;
    let mut first = DVec3::ZERO;
    let mut second = DMat3::ZERO;
    let mut thickness_inertia = DMat3::ZERO;
    let mut samples = Vec::with_capacity(body.panels.len() * 3);
    let mut hull = Vec::with_capacity(body.panels.len() * 6);
    let thickness = body.thickness_m * scale;
    // Accumulate about a nearby reference, not the global asset origin. The
    // parallel-axis subtraction must not cancel huge raw coordinate moments.
    let reference = body
        .panels
        .first()
        .map_or(DVec3::ZERO, |p| DVec3::from_array(p.vertices[0]) * scale);
    for panel in &body.panels {
        if panel.vertices.iter().any(|p| !vec(*p, 10000.)) {
            return Err(invalid("Aerodynamic panel coordinates"));
        }
        let [a, b, c] = panel.vertices.map(|p| DVec3::from_array(p) * scale);
        let cross = (b - a).cross(c - a);
        let ar = cross.length() * 0.5;
        if !bounded(ar, 1e-12, 1e8) {
            return Err(invalid("Degenerate or oversized aerodynamic panel"));
        }
        let n = cross.normalize();
        let t = (b - a).normalize();
        let axes = [n, t, n.cross(t)];
        let [ma, mb, mc] = [a - reference, b - reference, c - reference];
        let sum = ma + mb + mc;
        area += ar;
        first += sum * (ar / 3.);
        second += (outer(sum) + outer(ma) + outer(mb) + outer(mc)) * (ar / 12.);
        thickness_inertia += (DMat3::IDENTITY - outer(n)) * (ar * thickness * thickness / 12.);
        for (x, y, z) in [
            (2. / 3., 1. / 6., 1. / 6.),
            (1. / 6., 2. / 3., 1. / 6.),
            (1. / 6., 1. / 6., 2. / 3.),
        ] {
            samples.push(SurfacePoint {
                position: a * x + b * y + c * z,
                axes,
                area: ar / 3.,
            });
        }
        for v in [a, b, c] {
            for sign in [-1., 1.] {
                hull.push((v + n * (sign * thickness * 0.5)).as_vec3().to_array());
            }
        }
    }
    if !bounded(area, 1e-12, 1e8) {
        return Err(invalid("Aerodynamic surface requires positive area"));
    }
    let density = body.sheet_mass_kg / area;
    let ballast = DVec3::from_array(body.ballast_position) * scale - reference;
    let mass = body.sheet_mass_kg + body.ballast_mass_kg;
    let relative_center = (first * density + ballast * body.ballast_mass_kg) / mass;
    let center = relative_center + reference;
    let covariance =
        second * density + outer(ballast) * body.ballast_mass_kg - outer(relative_center) * mass;
    let trace = covariance.x_axis.x + covariance.y_axis.y + covariance.z_axis.z;
    let inertia = DMat3::IDENTITY * trace - covariance + thickness_inertia * density;
    let eig = rapier3d::na::Matrix3::from_column_slice(&inertia.to_cols_array()).symmetric_eigen();
    if !center.is_finite() || eig.eigenvalues.iter().any(|v| !v.is_finite() || *v < 1e-15) {
        return Err(invalid(
            "Aerodynamic inertia must be positive definite and representable",
        ));
    }
    if eig.eigenvalues.min() < eig.eigenvalues.max() * 1e-6 {
        return Err(invalid(
            "Aerodynamic inertia condition number exceeds the f32 backend domain",
        ));
    }
    let points: Vec<_> = hull
        .iter()
        .copied()
        .map(rapier3d::na::Point3::from)
        .collect();
    if rapier3d::prelude::SharedShape::convex_hull(&points).is_none() {
        return Err(invalid(
            "Aerodynamic collision thickness is not representable in the physics backend",
        ));
    }
    Ok((
        Mass {
            mass,
            center,
            inertia,
        },
        samples,
        hull,
    ))
}

/// Exact scalar backward-Euler drag solve, including rotational effective mass.
/// Updates velocities and returns the impulse magnitude. Never clamps force.
#[allow(clippy::too_many_arguments)] // Explicit inputs of the rank-one mechanical operator.
pub fn implicit_drag(
    velocity: &mut DVec3,
    omega: &mut DVec3,
    inverse_mass: f64,
    inverse_inertia: DMat3,
    offset: DVec3,
    direction: DVec3,
    air: DVec3,
    coefficient_dt: f64,
) -> f64 {
    let lever = offset.cross(direction);
    let inverse_effective_mass = inverse_mass + lever.dot(inverse_inertia * lever);
    let relative = (air - *velocity - omega.cross(offset)).dot(direction);
    let after = 2. * relative
        / (1. + (1. + 4. * coefficient_dt * inverse_effective_mass * relative.abs()).sqrt());
    let impulse = coefficient_dt * after.abs() * after;
    *velocity += direction * (impulse * inverse_mass);
    *omega += inverse_inertia * lever * impulse;
    impulse
}
