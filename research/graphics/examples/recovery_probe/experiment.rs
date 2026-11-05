use super::{
    certificates, compiler,
    render::{Bound, Gpu, Pose},
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
pub fn run(old_dir: &Path, dir: &Path) -> Result<()> {
    let prefix = if std::env::var_os("RECOVERY_BORDERS").is_some() {
        "stable"
    } else {
        "source"
    };
    let hierarchical = std::env::var_os("RECOVERY_CERTIFICATES").is_some();
    let adaptive = std::env::var_os("RECOVERY_FALLBACK").is_some();
    ensure!(
        !hierarchical || (adaptive && prefix == "stable"),
        "hierarchy requires RECOVERY_FALLBACK and RECOVERY_BORDERS controls"
    );
    let canonical =
        compiler::decode(&std::fs::read(old_dir.join(format!("{prefix}-1.igc")))?)?.source;
    let mut rows = vec![];
    let mut timings = vec![];
    let mut adapters = vec![];
    for factor in [1, 2, 10, 100] {
        let old = compiler::decode(&std::fs::read(
            old_dir.join(format!("{prefix}-{factor}.igc")),
        )?)?;
        let package = std::fs::read(
            if hierarchical { old_dir } else { dir }.join(format!("stable-{factor}.igc")),
        )?;
        let new = compiler::decode(&package)?;
        let hierarchy = if hierarchical {
            Some(certificates::Hierarchy::load(
                &std::fs::read(dir.join(format!("certificates-{factor}.ich")))?,
                &new,
                compiler::hash(&package),
            )?)
        } else {
            None
        };
        ensure!(
            bytemuck::cast_slice::<_, u8>(&old.source.vertices)
                == bytemuck::cast_slice::<_, u8>(&new.source.vertices)
                && old.source.faces == new.source.faces,
            "source changed"
        );
        let bound = Bound::new(&new);
        let old_bound = Bound::new(&old);
        for (width, height) in [(1728, 1117), (3456, 2234)] {
            let gpu = pollster::block_on(Gpu::new(width, height))?;
            if !adapters.contains(&gpu.adapter) {
                adapters.push(gpu.adapter.clone());
            }
            let base = gpu.upload(&canonical);
            let src = gpu.upload(&old.source);
            let prior = gpu.upload(&old.proxy);
            let stable = gpu.upload(&new.proxy);
            let full = vec![(0, old.source.faces.len() as u32 * 3)];
            let old_range = vec![(0, old.proxy.faces.len() as u32 * 3)];
            let new_range = vec![(0, new.proxy.faces.len() as u32 * 3)];
            let base_range = vec![(0, canonical.faces.len() as u32 * 3)];
            for (di, distance) in [2.7, 64.].into_iter().enumerate() {
                let mut first = None;
                for frame in 0..9 {
                    let phase = (frame % 8) as f32 * std::f32::consts::TAU / 8.;
                    let pose = Pose {
                        angle: phase,
                        distance,
                        light: phase * 1.7 + 0.3,
                    };
                    ensure!(
                        adaptive
                            || (bound.pixels(pose, height) <= 0.15
                                && bound.normal_error <= 0.000101),
                        "candidate outside quality domain"
                    );
                    let a = gpu.capture_selected(&src, pose, Some(&full))?;
                    let old_fallback = adaptive && !old_bound.choose(pose, height);
                    let decision = hierarchy.as_ref().map(|h| h.decide(pose, height));
                    let fallback = adaptive
                        && !decision.map_or_else(|| bound.choose(pose, height), |d| d.proxy);
                    let b = if old_fallback {
                        a.clone()
                    } else {
                        gpu.capture_selected(&prior, pose, Some(&old_range))?
                    };
                    let c = if fallback {
                        a.clone()
                    } else {
                        gpu.capture_selected(&stable, pose, Some(&new_range))?
                    };
                    let d = gpu.capture_selected(&base, pose, Some(&base_range))?;
                    let hashes = [&a, &b, &c, &d].map(|p| compiler::hash(bytemuck::cast_slice(p)));
                    if frame == 0 {
                        first = Some(hashes);
                    }
                    if frame == 8 {
                        ensure!(first == Some(hashes), "return mismatch");
                    }
                    rows.push(serde_json::json!({"factor":factor,"width":width,"height":height,"distance":distance,"frame":frame,"source_triangles":old.source.faces.len(),"old_triangles":old.proxy.faces.len(),"triangles":new.proxy.faces.len(),"hierarchy":decision,"projected_error":if fallback {0.} else {decision.map_or_else(||bound.pixels(pose,height),|d|d.accepted_error_px)},"candidate_projected_error":bound.pixels(pose,height),"old_fallback":old_fallback,"fallback":fallback,"drawn_triangles":if fallback {old.source.faces.len()} else {new.proxy.faces.len()},"normal_error":bound.normal_error,"density":quality(&d,&a),"old_source":quality(&a,&b),"stable_source":quality(&a,&c),"old_base":quality(&d,&b),"stable_base":quality(&d,&c)}));
                    if hierarchical || frame % 2 == 0 {
                        for (name, pixels) in
                            [("source", &a), ("old", &b), ("stable", &c), ("base", &d)]
                        {
                            png(
                                &dir.join(format!("{factor}-{width}-{di}-{frame}-{name}.png")),
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
                ensure!(
                    adaptive || poses.iter().all(|p| bound.pixels(*p, height) <= 0.15),
                    "benchmark outside quality domain"
                );
                if std::env::var_os("RECOVERY_QUALITY_ONLY").is_none() {
                    for round in 0..8 {
                        for slot in 0..8 {
                            let mode = if round % 2 == 0 {
                                [0, 1, 2, 3, 3, 2, 1, 0][slot]
                            } else {
                                [3, 2, 1, 0, 0, 1, 2, 3][slot]
                            };
                            let (buffer, ranges) = match mode {
                                0 => (&src, &full),
                                1 => (&prior, &old_range),
                                2 => (&stable, &new_range),
                                _ => (&base, &base_range),
                            };
                            let choose = |p| {
                                if mode == 1 || mode == 2 {
                                    let b = if mode == 1 { &old_bound } else { &bound };
                                    assert!(
                                        b.pixels(p, height) <= 0.15 && b.normal_error <= 0.000101,
                                        "benchmark outside certified domain"
                                    );
                                }
                                ranges.clone()
                            };
                            let timing = if adaptive {
                                let b = match mode {
                                    1 => Some(&old_bound),
                                    2 => Some(&bound),
                                    _ => None,
                                };
                                let full_buffer = if mode == 3 { &base } else { &src };
                                let select = |p| {
                                    if mode == 2
                                        && let Some(h) = &hierarchy
                                    {
                                        h.decide(p, height).proxy
                                    } else {
                                        b.is_some_and(|b| b.choose(p, height))
                                    }
                                };
                                let _ = gpu.batch_decision(full_buffer, buffer, &poses, select)?;
                                gpu.batch_decision(full_buffer, buffer, &poses, select)?
                            } else {
                                let _ = gpu.batch_selected(buffer, &poses, choose)?;
                                gpu.batch_selected(buffer, &poses, choose)?
                            };
                            timings.push(serde_json::json!({"factor":factor,"width":width,"height":height,"distance":distance,"round":round,"slot":slot,"mode":(["source","old","stable","base"][mode]),"timing":timing}));
                        }
                    }
                }
                println!("factor{factor} {width} d{distance} complete");
            }
        }
    }
    std::fs::write(
        dir.join(if std::env::var_os("RECOVERY_QUALITY_ONLY").is_some() {
            "quality-only.json"
        } else {
            "results.json"
        }),
        serde_json::to_vec_pretty(
            &serde_json::json!({"hierarchical_certificates":hierarchical,"adaptive_fallback":adaptive,"adapters":adapters,"quality":rows,"timings":timings}),
        )?,
    )?;
    Ok(())
}
