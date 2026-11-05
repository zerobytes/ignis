//! IGP1: source-bound signed-normalized slopes, uploaded without expansion.
use super::{compiler, experiment, material, render::Gpu, tangent};
use anyhow::{Result, ensure};
use std::{path::Path, time::Instant};
const MODES: [&str; 6] = [
    "implicit",
    "geometric-id",
    "geometric-fetch",
    "geometric-grad",
    "geometric-screen",
    "geometric-packed",
];
const HEADER: usize = 40;
fn pack(values: &[[f32; 2]]) -> Result<Vec<u32>> {
    ensure!(
        !values.is_empty() && values.iter().flatten().all(|v| v.is_finite()),
        "finite nonempty slopes required"
    );
    let maximum = values.iter().flatten().fold(0f32, |a, b| a.max(b.abs()));
    let scale = 2f32.powf(maximum.max(1.).log2().ceil());
    ensure!(
        scale.is_finite() && scale >= maximum,
        "slope scale overflow"
    );
    let quantize = |v: f32| {
        (f64::from(v) / f64::from(scale) * 32767.)
            .round()
            .clamp(-32767., 32767.) as i16 as u16 as u32
    };
    let mut words = Vec::with_capacity(values.len() + 1);
    words.push(scale.to_bits());
    words.extend(
        values
            .iter()
            .map(|v| quantize(v[0]) | (quantize(v[1]) << 16)),
    );
    Ok(words)
}
fn unpack(word: u32, scale: f32) -> [f32; 2] {
    [word as u16 as i16, (word >> 16) as u16 as i16]
        .map(|v| (f32::from(v) / 32767.).max(-1.) * scale)
}
fn encode(mesh: &compiler::Mesh, values: &[[f32; 2]]) -> Result<Vec<u8>> {
    ensure!(values.len() == mesh.faces.len(), "slope face count");
    let words = pack(values)?;
    let payload: Vec<u8> = words.iter().flat_map(|v| v.to_le_bytes()).collect();
    let mut b = b"IGP1".to_vec();
    b.extend(1u32.to_le_bytes());
    b.extend((mesh.faces.len() as u64).to_le_bytes());
    b.extend(compiler::hash(bytemuck::cast_slice(&mesh.vertices)).to_le_bytes());
    b.extend(compiler::hash(bytemuck::cast_slice(&mesh.faces)).to_le_bytes());
    b.extend(compiler::hash(&payload).to_le_bytes());
    b.extend(payload);
    Ok(b)
}
fn decode(b: &[u8], mesh: &compiler::Mesh) -> Result<Vec<u32>> {
    ensure!(
        b.len() >= HEADER + 8 && &b[..8] == b"IGP1\x01\0\0\0",
        "IGP1 header"
    );
    let u64_at = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    ensure!(
        u64_at(8) == mesh.faces.len() as u64 && b.len() == HEADER + 4 + mesh.faces.len() * 4,
        "IGP1 length"
    );
    ensure!(
        u64_at(16) == compiler::hash(bytemuck::cast_slice(&mesh.vertices))
            && u64_at(24) == compiler::hash(bytemuck::cast_slice(&mesh.faces)),
        "IGP1 source mismatch"
    );
    ensure!(u64_at(32) == compiler::hash(&b[HEADER..]), "IGP1 checksum");
    let words: Vec<u32> = b[HEADER..]
        .chunks_exact(4)
        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
        .collect();
    let scale = f32::from_bits(words[0]);
    ensure!(
        scale.is_finite() && scale >= 1. && words[0] & 0x007fffff == 0,
        "IGP1 power-of-two scale"
    );
    ensure!(
        words[1..]
            .iter()
            .all(|w| *w as u16 != 0x8000 && (*w >> 16) as u16 != 0x8000),
        "IGP1 canonical snorm"
    );
    Ok(words)
}
fn error(values: &[[f32; 2]], words: &[u32]) -> f32 {
    let scale = f32::from_bits(words[0]);
    values
        .iter()
        .zip(&words[1..])
        .flat_map(|(v, w)| {
            v.iter()
                .copied()
                .zip(unpack(*w, scale))
                .map(|(a, b)| (a - b).abs())
        })
        .fold(0., f32::max)
}
pub fn build(dir: &Path) -> Result<()> {
    let mesh = tangent::source(dir)?;
    let values = tangent::decode(&std::fs::read(dir.join("source.igt"))?, &mesh)?;
    let start = Instant::now();
    let bytes = encode(&mesh, &values)?;
    let words = decode(&bytes, &mesh)?;
    let scale = f32::from_bits(words[0]);
    let maximum_error = error(&values, &words);
    ensure!(
        maximum_error <= scale / 65534. + scale * f32::EPSILON,
        "IGP1 quantization bound"
    );
    std::fs::write(dir.join("source.igp"), &bytes)?;
    println!(
        "IGP1 {} faces, {} bytes, scale {}, max error {}, {:.3}s compile+verify",
        mesh.faces.len(),
        bytes.len(),
        scale,
        maximum_error,
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
pub fn run(dir: &Path) -> Result<()> {
    let mesh = tangent::source(dir)?;
    let raw = std::fs::read(dir.join("source.igt"))?;
    let compiled = std::fs::read(dir.join("source.igp"))?;
    let words = decode(&compiled, &mesh)?;
    let values = tangent::decode(&raw, &mesh)?;
    let color = material::decode(&std::fs::read(dir.join("color.irm"))?)?;
    let normal = material::decode(&std::fs::read(dir.join("normal.irm"))?)?;
    let texture_bytes: usize = color
        .levels
        .iter()
        .chain(&normal.levels)
        .map(Vec::len)
        .sum();
    let full = vec![(0, mesh.faces.len() as u32 * 3)];
    let mut quality = vec![];
    let mut returns = vec![];
    let mut repeated = vec![];
    let mut timings = vec![];
    let mut adapters = vec![];
    let mut geometry_bytes = 0;
    for (width, height) in [(1728, 1117), (3456, 2234)] {
        let mut gpu = pollster::block_on(Gpu::new_features(
            width,
            height,
            wgpu::Features::SHADER_PRIMITIVE_INDEX,
        ))?;
        gpu.install_rich_material(color.size, &color.levels, normal.size, &normal.levels);
        gpu.install_rich_slopes(&values);
        let mut variants = MODES[..5]
            .iter()
            .map(|m| gpu.rich_variant(m))
            .collect::<Vec<_>>();
        gpu.install_rich_packed_slopes(&words);
        variants.push(gpu.rich_variant(MODES[5]));
        let uploaded = gpu.upload_rich(&mesh);
        geometry_bytes = uploaded.bytes;
        adapters.push(gpu.adapter.clone());
        // Alternate cached pipelines, no diagnostic MRT and no texture reuploads.
        for case in [0, 3] {
            let p = experiment::pose(case, 0, 8);
            let mut first = Vec::new();
            for variant in &variants {
                gpu.select_rich_variant(variant);
                first.push(gpu.capture_selected(&uploaded, p, Some(&full))?);
            }
            for turn in 0..32 {
                for mode in if turn % 2 == 0 {
                    [0, 1, 2, 3, 4, 5]
                } else {
                    [5, 4, 3, 2, 1, 0]
                } {
                    gpu.select_rich_variant(&variants[mode]);
                    let pixels = gpu.capture_selected(&uploaded, p, Some(&full))?;
                    let changed = pixels
                        .iter()
                        .zip(&first[mode])
                        .filter(|(a, b)| a != b)
                        .count();
                    repeated.push(serde_json::json!({"width":width,"case":case,"turn":turn,"mode":MODES[mode],"changed_pixels":changed,"first_hash":compiler::hash(bytemuck::cast_slice(&first[mode])).to_string(),"hash":compiler::hash(bytemuck::cast_slice(&pixels)).to_string(),"max_delta":first[mode].iter().flatten().zip(pixels.iter().flatten()).map(|(a,b)|(a-b).abs()).fold(0f32,f32::max)}));
                }
            }
        }
        for case in 0..5 {
            let steps = if case == 4 { 32 } else { 8 };
            let mut first = [0; 6];
            for frame in 0..=steps {
                let p = experiment::pose(case, frame, steps);
                let mut captures = Vec::new();
                for variant in &variants {
                    gpu.select_rich_variant(variant);
                    captures.push(gpu.capture_selected(&uploaded, p, Some(&full))?);
                }
                for (mode, pixels) in captures.iter().enumerate() {
                    let hash = compiler::hash(bytemuck::cast_slice(pixels));
                    if frame == 0 {
                        first[mode] = hash;
                    }
                    if frame == steps {
                        returns.push(serde_json::json!({"width":width,"case":case,"mode":MODES[mode],"pass":first[mode]==hash,"first_hash":first[mode].to_string(),"return_hash":hash.to_string()}));
                    }
                    if mode != 1 && mode != 2 {
                        experiment::png(
                            &dir.join(format!(
                                "compact-{width}-{case}-{frame}-{}.png",
                                MODES[mode]
                            )),
                            pixels,
                            width,
                            height,
                        )?;
                    }
                }
                let mut row = serde_json::json!({"width":width,"height":height,"case":case,"frame":frame,"distance":p.distance});
                for mode in 1..6 {
                    row[MODES[mode]] =
                        serde_json::to_value(experiment::quality(&captures[0], &captures[mode]))?;
                }
                row["projection_only"] =
                    serde_json::to_value(experiment::quality(&captures[3], &captures[4]))?;
                row["quantization_only"] =
                    serde_json::to_value(experiment::quality(&captures[4], &captures[5]))?;
                quality.push(row);
            }
            let poses = (0..32)
                .map(|i| experiment::pose(case, i, 32))
                .collect::<Vec<_>>();
            for round in 0..8 {
                for (slot, mode) in (if round % 2 == 0 {
                    [0, 1, 2, 3, 4, 5, 5, 4, 3, 2, 1, 0]
                } else {
                    [5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5]
                })
                .into_iter()
                .enumerate()
                {
                    gpu.select_rich_variant(&variants[mode]);
                    let _ = gpu.batch_selected(&uploaded, &poses, |_| full.clone())?;
                    let timing = gpu.batch_selected(&uploaded, &poses, |_| full.clone())?;
                    timings.push(serde_json::json!({"width":width,"case":case,"round":round,"slot":slot,"mode":MODES[mode],"timing":timing}));
                }
            }
            println!("compact {width} case{case} complete");
        }
    }
    let data = serde_json::json!({"adapters":adapters,"source_triangles":mesh.faces.len(),"geometry_bytes":geometry_bytes,"texture_bytes":texture_bytes,"slopes_bytes":values.len()*8,"packed_bytes":words.len()*4,"quantization_max_error":error(&values,&words),"scale":f32::from_bits(words[0]),"compiled_file_bytes":compiled.len(),"payload_hash":compiler::hash(&compiled).to_string(),"quality":quality,"returns":returns,"repeated":repeated,"timings":timings});
    std::fs::write(
        dir.join("compact-results.json"),
        serde_json::to_vec_pretty(&data)?,
    )?;
    ensure!(
        returns.iter().all(|r| r["pass"] == true)
            && repeated.iter().all(|r| r["changed_pixels"] == 0),
        "compact return gate failed; data retained"
    );
    ensure!(
        quality
            .iter()
            .all(|r| MODES[1..].iter().all(|m| r[*m]["pass"] == true)),
        "compact quality gate failed; data retained"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snorm_roundtrip_and_bound() {
        let values: Vec<_> = (-10000..=10000)
            .map(|i| [i as f32 * 0.0006667, -i as f32 * 0.0003333])
            .collect();
        let words = pack(&values).unwrap();
        assert_eq!(f32::from_bits(words[0]), 8.);
        assert!(error(&values, &words) <= 8. / 65534. + 8. * f32::EPSILON);
        let w = pack(&[[-8., 8.], [0., 0.]]).unwrap();
        assert_eq!(unpack(w[1], 8.), [-8., 8.]);
        assert_eq!(unpack(w[2], 8.), [0., 0.]);
        assert!(pack(&[]).is_err());
        assert!(pack(&[[f32::NAN, 0.]]).is_err());
        assert!(pack(&[[f32::MAX, 0.]]).is_err());
    }
    #[test]
    fn codec_rejects_corruption_stale_source_and_invalid_scale() {
        let mesh = compiler::Mesh {
            vertices: vec![compiler::Vertex {
                p: [0.; 3],
                n: [0.; 3],
            }],
            faces: vec![[0; 4]],
        };
        let b = encode(&mesh, &[[2., -3.]]).unwrap();
        assert_eq!(decode(&b, &mesh).unwrap(), pack(&[[2., -3.]]).unwrap());
        for i in [0, 4, 8, 16, 24, 32, 40, 44] {
            let mut bad = b.clone();
            bad[i] ^= 1;
            assert!(decode(&bad, &mesh).is_err());
        }
        for n in 0..b.len() {
            assert!(decode(&b[..n], &mesh).is_err());
        }
        let mut other = mesh.clone();
        other.vertices[0].p[0] = 1.;
        assert!(decode(&b, &other).is_err());
        for scale in [0., -1., 3., f32::NAN, f32::INFINITY] {
            let mut bad = b.clone();
            bad[40..44].copy_from_slice(&scale.to_le_bytes());
            let h = compiler::hash(&bad[40..]);
            bad[32..40].copy_from_slice(&h.to_le_bytes());
            assert!(decode(&bad, &mesh).is_err());
        }
    }
    #[test]
    fn screen_projection_matches_plane_jacobian() {
        use glam::{DMat4, DVec3, DVec4};
        for width in [1728., 3456.] {
            for distance in [2.4, 6., 16.] {
                for angle in [0.1f64, 0.8, 2.5] {
                    for slope in [[0., 0.], [6.5, -6.5], [-2., 3.]] {
                        let matrix =
                            DMat4::perspective_rh(0.8, width / (width * 1117. / 1728.), 0.01, 100.)
                                * DMat4::look_at_rh(
                                    DVec3::new(
                                        angle.cos() * distance,
                                        angle.sin() * distance,
                                        distance,
                                    ),
                                    DVec3::ZERO,
                                    DVec3::Z,
                                );
                        let clip = matrix * DVec4::new(0.2, -0.1, 0.05, 1.);
                        let ndc = clip.truncate() / clip.w;
                        let pixel = [
                            (ndc.x + 1.) * width / 2.,
                            (1. - ndc.y) * (width * 1117. / 1728.) / 2.,
                            1. / clip.w,
                        ];
                        let recovered = glam::DVec2::new(
                            pixel[0] * 2. / width - 1.,
                            1. - pixel[1] * 2. / (width * 1117. / 1728.),
                        );
                        assert!((recovered - ndc.truncate()).length() < 1e-12);
                        for (axis, s) in [(0, slope[0]), (1, slope[1])] {
                            let direction = if axis == 0 {
                                DVec4::new(1., 0., s, 0.)
                            } else {
                                DVec4::new(0., 1., s, 0.)
                            };
                            let a = matrix * direction;
                            let b = matrix.col(axis) + matrix.z_axis * s;
                            let old = (a.truncate().truncate() - ndc.truncate() * a.w) / clip.w;
                            let new = (b.truncate().truncate() - recovered * b.w) * pixel[2];
                            assert!((old - new).length() < 1e-12);
                        }
                    }
                }
            }
        }
    }
}
