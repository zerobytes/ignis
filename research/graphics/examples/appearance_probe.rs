//! Isolated graphics research. No engine scene, gameplay or editor contracts change.
//! cargo run --locked --release --example appearance_probe
#[path = "appearance_probe/adaptive.rs"]
mod adaptive;
#[path = "appearance_probe/geometry.rs"]
mod geometry;
#[path = "appearance_probe/gpu.rs"]
mod gpu;
#[path = "appearance_probe/hierarchy.rs"]
mod hierarchy;
#[path = "appearance_probe/metrics.rs"]
mod metrics;
#[path = "appearance_probe/multiscale.rs"]
mod multiscale;
#[path = "appearance_probe/quality.rs"]
mod quality;
#[path = "appearance_probe/selection.rs"]
mod selection;
#[path = "appearance_probe/summary_cost.rs"]
mod summary_cost;
use anyhow::{Context, Result};
use gpu::{Case, Gpu};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, time::Instant};

const SIZE: u32 = 256;
const METHODS: [&str; 4] = ["full", "flat", "distribution32", "distribution128"];
fn halton(mut n: usize, base: usize) -> f32 {
    let mut f = 1.0_f32;
    let mut value = 0.0;
    while n > 0 {
        f /= base as f32;
        value += f * (n % base) as f32;
        n /= base;
    }
    value
}
fn reference(
    gpu: &mut Gpu,
    mesh: &gpu::Uploaded,
    case: Case,
) -> Result<(Vec<[f32; 4]>, metrics::Error)> {
    let mut sum = vec![[0_f32; 4]; (SIZE * SIZE) as usize];
    let mut half = sum.clone();
    for pass in 0..32 {
        gpu.jitter = [halton(pass + 1, 2) - 0.5, halton(pass + 1, 3) - 0.5];
        let image = gpu.render(mesh, case, 0, SIZE, 8, false)?.pixels;
        for (out, pixel) in sum.iter_mut().zip(image) {
            for c in 0..4 {
                out[c] += pixel[c] / 32.0;
            }
        }
        if pass == 15 {
            for (out, pixel) in half.iter_mut().zip(&sum) {
                for c in 0..4 {
                    out[c] = pixel[c] * 2.0;
                }
            }
        }
    }
    gpu.jitter = [0.0; 2];
    let convergence = metrics::compare(&sum, &half);
    Ok((sum, convergence))
}
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let first = args.next();
    if matches!(
        first.as_deref(),
        Some(
            "--bake-regional-scene"
                | "--regional-scene"
                | "--regional-efficiency"
                | "--bake-gpu-visibility"
                | "--gpu-visibility"
                | "--verify-gpu-visibility"
                | "--benchmark-gpu-visibility"
                | "--bake-triangle-scaling"
                | "--triangle-scaling"
                | "--confirm-triangle-scaling"
                | "--bake-regional-scaling"
                | "--regional-scaling"
                | "--confirm-regional-scaling"
                | "--regional-transitions"
                | "--fixed-tiled-field"
                | "--tune-tiled-field"
                | "--calibrate-tiled-field"
                | "--capture-tiled-field"
                | "--motion-tiled-field"
                | "--depth-tiled-field"
                | "--bench-tiled-field"
                | "--repeat-tiled-field"
                | "--audit-tiled-field"
                | "--smoke-tiled-field"
                | "--tune-hierarchical-field"
                | "--compile-hierarchical-field"
                | "--calibrate-hierarchical-field"
                | "--capture-hierarchical-field"
                | "--motion-hierarchical-field"
                | "--depth-hierarchical-field"
                | "--bench-hierarchical-field"
                | "--repeat-hierarchical-field"
                | "--audit-hierarchical-field"
                | "--smoke-hierarchical-field"
                | "--work-hierarchical-field"
                | "--audit-geometric-field"
                | "--motion-geometric-field"
                | "--depth-geometric-field"
                | "--compile-geometric-field"
                | "--capture-geometric-field"
                | "--calibrate-geometric-field"
                | "--bench-geometric-field"
                | "--repeat-geometric-field"
                | "--smoke-geometric-field"
                | "--compile-combined-symbol"
                | "--calibrate-combined-symbol"
                | "--capture-combined-symbol"
                | "--bench-combined-symbol"
                | "--repeat-combined-symbol"
                | "--smoke-combined-symbol"
                | "--bake-hybrid-scaling"
                | "--hybrid-scaling"
                | "--confirm-hybrid-scaling"
                | "--hybrid-transitions"
                | "--calibrate-cost-scaling"
                | "--validate-cost-scaling"
                | "--capture-cost-scaling"
                | "--inspect-cost-scaling"
                | "--repeat-cost-scaling"
                | "--ablate-scaling"
                | "--validate-ablation"
                | "--capture-ablation"
                | "--order-ablation"
                | "--repeat-ablation"
                | "--extend-domain-automatic-shading"
                | "--calibrate-automatic-shading"
                | "--validate-automatic-shading"
                | "--repeat-automatic-shading"
                | "--capture-automatic-shading"
                | "--isolate-near-plateau"
                | "--profile-near-plateau"
                | "--pixel-density"
                | "--pixel-density-smoke"
                | "--bake-near-plateau"
                | "--footprint-near-plateau"
                | "--bench-near-plateau"
                | "--repeat-near-plateau"
                | "--capture-near-plateau"
                | "--depth-ablation"
                | "--capture-depth-ablation"
                | "--repeat-depth-ablation"
        )
    ) {
        anyhow::ensure!(args.next().is_none(), "regional scene takes no arguments");
        return gpu::views::representative::regional::run(first.as_deref().unwrap());
    }
    if matches!(
        first.as_deref(),
        Some(
            "--compile-rock"
                | "--compiled-rock"
                | "--compiled-rock-smoke"
                | "--compiled-rock-load"
                | "--compiled-rock-bench"
        )
    ) {
        anyhow::ensure!(
            args.next().is_none(),
            "compiled rock takes no extra arguments"
        );
        return gpu::views::representative::symbol::compiled::run(first.as_deref().unwrap());
    }
    if matches!(
        first.as_deref(),
        Some("--rock-symbol" | "--rock-symbol-smoke" | "--rock-symbol-bench")
    ) {
        anyhow::ensure!(
            args.next().is_none(),
            "rock symbol takes no extra arguments"
        );
        return gpu::views::representative::symbol::run(
            first.as_deref().unwrap().ends_with("smoke"),
            first.as_deref().unwrap().ends_with("bench"),
        );
    }
    if first.as_deref() == Some("--adaptive-assets") {
        anyhow::ensure!(args.next().is_none(), "adaptive assets takes no arguments");
        return gpu::views::representative::physical::run();
    }
    if matches!(
        first.as_deref(),
        Some("--bake-assets" | "--assets" | "--assets-smoke")
    ) {
        anyhow::ensure!(
            args.next().is_none(),
            "asset experiment takes no extra arguments"
        );
        return gpu::views::representative::run(first.as_deref().unwrap());
    }
    if matches!(
        first.as_deref(),
        Some("--compress-views" | "--compressed-views" | "--compressed-views-smoke")
    ) {
        anyhow::ensure!(
            args.next().is_none(),
            "compression takes no extra arguments"
        );
        return gpu::views::compressed::run(first.as_deref().unwrap());
    }
    if matches!(first.as_deref(), Some("--bake-views" | "--views")) {
        anyhow::ensure!(
            args.next().is_none(),
            "view experiment takes no extra arguments"
        );
        return gpu::views::run(first.as_deref() == Some("--bake-views"));
    }
    if matches!(
        first.as_deref(),
        Some("--summary-cost" | "--summary-cost-smoke")
    ) {
        anyhow::ensure!(
            args.next().is_none(),
            "summary cost mode takes no extra arguments"
        );
        return summary_cost::run(first.as_deref() == Some("--summary-cost-smoke"));
    }
    if first.as_deref() == Some("--quality-check") {
        anyhow::ensure!(
            args.next().is_none(),
            "--quality-check takes no extra arguments"
        );
        return quality::verify_endpoints();
    }
    if first.as_deref() == Some("--quality") {
        anyhow::ensure!(args.next().is_none(), "--quality takes no extra arguments");
        return quality::run();
    }
    if first.as_deref() == Some("--multiscale") {
        anyhow::ensure!(
            args.next().is_none(),
            "--multiscale takes no extra arguments"
        );
        return multiscale::run();
    }
    if first.as_deref() == Some("--adaptive") {
        anyhow::ensure!(args.next().is_none(), "--adaptive takes no extra arguments");
        return adaptive::run();
    }
    let output = match first.as_deref() {
        None => PathBuf::from("artifacts/appearance-probe"),
        Some("--output") => PathBuf::from(args.next().context("--output requires path")?),
        _ => anyhow::bail!(
            "usage: appearance_probe [--output PATH] | --adaptive | --multiscale | --quality | --quality-check | --summary-cost | --summary-cost-smoke | --bake-views | --views"
        ),
    };
    anyhow::ensure!(args.next().is_none(), "unexpected arguments");
    fs::create_dir_all(&output)?;
    let start = Instant::now();
    let mut gpu = pollster::block_on(Gpu::new())?;
    println!(
        "Adapter: {}. GPU timestamps: {:?}",
        gpu.adapter, gpu.timestamp_period_ns
    );
    let cases = [
        (
            "near_diffuse",
            Case {
                distance: 3.,
                tilt: 0.,
                light_angle: 25.,
                exponent: 8.,
                specular: 0.,
                shift: 0.,
            },
        ),
        (
            "middle_diffuse",
            Case {
                distance: 6.,
                tilt: 0.,
                light_angle: 25.,
                exponent: 8.,
                specular: 0.,
                shift: 0.,
            },
        ),
        (
            "far_diffuse",
            Case {
                distance: 12.,
                tilt: 0.,
                light_angle: 25.,
                exponent: 8.,
                specular: 0.,
                shift: 0.,
            },
        ),
        (
            "far_rough",
            Case {
                distance: 12.,
                tilt: 0.,
                light_angle: 25.,
                exponent: 8.,
                specular: 2.,
                shift: 0.,
            },
        ),
        (
            "far_medium",
            Case {
                distance: 12.,
                tilt: 0.,
                light_angle: 25.,
                exponent: 32.,
                specular: 2.,
                shift: 0.,
            },
        ),
        (
            "middle_glossy",
            Case {
                distance: 6.,
                tilt: 0.,
                light_angle: 25.,
                exponent: 32.,
                specular: 2.,
                shift: 0.,
            },
        ),
        (
            "far_glossy",
            Case {
                distance: 12.,
                tilt: 0.,
                light_angle: 25.,
                exponent: 96.,
                specular: 2.,
                shift: 0.,
            },
        ),
        (
            "tilted_glossy",
            Case {
                distance: 6.,
                tilt: 45.,
                light_angle: -20.,
                exponent: 32.,
                specular: 2.,
                shift: 0.,
            },
        ),
        (
            "grazing_view",
            Case {
                distance: 6.,
                tilt: 75.,
                light_angle: -20.,
                exponent: 96.,
                specular: 2.,
                shift: 0.,
            },
        ),
        (
            "grazing_light",
            Case {
                distance: 6.,
                tilt: 0.,
                light_angle: 82.,
                exponent: 32.,
                specular: 2.,
                shift: 0.,
            },
        ),
    ];
    let mut rows = Vec::<Value>::new();
    let mut meshes = Vec::<Value>::new();
    let mut sequence = Vec::<Value>::new();
    for n in [32, 128, 512] {
        let mesh = geometry::generate(n);
        let uploaded = gpu.upload(&mesh)?;
        meshes.push(json!({"cells_per_side":n,"triangles":mesh.vertices.len()/3,"generation_ms":mesh.generation_ms,"aggregation_ms":mesh.aggregation_ms,"upload_and_wait_ms":uploaded.upload_ms,"full_vertex_bytes":std::mem::size_of_val(mesh.vertices.as_slice()),"proxy_vertex_bytes":std::mem::size_of_val(mesh.proxy.as_slice()),"distribution32_bytes":std::mem::size_of_val(mesh.lobes32.as_slice()),"distribution128_bytes":std::mem::size_of_val(mesh.lobes128.as_slice()),"proxy_triangles":mesh.proxy.len()/3}));
        for (case_index, (name, case)) in cases.into_iter().enumerate() {
            let prefix = format!("n{n}_{name}");
            // Native timings are compared at identical resolution with the same response function.
            let mut ordered: Vec<Option<gpu::Capture>> = (0..4).map(|_| None).collect();
            for offset in 0..4 {
                let mode = (case_index + offset) % 4;
                ordered[mode] = Some(gpu.render(&uploaded, case, mode, SIZE, 1, true)?);
            }
            let native = ordered
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .context("all methods rendered")?;
            if name == "near_diffuse" {
                let repeated = gpu.render(&uploaded, case, 0, SIZE, 1, false)?.pixels;
                anyhow::ensure!(native[0].pixels == repeated, "repeated GPU capture differs");
            }
            let reference4 = gpu.render(&uploaded, case, 0, SIZE, 4, false)?.pixels;
            let (reference8, convergence) = reference(&mut gpu, &uploaded, case)?;
            anyhow::ensure!(
                reference8.iter().any(|p| p[3] > 0.99),
                "empty rasterized reference"
            );
            let regular_grid_error = metrics::compare(&reference8, &reference4);
            metrics::png(
                &output.join(format!("{prefix}_reference.png")),
                &reference8,
                SIZE,
            )?;
            let mut comparisons = Vec::new();
            for (mode, method) in METHODS.iter().enumerate() {
                let image = if mode == 0 {
                    reference4.clone()
                } else {
                    gpu.render(&uploaded, case, mode, SIZE, 4, false)?.pixels
                };
                metrics::png(&output.join(format!("{prefix}_{method}.png")), &image, SIZE)?;
                if mode > 0 {
                    metrics::error_png(
                        &output.join(format!("{prefix}_{method}_error.png")),
                        &reference8,
                        &image,
                        SIZE,
                    )?;
                }
                comparisons.push(json!({"method":method,"native_timing":native[mode].timing,"native_error_vs_reference":metrics::compare(&reference8,&native[mode].pixels),"filtered_error_vs_reference":metrics::compare(&reference8,&image)}));
            }
            println!(
                "{prefix}: 16/32-pass reference RMSE {:.6}",
                convergence.rmse_surface
            );
            rows.push(json!({"id":prefix,"cells_per_side":n,"case":name,"parameters":case,"reference_half_vs_full":convergence,"regular_grid_4x_error":regular_grid_error,"methods":comparisons}));
        }
        if n == 512 {
            let mut previous: Option<Vec<Vec<[f32; 4]>>> = None;
            for frame in 0..12 {
                let phase = frame as f32 / 11.;
                let case = Case {
                    distance: 6.,
                    tilt: 20. + 4. * phase,
                    light_angle: 5. + 25. * phase,
                    exponent: 96.,
                    specular: 2.,
                    shift: 0.015 * phase,
                };
                let mut images: Vec<Vec<[f32; 4]>> = Vec::new();
                let mut data = Vec::new();
                for (index, mode) in [0, 1, 2].into_iter().enumerate() {
                    let image = if mode == 0 {
                        reference(&mut gpu, &uploaded, case)?.0
                    } else {
                        gpu.render(&uploaded, case, mode, SIZE, 4, false)?.pixels
                    };
                    let method = METHODS[mode];
                    let file = format!("motion_{frame:02}_{method}.png");
                    metrics::png(&output.join(&file), &image, SIZE)?;
                    if index > 0 {
                        data.push(json!({"method":method,"error":metrics::compare(&images[0],&image),"temporal_residual_rmse":previous.as_ref().map(|p|metrics::temporal(&p[0],&p[index],&images[0],&image))}));
                    }
                    images.push(image);
                }
                sequence.push(json!({"frame":frame,"parameters":case,"methods":data}));
                previous = Some(images);
            }
        }
        // Mesh and upload drop here: only one density is resident at a time.
    }
    let report = json!({
        "schema":1,"experiment":"appearance_probe","adapter":gpu.adapter,"timestamp_period_ns":gpu.timestamp_period_ns,
        "image_size":SIZE,"reference_axis_samples":8,"reference_jitter_passes":32,"reference_samples_per_pixel":2048,"candidate_axis_samples":4,"benchmark_axis_samples":1,
        "reference_filter":"32 Halton-jittered 8x8 grids, linear box average; convergence 16-pass prefix vs32; numerical, not ground truth",
        "timing_scope":"31 GPU render passes after one completed warmup, includes vertex aggregate evaluation and attachment clear/store; excludes resource preparation/upload/readback/PNG; rotated method order across cases, shared system load",
        "lighting":"uniform directional light and uniform view direction for shading; perspective projection; Lambert-like diffuse plus unnormalized Blinn-Phong lobe; no light shadows/GI",
        "aggregate":"8x8 fixed regions, projected-area-weighted joint normal/material bins; no view-dependent hidden-surface correction, no adaptive hierarchy",
        "temporal_metric":"RMSE of difference of image residuals at fixed screen coordinates; includes edges; not perceptual or optical-flow compensated",
        "display":"Reinhard then sRGB; metrics use untonemapped linear RGB; error maps absolute RGB x8 then same display transform",
        "mesh_records":meshes,"cases":rows,"motion":sequence,"total_seconds":start.elapsed().as_secs_f64()
    });
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "Report: {} ({} seconds)",
        output.join("report.json").display(),
        start.elapsed().as_secs()
    );
    Ok(())
}
fn main() -> Result<()> {
    run()
}
