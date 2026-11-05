//! Surface symbols evaluated directly, without reconstructing source triangles.
use super::*;
#[path = "rock_symbol_build.rs"]
mod build;
#[path = "compiled_rock.rs"]
pub mod compiled;
use build::Symbol;
const ROOT: &str = "artifacts/appearance-probe/rock-symbol";
const SHADER: &str = include_str!("rock_symbol.wgsl");
const SIZES: [[u32; 2]; 2] = [[3456, 2234], [1728, 1117]];
const GRIDS: [u32; 3] = [16, 32, 64];
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    matrix: [[f32; 4]; 4],
    inverse: [[f32; 4]; 4],
    eye: [f32; 4],
    light: [f32; 4],
    screen: [f32; 4],
}
#[derive(Clone, Copy, serde::Serialize)]
struct Orbit {
    camera: f32,
    elevation: f32,
    light: f32,
    light_elevation: f32,
    distance: f32,
}
fn orbit(scene: usize, frame: usize, distance: f32) -> Orbit {
    let a = frame as f32 * 45.;
    Orbit {
        camera: if scene == 1 { 23.75 } else { 23.75 + a },
        elevation: if scene == 2 {
            25. + 20. * a.to_radians().sin()
        } else {
            35.
        },
        light: if scene == 0 { 67.25 } else { 67.25 + a * 2. },
        light_elevation: if scene == 2 {
            30. + 25. * a.to_radians().cos()
        } else {
            45.
        },
        distance,
    }
}
fn globals(p: Orbit, size: [u32; 2], jitter: [f32; 2]) -> Globals {
    let eye = CENTER + direction(p.camera.rem_euclid(360.), p.elevation) * p.distance;
    let projection = Mat4::perspective_infinite_reverse_rh(
        60f32.to_radians(),
        size[0] as f32 / size[1] as f32,
        0.1,
    );
    let shift = Mat4::from_translation(Vec3::new(
        2. * jitter[0] / size[0] as f32,
        -2. * jitter[1] / size[1] as f32,
        0.,
    ));
    let matrix = shift * projection * Mat4::look_at_rh(eye, CENTER, Vec3::Y);
    Globals {
        matrix: matrix.to_cols_array_2d(),
        inverse: matrix.inverse().to_cols_array_2d(),
        eye: eye.extend(0.).to_array(),
        light: direction(p.light.rem_euclid(360.), p.light_elevation)
            .extend(0.)
            .to_array(),
        screen: [size[0] as f32, size[1] as f32, 0., 0.],
    }
}
struct Target {
    color: wgpu::Texture,
    view: wgpu::TextureView,
    _depth: wgpu::Texture,
    depth: wgpu::TextureView,
    _accum: wgpu::Texture,
    accum: wgpu::TextureView,
    read: wgpu::Buffer,
    accum_group: wgpu::BindGroup,
    resolve_group: wgpu::BindGroup,
}
struct Uploaded {
    _symbols: wgpu::Buffer,
    group: wgpu::BindGroup,
    count: u32,
}
struct Renderer {
    gpu: Gpu,
    uniform: wgpu::Buffer,
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    indices: u32,
    levels: Vec<Uploaded>,
    full: wgpu::RenderPipeline,
    nearest: wgpu::RenderPipeline,
    depth: wgpu::RenderPipeline,
    accum: wgpu::RenderPipeline,
    resolve: wgpu::RenderPipeline,
    texture_layout: wgpu::BindGroupLayout,
    dummy_depth: wgpu::Texture,
    dummy_accum: wgpu::Texture,
    target: Option<Target>,
    size: [u32; 2],
    compiled: Option<compiled::Draw>,
    override_globals: Option<Globals>,
}
impl Renderer {
    async fn new(v: &[Vertex], i: &[u32], levels: &[Vec<Symbol>]) -> Result<Self> {
        Self::new_shader(v, i, levels, SHADER).await
    }
    async fn new_shader(
        v: &[Vertex],
        i: &[u32],
        levels: &[Vec<Symbol>],
        source: &str,
    ) -> Result<Self> {
        let gpu = Gpu::new().await?;
        ensure!(
            gpu.query.is_some(),
            "timestamp query required for experiment"
        );
        let d = &gpu.device;
        let uniform = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let buffer = |label, bytes: &[u8], usage| {
            d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytes,
                usage,
            })
        };
        let vertex = buffer(
            "rock source",
            bytemuck::cast_slice(v),
            wgpu::BufferUsages::VERTEX,
        );
        let index = buffer(
            "rock indices",
            bytemuck::cast_slice(i),
            wgpu::BufferUsages::INDEX,
        );
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let levels = levels
            .iter()
            .map(|ss| {
                let symbols = buffer(
                    "rock symbols",
                    bytemuck::cast_slice(ss),
                    wgpu::BufferUsages::STORAGE,
                );
                let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: symbols.as_entire_binding(),
                        },
                    ],
                });
                Uploaded {
                    _symbols: symbols,
                    group,
                    count: ss.len() as u32,
                }
            })
            .collect();
        let texture_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let base = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let filtered = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout, &texture_layout],
            push_constant_ranges: &[],
        });
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("surface symbols"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Uint32];
        let buffers = [wgpu::VertexBufferLayout {
            array_stride: 40,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attrs,
        }];
        let make = |vs, fs, mesh, depth, filter, blend, format| {
            let targets = [Some(wgpu::ColorTargetState {
                format,
                blend: if blend {
                    Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    })
                } else {
                    None
                },
                write_mask: wgpu::ColorWrites::ALL,
            })];
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(fs),
                layout: Some(if filter { &filtered } else { &base }),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers: if mesh { &buffers } else { &[] },
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: if fs == "depth_splat" { &[] } else { &targets },
                }),
                primitive: Default::default(),
                depth_stencil: if depth {
                    Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Greater,
                        stencil: Default::default(),
                        bias: Default::default(),
                    })
                } else {
                    None
                },
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let full = make(
            "mesh",
            "shade_mesh",
            true,
            true,
            false,
            false,
            wgpu::TextureFormat::Rgba32Float,
        );
        let nearest = make(
            "splat",
            "shade_splat",
            false,
            true,
            false,
            false,
            wgpu::TextureFormat::Rgba32Float,
        );
        let depth = make(
            "splat",
            "depth_splat",
            false,
            true,
            false,
            false,
            wgpu::TextureFormat::Rgba32Float,
        );
        let accum = make(
            "splat",
            "accumulate",
            false,
            false,
            true,
            true,
            wgpu::TextureFormat::Rgba16Float,
        );
        let resolve = make(
            "screen",
            "resolve",
            false,
            false,
            true,
            false,
            wgpu::TextureFormat::Rgba32Float,
        );
        let dummy_depth = texture(
            d,
            1,
            1,
            1,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let dummy_accum = texture(
            d,
            1,
            1,
            1,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );
        Ok(Self {
            gpu,
            uniform,
            vertex,
            index,
            indices: i.len() as u32,
            levels,
            full,
            nearest,
            depth,
            accum,
            resolve,
            texture_layout,
            dummy_depth,
            dummy_accum,
            target: None,
            size: [0, 0],
            compiled: None,
            override_globals: None,
        })
    }
    fn resize(&mut self, size: [u32; 2]) {
        if self.size == size {
            return;
        }
        let d = &self.gpu.device;
        let color = texture(
            d,
            size[0],
            size[1],
            1,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let depth_size = if self.compiled.is_some() && self.indices == 0 {
            [1, 1]
        } else {
            size
        };
        let accum_size = if self.compiled.is_some() {
            [1, 1]
        } else {
            size
        };
        let depth = texture(
            d,
            depth_size[0],
            depth_size[1],
            1,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let accum = texture(
            d,
            accum_size[0],
            accum_size[1],
            1,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let view = color.create_view(&Default::default());
        let dv = depth.create_view(&Default::default());
        let av = accum.create_view(&Default::default());
        let group = |depth, accum| {
            d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(accum),
                    },
                ],
            })
        };
        let dummy_accum_view = self.dummy_accum.create_view(&Default::default());
        let dummy_depth_view = self.dummy_depth.create_view(&Default::default());
        let accum_group = group(&dv, &dummy_accum_view);
        let resolve_group = group(&dummy_depth_view, &av);
        let read = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size[0] as u64 * size[1] as u64 * 16,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.target = Some(Target {
            color,
            view,
            _depth: depth,
            depth: dv,
            _accum: accum,
            accum: av,
            read,
            accum_group,
            resolve_group,
        });
        self.size = size;
    }
    fn encode(
        &self,
        e: &mut wgpu::CommandEncoder,
        p: Orbit,
        mode: usize,
        q: Option<u32>,
        jitter: [f32; 2],
    ) {
        if mode >= 7 {
            self.compiled.as_ref().expect("compiled data").encode(
                &self.gpu,
                e,
                self.target.as_ref().unwrap(),
                p,
                self.size,
                mode - 7,
                q,
                jitter,
            );
            return;
        }
        self.gpu.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(
                &self
                    .override_globals
                    .unwrap_or_else(|| globals(p, self.size, jitter)),
            ),
        );
        let filtered = mode > 0 && mode.is_multiple_of(2);
        let level = if mode == 0 { 0 } else { (mode - 1) / 2 };
        let t = self.target.as_ref().unwrap();
        for stage in 0..if filtered { 3 } else { 1 } {
            let depth = stage == 0;
            let target = if stage == 1 { &t.accum } else { &t.view };
            let attachment = [Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(if stage == 1 {
                        wgpu::Color::TRANSPARENT
                    } else {
                        wgpu::Color {
                            r: 0.04,
                            g: 0.07,
                            b: 0.11,
                            a: 0.,
                        }
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: if filtered && stage == 0 {
                    &[]
                } else {
                    &attachment
                },
                depth_stencil_attachment: if depth {
                    Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &t.depth,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    })
                } else {
                    None
                },
                timestamp_writes: q.filter(|_| !filtered || stage != 1).and_then(|base| {
                    self.gpu
                        .query
                        .as_ref()
                        .map(|qs| wgpu::RenderPassTimestampWrites {
                            query_set: qs,
                            beginning_of_pass_write_index: if stage == 0 {
                                Some(base)
                            } else {
                                None
                            },
                            end_of_pass_write_index: if !filtered || stage == 2 {
                                Some(base + 1)
                            } else {
                                None
                            },
                        })
                }),
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &self.levels[level].group, &[]);
            if mode == 0 {
                pass.set_pipeline(&self.full);
                pass.set_vertex_buffer(0, self.vertex.slice(..));
                pass.set_index_buffer(self.index.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.indices, 0, 0..1);
            } else {
                pass.set_pipeline(if !filtered {
                    &self.nearest
                } else {
                    match stage {
                        0 => &self.depth,
                        1 => &self.accum,
                        _ => &self.resolve,
                    }
                });
                if stage > 0 {
                    pass.set_bind_group(
                        1,
                        if stage == 1 {
                            &t.accum_group
                        } else {
                            &t.resolve_group
                        },
                        &[],
                    );
                }
                if stage == 2 {
                    pass.draw(0..3, 0..1);
                } else {
                    pass.draw(0..6, 0..self.levels[level].count);
                }
            }
        }
    }
    fn wait(&self, id: wgpu::SubmissionIndex) -> Result<()> {
        self.gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(id),
            timeout: Some(Duration::from_secs(60)),
        })?;
        Ok(())
    }
    fn capture(
        &mut self,
        p: Orbit,
        size: [u32; 2],
        mode: usize,
        jitter: [f32; 2],
    ) -> Result<Image> {
        self.resize(size);
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        self.encode(&mut e, p, mode, None, jitter);
        let t = self.target.as_ref().unwrap();
        e.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &t.color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &t.read,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size[0] * 16),
                    rows_per_image: Some(size[1]),
                },
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        self.wait(self.gpu.queue.submit([e.finish()]))?;
        self.gpu.map(&t.read)?;
        let data = t.read.slice(..).get_mapped_range();
        let pixels = bytemuck::cast_slice::<u8, [f32; 4]>(&data).to_vec();
        drop(data);
        t.read.unmap();
        ensure!(
            pixels.iter().flatten().all(|x| x.is_finite()),
            "nonfinite capture"
        );
        Ok(pixels)
    }
    fn integrated(&mut self, p: Orbit, size: [u32; 2], samples: usize) -> Result<Image> {
        let mut sum = vec![[0.; 4]; (size[0] * size[1]) as usize];
        for s in 0..samples {
            let image = self.capture(
                p,
                size,
                0,
                [crate::halton(s + 1, 2) - 0.5, crate::halton(s + 1, 3) - 0.5],
            )?;
            for (a, b) in sum.iter_mut().zip(image) {
                for k in 0..4 {
                    a[k] += b[k] / samples as f32;
                }
            }
        }
        Ok(sum)
    }
    fn bench(&mut self, p: Orbit, size: [u32; 2]) -> Result<serde_json::Value> {
        self.bench_modes(p, size, &[0, 1, 2, 3, 4, 5, 6])
    }
    fn bench_modes(
        &mut self,
        p: Orbit,
        size: [u32; 2],
        modes: &[usize],
    ) -> Result<serde_json::Value> {
        self.resize(size);
        let frames = 32;
        let slots = modes.len() * 2;
        let mut totals = vec![vec![]; slots];
        let mut cpus = totals.clone();
        let mut gpus = totals.clone();
        for &mode in modes {
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            self.encode(&mut e, p, mode, None, [0.; 2]);
            self.wait(self.gpu.queue.submit([e.finish()]))?;
        }
        for round in 0..7 {
            for offset in 0..slots {
                let s = if round % 2 == 0 {
                    (offset + round * 3) % slots
                } else {
                    (slots - 1 - offset + round * 3) % slots
                };
                let mode = modes[s % modes.len()];
                let start = Instant::now();
                let mut last = None;
                for _ in 0..frames {
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.encode(&mut e, p, mode, None, [0.; 2]);
                    last = Some(self.gpu.queue.submit([e.finish()]));
                }
                cpus[s].push(start.elapsed().as_secs_f64() * 1000. / frames as f64);
                self.wait(last.unwrap())?;
                totals[s].push(start.elapsed().as_secs_f64() * 1000. / frames as f64);
                let mut last = None;
                for i in 0..frames {
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.encode(&mut e, p, mode, Some(i as u32 * 2), [0.; 2]);
                    last = Some(self.gpu.queue.submit([e.finish()]));
                }
                self.wait(last.unwrap())?;
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                e.resolve_query_set(
                    self.gpu.query.as_ref().unwrap(),
                    0..64,
                    &self.gpu.query_resolve,
                    0,
                );
                e.copy_buffer_to_buffer(
                    &self.gpu.query_resolve,
                    0,
                    &self.gpu.query_readback,
                    0,
                    512,
                );
                self.wait(self.gpu.queue.submit([e.finish()]))?;
                self.gpu.map(&self.gpu.query_readback)?;
                let data = self.gpu.query_readback.slice(..).get_mapped_range();
                let stamps = bytemuck::cast_slice::<u8, u64>(&data);
                let mut sum = 0;
                for i in 0..frames {
                    ensure!(stamps[2 * i + 1] >= stamps[2 * i], "GPU clock");
                    sum += stamps[2 * i + 1] - stamps[2 * i];
                }
                gpus[s].push(
                    sum as f64 * self.gpu.timestamp_period_ns.unwrap() as f64 / frames as f64 / 1e6,
                );
                drop(data);
                self.gpu.query_readback.unmap();
            }
        }
        Ok(json!((0..slots).map(|s|json!({"method":modes[s%modes.len()],"control":s/modes.len(),"frames":frames,"total_ms":stats(&mut totals[s].clone()),"gpu_ms":stats(&mut gpus[s].clone()),"cpu_ms":stats(&mut cpus[s].clone()),"total_samples_ms":totals[s],"gpu_samples_ms":gpus[s],"cpu_samples_ms":cpus[s]})).collect::<Vec<_>>()))
    }
}
fn quality(a: &[[f32; 4]], b: &[[f32; 4]]) -> serde_json::Value {
    let m = metrics::compare(a, b);
    let mut diffs = vec![];
    let mut coverage = 0usize;
    let mut covered = 0usize;
    for (a, b) in a.iter().zip(b) {
        covered += usize::from(a[3] > 0.);
        if a[3] > 0. || b[3] > 0. {
            diffs.push((0..3).map(|k| (a[k] - b[k]).abs()).fold(0f32, f32::max));
            coverage += usize::from(a[3] != b[3]);
        }
    }
    diffs.sort_unstable_by(f32::total_cmp);
    let n = diffs.len();
    let p99 = if n == 0 { 0. } else { diffs[n * 99 / 100] };
    let fraction = coverage as f64 / n.max(1) as f64;
    json!({"rmse":m.rmse_surface,"interior_rmse":m.rmse_interior,"p99":p99,"max_rgb":m.max_abs,"coverage_fraction":fraction,"coverage_mismatches":coverage,"reference_covered_pixels":covered,"union_pixels":n,"pass":m.rmse_surface<=0.005&&p99<=0.02&&fraction<=0.0005})
}
pub fn run(smoke: bool, bench_only: bool) -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let source = Path::new(ASSETS).join("rock");
    let vb = fs::read(source.join("vertices.bin"))?;
    let ib = fs::read(source.join("indices.bin"))?;
    let (v, i) = decode(&vb, &ib)?;
    if bench_only {
        let old: serde_json::Value =
            serde_json::from_slice(&fs::read(Path::new(ROOT).join("build.json"))?)?;
        ensure!(
            old["source_hash"] == json!([hash(&vb), hash(&ib)])
                && old["shader_hash"] == json!(hash(SHADER.as_bytes())),
            "stale source/shader for benchmark resume"
        );
    }
    let mut levels = vec![];
    let mut meta = vec![];
    for grid in GRIDS {
        let now = Instant::now();
        let symbols = build::build(&v, &i, grid)?;
        let ms = now.elapsed().as_secs_f64() * 1000.;
        meta.push(json!({"grid":grid,"symbols":symbols.len(),"bytes":std::mem::size_of_val(symbols.as_slice()),"build_ms":ms,"max_radius":symbols.iter().map(|s|s.center_radius[3]).fold(0f32,f32::max),"max_thickness":symbols.iter().map(|s|s.normal_thickness[3]).fold(0f32,f32::max)}));
        if bench_only {
            ensure!(
                fs::read(Path::new(ROOT).join(format!("grid{grid}.symbols")))?
                    == bytemuck::cast_slice::<Symbol, u8>(&symbols),
                "stale symbol payload for benchmark resume"
            );
        }
        fs::write(
            Path::new(ROOT).join(format!("grid{grid}.symbols")),
            bytemuck::cast_slice(&symbols),
        )?;
        levels.push(symbols);
    }
    let start = Instant::now();
    let mut r = pollster::block_on(Renderer::new(&v, &i, &levels))?;
    r.wait(r.gpu.queue.submit([]))?;
    let init_ms = start.elapsed().as_secs_f64() * 1000.;
    if !bench_only {
        fs::write(
            Path::new(ROOT).join("build.json"),
            serde_json::to_vec_pretty(
                &json!({"source_triangles":i.len()/3,"source_vertices":v.len(),"source_bytes":vb.len()+ib.len(),"source_hash":[hash(&vb),hash(&ib)],"shader_hash":hash(SHADER.as_bytes()),"levels":meta,"renderer_init_and_all_upload_ms":init_ms,"adapter":r.gpu.adapter,"symbol_stride":48,"color_bytes_per_pixel":16,"depth_bytes_per_pixel":4,"filtered_extra_bytes_per_pixel":8,"readback_bytes_per_pixel":16}),
            )?,
        )?;
        let mut rows = vec![];
        for (res, size) in SIZES.into_iter().enumerate() {
            if smoke && res == 0 {
                continue;
            }
            for (distance_id, distance) in [4., 24., 96.].into_iter().enumerate() {
                if smoke && distance_id > 0 {
                    continue;
                }
                for scene in 0..3 {
                    if smoke && scene > 0 {
                        continue;
                    }
                    let mut previous: Vec<Option<(Image, Image)>> = (0..6).map(|_| None).collect();
                    for frame in 0..9 {
                        if smoke && frame > 0 {
                            continue;
                        }
                        let p = orbit(scene, frame, distance);
                        let id = format!("r{res}_d{distance_id}_s{scene}_{frame}");
                        let full = r.capture(p, size, 0, [0.; 2])?;
                        metrics::png(&Path::new(ROOT).join(format!("{id}_0.png")), &full, size[0])?;
                        let mut qs = vec![];
                        for mode in 1..7 {
                            let image = r.capture(p, size, mode, [0.; 2])?;
                            metrics::png(
                                &Path::new(ROOT).join(format!("{id}_{mode}.png")),
                                &image,
                                size[0],
                            )?;
                            let mut q = quality(&full, &image);
                            q["temporal_residual"] = json!(
                                previous[mode - 1]
                                    .as_ref()
                                    .map(|(a, b)| metrics::temporal(a, b, &full, &image))
                            );
                            previous[mode - 1] = Some((full.clone(), image));
                            qs.push(q);
                        }
                        println!(
                            "capture {id}: {} / 6 quality gates",
                            qs.iter().filter(|q| q["pass"] == true).count()
                        );
                        rows.push(json!({"id":id,"resolution":res,"size":size,"distance_id":distance_id,"scene":scene,"frame":frame,"pose":p,"checks":qs}));
                        fs::write(
                            Path::new(ROOT).join(if smoke { "smoke.json" } else { "quality.json" }),
                            serde_json::to_vec_pretty(&rows)?,
                        )?;
                    }
                }
            }
        }
        if smoke {
            return Ok(());
        }
        let mut checks = vec![];
        let p = orbit(0, 0, 4.);
        for mode in 0..7 {
            let a = r.capture(p, SIZES[1], mode, [0.; 2])?;
            ensure!(a == r.capture(p, SIZES[1], mode, [0.; 2])?, "repeat");
            ensure!(
                a == r.capture(
                    Orbit {
                        camera: p.camera + 360.,
                        light: p.light + 360.,
                        ..p
                    },
                    SIZES[1],
                    mode,
                    [0.; 2]
                )?,
                "wrap"
            );
            let changed = r.capture(
                Orbit {
                    light: p.light + 97.,
                    ..p
                },
                SIZES[1],
                mode,
                [0.; 2],
            )?;
            ensure!(a != changed, "light has no effect");
            ensure!(a == r.capture(p, SIZES[1], mode, [0.; 2])?, "return");
            checks.push(json!({"method":mode,"repeat_wrap_light_return":true}));
        }
        fs::write(
            Path::new(ROOT).join("checks.json"),
            serde_json::to_vec_pretty(&checks)?,
        )?;
        let mut integrated = vec![];
        for (res, size) in SIZES.into_iter().enumerate() {
            for (distance_id, distance) in [4., 24., 96.].into_iter().enumerate() {
                let p = orbit(0, 0, distance);
                let a = r.integrated(p, size, 16)?;
                let b = r.integrated(p, size, 64)?;
                metrics::png(
                    &Path::new(ROOT).join(format!("r{res}_d{distance_id}_integrated.png")),
                    &b,
                    size[0],
                )?;
                let mut errs = vec![];
                for mode in 0..7 {
                    let image = r.capture(p, size, mode, [0.; 2])?;
                    errs.push(metrics::compare(&b, &image));
                }
                integrated.push(json!({"resolution":res,"distance_id":distance_id,"reference_spp":64,"convergence_16_64":metrics::compare(&b,&a),"errors_1spp":errs}));
                println!("integrated r{res} d{distance_id}");
            }
        }
        fs::write(
            Path::new(ROOT).join("integrated.json"),
            serde_json::to_vec_pretty(&integrated)?,
        )?;
    }
    for repeat in [false, true] {
        let mut timings = vec![];
        for (res, size) in SIZES.into_iter().enumerate() {
            for (distance_id, distance) in [4., 24., 96.].into_iter().enumerate() {
                for frame in if repeat { vec![0] } else { vec![0, 3, 6] } {
                    let p = orbit(2, frame, distance);
                    let samples = r.bench(p, size)?;
                    timings.push(json!({"resolution":res,"size":size,"distance_id":distance_id,"scene":2,"frame":frame,"pose":p,"samples":samples}));
                    fs::write(
                        Path::new(ROOT).join(if repeat {
                            "repeat.json"
                        } else {
                            "timings.json"
                        }),
                        serde_json::to_vec_pretty(&timings)?,
                    )?;
                    println!("benchmark repeat{repeat} r{res} d{distance_id} f{frame}");
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_projection_wrap_and_empty_quality() {
        let p = orbit(0, 0, 4.);
        let a = globals(p, SIZES[0], [0.; 2]);
        let b = globals(
            Orbit {
                camera: p.camera + 360.,
                light: p.light + 360.,
                ..p
            },
            SIZES[0],
            [0.; 2],
        );
        assert_eq!(a.matrix, b.matrix);
        assert_eq!(a.light, b.light);
        assert_eq!(a.screen[0], 3456.);
        assert_eq!(quality(&[[0.; 4]], &[[0.; 4]])["pass"], true);
        assert_eq!(quality(&[[1., 1., 1., 1.]], &[[0.; 4]])["pass"], false);
    }
}
