//! Cold cubed-sphere publication. Integer cube addresses weld all globe seams/corners.
use super::*;
use crate::meshes::{Asset, Geometry, Instance, Shading};
use std::{collections::BTreeMap, sync::Arc};
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Face {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}
impl Face {
    pub(super) fn key(self, u: i64, v: i64, n: i64) -> [i64; 3] {
        match self {
            Self::PositiveX => [n, v, -u],
            Self::NegativeX => [-n, v, u],
            Self::PositiveY => [u, n, -v],
            Self::NegativeY => [u, -n, v],
            Self::PositiveZ => [u, v, n],
            Self::NegativeZ => [-u, v, -n],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Area {
    Globe {
        subdivisions: u16,
    },
    Tile {
        face: Face,
        level: u8,
        tile: [u32; 2],
        subdivisions: u16,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Projection {
    Centered,
    Surface { pose: Pose },
}
pub struct Compiled {
    pub geometry: Arc<Geometry>,
    pub report: Value,
}
impl Projection {
    fn project(&self, d: DVec3, h: f64, r: f64, basis: Option<Frame>) -> [f64; 3] {
        match self {
            Self::Centered => (d * (r + h)).to_array(),
            Self::Surface { pose } => {
                let f = basis.expect("validated surface projection");
                let up = DVec3::from(f.up);
                let delta = d - up;
                let right = DVec3::from(f.right);
                let back = -DVec3::from(f.forward);
                // Stable sagitta avoids subtracting two planet radii near the ground.
                [
                    r * delta.dot(right) + h * d.dot(right),
                    -0.5 * r * delta.length_squared() + h * d.dot(up) - pose.altitude_meters,
                    r * delta.dot(back) + h * d.dot(back),
                ]
            }
        }
    }
}
pub fn compile(document: &Document, area: &Area, projection: &Projection) -> Result<Compiled> {
    let (n, faces, level, tile) = match *area {
        Area::Globe { subdivisions } => (subdivisions, 6, 0, [0; 2]),
        Area::Tile {
            level,
            tile,
            subdivisions,
            ..
        } => (subdivisions, 1, level, tile),
    };
    if n == 0
        || n > 128
        || !n.is_power_of_two()
        || level > 24
        || tile.iter().any(|&v| u64::from(v) >= (1u64 << level))
    {
        return Err(invalid(
            "Planet mesh needs power-of-two subdivisions1..128, level0..24 and tile inside2^level",
        ));
    }
    let count = if faces == 6 {
        6 * usize::from(n).pow(2) + 2
    } else {
        (usize::from(n) + 1).pow(2)
    };
    document.budget(count)?;
    let sampler = Sampler::new(document)?;
    let basis = match projection {
        Projection::Centered => None,
        Projection::Surface { pose } => {
            if pose.altitude_meters.abs() > document.radius() * 0.25 {
                return Err(invalid(
                    "Surface frame altitude must be within25% of planet radius",
                ));
            }
            Some(pose.frame()?)
        }
    };
    let origin = match (projection, basis) {
        (Projection::Surface { pose }, Some(f)) => document
            .center
            .translated((DVec3::from(f.up) * (document.radius() + pose.altitude_meters)).to_array())
            .map_err(|e| invalid(e.to_string()))?,
        _ => document.center,
    };
    let mut positions = Vec::with_capacity(count);
    let mut triangles = Vec::with_capacity(faces * 2 * usize::from(n).pow(2));
    let mut welded = BTreeMap::new();
    let mut grid = Vec::with_capacity((usize::from(n) + 1).pow(2));
    let mut min_height = f64::INFINITY;
    let mut max_height = f64::NEG_INFINITY;
    let mut max_error = 0f64;
    let all = [
        Face::PositiveX,
        Face::NegativeX,
        Face::PositiveY,
        Face::NegativeY,
        Face::PositiveZ,
        Face::NegativeZ,
    ];
    let selected = match *area {
        Area::Tile { face, .. } => [face; 6],
        _ => all,
    };
    let denom = i64::from(n) << level;
    for &face in selected.iter().take(faces) {
        grid.clear();
        for y in 0..=n {
            for x in 0..=n {
                let u = 2 * (i64::from(tile[0]) * i64::from(n) + i64::from(x)) - denom;
                let v = 2 * (i64::from(tile[1]) * i64::from(n) + i64::from(y)) - denom;
                let key = face.key(u, v, denom);
                let index = if let Some(&i) = welded.get(&key) {
                    i
                } else {
                    let d = DVec3::new(key[0] as f64, key[1] as f64, key[2] as f64).normalize();
                    let h = sampler.height(d);
                    min_height = min_height.min(h);
                    max_height = max_height.max(h);
                    let p = projection.project(d, h, document.radius(), basis);
                    let pf = p.map(|x| x as f32);
                    if p.iter().any(|x| !x.is_finite() || x.abs() > 10000.) {
                        return Err(invalid(
                            "Planet publication exceeds resident +/-10000m; use a smaller tile and a local surface frame",
                        ));
                    }
                    for (a, b) in p.into_iter().zip(pf) {
                        max_error = max_error.max((a - f64::from(b)).abs());
                    }
                    if max_error > 0.001 {
                        return Err(invalid(
                            "Planet local projection exceeds1mm f32 error; use a closer local surface frame",
                        ));
                    }
                    let i = positions.len() as u32;
                    positions.push(pf);
                    welded.insert(key, i);
                    i
                };
                grid.push(index);
            }
        }
        let w = usize::from(n) + 1;
        for y in 0..usize::from(n) {
            for x in 0..usize::from(n) {
                let a = y * w + x;
                triangles.extend([
                    [grid[a], grid[a + 1], grid[a + w]],
                    [grid[a + 1], grid[a + w + 1], grid[a + w]],
                ]);
            }
        }
    }
    let geometry = Arc::new(Geometry {
        positions,
        triangles,
        material_coordinates: None,
    });
    // Reject collapsed triangles in the f32 resident projection.
    geometry.validate()?;
    let report = json!({"planet_format":VERSION,"source_id":document.id,"source_revision":document.revision,"source_sha256":crate::terrain::source_digest(document)?,
        "diameter_meters":document.diameter_meters,"area":area,"frame":projection,"global_origin":origin,"local_up":basis.map(|_| [0,1,0]),"vertices":geometry.positions.len(),"triangles":geometry.triangles.len(),
        "sampled_height_range_meters":[min_height,max_height],"maximum_projection_error_meters":max_error,"whole_globe_welded":faces==6,"approximation_error_certified":false,
        "source_embedded_in_scene":false,"world_rebased":false,"radial_gravity_applied":false,"streaming":false});
    Ok(Compiled { geometry, report })
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub document: Document,
    pub area: Area,
    pub frame: Projection,
    pub asset: String,
    pub instance: String,
    pub color: [f32; 3],
    #[serde(default)]
    pub material: crate::materials::Material,
    /// Explicit static collision only. Does not change gravity or the character controller.
    #[serde(default)]
    pub collision: Option<bool>,
}
pub fn publication(p: Publication) -> Result<(Asset, Instance, Compiled)> {
    let compiled = compile(&p.document, &p.area, &p.frame)?;
    let asset = Asset {
        id: p.asset.clone(),
        geometry: compiled.geometry.clone(),
        shading: Shading::Smooth,
        material: p.material,
        rig: None,
        clips: vec![],
    };
    let instance = Instance {
        id: p.instance,
        asset: p.asset,
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: p.color,
        playback: None,
        attachment: None,
    };
    Ok((asset, instance, compiled))
}
