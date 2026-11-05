//! Cold seeded geometric wall relief; no detached decorative meshes.
use crate::{
    meshes::Geometry,
    model::{Result, invalid},
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub amplitude: f32,
    pub stratum_height: f32,
    pub joint_width: f32,
    pub seed: u64,
}
pub fn apply(g: &Geometry, reference: &Geometry, c: &Config, origin: [f32; 3]) -> Result<Geometry> {
    if !c.amplitude.is_finite()
        || !(0. ..=2.).contains(&c.amplitude)
        || !c.stratum_height.is_finite()
        || c.stratum_height < 0.1
        || !c.joint_width.is_finite()
        || c.joint_width < 1.
    {
        return Err(invalid(
            "Wall rock requires amplitude0..2m,stratum height>=0.1m,joint width>=1m",
        ));
    }
    g.validate()?;
    reference.validate()?;
    if g.positions.len() != reference.positions.len() || g.triangles != reference.triangles {
        return Err(invalid("Wall reference connectivity differs"));
    }
    let mut normal = vec![Vec3::ZERO; g.positions.len()];
    let mut room = vec![f32::INFINITY; g.positions.len()];
    for t in &g.triangles {
        let p = t.map(|v| Vec3::from_array(g.positions[v as usize]));
        for k in 0..3 {
            let length = (p[k] - p[(k + 1) % 3]).length() * 0.2;
            room[t[k] as usize] = room[t[k] as usize].min(length);
            room[t[(k + 1) % 3] as usize] = room[t[(k + 1) % 3] as usize].min(length);
        }
    }
    let mut top = vec![false; g.positions.len()];
    let mut boundary = vec![Vec3::ZERO; g.positions.len()];
    for t in &reference.triangles {
        let p = t.map(|v| Vec3::from_array(reference.positions[v as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        for k in 0..3 {
            let a = t[k] as usize;
            normal[a] += n;
            top[a] |= n.y > 0.8;
            // Oriented neighbour vectors cancel on closed fans, remain at open borders.
            boundary[a] += p[(k + 1) % 3] - p[(k + 2) % 3];
        }
    }
    let mut out = g.clone();
    for (i, p) in out.positions.iter_mut().enumerate() {
        if top[i] || boundary[i].length_squared() > 0.0001 {
            continue;
        }
        let n = normal[i].normalize_or_zero();
        if n.y.abs() > 0.65 {
            continue;
        }
        let horizontal = Vec3::new(n.x, 0., n.z).normalize_or_zero();
        let q = Vec3::from_array(*p) + Vec3::from_array(origin);
        let phase = crate::mesh_chamfer::random(c.seed) * 6.;
        let strata = (q.y / c.stratum_height + 0.11 * q.x / c.joint_width + phase).rem_euclid(1.);
        let joint = ((q.x + 0.43 * q.z) / c.joint_width + phase).fract().abs();
        // Asymmetric planar teeth, with a narrower setback at each stratum joint.
        let layer = if strata < 0.18 {
            strata / 0.18
        } else {
            1. - (strata - 0.18) * 0.6
        };
        let block = if joint < 0.25 {
            joint / 0.25
        } else {
            1. - (joint - 0.25) * 0.45
        };
        *p = (Vec3::from_array(*p) + horizontal * c.amplitude.min(room[i]) * (layer * block - 0.3))
            .to_array();
    }
    let displaced = out.positions.clone();
    let mut accepted = false;
    for attempt in 0..9 {
        let fraction = 0.5f32.powi(attempt);
        for (i, p) in out.positions.iter_mut().enumerate() {
            let a = Vec3::from_array(g.positions[i]);
            let b = Vec3::from_array(displaced[i]);
            *p = (a + (b - a) * fraction).to_array();
        }
        if g.triangles.iter().all(|t| {
            let a = t.map(|i| Vec3::from_array(g.positions[i as usize]));
            let b = t.map(|i| Vec3::from_array(out.positions[i as usize]));
            (a[1] - a[0])
                .cross(a[2] - a[0])
                .dot((b[1] - b[0]).cross(b[2] - b[0]))
                > 0.
        }) {
            accepted = true;
            break;
        }
    }
    if !accepted {
        return Err(invalid(
            "Wall relief would invert source faces; reduce amplitude",
        ));
    }
    out.material_coordinates = None;
    out.validate()?;
    Ok(out)
}
