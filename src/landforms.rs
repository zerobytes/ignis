//! Local 3D landform tools: editable concave profiles, homothetic sections and voids.
//! Geological presets produce ordinary data. No physics, erosion or CSG is implied.
use crate::{
    meshes::{self, Geometry},
    model::{Result, invalid},
    world_space::Position,
};
use glam::{DVec2, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod presets;
mod triangulate;
pub use presets::Preset;

pub const VERSION: u16 = 1;
pub const MAX_PIECES: usize = 32;
pub const MAX_WORK: usize = 4_000_000;
pub const MAX_RAY_HITS: usize = 512;
pub const MAX_QUERY_HITS: usize = 4096;
const MAX_RAW_HITS: usize = 4096;
fn pieces<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Piece>, D::Error> {
    crate::terrain::limited::<D, Piece, MAX_PIECES>(d)
}
fn profile<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<[f64; 2]>, D::Error> {
    crate::terrain::limited::<D, [f64; 2], 96>(d)
}
fn sections<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Section>, D::Error> {
    crate::terrain::limited::<D, Section, 128>(d)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub at: f64,
    pub offset: [f64; 2],
    pub scale: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub id: String,
    pub anchor: [f64; 3],
    pub axis: [f64; 3],
    pub profile_x: [f64; 3],
    #[serde(deserialize_with = "profile")]
    pub profile: Vec<[f64; 2]>,
    #[serde(deserialize_with = "sections")]
    pub sections: Vec<Section>,
    /// A smaller homothetic contour creates an actual longitudinal opening.
    pub inner_scale: Option<f64>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u16,
    pub space_version: u16,
    pub id: String,
    pub revision: u32,
    #[serde(with = "crate::terrain::position_codec")]
    #[schemars(with = "Position")]
    pub origin: Position,
    #[serde(deserialize_with = "pieces")]
    pub pieces: Vec<Piece>,
}
struct Prepared {
    profile: triangulate::Profile,
    axis: DVec3,
    x: DVec3,
    y: DVec3,
    vertices: usize,
    triangles: usize,
}
fn finite(v: &[f64], bound: f64) -> bool {
    v.iter().all(|n| n.is_finite() && n.abs() <= bound)
}
impl Piece {
    fn prepare(&self) -> Result<Prepared> {
        if !crate::terrain::id_valid(&self.id)
            || !finite(&self.anchor, 10000.)
            || !finite(&self.axis, 10000.)
            || !finite(&self.profile_x, 10000.)
            || !(2..=128).contains(&self.sections.len())
        {
            return Err(invalid(
                "Invalid landform piece ID, frame or section count2..128",
            ));
        }
        let axis = DVec3::from_array(self.axis)
            .try_normalize()
            .ok_or_else(|| invalid("Landform axis must be nonzero"))?;
        let x = DVec3::from_array(self.profile_x);
        let x = (x - axis * x.dot(axis))
            .try_normalize()
            .ok_or_else(|| invalid("Landform profile_x must not be parallel to axis"))?;
        let y = axis.cross(x);
        let profile = triangulate::prepare(&self.profile)?;
        let mut apex = false;
        for (i, s) in self.sections.iter().enumerate() {
            if !s.at.is_finite()
                || s.at.abs() > 10000.
                || !finite(&s.offset, 10000.)
                || !s.scale.is_finite()
                || !(0. ..=1000.).contains(&s.scale)
            {
                return Err(invalid("Invalid landform section coordinates or scale"));
            }
            if s.scale == 0. {
                if i + 1 != self.sections.len() || i == 0 || self.inner_scale.is_some() {
                    return Err(invalid(
                        "Only a solid terminal section may collapse to an apex",
                    ));
                }
                apex = true;
            }
            if i > 0 {
                let previous = &self.sections[i - 1];
                let delta = s.at - previous.at;
                if delta < 0. || (delta > 0. && delta < 0.0001) {
                    return Err(invalid(
                        "Landform section planes must increase, with at least0.1mm separation",
                    ));
                }
                if delta == 0.
                    && (!profile.star_shaped
                        || s.offset != previous.offset
                        || s.scale == previous.scale
                        || s.scale == 0.
                        || i == 1
                        || i + 1 == self.sections.len()
                        || (i > 1 && self.sections[i - 2].at == s.at))
                {
                    return Err(invalid(
                        "A horizontal ledge needs two interior nested star-shaped sections at the same center; no coplanar endcaps or repeated steps",
                    ));
                }
            }
        }
        if self.sections.last().unwrap().at - self.sections[0].at < 0.0001 {
            return Err(invalid("Landform needs nonzero longitudinal extent"));
        }
        if let Some(inner) = self.inner_scale {
            let min_scale = self
                .sections
                .iter()
                .map(|s| s.scale)
                .fold(f64::INFINITY, f64::min);
            if !inner.is_finite()
                || !(0.001..=0.999).contains(&inner)
                || !profile.star_shaped
                || profile.kernel_radius * min_scale * (1. - inner) < 0.0001
            {
                return Err(invalid(
                    "Landform cavity needs a star-shaped profile and nonzero wall thickness",
                ));
            }
        }
        let n = self.profile.len();
        let rows = self.sections.len();
        let (vertices, triangles) = if self.inner_scale.is_some() {
            (2 * n * rows, 4 * n * rows)
        } else {
            (
                n * (rows - usize::from(apex)) + usize::from(apex),
                2 * n * (rows - 1) - n * usize::from(apex) + (2 - usize::from(apex)) * (n - 2),
            )
        };
        Ok(Prepared {
            profile,
            axis,
            x,
            y,
            vertices,
            triangles,
        })
    }
}
impl Document {
    fn prepare(&self) -> Result<Vec<Prepared>> {
        if self.version != VERSION
            || self.space_version != crate::world_space::VERSION
            || !crate::terrain::id_valid(&self.id)
            || self.revision == 0
            || !(1..=MAX_PIECES).contains(&self.pieces.len())
        {
            return Err(invalid(
                "Invalid landform version, space, ID, revision or piece count1..32",
            ));
        }
        let mut ids = BTreeSet::new();
        let mut work = 0usize;
        for p in &self.pieces {
            if !(3..=96).contains(&p.profile.len())
                || !(2..=128).contains(&p.sections.len())
                || !ids.insert(&p.id)
            {
                return Err(invalid(
                    "Landform contour/section budget or duplicate piece ID",
                ));
            }
            work = work
                .saturating_add(p.profile.len().pow(3) + p.profile.len() * p.sections.len() * 8);
        }
        if work > MAX_WORK {
            return Err(invalid(
                "Landform compilation exceeds preflight work budget",
            ));
        }
        let mut out = Vec::with_capacity(self.pieces.len());
        let mut vertices = 0;
        let mut triangles = 0;
        for p in &self.pieces {
            let prepared = p.prepare()?;
            vertices += prepared.vertices;
            triangles += prepared.triangles;
            if vertices > meshes::MAX_VERTICES || triangles > meshes::MAX_TRIANGLES {
                return Err(invalid(
                    "Landform exceeds native mesh budget; split the document",
                ));
            }
            out.push(prepared);
        }
        Ok(out)
    }
    pub fn validate(&self) -> Result<()> {
        self.prepare().map(|_| ())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Part {
    pub id: String,
    pub vertices: [usize; 2],
    pub triangles: [usize; 2],
    pub signed_volume_cubic_meters: f64,
    pub closed_oriented: bool,
    pub inner_void: bool,
    pub terminal_apex: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Compiled {
    pub version: u16,
    pub source_id: String,
    pub source_revision: u32,
    pub source_sha256: String,
    pub origin: Position,
    pub geometry: Arc<Geometry>,
    pub parts: Vec<Part>,
    pub maximum_vertex_rounding_meters: f64,
    pub boolean_union: bool,
    pub visual_only: bool,
}
fn closed_volume(geometry: &Geometry, range: [usize; 2], pivot: DVec3) -> Result<f64> {
    let mut edges = BTreeMap::<[u32; 2], (u8, i8)>::new();
    let mut volume = 0.;
    for &t in &geometry.triangles[range[0]..range[1]] {
        let [a, b, c] =
            t.map(|i| DVec3::from_array(geometry.positions[i as usize].map(f64::from)) - pivot);
        volume += a.dot(b.cross(c)) / 6.;
        for i in 0..3 {
            let a = t[i];
            let b = t[(i + 1) % 3];
            let key = [a.min(b), a.max(b)];
            let entry = edges.entry(key).or_default();
            entry.0 += 1;
            entry.1 += if a < b { 1 } else { -1 };
            if entry.0 > 2 {
                return Err(invalid("Landform has a non-manifold edge"));
            }
        }
    }
    if edges.values().any(|&(n, s)| n != 2 || s != 0) || !volume.is_finite() || volume <= 1e-10 {
        return Err(invalid(
            "Landform is open, inverted or has negligible volume",
        ));
    }
    Ok(volume)
}
pub fn compile(document: &Document) -> Result<Compiled> {
    let prepared = document.prepare()?;
    let mut geometry = Geometry {
        positions: Vec::with_capacity(prepared.iter().map(|p| p.vertices).sum()),
        triangles: Vec::with_capacity(prepared.iter().map(|p| p.triangles).sum()),
        material_coordinates: None,
    };
    let mut parts = Vec::with_capacity(prepared.len());
    let mut rounding = 0f64;
    for (piece, p) in document.pieces.iter().zip(prepared) {
        let first_vertex = geometry.positions.len();
        let first_triangle = geometry.triangles.len();
        let n = piece.profile.len();
        let hollow = piece.inner_scale.is_some();
        let apex = piece.sections.last().unwrap().scale == 0.;
        let rows = piece.sections.len() - usize::from(apex);
        for side in 0..if hollow { 2 } else { 1 } {
            let factor = if side == 0 {
                1.
            } else {
                piece.inner_scale.unwrap()
            };
            for s in piece.sections.iter().take(rows) {
                for point in &piece.profile {
                    let local = DVec2::from_array(*point) * (s.scale * factor)
                        + DVec2::from_array(s.offset);
                    let v = DVec3::from_array(piece.anchor)
                        + p.axis * s.at
                        + p.x * local.x
                        + p.y * local.y;
                    let rounded = v.as_vec3();
                    rounding = rounding.max(v.distance(rounded.as_dvec3()));
                    geometry.positions.push(rounded.to_array());
                }
            }
        }
        let base = first_vertex as u32;
        let n32 = n as u32;
        for side in 0..if hollow { 2 } else { 1 } {
            let start = base + (side * rows * n) as u32;
            for row in 0..rows - 1 {
                for j in 0..n {
                    let a = start + (row * n + j) as u32;
                    let b = start + (row * n + (j + 1) % n) as u32;
                    let c = a + n32;
                    let d = b + n32;
                    let mut pair = [[a, b, c], [b, d, c]];
                    if side == 1 {
                        for t in &mut pair {
                            t.swap(1, 2);
                        }
                    }
                    geometry.triangles.extend(pair);
                }
            }
        }
        if apex {
            let s = piece.sections.last().unwrap();
            let v = DVec3::from_array(piece.anchor)
                + p.axis * s.at
                + p.x * s.offset[0]
                + p.y * s.offset[1];
            let rounded = v.as_vec3();
            rounding = rounding.max(v.distance(rounded.as_dvec3()));
            let tip = geometry.positions.len() as u32;
            geometry.positions.push(rounded.to_array());
            let last = base + ((rows - 1) * n) as u32;
            for j in 0..n {
                geometry
                    .triangles
                    .push([last + j as u32, last + ((j + 1) % n) as u32, tip]);
            }
        }
        if hollow {
            for end in 0..2 {
                let row = if end == 0 { 0 } else { rows - 1 };
                let outer = base + (row * n) as u32;
                let inner = base + ((rows + row) * n) as u32;
                for j in 0..n {
                    let next = (j + 1) % n;
                    let mut pair = [
                        [outer + j as u32, inner + j as u32, outer + next as u32],
                        [outer + next as u32, inner + j as u32, inner + next as u32],
                    ];
                    if end == 1 {
                        for t in &mut pair {
                            t.swap(1, 2);
                        }
                    }
                    geometry.triangles.extend(pair);
                }
            }
        } else {
            for &[a, b, c] in &p.profile.caps {
                geometry
                    .triangles
                    .push([base + a as u32, base + c as u32, base + b as u32]);
            }
            if !apex {
                let end = base + ((rows - 1) * n) as u32;
                for &[a, b, c] in &p.profile.caps {
                    geometry
                        .triangles
                        .push([end + a as u32, end + b as u32, end + c as u32]);
                }
            }
        }
        let triangles = [first_triangle, geometry.triangles.len()];
        let pivot = DVec3::from_array(geometry.positions[first_vertex].map(f64::from));
        let volume = closed_volume(&geometry, triangles, pivot)?;
        parts.push(Part {
            id: piece.id.clone(),
            vertices: [first_vertex, geometry.positions.len()],
            triangles,
            signed_volume_cubic_meters: volume,
            closed_oriented: true,
            inner_void: hollow,
            terminal_apex: apex,
        });
    }
    geometry.validate()?;
    Ok(Compiled {
        version: VERSION,
        source_id: document.id.clone(),
        source_revision: document.revision,
        source_sha256: crate::terrain::source_digest(document)?,
        origin: document.origin,
        geometry: Arc::new(geometry),
        parts,
        maximum_vertex_rounding_meters: rounding,
        boolean_union: false,
        visual_only: true,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    Append {
        piece: Piece,
    },
    Replace {
        id: String,
        piece: Piece,
    },
    Remove {
        id: String,
    },
    SetProfile {
        id: String,
        #[serde(deserialize_with = "profile")]
        profile: Vec<[f64; 2]>,
    },
    SetSections {
        id: String,
        #[serde(deserialize_with = "sections")]
        sections: Vec<Section>,
    },
    SetInnerScale {
        id: String,
        inner_scale: Option<f64>,
    },
}
fn edit_list<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Edit>, D::Error> {
    crate::terrain::limited::<D, Edit, 32>(d)
}
pub fn edit(mut document: Document, expected_revision: u32, edits: Vec<Edit>) -> Result<Document> {
    document.validate()?;
    if document.revision != expected_revision || !(1..=32).contains(&edits.len()) {
        return Err(invalid("Landform revision conflict or edit budget"));
    }
    let revision = document
        .revision
        .checked_add(1)
        .ok_or_else(|| invalid("Landform revision exhausted"))?;
    for edit in edits {
        if let Edit::Append { piece } = edit {
            if document.pieces.len() == MAX_PIECES {
                return Err(invalid("Landform piece budget exhausted"));
            }
            document.pieces.push(piece);
            continue;
        }
        let id = match &edit {
            Edit::Replace { id, .. }
            | Edit::Remove { id }
            | Edit::SetProfile { id, .. }
            | Edit::SetSections { id, .. }
            | Edit::SetInnerScale { id, .. } => id,
            Edit::Append { .. } => unreachable!(),
        };
        let index = document
            .pieces
            .iter()
            .position(|p| &p.id == id)
            .ok_or_else(|| invalid("Missing landform piece"))?;
        match edit {
            Edit::Replace { piece, .. } => document.pieces[index] = piece,
            Edit::Remove { .. } => {
                document.pieces.remove(index);
            }
            Edit::SetProfile { profile, .. } => document.pieces[index].profile = profile,
            Edit::SetSections { sections, .. } => document.pieces[index].sections = sections,
            Edit::SetInnerScale { inner_scale, .. } => {
                document.pieces[index].inner_scale = inner_scale
            }
            Edit::Append { .. } => unreachable!(),
        }
    }
    document.validate()?;
    document.revision = revision;
    Ok(document)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Ray {
    pub origin: [f64; 3],
    pub direction: [f64; 3],
    pub max_distance_meters: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub distance_meters: f64,
    pub point: [f64; 3],
    pub normal: [f64; 3],
    pub piece: String,
    pub triangle: u32,
}
impl Compiled {
    pub fn intersections(&self, ray: &Ray) -> Result<Vec<Hit>> {
        if !finite(&ray.origin, 100_000.)
            || !finite(&ray.direction, 10000.)
            || !ray.max_distance_meters.is_finite()
            || !(0.0001..=100_000.).contains(&ray.max_distance_meters)
        {
            return Err(invalid("Invalid landform ray"));
        }
        let o = DVec3::from_array(ray.origin);
        let d = DVec3::from_array(ray.direction)
            .try_normalize()
            .ok_or_else(|| invalid("Landform ray direction is zero"))?;
        struct RawHit {
            distance: f64,
            part: usize,
            triangle: usize,
        }
        let mut hits = Vec::with_capacity(32);
        for (part_index, part) in self.parts.iter().enumerate() {
            for (offset, t) in self.geometry.triangles[part.triangles[0]..part.triangles[1]]
                .iter()
                .enumerate()
            {
                let [a, b, c] = t
                    .map(|i| DVec3::from_array(self.geometry.positions[i as usize].map(f64::from)));
                let ab = b - a;
                let ac = c - a;
                let h = d.cross(ac);
                let det = ab.dot(h);
                if det.abs() < 1e-12 {
                    continue;
                }
                let inv = 1. / det;
                let s = o - a;
                let u = inv * s.dot(h);
                if !(-1e-10..=1. + 1e-10).contains(&u) {
                    continue;
                }
                let q = s.cross(ab);
                let v = inv * d.dot(q);
                if v < -1e-10 || u + v > 1. + 1e-10 {
                    continue;
                }
                let distance = inv * ac.dot(q);
                if distance < 0. || distance > ray.max_distance_meters {
                    continue;
                }
                if hits.len() == MAX_RAW_HITS {
                    return Err(invalid(
                        "Landform ray exceeds raw intersection budget; narrow the query",
                    ));
                }
                hits.push(RawHit {
                    distance,
                    part: part_index,
                    triangle: part.triangles[0] + offset,
                });
            }
        }
        // Group by part before merging coincident edge hits. Another overlapping
        // piece must not separate duplicates; composition still is not a union.
        hits.sort_by(|a, b| {
            a.part
                .cmp(&b.part)
                .then(a.distance.total_cmp(&b.distance))
                .then(a.triangle.cmp(&b.triangle))
        });
        hits.dedup_by(|a, b| a.part == b.part && (a.distance - b.distance).abs() < 1e-7);
        if hits.len() > MAX_RAY_HITS {
            return Err(invalid(
                "Landform ray exceeds intersection result budget; narrow the query",
            ));
        }
        hits.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(self.parts[a.part].id.cmp(&self.parts[b.part].id))
                .then(a.triangle.cmp(&b.triangle))
        });
        Ok(hits
            .into_iter()
            .map(|h| {
                let [a, b, c] = self.geometry.triangles[h.triangle]
                    .map(|i| DVec3::from_array(self.geometry.positions[i as usize].map(f64::from)));
                Hit {
                    distance_meters: h.distance,
                    point: (o + d * h.distance).to_array(),
                    normal: (b - a).cross(c - a).normalize().to_array(),
                    piece: self.parts[h.part].id.clone(),
                    triangle: h.triangle as u32,
                }
            })
            .collect())
    }
}
fn rays<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Ray>, D::Error> {
    crate::terrain::limited::<D, Ray, 128>(d)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    Preset {
        id: String,
        origin: Position,
        preset: Preset,
    },
    Inspect {
        document: Document,
    },
    Compile {
        document: Document,
    },
    Edit {
        document: Document,
        expected_revision: u32,
        #[serde(deserialize_with = "edit_list")]
        edits: Vec<Edit>,
    },
    Raycast {
        document: Document,
        #[serde(deserialize_with = "rays")]
        rays: Vec<Ray>,
    },
}
pub fn query(query: Query) -> Result<serde_json::Value> {
    match query {
        Query::Preset { id, origin, preset } => {
            Ok(serde_json::json!({"document":presets::create(id,origin,preset)?}))
        }
        Query::Inspect { document } => {
            document.validate()?;
            Ok(
                serde_json::json!({"document":document,"visual_only":true,"boolean_union":false,"limits":{"pieces":MAX_PIECES,"profile_points":96,"sections":128,"work":MAX_WORK,"ray_hits":MAX_RAY_HITS,"query_hits":MAX_QUERY_HITS}}),
            )
        }
        Query::Compile { document } => {
            serde_json::to_value(compile(&document)?).map_err(|e| invalid(e.to_string()))
        }
        Query::Edit {
            document,
            expected_revision,
            edits,
        } => Ok(serde_json::json!({"document":edit(document,expected_revision,edits)?})),
        Query::Raycast { document, rays } => {
            if rays.len() > 128 {
                return Err(invalid("Landform ray budget exceeded"));
            }
            let c = compile(&document)?;
            if rays.len().saturating_mul(c.geometry.triangles.len()) > MAX_WORK {
                return Err(invalid("Landform ray work budget exceeded"));
            }
            let mut hits = Vec::with_capacity(rays.len());
            let mut total = 0;
            for ray in &rays {
                let result = c.intersections(ray)?;
                total += result.len();
                if total > MAX_QUERY_HITS {
                    return Err(invalid(
                        "Landform batch exceeds intersection result budget; split the query",
                    ));
                }
                hits.push(result);
            }
            Ok(
                serde_json::json!({"hits":hits,"source_sha256":c.source_sha256,"visual_query_only":true,"boolean_union":false}),
            )
        }
    }
}
pub fn encode(document: &Document) -> Result<Vec<u8>> {
    document.validate()?;
    crate::terrain::encode_asset(b"IGNISLFM", VERSION, document)
}
pub fn decode(bytes: &[u8]) -> Result<Document> {
    let d: Document = crate::terrain::decode_asset(bytes, b"IGNISLFM", VERSION)?;
    d.validate()?;
    Ok(d)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    /// None preserves an existing binding; true/false explicitly enable/disable.
    #[serde(default)]
    pub collision: Option<bool>,
    pub document: Document,
    pub asset: String,
    pub instance: String,
    pub frame_origin: Position,
    pub color: [f32; 3],
    #[serde(default)]
    pub shading: meshes::Shading,
    #[serde(default)]
    pub material: crate::materials::Material,
}
pub fn publication(p: Publication) -> Result<(meshes::Asset, meshes::Instance, Compiled)> {
    let compiled = compile(&p.document)?;
    let local = compiled
        .origin
        .project(p.frame_origin, 10000., 0.001)
        .map_err(|e| invalid(e.to_string()))?;
    let asset = meshes::Asset {
        id: p.asset.clone(),
        geometry: Arc::clone(&compiled.geometry),
        shading: p.shading,
        material: p.material,
        rig: None,
        clips: vec![],
    };
    let instance = meshes::Instance {
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
