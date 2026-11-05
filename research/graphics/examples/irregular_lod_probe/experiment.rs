use super::{
    compile,
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
pub fn run(file: &Path, dir: &Path, smoke: bool) -> Result<()> {
    let bytes = std::fs::read(file)?;
    let asset = compile::decode(&bytes)?;
    std::fs::create_dir_all(dir)?;
    let mut rows = vec![];
    let mut timings = vec![];
    let mut failed = 0;
    for (width, height) in if smoke {
        vec![(864, 558)]
    } else {
        vec![(1728, 1117), (3456, 2234)]
    } {
        let gpu = pollster::block_on(Gpu::new(width, height))?;
        let uploaded = gpu.upload(&asset.mesh);
        let full = vec![(0, asset.source_count)];
        println!(
            "{} {width}x{height}, {} triangles, {} bytes resident",
            gpu.adapter,
            asset.source_count / 3,
            uploaded.bytes
        );
        for (di, distance) in [2.7, 4., 8., 16., 32., 64.].into_iter().enumerate() {
            for frame in 0..if smoke { 2 } else { 9 } {
                let phase = (frame % 8) as f32 * std::f32::consts::TAU / 8.;
                let pose = Pose {
                    angle: phase,
                    distance,
                    light: phase * 1.7 + 0.3,
                };
                let choice = select::choose(&asset, pose, height, 0.15, false);
                let a = gpu.capture_selected(&uploaded, pose, Some(&full))?;
                let b = gpu.capture_selected(&uploaded, pose, Some(&choice.ranges))?;
                let q = quality(&a, &b);
                failed += usize::from(!q.pass);
                rows.push(serde_json::json!({"width":width,"height":height,"distance":distance,"frame":frame,"quality":q,"selection":choice}));
                if frame % 2 == 0 {
                    let control =
                        select::choose_with_normal(&asset, pose, height, 0.15, false, 2.0);
                    let c = gpu.capture_selected(&uploaded, pose, Some(&control.ranges))?;
                    rows.push(serde_json::json!({"geometry_only_control":true,"width":width,"height":height,"distance":distance,"frame":frame,"quality":quality(&a,&c),"selection":control}));
                    png(
                        &dir.join(format!("{width}-{di}-{frame}-geometry-only.png")),
                        &c,
                        width,
                        height,
                    )?;
                }
                if frame % 2 == 0 {
                    for (name, pixels) in [("source", &a), ("adaptive", &b)] {
                        png(
                            &dir.join(format!("{width}-{di}-{frame}-{name}.png")),
                            pixels,
                            width,
                            height,
                        )?;
                    }
                }
            }
            let poses = (0..32)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 32.;
                    Pose {
                        angle: a,
                        distance,
                        light: a * 1.7 + 0.3,
                    }
                })
                .collect::<Vec<_>>();
            for adaptive in [false, true] {
                let _ = gpu.batch_selected(&uploaded, &poses, |p| {
                    if adaptive {
                        select::choose(&asset, p, height, 0.15, false).ranges
                    } else {
                        full.clone()
                    }
                })?;
            }
            for round in 0..if smoke { 2 } else { 8 } {
                for (slot, adaptive) in if round % 2 == 0 {
                    [false, true, true, false]
                } else {
                    [true, false, false, true]
                }
                .into_iter()
                .enumerate()
                {
                    let t = gpu.batch_selected(&uploaded, &poses, |p| {
                        if adaptive {
                            select::choose(&asset, p, height, 0.15, false).ranges
                        } else {
                            full.clone()
                        }
                    })?;
                    timings.push(serde_json::json!({"width":width,"height":height,"distance":distance,"round":round,"slot":slot,"adaptive":adaptive,"timing":t}));
                }
            }
            println!("distance {distance}, failures {failed}");
        }
        if !smoke {
            let mut first = None;
            for frame in 0..=24 {
                let phase = if frame == 24 {
                    0.
                } else {
                    frame as f32 * std::f32::consts::TAU / 24.
                };
                let distance = if frame == 24 {
                    2.7
                } else {
                    2.7 * ((64f32 / 2.7).ln() * (phase * 0.5).sin()).exp()
                };
                let pose = Pose {
                    angle: phase,
                    distance,
                    light: phase * 1.7 + 0.3,
                };
                let choice = select::choose(&asset, pose, height, 0.15, false);
                let a = gpu.capture_selected(&uploaded, pose, Some(&full))?;
                let b = gpu.capture_selected(&uploaded, pose, Some(&choice.ranges))?;
                let hashes = (
                    super::compiler::hash(bytemuck::cast_slice(&a)),
                    super::compiler::hash(bytemuck::cast_slice(&b)),
                );
                if frame == 0 {
                    first = Some(hashes);
                }
                if frame == 24 {
                    ensure!(first == Some(hashes), "return pixels changed");
                }
                let q = quality(&a, &b);
                failed += usize::from(!q.pass);
                rows.push(serde_json::json!({"trajectory":true,"width":width,"height":height,"distance":distance,"frame":frame,"return_identical":frame==24,"quality":q,"selection":choice}));
            }
        }
        // Negative control: a deliberately uncertified view must not be confused with accepted selection.
        let pose = Pose {
            angle: 0.7,
            distance: 2.7,
            light: 1.1,
        };
        let forced = select::choose(&asset, pose, height, 0.15, true);
        let a = gpu.capture_selected(&uploaded, pose, Some(&full))?;
        let b = gpu.capture_selected(&uploaded, pose, Some(&forced.ranges))?;
        rows.push(serde_json::json!({"negative_control":true,"width":width,"quality":quality(&a,&b),"selection":forced}));
    }
    let r = serde_json::json!({"source_triangles":asset.source_count/3,"file_bytes":bytes.len(),"gpu_geometry_bytes":asset.mesh.vertices.len()*24+asset.mesh.faces.len()*12,"quality":rows,"timings":timings,"failed":failed});
    std::fs::write(
        dir.join(if smoke { "smoke.json" } else { "results.json" }),
        serde_json::to_vec_pretty(&r)?,
    )?;
    ensure!(failed == 0, "{failed} image gates failed");
    Ok(())
}
