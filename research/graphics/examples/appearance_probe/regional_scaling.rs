//! Source-density scaling; redundant tessellation is a control, not the result.
use super::*;
#[path = "scaling_ablation.rs"]
mod ablation;
#[path = "automatic_shading.rs"]
mod automatic;
#[path = "regional_scaling_build.rs"]
mod build;
#[path = "cost_scaling.rs"]
mod calibrated;
#[path = "combined_symbol.rs"]
mod combined_symbol;
#[path = "cost_model.rs"]
mod cost;
#[path = "pixel_density.rs"]
mod density;
#[path = "geometric_comparison.rs"]
mod geometric_comparison;
#[path = "geometric_field.rs"]
mod geometric_field;
#[path = "hierarchical_comparison.rs"]
mod hierarchical_comparison;
#[path = "hierarchical_field.rs"]
mod hierarchical_field;
#[path = "hybrid_scaling.rs"]
mod hybrid;
#[path = "material_symbol.rs"]
mod material_symbol;
#[path = "near_plateau.rs"]
mod plateau;
#[path = "regional_scaling_render.rs"]
mod render;
#[path = "tiled_comparison.rs"]
mod tiled_comparison;
#[path = "tiled_field.rs"]
mod tiled_field;
use build::{Asset, bake};
#[cfg(test)]
use build::{GRIDS, V, interpolate};
use render::{Renderer, pose};
#[cfg(test)]
use render::{precise_error, projected_error};
const ROOT: &str = "artifacts/appearance-probe/regional-scaling";
const MODES: [&str; 4] = ["full", "global_old", "global_precise", "regional_precise"];
use super::scaling::quality;
pub(super) fn run(mode: &str) -> Result<()> {
    if mode.contains("tiled-field") {
        return tiled_comparison::run(mode);
    }
    if mode.contains("hierarchical-field") {
        return hierarchical_comparison::run(mode);
    }
    if mode.contains("geometric-field") {
        return geometric_comparison::run(mode);
    }
    if mode.contains("combined-symbol") {
        return combined_symbol::run(mode);
    }
    if mode.contains("pixel-density") {
        return density::run(mode.ends_with("smoke"));
    }
    if mode.contains("near-plateau") {
        return plateau::run(mode);
    }
    if mode.contains("automatic-shading") {
        return automatic::run(mode);
    }
    if mode.contains("ablat") {
        return ablation::run(mode);
    }
    if mode.contains("cost-scaling") {
        return calibrated::run(mode);
    }
    if mode.contains("hybrid-") {
        return hybrid::run(mode);
    }
    if mode == "--regional-transitions" {
        return transitions();
    }
    if mode == "--bake-regional-scaling" {
        return bake();
    }
    if mode == "--confirm-regional-scaling" {
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
                    .context("run --bake-regional-scaling")?,
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
                    let mut triangles = vec![asset.levels.last().unwrap().count as usize / 3];
                    for m in 1..4 {
                        let (candidate, level) = renderer.capture(&uploaded, &asset, &cam, m)?;
                        save(&root.join(format!("{name}_{m}.png")), &candidate, size[0])?;
                        let (check, residual) =
                            quality(&full, &candidate, &previous[m as usize - 1]);
                        previous[m as usize - 1] = residual;
                        checks.push(check);
                        triangles.push(level);
                    }
                    let timings: Option<serde_json::Value> = None;
                    println!(
                        "{name}: {triangles:?}, gate {:?}",
                        checks
                            .iter()
                            .map(|c| c["quality_gate"].clone())
                            .collect::<Vec<_>>()
                    );
                    let (draws, n, _) = render::selection(&asset, &cam, 3);
                    let draw_triangles = draws[..n].iter().map(|d| d.1 / 3).collect::<Vec<_>>();
                    rows.push(json!({"id":name,"family":family,"density":density,"factor":asset.factor,"resolution":resolution,"size":size,"frame":frame,"distance":cam.eye.length(),"triangles":triangles,"levels":[0,0,0,0],"source_triangles_per_covered_pixel":asset.levels.last().unwrap().count as f64/3./visible as f64,"covered_pixels":visible,"checks":checks,"draw_triangles":draw_triangles,"timings":timings}));
                }
            }
            assets.push(serde_json::to_value(&asset)?);
            fs::write(
                root.join("report.json"),
                serde_json::to_vec_pretty(
                    &json!({"schema":1,"adapter":renderer.gpu.adapter,"modes":MODES,"assets":assets,"rows":rows,"threshold_px":0.15,"regions":64,"throughput":"See confirmation.json: 128 submissions, four densities interleaved, selector included"}),
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
    fn projection_bound_contains_sampled_vertical_displacements() {
        let bounds = [[-6., -0.7, -6.], [6., 0.7, 6.]];
        for frame in 0..5 {
            for size in RESOLUTIONS {
                let c = pose(frame, size);
                let e = 0.03;
                let bound = precise_error(&c, e, bounds);
                for x in 0..13 {
                    for y in 0..13 {
                        for z in 0..13 {
                            let p = Vec3::new(
                                x as f32 - 6.,
                                -0.7 + y as f32 * 1.4 / 12.,
                                z as f32 - 6.,
                            );
                            for sign in [-1., 1.] {
                                let q = p + Vec3::Y * e * sign;
                                if q.y < -0.7 || q.y > 0.7 {
                                    continue;
                                }
                                let a = c.matrix.project_point3(p);
                                let b = c.matrix.project_point3(q);
                                let delta = (a - b).truncate()
                                    * glam::Vec2::new(size[0] as f32, size[1] as f32)
                                    * 0.5;
                                assert!(delta.length() <= bound + 0.001);
                            }
                        }
                    }
                }
                assert!(bound < projected_error(&c, e));
            }
        }
        let mut c = pose(0, RESOLUTIONS[0]);
        c.view = Mat4::IDENTITY;
        assert!(precise_error(&c, 0.01, bounds).is_infinite());
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
        let cases = (0..2)
            .flat_map(|r| (0..5).map(move |f| (r, f)))
            .collect::<Vec<_>>();
        for (resolution, frame) in cases {
            renderer.resize(RESOLUTIONS[resolution]);
            let timings = renderer.bench_set(&inputs, frame, RESOLUTIONS[resolution], 128)?;
            println!("confirmed family{family} resolution{resolution} frame{frame}");
            result.push(json!({"family":family,"resolution":resolution,"frame":frame,"batch_frames":128,"timings":timings,"resident_asset_bytes":assets.iter().map(|a|a.bytes).sum::<u64>()}));
            fs::write(
                root.join("confirmation.json"),
                serde_json::to_vec_pretty(&result)?,
            )?;
        }
    }
    fs::write(
        root.join("confirmation.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(())
}

// Intermediate camera positions exercise mixed levels instead of only powers of two.
fn transitions() -> Result<()> {
    let root = Path::new(ROOT);
    let mut renderer = pollster::block_on(Renderer::new())?;
    let a: Asset = serde_json::from_slice(&fs::read(root.join("f1_d3.json"))?)?;
    let u = renderer.upload(&a)?;
    let mut rows = Vec::new();
    for size in RESOLUTIONS {
        let mut last = 0.;
        let mut count = 0;
        for half in 28..113 {
            let distance = half as f32 / 2.;
            let cam = render::camera_at(distance, size);
            let (_, draws, triangles) = render::selection(&a, &cam, 3);
            if draws <= 1 || distance - last < 5. || count >= 3 {
                continue;
            }
            last = distance;
            count += 1;
            let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
            let (regional, _) = renderer.capture(&u, &a, &cam, 3)?;
            let (check, _) = quality(&full, &regional, &[]);
            let id = format!("transition_{}_{}", size[0], half);
            save(&root.join(format!("{id}_0.png")), &full, size[0])?;
            save(&root.join(format!("{id}_3.png")), &regional, size[0])?;
            println!("{id}: distance{distance}, {draws} draws, {triangles} triangles, {check}");
            rows.push(json!({"id":id,"size":size,"distance":distance,"draws":draws,"triangles":triangles,"check":check}));
        }
        ensure!(count > 0, "trajectory did not exercise mixed levels");
    }
    fs::write(
        root.join("transitions.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}
