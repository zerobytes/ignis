//! Fixed-geometry scalar transport response; intentionally a bounded laboratory.
use super::*;
const SIDE: u32 = 32;
const SEED: u32 = 317;
fn publish(
    r: &Renderer,
    from: &wgpu::Buffer,
    to: &wgpu::Buffer,
    lab: &Lab,
    n: u64,
) -> Result<(f64, f64)> {
    let t = Instant::now();
    let mut e = r.device.create_command_encoder(&Default::default());
    lab.pass(
        &mut e,
        &lab.begin_batch,
        &lab.groups[1],
        [Some(0), None],
        Dispatch::Grid(1, 1, 1),
    );
    e.copy_buffer_to_buffer(from, 0, to, 0, n * 16);
    lab.pass(
        &mut e,
        &lab.begin_batch,
        &lab.groups[1],
        [None, Some(1)],
        Dispatch::Grid(1, 1, 1),
    );
    r.queue.submit([e.finish()]);
    let cpu = ms(t);
    wait(r)?;
    let mut e = r.device.create_command_encoder(&Default::default());
    e.resolve_query_set(&lab.queries, 0..2, &lab.resolve, 0);
    e.copy_buffer_to_buffer(&lab.resolve, 0, &lab.timing, 0, 16);
    r.queue.submit([e.finish()]);
    let v = read(r, &lab.timing, 16)?;
    let a = u64::from_le_bytes(v[..8].try_into()?);
    let b = u64::from_le_bytes(v[8..].try_into()?);
    ensure!(b > a, "Bad response publication timing");
    Ok((
        cpu,
        (b - a) as f64 * r.queue.get_timestamp_period() as f64 / 1e6,
    ))
}
fn hash(mut v: u32) -> u32 {
    v = (v ^ (v >> 16)).wrapping_mul(0x7feb352d);
    v = (v ^ (v >> 15)).wrapping_mul(0x846ca68b);
    v ^ (v >> 16)
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    r: &mut Renderer,
    engine: Engine,
    base_boxes: Vec<BoxData>,
    native: wgpu::Texture,
    target: wgpu::Texture,
    p: Params,
    dir: &str,
) -> Result<()> {
    ensure!(
        base_boxes.len() <= 12,
        "Response prototype supports <=12slots"
    );
    let n = base_boxes.len() as u64 * 6 * u64::from(SIDE * SIDE);
    let stride = (base_boxes.len() as u32).div_ceil(4);
    let nv = native.create_view(&Default::default());
    let tv = target.create_view(&Default::default());
    let cache = Lab::new_sized(r, &p, &base_boxes, &nv, &tv, n * u64::from(stride) * 16);
    let mut bp = p;
    bp.grid = [1., 1., 1., 0.5];
    bp.counts = [SIDE, 64, stride, SEED];
    let cold = cache.run(r, &bp, "response-build")?;
    let weights = bake::pull(r, &cache.probes[1], n * u64::from(stride) * 16)?;
    fs::write(format!("{dir}/weights.bin"), &weights)?;
    for row in weights.chunks_exact(stride as usize * 16) {
        let values: Vec<f32> = row
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        ensure!(
            values.iter().all(|v| v.is_finite() && *v >= 0.)
                && values.iter().sum::<f32>() <= 1.0001,
            "Invalid response weight/row energy"
        );
    }
    let base = engine.scene.clone();
    fs::write(
        format!("{dir}/source-scene.json"),
        serde_json::to_vec_pretty(&base)?,
    )?;
    let mut cases = vec![];
    for step in 0..20u32 {
        let mut scene = base.clone();
        for (j, e) in scene.entities.iter_mut().enumerate() {
            if step == 1 {
                e.color = [0.; 3];
            } else if (2..=5).contains(&step) {
                e.color = if j == [0usize, 1, 2, 7][(step - 2) as usize] {
                    [1.; 3]
                } else {
                    [0.; 3]
                };
            } else if step == 6 {
                e.color = [1.; 3];
            } else if step >= 9 {
                for (c, x) in e.color.iter_mut().enumerate() {
                    *x = (hash(
                        step.wrapping_mul(92837)
                            .wrapping_add(j as u32 * 7919 + c as u32 * 131),
                    ) >> 8) as f32
                        / 16777216.;
                }
            }
        }
        if step == 7 {
            scene.light.direct_color = [0., 0., 0.];
        }
        if step == 8 {
            scene.light.direct_color = [0.4, 0.8, 1.];
            scene.light.direct_intensity = 2.;
        }
        if step % 3 == 1 {
            scene.camera.position = [-7., 3.6, 8.9];
            scene.camera.target = [-0.2, 0.5, -1.];
        }
        if step % 3 == 2 {
            scene.camera.position = [8., 3.5, -9.];
            scene.camera.target = [0., 1., 0.];
        }
        let current = Engine::new(scene)?;
        let boxes = bake::boxes_for(&current);
        let mut cp = bake::params(&current, bp);
        cp.grid = [1., 1., 1., 0.5];
        cp.counts = [SIDE, 64, stride, SEED];
        ensure!(
            cp.sun == bp.sun
                && boxes.len() == base_boxes.len()
                && boxes
                    .iter()
                    .zip(&base_boxes)
                    .all(|(a, b)| a.lo == b.lo && a.hi == b.hi),
            "Unsupported response geometry/sun edit"
        );
        let path = format!("{dir}/{step:02}");
        fs::create_dir_all(&path)?;
        fs::write(
            format!("{path}/scene.json"),
            serde_json::to_vec_pretty(&current.scene)?,
        )?;
        let t = Instant::now();
        r.queue
            .write_buffer(&cache.geometry, 0, bytemuck::cast_slice(&boxes));
        let upload = ms(t);
        let full = Lab::new_sized(r, &cp, &boxes, &nv, &tv, n * 16);
        let view = Lab::new_sized(r, &cp, &boxes, &nv, &tv, n * 16);
        let cost = cache.run(r, &cp, "response-combine")?;
        let combined = bake::pull(r, &cache.values, n * 16)?;
        full.run(r, &cp, "bake-build")?;
        let reference = bake::pull(r, &full.probes[1], n * 16)?;
        let max_error = combined
            .chunks_exact(16)
            .zip(reference.chunks_exact(16))
            .map(|(a, b)| {
                ensure!(a[12..] == b[12..], "Response changed chart validity");
                Ok(a[..12]
                    .chunks_exact(4)
                    .zip(b[..12].chunks_exact(4))
                    .map(|(x, y)| {
                        (f32::from_le_bytes(x.try_into().unwrap())
                            - f32::from_le_bytes(y.try_into().unwrap()))
                        .abs()
                    })
                    .fold(0f32, f32::max))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .fold(0f32, f32::max);
        ensure!(
            max_error <= 0.0002,
            "Response linear error exceeded declared bound: {step}/{max_error}"
        );
        fs::write(format!("{path}/combined.bin"), &combined)?;
        fs::write(format!("{path}/reference.bin"), &reference)?;
        let mut frame = current.render_frame();
        frame.suppress_diagnostic_grid = true;
        r.draw(&frame, &nv, W, H)?;
        let mut qp = cp;
        qp.dims[3] = 24;
        qp.counts[3] = 7;
        publish(r, &full.probes[1], &view.probes[1], &full, n)?;
        view.run(r, &qp, "bake-full")?;
        let expected = view.values(r)?;
        fs::write(
            format!("{path}/reference.png"),
            r.read_texture_png(&target, W, H)?,
        )?;
        publish(r, &cache.values, &view.probes[1], &cache, n)?;
        view.run(r, &qp, "bake-full")?;
        let output = view.values(r)?;
        fs::write(
            format!("{path}/combined.png"),
            r.read_texture_png(&target, W, H)?,
        )?;
        let view_error = output
            .chunks_exact(16)
            .zip(expected.chunks_exact(16))
            .map(|(a, b)| {
                a[..12]
                    .chunks_exact(4)
                    .zip(b[..12].chunks_exact(4))
                    .map(|(x, y)| {
                        (f32::from_le_bytes(x.try_into().unwrap())
                            - f32::from_le_bytes(y.try_into().unwrap()))
                        .abs()
                    })
                    .fold(0f32, f32::max)
            })
            .fold(0f32, f32::max);
        ensure!(view_error <= 0.0002, "Response visible RGB error too large");
        let mut rows = vec![];
        for trial in 0..16 {
            for method in if trial % 2 == 0 {
                ["coefficients", "full-bake"]
            } else {
                ["full-bake", "coefficients"]
            } {
                let (lab, stage) = if method == "coefficients" {
                    (&cache, "response-combine")
                } else {
                    (&full, "bake-build")
                };
                let c = lab.run(r, &cp, stage)?;
                let source = if method == "coefficients" {
                    &cache.values
                } else {
                    &full.probes[1]
                };
                let pub_cost = publish(r, source, &view.probes[1], lab, n)?;
                rows.push(json!({"trial":trial,"method":method,"cpu_submit_ms":c.0+pub_cost.0,"gpu_ms":c.1+pub_cost.1,"compute_gpu_ms":c.1,"publication_gpu_ms":pub_cost.1}));
            }
        }
        cases.push(json!({"step":step,"field_max_error":max_error,"visible_max_error":view_error,"geometry_upload_submit_ms":upload,"initial_combine_gpu_ms":cost.1,"rows":rows}));
        println!("Response {step:02}: error {max_error}");
    }
    fs::write(
        format!("{dir}/results.json"),
        serde_json::to_vec_pretty(
            &json!({"width":W,"height":H,"side":SIDE,"objects":base_boxes.len(),"texels":n,"stride_vec4":stride,"seed":SEED,"samples":4096,"weight_bytes":weights.len(),"cold_build_cpu_ms":cold.0,"cold_build_gpu_ms":cold.1,"adapter":r.adapter.get_info().name,"cases":cases}),
        )?,
    )?;
    Ok(())
}
