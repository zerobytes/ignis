//! Paired ablations, unchanged appearance, and a deliberately non-causal visibility oracle.
use super::*;
pub(super) const ROOT: &str = "artifacts/appearance-probe/regional-efficiency";
pub(super) const MODES: [&str; 6] = [
    "full",
    "regional",
    "fast",
    "grouped_full",
    "grouped_regional",
    "oracle_full",
];

pub(super) fn fast_bound(
    cam: &Camera,
    r: &Region,
    o: &Object,
    view: [Vec3; 2],
    origin: Vec3,
) -> [f32; 2] {
    if r.count < 24 || matches!(r.material, 2..=4) {
        return [f32::INFINITY; 2];
    }
    let center = origin + view[0] * o.scale;
    let rounding = 32.
        * f32::EPSILON
        * (cam.eye.abs().max_element()
            + o.position.abs().max_element()
            + r.max.abs().max_element() * o.scale
            + 1.);
    let half = view[1] * o.scale + Vec3::splat(rounding);
    let depth = -center.z - half.z;
    if depth <= 0.1 || !depth.is_finite() {
        return [f32::INFINITY; 2];
    }
    if depth < 4. {
        return bound(cam, r, o);
    }
    let xy = (center.abs() + half).truncate();
    // Algebraic support of a transformed AABB: max |x| = |center.x| + extent.x.
    // Small outward rounding protects the conservative bound; near decision/filter
    // boundaries use the original corner arithmetic to preserve its exact choice.
    let scale = cam.size[1] as f32 / (2. * 30f32.to_radians().tan()) / depth
        * (1. + xy.length_squared() / depth.powi(2)).sqrt()
        * o.scale
        * 1.00001;
    let b = [
        r.edge * scale,
        2f32.sqrt() * 4. * r.radius / SIDE as f32 * scale,
    ];
    let near_lod = [
        (std::f32::consts::FRAC_1_SQRT_2 * 0.25),
        (std::f32::consts::FRAC_1_SQRT_2 * 0.5),
        std::f32::consts::FRAC_1_SQRT_2,
    ]
    .iter()
    .any(|v| (b[1] - v).abs() < 0.0001);
    if b.iter()
        .any(|x| (*x - 0.75).abs() < 0.0001 || (*x - 1.).abs() < 0.0001)
        || near_lod
    {
        bound(cam, r, o)
    } else {
        b
    }
}

pub(super) fn group(plan: &mut Plan) {
    plan.draws
        .sort_unstable_by_key(|d| (d.indices.start, d.indices.end, d.instance));
    let mut write = 0;
    for read in 0..plan.draws.len() {
        let d = plan.draws[read].clone();
        debug_assert_eq!(d.count, 1);
        let instance = plan.instances[d.instance as usize];
        let slot = plan.instances.len() as u32;
        plan.instances.push(instance);
        if write > 0 && plan.draws[write - 1].indices == d.indices {
            plan.draws[write - 1].count += 1;
        } else {
            plan.draws[write] = Draw {
                indices: d.indices,
                instance: slot,
                count: 1,
            };
            write += 1;
        }
    }
    plan.draws.truncate(write);
    plan.counts.draw_calls = write + usize::from(!plan.proxies.is_empty());
    plan.counts.upload_bytes = plan.instances.len() * 32;
}

// One draw/region only for diagnostics. meta.w identifies the exact source region.
fn diagnostic(scene: &Scene, cam: &Camera) -> Plan {
    let mut p = Plan::new(scene.objects.len() * scene.regions.len() + 1);
    scene.plan(cam, false, &mut scene.states(), &mut p);
    p.draws.truncate(1);
    for (id, o) in scene.objects.iter().enumerate() {
        let [min, max] = scene.asset_bounds[o.asset];
        if !visible(cam, min * o.scale + o.position, max * o.scale + o.position) {
            continue;
        }
        for (rid, r) in scene.assets[o.asset].regions.iter().enumerate() {
            let slot = p.instances.len() as u32;
            p.instances.push(InstanceGpu {
                position: o.position.extend(o.scale).to_array(),
                meta: [
                    0,
                    0,
                    id as u32 + 1,
                    (id * scene.regions.len() + scene.region_offsets[o.asset] + rid + 1) as u32,
                ],
            });
            let first = scene.ranges[o.asset].start + r.first;
            p.draws.push(Draw {
                indices: first..first + r.count,
                instance: slot,
                count: 1,
            });
        }
    }
    p.counts.draw_calls = p.draws.len();
    p
}
fn oracle(scene: &Scene, cam: &Camera, mask: &[bool], p: &mut Plan) {
    scene.plan(cam, false, &mut [], p);
    p.draws.truncate(1);
    p.counts.triangles = (scene.ground.end - scene.ground.start) as u64 / 3;
    p.counts.geometry_regions = 0;
    for (id, o) in scene.objects.iter().enumerate() {
        let [min, max] = scene.asset_bounds[o.asset];
        if !visible(cam, min * o.scale + o.position, max * o.scale + o.position) {
            continue;
        }
        let mut run: Option<Range<u32>> = None;
        for (rid, r) in scene.assets[o.asset].regions.iter().enumerate() {
            if mask[id * scene.regions.len() + scene.region_offsets[o.asset] + rid + 1] {
                let first = scene.ranges[o.asset].start + r.first;
                if let Some(range) = &mut run {
                    range.end = first + r.count;
                } else {
                    run = Some(first..first + r.count);
                }
                p.counts.triangles += r.count as u64 / 3;
                p.counts.geometry_regions += 1;
            } else if let Some(indices) = run.take() {
                p.draws.push(Draw {
                    indices,
                    instance: id as u32 + 1,
                    count: 1,
                });
            }
        }
        if let Some(indices) = run {
            p.draws.push(Draw {
                indices,
                instance: id as u32 + 1,
                count: 1,
            });
        }
    }
    group(p);
}
pub(super) fn prepare(
    scene: &Scene,
    cam: &Camera,
    state: &mut [Vec<bool>],
    p: &mut Plan,
    mode: usize,
    mask: &[bool],
) {
    let start = Instant::now();
    if mode == 5 {
        oracle(scene, cam, mask, p);
    } else {
        scene.plan_variant(
            cam,
            matches!(mode, 1 | 2 | 4),
            state,
            p,
            matches!(mode, 2 | 4),
        );
        if mode >= 3 {
            group(p);
        }
    }
    p.counts.selection_us = start.elapsed().as_secs_f64() * 1e6;
}
fn assert_same(a: &[[f32; 4]], b: &[[f32; 4]], label: &str) -> Result<serde_json::Value> {
    let e = errors(a, b, a);
    ensure!(
        e["foreground_rmse"].as_f64().unwrap() < 0.00002
            && e["id_mismatch_pixels"].as_u64().unwrap() == 0,
        "{label}: {e}"
    );
    Ok(e)
}
pub(super) fn run() -> Result<()> {
    fs::create_dir_all(ROOT)?;
    let scene = Scene::new(assets(false)?);
    let mut renderer = pollster::block_on(Renderer::new(&scene))?;
    let root = Path::new(ROOT);
    let cap = scene.objects.len() * scene.regions.len() + 1;
    let mut plans = (0..6).map(|_| Plan::new(cap)).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
        let mut states = (0..6).map(|_| scene.states()).collect::<Vec<_>>();
        let mut previous_mask: Option<Vec<bool>> = None;
        for (frame, step) in (0..13).chain((0..13).rev()).enumerate() {
            let cam = camera(step as f32 / 12., size);
            let id = format!("r{resolution}_{frame:02}");
            for mode in 0..5 {
                prepare(&scene, &cam, &mut states[mode], &mut plans[mode], mode, &[]);
            }
            ensure!(
                states[1] == states[2] && states[1] == states[4],
                "selection changed at {id}"
            );
            let full = renderer.capture(&cam, &plans[0], false, [0.; 2])?;
            let old = renderer.capture(&cam, &plans[1], false, [0.; 2])?;
            let grouped_full = renderer.capture(&cam, &plans[3], false, [0.; 2])?;
            let optimized = renderer.capture(&cam, &plans[4], false, [0.; 2])?;
            let quality = json!({"grouped_full_vs_full":assert_same(&full,&grouped_full,"grouped full")?,"optimized_vs_old":assert_same(&old,&optimized,"optimized")?,"optimized_vs_full":errors(&full,&optimized,&full)});
            let map = renderer.capture(&cam, &plans[4], true, [0.; 2])?;
            save(&root.join(format!("{id}_full.png")), &full, size[0])?;
            save(
                &root.join(format!("{id}_optimized.png")),
                &optimized,
                size[0],
            )?;
            save(&root.join(format!("{id}_map.png")), &map, size[0])?;
            let mut oracle_report = serde_json::Value::Null;
            let mut timings = serde_json::Value::Null;
            if step % 3 == 0 {
                let begin = Instant::now();
                let diag = diagnostic(&scene, &cam);
                let mut image = renderer.capture_mode(&cam, &diag, 2, [0.; 2])?;
                let mut mask = vec![false; cap];
                for p in &mut image {
                    let k = p[3].round() as usize;
                    ensure!(k < cap, "region ID");
                    mask[k] = true;
                    p[3] = if k == 0 {
                        0.
                    } else {
                        ((k - 1) / scene.regions.len() + 1) as f32
                    };
                }
                let discovery_ms = begin.elapsed().as_secs_f64() * 1000.;
                let diagnostic_error = assert_same(&full, &image, "diagnostic split")?;
                prepare(&scene, &cam, &mut states[5], &mut plans[5], 5, &mask);
                let ideal = renderer.capture(&cam, &plans[5], false, [0.; 2])?;
                let same = assert_same(&full, &ideal, "oracle")?;
                save(&root.join(format!("{id}_oracle.png")), &ideal, size[0])?;
                let stale = if let Some(previous) = &previous_mask {
                    let mut p = Plan::new(cap);
                    prepare(&scene, &cam, &mut states[5], &mut p, 5, previous);
                    let stale = renderer.capture(&cam, &p, false, [0.; 2])?;
                    Some(errors(&full, &stale, &full))
                } else {
                    None
                };
                oracle_report = json!({"discovery_cpu_gpu_readback_scan_ms":discovery_ms,"diagnostic_draws":diag.counts.draw_calls,"visible_source_regions":mask.iter().skip(1).filter(|v|**v).count(),"diagnostic_error":diagnostic_error,"error_vs_full":same,"stale_previous_sample_error":stale,"counts":plans[5].counts});
                timings = renderer.bench_efficiency(&cam, &scene, &states, &mask)?;
                previous_mask = Some(mask);
            }
            println!(
                "{id}: draws {} -> {}, CPU {:.1} -> {:.1} us, quality {}",
                plans[1].counts.draw_calls,
                plans[4].counts.draw_calls,
                plans[1].counts.selection_us,
                plans[4].counts.selection_us,
                quality["optimized_vs_old"]["foreground_rmse"]
            );
            rows.push(json!({"id":id,"resolution":resolution,"size":size,"step":step,"frame":frame,"eye":cam.eye,"counts":plans[..5].iter().map(|p|&p.counts).collect::<Vec<_>>(),"quality":quality,"visible_regions":visible_regions(&map,&full,size),"oracle":oracle_report,"timings":timings}));
        }
    }
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":1,"adapter":renderer.gpu.adapter,"modes":MODES,"cache_bytes":renderer.cache_bytes,"rows":rows}),
        )?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analytic_support_matches_corner_bound_over_camera_and_scale() {
        let r = Region {
            min: Vec3::new(-0.2, -0.1, -0.3),
            max: Vec3::new(0.4, 0.3, 0.1),
            center: Vec3::new(0.1, 0.1, -0.1),
            radius: 0.35,
            edge: 0.02,
            first: 0,
            count: 300,
            material: 0,
        };
        for size in RESOLUTIONS {
            for step in 0..13 {
                let c = camera(step as f32 / 12., size);
                let abs = glam::Mat3::from_cols(
                    c.view.x_axis.truncate().abs(),
                    c.view.y_axis.truncate().abs(),
                    c.view.z_axis.truncate().abs(),
                );
                for distance in [0., 1., 10., 100., 400., 1000.] {
                    for scale in [0.75, 1., 2.] {
                        let o = Object {
                            asset: 0,
                            position: c.eye + Vec3::new(2., 0., distance),
                            scale,
                        };
                        let expected = bound(&c, &r, &o);
                        let actual = fast_bound(
                            &c,
                            &r,
                            &o,
                            [
                                c.view.transform_vector3(r.center),
                                abs * ((r.max - r.min) * 0.5),
                            ],
                            c.view.transform_point3(o.position),
                        );
                        for k in 0..2 {
                            if expected[k].is_finite() {
                                assert!(
                                    (actual[k] - expected[k]).abs() < expected[k] * 0.001 + 0.0001
                                );
                                assert!(actual[k] >= expected[k] * 0.999999);
                            } else {
                                assert_eq!(actual[k], expected[k]);
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn grouping_preserves_triangle_instance_pairs_and_proxy_range() {
        let mut p = Plan::new(12);
        for id in 0..4 {
            p.instances.push(InstanceGpu {
                position: [0.; 4],
                meta: [0, 0, id, 0],
            });
        }
        p.proxies = 3..4;
        p.draws = vec![
            Draw {
                indices: 0..6,
                instance: 1,
                count: 1,
            },
            Draw {
                indices: 9..12,
                instance: 0,
                count: 1,
            },
            Draw {
                indices: 0..6,
                instance: 2,
                count: 1,
            },
        ];
        group(&mut p);
        assert_eq!(p.draws.len(), 2);
        assert_eq!(p.proxies, 3..4);
        let d = &p.draws[0];
        assert_eq!(d.count, 2);
        assert_eq!(p.instances[d.instance as usize].meta[2], 1);
        assert_eq!(p.instances[d.instance as usize + 1].meta[2], 2);
    }
}
