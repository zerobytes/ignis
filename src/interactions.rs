//! Game-defined interactions and dialogue; effects are typed, never Rust reflection.
use crate::{
    identity::Reference,
    model::{BodyKind, Result, Scene, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const POINTS: usize = 64;
pub const ACTIONS: usize = 32;
pub const VALUES: usize = 64;
pub const RECEIPTS: usize = 256;
const LIMIT: i64 = crate::abilities::LIMIT;
fn label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Document {
    pub values: Vec<Value>,
    pub events: Vec<String>,
    pub points: Vec<Point>,
    pub actions: Vec<Action>,
    pub dialogues: Vec<Dialogue>,
}
impl Document {
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
            && self.events.is_empty()
            && self.points.is_empty()
            && self.actions.is_empty()
            && self.dialogues.is_empty()
    }
    pub fn validate(&self, scene: &Scene) -> Result<()> {
        if self.values.len() > VALUES
            || self.events.len() > VALUES
            || self.points.len() > POINTS
            || self.actions.len() > ACTIONS
            || self.dialogues.len() > 16
        {
            return Err(invalid("Interaction document budget exceeded"));
        }
        fn unique<'a>(mut ids: impl Iterator<Item = &'a str>) -> bool {
            let mut seen = std::collections::BTreeSet::new();
            ids.all(|s| label(s) && seen.insert(s))
        }
        if !unique(self.values.iter().map(|v| v.id.as_str()))
            || !unique(self.events.iter().map(String::as_str))
            || !unique(self.points.iter().map(|v| v.id.as_str()))
            || !unique(self.actions.iter().map(|v| v.id.as_str()))
            || !unique(self.dialogues.iter().map(|v| v.id.as_str()))
        {
            return Err(invalid("Interaction IDs must be unique ASCII identifiers"));
        }
        for value in &self.values {
            if value.minimum < -LIMIT
                || value.maximum > LIMIT
                || value.minimum > value.maximum
                || !(value.minimum..=value.maximum).contains(&value.initial)
            {
                return Err(invalid("Invalid interaction value bounds/initial"));
            }
        }
        let mut anchors = std::collections::BTreeSet::new();
        if self
            .points
            .iter()
            .flat_map(|p| p.tags.iter())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            > 64
        {
            return Err(invalid("At most64interaction tags"));
        }
        for point in &self.points {
            let e = scene
                .entities
                .iter()
                .find(|e| e.id == point.entity)
                .ok_or_else(|| invalid("Unknown interaction point entity"))?;
            if !anchors.insert(&point.entity)
                || (!point.mutable && point.initial != ObjectPatch::default())
                || point
                    .initial
                    .sound_transmission
                    .is_some_and(|v| !v.is_finite() || !(0. ..=1.).contains(&v))
                || point.offset.iter().any(|v| !v.is_finite() || v.abs() > 32.)
                || point
                    .approach
                    .is_some_and(|p| p.iter().any(|v| !v.is_finite() || v.abs() > 10000.))
                || point.tags.len() > 8
                || !unique(point.tags.iter().map(String::as_str))
                || point.actions.len() > ACTIONS
                || !unique(point.actions.iter().map(String::as_str))
                || point
                    .actions
                    .iter()
                    .any(|id| !self.actions.iter().any(|a| a.id == *id))
            {
                return Err(invalid(
                    "Invalid interaction point offset/tags/actions or duplicate anchor",
                ));
            }
            if point.mutable
                && (e.body != Some(BodyKind::Fixed)
                    || e.trigger.is_some()
                    || scene.entities.iter().chain(&scene.npc_templates).any(|n| {
                        n.npc.as_ref().is_some_and(|n| n.surface == e.id)
                            || n.footsteps.as_ref().is_some_and(|f| f.surface == e.id)
                    }))
            {
                return Err(invalid(
                    "Mutable object requires a fixed non-surface, non-trigger entity",
                ));
            }
        }
        for action in &self.actions {
            if !action.range.is_finite()
                || !(0.1..=8.).contains(&action.range)
                || action.cooldown_ticks > 36000
                || action
                    .dialogue
                    .as_ref()
                    .is_some_and(|id| !self.dialogues.iter().any(|d| d.id == *id))
            {
                return Err(invalid(
                    "Interaction requires range [0.1,8], cooldown<=36000 and known dialogue",
                ));
            }
            self.validate_conditions(&action.conditions, scene)?;
            self.validate_effects(&action.effects, scene)?;
        }
        for dialogue in &self.dialogues {
            if dialogue.nodes.is_empty()
                || dialogue.nodes.len() > 32
                || !unique(dialogue.nodes.iter().map(|n| n.id.as_str()))
                || !dialogue.nodes.iter().any(|n| n.id == dialogue.entry)
                || !dialogue.range.is_finite()
                || !(0.1..=8.).contains(&dialogue.range)
                || !(1..=36000).contains(&dialogue.session_ticks)
            {
                return Err(invalid("Invalid dialogue nodes/entry/range/session budget"));
            }
            for node in &dialogue.nodes {
                if node.text.len() > 4096
                    || node.choices.len() > 8
                    || !unique(node.choices.iter().map(|c| c.id.as_str()))
                {
                    return Err(invalid("Dialogue text/choice budget or IDs invalid"));
                }
                for choice in &node.choices {
                    if choice.text.len() > 1024
                        || choice
                            .next
                            .as_ref()
                            .is_some_and(|id| !dialogue.nodes.iter().any(|n| n.id == *id))
                        || choice
                            .personality_weights
                            .iter()
                            .any(|w| w.unsigned_abs() > 1000)
                    {
                        return Err(invalid(
                            "Dialogue choice text/next/personality weights invalid",
                        ));
                    }
                    self.validate_conditions(&choice.conditions, scene)?;
                    self.validate_effects(&choice.effects, scene)?;
                }
            }
        }
        Ok(())
    }
    fn resource(&self, scene: &Scene, id: &str) -> bool {
        scene.ability_catalog.resources.iter().any(|r| r.id == id)
    }
    fn validate_conditions(&self, conditions: &[Condition], scene: &Scene) -> Result<()> {
        if conditions.len() > 8 {
            return Err(invalid("At most8interaction conditions"));
        }
        for c in conditions {
            let valid = match c {
                Condition::Value {
                    key,
                    minimum,
                    maximum,
                } => {
                    self.values.iter().any(|v| v.id == *key)
                        && minimum <= maximum
                        && *minimum >= -LIMIT
                        && *maximum <= LIMIT
                }
                Condition::Resource {
                    resource,
                    minimum,
                    maximum,
                } => {
                    self.resource(scene, resource)
                        && minimum <= maximum
                        && *minimum >= 0
                        && *maximum <= LIMIT
                }
            };
            if !valid {
                return Err(invalid("Invalid interaction condition key/bounds"));
            }
        }
        Ok(())
    }
    fn validate_effects(&self, effects: &[Effect], scene: &Scene) -> Result<()> {
        if effects.len() > 8 {
            return Err(invalid("At most8interaction effects"));
        }
        for effect in effects {
            let valid = match effect {
                Effect::SetValue { key, value } => self
                    .values
                    .iter()
                    .any(|v| v.id == *key && (v.minimum..=v.maximum).contains(value)),
                Effect::AddValue { key, delta } => {
                    self.values.iter().any(|v| v.id == *key) && (-LIMIT..=LIMIT).contains(delta)
                }
                Effect::Resource { resource, delta } => {
                    self.resource(scene, resource) && (-LIMIT..=LIMIT).contains(delta)
                }
                Effect::Object { point, state } => {
                    self.points.iter().any(|p| p.id == *point && p.mutable)
                        && state
                            .sound_transmission
                            .is_none_or(|v| v.is_finite() && (0. ..=1.).contains(&v))
                }
                Effect::Emit { event, .. } => self.events.contains(event),
            };
            if !valid {
                return Err(invalid(
                    "Invalid interaction effect key/value/resource/object/event",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Value {
    pub id: String,
    pub initial: i64,
    pub minimum: i64,
    pub maximum: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Point {
    #[serde(default)]
    pub initial: ObjectPatch,
    pub id: String,
    pub entity: String,
    #[serde(default)]
    pub offset: [f32; 3],
    #[serde(default)]
    pub approach: Option<[f32; 3]>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub actions: Vec<String>,
    #[serde(default)]
    pub mutable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub range: f32,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub cooldown_ticks: u32,
    #[serde(default)]
    pub dialogue: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    Value {
        key: String,
        minimum: i64,
        maximum: i64,
    },
    Resource {
        resource: String,
        minimum: i64,
        maximum: i64,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    SetValue {
        key: String,
        value: i64,
    },
    AddValue {
        key: String,
        delta: i64,
    },
    Resource {
        resource: String,
        delta: i64,
    },
    Object {
        point: String,
        state: ObjectPatch,
    },
    Emit {
        event: String,
        #[serde(default)]
        value: i64,
    },
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ObjectPatch {
    pub solid: Option<bool>,
    pub visible: Option<bool>,
    pub blocks_vision: Option<bool>,
    pub sound_transmission: Option<f32>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ObjectState {
    pub solid: bool,
    pub visible: bool,
    pub blocks_vision: bool,
    pub sound_transmission: f32,
}
impl ObjectState {
    pub fn from_entity(e: &crate::model::Entity) -> Self {
        Self {
            solid: e.body.is_some(),
            visible: e.visible != Some(false),
            blocks_vision: e.blocks_vision.unwrap_or(e.body.is_some()),
            sound_transmission: e.sound_transmission.unwrap_or(1.),
        }
    }
    pub fn patched(self, p: ObjectPatch) -> Self {
        Self {
            solid: p.solid.unwrap_or(self.solid),
            visible: p.visible.unwrap_or(self.visible),
            blocks_vision: p.blocks_vision.unwrap_or(self.blocks_vision),
            sound_transmission: p.sound_transmission.unwrap_or(self.sound_transmission),
        }
    }
}
fn dialogue_range() -> f32 {
    3.
}
fn session_ticks() -> u32 {
    600
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Dialogue {
    pub id: String,
    pub entry: String,
    pub nodes: Vec<Node>,
    #[serde(default = "dialogue_range")]
    pub range: f32,
    #[serde(default = "session_ticks")]
    pub session_ticks: u32,
    #[serde(default)]
    pub lock_locomotion: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub text: String,
    pub choices: Vec<Choice>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub personality_weights: [i16; 9],
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Use {
        point: u8,
        action: u8,
        instance: Reference,
        life: Option<u64>,
    },
    Talk {
        target: u16,
        instance: Reference,
        life: u64,
        dialogue: u8,
    },
    Choose {
        session: u64,
        revision: u64,
        choice: u8,
    },
    Close {
        session: u64,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Anchor {
    pub entity: u16,
    pub instance: Reference,
    pub life: Option<u64>,
    pub offset: [f32; 3],
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub revision: u64,
    pub phase: SessionPhase,
    pub serial: u64,
    pub actor: Reference,
    pub life: u64,
    pub anchor: Anchor,
    pub dialogue: u8,
    pub node: u8,
    pub expires_at: u64,
    pub range: f32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Active,
    Closed,
    Cancelled,
    Expired,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Applied,
    Opened,
    Advanced,
    Closed,
    Stale,
    OutOfRange,
    ConditionFailed,
    InsufficientResource,
    ValueBounds,
    Occupied,
    Cooldown,
    Busy,
    Unavailable,
    SessionExhausted,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub tick: u64,
    pub request: Option<u64>,
    pub actor: Reference,
    pub operation: Request,
    pub outcome: Outcome,
    pub events: [Option<(u8, i64)>; 8],
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct State {
    pub auto_cursor: u16,
    pub values: Vec<i64>,
    pub objects: Vec<ObjectState>,
    pub anchors: Vec<Option<Reference>>,
    pub sessions: Vec<Option<Session>>,
    pub cooldowns: Vec<[u64; ACTIONS]>,
    pub next_session: u64,
    pub receipts: std::collections::VecDeque<Receipt>,
    pub processed: u64,
    pub dropped: u64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            auto_cursor: 0,
            values: Vec::new(),
            objects: Vec::new(),
            anchors: Vec::new(),
            sessions: Vec::new(),
            cooldowns: Vec::new(),
            next_session: 1,
            receipts: Default::default(),
            processed: 0,
            dropped: 0,
        }
    }
}
impl State {
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
            && self.anchors.is_empty()
            && self.auto_cursor == 0
            && self.objects.is_empty()
            && self.sessions.is_empty()
            && self.cooldowns.is_empty()
            && self.receipts.is_empty()
            && self.next_session == 1
            && self.processed == 0
            && self.dropped == 0
    }
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(remote = "State")]
        struct Wire {
            pub auto_cursor: u16,
            pub values: Vec<i64>,
            pub objects: Vec<ObjectState>,
            pub anchors: Vec<Option<Reference>>,
            pub sessions: Vec<Option<Session>>,
            pub cooldowns: Vec<[u64; ACTIONS]>,
            pub next_session: u64,
            pub receipts: std::collections::VecDeque<Receipt>,
            pub processed: u64,
            pub dropped: u64,
        }
        let s = Wire::deserialize(d)?;
        let valid_ref = |r: Reference| {
            r.instance > 0 && r.generation > 0 && (r.slot as usize) < crate::model::MAX_ENTITIES
        };
        if s.auto_cursor as usize >= crate::targeting::MAX_ACTORS
            || s.values.len() > VALUES
            || s.values.iter().any(|v| !(-LIMIT..=LIMIT).contains(v))
            || s.objects.len() > POINTS
            || s.anchors.len() != s.objects.len()
            || s.sessions.len() > crate::targeting::MAX_ACTORS
            || s.cooldowns.len() > crate::targeting::MAX_ACTORS
            || s.receipts.len() > RECEIPTS
            || s.next_session == 0
            || s.objects.iter().any(|o| {
                !o.sound_transmission.is_finite() || !(0. ..=1.).contains(&o.sound_transmission)
            })
            || s.anchors.iter().flatten().any(|r| !valid_ref(*r))
            || s.sessions.iter().flatten().any(|v| {
                v.serial == 0
                    || v.serial >= s.next_session
                    || v.dialogue >= 16
                    || v.node >= 32
                    || !valid_ref(v.actor)
                    || !valid_ref(v.anchor.instance)
                    || v.anchor.entity as usize >= crate::model::MAX_ENTITIES
                    || !v.range.is_finite()
                    || !(0.1..=8.).contains(&v.range)
                    || v.anchor
                        .offset
                        .iter()
                        .any(|v| !v.is_finite() || v.abs() > 32.)
            })
        {
            return Err(serde::de::Error::custom(
                "Invalid interaction state budget/session/identity",
            ));
        }
        Ok(s)
    }
}

#[derive(Clone, Copy)]
enum Test {
    Value(u8, i64, i64),
    Resource(u8, i64, i64),
}
#[derive(Clone, Copy)]
enum Write {
    Set(u8, i64),
    Add(u8, i64),
    Resource(u8, i64),
    Object(u8, ObjectPatch),
    Emit(u8, i64),
}
struct Program {
    tests: Vec<Test>,
    writes: Vec<Write>,
}
pub(crate) struct CompiledPoint {
    pub entity: u16,
    pub offset: [f32; 3],
    pub tags: u64,
    pub actions: u32,
    pub stationary: bool,
}
pub struct Runtime {
    pub document: std::sync::Arc<Document>,
    pub state: State,
    pub(crate) points: Vec<CompiledPoint>,
    tags: Vec<String>,
    actions: Vec<Program>,
    choices: Vec<Vec<Vec<Program>>>,
}
pub(crate) struct World<'a> {
    pub scene: &'a Scene,
    pub actors: &'a crate::targeting::Catalog,
    pub vitality: &'a crate::vitality::Runtime,
    pub abilities: &'a crate::abilities::Runtime,
    pub position: &'a dyn Fn(u16) -> Option<[f32; 3]>,
    pub can_close: &'a dyn Fn(u16) -> bool,
    pub actor: u16,
    pub tick: u64,
}
pub(crate) struct Plan {
    pub resources: crate::transaction::Writes<u8, i64, 8>,
    pub objects: crate::transaction::Writes<u8, ObjectState, 8>,
    values: crate::transaction::Writes<u8, i64, 8>,
    session: Option<Option<Session>>,
    cooldown: Option<(u8, u64)>,
    pub outcome: Outcome,
    pub events: [Option<(u8, i64)>; 8],
}
impl Default for Plan {
    fn default() -> Self {
        Self {
            resources: Default::default(),
            objects: Default::default(),
            values: Default::default(),
            session: None,
            cooldown: None,
            outcome: Outcome::Applied,
            events: [None; 8],
        }
    }
}
impl Runtime {
    pub fn new(scene: &Scene, actors: &crate::targeting::Catalog) -> Self {
        let document = std::sync::Arc::new(scene.interactions.clone());
        let tags: Vec<_> = document
            .points
            .iter()
            .flat_map(|p| p.tags.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut anchors = Vec::with_capacity(document.points.len());
        let mut objects = Vec::with_capacity(document.points.len());
        let points = document
            .points
            .iter()
            .map(|p| {
                let entity = scene
                    .entities
                    .iter()
                    .position(|e| e.id == p.entity)
                    .unwrap();
                let e = &scene.entities[entity];
                anchors.push(actors.entity_reference(entity));
                objects.push(ObjectState::from_entity(e).patched(p.initial));
                CompiledPoint {
                    entity: entity as u16,
                    offset: p.offset,
                    tags: p.tags.iter().fold(0, |mask, id| {
                        mask | 1 << tags.iter().position(|t| t == id).unwrap()
                    }),
                    actions: p.actions.iter().fold(0, |mask, id| {
                        mask | 1 << document.actions.iter().position(|a| a.id == *id).unwrap()
                    }),
                    stationary: !matches!(
                        e.body,
                        Some(BodyKind::Player | BodyKind::Npc | BodyKind::Dynamic)
                    ),
                }
            })
            .collect();
        let program = |conditions: &[Condition], effects: &[Effect]| {
            let value = |key: &str| document.values.iter().position(|v| v.id == key).unwrap() as u8;
            let resource = |id: &str| {
                scene
                    .ability_catalog
                    .resources
                    .iter()
                    .position(|r| r.id == id)
                    .unwrap() as u8
            };
            Program {
                tests: conditions
                    .iter()
                    .map(|c| match c {
                        Condition::Value {
                            key,
                            minimum,
                            maximum,
                        } => Test::Value(value(key), *minimum, *maximum),
                        Condition::Resource {
                            resource: id,
                            minimum,
                            maximum,
                        } => Test::Resource(resource(id), *minimum, *maximum),
                    })
                    .collect(),
                writes: effects
                    .iter()
                    .map(|e| match e {
                        Effect::SetValue { key, value: v } => Write::Set(value(key), *v),
                        Effect::AddValue { key, delta } => Write::Add(value(key), *delta),
                        Effect::Resource {
                            resource: id,
                            delta,
                        } => Write::Resource(resource(id), *delta),
                        Effect::Object { point, state } => Write::Object(
                            document.points.iter().position(|p| p.id == *point).unwrap() as u8,
                            *state,
                        ),
                        Effect::Emit { event, value } => Write::Emit(
                            document.events.iter().position(|id| id == event).unwrap() as u8,
                            *value,
                        ),
                    })
                    .collect(),
            }
        };
        let actions = document
            .actions
            .iter()
            .map(|a| program(&a.conditions, &a.effects))
            .collect();
        let choices = document
            .dialogues
            .iter()
            .map(|d| {
                d.nodes
                    .iter()
                    .map(|n| {
                        n.choices
                            .iter()
                            .map(|c| program(&c.conditions, &c.effects))
                            .collect()
                    })
                    .collect()
            })
            .collect();
        let count = actors.actors.len();
        let state = State {
            auto_cursor: 0,
            values: document.values.iter().map(|v| v.initial).collect(),
            objects,
            anchors,
            sessions: if document.dialogues.is_empty() {
                Vec::new()
            } else {
                vec![None; count]
            },
            cooldowns: if document.actions.is_empty() {
                Vec::new()
            } else {
                vec![[0; ACTIONS]; count]
            },
            next_session: 1,
            receipts: if document.is_empty() {
                Default::default()
            } else {
                std::collections::VecDeque::with_capacity(RECEIPTS)
            },
            processed: 0,
            dropped: 0,
        };
        Self {
            document,
            state,
            points,
            tags,
            actions,
            choices,
        }
    }
    pub(crate) fn tag(&self, id: &str) -> Option<u8> {
        self.tags.iter().position(|v| v == id).map(|i| i as u8)
    }
    fn current_anchor(&self, w: &World<'_>, anchor: Anchor) -> bool {
        w.actors.entity_matches(anchor.entity, anchor.instance)
            && anchor.life.is_none_or(|life| {
                w.actors
                    .actor_slot(anchor.instance)
                    .is_some_and(|a| !w.vitality.dead(a) && w.vitality.state.lives[a] == life)
            })
    }
    pub(crate) fn point_anchor(&self, w: &World<'_>, point: u8) -> Option<Anchor> {
        let p = self.points.get(point as usize)?;
        let instance = self.state.anchors[point as usize]?;
        let life = w
            .actors
            .actor_slot(instance)
            .map(|a| w.vitality.state.lives[a]);
        let anchor = Anchor {
            entity: p.entity,
            instance,
            life,
            offset: p.offset,
        };
        self.current_anchor(w, anchor).then_some(anchor)
    }
    pub(crate) fn validate_request(&self, w: &World<'_>, request: Request) -> Result<()> {
        let valid = match request {
            Request::Use {
                point,
                action,
                instance,
                life,
            } => {
                self.points.get(point as usize).is_some_and(|p| {
                    (action as usize) < self.actions.len() && p.actions & (1u32 << action) != 0
                }) && self
                    .point_anchor(w, point)
                    .is_some_and(|a| a.instance == instance && a.life == life)
            }
            Request::Talk {
                target,
                instance,
                life,
                dialogue,
            } => {
                w.actors.matches(target, instance)
                    && !w.vitality.dead(target as usize)
                    && w.vitality.state.lives[target as usize] == life
                    && self
                        .document
                        .dialogues
                        .get(dialogue as usize)
                        .is_some_and(|d| {
                            w.scene.entities[w.actors.entity(target)]
                                .npc
                                .as_ref()
                                .and_then(|n| n.dialogue.as_ref())
                                == Some(&d.id)
                        })
            }
            Request::Choose {
                session,
                revision,
                choice,
            } => self
                .state
                .sessions
                .get(w.actor as usize)
                .copied()
                .flatten()
                .is_some_and(|s| {
                    s.serial == session
                        && s.revision == revision
                        && s.phase == SessionPhase::Active
                        && s.actor == w.actors.reference(w.actor)
                        && s.life == w.vitality.state.lives[w.actor as usize]
                        && w.tick < s.expires_at
                        && self.current_anchor(w, s.anchor)
                        && (choice as usize)
                            < self.document.dialogues[s.dialogue as usize].nodes[s.node as usize]
                                .choices
                                .len()
                }),
            Request::Close { session } => self
                .state
                .sessions
                .get(w.actor as usize)
                .copied()
                .flatten()
                .is_some_and(|s| {
                    s.serial == session
                        && s.phase == SessionPhase::Active
                        && s.actor == w.actors.reference(w.actor)
                        && s.life == w.vitality.state.lives[w.actor as usize]
                }),
        };
        if valid {
            Ok(())
        } else {
            Err(invalid("Stale or invalid interaction/dialogue request"))
        }
    }
    fn within_range(&self, w: &World<'_>, anchor: Anchor, range: f32) -> bool {
        let Some(own) = (w.position)(w.actors.entity(w.actor) as u16) else {
            return false;
        };
        let Some(target) = (w.position)(anchor.entity) else {
            return false;
        };
        let own = glam::Vec3::from(own);
        let target = glam::Vec3::from(target) + glam::Vec3::from(anchor.offset);
        own.distance_squared(target) <= range * range
            && !w.scene.entities.iter().enumerate().any(|(i, e)| {
                i != w.actors.entity(w.actor)
                    && i != anchor.entity as usize
                    && e.blocks_vision.unwrap_or(e.body.is_some())
                    && (w.position)(i as u16).is_some_and(|p| {
                        crate::geometry::Volume::entity(e, p.into())
                            .cast(own, target - own, glam::Vec3::ZERO, true, 0.)
                            .is_some()
                    })
            })
    }
    fn eligible(&self, w: &World<'_>, program: &Program) -> bool {
        program.tests.iter().all(|test| match *test {
            Test::Value(i, min, max) => (min..=max).contains(&self.state.values[i as usize]),
            Test::Resource(i, min, max) => w
                .abilities
                .resource_level(w.actor as usize, i as usize)
                .is_some_and(|(v, _)| (min..=max).contains(&v)),
        })
    }
    fn effects(
        &self,
        w: &World<'_>,
        program: &Program,
        plan: &mut Plan,
    ) -> std::result::Result<(), Outcome> {
        if !self.eligible(w, program) {
            return Err(Outcome::ConditionFailed);
        }
        let mut event = 0;
        for write in &program.writes {
            match *write {
                Write::Set(i, value) => {
                    *plan
                        .values
                        .get_or_insert(i, || self.state.values[i as usize]) = value
                }
                Write::Add(i, delta) => {
                    let value = plan
                        .values
                        .get_or_insert(i, || self.state.values[i as usize]);
                    *value = value.checked_add(delta).ok_or(Outcome::ValueBounds)?;
                }
                Write::Resource(i, delta) => {
                    let (before, capacity) = w
                        .abilities
                        .resource_level(w.actor as usize, i as usize)
                        .ok_or(Outcome::InsufficientResource)?;
                    let value = plan.resources.get_or_insert(i, || before);
                    *value = value
                        .checked_add(delta)
                        .ok_or(Outcome::InsufficientResource)?
                        .min(capacity);
                    if *value < 0 {
                        return Err(Outcome::InsufficientResource);
                    }
                }
                Write::Object(i, patch) => {
                    let value = plan
                        .objects
                        .get_or_insert(i, || self.state.objects[i as usize]);
                    *value = value.patched(patch);
                }
                Write::Emit(i, value) => {
                    plan.events[event] = Some((i, value));
                    event += 1;
                }
            }
        }
        for &(i, value) in plan.values.iter() {
            let def = &self.document.values[i as usize];
            if !(def.minimum..=def.maximum).contains(&value) {
                return Err(Outcome::ValueBounds);
            }
        }
        for &(i, value) in plan.objects.iter() {
            let Some(anchor) = self.point_anchor(w, i) else {
                return Err(Outcome::Unavailable);
            };
            if value.solid && !self.state.objects[i as usize].solid && !(w.can_close)(anchor.entity)
            {
                return Err(Outcome::Occupied);
            }
        }
        Ok(())
    }
    fn open(
        &self,
        w: &World<'_>,
        dialogue: u8,
        anchor: Anchor,
        range: f32,
        plan: &mut Plan,
    ) -> std::result::Result<(), Outcome> {
        if self.state.sessions[w.actor as usize]
            .is_some_and(|s| s.phase == SessionPhase::Active && w.tick < s.expires_at)
        {
            return Err(Outcome::Busy);
        }
        if self.state.next_session == u64::MAX {
            return Err(Outcome::SessionExhausted);
        }
        let d = &self.document.dialogues[dialogue as usize];
        let node = d.nodes.iter().position(|n| n.id == d.entry).unwrap() as u8;
        let phase = if d.nodes[node as usize].choices.is_empty() {
            SessionPhase::Closed
        } else {
            SessionPhase::Active
        };
        plan.session = Some(Some(Session {
            revision: 0,
            phase,
            serial: self.state.next_session,
            actor: w.actors.reference(w.actor),
            life: w.vitality.state.lives[w.actor as usize],
            anchor,
            dialogue,
            node,
            expires_at: w.tick.saturating_add(u64::from(d.session_ticks)),
            range,
        }));
        plan.outcome = if phase == SessionPhase::Active {
            Outcome::Opened
        } else {
            Outcome::Closed
        };
        Ok(())
    }
    pub(crate) fn prepare(
        &self,
        w: &World<'_>,
        request: Request,
    ) -> std::result::Result<Plan, Outcome> {
        self.validate_request(w, request)
            .map_err(|_| Outcome::Stale)?;
        let mut plan = Plan::default();
        match request {
            Request::Use { point, action, .. } => {
                let a = &self.document.actions[action as usize];
                let anchor = self.point_anchor(w, point).ok_or(Outcome::Unavailable)?;
                if !self.within_range(w, anchor, a.range) {
                    return Err(Outcome::OutOfRange);
                }
                if w.tick < self.state.cooldowns[w.actor as usize][action as usize] {
                    return Err(Outcome::Cooldown);
                }
                self.effects(w, &self.actions[action as usize], &mut plan)?;
                if let Some(id) = &a.dialogue {
                    let d = self
                        .document
                        .dialogues
                        .iter()
                        .position(|d| d.id == *id)
                        .unwrap() as u8;
                    self.open(w, d, anchor, a.range, &mut plan)?;
                }
                plan.cooldown = Some((action, w.tick.saturating_add(u64::from(a.cooldown_ticks))));
            }
            Request::Talk {
                target,
                instance,
                life,
                dialogue,
            } => {
                let anchor = Anchor {
                    entity: w.actors.entity(target) as u16,
                    instance,
                    life: Some(life),
                    offset: [0.; 3],
                };
                let range = self.document.dialogues[dialogue as usize].range;
                if !self.within_range(w, anchor, range) {
                    return Err(Outcome::OutOfRange);
                }
                self.open(w, dialogue, anchor, range, &mut plan)?;
            }
            Request::Choose { choice, .. } => {
                let s = self.state.sessions[w.actor as usize].unwrap();
                if !self.within_range(w, s.anchor, s.range) {
                    return Err(Outcome::OutOfRange);
                }
                self.effects(
                    w,
                    &self.choices[s.dialogue as usize][s.node as usize][choice as usize],
                    &mut plan,
                )?;
                let d = &self.document.dialogues[s.dialogue as usize];
                let choice = &d.nodes[s.node as usize].choices[choice as usize];
                let node = choice
                    .next
                    .as_ref()
                    .map(|id| d.nodes.iter().position(|n| n.id == *id).unwrap() as u8);
                let phase = if node.is_none_or(|n| d.nodes[n as usize].choices.is_empty()) {
                    SessionPhase::Closed
                } else {
                    SessionPhase::Active
                };
                plan.session = Some(Some(Session {
                    revision: s.revision.checked_add(1).ok_or(Outcome::SessionExhausted)?,
                    node: node.unwrap_or(s.node),
                    phase,
                    ..s
                }));
                plan.outcome = if phase == SessionPhase::Active {
                    Outcome::Advanced
                } else {
                    Outcome::Closed
                };
            }
            Request::Close { .. } => {
                plan.session = Some(Some(Session {
                    phase: SessionPhase::Cancelled,
                    ..self.state.sessions[w.actor as usize].unwrap()
                }));
                plan.outcome = Outcome::Closed;
            }
        }
        Ok(plan)
    }
    pub(crate) fn commit(&mut self, actor: u16, plan: &Plan) {
        for &(i, value) in plan.values.iter() {
            self.state.values[i as usize] = value;
        }
        for &(i, value) in plan.objects.iter() {
            self.state.objects[i as usize] = value;
        }
        if let Some((action, tick)) = plan.cooldown {
            self.state.cooldowns[actor as usize][action as usize] = tick;
        }
        if let Some(session) = plan.session {
            if session.is_some_and(|s| s.serial == self.state.next_session) {
                self.state.next_session += 1;
            }
            self.state.sessions[actor as usize] = session;
        }
    }
    pub(crate) fn record(
        &mut self,
        tick: u64,
        request: Option<u64>,
        actor: Reference,
        operation: Request,
        outcome: Outcome,
        events: [Option<(u8, i64)>; 8],
    ) {
        if self.state.receipts.len() == RECEIPTS {
            self.state.receipts.pop_front();
            self.state.dropped = self.state.dropped.saturating_add(1);
        }
        self.state.receipts.push_back(Receipt {
            tick,
            request,
            actor,
            operation,
            outcome,
            events,
        });
        self.state.processed = self.state.processed.saturating_add(1);
    }
    pub(crate) fn cleanup(
        &mut self,
        actors: &crate::targeting::Catalog,
        vitality: &crate::vitality::Runtime,
        tick: u64,
    ) {
        for (actor, session) in self.state.sessions.iter_mut().enumerate() {
            if let Some(s) = *session
                && (!actors.matches(actor as u16, s.actor)
                    || vitality.dead(actor)
                    || vitality.state.lives[actor] != s.life
                    || !actors.entity_matches(s.anchor.entity, s.anchor.instance)
                    || s.anchor.life.is_some_and(|life| {
                        actors
                            .actor_slot(s.anchor.instance)
                            .is_none_or(|a| vitality.dead(a) || vitality.state.lives[a] != life)
                    }))
            {
                *session = None;
            } else if let Some(s) = session
                && s.phase == SessionPhase::Active
                && tick >= s.expires_at
            {
                s.phase = SessionPhase::Expired;
            }
        }
    }
    pub(crate) fn movement_locks(
        &self,
        actors: &crate::targeting::Catalog,
    ) -> crate::actor_set::ActorSet {
        let mut locks = crate::actor_set::ActorSet::EMPTY;
        for (a, s) in self
            .state
            .sessions
            .iter()
            .enumerate()
            .filter_map(|(a, s)| s.map(|s| (a, s)))
        {
            if s.phase == SessionPhase::Active
                && self.document.dialogues[s.dialogue as usize].lock_locomotion
            {
                locks.insert(a);
                if let Some(a) = actors.actor_slot(s.anchor.instance) {
                    locks.insert(a);
                }
            }
        }
        locks
    }
    pub(crate) fn inspect_dialogue(&self, w: &World<'_>) -> serde_json::Value {
        let Some(s) = self
            .state
            .sessions
            .get(w.actor as usize)
            .copied()
            .flatten()
            .filter(|s| {
                s.actor == w.actors.reference(w.actor)
                    && s.life == w.vitality.state.lives[w.actor as usize]
                    && self.current_anchor(w, s.anchor)
            })
        else {
            return serde_json::json!({"session":null});
        };
        let d = &self.document.dialogues[s.dialogue as usize];
        let node = &d.nodes[s.node as usize];
        if s.phase != SessionPhase::Active || w.tick >= s.expires_at {
            return serde_json::json!({"session":null,"last":s,"dialogue":d.id,"node":node.id,"text":node.text});
        }
        let range = self.within_range(w, s.anchor, s.range);
        let choices:Vec<_>=node.choices.iter().enumerate().map(|(i,c)|serde_json::json!({"id":c.id,"text":c.text,"eligible":range&&self.eligible(w,&self.choices[s.dialogue as usize][s.node as usize][i])})).collect();
        serde_json::json!({"session":s,"dialogue":d.id,"node":node.id,"text":node.text,"choices":choices,"in_range":range})
    }
    pub(crate) fn choose_dialogue(
        &self,
        w: &World<'_>,
        profile: crate::npc_mind::Profile,
        cursor: u8,
    ) -> (Option<Request>, [Option<i32>; 8], u8) {
        let mut scores = [None; 8];
        let Some(session) = self
            .state
            .sessions
            .get(w.actor as usize)
            .copied()
            .flatten()
            .filter(|s| {
                s.phase == SessionPhase::Active
                    && s.actor == w.actors.reference(w.actor)
                    && s.life == w.vitality.state.lives[w.actor as usize]
                    && w.tick < s.expires_at
                    && self.current_anchor(w, s.anchor)
                    && self.within_range(w, s.anchor, s.range)
            })
        else {
            return (None, scores, 0);
        };
        let choices = &self.document.dialogues[session.dialogue as usize].nodes
            [session.node as usize]
            .choices;
        let budget = (1 + usize::from(profile.intelligence) * 7 / 10000).min(choices.len());
        let mut best: Option<(u8, i32)> = None;
        for offset in 0..budget {
            let at = (usize::from(cursor) + offset) % choices.len();
            let program = &self.choices[session.dialogue as usize][session.node as usize][at];
            if !self.eligible(w, program) {
                continue;
            }
            let mut costs = crate::transaction::Writes::<u8, i64, 8>::default();
            let affordable = program.writes.iter().all(|write| {
                let Write::Resource(i, delta) = *write else {
                    return true;
                };
                let Some((value, capacity)) =
                    w.abilities.resource_level(w.actor as usize, i as usize)
                else {
                    return false;
                };
                let value = costs.get_or_insert(i, || value);
                let Some(next) = value.checked_add(delta) else {
                    return false;
                };
                *value = next.min(capacity);
                *value >= 0
            });
            if !affordable {
                continue;
            }
            let score = choices[at]
                .personality_weights
                .iter()
                .zip(profile.traits)
                .map(|(&w, v)| i32::from(w) * i32::from(v) / 1000)
                .sum();
            scores[at] = Some(score);
            if best.is_none_or(|(old, s)| score > s || (score == s && at < usize::from(old))) {
                best = Some((at as u8, score));
            }
        }
        (
            best.map(|(choice, _)| Request::Choose {
                session: session.serial,
                revision: session.revision,
                choice,
            }),
            scores,
            budget as u8,
        )
    }
}
