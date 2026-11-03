//! Bounded shared action executor. Authoring is compiled once; no JSON in the tick.
use crate::{
    model::{BodyKind, Entity, Result, Scene, invalid},
    noise::Noise,
    perception::NpcState,
    targeting::{Attitude, Catalog as Actors},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
pub const RESOURCES: usize = 32;
pub const GRANTS: usize = 16;
pub const LIMIT: i64 = 1_000_000_000_000;
pub const MAX_INPUTS: usize = 4096;
pub const PER_ACTOR_PER_TICK: usize = 16;
pub const PER_TICK: usize = PER_ACTOR_PER_TICK * crate::targeting::MAX_ACTORS;
const NO_ENTRY: u16 = u16::MAX;
const _: () = assert!(PER_TICK < u16::MAX as usize);
pub fn input_capacity(actors: usize) -> usize {
    actors.min(crate::targeting::MAX_ACTORS) * PER_ACTOR_PER_TICK
}
const RECEIPTS: usize = 256;
const MAX_COSTS: usize = 8;
pub const NOISES_PER_SOURCE: usize = 16;
pub const RECENT_NOISES: usize = 128;
pub fn noise_capacity(actors: usize) -> usize {
    actors.min(crate::targeting::MAX_ACTORS) * NOISES_PER_SOURCE
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    #[serde(default, skip_serializing_if = "crate::attributes::Config::is_empty")]
    pub attributes: crate::attributes::Config,
    pub resources: Vec<Resource>,
    pub abilities: Vec<Definition>,
}
impl Catalog {
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty() && self.abilities.is_empty() && self.attributes.is_empty()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub scale: u32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Loadout {
    pub resources: Vec<Pool>,
    pub grants: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    pub resource: String,
    pub initial: i64,
    pub capacity: i64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Selection {
    SelfActor,
    Actor,
    Spatial,
}
/// Explicit spatial intent, independent of actor identity. Frozen when queued.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Aim {
    Direction { vector: [f32; 3] },
    Point { position: [f32; 3] },
}
impl Aim {
    pub fn validate(self) -> Result<()> {
        let values = match self {
            Self::Direction { vector } => vector,
            Self::Point { position } => position,
        };
        if values.iter().any(|v| !v.is_finite() || v.abs() > 10_000.)
            || matches!(self, Self::Direction { .. })
                && glam::Vec3::from(values).length_squared() < 0.000001
        {
            return Err(invalid(
                "Spatial aim requires finite components within +/-10000 and a nonzero direction (length >= 0.001)",
            ));
        }
        Ok(())
    }
    pub fn direction(self, origin: glam::Vec3) -> Option<glam::Vec3> {
        let delta = match self {
            Self::Direction { vector } => vector.into(),
            Self::Point { position } => glam::Vec3::from(position) - origin,
        };
        (delta.is_finite() && delta.length_squared() >= 0.000001).then(|| delta.normalize())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Recipient {
    Source,
    Target,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Amount {
    pub resource: String,
    pub units: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    Resurrect {
        recipient: Recipient,
        amount: i64,
    },
    ApplyBuff {
        recipient: Recipient,
        buff: String,
    },
    Interrupt {
        recipient: Recipient,
        require_active: bool,
    },
    Dispel {
        recipient: Recipient,
        category: String,
        max_instances: u8,
        require_match: bool,
    },
    DamageComponents {
        recipient: Recipient,
        components: Vec<crate::damage::Component>,
    },
    Damage {
        recipient: Recipient,
        amount: i64,
    },
    Heal {
        recipient: Recipient,
        amount: i64,
    },
    Resource {
        recipient: Recipient,
        resource: String,
        delta: i64,
    },
    Noise {
        radius: f32,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum CastMovement {
    Free {},
    Interrupt { speed_tolerance: f32 },
    Lock {},
}
impl Default for CastMovement {
    fn default() -> Self {
        Self::Free {}
    }
}
impl CastMovement {
    fn is_free(&self) -> bool {
        matches!(self, Self::Free {})
    }
    fn validate(self, version: u32) -> Result<()> {
        if !self.is_free() && version < 29 {
            return Err(invalid("Cast movement requires scene version 29"));
        }
        if let Self::Interrupt { speed_tolerance } = self
            && (!speed_tolerance.is_finite() || !(0.01..=2.).contains(&speed_tolerance))
        {
            return Err(invalid("Cast movement speed_tolerance must be 0.01..2m/s"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MotionSample {
    pub speed_squared: f32,
    pub requested: bool,
    pub lock_supported: bool,
}
impl Default for MotionSample {
    fn default() -> Self {
        Self {
            speed_squared: 0.,
            requested: false,
            lock_supported: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<crate::area::Area>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub interruptible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<crate::channel::Channel>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub dispellable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub periodic_category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub periodic: Option<crate::periodic::Schedule>,
    #[serde(
        default,
        skip_serializing_if = "crate::projectiles::Delivery::is_direct"
    )]
    pub delivery: crate::projectiles::Delivery,
    #[serde(default, skip_serializing_if = "CastMovement::is_free")]
    pub movement: CastMovement,
    pub id: String,
    pub selection: Selection,
    pub range: f32,
    pub relation: Option<Attitude>,
    pub cast_ticks: u32,
    pub cooldown_ticks: u32,
    pub costs: Vec<Amount>,
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub requires_below: Option<Amount>,
}
fn default_true() -> bool {
    true
}
fn is_true(value: &bool) -> bool {
    *value
}
fn label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-.".contains(&c))
}
fn categories(catalog: &Catalog) -> BTreeSet<&str> {
    catalog
        .abilities
        .iter()
        .flat_map(|d| {
            d.periodic_category
                .as_deref()
                .into_iter()
                .chain(d.effects.iter().filter_map(|e| {
                    if let Effect::Dispel { category, .. } = e {
                        Some(category.as_str())
                    } else {
                        None
                    }
                }))
        })
        .chain(
            catalog
                .attributes
                .buffs
                .iter()
                .filter_map(|b| b.category.as_deref()),
        )
        .collect()
}
pub fn validate(scene: &Scene) -> Result<()> {
    crate::attributes::validate(scene)?;
    crate::damage::validate(scene)?;
    let c = &scene.ability_catalog;
    if (!c.is_empty()
        || scene.entities.iter().any(|e| {
            e.abilities.is_some()
                || e.npc
                    .as_ref()
                    .and_then(|n| n.objectives.as_ref())
                    .is_some_and(|os| {
                        os.iter().any(|o| {
                            matches!(o.action, crate::objectives::Action::UseAbility { .. })
                        })
                    })
        }))
        && scene.version < 26
    {
        return Err(invalid("Abilities require scene version 26"));
    }
    if c.resources.len() > RESOURCES || c.abilities.len() > 64 {
        return Err(invalid(
            "Ability catalog exceeds 32 resources or 64 definitions",
        ));
    }
    let mut ids = BTreeSet::new();
    for r in &c.resources {
        if !label(&r.id) || !ids.insert(&r.id) || r.scale == 0 || r.scale > 1_000_000 {
            return Err(invalid("Resource requires unique ID and scale 1..1000000"));
        }
    }
    let resource = |id: &str| c.resources.iter().position(|r| r.id == id);
    let mut ids = BTreeSet::new();
    for d in &c.abilities {
        if !d.interruptible && scene.version < 37 {
            return Err(invalid("Interrupt protection requires scene version37"));
        }
        if let Some(area) = d.area {
            area.validate(
                scene.version,
                &d.effects,
                d.periodic.is_some() || d.channel.is_some(),
            )?;
            if area
                .persistence
                .is_some_and(|p| p.anchor == crate::persistent_area::Anchor::Source)
                && (d.selection != Selection::SelfActor || !d.delivery.is_direct())
            {
                return Err(invalid(
                    "Source-anchored area requires direct self_actor delivery",
                ));
            }
        }
        let resurrection = d
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::Resurrect { .. }))
            .count();
        if resurrection != 0 && (scene.version < 46 || resurrection != 1
            || d.selection != Selection::Actor || !d.delivery.is_direct()
            || d.area.is_some() || d.channel.is_some() || d.periodic.is_some()
            || d.effects.iter().any(|e| !matches!(e,
                Effect::Resurrect { recipient: Recipient::Target, amount } if (1..=LIMIT).contains(amount)
            ) && !matches!(e, Effect::Resource { recipient: Recipient::Source, .. } | Effect::Noise { .. }))) {
            return Err(invalid("Resurrection requires scene46, direct actor selection, one target resurrection and optional source resource/noise effects; no area/channel/periodic delivery"));
        }
        d.movement.validate(scene.version)?;
        d.delivery.validate(scene.version)?;
        if let Some(channel) = d.channel {
            channel.validate(scene.version)?;
            if !d.delivery.is_direct() || d.periodic.is_some() {
                return Err(invalid(
                    "Channel requires direct delivery without an independent periodic schedule",
                ));
            }
        }
        if !d.dispellable && (scene.version < 35 || d.periodic.is_none()) {
            return Err(invalid(
                "Non-dispellable effects require scene version35 and a periodic schedule",
            ));
        }
        if let Some(category) = &d.periodic_category
            && (scene.version < 34 || d.periodic.is_none() || !label(category))
        {
            return Err(invalid(
                "Periodic category requires version34, a schedule and a valid label",
            ));
        }
        if let Some(schedule) = d.periodic {
            schedule.validate(scene.version)?;
            if d.effects.iter().any(|e| {
                !matches!(
                    e,
                    Effect::DamageComponents {
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
                    "Periodic effects require only target damage/heal/resource effects",
                ));
            }
        }
        if d.selection == Selection::Spatial
            && (scene.version < 31
                || if d.delivery.is_direct() {
                    d.area.is_none() || d.range < 0.1
                } else {
                    d.range != 0.
                })
        {
            return Err(invalid(
                "Spatial selection requires projectile delivery with range=0, or instant direct area with range0.1..100",
            ));
        }
        if !d.delivery.is_direct()
            && (d.selection == Selection::SelfActor
                || d.effects.iter().any(|e| {
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
                }))
        {
            return Err(invalid(
                "Projectile delivery requires actor/spatial selection and only target damage/heal/resource/dispel/interrupt effects",
            ));
        }

        if !label(&d.id)
            || !ids.insert(&d.id)
            || !d.range.is_finite()
            || !(0.0..=100.).contains(&d.range)
            || d.cast_ticks > 3600
            || d.cooldown_ticks > 3600
            || d.costs.len() > MAX_COSTS
            || !(1..=8).contains(&d.effects.len())
        {
            return Err(invalid(
                "Invalid ability ID, range, ticks or effect/cost budget",
            ));
        }
        let mut sums = [0_i64; RESOURCES];
        for a in &d.costs {
            let Some(i) = resource(&a.resource) else {
                return Err(invalid("Unknown cost resource"));
            };
            if !(0..=LIMIT).contains(&a.units) {
                return Err(invalid("Cost must be 0..1000000000000 units"));
            }
            sums[i] += a.units;
            if sums[i] > LIMIT {
                return Err(invalid("Aggregated cost exceeds limit"));
            }
        }
        if let Some(a) = &d.requires_below
            && (resource(&a.resource).is_none() || !(1..=LIMIT).contains(&a.units))
        {
            return Err(invalid("Invalid own-resource threshold"));
        }
        for e in &d.effects {
            match e {
                Effect::ApplyBuff { buff, .. }
                    if scene.version < 38 || !c.attributes.buffs.iter().any(|b| b.id == *buff) =>
                {
                    return Err(invalid(
                        "Buff application requires version38 and a registered buff",
                    ));
                }
                Effect::Interrupt { recipient, .. }
                    if scene.version < 37
                        || *recipient != Recipient::Target
                        || (d.selection == Selection::SelfActor && d.area.is_none())
                        || d.channel.is_some() =>
                {
                    return Err(invalid(
                        "Interrupt requires version37, another target, and a non-channel delivery",
                    ));
                }
                Effect::Dispel {
                    category,
                    max_instances,
                    ..
                } if scene.version < 34
                    || !label(category)
                    || !(1..=64).contains(max_instances) =>
                {
                    return Err(invalid(
                        "Dispel requires version34, a valid category and max_instances1..64",
                    ));
                }
                Effect::Damage { amount, .. } | Effect::Heal { amount, .. }
                    if scene.version < 27 || !(1..=LIMIT).contains(amount) =>
                {
                    return Err(invalid(
                        "Damage/heal require version 27 and positive bounded units",
                    ));
                }
                Effect::Resource {
                    resource: id,
                    delta,
                    ..
                } if resource(id).is_none() || !(-LIMIT..=LIMIT).contains(delta) => {
                    return Err(invalid("Invalid resource effect"));
                }
                Effect::Noise { radius }
                    if !radius.is_finite() || !(0.1..=100.).contains(radius) =>
                {
                    return Err(invalid("Noise radius must be 0.1..100m"));
                }
                _ => {}
            }
        }
    }
    if categories(c).len() > 64 {
        return Err(invalid("Ability catalog exceeds 64 periodic categories"));
    }
    for e in &scene.entities {
        if let Some(l) = &e.abilities {
            if !matches!(e.body, Some(BodyKind::Npc | BodyKind::Player))
                || l.resources.len() > 16
                || l.grants.len() > GRANTS
            {
                return Err(invalid("Loadout requires actor, <=16 pools and grants"));
            }
            let mut pools = BTreeSet::new();
            for p in &l.resources {
                if resource(&p.resource).is_none()
                    || !pools.insert(&p.resource)
                    || !(0..=LIMIT).contains(&p.capacity)
                    || !(0..=p.capacity).contains(&p.initial)
                {
                    return Err(invalid("Invalid/duplicate pool or balance"));
                }
            }
            let mut grants = BTreeSet::new();
            for id in &l.grants {
                let Some(d) = c.abilities.iter().find(|d| d.id == *id) else {
                    return Err(invalid("Unknown granted ability"));
                };
                if !grants.insert(id) {
                    return Err(invalid("Duplicate grant"));
                }
                let has = |id: &str| l.resources.iter().any(|p| p.resource == id);
                if d.costs.iter().any(|a|!has(&a.resource))||d.requires_below.as_ref().is_some_and(|a|!has(&a.resource))||d.effects.iter().any(|x|matches!(x,Effect::Resource{recipient,resource,..} if (*recipient==Recipient::Source||(d.selection==Selection::SelfActor && d.area.is_none()))&&!has(resource))){return Err(invalid("Ability source lacks required pool"))}
            }
        }
        if let Some(os) = e.npc.as_ref().and_then(|n| n.objectives.as_ref()) {
            for o in os {
                if let crate::objectives::Action::UseAbility {
                    slot, policy: None, ..
                } = o.action
                    && e.abilities
                        .as_ref()
                        .and_then(|l| slot.checked_sub(1).and_then(|i| l.grants.get(i as usize)))
                        .and_then(|id| c.abilities.iter().find(|d| d.id == *id))
                        .is_some_and(|d| d.selection == Selection::Spatial)
                {
                    return Err(invalid(
                        "Automatic spatial use_ability requires a spatial_aim policy",
                    ));
                }

                if let crate::objectives::Action::UseAbility {
                    slot,
                    policy: Some(policy),
                    ..
                } = o.action
                {
                    let definition = e
                        .abilities
                        .as_ref()
                        .and_then(|l| slot.checked_sub(1).and_then(|i| l.grants.get(i as usize)))
                        .and_then(|id| c.abilities.iter().find(|d| d.id == *id))
                        .ok_or_else(|| invalid("Ability policy requires a granted slot"))?;
                    policy.validate(scene.version, definition.selection, definition.range)?;
                    if definition
                        .effects
                        .iter()
                        .any(|e| matches!(e, Effect::Resurrect { .. }))
                    {
                        return Err(invalid(
                            "Resurrection uses an explicit NPC target without an ability policy",
                        ));
                    }
                    if policy.require_dispel_match
                        && (definition.area.is_some()
                            || definition.channel.is_some()
                            || definition.periodic.is_some()
                            || !definition.effects.iter().any(|e| {
                                matches!(e,
                                Effect::Dispel { recipient, .. } if *recipient == Recipient::Target
                                    || definition.selection == Selection::SelfActor)
                            }))
                    {
                        return Err(invalid(
                            "Dispel policy requires a recipient dispel and no area/periodic/channel delivery",
                        ));
                    }

                    if policy.allies.is_some_and(|a| a.include_self)
                        && !definition.delivery.is_direct()
                    {
                        return Err(invalid("Ally triage include_self requires direct delivery"));
                    }
                    if definition.area.is_some()
                        && definition.delivery.is_direct()
                        && policy
                            .spatial_aim
                            .is_some_and(|aim| aim.range() > definition.range)
                    {
                        return Err(invalid(
                            "Spatial area policy range cannot exceed direct placement range",
                        ));
                    }
                }
                if let crate::objectives::Action::UseAbility {
                    slot, retry_ticks, ..
                } = o.action
                    && (slot == 0
                        || e.abilities
                            .as_ref()
                            .is_none_or(|l| slot as usize > l.grants.len())
                        || !(1..=3600).contains(&retry_ticks))
                {
                    return Err(invalid(
                        "Ability objective requires granted slot and retry 1..3600 ticks",
                    ));
                }
            }
        }
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Activate { slot: u8, target: Option<u16> },
    Aim { slot: u8, aim: Aim },
    Cancel { execution_id: u64 },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub life: LifeStamp,
    pub tick: u64,
    pub sequence: u64,
    pub actor: u16,
    pub operation: Operation,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifeStamp {
    pub source_instance: crate::identity::Reference,
    pub target_instance: Option<crate::identity::Reference>,
    pub actor: u64,
    pub target: Option<u64>,
}
/// Original observation that justified an automatic point, not a tracking target.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AimObservation {
    pub life: u64,
    pub actor: u16,
    pub tick: u64,
    pub source: crate::perception::Source,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Cast {
    pub definition: u8,
    pub origin: Option<u16>,
    pub life: LifeStamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<crate::channel::Progress>,
    pub id: u64,
    pub request: Option<u64>,
    pub slot: u8,
    pub target: Option<u16>,
    pub aim: Option<Aim>,
    pub aim_observation: Option<AimObservation>,
    pub started: u64,
    pub execute_at: u64,
    pub objective: Option<u8>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    OriginBudget,
    SourceUnavailable,
    InstanceChanged,
    LifeChanged,
    TargetAlive,
    LifeBudget,
    Started,
    ChannelStarted,
    ChannelPulse,
    Launched,
    Impacted,
    AreaApplied,
    AreaBudget,
    ProjectileBudget,
    PeriodicBudget,
    NoMatchingEffect,
    NoInterruptibleExecution,
    BuffBudget,
    InterruptedByAbility,
    PeriodicPulse,
    InvalidAim,
    Completed,
    Cancelled,
    InsufficientResource,
    ConditionFailed,
    Cooldown,
    Busy,
    TargetUnavailable,
    ActorUnavailable,
    OutOfRange,
    Occluded,
    RelationDenied,
    MissingResource,
    MissingVitality,
    TargetDead,
    ActorDead,
    CancelledByDeath,
    Moving,
    InterruptedByMovement,
    MovementLockUnavailable,
    NoiseBudget,
    NoActiveExecution,
}
/// Opt-in invocation outcomes. Delivery aftermath is represented by gameplay state.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct RequestResult {
    pub tick: u64,
    pub actor: crate::identity::Reference,
    pub request: u64,
    pub execution: Option<u64>,
    pub outcome: Outcome,
}
struct RequestTap {
    watched: BTreeSet<u64>,
    events: Vec<RequestResult>,
    dropped: u64,
    limit: usize,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DispelApplied {
    pub category: u8,
    pub removed: u8,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterruptStatus {
    Interrupted,
    NotCasting,
    Protected,
    AlreadySelected,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterruptApplied {
    pub status: InterruptStatus,
    pub execution_id: Option<u64>,
    pub channeling: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterruptedBy {
    pub actor_instance: crate::identity::Reference,
    pub actor: u16,
    pub execution_id: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Applied {
    pub actor_instance: crate::identity::Reference,
    pub buff: Option<u8>,
    pub damage_multiplier: Option<i64>,
    pub adjusted_amount: Option<i64>,
    pub interrupt: Option<InterruptApplied>,
    pub dispel: Option<DispelApplied>,
    pub actor: u16,
    pub resource: Option<u8>,
    pub requested: i64,
    pub applied: i64,
    pub noise_radius: Option<f32>,
    pub vitality: Option<crate::vitality::Applied>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub definition: Option<u8>,
    pub actor_instance: crate::identity::Reference,
    pub damage_components: [Option<crate::damage::Breakdown>; 8],
    pub interrupted_by: Option<InterruptedBy>,
    pub slot: Option<u8>,
    pub tick: u64,
    pub actor: u16,
    pub execution_id: Option<u64>,
    pub request: Option<u64>,
    pub outcome: Outcome,
    pub effects: [Option<Applied>; 8],
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct State {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) support_claims: Vec<crate::npc_cooperation::SupportLease>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) support_coordination: Vec<crate::npc_cooperation::SupportDecision>,
    pub origins: crate::origins::State,
    pub areas: crate::area::State,
    pub persistent_areas: crate::persistent_area::State,
    pub buffs: crate::attributes::State,
    pub interrupted_until: Vec<u64>,
    pub periodic: crate::periodic::State,
    pub projectiles: crate::projectiles::State,
    pub motion: Vec<MotionSample>,
    pub balances: Vec<[i64; RESOURCES]>,
    pub ready_at: Vec<[u64; GRANTS]>,
    pub casts: Vec<Option<Cast>>,
    pub retry_at: Vec<[u64; GRANTS]>,
    pub next_execution: u64,
    pub receipts: VecDeque<Receipt>,
    pub dropped_receipts: u64,
    pub noises: Vec<Noise>,
    pub recent_noises: VecDeque<Noise>,
    pub emitted: u64,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(remote = "State")]
        struct Wire {
            #[serde(default)]
            pub support_claims: Vec<crate::npc_cooperation::SupportLease>,
            #[serde(default)]
            pub support_coordination: Vec<crate::npc_cooperation::SupportDecision>,
            pub origins: crate::origins::State,
            pub areas: crate::area::State,
            pub persistent_areas: crate::persistent_area::State,
            pub buffs: crate::attributes::State,
            pub interrupted_until: Vec<u64>,
            pub periodic: crate::periodic::State,
            pub projectiles: crate::projectiles::State,
            pub motion: Vec<MotionSample>,
            pub balances: Vec<[i64; RESOURCES]>,
            pub ready_at: Vec<[u64; GRANTS]>,
            pub casts: Vec<Option<Cast>>,
            pub retry_at: Vec<[u64; GRANTS]>,
            pub next_execution: u64,
            pub receipts: VecDeque<Receipt>,
            pub dropped_receipts: u64,
            pub noises: Vec<Noise>,
            pub recent_noises: VecDeque<Noise>,
            pub emitted: u64,
        }
        let mut state = Wire::deserialize(d)?;
        if state.support_claims.len() > crate::targeting::MAX_ACTORS
            || state.support_coordination.len() > crate::targeting::MAX_ACTORS
        {
            return Err(serde::de::Error::custom(
                "Support coordination capacity exceeded",
            ));
        }
        let mut sources = crate::actor_set::ActorSet::EMPTY;
        for lease in &state.support_claims {
            if lease.group == 0
                || lease.actor as usize >= crate::targeting::MAX_ACTORS
                || lease.target as usize >= crate::targeting::MAX_ACTORS
                || sources.contains(lease.actor as usize)
            {
                return Err(serde::de::Error::custom(
                    "Invalid support lease owner/target",
                ));
            }
            sources.insert(lease.actor as usize);
        }
        if !state.origins.prepare_from_buffs(&state.buffs.active) {
            return Err(serde::de::Error::custom(
                "Invalid origin owner contribution budget",
            ));
        }
        Ok(state)
    }
}

#[derive(Clone, Copy)]
enum CompiledEffect {
    ApplyBuff {
        recipient: Recipient,
        buff: u8,
    },
    Interrupt {
        require_active: bool,
    },
    Dispel {
        recipient: Recipient,
        category: u8,
        max_instances: u8,
        require_match: bool,
    },
    Vital {
        packet: Option<u16>,
        recipient: Recipient,
        amount: i64,
        kind: crate::vitality::Change,
    },
    Resource {
        recipient: Recipient,
        index: usize,
        delta: i64,
    },
    Noise {
        radius: f32,
    },
}
struct Compiled {
    resurrect: bool,
    area: Option<crate::area::Area>,
    interruptible: bool,
    channel: Option<crate::channel::Channel>,
    dispellable: bool,
    periodic_category: Option<u8>,
    periodic: Option<crate::periodic::Schedule>,
    delivery: crate::projectiles::Delivery,
    movement: CastMovement,
    selection: Selection,
    range: f32,
    relation: Option<Attitude>,
    cast: u32,
    cooldown: u32,
    costs: [i64; RESOURCES],
    effects: Vec<CompiledEffect>,
    below: Option<(usize, i64)>,
}
type AutomaticIntention = (u8, u8, Option<u16>, u32);
#[derive(Default)]
struct FrameScratch {
    support_candidates: Vec<crate::npc_cooperation::SupportCandidate>,
    support_blocked: crate::actor_set::ActorSet,
    entry_heads: Vec<u16>,
    entry_tails: Vec<u16>,
    entry_next: Vec<u16>,
    lives: Vec<u64>,
    occupied: Vec<bool>,
    intentions: Vec<Option<AutomaticIntention>>,
    ready: Vec<u16>,
}
impl FrameScratch {
    fn new(count: usize) -> Self {
        Self {
            support_candidates: Vec::new(),
            support_blocked: crate::actor_set::ActorSet::EMPTY,
            entry_heads: vec![NO_ENTRY; count],
            entry_tails: vec![NO_ENTRY; count],
            entry_next: Vec::with_capacity(input_capacity(count)),
            lives: vec![0; count],
            occupied: vec![false; count],
            intentions: vec![None; count],
            ready: vec![0; count],
        }
    }
    fn index_entries(&mut self, entries: &[Entry]) {
        assert!(
            entries.len() <= input_capacity(self.entry_heads.len()),
            "validated frame input capacity"
        );
        self.entry_heads.fill(NO_ENTRY);
        self.entry_tails.fill(NO_ENTRY);
        self.entry_next.clear();
        self.entry_next.resize(entries.len(), NO_ENTRY);
        for (i, entry) in entries.iter().enumerate() {
            let actor = entry.actor as usize;
            let tail = self.entry_tails[actor];
            if tail == NO_ENTRY {
                self.entry_heads[actor] = i as u16;
            } else {
                self.entry_next[tail as usize] = i as u16;
            }
            self.entry_tails[actor] = i as u16;
        }
    }
}
pub struct Runtime {
    support_enabled: bool,
    areas_cleanup_needed: bool,
    noise_counts: Vec<u8>,
    outcome_counts: Vec<(Outcome, u64)>,
    request_tap: Option<RequestTap>,
    frame: FrameScratch,
    actor_instances: Vec<Option<crate::identity::Reference>>,
    damage_packets: Vec<crate::damage::Packet>,
    pub attributes: crate::attributes::Registry,
    pub state: State,
    definitions: Vec<Compiled>,
    categories: Vec<String>,
    grants: Vec<[Option<usize>; GRANTS]>,
    capacity: Vec<[i64; RESOURCES]>,
}
pub struct World<'a> {
    pub scene: &'a Scene,
    pub spatial: Option<&'a crate::combat_spatial::Index>,
    pub vitality: &'a mut crate::vitality::Runtime,
    pub actors: &'a Actors,
    pub positions: &'a BTreeMap<String, [f32; 3]>,
    pub npcs: &'a BTreeMap<String, NpcState>,
    pub tick: u64,
}
impl World<'_> {
    pub(crate) fn position(&self, entity: usize) -> Option<glam::Vec3> {
        if let Some(index) = self.spatial {
            index.position(entity)
        } else {
            self.positions
                .get(&self.scene.entities[entity].id)
                .copied()
                .map(Into::into)
        }
    }
}
#[derive(Default)]
struct DispelSelection {
    periodic: crate::transaction::IdSet<512>,
    buffs: crate::transaction::IdSet<512>,
}
struct DispelQuery {
    actor: u16,
    category: u8,
    max_instances: u8,
}

struct Plan {
    damage_components: [Option<crate::damage::Breakdown>; 8],
    buffs: [Option<crate::attributes::Install>; 8],
    buff_dispels: crate::transaction::IdSet<512>,
    interruption: Option<u64>,
    periodic_dispels: crate::transaction::IdSet<512>,
    vitality: crate::vitality::Changes,
    balances: crate::transaction::Writes<(usize, usize), i64, { MAX_COSTS + 8 }>,
    effects: [Option<Applied>; 8],
    noises: usize,
}
impl Runtime {
    pub fn new(scene: &Scene) -> Self {
        let actors = Actors::new(scene);
        Self::with_actors(scene, &actors)
    }
    pub(crate) fn with_actors(scene: &Scene, actors: &Actors) -> Self {
        let catalog = &scene.ability_catalog;
        let index = |id: &str| catalog.resources.iter().position(|r| r.id == id).unwrap();
        let categories: Vec<String> = categories(catalog).into_iter().map(str::to_owned).collect();
        let mut s = Self {
            support_enabled: scene.entities.iter().chain(&scene.npc_templates).any(|e| {
                e.npc
                    .as_ref()
                    .and_then(|n| n.cooperation)
                    .is_some_and(|c| c.reserve_support)
            }),
            areas_cleanup_needed: true,
            noise_counts: vec![0; actors.actors.len()],
            outcome_counts: Vec::with_capacity(64),
            request_tap: None,
            frame: FrameScratch::new(actors.actors.len()),
            actor_instances: (0..actors.actors.len())
                .map(|a| {
                    actors
                        .actors
                        .get(a)
                        .and_then(|&e| actors.entity_reference(e as usize))
                })
                .collect(),
            damage_packets: Vec::new(),
            attributes: crate::attributes::Registry::new(scene, &categories),
            state: State {
                support_claims: Vec::new(),
                support_coordination: Vec::new(),
                origins: crate::origins::State::new(actors.actors.len()),
                areas: crate::area::State::default(),
                persistent_areas: crate::persistent_area::State::default(),
                buffs: Default::default(),
                interrupted_until: vec![0; actors.actors.len()],
                periodic: Default::default(),
                projectiles: Default::default(),
                motion: vec![MotionSample::default(); actors.actors.len()],
                balances: vec![[0; RESOURCES]; actors.actors.len()],
                ready_at: vec![[0; GRANTS]; actors.actors.len()],
                casts: vec![None; actors.actors.len()],
                retry_at: vec![[0; GRANTS]; actors.actors.len()],
                next_execution: 1,
                receipts: VecDeque::with_capacity(RECEIPTS),
                dropped_receipts: 0,
                noises: Vec::with_capacity(noise_capacity(actors.actors.len())),
                recent_noises: VecDeque::with_capacity(RECENT_NOISES),
                emitted: 0,
            },
            definitions: Vec::with_capacity(catalog.abilities.len()),
            categories,
            grants: vec![[None; GRANTS]; actors.actors.len()],
            capacity: vec![[-1; RESOURCES]; actors.actors.len()],
        };
        let category_index =
            |name: &str| s.categories.iter().position(|v| v == name).unwrap() as u8;
        let mut packets = Vec::new();
        for d in &catalog.abilities {
            let mut costs = [0; RESOURCES];
            for a in &d.costs {
                costs[index(&a.resource)] += a.units;
            }
            s.definitions.push(Compiled {
                resurrect: d
                    .effects
                    .iter()
                    .any(|e| matches!(e, Effect::Resurrect { .. })),
                area: d.area,
                interruptible: d.interruptible,
                channel: d.channel,
                dispellable: d.dispellable,
                periodic_category: d.periodic_category.as_deref().map(category_index),
                periodic: d.periodic,
                delivery: d.delivery,
                movement: d.movement,
                selection: d.selection,
                range: d.range,
                relation: d.relation,
                cast: d.cast_ticks,
                cooldown: d.cooldown_ticks,
                costs,
                effects: d
                    .effects
                    .iter()
                    .map(|e| match e {
                        Effect::ApplyBuff { recipient, buff } => CompiledEffect::ApplyBuff {
                            recipient: *recipient,
                            buff: catalog
                                .attributes
                                .buffs
                                .iter()
                                .position(|b| b.id == *buff)
                                .unwrap() as u8,
                        },
                        Effect::Interrupt { require_active, .. } => CompiledEffect::Interrupt {
                            require_active: *require_active,
                        },
                        Effect::Dispel {
                            recipient,
                            category,
                            max_instances,
                            require_match,
                        } => CompiledEffect::Dispel {
                            recipient: *recipient,
                            category: category_index(category),
                            max_instances: *max_instances,
                            require_match: *require_match,
                        },
                        Effect::Damage { recipient, amount } => {
                            let packet = packets.len() as u16;
                            packets.push(crate::damage::Packet {
                                components: vec![crate::damage::CompiledComponent {
                                    amount: *amount,
                                    tags: 0,
                                }],
                            });
                            CompiledEffect::Vital {
                                packet: Some(packet),
                                recipient: *recipient,
                                amount: *amount,
                                kind: crate::vitality::Change::Damage,
                            }
                        }
                        Effect::DamageComponents {
                            recipient,
                            components,
                        } => {
                            let packet = packets.len() as u16;
                            let amount = components.iter().map(|c| c.amount).sum();
                            packets.push(crate::damage::Packet {
                                components: components
                                    .iter()
                                    .map(|c| crate::damage::CompiledComponent {
                                        amount: c.amount,
                                        tags: s.attributes.tag_mask(&c.tags),
                                    })
                                    .collect(),
                            });
                            CompiledEffect::Vital {
                                packet: Some(packet),
                                recipient: *recipient,
                                amount,
                                kind: crate::vitality::Change::Damage,
                            }
                        }
                        Effect::Resurrect { recipient, amount } => CompiledEffect::Vital {
                            packet: None,
                            recipient: *recipient,
                            amount: *amount,
                            kind: crate::vitality::Change::Resurrect,
                        },
                        Effect::Heal { recipient, amount } => CompiledEffect::Vital {
                            packet: None,
                            recipient: *recipient,
                            amount: *amount,
                            kind: crate::vitality::Change::Heal,
                        },
                        Effect::Resource {
                            recipient,
                            resource,
                            delta,
                        } => CompiledEffect::Resource {
                            recipient: *recipient,
                            index: index(resource),
                            delta: *delta,
                        },
                        Effect::Noise { radius } => CompiledEffect::Noise { radius: *radius },
                    })
                    .collect(),
                below: d
                    .requires_below
                    .as_ref()
                    .map(|a| (index(&a.resource), a.units)),
            });
        }
        s.damage_packets = packets;
        for (a, &entity) in actors.actors.iter().enumerate() {
            if let Some(l) = &scene.entities[entity as usize].abilities {
                for p in &l.resources {
                    s.capacity[a][index(&p.resource)] = p.capacity;
                    s.state.balances[a][index(&p.resource)] = p.initial;
                }
                for (slot, id) in l.grants.iter().enumerate() {
                    s.grants[a][slot] = catalog.abilities.iter().position(|d| d.id == *id);
                }
            }
        }
        s
    }
    pub(crate) fn install_actor(
        &mut self,
        scene: &Scene,
        actors: &Actors,
        actor: u16,
        template: usize,
    ) {
        let a = actor as usize;
        self.actor_instances[a] = Some(actors.reference(actor));
        self.state.casts[a] = None;
        self.state.balances[a] = [0; RESOURCES];
        self.state.ready_at[a] = [0; GRANTS];
        self.state.retry_at[a] = [0; GRANTS];
        self.state.interrupted_until[a] = 0;
        self.state.motion[a] = Default::default();
        self.grants[a] = [None; GRANTS];
        self.capacity[a] = [-1; RESOURCES];
        self.attributes
            .install_template(actors.entity(actor), template);
        if let Some(l) = &scene.entities[actors.entity(actor)].abilities {
            for p in &l.resources {
                let i = scene
                    .ability_catalog
                    .resources
                    .iter()
                    .position(|r| r.id == p.resource)
                    .unwrap();
                self.capacity[a][i] = p.capacity;
                self.state.balances[a][i] = p.initial;
            }
            for (slot, id) in l.grants.iter().enumerate() {
                self.grants[a][slot] = scene
                    .ability_catalog
                    .abilities
                    .iter()
                    .position(|d| d.id == *id);
            }
        }
    }
    pub fn validate_entry(&self, entry: Entry, actors: &Actors) -> Result<()> {
        if entry.actor as usize >= actors.actors.len() {
            return Err(invalid("Unknown ability actor"));
        }
        if !actors.matches(entry.actor, entry.life.source_instance) {
            return Err(invalid("Stale ability source instance"));
        }
        let expected_target = match entry.operation {
            Operation::Activate { target, .. } => Some(target.unwrap_or(entry.actor)),
            _ => None,
        };
        if expected_target.is_some_and(|a| a as usize >= actors.actors.len()) {
            return Err(invalid("Unknown ability target"));
        }
        if expected_target.and_then(|a| actors.entity_reference(actors.entity(a)))
            != entry.life.target_instance
        {
            return Err(invalid("Stale ability target instance"));
        }
        if let Operation::Aim { slot, aim } = entry.operation {
            aim.validate()?;
            let d = slot
                .checked_sub(1)
                .and_then(|i| self.grants[entry.actor as usize].get(i as usize))
                .copied()
                .flatten()
                .map(|i| &self.definitions[i])
                .ok_or_else(|| invalid("Unknown ability slot"))?;
            if d.selection != Selection::Spatial {
                return Err(invalid("Spatial aim requires spatial selection"));
            }
            if d.delivery.is_direct() && !matches!(aim, Aim::Point { .. }) {
                return Err(invalid(
                    "Direct area requires an explicit point, not a direction",
                ));
            }
        }
        if let Operation::Activate { slot, target } = entry.operation {
            if slot == 0
                || slot as usize > GRANTS
                || self.grants[entry.actor as usize][slot as usize - 1].is_none()
                || target.is_some_and(|t| t as usize >= actors.actors.len())
            {
                return Err(invalid("Unknown ability slot/target"));
            }
            let d =
                &self.definitions[self.grants[entry.actor as usize][slot as usize - 1].unwrap()];
            if d.selection == Selection::Spatial
                || (!d.delivery.is_direct() && target == Some(entry.actor))
                || (d.selection == Selection::Actor && target.is_none())
                || (d.selection == Selection::SelfActor && target.is_some_and(|t| t != entry.actor))
            {
                return Err(invalid("Selection does not match ability"));
            }
        }
        Ok(())
    }
    pub fn ready_masks(&self, tick: u64) -> Vec<u16> {
        let mut masks = vec![0; self.state.casts.len()];
        self.fill_ready_masks(tick, &mut masks);
        masks
    }
    pub(crate) fn fill_ready_masks(&self, tick: u64, masks: &mut [u16]) {
        masks.fill(0);
        if self.definitions.is_empty() {
            return;
        }
        let mut pending_support = crate::actor_set::ActorSet::EMPTY;
        for lease in &self.state.support_claims {
            if tick < lease.expires_at {
                pending_support.insert(lease.actor as usize);
            }
        }
        for (a, mask) in masks.iter_mut().enumerate() {
            for slot in 0..GRANTS {
                if let Some(d) = self.grants[a][slot] {
                    let running = self.state.casts[a]
                        .is_some_and(|c| c.objective.is_some() && c.slot as usize == slot);
                    if running
                        || (self.state.casts[a].is_none()
                            && tick >= self.state.ready_at[a][slot]
                            && !(pending_support.contains(a)
                                && Self::healing_definition(&self.definitions[d]))
                            && tick >= self.state.retry_at[a][slot]
                            && self.definitions[d]
                                .below
                                .is_none_or(|(r, v)| self.state.balances[a][r] < v))
                    {
                        *mask |= 1 << slot;
                    }
                }
            }
        }
    }
    pub(crate) fn policy_range(&self, actor: usize, slot: u8) -> f32 {
        self.definitions[self.grants[actor][slot as usize - 1].unwrap()].range
    }
    pub(crate) fn policy_offer(&self, actor: usize, slot: u8) -> crate::ability_policy::Offer {
        let slot = slot as usize - 1;
        let d = &self.definitions[self.grants[actor][slot].unwrap()];
        crate::ability_policy::Offer {
            selection: d.selection,
            range: d.range,
            relation: d.relation,
            affordable: self.state.casts[actor]
                .is_some_and(|c| c.slot as usize == slot && c.channel.is_some())
                || d.costs
                    .iter()
                    .zip(&self.state.balances[actor])
                    .all(|(&cost, &balance)| cost <= balance),
            cast_target: self.state.casts[actor]
                .filter(|c| c.objective.is_some() && c.slot as usize == slot)
                .and_then(|c| c.aim_observation.map(|o| o.actor).or(c.target)),
        }
    }
    /// Same bounded selection used by transaction planning. Does not mutate runtime.
    fn select_dispel(
        &self,
        _scene: &Scene,
        actors: &Actors,
        tick: u64,
        q: DispelQuery,
        selected: &mut DispelSelection,
    ) -> u8 {
        let mut removed = 0;
        for &index in self.state.periodic.active.target_positions(q.actor) {
            let effect = &self.state.periodic.active[index];
            if !selected.periodic.contains(effect.cast.id)
                && effect.cast.target == Some(q.actor)
                && effect
                    .cast
                    .life
                    .target_instance
                    .is_some_and(|r| actors.matches(q.actor, r))
                && (actors.matches(effect.actor, effect.cast.life.source_instance)
                    || self.retained_origin(effect.cast).is_some())
                && self.definitions[effect.definition as usize].periodic_category
                    == Some(q.category)
                && self
                    .source_value(
                        actors,
                        effect.actor as usize,
                        effect.cast,
                        crate::attributes::DISPELLABLE,
                        crate::attributes::Value::boolean(
                            self.definitions[effect.definition as usize].dispellable,
                        ),
                        self.attributes.context(effect.definition as usize),
                        tick,
                    )
                    .as_bool()
            {
                selected.periodic.insert(effect.cast.id);
                removed += 1;
                if removed == q.max_instances {
                    break;
                }
            }
        }
        let owner = actors.entity(q.actor) as u16;
        for &index in self.state.buffs.active.owner_indices(owner) {
            let instance = &self.state.buffs.active[index];
            if removed == q.max_instances {
                break;
            }
            if !selected.buffs.contains(instance.id)
                && tick < instance.expires_at
                && self.attributes.removable(
                    actors,
                    instance,
                    actors.entity(q.actor) as u16,
                    q.category,
                )
            {
                selected.buffs.insert(instance.id);
                removed += 1;
            }
        }
        removed
    }
    /// Caller must establish current visual evidence or self knowledge before querying.
    pub(crate) fn policy_dispel_match(
        &self,
        scene: &Scene,
        actors: &Actors,
        tick: u64,
        actor: u16,
        slot: u8,
        target: u16,
    ) -> bool {
        let d = &self.definitions[self.grants[actor as usize][slot as usize - 1].unwrap()];
        let mut selected = DispelSelection::default();
        let mut target_removed = 0;
        for effect in &d.effects {
            if let CompiledEffect::Dispel {
                recipient,
                category,
                max_instances,
                require_match,
            } = *effect
            {
                let owner = if recipient == Recipient::Source {
                    actor
                } else {
                    target
                };
                let removed = self.select_dispel(
                    scene,
                    actors,
                    tick,
                    DispelQuery {
                        actor: owner,
                        category,
                        max_instances,
                    },
                    &mut selected,
                );
                if require_match && removed == 0 {
                    return false;
                }
                if owner == target {
                    target_removed += u16::from(removed);
                }
            }
        }
        target_removed > 0
    }
    fn movement_failure(&self, actor: usize, slot: u8) -> Option<Outcome> {
        let movement = self.definitions[self.grants[actor][slot as usize].unwrap()].movement;
        let sample = self.state.motion[actor];
        match movement {
            CastMovement::Interrupt { speed_tolerance }
                if sample.requested || sample.speed_squared > speed_tolerance * speed_tolerance =>
            {
                Some(Outcome::Moving)
            }
            CastMovement::Lock {} if !sample.lock_supported => {
                Some(Outcome::MovementLockUnavailable)
            }
            _ => None,
        }
    }
    pub fn movement_locks(&self) -> crate::actor_set::ActorSet {
        let mut result = crate::actor_set::ActorSet::EMPTY;
        for (a, cast) in self.state.casts.iter().enumerate() {
            if cast.is_some_and(|c| {
                self.definitions[c.definition as usize].movement == (CastMovement::Lock {})
            }) {
                result.insert(a);
            }
        }
        result
    }
    /// Sampled at the ability boundary and again after resolved physical motion.
    pub fn interrupt_movement(
        &mut self,
        scene: &Scene,
        actors: &Actors,
        tick: u64,
    ) -> crate::actor_set::ActorSet {
        let mut interrupted = crate::actor_set::ActorSet::EMPTY;
        for a in 0..actors.actors.len() {
            if let Some(c) = self.state.casts[a]
                && let Some(reason) = self.movement_failure(a, c.slot)
            {
                self.state.casts[a] = None;
                self.defer_retry_at(scene, actors, tick, a, c);
                self.record(
                    tick,
                    a,
                    Some(c),
                    c.request,
                    if reason == Outcome::Moving {
                        Outcome::InterruptedByMovement
                    } else {
                        reason
                    },
                    [None; 8],
                );
                interrupted.insert(a);
            }
        }
        interrupted
    }
    fn record(
        &mut self,
        tick: u64,
        actor: usize,
        cast: Option<Cast>,
        request: Option<u64>,
        outcome: Outcome,
        effects: [Option<Applied>; 8],
    ) {
        if !matches!(
            outcome,
            Outcome::AreaApplied
                | Outcome::Impacted
                | Outcome::PeriodicPulse
                | Outcome::ChannelPulse
        ) && let Some(tap) = &mut self.request_tap
        {
            let original = cast.and_then(|c| c.request);
            for id in [request, original.filter(|id| Some(*id) != request)]
                .into_iter()
                .flatten()
            {
                if tap.watched.contains(&id) {
                    if tap.events.len() < tap.events.capacity() {
                        tap.events.push(RequestResult {
                            tick,
                            actor: cast.map_or_else(
                                || self.actor_instances[actor].unwrap(),
                                |c| c.life.source_instance,
                            ),
                            request: id,
                            execution: cast.map(|c| c.id),
                            outcome,
                        });
                    } else {
                        tap.dropped = tap.dropped.saturating_add(1);
                    }
                    if outcome != Outcome::Started {
                        tap.watched.remove(&id);
                    }
                }
            }
        }
        if let Some((_, count)) = self.outcome_counts.iter_mut().find(|(o, _)| *o == outcome) {
            *count = count.saturating_add(1);
        } else {
            self.outcome_counts.push((outcome, 1));
        }
        if self.state.receipts.len() == RECEIPTS {
            self.state.receipts.pop_front();
            self.state.dropped_receipts += 1;
        }
        self.state.receipts.push_back(Receipt {
            definition: cast.map(|c| c.definition),
            actor_instance: cast.map_or_else(
                || self.actor_instances[actor].unwrap(),
                |c| c.life.source_instance,
            ),
            damage_components: [None; 8],
            interrupted_by: None,
            slot: cast.map(|c| c.slot),
            tick,
            actor: actor as u16,
            execution_id: cast.map(|c| c.id),
            request,
            outcome,
            effects,
        });
    }
    fn reject(
        &mut self,
        tick: u64,
        actor: usize,
        slot: u8,
        request: Option<u64>,
        outcome: Outcome,
    ) {
        self.record(tick, actor, None, request, outcome, [None; 8]);
        self.state.receipts.back_mut().unwrap().slot = Some(slot);
        self.state.receipts.back_mut().unwrap().definition =
            self.grants[actor][slot as usize].map(|d| d as u8);
    }
    fn plan(&self, w: &World<'_>, a: usize, c: Cast) -> std::result::Result<Plan, Outcome> {
        self.validate_execution(w, a, c)?;
        let d = &self.definitions[c.definition as usize];
        let charge = c.channel.is_none()
            || d.channel.unwrap().cost_mode == crate::channel::CostMode::PerPulse;
        self.plan_effects(w, a, c, charge)
    }
    fn needs_origin(&self, c: Cast) -> bool {
        let d = &self.definitions[c.definition as usize];
        !d.delivery.is_direct()
            || d.periodic.is_some()
            || d.area.is_some_and(|a| a.persistence.is_some())
    }
    fn retained_origin(&self, c: Cast) -> Option<&crate::origins::Slot> {
        self.state
            .origins
            .get(c.origin, c.life.source_instance)
            .filter(|s| s.detached)
    }
    fn validate_delivery(
        &self,
        actors: &Actors,
        a: usize,
        c: Cast,
    ) -> std::result::Result<(), Outcome> {
        if !actors.matches(a as u16, c.life.source_instance) && self.retained_origin(c).is_none() {
            return Err(Outcome::InstanceChanged);
        }
        if c.target
            .is_some_and(|t| c.life.target_instance.is_none_or(|r| !actors.matches(t, r)))
        {
            return Err(Outcome::InstanceChanged);
        }
        Ok(())
    }
    fn source_dead(&self, w: &World<'_>, a: usize, c: Cast) -> bool {
        if w.actors.matches(a as u16, c.life.source_instance) {
            w.vitality.dead(a) || w.vitality.state.lives[a] != c.life.actor
        } else {
            self.retained_origin(c)
                .is_none_or(|s| s.dead || s.life != c.life.actor)
        }
    }
    fn source_attitude(
        &self,
        w: &World<'_>,
        a: usize,
        c: Cast,
        target: u16,
    ) -> crate::targeting::Attitude {
        if w.actors.matches(a as u16, c.life.source_instance) {
            w.actors.attitude(a as u16, target)
        } else {
            w.actors
                .attitude_from(self.retained_origin(c).and_then(|s| s.faction), target)
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn source_value(
        &self,
        actors: &Actors,
        a: usize,
        c: Cast,
        key: u8,
        fallback: crate::attributes::Value,
        context: crate::attributes::Context,
        tick: u64,
    ) -> crate::attributes::Value {
        if actors.matches(a as u16, c.life.source_instance) {
            self.attributes.evaluate(
                actors,
                &self.state.buffs,
                actors.entity(a as u16) as u16,
                key,
                fallback,
                context,
                tick,
            )
        } else {
            self.attributes.evaluate_source(
                &self
                    .retained_origin(c)
                    .expect("validated origin")
                    .attributes,
                key,
                fallback,
                context,
                tick,
            )
        }
    }
    fn validate_instances(
        &self,
        actors: &Actors,
        a: usize,
        c: Cast,
    ) -> std::result::Result<(), Outcome> {
        if !actors.matches(a as u16, c.life.source_instance) {
            return Err(Outcome::InstanceChanged);
        }
        let target = c.target.or(c.aim_observation.map(|o| o.actor));
        if target.is_some_and(|t| c.life.target_instance.is_none_or(|r| !actors.matches(t, r))) {
            return Err(Outcome::InstanceChanged);
        }
        Ok(())
    }
    fn validate_execution(
        &self,
        w: &World<'_>,
        a: usize,
        c: Cast,
    ) -> std::result::Result<(), Outcome> {
        self.validate_instances(w.actors, a, c)?;
        if self.needs_origin(c)
            && !self
                .state
                .origins
                .available(a as u16, c.life.source_instance)
        {
            return Err(Outcome::OriginBudget);
        }
        if w.vitality.dead(a) {
            return Err(Outcome::ActorDead);
        }
        if c.life.actor != w.vitality.state.lives[a] {
            return Err(Outcome::LifeChanged);
        }
        let d = &self.definitions[c.definition as usize];
        let source = w.actors.id(w.scene, a as u16);
        if w.scene.entities[w.actors.entity(a as u16)]
            .npc
            .as_ref()
            .is_some_and(|n| !n.enabled)
        {
            return Err(Outcome::ActorUnavailable);
        }
        if d.delivery.is_direct() {
            self.check_area_capacity(w, a, c)?;
        }
        if let Some(aim) = c.aim {
            if let Some(original) = c.aim_observation {
                let current = w
                    .npcs
                    .get(source)
                    .filter(|n| n.observed_this_tick)
                    .and_then(|n| n.memory)
                    .filter(|o| {
                        o.tick == w.tick
                            && o.source.is_visual()
                            && o.current_actor(w.actors) == Some(original.actor)
                    })
                    .ok_or(Outcome::TargetUnavailable)?;
                if original.life != w.vitality.state.lives[original.actor as usize] {
                    return Err(Outcome::LifeChanged);
                }
                if w.vitality.dead(original.actor as usize) {
                    return Err(Outcome::TargetDead);
                }
                let crate::objectives::Action::UseAbility {
                    policy: Some(policy),
                    ..
                } = w.scene.entities[w.actors.entity(a as u16)]
                    .npc
                    .as_ref()
                    .unwrap()
                    .objectives
                    .as_ref()
                    .unwrap()[c.objective.unwrap() as usize]
                    .action
                else {
                    unreachable!("validated automatic spatial policy")
                };
                let range = policy.range(d.range);
                if glam::Vec3::from(w.positions[source]).distance_squared(current.position.into())
                    > range * range
                {
                    return Err(Outcome::OutOfRange);
                }
                if d.relation
                    .is_some_and(|r| w.actors.attitude(a as u16, original.actor) != r)
                {
                    return Err(Outcome::RelationDenied);
                }
                if let Some(t) = policy.vitality_below {
                    let subject = if t.subject == Recipient::Source {
                        a
                    } else {
                        original.actor as usize
                    };
                    let health =
                        w.vitality.state.actors[subject].ok_or(Outcome::MissingVitality)?;
                    if health.units * 10000 >= health.maximum * i64::from(t.basis_points) {
                        return Err(Outcome::ConditionFailed);
                    }
                }
            }
            if d.delivery.is_direct() {
                let Aim::Point { position } = aim else {
                    return Err(Outcome::InvalidAim);
                };
                let from = glam::Vec3::from(w.positions[source]);
                let to = glam::Vec3::from(position);
                if from.distance_squared(to) > d.range * d.range {
                    return Err(Outcome::OutOfRange);
                }
                if d.below.is_some_and(|(r, v)| self.state.balances[a][r] >= v) {
                    return Err(Outcome::ConditionFailed);
                }
                return Ok(());
            }
            if aim.direction(w.positions[source].into()).is_none() {
                return Err(Outcome::InvalidAim);
            }
            if !self
                .state
                .projectiles
                .active
                .available(a as u16, self.actor_instances.len())
            {
                return Err(Outcome::ProjectileBudget);
            }
            if d.below.is_some_and(|(r, v)| self.state.balances[a][r] >= v) {
                return Err(Outcome::ConditionFailed);
            }
            return Ok(());
        }
        let target_slot = c.target.ok_or(Outcome::TargetUnavailable)?;
        let target = target_slot as usize;
        let target_id = w.actors.id(w.scene, target_slot);
        if a == target
            && d.selection == Selection::Actor
            && d.relation
                .is_some_and(|r| w.actors.attitude(a as u16, target_slot) != r)
        {
            return Err(Outcome::RelationDenied);
        }
        if a != target {
            // All NPC requests, including external control, retain the NPC's sensory restrictions.
            if let Some(n) = w.npcs.get(source)
                && if d.resurrect {
                    n.corpse_visual
                        .filter(|o| {
                            o.tick == w.tick
                                && o.current_actor(w.actors) == Some(target_slot)
                                && o.life == c.life.target
                        })
                        .is_none()
                } else {
                    n.current_visual(target_slot, w.tick, w.actors)
                        .filter(|o| o.life == c.life.target)
                        .is_none()
                }
            {
                return Err(Outcome::TargetUnavailable);
            }
            let from = glam::Vec3::from(w.positions[source]);
            let to = glam::Vec3::from(w.positions[target_id]);
            if from.distance_squared(to) > d.range * d.range {
                return Err(Outcome::OutOfRange);
            }
            if d.relation
                .is_some_and(|r| w.actors.attitude(a as u16, target_slot) != r)
            {
                return Err(Outcome::RelationDenied);
            }
            let blocks = |e: &Entity, center| {
                e.id != source
                    && e.id != target_id
                    && e.blocks_vision.unwrap_or(e.body.is_some())
                    && crate::perception::segment_intersects_entity(from, to, e, center)
            };
            let occluded = if let Some(index) = w.spatial {
                index
                    .candidates(from, to)
                    .any(|(entity, center)| blocks(&w.scene.entities[entity], center))
            } else {
                w.scene.entities.iter().any(|e| {
                    w.positions
                        .get(&e.id)
                        .is_some_and(|&center| blocks(e, center.into()))
                })
            };
            if occluded {
                return Err(Outcome::Occluded);
            }
        }
        if c.life.target != Some(w.vitality.state.lives[target]) {
            return Err(Outcome::LifeChanged);
        }
        if d.resurrect {
            if w.vitality.state.actors[target].is_none() {
                return Err(Outcome::MissingVitality);
            }
            if !w.vitality.dead(target) {
                return Err(Outcome::TargetAlive);
            }
            if w.vitality.state.lives[target] == u64::MAX {
                return Err(Outcome::LifeBudget);
            }
        } else if w.vitality.dead(target) {
            return Err(Outcome::TargetDead);
        }
        if d.below.is_some_and(|(r, v)| self.state.balances[a][r] >= v) {
            return Err(Outcome::ConditionFailed);
        }
        if !d.delivery.is_direct() {
            if !self
                .state
                .projectiles
                .active
                .available(a as u16, self.actor_instances.len())
            {
                return Err(Outcome::ProjectileBudget);
            }
            if a == target
                || glam::Vec3::from(w.positions[source])
                    .distance_squared(glam::Vec3::from(w.positions[target_id]))
                    < 0.000001
            {
                return Err(Outcome::InvalidAim);
            }
        }
        if let Some(i) = c.objective
            && let crate::objectives::Action::UseAbility {
                policy: Some(policy),
                ..
            } = w.scene.entities[w.actors.entity(a as u16)]
                .npc
                .as_ref()
                .unwrap()
                .objectives
                .as_ref()
                .unwrap()[i as usize]
                .action
        {
            if policy.require_dispel_match
                && !self.policy_dispel_match(
                    w.scene,
                    w.actors,
                    w.tick,
                    a as u16,
                    c.slot + 1,
                    target as u16,
                )
            {
                return Err(Outcome::NoMatchingEffect);
            }
            if (c.channel.is_some() || policy.allies.is_some())
                && let Some(t) = policy.vitality_below
            {
                let subject = if t.subject == Recipient::Source {
                    a
                } else {
                    target
                };
                let health = w.vitality.state.actors[subject].ok_or(Outcome::MissingVitality)?;
                if health.units * 10000 >= health.maximum * i64::from(t.basis_points) {
                    return Err(Outcome::ConditionFailed);
                }
            }
        }
        self.check_periodic_capacity(a, c, d.delivery.is_direct())?;
        Ok(())
    }
    fn check_periodic_capacity(
        &self,
        a: usize,
        c: Cast,
        install: bool,
    ) -> std::result::Result<(), Outcome> {
        let index = c.definition as usize;
        if install
            && let Some(schedule) = self.definitions[index].periodic
            && !self.state.periodic.available(
                a as u16,
                self.actor_instances.len(),
                index as u8,
                c.target.ok_or(Outcome::TargetUnavailable)?,
                schedule,
                c.life,
            )
        {
            return Err(Outcome::PeriodicBudget);
        }
        Ok(())
    }
    fn plan_effects(
        &self,
        w: &World<'_>,
        a: usize,
        c: Cast,
        charge: bool,
    ) -> std::result::Result<Plan, Outcome> {
        self.validate_delivery(w.actors, a, c)?;
        let definition = c.definition as usize;
        let d = &self.definitions[definition];
        let present = w.actors.matches(a as u16, c.life.source_instance);
        if !present
            && (charge
                || d.effects.iter().any(|e| {
                    matches!(
                        e,
                        CompiledEffect::ApplyBuff {
                            recipient: Recipient::Source,
                            ..
                        } | CompiledEffect::Dispel {
                            recipient: Recipient::Source,
                            ..
                        } | CompiledEffect::Vital {
                            recipient: Recipient::Source,
                            ..
                        } | CompiledEffect::Resource {
                            recipient: Recipient::Source,
                            ..
                        } | CompiledEffect::Noise { .. }
                    )
                }))
        {
            return Err(Outcome::SourceUnavailable);
        }
        let source = if present {
            w.actors.entity(a as u16) as u16
        } else {
            self.retained_origin(c).unwrap().entity
        };
        let mut p = Plan {
            damage_components: [None; 8],
            buffs: [None; 8],
            buff_dispels: Default::default(),
            interruption: None,
            periodic_dispels: Default::default(),
            vitality: Default::default(),
            balances: Default::default(),
            effects: [None; 8],
            noises: 0,
        };
        for r in 0..RESOURCES {
            if !charge {
                break;
            }
            if d.costs[r] == 0 {
                continue;
            }
            let balance = p
                .balances
                .get_or_insert((a, r), || self.state.balances[a][r]);
            if *balance < d.costs[r] {
                return Err(Outcome::InsufficientResource);
            }
            *balance -= d.costs[r];
        }
        if charge && (c.aim.is_some() || d.area.is_some()) {
            return Ok(p);
        }
        let target = c.target.ok_or(Outcome::TargetUnavailable)? as usize;
        let mut component_count = 0;
        for (i, e) in d.effects.iter().enumerate() {
            match *e {
                CompiledEffect::ApplyBuff { recipient, buff } => {
                    let actor = if recipient == Recipient::Source {
                        a
                    } else {
                        target
                    };
                    let install = crate::attributes::Install {
                        source_instance: c.life.source_instance,
                        owner_instance: w.actors.reference(actor as u16),
                        buff,
                        source,
                        owner: w.actors.entity(actor as u16) as u16,
                        execution_id: c.id,
                    };
                    if !self.attributes.available(
                        &self.state.buffs,
                        install,
                        &p.buffs,
                        &p.buff_dispels,
                    ) {
                        return Err(Outcome::BuffBudget);
                    }
                    p.buffs[i] = Some(install);
                    p.effects[i] = Some(Applied {
                        actor_instance: w.actors.reference(actor as u16),
                        buff: Some(buff),
                        damage_multiplier: None,
                        adjusted_amount: None,
                        interrupt: None,
                        dispel: None,
                        actor: actor as u16,
                        resource: None,
                        requested: 1,
                        applied: 1,
                        noise_radius: None,
                        vitality: None,
                    });
                }
                CompiledEffect::Interrupt { require_active } => {
                    let candidate = self.state.casts[target].as_ref().filter(|_| a != target);
                    let status = match candidate {
                        None => InterruptStatus::NotCasting,
                        Some(_) if p.interruption.is_some() => InterruptStatus::AlreadySelected,
                        Some(pending)
                            if !self.effective_flag(
                                w.scene,
                                w.actors,
                                target,
                                self.grants[target][pending.slot as usize].unwrap(),
                                crate::attributes::INTERRUPTIBLE,
                                w.tick,
                            ) =>
                        {
                            InterruptStatus::Protected
                        }
                        Some(_) => InterruptStatus::Interrupted,
                    };
                    let selected = status == InterruptStatus::Interrupted;
                    if require_active && !selected {
                        return Err(Outcome::NoInterruptibleExecution);
                    }
                    if selected {
                        p.interruption = candidate.map(|pending| pending.id);
                    }
                    p.effects[i] = Some(Applied {
                        actor_instance: w.actors.reference(target as u16),
                        buff: None,
                        damage_multiplier: None,
                        adjusted_amount: None,
                        interrupt: Some(InterruptApplied {
                            status,
                            execution_id: candidate.map(|pending| pending.id),
                            channeling: candidate.is_some_and(|pending| pending.channel.is_some()),
                        }),
                        dispel: None,
                        actor: target as u16,
                        resource: None,
                        requested: 1,
                        applied: i64::from(selected),
                        noise_radius: None,
                        vitality: None,
                    });
                }
                CompiledEffect::Dispel {
                    recipient,
                    category,
                    max_instances,
                    require_match,
                } => {
                    let actor = if recipient == Recipient::Source {
                        a
                    } else {
                        target
                    };
                    let mut selected = DispelSelection {
                        periodic: std::mem::take(&mut p.periodic_dispels),
                        buffs: std::mem::take(&mut p.buff_dispels),
                    };
                    let removed = self.select_dispel(
                        w.scene,
                        w.actors,
                        w.tick,
                        DispelQuery {
                            actor: actor as u16,
                            category,
                            max_instances,
                        },
                        &mut selected,
                    );
                    p.periodic_dispels = selected.periodic;
                    p.buff_dispels = selected.buffs;
                    if require_match && removed == 0 {
                        return Err(Outcome::NoMatchingEffect);
                    }
                    p.effects[i] = Some(Applied {
                        actor_instance: w.actors.reference(actor as u16),
                        buff: None,
                        damage_multiplier: None,
                        adjusted_amount: None,
                        interrupt: None,
                        dispel: Some(DispelApplied { category, removed }),
                        actor: actor as u16,
                        resource: None,
                        requested: i64::from(max_instances),
                        applied: i64::from(removed),
                        noise_radius: None,
                        vitality: None,
                    });
                }
                CompiledEffect::Vital {
                    packet,
                    recipient,
                    amount,
                    kind,
                } => {
                    let actor = if recipient == Recipient::Source {
                        a
                    } else {
                        target
                    };
                    let change = p
                        .vitality
                        .get_or_insert(actor, || crate::vitality::ChangeSet {
                            health: w.vitality.state.actors[actor],
                            cause: None,
                        });
                    let h = change.health.as_mut().ok_or(Outcome::MissingVitality)?;
                    let before = h.units;
                    let mut multiplier = None;
                    let adjusted = if let Some(packet) = packet {
                        let packet = &self.damage_packets[packet as usize];
                        let owner = w.actors.entity(actor as u16) as u16;
                        let mut total = 0_i64;
                        for (component_index, component) in packet.components.iter().enumerate() {
                            let m = self
                                .attributes
                                .evaluate(
                                    w.actors,
                                    &self.state.buffs,
                                    owner,
                                    crate::attributes::DAMAGE_RECEIVED,
                                    crate::attributes::Value::number(crate::attributes::SCALE),
                                    crate::attributes::Context {
                                        tags: component.tags,
                                        ability: Some(definition as u8),
                                    },
                                    w.tick,
                                )
                                .units();
                            let dealt = self
                                .source_value(
                                    w.actors,
                                    a,
                                    c,
                                    crate::attributes::DAMAGE_DEALT,
                                    crate::attributes::Value::number(crate::attributes::SCALE),
                                    crate::attributes::Context {
                                        tags: component.tags,
                                        ability: Some(definition as u8),
                                    },
                                    w.tick,
                                )
                                .units();
                            // Amount sum <= LIMIT, both factors <=100x: sum <=1e16.
                            // Round only once, preserving fractional outgoing damage until mitigation.
                            let adjusted =
                                (i128::from(component.amount) * i128::from(dealt) * i128::from(m)
                                    / (i128::from(crate::attributes::SCALE)
                                        * i128::from(crate::attributes::SCALE)))
                                    as i64;
                            total += adjusted;
                            p.damage_components[component_count] = Some(crate::damage::Breakdown {
                                effect_index: i as u8,
                                component_index: component_index as u8,
                                tags: component.tags,
                                requested: component.amount,
                                multiplier: m,
                                dealt_multiplier: dealt,
                                adjusted,
                            });
                            component_count += 1;
                            if packet.components.len() == 1 {
                                multiplier = Some(m);
                            }
                        }
                        total.min(LIMIT)
                    } else {
                        amount
                    };
                    let applied = match kind {
                        crate::vitality::Change::Damage => {
                            let v = adjusted.min(before);
                            h.units -= v;
                            if before > 0 && h.units == 0 {
                                change.cause = Some(i as u8);
                            }
                            v
                        }
                        crate::vitality::Change::Resurrect => {
                            if before != 0 {
                                return Err(Outcome::TargetAlive);
                            }
                            let v = amount.min(h.maximum);
                            h.units = v;
                            v
                        }
                        crate::vitality::Change::Heal => {
                            let v = amount.min(h.maximum - before);
                            h.units += v;
                            if h.units > 0 {
                                change.cause = None;
                            }
                            v
                        }
                    };
                    p.effects[i] = Some(Applied {
                        actor_instance: w.actors.reference(actor as u16),
                        buff: None,
                        damage_multiplier: multiplier,
                        adjusted_amount: packet.map(|_| adjusted),
                        interrupt: None,
                        dispel: None,
                        actor: actor as u16,
                        resource: None,
                        requested: amount,
                        applied,
                        noise_radius: None,
                        vitality: Some(crate::vitality::Applied {
                            kind,
                            before,
                            after: h.units,
                            excess: adjusted - applied,
                        }),
                    });
                }
                CompiledEffect::Resource {
                    recipient,
                    index,
                    delta,
                } => {
                    let actor = if recipient == Recipient::Source {
                        a
                    } else {
                        target
                    };
                    let capacity = self.capacity[actor][index];
                    if capacity < 0 {
                        return Err(Outcome::MissingResource);
                    }
                    let balance = p
                        .balances
                        .get_or_insert((actor, index), || self.state.balances[actor][index]);
                    let before = *balance;
                    let after = (before + delta).clamp(0, capacity);
                    *balance = after;
                    p.effects[i] = Some(Applied {
                        actor_instance: w.actors.reference(actor as u16),
                        buff: None,
                        damage_multiplier: None,
                        adjusted_amount: None,
                        interrupt: None,
                        dispel: None,
                        actor: actor as u16,
                        resource: Some(index as u8),
                        requested: delta,
                        applied: after - before,
                        noise_radius: None,
                        vitality: None,
                    });
                }
                CompiledEffect::Noise { radius } => {
                    p.noises += 1;
                    p.effects[i] = Some(Applied {
                        actor_instance: w.actors.reference(a as u16),
                        buff: None,
                        damage_multiplier: None,
                        adjusted_amount: None,
                        interrupt: None,
                        dispel: None,
                        actor: a as u16,
                        resource: None,
                        requested: 0,
                        applied: 0,
                        noise_radius: Some(radius),
                        vitality: None,
                    });
                }
            }
        }
        let delivering = c
            .channel
            .map_or(d.channel.is_none() && c.execute_at <= w.tick, |p| {
                w.tick >= p.next_tick
            });
        if delivering
            && (self.state.noises.len() + p.noises > noise_capacity(self.noise_counts.len())
                || self.noise_counts[a] as usize + p.noises > NOISES_PER_SOURCE
                || self
                    .state
                    .emitted
                    .checked_add(p.noises as u64)
                    .is_none_or(|n| n > crate::noise::MAX_SERIAL))
        {
            return Err(Outcome::NoiseBudget);
        }
        Ok(p)
    }
    fn advance_channel(&mut self, w: &mut World<'_>, a: usize, mut c: Cast) {
        if let Err(reason) = self.validate_execution(w, a, c) {
            self.complete(w, a, c, Err(reason));
            return;
        }
        let mut progress = c.channel.unwrap();
        if w.tick < progress.next_tick {
            return;
        }
        let channel = self.definitions[c.definition as usize].channel.unwrap();
        let plan = match self.plan_effects(
            w,
            a,
            c,
            channel.cost_mode == crate::channel::CostMode::PerPulse,
        ) {
            Ok(plan) => plan,
            Err(reason) => {
                self.complete(w, a, c, Err(reason));
                return;
            }
        };
        self.apply_effects(w, a, c, plan, Outcome::ChannelPulse);
        // A pulse may kill its own source, which already cancelled the execution.
        if self.state.casts[a].is_none() {
            return;
        }
        progress.remaining -= 1;
        if w.vitality.dead(c.target.unwrap() as usize) || progress.remaining == 0 {
            self.state.casts[a] = None;
            self.record(
                w.tick,
                a,
                Some(c),
                c.request,
                if w.vitality.dead(c.target.unwrap() as usize) {
                    Outcome::TargetDead
                } else {
                    Outcome::Completed
                },
                [None; 8],
            );
        } else {
            progress.next_tick += u64::from(channel.interval_ticks);
            c.channel = Some(progress);
            self.state.casts[a] = Some(c);
        }
    }
    fn finish(&mut self, w: &mut World<'_>, a: usize, c: Cast) {
        let result = self.plan(w, a, c);
        self.complete(w, a, c, result);
    }
    fn complete(
        &mut self,
        w: &mut World<'_>,
        a: usize,
        c: Cast,
        result: std::result::Result<Plan, Outcome>,
    ) {
        self.state.casts[a] = None;
        match result {
            Ok(p) => {
                let mut c = c;
                if self.needs_origin(c) {
                    c.origin = Some(
                        self.state
                            .origins
                            .reserve(a as u16, c.life.source_instance)
                            .expect("admission checked"),
                    );
                }
                if c.origin.is_some() {
                    self.state.origins.reserve_buffs(
                        c.life.source_instance,
                        self.state
                            .buffs
                            .active
                            .owner_count(w.actors.entity(a as u16) as u16),
                    );
                }
                let d = &self.definitions[c.definition as usize];
                self.state.ready_at[a][c.slot as usize] = w.tick + u64::from(d.cooldown);
                if let Some(channel) = d.channel {
                    if channel.cost_mode == crate::channel::CostMode::OnStart {
                        for (balance, cost) in self.state.balances[a].iter_mut().zip(&d.costs) {
                            *balance -= cost;
                        }
                    }
                    let running = Cast {
                        channel: Some(channel.start(w.tick)),
                        ..c
                    };
                    self.state.casts[a] = Some(running);
                    self.record(
                        w.tick,
                        a,
                        Some(running),
                        c.request,
                        Outcome::ChannelStarted,
                        [None; 8],
                    );
                } else if let crate::projectiles::Delivery::Projectile {
                    speed,
                    half_extent,
                    max_distance,
                } = d.delivery
                {
                    let from = glam::Vec3::from(w.positions[w.actors.id(w.scene, a as u16)]);
                    let direction = c.aim.map_or_else(
                        || {
                            (glam::Vec3::from(w.positions[w.actors.id(w.scene, c.target.unwrap())])
                                - from)
                                .normalize()
                        },
                        |aim| aim.direction(from).expect("plan validated aim"),
                    );
                    for (balance, cost) in self.state.balances[a].iter_mut().zip(&d.costs) {
                        *balance -= cost;
                    }
                    self.state
                        .projectiles
                        .active
                        .push(crate::projectiles::Shot {
                            actor: a as u16,
                            cast: c,
                            launched_tick: w.tick,
                            position: from.to_array(),
                            direction: direction.to_array(),
                            speed,
                            half_extent,
                            remaining_distance: max_distance,
                        });
                    self.record(w.tick, a, Some(c), c.request, Outcome::Launched, [None; 8]);
                } else if let Some(area) = d.area {
                    for &((actor, resource), value) in p.balances.iter() {
                        self.state.balances[actor][resource] = value;
                    }
                    let center = match c.aim {
                        Some(Aim::Point { position }) => position,
                        _ => w.positions[w.actors.id(w.scene, c.target.unwrap())],
                    };
                    self.record(w.tick, a, Some(c), c.request, Outcome::Completed, [None; 8]);
                    self.deliver_area(w, a, c, center, area)
                        .expect("direct capacity validated before payment");
                } else {
                    self.deliver_effects(w, a, c, p, Outcome::Completed);
                }
            }
            Err(reason) => {
                self.defer_retry(w, a, c);
                self.record(w.tick, a, Some(c), c.request, reason, [None; 8]);
            }
        }
    }
    fn deliver_effects(&mut self, w: &mut World<'_>, a: usize, c: Cast, p: Plan, outcome: Outcome) {
        let index = c.definition as usize;
        let d = &self.definitions[index];
        if let Some(schedule) = d.periodic {
            // The preview validated the whole pulse, but only the one-time charge commits now.
            if outcome == Outcome::Completed {
                for (balance, cost) in self.state.balances[a].iter_mut().zip(&d.costs) {
                    *balance -= cost;
                }
            }
            self.state
                .periodic
                .install(a as u16, index as u8, c, schedule, w.tick);
            self.record(w.tick, a, Some(c), c.request, outcome, [None; 8]);
        } else {
            self.apply_effects(w, a, c, p, outcome);
        }
    }
    fn advance_periodic(&mut self, w: &mut World<'_>) {
        use crate::periodic::Outcome as End;
        let mut retained = 0;
        for i in 0..self.state.periodic.active.len() {
            let mut effect = self.state.periodic.active[i];
            let mut end = None;
            if self
                .validate_delivery(w.actors, effect.actor as usize, effect.cast)
                .is_err()
            {
                end = Some(End::InstanceChanged);
            } else if w.vitality.dead(effect.cast.target.unwrap() as usize)
                || effect.cast.life.target
                    != Some(w.vitality.state.lives[effect.cast.target.unwrap() as usize])
            {
                end = Some(if w.vitality.dead(effect.cast.target.unwrap() as usize) {
                    End::TargetDead
                } else {
                    End::LifeChanged
                });
            } else if w.tick >= effect.next_tick {
                match self.plan_effects(w, effect.actor as usize, effect.cast, false) {
                    Ok(plan) => {
                        self.apply_effects(
                            w,
                            effect.actor as usize,
                            effect.cast,
                            plan,
                            Outcome::PeriodicPulse,
                        );
                        effect.remaining -= 1;
                        effect.next_tick += u64::from(effect.interval);
                        if w.vitality.dead(effect.cast.target.unwrap() as usize)
                            || effect.cast.life.target
                                != Some(
                                    w.vitality.state.lives[effect.cast.target.unwrap() as usize],
                                )
                        {
                            end = Some(if w.vitality.dead(effect.cast.target.unwrap() as usize) {
                                End::TargetDead
                            } else {
                                End::LifeChanged
                            });
                        } else if effect.remaining == 0 {
                            end = Some(End::Finished);
                        }
                    }
                    Err(reason) => {
                        self.record(
                            w.tick,
                            effect.actor as usize,
                            Some(effect.cast),
                            effect.cast.request,
                            reason,
                            [None; 8],
                        );
                        end = Some(End::EffectFailed);
                    }
                }
            }
            if let Some(reason) = end {
                self.state.periodic.record(effect, w.tick, reason, None);
            } else {
                self.state
                    .periodic
                    .active
                    .write_retained(retained, i, effect);
                retained += 1;
            }
        }
        self.state.periodic.active.finish_retained(retained);
    }
    fn remove_dead_periodic(&mut self, w: &World<'_>) {
        let mut retained = 0;
        for i in 0..self.state.periodic.active.len() {
            let effect = self.state.periodic.active[i];
            if self
                .validate_delivery(w.actors, effect.actor as usize, effect.cast)
                .is_err()
            {
                self.state.periodic.record(
                    effect,
                    w.tick,
                    crate::periodic::Outcome::InstanceChanged,
                    None,
                );
            } else if w.vitality.dead(effect.cast.target.unwrap() as usize)
                || effect.cast.life.target
                    != Some(w.vitality.state.lives[effect.cast.target.unwrap() as usize])
            {
                self.state.periodic.record(
                    effect,
                    w.tick,
                    if w.vitality.dead(effect.cast.target.unwrap() as usize) {
                        crate::periodic::Outcome::TargetDead
                    } else {
                        crate::periodic::Outcome::LifeChanged
                    },
                    None,
                );
            } else {
                self.state
                    .periodic
                    .active
                    .write_retained(retained, i, effect);
                retained += 1;
            }
        }
        self.state.periodic.active.finish_retained(retained);
    }
    fn apply_effects(
        &mut self,
        w: &mut World<'_>,
        a: usize,
        c: Cast,
        mut p: Plan,
        outcome: Outcome,
    ) {
        for install in p.buffs.iter().flatten() {
            let required = self.attributes.projected_owner_count(
                &self.state.buffs,
                &p.buffs,
                &p.buff_dispels,
                install.owner,
            );
            self.state
                .origins
                .reserve_buffs(install.owner_instance, required);
        }
        for &((actor, resource), value) in p.balances.iter() {
            self.state.balances[actor][resource] = value;
        }
        if !p.buff_dispels.is_empty() {
            self.state.buffs.remove(w.tick, |_, i| {
                p.buff_dispels.contains(i.id).then_some((
                    crate::attributes::End::Dispelled,
                    Some(crate::attributes::Responsible {
                        instance: c.life.source_instance,
                        entity: w.actors.entity(a as u16) as u16,
                        execution_id: c.id,
                    }),
                ))
            });
        }
        for install in p.buffs.into_iter().flatten() {
            self.attributes
                .install(&mut self.state.buffs, install, w.tick);
        }
        if let Some(expected) = p.interruption {
            let victim = c.target.expect("interrupt plan has a target") as usize;
            if let Some(pending) = self.state.casts[victim].filter(|pending| pending.id == expected)
            {
                self.state.casts[victim] = None;
                self.state.interrupted_until[victim] = w.tick + 1;
                self.defer_retry(w, victim, pending);
                self.record(
                    w.tick,
                    victim,
                    Some(pending),
                    pending.request,
                    Outcome::InterruptedByAbility,
                    [None; 8],
                );
                self.state.receipts.back_mut().unwrap().interrupted_by = Some(InterruptedBy {
                    actor_instance: c.life.source_instance,
                    actor: a as u16,
                    execution_id: c.id,
                });
            }
        }
        self.state.periodic.dispel(
            &p.periodic_dispels,
            w.tick,
            a as u16,
            c.id,
            c.life.source_instance,
        );
        w.vitality.commit(
            &mut p.vitality,
            w.tick,
            a as u16,
            c.id,
            w.actors,
            c.life.source_instance,
        );
        if p.vitality.iter().any(|(_, change)| change.cause.is_some()) {
            self.areas_cleanup_needed = true;
            self.clean_buffs(w);
        }
        for e in p.effects.iter().flatten() {
            if let Some(radius) = e.noise_radius {
                let noise = Noise {
                    emitter_instance: Some(c.life.source_instance),
                    id: crate::noise::event_id(crate::noise::Domain::Ability, self.state.emitted),
                    tick: w.tick + 1,
                    position: w.positions[w.actors.id(w.scene, a as u16)],
                    radius,
                    emitter_index: Some(w.actors.entity(a as u16) as u16),
                };
                self.state.emitted += 1;
                self.noise_counts[a] += 1;
                self.state.noises.push(noise);
                if self.state.recent_noises.len() == RECENT_NOISES {
                    self.state.recent_noises.pop_front();
                }
                self.state.recent_noises.push_back(noise);
            }
        }
        self.record(w.tick, a, Some(c), c.request, outcome, p.effects);
        self.state.receipts.back_mut().unwrap().damage_components = p.damage_components;
        for &(victim, _) in p.vitality.iter() {
            if w.vitality.dead(victim)
                && let Some(pending) = self.state.casts[victim].take()
            {
                self.record(
                    w.tick,
                    victim,
                    Some(pending),
                    pending.request,
                    Outcome::CancelledByDeath,
                    [None; 8],
                );
            }
        }
    }
    fn check_area_capacity(
        &self,
        w: &World<'_>,
        a: usize,
        c: Cast,
    ) -> std::result::Result<(), Outcome> {
        let definition = c.definition as usize;
        if let Some(schedule) = self.definitions[definition]
            .area
            .and_then(|area| area.persistence)
        {
            if schedule.end_on_source_death && self.source_dead(w, a, c) {
                return Err(Outcome::ActorDead);
            }
            let active = &self.state.persistent_areas.active;
            let own = active.source_positions(a as u16);
            let live = |i: &crate::persistent_area::Instance| {
                i.end(w.tick, self.source_dead(w, i.actor as usize, i.cast))
                    .is_none()
            };
            let matching = schedule.reapply == crate::periodic::Reapply::Refresh
                && own.iter().any(|&index| {
                    let i = &active[index];
                    i.cast.life.source_instance == c.life.source_instance
                        && i.definition as usize == definition
                        && live(i)
                });
            if !matching
                && own.iter().filter(|&&index| live(&active[index])).count()
                    >= crate::persistent_area::CAPACITY
            {
                return Err(Outcome::AreaBudget);
            }
        }
        Ok(())
    }
    fn clean_areas(&mut self, w: &World<'_>) {
        if !self.areas_cleanup_needed {
            return;
        }
        let mut retained = 0;
        for index in 0..self.state.persistent_areas.active.len() {
            let i = self.state.persistent_areas.active[index];
            if !w.actors.matches(i.actor, i.cast.life.source_instance)
                && (i.schedule().anchor == crate::persistent_area::Anchor::Source
                    || self.retained_origin(i.cast).is_none())
            {
                self.state.persistent_areas.record(
                    i,
                    w.tick,
                    crate::persistent_area::Outcome::SourceRemoved,
                    None,
                );
            } else if let Some(end) = i.end(w.tick, self.source_dead(w, i.actor as usize, i.cast)) {
                self.state.persistent_areas.record(i, w.tick, end, None);
            } else {
                if retained != index {
                    self.state
                        .persistent_areas
                        .active
                        .write_retained(retained, index, i);
                }
                retained += 1;
            }
        }
        self.state.persistent_areas.active.finish_retained(retained);
        self.areas_cleanup_needed = false;
    }
    fn advance_areas(&mut self, w: &mut World<'_>) {
        self.clean_areas(w);
        for index in 0..self.state.persistent_areas.active.len() {
            let mut i = self.state.persistent_areas.active[index];
            if i.end(w.tick, self.source_dead(w, i.actor as usize, i.cast))
                .is_some()
                || w.tick < i.next_tick
            {
                continue;
            }
            if i.schedule().anchor == crate::persistent_area::Anchor::Source {
                i.center = w.positions[w.actors.id(w.scene, i.actor)];
            }
            i.pulses += 1;
            i.next_tick += u64::from(i.schedule().interval_ticks);
            self.pulse_area(
                w,
                i.actor as usize,
                i.cast,
                i.center,
                i.area,
                Some(i.pulses),
            );
            self.state.persistent_areas.active.replace(index, i);
        }
        self.clean_areas(w);
    }
    fn deliver_area(
        &mut self,
        w: &mut World<'_>,
        a: usize,
        c: Cast,
        center: [f32; 3],
        area: crate::area::Area,
    ) -> std::result::Result<(), Outcome> {
        if !w.actors.matches(a as u16, c.life.source_instance) && self.retained_origin(c).is_none()
        {
            return Err(Outcome::InstanceChanged);
        }
        if let Some(schedule) = area.persistence {
            if schedule.anchor == crate::persistent_area::Anchor::Source
                && !w.actors.matches(a as u16, c.life.source_instance)
            {
                return Err(Outcome::SourceUnavailable);
            }
            self.check_area_capacity(w, a, c)?;
            self.clean_areas(w);
            let instance = crate::persistent_area::Instance {
                actor: a as u16,
                definition: c.definition as usize as u8,
                cast: c,
                center,
                area,
                created_tick: w.tick,
                expires_at: w.tick + u64::from(schedule.duration_ticks),
                next_tick: w.tick + u64::from(schedule.interval_ticks),
                pulses: u32::from(schedule.apply_on_spawn),
            };
            self.state.persistent_areas.install(instance);
            if schedule.apply_on_spawn {
                self.pulse_area(w, a, c, center, area, Some(1));
            }
        } else {
            self.pulse_area(w, a, c, center, area, None);
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn pulse_area(
        &mut self,
        w: &mut World<'_>,
        a: usize,
        c: Cast,
        center: [f32; 3],
        area: crate::area::Area,
        pulse: Option<u32>,
    ) {
        let relation = self.definitions[c.definition as usize].relation;
        let mut selection = crate::area::select_for_execution(
            w,
            c.life.source_instance,
            center,
            area,
            relation,
            |t| self.source_attitude(w, a, c, t),
        );
        // The list is frozen; each target is an independent transaction in this order.
        for candidate in &mut selection.selected {
            if candidate.status != crate::area::Status::Selected {
                continue;
            }
            let delivery = Cast {
                target: Some(candidate.actor),
                life: LifeStamp {
                    source_instance: c.life.source_instance,
                    target_instance: Some(w.actors.reference(candidate.actor)),
                    actor: c.life.actor,
                    target: Some(w.vitality.state.lives[candidate.actor as usize]),
                },
                ..c
            };
            match self.plan_effects(w, a, delivery, false) {
                Ok(p) => {
                    self.apply_effects(w, a, delivery, p, Outcome::AreaApplied);
                    candidate.status = crate::area::Status::Applied;
                }
                Err(reason) => {
                    candidate.status = crate::area::Status::EffectFailed;
                    candidate.failure = Some(reason);
                }
            }
        }
        self.state.areas.record(crate::area::Receipt {
            actor_instance: c.life.source_instance,
            pulse,
            actor: a as u16,
            slot: c.slot,
            execution_id: c.id,
            tick: w.tick,
            center,
            area,
            candidates: selection.finish(),
        });
    }
    fn advance_projectiles(&mut self, w: &mut World<'_>) {
        use crate::projectiles::{self, Outcome as End, Receipt};
        let mut retained = 0;
        for i in 0..self.state.projectiles.active.len() {
            let mut shot = self.state.projectiles.active[i];
            let source = if w.actors.matches(shot.actor, shot.cast.life.source_instance) {
                w.actors.entity(shot.actor)
            } else {
                usize::MAX
            };
            let contact = projectiles::advance_with_spatial(
                &mut shot,
                w.scene,
                w.positions,
                source,
                w.spatial,
            );
            let mut outcome = None;
            if let Some(entity) = contact {
                let target = w.actors.slot(entity as usize);
                let area = self.definitions[shot.cast.definition as usize].area;
                if let Some(area) = area {
                    outcome = Some(
                        match self.deliver_area(
                            w,
                            shot.actor as usize,
                            shot.cast,
                            shot.position,
                            area,
                        ) {
                            Ok(()) => End::Hit,
                            Err(reason) => {
                                self.record(
                                    w.tick,
                                    shot.actor as usize,
                                    Some(shot.cast),
                                    shot.cast.request,
                                    reason,
                                    [None; 8],
                                );
                                End::EffectFailed
                            }
                        },
                    );
                } else if target as usize >= w.actors.actors.len() {
                    outcome = Some(End::Blocked);
                } else if w.vitality.dead(target as usize) {
                    outcome = Some(End::TargetDead);
                } else {
                    let d = &self.definitions[shot.cast.definition as usize];
                    if d.relation.is_some_and(|r| {
                        self.source_attitude(w, shot.actor as usize, shot.cast, target) != r
                    }) {
                        outcome = Some(End::RelationDenied);
                    } else {
                        let impact = Cast {
                            target: Some(target),
                            life: LifeStamp {
                                source_instance: shot.cast.life.source_instance,
                                target_instance: Some(w.actors.reference(target)),
                                actor: shot.cast.life.actor,
                                target: Some(w.vitality.state.lives[target as usize]),
                            },
                            ..shot.cast
                        };
                        match self
                            .check_periodic_capacity(shot.actor as usize, impact, true)
                            .and_then(|()| self.plan_effects(w, shot.actor as usize, impact, false))
                        {
                            Ok(p) => {
                                self.deliver_effects(
                                    w,
                                    shot.actor as usize,
                                    impact,
                                    p,
                                    Outcome::Impacted,
                                );
                                outcome = Some(End::Hit);
                            }
                            Err(reason) => {
                                self.record(
                                    w.tick,
                                    shot.actor as usize,
                                    Some(impact),
                                    impact.request,
                                    reason,
                                    [None; 8],
                                );
                                outcome = Some(End::EffectFailed);
                            }
                        }
                    }
                }
            } else if shot.remaining_distance <= 0. {
                outcome = Some(End::Expired);
            }
            if let Some(outcome) = outcome {
                self.state.projectiles.record(Receipt {
                    actor_instance: shot.cast.life.source_instance,
                    entity_instance: contact.and_then(|e| w.actors.entity_reference(e as usize)),
                    execution_id: shot.cast.id,
                    actor: shot.actor,
                    tick: w.tick,
                    position: shot.position,
                    entity: contact,
                    outcome,
                });
            } else {
                self.state
                    .projectiles
                    .active
                    .write_retained(retained, i, shot);
                retained += 1;
            }
        }
        self.state.projectiles.active.finish_retained(retained);
    }
    fn defer_retry(&mut self, w: &World<'_>, a: usize, c: Cast) {
        self.defer_retry_at(w.scene, w.actors, w.tick, a, c);
    }
    fn defer_retry_at(&mut self, scene: &Scene, actors: &Actors, tick: u64, a: usize, c: Cast) {
        if let Some(i) = c.objective
            && let Some(config) = scene.entities[actors.entity(a as u16)]
                .npc
                .as_ref()
                .and_then(|n| n.objectives.as_ref())
            && let crate::objectives::Action::UseAbility { retry_ticks, .. } =
                config[i as usize].action
        {
            self.state.retry_at[a][c.slot as usize] = tick + u64::from(retry_ticks);
        }
    }
    fn clean_buffs(&mut self, w: &World<'_>) {
        self.state.buffs.remove(w.tick, |_, i| {
            if w.tick >= i.expires_at {
                Some((crate::attributes::End::Expired, None))
            } else if !w.actors.entity_matches(i.owner, i.owner_instance) {
                Some((crate::attributes::End::OwnerRemoved, None))
            } else if w.vitality.dead(w.actors.slot(i.owner as usize) as usize) {
                Some((crate::attributes::End::OwnerDead, None))
            } else {
                None
            }
        });
    }
    pub fn effective_flag(
        &self,
        _scene: &Scene,
        actors: &Actors,
        actor: usize,
        definition: usize,
        key: u8,
        tick: u64,
    ) -> bool {
        let d = &self.definitions[definition];
        let base = if key == crate::attributes::INTERRUPTIBLE {
            d.interruptible
        } else {
            d.dispellable
        };
        self.attributes
            .evaluate(
                actors,
                &self.state.buffs,
                actors.entity(actor as u16) as u16,
                key,
                crate::attributes::Value::boolean(base),
                self.attributes.context(definition),
                tick,
            )
            .as_bool()
    }
    pub fn step(&mut self, mut w: World<'_>, entries: &[Entry]) {
        self.areas_cleanup_needed = true;
        if let Some(tap) = &mut self.request_tap {
            tap.events.clear();
        }
        // Receipts without a Cast still need the current actor's identity.
        for (a, reference) in self.actor_instances.iter_mut().enumerate() {
            *reference = w.actors.entity_reference(w.actors.actors[a] as usize);
        }
        self.clean_buffs(&w);
        // Previous noises have already been consumed by this tick's sensors.
        self.state.noises.clear();
        if self.support_enabled {
            let count = w.actors.actors.len();
            self.frame
                .support_candidates
                .reserve(count.saturating_sub(self.frame.support_candidates.len()));
            self.state
                .support_claims
                .reserve(count.saturating_sub(self.state.support_claims.len()));
            self.state
                .support_coordination
                .reserve(count.saturating_sub(self.state.support_coordination.len()));
            self.clean_support(&w);
        }
        self.state
            .noises
            .reserve(noise_capacity(self.noise_counts.len()));
        self.noise_counts.fill(0);
        if self.definitions.is_empty() && entries.is_empty() {
            return;
        }
        let mut frame = std::mem::take(&mut self.frame);
        frame.lives.copy_from_slice(&w.vitality.state.lives);
        frame.occupied.fill(false);
        frame.intentions.fill(None);
        self.fill_ready_masks(w.tick, &mut frame.ready);
        frame.index_entries(entries);
        // Gather all automatic intentions before any commit changes balances.
        for (a, &entity) in w.actors.actors.iter().enumerate() {
            let e = &w.scene.entities[entity as usize];
            if !w.positions.contains_key(&e.id) {
                continue;
            }
            if let Some(n) = &e.npc {
                let state = &w.npcs[&e.id];
                if n.enabled
                    && !w.vitality.dead(a)
                    && !state.territory.blocked()
                    && let Some(i) = state.objectives.active
                    && let crate::objectives::Action::UseAbility {
                        slot,
                        retry_ticks,
                        policy,
                    } = n.objectives.as_ref().unwrap()[i as usize].action
                    && frame.ready[a] & (1 << (slot - 1)) != 0
                    && (policy.is_none()
                        || state.objectives.ability_decisions[i as usize].execute())
                {
                    let d = &self.definitions[self.grants[a][slot as usize - 1].unwrap()];
                    let target = if d.selection == Selection::SelfActor {
                        Some(a as u16)
                    } else if d.resurrect {
                        state
                            .corpse_visual
                            .filter(|o| o.tick == w.tick)
                            .and_then(|o| o.current_actor(w.actors))
                    } else if policy.is_some_and(|p| p.allies.is_some()) {
                        state.objectives.ability_decisions[i as usize].target
                    } else {
                        state
                            .memory
                            .filter(|m| {
                                state.observed_this_tick && m.tick == w.tick && m.source.is_visual()
                            })
                            .and_then(|m| m.current_actor(w.actors))
                    };
                    frame.intentions[a] = Some((i, slot - 1, target, retry_ticks));
                }
            }
        }
        for entry in entries {
            if let Operation::Cancel { execution_id } = entry.operation {
                let a = entry.actor as usize;
                if !w.actors.matches(entry.actor, entry.life.source_instance) {
                    let current_instance = self.actor_instances[a];
                    self.actor_instances[a] = Some(entry.life.source_instance);
                    self.record(
                        w.tick,
                        a,
                        None,
                        Some(entry.sequence),
                        Outcome::InstanceChanged,
                        [None; 8],
                    );
                    self.state.receipts.back_mut().unwrap().actor_instance =
                        entry.life.source_instance;
                    self.actor_instances[a] = current_instance;
                    continue;
                }
                frame.occupied[a] = true;
                let c = self.state.casts[a].filter(|c| c.id == execution_id);
                if let Some(c) = c {
                    self.defer_retry(&w, a, c);
                    self.state.casts[a] = None;
                }
                self.record(
                    w.tick,
                    a,
                    c,
                    Some(entry.sequence),
                    if c.is_some() {
                        Outcome::Cancelled
                    } else {
                        Outcome::NoActiveExecution
                    },
                    [None; 8],
                );
            }
        }
        self.advance_periodic(&mut w);
        self.advance_areas(&mut w);
        self.advance_projectiles(&mut w);
        self.collect_origins();
        let interrupted = self.interrupt_movement(w.scene, w.actors, w.tick);
        for a in w
            .actors
            .order()
            .iter()
            .copied()
            .take(w.actors.actors.len())
            .map(usize::from)
        {
            frame.occupied[a] |= interrupted.contains(a);
            if let Some(c) = self.state.casts[a] {
                if c.objective.is_some()
                    && frame.intentions[a]
                        .is_none_or(|(i, slot, _, _)| c.objective != Some(i) || slot != c.slot)
                {
                    self.state.casts[a] = None;
                    self.defer_retry(&w, a, c);
                    frame.occupied[a] = true;
                    self.record(w.tick, a, Some(c), c.request, Outcome::Cancelled, [None; 8]);
                } else if c.channel.is_some() {
                    self.advance_channel(&mut w, a, c);
                    frame.occupied[a] = true;
                } else if w.tick >= c.execute_at {
                    self.finish(&mut w, a, c);
                    frame.occupied[a] = true;
                }
            }
        }
        self.schedule_support(&w, &mut frame);
        for a in w
            .actors
            .order()
            .iter()
            .copied()
            .take(w.actors.actors.len())
            .map(usize::from)
        {
            let automatic = frame.intentions[a].filter(|_| !frame.support_blocked.contains(a));
            let mut external = false;
            let mut cursor = frame.entry_heads[a];
            while cursor != NO_ENTRY {
                let index = cursor as usize;
                cursor = frame.entry_next[index];
                let e = &entries[index];
                if matches!(e.operation, Operation::Cancel { .. }) {
                    continue;
                }
                if !w.actors.matches(e.actor, e.life.source_instance) {
                    let current = self.actor_instances[a];
                    self.actor_instances[a] = Some(e.life.source_instance);
                    self.record(
                        w.tick,
                        a,
                        None,
                        Some(e.sequence),
                        Outcome::InstanceChanged,
                        [None; 8],
                    );
                    self.actor_instances[a] = current;
                    continue;
                }
                if let Operation::Aim { slot, aim } = e.operation {
                    external = true;
                    self.start(
                        &mut w,
                        a,
                        slot - 1,
                        None,
                        Some(aim),
                        Some(e.sequence),
                        None,
                        e.life,
                        frame.occupied[a],
                    );
                    frame.occupied[a] = true;
                }
                if let Operation::Activate { slot, target } = e.operation {
                    external = true;
                    let target = target.unwrap_or(a as u16);
                    self.start(
                        &mut w,
                        a,
                        slot - 1,
                        Some(target),
                        None,
                        Some(e.sequence),
                        None,
                        e.life,
                        frame.occupied[a],
                    );
                    frame.occupied[a] = true;
                }
            }
            if !external
                && !frame.occupied[a]
                && self.state.casts[a].is_none()
                && let Some((objective, slot, target, retry)) = automatic
            {
                self.state.retry_at[a][slot as usize] = w.tick + u64::from(retry);
                let life = LifeStamp {
                    source_instance: w.actors.reference(a as u16),
                    target_instance: target.map(|t| w.actors.reference(t)),
                    actor: frame.lives[a],
                    target: target.map(|t| frame.lives[t as usize]),
                };
                let support = target.and_then(|target| {
                    let configuration = w.scene.entities[w.actors.entity(a as u16)]
                        .npc
                        .as_ref()?
                        .cooperation
                        .filter(|c| c.reserve_support)?;
                    Self::healing_definition(
                        &self.definitions[self.grants[a][slot as usize].unwrap()],
                    )
                    .then_some((configuration, target))
                });
                let execution = self.state.next_execution;
                self.start(
                    &mut w,
                    a,
                    slot,
                    target,
                    None,
                    None,
                    Some(objective),
                    life,
                    false,
                );
                if self.state.next_execution != execution
                    && let Some((configuration, target)) = support
                {
                    self.state
                        .support_claims
                        .push(crate::npc_cooperation::SupportLease {
                            group: configuration.group,
                            actor: a as u16,
                            source: life.source_instance,
                            source_life: life.actor,
                            target,
                            target_instance: life.target_instance.unwrap(),
                            target_life: life.target.unwrap(),
                            execution,
                            expires_at: w
                                .tick
                                .saturating_add(u64::from(configuration.support_lease_ticks)),
                        });
                }
            }
        }
        self.remove_dead_periodic(&w);
        self.clean_areas(&w);
        self.collect_origins();
        if self.support_enabled {
            self.clean_support(&w);
        }
        self.frame = frame;
    }
    #[allow(clippy::too_many_arguments)]
    fn start(
        &mut self,
        w: &mut World<'_>,
        a: usize,
        slot: u8,
        target: Option<u16>,
        aim: Option<Aim>,
        request: Option<u64>,
        objective: Option<u8>,
        life: LifeStamp,
        occupied: bool,
    ) {
        let reason = if !w.actors.matches(a as u16, life.source_instance) {
            Some(Outcome::InstanceChanged)
        } else if w.vitality.dead(a) {
            Some(Outcome::ActorDead)
        } else if life.actor != w.vitality.state.lives[a] {
            Some(Outcome::LifeChanged)
        } else if occupied
            || self.state.casts[a].is_some()
            || w.tick < self.state.interrupted_until[a]
        {
            Some(Outcome::Busy)
        } else if w.tick < self.state.ready_at[a][slot as usize] {
            Some(Outcome::Cooldown)
        } else if target.is_none() && aim.is_none() {
            Some(Outcome::TargetUnavailable)
        } else {
            self.movement_failure(a, slot)
        };
        if let Some(reason) = reason {
            let current_instance = self.actor_instances[a];
            self.actor_instances[a] = Some(life.source_instance);
            self.reject(w.tick, a, slot, request, reason);
            self.state.receipts.back_mut().unwrap().actor_instance = life.source_instance;
            self.actor_instances[a] = current_instance;
            if reason == Outcome::InstanceChanged {
                self.state.receipts.back_mut().unwrap().definition = None;
            }
            return;
        }
        let d = &self.definitions[self.grants[a][slot as usize].unwrap()];
        let mut aim_observation = None;
        let (target, aim) = if d.selection == Selection::Spatial && objective.is_some() {
            let observed = w
                .npcs
                .get(w.actors.id(w.scene, a as u16))
                .filter(|n| n.observed_this_tick)
                .and_then(|n| n.memory)
                .filter(|o| {
                    o.tick == w.tick && o.source.is_visual() && o.current_actor(w.actors) == target
                });
            let Some(observed) = observed else {
                self.reject(w.tick, a, slot, request, Outcome::TargetUnavailable);
                return;
            };
            aim_observation = Some(AimObservation {
                life: observed.life.expect("visual life stamp"),
                actor: target.unwrap(),
                tick: observed.tick,
                source: observed.source,
            });
            (
                None,
                Some(Aim::Point {
                    position: observed.position,
                }),
            )
        } else {
            (target, aim)
        };
        let c = Cast {
            definition: self.grants[a][slot as usize].unwrap() as u8,
            origin: None,
            life,
            channel: None,
            id: self.state.next_execution,
            request,
            slot,
            target,
            aim,
            aim_observation,
            started: w.tick,
            execute_at: w.tick + u64::from(d.cast),
            objective,
        };
        let plan = match self.plan(w, a, c) {
            Ok(plan) => plan,
            Err(reason) => {
                self.reject(w.tick, a, slot, request, reason);
                return;
            }
        };
        self.state.next_execution += 1;
        self.state.casts[a] = Some(c);
        self.record(w.tick, a, Some(c), request, Outcome::Started, [None; 8]);
        if c.execute_at == w.tick {
            // No intervening effects: an instant cast can commit its validated plan.
            self.complete(w, a, c, Ok(plan));
        }
    }
    pub fn inspect_origins(&self) -> serde_json::Value {
        self.state.origins.inspect()
    }
    /// Caller coordinates world/component removal after this infallible, pre-reserved capture.
    pub(crate) fn detach_origin(
        &mut self,
        actors: &Actors,
        vitality: &crate::vitality::Runtime,
        actor: u16,
        tick: u64,
    ) {
        let reference = actors.reference(actor);
        if let Some(slot) = self.state.origins.source_mut(reference) {
            slot.detached = true;
            slot.entity = actors.entity(actor) as u16;
            slot.faction = actors.faction(actor);
            slot.life = vitality.state.lives[actor as usize];
            slot.dead = vitality.dead(actor as usize);
            self.attributes.capture_source(
                &self.state.buffs,
                slot.entity,
                reference,
                tick,
                &mut slot.attributes,
            );
        }
        if let Some(cast) = self.state.casts[actor as usize].take() {
            self.record(
                tick,
                actor as usize,
                Some(cast),
                cast.request,
                Outcome::SourceUnavailable,
                [None; 8],
            );
        }
    }
    fn collect_origins(&mut self) {
        let casts = self
            .state
            .projectiles
            .active
            .iter()
            .map(|i| i.cast)
            .chain(self.state.periodic.active.iter().map(|i| i.cast))
            .chain(self.state.persistent_areas.active.iter().map(|i| i.cast));
        self.state
            .origins
            .collect(casts.filter_map(|c| c.origin.map(|lease| (lease, c.life.source_instance))));
    }
    pub fn resource_level(&self, actor: usize, resource: usize) -> Option<(i64, i64)> {
        (self.capacity[actor][resource] >= 0).then_some((
            self.state.balances[actor][resource],
            self.capacity[actor][resource],
        ))
    }
    fn healing_definition(d: &Compiled) -> bool {
        d.selection != Selection::Spatial
            && !d.resurrect
            && d.effects.iter().any(|effect| {
                matches!(effect,
            CompiledEffect::Vital {recipient,kind:crate::vitality::Change::Heal,..}
                if *recipient==Recipient::Target || d.selection==Selection::SelfActor)
            })
    }
    fn clean_support(&mut self, w: &World<'_>) {
        let casts = &self.state.casts;
        let shots = &self.state.projectiles.active;
        let periodic = &self.state.periodic.active;
        let areas = &self.state.persistent_areas.active;
        self.state.support_claims.retain(|lease| {
            let a = lease.actor as usize;
            w.tick < lease.expires_at
                && w.actors.matches(lease.actor, lease.source)
                && !w.vitality.dead(a)
                && w.vitality.state.lives[a] == lease.source_life
                && w.actors.matches(lease.target, lease.target_instance)
                && !w.vitality.dead(lease.target as usize)
                && w.vitality.state.lives[lease.target as usize] == lease.target_life
                && (casts[a].is_some_and(|c| c.id == lease.execution)
                    || shots
                        .source_positions(lease.actor)
                        .iter()
                        .any(|&i| shots[i].cast.id == lease.execution)
                    || periodic
                        .source_positions(lease.actor)
                        .iter()
                        .any(|&i| periodic[i].cast.id == lease.execution)
                    || areas
                        .source_positions(lease.actor)
                        .iter()
                        .any(|&i| areas[i].cast.id == lease.execution))
        });
    }
    fn schedule_support(&mut self, w: &World<'_>, frame: &mut FrameScratch) {
        frame.support_candidates.clear();
        frame.support_blocked = crate::actor_set::ActorSet::EMPTY;
        self.state.support_coordination.clear();
        if !self.support_enabled {
            return;
        }
        let mut leased = crate::actor_set::ActorSet::EMPTY;
        for lease in &self.state.support_claims {
            leased.insert(lease.actor as usize);
            frame
                .support_candidates
                .push(crate::npc_cooperation::SupportCandidate {
                    group: lease.group,
                    source: lease.actor,
                    source_instance: lease.source,
                    target: lease.target,
                    target_instance: lease.target_instance,
                    target_life: lease.target_life,
                    score: i32::MAX,
                    existing: true,
                });
        }
        for (a, intent) in frame.intentions.iter().copied().enumerate() {
            if leased.contains(a)
                || frame.occupied[a]
                || frame.entry_heads[a] != NO_ENTRY
                || self.state.casts[a].is_some()
            {
                continue;
            }
            let Some((_, slot, Some(target), _)) = intent else {
                continue;
            };
            let e = &w.scene.entities[w.actors.entity(a as u16)];
            let Some(c) = e
                .npc
                .as_ref()
                .and_then(|n| n.cooperation)
                .filter(|c| c.reserve_support)
            else {
                continue;
            };
            if !Self::healing_definition(&self.definitions[self.grants[a][slot as usize].unwrap()])
            {
                continue;
            }
            let profile = w.npcs[&e.id]
                .mind_profile
                .unwrap_or(crate::npc_mind::Profile {
                    intelligence: 5000,
                    traits: [5000; 9],
                });
            let score = i32::from(profile.traits[2]) * 2
                + i32::from(profile.traits[7])
                + i32::from(profile.intelligence)
                + i32::from(profile.traits[8]) / 2;
            frame
                .support_candidates
                .push(crate::npc_cooperation::SupportCandidate {
                    group: c.group,
                    source: a as u16,
                    source_instance: w.actors.reference(a as u16),
                    target,
                    target_instance: w.actors.reference(target),
                    target_life: frame.lives[target as usize],
                    score,
                    existing: false,
                });
        }
        frame.support_candidates.sort_unstable_by_key(|c| {
            (
                c.group,
                c.target,
                c.target_life,
                std::cmp::Reverse(c.existing),
                std::cmp::Reverse(c.score),
                c.source_instance.instance,
            )
        });
        let mut last = None;
        let mut winner = None;
        for c in &frame.support_candidates {
            let key = (c.group, c.target, c.target_life);
            if last != Some(key) {
                last = Some(key);
                winner = Some(c.source_instance);
            }
            let by = winner.unwrap();
            let allowed = by == c.source_instance;
            if !allowed && !c.existing {
                frame.support_blocked.insert(c.source as usize);
            }
            self.state
                .support_coordination
                .push(crate::npc_cooperation::SupportDecision {
                    group: c.group,
                    source: c.source_instance,
                    target: c.target_instance,
                    target_life: c.target_life,
                    reserved_by: by,
                    allowed,
                    score: c.score,
                });
        }
    }
    pub fn enable_request_results(&mut self, enabled: bool) {
        if enabled && self.request_tap.is_none() {
            let actors = self.actor_instances.len();
            self.request_tap = Some(RequestTap {
                watched: BTreeSet::new(),
                events: Vec::with_capacity(input_capacity(actors) + actors * 4),
                dropped: 0,
                limit: input_capacity(actors),
            });
        } else if !enabled {
            self.request_tap = None;
        }
    }
    pub fn request_results_enabled(&self) -> bool {
        self.request_tap.is_some()
    }
    pub fn watch_request(&mut self, request: u64) -> Result<()> {
        let tap = self
            .request_tap
            .as_mut()
            .ok_or_else(|| invalid("Request result observer is disabled"))?;
        if tap.watched.len() >= tap.limit && !tap.watched.contains(&request) {
            return Err(invalid("Request result observer capacity"));
        }
        tap.watched.insert(request);
        Ok(())
    }
    pub fn unwatch_request(&mut self, request: u64) {
        if let Some(tap) = &mut self.request_tap {
            tap.watched.remove(&request);
        }
    }
    pub fn request_results(&self) -> &[RequestResult] {
        self.request_tap
            .as_ref()
            .map_or(&[], |t| t.events.as_slice())
    }
    pub fn dropped_request_results(&self) -> u64 {
        self.request_tap.as_ref().map_or(0, |t| t.dropped)
    }
    /// Diagnostic totals since this Runtime was created; not part of saved gameplay State.
    pub fn outcome_counts(&self) -> &[(Outcome, u64)] {
        &self.outcome_counts
    }
    pub fn inspect(&self, scene: &Scene, actors: &Actors, tick: u64) -> serde_json::Value {
        use serde_json::json;
        let observation = |c: Cast| {
            c.aim_observation
                .map(|o| json!({"actor":c.life.target_instance.and_then(|r|actors.label(scene,r)),"instance_id":c.life.target_instance.map(|r|r.id()),"life_index":o.life,"tick":o.tick,"source":o.source}))
        };
        let mut states = serde_json::Map::new();
        let locks = self.movement_locks();
        for (a, &entity) in actors.actors.iter().enumerate() {
            if actors.entity_reference(entity as usize).is_none() {
                continue;
            }
            let e = &scene.entities[entity as usize];
            if e.abilities.is_none() {
                continue;
            }
            let pools:Vec<_>=scene.ability_catalog.resources.iter().enumerate().filter(|(r,_)|self.capacity[a][*r]>=0).map(|(r,d)|json!({"id":d.id,"scale":d.scale,"units":self.state.balances[a][r],"capacity":self.capacity[a][r]})).collect();
            let grants:Vec<_>=self.grants[a].iter().enumerate().filter_map(|(slot,d)|d.map(|i|json!({"slot":slot+1,"ability":scene.ability_catalog.abilities[i].id,"tags":scene.ability_catalog.abilities[i].tags,"ready_at":self.state.ready_at[a][slot],"retry_at":self.state.retry_at[a][slot],"selection":self.definitions[i].selection,"area":self.definitions[i].area,"interruptible_base":self.definitions[i].interruptible,"interruptible":self.effective_flag(scene,actors,a,i,crate::attributes::INTERRUPTIBLE,tick),"movement":self.definitions[i].movement,"delivery":self.definitions[i].delivery,"periodic":self.definitions[i].periodic,"channel":self.definitions[i].channel,"dispellable_base":self.definitions[i].dispellable,"dispellable":self.effective_flag(scene,actors,a,i,crate::attributes::DISPELLABLE,tick),"periodic_category":self.definitions[i].periodic_category.map(|c|&self.categories[c as usize])}))).collect();
            let cast=self.state.casts[a].map(|c|json!({"life_stamp":c.life,"execution_id":c.id,"slot":c.slot+1,"target":c.target.and(c.life.target_instance).and_then(|r|actors.label(scene,r)),"aim":c.aim,"aim_observation":observation(c),"started_tick":c.started,"execute_at":c.execute_at,"interruptible":self.effective_flag(scene,actors,a,c.definition as usize,crate::attributes::INTERRUPTIBLE,tick),"phase":if c.channel.is_some(){"channeling"}else{"preparing"},"channel":c.channel,"objective":c.objective,"request_id":c.request}));
            states.insert(
                e.id.clone(),
                json!({"interrupted_until":self.state.interrupted_until[a],"resources":pools,"grants":grants,"cast":cast,"movement_locked":locks.contains(a),"motion":{"speed":self.state.motion[a].speed_squared.sqrt(),"requested":self.state.motion[a].requested,"lock_supported":self.state.motion[a].lock_supported}}),
            );
        }
        let receipts:Vec<_>=self.state.receipts.iter().map(|r|{
            let definition=r.definition.map(usize::from);
            let costs:Vec<_>=definition.filter(|&i| match r.outcome {
                Outcome::Completed | Outcome::Launched => self.definitions[i].channel.is_none(),
                Outcome::ChannelStarted => self.definitions[i].channel.is_some_and(|c|c.cost_mode==crate::channel::CostMode::OnStart),
                Outcome::ChannelPulse => self.definitions[i].channel.is_some_and(|c|c.cost_mode==crate::channel::CostMode::PerPulse),
                _ => false,
            }).into_iter()
                .flat_map(|i|self.definitions[i].costs.iter().enumerate())
                .filter(|(_,units)|**units>0)
                .map(|(i,units)|json!({"resource":scene.ability_catalog.resources[i].id,"units":units})).collect();
            let effects:Vec<_>=r.effects.iter().enumerate().filter_map(|(index,e)|e.as_ref().map(|e|(index,e))).map(|(index,e)| {
                let components:Vec<_>=r.damage_components.iter().flatten().filter(|c|c.effect_index as usize==index).map(|c|json!({"index":c.component_index,"tags":self.attributes.tags.iter().enumerate().filter(|(i,_)|c.tags & (1u64 << i)!=0).map(|(_,t)|t).collect::<Vec<_>>(),"requested":c.requested,"multiplier":c.multiplier,"received_multiplier":c.multiplier,"dealt_multiplier":c.dealt_multiplier,"adjusted":c.adjusted})).collect();
                let total: i64=r.damage_components.iter().flatten().filter(|c|c.effect_index as usize==index).map(|c|c.adjusted).sum();
                json!({
                "effect_index":index,"damage_components":components,"damage_before_limit":e.adjusted_amount.map(|_|total),"damage_limit_excess":e.adjusted_amount.map(|adjusted|total-adjusted),
                "actor":actors.label(scene,e.actor_instance),"actor_instance":e.actor_instance.id(),
                "resource":e.resource.map(|i|&scene.ability_catalog.resources[i as usize].id),
                "buff":e.buff.map(|b|&scene.ability_catalog.attributes.buffs[b as usize].id),"damage_multiplier":e.damage_multiplier,"adjusted_amount":e.adjusted_amount,"interrupt":e.interrupt,"requested":e.requested,"applied":e.applied,"noise_radius":e.noise_radius,"vitality":e.vitality,
                "dispel":e.dispel.map(|d|json!({"category":self.categories[d.category as usize],"removed":d.removed}))
            })}).collect();
            json!({"interrupted_by":r.interrupted_by.map(|by|json!({"actor":actors.label(scene,by.actor_instance),"actor_instance":by.actor_instance.id(),"execution_id":by.execution_id})),"tick":r.tick,"actor":actors.label(scene,r.actor_instance),"actor_instance":r.actor_instance.id(),"execution_id":r.execution_id,
                "request_id":r.request,"outcome":r.outcome,"slot":r.slot.map(|slot|slot+1),
                "ability":definition.map(|i|&scene.ability_catalog.abilities[i].id),"costs":costs,"effects":effects})
        }).collect();
        let active:Vec<_>=self.state.projectiles.active.iter().map(|p|json!({"execution_id":p.cast.id,"actor":actors.label(scene,p.cast.life.source_instance),"actor_instance":p.cast.life.source_instance.id(),"intended_target":p.cast.target.and(p.cast.life.target_instance).and_then(|r|actors.label(scene,r)),"aim":p.cast.aim,"aim_observation":observation(p.cast),"slot":p.cast.slot+1,"launched_tick":p.launched_tick,"position":p.position,"direction":p.direction,"speed":p.speed,"half_extent":p.half_extent,"remaining_distance":p.remaining_distance})).collect();
        let impacts:Vec<_>=self.state.projectiles.receipts.iter().map(|p|json!({"execution_id":p.execution_id,"actor":actors.label(scene,p.actor_instance),"actor_instance":p.actor_instance.id(),"tick":p.tick,"position":p.position,"entity":p.entity_instance.and_then(|r|actors.label(scene,r)),"entity_instance":p.entity_instance.map(|r|r.id()),"outcome":p.outcome})).collect();
        let periodic: Vec<_> = self.state.periodic.active.iter().map(|e| json!({
            "life_stamp":e.cast.life,"execution_id":e.cast.id,"actor":actors.label(scene,e.cast.life.source_instance),"target":e.cast.life.target_instance.and_then(|r|actors.label(scene,r)),
            "ability":scene.ability_catalog.abilities[e.definition as usize].id,"category":self.definitions[e.definition as usize].periodic_category.map(|c|&self.categories[c as usize]),"dispellable_base":self.definitions[e.definition as usize].dispellable,"dispellable":(actors.matches(e.actor,e.cast.life.source_instance)||self.retained_origin(e.cast).is_some()).then(||self.source_value(actors,e.actor as usize,e.cast,crate::attributes::DISPELLABLE,crate::attributes::Value::boolean(self.definitions[e.definition as usize].dispellable),self.attributes.context(e.definition as usize),tick).as_bool()),"applied_tick":e.applied_tick,
            "next_tick":e.next_tick,"remaining_pulses":e.remaining,"interval_ticks":e.interval,
            "last_tick":e.next_tick+u64::from(e.remaining-1)*u64::from(e.interval),
            "reapply":self.definitions[e.definition as usize].periodic.unwrap().reapply
        })).collect();
        let periodic_receipts: Vec<_> = self.state.periodic.receipts.iter().map(|r| json!({
            "execution_id":r.execution_id,"actor":actors.label(scene,r.actor_instance),"actor_instance":r.actor_instance.id(),"target":actors.label(scene,r.target_instance),"target_instance":r.target_instance.id(),
            "tick":r.tick,"outcome":r.outcome,"replaced_by":r.replaced_by,
            "removed_by":r.removed_by.map(|by|json!({"actor":actors.label(scene,by.actor_instance),"actor_instance":by.actor_instance.id(),"execution_id":by.execution_id}))
        })).collect();
        json!({"origins":self.inspect_origins(),"persistent_areas":self.state.persistent_areas.inspect(scene,actors),"areas":self.state.areas.inspect(scene,actors),"buffs":self.state.buffs.inspect(scene,actors,tick),"periodic":{"active":periodic,"receipts":periodic_receipts,"capacity":crate::periodic::CAPACITY*self.actor_instances.len(),"per_actor_capacity":crate::periodic::CAPACITY,"dropped_receipts":self.state.periodic.dropped_receipts},"projectiles":{"active":active,"receipts":impacts,"capacity":crate::projectiles::CAPACITY*self.actor_instances.len(),"per_actor_capacity":crate::projectiles::CAPACITY,"dropped_receipts":self.state.projectiles.dropped_receipts},"actors":states,"receipts":receipts,"dropped_receipts":self.state.dropped_receipts,"outcomes_since_reset":self.outcome_counts.iter().map(|(outcome,count)|json!({"outcome":outcome,"count":count})).collect::<Vec<_>>(),"emitted_noises":self.state.emitted,"noise_limits":{"per_source_per_tick":NOISES_PER_SOURCE,"per_tick":noise_capacity(self.noise_counts.len())}})
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedRuntime {
    cleanup_needed: bool,
    pub state: State,
    bases: Vec<Vec<(u8, crate::attributes::Value)>>,
    outcomes: Vec<(Outcome, u64)>,
}
impl Runtime {
    pub(crate) fn checkpoint(&self) -> SavedRuntime {
        SavedRuntime {
            cleanup_needed: self.areas_cleanup_needed,
            state: self.state.clone(),
            bases: self.attributes.checkpoint_bases(),
            outcomes: self.outcome_counts.clone(),
        }
    }
    pub(crate) fn restore_checkpoint(
        &mut self,
        saved: SavedRuntime,
        scene: &Scene,
        actors: &Actors,
    ) -> crate::model::Result<()> {
        let n = actors.actors.len();
        let s = &saved.state;
        if [
            s.interrupted_until.len(),
            s.motion.len(),
            s.balances.len(),
            s.ready_at.len(),
            s.casts.len(),
            s.retry_at.len(),
        ]
        .iter()
        .any(|x| *x != n)
            || s.next_execution == 0
            || s.receipts.len() > RECEIPTS
            || s.recent_noises.len() > RECENT_NOISES
            || s.noises.len() > noise_capacity(n)
            || saved.outcomes.len() > 64
        {
            return Err(invalid("Checkpoint ability dimensions"));
        }
        let valid = |a: usize, c: &Cast| {
            a < n
                && (c.definition as usize) < scene.ability_catalog.abilities.len()
                && c.target.is_none_or(|t| (t as usize) < n)
                && c.id > 0
                && c.id < s.next_execution
                && (c.life.source_instance.slot as usize) < scene.entities.len()
                && c.origin.is_none_or(|i| {
                    s.origins
                        .slots()
                        .get(i as usize)
                        .is_some_and(|o| o.reference == Some(c.life.source_instance))
                })
        };
        if s.casts
            .iter()
            .enumerate()
            .any(|(a, c)| c.is_some_and(|c| !valid(a, &c)))
            || s.projectiles
                .active
                .iter()
                .any(|p| !valid(p.actor as usize, &p.cast))
            || s.periodic
                .active
                .iter()
                .any(|p| !valid(p.actor as usize, &p.cast))
            || s.persistent_areas
                .active
                .iter()
                .any(|p| !valid(p.actor as usize, &p.cast))
        {
            return Err(invalid("Checkpoint effect source/definition/target"));
        }
        if s.buffs.active.iter().any(|b| {
            b.owner as usize >= scene.entities.len()
                || b.buff as usize >= scene.ability_catalog.attributes.buffs.len()
        }) || s
            .periodic
            .active
            .iter()
            .any(|p| p.definition != p.cast.definition || p.remaining == 0 || p.interval == 0)
            || s.persistent_areas
                .active
                .iter()
                .any(|p| p.definition != p.cast.definition)
        {
            return Err(invalid("Checkpoint scheduled effect/buff domain"));
        }
        for (a, b) in s.balances.iter().enumerate() {
            for (r, v) in b.iter().enumerate() {
                if *v < 0
                    || self.capacity[a][r] < 0 && *v != 0
                    || self.capacity[a][r] >= 0 && *v > self.capacity[a][r]
                {
                    return Err(invalid("Checkpoint resource balance"));
                }
            }
        }
        self.attributes.restore_checkpoint_bases(saved.bases)?;
        self.state = saved.state;
        self.outcome_counts = saved.outcomes;
        self.areas_cleanup_needed = saved.cleanup_needed;
        Ok(())
    }
}
