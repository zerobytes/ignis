//! Nonlinear material lattice, XPBD distance/attachment constraints and proxy contact.
use super::{
    bake::{Compiled, Frame, normal},
    *,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const MAX_NODES: usize = 1_572_864;
const MAX_PAIRS: usize = 100000;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub gravity: [f64; 3],
    pub wind: [f64; 3],
    pub ground: Option<f64>,
    pub age: f64,
    pub growth_rate: f64,
    pub damping: f64,
    pub iterations: u16,
    pub self_contact: bool,
    pub leaf_contact: bool,
    pub compliance_scale: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            gravity: [0., -9.81, 0.],
            wind: [0.; 3],
            ground: Some(0.),
            age: 100.,
            growth_rate: 0.,
            damping: 0.8,
            iterations: 16,
            self_contact: false,
            leaf_contact: false,
            compliance_scale: 1.,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Start {
        #[serde(default)]
        settings: Settings,
    },
    Step {
        ticks: u32,
    },
    Play {
        playing: bool,
    },
    Reset,
    Grow {
        age: f64,
    },
    Prune {
        organ: String,
    },
    Inspect,
}
#[derive(Clone)]
struct Node {
    key: String,
    organ: usize,
    position: DVec3,
    velocity: DVec3,
    rest: DVec3,
    mass: f64,
    pinned: bool,
    ground_contact: bool,
}
#[derive(Clone)]
struct Distance {
    a: usize,
    b: usize,
    rest: f64,
    compliance: f64,
    lambda: f64,
}
#[derive(Clone)]
struct Cell {
    nodes: [usize; 4],
    volume: f64,
}
#[derive(Clone)]
struct Attachment {
    child: usize,
    parents: Vec<(usize, f64)>,
    lambda: DVec3,
}
#[derive(Clone)]
struct Capsule {
    organ: usize,
    section: usize,
    a: [usize; 4],
    b: [usize; 4],
    radius: f64,
}
#[derive(Clone, Default)]
pub struct Ledger {
    pub added_mass: f64,
    pub removed_mass: f64,
    pub removed_momentum: DVec3,
    pub initial_mass: f64,
    pub force_impulse: DVec3,
    pub solver_damping_impulse: DVec3,
    pub contacts: u64,
    pub pair_candidates: usize,
}
#[derive(Clone)]
pub struct Session {
    pub recipe: Recipe,
    pub compiled: Arc<Compiled>,
    pub settings: Settings,
    pub playing: bool,
    pub tick: u64,
    nodes: Vec<Node>,
    rings: Vec<Vec<[usize; 4]>>,
    distances: Vec<Distance>,
    cells: Vec<Cell>,
    attachments: Vec<Attachment>,
    capsules: Vec<Capsule>,
    fractions: Vec<f64>,
    removed: BTreeSet<usize>,
    pub ledger: Ledger,
}
fn smooth(x: f64) -> f64 {
    let x = x.clamp(0., 1.);
    x * x * (3. - 2. * x)
}
fn center(nodes: &[Node], ring: [usize; 4]) -> DVec3 {
    ring.into_iter().map(|i| nodes[i].position).sum::<DVec3>() / 4.
}
fn rest_center(nodes: &[Node], ring: [usize; 4]) -> DVec3 {
    ring.into_iter().map(|i| nodes[i].rest).sum::<DVec3>() / 4.
}
fn inverse(node: &Node) -> f64 {
    if node.pinned || node.mass <= 1e-18 {
        0.
    } else {
        1. / node.mass
    }
}
impl Settings {
    fn validate(&self) -> Result<()> {
        if self
            .gravity
            .iter()
            .chain(&self.wind)
            .any(|x| !x.is_finite() || x.abs() > 100.)
            || self
                .ground
                .is_some_and(|x| !x.is_finite() || x.abs() > 10000.)
            || !self.age.is_finite()
            || self.age < 0.
            || self.age > 100000.
            || !self.growth_rate.is_finite()
            || !(0. ..=10.).contains(&self.growth_rate)
            || !self.damping.is_finite()
            || !(0. ..=100.).contains(&self.damping)
            || !(4..=128).contains(&self.iterations)
            || !self.compliance_scale.is_finite()
            || !(0.01..=100.).contains(&self.compliance_scale)
        {
            return Err(invalid("Invalid botanical physics settings"));
        }
        Ok(())
    }
}
impl Session {
    pub fn new(recipe: Recipe, compiled: Compiled, settings: Settings) -> Result<Self> {
        settings.validate()?;
        let counts = compiled
            .poses
            .iter()
            .map(|p| {
                (if p.width == 0. {
                    recipe.settings.axis_samples as usize
                } else {
                    4
                }) + 1
            })
            .sum::<usize>()
            * 4;
        if counts > MAX_NODES {
            return Err(invalid("Botanical physical node budget exceeded"));
        }
        let mut s = Self {
            recipe,
            compiled: Arc::new(compiled),
            settings,
            playing: false,
            tick: 0,
            nodes: vec![],
            rings: vec![],
            distances: vec![],
            cells: vec![],
            attachments: vec![],
            capsules: vec![],
            fractions: vec![],
            removed: BTreeSet::new(),
            ledger: Ledger::default(),
        };
        let parents: std::collections::BTreeSet<_> =
            s.compiled.poses.iter().filter_map(|p| p.parent).collect();
        for (index, pose) in s.compiled.poses.iter().enumerate() {
            let o = &s.recipe.source.organs[pose.source];
            let steps = if pose.width == 0. {
                s.recipe.settings.axis_samples as usize
            } else {
                4
            };
            let root = super::roots::system_for(&s.recipe.source, &o.id);
            let anchor = root.map_or(0., |r| r.settings.anchored_fraction);
            let mut rings = vec![];
            for i in 0..=steps {
                let mut ring = [0; 4];
                for (k, slot) in ring.iter_mut().enumerate() {
                    *slot = s.nodes.len();
                    s.nodes.push(Node {
                        key: format!("{}@{:016x}/{k}", o.id, (i as f64 / steps as f64).to_bits()),
                        organ: index,
                        position: DVec3::ZERO,
                        velocity: DVec3::ZERO,
                        rest: DVec3::ZERO,
                        mass: 0.,
                        pinned: (pose.parent.is_none() && i == 0)
                            || (root.is_some()
                                && !parents.contains(&index)
                                && anchor > 0.
                                && i as f64 / steps as f64 >= 1. - anchor),
                        ground_contact: root.is_none(),
                    });
                }
                rings.push(ring);
            }
            s.rings.push(rings);
        }
        s.update_rest(false)?;
        for n in &mut s.nodes {
            n.position = n.rest;
        }
        s.rebuild()?;
        s.ledger.initial_mass = s.nodes.iter().map(|n| n.mass).sum();
        Ok(s)
    }
    fn update_rest(&mut self, preserve: bool) -> Result<()> {
        let position = DVec3::from_array(self.recipe.settings.position.map(f64::from));
        let old_masses: Vec<_> = self.nodes.iter().map(|n| n.mass).collect();
        self.fractions.clear();
        let mut origins = vec![];
        for (i, p) in self.compiled.poses.iter().enumerate() {
            let o = &self.recipe.source.organs[p.source];
            let g = if self.removed.contains(&i) {
                0.
            } else {
                smooth((self.settings.age - o.birth) / o.duration)
            };
            self.fractions.push(g);
            let origin = if let Some(parent) = p.parent {
                let old = self.compiled.poses[parent].at(p.insertion);
                origins[parent]
                    + (old.p - self.compiled.poses[parent].samples[0].p) * self.fractions[parent]
                    + (p.samples[0].p - old.p) * self.fractions[parent]
            } else {
                p.samples[0].p + position
            };
            origins.push(origin);
            let steps = self.rings[i].len() - 1;
            for j in 0..=steps {
                let u = j as f64 / steps as f64;
                let frame = p.at(u);
                let c = origin + (frame.p - p.samples[0].p) * g;
                let (ra, rb) = p.radius_at(&self.recipe.source, u);
                let (a, b) = if p.width == 0. {
                    (
                        ra * g / std::f64::consts::SQRT_2,
                        rb * g / std::f64::consts::SQRT_2,
                    )
                } else {
                    let envelope = 1.;
                    (
                        p.width * envelope * g / (2. * std::f64::consts::SQRT_2),
                        rb * g / std::f64::consts::SQRT_2,
                    )
                };
                let area = if super::root_sections::material_section(p, u) {
                    super::root_sections::area_bound(p, &self.recipe.source, u).0
                } else if p.width == 0. {
                    std::f64::consts::PI * ra * rb
                } else {
                    p.width * rb * 2. * (std::f64::consts::PI * u).sin().abs().max(0.01)
                };
                let weight = if j == 0 || j == steps { 0.5 } else { 1. };
                let mass = o.density * area * p.length / steps as f64 * weight * g.powi(3) / 4.;
                for (k, offset) in [frame.n * a, -frame.n * a, frame.b() * b, -frame.b() * b]
                    .into_iter()
                    .enumerate()
                {
                    let index = self.rings[i][j][k];
                    let n = &mut self.nodes[index];
                    let old = old_masses[index];
                    let old_momentum = n.velocity * old;
                    n.rest = c + offset;
                    n.mass = mass;
                    if preserve {
                        if mass > 1e-18 {
                            n.velocity = old_momentum / mass;
                            if old <= 1e-18 {
                                n.position = n.rest;
                            }
                        } else {
                            self.ledger.removed_mass += old;
                            self.ledger.removed_momentum += old_momentum;
                            n.velocity = DVec3::ZERO;
                            n.position = n.rest;
                        }
                        if mass > old {
                            self.ledger.added_mass += mass - old;
                        }
                    }
                    if n.pinned {
                        n.position = n.rest;
                        n.velocity = DVec3::ZERO;
                    }
                }
            }
        }
        Ok(())
    }
    fn rebuild(&mut self) -> Result<()> {
        self.distances.clear();
        self.cells.clear();
        self.attachments.clear();
        self.capsules.clear();

        for (i, pose) in self.compiled.poses.iter().enumerate() {
            let organ = &self.recipe.source.organs[pose.source];
            let mut pairs = BTreeSet::new();
            let steps = self.rings[i].len() - 1;
            let area = if super::root_sections::material_section(pose, 0.5) {
                super::root_sections::area_bound(pose, &self.recipe.source, 0.5).0
            } else if pose.width == 0. {
                std::f64::consts::PI * pose.radius.powi(2)
            } else {
                pose.width * pose.radius * 2.
            };
            for j in 0..=steps {
                let ring = self.rings[i][j];
                for a in 0..4 {
                    for b in a + 1..4 {
                        pairs.insert((ring[a], ring[b], 0u8));
                    }
                }
                if j < steps {
                    let next = self.rings[i][j + 1];
                    for nodes in [
                        [ring[0], ring[2], ring[1], next[0]],
                        [ring[0], ring[1], ring[3], next[1]],
                    ] {
                        let [a, b, c, d] = nodes.map(|k| self.nodes[k].rest);
                        let volume = (b - a).cross(c - a).dot(d - a) / 6.;
                        if volume.abs() > 1e-24 {
                            self.cells.push(Cell { nodes, volume });
                        }
                    }
                    for a in 0..4 {
                        pairs.insert((ring[a], next[a], 1));
                        pairs.insert((ring[a], next[(a + 1) % 4], 2));
                        pairs.insert((ring[a], next[(a + 2) % 4], 2));
                    }
                    let u = (j as f64 + 0.5) / steps as f64;
                    let (ra, rb) = pose.radius_at(&self.recipe.source, u);
                    self.capsules.push(Capsule {
                        organ: i,
                        section: j,
                        a: ring,
                        b: next,
                        radius: if super::root_sections::material_section(pose, u) {
                            super::root_sections::area_bound(pose, &self.recipe.source, u).1
                        } else {
                            ra.max(rb)
                        } * self.fractions[i],
                    });
                }
            }
            for &(a, b, kind) in pairs.iter().filter(|(a, _, _)| self.nodes[*a].organ == i) {
                let rest = self.nodes[a].rest.distance(self.nodes[b].rest);
                let stiffness = if kind == 0 {
                    organ.young * pose.stiffness * area / rest.max(1e-6)
                } else if kind == 1 {
                    organ.young * pose.stiffness * area / (4. * pose.length / steps as f64)
                } else {
                    organ.shear * pose.stiffness * area / (4. * pose.length / steps as f64)
                };
                self.distances.push(Distance {
                    a,
                    b,
                    rest,
                    compliance: self.settings.compliance_scale / stiffness.max(1e-6),
                    lambda: 0.,
                });
            }
            if let Some(parent) = pose.parent {
                let parent_rings = &self.rings[parent];
                let x = pose.insertion * (parent_rings.len() - 1) as f64;
                let j = (x as usize).min(parent_rings.len() - 2);
                let f = x - j as f64;
                let r0 = parent_rings[j];
                let r1 = parent_rings[j + 1];
                let c0 = rest_center(&self.nodes, r0);
                let c1 = rest_center(&self.nodes, r1);
                let c = c0.lerp(c1, f);
                let nv = (self.nodes[r0[0]].rest - self.nodes[r0[1]].rest)
                    .lerp(self.nodes[r1[0]].rest - self.nodes[r1[1]].rest, f);
                let bv = (self.nodes[r0[2]].rest - self.nodes[r0[3]].rest)
                    .lerp(self.nodes[r1[2]].rest - self.nodes[r1[3]].rest, f);
                for child in self.rings[i][0] {
                    let d = self.nodes[child].rest - c;
                    let mut weights = BTreeMap::new();
                    for (ring, mix) in [(r0, 1. - f), (r1, f)] {
                        for node in ring {
                            *weights.entry(node).or_insert(0.) += mix / 4.;
                        }
                    }
                    let basis = glam::DMat3::from_cols(c1 - c0, nv, bv);
                    let coordinates = if basis.determinant().abs() > 1e-30 {
                        basis.inverse() * d
                    } else {
                        DVec3::ZERO
                    };
                    let along = coordinates.x;
                    for node in r0 {
                        *weights.entry(node).or_insert(0.) -= along / 4.;
                    }
                    for node in r1 {
                        *weights.entry(node).or_insert(0.) += along / 4.;
                    }
                    for (ring, mix) in [(r0, 1. - f), (r1, f)] {
                        if nv.length_squared() > 1e-20 {
                            let q = coordinates.y;
                            *weights.entry(ring[0]).or_insert(0.) += mix * q;
                            *weights.entry(ring[1]).or_insert(0.) -= mix * q;
                        }
                        if bv.length_squared() > 1e-20 {
                            let q = coordinates.z;
                            *weights.entry(ring[2]).or_insert(0.) += mix * q;
                            *weights.entry(ring[3]).or_insert(0.) -= mix * q;
                        }
                    }
                    self.attachments.push(Attachment {
                        child,
                        parents: weights.into_iter().collect(),
                        lambda: DVec3::ZERO,
                    });
                }
            }
        }
        if self.distances.len() > 6_000_000 {
            return Err(invalid("Botanical constraint budget exceeded"));
        }
        Ok(())
    }
    fn project_attachments(&mut self) {
        for c in &mut self.attachments {
            if self.nodes[c.child].mass <= 1e-18 {
                continue;
            }
            let target = c
                .parents
                .iter()
                .map(|(i, w)| self.nodes[*i].position * *w)
                .sum::<DVec3>();
            let d = self.nodes[c.child].position - target;
            let wc = inverse(&self.nodes[c.child]);
            let total = wc
                + c.parents
                    .iter()
                    .map(|(i, w)| inverse(&self.nodes[*i]) * w * w)
                    .sum::<f64>();
            if total > 0. {
                let dl = -d / total;
                self.nodes[c.child].position += dl * wc;
                for (i, w) in &c.parents {
                    let inv = inverse(&self.nodes[*i]);
                    self.nodes[*i].position -= dl * (*w * inv);
                }
                c.lambda += dl;
            }
        }
    }
    fn attachment_error(&self) -> f64 {
        self.attachments
            .iter()
            .filter(|c| self.nodes[c.child].mass > 1e-18)
            .map(|c| {
                let target = c
                    .parents
                    .iter()
                    .map(|(i, w)| self.nodes[*i].position * *w)
                    .sum::<DVec3>();
                self.nodes[c.child].position.distance(target)
            })
            .fold(0., f64::max)
    }
    fn close_attachments(&mut self) -> Result<()> {
        if self.attachment_error() < 1e-7 {
            return Ok(());
        }
        // Exact mass-weighted projection of the linear material bindings.
        // Matrix-free preconditioned CG solves (J M^-1 J^T) lambda = J x.
        let active: Vec<_> = self
            .attachments
            .iter()
            .filter(|c| self.nodes[c.child].mass > 1e-18)
            .collect();
        let diagonal: Vec<_> = active
            .iter()
            .map(|c| {
                inverse(&self.nodes[c.child])
                    + c.parents
                        .iter()
                        .map(|(i, w)| inverse(&self.nodes[*i]) * w * w)
                        .sum::<f64>()
            })
            .collect();
        let mut residual: Vec<_> = active
            .iter()
            .map(|c| {
                self.nodes[c.child].position
                    - c.parents
                        .iter()
                        .map(|(i, w)| self.nodes[*i].position * *w)
                        .sum::<DVec3>()
            })
            .collect();
        let mut direction: Vec<_> = residual
            .iter()
            .zip(&diagonal)
            .map(|(r, d)| *r / *d)
            .collect();
        let mut multiplier = vec![DVec3::ZERO; active.len()];
        let mut scratch = vec![DVec3::ZERO; self.nodes.len()];
        let mut product = vec![DVec3::ZERO; active.len()];
        let mut rz = residual
            .iter()
            .zip(&direction)
            .map(|(r, z)| r.dot(*z))
            .sum::<f64>();
        for _ in 0..512 {
            scratch.fill(DVec3::ZERO);
            for (c, p) in active.iter().zip(&direction) {
                scratch[c.child] += *p;
                for (i, w) in &c.parents {
                    scratch[*i] -= *p * *w;
                }
            }
            for (v, n) in scratch.iter_mut().zip(&self.nodes) {
                *v *= inverse(n);
            }
            for (ap, c) in product.iter_mut().zip(&active) {
                *ap = scratch[c.child]
                    - c.parents
                        .iter()
                        .map(|(i, w)| scratch[*i] * *w)
                        .sum::<DVec3>();
            }
            let denominator = direction
                .iter()
                .zip(&product)
                .map(|(p, ap)| p.dot(*ap))
                .sum::<f64>();
            if !denominator.is_finite() || denominator <= 1e-30 {
                break;
            }
            let alpha = rz / denominator;
            for ((lambda, r), (p, ap)) in multiplier
                .iter_mut()
                .zip(&mut residual)
                .zip(direction.iter().zip(&product))
            {
                *lambda += *p * alpha;
                *r -= *ap * alpha;
            }
            if residual.iter().all(|r| r.length() < 1e-8) {
                break;
            }
            let next = residual
                .iter()
                .zip(&diagonal)
                .map(|(r, d)| r.length_squared() / *d)
                .sum::<f64>();
            let beta = next / rz;
            for ((p, r), d) in direction.iter_mut().zip(&residual).zip(&diagonal) {
                *p = *r / *d + *p * beta;
            }
            rz = next;
        }
        scratch.fill(DVec3::ZERO);
        for (c, lambda) in active.iter().zip(&multiplier) {
            scratch[c.child] += *lambda;
            for (i, w) in &c.parents {
                scratch[*i] -= *lambda * *w;
            }
        }
        for (node, correction) in self.nodes.iter_mut().zip(scratch) {
            let w = inverse(node);
            node.position -= correction * w;
        }
        if self
            .nodes
            .iter()
            .any(|n| !n.position.is_finite() || !n.velocity.is_finite())
        {
            return Err(invalid(
                "Nonfinite botanical material projection; previous state preserved",
            ));
        }
        if self.attachment_error() > 1e-4 {
            return Err(invalid(
                "Botanical material bindings did not converge; previous state preserved",
            ));
        }
        Ok(())
    }
    pub fn grow(&mut self, age: f64) -> Result<()> {
        if !age.is_finite() || age < self.settings.age || age > 100000. {
            return Err(invalid(
                "Physical growth time is monotonic; reset to inspect an earlier age",
            ));
        }
        let mut candidate = self.clone();
        candidate.settings.age = age;
        candidate.update_rest(true)?;
        candidate.rebuild()?;
        candidate.close_attachments()?;
        *self = candidate;
        Ok(())
    }
    pub fn prune(&mut self, id: &str) -> Result<()> {
        let root = self
            .compiled
            .poses
            .iter()
            .position(|p| self.recipe.source.organs[p.source].id == id)
            .ok_or_else(|| invalid("Unknown physical organ"))?;
        let mut candidate = self.clone();
        candidate.removed.insert(root);
        for (i, p) in candidate.compiled.poses.iter().enumerate() {
            if p.parent.is_some_and(|p| candidate.removed.contains(&p)) {
                candidate.removed.insert(i);
            }
        }
        candidate.update_rest(true)?;
        candidate.rebuild()?;
        candidate.close_attachments()?;
        *self = candidate;
        Ok(())
    }
    pub fn step(&mut self, ticks: u32) -> Result<()> {
        if !(1..=3600).contains(&ticks) {
            return Err(invalid("Botanical stepping requires1..3600 ticks"));
        }
        let mut candidate = self.clone();
        for _ in 0..ticks {
            candidate.substep(1. / 120.)?;
        }
        *self = candidate;
        Ok(())
    }
    fn substep(&mut self, dt: f64) -> Result<()> {
        if self.settings.growth_rate > 0. {
            self.settings.age = (self.settings.age + self.settings.growth_rate * dt).min(100000.);
            self.update_rest(true)?;
            self.rebuild()?;
        }
        let momentum_before = self
            .nodes
            .iter()
            .map(|n| n.velocity * n.mass)
            .sum::<DVec3>();
        let previous: Vec<_> = self.nodes.iter().map(|n| n.position).collect();
        for n in &mut self.nodes {
            if n.mass <= 1e-18 {
                continue;
            }
            if n.pinned {
                n.position = n.rest;
                continue;
            }
            let wind = DVec3::from_array(self.settings.wind);
            let relative = wind - n.velocity;
            let drag = relative * (relative.length() * 0.04); // Effective bounded area/mass acceleration.
            n.velocity += (DVec3::from_array(self.settings.gravity) + drag) * dt;
            n.position += n.velocity * dt;
        }
        let predicted_momentum = self
            .nodes
            .iter()
            .map(|n| n.velocity * n.mass)
            .sum::<DVec3>();
        self.ledger.force_impulse += predicted_momentum - momentum_before;
        for c in &mut self.distances {
            c.lambda = 0.;
        }
        for c in &mut self.attachments {
            c.lambda = DVec3::ZERO;
        }
        let contacts = if self.settings.self_contact {
            self.contact_pairs()?
        } else {
            vec![]
        };
        self.ledger.pair_candidates = contacts.len();
        for _ in 0..self.settings.iterations {
            for c in &mut self.distances {
                if self.nodes[c.a].mass <= 1e-18 || self.nodes[c.b].mass <= 1e-18 {
                    continue;
                }
                let d = self.nodes[c.a].position - self.nodes[c.b].position;
                let length = d.length();
                if length < 1e-12 {
                    continue;
                }
                let a = inverse(&self.nodes[c.a]);
                let b = inverse(&self.nodes[c.b]);
                let alpha = c.compliance / (dt * dt);
                if a + b == 0. {
                    continue;
                }
                let dl = (-(length - c.rest) - alpha * c.lambda) / (a + b + alpha);
                c.lambda += dl;
                self.nodes[c.a].position += d * (a * dl / length);
                self.nodes[c.b].position -= d * (b * dl / length);
            }
            self.project_attachments();
            if let Some(y) = self.settings.ground {
                for n in &mut self.nodes {
                    if n.ground_contact && !n.pinned && n.mass > 1e-18 && n.position.y < y {
                        n.position.y = y;
                        self.ledger.contacts = self.ledger.contacts.saturating_add(1);
                    }
                }
            }
            for &(a, b) in &contacts {
                self.project_contact(a, b);
            }
            // Signed-cell barrier prevents mirror inversion without an angular clamp.
            for cell in &self.cells {
                if self.nodes[cell.nodes[0]].mass <= 1e-18 {
                    continue;
                }
                let [a, b, c, d] = cell.nodes.map(|k| self.nodes[k].position);
                let sign = cell.volume.signum();
                let volume = (b - a).cross(c - a).dot(d - a) / 6.;
                let error = volume * sign - cell.volume.abs() * 0.15;
                if error >= 0. {
                    continue;
                }
                let gb = (c - a).cross(d - a) * (sign / 6.);
                let gc = (d - a).cross(b - a) * (sign / 6.);
                let gd = (b - a).cross(c - a) * (sign / 6.);
                let gradients = [-gb - gc - gd, gb, gc, gd];
                let denom = cell
                    .nodes
                    .iter()
                    .zip(gradients)
                    .map(|(k, g)| inverse(&self.nodes[*k]) * g.length_squared())
                    .sum::<f64>();
                if denom > 1e-30 {
                    for (k, g) in cell.nodes.into_iter().zip(gradients) {
                        let w = inverse(&self.nodes[k]);
                        self.nodes[k].position -= g * (error * w / denom);
                    }
                }
            }
        }
        self.close_attachments()?;
        for (i, n) in self.nodes.iter_mut().enumerate() {
            if n.mass <= 1e-18 {
                continue;
            }
            if n.ground_contact
                && !n.pinned
                && let Some(y) = self.settings.ground
            {
                n.position.y = n.position.y.max(y);
            }
            n.velocity = if n.pinned {
                DVec3::ZERO
            } else {
                (n.position - previous[i]) / dt * (-self.settings.damping * dt).exp()
            };
            if !n.position.is_finite()
                || !n.velocity.is_finite()
                || n.position.abs().max_element() > 10000.
                || n.velocity.length() > 1000.
            {
                return Err(invalid(
                    "Botanical nonlinear solve diverged; previous state preserved",
                ));
            }
        }
        let final_momentum = self
            .nodes
            .iter()
            .map(|n| n.velocity * n.mass)
            .sum::<DVec3>();
        self.ledger.solver_damping_impulse += final_momentum - predicted_momentum;
        self.tick = self
            .tick
            .checked_add(1)
            .ok_or_else(|| invalid("Botanical tick exhausted"))?;
        Ok(())
    }
    fn contact_pairs(&self) -> Result<Vec<(usize, usize)>> {
        let cell = self
            .capsules
            .iter()
            .map(|c| c.radius * 2.)
            .fold(0.05, f64::max)
            .max(0.05);
        let mut grid: BTreeMap<(i32, i32, i32), Vec<usize>> = BTreeMap::new();
        let mut work = 0usize;
        for (i, c) in self.capsules.iter().enumerate() {
            if self.fractions[c.organ] <= 0.
                || (!self.settings.leaf_contact && self.compiled.poses[c.organ].width > 0.)
            {
                continue;
            }
            let a = center(&self.nodes, c.a);
            let b = center(&self.nodes, c.b);
            let lo = ((a.min(b) - DVec3::splat(c.radius)) / cell)
                .floor()
                .as_ivec3();
            let hi = ((a.max(b) + DVec3::splat(c.radius)) / cell)
                .floor()
                .as_ivec3();
            for x in lo.x..=hi.x {
                for y in lo.y..=hi.y {
                    for z in lo.z..=hi.z {
                        work += 1;
                        if work > 1_000_000 {
                            return Err(invalid(
                                "Botanical contact broadphase work budget exceeded",
                            ));
                        }
                        grid.entry((x, y, z)).or_default().push(i);
                    }
                }
            }
        }
        let mut pairs = BTreeSet::new();
        for values in grid.values() {
            for (k, &a) in values.iter().enumerate() {
                for &b in &values[k + 1..] {
                    let ca = &self.capsules[a];
                    let cb = &self.capsules[b];
                    if ca.organ == cb.organ && ca.section.abs_diff(cb.section) <= 2 {
                        continue;
                    }
                    let attached = |child: &Capsule, parent: &Capsule| {
                        let pose = &self.compiled.poses[child.organ];
                        let station =
                            (pose.insertion * (self.rings[parent.organ].len() - 1) as f64) as usize;
                        pose.parent == Some(parent.organ)
                            && child.section <= 1
                            && parent.section.abs_diff(station) <= 1
                    };
                    if attached(ca, cb) || attached(cb, ca) {
                        continue;
                    }
                    pairs.insert((a.min(b), a.max(b)));
                    if pairs.len() > MAX_PAIRS {
                        return Err(invalid("Botanical contact pair budget exceeded"));
                    }
                }
            }
        }
        Ok(pairs.into_iter().collect())
    }
    fn project_contact(&mut self, a: usize, b: usize) {
        let a = &self.capsules[a];
        let b = &self.capsules[b];
        let aa = center(&self.nodes, a.a);
        let ab = center(&self.nodes, a.b);
        let ba = center(&self.nodes, b.a);
        let bb = center(&self.nodes, b.b);
        let (s, t) = closest(aa, ab, ba, bb);
        let delta = aa.lerp(ab, s) - ba.lerp(bb, t);
        let distance = delta.length();
        let penetration = a.radius + b.radius - distance;
        if penetration <= 0. {
            return;
        }
        let normal = if distance > 1e-10 {
            delta / distance
        } else {
            DVec3::X
        };
        let mut weights = BTreeMap::new();
        for (ring, w) in [
            (a.a, (1. - s) / 4.),
            (a.b, s / 4.),
            (b.a, -(1. - t) / 4.),
            (b.b, -t / 4.),
        ] {
            for i in ring {
                *weights.entry(i).or_insert(0.) += w;
            }
        }
        let denominator = weights
            .iter()
            .map(|(i, w)| inverse(&self.nodes[*i]) * w * w)
            .sum::<f64>();
        if denominator > 1e-18 {
            for (i, w) in weights {
                let inv = inverse(&self.nodes[i]);
                self.nodes[i].position += normal * (penetration * w * inv / denominator);
            }
            self.ledger.contacts = self.ledger.contacts.saturating_add(1);
        }
    }
    pub fn frame(&self, organ: usize, u: f64) -> (Frame, f64, f64) {
        let rings = &self.rings[organ];
        let x = u.clamp(0., 1.) * (rings.len() - 1) as f64;
        let i = (x as usize).min(rings.len() - 2);
        let f = x - i as f64;
        let frame = |ring: [usize; 4], i: usize, rest: bool| {
            let point = |k: usize| {
                if rest {
                    self.nodes[k].rest
                } else {
                    self.nodes[k].position
                }
            };
            let center = |r: [usize; 4]| r.into_iter().map(point).sum::<DVec3>() / 4.;
            let p = center(ring);
            let d = point(ring[0]) - point(ring[1]);
            let next = if i + 1 < rings.len() {
                center(rings[i + 1]) - p
            } else {
                p - center(rings[i - 1])
            };
            let fallback = self.compiled.poses[organ].at(i as f64 / (rings.len() - 1) as f64);
            let t = if next.length_squared() > 1e-20 {
                next.normalize()
            } else {
                fallback.t
            };
            Frame {
                p,
                t,
                n: normal(
                    t,
                    if d.length_squared() > 1e-20 {
                        d
                    } else {
                        fallback.n
                    },
                ),
            }
        };
        let interpolate = |a: Frame, b: Frame| {
            let t = a.t.lerp(b.t, f).normalize_or_zero();
            let t = if t.length_squared() > 0.5 { t } else { a.t };
            Frame {
                p: a.p.lerp(b.p, f),
                t,
                n: normal(t, a.n.lerp(b.n, f)),
            }
        };
        let current = interpolate(frame(rings[i], i, false), frame(rings[i + 1], i + 1, false));
        let rest = interpolate(frame(rings[i], i, true), frame(rings[i + 1], i + 1, true));
        let rotation = glam::DMat3::from_cols(current.t, current.n, current.b())
            * glam::DMat3::from_cols(rest.t, rest.n, rest.b()).transpose();
        let mature = &self.compiled.poses[organ];
        let target = mature.at(u);
        let rest_target = rest_center(&self.nodes, rings[0])
            + (target.p - mature.samples[0].p) * self.fractions[organ];
        let t = rotation * target.t;
        let result = Frame {
            p: current.p + rotation * (rest_target - rest.p)
                - DVec3::from_array(self.recipe.settings.position.map(f64::from)),
            t,
            n: normal(t, rotation * target.n),
        };
        let span = |r: [usize; 4], a: usize, b: usize, rest: bool| {
            let point = |i: usize| {
                if rest {
                    self.nodes[i].rest
                } else {
                    self.nodes[i].position
                }
            };
            point(r[a]).distance(point(r[b])) / 2. * std::f64::consts::SQRT_2
        };
        let current_a =
            span(rings[i], 0, 1, false) * (1. - f) + span(rings[i + 1], 0, 1, false) * f;
        let current_b =
            span(rings[i], 2, 3, false) * (1. - f) + span(rings[i + 1], 2, 3, false) * f;
        let rest_a = span(rings[i], 0, 1, true) * (1. - f) + span(rings[i + 1], 0, 1, true) * f;
        let rest_b = span(rings[i], 2, 3, true) * (1. - f) + span(rings[i + 1], 2, 3, true) * f;
        (
            result,
            current_a / rest_a.max(1e-12) * self.fractions[organ],
            current_b / rest_b.max(1e-12) * self.fractions[organ],
        )
    }
    pub fn document(&self) -> meshes::Document {
        let mut result = self.compiled.document.clone();
        for part in 0..2 {
            if self.compiled.vertices[part].is_empty() {
                continue;
            }
            let source = &self.compiled.document.assets[part].geometry;
            let mut geometry = (**source).clone();
            for (p, v) in geometry
                .positions
                .iter_mut()
                .zip(&self.compiled.vertices[part])
            {
                let (f, a, b) = self.frame(v.organ, v.u);
                *p = (f.p
                    + f.t * (v.local.x * self.fractions[v.organ])
                    + f.n * (v.local.y * a)
                    + f.b() * (v.local.z * b))
                    .as_vec3()
                    .to_array();
            }
            geometry.triangles.retain(|t| {
                let [a, b, c] = t.map(|i| glam::Vec3::from_array(geometry.positions[i as usize]));
                (b - a).cross(c - a).length_squared() > 1e-20
            });
            if geometry.triangles.is_empty() {
                geometry = meshes::Geometry {
                    positions: vec![[0.; 3], [0.00001, 0., 0.], [0., 0.00001, 0.]],
                    triangles: vec![[0, 1, 2]],
                    material_coordinates: None,
                };
            }
            result.assets[part].geometry = Arc::new(geometry);
        }
        result
    }
    pub fn mass(&self) -> f64 {
        self.nodes.iter().map(|n| n.mass).sum()
    }
    pub fn inspect(&self) -> Value {
        let mut max_angle = 0f64;
        let mut attachment_error = 0f64;
        for (i, rings) in self.rings.iter().enumerate() {
            if self.fractions[i] <= 0. {
                continue;
            }
            for (j, r) in rings.windows(2).enumerate() {
                let t = (center(&self.nodes, r[1]) - center(&self.nodes, r[0])).normalize_or_zero();
                let rest = self.compiled.poses[i]
                    .at((j as f64 + 0.5) / (rings.len() - 1) as f64)
                    .t;
                max_angle = max_angle.max(t.dot(rest).clamp(-1., 1.).acos());
            }
        }
        for c in &self.attachments {
            if self.nodes[c.child].mass > 1e-18 {
                let target = c
                    .parents
                    .iter()
                    .map(|(i, w)| self.nodes[*i].position * *w)
                    .sum::<DVec3>();
                attachment_error =
                    attachment_error.max(self.nodes[c.child].position.distance(target));
            }
        }
        let mut contact_overlap = 0f64;
        let mut contact_probe_error = None;
        let pairs = if self.settings.self_contact {
            match self.contact_pairs() {
                Ok(p) => Some(p),
                Err(e) => {
                    contact_probe_error = Some(e.message);
                    None
                }
            }
        } else {
            None
        };
        let contact_probe_complete = pairs.is_some();
        if let Some(pairs) = pairs {
            for (a, b) in pairs {
                let a = &self.capsules[a];
                let b = &self.capsules[b];
                let aa = center(&self.nodes, a.a);
                let ab = center(&self.nodes, a.b);
                let ba = center(&self.nodes, b.a);
                let bb = center(&self.nodes, b.b);
                let (s, t) = closest(aa, ab, ba, bb);
                contact_overlap = contact_overlap
                    .max(a.radius + b.radius - aa.lerp(ab, s).distance(ba.lerp(bb, t)));
            }
        }
        let mass = self.mass();
        let p = self
            .nodes
            .iter()
            .map(|n| n.velocity * n.mass)
            .sum::<DVec3>();
        json!({"self_contact_enabled":self.settings.self_contact,"contact_probe_complete":contact_probe_complete,"contact_probe_error":contact_probe_error,"max_proxy_overlap_m":contact_overlap,"max_frame_rotation_rad":max_angle,"attachment_error_m":attachment_error,"root_anchor_nodes":self.nodes.iter().filter(|n|!n.ground_contact&&n.pinned).count(),"min_node_y":self.nodes.iter().filter(|n|n.mass>1e-18).map(|n|n.position.y).fold(f64::INFINITY,f64::min),"tick":self.tick,"playing":self.playing,"age":self.settings.age,"nodes":self.nodes.len(),"node_identity":self.nodes.iter().flat_map(|n|n.key.bytes()).fold(0xcbf29ce484222325u64,|h,b|(h^b as u64).wrapping_mul(0x100000001b3)),"constraints":self.distances.len()+self.attachments.len()+self.cells.len(),"mass_kg":mass,"momentum":p.to_array(),"initial_mass_kg":self.ledger.initial_mass,"force_impulse":self.ledger.force_impulse.to_array(),"solver_damping_impulse":self.ledger.solver_damping_impulse.to_array(),"added_mass_kg":self.ledger.added_mass,"removed_mass_kg":self.ledger.removed_mass,"removed_momentum":self.ledger.removed_momentum.to_array(),"contacts":self.ledger.contacts,"contact_candidates":self.ledger.pair_candidates,"removed_organs":self.removed.len(),"model":"nonlinear_xpbd_material_lattice","contact":"capsule_proxies_and_explicit_ground","angle_limit":null})
    }
}
fn closest(p: DVec3, q: DVec3, r: DVec3, s: DVec3) -> (f64, f64) {
    let d1 = q - p;
    let d2 = s - r;
    let v = p - r;
    let a = d1.length_squared();
    let e = d2.length_squared();
    let b = d1.dot(d2);
    let c = d1.dot(v);
    let f = d2.dot(v);
    if a < 1e-18 {
        return (0., if e > 1e-18 { (f / e).clamp(0., 1.) } else { 0. });
    }
    if e < 1e-18 {
        return ((-c / a).clamp(0., 1.), 0.);
    }
    let denom = a * e - b * b;
    let mut x = if denom.abs() > 1e-18 {
        ((b * f - c * e) / denom).clamp(0., 1.)
    } else {
        0.
    };
    let mut y = (b * x + f) / e;
    if y < 0. {
        y = 0.;
        x = (-c / a).clamp(0., 1.);
    } else if y > 1. {
        y = 1.;
        x = ((b - c) / a).clamp(0., 1.);
    }
    (x, y)
}
