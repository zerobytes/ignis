use super::{Pack, packet};
use crate::{
    render::RenderFrame,
    render_visibility::{CAMERA, Visibility},
};
use anyhow::{Result, ensure};
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use std::{collections::HashMap, sync::Arc};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    world: [[f32; 4]; 4],
    control: [u32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Color {
    color: [f32; 3],
    padding: u32,
}
#[derive(Default, serde::Serialize)]
pub struct Stats {
    pub enabled: bool,
    pub bound_objects: usize,
    pub selected_objects: usize,
    pub candidate_triangles_replaced: usize,
    pub quad_triangles: usize,
    pub max_selected_diameter_px: f32,
    pub hierarchy_bytes: usize,
    pub seed_depth_active: bool,
    pub near_guard: bool,
    pub near_screen_area_px: f32,
    pub source_objects_by_size: usize,
    pub selection_cache_bytes: usize,
    pub triangle_bytes: usize,
    pub instance_bytes: usize,
    pub allocated_instance_bytes: u64,
    pub palette_bytes: usize,
}
pub struct Gpu {
    pub enabled: bool,
    pub density: f32,
    pub raster_grid: u32,
    pub native_depth_order: bool,
    pub max_pixels: f32,
    pub cost_guard: bool,
    pub seed_culling: bool,
    pub stats: Stats,
    pack: Arc<Pack>,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    triangles: wgpu::Buffer,
    nodes: wgpu::Buffer,
    parameters: wgpu::Buffer,
    seed: wgpu::TextureView,
    empty_seed: wgpu::TextureView,
    seed_settings: wgpu::Buffer,
    instances: Vec<Instance>,
    colors: Vec<Color>,
    instance_buffer: wgpu::Buffer,
    color_buffer: wgpu::Buffer,
    bind: wgpu::BindGroup,
    generation: Option<u64>,
    members: Vec<usize>,
    bound_parts: usize,
    choices: Vec<bool>,
    selected_groups: Vec<usize>,
}
fn buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
#[allow(clippy::too_many_arguments)]
fn bindings(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    triangles: &wgpu::Buffer,
    nodes: &wgpu::Buffer,
    parameters: &wgpu::Buffer,
    instances: &wgpu::Buffer,
    colors: &wgpu::Buffer,
    seed: &wgpu::TextureView,
    seed_settings: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("collective ICB1 data"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: triangles.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: nodes.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: parameters.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: instances.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: colors.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(seed),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: seed_settings.as_entire_binding(),
            },
        ],
    })
}
impl Gpu {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        pack: Arc<Pack>,
    ) -> Result<Self> {
        pack.validate_layout()?;
        ensure!(
            pack.nodes.len() as u64 * 32
                <= u64::from(device.limits().max_storage_buffer_binding_size)
                && pack.triangles.len() as u64 * 80
                    <= u64::from(device.limits().max_storage_buffer_binding_size),
            "ICB1 exceeds device binding; use source"
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ICB1 layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ICB1 + native atmosphere"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}\n{}",
                    include_str!("../scene.wgsl"),
                    include_str!("raster.wgsl"),
                    include_str!("collective.wgsl")
                )
                .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ICB1 pipeline"),
            bind_group_layouts: &[scene_layout, &layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ICB1 quad"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_collective"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
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
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_collective"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });
        let triangles = buffer(
            device,
            "ICB1 exact source corners",
            (pack.triangles.len() * 80) as u64,
        );
        let nodes = buffer(
            device,
            "ICB1 containing hierarchy",
            (pack.nodes.len() * 32) as u64,
        );
        queue.write_buffer(&triangles, 0, bytemuck::cast_slice(&pack.triangles));
        queue.write_buffer(&nodes, 0, bytemuck::cast_slice(&pack.nodes));
        let parameters = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ICB1 bounds"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let seed_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("empty collective depth seed"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let empty_seed = seed_texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("initialize empty depth seed"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &empty_seed,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        queue.submit([encoder.finish()]);
        let seed_settings = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("collective depth guards"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_buffer = buffer(device, "ICB1 placements", 80);
        let color_buffer = buffer(device, "ICB1 colors", 16);
        let bind = bindings(
            device,
            &layout,
            &triangles,
            &nodes,
            &parameters,
            &instance_buffer,
            &color_buffer,
            &empty_seed,
            &seed_settings,
        );
        Ok(Self {
            enabled: false,
            density: 16.,
            raster_grid: 0,
            native_depth_order: false,
            max_pixels: 8.,
            cost_guard: true,
            seed_culling: true,
            stats: Default::default(),
            pack,
            pipeline,
            layout,
            triangles,
            nodes,
            parameters,
            seed: empty_seed.clone(),
            empty_seed,
            seed_settings,
            instances: vec![],
            colors: vec![],
            instance_buffer,
            color_buffer,
            bind,
            generation: None,
            members: vec![],
            bound_parts: 0,
            choices: vec![],
            selected_groups: vec![],
        })
    }
    fn resolve(&mut self, frame: &RenderFrame) {
        self.members.clear();
        let doc = &frame.meshes.source;
        let valid = self.pack.parts.iter().all(|p| {
            doc.assets.iter().any(|a| {
                a.id == p.name
                    && Arc::ptr_eq(&a.geometry, &p.geometry)
                    && a.shading == p.shading
                    && packet::eligible(a)
            })
        });
        if !valid {
            return;
        }
        let lookup: HashMap<_, _> = frame
            .meshes
            .draws
            .iter()
            .enumerate()
            .map(|(n, d)| (d.instance.as_str(), n))
            .collect();
        for object in doc.objects.iter() {
            if object.members.len() != self.pack.parts.len() {
                continue;
            }
            let mut indexes: Vec<usize> = vec![];
            for (m, p) in object.members.iter().zip(&self.pack.parts) {
                let Some(&n) = lookup.get(m.as_str()) else {
                    break;
                };
                let d = &frame.meshes.draws[n];
                if !d.eligible
                    || !d.shared
                    || doc.assets[d.asset].id != p.name
                    || indexes
                        .first()
                        .is_some_and(|&i| frame.meshes.draws[i].world != d.world)
                {
                    break;
                }
                indexes.push(n);
            }
            if indexes.len() == self.pack.parts.len() {
                self.members.extend(indexes);
            }
        }
        self.bound_parts = self.pack.parts.len();
        self.choices
            .resize(self.members.len() / self.bound_parts, false);
        self.choices.fill(false);
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &RenderFrame,
        visibility: &mut Visibility,
        width: u32,
        height: u32,
    ) {
        self.instances.clear();
        self.colors.clear();
        self.selected_groups.clear();
        self.stats = Stats {
            enabled: self.enabled,
            hierarchy_bytes: self.pack.nodes.len() * 32,
            triangle_bytes: self.pack.triangles.len() * 80,
            ..Default::default()
        };
        if self.pack.source_triangles <= 2 {
            return;
        }
        if !self.enabled
            || frame.light.shadow.is_some()
            || frame.light.environment.is_some_and(|e| e.intensity > 0.)
            || frame
                .light
                .atmosphere
                .and_then(|a| a.edge_softness)
                .is_some_and(|s| s.active())
        {
            return;
        }
        if self.generation != Some(frame.meshes.static_generation) {
            self.resolve(frame);
            self.generation = Some(frame.meshes.static_generation);
        }
        if self.bound_parts == 0 {
            return;
        }
        self.stats.bound_objects = self.members.len() / self.bound_parts;
        let eye = Vec3::from(frame.camera.position);
        let forward = (Vec3::from(frame.camera.target) - eye).normalize();
        let focal = height as f32 / (2. * (frame.camera.fov_degrees.to_radians() * 0.5).tan());
        let max_buffer = u64::from(device.limits().max_storage_buffer_binding_size)
            .min(device.limits().max_buffer_size);
        let capacity = (max_buffer / 80).min(max_buffer / (16 * self.bound_parts as u64)) as usize;
        for (group, members) in self.members.chunks_exact(self.bound_parts).enumerate() {
            if self.instances.len() >= capacity {
                break;
            }
            if !members.iter().any(|&n| visibility.masks[n] & CAMERA != 0)
                || members.iter().any(|&n| {
                    frame.selection_mesh.as_ref() == Some(&frame.meshes.draws[n].instance)
                })
            {
                continue;
            }
            let root = &frame.meshes.draws[members[0]];
            let scale = root.world.x_axis.truncate().length();
            let center = root.world.transform_point3(self.pack.center);
            let radius = self.pack.radius * scale;
            let nearest = (center - eye).dot(forward) - radius;
            if self.cost_guard && nearest <= 0.05 {
                self.stats.near_guard = true;
            }
            if nearest <= 0.05 || (center.y - radius < 0.021 && center.y + radius > -0.021) {
                self.choices[group] = false;
                if self.stats.near_guard {
                    break;
                }
                continue;
            }
            let diameter = 2. * focal * radius / nearest;
            if self.max_pixels > 0. && diameter > self.max_pixels {
                self.stats.near_screen_area_px += diameter * diameter;
            }
            if self.cost_guard
                && self.stats.near_screen_area_px > width as f32 * height as f32 * 0.5
            {
                self.stats.near_guard = true;
                break;
            }
            // Enter below the limit, leave above it. This changes only path choice;
            // it is neither a silhouette certificate nor a geometric error budget.
            let limit = self.max_pixels * if self.choices[group] { 1.125 } else { 1. };
            if limit > 0. && diameter > limit {
                self.choices[group] = false;
                self.stats.source_objects_by_size += 1;
                continue;
            }
            let density = self.pack.source_triangles as f32 / (diameter * diameter).max(1.);
            if !density.is_finite() || density < self.density {
                self.choices[group] = false;
                continue;
            }
            self.choices[group] = true;
            self.selected_groups.push(group);
            let base = self.colors.len() as u32;
            self.instances.push(Instance {
                world: root.world.to_cols_array_2d(),
                control: [
                    base,
                    self.raster_grid,
                    u32::from(self.native_depth_order),
                    0,
                ],
            });
            for &n in members {
                let draw = &frame.meshes.draws[n];
                self.colors.push(Color {
                    color: draw.color,
                    padding: 0,
                });
                if visibility.masks[n] & CAMERA != 0 {
                    self.stats.candidate_triangles_replaced += draw.range.len() / 3;
                }
            }
            self.stats.max_selected_diameter_px = self.stats.max_selected_diameter_px.max(diameter);
        }
        self.stats.selection_cache_bytes = self.choices.capacity()
            + self.selected_groups.capacity() * std::mem::size_of::<usize>();
        if self.stats.near_guard {
            self.instances.clear();
            self.colors.clear();
            self.selected_groups.clear();
            self.choices.fill(false);
            self.stats.candidate_triangles_replaced = 0;
            self.stats.max_selected_diameter_px = 0.;
            return;
        }
        // Publish masks only after the whole-view cost guard accepts the plan.
        for &group in &self.selected_groups {
            for &n in &self.members[group * self.bound_parts..(group + 1) * self.bound_parts] {
                visibility.masks[n] &= !CAMERA;
            }
        }
        self.stats.selected_objects = self.instances.len();
        self.stats.quad_triangles = self.instances.len() * 2;
        self.stats.instance_bytes = self.instances.len() * 80;
        self.stats.palette_bytes = self.colors.len() * 16;
        if self.instances.is_empty() {
            return;
        }
        let mut changed = false;
        if self.instance_buffer.size() < self.stats.instance_bytes as u64 {
            self.instance_buffer =
                buffer(device, "ICB1 placements", self.stats.instance_bytes as u64);
            changed = true;
        }
        if self.color_buffer.size() < self.stats.palette_bytes as u64 {
            self.color_buffer = buffer(device, "ICB1 colors", self.stats.palette_bytes as u64);
            changed = true;
        }
        if changed {
            self.bind = bindings(
                device,
                &self.layout,
                &self.triangles,
                &self.nodes,
                &self.parameters,
                &self.instance_buffer,
                &self.color_buffer,
                &self.seed,
                &self.seed_settings,
            );
        }
        let p = [
            self.pack.center.x,
            self.pack.center.y,
            self.pack.center.z,
            self.pack.radius,
            2. / width as f32,
            -2. / height as f32,
            -1.,
            1.,
        ];
        queue.write_buffer(&self.parameters, 0, bytemuck::cast_slice(&p));
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
        queue.write_buffer(&self.color_buffer, 0, bytemuck::cast_slice(&self.colors));
        self.stats.allocated_instance_bytes =
            self.instance_buffer.size() + self.color_buffer.size();
    }
    pub fn set_seed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        depth: Option<&wgpu::TextureView>,
    ) {
        let active = self.seed_culling && !self.instances.is_empty() && depth.is_some();
        let view = if active {
            depth.unwrap()
        } else {
            &self.empty_seed
        };
        if self.seed != *view {
            self.seed = view.clone();
            self.bind = bindings(
                device,
                &self.layout,
                &self.triangles,
                &self.nodes,
                &self.parameters,
                &self.instance_buffer,
                &self.color_buffer,
                &self.seed,
                &self.seed_settings,
            );
        }
        queue.write_buffer(
            &self.seed_settings,
            0,
            bytemuck::cast_slice(&[f32::from(active), 0.05001, 1.00001, 0.00000048]),
        );
        self.stats.seed_depth_active = active;
    }
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, scene: &wgpu::BindGroup) {
        if self.instances.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene, &[]);
        pass.set_bind_group(1, &self.bind, &[]);
        pass.draw(0..6, 0..self.instances.len() as u32);
    }
}
