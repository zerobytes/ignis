//! Source-density scaling; redundant tessellation is a control, not the result.
use super::*;
#[path = "triangle_scaling_build.rs"]
mod build;
#[path = "triangle_scaling_render.rs"]
mod render;
use build::{Asset, bake};
#[cfg(test)]
use build::{GRIDS, Level, V, interpolate};
use render::{Renderer, pose};
#[cfg(test)]
use render::{projected_error, select};
const ROOT: &str = "artifacts/appearance-probe/triangle-scaling";
const MODES: [&str; 4] = [
    "full",
    "position_lod",
    "normal_guard_lod",
    "normal_texture_lod",
];
pub(super) fn quality(
    reference: &[[f32; 4]],
    candidate: &[[f32; 4]],
    previous: &[[f32; 4]],
) -> (serde_json::Value, Vec<[f32; 4]>) {
    let mut sum = 0f64;
    let mut tail = Vec::new();
    let mut coverage = 0;
    let mut changed = 0;
    let mut temporal = 0f64;
    let mut temporal_n = 0;
    let mut residual = Vec::with_capacity(reference.len());
    for (i, (a, b)) in reference.iter().zip(candidate).enumerate() {
        let mut e = [
            b[0] - a[0],
            b[1] - a[1],
            b[2] - a[2],
            f32::from(a[3] > 0. || b[3] > 0.),
        ];
        if e[3] > 0. {
            sum += (0..3).map(|k| f64::from(e[k]).powi(2)).sum::<f64>();
            tail.push(e[..3].iter().map(|x| x.abs()).fold(0f32, f32::max));
            coverage += usize::from(a[3] != b[3]);
            changed += usize::from(a != b);
        }
        if !previous.is_empty() && (e[3] > 0. || previous[i][3] > 0.) {
            temporal += (0..3)
                .map(|k| f64::from(e[k] - previous[i][k]).powi(2))
                .sum::<f64>();
            temporal_n += 1;
        }
        if e[3] == 0. {
            e = [0.; 4];
        }
        residual.push(e);
    }
    tail.sort_unstable_by(f32::total_cmp);
    let n = tail.len();
    let rmse = (sum / (3 * n.max(1)) as f64).sqrt();
    let p99 = tail[(n * 99 / 100).min(n - 1)];
    let cover = coverage as f64 / n as f64;
    (
        json!({"foreground_pixels":n,"foreground_rmse":rmse,"p99_rgb":p99,"max_rgb":tail.last(),"coverage_mismatches":coverage,"coverage_fraction":cover,"changed_pixels":changed,"temporal_residual_rmse":if previous.is_empty(){None}else{Some((temporal/(3*temporal_n.max(1))as f64).sqrt())},"quality_gate":rmse<=0.005&&p99<=0.02&&cover<=0.0005}),
        residual,
    )
}
pub(super) fn run(mode: &str) -> Result<()> {
    if mode == "--bake-triangle-scaling" {
        return bake();
    }
    if mode == "--confirm-triangle-scaling" {
        return confirm();
    }
    let root = Path::new(ROOT);
    let mut renderer = pollster::block_on(Renderer::new())?;
    let mut rows = Vec::new();
    let mut assets = Vec::new();
    for family in 0..2 {
        for density in [0, 3, 1, 2] {
            let id = format!("f{family}_d{density}");
            let asset: Asset = serde_json::from_slice(
                &fs::read(root.join(format!("{id}.json")))
                    .context("run --bake-triangle-scaling")?,
            )?;
            let uploaded = renderer.upload(&asset)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                let mut previous = vec![Vec::new(); 3];
                for frame in 0..9 {
                    let cam = pose(frame, size);
                    let name = format!("{id}_r{resolution}_{frame}");
                    let (full, _) = renderer.capture(&uploaded, &asset, &cam, 0)?;
                    save(&root.join(format!("{name}_0.png")), &full, size[0])?;
                    let visible = full.iter().filter(|p| p[3] > 0.).count();
                    let mut checks = Vec::new();
                    let mut levels = vec![asset.levels.len() - 1];
                    for m in 1..4 {
                        let (candidate, level) = renderer.capture(&uploaded, &asset, &cam, m)?;
                        save(&root.join(format!("{name}_{m}.png")), &candidate, size[0])?;
                        let (check, residual) =
                            quality(&full, &candidate, &previous[m as usize - 1]);
                        previous[m as usize - 1] = residual;
                        checks.push(check);
                        levels.push(level);
                    }
                    let timings = if frame <= 4 {
                        Some(renderer.bench(&uploaded, &asset, frame, size)?)
                    } else {
                        None
                    };
                    let triangles = levels
                        .iter()
                        .map(|&l| asset.levels[l].count / 3)
                        .collect::<Vec<_>>();
                    println!(
                        "{name}: {triangles:?}, gate {:?}",
                        checks
                            .iter()
                            .map(|c| c["quality_gate"].clone())
                            .collect::<Vec<_>>()
                    );
                    rows.push(json!({"id":name,"family":family,"density":density,"factor":asset.factor,"resolution":resolution,"size":size,"frame":frame,"distance":cam.eye.length(),"triangles":triangles,"levels":levels,"source_triangles_per_covered_pixel":asset.levels.last().unwrap().count as f64/3./visible as f64,"covered_pixels":visible,"checks":checks,"timings":timings}));
                }
            }
            assets.push(serde_json::to_value(&asset)?);
            fs::write(
                root.join("report.json"),
                serde_json::to_vec_pretty(
                    &json!({"schema":1,"adapter":renderer.gpu.adapter,"modes":MODES,"assets":assets,"rows":rows,"threshold_px":0.15,"normal_guard":0.005,"throughput":"8 submissions same pose; selector and camera evaluated every frame; no readback"}),
                )?,
            )?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quality_gate_does_not_dilute_a_missing_pixel_with_background() {
        let mut reference = vec![[0.; 4]; 10_000];
        for pixel in &mut reference[..100] {
            *pixel = [0.3, 0.3, 0.3, 1.];
        }
        let mut candidate = reference.clone();
        candidate[0] = [0.; 4];
        let (q, _) = quality(&reference, &candidate, &[]);
        assert_eq!(q["foreground_pixels"], 100);
        assert_eq!(q["coverage_mismatches"], 1);
        assert_eq!(q["quality_gate"], false);
    }
    #[test]
    fn exact_density_ratios() {
        for (g, m) in GRIDS.iter().zip([1, 2, 10, 100]) {
            assert_eq!(g[0] * g[1] * 2, 32768 * m);
        }
    }
    #[test]
    fn diagonal_interpolation_preserves_planar_height_and_normals() {
        let source = (0..=8)
            .flat_map(|y| {
                (0..=8).map(move |x| V {
                    p: [x as f32, 2. * x as f32 + 3. * y as f32, y as f32],
                    n: [0., 1., 0.],
                })
            })
            .collect::<Vec<_>>();
        for s in [1, 2, 4, 8] {
            for y in 0..=8 {
                for x in 0..=8 {
                    let v = interpolate(&source, [8, 8], s, x, y);
                    assert!((v.p[1] - (2 * x + 3 * y) as f32).abs() < 0.00001);
                    assert_eq!(v.n, [0., 1., 0.]);
                }
            }
        }
    }
    #[test]
    fn projected_error_scales_with_resolution_and_distance() {
        let c = pose(0, RESOLUTIONS[0]);
        let half = pose(0, RESOLUTIONS[1]);
        assert!((projected_error(&c, 0.01) / projected_error(&half, 0.01) - 2.).abs() < 0.0001);
        assert!(projected_error(&pose(4, RESOLUTIONS[0]), 0.01) < projected_error(&c, 0.01));
    }
    #[test]
    fn appearance_guard_rejects_shape_only_shortcut() {
        let a = Asset {
            id: String::new(),
            family: 1,
            factor: 100,
            grid: [1280; 2],
            bytes: 0,
            build_ms: 0.,
            levels: vec![
                Level {
                    first: 0,
                    count: 6,
                    position_error: 0.,
                    normal_error: 0.1,
                    stride: 2,
                },
                Level {
                    first: 6,
                    count: 24,
                    position_error: 0.,
                    normal_error: 0.,
                    stride: 1,
                },
            ],
        };
        let c = pose(4, RESOLUTIONS[0]);
        assert_eq!(select(&a, &c, 1), 0);
        assert_eq!(select(&a, &c, 2), 1);
        assert_eq!(select(&a, &c, 3), 0);
    }
}

fn confirm() -> Result<()> {
    let root = Path::new(ROOT);
    let mut renderer = pollster::block_on(Renderer::new())?;
    let mut result = Vec::new();
    for family in 0..2 {
        let assets = (0..4)
            .map(|d| -> Result<Asset> {
                Ok(serde_json::from_slice(&fs::read(
                    root.join(format!("f{family}_d{d}.json")),
                )?)?)
            })
            .collect::<Result<Vec<_>>>()?;
        let uploaded = assets
            .iter()
            .map(|a| renderer.upload(a))
            .collect::<Result<Vec<_>>>()?;
        let inputs = uploaded.iter().zip(&assets).collect::<Vec<_>>();
        let cases = if family == 0 {
            vec![(0, 0), (0, 3)]
        } else {
            vec![(0, 0), (0, 3), (1, 0), (1, 2), (1, 3)]
        };
        for (resolution, frame) in cases {
            renderer.resize(RESOLUTIONS[resolution]);
            let timings = renderer.bench_set(&inputs, frame, RESOLUTIONS[resolution], 128)?;
            println!("confirmed family{family} resolution{resolution} frame{frame}");
            result.push(json!({"family":family,"resolution":resolution,"frame":frame,"batch_frames":128,"timings":timings,"resident_asset_bytes":assets.iter().map(|a|a.bytes).sum::<u64>()}));
        }
    }
    fs::write(
        root.join("confirmation.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(())
}
