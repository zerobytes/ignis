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
        self.validate_hierarchy(rig, false)
    }
    /// Humanoid sources may contain intermediate twist/deformation joints.
    pub fn validate_ancestry(&self, rig: &Rig) -> Result<()> {
        self.validate_hierarchy(rig, true)
    }
    fn validate_hierarchy(&self, rig: &Rig, ancestors: bool) -> Result<()> {
        let [a, b, c] = self.bones.map(usize::from);
        let follows = |child: usize, parent: usize| {
            let mut cursor = rig.bones.get(child).and_then(|b| b.parent);
            for _ in 0..MAX_BONES {
                let Some(i) = cursor else {
                    return false;
                };
                if i as usize == parent {
                    return true;
                }
                if !ancestors {
                    return false;
                }
                cursor = rig.bones.get(i as usize).and_then(|b| b.parent);
            }
            false
        };
        let unit = |v: [f32; 3]| {
            let v = Vec3::from(v);
            v.is_finite() && (v.length_squared() - 1.).abs() < 0.001
        };
        let range = |x: f32, lo: f32, hi: f32| x.is_finite() && (lo..=hi).contains(&x);
        if [a, b, c].iter().any(|i| *i >= rig.bones.len())
            || a == b
            || b == c
            || a == c
            || !follows(b, a)
            || !follows(c, b)
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub chain: Chain,
    /// Asset-local coordinates, before the actor placement/scale.
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub weight: f32,
}

/// Bone matrices from the same native clip/cross-fade evaluator as rendering.
/// The asset must have passed the native document validator.
pub fn pose_joints(
    asset: &crate::meshes::Asset,
    pose: &crate::npc_presentation::Pose,
) -> Result<Vec<[[f32; 4]; 4]>> {
    pose.validate_asset(asset)?;
    let rig = asset
        .rig
        .as_ref()
        .ok_or_else(|| invalid("Pose query requires a rig"))?;
    let local = crate::meshes::avatar_locals(asset, pose);
    Ok(globals(rig, &local)[..rig.bones.len()]
        .iter()
        .map(Mat4::to_cols_array_2d)
        .collect())
}

/// Apply explicit contact targets without moving the actor or stretching bones.
pub fn constrain_pose(
    asset: &crate::meshes::Asset,
    pose: &crate::npc_presentation::Pose,
    targets: &[Target],
) -> Result<crate::npc_presentation::Pose> {
    pose.validate_asset(asset)?;
    let rig = asset
        .rig
        .as_ref()
        .ok_or_else(|| invalid("Contact solve requires rig"))?;
    if targets.len() > CHAINS {
        return Err(invalid("Too many contact targets"));
    }
    let mut used = [false; MAX_BONES];
    for t in targets {
        t.chain.validate_ancestry(rig)?;
        if !Vec3::from(t.point).is_finite()
            || Vec3::from(t.point).abs().max_element() > 10000.
            || !Vec3::from(t.normal).is_finite()
            || (Vec3::from(t.normal).length_squared() - 1.).abs() > 0.001
            || !t.weight.is_finite()
            || !(0.0..=1.).contains(&t.weight)
        {
            return Err(invalid("Invalid contact target/normal/weight"));
        }
        for b in t.chain.bones {
            if used[b as usize] {
                return Err(invalid("Contact chains cannot share joints"));
            }
            used[b as usize] = true;
        }
    }
    let mut local = crate::meshes::avatar_locals(asset, pose);
    let mut out = *pose;
    for t in targets {
        if t.weight == 0. {
            continue;
        }
        let before = t.chain.bones.map(|i| local[i as usize]);
        let global = globals(rig, &local);
        let end = global[t.chain.bones[2] as usize].transform_point3(Vec3::ZERO);
        let delta = Vec3::from(t.point) - end;
        let limited = delta.length() > t.chain.max_correction;
        let target = end + delta.clamp_length_max(t.chain.max_correction);
        if limited {
            out.ik_limited = out.ik_limited.saturating_add(1).min(CHAINS as u8);
        }
        if !solve(rig, &mut local, &t.chain, target, t.normal.into()) {
            out.ik_limited = out.ik_limited.saturating_add(1).min(CHAINS as u8);
            continue;
        }
        let reached = globals(rig, &local)[t.chain.bones[2] as usize].transform_point3(Vec3::ZERO);
        if !limited && reached.distance(target) > 0.0001 {
            out.ik_limited = out.ik_limited.saturating_add(1).min(CHAINS as u8);
        }
        for (index, &bone) in t.chain.bones.iter().enumerate() {
            let (_, a, translation) = before[index].to_scale_rotation_translation();
            let (_, b, _) = local[bone as usize].to_scale_rotation_translation();
            let rotation = a.slerp(b, t.weight).normalize();
            local[bone as usize] = Mat4::from_rotation_translation(rotation, translation);
            let joint = Joint {
                bone,
                translation: translation.to_array(),
                rotation: rotation.to_array(),
            };
            if let Some(i) = out.joints[..out.joint_count as usize]
                .iter()
                .position(|j| j.bone == bone)
            {
                out.joints[i] = joint;
            } else {
                if out.joint_count as usize >= JOINTS {
                    return Err(invalid("Contact pose exceeds joint budget"));
                }
                out.joints[out.joint_count as usize] = joint;
                out.joint_count += 1;
            }
        }
    }
    out.validate_asset(asset)?;
    Ok(out)
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
