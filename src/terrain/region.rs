//! Bounded regional publication, sharing indices/normals across the entire finite surface.
use super::*;
use std::sync::Arc;
pub const MAX_VERTICES: usize = 1_048_576;
pub const MAX_REGION_WORK: usize = 64_000_000;
pub const MAX_AXIS_POINTS: usize = 8192;

fn offsets<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<f64>, D::Error> {
    limited::<D, f64, MAX_AXIS_POINTS>(d)
}

/// Optional explicit grid coordinates, measured from the region origin.
/// They replace uniform spacing; all cells still share indexed vertices.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Axes {
    #[serde(deserialize_with = "offsets")]
    pub x: Vec<f64>,
    #[serde(deserialize_with = "offsets")]
    pub z: Vec<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Region {
    #[serde(with = "position_codec")]
    #[schemars(with = "Position")]
    pub origin: Position,
    pub cells: [u16; 2],
    pub spacing_meters: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis_offsets_meters: Option<Axes>,
}
impl Region {
    fn offset(&self, axis: usize, i: u16) -> f64 {
        match &self.axis_offsets_meters {
            Some(a) => [&a.x, &a.z][axis][usize::from(i)],
            None => f64::from(i) * self.spacing_meters,
        }
    }

    fn count(&self, doc: &Document) -> Result<usize> {
        let n = (usize::from(self.cells[0]) + 1)
            .checked_mul(usize::from(self.cells[1]) + 1)
            .ok_or_else(|| invalid("Regional vertex count overflow"))?;
        if self.cells.contains(&0)
            || n > MAX_VERTICES
            || !self.spacing_meters.is_finite()
            || !(0.125..=4096.).contains(&self.spacing_meters)
            || doc.work_per_point().saturating_mul(n) > MAX_REGION_WORK
            || !same_datum(self.origin, doc.datum)
        {
            return Err(invalid(
                "Regional terrain exceeds cells/extent/vertex/work budget or vertical datum",
            ));
        }
        if let Some(a) = &self.axis_offsets_meters {
            for (axis, values) in [&a.x, &a.z].into_iter().enumerate() {
                if values.len() != usize::from(self.cells[axis]) + 1
                    || values.len() > MAX_AXIS_POINTS
                    || values.first() != Some(&0.)
                    || values
                        .iter()
                        .any(|v| !v.is_finite() || !(0. ..=10000.).contains(v))
                    || values
                        .windows(2)
                        .any(|v| !(0.125..=4096.).contains(&(v[1] - v[0])))
                {
                    return Err(invalid(
                        "Regional axes require cells+1 finite increasing offsets, starting at0, extent<=10000m and steps0.125..4096m",
                    ));
                }
            }
        } else if self
            .cells
            .iter()
            .any(|&c| f64::from(c) * self.spacing_meters > 10000.)
        {
            return Err(invalid("Regional uniform extent exceeds10000m"));
        }
        self.origin
            .translated([
                self.offset(0, self.cells[0]),
                0.,
                self.offset(1, self.cells[1]),
            ])
            .map_err(|e| invalid(e.to_string()))?;
        Ok(n)
    }
}
pub struct Compiled {
    pub geometry: Arc<crate::meshes::Geometry>,
    pub region: Region,
    pub source_sha256: String,
    pub source_id: String,
    pub source_revision: u32,
    pub bounds: [[f32; 3]; 2],
}
impl Compiled {
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({"source_id":self.source_id,"source_revision":self.source_revision,"source_sha256":self.source_sha256,"region":self.region,"bounds":self.bounds,"vertices":self.geometry.positions.len(),"triangles":self.geometry.triangles.len(),"shared_grid_vertices":true,"streaming":false})
    }
}
pub fn compile(doc: &Document, region: Region) -> Result<Compiled> {
    doc.validate()?;
    let n = region.count(doc)?;
    let sampler = Sampler::new(doc)?;
    let mut positions = Vec::with_capacity(n);
    let mut triangles =
        Vec::with_capacity(usize::from(region.cells[0]) * usize::from(region.cells[1]) * 2);
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for z in 0..=region.cells[1] {
        for x in 0..=region.cells[0] {
            let dx = region.offset(0, x);
            let dz = region.offset(1, z);
            let p = region
                .origin
                .translated([dx, 0., dz])
                .map_err(|e| invalid(e.to_string()))?;
            let h = sampler
                .sample(p)?
                .ok_or_else(|| invalid("Regional terrain extends outside fixed sampled source"))?
                as f32;
            lo = lo.min(h);
            hi = hi.max(h);
            positions.push([dx as f32, h, dz as f32]);
        }
    }
    let w = u32::from(region.cells[0]) + 1;
    for z in 0..u32::from(region.cells[1]) {
        for x in 0..u32::from(region.cells[0]) {
            let a = z * w + x;
            triangles.extend([[a, a + w, a + 1], [a + 1, a + w, a + w + 1]]);
        }
    }
    let geometry = crate::meshes::Geometry {
        positions,
        triangles,
        material_coordinates: None,
    };
    geometry.validate()?;
    let bounds = [
        [0., lo, 0.],
        [
            region.offset(0, region.cells[0]) as f32,
            hi,
            region.offset(1, region.cells[1]) as f32,
        ],
    ];
    Ok(Compiled {
        geometry: Arc::new(geometry),
        region,
        source_sha256: source_digest(doc)?,
        source_id: doc.id.clone(),
        source_revision: doc.revision,
        bounds,
    })
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub document: Document,
    pub region: Region,
    pub frame_origin: Position,
    pub asset: String,
    pub instance: String,
    pub color: [f32; 3],
    #[serde(default)]
    pub material: crate::materials::Material,
    #[serde(default)]
    pub collision: Option<bool>,
    #[serde(default)]
    pub world_material_coordinates: bool,
}
pub fn publication(
    p: Publication,
) -> Result<(crate::meshes::Asset, crate::meshes::Instance, Compiled)> {
    // Scene bounds are a box, not a sphere. Project into a circumscribing
    // sphere first, then enforce the existing per-component scene envelope.
    let local = p
        .region
        .origin
        .project(p.frame_origin, 20000., 0.001)
        .map_err(|e| invalid(e.to_string()))?;
    if local.iter().any(|v| v.abs() > 10000.) {
        return Err(invalid(
            "Regional origin exceeds the scene envelope +/-10000m",
        ));
    }
    let mut compiled = compile(&p.document, p.region)?;
    if p.world_material_coordinates {
        let coordinates = compiled
            .geometry
            .positions
            .iter()
            .map(|v| [v[0] + local[0], v[1] + local[1], v[2] + local[2]])
            .collect();
        Arc::make_mut(&mut compiled.geometry).material_coordinates = Some(coordinates);
        compiled.geometry.validate()?;
    }
    let asset = crate::meshes::Asset {
        id: p.asset.clone(),
        geometry: compiled.geometry.clone(),
        shading: crate::meshes::Shading::Smooth,
        material: p.material,
        rig: None,
        clips: vec![],
    };
    let instance = crate::meshes::Instance {
        id: p.instance,
        asset: p.asset,
        position: local,
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: p.color,
        playback: None,
        attachment: None,
    };
    Ok((asset, instance, compiled))
}
