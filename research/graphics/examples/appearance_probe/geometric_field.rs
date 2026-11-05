//! Exact triangular heightfield data; monotone-ray domain, no view/color atlas.
use super::*;
pub(super) const ROOT: &str = "artifacts/appearance-probe/geometric-field";
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
const HEADER: usize = 64;
use material_symbol::hash;
#[derive(Clone)]
pub(super) struct Field {
    pub grid: [u32; 2],
    pub limits: [f32; 4], // min/max height, maximum absolute x/z derivatives
    pub heights: Vec<f32>,
}
impl Field {
    pub fn rho(&self, c: &Camera) -> f32 {
        (self.limits[2] * (c.eye.x.abs() + 6.) + self.limits[3] * (c.eye.z.abs() + 6.))
            / (c.eye.y - self.limits[1])
    }
    pub fn eligible(&self, c: &Camera) -> bool {
        c.eye.y > self.limits[1] + 0.1 && self.rho(c) < 0.95
    }
}
pub(super) fn decode(b: &[u8]) -> Result<Field> {
    ensure!(
        b.len() >= HEADER && &b[..4] == b"IGF1",
        "geometric symbol header"
    );
    let word = |i| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    let grid = [word(8), word(12)];
    ensure!(
        word(4) == 1 && grid.iter().all(|n| (1..=4096).contains(n)),
        "geometric symbol dimensions/version"
    );
    let count = (grid[0] as usize + 1) * (grid[1] as usize + 1);
    ensure!(b.len() == HEADER + count * 4, "geometric symbol length");
    ensure!(
        u64::from_le_bytes(b[32..40].try_into().unwrap()) == hash(&b[HEADER..])
            && u64::from_le_bytes(b[40..48].try_into().unwrap()) == hash(&b[..40]),
        "geometric symbol checksum"
    );
    ensure!(b[48..64] == [0; 16], "reserved header");
    let limits = std::array::from_fn(|i| f32::from_bits(word(16 + i * 4)));
    ensure!(
        limits.iter().all(|v| v.is_finite())
            && limits[0] <= limits[1]
            && limits[2] >= 0.
            && limits[3] >= 0.,
        "geometric limits"
    );
    let heights: Vec<f32> = b[HEADER..]
        .chunks_exact(4)
        .map(|w| f32::from_le_bytes(w.try_into().unwrap()))
        .collect();
    ensure!(
        heights
            .iter()
            .all(|h| h.is_finite() && *h >= limits[0] && *h <= limits[1]),
        "geometric heights"
    );
    Ok(Field {
        grid,
        limits,
        heights,
    })
}
pub(super) fn compile() -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let mut report = vec![];
    for density in 0..4 {
        let start = Instant::now();
        let id = format!("f1_d{density}");
        let root = Path::new(SOURCE);
        let a: Asset = serde_json::from_slice(&fs::read(root.join(format!("{id}.json")))?)?;
        let vb = fs::read(root.join(format!("{id}.vertices")))?;
        let ib = fs::read(root.join(format!("{id}.indices")))?;
        let vertices: &[build::V] = bytemuck::cast_slice(&vb);
        let indices: &[u32] = bytemuck::cast_slice(&ib);
        let level = a.levels.last().unwrap();
        let base = indices[level.first as usize] as usize;
        let [nx, nz] = a.grid.map(|x| x as usize);
        let source = &vertices[base..base + (nx + 1) * (nz + 1)];
        let mut payload = Vec::with_capacity(source.len() * 4);
        let mut bounds = [f32::INFINITY, f32::NEG_INFINITY, 0f32, 0f32];
        for z in 0..=nz {
            for x in 0..=nx {
                let p = source[z * (nx + 1) + x].p;
                ensure!(
                    p[0] == (x as f32 / nx as f32 - 0.5) * 12.
                        && p[2] == (z as f32 / nz as f32 - 0.5) * 12.,
                    "not a uniform heightfield"
                );
                bounds[0] = bounds[0].min(p[1]);
                bounds[1] = bounds[1].max(p[1]);
                payload.extend(p[1].to_le_bytes());
                if x < nx {
                    let q = source[z * (nx + 1) + x + 1].p;
                    bounds[2] = bounds[2].max(
                        ((q[1] as f64 - p[1] as f64) / (q[0] as f64 - p[0] as f64)).abs() as f32,
                    );
                }
                if z < nz {
                    let q = source[(z + 1) * (nx + 1) + x].p;
                    bounds[3] = bounds[3].max(
                        ((q[1] as f64 - p[1] as f64) / (q[2] as f64 - p[2] as f64)).abs() as f32,
                    );
                }
                if x < nx && z < nz {
                    let k = (z * nx + x) * 6 + level.first as usize;
                    let i = (base + z * (nx + 1) + x) as u32;
                    let step = nx as u32 + 1;
                    ensure!(
                        indices[k..k + 6] == [i, i + step, i + 1, i + 1, i + step, i + step + 1],
                        "unsupported diagonal/topology"
                    );
                }
            }
        }
        bounds[2] = bounds[2].next_up();
        bounds[3] = bounds[3].next_up();
        let mut b = b"IGF1".to_vec();
        b.extend(1u32.to_le_bytes());
        for n in a.grid {
            b.extend(n.to_le_bytes());
        }
        for n in bounds {
            b.extend(n.to_le_bytes());
        }
        b.extend(hash(&payload).to_le_bytes());
        b.extend(hash(&b).to_le_bytes());
        b.resize(HEADER, 0);
        b.extend(payload);
        decode(&b)?;
        fs::write(Path::new(ROOT).join(format!("{id}.igf")), &b)?;
        report.push(json!({"id":id,"disk_bytes":b.len(),"gpu_bytes":b.len()-HEADER,"normal_bytes_shared":source.len()*16,"source_full_geometry_bytes":source.len()*24+level.count as usize*4,"baseline_asset_bytes":a.bytes,"bounds":bounds,"build_ms":start.elapsed().as_secs_f64()*1000.,"hash":hash(&b)}));
    }
    fs::write(
        Path::new(ROOT).join("build.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    inverse: [[f32; 4]; 4],
    bounds: [f32; 4],
    size: [f32; 4],
}
pub(super) struct Data {
    pub field: Field,
    pub certificate: Option<hierarchical_field::Certificate>,
    pub group: wgpu::BindGroup,
    pub tile_group: Option<wgpu::BindGroup>,
}
pub(super) struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub flat_pipeline: Option<wgpu::RenderPipeline>,
    pub diagnostic: Option<wgpu::RenderPipeline>,
    pub flat_diagnostic: Option<wgpu::RenderPipeline>,
    pub hierarchical: bool,
    pub tiled_pipeline: Option<wgpu::RenderPipeline>,
    pub tiles: Option<tiled_field::Tiles>,
    pub hierarchy_rounds: u32,
    layout: wgpu::BindGroupLayout,
    params: wgpu::Buffer,
    pub enabled: u8,
}
impl Pipeline {
    pub fn new(gpu: &Gpu, geometry: &wgpu::BindGroupLayout) -> Self {
        Self::create(gpu, geometry, false, false)
    }
    pub fn new_hierarchical(gpu: &Gpu, geometry: &wgpu::BindGroupLayout) -> Self {
        Self::create(gpu, geometry, true, false)
    }
    pub fn new_tiled(gpu: &Gpu, geometry: &wgpu::BindGroupLayout) -> Self {
        Self::create(gpu, geometry, true, true)
    }
    fn create(
        gpu: &Gpu,
        geometry: &wgpu::BindGroupLayout,
        hierarchical: bool,
        tiled: bool,
    ) -> Self {
        let d = &gpu.device;
        let mut entries = Vec::from([
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]);
        if hierarchical {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
        }
        if tiled {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }
        let tiles = tiled.then(|| tiled_field::Tiles::new(gpu));
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("field data"),
            entries: &entries,
        });
        let params = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let make = |hierarchy: bool, diagnostic: bool, tiled: bool| {
            let mut field = include_str!("geometric_field.wgsl").to_owned();
            if hierarchy {
                field=field.replace("    var left=origin+ray*begin;", "    let narrowed=narrow_interval(origin,ray,vec2(begin,end)); begin=narrowed.x;end=narrowed.y;if begin>end {discard;}
    var left=origin+ray*begin;");
            }
            if tiled {
                field.push_str(
                    "\n@group(1) @binding(3) var<storage,read> tile_intervals:array<vec2<f32>>;\n",
                );
                field=field.replace("    var left=origin+ray*begin;", "    let tile_xy=vec2<u32>(screen.xy)/u32(field.size.w); let tile_width=u32(ceil(field.size.x/field.size.w));\n    let heights=tile_intervals[tile_xy.y*tile_width+tile_xy.x];\n    let slack=max(abs(begin),abs(end))*.000001;\n    begin=max(begin,(heights.y-origin.y)/ray.y-slack); end=min(end,(heights.x-origin.y)/ray.y+slack);\n    if begin>end {discard;}\n    var left=origin+ray*begin;");
            }
            if diagnostic {
                field = field.replace(
                    "    for(var iteration=0u;",
                    "    var fine_steps=0u;
    for(var iteration=0u;",
                );
                field = field.replace(
                    "        world=origin+ray*t;",
                    "        fine_steps+=1u;
        world=origin+ray*t;",
                );
                field = field.replace(
                    "out.color=material(source_normal(world),world);",
                    if hierarchy {
                        "out.color=vec4(f32(fine_steps),narrowed.z,1.,1.);"
                    } else {
                        "out.color=vec4(f32(fine_steps),0.,1.,1.);"
                    },
                );
            }
            let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("field variant"),
                source: wgpu::ShaderSource::Wgsl(
                    format!(
                        "{}\n{}\n{}",
                        include_str!("triangle_scaling.wgsl"),
                        field,
                        if hierarchy {
                            include_str!("hierarchical_field.wgsl")
                        } else {
                            ""
                        }
                    )
                    .into(),
                ),
            });
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("geometric field"),
                layout: Some(&d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[geometry, &layout],
                    push_constant_ranges: &[],
                })),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("field_vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("field_fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
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
        let pipeline = make(hierarchical, false, false);
        let flat_pipeline = hierarchical.then(|| make(false, false, false));
        let diagnostic = hierarchical.then(|| make(true, true, false));
        let flat_diagnostic = hierarchical.then(|| make(false, true, false));
        let tiled_pipeline = tiled.then(|| make(false, false, true));
        Self {
            tiled_pipeline,
            tiles,
            pipeline,
            flat_pipeline,
            diagnostic,
            flat_diagnostic,
            hierarchical,
            hierarchy_rounds: hierarchical_field::ROUNDS,
            layout,
            params,
            enabled: 0,
        }
    }
    pub fn upload(&self, gpu: &Gpu, id: &str) -> Result<Data> {
        let field = decode(&fs::read(Path::new(ROOT).join(format!("{id}.igf")))?)?;
        let [w, h] = field.grid.map(|n| n + 1);
        let tex = texture(
            &gpu.device,
            w,
            h,
            1,
            wgpu::TextureFormat::R32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&field.heights),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        let hierarchy = if self.hierarchical {
            Some(hierarchical_field::upload(gpu, id, &field)?)
        } else {
            None
        };
        let view = tex.create_view(&Default::default());
        let hierarchy_view = hierarchy
            .as_ref()
            .map(|h| h.0.create_view(&Default::default()));
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: self.params.as_entire_binding(),
            },
        ];
        if let Some(view) = &hierarchy_view {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }
        if let Some(tiles) = &self.tiles {
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: tiles.scratch.as_entire_binding(),
            });
        }
        let tile_group = self
            .tiles
            .as_ref()
            .map(|t| t.group(gpu, hierarchy_view.as_ref().unwrap()));
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &entries,
        });
        // CPU height payload is only needed for upload, not retained in the frame loop.
        Ok(Data {
            tile_group,
            certificate: hierarchy.map(|h| h.1),
            field: Field {
                heights: vec![],
                ..field
            },
            group,
        })
    }
    pub fn prepare(&self, gpu: &Gpu, data: &Data, c: &Camera) {
        gpu.queue.write_buffer(
            &self.params,
            0,
            bytemuck::bytes_of(&Params {
                inverse: (c.view.inverse()
                    * Mat4::from_cols(
                        glam::Vec4::new(
                            30f32.to_radians().tan() * c.size[0] as f32 / c.size[1] as f32,
                            0.,
                            0.,
                            0.,
                        ),
                        glam::Vec4::new(0., 30f32.to_radians().tan(), 0., 0.),
                        glam::Vec4::ZERO,
                        glam::Vec4::new(0., 0., -1., 0.),
                    ))
                .to_cols_array_2d(),
                bounds: data.field.limits,
                size: [
                    c.size[0] as f32,
                    c.size[1] as f32,
                    self.hierarchy_rounds as f32,
                    self.tiles.as_ref().map_or(0., |t| t.edge as f32),
                ],
            }),
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_binary() {
        assert!(decode(&[]).is_err());
        assert!(decode(&[0; 64]).is_err());
    }
    #[test]
    fn binary_roundtrip_and_corruption() {
        let mut b = b"IGF1".to_vec();
        for word in [1u32, 1, 1] {
            b.extend(word.to_le_bytes());
        }
        for value in [0f32, 1., 1., 1.] {
            b.extend(value.to_le_bytes());
        }
        let payload = bytemuck::cast_slice::<f32, u8>(&[0., 0.25, 0.5, 1.]).to_vec();
        b.extend(hash(&payload).to_le_bytes());
        b.extend(hash(&b).to_le_bytes());
        b.resize(64, 0);
        b.extend(payload);
        let decoded = decode(&b).unwrap();
        assert_eq!(decoded.grid, [1, 1]);
        assert_eq!(decoded.heights, [0., 0.25, 0.5, 1.]);
        for byte in [4, 8, 16, 24, 32, 40, 48, 64, 79] {
            let mut damaged = b.clone();
            damaged[byte] ^= 1;
            assert!(decode(&damaged).is_err(), "byte {byte}");
        }
        assert!(decode(&b[..b.len() - 1]).is_err());
        b.push(0);
        assert!(decode(&b).is_err());
    }
    #[test]
    fn ray_domain_is_conservative() {
        let f = Field {
            grid: [1, 1],
            limits: [0., 1., 0.6, 0.6],
            heights: vec![],
        };
        assert!(!f.eligible(&render::camera_at(14., [100, 100])));
        let mut c = render::camera_at(100., [100, 100]);
        c.eye = Vec3::new(0., 100., 0.);
        assert!(f.eligible(&c));
        c.eye.y = 0.;
        assert!(!f.eligible(&c));
    }
}
