//! Native reference-sphere camera. Global addresses never enter a GPU float.
use super::*;
use bytemuck::{Pod, Zeroable};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub pose: Pose,
    pub pitch_degrees: f64,
    pub fov_degrees: f64,
    #[serde(default)]
    pub diagnostic_mask: bool,
}
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct Uniform {
    pub forward: [f32; 4],
    pub right: [f32; 4],
    pub camera_up: [f32; 4],
    pub radial_up: [f32; 4],
    /// c, 1+eta, horizontal tangent half-FOV, vertical tangent half-FOV.
    pub shape: [f32; 4],
    pub extent: [f32; 4],
}
pub struct Prepared {
    pub camera_position: Position,
    pub radius_meters: f64,
    pub altitude_meters: f64,
    pub forward: DVec3,
    pub right: DVec3,
    pub camera_up: DVec3,
    pub radial_up: DVec3,
    pub half_fov_tangent: f64,
    pub aspect: f64,
    pub width: u32,
    pub height: u32,
    pub(crate) uniform: Uniform,
}
impl View {
    pub fn prepare(&self, document: &Document, width: u32, height: u32) -> Result<Prepared> {
        document.validate()?;
        if document.base_height_meters != 0. || !document.operations.is_empty() {
            return Err(invalid(
                "Reference sphere camera requires zero base height and no relief; use the mesh renderer for terrain",
            ));
        }
        if !(64..=2048).contains(&width)
            || !(64..=2048).contains(&height)
            || !self.pitch_degrees.is_finite()
            || !(-90. ..=90.).contains(&self.pitch_degrees)
            || !self.fov_degrees.is_finite()
            || !(10. ..=120.).contains(&self.fov_degrees)
            || !self.pose.altitude_meters.is_finite()
            || self.pose.altitude_meters < 0.01
        {
            return Err(invalid(
                "Planet reference needs dimensions64..2048, pitch[-90,90], FOV[10,120] and finite altitude>=0.01m",
            ));
        }
        let f = self.pose.frame()?;
        let radial_up = DVec3::from(f.up);
        let tangent = DVec3::from(f.forward);
        let right = DVec3::from(f.right);
        let (s, c) = self.pitch_degrees.to_radians().sin_cos();
        let forward = tangent * c + radial_up * s;
        let camera_up = radial_up * c - tangent * s;
        let eta = self.pose.altitude_meters / document.radius();
        let half_fov_tangent = (self.fov_degrees.to_radians() * 0.5).tan();
        let aspect = f64::from(width) / f64::from(height);
        let camera_position = document
            .center
            .translated((radial_up * (document.radius() + self.pose.altitude_meters)).to_array())
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Prepared {
            camera_position,
            radius_meters: document.radius(),
            altitude_meters: self.pose.altitude_meters,
            forward,
            right,
            camera_up,
            radial_up,
            half_fov_tangent,
            aspect,
            width,
            height,
            uniform: Uniform {
                forward: forward.as_vec3().extend(0.).to_array(),
                right: right.as_vec3().extend(0.).to_array(),
                camera_up: camera_up.as_vec3().extend(0.).to_array(),
                radial_up: radial_up.as_vec3().extend(0.).to_array(),
                shape: [
                    if eta < 1. {
                        (eta * (2. + eta)) as f32
                    } else {
                        0.
                    },
                    (1. + eta) as f32,
                    (half_fov_tangent * aspect) as f32,
                    half_fov_tangent as f32,
                ],
                extent: [
                    width as f32,
                    height as f32,
                    if self.diagnostic_mask { 1. } else { 0. },
                    if eta >= 1. { 1. } else { 0. },
                ],
            },
        })
    }
}
impl Prepared {
    pub fn ray(&self, x: f64, y: f64) -> DVec3 {
        let sx = (2. * x / f64::from(self.width) - 1.) * self.half_fov_tangent * self.aspect;
        let sy = (1. - 2. * y / f64::from(self.height)) * self.half_fov_tangent;
        (self.forward + self.right * sx + self.camera_up * sy).normalize()
    }
    /// Double-precision oracle in physical meters, independent of the GPU coefficients.
    pub fn hit_distance(&self, ray: DVec3) -> Option<f64> {
        if !ray.is_finite() || (ray.length_squared() - 1.).abs() > 1e-8 {
            return None;
        }
        let b = (self.radius_meters + self.altitude_meters) * self.radial_up.dot(ray);
        if b >= 0. {
            return None;
        }
        // Far from the sphere, use a perpendicular distance instead of subtracting
        // two squared astronomical distances to recover a much smaller radius.
        if self.altitude_meters >= self.radius_meters {
            let perpendicular =
                (self.radius_meters + self.altitude_meters) * self.radial_up.cross(ray).length();
            if perpendicular > self.radius_meters {
                return None;
            }
            return Some(
                -b - (self.radius_meters * self.radius_meters - perpendicular * perpendicular)
                    .max(0.)
                    .sqrt(),
            );
        }
        let c = self.altitude_meters * (2. * self.radius_meters + self.altitude_meters);
        let discriminant = b * b - c;
        (discriminant >= 0.).then(|| c / (-b + discriminant.sqrt()))
    }
    pub fn metadata(&self) -> Value {
        let h = self.altitude_meters;
        let r = self.radius_meters;
        let tangent = (h * (2. * r + h)).sqrt();
        json!({"camera_position":self.camera_position,"radius_meters":r,"altitude_meters":h,
            "camera_forward":self.forward.to_array(),"camera_up":self.camera_up.to_array(),"radial_up":self.radial_up.to_array(),
            "horizon_line_of_sight_meters":tangent,"horizon_surface_arc_meters":r*tangent.atan2(r),"horizon_dip_degrees":tangent.atan2(r).to_degrees(),
            "representation":"analytic_reference_sphere","whole_reference_sphere":true,"far_plane_meters":null,"resident_mesh_limit_applied":false,"reference_sphere_only":true,"terrain":false,"atmosphere":false,"refraction":false,"gameplay_simulated":false,"world_changed":false})
    }
}
/// Reuses the normal capture renderer's device and PNG readback. No second GPU device.
pub struct Gpu {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
}
impl Gpu {
    pub fn new(renderer: &crate::render::Renderer) -> Self {
        let device = &renderer.device;
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("planet reference camera"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("planet reference camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("planet reference camera"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("planet reference camera"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("planet reference sphere"),
            source: wgpu::ShaderSource::Wgsl(include_str!("reference.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("planet reference sphere"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.format,
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
        Self {
            pipeline,
            uniform,
            bind,
        }
    }
    pub fn capture(
        &self,
        renderer: &crate::render::Renderer,
        view: &Prepared,
    ) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(
            (64..=2048).contains(&view.width) && (64..=2048).contains(&view.height),
            "Planet capture dimensions require64..2048"
        );
        anyhow::ensure!(
            renderer.format == wgpu::TextureFormat::Rgba8UnormSrgb,
            "Planet capture requires RGBA renderer"
        );
        renderer
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&view.uniform));
        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("planet reference capture"),
            size: wgpu::Extent3d {
                width: view.width,
                height: view.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target = texture.create_view(&Default::default());
        let mut encoder = renderer.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("planet reference capture"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            pass.draw(0..3, 0..1);
        }
        renderer.queue.submit([encoder.finish()]);
        renderer.read_texture_png(&texture, view.width, view.height)
    }
}
