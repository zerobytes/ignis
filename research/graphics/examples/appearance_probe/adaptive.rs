//! Bidirectional camera experiment. Selection is evaluated at final output resolution.
use super::{
    SIZE, geometry,
    gpu::{Case, Gpu},
    metrics, reference,
    selection::{self, Selector},
};
use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path, time::Instant};

pub fn run() -> Result<()> {
    let output = Path::new("artifacts/appearance-probe/adaptive");
    fs::create_dir_all(output)?;
    let start = Instant::now();
    let mut gpu = pollster::block_on(Gpu::new())?;
    let mesh = geometry::generate(128);
    let uploaded = gpu.upload(&mesh)?;
    let mut rows = Vec::new();
    // Same poses in opposite directions, retaining selection history through the turn.
    let distances: Vec<f32> = (0..=24)
        .chain((0..24).rev())
        .map(|i| 2.5 + 13.5 * i as f32 / 24.0)
        .collect();
    for (name, tilt, exponent, specular) in [
        ("diffuse", 35.0, 8.0, 0.0),
        ("glossy", 35.0, 96.0, 2.0),
        ("grazing", 75.0, 96.0, 2.0),
    ] {
        let mut selector = Selector::default();
        let mut previous = None;
        for (frame, &distance) in distances.iter().enumerate() {
            let case = Case {
                distance,
                tilt,
                light_angle: 25.0,
                exponent,
                specular,
                shift: 0.0,
            };
            let before = selector.detailed;
            let select_start = Instant::now();
            let pixels = selector.update(&mesh.bounds, case, SIZE);
            let selection_us = select_start.elapsed().as_secs_f64() * 1e6;
            gpu.detailed = selector.detailed;
            let changed = before
                .iter()
                .zip(selector.detailed)
                .filter(|(a, b)| **a != *b)
                .count();
            let detail_count = selector.detailed.iter().filter(|v| **v).count();
            let triangles = detail_count * (mesh.vertices.len() / 3 / geometry::TILE_COUNT)
                + (geometry::TILE_COUNT - detail_count) * 2;
            let draws = selection::ranges(&selector.detailed, true).count()
                + selection::ranges(&selector.detailed, false).count();
            let (truth, convergence) = reference(&mut gpu, &uploaded, case)?;
            let full = gpu.render(&uploaded, case, 0, SIZE, 4, false)?.pixels;
            let adaptive = gpu.render(&uploaded, case, 4, SIZE, 4, false)?.pixels;
            let summary = gpu.render(&uploaded, case, 2, SIZE, 4, false)?.pixels;
            let mask = gpu.render(&uploaded, case, 5, SIZE, 4, false)?.pixels;
            // The same camera and filter, only the previous selection retained: isolates a switch.
            gpu.detailed = before;
            let switch_rmse = if changed > 0 {
                let old = gpu.render(&uploaded, case, 4, SIZE, 4, false)?.pixels;
                metrics::compare(&old, &adaptive).rmse_surface
            } else {
                0.0
            };
            gpu.detailed = selector.detailed;
            if detail_count == geometry::TILE_COUNT {
                anyhow::ensure!(full == adaptive, "all-detail render differs from full");
            }
            if detail_count == 0 {
                anyhow::ensure!(
                    summary == adaptive,
                    "all-summary render differs from summary"
                );
            }
            if frame == 0 || frame == distances.len() - 1 {
                anyhow::ensure!(
                    detail_count == geometry::TILE_COUNT,
                    "near camera lost geometry"
                );
            }
            if frame == 24 {
                anyhow::ensure!(detail_count == 0, "far camera not summarized");
            }
            let native = gpu.render(&uploaded, case, 4, SIZE, 1, true)?;
            let native_full = gpu.render(&uploaded, case, 0, SIZE, 1, true)?;
            let temporal =
                previous
                    .as_ref()
                    .map_or(0.0, |(r, c): &(Vec<[f32; 4]>, Vec<[f32; 4]>)| {
                        metrics::temporal(r, c, &truth, &adaptive)
                    });
            let id = format!("{name}_{frame:02}");
            for (label, image) in [
                ("reference", &truth),
                ("full", &full),
                ("adaptive", &adaptive),
                ("summary", &summary),
                ("selection", &mask),
            ] {
                metrics::png(&output.join(format!("{id}_{label}.png")), image, SIZE)?;
            }
            metrics::error_png(
                &output.join(format!("{id}_error.png")),
                &truth,
                &adaptive,
                SIZE,
            )?;
            rows.push(json!({"id":id,"case":name,"frame":frame,"direction":if frame <= 24 {"afastando"} else {"aproximando"},"parameters":case,
                "detail_regions":detail_count,"summary_regions":geometry::TILE_COUNT-detail_count,"triangles":triangles,"draw_calls":draws,
                "changed_regions":changed,"detailed":selector.detailed.as_slice(),"footprint_px":pixels.as_slice(),"selection_cpu_us":selection_us,
                "adaptive_error":metrics::compare(&truth,&adaptive),"full_filtered_error":metrics::compare(&truth,&full),"summary_error":metrics::compare(&truth,&summary),
                "reference_half_vs_full":convergence,"switch_same_pose_rmse":switch_rmse,"temporal_residual_rmse":temporal,
                "native_timing":native.timing,"native_full_timing":native_full.timing,
                "native_error":metrics::compare(&truth,&native.pixels),"native_full_error":metrics::compare(&truth,&native_full.pixels)}));
            previous = Some((truth, adaptive));
        }
        println!(
            "{name}: {} frames, near/far/reversal equality checks passed",
            distances.len()
        );
    }
    // An adversarial oscillation around a single threshold. Hysteresis must suppress chatter.
    let mut stable = Selector::default();
    let mut hard = [true; geometry::TILE_COUNT];
    let mut jitter_rows = Vec::new();
    let target = mesh.bounds[0];
    let camera = |distance| Case {
        distance,
        tilt: 35.0,
        light_angle: 25.0,
        exponent: 96.0,
        specular: 2.0,
        shift: 0.0,
    };
    let mut lo = 2.5;
    let mut hi = 16.0;
    for _ in 0..32 {
        let mid = (lo + hi) * 0.5;
        if selection::footprint(target, camera(mid), SIZE) > 0.875 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mut stable_changes = 0;
    let mut hard_changes = 0;
    for frame in 0..40 {
        let case = camera((lo + hi) * 0.5 + if frame % 2 == 0 { -0.03 } else { 0.03 });
        let before = stable.detailed;
        let pixels = stable.update(&mesh.bounds, case, SIZE);
        let next_hard = pixels.map(|px| px >= 0.875);
        if frame > 0 {
            stable_changes += before
                .iter()
                .zip(stable.detailed)
                .filter(|(a, b)| **a != *b)
                .count();
            hard_changes += hard
                .iter()
                .zip(next_hard)
                .filter(|(a, b)| **a != *b)
                .count();
        }
        hard = next_hard;
        jitter_rows.push(json!({"frame":frame,"distance":case.distance,"detailed":stable.detailed.as_slice(),"hard_detailed":hard.as_slice(),"footprint_px":pixels.as_slice()}));
    }
    anyhow::ensure!(
        stable_changes == 0 && hard_changes > 0,
        "jitter control failed"
    );
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":1,"adapter":gpu.adapter,"image_size":SIZE,"cells_per_side":128,
        "full_triangles":mesh.vertices.len()/3,"summary_triangles":mesh.proxy.len()/3,"enter_summary_px":selection::ENTER_SUMMARY,"return_geometry_px":selection::RETURN_GEOMETRY,
        "reference_samples_per_pixel":2048,"candidate_samples_per_pixel":16,"benchmark_samples_per_pixel":1,
        "bound":"Perspective Jacobian norm over each region AABB times maximum microcell bounding-box diameter. Geometric bound only; no radiance/visibility guarantee.",
        "selection":"64 regions, persistent hysteresis, discrete switch; evaluated at output resolution independently of supersampling. Near-plane intersection forces detail.",
        "jitter":{"frames":jitter_rows,"hysteresis_changes_after_initialization":stable_changes,"hard_threshold_changes_after_initialization":hard_changes},
        "cases":rows,"total_seconds":start.elapsed().as_secs_f64()}),
        )?,
    )?;
    println!(
        "{} in {:.1}s",
        output.join("report.json").display(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
