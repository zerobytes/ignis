// Rich material experiment only; both reference and candidates share these maps.
pub struct RichMaterial {
    pipeline: wgpu::RenderPipeline,
    group: wgpu::BindGroup,
    faces: Option<RichFaces>,
}
#[derive(Clone)]
struct RichFaces {
    layout: wgpu::BindGroupLayout,
    group: wgpu::BindGroup,
    face_count: u32,
}
struct RichInputs {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}
#[allow(dead_code)]
impl Gpu {
    /// Source-only diagnostic: primitive_index is relative to the single draw.
    pub fn install_rich_slopes(&mut self, slopes: &[[f32; 2]]) {
        self.install_rich_face_bytes(bytemuck::cast_slice(slopes), slopes.len() as u32);
    }
    pub fn install_rich_packed_slopes(&mut self, words: &[u32]) {
        assert!(words.len() > 1);
        self.install_rich_face_bytes(bytemuck::cast_slice(words), (words.len() - 1) as u32);
    }
    fn install_rich_face_bytes(&mut self, bytes: &[u8], face_count: u32) {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("compiled-face-slopes"),
                contents: bytes,
                usage: wgpu::BufferUsages::STORAGE,
            });
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compiled-face-slopes"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(8),
                    },
                    count: None,
                }],
            });
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compiled-face-slopes"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        self.rich_faces = Some(RichFaces {
            layout,
            group,
            face_count,
        });
    }
    pub fn upload_rich(&self, mesh: &Mesh) -> Uploaded {
        assert!(mesh.faces.iter().all(|f| f[3] == 0));
        let positions = mesh.vertices.iter().map(|v| v.p).collect::<Vec<_>>();
        let indices = mesh
            .faces
            .iter()
            .flat_map(|f| f[..3].iter().copied())
            .collect::<Vec<_>>();
        let v = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rich-positions"),
                contents: bytemuck::cast_slice(&positions),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let i = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rich-indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let end = indices.len() as u32;
        Uploaded {
            v,
            i,
            uv: None,
            ranges: [0..end, end..end, end..end],
            bytes: positions.len() * 12 + indices.len() * 4,
        }
    }
    pub fn install_rich_material(
        &mut self,
        color_size: u32,
        colors: &[Vec<u8>],
        normal_size: u32,
        normals: &[Vec<u8>],
    ) {
        self.install_rich_variant(color_size, colors, normal_size, normals, "implicit");
    }
    pub fn install_rich_variant(
        &mut self,
        color_size: u32,
        colors: &[Vec<u8>],
        normal_size: u32,
        normals: &[Vec<u8>],
        variant: &str,
    ) {
        self.rich_faces_enabled = false;
        let diagnostic = variant.ends_with("-inputs");
        let variant = variant.strip_suffix("-inputs").unwrap_or(variant);
        self.rich_inputs = diagnostic.then(|| {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("rich-derivatives-diagnostic"),
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = texture.create_view(&Default::default());
            RichInputs { texture, view }
        });
        let upload = |size: u32, levels: &[Vec<u8>]| {
            assert_eq!(levels.len(), size.ilog2() as usize + 1);
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("compiled-material-map"),
                size: wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: levels.len() as u32,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            for (mip, pixels) in levels.iter().enumerate() {
                let n = size >> mip;
                assert_eq!(pixels.len(), (n * n * 4) as usize);
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: mip as u32,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(n * 4),
                        rows_per_image: Some(n),
                    },
                    wgpu::Extent3d {
                        width: n,
                        height: n,
                        depth_or_array_layers: 1,
                    },
                );
            }
            texture.create_view(&Default::default())
        };
        let color = upload(color_size, colors);
        let normal = upload(normal_size, normals);
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    texture_entry(0),
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    texture_entry(2),
                ],
            });
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal),
                },
            ],
        });
        self.normal_pipeline = Some(self.rich_pipeline(&layout, variant, diagnostic));
        self.rich_faces_enabled = variant.starts_with("geometric-");
        self.normal_group = Some(group);
        self.normal_enabled = true;
    }
    /// Build a second shader over the SAME resident textures, outside timings.
    pub fn rich_variant(&self, variant: &str) -> RichMaterial {
        assert!(!variant.ends_with("-inputs"));
        let layout = self
            .normal_pipeline
            .as_ref()
            .unwrap()
            .get_bind_group_layout(1);
        RichMaterial {
            pipeline: self.rich_pipeline(&layout, variant, false),
            group: self.normal_group.as_ref().unwrap().clone(),
            faces: if variant.starts_with("geometric-") {
                self.rich_faces.clone()
            } else {
                None
            },
        }
    }
    pub fn select_rich_variant(&mut self, material: &RichMaterial) {
        self.rich_inputs = None;
        self.rich_faces_enabled = material.faces.is_some();
        self.rich_faces = material.faces.clone();
        self.normal_pipeline = Some(material.pipeline.clone());
        self.normal_group = Some(material.group.clone());
        self.normal_enabled = true;
    }
    fn rich_pipeline(
        &self,
        layout: &wgpu::BindGroupLayout,
        variant: &str,
        diagnostic: bool,
    ) -> wgpu::RenderPipeline {
        let globals = self.pipeline.get_bind_group_layout(0);
        let geometric = variant.starts_with("geometric-");
        let mut layouts = vec![&globals, layout];
        if geometric {
            layouts.push(
                &self
                    .rich_faces
                    .as_ref()
                    .expect("compiled faces installed")
                    .layout,
            );
        }
        let pl = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &layouts,
                push_constant_ranges: &[],
            });
        let source = include_str!("shader.wgsl");
        let source = match variant {
            "implicit" => source.to_owned(),
            "analytic-lod" | "analytic-manual" => {
                let manual = variant == "analytic-manual";
                let source = format!("{}\n{}\n{}", include_str!("texel.wgsl"), include_str!("analytic.wgsl"), source)
                    .replace("let material=", if manual {
                        "let tangent=texel_bilinear(normals,v.uv,0).xyz*2.-1.;let footprint=analytic_footprint(v.p,tangent);let material="
                    } else {
                        "let tangent=textureSampleLevel(normals,filtering,v.uv,0.).xyz*2.-1.;let footprint=analytic_footprint(v.p,tangent);let material="
                    });
                if manual {
                    source.replace("textureSample(colors,filtering,v.uv)", "texel_trilinear(colors,v.uv,footprint)")
                        .replace("textureSample(normals,filtering,v.uv)", "texel_trilinear(normals,v.uv,footprint)")
                } else {
                    source.replace("textureSample(colors,filtering,v.uv)", "textureSampleLevel(colors,filtering,v.uv,log2(max(footprint*f32(textureDimensions(colors).x),1e-10)))")
                        .replace("textureSample(normals,filtering,v.uv)", "textureSampleLevel(normals,filtering,v.uv,log2(max(footprint*f32(textureDimensions(normals).x),1e-10)))")
                }
            },
            "geometric-id" | "geometric-fetch" => {
                // Guards keep the index/read observable without changing valid source pixels.
                // This is a pipeline control, not an isolated instruction timer.
                let guard = if variant == "geometric-id" {
                    "if face >= arrayLength(&slopes) {discard;}"
                } else { "if any(abs(slopes[face]) > vec2(1e20)) {discard;}" };
                format!("@group(2) @binding(0) var<storage,read> slopes:array<vec2<f32>>;\n{}",source)
                    .replace("@builtin(front_facing) front:bool", "@builtin(front_facing) front:bool,@builtin(primitive_index) face:u32")
                    .replace("return vec4(material.rgb", &format!("{guard} return vec4(material.rgb"))
            },
            "geometric-screen" | "geometric-packed" => {
                let (binding, slope) = if variant == "geometric-packed" {
                    ("@group(2) @binding(0) var<storage,read> packed:array<u32>;", "unpack2x16snorm(packed[face+1u])*bitcast<f32>(packed[0])")
                } else { ("@group(2) @binding(0) var<storage,read> slopes:array<vec2<f32>>;", "slopes[face]") };
                format!("{}\n{}\n{}",binding,include_str!("screen.wgsl"),source)
                    .replace("@builtin(front_facing) front:bool", "@builtin(front_facing) front:bool,@builtin(primitive_index) face:u32")
                    .replace("let material=", &format!("let grad=screen_gradients(v.clip,{slope});let material="))
                    .replace("textureSample(colors,filtering,v.uv)", "textureSampleGrad(colors,filtering,v.uv,grad.xy,grad.zw)")
                    .replace("textureSample(normals,filtering,v.uv)", "textureSampleGrad(normals,filtering,v.uv,grad.xy,grad.zw)")
            },
            "geometric-lod" | "geometric-grad" => {
                let source = format!("{}\n{}", include_str!("geometric.wgsl"), source)
                    .replace("@builtin(front_facing) front:bool", "@builtin(front_facing) front:bool,@builtin(primitive_index) face:u32")
                    .replace("let material=", "let grad=geometric_gradients(v.p,slopes[face]);let material=");
                if variant == "geometric-grad" {
                    source.replace("textureSample(colors,filtering,v.uv)", "textureSampleGrad(colors,filtering,v.uv,grad.xy,grad.zw)")
                        .replace("textureSample(normals,filtering,v.uv)", "textureSampleGrad(normals,filtering,v.uv,grad.xy,grad.zw)")
                } else {
                    source.replace("let material=", "let footprint=max(length(grad.xy),length(grad.zw));let material=")
                        .replace("textureSample(colors,filtering,v.uv)", "textureSampleLevel(colors,filtering,v.uv,log2(max(footprint*f32(textureDimensions(colors).x),1e-10)))")
                        .replace("textureSample(normals,filtering,v.uv)", "textureSampleLevel(normals,filtering,v.uv,log2(max(footprint*f32(textureDimensions(normals).x),1e-10)))")
                }
            },
            "manual-trilinear" => format!("{}\n{}", include_str!("texel.wgsl"), source)
                .replace("let material=", "let dx=dpdxFine(v.uv);let dy=dpdyFine(v.uv);let footprint=max(length(dx),length(dy));let material=")
                .replace("textureSample(colors,filtering,v.uv)", "texel_trilinear(colors,v.uv,footprint)")
                .replace("textureSample(normals,filtering,v.uv)", "texel_trilinear(normals,v.uv,footprint)"),
            "fine" => source
                .replace(
                    "textureSample(colors,filtering,v.uv)",
                    "textureSampleGrad(colors,filtering,v.uv,dpdxFine(v.uv),dpdyFine(v.uv))",
                )
                .replace(
                    "textureSample(normals,filtering,v.uv)",
                    "textureSampleGrad(normals,filtering,v.uv,dpdxFine(v.uv),dpdyFine(v.uv))",
                ),
            "lod" => format!(
                "{}@fragment fn fs(v:V)->@location(0) vec4<f32> {{let dx=dpdxFine(v.uv);let dy=dpdyFine(v.uv);return vec4(dx,dy);}}",
                source.split("@fragment").next().unwrap()
            ),
            "explicit-lod" => source
                .replace("let material=", "let dx=dpdxFine(v.uv);let dy=dpdyFine(v.uv);let footprint=max(length(dx),length(dy));let material=")
                .replace("textureSample(colors,filtering,v.uv)", "textureSampleLevel(colors,filtering,v.uv,log2(max(footprint*f32(textureDimensions(colors).x),1e-10)))")
                .replace("textureSample(normals,filtering,v.uv)", "textureSampleLevel(normals,filtering,v.uv,log2(max(footprint*f32(textureDimensions(normals).x),1e-10)))"),
            "level0" => source
                .replace(
                    "textureSample(colors,filtering,v.uv)",
                    "textureSampleLevel(colors,filtering,v.uv,0.)",
                )
                .replace(
                    "textureSample(normals,filtering,v.uv)",
                    "textureSampleLevel(normals,filtering,v.uv,0.)",
                ),
            "uv" => format!(
                "{}@fragment fn fs(v:V)->@location(0) vec4<f32> {{return vec4(v.uv,0.,1.);}}",
                source.split("@fragment").next().unwrap()
            ),
            _ => panic!("unknown rich material variant"),
        };
        let source = if diagnostic {
            format!("struct Diagnostic {{@location(0) color:vec4<f32>,@location(1) derivatives:vec4<f32>}};\n{}", source)
                .replace("->@location(0) vec4<f32> {", "->Diagnostic {")
                .replace("return vec4(material.rgb*diffuse+vec3(spec),1.);", "return Diagnostic(vec4(material.rgb*diffuse+vec3(spec),1.),vec4(dpdxFine(v.uv),dpdyFine(v.uv)));")
        } else {
            source
        };
        let target = Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Rgba32Float,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        });
        let targets = [target.clone(), target];
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("rich-material"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 12,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0=>Float32x3],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &targets[..if diagnostic { 2 } else { 1 }],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
    }
    /// Read the attachment produced alongside the last color draw; no new draw.
    pub fn rich_derivatives(&self) -> Result<Vec<[f32; 4]>> {
        let input = self
            .rich_inputs
            .as_ref()
            .expect("input diagnostic installed");
        let stride = (self.width * 16).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rich-input-readback"),
            size: stride as u64 * self.height as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &input.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let bytes = self.mapped(&buffer)?;
        Ok(bytes
            .chunks_exact(stride as usize)
            .flat_map(|row| {
                row[..self.width as usize * 16].chunks_exact(16).map(|p| {
                    std::array::from_fn(|i| {
                        f32::from_le_bytes(p[i * 4..i * 4 + 4].try_into().unwrap())
                    })
                })
            })
            .collect())
    }
}
