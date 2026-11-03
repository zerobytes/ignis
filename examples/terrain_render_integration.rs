//! Integration oracle: native terrain/landform publications use the same renderer.
use anyhow::Result;
use ignis::{engine::Engine, model::Scene, render::Renderer};
use serde_json::{Value, json};
use std::fs;

fn configure(renderer: &mut Renderer, enabled: bool) {
    renderer.set_visibility_culling(enabled);
    renderer.set_geometry_indexing(enabled);
    renderer.set_compact_vertices(enabled);
    renderer.set_minimal_vertices(enabled);
    renderer.set_occlusion_culling(enabled);
}

fn main() -> Result<()> {
    let out = "artifacts/terrain-render-integration";
    let manifest: Value = serde_json::from_slice(&fs::read(format!("{out}/manifest.json"))?)?;
    let gpu = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&gpu, None))?;
    let mut rows = vec![];
    for case in manifest["cases"].as_array().unwrap() {
        let name = case["id"].as_str().unwrap();
        let scene: Scene = serde_json::from_slice(&fs::read(case["path"].as_str().unwrap())?)?;
        let mut frame = Engine::new(scene)?.render_frame();
        let position = glam::Vec3::from(frame.camera.position);
        let target = glam::Vec3::from(frame.camera.target);
        for pose in 0..3 {
            frame.camera.position = (target + (position - target) * (1. + pose as f32)).into();
            frame.light.direction = [0.65 - pose as f32 * 0.3, -1., -0.25 + pose as f32 * 0.2];
            let (w, h) = if pose == 1 { (640, 864) } else { (1280, 800) };
            // Keep appearance identical while testing the lossless optimizations.
            frame.light.atmosphere.as_mut().unwrap().reduce_details = false;
            configure(&mut renderer, false);
            let reference = renderer.capture(&frame, w, h)?;
            anyhow::ensure!(
                reference == renderer.capture(&frame, w, h)?,
                "{name}/{pose}: reference unstable"
            );
            let before = renderer.info();
            configure(&mut renderer, true);
            let optimized = renderer.capture(&frame, w, h)?;
            anyhow::ensure!(
                optimized == renderer.capture(&frame, w, h)?,
                "{name}/{pose}: optimized unstable"
            );
            anyhow::ensure!(
                reference == optimized,
                "{name}/{pose}: optimization changed pixels"
            );
            let exact = renderer.info();
            // Separately exercise compiled atmospheric representations. Their
            // approximation has a visual budget; do not label it pixel-exact.
            frame.light.atmosphere.as_mut().unwrap().reduce_details = true;
            let combined = renderer.capture(&frame, w, h)?;
            let combined_info = renderer.info();
            let a = ignis::image_similarity::Image::decode(&reference)?;
            let b = ignis::image_similarity::Image::decode(&combined)?;
            let errors: Vec<_> = a
                .pixels
                .iter()
                .flatten()
                .zip(b.pixels.iter().flatten())
                .map(|(a, b)| a.abs_diff(*b))
                .collect();
            let row = json!({"case":name,"pose":pose,"width":w,"height":h,
                "lossless_exact":true,"reference":before,"optimized":exact,
                "combined":combined_info,"combined_max_rgb8":errors.iter().max(),
                "combined_mean_rgb8":errors.iter().map(|&n|u64::from(n)).sum::<u64>() as f64 / errors.len() as f64});
            if pose == 0 {
                fs::write(format!("{out}/{name}-reference.png"), reference)?;
                fs::write(format!("{out}/{name}-optimized.png"), optimized)?;
                fs::write(format!("{out}/{name}-combined.png"), combined)?;
            }
            println!(
                "{name}/{pose}: lossless exact, combined max{}",
                row["combined_max_rgb8"]
            );
            rows.push(row);
            fs::write(
                format!("{out}/render.json"),
                serde_json::to_vec_pretty(&json!({
                "build_id":manifest["build_id"],"rows":rows,"performance_claim":false}))?,
            )?;
        }
    }
    Ok(())
}
