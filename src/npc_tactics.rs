//! Positional suggestions over a frozen cue and explicitly known static layout.
use crate::{
    combat_spatial::Index,
    geometry::Volume,
    model::{BodyKind, Entity, Result, Scene, invalid},
    npc_mind::Profile,
    perception::Observation,
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub range: f32,
    pub tolerance: f32,
    pub cover: f32,
    pub flank: f32,
    pub max_candidates: u8,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            range: 4.,
            tolerance: 0.5,
            cover: 0.5,
            flank: 0.5,
            max_candidates: 32,
        }
    }
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if !self.range.is_finite()
            || !(0.5..=32.).contains(&self.range)
            || !self.tolerance.is_finite()
            || !(0.1..=4.).contains(&self.tolerance)
            || [self.cover, self.flank]
                .iter()
                .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            || !(8..=64).contains(&self.max_candidates)
        {
            return Err(invalid(
                "Tactics require range [0.5,32], tolerance [0.1,4], cover/flank [0,1], max_candidates [8,64]",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Choice {
    pub position: [f32; 3],
    pub score: f32,
    pub range_error: f32,
    pub covered: bool,
    pub flank: f32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Selected,
    Thinking,
    NoSafeCandidate,
    MovementLocked,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub evidence: Observation,
    pub choice: Option<Choice>,
    pub samples: u8,
    pub eligible: u8,
    pub limited: bool,
    pub next_think_tick: u64,
    pub reason: Reason,
}
fn covered(scene: &Scene, index: &Index, from: Vec3, to: Vec3) -> bool {
    index.interior_candidates(from, to).any(|(i, p)| {
        let e = &scene.entities[i];
        e.body == Some(BodyKind::Fixed)
            && Volume::entity(e, p)
                .cast(from, to - from, Vec3::ZERO, true, 0.)
                .is_some()
    })
}
fn safe(
    scene: &Scene,
    index: &Index,
    ground: Option<&crate::ground_navigation::World>,
    e: &Entity,
    p: Vec3,
) -> bool {
    let half = Vec3::from(e.transform.size) * 0.5;
    if let Some(world) = ground.filter(|_| e.character_controller.is_some()) {
        let feet = (p - Vec3::Y * half.y).to_array();
        if e.npc
            .as_ref()
            .unwrap()
            .mobility
            .is_some_and(|m| m.fly_speed.is_some())
        {
            return world.free(e, feet);
        }
        return world.trace_limited(e, feet, feet, 1).status
            == crate::ground_navigation::Status::Arrived;
    }
    let floor = scene
        .entities
        .iter()
        .find(|v| v.id == e.npc.as_ref().unwrap().surface)
        .unwrap();
    let d = (p - Vec3::from(floor.transform.position)).abs();
    let size = Vec3::from(floor.transform.size) * 0.5;
    let flying = e
        .npc
        .as_ref()
        .unwrap()
        .mobility
        .is_some_and(|m| m.fly_speed.is_some());
    if !flying
        && (d.x + half.x > size.x
            || d.z + half.z > size.z
            || (p.y - half.y - floor.transform.position[1] - size.y).abs() > 0.12)
    {
        return false;
    }
    !index
        .sweep(p, Vec3::ZERO, half.max_element())
        .any(|(i, center)| {
            let solid = &scene.entities[i];
            solid.body == Some(BodyKind::Fixed)
                && (flying || solid.id != floor.id)
                && Volume::entity(solid, center)
                    .cast(
                        p,
                        Vec3::ZERO,
                        (half - Vec3::splat(0.005)).max(Vec3::splat(0.001)),
                        true,
                        0.,
                    )
                    .is_some()
        })
}
pub struct Context<'a> {
    pub scene: &'a Scene,
    pub index: &'a Index,
    pub ground: Option<&'a crate::ground_navigation::World>,
    pub entity: &'a Entity,
    pub position: Vec3,
    pub health: Option<crate::vitality::Health>,
    pub profile: Option<Profile>,
    pub tick: u64,
}
pub fn choose(
    context: Context<'_>,
    config: Config,
    evidence: Observation,
    previous: Option<State>,
) -> State {
    let Context {
        scene,
        index,
        ground,
        entity,
        position: own,
        health,
        profile,
        tick,
    } = context;
    let profile = profile.unwrap_or(Profile {
        intelligence: 5000,
        traits: [5000; 9],
    });
    if let Some(mut old) = previous.filter(|old| {
        old.evidence.instance == evidence.instance
            && old.evidence.life == evidence.life
            && old.evidence.noise_id == evidence.noise_id
            && old.evidence.source == evidence.source
            && tick < old.next_think_tick
    }) {
        old.reason = Reason::Thinking;
        return old;
    }
    let traits = profile.traits.map(|v| v as f32 / 10_000.);
    let injury = health.map_or(0., |h| 1. - h.units as f32 / h.maximum.max(1) as f32);
    let wanted =
        config.range * (1. + injury * (traits[1] + traits[3] - traits[4] * 0.5).max(0.) * 0.5);
    let cue = Vec3::from(evidence.position);
    let axis = (own - cue).with_y(0.).try_normalize().unwrap_or(Vec3::X);
    let right = Vec3::new(-axis.z, 0., axis.x);
    let budget = 8 + (config.max_candidates as usize - 8) * profile.intelligence as usize / 10_000;
    let map_budget = (budget / 2).min(scene.spatial_network.nodes.len());
    let ring_budget = (budget - map_budget - 1).min(16);
    let mut out = State {
        evidence,
        choice: None,
        samples: 0,
        eligible: 0,
        limited: scene.spatial_network.nodes.len() > map_budget,
        next_think_tick: tick + profile.think_interval(),
        reason: Reason::NoSafeCandidate,
    };
    for i in 0..=map_budget + ring_budget {
        let p = if i == 0 {
            own
        } else if i <= map_budget {
            // Spread effort over the known map rather than favoring its first IDs.
            let at = (i - 1) * scene.spatial_network.nodes.len() / map_budget;
            Vec3::from(scene.spatial_network.nodes[at])
                + Vec3::Y
                    * (entity.transform.size[1] * 0.5
                        + entity.character_controller.map_or(0., |c| c.offset))
        } else {
            let angle = (i - map_budget - 1) as f32 * std::f32::consts::TAU / ring_budget as f32;
            let p = cue + (axis * angle.cos() + right * angle.sin()) * wanted;
            p.with_y(own.y)
        };
        out.samples += 1;
        if !safe(scene, index, ground, entity, p) {
            continue;
        }
        out.eligible += 1;
        let distance = p.distance(cue);
        let error = ((distance - wanted).abs() - config.tolerance).max(0.);
        let cover = covered(scene, index, p, cue);
        let flank = 1. - (p - cue).with_y(0.).normalize_or_zero().dot(axis).abs();
        let score = -error * 2. - p.distance(own) * 0.12
            + if cover {
                config.cover * (traits[1] + traits[3] + injury - traits[4] * 0.5) * 4.
            } else {
                0.
            }
            + flank
                * config.flank
                * (traits[5] + traits[6] + profile.intelligence as f32 / 10_000.);
        let choice = Choice {
            position: p.into(),
            score,
            range_error: error,
            covered: cover,
            flank,
        };
        if out.choice.is_none_or(|old| score > old.score + 0.0001) {
            out.choice = Some(choice);
        }
    }
    if out.choice.is_some() {
        out.reason = Reason::Selected;
    }
    out
}
