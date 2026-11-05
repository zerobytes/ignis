//! GPU depth hierarchy and indirect instance compaction. No visibility readback in encode.
use super::*;
#[path = "gpu_visibility_bench.rs"]
mod bench;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    matrix: [[f32; 4]; 4],
    inverse: [[f32; 4]; 4],
    eye: [f32; 4],
    light: [f32; 4],
    screen: [f32; 4],
}
struct Targets {
    color: wgpu::Texture,
    view: wgpu::TextureView,
    _depth: wgpu::Texture,
    depth: wgpu::TextureView,
    read: wgpu::Buffer,
    _pyramid: wgpu::Texture,
    levels: u32,
    base: [u32; 2],
    reduce_groups: Vec<wgpu::BindGroup>,
    cull_group: wgpu::BindGroup,
    pyramid_bytes: u64,
}
pub(super) struct Renderer {
    pub gpu: Gpu,
    uniform: wgpu::Buffer,
    settings: wgpu::Buffer,
    input: wgpu::Buffer,
    output: wgpu::Buffer,
    nodes: wgpu::Buffer,
    members: wgpu::Buffer,
    alive: wgpu::Buffer,
    commands: wgpu::Buffer,
    counter_read: wgpu::Buffer,
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    input_group: wgpu::BindGroup,
    full: wgpu::RenderPipeline,
    root_bundle: wgpu::RenderBundle,
    region_bundle: wgpu::RenderBundle,
    depth_layout: wgpu::BindGroupLayout,
    cull_layout: wgpu::BindGroupLayout,
    base_pipeline: wgpu::ComputePipeline,
    reduce_pipeline: wgpu::ComputePipeline,
    roots_pipeline: wgpu::ComputePipeline,
    regions_pipeline: wgpu::ComputePipeline,
    target: Option<Targets>,
    size: [u32; 2],
    regions: u32,
    pub static_bytes: u64,
}
fn buffer(d: &wgpu::Device, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    d.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage,
        mapped_at_creation: false,
    })
}
fn entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}
fn storage(read_only: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}
fn uniform_type() -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}
fn tex(sample_type: wgpu::TextureSampleType) -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        sample_type,
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
    }
}
fn bind(binding: u32, b: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: b.as_entire_binding(),
    }
}
fn view(binding: u32, t: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(t),
    }
}
impl Renderer {
    pub async fn new(world: &World) -> Result<Self> {
        let gpu = Gpu::with_features(wgpu::Features::INDIRECT_FIRST_INSTANCE).await?;
        let d = &gpu.device;
        let regions = world.hierarchy.regions as u32;
        let uniform = buffer(
            d,
            std::mem::size_of::<Globals>() as u64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let settings = buffer(
            d,
            16,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let input = buffer(
            d,
            200 * 32,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        let output = buffer(
            d,
            u64::from(regions + 3) * 96 * 32,
            wgpu::BufferUsages::STORAGE,
        );
        let commands = buffer(
            d,
            u64::from(regions + 3) * 20,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_SRC,
        );
        let counter_read = buffer(
            d,
            commands.size(),
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let alive = buffer(d, 96 * 4, wgpu::BufferUsages::STORAGE);
        let init = |data: &[u8], usage| {
            d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: data,
                usage,
            })
        };
        let nodes = init(
            bytemuck::cast_slice(&world.hierarchy.nodes),
            wgpu::BufferUsages::STORAGE,
        );
        let members = init(
            bytemuck::cast_slice(&world.members),
            wgpu::BufferUsages::STORAGE,
        );
        let vertex = init(
            bytemuck::cast_slice(&world.scene.vertices),
            wgpu::BufferUsages::VERTEX,
        );
        let index = init(
            bytemuck::cast_slice(&world.scene.indices),
            wgpu::BufferUsages::INDEX,
        );
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let c = wgpu::ShaderStages::COMPUTE;
        let geometry_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[entry(0, vf, uniform_type()), entry(2, vf, storage(true))],
        });
        let input_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &geometry_layout,
            entries: &[bind(0, &uniform), bind(2, &input)],
        });
        let output_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &geometry_layout,
            entries: &[bind(0, &uniform), bind(2, &output)],
        });
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("unchanged geometry shading"),
            source: wgpu::ShaderSource::Wgsl(include_str!("regional_scene.wgsl").into()),
        });
        let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&geometry_layout],
            push_constant_ranges: &[],
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Uint32];
        let full = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("geometry"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 40,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("surface"),
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
        });
        let bundle = |range: Range<u32>| {
            let mut b = d.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: None,
                color_formats: &[Some(wgpu::TextureFormat::Rgba32Float)],
                depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_read_only: false,
                    stencil_read_only: true,
                }),
                sample_count: 1,
                multiview: None,
            });
            b.set_pipeline(&full);
            b.set_bind_group(0, &output_group, &[]);
            b.set_vertex_buffer(0, vertex.slice(..));
            b.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
            for i in range {
                b.draw_indexed_indirect(&commands, u64::from(i) * 20);
            }
            b.finish(&Default::default())
        };
        let root_bundle = bundle(0..3);
        let region_bundle = bundle(3..3 + regions);
        let depth_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                entry(0, c, tex(wgpu::TextureSampleType::Depth)),
                entry(
                    1,
                    c,
                    tex(wgpu::TextureSampleType::Float { filterable: false }),
                ),
                entry(
                    2,
                    c,
                    wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
            ],
        });
        let cull_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                entry(0, c, uniform_type()),
                entry(1, c, storage(true)),
                entry(2, c, storage(true)),
                entry(
                    3,
                    c,
                    tex(wgpu::TextureSampleType::Float { filterable: false }),
                ),
                entry(4, c, storage(false)),
                entry(5, c, storage(false)),
                entry(6, c, storage(false)),
                entry(7, c, storage(true)),
                entry(8, c, uniform_type()),
            ],
        });
        let depth_shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("conservative reverse depth min hierarchy"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gpu_visibility_depth.wgsl").into()),
        });
        let cull_shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("current frame visibility"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gpu_visibility.wgsl").into()),
        });
        let pipeline = |shader: &wgpu::ShaderModule, layout: &wgpu::BindGroupLayout, name: &str| {
            let p = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });
            d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&p),
                module: shader,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let base_pipeline = pipeline(&depth_shader, &depth_layout, "base");
        let reduce_pipeline = pipeline(&depth_shader, &depth_layout, "reduce");
        let roots_pipeline = pipeline(&cull_shader, &cull_layout, "roots");
        let regions_pipeline = pipeline(&cull_shader, &cull_layout, "regions");
        let static_bytes = uniform.size()
            + settings.size()
            + input.size()
            + output.size()
            + commands.size()
            + alive.size()
            + nodes.size()
            + members.size()
            + vertex.size()
            + index.size();
        Ok(Self {
            gpu,
            uniform,
            settings,
            input,
            output,
            nodes,
            members,
            alive,
            commands,
            counter_read,
            vertex,
            index,
            input_group,
            full,
            root_bundle,
            region_bundle,
            depth_layout,
            cull_layout,
            base_pipeline,
            reduce_pipeline,
            roots_pipeline,
            regions_pipeline,
            target: None,
            size: [0; 2],
            regions,
            static_bytes,
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
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let depth_view = depth.create_view(&Default::default());
        let base = [
            w.div_ceil(4).next_power_of_two(),
            h.div_ceil(4).next_power_of_two(),
        ];
        let levels = base[0].max(base[1]).ilog2() + 1;
        let pyramid = d.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth minimum pyramid"),
            size: wgpu::Extent3d {
                width: base[0],
                height: base[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let dummy = texture(
            d,
            1,
            1,
            1,
            wgpu::TextureFormat::R32Float,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let dummy_view = dummy.create_view(&Default::default());
        let mips = (0..levels)
            .map(|i| {
                pyramid.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: i,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>();
        let reduce_groups = (0..levels)
            .map(|i| {
                d.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.depth_layout,
                    entries: &[
                        view(0, &depth_view),
                        view(
                            1,
                            if i == 0 {
                                &dummy_view
                            } else {
                                &mips[i as usize - 1]
                            },
                        ),
                        view(2, &mips[i as usize]),
                    ],
                })
            })
            .collect();
        let all = pyramid.create_view(&Default::default());
        let cull_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.cull_layout,
            entries: &[
                bind(0, &self.uniform),
                bind(1, &self.input),
                bind(2, &self.nodes),
                view(3, &all),
                bind(4, &self.output),
                bind(5, &self.commands),
                bind(6, &self.alive),
                bind(7, &self.members),
                bind(8, &self.settings),
            ],
        });
        self.target = Some(Targets {
            view: color.create_view(&Default::default()),
            color,
            _depth: depth,
            depth: depth_view,
            read: buffer(
                d,
                u64::from(w) * u64::from(h) * 16,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            ),
            _pyramid: pyramid,
            levels,
            base,
            reduce_groups,
            cull_group,
            pyramid_bytes: (0..levels)
                .map(|i| u64::from((base[0] >> i).max(1)) * u64::from((base[1] >> i).max(1)) * 4)
                .sum(),
        });
        self.size = size;
    }
    fn update(&self, p: &Pose, mode: u32) {
        let cam = &p.camera;
        let g = Globals {
            matrix: cam.matrix.to_cols_array_2d(),
            inverse: cam.inverse.to_cols_array_2d(),
            eye: cam.eye.extend(0.).to_array(),
            light: cam.light.extend(0.).to_array(),
            screen: [cam.size[0] as f32, cam.size[1] as f32, 0., 0.],
        };
        self.gpu
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&g));
        self.gpu
            .queue
            .write_buffer(&self.input, 0, bytemuck::cast_slice(&p.input));
        self.gpu.queue.write_buffer(
            &self.settings,
            0,
            bytemuck::cast_slice(&[
                self.regions,
                mode,
                self.target.as_ref().unwrap().levels,
                (p.obstacles.len() - 1) as u32,
            ]),
        );
    }
    fn encode(&self, e: &mut wgpu::CommandEncoder, p: &Pose, mode: u32, q: Option<u32>) {
        let t = self.target.as_ref().unwrap();
        if mode >= 4 {
            self.cull(e, mode, q);
        }
        {
            let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("current opaque occluders"),
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
                timestamp_writes: self.render_stamps(
                    q.filter(|_| mode < 4),
                    q.filter(|_| mode == 0 || mode >= 4).map(|x| x + 1),
                ),
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.full);
            pass.set_bind_group(0, &self.input_group, &[]);
            pass.set_vertex_buffer(0, self.vertex.slice(..));
            pass.set_index_buffer(self.index.slice(..), wgpu::IndexFormat::Uint32);
            for d in &p.obstacles {
                pass.draw_indexed(d.indices.clone(), 0, d.instance..d.instance + d.count);
            }
            if mode == 0 {
                for d in &p.baseline {
                    pass.draw_indexed(d.indices.clone(), 0, d.instance..d.instance + d.count);
                }
            }
            if mode >= 4 {
                pass.execute_bundles([if mode == 4 {
                    &self.root_bundle
                } else {
                    &self.region_bundle
                }]);
            }
        }
        if mode == 0 || mode >= 4 {
            return;
        }
        if mode >= 2 {
            for i in 0..t.levels {
                let mut pass = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("depth minimum reduction"),
                    timestamp_writes: self.compute_stamps(
                        q.filter(|_| i == 0).map(|x| x + 2),
                        q.filter(|_| i == t.levels - 1).map(|x| x + 3),
                    ),
                });
                pass.set_pipeline(if i == 0 {
                    &self.base_pipeline
                } else {
                    &self.reduce_pipeline
                });
                pass.set_bind_group(0, &t.reduce_groups[i as usize], &[]);
                pass.dispatch_workgroups(
                    (t.base[0] >> i).max(1).div_ceil(8),
                    (t.base[1] >> i).max(1).div_ceil(8),
                    1,
                );
            }
        }
        self.cull(e, mode, q);
        {
            let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GPU selected geometry"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: self.render_stamps(None, q.map(|x| x + 1)),
                occlusion_query_set: None,
            });
            pass.execute_bundles([if matches!(mode, 2 | 4) {
                &self.root_bundle
            } else {
                &self.region_bundle
            }]);
        }
    }
    fn cull(&self, e: &mut wgpu::CommandEncoder, mode: u32, q: Option<u32>) {
        let t = self.target.as_ref().unwrap();
        {
            let mut pass = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("object visibility and instance compaction"),
                timestamp_writes: self.compute_stamps(
                    q.map(|x| if mode >= 4 { x } else { x + 4 }),
                    q.filter(|_| matches!(mode, 2 | 4)).map(|x| x + 5),
                ),
            });
            pass.set_pipeline(&self.roots_pipeline);
            pass.set_bind_group(0, &t.cull_group, &[]);
            pass.dispatch_workgroups(3, 1, 1);
        }
        if !matches!(mode, 2 | 4) {
            let mut pass = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("region visibility and instance compaction"),
                timestamp_writes: self.compute_stamps(None, q.map(|x| x + 5)),
            });
            pass.set_pipeline(&self.regions_pipeline);
            pass.set_bind_group(0, &t.cull_group, &[]);
            pass.dispatch_workgroups(self.regions, 1, 1);
        }
    }
    fn render_stamps(
        &self,
        start: Option<u32>,
        end: Option<u32>,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        if start.is_none() && end.is_none() {
            return None;
        }
        self.gpu
            .query
            .as_ref()
            .map(|s| wgpu::RenderPassTimestampWrites {
                query_set: s,
                beginning_of_pass_write_index: start,
                end_of_pass_write_index: end,
            })
    }
    fn compute_stamps(
        &self,
        start: Option<u32>,
        end: Option<u32>,
    ) -> Option<wgpu::ComputePassTimestampWrites<'_>> {
        if start.is_none() && end.is_none() {
            return None;
        }
        self.gpu
            .query
            .as_ref()
            .map(|s| wgpu::ComputePassTimestampWrites {
                query_set: s,
                beginning_of_pass_write_index: start,
                end_of_pass_write_index: end,
            })
    }
    fn submit(&self, e: wgpu::CommandEncoder) -> Result<()> {
        let id = self.gpu.queue.submit([e.finish()]);
        self.gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(id),
            timeout: Some(Duration::from_secs(60)),
        })?;
        Ok(())
    }
    pub fn capture(&mut self, p: &Pose, mode: u32) -> Result<(Vec<[f32; 4]>, serde_json::Value)> {
        self.target(p.camera.size);
        self.update(p, mode);
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        self.encode(&mut e, p, mode, None);
        let t = self.target.as_ref().unwrap();
        copy_readback(&mut e, &t.color, &t.read, self.size[0], self.size[1], 16);
        if mode > 0 {
            e.copy_buffer_to_buffer(
                &self.commands,
                0,
                &self.counter_read,
                0,
                self.commands.size(),
            );
        }
        self.submit(e)?;
        self.gpu.map(&t.read)?;
        let bytes = t.read.slice(..).get_mapped_range();
        let pixels = bytemuck::cast_slice::<u8, [f32; 4]>(&bytes).to_vec();
        drop(bytes);
        t.read.unmap();
        ensure!(
            pixels.iter().flatten().all(|x| x.is_finite()),
            "nonfinite pixels"
        );
        let mut triangles = p
            .obstacles
            .iter()
            .map(|d| u64::from(d.indices.end - d.indices.start) / 3)
            .sum::<u64>();
        let mut active = 0;
        let mut selected = 0;
        if mode > 0 {
            self.gpu.map(&self.counter_read)?;
            let bytes = self.counter_read.slice(..).get_mapped_range();
            let cmds = bytemuck::cast_slice::<u8, u32>(&bytes);
            let range = if matches!(mode, 2 | 4) {
                0..3
            } else {
                3..self.regions as usize + 3
            };
            for i in range {
                let c = &cmds[i * 5..i * 5 + 5];
                ensure!(c[1] <= 32, "compaction overflow");
                triangles += u64::from(c[0]) * u64::from(c[1]) / 3;
                active += usize::from(c[1] > 0);
                selected += c[1];
            }
            drop(bytes);
            self.counter_read.unmap();
        } else {
            triangles = p.triangles;
            active = p.baseline.iter().filter(|d| d.count > 0).count();
            selected = p.baseline.iter().map(|d| d.count).sum();
        }
        Ok((
            pixels,
            json!({"triangles":triangles,"active_geometry_commands":active,"submitted_geometry_commands":if matches!(mode,0|2|4){3}else{self.regions},"selected_instances":selected,"pyramid_bytes":t.pyramid_bytes,"input_upload_bytes":p.input.len()*32}),
        ))
    }
}
