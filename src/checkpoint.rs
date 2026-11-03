//! Local native save files. Not a remote protocol or a replay of the full history.
use crate::{
    engine::{Engine, ExecutionMode},
    model::{Result, Scene, invalid},
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
pub const VERSION: u16 = 3;
pub const MAX_BYTES: usize = 128 * 1024 * 1024;
const HEADER: usize = 50;
const MAGIC: &[u8; 8] = b"IGNISCPT";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub version: u16,
    pub scene_version: u32,
    pub physics_version: String,
    pub platform: String,
    pub build_id: String,
    pub scene: Scene,
    pub mode: ExecutionMode,
    pub(crate) sequences: [u64; 3],
    pub(crate) manual: std::collections::VecDeque<crate::noise::Noise>,
    pub(crate) dropped_manual: u64,
    pub(crate) state: crate::physics::SavedSimulation,
}
impl Checkpoint {
    pub fn capture(engine: &Engine) -> Result<Self> {
        engine.capture_checkpoint()
    }
    pub fn tick(&self) -> u64 {
        self.state.tick()
    }
    pub fn validate_header(&self) -> Result<()> {
        if self.version != VERSION
            || self.scene_version != crate::model::SCENE_VERSION
            || self.scene.version != self.scene_version
            || self.physics_version != "rapier3d-0.30.1"
            || self.platform != platform()
            || self.manual.len() > 128
            || self.tick() > crate::engine::MAX_RUNTIME_TICK
            || self.sequences[0] > crate::noise::MAX_SERIAL
        {
            return Err(invalid(
                "Incompatible checkpoint format/physics/platform/domain",
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
            .map_err(|e| invalid(format!("Checkpoint encode: {e}")))?;
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
            return Err(invalid("Invalid checkpoint envelope/length/version"));
        }
        let payload = &bytes[HEADER..];
        let hash = ring::digest::digest(&ring::digest::SHA256, payload);
        if hash.as_ref() != &bytes[18..50] {
            return Err(invalid("Checkpoint integrity mismatch"));
        }
        let mut reader = std::io::Cursor::new(payload);
        let checkpoint: Self = ciborium::de::from_reader_with_recursion_limit(&mut reader, 128)
            .map_err(|e| invalid(format!("Checkpoint decode: {e}")))?;
        if reader.position() != payload.len() as u64 {
            return Err(invalid("Trailing checkpoint data"));
        }
        checkpoint.validate_header()?;
        Ok(checkpoint)
    }
    pub fn read(path: &std::path::Path) -> Result<Vec<u8>> {
        let file = std::fs::File::open(path).map_err(|e| invalid(e.to_string()))?;
        if file.metadata().map_err(|e| invalid(e.to_string()))?.len() > MAX_BYTES as u64 {
            return Err(invalid("Checkpoint exceeds128MiB"));
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| invalid(e.to_string()))?;
        if bytes.len() > MAX_BYTES {
            return Err(invalid("Checkpoint exceeds128MiB"));
        }
        Ok(bytes)
    }
}
pub(crate) fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}
struct Limited {
    bytes: Vec<u8>,
}
impl Write for Limited {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > MAX_BYTES - self.bytes.len() {
            return Err(std::io::Error::other("Checkpoint exceeds128MiB"));
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Replay JSON wraps the native binary baseline in base64; no JSON conversion of
/// Rapier's floating-point sentinels, maps or internals, and no nested checkpoints.
pub(crate) mod baseline {
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;
    pub fn serialize<S: Serializer>(
        value: &Option<Arc<[u8]>>,
        s: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        value
            .as_ref()
            .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
            .serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<Option<Arc<[u8]>>, D::Error> {
        let value = Option::<String>::deserialize(d)?;
        value
            .map(|s| {
                if s.len() > super::MAX_BYTES.div_ceil(3) * 4 {
                    return Err(serde::de::Error::custom(
                        "Baseline exceeds checkpoint budget",
                    ));
                }
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(s)
                    .map_err(serde::de::Error::custom)?;
                if bytes.len() > super::MAX_BYTES {
                    return Err(serde::de::Error::custom(
                        "Baseline exceeds checkpoint budget",
                    ));
                }
                Ok(Arc::from(bytes))
            })
            .transpose()
    }
}
