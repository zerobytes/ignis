//! Visible coverage of the actual selected surface; no material, bounding box or x-ray.
use crate::render::{RenderBox, RenderFrame, Vertex};
use bytemuck::Zeroable;
use glam::Mat4;

pub(crate) struct Outline {
    mask_pipeline: wgpu::RenderPipeline,
    edge_pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    camera_group: wgpu::BindGroup,
    mask_layout: wgpu::BindGroupLayout,
    target: Option<(u32, u32, wgpu::TextureView, wgpu::BindGroup)>,
    vertices: wgpu::Buffer,
    capacity: usize,
    positions: Vec<[f32; 3]>,
    key: Option<(u64, Option<String>, Option<RenderBox>)>,
}
impl Outline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("visible silhouette"),
            source: wgpu::ShaderSource::Wgsl(include_str!("outline_mask.wgsl").into()),
        });
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("outline camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("outline camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("outline camera"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let mask_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("outline mask"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let camera_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("outline mask"),
                bind_group_layouts: &[&camera_layout],
                push_constant_ranges: &[],
            });
        let edge_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("outline edge"),
            bind_group_layouts: &[&mask_layout],
            push_constant_ranges: &[],
        });
        let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("selected coverage"),
            layout: Some(&camera_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_mask"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 12,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0=>Float32x3],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_mask"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let edge_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("silhouette edge"),
            source: wgpu::ShaderSource::Wgsl(include_str!("outline.wgsl").into()),
        });
        let edge_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("selected silhouette edge"),
            layout: Some(&edge_layout),
            vertex: wgpu::VertexState {
                module: &edge_shader,
                entry_point: Some("vs_edge"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &edge_shader,
                entry_point: Some("fs_edge"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("outline positions"),
            size: 12,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            mask_pipeline,
            edge_pipeline,
            uniform,
            camera_group,
            mask_layout,
            target: None,
            vertices,
            capacity: 1,
            positions: Vec::new(),
            key: None,
        }
    }
    pub fn info(&self) -> serde_json::Value {
        serde_json::json!({"active":!self.positions.is_empty(),"triangles":self.positions.len()/3,
            "vertex_bytes":self.positions.len()*12,"allocated_vertex_bytes":self.capacity*12,
            "mask_bytes":self.target.as_ref().map_or(0,|t|u64::from(t.0)*u64::from(t.1))})
    }
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &RenderFrame) {
        let selected = frame.selection.filter(|s| {
            frame
                .boxes
                .iter()
                .any(|b| b.position == s.position && b.size == s.size && b.geometry == s.geometry)
        });
        let key = (
            frame.meshes.generation,
            frame.selection_mesh.clone(),
            selected,
        );
        if self.key.as_ref() == Some(&key) {
            return;
        }
        self.positions.clear();
        if let Some(id) = &frame.selection_mesh {
            if let Some(draw) = frame.meshes.draws.iter().find(|d| &d.instance == id) {
                self.positions.extend(frame.meshes.draw_positions(draw));
            }
        } else if let Some(selected) = selected {
            let mut vertices = [Vertex::zeroed(); 36];
            crate::render::box_vertices(&selected, &mut vertices);
            self.positions.extend(vertices.iter().map(|v| v.position));
        }
        self.key = Some(key);
        if self.positions.is_empty() {
            self.target = None;
            return;
        }
        let n = self.positions.len();
        if n > self.capacity || self.capacity > n.next_power_of_two() * 2 {
            self.capacity = n.next_power_of_two();
            self.vertices = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("outline positions"),
                size: (self.capacity * 12) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.vertices, 0, bytemuck::cast_slice(&self.positions));
    }
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        width: u32,
        height: u32,
        vp: Mat4,
    ) {
        if self.positions.is_empty() {
            return;
        }
        if self
            .target
            .as_ref()
            .is_none_or(|t| t.0 != width || t.1 != height)
        {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("selected silhouette mask"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let mask = texture.create_view(&Default::default());
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("outline mask"),
                layout: &self.mask_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&mask),
                }],
            });
            self.target = Some((width, height, mask, group));
        }
        queue.write_buffer(&self.uniform, 0, bytemuck::cast_slice(&vp.to_cols_array()));
        let (_, _, mask, group) = self.target.as_ref().unwrap();
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("selection visible coverage"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: mask,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            p.set_pipeline(&self.mask_pipeline);
            p.set_bind_group(0, &self.camera_group, &[]);
            p.set_vertex_buffer(0, self.vertices.slice(..));
            p.draw(0..self.positions.len() as u32, 0..1);
        }
        let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("selection edge"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        p.set_pipeline(&self.edge_pipeline);
        p.set_bind_group(0, group, &[]);
        p.draw(0..3, 0..1);
    }
}
