//! Incremental B versus B+response symbols; frozen calibration, held-out motion.
use super::material_symbol as symbol;
use super::*;
const ROOT: &str = symbol::ROOT;
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
const PACKED: &str = "artifacts/appearance-probe/near-plateau";
const BASE: u32 = 135;
const FORCE: u32 = 305;
const AUTO: u32 = 315;
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
    let yaw = 0.201
        + if scene == 1 {
            0.
        } else {
            0.32 * (a + offset).sin()
        };
    let elevation = 0.72
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
    let s = render::selection(a, c, 5);
    let native = a
        .sealed
        .last()
        .is_some_and(|l| s.1 == 1 && s.0[0] == (l.first, l.count));
    let depth = r
        .shading_model
        .as_ref()
        .unwrap()
        .depth(automatic::features(a, c, s.2, native));
    symbol::bucket(
        depth,
        cost::area(a, c) * 1e6 / (c.size[0] as f64 * c.size[1] as f64),
    )
    .unwrap()
}
fn enabled(r: &Renderer, a: &Asset, c: &Camera) -> bool {
    r.material_symbol.as_ref().unwrap().enabled & (1 << class(r, a, c)) != 0
}
fn policy(r: &mut Renderer, write: Option<u8>) -> Result<()> {
    let path = Path::new(ROOT).join("policy.bin");
    let adapter = symbol::hash(r.gpu.adapter.as_bytes());
    let model = symbol::hash(&fs::read(
        "artifacts/appearance-probe/automatic-shading/model.json",
    )?);
    let material = symbol::hash(&fs::read(Path::new(ROOT).join("material.ims"))?);
    if let Some(mask) = write {
        let mut b = b"ISP1".to_vec();
        b.extend([mask, 0, 0, 0]);
        b.extend(adapter.to_le_bytes());
        b.extend(model.to_le_bytes());
        b.extend(material.to_le_bytes());
        let sum = symbol::hash(&b);
        b.extend(sum.to_le_bytes());
        fs::write(&path, b)?;
    }
    let b = fs::read(&path)?;
    ensure!(
        b.len() == 40 && &b[..4] == b"ISP1" && b[5..8] == [0; 3],
        "policy layout/version"
    );
    let get = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    ensure!(
        get(8) == adapter
            && get(16) == model
            && get(24) == material
            && get(32) == symbol::hash(&b[..32]),
        "policy domain/checksum"
    );
    r.material_symbol.as_mut().unwrap().enabled = b[4];
    Ok(())
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Timing {
    mode: u32,
    duplicate: usize,
    total: Vec<f64>,
    gpu: Vec<f64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Cohort {
    density: usize,
    resolution: usize,
    distance: f32,
    classes: Vec<usize>,
    selected: usize,
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
            })
        })
        .collect::<Vec<_>>();
    let mut classes = Vec::new();
    let mut selected = 0;
    for i in 0..64 {
        let c = camera(distance, size, i as f32 / 64., 2, train);
        classes.push(class(r, a, &c));
        selected += usize::from(enabled(r, a, &c));
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
            r.wait(last.take().unwrap())?;
            samples[s]
                .total
                .push(start.elapsed().as_secs_f64() * 1000. / 64.);
            for i in 0..32 {
                let c = camera(distance, size, i as f32 / 32., 2, train);
                let mut e = r.gpu.device.create_command_encoder(&Default::default());
                r.encode(&mut e, u, a, &c, mode, Some(i * 2));
                last = Some(r.gpu.queue.submit([e.finish()]));
            }
            r.wait(last.take().unwrap())?;
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
            let stamps = bytemuck::cast_slice::<u8, u64>(&bytes);
            let mut sum = 0;
            for t in stamps.chunks_exact(2) {
                ensure!(t[1] >= t[0], "GPU clock");
                sum += t[1] - t[0];
            }
            samples[s]
                .gpu
                .push(sum as f64 * r.gpu.timestamp_period_ns.unwrap() as f64 / 32e6);
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
        samples,
    })
}
fn fit(r: &mut Renderer, rows: &[Cohort]) -> Result<()> {
    let mut reports = vec![];
    let mut mask = 0u8;
    for k in 0..8 {
        let mut ratios = vec![];
        let mut spreads = vec![];
        for row in rows.iter().filter(|r| r.classes.iter().all(|&v| v == k)) {
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
fn captures(r: &mut Renderer, smoke: bool) -> Result<()> {
    let mut rows = vec![];
    for density in 0..4 {
        if smoke && density != 3 {
            continue;
        }
        let a = asset(density)?;
        let mut u = r.upload_from(&a, SOURCE)?;
        r.upload_packed(&mut u, &a, PACKED)?;
        for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
            if smoke && resolution == 0 {
                continue;
            }
            if !smoke {
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
                if smoke && di > 0 {
                    continue;
                }
                for scene in 0..3 {
                    if scene != 2 && (density != 3 || di != 0) {
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
                        ensure!(triangles == ft && ft == ct, "geometry changed");
                        if frame == 0 && di == 0 {
                            let mask = r.material_symbol.as_ref().unwrap().enabled;
                            r.material_symbol.as_mut().unwrap().enabled = 255;
                            let (active, _) = r.capture(&u, &a, &c, AUTO)?;
                            ensure!(active == forced, "enabled path differs from forced");
                            r.material_symbol.as_mut().unwrap().enabled = mask;
                        }
                        let chosen = enabled(r, &a, &c);
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
                        rows.push(json!({"id":id,"density":density,"factor":a.factor,"resolution":resolution,"size":size,"distance":distance,"scene":scene,"frame":frame,"triangles":triangles,"source_triangles":a.levels.last().unwrap().count/3,"class":class(r,&a,&c),"symbol_selected":chosen,"checks":checks,"forced_vs_baseline":delta,"baseline_bytes":a.bytes,"eye":c.eye,"light":c.light}));
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
pub(super) fn run(command: &str) -> Result<()> {
    fs::create_dir_all(ROOT)?;
    if command == "--compile-combined-symbol" {
        return symbol::compile();
    }
    let mut r = pollster::block_on(Renderer::new_automatic())?;
    automatic::load(&mut r)?;
    r.load_material_symbol()?;
    let train = command == "--calibrate-combined-symbol";
    if !train && !command.contains("smoke") {
        policy(&mut r, None)?;
    }
    if command == "--capture-combined-symbol" || command == "--smoke-combined-symbol" {
        return captures(&mut r, command.contains("smoke"));
    }
    let mut rows = vec![];
    let repeat = command == "--repeat-combined-symbol";
    for density in 0..4 {
        let a = asset(density)?;
        let mut u = r.upload_from(&a, SOURCE)?;
        r.upload_packed(&mut u, &a, PACKED)?;
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
        &json!({"adapter":r.gpu.adapter,"resolutions":RESOLUTIONS,"baseline_mode":BASE,"forced_mode":FORCE,"optional_mode":AUTO,"symbol_mask":r.material_symbol.as_ref().unwrap().enabled,"batch_frames":64,"gpu_batch":32,"rounds":8,"duplicates":2}),
    )?;
    Ok(())
}
