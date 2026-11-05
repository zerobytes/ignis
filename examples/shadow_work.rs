//! Exact-image/native-only test of zero-contribution shadow query rejection.
use anyhow::{Result, ensure};
use ignis::{
    engine::Engine,
    environment::Environment,
    materials::{BlendMode, Coverage, Glass, Material, Pattern, PatternKind, Reflectance, Surface},
    model::{Camera, DirectionalShadow, RoomRecipe, Scene},
    render::Renderer,
};
use serde_json::json;
use std::{fs, time::Instant};
const W: u32 = 1280;
const H: u32 = 832;
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.
}
fn scene() -> Result<Scene> {
    let mut s = RoomRecipe::default().generate()?;
    let original = s.entities[0].clone();
    s.entities.clear();
    s.recipe = None;
    s.name = "Native shadow-work audit".into();
    s.camera = Camera {
        position: [7., 7., 12.],
        target: [0., 1., -0.5],
        fov_degrees: 43.,
    };
    s.light.direction = [-0.5, -0.82, -0.35];
    s.light.ambient = 0.24;
    s.light.direct_intensity = 1.1;
    s.light.direct_color = [1., 0.93, 0.84];
    s.light.environment = None;
    s.light.atmosphere = None;
    s.light.shadow = Some(DirectionalShadow {
        center: [0., 1., 0.],
        extent: 12.,
        resolution: 2048,
        bias: 0.0005,
        receiver_plane_bias: false,
    });
    for (j, (p, size, color)) in [
        ([0., -0.2, 0.], [10., 0.4, 10.], [0.65; 3]),
        ([0., 2., -4.], [8., 4., 0.25], [0.03, 0.10, 0.75]),
        ([-4., 2., 0.], [0.25, 4., 8.], [0.75, 0.06, 0.03]),
        ([3.5, 1.5, -2.], [0.25, 3., 4.], [0.65; 3]),
        ([0., 2.8, -1.2], [5., 0.3, 3.5], [0.6; 3]),
        ([-2., 1.3, -1.], [0.5, 2.6, 0.5], [0.5; 3]),
        ([2., 1.3, -1.], [0.5, 2.6, 0.5], [0.5; 3]),
        ([1.2, 0.45, 2.], [1.2, 0.9, 1.], [0.6; 3]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut e = original.clone();
        e.id = format!("native_{j}");
        e.body = None;
        e.transform.position = p;
        e.transform.size = size;
        e.color = color;
        s.entities.push(e);
    }
    Ok(s)
}
fn mesh_boxes(s: &mut Scene, material: &Material) {
    let entities = std::mem::take(&mut s.entities);
    for (j, e) in entities.into_iter().enumerate() {
        let h = e.transform.size.map(|x| x * 0.5);
        let positions = vec![
            [-h[0], -h[1], -h[2]],
            [h[0], -h[1], -h[2]],
            [h[0], h[1], -h[2]],
            [-h[0], h[1], -h[2]],
            [-h[0], -h[1], h[2]],
            [h[0], -h[1], h[2]],
            [h[0], h[1], h[2]],
            [-h[0], h[1], h[2]],
        ];
        let geometry = ignis::meshes::Geometry {
            positions,
            triangles: vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
                [0, 1, 5],
                [0, 5, 4],
                [3, 7, 6],
                [3, 6, 2],
            ],
            material_coordinates: None,
        };
        let id = format!("box-{j}");
        s.meshes.assets.push(ignis::meshes::Asset {
            id: id.clone(),
            geometry: std::sync::Arc::new(geometry),
            shading: ignis::meshes::Shading::Flat,
            material: material.clone(),
            rig: None,
            clips: vec![],
        });
        s.meshes.instances.push(ignis::meshes::Instance {
            id: format!("mesh-{j}"),
            asset: id,
            position: e.transform.position,
            rotation: [0., 0., 0., 1.],
            scale: 1.,
            color: e.color,
            playback: None,
            attachment: None,
        });
    }
}
fn prepared_frame(engine: &Engine, shared: bool) -> ignis::render::RenderFrame {
    let mut f = engine.render_frame();
    if !shared && !engine.scene.meshes.assets.is_empty() {
        f.meshes.set_unique_static_residency(false);
        engine.update_render_frame(&mut f);
    }
    f.suppress_diagnostic_grid = true;
    f
}
fn main() -> Result<()> {
    ensure!(
        std::env::args().len() == 1,
        "No shadow-work arguments supported"
    );
    let out = "artifacts/shadow-work";
    fs::create_dir_all(out)?;
    let base = scene()?;
    let instance = wgpu::Instance::default();
    let mut original =
        pollster::block_on(Renderer::new_profiled_shadow_work(&instance, None, false))?;
    let mut optimized =
        pollster::block_on(Renderer::new_profiled_shadow_work(&instance, None, true))?;
    ensure!(
        original.adapter.get_info().name == optimized.adapter.get_info().name,
        "Adapter mismatch"
    );
    let texture = |r: &Renderer| {
        r.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow-work current target"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: r.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    };
    let a = texture(&original);
    let b = texture(&optimized);
    let av = a.create_view(&Default::default());
    let bv = b.create_view(&Default::default());
    let mut cases = vec![];
    for mode in [
        "matte",
        "bias",
        "dielectric",
        "metal",
        "transmission",
        "mask",
        "glass",
        "bump",
        "emissive-ibl",
        "zero-direct",
        "shadow-off",
        "volume-boundary",
        "shared",
        "shared-bump",
        "shared-transmission",
        "shared-mask",
        "shared-glass",
    ] {
        for pose in 0..3 {
            let mut s = base.clone();
            if pose == 1 {
                s.camera.position = [-7., 3.6, 8.9];
                s.camera.target = [-0.2, 0.5, -1.];
            }
            if pose == 2 {
                s.camera.position = [8., 3.5, -9.];
                s.camera.target = [0., 1., 0.];
            }
            if mode == "bias" || mode.ends_with("bump") || mode == "volume-boundary" {
                s.light.shadow.as_mut().unwrap().receiver_plane_bias = true;
            }
            if mode == "volume-boundary" {
                s.light.shadow.as_mut().unwrap().extent = 3.5;
            }
            if mode == "zero-direct" {
                s.light.direct_intensity = 0.;
            }
            if mode == "shadow-off" {
                s.light.shadow = None;
            }
            if mode == "emissive-ibl" {
                s.light.environment = Some(Environment {
                    sky: [0.2, 0.3, 0.4],
                    horizon: [0.3; 3],
                    ground: [0.04; 3],
                    axis: [0., 1., 0.],
                    intensity: 0.8,
                });
            }
            let mut m = Material::default();
            if mode.ends_with("dielectric") || mode.ends_with("metal") {
                m.reflectance = Some(Reflectance {
                    roughness: 0.35,
                    f0: 0.04,
                    metallic: if mode.ends_with("metal") { 0.85 } else { 0. },
                });
            }
            if mode.ends_with("transmission") {
                m.transmission = 0.7;
            }
            if mode.ends_with("mask") {
                m.surface = Some(Surface {
                    mode: BlendMode::Mask,
                    coverage: Some(Coverage::default()),
                    ..Default::default()
                });
                m.transmission = 0.5;
            }
            if mode.ends_with("glass") {
                m.surface = Some(Surface {
                    mode: BlendMode::Blend,
                    glass: Some(Glass::default()),
                    ..Default::default()
                });
            }
            if mode.ends_with("bump") {
                m.pattern = Some(Pattern {
                    frequency: [3.; 3],
                    low: [0.6; 3],
                    high: [1.; 3],
                    bump: 0.04,
                    warp: 0.2,
                    octaves: 2,
                    seed: 7,
                    contrast: 1.,
                    kind: PatternKind::Noise,
                });
            }
            if mode == "emissive-ibl" {
                m.surface = Some(Surface {
                    emissive: [0.02, 0.03, 0.04],
                    ..Default::default()
                });
            }
            m.validate()?;
            if mode.starts_with("shared") {
                let material = m.clone();
                s.entities.truncate(1);
                s.meshes.assets.push(ignis::meshes::Asset {
                    id: "panels".into(),
                    geometry: std::sync::Arc::new(ignis::meshes::Geometry::grid(8, 4.)?),
                    shading: ignis::meshes::Shading::Smooth,
                    material,
                    rig: None,
                    clips: vec![],
                });
                for (j, (position, rotation)) in [
                    ([0., 0.1, 0.], glam::Quat::IDENTITY),
                    (
                        [0., 3., 0.],
                        glam::Quat::from_rotation_z(std::f32::consts::PI),
                    ),
                    (
                        [-3., 2., 0.],
                        glam::Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                    ),
                    (
                        [3., 2., 0.],
                        glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                    ),
                    (
                        [0., 2., -3.],
                        glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    ),
                    (
                        [0., 2., 3.],
                        glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    ),
                ]
                .into_iter()
                .enumerate()
                {
                    s.meshes.instances.push(ignis::meshes::Instance {
                        id: format!("panel-{j}"),
                        asset: "panels".into(),
                        position,
                        rotation: rotation.to_array(),
                        scale: 1.,
                        color: [0.45, 0.65, 0.3],
                        playback: None,
                        attachment: None,
                    });
                }
            }
            if !mode.starts_with("shared") && m != Material::default() {
                mesh_boxes(&mut s, &m);
            }
            let engine = Engine::new(s)?;
            let mut frame = prepared_frame(&engine, mode.starts_with("shared"));
            frame.suppress_diagnostic_grid = true;
            for _ in 0..8 {
                original.draw(&frame, &av, W, H)?;
                original.read_gpu_timing()?;
                optimized.draw(&frame, &bv, W, H)?;
                optimized.read_gpu_timing()?;
            }
            original.draw(&frame, &av, W, H)?;
            optimized.draw(&frame, &bv, W, H)?;
            let old = original.read_texture_png(&a, W, H)?;
            let new = optimized.read_texture_png(&b, W, H)?;
            ensure!(old == new, "Shadow shortcut changed pixels: {mode}/{pose}");
            let reference_info = original.info();
            let candidate_info = optimized.info();
            for key in ["instancing", "transparency", "procedural_materials"] {
                ensure!(
                    reference_info[key] == candidate_info[key],
                    "Work/resources differ for {mode}/{pose}/{key}"
                );
            }
            if mode.ends_with("glass") {
                ensure!(
                    reference_info["transparency"]["active"] == true,
                    "Glass did not exercise transparent composition"
                );
            }
            if mode.starts_with("shared") {
                ensure!(
                    reference_info["instancing"]["instances"]
                        .as_u64()
                        .unwrap_or(0)
                        > 0,
                    "Shared path not exercised"
                );
            }
            let label = format!("{mode}-{pose}");
            fs::write(format!("{out}/{label}.png"), &new)?;
            let mut rows = vec![];
            for trial in 0..64 {
                for enabled in if trial % 2 == 0 {
                    [false, true]
                } else {
                    [true, false]
                } {
                    let (r, v) = if enabled {
                        (&mut optimized, &bv)
                    } else {
                        (&mut original, &av)
                    };
                    let t = Instant::now();
                    engine.update_render_frame(&mut frame);
                    frame.suppress_diagnostic_grid = true;
                    let prep = ms(t);
                    let t = Instant::now();
                    r.draw(&frame, v, W, H)?;
                    let cpu = ms(t);
                    let timing = r.read_gpu_timing()?.unwrap();
                    rows.push(
                        json!({"trial":trial,"enabled":enabled,"cpu_ms":prep+cpu,"timing":timing}),
                    );
                }
            }
            // Movement/changed sun uses the same image oracle, beyond static timing poses.
            for step in 0..4 {
                engine.update_render_frame(&mut frame);
                frame.suppress_diagnostic_grid = true;
                frame.camera.position[0] += step as f32 * 0.19;
                frame.camera.position[1] += step as f32 * 0.08;
                frame.light.direction[0] += step as f32 * 0.03;
                original.draw(&frame, &av, W, H)?;
                optimized.draw(&frame, &bv, W, H)?;
                ensure!(
                    original.read_texture_png(&a, W, H)? == optimized.read_texture_png(&b, W, H)?,
                    "Moving shadow shortcut changed pixels: {label}/{step}"
                );
            }
            cases.push(json!({"label":label,"mode":mode,"pose":pose,"png_exact":true,"moving_pairs":4,"reference_info":reference_info,"candidate_info":candidate_info,"rows":rows}));
            println!("Passed {label}");
        }
    }
    fs::write(
        format!("{out}/results.json"),
        serde_json::to_vec_pretty(
            &json!({"width":W,"height":H,"adapter":original.adapter.get_info().name,"backend":format!("{:?}",original.adapter.get_info().backend),"added_gi":false,"cases":cases}),
        )?,
    )?;
    Ok(())
}
