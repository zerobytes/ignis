//! Same vegetation/geometry: submission order and occlusion, no content reduction.
use anyhow::{Context, Result, ensure};
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::{Camera, Scene},
    render::Renderer,
};
use serde_json::{Value, json};
use std::{fs, io::BufReader, path::Path, time::Instant};

fn stats(v: &mut [f64]) -> Value {
    v.sort_by(f64::total_cmp);
    json!({"n":v.len(),"median_ms":v[v.len()/2],"p95_ms":v[(v.len()-1)*95/100],"max_ms":v[v.len()-1]})
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let scene_path = args.get(1).context("scene path required")?;
    let output = Path::new(args.get(2).context("output directory required")?);
    let width: u32 = args.get(3).map_or(Ok(1100), |s| s.parse())?;
    let height: u32 = args.get(4).map_or(Ok(760), |s| s.parse())?;
    fs::create_dir_all(output)?;
    let scene: Scene = serde_json::from_reader(BufReader::new(fs::File::open(scene_path)?))?;
    let mut e = Engine::new_with_mode(scene, ExecutionMode::Continuous)?;
    e.execute(
        Command::SetPlayerView {
            view: Some(ignis::player_view::Config {
                actor: "player".into(),
                yaw_degrees: 0.,
                pitch_degrees: -3.,
                eye_height: 1.65,
                fov_degrees: 65.,
                mode: ignis::player_view::Mode::FirstPerson,
                third_person: Default::default(),
            }),
        }
        .into(),
    )?;
    e.advance(60)?;
    // Obtaining a Replay expands every authored instance pose; a cost probe
    // needs only link provenance, never that population-sized snapshot.
    let provenance: serde_json::Value = serde_json::from_slice(&std::fs::read(
        "artifacts/horizon-meadow/batch-link-provenance.json",
    )?)?;
    let build = provenance["library_sha256"].clone();
    let mut frame = e.render_frame();
    let source = e.scene.meshes.clone();
    let mut sorted = source.clone();
    // This is a NEW authored order for a candidate scene, not an undocumented
    // renderer reordering policy. All per-ID records and geometry stay identical.
    sorted.instances.sort_by(|a, b| a.asset.cmp(&b.asset));
    let mut before: Vec<_> = source.instances.iter().collect();
    let mut after: Vec<_> = sorted.instances.iter().collect();
    before.sort_by_key(|i| &i.id);
    after.sort_by_key(|i| &i.id);
    ensure!(before == after, "ordering changed instance data");
    drop((before, after));
    let views: Value =
        serde_json::from_reader(fs::File::open("artifacts/botanical-valley/views.json")?)?;
    let cameras: Vec<(&str, Camera)> = vec![
        ("spawn", frame.camera.clone()),
        ("valley", serde_json::from_value(views["valley"].clone())?),
        (
            "panorama",
            serde_json::from_value(views["panorama"].clone())?,
        ),
    ];
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&instance, None))?;
    let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("vegetation batching probe"),
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
    let target = texture.create_view(&Default::default());
    let mut rows = vec![];
    for (name, camera) in cameras {
        let mut reference = vec![];
        for (variant, doc, occlusion) in [
            ("original", &source, false),
            ("grouped", &sorted, false),
            ("grouped_occlusion", &sorted, true),
        ] {
            frame.meshes.update(doc, 0);
            frame.camera = camera.clone();
            renderer.set_occlusion_culling(occlusion);
            for _ in 0..3 {
                renderer.draw(&frame, &target, width, height)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
            }
            let mut submit = vec![];
            let mut total = vec![];
            for _ in 0..8 {
                let t = Instant::now();
                renderer.draw(&frame, &target, width, height)?;
                submit.push(t.elapsed().as_secs_f64() * 1000.);
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                total.push(t.elapsed().as_secs_f64() * 1000.);
            }
            let bytes = renderer.capture(&frame, width, height)?;
            let pixels = ignis::image_similarity::Image::decode(&bytes)?.pixels;
            if reference.is_empty() {
                reference = pixels.clone();
            }
            let differing = reference
                .iter()
                .zip(&pixels)
                .filter(|(a, b)| a != b)
                .count();
            let maximum = reference
                .iter()
                .zip(&pixels)
                .flat_map(|(a, b)| a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)))
                .max()
                .unwrap_or(0);
            fs::write(output.join(format!("{name}-{variant}.png")), bytes)?;
            let row = json!({"camera":name,"variant":variant,"width":width,"height":height,
                "cpu_submit":stats(&mut submit),"draw_wait":stats(&mut total),"maximum_rgb8":maximum,
                "differing_pixels":differing,"renderer":renderer.info()});
            println!(
                "{}",
                json!({"camera":name,"variant":variant,"draw_wait":row["draw_wait"],"different":differing,"max_rgb8":maximum,"draws":row["renderer"]["instancing"]["draw_calls"],"occlusion":row["renderer"]["occlusion"]["active"]})
            );
            rows.push(row);
            fs::write(
                output.join("batch-probe.json"),
                serde_json::to_vec_pretty(
                    &json!({"library_sha256":build,"scene":scene_path,"instances":source.instances.len(),"same_instance_records":true,"method":"warm serial draw plus completion; CPU/GPU overlap not separated; not presentation FPS","rows":rows}),
                )?,
            )?;
        }
    }
    Ok(())
}
