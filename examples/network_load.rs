//! Finite real-TLS load probe. All peers receive the same unculled authoritative combat.
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ignis::{
    abilities::Outcome,
    engine::{Engine, ExecutionMode},
    network::{
        client::{Client, Connector, State},
        host::Host,
        protocol::*,
    },
    physics::Input,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
#[path = "support/load_scene.rs"]
mod load_scene;
use load_scene::{Workload, delivery_for, scene};

fn stats(samples: &[f64]) -> Value {
    if samples.is_empty() {
        return Value::Null;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |q: f64| sorted[(sorted.len() as f64 * q).ceil().max(1.) as usize - 1];
    json!({"samples":samples.len(),"min_ms":sorted[0],"mean_ms":samples.iter().sum::<f64>()/samples.len() as f64,"p50_ms":at(0.5),"p95_ms":at(0.95),"p99_ms":at(0.99),"max_ms":sorted.last()})
}
struct Driver {
    stop: Arc<AtomicBool>,
    phase: Arc<AtomicU8>,
    tick: Arc<std::sync::atomic::AtomicU64>,
    thread: Option<std::thread::JoinHandle<Result<Value>>>,
    address: SocketAddr,
    cert: Vec<u8>,
}
impl Driver {
    fn start(players: usize, kind: Workload) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let phase = Arc::new(AtomicU8::new(0));
        let stage = phase.clone();
        let tick = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let observed_tick = tick.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || -> Result<Value> {
            let start = Instant::now();
            let listeners = if kind == Workload::Boss { 7 } else { 8 };
            let authored = scene(players, kind, true, true, listeners)?;
            let mut engine = Engine::new_with_mode(authored, ExecutionMode::Continuous)?;
            let mut host = Host::launch(&mut engine, "127.0.0.1:0".parse()?, players)?;
            let setup_ms = loop {
                host.poll(&mut engine)?;
                let info = host.inspect();
                if info["transport"]["state"] == "listening" {
                    tx.send((info, start.elapsed().as_secs_f64() * 1000.))?;
                    break start.elapsed().as_secs_f64() * 1000.;
                }
                ensure!(
                    start.elapsed() < Duration::from_secs(15),
                    "Host startup deadline"
                );
                std::thread::sleep(Duration::from_millis(2));
            };
            let mut next = Instant::now();
            let mut before = Vec::with_capacity(7200);
            let mut simulation = Vec::with_capacity(7200);
            let mut after = Vec::with_capacity(7200);
            let mut total = Vec::with_capacity(7200);
            let mut peaks = [0; 4];
            let mut heard = BTreeSet::new();
            let mut overrun = 0;
            let mut combat_ticks = 0;
            while !stopping.load(Ordering::Relaxed) {
                ensure!(
                    start.elapsed() < Duration::from_secs(120),
                    "Finite probe deadline"
                );
                if Instant::now() < next {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                let phase = stage.load(Ordering::Relaxed);
                let at = Instant::now();
                host.before_tick(&mut engine)?;
                let admitted = Instant::now();
                engine.advance(1)?;
                let simulated = Instant::now();
                host.after_tick(&mut engine)?;
                let finished = Instant::now();
                observed_tick.store(engine.simulation.tick, Ordering::Relaxed);
                if phase == 1 {
                    combat_ticks += 1;
                    before.push((admitted - at).as_secs_f64() * 1000.);
                    simulation.push((simulated - admitted).as_secs_f64() * 1000.);
                    after.push((finished - simulated).as_secs_f64() * 1000.);
                    total.push((finished - at).as_secs_f64() * 1000.);
                    if finished - at > Duration::from_secs_f64(1. / 60.) {
                        overrun += 1;
                    }
                }
                // Observability is outside the measured world-step interval.
                let state = &engine.simulation.abilities.state;
                for (peak, value) in peaks.iter_mut().zip([
                    state.projectiles.active.len(),
                    state.periodic.active.len(),
                    state.persistent_areas.active.len(),
                    state.buffs.active.len(),
                ]) {
                    *peak = (*peak).max(value);
                }
                for (id, npc) in &engine.simulation.npcs {
                    if npc.heard_this_tick {
                        heard.insert(id.clone());
                    }
                }
                next += Duration::from_secs_f64(1. / 60.);
                if next + Duration::from_millis(100) < Instant::now() {
                    next = Instant::now();
                }
            }
            let health: BTreeMap<_, _> = engine
                .simulation
                .actors
                .actors
                .iter()
                .enumerate()
                .filter_map(|(a, _)| {
                    engine.simulation.vitality.state.actors[a].map(|h| {
                        (
                            engine
                                .simulation
                                .actors
                                .id(engine.simulation.definitions(), a as u16)
                                .to_owned(),
                            h.units,
                        )
                    })
                })
                .collect();
            let min_player_feet = engine
                .simulation
                .definitions()
                .entities
                .iter()
                .filter(|e| e.body == Some(ignis::model::BodyKind::Player))
                .map(|e| engine.simulation.position(&e.id).unwrap()[1] - e.transform.size[1] * 0.5)
                .fold(f32::INFINITY, f32::min);
            let grounded = engine
                .simulation
                .character_controllers
                .values()
                .filter(|s| s.active && s.grounded)
                .count();
            let step_emitters = engine
                .simulation
                .footsteps
                .states
                .values()
                .filter(|s| s.emitted > 0)
                .count();
            let missing_steps: BTreeMap<_,_> = engine.simulation.footsteps.states.iter().filter(|(_,s)|s.emitted==0).map(|(id,s)|(id.clone(),json!({"state":s,"position":engine.simulation.position(id),"motor":engine.simulation.character_controllers.get(id)}))).collect();
            let outcomes: Vec<_> = engine
                .simulation
                .abilities
                .outcome_counts()
                .iter()
                .map(|(o, n)| json!({"outcome":o,"count":n}))
                .collect();
            let result = json!({"build_id":env!("IGNIS_BUILD_ID"),"host":host.inspect(),"setup_ms":setup_ms,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"combat_ticks":combat_ticks,"over_budget_ticks":overrun,"before":stats(&before),"simulation":stats(&simulation),"after":stats(&after),"total":stats(&total),"health":health,"min_player_feet":min_player_feet,"grounded":grounded,"controllers":engine.simulation.character_controllers.len(),"missing_steps":missing_steps,"step_emitters":step_emitters,"footstep_events":engine.simulation.footsteps.total_emitted,"listeners_heard":heard.len(),"listeners":listeners,"active_projectiles":engine.simulation.abilities.state.projectiles.active.len(),"active_periodics":engine.simulation.abilities.state.periodic.active.len(),"active_areas":engine.simulation.abilities.state.persistent_areas.active.len(),"peak_projectiles":peaks[0],"peak_periodics":peaks[1],"peak_areas":peaks[2],"peak_buffs":peaks[3],"ability_noise_events":engine.simulation.abilities.state.emitted,"outcomes":outcomes,"execution":engine.inspect_execution(),"tick":engine.simulation.tick});
            host.close(&mut engine);
            Ok(result)
        });
        let (info, setup) = rx.recv_timeout(Duration::from_secs(20))?;
        eprintln!("Host ready: {players} players, setup {setup:.3}ms");
        Ok(Self {
            stop,
            phase,
            tick,
            thread: Some(thread),
            address: info["transport"]["address"]
                .as_str()
                .context("host address")?
                .parse()?,
            cert: STANDARD.decode(
                info["transport"]["certificate_der"]
                    .as_str()
                    .context("host pin")?,
            )?,
        })
    }
    fn finish(mut self) -> Result<Value> {
        self.stop.store(true, Ordering::Relaxed);
        let report = self
            .thread
            .take()
            .unwrap()
            .join()
            .map_err(|_| anyhow::anyhow!("Host panicked"))??;
        std::fs::write(
            "artifacts/network-load-state.json",
            serde_json::to_vec_pretty(&report)?,
        )?;
        Ok(report)
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take()
            && let Ok(Ok(report)) = t.join()
        {
            let _ = std::fs::write(
                "artifacts/joint-network-failed-state.json",
                report.to_string(),
            );
        }
    }
}
async fn wait_all(users: &[Client], predicate: impl Fn(&State) -> bool) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let mut ready = true;
            for user in users {
                let s = user.state.lock().unwrap();
                ensure!(s.connected, "Client disconnected: {:?}", s.error);
                ready &= predicate(&s);
            }
            if ready {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .context("All-client convergence deadline")?
}
fn movement(users: &[Client], targets: &[Target], sequence: &mut u64, x: f32) -> Result<()> {
    *sequence = sequence.checked_add(1).context("sequence exhaustion")?;
    for (user, target) in users.iter().zip(targets) {
        user.movement(Movement {
            sequence: *sequence,
            life: target.life,
            input: Input {
                x,
                ..Default::default()
            },
        })?;
    }
    Ok(())
}
fn successful(kind: Workload, outcome: Outcome) -> bool {
    match kind {
        Workload::Projectile => outcome == Outcome::Launched,
        // Persistent area admission is Completed; AreaApplied is its later pulse.
        Workload::Area => outcome == Outcome::Completed,
        _ => outcome == Outcome::Completed,
    }
}
async fn run(players: usize, rounds: u64, kind: Workload, endpoints: usize) -> Result<Value> {
    let driver = Driver::start(players, kind)?;
    let connectors: Vec<_> = (0..endpoints)
        .map(|_| Connector::new(driver.address, &driver.cert))
        .collect::<Result<_>>()?;
    let at = Instant::now();
    let mut users = Vec::with_capacity(players);
    for base in (0..players).step_by(16) {
        let mut tasks = tokio::task::JoinSet::new();
        for _ in base..(base + 16).min(players) {
            let c = connectors[base / 16 % endpoints].clone();
            tasks.spawn(async move { c.connect(None).await });
        }
        while let Some(result) = tasks.join_next().await {
            users.push(result??);
        }
    }
    let connection_ms = at.elapsed().as_secs_f64() * 1000.;
    let actors = players + 8;
    wait_all(&users, |s| {
        s.content.is_some() && s.replica.actor_count() == actors && s.owner.is_some()
    })
    .await?;
    users.sort_by_key(|u| {
        let s = u.state.lock().unwrap();
        s.replica.actors()[&s.actor.actor].appearance.label.clone()
    });
    let targets: Vec<_> = users
        .iter()
        .map(|u| u.state.lock().unwrap().actor)
        .collect();
    ensure!(
        targets
            .iter()
            .map(|t| t.actor)
            .collect::<BTreeSet<_>>()
            .len()
            == players,
        "Ownership collision"
    );
    let boss = if kind == Workload::Boss {
        let s = users[0].state.lock().unwrap();
        Some(
            s.replica
                .actors()
                .values()
                .find(|a| a.appearance.label == "zz_boss")
                .map(|a| Target {
                    actor: a.id,
                    life: a.life,
                })
                .context("Boss projection")?,
        )
    } else {
        None
    };
    let mut sequence = 0;
    let mut input_timer = tokio::time::interval(Duration::from_millis(100));
    input_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // All connections are ready before movement/measurement; one second of grounding warmup.
    for _ in 0..10 {
        input_timer.tick().await;
        movement(&users, &targets, &mut sequence, 0.25)?;
    }
    driver.phase.store(1, Ordering::Relaxed);
    let combat_at = Instant::now();
    let mut outcomes = BTreeMap::new();
    let mut latencies = Vec::with_capacity(players * rounds as usize);
    let mut lag = Vec::with_capacity(players * rounds as usize);
    let mut world_lag = Vec::with_capacity(players * rounds as usize);
    let mut completed = 0;
    let mut occluded = 0;
    for id in 1..=rounds {
        let round_at = Instant::now();
        for (i, user) in users.iter().enumerate() {
            let delivery = delivery_for(kind, i);
            user.operation(Operation {
                id,
                life: targets[i].life,
                intent: Intent::Activate {
                    slot: if kind == Workload::Mixed {
                        (i / 2 % 4 + 1) as u8
                    } else {
                        1
                    },
                    target: if kind == Workload::Boss {
                        boss
                    } else if delivery == Workload::Area {
                        None
                    } else {
                        Some(targets[i ^ 1])
                    },
                },
            })?;
        }
        let mut resolved = vec![false; players];
        let mut remaining = players;
        while remaining > 0 {
            ensure!(
                round_at.elapsed() < Duration::from_secs(15),
                "Round {id} unresolved for {remaining} peers"
            );
            for (i, user) in users.iter().enumerate() {
                if resolved[i] {
                    continue;
                }
                let s = user.state.lock().unwrap();
                ensure!(s.connected, "Round {id}: disconnected {:?}", s.error);
                if let Some(result) = s
                    .results
                    .iter()
                    .rev()
                    .find(|r| r.id == id && r.status.terminal())
                {
                    let Status::Resolved { outcome, .. } = result.status else {
                        bail!("Round {id}, player {i}: {:?}", result.status)
                    };
                    if kind == Workload::Boss {
                        match outcome {
                            Outcome::Completed => completed += 1,
                            Outcome::Occluded => occluded += 1,
                            _ => bail!("Unclassified boss outcome {outcome:?}"),
                        }
                    } else {
                        ensure!(
                            successful(delivery_for(kind, i), outcome),
                            "Round {id}, player {i}: {outcome:?}"
                        );
                    }
                    *outcomes.entry(format!("{outcome:?}")).or_insert(0u64) += 1;
                    latencies.push(round_at.elapsed().as_secs_f64() * 1000.);
                    world_lag.push(
                        driver
                            .tick
                            .load(Ordering::Relaxed)
                            .saturating_sub(s.replica.tick().unwrap_or(0))
                            as f64
                            * 1000.
                            / 60.,
                    );
                    lag.push(result.tick.saturating_sub(s.replica.tick().unwrap_or(0)) as f64);
                    resolved[i] = true;
                    remaining -= 1;
                }
            }
            if remaining > 0 {
                tokio::select! {_ = input_timer.tick()=>movement(&users,&targets,&mut sequence,if (combat_at.elapsed().as_secs()/2).is_multiple_of(2){0.25}else{-0.25})?,_ = tokio::time::sleep(Duration::from_millis(5))=>{}}
            }
        }
        // Keep the requested 10Hz action rate when I/O and simulation finish early.
        while round_at.elapsed() < Duration::from_millis(100) {
            tokio::select! {_ = input_timer.tick()=>movement(&users,&targets,&mut sequence,if (combat_at.elapsed().as_secs()/2).is_multiple_of(2){0.25}else{-0.25})?,_ = tokio::time::sleep(Duration::from_millis(2))=>{}}
        }
    }
    let combat_ms = combat_at.elapsed().as_secs_f64() * 1000.;
    driver.phase.store(2, Ordering::Relaxed);
    movement(&users, &targets, &mut sequence, 0.)?;
    let final_hp = if kind == Workload::Boss {
        1_000_000
    } else {
        1_000_000 - rounds as i64
    };
    let boss_hp = 1_000_000 - completed;
    let drain_at = Instant::now();
    wait_all(&users, |s| {
        s.replica.actor_count() == actors
            && s.replica.actors().values().all(|a| {
                if a.appearance.label == "zz_boss" {
                    a.health.is_some_and(|h| h.units == boss_hp)
                } else if a.appearance.player {
                    a.health.is_some_and(|h| h.units == final_hp)
                } else {
                    true
                }
            })
    })
    .await?;
    let drain_ms = drain_at.elapsed().as_secs_f64() * 1000.;
    let mut sent = 0;
    let mut received = 0;
    let mut lost = 0;
    let mut rtt = Vec::with_capacity(players);
    let mut ticks = Vec::with_capacity(players);
    let mut handshake_attempts = BTreeMap::new();
    for user in &users {
        let t = user.transport_stats();
        *handshake_attempts
            .entry(t["handshake_attempts"].as_u64().unwrap())
            .or_insert(0usize) += 1;
        sent += t["udp_sent_bytes"].as_u64().unwrap();
        received += t["udp_received_bytes"].as_u64().unwrap();
        lost += t["lost_packets"].as_u64().unwrap();
        rtt.push(t["rtt_us"].as_f64().unwrap() / 1000.);
        ticks.push(user.state.lock().unwrap().replica.tick().unwrap());
    }
    // Prove every replica still advances, not just an old matching health frame.
    let progress_at = Instant::now();
    for (user, tick) in users.iter().zip(&ticks) {
        wait_all(std::slice::from_ref(user), |s| {
            s.replica.tick().unwrap_or(0) > *tick
        })
        .await?;
    }
    let progress_ms = progress_at.elapsed().as_secs_f64() * 1000.;
    let final_world = driver.tick.load(Ordering::Relaxed);
    let final_replica_ticks: Vec<_> = users
        .iter()
        .map(|u| u.state.lock().unwrap().replica.tick().unwrap())
        .collect();
    ensure!(
        final_world.saturating_sub(*final_replica_ticks.iter().min().unwrap()) <= 12,
        "A replica remained more than two snapshot periods behind after drain"
    );
    let receive_counts: Vec<_> = users
        .iter()
        .map(|u| {
            let s = u.state.lock().unwrap();
            (s.full_frames, s.delta_frames, s.need_baseline)
        })
        .collect();
    let mut report = driver.finish()?;
    ensure!(
        report["controllers"] == players && report["grounded"] == players,
        "KCC grounding mismatch"
    );
    ensure!(
        report["step_emitters"] == players,
        "A player never emitted footsteps"
    );
    ensure!(
        report["listeners_heard"] == report["listeners"],
        "A listener never heard stimuli"
    );
    ensure!(
        report["execution"]["admitted"]["abilities"] == players as u64 * rounds,
        "Admission count mismatch"
    );
    ensure!(
        report["execution"]["retained"]
            .as_object()
            .unwrap()
            .values()
            .all(|n| n == 0),
        "History accumulated"
    );
    for (id, hp) in report["health"].as_object().unwrap() {
        ensure!(
            hp == if id == "zz_boss" { boss_hp } else { final_hp },
            "Authority health mismatch {id}: {hp}"
        );
    }
    ensure!(
        report["active_projectiles"] == 0
            && report["active_periodics"] == 0
            && report["active_areas"] == 0,
        "Delivery drain incomplete"
    );
    ensure!(
        report["execution"]["pending"]
            .as_object()
            .unwrap()
            .values()
            .all(|n| n == 0),
        "Pending commands remain"
    );
    if kind == Workload::Boss {
        ensure!(
            completed + occluded == players as i64 * rounds as i64,
            "Missing boss outcomes"
        );
        ensure!(
            report["peak_buffs"] == players,
            "Not all sources contributed a boss buff"
        );
        ensure!(
            report["ability_noise_events"] == completed,
            "Boss noise/commit mismatch"
        );
    }
    ensure!(
        report["min_player_feet"].as_f64().unwrap() >= -0.003,
        "A player penetrated the floor"
    );
    report["handshake_attempts"] = json!(handshake_attempts);
    report["passed"] = json!(true);
    report["recipe"] = json!(kind.label());
    report["players"] = json!(players);
    report["rounds"] = json!(rounds);
    report["connection_ms"] = json!(connection_ms);
    report["combat_ms"] = json!(combat_ms);
    report["drain_ms"] = json!(drain_ms);
    report["progress_ms"] = json!(progress_ms);
    report["observed_results"] = json!(outcomes);
    report["result_latency"] = stats(&latencies);
    report["world_replica_lag_estimated_ms"] = stats(&world_lag);
    report["final_replica_lag_ticks"] = json!({"world":final_world,"min_client":final_replica_ticks.iter().min(),"max_client":final_replica_ticks.iter().max(),"max_lag":final_world.saturating_sub(*final_replica_ticks.iter().min().unwrap())});
    report["client_receive_counts"] = json!({"full":receive_counts.iter().map(|r|r.0).sum::<u64>(),"delta":receive_counts.iter().map(|r|r.1).sum::<u64>(),"need_baseline":receive_counts.iter().map(|r|r.2).sum::<u64>()});
    report["result_replica_tick_lag"] = json!({"max_ticks":lag.iter().copied().fold(0.,f64::max),"mean_ticks":lag.iter().sum::<f64>()/lag.len() as f64});
    report["transport_rtt"] = stats(&rtt);
    report["udp_client_sent_bytes"] = json!(sent);
    report["udp_client_received_bytes"] = json!(received);
    report["lost_packets"] = json!(lost);
    report["boss_completed"] = json!(completed);
    report["boss_occluded"] = json!(occluded);
    report["client_replica_ticks_before_progress"] =
        json!({"min":ticks.iter().min(),"max":ticks.iter().max()});
    report["limitations"] = json!(
        "Real QUIC/TLS loopback; load-generator clients share two I/O workers and the host machine; no client physics, WAN, FPS, prediction, accounts or durable checkpoint."
    );
    Ok(report)
}
fn main() -> Result<()> {
    let mut players = 1000usize;
    let mut workers = 4usize;
    let mut endpoints = 32usize;
    let mut rounds = 100u64;
    let mut kind = Workload::Mixed;
    let mut output = std::path::PathBuf::from("artifacts/network-load.json");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--endpoints" => endpoints = args.next().context("--endpoints value")?.parse()?,
            "--workers" => workers = args.next().context("--workers value")?.parse()?,
            "--players" => players = args.next().context("--players value")?.parse()?,
            "--rounds" => rounds = args.next().context("--rounds value")?.parse()?,
            "--delivery" => kind = Workload::parse(&args.next().context("--delivery value")?)?,
            "--output" => output = args.next().context("--output value")?.into(),
            _ => bail!("Unknown argument {a}"),
        }
    }
    ensure!(
        (2..=1000).contains(&players) && players.is_multiple_of(2),
        "Players even2..1000"
    );
    ensure!((10..=100).contains(&rounds), "Rounds10..100");
    ensure!(
        matches!(kind, Workload::Mixed | Workload::Boss),
        "Joint network probe supports mixed|boss"
    );
    ensure!((2..=12).contains(&workers), "Client generator workers2..12");
    ensure!((1..=64).contains(&endpoints), "Client endpoints1..64");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()?;
    let report = runtime.block_on(run(players, rounds, kind, endpoints));
    runtime.shutdown_timeout(Duration::from_secs(1));
    let mut report = report?;
    report["client_generator_workers"] = json!(workers);
    report["client_generator_endpoints"] = json!(endpoints);
    std::fs::write(&output, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "{}",
        json!({"passed":true,"output":output,"players":players,"rounds":rounds,"simulation":report["simulation"],"total":report["total"],"result_latency":report["result_latency"],"udp_received_bytes":report["udp_client_received_bytes"]})
    );
    Ok(())
}
