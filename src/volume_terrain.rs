//! Editable 3-D density fields. Positive is solid, zero is surface; not an SDF.
//! GPU Gems 3 ch.1 mathematics, with native seeded value noise (not NVIDIA textures).
use crate::model::{Result, invalid};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
mod extract;
pub mod gpu;
mod tables;
pub const VERSION: u16 = 2;
pub const MAX_BYTES: usize = 65536;
pub const MAX_POINTS: usize = 2_200_000;
pub const MAX_VERTICES: usize = 600_000;
pub const MAX_TRIANGLES: usize = 1_200_000;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Base {
    Plane { height: f64 },
    Sphere { center: [f64; 3], radius: f64 },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Octave {
    pub frequency: f64,
    pub amplitude: f64,
    pub rotation_degrees: [f64; 3],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Warp {
    pub frequency: f64,
    pub amplitude: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Terraces {
    pub spacing: f64,
    pub strength: f64,
    pub overhang: f64,
    pub sharpness: f64,
    /// 0 = original altitude, 1 = fully warped altitude.
    pub warp: f64,
    pub phase: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shelf {
    pub altitude: f64,
    pub half_thickness: f64,
    pub strength: f64,
    pub blend: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Floor {
    pub altitude: f64,
    pub transition: f64,
    pub strength: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u16,
    pub seed: u32,
    pub base: Base,
    #[serde(default = "unit_scale", skip_serializing_if = "is_unit_scale")]
    pub noise_domain_scale: [f64; 3],
    pub vertical_strength: f64,
    #[serde(deserialize_with = "octaves")]
    pub octaves: Vec<Octave>,
    #[serde(default)]
    pub warp: Option<Warp>,
    #[serde(default)]
    pub terraces: Option<Terraces>,
    #[serde(default)]
    pub shelf: Option<Shelf>,
    #[serde(default)]
    pub floor: Option<Floor>,
}
fn unit_scale() -> [f64; 3] {
    [1.; 3]
}
fn is_unit_scale(v: &[f64; 3]) -> bool {
    *v == [1.; 3]
}
fn octaves<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Octave>, D::Error> {
    crate::terrain::limited::<D, Octave, 12>(d)
}
fn bounded(x: f64, lo: f64, hi: f64) -> bool {
    x.is_finite() && (lo..=hi).contains(&x)
}
fn point(p: [f64; 3]) -> bool {
    p.iter().all(|&x| bounded(x, -1e12, 1e12))
}
impl Document {
    pub fn validate(&self) -> Result<()> {
        if ![1, VERSION].contains(&self.version)
            || self
                .noise_domain_scale
                .iter()
                .any(|&v| !bounded(v, 0.0001, 10000.))
            || (self.version < 2 && !is_unit_scale(&self.noise_domain_scale))
            || !bounded(self.vertical_strength, 0.001, 100.)
            || self.octaves.len() > 12
        {
            return Err(invalid(
                "Density requires version1/2, positive domain scales0.0001..10000 (non-unit requires2), strength0.001..100 and at most12 octaves",
            ));
        }
        let base_ok = match self.base {
            Base::Plane { height } => bounded(height, -1e12, 1e12),
            Base::Sphere { center, radius } => point(center) && bounded(radius, 1., 1e12),
        };
        if !base_ok
            || self.octaves.iter().any(|o| {
                !bounded(o.frequency, 1e-9, 100.)
                    || !bounded(o.amplitude, 0., 10000.)
                    || o.rotation_degrees.iter().any(|&a| !bounded(a, -360., 360.))
            })
            || self.warp.as_ref().is_some_and(|w| {
                !bounded(w.frequency, 1e-9, 100.) || !bounded(w.amplitude, 0., 10000.)
            })
            || self.terraces.as_ref().is_some_and(|t| {
                !bounded(t.spacing, 0.01, 100000.)
                    || !bounded(t.strength, 0., 10000.)
                    || !bounded(t.overhang, 0., 10000.)
                    || !bounded(t.sharpness, 1., 16.)
                    || !bounded(t.warp, 0., 1.)
                    || !bounded(t.phase, -100000., 100000.)
            })
            || self.shelf.as_ref().is_some_and(|s| {
                !bounded(s.altitude, -1e9, 1e9)
                    || !bounded(s.half_thickness, 0.001, 10000.)
                    || !bounded(s.strength, 0., 10000.)
                    || !bounded(s.blend, 0., 1.)
            })
            || self.floor.as_ref().is_some_and(|f| {
                !bounded(f.altitude, -1e9, 1e9)
                    || !bounded(f.transition, 0.001, 10000.)
                    || !bounded(f.strength, 0., 10000.)
            })
        {
            return Err(invalid(
                "Density recipe contains invalid base, noise, warp, terrace, shelf or floor parameters",
            ));
        }
        Ok(())
    }
}
struct PreparedOctave {
    frequency: f64,
    amplitude: f64,
    rotation: glam::DMat3,
    seed: u32,
}
pub struct Sampler<'a> {
    doc: &'a Document,
    domain_scale: DVec3,
    inverse_domain_scale: DVec3,
    octaves: Vec<PreparedOctave>,
}
impl<'a> Sampler<'a> {
    pub fn new(doc: &'a Document) -> Result<Self> {
        doc.validate()?;
        let octaves = doc
            .octaves
            .iter()
            .enumerate()
            .map(|(i, o)| PreparedOctave {
                frequency: o.frequency,
                amplitude: o.amplitude,
                rotation: glam::DMat3::from_quat(glam::DQuat::from_euler(
                    glam::EulerRot::XYZ,
                    o.rotation_degrees[0].to_radians(),
                    o.rotation_degrees[1].to_radians(),
                    o.rotation_degrees[2].to_radians(),
                )),
                seed: doc.seed.wrapping_add((i as u32).wrapping_mul(0x9e3779b9)),
            })
            .collect();
        Ok(Self {
            doc,
            octaves,
            domain_scale: DVec3::from_array(doc.noise_domain_scale),
            inverse_domain_scale: DVec3::ONE / DVec3::from_array(doc.noise_domain_scale),
        })
    }
    fn altitude(&self, p: DVec3) -> f64 {
        match self.doc.base {
            Base::Plane { height } => p.y - height,
            Base::Sphere { center, radius } => p.distance(DVec3::from_array(center)) - radius,
        }
    }
    pub fn sample(&self, p: [f64; 3]) -> Result<f64> {
        if !point(p) {
            return Err(invalid(
                "Density sample position must be finite within +/-1e12m",
            ));
        }
        Ok(self.value(DVec3::from_array(p)))
    }
    fn value(&self, p: DVec3) -> f64 {
        let original_altitude = self.altitude(p);
        let mut q = p;
        let noise_p = p * self.domain_scale;
        if let Some(w) = &self.doc.warp {
            for axis in 0..3 {
                let seed = self
                    .doc
                    .seed
                    .wrapping_add(0x85ebca6b_u32.wrapping_mul(axis as u32 + 1));
                q[axis] += w.amplitude
                    * self.inverse_domain_scale[axis]
                    * (noise(noise_p * w.frequency, seed) * 0.64
                        + noise(noise_p * (w.frequency * 0.461), seed ^ 0xc2b2ae35) * 0.32);
            }
        }
        let altitude = self.altitude(q);
        let mut d = -altitude * self.doc.vertical_strength;
        if let Some(f) = &self.doc.floor {
            d += ((f.altitude - original_altitude) / f.transition).clamp(0., 1.) * f.strength;
        }
        if let Some(s) = &self.doc.shelf {
            let weight = (s.half_thickness - (altitude - s.altitude).abs()).clamp(0., 1.) * s.blend;
            d += (s.strength - d) * weight;
        }
        if let Some(t) = &self.doc.terraces {
            let height = original_altitude + (altitude - original_altitude) * t.warp;
            let fy = -height / t.spacing + t.phase;
            let fraction = fy - fy.floor();
            let snapped = smooth_snap(fraction, t.sharpness);
            d += (fy.floor() + snapped - t.phase) * t.strength + (snapped - fraction) * t.overhang;
        }
        let noise_q = q * self.domain_scale;
        for o in &self.octaves {
            d += noise((o.rotation * noise_q) * o.frequency, o.seed) * o.amplitude;
        }
        d
    }
    pub fn normal(&self, p: [f64; 3], epsilon: f64) -> Result<[f64; 3]> {
        if !point(p) || !bounded(epsilon, 1e-6, 100.) {
            return Err(invalid(
                "Density normal requires valid position and epsilon1e-6..100m",
            ));
        }
        let p = DVec3::from_array(p);
        let g = DVec3::new(
            self.value(p + DVec3::X * epsilon) - self.value(p - DVec3::X * epsilon),
            self.value(p + DVec3::Y * epsilon) - self.value(p - DVec3::Y * epsilon),
            self.value(p + DVec3::Z * epsilon) - self.value(p - DVec3::Z * epsilon),
        );
        Ok((-g).try_normalize().unwrap_or(DVec3::ZERO).to_array())
    }
}
fn smooth_snap(t: f64, m: f64) -> f64 {
    if t <= 0.5 {
        0.5 * (2. * t).powf(m)
    } else {
        1. - 0.5 * (2. * (1. - t)).powf(m)
    }
}
fn hash(mut v: u32) -> u32 {
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846ca68b);
    v ^ (v >> 16)
}
fn noise(p: DVec3, seed: u32) -> f64 {
    let cell = p.floor();
    let f = p - cell;
    let f = f * f * f * (f * (f * 6. - DVec3::splat(15.)) + DVec3::splat(10.));
    let c = cell.to_array().map(|x| x as i64 as u32);
    let mut value = 0.;
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let h = hash(
                    c[0].wrapping_add(x)
                        ^ hash(c[1].wrapping_add(y))
                        ^ hash(c[2].wrapping_add(z)).rotate_left(13)
                        ^ seed,
                );
                let v = f64::from(h) * (2. / f64::from(u32::MAX)) - 1.;
                value += v
                    * if x == 0 { 1. - f.x } else { f.x }
                    * if y == 0 { 1. - f.y } else { f.y }
                    * if z == 0 { 1. - f.z } else { f.z };
            }
        }
    }
    value
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub origin: [f64; 3],
    pub cells: [u16; 3],
    pub spacing: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_spacing: Option<f64>,
}
impl Grid {
    fn step(&self, axis: usize) -> f64 {
        if axis == 1 {
            self.vertical_spacing.unwrap_or(self.spacing)
        } else {
            self.spacing
        }
    }
    fn dimensions(&self) -> Result<([usize; 3], usize)> {
        let n = self.cells.map(|v| usize::from(v) + 1);
        let count = n
            .iter()
            .try_fold(1usize, |a, &b| a.checked_mul(b))
            .ok_or_else(|| invalid("Density sample count overflow"))?;
        if self.cells.contains(&0)
            || count > MAX_POINTS
            || !bounded(self.spacing, 0.01, 4096.)
            || !bounded(self.step(1), 0.01, 4096.)
            || !point(self.origin)
            || (0..3).any(|i| self.origin[i].abs() + f64::from(self.cells[i]) * self.step(i) > 1e12)
            || (0..3).any(|i| f64::from(self.cells[i]) * self.step(i) > 10000.)
        {
            return Err(invalid(
                "Density grid exceeds finite position, spacing0.01..4096m, 10000m extent or 2200000sample budget",
            ));
        }
        Ok((n, count))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    Samples {
        document: Document,
        #[serde(deserialize_with = "sample_points")]
        points: Vec<[f64; 3]>,
    },
    Inspect {
        document: Document,
        grid: Grid,
    },
}
fn sample_points<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<[f64; 3]>, D::Error> {
    crate::terrain::limited::<D, [f64; 3], 4096>(d)
}
pub fn query(q: Query) -> Result<serde_json::Value> {
    match q {
        Query::Samples { document, points } => {
            if points.len() > 4096 {
                return Err(invalid("At most4096 density queries"));
            }
            let s = Sampler::new(&document)?;
            let samples=points.iter().map(|&p|Ok(serde_json::json!({"position":p,"density":s.sample(p)?,"normal":s.normal(p,0.001)?}))).collect::<Result<Vec<_>>>()?;
            Ok(
                serde_json::json!({"samples":samples,"positive_is_solid":true,"signed_distance":false}),
            )
        }
        Query::Inspect { document, grid } => Ok(compile(&document, &grid)?.report),
    }
}
pub struct Compiled {
    pub geometry: Arc<crate::meshes::Geometry>,
    pub report: serde_json::Value,
}
pub fn compile(document: &Document, grid: &Grid) -> Result<Compiled> {
    extract::compile(document, grid)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub document: Document,
    pub grid: Grid,
    pub asset: String,
    pub instance: String,
    pub position: [f32; 3],
    pub color: [f32; 3],
    #[serde(default)]
    pub material: crate::materials::Material,
    #[serde(default)]
    pub collision: Option<bool>,
}
pub fn publication(
    p: Publication,
) -> Result<(crate::meshes::Asset, crate::meshes::Instance, Compiled)> {
    let c = compile(&p.document, &p.grid)?;
    if c.geometry.triangles.is_empty() {
        return Err(invalid(
            "Density grid is empty; inspect or choose another region before publishing",
        ));
    }
    Ok(publication_from(p, c))
}
pub(crate) fn publication_from(
    p: Publication,
    c: Compiled,
) -> (crate::meshes::Asset, crate::meshes::Instance, Compiled) {
    let asset = crate::meshes::Asset {
        id: p.asset.clone(),
        geometry: c.geometry.clone(),
        shading: crate::meshes::Shading::Smooth,
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
    (asset, instance, c)
}
pub fn encode(doc: &Document) -> Result<Vec<u8>> {
    doc.validate()?;
    let mut bytes = b"IGNISVOL\x01".to_vec();
    ciborium::into_writer(doc, &mut bytes).map_err(|e| invalid(e.to_string()))?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("Density source exceeds64KiB"));
    }
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Document> {
    if bytes.len() > MAX_BYTES || !bytes.starts_with(b"IGNISVOL\x01") {
        return Err(invalid("Invalid density source envelope"));
    }
    let mut reader = std::io::Cursor::new(&bytes[9..]);
    let doc: Document = ciborium::from_reader(&mut reader).map_err(|e| invalid(e.to_string()))?;
    if reader.position() as usize != bytes.len() - 9 {
        return Err(invalid("Trailing density source bytes"));
    }
    doc.validate()?;
    Ok(doc)
}
#[cfg(test)]
mod tests;
