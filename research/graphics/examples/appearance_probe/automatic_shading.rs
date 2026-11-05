//! Conservative pipeline choice after geometric eligibility, without frame readback.
use super::*;
const OUTPUT: &str = "artifacts/appearance-probe/automatic-shading";
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
const DISTANCES: [f32; 10] = [14., 19., 21., 28., 32., 56., 84., 112., 128., 224.];
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Model {
    pub benefit: [f64; 6],
    pub baseline: [f64; 6],
    pub margin_ms: f64,
    pub max_work: [f64; 6],
    pub screen_range: [f64; 2],
    pub adapter: String,
}
impl Model {
    fn validate(&self, adapter: &str) -> Result<()> {
        ensure!(self.adapter == adapter, "model adapter mismatch");
        ensure!(
            self.benefit
                .iter()
                .chain(&self.baseline)
                .chain(&self.max_work)
                .chain(&self.screen_range)
                .all(|v| v.is_finite()),
            "nonfinite model"
        );
        ensure!(
            self.margin_ms.is_finite()
                && self.margin_ms >= 0.
                && self.screen_range[0] > 0.
                && self.screen_range[1] >= self.screen_range[0]
                && self.max_work.iter().all(|v| *v >= 0.),
            "invalid model domain"
        );
        Ok(())
    }
    pub fn depth(&self, f: [f64; 6]) -> bool {
        if f.iter().any(|v| !v.is_finite() || *v < 0.)
            || f[1] < self.screen_range[0]
            || f[1] > self.screen_range[1]
            || (2..6).any(|i| f[i] > self.max_work[i] * 1.25)
        {
            return false;
        }
        let dot = |c: [f64; 6]| c.into_iter().zip(f).map(|(a, b)| a * b).sum::<f64>();
        dot(self.benefit) > self.margin_ms.max(0.05 * dot(self.baseline).max(0.))
    }
}
pub(super) fn features(a: &Asset, cam: &Camera, triangles: u32, native: bool) -> [f64; 6] {
    let area = cost::area(a, cam);
    let t = triangles as f64 / 1e6;
    [
        1.,
        cam.size[0] as f64 * cam.size[1] as f64 / 1e6,
        t,
        (t * area).sqrt(),
        area,
        if native { area } else { 0. },
    ]
}
fn decision(a: &Asset, cam: &Camera) -> (render::DrawSelection, [f64; 6]) {
    let s = render::selection(a, cam, 5);
    let native = a
        .sealed
        .last()
        .is_some_and(|l| s.1 == 1 && s.0[0] == (l.first, l.count));
    let f = features(a, cam, s.2, native);
    (s, f)
}
fn asset(f: usize, d: usize) -> Result<Asset> {
    Ok(serde_json::from_slice(&fs::read(
        Path::new(SOURCE).join(format!("f{f}_d{d}.json")),
    )?)?)
}
pub(super) fn load(renderer: &mut Renderer) -> Result<()> {
    let model: Model = serde_json::from_slice(&fs::read(Path::new(OUTPUT).join("model.json"))?)?;
    model.validate(&renderer.gpu.adapter)?;
    renderer.shading_model = Some(model);
    Ok(())
}
pub(super) fn run(mode: &str) -> Result<()> {
    fs::create_dir_all(OUTPUT)?;
    let mut renderer = pollster::block_on(Renderer::new_automatic())?;
    if mode == "--capture-automatic-shading" {
        load(&mut renderer)?;
        return capture(&mut renderer);
    }
    let extend = mode == "--extend-domain-automatic-shading";
    let calibration = mode == "--calibrate-automatic-shading" || extend;
    let repeat = mode == "--repeat-automatic-shading";
    if !calibration {
        load(&mut renderer)?;
    }
    let codes: &[u32] = if calibration {
        &[5, 85]
    } else {
        &[0, 5, 85, 95]
    };
    let distances: &[f32] = if extend {
        &[12.]
    } else if calibration {
        &[16., 24., 48., 96., 192.]
    } else {
        &DISTANCES
    };
    let name = if calibration {
        "calibration"
    } else if repeat {
        "repeat"
    } else {
        "validation"
    };
    let mut rows: Vec<serde_json::Value> = if extend {
        serde_json::from_slice(&fs::read(Path::new(OUTPUT).join("calibration.json"))?)?
    } else {
        Vec::new()
    };
    if extend {
        ensure!(
            rows.len() == 20,
            "domain extension expects initial calibration"
        );
    }
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
            for &distance in distances {
                if repeat && !(distance == 14. || distance == 224.) {
                    continue;
                }
                let cam = render::camera_at(distance, size);
                let selected = assets.iter().map(|a| decision(a, &cam)).collect::<Vec<_>>();
                let selector_us = selected
                    .iter()
                    .zip(&assets)
                    .map(|(s, a)| {
                        let start = Instant::now();
                        for _ in 0..10000 {
                            let f = features(
                                std::hint::black_box(a),
                                std::hint::black_box(&cam),
                                s.0.2,
                                s.1[5] > 0.,
                            );
                            std::hint::black_box(
                                renderer.shading_model.as_ref().is_some_and(|m| m.depth(f)),
                            );
                        }
                        start.elapsed().as_secs_f64() * 1e6 / 10000.
                    })
                    .collect::<Vec<_>>();
                let timings = renderer.bench_cost(&inputs, distance, size, codes)?;
                rows.push(json!({"family":family,"resolution":resolution,"size":size,"distance":distance,"codes":codes,"timings":timings,"selector_us":selector_us,"features":selected.iter().map(|s|s.1).collect::<Vec<_>>(),"decisions":selected.iter().map(|s|if renderer.shading_model.as_ref().is_some_and(|m|m.depth(s.1)){85}else{5}).collect::<Vec<_>>(),"triangles":selected.iter().map(|s|s.0.2).collect::<Vec<_>>(),"adapter":renderer.gpu.adapter}));
                fs::write(
                    Path::new(OUTPUT).join(format!("{name}.json")),
                    serde_json::to_vec_pretty(&rows)?,
                )?;
                println!("{name}: family{family} resolution{resolution} distance{distance}");
            }
        }
    }
    Ok(())
}
fn capture(renderer: &mut Renderer) -> Result<()> {
    let root = Path::new(OUTPUT);
    let mut rows = Vec::new();
    let mut transitions = Vec::new();
    for family in 0..2 {
        for density in 0..4 {
            let a = asset(family, density)?;
            let u = renderer.upload_from(&a, SOURCE)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                let mut previous = [Vec::new(), Vec::new(), Vec::new()];
                for frame in 0..19 {
                    let distance = DISTANCES[frame.min(18 - frame)];
                    let cam = render::camera_at(distance, size);
                    let (s, f) = decision(&a, &cam);
                    let code = if renderer.shading_model.as_ref().unwrap().depth(f) {
                        85
                    } else {
                        5
                    };
                    let id = format!("f{family}_d{density}_r{resolution}_{frame}");
                    let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
                    let (old, _) = renderer.capture(&u, &a, &cam, 5)?;
                    let (depth, _) = renderer.capture(&u, &a, &cam, 85)?;
                    let (auto, _) = renderer.capture(&u, &a, &cam, 95)?;
                    ensure!(
                        &auto == if code == 85 { &depth } else { &old },
                        "chosen image mismatch"
                    );
                    let mut checks = Vec::new();
                    for (i, image) in [&old, &depth, &auto].into_iter().enumerate() {
                        let (q, r) = quality(&full, image, &previous[i]);
                        previous[i] = r;
                        checks.push(q);
                    }
                    // Reuse captures on the return path; exact pixels are still checked by rendering.
                    for (i, image) in [&full, &old, &depth, &auto].into_iter().enumerate() {
                        save(&root.join(format!("{id}_{i}.png")), image, size[0])?;
                    }
                    rows.push(json!({"id":id,"family":family,"density":density,"resolution":resolution,"size":size,"frame":frame,"distance":distance,"factor":a.factor,"checks":checks,"chosen":code,"chosen_image_identical":true,"triangles":[a.levels.last().unwrap().count/3,s.2,s.2,s.2],"covered_pixels":full.iter().filter(|p|p[3]>0.).count()}));
                }
                // Freeze crossing poses from the decision alone, before evaluating image errors.
                let mut last = None;
                for step in 0..421 {
                    let distance = 14. + step as f32 * 0.5;
                    let cam = render::camera_at(distance, size);
                    let code = renderer
                        .shading_model
                        .as_ref()
                        .unwrap()
                        .depth(decision(&a, &cam).1);
                    if last.is_some_and(|v| v != code) {
                        for offset in [-0.51, -0.01, 0.01] {
                            let d = distance + offset;
                            let cam = render::camera_at(d, size);
                            let (full, _) = renderer.capture(&u, &a, &cam, 0)?;
                            let (image, _) = renderer.capture(&u, &a, &cam, 95)?;
                            let chosen = if renderer
                                .shading_model
                                .as_ref()
                                .unwrap()
                                .depth(decision(&a, &cam).1)
                            {
                                85
                            } else {
                                5
                            };
                            let (direct, _) = renderer.capture(&u, &a, &cam, chosen)?;
                            ensure!(image == direct, "transition path mismatch");
                            transitions.push(json!({"family":family,"density":density,"resolution":resolution,"size":size,"distance":d,"chosen":chosen,"quality":quality(&full,&image,&[]).0,"chosen_image_identical":true}));
                        }
                    }
                    last = Some(code);
                }
            }
            fs::write(root.join("quality.json"), serde_json::to_vec_pretty(&rows)?)?;
            fs::write(
                root.join("transitions.json"),
                serde_json::to_vec_pretty(&transitions)?,
            )?;
            println!("quality family{family} density{density}");
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conservative_fallback_for_uncertainty_domain_and_invalid_models() {
        let mut m = Model {
            benefit: [0.; 6],
            baseline: [0.1, 0., 0., 0., 0., 0.],
            margin_ms: 0.01,
            max_work: [10.; 6],
            screen_range: [1., 8.],
            adapter: "test".into(),
        };
        let f = [1., 4., 0.5, 1., 2., 0.];
        assert!(!m.depth(f));
        m.benefit[0] = m.margin_ms;
        assert!(!m.depth(f), "equality at uncertainty margin keeps old path");
        m.benefit[0] = 0.02;
        assert!(m.depth(f));
        m.baseline[0] = 1.;
        assert!(!m.depth(f), "absolute gain must also clear relative margin");
        m.baseline[0] = 0.1;
        assert!(!m.depth([1., 9., 0.5, 1., 2., 0.]));
        assert!(!m.depth([1., 4., 99., 1., 2., 0.]));
        assert!(!m.depth([1., 4., f64::NAN, 1., 2., 0.]));
        assert!(m.validate("other").is_err());
        m.margin_ms = -1.;
        assert!(m.validate("test").is_err());
    }
}
