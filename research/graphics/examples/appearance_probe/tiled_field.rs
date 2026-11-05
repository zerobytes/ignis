//! View-dependent conservative height intervals shared by a screen block.
use super::*;
pub(super) const ROOT: &str = "artifacts/appearance-probe/tiled-field";
pub(super) const EDGE: u32 = 8;
pub(super) const MAX_SIZE: [u32; 2] = [3456, 2234];
pub(super) const SCRATCH_BYTES: u64 =
    (MAX_SIZE[0].div_ceil(8) * MAX_SIZE[1].div_ceil(8) * 8) as u64;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    inverse: [[f32; 4]; 4],
    eye: [f32; 4],
    bounds: [f32; 4],
    size: [u32; 4],
    dispatch: [u32; 4],
}
pub(super) fn rays(c: &Camera) -> [[f32; 4]; 4] {
    (c.view.inverse()
        * Mat4::from_cols(
            glam::Vec4::new(
                30f32.to_radians().tan() * c.size[0] as f32 / c.size[1] as f32,
                0.,
                0.,
                0.,
            ),
            glam::Vec4::new(0., 30f32.to_radians().tan(), 0., 0.),
            glam::Vec4::ZERO,
            glam::Vec4::new(0., 0., -1., 0.),
        ))
    .to_cols_array_2d()
}
pub(super) fn scissor(a: &Asset, c: &Camera) -> [u32; 4] {
    let mut low = glam::Vec2::splat(f32::INFINITY);
    let mut high = glam::Vec2::splat(f32::NEG_INFINITY);
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                let clip =
                    c.matrix * Vec3::new(a.bounds[x][0], a.bounds[y][1], a.bounds[z][2]).extend(1.);
                let ndc = clip.truncate() / clip.w;
                let pixel = (ndc.truncate() * glam::Vec2::new(0.5, -0.5) + glam::Vec2::splat(0.5))
                    * glam::Vec2::new(c.size[0] as f32, c.size[1] as f32);
                low = low.min(pixel);
                high = high.max(pixel);
            }
        }
    }
    let x = (low.x.floor() - 1.).max(0.).min(c.size[0] as f32) as u32;
    let y = (low.y.floor() - 1.).max(0.).min(c.size[1] as f32) as u32;
    let w = ((high.x.ceil() + 1.).max(0.).min(c.size[0] as f32) as u32).saturating_sub(x);
    let h = ((high.y.ceil() + 1.).max(0.).min(c.size[1] as f32) as u32).saturating_sub(y);
    [x, y, w, h]
}
pub(super) struct Tiles {
    pub edge: u32,
    pub scratch: wgpu::Buffer,
    params: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}
impl Tiles {
    pub fn new(gpu: &Gpu) -> Self {
        let d = &gpu.device;
        let scratch = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tile intervals"),
            size: SCRATCH_BYTES,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let params = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tile camera"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tile interval compute"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tiled_field.wgsl").into()),
        });
        let pipeline = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("tile interval compute"),
            layout: Some(&d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            })),
            module: &shader,
            entry_point: Some("tiles"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            edge: EDGE,
            scratch,
            params,
            layout,
            pipeline,
        }
    }
    pub fn group(&self, gpu: &Gpu, hierarchy: &wgpu::TextureView) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tile input"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(hierarchy),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.scratch.as_entire_binding(),
                },
            ],
        })
    }
    pub fn encode(
        &self,
        gpu: &Gpu,
        e: &mut wgpu::CommandEncoder,
        data: &geometric_field::Data,
        c: &Camera,
        rect: [u32; 4],
        q: Option<u32>,
    ) {
        assert!(
            c.size[0] <= MAX_SIZE[0]
                && c.size[1] <= MAX_SIZE[1]
                && [8, 16, 32].contains(&self.edge)
        );
        let [x, y, w, h] = rect;
        let start = [x / self.edge, y / self.edge];
        let end = [(x + w).div_ceil(self.edge), (y + h).div_ceil(self.edge)];
        gpu.queue.write_buffer(
            &self.params,
            0,
            bytemuck::bytes_of(&Params {
                inverse: rays(c),
                eye: c.eye.extend(0.).to_array(),
                bounds: [
                    data.field.limits[0],
                    data.field.limits[1],
                    data.field.grid[0] as f32,
                    data.field.grid[1] as f32,
                ],
                size: [
                    c.size[0],
                    c.size[1],
                    self.edge,
                    c.size[0].div_ceil(self.edge),
                ],
                dispatch: [start[0], start[1], end[0] - start[0], end[1] - start[1]],
            }),
        );
        let mut pass = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("shared tile search"),
            timestamp_writes: q.and_then(|base| {
                gpu.query
                    .as_ref()
                    .map(|query_set| wgpu::ComputePassTimestampWrites {
                        query_set,
                        beginning_of_pass_write_index: Some(base),
                        end_of_pass_write_index: None,
                    })
            }),
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, data.tile_group.as_ref().unwrap(), &[]);
        pass.dispatch_workgroups(
            (end[0] - start[0]).div_ceil(8),
            (end[1] - start[1]).div_ceil(8),
            1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn perspective_cone_contains_interior_rays() {
        // Test the geometric bound independently from hierarchy/height implementation.
        for tilt in [-0.8f64, -0.3, 0.0, 0.4, 0.8] {
            let direction = |x: f64, z: f64| [x + 0.2 * z, -1. + tilt * x + 0.1 * z, z - 0.15 * x];
            let mut lo = [f64::INFINITY; 2];
            let mut hi = [f64::NEG_INFINITY; 2];
            for x in [-0.5, 0.5] {
                for z in [-0.5, 0.5] {
                    let d = direction(x, z);
                    assert!(d[1] < 0.);
                    for j in 0..2 {
                        let v = d[j * 2] / d[1];
                        lo[j] = lo[j].min(v);
                        hi[j] = hi[j].max(v);
                    }
                }
            }
            for x in 0..65 {
                for z in 0..65 {
                    let d = direction(x as f64 / 64. - 0.5, z as f64 / 64. - 0.5);
                    for j in 0..2 {
                        let v = d[j * 2] / d[1];
                        assert!(v >= lo[j] - 1e-14 && v <= hi[j] + 1e-14);
                    }
                }
            }
        }
    }
    #[test]
    fn partial_tiles_fit_scratch_and_cover_scissor() {
        for edge in [8u32, 16, 32] {
            for size in [MAX_SIZE, [1728, 1117], [101, 73]] {
                let stride = size[0].div_ceil(edge);
                let rows = size[1].div_ceil(edge);
                assert!(u64::from(stride * rows) * 8 <= SCRATCH_BYTES);
                for [x, y, w, h] in [
                    [0, 0, size[0], size[1]],
                    [3, 7, size[0] - 3, size[1] - 7],
                    [size[0] - 1, size[1] - 1, 1, 1],
                ] {
                    let begin = [x / edge, y / edge];
                    let end = [(x + w).div_ceil(edge), (y + h).div_ceil(edge)];
                    for pixel in [[x, y], [x + w - 1, y + h - 1]] {
                        let t = pixel.map(|p| p / edge);
                        assert!(
                            t[0] >= begin[0] && t[0] < end[0] && t[1] >= begin[1] && t[1] < end[1]
                        );
                        assert!(t[1] * stride + t[0] < stride * rows);
                    }
                }
            }
        }
    }
}
