#[path = "views.rs"]
pub mod views;
use super::geometry::{Lobe, Mesh, TILE_COUNT, Vertex};
use super::selection;
use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use serde::Serialize;
use std::{
    cell::Cell,
    collections::HashMap,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Serialize)]
pub struct Case {
    pub distance: f32,
    pub tilt: f32,
    pub light_angle: f32,
    pub exponent: f32,
    pub specular: f32,
    pub shift: f32,
}
pub const FOV_Y: f32 = std::f32::consts::FRAC_PI_4;
pub const NEAR: f32 = 0.1;
impl Case {
    pub fn view_matrix(self) -> Mat4 {
        let angle = self.tilt.to_radians();
        let view = Vec3::new(angle.sin(), angle.cos(), 0.0);
        let center = Vec3::new(self.shift, 0.0, 0.0);
        Mat4::look_at_rh(center + view * self.distance, center, Vec3::Z)
    }
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    matrix: [[f32; 4]; 4],
    light: [f32; 4],
    view: [f32; 4],
    settings: [f32; 4],
    jitter: [f32; 4],
}
struct DrawOptions {
    mode: usize,
    index: Option<u32>,
    unreduced: bool,
    prepare: bool,
}
fn lighting_key(case: Case) -> [u32; 4] {
    [
        case.tilt.to_bits(),
        case.light_angle.to_bits(),
        case.exponent.to_bits(),
        case.specular.to_bits(),
    ]
}
struct Target {
    color: wgpu::Texture,
    color_view: wgpu::TextureView,
    _depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
    readback: wgpu::Buffer,
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct Transition {
    pub parent: u32,
    pub weight: f32,
    // [nearest active ancestor, reserved]; old modes ignore this extension.
    pub padding: [u32; 2],
}
struct IndexPlan {
    unreduced: Option<wgpu::Buffer>,
    full: wgpu::Buffer,
    proxy: wgpu::Buffer,
    full_count: u32,
    proxy_count: u32,
    proxy_capacity: usize,
}
pub struct Uploaded {
    transitions: wgpu::Buffer,
    compute_group: wgpu::BindGroup,
    transition_capacity: usize,
    active: Cell<usize>,
    cache_key: Cell<Option<[u32; 4]>>,
    indexed: Option<IndexPlan>,
    full: wgpu::Buffer,
    proxy: wgpu::Buffer,
    full_count: u32,
    proxy_count: u32,
    groups: [wgpu::BindGroup; 2],
    pub upload_ms: f64,
}
#[derive(Serialize)]
pub struct Timing {
    pub gpu_ms_median: Option<f64>,
    pub gpu_ms_min: Option<f64>,
    pub gpu_ms_max: Option<f64>,
    pub gpu_ms_p10: Option<f64>,
    pub gpu_ms_p90: Option<f64>,
    pub repeats: usize,
    pub preparation_ms_median: Option<f64>,
    pub draw_ms_median: Option<f64>,
}
pub struct Capture {
    pub pixels: Vec<[f32; 4]>,
    pub timing: Timing,
    pub prepared: bool,
}
pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    cached_pipeline: wgpu::RenderPipeline,
    full_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    compute_layout: wgpu::BindGroupLayout,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    targets: HashMap<u32, Target>,
    query: Option<wgpu::QuerySet>,
    query_resolve: wgpu::Buffer,
    query_readback: wgpu::Buffer,
    pub adapter: String,
    pub timestamp_period_ns: Option<f32>,
    pub jitter: [f32; 2],
    pub detailed: [bool; TILE_COUNT],
}
impl Gpu {
    pub async fn new() -> Result<Self> {
        Self::with_features(wgpu::Features::empty()).await
    }
    pub async fn with_features(extra: wgpu::Features) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        anyhow::ensure!(
            adapter.features().contains(extra),
            "requested experiment features unavailable: {extra:?}"
        );
        let info = adapter.get_info();
        let timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Appearance research offscreen"),
                required_features: extra
                    | if timestamps {
                        wgpu::Features::TIMESTAMP_QUERY
                    } else {
                        wgpu::Features::empty()
                    },
                ..Default::default()
            })
            .await?;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("summary evaluation"),
            entries: &(0..4)
                .map(|binding| wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: if binding == 0 {
                            wgpu::BufferBindingType::Uniform
                        } else {
                            wgpu::BufferBindingType::Storage {
                                read_only: binding != 3,
                            }
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                })
                .collect::<Vec<_>>(),
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("appearance probe"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Uint32];
        let make_pipeline = |entry| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Full and aggregate same response"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &attrs,
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let pipeline = make_pipeline("vs_main");
        let cached_pipeline = make_pipeline("vs_cached");
        let full_pipeline = make_pipeline("vs_full");
        let compute_source = include_str!("shader.wgsl").replace(
            "var<storage, read> evaluated:",
            "var<storage, read_write> evaluated:",
        );
        let compute_source = format!("{}\n{}", compute_source, include_str!("evaluate.wgsl"));
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("evaluate each active summary once"),
            source: wgpu::ShaderSource::Wgsl(compute_source.into()),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&compute_layout],
                push_constant_ranges: &[],
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("summary evaluation"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("evaluate"),
            compilation_options: Default::default(),
            cache: None,
        });
        let query = timestamps.then(|| {
            device.create_query_set(&wgpu::QuerySetDescriptor {
                label: None,
                ty: wgpu::QueryType::Timestamp,
                count: 1024,
            })
        });
        let query_resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 8192,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let query_readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 8192,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let timestamp_period_ns = timestamps.then(|| queue.get_timestamp_period());
        Ok(Self {
            device,
            queue,
            pipeline,
            cached_pipeline,
            full_pipeline,
            compute_pipeline,
            compute_layout,
            layout,
            uniform,
            targets: HashMap::new(),
            query,
            query_resolve,
            query_readback,
            adapter: format!("{} / {:?} / {}", info.name, info.backend, info.driver),
            timestamp_period_ns,
            jitter: [0.0; 2],
            detailed: [true; TILE_COUNT],
        })
    }
    pub fn upload(&self, mesh: &Mesh) -> Result<Uploaded> {
        self.upload_parts(&mesh.vertices, &mesh.proxy, &mesh.lobes32, &mesh.lobes128)
    }
    pub fn upload_hierarchy(
        &self,
        mesh: &Mesh,
        proxy: &[Vertex],
        lobes: &[Lobe],
    ) -> Result<Uploaded> {
        let mut uploaded = self.upload_parts(&mesh.vertices, proxy, lobes, lobes)?;
        let buffer = |count: usize| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("selected geometry indices"),
                size: (count.max(1) * 4) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        uploaded.indexed = Some(IndexPlan {
            unreduced: None,
            full: buffer(mesh.vertices.len()),
            proxy: buffer(proxy.len()),
            full_count: 0,
            proxy_count: 0,
            proxy_capacity: proxy.len(),
        });
        Ok(uploaded)
    }
    pub fn select_indices(&self, mesh: &mut Uploaded, full: &[u32], proxy: &[u32]) -> Result<()> {
        let plan = mesh.indexed.as_mut().context("indexed mesh required")?;
        anyhow::ensure!(
            full.len() <= mesh.full_count as usize && proxy.len() <= plan.proxy_capacity,
            "index capacity exceeded"
        );
        anyhow::ensure!(
            full.iter().all(|i| *i < mesh.full_count)
                && proxy.iter().all(|i| *i < mesh.proxy_count),
            "index outside uploaded vertices"
        );
        for (buffer, indices) in [(&plan.full, full), (&plan.proxy, proxy)] {
            if !indices.is_empty() {
                self.queue
                    .write_buffer(buffer, 0, bytemuck::cast_slice(indices));
            }
        }
        plan.full_count = full.len() as u32;
        plan.proxy_count = proxy.len() as u32;
        Ok(())
    }
    pub fn set_unreduced_order(&self, mesh: &mut Uploaded, indices: &[u32]) -> Result<()> {
        anyhow::ensure!(
            indices.len() == mesh.full_count as usize,
            "full control must cover all vertices"
        );
        let mut seen = vec![false; indices.len()];
        for &index in indices {
            anyhow::ensure!(
                (index as usize) < seen.len() && !seen[index as usize],
                "invalid full control permutation"
            );
            seen[index as usize] = true;
        }
        let plan = mesh.indexed.as_mut().context("indexed mesh required")?;
        plan.unreduced = Some(
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("unreduced control in identical leaf order"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        );
        Ok(())
    }
    pub fn set_transitions(&self, mesh: &Uploaded, states: &[Transition]) -> Result<()> {
        anyhow::ensure!(
            states.len() == mesh.transition_capacity,
            "transition capacity mismatch"
        );
        // Parents must precede children: bounds + acyclicity, validated before upload.
        anyhow::ensure!(
            states
                .iter()
                .enumerate()
                .all(|(id, s)| (s.parent == u32::MAX || (s.parent as usize) < id)
                    && s.weight.is_finite()
                    && (0.0..=1.0).contains(&s.weight)),
            "invalid transition state"
        );
        anyhow::ensure!(
            states.iter().all(|s| {
                let p = s.parent;
                let expected = if p == u32::MAX {
                    u32::MAX
                } else if states[p as usize].weight > 0. {
                    p
                } else {
                    states[p as usize].padding[0]
                };
                s.padding[0] == expected
            }),
            "invalid active-ancestor link"
        );
        mesh.active
            .set(states.iter().filter(|s| s.weight > 0.).count());
        self.queue
            .write_buffer(&mesh.transitions, 0, bytemuck::cast_slice(states));
        Ok(())
    }
    fn upload_parts(
        &self,
        vertices: &[Vertex],
        proxy_vertices: &[Vertex],
        lobes32: &[Lobe],
        lobes128: &[Lobe],
    ) -> Result<Uploaded> {
        let start = Instant::now();
        let full = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("full microgeometry"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let proxy = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fixed tile proxy"),
                contents: bytemuck::cast_slice(proxy_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let transition_capacity = (lobes32.len() / 32).max(1);
        let transitions = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bounded hierarchy transition state"),
            size: (transition_capacity * std::mem::size_of::<Transition>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let evaluated = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("evaluated summary radiance"),
            size: (transition_capacity * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let group = |lobes: &[Lobe]| {
            let buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("joint normal material distribution"),
                    contents: bytemuck::cast_slice(lobes),
                    usage: wgpu::BufferUsages::STORAGE,
                });
            let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: transitions.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: evaluated.as_entire_binding(),
                    },
                ],
            });
            (group, buffer)
        };
        let (first, lobe_buffer) = group(lobes32);
        let compute_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("active summaries"),
            layout: &self.compute_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: lobe_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: transitions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: evaluated.as_entire_binding(),
                },
            ],
        });
        let second = if std::ptr::eq(lobes32, lobes128) {
            first.clone()
        } else {
            group(lobes128).0
        };
        let groups = [first, second];
        let submission = self.queue.submit([]);
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(60)),
        })?;
        Ok(Uploaded {
            transitions,
            compute_group,
            transition_capacity,
            active: Cell::new(0),
            cache_key: Cell::new(None),
            indexed: None,
            full,
            proxy,
            full_count: vertices.len() as u32,
            proxy_count: proxy_vertices.len() as u32,
            groups,
            upload_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
    fn ensure_target(&mut self, size: u32) {
        if self.targets.contains_key(&size) {
            return;
        }
        let extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        let texture = |format, usage| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: extent,
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
        let depth = texture(
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let color_view = color.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(size) * u64::from(size) * 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.targets.insert(
            size,
            Target {
                color,
                color_view,
                _depth: depth,
                depth_view,
                readback,
            },
        );
    }
    // 0 full; 1 flat; 2 distribution32; 3 distribution128; 4 adaptive32; 5 selection colors.
    pub fn render(
        &mut self,
        mesh: &Uploaded,
        case: Case,
        mode: usize,
        size: u32,
        samples: u32,
        benchmark: bool,
    ) -> Result<Capture> {
        anyhow::ensure!(
            size.is_multiple_of(64) && size <= 512 && (1..=8).contains(&samples),
            "bounded aligned dimensions required"
        );
        anyhow::ensure!(mode <= 10, "unknown rendering mode");
        anyhow::ensure!(
            mode < 6 || mesh.indexed.is_some(),
            "hierarchy mode requires index plan"
        );
        let target_size = size * samples;
        self.ensure_target(target_size);
        let target = &self.targets[&target_size];
        let globals = self.globals(case, mode, target_size);
        self.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&globals));
        let prepare = mode >= 9
            && mesh.active.get() > 0
            && (mode == 10 || mesh.cache_key.get() != Some(lighting_key(case)));
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let repeats = if benchmark { 31 } else { 1 };
        // Warmup is in a separate completed submission, excludes lazy pipeline compilation.
        let draw = |encoder: &mut wgpu::CommandEncoder, index: Option<u32>| {
            self.encode_frame(
                encoder,
                target,
                mesh,
                DrawOptions {
                    mode,
                    index,
                    unreduced: false,
                    prepare,
                },
            );
        };

        if benchmark {
            draw(&mut encoder, None);
            let submission = self.queue.submit([encoder.finish()]);
            self.device.poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(60)),
            })?;
            encoder = self.device.create_command_encoder(&Default::default());
        }
        for i in 0..repeats {
            draw(&mut encoder, benchmark.then_some(i as u32));
        }
        // Resolve after the measured submission completes: Metal stage-boundary samples
        // can otherwise still be unavailable when a same-submission blit resolves them.
        let submission = self.queue.submit([encoder.finish()]);
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(60)),
        })?;
        encoder = self.device.create_command_encoder(&Default::default());
        if benchmark && let Some(query) = &self.query {
            encoder.resolve_query_set(query, 0..repeats as u32 * 4, &self.query_resolve, 0);
            encoder.copy_buffer_to_buffer(&self.query_resolve, 0, &self.query_readback, 0, 8192);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target.color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &target.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(target_size * 16),
                    rows_per_image: Some(target_size),
                },
            },
            wgpu::Extent3d {
                width: target_size,
                height: target_size,
                depth_or_array_layers: 1,
            },
        );
        let submission = self.queue.submit([encoder.finish()]);
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(60)),
        })?;
        let mut values = Vec::new();
        let mut preparations = Vec::new();
        let mut draws = Vec::new();
        if benchmark && let Some(period) = self.timestamp_period_ns {
            self.map(&self.query_readback)?;
            let data = self.query_readback.slice(..).get_mapped_range();
            for chunk in data[..repeats * 32].chunks_exact(32) {
                let read =
                    |i: usize| u64::from_le_bytes(chunk[i * 8..i * 8 + 8].try_into().unwrap());
                let draw_begin = read(2);
                let end = read(3);
                let begin = if prepare { read(0) } else { draw_begin };
                anyhow::ensure!(
                    end >= begin && end >= draw_begin,
                    "nonmonotonic GPU timestamps"
                );
                let scale = f64::from(period) / 1e6;
                values.push((end - begin) as f64 * scale);
                draws.push((end - draw_begin) as f64 * scale);
                if prepare {
                    let prepared = read(1);
                    anyhow::ensure!(
                        prepared >= begin && draw_begin >= prepared,
                        "nonmonotonic preparation timestamps"
                    );
                    preparations.push((prepared - begin) as f64 * scale);
                }
            }
            drop(data);
            self.query_readback.unmap();
            values.sort_by(f64::total_cmp);
            preparations.sort_by(f64::total_cmp);
            draws.sort_by(f64::total_cmp);
        }
        self.map(&target.readback)?;
        let data = target.readback.slice(..).get_mapped_range();
        let floats: &[f32] = bytemuck::cast_slice(&data);
        let mut pixels = vec![[0_f32; 4]; (size * size) as usize];
        let weight = 1.0 / (samples * samples) as f32;
        for y in 0..target_size {
            for x in 0..target_size {
                let src = ((y * target_size + x) * 4) as usize;
                let dst = ((y / samples) * size + x / samples) as usize;
                for c in 0..4 {
                    pixels[dst][c] += floats[src + c] * weight;
                }
            }
        }
        drop(data);
        target.readback.unmap();
        anyhow::ensure!(
            pixels.iter().flatten().all(|v| v.is_finite()),
            "nonfinite GPU output"
        );
        if prepare {
            mesh.cache_key.set(Some(lighting_key(case)));
        }
        Ok(Capture {
            prepared: prepare,
            pixels,
            timing: Timing {
                gpu_ms_median: values.get(values.len() / 2).copied(),
                gpu_ms_min: values.first().copied(),
                gpu_ms_max: values.last().copied(),
                gpu_ms_p10: values.get(values.len() / 10).copied(),
                gpu_ms_p90: values.get(values.len() * 9 / 10).copied(),
                repeats,
                preparation_ms_median: preparations.get(preparations.len() / 2).copied(),
                draw_ms_median: draws.get(draws.len() / 2).copied(),
            },
        })
    }
    pub fn benchmark_comparison(
        &mut self,
        mesh: &Uploaded,
        case: Case,
        size: u32,
    ) -> Result<Vec<Timing>> {
        anyhow::ensure!(
            mesh.indexed.is_some() && size.is_multiple_of(64) && size > 0 && size <= 512,
            "invalid comparison target"
        );
        self.ensure_target(size);
        let target = &self.targets[&size];
        let globals = self.globals(case, 8, size);
        self.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&globals));
        let encode = |encoder: &mut wgpu::CommandEncoder,
                      method: usize,
                      slot: Option<u32>,
                      first: bool| {
            let prepare = mesh.active.get() > 0
                && (method == 4
                    || (first && method == 1 && mesh.cache_key.get() != Some(lighting_key(case))));
            self.encode_frame(
                encoder,
                target,
                mesh,
                DrawOptions {
                    mode: [8, 9, 0, 9, 10][method],
                    index: slot,
                    unreduced: method == 3,
                    prepare,
                },
            );
        };
        let mut encoder = self.device.create_command_encoder(&Default::default());
        for round in 0..4 {
            for method in 0..5 {
                encode(&mut encoder, method, None, round == 0);
            }
        }
        let submission = self.queue.submit([encoder.finish()]);
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(60)),
        })?;
        if mesh.active.get() > 0 {
            mesh.cache_key.set(Some(lighting_key(case)));
        }
        encoder = self.device.create_command_encoder(&Default::default());
        for repeat in 0..31 {
            for offset in 0..5 {
                let method = (repeat + offset) % 5;
                encode(
                    &mut encoder,
                    method,
                    Some((repeat * 5 + method) as u32),
                    false,
                );
            }
        }
        let submission = self.queue.submit([encoder.finish()]);
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(60)),
        })?;
        let Some(query) = &self.query else {
            return Ok((0..5)
                .map(|_| Timing {
                    gpu_ms_median: None,
                    gpu_ms_min: None,
                    gpu_ms_max: None,
                    gpu_ms_p10: None,
                    gpu_ms_p90: None,
                    repeats: 31,
                    preparation_ms_median: None,
                    draw_ms_median: None,
                })
                .collect());
        };
        encoder = self.device.create_command_encoder(&Default::default());
        encoder.resolve_query_set(query, 0..620, &self.query_resolve, 0);
        encoder.copy_buffer_to_buffer(&self.query_resolve, 0, &self.query_readback, 0, 8192);
        let submission = self.queue.submit([encoder.finish()]);
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(60)),
        })?;
        self.map(&self.query_readback)?;
        let data = self.query_readback.slice(..).get_mapped_range();
        let scale = f64::from(self.timestamp_period_ns.unwrap()) / 1e6;
        let read = |i: usize| u64::from_le_bytes(data[i * 8..i * 8 + 8].try_into().unwrap());
        let mut results = Vec::new();
        for method in 0..5 {
            let mut total = Vec::with_capacity(31);
            let mut prepare = Vec::with_capacity(31);
            let mut draw = Vec::with_capacity(31);
            for repeat in 0..31 {
                let slot = (repeat * 5 + method) * 4;
                let drawing = read(slot + 2);
                let end = read(slot + 3);
                let compute = method == 4 && mesh.active.get() > 0;
                let begin = if compute { read(slot) } else { drawing };
                anyhow::ensure!(
                    end >= drawing && drawing >= begin,
                    "invalid comparison timestamps"
                );
                total.push((end - begin) as f64 * scale);
                draw.push((end - drawing) as f64 * scale);
                if compute {
                    let done = read(slot + 1);
                    anyhow::ensure!(
                        done >= begin && done <= drawing,
                        "invalid compute timestamps"
                    );
                    prepare.push((done - begin) as f64 * scale);
                }
            }
            total.sort_by(f64::total_cmp);
            prepare.sort_by(f64::total_cmp);
            draw.sort_by(f64::total_cmp);
            results.push(Timing {
                gpu_ms_median: Some(total[15]),
                gpu_ms_min: Some(total[0]),
                gpu_ms_max: Some(total[30]),
                gpu_ms_p10: Some(total[3]),
                gpu_ms_p90: Some(total[27]),
                repeats: 31,
                preparation_ms_median: prepare.get(15).copied(),
                draw_ms_median: Some(draw[15]),
            });
        }
        drop(data);
        self.query_readback.unmap();
        Ok(results)
    }
    fn globals(&self, case: Case, mode: usize, target_size: u32) -> Globals {
        let angle = case.tilt.to_radians();
        let view = Vec3::new(angle.sin(), angle.cos(), 0.0);
        let matrix = Mat4::perspective_rh(FOV_Y, 1.0, NEAR, 100.0) * case.view_matrix();
        let la = case.light_angle.to_radians();
        let light = Vec3::new(la.sin(), la.cos(), 0.2).normalize();
        Globals {
            matrix: matrix.to_cols_array_2d(),
            light: [light.x, light.y, light.z, 0.0],
            view: [view.x, view.y, view.z, 0.0],
            settings: [
                mode as f32,
                if mode == 3 { 128.0 } else { 32.0 },
                case.exponent,
                case.specular,
            ],
            jitter: [
                2.0 * self.jitter[0] / target_size as f32,
                2.0 * self.jitter[1] / target_size as f32,
                0.0,
                0.0,
            ],
        }
    }
    fn encode_frame(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &Target,
        mesh: &Uploaded,
        options: DrawOptions,
    ) {
        let DrawOptions {
            mode,
            index,
            unreduced,
            prepare,
        } = options;
        if prepare {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("summary preparation included in frame timing"),
                timestamp_writes: index.and_then(|i| {
                    self.query
                        .as_ref()
                        .map(|q| wgpu::ComputePassTimestampWrites {
                            query_set: q,
                            beginning_of_pass_write_index: Some(i * 4),
                            end_of_pass_write_index: Some(i * 4 + 1),
                        })
                }),
            });
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &mesh.compute_group, &[]);
            pass.dispatch_workgroups((mesh.transition_capacity as u32).div_ceil(64), 1, 1);
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("appearance draw"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: index.and_then(|i| {
                self.query
                    .as_ref()
                    .map(|q| wgpu::RenderPassTimestampWrites {
                        query_set: q,
                        beginning_of_pass_write_index: Some(i * 4 + 2),
                        end_of_pass_write_index: Some(i * 4 + 3),
                    })
            }),
            occlusion_query_set: None,
        });
        pass.set_pipeline(if mode == 0 || (mode >= 9 && mesh.active.get() == 0) {
            &self.full_pipeline
        } else if mode >= 9 {
            &self.cached_pipeline
        } else {
            &self.pipeline
        });
        pass.set_bind_group(0, &mesh.groups[usize::from(mode == 3)], &[]);
        if unreduced {
            let indices = mesh.indexed.as_ref().unwrap().unreduced.as_ref().unwrap();
            pass.set_vertex_buffer(0, mesh.full.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.full_count, 0, 0..1);
        } else if mode >= 6 {
            let plan = mesh.indexed.as_ref().expect("validated index plan");
            for (vertices, indices, count) in [
                (&mesh.full, &plan.full, plan.full_count),
                (&mesh.proxy, &plan.proxy, plan.proxy_count),
            ] {
                if count > 0 {
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..count, 0, 0..1);
                }
            }
        } else if mode >= 4 {
            for detail in [true, false] {
                pass.set_vertex_buffer(
                    0,
                    if detail {
                        mesh.full.slice(..)
                    } else {
                        mesh.proxy.slice(..)
                    },
                );
                let count = if detail {
                    mesh.full_count
                } else {
                    mesh.proxy_count
                } / TILE_COUNT as u32;
                for tiles in selection::ranges(&self.detailed, detail) {
                    pass.draw(tiles.start * count..tiles.end * count, 0..1);
                }
            }
        } else {
            pass.set_vertex_buffer(
                0,
                if mode == 0 {
                    mesh.full.slice(..)
                } else {
                    mesh.proxy.slice(..)
                },
            );
            pass.draw(
                0..if mode == 0 {
                    mesh.full_count
                } else {
                    mesh.proxy_count
                },
                0..1,
            );
        }
    }
    fn map(&self, buffer: &wgpu::Buffer) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })?;
        rx.recv_timeout(Duration::from_secs(30))
            .context("readback callback")??;
        Ok(())
    }
}
