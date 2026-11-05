//! Incremental B versus B+geometric fields; frozen calibration, held-out motion.
use super::geometric_field as symbol;
use super::*;
const ROOT: &str = symbol::ROOT;
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
const PACKED: &str = "artifacts/appearance-probe/near-plateau";
const BASE: u32 = 135;
const FORCE: u32 = 325;
const AUTO: u32 = 335;
fn save_json(name: &str, v: &impl serde::Serialize) -> Result<()> {
    fs::write(Path::new(ROOT).join(name), serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn asset(d: usize) -> Result<Asset> {
    Ok(serde_json::from_slice(&fs::read(
        Path::new(SOURCE).join(format!("f1_d{d}.json")),
    )?)?)
}
// Closed paths; calibration phase differs from validation, no pose lookup table.
fn camera(distance: f32, size: [u32; 2], phase: f32, scene: usize, train: bool) -> Camera {
    let a = phase * std::f32::consts::TAU;
    let offset = if train { 0.137 } else { 0. };
    let yaw = 0.201 + if scene == 1 { 0. } else { a + offset };
    let elevation = 0.95
        + if scene == 1 {
            0.
        } else {
            0.025 * (2. * a + offset).sin()
        };
    let distance = distance * (1. + if scene == 1 { 0. } else { 0.12 * a.sin() });
    let eye = Vec3::new(
        yaw.sin() * elevation.cos(),
        elevation.sin(),
        yaw.cos() * elevation.cos(),
    ) * distance;
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let matrix = Mat4::perspective_infinite_reverse_rh(
        60f32.to_radians(),
        size[0] as f32 / size[1] as f32,
        0.1,
    ) * view;
    let light_angle = 3.816 + if scene == 0 { 0. } else { a + offset };
    let light = Vec3::new(light_angle.sin() * 0.67, 0.74, light_angle.cos() * 0.67).normalize();
    Camera {
        matrix,
        inverse: matrix.inverse(),
        view,
        eye,
        size,
        light,
    }
}
fn class(r: &Renderer, a: &Asset, c: &Camera) -> usize {
    r.field_class(a, c)
}
fn enabled(r: &Renderer, u: &render::Uploaded, a: &Asset, c: &Camera) -> bool {
    r.field_selected(u, a, c, false)
}
fn policy(r: &mut Renderer, write: Option<u8>) -> Result<()> {
    let path = Path::new(ROOT).join("policy.bin");
    let adapter = material_symbol::hash(r.gpu.adapter.as_bytes());
    let model = material_symbol::hash(&fs::read(
        "artifacts/appearance-probe/automatic-shading/model.json",
    )?);
    let mut signature = Vec::new();
    for d in 0..4 {
        signature.extend(fs::read(Path::new(ROOT).join(format!("f1_d{d}.igf")))?);
    }
    signature.extend(include_bytes!("geometric_field.wgsl"));
    let material = material_symbol::hash(&signature);
    if let Some(mask) = write {
        let mut b = b"IGP1".to_vec();
        b.extend([mask, 0, 0, 0]);
        b.extend(adapter.to_le_bytes());
        b.extend(model.to_le_bytes());
        b.extend(material.to_le_bytes());
        let sum = material_symbol::hash(&b);
        b.extend(sum.to_le_bytes());
        fs::write(&path, b)?;
    }
    let b = fs::read(&path)?;
    ensure!(
        b.len() == 40 && &b[..4] == b"IGP1" && b[5..8] == [0; 3],
        "policy layout/version"
    );
    let get = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    ensure!(
        get(8) == adapter
            && get(16) == model
            && get(24) == material
            && get(32) == material_symbol::hash(&b[..32]),
        "policy domain/checksum"
    );
    r.geometric_field.as_mut().unwrap().enabled = b[4];
    Ok(())
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Timing {
    mode: u32,
    duplicate: usize,
    total: Vec<f64>,
    gpu: Vec<f64>,
    cpu_submit: Vec<f64>,
    #[serde(default)]
    instrumented_total: Vec<f64>,
    #[serde(default)]
    pass_latency: Vec<f64>,
    #[serde(default)]
    gpu_overlap_count: Vec<usize>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Cohort {
    density: usize,
    resolution: usize,
    distance: f32,
    classes: Vec<usize>,
    selected: usize,
    eligible: usize,
    samples: Vec<Timing>,
}
fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    s[s.len() / 2]
}
fn measure(
    r: &mut Renderer,
    u: &render::Uploaded,
    a: &Asset,
    resolution: usize,
    density: usize,
    distance: f32,
    train: bool,
) -> Result<Cohort> {
    let size = RESOLUTIONS[resolution];
    r.resize(size);
    let modes = if train {
        vec![BASE, FORCE]
    } else {
        vec![BASE, FORCE, AUTO]
    };
    let mut samples = (0..2)
        .flat_map(|duplicate| {
            modes.iter().map(move |&mode| Timing {
                mode,
                duplicate,
                total: vec![],
                gpu: vec![],
                cpu_submit: vec![],
                instrumented_total: vec![],
                pass_latency: vec![],
                gpu_overlap_count: vec![],
            })
        })
        .collect::<Vec<_>>();
    let mut classes = Vec::new();
    let mut selected = 0;
    let mut eligible = 0;
    for i in 0..64 {
        let c = camera(distance, size, i as f32 / 64., 2, train);
        classes.push(class(r, a, &c));
        eligible += usize::from(r.field_selected(u, a, &c, true));
        selected += usize::from(enabled(r, u, a, &c));
    }
    for &mode in &modes {
        let mut last = None;
        for i in 0..32 {
            let c = camera(distance, size, i as f32 / 32., 2, train);
            let mut e = r.gpu.device.create_command_encoder(&Default::default());
            r.encode(&mut e, u, a, &c, mode, None);
            last = Some(r.gpu.queue.submit([e.finish()]));
        }
        r.wait(last.take().unwrap())?;
    }
    for round in 0..8 {
        for offset in 0..samples.len() {
            let s = if round % 2 == 0 {
                (offset + round) % samples.len()
            } else {
                (samples.len() - 1 - offset + round) % samples.len()
            };
            let mode = samples[s].mode;
            let start = Instant::now();
            let mut last = None;
            for i in 0..64 {
                let c = camera(distance, size, i as f32 / 64., 2, train);
                let mut e = r.gpu.device.create_command_encoder(&Default::default());
                r.encode(&mut e, u, a, &c, mode, None);
                last = Some(r.gpu.queue.submit([e.finish()]));
            }
            let cpu_ms = start.elapsed().as_secs_f64() * 1000. / 64.;
            r.wait(last.take().unwrap())?;
            samples[s].cpu_submit.push(cpu_ms);
            samples[s]
                .total
                .push(start.elapsed().as_secs_f64() * 1000. / 64.);
            let instrumented_start = Instant::now();
            for i in 0..32 {
                let c = camera(distance, size, i as f32 / 32., 2, train);
                let mut e = r.gpu.device.create_command_encoder(&Default::default());
                r.encode(&mut e, u, a, &c, mode, Some(i * 2));
                last = Some(r.gpu.queue.submit([e.finish()]));
            }
            r.wait(last.take().unwrap())?;
            samples[s]
                .instrumented_total
                .push(instrumented_start.elapsed().as_secs_f64() * 1000. / 32.);
            let mut e = r.gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(
                r.gpu.query.as_ref().unwrap(),
                0..64,
                &r.gpu.query_resolve,
                0,
            );
            e.copy_buffer_to_buffer(&r.gpu.query_resolve, 0, &r.gpu.query_readback, 0, 512);
            r.wait(r.gpu.queue.submit([e.finish()]))?;
            r.gpu.map(&r.gpu.query_readback)?;
            let bytes = r.gpu.query_readback.slice(..).get_mapped_range();
            let stamps = bytemuck::cast_slice::<u8, u64>(&bytes[..512]);
            let mut sum = 0;
            for t in stamps.chunks_exact(2) {
                ensure!(t[1] >= t[0], "GPU clock");
                sum += t[1] - t[0];
            }
            let unit = r.gpu.timestamp_period_ns.unwrap() as f64 / 32e6;
            // On Metal pass intervals can overlap across queued submissions.
            // Summing their latency is not GPU throughput. Use the batch envelope.
            let first = stamps.chunks_exact(2).map(|p| p[0]).min().unwrap();
            let last = stamps.chunks_exact(2).map(|p| p[1]).max().unwrap();
            samples[s].gpu.push((last - first) as f64 * unit);
            samples[s].pass_latency.push(sum as f64 * unit);
            samples[s].gpu_overlap_count.push(
                (1..32)
                    .filter(|i| stamps[2 * i] < stamps[2 * i - 1])
                    .count(),
            );
            drop(bytes);
            r.gpu.query_readback.unmap();
        }
    }
    Ok(Cohort {
        density,
        resolution,
        distance,
        classes,
        selected,
        eligible,
        samples,
    })
}
fn fit(r: &mut Renderer, rows: &[Cohort]) -> Result<()> {
    let mut reports = vec![];
    let mut mask = 0u8;
    for k in 0..4 {
        let mut ratios = vec![];
        let mut spreads = vec![];
        for row in rows
            .iter()
            .filter(|r| r.eligible == 64 && r.classes.iter().all(|&v| v == k))
        {
            let b = row
                .samples
                .iter()
                .filter(|s| s.mode == BASE)
                .map(|s| median(&s.total))
                .collect::<Vec<_>>();
            let d = row
                .samples
                .iter()
                .filter(|s| s.mode == FORCE)
                .map(|s| median(&s.total))
                .collect::<Vec<_>>();
            ratios.push((d[0] + d[1]) / (b[0] + b[1]));
            spreads.push((b[0] / b[1] - 1.).abs().max((d[0] / d[1] - 1.).abs()));
        }
        let worst = ratios.iter().copied().fold(0f64, f64::max);
        let spread = spreads.iter().copied().fold(0f64, f64::max);
        let accept = ratios.len() >= 3 && worst < 1. - 0.05f64.max(spread);
        if accept {
            mask |= 1 << k;
        }
        reports.push(json!({"class":k,"cohorts":ratios.len(),"ratios":ratios,"max_duplicate_spread":spread,"accepted":accept}));
    }
    policy(r, Some(mask))?;
    save_json(
        "fit.json",
        &json!({"enabled_mask":mask,"classes":reports,"calibration_only":true,"minimum_benefit":0.05}),
    )
}
fn captures(r: &mut Renderer, smoke: bool, motion: bool) -> Result<()> {
    let mut rows: Vec<serde_json::Value> = if motion {
        serde_json::from_slice(&fs::read(Path::new(ROOT).join("quality.json"))?)?
    } else {
        vec![]
    };
    if motion {
        rows.retain(|row| !(row["density"] == 3 && row["distance"] == 28. && row["scene"] != 2));
    }
    for density in 0..4 {
        if (smoke || motion) && density != 3 {
            continue;
        }
        let a = asset(density)?;
        let mut u = r.upload_from(&a, SOURCE)?;
        r.upload_packed(&mut u, &a, PACKED)?;
        r.upload_field(&mut u, &a)?;
        for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
            if smoke && resolution == 0 {
                continue;
            }
            if !smoke && !motion {
                let old = render::camera_at(14., size);
                let (image, _) = r.capture(&u, &a, &old, BASE)?;
                let path = Path::new(ROOT).join(format!("legacy_d{density}_r{resolution}.png"));
                save(&path, &image, size[0])?;
                ensure!(
                    fs::read(&path)?
                        == fs::read(
                            Path::new(PACKED).join(format!("f1_d{density}_r{resolution}_1_5.png"))
                        )?,
                    "baseline legacy changed"
                );
            }
            for (di, distance) in [14., 28., 224.].into_iter().enumerate() {
                if (smoke || motion) && di != 1 {
                    continue;
                }
                for scene in 0..3 {
                    if motion && scene == 2 {
                        continue;
                    }
                    if scene != 2 && (density != 3 || di != if motion { 1 } else { 0 }) {
                        continue;
                    }
                    if smoke && scene != 2 {
                        continue;
                    }
                    let mut previous: Vec<Vec<[f32; 4]>> = vec![vec![], vec![], vec![]];
                    let mut first: Vec<Vec<[f32; 4]>> = vec![];
                    for frame in 0..=8 {
                        if smoke && frame > 0 {
                            continue;
                        }
                        let c = camera(distance, size, (frame % 8) as f32 / 8., scene, false);
                        let id = format!("d{density}_r{resolution}_z{di}_s{scene}_f{frame}");
                        let (full, _) = r.capture(&u, &a, &c, 0)?;
                        let (baseline, triangles) = r.capture(&u, &a, &c, BASE)?;
                        let (forced, ft) = r.capture(&u, &a, &c, FORCE)?;
                        let (automatic, ct) = r.capture(&u, &a, &c, AUTO)?;
                        let eligible = r.field_selected(&u, &a, &c, true);
                        ensure!(
                            ft == if eligible { 1 } else { triangles },
                            "forced geometry count"
                        );
                        ensure!(
                            ct == if enabled(r, &u, &a, &c) { 1 } else { triangles },
                            "optional geometry count"
                        );
                        if frame == 0 && di == 0 {
                            let mask = r.geometric_field.as_ref().unwrap().enabled;
                            r.geometric_field.as_mut().unwrap().enabled = 255;
                            let (active, _) = r.capture(&u, &a, &c, AUTO)?;
                            ensure!(active == forced, "enabled path differs from forced");
                            r.geometric_field.as_mut().unwrap().enabled = mask;
                        }
                        let chosen = enabled(r, &u, &a, &c);
                        ensure!(
                            automatic == *if chosen { &forced } else { &baseline },
                            "selection pixels"
                        );
                        if frame == 0 {
                            first = vec![
                                full.clone(),
                                baseline.clone(),
                                forced.clone(),
                                automatic.clone(),
                            ];
                        }
                        if frame == 8 {
                            ensure!(
                                first
                                    == [
                                        full.clone(),
                                        baseline.clone(),
                                        forced.clone(),
                                        automatic.clone()
                                    ],
                                "closed path mismatch"
                            );
                        }
                        let mut checks = vec![];
                        for (k, img) in [&baseline, &forced, &automatic].into_iter().enumerate() {
                            let (q, residual) = quality(&full, img, &previous[k]);
                            previous[k] = residual;
                            checks.push(q);
                        }
                        let delta = quality(&baseline, &forced, &[]).0;
                        for (code, img) in [(0, &full), (BASE, &baseline), (FORCE, &forced)] {
                            save(
                                &Path::new(ROOT).join(format!("{id}_{code}.png")),
                                img,
                                size[0],
                            )?;
                        }
                        rows.push(json!({"id":id,"density":density,"factor":a.factor,"resolution":resolution,"size":size,"distance":distance,"scene":scene,"frame":frame,"triangles":triangles,"forced_triangles":ft,"optional_triangles":ct,"eligible":eligible,"rho":u.field.as_ref().unwrap().field.rho(&c),"source_triangles":a.levels.last().unwrap().count/3,"class":class(r,&a,&c),"symbol_selected":chosen,"checks":checks,"forced_vs_baseline":delta,"baseline_bytes":a.bytes,"eye":c.eye,"light":c.light}));
                        println!(
                            "capture {id}: chosen={chosen} gates={:?}",
                            rows.last().unwrap()["checks"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|q| q["quality_gate"].clone())
                                .collect::<Vec<_>>()
                        );
                        save_json(if smoke { "smoke.json" } else { "quality.json" }, &rows)?;
                    }
                }
            }
        }
    }
    Ok(())
}
fn depth_checks(r: &mut Renderer) -> Result<()> {
    let a = asset(3)?;
    let mut u = r.upload_from(&a, SOURCE)?;
    r.upload_packed(&mut u, &a, PACKED)?;
    r.upload_field(&mut u, &a)?;
    let mut rows = vec![];
    for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
        for frame in 0..9 {
            let c = camera(28., size, (frame % 8) as f32 / 8., 2, false);
            let eligible = r.field_selected(&u, &a, &c, true);
            let (full, _) = r.capture(&u, &a, &c, 0)?;
            let a_depth = r.read_field_depth()?;
            let (symbol, _) = r.capture(&u, &a, &c, FORCE)?;
            let s_depth = r.read_field_depth()?;
            let mut max = 0f32;
            let mut sum = 0f64;
            let mut count = 0;
            for i in 0..full.len() {
                ensure!((s_depth[i] > 0.) == (symbol[i][3] > 0.), "depth coverage");
                if full[i][3] > 0. && symbol[i][3] > 0. {
                    let error = (a_depth[i] - s_depth[i]).abs();
                    max = max.max(error);
                    sum += (error as f64).powi(2);
                    count += 1;
                }
            }
            ensure!(count > 0 && max < 0.00001, "field depth divergence {max}");
            rows.push(json!({"resolution":resolution,"frame":frame,"symbol_active":eligible,"matched_pixels":count,"max_depth_error":max,"rms_depth_error":(sum/count as f64).sqrt(),"depth_coverage_matches_color":true}));
        }
    }
    save_json("depth.json", &rows)
}
pub(super) fn run(command: &str) -> Result<()> {
    fs::create_dir_all(ROOT)?;
    if command == "--compile-geometric-field" {
        return symbol::compile();
    }
    let mut r = pollster::block_on(Renderer::new_automatic())?;
    automatic::load(&mut r)?;
    r.load_geometric_field();
    if command == "--depth-geometric-field" {
        return depth_checks(&mut r);
    }
    let train = command == "--calibrate-geometric-field";
    if !train && !command.contains("smoke") {
        policy(&mut r, None)?;
    }
    if command == "--capture-geometric-field"
        || command == "--smoke-geometric-field"
        || command == "--motion-geometric-field"
    {
        return captures(
            &mut r,
            command.contains("smoke"),
            command.contains("motion"),
        );
    }
    if command == "--audit-geometric-field" {
        let mut rows = vec![];
        for density in [0, 3] {
            let a = asset(density)?;
            let mut u = r.upload_from(&a, SOURCE)?;
            r.upload_packed(&mut u, &a, PACKED)?;
            r.upload_field(&mut u, &a)?;
            for resolution in 0..2 {
                let distance = if density == 0 { 14. } else { 28. };
                rows.push(measure(
                    &mut r, &u, &a, resolution, density, distance, false,
                )?);
            }
        }
        return save_json("timestamp-audit.json", &rows);
    }
    let mut rows = vec![];
    let repeat = command == "--repeat-geometric-field";
    for density in 0..4 {
        let a = asset(density)?;
        let mut u = r.upload_from(&a, SOURCE)?;
        r.upload_packed(&mut u, &a, PACKED)?;
        r.upload_field(&mut u, &a)?;
        for resolution in 0..2 {
            let distances: &[f32] = if train {
                &[16., 48., 192.]
            } else if repeat {
                &[14., 224.]
            } else {
                &[14., 28., 224.]
            };
            for &distance in distances {
                rows.push(measure(
                    &mut r, &u, &a, resolution, density, distance, train,
                )?);
                save_json(
                    if train {
                        "calibration.json"
                    } else if repeat {
                        "repeat.json"
                    } else {
                        "timings.json"
                    },
                    &rows,
                )?;
                println!("{command} density{density} resolution{resolution} distance{distance}");
            }
        }
    }
    if train {
        fit(&mut r, &rows)?;
    }
    save_json(
        "environment.json",
        &json!({"adapter":r.gpu.adapter,"resolutions":RESOLUTIONS,"baseline_mode":BASE,"forced_mode":FORCE,"optional_mode":AUTO,"symbol_mask":r.geometric_field.as_ref().unwrap().enabled,"batch_frames":64,"gpu_batch":32,"rounds":8,"duplicates":2}),
    )?;
    Ok(())
}
