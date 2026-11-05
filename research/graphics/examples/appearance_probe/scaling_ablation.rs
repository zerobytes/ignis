//! Controlled shader interventions; no fitting to measured winners.
use super::*;
const OUTPUT: &str = "artifacts/appearance-probe/scaling-ablation";
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
fn asset(family: usize, density: usize) -> Result<Asset> {
    Ok(serde_json::from_slice(&fs::read(
        Path::new(SOURCE).join(format!("f{family}_d{density}.json")),
    )?)?)
}
pub(super) fn run(mode: &str) -> Result<()> {
    fs::create_dir_all(OUTPUT)?;
    if mode == "--capture-depth-ablation" {
        return capture_depth();
    }
    if mode == "--capture-ablation" {
        return capture();
    }
    let depth = mode.contains("depth");
    let repeat = mode.starts_with("--repeat");
    let order = mode == "--order-ablation";
    let diagnostic = mode == "--ablate-scaling" || order;
    let mut renderer = pollster::block_on(Renderer::new_diagnostic())?;
    let modes: &[u32] = if depth {
        &[0, 5, 45, 75, 85]
    } else if order {
        &[3, 4, 53, 54]
    } else if diagnostic {
        &[3, 4, 13, 14, 23, 24, 33, 34, 43, 44]
    } else {
        &[0, 3, 4, 5, 45, 65, 75]
    };
    let distances: &[f32] = if diagnostic {
        &[14., 224.]
    } else {
        &[14., 21., 28., 56., 84., 112., 224.]
    };
    let mut results = Vec::new();
    for family in 0..2 {
        let assets = (0..4)
            .map(|d| asset(family, d))
            .collect::<Result<Vec<_>>>()?;
        let mut uploaded = assets
            .iter()
            .map(|a| renderer.upload_from(a, SOURCE))
            .collect::<Result<Vec<_>>>()?;
        if order {
            for (u, a) in uploaded.iter_mut().zip(&assets) {
                renderer.reverse_order(u, a, SOURCE)?;
            }
        }
        let inputs = uploaded.iter().zip(&assets).collect::<Vec<_>>();
        for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
            if diagnostic && resolution != 0 {
                continue;
            }
            renderer.resize(size);
            for &distance in distances {
                if repeat
                    && !(resolution == 0 && (distance == 14. || (family == 1 && distance == 224.)))
                {
                    continue;
                }
                let cam = render::camera_at(distance, size);
                let selections = assets
                    .iter()
                    .map(|a| {
                        [0, 3, 4, 5].map(|m| {
                            let s = render::selection(a, &cam, m);
                            json!({"triangles":s.2,"draws":s.1})
                        })
                    })
                    .collect::<Vec<_>>();
                let timings = renderer.bench_cost(&inputs, distance, size, modes)?;
                results.push(json!({"family":family,"resolution":resolution,"size":size,"distance":distance,"codes":modes,"timings":timings,"selections":selections,"adapter":renderer.gpu.adapter}));
                let name = if depth {
                    if repeat {
                        "depth-recheck.json"
                    } else {
                        "depth.json"
                    }
                } else if repeat {
                    "recheck.json"
                } else if order {
                    "order.json"
                } else if diagnostic {
                    "diagnostic.json"
                } else {
                    "confirmation.json"
                };
                fs::write(
                    Path::new(OUTPUT).join(name),
                    serde_json::to_vec_pretty(&results)?,
                )?;
                println!("{name}: family{family} resolution{resolution} distance{distance}");
            }
        }
    }
    Ok(())
}
fn capture() -> Result<()> {
    let mut renderer = pollster::block_on(Renderer::new_diagnostic())?;
    let mut rows = Vec::new();
    for family in 0..2 {
        for density in 0..4 {
            let a = asset(family, density)?;
            let mut u = renderer.upload_from(&a, SOURCE)?;
            renderer.reverse_order(&mut u, &a, SOURCE)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                let mut previous = vec![Vec::new(); 7];
                for (frame, distance) in [
                    14., 21., 28., 56., 84., 112., 224., 112., 84., 56., 28., 21., 14.,
                ]
                .into_iter()
                .enumerate()
                {
                    let cam = render::camera_at(distance, size);
                    let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
                    let id = format!("f{family}_d{density}_r{resolution}_{frame}");
                    let mut checks = Vec::new();
                    let mut old_check = serde_json::Value::Null;
                    for (i, code) in [43, 44, 45, 63, 64, 65, 75].into_iter().enumerate() {
                        let base = code % 10;
                        let (old, _) = renderer.capture(&u, &a, &cam, base)?;
                        let (new, _) = renderer.capture(&u, &a, &cam, code)?;
                        let (check, residual) = quality(&full, &new, &previous[i]);
                        previous[i] = residual;
                        let difference = old
                            .iter()
                            .zip(&new)
                            .flat_map(|(a, b)| a.iter().zip(b).map(|(x, y)| (x - y).abs()))
                            .fold(0f32, f32::max);
                        checks.push(
                            json!({"code":code,"quality":check,"max_change":difference,"exact":old==new}),
                        );
                        if code == 45 {
                            save(
                                &Path::new(OUTPUT).join(format!("{id}_2.png")),
                                &new,
                                size[0],
                            )?;
                        }
                        if code == 75 {
                            save(
                                &Path::new(OUTPUT).join(format!("{id}_4.png")),
                                &new,
                                size[0],
                            )?;
                        }
                        if code == 65 {
                            old_check = quality(&full, &old, &[]).0;
                            save(
                                &Path::new(OUTPUT).join(format!("{id}_1.png")),
                                &old,
                                size[0],
                            )?;
                            save(
                                &Path::new(OUTPUT).join(format!("{id}_3.png")),
                                &new,
                                size[0],
                            )?;
                        }
                    }
                    let mut order_checks = Vec::new();
                    for base in [3, 4] {
                        let (old, _) = renderer.capture(&u, &a, &cam, base)?;
                        let (reversed, _) = renderer.capture(&u, &a, &cam, base + 50)?;
                        let (check, _) = quality(&old, &reversed, &[]);
                        order_checks
                            .push(json!({"base":base,"exact":old==reversed,"quality":check}));
                    }
                    let triangles = [0, 5, 5, 5, 5].map(|m| render::selection(&a, &cam, m).2);
                    let visible = full.iter().filter(|p| p[3] > 0.).count();
                    save(
                        &Path::new(OUTPUT).join(format!("{id}_0.png")),
                        &full,
                        size[0],
                    )?;
                    rows.push(json!({"id":id,"family":family,"density":density,"resolution":resolution,"size":size,"distance":distance,"frame":frame,"checks":checks,"order_checks":order_checks,"old_check":old_check,"triangles":triangles,"covered_pixels":visible}));
                }
            }
            fs::write(
                Path::new(OUTPUT).join("quality.json"),
                serde_json::to_vec_pretty(&rows)?,
            )?;
            println!("quality family{family} density{density}");
        }
    }
    Ok(())
}

fn capture_depth() -> Result<()> {
    let root = Path::new(OUTPUT);
    let mut renderer = pollster::block_on(Renderer::new_diagnostic())?;
    let mut rows = Vec::new();
    for family in 0..2 {
        for density in 0..4 {
            let a = asset(family, density)?;
            let u = renderer.upload_from(&a, SOURCE)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                let mut previous = Vec::new();
                for (frame, distance) in [
                    14., 21., 28., 56., 84., 112., 224., 112., 84., 56., 28., 21., 14.,
                ]
                .into_iter()
                .enumerate()
                {
                    let cam = render::camera_at(distance, size);
                    let id = format!("f{family}_d{density}_r{resolution}_{frame}");
                    let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
                    let (old, _) = renderer.capture(&u, &a, &cam, 5)?;
                    for (m, pixels) in [(0, &full), (1, &old)] {
                        save(&root.join("control.png"), pixels, size[0])?;
                        ensure!(
                            fs::read(root.join("control.png"))?
                                == fs::read(root.join(format!("{id}_{m}.png")))?,
                            "depth control changed {id}/{m}"
                        );
                    }
                    let (image, _) = renderer.capture(&u, &a, &cam, 85)?;
                    let (check, residual) = quality(&full, &image, &previous);
                    previous = residual;
                    let difference = old
                        .iter()
                        .zip(&image)
                        .flat_map(|(a, b)| a.iter().zip(b).map(|(x, y)| (x - y).abs()))
                        .fold(0f32, f32::max);
                    save(&root.join(format!("{id}_5.png")), &image, size[0])?;
                    rows.push(json!({"id":id,"family":family,"density":density,"resolution":resolution,"size":size,"distance":distance,"frame":frame,"quality":check,"max_change":difference,"controls_identical":true}));
                }
            }
            fs::write(
                root.join("depth-quality.json"),
                serde_json::to_vec_pretty(&rows)?,
            )?;
            println!("depth quality family{family} density{density}");
        }
    }
    let transitions: Vec<serde_json::Value> =
        serde_json::from_slice(&fs::read(Path::new(SOURCE).join("transitions.json"))?)?;
    let a = asset(1, 3)?;
    let u = renderer.upload_from(&a, SOURCE)?;
    let mut results = Vec::new();
    for t in transitions {
        let size: [u32; 2] = serde_json::from_value(t["size"].clone())?;
        let distance = t["distance"].as_f64().unwrap() as f32;
        let cam = render::camera_at(distance, size);
        let id = t["id"].as_str().unwrap();
        let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
        let (image, _) = renderer.capture(&u, &a, &cam, 85)?;
        let (check, _) = quality(&full, &image, &[]);
        save(&root.join(format!("{id}_0.png")), &full, size[0])?;
        save(&root.join(format!("{id}_5.png")), &image, size[0])?;
        results.push(json!({"id":id,"size":size,"distance":distance,"strategy":render::hybrid_choice(&a,&cam,true).1,"quality":check}));
    }
    fs::write(
        root.join("depth-transitions.json"),
        serde_json::to_vec_pretty(&results)?,
    )?;
    fs::remove_file(root.join("control.png"))?;
    Ok(())
}
