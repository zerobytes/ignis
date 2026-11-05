//! Conservative regional height intervals and directional derivative certificates.
use super::*;
use material_symbol::hash;
pub(super) const ROOT: &str = "artifacts/appearance-probe/hierarchical-field";
pub(super) const ROUNDS: u32 = 4;
const LEAF: u32 = 8;
const HEADER: usize = 56;
#[derive(Clone)]
pub(super) struct Certificate {
    pub slopes: [[f32; 4]; 64],
}
impl Certificate {
    pub fn rho(&self, field: &geometric_field::Field, c: &Camera) -> f32 {
        let mut worst = f32::NEG_INFINITY;
        for (i, g) in self.slopes.iter().enumerate() {
            let x = -6. + (i % 8) as f32 * 1.5;
            let z = -6. + (i / 8) as f32 * 1.5;
            let contribution = |lo: f32, hi: f32, eye: f32, g0: f32, g1: f32| {
                [
                    -g0 * (lo - eye),
                    -g0 * (hi - eye),
                    -g1 * (lo - eye),
                    -g1 * (hi - eye),
                ]
                .into_iter()
                .fold(f32::NEG_INFINITY, f32::max)
            };
            let v = contribution(x, x + 1.5, c.eye.x, g[0], g[1])
                + contribution(z, z + 1.5, c.eye.z, g[2], g[3]);
            worst = worst.max(v / (c.eye.y - field.limits[1]));
        }
        worst
    }
    pub fn eligible(&self, field: &geometric_field::Field, c: &Camera) -> bool {
        c.eye.y > field.limits[1] + 0.1 && self.rho(field, c) < 0.95
    }
}
pub(super) struct Hierarchy {
    pub size: [u32; 2],
    pub mips: Vec<Vec<[f32; 2]>>,
    pub certificate: Certificate,
}
fn dimensions(grid: [u32; 2]) -> [u32; 2] {
    grid.map(|n| n.div_ceil(LEAF).next_power_of_two())
}
fn build(f: &geometric_field::Field) -> Hierarchy {
    let [nx, nz] = f.grid;
    let size = dimensions(f.grid);
    let mut leaves = vec![[f32::MAX, -f32::MAX]; (size[0] * size[1]) as usize];
    let mut slopes = [[f32::MAX, -f32::MAX, f32::MAX, -f32::MAX]; 64];
    let h = |x: u32, z: u32| f.heights[(z * (nx + 1) + x) as usize];
    let pos = |i: u32, n: u32| ((i as f32 / n as f32 - 0.5) * 12.) as f64;
    for z in 0..nz {
        for x in 0..nx {
            let hs = [h(x, z), h(x + 1, z), h(x, z + 1), h(x + 1, z + 1)];
            let b = &mut leaves[((z / LEAF) * size[0] + x / LEAF) as usize];
            for y in hs {
                b[0] = b[0].min(y);
                b[1] = b[1].max(y);
            }
            let gx = [
                (hs[1] as f64 - hs[0] as f64) / (pos(x + 1, nx) - pos(x, nx)),
                (hs[3] as f64 - hs[2] as f64) / (pos(x + 1, nx) - pos(x, nx)),
            ];
            let gz = [
                (hs[2] as f64 - hs[0] as f64) / (pos(z + 1, nz) - pos(z, nz)),
                (hs[3] as f64 - hs[1] as f64) / (pos(z + 1, nz) - pos(z, nz)),
            ];
            let s = &mut slopes[((z * 8 / nz) * 8 + x * 8 / nx) as usize];
            for v in gx {
                s[0] = s[0].min((v as f32).next_down());
                s[1] = s[1].max((v as f32).next_up());
            }
            for v in gz {
                s[2] = s[2].min((v as f32).next_down());
                s[3] = s[3].max((v as f32).next_up());
            }
        }
    }
    // Expand once for f32 arithmetic; parents are exact min/max unions.
    for b in &mut leaves {
        if b[0] <= b[1] {
            b[0] = (b[0] - 0.000002).next_down();
            b[1] = (b[1] + 0.000002).next_up();
        }
    }
    let mut mips = vec![leaves];
    let mut wh = size;
    while wh != [1, 1] {
        let next = wh.map(|n| (n / 2).max(1));
        let child = mips.last().unwrap();
        let mut parent = vec![[f32::MAX, -f32::MAX]; (next[0] * next[1]) as usize];
        for z in 0..next[1] {
            for x in 0..next[0] {
                let out = &mut parent[(z * next[0] + x) as usize];
                for dz in 0..2 {
                    for dx in 0..2 {
                        let b = child[((z * 2 + dz).min(wh[1] - 1) * wh[0]
                            + (x * 2 + dx).min(wh[0] - 1))
                            as usize];
                        out[0] = out[0].min(b[0]);
                        out[1] = out[1].max(b[1]);
                    }
                }
            }
        }
        mips.push(parent);
        wh = next;
    }
    Hierarchy {
        size,
        mips,
        certificate: Certificate { slopes },
    }
}
fn encode(h: &Hierarchy, grid: [u32; 2], source: u64) -> Vec<u8> {
    let mut payload = bytemuck::cast_slice(&h.certificate.slopes).to_vec();
    for mip in &h.mips {
        payload.extend(bytemuck::cast_slice(mip));
    }
    let mut b = b"IHF1".to_vec();
    for word in [
        1,
        grid[0],
        grid[1],
        h.size[0],
        h.size[1],
        h.mips.len() as u32,
        64,
    ] {
        b.extend(word.to_le_bytes());
    }
    b.extend(source.to_le_bytes());
    b.extend(hash(&payload).to_le_bytes());
    b.extend(hash(&b).to_le_bytes());
    b.extend(payload);
    b
}
fn decode(b: &[u8], f: &geometric_field::Field, source: u64) -> Result<Hierarchy> {
    ensure!(b.len() >= HEADER && &b[..4] == b"IHF1", "hierarchy header");
    let word = |i| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    let long = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    let size = dimensions(f.grid);
    let levels = size[0].max(size[1]).ilog2() + 1;
    ensure!(
        word(4) == 1
            && [word(8), word(12)] == f.grid
            && [word(16), word(20)] == size
            && word(24) == levels
            && word(28) == 64,
        "hierarchy layout"
    );
    ensure!(
        long(32) == source && long(40) == hash(&b[HEADER..]) && long(48) == hash(&b[..48]),
        "stale/corrupt hierarchy"
    );
    let mut count = 64 * 16;
    for l in 0..levels {
        count += ((size[0] >> l).max(1) * (size[1] >> l).max(1) * 8) as usize;
    }
    ensure!(b.len() == HEADER + count, "hierarchy length");
    let values: Vec<f32> = b[HEADER..]
        .chunks_exact(4)
        .map(|w| f32::from_le_bytes(w.try_into().unwrap()))
        .collect();
    ensure!(values.iter().all(|v| v.is_finite()), "nonfinite hierarchy");
    let slopes = std::array::from_fn(|i| values[i * 4..i * 4 + 4].try_into().unwrap());
    ensure!(
        slopes
            .iter()
            .all(|s: &[f32; 4]| s[0] <= s[1] && s[2] <= s[3]),
        "invalid derivatives"
    );
    let mut offset = 256;
    let mut mips = vec![];
    for l in 0..levels {
        let n = ((size[0] >> l).max(1) * (size[1] >> l).max(1)) as usize;
        mips.push(
            values[offset..offset + n * 2]
                .chunks_exact(2)
                .map(|a| [a[0], a[1]])
                .collect(),
        );
        offset += n * 2;
    }
    Ok(Hierarchy {
        size,
        mips,
        certificate: Certificate { slopes },
    })
}
pub(super) fn compile() -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let mut report = vec![];
    for d in 0..4 {
        let start = Instant::now();
        let id = format!("f1_d{d}");
        let source = fs::read(Path::new(geometric_field::ROOT).join(format!("{id}.igf")))?;
        let f = geometric_field::decode(&source)?;
        ensure!(
            f.grid.iter().all(|n| *n >= 8 && *n % 8 == 0),
            "hierarchy certificate requires grid divisible by8"
        );
        let h = build(&f);
        let b = encode(&h, f.grid, hash(&source));
        decode(&b, &f, hash(&source))?;
        fs::write(Path::new(ROOT).join(format!("{id}.ihf")), &b)?;
        report.push(json!({"id":id,"hierarchy_disk_bytes":b.len(),"hierarchy_gpu_bytes":h.mips.iter().map(|m|m.len()*8).sum::<usize>(),"certificate_cpu_bytes":1024,"height_gpu_bytes":f.heights.len()*4,"total_symbol_gpu_bytes":f.heights.len()*4+h.mips.iter().map(|m|m.len()*8).sum::<usize>(),"levels":h.mips.len(),"size":h.size,"leaf_cells":8,"build_ms":start.elapsed().as_secs_f64()*1000.}));
    }
    fs::write(
        Path::new(ROOT).join("build.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
pub(super) fn upload(
    gpu: &Gpu,
    id: &str,
    f: &geometric_field::Field,
) -> Result<(wgpu::Texture, Certificate)> {
    let source = fs::read(Path::new(geometric_field::ROOT).join(format!("{id}.igf")))?;
    let h = decode(
        &fs::read(Path::new(ROOT).join(format!("{id}.ihf")))?,
        f,
        hash(&source),
    )?;
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("compiled height intervals"),
        size: wgpu::Extent3d {
            width: h.size[0],
            height: h.size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: h.mips.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rg32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (level, mip) in h.mips.iter().enumerate() {
        let [w, height] = h.size.map(|n| (n >> level).max(1));
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: level as u32,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(mip),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 8),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width: w,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
    Ok((tex, h.certificate))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> geometric_field::Field {
        geometric_field::Field {
            grid: [16, 24],
            limits: [-1., 1., 1., 1.],
            heights: (0..25)
                .flat_map(|z| (0..17).map(move |x| ((x * 17 + z * 11) as f32).sin() * 0.7))
                .collect(),
        }
    }
    #[test]
    fn every_leaf_and_parent_contains_descendants() {
        let f = fixture();
        let h = build(&f);
        for z in 0..f.grid[1] {
            for x in 0..f.grid[0] {
                let b = h.mips[0][((z / 8) * h.size[0] + x / 8) as usize];
                for dz in 0..2 {
                    for dx in 0..2 {
                        let y = f.heights[((z + dz) * (f.grid[0] + 1) + x + dx) as usize];
                        assert!(b[0] <= y && b[1] >= y);
                    }
                }
            }
        }
        for l in 1..h.mips.len() {
            let prev = h.size.map(|n| (n >> (l - 1)).max(1));
            let size = h.size.map(|n| (n >> l).max(1));
            for z in 0..prev[1] {
                for x in 0..prev[0] {
                    let c = h.mips[l - 1][(z * prev[0] + x) as usize];
                    let p = h.mips[l][((z / 2) * size[0] + x / 2) as usize];
                    assert!(p[0] <= c[0] && p[1] >= c[1]);
                }
            }
        }
    }
    #[test]
    fn hierarchy_roundtrip_corruption_and_source_binding() {
        let f = fixture();
        let h = build(&f);
        let b = encode(&h, f.grid, 42);
        assert_eq!(decode(&b, &f, 42).unwrap().mips, h.mips);
        assert!(decode(&b, &f, 43).is_err());
        for i in [0, 4, 8, 16, 24, 32, 40, 48, 56, b.len() - 1] {
            let mut c = b.clone();
            c[i] ^= 1;
            assert!(decode(&c, &f, 42).is_err());
        }
        assert!(decode(&b[..b.len() - 1], &f, 42).is_err());
    }
    #[test]
    fn certificate_bounds_directional_derivatives() {
        let f = fixture();
        let h = build(&f);
        let mut c = render::camera_at(20., [100, 100]);
        c.eye = Vec3::new(3., 100., -4.);
        let rho = h.certificate.rho(&f, &c);
        assert!(h.certificate.eligible(&f, &c));
        for (i, g) in h.certificate.slopes.iter().enumerate() {
            for gx in [g[0], g[1]] {
                for gz in [g[2], g[3]] {
                    for x in [-6. + (i % 8) as f32 * 1.5, -4.5 + (i % 8) as f32 * 1.5] {
                        for z in [-6. + (i / 8) as f32 * 1.5, -4.5 + (i / 8) as f32 * 1.5] {
                            assert!(
                                (-gx * (x - c.eye.x) - gz * (z - c.eye.z))
                                    / (c.eye.y - f.limits[1])
                                    <= rho + 1e-6
                            );
                        }
                    }
                }
            }
        }
    }
}
