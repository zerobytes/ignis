use super::*;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    matrix: [[f32; 4]; 4],
    eye: [f32; 4],
    light: [f32; 4],
    settings: [u32; 4],
}
pub(super) struct Uploaded {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    reversed: Option<wgpu::Buffer>,
    packed: Vec<(wgpu::Buffer, wgpu::Buffer)>,
    native_range: std::ops::Range<u32>,
    group: wgpu::BindGroup,
    _normal: wgpu::Texture,
    pub(super) field: Option<geometric_field::Data>,
}
struct Target {
    color: wgpu::Texture,
    view: wgpu::TextureView,
    _depth: wgpu::Texture,
    depth: wgpu::TextureView,
    read: wgpu::Buffer,
    surface: Option<(wgpu::TextureView, wgpu::BindGroup)>,
    depth_group: Option<wgpu::BindGroup>,
}
pub(super) struct Renderer {
    pub gpu: Gpu,
    pub(super) density_sources: Vec<density::Dense>,
    pub(super) cost_model: Option<cost::Model>,
    pub(super) shading_model: Option<automatic::Model>,
    pub(super) material_symbol: Option<super::material_symbol::Pipelines>,
    pub(super) geometric_field: Option<geometric_field::Pipeline>,
    stage_queries: bool,
    uniform: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    diagnostics: Vec<wgpu::RenderPipeline>,
    deferred: Option<(wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::Buffer)>,
    depth_shade: Option<(wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::Buffer)>,
    depth_pipeline: Option<wgpu::RenderPipeline>,
    packed_pipeline: Option<wgpu::RenderPipeline>,
    target: Option<Target>,
    size: [u32; 2],
}
pub(super) fn pose(frame: usize, size: [u32; 2]) -> Camera {
    let step = if frame <= 4 { frame } else { 8 - frame };
    let distance = 14. * 2f32.powi(step as i32);
    camera_at(distance, size)
}
pub(super) fn camera_at(distance: f32, size: [u32; 2]) -> Camera {
    let eye = Vec3::new(0.15, 0.66, 0.736).normalize() * distance;
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let matrix = Mat4::perspective_infinite_reverse_rh(
        60f32.to_radians(),
        size[0] as f32 / size[1] as f32,
        0.1,
    ) * view;
    Camera {
        eye,
        view,
        matrix,
        inverse: matrix.inverse(),
        size,
        light: Vec3::new(-0.4, 0.7, -0.5).normalize(),
    }
}
pub(super) fn projected_error(cam: &Camera, error: f32) -> f32 {
    let nearest = cam.eye.length() - 9.;
    if nearest <= 0.1 {
        return f32::INFINITY;
    }
    let focal = cam.size[1] as f32 / (2. * 30f32.to_radians().tan());
    2. * focal * error / nearest * (1. + 9. / nearest)
}
// Exact perspective displacement numerator is affine in p. Its norm reaches
// its maximum over a box at a corner. Both endpoints stay in the same box,
// so the product of their depths is at least z_min squared.
pub(super) fn precise_error(cam: &Camera, error: f32, bounds: [[f32; 3]; 2]) -> f32 {
    let a = cam.view.transform_vector3(Vec3::Y);
    let mut nearest = f32::INFINITY;
    let mut numerator = 0f32;
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                let p =
                    cam.view
                        .transform_point3(Vec3::new(bounds[x][0], bounds[y][1], bounds[z][2]));
                nearest = nearest.min(-p.z);
                numerator = numerator.max((a.truncate() * p.z - p.truncate() * a.z).length());
            }
        }
    }
    if nearest <= 0.1 {
        return f32::INFINITY;
    }
    let focal = cam.size[1] as f32 / (2. * 30f32.to_radians().tan());
    focal * error * numerator / (nearest * nearest)
}
pub(super) fn selected(
    levels: &[build::Level],
    cam: &Camera,
    bounds: [[f32; 3]; 2],
    old: bool,
) -> usize {
    let factor = if old {
        projected_error(cam, 1.)
    } else {
        precise_error(cam, 1., bounds)
    };
    levels
        .iter()
        .position(|l| l.position_error * factor <= 0.15)
        .unwrap_or(levels.len() - 1)
}
pub(super) fn selection(a: &Asset, cam: &Camera, mode: u32) -> ([(u32, u32); 64], usize, u32) {
    if mode == 5 {
        return hybrid_choice(a, cam, true).0;
    }
    if mode == 4 {
        let l = &a.sealed[selected(&a.sealed, cam, a.bounds, false)];
        let mut draws = [(0, 0); 64];
        draws[0] = (l.first, l.count);
        return (draws, 1, l.count / 3);
    }
    let mut draws = [(0, 0); 64];
    if mode != 3 {
        let level = if mode == 0 {
            a.levels.len() - 1
        } else {
            selected(&a.levels, cam, a.bounds, mode == 1)
        };
        let l = &a.levels[level];
        draws[0] = (l.first, l.count);
        return (draws, 1, l.count / 3);
    }
    let mut triangles = 0;
    let mut n = 0;
    for r in &a.regions {
        let l = &r.levels[selected(&r.levels, cam, r.bounds, false)];
        if n > 0 && draws[n - 1].0 + draws[n - 1].1 == l.first {
            draws[n - 1].1 += l.count;
        } else {
            draws[n] = (l.first, l.count);
            n += 1;
        }
        triangles += l.count / 3;
    }
    (draws, n, triangles)
}
pub(super) type DrawSelection = ([(u32, u32); 64], usize, u32);
const DRAW_WEIGHT: u64 = 1024;
fn cost(triangles: u32, draws: usize) -> u64 {
    triangles as u64 + draws as u64 * DRAW_WEIGHT
}
pub(super) fn hybrid_choice(a: &Asset, cam: &Camera, prune: bool) -> (DrawSelection, u32, usize) {
    let global = selection(a, cam, 4);
    let limit = cost(global.2, 1);
    let mut remaining = a.regional_floor;
    if prune && limit <= cost(remaining, 1) {
        return (global, 4, 0);
    }
    let mut draws = [(0, 0); 64];
    let (mut n, mut triangles) = (0, 0);
    for (i, r) in a.regions.iter().enumerate() {
        let l = &r.levels[selected(&r.levels, cam, r.bounds, false)];
        remaining -= r.floor;
        triangles += l.count / 3;
        if n > 0 && draws[n - 1].0 + draws[n - 1].1 == l.first {
            draws[n - 1].1 += l.count;
        } else {
            draws[n] = (l.first, l.count);
            n += 1;
        }
        if prune && limit <= cost(triangles + remaining, n) {
            return (global, 4, i + 1);
        }
    }
    if limit <= cost(triangles, n) {
        (global, 4, a.regions.len())
    } else {
        ((draws, n, triangles), 3, a.regions.len())
    }
}
fn mode_name(mode: u32) -> &'static str {
    match mode {
        0 => "full",
        1 => "global_old",
        2 => "global_precise",
        3 => "regional_precise",
        4 => "global_sealed",
        5 => "hybrid",
        6 => "calibrated",
        _ => panic!("mode"),
    }
}
impl Renderer {
    pub(super) fn read_field_depth(&self) -> Result<Vec<f32>> {
        let t = self.target.as_ref().unwrap();
        let [w, h] = self.size;
        let read = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("diagnostic depth readback"),
            size: w as u64 * h as u64 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        copy_readback(&mut e, &t._depth, &read, w, h, 4);
        self.wait(self.gpu.queue.submit([e.finish()]))?;
        self.gpu.map(&read)?;
        let bytes = read.slice(..).get_mapped_range();
        let values = bytemuck::cast_slice::<u8, f32>(&bytes).to_vec();
        ensure!(
            values
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
            "invalid field depth"
        );
        drop(bytes);
        read.unmap();
        Ok(values)
    }
    pub(super) fn load_tiled_field(&mut self) {
        self.geometric_field = Some(geometric_field::Pipeline::new_tiled(
            &self.gpu,
            &self.layout,
        ));
    }
    pub(super) fn load_hierarchical_field(&mut self) {
        self.geometric_field = Some(geometric_field::Pipeline::new_hierarchical(
            &self.gpu,
            &self.layout,
        ));
    }
    pub(super) fn load_geometric_field(&mut self) {
        self.geometric_field = Some(geometric_field::Pipeline::new(&self.gpu, &self.layout));
    }
    pub(super) fn upload_field(&self, u: &mut Uploaded, a: &Asset) -> Result<()> {
        u.field = Some(
            self.geometric_field
                .as_ref()
                .unwrap()
                .upload(&self.gpu, &a.id)?,
        );
        Ok(())
    }
    pub(super) fn field_class(&self, a: &Asset, c: &Camera) -> usize {
        let triangles = selection(a, c, 5).2 as f64;
        let area = (cost::area(a, c) * 1e6).max(1.);
        let ratio = triangles / area;
        if ratio < 0.05 {
            0
        } else if ratio < 0.25 {
            1
        } else if ratio < 1. {
            2
        } else {
            3
        }
    }
    pub(super) fn field_selected(&self, u: &Uploaded, a: &Asset, c: &Camera, force: bool) -> bool {
        let Some(p) = &self.geometric_field else {
            return false;
        };
        if !force && p.enabled == 0 {
            return false;
        }
        u.field.as_ref().is_some_and(|d| {
            d.certificate.as_ref().map_or_else(
                || d.field.eligible(c),
                |certificate| certificate.eligible(&d.field, c),
            )
        }) && (force || p.enabled & (1 << self.field_class(a, c)) != 0)
    }

    pub async fn new() -> Result<Self> {
        Self::create(false).await
    }
    pub async fn new_diagnostic() -> Result<Self> {
        Self::create(true).await
    }
    pub async fn new_automatic() -> Result<Self> {
        let mut renderer = Self::create(true).await?;
        // The automatic experiment needs no normal/color intermediate target.
        renderer.deferred = None;
        renderer.diagnostics.clear();
        Ok(renderer)
    }
    async fn create(diagnostic: bool) -> Result<Self> {
        let gpu = Gpu::new().await?;
        let d = &gpu.device;
        let uniform = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 112,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle_scaling.wgsl").into()),
        });
        let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3];
        let make_pipeline = |entry| {
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 24,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &attrs,
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
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
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let pipeline = make_pipeline("fragment");
        let diagnostics = if diagnostic {
            [
                "flat_fragment",
                "normal_fragment",
                "native_fragment",
                "branchless_fragment",
                "gbuffer_fragment",
                "native_gbuffer_fragment",
            ]
            .into_iter()
            .map(make_pipeline)
            .collect()
        } else {
            Vec::new()
        };
        let lighting_pipeline = |source: &str, sample_type: wgpu::TextureSampleType| {
            let layout2 = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
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
            let inverse = d.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    format!("{}\n{}", include_str!("triangle_scaling.wgsl"), source).into(),
                ),
            });
            let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&layout, &layout2],
                push_constant_ranges: &[],
            });
            let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shade visible pixels"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("fullscreen"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("shade_visible"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
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
            (pipeline, layout2, inverse)
        };
        let deferred = diagnostic.then(|| {
            lighting_pipeline(
                include_str!("scaling_deferred.wgsl"),
                wgpu::TextureSampleType::Float { filterable: false },
            )
        });
        let depth_shade = diagnostic.then(|| {
            lighting_pipeline(
                include_str!("scaling_depth.wgsl"),
                wgpu::TextureSampleType::Depth,
            )
        });
        let make_depth = |packed: bool| {
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("depth only geometry"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(if packed { "position_vertex" } else { "vertex" }),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: if packed { 12 } else { 24 },
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: if packed { &attrs[..1] } else { &attrs },
                    }],
                },
                fragment: None,
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let depth_pipeline = diagnostic.then(|| make_depth(false));
        let packed_pipeline = diagnostic.then(|| make_depth(true));
        Ok(Self {
            depth_shade,
            depth_pipeline,
            packed_pipeline,
            deferred,
            diagnostics,
            cost_model: None,
            shading_model: None,
            material_symbol: None,
            geometric_field: None,
            stage_queries: false,
            density_sources: Vec::new(),
            gpu,
            uniform,
            layout,
            pipeline,
            target: None,
            size: [0, 0],
        })
    }
    pub(super) fn load_material_symbol(&mut self) -> Result<()> {
        self.material_symbol = Some(super::material_symbol::Pipelines::new(
            &self.gpu,
            &self.layout,
            &self.depth_shade.as_ref().unwrap().1,
        )?);
        Ok(())
    }
    pub fn upload(&self, a: &Asset) -> Result<Uploaded> {
        self.upload_from(a, ROOT)
    }
    pub fn upload_from(&self, a: &Asset, root: &str) -> Result<Uploaded> {
        let root = Path::new(root);
        let d = &self.gpu.device;
        let v = fs::read(root.join(format!("{}.vertices", a.id)))?;
        let i = fs::read(root.join(format!("{}.indices", a.id)))?;
        let n = fs::read(root.join(format!("{}.normals", a.id)))?;
        ensure!(
            v.len() as u64 + i.len() as u64 + n.len() as u64 == a.bytes,
            "stale asset bytes"
        );
        let vertex = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &v,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &i,
            usage: wgpu::BufferUsages::INDEX,
        });
        let normal = texture(
            d,
            a.grid[0] + 1,
            a.grid[1] + 1,
            1,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        self.gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &normal,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &n,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((a.grid[0] + 1) * 16),
                rows_per_image: Some(a.grid[1] + 1),
            },
            wgpu::Extent3d {
                width: a.grid[0] + 1,
                height: a.grid[1] + 1,
                depth_or_array_layers: 1,
            },
        );
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &normal.create_view(&Default::default()),
                    ),
                },
            ],
        });
        // Regional source levels are contiguous in the bake. Only those exact
        // triangles can use native vertex normals without an approximation.
        let native_levels = a
            .regions
            .iter()
            .map(|r| r.levels.last().unwrap())
            .collect::<Vec<_>>();
        ensure!(
            native_levels
                .iter()
                .all(|l| l.stride == 1 && l.position_error == 0.),
            "native source level"
        );
        ensure!(
            native_levels
                .windows(2)
                .all(|w| w[0].first + w[0].count == w[1].first),
            "native ranges contiguous"
        );
        let native_range = native_levels.first().unwrap().first
            ..native_levels.last().map(|l| l.first + l.count).unwrap();
        Ok(Uploaded {
            native_range,
            reversed: None,
            packed: Vec::new(),
            vertex,
            index,
            group,
            _normal: normal,
            field: None,
        })
    }
    pub fn upload_packed(&self, u: &mut Uploaded, a: &Asset, root: &str) -> Result<()> {
        for name in ["packed", "spatial"] {
            let v = fs::read(Path::new(root).join(format!("{}.{}.positions", a.id, name)))?;
            let i = fs::read(Path::new(root).join(format!("{}.{}.indices", a.id, name)))?;
            ensure!(
                !v.is_empty() && v.len() % 12 == 0 && i.len() % 12 == 0,
                "packed buffer alignment"
            );
            ensure!(
                bytemuck::cast_slice::<u8, u32>(&i)
                    .iter()
                    .all(|&id| (id as usize) < v.len() / 12),
                "packed index bounds"
            );
            u.packed.push((
                self.gpu
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(name),
                        contents: &v,
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                self.gpu
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(name),
                        contents: &i,
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            ));
        }
        Ok(())
    }
    pub fn reverse_order(&self, u: &mut Uploaded, a: &Asset, root: &str) -> Result<()> {
        let bytes = fs::read(Path::new(root).join(format!("{}.indices", a.id)))?;
        let mut indices = bytemuck::cast_slice::<u8, u32>(&bytes).to_vec();
        let mut ranges = a
            .levels
            .iter()
            .chain(&a.sealed)
            .chain(a.regions.iter().flat_map(|r| &r.levels))
            .map(|l| (l.first as usize, l.count as usize))
            .collect::<Vec<_>>();
        ranges.sort_unstable();
        ranges.dedup();
        ensure!(
            ranges.windows(2).all(|w| w[0].0 + w[0].1 <= w[1].0),
            "overlapping index ranges"
        );
        for (first, count) in ranges {
            reverse_triangles(&mut indices[first..first + count]);
        }
        u.reversed = Some(
            self.gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("reversed triangle order diagnostic"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        );
        Ok(())
    }
    pub(super) fn physical_size(&self) -> [u32; 2] {
        self.size
    }
    pub fn resize(&mut self, size: [u32; 2]) {
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
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | if self.geometric_field.is_some() {
                    wgpu::TextureUsages::COPY_SRC
                } else {
                    wgpu::TextureUsages::empty()
                }
                | if self.depth_shade.is_some() {
                    wgpu::TextureUsages::TEXTURE_BINDING
                } else {
                    wgpu::TextureUsages::empty()
                },
        );
        let surface = self.deferred.as_ref().map(|(_, layout, inverse)| {
            let tex = texture(
                d,
                w,
                h,
                1,
                wgpu::TextureFormat::Rgba32Float,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            );
            let view = tex.create_view(&Default::default());
            let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: inverse.as_entire_binding(),
                    },
                ],
            });
            (view, group)
        });
        let depth_view = depth.create_view(&Default::default());
        let depth_group = self.depth_shade.as_ref().map(|(_, layout, inverse)| {
            d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: inverse.as_entire_binding(),
                    },
                ],
            })
        });
        self.target = Some(Target {
            depth_group,
            surface,
            view: color.create_view(&Default::default()),
            color,
            depth: depth.create_view(&Default::default()),
            _depth: depth,
            read: d.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: w as u64 * h as u64 * 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
        });
        self.size = size;
    }
    pub(super) fn encode(
        &self,
        e: &mut wgpu::CommandEncoder,
        u: &Uploaded,
        a: &Asset,
        cam: &Camera,
        mode: u32,
        q: Option<u32>,
    ) -> usize {
        let symbol_requested = matches!(mode, 325 | 335 | 345 | 355 | 365 | 375 | 385 | 395 | 405);
        let flat = matches!(mode, 325 | 375 | 385)
            && self
                .geometric_field
                .as_ref()
                .is_some_and(|s| s.hierarchical);
        let selected = if flat && mode == 325 {
            u.field.as_ref().is_some_and(|d| d.field.eligible(cam))
        } else {
            symbol_requested
                && self.field_selected(u, a, cam, matches!(mode, 325 | 345 | 365 | 375 | 385 | 395))
        };
        if symbol_requested && selected {
            let t = self.target.as_ref().unwrap();
            let data = u.field.as_ref().unwrap();
            let symbol = self.geometric_field.as_ref().unwrap();
            symbol.prepare(&self.gpu, data, cam);
            let rect = tiled_field::scissor(a, cam);
            let use_tiles = matches!(mode, 395 | 405);
            if use_tiles {
                symbol
                    .tiles
                    .as_ref()
                    .unwrap()
                    .encode(&self.gpu, e, data, cam, rect, q);
            }
            self.gpu.queue.write_buffer(
                &self.uniform,
                0,
                bytemuck::bytes_of(&Uniforms {
                    matrix: cam.matrix.to_cols_array_2d(),
                    eye: cam.eye.extend(0.).to_array(),
                    light: cam.light.extend(0.).to_array(),
                    settings: [3, 0, 0, 0],
                }),
            );
            let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("exact heightfield symbol"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.07,
                            b: 0.11,
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
                timestamp_writes: q.and_then(|base| {
                    self.gpu
                        .query
                        .as_ref()
                        .map(|query_set| wgpu::RenderPassTimestampWrites {
                            query_set,
                            beginning_of_pass_write_index: if use_tiles {
                                None
                            } else {
                                Some(base)
                            },
                            end_of_pass_write_index: Some(base + 1),
                        })
                }),
                occlusion_query_set: None,
            });
            pass.set_pipeline(match mode {
                395 | 405 => symbol.tiled_pipeline.as_ref().unwrap(),
                365 => symbol.diagnostic.as_ref().unwrap(),
                375 => symbol.flat_diagnostic.as_ref().unwrap(),
                325 | 385 if flat => symbol.flat_pipeline.as_ref().unwrap(),
                _ => &symbol.pipeline,
            });
            pass.set_bind_group(0, &u.group, &[]);
            pass.set_bind_group(1, &data.group, &[]);
            let [x, y, w, h] = rect;
            if w > 0 && h > 0 {
                pass.set_scissor_rect(x, y, w, h);
                pass.draw(0..3, 0..1);
            }
            return 1;
        }
        let mode = if symbol_requested { 135 } else { mode };
        let source_id = mode / 1000;
        let mode = mode % 1000;
        let symbol_mode = mode / 10;
        let mode = if symbol_mode == 30 || symbol_mode == 31 {
            135
        } else {
            mode
        };
        let experiment = mode / 10;
        let geometry_only = experiment == 18;
        let material_only = experiment == 28;
        let mut variant = if geometry_only || material_only {
            8
        } else {
            experiment
        };
        let mode = mode % 10;
        let (draws, draw_count, triangles) = if mode == 6 {
            cost::choose(
                a,
                cam,
                self.cost_model.as_ref().expect("calibrated model"),
                true,
            )
            .0
        } else {
            selection(a, cam, mode)
        };
        let dense = if experiment == 14 || experiment == 15 {
            let source = &self.density_sources
                [source_id.checked_sub(1).expect("density source id") as usize];
            variant = 8;
            if experiment == 14 || !density::eligible(Some(source.error_world), cam, a.bounds) {
                Some(source)
            } else {
                None
            }
        } else {
            None
        };
        let native_source = mode >= 4
            && a.sealed
                .last()
                .is_some_and(|l| draw_count == 1 && draws[0] == (l.first, l.count));
        let packed_choice = match variant {
            10 => Some(0),
            11 | 13 => Some(1),
            _ => None,
        };
        if variant == 10 || variant == 11 {
            variant = 8;
        }
        if variant == 9 || variant == 13 {
            variant = self.shading_model.as_ref().map_or(0, |model| {
                if model.depth(automatic::features(a, cam, triangles, native_source)) {
                    8
                } else {
                    0
                }
            });
        }
        let use_symbol = symbol_mode == 30
            || (symbol_mode == 31
                && self.material_symbol.as_ref().is_some_and(|s| {
                    // Avoid projection work when compilation/calibration rejected every class.
                    s.enabled != 0
                        && s.allows(
                            variant == 8,
                            cost::area(a, cam) * 1e6 / (cam.size[0] as f64 * cam.size[1] as f64),
                        )
                }));
        let t = self.target.as_ref().unwrap();
        self.gpu.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&Uniforms {
                matrix: cam.matrix.to_cols_array_2d(),
                eye: cam.eye.extend(0.).to_array(),
                light: cam.light.extend(0.).to_array(),
                settings: [if mode == 0 || native_source { 0 } else { 3 }, 0, 0, 0],
            }),
        );
        if !material_only {
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view: if variant == 7 {
                    &t.surface.as_ref().unwrap().0
                } else {
                    &t.view
                },
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.04,
                        g: 0.07,
                        b: 0.11,
                        a: 0.,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: if variant == 8 { &[] } else { &attachments },
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: q.and_then(|base| {
                    self.gpu
                        .query
                        .as_ref()
                        .map(|query_set| wgpu::RenderPassTimestampWrites {
                            query_set,
                            beginning_of_pass_write_index: Some(base),
                            end_of_pass_write_index: if variant >= 7
                                && !self.stage_queries
                                && !geometry_only
                            {
                                None
                            } else {
                                Some(base + 1)
                            },
                        })
                }),
                occlusion_query_set: None,
            });
            pass.set_pipeline(if variant == 8 {
                if packed_choice.is_some() || dense.is_some() {
                    self.packed_pipeline.as_ref().unwrap()
                } else {
                    self.depth_pipeline.as_ref().unwrap()
                }
            } else if use_symbol {
                &self.material_symbol.as_ref().unwrap().forward
            } else if variant == 0 || variant == 5 {
                &self.pipeline
            } else {
                &self.diagnostics[if variant == 7 {
                    4
                } else if variant == 6 {
                    3
                } else {
                    variant as usize - 1
                }]
            });
            pass.set_bind_group(0, &u.group, &[]);
            if use_symbol && variant != 8 {
                pass.set_bind_group(1, &self.material_symbol.as_ref().unwrap().group, &[]);
            }
            if let Some(source) = dense {
                for chunk in &source.chunks {
                    pass.set_vertex_buffer(0, chunk.vertex.slice(..));
                    pass.set_index_buffer(chunk.index.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..chunk.count, 0, 0..1);
                }
            } else {
                let packed = if variant == 8 {
                    packed_choice.map(|index| &u.packed[index])
                } else {
                    None
                };
                pass.set_vertex_buffer(0, packed.map_or(&u.vertex, |p| &p.0).slice(..));
                pass.set_index_buffer(
                    if let Some(p) = packed {
                        p.1.slice(..)
                    } else if variant == 5 {
                        u.reversed.as_ref().expect("reversed indices").slice(..)
                    } else {
                        u.index.slice(..)
                    },
                    wgpu::IndexFormat::Uint32,
                );
                for &(first, count) in &draws[..draw_count] {
                    if variant == 6 || variant == 7 {
                        let native =
                            first >= u.native_range.start && first + count <= u.native_range.end;
                        pass.set_pipeline(
                            &self.diagnostics[if variant == 7 {
                                if native { 5 } else { 4 }
                            } else if native {
                                2
                            } else {
                                3
                            }],
                        );
                    }
                    pass.draw_indexed(first..first + count, 0, 0..1);
                }
            }
            drop(pass);
        }
        if variant >= 7 && !geometry_only {
            let (pipeline, _, inverse) = if variant == 8 {
                self.depth_shade.as_ref().unwrap()
            } else {
                self.deferred.as_ref().unwrap()
            };
            self.gpu.queue.write_buffer(
                inverse,
                0,
                bytemuck::cast_slice(&cam.inverse.to_cols_array()),
            );
            let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shade visible pixels"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: q.and_then(|base| {
                    self.gpu
                        .query
                        .as_ref()
                        .map(|query_set| wgpu::RenderPassTimestampWrites {
                            query_set,
                            beginning_of_pass_write_index: if material_only {
                                Some(base)
                            } else if self.stage_queries {
                                Some(base + 2)
                            } else {
                                None
                            },
                            end_of_pass_write_index: Some(
                                base + if self.stage_queries { 3 } else { 1 },
                            ),
                        })
                }),
                occlusion_query_set: None,
            });
            pass.set_pipeline(if use_symbol {
                &self.material_symbol.as_ref().unwrap().depth
            } else {
                pipeline
            });
            pass.set_bind_group(0, &u.group, &[]);
            if use_symbol {
                pass.set_bind_group(2, &self.material_symbol.as_ref().unwrap().group, &[]);
            }
            pass.set_bind_group(
                1,
                if variant == 8 {
                    t.depth_group.as_ref().unwrap()
                } else {
                    &t.surface.as_ref().unwrap().1
                },
                &[],
            );
            pass.draw(0..3, 0..1);
        }
        dense.map_or(triangles as usize, |s| s.triangles as usize)
    }
    pub(super) fn wait(&self, id: wgpu::SubmissionIndex) -> Result<()> {
        self.gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(id),
            timeout: Some(Duration::from_secs(60)),
        })?;
        Ok(())
    }
    pub(super) fn read_field_color(&self, size: [u32; 2]) -> Result<Vec<[f32; 4]>> {
        let t = self.target.as_ref().unwrap();
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        copy_readback(&mut e, &t.color, &t.read, size[0], size[1], 16);
        self.wait(self.gpu.queue.submit([e.finish()]))?;
        self.gpu.map(&t.read)?;
        let bytes = t.read.slice(..).get_mapped_range();
        let pixels = bytemuck::cast_slice::<u8, [f32; 4]>(&bytes).to_vec();
        drop(bytes);
        t.read.unmap();
        ensure!(
            pixels.iter().flatten().all(|x| x.is_finite()),
            "nonfinite pixels"
        );
        Ok(pixels)
    }
    pub fn capture(
        &mut self,
        u: &Uploaded,
        a: &Asset,
        cam: &Camera,
        mode: u32,
    ) -> Result<(Vec<[f32; 4]>, usize)> {
        self.resize(cam.size);
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        let level = self.encode(&mut e, u, a, cam, mode, None);
        let t = self.target.as_ref().unwrap();
        copy_readback(&mut e, &t.color, &t.read, cam.size[0], cam.size[1], 16);
        self.wait(self.gpu.queue.submit([e.finish()]))?;
        self.gpu.map(&t.read)?;
        let bytes = t.read.slice(..).get_mapped_range();
        let pixels = bytemuck::cast_slice::<u8, [f32; 4]>(&bytes).to_vec();
        drop(bytes);
        t.read.unmap();
        ensure!(
            pixels.iter().flatten().all(|x| x.is_finite()),
            "nonfinite pixels"
        );
        Ok((pixels, level))
    }
    // Sustained batches: no timestamp instrumentation in end-to-end measurements.
    pub fn profile_stages(
        &mut self,
        inputs: &[(&Uploaded, &Asset)],
        distance: f32,
        size: [u32; 2],
    ) -> Result<serde_json::Value> {
        ensure!(self.gpu.query.is_some(), "stage timestamps unavailable");
        self.resize(size);
        self.stage_queries = true;
        let result = (|| -> Result<serde_json::Value> {
            let cam = camera_at(distance, size);
            let mut samples = vec![Vec::new(); inputs.len()];
            for round in 0..8 {
                for offset in 0..inputs.len() {
                    let d = (offset + round) % inputs.len();
                    let (u, a) = inputs[d];
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    for _ in 0..32 {
                        self.encode(&mut e, u, a, &cam, 85, None);
                    }
                    self.wait(self.gpu.queue.submit([e.finish()]))?;
                    let mut last = None;
                    for i in 0..16 {
                        let mut e = self.gpu.device.create_command_encoder(&Default::default());
                        self.encode(&mut e, u, a, &cam, 85, Some(i * 4));
                        last = Some(self.gpu.queue.submit([e.finish()]));
                    }
                    self.wait(last.unwrap())?;
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    e.resolve_query_set(
                        self.gpu.query.as_ref().unwrap(),
                        0..64,
                        &self.gpu.query_resolve,
                        0,
                    );
                    e.copy_buffer_to_buffer(
                        &self.gpu.query_resolve,
                        0,
                        &self.gpu.query_readback,
                        0,
                        512,
                    );
                    self.wait(self.gpu.queue.submit([e.finish()]))?;
                    self.gpu.map(&self.gpu.query_readback)?;
                    let bytes = self.gpu.query_readback.slice(..).get_mapped_range();
                    let stamps = bytemuck::cast_slice::<u8, u64>(&bytes);
                    let mut sums = [0f64; 4];
                    for t in stamps.chunks_exact(4) {
                        ensure!(
                            t[1] >= t[0] && t[3] >= t[2] && t[3] >= t[0],
                            "invalid stage interval {t:?}"
                        );
                        sums[0] += (t[1] - t[0]) as f64;
                        sums[1] += t[2] as f64 - t[1] as f64;
                        sums[2] += (t[3] - t[2]) as f64;
                        sums[3] += (t[3] - t[0]) as f64;
                    }
                    let ms = sums.map(|v| v * self.gpu.timestamp_period_ns.unwrap() as f64 / 16e6);
                    let raw = stamps.to_vec();
                    drop(bytes);
                    self.gpu.query_readback.unmap();
                    samples[d].push(json!({"geometry_ms":ms[0],"between_ms":ms[1],"material_ms":ms[2],"total_ms":ms[3],"raw_stamps":raw}));
                }
            }
            Ok(json!(samples))
        })();
        self.stage_queries = false;
        result
    }
    pub fn bench_cost(
        &self,
        inputs: &[(&Uploaded, &Asset)],
        distance: f32,
        size: [u32; 2],
        modes: &[u32],
    ) -> Result<serde_json::Value> {
        let count = modes.len();
        let slots = inputs.len() * count * 2;
        let mut total = vec![Vec::new(); slots];
        let mut cpu = total.clone();
        let mut gpu = total.clone();
        let cam = camera_at(distance, size);
        for &(u, a) in inputs {
            for &m in modes {
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                for _ in 0..32 {
                    self.encode(&mut e, u, a, &cam, m, None);
                }
                self.wait(self.gpu.queue.submit([e.finish()]))?;
            }
        }
        for round in 0..8 {
            // Both direction and starting density rotate; paired controls remain explicit.
            for offset in 0..slots {
                let s = if round % 2 == 0 {
                    (offset + round * count) % slots
                } else {
                    (slots - 1 - offset + round * count) % slots
                };
                let (u, a) = inputs[(s % (inputs.len() * count)) / count];
                let mode = modes[s % count];
                if mode / 10 == 28 {
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.encode(&mut e, u, a, &cam, 85, None);
                    self.wait(self.gpu.queue.submit([e.finish()]))?;
                }
                let start = Instant::now();
                let mut last = None;
                for _ in 0..128 {
                    let cam = camera_at(distance, size);
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.encode(&mut e, u, a, &cam, mode, None);
                    last = Some(self.gpu.queue.submit([e.finish()]));
                }
                cpu[s].push(start.elapsed().as_secs_f64() * 1000. / 128.);
                self.wait(last.unwrap())?;
                total[s].push(start.elapsed().as_secs_f64() * 1000. / 128.);
                if let Some(q) = &self.gpu.query {
                    let mut last = None;
                    for i in 0..32 {
                        let mut e = self.gpu.device.create_command_encoder(&Default::default());
                        self.encode(&mut e, u, a, &cam, mode, Some(i * 2));
                        last = Some(self.gpu.queue.submit([e.finish()]));
                    }
                    self.wait(last.unwrap())?;
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    e.resolve_query_set(q, 0..64, &self.gpu.query_resolve, 0);
                    e.copy_buffer_to_buffer(
                        &self.gpu.query_resolve,
                        0,
                        &self.gpu.query_readback,
                        0,
                        512,
                    );
                    self.wait(self.gpu.queue.submit([e.finish()]))?;
                    self.gpu.map(&self.gpu.query_readback)?;
                    let bytes = self.gpu.query_readback.slice(..).get_mapped_range();
                    let stamps = bytemuck::cast_slice::<u8, u64>(&bytes);
                    let mut sum = 0u64;
                    for i in 0..32 {
                        ensure!(
                            stamps[i * 2 + 1] >= stamps[i * 2],
                            "GPU clock at {i}: {} -> {}",
                            stamps[i * 2],
                            stamps[i * 2 + 1]
                        );
                        sum += stamps[i * 2 + 1] - stamps[i * 2];
                    }
                    gpu[s].push(sum as f64 * self.gpu.timestamp_period_ns.unwrap() as f64 / 32e6);
                    drop(bytes);
                    self.gpu.query_readback.unmap();
                }
            }
        }
        Ok(json!(
            (0..slots)
                .map(
                    |s| json!({"mode":mode_name(modes[s%count]%10),"variant":modes[s%count]/10,
            "gpu_ms":stats(&mut gpu[s].clone()),"throughput_ms":stats(&mut total[s].clone()),
            "cpu_ms":stats(&mut cpu[s].clone()),"samples_ms":total[s],"gpu_samples_ms":gpu[s]})
                )
                .collect::<Vec<_>>()
        ))
    }

    pub fn bench_set(
        &self,
        inputs: &[(&Uploaded, &Asset)],
        frame: usize,
        size: [u32; 2],
        batch: usize,
    ) -> Result<serde_json::Value> {
        self.bench_modes(inputs, frame, size, batch, &[0, 1, 2, 3])
    }
    pub fn bench_modes(
        &self,
        inputs: &[(&Uploaded, &Asset)],
        frame: usize,
        size: [u32; 2],
        batch: usize,
        modes: &[u32],
    ) -> Result<serde_json::Value> {
        let count = modes.len();
        let slots = inputs.len() * count * 2;
        ensure!(slots * 12 * 2 <= 1024, "query capacity");
        for _ in 0..12 {
            for entry in 0..inputs.len() * count {
                let (u, a) = inputs[entry / count];
                let mode = modes[entry % count];
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.encode(&mut e, u, a, &pose(frame, size), mode, None);
                self.wait(self.gpu.queue.submit([e.finish()]))?;
            }
        }
        let mut gpu = vec![Vec::new(); slots];
        let mut serial = gpu.clone();
        let mut throughput = gpu.clone();
        let mut cpu = gpu.clone();
        for r in 0..12 {
            for offset in 0..slots {
                let s = (r * 7 + offset) % slots;
                let (u, a) = inputs[(s % (inputs.len() * count)) / count];
                let start = Instant::now();
                let cam = pose(frame, size);
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                self.encode(
                    &mut e,
                    u,
                    a,
                    &cam,
                    modes[s % count],
                    Some(((r * slots + s) * 2) as u32),
                );
                cpu[s].push(start.elapsed().as_secs_f64() * 1000.);
                self.wait(self.gpu.queue.submit([e.finish()]))?;
                serial[s].push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        if let Some(q) = &self.gpu.query {
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(q, 0..(slots * 12 * 2) as u32, &self.gpu.query_resolve, 0);
            e.copy_buffer_to_buffer(
                &self.gpu.query_resolve,
                0,
                &self.gpu.query_readback,
                0,
                8192,
            );
            self.wait(self.gpu.queue.submit([e.finish()]))?;
            self.gpu.map(&self.gpu.query_readback)?;
            let bytes = self.gpu.query_readback.slice(..).get_mapped_range();
            let stamps = bytemuck::cast_slice::<u8, u64>(&bytes);
            for r in 0..12 {
                for (s, g) in gpu.iter_mut().enumerate() {
                    let b = (r * slots + s) * 2;
                    ensure!(stamps[b + 1] >= stamps[b], "GPU clock");
                    g.push(
                        (stamps[b + 1] - stamps[b]) as f64
                            * self.gpu.timestamp_period_ns.unwrap() as f64
                            / 1e6,
                    );
                }
            }
            drop(bytes);
            self.gpu.query_readback.unmap();
        }
        for r in 0..12 {
            for offset in 0..slots {
                let s = (r * 7 + offset) % slots;
                let (u, a) = inputs[(s % (inputs.len() * count)) / count];
                let start = Instant::now();
                let mut last = None;
                for _ in 0..batch {
                    let mut e = self.gpu.device.create_command_encoder(&Default::default());
                    self.encode(&mut e, u, a, &pose(frame, size), modes[s % count], None);
                    last = Some(self.gpu.queue.submit([e.finish()]));
                }
                self.wait(last.unwrap())?;
                throughput[s].push(start.elapsed().as_secs_f64() * 1000. / batch as f64);
            }
        }
        Ok(json!((0..slots).map(|s|json!({"mode":mode_name(modes[s%count]%10),"variant":modes[s%count]/10,"gpu_ms":stats(&mut gpu[s]),"serial_ms":stats(&mut serial[s]),"throughput_ms":stats(&mut throughput[s]),"cpu_ms":stats(&mut cpu[s])})).collect::<Vec<_>>()))
    }
}

#[cfg(test)]
mod hybrid_tests {
    use super::*;
    #[test]
    fn pruned_choice_equals_exhaustive_cost_and_handles_ties() {
        let mut saw = [false; 3];
        for global_triangles in [16, 2048, 4096, 16384, 65536, 999999] {
            let regions=(0..64).map(|i|json!({"floor":16,"bounds":[[-6.,-0.7,-6.],[6.,0.7,6.]],"levels":[
                {"first":i*20000,"count":48,"position_error":0.01+(i%7)as f32*0.01,"normal_error":0.,"stride":4},
                {"first":i*20000+48,"count":3072,"position_error":0.001,"normal_error":0.,"stride":2},
                {"first":i*20000+3120,"count":12288,"position_error":0.,"normal_error":0.,"stride":1}]})).collect::<Vec<_>>();
            let a:Asset=serde_json::from_value(json!({"id":"test","family":1,"factor":1,"grid":[128,128],"levels":[],"sealed":[{"first":2000000,"count":global_triangles*3,"position_error":0.,"normal_error":0.,"stride":1}],"regions":regions,"regional_floor":1024,"bounds":[[-6.,-0.7,-6.],[6.,0.7,6.]],"bytes":0,"build_ms":0.})).unwrap();
            for distance in [8., 14., 23., 45., 79., 112., 224.] {
                for size in RESOLUTIONS {
                    let cam = camera_at(distance, size);
                    let pruned = hybrid_choice(&a, &cam, true);
                    let exhaustive = hybrid_choice(&a, &cam, false);
                    assert_eq!(pruned.0, exhaustive.0);
                    assert_eq!(pruned.1, exhaustive.1);
                    saw[if pruned.2 == 0 {
                        0
                    } else if pruned.2 == 64 {
                        2
                    } else {
                        1
                    }] = true;
                    for coefficients in [
                        [-0.001, 0.8, 0.0003, 0.02, 0.003, 0.04],
                        [0.01, 0.2, 0.0001, 0.01, -0.002, 0.02],
                    ] {
                        let model = cost::Model { coefficients };
                        let pruned = cost::choose(&a, &cam, &model, true);
                        let exhaustive = cost::choose(&a, &cam, &model, false);
                        assert_eq!(pruned.0, exhaustive.0);
                        assert_eq!(pruned.1, exhaustive.1);
                        assert_eq!(
                            pruned.1,
                            if model.predict(cost::features(&a, &cam)) <= 0. {
                                4
                            } else {
                                3
                            }
                        );
                    }
                    let regional = selection(&a, &cam, 3);
                    let global = selection(&a, &cam, 4);
                    assert_eq!(
                        cost(pruned.0.2, pruned.0.1),
                        cost(global.2, global.1).min(cost(regional.2, regional.1))
                    );
                }
            }
        }
        assert!(saw.into_iter().all(|x| x));
        assert_eq!(cost(1024, 2), cost(2048, 1));
    }
}

// Keep winding and vertex interpolation order; only primitive submission order changes.
fn reverse_triangles(indices: &mut [u32]) {
    assert_eq!(indices.len() % 3, 0);
    let n = indices.len() / 3;
    for i in 0..n / 2 {
        for k in 0..3 {
            indices.swap(i * 3 + k, (n - 1 - i) * 3 + k);
        }
    }
}
#[cfg(test)]
mod order_tests {
    use super::*;
    #[test]
    fn reversal_preserves_each_oriented_triangle() {
        for n in 0..100 {
            let original = (0..n * 3).collect::<Vec<u32>>();
            let mut v = original.clone();
            reverse_triangles(&mut v);
            assert!(v.chunks_exact(3).eq(original.chunks_exact(3).rev()));
            reverse_triangles(&mut v);
            assert_eq!(v, original);
        }
    }
}
