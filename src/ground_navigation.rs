//! Bounded, read-only KCC admission against immutable fixed geometry.
use crate::{
    character_controller::Config,
    model::{BodyKind, Entity, FIXED_DT, Scene},
};
use glam::Vec3;
use rapier3d::parry::shape::Cuboid;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const MAX_TICKS: u16 = 600;
const ARRIVAL_XZ: f32 = 0.002;
const ARRIVAL_Y: f32 = 0.12;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Arrived,
    UnsupportedStart,
    InitialOverlap,
    UnsupportedEnd,
    EndpointOverlap,
    Unsupported,
    Blocked,
    EndpointMismatch,
    BudgetExceeded,
    NotWalking,
}
impl Status {
    pub fn reason(self) -> Option<&'static str> {
        match self {
            Self::Arrived => None,
            Self::UnsupportedStart => Some("ground_start_unsupported"),
            Self::InitialOverlap => Some("ground_initial_overlap"),
            Self::UnsupportedEnd => Some("ground_end_unsupported"),
            Self::EndpointOverlap => Some("ground_endpoint_overlap"),
            Self::Unsupported => Some("ground_support_lost"),
            Self::Blocked => Some("ground_path_blocked"),
            Self::EndpointMismatch => Some("ground_endpoint_mismatch"),
            Self::BudgetExceeded => Some("ground_probe_budget"),
            Self::NotWalking => Some("missing_capability"),
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub status: Status,
    pub ticks: u16,
    pub distance: f32,
    pub final_feet: [f32; 3],
}
impl Report {
    pub(crate) fn rejected(status: Status, feet: [f32; 3]) -> Self {
        Self {
            status,
            ticks: 0,
            distance: 0.,
            final_feet: feet,
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
struct Profile {
    size: [f32; 3],
    speed: f32,
    config: Config,
}
struct Cached {
    profile: Profile,
    reports: Arc<[Report]>,
}
/// The world is fixed for a Simulation; births reuse profiles warmed from templates.
pub struct World {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    broad: BroadPhaseBvh,
    narrow: NarrowPhase,
    profiles: Vec<Cached>,
}
impl World {
    pub fn new(scene: &Scene) -> Self {
        let mut w = Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            broad: BroadPhaseBvh::new(),
            narrow: NarrowPhase::new(),
            profiles: Vec::new(),
        };
        // Match Simulation insertion order, independently of authoring order.
        let mut fixed: Vec<_> = scene
            .entities
            .iter()
            .filter(|e| e.body == Some(BodyKind::Fixed))
            .collect();
        fixed.sort_by_key(|e| &e.id);
        for e in fixed {
            let shape = if e.geometry.is_box() {
                ColliderBuilder::cuboid(
                    e.transform.size[0] * 0.5,
                    e.transform.size[1] * 0.5,
                    e.transform.size[2] * 0.5,
                )
            } else {
                let vertices = e
                    .geometry
                    .vertices(e.transform.size)
                    .map(|p| point![p.x, p.y, p.z]);
                ColliderBuilder::convex_hull(&vertices).expect("validated wedge")
            };
            let p = e.transform.position;
            let h = w
                .colliders
                .insert(shape.translation(vector![p[0], p[1], p[2]]));
            w.broad.set_aabb(
                &IntegrationParameters::default(),
                h,
                w.colliders[h].compute_aabb(),
            );
        }
        for s in crate::surfaces::prepare(scene) {
            let h = w.colliders.insert(s.collider());
            w.broad.set_aabb(
                &IntegrationParameters::default(),
                h,
                w.colliders[h].compute_aabb(),
            );
        }
        w
    }
    fn profile(e: &Entity) -> Profile {
        Profile {
            size: e.transform.size,
            speed: e
                .npc
                .as_ref()
                .map_or_else(|| e.locomotion.unwrap_or_default().walk.speed, |n| n.speed),
            config: e.character_controller.expect("ground motor"),
        }
    }
    pub(crate) fn compile(&mut self, scene: &Scene, e: &Entity) -> Arc<[Report]> {
        let profile = Self::profile(e);
        if let Some(cache) = self.profiles.iter().find(|p| p.profile == profile) {
            return cache.reports.clone();
        }
        let reports: Arc<[Report]> = scene
            .spatial_network
            .links
            .iter()
            .map(|l| {
                let a = scene.spatial_network.nodes[l.from as usize];
                let b = scene.spatial_network.nodes[l.to as usize];
                if l.mode == crate::mobility::Mode::Walk {
                    self.trace(
                        e,
                        (Vec3::from(a) + Vec3::Y * profile.config.offset).into(),
                        (Vec3::from(b) + Vec3::Y * profile.config.offset).into(),
                    )
                } else {
                    Report::rejected(Status::NotWalking, a)
                }
            })
            .collect();
        // Only authored NPCs/templates can install profiles; at most8+16 per scene.
        assert!(
            self.profiles.len() < crate::templates::MAX_NPCS + crate::templates::MAX_TEMPLATES,
            "bounded ground profiles"
        );
        self.profiles.push(Cached {
            profile,
            reports: reports.clone(),
        });
        reports
    }
    pub(crate) fn free(&self, e: &Entity, feet: [f32; 3]) -> bool {
        let half = Vec3::from(e.transform.size) * 0.5;
        let shape = Cuboid::new(vector![
            (half.x - 0.005).max(0.001),
            (half.y - 0.005).max(0.001),
            (half.z - 0.005).max(0.001)
        ]);
        let pose = Isometry::translation(feet[0], feet[1] + half.y, feet[2]);
        self.broad
            .as_query_pipeline(
                self.narrow.query_dispatcher(),
                &self.bodies,
                &self.colliders,
                QueryFilter::default(),
            )
            .intersect_shape(pose, &shape)
            .next()
            .is_none()
    }
    pub fn trace(&self, e: &Entity, start: [f32; 3], end: [f32; 3]) -> Report {
        self.trace_limited(e, start, end, MAX_TICKS)
    }
    pub(crate) fn trace_limited(
        &self,
        e: &Entity,
        start: [f32; 3],
        end: [f32; 3],
        limit: u16,
    ) -> Report {
        if limit == 0 {
            return Report::rejected(Status::BudgetExceeded, start);
        }
        let profile = Self::profile(e);
        let controller = profile.config.compile();
        let half = Vec3::from(profile.size) * 0.5;
        let shape = Cuboid::new(vector![half.x, half.y, half.z]);
        let goal = Vec3::from(end);
        let mut feet = Vec3::from(start);
        let mut out = Report::rejected(Status::BudgetExceeded, start);
        let dispatcher =
            crate::character_cast::StableContacts(self.narrow.query_dispatcher(), false);
        let queries = self.broad.as_query_pipeline(
            &dispatcher,
            &self.bodies,
            &self.colliders,
            QueryFilter::default(),
        );
        let mut pose = Isometry::translation(feet.x, feet.y + half.y, feet.z);
        // Ignore contact tolerance, but refuse an authored body penetrating a solid.
        let inset = Cuboid::new(vector![
            (half.x - 0.005).max(0.001),
            (half.y - 0.005).max(0.001),
            (half.z - 0.005).max(0.001)
        ]);
        if queries.intersect_shape(pose, &inset).next().is_some() {
            out.status = Status::InitialOverlap;
            return out;
        }
        let initial = controller.move_shape(
            FIXED_DT,
            &queries,
            &shape,
            &pose,
            vector![0., -profile.config.gravity * FIXED_DT * FIXED_DT, 0.],
            |_| {},
        );
        if !initial.grounded {
            out.status = Status::UnsupportedStart;
            return out;
        }
        let end_pose = Isometry::translation(goal.x, goal.y + half.y, goal.z);
        if queries.intersect_shape(end_pose, &inset).next().is_some() {
            out.status = Status::EndpointOverlap;
            return out;
        }
        let end_contact = controller.move_shape(
            FIXED_DT,
            &queries,
            &shape,
            &end_pose,
            vector![0., -profile.config.gravity * FIXED_DT * FIXED_DT, 0.],
            |_| {},
        );
        if !end_contact.grounded {
            out.status = Status::UnsupportedEnd;
            return out;
        }
        let mut vertical = 0.;
        let mut stalled = 0;
        let mut step_grace = 0;
        let mut contact = initial.grounded;
        for tick in 1..=limit.min(MAX_TICKS) {
            let d = goal - feet;
            let horizontal = Vec3::new(d.x, 0., d.z);
            if horizontal.length_squared() <= ARRIVAL_XZ * ARRIVAL_XZ {
                out.status = if d.y.abs() <= ARRIVAL_Y && contact {
                    Status::Arrived
                } else {
                    Status::EndpointMismatch
                };
                return out;
            }
            vertical =
                (vertical - profile.config.gravity * FIXED_DT).max(-profile.config.max_fall_speed);
            let v =
                horizontal.normalize_or_zero() * profile.speed.min(horizontal.length() / FIXED_DT);
            let movement = controller.move_shape(
                FIXED_DT,
                &queries,
                &shape,
                &pose,
                vector![v.x, vertical, v.z] * FIXED_DT,
                |_| {},
            );
            let delta = Vec3::new(
                movement.translation.x,
                movement.translation.y,
                movement.translation.z,
            );
            feet += delta;
            pose.translation.vector += movement.translation;
            out.ticks = tick;
            out.distance += delta.length();
            out.final_feet = feet.into();
            if !movement.grounded {
                // Autostep can lift the box just before its front reaches the top.
                // Permit only this upward transition, bounded to three ticks.
                if contact
                    && profile.config.step.is_some_and(|s| {
                        delta.y > profile.config.offset
                            && delta.y <= s.max_height + profile.config.offset * 2.
                    })
                {
                    step_grace = 3;
                }
                if step_grace == 0 {
                    out.status = Status::Unsupported;
                    return out;
                }
                step_grace -= 1;
            } else {
                vertical = 0.;
                step_grace = 0;
            }
            contact = movement.grounded;
            let remaining = goal - feet;
            if horizontal.length() - Vec3::new(remaining.x, 0., remaining.z).length() < 0.0001 {
                stalled += 1;
            } else {
                stalled = 0;
            }
            if stalled >= 8 {
                out.status = Status::Blocked;
                return out;
            }
        }
        let d = goal - feet;
        if d.x * d.x + d.z * d.z <= ARRIVAL_XZ * ARRIVAL_XZ && d.y.abs() <= ARRIVAL_Y && contact {
            out.status = Status::Arrived;
        }
        out
    }
}
