//! Pure canonical-source transfer. The caller owns semantic surface correspondence
//! and publishes the returned asset only after this complete evaluation succeeds.
use crate::{
    meshes,
    model::{Result, invalid},
};
use glam::{DQuat, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Tolerance in normalized material space, independent of object dimensions.
pub const MAPPING_TOLERANCE: f64 = 1e-5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplacementPolicy {
    /// Keep the authored vector in local object units. This is not normal transport.
    Physical,
    /// Apply the reference-to-target dimensional scale in local object XYZ.
    Proportional,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Correspondence {
    pub triangle: u32,
    pub barycentric: [f32; 3],
}

pub struct Evaluation<'a> {
    pub source: &'a meshes::Asset,
    pub reference_base: &'a meshes::Geometry,
    pub target_base: &'a meshes::Geometry,
    pub reference_dimensions: [f32; 3],
    pub target_dimensions: [f32; 3],
    pub correspondences: &'a [Correspondence],
    pub detail_policy: DisplacementPolicy,
    pub animation_translation_policy: DisplacementPolicy,
}

/// Explicit material coordinates and mapped global bind origins for non-affine
/// authoring volumes. The caller owns semantic identities and the volume map.
pub struct MaterialDomain<'a> {
    pub reference: &'a [[f32; 3]],
    pub target: &'a [[f32; 3]],
    pub bone_origins: &'a [[f32; 3]],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub source_vertices: usize,
    pub target_vertices: usize,
    /// Maximum Euclidean distance in normalized material coordinates.
    pub max_mapping_error: f64,
    /// Maximum output detail-vector length in local object units.
    pub max_detail_displacement: f64,
    pub adapted_keyframes: usize,
}

#[derive(Debug, Clone)]
pub struct Evaluated {
    pub asset: meshes::Asset,
    pub report: Report,
}

fn vector(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
fn array(p: DVec3) -> [f32; 3] {
    p.to_array().map(|x| x as f32)
}
fn rotation(q: [f32; 4]) -> DQuat {
    DQuat::from_array(q.map(f64::from)).normalize()
}
fn validate_asset(asset: &meshes::Asset) -> Result<()> {
    meshes::Document {
        objects: Default::default(),
        assets: vec![asset.clone()],
        instances: vec![],
    }
    .validate()
}
fn dimensions(d: [f32; 3]) -> Result<DVec3> {
    if d.iter().any(|x| !x.is_finite() || *x <= 0.) {
        return Err(invalid(
            "Preservation dimensions must be finite and positive",
        ));
    }
    Ok(vector(d))
}
fn material_bounds(geometry: &meshes::Geometry, dimensions: DVec3) -> Result<()> {
    for &p in &geometry.positions {
        if (vector(p) / dimensions).abs().max_element() > 0.5 + MAPPING_TOLERANCE {
            return Err(invalid(
                "Preservation base lies outside its centered material volume",
            ));
        }
    }
    Ok(())
}
fn barycentric(c: &Correspondence, reference: &meshes::Geometry) -> Result<([usize; 3], [f64; 3])> {
    let triangle = reference
        .triangles
        .get(c.triangle as usize)
        .ok_or_else(|| invalid("Preservation correspondence references a missing triangle"))?;
    if c.barycentric.iter().any(|x| {
        !x.is_finite()
            || f64::from(*x) < -MAPPING_TOLERANCE
            || f64::from(*x) > 1. + MAPPING_TOLERANCE
    }) {
        return Err(invalid(
            "Preservation barycentric coordinates must be finite within0..1",
        ));
    }
    let sum: f64 = c.barycentric.iter().map(|x| f64::from(*x)).sum();
    if (sum - 1.).abs() > MAPPING_TOLERANCE {
        return Err(invalid("Preservation barycentric coordinates must sum to1"));
    }
    let clamped = c.barycentric.map(|x| f64::from(x).clamp(0., 1.));
    let clamped_sum: f64 = clamped.iter().sum();
    Ok((
        triangle.map(|x| x as usize),
        clamped.map(|x| x / clamped_sum),
    ))
}
fn interpolate(positions: &[[f32; 3]], indices: [usize; 3], weights: [f64; 3]) -> DVec3 {
    indices
        .into_iter()
        .zip(weights)
        .map(|(i, w)| vector(positions[i]) * w)
        .sum()
}
pub(crate) fn transfer_weight(
    rig: &meshes::Rig,
    indices: [usize; 3],
    bary: [f64; 3],
) -> Result<meshes::Weight> {
    let mut influences = [0.; meshes::MAX_BONES];
    for (index, factor) in indices.into_iter().zip(bary) {
        let source = &rig.weights[index];
        let sum: f64 = source.influences().map(|(_, w)| f64::from(w)).sum();
        for (joint, weight) in source.influences() {
            influences[joint as usize] += f64::from(weight) / sum * factor;
        }
    }
    meshes::Weight::from_sums(&influences)
}

/// Map bind origins by dimensional scale; retain global rotations. Rigid bone
/// frames cannot represent the shear of a full nonuniform affine frame mapping.
fn adapt_rig(
    source: &meshes::Rig,
    scale: DVec3,
    origins: Option<&[[f32; 3]]>,
) -> (meshes::Rig, [DQuat; meshes::MAX_BONES]) {
    let mut globals = [DVec3::ZERO; meshes::MAX_BONES];
    let mut rotations = [DQuat::IDENTITY; meshes::MAX_BONES];
    let mut mapped = [DVec3::ZERO; meshes::MAX_BONES];
    let mut bones = source.bones.clone();
    for (i, bone) in source.bones.iter().enumerate() {
        let local = vector(bone.translation);
        let local_rotation = rotation(bone.rotation);
        if let Some(parent) = bone.parent {
            let p = parent as usize;
            globals[i] = globals[p] + rotations[p] * local;
            rotations[i] = (rotations[p] * local_rotation).normalize();
            mapped[i] = origins.map_or(scale * globals[i], |o| vector(o[i]));
            let relative = if origins.is_some() {
                mapped[i] - mapped[p]
            } else {
                scale * (globals[i] - globals[p])
            };
            bones[i].translation = array(rotations[p].conjugate() * relative);
        } else {
            globals[i] = local;
            rotations[i] = local_rotation;
            mapped[i] = origins.map_or(scale * local, |o| vector(o[i]));
            bones[i].translation = array(mapped[i]);
        }
    }
    (
        meshes::Rig {
            bones,
            weights: Vec::new(),
        },
        rotations,
    )
}

fn adapt_clips(
    source: &meshes::Asset,
    rig: &meshes::Rig,
    global_rotations: &[DQuat; meshes::MAX_BONES],
    scale: DVec3,
    policy: DisplacementPolicy,
) -> (Vec<Arc<meshes::Clip>>, usize) {
    let old = source.rig.as_ref().expect("validated source rig");
    let mut count = 0;
    let clips = source
        .clips
        .iter()
        .map(|clip| {
            let mut result = (**clip).clone();
            for track in &mut result.tracks {
                let index = track.bone as usize;
                let old_bone = &old.bones[index];
                let new_bone = &rig.bones[index];
                let old_rotation = rotation(old_bone.rotation);
                let new_rotation = rotation(new_bone.rotation);
                for key in &mut track.keys {
                    // Absolute local keys become rest-relative deltas, then compile
                    // back to absolute keys against the new local rest transform.
                    let mut delta = old_rotation.conjugate()
                        * (vector(key.translation) - vector(old_bone.translation));
                    if policy == DisplacementPolicy::Proportional {
                        let global = global_rotations[index];
                        delta = global.conjugate() * (scale * (global * delta));
                    }
                    key.translation = array(vector(new_bone.translation) + new_rotation * delta);
                    let delta_rotation = old_rotation.conjugate() * rotation(key.rotation);
                    key.rotation = (new_rotation * delta_rotation)
                        .normalize()
                        .to_array()
                        .map(|x| x as f32);
                    count += 1;
                }
            }
            Arc::new(result)
        })
        .collect();
    (clips, count)
}

pub fn evaluate(input: &Evaluation<'_>) -> Result<Evaluated> {
    evaluate_inner(input, None)
}

pub fn evaluate_material(input: &Evaluation<'_>, domain: &MaterialDomain<'_>) -> Result<Evaluated> {
    evaluate_inner(input, Some(domain))
}

fn evaluate_inner(
    input: &Evaluation<'_>,
    domain: Option<&MaterialDomain<'_>>,
) -> Result<Evaluated> {
    let reference_dimensions = dimensions(input.reference_dimensions)?;
    let target_dimensions = dimensions(input.target_dimensions)?;
    let scale = target_dimensions / reference_dimensions;
    validate_asset(input.source)?;
    input.reference_base.validate()?;
    input.target_base.validate()?;
    if input.source.geometry.positions.len() != input.reference_base.positions.len()
        || input.source.geometry.triangles != input.reference_base.triangles
    {
        return Err(invalid(
            "Preservation source topology differs from canonical base",
        ));
    }
    if input.correspondences.len() != input.target_base.positions.len() {
        return Err(invalid(
            "Preservation requires one correspondence per target vertex",
        ));
    }
    // A curved material volume can leave the centered physical box. Its
    // normalized coordinates are validated below; the affine path still needs
    // physical bounds because it derives material coordinates from XYZ.
    if domain.is_none() {
        material_bounds(input.reference_base, reference_dimensions)?;
        material_bounds(input.target_base, target_dimensions)?;
    }
    if let Some(d) = domain
        && (d.reference.len() != input.reference_base.positions.len()
            || d.target.len() != input.target_base.positions.len()
            || d.bone_origins.len() != input.source.rig.as_ref().map_or(0, |r| r.bones.len())
            || d.reference.iter().chain(d.target).any(|p| {
                p.iter()
                    .any(|v| !v.is_finite() || v.abs() > 0.5 + MAPPING_TOLERANCE as f32)
            })
            || d.bone_origins.iter().flatten().any(|v| !v.is_finite()))
    {
        return Err(invalid(
            "Invalid explicit preservation material domain or bind origins",
        ));
    }
    // Validate every map before allocating the evaluated geometry/rig/clips.
    let mut max_mapping_error: f64 = 0.;
    for (index, (&point, correspondence)) in input
        .target_base
        .positions
        .iter()
        .zip(input.correspondences)
        .enumerate()
    {
        let (indices, bary) = barycentric(correspondence, input.reference_base)?;
        let (reference, target) = if let Some(d) = domain {
            (
                interpolate(d.reference, indices, bary),
                vector(d.target[index]),
            )
        } else {
            (
                interpolate(&input.reference_base.positions, indices, bary) / reference_dimensions,
                vector(point) / target_dimensions,
            )
        };
        let error = (reference - target).length();
        if !error.is_finite() || error > MAPPING_TOLERANCE {
            return Err(invalid(
                "Preservation correspondence does not reconstruct target material position",
            ));
        }
        max_mapping_error = max_mapping_error.max(error);
    }
    let (mut rig, rotations) = match &input.source.rig {
        Some(source) => {
            let (mut rig, rotations) = adapt_rig(source, scale, domain.map(|d| d.bone_origins));
            rig.weights.reserve(input.correspondences.len());
            (Some(rig), rotations)
        }
        None => (None, [DQuat::IDENTITY; meshes::MAX_BONES]),
    };
    let mut positions = Vec::with_capacity(input.target_base.positions.len());
    let mut material_coordinates = input
        .source
        .geometry
        .material_coordinates
        .as_ref()
        .map(|_| Vec::with_capacity(input.target_base.positions.len()));
    let mut max_detail_displacement: f64 = 0.;
    for (&point, correspondence) in input
        .target_base
        .positions
        .iter()
        .zip(input.correspondences)
    {
        let (indices, bary) = barycentric(correspondence, input.reference_base)?;
        if let (Some(source), Some(target)) = (
            &input.source.geometry.material_coordinates,
            &mut material_coordinates,
        ) {
            target.push(array(interpolate(source, indices, bary)));
        }
        // Subtract each canonical vertex first, avoiding cancellation between
        // interpolated large base positions for a small authored displacement.
        let mut delta: DVec3 = indices
            .into_iter()
            .zip(bary)
            .map(|(i, w)| {
                (vector(input.source.geometry.positions[i])
                    - vector(input.reference_base.positions[i]))
                    * w
            })
            .sum();
        if input.detail_policy == DisplacementPolicy::Proportional {
            delta *= scale;
        }
        max_detail_displacement = max_detail_displacement.max(delta.length());
        positions.push(array(vector(point) + delta));
        if let (Some(source), Some(target)) = (&input.source.rig, &mut rig) {
            target.weights.push(transfer_weight(source, indices, bary)?);
        }
    }
    let geometry = meshes::Geometry {
        material_coordinates,
        positions,
        triangles: input.target_base.triangles.clone(),
    };
    geometry.validate()?;
    // Local fold/degeneracy check, not a global self-intersection test.
    for triangle in &geometry.triangles {
        let [a, b, c] = triangle.map(|i| vector(geometry.positions[i as usize]));
        let [x, y, z] = triangle.map(|i| vector(input.target_base.positions[i as usize]));
        if (b - a).cross(c - a).dot((y - x).cross(z - x)) <= 0. {
            return Err(invalid(
                "Preservation detail folds a triangle relative to target base",
            ));
        }
    }
    let (clips, adapted_keyframes) = if let Some(ref rig) = rig {
        adapt_clips(
            input.source,
            rig,
            &rotations,
            scale,
            input.animation_translation_policy,
        )
    } else {
        (Vec::new(), 0)
    };
    let asset = meshes::Asset {
        id: input.source.id.clone(),
        geometry: Arc::new(geometry),
        shading: input.source.shading,
        material: input.source.material.clone(),
        rig: rig.map(Arc::new),
        clips,
    };
    validate_asset(&asset)?;
    Ok(Evaluated {
        asset,
        report: Report {
            source_vertices: input.source.geometry.positions.len(),
            target_vertices: input.target_base.positions.len(),
            max_mapping_error,
            max_detail_displacement,
            adapted_keyframes,
        },
    })
}
