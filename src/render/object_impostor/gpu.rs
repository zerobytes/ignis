use super::*;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Job {
    world: [[f32; 4]; 4],
    eye: [f32; 4],
    bounds: [f32; 4],
    control: [u32; 4],
    mix_lod: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Palette {
    ratio: [f32; 3],
    material: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Parameters {
    grid: [u32; 4],
    pixel: [f32; 4],
    elevations: [f32; 4],
}
#[derive(Default, serde::Serialize)]
pub struct Stats {
    pub objects: usize,
    pub parts: usize,
    pub shadow_objects: usize,
    pub shadow_source_triangles: usize,
    pub resident_models: usize,
    pub source_triangles: usize,
    pub quad_triangles: usize,
    pub gpu_atlas_bytes: usize,
    pub maximum_projected_pixels: f32,
    pub cpu_prepare_ms: f64,
}
struct Prepared {
    world: Mat4,
    inverse: Mat4,
    center: Vec3,
    radius: f32,
    entry: usize,
    members: Vec<usize>,
    palette: Vec<Palette>,
    source_triangles: usize,
    stable_id: u32,
}
pub(crate) struct State {
    pack: Arc<Pack>,
    pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    bind: wgpu::BindGroup,
    base: wgpu::TextureView,
    normal: wgpu::TextureView,
    depth: wgpu::TextureView,
    sampler: wgpu::Sampler,
    jobs: wgpu::Buffer,
    palette: wgpu::Buffer,
    params: wgpu::Buffer,
    assets: Vec<crate::meshes::Asset>,
    ready: Vec<Option<Vec<usize>>>,
    hidden: Vec<bool>,
    hidden_shadow: Vec<bool>,
    instance_slots: BTreeMap<String, usize>,
    instances_source: Vec<crate::meshes::Instance>,
    groups_source: Arc<Vec<crate::mesh_objects::Object>>,
    prepared: Vec<Prepared>,
    resident_entries: usize,
    pub shadows_enabled: bool,
    pub maximum_shadow_pixels: f32,
    saved_masks: Vec<(usize, u8)>,
    pub stats: Stats,
    pub maximum_pixels: f32,
}
impl State {
    pub fn new(r: &Renderer, pack: Arc<Pack>, maximum_pixels: f32) -> Result<Self> {
        pack.validate()?;
        ensure!(
            maximum_pixels.is_finite() && (16. ..=1024.).contains(&maximum_pixels),
            "Invalid atlas pixel threshold"
        );
        let m = &pack.metadata;
        let resident_entries = m
            .entries
            .len()
            .min(r.device.limits().max_texture_array_layers as usize / m.views());
        ensure!(
            resident_entries > 0,
            "Device cannot hold one object view grid"
        );
        let count = (resident_entries * m.views()) as u32;
        ensure!(
            pack.layers.len() == m.entries.len() * m.views(),
            "Atlas layer mismatch"
        );
        let formats = [
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::R16Uint,
        ];
        let mut textures = vec![];
        for format in formats {
            textures.push(r.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("resident object appearance atlas"),
                size: wgpu::Extent3d {
                    width: m.resolution,
                    height: m.resolution,
                    depth_or_array_layers: count,
                },
                mip_level_count: m.levels(),
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }));
        }
        for (layer, data) in pack.layers.iter().take(count as usize).enumerate() {
            for (level, data) in data.mips.iter().enumerate() {
                let side = m.resolution >> level;
                for (n, bytes) in [&data.base, &data.normal_part, &data.depth]
                    .into_iter()
                    .enumerate()
                {
                    r.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &textures[n],
                            mip_level: level as u32,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: layer as u32,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        bytes,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(side * if n == 2 { 2 } else { 4 }),
                            rows_per_image: Some(side),
                        },
                        wgpu::Extent3d {
                            width: side,
                            height: side,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }
        }
        let views: Vec<_> = textures
            .iter()
            .map(|t| {
                t.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                })
            })
            .collect();
        let sampler = r.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let entries: Vec<_> = (0..7)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: match binding {
                    0 | 1 => wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    2 => wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    3 => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    4 | 5 => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    _ => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                count: None,
            })
            .collect();
        let layout = r
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("object atlas scene data"),
                entries: &entries,
            });
        let jobs = buffer(&r.device, 128, wgpu::BufferUsages::STORAGE);
        let palette = buffer(&r.device, 16, wgpu::BufferUsages::STORAGE);
        let params = buffer(&r.device, 48, wgpu::BufferUsages::UNIFORM);
        let bind = bind(
            &r.device, &layout, &views[0], &views[1], &views[2], &sampler, &jobs, &palette, &params,
        );
        let shader = r.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("depth reprojected object atlas"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}",
                    include_str!("../../scene.wgsl"),
                    include_str!("impostor.wgsl")
                )
                .into(),
            ),
        });
        let scene_layout = r.pipeline.get_bind_group_layout(0);
        let pipeline_layout = r
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&scene_layout, &layout],
                push_constant_ranges: &[],
            });
        let pipeline = r
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("object atlas coverage composite"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_object_atlas"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_object_atlas"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[("uniform_material", 1.)],
                        ..Default::default()
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: r.format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });
        let shadow_scene_layout = r.shadow_pipeline.get_bind_group_layout(0);
        let shadow_layout = r
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("object atlas shadow layout"),
                bind_group_layouts: &[&shadow_scene_layout, &layout],
                push_constant_ranges: &[],
            });
        let shadow_pipeline = r
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("object atlas shadow"),
                layout: Some(&shadow_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_object_shadow"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_object_shadow"),
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
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });
        let mut views = views.into_iter();
        let base = views.next().unwrap();
        let normal = views.next().unwrap();
        let depth = views.next().unwrap();
        let stats = Stats {
            gpu_atlas_bytes: m.payload_bytes()? * resident_entries / m.entries.len(),
            resident_models: resident_entries,
            ..Default::default()
        };
        Ok(Self {
            pack,
            pipeline,
            shadow_pipeline,
            layout,
            bind,
            base,
            normal,
            depth,
            sampler,
            jobs,
            palette,
            params,
            assets: vec![],
            ready: vec![],
            hidden: vec![],
            hidden_shadow: vec![],
            instance_slots: Default::default(),
            instances_source: vec![],
            groups_source: Default::default(),
            prepared: vec![],
            resident_entries,
            shadows_enabled: false,
            maximum_shadow_pixels: 512.,
            saved_masks: vec![],
            stats,
            maximum_pixels,
        })
    }
    pub fn same_pack(&self, pack: &Arc<Pack>) -> bool {
        Arc::ptr_eq(&self.pack, pack)
    }
    fn rebuild_objects(&mut self, doc: &crate::meshes::Document) {
        self.instance_slots = doc
            .instances
            .iter()
            .enumerate()
            .map(|(n, i)| (i.id.clone(), n))
            .collect();
        self.hidden = vec![false; doc.instances.len()];
        self.hidden_shadow = vec![false; doc.instances.len()];
        self.prepared.clear();
        let signatures: BTreeMap<_, _> = self
            .pack
            .metadata
            .entries
            .iter()
            .enumerate()
            .filter(|(n, _)| self.ready[*n].is_some())
            .map(|(n, e)| {
                (
                    e.parts.iter().map(|p| p.asset.as_str()).collect::<Vec<_>>(),
                    n,
                )
            })
            .collect();
        for (stable_id, object) in doc.objects.iter().enumerate() {
            let Some(members) = object
                .members
                .iter()
                .map(|m| self.instance_slots.get(m).copied())
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            if members.is_empty() {
                continue;
            }
            let parts = members
                .iter()
                .map(|&n| &doc.instances[n])
                .collect::<Vec<_>>();
            let key = parts.iter().map(|p| p.asset.as_str()).collect::<Vec<_>>();
            let Some(&entry) = signatures.get(&key) else {
                continue;
            };
            let root = parts[0];
            if parts.iter().any(|i| {
                i.position != root.position
                    || i.rotation != root.rotation
                    || i.scale != root.scale
                    || i.playback.is_some()
                    || i.attachment.is_some()
            }) {
                continue;
            }
            let e = &self.pack.metadata.entries[entry];
            let mut palette = vec![];
            let mut valid = true;
            for ((i, p), &slot) in parts
                .iter()
                .zip(&e.parts)
                .zip(self.ready[entry].as_ref().unwrap())
            {
                let mut ratio = [1.; 3];
                for (channel, value) in ratio.iter_mut().enumerate() {
                    if p.tint[channel] == 0. {
                        if i.color[channel] != 0. {
                            valid = false;
                            break;
                        }
                    } else {
                        *value = i.color[channel] / p.tint[channel];
                    }
                }
                palette.push(Palette {
                    ratio,
                    material: (slot + 1) as u32,
                });
            }
            if !valid {
                continue;
            }
            let world = world(root);
            let center = world.transform_point3(Vec3::from(e.center));
            let radius = e.radius * root.scale.abs();
            let source_triangles = self.ready[entry]
                .as_ref()
                .unwrap()
                .iter()
                .map(|&n| doc.assets[n].geometry.triangles.len())
                .sum();
            self.prepared.push(Prepared {
                world,
                inverse: world.inverse(),
                center,
                radius,
                entry,
                members,
                palette,
                source_triangles,
                stable_id: stable_id as u32,
            });
        }
        self.instances_source = doc.instances.clone();
        self.groups_source = doc.objects.clone();
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        f: &RenderFrame,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let started = std::time::Instant::now();
        self.hidden.fill(false);
        self.hidden_shadow.fill(false);
        let bytes = self.stats.gpu_atlas_bytes;
        self.stats = Stats {
            gpu_atlas_bytes: bytes,
            resident_models: self.resident_entries,
            ..Default::default()
        };
        if f.clip.is_some()
            || f.planet_air.is_some()
            || f.selection.is_some()
            || f.selection_mesh.is_some()
            || !f.meshes.proxy_compatible()
            || f.meshes
                .source
                .assets
                .iter()
                .any(|a| a.material.has_coverage() || a.material.transmission > 0.)
        {
            return Ok(());
        }
        let doc = &f.meshes.source;
        let changed = self.assets.len() != doc.assets.len()
            || self.assets.iter().zip(&doc.assets).any(|(a, b)| {
                a.id != b.id
                    || !Arc::ptr_eq(&a.geometry, &b.geometry)
                    || a.material != b.material
                    || a.shading != b.shading
            });
        if changed {
            self.assets = doc.assets.clone();
            self.ready.clear();
            let mut digests = BTreeMap::new();
            for (entry, e) in self.pack.metadata.entries.iter().enumerate() {
                if entry >= self.resident_entries {
                    self.ready.push(None);
                    continue;
                }
                let mut slots = vec![];
                let mut valid = true;
                for p in &e.parts {
                    let Some(slot) = doc.assets.iter().position(|a| a.id == p.asset) else {
                        valid = false;
                        break;
                    };
                    let a = &doc.assets[slot];
                    if !eligible(a)
                        || *digests
                            .entry(slot)
                            .or_insert_with(|| crate::distant::digest(a))
                            != p.digest
                    {
                        valid = false;
                        break;
                    }
                    slots.push(slot);
                }
                self.ready.push(valid.then_some(slots));
            }
        }
        if changed
            || !Arc::ptr_eq(&self.groups_source, &doc.objects)
            || self.instances_source != doc.instances
        {
            self.rebuild_objects(doc);
        }
        let eye = Vec3::from(f.camera.position);
        let forward = (Vec3::from(f.camera.target) - eye).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward);
        let focal = height as f32 / (2. * (f.camera.fov_degrees.to_radians() * 0.5).tan());
        let mut camera = vec![];
        let mut shadow = vec![];
        let mut palette = vec![];
        let shadow_vp = super::super::shadow_view_projection(&f.light);
        for p in &self.prepared {
            let e = &self.pack.metadata.entries[p.entry];
            let m = &self.pack.metadata;
            let delta = p.center - eye;
            let z = delta.dot(forward);
            let mut base_job = Job {
                world: p.world.to_cols_array_2d(),
                eye: p.inverse.transform_point3(eye).extend(0.).to_array(),
                bounds: [e.center[0], e.center[1], e.center[2], e.radius],
                control: [palette.len() as u32, 0, 0, p.stable_id],
                mix_lod: [0.; 4],
            };
            let mut any = false;
            if z > p.radius + 0.05
                && delta.dot(right).abs() <= (z + p.radius) * width as f32 / (2. * focal) + p.radius
                && delta.dot(up).abs() <= (z + p.radius) * height as f32 / (2. * focal) + p.radius
            {
                let projected = 2. * p.radius * focal / (z - p.radius)
                    * (1. + delta.cross(forward).length() / z);
                let view_delta = Vec3::from_slice(&base_job.eye[..3]) - Vec3::from(e.center);
                if projected <= self.maximum_pixels
                    && select_view(&mut base_job, m, p.entry, view_delta, projected)
                {
                    camera.push((z, base_job));
                    for &n in &p.members {
                        self.hidden[n] = true;
                    }
                    self.stats.parts += p.members.len();
                    self.stats.source_triangles += p.source_triangles;
                    self.stats.maximum_projected_pixels =
                        self.stats.maximum_projected_pixels.max(projected);
                    any = true;
                }
            }
            if self.shadows_enabled
                && let (Some(light), Some(vp)) = (&f.light.shadow, shadow_vp)
            {
                let projected = p.radius * light.resolution as f32 / light.extent;
                if projected <= self.maximum_shadow_pixels && sphere_visible(p.center, p.radius, vp)
                {
                    let direction = p.inverse.transform_vector3(-Vec3::from(f.light.direction));
                    let mut job = base_job;
                    if select_view(&mut job, m, p.entry, direction, projected) {
                        shadow.push(job);
                        for &n in &p.members {
                            self.hidden_shadow[n] = true;
                        }
                        self.stats.shadow_source_triangles += p.source_triangles;
                        any = true;
                    }
                }
            }
            if any {
                palette.extend_from_slice(&p.palette);
            }
        }
        camera.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut jobs = camera.into_iter().map(|p| p.1).collect::<Vec<_>>();
        self.stats.objects = jobs.len();
        self.stats.shadow_objects = shadow.len();
        jobs.extend(shadow);
        self.stats.quad_triangles = jobs.len() * 2;
        if !jobs.is_empty() {
            let mut rebound = false;
            for (target, raw) in [
                (&mut self.jobs, bytemuck::cast_slice::<Job, u8>(&jobs)),
                (
                    &mut self.palette,
                    bytemuck::cast_slice::<Palette, u8>(&palette),
                ),
            ] {
                if raw.len() as u64 > target.size() {
                    ensure!(
                        raw.len() as u64
                            <= u64::from(device.limits().max_storage_buffer_binding_size),
                        "Object jobs exceed device binding"
                    );
                    *target = buffer(
                        device,
                        (raw.len() as u64).next_power_of_two(),
                        wgpu::BufferUsages::STORAGE,
                    );
                    rebound = true;
                }
                queue.write_buffer(target, 0, raw);
            }
            if rebound {
                self.bind = bind(
                    device,
                    &self.layout,
                    &self.base,
                    &self.normal,
                    &self.depth,
                    &self.sampler,
                    &self.jobs,
                    &self.palette,
                    &self.params,
                );
            }
            let m = &self.pack.metadata;
            let mut elevations = [0.; 4];
            elevations[..m.elevations.len()].copy_from_slice(&m.elevations);
            let params = Parameters {
                grid: [
                    m.resolution,
                    m.azimuths,
                    m.elevations.len() as u32,
                    m.levels(),
                ],
                pixel: [2. / width as f32, -2. / height as f32, -1., 1.],
                elevations,
            };
            queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
        }
        self.stats.cpu_prepare_ms = started.elapsed().as_secs_f64() * 1000.;
        Ok(())
    }
    pub fn mask(&mut self, visibility: &mut crate::render_visibility::Visibility, f: &RenderFrame) {
        self.saved_masks.clear();
        for (index, (draw, mask)) in f.meshes.draws.iter().zip(&mut visibility.masks).enumerate() {
            if let Some(&slot) = self.instance_slots.get(&draw.instance) {
                let old = *mask;
                if self.hidden.get(slot) == Some(&true) {
                    *mask &= !crate::render_visibility::CAMERA;
                }
                if self.hidden_shadow.get(slot) == Some(&true) {
                    *mask &= !crate::render_visibility::SHADOW;
                }
                if old != *mask {
                    self.saved_masks.push((index, old));
                }
            }
        }
    }
    pub fn restore(&mut self, visibility: &mut crate::render_visibility::Visibility) {
        for (index, mask) in self.saved_masks.drain(..) {
            visibility.masks[index] = mask;
        }
    }
    pub fn draw_shadow(&self, pass: &mut wgpu::RenderPass<'_>, scene: &wgpu::BindGroup) {
        if self.stats.shadow_objects == 0 {
            return;
        }
        pass.set_pipeline(&self.shadow_pipeline);
        pass.set_bind_group(0, scene, &[]);
        pass.set_bind_group(1, &self.bind, &[]);
        pass.draw(
            0..6,
            self.stats.objects as u32..(self.stats.objects + self.stats.shadow_objects) as u32,
        );
    }
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, scene: &wgpu::BindGroup) {
        if self.stats.objects == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene, &[]);
        pass.set_bind_group(1, &self.bind, &[]);
        pass.draw(0..6, 0..self.stats.objects as u32);
    }
}
fn buffer(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("object atlas commands"),
        size,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
#[allow(clippy::too_many_arguments)]
fn bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    base: &wgpu::TextureView,
    normal: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    jobs: &wgpu::Buffer,
    palette: &wgpu::Buffer,
    params: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("object appearance/depth atlas"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(base),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(normal),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(depth),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: jobs.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: palette.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: params.as_entire_binding(),
            },
        ],
    })
}

fn select_view(job: &mut Job, m: &Metadata, entry: usize, v: Vec3, projected: f32) -> bool {
    let elevation = (v.y / v.length()).clamp(-1., 1.).asin();
    if elevation < m.elevations[0] - 0.15 || elevation > m.elevations[m.elevations.len() - 1] + 0.15
    {
        return false;
    }
    let row = m
        .elevations
        .iter()
        .enumerate()
        .min_by(|a, b| (a.1 - elevation).abs().total_cmp(&(b.1 - elevation).abs()))
        .unwrap()
        .0;
    let az = v.x.atan2(v.z).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU
        * m.azimuths as f32;
    let first = az.floor() as u32;
    let base = (entry * m.views() + row * m.azimuths as usize) as u32;
    job.control[1] = base + first;
    job.control[2] = base + (first + 1) % m.azimuths;
    job.mix_lod = [
        az.fract(),
        (m.resolution as f32 / projected).log2().max(0.),
        0.,
        0.,
    ];
    true
}
fn sphere_visible(center: Vec3, radius: f32, vp: Mat4) -> bool {
    let m = vp.transpose();
    [
        m.w_axis + m.x_axis,
        m.w_axis - m.x_axis,
        m.w_axis + m.y_axis,
        m.w_axis - m.y_axis,
        m.z_axis,
        m.w_axis - m.z_axis,
    ]
    .into_iter()
    .all(|p| p.truncate().dot(center) + p.w + radius * p.truncate().length() >= -0.001)
}
