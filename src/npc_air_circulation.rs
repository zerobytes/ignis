//! Receding-horizon guidance, evaluated by the same motor that commits the step.
use super::*;
use crate::{flight, mobility::Mode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AirPhase {
    Flight,
    Holding,
    Launch,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AirReport {
    pub phase: AirPhase,
    pub prediction_ticks: u8,
    pub static_rejections: u8,
    /// No collision-free candidate was found; the least-risk admissible motor command wins.
    pub fallback: bool,
}
pub(crate) struct Air<'a> {
    pub index: &'a Index,
    pub velocities: &'a [Vec3],
    pub entity: usize,
    pub profile: Option<Profile>,
    pub tick: u64,
    pub decision: &'a mut Option<State>,
}
#[derive(Clone, Copy)]
struct Command {
    side: f32,
    up: f32,
    scale: f32,
}
// Symmetric directions with deterministic right-hand preference. Vertical options
// precede wide lateral turns so every intelligence tier can use all three axes.
const COMMANDS: [Command; 11] = [
    Command {
        side: 0.,
        up: 0.,
        scale: 1.,
    },
    Command {
        side: 0.65,
        up: 0.,
        scale: 1.,
    },
    Command {
        side: -0.65,
        up: 0.,
        scale: 1.,
    },
    Command {
        side: 0.,
        up: 0.65,
        scale: 1.,
    },
    Command {
        side: 0.,
        up: -0.65,
        scale: 1.,
    },
    Command {
        side: 1.7,
        up: 0.,
        scale: 1.,
    },
    Command {
        side: -1.7,
        up: 0.,
        scale: 1.,
    },
    Command {
        side: 0.,
        up: 1.7,
        scale: 1.,
    },
    Command {
        side: 0.,
        up: -1.7,
        scale: 1.,
    },
    Command {
        side: 0.,
        up: 0.,
        scale: 0.5,
    },
    Command {
        side: 0.,
        up: 0.,
        scale: 0.,
    },
];
fn safe(scene: &Scene, index: &Index, e: &Entity, a: Vec3, b: Vec3) -> bool {
    let half = Vec3::from(e.transform.size) * 0.5;
    !index.sweep(a, b - a, half.max_element()).any(|(i, p)| {
        let solid = &scene.entities[i];
        solid.body == Some(BodyKind::Fixed)
            && Volume::entity(solid, p)
                .cast(
                    a,
                    b - a,
                    (half - Vec3::splat(0.005)).max(Vec3::splat(0.001)),
                    true,
                    0.,
                )
                .is_some()
    })
}
fn collision(a: Vec3, b: Vec3, extent: Vec3) -> bool {
    // Slab intersection in relative coordinates; test the whole interval, not
    // just its ends or Euclidean closest approach for an anisotropic box.
    let delta = b - a;
    let (mut enter, mut leave) = (0_f32, 1_f32);
    for axis in 0..3 {
        if delta[axis].abs() < 1e-8 {
            if a[axis].abs() >= extent[axis] {
                return false;
            }
        } else {
            let lo = (-extent[axis] - a[axis]) / delta[axis];
            let hi = (extent[axis] - a[axis]) / delta[axis];
            enter = enter.max(lo.min(hi));
            leave = leave.min(lo.max(hi));
            if enter >= leave {
                return false;
            }
        }
    }
    enter < leave
}
impl Air<'_> {
    fn sensed(
        &self,
        scene: &Scene,
        e: &Entity,
        p: Vec3,
        desired: Vec3,
        phase: AirPhase,
    ) -> (State, [Neighbor; 16], usize, [f32; 9], usize) {
        let config = e.npc.as_ref().unwrap().circulation.unwrap();
        let profile = self.profile.unwrap_or(Profile {
            intelligence: 5000,
            traits: [5000; 9],
        });
        let q = profile.intelligence as f32 / 10_000.;
        let traits = profile.traits.map(|v| v as f32 / 10_000.);
        let mut out = State::inactive(self.tick, Reason::Clear, desired);
        out.horizon_seconds = config.horizon_seconds * (0.35 + 0.65 * q);
        out.personal_space =
            config.personal_space * (0.5 + traits[1] + traits[3] * 0.5 - traits[4] * 0.25);
        let (ns, count) = sense(
            scene,
            self.index,
            self.velocities,
            self.entity,
            p,
            config,
            &mut out,
            true,
        );
        out.neighbors = count as u8;
        out.air = Some(AirReport {
            phase,
            prediction_ticks: (out.horizon_seconds / FIXED_DT).ceil().clamp(1., 120.) as u8,
            static_rejections: 0,
            fallback: false,
        });
        (
            out,
            ns,
            count,
            traits,
            if q < 0.34 {
                5
            } else if q < 0.67 {
                7
            } else {
                9
            },
        )
    }
    /// Feet-space belongs to mobility; sensing/collision use body centers.
    #[allow(clippy::too_many_arguments)]
    pub fn advance(
        &mut self,
        scene: &Scene,
        e: &Entity,
        own: Vec3,
        end: Vec3,
        motor: Option<(flight::Settings, flight::State)>,
        speed: f32,
        radius: f32,
        phase: AirPhase,
    ) -> (Vec3, Option<flight::State>, bool) {
        let half = Vec3::from(e.transform.size) * 0.5;
        let center = own + Vec3::Y * half.y;
        let desired = (end - own).normalize_or_zero() * speed.min(own.distance(end) / FIXED_DT);
        let (mut out, ns, count, traits, alternatives) =
            self.sensed(scene, e, center, desired, phase);
        // The launch preview and per-step clearance already protect the original
        // path. Without sensed neighbors there is no local interaction to predict.
        let ticks = if count == 0 {
            1
        } else {
            out.air.unwrap().prediction_ticks
        };
        out.air.as_mut().unwrap().prediction_ticks = ticks;
        let dir = (end - own).normalize_or_zero();
        let right = if dir.x * dir.x + dir.z * dir.z > 1e-6 {
            Vec3::new(-dir.z, 0., dir.x).normalize()
        } else {
            Vec3::X * if dir.y >= 0. { 1. } else { -1. }
        };
        let hemisphere = if dir.x.abs().max(dir.z.abs()) < 0.001 {
            dir.y
        } else if dir.x.abs() >= dir.z.abs() {
            dir.x
        } else {
            dir.z
        };
        let up = right.cross(dir).normalize_or_zero() * hemisphere.signum();
        let mut best = None;
        let mut best_score = f32::NEG_INFINITY;
        let mut best_risk = f32::INFINITY;
        for i in (0..alternatives).chain(9..11) {
            let command = COMMANDS[i];
            let mut p = own;
            let mut f = motor.map(|(_, f)| f);
            let mut first = None;
            let mut penalty = 0_f32;
            let mut blocked = false;
            out.candidates += 1;
            for step in 0..ticks {
                let direction =
                    ((end - p).normalize_or_zero() + right * command.side + up * command.up)
                        .normalize_or_zero();
                let (v, arrived) = if let Some((c, _)) = motor {
                    let f = f.as_mut().unwrap();
                    let arrived = if phase == AirPhase::Holding && i == 0 {
                        f.orbit(p);
                        false
                    } else {
                        let arrived = f.advance_guided(c, speed, p, end, direction, command.scale);
                        if phase == AirPhase::Holding {
                            f.orbit_speed = Vec3::from(f.velocity).length();
                        }
                        arrived
                    };
                    (Vec3::from(f.velocity), arrived)
                } else {
                    let arrived = p.distance(end) <= radius;
                    (
                        if arrived {
                            Vec3::ZERO
                        } else {
                            direction * (speed * command.scale).min(p.distance(end) / FIXED_DT)
                        },
                        arrived,
                    )
                };
                if first.is_none() {
                    first = Some((v, f, arrived));
                    if i == 0 {
                        out.requested = v.into();
                    }
                }
                let next = p + v * FIXED_DT;
                if !safe(
                    scene,
                    self.index,
                    e,
                    p + Vec3::Y * half.y,
                    next + Vec3::Y * half.y,
                ) {
                    blocked = true;
                    break;
                }
                let t = step as f32 * FIXED_DT;
                for n in &ns[..count] {
                    let a = p + Vec3::Y * half.y - n.position - n.velocity * t;
                    let b = next + Vec3::Y * half.y - n.position - n.velocity * (t + FIXED_DT);
                    let extent = half + n.half;
                    let weight = 1. + (ticks - step) as f32 / ticks as f32;
                    if collision(a, b, extent) {
                        penalty = penalty.max(10. * weight);
                    } else if collision(a, b, extent + Vec3::splat(out.personal_space)) {
                        penalty = penalty.max(weight);
                    }
                }
                p = next;
            }
            if blocked {
                out.air.as_mut().unwrap().static_rejections += 1;
                continue;
            }
            let progress = (own.distance(end) - p.distance(end))
                / (speed * ticks as f32 * FIXED_DT).max(0.001);
            let score = progress
                - penalty * (3. + traits[1] + traits[3] - traits[4] * 0.5 - traits[6] * 0.5)
                + command.side * (0.025 + traits[8] * 0.025)
                - command.up.abs() * 0.01;
            if score > best_score {
                best_score = score;
                best = first;
                best_risk = penalty;
            }
            // Preserve the old trajectory and skip extra work when it is safe.
            if i == 0 && penalty == 0. {
                break;
            }
        }
        let result = if let Some(result) = best {
            out.air.as_mut().unwrap().fallback = best_risk > 0.;
            out.reason = if result.0.length_squared() < 1e-8 {
                Reason::Yielding
            } else if result.0.distance_squared(out.requested.into()) > 1e-8 && out.candidates > 1 {
                Reason::Avoiding
            } else {
                Reason::Clear
            };
            result
        } else {
            // No statically admissible motor command: keep the ordinary motor's
            // first step. Rapier remains collision authority; never stop a
            // non-hover motor by changing its velocity after integration.
            out.reason = Reason::Blocked;
            out.air.as_mut().unwrap().fallback = true;
            let mut f = motor.map(|(_, f)| f);
            let arrived = if let Some((c, _)) = motor {
                f.as_mut().unwrap().advance(c, speed, own, end)
            } else {
                false
            };
            (f.map_or(Vec3::ZERO, |f| f.velocity.into()), f, arrived)
        };
        out.chosen = result.0.into();
        *self.decision = Some(out);
        result
    }
    /// Ballistic/sinking segments can wait on the ground, never steer in mid-air.
    pub fn defer_launch(
        &mut self,
        scene: &Scene,
        e: &Entity,
        own: Vec3,
        v: Vec3,
        mode: Mode,
    ) -> bool {
        let half = Vec3::from(e.transform.size) * 0.5;
        let p = own + Vec3::Y * half.y;
        let (mut out, ns, count, _, _) = self.sensed(scene, e, p, v, AirPhase::Launch);
        let mut a = p;
        let mut blocked = false;
        for step in 0..out.air.unwrap().prediction_ticks {
            let t = step as f32 * FIXED_DT;
            let next_t = t + FIXED_DT;
            let gravity = if mode == Mode::Jump {
                Vec3::Y * -9.81
            } else {
                Vec3::ZERO
            };
            let b = p + v * next_t + gravity * (0.5 * next_t * next_t);
            blocked |= ns[..count].iter().any(|n| {
                collision(
                    a - n.position - n.velocity * t,
                    b - n.position - n.velocity * next_t,
                    half + n.half + Vec3::splat(out.personal_space),
                )
            });
            a = b;
        }
        out.candidates = 1;
        if blocked {
            out.reason = Reason::LaunchDeferred;
            out.chosen = [0.; 3];
        }
        *self.decision = Some(out);
        blocked
    }
}
