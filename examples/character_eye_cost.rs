//! Warm ocular draw cost on reused targets; not presented/gameplay FPS.
use anyhow::Result;
use ignis::{engine::Engine, model::Scene, render::Renderer};
use serde_json::json;
use std::{fs, time::Instant};
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let instance = wgpu::Instance::default();
    let mut rows = vec![];
    for (label, path) in [("old caps", &args[1]), ("seeded iris and cornea", &args[2])] {
        let scene: Scene = serde_json::from_reader(std::io::BufReader::new(fs::File::open(path)?))?;
        let engine = Engine::new(scene)?;
        let mut frame = engine.render_frame();
        let mut renderer = pollster::block_on(Renderer::new(&instance, None))?;
        let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("eye cost"),
            size: wgpu::Extent3d {
                width: 1000,
                height: 900,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let wait = |r: &Renderer| -> Result<()> {
            r.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(30)),
            })?;
            Ok(())
        };
        let center = glam::Vec3::from(frame.camera.target);
        let direction = (glam::Vec3::from(frame.camera.position) - center).normalize();
        for (pose, distance) in [("close", 0.075), ("portrait", 0.36), ("body", 2.5)] {
            frame.camera.position = (center + direction * distance).to_array();
            for _ in 0..4 {
                renderer.draw(&frame, &view, 1000, 900)?;
                wait(&renderer)?;
            }
            let mut total = vec![];
            let mut submit = vec![];
            for _ in 0..16 {
                let t = Instant::now();
                renderer.draw(&frame, &view, 1000, 900)?;
                submit.push(t.elapsed().as_secs_f64() * 1000.);
                wait(&renderer)?;
                total.push(t.elapsed().as_secs_f64() * 1000.);
            }
            total.sort_by(f64::total_cmp);
            submit.sort_by(f64::total_cmp);
            rows.push(json!({"variant":label,"pose":pose,"median_draw_wait_ms":total[8],"median_cpu_submit_ms":submit[8],"draw_wait_samples_ms":total,"renderer":renderer.info(),"triangles":frame.meshes.triangle_count()}));
        }
    }
    fs::write(
        &args[3],
        serde_json::to_vec_pretty(
            &json!({"build":env!("IGNIS_BUILD_ID"),"pixels":[1000,900],"rows":rows,"scope":"16 warm samples/pose/variant, reused target, draw plus blocking completion; CPU prepare/submit separate; no GPU timestamps, readback, PNG, present cadence or gameplay FPS. Workload-specific; transparency has full-target overhead."}),
        )?,
    )?;
    Ok(())
}
