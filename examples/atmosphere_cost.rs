//! End-to-end offscreen draw+GPU completion, not a GPU timestamp or gameplay FPS.
use anyhow::Result;
use ignis::{engine::Engine, model::Scene, render::Renderer};
use serde_json::json;
use std::{path::PathBuf, time::Instant};
fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| "artifacts/atmosphere/mountains-compiled.scene.json".into());
    let scene: Scene = serde_json::from_slice(&std::fs::read(&path)?)?;
    let engine = Engine::new(scene)?;
    let mut frame = engine.render_frame();
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&instance, None))?;
    let (width, height) = (1728, 1117);
    let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("atmosphere cost current target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let mut rows = Vec::new();
    for (case, distance, density, fov, color_error) in [
        ("near", 100., 0.009, 35., 0.002),
        ("far", 1000., 0.009, 35., 0.002),
        ("horizon", 9000., 0.009, 35., 0.002),
        ("return", 100., 0.009, 35., 0.002),
        ("visible", 9000., 0.0004, 12., 0.01),
    ] {
        frame.camera.position = [0., 20., distance];
        frame.camera.target = [0., 10., 0.];
        frame.camera.fov_degrees = fov;
        frame.light.atmosphere.as_mut().unwrap().density = density;
        frame.light.atmosphere.as_mut().unwrap().color_error = color_error;
        for repeat in 0..4 {
            // Alternate order to expose warm-up/thermal bias.
            for enabled in if repeat % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                frame.light.atmosphere.as_mut().unwrap().reduce_details = enabled;
                let mut samples = Vec::new();
                let mut transition = 0.;
                for i in 0..44 {
                    let start = Instant::now();
                    renderer.draw(&frame, &view, width, height)?;
                    renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                    let elapsed = start.elapsed().as_secs_f64() * 1000.;
                    if i == 0 {
                        transition = elapsed;
                    }
                    if i >= 12 {
                        samples.push(elapsed);
                    }
                }
                samples.sort_by(f64::total_cmp);
                rows.push(json!({"case":case,"distance":distance,"repeat":repeat,"reduced":enabled,
                    "median_ms":samples[16],"p95_ms":samples[30],"transition_ms":transition,"stats":renderer.info()["distant_selection"]}));
            }
        }
    }
    frame.camera.fov_degrees = 35.;
    frame.light.atmosphere.as_mut().unwrap().density = 0.009;
    frame.light.atmosphere.as_mut().unwrap().color_error = 0.002;
    let mut moving = Vec::new();
    for enabled in [false, true, false, true] {
        frame.light.atmosphere.as_mut().unwrap().reduce_details = enabled;
        let mut times = Vec::new();
        for i in 0..96 {
            let phase = if i < 48 { i } else { 95 - i };
            frame.camera.position = [
                (i as f32 * 0.11).sin() * 12.,
                20.,
                100. + phase as f32 / 47. * 8900.,
            ];
            frame.light.direction = [(i as f32 * 0.03).sin(), -1., (i as f32 * 0.03).cos()];
            let start = Instant::now();
            renderer.draw(&frame, &view, width, height)?;
            renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
            times.push(start.elapsed().as_secs_f64() * 1000.);
        }
        times.sort_by(f64::total_cmp);
        moving.push(
            json!({"reduced":enabled,"median_ms":times[48],"p95_ms":times[91],"max_ms":times[95]}),
        );
    }
    let report = json!({"method":"native1728x1117, draw+GPU wait per frame,12warmups/32samples,4alternating rounds; includes CPU selection/uploads/submission and GPU completion, excludes PNG/presentation/simulation", "adapter":renderer.info(),"source":path,"rows":rows,"moving_camera_and_sun":moving});
    std::fs::write(
        "artifacts/atmosphere/cost.json",
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", report);
    Ok(())
}
