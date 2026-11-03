//! Bounded, order-independent surface composition. No geometry-specific rules.
#[derive(Default)]
pub(crate) struct Transparency {
    gpu: Option<Gpu>,
    targets: Option<Targets>,
    pub draw_units: usize,
}
struct Gpu {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}
struct Targets {
    size: (u32, u32),
    background: wgpu::TextureView,
    layers: [wgpu::TextureView; 3],
    group: wgpu::BindGroup,
}
pub(crate) fn targets() -> [Option<wgpu::ColorTargetState>; 3] {
    let add = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    let multiply = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::Src,
        operation: wgpu::BlendOperation::Add,
    };
    [add, multiply, add].map(|component| {
        Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Rgba16Float,
            blend: Some(wgpu::BlendState {
                color: component,
                alpha: component,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })
    })
}
impl Transparency {
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        draw_units: usize,
    ) {
        self.draw_units = draw_units;
        if draw_units == 0 {
            self.targets = None;
            return;
        }
        let gpu = self.gpu.get_or_insert_with(|| {
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("surface composition inputs"),
                entries: &(0..4)
                    .map(|binding| wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>(),
            });
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("surface transparency composition"),
                source: wgpu::ShaderSource::Wgsl(include_str!("render_transparency.wgsl").into()),
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("surface composition"),
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("surface composition"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
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
            });
            Gpu { layout, pipeline }
        });
        if self
            .targets
            .as_ref()
            .is_some_and(|t| t.size == (width, height))
        {
            return;
        }
        let image = |name, format| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(name),
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
        let background = image("opaque background for surface composition", format);
        let layers = [
            "weighted surface radiance",
            "colored surface transmission",
            "additive surface radiance",
        ]
        .map(|name| image(name, wgpu::TextureFormat::Rgba16Float));
        let views = [&background, &layers[0], &layers[1], &layers[2]];
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("surface composition inputs"),
            layout: &gpu.layout,
            entries: &views
                .iter()
                .enumerate()
                .map(|(binding, view)| wgpu::BindGroupEntry {
                    binding: binding as u32,
                    resource: wgpu::BindingResource::TextureView(view),
                })
                .collect::<Vec<_>>(),
        });
        self.targets = Some(Targets {
            size: (width, height),
            background,
            layers,
            group,
        });
    }
    pub fn background(&self) -> Option<&wgpu::TextureView> {
        self.targets.as_ref().map(|t| &t.background)
    }
    pub fn layers(&self) -> Option<&[wgpu::TextureView; 3]> {
        self.targets.as_ref().map(|t| &t.layers)
    }
    pub fn composite(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let (Some(gpu), Some(t)) = (&self.gpu, &self.targets) else {
            return;
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("composite surface transparency"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&gpu.pipeline);
        pass.set_bind_group(0, &t.group, &[]);
        pass.draw(0..3, 0..1);
    }
    pub fn info(&self) -> serde_json::Value {
        serde_json::json!({"active":self.targets.is_some(),"draw_units":self.draw_units,"method":"weighted_colored_transmission","image_bytes":self.targets.as_ref().map_or(0,|t|u64::from(t.size.0)*u64::from(t.size.1)*28),"spatial_refraction":false})
    }
}
