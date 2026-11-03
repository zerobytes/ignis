//! Compile and measure a dense native field. The original botanical scene is immutable.
use anyhow::{Context, Result, ensure};
use glam::Vec3;
use ignis::{
    botany, meshes,
    model::Scene,
    surface_field::{self, ABSENT, Level, Pack, SIDE, SampleVertex},
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::{BufReader, BufWriter},
    path::Path,
    sync::Arc,
    time::Instant,
};

fn ribbon(
    compiled: &botany::bake::Compiled,
    normals: &[Vec3],
    segments: usize,
    across: usize,
) -> Result<Vec<SampleVertex>> {
    let g = &compiled.document.assets[1].geometry;
    let addresses: BTreeMap<_, _> = compiled.vertices[1]
        .iter()
        .enumerate()
        .map(|(i, v)| (v.key.as_str(), i))
        .collect();
    let recipe: botany::Recipe = serde_json::from_reader(fs::File::open(
        "assets/botany/groundcover/grama_baixa_0-green.ignis-botany",
    )?)?;
    let mut result = vec![];
    for organ in &recipe.source.organs {
        let point = |u: f64, v: f64| -> Result<(Vec3, Vec3)> {
            let v = if u == 0. || u == 1. { 0. } else { v };
            let key = format!(
                "{}:leaf:0:{:016x}:{:016x}",
                organ.id,
                u.to_bits(),
                v.to_bits()
            );
            let index = addresses
                .get(key.as_str())
                .with_context(|| format!("Missing native blade station {key}"))?;
            Ok((
                Vec3::from(g.positions[*index]),
                normals[*index].normalize_or_zero(),
            ))
        };
        for u in 0..segments {
            for v in 0..across {
                let a = point(
                    u as f64 / segments as f64,
                    -1. + 2. * v as f64 / across as f64,
                )?;
                let b = point(
                    (u + 1) as f64 / segments as f64,
                    -1. + 2. * v as f64 / across as f64,
                )?;
                let c = point(
                    (u + 1) as f64 / segments as f64,
                    -1. + 2. * (v + 1) as f64 / across as f64,
                )?;
                let d = point(
                    u as f64 / segments as f64,
                    -1. + 2. * (v + 1) as f64 / across as f64,
                )?;
                for p in [[a, b, c], [a, c, d]] {
                    let n = (p[1].0 - p[0].0).cross(p[2].0 - p[0].0);
                    if n.length_squared() < 1e-20 {
                        continue;
                    }
                    result.extend(p.map(|p| SampleVertex {
                        position: [p.0.x, p.0.y, p.0.z, 0.],
                        normal: [p.1.x, p.1.y, p.1.z, 0.],
                    }));
                }
            }
        }
    }
    Ok(result)
}
fn terrain(scene: &Scene) -> Result<&meshes::Geometry> {
    Ok(&scene
        .meshes
        .assets
        .iter()
        .find(|a| a.id == "valley_ground")
        .context("Source terrain missing")?
        .geometry)
}
fn sample(g: &meshes::Geometry, x: f32, z: f32) -> f32 {
    let x = (x + 500.) / 2.;
    let z = (z + 500.) / 2.;
    let i = (x.floor() as usize).min(499);
    let j = (z.floor() as usize).min(499);
    let u = x - i as f32;
    let v = z - j as f32;
    let a = g.positions[j * 501 + i][1];
    let b = g.positions[j * 501 + i + 1][1];
    let c = g.positions[(j + 1) * 501 + i][1];
    let d = g.positions[(j + 1) * 501 + i + 1][1];
    if u + v <= 1. {
        a + (b - a) * u + (c - a) * v
    } else {
        d + (c - d) * (1. - u) + (b - d) * (1. - v)
    }
}
fn compile(out: &Path) -> Result<()> {
    ensure!(
        SIDE == 16,
        "This field recipe requires the current 16x16 block compiler"
    );
    fs::create_dir_all(out)?;
    let original: Scene = serde_json::from_reader(BufReader::new(fs::File::open(
        "artifacts/botanical-valley/world.scene.json",
    )?))?;
    let g = terrain(&original)?;
    ensure!(
        g.positions.len() == 501 * 501 && g.triangles.len() == 500_000,
        "This recipe requires the verified 1km,2m source grid"
    );
    ensure!(
        g.positions
            .first()
            .is_some_and(|p| p[0] == 0. && p[2] == 0.)
            && g.positions
                .last()
                .is_some_and(|p| p[0] == 1000. && p[2] == 1000.),
        "Terrain source coordinates changed"
    );
    let placement = original
        .meshes
        .instances
        .iter()
        .find(|i| i.id == "valley_ground")
        .context("Missing terrain placement")?;
    ensure!(
        placement.position == [-500., 0., -500.]
            && placement.rotation == [0., 0., 0., 1.]
            && placement.scale == 1.,
        "Terrain placement changed"
    );
    let recipe: botany::Recipe = serde_json::from_reader(fs::File::open(
        "assets/botany/groundcover/grama_baixa_0-green.ignis-botany",
    )?)?;
    let compiled = botany::bake::compile(&recipe)?;
    let geometry = &compiled.document.assets[1].geometry;
    let mut normals = vec![Vec3::ZERO; geometry.positions.len()];
    for t in &geometry.triangles {
        let p = t.map(|i| Vec3::from(geometry.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for i in t {
            normals[*i as usize] += n;
        }
    }
    let full = geometry
        .triangles
        .iter()
        .flat_map(|t| {
            t.map(|i| {
                let p = geometry.positions[i as usize];
                let n = normals[i as usize].normalize_or_zero().to_array();
                SampleVertex {
                    position: [p[0], p[1], p[2], 0.],
                    normal: [n[0], n[1], n[2], 0.],
                }
            })
        })
        .collect();
    let medium = ribbon(&compiled, &normals, 4, 2)?;
    let small = ribbon(&compiled, &normals, 4, 1)?;
    let tiny = ribbon(&compiled, &normals, 2, 1)?;
    let levels = [
        Level {
            vertices: full,
            stride: 1,
        },
        Level {
            vertices: medium,
            stride: 1,
        },
        Level {
            vertices: small,
            stride: 2,
        },
        Level {
            vertices: tiny.clone(),
            stride: 4,
        },
        Level {
            vertices: tiny.clone(),
            stride: 8,
        },
        Level {
            vertices: tiny,
            stride: 16,
        },
    ];
    let mut pack = Pack {
        origin: [-500., -500.],
        spacing: 0.25,
        tiles: [250, 250],
        heights: Vec::with_capacity(16_000_000),
        levels,
        color: recipe.settings.leaf_color,
    };
    let mut occupied = [0usize; 400];
    let mut available = [0usize; 400];
    for tz in 0..250 {
        for tx in 0..250 {
            for z in 0..SIDE {
                for x in 0..SIDE {
                    let index = pack.heights.len() as u32;
                    let jitter = surface_field::jitter(index);
                    let wx = -500.
                        + (tx as f32 * SIDE as f32 + x as f32 + 0.5 + jitter[0] * 0.9)
                            * pack.spacing;
                    let wz = -500.
                        + (tz as f32 * SIDE as f32 + z as f32 + 0.5 + jitter[1] * 0.9)
                            * pack.spacing;
                    let height = sample(g, wx, wz);
                    let path = -94. + 22. * (wz / 85.).sin() + 12. * (wz / 31.).sin();
                    let cell = ((wz + 500.) / 50.) as usize * 20 + ((wx + 500.) / 50.) as usize;
                    available[cell] += 1;
                    let y = if height < 5.45 || (wx - path).abs() < 2.2 {
                        ABSENT
                    } else {
                        occupied[cell] += 1;
                        height - 0.025
                    };
                    pack.heights.push(y);
                }
            }
        }
    }
    let file = out.join("ground.ignis-field");
    let temporary = out.join("ground.ignis-field.candidate");
    pack.write(BufWriter::new(fs::File::create(&temporary)?))?;
    fs::rename(temporary, &file)?;
    let population = pack.heights.iter().filter(|h| **h != ABSENT).count();
    // Preserve actual canonical trees, shrubs and flowering components. Only the
    // low ground layer is replaced by the explicit experimental field resource.
    let mut scene = original;
    scene.name = "Horizon · prado contínuo de 1 km² (laboratório)".into();
    let kept: std::collections::BTreeSet<_> = scene
        .meshes
        .assets
        .iter()
        .filter(|a| !a.id.starts_with("gc_") && !a.id.starts_with("tg_"))
        .map(|a| a.id.clone())
        .collect();
    scene.meshes.instances.retain(|i| kept.contains(&i.asset));
    let ids: std::collections::BTreeSet<_> = scene
        .meshes
        .instances
        .iter()
        .map(|i| i.id.clone())
        .collect();
    scene.meshes.objects = Arc::new(
        scene
            .meshes
            .objects
            .iter()
            .filter(|o| o.members.iter().all(|m| ids.contains(m)))
            .cloned()
            .collect(),
    );
    scene.meshes.assets.retain(|a| kept.contains(&a.id));
    scene.meshes.instances.sort_by(|a, b| a.asset.cmp(&b.asset));
    scene.validate()?;
    let scene_path = out.join("world.scene.json");
    let candidate = out.join("world.scene.json.candidate");
    serde_json::to_writer(BufWriter::new(fs::File::create(&candidate)?), &scene)?;
    fs::rename(candidate, scene_path)?;
    let summary = json!({"size_meters":[1000,1000],"spacing_meters":pack.spacing,"sites":pack.heights.len(),"population":population,"blades_per_plant":recipe.source.organs.len(),"logical_blades":population*recipe.source.organs.len(),"native_retained_instances":scene.meshes.instances.len(),"field_file_bytes":fs::metadata(file)?.len(),"coverage_50m_cells":occupied.iter().zip(available).map(|(n,a)|*n as f64/a as f64).collect::<Vec<_>>(),"triangles_per_template":pack.levels.iter().map(|l|l.vertices.len()/3).collect::<Vec<_>>(),"limitations":["Experimental approximate mesh/density scales; no image-error certificate","Not a Scene field format; launch through explicit sidecar opt-in","Native terrain, trees and shrubs retained; source scene untouched"]});
    fs::write(
        out.join("compile.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    println!("{summary}");
    Ok(())
}
fn probe(out: &Path) -> Result<()> {
    let s: Scene = serde_json::from_reader(BufReader::new(fs::File::open(
        out.join("world.scene.json"),
    )?))?;
    let mut e = ignis::engine::Engine::new_with_mode(s, ignis::engine::ExecutionMode::Continuous)?;
    e.execute(
        ignis::engine::Command::SetPlayerView {
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
    let mut frame = e.render_frame();
    let views: serde_json::Value =
        serde_json::from_reader(fs::File::open("artifacts/botanical-valley/views.json")?)?;
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(ignis::render::Renderer::new(&instance, None))?;
    let pack = Arc::new(Pack::read(BufReader::new(fs::File::open(
        out.join("ground.ignis-field"),
    )?))?);
    let (width, height) = (1100, 760);
    let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
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
    let cameras = [
        ("walk", frame.camera.clone()),
        ("valley", serde_json::from_value(views["valley"].clone())?),
        (
            "panorama",
            serde_json::from_value(views["panorama"].clone())?,
        ),
    ];
    let mut rows = vec![];
    for (name, camera) in cameras {
        frame.camera = camera;
        for enabled in [false, true] {
            renderer.set_surface_field(enabled.then(|| pack.clone()))?;
            let variant = if enabled {
                "field"
            } else {
                "native_only_ablation"
            };
            for _ in 0..3 {
                renderer.draw(&frame, &target, width, height)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
            }
            let mut times = vec![];
            for _ in 0..8 {
                let t = Instant::now();
                renderer.draw(&frame, &target, width, height)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                times.push(t.elapsed().as_secs_f64() * 1000.);
            }
            times.sort_by(f64::total_cmp);
            let image = renderer.capture(&frame, width, height)?;
            fs::write(out.join(format!("meadow-{name}-{variant}.png")), image)?;
            let row = json!({"camera":name,"variant":variant,"draw_wait_median_ms":times[times.len()/2],"field":renderer.surface_field_info(),"native":renderer.info()});
            println!(
                "{}",
                json!({"camera":name,"variant":variant,"draw_wait_median_ms":row["draw_wait_median_ms"],"field":row["field"]})
            );
            rows.push(row);
        }
    }
    fs::write(
        out.join("field-probe.json"),
        serde_json::to_vec_pretty(
            &json!({"build_id":e.replay().build_id,"method":"warm draw plus completion; not presentation FPS","width":width,"height":height,"rows":rows}),
        )?,
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "probe".into());
    let out = Path::new("artifacts/horizon-meadow");
    match mode.as_str() {
        "compile" => compile(out),
        "probe" => probe(out),
        _ => anyhow::bail!("Expected compile or probe"),
    }
}
