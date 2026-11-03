//! Critical real-socket TLS/ownership/dedup/resumption and bounded-loss gates.
#[path = "support/network.rs"]
mod support;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ignis::{
    engine::Engine,
    network::{
        client::{Client, Connector},
        host::Host,
        protocol::*,
        replica::{Applied, Frame, Replica},
        wire::{Assembler, Hub},
    },
    physics::Input,
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
struct Driver {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<serde_json::Value>>,
    address: std::net::SocketAddr,
    cert: Vec<u8>,
}
impl Driver {
    fn start(players: usize) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || {
            let mut engine = Engine::new(support::scene(players)).unwrap();
            let mut host =
                Host::launch(&mut engine, "127.0.0.1:0".parse().unwrap(), players).unwrap();
            let started = Instant::now();
            loop {
                host.poll(&mut engine).unwrap();
                let info = host.inspect();
                if info["transport"]["state"] == "listening" {
                    tx.send(info).unwrap();
                    break;
                }
                assert!(started.elapsed() < Duration::from_secs(5));
                std::thread::sleep(Duration::from_millis(2));
            }
            let mut next = Instant::now();
            let mut steps = Vec::new();
            while !stopping.load(Ordering::Relaxed) {
                if Instant::now() >= next {
                    let start = Instant::now();
                    host.before_tick(&mut engine).unwrap();
                    engine.advance(1).unwrap();
                    host.after_tick(&mut engine).unwrap();
                    steps.push(start.elapsed().as_secs_f64() * 1000.);
                    next += Duration::from_secs_f64(1. / 60.);
                    if next + Duration::from_millis(100) < Instant::now() {
                        next = Instant::now();
                    }
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            let report = json!({"host":host.inspect(),"ticks":engine.simulation.tick,"hp":engine.simulation.vitality.state.actors.iter().filter_map(|a|a.map(|a|a.units)).collect::<Vec<_>>(),"resources":(0..players).map(|i|engine.simulation.abilities.resource_level(i,0)).collect::<Vec<_>>(),"step_ms":steps,"history":engine.inspect_execution()["retained"]["ticks"]});
            host.close(&mut engine);
            report
        });
        let info = rx.recv_timeout(Duration::from_secs(8)).unwrap();
        Self {
            stop,
            thread: Some(thread),
            address: info["transport"]["address"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
            cert: STANDARD
                .decode(info["transport"]["certificate_der"].as_str().unwrap())
                .unwrap(),
        }
    }
    fn finish(mut self) -> serde_json::Value {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().unwrap()
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take()
            && let Ok(report) = t.join()
            && std::thread::panicking()
        {
            let _ = std::fs::write(
                "artifacts/quic-failed-state.json",
                serde_json::to_vec_pretty(&report).unwrap(),
            );
        }
    }
}
async fn until(client: &Client, condition: impl Fn(&ignis::network::client::State) -> bool) {
    tokio::time::timeout(Duration::from_secs(12), async {
        loop {
            let done = {
                let s = client.state.lock().unwrap();
                assert!(s.connected, "{:?}", s.error);
                condition(&s)
            };
            if done {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}
#[test]
fn real_quic_tls_thousand_capable_sessions_dedup_resume_and_loss() {
    let players: usize = std::env::var("IGNIS_QUIC_PLAYERS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(16);
    assert!((2..=1000).contains(&players) && players.is_multiple_of(2));
    let rounds: u64 = std::env::var("IGNIS_QUIC_ROUNDS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(1);
    assert!((1..=32).contains(&rounds));
    let driver = Driver::start(players);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let alien = rcgen::generate_simple_self_signed(vec!["ignis.local".into()]).unwrap();
        let bad = Connector::new(driver.address, alien.cert.der()).unwrap();
        assert!(bad.connect(None).await.is_err(), "Unpinned server accepted");
        let connector = Connector::new(driver.address, &driver.cert).unwrap();
        let mut users = Vec::with_capacity(players);
        let start = Instant::now();
        // Shared UDP endpoint still creates distinct QUIC/TLS connections, not simulated peers.
        for chunk in 0..players.div_ceil(16) {
            let mut tasks = tokio::task::JoinSet::new();
            for _ in chunk * 16..((chunk + 1) * 16).min(players) {
                let c = connector.clone();
                tasks.spawn(async move {
                    c.connect(None)
                        .await
                        .unwrap_or_else(|e| panic!("Connection batch {chunk}: {e:#}"))
                });
            }
            while let Some(c) = tasks.join_next().await {
                users.push(c.unwrap());
            }
        }
        let connection_ms = start.elapsed().as_secs_f64() * 1000.;
        users.sort_by_key(|c| c.state.lock().unwrap().actor.actor);
        let targets: Vec<_> = users
            .iter()
            .map(|c| c.state.lock().unwrap().actor)
            .collect();
        assert_eq!(
            targets
                .iter()
                .map(|t| t.actor)
                .collect::<BTreeSet<_>>()
                .len(),
            players
        );
        for (i, user) in users.iter().enumerate() {
            user.operation(Operation {
                id: 1,
                life: targets[i].life,
                intent: Intent::Activate {
                    slot: 1,
                    target: Some(targets[i ^ 1]),
                },
            })
            .unwrap();
            user.movement(Movement {
                sequence: 1,
                life: targets[i].life,
                input: Input {
                    x: 0.25,
                    ..Default::default()
                },
            })
            .unwrap();
        }
        for user in &users {
            until(user, |s| {
                s.results.iter().any(|r| {
                    r.id == 1
                        && matches!(
                            r.status,
                            Status::Resolved {
                                outcome: ignis::abilities::Outcome::Completed,
                                ..
                            }
                        )
                })
            })
            .await;
        }
        // All recipients see the full roster; no camera/AOI shortcut in this gate.
        for user in &users {
            until(user, |s| {
                s.replica.actor_count() == players
                    && s.replica.actors().values().all(|a| {
                        !a.appearance.kind.actor() || a.health.is_some_and(|h| h.units == 999)
                    })
                    && s.owner.as_ref().is_some_and(|o| o.resources[0].1 == 99)
            })
            .await;
        }
        let combat_at = Instant::now();
        for id in 2..=rounds {
            for (i, user) in users.iter().enumerate() {
                user.operation(Operation {
                    id,
                    life: targets[i].life,
                    intent: Intent::Activate {
                        slot: 1,
                        target: Some(targets[i ^ 1]),
                    },
                })
                .unwrap();
                user.movement(Movement {
                    sequence: id,
                    life: targets[i].life,
                    input: Input {
                        x: 0.25,
                        ..Default::default()
                    },
                })
                .unwrap();
            }
            for user in &users {
                until(user, |s| {
                    s.results.iter().any(|r| {
                        r.id == id
                            && matches!(
                                r.status,
                                Status::Resolved {
                                    outcome: ignis::abilities::Outcome::Completed,
                                    ..
                                }
                            )
                    })
                })
                .await;
            }
            tokio::time::sleep(Duration::from_millis(80)).await;
        }
        let combat_ms = combat_at.elapsed().as_secs_f64() * 1000.;
        for user in &users {
            until(user, |s| {
                s.replica.actors().values().all(|a| {
                    !a.appearance.kind.actor()
                        || a.health.is_some_and(|h| h.units == 1000 - rounds as i64)
                }) && s
                    .owner
                    .as_ref()
                    .is_some_and(|o| o.resources[0].1 == 100 - rounds as i64)
            })
            .await;
        }
        let ticket = users[0].resume();
        let resumed = connector.connect(Some(ticket)).await.unwrap();
        assert_eq!(
            resumed.state.lock().unwrap().session,
            users[0].state.lock().unwrap().session
        );
        assert!(
            resumed.state.lock().unwrap().connection > users[0].state.lock().unwrap().connection
        );
        let cached_results = resumed.state.lock().unwrap().results.len();
        resumed
            .operation(Operation {
                id: 1,
                life: targets[0].life,
                intent: Intent::Activate {
                    slot: 3,
                    target: Some(targets[1]),
                },
            })
            .unwrap();
        until(&resumed, |s| {
            s.results.len() > cached_results
                && s.results.iter().any(|r| r.id == 1 && r.status.terminal())
        })
        .await;
        until(&resumed, |s| {
            s.owner
                .as_ref()
                .is_some_and(|o| o.resources[0].1 == 100 - rounds as i64)
                && s.replica.actor_count() == players
        })
        .await;
        tokio::time::timeout(Duration::from_secs(5), async {
            while users[0].state.lock().unwrap().connected {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        // Drop application snapshot fragments on the receiving path; later baseline ACK/full heals.
        {
            resumed.state.lock().unwrap().drop_every = 2;
        }
        for sequence in 1..=30 {
            resumed
                .movement(Movement {
                    sequence,
                    life: targets[0].life,
                    input: Input {
                        x: 0.2,
                        ..Default::default()
                    },
                })
                .unwrap();
            tokio::time::sleep(Duration::from_millis(18)).await;
        }
        {
            resumed.state.lock().unwrap().drop_every = 0;
        }
        let tick = resumed.state.lock().unwrap().replica.tick().unwrap();
        until(&resumed, |s| s.replica.tick().unwrap_or(0) > tick + 12).await;
        // A TLS peer that stops consuming streams cannot hold the world hostage.
        let mut raw_endpoint = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        let mut raw_config = ignis::network::transport::client_config(&driver.cert).unwrap();
        let mut limits = ignis::network::transport::transport_config();
        limits
            .stream_receive_window(1024u32.into())
            .receive_window(2048u32.into());
        raw_config.transport_config(Arc::new(limits));
        raw_endpoint.set_default_client_config(raw_config);
        let raw = raw_endpoint
            .connect(driver.address, "ignis.local")
            .unwrap()
            .await
            .unwrap();
        let (mut send, mut receive) = raw.open_bi().await.unwrap();
        let ticket = users[1].resume();
        ignis::network::wire::write_json(
            &mut send,
            &ignis::network::wire::Hello::Resume {
                version: VERSION,
                session: ticket.session,
                secret: ticket.secret,
            },
        )
        .await
        .unwrap();
        let _: ignis::network::wire::Message =
            ignis::network::wire::read_json(&mut receive, ignis::network::wire::CONTROL_BYTES)
                .await
                .unwrap();
        let before_slow = resumed.state.lock().unwrap().replica.tick().unwrap();
        let slow_at = Instant::now();
        tokio::time::timeout(Duration::from_secs(8), raw.closed())
            .await
            .unwrap();
        let slow_closed_ms = slow_at.elapsed().as_secs_f64() * 1000.;
        until(&resumed, |s| {
            s.replica.tick().unwrap_or(0) > before_slow + 60
        })
        .await;
        let mut gpu_capture=None;
        if std::env::var("IGNIS_QUIC_CAPTURE").as_deref()==Ok("1") {
            let camera=ignis::model::Camera{position:[55.,60.,65.],target:[0.,0.,0.],fov_degrees:55.};camera.validate().unwrap();
            let frame={let state=resumed.state.lock().unwrap();let content=state.content.as_ref().unwrap();let mut frame=ignis::network::presentation::empty(content.camera.clone(),content.light.clone());ignis::network::presentation::update(&state,&mut frame,Some(&camera)).unwrap();assert_eq!(frame.boxes.len(),state.replica.actors().len()+state.replica.projectiles().len());assert_eq!(state.replica.actor_count(),players);frame};
            let instance=wgpu::Instance::new(&Default::default());let mut renderer=ignis::render::Renderer::new(&instance,None).await.unwrap();let png=renderer.capture(&frame,960,640).unwrap();std::fs::write("artifacts/network-thousand.png",png).unwrap();gpu_capture=Some(json!({"actors":players,"boxes":frame.boxes.len(),"width":960,"height":640,"camera":camera}));
        }
        let mut udp_sent = raw.stats().udp_tx.bytes;
        let mut udp_received = raw.stats().udp_rx.bytes;
        for user in users.iter().chain(std::iter::once(&resumed)) {
            let stats = user.transport_stats();
            udp_sent += stats["udp_sent_bytes"].as_u64().unwrap();
            udp_received += stats["udp_received_bytes"].as_u64().unwrap();
        }
        let replica_tick = resumed.state.lock().unwrap().replica.tick();
        drop(resumed);
        drop(users);
        let report = driver.finish();
        assert!(
            report["hp"]
                .as_array()
                .unwrap()
                .iter()
                .all(|h| h == 1000 - rounds)
        );
        assert_eq!(report["history"], 0);
        let mut output = report;
        output["players"] = json!(players);
        output["combat_rounds"] = json!(rounds);
        output["combat_ms"] = json!(combat_ms);
        output["connection_ms"] = json!(connection_ms);
        output["udp_client_sent_bytes"] = json!(udp_sent);
        output["udp_client_received_bytes"] = json!(udp_received);
        output["replica_tick"] = json!(replica_tick);
        output["slow_closed_ms"] = json!(slow_closed_ms);
        output["gpu_capture"]=json!(gpu_capture);
        std::fs::write(
            "artifacts/quic-real-load.json",
            serde_json::to_vec_pretty(&output).unwrap(),
        )
        .unwrap();
    });
    runtime.shutdown_timeout(Duration::from_secs(1));
}
#[test]
fn fragment_reorder_duplicates_limits_and_lost_baseline_are_atomic() {
    let e = Engine::new(support::scene(1000)).unwrap();
    let full = Arc::new(Frame::capture(&e, 42, None).unwrap());
    let mut hub = Hub::default();
    let first = hub.packet(full.clone(), None).unwrap().unwrap();
    let mut replica = Replica::new(42);
    assert_eq!(replica.apply(&first.data).unwrap(), Applied::Current);
    let mut changed = (*full).clone();
    changed.tick = 6;
    for actor in &mut changed.actors {
        actor.millimeters[0] += 100;
    }
    let delta = Arc::new(changed);
    let packet = hub.packet(delta, Some(0)).unwrap().unwrap();
    assert!(!packet.full);
    assert!(packet.fragments.len() > 1);
    let mut assembler = Assembler::default();
    let mut assembled = None;
    for part in packet.fragments.iter().rev() {
        let first = assembler.push(42, part.clone()).unwrap();
        if first.is_some() {
            assembled = first;
            break;
        }
        assert!(assembler.push(42, part.clone()).unwrap().is_none());
    }
    let assembled = assembled.unwrap();
    assert_eq!(replica.apply(&assembled).unwrap(), Applied::Current);
    let before = replica.clone();
    let mut malformed = packet.fragments[0].to_vec();
    malformed[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(assembler.push(42, malformed.into()).is_err());
    assert_eq!(before, replica);
    assert!(assembler.push(43, packet.fragments[0].clone()).is_err());
    let mut empty = Replica::new(42);
    assert_eq!(empty.apply(&assembled).unwrap(), Applied::NeedBaseline);
    assert_eq!(empty.tick(), None);
}

// Critical: a lost initial TLS attempt may retry, but must not repeat Join,
// ownership or gameplay, and a bad pin must fail without retrying security errors.
#[test]
fn initial_handshake_retry_survives_a_blackhole_without_repeating_join_or_damage() {
    let driver = Driver::start(2);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let front=tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let back=tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let proxy=front.local_addr().unwrap();let host=driver.address;
        let enabled=Arc::new(AtomicBool::new(false));let forward=enabled.clone();
        let dropped=Arc::new(std::sync::atomic::AtomicU64::new(0));let drops=dropped.clone();
        let relay=tokio::spawn(async move {
            let mut client=None;let mut a=[0u8;65536];let mut b=[0u8;65536];
            loop {tokio::select! {
                r=front.recv_from(&mut a)=>{let(n,from)=r.unwrap();client=Some(from);if forward.load(Ordering::Relaxed){back.send_to(&a[..n],host).await.unwrap();}else{drops.fetch_add(1,Ordering::Relaxed);}},
                r=back.recv_from(&mut b)=>{let(n,from)=r.unwrap();assert_eq!(from,host);if let Some(client)=client{front.send_to(&b[..n],client).await.unwrap();}}
            }}
        });
        let connector=Connector::new(proxy,&driver.cert).unwrap();
        let restore=async {tokio::time::sleep(Duration::from_millis(3050)).await;enabled.store(true,Ordering::Relaxed);};
        let (client,())=tokio::join!(connector.connect(None),restore);let client=client.unwrap();
        assert_eq!(client.state.lock().unwrap().handshake_attempts,2);
        assert_eq!(connector.handshakes_started(),2);
        assert!(dropped.load(Ordering::Relaxed)>0);
        until(&client,|s|s.replica.actor_count()==2&&s.owner.is_some()).await;
        let (life,target)={let s=client.state.lock().unwrap();let target=s.replica.actors().values().find(|a|a.appearance.label=="p0001").unwrap();(s.actor.life,Target{actor:target.id,life:target.life})};
        let op=Operation{id:1,life,intent:Intent::Activate{slot:1,target:Some(target)}};
        client.operation(op).unwrap();until(&client,|s|s.results.iter().any(|r|r.id==1&&r.status.terminal())).await;
        client.operation(Operation{intent:Intent::Activate{slot:3,target:Some(target)},..op}).unwrap();
        until(&client,|s|s.results.iter().filter(|r|r.id==1&&r.status.terminal()).count()>=2).await;
        until(&client,|s|s.owner.as_ref().is_some_and(|o|o.resources[0].1==99)&&s.replica.actors()[&target.actor].health.is_some_and(|h|h.units==999)).await;
        let alien=rcgen::generate_simple_self_signed(vec!["ignis.local".into()]).unwrap();
        let bad=Connector::new(driver.address,alien.cert.der()).unwrap();assert!(bad.connect(None).await.is_err());assert_eq!(bad.handshakes_started(),1);
        // Explicit native disconnect while TLS is blackholed cancels the retry loop.
        let blackhole=tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address=blackhole.local_addr().unwrap();
        let packets=Arc::new(std::sync::atomic::AtomicU64::new(0));let seen=packets.clone();
        let sink=tokio::spawn(async move{let mut bytes=[0u8;65536];loop{blackhole.recv_from(&mut bytes).await.unwrap();seen.fetch_add(1,Ordering::Relaxed);}});
        let native=ignis::network::client::Native::launch(address,driver.cert.clone(),None).unwrap();
        tokio::time::timeout(Duration::from_secs(2),async{while packets.load(Ordering::Relaxed)==0{tokio::time::sleep(Duration::from_millis(10)).await;}}).await.unwrap();
        drop(native);tokio::time::sleep(Duration::from_millis(200)).await;
        let stopped=packets.load(Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(3200)).await;
        assert_eq!(packets.load(Ordering::Relaxed),stopped,"Disconnected native client retried TLS");
        sink.abort();
        let report=driver.finish();
        assert_eq!(report["host"]["domain"]["sessions"].as_array().unwrap().len(),1);
        assert_eq!(report["hp"],json!([1000,999]));
        let proof=json!({"passed":true,"build_id":env!("IGNIS_BUILD_ID"),"initial_packets_dropped":dropped.load(Ordering::Relaxed),"handshakes":connector.handshakes_started(),"bad_pin_handshakes":bad.handshakes_started(),"host":report});
        std::fs::write("artifacts/handshake-recovery-verification.json",serde_json::to_vec_pretty(&proof).unwrap()).unwrap();
        relay.abort();
    });
    runtime.shutdown_timeout(Duration::from_secs(1));
}
