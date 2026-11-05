//! Cache per-node summary radiance; measure reuse and full preparation plus drawing.
use super::{
    SIZE, geometry,
    gpu::{Case, Gpu},
    hierarchy::{self, Tree},
    metrics,
    multiscale::filtered,
    quality::{Cut, PROFILES, column_step, trajectories},
    reference,
};
use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path, time::Instant};
pub fn run(smoke: bool) -> Result<()> {
    let root = Path::new("artifacts/appearance-probe/cost");
    fs::create_dir_all(root)?;
    let start = Instant::now();
    let mut gpu = pollster::block_on(Gpu::new())?;
    let mut mesh = geometry::generate(128);
    let tree = Tree::build(&mesh, 128);
    let base = gpu.upload(&mesh)?;
    for (id, n) in tree.nodes.iter().enumerate().filter(|(_, n)| n.width == 1) {
        for v in &mut mesh.vertices[n.full_start as usize..n.full_start as usize + 12] {
            v.tile = id as u32;
        }
    }
    let mut spatial = gpu.upload_hierarchy(&mesh, &tree.proxy, &tree.lobes)?;
    let full_order: Vec<u32> = tree
        .nodes
        .iter()
        .filter(|n| n.width == 1)
        .flat_map(|n| n.full_start..n.full_start + 12)
        .collect();
    gpu.set_unreduced_order(&mut spatial, &full_order)?;
    let mut cut = Cut::new(&tree);
    let mut coverage = vec![0; 128 * 128];
    let cache_checks = verify_cache(&mut gpu, &mesh, &tree, &mut spatial)?;
    let mut rows = Vec::new();
    for (scene, tilt, exponent, specular, distances) in trajectories() {
        if smoke && scene != "glossy" {
            continue;
        }
        for (frame, &distance) in distances.iter().enumerate() {
            if smoke && ![0, 9, 16, 21].contains(&frame) {
                continue;
            }
            let case = Case {
                distance,
                tilt,
                light_angle: 25.,
                exponent,
                specular,
                shift: 0.,
            };
            let id = format!("{scene}_{frame:02}");
            let (truth, residual) = if smoke {
                (Vec::new(), None)
            } else {
                let (t, r) = reference(&mut gpu, &base, case)?;
                (t, Some(r))
            };
            if !smoke {
                metrics::png(&root.join(format!("{id}_reference.png")), &truth, SIZE)?;
            }
            let mut methods = Vec::new();
            for (profile, (_, budget)) in PROFILES.into_iter().enumerate() {
                let now = Instant::now();
                cut.update(&tree, case, budget, true);
                let cpu_us = now.elapsed().as_secs_f64() * 1e6;
                cut.validate(&tree, case, budget, &mut coverage);
                let now = Instant::now();
                gpu.select_indices(&mut spatial, &cut.plan.full, &cut.plan.proxy)?;
                gpu.set_transitions(&spatial, &cut.states)?;
                let queue_us = now.elapsed().as_secs_f64() * 1e6;
                let old = filtered(&mut gpu, &spatial, case, 8)?;
                let first = gpu.render(&spatial, case, 9, SIZE, 1, false)?;
                let prepared = first.prepared;
                let cached = filtered(&mut gpu, &spatial, case, 9)?;
                let delta = metrics::compare(&old, &cached);
                anyhow::ensure!(
                    delta.max_abs < 1e-5 && delta.alpha_rmse == 0.,
                    "changed appearance: {delta:?}"
                );
                let timings = gpu.benchmark_comparison(&spatial, case, SIZE)?;
                if !smoke {
                    metrics::png(
                        &root.join(format!("{id}_p{profile}_before.png")),
                        &old,
                        SIZE,
                    )?;
                    metrics::png(
                        &root.join(format!("{id}_p{profile}_after.png")),
                        &cached,
                        SIZE,
                    )?;
                    metrics::error_png(
                        &root.join(format!("{id}_p{profile}_error.png")),
                        &truth,
                        &cached,
                        SIZE,
                    )?;
                }
                println!(
                    "{id} p{profile}: {} tris, old {:.4} / new {:.4} / full {:.4} / unreduced {:.4} ms, Δmax {}",
                    cut.plan.triangles(),
                    timings[0].gpu_ms_median.unwrap_or(0.),
                    timings[1].gpu_ms_median.unwrap_or(0.),
                    timings[2].gpu_ms_median.unwrap_or(0.),
                    timings[3].gpu_ms_median.unwrap_or(0.),
                    delta.max_abs
                );
                methods.push(json!({"profile":profile,"triangles":cut.plan.triangles(),"cpu_us":cpu_us,"queue_us":queue_us,"prepared":prepared,"active_nodes":cut.states.iter().filter(|s|s.weight>0.).count(),"delta":delta,"error":if smoke{None}else{Some(metrics::compare(&truth,&cached))},"column_step":if smoke{None}else{Some(column_step(&truth,&cached))},"before":timings[0],"after":timings[1],"full":timings[2],"unreduced":timings[3],"cold":timings[4]}));
            }
            let p = hierarchy::project(tree.nodes[0].bounds, case.view_matrix(), SIZE);
            rows.push(json!({"id":id,"case":scene,"frame":frame,"turn":distances.len()/2,"parameters":case,"screen_bounds":[p.min.x,p.min.y,p.max.x,p.max.y],"reference_residual":residual,"methods":methods}));
        }
    }
    fs::write(
        root.join(if smoke { "smoke.json" } else { "report.json" }),
        serde_json::to_vec_pretty(
            &json!({"schema":1,"cache_checks":cache_checks,"adapter":gpu.adapter,"profiles":PROFILES.map(|(name,budget)|json!({"name":name,"budget":budget})),"reference_spp":2048,"candidate_spp":128,"evaluated_bytes":tree.nodes.len()*16,"cases":rows,"seconds":start.elapsed().as_secs_f64()}),
        )?,
    )?;
    Ok(())
}

fn verify_cache(
    gpu: &mut Gpu,
    mesh: &geometry::Mesh,
    tree: &Tree,
    spatial: &mut super::gpu::Uploaded,
) -> Result<Vec<serde_json::Value>> {
    let original = Case {
        distance: 16.,
        tilt: 35.,
        light_angle: 25.,
        exponent: 96.,
        specular: 2.,
        shift: 0.,
    };
    let light = Case {
        light_angle: 48.,
        ..original
    };
    let view = Case { tilt: 65., ..light };
    let exponent = Case {
        exponent: 48.,
        ..view
    };
    let amplitude = Case {
        specular: 0.5,
        ..exponent
    };
    let cases = [
        ("first", original, 1., true),
        (
            "distance",
            Case {
                distance: 32.,
                ..original
            },
            1.,
            false,
        ),
        (
            "translation",
            Case {
                shift: 0.1,
                ..original
            },
            1.,
            false,
        ),
        ("profile", original, 0.5, false),
        ("light", light, 1., true),
        ("view", view, 1., true),
        ("exponent", exponent, 1., true),
        ("amplitude", amplitude, 1., true),
        ("return", original, 1., true),
    ];
    let mut cut = Cut::new(tree);
    let mut checks = Vec::new();
    for (label, case, budget, expected) in cases {
        cut.update(tree, case, budget, true);
        gpu.select_indices(spatial, &cut.plan.full, &cut.plan.proxy)?;
        gpu.set_transitions(spatial, &cut.states)?;
        let first = gpu.render(spatial, case, 9, SIZE, 1, false)?;
        anyhow::ensure!(first.prepared == expected, "wrong cache decision: {label}");
        let warm = gpu.render(spatial, case, 9, SIZE, 1, false)?;
        anyhow::ensure!(
            !warm.prepared && first.pixels == warm.pixels,
            "unstable cache: {label}"
        );
        let before = filtered(gpu, spatial, case, 8)?;
        let after = filtered(gpu, spatial, case, 9)?;
        let error = metrics::compare(&before, &after);
        anyhow::ensure!(
            error.max_abs < 1e-5 && error.alpha_rmse == 0.,
            "stale radiance: {label} {error:?}"
        );
        let mut bad = cut.states.clone();
        bad[1].padding[0] = 1;
        anyhow::ensure!(
            gpu.set_transitions(spatial, &bad).is_err(),
            "invalid shortcut accepted"
        );
        let unchanged = gpu.render(spatial, case, 9, SIZE, 1, false)?;
        anyhow::ensure!(
            !unchanged.prepared && unchanged.pixels == warm.pixels,
            "failed mutation changed cache"
        );
        checks.push(
            json!({"case":label,"prepared":first.prepared,"delta":error,"atomic_error":true}),
        );
    }
    // A different uploaded distribution owns a different cache, even at identical light/view.
    let mut lobes = tree.lobes.clone();
    for l in &mut lobes {
        for c in &mut l.color[..3] {
            *c *= 0.5;
        }
    }
    let mut second = gpu.upload_hierarchy(mesh, &tree.proxy, &lobes)?;
    gpu.select_indices(&mut second, &cut.plan.full, &cut.plan.proxy)?;
    gpu.set_transitions(&second, &cut.states)?;
    let first = gpu.render(&second, original, 9, SIZE, 1, false)?;
    anyhow::ensure!(first.prepared, "shared stale mesh cache");
    let expected = gpu.render(&second, original, 8, SIZE, 1, false)?;
    let error = metrics::compare(&expected.pixels, &first.pixels);
    anyhow::ensure!(
        error.max_abs < 1e-5 && error.alpha_rmse == 0.,
        "wrong mesh distribution"
    );
    let restored = gpu.render(spatial, original, 9, SIZE, 1, false)?;
    anyhow::ensure!(!restored.prepared, "other mesh invalidated original");
    checks.push(json!({"case":"separate_mesh","prepared":true,"delta":error}));
    // Render resolution and jitter do not enter the directional radiance key.
    gpu.jitter = [0.25, -0.25];
    let high = gpu.render(spatial, original, 9, 512, 1, false)?;
    let expected = gpu.render(spatial, original, 8, 512, 1, false)?;
    gpu.jitter = [0.; 2];
    let error = metrics::compare(&expected.pixels, &high.pixels);
    anyhow::ensure!(
        !high.prepared && error.max_abs < 1e-5 && error.alpha_rmse == 0.,
        "resize/jitter changed cached response"
    );
    checks.push(json!({"case":"resolution_jitter","prepared":false,"delta":error}));
    Ok(checks)
}
