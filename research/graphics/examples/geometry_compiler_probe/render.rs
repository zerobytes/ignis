use super::compiler::{Compiled, Mesh, Vertex};
use anyhow::{Result, ensure};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use serde::Serialize;
use std::time::Instant;
use wgpu::util::DeviceExt;
const BATCH_FRAMES: usize = 128;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniform {
    matrix: [[f32; 4]; 4],
    eye: [f32; 4],
    light: [f32; 4],
    material: [f32; 4],
}
#[derive(Clone, Copy)]
pub struct Pose {
    pub angle: f32,
    pub distance: f32,
    pub light: f32,
}
impl Pose {
    pub(crate) fn eye(self) -> Vec3 {
        Vec3::new(self.angle.cos() * 0.85, self.angle.sin() * 0.85, 0.53).normalize()
            * self.distance
    }
}
pub struct Bound {
    pub low: Vec3,
    pub high: Vec3,
    pub error: f32,
    pub normal_error: f32,
    pub reduces: bool,
}
impl Bound {
    pub fn new(c: &Compiled) -> Self {
        let mut b = Self {
            low: Vec3::splat(f32::INFINITY),
            high: Vec3::splat(f32::NEG_INFINITY),
            error: 0.,
            normal_error: 0.,
            reduces: c.proxy.faces.len() < c.source.faces.len(),
        };
        for r in &c.regions {
            b.low = b.low.min(Vec3::from_array(r.low));
            b.high = b.high.max(Vec3::from_array(r.high));
            b.error = b.error.max(r.error);
            b.normal_error = b.normal_error.max(r.normal_error);
        }
        b
    }
    pub fn pixels(&self, p: Pose, height: u32) -> f32 {
        let eye = p.eye();
        let forward = -eye.normalize();
        let mut depth = f32::INFINITY;
        let mut radius = 0f32;
        for bits in 0..8 {
            let v = Vec3::new(
                if bits & 1 == 0 {
                    self.low.x
                } else {
                    self.high.x
                },
                if bits & 2 == 0 {
                    self.low.y
                } else {
                    self.high.y
                },
                if bits & 4 == 0 {
                    self.low.z
                } else {
                    self.high.z
                },
            ) - eye;
            let z = v.dot(forward);
            depth = depth.min(z);
            radius = radius.max((v - forward * z).length());
        }
        let e = self.error;
        if depth <= e + 0.1 {
            return f32::INFINITY;
        }
        let focal = height as f32 / (2. * (std::f32::consts::FRAC_PI_4 / 2.).tan());
        focal * e * (1. + radius / depth) / (depth - e)
    }
    pub fn choose(&self, p: Pose, height: u32) -> bool {
        self.reduces && self.normal_error <= 0.000101 && self.pixels(p, height) <= 0.15
    }
}
pub struct Uploaded {
    v: wgpu::Buffer,
    i: wgpu::Buffer,
    uv: Option<wgpu::Buffer>,
    ranges: [std::ops::Range<u32>; 3],
    pub bytes: usize,
}
pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    normal_pipeline: Option<wgpu::RenderPipeline>,
    normal_group: Option<wgpu::BindGroup>,
    normal_enabled: bool,
    rich_inputs: Option<RichInputs>,
    rich_faces: Option<RichFaces>,
    rich_faces_enabled: bool,
    indirect: Option<wgpu::Buffer>,
    indirect_enabled: bool,
    uniform: wgpu::Buffer,
    group: wgpu::BindGroup,
    color: wgpu::Texture,
    view: wgpu::TextureView,
    _depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
    query: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    pub width: u32,
    pub height: u32,
    pub adapter: String,
}
#[derive(Serialize)]
pub struct Timing {
    pub frames: usize,
    pub gpu_ms: f64,
    pub cpu_encode_ms: f64,
    pub total_batch_ms_per_frame: f64,
    pub proxy_frames: usize,
}
impl Gpu {
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        Self::new_features(width, height, wgpu::Features::empty()).await
    }
    pub async fn new_features(width: u32, height: u32, extra: wgpu::Features) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await?;
        ensure!(
            adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY),
            "timestamp queries required"
        );
        ensure!(
            adapter.features().contains(extra),
            "requested diagnostic GPU features unavailable"
        );
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::TIMESTAMP_QUERY | extra,
                ..Default::default()
            })
            .await?;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(112),
                },
                count: None,
            }],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (BATCH_FRAMES * 3 * 256) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform,
                    offset: 0,
                    size: wgpu::BufferSize::new(112),
                }),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 24,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
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
        let texture = |format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let color = texture(
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let view = color.create_view(&Default::default());
        let depth = texture(
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let depth_view = depth.create_view(&Default::default());
        let query = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: (BATCH_FRAMES * 2) as u32,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (BATCH_FRAMES * 16) as u64,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (BATCH_FRAMES * 16) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(Self {
            device,
            queue,
            pipeline,
            normal_pipeline: None,
            rich_inputs: None,
            rich_faces: None,
            rich_faces_enabled: false,
            normal_group: None,
            normal_enabled: false,
            indirect: None,
            indirect_enabled: false,
            uniform,
            group,
            color,
            view,
            _depth: depth,
            depth_view,
            query,
            resolve,
            read,
            width,
            height,
            adapter: format!("{} / {:?}", info.name, info.backend),
        })
    }
    #[allow(dead_code)]
    pub fn enable_indirect(&mut self, enabled: bool) {
        if enabled && self.indirect.is_none() {
            self.indirect = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("grouped-draw-arguments"),
                size: (BATCH_FRAMES * 4096 * 20) as u64,
                usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        self.indirect_enabled = enabled;
    }
    pub fn upload(&self, m: &Mesh) -> Uploaded {
        let mut indices = vec![];
        let ranges = std::array::from_fn(|material| {
            let start = indices.len() as u32;
            for f in &m.faces {
                if f[3] as usize == material {
                    indices.extend_from_slice(&f[..3]);
                }
            }
            start..indices.len() as u32
        });
        let v = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice::<Vertex, u8>(&m.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let i = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice::<u32, u8>(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        Uploaded {
            v,
            i,
            uv: None,
            ranges,
            bytes: m.vertices.len() * 24 + indices.len() * 4,
        }
    }
    fn prepare(&self, p: Pose, frame: usize) {
        self.prepare_jitter(p, frame, [0., 0.]);
    }
    fn prepare_jitter(&self, p: Pose, frame: usize, jitter: [f32; 2]) {
        let eye = p.eye();
        let mut matrix = Mat4::perspective_infinite_reverse_rh(
            std::f32::consts::FRAC_PI_4,
            self.width as f32 / self.height as f32,
            0.1,
        ) * Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Z);
        if jitter != [0., 0.] {
            matrix = Mat4::from_translation(Vec3::new(
                -2. * jitter[0] / self.width as f32,
                2. * jitter[1] / self.height as f32,
                0.,
            )) * matrix;
        }
        let light = Vec3::new(p.light.cos(), p.light.sin(), 0.6).normalize();
        for mat in 0..3 {
            let u = Uniform {
                matrix: matrix.to_cols_array_2d(),
                eye: eye.extend(0.).to_array(),
                light: light.extend(0.).to_array(),
                material: [mat as f32, self.width as f32, self.height as f32, 0.],
            };
            self.queue.write_buffer(
                &self.uniform,
                ((frame * 3 + mat) * 256) as u64,
                bytemuck::bytes_of(&u),
            );
        }
    }
    fn draw(&self, encoder: &mut wgpu::CommandEncoder, m: &Uploaded, frame: usize, timed: bool) {
        self.draw_selection(encoder, m, frame, timed, None);
    }
    fn draw_selection(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        m: &Uploaded,
        frame: usize,
        timed: bool,
        ranges: Option<&[(u32, u32)]>,
    ) {
        let attachment = |view| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        let diagnostic = self.rich_inputs.as_ref().filter(|_| self.normal_enabled);
        let attachments = [
            attachment(&self.view),
            diagnostic.and_then(|d| attachment(&d.view)),
        ];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &attachments[..if diagnostic.is_some() { 2 } else { 1 }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: timed.then_some(wgpu::RenderPassTimestampWrites {
                query_set: &self.query,
                beginning_of_pass_write_index: Some(frame as u32 * 2),
                end_of_pass_write_index: Some(frame as u32 * 2 + 1),
            }),
            occlusion_query_set: None,
        });
        if self.normal_enabled {
            pass.set_pipeline(
                self.normal_pipeline
                    .as_ref()
                    .expect("normal pipeline installed"),
            );
            pass.set_bind_group(1, self.normal_group.as_ref().unwrap(), &[]);
            if self.rich_faces_enabled {
                assert_eq!(
                    ranges,
                    Some(&[(0, self.rich_faces.as_ref().unwrap().face_count * 3)][..]),
                    "face metadata requires one full source draw starting at zero"
                );
                pass.set_bind_group(2, &self.rich_faces.as_ref().unwrap().group, &[]);
            }
            // Some material pipelines derive UV from position and need no UV stream.
            if let Some(uv) = &m.uv {
                pass.set_vertex_buffer(1, uv.slice(..));
            }
        } else {
            pass.set_pipeline(&self.pipeline);
        }
        pass.set_vertex_buffer(0, m.v.slice(..));
        pass.set_index_buffer(m.i.slice(..), wgpu::IndexFormat::Uint32);
        if let Some(ranges) = ranges {
            pass.set_bind_group(0, &self.group, &[(frame * 3 * 256) as u32]);
            if self.indirect_enabled {
                assert!(ranges.len() <= 4096, "indirect command capacity");
                let args = ranges
                    .iter()
                    .map(|&(start, count)| [count, 1, start, 0, 0])
                    .collect::<Vec<_>>();
                let offset = (frame * 4096 * 20) as u64;
                let buffer = self.indirect.as_ref().unwrap();
                self.queue
                    .write_buffer(buffer, offset, bytemuck::cast_slice(&args));
                pass.multi_draw_indexed_indirect(buffer, offset, ranges.len() as u32);
            } else {
                for &(start, count) in ranges {
                    pass.draw_indexed(start..start + count, 0, 0..1);
                }
            }
            return;
        }
        for mat in 0..3 {
            if !m.ranges[mat].is_empty() {
                pass.set_bind_group(0, &self.group, &[((frame * 3 + mat) * 256) as u32]);
                pass.draw_indexed(m.ranges[mat].clone(), 0, 0..1);
            }
        }
    }
    fn mapped(&self, b: &wgpu::Buffer) -> Result<Vec<u8>> {
        let (tx, rx) = std::sync::mpsc::channel();
        b.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::PollType::wait_indefinitely())?;
        rx.recv()??;
        let bytes = b.slice(..).get_mapped_range().to_vec();
        b.unmap();
        Ok(bytes)
    }
    pub fn capture(&self, m: &Uploaded, p: Pose) -> Result<Vec<[f32; 4]>> {
        self.capture_selected(m, p, None)
    }
    pub fn capture_selected(
        &self,
        m: &Uploaded,
        p: Pose,
        ranges: Option<&[(u32, u32)]>,
    ) -> Result<Vec<[f32; 4]>> {
        self.capture_region(m, p, [0, 0, self.width, self.height], [0., 0.], ranges)
    }
    /// Diagnostic readback: preserves the full viewport and reads only a rectangle.
    #[allow(dead_code)]
    pub fn capture_region(
        &self,
        m: &Uploaded,
        p: Pose,
        rect: [u32; 4],
        jitter: [f32; 2],
        ranges: Option<&[(u32, u32)]>,
    ) -> Result<Vec<[f32; 4]>> {
        let [x, y, width, height] = rect;
        ensure!(
            width > 0
                && height > 0
                && x.checked_add(width).is_some_and(|v| v <= self.width)
                && y.checked_add(height).is_some_and(|v| v <= self.height),
            "capture rectangle"
        );
        ensure!(
            jitter.iter().all(|v| v.is_finite() && v.abs() <= 0.5),
            "subpixel jitter"
        );
        self.prepare_jitter(p, 0, jitter);
        let stride = (width * 16).div_ceil(256) * 256;
        let b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: stride as u64 * height as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut e = self.device.create_command_encoder(&Default::default());
        self.draw_selection(&mut e, m, 0, false, ranges);
        e.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.color,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &b,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([e.finish()]);
        let bytes = self.mapped(&b)?;
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for row in bytes.chunks_exact(stride as usize) {
            for px in row[..width as usize * 16].chunks_exact(16) {
                pixels.push(std::array::from_fn(|i| {
                    f32::from_le_bytes(px[i * 4..i * 4 + 4].try_into().unwrap())
                }));
            }
        }
        Ok(pixels)
    }
    pub fn batch(
        &self,
        source: &Uploaded,
        proxy: &Uploaded,
        bound: &Bound,
        optional: bool,
        distance: f32,
        phase: f32,
    ) -> Result<Timing> {
        let start = Instant::now();
        let mut e = self.device.create_command_encoder(&Default::default());
        let mut proxy_frames = 0;
        for frame in 0..BATCH_FRAMES {
            let p = Pose {
                angle: phase + (frame % 32) as f32 * std::f32::consts::TAU / 32.,
                distance,
                light: phase + (frame % 32) as f32 * 0.37,
            };
            let selected = optional && bound.choose(p, self.height);
            proxy_frames += usize::from(selected);
            self.prepare(p, frame);
            self.draw(&mut e, if selected { proxy } else { source }, frame, true);
        }
        e.resolve_query_set(&self.query, 0..(BATCH_FRAMES * 2) as u32, &self.resolve, 0);
        e.copy_buffer_to_buffer(&self.resolve, 0, &self.read, 0, (BATCH_FRAMES * 16) as u64);
        self.queue.submit([e.finish()]);
        let cpu = start.elapsed().as_secs_f64() * 1000. / BATCH_FRAMES as f64;
        let bytes = self.mapped(&self.read)?;
        let total = start.elapsed().as_secs_f64() * 1000. / BATCH_FRAMES as f64;
        let times = bytes
            .chunks_exact(8)
            .map(|x| u64::from_le_bytes(x.try_into().unwrap()))
            .collect::<Vec<_>>();
        let begin = (0..BATCH_FRAMES).map(|i| times[i * 2]).min().unwrap();
        let end = (0..BATCH_FRAMES).map(|i| times[i * 2 + 1]).max().unwrap();
        Ok(Timing {
            frames: BATCH_FRAMES,
            gpu_ms: (end - begin) as f64 * self.queue.get_timestamp_period() as f64
                / (BATCH_FRAMES as f64 * 1e6),
            cpu_encode_ms: cpu,
            total_batch_ms_per_frame: total,
            proxy_frames,
        })
    }
    #[allow(dead_code)] // Used by the separate irregular-mesh research harness.
    pub fn batch_selected(
        &self,
        m: &Uploaded,
        poses: &[Pose],
        mut select: impl FnMut(Pose) -> Vec<(u32, u32)>,
    ) -> Result<Timing> {
        let start = Instant::now();
        let mut e = self.device.create_command_encoder(&Default::default());
        let proxy_frames = 0;
        for frame in 0..BATCH_FRAMES {
            let pose = poses[frame % poses.len()];
            let ranges = select(pose);
            self.prepare(pose, frame);
            self.draw_selection(&mut e, m, frame, true, Some(&ranges));
        }
        e.resolve_query_set(&self.query, 0..(BATCH_FRAMES * 2) as u32, &self.resolve, 0);
        e.copy_buffer_to_buffer(&self.resolve, 0, &self.read, 0, (BATCH_FRAMES * 16) as u64);
        self.queue.submit([e.finish()]);
        let cpu = start.elapsed().as_secs_f64() * 1000. / BATCH_FRAMES as f64;
        let bytes = self.mapped(&self.read)?;
        let total = start.elapsed().as_secs_f64() * 1000. / BATCH_FRAMES as f64;
        let times = bytes
            .chunks_exact(8)
            .map(|x| u64::from_le_bytes(x.try_into().unwrap()))
            .collect::<Vec<_>>();
        let begin = (0..BATCH_FRAMES).map(|i| times[i * 2]).min().unwrap();
        let end = (0..BATCH_FRAMES).map(|i| times[i * 2 + 1]).max().unwrap();
        Ok(Timing {
            frames: BATCH_FRAMES,
            gpu_ms: (end - begin) as f64 * self.queue.get_timestamp_period() as f64
                / (BATCH_FRAMES as f64 * 1e6),
            cpu_encode_ms: cpu,
            total_batch_ms_per_frame: total,
            proxy_frames,
        })
    }
    #[allow(dead_code)]
    pub fn batch_adaptive(
        &self,
        source: &Uploaded,
        proxy: &Uploaded,
        poses: &[Pose],
        bound: Option<&Bound>,
    ) -> Result<Timing> {
        self.batch_decision(source, proxy, poses, |p| {
            bound.is_some_and(|b| b.choose(p, self.height))
        })
    }
    pub fn batch_decision(
        &self,
        source: &Uploaded,
        proxy: &Uploaded,
        poses: &[Pose],
        mut choose: impl FnMut(Pose) -> bool,
    ) -> Result<Timing> {
        let start = Instant::now();
        let mut e = self.device.create_command_encoder(&Default::default());
        let mut proxy_frames = 0;
        for frame in 0..BATCH_FRAMES {
            let pose = poses[frame % poses.len()];
            let selected = choose(pose);
            proxy_frames += usize::from(selected);
            self.prepare(pose, frame);
            self.draw(&mut e, if selected { proxy } else { source }, frame, true);
        }
        e.resolve_query_set(&self.query, 0..(BATCH_FRAMES * 2) as u32, &self.resolve, 0);
        e.copy_buffer_to_buffer(&self.resolve, 0, &self.read, 0, (BATCH_FRAMES * 16) as u64);
        self.queue.submit([e.finish()]);
        let cpu = start.elapsed().as_secs_f64() * 1000. / BATCH_FRAMES as f64;
        let bytes = self.mapped(&self.read)?;
        let total = start.elapsed().as_secs_f64() * 1000. / BATCH_FRAMES as f64;
        let times = bytes
            .chunks_exact(8)
            .map(|x| u64::from_le_bytes(x.try_into().unwrap()))
            .collect::<Vec<_>>();
        let begin = (0..BATCH_FRAMES).map(|i| times[i * 2]).min().unwrap();
        let end = (0..BATCH_FRAMES).map(|i| times[i * 2 + 1]).max().unwrap();
        Ok(Timing {
            frames: BATCH_FRAMES,
            gpu_ms: (end - begin) as f64 * self.queue.get_timestamp_period() as f64
                / (BATCH_FRAMES as f64 * 1e6),
            cpu_encode_ms: cpu,
            total_batch_ms_per_frame: total,
            proxy_frames,
        })
    }
}

include!("../normal_field_probe/gpu.rs");

include!("../rich_relief_probe/gpu.rs");
