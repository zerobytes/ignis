//! Fixed piecewise-linear content; only barycentric subdivision changes.
use super::*;
const OUTPUT: &str = "artifacts/appearance-probe/pixel-density";
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
const TARGETS: [u64; 4] = [1, 2, 10, 100];
const DISTANCES: [f32; 5] = [12., 14., 16., 14., 12.];
const CHUNK_TRIANGLES: usize = 8_000_000;
pub(super) struct Chunk {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub count: u32,
}
pub(super) struct Dense {
    pub chunks: Vec<Chunk>,
    pub triangles: u64,
    pub error_world: f64,
    pub metadata: serde_json::Value,
}
// For a view-space displacement of norm <= e:
// |screen(p+d)-screen(p)| <= focal*e*(1+|p.xy|/z)/(z-e).
// Corner bounds enclose both the fixed surface and its rounded subdivisions.
pub(super) fn projected(error: f64, cam: &Camera, bounds: [[f32; 3]; 2]) -> f64 {
    if !error.is_finite() || error < 0. {
        return f64::INFINITY;
    }
    let mut z = f64::INFINITY;
    let mut r = 0f64;
    for x in 0..2 {
        for y in 0..2 {
            for zc in 0..2 {
                let p =
                    cam.view
                        .transform_point3(Vec3::new(bounds[x][0], bounds[y][1], bounds[zc][2]));
                z = z.min(-p.z as f64);
                r = r.max(p.truncate().length() as f64);
            }
        }
    }
    if z <= error + 0.1 {
        return f64::INFINITY;
    }
    let focal = cam.size[1] as f64 / (2. * 30f64.to_radians().tan());
    focal * error * (1. + r / z) / (z - error)
}
pub(super) fn eligible(error: Option<f64>, cam: &Camera, bounds: [[f32; 3]; 2]) -> bool {
    error.is_some_and(|e| projected(e, cam, bounds) <= 0.15)
}
#[derive(Clone, Copy, Debug)]
struct Plan {
    n: u32,
    splits: u64,
    base: u64,
    parents: u64,
    triangles: u64,
}
fn plan(parents: u64, target: u64) -> Result<Plan> {
    ensure!(
        parents > 0 && target >= parents && target < u32::MAX as u64,
        "density outside experiment bounds"
    );
    let n = ((target as f64 / parents as f64).sqrt().floor() as u32).max(1);
    let base = parents * n as u64 * n as u64;
    let splits = (target - base).div_ceil(2);
    Ok(Plan {
        n,
        splits,
        base,
        parents,
        triangles: base + 2 * splits,
    })
}
impl Plan {
    fn splits_at(self, i: u64) -> u32 {
        (((i + 1) * self.splits / self.base) - (i * self.splits / self.base)) as u32
    }
}
fn split_interior(t: [u32; 3], splits: u32, v: &mut Vec<[f32; 3]>, out: &mut Vec<u32>) -> f64 {
    if splits == 0 {
        out.extend_from_slice(&t);
        return 0.;
    }
    let ideal = std::array::from_fn::<_, 3, _>(|axis| {
        t.iter().map(|&id| v[id as usize][axis] as f64).sum::<f64>() / 3.
    });
    let stored = ideal.map(|x| x as f32);
    let center = v.len() as u32;
    v.push(stored);
    let error = (0..3)
        .map(|axis| (stored[axis] as f64 - ideal[axis]).powi(2))
        .sum::<f64>()
        .sqrt();
    let nested = split_interior([t[0], t[1], center], splits - 1, v, out);
    out.extend_from_slice(&[t[1], t[2], center, t[2], t[0], center]);
    error + nested
}

fn subdivide(p: [[f32; 3]; 3], n: u32, v: &mut Vec<[f32; 3]>, indices: &mut Vec<u32>) -> f64 {
    let base = v.len() as u32;
    let mut error = 0f64;
    for j in 0..=n {
        for i in 0..=n - j {
            let ideal = std::array::from_fn::<_, 3, _>(|axis| {
                (p[0][axis] as f64 * (n - i - j) as f64
                    + p[1][axis] as f64 * i as f64
                    + p[2][axis] as f64 * j as f64)
                    / n as f64
            });
            let stored = ideal.map(|x| x as f32);
            error = error.max(
                (0..3)
                    .map(|axis| (stored[axis] as f64 - ideal[axis]).powi(2))
                    .sum::<f64>()
                    .sqrt(),
            );
            v.push(stored);
        }
    }
    let row = |j: u32| j * (n + 1) - j * j.saturating_sub(1) / 2;
    for j in 0..n {
        for i in 0..n - j {
            let a = base + row(j) + i;
            let b = base + row(j + 1) + i;
            indices.extend_from_slice(&[a, a + 1, b]);
            if i + 1 < n - j {
                indices.extend_from_slice(&[a + 1, b + 1, b]);
            }
        }
    }
    error
}
pub(super) fn prepare(renderer: &Renderer, a: &Asset, pixels: u64, target: u64) -> Result<Dense> {
    let start = Instant::now();
    let vb = fs::read(Path::new(SOURCE).join(format!("{}.vertices", a.id)))?;
    let ib = fs::read(Path::new(SOURCE).join(format!("{}.indices", a.id)))?;
    let source = bytemuck::cast_slice::<u8, build::V>(&vb);
    let indices = bytemuck::cast_slice::<u8, u32>(&ib);
    let l = a.levels.last().unwrap();
    let indices = &indices[l.first as usize..(l.first + l.count) as usize];
    let plan = plan(l.count as u64 / 3, pixels * target)?;
    let mut vertices = Vec::new();
    let mut out = Vec::new();
    let mut chunks = Vec::new();
    let mut total_bytes = 0u64;
    let mut total_indices = 0u64;
    let mut error = 0f64;
    let mut peak_scratch = 0usize;
    let mut scratch = Vec::new();
    let mut split_error = 0f64;
    let max_buffer = (renderer.gpu.device.limits().max_buffer_size as usize).min(128 * 1024 * 1024);
    let mut flush = |v: &mut Vec<[f32; 3]>, i: &mut Vec<u32>| -> Result<()> {
        if i.is_empty() {
            return Ok(());
        }
        ensure!(
            v.len() * 12 <= max_buffer && i.len() * 4 <= max_buffer,
            "source chunk exceeds buffer limit"
        );
        peak_scratch = peak_scratch.max(v.capacity() * 12 + i.capacity() * 4);
        total_bytes += (v.len() * 12 + i.len() * 4) as u64;
        total_indices += i.len() as u64;
        chunks.push(Chunk {
            vertex: renderer
                .gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("fixed-surface subdivision positions"),
                    contents: bytemuck::cast_slice(v),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            index: renderer
                .gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("fixed-surface subdivision indices"),
                    contents: bytemuck::cast_slice(i),
                    usage: wgpu::BufferUsages::INDEX,
                }),
            count: u32::try_from(i.len())?,
        });
        v.clear();
        i.clear();
        Ok(())
    };
    for (id, t) in indices.chunks_exact(3).enumerate() {
        let n = plan.n;
        let first = id as u64 * n as u64 * n as u64;
        let last = first + n as u64 * n as u64;
        let splits = (last * plan.splits / plan.base - first * plan.splits / plan.base) as usize;
        let nv = (n as usize + 1) * (n as usize + 2) / 2 + splits;
        let ni = 3 * (n as usize * n as usize + 2 * splits);
        if out.len() + ni > CHUNK_TRIANGLES * 3
            || (vertices.len() + nv) * 12 > max_buffer
            || (out.len() + ni) * 4 > max_buffer
        {
            flush(&mut vertices, &mut out)?;
        }
        scratch.clear();
        error = error.max(subdivide(
            [
                source[t[0] as usize].p,
                source[t[1] as usize].p,
                source[t[2] as usize].p,
            ],
            n,
            &mut vertices,
            &mut scratch,
        ));
        for (local, t) in scratch.chunks_exact(3).enumerate() {
            split_error = split_error.max(split_interior(
                [t[0], t[1], t[2]],
                plan.splits_at(first + local as u64),
                &mut vertices,
                &mut out,
            ));
        }
    }
    flush(&mut vertices, &mut out)?;
    ensure!(
        total_indices / 3 == plan.triangles,
        "actual subdivision count mismatch"
    );
    // Numeric safety allowance for f32 coordinates/transforms, separate from the
    // measured vertex rounding. Raster coverage is still checked independently.
    let extent = a
        .bounds
        .iter()
        .flatten()
        .map(|x| x.abs() as f64)
        .fold(1f64, f64::max);
    let guarded = 2. * (error + split_error) + 8. * f32::EPSILON as f64 * extent;
    renderer.wait(renderer.gpu.queue.submit([]))?;
    let metadata = json!({"target_triangles_per_pixel":target,"covered_pixels_at14":pixels,"triangles":plan.triangles,"actual_triangles_per_pixel":plan.triangles as f64/pixels as f64,"parent_triangles":plan.parents,"uniform_edge_subdivisions":plan.n,"interior_splits":plan.splits,"base_grid_triangles":plan.base,"geometry_bytes":total_bytes,"draws":chunks.len(),"vertex_rounding_world":error,"interior_rounding_bound_world":split_error,"guarded_error_world":guarded,"prepare_upload_ms":start.elapsed().as_secs_f64()*1000.,"peak_scratch_capacity_bytes":peak_scratch,"max_buffer_bytes":max_buffer});
    Ok(Dense {
        chunks,
        triangles: plan.triangles,
        error_world: guarded,
        metadata,
    })
}
fn modes(d: usize) -> [u32; 3] {
    [80, (d as u32 + 1) * 1000 + 140, (d as u32 + 1) * 1000 + 150]
}
fn benchmark(
    renderer: &Renderer,
    u: &render::Uploaded,
    a: &Asset,
    phase: usize,
) -> Result<serde_json::Value> {
    let n = renderer.density_sources.len();
    let slots = n * 3 * 2;
    let cam = render::camera_at(14., renderer_size(renderer));
    let mut totals = vec![Vec::new(); slots];
    let mut cpus = totals.clone();
    let mut gpus = totals.clone();
    let frames = (0..slots)
        .map(|s| {
            if s % 3 == 1 {
                (32_000_000 / renderer.density_sources[(s / 3) % n].triangles).clamp(2, 128)
                    as usize
            } else {
                128
            }
        })
        .collect::<Vec<_>>();
    for d in 0..n {
        for code in modes(d) {
            for _ in 0..2 {
                let mut e = renderer
                    .gpu
                    .device
                    .create_command_encoder(&Default::default());
                renderer.encode(&mut e, u, a, &cam, code, None);
                renderer.wait(renderer.gpu.queue.submit([e.finish()]))?;
            }
        }
    }
    for round in 0..8 {
        for offset in 0..slots {
            let r = round + phase * 8;
            let s = if r.is_multiple_of(2) {
                (offset + r * 3) % slots
            } else {
                (slots - 1 - offset + r * 3) % slots
            };
            let d = (s / 3) % n;
            let code = modes(d)[s % 3];
            let count = frames[s];
            let start = Instant::now();
            let mut last = None;
            for _ in 0..count {
                let cam = render::camera_at(14., renderer_size(renderer));
                let mut e = renderer
                    .gpu
                    .device
                    .create_command_encoder(&Default::default());
                renderer.encode(&mut e, u, a, &cam, code, None);
                last = Some(renderer.gpu.queue.submit([e.finish()]));
            }
            cpus[s].push(start.elapsed().as_secs_f64() * 1000. / count as f64);
            renderer.wait(last.unwrap())?;
            totals[s].push(start.elapsed().as_secs_f64() * 1000. / count as f64);
            if let Some(q) = &renderer.gpu.query {
                let count = count.min(32);
                let mut last = None;
                for i in 0..count {
                    let mut e = renderer
                        .gpu
                        .device
                        .create_command_encoder(&Default::default());
                    renderer.encode(&mut e, u, a, &cam, code, Some(i as u32 * 2));
                    last = Some(renderer.gpu.queue.submit([e.finish()]));
                }
                renderer.wait(last.unwrap())?;
                let bytes = count as u64 * 16;
                let mut e = renderer
                    .gpu
                    .device
                    .create_command_encoder(&Default::default());
                e.resolve_query_set(q, 0..count as u32 * 2, &renderer.gpu.query_resolve, 0);
                e.copy_buffer_to_buffer(
                    &renderer.gpu.query_resolve,
                    0,
                    &renderer.gpu.query_readback,
                    0,
                    bytes,
                );
                renderer.wait(renderer.gpu.queue.submit([e.finish()]))?;
                renderer.gpu.map(&renderer.gpu.query_readback)?;
                let data = renderer.gpu.query_readback.slice(..).get_mapped_range();
                let stamps = bytemuck::cast_slice::<u8, u64>(&data);
                let mut sum = 0u64;
                for i in 0..count {
                    ensure!(stamps[2 * i + 1] >= stamps[2 * i], "GPU clock reversed");
                    sum += stamps[2 * i + 1] - stamps[2 * i];
                }
                gpus[s].push(
                    sum as f64 * renderer.gpu.timestamp_period_ns.unwrap() as f64
                        / count as f64
                        / 1e6,
                );
                drop(data);
                renderer.gpu.query_readback.unmap();
            }
        }
    }
    Ok(json!((0..slots).map(|s|json!({"density":(s/3)%n,"method":s%3,"control":s/(n*3),"frames":frames[s],"gpu_frames":frames[s].min(32),"total_ms":stats(&mut totals[s].clone()),"cpu_ms":stats(&mut cpus[s].clone()),"gpu_ms":stats(&mut gpus[s].clone()),"total_samples_ms":totals[s],"cpu_samples_ms":cpus[s],"gpu_samples_ms":gpus[s]})).collect::<Vec<_>>()))
}
fn renderer_size(renderer: &Renderer) -> [u32; 2] {
    renderer.physical_size()
}
pub(super) fn run(smoke: bool) -> Result<()> {
    fs::create_dir_all(OUTPUT)?;
    let a: Asset = serde_json::from_slice(&fs::read(Path::new(SOURCE).join("f1_d2.json"))?)?;
    let mut renderer = pollster::block_on(Renderer::new_automatic())?;
    let u = renderer.upload_from(&a, SOURCE)?;
    let mut quality_rows = Vec::new();
    let mut timings = Vec::new();
    let mut repeats = Vec::new();
    let mut preparations = Vec::new();
    for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
        if smoke && resolution == 0 {
            continue;
        }
        let cam = render::camera_at(14., size);
        let (reference, _) = renderer.capture(&u, &a, &cam, 80)?;
        let covered = reference.iter().filter(|p| p[3] > 0.).count() as u64;
        drop(reference);
        renderer.density_sources.clear();
        for (d, &target) in TARGETS.iter().enumerate() {
            if smoke && d > 0 {
                break;
            }
            let source = prepare(&renderer, &a, covered, target)?;
            ensure!(
                eligible(Some(source.error_world), &cam, a.bounds),
                "constructed source lacks eligible parent at14"
            );
            preparations.push(json!({"resolution":resolution,"size":size,"density":d,"metadata":source.metadata,"bound_px_at14":projected(source.error_world,&cam,a.bounds)}));
            println!(
                "prepared r{resolution} {target} triangles/pixel: {} triangles, {} bytes",
                source.triangles, source.metadata["geometry_bytes"]
            );
            renderer.density_sources.push(source);
        }
        fs::write(
            Path::new(OUTPUT).join(if smoke {
                "smoke-prepare.json"
            } else {
                "preparation.json"
            }),
            serde_json::to_vec_pretty(&preparations)?,
        )?;
        let saved_error = renderer.density_sources[0].error_world;
        renderer.density_sources[0].error_world = f64::INFINITY;
        let (fallback, fallback_triangles) = renderer.capture(&u, &a, &cam, modes(0)[2])?;
        let (full, full_triangles) = renderer.capture(&u, &a, &cam, modes(0)[1])?;
        ensure!(
            fallback == full && fallback_triangles == full_triangles,
            "uncertified fallback differs from full source"
        );
        renderer.density_sources[0].error_world = saved_error;
        drop(fallback);
        drop(full);
        for (frame, &distance) in DISTANCES.iter().enumerate() {
            let cam = render::camera_at(distance, size);
            let (reference, _) = renderer.capture(&u, &a, &cam, 80)?;
            let covered = reference.iter().filter(|p| p[3] > 0.).count();
            for (d, &target) in TARGETS
                .iter()
                .enumerate()
                .take(renderer.density_sources.len())
            {
                let id = format!("r{resolution}_d{d}_{frame}");
                let (full, full_triangles) = renderer.capture(&u, &a, &cam, modes(d)[1])?;
                let (adaptive, adaptive_triangles) = renderer.capture(&u, &a, &cam, modes(d)[2])?;
                let checks = [
                    quality(&reference, &full, &[]).0,
                    quality(&reference, &adaptive, &[]).0,
                ];
                let bound = projected(renderer.density_sources[d].error_world, &cam, a.bounds);
                for (i, image) in [&reference, &full, &adaptive].into_iter().enumerate() {
                    save(
                        &Path::new(OUTPUT).join(format!("{id}_{i}.png")),
                        image,
                        size[0],
                    )?;
                }
                quality_rows.push(json!({"id":id,"resolution":resolution,"size":size,"density":d,"frame":frame,"distance":distance,"target_at14":target,"source_triangles":full_triangles,"selected_triangles":adaptive_triangles,"covered_pixels":covered,"source_triangles_per_pixel":full_triangles as f64/covered as f64,"parent_bound_px":bound,"certified":eligible(Some(renderer.density_sources[d].error_world),&cam,a.bounds),"exact_parent_image":adaptive==reference,"invalid_certificate_fallback_checked":true,"checks":checks}));
                println!(
                    "capture {id}: checks {} {}",
                    checks[0]["quality_gate"], checks[1]["quality_gate"]
                );
            }
            fs::write(
                Path::new(OUTPUT).join(if smoke {
                    "smoke-quality.json"
                } else {
                    "quality.json"
                }),
                serde_json::to_vec_pretty(&quality_rows)?,
            )?;
        }
        if !smoke {
            timings.push(json!({"resolution":resolution,"size":size,"distance":14,"adapter":renderer.gpu.adapter,"samples":benchmark(&renderer,&u,&a,0)?}));
            fs::write(
                Path::new(OUTPUT).join("validation.json"),
                serde_json::to_vec_pretty(&timings)?,
            )?;
            println!("benchmark r{resolution}");
            repeats.push(json!({"resolution":resolution,"size":size,"distance":14,"adapter":renderer.gpu.adapter,"samples":benchmark(&renderer,&u,&a,1)?}));
            fs::write(
                Path::new(OUTPUT).join("repeat.json"),
                serde_json::to_vec_pretty(&repeats)?,
            )?;
            println!("repeat r{resolution}");
        }
        renderer.density_sources.clear();
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subdivision_preserves_area_winding_bounds_and_exact_count() {
        let p = [[0., 0., 0.], [2., 0., 0.], [0., 1., 3.]];
        let expected = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
            .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]));
        for n in [1, 2, 3, 7, 27] {
            let mut v = Vec::new();
            let mut i = Vec::new();
            let error = subdivide(p, n, &mut v, &mut i);
            assert_eq!(i.len(), (n * n * 3) as usize);
            assert_eq!(v.len(), ((n + 1) * (n + 2) / 2) as usize);
            let mut area = Vec3::ZERO;
            for t in i.chunks(3) {
                let a = Vec3::from_array(v[t[0] as usize]);
                let b = Vec3::from_array(v[t[1] as usize]);
                let c = Vec3::from_array(v[t[2] as usize]);
                let cross = (b - a).cross(c - a);
                assert!(cross.dot(expected) > 0.);
                area += cross;
            }
            assert!((area - expected).length() < 0.0001);
            assert!(error < 1e-6);
            for v in v {
                assert!(v[0] >= 0. && v[1] >= 0. && v[0] / 2. + v[1] <= 1.000001);
                assert!((v[2] - 3. * v[1]).abs() < 1e-6);
            }
        }
    }
    #[test]
    fn interior_refinement_keeps_shared_edges_and_uniform_grid_edges_match() {
        for splits in [0, 1, 2] {
            let mut v = vec![[0., 0., 0.], [2., 0., 0.], [0., 0., 2.]];
            let mut indices = Vec::new();
            let e = split_interior([0, 1, 2], splits, &mut v, &mut indices);
            assert_eq!(indices.len(), 3 * (1 + 2 * splits) as usize);
            assert!(e < 1e-6);
            let mut edges = std::collections::BTreeMap::new();
            let mut area = 0f32;
            for t in indices.chunks_exact(3) {
                let a = Vec3::from_array(v[t[0] as usize]);
                let b = Vec3::from_array(v[t[1] as usize]);
                let c = Vec3::from_array(v[t[2] as usize]);
                let cross = (b - a).cross(c - a);
                assert!(cross.y < 0.);
                area -= cross.y;
                for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                    *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                }
            }
            assert!((area - 4.).abs() < 1e-5);
            assert_eq!(
                edges
                    .iter()
                    .filter(|(_, n)| **n == 1)
                    .map(|(e, _)| *e)
                    .collect::<Vec<_>>(),
                vec![(0, 1), (0, 2), (1, 2)]
            );
            assert!(edges.values().all(|&n| n == 1 || n == 2));
        }
        let a = [-3.1467, 0.12345, 2.7192];
        let b = [1.23456, -0.4321, -2.34567];
        for n in [1, 2, 3, 8, 26, 27] {
            let mut v = Vec::new();
            let mut w = Vec::new();
            let mut i = Vec::new();
            subdivide([a, b, [0., 1., 0.]], n, &mut v, &mut i);
            i.clear();
            subdivide([b, a, [0., -1., 0.]], n, &mut w, &mut i);
            for j in 0..=n as usize {
                assert_eq!(v[j].map(f32::to_bits), w[n as usize - j].map(f32::to_bits));
            }
        }
    }
    #[test]
    fn density_plan_targets_physical_coverage_and_conservative_fallback() {
        for parents in [32, 327680] {
            for ratio in [1, 2, 10, 100] {
                let target = 2263904 * ratio;
                let p = plan(parents, target).unwrap();
                assert!(p.triangles.abs_diff(target) <= 1);
                assert_eq!(p.base + 2 * p.splits, p.triangles);
            }
        }
        assert!(plan(10, 9).is_err());
        let cam = render::camera_at(14., [3456, 2234]);
        let bounds = [[-6., -1., -6.], [6., 1., 6.]];
        assert!(eligible(Some(1e-6), &cam, bounds));
        assert!(!eligible(None, &cam, bounds));
        assert!(!eligible(Some(1.), &cam, bounds));
        assert!(!eligible(Some(f64::NAN), &cam, bounds));
        assert!(!eligible(
            Some(1e-6),
            &render::camera_at(1., [3456, 2234]),
            bounds
        ));
    }
}
