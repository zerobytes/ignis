//! Optional native client, also usable by headless tools. No dependency on Engine.
use super::{
    protocol::*,
    replica::{Applied, Replica},
    transport,
    wire::{self, Control, Hello, Message},
};
use anyhow::{Context, Result, ensure};
use quinn::{Connection, Endpoint};
use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::sync::{mpsc, watch};
#[derive(Clone, Debug)]
pub struct Resume {
    pub session: u64,
    pub secret: Secret,
}
#[derive(Clone)]
pub struct Connector {
    endpoint: Endpoint,
    address: SocketAddr,
}
impl Connector {
    pub fn new(address: SocketAddr, certificate: &[u8]) -> Result<Self> {
        let bind = if address.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let mut endpoint = Endpoint::client(bind.parse()?)?;
        endpoint.set_default_client_config(transport::client_config(certificate)?);
        Ok(Self { endpoint, address })
    }
    pub fn handshakes_started(&self) -> u64 {
        self.endpoint.stats().outgoing_handshakes
    }
    pub async fn connect(&self, resume: Option<Resume>) -> Result<Client> {
        let mut handshake_attempts = 0u8;
        let connection = loop {
            handshake_attempts += 1;
            match tokio::time::timeout(
                wire::DEADLINE,
                self.endpoint.connect(self.address, "ignis.local")?,
            )
            .await
            {
                Ok(result) => break result.context("QUIC handshake failed")?,
                Err(error) if handshake_attempts == 3 => {
                    return Err(error).context("QUIC handshake timeout after 3 attempts");
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100 * u64::from(handshake_attempts)))
                        .await
                }
            }
        };
        let (mut send, mut recv) = connection.open_bi().await?;
        let hello = match resume {
            Some(r) => Hello::Resume {
                version: VERSION,
                session: r.session,
                secret: r.secret,
            },
            None => Hello::Join { version: VERSION },
        };
        wire::write_json(&mut send, &hello)
            .await
            .context("Session hello failed")?;
        let message: Message = tokio::time::timeout(
            wire::DEADLINE,
            wire::read_json(&mut recv, wire::CONTROL_BYTES),
        )
        .await
        .context("Session welcome timeout")?
        .context("Session welcome failed")?;
        let Message::Welcome { welcome } = message else {
            anyhow::bail!("Expected multiplayer welcome")
        };
        ensure!(
            welcome.version == VERSION
                && welcome.world != 0
                && welcome.results.len() <= RESULTS + OPERATIONS
                && welcome.grants.len() <= crate::abilities::GRANTS,
            "Invalid welcome"
        );
        let mut initial = State::connected(&welcome);
        initial.handshake_attempts = handshake_attempts;
        let state = Arc::new(Mutex::new(initial));
        let ticket = Resume {
            session: welcome.session,
            secret: welcome.secret,
        };
        let (tx, rx) = mpsc::channel(OPERATIONS);
        let (input, input_rx) = watch::channel(None);
        let running = connection.clone();
        let status = state.clone();
        tokio::spawn(async move {
            let result = run(running.clone(), send, recv, rx, input_rx, status.clone()).await;
            running.close(0u32.into(), b"client stopped");
            let mut state = status.lock().unwrap();
            state.connected = false;
            if let Err(error) = result {
                state.error = Some(error.to_string());
            }
        });
        Ok(Client {
            connector: self.clone(),
            connection,
            state,
            ticket,
            operations: tx,
            input,
        })
    }
}
pub struct State {
    pub handshake_attempts: u8,
    pub connected: bool,
    pub error: Option<String>,
    pub world: u64,
    pub session: u64,
    pub connection: u64,
    pub actor: Target,
    pub last_operation: u64,
    pub owner: Option<OwnerView>,
    pub grants: Vec<Grant>,
    pub replica: Replica,
    pub content: Option<Arc<super::content::Content>>,
    pub results: VecDeque<OperationResult>,
    pub dropped_results: u64,
    pub full_frames: u64,
    pub delta_frames: u64,
    pub need_baseline: u64,
    // Fault injection is native/local instrumentation; never a server command.
    pub drop_every: u32,
    pub datagrams_seen: u64,
}
impl State {
    fn connected(w: &Welcome) -> Self {
        Self {
            handshake_attempts: 1,
            connected: true,
            error: None,
            world: w.world,
            session: w.session,
            connection: w.connection,
            actor: w.actor,
            last_operation: w.last_operation,
            owner: None,
            grants: w.grants.clone(),
            replica: Replica::new(w.world),
            content: None,
            results: w
                .results
                .iter()
                .rev()
                .take(RESULTS)
                .rev()
                .cloned()
                .collect(),
            dropped_results: w.results.len().saturating_sub(RESULTS) as u64,
            full_frames: 0,
            delta_frames: 0,
            need_baseline: 0,
            drop_every: 0,
            datagrams_seen: 0,
        }
    }
    fn result(&mut self, result: OperationResult) {
        if self.results.len() == RESULTS {
            self.results.pop_front();
            self.dropped_results = self.dropped_results.saturating_add(1);
        }
        // An envelope rejected before admission (for example a sequence jump)
        // must not advertise that ID as the server's consumed operation.
        if !matches!(result.status, Status::Rejected { .. }) {
            self.last_operation = self.last_operation.max(result.id);
        }
        self.results.push_back(result);
    }
    pub fn inspect(&self, actors: bool) -> serde_json::Value {
        serde_json::json!({"handshake_attempts":self.handshake_attempts,"state":if self.connected{"connected"}else{"disconnected"},"error":self.error,"world":self.world,"session":self.session,"connection":self.connection,"actor":self.actor,"last_operation":self.last_operation,"owner":self.owner,"grants":self.grants,"replica_tick":self.replica.tick(),"actor_count":self.replica.actor_count(),"entity_count":self.replica.actors().len(),"projectile_count":self.replica.projectiles().len(),"omitted_projectiles":self.replica.omitted_projectiles(),"content":self.content.as_ref().map(|c|serde_json::json!({"version":c.version,"world":c.world,"camera":c.camera,"assets":c.meshes.assets.len(),"instances":c.meshes.instances.len()})),"actors":actors.then(||self.replica.actors()),"results":self.results,"dropped_results":self.dropped_results,"full_frames":self.full_frames,"delta_frames":self.delta_frames,"need_baseline":self.need_baseline})
    }
}
pub struct Client {
    connector: Connector,
    connection: Connection,
    pub state: Arc<Mutex<State>>,
    ticket: Resume,
    operations: mpsc::Sender<Operation>,
    input: watch::Sender<Option<Movement>>,
}
impl Client {
    pub fn resume(&self) -> Resume {
        self.ticket.clone()
    }
    pub fn operation(&self, operation: Operation) -> Result<()> {
        self.operations
            .try_send(operation)
            .map_err(|_| anyhow::anyhow!("Client operation mailbox full/closed"))
    }
    pub fn movement(&self, movement: Movement) -> Result<()> {
        ensure!(self.state.lock().unwrap().connected, "Client disconnected");
        self.input.send_replace(Some(movement));
        Ok(())
    }
    pub fn disconnect(&self) {
        self.connection.close(0u32.into(), b"client disconnect");
    }
    pub fn transport_stats(&self) -> serde_json::Value {
        let stats = self.connection.stats();
        serde_json::json!({"handshake_attempts":self.state.lock().unwrap().handshake_attempts,"udp_sent_bytes":stats.udp_tx.bytes,"udp_received_bytes":stats.udp_rx.bytes,"lost_packets":stats.path.lost_packets,"rtt_us":stats.path.rtt.as_micros()})
    }
    pub async fn reconnect(&self) -> Result<Client> {
        self.connector.connect(Some(self.resume())).await
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.disconnect();
    }
}
fn apply(
    state: &Arc<Mutex<State>>,
    connection: &Connection,
    bytes: &[u8],
    full: bool,
) -> Result<()> {
    let (world, tick) = {
        let mut state = state.lock().unwrap();
        let content = state.content.clone();
        match state.replica.apply_with_avatars(bytes, |poses| {
            let content = content
                .as_ref()
                .ok_or_else(|| crate::model::invalid("Visual content not ready"))?;
            for p in poses {
                let asset = content
                    .meshes
                    .assets
                    .get(p.asset as usize)
                    .ok_or_else(|| crate::model::invalid("Avatar asset absent"))?;
                p.validate_asset(asset)?;
            }
            Ok(())
        })? {
            Applied::Current => {
                if full {
                    state.full_frames += 1;
                } else {
                    state.delta_frames += 1;
                }
            }
            Applied::NeedBaseline => state.need_baseline += 1,
            Applied::Stale => {}
        }
        (state.world, state.replica.tick())
    };
    connection.send_datagram(wire::ack(world, tick))?;
    Ok(())
}
async fn run(
    connection: Connection,
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    mut operations: mpsc::Receiver<Operation>,
    mut input: watch::Receiver<Option<Movement>>,
    state: Arc<Mutex<State>>,
) -> Result<()> {
    let reader = async {
        loop {
            let message: Message = wire::read_json(&mut recv, wire::CONTROL_BYTES).await?;
            match message {
                Message::Update { results, owner } => {
                    ensure!(results.len() <= RESULTS, "Result budget exceeded");
                    let mut state = state.lock().unwrap();
                    for result in results {
                        state.result(result);
                    }
                    if let Some(owner) = owner {
                        ensure!(
                            owner.actor.actor == state.actor.actor
                                && owner.resources.len() <= crate::abilities::RESOURCES,
                            "Private owner mismatch/budget"
                        );
                        state.actor = owner.actor;
                        state.owner = Some(owner);
                    }
                }
                Message::Rejected { operation, reason } => {
                    state.lock().unwrap().result(OperationResult {
                        id: operation,
                        tick: 0,
                        status: Status::Rejected { reason },
                    });
                }
                Message::Welcome { .. } => anyhow::bail!("Unexpected second welcome"),
            }
        }
    };
    let writer = async {
        let mut ping = tokio::time::interval(Duration::from_secs(2));
        loop {
            tokio::select! {
            operation=operations.recv()=>{let Some(operation)=operation else{return Ok(())};wire::write_json(&mut send,&Control::Operation{operation}).await?;}
            change=input.changed()=>{change?;let movement=*input.borrow_and_update();if let Some(movement)=movement{connection.send_datagram(wire::movement(&movement)?)?;}}
            _=ping.tick()=>{wire::write_json(&mut send,&Control::Ping).await?;let(world,tick)={let s=state.lock().unwrap();(s.world,s.replica.tick())};connection.send_datagram(wire::ack(world,tick))?;}
            }
        }
    };
    let full = async {
        loop {
            let mut stream = connection.accept_uni().await?;
            let (kind, data) = wire::read_blob(&mut stream).await?;
            if kind == 0 {
                let world = state.lock().unwrap().world;
                let content = super::content::Content::decode(world, &data)?;
                let mut state = state.lock().unwrap();
                ensure!(state.content.is_none(), "Unexpected content replacement");
                state.content = Some(Arc::new(content));
            } else {
                apply(&state, &connection, &data, true)?;
            }
        }
    };
    let delta = async {
        let mut assembler = wire::Assembler::default();
        loop {
            let data = connection.read_datagram().await?;
            let (world, drop) = {
                let mut s = state.lock().unwrap();
                s.datagrams_seen += 1;
                (
                    s.world,
                    s.drop_every > 0 && s.datagrams_seen.is_multiple_of(s.drop_every as u64),
                )
            };
            if drop {
                continue;
            }
            if let Some(data) = assembler.push(world, data)? {
                apply(&state, &connection, &data, false)?;
            }
        }
    };
    tokio::select! {r=reader=>r,r=writer=>r,r=full=>r,r=delta=>r}
}
/// Service-side handle: starting/stopping and input submission perform no socket I/O.
pub struct Native {
    pub state: Arc<Mutex<Option<Arc<Mutex<State>>>>>,
    pub error: Arc<Mutex<Option<String>>>,
    operations: mpsc::Sender<Operation>,
    input: watch::Sender<Option<super::controls::InputUpdate>>,
    sequence: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    ticket: Arc<Mutex<Option<Resume>>>,
    address: SocketAddr,
    certificate: Vec<u8>,
}
impl Native {
    pub fn launch(
        address: SocketAddr,
        certificate: Vec<u8>,
        resume: Option<Resume>,
    ) -> Result<Self> {
        ensure!(
            !certificate.is_empty() && certificate.len() <= 16 * 1024,
            "Invalid certificate budget"
        );
        let (state, error, stop, ticket) = (
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(None)),
            Arc::new(AtomicBool::new(false)),
            Arc::new(Mutex::new(None)),
        );
        let (operations, mut rx) = mpsc::channel(OPERATIONS);
        let (input, mut input_rx) = watch::channel(None);
        let sequence = Arc::new(AtomicU64::new(0));
        let input_sequence = sequence.clone();
        let (ts, te, tx, tt, cert) = (
            state.clone(),
            error.clone(),
            stop.clone(),
            ticket.clone(),
            certificate.clone(),
        );
        std::thread::Builder::new().name("ignis-quic-client".into()).spawn(move || {
            let result = (|| -> Result<()> {
                let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?;
                let result = runtime.block_on(async {
                    let connector = Connector::new(address, &cert)?;
                    let cancelled = async {
                        loop {
                            if tx.load(Ordering::Relaxed) { break; }
                            tokio::time::sleep(Duration::from_millis(25)).await;
                        }
                    };
                    let client = tokio::select! {
                        result = connector.connect(resume) => result?,
                        _ = cancelled => return Ok::<_, anyhow::Error>(()),
                    };
                    if tx.load(Ordering::Relaxed) {
                        client.disconnect();
                        return Ok::<_, anyhow::Error>(());
                    }
                    *tt.lock().unwrap() = Some(client.resume());
                    *ts.lock().unwrap() = Some(client.state.clone());
                    let mut timer = tokio::time::interval(Duration::from_millis(25));
                    let mut held=None;let mut last_input=None;
                    loop {
                        tokio::select! {
                            operation = rx.recv() => {
                                let Some(operation) = operation else { break };
                                client.operation(operation)?;
                            }
                            change = input_rx.changed() => {
                                if change.is_err() { break; }
                                let update = *input_rx.borrow_and_update();
                                match update {
                                    Some(super::controls::InputUpdate::Raw(movement))=>{held=None;client.movement(movement)?;input_sequence.fetch_max(movement.sequence,Ordering::Relaxed);},
                                    Some(super::controls::InputUpdate::Hold(hold))=>{held=Some(hold);send_held(&client,&mut held,&input_sequence)?;last_input=Some(std::time::Instant::now());},
                                    None=>held=None,
                                }
                            }
                            _ = timer.tick() => {
                                if tx.load(Ordering::Relaxed) || !client.state.lock().unwrap().connected { break; }
                                if last_input.is_none_or(|at:std::time::Instant|at.elapsed()>=Duration::from_millis(100)) {
                                    send_held(&client,&mut held,&input_sequence)?;last_input=Some(std::time::Instant::now());
                                }
                            }
                        }
                    }
                    client.disconnect();
                    client.state.lock().unwrap().connected = false;
                    Ok::<_, anyhow::Error>(())
                });
                runtime.shutdown_timeout(Duration::from_secs(1));
                result
            })();
            let error = result.err().map(|e|e.to_string());
            if let Some(state) = &*ts.lock().unwrap() {
                let mut state = state.lock().unwrap();
                state.connected = false;
                if error.is_some() { state.error = error.clone(); }
            }
            *te.lock().unwrap() = error;
        })?;
        Ok(Self {
            state,
            error,
            operations,
            input,
            sequence,
            stop,
            ticket,
            address,
            certificate,
        })
    }
    pub fn reconnect(&self) -> Result<Self> {
        let ticket = self
            .ticket
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No resumable session"))?;
        self.stop.store(true, Ordering::Relaxed);
        Self::launch(self.address, self.certificate.clone(), Some(ticket))
    }
    pub fn operation(&self, operation: Operation) -> Result<()> {
        self.operations
            .try_send(operation)
            .map_err(|_| anyhow::anyhow!("Client operation mailbox full/closed"))
    }
    pub fn movement(&self, movement: Movement) -> Result<()> {
        crate::engine::Engine::validate_input(movement.input)?;
        let shared = self
            .state
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Client is not ready"))?;
        let state = shared.lock().unwrap();
        let context = super::controls::Context::current(&state)
            .filter(|c| c.alive)
            .ok_or_else(|| anyhow::anyhow!("Client character is not ready/alive"))?;
        let local_sequence = self.sequence.load(Ordering::Relaxed);
        ensure!(
            movement.life == context.actor.life
                && movement.sequence > local_sequence
                && movement.sequence <= local_sequence.saturating_add(1024),
            "Stale/out-of-window native input"
        );
        drop(state);
        self.input
            .send(Some(super::controls::InputUpdate::Raw(movement)))
            .map_err(|_| anyhow::anyhow!("Client input mailbox closed"))
    }
    pub fn hold(&self, hold: super::controls::Hold) -> Result<()> {
        crate::engine::Engine::validate_input(hold.input)?;
        let shared = self
            .state
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Client is not ready"))?;
        ensure!(
            hold.context.valid(&shared.lock().unwrap()),
            "Remote input context changed"
        );
        self.input
            .send(Some(super::controls::InputUpdate::Hold(hold)))
            .map_err(|_| anyhow::anyhow!("Client input mailbox closed"))
    }
    pub fn inspect(&self, actors: bool) -> serde_json::Value {
        if let Some(state) = &*self.state.lock().unwrap() {
            let mut value = state.lock().unwrap().inspect(actors);
            value["local_input_sequence"] =
                serde_json::json!(self.sequence.load(Ordering::Relaxed));
            value
        } else {
            let error = self.error.lock().unwrap().clone();
            serde_json::json!({"state":if self.stop.load(Ordering::Relaxed){"stopped"}else if error.is_some(){"failed"}else{"connecting"},"error":error})
        }
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn send_held(
    client: &Client,
    held: &mut Option<super::controls::Hold>,
    sequence: &AtomicU64,
) -> Result<()> {
    let Some(hold) = *held else { return Ok(()) };
    let ack = {
        let state = client.state.lock().unwrap();
        if !hold.context.valid(&state) {
            *held = None;
            return Ok(());
        }
        state.owner.as_ref().unwrap().input_ack
    };
    let next = sequence
        .load(Ordering::Relaxed)
        .max(ack)
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("Client input sequence exhausted"))?;
    client.movement(Movement {
        sequence: next,
        life: hold.context.actor.life,
        input: hold.input,
    })?;
    sequence.store(next, Ordering::Relaxed);
    Ok(())
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
