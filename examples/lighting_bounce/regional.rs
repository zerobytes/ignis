//! Conservative support for a finite sampled bake, with private repair/publication.
use super::*;
const SIDE: u32 = 32;
const SEED: u32 = 317;
fn copy(r: &Renderer, items: &[(&wgpu::Buffer, &wgpu::Buffer, u64)]) {
    let mut e = r.device.create_command_encoder(&Default::default());
    for (a, b, n) in items {
        e.copy_buffer_to_buffer(a, 0, b, 0, *n);
    }
    r.queue.submit([e.finish()]);
}
fn snapshot(r: &Renderer, b: &wgpu::Buffer, n: u64) -> wgpu::Buffer {
    let saved = buffer(
        r,
        "regional replay snapshot",
        n,
        wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
    );
    copy(r, &[(b, &saved, n)]);
    saved
}
fn publish(r: &Renderer, cache: &Lab, view: &Lab, n: u64) -> Result<(f64, f64)> {
    let t = Instant::now();
    let mut e = r.device.create_command_encoder(&Default::default());
    cache.pass(
        &mut e,
        &cache.begin_batch,
        &cache.groups[1],
        [Some(0), None],
        Dispatch::Grid(1, 1, 1),
    );
    e.copy_buffer_to_buffer(&cache.probes[1], 0, &view.probes[1], 0, n * 16);
    e.copy_buffer_to_buffer(&cache.guides, 0, &view.guides, 0, n * 4);
    cache.pass(
        &mut e,
        &cache.begin_batch,
        &cache.groups[1],
        [None, Some(1)],
        Dispatch::Grid(1, 1, 1),
    );
    r.queue.submit([e.finish()]);
    let cpu = ms(t);
    wait(r)?;
    let mut e = r.device.create_command_encoder(&Default::default());
    e.resolve_query_set(&cache.queries, 0..2, &cache.resolve, 0);
    e.copy_buffer_to_buffer(&cache.resolve, 0, &cache.timing, 0, 16);
    r.queue.submit([e.finish()]);
    let v = read(r, &cache.timing, 16)?;
    let a = u64::from_le_bytes(v[..8].try_into()?);
    let b = u64::from_le_bytes(v[8..].try_into()?);
    ensure!(b > a, "Invalid publication timestamps");
    Ok((
        cpu,
        (b - a) as f64 * r.queue.get_timestamp_period() as f64 / 1e6,
    ))
}
fn compatible(
    a: &[BoxData],
    b: &[BoxData],
    old: &Params,
    new: &Params,
    kind: u32,
    index: usize,
) -> Result<()> {
    ensure!(
        a.len() == b.len() && a.len() <= 32,
        "Regional bake needs stable <=32 object slots"
    );
    let changed: Vec<_> = a
        .iter()
        .zip(b)
        .enumerate()
        .filter(|(_, (a, b))| bytemuck::bytes_of(*a) != bytemuck::bytes_of(*b))
        .map(|(i, _)| i)
        .collect();
    if kind == 0 {
        ensure!(
            changed.is_empty() && old.sun == new.sun && old.energy == new.energy,
            "Invalid no-change event"
        );
    } else if kind == 3 {
        ensure!(
            changed.is_empty(),
            "Global light event cannot hide geometry edits"
        );
    } else {
        ensure!(
            changed == vec![index] && old.sun == new.sun && old.energy == new.energy,
            "Regional edit mismatch"
        );
        if kind == 1 {
            ensure!(
                a[index].lo == b[index].lo && a[index].hi == b[index].hi,
                "Material event changed geometry"
            );
        }
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    r: &mut Renderer,
    engine: Engine,
    initial_boxes: Vec<BoxData>,
    native: wgpu::Texture,
    target: wgpu::Texture,
    p: Params,
    dir: &str,
) -> Result<()> {
    let n = initial_boxes.len() as u64 * 6 * u64::from(SIDE * SIDE);
    ensure!(
        n <= u64::from(W * H),
        "Diagnostic metadata capacity exceeded"
    );
    let nv = native.create_view(&Default::default());
    let tv = target.create_view(&Default::default());
    let cache = Lab::new_sized(r, &p, &initial_boxes, &nv, &tv, n * 16);
    let mut bp = p;
    bp.grid[3] = 0.5;
    bp.counts = [SIDE, 64, 0, SEED];
    bp.dims[3] = 0;
    let initial_cost = cache.run(r, &bp, "region-full")?;
    fs::write(
        format!("{dir}/initial-atlas.bin"),
        bake::pull(r, &cache.probes[1], n * 16)?,
    )?;
    fs::write(
        format!("{dir}/initial-deps.bin"),
        bake::pull(r, &cache.receivers, n * 48)?,
    )?;
    let initial = engine.scene.clone();
    let mut previous = initial.clone();
    let mut old_boxes = initial_boxes;
    let mut old_params = bp;
    let mut events = vec![];
    let labels = [
        "unchanged",
        "wall-color",
        "small-color",
        "move-a",
        "move-b",
        "return",
        "roof-size",
        "roof-return",
        "offscreen-blocker",
        "blocker-return",
        "sun-direction",
        "unchanged-after-sun",
    ];
    for (step, label) in labels.into_iter().enumerate() {
        let mut scene = previous.clone();
        let last = scene.entities.len() - 1;
        let (kind, index) = match step {
            0 | 11 => (0, 0),
            1 => {
                scene.entities[1].color = [0.8, 0.12, 0.03];
                (1, 1)
            }
            2 => {
                scene.entities[7].color = [0.05, 0.6, 0.2];
                (1, 7)
            }
            3 => {
                scene.entities[last].transform.position[0] += 1.7;
                scene.entities[last].transform.position[2] -= 1.2;
                (2, last)
            }
            4 => {
                scene.entities[last].transform.position[0] -= 2.8;
                scene.entities[last].transform.position[1] += 0.4;
                scene.entities[last].transform.position[2] += 0.5;
                (2, last)
            }
            5 => {
                scene.entities[last].transform = initial.entities[last].transform.clone();
                (2, last)
            }
            6 => {
                scene.entities[4].transform.size[0] *= 0.7;
                scene.entities[4].transform.size[2] *= 1.3;
                (2, 4)
            }
            7 => {
                scene.entities[4].transform = initial.entities[4].transform.clone();
                (2, 4)
            }
            8 => {
                scene.entities[last].transform.position = [5., 9., 3.];
                scene.entities[last].transform.size = [3., 1., 3.];
                (2, last)
            }
            9 => {
                scene.entities[last].transform = initial.entities[last].transform.clone();
                (2, last)
            }
            10 => {
                scene.light.direction = [0.45, -0.8, -0.25];
                (3, 0)
            }
            _ => unreachable!(),
        };
        let current = Engine::new(scene.clone())?;
        let boxes = bake::boxes_for(&current);
        let mut cp = bake::params(&current, bp);
        cp.grid = [1., 1., 1., 0.5];
        cp.counts = [SIDE, 64, index as u32, SEED];
        cp.dims[3] = kind;
        compatible(&old_boxes, &boxes, &old_params, &cp, kind, index)?;
        let path = format!("{dir}/{step:02}-{label}");
        fs::create_dir_all(&path)?;
        fs::write(
            format!("{path}/scene.json"),
            serde_json::to_vec_pretty(&scene)?,
        )?;
        let saved_atlas = snapshot(r, &cache.probes[1], n * 16);
        let saved_deps = snapshot(r, &cache.receivers, n * 48);
        let before = bake::pull(r, &cache.probes[1], n * 16)?;
        let upload_start = Instant::now();
        r.queue
            .write_buffer(&cache.geometry, 0, bytemuck::cast_slice(&boxes));
        let geometry_upload_submit_ms = ms(upload_start);
        let mark = cache.run(r, &cp, "region-mark")?;
        let flags = bake::pull(r, &cache.guides, n * 4)?;
        let count = flags
            .chunks_exact(4)
            .filter(|v| u32::from_le_bytes((*v).try_into().unwrap()) != 0)
            .count();
        fs::write(format!("{path}/dirty.bin"), &flags)?;
        let work = bake::pull(r, &cache.work, 8 + n * 4)?;
        fs::write(format!("{path}/work.bin"), work)?;
        let dispatch = bake::pull(r, &cache.indirect, 24)?;
        fs::write(format!("{path}/dispatch.bin"), dispatch)?;
        let view = Lab::new_sized(r, &cp, &boxes, &nv, &tv, n * 16);
        let oracle = Lab::new_sized(r, &cp, &boxes, &nv, &tv, n * 16);
        publish(r, &cache, &view, n)?;
        let mut frame = current.render_frame();
        frame.suppress_diagnostic_grid = true;
        r.draw(&frame, &nv, W, H)?;
        let mut qp = cp;
        qp.counts[3] = 7;
        qp.dims[3] = 24;
        view.run(r, &qp, "region-query")?;
        let pending = view.values(r)?;
        fs::write(format!("{path}/pending.rgba32f"), &pending)?;
        fs::write(
            format!("{path}/pending.png"),
            r.read_texture_png(&target, W, H)?,
        )?;
        let receivers = bake::pull(r, &view.receivers, u64::from(W * H) * 48)?;
        fs::write(format!("{path}/receivers.bin"), &receivers)?;
        let visible = receivers
            .chunks_exact(48)
            .filter(|v| f32::from_le_bytes(v[12..16].try_into().unwrap()) == (index + 1) as f32)
            .count();
        let oracle_cost = oracle.run(r, &cp, "region-full")?;
        let full = bake::pull(r, &oracle.probes[1], n * 16)?;
        let full_deps = bake::pull(r, &oracle.receivers, n * 48)?;
        let plain_cost = oracle.run(r, &cp, "bake-build")?;
        let plain = bake::pull(r, &oracle.probes[1], n * 16)?;
        let plain_error = plain
            .chunks_exact(4)
            .zip(full.chunks_exact(4))
            .map(|(a, b)| {
                (f32::from_le_bytes(a.try_into().unwrap())
                    - f32::from_le_bytes(b.try_into().unwrap()))
                .abs()
            })
            .fold(0f32, f32::max);
        ensure!(plain_error < 3e-6, "Dependency bake changed radiance");
        let changed = before
            .chunks_exact(16)
            .zip(full.chunks_exact(16))
            .filter(|(a, b)| a != b)
            .count();
        ensure!(
            before
                .chunks_exact(16)
                .zip(full.chunks_exact(16))
                .zip(flags.chunks_exact(4))
                .all(|((a, b), f)| a == b || u32::from_le_bytes(f.try_into().unwrap()) != 0),
            "Missed changed atlas texel at {label}"
        );
        let repair = cache.run(
            r,
            &cp,
            if kind == 3 {
                "region-full"
            } else {
                "region-repair"
            },
        )?;
        ensure!(
            bake::pull(r, &cache.probes[1], n * 16)? == full,
            "Regional atlas differs from full bake at {label}"
        );
        ensure!(
            bake::pull(r, &cache.receivers, n * 48)? == full_deps,
            "Regional dependencies differ from full bake at {label}"
        );
        let cleared = bake::pull(r, &cache.guides, n * 4)?;
        ensure!(
            cleared.iter().all(|b| *b == 0),
            "Unrepaired texel published"
        );
        fs::write(format!("{path}/atlas.bin"), &full)?;
        fs::write(format!("{path}/deps.bin"), &full_deps)?;
        let publication = publish(r, &cache, &view, n)?;
        view.run(r, &qp, "region-query")?;
        let published = view.values(r)?;
        fs::write(format!("{path}/published.rgba32f"), &published)?;
        let published_png = r.read_texture_png(&target, W, H)?;
        fs::write(format!("{path}/published.png"), &published_png)?;
        view.run(r, &qp, "bake-full")?;
        let full_view = view.values(r)?;
        ensure!(
            published == full_view,
            "Published frame differs from full bake at {label}"
        );
        ensure!(
            published_png == r.read_texture_png(&target, W, H)?,
            "Published PNG differs from full bake"
        );
        qp.dims[3] = 2;
        view.run(r, &qp, "full")?;
        let direct = view.values(r)?;
        fs::write(format!("{path}/direct.rgba32f"), &direct)?;
        fs::write(
            format!("{path}/direct.png"),
            r.read_texture_png(&target, W, H)?,
        )?;
        ensure!(
            pending
                .chunks_exact(16)
                .zip(published.chunks_exact(16))
                .zip(direct.chunks_exact(16))
                .all(|((a, b), d)| {
                    let flag = f32::from_le_bytes(a[12..].try_into().unwrap());
                    if flag == 2. {
                        a[..12] == d[..12]
                    } else {
                        a[..12] == b[..12]
                    }
                }),
            "Pending frame used stale dirty support"
        );
        let repaired_atlas = snapshot(r, &cache.probes[1], n * 16);
        let repaired_deps = snapshot(r, &cache.receivers, n * 48);
        let mut rows = vec![];
        for trial in 0..16 {
            for offset in 0..3 {
                let method = ["partial", "full-dependencies", "full-plain"][(trial + offset) % 3];
                copy(
                    r,
                    &[
                        (&saved_atlas, &cache.probes[1], n * 16),
                        (&saved_deps, &cache.receivers, n * 48),
                    ],
                );
                wait(r)?;
                let stage = match method {
                    "partial" => "region-update",
                    "full-dependencies" => "region-full",
                    _ => "bake-build",
                };
                let c = cache.run(r, &cp, stage)?;
                let pub_cost = publish(r, &cache, &view, n)?;
                rows.push(json!({"trial":trial,"method":method,"cpu_submit_ms":c.0+pub_cost.0,"gpu_ms":c.1+pub_cost.1,"repair_gpu_ms":c.1,"publication_gpu_ms":pub_cost.1}));
            }
        }
        // Restore the actual repaired state after controlled repeated measurements.
        copy(
            r,
            &[
                (&repaired_atlas, &cache.probes[1], n * 16),
                (&repaired_deps, &cache.receivers, n * 48),
            ],
        );
        wait(r)?;
        ensure!(
            bake::pull(r, &cache.probes[1], n * 16)? == full,
            "Replay restoration changed atlas"
        );
        ensure!(
            bake::pull(r, &cache.receivers, n * 48)? == full_deps,
            "Replay restoration changed dependencies"
        );
        let mut render_rows = vec![];
        publish(r, &cache, &view, n)?;
        for trial in 0..16 {
            for method in if trial % 2 == 0 {
                ["baked", "direct64"]
            } else {
                ["direct64", "baked"]
            } {
                let mut q = cp;
                q.counts[3] = trial;
                q.dims[3] = if method == "baked" { 24 } else { 2 };
                let stage = if method == "baked" {
                    "region-query"
                } else {
                    "full"
                };
                let t = Instant::now();
                let mut f = current.render_frame();
                f.suppress_diagnostic_grid = true;
                let prep = ms(t);
                let t = Instant::now();
                r.draw(&f, &nv, W, H)?;
                let cpu = ms(t);
                let raster = r.read_gpu_timing()?.unwrap()["sum_ms"].as_f64().unwrap();
                let c = view.run(r, &q, stage)?;
                render_rows.push(json!({"trial":trial,"method":method,"gpu_ms":raster+c.1,"cpu_ms":prep+cpu+c.0}));
            }
        }
        events.push(json!({"step":step,"label":label,"kind":kind,"object":index,"texels":n,"dirty":count,"changed_rgb":changed,"visible_changed_object_pixels":visible,"atlas_exact":true,"dependencies_exact":true,"published_png_exact":true,"pending_composition_exact":true,"plain_rgb_max_error":plain_error,"initial_mark_gpu_ms":mark.1,"initial_repair_gpu_ms":repair.1,"full_dependency_gpu_ms":oracle_cost.1,"full_plain_gpu_ms":plain_cost.1,"publication_gpu_ms":publication.1,"geometry_upload_submit_ms":geometry_upload_submit_ms,"geometry_upload_bytes":boxes.len()*48,"restored_actual_partial_output":true,"rows":rows,"render_rows":render_rows}));
        previous = scene;
        old_boxes = boxes;
        old_params = cp;
        println!("Regional {step:02} {label}: {count}/{n} dirty, {changed} changed");
    }
    fs::write(
        format!("{dir}/results.json"),
        serde_json::to_vec_pretty(
            &json!({"width":W,"height":H,"side":SIDE,"samples":4096,"seed":SEED,"texels":n,"atlas_bytes":n*16,"dependency_bytes":n*48,"dirty_bytes":n*4,"initial_dependency_build_gpu_ms":initial_cost.1,"adapter":r.adapter.get_info().name,"events":events}),
        )?,
    )?;
    Ok(())
}
