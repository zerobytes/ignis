//! Held-out cost prediction experiment. Calibration never consumes validation results.
use super::*;
const OUTPUT: &str = "artifacts/appearance-probe/cost-scaling";
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
const CODES: [u32; 5] = [0, 3, 4, 5, 6];
const NAMES: [&str; 5] = [
    "full",
    "regional_precise",
    "global_sealed",
    "hybrid",
    "calibrated",
];
fn asset(family: usize, density: usize) -> Result<Asset> {
    Ok(serde_json::from_slice(&fs::read(
        Path::new(SOURCE).join(format!("f{family}_d{density}.json")),
    )?)?)
}
fn model() -> Result<cost::Model> {
    let m: cost::Model = serde_json::from_slice(
        &fs::read(Path::new(OUTPUT).join("model.json")).context("run scripts/fit_cost_model.py")?,
    )?;
    m.validate()?;
    Ok(m)
}
pub(super) fn run(mode: &str) -> Result<()> {
    fs::create_dir_all(OUTPUT)?;
    if mode == "--inspect-cost-scaling" {
        return inspect_selection();
    }
    if mode == "--capture-cost-scaling" {
        return capture();
    }
    let calibration = mode == "--calibrate-cost-scaling";
    let repeat = mode == "--repeat-cost-scaling";
    let mut renderer = pollster::block_on(Renderer::new())?;
    if !calibration {
        renderer.cost_model = Some(model()?);
    }
    let distances: &[f32] = if calibration {
        &[18., 40., 80., 160.]
    } else {
        &[14., 28., 56., 112., 224.]
    };
    let modes: &[u32] = if calibration { &[3, 4] } else { &CODES };
    let mut results = Vec::new();
    for family in 0..2 {
        let assets = (0..4)
            .map(|d| asset(family, d))
            .collect::<Result<Vec<_>>>()?;
        let uploaded = assets
            .iter()
            .map(|a| renderer.upload_from(a, SOURCE))
            .collect::<Result<Vec<_>>>()?;
        let inputs = uploaded.iter().zip(&assets).collect::<Vec<_>>();
        for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
            renderer.resize(size);
            for (frame, &distance) in distances.iter().enumerate() {
                if repeat
                    && !((family == 0 && resolution == 0 && frame == 0)
                        || (family == 1 && resolution == 1 && frame == 4))
                {
                    continue;
                }
                let cam = render::camera_at(distance, size);
                let features = assets
                    .iter()
                    .map(|a| cost::features(a, &cam))
                    .collect::<Vec<_>>();
                let timings = renderer.bench_cost(&inputs, distance, size, modes)?;
                results.push(json!({"family":family,"resolution":resolution,"size":size,"frame":frame,"distance":distance,"features":features,"timings":timings,"batch_frames":128,"rounds":8,"gpu_batch":32,"adapter":renderer.gpu.adapter,"model":renderer.cost_model}));
                let name = if repeat {
                    "recheck.json"
                } else if calibration {
                    "calibration.json"
                } else {
                    "confirmation.json"
                };
                fs::write(
                    Path::new(OUTPUT).join(name),
                    serde_json::to_vec_pretty(&results)?,
                )?;
                println!("{name} family{family} resolution{resolution} distance{distance}");
            }
        }
    }
    Ok(())
}
fn capture() -> Result<()> {
    let root = Path::new(OUTPUT);
    let m = model()?;
    let mut renderer = pollster::block_on(Renderer::new())?;
    renderer.cost_model = Some(m.clone());
    let mut rows = Vec::new();
    let mut assets = Vec::new();
    let mut transitions = Vec::new();
    for family in 0..2 {
        for density in [0, 3, 1, 2] {
            let a = asset(family, density)?;
            let u = renderer.upload_from(&a, SOURCE)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                let mut previous = vec![Vec::new(); 4];
                for frame in 0..9 {
                    let cam = pose(frame, size);
                    let id = format!("f{family}_d{density}_r{resolution}_{frame}");
                    let (selected, strategy, visited) = cost::choose(&a, &cam, &m, true);
                    let exhaustive = cost::choose(&a, &cam, &m, false);
                    ensure!(
                        selected == exhaustive.0 && strategy == exhaustive.1,
                        "pruning decision"
                    );
                    let (full, n) = renderer.capture(&u, &a, &cam, 0)?;
                    save(&root.join(format!("{id}_0.png")), &full, size[0])?;
                    let mut checks = Vec::new();
                    let mut triangles = vec![n];
                    let mut chosen = None;
                    for (index, &code) in CODES.iter().enumerate().skip(1) {
                        let (image, n) = renderer.capture(&u, &a, &cam, code)?;
                        save(&root.join(format!("{id}_{index}.png")), &image, size[0])?;
                        let (check, residual) = quality(&full, &image, &previous[index - 1]);
                        previous[index - 1] = residual;
                        if code == 6 {
                            ensure!(chosen.as_ref() == Some(&image), "selected image");
                        }
                        if code == strategy {
                            chosen = Some(image);
                        }
                        checks.push(check);
                        triangles.push(n);
                    }
                    let visible = full.iter().filter(|p| p[3] > 0.).count();
                    rows.push(json!({"id":id,"family":family,"density":density,"factor":a.factor,"resolution":resolution,"size":size,"frame":frame,"distance":cam.eye.length(),"triangles":triangles,"levels":vec![0;5],"checks":checks,"covered_pixels":visible,"source_triangles_per_covered_pixel":triangles[0]as f64/visible as f64,"strategy":strategy,"visited":visited,"draws":selected.1,"chosen_image_identical":true,"features":cost::features(&a,&cam),"predicted_delta_ms":m.predict(cost::features(&a,&cam)),"timings":null}));
                }
                if family == 1 && density == 3 {
                    let mut last = cost::choose(&a, &render::camera_at(14., size), &m, true).1;
                    let mut distances = Vec::new();
                    for step in 29..=448 {
                        let d = step as f32 * 0.5;
                        let choice = cost::choose(&a, &render::camera_at(d, size), &m, true).1;
                        if choice != last {
                            distances.extend([d - 0.5, d]);
                        }
                        last = choice;
                    }
                    distances.dedup();
                    let mut previous = Vec::new();
                    for (i, d) in distances.into_iter().enumerate() {
                        let cam = render::camera_at(d, size);
                        let (selected, strategy, visited) = cost::choose(&a, &cam, &m, true);
                        let exhaustive = cost::choose(&a, &cam, &m, false);
                        ensure!(
                            selected == exhaustive.0 && strategy == exhaustive.1,
                            "transition pruning"
                        );
                        let id = format!("transition_{}_{i}", size[0]);
                        let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
                        let (image, _) = renderer.capture(&u, &a, &cam, 6)?;
                        let (chosen, _) = renderer.capture(&u, &a, &cam, strategy)?;
                        ensure!(image == chosen, "transition candidate");
                        let (check, residual) = quality(&full, &image, &previous);
                        previous = residual;
                        save(&root.join(format!("{id}_0.png")), &full, size[0])?;
                        save(&root.join(format!("{id}_4.png")), &image, size[0])?;
                        transitions.push(json!({"id":id,"size":size,"distance":d,"strategy":strategy,"visited":visited,"draws":selected.1,"triangles":selected.2,"check":check,"chosen_image_identical":true}));
                    }
                }
            }
            assets.push(serde_json::to_value(&a)?);
            fs::write(
                root.join("report.json"),
                serde_json::to_vec_pretty(
                    &json!({"rows":rows,"assets":assets,"modes":NAMES,"model":m,"adapter":renderer.gpu.adapter}),
                )?,
            )?;
            fs::write(
                root.join("transitions.json"),
                serde_json::to_vec_pretty(&transitions)?,
            )?;
            println!("captured family{family} density{density}");
        }
    }
    Ok(())
}

fn inspect_selection() -> Result<()> {
    let model = model()?;
    let mut rows = Vec::new();
    for family in 0..2 {
        for density in 0..4 {
            let a = asset(family, density)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                for frame in 0..5 {
                    let cam = pose(frame, size);
                    let mut samples = vec![Vec::new(); 4];
                    for round in 0..8 {
                        for offset in 0..4 {
                            let m = (round + offset) % 4;
                            let start = Instant::now();
                            for _ in 0..1000 {
                                let a = std::hint::black_box(&a);
                                let cam = std::hint::black_box(&cam);
                                let selected = if m == 3 {
                                    cost::choose(a, cam, &model, true).0
                                } else {
                                    render::selection(a, cam, [3, 4, 5][m])
                                };
                                std::hint::black_box(selected);
                            }
                            samples[m].push(start.elapsed().as_secs_f64());
                        }
                    }
                    // seconds per1000 calls numerically equals ms per call.
                    rows.push(json!({"family":family,"density":density,"resolution":resolution,"frame":frame,
                "distance":cam.eye.length(),"milliseconds_per_selection":samples.iter_mut().map(|x|stats(x)).collect::<Vec<_>>()}));
                }
            }
        }
    }
    fs::write(
        Path::new(OUTPUT).join("selection-cpu.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    println!("selection CPU: {}poses,1000calls/block,8rounds", rows.len());
    Ok(())
}
