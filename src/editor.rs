//! Bounded editor session document. Never part of scene, simulation or replay.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Panel {
    Project,
    Viewport,
    Hierarchy,
    Properties,
    Templates,
    Events,
}
impl Panel {
    pub const ALL: [Self; 6] = [
        Self::Project,
        Self::Viewport,
        Self::Hierarchy,
        Self::Properties,
        Self::Templates,
        Self::Events,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Project => "Projeto",
            Self::Viewport => "Cena",
            Self::Hierarchy => "Hierarquia",
            Self::Properties => "Propriedades",
            Self::Templates => "Templates",
            Self::Events => "Eventos",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Area {
    Tabs {
        id: u32,
        tabs: Vec<Panel>,
        active: Panel,
    },
    Split {
        id: u32,
        axis: Axis,
        ratio: f32,
        first: Box<Area>,
        second: Box<Area>,
    },
}
impl Area {
    pub fn id(&self) -> u32 {
        match self {
            Self::Tabs { id, .. } | Self::Split { id, .. } => *id,
        }
    }
    pub fn find_mut(&mut self, target: u32) -> Option<&mut Self> {
        if self.id() == target {
            return Some(self);
        }
        match self {
            Self::Split { first, second, .. } => {
                first.find_mut(target).or_else(|| second.find_mut(target))
            }
            _ => None,
        }
    }
    pub fn find(&self, target: u32) -> Option<&Self> {
        if self.id() == target {
            return Some(self);
        }
        match self {
            Self::Split { first, second, .. } => first.find(target).or_else(|| second.find(target)),
            _ => None,
        }
    }
    pub fn leaves(&self, out: &mut Vec<(u32, Panel)>) {
        match self {
            Self::Tabs { id, active, .. } => out.push((*id, *active)),
            Self::Split { first, second, .. } => {
                first.leaves(out);
                second.leaves(out);
            }
        }
    }
    pub fn panel_area(&self, panel: Panel) -> Option<u32> {
        match self {
            Self::Tabs { id, tabs, .. } => tabs.contains(&panel).then_some(*id),
            Self::Split { first, second, .. } => {
                first.panel_area(panel).or_else(|| second.panel_area(panel))
            }
        }
    }
    fn max_id(&self) -> u32 {
        match self {
            Self::Tabs { id, .. } => *id,
            Self::Split {
                id, first, second, ..
            } => *id.max(&first.max_id()).max(&second.max_id()),
        }
    }
    fn prune(&mut self) {
        if let Self::Split { first, second, .. } = self {
            first.prune();
            second.prune();
            let empty = |a: &Area| matches!(a,Area::Tabs{tabs,..} if tabs.is_empty());
            if empty(first) {
                *self = *second.clone();
            } else if empty(second) {
                *self = *first.clone();
            }
        }
    }
    fn validate(
        &self,
        depth: usize,
        ids: &mut BTreeSet<u32>,
        counts: &mut [usize; 3],
    ) -> Result<()> {
        if depth > 8 || !ids.insert(self.id()) {
            return Err(invalid("Editor layout depth exceeds8 or area IDs repeat"));
        }
        match self {
            Self::Tabs { tabs, active, .. } => {
                counts[0] += 1;
                counts[1] += tabs.len();
                counts[2] += usize::from(tabs.contains(&Panel::Viewport));
                if tabs.is_empty()
                    || tabs.len() > 8
                    || !tabs.contains(active)
                    || tabs.iter().enumerate().any(|(i, p)| tabs[..i].contains(p))
                {
                    return Err(invalid(
                        "Area needs1..8 unique tabs and an existing active tab",
                    ));
                }
            }
            Self::Split {
                ratio,
                first,
                second,
                ..
            } => {
                if !ratio.is_finite() || !(0.1..=0.9).contains(ratio) {
                    return Err(invalid("Split ratio must be finite in [0.1,0.9]"));
                }
                first.validate(depth + 1, ids, counts)?;
                second.validate(depth + 1, ids, counts)?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub layout: Area,
    pub maximized: Option<u32>,
    #[serde(default)]
    pub camera: Option<crate::model::Camera>,
}

pub fn studio_light() -> Option<crate::model::Light> {
    Some(crate::model::Light {
        direction: [0.7, -1., -0.4],
        ambient: 0.08,
        direct_intensity: 1.,
        environment: None,
        atmosphere: None,
        shadow: Some(crate::model::DirectionalShadow {
            center: [0., 0., 0.],
            extent: 16.,
            resolution: 1024,
            bias: 0.002,
        }),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SculptSettings {
    pub brush: crate::meshes::BrushKind,
    pub radius: f32,
    pub draw_strength: f32,
    /// Old layouts used metres per sample; native controls migrate once.
    #[serde(default)]
    pub draw_strength_is_relative: bool,
    #[serde(default = "default_adaptive_detail")]
    pub adaptive_detail: bool,
    #[serde(default)]
    pub detail: crate::mesh_tools::SculptDetail,
    pub grab_strength: f32,
    pub smooth_strength: f32,
    #[serde(default = "default_shape_strength")]
    pub flatten_strength: f32,
    #[serde(default = "default_shape_strength")]
    pub pinch_strength: f32,
    #[serde(default = "default_shape_strength")]
    pub crease_strength: f32,
    #[serde(default = "default_shape_strength")]
    pub scrape_strength: f32,
    #[serde(default = "default_shape_strength")]
    pub fill_strength: f32,
    #[serde(default = "default_shape_strength")]
    pub relax_strength: f32,
}
fn default_adaptive_detail() -> bool {
    true
}
fn default_shape_strength() -> f32 {
    0.25
}
impl Default for SculptSettings {
    fn default() -> Self {
        Self {
            brush: crate::meshes::BrushKind::Draw,
            radius: 0.4,
            draw_strength: 0.6,
            draw_strength_is_relative: true,
            adaptive_detail: true,
            detail: Default::default(),
            grab_strength: 0.7,
            smooth_strength: 0.5,
            flatten_strength: default_shape_strength(),
            pinch_strength: default_shape_strength(),
            crease_strength: default_shape_strength(),
            scrape_strength: default_shape_strength(),
            fill_strength: default_shape_strength(),
            relax_strength: default_shape_strength(),
        }
    }
}
impl SculptSettings {
    pub(crate) fn relative(&self) -> Self {
        let mut settings = self.clone();
        if !settings.draw_strength_is_relative {
            // Preserve the height of a single legacy dab, not unbounded overlap.
            settings.draw_strength =
                (2. * settings.draw_strength / settings.radius).clamp(0.001, 1.);
            settings.draw_strength_is_relative = true;
        }
        settings
    }
    pub fn strength_mut(&mut self, kind: crate::meshes::BrushKind) -> &mut f32 {
        use crate::meshes::BrushKind::*;
        match kind {
            Draw => &mut self.draw_strength,
            Grab => &mut self.grab_strength,
            Smooth => &mut self.smooth_strength,
            Flatten => &mut self.flatten_strength,
            Pinch => &mut self.pinch_strength,
            Crease => &mut self.crease_strength,
            Scrape => &mut self.scrape_strength,
            Fill => &mut self.fill_strength,
            Relax => &mut self.relax_strength,
        }
    }
    pub fn strength(&self, kind: crate::meshes::BrushKind) -> f32 {
        use crate::meshes::BrushKind::*;
        match kind {
            Draw => self.draw_strength,
            Grab => self.grab_strength,
            Smooth => self.smooth_strength,
            Flatten => self.flatten_strength,
            Pinch => self.pinch_strength,
            Crease => self.crease_strength,
            Scrape => self.scrape_strength,
            Fill => self.fill_strength,
            Relax => self.relax_strength,
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.detail.validate()?;
        if !self.radius.is_finite()
            || !(0.001..=100.).contains(&self.radius)
            || [
                self.draw_strength,
                self.grab_strength,
                self.smooth_strength,
                self.flatten_strength,
                self.pinch_strength,
                self.crease_strength,
                self.scrape_strength,
                self.fill_strength,
                self.relax_strength,
            ]
            .iter()
            .any(|x| !x.is_finite() || !(0.001..=1.).contains(x))
        {
            return Err(invalid(
                "Sculpt radius requires0.001..100m; brush strengths0.001..1",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    pub revision: u64,
    pub active: String,
    pub workspaces: Vec<Workspace>,
    #[serde(default = "studio_light")]
    pub work_light: Option<crate::model::Light>,
    #[serde(default)]
    pub sculpt: SculptSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Lighting {
        light: Option<crate::model::Light>,
    },
    Sculpt {
        settings: SculptSettings,
    },
    Camera {
        camera: Option<crate::model::Camera>,
    },
    Switch {
        workspace: String,
    },
    Duplicate {
        id: String,
        name: String,
    },
    Rename {
        name: String,
    },
    RemoveWorkspace,
    Restore,
    Activate {
        area: u32,
        panel: Panel,
    },
    Open {
        area: u32,
        panel: Panel,
    },
    Close {
        area: u32,
        panel: Panel,
    },
    Split {
        area: u32,
        axis: Axis,
        panel: Panel,
    },
    Resize {
        area: u32,
        ratio: f32,
    },
    Move {
        from: u32,
        to: u32,
        panel: Panel,
    },
    Maximize {
        area: Option<u32>,
    },
}
fn tabs(id: u32, panels: &[Panel]) -> Area {
    Area::Tabs {
        id,
        tabs: panels.to_vec(),
        active: panels[0],
    }
}
fn split(id: u32, axis: Axis, ratio: f32, first: Area, second: Area) -> Area {
    Area::Split {
        id,
        axis,
        ratio,
        first: Box::new(first),
        second: Box::new(second),
    }
}
impl Default for State {
    fn default() -> Self {
        use Panel::*;
        Self {
            version: 3,
            work_light: studio_light(),
            sculpt: SculptSettings::default(),
            revision: 0,
            active: "scene".into(),
            workspaces: vec![
                Workspace {
                    id: "scene".into(),
                    name: "Cena".into(),
                    maximized: None,
                    camera: None,
                    layout: split(
                        0,
                        Axis::Horizontal,
                        0.76,
                        tabs(1, &[Viewport]),
                        split(
                            2,
                            Axis::Vertical,
                            0.36,
                            tabs(3, &[Hierarchy, Project, Templates]),
                            tabs(4, &[Properties, Events]),
                        ),
                    ),
                },
                Workspace {
                    id: "inspect".into(),
                    name: "Inspeção".into(),
                    maximized: None,
                    camera: None,
                    layout: split(
                        0,
                        Axis::Horizontal,
                        0.68,
                        split(
                            1,
                            Axis::Vertical,
                            0.75,
                            tabs(2, &[Viewport]),
                            tabs(3, &[Events, Templates]),
                        ),
                        split(
                            4,
                            Axis::Vertical,
                            0.35,
                            tabs(5, &[Hierarchy]),
                            tabs(6, &[Properties]),
                        ),
                    ),
                },
            ],
        }
    }
}
fn name_valid(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 48 && !s.chars().any(char::is_control)
}
impl State {
    pub fn current(&self) -> &Workspace {
        self.workspaces
            .iter()
            .find(|w| w.id == self.active)
            .expect("validated workspace")
    }
    fn current_mut(&mut self) -> &mut Workspace {
        self.workspaces
            .iter_mut()
            .find(|w| w.id == self.active)
            .expect("validated workspace")
    }
    pub fn validate(&self) -> Result<()> {
        if !(1..=3).contains(&self.version)
            || self.workspaces.is_empty()
            || self.workspaces.len() > 8
        {
            return Err(invalid("Editor versions1..3 require1..8 workspaces"));
        }
        if self.version == 1
            && self
                .workspaces
                .iter()
                .any(|w| w.layout.panel_area(Panel::Project).is_some())
        {
            return Err(invalid("Project panel requires editor layout2"));
        }
        self.sculpt.validate()?;
        if let Some(light) = &self.work_light {
            light.validate()?;
        }
        let mut names = BTreeSet::new();
        for w in &self.workspaces {
            if let Some(camera) = &w.camera {
                camera.validate()?;
            }
            if !name_valid(&w.id) || !name_valid(&w.name) || !names.insert(&w.id) {
                return Err(invalid(
                    "Workspace names/IDs must be unique IDs and1..48 printable bytes",
                ));
            }
            let mut ids = BTreeSet::new();
            let mut counts = [0; 3];
            w.layout.validate(0, &mut ids, &mut counts)?;
            if counts[0] > 16 || counts[1] > 32 || counts[2] != 1 {
                return Err(invalid(
                    "Layout permits16 areas/32 tabs and requires exactly one viewport",
                ));
            }
            if w.maximized
                .is_some_and(|id| !matches!(w.layout.find(id), Some(Area::Tabs { .. })))
            {
                return Err(invalid("Maximized area must be an existing leaf"));
            }
        }
        if !names.contains(&self.active) {
            return Err(invalid("Active workspace does not exist"));
        }
        Ok(())
    }
    pub fn apply(&mut self, action: Action) -> Result<()> {
        let mut next = self.clone();
        next.version = 3;
        next.change(action)?;
        next.validate()?;
        next.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Editor revision exhausted"))?;
        *self = next;
        Ok(())
    }
    fn leaf_mut(&mut self, id: u32) -> Result<(&mut Vec<Panel>, &mut Panel)> {
        match self.current_mut().layout.find_mut(id) {
            Some(Area::Tabs { tabs, active, .. }) => Ok((tabs, active)),
            _ => Err(invalid("Editor area does not exist or is a split")),
        }
    }
    fn remove_tab(&mut self, id: u32, panel: Panel) -> Result<()> {
        let (tabs, active) = self.leaf_mut(id)?;
        let at = tabs
            .iter()
            .position(|p| *p == panel)
            .ok_or_else(|| invalid("Tab not present"))?;
        tabs.remove(at);
        if *active == panel && !tabs.is_empty() {
            *active = tabs[0];
        }
        self.current_mut().layout.prune();
        Ok(())
    }
    fn change(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Lighting { light } => self.work_light = light,
            Action::Sculpt { settings } => self.sculpt = settings,
            Action::Camera { camera } => self.current_mut().camera = camera,
            Action::Switch { workspace } => self.active = workspace,
            Action::Duplicate { id, name } => {
                let mut w = self.current().clone();
                w.id = id.clone();
                w.name = name;
                self.workspaces.push(w);
                self.active = id;
            }
            Action::Rename { name } => self.current_mut().name = name,
            Action::RemoveWorkspace => {
                let at = self
                    .workspaces
                    .iter()
                    .position(|w| w.id == self.active)
                    .unwrap();
                self.workspaces.remove(at);
                self.active = self
                    .workspaces
                    .first()
                    .map(|w| w.id.clone())
                    .unwrap_or_default();
            }
            Action::Restore => {
                let current = self.current_mut();
                let preset = State::default();
                current.layout = preset
                    .workspaces
                    .iter()
                    .find(|w| w.id == current.id)
                    .unwrap_or(&preset.workspaces[0])
                    .layout
                    .clone();
                current.maximized = None;
                current.camera = None;
            }
            Action::Activate { area, panel } => {
                let (tabs, active) = self.leaf_mut(area)?;
                if !tabs.contains(&panel) {
                    return Err(invalid("Tab not present"));
                }
                *active = panel;
            }
            Action::Open { area, panel } => {
                let (tabs, active) = self.leaf_mut(area)?;
                if !tabs.contains(&panel) {
                    tabs.push(panel);
                }
                *active = panel;
            }
            Action::Close { area, panel } => {
                self.remove_tab(area, panel)?;
                self.current_mut().maximized = None;
            }
            Action::Move { from, to, panel } => {
                if from == to {
                    let (tabs, _) = self.leaf_mut(from)?;
                    if !tabs.contains(&panel) {
                        return Err(invalid("Tab not present"));
                    }
                    return Ok(());
                }
                self.remove_tab(from, panel)?;
                let (tabs, active) = self.leaf_mut(to)?;
                if !tabs.contains(&panel) {
                    tabs.push(panel);
                }
                *active = panel;
                self.current_mut().maximized = None;
            }
            Action::Resize { area, ratio } => match self.current_mut().layout.find_mut(area) {
                Some(Area::Split { ratio: r, .. }) => *r = ratio,
                _ => return Err(invalid("Resize requires a split")),
            },
            Action::Split { area, axis, panel } => {
                let id = self
                    .current()
                    .layout
                    .max_id()
                    .checked_add(1)
                    .filter(|n| *n < u32::MAX)
                    .ok_or_else(|| invalid("Area identity exhausted"))?;
                let original = self
                    .current_mut()
                    .layout
                    .find_mut(area)
                    .ok_or_else(|| invalid("Area not found"))?;
                if !matches!(original, Area::Tabs { .. }) {
                    return Err(invalid("Split requires a leaf"));
                }
                *original = split(id, axis, 0.6, original.clone(), tabs(id + 1, &[panel]));
                self.current_mut().maximized = None;
            }
            Action::Maximize { area } => self.current_mut().maximized = area,
        }
        Ok(())
    }
}

/// Session-only viewport overlays; authored render settings and saved layouts stay unchanged.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct MeshView {
    pub edges: bool,
    pub vertices: bool,
    pub bones: bool,
    pub xray: bool,
}
