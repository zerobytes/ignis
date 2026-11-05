//! Opt-in build-time object appearance/depth atlas laboratory. Not an image certificate.
//! Source geometry remains authoritative for near views, editing and unsupported passes.
use super::{RenderFrame, Renderer, Vertex};
use anyhow::{Context, Result, ensure};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

mod bake;
mod gpu;
pub use bake::compile;
pub(crate) use gpu::State;
const MAGIC: &[u8; 8] = b"IGOIM1\0\0";
const MAX_BYTES: usize = 1024 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
pub struct Part {
    pub asset: String,
    pub digest: [u8; 32],
    pub tint: [f32; 3],
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub parts: Vec<Part>,
    pub center: [f32; 3],
    pub radius: f32,
}
#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub resolution: u32,
    pub azimuths: u32,
    pub elevations: Vec<f32>,
    pub entries: Vec<Entry>,
}
pub struct Mip {
    pub base: Vec<u8>,
    /// RGB encodes the source shading normal; A is a nearest-sampled part address.
    pub normal_part: Vec<u8>,
    /// Normalized capture-axis position, quantized to u16 little-endian.
    pub depth: Vec<u8>,
}
pub struct Layer {
    pub mips: Vec<Mip>,
}
pub struct Pack {
    pub metadata: Metadata,
    pub layers: Vec<Layer>,
}
impl Metadata {
    pub fn levels(&self) -> u32 {
        self.resolution.ilog2() + 1
    }
    pub fn views(&self) -> usize {
        self.azimuths as usize * self.elevations.len()
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            self.resolution.is_power_of_two() && (32..=512).contains(&self.resolution),
            "Atlas resolution must be a power of two32..512"
        );
        ensure!(
            (4..=32).contains(&self.azimuths)
                && (1..=4).contains(&self.elevations.len())
                && self
                    .elevations
                    .iter()
                    .all(|v| v.is_finite() && (-1.55..=1.55).contains(v))
                && self.elevations.windows(2).all(|w| w[0] < w[1]),
            "Invalid atlas view grid"
        );
        ensure!(
            !self.entries.is_empty() && self.entries.len() <= 512,
            "Invalid atlas catalog"
        );
        for e in &self.entries {
            // This is a compact packet encoding limit; unsupported objects keep geometry.
            ensure!(
                !e.parts.is_empty()
                    && e.parts.len() <= 256
                    && e.center.iter().all(|v| v.is_finite())
                    && e.radius.is_finite()
                    && e.radius > 0.
                    && e.radius < 100000.,
                "Invalid atlas entry"
            );
            ensure!(
                e.parts.iter().all(|p| !p.asset.is_empty()
                    && p.asset.len() <= 4096
                    && p.tint
                        .iter()
                        .all(|c| c.is_finite() && (0. ..=1.).contains(c))),
                "Invalid atlas part"
            );
        }
        Ok(())
    }
    fn payload_bytes(&self) -> Result<usize> {
        self.validate()?;
        let pixels = (0..self.levels())
            .map(|l| (self.resolution >> l).pow(2) as usize)
            .sum::<usize>();
        let size = pixels
            .checked_mul(10)
            .and_then(|n| n.checked_mul(self.views()))
            .and_then(|n| n.checked_mul(self.entries.len()))
            .context("Atlas payload overflow")?;
        ensure!(
            size < MAX_BYTES,
            "Atlas resource exceeds diagnostic memory budget"
        );
        Ok(size)
    }
}
impl Pack {
    pub fn validate(&self) -> Result<()> {
        self.metadata.payload_bytes()?;
        ensure!(
            self.layers.len() == self.metadata.entries.len() * self.metadata.views(),
            "Atlas layer count mismatch"
        );
        for (layer_index, layer) in self.layers.iter().enumerate() {
            ensure!(
                layer.mips.len() == self.metadata.levels() as usize,
                "Atlas mip count mismatch"
            );
            let part_count = self.metadata.entries[layer_index / self.metadata.views()]
                .parts
                .len();
            for (level, m) in layer.mips.iter().enumerate() {
                let pixels = (self.metadata.resolution >> level).pow(2) as usize;
                ensure!(
                    m.base.len() == pixels * 4
                        && m.normal_part.len() == pixels * 4
                        && m.depth.len() == pixels * 2,
                    "Atlas mip length mismatch"
                );
                ensure!(
                    m.normal_part
                        .chunks_exact(4)
                        .all(|n| (n[3] as usize) < part_count),
                    "Atlas part address outside source object"
                );
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let expected = self.metadata.payload_bytes()?;
        ensure!(
            self.layers.len() == self.metadata.entries.len() * self.metadata.views(),
            "Atlas layer count mismatch"
        );
        let mut meta = vec![];
        ciborium::into_writer(&self.metadata, &mut meta)?;
        ensure!(meta.len() <= 1024 * 1024, "Atlas metadata too large");
        let mut out = Vec::with_capacity(12 + meta.len() + expected + 32);
        out.extend(MAGIC);
        out.extend((meta.len() as u32).to_le_bytes());
        out.extend(meta);
        for layer in &self.layers {
            ensure!(
                layer.mips.len() == self.metadata.levels() as usize,
                "Atlas mip count mismatch"
            );
            for (level, m) in layer.mips.iter().enumerate() {
                let pixels = (self.metadata.resolution >> level).pow(2) as usize;
                ensure!(
                    m.base.len() == pixels * 4
                        && m.normal_part.len() == pixels * 4
                        && m.depth.len() == pixels * 2,
                    "Atlas mip length mismatch"
                );
                out.extend(&m.base);
                out.extend(&m.normal_part);
                out.extend(&m.depth);
            }
        }
        let hash = ring::digest::digest(&ring::digest::SHA256, &out);
        out.extend(hash.as_ref());
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        ensure!(
            (44..=MAX_BYTES + 1024 * 1024 + 44).contains(&bytes.len())
                && bytes.get(..8) == Some(MAGIC),
            "Invalid atlas packet size/header"
        );
        let end = bytes.len() - 32;
        ensure!(
            ring::digest::digest(&ring::digest::SHA256, &bytes[..end]).as_ref() == &bytes[end..],
            "Atlas checksum mismatch"
        );
        let n = u32::from_le_bytes(bytes[8..12].try_into()?) as usize;
        ensure!(
            n <= 1024 * 1024 && 12 + n <= end,
            "Invalid atlas metadata length"
        );
        let mut cursor = std::io::Cursor::new(&bytes[12..12 + n]);
        let metadata: Metadata = ciborium::from_reader(&mut cursor)?;
        ensure!(cursor.position() as usize == n, "Trailing atlas metadata");
        ensure!(
            metadata.payload_bytes()? == end - 12 - n,
            "Atlas payload length mismatch"
        );
        let mut at = 12 + n;
        let mut layers = vec![];
        for _ in 0..metadata.entries.len() * metadata.views() {
            let mut mips = vec![];
            for l in 0..metadata.levels() {
                let p = (metadata.resolution >> l).pow(2) as usize;
                let base = bytes[at..at + p * 4].to_vec();
                at += p * 4;
                let normal_part = bytes[at..at + p * 4].to_vec();
                at += p * 4;
                let depth = bytes[at..at + p * 2].to_vec();
                at += p * 2;
                mips.push(Mip {
                    base,
                    normal_part,
                    depth,
                });
            }
            layers.push(Layer { mips });
        }
        let pack = Self { metadata, layers };
        pack.validate()?;
        Ok(pack)
    }
}

fn eligible(a: &crate::meshes::Asset) -> bool {
    a.rig.is_none()
        && a.clips.is_empty()
        && a.shading == crate::meshes::Shading::Smooth
        && a.geometry.material_coordinates.is_none()
        && a.material.detail.is_none()
        && a.material.procedural.is_none()
        && !a.material.has_coverage()
        && a.material.surface.is_none()
        && a.material.transmission == 0.
}
fn basis(az: f32, elevation: f32) -> (Vec3, Vec3, Vec3) {
    let forward = -Vec3::new(
        az.sin() * elevation.cos(),
        elevation.sin(),
        az.cos() * elevation.cos(),
    );
    let right = forward.cross(Vec3::Y).normalize();
    (right, right.cross(forward), forward)
}
fn world(i: &crate::meshes::Instance) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        Vec3::splat(i.scale),
        Quat::from_array(i.rotation),
        Vec3::from(i.position),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Pack {
        let metadata = Metadata {
            resolution: 32,
            azimuths: 4,
            elevations: vec![0.],
            entries: vec![Entry {
                parts: vec![Part {
                    asset: "sample".into(),
                    digest: [7; 32],
                    tint: [0.5; 3],
                }],
                center: [0.; 3],
                radius: 1.,
            }],
        };
        let layers = (0..4)
            .map(|_| Layer {
                mips: (0..metadata.levels())
                    .map(|l| {
                        let pixels = (32u32 >> l).pow(2) as usize;
                        Mip {
                            base: vec![0; pixels * 4],
                            normal_part: vec![0; pixels * 4],
                            depth: vec![0; pixels * 2],
                        }
                    })
                    .collect(),
            })
            .collect();
        Pack { metadata, layers }
    }
    #[test]
    fn binary_seal_lengths_and_part_addresses_are_validated() {
        let mut p = fixture();
        let b = p.encode().unwrap();
        assert_eq!(Pack::decode(&b).unwrap().encode().unwrap(), b);
        for n in [0, 7, b.len() - 1] {
            assert!(Pack::decode(&b[..n]).is_err());
        }
        let mut bad = b.clone();
        bad[32] ^= 1;
        assert!(Pack::decode(&bad).is_err());
        let mut extra = b;
        extra.push(0);
        assert!(Pack::decode(&extra).is_err());
        p.layers[0].mips[0].normal_part[3] = 1;
        assert!(p.validate().is_err());
        p.layers[0].mips[0].normal_part[3] = 0;
        p.metadata.entries[0].radius = f32::NAN;
        assert!(p.validate().is_err());
    }
    #[test]
    fn both_object_shaders_validate() {
        for source in [
            include_str!("object_impostor/bake.wgsl"),
            include_str!("object_impostor/impostor.wgsl"),
        ] {
            let source = format!("{}\n{}", include_str!("../scene.wgsl"), source);
            let module = wgpu::naga::front::wgsl::parse_str(&source)
                .unwrap_or_else(|e| panic!("{}", e.emit_to_string(&source)));
            wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .unwrap();
        }
    }
}
