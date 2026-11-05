//! Multi-resolution spatial summaries, evaluated against the same camera and light.
use super::{
    SIZE, geometry,
    gpu::{Case, Gpu},
    hierarchy::{self, Plan, Selector, Tree},
    metrics, reference, selection,
};
use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path, time::Instant};

// Same 128-sample integration for every candidate; selection stays at output resolution.
pub(super) fn filtered(
    gpu: &mut Gpu,
    mesh: &super::gpu::Uploaded,
    case: Case,
    mode: usize,
) -> Result<Vec<[f32; 4]>> {
    let mut sum = vec![[0.0_f32; 4]; (SIZE * SIZE) as usize];
    for pass in 0..8 {
        gpu.jitter = [
            super::halton(pass + 1, 2) - 0.5,
            super::halton(pass + 1, 3) - 0.5,
        ];
        let image = gpu.render(mesh, case, mode, SIZE, 4, false)?.pixels;
        for (out, pixel) in sum.iter_mut().zip(image) {
            for c in 0..4 {
                out[c] += pixel[c] / 8.0;
            }
        }
    }
    gpu.jitter = [0.0; 2];
    Ok(sum)
}

pub fn run() -> Result<()> {
    let output = Path::new("artifacts/appearance-probe/multiscale");
    fs::create_dir_all(output)?;
    let start = Instant::now();
    let mut gpu = pollster::block_on(Gpu::new())?;
    let mesh = geometry::generate(128);
    let tree = Tree::build(&mesh, 128);
    let base = gpu.upload(&mesh)?;
    let mut spatial = gpu.upload_hierarchy(&mesh, &tree.proxy, &tree.lobes)?;
    let mut distances: Vec<f32> = (0..=24).map(|i| 2.5 + 13.5 * i as f32 / 24.).collect();
    distances.extend([20., 24., 28., 32., 40., 48., 56., 64.]);
    let turn = distances.len() - 1;
    for i in (0..turn).rev() {
        distances.push(distances[i]);
    }
    let mut rows = Vec::new();
    let mut coverage = vec![0_u8; tree.n * tree.n];
    for (name, tilt, exponent, specular) in [
        ("diffuse", 35., 8., 0.),
        ("glossy", 35., 96., 2.),
        ("grazing", 75., 96., 2.),
    ] {
        let mut selector = Selector::new(&tree);
        let mut legacy = selection::Selector::default();
        let mut plan = Plan::new(&tree);
        let mut previous_plan = Plan::new(&tree);
        let mut previous_images = None;
        for (frame, &distance) in distances.iter().enumerate() {
            let case = Case {
                distance,
                tilt,
                light_angle: 25.,
                exponent,
                specular,
                shift: 0.,
            };
            let select_start = Instant::now();
            selector.update(&tree, case, SIZE, &mut plan);
            let selection_us = select_start.elapsed().as_secs_f64() * 1e6;
            let update_start = Instant::now();
            gpu.select_indices(&mut spatial, &plan.full, &plan.proxy)?;
            let index_submit_us = update_start.elapsed().as_secs_f64() * 1e6;
            // Exhaustive offline validation is outside the measured selection path.
            coverage.fill(0);
            for &id in plan.summaries.iter().chain(&plan.leaves) {
                let n = &tree.nodes[id];
                for z in n.z..n.z + n.width {
                    for x in n.x..n.x + n.width {
                        coverage[z * tree.n + x] += 1;
                    }
                }
            }
            anyhow::ensure!(
                coverage.iter().all(|n| *n == 1),
                "tree cut does not partition source cells"
            );
            let view = case.view_matrix();
            for &id in &plan.summaries {
                let p = hierarchy::project(tree.nodes[id].bounds, view, SIZE);
                anyhow::ensure!(
                    p.pixel.is_some() && p.span() < 1.,
                    "selected summary crosses pixel boundary"
                );
            }
            legacy.update(&mesh.bounds, case, SIZE);
            gpu.detailed = legacy.detailed;
            let (truth, convergence) = reference(&mut gpu, &base, case)?;
            let full = filtered(&mut gpu, &base, case, 0)?;
            let fixed = filtered(&mut gpu, &base, case, 2)?;
            let old = filtered(&mut gpu, &base, case, 4)?;
            let multiscale = filtered(&mut gpu, &spatial, case, 6)?;
            let diagnostic = gpu.render(&spatial, case, 7, SIZE, 4, false)?.pixels;
            if plan.summaries.is_empty() {
                anyhow::ensure!(full == multiscale, "indexed full geometry changed pixels");
            }
            if frame == 0 || frame == distances.len() - 1 {
                anyhow::ensure!(
                    plan.summaries.is_empty(),
                    "near pose did not recover all geometry"
                );
            }
            let selection_changed =
                plan.full != previous_plan.full || plan.proxy != previous_plan.proxy;
            let switch_rmse = if frame > 0 && selection_changed {
                gpu.select_indices(&mut spatial, &previous_plan.full, &previous_plan.proxy)?;
                let before = filtered(&mut gpu, &spatial, case, 6)?;
                gpu.select_indices(&mut spatial, &plan.full, &plan.proxy)?;
                metrics::compare(&before, &multiscale).rmse_surface
            } else {
                0.
            };
            // Alternate order to reduce fixed method-order bias; all native resolution.
            let mut native: Vec<Option<super::gpu::Capture>> = (0..4).map(|_| None).collect();
            for offset in 0..4 {
                let method = (frame + offset) % 4;
                let (mesh, mode) = match method {
                    0 => (&base, 0),
                    1 => (&base, 2),
                    2 => (&base, 4),
                    _ => (&spatial, 6),
                };
                native[method] = Some(gpu.render(mesh, case, mode, SIZE, 1, true)?);
            }
            let native = native
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .expect("all methods captured");
            let temporal =
                previous_images
                    .as_ref()
                    .map_or(0., |(r, c): &(Vec<[f32; 4]>, Vec<[f32; 4]>)| {
                        metrics::temporal(r, c, &truth, &multiscale)
                    });
            let id = format!("{name}_{frame:02}");
            for (label, image) in [
                ("reference", &truth),
                ("full", &full),
                ("fixed", &fixed),
                ("previous", &old),
                ("multiscale", &multiscale),
                ("diagnostic", &diagnostic),
            ] {
                metrics::png(&output.join(format!("{id}_{label}.png")), image, SIZE)?;
            }
            metrics::error_png(
                &output.join(format!("{id}_error.png")),
                &truth,
                &multiscale,
                SIZE,
            )?;
            let methods=[("full",&full),("fixed",&fixed),("previous",&old),("multiscale",&multiscale)].into_iter().enumerate().map(|(i,(name,image))|json!({"method":name,"filtered_error":metrics::compare(&truth,image),"native_error":metrics::compare(&truth,&native[i].pixels),"native_timing":native[i].timing})).collect::<Vec<_>>();
            let screen_bounds = hierarchy::project(tree.nodes[0].bounds, view, SIZE);
            rows.push(json!({"screen_bounds":[screen_bounds.min.x,screen_bounds.min.y,screen_bounds.max.x,screen_bounds.max.y],"id":id,"case":name,"frame":frame,"direction":if frame<=turn {"afastando"} else {"aproximando"},"parameters":case,
                "triangles":plan.triangles(),"geometry_cells":plan.leaves.len(),"summary_nodes":plan.summaries.len(),"summaries_by_cell_width":plan.levels(&tree),
                "draw_calls":usize::from(!plan.full.is_empty())+usize::from(!plan.proxy.is_empty()),"visited_nodes":plan.visited,"max_summary_span_px":plan.max_span,
                "selection_cpu_us":selection_us,"index_validation_and_queue_us":index_submit_us,"index_upload_bytes":(plan.full.len()+plan.proxy.len())*4,
                "previous_triangles":legacy.detailed.iter().filter(|v|**v).count()*1024 + legacy.detailed.iter().filter(|v|!**v).count()*2,
                "selection_changed":selection_changed,"switch_same_pose_rmse":switch_rmse,"temporal_residual_rmse":temporal,"reference_half_vs_full":convergence,"methods":methods}));
            previous_images = Some((truth, multiscale));
            std::mem::swap(&mut plan, &mut previous_plan);
        }
        println!(
            "{name}: {} frames, partition/pixel containment/near-return checks passed",
            distances.len()
        );
    }
    let memory = json!({"node_metadata_bytes":std::mem::size_of_val(tree.nodes.as_slice()),"distribution_bytes":std::mem::size_of_val(tree.lobes.as_slice()),"proxy_vertex_bytes":std::mem::size_of_val(tree.proxy.as_slice()),"full_vertex_bytes":std::mem::size_of_val(mesh.vertices.as_slice()),"gpu_index_capacity_bytes":(mesh.vertices.len()+tree.proxy.len())*4});
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":1,"adapter":gpu.adapter,"image_size":SIZE,"cells_per_side":tree.n,"tree_nodes":tree.nodes.len(),"tree_build_ms":tree.build_ms,"hierarchy_upload_and_wait_ms":spatial.upload_ms,
        "memory":memory,"turn_frame":turn,"frames_per_case":distances.len(),"reference_samples_per_pixel":2048,"candidate_samples_per_pixel":128,"benchmark_samples_per_pixel":1,
        "gate":"All eight corners of complete node AABB (including microheight) project inside a single final-resolution pixel; conservative f32 margin. Enter below 0.75px max extent, retain below 1px only while containment remains true.",
        "limits":"Fixed 128x128 procedural cells, resident quadtree+mesh, CPU selection and index upload. Planar proxy approximates coverage/occlusion; pixel containment is not radiance or visibility equivalence. No crossfade/TAA/culling/streaming/GI.",
        "cases":rows,"total_seconds":start.elapsed().as_secs_f64()}),
        )?,
    )?;
    println!(
        "{} in {:.1}s",
        output.join("report.json").display(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
