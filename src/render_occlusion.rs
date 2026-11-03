//! Current-frame conservative depth tiles and stable GPU instance compaction.
//! Readback is an explicit benchmark diagnostic, never a rendering dependency.
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct Bound {
    pub rect: [f32; 4],
    pub nearest: f32,
    pub pad: [f32; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct Job {
    pub start: u32,
    pub count: u32,
    pub vertices: u32,
    pub indexed: u32,
}
struct Resources {
    width: u32,
    height: u32,
    instances: usize,
    jobs: usize,
    depth: wgpu::TextureView,
    tiles: wgpu::TextureView,
    fine: wgpu::TextureView,
    bounds: wgpu::Buffer,
    job_buffer: wgpu::Buffer,
    output: wgpu::Buffer,
    args: wgpu::Buffer,
    input: wgpu::Buffer,
    reduce_bind: wgpu::BindGroup,
    fine_bind: wgpu::BindGroup,
    coarse_bind: wgpu::BindGroup,
    compact_bind: wgpu::BindGroup,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Stats {
    pub active: bool,
    pub fine_tiles: bool,
    pub occluder_instances: usize,
    pub occluder_triangles: usize,
    pub candidate_instances: usize,
    pub batches: usize,
    pub image_bytes: u64,
    pub buffer_bytes: u64,
    /// Referenced instance buffer, usually shared with the main path; do not add twice.
    pub source_buffer_bytes: u64,
    pub depth_frame: &'static str,
}
pub(crate) struct Gpu {
    reduce: wgpu::ComputePipeline,
    compact: wgpu::ComputePipeline,
    reduce_fine: wgpu::ComputePipeline,
    coarse: wgpu::ComputePipeline,
    compact_fine: wgpu::ComputePipeline,
    pub fine_enabled: bool,
    resources: Option<Resources>,
    pub stats: Stats,
}
fn buffer(device: &wgpu::Device, size: usize, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("occlusion scratch"),
        size: size as u64,
        usage,
        mapped_at_creation: false,
    })
}
impl Gpu {
    pub fn new(device: &wgpu::Device) -> Self {
        // Explicit unfilterable R32 layout; auto layouts require float32 filtering.
        let reduce_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("conservative depth tiles"),
            source: wgpu::ShaderSource::Wgsl(include_str!("occlusion_tiles.wgsl").into()),
        });
        let reduce = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("conservative depth tiles"),
            layout: None,
            module: &reduce_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let reduce_fine = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("fine conservative tiles"),
            layout: None,
            module: &reduce_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[("tile_size", 4.)],
                ..Default::default()
            },
            cache: None,
        });
        let sampled = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let coarse_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("coarse tile reduction"),
                entries: &[
                    sampled(0),
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::R32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let coarse_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("coarse reduction"),
            bind_group_layouts: &[&coarse_bind_layout],
            push_constant_ranges: &[],
        });
        let coarse_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("coarse reduction"),
            source: wgpu::ShaderSource::Wgsl(include_str!("occlusion_coarse.wgsl").into()),
        });
        let coarse = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("coarse reduction"),
            layout: Some(&coarse_layout),
            module: &coarse_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let storage = |binding, read_only| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("occlusion compact"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(1, true),
                storage(2, true),
                storage(3, true),
                storage(4, false),
                storage(5, false),
                sampled(6),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("occlusion compact"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("stable compact"),
            source: wgpu::ShaderSource::Wgsl(include_str!("occlusion_compact.wgsl").into()),
        });
        let compact = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("stable compact"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let compact_fine = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("fine stable compact"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[("fine_tiles", 1.)],
                ..Default::default()
            },
            cache: None,
        });
        Self {
            reduce,
            compact,
            reduce_fine,
            coarse,
            compact_fine,
            fine_enabled: true,
            resources: None,
            stats: Default::default(),
        }
    }
    pub fn reset(&mut self) {
        // Cached resources remain resident even when this frame takes fallback.
        self.stats = Stats {
            image_bytes: self.stats.image_bytes,
            buffer_bytes: self.stats.buffer_bytes,
            source_buffer_bytes: self.stats.source_buffer_bytes,
            ..Default::default()
        };
    }
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input: &wgpu::Buffer,
        bounds: &[Bound],
        jobs: &[Job],
        width: u32,
        height: u32,
        occluders: usize,
        triangles: usize,
    ) {
        let needs = self.resources.as_ref().is_none_or(|r| {
            r.width != width
                || r.height != height
                || r.instances < bounds.len()
                || r.jobs < jobs.len()
        });
        if needs {
            let ni = bounds.len().next_power_of_two();
            let nj = jobs.len().next_power_of_two();
            let texture = |format, w, h, usage| {
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some("current occlusion depth"),
                        size: wgpu::Extent3d {
                            width: w,
                            height: h,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            };
            let depth = texture(
                wgpu::TextureFormat::Depth32Float,
                width,
                height,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            );
            let tiles = texture(
                wgpu::TextureFormat::R32Float,
                width.div_ceil(16),
                height.div_ceil(16),
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            );
            let fine = texture(
                wgpu::TextureFormat::R32Float,
                width.div_ceil(4),
                height.div_ceil(4),
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            );
            let bounds = buffer(
                device,
                ni * 32,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            let job_buffer = buffer(
                device,
                nj * 16,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            let output = buffer(
                device,
                ni * 80,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            );
            let args = buffer(
                device,
                nj * 32,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_SRC,
            );
            let reduce_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("reduce depth"),
                layout: &self.reduce.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&tiles),
                    },
                ],
            });
            let fine_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("reduce fine depth"),
                layout: &self.reduce_fine.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&fine),
                    },
                ],
            });
            let coarse_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("reduce coarse tiles"),
                layout: &self.coarse.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&fine),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&tiles),
                    },
                ],
            });
            let compact_bind = self.bind(
                device,
                input,
                &tiles,
                &fine,
                &bounds,
                &job_buffer,
                &output,
                &args,
            );
            self.resources = Some(Resources {
                width,
                height,
                instances: ni,
                jobs: nj,
                depth,
                tiles,
                fine,
                bounds,
                job_buffer,
                output,
                args,
                input: input.clone(),
                reduce_bind,
                fine_bind,
                coarse_bind,
                compact_bind,
            });
        } else if self.resources.as_ref().unwrap().input != *input {
            let r = self.resources.as_ref().unwrap();
            let bind = self.bind(
                device,
                input,
                &r.tiles,
                &r.fine,
                &r.bounds,
                &r.job_buffer,
                &r.output,
                &r.args,
            );
            let r = self.resources.as_mut().unwrap();
            r.input = input.clone();
            r.compact_bind = bind;
        }
        let r = self.resources.as_ref().unwrap();
        queue.write_buffer(&r.bounds, 0, bytemuck::cast_slice(bounds));
        queue.write_buffer(&r.job_buffer, 0, bytemuck::cast_slice(jobs));
        self.stats = Stats {
            active: true,
            // Fine reduction only earns its extra pass when candidate geometry
            // is dense relative to the actual framebuffer. Diagnostic OFF keeps
            // the original16px pipeline as an A/B reference.
            fine_tiles: self.fine_enabled
                && jobs
                    .iter()
                    .map(|j| u64::from(j.count) * u64::from(j.vertices) / 3)
                    .sum::<u64>()
                    > u64::from(width) * u64::from(height) * 32,
            occluder_instances: occluders,
            occluder_triangles: triangles,
            candidate_instances: jobs.iter().map(|j| j.count as usize).sum(),
            batches: jobs.len(),
            image_bytes: 4
                * (width as u64 * height as u64
                    + width.div_ceil(16) as u64 * height.div_ceil(16) as u64
                    + width.div_ceil(4) as u64 * height.div_ceil(4) as u64),
            buffer_bytes: r.bounds.size() + r.job_buffer.size() + r.output.size() + r.args.size(),
            source_buffer_bytes: r.input.size(),
            depth_frame: "current",
        };
    }
    #[allow(clippy::too_many_arguments)]
    fn bind(
        &self,
        device: &wgpu::Device,
        input: &wgpu::Buffer,
        tiles: &wgpu::TextureView,
        fine: &wgpu::TextureView,
        bounds: &wgpu::Buffer,
        jobs: &wgpu::Buffer,
        output: &wgpu::Buffer,
        args: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compact visible instances"),
            layout: &self.compact.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(tiles),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bounds.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: jobs.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: args.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(fine),
                },
            ],
        })
    }
    pub fn depth(&self) -> &wgpu::TextureView {
        &self.resources.as_ref().unwrap().depth
    }
    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.stats.batches == 0 {
            return;
        }
        let r = self.resources.as_ref().unwrap();
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("conservative tiles"),
                timestamp_writes: None,
            });
            if self.stats.fine_tiles {
                pass.set_pipeline(&self.reduce_fine);
                pass.set_bind_group(0, &r.fine_bind, &[]);
                pass.dispatch_workgroups(r.width.div_ceil(32), r.height.div_ceil(32), 1);
            } else {
                pass.set_pipeline(&self.reduce);
                pass.set_bind_group(0, &r.reduce_bind, &[]);
                pass.dispatch_workgroups(r.width.div_ceil(128), r.height.div_ceil(128), 1);
            }
        }
        if self.stats.fine_tiles {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("exact coarse tiles"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.coarse);
            pass.set_bind_group(0, &r.coarse_bind, &[]);
            pass.dispatch_workgroups(r.width.div_ceil(128), r.height.div_ceil(128), 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("stable GPU culling"),
                timestamp_writes: None,
            });
            pass.set_pipeline(if self.stats.fine_tiles {
                &self.compact_fine
            } else {
                &self.compact
            });
            pass.set_bind_group(0, &r.compact_bind, &[]);
            pass.dispatch_workgroups(self.stats.batches as u32, 1, 1);
        }
    }
    pub fn output(&self) -> &wgpu::Buffer {
        &self.resources.as_ref().unwrap().output
    }
    pub fn args(&self) -> &wgpu::Buffer {
        &self.resources.as_ref().unwrap().args
    }
    pub fn diagnostic(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<serde_json::Value> {
        if !self.stats.active {
            return Ok(serde_json::json!({"active":false}));
        }
        if self.stats.batches == 0 {
            return Ok(serde_json::json!({"active":true,"batches":[]}));
        }
        let size = (self.stats.batches * 32) as u64;
        let read = buffer(
            device,
            size as usize,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(self.args(), 0, &read, 0, size);
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;
        rx.recv()??;
        let bytes = read.slice(..).get_mapped_range();
        let words: &[u32] = bytemuck::cast_slice(&bytes);
        let mut kept = 0u64;
        let mut total = 0u64;
        let mut triangles = 0u64;
        let mut rejected_triangles = 0u64;
        for a in words.chunks_exact(8) {
            kept += a[1] as u64;
            total += a[5] as u64;
            triangles += a[0] as u64 / 3 * a[1] as u64;
            rejected_triangles += a[0] as u64 / 3 * (a[5] - a[1]) as u64;
        }
        Ok(
            serde_json::json!({"active":true,"candidate_instances":total,"kept_instances":kept,"rejected_instances":total-kept,"kept_shared_triangles":triangles,"rejected_shared_triangles":rejected_triangles,"diagnostic_readback":true}),
        )
    }
}
