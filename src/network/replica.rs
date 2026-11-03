//! Versioned presentation projection. Logical IDs only; no simulation structs on wire.
use crate::{
    engine::Engine,
    model::{Result, invalid},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
const MAX_OBJECTS: usize = crate::model::MAX_ENTITIES;
pub const MAX_PROJECTILE_VISUALS: usize = 4096;
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;
const MAGIC: &[u8; 4] = b"IGNR";
const LIVE: u8 = 1;
const NEW: u8 = 2;
const X: u8 = 4;
const Y: u8 = 8;
const Z: u8 = 16;
const LIFE: u8 = 32;
const ALL: u8 = LIVE | NEW | X | Y | Z | LIFE;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Health {
    pub units: i64,
    pub maximum: i64,
    pub scale: u32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Kind {
    Visual,
    Fixed,
    Dynamic,
    Player,
    Npc,
}
impl Kind {
    pub fn actor(self) -> bool {
        matches!(self, Self::Player | Self::Npc)
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Projectile {
    pub execution: u64,
    pub millimeters: [i32; 3],
    pub half_extent: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Appearance {
    pub label: String,
    pub size: [f32; 3],
    pub color: [f32; 3],
    pub player: bool,
    pub kind: Kind,
    pub geometry: crate::geometry::Geometry,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Actor {
    pub id: u64,
    pub millimeters: [i32; 3],
    pub life: u64,
    pub health: Option<Health>,
    pub appearance: Arc<Appearance>,
}
impl Actor {
    pub fn position(&self) -> [f32; 3] {
        self.millimeters.map(|x| x as f32 / 1000.)
    }
}
#[derive(Debug, Clone)]
pub struct Frame {
    pub avatars: Vec<crate::npc_presentation::Pose>,
    pub world: u64,
    pub tick: u64,
    pub actors: Vec<Actor>,
    pub projectiles: Vec<Projectile>,
    pub omitted_projectiles: u32,
}
impl Frame {
    pub fn capture(engine: &Engine, world: u64, previous: Option<&Self>) -> Result<Self> {
        let previous = previous.filter(|frame| frame.world == world);
        let definitions = engine.simulation.definitions();
        let mut actors = Vec::with_capacity(definitions.entities.len());
        for (entity, definition) in definitions.entities.iter().enumerate() {
            let Some(reference) = engine.simulation.actors.entity_reference(entity) else {
                continue;
            };
            if definition.visible == Some(false) {
                continue;
            }
            let Some(position) = engine.simulation.position(&definition.id) else {
                continue;
            };
            let kind = match definition.body {
                None => Kind::Visual,
                Some(crate::model::BodyKind::Fixed) => Kind::Fixed,
                Some(crate::model::BodyKind::Dynamic) => Kind::Dynamic,
                Some(crate::model::BodyKind::Player) => Kind::Player,
                Some(crate::model::BodyKind::Npc) => Kind::Npc,
            };
            let appearance = previous
                .and_then(|f| {
                    f.actors
                        .binary_search_by_key(&reference.instance, |a| a.id)
                        .ok()
                        .map(|i| f.actors[i].appearance.clone())
                })
                .filter(|a| {
                    a.label == definition.id
                        && a.size == definition.transform.size
                        && a.color == definition.color
                        && a.player == (kind == Kind::Player)
                        && a.kind == kind
                        && a.geometry == definition.geometry
                })
                .unwrap_or_else(|| {
                    Arc::new(Appearance {
                        label: definition.id.clone(),
                        size: definition.transform.size,
                        color: definition.color,
                        player: kind == Kind::Player,
                        kind,
                        geometry: definition.geometry,
                    })
                });
            let slot = engine.simulation.actors.slot(entity) as usize;
            let (life, health) = if kind.actor() {
                (
                    engine.simulation.vitality.state.lives[slot],
                    engine.simulation.vitality.state.actors[slot].map(|h| Health {
                        units: h.units,
                        maximum: h.maximum,
                        scale: h.scale,
                    }),
                )
            } else {
                (0, None)
            };
            actors.push(Actor {
                id: reference.instance,
                millimeters: quantize(position)?,
                life,
                health,
                appearance,
            });
        }
        actors.sort_unstable_by_key(|a| a.id);
        let active = &engine.simulation.abilities.state.projectiles.active;
        let mut projectiles = Vec::with_capacity(active.len().min(MAX_PROJECTILE_VISUALS));
        // Four per source first: one crowded source cannot hide all other emitters.
        let mut counts = [0u8; crate::targeting::MAX_ACTORS];
        for shot in active.iter() {
            let count = &mut counts[shot.actor as usize];
            if *count < 4 {
                *count += 1;
                projectiles.push(Projectile {
                    execution: shot.cast.id,
                    millimeters: quantize(shot.position)?,
                    half_extent: shot.half_extent,
                });
            }
        }
        if projectiles.len() < MAX_PROJECTILE_VISUALS {
            counts.fill(0);
            for shot in active.iter() {
                let count = &mut counts[shot.actor as usize];
                *count += 1;
                if *count > 4 {
                    projectiles.push(Projectile {
                        execution: shot.cast.id,
                        millimeters: quantize(shot.position)?,
                        half_extent: shot.half_extent,
                    });
                    if projectiles.len() == MAX_PROJECTILE_VISUALS {
                        break;
                    }
                }
            }
        }
        projectiles.sort_unstable_by_key(|p| p.execution);
        let omitted_projectiles = (active.len() - projectiles.len()) as u32;
        let mut avatars = engine.simulation.presentation_poses();
        avatars.sort_unstable_by_key(|p| p.actor);
        Ok(Self {
            avatars,
            world,
            tick: engine.simulation.tick,
            actors,
            projectiles,
            omitted_projectiles,
        })
    }
    pub fn encode(&self, baseline: Option<&Self>) -> Result<Vec<u8>> {
        if self.actors.len() > MAX_OBJECTS
            || self.actors.windows(2).any(|a| a[0].id >= a[1].id)
            || self.actors.iter().any(|a| a.id == 0)
        {
            return Err(invalid("Invalid replica actor order/domain"));
        }
        if baseline.is_some_and(|b| {
            b.actors.len() > MAX_OBJECTS
                || b.actors.windows(2).any(|a| a[0].id >= a[1].id)
                || b.actors.iter().any(|a| a.id == 0)
        }) {
            return Err(invalid("Invalid baseline actor domain"));
        }
        if baseline.is_some_and(|b| b.world != self.world || b.tick >= self.tick) {
            return Err(invalid("Invalid replication baseline"));
        }
        let mut rows: Vec<(u64, u8, Option<&Actor>, Option<&Actor>)> =
            Vec::with_capacity(self.actors.len() + baseline.map_or(0, |b| b.actors.len()));
        let old = baseline.map_or(&[][..], |b| b.actors.as_slice());
        let (mut i, mut j) = (0, 0);
        while i < self.actors.len() || j < old.len() {
            match (self.actors.get(i), old.get(j)) {
                (Some(current), Some(before)) if current.id == before.id => {
                    let mut flags = LIVE;
                    for (axis, bit) in [X, Y, Z].into_iter().enumerate() {
                        if current.millimeters[axis] != before.millimeters[axis] {
                            flags |= bit;
                        }
                    }
                    if current.life != before.life || current.health != before.health {
                        flags |= LIFE;
                    }
                    if current.appearance != before.appearance {
                        flags = ALL;
                    }
                    if flags != LIVE {
                        rows.push((current.id, flags, Some(current), Some(before)));
                    }
                    i += 1;
                    j += 1;
                }
                (Some(current), Some(before)) if current.id > before.id => {
                    rows.push((before.id, 0, None, Some(before)));
                    j += 1;
                }
                (Some(current), _) => {
                    rows.push((current.id, ALL, Some(current), None));
                    i += 1;
                }
                (None, Some(before)) => {
                    rows.push((before.id, 0, None, Some(before)));
                    j += 1;
                }
                (None, None) => break,
            }
        }
        let mut out = Vec::with_capacity(rows.len() * 12 + 40);
        out.extend(MAGIC);
        out.extend(super::protocol::VERSION.to_le_bytes());
        out.extend(self.world.to_le_bytes());
        out.extend(self.tick.to_le_bytes());
        out.extend(baseline.map_or(0, |b| b.tick).to_le_bytes());
        out.push(u8::from(baseline.is_none()));
        out.extend((rows.len() as u16).to_le_bytes());
        let mut last = 0;
        for (id, flags, current, before) in rows {
            var(&mut out, id - last);
            last = id;
            out.push(flags);
            let Some(current) = current else { continue };
            if flags & NEW != 0 {
                let app = &current.appearance;
                if app.label.is_empty()
                    || app.label.len() > 256
                    || app.player != (app.kind == Kind::Player)
                    || app
                        .size
                        .iter()
                        .any(|v| !v.is_finite() || *v < 0.05 || *v > 1000.)
                    || app
                        .color
                        .iter()
                        .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
                {
                    return Err(invalid("Invalid replica appearance"));
                }
                var(&mut out, app.label.len() as u64);
                out.extend(app.label.as_bytes());
                out.push(app.kind as u8);
                out.push(geometry_code(app.geometry));
                for v in app.size.into_iter().chain(app.color) {
                    out.extend(v.to_le_bytes());
                }
            }
            for (axis, bit) in [X, Y, Z].into_iter().enumerate() {
                if flags & bit != 0 {
                    let value = i64::from(current.millimeters[axis])
                        - if flags & NEW == 0 {
                            i64::from(before.unwrap().millimeters[axis])
                        } else {
                            0
                        };
                    signed(&mut out, value);
                }
            }
            if flags & LIFE != 0 {
                var(&mut out, current.life);
                if let Some(h) = current.health {
                    if !valid_health(h) {
                        return Err(invalid("Invalid replicated health"));
                    }
                    out.push(1);
                    var(&mut out, h.units as u64);
                    var(&mut out, h.maximum as u64);
                    var(&mut out, u64::from(h.scale));
                } else {
                    out.push(0);
                }
            }
        }
        if self
            .actors
            .iter()
            .filter(|a| a.appearance.kind.actor())
            .count()
            > crate::targeting::MAX_ACTORS
            || self.projectiles.len() > MAX_PROJECTILE_VISUALS
            || u64::from(self.omitted_projectiles) + self.projectiles.len() as u64
                > (crate::targeting::MAX_ACTORS * crate::projectiles::CAPACITY) as u64
            || self
                .projectiles
                .windows(2)
                .any(|p| p[0].execution >= p[1].execution)
        {
            return Err(invalid("Replica actor/projectile budget/order"));
        }
        var(&mut out, u64::from(self.omitted_projectiles));
        var(&mut out, self.projectiles.len() as u64);
        let mut prior = 0;
        for p in &self.projectiles {
            if p.execution == 0
                || !p.half_extent.is_finite()
                || !(0.0..=1.0).contains(&p.half_extent)
                || p.half_extent == 0.
            {
                return Err(invalid("Invalid projectile visual"));
            }
            var(&mut out, p.execution - prior);
            prior = p.execution;
            for value in p.millimeters {
                signed(&mut out, i64::from(value));
            }
            out.extend_from_slice(&p.half_extent.to_le_bytes());
        }
        if self.avatars.len() > crate::targeting::MAX_ACTORS
            || self.avatars.windows(2).any(|p| p[0].actor >= p[1].actor)
        {
            return Err(invalid("Avatar order/budget"));
        }
        var(&mut out, self.avatars.len() as u64);
        for p in &self.avatars {
            p.validate()?;
            if !self
                .actors
                .iter()
                .any(|a| a.id == p.actor && a.life == p.life && a.appearance.kind == Kind::Npc)
            {
                return Err(invalid("Avatar owner/life absent"));
            }
            var(&mut out, p.actor);
            var(&mut out, p.life);
            var(&mut out, u64::from(p.asset));
            out.push(p.clip.unwrap_or(255));
            var(&mut out, u64::from(p.clip_tick));
            out.push(p.previous_clip.unwrap_or(255));
            var(&mut out, u64::from(p.previous_tick));
            out.extend(p.blend.to_le_bytes());
            out.extend(p.yaw.to_le_bytes());
            out.extend(p.pitch.to_le_bytes());
            out.extend(p.scale.to_le_bytes());
            for x in p.offset {
                out.extend(x.to_le_bytes());
            }
            out.push(u8::from(p.hide_body));
            for x in p.orientation {
                out.extend(x.to_le_bytes());
            }
            out.push(p.strip_root.unwrap_or(255));
            out.push(p.joint_count);
            out.push(p.ik_limited);
            for j in &p.joints[..p.joint_count as usize] {
                out.push(j.bone);
                for x in j.translation.into_iter().chain(j.rotation) {
                    out.extend(x.to_le_bytes());
                }
            }
        }
        if out.len() > MAX_FRAME_BYTES {
            return Err(invalid("Replica frame exceeds capacity"));
        }
        Ok(out)
    }
}
fn quantize(position: [f32; 3]) -> Result<[i32; 3]> {
    let mut out = [0; 3];
    for (q, value) in out.iter_mut().zip(position) {
        let value = (f64::from(value) * 1000.).round();
        if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
            return Err(invalid("Object outside replicated coordinate domain"));
        }
        *q = value as i32;
    }
    Ok(out)
}
fn geometry_code(value: crate::geometry::Geometry) -> u8 {
    use crate::geometry::{Geometry, Rise};
    match value {
        Geometry::Box => 0,
        Geometry::Wedge { rise } => match rise {
            Rise::PositiveX => 1,
            Rise::NegativeX => 2,
            Rise::PositiveZ => 3,
            Rise::NegativeZ => 4,
        },
    }
}
fn geometry_from(value: u8) -> Result<crate::geometry::Geometry> {
    use crate::geometry::{Geometry, Rise};
    Ok(match value {
        0 => Geometry::Box,
        1 => Geometry::Wedge {
            rise: Rise::PositiveX,
        },
        2 => Geometry::Wedge {
            rise: Rise::NegativeX,
        },
        3 => Geometry::Wedge {
            rise: Rise::PositiveZ,
        },
        4 => Geometry::Wedge {
            rise: Rise::NegativeZ,
        },
        _ => return Err(invalid("Invalid replicated geometry")),
    })
}
fn valid_health(h: Health) -> bool {
    h.units >= 0
        && h.maximum > 0
        && h.units <= h.maximum
        && h.maximum <= crate::abilities::LIMIT
        && h.scale > 0
}
fn var(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
fn signed(out: &mut Vec<u8>, value: i64) {
    var(out, ((value << 1) ^ (value >> 63)) as u64);
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| invalid("Invalid replica length"))?;
        let value = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| invalid("Truncated replica frame"))?;
        self.at = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }
    fn float(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    fn var(&mut self) -> Result<u64> {
        let mut value = 0;
        for shift in (0..=63).step_by(7) {
            let b = self.byte()?;
            if shift == 63 && b > 1 {
                return Err(invalid("Replica integer overflow"));
            }
            value |= u64::from(b & 127) << shift;
            if b & 128 == 0 {
                if shift > 0 && b == 0 {
                    return Err(invalid("Noncanonical replica integer"));
                }
                return Ok(value);
            }
        }
        Err(invalid("Replica integer overflow"))
    }
    fn signed(&mut self) -> Result<i64> {
        let value = self.var()?;
        Ok(((value >> 1) as i64) ^ -((value & 1) as i64))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    Current,
    Stale,
    NeedBaseline,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Replica {
    avatars: Vec<crate::npc_presentation::Pose>,
    world: u64,
    tick: Option<u64>,
    actors: BTreeMap<u64, Actor>,
    projectiles: Vec<Projectile>,
    omitted_projectiles: u32,
}
impl Replica {
    pub fn new(world: u64) -> Self {
        Self {
            avatars: Vec::new(),
            world,
            tick: None,
            actors: BTreeMap::new(),
            projectiles: Vec::new(),
            omitted_projectiles: 0,
        }
    }
    pub fn avatars(&self) -> &[crate::npc_presentation::Pose] {
        &self.avatars
    }
    pub fn world(&self) -> u64 {
        self.world
    }
    pub fn tick(&self) -> Option<u64> {
        self.tick
    }
    pub fn actors(&self) -> &BTreeMap<u64, Actor> {
        &self.actors
    }
    pub fn actor_count(&self) -> usize {
        self.actors
            .values()
            .filter(|a| a.appearance.kind.actor())
            .count()
    }
    pub fn projectiles(&self) -> &[Projectile] {
        &self.projectiles
    }
    pub fn omitted_projectiles(&self) -> u32 {
        self.omitted_projectiles
    }
    /// Every field and resulting actor count is validated before any visible mutation.
    pub fn apply(&mut self, bytes: &[u8]) -> Result<Applied> {
        self.apply_with_avatars(bytes, |_| Ok(()))
    }
    pub fn apply_with_avatars(
        &mut self,
        bytes: &[u8],
        validate: impl FnOnce(&[crate::npc_presentation::Pose]) -> Result<()>,
    ) -> Result<Applied> {
        if bytes.len() > MAX_FRAME_BYTES {
            return Err(invalid("Replica frame exceeds capacity"));
        }
        let mut r = Reader { bytes, at: 0 };
        if r.bytes(4)? != MAGIC || r.u16()? != super::protocol::VERSION {
            return Err(invalid("Unsupported replica protocol"));
        }
        if r.u64()? != self.world {
            return Err(invalid("Replica world mismatch"));
        }
        let tick = r.u64()?;
        let baseline = r.u64()?;
        let full = match r.byte()? {
            0 => false,
            1 => true,
            _ => return Err(invalid("Invalid replica kind")),
        };
        let count = r.u16()? as usize;
        if count > MAX_OBJECTS * 2 || full && count > MAX_OBJECTS {
            return Err(invalid("Replica actor capacity"));
        }
        if self.tick.is_some_and(|t| tick <= t) {
            return Ok(Applied::Stale);
        }
        if !full && self.tick != Some(baseline) {
            return Ok(Applied::NeedBaseline);
        }
        if full && baseline != 0 {
            return Err(invalid("Full frame has a baseline"));
        }
        if !full && baseline >= tick {
            return Err(invalid("Invalid baseline order"));
        }
        let mut patches = Vec::with_capacity(count);
        let mut last = 0u64;
        let mut resulting = if full { 0 } else { self.actors.len() };
        let mut actor_count = if full { 0 } else { self.actor_count() };
        for _ in 0..count {
            let delta = r.var()?;
            if delta == 0 {
                return Err(invalid("Duplicate replica identity"));
            }
            let id = last
                .checked_add(delta)
                .ok_or_else(|| invalid("Replica identity overflow"))?;
            last = id;
            let flags = r.byte()?;
            if flags & !ALL != 0 || flags & LIVE == 0 && flags != 0 || full && flags != ALL {
                return Err(invalid("Invalid replica component mask"));
            }
            let before = if full { None } else { self.actors.get(&id) };
            if flags == 0 {
                if before.is_none() {
                    return Err(invalid("Removed replica is absent"));
                }
                resulting -= 1;
                actor_count -= usize::from(before.unwrap().appearance.kind.actor());
                patches.push((id, None));
                continue;
            }
            if before.is_none() && flags != ALL || flags & NEW != 0 && flags != ALL {
                return Err(invalid("Incomplete replica birth"));
            }
            let appearance = if flags & NEW != 0 {
                let len =
                    usize::try_from(r.var()?).map_err(|_| invalid("Invalid replica label"))?;
                if len == 0 || len > 256 {
                    return Err(invalid("Invalid replica label"));
                }
                let label = std::str::from_utf8(r.bytes(len)?)
                    .map_err(|_| invalid("Invalid replica UTF8"))?
                    .to_owned();
                let kind = match r.byte()? {
                    0 => Kind::Visual,
                    1 => Kind::Fixed,
                    2 => Kind::Dynamic,
                    3 => Kind::Player,
                    4 => Kind::Npc,
                    _ => return Err(invalid("Invalid object kind")),
                };
                let geometry = geometry_from(r.byte()?)?;
                if !matches!(kind, Kind::Visual | Kind::Fixed) && !geometry.is_box() {
                    return Err(invalid("Invalid object geometry/kind"));
                }
                let player = kind == Kind::Player;
                let mut size = [0.; 3];
                let mut color = [0.; 3];
                for v in &mut size {
                    *v = r.float()?;
                    if !v.is_finite() || *v < 0.05 || *v > 1000. {
                        return Err(invalid("Invalid replica size"));
                    }
                }
                for v in &mut color {
                    *v = r.float()?;
                    if !v.is_finite() || !(0. ..=1.).contains(v) {
                        return Err(invalid("Invalid replica color"));
                    }
                }
                Arc::new(Appearance {
                    label,
                    size,
                    color,
                    player,
                    kind,
                    geometry,
                })
            } else {
                before.unwrap().appearance.clone()
            };
            let mut millimeters = if flags & NEW != 0 {
                [0; 3]
            } else {
                before.unwrap().millimeters
            };
            for (axis, bit) in [X, Y, Z].into_iter().enumerate() {
                if flags & bit != 0 {
                    let value = r.signed()?;
                    let base = if flags & NEW == 0 {
                        i64::from(millimeters[axis])
                    } else {
                        0
                    };
                    millimeters[axis] = i32::try_from(
                        base.checked_add(value)
                            .ok_or_else(|| invalid("Replica position overflow"))?,
                    )
                    .map_err(|_| invalid("Replica coordinate domain"))?;
                }
            }
            let (life, health) = if flags & LIFE != 0 {
                let life = r.var()?;
                let health = match r.byte()? {
                    0 => None,
                    1 => {
                        let units = i64::try_from(r.var()?)
                            .map_err(|_| invalid("Invalid replica health"))?;
                        let maximum = i64::try_from(r.var()?)
                            .map_err(|_| invalid("Invalid replica health"))?;
                        let scale = u32::try_from(r.var()?)
                            .map_err(|_| invalid("Invalid replica scale"))?;
                        let h = Health {
                            units,
                            maximum,
                            scale,
                        };
                        if !valid_health(h) {
                            return Err(invalid("Invalid replica health"));
                        }
                        Some(h)
                    }
                    _ => return Err(invalid("Invalid health tag")),
                };
                (life, health)
            } else {
                (before.unwrap().life, before.unwrap().health)
            };
            if let Some(old) = before
                && (life < old.life
                    || life == old.life
                        && old.health.is_some_and(|h| h.units == 0)
                        && health.is_some_and(|h| h.units > 0))
            {
                return Err(invalid("Replica life regressed"));
            }
            if before.is_none() {
                resulting += 1;
            }
            actor_count = actor_count
                - usize::from(before.is_some_and(|a| a.appearance.kind.actor()))
                + usize::from(appearance.kind.actor());
            if !appearance.kind.actor() && (life != 0 || health.is_some()) {
                return Err(invalid("Nonactor has private actor state"));
            }

            patches.push((
                id,
                Some(Actor {
                    id,
                    millimeters,
                    life,
                    health,
                    appearance,
                }),
            ));
        }
        let omitted_projectiles =
            u32::try_from(r.var()?).map_err(|_| invalid("Projectile omission count"))?;
        let shots = usize::try_from(r.var()?).map_err(|_| invalid("Projectile visual count"))?;
        if shots > MAX_PROJECTILE_VISUALS
            || u64::from(omitted_projectiles) + shots as u64
                > (crate::targeting::MAX_ACTORS * crate::projectiles::CAPACITY) as u64
        {
            return Err(invalid("Projectile visual budget"));
        }
        let mut projectiles = Vec::with_capacity(shots);
        let mut last = 0u64;
        for _ in 0..shots {
            let delta = r.var()?;
            if delta == 0 {
                return Err(invalid("Duplicate projectile visual"));
            }
            last = last
                .checked_add(delta)
                .ok_or_else(|| invalid("Projectile identity overflow"))?;
            let mut millimeters = [0; 3];
            for q in &mut millimeters {
                *q = i32::try_from(r.signed()?)
                    .map_err(|_| invalid("Projectile coordinate domain"))?;
            }
            let half_extent = r.float()?;
            if !half_extent.is_finite() || half_extent <= 0. || half_extent > 1. {
                return Err(invalid("Projectile extent domain"));
            }
            projectiles.push(Projectile {
                execution: last,
                millimeters,
                half_extent,
            });
        }
        let count = usize::try_from(r.var()?).map_err(|_| invalid("Avatar count"))?;
        if count > crate::targeting::MAX_ACTORS {
            return Err(invalid("Avatar budget"));
        }
        let mut avatars = Vec::with_capacity(count);
        let mut last_actor = 0;
        for _ in 0..count {
            let actor = r.var()?;
            let life = r.var()?;
            let asset =
                u32::try_from(r.var()?).map_err(|_| invalid("Avatar asset index exceeds u32"))?;
            let c = r.byte()?;
            let clip = (c != 255).then_some(c);
            let clip_tick = u32::try_from(r.var()?).map_err(|_| invalid("Avatar tick"))?;
            let c = r.byte()?;
            let previous_clip = (c != 255).then_some(c);
            let previous_tick = u32::try_from(r.var()?).map_err(|_| invalid("Avatar tick"))?;
            let blend = r.u16()?;
            let yaw = i16::from_le_bytes(r.bytes(2)?.try_into().unwrap());
            let pitch = i16::from_le_bytes(r.bytes(2)?.try_into().unwrap());
            let scale = r.float()?;
            let mut offset = [0.; 3];
            for x in &mut offset {
                *x = r.float()?;
            }
            let hide_body = match r.byte()? {
                0 => false,
                1 => true,
                _ => return Err(invalid("Avatar visibility flag")),
            };
            let mut orientation = [0.; 4];
            for x in &mut orientation {
                *x = r.float()?;
            }
            let strip_root = match r.byte()? {
                255 => None,
                b => Some(b),
            };
            let joint_count = r.byte()?;
            let ik_limited = r.byte()?;
            if joint_count as usize > crate::avatar_ik::JOINTS {
                return Err(invalid("Avatar joint budget"));
            }
            let mut joints = [crate::avatar_ik::Joint::default(); crate::avatar_ik::JOINTS];
            for j in &mut joints[..joint_count as usize] {
                j.bone = r.byte()?;
                for x in &mut j.translation {
                    *x = r.float()?;
                }
                for x in &mut j.rotation {
                    *x = r.float()?;
                }
            }
            let p = crate::npc_presentation::Pose {
                actor,
                life,
                asset,
                clip,
                clip_tick,
                previous_clip,
                previous_tick,
                blend,
                yaw,
                pitch,
                scale,
                offset,
                hide_body,
                orientation,
                strip_root,
                joint_count,
                ik_limited,
                joints,
            };
            p.validate()?;
            if actor <= last_actor {
                return Err(invalid("Avatar order"));
            }
            last_actor = actor;
            let owner = patches
                .iter()
                .find(|(id, _)| *id == actor)
                .and_then(|(_, a)| a.as_ref())
                .or_else(|| {
                    if !full && !patches.iter().any(|(id, _)| *id == actor) {
                        self.actors.get(&actor)
                    } else {
                        None
                    }
                });
            if owner.is_none_or(|a| a.life != life || a.appearance.kind != Kind::Npc) {
                return Err(invalid("Avatar owner/life absent"));
            }
            avatars.push(p);
        }
        if r.at != bytes.len()
            || resulting > MAX_OBJECTS
            || actor_count > crate::targeting::MAX_ACTORS
        {
            return Err(invalid("Invalid replica end/count"));
        }
        validate(&avatars)?;
        if full {
            self.actors.clear();
        }
        for (id, actor) in patches {
            if let Some(actor) = actor {
                self.actors.insert(id, actor);
            } else {
                self.actors.remove(&id);
            }
        }
        self.avatars = avatars;
        self.projectiles = projectiles;
        self.omitted_projectiles = omitted_projectiles;
        self.tick = Some(tick);
        Ok(Applied::Current)
    }
}
