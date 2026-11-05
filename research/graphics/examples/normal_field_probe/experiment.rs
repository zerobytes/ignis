use super::{
    field,
    render::{Gpu, Pose},
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
pub fn run(dir: &Path, files: &[String], smoke: bool) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let mut rows = vec![];
    let mut timings = vec![];
    let mut memory = vec![];
    for file in files {
        let bytes = std::fs::read(file)?;
        let field = field::decode(&bytes)?;
        let name = Path::new(file)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        memory.push(serde_json::json!({"name":name,"file_bytes":bytes.len(),"texture_bytes":field.pixels.len(),"gpu_geometry_uv_bytes":field.asset.mesh.vertices.len()*32+field.asset.mesh.faces.len()*12,"width":field.width,"height":field.height}));
        for (width, height) in if smoke {
            vec![(864, 558)]
        } else {
            vec![(1728, 1117), (3456, 2234)]
        } {
            let mut gpu = pollster::block_on(Gpu::new(width, height))?;
            gpu.install_normal_field(&field.pixels, field.width, field.height);
            let uploaded = gpu.upload_with_uv(&field.asset.mesh, &field.uv);
            let full = vec![(0, field.asset.source_count)];
            let mut failures = [0; 3];
            for (di, distance) in [2.7, 8., 32., 64.].into_iter().enumerate() {
                let mut first = None;
                for frame in 0..if smoke { 4 } else { 9 } {
                    let phase = (frame % 8) as f32 * std::f32::consts::TAU / 8.;
                    let pose = Pose {
                        angle: phase,
                        distance,
                        light: phase * 1.7 + 0.3,
                    };
                    let prior = select::choose(&field.asset, pose, height, 0.15, false);
                    let reduced =
                        select::choose_with_normal(&field.asset, pose, height, 0.15, false, 2.);
                    gpu.enable_normal_field(false);
                    let reference = gpu.capture_selected(&uploaded, pose, Some(&full))?;
                    let conservative =
                        gpu.capture_selected(&uploaded, pose, Some(&prior.ranges))?;
                    let coarse = gpu.capture_selected(&uploaded, pose, Some(&reduced.ranges))?;
                    gpu.enable_normal_field(true);
                    let sampled = gpu.capture_selected(&uploaded, pose, Some(&reduced.ranges))?;
                    let hashes = (
                        super::compiler::hash(bytemuck::cast_slice(&reference)),
                        super::compiler::hash(bytemuck::cast_slice(&sampled)),
                    );
                    if frame == 0 {
                        first = Some(hashes);
                    }
                    if frame == 8 {
                        ensure!(first == Some(hashes), "orbit return changed");
                    }
                    let a = quality(&reference, &conservative);
                    let b = quality(&reference, &coarse);
                    let c = quality(&reference, &sampled);
                    ensure!(a.pass, "prior conservative path regressed");
                    failures[0] += usize::from(!a.pass);
                    failures[1] += usize::from(!b.pass);
                    failures[2] += usize::from(!c.pass);
                    rows.push(serde_json::json!({"name":name,"width":width,"height":height,"distance":distance,"frame":frame,"prior":a,"reduced":b,"field":c,"selection":reduced,"prior_triangles":prior.triangles}));
                    if frame % 2 == 0 {
                        for (kind, pixels) in [
                            ("source", &reference),
                            ("reduced", &coarse),
                            ("field", &sampled),
                        ] {
                            png(
                                &dir.join(format!("{name}-{width}-{di}-{frame}-{kind}.png")),
                                pixels,
                                width,
                                height,
                            )?;
                        }
                    }
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
                // Warm every path before alternating duplicate measurements.
                for mode in 0..4 {
                    gpu.enable_normal_field(mode == 3);
                    let _ = gpu.batch_selected(&uploaded, &poses, |p| match mode {
                        0 => full.clone(),
                        1 => select::choose(&field.asset, p, height, 0.15, false).ranges,
                        _ => {
                            select::choose_with_normal(&field.asset, p, height, 0.15, false, 2.)
                                .ranges
                        }
                    })?;
                }
                for round in 0..if smoke { 1 } else { 8 } {
                    for slot in 0..8 {
                        let mode = if round % 2 == 0 {
                            [0, 1, 2, 3, 3, 2, 1, 0][slot]
                        } else {
                            [3, 2, 1, 0, 0, 1, 2, 3][slot]
                        };
                        gpu.enable_normal_field(mode == 3);
                        let timing = gpu.batch_selected(&uploaded, &poses, |p| match mode {
                            0 => full.clone(),
                            1 => select::choose(&field.asset, p, height, 0.15, false).ranges,
                            _ => {
                                select::choose_with_normal(&field.asset, p, height, 0.15, false, 2.)
                                    .ranges
                            }
                        })?;
                        timings.push(serde_json::json!({"name":name,"width":width,"distance":distance,"round":round,"slot":slot,"mode":(["source","prior","reduced","field"][mode]),"timing":timing}));
                    }
                }
                println!("{name} {width} d{distance}: failures prior/reduced/field {failures:?}");
            }
            if !smoke {
                let mut first = None;
                for frame in 0..=24 {
                    let phase = if frame == 24 {
                        0.
                    } else {
                        frame as f32 * std::f32::consts::TAU / 24.
                    };
                    let distance = 2.7 * ((64f32 / 2.7).ln() * (phase * 0.5).sin()).exp();
                    let pose = Pose {
                        angle: phase,
                        distance,
                        light: phase * 1.7 + 0.3,
                    };
                    let prior = select::choose(&field.asset, pose, height, 0.15, false);
                    let reduced =
                        select::choose_with_normal(&field.asset, pose, height, 0.15, false, 2.);
                    gpu.enable_normal_field(false);
                    let reference = gpu.capture_selected(&uploaded, pose, Some(&full))?;
                    let conservative =
                        gpu.capture_selected(&uploaded, pose, Some(&prior.ranges))?;
                    let coarse = gpu.capture_selected(&uploaded, pose, Some(&reduced.ranges))?;
                    gpu.enable_normal_field(true);
                    let sampled = gpu.capture_selected(&uploaded, pose, Some(&reduced.ranges))?;
                    let hashes = (
                        super::compiler::hash(bytemuck::cast_slice(&reference)),
                        super::compiler::hash(bytemuck::cast_slice(&sampled)),
                    );
                    if frame == 0 {
                        first = Some(hashes);
                    }
                    if frame == 24 {
                        ensure!(first == Some(hashes), "trajectory return changed");
                    }
                    let a = quality(&reference, &conservative);
                    ensure!(a.pass, "prior regression");
                    rows.push(serde_json::json!({"trajectory":true,"name":name,"width":width,"height":height,"distance":distance,"frame":frame,"prior":a,"reduced":quality(&reference,&coarse),"field":quality(&reference,&sampled),"selection":reduced,"prior_triangles":prior.triangles}));
                }
            }
        }
    }
    std::fs::write(
        dir.join(if smoke { "smoke.json" } else { "results.json" }),
        serde_json::to_vec_pretty(
            &serde_json::json!({"quality":rows,"timings":timings,"memory":memory}),
        )?,
    )?;
    Ok(())
}
