//! Factored material response experiment. Does not replace geometry or visibility.
use super::*;
pub(super) const ROOT: &str = "artifacts/appearance-probe/combined-symbol";
const SIDE: usize = 128;
const HEADER: usize = 32;
pub(super) const ERROR: f64 = 0.5 / 1023. + 0.08 * 24. * 23. / (8. * 127. * 127.);
pub(super) fn hash(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ *b as u64).wrapping_mul(0x100000001b3)
    })
}
fn signature() -> u64 {
    hash(include_str!("triangle_scaling.wgsl").as_bytes())
}
fn response(x: f64, y: f64) -> [f64; 3] {
    [0.36, 0.29, 0.19].map(|c| c * (0.15 + 0.85 * x) + 0.08 * y.powi(24))
}
fn payload() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(SIDE * SIDE * 4);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let rgb = response(x as f64 / 127., y as f64 / 127.);
            let mut w = 3u32 << 30;
            for (i, c) in rgb.into_iter().enumerate() {
                w |= ((c * 1023.).round() as u32) << (i * 10);
            }
            bytes.extend_from_slice(&w.to_le_bytes());
        }
    }
    bytes
}
pub(super) fn compile() -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let start = Instant::now();
    let p = payload();
    let mut b = Vec::with_capacity(HEADER + p.len());
    b.extend_from_slice(b"IMS1");
    b.extend_from_slice(&1u32.to_le_bytes());
    b.extend_from_slice(&(SIDE as u32).to_le_bytes());
    b.extend_from_slice(&(p.len() as u32).to_le_bytes());
    b.extend_from_slice(&signature().to_le_bytes());
    b.extend_from_slice(&hash(&p).to_le_bytes());
    b.extend(p);
    validate(&b)?;
    fs::write(Path::new(ROOT).join("material.ims"), &b)?;
    fs::write(
        Path::new(ROOT).join("build.json"),
        serde_json::to_vec_pretty(
            &json!({"disk_bytes":b.len(),"gpu_payload_bytes":SIDE*SIDE*4,"build_ms":start.elapsed().as_secs_f64()*1000.,"checksum":hash(&b),"max_error_analytic":ERROR,"shader_signature":signature(),"format":"RGB10A2Unorm; alpha is opaque coverage","symbol_type":"material response only; geometry unchanged"}),
        )?,
    )?;
    Ok(())
}
fn validate(b: &[u8]) -> Result<&[u8]> {
    ensure!(b.len() == HEADER + SIDE * SIDE * 4, "invalid symbol length");
    ensure!(&b[..4] == b"IMS1", "symbol magic");
    let u32at = |i| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    let u64at = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    ensure!(
        u32at(4) == 1 && u32at(8) == 128 && u32at(12) == 65536,
        "symbol version/layout"
    );
    ensure!(
        u64at(16) == signature() && u64at(24) == hash(&b[HEADER..]),
        "stale/corrupt symbol"
    );
    ensure!(
        b[HEADER..]
            .chunks_exact(4)
            .all(|w| u32::from_le_bytes(w.try_into().unwrap()) >> 30 == 3),
        "opaque coverage"
    );
    Ok(&b[HEADER..])
}
pub(super) fn bucket(depth: bool, area: f64) -> Option<usize> {
    if !area.is_finite() || !(0.0..=1.00001).contains(&area) {
        return None;
    }
    Some(
        usize::from(depth) * 4
            + if area < 0.01 {
                0
            } else if area < 0.1 {
                1
            } else if area < 0.5 {
                2
            } else {
                3
            },
    )
}
pub(super) struct Pipelines {
    pub forward: wgpu::RenderPipeline,
    pub depth: wgpu::RenderPipeline,
    pub group: wgpu::BindGroup,
    pub enabled: u8,
}
impl Pipelines {
    pub fn allows(&self, depth: bool, area: f64) -> bool {
        bucket(depth, area).is_some_and(|i| self.enabled & (1 << i) != 0)
    }
    pub fn new(
        gpu: &Gpu,
        geometry: &wgpu::BindGroupLayout,
        depth: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        let bytes = fs::read(Path::new(ROOT).join("material.ims"))?;
        let p = validate(&bytes)?;
        let d = &gpu.device;
        let tex = texture(
            d,
            128,
            128,
            1,
            wgpu::TextureFormat::Rgb10a2Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            p,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(512),
                rows_per_image: Some(128),
            },
            wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
        );
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compiled material"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = d.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &tex.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3];
        let make = |deferred: bool| {
            let base = include_str!("triangle_scaling.wgsl");
            let start = base.find("@fragment fn fragment(").unwrap();
            let end = base[start..].find("// Diagnostic").unwrap() + start;
            let replacement = "@fragment fn fragment(s:Surface)->@location(0)vec4<f32>{var n=s.normal;if g.settings.x==3u{n=source_normal(s.world);}return symbol_material(n,s.world);}\n";
            let forward = format!("{}{}{}", &base[..start], replacement, &base[end..]);
            let appendix = if deferred {
                include_str!("scaling_depth.wgsl")
                    .replace("return material(", "return symbol_material(")
            } else {
                String::new()
            };
            let lookup = include_str!("material_symbol.wgsl")
                .replace("GROUP", if deferred { "2" } else { "1" });
            let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("compiled material response"),
                source: wgpu::ShaderSource::Wgsl(format!("{forward}\n{appendix}\n{lookup}").into()),
            });
            let layouts = if deferred {
                vec![geometry, depth, &layout]
            } else {
                vec![geometry, &layout]
            };
            let buffers = [wgpu::VertexBufferLayout {
                array_stride: 24,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attrs,
            }];
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("optional response symbol"),
                layout: Some(&d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                })),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(if deferred { "fullscreen" } else { "vertex" }),
                    compilation_options: Default::default(),
                    buffers: if deferred { &[] } else { &buffers },
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if deferred {
                        "shade_visible"
                    } else {
                        "fragment"
                    }),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: (!deferred).then_some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        Ok(Self {
            forward: make(false),
            depth: make(true),
            group,
            enabled: 0,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiled_response_bound_and_coverage() {
        let p = payload();
        let mut worst = 0f64;
        let at = |x: usize, y: usize, k: usize| {
            let i = (y * SIDE + x) * 4;
            let w = u32::from_le_bytes(p[i..i + 4].try_into().unwrap());
            assert_eq!(w >> 30, 3);
            ((w >> (k * 10)) & 1023) as f64 / 1023.
        };
        for y in 0..1025 {
            for x in 0..257 {
                let u = x as f64 / 256. * 127.;
                let v = y as f64 / 1024. * 127.;
                let a = (u.floor() as usize).min(126);
                let b = (v.floor() as usize).min(126);
                let fx = u - a as f64;
                let fy = v - b as f64;
                for (k, r) in response(u / 127., v / 127.).into_iter().enumerate() {
                    let s = (at(a, b, k) * (1. - fx) + at(a + 1, b, k) * fx) * (1. - fy)
                        + (at(a, b + 1, k) * (1. - fx) + at(a + 1, b + 1, k) * fx) * fy;
                    worst = worst.max((s - r).abs());
                }
            }
        }
        assert!(worst <= ERROR, "{worst}");
    }
    #[test]
    fn corrupt_material_is_rejected() {
        let p = payload();
        let mut b = b"IMS1".to_vec();
        for w in [1u32, 128, 65536] {
            b.extend(w.to_le_bytes());
        }
        b.extend(signature().to_le_bytes());
        b.extend(hash(&p).to_le_bytes());
        b.extend(p);
        assert!(validate(&b).is_ok());
        for i in [0, 4, 8, 12, 16, 24, 100, b.len() - 1] {
            let mut c = b.clone();
            c[i] ^= 1;
            assert!(validate(&c).is_err());
        }
        assert!(validate(&b[..b.len() - 1]).is_err());
    }
    #[test]
    fn invalid_selector_domain_falls_back() {
        assert_eq!(bucket(false, f64::NAN), None);
        assert_eq!(bucket(true, 1.1), None);
        assert_eq!(bucket(false, 0.01), Some(1));
        assert_eq!(bucket(true, 0.5), Some(7));
    }
}
