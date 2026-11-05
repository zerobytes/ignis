use super::*;
use wgpu::util::DeviceExt;

fn frame(doc: &crate::meshes::Document) -> Result<RenderFrame> {
    let s = crate::model::RoomRecipe::default().generate()?;
    let mut light = s.light;
    light.ambient = 1.;
    light.direct_intensity = 0.;
    light.environment = None;
    light.shadow = None;
    light.atmosphere = None;
    let mut f = RenderFrame {
        clip: None,
        planet_air: None,
        suppress_diagnostic_grid: true,
        meshes: Default::default(),
        particles: Default::default(),
        distant_meshes: Default::default(),
        mesh_lods: Default::default(),
        mesh_lod_settings: Default::default(),
        camera: s.camera,
        light,
        boxes: vec![],
        player_primitives: Vec::new(),
        vision_guides: vec![],
        hearing_guides: vec![],
        noise_guides: vec![],
        route_guides: vec![],
        truncated_navigation_paths: 0,
        selection: None,
        selection_mesh: None,
    };
    let mut d = doc.clone();
    d.instances.clear();
    d.objects = Default::default();
    f.meshes.update(&d, 0);
    Ok(f)
}
fn linear(v: u8) -> f32 {
    let v = v as f32 / 255.;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(v: f32) -> u8 {
    let v = v.clamp(0., 1.);
    ((if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1. / 2.4) - 0.055
    }) * 255.)
        .round() as u8
}

/// Compile explicit co-transforming static objects. Unsupported objects are refused
/// here; runtime source fallback never changes the authored object count.
pub fn compile(
    renderer: &mut Renderer,
    doc: &crate::meshes::Document,
    objects: &[String],
    resolution: u32,
) -> Result<Pack> {
    ensure!(!objects.is_empty(), "No object models supplied");
    doc.validate().map_err(|e| anyhow::anyhow!(e.message))?;
    let mut metadata = Metadata {
        resolution,
        azimuths: 12,
        elevations: vec![-0.35, 0.175, 0.7, 1.22],
        entries: vec![],
    };
    let mut meshes = vec![];
    for name in objects {
        let object = doc
            .objects
            .iter()
            .find(|o| &o.id == name)
            .context("Missing object")?;
        ensure!(
            object.members.len() <= 256,
            "Object requires a wider part encoding; keep source"
        );
        let root = doc
            .instances
            .iter()
            .find(|i| i.id == object.members[0])
            .context("Missing object root")?;
        let mut parts = vec![];
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = -lo;
        for id in &object.members {
            let i = doc
                .instances
                .iter()
                .find(|i| &i.id == id)
                .context("Missing object member")?;
            ensure!(
                i.position == root.position
                    && i.rotation == root.rotation
                    && i.scale == root.scale
                    && i.playback.is_none()
                    && i.attachment.is_none(),
                "Object parts must co-transform statically"
            );
            let slot = doc
                .assets
                .iter()
                .position(|a| a.id == i.asset)
                .context("Missing object asset")?;
            let a = &doc.assets[slot];
            ensure!(eligible(a), "Unsupported source uses geometry");
            let g = &a.geometry;
            let mut normals = vec![Vec3::ZERO; g.positions.len()];
            crate::mesh_normals::accumulate(&g.triangles, &g.positions, &mut normals);
            let base = vertices.len() as u32;
            for (p, n) in g.positions.iter().zip(&normals) {
                let p3 = Vec3::from(*p);
                lo = lo.min(p3);
                hi = hi.max(p3);
                let normal = if *n == Vec3::ZERO { Vec3::Y } else { *n };
                vertices.push(Vertex {
                    position: *p,
                    normal: normal.to_array(),
                    color: i.color,
                    rest_position: *p,
                    material: (slot + 1) as u32,
                    material_projection_weights: normal.abs().to_array(),
                });
            }
            for triangle in &g.triangles {
                let [a, b, c] = triangle.map(|i| Vec3::from(g.positions[i as usize]));
                let face = (b - a).cross(c - a).normalize();
                for &i in triangle {
                    if normals[i as usize] == Vec3::ZERO {
                        let mut v = vertices[(base + i) as usize];
                        v.normal = face.to_array();
                        indices.push(vertices.len() as u32);
                        vertices.push(v);
                    } else {
                        indices.push(base + i);
                    }
                }
            }
            parts.push(Part {
                asset: a.id.clone(),
                digest: crate::distant::digest(a),
                tint: i.color,
            });
        }
        let center = (lo + hi) * 0.5;
        let radius = (hi - lo).length() * 0.505;
        metadata.entries.push(Entry {
            parts,
            center: center.to_array(),
            radius,
        });
        meshes.push((vertices, indices));
    }
    metadata.payload_bytes()?;
    let mut f = frame(doc)?;
    let size = resolution * 4;
    ensure!(
        size <= renderer.device.limits().max_texture_dimension_2d,
        "Build raster exceeds device resolution"
    );
    let formats = [
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Rgba32Float,
        wgpu::TextureFormat::R32Float,
        wgpu::TextureFormat::R32Uint,
    ];
    let textures: Vec<_> = formats
        .iter()
        .map(|format| {
            renderer.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("build-time object surface raster"),
                size: wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: *format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        })
        .collect();
    let targets: Vec<_> = formats
        .iter()
        .map(|format| {
            Some(wgpu::ColorTargetState {
                format: *format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        })
        .collect();
    let views: Vec<_> = textures
        .iter()
        .map(|t| t.create_view(&Default::default()))
        .collect();
    let depth = renderer
        .device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("build-time object depth"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default());
    let scene_layout = renderer.pipeline.get_bind_group_layout(0);
    let bake_layout = renderer
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
    let settings = renderer.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bake_bind = renderer
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bake_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: settings.as_entire_binding(),
            }],
        });
    let layout = renderer
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&scene_layout, &bake_layout],
            push_constant_ranges: &[],
        });
    let shader = renderer
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("canonical material surface bake"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}",
                    include_str!("../../scene.wgsl"),
                    include_str!("bake.wgsl")
                )
                .into(),
            ),
        });
    let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Float32x3,4=>Uint32,5=>Float32x3];
    let pipeline = renderer
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("canonical object MRT bake"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_object_bake"),
                compilation_options: Default::default(),
                targets: &targets,
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
    let dummy = renderer
        .device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default());
    let plane = size as u64 * size as u64;
    let lengths = [plane * 4, plane * 16, plane * 4, plane * 4];
    let readback = renderer.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("object bake readback"),
        size: lengths.iter().sum(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut layers = vec![];
    for (entry, (vertices, indices)) in metadata.entries.iter().zip(meshes) {
        let vb = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("canonical source vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ib = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("canonical source indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let center = Vec3::from(entry.center);
        let radius = entry.radius;
        let part_map: BTreeMap<_, _> = entry
            .parts
            .iter()
            .enumerate()
            .map(|(n, p)| {
                (
                    (doc.assets.iter().position(|a| a.id == p.asset).unwrap() + 1) as u32,
                    n,
                )
            })
            .collect();
        for &elevation in &metadata.elevations {
            for az in 0..metadata.azimuths {
                let (_, up, forward) = basis(
                    az as f32 * std::f32::consts::TAU / metadata.azimuths as f32,
                    elevation,
                );
                let eye = center - forward * radius * 3.;
                f.camera.position = eye.to_array();
                f.camera.target = center.to_array();
                f.camera.fov_degrees = 45.;
                renderer.draw(&f, &dummy, 64, 64)?;
                let vp = Mat4::orthographic_rh(
                    -radius,
                    radius,
                    -radius,
                    radius,
                    radius * 0.01,
                    radius * 6.,
                ) * Mat4::look_at_rh(eye, center, up);
                renderer.queue.write_buffer(
                    &renderer.uniform,
                    0,
                    bytemuck::cast_slice(&vp.to_cols_array()),
                );
                renderer.queue.write_buffer(
                    &settings,
                    0,
                    bytemuck::cast_slice(&[center.x, center.y, center.z, radius]),
                );
                let attachments: Vec<_> = views
                    .iter()
                    .map(|v| {
                        Some(wgpu::RenderPassColorAttachment {
                            view: v,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })
                    })
                    .collect();
                let mut encoder = renderer.device.create_command_encoder(&Default::default());
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("object surface compile"),
                        color_attachments: &attachments,
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &renderer.bind_group, &[]);
                    pass.set_bind_group(1, &bake_bind, &[]);
                    pass.set_vertex_buffer(0, vb.slice(..));
                    pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }
                let mut offset = 0;
                for ((texture, format), length) in textures.iter().zip(&formats).zip(lengths) {
                    let bpp = if *format == wgpu::TextureFormat::Rgba32Float {
                        16
                    } else {
                        4
                    };
                    encoder.copy_texture_to_buffer(
                        wgpu::TexelCopyTextureInfo {
                            texture,
                            mip_level: 0,
                            origin: Default::default(),
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyBufferInfo {
                            buffer: &readback,
                            layout: wgpu::TexelCopyBufferLayout {
                                offset,
                                bytes_per_row: Some(size * bpp),
                                rows_per_image: Some(size),
                            },
                        },
                        wgpu::Extent3d {
                            width: size,
                            height: size,
                            depth_or_array_layers: 1,
                        },
                    );
                    offset += length;
                }
                renderer.queue.submit([encoder.finish()]);
                let (tx, rx) = std::sync::mpsc::channel();
                readback.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                    let _ = tx.send(r);
                });
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                rx.recv()??;
                let bytes = readback.slice(..).get_mapped_range();
                let base = &bytes[..lengths[0] as usize];
                let normals = &bytes[lengths[0] as usize..(lengths[0] + lengths[1]) as usize];
                let depths = &bytes[(lengths[0] + lengths[1]) as usize
                    ..(lengths[0] + lengths[1] + lengths[2]) as usize];
                let parts = &bytes[(lengths[0] + lengths[1] + lengths[2]) as usize..];
                let mut samples = Vec::with_capacity((size * size) as usize);
                for n in 0..(size * size) as usize {
                    let alpha = base[n * 4 + 3] as f32 / 255.;
                    let read = |data: &[u8], at: usize| {
                        f32::from_le_bytes(data[at..at + 4].try_into().unwrap())
                    };
                    let id = u32::from_le_bytes(parts[n * 4..n * 4 + 4].try_into().unwrap());
                    samples.push(Sample {
                        color: Vec3::new(
                            linear(base[n * 4]),
                            linear(base[n * 4 + 1]),
                            linear(base[n * 4 + 2]),
                        ),
                        normal: Vec3::new(
                            read(normals, n * 16),
                            read(normals, n * 16 + 4),
                            read(normals, n * 16 + 8),
                        ),
                        alpha,
                        depth: read(depths, n * 4),
                        part: part_map.get(&id).copied().unwrap_or(0),
                    });
                }
                drop(bytes);
                readback.unmap();
                let mut side = size;
                while side > resolution {
                    samples = downsample(&samples, side);
                    side /= 2;
                }
                let mut mips = vec![];
                loop {
                    mips.push(encode(&samples));
                    if side == 1 {
                        break;
                    }
                    samples = downsample(&samples, side);
                    side /= 2;
                }
                layers.push(Layer { mips });
            }
        }
        println!(
            "compiled object atlas: {} parts / {} source triangles",
            entry.parts.len(),
            indices.len() / 3
        );
    }
    Ok(Pack { metadata, layers })
}
#[derive(Clone, Copy)]
struct Sample {
    color: Vec3,
    normal: Vec3,
    alpha: f32,
    depth: f32,
    part: usize,
}
fn downsample(source: &[Sample], side: u32) -> Vec<Sample> {
    let mut out = Vec::with_capacity((side * side / 4) as usize);
    for y in 0..side / 2 {
        for x in 0..side / 2 {
            let a = [
                source[(y * 2 * side + x * 2) as usize],
                source[(y * 2 * side + x * 2 + 1) as usize],
                source[((y * 2 + 1) * side + x * 2) as usize],
                source[((y * 2 + 1) * side + x * 2 + 1) as usize],
            ];
            let weight = a.iter().map(|s| s.alpha).sum::<f32>();
            let denom = weight.max(1e-12);
            let mut color = Vec3::ZERO;
            let mut normal = Vec3::ZERO;
            let mut depth = 0.;
            let mut votes = BTreeMap::new();
            for s in a {
                color += s.color * s.alpha;
                normal += s.normal * s.alpha;
                depth += s.depth * s.alpha;
                *votes.entry(s.part).or_insert(0.) += s.alpha;
            }
            let part = votes
                .into_iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map_or(0, |a| a.0);
            out.push(Sample {
                color: color / denom,
                normal: normal.normalize_or_zero(),
                alpha: weight / 4.,
                depth: depth / denom,
                part,
            });
        }
    }
    out
}
fn encode(samples: &[Sample]) -> Mip {
    let mut base = vec![];
    let mut normal_part = vec![];
    let mut depth = vec![];
    for s in samples {
        base.extend([
            srgb(s.color.x * s.alpha),
            srgb(s.color.y * s.alpha),
            srgb(s.color.z * s.alpha),
            (s.alpha * 255.).round() as u8,
        ]);
        let n = s.normal * (127. * s.alpha) + Vec3::splat(128.);
        normal_part.extend([
            n.x.round() as u8,
            n.y.round() as u8,
            n.z.round() as u8,
            s.part as u8,
        ]);
        depth.extend(((s.depth.clamp(0., 1.) * 65535.).round() as u16).to_le_bytes());
    }
    Mip {
        base,
        normal_part,
        depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filtered_coverage_keeps_straight_colour_and_packs_premultiplied_radiance() {
        let covered = Sample {
            color: Vec3::new(0.8, 0.2, 0.05),
            normal: Vec3::Y,
            alpha: 1.,
            depth: 0.7,
            part: 0,
        };
        let empty = Sample {
            color: Vec3::ZERO,
            normal: Vec3::ZERO,
            alpha: 0.,
            depth: 0.,
            part: 0,
        };
        let filtered = downsample(&[covered, empty, empty, empty], 2);
        assert!((filtered[0].alpha - 0.25).abs() < 1e-6);
        assert!((filtered[0].color - covered.color).length() < 1e-6);
        assert!((filtered[0].depth - 0.7).abs() < 1e-6);
        let encoded = encode(&filtered);
        let alpha = encoded.base[3] as f32 / 255.;
        assert!((linear(encoded.base[0]) / alpha - 0.8).abs() < 0.02);
        assert!(encoded.normal_part[1] > 128);
        let empty = encode(&[empty]);
        assert_eq!(&empty.normal_part[..3], &[128, 128, 128]);
    }
}
