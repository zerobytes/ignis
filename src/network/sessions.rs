//! One authoritative owner per Player. This module performs no socket I/O.
use super::protocol::*;
use crate::{
    abilities::{Outcome, RequestResult},
    engine::{Command, Engine, ExecutionMode},
    identity::Reference,
    model::{BodyKind, Result, invalid},
    physics::Input,
};
use std::collections::{BTreeMap, VecDeque};
type NetResult<T> = std::result::Result<T, Rejection>;
struct Session {
    actor: u16,
    reference: Reference,
    handle: Option<String>,
    secret: Secret,
    connection: u64,
    connected: bool,
    disconnected_at: Option<u64>,
    input_sequence: u64,
    input_ack: u64,
    latest: Option<(Movement, u64)>,
    input_at: Option<u64>,
    last_operation: u64,
    queue: VecDeque<Operation>,
    pending: BTreeMap<u64, OperationResult>,
    recent: VecDeque<OperationResult>,
    outgoing: VecDeque<OperationResult>,
    slow: bool,
}
impl Session {
    fn result(&self, id: u64) -> Option<OperationResult> {
        self.pending
            .get(&id)
            .cloned()
            .or_else(|| self.recent.iter().find(|r| r.id == id).cloned())
    }
    fn record(&mut self, result: OperationResult) {
        if result.status.terminal() {
            self.pending.remove(&result.id);
            if let Some(old) = self.recent.iter_mut().find(|r| r.id == result.id) {
                *old = result.clone();
            } else {
                if self.recent.len() == RESULTS {
                    self.recent.pop_front();
                }
                self.recent.push_back(result.clone());
            }
        } else {
            self.pending.insert(result.id, result.clone());
        }
        if self.connected {
            if self.outgoing.len() < RESULTS {
                self.outgoing.push_back(result);
            } else {
                self.slow = true;
            }
        }
    }
}
/// Session state is transient. Durable accounts/characters are not implied by a lease.
pub struct Authority {
    scope: String,
    world: u64,
    next_session: u64,
    max_sessions: usize,
    sessions: BTreeMap<u64, Session>,
    owners: BTreeMap<u64, u64>,
    requests: BTreeMap<u64, (u64, u64)>,
    last_result_tick: Option<u64>,
    closed: bool,
}
impl Authority {
    pub fn new(engine: &mut Engine, max_sessions: usize) -> Result<Self> {
        if !engine.scene.wind.is_empty() {
            return Err(invalid(
                "Wind/free bodies are not yet part of the replicated authority",
            ));
        }
        if !(1..=crate::targeting::MAX_PLAYERS).contains(&max_sessions) {
            return Err(invalid("Invalid session capacity"));
        }
        if engine.simulation.abilities.request_results_enabled() {
            return Err(invalid("An invocation observer already owns this runtime"));
        }
        let mut bytes = [0; 8];
        getrandom::fill(&mut bytes).map_err(|_| invalid("World epoch entropy unavailable"))?;
        engine.set_execution_mode(ExecutionMode::Continuous)?;
        engine.simulation.abilities.enable_request_results(true);
        Ok(Self {
            scope: engine.runtime_scope().into(),
            world: u64::from_le_bytes(bytes).max(1),
            next_session: 1,
            max_sessions,
            sessions: BTreeMap::new(),
            owners: BTreeMap::new(),
            requests: BTreeMap::new(),
            last_result_tick: None,
            closed: false,
        })
    }
    pub fn world(&self) -> u64 {
        self.world
    }
    pub fn valid(&self, engine: &Engine) -> bool {
        !self.closed && self.scope == engine.runtime_scope()
    }
    fn check(&self, engine: &Engine) -> NetResult<()> {
        if self.closed {
            Err(Rejection::Unavailable)
        } else if self.scope != engine.runtime_scope() {
            Err(Rejection::WorldChanged)
        } else {
            Ok(())
        }
    }
    fn session(&self, id: u64, connection: u64) -> NetResult<&Session> {
        let session = self.sessions.get(&id).ok_or(Rejection::UnknownSession)?;
        if !session.connected || session.connection != connection {
            return Err(Rejection::OldConnection);
        }
        Ok(session)
    }
    fn welcome(&self, engine: &Engine, id: u64) -> Welcome {
        let s = &self.sessions[&id];
        Welcome {
            version: VERSION,
            world: self.world,
            session: id,
            connection: s.connection,
            actor: Target {
                actor: s.reference.instance,
                life: engine.simulation.vitality.state.lives[s.actor as usize],
            },
            secret: s.secret.clone(),
            last_operation: s.last_operation,
            results: s.recent.iter().chain(s.pending.values()).cloned().collect(),
            grants: engine.simulation.definitions().entities
                [engine.simulation.actors.entity(s.actor)]
            .abilities
            .as_ref()
            .map_or_else(Vec::new, |loadout| {
                loadout
                    .grants
                    .iter()
                    .enumerate()
                    .map(|(slot, id)| Grant {
                        slot: slot as u8 + 1,
                        id: id.clone(),
                        selection: engine
                            .simulation
                            .definitions()
                            .ability_catalog
                            .abilities
                            .iter()
                            .find(|d| d.id == *id)
                            .expect("compiled grant")
                            .selection,
                    })
                    .collect()
            }),
        }
    }
    pub fn join(&mut self, engine: &mut Engine, now_ms: u64) -> Result<Welcome> {
        self.check(engine)
            .map_err(|_| invalid("Session world is not current"))?;
        self.expire(engine, now_ms);
        if self.sessions.len() >= self.max_sessions {
            return Err(invalid("Session capacity exhausted"));
        }
        let next = self
            .next_session
            .checked_add(1)
            .ok_or_else(|| invalid("Session identity exhausted"))?;
        let secret = Secret::new()?;
        let candidates: Vec<_> = engine
            .simulation
            .actors
            .order()
            .iter()
            .copied()
            .filter_map(|a| {
                let entity = engine.simulation.actors.entity(a);
                let reference = engine.simulation.actors.entity_reference(entity)?;
                (engine.simulation.definitions().entities[entity].body == Some(BodyKind::Player)
                    && !self.owners.contains_key(&reference.instance)
                    && !engine.simulation.vitality.dead(a as usize))
                .then_some((a, reference))
            })
            .collect();
        let mut assigned = None;
        for (actor, reference) in candidates {
            let name = engine
                .simulation
                .actors
                .id(engine.simulation.definitions(), actor)
                .to_owned();
            if let Ok(value) = engine.execute(Command::BindControl { actor: name }.into()) {
                assigned = Some((
                    actor,
                    reference,
                    value["control"]
                        .as_str()
                        .expect("control result")
                        .to_owned(),
                ));
                break;
            }
        }
        let (actor, reference, handle) =
            assigned.ok_or_else(|| invalid("No unowned live Player is available"))?;
        let id = self.next_session;
        self.next_session = next;
        self.owners.insert(reference.instance, id);
        self.sessions.insert(
            id,
            Session {
                actor,
                reference,
                handle: Some(handle),
                secret,
                connection: 1,
                connected: true,
                disconnected_at: None,
                input_sequence: 0,
                input_ack: 0,
                latest: None,
                input_at: None,
                last_operation: 0,
                queue: VecDeque::with_capacity(OPERATIONS),
                pending: BTreeMap::new(),
                recent: VecDeque::with_capacity(RESULTS),
                outgoing: VecDeque::with_capacity(RESULTS),
                slow: false,
            },
        );
        Ok(self.welcome(engine, id))
    }
    pub fn resume(
        &mut self,
        engine: &mut Engine,
        id: u64,
        secret: &Secret,
        now_ms: u64,
    ) -> NetResult<Welcome> {
        self.check(engine)?;
        self.expire(engine, now_ms);
        let old = self.sessions.get(&id).ok_or(Rejection::UnknownSession)?;
        if !old.secret.matches(secret) {
            return Err(Rejection::Unavailable);
        }
        if !engine.simulation.actors.matches(old.actor, old.reference) {
            return Err(Rejection::Unavailable);
        }
        let connection = old
            .connection
            .checked_add(1)
            .ok_or(Rejection::Unavailable)?;
        let name = engine
            .simulation
            .actors
            .id(engine.simulation.definitions(), old.actor)
            .to_owned();
        // Fence first: an unexpected bind failure must never leave the previous
        // transport marked connected with a released control handle.
        let old = self.sessions.get_mut(&id).unwrap();
        old.connection = connection;
        old.connected = false;
        old.disconnected_at = Some(now_ms);
        old.latest = None;
        old.input_at = None;
        let old_handle = old.handle.take();
        if let Some(handle) = old_handle {
            let _ = engine.execute(Command::ReleaseControl { control: handle }.into());
        }
        let handle = engine
            .execute(Command::BindControl { actor: name }.into())
            .map_err(|_| Rejection::Unavailable)?["control"]
            .as_str()
            .expect("control result")
            .to_owned();
        let s = self.sessions.get_mut(&id).unwrap();
        s.handle = Some(handle);
        s.connection = connection;
        s.connected = true;
        s.disconnected_at = None;
        s.input_sequence = 0;
        s.input_ack = 0;
        s.latest = None;
        s.input_at = None;
        s.outgoing.clear();
        s.slow = false;
        Ok(self.welcome(engine, id))
    }
    pub fn movement(
        &mut self,
        engine: &Engine,
        id: u64,
        connection: u64,
        movement: Movement,
        received_ms: u64,
    ) -> NetResult<bool> {
        self.check(engine)?;
        let s = self.session(id, connection)?;
        if !engine.simulation.actors.matches(s.actor, s.reference)
            || engine.simulation.vitality.dead(s.actor as usize)
            || movement.life != engine.simulation.vitality.state.lives[s.actor as usize]
        {
            return Err(Rejection::StaleLife);
        }
        Engine::validate_input(movement.input).map_err(|_| Rejection::InvalidOperation)?;
        if movement.sequence <= s.input_sequence {
            return Ok(false);
        }
        if movement.sequence > s.input_sequence.saturating_add(1024) {
            return Err(Rejection::Sequence);
        }
        let s = self.sessions.get_mut(&id).unwrap();
        s.input_sequence = movement.sequence;
        s.latest = Some((movement, received_ms));
        Ok(true)
    }
    pub fn operation(
        &mut self,
        engine: &Engine,
        id: u64,
        connection: u64,
        operation: Operation,
    ) -> NetResult<OperationResult> {
        self.check(engine)?;
        let s = self.session(id, connection)?;
        if operation.id <= s.last_operation {
            let result = s.result(operation.id).ok_or(Rejection::ExpiredResult)?;
            self.sessions.get_mut(&id).unwrap().record(result.clone());
            return Ok(result);
        }
        if s.last_operation.checked_add(1) != Some(operation.id) {
            return Err(Rejection::Sequence);
        }
        if s.pending.len() >= OPERATIONS {
            return Err(Rejection::Capacity);
        }
        if !engine.simulation.actors.matches(s.actor, s.reference)
            || operation.life != engine.simulation.vitality.state.lives[s.actor as usize]
        {
            return Err(Rejection::StaleLife);
        }
        let result = OperationResult {
            id: operation.id,
            tick: engine.simulation.tick,
            status: Status::Received,
        };
        let s = self.sessions.get_mut(&id).unwrap();
        s.last_operation = operation.id;
        s.queue.push_back(operation);
        s.record(result.clone());
        Ok(result)
    }
    fn command(engine: &Engine, s: &Session, operation: Operation) -> NetResult<Command> {
        if !engine.simulation.actors.matches(s.actor, s.reference) {
            return Err(Rejection::Unavailable);
        }
        if operation.life != engine.simulation.vitality.state.lives[s.actor as usize] {
            return Err(Rejection::StaleLife);
        }
        let actor = engine
            .simulation
            .actors
            .id(engine.simulation.definitions(), s.actor)
            .to_owned();
        Ok(match operation.intent {
            Intent::Activate { slot, target } => {
                let target = target
                    .map(|target| {
                        let a = engine
                            .simulation
                            .actors
                            .slot_for_instance(target.actor)
                            .ok_or(Rejection::Target)?;
                        if engine.simulation.vitality.state.lives[a as usize] != target.life {
                            return Err(Rejection::Target);
                        }
                        Ok(engine
                            .simulation
                            .actors
                            .id(engine.simulation.definitions(), a)
                            .to_owned())
                    })
                    .transpose()?;
                Command::ActivateAbility {
                    actor,
                    slot,
                    target,
                }
            }
            Intent::Aim { slot, aim } => Command::AimAbility { actor, slot, aim },
            Intent::Cancel { execution } => Command::CancelAbility {
                actor,
                execution_id: execution,
            },
        })
    }
    pub fn before_tick(&mut self, engine: &mut Engine, now_ms: u64) -> NetResult<()> {
        self.check(engine)?;
        self.expire(engine, now_ms);
        let ids: Vec<_> = self.sessions.keys().copied().collect();
        for id in ids {
            if self.sessions[&id].slow {
                let connection = self.sessions[&id].connection;
                self.disconnect(engine, id, connection, now_ms);
                continue;
            }
            let s = self.sessions.get_mut(&id).unwrap();
            if !s.connected {
                continue;
            }
            let live = engine.simulation.actors.matches(s.actor, s.reference)
                && !engine.simulation.vitality.dead(s.actor as usize);
            let latest = s.latest.take().filter(|(m, received)| {
                live && m.life == engine.simulation.vitality.state.lives[s.actor as usize]
                    && now_ms.saturating_sub(*received) <= INPUT_TTL_MS
            });
            let expired = s
                .input_at
                .is_some_and(|at| now_ms.saturating_sub(at) > INPUT_TTL_MS)
                || !live;
            if let Some((movement, received)) = latest {
                if let Some(handle) = &s.handle {
                    if engine
                        .execute(
                            Command::SetControlInput {
                                control: handle.clone(),
                                input: movement.input,
                            }
                            .into(),
                        )
                        .is_ok()
                    {
                        s.input_ack = movement.sequence;
                        s.input_at = Some(received);
                    } else {
                        s.slow = true;
                    }
                }
            } else if expired && let Some(handle) = &s.handle {
                let _ = engine.execute(
                    Command::SetControlInput {
                        control: handle.clone(),
                        input: Input::default(),
                    }
                    .into(),
                );
                s.input_at = None;
            }
            while let Some(operation) = s.queue.pop_front() {
                let result = Self::command(engine, s, operation).and_then(|command| {
                    engine
                        .execute(command.into())
                        .map_err(|_| Rejection::InvalidOperation)
                });
                match result {
                    Ok(value) => {
                        let request = value["request_id"].as_u64().expect("queued request");
                        if engine.simulation.abilities.watch_request(request).is_err() {
                            return Err(Rejection::Capacity);
                        }
                        self.requests.insert(request, (id, operation.id));
                        s.record(OperationResult {
                            id: operation.id,
                            tick: engine.simulation.tick,
                            status: Status::Queued,
                        });
                    }
                    Err(reason) => s.record(OperationResult {
                        id: operation.id,
                        tick: engine.simulation.tick,
                        status: Status::Rejected { reason },
                    }),
                }
            }
        }
        Ok(())
    }
    pub fn after_tick(&mut self, engine: &Engine) -> NetResult<()> {
        self.check(engine)?;
        if engine.simulation.abilities.dropped_request_results() != 0 {
            return Err(Rejection::Capacity);
        }
        if self.last_result_tick == Some(engine.simulation.tick) {
            return Ok(());
        }
        self.last_result_tick = Some(engine.simulation.tick);
        for &RequestResult {
            tick,
            actor,
            request,
            execution,
            outcome,
        } in engine.simulation.abilities.request_results()
        {
            if let Some(&(id, operation)) = self.requests.get(&request) {
                let status = if outcome == Outcome::Started {
                    Status::Started { execution }
                } else {
                    Status::Resolved { outcome, execution }
                };
                let terminal = status.terminal();
                if let Some(s) = self.sessions.get_mut(&id)
                    && s.reference == actor
                {
                    s.record(OperationResult {
                        id: operation,
                        tick,
                        status,
                    });
                }
                if terminal {
                    self.requests.remove(&request);
                }
            }
        }
        Ok(())
    }
    pub fn connections(&self) -> impl Iterator<Item = (u64, u64)> + '_ {
        self.sessions
            .iter()
            .filter(|(_, session)| session.connected && !session.slow)
            .map(|(&id, s)| (id, s.connection))
    }
    pub fn is_connected(&self, id: u64, connection: u64) -> bool {
        self.session(id, connection).is_ok_and(|s| !s.slow)
    }
    pub fn take_results(&mut self, id: u64, connection: u64) -> NetResult<Vec<OperationResult>> {
        self.session(id, connection)?;
        Ok(self
            .sessions
            .get_mut(&id)
            .unwrap()
            .outgoing
            .drain(..)
            .collect())
    }
    pub fn owner_view(&self, engine: &Engine, id: u64, connection: u64) -> NetResult<OwnerView> {
        self.check(engine)?;
        let s = self.session(id, connection)?;
        if !engine.simulation.actors.matches(s.actor, s.reference) {
            return Err(Rejection::Unavailable);
        }
        let resources = (0..engine
            .simulation
            .definitions()
            .ability_catalog
            .resources
            .len())
            .filter_map(|r| {
                engine
                    .simulation
                    .abilities
                    .resource_level(s.actor as usize, r)
                    .map(|(value, capacity)| (r as u8, value, capacity))
            })
            .collect();
        Ok(OwnerView {
            actor: Target {
                actor: s.reference.instance,
                life: engine.simulation.vitality.state.lives[s.actor as usize],
            },
            tick: engine.simulation.tick,
            input_ack: s.input_ack,
            position: engine
                .simulation
                .position(
                    engine
                        .simulation
                        .actors
                        .id(engine.simulation.definitions(), s.actor),
                )
                .ok_or(Rejection::Unavailable)?,
            resources,
            ready_at: engine.simulation.abilities.state.ready_at[s.actor as usize],
            alive: !engine.simulation.vitality.dead(s.actor as usize),
            cast_execution: engine.simulation.abilities.state.casts[s.actor as usize].map(|c| c.id),
        })
    }
    pub fn disconnect(&mut self, engine: &mut Engine, id: u64, connection: u64, now_ms: u64) {
        if self.session(id, connection).is_err() {
            return;
        }
        let s = self.sessions.get_mut(&id).unwrap();
        s.connected = false;
        s.disconnected_at = Some(now_ms);
        s.latest = None;
        s.input_at = None;
        if let Some(handle) = s.handle.take() {
            let _ = engine.execute(Command::ReleaseControl { control: handle }.into());
        }
        s.outgoing.clear();
        while let Some(op) = s.queue.pop_front() {
            s.record(OperationResult {
                id: op.id,
                tick: engine.simulation.tick,
                status: Status::Rejected {
                    reason: Rejection::Unavailable,
                },
            });
        }
    }
    fn expire(&mut self, engine: &mut Engine, now_ms: u64) {
        let expired: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, s)| {
                s.disconnected_at
                    .is_some_and(|at| now_ms.saturating_sub(at) > RESUME_MS)
            })
            .map(|(&id, _)| id)
            .collect();
        for id in expired {
            let s = self.sessions.remove(&id).unwrap();
            self.owners.remove(&s.reference.instance);
            self.requests.retain(|&request, (session, _)| {
                if *session == id {
                    engine.simulation.abilities.unwatch_request(request);
                    false
                } else {
                    true
                }
            });
        }
    }
    pub fn close(&mut self, engine: &mut Engine) {
        if self.closed {
            return;
        }
        if self.scope == engine.runtime_scope() {
            let ids: Vec<_> = self
                .sessions
                .iter()
                .filter(|(_, s)| s.connected)
                .map(|(&id, s)| (id, s.connection))
                .collect();
            for (id, connection) in ids {
                self.disconnect(engine, id, connection, 0);
            }
            engine.simulation.abilities.enable_request_results(false);
        }
        self.sessions.clear();
        self.owners.clear();
        self.requests.clear();
        self.closed = true;
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"world":self.world,"capacity":self.max_sessions,"sessions":self.sessions.iter().map(|(&id,s)|serde_json::json!({
            "session":id,"connection":s.connection,"actor":s.reference.instance,"connected":s.connected,"pending":s.pending.len(),"queued":s.queue.len(),"recent":s.recent.len(),"outgoing":s.outgoing.len(),"input_ack":s.input_ack,"last_operation":s.last_operation,"slow":s.slow
        })).collect::<Vec<_>>(),"requests":self.requests.len()})
    }
}
