//! Native raster + explicit diagnostic GPU ambient-visibility estimators.
use anyhow::{Result, ensure};
use bytemuck::{Pod, Zeroable};
use ignis::{
    engine::Engine,
    model::{Camera, RoomRecipe},
    render::Renderer,
};
use serde_json::json;
use std::{
    fs,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
const W: u32 = 640;
const H: u32 = 416;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    inv: [[f32; 4]; 4],
    eye: [f32; 4],
    dims: [u32; 4],
    light: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BoxData {
    lo: [f32; 4],
    hi: [f32; 4],
    color: [f32; 4],
}
fn elapsed(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.
}
fn wait(r: &Renderer) -> Result<()> {
    r.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(30)),
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
    rx.recv_timeout(Duration::from_secs(30))??;
    let m = slice.get_mapped_range();
    let v = m.to_vec();
    drop(m);
    b.unmap();
    Ok(v)
}
fn main() -> Result<()> {
    let audit = std::env::args().any(|a| a == "--visibility-audit");
    let guard = std::env::args().any(|a| a == "--offscreen-guard");
    ensure!(!guard || audit, "Offscreen guard requires visibility audit");
    let dir = if guard {
        "artifacts/lighting-visibility/guard"
    } else if audit {
        "artifacts/lighting-visibility/gallery"
    } else {
        "artifacts/lighting-gallery"
    };
    fs::create_dir_all(dir)?;
    let mut scene = RoomRecipe::default().generate()?;
    let prototype = scene.entities[0].clone();
    scene.entities.clear();
    scene.recipe = None;
    scene.camera = Camera {
        position: [8., 6.3, 10.],
        target: [0., 1., -0.5],
        fov_degrees: 43.,
    };
    scene.light.direction = [0.5, -0.82, -0.35];
    scene.light.ambient = 0.3;
    scene.light.direct_intensity = 1.1;
    scene.light.direct_color = [1., 0.93, 0.84];
    scene.light.environment = None;
    scene.light.atmosphere = None;
    scene.light.shadow = Some(ignis::model::DirectionalShadow {
        center: [0., 1., 0.],
        extent: 12.,
        resolution: 2048,
        bias: 0.0005,
        receiver_plane_bias: false,
    });
    if guard {
        scene.camera = Camera {
            position: [0., 1.2, 0.],
            target: [0., 0.5, -3.],
            fov_degrees: 60.,
        };
    }
    let objects = if guard {
        vec![
            ([0., -0.2, -2.], [8., 0.4, 8.], [0.65, 0.64, 0.59]),
            ([0., 1., -5.], [8., 2., 0.2], [0.48, 0.22, 0.14]),
            ([0., 1., 0.6], [3., 2., 0.2], [0.18, 0.42, 0.18]),
        ]
    } else {
        vec![
            ([0., -0.2, 0.], [11., 0.4, 10.], [0.65, 0.64, 0.59]),
            ([0., 2., -4.], [11., 4., 0.3], [0.66, 0.65, 0.59]),
            ([-5., 2., 0.], [0.3, 4., 8.], [0.48, 0.22, 0.14]),
            ([-2.3, 1.6, -1.4], [1., 3.2, 1.], [0.7, 0.58, 0.35]),
            ([1.2, 1.6, -1.4], [1., 3.2, 1.], [0.7, 0.58, 0.35]),
            ([-0.55, 3., -1.4], [4.5, 0.5, 1.2], [0.7, 0.58, 0.35]),
            ([-0.5, 0.4, 1.], [3.8, 0.8, 2.2], [0.55, 0.54, 0.5]),
            ([-1.1, 1.15, 1.], [1.2, 0.7, 1.2], [0.08, 0.25, 0.55]),
            ([2.7, 0.3, 1.5], [1.6, 0.6, 1.8], [0.6, 0.23, 0.09]),
            ([2.7, 0.75, 0.9], [1.6, 0.3, 0.6], [0.6, 0.23, 0.09]),
            ([-3.5, 0.6, 2.3], [0.7, 1.2, 0.7], [0.18, 0.42, 0.18]),
        ]
    };
    let mut boxes = Vec::new();
    for (j, (p, s, c)) in objects.into_iter().enumerate() {
        let mut e = prototype.clone();
        e.id = format!("gallery_{j}");
        e.body = None;
        e.transform.position = p;
        e.transform.size = s;
        e.color = c;
        scene.entities.push(e);
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
    let native_default = {
        let mut r = pollster::block_on(Renderer::new(&instance, None))?;
        ensure!(
            !r.device
                .features()
                .contains(wgpu::Features::TIMESTAMP_QUERY)
        );
        r.capture(&frame, W, H)?
    };
    let start = Instant::now();
    let mut r = pollster::block_on(Renderer::new_profiled(&instance, None))?;
    let renderer_setup = elapsed(start);
    let profiled = r.capture(&frame, W, H)?;
    ensure!(native_default == profiled, "Profiling changed native image");
    fs::write(format!("{dir}/00-native.png"), &profiled)?;
    ensure!(
        r.read_gpu_timing()?.is_some(),
        "GPU timestamp queries unavailable"
    );
    let start = Instant::now();
    let make_texture = |format, usage| {
        r.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gallery target"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let native = make_texture(
        r.format,
        wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
    );
    let target = make_texture(
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let nv = native.create_view(&Default::default());
    let tv = target.create_view(&Default::default());
    let mut params = Params {
        inv: ignis::picking::view_projection(&frame.camera, W, H)
            .inverse()
            .to_cols_array_2d(),
        eye: [
            frame.camera.position[0],
            frame.camera.position[1],
            frame.camera.position[2],
            1.,
        ],
        dims: [W, H, boxes.len() as u32, 0],
        light: [frame.light.ambient, 0., 0., 0.],
    };
    let uniform = r
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gallery parameters"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    let geometry = r
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gallery bounds"),
            contents: bytemuck::cast_slice(&boxes),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let receiver_bytes = u64::from(W) * u64::from(H) * 160;
    let receivers = buffer(
        &r,
        "receivers and angular cache",
        receiver_bytes,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let estimate_bytes = u64::from(W) * u64::from(H) * 4;
    let estimates = buffer(
        &r,
        "raw signed AO",
        estimate_bytes,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let raw = buffer(
        &r,
        "AO readback",
        estimate_bytes,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    );
    let shader = r.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("laboratory estimators"),
        source: wgpu::ShaderSource::Wgsl(include_str!("lighting_gallery/estimators.wgsl").into()),
    });
    let entries = [
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
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
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
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ];
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
    let setup = pipeline("setup");
    let shade = pipeline("shade");
    let group = r.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                resource: wgpu::BindingResource::TextureView(&nv),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&tv),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: estimates.as_entire_binding(),
            },
        ],
    });
    let queries = r.device.create_query_set(&wgpu::QuerySetDescriptor {
        label: None,
        ty: wgpu::QueryType::Timestamp,
        count: 2,
    });
    let resolve = buffer(
        &r,
        "compute query resolve",
        256,
        wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
    );
    let timing = buffer(
        &r,
        "compute query readback",
        16,
        wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    );
    let resources_ms = elapsed(start);
    let names = [
        "native",
        "mc16",
        "cache",
        "cv",
        "pilot",
        "validated",
        "ttl",
        "revision",
        "watched",
        "reference512",
    ];
    ensure!(
        boxes.len() <= 32,
        "Diagnostic influence mask supports at most32 boxes"
    );
    let mut rows = vec![];
    for round in 0..2 {
        params.light[3] = if audit { 1. } else { 0. };
        let mut cold = [(0., 0.); 10];
        for mode in 1..10 {
            params.dims[3] = mode;
            params.light[1] = 0.;
            r.queue
                .write_buffer(&uniform, 0, bytemuck::bytes_of(&params));
            cold[mode as usize] = compute(&r, &setup, &group, &queries, &resolve, &timing)?;
            for warm in 0..8 {
                params.light[1] = warm as f32;
                r.queue
                    .write_buffer(&uniform, 0, bytemuck::bytes_of(&params));
                compute(&r, &shade, &group, &queries, &resolve, &timing)?;
            }
        }
        // All methods share the same immutable angular field and receiver map.
        // The three epoch policies have identical weights under this static revision;
        // their refresh work is still executed and timed independently.
        params.dims[3] = 3;
        r.queue
            .write_buffer(&uniform, 0, bytemuck::bytes_of(&params));
        compute(&r, &setup, &group, &queries, &resolve, &timing)?;
        for index in 0..64 {
            for offset in 0..10 {
                let mode = (index + if round == 0 { offset } else { 9 - offset }) % 10;
                for cull in 0..if audit { 2 } else { 1 } {
                    let cull = if audit && (index + offset) % 2 == 1 {
                        1 - cull
                    } else {
                        cull
                    };
                    params.light[3] = cull as f32;
                    let cold = cold[mode as usize];
                    params.dims[3] = mode;
                    let t = Instant::now();
                    let mut f = engine.render_frame();
                    f.suppress_diagnostic_grid = true;
                    let prepare = elapsed(t);
                    let t = Instant::now();
                    r.draw(&f, &nv, W, H)?;
                    let native_cpu = elapsed(t);
                    let native_gpu = r.read_gpu_timing()?.unwrap()["sum_ms"].as_f64().unwrap();
                    params.light[1] = index as f32;
                    let t = Instant::now();
                    r.queue
                        .write_buffer(&uniform, 0, bytemuck::bytes_of(&params));
                    let upload = elapsed(t);
                    let (cpu, gpu) = if mode == 0 {
                        (0., 0.)
                    } else {
                        compute(&r, &shade, &group, &queries, &resolve, &timing)?
                    };
                    rows.push(json!({"round":round,"culling":cull,"mode":mode,"name":names[mode as usize],"frame":index,"prepare_cpu_ms":prepare,"raster_cpu_ms":native_cpu,"raster_gpu_ms":native_gpu,"compute_cpu_ms":cpu+if mode==0{0.}else{upload},"compute_gpu_ms":gpu,"setup_cpu_ms":cold.0,"setup_gpu_ms":cold.1}));
                    if round == 0 && (index == 15 || (mode == 0 && index == 0)) {
                        let image = if mode == 0 {
                            r.read_texture_png(&native, W, H)?
                        } else {
                            r.read_texture_png(&target, W, H)?
                        };
                        let suffix = if audit {
                            format!("-{cull}")
                        } else {
                            String::new()
                        };
                        fs::write(
                            format!("{dir}/{mode:02}-{}{suffix}.png", names[mode as usize]),
                            image,
                        )?;
                        if mode != 0 {
                            let mut e = r.device.create_command_encoder(&Default::default());
                            e.copy_buffer_to_buffer(&estimates, 0, &raw, 0, estimate_bytes);
                            r.queue.submit([e.finish()]);
                            let data = read(&r, &raw, estimate_bytes)?;
                            ensure!(
                                data.chunks_exact(4)
                                    .all(|x| f32::from_le_bytes(x.try_into().unwrap()).is_finite()),
                                "Nonfinite estimate"
                            );
                            fs::write(
                                format!("{dir}/{mode:02}-{}{suffix}.f32", names[mode as usize]),
                                &data,
                            )?;
                        }
                    }
                }
            }
            if index % 16 == 15 {
                eprintln!("round {round} frame {index} complete");
            }
        }
    }
    // Explicit all-visible composite oracle, using the same native image and receiver map.
    params.dims[3] = 0;
    r.queue
        .write_buffer(&uniform, 0, bytemuck::bytes_of(&params));
    compute(&r, &shade, &group, &queries, &resolve, &timing)?;
    let pass = r.read_texture_png(&target, W, H)?;
    let baseline = r.read_texture_png(&native, W, H)?;
    ensure!(pass == baseline, "sRGB passthrough changed pixels");
    let suffix = if audit { "-0" } else { "" };
    ensure!(
        fs::read(format!("{dir}/06-ttl{suffix}.f32"))?
            == fs::read(format!("{dir}/07-revision{suffix}.f32"))?,
        "Static TTL/revision mismatch"
    );
    if audit {
        for mode in 0..10 {
            for ext in if mode == 0 {
                vec!["png"]
            } else {
                vec!["png", "f32"]
            } {
                let stem = format!("{dir}/{mode:02}-{}", names[mode as usize]);
                ensure!(
                    fs::read(format!("{stem}-0.{ext}"))? == fs::read(format!("{stem}-1.{ext}"))?,
                    "Influence culling changed {stem}.{ext}"
                );
            }
        }
        let staging = buffer(
            &r,
            "visibility audit receivers",
            receiver_bytes,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let mut encoder = r.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&receivers, 0, &staging, 0, receiver_bytes);
        r.queue.submit([encoder.finish()]);
        fs::write(
            format!("{dir}/receivers.bin"),
            read(&r, &staging, receiver_bytes)?,
        )?;
        if guard {
            params.dims[3] = 9;
            params.light[1] = 15.;
            params.light[3] = 2.;
            r.queue
                .write_buffer(&uniform, 0, bytemuck::bytes_of(&params));
            compute(&r, &shade, &group, &queries, &resolve, &timing)?;
            let bad = r.read_texture_png(&target, W, H)?;
            ensure!(
                bad != fs::read(format!("{dir}/09-reference512-0.png"))?,
                "Offscreen blocker had no AO influence"
            );
            fs::write(format!("{dir}/incorrect-offscreen-removal.png"), bad)?;
        }
    }
    fs::write(
        format!("{dir}/results.json"),
        serde_json::to_vec_pretty(
            &json!({"adapter":{"name":r.adapter.get_info().name,"backend":format!("{:?}",r.adapter.get_info().backend),"device_type":format!("{:?}",r.adapter.get_info().device_type)},"width":W,"height":H,"renderer_setup_cpu_ms":renderer_setup,"resources_setup_cpu_ms":resources_ms,"receiver_bytes":receiver_bytes,"additional_resident_bytes":receiver_bytes+estimate_bytes+2*u64::from(W)*u64::from(H)*4+boxes.len() as u64*48+112+256+16,"timestamp_period_ns":r.queue.get_timestamp_period(),"visibility_audit":audit,"offscreen_guard":guard,"timestamp_resolution_fenced":true,"compute_timestamps_monotonic":true,"native_profile_identity":true,"passthrough_identity":true,"static_ttl_revision_identity":true,"rows":rows}),
        )?,
    )?;
    Ok(())
}
fn compute(
    r: &Renderer,
    p: &wgpu::ComputePipeline,
    group: &wgpu::BindGroup,
    q: &wgpu::QuerySet,
    resolve: &wgpu::Buffer,
    timing: &wgpu::Buffer,
) -> Result<(f64, f64)> {
    let t = Instant::now();
    let mut e = r.device.create_command_encoder(&Default::default());
    {
        let mut pass = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("AO diagnostic"),
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set: q,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: Some(1),
            }),
        });
        pass.set_pipeline(p);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
    }
    r.queue.submit([e.finish()]);
    let cpu = elapsed(t);
    // Fence the compute pass before resolving Metal counter samples.
    wait(r)?;
    let mut e = r.device.create_command_encoder(&Default::default());
    e.resolve_query_set(q, 0..2, resolve, 0);
    e.copy_buffer_to_buffer(resolve, 0, timing, 0, 16);
    r.queue.submit([e.finish()]);
    let bytes = read(r, timing, 16)?;
    let a = u64::from_le_bytes(bytes[..8].try_into()?);
    let b = u64::from_le_bytes(bytes[8..].try_into()?);
    static LAST_END: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let previous = LAST_END.swap(b, std::sync::atomic::Ordering::Relaxed);
    ensure!(
        a >= previous && b > a,
        "Stale/nonmonotonic compute timestamps: {a}..{b}, previous end {previous}"
    );
    Ok((
        cpu,
        (b - a) as f64 * r.queue.get_timestamp_period() as f64 / 1e6,
    ))
}
