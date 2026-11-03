//! IGL1 sidecar: source-corner references and u16 indices, never replacement physics.
use crate::{
    meshes::{Asset, Document as Meshes, Geometry, Shading},
    model::{Result, invalid},
};
use glam::{DVec3, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod certificate;
pub(crate) mod selection;
const MAGIC: &[u8; 8] = b"IGLOD1\0\0";
pub const MAX_BYTES: usize = 128 * 1024 * 1024;
const MAX_FACES: usize = 32768; // Compiler work budget; source fallback is unlimited by this operation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub enabled: bool,
    pub error_px: f32,
    pub color_error: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            error_px: 0.25,
            color_error: 0.002,
        }
    }
}
impl Settings {
    pub fn validate(self) -> Result<()> {
        if !self.error_px.is_finite()
            || !(0. ..=2.).contains(&self.error_px)
            || !self.color_error.is_finite()
            || !(0. ..=0.1).contains(&self.color_error)
        {
            return Err(invalid(
                "LOD errors require finite pixels[0,2]/color[0,0.1]",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Level {
    pub corners: Vec<u32>,
    pub triangles: Vec<[u16; 3]>,
    pub error: f32,
    pub normal_error: f32,
}
#[derive(Clone)]
pub struct Pack {
    pub asset: String,
    hash: [u8; 32],
    geometry: Arc<Geometry>,
    material: crate::materials::Material,
    shading: Shading,
    pub levels: Vec<Level>,
}
impl Pack {
    pub fn matches(&self, a: &Asset) -> bool {
        self.asset == a.id
            && Arc::ptr_eq(&self.geometry, &a.geometry)
            && self.shading == a.shading
            && self.material == a.material
            && eligible(a)
    }
}
#[derive(Clone, Default)]
pub struct Document {
    pub packs: Vec<Arc<Pack>>,
}
impl Document {
    pub fn filtered(&self, m: &Meshes) -> Self {
        Self {
            packs: self
                .packs
                .iter()
                .filter(|p| m.assets.iter().any(|a| p.matches(a)))
                .cloned()
                .collect(),
        }
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"format":"IGL1","bytes":self.byte_len(),"assets":self.packs.iter().map(|p|serde_json::json!({"asset":p.asset,"source_triangles":p.geometry.triangles.len(),"levels":p.levels.iter().map(|l|serde_json::json!({"triangles":l.triangles.len(),"vertices":l.corners.len(),"surface_error":l.error,"normal_correspondence_error":l.normal_error})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"physics":"source_unchanged","image_error_is_not_certified":true})
    }
    pub fn byte_len(&self) -> usize {
        44 + self
            .packs
            .iter()
            .map(|p| {
                40 + p.asset.len()
                    + p.levels
                        .iter()
                        .map(|l| 16 + l.corners.len() * 4 + l.triangles.len() * 6)
                        .sum::<usize>()
            })
            .sum::<usize>()
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        put(&mut out, self.packs.len() as u32);
        for p in &self.packs {
            put(&mut out, p.asset.len() as u32);
            out.extend(p.asset.as_bytes());
            out.extend(p.hash);
            put(&mut out, p.levels.len() as u32);
            for l in &p.levels {
                put(&mut out, l.error.to_bits());
                put(&mut out, l.normal_error.to_bits());
                put(&mut out, l.corners.len() as u32);
                put(&mut out, l.triangles.len() as u32);
                for c in &l.corners {
                    put(&mut out, *c);
                }
                for t in &l.triangles {
                    for i in t {
                        out.extend(i.to_le_bytes());
                    }
                }
            }
        }
        let hash = ring::digest::digest(&ring::digest::SHA256, &out);
        out.extend(hash.as_ref());
        out
    }
    pub fn decode(bytes: &[u8], meshes: &Meshes) -> Result<Self> {
        if bytes.len() < 44 || bytes.len() > MAX_BYTES || &bytes[..8] != MAGIC {
            return Err(invalid("Invalid IGL1 header/size"));
        }
        let (payload, seal) = bytes.split_at(bytes.len() - 32);
        if ring::digest::digest(&ring::digest::SHA256, payload).as_ref() != seal {
            return Err(invalid("IGL1 checksum mismatch"));
        }
        let mut r = Reader {
            bytes: payload,
            at: 8,
        };
        let count = r.word()? as usize;
        if count > meshes.assets.len() {
            return Err(invalid("IGL1 asset count exceeds source"));
        }
        let mut out = Self::default();
        let mut names = BTreeSet::new();
        for _ in 0..count {
            let n = r.word()? as usize;
            if !(1..=80).contains(&n) {
                return Err(invalid("Invalid IGL1 asset name"));
            }
            let name = std::str::from_utf8(r.take(n)?)
                .map_err(|_| invalid("Invalid IGL1 UTF8"))?
                .to_string();
            if !names.insert(name.clone()) {
                return Err(invalid("Duplicate IGL1 asset"));
            }
            let hash: [u8; 32] = r.take(32)?.try_into().unwrap();
            let a = meshes
                .assets
                .iter()
                .find(|a| a.id == name)
                .ok_or_else(|| invalid("IGL1 source asset missing"))?;
            if !eligible(a) || hash != digest(a) {
                return Err(invalid("IGL1 source mismatch/ineligible material"));
            }
            let nl = r.word()? as usize;
            if !(1..=3).contains(&nl) {
                return Err(invalid("IGL1 level count"));
            }
            let source = source_vertices(a);
            let components = components(a);
            let mut levels = Vec::new();
            for _ in 0..nl {
                let error = f32::from_bits(r.word()?);
                let normal_error = f32::from_bits(r.word()?);
                let nv = r.word()? as usize;
                let nt = r.word()? as usize;
                if !error.is_finite()
                    || error < 0.
                    || !normal_error.is_finite()
                    || !(0. ..=2.).contains(&normal_error)
                    || !(3..65535).contains(&nv)
                    || nt == 0
                    || nt >= a.geometry.triangles.len()
                    || nt > MAX_FACES
                {
                    return Err(invalid("IGL1 level bounds/counts"));
                }
                if nv * 4 + nt * 6 > r.remaining() {
                    return Err(invalid("Truncated IGL1 level"));
                }
                let mut corners = Vec::with_capacity(nv);
                for _ in 0..nv {
                    let c = r.word()?;
                    if c as usize >= source.len() {
                        return Err(invalid("IGL1 source corner out of bounds"));
                    }
                    corners.push(c);
                }
                let mut triangles = Vec::with_capacity(nt);
                for _ in 0..nt {
                    let mut t = [0; 3];
                    for i in &mut t {
                        *i = u16::from_le_bytes(r.take(2)?.try_into().unwrap());
                        if *i as usize >= nv {
                            return Err(invalid("IGL1 index out of bounds"));
                        }
                    }
                    triangles.push(t);
                }
                let l = Level {
                    corners,
                    triangles,
                    error,
                    normal_error,
                };
                verify(&source, &components, &l)?;
                levels.push(l);
            }
            out.packs.push(Arc::new(Pack {
                asset: name,
                hash,
                geometry: a.geometry.clone(),
                material: a.material.clone(),
                shading: a.shading,
                levels,
            }));
        }
        if r.remaining() != 0 {
            return Err(invalid("Trailing IGL1 bytes"));
        }
        Ok(out)
    }
}
fn put(out: &mut Vec<u8>, v: u32) {
    out.extend(v.to_le_bytes());
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| invalid("IGL1 length overflow"))?;
        let x = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| invalid("Truncated IGL1"))?;
        self.at = end;
        Ok(x)
    }
    fn word(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn remaining(&self) -> usize {
        self.bytes.len() - self.at
    }
}
fn digest(a: &Asset) -> [u8; 32] {
    let mut h = ring::digest::Context::new(&ring::digest::SHA256);
    h.update(&crate::distant::digest(a));
    h.update(&[u8::from(a.shading == Shading::Smooth)]);
    h.finish().as_ref().try_into().unwrap()
}
fn eligible(a: &Asset) -> bool {
    a.rig.is_none()
        && a.clips.is_empty()
        && a.geometry.triangles.len() <= MAX_FACES
        && a.geometry.positions.len() <= MAX_FACES * 3
        && (a.shading == Shading::Smooth || a.geometry.triangles.len() * 3 < 65534)
        && a.material.procedural.is_none()
        && a.material.pattern.is_none()
        && a.material.secondary_pattern.is_none()
        && a.material.detail.is_none()
        && a.material.transmission == 0.
        && a.material
            .reflectance
            .is_none_or(|r| r.f0 == 0. && r.metallic == 0.)
}
#[derive(Clone, Copy)]
pub(super) struct V {
    p: DVec3,
    n: DVec3,
}
fn source_vertices(a: &Asset) -> Vec<V> {
    let mut normals = vec![Vec3::ZERO; a.geometry.positions.len()];
    if a.shading == Shading::Smooth {
        crate::mesh_normals::accumulate(&a.geometry.triangles, &a.geometry.positions, &mut normals);
    }
    a.geometry
        .triangles
        .iter()
        .flat_map(|t| {
            let face =
                crate::mesh_normals::face(t.map(|i| Vec3::from(a.geometry.positions[i as usize])))
                    .0;
            t.map(|i| V {
                p: Vec3::from(a.geometry.positions[i as usize]).as_dvec3(),
                n: crate::mesh_normals::corner(
                    a.shading == Shading::Smooth,
                    normals[i as usize],
                    face,
                )
                .as_dvec3(),
            })
        })
        .collect()
}
fn components(a: &Asset) -> Vec<Vec<usize>> {
    let mut by_vertex = vec![Vec::new(); a.geometry.positions.len()];
    for (i, t) in a.geometry.triangles.iter().enumerate() {
        for &v in t {
            by_vertex[v as usize].push(i);
        }
    }
    let mut seen = vec![false; a.geometry.triangles.len()];
    let mut out = Vec::new();
    for i in 0..seen.len() {
        if seen[i] {
            continue;
        }
        seen[i] = true;
        let mut stack = vec![i];
        let mut part = Vec::new();
        while let Some(t) = stack.pop() {
            part.push(t);
            for &v in &a.geometry.triangles[t] {
                for &f in &by_vertex[v as usize] {
                    if !seen[f] {
                        seen[f] = true;
                        stack.push(f);
                    }
                }
            }
        }
        part.sort_unstable();
        out.push(part);
    }
    out
}
// Local candidate generation preserves region borders; a difficult junction
// no longer forces every face of the same connected component to stay full.
// This is a compiler work partition, never a new runtime object or draw call.
fn regions(a: &Asset, components: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut adjacent = vec![Vec::new(); a.geometry.positions.len()];
    for (f, t) in a.geometry.triangles.iter().enumerate() {
        for &v in t {
            adjacent[v as usize].push(f);
        }
    }
    let mut used = vec![false; a.geometry.triangles.len()];
    let mut out = Vec::new();
    for part in components {
        for &first in part {
            if used[first] {
                continue;
            }
            let mut region = Vec::new();
            let mut frontier = std::collections::VecDeque::from([first]);
            used[first] = true;
            while let Some(f) = frontier.pop_front() {
                region.push(f);
                if region.len() + frontier.len() >= 128 {
                    continue;
                }
                'neighbors: for &v in &a.geometry.triangles[f] {
                    for &n in &adjacent[v as usize] {
                        if !used[n] {
                            used[n] = true;
                            frontier.push_back(n);
                            if region.len() + frontier.len() >= 128 {
                                break 'neighbors;
                            }
                        }
                    }
                }
            }
            region.sort_unstable();
            out.push(region);
        }
    }
    out
}
fn append(level: &mut Level, faces: &[[u32; 3]]) -> Result<()> {
    let mut remap = BTreeMap::new();
    for f in faces {
        let mut t = [0u16; 3];
        for (at, &c) in f.iter().enumerate() {
            let id = if let Some(&n) = remap.get(&c) {
                n
            } else {
                let n = level.corners.len();
                if n >= 65534 {
                    return Err(invalid("IGL1 candidate exceeds u16; use source"));
                }
                level.corners.push(c);
                remap.insert(c, n as u16);
                n as u16
            };
            t[at] = id;
        }
        level.triangles.push(t);
    }
    Ok(())
}
fn generate_level(
    a: &Asset,
    source: &[V],
    parts: &[Vec<usize>],
    ratio: f64,
    normal_cap: f32,
) -> Result<Level> {
    let mut level = Level {
        corners: Vec::new(),
        triangles: Vec::new(),
        error: 0.,
        normal_error: 0.,
    };
    let mut budget = 8_000_000usize;
    let mut diagnostic = (0usize, 0usize, 0usize, 0f32, 0f32);
    for part in parts {
        let mut lookup = BTreeMap::new();
        let mut corners = Vec::new();
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut faces = Vec::new();
        for &f in part {
            let mut tri = [0; 3];
            for (j, i) in tri.iter_mut().enumerate() {
                let corner = f * 3 + j;
                let v = source[corner];
                let key = a.geometry.triangles[f][j];
                *i = *lookup.entry(key).or_insert_with(|| {
                    let n = corners.len() as u32;
                    corners.push(corner as u32);
                    positions.push(v.p.as_vec3().to_array());
                    normals.extend(v.n.as_vec3().to_array());
                    n
                });
            }
            faces.push(tri);
        }
        let original: Vec<_> = faces.iter().flatten().copied().collect();
        let original_source: Vec<_> = part
            .iter()
            .map(|&f| [(f * 3) as u32, (f * 3 + 1) as u32, (f * 3 + 2) as u32])
            .collect();
        let mut reduced_source = original_source.clone();
        let mut proof = None;
        let mut attempted = false;
        if faces.len() >= 3 && budget > 100 {
            let adapter = meshopt::VertexDataAdapter::new(bytemuck::cast_slice(&positions), 12, 0)
                .map_err(|e| invalid(e.to_string()))?;
            // Spend build time searching for a useful candidate instead
            // of accepting the first (position-dominated) collapse.
            for weight in [32., 8., 1.] {
                let candidate = meshopt::simplify_with_attributes_and_locks(
                    &original,
                    &adapter,
                    &normals,
                    &[weight; 3],
                    12,
                    &vec![false; positions.len()],
                    ((original.len() as f64 * ratio) as usize / 3 * 3).max(3),
                    0.04,
                    meshopt::SimplifyOptions::LockBorder,
                    None,
                );
                if candidate.is_empty() || candidate.len() >= original.len() {
                    continue;
                }
                attempted = true;
                let reduced: Vec<_> = candidate
                    .chunks_exact(3)
                    .map(|t| {
                        [
                            corners[t[0] as usize],
                            corners[t[1] as usize],
                            corners[t[2] as usize],
                        ]
                    })
                    .collect();
                if let Ok(bound) = certificate::bounds(
                    source,
                    &original_source,
                    &reduced,
                    0.005,
                    0.06,
                    &mut budget,
                ) && bound.1 <= normal_cap
                {
                    proof = Some(bound);
                    reduced_source = reduced;
                    break;
                }
                if budget <= 100 {
                    break;
                }
            }
        }
        diagnostic.0 += usize::from(attempted);
        diagnostic.1 += usize::from(proof.is_some());
        if let Some((p, n)) = proof {
            diagnostic.3 = diagnostic.3.max(p);
            diagnostic.4 = diagnostic.4.max(n);
        }
        // Build records correspondence bounds; physical-pixel/color gates
        // decide at runtime whether they are useful at the current distance.
        if let Some((error, normal)) = proof.filter(|&(_, n)| n <= normal_cap) {
            if append(&mut level, &reduced_source).is_err() {
                return Err(invalid("IGL1 candidate exceeds u16; try another partition"));
            }
            diagnostic.2 += faces.len() - reduced_source.len();
            level.error = level.error.max(error);
            level.normal_error = level.normal_error.max(normal);
        } else {
            if append(&mut level, &original_source).is_err() {
                return Err(invalid("IGL1 candidate exceeds u16; try another partition"));
            }
        }
    }
    if std::env::var_os("IGNIS_LOD_DIAGNOSTICS").is_some() {
        eprintln!(
            "{} ratio{ratio}: attempts={} proofs={} saved={} bounds={:.6}/{:.6} budget={}",
            a.id, diagnostic.0, diagnostic.1, diagnostic.2, diagnostic.3, diagnostic.4, budget
        );
    }
    Ok(level)
}
pub fn compile(meshes: &Meshes) -> Result<Document> {
    meshes.validate()?;
    let mut out = Document::default();
    for a in &meshes.assets {
        if !eligible(a) || a.geometry.triangles.len() < 16 {
            continue;
        }
        let source = source_vertices(a);
        let components = components(a);
        let regions = regions(a, &components);
        let mut levels = Vec::new();
        for (ratio, normal_cap) in [(0.75, 0.02), (0.5, 0.06), (0.25, 0.2)] {
            let mut best: Option<Level> = None;
            // Keep the existing whole-component candidate when partitioning
            // would cost more. Region boundaries must never regress simple meshes.
            for parts in [&components, &regions] {
                let Ok(mut level) = generate_level(a, &source, parts, ratio, normal_cap) else {
                    continue;
                };
                if level.triangles.len() * 10 >= a.geometry.triangles.len() * 9 {
                    continue;
                }
                let Ok((error, normal)) = measure(&source, &components, &level) else {
                    continue;
                };
                if normal > normal_cap {
                    continue;
                }
                level.error = level.error.max(error);
                level.normal_error = level.normal_error.max(normal);
                if best.as_ref().is_none_or(|b| {
                    level.triangles.len() < b.triangles.len()
                        || (level.triangles.len() == b.triangles.len() && level.error < b.error)
                }) {
                    best = Some(level);
                }
            }
            if let Some(level) = best
                && !levels.iter().any(|l: &Level| {
                    l.triangles.len() <= level.triangles.len()
                        && l.error <= level.error
                        && l.normal_error <= level.normal_error
                })
            {
                levels.push(level);
            }
        }
        if !levels.is_empty() {
            out.packs.push(Arc::new(Pack {
                asset: a.id.clone(),
                hash: digest(a),
                geometry: a.geometry.clone(),
                material: a.material.clone(),
                shading: a.shading,
                levels,
            }));
        }
    }
    if out.encode().len() > MAX_BYTES {
        return Err(invalid("IGL1 compiler output exceeds byte budget"));
    }
    Ok(out)
}
fn measure(source: &[V], parts: &[Vec<usize>], l: &Level) -> Result<(f32, f32)> {
    let mut membership = vec![0; source.len() / 3];
    for (n, p) in parts.iter().enumerate() {
        for &f in p {
            membership[f] = n;
        }
    }
    let mut groups = vec![Vec::new(); parts.len()];
    for t in &l.triangles {
        let ids = t.map(|i| l.corners[i as usize]);
        let c = membership[ids[0] as usize / 3];
        if ids.iter().any(|&i| membership[i as usize / 3] != c) {
            return Err(invalid("IGL1 triangle joins disconnected components"));
        }
        groups[c].push(ids);
    }
    let mut budget = 8_000_000;
    let (mut error, mut normal) = (0f32, 0f32);
    for (p, faces) in parts.iter().zip(groups) {
        if faces.is_empty() {
            return Err(invalid("IGL1 loses a component"));
        }
        let orig: Vec<_> = p
            .iter()
            .map(|&f| [(f * 3) as u32, (f * 3 + 1) as u32, (f * 3 + 2) as u32])
            .collect();
        // Exact unchanged corner topology requires no expensive surface query.
        if faces == orig {
            continue;
        }
        let bound = certificate::bounds(source, &orig, &faces, 0.005, 0.06, &mut budget)?;
        error = error.max(bound.0);
        normal = normal.max(bound.1);
    }
    Ok((error, normal))
}
fn verify(source: &[V], parts: &[Vec<usize>], l: &Level) -> Result<()> {
    let (error, normal) = measure(source, parts, l)?;
    if l.error + 1e-6 < error || l.normal_error + 1e-6 < normal {
        return Err(invalid("IGL1 surface/normal bound mismatch"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
