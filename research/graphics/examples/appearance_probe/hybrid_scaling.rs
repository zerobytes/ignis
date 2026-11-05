//! Hybrid selector experiment; the reference, bake and GPU machinery are shared.
use super::*;
const OUTPUT: &str = "artifacts/appearance-probe/hybrid-scaling";
const CODES: [u32; 5] = [0, 2, 3, 4, 5];
const NAMES: [&str; 5] = [
    "full",
    "global_precise",
    "regional_precise",
    "global_sealed",
    "hybrid",
];
pub(super) fn run(mode: &str) -> Result<()> {
    match mode {
        "--bake-hybrid-scaling" => build::bake_at(OUTPUT, true),
        "--confirm-hybrid-scaling" => confirm(),
        "--hybrid-transitions" => transitions(),
        _ => capture(),
    }
}
fn asset(family: usize, density: usize) -> Result<Asset> {
    let a: Asset = serde_json::from_slice(
        &fs::read(Path::new(OUTPUT).join(format!("f{family}_d{density}.json")))
            .context("run --bake-hybrid-scaling")?,
    )?;
    ensure!(
        !a.sealed.is_empty() && a.regional_floor > 0,
        "missing hybrid bake"
    );
    Ok(a)
}
fn capture() -> Result<()> {
    let root = Path::new(OUTPUT);
    let mut renderer = pollster::block_on(Renderer::new())?;
    let mut rows = Vec::new();
    let mut assets = Vec::new();
    for family in 0..2 {
        for density in [0, 3, 1, 2] {
            let a = asset(family, density)?;
            let u = renderer.upload_from(&a, OUTPUT)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                let mut previous = vec![Vec::new(); 4];
                for frame in 0..9 {
                    let cam = pose(frame, size);
                    let id = format!("f{family}_d{density}_r{resolution}_{frame}");
                    let (full, n) = renderer.capture(&u, &a, &cam, 0)?;
                    save(&root.join(format!("{id}_0.png")), &full, size[0])?;
                    let visible = full.iter().filter(|p| p[3] > 0.).count();
                    let mut triangles = vec![n];
                    let mut checks = Vec::new();
                    let (selected, strategy, visited) = render::hybrid_choice(&a, &cam, true);
                    let exhaustive = render::hybrid_choice(&a, &cam, false);
                    ensure!(
                        selected == exhaustive.0 && strategy == exhaustive.1,
                        "pruning changed decision"
                    );
                    let mut chosen = None;
                    for (m, &code) in CODES.iter().enumerate().skip(1) {
                        let (image, n) = renderer.capture(&u, &a, &cam, code)?;
                        save(&root.join(format!("{id}_{m}.png")), &image, size[0])?;
                        let (check, residual) = quality(&full, &image, &previous[m - 1]);
                        previous[m - 1] = residual;
                        checks.push(check);
                        triangles.push(n);
                        if code == 5 {
                            ensure!(
                                chosen.as_ref() == Some(&image),
                                "hybrid differs from selected candidate"
                            );
                        }
                        if code == strategy {
                            chosen = Some(image);
                        }
                    }
                    println!(
                        "{id}: {triangles:?}, strategy{strategy}, visited{visited}, gate{}",
                        checks[3]["quality_gate"]
                    );
                    rows.push(json!({"id":id,"family":family,"density":density,"factor":a.factor,"resolution":resolution,"size":size,"frame":frame,"distance":cam.eye.length(),"triangles":triangles,"levels":vec![0;5],"checks":checks,"covered_pixels":visible,"source_triangles_per_covered_pixel":triangles[0]as f64/visible as f64,"strategy":strategy,"visited":visited,"draws":selected.1,"chosen_image_identical":true,"timings":null}));
                }
            }
            assets.push(serde_json::to_value(&a)?);
            fs::write(
                root.join("report.json"),
                serde_json::to_vec_pretty(
                    &json!({"schema":1,"adapter":renderer.gpu.adapter,"modes":NAMES,"codes":CODES,"assets":assets,"rows":rows,"threshold_px":0.15,"draw_weight":1024}),
                )?,
            )?;
        }
    }
    Ok(())
}
fn confirm() -> Result<()> {
    let root = Path::new(OUTPUT);
    let mut renderer = pollster::block_on(Renderer::new())?;
    let mut results = Vec::new();
    for family in 0..2 {
        let assets = (0..4)
            .map(|d| asset(family, d))
            .collect::<Result<Vec<_>>>()?;
        let uploaded = assets
            .iter()
            .map(|a| renderer.upload_from(a, OUTPUT))
            .collect::<Result<Vec<_>>>()?;
        let inputs = uploaded.iter().zip(&assets).collect::<Vec<_>>();
        for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
            for frame in 0..5 {
                renderer.resize(size);
                let timings = renderer.bench_modes(&inputs, frame, size, 128, &CODES)?;
                results.push(json!({"family":family,"resolution":resolution,"frame":frame,"batch_frames":128,"timings":timings,"resident_asset_bytes":assets.iter().map(|a|a.bytes).sum::<u64>()}));
                fs::write(
                    root.join("confirmation.json"),
                    serde_json::to_vec_pretty(&results)?,
                )?;
                println!("confirmed family{family} resolution{resolution} frame{frame}");
            }
        }
    }
    Ok(())
}
fn transitions() -> Result<()> {
    let root = Path::new(OUTPUT);
    let a = asset(1, 3)?;
    let mut renderer = pollster::block_on(Renderer::new())?;
    let u = renderer.upload_from(&a, OUTPUT)?;
    let mut rows = Vec::new();
    for size in RESOLUTIONS {
        let mut distances = Vec::new();
        let mut last = render::hybrid_choice(&a, &render::camera_at(14., size), true).1;
        for step in 29..=448 {
            let d = step as f32 * 0.5;
            let choice = render::hybrid_choice(&a, &render::camera_at(d, size), true).1;
            if choice != last {
                distances.extend([d - 0.5, d]);
            }
            last = choice;
        }
        ensure!(!distances.is_empty(), "no strategy transition exercised");
        distances.dedup();
        let mut previous = Vec::new();
        for (frame, &distance) in distances.iter().enumerate() {
            let cam = render::camera_at(distance, size);
            let (selection, strategy, visited) = render::hybrid_choice(&a, &cam, true);
            let id = format!("transition_{}_{}", size[0], frame);
            let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
            let (hybrid, _) = renderer.capture(&u, &a, &cam, 5)?;
            let (chosen, _) = renderer.capture(&u, &a, &cam, strategy)?;
            ensure!(hybrid == chosen, "transition candidate mismatch");
            let (check, residual) = quality(&full, &hybrid, &previous);
            previous = residual;
            save(&root.join(format!("{id}_0.png")), &full, size[0])?;
            save(&root.join(format!("{id}_4.png")), &hybrid, size[0])?;
            rows.push(json!({"id":id,"size":size,"distance":distance,"strategy":strategy,"visited":visited,"draws":selection.1,"triangles":selection.2,"check":check,"chosen_image_identical":true}));
            println!("transition {id} distance{distance} strategy{strategy}");
        }
    }
    fs::write(
        root.join("transitions.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}
