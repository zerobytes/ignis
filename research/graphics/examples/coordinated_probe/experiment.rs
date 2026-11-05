use super::{
    compiler, field,
    render::{Bound, Gpu, Pose},
    select,
};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::path::Path;
#[derive(Serialize)]
struct Quality {
    rmse: f64,
    p99: f32,
    max: f32,
    coverage_error: f64,
    covered: usize,
    pass: bool,
}
fn quality(a: &[[f32; 4]], b: &[[f32; 4]]) -> Quality {
    let mut es = vec![];
    let mut sum = 0.;
    let mut mismatch = 0;
    let mut covered = 0;
    for (a, b) in a.iter().zip(b) {
        if a[3] > 0.5 || b[3] > 0.5 {
            covered += 1;
            mismatch += usize::from((a[3] > 0.5) != (b[3] > 0.5));
            for i in 0..3 {
                let e = (a[i] - b[i]).abs();
                es.push(e);
                sum += (e as f64).powi(2);
            }
        }
    }
    es.sort_unstable_by(f32::total_cmp);
    let rmse = (sum / (covered * 3).max(1) as f64).sqrt();
    let p99 = es.get(es.len() * 99 / 100).copied().unwrap_or(0.);
    let coverage_error = mismatch as f64 / covered.max(1) as f64;
    Quality {
        rmse,
        p99,
        max: es.last().copied().unwrap_or(0.),
        coverage_error,
        covered,
        pass: rmse <= 0.005 && p99 <= 0.02 && coverage_error <= 0.0005,
    }
}
fn png(path: &Path, pixels: &[[f32; 4]], width: u32, height: u32) -> Result<()> {
    let mut rgb = vec![];
    for p in pixels {
        for &v in &p[..3] {
            let s = if v <= 0.0031308 {
                v * 12.92
            } else {
                1.055 * v.max(0.).powf(1. / 2.4) - 0.055
            };
            rgb.push((s.clamp(0., 1.) * 255.).round() as u8);
        }
    }
    let mut e = png::Encoder::new(std::fs::File::create(path)?, width, height);
    e.set_color(png::ColorType::Rgb);
    e.write_header()?.write_image_data(&rgb)?;
    Ok(())
}
pub fn run(dir: &Path, smoke: bool) -> Result<()> {
    let canonical = compiler::decode(&std::fs::read(dir.join("source-1.igc"))?)?.source;
    let mut rows = vec![];
    let mut timings = vec![];
    for factor in if smoke { vec![1] } else { vec![1, 2, 10, 100] } {
        let source = compiler::decode(&std::fs::read(dir.join(format!("source-{factor}.igc")))?)?;
        let recovery_bound = Bound::new(&source);
        let old = field::decode(&std::fs::read(dir.join(format!("old-{factor}.inf")))?)?;
        let new = field::decode(&std::fs::read(
            dir.join(format!("coordinated-{factor}.inf")),
        )?)?;
        for (width, height) in if smoke {
            vec![(864, 558)]
        } else {
            vec![(1728, 1117), (3456, 2234)]
        } {
            let mut gpu = pollster::block_on(Gpu::new(width, height))?;
            gpu.install_normal_field(&old.pixels, old.width, old.height);
            let old_binding = gpu.normal_binding();
            gpu.install_normal_field(&new.pixels, new.width, new.height);
            let new_binding = gpu.normal_binding();
            let canonical_buffer = gpu.upload(&canonical);
            let canonical_range = vec![(0, canonical.faces.len() as u32 * 3)];
            let src = gpu.upload(&source.source);
            let recovered = gpu.upload(&source.proxy);
            let old_buf = gpu.upload_with_uv(&old.asset.mesh, &old.uv);
            let new_buf = gpu.upload_with_uv(&new.asset.mesh, &new.uv);
            let full = vec![(0, source.source.faces.len() as u32 * 3)];
            let recovered_range = vec![(0, source.proxy.faces.len() as u32 * 3)];
            for (di, distance) in [2.7, 64.].into_iter().enumerate() {
                let mut first = None;
                for frame in 0..if smoke { 4 } else { 9 } {
                    let phase = (frame % 8) as f32 * std::f32::consts::TAU / 8.;
                    let pose = Pose {
                        angle: phase,
                        distance,
                        light: phase * 1.7 + 0.3,
                    };
                    ensure!(
                        recovery_bound.pixels(pose, height) <= 0.15
                            && recovery_bound.normal_error <= 0.000101,
                        "recovery outside tested quality domain"
                    );
                    let a = select::choose_with_normal(&old.asset, pose, height, 0.15, false, 2.);
                    let b = select::choose_with_normal(&new.asset, pose, height, 0.15, false, 2.);
                    gpu.enable_indirect(false);
                    gpu.enable_normal_field(false);
                    let reference = gpu.capture_selected(&src, pose, Some(&full))?;
                    let density_reference =
                        gpu.capture_selected(&canonical_buffer, pose, Some(&canonical_range))?;
                    let density_quality = quality(&density_reference, &reference);
                    if !density_quality.pass {
                        eprintln!(
                            "invalid density control: factor={factor} width={width} distance={distance} frame={frame}: {}",
                            serde_json::to_string(&density_quality)?
                        );
                        png(
                            &dir.join(format!("density-{factor}-{width}-{di}-{frame}-base.png")),
                            &density_reference,
                            width,
                            height,
                        )?;
                        png(
                            &dir.join(format!(
                                "density-{factor}-{width}-{di}-{frame}-subdivided.png"
                            )),
                            &reference,
                            width,
                            height,
                        )?;
                    }
                    let plain = gpu.capture_selected(&recovered, pose, Some(&recovered_range))?;
                    let geometry = gpu.capture_selected(&new_buf, pose, Some(&b.ranges))?;
                    gpu.select_normal_binding(&old_binding);
                    gpu.enable_normal_field(true);
                    let previous = gpu.capture_selected(&old_buf, pose, Some(&a.ranges))?;
                    gpu.select_normal_binding(&new_binding);
                    let candidate = gpu.capture_selected(&new_buf, pose, Some(&b.ranges))?;
                    gpu.enable_indirect(true);
                    let grouped = gpu.capture_selected(&new_buf, pose, Some(&b.ranges))?;
                    ensure!(
                        bytemuck::cast_slice::<[f32; 4], u8>(&candidate)
                            == bytemuck::cast_slice::<[f32; 4], u8>(&grouped),
                        "indirect pixels changed"
                    );
                    let hashes = (
                        compiler::hash(bytemuck::cast_slice(&reference)),
                        compiler::hash(bytemuck::cast_slice(&candidate)),
                    );
                    if frame == 0 {
                        first = Some(hashes);
                    }
                    if frame == 8 {
                        ensure!(first == Some(hashes), "return changed");
                    }
                    rows.push(serde_json::json!({"factor":factor,"width":width,"height":height,"distance":distance,"frame":frame,"source_triangles":source.source.faces.len(),"recovered_triangles":source.proxy.faces.len(),"recovered":quality(&reference,&plain),"old":quality(&reference,&previous),"geometry":quality(&reference,&geometry),"coordinated":quality(&reference,&candidate),"indirect_identical":true,"density_reference":density_quality,"old_selection":a,"selection":b}));
                    if frame % 2 == 0 {
                        for (name, pixels) in [
                            ("source", &reference),
                            ("recovered", &plain),
                            ("old", &previous),
                            ("geometry", &geometry),
                            ("coordinated", &candidate),
                        ] {
                            png(
                                &dir.join(format!("{factor}-{width}-{di}-{frame}-{name}.png")),
                                pixels,
                                width,
                                height,
                            )?;
                        }
                    }
                }
                if std::env::var_os("COORDINATED_QUALITY_ONLY").is_some() {
                    continue;
                }
                let poses = (0..32)
                    .map(|i| {
                        let phase = i as f32 * std::f32::consts::TAU / 32.;
                        Pose {
                            angle: phase,
                            distance,
                            light: phase * 1.7 + 0.3,
                        }
                    })
                    .collect::<Vec<_>>();
                // Both atlases remain resident; only bindings switch between modes.
                for round in 0..if smoke { 2 } else { 8 } {
                    for slot in 0..10 {
                        let mode = if round % 2 == 0 {
                            [0, 1, 2, 3, 4, 4, 3, 2, 1, 0][slot]
                        } else {
                            [4, 3, 2, 1, 0, 0, 1, 2, 3, 4][slot]
                        };
                        gpu.enable_indirect(mode == 4);
                        gpu.enable_normal_field(mode >= 2);
                        if mode == 2 {
                            gpu.select_normal_binding(&old_binding);
                        } else if mode >= 3 {
                            gpu.select_normal_binding(&new_binding);
                        }
                        let buffer = match mode {
                            0 => &src,
                            1 => &recovered,
                            2 => &old_buf,
                            _ => &new_buf,
                        };
                        let choose = |p| match mode {
                            0 => full.clone(),
                            1 => recovered_range.clone(),
                            2 => {
                                select::choose_with_normal(&old.asset, p, height, 0.15, false, 2.)
                                    .ranges
                            }
                            _ => {
                                select::choose_with_normal(&new.asset, p, height, 0.15, false, 2.)
                                    .ranges
                            }
                        };
                        let _ = gpu.batch_selected(buffer, &poses, choose)?;
                        let timing = gpu.batch_selected(buffer, &poses, choose)?;
                        timings.push(serde_json::json!({"factor":factor,"width":width,"height":height,"distance":distance,"round":round,"slot":slot,"mode":(["source","recovered","old","coordinated","indirect"][mode]),"timing":timing}));
                    }
                }
                println!("factor{factor} {width} d{distance} complete");
            }
        }
    }
    std::fs::write(
        dir.join(if std::env::var_os("COORDINATED_QUALITY_ONLY").is_some() {
            "quality-only.json"
        } else if smoke {
            "smoke.json"
        } else {
            "results.json"
        }),
        serde_json::to_vec_pretty(&serde_json::json!({"quality":rows,"timings":timings}))?,
    )?;
    Ok(())
}
