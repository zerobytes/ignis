use super::*;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    matrix: [[f32; 4]; 4],
    eye: [f32; 4],
    light: [f32; 4],
    settings: [u32; 4],
}
pub(super) struct Uploaded {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    group: wgpu::BindGroup,
    _normal: wgpu::Texture,
}
struct Target {
    color: wgpu::Texture,
    view: wgpu::TextureView,
    _depth: wgpu::Texture,
    depth: wgpu::TextureView,
    read: wgpu::Buffer,
}
pub(super) struct Renderer {
    pub gpu: Gpu,
    uniform: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    target: Option<Target>,
    size: [u32; 2],
}
pub(super) fn pose(frame: usize, size: [u32; 2]) -> Camera {
    let step = if frame <= 4 { frame } else { 8 - frame };
    let distance = 14. * 2f32.powi(step as i32);
    let eye = Vec3::new(0.15, 0.66, 0.736).normalize() * distance;
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let matrix = Mat4::perspective_infinite_reverse_rh(
        60f32.to_radians(),
        size[0] as f32 / size[1] as f32,
        0.1,
    ) * view;
    Camera {
        eye,
        view,
        matrix,
        inverse: matrix.inverse(),
        size,
        light: Vec3::new(-0.4, 0.7, -0.5).normalize(),
    }
}
pub(super) fn projected_error(cam: &Camera, error: f32) -> f32 {
    let nearest = cam.eye.length() - 9.;
    if nearest <= 0.1 {
        return f32::INFINITY;
    }
    let focal = cam.size[1] as f32 / (2. * 30f32.to_radians().tan());
    2. * focal * error / nearest * (1. + 9. / nearest)
}
pub(super) fn select(asset: &Asset, cam: &Camera, mode: u32) -> usize {
    if mode == 0 {
        return asset.levels.len() - 1;
    }
    asset
        .levels
        .iter()
        .position(|l| {
            projected_error(cam, l.position_error) <= 0.15 && (mode != 2 || l.normal_error <= 0.005)
        })
        .unwrap_or(asset.levels.len() - 1)
}
impl Renderer {
    pub async fn new() -> Result<Self> {
        let gpu = Gpu::new().await?;
        let d = &gpu.device;
        let uniform = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 112,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
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
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle_scaling.wgsl").into()),
        });
        let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3];
        let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 24,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
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
        });
        Ok(Self {
            gpu,
            uniform,
            layout,
            pipeline,
            target: None,
            size: [0, 0],
        })
    }
    pub fn upload(&self, a: &Asset) -> Result<Uploaded> {
        let root = Path::new(ROOT);
        let d = &self.gpu.device;
        let v = fs::read(root.join(format!("{}.vertices", a.id)))?;
        let i = fs::read(root.join(format!("{}.indices", a.id)))?;
        let n = fs::read(root.join(format!("{}.normals", a.id)))?;
        ensure!(
            v.len() as u64 + i.len() as u64 + n.len() as u64 == a.bytes,
            "stale asset bytes"
        );
        let vertex = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &v,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &i,
            usage: wgpu::BufferUsages::INDEX,
        });
        let normal = texture(
            d,
            a.grid[0] + 1,
            a.grid[1] + 1,
            1,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        self.gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &normal,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &n,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((a.grid[0] + 1) * 16),
                rows_per_image: Some(a.grid[1] + 1),
            },
            wgpu::Extent3d {
                width: a.grid[0] + 1,
                height: a.grid[1] + 1,
                depth_or_array_layers: 1,
            },
        );
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &normal.create_view(&Default::default()),
                    ),
                },
            ],
        });
        Ok(Uploaded {
            vertex,
            index,
            group,
            _normal: normal,
        })
    }
    pub fn resize(&mut self, size: [u32; 2]) {
        if self.size == size {
            return;
        }
        let [w, h] = size;
        let d = &self.gpu.device;
        let color = texture(
            d,
            w,
            h,
            1,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = texture(
            d,
            w,
            h,
            1,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        self.target = Some(Target {
            view: color.create_view(&Default::default()),
            color,
            depth: depth.create_view(&Default::default()),
            _depth: depth,
            read: d.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: w as u64 * h as u64 * 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
        });
        self.size = size;
    }
    fn encode(
        &self,
        e: &mut wgpu::CommandEncoder,
        u: &Uploaded,
        a: &Asset,
        cam: &Camera,
        mode: u32,
        q: Option<u32>,
    ) -> usize {
        let level = select(a, cam, mode);
        let l = &a.levels[level];
        let t = self.target.as_ref().unwrap();
        self.gpu.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&Uniforms {
                matrix: cam.matrix.to_cols_array_2d(),
                eye: cam.eye.extend(0.).to_array(),
                light: cam.light.extend(0.).to_array(),
                settings: [mode, 0, 0, 0],
            }),
        );
        let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &t.view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.04,
                        g: 0.07,
                        b: 0.11,
                        a: 0.,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &t.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: q.and_then(|base| {
                self.gpu
                    .query
                    .as_ref()
                    .map(|query_set| wgpu::RenderPassTimestampWrites {
                        query_set,
                        beginning_of_pass_write_index: Some(base),
                        end_of_pass_write_index: Some(base + 1),
                    })
            }),
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &u.group, &[]);
        pass.set_vertex_buffer(0, u.vertex.slice(..));
        pass.set_index_buffer(u.index.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(l.first..l.first + l.count, 0, 0..1);
        level
    }
    fn wait(&self, id: wgpu::SubmissionIndex) -> Result<()> {
        self.gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(id),
            timeout: Some(Duration::from_secs(60)),
        })?;
        Ok(())
    }
    pub fn capture(
        &mut self,
        u: &Uploaded,
        a: &Asset,
        cam: &Camera,
        mode: u32,
    ) -> Result<(Vec<[f32; 4]>, usize)> {
        self.resize(cam.size);
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        let level = self.encode(&mut e, u, a, cam, mode, None);
        let t = self.target.as_ref().unwrap();
        copy_readback(&mut e, &t.color, &t.read, cam.size[0], cam.size[1], 16);
        self.wait(self.gpu.queue.submit([e.finish()]))?;
        self.gpu.map(&t.read)?;
        let bytes = t.read.slice(..).get_mapped_range();
        let pixels = bytemuck::cast_slice::<u8, [f32; 4]>(&bytes).to_vec();
        drop(bytes);
        t.read.unmap();
        ensure!(
            pixels.iter().flatten().all(|x| x.is_finite()),
            "nonfinite pixels"
        );
        Ok((pixels, level))
    }
    pub fn bench(
        &self,
        u: &Uploaded,
        a: &Asset,
        frame: usize,
        size: [u32; 2],
    ) -> Result<serde_json::Value> {
        self.bench_set(&[(u, a)], frame, size, 8)
    }
    pub fn bench_set(
        &self,
        inputs: &[(&Uploaded, &Asset)],
        frame: usize,
        size: [u32; 2],
        batch: usize,
    ) -> Result<serde_json::Value> {
        let slots = inputs.len() * 8;
        ensure!(slots * 12 * 2 <= 1024, "query capacity");
        for _ in 0..12 {
            for entry in 0..inputs.len() * 4 {
                let (u, a) = inputs[entry / 4];
                let mode = (entry % 4) as u32;
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.encode(&mut e, u, a, &pose(frame, size), mode, None);
                self.wait(self.gpu.queue.submit([e.finish()]))?;
            }
        }
        let mut gpu = vec![Vec::new(); slots];
        let mut serial = gpu.clone();
        let mut throughput = gpu.clone();
        let mut cpu = gpu.clone();
        for r in 0..12 {
            for offset in 0..slots {
                let s = (r * 7 + offset) % slots;
                let (u, a) = inputs[(s % (inputs.len() * 4)) / 4];
                let start = Instant::now();
                let cam = pose(frame, size);
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.encode(
                    &mut e,
                    u,
                    a,
                    &cam,
                    (s % 4) as u32,
                    Some(((r * slots + s) * 2) as u32),
                );
                cpu[s].push(start.elapsed().as_secs_f64() * 1000.);
                self.wait(self.gpu.queue.submit([e.finish()]))?;
                serial[s].push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        if let Some(q) = &self.gpu.query {
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(q, 0..(slots * 12 * 2) as u32, &self.gpu.query_resolve, 0);
            e.copy_buffer_to_buffer(
                &self.gpu.query_resolve,
                0,
                &self.gpu.query_readback,
                0,
                8192,
            );
            self.wait(self.gpu.queue.submit([e.finish()]))?;
            self.gpu.map(&self.gpu.query_readback)?;
            let bytes = self.gpu.query_readback.slice(..).get_mapped_range();
            let stamps = bytemuck::cast_slice::<u8, u64>(&bytes);
            for r in 0..12 {
                for (s, g) in gpu.iter_mut().enumerate() {
                    let b = (r * slots + s) * 2;
                    ensure!(stamps[b + 1] >= stamps[b], "GPU clock");
                    g.push(
                        (stamps[b + 1] - stamps[b]) as f64
                            * self.gpu.timestamp_period_ns.unwrap() as f64
                            / 1e6,
                    );
                }
            }
            drop(bytes);
            self.gpu.query_readback.unmap();
        }
        for r in 0..12 {
            for offset in 0..slots {
                let s = (r * 7 + offset) % slots;
                let (u, a) = inputs[(s % (inputs.len() * 4)) / 4];
                let start = Instant::now();
                let mut last = None;
                for _ in 0..batch {
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.encode(&mut e, u, a, &pose(frame, size), (s % 4) as u32, None);
                    last = Some(self.gpu.queue.submit([e.finish()]));
                }
                self.wait(last.unwrap())?;
                throughput[s].push(start.elapsed().as_secs_f64() * 1000. / batch as f64);
            }
        }
        Ok(json!((0..slots).map(|s|json!({"mode":MODES[s%4],"gpu_ms":stats(&mut gpu[s]),"serial_ms":stats(&mut serial[s]),"throughput_ms":stats(&mut throughput[s]),"cpu_ms":stats(&mut cpu[s])})).collect::<Vec<_>>()))
    }
}
