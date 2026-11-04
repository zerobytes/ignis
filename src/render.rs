use crate::model::{Camera, Light};
use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

pub mod object_impostor;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    pub rest_position: [f32; 3],
    pub material: u32,
    pub material_projection_weights: [f32; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_projection: [[f32; 4]; 4],
    shadow_projection: [[f32; 4]; 4],
    light: [f32; 4],
    shadow_settings: [f32; 4],
    view_position: [f32; 4],
    environment: [f32; 4],
    atmosphere: [f32; 4],
    atmosphere_sun: [f32; 4],
    atmosphere_right: [f32; 4],
    atmosphere_up: [f32; 4],
    atmosphere_forward: [f32; 4],
    atmosphere_sky: [f32; 4],
    atmosphere_base: [f32; 4],
    atmosphere_detail: [f32; 4],
    planet_optics: [f32; 4],
    planet_radial: [f32; 4],
    planet_space: [f32; 4],
    planet_eye_high: [f32; 4],
    planet_eye_low: [f32; 4],
    planet_axis0: [f32; 4],
    planet_axis1: [f32; 4],
    planet_axis2: [f32; 4],
    // xyz: linear color * direct intensity; w: exact legacy-white path for glass.
    direct_light: [f32; 4],
}

#[cfg(test)]
#[test]
fn globals_match_wgsl_camera_layout() {
    assert_eq!(std::mem::size_of::<Globals>(), 464);
    assert_eq!(std::mem::offset_of!(Globals, direct_light), 448);
    assert_eq!(std::mem::offset_of!(Globals, planet_optics), 320);
    assert_eq!(std::mem::offset_of!(Globals, planet_radial), 336);
    assert_eq!(std::mem::offset_of!(Globals, planet_space), 352);
    assert_eq!(std::mem::offset_of!(Globals, planet_eye_high), 368);
    assert_eq!(std::mem::offset_of!(Globals, planet_eye_low), 384);
    assert_eq!(std::mem::offset_of!(Globals, planet_axis0), 400);
    assert_eq!(std::mem::offset_of!(Globals, planet_axis2), 432);
    assert_eq!(std::mem::offset_of!(Globals, view_position), 160);
    assert_eq!(std::mem::offset_of!(Globals, environment), 176);
    assert_eq!(std::mem::offset_of!(Globals, atmosphere), 192);
    assert_eq!(std::mem::offset_of!(Globals, atmosphere_sun), 208);
    assert_eq!(std::mem::offset_of!(Globals, atmosphere_right), 224);
}

// Small meshes keep small allocations; large streams avoid power-of-two cliffs.
fn mesh_buffer_capacity(count: usize) -> usize {
    if count <= 4096 {
        count.max(1).next_power_of_two()
    } else {
        count.div_ceil(4096) * 4096
    }
}

/// Explicit light volume, independent of the viewing camera and gameplay.
pub fn shadow_view_projection(light: &Light) -> Option<Mat4> {
    let shadow = light.shadow.as_ref()?;
    let center = Vec3::from(shadow.center);
    let direction = Vec3::from(light.direction).normalize();
    let up = if direction.dot(Vec3::Y).abs() > 0.95 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    let extent = shadow.extent;
    Some(
        Mat4::orthographic_rh(-extent, extent, -extent, extent, 0.01, extent * 4.)
            * Mat4::look_at_rh(center - direction * extent * 2., center, up),
    )
}

fn shadow_map(device: &wgpu::Device, size: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("directional shadow depth"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

fn scene_bindings(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform: &wgpu::Buffer,
    map: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    materials: (&wgpu::Buffer, &wgpu::Buffer),
    environment: &crate::environment_gpu::Gpu,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scene camera light shadow"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(map),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: materials.0.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&environment.specular_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&environment.diffuse_view),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(&environment.lut_view),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Sampler(&environment.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: materials.1.as_entire_binding(),
            },
        ],
    })
}

fn depth_bindings(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform: &wgpu::Buffer,
    materials: &wgpu::Buffer,
    procedural: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("shadow uniforms and surface coverage"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: materials.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: procedural.as_entire_binding(),
            },
        ],
    })
}

/// Compact, reusable snapshot. No authoring IDs, scripts or NPC configuration.
pub struct RenderFrame {
    /// Render-only projection. None preserves the legacy main/depth contract.
    pub clip: Option<Clip>,
    pub planet_air: Option<PlanetAir>,
    /// A transported frame has no stationary diagnostic plane in world XZ.
    pub suppress_diagnostic_grid: bool,
    pub meshes: crate::meshes::Frame,
    pub particles: crate::particles::Frame,
    pub distant_meshes: std::sync::Arc<Vec<crate::distant::Pack>>,
    pub mesh_lods: std::sync::Arc<crate::mesh_lods::Document>,
    pub mesh_lod_settings: crate::mesh_lods::Settings,
    pub camera: Camera,
    pub light: Light,
    pub boxes: Vec<RenderBox>,
    pub vision_guides: Vec<VisionGuide>,
    pub hearing_guides: Vec<RingGuide>,
    pub noise_guides: Vec<RingGuide>,
    pub route_guides: Vec<LineGuide>,
    pub truncated_navigation_paths: u8,
    pub selection: Option<RenderBox>,
    pub selection_mesh: Option<String>,
}
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Clip {
    pub near: f32,
    pub far: f32,
    pub reverse_z: bool,
}
impl Clip {
    pub(crate) fn validate(self) -> crate::model::Result<()> {
        if !self.near.is_finite()
            || !self.far.is_finite()
            || self.near <= 0.
            || self.far <= self.near
        {
            return Err(crate::model::invalid(
                "Render clipping requires finite0<near<far",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub struct PlanetAir {
    pub radius: f64,
    pub altitude: f64,
    pub radial: [f32; 3],
    pub height: f64,
    pub optical_depth: f64,
    pub space_color: [f32; 3],
    /// Consecutive generated terrain materials, excluding the avatar proxy.
    pub terrain_materials: u32,
    pub(crate) eye: crate::meshes::SplitPosition,
    pub(crate) axes: [[f32; 3]; 3],
}
#[derive(Clone, Copy, PartialEq)]
pub struct RenderBox {
    pub geometry: crate::geometry::Geometry,
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 3],
}
/// Visualization only: current orientation and last sensor result, never a perception input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisionGuide {
    pub position: [f32; 3],
    pub top_y: f32,
    pub ground_y: f32,
    pub facing_degrees: f32,
    pub spatial_eye: Option<[f32; 3]>,
    pub pitch_degrees: f32,
    pub range: f32,
    pub fov_degrees: f32,
    pub directional: bool,
    pub color: [f32; 3],
}
/// Nominal horizontal range; marker distinguishes an emitter from a listener.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RingGuide {
    pub center: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    pub emitter: bool,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineGuide {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 3],
    pub dashed: bool,
}
#[derive(Default)]
struct GuideCache {
    guides: Vec<VisionGuide>,
    hearing: Vec<RingGuide>,
    noises: Vec<RingGuide>,
    routes: Vec<LineGuide>,
    selection: Option<RenderBox>,
    vertices: Vec<Vertex>,
}
impl GuideCache {
    fn update(
        &mut self,
        guides: &[VisionGuide],
        hearing: &[RingGuide],
        noises: &[RingGuide],
        routes: &[LineGuide],
        selection: Option<RenderBox>,
    ) -> bool {
        if self.guides == guides
            && self.hearing == hearing
            && self.noises == noises
            && self.routes == routes
            && self.selection == selection
        {
            return false;
        }
        self.guides.clear();
        self.guides.extend_from_slice(guides);
        self.hearing.clear();
        self.hearing.extend_from_slice(hearing);
        self.noises.clear();
        self.noises.extend_from_slice(noises);
        self.routes.clear();
        self.routes.extend_from_slice(routes);
        self.selection = selection;
        self.vertices.clear();
        if let Some(selected) = selection {
            selection_vertices(selected, &mut self.vertices);
        }
        for line in routes {
            line_vertices(line, &mut self.vertices);
        }
        for guide in guides {
            guide_vertices(guide, &mut self.vertices);
        }
        for ring in hearing.iter().chain(noises) {
            ring_vertices(ring, &mut self.vertices);
        }
        true
    }
}

#[derive(Default)]
struct GeometryCache {
    boxes: Vec<RenderBox>,
    vertices: Vec<Vertex>,
}
impl GeometryCache {
    /// Returns the changed vertex span; removal only changes the draw count.
    fn update(&mut self, boxes: &[RenderBox]) -> Option<std::ops::Range<usize>> {
        let old_len = self.boxes.len();
        self.vertices.resize(boxes.len() * 36, Vertex::zeroed());
        let mut dirty: Option<std::ops::Range<usize>> = None;
        for (i, item) in boxes.iter().enumerate() {
            if i >= old_len || self.boxes[i] != *item {
                box_vertices(item, &mut self.vertices[i * 36..(i + 1) * 36]);
                if i < old_len {
                    self.boxes[i] = *item;
                } else {
                    self.boxes.push(*item);
                }
                match &mut dirty {
                    Some(span) => span.end = (i + 1) * 36,
                    None => dirty = Some(i * 36..(i + 1) * 36),
                }
            }
        }
        self.boxes.truncate(boxes.len());
        dirty
    }
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    transparent_pipeline: wgpu::RenderPipeline,
    transparency: crate::render_transparency::Transparency,
    particles: crate::render_particles::Gpu,
    surface_field: Option<crate::surface_field::Gpu>,
    mesh_proxy: Option<crate::mesh_proxy::State>,
    object_impostor: Option<object_impostor::State>,
    depth_layout: wgpu::BindGroupLayout,
    shadow_pipeline: wgpu::RenderPipeline,
    shadow_bind_group: wgpu::BindGroup,
    shadow_layout: wgpu::BindGroupLayout,
    shadow_map: (u32, wgpu::TextureView),
    shadow_sampler: wgpu::Sampler,
    guide_pipeline: wgpu::RenderPipeline,
    atmosphere_pipeline: wgpu::RenderPipeline,
    atmosphere_reverse: wgpu::RenderPipeline,
    distant_pipeline: wgpu::RenderPipeline,
    distant: crate::distant::selection::Selection,
    instanced: crate::render_instances::Gpu,
    collective: Option<crate::collective_lod::Gpu>,
    visibility: crate::render_visibility::Visibility,
    visibility_enabled: bool,
    occlusion_enabled: bool,
    distant_buffer: wgpu::Buffer,
    distant_capacity: usize,
    distant_vertices: Vec<crate::distant::selection::CheapVertex>,
    distant_index_buffer: wgpu::Buffer,
    distant_index_capacity: usize,
    distant_indices: Vec<u32>,
    guides: GuideCache,
    outline: crate::render_outline::Outline,
    atmosphere_edges: crate::atmosphere_edges::Edges,
    guide_buffer: wgpu::Buffer,
    guide_capacity: usize,
    uniform: wgpu::Buffer,
    material_buffer: wgpu::Buffer,
    procedural_buffer: wgpu::Buffer,
    procedural_uploads: u64,
    material_cache: Option<Vec<crate::materials::GpuMaterial>>,
    material_source: Vec<crate::materials::Material>,
    environment: crate::environment_gpu::Gpu,
    bind_group: wgpu::BindGroup,
    geometry: GeometryCache,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    mesh_buffer: wgpu::Buffer,
    mesh_capacity: usize,
    mesh_vertices: Vec<Vertex>,
    depth: Option<(u32, u32, wgpu::TextureView, bool)>,
}

impl Renderer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: surface,
                force_fallback_adapter: false,
            })
            .await
            .context("No compatible graphics adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Ignis device"),
                required_limits: wgpu::Limits {
                    max_buffer_size: adapter.limits().max_buffer_size,
                    max_texture_array_layers: adapter.limits().max_texture_array_layers,
                    max_storage_buffer_binding_size: adapter
                        .limits()
                        .max_storage_buffer_binding_size,
                    ..Default::default()
                },
                ..Default::default()
            })
            .await?;
        let format = surface
            .map(|s| {
                let caps = s.get_capabilities(&adapter);
                caps.formats
                    .iter()
                    .copied()
                    .find(|f| f.is_srgb())
                    .unwrap_or(caps.formats[0])
            })
            .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ignis lit boxes"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("scene.wgsl"),
                    "\n",
                    include_str!("particles.wgsl")
                )
                .into(),
            ),
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera and light"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let map = shadow_map(&device, 1);
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow comparison"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let material_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("surface material storage"),
            size: std::mem::size_of::<crate::materials::GpuMaterial>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let procedural_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("procedural programs and fixed samples"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let environment = crate::environment_gpu::Gpu::new(&device);
        let bind_group = scene_bindings(
            &device,
            &layout,
            &uniform,
            &map,
            &shadow_sampler,
            (&material_buffer, &procedural_buffer),
            &environment,
        );
        // Separate depth-only bindings avoid sampling the map while rendering into it.
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow uniforms and surface coverage"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shadow_bind_group = depth_bindings(
            &device,
            &depth_layout,
            &uniform,
            &material_buffer,
            &procedural_buffer,
        );
        let depth_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shadow depth layout"),
                bind_group_layouts: &[&depth_layout],
                push_constant_ranges: &[],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let instanced = crate::render_instances::Gpu::new(
            &device,
            &shader,
            &pipeline_layout,
            &depth_pipeline_layout,
            format,
        );
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Float32x3,4=>Uint32,5=>Float32x3];
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("directional shadow depth"),
            layout: Some(&depth_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_shadow"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_shadow"),
                compilation_options: Default::default(),
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: 1,
                    slope_scale: 1.,
                    clamp: 0.,
                },
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Ignis scene pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
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
                    format,
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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let transparent_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("native transparent surfaces"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_transparent"),
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
        let distant_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("compiled distant fill, no material evaluation"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_distant"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 24,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_distant"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let distant_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("selected distant vertices"),
            size: 24,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let distant_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("selected distant indices"),
            size: 4,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let atmosphere_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("atmosphere background"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_atmosphere"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_atmosphere"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
        let atmosphere_reverse = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("atmosphere background"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_atmosphere"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_atmosphere"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        // Opaque diagnostic lines drawn on top: nominal limits, not an occlusion mask.
        let guide_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NPC vision guides"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_guide"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let guide_capacity = 6;
        let guide_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vision guide vertices"),
            size: (guide_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_capacity = 36;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene vertices"),
            size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mesh_capacity = 3;
        let mesh_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("native mesh vertices"),
            size: (mesh_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let particles = crate::render_particles::Gpu::new(&device, &pipeline_layout, &shader);
        let outline = crate::render_outline::Outline::new(&device, format);
        let mut renderer = Self {
            environment,
            material_buffer,
            procedural_buffer,
            procedural_uploads: 0,
            material_cache: None,
            material_source: Vec::new(),
            transparent_pipeline,
            transparency: Default::default(),
            particles,
            surface_field: None,
            mesh_proxy: None,
            object_impostor: None,
            depth_layout,
            shadow_pipeline,
            shadow_bind_group,
            shadow_layout: layout,
            shadow_map: (1, map),
            shadow_sampler,
            mesh_buffer,
            mesh_capacity,
            mesh_vertices: Vec::new(),
            device,
            queue,
            adapter,
            format,
            pipeline,
            guide_pipeline,
            atmosphere_pipeline,
            atmosphere_reverse,
            distant_pipeline,
            distant_buffer,
            distant_capacity: 1,
            distant_vertices: Vec::new(),
            distant_index_buffer,
            distant_index_capacity: 1,
            distant_indices: Vec::new(),
            distant: Default::default(),
            instanced,
            collective: None,
            visibility: Default::default(),
            visibility_enabled: true,
            occlusion_enabled: true,
            guides: GuideCache::default(),
            outline,
            atmosphere_edges: Default::default(),
            guide_buffer,
            guide_capacity,
            uniform,
            bind_group,
            geometry: GeometryCache::default(),
            vertex_buffer,
            vertex_capacity,
            depth: None,
        };
        // Explicit laboratory sidecar applies equally to viewport and MCP captures.
        if let Some(path) = std::env::var_os("IGNIS_HORIZON_FIELD") {
            let field = crate::surface_field::Pack::read(std::io::BufReader::new(
                std::fs::File::open(path)?,
            ))?;
            renderer.set_surface_field(Some(std::sync::Arc::new(field)))?;
        }
        if let Ok(mode) = std::env::var("IGNIS_HORIZON_COORDS") {
            renderer.set_reconstructed_coordinates(true, mode == "indexed");
        }
        if let Some(path) = std::env::var_os("IGNIS_HORIZON_OBJECTS") {
            anyhow::ensure!(
                std::fs::metadata(&path)?.len() <= 1025 * 1024 * 1024,
                "Object atlas packet too large"
            );
            let packet = object_impostor::Pack::decode(&std::fs::read(path)?)?;
            let pixels = std::env::var("IGNIS_HORIZON_OBJECT_PIXELS")
                .ok()
                .map(|v| v.parse::<f32>())
                .transpose()?
                .unwrap_or(256.);
            renderer.set_object_impostors(Some(std::sync::Arc::new(packet)), pixels)?;
            renderer.set_object_impostor_shadows(
                std::env::var_os("IGNIS_HORIZON_OBJECT_SHADOWS").is_some(),
            );
        }
        if let Some(path) = std::env::var_os("IGNIS_HORIZON_PROXIES") {
            anyhow::ensure!(
                std::fs::metadata(&path)?.len() <= 256 * 1024 * 1024,
                "Diagnostic proxy packet too large"
            );
            renderer.set_mesh_proxy(Some(std::sync::Arc::new(crate::mesh_proxy::Pack::decode(
                &std::fs::read(path)?,
            )?)));
        }
        Ok(renderer)
    }
    /// Current sprite allocation and upload counters.
    pub fn particle_diagnostics(&self) -> serde_json::Value {
        self.particles.inspect()
    }
    /// Diagnostic field installation; does not mutate Scene, physics, or replay.
    pub fn set_surface_field(
        &mut self,
        field: Option<std::sync::Arc<crate::surface_field::Pack>>,
    ) -> Result<()> {
        self.surface_field = field
            .map(|p| {
                crate::surface_field::Gpu::new(
                    &self.device,
                    &self.queue,
                    &self.shadow_layout,
                    &self.depth_layout,
                    self.format,
                    p,
                )
            })
            .transpose()?;
        Ok(())
    }
    pub fn surface_field_info(&self) -> serde_json::Value {
        serde_json::to_value(self.surface_field.as_ref().map(|g| &g.stats)).unwrap()
    }
    /// Experimental estimated-error proxies; not an IGL/image certificate.
    pub fn set_mesh_proxy(&mut self, packet: Option<std::sync::Arc<crate::mesh_proxy::Pack>>) {
        self.mesh_proxy = packet.map(crate::mesh_proxy::State::new);
    }
    /// Experimental reconstruction of redundant static procedural coordinates.
    pub fn set_reconstructed_coordinates(&mut self, enabled: bool, indexed: bool) {
        self.instanced
            .set_reconstructed_coordinates(enabled, indexed);
    }
    pub fn mesh_proxy_info(&self) -> serde_json::Value {
        serde_json::to_value(self.mesh_proxy.as_ref().map(|p| &p.stats)).unwrap()
    }
    /// Diagnostic source-linked coverage/depth atlas. Not an image-error certificate.
    pub fn set_object_impostors(
        &mut self,
        packet: Option<std::sync::Arc<object_impostor::Pack>>,
        maximum_pixels: f32,
    ) -> Result<()> {
        anyhow::ensure!(
            maximum_pixels.is_finite() && (16. ..=1024.).contains(&maximum_pixels),
            "Invalid object atlas pixel threshold"
        );
        if let (Some(current), Some(pack)) = (&mut self.object_impostor, &packet)
            && current.same_pack(pack)
        {
            current.maximum_pixels = maximum_pixels;
            return Ok(());
        }
        let next = packet
            .map(|p| object_impostor::State::new(self, p, maximum_pixels))
            .transpose()?;
        self.object_impostor = next;
        Ok(())
    }
    pub fn set_object_impostor_shadows(&mut self, enabled: bool) {
        if let Some(s) = &mut self.object_impostor {
            s.shadows_enabled = enabled;
        }
    }
    pub fn object_impostor_info(&self) -> serde_json::Value {
        serde_json::to_value(self.object_impostor.as_ref().map(|p| &p.stats)).unwrap()
    }
    /// Experimental sampled-coverage packet. Installing it does not enable it.
    /// No Scene/physics/replay state changes; a failed installation preserves the old packet.
    pub fn set_collective_packet(
        &mut self,
        packet: Option<std::sync::Arc<crate::collective_lod::Pack>>,
    ) -> Result<()> {
        let next = packet
            .map(|p| {
                crate::collective_lod::Gpu::new(
                    &self.device,
                    &self.queue,
                    &self.shadow_layout,
                    self.format,
                    p,
                )
            })
            .transpose()?;
        self.collective = next;
        Ok(())
    }
    pub fn set_collective_enabled(&mut self, enabled: bool) {
        if let Some(gpu) = &mut self.collective {
            gpu.enabled = enabled;
        }
    }
    pub fn set_collective_seed_culling(&mut self, enabled: bool) {
        if let Some(gpu) = &mut self.collective {
            gpu.seed_culling = enabled;
        }
    }
    /// Diagnostic device-specific raster grid. Zero retains exact ray intersections.
    pub fn set_collective_raster_grid(&mut self, grid: u32) -> Result<()> {
        anyhow::ensure!(
            grid == 0 || (grid.is_power_of_two() && (16..=4096).contains(&grid)),
            "Invalid collective raster grid"
        );
        self.collective
            .as_mut()
            .context("Install a collective packet first")?
            .raster_grid = grid;
        Ok(())
    }
    /// Diagnostic comparison of conventional-depth ties versus nearest geometry.
    pub fn set_collective_depth_order(&mut self, native: bool) -> Result<()> {
        self.collective
            .as_mut()
            .context("Install a collective packet first")?
            .native_depth_order = native;
        Ok(())
    }
    /// Diagnostic conservative selection. Zero disables the projected size cap.
    pub fn set_collective_selection(&mut self, max_pixels: f32, cost_guard: bool) -> Result<()> {
        anyhow::ensure!(
            max_pixels.is_finite() && (max_pixels == 0. || (0.25..=4096.).contains(&max_pixels)),
            "Invalid collective screen limit"
        );
        let gpu = self
            .collective
            .as_mut()
            .context("Install a collective packet first")?;
        gpu.max_pixels = max_pixels;
        gpu.cost_guard = cost_guard;
        Ok(())
    }
    pub fn set_collective_density(&mut self, density: f32) -> Result<()> {
        anyhow::ensure!(
            density.is_finite() && (0.25..=4096.).contains(&density),
            "Invalid triangles-per-pixel cost threshold"
        );
        self.collective
            .as_mut()
            .context("Install a collective packet first")?
            .density = density;
        Ok(())
    }
    /// Diagnostic oracle switch; normal rendering rejects only proven invisible bounds.
    pub fn set_occlusion_culling(&mut self, enabled: bool) {
        self.occlusion_enabled = enabled;
    }
    /// Diagnostic A/B control for the depth-seed work policy; source geometry is unchanged.
    pub fn set_adaptive_occluders(&mut self, enabled: bool) {
        self.instanced.set_adaptive_occluders(enabled);
    }
    /// Diagnostic A/B control; automatic fine tiles retain the same source/depth guards.
    pub fn set_fine_occlusion(&mut self, enabled: bool) {
        self.instanced.set_fine_occlusion(enabled);
    }
    /// Explicit blocking diagnostic; excluded from measured draws and runtime decisions.
    pub fn inspect_occlusion_gpu(&self) -> Result<serde_json::Value> {
        self.instanced
            .occlusion
            .diagnostic(&self.device, &self.queue)
    }
    /// Diagnostic oracle: retain the original per-part frustum path.
    pub fn set_object_culling(&mut self, enabled: bool) {
        self.visibility.objects_disabled = !enabled;
    }
    pub fn set_visibility_culling(&mut self, enabled: bool) {
        self.visibility_enabled = enabled;
    }
    /// Diagnostic A/B control. Indexing is lossless and automatic in normal rendering.
    pub fn set_geometry_indexing(&mut self, enabled: bool) {
        self.instanced.set_indexing(enabled);
    }
    /// Diagnostic A/B control for unused vertex/material channels.
    /// No authored channels are discarded from the source.
    pub fn set_minimal_vertices(&mut self, enabled: bool) {
        self.instanced.set_minimal_vertices(enabled);
    }
    /// Diagnostic A/B control for redundant per-vertex constants.
    pub fn set_compact_vertices(&mut self, enabled: bool) {
        self.instanced.set_compact_vertices(enabled);
    }
    /// Experimental projection bound. Disabled by default pending coverage validation.
    pub fn set_precise_lod_projection(&mut self, enabled: bool) {
        self.distant.precise_projection = enabled;
    }
    pub fn info(&self) -> serde_json::Value {
        let info = self.adapter.get_info();
        serde_json::json!({"name":info.name,"backend":format!("{:?}",info.backend),"device_type":format!("{:?}",info.device_type),"environment_preparation":self.environment.stats(),"distant_selection":self.distant.stats,"mesh_lods":self.distant.lods.stats,"instancing":self.instanced.stats,"collective_lod":self.collective.as_ref().map(|g|&g.stats),"object_impostor":self.object_impostor_info(),"visibility":self.visibility.stats,"occlusion":self.instanced.occlusion.stats,"device_limits":{"max_buffer_bytes":self.device.limits().max_buffer_size,"max_material_binding_bytes":self.device.limits().max_storage_buffer_binding_size},"selection_outline":self.outline.info(),"atmospheric_edges":self.atmosphere_edges.info(),"transparency":self.transparency.info(),"particles":self.particles.inspect(),"procedural_materials":{"storage_bytes":self.procedural_buffer.size(),"uploads":self.procedural_uploads,"materials":self.material_source.iter().filter(|m|m.procedural.is_some()).count()}})
    }
    pub fn draw(
        &mut self,
        scene: &RenderFrame,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<()> {
        anyhow::ensure!(width > 0 && height > 0, "Viewport cannot be empty");
        if let Some(objects) = &mut self.object_impostor {
            objects.prepare(&self.device, &self.queue, scene, width, height)?;
        }
        let result = self.draw_prepared(scene, view, width, height);
        if let Some(objects) = &mut self.object_impostor {
            objects.restore(&mut self.visibility);
        }
        result
    }
    fn draw_prepared(
        &mut self,
        scene: &RenderFrame,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<()> {
        anyhow::ensure!(width > 0 && height > 0, "Viewport cannot be empty");
        if let Some(mut proxy) = self.mesh_proxy.take() {
            let result = proxy.prepare(scene, width, height).and_then(|selected| {
                self.draw_prepared(
                    if selected {
                        proxy.frame.as_ref().unwrap()
                    } else {
                        scene
                    },
                    view,
                    width,
                    height,
                )
            });
            self.mesh_proxy = Some(proxy);
            return result;
        }
        if let Some(clip) = scene.clip {
            clip.validate()?;
        }
        let reverse = scene.clip.is_some_and(|c| c.reverse_z);
        anyhow::ensure!(
            scene.particles.samples.is_empty() || (!reverse && scene.planet_air.is_none()),
            "Particle sprites require the ordinary metre-frame adapter"
        );
        if reverse {
            anyhow::ensure!(
                scene.boxes.is_empty()
                    && scene.meshes.draws.iter().all(|d| d.shared)
                    && scene.light.shadow.is_none()
                    && scene.selection.is_none()
                    && scene.selection_mesh.is_none()
                    && scene
                        .light
                        .atmosphere
                        .and_then(|a| a.edge_softness)
                        .is_none_or(|a| !a.active())
                    && scene
                        .meshes
                        .source
                        .assets
                        .iter()
                        .all(|a| a.material.opaque_occluder())
                    && scene.vision_guides.is_empty()
                    && scene.hearing_guides.is_empty()
                    && scene.noise_guides.is_empty()
                    && scene.route_guides.is_empty(),
                "Reverse planet view requires opaque mesh-only geometry without shadows, selection, guides or depth-edge filtering"
            );
            if let Some(gpu) = &mut self.collective {
                gpu.stats = Default::default();
            }
        }

        let edge_settings = scene
            .light
            .atmosphere
            .and_then(|a| a.edge_softness)
            .filter(|s| s.active());
        let soft = edge_settings.is_some();
        if self
            .depth
            .as_ref()
            .is_none_or(|(w, h, _, readable)| *w != width || *h != height || *readable != soft)
        {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("depth"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | if soft {
                        wgpu::TextureUsages::TEXTURE_BINDING
                    } else {
                        wgpu::TextureUsages::empty()
                    },
                view_formats: &[],
            });
            self.depth = Some((
                width,
                height,
                texture.create_view(&Default::default()),
                soft,
            ));
        }
        self.environment
            .prepare(&self.queue, scene.light.environment.as_ref());
        let camera = &scene.camera;
        let vp = crate::picking::view_projection_clip(camera, width, height, scene.clip);
        let light = scene.light.direction;
        let shadow = scene.light.shadow.as_ref();
        let resolution = shadow.map_or(1, |s| s.resolution);
        if self.shadow_map.0 != resolution {
            self.shadow_map = (resolution, shadow_map(&self.device, resolution));
            self.bind_group = scene_bindings(
                &self.device,
                &self.shadow_layout,
                &self.uniform,
                &self.shadow_map.1,
                &self.shadow_sampler,
                (&self.material_buffer, &self.procedural_buffer),
                &self.environment,
            );
        }
        self.visibility
            .update(scene, vp, width, height, self.visibility_enabled);
        if let Some(gpu) = &mut self.collective
            && !reverse
        {
            gpu.prepare(
                &self.device,
                &self.queue,
                scene,
                &mut self.visibility,
                width,
                height,
            );
        }
        if let Some(objects) = &mut self.object_impostor {
            objects.mask(&mut self.visibility, scene);
        }
        self.distant.update(scene, height, &self.visibility);
        self.instanced.prepare(
            &self.device,
            &self.queue,
            scene,
            &self.distant,
            &self.visibility,
        )?;
        self.instanced.prepare_occlusion(
            &self.device,
            &self.queue,
            scene,
            &self.distant,
            &self.visibility,
            width,
            height,
            self.occlusion_enabled && scene.clip.is_none(),
            self.collective
                .as_ref()
                .map_or(0, |g| g.stats.candidate_triangles_replaced),
        );
        if let Some(gpu) = &mut self.collective
            && !reverse
        {
            gpu.set_seed(
                &self.device,
                &self.queue,
                self.instanced
                    .occlusion
                    .stats
                    .active
                    .then(|| self.instanced.occlusion.depth()),
            );
        }

        let selected_materials = if self.distant.active {
            &self.distant.materials
        } else {
            &scene.meshes.materials
        };
        // Slot zero also shades legacy boxes when the mesh frame is empty.
        // Reserve it before appending emitter materials.
        let default_material = [crate::materials::Material::default()];
        let selected_materials = if selected_materials.is_empty() {
            default_material.as_slice()
        } else {
            selected_materials.as_slice()
        };
        let particle_base = selected_materials.len() as u32;
        let all_materials = selected_materials
            .iter()
            .chain(scene.particles.materials.iter());
        if self.material_cache.is_none()
            || self.material_source.len() != all_materials.clone().count()
            || !self
                .material_source
                .iter()
                .zip(all_materials.clone())
                .all(|(a, b)| a.resources_match(b) && a == b)
        {
            let combined: Vec<_> = all_materials.cloned().collect();
            let selected_materials = &combined;
            let limit = u64::from(self.device.limits().max_storage_buffer_binding_size);
            let table_bytes = selected_materials.len().max(1) as u64
                * std::mem::size_of::<crate::materials::GpuMaterial>() as u64;
            let sample_bytes = crate::materials::recipe::storage_bytes(selected_materials);
            anyhow::ensure!(
                table_bytes <= limit && sample_bytes <= limit,
                "Material tables require {}+{} bytes; this GPU supports {} bytes per storage binding",
                table_bytes,
                sample_bytes,
                limit
            );
            let (mut material_table, procedural) =
                crate::materials::recipe::pack(selected_materials);
            if material_table.is_empty() {
                material_table.push((&crate::materials::Material::default()).into());
            }
            let bytes = bytemuck::cast_slice(&material_table);
            let procedural_bytes = bytemuck::cast_slice(&procedural);
            let limit = u64::from(self.device.limits().max_storage_buffer_binding_size);
            anyhow::ensure!(
                bytes.len() as u64 <= limit && procedural_bytes.len() as u64 <= limit,
                "Material tables require {}+{} bytes; this GPU supports {} bytes per storage binding",
                bytes.len(),
                procedural_bytes.len(),
                limit
            );
            let mut bindings_changed = false;
            if self.material_buffer.size() != bytes.len() as u64 {
                self.material_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("surface material storage"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                bindings_changed = true;
                self.material_cache = None;
            }
            if self.procedural_buffer.size() != procedural_bytes.len() as u64 {
                self.procedural_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("procedural programs and fixed samples"),
                    size: procedural_bytes.len() as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                bindings_changed = true;
            }
            if bindings_changed {
                self.bind_group = scene_bindings(
                    &self.device,
                    &self.shadow_layout,
                    &self.uniform,
                    &self.shadow_map.1,
                    &self.shadow_sampler,
                    (&self.material_buffer, &self.procedural_buffer),
                    &self.environment,
                );
                self.shadow_bind_group = depth_bindings(
                    &self.device,
                    &self.depth_layout,
                    &self.uniform,
                    &self.material_buffer,
                    &self.procedural_buffer,
                );
            }
            self.queue
                .write_buffer(&self.procedural_buffer, 0, procedural_bytes);
            self.procedural_uploads += 1;
            if self.material_cache.as_ref() != Some(&material_table) {
                self.queue.write_buffer(&self.material_buffer, 0, bytes);
                self.material_cache = Some(material_table);
            }
            self.material_source.clone_from(selected_materials);
        }
        let atmosphere = scene
            .light
            .atmosphere
            .filter(|a| a.density > 0. || a.layers.is_some());
        // Camera-relative rays avoid far-plane unprojection/cancellation at large coordinates.
        let (right, up, forward) = {
            let forward = (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize();
            let right = forward.cross(Vec3::Y).normalize();
            let scale = (camera.fov_degrees.to_radians() * 0.5).tan();
            (
                right * scale * width as f32 / height as f32,
                right.cross(forward) * scale,
                forward,
            )
        };
        self.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&Globals {
                view_projection: vp.to_cols_array_2d(),
                shadow_projection: shadow_view_projection(&scene.light)
                    .unwrap_or(Mat4::IDENTITY)
                    .to_cols_array_2d(),
                light: [light[0], light[1], light[2], scene.light.ambient],
                shadow_settings: shadow.map_or([0.; 4], |s| {
                    [
                        1.,
                        s.bias,
                        1. / s.resolution as f32,
                        f32::from(s.receiver_plane_bias),
                    ]
                }),
                view_position: [
                    camera.position[0],
                    camera.position[1],
                    camera.position[2],
                    0.,
                ],
                atmosphere: atmosphere
                    .map_or([0.; 4], |a| [a.color[0], a.color[1], a.color[2], a.density]),
                atmosphere_sun: atmosphere.map_or([0.; 4], |a| {
                    [
                        a.sun_color[0],
                        a.sun_color[1],
                        a.sun_color[2],
                        a.sun_strength,
                    ]
                }),
                atmosphere_right: right.extend(0.).to_array(),
                atmosphere_up: up.extend(0.).to_array(),
                atmosphere_forward: forward.extend(0.).to_array(),
                atmosphere_sky: atmosphere.and_then(|a| a.layers).map_or([0.; 4], |l| {
                    [l.sky_color[0], l.sky_color[1], l.sky_color[2], 1.]
                }),
                atmosphere_base: atmosphere.and_then(|a| a.layers).map_or([0.; 4], |l| {
                    [
                        l.mountain_color[0],
                        l.mountain_color[1],
                        l.mountain_color[2],
                        l.influence,
                    ]
                }),
                atmosphere_detail: atmosphere
                    .and_then(|a| a.layers)
                    .map_or([0.; 4], |l| [l.detail_start, l.detail_end, 0., 0.]),
                planet_optics: scene.planet_air.map_or([0.; 4], |p| {
                    [
                        1.,
                        (p.altitude / p.radius) as f32,
                        (p.height / p.radius) as f32,
                        (p.optical_depth * p.radius / p.height) as f32,
                    ]
                }),
                planet_radial: scene.planet_air.map_or([0.; 4], |p| {
                    [p.radial[0], p.radial[1], p.radial[2], p.radius as f32]
                }),
                planet_space: scene.planet_air.map_or([0.; 4], |p| {
                    [
                        p.space_color[0],
                        p.space_color[1],
                        p.space_color[2],
                        p.terrain_materials as f32,
                    ]
                }),
                planet_eye_high: scene.planet_air.map_or([0.; 4], |p| {
                    [
                        p.eye.high[0],
                        p.eye.high[1],
                        p.eye.high[2],
                        (crate::atmosphere::transport::correction_limit(p.radius, p.altitude)
                            / p.radius) as f32,
                    ]
                }),
                planet_eye_low: scene
                    .planet_air
                    .map_or([0.; 4], |p| [p.eye.low[0], p.eye.low[1], p.eye.low[2], 0.]),
                planet_axis0: scene
                    .planet_air
                    .map_or([0.; 4], |p| [p.axes[0][0], p.axes[0][1], p.axes[0][2], 0.]),
                planet_axis1: scene
                    .planet_air
                    .map_or([0.; 4], |p| [p.axes[1][0], p.axes[1][1], p.axes[1][2], 0.]),
                planet_axis2: scene
                    .planet_air
                    .map_or([0.; 4], |p| [p.axes[2][0], p.axes[2][1], p.axes[2][2], 0.]),
                direct_light: [
                    scene.light.direct_color[0] * scene.light.direct_intensity,
                    scene.light.direct_color[1] * scene.light.direct_intensity,
                    scene.light.direct_color[2] * scene.light.direct_intensity,
                    f32::from(scene.light.direct_color == [1.; 3]),
                ],
                environment: [
                    scene.light.environment.map_or(0., |e| e.intensity),
                    6.,
                    scene.light.direct_intensity,
                    f32::from(scene.suppress_diagnostic_grid),
                ],
            }),
        );
        let mut dirty = self.geometry.update(&self.visibility.boxes);
        let count = self.geometry.vertices.len();
        if count > self.vertex_capacity {
            self.vertex_capacity = count.next_power_of_two();
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("scene vertices"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            dirty = Some(0..count);
        }
        if let Some(span) = dirty {
            self.queue.write_buffer(
                &self.vertex_buffer,
                (span.start * std::mem::size_of::<Vertex>()) as u64,
                bytemuck::cast_slice(&self.geometry.vertices[span]),
            );
        }
        let selected_vertices = if self.distant.active {
            &self.distant.full
        } else {
            &scene.meshes.vertices
        };
        let mesh_count = selected_vertices.len();
        for (name, count, stride) in [
            ("expanded mesh vertices", mesh_count, 64usize),
            ("atmospheric vertices", self.distant.cheap.len(), 24),
            ("atmospheric indices", self.distant.indices.len(), 4),
        ] {
            let bytes = mesh_buffer_capacity(count).checked_mul(stride);
            anyhow::ensure!(
                count <= u32::MAX as usize
                    && bytes.is_some_and(|n| n as u64 <= self.device.limits().max_buffer_size),
                "{name}: {count} elements exceed this device's buffer/draw capacity ({} bytes per buffer)",
                self.device.limits().max_buffer_size
            );
        }

        // Compare the upload payload directly. Field-by-field float equality
        // scans millions of components even when the static mesh is unchanged.
        if bytemuck::cast_slice::<_, u8>(&self.mesh_vertices)
            != bytemuck::cast_slice::<_, u8>(selected_vertices)
        {
            if mesh_count > self.mesh_capacity
                || self.mesh_capacity > mesh_buffer_capacity(mesh_count) * 5 / 4
            {
                self.mesh_capacity = mesh_buffer_capacity(mesh_count);
                self.mesh_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("native mesh vertices"),
                    size: (self.mesh_capacity * std::mem::size_of::<Vertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            if mesh_count > 0 {
                self.queue.write_buffer(
                    &self.mesh_buffer,
                    0,
                    bytemuck::cast_slice(selected_vertices),
                );
            }
            self.mesh_vertices.clone_from(selected_vertices);
        }
        let distant_count = self.distant.cheap.len();
        if bytemuck::cast_slice::<_, u8>(&self.distant_vertices)
            != bytemuck::cast_slice::<_, u8>(&self.distant.cheap)
        {
            if distant_count > self.distant_capacity
                || self.distant_capacity > mesh_buffer_capacity(distant_count) * 5 / 4
            {
                self.distant_capacity = mesh_buffer_capacity(distant_count);
                self.distant_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("selected distant vertices"),
                    size: (self.distant_capacity * 24) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            if distant_count > 0 {
                self.queue.write_buffer(
                    &self.distant_buffer,
                    0,
                    bytemuck::cast_slice(&self.distant.cheap),
                );
            }
            self.distant_vertices.clone_from(&self.distant.cheap);
        }
        let index_count = self.distant.indices.len();
        if self.distant_indices != self.distant.indices {
            let capacity = mesh_buffer_capacity(index_count);
            if index_count > self.distant_index_capacity
                || self.distant_index_capacity > capacity * 5 / 4
            {
                self.distant_index_capacity = capacity;
                self.distant_index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("selected distant indices"),
                    size: (capacity * 4) as u64,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            if index_count > 0 {
                self.queue.write_buffer(
                    &self.distant_index_buffer,
                    0,
                    bytemuck::cast_slice(&self.distant.indices),
                );
            }
            self.distant_indices.clone_from(&self.distant.indices);
        }
        self.distant.stats.allocated_index_bytes =
            self.distant_index_capacity * 4 + self.instanced.stats.allocated_index_bytes;
        self.distant.stats.allocated_vertex_bytes = self.mesh_capacity * 64
            + self.distant_capacity * 24
            + self.instanced.stats.allocated_geometry_bytes
            - self.instanced.stats.allocated_index_bytes;
        self.distant.stats.allocated_geometry_bytes =
            self.distant.stats.allocated_vertex_bytes + self.distant.stats.allocated_index_bytes;
        self.distant.stats.detailed_triangles =
            mesh_count / 3 + self.instanced.stats.detailed_triangles;
        self.distant.stats.cheap_triangles =
            self.distant.indices.len() / 3 + self.instanced.stats.cheap_triangles;
        self.distant.stats.selected_vertex_bytes = mesh_count * 64
            + self.distant.cheap.len() * 24
            + self.instanced.stats.selected_geometry_bytes
            - self.instanced.stats.selected_index_bytes;
        self.distant.stats.selected_index_bytes =
            self.distant.indices.len() * 4 + self.instanced.stats.selected_index_bytes;
        self.distant.stats.selected_geometry_bytes =
            self.distant.stats.selected_vertex_bytes + self.distant.stats.selected_index_bytes;
        self.distant.stats.substituted_instances = self
            .distant
            .plan
            .iter()
            .filter(|c| c.color.is_some())
            .count();
        self.distant.stats.geometric_lod_instances = self
            .distant
            .plan
            .iter()
            .filter(|c| c.level.is_some())
            .count();
        let guides_changed = self.guides.update(
            &scene.vision_guides,
            &scene.hearing_guides,
            &scene.noise_guides,
            &scene.route_guides,
            None,
        );
        let guide_count = self.guides.vertices.len();
        if guide_count > self.guide_capacity {
            self.guide_capacity = guide_count.next_power_of_two();
            self.guide_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vision guide vertices"),
                size: (self.guide_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if guides_changed && guide_count > 0 {
            self.queue.write_buffer(
                &self.guide_buffer,
                0,
                bytemuck::cast_slice(&self.guides.vertices),
            );
        }
        self.atmosphere_edges.prepare(
            &self.device,
            &self.queue,
            self.format,
            &self.depth.as_ref().unwrap().2,
            width,
            height,
            &scene.camera,
            edge_settings,
        );
        self.particles
            .prepare(&self.device, &self.queue, &scene.particles, particle_base);
        let transparent_draw_units = self.instanced.transparent_draw_units()
            + usize::from(!scene.particles.samples.is_empty());
        self.transparency.prepare(
            &self.device,
            self.format,
            width,
            height,
            transparent_draw_units,
        );
        let composed_view = self.transparency.background().unwrap_or(view);
        let scene_view = self.atmosphere_edges.source().unwrap_or(composed_view);
        if let Some(field) = &mut self.surface_field {
            anyhow::ensure!(
                !reverse,
                "Diagnostic surface field does not support planetary reverse depth"
            );
            field.prepare(&self.queue, &scene.camera, width, height, &scene.light);
        }
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.instanced.encode_occlusion(
            &mut encoder,
            &self.bind_group,
            &self.mesh_buffer,
            &self.distant_buffer,
            &self.distant_index_buffer,
        );
        if shadow.is_some() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("directional shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_map.1,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.shadow_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            for range in &self.visibility.box_shadow {
                pass.draw(range.clone(), 0..1);
            }
            self.instanced.draw(
                &mut pass,
                crate::render_instances::Pass::Shadow,
                &self.shadow_pipeline,
                &self.distant_pipeline,
                &self.mesh_buffer,
                &self.distant_buffer,
                &self.distant_index_buffer,
            );
            if let Some(field) = &self.surface_field {
                field.draw(&mut pass, &self.shadow_bind_group, true);
            }
            if let Some(objects) = &self.object_impostor {
                objects.draw_shadow(&mut pass, &self.shadow_bind_group);
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: scene_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.027,
                            b: 0.045,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.as_ref().unwrap().2,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(if reverse { 0. } else { 1. }),
                        store: if soft
                            || transparent_draw_units > 0
                            || scene.selection.is_some()
                            || scene.selection_mesh.is_some()
                        {
                            wgpu::StoreOp::Store
                        } else {
                            wgpu::StoreOp::Discard
                        },
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            for range in &self.visibility.box_camera {
                pass.draw(range.clone(), 0..1);
            }
            self.instanced.draw(
                &mut pass,
                crate::render_instances::Pass::Opaque,
                &self.pipeline,
                &self.distant_pipeline,
                &self.mesh_buffer,
                &self.distant_buffer,
                &self.distant_index_buffer,
            );
            if let Some(field) = &self.surface_field {
                field.draw(&mut pass, &self.bind_group, false);
            }
            if let Some(gpu) = &self.collective
                && !reverse
            {
                gpu.draw(&mut pass, &self.bind_group);
            }
            if atmosphere.is_some() {
                pass.set_pipeline(if reverse {
                    &self.atmosphere_reverse
                } else {
                    &self.atmosphere_pipeline
                });
                pass.draw(0..3, 0..1);
            }
            if let Some(objects) = &self.object_impostor {
                objects.draw(&mut pass, &self.bind_group);
            }
            if guide_count > 0 && !soft && transparent_draw_units == 0 {
                pass.set_pipeline(&self.guide_pipeline);
                pass.set_vertex_buffer(0, self.guide_buffer.slice(..));
                pass.draw(0..guide_count as u32, 0..1);
            }
        }
        self.atmosphere_edges.draw(&mut encoder, composed_view);
        if let Some(layers) = self.transparency.layers() {
            let attachments: [_; 3] = std::array::from_fn(|i| {
                Some(wgpu::RenderPassColorAttachment {
                    view: &layers[i],
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(if i == 1 {
                            wgpu::Color::WHITE
                        } else {
                            wgpu::Color::TRANSPARENT
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("order-independent transparent surfaces"),
                color_attachments: &attachments,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.as_ref().unwrap().2,
                    depth_ops: None,
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &self.bind_group, &[]);
            self.instanced.draw(
                &mut pass,
                crate::render_instances::Pass::Transparent,
                &self.transparent_pipeline,
                &self.distant_pipeline,
                &self.mesh_buffer,
                &self.distant_buffer,
                &self.distant_index_buffer,
            );
            self.particles.draw(&mut pass);
        }
        self.transparency.composite(&mut encoder, view);
        if (soft || transparent_draw_units > 0) && guide_count > 0 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sharp diagnostic overlays"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.as_ref().unwrap().2,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.guide_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.guide_buffer.slice(..));
            pass.draw(0..guide_count as u32, 0..1);
        }
        self.outline.prepare(&self.device, &self.queue, scene);
        self.outline.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            view,
            &self.depth.as_ref().unwrap().2,
            width,
            height,
            vp,
        );
        self.queue.submit([encoder.finish()]);
        Ok(())
    }
    pub fn capture(&mut self, scene: &RenderFrame, width: u32, height: u32) -> Result<Vec<u8>> {
        anyhow::ensure!(
            (64..=2048).contains(&width) && (64..=2048).contains(&height),
            "Capture dimensions must be in [64,2048]"
        );
        anyhow::ensure!(
            self.format == wgpu::TextureFormat::Rgba8UnormSrgb,
            "Capture requires RGBA renderer"
        );
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen capture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.draw(
            scene,
            &texture.create_view(&Default::default()),
            width,
            height,
        )?;
        self.read_texture_png(&texture, width, height)
    }
    pub(crate) fn read_texture_png(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let padded = (width * 4).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("capture readback"),
            size: padded as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            size,
        );
        let submission = self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(std::time::Duration::from_secs(20)),
        })?;
        rx.recv_timeout(std::time::Duration::from_secs(20))??;
        let mapped = buffer.slice(..).get_mapped_range();
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for row in mapped.chunks(padded as usize) {
            pixels.extend_from_slice(&row[..(width * 4) as usize]);
        }
        drop(mapped);
        buffer.unmap();
        let mut png_bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_bytes, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&pixels)?;
        }
        Ok(png_bytes)
    }
}

// Crossed ribbons keep vertical and camera-aligned edges visible without a new pipeline.
fn selection_vertices(selected: RenderBox, out: &mut Vec<Vertex>) {
    if !selected.geometry.is_box() {
        let vertices = selected.geometry.vertices(selected.size);
        for [a, b] in crate::geometry::EDGES {
            let a = Vec3::from(selected.position) + vertices[a];
            let b = Vec3::from(selected.position) + vertices[b];
            let delta = b - a;
            let mut side = delta.cross(Vec3::Y);
            if side.length_squared() < 1e-12 {
                side = delta.cross(Vec3::X);
            }
            let side = side.normalize() * 0.018;
            for side in [side, delta.normalize().cross(side)] {
                for p in [a - side, a + side, b + side, a - side, b + side, b - side] {
                    out.push(Vertex {
                        material_projection_weights: [1. / 3.; 3],
                        rest_position: [0.; 3],
                        material: 0,
                        position: p.to_array(),
                        normal: [0., 1., 0.],
                        color: selected.color,
                    });
                }
            }
        }
        return;
    }
    let center = Vec3::from(selected.position);
    let half = Vec3::from(selected.size) * 0.5;
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        for su in [-1., 1.] {
            for sv in [-1., 1.] {
                let mut a = center;
                a[axis] -= half[axis];
                a[u] += su * half[u];
                a[v] += sv * half[v];
                let mut b = a;
                b[axis] += 2. * half[axis];
                for side_axis in [u, v] {
                    let mut side = Vec3::ZERO;
                    side[side_axis] = 0.018;
                    for point in [a - side, a + side, b + side, a - side, b + side, b - side] {
                        out.push(Vertex {
                            material_projection_weights: [1. / 3.; 3],
                            rest_position: [0.; 3],
                            material: 0,
                            position: point.into(),
                            normal: [0., 1., 0.],
                            color: selected.color,
                        });
                    }
                }
            }
        }
    }
}

fn direction(degrees: f32) -> Vec3 {
    let (sin, cos) = degrees.to_radians().sin_cos();
    Vec3::new(sin, 0., -cos)
}
fn ribbon(a: Vec3, b: Vec3, width: f32, color: [f32; 3], out: &mut Vec<Vertex>) {
    let delta = b - a;
    let mut side = delta.cross(Vec3::Y);
    if side.length_squared() < 1e-12 {
        side = delta.cross(Vec3::X);
    }
    let side = side.normalize_or_zero() * width * 0.5;
    for point in [a - side, a + side, b + side, a - side, b + side, b - side] {
        out.push(Vertex {
            material_projection_weights: [1. / 3.; 3],
            rest_position: [0.; 3],
            material: 0,
            position: point.to_array(),
            normal: [0., 1., 0.],
            color,
        });
    }
}
fn line_vertices(g: &LineGuide, out: &mut Vec<Vertex>) {
    let a = Vec3::from(g.start);
    let b = Vec3::from(g.end);
    if g.dashed {
        for i in 0..8 {
            ribbon(
                a.lerp(b, i as f32 / 8.),
                a.lerp(b, (i as f32 + 0.5) / 8.),
                0.035,
                g.color,
                out,
            );
        }
    } else {
        ribbon(a, b, 0.055, g.color, out);
    }
}
fn ring_vertices(g: &RingGuide, out: &mut Vec<Vertex>) {
    let center = Vec3::from(g.center);
    for i in 0..48 {
        if !g.emitter || i % 2 == 0 {
            ribbon(
                center + direction(i as f32 * 7.5) * g.radius,
                center + direction((i + 1) as f32 * 7.5) * g.radius,
                0.045,
                g.color,
                out,
            );
        }
    }
    if g.emitter {
        for axis in [Vec3::X, Vec3::Z] {
            ribbon(center - axis * 0.2, center + axis * 0.2, 0.07, g.color, out);
        }
    }
}
fn guide_vertices(g: &VisionGuide, out: &mut Vec<Vertex>) {
    if g.directional
        && let Some(eye) = g.spatial_eye
    {
        spatial_guide_vertices(g, Vec3::from(eye), out);
        return;
    }
    let center = Vec3::new(g.position[0], g.top_y, g.position[2]);
    if g.directional {
        let forward = direction(g.facing_degrees);
        let side = forward.cross(Vec3::Y);
        let tip = center + forward * 1.3;
        ribbon(center, tip, 0.09, g.color, out);
        ribbon(tip, tip - forward * 0.38 + side * 0.27, 0.09, g.color, out);
        ribbon(tip, tip - forward * 0.38 - side * 0.27, 0.09, g.color, out);
    } else {
        // Omniscience has no meaningful gaze; use a diamond instead of a cone/arrow.
        for i in 0..4 {
            ribbon(
                center + direction(i as f32 * 90.) * 0.5,
                center + direction((i + 1) as f32 * 90.) * 0.5,
                0.08,
                g.color,
                out,
            );
        }
    }
    if g.range <= 0. {
        return;
    }
    let base = Vec3::new(g.position[0], g.ground_y, g.position[2]);
    let start = g.facing_degrees - g.fov_degrees * 0.5;
    // Dashed nominal arc: it deliberately does not claim line-of-sight around obstacles.
    for i in 0..48 {
        if i % 2 == 0 {
            ribbon(
                base + direction(start + g.fov_degrees * i as f32 / 48.) * g.range,
                base + direction(start + g.fov_degrees * (i + 1) as f32 / 48.) * g.range,
                0.045,
                g.color,
                out,
            );
        }
    }
    if g.fov_degrees < 360. {
        for degrees in [start, start + g.fov_degrees] {
            let edge = direction(degrees) * g.range;
            for i in 0..12 {
                ribbon(
                    base + edge * (i as f32 / 12.),
                    base + edge * ((i as f32 + 0.55) / 12.),
                    0.035,
                    g.color,
                    out,
                );
            }
        }
    }
}
/// Nominal spherical cap, not an occlusion map. Fixed resolution bounds guide cost.
fn spatial_guide_vertices(g: &VisionGuide, eye: Vec3, out: &mut Vec<Vertex>) {
    let forward = crate::perception::view_direction(g.facing_degrees, g.pitch_degrees);
    let reference = if forward.y.abs() > 0.99 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let right = forward.cross(reference).normalize();
    let up = right.cross(forward).normalize();
    let tip = eye + forward * 1.3;
    ribbon(eye, tip, 0.09, g.color, out);
    for sign in [-1., 1.] {
        ribbon(
            tip,
            tip - forward * 0.38 + right * (sign * 0.27),
            0.09,
            g.color,
            out,
        );
    }
    if g.range <= 0. {
        return;
    }
    let half = (g.fov_degrees * 0.5).to_radians();
    let (sin, cos) = half.sin_cos();
    if g.fov_degrees < 360. {
        let center = eye + forward * (g.range * cos);
        for i in (0..48).step_by(2) {
            let point = |i: usize| {
                let (s, c) = (i as f32 * std::f32::consts::TAU / 48.).sin_cos();
                center + (right * c + up * s) * (g.range * sin)
            };
            ribbon(point(i), point(i + 1), 0.045, g.color, out);
        }
    }
    for tangent in [right, -right, up, -up] {
        for i in (0..24).step_by(2) {
            let point = |i: usize| {
                let (s, c) = (half * i as f32 / 24.).sin_cos();
                eye + (forward * c + tangent * s) * g.range
            };
            ribbon(point(i), point(i + 1), 0.035, g.color, out);
        }
        if g.fov_degrees < 360. {
            let edge = (forward * cos + tangent * sin) * g.range;
            for i in (0..12).step_by(2) {
                ribbon(
                    eye + edge * (i as f32 / 12.),
                    eye + edge * ((i + 1) as f32 / 12.),
                    0.035,
                    g.color,
                    out,
                );
            }
        }
    }
}

pub(crate) fn box_vertices(e: &RenderBox, out: &mut [Vertex]) {
    if !e.geometry.is_box() {
        let vertices = e.geometry.vertices(e.size);
        let center = vertices.iter().copied().sum::<Vec3>() / 6.;
        out.fill(Vertex {
            material_projection_weights: [1. / 3.; 3],
            rest_position: [0.; 3],
            material: 0,
            position: e.position,
            normal: [0., 1., 0.],
            color: e.color,
        });
        for (i, [a, b, c]) in crate::geometry::TRIANGLES.into_iter().enumerate() {
            let mut points = [vertices[a], vertices[b], vertices[c]];
            let mut normal = (points[1] - points[0]).cross(points[2] - points[0]);
            if normal.dot((points[0] + points[1] + points[2]) / 3. - center) < 0. {
                points.swap(1, 2);
                normal = -normal;
            }
            let normal = normal.normalize().to_array();
            for (j, p) in points.into_iter().enumerate() {
                out[i * 3 + j] = Vertex {
                    material_projection_weights: [1. / 3.; 3],
                    rest_position: [0.; 3],
                    material: 0,
                    position: (Vec3::from(e.position) + p).to_array(),
                    normal,
                    color: e.color,
                };
            }
        }
        return;
    }
    let faces = [
        (
            [1., 0., 0.],
            [[1., -1., -1.], [1., 1., -1.], [1., 1., 1.], [1., -1., 1.]],
        ),
        (
            [-1., 0., 0.],
            [
                [-1., -1., 1.],
                [-1., 1., 1.],
                [-1., 1., -1.],
                [-1., -1., -1.],
            ],
        ),
        (
            [0., 1., 0.],
            [[-1., 1., -1.], [-1., 1., 1.], [1., 1., 1.], [1., 1., -1.]],
        ),
        (
            [0., -1., 0.],
            [
                [-1., -1., 1.],
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., -1., 1.],
            ],
        ),
        (
            [0., 0., 1.],
            [[1., -1., 1.], [1., 1., 1.], [-1., 1., 1.], [-1., -1., 1.]],
        ),
        (
            [0., 0., -1.],
            [
                [-1., -1., -1.],
                [-1., 1., -1.],
                [1., 1., -1.],
                [1., -1., -1.],
            ],
        ),
    ];
    let position = Vec3::from(e.position);
    let half = Vec3::from(e.size) * 0.5;
    let mut index = 0;
    for (normal, corners) in &faces {
        for i in [0, 1, 2, 0, 2, 3] {
            out[index] = Vertex {
                material_projection_weights: [1. / 3.; 3],
                rest_position: [0.; 3],
                material: 0,
                position: (position + Vec3::from(corners[i]) * half).to_array(),
                normal: *normal,
                color: e.color,
            };
            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wedge_render_has_outward_solid_faces_and_degenerate_padding_only() {
        use crate::geometry::{Geometry, Rise, Volume};
        for rise in [
            Rise::PositiveX,
            Rise::NegativeX,
            Rise::PositiveZ,
            Rise::NegativeZ,
        ] {
            let item = RenderBox {
                geometry: Geometry::Wedge { rise },
                position: [2., 3., 4.],
                size: [6., 2., 4.],
                color: [0.2, 0.5, 0.7],
            };
            let mut vertices = [Vertex::zeroed(); 36];
            box_vertices(&item, &mut vertices);
            let volume = Volume {
                geometry: item.geometry,
                center: item.position.into(),
                size: item.size.into(),
            };
            let center = item
                .geometry
                .vertices(item.size)
                .iter()
                .copied()
                .sum::<Vec3>()
                / 6.
                + Vec3::from(item.position);
            for triangle in vertices[..24].chunks_exact(3) {
                let p = [
                    Vec3::from(triangle[0].position),
                    Vec3::from(triangle[1].position),
                    Vec3::from(triangle[2].position),
                ];
                let normal = Vec3::from(triangle[0].normal);
                assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 0.);
                assert!(normal.dot((p[0] + p[1] + p[2]) / 3. - center) > 0.);
                for v in triangle {
                    let p = Vec3::from(v.position);
                    assert!(p.is_finite());
                    assert!(
                        volume
                            .cast(p, Vec3::ZERO, Vec3::splat(0.00001), false, 0.)
                            .is_some()
                    );
                }
            }
            assert!(
                vertices[24..]
                    .iter()
                    .all(|v| v.position == item.position && Vec3::from(v.normal).is_finite())
            );
        }
    }

    #[test]
    fn geometry_cache_and_selection_invalidate_shape_and_restore_box() {
        let mut item = RenderBox {
            geometry: Default::default(),
            position: [0.; 3],
            size: [4., 2., 4.],
            color: [1.; 3],
        };
        let mut cache = GeometryCache::default();
        assert_eq!(cache.update(&[item]), Some(0..36));
        let original = bytemuck::cast_slice::<_, u8>(&cache.vertices).to_vec();
        item.geometry = crate::geometry::Geometry::Wedge {
            rise: crate::geometry::Rise::PositiveX,
        };
        assert_eq!(cache.update(&[item]), Some(0..36));
        assert!(cache.update(&[item]).is_none());
        let mut guides = GuideCache::default();
        assert!(guides.update(&[], &[], &[], &[], Some(item)));
        assert_eq!(guides.vertices.len(), 108);
        item.geometry = Default::default();
        assert_eq!(cache.update(&[item]), Some(0..36));
        assert_eq!(bytemuck::cast_slice::<_, u8>(&cache.vertices), original);
        assert!(guides.update(&[], &[], &[], &[], Some(item)));
        assert_eq!(guides.vertices.len(), 144);
        cache.update(&[]);
        assert!(cache.vertices.is_empty());
    }

    #[test]
    fn spatial_guides_render_vertical_arrows_and_invalidate_pitch_cache() {
        let mut g = VisionGuide {
            position: [0.; 3],
            top_y: 2.,
            ground_y: 0.,
            facing_degrees: 0.,
            spatial_eye: Some([0., 1., 0.]),
            pitch_degrees: 90.,
            range: 6.,
            fov_degrees: 60.,
            directional: true,
            color: [0.1, 0.6, 1.],
        };
        let mut cache = GuideCache::default();
        assert!(cache.update(&[g], &[], &[], &[], None));
        let a = Vec3::from(cache.vertices[0].position);
        let b = Vec3::from(cache.vertices[1].position);
        let c = Vec3::from(cache.vertices[2].position);
        assert!(
            (b - a).cross(c - a).length() > 0.01,
            "vertical ribbon must have area"
        );
        assert!(
            cache
                .vertices
                .iter()
                .all(|v| Vec3::from(v.position).is_finite())
        );
        let first = bytemuck::cast_slice::<_, u8>(&cache.vertices).to_vec();
        assert!(!cache.update(&[g], &[], &[], &[], None));
        g.pitch_degrees = -90.;
        assert!(cache.update(&[g], &[], &[], &[], None));
        assert_ne!(bytemuck::cast_slice::<_, u8>(&cache.vertices), first);
        g.fov_degrees = 360.;
        assert!(cache.update(&[g], &[], &[], &[], None));
        assert!(cache.vertices.len() < 1000);
        g.range = 0.;
        assert!(cache.update(&[g], &[], &[], &[], None));
        assert_eq!(cache.vertices.len(), 18);
    }

    #[test]
    fn guide_geometry_tracks_yaw_and_reuses_cache_without_stale_vertices() {
        assert!(direction(0.).abs_diff_eq(Vec3::NEG_Z, 1e-6));
        assert!(direction(90.).abs_diff_eq(Vec3::X, 1e-6));
        assert!(direction(-90.).abs_diff_eq(Vec3::NEG_X, 1e-6));
        let mut g = VisionGuide {
            position: [0.; 3],
            top_y: 2.,
            ground_y: 0.04,
            facing_degrees: 0.,
            spatial_eye: None,
            pitch_degrees: 0.,
            range: 6.,
            fov_degrees: 120.,
            directional: true,
            color: [0.1, 0.65, 1.],
        };
        let mut cache = GuideCache::default();
        assert!(cache.update(&[g], &[], &[], &[], None));
        assert_eq!(cache.vertices.len(), 306);
        let ptr = cache.vertices.as_ptr();
        let initial = bytemuck::cast_slice::<_, u8>(&cache.vertices).to_vec();
        assert!(!cache.update(&[g], &[], &[], &[], None));
        assert_eq!(cache.vertices.as_ptr(), ptr);
        g.facing_degrees = 90.;
        assert!(cache.update(&[g], &[], &[], &[], None));
        assert_eq!(cache.vertices.as_ptr(), ptr);
        assert_ne!(bytemuck::cast_slice::<_, u8>(&cache.vertices), initial);
        // The shaft's two tip vertices average to +X for a 90 degree yaw.
        let tip =
            (Vec3::from(cache.vertices[2].position) + Vec3::from(cache.vertices[5].position)) * 0.5;
        assert!(tip.abs_diff_eq(Vec3::new(1.3, 2., 0.), 1e-6));
        g.fov_degrees = 360.;
        assert!(cache.update(&[g], &[], &[], &[], None));
        assert_eq!(cache.vertices.len(), 162);
        assert!(cache.update(&[g; 8], &[], &[], &[], None));
        assert_eq!(cache.vertices.len(), 8 * 162);
        assert!(cache.update(&[], &[], &[], &[], None));
        assert!(cache.vertices.is_empty());
        g.range = 0.;
        g.directional = false;
        assert!(cache.update(&[g], &[], &[], &[], None));
        assert_eq!(cache.vertices.len(), 24);
    }

    #[test]
    fn acoustic_rings_reuse_buffers_and_clear_all_layers() {
        let listener = RingGuide {
            center: [0., 0.05, 0.],
            radius: 12.,
            color: [0.75, 0.45, 1.],
            emitter: false,
        };
        let noise = RingGuide {
            radius: 3.,
            emitter: true,
            ..listener
        };
        let mut cache = GuideCache::default();
        assert!(cache.update(&[], &[listener; 8], &[noise; 16], &[], None));
        assert_eq!(cache.vertices.len(), 8 * 288 + 16 * 156);
        let ptr = cache.vertices.as_ptr();
        assert!(!cache.update(&[], &[listener; 8], &[noise; 16], &[], None));
        assert_eq!(cache.vertices.as_ptr(), ptr);
        assert!(cache.update(&[], &[listener], &[], &[], None));
        assert_eq!(cache.vertices.len(), 288);
        assert_eq!(cache.vertices.as_ptr(), ptr);
        assert!(cache.update(&[], &[], &[], &[], None));
        assert!(cache.vertices.is_empty());
    }

    #[test]
    fn route_lines_cache_growth_style_shrink_and_removal() {
        let line = LineGuide {
            start: [0., 0.07, 0.],
            end: [2., 0.07, 2.],
            color: [1.; 3],
            dashed: false,
        };
        let mut lines = vec![line; 8 * (32 + 256 + 2)];
        lines.extend(
            [LineGuide {
                dashed: true,
                ..line
            }; 8 * 16],
        );
        let mut cache = GuideCache::default();
        assert!(cache.update(&[], &[], &[], &lines, None));
        assert_eq!(cache.vertices.len(), 20064);
        let ptr = cache.vertices.as_ptr();
        assert!(!cache.update(&[], &[], &[], &lines, None));
        assert_eq!(cache.vertices.as_ptr(), ptr);
        lines.truncate(1);
        assert!(cache.update(&[], &[], &[], &lines, None));
        assert_eq!(cache.vertices.len(), 6);
        lines[0].dashed = true;
        lines[0].color = [1., 0., 0.];
        assert!(cache.update(&[], &[], &[], &lines, None));
        assert_eq!(cache.vertices.len(), 48);
        assert_eq!(cache.vertices.as_ptr(), ptr);
        assert!(cache.update(&[], &[], &[], &[], None));
        assert!(cache.vertices.is_empty());
    }

    #[test]
    fn selection_outline_handles_vertical_edges_and_reuses_cache() {
        let mut cache = GuideCache::default();
        let mut selected = RenderBox {
            geometry: Default::default(),
            position: [0., 1., 0.],
            size: [1., 2., 1.],
            color: [1., 0.8, 0.15],
        };
        assert!(cache.update(&[], &[], &[], &[], Some(selected)));
        assert_eq!(cache.vertices.len(), 144);
        let ptr = cache.vertices.as_ptr();
        for triangle in cache.vertices.chunks_exact(3) {
            let a = Vec3::from(triangle[0].position);
            let b = Vec3::from(triangle[1].position);
            let c = Vec3::from(triangle[2].position);
            assert!((b - a).cross(c - a).length_squared() > 0.);
        }
        assert!(!cache.update(&[], &[], &[], &[], Some(selected)));
        selected.position[0] = 3.;
        assert!(cache.update(&[], &[], &[], &[], Some(selected)));
        assert_eq!(cache.vertices.as_ptr(), ptr);
        assert!(cache.update(&[], &[], &[], &[], None));
        assert!(cache.vertices.is_empty());
        assert_eq!(cache.vertices.as_ptr(), ptr);
    }

    #[test]
    fn geometry_cache_invalidates_content_growth_removal_and_reordering() {
        let a = RenderBox {
            geometry: Default::default(),
            position: [0., 1., 0.],
            size: [2.; 3],
            color: [1., 0., 0.],
        };
        let b = RenderBox {
            position: [4., 1., 0.],
            ..a
        };
        let mut cache = GeometryCache::default();
        assert_eq!(cache.update(&[a, b]), Some(0..72));
        let original = bytemuck::cast_slice::<_, u8>(&cache.vertices).to_vec();
        let ptr = cache.vertices.as_ptr();
        assert_eq!(cache.update(&[a, b]), None);
        assert_eq!(cache.vertices.as_ptr(), ptr);
        for changed in [
            RenderBox {
                color: [0., 1., 0.],
                ..b
            },
            RenderBox { size: [4.; 3], ..b },
            RenderBox {
                position: [8., 2., 0.],
                ..b
            },
        ] {
            assert_eq!(cache.update(&[a, changed]), Some(36..72));
            assert_ne!(bytemuck::cast_slice::<_, u8>(&cache.vertices), original);
        }
        assert_eq!(cache.update(&[a, b]), Some(36..72));
        assert_eq!(bytemuck::cast_slice::<_, u8>(&cache.vertices), original);
        assert_eq!(cache.update(&[a]), None);
        assert_eq!(cache.vertices.len(), 36);
        assert_eq!(cache.update(&[a, b]), Some(36..72));
        assert_eq!(cache.update(&[b, a]), Some(0..72));
        assert_eq!(cache.update(&[]), None);
        assert!(cache.vertices.is_empty());
        assert_eq!(cache.update(&[a, b]), Some(0..72));
        assert_eq!(bytemuck::cast_slice::<_, u8>(&cache.vertices), original);
    }
}
