//! Bounded physical recipient selection, independent of NPC knowledge.
use crate::{
    abilities::{Effect, Outcome, Recipient, World},
    model::{Result, invalid},
    targeting::MAX_ACTORS,
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, ops::Deref, sync::OnceLock};

pub const RECEIPTS: usize = 64;
static UNIT_CIRCLE: std::sync::LazyLock<[[f32; 2]; 32]> = std::sync::LazyLock::new(|| {
    std::array::from_fn(|i| {
        let (s, c) = (i as f32 * std::f32::consts::TAU / 32.).sin_cos();
        [c, s]
    })
});
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Area {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistence: Option<crate::persistent_area::Schedule>,
    pub radius: f32,
    pub max_targets: u16,
    pub include_source: bool,
    /// Solid bodies block the center-to-center segment; visual visibility is irrelevant.
    pub occlusion: bool,
}
impl Area {
    pub fn validate(self, version: u32, effects: &[Effect], scheduled: bool) -> Result<()> {
        if version < 41
            || !self.radius.is_finite()
            || !(0.1..=100.).contains(&self.radius)
            || !(1..=MAX_ACTORS as u16).contains(&self.max_targets)
            || (version < 87 && self.max_targets > 9)
            || scheduled
        {
            return Err(invalid(
                "Area requires version41, radius0.1..100, max_targets1..1024 (above9 requires version87) and no periodic/channel schedule",
            ));
        }
        if let Some(schedule) = self.persistence {
            schedule.validate(version)?;
        }
        if effects.iter().any(|e| {
            !matches!(
                e,
                Effect::ApplyBuff {
                    recipient: Recipient::Target,
                    ..
                } | Effect::Interrupt {
                    recipient: Recipient::Target,
                    ..
                } | Effect::Dispel {
                    recipient: Recipient::Target,
                    ..
                } | Effect::DamageComponents {
                    recipient: Recipient::Target,
                    ..
                } | Effect::Damage {
                    recipient: Recipient::Target,
                    ..
                } | Effect::Heal {
                    recipient: Recipient::Target,
                    ..
                } | Effect::Resource {
                    recipient: Recipient::Target,
                    ..
                }
            )
        }) {
            return Err(invalid(
                "Area effects must target recipients; source effects and noise cannot be repeated per recipient",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Selected,
    SourceExcluded,
    Dead,
    RelationDenied,
    Outside,
    Occluded,
    TargetLimit,
    Applied,
    EffectFailed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Candidate {
    pub actor_instance: crate::identity::Reference,
    pub actor: u16,
    pub distance_squared: f32,
    pub status: Status,
    pub failure: Option<Outcome>,
}
fn order(a: &Candidate, b: &Candidate) -> std::cmp::Ordering {
    a.distance_squared
        .total_cmp(&b.distance_squared)
        .then_with(|| a.actor_instance.instance.cmp(&b.actor_instance.instance))
}
fn row_order(a: &Option<Candidate>, b: &Option<Candidate>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => order(a, b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
/// Frozen complete rows; diagnostic ordering is derived only when observed.
#[derive(Debug)]
pub struct Candidates {
    by_actor: Vec<Option<Candidate>>,
    sorted: OnceLock<Vec<Option<Candidate>>>,
}
impl Candidates {
    fn new(by_actor: Vec<Option<Candidate>>) -> Self {
        Self {
            by_actor,
            sorted: OnceLock::new(),
        }
    }
    fn into_sorted(self) -> Vec<Option<Candidate>> {
        if let Some(sorted) = self.sorted.into_inner() {
            sorted
        } else {
            let mut rows = self.by_actor;
            rows.sort_unstable_by(row_order);
            rows
        }
    }
}
impl Clone for Candidates {
    fn clone(&self) -> Self {
        Self::new(self.by_actor.clone())
    }
}
impl PartialEq for Candidates {
    fn eq(&self, other: &Self) -> bool {
        self.by_actor == other.by_actor
    }
}
impl Deref for Candidates {
    type Target = [Option<Candidate>];
    fn deref(&self) -> &Self::Target {
        self.sorted.get_or_init(|| {
            let mut rows = self.by_actor.clone();
            rows.sort_unstable_by(row_order);
            rows
        })
    }
}
impl Serialize for Candidates {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.deref().serialize(s)
    }
}
impl<'de> Deserialize<'de> for Candidates {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        use serde::de::Error;
        let rows = Vec::<Option<Candidate>>::deserialize(d)?;
        if rows.len() > MAX_ACTORS {
            return Err(D::Error::custom("Area candidate capacity exceeded"));
        }
        let mut by_actor = vec![None; rows.len()];
        for candidate in rows.into_iter().flatten() {
            let slot = by_actor
                .get_mut(candidate.actor as usize)
                .ok_or_else(|| D::Error::custom("Area candidate actor outside row domain"))?;
            if slot.is_some() {
                return Err(D::Error::custom("Duplicate area candidate actor"));
            }
            *slot = Some(candidate);
        }
        Ok(Self::new(by_actor))
    }
}
pub(crate) struct Selection {
    rows: Vec<Option<Candidate>>,
    pub selected: Vec<Candidate>,
}
impl Selection {
    pub(crate) fn finish(mut self) -> Candidates {
        for candidate in self.selected {
            self.rows[candidate.actor as usize] = Some(candidate);
        }
        Candidates::new(self.rows)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub actor_instance: crate::identity::Reference,
    pub pulse: Option<u32>,
    pub actor: u16,
    pub slot: u8,
    pub execution_id: u64,
    pub tick: u64,
    pub center: [f32; 3],
    pub area: Area,
    pub candidates: Candidates,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub receipts: VecDeque<Receipt>,
    pub dropped_receipts: u64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            receipts: VecDeque::with_capacity(RECEIPTS),
            dropped_receipts: 0,
        }
    }
}
impl State {
    pub fn record(&mut self, r: Receipt) {
        if self.receipts.len() == RECEIPTS {
            self.receipts.pop_front();
            self.dropped_receipts += 1;
        }
        self.receipts.push_back(r);
    }
    pub fn inspect(
        &self,
        scene: &crate::model::Scene,
        actors: &crate::targeting::Catalog,
    ) -> serde_json::Value {
        use serde_json::json;
        let receipts:Vec<_>=self.receipts.iter().map(|r|json!({
            "actor":actors.label(scene,r.actor_instance),"actor_instance":r.actor_instance.id(),"slot":r.slot+1,"execution_id":r.execution_id,"tick":r.tick,
            "center":r.center,"area":r.area,"pulse":r.pulse,"candidates":r.candidates.iter().flatten().map(|c|json!({
                "actor":actors.label(scene,c.actor_instance),"actor_instance":c.actor_instance.id(),"distance_squared":c.distance_squared,"status":c.status,"failure":c.failure
            })).collect::<Vec<_>>()
        })).collect();
        json!({"receipts":receipts,"capacity":RECEIPTS,"dropped_receipts":self.dropped_receipts})
    }
}
// Boundary contact alone must not let the floor shield a ground-centered area.
fn crosses_interior(from: Vec3, to: Vec3, center: Vec3, half: Vec3) -> bool {
    let delta = to - from;
    let (mut enter, mut leave) = (0_f32, 1_f32);
    for axis in 0..3 {
        let (min, max) = (center[axis] - half[axis], center[axis] + half[axis]);
        if delta[axis] == 0. {
            if from[axis] <= min || from[axis] >= max {
                return false;
            }
        } else {
            let a = (min - from[axis]) / delta[axis];
            let b = (max - from[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter >= leave {
                return false;
            }
        }
    }
    true
}
pub fn occluded(w: &World<'_>, from: Vec3, to: Vec3) -> bool {
    let blocks = |e: &crate::model::Entity, center| {
        matches!(
            e.body,
            Some(crate::model::BodyKind::Fixed | crate::model::BodyKind::Dynamic)
        ) && if e.geometry.is_box() {
            crosses_interior(from, to, center, Vec3::from(e.transform.size) * 0.5)
        } else {
            crate::geometry::Volume::entity(e, center)
                .cast(from, to - from, Vec3::ZERO, true, 0.)
                .is_some()
        }
    };
    if let Some(index) = w.spatial {
        index
            .interior_candidates(from, to)
            .any(|(entity, center)| blocks(&w.scene.entities[entity], center))
    } else {
        w.scene.entities.iter().any(|e| {
            matches!(
                e.body,
                Some(crate::model::BodyKind::Fixed | crate::model::BodyKind::Dynamic)
            ) && w
                .positions
                .get(&e.id)
                .is_some_and(|&center| blocks(e, center.into()))
        })
    }
}
pub fn select(
    w: &World<'_>,
    actor: u16,
    center: [f32; 3],
    area: Area,
    relation: Option<crate::targeting::Attitude>,
) -> Vec<Option<Candidate>> {
    select_from(w, w.actors.reference(actor), center, area, relation, |t| {
        w.actors.attitude(actor, t)
    })
}
pub fn select_from(
    w: &World<'_>,
    source: crate::identity::Reference,
    center: [f32; 3],
    area: Area,
    relation: Option<crate::targeting::Attitude>,
    attitude: impl Fn(u16) -> crate::targeting::Attitude,
) -> Vec<Option<Candidate>> {
    select_for_execution(w, source, center, area, relation, attitude)
        .finish()
        .into_sorted()
}
pub(crate) fn select_for_execution(
    w: &World<'_>,
    source: crate::identity::Reference,
    center: [f32; 3],
    area: Area,
    relation: Option<crate::targeting::Attitude>,
    attitude: impl Fn(u16) -> crate::targeting::Attitude,
) -> Selection {
    let mut result = vec![None; w.actors.actors.len()];
    let from = Vec3::from(center);
    for (a, entry) in result.iter_mut().enumerate().take(w.actors.actors.len()) {
        let a = a as u16;
        let Some(to) = w.position(w.actors.entity(a)) else {
            continue;
        };
        let distance_squared = from.distance_squared(to);
        let same_source =
            (!area.include_source || relation.is_some()) && w.actors.matches(a, source);
        let status = if same_source && !area.include_source {
            Status::SourceExcluded
        } else if w.vitality.dead(a as usize) {
            Status::Dead
        } else if !same_source && relation.is_some_and(|r| attitude(a) != r) {
            Status::RelationDenied
        } else if distance_squared > area.radius * area.radius {
            Status::Outside
        } else if area.occlusion && occluded(w, from, to) {
            Status::Occluded
        } else {
            Status::Selected
        };
        *entry = Some(Candidate {
            actor_instance: w.actors.reference(a),
            actor: a,
            distance_squared,
            status,
            failure: None,
        });
    }
    let mut selected: Vec<_> = result
        .iter()
        .flatten()
        .filter(|c| c.status == Status::Selected)
        .copied()
        .collect();
    selected.sort_unstable_by(order);
    for candidate in selected.iter().skip(area.max_targets as usize) {
        result[candidate.actor as usize].as_mut().unwrap().status = Status::TargetLimit;
    }
    selected.truncate(area.max_targets as usize);
    Selection {
        rows: result,
        selected,
    }
}
/// Three great circles; nominal geometry only, never a knowledge or hit-test input.
pub fn sphere(
    lines: &mut Vec<crate::render::LineGuide>,
    center: [f32; 3],
    radius: f32,
    color: [f32; 3],
) {
    for axis in 0..3 {
        for i in 0..32 {
            let point = |step: usize| {
                let [c, s] = UNIT_CIRCLE[step % 32];
                let mut p = center;
                p[(axis + 1) % 3] += radius * c;
                p[(axis + 2) % 3] += radius * s;
                p
            };
            lines.push(crate::render::LineGuide {
                start: point(i),
                end: point(i + 1),
                color,
                dashed: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BodyKind, RoomRecipe};
    use std::collections::BTreeMap;
    fn legacy_occluded(w: &World<'_>, from: Vec3, to: Vec3) -> bool {
        w.scene.entities.iter().any(|e| {
            w.positions.contains_key(&e.id)
                && matches!(
                    e.body,
                    Some(crate::model::BodyKind::Fixed | crate::model::BodyKind::Dynamic)
                )
                && if e.geometry.is_box() {
                    crosses_interior(
                        from,
                        to,
                        Vec3::from(w.positions[&e.id]),
                        Vec3::from(e.transform.size) * 0.5,
                    )
                } else {
                    crate::geometry::Volume::entity(e, w.positions[&e.id].into())
                        .cast(from, to - from, Vec3::ZERO, true, 0.)
                        .is_some()
                }
        })
    }
    fn legacy_select(
        w: &World<'_>,
        source: crate::identity::Reference,
        center: [f32; 3],
        area: Area,
        relation: Option<crate::targeting::Attitude>,
        attitude: impl Fn(u16) -> crate::targeting::Attitude,
    ) -> Vec<Option<Candidate>> {
        let mut result = vec![None; w.actors.actors.len()];
        let from = Vec3::from(center);
        for (a, entry) in result.iter_mut().enumerate().take(w.actors.actors.len()) {
            let a = a as u16;
            let Some(position) = w.positions.get(w.actors.id(w.scene, a)) else {
                continue;
            };
            let to = Vec3::from(*position);
            let distance_squared = from.distance_squared(to);
            let same_source = w.actors.matches(a, source);
            let status = if same_source && !area.include_source {
                Status::SourceExcluded
            } else if w.vitality.dead(a as usize) {
                Status::Dead
            } else if !same_source && relation.is_some_and(|r| attitude(a) != r) {
                Status::RelationDenied
            } else if distance_squared > area.radius * area.radius {
                Status::Outside
            } else if area.occlusion && legacy_occluded(w, from, to) {
                Status::Occluded
            } else {
                Status::Selected
            };
            *entry = Some(Candidate {
                actor_instance: w.actors.reference(a),
                actor: a,
                distance_squared,
                status,
                failure: None,
            });
        }
        let populated = &mut result[..w.actors.actors.len()];
        populated.sort_unstable_by(|a, b| {
            let (a, b) = match (a, b) {
                (Some(a), Some(b)) => (a, b),
                (Some(_), None) => return std::cmp::Ordering::Less,
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (None, None) => return std::cmp::Ordering::Equal,
            };
            a.distance_squared
                .total_cmp(&b.distance_squared)
                .then_with(|| a.actor_instance.instance.cmp(&b.actor_instance.instance))
        });
        let mut count = 0;
        for c in populated.iter_mut().flatten() {
            if c.status == Status::Selected {
                if count >= area.max_targets {
                    c.status = Status::TargetLimit;
                } else {
                    count += 1;
                }
            }
        }
        result
    }

    // Critical oracle: deferring diagnostics must not change target order, frozen
    // status/results, presence, occlusion or persisted complete rows.
    #[test]
    fn deferred_rows_and_selected_order_match_legacy_and_survive_world_changes() {
        let mut scene = RoomRecipe {
            width: 24.,
            depth: 24.,
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let prototype = scene
            .entities
            .iter()
            .find(|e| e.body == Some(BodyKind::Player))
            .unwrap()
            .clone();
        scene.entities.retain(|e| e.body != Some(BodyKind::Player));
        for actor in 0..80 {
            let mut e = prototype.clone();
            e.id = format!("p{actor:03}");
            e.name = e.id.clone();
            e.transform.position = [
                -8. + (actor % 10) as f32 * 1.6,
                0.8,
                -6. + (actor / 10) as f32 * 1.6,
            ];
            e.vitality = Some(
                serde_json::from_value(serde_json::json!({"initial":100,"maximum":100,"scale":1}))
                    .unwrap(),
            );
            scene.entities.push(e);
        }
        let mut wall = scene
            .entities
            .iter()
            .find(|e| e.body == Some(BodyKind::Fixed))
            .unwrap()
            .clone();
        wall.id = "cover".into();
        wall.transform.position = [-2., 0.8, 0.];
        wall.transform.size = [0.05, 3., 6.];
        scene.entities.push(wall.clone());
        wall.id = "wedge".into();
        wall.transform.position = [3., 0.5, 0.];
        wall.transform.size = [2., 1., 3.];
        wall.geometry = crate::geometry::Geometry::Wedge {
            rise: crate::geometry::Rise::PositiveX,
        };
        scene.entities.push(wall);
        scene.validate().unwrap();
        let actors = crate::targeting::Catalog::new(&scene);
        let mut initial: BTreeMap<_, _> = scene
            .entities
            .iter()
            .map(|e| (e.id.clone(), e.transform.position))
            .collect();
        initial.remove("p007");
        // Equal distance and nearly parallel/boundary segments exercise ordering/interior policy.
        initial.insert("p010".into(), [0., 0.8, 0.]);
        initial.insert("p011".into(), [0., 0.8, 0.]);
        let npcs = BTreeMap::new();
        for indexed in [false, true] {
            for case in 0..6 {
                let mut positions = initial.clone();
                let mut index = crate::combat_spatial::Index::default();
                index.rebuild(&scene, &positions);
                let mut vitality = crate::vitality::Runtime::with_actors(&scene, &actors);
                vitality.state.actors[3].as_mut().unwrap().units = 0;
                let source = actors.reference(0);
                let source = if case == 5 {
                    crate::identity::Reference {
                        instance: source.instance + 10000,
                        ..source
                    }
                } else {
                    source
                };
                let area = Area {
                    persistence: None,
                    radius: if case == 0 { 3. } else { 30. },
                    max_targets: if case == 1 { 80 } else { 5 },
                    include_source: case != 0,
                    occlusion: case % 2 == 0,
                };
                let center = if case == 0 {
                    [-8., 0.8, -6.]
                } else if case == 4 {
                    [3., 1., 0.]
                } else {
                    [0., 0., 0.]
                };
                let relation = (case == 3).then_some(crate::targeting::Attitude::Hostile);
                let before = vitality.state.clone();
                let (frozen, expected) = {
                    let w = World {
                        scene: &scene,
                        actors: &actors,
                        vitality: &mut vitality,
                        positions: &positions,
                        spatial: indexed.then_some(&index),
                        npcs: &npcs,
                        tick: 42,
                    };
                    let attitude = |target| actors.attitude(0, target);
                    let mut expected = legacy_select(&w, source, center, area, relation, attitude);
                    assert_eq!(
                        select_from(&w, source, center, area, relation, attitude),
                        expected
                    );
                    let mut selected =
                        select_for_execution(&w, source, center, area, relation, attitude);
                    let eligible: Vec<_> = expected
                        .iter()
                        .flatten()
                        .filter(|c| c.status == Status::Selected)
                        .copied()
                        .collect();
                    assert_eq!(selected.selected, eligible);
                    for (i, candidate) in selected.selected.iter_mut().enumerate() {
                        candidate.status = if i == 0 {
                            Status::EffectFailed
                        } else {
                            Status::Applied
                        };
                        candidate.failure = (i == 0).then_some(Outcome::MissingResource);
                        *expected
                            .iter_mut()
                            .flatten()
                            .find(|c| c.actor == candidate.actor)
                            .unwrap() = *candidate;
                    }
                    assert_eq!(w.tick, 42);
                    assert_eq!(w.vitality.state, before);
                    (selected.finish(), expected)
                };
                positions.clear();
                vitality.state.actors.fill(None);
                index.rebuild(&scene, &positions);
                // First observation is deliberately after the originating world changed.
                assert_eq!(&*frozen, expected.as_slice());
                let cloned = frozen.clone();
                assert_eq!(cloned, frozen);
                let encoded = serde_json::to_value(&cloned).unwrap();
                assert_eq!(encoded, serde_json::to_value(&expected).unwrap());
                let restored: Candidates = serde_json::from_value(encoded.clone()).unwrap();
                assert_eq!(restored, frozen);
                assert_eq!(&*restored, expected.as_slice());
                let mut malformed = encoded;
                if let Some(first) = malformed
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| !v.is_null())
                    .cloned()
                {
                    malformed.as_array_mut().unwrap().push(first);
                    assert!(serde_json::from_value::<Candidates>(malformed).is_err());
                }
            }
        }
    }
}
