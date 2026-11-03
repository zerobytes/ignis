//! Diagnostic build-time grid field. Not a Scene format or an automatic LOD certificate.
//! Positions/masks are compiled once; runtime submits visible spatial blocks.
use anyhow::{Result, ensure};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::{
    io::{Read, Write},
    sync::Arc,
};

const MAGIC: &[u8; 8] = b"IGFLD1\0\0";
pub const SIDE: u32 = 16;
pub const LEVELS: usize = 6;
pub const ABSENT: f32 = -1_000_000.;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SampleVertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Pack {
        Pack {
            origin: [-4., -4.],
            spacing: 0.25,
            tiles: [1, 1],
            heights: vec![0.; (SIDE * SIDE) as usize],
            color: [0.1, 0.3, 0.04],
            levels: std::array::from_fn(|_| Level {
                stride: 1,
                vertices: vec![
                    SampleVertex {
                        position: [0., 0., 0., 0.],
                        normal: [0., 1., 0., 0.]
                    };
                    3
                ],
            }),
        }
    }
    #[test]
    fn binary_roundtrip_and_invalid_resource_rejection() {
        let p = fixture();
        let mut b = vec![];
        p.write(&mut b).unwrap();
        let restored = Pack::read(b.as_slice()).unwrap();
        assert_eq!(restored.heights, p.heights);
        assert_eq!(restored.tiles, p.tiles);
        for n in [0, 7, 20, b.len() - 1] {
            assert!(Pack::read(&b[..n]).is_err());
        }
        let mut bad = b.clone();
        bad[32..40].fill(255);
        assert!(Pack::read(bad.as_slice()).is_err());
        b.push(0);
        assert!(Pack::read(b.as_slice()).is_err());
        let mut p = fixture();
        p.heights[3] = f32::NAN;
        assert!(p.validate().is_err());
        p.heights[3] = ABSENT;
        p.levels[0].stride = 3;
        assert!(p.validate().is_err());
    }
    #[test]
    fn conservative_block_test_preserves_near_plane_and_partial_intersection() {
        let vp = Mat4::perspective_rh(1., 1.5, 0.05, 100.);
        assert!(visible(Vec3::new(-1., -1., -2.), Vec3::new(1., 1., 1.), vp));
        assert!(!visible(Vec3::new(-1., -1., 2.), Vec3::new(1., 1., 3.), vp));
        assert!(visible(
            Vec3::new(-100., -1., -3.),
            Vec3::new(1., 1., -2.),
            vp
        ));
        assert!(!visible(
            Vec3::new(100., -1., -3.),
            Vec3::new(101., 1., -2.),
            vp
        ));
    }
}
pub struct Level {
    pub vertices: Vec<SampleVertex>,
    pub stride: u32,
}
pub struct Pack {
    pub origin: [f32; 2],
    pub spacing: f32,
    pub tiles: [u32; 2],
    /// Tile-major heights, including masked locations. Four bytes per planting site.
    pub heights: Vec<f32>,
    pub levels: [Level; LEVELS],
    pub color: [f32; 3],
}
impl Pack {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.origin.iter().all(|v| v.is_finite())
                && self.spacing.is_finite()
                && self.spacing > 0.,
            "Invalid field metric"
        );
        let count = u64::from(self.tiles[0])
            .checked_mul(u64::from(self.tiles[1]))
            .and_then(|n| n.checked_mul(u64::from(SIDE * SIDE)))
            .ok_or_else(|| anyhow::anyhow!("Field size overflow"))?;
        ensure!(
            self.tiles.iter().all(|n| *n > 0) && count == self.heights.len() as u64,
            "Field grid length mismatch"
        );
        ensure!(
            self.heights
                .iter()
                .all(|v| v.is_finite() && (*v == ABSENT || v.abs() < 100_000.)),
            "Invalid field heights"
        );
        ensure!(
            self.color
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
            "Invalid field color"
        );
        for l in &self.levels {
            ensure!(
                l.stride.is_power_of_two()
                    && l.stride <= SIDE
                    && !l.vertices.is_empty()
                    && l.vertices.len() % 3 == 0
                    && l.vertices.len() < 1_000_000,
                "Invalid field level"
            );
            ensure!(
                l.vertices.iter().all(|v| v
                    .position
                    .iter()
                    .chain(v.normal.iter())
                    .all(|c| c.is_finite())),
                "Invalid field template"
            );
            ensure!(
                l.vertices
                    .len()
                    .checked_mul((SIDE / l.stride).pow(2) as usize)
                    .is_some_and(|v| v < u32::MAX as usize),
                "Field draw exceeds u32"
            );
        }
        Ok(())
    }
    pub fn write(&self, mut w: impl Write) -> Result<()> {
        self.validate()?;
        w.write_all(MAGIC)?;
        for v in [
            self.origin[0],
            self.origin[1],
            self.spacing,
            self.color[0],
            self.color[1],
            self.color[2],
        ] {
            w.write_all(&v.to_le_bytes())?;
        }
        for v in self.tiles {
            w.write_all(&v.to_le_bytes())?;
        }
        for l in &self.levels {
            w.write_all(&l.stride.to_le_bytes())?;
            w.write_all(&(l.vertices.len() as u32).to_le_bytes())?;
            for v in &l.vertices {
                for f in v.position.into_iter().chain(v.normal) {
                    w.write_all(&f.to_le_bytes())?;
                }
            }
        }
        for h in &self.heights {
            w.write_all(&h.to_le_bytes())?;
        }
        Ok(())
    }
    pub fn read(mut r: impl Read) -> Result<Self> {
        fn word(r: &mut impl Read) -> Result<u32> {
            let mut b = [0; 4];
            r.read_exact(&mut b)?;
            Ok(u32::from_le_bytes(b))
        }
        let mut magic = [0; 8];
        r.read_exact(&mut magic)?;
        ensure!(&magic == MAGIC, "Invalid IGFLD1 header");
        let mut values = [0.; 6];
        for v in &mut values {
            *v = f32::from_bits(word(&mut r)?);
        }
        let tiles = [word(&mut r)?, word(&mut r)?];
        let count = u64::from(tiles[0])
            .checked_mul(u64::from(tiles[1]))
            .and_then(|n| n.checked_mul(u64::from(SIDE * SIDE)))
            .ok_or_else(|| anyhow::anyhow!("Field grid size overflow"))?;
        ensure!(
            count > 0 && count <= 128 * 1024 * 1024,
            "Invalid diagnostic field resource size"
        );
        let mut levels = Vec::new();
        for _ in 0..LEVELS {
            let stride = word(&mut r)?;
            let n = word(&mut r)?;
            ensure!(
                n > 0 && n < 1_000_000 && n % 3 == 0,
                "Invalid field template length"
            );
            let mut vertices = Vec::with_capacity(n as usize);
            for _ in 0..n {
                let mut a = [0.; 8];
                for f in &mut a {
                    *f = f32::from_bits(word(&mut r)?);
                }
                vertices.push(SampleVertex {
                    position: a[..4].try_into().unwrap(),
                    normal: a[4..].try_into().unwrap(),
                });
            }
            levels.push(Level { vertices, stride });
        }
        let mut heights = Vec::with_capacity(count as usize);
        for _ in 0..count {
            heights.push(f32::from_bits(word(&mut r)?));
        }
        let mut tail = [0];
        ensure!(r.read(&mut tail)? == 0, "Trailing field bytes");
        let out = Self {
            origin: [values[0], values[1]],
            spacing: values[2],
            tiles,
            heights,
            levels: levels.try_into().ok().unwrap(),
            color: values[3..].try_into().unwrap(),
        };
        out.validate()?;
        Ok(out)
    }
}
/// Shared build/runtime integer jitter; no platform-dependent random generator.
pub fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}
pub fn jitter(id: u32) -> [f32; 2] {
    let h = hash(id);
    [
        (h & 65535) as f32 / 65535. - 0.5,
        (h >> 16) as f32 / 65535. - 0.5,
    ]
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    metric: [f32; 4],
    grid: [u32; 4],
    color: [f32; 4],
    levels: [[u32; 4]; LEVELS],
}
#[derive(Default, serde::Serialize)]
pub struct Stats {
    pub population: usize,
    pub source_sites: usize,
    pub visible_blocks: usize,
    pub selected_sites: usize,
    pub triangles: usize,
    pub gpu_bytes: u64,
    pub block_upload_bytes: usize,
    pub draws: usize,
    pub casts_shadows: bool,
    pub shadow_triangles: usize,
    pub shadow_blocks: usize,
    pub cpu_prepare_ms: f64,
}
pub(crate) struct Gpu {
    pack: Arc<Pack>,
    pipeline: wgpu::RenderPipeline,
    shadow: wgpu::RenderPipeline,
    bind: wgpu::BindGroup,
    blocks: wgpu::Buffer,
    bounds: Vec<Option<(Vec3, Vec3)>>,
    ranges: [std::ops::Range<u32>; LEVELS],
    shadow_blocks: wgpu::Buffer,
    shadow_ranges: [std::ops::Range<u32>; LEVELS],
    pub stats: Stats,
}
fn visible(lo: Vec3, hi: Vec3, vp: Mat4) -> bool {
    let row = vp.transpose();
    let c = ((lo + hi) * 0.5).extend(1.);
    let e = (hi - lo) * 0.5;
    [
        row.w_axis + row.x_axis,
        row.w_axis - row.x_axis,
        row.w_axis + row.y_axis,
        row.w_axis - row.y_axis,
        row.z_axis,
        row.w_axis - row.z_axis,
    ]
    .iter()
    .all(|p| p.dot(c) + p.truncate().abs().dot(e) >= 0.)
}
impl Gpu {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &wgpu::BindGroupLayout,
        depth: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        pack: Arc<Pack>,
    ) -> Result<Self> {
        use wgpu::util::DeviceExt;
        pack.validate()?;
        let mut params = Params {
            metric: [pack.origin[0], pack.origin[1], pack.spacing, 0.],
            grid: [pack.tiles[0], pack.tiles[1], SIDE, 0],
            color: [pack.color[0], pack.color[1], pack.color[2], 1.],
            levels: [[0; 4]; LEVELS],
        };
        let mut vertices = Vec::new();
        for (i, l) in pack.levels.iter().enumerate() {
            params.levels[i] = [vertices.len() as u32, l.vertices.len() as u32, l.stride, 0];
            vertices.extend_from_slice(&l.vertices);
        }
        ensure!(
            pack.heights.len() as u64 * 4
                <= u64::from(device.limits().max_storage_buffer_binding_size),
            "Field exceeds device storage binding; split the field into resources"
        );
        let heights = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("IGFLD1 resident planting heights"),
            contents: bytemuck::cast_slice(&pack.heights),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let template = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("IGFLD1 compiled mesh scales"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("IGFLD1 metric"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("IGFLD1 data"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
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
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("IGFLD1 resident data"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: heights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: template.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("IGFLD1 native surfaces"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}\n{}",
                    include_str!("scene.wgsl"),
                    "",
                    include_str!("surface_field.wgsl")
                )
                .into(),
            ),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("IGFLD1 native layout"),
            bind_group_layouts: &[scene, &layout],
            push_constant_ranges: &[],
        });
        let shadow_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("IGFLD1 depth layout"),
            bind_group_layouts: &[depth, &layout],
            push_constant_ranges: &[],
        });
        let attrs = wgpu::vertex_attr_array![0=>Uint32x4];
        let targets = [Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let make = |shadow: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("IGFLD1 mesh field"),
                layout: Some(if shadow { &shadow_pl } else { &pl }),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(if shadow {
                        "vs_field_shadow"
                    } else {
                        "vs_field"
                    }),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[("uniform_material", 1.)],
                        ..Default::default()
                    },
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 16,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &attrs,
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if shadow { "fs_shadow" } else { "fs_main" }),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[("uniform_material", 1.)],
                        ..Default::default()
                    },
                    targets: if shadow { &[] } else { &targets },
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
            })
        };
        let pipeline = make(false);
        let shadow = make(true);
        let block_bytes = u64::from(pack.tiles[0]) * u64::from(pack.tiles[1]) * 16;
        let create = || {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("IGFLD1 visible block commands"),
                size: block_bytes.max(16),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let blocks = create();
        let shadow_blocks = create();
        let width = pack.spacing * SIDE as f32;
        let radius = pack
            .levels
            .iter()
            .flat_map(|l| {
                l.vertices.iter().map(move |v| {
                    (v.position[0] * v.position[0] + v.position[2] * v.position[2]).sqrt()
                        * l.stride as f32
                })
            })
            .fold(0., f32::max);
        let min_y = pack
            .levels
            .iter()
            .flat_map(|l| l.vertices.iter().map(|v| v.position[1]))
            .fold(0., f32::min);
        let max_y = pack
            .levels
            .iter()
            .flat_map(|l| l.vertices.iter().map(|v| v.position[1]))
            .fold(0., f32::max);
        let bounds = pack
            .heights
            .chunks_exact((SIDE * SIDE) as usize)
            .enumerate()
            .map(|(i, h)| {
                let lo = h
                    .iter()
                    .copied()
                    .filter(|v| *v != ABSENT)
                    .reduce(f32::min)?;
                let hi = h
                    .iter()
                    .copied()
                    .filter(|v| *v != ABSENT)
                    .reduce(f32::max)?;
                let x = pack.origin[0] + (i as u32 % pack.tiles[0]) as f32 * width;
                let z = pack.origin[1] + (i as u32 / pack.tiles[0]) as f32 * width;
                Some((
                    Vec3::new(x - radius, lo + min_y, z - radius),
                    Vec3::new(x + width + radius, hi + max_y, z + width + radius),
                ))
            })
            .collect();
        let population = pack.heights.iter().filter(|v| **v != ABSENT).count();
        let stats = Stats {
            population,
            source_sites: pack.heights.len(),
            gpu_bytes: heights.size()
                + template.size()
                + uniform.size()
                + blocks.size()
                + shadow_blocks.size(),
            casts_shadows: true,
            ..Default::default()
        };
        let _ = queue;
        Ok(Self {
            pack,
            pipeline,
            shadow,
            bind,
            blocks,
            bounds,
            ranges: std::array::from_fn(|_| 0..0),
            shadow_blocks,
            shadow_ranges: std::array::from_fn(|_| 0..0),
            stats,
        })
    }
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        camera: &crate::model::Camera,
        width: u32,
        height: u32,
        light: &crate::model::Light,
    ) {
        let started = std::time::Instant::now();
        let eye = Vec3::from(camera.position);
        let vp = Mat4::perspective_rh(
            camera.fov_degrees.to_radians(),
            width as f32 / height as f32,
            0.05,
            10000.,
        ) * Mat4::look_at_rh(eye, Vec3::from(camera.target), Vec3::Y);
        let focal = height as f32 / (2. * (camera.fov_degrees.to_radians() * 0.5).tan());
        let mut groups: [Vec<[u32; 4]>; LEVELS] = Default::default();
        let mut shadows: [Vec<[u32; 4]>; LEVELS] = Default::default();
        let shadow_vp = crate::render::shadow_view_projection(light);
        for (i, b) in self.bounds.iter().enumerate() {
            let Some((lo, hi)) = b else { continue };
            let in_view = visible(*lo, *hi, vp);
            // This diagnostic selection is approximate, not a certified appearance bound.
            let d = (eye - eye.clamp(*lo, *hi)).length().max(0.1);
            let metric = d / focal;
            let level = if metric < 0.002 {
                0
            } else if metric < 0.012 {
                1
            } else if metric < 0.045 {
                2
            } else if metric < 0.13 {
                3
            } else if metric < 0.4 {
                4
            } else {
                5
            };
            let command = [
                i as u32,
                i as u32 % self.pack.tiles[0],
                i as u32 / self.pack.tiles[0],
                level as u32,
            ];
            if in_view {
                groups[level].push(command);
            }
            if let Some(shadow_vp) = shadow_vp
                && visible(*lo, *hi, shadow_vp)
            {
                let mut command = command;
                command[3] = 5;
                shadows[5].push(command);
            }
        }
        let mut data = Vec::new();
        self.stats.selected_sites = 0;
        self.stats.triangles = 0;
        self.stats.draws = 0;
        for (l, g) in groups.into_iter().enumerate() {
            let start = data.len() as u32;
            let n = (SIDE / self.pack.levels[l].stride).pow(2) as usize;
            self.stats.selected_sites += g.len() * n;
            self.stats.triangles += g.len() * n * self.pack.levels[l].vertices.len() / 3;
            data.extend(g);
            self.ranges[l] = start..data.len() as u32;
            self.stats.draws += usize::from(!self.ranges[l].is_empty());
        }
        self.stats.visible_blocks = data.len();
        self.stats.block_upload_bytes = data.len() * 16;
        if !data.is_empty() {
            queue.write_buffer(&self.blocks, 0, bytemuck::cast_slice(&data));
        }
        data.clear();
        self.stats.shadow_triangles = 0;
        for (l, g) in shadows.into_iter().enumerate() {
            self.stats.shadow_triangles += g.len()
                * (SIDE / self.pack.levels[l].stride).pow(2) as usize
                * self.pack.levels[l].vertices.len()
                / 3;
            let start = data.len() as u32;
            data.extend(g);
            self.shadow_ranges[l] = start..data.len() as u32;
        }
        if !data.is_empty() {
            queue.write_buffer(&self.shadow_blocks, 0, bytemuck::cast_slice(&data));
        }
        self.stats.shadow_blocks = data.len();
        self.stats.casts_shadows = shadow_vp.is_some();
        self.stats.cpu_prepare_ms = started.elapsed().as_secs_f64() * 1000.;
    }
    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        scene: &'a wgpu::BindGroup,
        shadow: bool,
    ) {
        pass.set_pipeline(if shadow { &self.shadow } else { &self.pipeline });
        pass.set_bind_group(0, scene, &[]);
        pass.set_bind_group(1, &self.bind, &[]);
        pass.set_vertex_buffer(
            0,
            if shadow {
                self.shadow_blocks.slice(..)
            } else {
                self.blocks.slice(..)
            },
        );
        for (l, range) in (if shadow {
            &self.shadow_ranges
        } else {
            &self.ranges
        })
        .iter()
        .enumerate()
        {
            if !range.is_empty() {
                pass.draw(
                    0..self.pack.levels[l].vertices.len() as u32
                        * (SIDE / self.pack.levels[l].stride).pow(2),
                    range.clone(),
                );
            }
        }
    }
}
