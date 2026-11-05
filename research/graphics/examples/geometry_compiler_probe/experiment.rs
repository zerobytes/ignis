use super::{
    compiler,
    render::{Bound, Gpu, Pose},
};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::{path::Path, time::Instant};
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
    let mut errors = vec![];
    let mut sum = 0.;
    let mut covered = 0;
    let mut mismatch = 0;
    for (a, b) in a.iter().zip(b) {
        let ca = a[3] > 0.5;
        let cb = b[3] > 0.5;
        if ca || cb {
            covered += 1;
            mismatch += usize::from(ca != cb);
            for i in 0..3 {
                let e = (a[i] - b[i]).abs();
                errors.push(e);
                sum += (e as f64).powi(2);
            }
        }
    }
    errors.sort_unstable_by(f32::total_cmp);
    let rmse = (sum / (covered * 3).max(1) as f64).sqrt();
    let p99 = errors.get(errors.len() * 99 / 100).copied().unwrap_or(0.);
    let max = errors.last().copied().unwrap_or(0.);
    let coverage_error = mismatch as f64 / covered.max(1) as f64;
    Quality {
        rmse,
        p99,
        max,
        coverage_error,
        covered,
        pass: rmse <= 0.005 && p99 <= 0.02 && coverage_error <= 0.0005,
    }
}
fn png(path: &Path, pixels: &[[f32; 4]], width: u32, height: u32) -> Result<()> {
    let mut data = Vec::with_capacity(pixels.len() * 3);
    for p in pixels {
        for &v in &p[..3] {
            let s = if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.max(0.).powf(1. / 2.4) - 0.055
            };
            data.push((s.clamp(0., 1.) * 255.).round() as u8);
        }
    }
    let mut enc = png::Encoder::new(std::fs::File::create(path)?, width, height);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(&data)?;
    Ok(())
}
pub fn run(dir: &Path, smoke: bool, timings_only: bool) -> Result<()> {
    let resolutions = if smoke {
        vec![(864, 558)]
    } else {
        vec![(1728, 1117), (3456, 2234)]
    };
    let factors = if smoke {
        vec![100]
    } else {
        vec![1, 2, 10, 100]
    };
    let mut rows = vec![];
    let mut timings = vec![];
    let mut assets = vec![];
    let mut failed = 0;
    for (width, height) in resolutions {
        let gpu = pollster::block_on(Gpu::new(width, height))?;
        println!("adapter {} {width}x{height}", gpu.adapter);
        for name in ["stone", "folded"] {
            for &factor in &factors {
                let file = dir.join(format!("{name}-{factor}.igc"));
                let start = Instant::now();
                let bytes = std::fs::read(&file)?;
                let compiled = compiler::decode(&bytes)?;
                let decode_ms = start.elapsed().as_secs_f64() * 1000.;
                let bound = Bound::new(&compiled);
                let start = Instant::now();
                let source = gpu.upload(&compiled.source);
                let proxy_buffer = bound.reduces.then(|| gpu.upload(&compiled.proxy));
                let proxy = proxy_buffer.as_ref().unwrap_or(&source);
                let upload_cpu_ms = start.elapsed().as_secs_f64() * 1000.;
                assets.push(serde_json::json!({"name":name,"factor":factor,"width":width,"height":height,"adapter":gpu.adapter,"source_triangles":compiled.source.faces.len(),"proxy_triangles":compiled.proxy.faces.len(),"source_gpu_bytes":source.bytes,"proxy_gpu_bytes":proxy_buffer.as_ref().map_or(0,|p|p.bytes),"certificates_bytes":compiled.regions.len()*48,"file_bytes":bytes.len(),"decode_ms":decode_ms,"upload_cpu_ms":upload_cpu_ms}));
                drop(bytes);
                for (range, distance) in [("near", 3.2), ("far", 10.)] {
                    let mut initial_hash = None;
                    for frame in 0..if timings_only {
                        0
                    } else if smoke {
                        3
                    } else {
                        9
                    } {
                        let phase = (frame % 8) as f32 * std::f32::consts::TAU / 8.;
                        let pose = Pose {
                            angle: phase,
                            distance,
                            light: phase * 1.7 + 0.3,
                        };
                        let selected = bound.choose(pose, height);
                        let a = gpu.capture(&source, pose)?;
                        let b = gpu.capture(if selected { proxy } else { &source }, pose)?;
                        let hashes = (
                            compiler::hash(bytemuck::cast_slice(&a)),
                            compiler::hash(bytemuck::cast_slice(&b)),
                        );
                        if frame == 0 {
                            initial_hash = Some(hashes);
                        }
                        if frame == 8 {
                            ensure!(
                                initial_hash == Some(hashes),
                                "non-deterministic return capture"
                            );
                        }
                        let q = quality(&a, &b);
                        if !q.pass {
                            failed += 1;
                        }
                        rows.push(serde_json::json!({"name":name,"factor":factor,"range":range,"width":width,"height":height,"frame":frame,"selected_proxy":selected,"projected_error_px":bound.pixels(pose,height),"source_triangles_per_covered_pixel":compiled.source.faces.len() as f64/q.covered.max(1) as f64,"quality":q,"return_identical":if frame==8 {Some(initial_hash==Some(hashes))} else {None}}));
                        if factor == 100 && frame % 2 == 0 {
                            for (kind, pixels) in [("source", &a), ("compiled", &b)] {
                                png(
                                    &dir.join(format!("{name}-{range}-{width}-{frame}-{kind}.png")),
                                    pixels,
                                    width,
                                    height,
                                )?;
                            }
                        }
                    }
                    // Warm both paths, then alternate paired duplicate controls in every round.
                    for optional in [false, true] {
                        let _ = gpu.batch(&source, proxy, &bound, optional, distance, 0.)?;
                    }
                    for round in 0..if smoke { 2 } else { 8 } {
                        let order = if round % 2 == 0 {
                            [false, true, true, false]
                        } else {
                            [true, false, false, true]
                        };
                        for (slot, optional) in order.into_iter().enumerate() {
                            let timing = gpu.batch(
                                &source,
                                proxy,
                                &bound,
                                optional,
                                distance,
                                round as f32 * 0.19,
                            )?;
                            timings.push(serde_json::json!({"name":name,"factor":factor,"range":range,"width":width,"height":height,"round":round,"slot":slot,"optional":optional,"timing":timing}));
                        }
                    }
                    println!(
                        "{name} {factor}x {range} {width}: {} -> {}, failures {failed}",
                        compiled.source.faces.len(),
                        compiled.proxy.faces.len()
                    );
                }
                // Invalid projection must select the original geometry, not a cheaper uncertified proxy.
                ensure!(
                    !bound.choose(
                        Pose {
                            angle: 0.4,
                            distance: 0.3,
                            light: 0.
                        },
                        height
                    ),
                    "near-plane fallback"
                );
            }
        }
    }
    let report = serde_json::json!({"protocol":"blind geometry compiler v1; fixed surface; factors are source multipliers, not triangles/pixel","quality":rows,"timings":timings,"assets":assets,"failed":failed});
    std::fs::write(
        dir.join(if timings_only {
            "timing-repeat.json"
        } else if smoke {
            "smoke.json"
        } else {
            "results.json"
        }),
        serde_json::to_vec_pretty(&report)?,
    )?;
    ensure!(failed == 0, "{failed} quality failures");
    Ok(())
}
