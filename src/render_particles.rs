//! Six generated vertices per sprite; the CPU uploads only the instance stream.
use crate::particles::{Document, Frame};
use std::sync::Arc;
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    center_size: [f32; 4],
    velocity_angle: [f32; 4],
    color: [f32; 4],
    shape: [f32; 4],
    material: [u32; 4],
}
pub(crate) struct Gpu {
    pipeline: wgpu::RenderPipeline,
    buffer: Option<wgpu::Buffer>,
    capacity: usize,
    staging: Vec<Instance>,
    key: Option<(Arc<Document>, u64, u32)>,
    pub uploads: u64,
}
impl Gpu {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
    ) -> Self {
        let attributes = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x4,3=>Float32x4,4=>Uint32x4];
        let buffers = [wgpu::VertexBufferLayout {
            array_stride: 80,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &attributes,
        }];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("instanced particle sprites"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_particle"),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_particle"),
                compilation_options: Default::default(),
                targets: &crate::render_transparency::targets(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        Self {
            pipeline,
            buffer: None,
            capacity: 0,
            staging: vec![],
            key: None,
            uploads: 0,
        }
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &Frame,
        base: u32,
    ) {
        let Some((doc, tick)) = frame.cache_key() else {
            self.staging.clear();
            self.key = None;
            return;
        };
        if self
            .key
            .as_ref()
            .is_some_and(|(d, t, b)| Arc::ptr_eq(d, doc) && *t == tick && *b == base)
        {
            return;
        }
        self.staging.clear();
        self.staging.extend(frame.samples.iter().map(|p| Instance {
            center_size: [p.position[0], p.position[1], p.position[2], p.size],
            velocity_angle: [p.velocity[0], p.velocity[1], p.velocity[2], p.angle],
            color: p.color,
            shape: [p.aspect, p.softness, f32::from(p.align_velocity), p.age],
            material: [base + p.material, 0, 0, 0],
        }));
        if !self.staging.is_empty() {
            if self.staging.len() > self.capacity {
                self.capacity = self.staging.len().next_power_of_two();
                self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("particle instance stream"),
                    size: self.capacity as u64 * 80,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            queue.write_buffer(
                self.buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.staging),
            );
            self.uploads += 1;
        }
        self.key = Some((doc.clone(), tick, base));
    }
    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if !self.staging.is_empty() {
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.buffer.as_ref().unwrap().slice(..));
            pass.draw(0..6, 0..self.staging.len() as u32);
        }
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"alive":self.staging.len(),"instance_stride":80,"capacity_bytes":self.capacity*80,"uploads":self.uploads,"draws":usize::from(!self.staging.is_empty())})
    }
}
