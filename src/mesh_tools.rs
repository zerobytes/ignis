//! Shared primitives, triangle picking and sculpt previews for MCP and the editor.
use crate::{
    meshes,
    model::{Camera, Result, invalid},
};
use glam::{DVec3, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

/// Edge length requested by dynamic sculpt refinement, independent of pressure.
/// Constant lengths are in asset-local metres, not pixels or world-space scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum SculptDetail {
    Brush { percent: f32 },
    Constant { edge_length: f32 },
}
impl Default for SculptDetail {
    fn default() -> Self {
        Self::Brush { percent: 12.5 }
    }
}
impl SculptDetail {
    pub fn validate(self) -> Result<()> {
        let valid = match self {
            Self::Brush { percent } => percent.is_finite() && (3.125..=100.).contains(&percent),
            Self::Constant { edge_length } => {
                edge_length.is_finite() && (0.00001..=100.).contains(&edge_length)
            }
        };
        if !valid {
            return Err(invalid(
                "Sculpt detail requires brush3.125..100percent or constant0.00001..100local metres",
            ));
        }
        Ok(())
    }
    pub(crate) fn max_edge(self, radius: f32) -> f32 {
        match self {
            Self::Brush { percent } => radius * (percent / 100.),
            Self::Constant { edge_length } => edge_length,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Primitive {
    Sphere,
    Cube,
    Cylinder,
    Plane,
}
pub fn primitive(kind: Primitive, resolution: u16, size: f32) -> Result<meshes::Geometry> {
    if !(4..=32).contains(&resolution) || !size.is_finite() || !(0.01..=100.).contains(&size) {
        return Err(invalid(
            "Primitive requires resolution4..32 and size0.01..100m",
        ));
    }
    if kind == Primitive::Plane {
        return meshes::Geometry::grid(resolution, size);
    }
    let n = resolution as usize;
    let mut p = Vec::<[f32; 3]>::new();
    let mut t = Vec::<[u32; 3]>::new();
    match kind {
        Primitive::Sphere => {
            p.push([0., 1., 0.]);
            for j in 1..n {
                let theta = std::f32::consts::PI * j as f32 / n as f32;
                for i in 0..2 * n {
                    let phi = std::f32::consts::TAU * i as f32 / (2 * n) as f32;
                    p.push([
                        theta.sin() * phi.cos(),
                        theta.cos(),
                        theta.sin() * phi.sin(),
                    ]);
                }
            }
            let bottom = p.len() as u32;
            p.push([0., -1., 0.]);
            for i in 0..2 * n {
                let a = 1 + i as u32;
                let b = 1 + ((i + 1) % (2 * n)) as u32;
                t.push([0, a, b]);
                t.push([
                    bottom,
                    1 + ((n - 2) * 2 * n + i) as u32,
                    1 + ((n - 2) * 2 * n + (i + 1) % (2 * n)) as u32,
                ]);
            }
            for j in 0..n - 2 {
                for i in 0..2 * n {
                    let a = 1 + (j * 2 * n + i) as u32;
                    let b = 1 + (j * 2 * n + (i + 1) % (2 * n)) as u32;
                    let c = a + (2 * n) as u32;
                    let d = b + (2 * n) as u32;
                    t.extend([[a, c, b], [b, c, d]]);
                }
            }
        }
        Primitive::Cube => {
            let mut ids = BTreeMap::new();
            for axis in 0..3 {
                for sign in [-1, 1] {
                    let mut grid = Vec::new();
                    for j in 0..=n {
                        for i in 0..=n {
                            let mut key = [0_i32; 3];
                            key[axis] = sign * n as i32;
                            key[(axis + 1) % 3] = 2 * i as i32 - n as i32;
                            key[(axis + 2) % 3] = 2 * j as i32 - n as i32;
                            let id = *ids.entry(key).or_insert_with(|| {
                                let id = p.len() as u32;
                                p.push(key.map(|v| v as f32 / n as f32));
                                id
                            });
                            grid.push(id);
                        }
                    }
                    for j in 0..n {
                        for i in 0..n {
                            let a = j * (n + 1) + i;
                            t.extend([
                                [grid[a], grid[a + 1], grid[a + n + 1]],
                                [grid[a + 1], grid[a + n + 2], grid[a + n + 1]],
                            ]);
                        }
                    }
                }
            }
        }
        Primitive::Cylinder => {
            for j in 0..=n {
                for i in 0..2 * n {
                    let angle = std::f32::consts::TAU * i as f32 / (2 * n) as f32;
                    p.push([angle.cos(), -1. + 2. * j as f32 / n as f32, angle.sin()]);
                }
            }
            for j in 0..n {
                for i in 0..2 * n {
                    let a = (j * 2 * n + i) as u32;
                    let b = (j * 2 * n + (i + 1) % (2 * n)) as u32;
                    let c = a + (2 * n) as u32;
                    let d = b + (2 * n) as u32;
                    t.extend([[a, b, c], [b, d, c]]);
                }
            }
            let bottom = p.len() as u32;
            p.push([0., -1., 0.]);
            let top = p.len() as u32;
            p.push([0., 1., 0.]);
            for i in 0..2 * n {
                let a = i as u32;
                let b = ((i + 1) % (2 * n)) as u32;
                t.extend([
                    [bottom, a, b],
                    [top, a + (n * 2 * n) as u32, b + (n * 2 * n) as u32],
                ]);
            }
        }
        Primitive::Plane => unreachable!(),
    }
    // All closed primitives are convex about the origin; orient every face outward.
    for face in &mut t {
        let [a, b, c] = face.map(|i| Vec3::from(p[i as usize]));
        if (b - a).cross(c - a).dot(a + b + c) < 0. {
            face.swap(1, 2);
        }
    }
    for v in &mut p {
        *v = (Vec3::from(*v) * (size * 0.5)).to_array();
    }
    let g = meshes::Geometry {
        positions: p,
        triangles: t,
        material_coordinates: None,
    };
    g.validate()?;
    Ok(g)
}
#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub instance: String,
    pub asset: String,
    pub face: u32,
    pub position: [f32; 3],
    pub local_position: [f32; 3],
    pub normal: [f32; 3],
    pub distance: f32,
}
/// Shared-edge-consistent segment/triangle test in sheared ray coordinates.
/// Neighbors evaluate their common edge with identical products and opposite
/// signs, preventing round-off from making both reject the visible surface.
/// Query arithmetic is f64; geometry, poses and Hit retain their formats.
struct TriangleRay {
    start: DVec3,
    axes: [usize; 3],
    shear: [f64; 3],
}
impl TriangleRay {
    fn new(start: Vec3, delta: Vec3) -> Self {
        let d = delta.as_dvec3();
        let mut z = 0;
        for i in 1..3 {
            if d[i].abs() > d[z].abs() {
                z = i;
            }
        }
        let mut axes = [(z + 1) % 3, (z + 2) % 3, z];
        if d[z] < 0. {
            axes.swap(0, 1);
        }
        Self {
            start: start.as_dvec3(),
            axes,
            shear: [d[axes[0]] / d[z], d[axes[1]] / d[z], 1. / d[z]],
        }
    }
    fn intersect(&self, vertices: [Vec3; 3]) -> Option<(f64, [f64; 3])> {
        let [a, b, c] = vertices.map(|v| v.as_dvec3());
        if (b - a).cross(c - a).length_squared() == 0. {
            return None;
        }
        let [x, y, z] = self.axes;
        let [sx, sy, sz] = self.shear;
        let [a, b, c] = [a, b, c].map(|p| {
            let p = p - self.start;
            DVec3::new(p[x] - sx * p[z], p[y] - sy * p[z], p[z] * sz)
        });
        let edges = [
            b.x * c.y - b.y * c.x,
            c.x * a.y - c.y * a.x,
            a.x * b.y - a.y * b.x,
        ];
        if edges.iter().any(|&e| e < 0.) && edges.iter().any(|&e| e > 0.) {
            return None;
        }
        let det = edges.iter().sum::<f64>();
        if det == 0. {
            return None;
        }
        let time = (edges[0] * a.z + edges[1] * b.z + edges[2] * c.z) / det;
        if !time.is_finite() || !(0.0..=1.0).contains(&time) {
            return None;
        }
        Some((time, edges.map(|e| e / det)))
    }
}
#[allow(clippy::too_many_arguments)]
pub fn pick(
    document: &meshes::Document,
    camera: &Camera,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    tick: u64,
    consumed: &[&str],
) -> Result<Option<Hit>> {
    let (start, delta) = crate::picking::ray(camera, x, y, width, height)?;
    let ray = TriangleRay::new(start, delta);
    let mut best: Option<(f64, Hit)> = None;
    for (instance, pose) in document.instances.iter().zip(document.poses(tick)) {
        if consumed.contains(&instance.id.as_str()) {
            continue;
        }
        let asset = document.asset(&instance.asset)?;
        for (index, face) in asset.geometry.triangles.iter().enumerate() {
            let [a, b, c] = face.map(|i| Vec3::from(pose.positions[i as usize]));
            let Some((time, weights)) = ray.intersect([a, b, c]) else {
                continue;
            };
            if best.as_ref().is_some_and(|(old, hit)| {
                time > *old || (time == *old && instance.id >= hit.instance)
            }) {
                continue;
            }
            let world = (start.as_dvec3() + delta.as_dvec3() * time).as_vec3();
            let [la, lb, lc] = face.map(|i| Vec3::from(asset.geometry.positions[i as usize]));
            best = Some((
                time,
                Hit {
                    instance: instance.id.clone(),
                    asset: asset.id.clone(),
                    face: index as u32,
                    position: world.to_array(),
                    local_position: (la.as_dvec3() * weights[0]
                        + lb.as_dvec3() * weights[1]
                        + lc.as_dvec3() * weights[2])
                        .as_vec3()
                        .to_array(),
                    normal: (b.as_dvec3() - a.as_dvec3())
                        .cross(c.as_dvec3() - a.as_dvec3())
                        .normalize_or_zero()
                        .as_vec3()
                        .to_array(),
                    distance: world.distance(camera.position.into()),
                },
            ));
        }
    }
    Ok(best.map(|(_, hit)| hit))
}
/// Preview uses exactly the same deformation kernel as EditMesh::Sculpt.
pub fn sculpt_preview(asset: &meshes::Asset, stroke: &meshes::Stroke) -> Result<meshes::Asset> {
    let mut result = asset.clone();
    Arc::make_mut(&mut result.geometry).sculpt(stroke)?;
    Ok(result)
}
