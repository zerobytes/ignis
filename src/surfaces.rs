//! Static physical meshes. Full authoring geometry, independent of render LOD.
use crate::model::{Result, Scene, invalid};
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn validate(scene: &Scene) -> Result<()> {
    if scene.collision_meshes.is_empty() {
        return Ok(());
    }
    if !scene.collision_meshes.is_empty() && scene.version < 104 {
        return Err(invalid("Mesh collision requires scene104"));
    }
    let instances: BTreeMap<_, _> = scene
        .meshes
        .instances
        .iter()
        .map(|i| (i.id.as_str(), i))
        .collect();
    let assets: BTreeMap<_, _> = scene
        .meshes
        .assets
        .iter()
        .map(|a| (a.id.as_str(), a))
        .collect();
    let mut seen = BTreeSet::new();
    let mut scales = BTreeSet::new();
    for id in &scene.collision_meshes {
        let i = instances
            .get(id.as_str())
            .ok_or_else(|| invalid("Collision mesh instance missing"))?;
        let a = assets[i.asset.as_str()];
        if !seen.insert(id) || i.attachment.is_some() || i.playback.is_some() || a.rig.is_some() {
            return Err(invalid(
                "Collision meshes must be unique static instances without attachment, playback or rig",
            ));
        }
        let q = glam::Quat::from_array(i.rotation).normalize();
        if scene
            .plants
            .connections
            .iter()
            .any(|c| c.child_instance == *id)
        {
            return Err(invalid(
                "A consumed child mesh cannot be a separate physical surface",
            ));
        }
        if scales.insert((i.asset.as_str(), i.scale.to_bits())) {
            for t in &a.geometry.triangles {
                let [a, b, c] =
                    t.map(|index| glam::Vec3::from(a.geometry.positions[index as usize]) * i.scale);
                if !(b - a).cross(c - a).length_squared().is_normal() {
                    return Err(invalid(
                        "Physical triangles lose area at the instance scale",
                    ));
                }
            }
        }
        for p in &a.geometry.positions {
            let p = q * (glam::Vec3::from(*p) * i.scale) + glam::Vec3::from(i.position);
            if p.to_array()
                .iter()
                .any(|x| !x.is_finite() || x.abs() > 10000.)
            {
                return Err(invalid(
                    "Physical mesh must fit resident coordinates +/-10000m",
                ));
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct Surface {
    pub id: String,
    pub asset: String,
    pub pose: Isometry<f32>,
    pub shape: SharedShape,
}
impl Surface {
    pub fn collider(&self) -> ColliderBuilder {
        ColliderBuilder::new(self.shape.clone())
            .position(self.pose)
            .friction(0.7)
            .restitution(0.)
    }
    /// World triangles intersecting an axis-aligned volume. Bounded and stable order.
    pub(crate) fn triangles(&self, min: glam::Vec3, max: glam::Vec3) -> Option<Vec<[[f32; 3]; 3]>> {
        let mesh = self.shape.as_trimesh().expect("static mesh");
        let world = rapier3d::parry::bounding_volume::Aabb::new(
            point![min.x, min.y, min.z],
            point![max.x, max.y, max.z],
        );
        let local = world.transform_by(&self.pose.inverse());
        let mut ids = Vec::new();
        for id in mesh.bvh().intersect_aabb(&local) {
            if ids.len() == 4096 {
                return None;
            }
            ids.push(id);
        }
        ids.sort_unstable();
        Some(
            ids.into_iter()
                .map(|id| {
                    let t = mesh.triangle(id);
                    [
                        (self.pose * t.a).into(),
                        (self.pose * t.b).into(),
                        (self.pose * t.c).into(),
                    ]
                })
                .collect(),
        )
    }
}

/// Build once per world; instances with the same asset and scale share one BVH.
/// Caller validated the Scene before publishing it.
pub fn prepare(scene: &Scene) -> Vec<Surface> {
    if scene.collision_meshes.is_empty() {
        return Vec::new();
    }
    let assets: BTreeMap<_, _> = scene
        .meshes
        .assets
        .iter()
        .map(|a| (a.id.as_str(), a))
        .collect();
    let instances: BTreeMap<_, _> = scene
        .meshes
        .instances
        .iter()
        .map(|i| (i.id.as_str(), i))
        .collect();
    let mut cache = BTreeMap::new();
    let mut ids: Vec<_> = scene.collision_meshes.iter().collect();
    ids.sort();
    ids.into_iter()
        .map(|id| {
            let i = instances[id.as_str()];
            let shape = cache
                .entry((i.asset.as_str(), i.scale.to_bits()))
                .or_insert_with(|| {
                    let g = &assets[i.asset.as_str()].geometry;
                    SharedShape::trimesh_with_flags(
                        g.positions
                            .iter()
                            .map(|p| point![p[0] * i.scale, p[1] * i.scale, p[2] * i.scale])
                            .collect(),
                        g.triangles.clone(),
                        TriMeshFlags::FIX_INTERNAL_EDGES,
                    )
                    .expect("validated nonempty triangles")
                })
                .clone();
            let q = glam::Quat::from_array(i.rotation).normalize();
            Surface {
                id: id.clone(),
                asset: i.asset.clone(),
                shape,
                pose: Isometry::from_parts(
                    Translation::from(vector![i.position[0], i.position[1], i.position[2]]),
                    UnitQuaternion::new_unchecked(Quaternion::new(q.w, q.x, q.y, q.z)),
                ),
            }
        })
        .collect()
}

pub fn inspect(scene: &Scene) -> serde_json::Value {
    serde_json::json!({"instances":scene.collision_meshes,"static_only":true,"source":"full_mesh_geometry","render_lod_affects_collision":false,"streaming":false})
}

#[path = "surfaces/support.rs"]
mod support;
pub(crate) use support::{Support, footprint, project};
