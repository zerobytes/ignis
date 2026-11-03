//! Benchmark the exact authored views; no world rescaling or renderer fork.
use anyhow::Result;
use ignis::{engine::Engine, model::Scene, render::Renderer};
use serde_json::{Value, json};
use std::{fs, time::Instant};

fn main() -> Result<()> {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "artifacts/atmosphere-landscape".into());
    let uncompiled_reference = std::env::args().any(|a| a == "--uncompiled-reference");
    let manifest: Value = serde_json::from_slice(&fs::read(format!("{root}/verification.json"))?)?;
    let scene: Scene = serde_json::from_slice(&fs::read(format!("{root}/landscape.scene.json"))?)?;
    let engine = Engine::new(scene)?;
    let mut frame = engine.render_frame();
    let compiled = frame.distant_meshes.clone();
    let empty = std::sync::Arc::new(Vec::new());
    let configure = |frame: &mut ignis::render::RenderFrame, enabled| {
        frame.light.atmosphere.as_mut().unwrap().reduce_details = enabled;
        frame.distant_meshes = if uncompiled_reference && !enabled {
            empty.clone()
        } else {
            compiled.clone()
        };
    };
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&instance, None))?;
    let (width, height) = (1728, 1117);
    let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("kilometre landscape benchmark"),
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
    let view = target.create_view(&Default::default());
    // Warm every authored profile before measuring. Record cold activations
    // separately; alternate profile order so time drift is not an effect gain.
    let cases = manifest["cases"].as_array().unwrap();
    if uncompiled_reference {
        anyhow::ensure!(
            cases
                .iter()
                .all(|c| c["light"]["atmosphere"]["edge_softness"].is_null()),
            "Uncompiled comparison requires blur absent in every profile"
        );
    }
    let mut activation = Vec::new();
    for case in cases {
        frame.camera = serde_json::from_value(case["camera"].clone())?;
        frame.light = serde_json::from_value(case["light"].clone())?;
        for enabled in [false, true] {
            configure(&mut frame, enabled);
            for i in 0..24 {
                let start = Instant::now();
                renderer.draw(&frame, &view, width, height)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                if i == 0 {
                    activation.push(json!({"case":case["id"],"selected":enabled,
                        "ms":start.elapsed().as_secs_f64()*1000.}));
                }
            }
        }
    }
    let mut rows = Vec::new();
    for round in 0..4 {
        for index in 0..cases.len() {
            let case = &cases[if round % 2 == 0 {
                index
            } else {
                cases.len() - 1 - index
            }];
            frame.camera = serde_json::from_value(case["camera"].clone())?;
            frame.light = serde_json::from_value(case["light"].clone())?;
            for enabled in if round % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                configure(&mut frame, enabled);
                let mut samples = Vec::new();
                let mut cpu_samples = Vec::new();
                let mut transition = 0.;
                for i in 0..52 {
                    let start = Instant::now();
                    renderer.draw(&frame, &view, width, height)?;
                    let cpu_ms = start.elapsed().as_secs_f64() * 1000.;
                    renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                    let ms = start.elapsed().as_secs_f64() * 1000.;
                    if i == 0 {
                        transition = ms;
                    }
                    if i >= 12 {
                        samples.push(ms);
                        cpu_samples.push(cpu_ms);
                    }
                }
                samples.sort_by(f64::total_cmp);
                cpu_samples.sort_by(f64::total_cmp);
                rows.push(json!({"case":case["id"],"round":round,"selected":enabled,
                    "median_ms":(samples[19]+samples[20])*0.5,"p95_ms":samples[37],
                    "cpu_draw_median_ms":(cpu_samples[19]+cpu_samples[20])*0.5,
                    "cpu_draw_p95_ms":cpu_samples[37],"transition_ms":transition,"stats":renderer.info()["distant_selection"],
                    "edges":renderer.info()["atmospheric_edges"]}));
            }
        }
    }
    // Include threshold crossings/uploads, not just stationary warmed frames.
    let mut motion = Vec::new();
    let authored = &manifest["cases"][1];
    for round in 0..2 {
        for enabled in if round == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            frame.light = serde_json::from_value(authored["light"].clone())?;
            configure(&mut frame, enabled);
            let mut samples = Vec::new();
            let mut substitutions = Vec::new();
            for i in 0usize..132 {
                let t = (i.saturating_sub(12) as f32 / 119.) * std::f32::consts::TAU;
                frame.camera.position = [
                    400. * t.sin(),
                    1200. + 150. * t.sin(),
                    7000. + 2000. * t.cos(),
                ];
                frame.camera.target = [0., 1300., 0.];
                frame.light.direction = [0.65 * t.cos(), -1., -0.25 + 0.3 * t.sin()];
                let start = Instant::now();
                renderer.draw(&frame, &view, width, height)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                if i >= 12 {
                    samples.push(start.elapsed().as_secs_f64() * 1000.);
                    substitutions.push(
                        renderer.info()["distant_selection"]["substituted_instances"].clone(),
                    );
                }
            }
            samples.sort_by(f64::total_cmp);
            motion.push(
                json!({"round":round,"selected":enabled,"median_ms":(samples[59]+samples[60])*0.5,
                "p95_ms":samples[113],"max_ms":samples[119],"substitutions":substitutions}),
            );
        }
    }
    let report = json!({"build_id":env!("IGNIS_BUILD_ID"),"adapter":renderer.info(),"rows":rows,"motion":motion,"cold_activation":activation,
        "uncompiled_reference":uncompiled_reference,
        "method":"1728x1117,4alternating rounds and profile order, all profiles prewarmed,12warmup+40samples; CPU draw+GPU completion wait. CPU draw includes selection, upload and driver submission; total adds blocking GPU completion wait (not GPU timestamps). Motion includes representation transitions. No capture/readback, presentation, simulation, source preparation or isolated-device guarantee."});
    fs::write(
        format!("{root}/cost.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
