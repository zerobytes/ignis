//! Lossless spatial packing, with filter halos and direct resident GPU sampling.
use super::*;
const OUT: &str = "artifacts/appearance-probe/compression";
const WIDTH: u32 = 2048;
const BAND: u32 = 16;
const BANDS: u32 = TEXELS / BAND;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, Serialize, Deserialize)]
struct Region {
    // Packed origin, source origin, extent, reserved.
    data: [u32; 8],
}
#[derive(Serialize, Deserialize)]
pub(super) struct PackedManifest {
    schema: u32,
    source_signature: String,
    source_checksum: u64,
    width: u32,
    height: u32,
    count: u32,
    band: u32,
    bytes: usize,
    checksum: u64,
    descriptor_checksum: u64,
    regions: Vec<Region>,
    build_seconds: f64,
}
fn source_offset(n: u32, v: u32, l: u32, x: u32, y: u32) -> usize {
    ((((v * (n + 1) + l) * TEXELS + y) * TEXELS + x) * 8) as usize
}
fn pack(source: &[u8], n: u32) -> Result<(PackedManifest, Vec<u8>)> {
    ensure!(source.len() == expected_bytes(n), "invalid source size");
    let start = Instant::now();
    let mut regions = Vec::new();
    let (mut px, mut py, mut row_height) = (0, 0, 0);
    for v in 0..n {
        let mut bounds = Vec::new();
        for b in 0..BANDS {
            let ymin = (b * BAND).saturating_sub(1);
            let ymax = ((b + 1) * BAND + 1).min(TEXELS);
            let (mut xmin, mut xmax) = (TEXELS, 0);
            for l in 0..=n {
                for y in ymin..ymax {
                    for x in 0..TEXELS {
                        let p = source_offset(n, v, l, x, y);
                        if source[p..p + 8].iter().any(|b| *b != 0) {
                            xmin = xmin.min(x);
                            xmax = xmax.max(x + 1);
                        }
                    }
                }
            }
            bounds.push(if xmin < xmax {
                [xmin.saturating_sub(1), ymin, (xmax + 1).min(TEXELS), ymax]
            } else {
                [0; 4]
            });
        }
        for _l in 0..=n {
            for &[xmin, ymin, xmax, ymax] in &bounds {
                let (w, h) = (xmax - xmin, ymax - ymin);
                if w == 0 {
                    regions.push(Region::default());
                    continue;
                }
                if px + w > WIDTH {
                    px = 0;
                    py += row_height;
                    row_height = 0;
                }
                regions.push(Region {
                    data: [px, py, xmin, ymin, w, h, 0, 0],
                });
                px += w;
                row_height = row_height.max(h);
            }
        }
    }
    let height = (py + row_height).max(1);
    ensure!(height <= 8192, "packed atlas exceeds experiment limit");
    let mut bytes = vec![0; (WIDTH * height * 8) as usize];
    for v in 0..n {
        for l in 0..=n {
            for b in 0..BANDS {
                let r = regions[((v * (n + 1) + l) * BANDS + b) as usize].data;
                for y in 0..r[5] {
                    let src = source_offset(n, v, l, r[2], r[3] + y);
                    let dst = ((r[1] + y) * WIDTH + r[0]) as usize * 8;
                    bytes[dst..dst + r[4] as usize * 8]
                        .copy_from_slice(&source[src..src + r[4] as usize * 8]);
                }
            }
        }
    }
    let mut m = PackedManifest {
        schema: 1,
        source_signature: String::new(),
        source_checksum: hash(source),
        width: WIDTH,
        height,
        count: n,
        band: BAND,
        bytes: bytes.len(),
        checksum: hash(&bytes),
        descriptor_checksum: hash(bytemuck::cast_slice(&regions)),
        regions,
        build_seconds: start.elapsed().as_secs_f64(),
    };
    verify_texels(&m, &bytes, source)?;
    m.build_seconds = start.elapsed().as_secs_f64();
    Ok((m, bytes))
}
fn validate_packed(m: &PackedManifest, bytes: &[u8], signature: &str) -> Result<()> {
    ensure!(
        m.schema == 1
            && m.count == 24
            && m.band == BAND
            && m.width == WIDTH
            && m.height > 0
            && m.height <= 8192,
        "unsupported packed atlas"
    );
    ensure!(m.source_signature == signature, "stale packed atlas");
    ensure!(
        m.bytes == bytes.len() && bytes.len() == (m.width * m.height * 8) as usize,
        "packed length mismatch"
    );
    ensure!(
        m.regions.len() == (m.count * (m.count + 1) * BANDS) as usize,
        "region count mismatch"
    );
    ensure!(
        hash(bytes) == m.checksum
            && hash(bytemuck::cast_slice(&m.regions)) == m.descriptor_checksum,
        "packed checksum mismatch"
    );
    for r in &m.regions {
        let [x, y, sx, sy, w, h, a, b] = r.data;
        ensure!(
            w <= TEXELS
                && h <= BAND + 2
                && sx <= TEXELS - w
                && sy <= TEXELS - h
                && x <= m.width - w
                && y <= m.height.saturating_sub(h)
                && h <= m.height
                && a == 0
                && b == 0,
            "invalid region bounds"
        );
    }
    Ok(())
}
fn verify_texels(m: &PackedManifest, bytes: &[u8], source: &[u8]) -> Result<()> {
    // Also verify the duplicated halo: reconstructing core texels alone cannot catch seams.
    for (i, region) in m.regions.iter().enumerate() {
        let tile = i as u32 / BANDS;
        let r = region.data;
        for y in 0..r[5] {
            for x in 0..r[4] {
                let src = source_offset(
                    m.count,
                    tile / (m.count + 1),
                    tile % (m.count + 1),
                    r[2] + x,
                    r[3] + y,
                );
                let dst = ((r[1] + y) * m.width + r[0] + x) as usize * 8;
                ensure!(
                    bytes[dst..dst + 8] == source[src..src + 8],
                    "lossy filter halo"
                );
            }
        }
    }
    // Every source texel must be retrievable, including RGB under zero alpha.
    for v in 0..m.count {
        for l in 0..=m.count {
            for y in 0..TEXELS {
                for x in 0..TEXELS {
                    let r = m.regions[((v * (m.count + 1) + l) * BANDS + y / BAND) as usize].data;
                    let src = source_offset(m.count, v, l, x, y);
                    let zero = [0u8; 8];
                    let decoded = if x >= r[2] && x < r[2] + r[4] && y >= r[3] && y < r[3] + r[5] {
                        let dst = ((r[1] + y - r[3]) * m.width + r[0] + x - r[2]) as usize * 8;
                        &bytes[dst..dst + 8]
                    } else {
                        &zero
                    };
                    ensure!(decoded == &source[src..src + 8], "lossy texel packing");
                }
            }
        }
    }
    Ok(())
}
fn build(vertices: &[Vertex]) -> Result<()> {
    build_in(Path::new(ROOT), Path::new(OUT), &signature(vertices, 24))
}
pub(super) fn build_in(root: &Path, out: &Path, sig: &str) -> Result<()> {
    fs::create_dir_all(out)?;
    let source: Manifest = serde_json::from_slice(&fs::read(root.join("atlas24.json"))?)?;
    let bytes = fs::read(root.join("atlas24.bin"))?;
    validate(&source, &bytes, sig)?;
    let (mut m, data) = pack(&bytes, 24)?;
    m.source_signature = sig.to_owned();
    validate_packed(&m, &data, &m.source_signature)?;
    fs::write(out.join("atlas.bin"), &data)?;
    fs::write(out.join("atlas.json"), serde_json::to_vec_pretty(&m)?)?;
    println!(
        "packed {} + {} bytes; source {}; texels and halos exact",
        m.bytes,
        m.regions.len() * 32,
        bytes.len()
    );
    Ok(())
}
fn load(lab: &Lab, vertices: &[Vertex]) -> Result<(Atlas, PackedManifest)> {
    load_in(lab, vertices, Path::new(OUT))
}
pub(super) fn load_in(
    lab: &Lab,
    vertices: &[Vertex],
    out: &Path,
) -> Result<(Atlas, PackedManifest)> {
    let start = Instant::now();
    let m: PackedManifest = serde_json::from_slice(
        &fs::read(out.join("atlas.json")).context("run --compress-views first")?,
    )?;
    let data = fs::read(out.join("atlas.bin"))?;
    validate_packed(&m, &data, &lab.source_signature(vertices, 24))?;
    let device = &lab.gpu.device;
    let t = texture(
        device,
        m.width,
        m.height,
        1,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    );
    lab.gpu.queue.write_texture(
        t.as_image_copy(),
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(m.width * 8),
            rows_per_image: Some(m.height),
        },
        wgpu::Extent3d {
            width: m.width,
            height: m.height,
            depth_or_array_layers: 1,
        },
    );
    let descriptors = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("packed regions"),
        contents: bytemuck::cast_slice(&m.regions),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("packed views"),
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let begin = SHADER.find("fn lookup(").unwrap();
    let end = SHADER.find("fn reconstruct(").unwrap();
    let shader = format!(
        "{}\n{}\n{}",
        &SHADER[..begin],
        include_str!("compressed_lookup.wgsl"),
        &SHADER[end..]
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("packed lookup"),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("packed proxy"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("screen"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("proxy"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });
    let mut groups = Vec::new();
    let mut uniforms = Vec::new();
    for _ in 0..2 {
        let u = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: u.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&t.create_view(
                        &wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&lab.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: descriptors.as_entire_binding(),
                },
            ],
        }));
        uniforms.push(u);
    }
    lab.submit(device.create_command_encoder(&Default::default()))?;
    let manifest = Manifest {
        schema: 1,
        count: 24,
        texels: TEXELS,
        signature: m.source_signature.clone(),
        checksum: m.checksum,
        bytes: m.bytes + m.regions.len() * 32,
        bake_seconds: m.build_seconds,
        triangles: lab.count as usize / 3,
        geometry_bytes: std::mem::size_of_val(vertices)
            + lab.index.as_ref().map_or(0, |_| lab.count as usize * 4),
    };
    Ok((
        Atlas {
            groups,
            uniforms,
            _texture: t,
            pipeline: Some(pipeline),
            manifest,
            load_ms: start.elapsed().as_secs_f64() * 1000.,
        },
        m,
    ))
}
pub fn run(mode: &str) -> Result<()> {
    let vertices = vertices();
    if mode == "--compress-views" {
        return build(&vertices);
    }
    let mut lab = pollster::block_on(Lab::new(&vertices))?;
    let original = lab.load(24, &vertices)?;
    let (compact, manifest) = load(&lab, &vertices)?;
    // Instrumentation only: release the dense CPU payload before capture/benchmark.
    {
        let source = fs::read(Path::new(ROOT).join("atlas24.bin"))?;
        ensure!(
            hash(&source) == manifest.source_checksum,
            "packed source mismatch"
        );
        let bytes = fs::read(Path::new(OUT).join("atlas.bin"))?;
        verify_texels(&manifest, &bytes, &source)?;
    }
    let smoke = mode == "--compressed-views-smoke";
    let mut rows = Vec::new();
    for scene in ["camera", "light", "both", "near", "seam"] {
        let count = if scene == "seam" { 17 } else { 49 };
        type Image = Vec<[f32; 4]>;
        let mut previous: Option<(Image, Image)> = None;
        for frame in 0..count {
            if smoke && ![0, count / 2, count - 1].contains(&frame) {
                continue;
            }
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
            let a = lab.capture(pose, Some(&original), 1, 4)?;
            let b = lab.capture(pose, Some(&compact), 1, 4)?;
            let delta = metrics::compare(&a, &b);
            let id = format!("{scene}_{frame:02}");
            println!(
                "{id} delta rmse {:.8} max {:.8} alpha {:.8}",
                delta.rmse_surface, delta.max_abs, delta.alpha_rmse
            );
            ensure!(
                delta.rmse_surface < 0.0002 && delta.alpha_rmse < 0.001 && delta.max_abs < 0.02,
                "compression exceeded linear image error budget: {id}"
            );
            if !smoke {
                let reference = lab.capture(pose, None, 0, 16)?;
                for (name, pixels) in [("original", &a), ("compact", &b), ("reference", &reference)]
                {
                    metrics::png(
                        &Path::new(OUT).join(format!("{id}_{name}.png")),
                        pixels,
                        SIZE,
                    )?;
                }
                metrics::error_png(
                    &Path::new(OUT).join(format!("{id}_delta.png")),
                    &a,
                    &b,
                    SIZE,
                )?;
                let an = lab.capture(pose, Some(&original), 1, 1)?;
                let bn = lab.capture(pose, Some(&compact), 1, 1)?;
                let fullscreen = lab.capture_options(pose, Some(&compact), 1, 1, false, [0.; 2])?;
                ensure!(bn == fullscreen, "packed bounding changed pixels");
                ensure!(
                    metrics::compare(&reference, &b).rmse_surface
                        <= metrics::compare(&reference, &a).rmse_surface + 0.00002,
                    "reference fidelity regressed: {id}"
                );
                let timings = lab.bench_choices(
                    pose,
                    [
                        (None, 0),
                        (Some(&original), 1),
                        (Some(&compact), 1),
                        (Some(&original), 0),
                    ],
                )?;
                rows.push(json!({"id":id,"scene":scene,"frame":frame,"pose":pose,"delta":delta,"native_delta":metrics::compare(&an,&bn),"exact":a==b,"original_error":metrics::compare(&reference,&a),"compact_error":metrics::compare(&reference,&b),"temporal_delta":previous.as_ref().map(|(pa,pb)|metrics::temporal(pa,pb,&a,&b)),"timings":timings}));
            }
            previous = Some((a, b));
        }
    }
    if !smoke {
        let p = Pose {
            camera: 13.75,
            light: 71.25,
            distance: 12.,
        };
        let a = lab.capture(p, Some(&compact), 1, 1)?;
        ensure!(
            a == lab.capture(p, Some(&compact), 1, 1)?,
            "nondeterministic compact capture"
        );
        ensure!(
            a == lab.capture(
                Pose {
                    camera: p.camera + 360.,
                    light: p.light + 360.,
                    ..p
                },
                Some(&compact),
                1,
                1
            )?,
            "compact angular wrap"
        );
        let changed = lab.capture(
            Pose {
                light: p.light + 70.,
                ..p
            },
            Some(&compact),
            1,
            1,
        )?;
        ensure!(a != changed, "compact ignores light");
        ensure!(
            a == lab.capture(p, Some(&compact), 1, 1)?,
            "compact return after light movement"
        );
        fs::write(
            Path::new(OUT).join("report.json"),
            serde_json::to_vec_pretty(
                &json!({"schema":1,"checks":{"texels_and_halos_exact":true,"bounded_pairs":rows.len(),"repeat_wrap_light_return":true,"linear_rmse_budget":0.0002,"alpha_rmse_budget":0.001,"max_rgb_budget":0.02,"reference_rmse_increase_budget":0.00002},"adapter":lab.gpu.adapter,"source_bytes":original.manifest.bytes,"packed_bytes":compact.manifest.bytes,"texture_bytes":manifest.bytes,"descriptor_bytes":manifest.regions.len()*32,"build_seconds":manifest.build_seconds,"original_load_ms":original.load_ms,"compact_load_ms":compact.load_ms,"geometry_bytes":std::mem::size_of_val(vertices.as_slice()),"triangles":vertices.len()/3,"cases":rows}),
            )?,
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packing_preserves_edges_band_boundaries_and_invisible_rgb() {
        let mut source = vec![0; expected_bytes(24)];
        for (x, y) in [(0, 0), (127, 127), (15, 15), (16, 16), (80, 31), (79, 32)] {
            for l in 0..=24 {
                let p = source_offset(24, 3, l, x, y);
                source[p] = 42;
                source[p + 2] = 99;
            }
        }
        let (mut m, mut bytes) = pack(&source, 24).unwrap();
        let halo = m
            .regions
            .iter()
            .enumerate()
            .find(|(i, r)| r.data[4] > 0 && r.data[3] < (*i as u32 % BANDS) * BAND)
            .unwrap()
            .1
            .data;
        let p = ((halo[1]) * WIDTH + halo[0]) as usize * 8;
        bytes[p] ^= 1;
        assert!(verify_texels(&m, &bytes, &source).is_err());
        bytes[p] ^= 1;
        m.source_signature = "fixture".into();
        assert!(validate_packed(&m, &bytes, "fixture").is_ok());
        assert!(validate_packed(&m, &bytes, "other").is_err());
        assert!(validate_packed(&m, &bytes[..bytes.len() - 1], "fixture").is_err());
        let old = m.regions[0];
        m.regions[0].data[0] = u32::MAX;
        m.descriptor_checksum = hash(bytemuck::cast_slice(&m.regions));
        assert!(validate_packed(&m, &bytes, "fixture").is_err());
        m.regions[0] = old;
        m.descriptor_checksum = hash(bytemuck::cast_slice(&m.regions));
        m.checksum ^= 1;
        assert!(validate_packed(&m, &bytes, "fixture").is_err());
    }
}
