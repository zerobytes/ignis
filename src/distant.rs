//! Explicit authoring-time compilation of bounded static mesh substitutes.
//! ATM2/ATM3 store f32 positions/u16 indices; JSON base64 is only the authoring boundary.
use crate::{
    meshes::{Asset, Document, Geometry},
    model::{Result, invalid},
};
use glam::{DVec3, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
};
mod heightfield;
pub(crate) mod selection;
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
const CELLS: [u32; 5] = [64, 32, 16, 8, 4];
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub asset: String,
    #[serde(with = "blob")]
    #[schemars(with = "String")]
    pub data: Arc<Vec<u8>>,
    #[serde(skip)]
    verified: Arc<OnceLock<Verified>>,
}
#[derive(Debug)]
struct Verified {
    asset: String,
    data: Arc<Vec<u8>>,
    geometry: Arc<Geometry>,
    material: crate::materials::Material,
}
impl PartialEq for Pack {
    fn eq(&self, other: &Self) -> bool {
        self.asset == other.asset && self.data == other.data
    }
}
pub fn is_empty(v: &Arc<Vec<Pack>>) -> bool {
    v.is_empty()
}
#[derive(Clone)]
pub(crate) struct Level {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u16; 3]>,
    pub error: f32,
    pub vertical: bool,
}
pub(crate) fn digest(a: &Asset) -> [u8; 32] {
    let g = &a.geometry;
    let mut h = ring::digest::Context::new(&ring::digest::SHA256);
    h.update(&(g.positions.len() as u32).to_le_bytes());
    for p in &g.positions {
        for v in p {
            h.update(&v.to_le_bytes());
        }
    }
    for t in &g.triangles {
        for v in t {
            h.update(&v.to_le_bytes());
        }
    }
    let mut material = Vec::new();
    ciborium::ser::into_writer(&a.material, &mut material)
        .expect("validated material binary encoding");
    h.update(&material);
    h.finish().as_ref().try_into().unwrap()
}
pub fn compile(document: &Document) -> Result<Arc<Vec<Pack>>> {
    document.validate()?;
    let mut packs = Vec::new();
    for a in &document.assets {
        if a.rig.is_none()
            && a.material.surface.is_none()
            && a.material.procedural.is_none()
            && a.clips.is_empty()
            && a.material
                .reflectance
                .is_none_or(|r| r.f0 == 0. && r.metallic == 0.)
        {
            packs.push(Pack {
                asset: a.id.clone(),
                data: Arc::new(encode(a, true)),
                verified: Default::default(),
            });
        }
    }
    Ok(Arc::new(packs))
}
fn encode(a: &Asset, planar: bool) -> Vec<u8> {
    let g = &a.geometry;
    let mut bytes = if planar { b"ATM3" } else { b"ATM2" }.to_vec();
    bytes.extend(digest(a));
    for v in selection::bound(&a.material, Vec3::ONE).to_array() {
        bytes.extend(v.to_le_bytes());
    }
    let levels = (if g.positions.len() > u16::MAX as usize {
        Some(Vec::new())
    } else if planar {
        heightfield::compile_with_planar(g)
    } else {
        heightfield::compile(g)
    })
    .unwrap_or_else(|| {
        CELLS
            .into_iter()
            .filter_map(|cells| cluster(g, cells))
            .collect()
    });
    let mut filtered: Vec<Level> = Vec::new();
    for l in levels {
        if l.triangles.len() >= g.triangles.len()
            || filtered.iter().any(|p| {
                p.triangles.len() <= l.triangles.len()
                    && p.error <= l.error
                    && p.vertical == l.vertical
            })
        {
            continue;
        }
        filtered.push(l);
        if filtered.len() == 5 {
            break;
        }
    }
    bytes.extend((filtered.len() as u32).to_le_bytes());
    for l in filtered {
        bytes.extend(l.error.to_le_bytes());
        bytes.extend(u32::from(l.vertical).to_le_bytes());
        bytes.extend((l.positions.len() as u32).to_le_bytes());
        bytes.extend((l.triangles.len() as u32).to_le_bytes());
        for p in l.positions {
            for v in p {
                bytes.extend(v.to_le_bytes());
            }
        }
        for t in l.triangles {
            for v in t {
                bytes.extend(v.to_le_bytes());
            }
        }
    }
    bytes
}
fn cluster(g: &Geometry, cells: u32) -> Option<Level> {
    let (lo, hi) = bounds(g.positions.iter().copied());
    let step = (hi - lo).max_element() as f64 / f64::from(cells);
    if step <= 0. {
        return None;
    }
    let low = lo.as_dvec3();
    let key = |p: [f32; 3]| {
        ((DVec3::from_array(p.map(f64::from)) - low) / step)
            .floor()
            .to_array()
            .map(|v| v as i32)
    };
    let mut groups = BTreeMap::<[i32; 3], (DVec3, usize, u16)>::new();
    for &p in &g.positions {
        let e = groups.entry(key(p)).or_default();
        e.0 += DVec3::from_array(p.map(f64::from));
        e.1 += 1;
    }
    // ATM indices use u16 and reserve MAX as a remapping sentinel.
    if groups.len() >= u16::MAX as usize {
        return None;
    }
    let mut positions = Vec::with_capacity(groups.len());
    for (sum, n, index) in groups.values_mut() {
        *index = positions.len() as u16;
        positions.push((*sum / *n as f64).as_vec3().to_array());
    }
    let mut error = 0f64;
    let mapping: Vec<_> = g
        .positions
        .iter()
        .map(|&p| {
            let i = groups[&key(p)].2;
            error = error.max(
                (DVec3::from_array(p.map(f64::from))
                    - DVec3::from_array(positions[i as usize].map(f64::from)))
                .length(),
            );
            i
        })
        .collect();
    let mut seen = BTreeSet::new();
    let mut triangles = Vec::new();
    for t in &g.triangles {
        let t = t.map(|i| mapping[i as usize]);
        if t[0] == t[1] || t[1] == t[2] || t[2] == t[0] {
            continue;
        }
        let mut sorted = t;
        sorted.sort_unstable();
        if seen.insert(sorted) {
            triangles.push(t);
        }
    }
    if triangles.is_empty() || triangles.len() >= g.triangles.len() {
        return None;
    }
    let mut compact = Vec::new();
    let mut remap = vec![u16::MAX; positions.len()];
    for t in &mut triangles {
        for i in t {
            let old = *i as usize;
            if remap[old] == u16::MAX {
                remap[old] = compact.len() as u16;
                compact.push(positions[old]);
            }
            *i = remap[old];
        }
    }
    positions = compact;
    // Convex barycentric interpolation bounds every mapped point by the maximum
    // vertex displacement. Deleted collapsed faces lie within this same bound.
    let pad = 16. * f32::EPSILON * lo.abs().max(hi.abs()).max_element().max(1.);
    Some(Level {
        positions,
        triangles,
        error: (error as f32 + pad).next_up(),
        vertical: false,
    })
}
pub(crate) fn bounds(points: impl Iterator<Item = [f32; 3]>) -> (Vec3, Vec3) {
    points.fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(lo, hi), p| {
            let p = Vec3::from(p);
            (lo.min(p), hi.max(p))
        },
    )
}
impl Pack {
    fn payload_verified(&self) -> bool {
        self.verified
            .get()
            .is_some_and(|v| v.asset == self.asset && Arc::ptr_eq(&v.data, &self.data))
    }
    fn is_verified(&self, a: &Asset) -> bool {
        self.verified.get().is_some_and(|v| {
            self.asset == a.id
                && v.asset == a.id
                && Arc::ptr_eq(&v.data, &self.data)
                && Arc::ptr_eq(&v.geometry, &a.geometry)
                && v.material == a.material
        })
    }
    pub(crate) fn matches(&self, a: &Asset) -> bool {
        self.is_verified(a)
            || (self.asset == a.id && self.data.len() >= 52 && self.data[4..36] == digest(a))
    }
    pub(crate) fn upper(&self) -> Vec3 {
        Vec3::from_array(std::array::from_fn(|i| {
            f32::from_le_bytes(self.data[36 + i * 4..40 + i * 4].try_into().unwrap())
        }))
    }
    pub(crate) fn levels(&self) -> Result<Vec<Level>> {
        let b = self.data.as_slice();
        if b.len() < 52 || b.len() > MAX_BYTES || (&b[..4] != b"ATM2" && &b[..4] != b"ATM3") {
            return Err(invalid("Invalid ATM payload header/size"));
        }
        let upper = self.upper();
        if !upper.is_finite() || upper.min_element() < 0. || upper.max_element() > 64. {
            return Err(invalid("Invalid ATM payload material bound"));
        }
        let mut offset = 48;
        let mut word = || -> Result<u32> {
            let bytes = b
                .get(offset..offset + 4)
                .ok_or_else(|| invalid("Truncated ATM payload"))?;
            offset += 4;
            Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
        };
        let count = word()? as usize;
        if count > CELLS.len() {
            return Err(invalid("ATM payload exceeds5 levels"));
        }
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let read = |offset: &mut usize| -> Result<u32> {
                let bytes = b
                    .get(*offset..*offset + 4)
                    .ok_or_else(|| invalid("Truncated ATM payload"))?;
                *offset += 4;
                Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
            };
            let error = f32::from_bits(read(&mut offset)?);
            let axis = read(&mut offset)?;
            if axis > 1 {
                return Err(invalid("Invalid ATM payload error axis"));
            }
            let nv = read(&mut offset)? as usize;
            let nt = read(&mut offset)? as usize;
            if !error.is_finite() || error < 0. || !(3..u16::MAX as usize).contains(&nv) || nt == 0
            {
                return Err(invalid("Invalid ATM payload counts/error"));
            }
            if offset + nv * 12 + nt * 6 > b.len() {
                return Err(invalid("Truncated ATM payload payload"));
            }
            let mut positions = Vec::with_capacity(nv);
            for _ in 0..nv {
                let p = [
                    f32::from_bits(read(&mut offset)?),
                    f32::from_bits(read(&mut offset)?),
                    f32::from_bits(read(&mut offset)?),
                ];
                if p.iter().any(|v| !v.is_finite() || v.abs() > 10000.) {
                    return Err(invalid("Invalid ATM payload position"));
                }
                positions.push(p);
            }
            let mut triangles = Vec::with_capacity(nt);
            for _ in 0..nt {
                let t = std::array::from_fn(|_| {
                    let v = u16::from_le_bytes(b[offset..offset + 2].try_into().unwrap());
                    offset += 2;
                    v
                });
                if t.iter().any(|&v| v as usize >= nv) {
                    return Err(invalid("Invalid ATM payload index"));
                }
                triangles.push(t);
            }
            out.push(Level {
                positions,
                triangles,
                error,
                vertical: axis == 1,
            });
        }
        if offset != b.len() {
            return Err(invalid("Trailing ATM payload bytes"));
        }
        Ok(out)
    }
}
pub fn validate(packs: &[Pack], document: &Document) -> Result<()> {
    let mut ids = BTreeSet::new();
    for p in packs {
        if !ids.insert(&p.asset) {
            return Err(invalid("Duplicate atmospheric asset"));
        }
        let source = document.assets.iter().find(|a| a.id == p.asset);
        if source.is_some_and(|a| p.is_verified(a)) {
            continue;
        }
        if !p.payload_verified() {
            p.levels()?;
        }
        if let Some(a) = source {
            // Stale external packs are ignored and dropped by scene normalization. A matching
            // pack must prove the exact deterministic derivation; hashes alone are not authority.
            if p.matches(a) {
                if !p.payload_verified() && p.data.as_slice() != encode(a, &p.data[..4] == b"ATM3")
                {
                    return Err(invalid("ATM payload derivation/certificate mismatch"));
                }
                // Retaining the immutable Arcs makes mutation through Arc::make_mut
                // produce a new allocation, which cannot reuse this proof. A loaded
                // packet starts without a proof and must reproduce its derivation.
                p.verified.get_or_init(|| Verified {
                    asset: a.id.clone(),
                    data: p.data.clone(),
                    geometry: a.geometry.clone(),
                    material: a.material.clone(),
                });
            }
        }
    }
    Ok(())
}
mod blob {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde::{
        Deserializer, Serializer,
        de::{self, Visitor},
    };
    pub fn serialize<S: Serializer>(
        v: &Arc<Vec<u8>>,
        s: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            s.serialize_str(&STANDARD.encode(v.as_slice()))
        } else {
            s.serialize_bytes(v)
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<Arc<Vec<u8>>, D::Error> {
        struct Bytes;
        impl<'de> Visitor<'de> for Bytes {
            type Value = Arc<Vec<u8>>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "bounded ATM bytes/base64")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
                if v.len() > MAX_BYTES.div_ceil(3) * 4 {
                    return Err(E::custom("ATM payload too large"));
                }
                let bytes = STANDARD.decode(v).map_err(E::custom)?;
                self.visit_byte_buf(bytes)
            }
            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> std::result::Result<Self::Value, E> {
                if v.len() > MAX_BYTES {
                    return Err(E::custom("ATM payload too large"));
                }
                self.visit_byte_buf(v.to_vec())
            }
            fn visit_byte_buf<E: de::Error>(
                self,
                v: Vec<u8>,
            ) -> std::result::Result<Self::Value, E> {
                if v.len() > MAX_BYTES {
                    Err(E::custom("ATM payload too large"))
                } else {
                    Ok(Arc::new(v))
                }
            }
        }
        d.deserialize_any(Bytes)
    }
}
