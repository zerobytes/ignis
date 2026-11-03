//! Offscreen experiment; projection is applied only to transient RenderFrame.
//! Usage: cargo run --release --example triplanar_probe -- input.scene.json output-dir
use anyhow::{Context, Result, ensure};
use ignis::{
    engine::Engine,
    materials::{Material, Triplanar},
    meshes::Frame,
    model::Scene,
    render::{RenderFrame, Renderer},
};
use std::{fs, path::Path, sync::Arc};

fn projected(materials: &[Material], sharpness: f32, color: bool, bump: bool) -> Vec<Material> {
    materials
        .iter()
        .cloned()
        .map(|mut m| {
            if let Some(d) = m.detail.as_mut() {
                let d = Arc::make_mut(d);
                d.triplanar = Some(Triplanar { sharpness });
                if !color {
                    d.opacity = 0.;
                }
                if !bump {
                    d.bump = 0.;
                }
            }
            m
        })
        .collect()
}
fn capture(renderer: &mut Renderer, f: &RenderFrame, out: &Path, name: &str) -> Result<Vec<u8>> {
    let png = renderer.capture(f, 800, 800)?;
    fs::write(out.join(format!("{name}.png")), &png)?;
    Ok(png)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(args.len() == 2, "Expected input scene and output directory");
    let scene: Scene = serde_json::from_slice(&fs::read(&args[0])?)?;
    let engine = Engine::new(scene)?;
    let authored = engine.scene.clone();
    let out = Path::new(&args[1]);
    fs::create_dir_all(out)?;
    let mut f = engine.render_frame();
    ensure!(
        f.meshes.materials.iter().any(|m| m.detail.is_some()),
        "Input needs native surface detail"
    );
    let base = f.meshes.materials.clone();
    let instance = wgpu::Instance::new(&Default::default());
    let mut renderer = pollster::block_on(Renderer::new(&instance, None))?;
    let planar = capture(&mut renderer, &f, out, "planar")?;
    ensure!(capture(&mut renderer, &f, out, "planar-repeat")? == planar);
    f.meshes.materials = projected(&base, 2., true, true);
    let tri = capture(&mut renderer, &f, out, "triplanar-p2")?;
    ensure!(tri != planar, "Projection must visibly affect this fixture");
    ensure!(capture(&mut renderer, &f, out, "triplanar-repeat")? == tri);
    for p in [1., 4., 8.] {
        f.meshes.materials = projected(&base, p, true, true);
        capture(&mut renderer, &f, out, &format!("triplanar-p{p}"))?;
    }
    // Isolate strokes from the existing 3D pattern for independent ablations.
    let strokes: Vec<_> = base
        .iter()
        .cloned()
        .map(|mut m| {
            m.pattern = None;
            m
        })
        .collect();
    let mut ablations = Vec::new();
    for (name, color, bump) in [
        ("neutral", false, false),
        ("color", true, false),
        ("bump", false, true),
        ("combined", true, true),
    ] {
        f.meshes.materials = projected(&strokes, 2., color, bump);
        ablations.push(capture(&mut renderer, &f, out, name)?);
    }
    for (i, a) in ablations.iter().enumerate() {
        for b in &ablations[..i] {
            ensure!(
                a != b,
                "Color and bump ablations must independently affect pixels"
            );
        }
    }
    f.meshes.materials = base.clone();
    ensure!(
        capture(&mut renderer, &f, out, "planar-restored")? == planar,
        "Material toggles must invalidate the GPU cache"
    );
    let camera = f.camera.clone();
    let target = glam::Vec3::from(camera.target);
    let offset = glam::Vec3::from(camera.position) - target;
    f.meshes.materials = projected(&base, 2., true, true);
    for (name, angle) in [
        ("side", std::f32::consts::FRAC_PI_2),
        ("back", std::f32::consts::PI),
        ("opposite", 3. * std::f32::consts::FRAC_PI_2),
    ] {
        f.camera.position = (target + glam::Quat::from_rotation_y(angle) * offset).to_array();
        capture(&mut renderer, &f, out, name)?;
    }
    f.camera = camera.clone();
    renderer.capture(&f, 640, 480)?;
    ensure!(capture(&mut renderer, &f, out, "viewport-restored")? == tri);

    // Identity skin must preserve pixels; actual animated translation must move
    // the surface while keeping material coordinates and weights attached.
    let mut doc = authored.meshes.clone();
    for a in &mut doc.assets {
        a.rig = Some(Arc::new(ignis::meshes::Rig {
            bones: vec![ignis::meshes::Bone {
                name: "root".into(),
                parent: None,
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            }],
            weights: vec![
                ignis::meshes::Weight {
                    extra: None,
                    joints: [0; 4],
                    weights: [1., 0., 0., 0.]
                };
                a.geometry.positions.len()
            ],
        }));
        a.clips = vec![Arc::new(ignis::meshes::Clip {
            id: "pose".into(),
            duration_ticks: 60,
            tracks: vec![ignis::meshes::Track {
                bone: 0,
                keys: vec![
                    ignis::meshes::Keyframe {
                        tick: 0,
                        translation: [0.; 3],
                        rotation: [0., 0., 0., 1.],
                    },
                    ignis::meshes::Keyframe {
                        tick: 60,
                        translation: [0.; 3],
                        rotation: [0., 0., 0., 1.],
                    },
                ],
            }],
        })];
    }
    for i in &mut doc.instances {
        i.playback = Some(ignis::meshes::Playback {
            clip: "pose".into(),
            offset_ticks: 0,
            looping: true,
        });
    }
    let mut posed = Frame::default();
    posed.update(&doc, 30);
    ensure!(
        bytemuck::cast_slice::<_, u8>(&posed.vertices)
            == bytemuck::cast_slice::<_, u8>(&f.meshes.vertices)
    );
    posed.materials = projected(&base, 2., true, true);
    f.meshes = posed;
    ensure!(capture(&mut renderer, &f, out, "identity-skin")? == tri);

    let rest_weights: Vec<_> = f
        .meshes
        .vertices
        .iter()
        .map(|v| (v.rest_position, v.material_projection_weights))
        .collect();
    for a in &mut doc.assets {
        Arc::make_mut(&mut a.clips[0]).tracks[0].keys[1].translation = [0., 0.25, 0.];
    }
    f.meshes.update(&doc, 30);
    ensure!(
        rest_weights
            == f.meshes
                .vertices
                .iter()
                .map(|v| (v.rest_position, v.material_projection_weights))
                .collect::<Vec<_>>()
    );
    f.meshes.materials = projected(&base, 2., true, true);
    ensure!(capture(&mut renderer, &f, out, "animated")? != tri);
    f.meshes.update(&doc, 0);
    f.meshes.materials = projected(&base, 2., true, true);
    ensure!(capture(&mut renderer, &f, out, "animation-reset")? == tri);

    // Rank-deficient material domains are legal but cannot carry full surface
    // information. They must remain deterministic and cannot generate NaNs in
    // the CPU attributes. Uniform fallback is explicitly only a fallback.
    for rank in [0, 1] {
        let mut degenerate = authored.meshes.clone();
        for a in &mut degenerate.assets {
            Arc::make_mut(&mut a.geometry).material_coordinates = Some(
                a.geometry
                    .positions
                    .iter()
                    .map(|p| if rank == 0 { [0.; 3] } else { [p[0], 0., 0.] })
                    .collect(),
            );
        }
        f.meshes.update(&degenerate, 0);
        ensure!(
            f.meshes
                .vertices
                .iter()
                .all(|v| v.material_projection_weights == [1. / 3.; 3])
        );
        f.meshes.materials = projected(&base, 2., true, true);
        let image = capture(&mut renderer, &f, out, &format!("rank{rank}"))?;
        ensure!(renderer.capture(&f, 800, 800)? == image);
    }
    // Material-space orientation must follow M, not the posed geometric normal.
    let mut rotated = authored.meshes.clone();
    for a in &mut rotated.assets {
        let g = Arc::make_mut(&mut a.geometry);
        g.material_coordinates = Some(
            g.material_coordinates
                .as_deref()
                .unwrap_or(&g.positions)
                .iter()
                .map(|p| [p[2], p[1], -p[0]])
                .collect(),
        );
    }
    f.meshes.update(&rotated, 0);
    f.meshes.materials = projected(&base, 2., true, true);
    let rotation = capture(&mut renderer, &f, out, "rotated-material")?;
    ensure!(rotation != tri);
    f.meshes.update(&authored.meshes, 0);
    f.meshes.materials = projected(&base, 2., true, true);
    ensure!(capture(&mut renderer, &f, out, "domain-restored")? == tri);
    ensure!(
        engine.scene == authored,
        "Transient GPU proof must preserve authoring"
    );
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "passed":true,"native_window":false,"transient_frame_only":true,
        "scene_version":ignis::model::SCENE_VERSION,"vertex_bytes":std::mem::size_of::<ignis::render::Vertex>(),
        "checks":["repeat and planar toggle", "color/bump independent ablations", "sharpness 1/2/4/8 and four views",
            "viewport restore", "identity skin preserves attributes and pixels; animated translation keeps M and weights", "rank 0/1 fallback",
            "rotated material frame and domain restore", "authoring unchanged"]
    }))?).context("write report")?;
    println!("Offscreen projection proof passed: {}", out.display());
    Ok(())
}
