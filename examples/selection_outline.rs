//! Native coverage probe: a perforated selected mesh, occlusion and moving camera.
use anyhow::{Result, ensure};
use ignis::{engine::Engine, model::RoomRecipe, render::Renderer};
use serde_json::json;
use std::{fs, io::Cursor};

fn pixels(png: &[u8]) -> Vec<u8> {
    let mut reader = png::Decoder::new(Cursor::new(png)).read_info().unwrap();
    let mut data = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut data).unwrap();
    data.truncate(info.buffer_size());
    data
}

fn main() -> Result<()> {
    let root = "artifacts/selection-outline";
    fs::create_dir_all(root)?;
    let mut scene = RoomRecipe::default().generate()?;
    for e in &mut scene.entities {
        e.visible = Some(false);
    }
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    for i in 0..64 {
        let a = i as f32 * std::f32::consts::TAU / 64.;
        for radius in [2.5, 1.2] {
            positions.push([radius * a.cos(), 3. + radius * a.sin(), 0.]);
        }
        let (n, m) = (i * 2, ((i + 1) % 64) * 2);
        triangles.extend([[n, m, m + 1], [n, m + 1, n + 1]]);
    }
    scene.meshes = serde_json::from_value(
        json!({"assets":[{"id":"ring","geometry":{"positions":positions,"triangles":triangles}}],
        "instances":[{"id":"ring","asset":"ring","position":[0,0,0],"rotation":[0,0,0,1],"scale":1,"color":[0.04,0.35,0.15]}]}),
    )?;
    scene.camera.position = [0., 3., 10.];
    scene.camera.target = [0., 3., 0.];
    scene.camera.fov_degrees = 50.;
    scene.light.ambient = 1.;
    scene.light.atmosphere = None;
    scene.validate()?;
    fs::write(
        format!("{root}/ring.scene.json"),
        serde_json::to_vec(&scene)?,
    )?;
    let engine = Engine::new(scene)?;
    let mut frame = engine.render_frame();
    frame.selection = None;
    frame.boxes.clear();
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&instance, None))?;
    let mut rows = Vec::new();
    for (label, eye, occlusion) in [
        ("ring", [0., 3., 10.], false),
        ("occluded", [0., 3., 10.], true),
        ("moved", [3., 4., 9.], true),
    ] {
        frame.camera.position = eye;
        frame.boxes.clear();
        if occlusion {
            frame.boxes.push(ignis::render::RenderBox {
                geometry: Default::default(),
                position: [-1.5, 3., 2.],
                size: [1.3, 5., 1.],
                color: [0.15, 0.15, 0.15],
            });
        }
        frame.selection_mesh = None;
        let plain = renderer.capture(&frame, 800, 600)?;
        frame.selection_mesh = Some("ring".into());
        let selected = renderer.capture(&frame, 800, 600)?;
        fs::write(format!("{root}/{label}-plain.png"), &plain)?;
        fs::write(format!("{root}/{label}-selected.png"), &selected)?;
        let (p, q) = (pixels(&plain), pixels(&selected));
        let mut changed = 0;
        // The outline must only replace visible green surface, never the hole,
        // background or the nearer grey occluder. This is an integration invariant.
        for (a, b) in p.chunks_exact(4).zip(q.chunks_exact(4)) {
            if a != b {
                changed += 1;
                ensure!(
                    a[1] > a[0] + 20 && a[1] > a[2],
                    "outline escaped visible selected surface"
                );
            }
        }
        ensure!(changed > 300, "missing silhouette");
        rows.push(json!({"view":label,"changed_pixels":changed,"stats":renderer.info()["selection_outline"]}));
        frame.selection_mesh = None;
        ensure!(
            renderer.capture(&frame, 800, 600)? == plain,
            "clearing selection left stale pixels"
        );
        // The optional atmospheric finish runs before the selection overlay.
        // Even when the ring itself is softened, its selection mask stays sharp.
        frame.light.atmosphere = Some(ignis::atmosphere::Atmosphere {
            density: 0.,
            edge_softness: Some(ignis::atmosphere::EdgeSoftness {
                strength: 1.,
                radius_px: 12,
                start: 0.,
                end: 1.,
            }),
            ..Default::default()
        });
        let soft_plain = renderer.capture(&frame, 800, 600)?;
        frame.selection_mesh = Some("ring".into());
        let soft_selected = renderer.capture(&frame, 800, 600)?;
        let (sp, sq) = (pixels(&soft_plain), pixels(&soft_selected));
        for ((a, b), (c, d)) in p
            .chunks_exact(4)
            .zip(q.chunks_exact(4))
            .zip(sp.chunks_exact(4).zip(sq.chunks_exact(4)))
        {
            ensure!(
                (a != b) == (c != d),
                "edge finish changed selection coverage"
            );
            if a != b {
                ensure!(b == d, "edge finish blurred the selection overlay");
            }
        }
        fs::write(format!("{root}/{label}-soft-selected.png"), soft_selected)?;
        frame.light.atmosphere = None;
        frame.selection_mesh = None;
        ensure!(
            renderer.capture(&frame, 800, 600)? == plain,
            "edge disable did not restore the source image"
        );
    }
    fs::write(
        format!("{root}/verification.json"),
        serde_json::to_vec_pretty(
            &json!({"build_id":env!("IGNIS_BUILD_ID"),"passed":true,"views":rows}),
        )?,
    )?;
    Ok(())
}
