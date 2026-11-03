//! Extensible typed attributes and bounded temporary contributions. Names compile once.
use crate::model::{Result, Scene, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
pub const SCALE: i64 = 10_000;
pub const LIMIT: i64 = 1_000_000_000_000;
pub const CAPACITY: usize = 64; // Minimum quota per source/owner, not a world-sized mask.
pub const MAX_OWNER_CAPACITY: usize = crate::targeting::MAX_ACTORS;
pub const MAX_CAPACITY: usize = MAX_OWNER_CAPACITY * MAX_OWNER_CAPACITY;
pub fn contribution_quota(actors: usize) -> usize {
    CAPACITY.max(actors)
}
pub fn contribution_capacity(actors: usize) -> usize {
    actors * contribution_quota(actors)
}
pub const INTERRUPTIBLE: u8 = 0;
pub const DISPELLABLE: u8 = 1;
pub const DAMAGE_RECEIVED: u8 = 2;
pub const DAMAGE_DEALT: u8 = 3;
const DAMAGE_DEALT_KEY: &str = "combat.damage_dealt.multiplier";
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Value {
    Bool { value: bool },
    Number { units: i64 },
}
impl Value {
    pub fn boolean(value: bool) -> Self {
        Self::Bool { value }
    }
    pub fn number(units: i64) -> Self {
        Self::Number { units }
    }
    pub fn as_bool(self) -> bool {
        match self {
            Self::Bool { value } => value,
            _ => unreachable!("validated boolean attribute"),
        }
    }
    pub fn units(self) -> i64 {
        match self {
            Self::Number { units } => units,
            _ => unreachable!("validated numeric attribute"),
        }
    }
    fn same_type(self, other: Self) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub key: String,
    pub default: Value,
    #[serde(default)]
    pub minimum: Option<i64>,
    #[serde(default)]
    pub maximum: Option<i64>,
}
impl Definition {
    fn clamp(&self, value: Value) -> Value {
        match value {
            Value::Number { units } => Value::number(units.clamp(
                self.minimum.unwrap_or(-LIMIT),
                self.maximum.unwrap_or(LIMIT),
            )),
            _ => value,
        }
    }
    fn valid_value(&self, value: Value) -> bool {
        self.default.same_type(value)
            && match value {
                Value::Bool { .. } => true,
                Value::Number { units } => {
                    (-LIMIT..=LIMIT).contains(&units) && self.clamp(value) == value
                }
            }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Set { value: Value },
    Add { units: i64 },
    Multiply { basis_points: i64 },
    Min { units: i64 },
    Max { units: i64 },
}
impl Operation {
    fn valid(self, d: &Definition) -> bool {
        match self {
            Self::Set { value } => d.valid_value(value),
            Self::Add { units } | Self::Min { units } | Self::Max { units } => {
                matches!(d.default, Value::Number { .. }) && (-LIMIT..=LIMIT).contains(&units)
            }
            Self::Multiply { basis_points } => {
                matches!(d.default, Value::Number { .. })
                    && (-1_000_000..=1_000_000).contains(&basis_points)
            }
        }
    }
    fn apply(self, base: Value, d: &Definition) -> Value {
        let value = match self {
            Self::Set { value } => value,
            Self::Add { units } => Value::number(
                (i128::from(base.units()) + i128::from(units))
                    .clamp(-i128::from(LIMIT), i128::from(LIMIT)) as i64,
            ),
            Self::Multiply { basis_points } => Value::number(
                (i128::from(base.units()) * i128::from(basis_points) / i128::from(SCALE))
                    .clamp(-i128::from(LIMIT), i128::from(LIMIT)) as i64,
            ),
            Self::Min { units } => Value::number(base.units().min(units)),
            Self::Max { units } => Value::number(base.units().max(units)),
        };
        d.clamp(value)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Modifier {
    pub attribute: String,
    pub operation: Operation,
    #[serde(default)]
    pub priority: i16,
    #[serde(default)]
    pub tags_all: Vec<String>,
    #[serde(default)]
    pub abilities: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Buff {
    pub id: String,
    pub duration_ticks: u32,
    pub reapply: crate::periodic::Reapply,
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default = "default_true")]
    pub dispellable: bool,
}
fn default_true() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Base {
    pub entity: String,
    pub attribute: String,
    pub value: Value,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub definitions: Vec<Definition>,
    #[serde(default)]
    pub bases: Vec<Base>,
    #[serde(default)]
    pub buffs: Vec<Buff>,
    #[serde(default)]
    pub tags: Vec<String>,
}
impl Config {
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
            && self.bases.is_empty()
            && self.buffs.is_empty()
            && self.tags.is_empty()
    }
}
fn label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
fn builtins() -> Vec<Definition> {
    vec![
        Definition {
            key: "ability.interruptible".into(),
            default: Value::boolean(true),
            minimum: None,
            maximum: None,
        },
        Definition {
            key: "effect.dispellable".into(),
            default: Value::boolean(true),
            minimum: None,
            maximum: None,
        },
        Definition {
            key: "combat.damage_received.multiplier".into(),
            default: Value::number(SCALE),
            minimum: Some(0),
            maximum: Some(1_000_000),
        },
        Definition {
            key: DAMAGE_DEALT_KEY.into(),
            default: Value::number(SCALE),
            minimum: Some(0),
            maximum: Some(1_000_000),
        },
    ]
}
fn references(scene: &Scene) -> impl Iterator<Item = &Vec<String>> {
    let c = &scene.ability_catalog;
    c.abilities
        .iter()
        .map(|a| &a.tags)
        .chain(
            c.attributes
                .buffs
                .iter()
                .flat_map(|b| b.modifiers.iter().map(|m| &m.tags_all)),
        )
        .chain(
            c.abilities
                .iter()
                .flat_map(|a| a.effects.iter())
                .filter_map(|e| match e {
                    crate::abilities::Effect::DamageComponents { components, .. } => {
                        Some(components)
                    }
                    _ => None,
                })
                .flatten()
                .map(|c| &c.tags),
        )
}
fn tags(scene: &Scene) -> BTreeSet<&str> {
    let declared = scene
        .ability_catalog
        .attributes
        .tags
        .iter()
        .map(String::as_str);
    if scene.version >= 39 {
        declared.collect()
    } else {
        declared
            .chain(references(scene).flatten().map(String::as_str))
            .collect()
    }
}
/// Preserve the inferred registry and ability-wide damage classification of pre39 scenes.
/// Only called after validation; new documents never infer unknown references.
pub fn migrate(scene: &mut Scene) {
    if scene.version >= 39 {
        return;
    }
    scene.ability_catalog.attributes.tags = tags(scene).into_iter().map(str::to_owned).collect();
    for a in &mut scene.ability_catalog.abilities {
        a.tags.sort();
        a.tags.dedup();
        if !a.tags.is_empty() {
            for effect in &mut a.effects {
                if let crate::abilities::Effect::Damage { recipient, amount } = effect {
                    *effect = crate::abilities::Effect::DamageComponents {
                        recipient: *recipient,
                        components: vec![crate::damage::Component {
                            amount: *amount,
                            tags: a.tags.clone(),
                        }],
                    };
                }
            }
        }
    }
    for b in &mut scene.ability_catalog.attributes.buffs {
        for m in &mut b.modifiers {
            m.tags_all.sort();
            m.tags_all.dedup();
        }
    }
}
pub fn validate(scene: &Scene) -> Result<()> {
    let c = &scene.ability_catalog.attributes;
    if scene.version < 38
        && (!c.is_empty()
            || scene
                .ability_catalog
                .abilities
                .iter()
                .any(|a| !a.tags.is_empty()))
    {
        return Err(invalid(
            "Attributes, buffs and ability tags require scene version38",
        ));
    }
    if c.definitions.iter().any(|d| d.key == DAMAGE_DEALT_KEY) {
        return Err(invalid(
            "combat.damage_dealt.multiplier is reserved; rename the custom key and its references before upgrading",
        ));
    }
    if scene.version < 40
        && (c.bases.iter().any(|b| b.attribute == DAMAGE_DEALT_KEY)
            || c.buffs
                .iter()
                .any(|b| b.modifiers.iter().any(|m| m.attribute == DAMAGE_DEALT_KEY)))
    {
        return Err(invalid("Damage dealt modifiers require scene version40"));
    }
    let base_limit = if scene.version < 88 {
        512
    } else {
        65 * (scene.entities.len() + scene.npc_templates.len())
    };
    if c.definitions.len() > 61
        || c.buffs.len() > 64
        || c.bases.len() > base_limit
        || c.tags.len() > 64
    {
        return Err(invalid(
            "Attribute limits:65 keys including4 builtins,64 buffs,64 tags; bases512 before scene88, then65 per entity/template",
        ));
    }
    let mut defs = builtins();
    let mut keys: BTreeSet<String> = defs.iter().map(|d| d.key.clone()).collect();
    for d in &c.definitions {
        if !label(&d.key)
            || !keys.insert(d.key.clone())
            || d.minimum.unwrap_or(-LIMIT) < -LIMIT
            || d.maximum.unwrap_or(LIMIT) > LIMIT
            || d.minimum.unwrap_or(-LIMIT) > d.maximum.unwrap_or(LIMIT)
            || !d.valid_value(d.default)
            || matches!(d.default, Value::Bool { .. })
                && (d.minimum.is_some() || d.maximum.is_some())
        {
            return Err(invalid("Invalid attribute key, type, bounds or default"));
        }
        defs.push(d.clone());
    }
    let find = |key: &str| defs.iter().find(|d| d.key == key);
    let owners: BTreeSet<_> = scene
        .entities
        .iter()
        .chain(&scene.npc_templates)
        .map(|e| e.id.as_str())
        .collect();
    let mut bindings = BTreeSet::new();
    for b in &c.bases {
        if !owners.contains(b.entity.as_str())
            || !bindings.insert((&b.entity, &b.attribute))
            || find(&b.attribute).is_none_or(|d| !d.valid_value(b.value))
        {
            return Err(invalid("Invalid or duplicate attribute base binding"));
        }
    }
    let mut ids = BTreeSet::new();
    for b in &c.buffs {
        if !label(&b.id)
            || !ids.insert(&b.id)
            || !(1..=216_000).contains(&b.duration_ticks)
            || b.modifiers.is_empty()
            || b.modifiers.len() > 8
            || b.category.as_deref().is_some_and(|c| !label(c))
        {
            return Err(invalid(
                "Invalid buff ID, duration, category or modifier count",
            ));
        }
        for m in &b.modifiers {
            if m.tags_all.len() > 64
                || m.abilities.len() > 64
                || find(&m.attribute).is_none_or(|d| !m.operation.valid(d))
                || m.abilities
                    .iter()
                    .any(|id| !scene.ability_catalog.abilities.iter().any(|a| a.id == *id))
            {
                return Err(invalid(
                    "Invalid modifier attribute, operation, context or ability filter",
                ));
            }
        }
    }
    let ts = tags(scene);
    if scene.version >= 39 {
        if ts.len() != c.tags.len() {
            return Err(invalid("Duplicate registered tag"));
        }
        for list in references(scene) {
            let mut used = BTreeSet::new();
            for tag in list {
                if !ts.contains(tag.as_str()) || !used.insert(tag) {
                    return Err(invalid(format!(
                        "Unregistered or duplicate tag reference: {tag}"
                    )));
                }
            }
        }
    }
    if ts.len() > 64
        || ts.iter().any(|t| !label(t))
        || scene
            .ability_catalog
            .abilities
            .iter()
            .any(|a| a.tags.len() > 64)
    {
        return Err(invalid(
            "Tags require valid labels and at most64 distinct entries",
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Instance {
    pub id: u64,
    pub buff: u8,
    pub owner: u16,
    pub owner_instance: crate::identity::Reference,
    pub source: u16,
    pub source_instance: crate::identity::Reference,
    pub execution_id: u64,
    pub applied_tick: u64,
    pub expires_at: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum End {
    Applied,
    Refreshed,
    Expired,
    Dispelled,
    OwnerDead,
    OwnerRemoved,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Responsible {
    pub entity: u16,
    pub instance: crate::identity::Reference,
    pub execution_id: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub instance: Instance,
    pub tick: u64,
    pub outcome: End,
    pub responsible: Option<Responsible>,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct State {
    pub active: Contributions,
    pub receipts: VecDeque<Receipt>,
    pub dropped_receipts: u64,
    pub next_id: u64,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(remote = "State")]
        struct Wire {
            pub active: Contributions,
            pub receipts: VecDeque<Receipt>,
            pub dropped_receipts: u64,
            pub next_id: u64,
        }
        let state = Wire::deserialize(d)?;
        if state.next_id == 0
            || state.active.iter().any(|i| i.id >= state.next_id)
            || state
                .receipts
                .iter()
                .any(|r| r.instance.id == 0 || r.instance.id >= state.next_id)
        {
            return Err(serde::de::Error::custom("Invalid buff identifier counter"));
        }
        Ok(state)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            active: Contributions::default(),
            receipts: VecDeque::with_capacity(256),
            dropped_receipts: 0,
            next_id: 1,
        }
    }
}
/// Dense application order plus a derived owner/definition index. No mutable slice
/// escapes: every structural mutation must repair the index before the next read.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Contributions {
    entries: Vec<Instance>,
    #[serde(skip)]
    by_owner_buff: Vec<usize>,
    #[serde(skip)]
    by_owner: Vec<usize>,
    #[serde(skip)]
    by_source: Vec<usize>,
    #[serde(skip)]
    by_id: Vec<usize>,
}
impl PartialEq for Contributions {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl Default for Contributions {
    fn default() -> Self {
        Self {
            entries: Vec::with_capacity(CAPACITY),
            by_owner_buff: Vec::with_capacity(CAPACITY),
            by_owner: Vec::with_capacity(CAPACITY),
            by_source: Vec::with_capacity(CAPACITY),
            by_id: Vec::with_capacity(CAPACITY),
        }
    }
}
impl<'de> Deserialize<'de> for Contributions {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let entries = Vec::<Instance>::deserialize(deserializer)?;
        let mut ids = BTreeSet::new();
        let mut owners = BTreeMap::new();
        let mut sources = BTreeMap::new();
        if entries.len() > MAX_CAPACITY
            || entries.iter().any(|i| {
                let owner_count = owners.entry(i.owner).or_insert(0usize);
                *owner_count += 1;
                let source_count = sources.entry(i.source).or_insert(0usize);
                *source_count += 1;
                i.owner as usize >= crate::model::MAX_ENTITIES
                    || i.source as usize >= crate::model::MAX_ENTITIES
                    || i.buff >= 64
                    || i.id == 0
                    || !ids.insert(i.id)
                    || *owner_count > MAX_OWNER_CAPACITY
                    || *source_count > MAX_OWNER_CAPACITY
            })
        {
            return Err(serde::de::Error::custom(
                "Invalid buff domain, ID or contribution capacity",
            ));
        }
        let mut result = Self::default();
        result.entries.extend(entries);
        result.reindex();
        Ok(result)
    }
}
impl std::ops::Deref for Contributions {
    type Target = [Instance];
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}
impl Contributions {
    fn reindex(&mut self) {
        for index in [
            &mut self.by_owner_buff,
            &mut self.by_owner,
            &mut self.by_source,
            &mut self.by_id,
        ] {
            index.clear();
            index.extend(0..self.entries.len());
        }
        self.by_owner_buff.sort_unstable_by_key(|&n| {
            let i = &self.entries[n];
            (i.owner, i.buff, n)
        });
        self.by_owner
            .sort_unstable_by_key(|&n| (self.entries[n].owner, n));
        self.by_source
            .sort_unstable_by_key(|&n| (self.entries[n].source, n));
        self.by_id.sort_unstable_by_key(|&n| self.entries[n].id);
    }
    fn range<'a>(
        &'a self,
        index: &'a [usize],
        value: u16,
        key: impl Fn(&Instance) -> u16,
    ) -> &'a [usize] {
        let start = index.partition_point(|&n| key(&self.entries[n]) < value);
        let count = index[start..].partition_point(|&n| key(&self.entries[n]) == value);
        &index[start..start + count]
    }
    pub(crate) fn owner_indices(&self, owner: u16) -> &[usize] {
        self.range(&self.by_owner, owner, |i| i.owner)
    }
    fn source_indices(&self, source: u16) -> &[usize] {
        self.range(&self.by_source, source, |i| i.source)
    }
    pub fn owner_count(&self, owner: u16) -> usize {
        self.owner_indices(owner).len()
    }
    pub fn source_count(&self, source: u16) -> usize {
        self.source_indices(source).len()
    }
    pub(crate) fn get_id(&self, id: u64) -> Option<&Instance> {
        self.by_id
            .binary_search_by_key(&id, |&n| self.entries[n].id)
            .ok()
            .map(|n| &self.entries[self.by_id[n]])
    }
    fn for_buff(&self, owner: u16, buff: u8) -> impl Iterator<Item = &Instance> {
        let key = |n: &usize| {
            let i = &self.entries[*n];
            (i.owner, i.buff)
        };
        let start = self
            .by_owner_buff
            .partition_point(|n| key(n) < (owner, buff));
        let count = self.by_owner_buff[start..].partition_point(|n| key(n) == (owner, buff));
        self.by_owner_buff[start..start + count]
            .iter()
            .map(|&n| &self.entries[n])
    }
    fn for_owner(&self, owner: u16) -> impl Iterator<Item = &Instance> {
        self.owner_indices(owner).iter().map(|&n| &self.entries[n])
    }
    fn push(&mut self, instance: Instance) {
        assert!(
            (instance.owner as usize) < crate::model::MAX_ENTITIES
                && (instance.source as usize) < crate::model::MAX_ENTITIES
                && instance.buff < 64
        );
        assert!(
            self.entries.len() < MAX_CAPACITY
                && self.owner_count(instance.owner) < MAX_OWNER_CAPACITY
                && self.source_count(instance.source) < MAX_OWNER_CAPACITY,
            "validated buff admission"
        );
        assert!(instance.id > 0 && self.get_id(instance.id).is_none());
        let n = self.entries.len();
        self.entries.push(instance);
        let at = self.by_owner_buff.partition_point(|&n| {
            let i = &self.entries[n];
            (i.owner, i.buff) <= (instance.owner, instance.buff)
        });
        self.by_owner_buff.insert(at, n);
        let at = self
            .by_owner
            .partition_point(|&n| self.entries[n].owner <= instance.owner);
        self.by_owner.insert(at, n);
        let at = self
            .by_source
            .partition_point(|&n| self.entries[n].source <= instance.source);
        self.by_source.insert(at, n);
        let at = self
            .by_id
            .partition_point(|&n| self.entries[n].id < instance.id);
        self.by_id.insert(at, n);
    }
    fn remove(&mut self, index: usize) -> Instance {
        let removed = self.entries.remove(index);
        for indices in [
            &mut self.by_owner_buff,
            &mut self.by_owner,
            &mut self.by_source,
            &mut self.by_id,
        ] {
            indices.retain_mut(|n| {
                if *n == index {
                    false
                } else {
                    if *n > index {
                        *n -= 1;
                    }
                    true
                }
            });
        }
        removed
    }
    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        for index in [
            &mut self.by_owner_buff,
            &mut self.by_owner,
            &mut self.by_source,
            &mut self.by_id,
        ] {
            index.clear();
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Install {
    pub buff: u8,
    pub source: u16,
    pub source_instance: crate::identity::Reference,
    pub owner: u16,
    pub owner_instance: crate::identity::Reference,
    pub execution_id: u64,
}
#[derive(Clone, Copy)]
struct Rule {
    buff: u8,
    index: u8,
    priority: i16,
    operation: Operation,
    tags: u64,
    abilities: u64,
}
struct CompiledBuff {
    duration: u32,
    reapply: crate::periodic::Reapply,
    category: Option<u8>,
    dispellable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceSnapshot {
    pub bases: Vec<(u8, Value)>,
    pub contributions: Vec<Instance>,
}
impl Default for SourceSnapshot {
    fn default() -> Self {
        Self {
            bases: Vec::with_capacity(65),
            contributions: Vec::with_capacity(CAPACITY),
        }
    }
}
pub struct Registry {
    actors: usize,
    definitions: Vec<Definition>,
    rules: Vec<Vec<Rule>>,
    bases: Vec<Vec<(u8, Value)>>,
    template_bases: Vec<Vec<(u8, Value)>>,
    buffs: Vec<CompiledBuff>,
    pub tags: Vec<String>,
    pub ability_tags: Vec<u64>,
}
#[derive(Clone, Copy, Default)]
pub struct Context {
    pub tags: u64,
    pub ability: Option<u8>,
}
impl Registry {
    pub(crate) fn install_template(&mut self, entity: usize, template: usize) {
        self.bases[entity].clone_from(&self.template_bases[template]);
    }
    pub fn new(scene: &Scene, categories: &[String]) -> Self {
        let c = &scene.ability_catalog.attributes;
        let mut definitions = builtins();
        definitions.extend(c.definitions.iter().cloned());
        let tags: Vec<String> = tags(scene).into_iter().map(Into::into).collect();
        let mask = |labels: &[String]| {
            labels.iter().fold(0, |m, t| {
                m | (1u64 << tags.iter().position(|x| x == t).unwrap())
            })
        };
        let mut rules = vec![Vec::new(); definitions.len()];
        for (b, buff) in c.buffs.iter().enumerate() {
            for (i, m) in buff.modifiers.iter().enumerate() {
                let attr = definitions
                    .iter()
                    .position(|d| d.key == m.attribute)
                    .unwrap();
                rules[attr].push(Rule {
                    buff: b as u8,
                    index: i as u8,
                    priority: m.priority,
                    operation: m.operation,
                    tags: mask(&m.tags_all),
                    abilities: m.abilities.iter().fold(0, |m, id| {
                        m | (1u64
                            << scene
                                .ability_catalog
                                .abilities
                                .iter()
                                .position(|a| a.id == *id)
                                .unwrap())
                    }),
                });
            }
        }
        for group in &mut rules {
            group.sort_by_key(|r| (r.priority, r.buff, r.index));
        }
        let entity_ids: BTreeMap<_, _> = scene
            .entities
            .iter()
            .enumerate()
            .map(|(n, e)| (e.id.as_str(), n))
            .collect();
        let template_ids: BTreeMap<_, _> = scene
            .npc_templates
            .iter()
            .enumerate()
            .map(|(n, e)| (e.id.as_str(), n))
            .collect();
        let mut bases = vec![Vec::new(); scene.entities.len()];
        let mut template_bases = vec![Vec::new(); scene.npc_templates.len()];
        for b in &c.bases {
            let e = entity_ids.get(b.entity.as_str()).copied();
            let attr = definitions
                .iter()
                .position(|d| d.key == b.attribute)
                .unwrap();
            if let Some(e) = e {
                bases[e].push((attr as u8, b.value));
            } else {
                let t = *template_ids
                    .get(b.entity.as_str())
                    .expect("validated template base");
                template_bases[t].push((attr as u8, b.value));
            }
        }
        Self {
            actors: scene
                .entities
                .iter()
                .filter(|e| {
                    matches!(
                        e.body,
                        Some(crate::model::BodyKind::Player | crate::model::BodyKind::Npc)
                    )
                })
                .count(),
            ability_tags: scene
                .ability_catalog
                .abilities
                .iter()
                .map(|a| mask(&a.tags))
                .collect(),
            definitions,
            rules,
            bases,
            template_bases,
            buffs: c
                .buffs
                .iter()
                .map(|b| CompiledBuff {
                    duration: b.duration_ticks,
                    reapply: b.reapply,
                    category: b
                        .category
                        .as_ref()
                        .map(|c| categories.iter().position(|x| x == c).unwrap() as u8),
                    dispellable: b.dispellable,
                })
                .collect(),
            tags,
        }
    }
    pub fn tag_mask(&self, tags: &[String]) -> u64 {
        tags.iter().fold(0, |mask, tag| {
            mask | (1u64
                << self
                    .tags
                    .iter()
                    .position(|t| t == tag)
                    .expect("validated tag"))
        })
    }
    pub fn context(&self, ability: usize) -> Context {
        Context {
            tags: self.ability_tags[ability],
            ability: Some(ability as u8),
        }
    }
    pub fn key(&self, key: &str) -> Option<u8> {
        self.definitions
            .iter()
            .position(|d| d.key == key)
            .map(|i| i as u8)
    }
    pub fn default_value(&self, key: u8) -> Value {
        self.definitions[key as usize].default
    }
    pub fn base(&self, owner: u16, key: u8, fallback: Value) -> Value {
        self.bases[owner as usize]
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(fallback, |(_, v)| *v)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate(
        &self,
        actors: &crate::targeting::Catalog,
        state: &State,
        owner: u16,
        key: u8,
        fallback: Value,
        context: Context,
        tick: u64,
    ) -> Value {
        self.resolve(
            actors,
            state,
            owner,
            key,
            fallback,
            context,
            tick,
            |_, _, _, _| {},
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn resolve(
        &self,
        actors: &crate::targeting::Catalog,
        state: &State,
        owner: u16,
        key: u8,
        fallback: Value,
        context: Context,
        tick: u64,
        mut visit: impl FnMut(&Instance, &Rule, Value, Value),
    ) -> Value {
        if state.active.is_empty() {
            return self.base(owner, key, fallback);
        }
        self.resolve_contributions(
            &self.bases[owner as usize],
            |buff| state.active.for_buff(owner, buff),
            key,
            fallback,
            context,
            tick,
            |i| i.owner == owner && actors.entity_matches(owner, i.owner_instance),
            &mut visit,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn resolve_contributions<'a, I: Iterator<Item = &'a Instance>>(
        &self,
        bases: &[(u8, Value)],
        instances: impl Fn(u8) -> I,
        key: u8,
        fallback: Value,
        context: Context,
        tick: u64,
        eligible: impl Fn(&Instance) -> bool,
        mut visit: impl FnMut(&Instance, &Rule, Value, Value),
    ) -> Value {
        let mut value = bases
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(fallback, |(_, v)| *v);
        for rule in &self.rules[key as usize] {
            if context.tags & rule.tags != rule.tags
                || rule.abilities != 0
                    && context
                        .ability
                        .is_none_or(|a| rule.abilities & (1u64 << a) == 0)
            {
                continue;
            }
            for instance in instances(rule.buff) {
                if instance.buff == rule.buff && tick < instance.expires_at && eligible(instance) {
                    let before = value;
                    value = rule.operation.apply(value, &self.definitions[key as usize]);
                    visit(instance, rule, before, value);
                }
            }
        }
        value
    }
    pub(crate) fn capture_source(
        &self,
        state: &State,
        entity: u16,
        reference: crate::identity::Reference,
        tick: u64,
        output: &mut SourceSnapshot,
    ) {
        output.bases.clear();
        output.bases.extend_from_slice(&self.bases[entity as usize]);
        output.contributions.clear();
        output.contributions.extend(
            state
                .active
                .for_owner(entity)
                .filter(|i| i.owner_instance == reference && tick < i.expires_at)
                .copied(),
        );
    }
    pub fn evaluate_source(
        &self,
        source: &SourceSnapshot,
        key: u8,
        fallback: Value,
        context: Context,
        tick: u64,
    ) -> Value {
        self.resolve_contributions(
            &source.bases,
            |buff| source.contributions.iter().filter(move |i| i.buff == buff),
            key,
            fallback,
            context,
            tick,
            |_| true,
            |_, _, _, _| {},
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn inspect_value(
        &self,
        actors: &crate::targeting::Catalog,
        state: &State,
        owner: u16,
        key: u8,
        fallback: Value,
        context: Context,
        tick: u64,
        scene: &Scene,
    ) -> serde_json::Value {
        let mut contributions = Vec::new();
        let effective=self.resolve(actors,state,owner,key,fallback,context,tick,|i,r,before,after|contributions.push(serde_json::json!({"instance_id":i.id,"buff":scene.ability_catalog.attributes.buffs[i.buff as usize].id,"source":actors.label(scene,i.source_instance),"source_instance":i.source_instance.id(),"execution_id":i.execution_id,"expires_at":i.expires_at,"modifier":r.index,"priority":r.priority,"operation":r.operation,"before":before,"after":after})));
        serde_json::json!({"entity":scene.entities[owner as usize].id,"attribute":self.definitions[key as usize].key,"base":self.base(owner,key,fallback),"effective":effective,"contributions":contributions,"numeric_scale":SCALE})
    }
    pub fn removable(
        &self,
        actors: &crate::targeting::Catalog,
        i: &Instance,
        owner: u16,
        category: u8,
    ) -> bool {
        let b = &self.buffs[i.buff as usize];
        i.owner == owner
            && actors.entity_matches(owner, i.owner_instance)
            && b.category == Some(category)
            && b.dispellable
    }
    fn matches(&self, i: &Instance, p: Install) -> bool {
        self.buffs[p.buff as usize].reapply == crate::periodic::Reapply::Refresh
            && i.buff == p.buff
            && i.owner == p.owner
            && i.owner_instance == p.owner_instance
            && i.source_instance == p.source_instance
            && i.source == p.source
    }
    fn present(&self, state: &State, p: Install, removed: &crate::transaction::IdSet<512>) -> bool {
        self.buffs[p.buff as usize].reapply == crate::periodic::Reapply::Refresh
            && state.active.source_indices(p.source).iter().any(|&n| {
                let i = &state.active[n];
                !removed.contains(i.id) && self.matches(i, p)
            })
    }
    fn same(&self, a: Install, b: Install) -> bool {
        self.buffs[b.buff as usize].reapply == crate::periodic::Reapply::Refresh
            && a.buff == b.buff
            && a.source == b.source
            && a.owner == b.owner
            && a.owner_instance == b.owner_instance
            && a.source_instance == b.source_instance
    }
    fn projected(
        &self,
        state: &State,
        pending: &[Option<Install>; 8],
        removed: &crate::transaction::IdSet<512>,
        owner: u16,
        source: u16,
    ) -> (usize, usize, usize) {
        let (mut total, mut own, mut authored) = (
            state.active.len(),
            state.active.owner_count(owner),
            state.active.source_count(source),
        );
        for i in removed.iter().filter_map(|id| state.active.get_id(id)) {
            total -= 1;
            own -= usize::from(i.owner == owner);
            authored -= usize::from(i.source == source);
        }
        for (n, q) in pending.iter().enumerate() {
            if let Some(q) = q
                && !self.present(state, *q, removed)
                && !pending[..n].iter().flatten().any(|&r| self.same(r, *q))
            {
                total += 1;
                own += usize::from(q.owner == owner);
                authored += usize::from(q.source == source);
            }
        }
        (total, own, authored)
    }
    pub(crate) fn projected_owner_count(
        &self,
        state: &State,
        pending: &[Option<Install>; 8],
        removed: &crate::transaction::IdSet<512>,
        owner: u16,
    ) -> usize {
        self.projected(state, pending, removed, owner, 0).1
    }
    pub(crate) fn available(
        &self,
        state: &State,
        p: Install,
        pending: &[Option<Install>; 8],
        removed: &crate::transaction::IdSet<512>,
    ) -> bool {
        if state
            .next_id
            .checked_add(pending.iter().flatten().count() as u64 + 1)
            .is_none()
        {
            return false;
        }
        if self.present(state, p, removed) || pending.iter().flatten().any(|&q| self.same(q, p)) {
            return true;
        }
        let (total, owner, source) = self.projected(state, pending, removed, p.owner, p.source);
        total < contribution_capacity(self.actors)
            && owner < contribution_quota(self.actors)
            && source < contribution_quota(self.actors)
    }
    pub fn install(&self, state: &mut State, p: Install, tick: u64) {
        assert!(state.next_id < u64::MAX, "validated buff identifier budget");
        if let Some(n) = state
            .active
            .source_indices(p.source)
            .iter()
            .copied()
            .find(|&n| self.matches(&state.active[n], p))
        {
            let old = state.active.remove(n);
            state.record(
                old,
                tick,
                End::Refreshed,
                Some(Responsible {
                    entity: p.source,
                    instance: p.source_instance,
                    execution_id: p.execution_id,
                }),
            );
        }
        let i = Instance {
            id: state.next_id,
            buff: p.buff,
            source: p.source,
            source_instance: p.source_instance,
            owner: p.owner,
            owner_instance: p.owner_instance,
            execution_id: p.execution_id,
            applied_tick: tick,
            expires_at: tick + u64::from(self.buffs[p.buff as usize].duration),
        };
        state.next_id += 1;
        debug_assert!(state.active.len() < contribution_capacity(self.actors));
        state.active.push(i);
        state.record(i, tick, End::Applied, None);
    }
}
impl State {
    fn record(&mut self, i: Instance, tick: u64, outcome: End, responsible: Option<Responsible>) {
        if self.receipts.len() == 256 {
            self.receipts.pop_front();
            self.dropped_receipts += 1;
        }
        self.receipts.push_back(Receipt {
            instance: i,
            tick,
            outcome,
            responsible,
        });
    }
    pub fn remove(
        &mut self,
        tick: u64,
        mut reason: impl FnMut(usize, &Instance) -> Option<(End, Option<Responsible>)>,
    ) {
        let mut retained = 0;
        for n in 0..self.active.len() {
            let i = self.active[n];
            if let Some((outcome, by)) = reason(n, &i) {
                self.record(i, tick, outcome, by);
            } else {
                if retained != n {
                    self.active.entries[retained] = i;
                }
                retained += 1;
            }
        }
        if retained != self.active.len() {
            self.active.entries.truncate(retained);
            self.active.reindex();
        }
    }
    pub fn inspect(
        &self,
        scene: &Scene,
        actors: &crate::targeting::Catalog,
        tick: u64,
    ) -> serde_json::Value {
        let instance = |i: &Instance| serde_json::json!({"id":i.id,"buff":scene.ability_catalog.attributes.buffs[i.buff as usize].id,"owner":actors.label(scene,i.owner_instance),"owner_instance":i.owner_instance.id(),"source":actors.label(scene,i.source_instance),"source_instance":i.source_instance.id(),"execution_id":i.execution_id,"applied_tick":i.applied_tick,"expires_at":i.expires_at,"category":scene.ability_catalog.attributes.buffs[i.buff as usize].category,"dispellable":scene.ability_catalog.attributes.buffs[i.buff as usize].dispellable});
        serde_json::json!({"active":self.active.iter().filter(|i|tick<i.expires_at).map(instance).collect::<Vec<_>>(),"receipts":self.receipts.iter().map(|r|serde_json::json!({"instance":instance(&r.instance),"tick":r.tick,"outcome":r.outcome,"responsible":r.responsible.map(|by|serde_json::json!({"entity":actors.label(scene,by.instance),"instance_id":by.instance.id(),"execution_id":by.execution_id}))})).collect::<Vec<_>>(),"capacity":contribution_capacity(actors.actors.len()),"per_owner_capacity":contribution_quota(actors.actors.len()),"per_source_capacity":contribution_quota(actors.actors.len()),"dropped_receipts":self.dropped_receipts})
    }
}

impl Registry {
    pub(crate) fn checkpoint_bases(&self) -> Vec<Vec<(u8, Value)>> {
        self.bases.clone()
    }
    pub(crate) fn restore_checkpoint_bases(
        &mut self,
        bases: Vec<Vec<(u8, Value)>>,
    ) -> crate::model::Result<()> {
        if bases.len() != self.bases.len()
            || bases.iter().any(|b| {
                b.len() > 65
                    || b.iter().any(|(i, v)| {
                        self.definitions
                            .get(*i as usize)
                            .is_none_or(|d| !d.valid_value(*v))
                    })
            })
        {
            return Err(invalid("Checkpoint attribute base dimensions"));
        }
        for row in &bases {
            let mut seen = [false; 256];
            for (i, _) in row {
                if seen[*i as usize] {
                    return Err(invalid("Duplicate checkpoint attribute base"));
                }
                seen[*i as usize] = true;
            }
        }
        self.bases = bases;
        Ok(())
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;
    use crate::{
        model::{BodyKind, RoomRecipe},
        targeting::Catalog,
    };
    use serde_json::json;

    fn fixture() -> (Registry, Catalog, Scene) {
        let mut scene = RoomRecipe::default().generate().unwrap();
        let mut partner = scene
            .entities
            .iter()
            .find(|e| e.body == Some(BodyKind::Player))
            .unwrap()
            .clone();
        partner.id = "partner".into();
        scene.entities.push(partner);
        scene.ability_catalog = serde_json::from_value(json!({"resources":[],"abilities":[],"attributes":{"buffs":[
            {"id":"add","duration_ticks":17,"reapply":"independent","modifiers":[{"attribute":"combat.damage_received.multiplier","priority":2,"operation":{"kind":"add","units":700}}]},
            {"id":"factor","duration_ticks":13,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_received.multiplier","priority":1,"operation":{"kind":"multiply","basis_points":5000}}]}
        ]}})).unwrap();
        scene.validate().unwrap();
        (Registry::new(&scene, &[]), Catalog::new(&scene), scene)
    }

    fn assert_index(registry: &Registry, actors: &Catalog, state: &State, tick: u64) {
        for &entity in &actors.actors {
            for buff in 0..2 {
                let indexed: Vec<_> = state.active.for_buff(entity, buff).map(|i| i.id).collect();
                let linear: Vec<_> = state
                    .active
                    .iter()
                    .filter(|i| i.owner == entity && i.buff == buff)
                    .map(|i| i.id)
                    .collect();
                assert_eq!(indexed, linear);
            }
            for at in [tick, tick + 30] {
                let fallback = Value::number(SCALE);
                let indexed = registry.evaluate(
                    actors,
                    state,
                    entity,
                    DAMAGE_RECEIVED,
                    fallback,
                    Context::default(),
                    at,
                );
                let linear = registry.resolve_contributions(
                    &registry.bases[entity as usize],
                    |buff| state.active.iter().filter(move |i| i.buff == buff),
                    DAMAGE_RECEIVED,
                    fallback,
                    Context::default(),
                    at,
                    |i| i.owner == entity && actors.entity_matches(entity, i.owner_instance),
                    |_, _, _, _| {},
                );
                assert_eq!(indexed, linear);
            }
        }
        // Empty/non-actor owner and unregistered buff must not expose neighbours.
        assert_eq!(state.active.for_buff(u16::MAX, 0).count(), 0);
        assert_eq!(state.active.for_buff(0, u8::MAX).count(), 0);
    }

    #[test]
    fn owner_definition_index_matches_linear_oracle_after_refresh_compaction_and_serde() {
        let (registry, actors, _) = fixture();
        let mut state = State::default();
        let entries_ptr = state.active.entries.as_ptr();
        let index_ptr = state.active.by_owner_buff.as_ptr();
        for tick in 0..256 {
            let owner = (tick % 2) as u16;
            let source = ((tick / 2) % 2) as u16;
            registry.install(
                &mut state,
                Install {
                    buff: ((tick / 3) % 2) as u8,
                    source: actors.entity(source) as u16,
                    source_instance: actors.reference(source),
                    owner: actors.entity(owner) as u16,
                    owner_instance: actors.reference(owner),
                    execution_id: tick,
                },
                tick,
            );
            if tick % 3 == 0 {
                state.remove(tick, |n, i| {
                    (n % 3 == 0 || tick >= i.expires_at).then_some((End::Expired, None))
                });
            }
            assert_index(&registry, &actors, &state, tick);
            let encoded = serde_json::to_value(&state).unwrap();
            assert!(encoded["active"].is_array());
            let decoded: State = serde_json::from_value(encoded).unwrap();
            assert_eq!(decoded, state);
            assert_index(&registry, &actors, &decoded, tick);
        }
        assert_eq!(state.active.entries.as_ptr(), entries_ptr);
        assert_eq!(state.active.by_owner_buff.as_ptr(), index_ptr);
        assert!(state.receipts.iter().any(|r| r.outcome == End::Refreshed));
        state.active.clear();
        assert_index(&registry, &actors, &state, 256);
    }

    #[test]
    fn indexed_contributions_filter_obsolete_instances_and_reject_duplicate_ids() {
        let (registry, actors, _) = fixture();
        let mut state = State::default();
        registry.install(
            &mut state,
            Install {
                buff: 0,
                source: actors.entity(0) as u16,
                source_instance: actors.reference(0),
                owner: actors.entity(0) as u16,
                // Wrong owner's generation/instance even though the numeric entity matches.
                owner_instance: actors.reference(1),
                execution_id: 1,
            },
            0,
        );
        assert_index(&registry, &actors, &state, 0);
        assert_eq!(
            registry.evaluate(
                &actors,
                &state,
                actors.entity(0) as u16,
                DAMAGE_RECEIVED,
                Value::number(SCALE),
                Context::default(),
                0
            ),
            Value::number(SCALE)
        );
        let entry = serde_json::to_value(state.active[0]).unwrap();
        assert!(serde_json::from_value::<Contributions>(json!(vec![entry; CAPACITY + 1])).is_err());
    }
}
