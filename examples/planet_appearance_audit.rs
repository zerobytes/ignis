//! Native appearance audit; no editor, window changes or runtime tuning.
use anyhow::Result;
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::Scene,
    planet::runtime::Config,
    render::Renderer,
};
use serde_json::json;
use std::fs;

fn shortcuts(r: &mut Renderer, enabled: bool) {
    r.set_visibility_culling(enabled);
    r.set_object_culling(enabled);
    r.set_occlusion_culling(enabled);
    r.set_geometry_indexing(enabled);
    r.set_compact_vertices(enabled);
    r.set_minimal_vertices(enabled);
}
fn pixels(bytes: &[u8]) -> Vec<[u8; 3]> {
    ignis::image_similarity::Image::decode(bytes)
        .unwrap()
        .pixels
}
fn main() -> Result<()> {
    let out = "artifacts/planet-appearance";
    fs::create_dir_all(out)?;
    let scene: Scene =
        serde_json::from_slice(&fs::read("artifacts/planet-runtime/player.scene.json")?)?;
    let config_bytes = fs::read("artifacts/planet-runtime/world.json")?;
    let config: Config = serde_json::from_slice(&config_bytes)?;
    let source_id = config.document.id.clone();
    let config_hash: String = ring::digest::digest(&ring::digest::SHA256, &config_bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let mut e = Engine::new_with_mode(scene.clone(), ExecutionMode::Continuous)?;
    e.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(config.clone())),
        }
        .into(),
    )?;
    e.advance(120)?;
    let mut neutral = Engine::new_with_mode(scene, ExecutionMode::Continuous)?;
    let mut plain = config;
    plain.material = Default::default();
    neutral.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(plain)),
        }
        .into(),
    )?;
    neutral.advance(120)?;
    let gpu = wgpu::Instance::default();
    let mut r = pollster::block_on(Renderer::new(&gpu, None))?;
    let mut rows = vec![];
    for (id, yaw, pitch) in [("valley", 0., 0.), ("west", 270., 12.), ("east", 90., 12.)] {
        let view = serde_json::from_value(
            json!({"actor":"player","yaw_degrees":yaw,"pitch_degrees":pitch,
            "eye_height":1.65,"fov_degrees":60.,"mode":"first_person"}),
        )?;
        e.execute(Command::SetPlayerView { view: Some(view) }.into())?;
        let mut f = e.render_frame();
        f.light.atmosphere.as_mut().unwrap().reduce_details = false;
        shortcuts(&mut r, false);
        let reference = r.capture(&f, 1280, 800)?;
        shortcuts(&mut r, true);
        let lossless = r.capture(&f, 1280, 800)?;
        let exact = pixels(&reference) == pixels(&lossless);
        let optimized_info = r.info();
        f.light.atmosphere.as_mut().unwrap().reduce_details = true;
        let combined = r.capture(&f, 1280, 800)?;
        let info = r.info();
        let a = pixels(&reference);
        let b = pixels(&combined);
        let mut max = 0;
        let mut sum = 0u64;
        let mut changed = 0;
        for (a, b) in a.iter().flatten().zip(b.iter().flatten()) {
            let d = a.abs_diff(*b);
            max = max.max(d);
            sum += u64::from(d);
            changed += usize::from(d != 0);
        }
        let mut nf = neutral.render_frame();
        nf.camera = f.camera.clone();
        nf.light.atmosphere.as_mut().unwrap().reduce_details = false;
        let plain = r.capture(&nf, 1280, 800)?;
        let neutral_info = r.info();
        for (suffix, png) in [
            ("full", reference),
            ("lossless", lossless),
            ("combined", combined),
            ("neutral", plain),
        ] {
            fs::write(format!("{out}/{id}-{suffix}.png"), png)?;
        }
        println!("{id}: lossless exact={exact}; ATM maxRGB8={max}, changed_channels={changed}");
        rows.push(
            json!({"pose":id,"lossless_exact":exact,"atmospheric_max_rgb8":max,
            "atmospheric_mean_rgb8":sum as f64/(1280.*800.*3.),"changed_channels":changed,
            "lossless_info":optimized_info,"combined_info":info,"neutral_info":neutral_info}),
        );
    }
    fs::write(
        format!("{out}/audit.json"),
        serde_json::to_vec_pretty(&json!({"build_id":env!("IGNIS_BUILD_ID"),
        "source_id":source_id,"config_sha256":config_hash,"config_path":"artifacts/planet-runtime/world.json",
        "rows":rows,"live_window_changed":false,"performance_claim":false}))?,
    )?;
    anyhow::ensure!(
        rows.iter().all(|r| r["lossless_exact"] == true),
        "lossless rendering changed pixels"
    );
    Ok(())
}
