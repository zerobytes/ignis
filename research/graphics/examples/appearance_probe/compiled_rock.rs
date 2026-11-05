//! Independent binary asset compiler/consumer and controlled renderer comparison.
use super::*;
#[path = "compiled_rock_data.rs"]
mod data;
use data::Asset;
const ROOT: &str = "artifacts/appearance-probe/compiled-rock";
const LOOKUP: &str = include_str!("compiled_rock.wgsl");
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LookupUniforms {
    inverse: [[f32; 4]; 4],
    eye: [f32; 4],
    center_extent: [f32; 4],
    right0: [f32; 4],
    up0: [f32; 4],
    view0: [f32; 4],
    right1: [f32; 4],
    up1: [f32; 4],
    view1: [f32; 4],
    rect: [f32; 4],
    grid: [u32; 4],
    indices: [u32; 4],
    weights_screen: [f32; 4],
}
struct Resident {
    asset: Asset,
    group: wgpu::BindGroup,
}
pub(super) struct Draw {
    uniform: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    resident: Vec<Resident>,
}
fn basis(angle: f32, elevation: f32) -> (Vec3, Vec3, Vec3) {
    let v = direction(angle, elevation);
    let r = Vec3::Y.cross(v).normalize();
    let u = v.cross(r);
    (r, u, v)
}
impl Draw {
    fn new(gpu: &Gpu, assets: Vec<Asset>) -> Self {
        let d = &gpu.device;
        let uniform = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("compiled rock frame"),
            size: std::mem::size_of::<LookupUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let entries: Vec<_> = (0..4)
            .map(|i| wgpu::BindGroupLayoutEntry {
                binding: i,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: if i == 0 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage { read_only: true }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });
        let resident = assets
            .into_iter()
            .map(|mut asset| {
                let buffers: Vec<_> = [&asset.directory, &asset.maps, &asset.responses]
                    .iter()
                    .map(|w| {
                        d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("compiled rock binary section"),
                            contents: bytemuck::cast_slice(w),
                            usage: wgpu::BufferUsages::STORAGE,
                        })
                    })
                    .collect();
                let entries = [
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buffers[0].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buffers[1].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: buffers[2].as_entire_binding(),
                    },
                ];
                let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &entries,
                });
                asset.directory = Vec::new();
                asset.maps = Vec::new();
                asset.responses = Vec::new();
                Resident { asset, group }
            })
            .collect();
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compiled response lookup; no BRDF"),
            source: wgpu::ShaderSource::Wgsl(LOOKUP.into()),
        });
        let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("compiled rock one pass"),
            layout: Some(&d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
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
        Self {
            uniform,
            pipeline,
            resident,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &self,
        gpu: &Gpu,
        e: &mut wgpu::CommandEncoder,
        target: &Target,
        p: Orbit,
        size: [u32; 2],
        level: usize,
        q: Option<u32>,
        jitter: [f32; 2],
    ) {
        let direct = level >= 2;
        let level = level % 2;
        let a = &self.resident[level].asset;
        assert_eq!(p.elevation, a.camera_elevation);
        assert_eq!(p.light_elevation, a.light_elevation);
        let base = globals(p, size, jitter);
        let matrix = Mat4::from_cols_array_2d(&base.matrix);
        let v = coordinate(p.camera, a.views);
        let v = if direct {
            v.round() % a.views as f32
        } else {
            v
        };
        let l = coordinate(p.light, a.lights);
        let l = if direct {
            l.round() % a.lights as f32
        } else {
            l
        };
        let v0 = v.floor() as u32;
        let v1 = (v0 + 1) % a.views;
        let l0 = l.floor() as u32;
        let (r0, u0, d0) = basis(v0 as f32 * 360. / a.views as f32, a.camera_elevation);
        let (r1, u1, d1) = basis(v1 as f32 * 360. / a.views as f32, a.camera_elevation);
        let mut min = glam::Vec2::ONE;
        let mut max = glam::Vec2::ZERO;
        for x in [-a.extent, a.extent] {
            for y in [-a.extent, a.extent] {
                for z in [-a.extent, a.extent] {
                    let p = matrix * (a.center + Vec3::new(x, y, z)).extend(1.);
                    let uv = glam::Vec2::new(p.x / p.w * 0.5 + 0.5, 0.5 - p.y / p.w * 0.5);
                    min = min.min(uv);
                    max = max.max(uv);
                }
            }
        }
        let u = LookupUniforms {
            inverse: base.inverse,
            eye: base.eye,
            center_extent: a.center.extend(a.extent).to_array(),
            right0: r0.extend(f32::from(direct)).to_array(),
            up0: u0.extend(0.).to_array(),
            view0: d0.extend(0.).to_array(),
            right1: r1.extend(0.).to_array(),
            up1: u1.extend(0.).to_array(),
            view1: d1.extend(0.).to_array(),
            rect: [min.x.max(0.), min.y.max(0.), max.x.min(1.), max.y.min(1.)],
            grid: [a.size, a.size / 8, a.lights, a.codes - 1],
            indices: [v0, v1, l0, (l0 + 1) % a.lights],
            weights_screen: [v.fract(), l.fract(), size[0] as f32, size[1] as f32],
        };
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        let mut pass = e.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("compiled lookup"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
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
            depth_stencil_attachment: None,
            timestamp_writes: q.map(|i| wgpu::RenderPassTimestampWrites {
                query_set: gpu.query.as_ref().unwrap(),
                beginning_of_pass_write_index: Some(i),
                end_of_pass_write_index: Some(i + 1),
            }),
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.resident[level].group, &[]);
        pass.draw(0..6, 0..1);
    }
}
fn dummy() -> Vec<Vec<Symbol>> {
    vec![vec![Symbol {
        center_radius: [0., 0., 0., 1.],
        normal_thickness: [0., 1., 0., 0.],
        color_area: [0.; 4],
    }]]
}
fn pose(scene: usize, frame: usize, distance: f32) -> Orbit {
    let a = (frame % 8) as f32 * 45.;
    Orbit {
        camera: 23.75 + if scene == 1 { 0. } else { a },
        light: 67.25 + if scene == 0 { 0. } else { a * 2. },
        elevation: 35.,
        light_elevation: 55.,
        distance,
    }
}
fn source() -> Result<(Vec<Vertex>, Vec<u32>)> {
    decode(
        &fs::read(Path::new(ASSETS).join("rock/vertices.bin"))?,
        &fs::read(Path::new(ASSETS).join("rock/indices.bin"))?,
    )
}
fn compile() -> Result<()> {
    let (v, i) = source()?;
    let mut report = vec![];
    let original =
        "return vec4<f32>(material(i.normal*select(-1.0,1.0,front),i.color,i.world),1.0);";
    ensure!(SHADER.contains(original), "material entry changed");
    let ns=SHADER.replace(original,"return vec4<f32>(normalize(i.normal)*select(-1.0,1.0,front),dot(i.world-vec3<f32>(0.,0.12,0.),normalize(g.eye.xyz-vec3<f32>(0.,0.12,0.)))+2.0);");
    let cs = SHADER.replace(original, "return vec4<f32>(i.color,1.0);");
    let mut nr = pollster::block_on(Renderer::new_shader(&v, &i, &dummy(), &ns))?;
    let mut cr = pollster::block_on(Renderer::new_shader(&v, &i, &dummy(), &cs))?;
    for (level, (size, views, lights, codes)) in [(128, 16, 16, 256), (256, 32, 32, 512)]
        .into_iter()
        .enumerate()
    {
        let start = Instant::now();
        let mut a = Asset {
            size,
            views,
            lights,
            codes,
            extent: 1.3,
            center: CENTER,
            camera_elevation: 35.,
            light_elevation: 55.,
            directory: vec![],
            maps: vec![],
            responses: vec![],
        };
        let mut error = 0f32;
        for view in 0..views {
            let p = Orbit {
                camera: view as f32 * 360. / views as f32,
                ..pose(0, 0, 4.)
            };
            let d = direction(p.camera, p.elevation);
            let matrix = Mat4::orthographic_rh(-a.extent, a.extent, -a.extent, a.extent, 10., 0.1)
                * Mat4::look_at_rh(CENTER + d * 4., CENTER, Vec3::Y);
            let mut u = globals(p, [size, size], [0.; 2]);
            u.matrix = matrix.to_cols_array_2d();
            u.inverse = matrix.inverse().to_cols_array_2d();
            nr.override_globals = Some(u);
            cr.override_globals = Some(u);
            let normals = nr.capture(p, [size, size], 0, [0.; 2])?;
            let colors = cr.capture(p, [size, size], 0, [0.; 2])?;
            error = error.max(data::append_view(&mut a, view, &normals, &colors)?);
        }
        let bytes = a.bytes();
        ensure!(Asset::load(&bytes)?.bytes() == bytes, "binary roundtrip");
        fs::write(Path::new(ROOT).join(format!("rock{level}.irk")), &bytes)?;
        report.push(json!({"level":level,"size":size,"views":views,"lights":lights,"codes":codes,"disk_bytes":bytes.len(),"gpu_payload_bytes":a.payload(),"directory_bytes":a.directory.len()*4,"map_bytes":a.maps.len()*4,"responses_bytes":a.responses.len()*4,"build_ms":start.elapsed().as_secs_f64()*1000.,"max_normal_or_color_group_error":error,"checksum":hash(&bytes)}));
        println!("compiled level{level}: {} bytes", bytes.len());
    }
    fs::write(
        Path::new(ROOT).join("build.json"),
        serde_json::to_vec_pretty(
            &json!({"levels":report,"source_bytes":v.len()*40+i.len()*4,"source_triangles":i.len()/3,"source_vertices":v.len(),"source_vertex_hash":hash(bytemuck::cast_slice(&v)),"source_index_hash":hash(bytemuck::cast_slice(&i)),"source_shader_hash":hash(SHADER.as_bytes()),"lookup_shader_hash":hash(LOOKUP.as_bytes()),"camera_elevation":35,"light_elevation":55,"build_includes":"GPU gbuffer captures, grouping, responses, serialization, validation and write; excludes shared device/pipeline initialization"}),
        )?,
    )?;
    Ok(())
}
fn load() -> Result<Vec<Asset>> {
    (0..2)
        .map(|level| Asset::load(&fs::read(Path::new(ROOT).join(format!("rock{level}.irk")))?))
        .collect()
}
fn consumer() -> Result<()> {
    // This branch never opens the source mesh, JSON metadata or compiler dictionaries.
    let start = Instant::now();
    let a = load()?;
    let load_ms = start.elapsed().as_secs_f64() * 1000.;
    let mut r = pollster::block_on(Renderer::new(&[], &[], &dummy()))?;
    let start = Instant::now();
    r.compiled = Some(Draw::new(&r.gpu, a));
    r.wait(r.gpu.queue.submit([]))?;
    let upload_ms = start.elapsed().as_secs_f64() * 1000.;
    let mut hashes = vec![];
    for mode in 7..11 {
        let image = r.capture(pose(2, 0, 4.), SIZES[1], mode, [0.; 2])?;
        hashes.push(hash(bytemuck::cast_slice(&image)));
        metrics::png(
            &Path::new(ROOT).join(format!("standalone{mode}.png")),
            &image,
            SIZES[1][0],
        )?;
    }
    fs::write(
        Path::new(ROOT).join("load.json"),
        serde_json::to_vec_pretty(
            &json!({"read_validate_ms":load_ms,"gpu_resources_upload_ms":upload_ms,"hashes":hashes,"source_mesh_read":false,"json_read":false,"runtime_uniform_bytes":std::mem::size_of::<LookupUniforms>()}),
        )?,
    )?;
    Ok(())
}
pub fn run(command: &str) -> Result<()> {
    fs::create_dir_all(ROOT)?;
    if command == "--compile-rock" {
        return compile();
    }
    if command == "--compiled-rock-load" {
        return consumer();
    }
    let assets = load()?;
    let (v, i) = source()?;
    let meta: serde_json::Value =
        serde_json::from_slice(&fs::read(Path::new(ROOT).join("build.json"))?)?;
    ensure!(
        meta["source_vertex_hash"] == json!(hash(bytemuck::cast_slice(&v)))
            && meta["source_index_hash"] == json!(hash(bytemuck::cast_slice(&i)))
            && meta["source_shader_hash"] == json!(hash(SHADER.as_bytes()))
            && meta["lookup_shader_hash"] == json!(hash(LOOKUP.as_bytes())),
        "stale experiment source; rebuild"
    );
    for (level, a) in assets.iter().enumerate() {
        ensure!(
            meta["levels"][level]["checksum"] == json!(hash(&a.bytes())),
            "stale compiled artifact"
        );
    }
    let mut r = pollster::block_on(Renderer::new(&v, &i, &dummy()))?;
    r.compiled = Some(Draw::new(&r.gpu, assets));
    let smoke = command.ends_with("smoke");
    let bench_only = command.ends_with("bench");
    if !bench_only {
        let mut rows = vec![];
        for (res, size) in SIZES.into_iter().enumerate() {
            if smoke && res == 0 {
                continue;
            }
            for (di, distance) in [4., 24., 96.].into_iter().enumerate() {
                if smoke && di > 0 {
                    continue;
                }
                for scene in 0..3 {
                    if smoke && scene > 0 {
                        continue;
                    }
                    let mut previous: Vec<Option<(Image, Image)>> = vec![None, None, None, None];
                    for frame in 0..9 {
                        if smoke && frame > 0 {
                            continue;
                        }
                        let p = pose(scene, frame, distance);
                        let id = format!("r{res}_d{di}_s{scene}_{frame}");
                        let full = r.capture(p, size, 0, [0.; 2])?;
                        metrics::png(&Path::new(ROOT).join(format!("{id}_0.png")), &full, size[0])?;
                        let mut checks = vec![];
                        for mode in 7..11 {
                            let image = r.capture(p, size, mode, [0.; 2])?;
                            metrics::png(
                                &Path::new(ROOT).join(format!("{id}_{mode}.png")),
                                &image,
                                size[0],
                            )?;
                            let mut q = quality(&full, &image);
                            q["metrics"] = serde_json::to_value(metrics::compare(&full, &image))?;
                            q["temporal_residual"] = json!(
                                previous[mode - 7]
                                    .as_ref()
                                    .map(|(a, b)| metrics::temporal(a, b, &full, &image))
                            );
                            previous[mode - 7] = Some((full.clone(), image));
                            checks.push(q);
                        }
                        rows.push(json!({"id":id,"resolution":res,"size":size,"distance_id":di,"scene":scene,"frame":frame,"pose":p,"checks":checks}));
                        fs::write(
                            Path::new(ROOT).join(if smoke { "smoke.json" } else { "quality.json" }),
                            serde_json::to_vec_pretty(&rows)?,
                        )?;
                        println!("compiled capture {id}");
                    }
                }
            }
        }
        if smoke {
            return Ok(());
        }
        let mut checks = vec![];
        for mode in [0, 7, 8, 9, 10] {
            let p = pose(2, 0, 4.);
            let a = r.capture(p, SIZES[1], mode, [0.; 2])?;
            ensure!(a == r.capture(p, SIZES[1], mode, [0.; 2])?, "repeat");
            ensure!(
                a == r.capture(
                    Orbit {
                        camera: p.camera + 360.,
                        light: p.light + 360.,
                        ..p
                    },
                    SIZES[1],
                    mode,
                    [0.; 2]
                )?,
                "periodic"
            );
            let b = r.capture(pose(2, 1, 4.), SIZES[1], mode, [0.; 2])?;
            ensure!(a != b, "motion has no effect");
            ensure!(a == r.capture(p, SIZES[1], mode, [0.; 2])?, "return");
            checks.push(json!({"mode":mode,"hash":hash(bytemuck::cast_slice(&a)),"repeat_wrap_return":true}));
        }
        fs::write(
            Path::new(ROOT).join("checks.json"),
            serde_json::to_vec_pretty(&checks)?,
        )?;
        let mut integrated = vec![];
        for (res, size) in SIZES.into_iter().enumerate() {
            for (di, distance) in [4., 24., 96.].into_iter().enumerate() {
                let p = pose(2, 0, distance);
                let a = r.integrated(p, size, 16)?;
                let b = r.integrated(p, size, 64)?;
                metrics::png(
                    &Path::new(ROOT).join(format!("integrated_r{res}_d{di}.png")),
                    &b,
                    size[0],
                )?;
                let mut methods = vec![];
                for mode in [0, 7, 8, 9, 10] {
                    let c = r.capture(p, size, mode, [0.; 2])?;
                    methods.push(json!({"mode":mode,"error":metrics::compare(&b,&c)}));
                }
                integrated.push(json!({"resolution":res,"distance_id":di,"convergence16_64":metrics::compare(&b,&a),"methods":methods}));
                println!("integrated {res}/{di}");
            }
        }
        fs::write(
            Path::new(ROOT).join("integrated.json"),
            serde_json::to_vec_pretty(&integrated)?,
        )?;
    }
    let mut timings = vec![];
    for repeat in 0..2 {
        for (res, size) in SIZES.into_iter().enumerate() {
            for (di, distance) in [4., 24., 96.].into_iter().enumerate() {
                for frame in [0, 3, 6] {
                    if repeat == 1 && frame != 0 {
                        continue;
                    }
                    let p = pose(2, frame, distance);
                    let samples = r.bench_modes(p, size, &[0, 7, 8, 9, 10])?;
                    timings.push(json!({"repeat":repeat,"resolution":res,"distance_id":di,"frame":frame,"pose":p,"samples":samples}));
                    fs::write(
                        Path::new(ROOT).join("timings.json"),
                        serde_json::to_vec_pretty(&timings)?,
                    )?;
                    println!("compiled bench {repeat}/{res}/{di}/{frame}");
                }
            }
        }
    }
    Ok(())
}
