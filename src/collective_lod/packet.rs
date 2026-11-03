//! ICB1 exact-source geometry hierarchy, compiled once and interpreted per covered pixel.
use crate::{
    mesh_objects::Object,
    meshes::{self, Asset, Document, Geometry, Shading},
};
use anyhow::{Result, anyhow, ensure};
use glam::{Mat4, Quat, Vec3};
use std::sync::Arc;
const MAGIC: &[u8; 8] = b"IGCB1\0\0\0";
const MAX_BYTES: usize = 128 * 1024 * 1024;
#[derive(Clone)]
pub struct Part {
    pub name: String,
    pub geometry: Arc<Geometry>,
    pub shading: Shading,
}
pub struct Pack {
    pub center: Vec3,
    pub radius: f32,
    pub parts: Vec<Part>,
    pub nodes: Vec<[u32; 8]>,
    pub triangles: Vec<[u32; 20]>,
    pub source_triangles: usize,
}
pub fn eligible(a: &Asset) -> bool {
    a.rig.is_none() && a.clips.is_empty() && a.material == Default::default()
}
pub fn world(i: &meshes::Instance) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        Vec3::splat(i.scale),
        Quat::from_array(i.rotation),
        Vec3::from(i.position),
    )
}
fn hash(parts: &[Part]) -> [u8; 32] {
    let mut h = ring::digest::Context::new(&ring::digest::SHA256);
    for p in parts {
        h.update(&(p.name.len() as u32).to_le_bytes());
        h.update(p.name.as_bytes());
        h.update(&[u8::from(p.shading == Shading::Smooth)]);
        h.update(&(p.geometry.positions.len() as u32).to_le_bytes());
        h.update(&(p.geometry.triangles.len() as u32).to_le_bytes());
        for v in &p.geometry.positions {
            for c in v {
                h.update(&c.to_le_bytes());
            }
        }
        for t in &p.geometry.triangles {
            for c in t {
                h.update(&c.to_le_bytes());
            }
        }
    }
    h.finish().as_ref().try_into().unwrap()
}
fn source(doc: &Document, object: &Object) -> Result<(Vec<Part>, Vec<crate::render::Vertex>)> {
    ensure!(
        !object.members.is_empty() && object.members.len() < u32::MAX as usize,
        "ICB1 requires nonempty constant material parts within structural u32 indexing"
    );
    let mut canonical = Document::default();
    let mut parts = vec![];
    let mut root: Option<&meshes::Instance> = None;
    for (slot, name) in object.members.iter().enumerate() {
        let i = doc
            .instances
            .iter()
            .find(|i| &i.id == name)
            .ok_or_else(|| anyhow!("Missing member"))?;
        ensure!(
            i.attachment.is_none() && i.playback.is_none(),
            "Animated/attached members use source"
        );
        if let Some(r) = root {
            ensure!(
                i.position == r.position && i.rotation == r.rotation && i.scale == r.scale,
                "ICB1 experiment requires co-transforming parts"
            );
        } else {
            root = Some(i);
        }
        let a = doc.asset(&i.asset).map_err(|e| anyhow!(e.message))?;
        ensure!(eligible(a), "Unsupported material/animation uses source");
        parts.push(Part {
            name: a.id.clone(),
            geometry: a.geometry.clone(),
            shading: a.shading,
        });
        let mut a = a.clone();
        a.id = format!("part{slot}");
        let mut i = i.clone();
        i.id = a.id.clone();
        i.asset = a.id.clone();
        i.position = [0.; 3];
        i.rotation = [0., 0., 0., 1.];
        i.scale = 1.;
        i.color = [1.; 3];
        canonical.assets.push(a);
        canonical.instances.push(i);
    }
    let mut frame = meshes::Frame::default();
    frame.update(&canonical, 0);
    Ok((parts, frame.vertices))
}
fn native(parts: &[Part]) -> Vec<[u32; 20]> {
    let mut doc = Document::default();
    for (n, p) in parts.iter().enumerate() {
        let id = format!("part{n}");
        doc.assets.push(Asset {
            id: id.clone(),
            geometry: p.geometry.clone(),
            shading: p.shading,
            material: Default::default(),
            rig: None,
            clips: vec![],
        });
        doc.instances.push(meshes::Instance {
            id: id.clone(),
            asset: id,
            position: [0.; 3],
            rotation: [0., 0., 0., 1.],
            scale: 1.,
            color: [1.; 3],
            playback: None,
            attachment: None,
        });
    }
    let mut frame = meshes::Frame::default();
    frame.update(&doc, 0);
    records(&frame.vertices)
}
fn records(vertices: &[crate::render::Vertex]) -> Vec<[u32; 20]> {
    vertices
        .chunks_exact(3)
        .enumerate()
        .map(|(face, t)| {
            let mut words = [0; 20];
            for n in 0..3 {
                for j in 0..3 {
                    words[n * 3 + j] = t[n].position[j].to_bits();
                    words[9 + n * 3 + j] = t[n].normal[j].to_bits();
                }
            }
            words[18] = t[0].material - 1;
            words[19] = face as u32;
            words
        })
        .collect()
}
fn position(t: &[u32; 20], corner: usize) -> Vec3 {
    Vec3::new(
        f32::from_bits(t[corner * 3]),
        f32::from_bits(t[corner * 3 + 1]),
        f32::from_bits(t[corner * 3 + 2]),
    )
}
fn bounds(ids: &[usize], source: &[[u32; 20]]) -> (Vec3, Vec3) {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for &i in ids {
        for j in 0..3 {
            let p = position(&source[i], j);
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    (lo, hi)
}
fn build(
    ids: &mut [usize],
    source: &[[u32; 20]],
    nodes: &mut Vec<[u32; 8]>,
    triangles: &mut Vec<[u32; 20]>,
    leaf: usize,
) {
    let (lo, hi) = bounds(ids, source);
    let pad = lo.abs().max(hi.abs()).max_element().max(1.) * 64. * f32::EPSILON;
    let (lo, hi) = (lo - Vec3::splat(pad), hi + Vec3::splat(pad));
    let at = nodes.len();
    nodes.push([0; 8]);
    let mut descriptor = 0;
    if ids.len() <= leaf {
        ids.sort_unstable();
        descriptor = ((triangles.len() as u32) << 4) | ids.len() as u32;
        triangles.extend(ids.iter().map(|&i| source[i]));
    } else {
        let extent = hi - lo;
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };
        ids.sort_unstable_by(|&a, &b| {
            let centroid = |i| {
                (position(&source[i], 0)[axis]
                    + position(&source[i], 1)[axis]
                    + position(&source[i], 2)[axis])
                    / 3.
            };
            centroid(a).total_cmp(&centroid(b)).then(a.cmp(&b))
        });
        let (a, b) = ids.split_at_mut(ids.len() / 2);
        build(a, source, nodes, triangles, leaf);
        build(b, source, nodes, triangles, leaf);
    }
    nodes[at] = [
        lo.x.to_bits(),
        lo.y.to_bits(),
        lo.z.to_bits(),
        nodes.len() as u32,
        hi.x.to_bits(),
        hi.y.to_bits(),
        hi.z.to_bits(),
        descriptor,
    ];
}
pub fn compile(doc: &Document, object: &Object, leaf: u32) -> Result<Pack> {
    doc.validate().map_err(|e| anyhow!(e.message))?;
    ensure!(
        doc.objects.iter().any(|o| o == object),
        "Compile a published object"
    );
    ensure!((1..=8).contains(&leaf), "ICB1 leaves require 1..8 faces");
    let count: usize = object
        .members
        .iter()
        .map(|m| {
            doc.instances
                .iter()
                .find(|i| &i.id == m)
                .and_then(|i| doc.assets.iter().find(|a| a.id == i.asset))
                .map_or(0, |a| a.geometry.triangles.len())
        })
        .sum();
    ensure!(
        count > 0 && count < MAX_BYTES / 144,
        "ICB1 compiler work budget; retain source"
    );
    let (parts, vertices) = source(doc, object)?;
    let data = records(&vertices);
    let mut ids: Vec<_> = (0..data.len()).collect();
    let (lo, hi) = bounds(&ids, &data);
    let center = (lo + hi) * 0.5;
    let radius = data
        .iter()
        .flat_map(|t| (0..3).map(move |j| position(t, j)))
        .map(|p| p.distance(center))
        .fold(0., f32::max)
        * 1.0001;
    ensure!(radius.is_finite() && radius > 0., "Invalid ICB1 bounds");
    let (mut nodes, mut triangles) = (vec![], vec![]);
    build(&mut ids, &data, &mut nodes, &mut triangles, leaf as usize);
    let pack = Pack {
        center,
        radius,
        parts,
        nodes,
        triangles,
        source_triangles: data.len(),
    };
    pack.validate_layout()?;
    Ok(pack)
}
fn put(out: &mut Vec<u8>, v: u32) {
    out.extend(v.to_le_bytes());
}
fn node_bounds(n: &[u32; 8]) -> (Vec3, Vec3) {
    (
        Vec3::new(
            f32::from_bits(n[0]),
            f32::from_bits(n[1]),
            f32::from_bits(n[2]),
        ),
        Vec3::new(
            f32::from_bits(n[4]),
            f32::from_bits(n[5]),
            f32::from_bits(n[6]),
        ),
    )
}
impl Pack {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        for v in [self.nodes.len(), self.triangles.len(), self.parts.len()] {
            put(&mut out, v as u32);
        }
        for v in self.center.to_array() {
            put(&mut out, v.to_bits());
        }
        put(&mut out, self.radius.to_bits());
        out.extend(hash(&self.parts));
        for p in &self.parts {
            put(&mut out, p.name.len() as u32);
            out.extend(p.name.as_bytes());
        }
        for n in &self.nodes {
            for &w in n {
                put(&mut out, w);
            }
        }
        for t in &self.triangles {
            for &w in t {
                put(&mut out, w);
            }
        }
        let seal = ring::digest::digest(&ring::digest::SHA256, &out);
        out.extend(seal.as_ref());
        out
    }
    pub fn decode(bytes: &[u8], doc: &Document) -> Result<Self> {
        ensure!(
            bytes.len() >= 100 && bytes.len() <= MAX_BYTES && &bytes[..8] == MAGIC,
            "Invalid ICB1 header/size"
        );
        let (body, seal) = bytes.split_at(bytes.len() - 32);
        ensure!(
            ring::digest::digest(&ring::digest::SHA256, body).as_ref() == seal,
            "ICB1 integrity mismatch"
        );
        let mut r = Reader { bytes: body, at: 8 };
        let nn = r.word()? as usize;
        let nt = r.word()? as usize;
        let np = r.word()? as usize;
        ensure!(
            nn > 0
                && nt > 0
                && np > 0
                && np <= doc.instances.len()
                && np <= MAX_BYTES / 5
                && nn <= MAX_BYTES / 32
                && nt <= MAX_BYTES / 80,
            "Invalid ICB1 counts"
        );
        let center = Vec3::new(
            f32::from_bits(r.word()?),
            f32::from_bits(r.word()?),
            f32::from_bits(r.word()?),
        );
        let radius = f32::from_bits(r.word()?);
        ensure!(
            center.is_finite() && radius.is_finite() && radius > 0.,
            "Invalid ICB1 sphere"
        );
        let digest = r.take(32)?;
        let mut parts = vec![];
        for _ in 0..np {
            let n = r.word()? as usize;
            ensure!((1..=80).contains(&n), "Invalid ICB1 name");
            let name = std::str::from_utf8(r.take(n)?)?;
            let a = doc.asset(name).map_err(|e| anyhow!(e.message))?;
            ensure!(eligible(a), "ICB1 source ineligible");
            parts.push(Part {
                name: name.into(),
                geometry: a.geometry.clone(),
                shading: a.shading,
            });
        }
        ensure!(hash(&parts).as_slice() == digest, "ICB1 source mismatch");
        ensure!(
            parts
                .iter()
                .map(|p| p.geometry.triangles.len())
                .sum::<usize>()
                == nt,
            "ICB1 source coverage count"
        );
        ensure!(
            r.bytes.len() - r.at == nn * 32 + nt * 80,
            "ICB1 length mismatch"
        );
        let mut nodes = Vec::with_capacity(nn);
        for _ in 0..nn {
            let mut n = [0; 8];
            for w in &mut n {
                *w = r.word()?;
            }
            nodes.push(n);
        }
        let mut triangles = Vec::with_capacity(nt);
        for _ in 0..nt {
            let mut t = [0; 20];
            for w in &mut t {
                *w = r.word()?;
            }
            triangles.push(t);
        }
        let pack = Pack {
            center,
            radius,
            parts,
            nodes,
            triangles,
            source_triangles: nt,
        };
        pack.validate_layout()?;
        // A checksum alone is not a geometric proof. Recompose every source
        // corner/normal and verify the exact permutation plus containing tree.
        let expected = native(&pack.parts);
        let mut seen = vec![false; nt];
        for t in &pack.triangles {
            let id = t[19] as usize;
            ensure!(
                id < nt && !seen[id] && *t == expected[id],
                "ICB1 source corner/normal derivation mismatch"
            );
            seen[id] = true;
        }
        Ok(pack)
    }
    pub(crate) fn validate_layout(&self) -> Result<()> {
        ensure!(
            self.center.is_finite()
                && self.radius.is_finite()
                && self.radius > 0.
                && !self.parts.is_empty()
                && self.parts.len() < u32::MAX as usize
                && !self.nodes.is_empty()
                && !self.triangles.is_empty()
                && self.nodes.len() * 32 + self.triangles.len() * 80 < MAX_BYTES,
            "Invalid ICB1 layout"
        );
        let mut seen = vec![false; self.triangles.len()];
        self.verify_node(0, self.nodes.len(), 0, &mut seen)?;
        ensure!(seen.iter().all(|x| *x), "ICB1 loses triangle slots");
        Ok(())
    }
    fn verify_node(&self, at: usize, end: usize, depth: usize, seen: &mut [bool]) -> Result<()> {
        ensure!(
            depth <= 64 && at < end && end <= self.nodes.len(),
            "Invalid ICB1 tree depth/range"
        );
        let node = &self.nodes[at];
        ensure!(node[3] as usize == end, "ICB1 escape mismatch");
        let (lo, hi) = node_bounds(node);
        ensure!(
            lo.is_finite() && hi.is_finite() && lo.cmple(hi).all(),
            "Invalid ICB1 node bounds"
        );
        let count = (node[7] & 15) as usize;
        if count > 0 {
            let first = (node[7] >> 4) as usize;
            ensure!(
                count <= 8 && end == at + 1 && first + count <= self.triangles.len(),
                "Invalid ICB1 leaf span"
            );
            for (slot, used) in seen.iter_mut().enumerate().skip(first).take(count) {
                ensure!(!*used, "ICB1 duplicates a leaf slot");
                *used = true;
                let t = &self.triangles[slot];
                ensure!((t[18] as usize) < self.parts.len(), "Invalid ICB1 palette");
                for j in 0..3 {
                    let p = position(t, j);
                    ensure!(
                        p.is_finite()
                            && p.cmpge(lo).all()
                            && p.cmple(hi).all()
                            && p.distance(self.center) <= self.radius,
                        "ICB1 leaf/sphere loses source"
                    );
                }
            }
        } else {
            ensure!(node[7] == 0 && at + 1 < end, "Invalid ICB1 branch");
            let right = self.nodes[at + 1][3] as usize;
            ensure!(right > at + 1 && right < end, "ICB1 child split");
            self.verify_node(at + 1, right, depth + 1, seen)?;
            self.verify_node(right, end, depth + 1, seen)?;
            for c in [at + 1, right] {
                let (a, b) = node_bounds(&self.nodes[c]);
                ensure!(
                    a.cmpge(lo).all() && b.cmple(hi).all(),
                    "ICB1 branch does not enclose children"
                );
            }
        }
        Ok(())
    }
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
            .ok_or_else(|| anyhow!("ICB1 overflow"))?;
        let b = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| anyhow!("Truncated ICB1"))?;
        self.at = end;
        Ok(b)
    }
    fn word(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
}
