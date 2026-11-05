//! Versioned game UI authoring, independent of scene, gameplay and egui memory.
//! egui remains responsible for layout, clipping, hit testing and native focus.
mod runtime;
use crate::model::{Result, invalid};
pub use runtime::{Capture, Event, Frame, Runtime, Value};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

pub const MAX_BYTES: usize = 256 * 1024;
pub const MAX_NODES: usize = 256;
pub const MAX_DEPTH: usize = 16;
pub const MAX_HISTORY: usize = 32;
pub const MAX_TEXT: usize = 4096;

pub(crate) fn key(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 64
        || !s
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    {
        return Err(invalid(
            "UI keys require 1..64 ASCII letters, digits, _, . or -",
        ));
    }
    Ok(())
}
fn range(v: f32, min: f32, max: f32) -> Result<()> {
    if !v.is_finite() || !(min..=max).contains(&v) {
        return Err(invalid("UI dimension outside finite bounds"));
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub background: [u8; 4],
    pub surface: [u8; 4],
    pub text: [u8; 4],
    pub normal: [u8; 4],
    pub hover: [u8; 4],
    pub pressed: [u8; 4],
    pub disabled: [u8; 4],
    pub focus: [u8; 4],
    pub accent: [u8; 4],
    pub font_size: f32,
    pub spacing: f32,
    pub rounding: u8,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            background: [14, 21, 31, 255],
            surface: [23, 34, 48, 255],
            text: [229, 235, 244, 255],
            normal: [38, 57, 76, 255],
            hover: [51, 79, 102, 255],
            pressed: [24, 106, 112, 255],
            disabled: [36, 40, 46, 255],
            focus: [112, 226, 209, 255],
            accent: [67, 196, 179, 255],
            font_size: 17.,
            spacing: 12.,
            rounding: 8,
        }
    }
}
impl Theme {
    fn validate(&self) -> Result<()> {
        range(self.font_size, 8., 48.)?;
        range(self.spacing, 0., 64.)?;
        if self.rounding > 64 {
            return Err(invalid("UI rounding exceeds 64 points"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "source",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Text {
    Literal(String),
    Key(String),
    Binding(String),
}
impl Default for Text {
    fn default() -> Self {
        Self::Literal(String::new())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Column,
    Row,
    Label,
    Heading,
    Button,
    TextInput,
    Progress,
    Viewport,
}
impl Kind {
    pub fn container(self) -> bool {
        matches!(self, Self::Column | Self::Row)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    #[serde(default)]
    pub padding: f32,
    #[serde(default)]
    pub min_height: f32,
    /// Preferred maximum width; always clamped to the available logical width.
    #[serde(default)]
    pub max_width: Option<f32>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub kind: Kind,
    #[serde(default)]
    pub text: Text,
    #[serde(default)]
    pub binding: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub layout: Layout,
    #[serde(default)]
    pub children: Vec<Node>,
}
fn yes() -> bool {
    true
}
impl Node {
    pub fn new(id: impl Into<String>, kind: Kind) -> Self {
        Self {
            id: id.into(),
            kind,
            text: Text::default(),
            binding: None,
            action: None,
            visible: true,
            enabled: true,
            layout: Layout::default(),
            children: vec![],
        }
    }
    pub fn find(&self, id: &str) -> Option<&Self> {
        if self.id == id {
            Some(self)
        } else {
            self.children.iter().find_map(|n| n.find(id))
        }
    }
    fn find_mut(&mut self, id: &str) -> Option<&mut Self> {
        if self.id == id {
            Some(self)
        } else {
            self.children.iter_mut().find_map(|n| n.find_mut(id))
        }
    }
    fn take(&mut self, id: &str) -> Option<Self> {
        if let Some(i) = self.children.iter().position(|n| n.id == id) {
            return Some(self.children.remove(i));
        }
        self.children.iter_mut().find_map(|n| n.take(id))
    }
    /// Reusable authored subtree. Bindings/actions intentionally remain consumer keys.
    pub fn instantiate(&self, prefix: &str) -> Result<Self> {
        key(prefix)?;
        let mut node = self.clone();
        fn rename(n: &mut Node, p: &str) -> Result<()> {
            n.id = format!("{p}.{}", n.id);
            key(&n.id)?;
            for c in &mut n.children {
                rename(c, p)?;
            }
            Ok(())
        }
        rename(&mut node, prefix)?;
        Ok(node)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u32,
    pub id: String,
    pub theme: Theme,
    #[serde(default)]
    pub translations: BTreeMap<String, BTreeMap<String, String>>,
    pub root: Node,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            id: "ui".into(),
            theme: Theme::default(),
            translations: BTreeMap::new(),
            root: Node::new("root", Kind::Column),
        }
    }
}
impl Document {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(invalid("Unsupported UI document version"));
        }
        key(&self.id)?;
        self.theme.validate()?;
        if !self.root.kind.container() {
            return Err(invalid("UI root must be a container"));
        }
        let mut ids = BTreeSet::new();
        let mut viewports = 0;
        fn visit(
            n: &Node,
            depth: usize,
            ids: &mut BTreeSet<String>,
            views: &mut usize,
        ) -> Result<()> {
            if depth > MAX_DEPTH || ids.len() >= MAX_NODES {
                return Err(invalid("UI hierarchy exceeds depth16 or256 nodes"));
            }
            key(&n.id)?;
            if !ids.insert(n.id.clone()) {
                return Err(invalid("Duplicate UI node ID"));
            }
            range(n.layout.padding, 0., 64.)?;
            range(n.layout.min_height, 0., 1024.)?;
            if let Some(w) = n.layout.max_width {
                range(w, 24., 4096.)?;
            }
            match &n.text {
                Text::Literal(t) if t.len() > MAX_TEXT => {
                    return Err(invalid("UI text exceeds4096 bytes"));
                }
                Text::Key(k) | Text::Binding(k) => key(k)?,
                _ => {}
            }
            for k in [&n.binding, &n.action].into_iter().flatten() {
                key(k)?;
            }
            if matches!(n.kind, Kind::TextInput | Kind::Progress) != n.binding.is_some() {
                return Err(invalid("Only text_input/progress require a binding"));
            }
            if (n.kind == Kind::Button) != n.action.is_some() {
                return Err(invalid("Only buttons require an action key"));
            }
            if !n.kind.container() && !n.children.is_empty() {
                return Err(invalid("Only UI containers accept children"));
            }
            if n.kind == Kind::Viewport {
                *views += 1;
            }
            for c in &n.children {
                visit(c, depth + 1, ids, views)?;
            }
            Ok(())
        }
        visit(&self.root, 0, &mut ids, &mut viewports)?;
        if viewports > 1 {
            return Err(invalid(
                "The current Graphics bridge supports one scene viewport",
            ));
        }
        if self.translations.len() > 8 {
            return Err(invalid("UI supports up to8 locale tables"));
        }
        for (locale, table) in &self.translations {
            key(locale)?;
            if table.len() > 512 {
                return Err(invalid("UI locale table exceeds512 entries"));
            }
            for (k, t) in table {
                key(k)?;
                if t.len() > MAX_TEXT {
                    return Err(invalid("UI translation too long"));
                }
            }
        }
        if serde_json::to_vec(self)
            .map_err(crate::service::io_error)?
            .len()
            > MAX_BYTES
        {
            return Err(invalid("UI document exceeds256 KiB"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    Theme {
        theme: Theme,
    },
    ReplaceNode {
        id: String,
        node: Box<Node>,
    },
    Insert {
        parent: String,
        index: usize,
        node: Box<Node>,
    },
    Remove {
        id: String,
    },
    Move {
        id: String,
        parent: String,
        index: usize,
    },
}
impl Edit {
    fn apply(&self, d: &mut Document) -> Result<()> {
        match self {
            Self::Theme { theme } => d.theme = theme.clone(),
            Self::ReplaceNode { id, node } => {
                if &node.id != id {
                    return Err(invalid("replace_node preserves identity"));
                }
                *d.root
                    .find_mut(id)
                    .ok_or_else(|| invalid("UI node missing"))? = *node.clone();
            }
            Self::Insert {
                parent,
                index,
                node,
            } => insert(&mut d.root, parent, *index, *node.clone())?,
            Self::Remove { id } => {
                d.root
                    .take(id)
                    .ok_or_else(|| invalid("UI node missing or root removal"))?;
            }
            Self::Move { id, parent, index } => {
                let node = d
                    .root
                    .take(id)
                    .ok_or_else(|| invalid("UI node missing or root move"))?;
                insert(&mut d.root, parent, *index, node)?;
            }
        }
        Ok(())
    }
}
fn insert(root: &mut Node, parent: &str, index: usize, node: Node) -> Result<()> {
    let p = root
        .find_mut(parent)
        .ok_or_else(|| invalid("UI parent missing or cyclic move"))?;
    if !p.kind.container() || index > p.children.len() {
        return Err(invalid("Invalid UI insertion target/index"));
    }
    p.children.insert(index, node);
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Inspect,
    Replace { document: Box<Document> },
    Edit { edits: Vec<Edit> },
    Undo,
    Redo,
    Save { path: String },
    Load { path: String },
}
#[derive(Default)]
pub struct State {
    document: Arc<Document>,
    revision: u64,
    undo: VecDeque<Arc<Document>>,
    redo: Vec<Arc<Document>>,
}
impl State {
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn snapshot(&self) -> Arc<Document> {
        self.document.clone()
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"ui_revision":self.revision,"document":self.document,"undo":self.undo.len(),"redo":self.redo.len()})
    }
    pub fn guard(&self, revision: Option<u64>) -> Result<()> {
        if revision.is_some_and(|r| r != self.revision) {
            return Err(invalid("Stale UI revision"));
        }
        Ok(())
    }
    pub fn replace(&mut self, d: Document) -> Result<()> {
        d.validate()?;
        if d == *self.document {
            return Ok(());
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("UI revision exhausted"))?;
        if self.undo.len() == MAX_HISTORY {
            self.undo.pop_front();
        }
        self.undo
            .push_back(std::mem::replace(&mut self.document, Arc::new(d)));
        self.redo.clear();
        self.revision = revision;
        Ok(())
    }
    pub fn edit(&mut self, edits: &[Edit]) -> Result<()> {
        if edits.is_empty() || edits.len() > 64 {
            return Err(invalid("UI transaction requires1..64 edits"));
        }
        let mut d = (*self.document).clone();
        for e in edits {
            e.apply(&mut d)?;
        }
        self.replace(d)
    }
    pub fn history(&mut self, redo: bool) -> Result<()> {
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("UI revision exhausted"))?;
        if redo {
            let next = self
                .redo
                .pop()
                .ok_or_else(|| invalid("Nothing to redo in UI"))?;
            self.undo
                .push_back(std::mem::replace(&mut self.document, next));
        } else {
            let prev = self
                .undo
                .pop_back()
                .ok_or_else(|| invalid("Nothing to undo in UI"))?;
            self.redo.push(std::mem::replace(&mut self.document, prev));
        }
        self.revision = revision;
        Ok(())
    }
}
