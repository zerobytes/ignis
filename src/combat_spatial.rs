//! Conservative, reusable broadphase of one pre-physics combat snapshot.
use crate::model::Scene;
use glam::Vec3;
use std::collections::BTreeMap;

const NONE: u32 = u32::MAX;
// A balanced tree over the entire u16 entity domain has at most 17 levels.
const STACK: usize = 32;

#[derive(Clone, Copy)]
struct Leaf {
    sensory: u8,
    entity: u16,
    center: Vec3,
    half: Vec3,
}
#[derive(Clone, Copy)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}
impl Bounds {
    fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
    fn include(&mut self, min: Vec3, max: Vec3) {
        self.min = self.min.min(min);
        self.max = self.max.max(max);
    }
    fn intersects(self, from: Vec3, delta: Vec3, padding: Vec3, epsilon: f32) -> bool {
        let (mut enter, mut leave) = (0_f32, 1_f32);
        for axis in 0..3 {
            // Match the exact box/wedge query's established parallel policy.
            let min = self.min[axis] - padding[axis];
            let max = self.max[axis] + padding[axis];
            if delta[axis] == 0. || delta[axis].abs() < epsilon {
                if from[axis] < min || from[axis] > max {
                    return false;
                }
            } else {
                let a = (min - from[axis]) / delta[axis];
                let b = (max - from[axis]) / delta[axis];
                enter = enter.max(a.min(b));
                leave = leave.min(a.max(b));
                if enter > leave {
                    return false;
                }
            }
        }
        true
    }
}
struct Node {
    sensory: u8,
    bounds: Bounds,
    left: u32,
    right: u32,
}

#[derive(Default)]
pub struct Index {
    leaves: Vec<Leaf>,
    nodes: Vec<Node>,
    positions: Vec<Option<Vec3>>,
}
impl Index {
    pub fn rebuild(&mut self, scene: &Scene, positions: &BTreeMap<String, [f32; 3]>) {
        self.leaves.clear();
        self.nodes.clear();
        self.positions.clear();
        self.positions.resize(scene.entities.len(), None);
        for (entity, e) in scene.entities.iter().enumerate() {
            if let Some(&position) = positions.get(&e.id) {
                self.positions[entity] = Some(position.into());
                self.leaves.push(Leaf {
                    sensory: u8::from(e.blocks_vision.unwrap_or(e.body.is_some()))
                        | (u8::from(e.sound_transmission.unwrap_or(1.) < 1.) << 1),
                    entity: u16::try_from(entity).expect("validated entity domain"),
                    center: position.into(),
                    half: Vec3::from(e.transform.size) * 0.5,
                });
            }
        }
        if !self.leaves.is_empty() {
            self.build(0, self.leaves.len());
        }
    }
    fn build(&mut self, start: usize, end: usize) -> u32 {
        let mut bounds = Bounds::empty();
        let mut centers = Bounds::empty();
        let mut sensory = 0;
        for leaf in &self.leaves[start..end] {
            sensory |= leaf.sensory;
            bounds.include(leaf.center - leaf.half, leaf.center + leaf.half);
            centers.include(leaf.center, leaf.center);
        }
        let node = self.nodes.len() as u32;
        self.nodes.push(Node {
            sensory,
            bounds,
            left: start as u32,
            right: NONE,
        });
        if end - start > 1 {
            let extent = centers.max - centers.min;
            let axis = if extent.x >= extent.y && extent.x >= extent.z {
                0
            } else if extent.y >= extent.z {
                1
            } else {
                2
            };
            let middle = (start + end) / 2;
            self.leaves[start..end].select_nth_unstable_by(middle - start, |a, b| {
                a.center[axis]
                    .total_cmp(&b.center[axis])
                    .then(a.entity.cmp(&b.entity))
            });
            let left = self.build(start, middle);
            let right = self.build(middle, end);
            self.nodes[node as usize].left = left;
            self.nodes[node as usize].right = right;
        }
        node
    }
    pub(crate) fn candidates(&self, from: Vec3, to: Vec3) -> Candidates<'_> {
        self.query(from, to - from, Vec3::ZERO, 1e-7)
    }
    pub(crate) fn sight_candidates(&self, from: Vec3, to: Vec3) -> Candidates<'_> {
        let mut q = self.candidates(from, to);
        q.sensory = 1;
        q
    }
    pub(crate) fn position(&self, entity: usize) -> Option<Vec3> {
        self.positions.get(entity).copied().flatten()
    }
    pub(crate) fn interior_candidates(&self, from: Vec3, to: Vec3) -> Candidates<'_> {
        // Closed bounds conservatively contain the narrowphase's open interior.
        self.query(from, to - from, Vec3::ZERO, 0.)
    }
    pub(crate) fn sweep(&self, from: Vec3, delta: Vec3, half_extent: f32) -> Candidates<'_> {
        // The legacy TOI subtracts the center first; bounds use world coordinates.
        // Cover their f32 operation roundoff conservatively, without changing TOI.
        let magnitude = self
            .nodes
            .first()
            .map_or(Vec3::ONE, |root| {
                root.bounds.min.abs().max(root.bounds.max.abs())
            })
            .max(from.abs())
            .max(delta.abs())
            .max(Vec3::ONE);
        self.query(
            from,
            delta,
            Vec3::splat(half_extent) + magnitude * (8. * f32::EPSILON),
            1e-8,
        )
    }
    fn query(&self, from: Vec3, delta: Vec3, padding: Vec3, epsilon: f32) -> Candidates<'_> {
        let mut stack = [0; STACK];
        stack[0] = 0;
        Candidates {
            sensory: 0,
            index: self,
            from,
            delta,
            padding,
            epsilon,
            stack,
            len: usize::from(!self.nodes.is_empty()),
        }
    }
}
pub(crate) struct Candidates<'a> {
    sensory: u8,
    index: &'a Index,
    from: Vec3,
    delta: Vec3,
    padding: Vec3,
    epsilon: f32,
    stack: [u32; STACK],
    len: usize,
}
impl Iterator for Candidates<'_> {
    type Item = (usize, Vec3);
    fn next(&mut self) -> Option<Self::Item> {
        while self.len != 0 {
            self.len -= 1;
            let node = &self.index.nodes[self.stack[self.len] as usize];
            if self.sensory != 0 && node.sensory & self.sensory == 0 {
                continue;
            }
            if !node
                .bounds
                .intersects(self.from, self.delta, self.padding, self.epsilon)
            {
                continue;
            }
            if node.right == NONE {
                let leaf = self.index.leaves[node.left as usize];
                return Some((leaf.entity as usize, leaf.center));
            }
            self.stack[self.len] = node.left;
            self.stack[self.len + 1] = node.right;
            self.len += 2;
        }
        None
    }
}

/// Author-order segment candidates without per-query heap allocation. Required by
/// sensory transmission multiplication, whose floating-point order is observable.
pub(crate) struct OrderedCandidates {
    words: [u64; crate::model::MAX_ENTITIES.div_ceil(64)],
    word: usize,
    end: usize,
}
impl Iterator for OrderedCandidates {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.word < self.end {
            let bits = &mut self.words[self.word];
            if *bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                *bits &= *bits - 1;
                return Some(self.word * 64 + bit);
            }
            self.word += 1;
        }
        None
    }
}
impl Index {
    pub(crate) fn ordered_candidates(&self, from: Vec3, to: Vec3) -> OrderedCandidates {
        let mut out = OrderedCandidates {
            words: [0; crate::model::MAX_ENTITIES.div_ceil(64)],
            word: crate::model::MAX_ENTITIES.div_ceil(64),
            end: 0,
        };
        let mut candidates = self.candidates(from, to);
        candidates.sensory = 2;
        for (i, _) in candidates {
            let word = i / 64;
            out.words[word] |= 1u64 << (i % 64);
            out.word = out.word.min(word);
            out.end = out.end.max(word + 1);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geometry::{Geometry, Rise},
        model::{BodyKind, RoomRecipe},
    };

    // Critical: a missed broadphase leaf changes damage/visibility and replay.
    // The unchanged linear narrowphase is the oracle, including absent/moved bodies.
    #[test]
    fn conservative_candidates_match_exact_linear_hits_after_rebuilds() {
        let mut scene = RoomRecipe::default().generate().unwrap();
        let mut prototype = scene
            .entities
            .iter()
            .find(|e| e.body == Some(BodyKind::Fixed))
            .unwrap()
            .clone();
        scene.entities.clear();
        let rises = [
            Rise::PositiveX,
            Rise::NegativeX,
            Rise::PositiveZ,
            Rise::NegativeZ,
        ];
        for i in 0..4096 {
            prototype.id = format!("b{i:04}");
            prototype.transform.position =
                [(i % 64) as f32 * 1.5, (i % 3) as f32, (i / 64) as f32 * 1.5];
            prototype.transform.size = [0.5 + (i % 4) as f32, 0.5, 0.75];
            prototype.geometry = if i % 2 == 0 {
                Geometry::Box
            } else {
                Geometry::Wedge {
                    rise: rises[(i / 2) % 4],
                }
            };
            prototype.body = if i % 5 == 0 {
                None
            } else {
                Some(BodyKind::Fixed)
            };
            prototype.blocks_vision = Some(i % 7 != 0);
            scene.entities.push(prototype.clone());
        }
        let mut positions: BTreeMap<_, _> = scene
            .entities
            .iter()
            .map(|e| (e.id.clone(), e.transform.position))
            .collect();
        let mut index = Index::default();
        for phase in 0..4 {
            if phase == 1 {
                positions.retain(|id, _| id[1..].parse::<usize>().unwrap() % 3 != 0);
                for position in positions.values_mut() {
                    position[0] += 0.25;
                    position[1] -= 0.125;
                }
            } else if phase == 2 {
                for position in positions.values_mut() {
                    position[0] += 9000.;
                    position[2] -= 9000.;
                }
            } else if phase == 3 {
                positions.clear();
            }
            index.rebuild(&scene, &positions);
            let rays = (0..160).map(|i| {
                let from = Vec3::new(
                    (i % 64) as f32 * 1.5 - 0.5,
                    (i % 3) as f32 + 0.25,
                    (i / 2) as f32,
                );
                let delta = match i % 5 {
                    0 => Vec3::ZERO,
                    1 => Vec3::new(1e-8, 0., 150.),
                    2 => Vec3::new(150., 0., 0.),
                    3 => Vec3::new(-150., -0.5, -150.),
                    _ => Vec3::new(1.5, 0., 0.75),
                };
                let offset = if phase == 2 {
                    Vec3::new(9000., 0., -9000.)
                } else {
                    Vec3::ZERO
                };
                (from + offset, from + offset + delta)
            });
            for (from, to) in rays {
                let hit = |e: &crate::model::Entity, center| {
                    e.blocks_vision.unwrap_or(e.body.is_some())
                        && crate::perception::segment_intersects_entity(from, to, e, center)
                };
                let linear: Vec<_> = scene
                    .entities
                    .iter()
                    .enumerate()
                    .filter_map(|(i, e)| {
                        positions
                            .get(&e.id)
                            .filter(|&&p| hit(e, p.into()))
                            .map(|_| i)
                    })
                    .collect();
                let mut indexed: Vec<_> = index
                    .candidates(from, to)
                    .filter_map(|(i, center)| hit(&scene.entities[i], center).then_some(i))
                    .collect();
                indexed.sort_unstable();
                assert_eq!(indexed, linear, "phase{phase}, {from:?} -> {to:?}");
                let reference = crate::identity::Reference {
                    slot: 0,
                    generation: 1,
                    instance: 1,
                };
                let direction = (to - from).normalize_or_zero();
                let mut linear = crate::projectiles::Shot {
                    actor: 0,
                    launched_tick: 0,
                    position: from.into(),
                    direction: if direction == Vec3::ZERO {
                        Vec3::X.into()
                    } else {
                        direction.into()
                    },
                    speed: 100.,
                    half_extent: 0.01 + (from.x.abs() % 1.) * 0.5,
                    remaining_distance: from.distance(to).min(1.66),
                    cast: crate::abilities::Cast {
                        definition: 0,
                        origin: None,
                        id: 1,
                        request: None,
                        slot: 0,
                        target: None,
                        aim: None,
                        aim_observation: None,
                        started: 0,
                        execute_at: 0,
                        objective: None,
                        channel: None,
                        life: crate::abilities::LifeStamp {
                            source_instance: reference,
                            target_instance: None,
                            actor: 0,
                            target: None,
                        },
                    },
                };
                let mut indexed = linear;
                let hit = crate::projectiles::advance(&mut linear, &scene, &positions, usize::MAX);
                let indexed_hit = crate::projectiles::advance_with_spatial(
                    &mut indexed,
                    &scene,
                    &positions,
                    usize::MAX,
                    Some(&index),
                );
                assert_eq!(hit, indexed_hit, "sweep phase{phase}: {from:?} -> {to:?}");
                assert_eq!(linear, indexed, "sweep TOI/position changed");
            }
        }
        // A removed high entity returns only after an explicit rebuild.
        positions.insert("b4095".into(), [0., 0., 0.]);
        index.rebuild(&scene, &positions);
        assert_eq!(
            index
                .candidates(Vec3::ZERO, Vec3::ZERO)
                .map(|(i, _)| i)
                .collect::<Vec<_>>(),
            vec![4095]
        );
    }
}
