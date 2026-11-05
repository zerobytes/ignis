//! Asynchronous, versioned, private repair in the finite-response laboratory.
use super::*;
use std::sync::mpsc::{Receiver, TryRecvError};
const SIDE: u32 = 32;
const SEED: u32 = 317;
const FRAME: Duration = Duration::from_nanos(16_666_667);
#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Mark,
    Repair,
    Publish,
    RegionalControl,
    FullControl,
    PlainControl,
}
struct Request {
    boxes: Vec<BoxData>,
    p: Params,
    upload: wgpu::Buffer,
    prepare_cpu_ms: f64,
}
impl Request {
    fn new(
        r: &Renderer,
        scene: ignis::model::Scene,
        base: &Params,
        old: &[BoxData],
        old_p: &Params,
    ) -> Result<Self> {
        let t = Instant::now();
        let engine = Engine::new(scene.clone())?;
        let boxes = bake::boxes_for(&engine);
        ensure!(
            boxes.len() == old.len() && boxes.len() <= 12,
            "Stable <=12 response slots required"
        );
        let mut p = bake::params(&engine, *base);
        p.grid = [0., 0., 0., 0.5];
        p.counts = [SIDE, 64, (boxes.len() as u32).div_ceil(4), SEED];
        let mut mask = 0u32;
        for (j, (a, b)) in old.iter().zip(&boxes).enumerate() {
            if a.lo != b.lo || a.hi != b.hi {
                mask |= 1 << j;
            }
        }
        p.dims[3] = if p.sun != old_p.sun {
            3
        } else if mask != 0 {
            2
        } else {
            0
        };
        p.grid[2] = mask as f32;
        let upload = r
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("response revision geometry"),
                contents: bytemuck::cast_slice(&boxes),
                usage: wgpu::BufferUsages::COPY_SRC,
            });
        Ok(Self {
            boxes,
            p,
            upload,
            prepare_cpu_ms: ms(t),
        })
    }
}
struct Flight {
    read: wgpu::Buffer,
    ready: Receiver<std::result::Result<(), wgpu::BufferAsyncError>>,
    epoch: u64,
    phase: Phase,
    issued_frame: u32,
    rows: u32,
    cpu_submit_ms: f64,
}
struct Completed {
    epoch: u64,
    phase: Phase,
    frame: u32,
    rows: u32,
    cpu_submit_ms: f64,
    gpu_ms: f64,
    dirty: u32,
}
impl Flight {
    fn poll(&self, r: &Renderer) -> Result<Option<Completed>> {
        r.device.poll(wgpu::PollType::Poll)?;
        match self.ready.try_recv() {
            Err(TryRecvError::Empty) => return Ok(None),
            Err(TryRecvError::Disconnected) => anyhow::bail!("Lost asynchronous response readback"),
            Ok(v) => v?,
        }
        let mapped = self.read.slice(..).get_mapped_range();
        let a = u64::from_le_bytes(mapped[..8].try_into()?);
        let b = u64::from_le_bytes(mapped[8..16].try_into()?);
        ensure!(
            b > a,
            "Invalid asynchronous timestamps: {:?} {a}..{b}",
            self.phase
        );
        let dirty = u32::from_le_bytes(mapped[16..20].try_into()?);
        drop(mapped);
        self.read.unmap();
        Ok(Some(Completed {
            epoch: self.epoch,
            phase: self.phase,
            frame: self.issued_frame,
            rows: self.rows,
            cpu_submit_ms: self.cpu_submit_ms,
            gpu_ms: (b - a) as f64 * r.queue.get_timestamp_period() as f64 / 1e6,
            dirty,
        }))
    }
}
#[allow(clippy::too_many_arguments)]
fn submit(
    r: &Renderer,
    committed: &Lab,
    working: &Lab,
    view: &Lab,
    request: &Request,
    phase: Phase,
    epoch: u64,
    frame: u32,
    offset: u32,
    rows: u32,
    n: u64,
) -> Flight {
    let t = Instant::now();
    let mut p = request.p;
    p.grid[0] = offset as f32;
    p.grid[1] = rows as f32;
    r.queue
        .write_buffer(&working.uniform_upload, 0, bytemuck::bytes_of(&p));
    let read = working.timing.clone();
    let queries = working.queries.clone();
    let resolve = working.resolve.clone();
    let mut e = r.device.create_command_encoder(&Default::default());
    let marker = |e: &mut wgpu::CommandEncoder, start: bool| {
        let mut pass = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("response batch boundary"),
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set: &queries,
                beginning_of_pass_write_index: if start { Some(0) } else { None },
                end_of_pass_write_index: if start { None } else { Some(1) },
            }),
        });
        pass.set_pipeline(&working.response_boundary);
        let end_view = !start
            && matches!(
                phase,
                Phase::Publish | Phase::RegionalControl | Phase::FullControl | Phase::PlainControl
            );
        pass.set_bind_group(
            0,
            if end_view {
                &view.compute_groups[1]
            } else {
                &working.compute_groups[1]
            },
            &[],
        );
        pass.dispatch_workgroups(1, 1, 1);
    };
    marker(&mut e, true);
    e.copy_buffer_to_buffer(
        &working.uniform_upload,
        0,
        &working.uniform,
        0,
        std::mem::size_of::<Params>() as u64,
    );
    let color_publish = phase == Phase::Publish && p.dims[3] == 0;
    if matches!(
        phase,
        Phase::Mark | Phase::RegionalControl | Phase::FullControl | Phase::PlainControl
    ) || color_publish
    {
        e.copy_buffer_to_buffer(
            &request.upload,
            0,
            &working.geometry,
            0,
            (request.boxes.len() * 48) as u64,
        );
    }
    if matches!(phase, Phase::Mark | Phase::RegionalControl) || color_publish {
        for (a, b, size) in [
            (
                &committed.probes[1],
                &working.probes[1],
                n * u64::from(p.counts[2]) * 16,
            ),
            (&committed.receivers, &working.receivers, n * 48),
            (&committed.values, &working.values, n * 16),
        ] {
            e.copy_buffer_to_buffer(a, 0, b, 0, size);
        }
    }
    if color_publish {
        e.clear_buffer(&working.guides, 0, Some(n * 4));
    }
    if matches!(phase, Phase::Mark | Phase::RegionalControl) {
        working.pass(
            &mut e,
            &working.response_region_mark,
            &working.compute_groups[1],
            [None, None],
            Dispatch::Grid((n as u32).div_ceil(64), 1, 1),
        );
        working.pass(
            &mut e,
            &working.reset,
            &working.compute_groups[1],
            [None, None],
            Dispatch::Grid(1, 1, 1),
        );
        working.pass(
            &mut e,
            &working.region_compact,
            &working.compute_groups[1],
            [None, None],
            Dispatch::Grid((n as u32).div_ceil(64), 1, 1),
        );
    }
    if matches!(phase, Phase::Repair | Phase::RegionalControl) {
        working.pass(
            &mut e,
            &working.response_region_repair,
            &working.compute_groups[1],
            [None, None],
            Dispatch::Grid(rows.min(6144), rows.div_ceil(6144), 1),
        );
    }
    if phase == Phase::FullControl {
        working.pass(
            &mut e,
            &working.response_region_full,
            &working.compute_groups[1],
            [None, None],
            Dispatch::Grid(6144, p.dims[2], 1),
        );
    }
    if phase == Phase::PlainControl {
        working.pass(
            &mut e,
            &working.atlas_build,
            &working.compute_groups[1],
            [None, None],
            Dispatch::Grid((n as u32).div_ceil(64), 1, 1),
        );
    }
    if matches!(
        phase,
        Phase::Publish | Phase::RegionalControl | Phase::FullControl
    ) {
        working.pass(
            &mut e,
            &working.response_combine,
            &working.compute_groups[1],
            [None, None],
            Dispatch::Grid((n as u32).div_ceil(64), 1, 1),
        );
        e.copy_buffer_to_buffer(&working.values, 0, &view.probes[1], 0, n * 16);
        e.copy_buffer_to_buffer(&working.guides, 0, &view.guides, 0, n * 4);
    } else if phase == Phase::PlainControl {
        e.copy_buffer_to_buffer(&working.probes[1], 0, &view.probes[1], 0, n * 16);
    }
    marker(&mut e, false);
    r.queue.submit([e.finish()]);
    let (tx, ready) = std::sync::mpsc::channel();
    // Metal stage-boundary counters must be resolved AFTER GPU completion.
    // This callback schedules telemetry; the frame thread never waits for it.
    let device = r.device.clone();
    let queue = r.queue.clone();

    let counts = working.work.clone();
    let mapped = read.clone();
    r.queue.on_submitted_work_done(move || {
        let mut e = device.create_command_encoder(&Default::default());
        e.resolve_query_set(&queries, 0..2, &resolve, 0);
        e.copy_buffer_to_buffer(&resolve, 0, &mapped, 0, 16);
        e.copy_buffer_to_buffer(&counts, 0, &mapped, 16, 8);
        queue.submit([e.finish()]);
        mapped.slice(..).map_async(wgpu::MapMode::Read, move |v| {
            let _ = tx.send(v);
        });
    });
    Flight {
        read,
        ready,
        epoch,
        phase,
        issued_frame: frame,
        rows,
        cpu_submit_ms: ms(t),
    }
}
fn control(r: &Renderer, flight: Flight) -> Result<Completed> {
    let start = Instant::now();
    loop {
        wait(r)?;
        if let Some(c) = flight.poll(r)? {
            return Ok(c);
        }
        ensure!(
            start.elapsed() < Duration::from_secs(60),
            "Control telemetry timed out"
        );
    }
}

struct ScheduleResult {
    rows: Vec<serde_json::Value>,
    ticks: Vec<serde_json::Value>,
    latency_ms: f64,
    publish_frame: u32,
    cancelled: u32,
    dirty: u32,
    working_epoch: u64,
    published: bool,
}
#[allow(clippy::too_many_arguments)]
fn schedule(
    r: &mut Renderer,
    committed: &Lab,
    working: &Lab,
    view: &Lab,
    first: &Request,
    replacement: Option<&Request>,
    cancel_at: Phase,
    n: u64,
    budget: f64,
    base_epoch: u64,
    cold_rate: f64,
    mut background: impl FnMut(&mut Renderer) -> Result<()>,
) -> Result<ScheduleResult> {
    let start = Instant::now();
    let mut frame = 0u32;
    let mut epoch = base_epoch;
    let mut req = first;
    let mut flight: Option<Flight> = None;
    let mut phase = if first.p.dims[3] == 0 {
        Phase::Publish
    } else {
        Phase::Mark
    };
    let mut offset = 0u32;
    let mut dirty = 0u32;
    let mut rows = vec![];
    let mut ticks = vec![];
    let mut cancelled = 0;
    let mut cancel_next = false;
    let mut costs = std::collections::VecDeque::from([cold_rate * 2.; 8]);
    let mut launch = 0.0f64;
    loop {
        ensure!(frame < 3000, "Response scheduler failed to converge");
        let deadline = start + FRAME * frame;
        if let Some(delay) = deadline.checked_duration_since(Instant::now()) {
            std::thread::sleep(delay);
        }
        let tick = Instant::now();
        let mut observed = None;
        if cancel_next {
            epoch += 1;
            req = replacement.unwrap();
            cancel_next = false;
            cancelled += 1;
            phase = if req.p.dims[3] == 0 {
                Phase::Publish
            } else {
                Phase::Mark
            };
            dirty = 0;
            offset = 0;
        }
        if let Some(f) = &flight {
            if let Some(c) = f.poll(r)? {
                observed = Some(c.phase);
                let stale = c.epoch != epoch;
                rows.push(json!({"epoch":c.epoch,"phase":format!("{:?}",c.phase),"issued_frame":c.frame,"completion_frame":frame,"rows":c.rows,"gpu_ms":c.gpu_ms,"cpu_submit_ms":c.cpu_submit_ms,"stale_discarded":stale,"over_budget":c.gpu_ms>budget}));
                if !stale {
                    if c.phase == Phase::Mark {
                        dirty = c.dirty;
                        phase = if dirty == 0 {
                            Phase::Publish
                        } else {
                            Phase::Repair
                        };
                    }
                    if c.phase == Phase::Repair {
                        if offset == 0 && c.rows == 1 {
                            launch = c.gpu_ms * 1.1;
                        } else {
                            costs.pop_front();
                            costs.push_back(
                                ((c.gpu_ms - launch).max(0.) / f64::from(c.rows))
                                    .max(cold_rate * 0.5),
                            );
                        }
                        if offset == 0 {
                            println!("Budget {budget}: one-row probe {:.3} ms", c.gpu_ms);
                        }
                        offset += c.rows;
                        phase = if offset >= dirty {
                            Phase::Publish
                        } else {
                            Phase::Repair
                        };
                    }
                    if c.phase == Phase::Publish {
                        ticks.push(json!({"frame":frame,"cpu_tick_ms":ms(tick),"submitted":false,"published_epoch":epoch}));
                        return Ok(ScheduleResult {
                            rows,
                            ticks,
                            latency_ms: ms(start),
                            publish_frame: frame,
                            cancelled,
                            dirty,
                            working_epoch: epoch,
                            published: true,
                        });
                    }
                }
                flight = None;
            }
        }
        if flight.is_none() && frame >= 150 {
            ticks.push(json!({"frame":frame,"cpu_tick_ms":ms(tick),"submitted":false,"published_epoch":0,"deadline_rejected":true}));
            return Ok(ScheduleResult {
                rows,
                ticks,
                latency_ms: ms(start),
                publish_frame: frame,
                cancelled,
                dirty,
                working_epoch: epoch,
                published: false,
            });
        }
        let mut admitted = 0;
        if flight.is_none() {
            if phase == Phase::Repair {
                let cost = costs.iter().copied().fold(0.00005f64, f64::max);
                let available = budget * 0.85 - launch;
                // An infeasible target is reported by actual overruns, never hidden
                // by reducing admission to one row forever.
                let cap = if offset == 0 {
                    1
                } else if available <= 0. {
                    128
                } else {
                    ((available / cost) as u32).clamp(1, 2048)
                };
                admitted = cap.min(dirty - offset);
            }
            let f = submit(
                r, committed, working, view, req, phase, epoch, frame, offset, admitted, n,
            );
            if replacement.is_some() && cancelled == 0 && phase == cancel_at {
                cancel_next = true;
            }
            flight = Some(f);
        }
        let repair_cpu = ms(tick);
        let draw = Instant::now();
        background(r)?;
        ticks.push(json!({"frame":frame,"cpu_tick_ms":repair_cpu,"background_cpu_ms":ms(draw),"total_cpu_tick_ms":ms(tick),"submitted":flight.as_ref().is_some_and(|f|f.issued_frame==frame),"rows_admitted":admitted,"observed":observed.map(|p|format!("{p:?}")),"requested_epoch":epoch}));
        frame += 1;
    }
}
fn max_rgb(a: &[u8], b: &[u8]) -> f32 {
    a.chunks_exact(16)
        .zip(b.chunks_exact(16))
        .flat_map(|(a, b)| {
            a[..12]
                .chunks_exact(4)
                .zip(b[..12].chunks_exact(4))
                .map(|(a, b)| {
                    (f32::from_le_bytes(a.try_into().unwrap())
                        - f32::from_le_bytes(b.try_into().unwrap()))
                    .abs()
                })
        })
        .fold(0., f32::max)
}
fn copy(r: &Renderer, a: &wgpu::Buffer, b: &wgpu::Buffer, n: u64) {
    let mut e = r.device.create_command_encoder(&Default::default());
    e.copy_buffer_to_buffer(a, 0, b, 0, n);
    r.queue.submit([e.finish()]);
}
#[allow(clippy::too_many_arguments)]
fn view(
    r: &mut Renderer,
    lab: &Lab,
    scene: &ignis::model::Scene,
    base: Params,
    native: &wgpu::Texture,
    target: &wgpu::Texture,
    path: &str,
    stage: &str,
) -> Result<(Vec<u8>, f64, f64)> {
    let engine = Engine::new(scene.clone())?;
    let boxes = bake::boxes_for(&engine);
    r.queue
        .write_buffer(&lab.geometry, 0, bytemuck::cast_slice(&boxes));
    let mut p = bake::params(&engine, base);
    p.grid = [1., 1., 1., 0.5];
    p.counts = [SIDE, 64, 0, 7];
    p.dims[3] = 24;
    let mut frame = engine.render_frame();
    frame.suppress_diagnostic_grid = true;
    r.draw(&frame, &native.create_view(&Default::default()), W, H)?;
    let raster = r.read_gpu_timing()?.unwrap()["sum_ms"].as_f64().unwrap();
    let c = lab.run(r, &p, stage)?;
    let values = lab.values(r)?;
    fs::write(format!("{path}.png"), r.read_texture_png(target, W, H)?)?;
    fs::write(format!("{path}.rgba32f"), &values)?;
    Ok((values, raster, c.1))
}
fn allocated(l: &Lab) -> u64 {
    [
        &l.geometry,
        &l.uniform,
        &l.uniform_upload,
        &l.receivers,
        &l.values,
        &l.filtered,
        &l.guides,
        &l.staging,
        &l.probes[0],
        &l.probes[1],
        &l.work,
        &l.indirect,
        &l.resolve,
        &l.timing,
    ]
    .iter()
    .map(|b| b.size())
    .sum()
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
    let stride = (initial_boxes.len() as u32).div_ceil(4);
    let wb = n * u64::from(stride) * 16;
    let nv = native.create_view(&Default::default());
    let tv = target.create_view(&Default::default());
    let initial = engine.scene.clone();
    let mut bp = p;
    bp.grid = [1., 1., 1., 0.5];
    bp.counts = [SIDE, 64, stride, SEED];
    let mut events = vec![];
    let mut memory = 0;
    for budget in [1.0f64, 2.] {
        let mut committed =
            Lab::new_sized_with_scratch(r, &bp, &initial_boxes, &nv, &tv, wb, n * 256);
        let mut working =
            Lab::new_sized_with_scratch(r, &bp, &initial_boxes, &nv, &tv, wb, n * 256);
        let display = Lab::new_sized(r, &bp, &initial_boxes, &nv, &tv, n * 16);
        let background_view = Lab::new_sized(r, &bp, &initial_boxes, &nv, &tv, n * 16);
        let oracle = Lab::new_sized_with_scratch(r, &bp, &initial_boxes, &nv, &tv, wb, n * 256);
        memory = allocated(&committed)
            + allocated(&working)
            + allocated(&display)
            + allocated(&oracle)
            + allocated(&background_view);
        let seeds_cost = committed.run(r, &bp, "response-seeds")?;
        copy(r, &committed.filtered, &working.filtered, n * 256);
        copy(r, &committed.filtered, &oracle.filtered, n * 256);
        fs::write(
            format!("{dir}/seeds.bin"),
            bake::pull(r, &committed.filtered, n * 256)?,
        )?;
        let cold = committed.run(r, &bp, "response-regional-full")?;
        println!(
            "Cold RNG {:.3} ms, coefficients {:.3} ms",
            seeds_cost.1, cold.1
        );
        let mut previous = initial.clone();
        let mut old_boxes = initial_boxes.clone();
        let mut old_p = bp;
        let budget_path = format!("{dir}/{budget:.1}");
        fs::create_dir_all(&budget_path)?;
        copy(r, &committed.values, &display.probes[1], n * 16);
        let (mut held, _, _) = view(
            r,
            &display,
            &previous,
            bp,
            &native,
            &target,
            &format!("{budget_path}/initial"),
            "bake-full",
        )?;
        let mut held_png = fs::read(format!("{budget_path}/initial.png"))?;
        fs::write(
            format!("{budget_path}/initial-scene.json"),
            serde_json::to_vec_pretty(&previous)?,
        )?;
        for step in 0..17 {
            let mut scene = previous.clone();
            let last = scene.entities.len() - 1;
            let label = match step {
                0 => "unchanged",
                1 => {
                    scene.entities[1].color = [0.8, 0.12, 0.03];
                    "color"
                }
                2 => {
                    scene.entities[last].transform.position[0] += 1.7;
                    scene.entities[last].transform.position[2] -= 1.2;
                    "move-a"
                }
                3 => {
                    scene.entities[last].transform.position[0] -= 2.8;
                    scene.entities[last].transform.position[1] += 0.4;
                    "move-b"
                }
                4 => {
                    scene.entities[last].transform = initial.entities[last].transform.clone();
                    "return"
                }
                5 => {
                    scene.entities[4].transform.size[0] *= 0.7;
                    scene.entities[4].transform.size[2] *= 1.3;
                    "roof"
                }
                6 => {
                    scene.entities[4].transform = initial.entities[4].transform.clone();
                    "roof-return"
                }
                7 => {
                    scene.entities[last].transform.position = [5., 9., 3.];
                    scene.entities[last].transform.size = [3., 1., 3.];
                    "offscreen-blocker"
                }
                8 => {
                    scene.entities[last].transform = initial.entities[last].transform.clone();
                    "blocker-return"
                }
                9 => {
                    scene.light.direction = [0.45, -0.8, -0.25];
                    "sun-direction"
                }
                10 => {
                    for e in &mut scene.entities {
                        e.color = [0.; 3];
                    }
                    "black"
                }
                11 => {
                    scene.entities[last].transform.position[0] += 0.8;
                    "black-geometry"
                }
                12 => {
                    for (e, b) in scene.entities.iter_mut().zip(&initial.entities) {
                        e.color = b.color;
                    }
                    scene.camera.position = [8., 3.5, -9.];
                    scene.camera.target = [0., 1., 0.];
                    "recolor-camera"
                }
                13 => {
                    scene.light.direct_color = [0.; 3];
                    "zero-light"
                }
                14 => {
                    scene.light.direct_color = [0.4, 0.8, 1.];
                    scene.light.direct_intensity = 2.;
                    "recover-light"
                }
                15 => {
                    scene.entities[last].transform.position[2] += 0.9;
                    "supersede-repair"
                }
                16 => {
                    scene.entities[1].color = [0.2, 0.8, 0.1];
                    "supersede-publication"
                }
                _ => unreachable!(),
            };
            ensure!(
                scene
                    .entities
                    .iter()
                    .zip(&previous.entities)
                    .all(|(a, b)| a.id == b.id),
                "Object slots changed identity"
            );
            let first = Request::new(r, scene.clone(), &bp, &old_boxes, &old_p)?;
            let replacement = if step >= 15 {
                if step == 15 {
                    scene.entities[last].transform.position[0] -= 1.6;
                    scene.entities[4].transform.size[2] *= 0.8;
                } else {
                    scene.entities[1].color = [0.9, 0.1, 0.3];
                    scene.light.direct_intensity = 1.3;
                }
                Some(Request::new(r, scene.clone(), &bp, &old_boxes, &old_p)?)
            } else {
                None
            };
            let request = replacement.as_ref().unwrap_or(&first);
            let path = format!("{budget_path}/{step:02}-{label}");
            fs::create_dir_all(&path)?;
            fs::write(
                format!("{path}/scene.json"),
                serde_json::to_vec_pretty(&scene)?,
            )?;
            let before = bake::pull(r, &committed.probes[1], wb)?;
            let active_deps = bake::pull(r, &committed.receivers, n * 48)?;
            r.queue
                .write_buffer(&oracle.geometry, 0, bytemuck::cast_slice(&request.boxes));
            let oracle_cost = oracle.run(r, &request.p, "response-regional-full")?;
            let full_weights = bake::pull(r, &oracle.probes[1], wb)?;
            let full_deps = bake::pull(r, &oracle.receivers, n * 48)?;
            let full_rgb = bake::pull(r, &oracle.values, n * 16)?;
            copy(r, &oracle.values, &display.probes[1], n * 16);
            let (expected, raster_gpu, query_gpu) = view(
                r,
                &display,
                &scene,
                bp,
                &native,
                &target,
                &format!("{path}/reference"),
                "bake-full",
            )?;
            let visible = bake::pull(r, &display.receivers, u64::from(W * H) * 48)?
                .chunks_exact(48)
                .filter(|v| f32::from_le_bytes(v[12..16].try_into().unwrap()) == (last + 1) as f32)
                .count();
            if step == 7 {
                ensure!(visible == 0, "Designated offscreen blocker is visible");
            }
            oracle.run(r, &request.p, "bake-build")?;
            let plain = bake::pull(r, &oracle.probes[1], n * 16)?;
            ensure!(
                max_rgb(&full_rgb, &plain) <= 0.0002,
                "Coefficient RGB differs from plain bake"
            );
            fs::write(format!("{path}/weights.bin"), &full_weights)?;
            fs::write(format!("{path}/deps.bin"), &full_deps)?;
            fs::write(format!("{path}/reference-atlas.bin"), &plain)?;
            // Pending fallback is diagnostic only, outside scheduler latency/budget.
            let mark = control(
                r,
                submit(
                    r,
                    &committed,
                    &working,
                    &display,
                    request,
                    Phase::Mark,
                    0,
                    0,
                    0,
                    n as u32,
                    n,
                ),
            )?;
            let flags = bake::pull(r, &working.guides, n * 4)?;
            fs::write(format!("{path}/dirty.bin"), &flags)?;
            let changed = before
                .chunks_exact(stride as usize * 16)
                .zip(full_weights.chunks_exact(stride as usize * 16))
                .zip(flags.chunks_exact(4))
                .filter(|((a, b), f)| {
                    let changed = a != b;
                    assert!(
                        !changed || u32::from_le_bytes((*f).try_into().unwrap()) != 0,
                        "Missed changed coefficient row"
                    );
                    changed
                })
                .count();
            // Include dependency-only differences as a stronger invalidation oracle.
            ensure!(
                active_deps
                    .chunks_exact(48)
                    .zip(full_deps.chunks_exact(48))
                    .zip(flags.chunks_exact(4))
                    .all(|((a, b), f)| a == b || u32::from_le_bytes(f.try_into().unwrap()) != 0),
                "Missed changed dependency row"
            );
            working.run(r, &request.p, "response-combine")?;
            copy(r, &working.values, &display.probes[1], n * 16);
            copy(r, &working.guides, &display.guides, n * 4);
            let (pending, _, pending_gpu) = view(
                r,
                &display,
                &scene,
                bp,
                &native,
                &target,
                &format!("{path}/pending64"),
                "region-query",
            )?;
            fs::write(format!("{path}/held.rgba32f"), &held)?;
            fs::write(format!("{path}/held.png"), &held_png)?;
            let mut controls = vec![];
            for trial in 0..4 {
                for offset in 0..3 {
                    let phase = [
                        Phase::RegionalControl,
                        Phase::FullControl,
                        Phase::PlainControl,
                    ][(trial + offset) % 3];
                    let c = control(
                        r,
                        submit(
                            r, &committed, &working, &display, request, phase, 0, 0, 0, n as u32, n,
                        ),
                    )?;
                    controls.push(json!({"trial":trial,"method":format!("{phase:?}"),"gpu_ms":c.gpu_ms,"cpu_submit_ms":c.cpu_submit_ms}));
                }
            }
            println!(
                "Full coefficient kernel {label}: {:.3} ms; control {:?}",
                oracle_cost.1,
                controls
                    .iter()
                    .filter(|c| c["method"] == "FullControl")
                    .map(|c| c["gpu_ms"].as_f64().unwrap())
                    .collect::<Vec<_>>()
            );
            ensure!(
                controls
                    .iter()
                    .filter(|c| c["method"] == "FullControl")
                    .all(|c| c["gpu_ms"].as_f64().unwrap() >= oracle_cost.1 * 0.5),
                "Async span understates fenced full kernel"
            );
            copy(r, &committed.values, &background_view.probes[1], n * 16);
            let (background_expected, background_native, background_query) = view(
                r,
                &background_view,
                &previous,
                bp,
                &native,
                &target,
                &format!("{path}/background"),
                "bake-full",
            )?;
            ensure!(
                background_expected == held,
                "Background render changed the committed image"
            );
            let old_engine = Engine::new(previous.clone())?;
            let mut old_frame = old_engine.render_frame();
            old_frame.suppress_diagnostic_grid = true;
            let old_nv = native.create_view(&Default::default());
            let result = schedule(
                r,
                &committed,
                &working,
                &display,
                &first,
                replacement.as_ref(),
                if step == 16 {
                    Phase::Publish
                } else {
                    Phase::Repair
                },
                n,
                budget,
                (step * 2 + 1) as u64,
                cold.1 / n as f64,
                |r| {
                    r.draw(&old_frame, &old_nv, W, H)?;
                    let mut e = r.device.create_command_encoder(&Default::default());
                    background_view.pass(
                        &mut e,
                        &background_view.primary,
                        &background_view.groups[1],
                        [None, None],
                        Dispatch::Grid(W.div_ceil(8), H.div_ceil(8), 1),
                    );
                    background_view.pass(
                        &mut e,
                        &background_view.atlas_query,
                        &background_view.groups[1],
                        [None, None],
                        Dispatch::Grid(W.div_ceil(8), H.div_ceil(8), 1),
                    );
                    r.queue.submit([e.finish()]);
                    Ok(())
                },
            )?;
            ensure!(
                background_view.values(r)? == held,
                "Background query used an incomplete revision"
            );
            let background_final = r.read_texture_png(&target, W, H)?;
            ensure!(
                background_final == held_png,
                "Background image changed before atomic publication"
            );
            fs::write(format!("{path}/background-final.png"), background_final)?;
            ensure!(
                result.cancelled == u32::from(replacement.is_some()),
                "Supersession not exercised"
            );
            // Test-driver settling is explicitly OUTSIDE the frame scheduler.
            // A rejected job never publishes. Complete a fresh controlled repair
            // only to validate the requested state and prepare the next fixture.
            let offline_completion = if result.published {
                0.
            } else {
                control(
                    r,
                    submit(
                        r,
                        &committed,
                        &working,
                        &display,
                        request,
                        Phase::RegionalControl,
                        0,
                        0,
                        0,
                        n as u32,
                        n,
                    ),
                )?
                .gpu_ms
            };
            ensure!(
                bake::pull(r, &working.probes[1], wb)? == full_weights,
                "Regional coefficients differ from full build: {label}"
            );
            ensure!(
                bake::pull(r, &working.receivers, n * 48)? == full_deps,
                "Regional dependencies differ from full build: {label}"
            );
            ensure!(
                bake::pull(r, &working.guides, n * 4)?
                    .iter()
                    .all(|b| *b == 0),
                "Published dirty coefficients"
            );
            ensure!(
                bake::pull(r, &working.values, n * 16)? == full_rgb,
                "Recombined atlas differs from full coefficient build"
            );
            let (published, _, published_query) = view(
                r,
                &display,
                &scene,
                bp,
                &native,
                &target,
                &format!("{path}/published"),
                "bake-full",
            )?;
            ensure!(
                published == expected,
                "Published receiver field differs from full coefficient reference"
            );
            ensure!(
                fs::read(format!("{path}/published.png"))?
                    == fs::read(format!("{path}/reference.png"))?,
                "Published PNG differs from full coefficient reference"
            );
            // Compare final display to old full-bake accumulation, not only to itself.
            copy(r, &oracle.probes[1], &display.probes[1], n * 16);
            let (plain_view, _, _) = view(
                r,
                &display,
                &scene,
                bp,
                &native,
                &target,
                &format!("{path}/plain"),
                "bake-full",
            )?;
            ensure!(
                max_rgb(&published, &plain_view) <= 0.0002,
                "Published RGB exceeds plain reference bound"
            );
            copy(r, &working.values, &display.probes[1], n * 16);
            events.push(json!({"step":step,"label":label,"budget_ms":budget,"cold_seed_gpu_ms":seeds_cost.1,"cold_coefficients_gpu_ms":cold.1,"dirty":result.dirty,"changed_weight_rows":changed,"visible_last_object_pixels":visible,"frames":result.publish_frame,"budget_completed":result.published,"offline_completion_gpu_ms":offline_completion,"latency_ms":result.latency_ms,"epoch":result.working_epoch,"cancelled":result.cancelled,"rows":result.rows,"ticks":result.ticks,"controls":controls,"initial_mark_gpu_ms":mark.gpu_ms,"full_kernel_gpu_ms":oracle_cost.1,"pending_max_error":max_rgb(&pending,&expected),"held_max_error":max_rgb(&held,&expected),"native_gpu_ms":raster_gpu,"background_native_gpu_ms":background_native,"background_query_gpu_ms":background_query,"baked_query_gpu_ms":query_gpu,"pending_query_gpu_ms":pending_gpu,"published_query_gpu_ms":published_query,"plain_field_max_error":max_rgb(&full_rgb,&plain),"plain_view_max_error":max_rgb(&published,&plain_view),"weights_exact":true,"dependencies_exact":true,"published_exact":true,"background_exact":true,"prepare_cpu_ms":first.prepare_cpu_ms+replacement.as_ref().map_or(0.,|r|r.prepare_cpu_ms)}));
            fs::write(
                format!("{path}/event.json"),
                serde_json::to_vec_pretty(events.last().unwrap())?,
            )?;
            held_png = fs::read(format!("{path}/published.png"))?;
            held = published;
            std::mem::swap(&mut committed, &mut working);
            previous = scene;
            old_boxes = request.boxes.clone();
            old_p = request.p;
            println!(
                "Response budget {budget} {label}: {}/{n} rows, {} frames, {:.1} ms, budget completed {}",
                result.dirty, result.publish_frame, result.latency_ms, result.published
            );
        }
    }
    fs::write(
        format!("{dir}/results.json"),
        serde_json::to_vec_pretty(
            &json!({"width":W,"height":H,"texels":n,"stride_vec4":stride,"weight_bytes_per_bank":wb,"metadata_bytes_per_bank":n*48,"diagnostic_lab_gpu_buffer_bytes":memory,"adapter":r.adapter.get_info().name,"samples":4096,"seed":SEED,"tick_hz":60,"events":events}),
        )?,
    )?;
    Ok(())
}
