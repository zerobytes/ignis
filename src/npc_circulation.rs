//! Anonymous local body sensing; never an input to target knowledge.
use crate::{
    combat_spatial::Index,
    geometry::Volume,
    model::{BodyKind, Entity, FIXED_DT, Result, Scene, invalid},
    npc_mind::Profile,
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[path = "npc_air_circulation.rs"]
mod air;
pub(crate) use air::Air;
pub use air::{AirPhase, AirReport};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub radius: f32,
    pub horizon_seconds: f32,
    pub personal_space: f32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            radius: 4.,
            horizon_seconds: 1.,
            personal_space: 0.15,
        }
    }
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if !self.radius.is_finite()
            || !(0.5..=16.).contains(&self.radius)
            || !self.horizon_seconds.is_finite()
            || !(0.1..=2.).contains(&self.horizon_seconds)
            || !self.personal_space.is_finite()
            || !(0. ..=2.).contains(&self.personal_space)
        {
            return Err(invalid(
                "Circulation requires radius [0.5,16], horizon_seconds [0.1,2], personal_space [0,2]",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Clear,
    Avoiding,
    Yielding,
    Stationary,
    CommittedAir,
    Dead,
    LaunchDeferred,
    Blocked,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct State {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub air: Option<AirReport>,
    pub tick: u64,
    pub reason: Reason,
    pub candidates: u8,
    pub neighbors: u8,
    pub scanned: u16,
    pub limited: bool,
    pub horizon_seconds: f32,
    pub personal_space: f32,
    pub requested: [f32; 3],
    pub chosen: [f32; 3],
}
impl State {
    pub fn inactive(tick: u64, reason: Reason, velocity: Vec3) -> Self {
        Self {
            air: None,
            tick,
            reason,
            candidates: 0,
            neighbors: 0,
            scanned: 0,
            limited: false,
            horizon_seconds: 0.,
            personal_space: 0.,
            requested: velocity.into(),
            chosen: velocity.into(),
        }
    }
}
#[derive(Clone, Copy)]
struct Neighbor {
    index: usize,
    position: Vec3,
    velocity: Vec3,
    half: Vec3,
    distance: f32,
}

fn visible(scene: &Scene, index: &Index, from: Vec3, to: Vec3) -> bool {
    !index.interior_candidates(from, to).any(|(i, p)| {
        let e = &scene.entities[i];
        e.body == Some(BodyKind::Fixed)
            && Volume::entity(e, p)
                .cast(from, to - from, Vec3::ZERO, true, 0.)
                .is_some()
    })
}
fn static_safe(
    scene: &Scene,
    index: &Index,
    e: &Entity,
    from: Vec3,
    v: Vec3,
    ground: Option<&crate::ground_navigation::World>,
) -> bool {
    let end = from + v * FIXED_DT;
    if let Some(world) = ground.filter(|_| e.character_controller.is_some()) {
        let half = e.transform.size[1] * 0.5;
        return world
            .trace_limited(
                e,
                (from - Vec3::Y * half).into(),
                (end - Vec3::Y * half).into(),
                2,
            )
            .status
            == crate::ground_navigation::Status::Arrived;
    }
    let surface = scene
        .entities
        .iter()
        .find(|s| s.id == e.npc.as_ref().unwrap().surface)
        .unwrap();
    let floor_half = Vec3::from(surface.transform.size) * 0.5;
    let delta = end - Vec3::from(surface.transform.position);
    let half = Vec3::from(e.transform.size) * 0.5;
    if delta.x.abs() + half.x > floor_half.x || delta.z.abs() + half.z > floor_half.z {
        return false;
    }
    !index
        .sweep(from, end - from, half.max_element())
        .any(|(i, p)| {
            let solid = &scene.entities[i];
            solid.body == Some(BodyKind::Fixed)
                && solid.id != surface.id
                && Volume::entity(solid, p)
                    .cast(
                        from,
                        end - from,
                        (half - Vec3::splat(0.005)).max(Vec3::splat(0.001)),
                        true,
                        0.,
                    )
                    .is_some()
        })
}
fn risk(p: Vec3, v: Vec3, half: Vec3, neighbors: &[Neighbor], horizon: f32, space: f32) -> f32 {
    let mut worst = 0_f32;
    for n in neighbors {
        let extent = half + n.half + Vec3::splat(space);
        let d = (p - n.position) / extent;
        let dv = (v - n.velocity) / extent;
        let t = if dv.length_squared() > 1e-8 {
            (-d.dot(dv) / dv.length_squared()).clamp(0., horizon)
        } else {
            0.
        };
        let near = (d + dv * t).abs().max_element();
        // A box contact is safe at normalized distance >=1; personal-space risk is gradual.
        worst = worst.max((1.2 - near).max(0.) * (1. + (horizon - t) / horizon));
    }
    worst
}
#[allow(clippy::too_many_arguments)]
fn sense(
    scene: &Scene,
    index: &Index,
    velocities: &[Vec3],
    entity: usize,
    p: Vec3,
    config: Config,
    out: &mut State,
    include_dynamic: bool,
) -> ([Neighbor; 16], usize) {
    let mut nearby: [Option<Neighbor>; 16] = [None; 16];
    let mut count = 0;
    for (i, position) in index.sweep(p, Vec3::ZERO, config.radius) {
        if out.scanned == 128 {
            out.limited = true;
            break;
        }
        out.scanned += 1;
        if i == entity
            || !(matches!(
                scene.entities[i].body,
                Some(BodyKind::Npc | BodyKind::Player)
            ) || (include_dynamic && scene.entities[i].body == Some(BodyKind::Dynamic)))
        {
            continue;
        }
        let distance = p.distance_squared(position);
        if distance > config.radius * config.radius || !visible(scene, index, p, position) {
            continue;
        }
        let n = Neighbor {
            index: i,
            position,
            velocity: velocities[i],
            half: Vec3::from(scene.entities[i].transform.size) * 0.5,
            distance,
        };
        let at = nearby[..count]
            .iter()
            .position(|v| {
                let v = v.unwrap();
                n.distance
                    .total_cmp(&v.distance)
                    .then(n.index.cmp(&v.index))
                    .is_lt()
            })
            .unwrap_or(count);
        if count == 16 {
            out.limited = true;
        }
        if at < 16 {
            for j in (at + 1..count.min(15) + 1).rev() {
                nearby[j] = nearby[j - 1];
            }
            nearby[at] = Some(n);
            count = (count + 1).min(16);
        }
    }
    let empty = Neighbor {
        index: 0,
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        half: Vec3::ZERO,
        distance: 0.,
    };
    (nearby.map(|v| v.unwrap_or(empty)), count)
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn steer(
    scene: &Scene,
    index: &Index,
    velocities: &[Vec3],
    entity: usize,
    p: Vec3,
    desired: Vec3,
    profile: Option<Profile>,
    ground: Option<&crate::ground_navigation::World>,
    tick: u64,
) -> State {
    let e = &scene.entities[entity];
    let config = e.npc.as_ref().unwrap().circulation.unwrap();
    let profile = profile.unwrap_or(Profile {
        intelligence: 5000,
        traits: [5000; 9],
    });
    let q = profile.intelligence as f32 / 10_000.;
    let traits = profile.traits.map(|v| v as f32 / 10_000.);
    let horizon = config.horizon_seconds * (0.35 + 0.65 * q);
    let space = config.personal_space * (0.5 + traits[1] + traits[3] * 0.5 - traits[4] * 0.25);
    let mut out = State::inactive(tick, Reason::Clear, desired);
    out.horizon_seconds = horizon;
    out.personal_space = space;
    if Vec3::new(desired.x, 0., desired.z).length_squared() < 1e-8 {
        out.reason = Reason::Stationary;
        return out;
    }
    let (ns, count) = sense(scene, index, velocities, entity, p, config, &mut out, false);
    out.neighbors = count as u8;
    if count == 0 {
        return out;
    }
    let half = Vec3::from(e.transform.size) * 0.5;
    let planar = Vec3::new(desired.x, 0., desired.z);
    let initial_risk = risk(p, planar, half, &ns[..count], horizon, space);
    if initial_risk <= 0. {
        return out;
    }
    let speed = planar.length();
    let dir = planar / speed;
    let right = Vec3::new(-dir.z, 0., dir.x);
    let alternatives = if q < 0.34 {
        5
    } else if q < 0.67 {
        7
    } else {
        9
    };
    // Same handedness in each actor's local frame makes head-on actors pass on opposite sides.
    let mut best = Vec3::ZERO;
    let mut best_score = f32::NEG_INFINITY;
    for i in 0..alternatives {
        let (forward, side) = match i {
            0 => (1., 0.),
            1 => (0.8660254, 0.5),
            2 => (0.8660254, -0.5),
            3 => (0.5, 0.8660254),
            4 => (0.5, -0.8660254),
            5 => (0., 1.),
            6 => (0., -1.),
            7 => (0.5, 0.),
            _ => (0., 0.),
        };
        let v = (dir * forward + right * side) * speed;
        out.candidates += 1;
        if v != Vec3::ZERO && !static_safe(scene, index, e, p, v, ground) {
            continue;
        }
        let penalty = risk(p, v, half, &ns[..count], horizon, space);
        let score = forward
            - penalty * (3. + traits[1] + traits[3] - traits[4] * 0.5 - traits[6] * 0.5)
            + side * (0.08 + traits[8] * 0.08);
        if score > best_score {
            best_score = score;
            best = v;
        }
    }
    // Explicit stop is always an option, including for low intelligence.
    let stop_score = -risk(p, Vec3::ZERO, half, &ns[..count], horizon, space) * 4. - 0.25;
    if stop_score > best_score {
        best = Vec3::ZERO;
    }
    out.reason = if best.length_squared() < 1e-8 {
        Reason::Yielding
    } else {
        Reason::Avoiding
    };
    out.chosen = [best.x, desired.y, best.z];
    out
}
