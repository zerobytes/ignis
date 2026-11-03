//! Explicit, bounded 3D traversal graph. Nodes are feet positions, never target observations.
use crate::{
    model::{BodyKind, Entity, FIXED_DT, Result, Scene, invalid},
    navigation::{NavGoal, NavState},
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const G: f32 = 9.81;
const ANCHOR: f32 = 0.35;
const ARRIVAL: f32 = 0.12;
// Rapier contact friction leaves sub-millimetre residuals; use one physical tolerance
// for precise node arrival and local range checks, without relaxing swept clearance.
pub(crate) const NODE_ALIGNMENT: f32 = 0.002;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Walk,
    Jump,
    Glide,
    Fly,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Glide {
    pub speed: f32,
    pub sink_speed: f32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub walk: bool,
    pub jump_speed: Option<f32>,
    pub fly_speed: Option<f32>,
    pub glide: Option<Glide>,
}
/// Behaviour when a complete compatible route cannot be found; independent of the body.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum UnreachablePolicy {
    Stop,
    ApproachReachable {
        min_progress: f32,
        max_path_ticks: u32,
    },
}
impl UnreachablePolicy {
    pub fn validate(self) -> Result<()> {
        if let Self::ApproachReachable {
            min_progress,
            max_path_ticks,
        } = self
            && (!min_progress.is_finite()
                || !(0.05..=10.).contains(&min_progress)
                || !(1..=3600).contains(&max_path_ticks))
        {
            return Err(invalid(
                "Partial approach requires min_progress [0.05,10]m and max_path_ticks [1,3600]",
            ));
        }
        Ok(())
    }
}
/// Local linear connections supplement the authored graph without changing it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LocalConnections {
    pub walk: bool,
    pub fly: bool,
    pub max_distance: f32,
}
impl LocalConnections {
    pub fn validate(self, caps: Option<Capabilities>) -> Result<()> {
        let Some(caps) = caps else {
            return Err(invalid("Local connections require spatial mobility"));
        };
        if (!self.walk && !self.fly)
            || (self.walk && !caps.walk)
            || (self.fly && caps.fly_speed.is_none())
            || !self.max_distance.is_finite()
            || !(0.25..=8.).contains(&self.max_distance)
        {
            return Err(invalid(
                "Local connections require available walk/fly modes and max_distance [0.25,8]m",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
    Entry,
    Exit,
    Direct,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Connector {
    pub kind: ConnectorKind,
    pub mode: Mode,
    pub endpoint: [f32; 3],
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub from: u8,
    pub to: u8,
    pub mode: Mode,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub nodes: Vec<[f32; 3]>,
    pub links: Vec<Link>,
}
impl Network {
    pub fn validate(&self, scene: &Scene) -> Result<()> {
        if self.nodes.len() > 64
            || self.links.len() > 128
            || (self.nodes.is_empty() && !self.links.is_empty())
        {
            return Err(invalid(
                "Spatial network is limited to 64 nodes and 128 directed links",
            ));
        }
        if self
            .nodes
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 10000.)
        {
            return Err(invalid(
                "Spatial nodes must be finite XYZ feet positions within +/-10000m",
            ));
        }
        for (i, l) in self.links.iter().enumerate() {
            if l.from as usize >= self.nodes.len()
                || l.to as usize >= self.nodes.len()
                || l.from == l.to
                || self.links[..i].iter().any(|p| p == l)
            {
                return Err(invalid(
                    "Spatial links require distinct existing nodes and no duplicate link",
                ));
            }
            let d = Vec3::from(self.nodes[l.to as usize]) - Vec3::from(self.nodes[l.from as usize]);
            if !(0.1..=32.).contains(&d.length())
                || (l.mode == Mode::Walk
                    && d.y.abs() > 0.02
                    && !(scene.version >= 84
                        && scene
                            .entities
                            .iter()
                            .chain(scene.npc_templates.iter())
                            .any(|e| {
                                e.character_controller.is_some()
                                    && e.npc.as_ref().is_some_and(|n| n.mobility.is_some())
                            })))
                || (l.mode == Mode::Glide && d.y >= -0.1)
            {
                return Err(invalid(
                    "Spatial links need length [0.1,32]m; walk is level and glide descends",
                ));
            }
        }
        if (!self.nodes.is_empty()
            || scene
                .entities
                .iter()
                .any(|e| e.npc.as_ref().is_some_and(|n| n.mobility.is_some())))
            && scene.version < 19
        {
            return Err(invalid("Spatial mobility requires scene version 19"));
        }
        Ok(())
    }
}
impl Capabilities {
    pub fn validate(self, n: &crate::model::NpcConfig) -> Result<()> {
        let speed = |v: f32| v.is_finite() && (0.1..=10.).contains(&v);
        if self.jump_speed.is_some_and(|v| !speed(v))
            || self.fly_speed.is_some_and(|v| !speed(v))
            || self
                .glide
                .is_some_and(|g| !speed(g.speed) || !speed(g.sink_speed))
        {
            return Err(invalid("Mobility speeds must be finite in [0.1,10]m/s"));
        }
        if n.territory.is_some() || n.passage_priority.is_some() || n.navigation_recovery.is_some()
        {
            return Err(invalid(
                "Spatial mobility cannot yet combine with planar territory, passages or recovery",
            ));
        }
        if n.objectives.as_ref().is_some_and(|os| {
            os.iter().any(|o| {
                matches!(
                    o.action,
                    crate::objectives::Action::Guard { .. }
                        | crate::objectives::Action::Flee { .. }
                )
            })
        }) {
            return Err(invalid(
                "Spatial mobility uses guard_3d/respond/wait objectives; planar guard/flee are unsupported",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Motion {
    pub velocity: [f32; 3],
    pub seconds: f32,
    pub distance: f32,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Idle,
    Traversing,
    Arrived,
    PartialReached,
    Unreachable,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    #[serde(default)]
    pub motor_mode: Option<Mode>,
    pub phase: Phase,
    pub reason: String,
    pub edge: Option<u16>,
    pub connector: Option<Connector>,
    pub route: Vec<u16>,
    pub started_tick: u64,
    pub retry_tick: u64,
    pub motion: Option<Motion>,
    pub launch_position: Option<[f32; 3]>,
    pub completed: u32,
    pub plans: u32,
    pub destination: Option<[f32; 3]>,
    pub committed: bool,
    pub partial_target: Option<u8>,
    pub planned_stop_distance: f32,
    pub flight: Option<crate::flight::State>,
    pub flight_path: Vec<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ground_planning: Option<GroundPlanning>,
}
impl State {
    pub(crate) fn revive(&mut self) {
        self.stop_for("lifecycle_changed");
        self.phase = Phase::Idle;
        self.retry_tick = 0;
    }
    pub fn die(&mut self) {
        self.stop_for("actor_dead");
    }
    pub(crate) fn can_lock_movement(&self, network: &Network) -> bool {
        let mode = self
            .connector
            .map(|c| c.mode)
            .or_else(|| self.edge.map(|i| network.links[i as usize].mode));
        mode.is_none_or(|mode| mode == Mode::Walk)
            && self
                .flight
                .is_none_or(|f| Vec3::from(f.velocity).length_squared() < 0.0001)
    }
    pub(crate) fn pause_ground_for_cast(&mut self) {
        if self.edge.is_some() || self.connector.is_some() {
            self.started_tick = self.started_tick.saturating_add(1);
        }
        "casting_movement_lock".clone_into(&mut self.reason);
    }
    pub(crate) fn hold_for_cast(&mut self) {
        self.stop_for("casting_movement_lock");
    }
    fn stop_for(&mut self, reason: &str) {
        self.phase = Phase::Idle;
        reason.clone_into(&mut self.reason);
        self.route.clear();
        self.flight_path.clear();
        self.committed = false;
        self.edge = None;
        self.connector = None;
        self.destination = None;
        self.partial_target = None;
        self.motion = None;
        self.launch_position = None;
        self.flight = None;
        self.motor_mode = None;
        self.ground_planning = None;
    }
}
impl Default for State {
    fn default() -> Self {
        Self {
            motor_mode: None,
            phase: Phase::Idle,
            reason: "no_known_goal".into(),
            edge: None,
            connector: None,
            route: Vec::with_capacity(64),
            started_tick: 0,
            retry_tick: 0,
            motion: None,
            launch_position: None,
            completed: 0,
            plans: 0,
            destination: None,
            committed: false,
            partial_target: None,
            planned_stop_distance: 0.,
            flight: None,
            flight_path: Vec::new(),
            ground_planning: None,
        }
    }
}
#[derive(Default)]
pub struct Runtime {
    pub states: BTreeMap<String, State>,
    pub compiled: BTreeMap<String, Vec<Option<Motion>>>,
    pub rejection_reasons: BTreeMap<String, Vec<Option<&'static str>>>,
    pub ground: Option<crate::ground_navigation::World>,
    pub ground_reports: BTreeMap<String, std::sync::Arc<[crate::ground_navigation::Report]>>,
}
pub struct Control {
    pub mode: Mode,
    pub velocity: Vec3,
    pub gravity: f32,
}
fn feet(p: Vec3, entity: &Entity) -> Vec3 {
    p - Vec3::Y * entity.transform.size[1] * 0.5
}
fn center(p: Vec3, entity: &Entity) -> Vec3 {
    p + Vec3::Y * entity.transform.size[1] * 0.5
}
fn supported(scene: &Scene, e: &Entity, p: Vec3) -> bool {
    crate::walk_support::query(scene, e, p, p).continuous()
}
fn segment_box(a: Vec3, b: Vec3, c: Vec3, h: Vec3) -> bool {
    let d = b - a;
    let (mut enter, mut leave) = (0_f32, 1_f32);
    for axis in 0..3 {
        if d[axis].abs() < 1e-7 {
            if a[axis] <= c[axis] - h[axis] || a[axis] >= c[axis] + h[axis] {
                return false;
            }
        } else {
            let u = (c[axis] - h[axis] - a[axis]) / d[axis];
            let v = (c[axis] + h[axis] - a[axis]) / d[axis];
            enter = enter.max(u.min(v));
            leave = leave.min(u.max(v));
            if enter >= leave {
                return false;
            }
        }
    }
    true
}
fn clear(scene: &Scene, e: &Entity, a: Vec3, b: Vec3) -> bool {
    let half = Vec3::from(e.transform.size) * 0.5;
    !scene.entities.iter().any(|o| {
        o.id != e.id
            && o.body == Some(BodyKind::Fixed)
            && if o.geometry.is_box() {
                segment_box(
                    center(a, e),
                    center(b, e),
                    o.transform.position.into(),
                    Vec3::from(o.transform.size) * 0.5 + half - Vec3::splat(0.005),
                )
            } else {
                crate::geometry::Volume::entity(o, o.transform.position.into())
                    .cast(center(a, e), b - a, half - Vec3::splat(0.005), true, 1e-7)
                    .is_some()
            }
    })
}
/// Static compilation is deliberately outside the simulation tick. Dynamic bodies remain Rapier obstacles.
pub(crate) fn admit_motion(scene: &Scene, e: &Entity, l: Link, a: Vec3, b: Vec3) -> Option<Motion> {
    motion(scene, e, l, a, b)
}
fn motion(scene: &Scene, e: &Entity, l: Link, a: Vec3, b: Vec3) -> Option<Motion> {
    let n = e.npc.as_ref()?;
    let c = n.mobility?;
    let d = b - a;
    let horizontal = Vec3::new(d.x, 0., d.z).length();
    let (seconds, v) = match l.mode {
        Mode::Walk if c.walk => {
            if !crate::walk_support::query(scene, e, a, b).continuous() {
                return None;
            }
            let t = horizontal / n.speed;
            (t, d / t)
        }
        Mode::Fly => {
            let t = d.length() / c.fly_speed?;
            (t, d / t)
        }
        Mode::Glide => {
            let g = c.glide?;
            if d.y >= -0.1 || !supported(scene, e, b) {
                return None;
            }
            let t = -d.y / g.sink_speed;
            if horizontal / t > g.speed {
                return None;
            }
            (t, d / t)
        }
        Mode::Jump => {
            let vy = c.jump_speed?;
            if !supported(scene, e, a) || !supported(scene, e, b) {
                return None;
            }
            let discriminant = vy * vy - 2. * G * d.y;
            if discriminant < 0. {
                return None;
            }
            let t = (vy + discriminant.sqrt()) / G;
            if horizontal / t > n.speed {
                return None;
            }
            (t, Vec3::new(d.x / t, vy, d.z / t))
        }
        _ => return None,
    };
    if !seconds.is_finite() || !(FIXED_DT..=10.).contains(&seconds) {
        return None;
    }
    if l.mode != Mode::Jump {
        return clear(scene, e, a, b).then_some(Motion {
            velocity: v.into(),
            seconds,
            distance: d.length(),
        });
    }
    let mut distance = 0.;
    let samples = (seconds / FIXED_DT).ceil() as u32;
    let mut previous = a;
    for i in 1..=samples {
        let t = seconds * i as f32 / samples as f32;
        let p = if l.mode == Mode::Jump {
            a + v * t - Vec3::Y * (0.5 * G * t * t)
        } else {
            a + d * (t / seconds)
        };
        if !clear(scene, e, previous, p) {
            return None;
        }
        distance += previous.distance(p);
        previous = p;
    }
    Some(Motion {
        velocity: v.into(),
        seconds,
        distance,
    })
}

fn ground_motion(
    e: &Entity,
    l: Link,
    a: Vec3,
    b: Vec3,
    r: crate::ground_navigation::Report,
) -> Option<Motion> {
    if l.mode != Mode::Walk || r.status != crate::ground_navigation::Status::Arrived {
        return None;
    }
    let d = b - a;
    let h = Vec3::new(d.x, 0., d.z);
    Some(Motion {
        velocity: (h.normalize_or_zero() * e.npc.as_ref().unwrap().speed).into(),
        seconds: r.ticks.max(1) as f32 * FIXED_DT,
        distance: r.distance,
    })
}

/// Linear local geometry: bounded by range, tested once for the whole swept body.
fn local_motion(
    scene: &Scene,
    e: &Entity,
    config: LocalConnections,
    a: Vec3,
    b: Vec3,
) -> Option<(Mode, Motion)> {
    let d = b - a;
    let distance_squared = d.length_squared();
    let range = config.max_distance + NODE_ALIGNMENT;
    if distance_squared < 1e-10 || distance_squared > range * range || !clear(scene, e, a, b) {
        return None;
    }
    let n = e.npc.as_ref().unwrap();
    let caps = n.mobility.unwrap();
    let distance = distance_squared.sqrt();
    let mut best = None;
    for (enabled, mode, speed) in [
        (config.walk, Mode::Walk, n.speed),
        (config.fly, Mode::Fly, caps.fly_speed.unwrap_or(0.)),
    ] {
        if !enabled {
            continue;
        }
        if mode == Mode::Walk && !crate::walk_support::query(scene, e, a, b).continuous() {
            continue;
        }
        let seconds = (distance / speed).max(FIXED_DT);
        if seconds > 10. {
            continue;
        }
        let m = Motion {
            velocity: (d / seconds).into(),
            seconds,
            distance,
        };
        if best.is_none_or(|(_, previous): (Mode, Motion)| ticks(m) < ticks(previous)) {
            best = Some((mode, m));
        }
    }
    best
}
fn ticks(m: Motion) -> u32 {
    (m.seconds / FIXED_DT).ceil() as u32
}

fn route_cost(m: Motion, mode: Mode, profile: Option<crate::npc_mind::Profile>) -> u32 {
    let preference = profile.map_or(0, |p| {
        if mode == Mode::Walk {
            return 0;
        }
        let aversion = (p.traits[1] as i32 + p.traits[3] as i32 - p.traits[4] as i32).max(0) as u32;
        aversion * (5000 + p.intelligence as u32 / 2) / 10_000 / 200
    });
    ticks(m).saturating_add(preference)
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroundPlanning {
    pub probes: u8,
    pub steps: u16,
    pub limited: bool,
    #[serde(default)]
    pub step_limit: u16,
    #[serde(default)]
    pub entry_limit: u8,
    #[serde(default)]
    pub exit_limit: u8,
}
const LOCAL_PROBES: usize = 8;
const LOCAL_STEPS: u16 = 2400;
#[derive(Clone, Copy)]
struct LocalRecord {
    a: Vec3,
    b: Vec3,
    motion: Option<(Mode, Motion)>,
}
struct LocalQueries<'a> {
    scene: &'a Scene,
    entity: &'a Entity,
    config: LocalConnections,
    ground: Option<&'a crate::ground_navigation::World>,
    stats: GroundPlanning,
    cache: [Option<LocalRecord>; LOCAL_PROBES],
    entries: [bool; 64],
    exits: [bool; 64],
    profile: Option<crate::npc_mind::Profile>,
}
impl<'a> LocalQueries<'a> {
    fn new(
        scene: &'a Scene,
        entity: &'a Entity,
        config: LocalConnections,
        ground: Option<&'a crate::ground_navigation::World>,
        own: Vec3,
        wanted: Vec3,
        profile: Option<crate::npc_mind::Profile>,
    ) -> Self {
        let mut q = Self {
            scene,
            entity,
            config,
            ground,
            stats: GroundPlanning {
                step_limit: profile.map_or(LOCAL_STEPS, |p| p.search_steps()),
                entry_limit: profile.map_or(3, |p| p.search_entries() as u8),
                exit_limit: profile.map_or(4, |p| p.search_exits() as u8),
                ..Default::default()
            },
            cache: [None; LOCAL_PROBES],
            entries: [true; 64],
            exits: [true; 64],
            profile,
        };
        if ground.is_some() {
            q.entries = q.candidates(own, q.stats.entry_limit as usize);
            q.exits = q.candidates(wanted, q.stats.exit_limit as usize);
        }
        q
    }
    fn point(&self, p: [f32; 3]) -> Vec3 {
        Vec3::from(p)
            + Vec3::Y
                * self
                    .ground
                    .map_or(0., |_| self.entity.character_controller.unwrap().offset)
    }
    fn candidates(&mut self, origin: Vec3, count: usize) -> [bool; 64] {
        let mut order: [usize; 64] = std::array::from_fn(|i| i);
        let nodes = &self.scene.spatial_network.nodes;
        order[..nodes.len()].sort_unstable_by(|&a, &b| {
            self.point(nodes[a])
                .distance_squared(origin)
                .total_cmp(&self.point(nodes[b]).distance_squared(origin))
                .then(a.cmp(&b))
        });
        let mut mask = [false; 64];
        let mut eligible = 0;
        for &i in &order[..nodes.len()] {
            if self.point(nodes[i]).distance_squared(origin)
                <= (self.config.max_distance + NODE_ALIGNMENT).powi(2)
            {
                if eligible < count {
                    mask[i] = true;
                } else {
                    self.stats.limited = true;
                }
                eligible += 1;
            }
        }
        mask
    }
    fn motion(&mut self, a: Vec3, b: Vec3) -> Option<(Mode, Motion)> {
        let Some(ground) = self.ground else {
            return local_motion(self.scene, self.entity, self.config, a, b);
        };
        if a.distance_squared(b) > (self.config.max_distance + NODE_ALIGNMENT).powi(2) {
            return None;
        }
        if let Some(r) = self.cache.iter().flatten().find(|r| r.a == a && r.b == b) {
            return r.motion;
        }
        if self.stats.probes as usize == LOCAL_PROBES || self.stats.steps >= self.stats.step_limit {
            self.stats.limited = true;
            return None;
        }
        let r = ground.trace_limited(
            self.entity,
            a.into(),
            b.into(),
            (self.stats.step_limit - self.stats.steps).min(crate::ground_navigation::MAX_TICKS),
        );
        self.stats.steps += r.ticks;
        self.stats.limited |= r.status == crate::ground_navigation::Status::BudgetExceeded;
        let ground_m = ground_motion(
            self.entity,
            Link {
                from: 0,
                to: 0,
                mode: Mode::Walk,
            },
            a,
            b,
            r,
        )
        .map(|m| (Mode::Walk, m));
        let air_m = if self.config.fly {
            local_motion(
                self.scene,
                self.entity,
                LocalConnections {
                    walk: false,
                    fly: true,
                    max_distance: self.config.max_distance,
                },
                a,
                b,
            )
        } else {
            None
        };
        let m = match (ground_m, air_m) {
            (Some(g), Some(f)) => Some(
                if route_cost(g.1, g.0, self.profile) <= route_cost(f.1, f.0, self.profile) {
                    g
                } else {
                    f
                },
            ),
            (g, f) => g.or(f),
        };
        self.cache[self.stats.probes as usize] = Some(LocalRecord { a, b, motion: m });
        self.stats.probes += 1;
        m
    }
}

#[allow(clippy::too_many_arguments)]
fn plan_local(
    scene: &Scene,
    e: &Entity,
    goal: &NavGoal,
    own: Vec3,
    wanted: Vec3,
    compiled: &[Option<Motion>],
    config: LocalConnections,
    state: &mut State,
    ground: Option<&crate::ground_navigation::World>,
    profile: Option<crate::npc_mind::Profile>,
) -> std::result::Result<Option<Connector>, &'static str> {
    let mut queries = LocalQueries::new(scene, e, config, ground, own, wanted, profile);
    let result = plan_local_inner(
        scene,
        e,
        goal,
        own,
        wanted,
        compiled,
        config,
        state,
        &mut queries,
    );
    state.ground_planning = ground.map(|_| queries.stats);
    if result.is_err() && ground.is_some() && queries.stats.limited {
        Err("ground_local_budget")
    } else {
        result
    }
}

/// Multi-source Dijkstra. Temporary connectors never enter the authored network.
#[allow(clippy::too_many_arguments)]
fn plan_local_inner(
    scene: &Scene,
    e: &Entity,
    goal: &NavGoal,
    own: Vec3,
    wanted: Vec3,
    compiled: &[Option<Motion>],
    config: LocalConnections,
    state: &mut State,
    queries: &mut LocalQueries<'_>,
) -> std::result::Result<Option<Connector>, &'static str> {
    let network = &scene.spatial_network;
    let direct = queries.motion(own, wanted);
    let mut best_cost = direct.map_or(u32::MAX, |(mode, m)| route_cost(m, mode, queries.profile));
    let mut target: Option<usize> = None;
    let mut costs = [u32::MAX; 64];
    let mut visited = [false; 64];
    let mut previous = [None; 64];
    let mut entries = [None; 64];
    let mut exits = [None; 64];
    let own_supported = queries.ground.is_none() && (config.fly || supported(scene, e, own));
    for (i, point) in network.nodes.iter().enumerate() {
        let point = queries.point(*point);
        let anchored = if queries.ground.is_some() {
            // A hybrid flyer can occupy a precise free-space node without ground contact.
            // Merely having fly capability never permits a displaced/obstructed anchor.
            config.fly && own.distance_squared(point) <= NODE_ALIGNMENT * NODE_ALIGNMENT
        } else {
            own.distance_squared(point) <= ARRIVAL * ARRIVAL && own_supported
        };
        if anchored && clear(scene, e, own, point) {
            costs[i] = 0;
        } else if queries.entries[i]
            && let Some((mode, m)) = queries.motion(own, point)
        {
            let d = point - own;
            if queries.ground.is_some()
                && d.x * d.x + d.z * d.z <= NODE_ALIGNMENT * NODE_ALIGNMENT
                && d.y.abs() <= ARRIVAL
            {
                // The physical probe confirmed this anchor; no zero-length entry loop.
                costs[i] = 0;
            } else {
                costs[i] = route_cost(m, mode, queries.profile);
                entries[i] = Some(Connector {
                    kind: ConnectorKind::Entry,
                    mode,
                    endpoint: point.into(),
                });
            }
        }
    }
    let direct_kind = if costs.contains(&0) {
        ConnectorKind::Exit
    } else {
        ConnectorKind::Direct
    };
    for _ in 0..network.nodes.len() {
        let next = (0..network.nodes.len())
            .filter(|&i| !visited[i] && costs[i] != u32::MAX)
            .min_by_key(|&i| (costs[i], i));
        let Some(i) = next else { break };
        visited[i] = true;
        let point = queries.point(network.nodes[i]);
        let exit_cost = if queries.ground.is_none()
            && point.distance_squared(wanted) <= goal.stop_distance * goal.stop_distance
            && clear(scene, e, point, wanted)
        {
            Some(0)
        } else if queries.exits[i]
            && (queries.ground.is_none() || costs[i] < best_cost)
            && let Some((mode, m)) = queries.motion(point, wanted)
        {
            exits[i] = Some(Connector {
                kind: ConnectorKind::Exit,
                mode,
                endpoint: wanted.into(),
            });
            Some(route_cost(m, mode, queries.profile))
        } else {
            None
        };
        if let Some(exit_cost) = exit_cost {
            let total = costs[i] + exit_cost;
            if total < best_cost || (total == best_cost && target.is_some_and(|old| i < old)) {
                target = Some(i);
                best_cost = total;
            }
        }
        for (edge, link) in network
            .links
            .iter()
            .enumerate()
            .filter(|(_, l)| l.from as usize == i)
        {
            let Some(m) = compiled[edge] else { continue };
            let cost = costs[i] + route_cost(m, link.mode, queries.profile);
            let to = link.to as usize;
            if cost < costs[to] {
                costs[to] = cost;
                previous[to] = Some(edge as u16);
            }
        }
    }
    state.route.clear();
    state.partial_target = None;
    if best_cost == u32::MAX {
        if let Some(UnreachablePolicy::ApproachReachable {
            min_progress,
            max_path_ticks,
        }) = e.npc.as_ref().unwrap().unreachable_policy
        {
            let improved = own.distance(wanted) - min_progress - ARRIVAL;
            if improved >= 0. {
                target = (0..network.nodes.len())
                    .filter(|&i| {
                        visited[i]
                            && costs[i] > 0
                            && costs[i] <= max_path_ticks
                            && Vec3::from(network.nodes[i]).distance_squared(wanted)
                                <= improved * improved
                    })
                    .min_by(|&i, &j| {
                        Vec3::from(network.nodes[i])
                            .distance_squared(wanted)
                            .total_cmp(&Vec3::from(network.nodes[j]).distance_squared(wanted))
                            .then(costs[i].cmp(&costs[j]))
                            .then(i.cmp(&j))
                    });
                state.partial_target = target.map(|i| i as u8);
            }
            if target.is_none() {
                return Err("no_partial_progress");
            }
        } else {
            return Err("no_local_route");
        }
    }
    let Some(target) = target else {
        return Ok(direct.map(|(mode, _)| Connector {
            kind: direct_kind,
            mode,
            endpoint: wanted.into(),
        }));
    };
    let mut root = target;
    while let Some(edge) = previous[root] {
        state.route.push(edge);
        root = network.links[edge as usize].from as usize;
    }
    state.route.reverse();
    if let Some(entry) = entries[root] {
        return Ok(Some(entry));
    }
    if !state.route.is_empty() || state.partial_target.is_some() {
        return Ok(None);
    }
    if let Some(exit) = exits[target] {
        if direct.is_none() {
            // A tolerance-based anchor is not a physical teleport. Align before an exit
            // whose range/geometry is valid only from the authored node.
            let point = queries.point(network.nodes[root]);
            let offset = point - own;
            if (offset.x * offset.x + offset.z * offset.z > NODE_ALIGNMENT * NODE_ALIGNMENT
                || (config.fly && offset.y.abs() > NODE_ALIGNMENT))
                && let Some((mode, _)) = queries.motion(own, point)
            {
                return Ok(Some(Connector {
                    kind: ConnectorKind::Entry,
                    mode,
                    endpoint: point.into(),
                }));
            }
            return Err("local_geometry_blocked");
        }
        return Ok(Some(exit));
    }
    // A nearby graph anchor can be within the goal radius while the physical body is not.
    direct
        .map(|(mode, _)| {
            Some(Connector {
                kind: direct_kind,
                mode,
                endpoint: wanted.into(),
            })
        })
        .ok_or("no_local_route")
}

impl Runtime {
    pub(crate) fn refresh_geometry(&mut self, scene: &Scene, previous: &Network) {
        let mut rebuilt = Self::new(scene);
        for (id, mut state) in std::mem::take(&mut self.states) {
            if let Some(edge) = state.edge
                && previous.links[edge as usize].mode != Mode::Walk
            {
                let link = previous.links[edge as usize];
                state.connector = Some(Connector {
                    kind: ConnectorKind::Entry,
                    mode: link.mode,
                    endpoint: previous.nodes[link.to as usize],
                });
                state.edge = None;
            }
            state.route.clear();
            state.partial_target = None;
            if state.connector.is_none_or(|c| c.mode == Mode::Walk)
                && state.motor_mode.is_none_or(|m| m == Mode::Walk)
            {
                state.revive();
            }
            rebuilt.states.insert(id, state);
        }
        *self = rebuilt;
    }
    pub fn new(scene: &Scene) -> Self {
        let mut r = Self::default();
        if scene
            .entities
            .iter()
            .chain(scene.npc_templates.iter())
            .any(|e| {
                e.character_controller.is_some()
                    && e.npc
                        .as_ref()
                        .is_some_and(|n| n.mobility.is_some() || n.circulation.is_some())
            })
        {
            let mut world = crate::ground_navigation::World::new(scene);
            for e in &scene.npc_templates {
                if e.character_controller.is_some()
                    && e.npc.as_ref().is_some_and(|n| n.mobility.is_some())
                {
                    world.compile(scene, e);
                }
            }
            r.ground = Some(world);
        }
        for e in &scene.entities {
            if !crate::templates::vacant(e) {
                r.install(scene, e);
            }
        }
        r
    }
    pub(crate) fn install(&mut self, scene: &Scene, e: &crate::model::Entity) {
        if e.npc.as_ref().is_none_or(|n| n.mobility.is_none()) {
            return;
        }
        self.states.insert(e.id.clone(), State::default());
        if e.character_controller.is_some() {
            let reports = self
                .ground
                .as_mut()
                .expect("ground world prepared from definitions")
                .compile(scene, e);
            self.compiled.insert(
                e.id.clone(),
                reports
                    .iter()
                    .zip(&scene.spatial_network.links)
                    .map(|(r, l)| {
                        let a = scene.spatial_network.nodes[l.from as usize].into();
                        let b = scene.spatial_network.nodes[l.to as usize].into();
                        if l.mode == Mode::Walk {
                            ground_motion(e, *l, a, b, *r)
                        } else {
                            motion(scene, e, *l, a, b)
                        }
                    })
                    .collect(),
            );
            self.rejection_reasons.insert(
                e.id.clone(),
                reports
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        if self.compiled[&e.id][i].is_some() {
                            None
                        } else if scene.spatial_network.links[i].mode == Mode::Walk {
                            r.status.reason()
                        } else {
                            Some("static_geometry_or_missing_capability")
                        }
                    })
                    .collect(),
            );
            self.ground_reports.insert(e.id.clone(), reports);
            return;
        }
        self.compiled.insert(
            e.id.clone(),
            scene
                .spatial_network
                .links
                .iter()
                .map(|l| {
                    motion(
                        scene,
                        e,
                        *l,
                        scene.spatial_network.nodes[l.from as usize].into(),
                        scene.spatial_network.nodes[l.to as usize].into(),
                    )
                })
                .collect(),
        );
        if let Some(c) = e.npc.as_ref().and_then(|n| n.mobility) {
            self.rejection_reasons.insert(
                e.id.clone(),
                scene
                    .spatial_network
                    .links
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        if self.compiled[&e.id][i].is_some() {
                            None
                        } else if !match l.mode {
                            Mode::Walk => c.walk,
                            Mode::Jump => c.jump_speed.is_some(),
                            Mode::Fly => c.fly_speed.is_some(),
                            Mode::Glide => c.glide.is_some(),
                        } {
                            Some("missing_capability")
                        } else if l.mode == Mode::Walk {
                            crate::walk_support::query(
                                scene,
                                e,
                                scene.spatial_network.nodes[l.from as usize].into(),
                                scene.spatial_network.nodes[l.to as usize].into(),
                            )
                            .rejection()
                            .or(Some("static_geometry_or_motion_constraint"))
                        } else {
                            Some("static_geometry_or_motion_constraint")
                        }
                    })
                    .collect(),
            );
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn control(
        &mut self,
        scene: &Scene,
        e: &Entity,
        goal: Option<&NavGoal>,
        nav: &mut NavState,
        tick: u64,
        p: Vec3,
        velocity: Vec3,
        controller_grounded: Option<bool>,
        profile: Option<crate::npc_mind::Profile>,
    ) -> Control {
        self.control_with_circulation(
            scene,
            e,
            goal,
            nav,
            tick,
            p,
            velocity,
            controller_grounded,
            profile,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn control_with_circulation(
        &mut self,
        scene: &Scene,
        e: &Entity,
        goal: Option<&NavGoal>,
        nav: &mut NavState,
        tick: u64,
        p: Vec3,
        velocity: Vec3,
        controller_grounded: Option<bool>,
        profile: Option<crate::npc_mind::Profile>,
        mut air: Option<crate::npc_circulation::Air<'_>>,
    ) -> Control {
        let out = self.control_inner(
            scene,
            e,
            goal,
            nav,
            tick,
            p,
            velocity,
            controller_grounded,
            profile,
            air.as_mut(),
        );
        let state = self.states.get_mut(&e.id).unwrap();
        state.motor_mode = Some(out.mode);
        if state.phase == Phase::Unreachable
            && scene
                .navigation_generation
                .as_ref()
                .and_then(|r| r.cached.as_ref())
                .is_some_and(|c| c.report.limited)
            && [
                "no_local_route",
                "target_not_on_network",
                "no_compatible_route",
                "not_anchored_to_network",
                "no_partial_progress",
            ]
            .contains(&state.reason.as_str())
        {
            "generated_navigation_incomplete".clone_into(&mut state.reason);
            state.reason.clone_into(&mut nav.reason);
        }
        out
    }
    #[allow(clippy::too_many_arguments)]
    fn control_inner(
        &mut self,
        scene: &Scene,
        e: &Entity,
        goal: Option<&NavGoal>,
        nav: &mut NavState,
        tick: u64,
        p: Vec3,
        velocity: Vec3,
        controller_grounded: Option<bool>,
        profile: Option<crate::npc_mind::Profile>,
        mut air: Option<&mut crate::npc_circulation::Air<'_>>,
    ) -> Control {
        let n = e.npc.as_ref().unwrap();
        let caps = n.mobility.unwrap();
        let network = &scene.spatial_network;
        let s = self.states.get_mut(&e.id).unwrap();
        let compiled = &self.compiled[&e.id];
        let mut out = Control {
            mode: if e.character_controller.is_some() {
                s.motor_mode
                    .filter(|m| *m == Mode::Fly)
                    .unwrap_or(Mode::Walk)
            } else if caps.fly_speed.is_some() {
                Mode::Fly
            } else {
                Mode::Walk
            },
            velocity: Vec3::new(0., velocity.y, 0.),
            gravity: 1.,
        };
        if caps.fly_speed.is_some() {
            out.velocity.y = 0.;
            out.gravity = 0.;
        }
        let own = feet(p, e);
        let own_supported = || controller_grounded.unwrap_or_else(|| supported(scene, e, own));
        let goal = goal.filter(|_| n.enabled);
        let wanted = goal.map(|g| Vec3::from(g.position) - Vec3::Y * g.target_half_height);
        s.committed = (s.edge.is_some() || s.connector.is_some())
            && (wanted.map(<[f32; 3]>::from) != s.destination
                || goal.is_none_or(|g| g.stop_distance != s.planned_stop_distance));
        if let Some(mut flight) = s.flight.filter(|f| f.holding) {
            if !n.enabled {
                s.flight = None;
                s.flight_path.clear();
            } else {
                let tracking = flight.tracks(own);
                let v = if let Some(air) = air.as_deref_mut() {
                    let mut sample = flight;
                    let tangent = sample.orbit(own).normalize_or_zero();
                    let (v, next, _) = air.advance(
                        scene,
                        e,
                        own,
                        own + tangent * 16.,
                        Some((n.flight.unwrap().settings().unwrap(), flight)),
                        caps.fly_speed.unwrap(),
                        ARRIVAL,
                        crate::npc_circulation::AirPhase::Holding,
                    );
                    flight = next.unwrap();
                    v
                } else {
                    flight.orbit(own)
                };
                if !tracking || !clear(scene, e, own, own + v * FIXED_DT) {
                    stop_flight(s, tick, "flight_holding_interrupted");
                    publish(s, network, e, nav, tick);
                    return out;
                }
                s.flight = Some(flight);
                out.velocity = v;
                out.mode = Mode::Fly;
                if goal.is_none()
                    || (s.destination == wanted.map(<[f32; 3]>::from)
                        && goal.is_some_and(|g| g.stop_distance == s.planned_stop_distance)
                        && (s.phase == Phase::Arrived
                            || (s.phase == Phase::Unreachable && tick < s.retry_tick)))
                {
                    if goal.is_none() {
                        s.phase = Phase::Idle;
                        s.destination = None;
                    }
                    (if goal.is_none() {
                        "holding_without_known_goal"
                    } else if s.phase == Phase::Unreachable {
                        "holding_while_route_unavailable"
                    } else {
                        "holding_after_arrival"
                    })
                    .clone_into(&mut s.reason);
                    publish(s, network, e, nav, tick);
                    return out;
                }
            }
        }
        let traversal = s
            .connector
            .map(|c| (c.mode, Vec3::from(c.endpoint)))
            .or_else(|| {
                s.edge.map(|edge| {
                    let l = network.links[edge as usize];
                    (l.mode, Vec3::from(network.nodes[l.to as usize]))
                })
            });
        if let Some((mode, end)) = traversal {
            out.mode = mode;
            let elapsed = tick - s.started_tick;
            let m = s.motion.unwrap();
            let curved = mode == Mode::Fly
                && n.flight
                    .and_then(crate::flight::Profile::settings)
                    .is_some();
            let mut curved_arrived = false;
            if curved {
                let config = n.flight.unwrap().settings().unwrap();
                let mut flight = s.flight.expect("curved traversal has flight state");
                if !flight.tracks(own) {
                    stop_flight(s, tick, "flight_tracking_interrupted");
                    publish(s, network, e, nav, tick);
                    return out;
                }
                curved_arrived = if let Some(air) = air.as_deref_mut() {
                    let (_, next, arrived) = air.advance(
                        scene,
                        e,
                        own,
                        end,
                        Some((config, flight)),
                        caps.fly_speed.unwrap(),
                        flight.arrival_radius,
                        crate::npc_circulation::AirPhase::Flight,
                    );
                    flight = next.unwrap();
                    arrived
                } else {
                    flight.advance(config, caps.fly_speed.unwrap(), own, end)
                };
                if curved_arrived {
                    if !config.can_hover {
                        if let Err(reason) =
                            crate::flight::validate_orbit(&mut flight, config, own, |a, b| {
                                clear(scene, e, a, b)
                            })
                        {
                            stop_flight(s, tick, reason);
                            publish(s, network, e, nav, tick);
                            return out;
                        }
                        out.velocity = flight.orbit(own);
                        out.mode = Mode::Fly;
                        s.flight = Some(flight);
                    } else {
                        // Retain the arrival heading for the next launch, even while hovering.
                        s.flight = Some(flight);
                    }
                } else {
                    let v = Vec3::from(flight.velocity);
                    if elapsed
                        > if air.is_some() {
                            crate::flight::MAX_TRAVEL_TICKS as u64
                        } else {
                            (m.seconds / FIXED_DT).ceil() as u64 + 120
                        }
                        || flight.ticks > crate::flight::MAX_TRAVEL_TICKS + 120
                    {
                        stop_flight(s, tick, "flight_traversal_timeout");
                    } else if !clear(scene, e, own, own + v * FIXED_DT) {
                        stop_flight(s, tick, "flight_path_interrupted");
                    } else {
                        s.flight = Some(flight);
                        out.velocity = v;
                        out.mode = Mode::Fly;
                        out.gravity = 0.;
                        s.phase = Phase::Traversing;
                        (if s.committed {
                            "finishing_committed_flight"
                        } else {
                            "executing_curved_flight"
                        })
                        .clone_into(&mut s.reason);
                    }
                    publish(s, network, e, nav, tick);
                    return out;
                }
            }
            let end_supported = mode == Mode::Fly && supported(scene, e, end);
            let precise_node = n.local_connections.is_some()
                && (s.edge.is_some()
                    || s.connector.is_some_and(|c| c.kind == ConnectorKind::Entry));
            let delta = end - own;
            let final_connection = s.connector.is_some_and(|c| c.kind != ConnectorKind::Entry);
            let near = if final_connection {
                own.distance_squared(end) <= s.planned_stop_distance * s.planned_stop_distance
                    && clear(scene, e, own, end)
            } else if precise_node {
                delta.x * delta.x + delta.z * delta.z <= NODE_ALIGNMENT * NODE_ALIGNMENT
                    && delta.y.abs()
                        <= if mode == Mode::Fly && !end_supported {
                            NODE_ALIGNMENT
                        } else {
                            ARRIVAL
                        }
            } else {
                own.distance_squared(end) <= ARRIVAL * ARRIVAL
            };
            // Rapier may still be resolving shallow penetration on the first landing tick.
            // Keep the segment active until the body can occupy the endpoint; otherwise the
            // immediately following plan reports an obstruction at its own landing surface.
            let landed =
                near && velocity.y.abs() < 0.5 && own_supported() && clear(scene, e, own, end);
            if curved_arrived
                || (elapsed > 0
                    && near
                    && ((mode == Mode::Fly && !end_supported) || landed)
                    && (!matches!(mode, Mode::Jump | Mode::Glide)
                        || elapsed as f32 * FIXED_DT >= m.seconds * 0.5))
            {
                if e.character_controller.is_some()
                    && landed
                    && mode == Mode::Fly
                    && s.flight.is_none_or(|f| !f.holding)
                {
                    out.mode = Mode::Walk;
                    out.gravity = 1.;
                }
                s.completed = s.completed.saturating_add(1);
                if s.partial_target.is_some() && !s.committed {
                    // Keep the whole partial route: an intermediate detour may be farther from the goal.
                    if s.edge.is_some() {
                        s.route.remove(0);
                    }
                } else {
                    s.route.clear();
                    s.partial_target = None;
                }
                s.committed = false;
                s.edge = None;
                s.connector = None;
                s.motion = None;
                s.phase = Phase::Idle;
                s.retry_tick = 0;
            } else if elapsed
                > if air.is_some() && mode == Mode::Fly {
                    crate::flight::MAX_TRAVEL_TICKS as u64
                } else {
                    (m.seconds / FIXED_DT).ceil() as u64 + 120
                }
            {
                s.edge = None;
                s.connector = None;
                s.motion = None;
                s.route.clear();
                s.partial_target = None;
                s.phase = Phase::Unreachable;
                s.committed = false;
                s.retry_tick = tick + 60;
                "traversal_timeout".clone_into(&mut s.reason);
            } else {
                let d = end - own;
                let mut v = Vec3::from(m.velocity);
                match mode {
                    Mode::Jump => {
                        v.y = velocity.y;
                        let remaining = Vec3::new(d.x, 0., d.z);
                        let speed = Vec3::new(v.x, 0., v.z).length();
                        let h = remaining.normalize_or_zero()
                            * speed.min(remaining.length() / FIXED_DT);
                        v.x = h.x;
                        v.z = h.z;
                        out.gravity = 1.;
                    }
                    Mode::Glide => {
                        out.gravity = 0.;
                        let h = Vec3::new(d.x, 0., d.z);
                        let speed = Vec3::new(v.x, 0., v.z).length();
                        let h = h.normalize_or_zero() * speed.min(h.length() / FIXED_DT);
                        v.x = h.x;
                        v.z = h.z; /* Sink never turns into an ascent. Rapier stops at the landing surface. */
                    }
                    Mode::Fly => {
                        out.gravity = 0.;
                        v = if let Some(air) = air.as_deref_mut() {
                            air.advance(
                                scene,
                                e,
                                own,
                                end,
                                None,
                                caps.fly_speed.unwrap(),
                                if final_connection {
                                    s.planned_stop_distance
                                } else {
                                    NODE_ALIGNMENT
                                },
                                crate::npc_circulation::AirPhase::Flight,
                            )
                            .0
                        } else {
                            d.normalize_or_zero()
                                * caps.fly_speed.unwrap().min(d.length() / FIXED_DT)
                        };
                    }
                    Mode::Walk => {
                        out.gravity = 1.;
                        let h = Vec3::new(d.x, 0., d.z);
                        v = h.normalize_or_zero() * n.speed.min(h.length() / FIXED_DT);
                        v.y = velocity.y;
                    }
                }
                out.velocity = v;
                s.phase = Phase::Traversing;
                if s.committed {
                    "finishing_committed_link".clone_into(&mut s.reason)
                } else if s.connector.is_some() {
                    "executing_local_connection".clone_into(&mut s.reason)
                } else {
                    "executing_link".clone_into(&mut s.reason)
                }
                publish(s, network, e, nav, tick);
                return out;
            }
        }
        let Some(g) = goal else {
            s.phase = Phase::Idle;
            s.destination = None;
            s.partial_target = None;
            s.planned_stop_distance = 0.;
            s.route.clear();
            "no_known_goal".clone_into(&mut s.reason);
            publish(s, network, e, nav, tick);
            return out;
        };
        let wanted = wanted.unwrap();
        if s.destination != Some(wanted.into()) || s.planned_stop_distance != g.stop_distance {
            // Failed expensive ground searches keep their cooldown even for moving cues.
            if !(e.character_controller.is_some()
                && n.local_connections.is_some()
                && s.phase == Phase::Unreachable
                && tick < s.retry_tick)
            {
                s.retry_tick = 0;
            }
            s.partial_target = None;
            s.route.clear();
        }
        s.planned_stop_distance = g.stop_distance;
        s.destination = Some(wanted.into());
        nav.effective_stop_distance = g.stop_distance;
        if own.distance_squared(wanted) <= g.stop_distance * g.stop_distance
            && clear(scene, e, own, wanted)
            && (caps.fly_speed.is_some() || own_supported())
        {
            s.phase = Phase::Arrived;
            s.partial_target = None;
            s.route.clear();
            "within_3d_stop_distance".clone_into(&mut s.reason);
            publish(s, network, e, nav, tick);
            return out;
        }
        if let Some(node) = s.partial_target
            && s.route.is_empty()
        {
            if own.distance_squared(network.nodes[node as usize].into()) <= ARRIVAL * ARRIVAL
                && (caps.fly_speed.is_some() || own_supported())
            {
                s.phase = Phase::PartialReached;
                "partial_route_end".clone_into(&mut s.reason);
                publish(s, network, e, nav, tick);
                return out;
            }
            // Physical displacement invalidates the stationary result, not the original goal.
            s.partial_target = None;
        }
        let mut next_connector = None;
        if s.partial_target.is_none() || s.route.is_empty() {
            if tick < s.retry_tick {
                publish(s, network, e, nav, tick);
                return out;
            }
            s.plans = s.plans.saturating_add(1);
            nav.planned_tick = Some(tick);
            s.retry_tick = tick + 60;
            if let Some(config) = n.local_connections {
                match plan_local(
                    scene,
                    e,
                    g,
                    own,
                    wanted,
                    compiled,
                    config,
                    s,
                    self.ground
                        .as_ref()
                        .filter(|_| e.character_controller.is_some()),
                    profile,
                ) {
                    Ok(connector) => next_connector = connector,
                    Err(reason) => {
                        s.phase = Phase::Unreachable;
                        reason.clone_into(&mut s.reason);
                        publish(s, network, e, nav, tick);
                        return out;
                    }
                }
            } else {
                let start = network
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, q)| own.distance_squared(Vec3::from(**q)) <= ANCHOR * ANCHOR)
                    .min_by(|(i, a), (j, b)| {
                        own.distance_squared(Vec3::from(**a))
                            .total_cmp(&own.distance_squared(Vec3::from(**b)))
                            .then(i.cmp(j))
                    })
                    .map(|(i, _)| i);
                let Some(start) = start else {
                    s.phase = Phase::Unreachable;
                    "not_anchored_to_network".clone_into(&mut s.reason);
                    publish(s, network, e, nav, tick);
                    return out;
                };
                let mut costs = [u32::MAX; 64];
                let mut visited = [false; 64];
                let mut previous = [None; 64];
                costs[start] = 0;
                let mut target = None;
                for _ in 0..network.nodes.len() {
                    let next = (0..network.nodes.len())
                        .filter(|i| !visited[*i] && costs[*i] != u32::MAX)
                        .min_by_key(|i| (costs[*i], *i));
                    let Some(i) = next else { break };
                    visited[i] = true;
                    if Vec3::from(network.nodes[i]).distance_squared(wanted)
                        <= g.stop_distance * g.stop_distance
                        && clear(scene, e, network.nodes[i].into(), wanted)
                    {
                        target = Some(i);
                        break;
                    }
                    for (index, l) in network
                        .links
                        .iter()
                        .enumerate()
                        .filter(|(_, l)| l.from as usize == i)
                    {
                        let Some(m) = compiled[index] else { continue };
                        let candidate = costs[i] + route_cost(m, l.mode, profile);
                        let to = l.to as usize;
                        if candidate < costs[to] {
                            costs[to] = candidate;
                            previous[to] = Some(index as u16);
                        }
                    }
                }
                s.partial_target = None;
                if target.is_none()
                    && let Some(UnreachablePolicy::ApproachReachable {
                        min_progress,
                        max_path_ticks,
                    }) = n.unreachable_policy
                {
                    // Reserve arrival tolerance so resolved progress still reaches the configured minimum.
                    let improved_distance = own.distance(wanted) - min_progress - ARRIVAL;
                    if improved_distance >= 0. {
                        target = (0..network.nodes.len())
                            .filter(|&i| {
                                i != start
                                    && visited[i]
                                    && costs[i] <= max_path_ticks
                                    && Vec3::from(network.nodes[i]).distance_squared(wanted)
                                        <= improved_distance * improved_distance
                            })
                            .min_by(|&i, &j| {
                                Vec3::from(network.nodes[i])
                                    .distance_squared(wanted)
                                    .total_cmp(
                                        &Vec3::from(network.nodes[j]).distance_squared(wanted),
                                    )
                                    .then(costs[i].cmp(&costs[j]))
                                    .then(i.cmp(&j))
                            });
                        s.partial_target = target.map(|i| i as u8);
                    }
                }
                s.route.clear();
                if let Some(mut i) = target {
                    while i != start {
                        let edge = previous[i].unwrap();
                        s.route.push(edge);
                        i = network.links[edge as usize].from as usize;
                    }
                    s.route.reverse();
                }
            }
        }
        if let Some(mut connector) = next_connector {
            let endpoint = Vec3::from(connector.endpoint);
            let launch = if let Some(world) = self
                .ground
                .as_ref()
                .filter(|_| e.character_controller.is_some() && connector.mode == Mode::Walk)
            {
                ground_motion(
                    e,
                    Link {
                        from: 0,
                        to: 0,
                        mode: Mode::Walk,
                    },
                    own,
                    endpoint,
                    world.trace(e, own.into(), endpoint.into()),
                )
                .map(|m| (Mode::Walk, m))
            } else {
                let mut config = n.local_connections.unwrap();
                if e.character_controller.is_some() {
                    config.walk = connector.mode == Mode::Walk;
                    config.fly = connector.mode == Mode::Fly;
                }
                local_motion(scene, e, config, own, endpoint)
            };
            let Some((mode, mut m)) = launch else {
                s.partial_target = None;
                s.route.clear();
                s.phase = Phase::Unreachable;
                "local_geometry_blocked".clone_into(&mut s.reason);
                publish(s, network, e, nav, tick);
                return out;
            };
            let radius = if connector.kind == ConnectorKind::Entry {
                NODE_ALIGNMENT
            } else {
                g.stop_distance
            };
            if matches!(mode, Mode::Jump | Mode::Glide)
                && supported(scene, e, own)
                && air
                    .as_deref_mut()
                    .is_some_and(|a| a.defer_launch(scene, e, own, m.velocity.into(), mode))
            {
                "air_launch_deferred".clone_into(&mut s.reason);
                publish(s, network, e, nav, tick);
                return out;
            }
            let holding = s.flight.filter(|f| f.holding);
            if let Err(reason) = prepare_flight(
                scene,
                e,
                s,
                mode,
                own,
                velocity,
                connector.endpoint.into(),
                radius,
                &mut m,
            ) {
                stop_flight(s, tick, reason);
                s.flight = holding;
                publish(s, network, e, nav, tick);
                return out;
            }
            if g.remaining_travel
                .is_some_and(|remaining| remaining < m.distance + 0.05)
            {
                s.partial_target = None;
                s.route.clear();
                s.phase = Phase::Unreachable;
                s.flight = holding;
                s.flight_path.clear();
                "insufficient_travel_budget".clone_into(&mut s.reason);
                publish(s, network, e, nav, tick);
                return out;
            }
            out.mode = mode;
            if mode == Mode::Walk && e.character_controller.is_some() {
                s.flight = None;
            }
            connector.mode = mode;
            s.connector = Some(connector);
            s.motion = Some(m);
            s.launch_position = Some(own.into());
            s.started_tick = tick;
            s.phase = Phase::Traversing;
            s.committed = false;
            "executing_local_connection".clone_into(&mut s.reason);
            out.velocity = m.velocity.into();
            out.gravity = if mode == Mode::Fly { 0. } else { 1. };
            if mode == Mode::Walk {
                out.velocity.y = velocity.y;
            }
            start_flight(scene, e, s, own, connector.endpoint.into(), &mut out, air);
            publish(s, network, e, nav, tick);
            return out;
        }
        let Some(&edge) = s.route.first() else {
            s.phase = Phase::Unreachable;
            let has_destination = network.nodes.iter().any(|p| {
                Vec3::from(*p).distance_squared(wanted) <= g.stop_distance * g.stop_distance
            });
            (if matches!(
                n.unreachable_policy,
                Some(UnreachablePolicy::ApproachReachable { .. })
            ) {
                "no_partial_progress"
            } else if has_destination {
                "no_compatible_route"
            } else {
                "target_not_on_network"
            })
            .clone_into(&mut s.reason);
            publish(s, network, e, nav, tick);
            return out;
        };
        let link = network.links[edge as usize];
        let endpoint = Vec3::from(network.nodes[link.to as usize]);
        let launch = if e.character_controller.is_some() && link.mode == Mode::Walk {
            ground_motion(
                e,
                link,
                own,
                endpoint,
                self.ground
                    .as_ref()
                    .unwrap()
                    .trace(e, own.into(), endpoint.into()),
            )
        } else {
            motion(scene, e, link, own, endpoint)
        };
        let Some(mut m) = launch else {
            s.phase = Phase::Unreachable;
            s.partial_target = None;
            s.route.clear();
            s.retry_tick = tick + 60;
            "launch_geometry_blocked".clone_into(&mut s.reason);
            publish(s, network, e, nav, tick);
            return out;
        };
        let radius = if n.local_connections.is_some() {
            NODE_ALIGNMENT
        } else {
            ARRIVAL
        };
        if matches!(link.mode, Mode::Jump | Mode::Glide)
            && supported(scene, e, own)
            && air
                .as_deref_mut()
                .is_some_and(|a| a.defer_launch(scene, e, own, m.velocity.into(), link.mode))
        {
            "air_launch_deferred".clone_into(&mut s.reason);
            publish(s, network, e, nav, tick);
            return out;
        }
        let holding = s.flight.filter(|f| f.holding);
        if let Err(reason) = prepare_flight(
            scene,
            e,
            s,
            link.mode,
            own,
            velocity,
            network.nodes[link.to as usize].into(),
            radius,
            &mut m,
        ) {
            stop_flight(s, tick, reason);
            s.flight = holding;
            publish(s, network, e, nav, tick);
            return out;
        }
        if g.remaining_travel
            .is_some_and(|remaining| remaining < m.distance + 0.05)
        {
            s.phase = Phase::Unreachable;
            s.partial_target = None;
            s.route.clear();
            s.retry_tick = tick + 60;
            s.flight = holding;
            s.flight_path.clear();
            "insufficient_travel_budget".clone_into(&mut s.reason);
            publish(s, network, e, nav, tick);
            return out;
        }
        out.mode = link.mode;
        if link.mode == Mode::Walk && e.character_controller.is_some() {
            s.flight = None;
        }
        s.edge = Some(edge);
        s.motion = Some(m);
        s.launch_position = Some(own.into());
        s.started_tick = tick;
        s.phase = Phase::Traversing;
        s.committed = false;
        "executing_link".clone_into(&mut s.reason);
        out.velocity = m.velocity.into();
        out.gravity = if matches!(link.mode, Mode::Fly | Mode::Glide) {
            0.
        } else {
            1.
        };
        if link.mode == Mode::Walk {
            out.velocity.y = velocity.y;
        }
        start_flight(
            scene,
            e,
            s,
            own,
            network.nodes[link.to as usize].into(),
            &mut out,
            air,
        );
        publish(s, network, e, nav, tick);
        out
    }
}
fn publish(s: &State, network: &Network, e: &Entity, nav: &mut NavState, tick: u64) {
    let status = match s.phase {
        Phase::Idle => "idle",
        Phase::Traversing => "following",
        Phase::Arrived => "arrived",
        Phase::Unreachable | Phase::PartialReached => "unreachable",
    };
    status.clone_into(&mut nav.status);
    s.reason.clone_into(&mut nav.reason);
    nav.decision_tick = tick;
    nav.target_position = s.destination.map(|p| center(p.into(), e).into());
    nav.path.clear();
    if let Some(c) = s.connector {
        nav.path.push(center(c.endpoint.into(), e).into());
    }
    for &edge in &s.route {
        nav.path.push(
            center(
                network.nodes[network.links[edge as usize].to as usize].into(),
                e,
            )
            .into(),
        );
    }
}

fn stop_flight(s: &mut State, tick: u64, reason: &str) {
    s.edge = None;
    s.connector = None;
    s.flight = None;
    s.flight_path.clear();
    s.motion = None;
    s.route.clear();
    s.partial_target = None;
    s.committed = false;
    s.phase = Phase::Unreachable;
    s.retry_tick = tick + 60;
    reason.clone_into(&mut s.reason);
}
#[allow(clippy::too_many_arguments)]
fn prepare_flight(
    scene: &Scene,
    e: &Entity,
    s: &mut State,
    mode: Mode,
    own: Vec3,
    velocity: Vec3,
    end: Vec3,
    radius: f32,
    motion: &mut Motion,
) -> std::result::Result<(), &'static str> {
    let n = e.npc.as_ref().unwrap();
    if mode == Mode::Fly
        && let Some(mut config) = n.flight.and_then(crate::flight::Profile::settings)
    {
        if let Some(previous) = s.flight {
            config.initial_direction = previous.heading.into();
        }
        let plan = crate::flight::preview(
            config,
            n.mobility.unwrap().fly_speed.unwrap(),
            own,
            velocity,
            end,
            radius,
            &mut s.flight_path,
            |a, b| clear(scene, e, a, b),
        )?;
        s.flight = Some(plan.initial);
        motion.seconds = plan.seconds;
        motion.distance = plan.distance;
    } else {
        s.flight = None;
        s.flight_path.clear();
    }
    Ok(())
}
fn start_flight(
    scene: &Scene,
    e: &Entity,
    s: &mut State,
    own: Vec3,
    end: Vec3,
    out: &mut Control,
    air: Option<&mut crate::npc_circulation::Air<'_>>,
) {
    let n = e.npc.as_ref().unwrap();
    if out.mode != Mode::Fly {
        return;
    }
    if let Some(air) = air {
        let motor = s.flight.map(|f| (n.flight.unwrap().settings().unwrap(), f));
        let (v, flight, _) = air.advance(
            scene,
            e,
            own,
            end,
            motor,
            n.mobility.unwrap().fly_speed.unwrap(),
            s.flight.map_or(NODE_ALIGNMENT, |f| f.arrival_radius),
            crate::npc_circulation::AirPhase::Flight,
        );
        s.flight = flight;
        out.velocity = v;
        out.gravity = 0.;
    } else if let Some(f) = s.flight.as_mut() {
        f.advance(
            n.flight.unwrap().settings().unwrap(),
            n.mobility.unwrap().fly_speed.unwrap(),
            own,
            end,
        );
        out.velocity = f.velocity.into();
        out.gravity = 0.;
        "executing_curved_flight".clone_into(&mut s.reason);
    }
}
