//! Full-frame offscreen renderer; one shared depth attachment for geometry and summaries.
use super::*;
#[path = "regional_efficiency_bench.rs"]
mod efficiency_bench;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    matrix: [[f32; 4]; 4],
    inverse: [[f32; 4]; 4],
    eye: [f32; 4],
    light: [f32; 4],
    screen: [f32; 4],
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
    instances: wgpu::Buffer,
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    group: wgpu::BindGroup,
    full: wgpu::RenderPipeline,
    proxy: wgpu::RenderPipeline,
    target: Option<Target>,
    size: [u32; 2],
    pub cache_bytes: usize,
}
impl Renderer {
    pub async fn new(scene: &Scene) -> Result<Self> {
        let gpu = Gpu::new().await?;
        let d = &gpu.device;
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("regional scene"),
            entries: &(0..5)
                .map(|i| wgpu::BindGroupLayoutEntry {
                    binding: i,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: if i < 3 {
                        wgpu::BindingType::Buffer {
                            ty: if i == 0 {
                                wgpu::BufferBindingType::Uniform
                            } else {
                                wgpu::BufferBindingType::Storage { read_only: true }
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        }
                    } else {
                        wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        }
                    },
                    count: None,
                })
                .collect::<Vec<_>>(),
        });
        let uniform = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: ((scene.objects.len() + 1 + scene.objects.len() * scene.regions.len()) * 32)
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let regions = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&scene.regions),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let vertex = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&scene.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&scene.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let width = 1024;
        let height = scene.cache.len().div_ceil(width);
        let mut colors = vec![[0f32; 4]; width * height];
        let mut normals = colors.clone();
        for (i, t) in scene.cache.iter().enumerate() {
            colors[i] = t.color;
            normals[i] = t.normal_depth;
        }
        let textures = [colors, normals].map(|data| {
            let t = texture(
                d,
                width as u32,
                height as u32,
                1,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );
            gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &t,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&data),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width as u32 * 16),
                    rows_per_image: Some(height as u32),
                },
                wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
            );
            t.create_view(&Default::default())
        });
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
                    resource: regions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&textures[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&textures[1]),
                },
            ],
        });
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("regional surface cache"),
            source: wgpu::ShaderSource::Wgsl(include_str!("regional_scene.wgsl").into()),
        });
        let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Uint32];
        let vb = [wgpu::VertexBufferLayout {
            array_stride: 40,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attrs,
        }];
        let pipeline = |proxy| {
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(if proxy { "quad" } else { "geometry" }),
                    compilation_options: Default::default(),
                    buffers: if proxy { &[] } else { &vb },
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if proxy { "summary" } else { "surface" }),
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
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let full = pipeline(false);
        let proxy = pipeline(true);
        Ok(Self {
            gpu,
            uniform,
            instances,
            vertex,
            index,
            group,
            full,
            proxy,
            target: None,
            size: [0, 0],
            cache_bytes: width * height * 32,
        })
    }
    fn target(&mut self, size: [u32; 2]) {
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
        let t = Target {
            view: color.create_view(&Default::default()),
            depth: depth.create_view(&Default::default()),
            color,
            _depth: depth,
            read: d.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: w as u64 * h as u64 * 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
        };
        self.target = Some(t);
        self.size = size;
    }
    fn update(&self, cam: &Camera, plan: &Plan, debug: u32, jitter: [f32; 2]) {
        let matrix = Mat4::from_translation(Vec3::new(
            jitter[0] * 2. / cam.size[0] as f32,
            jitter[1] * 2. / cam.size[1] as f32,
            0.,
        )) * cam.matrix;
        let g = Globals {
            matrix: matrix.to_cols_array_2d(),
            inverse: if jitter == [0.; 2] {
                cam.inverse
            } else {
                matrix.inverse()
            }
            .to_cols_array_2d(),
            eye: cam.eye.extend(0.).to_array(),
            light: cam.light.extend(0.).to_array(),
            screen: [cam.size[0] as f32, cam.size[1] as f32, debug as f32, 0.],
        };
        self.gpu
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&g));
        self.gpu
            .queue
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(&plan.instances));
    }
    fn draw(&self, enc: &mut wgpu::CommandEncoder, plan: &Plan, query: Option<u32>) {
        let t = self.target.as_ref().unwrap();
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("complete scene"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &t.view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.12,
                        g: 0.19,
                        b: 0.28,
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
            timestamp_writes: query.and_then(|q| {
                self.gpu
                    .query
                    .as_ref()
                    .map(|set| wgpu::RenderPassTimestampWrites {
                        query_set: set,
                        beginning_of_pass_write_index: Some(q),
                        end_of_pass_write_index: Some(q + 1),
                    })
            }),
            occlusion_query_set: None,
        });
        pass.set_bind_group(0, &self.group, &[]);
        pass.set_pipeline(&self.full);
        pass.set_vertex_buffer(0, self.vertex.slice(..));
        pass.set_index_buffer(self.index.slice(..), wgpu::IndexFormat::Uint32);
        for d in &plan.draws {
            pass.draw_indexed(d.indices.clone(), 0, d.instance..d.instance + d.count);
        }
        if !plan.proxies.is_empty() {
            pass.set_pipeline(&self.proxy);
            pass.draw(0..6, plan.proxies.clone());
        }
    }
    fn submit(&self, enc: wgpu::CommandEncoder) -> Result<()> {
        let i = self.gpu.queue.submit([enc.finish()]);
        self.gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(i),
            timeout: Some(Duration::from_secs(60)),
        })?;
        Ok(())
    }
    pub fn capture(
        &mut self,
        cam: &Camera,
        plan: &Plan,
        debug: bool,
        jitter: [f32; 2],
    ) -> Result<Vec<[f32; 4]>> {
        self.capture_mode(cam, plan, u32::from(debug), jitter)
    }
    pub fn capture_mode(
        &mut self,
        cam: &Camera,
        plan: &Plan,
        debug: u32,
        jitter: [f32; 2],
    ) -> Result<Vec<[f32; 4]>> {
        self.target(cam.size);
        self.update(cam, plan, debug, jitter);
        let mut enc = self.gpu.device.create_command_encoder(&Default::default());
        self.draw(&mut enc, plan, None);
        let t = self.target.as_ref().unwrap();
        copy_readback(&mut enc, &t.color, &t.read, cam.size[0], cam.size[1], 16);
        self.submit(enc)?;
        self.gpu.map(&t.read)?;
        let bytes = t.read.slice(..).get_mapped_range();
        let pixels = bytemuck::cast_slice::<u8, [f32; 4]>(&bytes).to_vec();
        drop(bytes);
        t.read.unmap();
        ensure!(
            pixels.iter().flatten().all(|v| v.is_finite()),
            "nonfinite scene capture"
        );
        Ok(pixels)
    }
    pub fn bench(
        &mut self,
        cam: &Camera,
        scene: &Scene,
        states: &[Vec<bool>],
        full: &Plan,
        adaptive: &Plan,
    ) -> Result<serde_json::Value> {
        self.target(cam.size);
        self.update(cam, adaptive, 0, [0.; 2]);
        let plans = [full, adaptive, full, adaptive];
        let mut enc = self.gpu.device.create_command_encoder(&Default::default());
        for _ in 0..3 {
            for p in plans {
                self.draw(&mut enc, p, None);
            }
        }
        self.submit(enc)?;
        let mut encode = vec![Vec::new(); 4];
        let mut enc = self.gpu.device.create_command_encoder(&Default::default());
        for r in 0..9 {
            for off in 0..4 {
                let m = (r + off) % 4;
                let start = Instant::now();
                self.draw(&mut enc, plans[m], Some(((r * 4 + m) * 2) as u32));
                encode[m].push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        self.submit(enc)?;
        let mut gpu = vec![Vec::new(); 4];
        if let Some(q) = &self.gpu.query {
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(q, 0..72, &self.gpu.query_resolve, 0);
            e.copy_buffer_to_buffer(
                &self.gpu.query_resolve,
                0,
                &self.gpu.query_readback,
                0,
                8192,
            );
            self.submit(e)?;
            self.gpu.map(&self.gpu.query_readback)?;
            let bytes = self.gpu.query_readback.slice(..).get_mapped_range();
            let stamps = bytemuck::cast_slice::<u8, u64>(&bytes);
            let scale = f64::from(self.gpu.timestamp_period_ns.unwrap()) / 1e6;
            for r in 0..9 {
                for (m, t) in gpu.iter_mut().enumerate() {
                    let i = (r * 4 + m) * 2;
                    ensure!(stamps[i + 1] >= stamps[i], "invalid GPU interval");
                    t.push((stamps[i + 1] - stamps[i]) as f64 * scale);
                }
            }
            drop(bytes);
            self.gpu.query_readback.unmap();
        }
        let mut serial_plan = Plan::new(scene.objects.len() * scene.regions.len());
        let mut serial_states = states.to_vec();
        let mut serial = vec![Vec::new(); 4];
        let mut upload = vec![Vec::new(); 4];
        for r in 0..5 {
            for off in 0..4 {
                let m = (r + off) % 4;
                let start = Instant::now();
                scene.plan(cam, m % 2 == 1, &mut serial_states, &mut serial_plan);
                let upload_start = Instant::now();
                self.update(cam, &serial_plan, 0, [0.; 2]);
                upload[m].push(upload_start.elapsed().as_secs_f64() * 1000.);
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.draw(&mut e, &serial_plan, None);
                self.submit(e)?;
                serial[m].push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        Ok(json!((0..4).map(|m|json!({"gpu_ms":stats(&mut gpu[m]),"encode_ms":stats(&mut encode[m]),"upload_ms":stats(&mut upload[m]),"serial_ms":stats(&mut serial[m])})).collect::<Vec<_>>()))
    }
}
