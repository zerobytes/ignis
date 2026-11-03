//! Portable, versioned Ignis save. Native physics caches are never required.
use crate::{
    engine::{Engine, ExecutionMode},
    model::{Result, Scene, invalid},
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
pub const VERSION: u16 = 1;
pub const MAX_BYTES: usize = 128 * 1024 * 1024;
const HEADER: usize = 50;
const MAGIC: &[u8; 8] = b"IGNISPSV";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableSave {
    pub version: u16,
    pub scene_version: u32,
    pub spatial_version: u16,
    pub origin: crate::world_space::Position,
    pub scene: Scene,
    pub mode: ExecutionMode,
    pub(crate) sequences: [u64; 3],
    pub(crate) manual: std::collections::VecDeque<crate::noise::Noise>,
    pub(crate) dropped_manual: u64,
    pub(crate) state: crate::physics::PortableSimulation,
}
impl PortableSave {
    pub fn capture(engine: &Engine) -> Result<Self> {
        engine.capture_portable_save()
    }
    pub fn tick(&self) -> u64 {
        self.state.tick()
    }
    pub fn validate_header(&self) -> Result<()> {
        if self.version != VERSION
            || self.scene_version != crate::model::SCENE_VERSION
            || self.scene.version != self.scene_version
            || self.spatial_version != crate::world_space::VERSION
            || self.origin != origin()
            || self.manual.len() > 128
            || self.tick() > crate::engine::MAX_RUNTIME_TICK
            || self.sequences[0] > crate::noise::MAX_SERIAL
        {
            return Err(invalid(
                "Incompatible portable save schema/scene/spatial frame/domain; migration required",
            ));
        }
        self.scene.validate()?;
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate_header()?;
        let mut output = Limited {
            bytes: vec![0; HEADER],
        };
        ciborium::ser::into_writer(self, &mut output)
            .map_err(|e| invalid(format!("Portable save encode: {e}")))?;
        let len = (output.bytes.len() - HEADER) as u64;
        let hash = ring::digest::digest(&ring::digest::SHA256, &output.bytes[HEADER..]);
        output.bytes[..8].copy_from_slice(MAGIC);
        output.bytes[8..10].copy_from_slice(&VERSION.to_le_bytes());
        output.bytes[10..18].copy_from_slice(&len.to_le_bytes());
        output.bytes[18..50].copy_from_slice(hash.as_ref());
        Ok(output.bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER
            || bytes.len() > MAX_BYTES
            || &bytes[..8] != MAGIC
            || u16::from_le_bytes(bytes[8..10].try_into().unwrap()) != VERSION
            || u64::from_le_bytes(bytes[10..18].try_into().unwrap())
                != (bytes.len() - HEADER) as u64
        {
            return Err(invalid("Invalid save envelope/length/version"));
        }
        let payload = &bytes[HEADER..];
        let hash = ring::digest::digest(&ring::digest::SHA256, payload);
        if hash.as_ref() != &bytes[18..50] {
            return Err(invalid("Portable save integrity mismatch"));
        }
        let mut reader = std::io::Cursor::new(payload);
        let save: Self = ciborium::de::from_reader_with_recursion_limit(&mut reader, 128)
            .map_err(|e| invalid(format!("Portable save decode: {e}")))?;
        if reader.position() != payload.len() as u64 {
            return Err(invalid("Trailing save data"));
        }
        save.validate_header()?;
        Ok(save)
    }
    /// Explicit v1 migration boundary. Compatible native input is restored first;
    /// unsupported native/platform/schema versions are rejected before any write.
    pub fn migrate(bytes: &[u8]) -> Result<Vec<u8>> {
        if bytes.starts_with(MAGIC) {
            let engine = Engine::from_portable_save(bytes, None)?;
            return Self::capture(&engine)?.encode();
        }
        let engine = Engine::from_checkpoint(bytes, None)?;
        Self::capture(&engine)?.encode()
    }
    pub fn read(path: &std::path::Path) -> Result<Vec<u8>> {
        let file = std::fs::File::open(path).map_err(|e| invalid(e.to_string()))?;
        if file.metadata().map_err(|e| invalid(e.to_string()))?.len() > MAX_BYTES as u64 {
            return Err(invalid("Portable save exceeds128MiB"));
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| invalid(e.to_string()))?;
        if bytes.len() > MAX_BYTES {
            return Err(invalid("Portable save exceeds128MiB"));
        }
        Ok(bytes)
    }
}
pub fn origin() -> crate::world_space::Position {
    crate::world_space::Position::normalize(crate::world_space::Coordinates {
        cell: [0; 3],
        offset: [0.; 3],
    })
    .expect("zero world origin")
}
struct Limited {
    bytes: Vec<u8>,
}
impl Write for Limited {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > MAX_BYTES - self.bytes.len() {
            return Err(std::io::Error::other("Portable save exceeds128MiB"));
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
