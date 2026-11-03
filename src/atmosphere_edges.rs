//! Optional bounded screen-space edge finish. Geometry selection remains independent.
use crate::{atmosphere::EdgeSoftness, model::Camera};

#[derive(Default)]
pub(crate) struct Edges {
    gpu: Option<Gpu>,
    targets: Option<Targets>,
    settings: Option<(u32, u32, f32, EdgeSoftness)>,
}
struct Gpu {
    horizontal: wgpu::RenderPipeline,
    vertical: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
}
struct Targets {
    image_bytes: u64,
    width: u32,
    height: u32,
    source: wgpu::TextureView,
    horizontal: wgpu::TextureView,
    groups: [wgpu::BindGroup; 2],
}
impl Gpu {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("atmospheric edge finish"),
            source: wgpu::ShaderSource::Wgsl(include_str!("atmosphere_edges.wgsl").into()),
        });
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("edge finish inputs"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    2,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edge finish"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = |entry_point| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let horizontal = pipeline("fs_horizontal");
        let vertical = pipeline("fs_vertical");
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edge finish settings"),
            size: 96,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            horizontal,
            vertical,
            layout,
            uniform,
        }
    }
}
impl Edges {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        depth: &wgpu::TextureView,
        width: u32,
        height: u32,
        camera: &Camera,
        settings: Option<EdgeSoftness>,
    ) {
        let Some(settings) = settings.filter(|s| s.active()) else {
            self.targets = None;
            self.settings = None;
            return;
        };
        let gpu = self.gpu.get_or_insert_with(|| Gpu::new(device, format));
        if self
            .targets
            .as_ref()
            .is_none_or(|t| t.width != width || t.height != height)
        {
            let view = |label| {
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some(label),
                        size: wgpu::Extent3d {
                            width,
                            height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            };
            let source = view("edge finish source");
            let horizontal = view("edge finish horizontal");
            let groups = [&source, &horizontal].map(|color| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("edge finish inputs"),
                    layout: &gpu.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(color),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(depth),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: gpu.uniform.as_entire_binding(),
                        },
                    ],
                })
            });
            self.targets = Some(Targets {
                image_bytes: u64::from(width)
                    * u64::from(height)
                    * u64::from(
                        format
                            .block_copy_size(None)
                            .expect("color format block size"),
                    )
                    * 2,
                width,
                height,
                source,
                horizontal,
                groups,
            });
        }
        let key = (width, height, camera.fov_degrees, settings);
        if self.settings != Some(key) {
            let mut data = [[0f32; 4]; 6];
            data[0] = [
                settings.strength,
                settings.radius_px as f32,
                settings.start,
                settings.end,
            ];
            let (n, f) = (crate::picking::NEAR, crate::picking::FAR);
            let tan = (camera.fov_degrees.to_radians() * 0.5).tan();
            data[1] = [
                f / (f - n),
                f * n / (f - n),
                tan * width as f32 / height as f32,
                tan,
            ];
            let sigma = (settings.radius_px as f32 / 3.).max(0.5);
            for i in 0..=12 {
                data[2 + i / 4][i % 4] = (-0.5 * (i as f32 / sigma).powi(2)).exp();
            }
            queue.write_buffer(&gpu.uniform, 0, bytemuck::cast_slice(&data));
            self.settings = Some(key);
        }
    }
    pub fn source(&self) -> Option<&wgpu::TextureView> {
        self.targets.as_ref().map(|t| &t.source)
    }
    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let (Some(gpu), Some(t)) = (&self.gpu, &self.targets) else {
            return;
        };
        for (pipeline, group, target) in [
            (&gpu.horizontal, &t.groups[0], &t.horizontal),
            (&gpu.vertical, &t.groups[1], output),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("atmospheric edge finish"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    pub fn info(&self) -> serde_json::Value {
        serde_json::json!({"active":self.targets.is_some(),"passes":if self.targets.is_some(){2}else{0},
            "image_bytes":self.targets.as_ref().map_or(0,|t|t.image_bytes),
            "pipeline_cached":self.gpu.is_some(),"cached_uniform_bytes":if self.gpu.is_some(){96}else{0},
            "settings":self.settings.map(|s|s.3)})
    }
}
