//! Bounded straight delivery, swept against the current tick's physical boxes.
use crate::{
    abilities::Cast,
    model::{Result, Scene, invalid},
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
pub const CAPACITY: usize = 64;
const RECEIPTS: usize = 256;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Delivery {
    Direct {},
    Projectile {
        speed: f32,
        half_extent: f32,
        max_distance: f32,
    },
}
impl Default for Delivery {
    fn default() -> Self {
        Self::Direct {}
    }
}
impl Delivery {
    pub fn is_direct(&self) -> bool {
        matches!(self, Self::Direct {})
    }
    pub fn validate(self, version: u32) -> Result<()> {
        if let Self::Projectile {
            speed,
            half_extent,
            max_distance,
        } = self
            && (version < 30
                || !speed.is_finite()
                || !(0.1..=100.).contains(&speed)
                || !half_extent.is_finite()
                || !(0.01..=1.).contains(&half_extent)
                || !max_distance.is_finite()
                || !(0.1..=100.).contains(&max_distance))
        {
            return Err(invalid(
                "Projectile requires version30, speed0.1..100m/s, half_extent0.01..1m, max_distance0.1..100m",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Shot {
    pub actor: u16,
    pub cast: Cast,
    pub launched_tick: u64,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub speed: f32,
    pub half_extent: f32,
    pub remaining_distance: f32,
}
impl crate::source_pool::SourceItem for Shot {
    const QUOTA: usize = CAPACITY;
    fn source(&self) -> u16 {
        self.actor
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Hit,
    Blocked,
    RelationDenied,
    TargetDead,
    EffectFailed,
    Expired,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub actor_instance: crate::identity::Reference,
    pub entity_instance: Option<crate::identity::Reference>,
    pub execution_id: u64,
    pub actor: u16,
    pub tick: u64,
    pub position: [f32; 3],
    pub entity: Option<u16>,
    pub outcome: Outcome,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub active: crate::source_pool::Pool<Shot>,
    pub receipts: VecDeque<Receipt>,
    pub dropped_receipts: u64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            active: Default::default(),
            receipts: VecDeque::with_capacity(RECEIPTS),
            dropped_receipts: 0,
        }
    }
}
impl State {
    pub fn record(&mut self, receipt: Receipt) {
        if self.receipts.len() == RECEIPTS {
            self.receipts.pop_front();
            self.dropped_receipts += 1;
        }
        self.receipts.push_back(receipt);
    }
}

/// Returns first blocking body; visibility and faction do not change physical contact.
pub fn advance(
    shot: &mut Shot,
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    source: usize,
) -> Option<u16> {
    advance_with_spatial(shot, scene, positions, source, None)
}
/// Same exact sweep, using candidates from the matching pre-physics snapshot.
pub(crate) fn advance_with_spatial(
    shot: &mut Shot,
    scene: &Scene,
    positions: &BTreeMap<String, [f32; 3]>,
    source: usize,
    spatial: Option<&crate::combat_spatial::Index>,
) -> Option<u16> {
    let from = Vec3::from(shot.position);
    let distance = (shot.speed * crate::model::FIXED_DT).min(shot.remaining_distance);
    let delta = Vec3::from(shot.direction) * distance;
    let mut nearest: Option<(f32, usize)> = None;
    let mut visit = |i: usize, center| {
        let e = &scene.entities[i];
        if i == source || e.body.is_none() {
            return;
        }
        let half = Vec3::from(e.transform.size) * 0.5 + Vec3::splat(shot.half_extent);
        let contact = if e.geometry.is_box() {
            toi(from - center, delta, half)
        } else {
            crate::geometry::Volume::entity(e, center).cast(
                from,
                delta,
                Vec3::splat(shot.half_extent),
                false,
                1e-8,
            )
        };
        if let Some(t) = contact
            && nearest
                .is_none_or(|(best, j)| t < best || (t == best && e.id < scene.entities[j].id))
        {
            nearest = Some((t, i));
        }
    };
    if let Some(index) = spatial {
        for (i, center) in index.sweep(from, delta, shot.half_extent) {
            visit(i, center);
        }
    } else {
        for (i, e) in scene.entities.iter().enumerate() {
            if let Some(position) = crate::runtime_world::position(positions, &e.id) {
                visit(i, position.into());
            }
        }
    }
    let fraction = nearest.map_or(1., |(t, _)| t);
    shot.position = (from + delta * fraction).to_array();
    shot.remaining_distance = (shot.remaining_distance - distance * fraction).max(0.);
    nearest.map(|(_, i)| i as u16)
}
fn toi(origin: Vec3, delta: Vec3, half: Vec3) -> Option<f32> {
    let mut near = 0_f32;
    let mut far = 1_f32;
    for axis in 0..3 {
        if delta[axis].abs() < 1e-8 {
            if origin[axis].abs() > half[axis] {
                return None;
            }
        } else {
            let a = (-half[axis] - origin[axis]) / delta[axis];
            let b = (half[axis] - origin[axis]) / delta[axis];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    Some(near)
}
