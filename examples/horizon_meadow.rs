//! Compile and measure a dense native field. The original botanical scene is immutable.
use anyhow::{Context, Result, ensure};
use glam::{Quat, Vec3};
use ignis::{
    botany, meshes,
    model::Scene,
    surface_field::{self, ABSENT, Level, Pack, SIDE, SampleVertex},
};
use serde_json::json;
#[path = "../src/mesh_proxy/sanitize.rs"]
mod proxy_sanitize;
use std::{
    collections::BTreeMap,
    fs,
    io::{BufReader, BufWriter, Read},
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
        continuous: None,
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
    let mut writer = BufWriter::new(fs::File::create(&temporary)?);
    pack.write(&mut writer)?;
    std::io::Write::flush(&mut writer)?;
    writer.get_ref().sync_all()?;
    drop(writer);
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
    let mut writer = BufWriter::new(fs::File::create(&candidate)?);
    serde_json::to_writer(&mut writer, &scene)?;
    std::io::Write::flush(&mut writer)?;
    writer.get_ref().sync_all()?;
    drop(writer);
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
    let case = std::env::args().nth(4).unwrap_or_else(|| "sparse".into());
    ensure!(
        case.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "Invalid case label"
    );
    let scene_hash = file_hash(&out.join("world.scene.json"))?;
    let mut s: Scene = serde_json::from_reader(BufReader::new(fs::File::open(
        out.join("world.scene.json"),
    )?))?;
    if case == "sparse" || case == "verify" {
        s.meshes
            .instances
            .retain(|i| !i.id.starts_with("horizon_tree_"));
        s.meshes.objects = Arc::new(
            s.meshes
                .objects
                .iter()
                .filter(|o| !o.id.starts_with("horizon_tree_"))
                .cloned()
                .collect(),
        );
    }

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
    let width = std::env::args().nth(2).map_or(Ok(1100), |s| s.parse())?;
    let height = std::env::args().nth(3).map_or(Ok(760), |s| s.parse())?;
    let proxy_path = out.join("mesh.ignis-proxies");
    let proxies = if proxy_path.exists() {
        Some(Arc::new(ignis::mesh_proxy::Pack::decode(&fs::read(
            proxy_path,
        )?)?))
    } else {
        None
    };
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
    let mut cameras = vec![
        ("walk", frame.camera.clone()),
        ("valley", serde_json::from_value(views["valley"].clone())?),
        (
            "panorama",
            serde_json::from_value(views["panorama"].clone())?,
        ),
    ];
    if case == "verify" {
        let base = cameras[0].1.clone();
        cameras.clear();
        for n in 0..8 {
            let mut camera = base.clone();
            let shift = [
                (n as f32 * 0.7).sin() * 2.,
                (n as f32 * 0.4).cos() * 0.15,
                n as f32 * -1.3,
            ];
            for (axis, d) in shift.into_iter().enumerate() {
                camera.position[axis] += d;
                camera.target[axis] += d;
            }
            cameras.push((
                match n {
                    0 => "move0",
                    1 => "move1",
                    2 => "move2",
                    3 => "move3",
                    4 => "move4",
                    5 => "move5",
                    6 => "move6",
                    _ => "move7",
                },
                camera,
            ));
        }
    }
    let mut rows = vec![];
    for (ordinal, (name, camera)) in cameras.into_iter().enumerate() {
        if case == "verify" {
            frame.light.direction = [
                (ordinal as f32 * 0.8).sin(),
                -0.65,
                (ordinal as f32 * 0.8).cos(),
            ];
        }
        frame.camera = camera;
        let mut reference = None;
        let mut proxy_reference = None;
        for (variant, enabled, proxy, coordinates, indexed) in [
            ("native_only_ablation", false, false, false, false),
            ("field", true, false, false, false),
            ("field_proxy", true, true, false, false),
            ("field_proxy_coords", true, true, true, false),
            ("field_proxy_coords_indexed", true, true, true, true),
        ] {
            if (case == "dense" || case == "verify") && !proxy {
                continue;
            }
            if case == "verify" && coordinates && !indexed {
                continue;
            }
            if proxy && proxies.is_none() {
                continue;
            }
            renderer.set_reconstructed_coordinates(coordinates, indexed);
            renderer.set_surface_field(enabled.then(|| pack.clone()))?;
            renderer.set_mesh_proxy(if proxy { proxies.clone() } else { None });
            for _ in 0..if case == "verify" { 1 } else { 3 } {
                renderer.draw(&frame, &target, width, height)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
            }
            let mut times = vec![];
            for _ in 0..if case == "verify" { 2 } else { 8 } {
                let t = Instant::now();
                renderer.draw(&frame, &target, width, height)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                times.push(t.elapsed().as_secs_f64() * 1000.);
            }
            times.sort_by(f64::total_cmp);
            let image = renderer.capture(&frame, width, height)?;
            let pixels = ignis::image_similarity::Image::decode(&image)?.pixels;
            if variant == "field" {
                reference = Some(pixels.clone());
            }
            let reference_image = if coordinates {
                &proxy_reference
            } else {
                &reference
            };
            let difference=reference_image.as_ref().map(|r:&Vec<[u8;3]>|{let mut n=0usize;let mut max=0u8;let mut sum=0f64;for(a,b)in r.iter().zip(&pixels){n+=usize::from(a!=b);for(a,b)in a.iter().zip(b){let d=a.abs_diff(*b);max=max.max(d);sum+=f64::from(d).powi(2);}}json!({"different_pixels":n,"maximum_rgb8":max,"rms_rgb8":(sum/(r.len()*3)as f64).sqrt()})});
            if variant == "field_proxy" {
                proxy_reference = Some(pixels.clone());
            }
            fs::write(
                out.join(format!("{case}-{name}-{variant}-{width}x{height}.png")),
                image,
            )?;
            let row = json!({"camera":name,"variant":variant,"draw_wait_median_ms":times[times.len()/2],"field":renderer.surface_field_info(),"proxy":renderer.mesh_proxy_info(),"difference_from_same_field_source":difference,"difference_reference":if coordinates{"field_proxy"}else{"field"},"native":renderer.info()});
            println!(
                "{}",
                json!({"camera":name,"variant":variant,"draw_wait_median_ms":row["draw_wait_median_ms"],"field":row["field"],"proxy":row["proxy"],"difference":difference})
            );
            rows.push(row);
            fs::write(
                out.join(format!("{case}-probe-{width}x{height}.json")),
                serde_json::to_vec_pretty(
                    &json!({"library_sha256":serde_json::from_slice::<serde_json::Value>(&fs::read(out.join("field-link-provenance.json"))?)?["library_sha256"],"scene_sha256":scene_hash,"source_instances":e.scene.meshes.instances.len(),"method":"warm draw plus completion; not isolated GPU timestamps or presentation FPS","case":case,"population_filter":if case=="sparse"||case=="verify"{"omit horizon_tree_ placements"}else{"none"},"width":width,"height":height,"rows":rows}),
                )?,
            )?;
        }
    }
    fs::write(
        out.join(format!("{case}-probe-{width}x{height}.json")),
        serde_json::to_vec_pretty(
            &json!({"library_sha256":serde_json::from_slice::<serde_json::Value>(&fs::read(out.join("field-link-provenance.json"))?)?["library_sha256"],"scene_sha256":scene_hash,"source_instances":e.scene.meshes.instances.len(),"method":"warm draw plus completion; not isolated GPU timestamps or presentation FPS","case":case,"population_filter":if case=="sparse"||case=="verify"{"omit horizon_tree_ placements"}else{"none"},"width":width,"height":height,"rows":rows}),
        )?,
    )?;
    Ok(())
}
fn file_hash(path: &Path) -> Result<String> {
    let mut r = BufReader::new(fs::File::open(path)?);
    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
    let mut block = vec![0; 1024 * 1024];
    loop {
        let n = r.read(&mut block)?;
        if n == 0 {
            break;
        }
        hash.update(&block[..n]);
    }
    Ok(hash
        .finish()
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
fn compile_proxies(out: &Path) -> Result<()> {
    let scene: Scene = serde_json::from_reader(BufReader::new(fs::File::open(
        out.join("world.scene.json"),
    )?))?;
    let mut pack = ignis::mesh_proxy::compile(&scene.meshes)?;
    // Treat candidate generation as untrusted. This independent publication
    // gate also works with the earlier frozen diagnostic library.
    for entry in &mut pack.entries {
        let a = scene
            .meshes
            .assets
            .iter()
            .find(|a| a.id == entry.asset)
            .context("Proxy asset missing")?;
        let labels = proxy_sanitize::labels(&a.geometry);
        for l in &mut entry.levels {
            let (removed, restored) =
                proxy_sanitize::repair(&a.geometry, &labels, &mut l.vertices, &mut l.triangles)?;
            if removed > 0 || restored > 0 {
                eprintln!(
                    "repaired {}: removed {removed} zero-area triangles, restored {restored} source triangles",
                    entry.asset
                );
            }
        }
        entry.levels.retain(|l| {
            let g = meshes::Geometry {
                positions: l
                    .vertices
                    .iter()
                    .map(|i| a.geometry.positions[*i as usize])
                    .collect(),
                triangles: l.triangles.clone(),
                material_coordinates: None,
            };
            if let Err(e) = g.validate() {
                eprintln!(
                    "rejected {} level {} triangles: {}",
                    entry.asset,
                    l.triangles.len(),
                    e.message
                );
                false
            } else {
                true
            }
        });
    }
    pack.entries.retain(|e| !e.levels.is_empty());
    let bytes = pack.encode()?;
    let candidate = out.join("mesh.ignis-proxies.candidate");
    fs::write(&candidate, &bytes)?;
    fs::rename(candidate, out.join("mesh.ignis-proxies"))?;
    fs::write(
        out.join("mesh-proxies.json"),
        serde_json::to_vec_pretty(&pack.inspect())?,
    )?;
    println!(
        "{}",
        json!({"bytes":bytes.len(),"assets":pack.entries.len()})
    );
    Ok(())
}
fn densify(out: &Path) -> Result<()> {
    let mut scene: Scene = serde_json::from_reader(BufReader::new(fs::File::open(
        out.join("world.scene.json"),
    )?))?;
    scene
        .meshes
        .instances
        .retain(|i| !i.id.starts_with("horizon_tree_"));
    scene.meshes.objects = Arc::new(
        scene
            .meshes
            .objects
            .iter()
            .filter(|o| !o.id.starts_with("horizon_tree_"))
            .cloned()
            .collect(),
    );
    let placements: Vec<serde_json::Value> = serde_json::from_reader(BufReader::new(
        fs::File::open("artifacts/botanical-valley/placements.json")?,
    ))?;
    let mut catalog = BTreeMap::new();
    for p in placements.iter().filter(|p| p["collection"] == "trees") {
        let name = p["entry"]
            .as_str()
            .context("Tree entry missing")?
            .to_owned();
        if catalog.contains_key(&name) {
            continue;
        }
        let id = p["id"].as_str().context("Tree id missing")?;
        let object = scene
            .meshes
            .objects
            .iter()
            .find(|o| o.id == id)
            .context("Tree object missing")?;
        let templates = object
            .members
            .iter()
            .map(|id| {
                scene
                    .meshes
                    .instances
                    .iter()
                    .find(|i| &i.id == id)
                    .cloned()
                    .context("Tree member missing")
            })
            .collect::<Result<Vec<_>>>()?;
        catalog.insert(name, templates);
    }
    let kinds = [
        "eucalipto",
        "figueira",
        "figueira_rs",
        "palmeirinha",
        "platano",
    ];
    ensure!(
        kinds.iter().all(|k| catalog.contains_key(*k)),
        "Canonical tree catalog incomplete"
    );
    let g = terrain(&scene)?;
    ensure!(
        g.positions.len() == 501 * 501
            && g.positions[0][0] == 0.
            && g.positions.last().unwrap()[0] == 1000.,
        "Unexpected terrain grid"
    );
    let mut additions = vec![];
    let mut objects = vec![];
    let mut counts = BTreeMap::new();
    let mut occupied = [0usize; 400];
    for z in 0..80 {
        for x in 0..80 {
            let id = z * 80 + x;
            let h = surface_field::hash(id ^ 0x98ea01b3);
            if h % 100 >= 86 {
                continue;
            }
            let j = surface_field::jitter(id ^ 0x276e831a);
            let wx = -500. + (x as f32 + 0.5 + j[0] * 0.86) * 12.5;
            let wz = -500. + (z as f32 + 0.5 + j[1] * 0.86) * 12.5;
            let y = sample(g, wx, wz);
            let trail = -94. + 22. * (wz / 85.).sin() + 12. * (wz / 31.).sin();
            if y < 5.45 || (wx - trail).abs() < 3.5 {
                continue;
            }
            let kind = match surface_field::hash(h) % 10 {
                0..=2 => 0,
                3..=4 => 1,
                5 => 2,
                6 => 3,
                _ => 4,
            };
            let name = kinds[kind];
            let rotation =
                Quat::from_rotation_y((h & 65535) as f32 / 65535. * std::f32::consts::TAU)
                    .to_array();
            let scale = 0.83 + (h >> 16) as f32 / 65535. * 0.34;
            let tint = 0.9 + (surface_field::hash(h ^ 0x83197a) & 65535) as f32 / 65535. * 0.2;
            let object_id = format!("horizon_tree_{id}");
            let mut members = vec![];
            for (part, template) in catalog[name].iter().enumerate() {
                let mut i = template.clone();
                i.id = format!("{object_id}_{part}");
                i.position = [wx, y - 0.025, wz];
                i.rotation = rotation;
                i.scale = scale;
                i.color = i.color.map(|c| (c * tint).min(1.));
                members.push(i.id.clone());
                additions.push(i);
            }
            objects.push(ignis::mesh_objects::Object {
                id: object_id,
                members,
            });
            *counts.entry(name.to_owned()).or_insert(0usize) += 1;
            occupied[((wz + 500.) / 50.) as usize * 20 + ((wx + 500.) / 50.) as usize] += 1;
        }
    }
    let added = objects.len();
    scene.meshes.instances.extend(additions);
    Arc::make_mut(&mut scene.meshes.objects).extend(objects);
    scene.meshes.instances.sort_by(|a, b| a.asset.cmp(&b.asset));
    scene.validate()?;
    let candidate = out.join("world.scene.json.candidate");
    let mut writer = BufWriter::new(fs::File::create(&candidate)?);
    serde_json::to_writer(&mut writer, &scene)?;
    std::io::Write::flush(&mut writer)?;
    writer.get_ref().sync_all()?;
    fs::rename(candidate, out.join("world.scene.json"))?;
    let report = json!({"new_trees":added,"original_trees":108,"total_trees":added+108,"counts":counts,"trees_per_50m_cell":occupied.to_vec(),"seed":2565472691u32,"terrain_collision":"unchanged full source","canonical_assets":"unchanged; same compiled proxies reusable across placements"});
    fs::write(out.join("forest.json"), serde_json::to_vec_pretty(&report)?)?;
    println!("{report}");
    Ok(())
}
fn main() -> Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "probe".into());
    let out = Path::new("artifacts/horizon-meadow");
    match mode.as_str() {
        "compile" => compile(out),
        "probe" => probe(out),
        "compile-proxies" => compile_proxies(out),
        "densify" => densify(out),
        "inspect-models" => {
            let s: Scene = serde_json::from_reader(BufReader::new(fs::File::open(
                out.join("world.scene.json"),
            )?))?;
            for a in &s.meshes.assets {
                if a.geometry.triangles.len() >= 100000 {
                    println!(
                        "{}",
                        json!({"asset":a.id,"triangles":a.geometry.triangles.len(),"shading":a.shading,"coordinates":a.geometry.material_coordinates.is_some(),"rig":a.rig.is_some(),"material":a.material})
                    );
                }
            }
            Ok(())
        }
        _ => anyhow::bail!("Expected compile, compile-proxies or probe"),
    }
}
