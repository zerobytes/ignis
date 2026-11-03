//! QUIC backend. Sockets, TLS, serialization and packet encoding never touch Engine.
use super::{
    protocol::*,
    replica::Frame,
    wire::{self, Control, Hello, Message},
};
use anyhow::{Result, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use quinn::{Connection, Endpoint};
use serde::Serialize;
use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Notify, Semaphore, mpsc, oneshot, watch},
    task::JoinSet,
};
#[derive(Debug, Clone, Serialize)]
pub struct Info {
    pub state: String,
    pub address: Option<String>,
    pub certificate_der: Option<String>,
    pub error: Option<String>,
}
impl Default for Info {
    fn default() -> Self {
        Self {
            state: "starting".into(),
            address: None,
            certificate_der: None,
            error: None,
        }
    }
}
#[derive(Default)]
pub struct Counters {
    pub accepted: AtomicU64,
    pub closed: AtomicU64,
    pub sent_bytes: AtomicU64,
    pub received_bytes: AtomicU64,
    pub full_frames: AtomicU64,
    pub delta_frames: AtomicU64,
    pub reordered_acks: AtomicU64,
}
#[derive(Default)]
pub struct Inbox {
    pub operations: VecDeque<Operation>,
    pub movement: Option<(Movement, u64)>,
}
#[derive(Default)]
pub struct Outbox {
    pub results: VecDeque<OperationResult>,
    pub owner: Option<OwnerView>,
}
pub struct Link {
    pub inbox: Mutex<Inbox>,
    pub outbox: Mutex<Outbox>,
    pub closed: AtomicBool,
    pub notify: Notify,
    ack: Mutex<Option<u64>>,
}
impl Default for Link {
    fn default() -> Self {
        Self::new()
    }
}
impl Link {
    pub fn new() -> Self {
        Self {
            inbox: Mutex::new(Inbox {
                operations: VecDeque::with_capacity(OPERATIONS),
                movement: None,
            }),
            outbox: Mutex::new(Outbox {
                results: VecDeque::with_capacity(RESULTS),
                owner: None,
            }),
            closed: AtomicBool::new(false),
            notify: Notify::new(),
            ack: Mutex::new(None),
        }
    }
    /// Replica ticks never regress within a connection. Delayed UDP ACKs cannot
    /// force an already-current client back onto an obsolete baseline.
    fn acknowledge(&self, tick: Option<u64>) -> bool {
        let mut current = self.ack.lock().unwrap();
        let regressed = current.is_some_and(|old| tick.is_none_or(|new| new < old));
        if let Some(tick) = tick
            && current.is_none_or(|old| tick > old)
        {
            *current = Some(tick);
        }
        regressed
    }
    pub fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
        self.notify.notify_one();
    }
}
pub struct Open {
    pub hello: Hello,
    pub reply: oneshot::Sender<Result<(Welcome, Arc<Link>), Rejection>>,
}
pub struct Backend {
    pub incoming: mpsc::Receiver<Open>,
    pub frames: watch::Sender<Option<Arc<Frame>>>,
    pub info: Arc<Mutex<Info>>,
    pub counters: Arc<Counters>,
    pub hub: Arc<Mutex<wire::Hub>>,
    stop: Arc<AtomicBool>,
    pub clock: Instant,
}
impl Backend {
    pub fn launch(
        address: SocketAddr,
        capacity: usize,
        content: Arc<super::content::Content>,
    ) -> Result<Self> {
        let (incoming_tx, incoming) = mpsc::channel(64);
        let (frames, rx) = watch::channel(None);
        let info = Arc::new(Mutex::new(Info::default()));
        let counters = Arc::new(Counters::default());
        let hub = Arc::new(Mutex::new(wire::Hub::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let clock = Instant::now();
        let (ti, tc, th, ts) = (info.clone(), counters.clone(), hub.clone(), stop.clone());
        std::thread::Builder::new()
            .name("ignis-quic-host".into())
            .spawn(move || {
                let result = (|| -> Result<()> {
                    let content: bytes::Bytes = content.encode()?.into();
                    let runtime = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build()?;
                    let result = runtime.block_on(serve(
                        address,
                        capacity,
                        incoming_tx,
                        rx,
                        ti.clone(),
                        tc,
                        th,
                        ts,
                        clock,
                        content,
                    ));
                    runtime.shutdown_timeout(Duration::from_secs(1));
                    result
                })();
                let mut info = ti.lock().unwrap();
                if let Err(error) = result {
                    info.state = "failed".into();
                    info.error = Some(error.to_string());
                } else {
                    info.state = "stopped".into();
                }
            })?;
        Ok(Self {
            incoming,
            frames,
            info,
            counters,
            hub,
            stop,
            clock,
        })
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
    pub fn now_ms(&self) -> u64 {
        self.clock.elapsed().as_millis().min(u64::MAX as u128) as u64
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.stop();
    }
}
pub fn transport_config() -> quinn::TransportConfig {
    let mut cfg = quinn::TransportConfig::default();
    cfg.max_concurrent_bidi_streams(1u32.into())
        .max_concurrent_uni_streams(1u32.into())
        .stream_receive_window((128u32 * 1024).into())
        .receive_window((256u32 * 1024).into())
        .send_window(64 * 1024)
        .datagram_receive_buffer_size(Some(64 * 1024))
        .datagram_send_buffer_size(64 * 1024)
        .max_idle_timeout(Some(quinn::VarInt::from_u32(10_000).into()))
        .keep_alive_interval(Some(Duration::from_secs(2)));
    cfg
}
pub fn client_config(certificate: &[u8]) -> Result<quinn::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(rustls::pki_types::CertificateDer::from(
        certificate.to_vec(),
    ))?;
    let mut crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    crypto.alpn_protocols = vec![b"ignis/2".to_vec()];
    crypto.enable_early_data = false;
    let mut cfg = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?,
    ));
    cfg.transport_config(Arc::new(transport_config()));
    Ok(cfg)
}
#[allow(clippy::too_many_arguments)]
async fn serve(
    address: SocketAddr,
    capacity: usize,
    incoming: mpsc::Sender<Open>,
    frames: watch::Receiver<Option<Arc<Frame>>>,
    info: Arc<Mutex<Info>>,
    counters: Arc<Counters>,
    hub: Arc<Mutex<wire::Hub>>,
    stop: Arc<AtomicBool>,
    clock: Instant,
    content: bytes::Bytes,
) -> Result<()> {
    let cert = rcgen::generate_simple_self_signed(vec!["ignis.local".into()])?;
    let der = cert.cert.der().clone();
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
    let mut crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![der.clone()], key.into())?;
    crypto.alpn_protocols = vec![b"ignis/2".to_vec()];
    // Game operations cannot be replayable TLS early data.
    crypto.max_early_data_size = 0;
    let mut config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(crypto)?,
    ));
    config.transport_config(Arc::new(transport_config()));
    config
        .max_incoming(64)
        .incoming_buffer_size(16 * 1024)
        .incoming_buffer_size_total(1024 * 1024);
    let endpoint = Endpoint::server(config, address)?;
    {
        let mut state = info.lock().unwrap();
        state.state = "listening".into();
        state.address = Some(endpoint.local_addr()?.to_string());
        state.certificate_der = Some(STANDARD.encode(der));
    }
    let slots = Arc::new(Semaphore::new(capacity + 32));
    let handshakes = Arc::new(Semaphore::new(32));
    let mut tasks = JoinSet::new();
    let mut timer = tokio::time::interval(Duration::from_millis(25));
    loop {
        tokio::select! {
        _=timer.tick()=>{if stop.load(Ordering::Relaxed){break;}}
        Some(_)=tasks.join_next(),if !tasks.is_empty()=>{}
        connection=endpoint.accept()=>{let Some(connection)=connection else{break};
        if !connection.remote_address_validated(){let _=connection.retry();continue;}
        let Ok(slot)=slots.clone().try_acquire_owned()else{connection.refuse();continue;};let Ok(handshake)=handshakes.clone().try_acquire_owned()else{connection.refuse();continue;};
        let(tx,frames,counters,hub,content)=(incoming.clone(),frames.clone(),counters.clone(),hub.clone(),content.clone());tasks.spawn(async move{let _slot=slot;
        let Ok(Ok(connection))=tokio::time::timeout(wire::DEADLINE,connection).await else{return};drop(handshake);counters.accepted.fetch_add(1,Ordering::Relaxed);
        let _=session(connection.clone(),tx,frames,counters.clone(),hub,clock,content).await;connection.close(0u32.into(),b"session closed");counters.closed.fetch_add(1,Ordering::Relaxed);
        });}
        }
    }
    endpoint.close(0u32.into(), b"host stopped");
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    let _ = tokio::time::timeout(Duration::from_secs(1), endpoint.wait_idle()).await;
    Ok(())
}
struct CloseLink(Arc<Link>);
impl Drop for CloseLink {
    fn drop(&mut self) {
        self.0.close();
    }
}
async fn session(
    connection: Connection,
    incoming: mpsc::Sender<Open>,
    frames: watch::Receiver<Option<Arc<Frame>>>,
    counters: Arc<Counters>,
    hub: Arc<Mutex<wire::Hub>>,
    clock: Instant,
    content: bytes::Bytes,
) -> Result<()> {
    let (mut send, mut recv) =
        tokio::time::timeout(wire::DEADLINE, connection.accept_bi()).await??;
    let hello: Hello = tokio::time::timeout(
        wire::DEADLINE,
        wire::read_json(&mut recv, wire::CLIENT_BYTES),
    )
    .await??;
    let version = match &hello {
        Hello::Join { version } | Hello::Resume { version, .. } => *version,
    };
    ensure!(version == VERSION, "Unsupported multiplayer version");
    let (reply, answer) = oneshot::channel();
    tokio::time::timeout(wire::DEADLINE, incoming.send(Open { hello, reply })).await??;
    let (welcome, link) = tokio::time::timeout(wire::DEADLINE, answer)
        .await??
        .map_err(|r| anyhow::anyhow!("Session rejected: {r:?}"))?;
    let _guard = CloseLink(link.clone());
    let world = welcome.world;
    wire::write_json(&mut send, &Message::Welcome { welcome }).await?;
    let reader = async {
        let mut rate = wire::Rate::new(128);
        loop {
            let control: Control = tokio::time::timeout(
                Duration::from_secs(10),
                wire::read_json(&mut recv, wire::CLIENT_BYTES),
            )
            .await??;
            ensure!(rate.admit(), "Control rate exceeded");
            match control {
                Control::Ping => {}
                Control::Operation { operation } => {
                    let mut inbox = link.inbox.lock().unwrap();
                    ensure!(
                        inbox.operations.len() < OPERATIONS,
                        "Operation mailbox full"
                    );
                    inbox.operations.push_back(operation);
                }
            }
        }
    };
    let datagrams = async {
        let mut rate = wire::Rate::new(128);
        loop {
            let bytes = connection.read_datagram().await?;
            counters
                .received_bytes
                .fetch_add(bytes.len() as u64, Ordering::Relaxed);
            ensure!(rate.admit(), "Datagram rate exceeded");
            match wire::datagram(&bytes)? {
                wire::Datagram::Ack { world: w, tick } => {
                    ensure!(w == world, "ACK world mismatch");
                    if link.acknowledge(tick) {
                        counters.reordered_acks.fetch_add(1, Ordering::Relaxed);
                    }
                }
                wire::Datagram::Movement(m) => {
                    let mut inbox = link.inbox.lock().unwrap();
                    if inbox
                        .movement
                        .as_ref()
                        .is_none_or(|(old, _)| m.sequence > old.sequence)
                    {
                        inbox.movement =
                            Some((m, clock.elapsed().as_millis().min(u64::MAX as u128) as u64));
                    }
                }
            }
        }
    };
    let writer = async {
        let mut timer = tokio::time::interval(Duration::from_millis(50));
        loop {
            tokio::select! {_=link.notify.notified()=>{},_=timer.tick()=>{}}
            ensure!(
                !link.closed.load(Ordering::Relaxed),
                "Session no longer current"
            );
            let (results, owner) = {
                let mut out = link.outbox.lock().unwrap();
                (out.results.drain(..).collect::<Vec<_>>(), out.owner.take())
            };
            if !results.is_empty() || owner.is_some() {
                wire::write_json(&mut send, &Message::Update { results, owner }).await?;
            }
        }
    };
    let snapshots = snapshots(
        connection.clone(),
        link.clone(),
        frames,
        counters.clone(),
        hub,
        content,
    );
    tokio::select! {r=reader=>r,r=datagrams=>r,r=writer=>r,r=snapshots=>r}
}
async fn snapshots(
    connection: Connection,
    link: Arc<Link>,
    mut frames: watch::Receiver<Option<Arc<Frame>>>,
    counters: Arc<Counters>,
    hub: Arc<Mutex<wire::Hub>>,
    content: bytes::Bytes,
) -> Result<()> {
    wire::write_blob(&connection, 0, &content).await?;
    let mut last_full = None::<Instant>;
    loop {
        let frame = frames.borrow_and_update().clone();
        if let Some(frame) = frame {
            let ack = *link.ack.lock().unwrap();
            let packet = hub.lock().unwrap().packet(frame, ack)?;
            if let Some(packet) = packet {
                if packet.full {
                    if last_full.is_none_or(|at| at.elapsed() >= Duration::from_secs(1)) {
                        wire::write_blob(&connection, 1, &packet.data).await?;
                        counters
                            .sent_bytes
                            .fetch_add(packet.data.len() as u64, Ordering::Relaxed);
                        counters.full_frames.fetch_add(1, Ordering::Relaxed);
                        last_full = Some(Instant::now());
                    }
                } else {
                    for fragment in &packet.fragments {
                        connection.send_datagram(fragment.clone())?;
                        counters
                            .sent_bytes
                            .fetch_add(fragment.len() as u64, Ordering::Relaxed);
                    }
                    counters.delta_frames.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        frames.changed().await?;
    }
}

#[cfg(test)]
mod ack_tests {
    use super::*;
    use crate::{
        engine::Engine,
        network::{
            replica::{Applied, Frame, Replica},
            wire::Hub,
        },
    };
    mod support {
        use crate as ignis;
        include!("../../tests/support/network.rs");
    }
    #[test]
    fn reordered_ack_does_not_poison_the_next_delta_for_an_already_current_replica() {
        let engine = Engine::new(support::scene(2)).unwrap();
        let link = Link::new();
        let mut hub = Hub::default();
        let mut replica = Replica::new(55);
        let first = Arc::new(Frame::capture(&engine, 55, None).unwrap());
        let packet = hub.packet(first.clone(), None).unwrap().unwrap();
        assert!(packet.full);
        assert_eq!(replica.apply(&packet.data).unwrap(), Applied::Current);
        assert!(!link.acknowledge(Some(0)));
        let mut second = (*first).clone();
        second.tick = 6;
        let moving = second
            .actors
            .iter_mut()
            .find(|a| a.appearance.player)
            .unwrap();
        let id = moving.id;
        moving.millimeters[0] += 10;
        let second = Arc::new(second);
        let packet = hub
            .packet(second.clone(), *link.ack.lock().unwrap())
            .unwrap()
            .unwrap();
        assert!(!packet.full);
        assert_eq!(replica.apply(&packet.data).unwrap(), Applied::Current);
        assert_eq!(replica.tick(), Some(6));
        assert!(!link.acknowledge(Some(6)));
        assert!(!link.acknowledge(Some(6)));
        // Both can arrive after ACK6 over an unordered/lossy UDP path.
        assert!(link.acknowledge(Some(0)));
        assert!(link.acknowledge(None));
        let mut third = (*second).clone();
        third.tick = 12;
        third
            .actors
            .iter_mut()
            .find(|a| a.id == id)
            .unwrap()
            .millimeters[0] += 10;
        let third = Arc::new(third);
        let packet = hub
            .packet(third.clone(), *link.ack.lock().unwrap())
            .unwrap()
            .unwrap();
        assert!(!packet.full);
        assert_eq!(replica.apply(&packet.data).unwrap(), Applied::Current);
        assert_eq!(replica.tick(), Some(12));
        assert_eq!(
            &replica.actors()[&id],
            third.actors.iter().find(|a| a.id == id).unwrap()
        );
    }
}
