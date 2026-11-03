//! Morphology-independent analytic two-segment contact IK. No actor/collider ownership.
use crate::{
    meshes::{MAX_BONES, Rig},
    model::{Result, invalid},
};
use glam::{Mat4, Quat, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CHAINS: usize = 8;
pub const JOINTS: usize = CHAINS * 3;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Chain {
    /// Direct ancestor chain: shoulder/hip -> intermediate -> effector, any species.
    pub bones: [u8; 3],
    /// Rest-space bend direction; projected perpendicular to target direction.
    pub pole: [f32; 3],
    /// Effector local surface normal (e.g. sole +Y, claw -Z).
    pub normal_axis: [f32; 3],
    pub probe_up: f32,
    pub probe_down: f32,
    pub max_correction: f32,
    /// Zero is a point contact. Positive radius requires four additional supports.
    #[serde(default)]
    pub support_radius: f32,
    #[serde(default = "support_tolerance")]
    pub support_tolerance: f32,
    pub sole_offset: f32,
    pub max_slope_degrees: f32,
}
fn support_tolerance() -> f32 {
    0.03
}
impl Chain {
    pub fn validate(&self, rig: &Rig) -> Result<()> {
        let [a, b, c] = self.bones.map(usize::from);
        let unit = |v: [f32; 3]| {
            let v = Vec3::from(v);
            v.is_finite() && (v.length_squared() - 1.).abs() < 0.001
        };
        let range = |x: f32, lo: f32, hi: f32| x.is_finite() && (lo..=hi).contains(&x);
        if [a, b, c].iter().any(|i| *i >= rig.bones.len())
            || a == b
            || b == c
            || a == c
            || rig.bones[b].parent != Some(a as u8)
            || rig.bones[c].parent != Some(b as u8)
            || Vec3::from(rig.bones[b].translation).length() < 0.001
            || Vec3::from(rig.bones[c].translation).length() < 0.001
            || !unit(self.pole)
            || !unit(self.normal_axis)
            || !range(self.probe_up, 0.001, 2.)
            || !range(self.probe_down, 0.001, 2.)
            || !range(self.max_correction, 0.001, 2.)
            || !range(self.support_radius, 0., 1.)
            || !range(self.support_tolerance, 0., 0.5)
            || !range(self.sole_offset, 0., 1.)
            || !range(self.max_slope_degrees, 0., 85.)
        {
            return Err(invalid(
                "Invalid IK direct chain, nonzero lengths, axes, contact bounds or slope",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Joint {
    pub bone: u8,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}
/// Evaluates skeleton globals from bounded local transforms. Parents precede children.
pub(crate) fn globals(rig: &Rig, local: &[Mat4; MAX_BONES]) -> [Mat4; MAX_BONES] {
    let mut out = [Mat4::IDENTITY; MAX_BONES];
    for (i, b) in rig.bones.iter().enumerate() {
        out[i] = b.parent.map_or(local[i], |p| out[p as usize] * local[i]);
    }
    out
}
fn set_global_rotation(rig: &Rig, local: &mut [Mat4; MAX_BONES], index: usize, rotation: Quat) {
    let world = globals(rig, local);
    let parent = rig.bones[index].parent.map_or(Quat::IDENTITY, |p| {
        world[p as usize].to_scale_rotation_translation().1
    });
    let (_, _, t) = local[index].to_scale_rotation_translation();
    local[index] = Mat4::from_rotation_translation((parent.inverse() * rotation).normalize(), t);
}
/// Returns false for degenerate/unreachable correction. Lengths never stretch;
/// an unreachable target is clamped to the limb's reachable annulus.
pub(crate) fn solve(
    rig: &Rig,
    local: &mut [Mat4; MAX_BONES],
    chain: &Chain,
    target: Vec3,
    normal: Vec3,
) -> bool {
    let [a, b, c] = chain.bones.map(usize::from);
    let g = globals(rig, local);
    let [root, joint, end] = [a, b, c].map(|i| g[i].transform_point3(Vec3::ZERO));
    let l1 = root.distance(joint);
    let l2 = joint.distance(end);
    let d = target - root;
    if !d.is_finite() || d.length() < 0.00001 || l1 < 0.001 || l2 < 0.001 {
        return false;
    }
    let axis = d.normalize();
    let pole = Vec3::from(chain.pole);
    let mut bend = (pole - axis * pole.dot(axis)).normalize_or_zero();
    if bend == Vec3::ZERO {
        bend = axis.any_orthonormal_vector();
    }
    let reach = d
        .length()
        .clamp((l1 - l2).abs() + 0.00001, l1 + l2 - 0.00001);
    let along = (l1 * l1 - l2 * l2 + reach * reach) / (2. * reach);
    let height = (l1 * l1 - along * along).max(0.).sqrt();
    let desired_joint = root + axis * along + bend * height;
    let desired_end = root + axis * reach;
    let q1 = Quat::from_rotation_arc(
        (joint - root).normalize(),
        (desired_joint - root).normalize(),
    ) * g[a].to_scale_rotation_translation().1;
    set_global_rotation(rig, local, a, q1);
    let g = globals(rig, local);
    let joint = g[b].transform_point3(Vec3::ZERO);
    let end = g[c].transform_point3(Vec3::ZERO);
    let q2 = Quat::from_rotation_arc((end - joint).normalize(), (desired_end - joint).normalize())
        * g[b].to_scale_rotation_translation().1;
    set_global_rotation(rig, local, b, q2);
    let g = globals(rig, local);
    let rotation = g[c].to_scale_rotation_translation().1;
    let from = rotation * Vec3::from(chain.normal_axis);
    set_global_rotation(
        rig,
        local,
        c,
        Quat::from_rotation_arc(from.normalize(), normal.normalize()) * rotation,
    );
    true
}
