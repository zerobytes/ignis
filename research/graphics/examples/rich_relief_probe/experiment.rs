use super::{
    certificates, compile, compiler, material,
    render::{Gpu, Pose},
    select,
};
use anyhow::{Result, ensure};
use glam::Vec3;
use serde::Serialize;
use std::path::Path;
#[derive(Serialize)]
pub(super) struct Quality {
    rmse: f64,
    p99: f32,
    max: f32,
    coverage_error: f64,
    covered: usize,
    pass: bool,
}
pub(super) fn quality(a: &[[f32; 4]], b: &[[f32; 4]]) -> Quality {
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
pub(super) fn png(path: &Path, pixels: &[[f32; 4]], width: u32, height: u32) -> Result<()> {
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
/// Heightfield diagnostic: exact shared chart boundaries, projected displacement
/// gate, no cone veto. Keep the cone-veto path as a separate measured control.
fn projected_shape(low: [f32; 3], high: [f32; 3], eye: Vec3, forward: Vec3) -> (f32, f32) {
    let mut depth = f32::INFINITY;
    let mut radius = 0f32;
    for bits in 0..8 {
        let v = Vec3::new(
            if bits & 1 == 0 { low[0] } else { high[0] },
            if bits & 2 == 0 { low[1] } else { high[1] },
            if bits & 4 == 0 { low[2] } else { high[2] },
        ) - eye;
        let z = v.dot(forward);
        depth = depth.min(z);
        radius = radius.max((v - forward * z).length());
    }
    (depth, radius)
}
pub(super) fn regional(
    asset: &compile::Asset,
    p: Pose,
    height: u32,
    budget: f32,
) -> select::Selection {
    let eye = p.eye();
    let forward = -eye.normalize();
    let focal = height as f32 / (2. * (std::f32::consts::FRAC_PI_4 / 2.).tan());
    let mut choices = Vec::with_capacity(asset.patches.len());
    let mut max_error_px = 0f32;
    let mut reduced = 0;
    for patch in &asset.patches {
        let mut best = 0;
        let mut e = 0.;
        let (depth, radius) = projected_shape(patch.low, patch.high, eye, forward);
        for (i, l) in patch.levels.iter().enumerate().skip(1) {
            let px = if depth <= l.error + 0.1 {
                f32::INFINITY
            } else {
                focal * l.error * (1. + radius / depth) / (depth - l.error)
            };
            if px <= budget && l.count < patch.levels[best].count {
                best = i;
                e = px;
            }
        }
        choices.push(best);
        max_error_px = max_error_px.max(e);
        reduced += usize::from(best > 0);
    }
    let mut ranges: Vec<(u32, u32)> = vec![];
    let mut triangles = 0;
    for level in 0..5 {
        for (patch, &choice) in asset.patches.iter().zip(&choices) {
            if choice != level {
                continue;
            }
            let l = &patch.levels[choice];
            triangles += l.count / 3;
            if let Some(last) = ranges.last_mut()
                && last.0 + last.1 == l.start
            {
                last.1 += l.count;
            } else {
                ranges.push((l.start, l.count));
            }
        }
    }
    select::Selection {
        ranges,
        triangles,
        reduced,
        silhouette_native: 0,
        max_error_px,
    }
}
pub(super) fn pose(case: usize, frame: usize, steps: usize) -> Pose {
    let t = (frame % steps) as f32 / steps as f32;
    let phase = t * std::f32::consts::TAU;
    let distance = if case == 4 {
        2.7 * (64f32 / 2.7).powf(1. - (2. * t - 1.).abs())
    } else {
        [2.7, 6., 16., 64.][case]
    };
    Pose {
        angle: phase,
        distance,
        light: phase * 1.7 + 0.3,
    }
}
pub fn run(dir: &Path, smoke: bool) -> Result<()> {
    run_kind(dir, smoke, false, false)
}
pub fn run_refined(dir: &Path, smoke: bool) -> Result<()> {
    run_kind(dir, smoke, true, false)
}
pub fn run_auto(dir: &Path) -> Result<()> {
    run_kind(dir, false, true, true)?;
    audit_auto(dir)
}
pub fn audit_auto(dir: &Path) -> Result<()> {
    let data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("auto-results.json"))?)?;
    ensure!(
        data["automatic"] == true && data["reference_stable"] == true,
        "invalid automatic reference"
    );
    let rows = data["quality"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing quality"))?;
    let returns = data["returns"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing returns"))?;
    ensure!(
        rows.len() == 138 && returns.len() == 50,
        "incomplete automatic experiment"
    );
    ensure!(
        rows.iter().all(|r| r["conservative"]["pass"] == true)
            && returns
                .iter()
                .filter(|r| r["mode"] == "conservative")
                .count()
                == 10
            && returns
                .iter()
                .filter(|r| r["mode"] == "conservative")
                .all(|r| r["pass"] == true),
        "automatic quality/return gate failed; diagnostic data retained, experiment not approved"
    );
    Ok(())
}
fn run_kind(dir: &Path, smoke: bool, refined: bool, automatic: bool) -> Result<()> {
    super::build::verify_inputs(dir)?;
    let prefix = if automatic {
        "auto-"
    } else if refined {
        "refined-"
    } else {
        ""
    };
    let mut asset = compile::decode(&std::fs::read(dir.join(if refined {
        "refined.igl"
    } else {
        "regional.igl"
    }))?)?;
    let uniform = if refined && !automatic {
        Some(compile::decode(&std::fs::read(dir.join("regional.igl"))?)?)
    } else {
        None
    };
    let uniform_start = asset.mesh.faces.len() as u32 * 3;
    if let Some(uniform) = &uniform {
        ensure!(
            bytemuck::cast_slice::<_, u8>(&asset.mesh.vertices)
                == bytemuck::cast_slice::<_, u8>(&uniform.mesh.vertices),
            "refined source vertices"
        );
        ensure!(
            asset.source_count == uniform.source_count
                && asset.mesh.faces[..asset.source_count as usize / 3]
                    == uniform.mesh.faces[..uniform.source_count as usize / 3],
            "refined source faces"
        );
        let offset = asset.mesh.vertices.len() as u32;
        asset
            .mesh
            .vertices
            .extend_from_slice(&uniform.mesh.vertices);
        asset.mesh.faces.extend(
            uniform
                .mesh
                .faces
                .iter()
                .map(|f| [f[0] + offset, f[1] + offset, f[2] + offset, f[3]]),
        );
    }
    let package = std::fs::read(dir.join("previous.igc"))?;
    let prior = compiler::decode(&package)?;
    let hierarchy = certificates::Hierarchy::load(
        &std::fs::read(dir.join("previous.ich"))?,
        &prior,
        compiler::hash(&package),
    )?;
    let old_start = asset.mesh.faces.len() as u32 * 3;
    let offset = asset.mesh.vertices.len() as u32;
    asset.mesh.vertices.extend_from_slice(&prior.proxy.vertices);
    asset.mesh.faces.extend(
        prior
            .proxy
            .faces
            .iter()
            .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset, 0]),
    );
    let old_count = prior.proxy.faces.len() as u32 * 3;
    let uniform_selection = |p, height| {
        if let Some(uniform) = &uniform {
            let mut s = regional(uniform, p, height, 0.05);
            for (start, _) in &mut s.ranges {
                *start += uniform_start;
            }
            s
        } else {
            select::choose_with_normal(&asset, p, height, 0.15, false, 2.)
        }
    };
    let frontier = super::cost::Frontier::new(&asset);
    let full = vec![(0, asset.source_count)];
    let color = material::decode(&std::fs::read(dir.join("color.irm"))?)?;
    let normal = material::decode(&std::fs::read(dir.join("normal.irm"))?)?;
    let mut rows = vec![];
    let mut timings = vec![];
    let mut returns = vec![];
    let mut adapters = vec![];
    let mut calibrations = vec![];
    let modes = [
        "source",
        "previous",
        "conservative",
        "regional",
        "exploratory",
    ];
    for (width, height) in if smoke {
        vec![(864, 558)]
    } else {
        vec![(1728, 1117), (3456, 2234)]
    } {
        let mut gpu = pollster::block_on(Gpu::new(width, height))?;
        gpu.install_rich_material(color.size, &color.levels, normal.size, &normal.levels);
        adapters.push(gpu.adapter.clone());
        let uploaded = gpu.upload_rich(&asset.mesh);
        let previous_ranges = |p| {
            if hierarchy.decide(p, height).proxy {
                vec![(old_start, old_count)]
            } else {
                full.clone()
            }
        };
        let model = if automatic {
            let started = std::time::Instant::now();
            let mut samples = vec![];
            let mut measurements = vec![];
            // Held-out from validation distances/angles; coefficients only survive fit.
            for (index, distance) in [3.3, 4.7, 9., 24., 43., 80.].into_iter().enumerate() {
                let p = Pose {
                    angle: 0.37 + index as f32 * 0.71,
                    distance,
                    light: 0.83 + index as f32 * 0.43,
                };
                let previous = previous_ranges(p);
                let local = regional(&asset, p, height, 0.05);
                let mut times = [vec![], vec![]];
                for round in 0..4 {
                    for mode in if round % 2 == 0 {
                        [0, 1, 1, 0]
                    } else {
                        [1, 0, 0, 1]
                    } {
                        let choose = |p| {
                            let previous = previous_ranges(p);
                            if mode == 0 {
                                previous
                            } else {
                                regional(&asset, p, height, 0.05).ranges
                            }
                        };
                        let _ = gpu.batch_selected(&uploaded, &[p], choose)?;
                        let t = gpu.batch_selected(&uploaded, &[p], choose)?;
                        times[mode].push(t.total_batch_ms_per_frame);
                        measurements.push(serde_json::json!({"index":index,"distance":distance,"angle":p.angle,"round":round,"local":mode==1,"timing":t}));
                    }
                }
                let noise = times
                    .iter()
                    .map(|ts| {
                        median(
                            &ts.chunks_exact(2)
                                .map(|p| (p[0] - p[1]).abs())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .sum();
                samples.push(super::cost::Sample {
                    saved_million: (previous.iter().map(|r| r.1 / 3).sum::<u32>() as f64
                        - local.triangles as f64)
                        / 1e6,
                    extra_draws: local.ranges.len().saturating_sub(1) as f64,
                    gain_ms: median(&times[0]) - median(&times[1]),
                    noise_ms: noise,
                });
            }
            let model = super::cost::Model::fit(&samples)?;
            calibrations.push(serde_json::json!({"width":width,"model":model,"samples":samples,"measurements":measurements,"elapsed_ms":started.elapsed().as_secs_f64()*1000.}));
            println!("calibration {width}: {}", serde_json::to_string(&model)?);
            Some(model)
        } else {
            None
        };
        let auto_selection = |p| {
            let previous = previous_ranges(p);
            let count = previous.iter().map(|r| r.1 / 3).sum();
            let model = model.as_ref().unwrap();
            if !model.choose(count, frontier.minimum(p.eye().length(), height, 0.05), 1) {
                return (previous, false);
            }
            let local = regional(&asset, p, height, 0.05);
            let chosen = model.choose(count, local.triangles, local.ranges.len());
            (if chosen { local.ranges } else { previous }, chosen)
        };
        // Exclude initial material/pipeline warm-up; reference repeatability is required.
        let p = pose(0, 0, 8);
        let _ = gpu.capture_selected(&uploaded, p, Some(&full))?;
        let a = gpu.capture_selected(&uploaded, p, Some(&full))?;
        let b = gpu.capture_selected(&uploaded, p, Some(&full))?;
        ensure!(a == b, "warmed reference unstable");
        drop((a, b));
        for case in 0..5 {
            let steps = if case == 4 { 32 } else { 8 };
            let mut first = None;
            for frame in 0..if smoke { 2 } else { steps + 1 } {
                let p = pose(case, frame, steps);
                let conservative = uniform_selection(p, height);
                let selection = regional(&asset, p, height, 0.05);
                let exploratory = regional(&asset, p, height, 0.15);
                let decision = hierarchy.decide(p, height);
                let old = if decision.proxy {
                    vec![(old_start, old_count)]
                } else {
                    full.clone()
                };
                if let Some(model) = &model {
                    let pruned = auto_selection(p).1;
                    ensure!(
                        pruned
                            == model.choose(
                                old.iter().map(|r| r.1 / 3).sum(),
                                selection.triangles,
                                selection.ranges.len()
                            ),
                        "pruning changed choice"
                    );
                }
                let a = gpu.capture_selected(&uploaded, p, Some(&full))?;
                let b = gpu.capture_selected(&uploaded, p, Some(&old))?;
                let auto = if automatic {
                    Some(auto_selection(p))
                } else {
                    None
                };
                let c = gpu.capture_selected(
                    &uploaded,
                    p,
                    Some(
                        auto.as_ref()
                            .map(|s| s.0.as_slice())
                            .unwrap_or(&conservative.ranges),
                    ),
                )?;
                let d = gpu.capture_selected(&uploaded, p, Some(&selection.ranges))?;
                let e = gpu.capture_selected(&uploaded, p, Some(&exploratory.ranges))?;
                let hashes = [&a, &b, &c, &d, &e].map(|p| compiler::hash(bytemuck::cast_slice(p)));
                if frame == 0 {
                    first = Some(hashes);
                }
                rows.push(serde_json::json!({"width":width,"height":height,"case":case,"distance":p.distance,"frame":frame,"previous":quality(&a,&b),"conservative":quality(&a,&c),"regional":quality(&a,&d),"exploratory":quality(&a,&e),"exploratory_selection":exploratory,"selection":selection,"auto_local":auto.as_ref().map(|s|s.1),"conservative_triangles":auto.as_ref().map(|s|s.0.iter().map(|r|r.1/3).sum::<u32>()).unwrap_or(conservative.triangles),"old_triangles":if decision.proxy{old_count/3}else{asset.source_count/3},"source_triangles":asset.source_count/3,"old_decision":decision}));
                for (name, pixels) in modes.into_iter().zip([&a, &b, &c, &d, &e]) {
                    png(
                        &dir.join(format!("{prefix}{width}-{case}-{frame}-{name}.png")),
                        pixels,
                        width,
                        height,
                    )?;
                }
                if frame == steps {
                    let expected = first.unwrap();
                    if !refined {
                        ensure!(
                            expected[0] == hashes[0],
                            "full reference orbit return {width} case{case}"
                        );
                    }
                    for (i, mode) in modes.into_iter().enumerate() {
                        returns.push(serde_json::json!({"width":width,"case":case,"mode":mode,"pass":expected[i]==hashes[i],"first_hash":expected[i].to_string(),"return_hash":hashes[i].to_string()}));
                    }
                }
            }
            let poses = (0..32).map(|i| pose(case, i, 32)).collect::<Vec<_>>();
            for round in 0..if smoke { 1 } else { 8 } {
                for slot in 0..10 {
                    let mode = if round % 2 == 0 {
                        [0, 1, 2, 3, 4, 4, 3, 2, 1, 0][slot]
                    } else {
                        [4, 3, 2, 1, 0, 0, 1, 2, 3, 4][slot]
                    };
                    let choose = |p| match mode {
                        0 => full.clone(),
                        1 => {
                            if hierarchy.decide(p, height).proxy {
                                vec![(old_start, old_count)]
                            } else {
                                full.clone()
                            }
                        }
                        2 => {
                            if automatic {
                                auto_selection(p).0
                            } else {
                                uniform_selection(p, height).ranges
                            }
                        }
                        3 => regional(&asset, p, height, 0.05).ranges,
                        _ => regional(&asset, p, height, 0.15).ranges,
                    };
                    let _ = gpu.batch_selected(&uploaded, &poses, choose)?;
                    let timing = gpu.batch_selected(&uploaded, &poses, choose)?;
                    timings.push(serde_json::json!({"width":width,"height":height,"case":case,"round":round,"slot":slot,"mode":modes[mode],"timing":timing}));
                }
            }
            println!("{width} case{case} complete");
        }
    }
    let reference_stable = returns
        .iter()
        .all(|r: &serde_json::Value| r["mode"] != "source" || r["pass"] == true);
    std::fs::write(
        dir.join(format!(
            "{prefix}{}",
            if smoke { "smoke.json" } else { "results.json" }
        )),
        serde_json::to_vec_pretty(
            &serde_json::json!({"automatic":automatic,"calibrations":calibrations,"auto_geometry_bytes":(asset.mesh.vertices.len()*12+asset.mesh.faces.len()*12),"refined":refined,"sampling":"implicit-original","reference_stable":reference_stable,"conservative_mode":if automatic {"calibrated-automatic"} else if refined {"uniform-previous-0.05px"} else {"silhouette-veto"},"adapters":adapters,"quality":rows,"timings":timings,"returns":returns}),
        )?,
    )?;
    ensure!(
        reference_stable,
        "reference repeatability failed; diagnostic data retained, experiment not approved"
    );
    Ok(())
}
fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    (sorted[(n - 1) / 2] + sorted[n / 2]) / 2.
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_projection_matches_existing_bound_bitwise() {
        for i in 0..32 {
            let p = Pose {
                angle: i as f32 * 0.2,
                distance: 2.7,
                light: 0.,
            };
            let eye = p.eye();
            let forward = -eye.normalize();
            for h in [1117, 2234] {
                for e in [0., 0.0001, 0.01] {
                    let b = super::super::render::Bound {
                        low: Vec3::splat(-1.),
                        high: Vec3::splat(1.),
                        error: e,
                        normal_error: 0.,
                        reduces: true,
                    };
                    let (depth, radius) =
                        projected_shape(b.low.to_array(), b.high.to_array(), eye, forward);
                    let focal = h as f32 / (2. * (std::f32::consts::FRAC_PI_4 / 2.).tan());
                    assert_eq!(
                        b.pixels(p, h),
                        focal * e * (1. + radius / depth) / (depth - e)
                    );
                }
            }
        }
    }
    #[test]
    fn selector_preserves_native_at_extreme_close_and_reduces_far() -> Result<()> {
        let src = super::super::build::source(32);
        let a = super::super::build::regional(&src, 32, 16)?;
        let frontier = super::super::cost::Frontier::new(&a);
        for h in [558, 1117, 2234] {
            for d in [0.01, 0.5, 2.7, 6., 16., 64., 10000.] {
                for angle in 0..16 {
                    let p = Pose {
                        angle: angle as f32 * 0.39,
                        distance: d,
                        light: 0.,
                    };
                    assert!(
                        frontier.minimum(p.eye().length(), h, 0.05)
                            <= regional(&a, p, h, 0.05).triangles
                    );
                }
            }
        }
        let near = regional(
            &a,
            Pose {
                angle: 0.,
                distance: 0.01,
                light: 0.,
            },
            2234,
            0.05,
        );
        assert_eq!(near.triangles, a.source_count / 3);
        let far = regional(
            &a,
            Pose {
                angle: 0.,
                distance: 10000.,
                light: 0.,
            },
            2234,
            0.05,
        );
        assert!(far.triangles < near.triangles);
        assert!(far.max_error_px <= 0.05);
        Ok(())
    }
}
