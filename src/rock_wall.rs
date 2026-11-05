//! Experimental seeded planar rock field, extracted natively by Dual Contouring.
//! Local authoring tool, not a replacement for terrain/planet streaming.
use crate::model::{Result, invalid};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
mod dual;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub seed: u64,
    pub width_meters: f64,
    pub height_meters: f64,
    pub depth_meters: f64,
    pub layer_meters: f64,
    pub block_meters: f64,
    pub relief_meters: f64,
    pub bedding_slope: f64,
    pub joint_shear: f64,
    pub bevel_meters: f64,
    pub spacing_meters: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub settings: Settings,
    pub asset: String,
    pub instance: String,
    pub position: [f32; 3],
    pub color: [f32; 3],
    #[serde(default)]
    pub material: crate::materials::Material,
    #[serde(default)]
    pub collision: Option<bool>,
}
fn hash(mut s: u64) -> f64 {
    s = s.wrapping_add(0x9e3779b97f4a7c15);
    s = (s ^ (s >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    s = (s ^ (s >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((s ^ (s >> 31)) >> 11) as f64 / (1u64 << 53) as f64
}
#[derive(Clone)]
struct Convex {
    planes: Vec<(DVec3, f64)>,
}
impl Convex {
    // A min-of-planes cannot beat the union once any plane is below it.
    fn above(&self, p: DVec3, threshold: f64) -> Option<(f64, DVec3)> {
        let mut best = (f64::INFINITY, DVec3::Y);
        for &(n, d) in &self.planes {
            let v = d - n.dot(p);
            if v <= threshold {
                return None;
            }
            if v < best.0 {
                best = (v, n);
            }
        }
        Some(best)
    }
    fn value(&self, p: DVec3) -> (f64, DVec3) {
        let mut best = (f64::INFINITY, DVec3::Y);
        for &(n, d) in &self.planes {
            let v = d - n.dot(p);
            if v < best.0 {
                best = (v, n);
            }
        }
        best
    }
}
fn box_planes(lo: DVec3, hi: DVec3) -> Vec<(DVec3, f64)> {
    vec![
        (DVec3::X, hi.x),
        (-DVec3::X, -lo.x),
        (DVec3::Y, hi.y),
        (-DVec3::Y, -lo.y),
        (DVec3::Z, hi.z),
        (-DVec3::Z, -lo.z),
    ]
}
impl Settings {
    fn validate(&self) -> Result<()> {
        let length = |v: f64| v.is_finite() && (0.05..=1000.).contains(&v);
        if ![
            self.width_meters,
            self.height_meters,
            self.depth_meters,
            self.layer_meters,
            self.block_meters,
            self.relief_meters,
        ]
        .into_iter()
        .all(length)
            || !self.spacing_meters.is_finite()
            || !(0.02..=2.).contains(&self.spacing_meters)
            || !self.bedding_slope.is_finite()
            || self.bedding_slope.abs() > 0.75
            || !self.joint_shear.is_finite()
            || self.joint_shear.abs() > 0.75
            || !self.bevel_meters.is_finite()
            || !(0.001..=self.layer_meters.min(self.block_meters) * 0.4)
                .contains(&self.bevel_meters)
            || self.relief_meters > self.depth_meters * 2.
        {
            return Err(invalid(
                "Rock experiment requires finite metric dimensions, bevel below40% layer/block, relief<=2depth, spacing0.02..2 and slopes<=0.75",
            ));
        }
        let work = (self.height_meters + self.width_meters * self.bedding_slope.abs())
            / self.layer_meters
            * (self.width_meters + self.height_meters * self.joint_shear.abs())
            / self.block_meters;
        if work > 256. {
            return Err(invalid(
                "Rock experiment exceeds256 approximate fracture cells; split authoring region",
            ));
        }
        Ok(())
    }
}

pub fn compile(s: &Settings) -> Result<(Arc<crate::meshes::Geometry>, serde_json::Value)> {
    s.validate()?;
    let w = s.width_meters;
    let h = s.height_meters;
    let depth = s.depth_meters;
    // Bedding/joint coordinates share geological direction, with correlated local
    // planes rather than independent per-vertex noise. Transform plane normals
    // back to world space, preserving exact planar Hermite normals.
    let normal = |n: DVec3| DVec3::new(n.x - s.bedding_slope * n.y, n.y - s.joint_shear * n.x, n.z);
    let mut blocks = vec![];
    let y_min = -s.bedding_slope.abs() * w * 0.5 - s.layer_meters;
    let y_max = h + s.bedding_slope.abs() * w * 0.5 + s.layer_meters;
    let x_span = w * 0.5 + h * s.joint_shear.abs() + s.block_meters * 2.;
    let mut y = y_min;
    let mut row = 0u64;
    while y < y_max {
        let r = hash(s.seed ^ row.wrapping_mul(1009));
        let layer = s.layer_meters * (0.65 + 0.7 * r);
        let mut x = -x_span - s.block_meters * hash(s.seed ^ row.wrapping_mul(37));
        let mut col = 0u64;
        while x < x_span {
            let seed = s.seed ^ row.wrapping_mul(0x9e3779b9) ^ col.wrapping_mul(0x85ebca6b);
            let a = hash(seed);
            let b = hash(seed.wrapping_add(1));
            let c = hash(seed.wrapping_add(2));
            let width = s.block_meters * (0.55 + 0.95 * a);
            let front = -s.relief_meters * (0.15 + 0.85 * b);
            let tilt_x = (c - 0.5) * 0.9;
            let tilt_y = 0.18 + (a - 0.5) * 0.8;
            let bottom = y + layer * (b - 0.5) * 0.8;
            let top = y + layer * (1.12 + 0.4 * c);
            let mid = (bottom + top) * 0.5;
            let mut planes = box_planes(
                DVec3::new(x, bottom, -depth * 4.),
                DVec3::new(x + width, top, depth),
            );
            // The front fracture face is oblique, not an extruded vertical prism.
            planes.push((
                DVec3::new(tilt_x, tilt_y, -1.),
                -front + tilt_x * (x + width * 0.5) + tilt_y * mid,
            ));
            // Secondary fractures truncate a portion of each exposed slab. Their
            // intersections make irregular polygonal faces, not rectangular tiles.
            for j in 0..2u64 {
                let u = hash(seed.wrapping_add(100 + j * 3));
                let v = hash(seed.wrapping_add(101 + j * 3));
                let nx = (u - 0.5) * 1.5;
                let ny = (v - 0.5) * 1.3;
                planes.push((
                    DVec3::new(nx, ny, -1.),
                    -front
                        + nx * (x + width * 0.5)
                        + ny * mid
                        + s.relief_meters * (0.05 + 0.18 * v),
                ));
            }
            // Irregular chipped margins: bevels cut planes where front meets
            // cross joints and bedding. Different widths retain larger flat faces.
            for (nx, ny) in [(1., 0.), (-1., 0.), (0., 1.), (0., -1.)] {
                let u = hash(seed.wrapping_add((nx + 2. * ny + 8.) as u64));
                let cut = s.bevel_meters * (0.35 + 1.3 * u);
                let boundary = if nx > 0. {
                    x + width
                } else if nx < 0. {
                    -x
                } else if ny > 0. {
                    top
                } else {
                    -bottom
                };
                let weight = 0.5 + u;
                planes.push((DVec3::new(nx, ny, -weight), boundary - weight * front - cut));
            }
            // Convert local q=(x-shear*y,y-bedding*x,z) planes to world.
            for (n, d) in &mut planes {
                let world = normal(*n);
                let len = world.length();
                *n = world / len;
                *d /= len;
            }
            blocks.push(Convex { planes });
            x += width * (0.86 + 0.08 * c);
            col += 1;
            if blocks.len() > 512 {
                return Err(invalid("Rock fracture cell budget exceeded512"));
            }
        }
        y += layer;
        row += 1;
    }
    let core = Convex {
        planes: box_planes(
            DVec3::new(-w * 0.5, 0., 0.15),
            DVec3::new(w * 0.5, h, depth),
        ),
    };
    let clip = Convex {
        planes: box_planes(
            DVec3::new(-w * 0.5, 0., -depth * 5.),
            DVec3::new(w * 0.5, h, depth),
        ),
    };
    let field = |p: DVec3| {
        let mut v = core.value(p);
        for b in &blocks {
            if let Some(candidate) = b.above(p, v.0) {
                v = candidate;
            }
        }
        let bound = clip.value(p);
        if bound.0 < v.0 { bound } else { v }
    };
    // Include a full exterior collar. All six domain faces must remain empty.
    let pad = s.spacing_meters * 2.137;
    let lo = DVec3::new(
        -w * 0.5 - pad,
        -pad,
        -s.relief_meters - s.block_meters - s.layer_meters - pad,
    );
    let hi = DVec3::new(w * 0.5 + pad, h + pad, depth + pad);
    let cells = (hi - lo)
        .to_array()
        .map(|v| (v / s.spacing_meters).ceil() as usize);
    let started = std::time::Instant::now();
    let (geometry, mut report) = dual::extract(field, lo, cells, s.spacing_meters)?;
    report["generation_seconds"] = serde_json::json!(started.elapsed().as_secs_f64());
    report["fracture_cells"] = serde_json::json!(blocks.len());
    report["settings"] = serde_json::to_value(s).map_err(|e| invalid(e.to_string()))?;
    report["model"] = serde_json::json!("union_of_oblique_bevelled_strata_with_connected_core");
    report["streaming"] = serde_json::json!(false);
    Ok((Arc::new(geometry), report))
}

pub fn publication(
    p: Publication,
) -> Result<(
    crate::meshes::Asset,
    crate::meshes::Instance,
    serde_json::Value,
)> {
    let (geometry, report) = compile(&p.settings)?;
    let asset = crate::meshes::Asset {
        id: p.asset.clone(),
        geometry,
        shading: crate::meshes::Shading::Flat,
        material: p.material,
        rig: None,
        clips: vec![],
    };
    let instance = crate::meshes::Instance {
        id: p.instance,
        asset: p.asset,
        position: p.position,
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: p.color,
        playback: None,
        attachment: None,
    };
    Ok((asset, instance, report))
}
