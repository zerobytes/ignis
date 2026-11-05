//! Persistent angular appearance atlas: offline bake and independent runtime comparison.
#[path = "compressed_views.rs"]
pub mod compressed;
#[path = "representative.rs"]
pub mod representative;
use super::Gpu;
use crate::{
    geometry::{self, Vertex},
    metrics,
};
use anyhow::{Context, Result, ensure};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
const ROOT: &str = "artifacts/appearance-probe/views";
const TEXELS: u32 = 128;
const SIZE: u32 = 256;
const EXTENT: f32 = 1.6;
const CENTER: Vec3 = Vec3::new(0., 0.12, 0.);
const SHADER: &str = include_str!("views.wgsl");
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    matrix: [[f32; 4]; 4],
    inverse: [[f32; 4]; 4],
    source0: [[f32; 4]; 4],
    source1: [[f32; 4]; 4],
    view: [f32; 4],
    light: [f32; 4],
    settings: [f32; 4],
    rect: [f32; 4],
}
#[derive(Clone, Copy, Serialize)]
struct Pose {
    camera: f32,
    light: f32,
    distance: f32,
}
fn angle(degrees: f32) -> f32 {
    let a = degrees.rem_euclid(360.);
    if a >= 360. { 0. } else { a }
}
fn direction(degrees: f32, elevation: f32) -> Vec3 {
    let a = angle(degrees).to_radians();
    let e = elevation.to_radians();
    Vec3::new(e.cos() * a.cos(), e.sin(), e.cos() * a.sin())
}
fn bake_matrix(angle: f32) -> Mat4 {
    Mat4::orthographic_rh(-EXTENT, EXTENT, -EXTENT, EXTENT, 0.1, 20.)
        * Mat4::look_at_rh(CENTER + direction(angle, 55.) * 6., CENTER, Vec3::Y)
}
fn coordinate(angle: f32, n: u32) -> f32 {
    let c = self::angle(angle) / 360. * n as f32;
    if c >= n as f32 { 0. } else { c }
}
fn geometry_uniforms(pose: Pose, bake: bool) -> Uniforms {
    let view = direction(pose.camera, 55.);
    let light = direction(pose.light, 35.);
    let matrix = if bake {
        bake_matrix(pose.camera)
    } else {
        Mat4::perspective_rh(45f32.to_radians(), 1., 0.1, 100.)
            * Mat4::look_at_rh(CENTER + view * pose.distance, CENTER, Vec3::Y)
    };
    Uniforms {
        matrix: matrix.to_cols_array_2d(),
        inverse: Mat4::IDENTITY.to_cols_array_2d(),
        source0: Mat4::IDENTITY.to_cols_array_2d(),
        source1: Mat4::IDENTITY.to_cols_array_2d(),
        view: view.extend(SIZE as f32).to_array(),
        light: light.extend(0.).to_array(),
        settings: [0.; 4],
        rect: [0., 0., 1., 1.],
    }
}
fn uniforms(pose: Pose, n: u32, mode: u32, bake: bool) -> Uniforms {
    let mut u = geometry_uniforms(pose, bake);
    let matrix = Mat4::from_cols_array_2d(&u.matrix);
    let v = coordinate(pose.camera, n);
    let angle = v.floor() * 360. / n as f32;
    u.inverse = matrix.inverse().to_cols_array_2d();
    u.source0 = bake_matrix(angle).to_cols_array_2d();
    u.source1 = bake_matrix(angle + 360. / n as f32).to_cols_array_2d();
    u.settings = [v, coordinate(pose.light, n), n as f32, mode as f32];
    u.rect = rectangle(matrix);
    u
}
// Conservative box includes planar-reprojection error and filtered coverage outside the mesh.
fn rectangle(matrix: Mat4) -> [f32; 4] {
    let mut min = glam::Vec2::ONE;
    let mut max = glam::Vec2::ZERO;
    for x in [-1.6, 1.6] {
        for y in [-0.3, 0.6] {
            for z in [-1.6, 1.6] {
                let p = matrix * Vec3::new(x, y, z).extend(1.);
                let uv = glam::Vec2::new(p.x / p.w * 0.5 + 0.5, 0.5 - p.y / p.w * 0.5);
                min = min.min(uv);
                max = max.max(uv);
            }
        }
    }
    [min.x.max(0.), min.y.max(0.), max.x.min(1.), max.y.min(1.)]
}
fn vertices() -> Vec<Vertex> {
    let mut v = geometry::generate(128).vertices;
    for t in v.chunks_exact_mut(3) {
        for p in t.iter_mut() {
            let [x, y, z] = p.position;
            p.position = [x, y + 0.09 * (x * 4.).sin() * (z * 3.).cos(), z];
        }
        let a = Vec3::from_array(t[0].position);
        let b = Vec3::from_array(t[1].position);
        let c = Vec3::from_array(t[2].position);
        let n = (b - a).cross(c - a).normalize().to_array();
        for p in t {
            p.normal = n;
        }
    }
    // Raised bridge and supports deliberately expose the single-layer disocclusion limit.
    for (lo, hi) in [
        (Vec3::new(-0.75, 0.32, -0.13), Vec3::new(0.75, 0.36, 0.13)),
        (Vec3::new(-0.7, -0.08, -0.1), Vec3::new(-0.6, 0.32, 0.1)),
        (Vec3::new(0.6, -0.08, -0.1), Vec3::new(0.7, 0.32, 0.1)),
    ] {
        let corners = [
            Vec3::new(lo.x, lo.y, lo.z),
            Vec3::new(hi.x, lo.y, lo.z),
            Vec3::new(hi.x, hi.y, lo.z),
            Vec3::new(lo.x, hi.y, lo.z),
            Vec3::new(lo.x, lo.y, hi.z),
            Vec3::new(hi.x, lo.y, hi.z),
            Vec3::new(hi.x, hi.y, hi.z),
            Vec3::new(lo.x, hi.y, hi.z),
        ];
        for [a, b, c, d] in [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
            [0, 1, 5, 4],
            [3, 7, 6, 2],
        ] {
            let n = (corners[b] - corners[a])
                .cross(corners[c] - corners[a])
                .normalize();
            for i in [a, b, c, a, c, d] {
                v.push(Vertex {
                    position: corners[i].to_array(),
                    normal: n.to_array(),
                    color: [0.08, 0.48, 0.31],
                    tile: 0,
                });
            }
        }
    }
    v
}
fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
fn signature(vertices: &[Vertex], n: u32) -> String {
    format!(
        "{:016x}-{:016x}-{:016x}-{:016x}-{n}-{TEXELS}-v1",
        hash(bytemuck::cast_slice(vertices)),
        hash(SHADER.as_bytes()),
        hash(include_bytes!("view_downsample.wgsl")),
        hash(bytemuck::bytes_of(&uniforms(
            Pose {
                camera: 17.,
                light: 33.,
                distance: 6.
            },
            n,
            0,
            true
        )))
    )
}
#[derive(Serialize, Deserialize)]
struct Manifest {
    schema: u32,
    count: u32,
    texels: u32,
    signature: String,
    checksum: u64,
    bytes: usize,
    bake_seconds: f64,
    triangles: usize,
    geometry_bytes: usize,
}
fn expected_bytes(n: u32) -> usize {
    n as usize * (n as usize + 1) * TEXELS as usize * TEXELS as usize * 8
}
fn validate(manifest: &Manifest, bytes: &[u8], expected: &str) -> Result<()> {
    ensure!(
        manifest.schema == 1 && [12, 24].contains(&manifest.count) && manifest.texels == TEXELS,
        "unsupported atlas"
    );
    ensure!(manifest.signature == expected, "stale bake: rebuild views");
    ensure!(
        bytes.len() == expected_bytes(manifest.count) && bytes.len() == manifest.bytes,
        "truncated atlas"
    );
    ensure!(hash(bytes) == manifest.checksum, "corrupted atlas");
    Ok(())
}
struct Atlas {
    groups: Vec<wgpu::BindGroup>,
    uniforms: Vec<wgpu::Buffer>,
    _texture: wgpu::Texture,
    pipeline: Option<wgpu::RenderPipeline>,
    manifest: Manifest,
    load_ms: f64,
}
struct Lab {
    gpu: Gpu,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    full: wgpu::RenderPipeline,
    bake: wgpu::RenderPipeline,
    proxy: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    dummy: wgpu::BindGroup,
    mesh: wgpu::Buffer,
    count: u32,
    index: Option<wgpu::Buffer>,
    index_hash: u64,
    shader: &'static str,
    bounds: Option<[Vec3; 2]>,
    output_size: u32,
    physical_viewport: Option<[u32; 2]>,
}
impl Lab {
    async fn new(vertices: &[Vertex]) -> Result<Self> {
        Self::new_source(vertices, &[], SHADER).await
    }
    async fn new_source(
        vertices: &[Vertex],
        indices: &[u32],
        shader: &'static str,
    ) -> Result<Self> {
        let gpu = Gpu::new().await?;
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("angular atlas"),
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("precomputed views"),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Uint32];
        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: 40,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attrs,
        }];
        let make = |proxy: bool, format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some(if proxy { "screen" } else { "geometry" }),
                    compilation_options: Default::default(),
                    buffers: if proxy { &[] } else { &vertex_buffers },
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(if proxy { "proxy" } else { "surface" }),
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
                    depth_write_enabled: !proxy,
                    depth_compare: if proxy {
                        wgpu::CompareFunction::Always
                    } else {
                        wgpu::CompareFunction::Less
                    },
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let full = make(false, wgpu::TextureFormat::Rgba32Float);
        let bake = make(false, wgpu::TextureFormat::Rgba16Float);
        let proxy = make(true, wgpu::TextureFormat::Rgba32Float);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let blank = texture(
            device,
            1,
            1,
            1,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let dummy = group(device, &layout, &uniform, &sampler, &blank);
        let mesh = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("full source"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index = (!indices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indexed source"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            })
        });
        Ok(Self {
            gpu,
            layout,
            uniform,
            full,
            bake,
            proxy,
            sampler,
            dummy,
            mesh,
            count: if indices.is_empty() {
                vertices.len() as u32
            } else {
                indices.len() as u32
            },
            index,
            index_hash: hash(bytemuck::cast_slice(indices)),
            shader,
            output_size: SIZE,
            physical_viewport: None,
            bounds: (!indices.is_empty()).then(|| {
                let mut lo = Vec3::splat(f32::INFINITY);
                let mut hi = Vec3::splat(f32::NEG_INFINITY);
                for v in vertices {
                    let p = Vec3::from_array(v.position);
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
                [lo - Vec3::splat(0.2), hi + Vec3::splat(0.2)]
            }),
        })
    }
    fn source_signature(&self, vertices: &[Vertex], n: u32) -> String {
        let base = signature(vertices, n);
        if self.index.is_none() {
            base
        } else {
            format!(
                "{base}-{:016x}-{:016x}",
                self.index_hash,
                hash(self.shader.as_bytes())
            )
        }
    }
    fn prepare(&self, pose: Pose, u: &mut Uniforms) {
        if let Some(viewport) = self.physical_viewport {
            let m = representative::physical::window_matrix(pose, viewport, self.output_size);
            u.matrix = m.to_cols_array_2d();
            u.inverse = m.inverse().to_cols_array_2d();
        }
        self.bound(u);
        u.view[3] = self.output_size as f32;
    }
    fn bound(&self, u: &mut Uniforms) {
        if let Some([lo, hi]) = self.bounds {
            let m = Mat4::from_cols_array_2d(&u.matrix);
            let mut min = glam::Vec2::ONE;
            let mut max = glam::Vec2::ZERO;
            for x in [lo.x, hi.x] {
                for y in [lo.y, hi.y] {
                    for z in [lo.z, hi.z] {
                        let p = m * Vec3::new(x, y, z).extend(1.);
                        let uv = glam::Vec2::new(p.x / p.w * 0.5 + 0.5, 0.5 - p.y / p.w * 0.5);
                        min = min.min(uv);
                        max = max.max(uv);
                    }
                }
            }
            u.rect = [min.x.max(0.), min.y.max(0.), max.x.min(1.), max.y.min(1.)];
        }
    }
    fn write(&self, u: &Uniforms) {
        self.gpu
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(u));
    }
    fn submit(&self, encoder: wgpu::CommandEncoder) -> Result<()> {
        let index = self.gpu.queue.submit([encoder.finish()]);
        self.gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: Some(Duration::from_secs(60)),
        })?;
        Ok(())
    }
    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: (&wgpu::TextureView, &wgpu::TextureView),
        pipeline: &wgpu::RenderPipeline,
        group: &wgpu::BindGroup,
        proxy: bool,
        query: Option<u32>,
    ) {
        self.draw_window(
            encoder,
            target,
            pipeline,
            group,
            proxy,
            query.map(|i| (Some(i * 2), Some(i * 2 + 1))),
        );
    }
    fn draw_window(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: (&wgpu::TextureView, &wgpu::TextureView),
        pipeline: &wgpu::RenderPipeline,
        group: &wgpu::BindGroup,
        proxy: bool,
        query: Option<(Option<u32>, Option<u32>)>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("view probe"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.0,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: target.1,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: query
                .filter(|(a, b)| a.is_some() || b.is_some())
                .and_then(|i| {
                    self.gpu
                        .query
                        .as_ref()
                        .map(|q| wgpu::RenderPassTimestampWrites {
                            query_set: q,
                            beginning_of_pass_write_index: i.0,
                            end_of_pass_write_index: i.1,
                        })
                }),
            occlusion_query_set: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        if proxy {
            pass.draw(0..6, 0..1)
        } else {
            pass.set_vertex_buffer(0, self.mesh.slice(..));
            if let Some(index) = &self.index {
                pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.count, 0, 0..1);
            } else {
                pass.draw(0..self.count, 0..1);
            }
        }
    }
    fn capture(
        &mut self,
        pose: Pose,
        atlas: Option<&Atlas>,
        mode: u32,
        samples: u32,
    ) -> Result<Vec<[f32; 4]>> {
        self.capture_options(pose, atlas, mode, samples, true, [0.; 2])
    }
    fn capture_options(
        &mut self,
        pose: Pose,
        atlas: Option<&Atlas>,
        mode: u32,
        samples: u32,
        bounded: bool,
        jitter: [f32; 2],
    ) -> Result<Vec<[f32; 4]>> {
        let size = self.output_size * samples;
        self.gpu.ensure_target(size);
        let mut u = atlas.map_or_else(
            || geometry_uniforms(pose, false),
            |a| uniforms(pose, a.manifest.count, mode, false),
        );
        self.prepare(pose, &mut u);
        u.view[3] = size as f32;
        if !bounded {
            u.rect = [0., 0., 1., 1.];
        }
        if jitter != [0.; 2] {
            let m = Mat4::from_translation(Vec3::new(
                2. * jitter[0] / size as f32,
                2. * jitter[1] / size as f32,
                0.,
            )) * Mat4::from_cols_array_2d(&u.matrix);
            u.matrix = m.to_cols_array_2d();
            u.inverse = m.inverse().to_cols_array_2d();
        }
        self.gpu.queue.write_buffer(
            atlas.map_or(&self.uniform, |a| &a.uniforms[mode as usize]),
            0,
            bytemuck::bytes_of(&u),
        );
        let target = &self.gpu.targets[&size];
        let mut enc = self.gpu.device.create_command_encoder(&Default::default());
        self.draw(
            &mut enc,
            (&target.color_view, &target.depth_view),
            atlas.map_or(&self.full, |a| a.pipeline.as_ref().unwrap_or(&self.proxy)),
            atlas.map_or(&self.dummy, |a| &a.groups[mode as usize]),
            atlas.is_some(),
            None,
        );
        copy_readback(&mut enc, &target.color, &target.readback, size, size, 16);
        self.submit(enc)?;
        self.gpu.map(&target.readback)?;
        let bytes = target.readback.slice(..).get_mapped_range();
        let floats: &[f32] = bytemuck::cast_slice(&bytes);
        let mut pixels = vec![[0.; 4]; (self.output_size * self.output_size) as usize];
        for y in 0..size {
            for x in 0..size {
                let src = ((y * size + x) * 4) as usize;
                let dst = ((y / samples) * self.output_size + x / samples) as usize;
                for c in 0..4 {
                    pixels[dst][c] += floats[src + c] / (samples * samples) as f32;
                }
            }
        }
        drop(bytes);
        target.readback.unmap();
        ensure!(
            pixels.iter().flatten().all(|v| v.is_finite()),
            "nonfinite pixels"
        );
        Ok(pixels)
    }
}
fn texture(
    d: &wgpu::Device,
    width: u32,
    height: u32,
    layers: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    d.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}
fn group(
    d: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    u: &wgpu::Buffer,
    s: &wgpu::Sampler,
    t: &wgpu::Texture,
) -> wgpu::BindGroup {
    let view = t.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: u.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(s),
            },
        ],
    })
}
fn copy_readback(
    e: &mut wgpu::CommandEncoder,
    t: &wgpu::Texture,
    b: &wgpu::Buffer,
    w: u32,
    h: u32,
    bpp: u32,
) {
    e.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: b,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * bpp),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}
impl Lab {
    fn bake_atlas(&self, n: u32, vertices: &[Vertex]) -> Result<()> {
        self.bake_atlas_in(n, vertices, Path::new(ROOT))
    }
    fn bake_atlas_in(&self, n: u32, vertices: &[Vertex], root: &Path) -> Result<()> {
        let start = Instant::now();
        let d = &self.gpu.device;
        let large = texture(
            d,
            TEXELS * 4,
            TEXELS * 4,
            1,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let depth = texture(
            d,
            TEXELS * 4,
            TEXELS * 4,
            1,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let small = texture(
            d,
            TEXELS,
            TEXELS,
            1,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let lv = large.create_view(&Default::default());
        let dv = depth.create_view(&Default::default());
        let sv = small.create_view(&Default::default());
        let read = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (TEXELS * TEXELS * 8) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&lv),
            }],
        });
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("view_downsample.wgsl").into()),
        });
        let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("16 sample box filter"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
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
        let mut bytes = Vec::with_capacity(expected_bytes(n));
        for v in 0..n {
            for l in 0..=n {
                self.write(&uniforms(
                    Pose {
                        camera: v as f32 * 360. / n as f32,
                        light: l as f32 * 360. / n as f32,
                        distance: 6.,
                    },
                    n,
                    if l == n { 2 } else { 0 },
                    true,
                ));
                let mut e = d.create_command_encoder(&Default::default());
                self.draw(&mut e, (&lv, &dv), &self.bake, &self.dummy, false, None);
                {
                    let mut p = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &sv,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    p.set_pipeline(&pipeline);
                    p.set_bind_group(0, &group, &[]);
                    p.draw(0..3, 0..1);
                }
                copy_readback(&mut e, &small, &read, TEXELS, TEXELS, 8);
                self.submit(e)?;
                self.gpu.map(&read)?;
                let data = read.slice(..).get_mapped_range();
                bytes.extend_from_slice(&data);
                drop(data);
                read.unmap();
            }
            println!("bake {n}: view {}/{n}", v + 1);
        }
        let manifest = Manifest {
            schema: 1,
            count: n,
            texels: TEXELS,
            signature: self.source_signature(vertices, n),
            checksum: hash(&bytes),
            bytes: bytes.len(),
            bake_seconds: start.elapsed().as_secs_f64(),
            triangles: self.count as usize / 3,
            geometry_bytes: std::mem::size_of_val(vertices)
                + self.index.as_ref().map_or(0, |_| self.count as usize * 4),
        };
        fs::write(root.join(format!("atlas{n}.bin.tmp")), &bytes)?;
        fs::rename(
            root.join(format!("atlas{n}.bin.tmp")),
            root.join(format!("atlas{n}.bin")),
        )?;
        fs::write(
            root.join(format!("atlas{n}.json")),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        println!(
            "baked {n}: {} bytes in {:.2}s",
            manifest.bytes, manifest.bake_seconds
        );
        Ok(())
    }
    fn load(&self, n: u32, vertices: &[Vertex]) -> Result<Atlas> {
        self.load_in(n, vertices, Path::new(ROOT))
    }
    fn load_in(&self, n: u32, vertices: &[Vertex], root: &Path) -> Result<Atlas> {
        let start = Instant::now();
        let manifest: Manifest = serde_json::from_slice(
            &fs::read(root.join(format!("atlas{n}.json"))).context("run --bake-views first")?,
        )?;
        let bytes = fs::read(root.join(format!("atlas{n}.bin")))?;
        ensure!(manifest.count == n, "wrong atlas count");
        validate(&manifest, &bytes, &self.source_signature(vertices, n))?;
        let t = texture(
            &self.gpu.device,
            TEXELS * (n + 1),
            TEXELS,
            n,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let tile_bytes = (TEXELS * TEXELS * 8) as usize;
        for v in 0..n {
            for l in 0..=n {
                let offset = (v * (n + 1) + l) as usize * tile_bytes;
                self.gpu.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &t,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: l * TEXELS,
                            y: 0,
                            z: v,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bytes[offset..offset + tile_bytes],
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(TEXELS * 8),
                        rows_per_image: Some(TEXELS),
                    },
                    wgpu::Extent3d {
                        width: TEXELS,
                        height: TEXELS,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
        let mut groups = Vec::new();
        let mut uniforms = Vec::new();
        for _ in 0..2 {
            let u = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            groups.push(group(&self.gpu.device, &self.layout, &u, &self.sampler, &t));
            uniforms.push(u);
        }
        self.submit(self.gpu.device.create_command_encoder(&Default::default()))?;
        Ok(Atlas {
            groups,
            uniforms,
            _texture: t,
            pipeline: None,
            manifest,
            load_ms: start.elapsed().as_secs_f64() * 1000.,
        })
    }
    fn write_method(&self, pose: Pose, atlas: Option<&Atlas>, mode: usize) -> f64 {
        let start = Instant::now();
        let mut u = atlas.map_or_else(
            || geometry_uniforms(pose, false),
            |a| uniforms(pose, a.manifest.count, mode as u32, false),
        );
        self.prepare(pose, &mut u);
        self.gpu.queue.write_buffer(
            atlas.map_or(&self.uniform, |a| &a.uniforms[mode]),
            0,
            bytemuck::bytes_of(&u),
        );
        start.elapsed().as_secs_f64() * 1e6
    }
    fn bench(&mut self, pose: Pose, atlases: &[Atlas]) -> Result<serde_json::Value> {
        let choices = [
            (None, 0),
            (Some(&atlases[0]), 1),
            (Some(&atlases[1]), 1),
            (Some(&atlases[1]), 0),
        ];
        self.bench_choices(pose, choices)
    }
    fn bench_choices(
        &mut self,
        pose: Pose,
        choices: [(Option<&Atlas>, usize); 4],
    ) -> Result<serde_json::Value> {
        self.bench_choices_batched(pose, choices, 1)
    }
    fn bench_choices_batched(
        &mut self,
        pose: Pose,
        choices: [(Option<&Atlas>, usize); 4],
        batch: usize,
    ) -> Result<serde_json::Value> {
        ensure!([1, 32].contains(&batch), "unsupported timing batch");
        let rounds = if batch == 1 { 63 } else { 9 };
        self.gpu.ensure_target(self.output_size);
        let target = &self.gpu.targets[&self.output_size];
        let cpu: Vec<_> = choices
            .iter()
            .map(|(a, m)| self.write_method(pose, *a, *m))
            .collect();
        let draw =
            |e: &mut wgpu::CommandEncoder, m: usize, q: Option<(Option<u32>, Option<u32>)>| {
                let (a, mode) = choices[m];
                self.draw_window(
                    e,
                    (&target.color_view, &target.depth_view),
                    a.map_or(&self.full, |a| a.pipeline.as_ref().unwrap_or(&self.proxy)),
                    a.map_or(&self.dummy, |a| &a.groups[mode]),
                    a.is_some(),
                    q,
                );
            };
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        for _ in 0..4 {
            for m in 0..4 {
                draw(&mut e, m, None);
            }
        }
        self.submit(e)?;
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        for r in 0..rounds {
            for offset in 0..4 {
                let m = (r + offset) % 4;
                let index = ((r * 4 + m) * 2) as u32;
                for repeat in 0..batch {
                    draw(
                        &mut e,
                        m,
                        Some((
                            (repeat == 0).then_some(index),
                            (repeat + 1 == batch).then_some(index + 1),
                        )),
                    );
                }
            }
        }
        self.submit(e)?;
        let mut timings = vec![Vec::new(); 4];
        if let Some(query) = &self.gpu.query {
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(query, 0..(rounds * 8) as u32, &self.gpu.query_resolve, 0);
            e.copy_buffer_to_buffer(
                &self.gpu.query_resolve,
                0,
                &self.gpu.query_readback,
                0,
                8192,
            );
            self.submit(e)?;
            self.gpu.map(&self.gpu.query_readback)?;
            let bytes = self.gpu.query_readback.slice(..).get_mapped_range();
            let stamps: &[u64] = bytemuck::cast_slice(&bytes);
            let scale = f64::from(self.gpu.timestamp_period_ns.unwrap()) / 1e6;
            for r in 0..rounds {
                for (m, times) in timings.iter_mut().enumerate() {
                    let i = (r * 4 + m) * 2;
                    ensure!(stamps[i + 1] >= stamps[i], "invalid timestamps");
                    times.push((stamps[i + 1] - stamps[i]) as f64 * scale / batch as f64);
                }
            }
            drop(bytes);
            self.gpu.query_readback.unmap();
        }
        // Serialized wall time includes update, encoding, submission and completion; no readback.
        let mut wall = vec![Vec::new(); 4];
        for r in 0..9 {
            for off in 0..4 {
                let m = (r + off) % 4;
                let start = Instant::now();
                self.write_method(pose, choices[m].0, choices[m].1);
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                draw(&mut e, m, None);
                self.submit(e)?;
                wall[m].push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        Ok(json!((0..4).map(|m|json!({"gpu":stats(&mut timings[m]),"serial_frame":stats(&mut wall[m]),"selection_uniform_us":cpu[m]})).collect::<Vec<_>>()))
    }
}
fn stats(v: &mut [f64]) -> serde_json::Value {
    v.sort_by(f64::total_cmp);
    json!({"median":v.get(v.len()/2),"p10":v.get(v.len()/10),"p90":v.get(v.len()*9/10),"samples":v.len()})
}
pub fn run(bake: bool) -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let vertices = vertices();
    let mut lab = pollster::block_on(Lab::new(&vertices))?;
    if bake {
        for n in [12, 24] {
            lab.bake_atlas(n, &vertices)?;
        }
        return Ok(());
    }
    let atlases = [lab.load(12, &vertices)?, lab.load(24, &vertices)?];
    let start = Instant::now();
    let mut rows = Vec::new();
    for scene in ["camera", "light", "both", "near", "seam"] {
        type Image = Vec<[f32; 4]>;
        let mut previous: Option<(Image, Vec<Image>)> = None;
        let count = if scene == "seam" { 17 } else { 49 };
        for frame in 0..count {
            let t = frame as f32 / (count - 1) as f32;
            let angle = if scene == "seam" {
                358. + t * 4.
            } else {
                3.75 + t * 360.
            };
            let pose = Pose {
                camera: if scene == "light" { 43.75 } else { angle },
                light: if scene == "camera" {
                    67.25
                } else if scene == "seam" {
                    angle + 0.25
                } else {
                    5.625 + t * 360. * 1.3
                },
                distance: if scene == "near" { 4. } else { 12. },
            };
            let truth = lab.capture(pose, None, 0, 16)?;
            let full = lab.capture(pose, None, 0, 4)?;
            let candidates = vec![
                lab.capture(pose, Some(&atlases[0]), 1, 4)?,
                lab.capture(pose, Some(&atlases[1]), 1, 4)?,
                lab.capture(pose, Some(&atlases[1]), 0, 4)?,
            ];
            let id = format!("{scene}_{frame:02}");
            let root = Path::new(ROOT);
            metrics::png(&root.join(format!("{id}_reference.png")), &truth, SIZE)?;
            metrics::png(&root.join(format!("{id}_full.png")), &full, SIZE)?;
            let mut errors = Vec::new();
            for (m, pixels) in candidates.iter().enumerate() {
                metrics::png(&root.join(format!("{id}_m{m}.png")), pixels, SIZE)?;
                metrics::error_png(
                    &root.join(format!("{id}_m{m}_error.png")),
                    &truth,
                    pixels,
                    SIZE,
                )?;
                let temporal = previous
                    .as_ref()
                    .map(|(r, ps)| metrics::temporal(r, &ps[m], &truth, pixels));
                errors.push(
                    json!({"error":metrics::compare(&truth,pixels),"temporal_residual":temporal}),
                );
            }
            for (index, mode) in [(0, 1), (1, 1), (1, 0)] {
                let bounded = lab.capture(pose, Some(&atlases[index]), mode, 1)?;
                let unbounded =
                    lab.capture_options(pose, Some(&atlases[index]), mode, 1, false, [0.; 2])?;
                ensure!(
                    bounded == unbounded,
                    "bounded proxy changed pixels: {id} atlas{index} mode{mode}"
                );
            }
            let timings = lab.bench(pose, &atlases)?;
            println!(
                "{id}: error12 {:.5}, error24 {:.5}",
                metrics::compare(&truth, &candidates[0]).rmse_surface,
                metrics::compare(&truth, &candidates[1]).rmse_surface
            );
            rows.push(json!({"id":id,"scene":scene,"frame":frame,"pose":pose,"full_error":metrics::compare(&truth,&full),"methods":errors,"timings":timings}));
            previous = Some((truth, candidates));
        }
    }
    let mut checks = checks(&mut lab, &atlases)?;
    checks["bounded_vs_unbounded_pairs"] = json!(rows.len() * 3);
    fs::write(
        Path::new(ROOT).join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":1,"adapter":lab.gpu.adapter,"seconds":start.elapsed().as_secs_f64(),"triangles":vertices.len()/3,"geometry_bytes":std::mem::size_of_val(vertices.as_slice()),"atlases":atlases.iter().map(|a|json!({"manifest":a.manifest,"load_ms":a.load_ms})).collect::<Vec<_>>(),"size":SIZE,"reference_spp":256,"candidate_spp":16,"timing_spp":1,"checks":checks,"cases":rows}),
        )?,
    )?;
    Ok(())
}
fn checks(lab: &mut Lab, atlases: &[Atlas]) -> Result<serde_json::Value> {
    let p = Pose {
        camera: 13.75,
        light: 71.25,
        distance: 12.,
    };
    let a = lab.capture(p, Some(&atlases[1]), 1, 1)?;
    let b = lab.capture(p, Some(&atlases[1]), 1, 1)?;
    ensure!(a == b, "nondeterministic replay");
    let wrap = lab.capture(
        Pose {
            camera: p.camera + 360.,
            light: p.light + 360.,
            ..p
        },
        Some(&atlases[1]),
        1,
        1,
    )?;
    let delta = metrics::compare(&a, &wrap);
    ensure!(a == wrap, "angular wrap discontinuity");
    let moved_light = lab.capture(
        Pose {
            light: p.light + 70.,
            ..p
        },
        Some(&atlases[1]),
        1,
        1,
    )?;
    ensure!(
        metrics::compare(&a, &moved_light).rmse_surface > 0.01,
        "light did not affect baked response"
    );
    let returned = lab.capture(p, Some(&atlases[1]), 1, 1)?;
    ensure!(a == returned, "light change left stale state");
    let mut grid = Vec::new();
    for angle in [0., 90., 180., 270.] {
        let pose = Pose {
            camera: angle,
            light: angle,
            distance: 12.,
        };
        let r = lab.capture(pose, None, 0, 16)?;
        let c = lab.capture(pose, Some(&atlases[1]), 1, 4)?;
        grid.push(json!({"angle":angle,"error":metrics::compare(&r,&c)}));
    }
    let mut convergence = Vec::new();
    for pose in [
        p,
        Pose {
            camera: 93.75,
            light: 122.625,
            distance: 4.,
        },
    ] {
        let r64 = lab.capture(pose, None, 0, 8)?;
        let r256 = lab.capture(pose, None, 0, 16)?;
        let mut r1024 = vec![[0.; 4]; (SIZE * SIZE) as usize];
        for x in [-0.25, 0.25] {
            for y in [-0.25, 0.25] {
                let image = lab.capture_options(pose, None, 0, 16, true, [x, y])?;
                for (dst, src) in r1024.iter_mut().zip(image) {
                    for c in 0..4 {
                        dst[c] += src[c] * 0.25;
                    }
                }
            }
        }
        convergence.push(json!({"pose":pose,"reference64_vs256":metrics::compare(&r256,&r64),"reference256_vs1024":metrics::compare(&r1024,&r256)}));
    }
    Ok(
        json!({"repeat_identical":true,"wrap_error":delta,"light_changes_response":true,"return_identical":true,"grid_controls":grid,"reference_convergence":convergence}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn angular_selection_wraps_without_index_overflow() {
        for n in [12, 24] {
            for a in [-720., -0.001, -1e-8, 0., 359.999, 359.99997, 360., 720.] {
                let x = coordinate(a, n);
                assert!(x >= 0. && x < n as f32);
                let i = x.floor() as u32;
                assert!((i + 1) % n < n);
            }
            assert_eq!(coordinate(15., n), coordinate(375., n));
            let p = Pose {
                camera: 13.75,
                light: 71.25,
                distance: 12.,
            };
            assert_eq!(
                bytemuck::bytes_of(&uniforms(p, n, 1, false)),
                bytemuck::bytes_of(&uniforms(
                    Pose {
                        camera: p.camera + 360.,
                        light: p.light + 360.,
                        ..p
                    },
                    n,
                    1,
                    false
                ))
            );
        }
    }
    #[test]
    fn artifact_validation_rejects_stale_corrupt_and_truncated_data() {
        let data = vec![0; expected_bytes(12)];
        let mut m = Manifest {
            schema: 1,
            count: 12,
            texels: TEXELS,
            signature: "valid".into(),
            checksum: hash(&data),
            bytes: data.len(),
            bake_seconds: 0.,
            triangles: 0,
            geometry_bytes: 0,
        };
        assert!(validate(&m, &data, "valid").is_ok());
        assert!(validate(&m, &data[..data.len() - 1], "valid").is_err());
        assert!(validate(&m, &data, "stale").is_err());
        m.checksum ^= 1;
        assert!(validate(&m, &data, "valid").is_err());
        m.schema = 2;
        assert!(validate(&m, &data, "valid").is_err());
    }
}
