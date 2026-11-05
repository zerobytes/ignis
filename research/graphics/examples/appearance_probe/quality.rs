//! Continuous hierarchical convergence before topology reduction, with subpixel budgets.
use super::{
    SIZE, geometry,
    gpu::{Case, FOV_Y, Gpu, Transition},
    hierarchy::{self, Plan, Projection, Tree},
    metrics,
    multiscale::filtered,
    reference, selection,
};
use anyhow::Result;
use glam::Mat4;
use serde_json::json;
use std::{fs, path::Path, time::Instant};

pub(super) const PROFILES: [(&str, f32); 3] = [
    ("Mais resumos", 1.0),
    ("Intermediário", 0.75),
    ("Mais geometria", 0.5),
];
fn smooth(x: f32) -> f32 {
    let t = x.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn weight(p: Projection, budget: f32, gradual: bool) -> f32 {
    if p.pixel.is_none() {
        return 0.;
    }
    let clearance = (p.min - p.min.floor())
        .min(p.min.floor() + glam::Vec2::ONE - p.max)
        .min_element();
    // Share the exact topology predicate; f32 smoothstep may round to1 early.
    if p.span() <= budget * 0.65 && clearance >= 0.06 {
        return 1.;
    }
    if !gradual {
        return 0.;
    }
    (smooth((budget - p.span()) / (budget * 0.35)) * smooth(clearance / 0.06))
        .min(1. - f32::EPSILON)
}
pub(super) struct Cut {
    pub(super) plan: Plan,
    pub(super) states: Vec<Transition>,
    pub(super) partial: usize,
}
impl Cut {
    pub(super) fn new(tree: &Tree) -> Self {
        let mut states = vec![
            Transition {
                parent: u32::MAX,
                weight: 0.,
                padding: [0; 2]
            };
            tree.nodes.len()
        ];
        for (id, n) in tree.nodes.iter().enumerate() {
            if let Some(children) = n.children {
                for c in children {
                    states[c].parent = id as u32;
                }
            }
        }
        Self {
            plan: Plan::new(tree),
            states,
            partial: 0,
        }
    }
    pub(super) fn update(&mut self, tree: &Tree, case: Case, budget: f32, gradual: bool) {
        assert!(budget.is_finite() && (0.5..=1.).contains(&budget));
        self.plan.full.clear();
        self.plan.proxy.clear();
        self.plan.summaries.clear();
        self.plan.leaves.clear();
        self.plan.visited = 0;
        self.plan.max_span = 0.;
        self.partial = 0;
        // Clear dormant descendants too, so profile changes have no stale state.
        for state in &mut self.states {
            state.weight = 0.;
        }
        let view = case.view_matrix();
        let focal = SIZE as f32 / (2. * (FOV_Y * 0.5).tan());
        self.visit(tree, 0, view, focal, budget, gradual);
        // Nearest active ancestor, in parent-before-child order. Zero-weight nodes
        // contribute no arithmetic; skipping them preserves the original mixture order.
        for id in 0..self.states.len() {
            let p = self.states[id].parent;
            self.states[id].padding[0] = if p == u32::MAX {
                u32::MAX
            } else if self.states[p as usize].weight > 0. {
                p
            } else {
                self.states[p as usize].padding[0]
            };
        }
    }
    fn visit(
        &mut self,
        tree: &Tree,
        id: usize,
        view: Mat4,
        focal: f32,
        budget: f32,
        gradual: bool,
    ) {
        let n = &tree.nodes[id];
        let p = hierarchy::project_with_focal(n.bounds, view, SIZE, focal);
        let w = weight(p, budget, gradual);
        self.states[id].weight = w;
        self.partial += usize::from(w > 0. && w < 1.);
        self.plan.visited += 1;
        if w == 1. {
            self.plan.summaries.push(id);
            self.plan.proxy.extend(id as u32 * 6..id as u32 * 6 + 6);
            self.plan.max_span = self.plan.max_span.max(p.span());
        } else if let Some(children) = n.children {
            for child in children {
                self.visit(tree, child, view, focal, budget, gradual);
            }
        } else {
            self.plan.leaves.push(id);
            self.plan.full.extend(n.full_start..n.full_start + 12);
        }
    }
    pub(super) fn validate(&self, tree: &Tree, case: Case, budget: f32, coverage: &mut [u8]) {
        coverage.fill(0);
        for &id in self.plan.summaries.iter().chain(&self.plan.leaves) {
            let n = &tree.nodes[id];
            for z in n.z..n.z + n.width {
                for x in n.x..n.x + n.width {
                    coverage[z * tree.n + x] += 1;
                }
            }
        }
        assert!(coverage.iter().all(|&v| v == 1));
        for (id, s) in self
            .states
            .iter()
            .enumerate()
            .filter(|(_, s)| s.weight > 0.)
        {
            let p = hierarchy::project(tree.nodes[id].bounds, case.view_matrix(), SIZE);
            assert!(p.pixel.is_some() && p.span() < budget);
            assert!(s.weight <= 1. && s.weight.is_finite());
        }
    }
}

// Coherent vertical error: signed RGB residual averaged down interior columns.
// This is a diagnostic, not a perceptual score or a guarantee of no stripes.
pub(super) fn column_step(truth: &[[f32; 4]], image: &[[f32; 4]]) -> f64 {
    let mut previous = None;
    let mut maximum = 0.0_f64;
    for x in 0..SIZE as usize {
        let mut sum = [0.0_f64; 3];
        let mut count = 0;
        for y in 0..SIZE as usize {
            let i = y * SIZE as usize + x;
            if truth[i][3] > 0.999 && image[i][3] > 0.999 {
                for (c, v) in sum.iter_mut().enumerate() {
                    *v += f64::from(image[i][c] - truth[i][c]);
                }
                count += 1;
            }
        }
        if count < 4 {
            previous = None;
            continue;
        }
        for v in &mut sum {
            *v /= count as f64;
        }
        if let Some(old) = previous {
            let old: [f64; 3] = old;
            maximum = maximum.max(
                (sum.iter()
                    .zip(old)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    / 3.)
                    .sqrt(),
            );
        }
        previous = Some(sum);
    }
    maximum
}

pub(super) type Trajectory = (&'static str, f32, f32, f32, Vec<f32>);
pub(super) fn trajectories() -> Vec<Trajectory> {
    [
        ("diffuse", 35., 8., 0.),
        ("glossy", 35., 96., 2.),
        ("grazing", 75., 96., 2.),
        ("transition", 35., 96., 2.),
    ]
    .into_iter()
    .map(|(name, tilt, exponent, specular)| {
        let mut d: Vec<f32> = if name == "transition" {
            (0..=24).map(|i| 10. + i as f32 / 24.).collect()
        } else {
            (0..=16)
                .map(|i| 2.5 + 13.5 * i as f32 / 16.)
                .chain([20., 24., 32., 48., 64.])
                .collect()
        };
        for i in (0..d.len() - 1).rev() {
            d.push(d[i]);
        }
        (name, tilt, exponent, specular, d)
    })
    .collect()
}

pub fn run() -> Result<()> {
    let output = Path::new("artifacts/appearance-probe/quality");
    fs::create_dir_all(output)?;
    let start = Instant::now();
    let mut gpu = pollster::block_on(Gpu::new())?;
    let mut mesh = geometry::generate(128);
    let tree = Tree::build(&mesh, 128);
    let base = gpu.upload(&mesh)?;
    // Only this uploaded experimental mesh uses leaf IDs on its full vertices.
    for (id, node) in tree.nodes.iter().enumerate().filter(|(_, n)| n.width == 1) {
        for v in &mut mesh.vertices[node.full_start as usize..node.full_start as usize + 12] {
            v.tile = id as u32;
        }
    }
    let mut spatial = gpu.upload_hierarchy(&mesh, &tree.proxy, &tree.lobes)?;
    let mut cut = Cut::new(&tree);
    let mut coverage = vec![0_u8; tree.n * tree.n];
    let mut rows = Vec::new();
    let paths = trajectories();
    for (name, tilt, exponent, specular, distances) in paths {
        let mut legacy = selection::Selector::default();
        let mut previous_truth: Option<Vec<[f32; 4]>> = None;
        let mut previous_images: Vec<Option<Vec<[f32; 4]>>> = vec![None; 6];
        for (frame, &distance) in distances.iter().enumerate() {
            let case = Case {
                distance,
                tilt,
                light_angle: 25.,
                exponent,
                specular,
                shift: 0.,
            };
            let id = format!("{name}_{frame:02}");
            let (truth, convergence) = reference(&mut gpu, &base, case)?;
            legacy.update(&mesh.bounds, case, SIZE);
            gpu.detailed = legacy.detailed;
            let mut controls = Vec::new();
            for (label, mode) in [("old", 4), ("full", 0), ("fixed", 2)] {
                let image = filtered(&mut gpu, &base, case, mode)?;
                metrics::png(&output.join(format!("{id}_{label}.png")), &image, SIZE)?;
                controls.push(json!({"method":label,"error":metrics::compare(&truth,&image),"column_step":column_step(&truth,&image)}));
            }
            metrics::png(&output.join(format!("{id}_reference.png")), &truth, SIZE)?;
            let mut methods = Vec::new();
            let mut counts = [0; 3];
            // Rotate order; each measurement includes native render warmup +31 samples.
            for offset in 0..6 {
                let index = (frame + offset) % 6;
                let profile = index / 2;
                let gradual = index % 2 == 0;
                let (_, budget) = PROFILES[profile];
                let label = if gradual { "smooth" } else { "hard" };
                let t = Instant::now();
                cut.update(&tree, case, budget, gradual);
                let cpu_us = t.elapsed().as_secs_f64() * 1e6;
                cut.validate(&tree, case, budget, &mut coverage);
                let t = Instant::now();
                gpu.select_indices(&mut spatial, &cut.plan.full, &cut.plan.proxy)?;
                if gradual {
                    gpu.set_transitions(&spatial, &cut.states)?;
                }
                let queue_us = t.elapsed().as_secs_f64() * 1e6;
                let mode = if gradual { 8 } else { 6 };
                let image = filtered(&mut gpu, &spatial, case, mode)?;
                let native = gpu.render(&spatial, case, mode, SIZE, 1, true)?;
                if gradual {
                    counts[profile] = cut.plan.triangles();
                }
                if (frame == 0 || frame == distances.len() - 1) && name != "transition" {
                    assert!(cut.plan.summaries.is_empty() && cut.partial == 0);
                    let full = filtered(&mut gpu, &base, case, 0)?;
                    anyhow::ensure!(
                        full == image,
                        "near pose must exactly restore original geometry"
                    );
                }
                let temporal = match (&previous_truth, &previous_images[index]) {
                    (Some(r), Some(c)) => metrics::temporal(r, c, &truth, &image),
                    _ => 0.,
                };
                let file = format!("{id}_p{profile}_{label}");
                metrics::png(&output.join(format!("{file}.png")), &image, SIZE)?;
                metrics::error_png(
                    &output.join(format!("{file}_error.png")),
                    &truth,
                    &image,
                    SIZE,
                )?;
                methods.push(json!({"profile":profile,"method":label,"file":file,"triangles":cut.plan.triangles(),"summaries":cut.plan.summaries.len(),"geometry_cells":cut.plan.leaves.len(),"partial_nodes":cut.partial,"levels":cut.plan.levels(&tree),"max_span":cut.plan.max_span,"cpu_us":cpu_us,"queue_us":queue_us,"upload_bytes":(cut.plan.full.len()+cut.plan.proxy.len())*4+if gradual {cut.states.len()*16} else {0},"error":metrics::compare(&truth,&image),"column_step":column_step(&truth,&image),"temporal":temporal,"native_error":metrics::compare(&truth,&native.pixels),"timing":native.timing}));
                previous_images[index] = Some(image);
            }
            assert!(
                counts[0] <= counts[1] && counts[1] <= counts[2],
                "quality must monotonically retain triangles"
            );
            let p = hierarchy::project(tree.nodes[0].bounds, case.view_matrix(), SIZE);
            rows.push(json!({"id":id,"case":name,"frame":frame,"parameters":case,"turn":distances.len()/2,"direction":if frame<=distances.len()/2 {"afastando"} else {"aproximando"},"screen_bounds":[p.min.x,p.min.y,p.max.x,p.max.y],"reference_residual":convergence,"controls":controls,"methods":methods}));
            previous_truth = Some(truth);
        }
        println!("{name}: {} poses validated", distances.len());
    }
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":1,"adapter":gpu.adapter,"profiles":PROFILES.map(|(name,budget)|json!({"name":name,"budget":budget})),"reference_spp":2048,"candidate_spp":128,"transition_bytes":tree.nodes.len()*16,"cases":rows,"seconds":start.elapsed().as_secs_f64()}),
        )?,
    )?;
    verify_endpoints()?;
    println!("quality: {:.1}s", start.elapsed().as_secs_f64());
    Ok(())
}

/// Verify actual GPU pixels before/after removing converged geometry.
pub fn verify_endpoints() -> Result<()> {
    let mut gpu = pollster::block_on(Gpu::new())?;
    let mut mesh = geometry::generate(128);
    let tree = Tree::build(&mesh, 128);
    for (id, node) in tree.nodes.iter().enumerate().filter(|(_, n)| n.width == 1) {
        for v in &mut mesh.vertices[node.full_start as usize..node.full_start as usize + 12] {
            v.tile = id as u32;
        }
    }
    let mut uploaded = gpu.upload_hierarchy(&mesh, &tree.proxy, &tree.lobes)?;
    let full: Vec<u32> = (0..mesh.vertices.len() as u32).collect();
    let mut cut = Cut::new(&tree);
    let mut results = Vec::new();
    for tilt in [35., 75.] {
        for (_, budget) in PROFILES {
            let case = Case {
                distance: 64.,
                tilt,
                light_angle: 25.,
                exponent: 96.,
                specular: 2.,
                shift: 0.,
            };
            cut.update(&tree, case, budget, true);
            gpu.set_transitions(&uploaded, &cut.states)?;
            gpu.select_indices(&mut uploaded, &cut.plan.full, &cut.plan.proxy)?;
            let reduced = filtered(&mut gpu, &uploaded, case, 8)?;
            // Same weights/camera, but keep all fine geometry: the endpoint before reduction.
            gpu.select_indices(&mut uploaded, &full, &[])?;
            let unreduced = filtered(&mut gpu, &uploaded, case, 8)?;
            let error = metrics::compare(&reduced, &unreduced);
            let max_alpha = reduced
                .iter()
                .zip(&unreduced)
                .map(|(a, b)| (a[3] - b[3]).abs())
                .fold(0.0_f32, f32::max);
            // Rasterization of different tessellations is not exactly equivalent on Metal.
            // Record this separately; do not turn a radiance endpoint proof into pixel equality.
            anyhow::ensure!(
                error.max_abs.is_finite() && max_alpha <= 2. / 128.,
                "coverage discrepancy exceeds two filtered samples: {error:?}, alpha {max_alpha}"
            );
            let mut bad = cut.states.clone();
            bad[1].parent = 1;
            anyhow::ensure!(
                gpu.set_transitions(&uploaded, &bad).is_err(),
                "cyclic state accepted"
            );
            bad = cut.states.clone();
            bad[1].weight = f32::NAN;
            anyhow::ensure!(
                gpu.set_transitions(&uploaded, &bad).is_err(),
                "NaN state accepted"
            );
            let after = filtered(&mut gpu, &uploaded, case, 8)?;
            anyhow::ensure!(after == unreduced, "invalid update changed GPU pixels");
            results.push(json!({"tilt":tilt,"budget":budget,"triangles":cut.plan.triangles(),"endpoint_error":error,"max_alpha_difference":max_alpha,"invalid_update_preserved_pixels":true}));
        }
    }
    fs::create_dir_all("artifacts/appearance-probe/quality")?;
    fs::write(
        "artifacts/appearance-probe/quality/endpoints.json",
        serde_json::to_vec_pretty(&results)?,
    )?;
    println!(
        "GPU topology residuals recorded; bounded coverage and atomic-error checks: {} passed",
        results.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    fn p(a: f32, b: f32) -> Projection {
        Projection {
            min: Vec2::splat(a),
            max: Vec2::splat(b),
            pixel: if a.floor() == b.floor() {
                Some([a as u32; 2])
            } else {
                None
            },
        }
    }
    #[test]
    fn continuous_weight_vanishes_at_pixel_and_size_boundaries() {
        assert_eq!(weight(p(0.2, 0.8), 1., true), 1.);
        assert_eq!(weight(p(0., 0.6), 1., true), 0.);
        assert_eq!(weight(p(-0.01, 0.59), 1., true), 0.);
        assert!(weight(p(0.00001, 0.60001), 1., true) < 1e-6);
        assert!(weight(p(0.1, 0.84999), 0.75, true) < 1e-6);
        for i in 0..1000 {
            let a = i as f32 / 1000.;
            assert!(
                (weight(p(a, a + 0.6), 1., true) - weight(p(a + 0.001, a + 0.601), 1., true)).abs()
                    < 0.026
            );
        }
    }
    #[test]
    fn cuts_partition_and_quality_changes_are_reversible() {
        let mesh = geometry::generate(16);
        let tree = Tree::build(&mesh, 16);
        let mut cut = Cut::new(&tree);
        let mut coverage = vec![0; 256];
        for distance in [2.5, 10., 32., 64., 128., 64., 10., 2.5] {
            let case = Case {
                distance,
                tilt: 35.,
                light_angle: 25.,
                exponent: 96.,
                specular: 2.,
                shift: 0.13,
            };
            let mut last = 0;
            for (_, b) in PROFILES {
                cut.update(&tree, case, b, true);
                cut.validate(&tree, case, b, &mut coverage);
                assert!(cut.plan.triangles() >= last);
                last = cut.plan.triangles();
                let states = cut.states.clone();
                let full = cut.plan.full.clone();
                let proxy = cut.plan.proxy.clone();
                cut.update(&tree, case, b, false);
                assert_eq!(full, cut.plan.full);
                assert_eq!(proxy, cut.plan.proxy);
                cut.update(&tree, case, 0.5, false);
                cut.update(&tree, case, b, true);
                assert_eq!(states, cut.states);
                assert_eq!(full, cut.plan.full);
                assert_eq!(proxy, cut.plan.proxy);
            }
        }
    }
}
