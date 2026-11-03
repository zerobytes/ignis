//! Bounded QUIC framing; application payloads are versioned independently of QUIC.
use super::{
    protocol::*,
    replica::{Frame, MAX_FRAME_BYTES},
};
use anyhow::{Result, bail, ensure};
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};
pub const CONTROL_BYTES: usize = 64 * 1024;
pub const CLIENT_BYTES: usize = 4096;
pub const CHUNK: usize = 1000;
const HEADER: usize = 28;
const CACHE: usize = 12;
pub const DEADLINE: Duration = Duration::from_secs(3);
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Hello {
    Join {
        version: u16,
    },
    Resume {
        version: u16,
        session: u64,
        secret: Secret,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Control {
    Operation { operation: Operation },
    Ping,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Message {
    Welcome {
        welcome: Welcome,
    },
    Update {
        results: Vec<OperationResult>,
        owner: Option<OwnerView>,
    },
    Rejected {
        operation: u64,
        reason: Rejection,
    },
}
pub async fn read_json<T: DeserializeOwned>(recv: &mut RecvStream, limit: usize) -> Result<T> {
    let mut head = [0; 4];
    recv.read_exact(&mut head).await?;
    let length = u32::from_le_bytes(head) as usize;
    ensure!(
        length > 0 && length <= limit,
        "Control frame exceeds budget"
    );
    let mut data = vec![0; length];
    recv.read_exact(&mut data).await?;
    Ok(serde_json::from_slice(&data)?)
}
pub async fn write_json(send: &mut SendStream, value: &impl Serialize) -> Result<()> {
    let data = serde_json::to_vec(value)?;
    ensure!(data.len() <= CONTROL_BYTES, "Control frame exceeds budget");
    tokio::time::timeout(DEADLINE, async {
        send.write_all(&(data.len() as u32).to_le_bytes()).await?;
        send.write_all(&data).await
    })
    .await??;
    Ok(())
}
pub fn movement(value: &Movement) -> Result<bytes::Bytes> {
    let mut data = vec![1];
    serde_json::to_writer(&mut data, value)?;
    ensure!(data.len() <= 512, "Movement exceeds budget");
    Ok(data.into())
}
pub fn ack(world: u64, tick: Option<u64>) -> bytes::Bytes {
    let mut out = Vec::with_capacity(18);
    out.push(2);
    out.extend_from_slice(&world.to_le_bytes());
    out.push(u8::from(tick.is_some()));
    out.extend_from_slice(&tick.unwrap_or(0).to_le_bytes());
    out.into()
}
pub enum Datagram {
    Movement(Movement),
    Ack { world: u64, tick: Option<u64> },
}
pub fn datagram(data: &[u8]) -> Result<Datagram> {
    match data.first() {
        Some(1) if data.len() <= 512 => Ok(Datagram::Movement(serde_json::from_slice(&data[1..])?)),
        Some(2) if data.len() == 18 => {
            ensure!(data[9] <= 1, "Invalid ACK flag");
            Ok(Datagram::Ack {
                world: u64::from_le_bytes(data[1..9].try_into()?),
                tick: (data[9] == 1).then(|| u64::from_le_bytes(data[10..18].try_into().unwrap())),
            })
        }
        _ => bail!("Invalid client datagram"),
    }
}
#[derive(Clone)]
pub struct Packet {
    pub world: u64,
    pub tick: u64,
    pub full: bool,
    pub data: bytes::Bytes,
    pub fragments: Vec<bytes::Bytes>,
}
impl Packet {
    fn new(frame: &Frame, base: Option<&Frame>) -> Result<Self> {
        let data: bytes::Bytes = frame.encode(base)?.into();
        let count = data.len().div_ceil(CHUNK);
        ensure!(
            count > 0 && count <= MAX_FRAME_BYTES.div_ceil(CHUNK),
            "Snapshot fragment budget"
        );
        let fragments = if base.is_some() {
            data.chunks(CHUNK)
                .enumerate()
                .map(|(index, part)| {
                    let mut v = Vec::with_capacity(HEADER + part.len());
                    v.extend_from_slice(b"IGD2");
                    v.extend_from_slice(&frame.world.to_le_bytes());
                    v.extend_from_slice(&frame.tick.to_le_bytes());
                    v.extend_from_slice(&(data.len() as u32).to_le_bytes());
                    v.extend_from_slice(&(index as u16).to_le_bytes());
                    v.extend_from_slice(&(count as u16).to_le_bytes());
                    v.extend_from_slice(part);
                    bytes::Bytes::from(v)
                })
                .collect()
        } else {
            Vec::new()
        };
        Ok(Self {
            world: frame.world,
            tick: frame.tick,
            full: base.is_none(),
            data,
            fragments,
        })
    }
}
#[derive(Default)]
pub struct Hub {
    frames: VecDeque<Arc<Frame>>,
    packets: BTreeMap<(u64, Option<u64>), Arc<Packet>>,
    pub encoded: u64,
}
impl Hub {
    pub fn publish(&mut self, frame: Arc<Frame>) {
        if self.frames.back().is_some_and(|f| f.world != frame.world) {
            self.frames.clear();
            self.packets.clear();
        }
        if self.frames.back().is_some_and(|f| f.tick >= frame.tick) {
            return;
        }
        self.frames.push_back(frame);
        if self.frames.len() > CACHE {
            self.frames.pop_front();
        }
        let first = self.frames.front().unwrap().tick;
        self.packets
            .retain(|&(tick, base), _| tick >= first && base.is_none_or(|b| b >= first));
    }
    pub fn packet(&mut self, frame: Arc<Frame>, ack: Option<u64>) -> Result<Option<Arc<Packet>>> {
        self.publish(frame.clone());
        if self.frames.front().is_some_and(|f| frame.tick < f.tick) {
            return Ok(None);
        }
        if ack == Some(frame.tick) {
            return Ok(None);
        }
        let baseline = ack.and_then(|tick| {
            self.frames
                .iter()
                .find(|f| f.world == frame.world && f.tick == tick && tick < frame.tick)
                .cloned()
        });
        let key = (frame.tick, baseline.as_ref().map(|b| b.tick));
        if let Some(p) = self.packets.get(&key) {
            return Ok(Some(p.clone()));
        }
        let packet = Packet::new(&frame, baseline.as_deref())?;
        let packet = Arc::new(if !packet.full && packet.data.len() > 32 * 1024 {
            Packet::new(&frame, None)?
        } else {
            packet
        });
        self.encoded = self.encoded.saturating_add(1);
        self.packets.insert(key, packet.clone());
        Ok(Some(packet))
    }
}
struct Assembly {
    world: u64,
    tick: u64,
    length: usize,
    parts: Vec<Option<bytes::Bytes>>,
    received: usize,
    created: Instant,
}
#[derive(Default)]
pub struct Assembler {
    pending: VecDeque<Assembly>,
}
impl Assembler {
    pub fn push(&mut self, expected_world: u64, data: bytes::Bytes) -> Result<Option<Vec<u8>>> {
        ensure!(
            data.len() >= HEADER && &data[..4] == b"IGD2",
            "Invalid snapshot fragment"
        );
        let world = u64::from_le_bytes(data[4..12].try_into()?);
        let tick = u64::from_le_bytes(data[12..20].try_into()?);
        let length = u32::from_le_bytes(data[20..24].try_into()?) as usize;
        let index = u16::from_le_bytes(data[24..26].try_into()?) as usize;
        let count = u16::from_le_bytes(data[26..28].try_into()?) as usize;
        ensure!(
            world == expected_world
                && length > 0
                && length <= MAX_FRAME_BYTES
                && count == length.div_ceil(CHUNK)
                && index < count,
            "Snapshot fragment domain/budget"
        );
        let expected = if index + 1 == count {
            length - index * CHUNK
        } else {
            CHUNK
        };
        ensure!(data.len() == HEADER + expected, "Snapshot fragment length");
        self.pending
            .retain(|a| a.created.elapsed() < Duration::from_millis(500));
        if !self
            .pending
            .iter()
            .any(|a| a.world == world && a.tick == tick)
        {
            if self.pending.len() == 2 {
                self.pending.pop_front();
            }
            self.pending.push_back(Assembly {
                world,
                tick,
                length,
                parts: vec![None; count],
                received: 0,
                created: Instant::now(),
            });
        }
        let a = self
            .pending
            .iter_mut()
            .find(|a| a.world == world && a.tick == tick)
            .unwrap();
        ensure!(
            a.length == length && a.parts.len() == count,
            "Conflicting fragment headers"
        );
        let part = data.slice(HEADER..);
        if let Some(old) = &a.parts[index] {
            ensure!(old == &part, "Conflicting duplicate fragment");
            return Ok(None);
        }
        a.parts[index] = Some(part);
        a.received += 1;
        if a.received < count {
            return Ok(None);
        }
        let mut out = Vec::with_capacity(length);
        for part in &a.parts {
            out.extend_from_slice(part.as_ref().unwrap());
        }
        self.pending.retain(|a| a.tick != tick);
        Ok(Some(out))
    }
}
pub struct Rate {
    start: Instant,
    count: usize,
    limit: usize,
}
impl Rate {
    pub fn new(limit: usize) -> Self {
        Self {
            start: Instant::now(),
            count: 0,
            limit,
        }
    }
    pub fn admit(&mut self) -> bool {
        if self.start.elapsed() >= Duration::from_secs(1) {
            self.start = Instant::now();
            self.count = 0;
        }
        self.count += 1;
        self.count <= self.limit
    }
}

/// Typed content/full streams, with progress and total bounds. I/O thread only.
pub async fn write_blob(connection: &quinn::Connection, kind: u8, bytes: &[u8]) -> Result<()> {
    let maximum = match kind {
        0 => super::content::MAX_BYTES,
        1 => MAX_FRAME_BYTES,
        _ => bail!("Unknown public stream kind"),
    };
    ensure!(bytes.len() <= maximum, "Public stream budget");
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        let mut stream = tokio::time::timeout(DEADLINE, connection.open_uni()).await??;
        stream.set_priority(-1)?;
        let mut head = [0; 5];
        head[0] = kind;
        head[1..].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
        tokio::time::timeout(DEADLINE, stream.write_all(&head)).await??;
        for chunk in bytes.chunks(32 * 1024) {
            tokio::time::timeout(DEADLINE, stream.write_all(chunk)).await??;
        }
        stream.finish()?;
        let stopped = tokio::time::timeout(DEADLINE, stream.stopped()).await??;
        ensure!(stopped.is_none(), "Peer refused public stream");
        Ok::<_, anyhow::Error>(())
    })
    .await??;
    Ok(())
}
pub async fn read_blob(stream: &mut quinn::RecvStream) -> Result<(u8, Vec<u8>)> {
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        let mut head = [0; 5];
        tokio::time::timeout(DEADLINE, stream.read_exact(&mut head)).await??;
        let maximum = match head[0] {
            0 => super::content::MAX_BYTES,
            1 => MAX_FRAME_BYTES,
            _ => bail!("Unknown public stream kind"),
        };
        let length = u32::from_le_bytes(head[1..].try_into()?) as usize;
        ensure!(length > 0 && length <= maximum, "Public stream budget");
        let mut bytes = vec![0; length];
        for part in bytes.chunks_mut(32 * 1024) {
            tokio::time::timeout(DEADLINE, stream.read_exact(part)).await??;
        }
        let mut end = [0; 1];
        ensure!(
            tokio::time::timeout(DEADLINE, stream.read(&mut end))
                .await??
                .is_none(),
            "Trailing public stream bytes"
        );
        Ok::<_, anyhow::Error>((head[0], bytes))
    })
    .await?
}
