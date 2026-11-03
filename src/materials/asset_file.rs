//! Bounded library2 envelope. PNG stores canonical float bytes losslessly, not
//! quantized colors; decoding reconstructs precisely the authored mip samples.
use super::{
    Material,
    recipe::{Fixed, MAX_FILE_BYTES, Sample, sample_count},
};
use crate::model::{Result, invalid};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::Arc};
const MAGIC: &[u8; 9] = b"IGNISMAT2";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    version: u16,
    material: Material,
}
fn error(e: impl std::fmt::Display) -> crate::model::Error {
    invalid(e.to_string())
}
pub(super) fn encode(material: &Material) -> Result<Vec<u8>> {
    material.validate()?;
    let fixed = material.procedural.as_ref().and_then(|p| p.fixed.as_ref());
    let mut metadata = material.clone();
    if let Some(p) = metadata.procedural.as_mut() {
        Arc::make_mut(p).fixed = None;
    }
    let json = serde_json::to_vec(&File {
        version: 2,
        material: metadata,
    })
    .map_err(error)?;
    let Some(fixed) = fixed else {
        if json.len() > MAX_FILE_BYTES {
            return Err(invalid("Material exceeds96MiB"));
        }
        return Ok(json);
    };
    if json.len() > 1024 * 1024 {
        return Err(invalid("Material metadata exceeds1MiB"));
    }
    let width = u32::from(fixed.resolution) * 12;
    let height = fixed.samples.len().div_ceil(usize::from(fixed.resolution)) as u32;
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for s in fixed.samples.iter() {
        for v in s
            .color_roughness
            .iter()
            .chain(&s.response)
            .chain(&s.emission)
        {
            pixels.extend(v.to_le_bytes());
        }
    }
    pixels.resize(width as usize * height as usize * 4, 0);
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Balanced);
        let mut writer = encoder.write_header().map_err(error)?;
        writer.write_image_data(&pixels).map_err(error)?;
    }
    let mut out = Vec::with_capacity(MAGIC.len() + 6 + json.len() + encoded.len());
    out.extend(MAGIC);
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(fixed.resolution.to_le_bytes());
    out.extend(json);
    out.extend(encoded);
    if out.len() > MAX_FILE_BYTES {
        return Err(invalid("Material exceeds96MiB"));
    }
    Ok(out)
}
pub(super) fn decode(bytes: &[u8]) -> Result<Material> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(invalid("Material exceeds96MiB"));
    }
    if !bytes.starts_with(MAGIC) {
        let f: File = serde_json::from_slice(bytes).map_err(error)?;
        if !(1..=2).contains(&f.version) {
            return Err(invalid("Unsupported material library version"));
        }
        f.material.validate()?;
        return Ok(f.material);
    }
    if bytes.len() < 15 {
        return Err(invalid("Truncated binary material header"));
    }
    let size = u32::from_le_bytes(bytes[9..13].try_into().unwrap()) as usize;
    let resolution = u16::from_le_bytes(bytes[13..15].try_into().unwrap());
    if size > 1024 * 1024
        || !matches!(resolution, 16 | 32 | 64 | 128 | 256 | 512 | 1024)
        || size > bytes.len() - 15
    {
        return Err(invalid("Invalid binary material metadata/resolution"));
    }
    let mut f: File = serde_json::from_slice(&bytes[15..15 + size]).map_err(error)?;
    if f.version != 2 {
        return Err(invalid("Binary material requires library2 metadata"));
    }
    f.material.validate()?;
    let p = f
        .material
        .procedural
        .as_mut()
        .ok_or_else(|| invalid("Binary material requires procedural source"))?;
    if p.fixed.is_some() {
        return Err(invalid("Binary metadata must not duplicate fixed maps"));
    }
    let count = sample_count(resolution);
    let width = u32::from(resolution) * 12;
    let height = count.div_ceil(usize::from(resolution)) as u32;
    let expected = width as usize * height as usize * 4;
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(&bytes[15 + size..]),
        png::Limits {
            bytes: expected + 2 * 1024 * 1024,
        },
    );
    decoder.set_transformations(png::Transformations::IDENTITY);
    let mut reader = decoder.read_info().map_err(error)?;
    let info = reader.info();
    if info.width != width
        || info.height != height
        || info.color_type != png::ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
        || info.interlaced
        || reader.output_buffer_size() != Some(expected)
    {
        return Err(invalid(
            "Material PNG dimensions/layout do not match bounded float payload",
        ));
    }
    let mut pixels = vec![0; expected];
    let info = reader.next_frame(&mut pixels).map_err(error)?;
    if info.buffer_size() != expected {
        return Err(invalid("Incomplete material sample payload"));
    }
    let mut samples = Vec::with_capacity(count);
    for chunk in pixels[..count * 48].chunks_exact(48) {
        let values: [f32; 12] = std::array::from_fn(|i| {
            f32::from_le_bytes(chunk[4 * i..4 * i + 4].try_into().unwrap())
        });
        samples.push(Sample {
            color_roughness: values[..4].try_into().unwrap(),
            response: values[4..8].try_into().unwrap(),
            emission: values[8..12].try_into().unwrap(),
        });
    }
    Arc::make_mut(p).fixed = Some(Fixed {
        resolution,
        samples: Arc::new(samples),
    });
    f.material.validate()?;
    Ok(f.material)
}
