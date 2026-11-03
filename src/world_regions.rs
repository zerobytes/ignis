//! Logical region selection only. Proposals never mutate residency or physics.
use crate::world_space::{CELL_METERS, Coordinates, Position, VERSION as SPACE_VERSION};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: u16 = 1;
pub const MAX_INTERESTS: usize = 1024;
pub const MAX_OBSERVED: usize = 4096;
pub const MAX_PROBES: u32 = 65_536;
type Result<T> = std::result::Result<T, String>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Key {
    #[serde(with = "crate::world_space::cell_codec")]
    #[schemars(with = "[String; 3]")]
    pub cell: [i64; 3],
    pub offset_meters: [u16; 3],
}
impl Key {
    fn at(position: Position, size: u32) -> Self {
        let raw: Coordinates = position.into();
        if size >= CELL_METERS as u32 {
            let stride = i64::from(size / CELL_METERS as u32);
            Self {
                cell: raw.cell.map(|v| v.div_euclid(stride) * stride),
                offset_meters: [0; 3],
            }
        } else {
            Self {
                cell: raw.cell,
                offset_meters: raw
                    .offset
                    .map(|v| ((v / f64::from(size)).floor() as u16) * size as u16),
            }
        }
    }
    fn valid(self, size: u32) -> bool {
        if size >= CELL_METERS as u32 {
            let stride = i64::from(size / CELL_METERS as u32);
            self.offset_meters == [0; 3]
                && self.cell.iter().all(|v| {
                    v.rem_euclid(stride) == 0
                        && i128::from(*v) + i128::from(stride) - 1 <= i128::from(i64::MAX)
                })
        } else {
            self.offset_meters
                .iter()
                .all(|v| *v < 1024 && u32::from(*v) % size == 0)
        }
    }
    fn origin(self) -> Result<Position> {
        Position::try_from(Coordinates {
            cell: self.cell,
            offset: self.offset_meters.map(f64::from),
        })
        .map_err(|e| e.to_string())
    }
    fn neighbor(self, delta: [i16; 3], size: u32) -> Option<Self> {
        let mut out = self;
        for (axis, step) in delta.into_iter().enumerate() {
            let local = i64::from(self.offset_meters[axis]) + i64::from(step) * i64::from(size);
            let carry = local.div_euclid(1024);
            out.cell[axis] = i64::try_from(i128::from(self.cell[axis]) + i128::from(carry)).ok()?;
            out.offset_meters[axis] = local.rem_euclid(1024) as u16;
        }
        Some(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Generator {
    pub id: String,
    pub version: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub occupied: u16,
    pub requests: u16,
    pub probes: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u16,
    pub space_version: u16,
    pub world_id: String,
    pub revision: u32,
    #[serde(with = "seed_codec")]
    #[schemars(with = "String")]
    pub seed: u64,
    pub generator: Generator,
    pub region_size_meters: u32,
    pub budget: Budget,
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:".contains(&b))
}
impl Manifest {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.space_version != SPACE_VERSION
            || self.revision == 0
            || !identifier(&self.world_id)
            || !identifier(&self.generator.id)
            || self.generator.version == 0
            || !(1..=1_048_576).contains(&self.region_size_meters)
            || !self.region_size_meters.is_power_of_two()
            || !(1..=MAX_OBSERVED as u16).contains(&self.budget.occupied)
            || !(1..=MAX_INTERESTS as u16).contains(&self.budget.requests)
            || !(1..=MAX_PROBES).contains(&self.budget.probes)
        {
            return Err("Invalid world manifest version/identity/grid/budget".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Interest {
    pub id: String,
    pub position: Position,
    pub priority: i16,
    pub load_radius: [u8; 3],
    pub retain_radius: [u8; 3],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Requested,
    Prepared,
    Resident,
    Retiring,
}
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Pin {
    Control,
    Actor,
    Delivery,
    Origin,
    Contact,
    DirtyState,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Observed {
    pub key: Key,
    pub phase: Phase,
    #[serde(deserialize_with = "read_pins")]
    pub pins: Vec<Pin>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub manifest: Manifest,
    #[serde(deserialize_with = "read_interests")]
    pub interests: Vec<Interest>,
    #[serde(deserialize_with = "read_observed")]
    pub observed: Vec<Observed>,
}
#[derive(Debug, Serialize)]
pub struct Load {
    pub key: Key,
    pub origin: Position,
    #[serde(with = "seed_codec")]
    pub seed: u64,
    pub interest: String,
    pub priority: i16,
    pub distance_regions: u16,
}
#[derive(Debug, Serialize)]
pub struct Release {
    pub key: Key,
    pub observed_phase: Phase,
}
#[derive(Debug, Default, Serialize)]
pub struct Totals {
    pub interests: usize,
    pub groups: usize,
    pub probes: u32,
    pub coordinate_domain_clipped: u32,
    pub wanted: usize,
    pub occupied: usize,
    pub free_slots: usize,
    pub overcommitted_slots: usize,
    pub already_present_or_pending: usize,
    pub waiting_for_retirement: usize,
    pub deferred_capacity: usize,
    pub deferred_request_budget: usize,
    pub held_by_pins: usize,
    pub held_by_interest: usize,
}
#[derive(Debug, Serialize)]
pub struct Plan {
    pub manifest: Manifest,
    pub region_meters: f64,
    pub proposals_only: bool,
    pub loads: Vec<Load>,
    pub unload_candidates: Vec<Release>,
    pub cancel_candidates: Vec<Release>,
    pub totals: Totals,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    center: Key,
    load: [u8; 3],
    retain: [u8; 3],
}
struct Group<'a> {
    shape: GroupKey,
    priority: i16,
    id: &'a str,
}
#[derive(Clone, Copy)]
struct Candidate {
    key: Key,
    group: usize,
    distance: u16,
}
fn comparison(a: &Candidate, b: &Candidate, groups: &[Group<'_>]) -> std::cmp::Ordering {
    let left = &groups[a.group];
    let right = &groups[b.group];
    right
        .priority
        .cmp(&left.priority)
        .then(a.distance.cmp(&b.distance))
        .then(a.key.cmp(&b.key))
        .then(left.id.cmp(right.id))
}
fn retained(key: Key, group: &Group<'_>, size: u32) -> bool {
    (0..3).all(|i| {
        let meters = (i128::from(key.cell[i]) - i128::from(group.shape.center.cell[i])) * 1024
            + i128::from(key.offset_meters[i])
            - i128::from(group.shape.center.offset_meters[i]);
        meters.abs() <= i128::from(group.shape.retain[i]) * i128::from(size)
    })
}

pub fn plan(query: Query) -> Result<Plan> {
    let Query {
        manifest,
        interests,
        observed,
    } = query;
    manifest.validate()?;
    if interests.len() > MAX_INTERESTS || observed.len() > MAX_OBSERVED {
        return Err("World region input exceeds its bounded domain".into());
    }
    let mut ids = BTreeSet::new();
    let mut grouped = BTreeMap::<GroupKey, Group<'_>>::new();
    for interest in &interests {
        if !identifier(&interest.id)
            || !ids.insert(interest.id.as_str())
            || (0..3).any(|i| {
                interest.load_radius[i] > 16
                    || interest.retain_radius[i] > 32
                    || interest.retain_radius[i] < interest.load_radius[i]
            })
        {
            return Err("Interests require unique IDs and radii load[0,16] <= retain[0,32]".into());
        }
        let shape = GroupKey {
            center: Key::at(interest.position, manifest.region_size_meters),
            load: interest.load_radius,
            retain: interest.retain_radius,
        };
        let proposal = Group {
            shape,
            priority: interest.priority,
            id: &interest.id,
        };
        grouped
            .entry(shape)
            .and_modify(|old| {
                if proposal.priority > old.priority
                    || (proposal.priority == old.priority && proposal.id < old.id)
                {
                    old.priority = proposal.priority;
                    old.id = proposal.id;
                }
            })
            .or_insert(proposal);
    }
    let groups: Vec<_> = grouped.into_values().collect();
    let probes = groups
        .iter()
        .map(|g| {
            g.shape
                .load
                .into_iter()
                .map(|r| u64::from(r) * 2 + 1)
                .product::<u64>()
        })
        .sum::<u64>();
    if probes > u64::from(manifest.budget.probes) {
        return Err(
            "Interest neighborhoods exceed the probe budget after grouping; no plan applied".into(),
        );
    }
    let mut known = BTreeMap::new();
    for region in &observed {
        if !region.key.valid(manifest.region_size_meters) {
            return Err("Observed region exceeds the coordinate domain".into());
        }
        let mut seen = 0_u8;
        if region.pins.len() > 6
            || region.pins.iter().any(|pin| {
                let bit = 1_u8 << (*pin as u8);
                let duplicate = seen & bit != 0;
                seen |= bit;
                duplicate
            })
            || known.insert(region.key, region).is_some()
        {
            return Err("Observed regions/pins must be unique and bounded".into());
        }
    }
    let mut totals = Totals {
        interests: interests.len(),
        groups: groups.len(),
        probes: probes as u32,
        occupied: known.len(),
        free_slots: usize::from(manifest.budget.occupied).saturating_sub(known.len()),
        overcommitted_slots: known
            .len()
            .saturating_sub(usize::from(manifest.budget.occupied)),
        ..Default::default()
    };
    let mut candidates = BTreeMap::<Key, Candidate>::new();
    for (group, g) in groups.iter().enumerate() {
        let radii = g.shape.load.map(i16::from);
        for x in -radii[0]..=radii[0] {
            for y in -radii[1]..=radii[1] {
                for z in -radii[2]..=radii[2] {
                    let Some(key) = g
                        .shape
                        .center
                        .neighbor([x, y, z], manifest.region_size_meters)
                    else {
                        totals.coordinate_domain_clipped += 1;
                        continue;
                    };
                    let candidate = Candidate {
                        key,
                        group,
                        distance: x.unsigned_abs() + y.unsigned_abs() + z.unsigned_abs(),
                    };
                    candidates
                        .entry(key)
                        .and_modify(|old| {
                            if comparison(&candidate, old, &groups).is_lt() {
                                *old = candidate;
                            }
                        })
                        .or_insert(candidate);
                }
            }
        }
    }
    totals.wanted = candidates.len();
    let mut ordered: Vec<_> = candidates.into_values().collect();
    ordered.sort_unstable_by(|a, b| comparison(a, b, &groups));
    let mut loads = Vec::with_capacity(usize::from(manifest.budget.requests).min(ordered.len()));
    let seed_prefix = manifest_seed(&manifest);
    for candidate in ordered {
        if let Some(existing) = known.get(&candidate.key) {
            if existing.phase == Phase::Retiring {
                totals.waiting_for_retirement += 1;
            } else {
                totals.already_present_or_pending += 1;
            }
        } else if loads.len() >= totals.free_slots {
            totals.deferred_capacity += 1;
        } else if loads.len() >= usize::from(manifest.budget.requests) {
            totals.deferred_request_budget += 1;
        } else {
            let g = &groups[candidate.group];
            loads.push(Load {
                key: candidate.key,
                origin: candidate.key.origin()?,
                seed: region_seed(seed_prefix, candidate.key),
                interest: g.id.to_owned(),
                priority: g.priority,
                distance_regions: candidate.distance,
            });
        }
    }
    let mut unload_candidates = Vec::new();
    let mut cancel_candidates = Vec::new();
    for (key, record) in known {
        if record.phase == Phase::Retiring {
            continue;
        }
        if !record.pins.is_empty() {
            totals.held_by_pins += 1;
            continue;
        }
        if groups
            .iter()
            .any(|g| retained(key, g, manifest.region_size_meters))
        {
            totals.held_by_interest += 1;
            continue;
        }
        let candidate = Release {
            key,
            observed_phase: record.phase,
        };
        if record.phase == Phase::Resident {
            unload_candidates.push(candidate);
        } else {
            cancel_candidates.push(candidate);
        }
    }
    Ok(Plan {
        region_meters: f64::from(manifest.region_size_meters),
        manifest,
        proposals_only: true,
        loads,
        unload_candidates,
        cancel_candidates,
        totals,
    })
}

/// Stable seed, NOT an identity/hash-based authorization token. Budget/revision changes
/// do not regenerate terrain; generator version/size/world seed do.
fn mix_seed(h: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *h = (*h ^ u64::from(*b)).wrapping_mul(0x100000001b3);
    }
}
fn manifest_seed(manifest: &Manifest) -> u64 {
    let mut h = 0xcbf29ce484222325;
    mix_seed(&mut h, b"IgnisRegionSeed1");
    for text in [&manifest.world_id, &manifest.generator.id] {
        mix_seed(&mut h, &(text.len() as u64).to_le_bytes());
        mix_seed(&mut h, text.as_bytes());
    }
    mix_seed(&mut h, &manifest.seed.to_le_bytes());
    mix_seed(&mut h, &manifest.generator.version.to_le_bytes());
    mix_seed(&mut h, &manifest.region_size_meters.to_le_bytes());
    h
}
fn region_seed(mut prefix: u64, key: Key) -> u64 {
    for coordinate in key.cell {
        mix_seed(&mut prefix, &coordinate.to_le_bytes());
    }
    for offset in key.offset_meters {
        mix_seed(&mut prefix, &offset.to_le_bytes());
    }
    prefix
}
fn read_vec<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const MAX: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct Bounded<T, const MAX: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const MAX: usize> serde::de::Visitor<'de> for Bounded<T, MAX> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "at most {MAX} entries")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            if seq.size_hint().is_some_and(|n| n > MAX) {
                return Err(serde::de::Error::custom("Region input limit exceeded"));
            }
            let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(MAX));
            while let Some(value) = seq.next_element()? {
                if out.len() == MAX {
                    return Err(serde::de::Error::custom("Region input limit exceeded"));
                }
                out.push(value);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Bounded::<T, MAX>(std::marker::PhantomData))
}
fn read_interests<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Interest>, D::Error> {
    read_vec::<D, Interest, MAX_INTERESTS>(d)
}
fn read_observed<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Observed>, D::Error> {
    read_vec::<D, Observed, MAX_OBSERVED>(d)
}
fn read_pins<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Pin>, D::Error> {
    read_vec::<D, Pin, 6>(d)
}
mod seed_codec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(value: &u64, s: S) -> std::result::Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            value.to_string().serialize(s)
        } else {
            value.serialize(s)
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<u64, D::Error> {
        if !d.is_human_readable() {
            return u64::deserialize(d);
        }
        let text = String::deserialize(d)?;
        let value: u64 = text.parse().map_err(serde::de::Error::custom)?;
        if value.to_string() != text {
            return Err(serde::de::Error::custom(
                "Seed must use canonical unsigned decimal",
            ));
        }
        Ok(value)
    }
}
