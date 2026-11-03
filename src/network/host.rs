//! Synchronous world side of the optional backend. No socket, serde or await here.
use super::{
    protocol::*,
    replica::Frame,
    sessions::Authority,
    transport::{Backend, Link},
    wire::Hello,
};
use crate::{
    engine::Engine,
    model::{Result, invalid},
};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
};
pub struct Host {
    authority: Authority,
    backend: Backend,
    links: BTreeMap<u64, (u64, Arc<Link>)>,
    previous: Option<Arc<Frame>>,
    closed: bool,
    operations: Vec<Operation>,
}
impl Host {
    pub fn launch(engine: &mut Engine, address: SocketAddr, capacity: usize) -> Result<Self> {
        if !engine.scene.wind.is_empty() {
            return Err(invalid(
                "Aerodynamic bodies and wind are not yet replicated",
            ));
        }
        let mut authority = Authority::new(engine, capacity)?;
        let content = Arc::new(super::content::Content::capture(engine, authority.world()));
        let backend = match Backend::launch(address, capacity, content) {
            Ok(b) => b,
            Err(e) => {
                authority.close(engine);
                return Err(invalid(e.to_string()));
            }
        };
        Ok(Self {
            authority,
            backend,
            links: BTreeMap::new(),
            previous: None,
            closed: false,
            operations: Vec::with_capacity(OPERATIONS),
        })
    }
    pub fn poll(&mut self, engine: &mut Engine) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        if !self.authority.valid(engine) {
            self.close(engine);
            return Err(invalid("Multiplayer world changed; host stopped"));
        }
        let now = self.backend.now_ms();
        for _ in 0..64 {
            let Ok(open) = self.backend.incoming.try_recv() else {
                break;
            };
            if open.reply.is_closed() {
                continue;
            }
            let welcome = match open.hello {
                Hello::Join { .. } => self
                    .authority
                    .join(engine, now)
                    .map_err(|_| Rejection::Unavailable),
                Hello::Resume {
                    session, secret, ..
                } => self.authority.resume(engine, session, &secret, now),
            };
            let response = welcome.map(|welcome| {
                let link = Arc::new(Link::new());
                if let Some((_, old)) = self
                    .links
                    .insert(welcome.session, (welcome.connection, link.clone()))
                {
                    old.close();
                }
                (welcome, link)
            });
            if let Err(Ok((welcome, link))) = open.reply.send(response) {
                link.close();
                self.authority
                    .disconnect(engine, welcome.session, welcome.connection, now);
            }
        }
        for (&id, (generation, link)) in &self.links {
            if !self.authority.is_connected(id, *generation) {
                link.close();
                continue;
            }
            if link.closed.load(Ordering::Relaxed) {
                self.authority.disconnect(engine, id, *generation, now);
                continue;
            }
            let movement = {
                let mut inbox = link.inbox.lock().unwrap();
                self.operations.extend(inbox.operations.drain(..));
                inbox.movement.take()
            };
            if let Some((movement, received)) = movement {
                let _ = self
                    .authority
                    .movement(engine, id, *generation, movement, received);
            }
            for operation in self.operations.drain(..) {
                if let Err(reason) = self.authority.operation(engine, id, *generation, operation) {
                    let mut out = link.outbox.lock().unwrap();
                    if out.results.len() == RESULTS {
                        link.close();
                        break;
                    }
                    out.results.push_back(OperationResult {
                        id: operation.id,
                        tick: engine.simulation.tick,
                        status: Status::Rejected { reason },
                    });
                    link.notify.notify_one();
                }
            }
        }
        self.links
            .retain(|_, (_, link)| !link.closed.load(Ordering::Relaxed));
        Ok(())
    }
    pub fn before_tick(&mut self, engine: &mut Engine) -> Result<()> {
        self.poll(engine)?;
        if self.closed {
            return Ok(());
        }
        self.authority
            .before_tick(engine, self.backend.now_ms())
            .map_err(|r| invalid(format!("Multiplayer admission failed: {r:?}")))
    }
    pub fn after_tick(&mut self, engine: &mut Engine) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        if let Err(reason) = self.authority.after_tick(engine) {
            self.close(engine);
            return Err(invalid(format!(
                "Multiplayer outcome overflow/world invalid: {reason:?}"
            )));
        }
        let snapshot = self
            .previous
            .as_ref()
            .is_none_or(|f| engine.simulation.tick.saturating_sub(f.tick) >= 6);
        for (&id, (generation, link)) in &self.links {
            let Ok(results) = self.authority.take_results(id, *generation) else {
                link.close();
                continue;
            };
            let mut out = link.outbox.lock().unwrap();
            if out.results.len() + results.len() > RESULTS {
                link.close();
                continue;
            }
            out.results.extend(results);
            if snapshot {
                out.owner = self.authority.owner_view(engine, id, *generation).ok();
            }
            drop(out);
            link.notify.notify_one();
        }
        if snapshot {
            let frame = Arc::new(Frame::capture(
                engine,
                self.authority.world(),
                self.previous.as_deref(),
            )?);
            self.backend.frames.send_replace(Some(frame.clone()));
            self.previous = Some(frame);
        }
        Ok(())
    }
    pub fn close(&mut self, engine: &mut Engine) {
        if self.closed {
            return;
        }
        self.authority.close(engine);
        for (_, link) in self.links.values() {
            link.close();
        }
        self.links.clear();
        self.backend.stop();
        self.closed = true;
    }
    pub fn inspect(&self) -> serde_json::Value {
        let c = &self.backend.counters;
        serde_json::json!({"transport":*self.backend.info.lock().unwrap(),"closed":self.closed,"domain":self.authority.inspect(),"accepted":c.accepted.load(Ordering::Relaxed),"closed_connections":c.closed.load(Ordering::Relaxed),"snapshot_payload_bytes":c.sent_bytes.load(Ordering::Relaxed),"input_payload_bytes":c.received_bytes.load(Ordering::Relaxed),"full_frames":c.full_frames.load(Ordering::Relaxed),"delta_frames":c.delta_frames.load(Ordering::Relaxed),"reordered_acks":c.reordered_acks.load(Ordering::Relaxed),"encoded_packets":self.backend.hub.lock().unwrap().encoded,"snapshot_hz":10,"control_per_second":128,"datagrams_per_second":128})
    }
}
