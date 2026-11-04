//! Cold, source-authored deformation of connected landform surfaces.
use super::*;
mod cuts;
pub use cuts::CreasePlane;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Falloff {
    #[default]
    Linear,
    Smooth,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    pub center: [f64; 3],
    pub axis: [f64; 3],
    pub profile_x: [f64; 3],
    /// Half extents along profile_x, axis×profile_x, and axis, in metres.
    pub half_extents: [f64; 3],
    /// Displacement in the document's local XYZ, not in the stamp frame.
    pub displacement: [f64; 3],
    #[serde(default)]
    pub flat_core: f64,
    #[serde(default)]
    pub falloff: Falloff,
}
fn stamps<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Stamp>, D::Error> {
    crate::terrain::limited::<D, Stamp, 32>(d)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shaping {
    pub subdivisions: u8,
    /// Conforming tessellation planes, in the unmodified document frame.
    #[serde(
        default,
        deserialize_with = "cuts::planes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub creases: Vec<CreasePlane>,
    #[serde(deserialize_with = "stamps")]
    pub stamps: Vec<Stamp>,
}
pub(super) struct PreparedStamp {
    center: DVec3,
    axes: [DVec3; 3],
    inverse_extent: DVec3,
    displacement: DVec3,
    core: f64,
    falloff: Falloff,
}
impl PreparedStamp {
    fn displacement(&self, p: DVec3) -> DVec3 {
        let p = p - self.center;
        let local = DVec3::new(
            p.dot(self.axes[0]),
            p.dot(self.axes[1]),
            p.dot(self.axes[2]),
        );
        let q = (local * self.inverse_extent).abs().max_element();
        let mut w = ((1. - q) / (1. - self.core)).clamp(0., 1.);
        if self.falloff == Falloff::Smooth {
            w = w * w * (3. - 2. * w);
        }
        self.displacement * w
    }
}
impl Shaping {
    pub(super) fn prepare(&self) -> Result<Vec<PreparedStamp>> {
        if self.subdivisions > 3 || self.stamps.len() > 32 || self.creases.len() > 24 {
            return Err(invalid(
                "Shaping needs0..3 subdivisions, at most32 stamps and24 crease planes",
            ));
        }
        for plane in &self.creases {
            plane.validate()?;
        }
        self.stamps
            .iter()
            .map(|s| {
                if !finite(&s.center, 10000.)
                    || !finite(&s.axis, 10000.)
                    || !finite(&s.profile_x, 10000.)
                    || !finite(&s.displacement, 10000.)
                    || !s
                        .half_extents
                        .iter()
                        .all(|v| v.is_finite() && (0.0001..=10000.).contains(v))
                    || !s.flat_core.is_finite()
                    || !(0. ..=0.95).contains(&s.flat_core)
                {
                    return Err(invalid(
                        "Invalid shaping stamp coordinates, extents or core0..0.95",
                    ));
                }
                let axis = DVec3::from_array(s.axis)
                    .try_normalize()
                    .ok_or_else(|| invalid("Shaping axis must be nonzero"))?;
                let x = DVec3::from_array(s.profile_x);
                let x = (x - axis * x.dot(axis))
                    .try_normalize()
                    .ok_or_else(|| invalid("Shaping profile_x must not be parallel to axis"))?;
                Ok(PreparedStamp {
                    center: DVec3::from_array(s.center),
                    axes: [x, axis.cross(x), axis],
                    inverse_extent: DVec3::from_array(s.half_extents).recip(),
                    displacement: DVec3::from_array(s.displacement),
                    core: s.flat_core,
                    falloff: s.falloff,
                })
            })
            .collect()
    }
    pub(super) fn counts(&self, vertices: usize, triangles: usize) -> Result<(usize, usize)> {
        if self.subdivisions > 3 {
            return Err(invalid("Shaping subdivision budget exceeded"));
        }
        let refined = triangles * 4usize.pow(u32::from(self.subdivisions));
        // Closed triangular surfaces have E=3T/2; shared edges add one midpoint.
        let vertices = vertices + (refined - triangles) / 2;
        if vertices > meshes::MAX_VERTICES || refined > meshes::MAX_TRIANGLES {
            return Err(invalid(
                "Shaping exceeds native mesh budget; refine a smaller artifact",
            ));
        }
        Ok((vertices, refined))
    }
}

pub(super) fn compile(document: &Document) -> Result<Compiled> {
    let counts = document.prepare()?; // Refined output/work bounds before cloning/generating.
    let frame = domain::Frame::new(document.origin, document.domain.as_ref())?;
    let mut base_source = document.clone();
    base_source.domain = None;
    for p in &mut base_source.pieces {
        p.shaping = None;
    }
    let base = super::compile(&base_source)?;
    let mut geometry = Geometry {
        positions: Vec::new(),
        triangles: Vec::new(),
        material_coordinates: None,
    };
    geometry
        .positions
        .reserve(counts.iter().map(|p| p.vertices).sum());
    geometry
        .triangles
        .reserve(counts.iter().map(|p| p.triangles).sum());
    let mut parts = Vec::with_capacity(base.parts.len());
    let mut rounding = base.maximum_vertex_rounding_meters;
    let mut sampled_error = frame.spherical().then_some(0f64);
    for (source, part) in document.pieces.iter().zip(&base.parts) {
        let first = part.vertices[0] as u32;
        let mut g = Geometry {
            positions: base.geometry.positions[part.vertices[0]..part.vertices[1]].to_vec(),
            triangles: base.geometry.triangles[part.triangles[0]..part.triangles[1]]
                .iter()
                .map(|t| t.map(|i| i - first))
                .collect(),
            material_coordinates: None,
        };
        if let Some(shape) = &source.shaping {
            for _ in 0..shape.subdivisions {
                let plan = crate::mesh_topology::Plan::new(&g, None)?;
                for &[a, b] in plan.split_edges() {
                    let a = DVec3::from_array(g.positions[a as usize].map(f64::from));
                    let b = DVec3::from_array(g.positions[b as usize].map(f64::from));
                    let value = (a + b) * 0.5;
                    rounding = rounding.max(value.distance(value.as_vec3().as_dvec3()));
                }
                g = plan.geometry(&g)?;
            }
            rounding = rounding.max(cuts::apply(&mut g, &shape.creases)?);
            let stamps = shape.prepare()?;
            let before = g.positions.clone();
            for p in &mut g.positions {
                let frozen = DVec3::from_array(p.map(f64::from));
                let value = frozen + stamps.iter().map(|s| s.displacement(frozen)).sum::<DVec3>();
                let rounded = value.as_vec3();
                rounding = rounding.max(value.distance(rounded.as_dvec3()));
                *p = rounded.to_array();
            }
            crate::mesh_sculpt_shapes::check_orientation(&g.triangles, &before, &g.positions)?;
            g.validate()?;
        }
        let vertex_start = geometry.positions.len();
        let triangle_start = geometry.triangles.len();
        if vertex_start + g.positions.len() > meshes::MAX_VERTICES
            || triangle_start + g.triangles.len() > meshes::MAX_TRIANGLES
        {
            return Err(invalid("Combined shaped pieces exceed native mesh budget"));
        }
        for p in &g.positions {
            let mapped = frame.project(DVec3::from_array(p.map(f64::from)))?;
            let rounded = mapped.as_vec3();
            rounding = rounding.max(mapped.distance(rounded.as_dvec3()));
            geometry.positions.push(rounded.to_array());
        }
        for t in &g.triangles {
            if frame.spherical() {
                let [a, b, c] =
                    t.map(|i| DVec3::from_array(g.positions[i as usize].map(f64::from)));
                let [wa, wb, wc] = t.map(|i| {
                    DVec3::from_array(geometry.positions[vertex_start + i as usize].map(f64::from))
                });
                if (wb - wa)
                    .cross(wc - wa)
                    .dot(frame.normal((a + b + c) / 3., (b - a).cross(c - a)))
                    <= 0.
                {
                    return Err(invalid(
                        "Radial shaped surface folds; refine or reduce deformation",
                    ));
                }
                for (p, q) in [
                    ((a + b) / 2., (wa + wb) / 2.),
                    ((b + c) / 2., (wb + wc) / 2.),
                    ((c + a) / 2., (wc + wa) / 2.),
                    ((a + b + c) / 3., (wa + wb + wc) / 3.),
                ] {
                    let error = frame.project(p)?.distance(q);
                    let maximum = sampled_error.as_mut().unwrap();
                    *maximum = maximum.max(error);
                }
            }
            geometry.triangles.push(t.map(|i| i + vertex_start as u32));
        }
        let range = [triangle_start, geometry.triangles.len()];
        let pivot = DVec3::from_array(geometry.positions[vertex_start].map(f64::from));
        let volume = super::closed_volume(&geometry, range, pivot)?;
        parts.push(Part {
            id: part.id.clone(),
            vertices: [vertex_start, geometry.positions.len()],
            triangles: range,
            signed_volume_cubic_meters: volume,
            closed_oriented: true,
            inner_void: part.inner_void,
            terminal_apex: part.terminal_apex,
        });
    }
    geometry.validate()?;
    Ok(Compiled {
        frame,
        version: document.version,
        source_id: document.id.clone(),
        source_revision: document.revision,
        source_sha256: crate::terrain::source_digest(document)?,
        origin: document.origin,
        geometry: Arc::new(geometry),
        parts,
        maximum_vertex_rounding_meters: rounding,
        maximum_sampled_domain_error_meters: sampled_error,
        boolean_union: false,
        visual_only: true,
    })
}
