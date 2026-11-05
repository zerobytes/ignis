//! Browser host packets use the native material validator and exact GPU packing.
//! This is transient renderer state, not a replacement for the editable asset.
use crate::model::{Result, invalid};

pub const PACKET_VERSION: u32 = 1;
pub const SHADER: &str = concat!(
    include_str!("../scene.wgsl"),
    "\n",
    include_str!("../particles.wgsl")
);

pub fn encode(material: &super::Material) -> Result<Vec<u8>> {
    material.validate()?;
    if material.transparent() || material.surface.as_ref().is_some_and(|s| s.glass.is_some()) {
        return Err(invalid(
            "Web preview1 supports opaque/masked materials only",
        ));
    }
    let (table, program) = super::recipe::pack(std::slice::from_ref(material));
    let table: &[u8] = bytemuck::cast_slice(&table);
    let program: &[u8] = bytemuck::cast_slice(&program);
    if table.len() + program.len() > 1024 * 1024 {
        return Err(invalid("Web preview packet exceeds1MiB"));
    }
    let mut bytes = Vec::with_capacity(16 + table.len() + program.len());
    for word in [
        u32::from_le_bytes(*b"IGWP"),
        PACKET_VERSION,
        table.len() as u32,
        program.len() as u32,
    ] {
        bytes.extend(word.to_le_bytes());
    }
    // GPU storage is little-endian in the browser; make host endianness explicit.
    for word in table.chunks_exact(4).chain(program.chunks_exact(4)) {
        bytes.extend(u32::from_ne_bytes(word.try_into().unwrap()).to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_contains_exact_native_buffers() {
        let material = super::super::library::material(
            Default::default(),
            super::super::library::Execution::Runtime,
        )
        .unwrap();
        let packet = encode(&material).unwrap();
        let (table, words) = super::super::recipe::pack(&[material]);
        assert_eq!(&packet[..4], b"IGWP");
        assert_eq!(&packet[4..8], &PACKET_VERSION.to_le_bytes());
        let size = u32::from_le_bytes(packet[8..12].try_into().unwrap()) as usize;
        assert_eq!(size, std::mem::size_of::<super::super::GpuMaterial>());
        assert_eq!(
            &packet[16..16 + size],
            bytemuck::cast_slice::<_, u8>(&table)
        );
        assert_eq!(&packet[16 + size..], bytemuck::cast_slice::<_, u8>(&words));
    }

    #[test]
    fn invalid_and_unsupported_materials_fail_before_export() {
        let mut material = super::super::Material {
            transmission: f32::NAN,
            ..Default::default()
        };
        assert!(encode(&material).is_err());
        material.transmission = 0.;
        material.surface = Some(super::super::Surface {
            mode: super::super::BlendMode::Blend,
            ..Default::default()
        });
        assert!(encode(&material).is_err());
    }
}
