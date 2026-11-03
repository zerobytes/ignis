//! Authored, bounded navigation derivation. A layout map is never target evidence.
use crate::{
    geometry::Volume,
    ground_navigation::{MAX_TICKS, Status, World},
    mobility::{Link, Mode, Network},
    model::{BodyKind, Entity, Result, Scene, invalid},
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_probes() -> u16 {
    512
}
fn default_ticks() -> u32 {
    24_000
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub entity: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub cell_size: f32,
    pub connection_radius: f32,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub include_air: bool,
    #[serde(default = "default_probes")]
    pub max_probes: u16,
    #[serde(default = "default_ticks")]
    pub max_virtual_ticks: u32,
    #[serde(skip)]
    pub cached: Option<Cache>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cache {
    signature: u64,
    network: u64,
    pub report: Report,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Report {
    pub columns: u16,
    pub node_candidates: u16,
    pub nodes: u16,
    pub links: u16,
    pub probes: u16,
    pub virtual_ticks: u32,
    pub limited: bool,
}
fn source<'a>(scene: &'a Scene, id: &str) -> Option<&'a Entity> {
    scene
        .entities
        .iter()
        .chain(scene.npc_templates.iter())
        .find(|e| e.id == id)
}
impl Recipe {
    pub fn validate(&self, scene: &Scene) -> Result<()> {
        if scene.version < 91 {
            return Err(invalid("Navigation generation requires scene91"));
        }
        if self
            .min
            .iter()
            .chain(self.max.iter())
            .any(|v| !v.is_finite() || v.abs() > 10000.)
            || (0..3).any(|i| self.min[i] > self.max[i])
            || !self.cell_size.is_finite()
            || !(0.25..=8.).contains(&self.cell_size)
            || !self.connection_radius.is_finite()
            || !(0.1..=32.).contains(&self.connection_radius)
            || !(64..=1024).contains(&self.max_probes)
            || !(600..=76_800).contains(&self.max_virtual_ticks)
        {
            return Err(invalid(
                "Invalid navigation region, spacing, radius or probe budget",
            ));
        }
        let counts: [usize; 3] = std::array::from_fn(|i| {
            ((self.max[i] - self.min[i]) / self.cell_size).floor() as usize + 1
        });
        if counts[0] * counts[2] > 64
            || (self.include_air && counts.iter().product::<usize>() > 512)
        {
            return Err(invalid(
                "Navigation generation is limited to64columns and512air samples",
            ));
        }
        let Some(e) = source(scene, &self.entity) else {
            return Err(invalid("Navigation source entity/template is absent"));
        };
        if e.character_controller.is_none()
            || e.npc
                .as_ref()
                .is_none_or(|n| n.mobility.is_none_or(|m| !m.walk) || n.local_connections.is_none())
        {
            return Err(invalid(
                "Navigation source requires controller, walk and local connections",
            ));
        }
        if self.include_air
            && e.npc
                .as_ref()
                .unwrap()
                .mobility
                .unwrap()
                .fly_speed
                .is_none()
        {
            return Err(invalid("Air navigation samples require fly capability"));
        }
        Ok(())
    }
    pub fn rebuild(&mut self, scene: &mut Scene) {
        let e = source(scene, &self.entity)
            .expect("validated navigation source")
            .clone();
        self.rebuild_with_source(scene, &e);
    }
    pub(crate) fn rebuild_with_source(&mut self, scene: &mut Scene, e: &Entity) {
        let signature = self.signature(scene, e);
        if self.cached.as_ref().is_some_and(|c| {
            c.signature == signature && c.network == network_signature(&scene.spatial_network)
        }) {
            return;
        }
        let (network, report) = self.generate_for(scene, e);
        let digest = network_signature(&network);
        scene.spatial_network = network;
        self.cached = Some(Cache {
            signature,
            network: digest,
            report,
        });
    }
    pub fn generate(&self, scene: &Scene) -> (Network, Report) {
        let e = source(scene, &self.entity).expect("validated navigation source");
        self.generate_for(scene, e)
    }
    fn generate_for(&self, scene: &Scene, e: &Entity) -> (Network, Report) {
        let world = World::new(scene);
        let mut fixed: Vec<_> = scene
            .entities
            .iter()
            .filter(|e| e.body == Some(BodyKind::Fixed))
            .collect();
        fixed.sort_unstable_by_key(|e| &e.id);
        let skin = e.character_controller.unwrap().offset;
        let half = Vec3::from(e.transform.size) * 0.5;
        let mut net = Network {
            nodes: Vec::with_capacity(64),
            links: Vec::with_capacity(128),
        };
        let mut ground = Vec::with_capacity(64);
        let mut report = Report::default();
        let counts: [usize; 3] = std::array::from_fn(|i| {
            ((self.max[i] - self.min[i]) / self.cell_size).floor() as usize + 1
        });
        let mut phase = if self.seed == 0 {
            [0., 0.]
        } else {
            [
                (self.seed % 4096) as f32 / 4096.,
                (self.seed.rotate_left(17) % 4096) as f32 / 4096.,
            ]
        };
        if counts[0] == 1 {
            phase[0] = 0.;
        }
        if counts[2] == 1 {
            phase[1] = 0.;
        }
        for iz in 0..counts[2] {
            for ix in 0..counts[0] {
                let x = self.min[0] + (ix as f32 + phase[0]) * self.cell_size;
                let z = self.min[2] + (iz as f32 + phase[1]) * self.cell_size;
                if x > self.max[0] || z > self.max[2] {
                    continue;
                }
                report.columns += 1;
                let mut heights = [f32::NEG_INFINITY; 8];
                let mut height_count = 0;
                for &o in &fixed {
                    let volume = Volume::entity(o, o.transform.position.into());
                    let Some(mut y) = volume.top(x, z) else {
                        continue;
                    };
                    for dx in [-half.x, half.x] {
                        for dz in [-half.z, half.z] {
                            if let Some(corner) = volume.top(x + dx, z + dz) {
                                y = y.max(corner);
                            }
                        }
                    }
                    if y < self.min[1]
                        || y > self.max[1]
                        || heights[..height_count]
                            .iter()
                            .any(|h| (y - *h).abs() < 0.001)
                    {
                        continue;
                    }
                    if height_count < 8 {
                        heights[height_count] = y;
                        height_count += 1;
                    } else {
                        report.limited = true;
                        let low = heights
                            .iter()
                            .enumerate()
                            .min_by(|a, b| a.1.total_cmp(b.1))
                            .unwrap()
                            .0;
                        if y > heights[low] {
                            heights[low] = y;
                        }
                    }
                }
                heights[..height_count].sort_unstable_by(|a, b| b.total_cmp(a));
                for &y in &heights[..height_count] {
                    report.node_candidates = report.node_candidates.saturating_add(1);
                    if net.nodes.len() == 64 || report.probes >= self.max_probes {
                        report.limited = true;
                        continue;
                    }
                    let p = [x, y + skin, z];
                    report.probes += 1;
                    if world.trace(e, p, p).status == Status::Arrived {
                        push_node(&mut net, &mut ground, [x, y, z], true);
                    }
                }
                if self.include_air {
                    for iy in 0..counts[1] {
                        let y = self.min[1] + iy as f32 * self.cell_size;
                        if net
                            .nodes
                            .iter()
                            .any(|p| Vec3::from(*p).distance_squared(Vec3::new(x, y, z)) < 0.0001)
                        {
                            continue;
                        }
                        report.node_candidates = report.node_candidates.saturating_add(1);
                        if net.nodes.len() == 64 || report.probes >= self.max_probes {
                            report.limited = true;
                            continue;
                        }
                        report.probes += 1;
                        if world.free(e, [x, y + skin, z]) {
                            push_node(&mut net, &mut ground, [x, y, z], false);
                        }
                    }
                }
            }
        }
        // Bounded authoring allocation only, never a per-tick candidate list.
        let mut pairs = Vec::with_capacity(net.nodes.len() * net.nodes.len().saturating_sub(1));
        for (a, p) in net.nodes.iter().enumerate() {
            for (b, q) in net.nodes.iter().enumerate() {
                let distance = Vec3::from(*p).distance_squared(Vec3::from(*q));
                if a != b
                    && distance >= 0.01
                    && distance <= self.connection_radius * self.connection_radius
                {
                    pairs.push((distance, a, b));
                }
            }
        }
        pairs.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        for (_, a, b) in pairs {
            if net.links.len() == 128
                || report.probes >= self.max_probes
                || report.virtual_ticks >= self.max_virtual_ticks
            {
                report.limited = true;
                break;
            }
            let p = net.nodes[a];
            let q = net.nodes[b];
            let mut mode = None;
            if ground[a] && ground[b] {
                report.probes += 1;
                let r = world.trace_limited(
                    e,
                    (Vec3::from(p) + Vec3::Y * skin).into(),
                    (Vec3::from(q) + Vec3::Y * skin).into(),
                    (self.max_virtual_ticks - report.virtual_ticks).min(MAX_TICKS as u32) as u16,
                );
                report.virtual_ticks += u32::from(r.ticks);
                report.limited |= r.status == Status::BudgetExceeded;
                if r.status == Status::Arrived {
                    mode = Some(Mode::Walk);
                }
            }
            if mode.is_none() {
                for candidate in [Mode::Jump, Mode::Glide, Mode::Fly] {
                    if report.probes >= self.max_probes {
                        report.limited = true;
                        break;
                    }
                    report.probes += 1;
                    if crate::mobility::admit_motion(
                        scene,
                        e,
                        Link {
                            from: a as u8,
                            to: b as u8,
                            mode: candidate,
                        },
                        p.into(),
                        q.into(),
                    )
                    .is_some()
                    {
                        mode = Some(candidate);
                        break;
                    }
                }
            }
            if let Some(mode) = mode {
                net.links.push(Link {
                    from: a as u8,
                    to: b as u8,
                    mode,
                });
            }
        }
        report.nodes = net.nodes.len() as u16;
        report.links = net.links.len() as u16;
        (net, report)
    }
    fn signature(&self, scene: &Scene, e: &Entity) -> u64 {
        let mut h = Hash::new();
        h.bytes(self.entity.as_bytes());
        h.floats(self.min);
        h.floats(self.max);
        h.floats([self.cell_size, self.connection_radius]);
        h.u64(self.seed);
        h.u64(self.include_air as u64);
        h.u64(self.max_probes as u64);
        h.u64(self.max_virtual_ticks as u64);
        h.floats(e.transform.size);
        let n = e.npc.as_ref().unwrap();
        h.floats([n.speed]);
        let c = e.character_controller.unwrap();
        h.floats([
            c.max_slope_degrees,
            c.slide_slope_degrees,
            c.snap_distance,
            c.offset,
            c.gravity,
            c.max_fall_speed,
            c.step.map_or(-1., |s| s.max_height),
            c.step.map_or(-1., |s| s.min_width),
        ]);
        let m = n.mobility.unwrap();
        h.floats([
            m.jump_speed.unwrap_or(-1.),
            m.fly_speed.unwrap_or(-1.),
            m.glide.map_or(-1., |g| g.speed),
            m.glide.map_or(-1., |g| g.sink_speed),
        ]);
        let mut fixed: Vec<_> = scene
            .entities
            .iter()
            .filter(|e| e.body == Some(BodyKind::Fixed))
            .collect();
        fixed.sort_unstable_by_key(|e| &e.id);
        for o in fixed {
            h.bytes(o.id.as_bytes());
            h.floats(o.transform.position);
            h.floats(o.transform.size);
            match o.geometry {
                crate::geometry::Geometry::Box => h.u64(0),
                crate::geometry::Geometry::Wedge { rise } => {
                    let (a, s) = rise.axis_sign();
                    h.u64(1 + a as u64);
                    h.floats([s]);
                }
            }
        }
        h.0
    }
}
fn push_node(net: &mut Network, ground: &mut Vec<bool>, p: [f32; 3], supported: bool) {
    if !net
        .nodes
        .iter()
        .any(|n| Vec3::from(*n).distance_squared(p.into()) < 0.0001)
    {
        net.nodes.push(p);
        ground.push(supported);
    }
}
struct Hash(u64);
impl Hash {
    fn new() -> Self {
        Self(14695981039346656037)
    }
    fn bytes(&mut self, b: &[u8]) {
        for v in b {
            self.0 = (self.0 ^ *v as u64).wrapping_mul(1099511628211);
        }
    }
    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    fn floats<const N: usize>(&mut self, v: [f32; N]) {
        for f in v {
            self.bytes(&f.to_bits().to_le_bytes());
        }
    }
}
fn network_signature(net: &Network) -> u64 {
    let mut h = Hash::new();
    for p in &net.nodes {
        h.floats(*p);
    }
    for l in &net.links {
        h.bytes(&[
            l.from,
            l.to,
            match l.mode {
                Mode::Walk => 0,
                Mode::Jump => 1,
                Mode::Glide => 2,
                Mode::Fly => 3,
            },
        ]);
    }
    h.0
}
