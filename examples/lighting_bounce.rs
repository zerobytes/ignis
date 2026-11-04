//! Explicit one-bounce RGB laboratory; no production lighting toggle.
use anyhow::{Result, ensure};
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use ignis::{
    engine::Engine,
    model::{Camera, DirectionalShadow, RoomRecipe},
    render::Renderer,
};
use serde_json::json;
use std::{
    fs,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
const W: u32 = 480;
const H: u32 = 320;
const NAMES: [&str; 9] = [
    "native",
    "strat16",
    "strat64",
    "coarse",
    "coarse-visible",
    "fine",
    "fine-visible",
    "reference-a",
    "reference-b",
];
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    inv: [[f32; 4]; 4],
    eye: [f32; 4],
    dims: [u32; 4],
    sun: [f32; 4],
    energy: [f32; 4],
    grid: [f32; 4],
    counts: [u32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BoxData {
    lo: [f32; 4],
    hi: [f32; 4],
    color: [f32; 4],
}
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.
}
fn wait(r: &Renderer) -> Result<()> {
    r.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(60)),
    })?;
    Ok(())
}
fn buffer(r: &Renderer, label: &str, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    r.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}
fn read(r: &Renderer, b: &wgpu::Buffer, size: u64) -> Result<Vec<u8>> {
    let slice = b.slice(0..size);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |v| {
        let _ = tx.send(v);
    });
    wait(r)?;
    rx.recv_timeout(Duration::from_secs(60))??;
    let map = slice.get_mapped_range();
    let result = map.to_vec();
    drop(map);
    b.unmap();
    Ok(result)
}
fn grid(p: &mut Params, fine: bool) {
    p.grid = [-5., 0.125, -5., if fine { 0.5 } else { 1. }];
    p.counts[..3].copy_from_slice(if fine { &[21, 9, 21] } else { &[11, 5, 11] });
}
fn nodes(fine: bool) -> u64 {
    if fine { 21 * 9 * 21 } else { 11 * 5 * 11 }
}
struct Lab {
    uniform: wgpu::Buffer,
    receivers: wgpu::Buffer,
    values: wgpu::Buffer,
    staging: wgpu::Buffer,
    probes: [wgpu::Buffer; 2],
    groups: [wgpu::BindGroup; 2],
    primary: wgpu::ComputePipeline,
    build: wgpu::ComputePipeline,
    shade: wgpu::ComputePipeline,
    queries: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    timing: wgpu::Buffer,
}
impl Lab {
    fn new(
        r: &Renderer,
        p: &Params,
        boxes: &[BoxData],
        native: &wgpu::TextureView,
        target: &wgpu::TextureView,
    ) -> Self {
        let uniform = r
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bounce parameters"),
                contents: bytemuck::bytes_of(p),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let geometry = r
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bounce boxes"),
                contents: bytemuck::cast_slice(boxes),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let receivers = buffer(
            r,
            "visible RGB receivers",
            u64::from(W) * u64::from(H) * 48,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        );
        let bytes = u64::from(W) * u64::from(H) * 16;
        let values = buffer(
            r,
            "unclamped RGB indirect",
            bytes,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let staging = buffer(
            r,
            "RGB readback",
            bytes,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let probes = [false, true].map(|fine| {
            buffer(
                r,
                "six-axis incident probes",
                nodes(fine) * 96,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            )
        });
        let entries: Vec<_> = (0..7)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                count: None,
                ty: match binding {
                    0 => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    1 => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    3 => wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    4 => wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    _ => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
            })
            .collect();
        let layout = r
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            });
        let pl = r
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            });
        let shader = r.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("one bounce transport"),
            source: wgpu::ShaderSource::Wgsl(include_str!("lighting_bounce/transport.wgsl").into()),
        });
        let pipeline = |entry| {
            r.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(entry),
                    layout: Some(&pl),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: None,
                })
        };
        let groups = std::array::from_fn(|i| {
            r.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: geometry.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: receivers.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(native),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(target),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: values.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: probes[i].as_entire_binding(),
                    },
                ],
            })
        });
        Self {
            uniform,
            receivers,
            values,
            staging,
            probes,
            groups,
            primary: pipeline("primary"),
            build: pipeline("build"),
            shade: pipeline("shade"),
            queries: r.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: None,
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            }),
            resolve: buffer(
                r,
                "bounce timestamp resolve",
                256,
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            timing: buffer(
                r,
                "bounce timestamp readback",
                16,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            ),
        }
    }
    fn run(&self, r: &Renderer, p: &Params, stage: &str) -> Result<(f64, f64)> {
        let t = Instant::now();
        r.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(p));
        let fine = p.grid[3] < 0.75;
        let group = &self.groups[usize::from(fine)];
        let mut e = r.device.create_command_encoder(&Default::default());
        {
            let mut pass = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(stage),
                timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                    query_set: &self.queries,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
            });
            pass.set_pipeline(match stage {
                "primary" => &self.primary,
                "build" => &self.build,
                _ => &self.shade,
            });
            pass.set_bind_group(0, group, &[]);
            if stage == "build" {
                pass.dispatch_workgroups((nodes(fine) as u32 * 6).div_ceil(64), 1, 1);
            } else {
                pass.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
            }
        }
        r.queue.submit([e.finish()]);
        let cpu = ms(t);
        wait(r)?;
        let mut e = r.device.create_command_encoder(&Default::default());
        e.resolve_query_set(&self.queries, 0..2, &self.resolve, 0);
        e.copy_buffer_to_buffer(&self.resolve, 0, &self.timing, 0, 16);
        r.queue.submit([e.finish()]);
        let bytes = read(r, &self.timing, 16)?;
        let a = u64::from_le_bytes(bytes[..8].try_into()?);
        let b = u64::from_le_bytes(bytes[8..].try_into()?);
        static LAST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let old = LAST.swap(b, std::sync::atomic::Ordering::Relaxed);
        ensure!(a >= old && b > a, "Stale RGB GPU timestamps");
        Ok((
            cpu,
            (b - a) as f64 * r.queue.get_timestamp_period() as f64 / 1e6,
        ))
    }
    fn values(&self, r: &Renderer) -> Result<Vec<u8>> {
        let size = u64::from(W) * u64::from(H) * 16;
        let mut e = r.device.create_command_encoder(&Default::default());
        e.copy_buffer_to_buffer(&self.values, 0, &self.staging, 0, size);
        r.queue.submit([e.finish()]);
        let raw = read(r, &self.staging, size)?;
        ensure!(
            raw.chunks_exact(4)
                .all(|b| f32::from_le_bytes(b.try_into().unwrap()).is_finite()),
            "Nonfinite RGB estimate"
        );
        Ok(raw)
    }
    fn save_values(&self, r: &Renderer, path: &str) -> Result<()> {
        fs::write(path, self.values(r)?)?;
        Ok(())
    }
}
fn main() -> Result<()> {
    for arg in std::env::args().skip(1) {
        ensure!(arg == "--thin-wall", "Unknown one-bounce argument: {arg}");
    }
    let thin = std::env::args().any(|s| s == "--thin-wall");
    let dir = if thin {
        "artifacts/lighting-bounce/thin-wall"
    } else {
        "artifacts/lighting-bounce/open"
    };
    fs::create_dir_all(dir)?;
    let mut scene = RoomRecipe::default().generate()?;
    let original = scene.entities[0].clone();
    scene.entities.clear();
    scene.recipe = None;
    scene.name = "RGB one-bounce laboratory".into();
    scene.camera = Camera {
        position: [7., 7., 12.],
        target: [0., 1., -0.5],
        fov_degrees: 43.,
    };
    scene.light.direction = [-0.5, -0.82, -0.35];
    scene.light.ambient = 0.;
    scene.light.direct_intensity = 1.1;
    scene.light.direct_color = [1., 0.93, 0.84];
    scene.light.environment = None;
    scene.light.atmosphere = None;
    scene.light.shadow = Some(DirectionalShadow {
        center: [0., 1., 0.],
        extent: 12.,
        resolution: 2048,
        bias: 0.0005,
        receiver_plane_bias: false,
    });
    let mut objects = vec![
        ([0., -0.2, 0.], [10., 0.4, 10.], [0.65, 0.65, 0.65]),
        ([0., 2., -4.], [8., 4., 0.25], [0.03, 0.10, 0.75]),
        ([-4., 2., 0.], [0.25, 4., 8.], [0.75, 0.06, 0.03]),
        ([3.5, 1.5, -2.], [0.25, 3., 4.], [0.65, 0.65, 0.65]),
        ([0., 2.8, -1.2], [5., 0.3, 3.5], [0.6, 0.6, 0.6]),
        ([-2., 1.3, -1.], [0.5, 2.6, 0.5], [0.5, 0.5, 0.5]),
        ([2., 1.3, -1.], [0.5, 2.6, 0.5], [0.5, 0.5, 0.5]),
        ([1.2, 0.45, 2.], [1.2, 0.9, 1.], [0.6, 0.6, 0.6]),
    ];
    if thin {
        objects.push(([0.27, 0.9, 1.], [0.08, 1.8, 4.], [0.65, 0.65, 0.65]));
    }
    let mut boxes = vec![];
    for (j, (p, s, c)) in objects.into_iter().enumerate() {
        let mut entity = original.clone();
        entity.id = format!("rgb_{j}");
        entity.body = None;
        entity.transform.position = p;
        entity.transform.size = s;
        entity.color = c;
        scene.entities.push(entity);
        boxes.push(BoxData {
            lo: [p[0] - s[0] / 2., p[1] - s[1] / 2., p[2] - s[2] / 2., 0.],
            hi: [p[0] + s[0] / 2., p[1] + s[1] / 2., p[2] + s[2] / 2., 0.],
            color: [c[0], c[1], c[2], 0.],
        });
    }
    fs::write(
        format!("{dir}/scene.json"),
        serde_json::to_vec_pretty(&scene)?,
    )?;
    let engine = Engine::new(scene)?;
    let mut frame = engine.render_frame();
    frame.suppress_diagnostic_grid = true;
    let instance = wgpu::Instance::default();
    let normal = {
        let mut r = pollster::block_on(Renderer::new(&instance, None))?;
        r.capture(&frame, W, H)?
    };
    let mut r = pollster::block_on(Renderer::new_profiled(&instance, None))?;
    let profile = r.capture(&frame, W, H)?;
    ensure!(normal == profile, "RGB profiling changed native pixels");
    ensure!(r.read_gpu_timing()?.is_some(), "RGB GPU timers unavailable");
    fs::write(format!("{dir}/native.png"), profile)?;
    let size = wgpu::Extent3d {
        width: W,
        height: H,
        depth_or_array_layers: 1,
    };
    let texture = |format, usage| {
        r.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let native = texture(
        r.format,
        wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
    );
    let target = texture(
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let nv = native.create_view(&Default::default());
    let tv = target.create_view(&Default::default());
    let sun = -Vec3::from(frame.light.direction).normalize();
    let mut p = Params {
        inv: ignis::picking::view_projection(&frame.camera, W, H)
            .inverse()
            .to_cols_array_2d(),
        eye: [7., 7., 12., 1.],
        dims: [W, H, boxes.len() as u32, 0],
        sun: sun.extend(0.).to_array(),
        energy: [1.1, 1.1 * 0.93, 1.1 * 0.84, 0.],
        grid: [0.; 4],
        counts: [0; 4],
    };
    grid(&mut p, false);
    let t = Instant::now();
    let lab = Lab::new(&r, &p, &boxes, &nv, &tv);
    let resource_cpu = ms(t);
    let primary = lab.run(&r, &p, "primary")?;
    let recv_size = u64::from(W) * u64::from(H) * 48;
    let staging = buffer(
        &r,
        "receiver audit",
        recv_size,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    );
    let mut e = r.device.create_command_encoder(&Default::default());
    e.copy_buffer_to_buffer(&lab.receivers, 0, &staging, 0, recv_size);
    r.queue.submit([e.finish()]);
    let receiver_raw = read(&r, &staging, recv_size)?;
    fs::write(format!("{dir}/receivers.bin"), &receiver_raw)?;
    let mut builds = vec![];
    for fine in [false, true] {
        grid(&mut p, fine);
        p.counts[3] = 0;
        let cost = lab.run(&r, &p, "build")?;
        builds.push(json!({"fine":fine,"seed":0,"cpu_ms":cost.0,"gpu_ms":cost.1}));
    }
    r.draw(&frame, &nv, W, H)?;
    r.read_gpu_timing()?;
    for mode in 1..7 {
        p.dims[3] = mode;
        grid(&mut p, mode >= 5);
        p.counts[3] = 0;
        for _ in 0..4 {
            lab.run(&r, &p, "shade")?;
        }
    }
    let mut rows = vec![];
    for round in 0..2 {
        for index in 0..64 {
            for offset in 0..7 {
                let mode = (index + if round == 0 { offset } else { 6 - offset }) % 7;
                let t = Instant::now();
                let mut f = engine.render_frame();
                f.suppress_diagnostic_grid = true;
                let prep = ms(t);
                let t = Instant::now();
                r.draw(&f, &nv, W, H)?;
                let raster_cpu = ms(t);
                let raster_gpu = r.read_gpu_timing()?.unwrap()["sum_ms"].as_f64().unwrap();
                p.dims[3] = mode;
                grid(&mut p, mode >= 5);
                p.counts[3] = index;
                let cost = if mode == 0 {
                    (0., 0.)
                } else {
                    lab.run(&r, &p, "shade")?
                };
                rows.push(json!({"round":round,"frame":index,"mode":mode,"name":NAMES[mode as usize],"prepare_cpu_ms":prep,"raster_cpu_ms":raster_cpu,"raster_gpu_ms":raster_gpu,"compute_cpu_ms":cost.0,"compute_gpu_ms":cost.1}));
            }
            if index % 16 == 15 {
                eprintln!("RGB round{round} frame{index}");
            }
        }
    }
    // Four independent field-build/query seeds: construction never hides in query timing.
    for seed in 0..4 {
        if seed > 0 {
            for fine in [false, true] {
                grid(&mut p, fine);
                p.counts[3] = seed;
                let cost = lab.run(&r, &p, "build")?;
                builds.push(json!({"fine":fine,"seed":seed,"cpu_ms":cost.0,"gpu_ms":cost.1}));
            }
        }
        for mode in 1..7 {
            grid(&mut p, mode >= 5);
            p.dims[3] = mode;
            p.counts[3] = [7, 23, 39, 55][seed as usize];
            lab.run(&r, &p, "shade")?;
            lab.save_values(&r, &format!("{dir}/quality-{mode}-{seed}.rgba32f"))?;
            if seed == 0 {
                fs::write(
                    format!("{dir}/{mode}-{}.png", NAMES[mode as usize]),
                    r.read_texture_png(&target, W, H)?,
                )?;
            }
        }
    }
    let mut references = vec![];
    for (mode, seed) in [(7, 91), (8, 193)] {
        grid(&mut p, false);
        p.dims[3] = mode;
        p.counts[3] = seed;
        let cost = lab.run(&r, &p, "shade")?;
        lab.save_values(&r, &format!("{dir}/reference-{mode}.rgba32f"))?;
        fs::write(
            format!("{dir}/{mode}-{}.png", NAMES[mode as usize]),
            r.read_texture_png(&target, W, H)?,
        )?;
        references.push(json!({"mode":mode,"seed":seed,"cpu_ms":cost.0,"gpu_ms":cost.1}));
    }
    // Retain the last field seed for independent CPU interpolation/visibility checks.
    for fine in [false, true] {
        let size = nodes(fine) * 96;
        let staging = buffer(
            &r,
            "probe field audit",
            size,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let mut e = r.device.create_command_encoder(&Default::default());
        e.copy_buffer_to_buffer(&lab.probes[usize::from(fine)], 0, &staging, 0, size);
        r.queue.submit([e.finish()]);
        fs::write(
            format!(
                "{dir}/probe-field-{}.rgba32f",
                if fine { "fine" } else { "coarse" }
            ),
            read(&r, &staging, size)?,
        )?;
    }
    // Rebuild timing has its own repeated study; four quality seeds are not enough.
    for sample in 0..32 {
        for fine in if sample % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            grid(&mut p, fine);
            p.counts[3] = 1000 + sample;
            let cost = lab.run(&r, &p, "build")?;
            builds.push(json!({"fine":fine,"seed":1000+sample,"calibration":true,"cpu_ms":cost.0,"gpu_ms":cost.1}));
        }
    }
    // Zero contribution reproduces the exact existing raster bytes.
    p.dims[3] = 0;
    lab.run(&r, &p, "shade")?;
    ensure!(
        r.read_texture_png(&target, W, H)? == r.read_texture_png(&native, W, H)?,
        "RGB passthrough mismatch"
    );
    let original_energy = p.energy;
    p.energy = [0.; 4];
    p.dims[3] = 1;
    lab.run(&r, &p, "shade")?;
    let zero = lab.values(&r)?;
    ensure!(
        zero.chunks_exact(4)
            .all(|b| f32::from_le_bytes(b.try_into().unwrap()) == 0.),
        "Zero sunlight produced indirect energy"
    );
    grid(&mut p, false);
    lab.run(&r, &p, "build")?;
    p.dims[3] = 4;
    lab.run(&r, &p, "shade")?;
    let zero_probe = lab.values(&r)?;
    ensure!(
        zero_probe.chunks_exact(16).all(|b| b[..12]
            .chunks_exact(4)
            .all(|c| f32::from_le_bytes(c.try_into().unwrap()) == 0.)),
        "Rebuilt zero-light probes produced RGB"
    );
    p.energy = [1., 0., 0., 0.];
    p.dims[3] = 2;
    lab.run(&r, &p, "shade")?;
    let red = lab.values(&r)?;
    ensure!(
        red.chunks_exact(16).all(|b| b[4..12]
            .chunks_exact(4)
            .all(|c| f32::from_le_bytes(c.try_into().unwrap()) == 0.)),
        "Sun channels coupled"
    );
    // A synthetic constant field tests interpolation and all six normal selections.
    let mut synthetic = Vec::<f32>::new();
    for normal in [
        [1., 0., 0.],
        [-1., 0., 0.],
        [0., 1., 0.],
        [0., -1., 0.],
        [0., 0., 1.],
        [0., 0., -1.],
    ] {
        synthetic.extend_from_slice(&[
            0., 1., 0., 1., normal[0], normal[1], normal[2], 0., 0.2, 0.3, 0.4, 0.,
        ]);
    }
    r.queue
        .write_buffer(&lab.receivers, 0, bytemuck::cast_slice(&synthetic));
    let constants: Vec<[f32; 4]> = (0..nodes(false) * 6)
        .map(|i| [0.01 * ((i % 6 + 1) as f32), 0.02, 0.03, 1.])
        .collect();
    r.queue
        .write_buffer(&lab.probes[0], 0, bytemuck::cast_slice(&constants));
    p.energy = original_energy;
    p.dims[3] = 3;
    lab.run(&r, &p, "shade")?;
    fs::write(format!("{dir}/constant-field.rgba32f"), lab.values(&r)?)?;
    fs::write(
        format!("{dir}/results.json"),
        serde_json::to_vec_pretty(
            &json!({"width":W,"height":H,"thin_wall":thin,"adapter":r.adapter.get_info().name,"backend":format!("{:?}",r.adapter.get_info().backend),"resource_setup_cpu_ms":resource_cpu,"primary_cpu_ms":primary.0,"primary_gpu_ms":primary.1,"receiver_bytes":recv_size,"probe_bytes":[nodes(false)*96,nodes(true)*96],"probe_nodes":[nodes(false),nodes(true)],"builds":builds,"references":references,"native_profile_identity":true,"passthrough_identity":true,"zero_sun_direct":true,"zero_sun_rebuilt_probe":true,"channel_isolation":true,"monotonic_gpu_timestamps":true,"rows":rows}),
        )?,
    )?;
    Ok(())
}
