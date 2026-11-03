//! Matched native A/B: same meshes, appearance selection, camera and resolution.
//! Select visibility, indexing, attribute compaction or experimental projection.
use anyhow::Result;
use ignis::{
    engine::Engine,
    model::{Camera, Scene},
    render::Renderer,
};
use serde_json::json;
use std::{fs, time::Instant};
fn main() -> Result<()> {
    let minimal = std::env::args().any(|a| a == "--minimal");
    let occlusion = std::env::args().any(|a| a == "--occlusion");
    let indexing = std::env::args().any(|a| a == "--indexing");
    let projection = std::env::args().any(|a| a == "--projection");
    let compact = std::env::args().any(|a| a == "--compact");
    let combined = std::env::args().any(|a| a == "--combined");
    anyhow::ensure!(
        [indexing, projection, compact, combined, occlusion, minimal]
            .into_iter()
            .filter(|&x| x)
            .count()
            <= 1,
        "choose one experiment"
    );
    let visibility = !indexing && !projection && !compact && !combined && !occlusion && !minimal;
    let path = std::env::args()
        .filter(|a| !a.starts_with("--"))
        .nth(1)
        .unwrap_or_else(|| "artifacts/landscape-forest/forest.scene.json".into());
    let out = std::path::Path::new(if minimal {
        "artifacts/minimal-vertices"
    } else if occlusion {
        "artifacts/occlusion"
    } else if combined {
        "artifacts/geometry-cost"
    } else if compact {
        "artifacts/compact-vertices"
    } else if projection {
        "artifacts/lod-projection"
    } else if indexing {
        "artifacts/indexing"
    } else {
        "artifacts/visibility"
    });
    fs::create_dir_all(out)?;
    let scene: Scene = serde_json::from_slice(&fs::read(path)?)?;
    let engine = Engine::new(scene)?;
    let mut frame = engine.render_frame();
    let gpu = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&gpu, None))?;
    // Isolate other experiments from the newly automatic occlusion path.
    renderer.set_occlusion_culling(minimal);
    renderer.set_minimal_vertices(false);
    let switch = |renderer: &mut Renderer, enabled| {
        if minimal {
            renderer.set_minimal_vertices(enabled);
        } else if occlusion {
            renderer.set_occlusion_culling(enabled);
        } else if combined {
            renderer.set_geometry_indexing(enabled);
            renderer.set_compact_vertices(enabled);
        } else if compact {
            renderer.set_compact_vertices(enabled);
        } else if projection {
            renderer.set_precise_lod_projection(enabled);
        } else if indexing {
            renderer.set_geometry_indexing(enabled);
        } else {
            renderer.set_visibility_culling(enabled);
        }
    };
    let (width, height) = (2048, 1536);
    let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("visibility cost"),
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
    let mut rows = vec![];
    let poses = [
        (
            "near",
            Camera {
                position: [445., 920., 8280.],
                target: [390., 880., 7780.],
                fov_degrees: 60.,
            },
        ),
        (
            "panorama",
            Camera {
                position: [0., 1200., 9000.],
                target: [0., 650., 5000.],
                fov_degrees: 60.,
            },
        ),
        (
            "reverse",
            Camera {
                position: [445., 920., 8280.],
                target: [500., 960., 8780.],
                fov_degrees: 60.,
            },
        ),
    ];
    if std::env::args().any(|a| a == "--stability") {
        let directory = std::path::Path::new("artifacts/geometry-stability");
        fs::create_dir_all(directory)?;
        frame.camera = poses[0].1.clone();
        let mut rows = Vec::new();
        for (indexed, compact) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut renderer = pollster::block_on(Renderer::new(&gpu, None))?;
            renderer.set_geometry_indexing(indexed);
            renderer.set_compact_vertices(compact);
            let mut first: Option<Vec<[u8; 3]>> = None;
            let mut previous: Option<Vec<[u8; 3]>> = None;
            for repeat in 0..8 {
                let png = renderer.capture(&frame, width, height)?;
                let image = ignis::image_similarity::Image::decode(&png)?;
                let metric = |reference: &Option<Vec<[u8; 3]>>| {
                    reference.as_ref().map(|p| {
                        let d: Vec<_> = p
                            .iter()
                            .flatten()
                            .zip(image.pixels.iter().flatten())
                            .map(|(a, b)| a.abs_diff(*b))
                            .collect();
                        json!({"max":d.iter().max(),"changed":d.iter().filter(|&&x|x>0).count()})
                    })
                };
                let row = json!({"indexed":indexed,"compact":compact,"repeat":repeat,"first":metric(&first),"previous":metric(&previous),"adapter":renderer.info(),"gpu_occlusion":if occlusion || minimal {renderer.inspect_occlusion_gpu()?} else {serde_json::Value::Null}});
                println!("{}", row);
                rows.push(row);
                if first.is_none() {
                    first = Some(image.pixels.clone());
                }
                previous = Some(image.pixels);
                fs::write(
                    directory.join(format!("{indexed}-{compact}-{repeat}.png")),
                    png,
                )?;
            }
        }
        fs::write(
            directory.join("stability.json"),
            serde_json::to_vec_pretty(&rows)?,
        )?;
        return Ok(());
    }
    for (name, camera) in poses {
        frame.camera = camera;
        let mut pixels: Option<Vec<[u8; 3]>> = None;
        let mut max_rgb8 = 0;
        let mut changed_channels = 0;
        for enabled in [false, true] {
            switch(&mut renderer, enabled);
            let png = renderer.capture(&frame, width, height)?;
            let image = ignis::image_similarity::Image::decode(&png)?;
            let initial_info = renderer.info();
            let repeat_png = renderer.capture(&frame, width, height)?;
            let repeated = ignis::image_similarity::Image::decode(&repeat_png)?;
            if image.pixels != repeated.pixels {
                fs::write(out.join(format!("{name}-{enabled}-unstable-a.png")), &png)?;
                fs::write(
                    out.join(format!("{name}-{enabled}-unstable-b.png")),
                    repeat_png,
                )?;
                let diffs: Vec<_> = image
                    .pixels
                    .iter()
                    .flatten()
                    .zip(repeated.pixels.iter().flatten())
                    .map(|(a, b)| a.abs_diff(*b))
                    .collect();
                eprintln!(
                    "repeatability: max {:?}, changed {}",
                    diffs.iter().max(),
                    diffs.iter().filter(|&&d| d > 0).count()
                );
                fs::write(
                    out.join("unstable-info.json"),
                    serde_json::to_vec_pretty(&json!({"a": initial_info, "b": renderer.info()}))?,
                )?;
                for attempt in 0..4 {
                    let next = renderer.capture(&frame, width, height)?;
                    let decoded = ignis::image_similarity::Image::decode(&next)?;
                    eprintln!(
                        "capture {attempt}: equals first {}, equals second {}",
                        decoded.pixels == image.pixels,
                        decoded.pixels == repeated.pixels
                    );
                    fs::write(
                        out.join(format!("{name}-{enabled}-unstable-{attempt}.png")),
                        next,
                    )?;
                }
            }
            anyhow::ensure!(
                image.pixels == repeated.pixels,
                "unchanged capture is unstable in {name}/{enabled}"
            );
            if let Some(ref reference) = pixels {
                let errors = image
                    .pixels
                    .iter()
                    .flatten()
                    .zip(reference.iter().flatten())
                    .map(|(a, b)| a.abs_diff(*b));
                for error in errors {
                    max_rgb8 = max_rgb8.max(error);
                    changed_channels += usize::from(error != 0);
                }
                if visibility || occlusion || minimal {
                    anyhow::ensure!(image.pixels == *reference, "pixels changed in {name}");
                }
            } else {
                pixels = Some(image.pixels);
            }
            fs::write(
                out.join(format!(
                    "{name}-{}.png",
                    if enabled { "optimized" } else { "reference" }
                )),
                png,
            )?;
        }
        eprintln!("{name}: max RGB8 {max_rgb8}, changed channels {changed_channels}");
        for repeat in 0..3 {
            for enabled in if repeat % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                switch(&mut renderer, enabled);
                let mut submit = vec![];
                let mut total = vec![];
                for n in 0..24 {
                    let start = Instant::now();
                    renderer.draw(&frame, &view, width, height)?;
                    let submitted = start.elapsed().as_secs_f64() * 1000.;
                    renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                    if n >= 8 {
                        submit.push(submitted);
                        total.push(start.elapsed().as_secs_f64() * 1000.);
                    }
                }
                submit.sort_by(f64::total_cmp);
                total.sort_by(f64::total_cmp);
                let row = json!({"pose":name,"camera":frame.camera,"repeat":repeat,"enabled":enabled,"cpu_submission_median_ms":submit[8],"draw_wait_median_ms":total[8],"draw_wait_p95_ms":total[15],"max_rgb8":max_rgb8,"changed_channels":changed_channels,"adapter":renderer.info(),"gpu_occlusion":if occlusion || minimal {renderer.inspect_occlusion_gpu()?} else {serde_json::Value::Null}});
                println!("{}", row);
                rows.push(row);
                fs::write(
                    out.join("cost.json"),
                    serde_json::to_vec_pretty(
                        &json!({"width":width,"height":height,"optimization":if minimal {"consumed vertex attributes"} else if occlusion {"current-frame GPU occlusion"} else if combined {"indexing+compact attributes"} else if compact {"compact attributes"} else if projection {"perspective bound"} else if indexing {"lossless vertex indexing"} else {"visibility"},"repeatability_max_rgb8":0,"method":"3 alternating rounds,8 warmup+16 samples. CPU submission includes driver; draw+GPU completion is not isolated GPU time or gameplay FPS. RGB8 differences measured before timing.","rows":rows}),
                    )?,
                )?;
            }
        }
    }
    // Camera translation, reversal and resize invalidate visibility without
    // rebuilding source geometry. Every pose is checked against the full path.
    let mut motion = Vec::new();
    for n in 0..8 {
        frame.camera.position = [
            445. + n as f32 * 40.,
            920. - n as f32 * 12.,
            8280. - n as f32 * 70.,
        ];
        frame.camera.target = [390., 880., 7780.];
        if minimal {
            frame.light.direction = [0.3 + n as f32 * 0.1, -1., 0.2 - n as f32 * 0.1];
        }
        let (w, h) = if n % 2 == 0 { (864, 558) } else { (558, 864) };
        switch(&mut renderer, false);
        let a = renderer.capture(&frame, w, h)?;
        switch(&mut renderer, true);
        let b = renderer.capture(&frame, w, h)?;
        let a = ignis::image_similarity::Image::decode(&a)?;
        let b = ignis::image_similarity::Image::decode(&b)?;
        let errors: Vec<_> = a
            .pixels
            .iter()
            .flatten()
            .zip(b.pixels.iter().flatten())
            .map(|(a, b)| a.abs_diff(*b))
            .collect();
        let max = errors.iter().max().copied().unwrap_or(0);
        motion.push(json!({"pose":n,"max_rgb8":max,"changed_channels":errors.iter().filter(|&&e|e>0).count()}));
        if visibility || occlusion || minimal {
            anyhow::ensure!(a.pixels == b.pixels, "motion {n} changed pixels");
        }
    }
    fs::write(
        out.join("motion.json"),
        serde_json::to_vec_pretty(
            &json!({"poses":8,"rows":motion,"portrait_landscape":true,"changing_light":minimal}),
        )?,
    )?;
    Ok(())
}
