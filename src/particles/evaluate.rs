use super::*;
use glam::{DQuat, DVec3, Vec3};
use serde_json::json;
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub color: [f32; 4],
    pub size: f32,
    pub angle: f32,
    pub aspect: f32,
    pub softness: f32,
    pub align_velocity: bool,
    pub age: f32,
    pub material: u32,
}
#[derive(Debug, Default, Clone, Serialize)]
pub struct Stats {
    pub tick: u64,
    pub evaluated: u64,
    pub candidates: u64,
    pub alive: usize,
    pub expired: u64,
    pub budget_drops: u64,
    pub projection_drops: u64,
    pub work_limited: bool,
    pub rebuilds: u64,
}
#[derive(Default)]
pub struct Frame {
    document: Option<Arc<Document>>,
    tick: Option<u64>,
    pub samples: Vec<Sample>,
    pub materials: Vec<Material>,
    pub stats: Stats,
    births: Vec<Birth>,
    compiled: Vec<Coefficients>,
}
fn hash(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
fn name(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325, |a, b| {
        (a ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}
fn random(seed: u64, channel: u64) -> f64 {
    (hash(seed ^ channel.wrapping_mul(0x9e3779b97f4a7c15)) >> 11) as f64 * (1. / 9007199254740992.)
}
fn range(r: [f32; 2], u: f64) -> f64 {
    f64::from(r[0]) + (f64::from(r[1]) - f64::from(r[0])) * u
}
/// Closed-form x/v for dv/dt=a-drag*v. Stable as drag approaches zero.
pub fn trajectory(
    position: [f64; 3],
    velocity: [f64; 3],
    acceleration: [f64; 3],
    drag: f64,
    time: f64,
) -> ([f64; 3], [f64; 3]) {
    let p = DVec3::from(position);
    let v = DVec3::from(velocity);
    let a = DVec3::from(acceleration);
    if drag == 0. {
        return (
            (p + v * time + a * (0.5 * time * time)).to_array(),
            (v + a * time).to_array(),
        );
    }
    let z = drag * time;
    let decay = (-z).exp();
    let (first, second) = if z.abs() < 0.0001 {
        (
            time * (1. - z / 2. + z * z / 6. - z * z * z / 24.),
            time * time * (0.5 - z / 6. + z * z / 24. - z * z * z / 120.),
        )
    } else {
        let f = -(-z).exp_m1() / drag;
        (f, (time - f) / drag)
    };
    (
        (p + v * first + a * second).to_array(),
        (v * decay + a * first).to_array(),
    )
}
fn curve_time<'a>(
    time: f32,
    keys: impl Iterator<Item = (f32, &'a [f32])>,
) -> Option<(&'a [f32], &'a [f32], f32)> {
    let mut previous: Option<(f32, &[f32])> = None;
    for (t, v) in keys {
        if let Some((pt, pv)) = previous
            && time <= t
        {
            return Some((pv, v, ((time - pt) / (t - pt)).clamp(0., 1.)));
        }
        previous = Some((t, v));
    }
    None
}
#[derive(Clone, Copy)]
struct Birth {
    tick: u64,
    key: u64,
}
#[derive(Clone, Copy)]
struct Coefficients {
    offset: DVec3,
    acceleration: DVec3,
    drag: f64,
    rotation: DQuat,
    seed: u64,
    axis: DVec3,
    side: DVec3,
    other: DVec3,
    cone_cos: f64,
}
impl Coefficients {
    fn new(s: &Source, p: &Placement, e: &Emitter) -> Self {
        let mut offset = DVec3::ZERO;
        let mut acceleration = DVec3::ZERO;
        let mut drag = 0.;
        for m in &e.modules {
            match m {
                Module::Offset { value } => offset += DVec3::from(value.map(f64::from)),
                Module::Acceleration { value } => acceleration += DVec3::from(value.map(f64::from)),
                Module::Drag { coefficient } => drag += f64::from(*coefficient),
                _ => {}
            }
        }
        let rotation = DQuat::from_xyzw(
            f64::from(p.rotation[0]),
            f64::from(p.rotation[1]),
            f64::from(p.rotation[2]),
            f64::from(p.rotation[3]),
        )
        .normalize();
        let seed = hash(
            hash(u64::from(s.seed) ^ name(&s.id))
                ^ name(&p.id).rotate_left(17)
                ^ u64::from(p.seed)
                ^ name(&e.id).rotate_left(33)
                ^ u64::from(e.seed),
        );
        let axis = DVec3::from(e.direction.map(f64::from)).normalize();
        let side = axis.any_orthonormal_vector();
        let other = axis.cross(side);
        Self {
            offset,
            acceleration,
            drag,
            rotation,
            seed,
            axis,
            side,
            other,
            cone_cos: f64::from(e.spread_degrees).to_radians().cos(),
        }
    }
}
impl Frame {
    pub(crate) fn cache_key(&self) -> Option<(&Arc<Document>, u64)> {
        Some((self.document.as_ref()?, self.tick?))
    }
    pub fn clear(&mut self) {
        self.document = None;
        self.tick = None;
        self.samples.clear();
        self.materials.clear();
        self.compiled.clear();
        self.births.clear();
        self.stats = Stats::default();
    }
    pub fn update(&mut self, document: &Arc<Document>, tick: u64) {
        if self.tick == Some(tick)
            && self
                .document
                .as_ref()
                .is_some_and(|d| Arc::ptr_eq(d, document))
        {
            return;
        }
        let changed = self
            .document
            .as_ref()
            .is_none_or(|d| !Arc::ptr_eq(d, document));
        self.stats = Stats {
            tick,
            rebuilds: self.stats.rebuilds + 1,
            ..Default::default()
        };
        self.samples.clear();
        if changed {
            self.materials.clear();
            self.compiled.clear();
            for p in &document.instances {
                let s = document
                    .sources
                    .iter()
                    .find(|s| s.id == p.source)
                    .expect("validated effect source");
                for e in &s.emitters {
                    self.compiled.push(Coefficients::new(s, p, e));
                    let m = if let Some(recipe) = &e.material.procedural {
                        recipe
                            .instance(&format!("{}/{}", p.id, e.id))
                            .map(|r| {
                                let mut m = e.material.clone();
                                m.procedural = Some(Arc::new(r));
                                m
                            })
                            .unwrap_or_else(|| e.material.clone())
                    } else {
                        e.material.clone()
                    };
                    self.materials.push(m);
                }
            }
        }
        let work_limit = u64::from(document.max_particles) * 16;
        let mut material = 0;
        for p in &document.instances {
            let s = document.sources.iter().find(|s| s.id == p.source).unwrap();
            for e in &s.emitters {
                let slot = material;
                let c = self.compiled[slot as usize];
                material += 1;
                if !p.enabled || tick < p.start_tick {
                    continue;
                }
                let time = tick - p.start_tick;
                if time < u64::from(e.delay_ticks) {
                    continue;
                }
                let max_life = u64::from(e.lifetime_ticks[1]);
                let lo = time.saturating_sub(max_life - 1);
                self.births.clear();
                let births = &mut self.births;
                let rate = (f64::from(e.rate_per_second) * 1000.).round() as u128;
                let end = if s.looping {
                    time
                } else {
                    time.min(u64::from(s.duration_ticks) - 1)
                };
                let start = u64::from(e.delay_ticks);
                let available = (work_limit - self.stats.candidates)
                    .min(u64::from(e.max_particles) * 16) as usize;
                if end >= start && rate > 0 && available > 0 {
                    let high = u128::from(end - start) * rate / 60000;
                    let low = if lo > start {
                        u128::from(lo - start - 1) * rate / 60000 + 1
                    } else {
                        1
                    };
                    if high >= low {
                        let count = (high - low + 1).min(available as u128);
                        self.stats.work_limited |= high - low + 1 > count;
                        self.stats.budget_drops = self.stats.budget_drops.saturating_add(
                            (high - low + 1 - count).min(u128::from(u64::MAX)) as u64,
                        );
                        for n in 0..count {
                            let ordinal = high - n;
                            let born = (ordinal * 60000).div_ceil(rate) as u64 + start;
                            births.push(Birth {
                                tick: born,
                                key: hash(ordinal as u64)
                                    ^ hash((ordinal >> 64) as u64)
                                    ^ 0xA341316C,
                            });
                        }
                    }
                }
                if e.burst > 0 && available > 0 {
                    let period = u64::from(s.duration_ticks);
                    let high = (time - start) / period;
                    let high = if s.looping { high } else { 0 };
                    let low = if lo > start {
                        (lo - start).div_ceil(period)
                    } else {
                        0
                    };
                    if high >= low {
                        let mut full = false;
                        for cycle in (low..=high).rev() {
                            let born = cycle * period + start;
                            if born > time {
                                continue;
                            }
                            for n in 0..e.burst {
                                if births.len() >= available {
                                    self.stats.work_limited = true;
                                    full = true;
                                    break;
                                }
                                births.push(Birth {
                                    tick: born,
                                    key: hash(hash(cycle) ^ u64::from(n)) ^ 0xC8013EA4,
                                });
                            }
                            if full {
                                break;
                            }
                        }
                    }
                }
                births.sort_unstable_by_key(|b| (std::cmp::Reverse(b.tick), b.key));
                let mut accepted = 0;
                for b in births.drain(..) {
                    if self.stats.candidates >= work_limit {
                        self.stats.work_limited = true;
                        break;
                    }
                    self.stats.candidates += 1;
                    if accepted >= e.max_particles
                        || self.samples.len() >= document.max_particles as usize
                    {
                        self.stats.budget_drops += 1;
                        continue;
                    }
                    self.stats.evaluated += 1;
                    let key = hash(c.seed ^ b.key);
                    let life = e.lifetime_ticks[0]
                        + (hash(key) % u64::from(e.lifetime_ticks[1] - e.lifetime_ticks[0] + 1))
                            as u32;
                    let age = time - b.tick;
                    if age >= u64::from(life) {
                        self.stats.expired += 1;
                        continue;
                    }
                    let t = age as f64 / 60.;
                    let fraction = age as f32 / life as f32;
                    let az = random(key, 1) * std::f64::consts::TAU;
                    let z = 1. - 2. * random(key, 2);
                    let radius = (1. - z * z).sqrt();
                    let unit = DVec3::new(radius * az.cos(), z, radius * az.sin());
                    let spawn = match e.shape {
                        Shape::Point => DVec3::ZERO,
                        Shape::Sphere { radius } => {
                            unit * (f64::from(radius) * random(key, 3).cbrt())
                        }
                        Shape::Box { half_extents } => {
                            DVec3::new(
                                random(key, 3) * 2. - 1.,
                                random(key, 4) * 2. - 1.,
                                random(key, 5) * 2. - 1.,
                            ) * DVec3::from(half_extents.map(f64::from))
                        }
                        Shape::Disc { radius } => {
                            DVec3::new(az.cos(), 0., az.sin())
                                * (f64::from(radius) * random(key, 3).sqrt())
                        }
                    };
                    let cosine = 1. - random(key, 6) * (1. - c.cone_cos);
                    let sine = (1. - cosine * cosine).max(0.).sqrt();
                    let angle = random(key, 7) * std::f64::consts::TAU;
                    let initial = (c.axis * cosine
                        + (c.side * angle.cos() + c.other * angle.sin()) * sine)
                        * range(e.speed, random(key, 8));
                    let (pos, vel) = trajectory(
                        (spawn + c.offset).to_array(),
                        initial.to_array(),
                        c.acceleration.to_array(),
                        c.drag,
                        t,
                    );
                    let mut pos = DVec3::from(pos);
                    let mut vel = DVec3::from(vel);
                    let mut rgba = std::array::from_fn(|c| {
                        e.color_min[c]
                            + (e.color_max[c] - e.color_min[c]) * random(key, 20 + c as u64) as f32
                    });
                    let mut size = range(e.size, random(key, 9)) as f32;
                    for m in &e.modules {
                        match m {
                            Module::Wave {
                                amplitude,
                                frequency_hz,
                            } => {
                                for c in 0..3 {
                                    let w = f64::from(frequency_hz[c]) * std::f64::consts::TAU;
                                    let phase = random(key, 40 + c as u64) * std::f64::consts::TAU;
                                    pos[c] += f64::from(amplitude[c])
                                        * ((w * t + phase).sin() - phase.sin());
                                    vel[c] += f64::from(amplitude[c]) * w * (w * t + phase).cos();
                                }
                            }
                            Module::ColorOverLife { keys } => {
                                if let Some((a, b, u)) = curve_time(
                                    fraction,
                                    keys.iter().map(|k| (k.time, k.value.as_slice())),
                                ) {
                                    for c in 0..4 {
                                        rgba[c] *= a[c] + (b[c] - a[c]) * u;
                                    }
                                }
                            }
                            Module::SizeOverLife { keys } => {
                                if let Some((a, b, u)) = curve_time(
                                    fraction,
                                    keys.iter()
                                        .map(|k| (k.time, std::slice::from_ref(&k.value))),
                                ) {
                                    size *= a[0] + (b[0] - a[0]) * u;
                                }
                            }
                            _ => {}
                        }
                    }
                    for (c, tint) in rgba.iter_mut().zip(p.tint) {
                        *c *= tint;
                    }
                    pos = c.rotation * pos * f64::from(p.scale);
                    vel = c.rotation * vel * f64::from(p.scale);
                    let projected = p
                        .position
                        .translated(pos.to_array())
                        .and_then(|w| w.project(origin(), 1_000_000., 0.001));
                    let Ok(position) = projected else {
                        self.stats.projection_drops += 1;
                        continue;
                    };
                    self.samples.push(Sample {
                        position,
                        velocity: Vec3::new(vel.x as f32, vel.y as f32, vel.z as f32).to_array(),
                        color: rgba,
                        size: size * p.scale,
                        angle: (range(e.rotation_radians, random(key, 10))
                            + range(e.angular_velocity, random(key, 11)) * t)
                            as f32,
                        aspect: e.aspect,
                        softness: e.softness,
                        align_velocity: e.align_velocity,
                        age: fraction,
                        material: slot,
                    });
                    accepted += 1;
                }
            }
        }
        self.stats.alive = self.samples.len();
        self.document = Some(document.clone());
        self.tick = Some(tick);
    }
    pub fn inspect(&self) -> serde_json::Value {
        json!({"stats":self.stats,"backend":"cpu_analytic_60hz","material_slots":self.materials.len(),"sample":self.samples.iter().take(16).map(|p|json!({"position":p.position,"velocity":p.velocity,"color":p.color,"size":p.size,"age":p.age})).collect::<Vec<_>>()})
    }
}
