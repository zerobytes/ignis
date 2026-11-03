//! Reused offscreen target: CPU submit and draw+completion, never gameplay FPS.
use anyhow::Result;
use ignis::{engine::Engine, model::Scene, render::Renderer};
use serde_json::json;
use std::{fs, time::Instant};

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.
}
fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}
fn wait(renderer: &Renderer) -> Result<()> {
    renderer.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(30)),
    })?;
    Ok(())
}
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("scene path");
    let out = std::env::args().nth(2).expect("report path");
    let counts: Vec<usize> = std::env::args()
        .skip(3)
        .map(|s| s.parse().expect("instance count"))
        .collect();
    let started = Instant::now();
    let base: Scene = serde_json::from_reader(std::io::BufReader::new(fs::File::open(path)?))?;
    let deserialize_ms = ms(started);
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&instance, None))?;
    let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("lookdev repeated target"),
        size: wgpu::Extent3d {
            width: 1200,
            height: 1200,
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
    let mut rows = vec![];
    for count in counts {
        let mut scene = base.clone();
        scene.botany.clear();
        let specimens: Vec<_> = base
            .meshes
            .instances
            .iter()
            .filter(|i| i.id != "lookdev_ground")
            .collect();
        scene.meshes.instances.retain(|i| i.id == "lookdev_ground");
        scene.meshes.objects = Default::default();
        let side = (count as f32).sqrt().ceil() as usize;
        if let Some(ground) = scene.meshes.instances.first_mut() {
            ground.scale = (side as f32 / 2.).max(1.);
        }
        for n in 0..count {
            for original in &specimens {
                let mut item = (*original).clone();
                item.id = format!("copy_{n}_{}", item.id);
                item.position[0] += ((n % side) as f32 - (side - 1) as f32 * 0.5) * 1.45;
                item.position[2] += ((n / side) as f32 - (side - 1) as f32 * 0.5) * 1.45;
                scene.meshes.instances.push(item);
            }
        }
        if count > 1 {
            scene.camera.position = [side as f32 * 2.2, side as f32 * 2.6, side as f32 * 4.];
            scene.camera.target = [0., 0.5, 0.];
            if let Some(shadow) = &mut scene.light.shadow {
                shadow.extent = side as f32 * 1.5;
                shadow.center = [0., 0.5, 0.];
            }
        }
        let started = Instant::now();
        let engine = Engine::new(scene)?;
        let engine_ms = ms(started);
        let started = Instant::now();
        let mut frame = engine.render_frame();
        let frame_ms = ms(started);
        let original_camera = frame.camera.clone();
        let mut info = vec![];
        let mut totals = [vec![], vec![]];
        let mut cpu = [vec![], vec![]];
        let mut cold = vec![];
        for enabled in [false, true] {
            frame.light.shadow.as_mut().unwrap().receiver_plane_bias = enabled;
            let started = Instant::now();
            renderer.draw(&frame, &view, 1200, 1200)?;
            wait(&renderer)?;
            cold.push(ms(started));
            for _ in 0..3 {
                renderer.draw(&frame, &view, 1200, 1200)?;
                wait(&renderer)?;
            }
        }
        // Alternate option order and repeat the same camera for both variants.
        for sample in 0..24 {
            for index in if sample % 2 == 0 { [0, 1] } else { [1, 0] } {
                frame.camera = original_camera.clone();
                frame.camera.position[0] += (sample as f32 - 11.5) * 0.002;
                frame.light.shadow.as_mut().unwrap().receiver_plane_bias = index == 1;
                let started = Instant::now();
                renderer.draw(&frame, &view, 1200, 1200)?;
                cpu[index].push(ms(started));
                wait(&renderer)?;
                totals[index].push(ms(started));
            }
        }
        for enabled in [false, true] {
            frame.light.shadow.as_mut().unwrap().receiver_plane_bias = enabled;
            renderer.draw(&frame, &view, 1200, 1200)?;
            wait(&renderer)?;
            info.push(renderer.info());
        }
        let row = json!({"copies":count,"engine_ms":engine_ms,"frame_ms":frame_ms,
            "triangles_before_culling":frame.meshes.triangle_count(),"resident_vertices":frame.meshes.resident_vertex_count(),
            "first_draw_and_wait_ms":cold,"median_draw_and_wait_ms":totals.each_mut().map(|v|median(v)),
            "median_cpu_prepare_submit_ms":cpu.each_mut().map(|v|median(v)),
            "draw_and_wait_samples_ms":totals,"cpu_prepare_submit_samples_ms":cpu,
            "renderer_info":info});
        println!("copies{count}: {}", row["median_draw_and_wait_ms"]);
        rows.push(row);
        fs::write(
            &out,
            serde_json::to_vec_pretty(&json!({"build_id":env!("IGNIS_BUILD_ID"),
            "deserialize_ms":deserialize_ms,"physical_pixels":[1200,1200],"rows":rows,
            "variant_order":["legacy PCF","receiver-plane PCF"],
            "method":"24 paired moving-camera samples, alternating order, reused target, warm caches. Draw+blocking GPU completion; CPU includes prepare/upload/driver; GPU timestamps and present cadence unavailable in this headless harness. No capture/readback/PNG or simulation in samples. Logical geometry/target bytes are not total device memory. Shared hardware may have other workloads."}))?,
        )?;
    }
    Ok(())
}
