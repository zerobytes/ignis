//! Typed botanical sources, stable material addresses and native authoring.
use crate::{
    meshes,
    model::{Result, invalid},
};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub mod bake;
pub mod deformation;
pub mod development;
pub mod dynamics;
pub mod growth;
pub mod lifecycle;
pub mod production;
mod reproduction;
pub mod root_crown;
pub mod root_sections;
pub mod roots;
pub mod wood_surface;
pub const VERSION: u16 = 10;
pub const LIVING_SCENE_VERSION: u32 = 121;
fn one() -> f64 {
    1.
}
fn is_zero(v: &f64) -> bool {
    *v == 0.
}
fn is_one(v: &f64) -> bool {
    *v == 1.
}
pub const MAX_SOURCE_BYTES: usize = 96 * 1024 * 1024;
pub const MAX_ORGANS: usize = 65536;
pub const MAX_VERTICES: usize = 3_000_000;
pub const MAX_TRIANGLES: usize = 6_000_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub version: u16,
    pub name: String,
    pub seed: u64,
    pub organs: Vec<Organ>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub development: Option<development::Profile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_systems: Vec<roots::System>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Connection {
    #[default]
    Graft,
    Sheath,
    /// One of two continuations at the end of the parent axis.
    Fork,
    Basal,
}
impl Connection {
    fn is_graft(&self) -> bool {
        *self == Self::Graft
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Organ {
    pub id: String,
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub life: Option<development::OrganLife>,
    pub insertion: f64,
    /// Canonical control polygon in plant coordinates; never triangulated source.
    pub path: Vec<[f64; 3]>,
    pub normal: [f64; 3],
    #[serde(default)]
    pub twist: f64,
    pub shape: Shape,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub taper_power: f64,
    #[serde(default, skip_serializing_if = "Connection::is_graft")]
    pub connection: Connection,
    #[serde(default)]
    pub birth: f64,
    pub duration: f64,
    pub density: f64,
    pub young: f64,
    pub shear: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Shape {
    Axis {
        radius: f64,
        tip_ratio: f64,
        ellipticity: f64,
    },
    Blade {
        width: f64,
        thickness: f64,
        fold: f64,
        cup: f64,
        outline: Outline,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outline {
    Strip {
        base_power: f64,
        tip_power: f64,
    },
    Palmate {
        tips: u8,
        spread: f64,
        depth: f64,
        rounding: f64,
        valley_rounding: f64,
        lengths: Vec<f64>,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age_years: Option<f64>,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub vigor: f64,
    pub scale: f64,
    pub crown_width: f64,
    pub radius: f64,
    pub leaf_length: f64,
    pub leaf_width: f64,
    pub leaf_density: f64,
    /// Strength of authored growth-stage differences on complete fronds.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub frond_growth_variation: f64,
    pub axis_samples: u16,
    /// Longitudinal tessellation of strip blades; independent of axis/physics sampling.
    pub blade_samples: u8,
    pub sides: u8,
    pub bark_color: [f32; 3],
    pub leaf_color: [f32; 3],
    pub position: [f32; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bark_pattern: Option<crate::materials::Pattern>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaf_pattern: Option<crate::materials::Pattern>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            age_years: None,
            vigor: 1.,
            scale: 1.,
            crown_width: 1.,
            radius: 1.,
            leaf_length: 1.,
            leaf_width: 1.,
            leaf_density: 1.,
            frond_growth_variation: 0.,
            axis_samples: 12,
            blade_samples: 12,
            sides: 8,
            bark_color: [0.34, 0.23, 0.12],
            leaf_color: [0.16, 0.38, 0.10],
            position: [0.; 3],
            bark_pattern: None,
            leaf_pattern: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct OrganEdit {
    pub offset: [f64; 3],
    pub rotation: [f64; 3],
    pub length: f64,
    pub width: f64,
    pub radius: f64,
    #[serde(default, skip_serializing_if = "deformation::Deformation::is_default")]
    pub deformation: deformation::Deformation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wood_surface: Option<wood_surface::Surface>,
}
impl Default for OrganEdit {
    fn default() -> Self {
        Self {
            offset: [0.; 3],
            rotation: [0.; 3],
            length: 1.,
            width: 1.,
            radius: 1.,
            deformation: Default::default(),
            wood_surface: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub living: Option<growth::Plant>,
    pub version: u16,
    pub prefix: String,
    pub source: Arc<Source>,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub edits: BTreeMap<String, OrganEdit>,
    #[serde(default)]
    pub pruned: BTreeSet<String>,
    /// Frame-local metre offsets, keyed by stable material address.
    #[serde(default)]
    pub manual: BTreeMap<String, [f64; 3]>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManualPolicy {
    #[default]
    Preserve,
    Reject,
    Discard,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    CreateLiving {
        definition: growth::Definition,
        born: u64,
    },
    Living {
        prefix: String,
        action: growth::Action,
    },
    PreviewLivingAge {
        prefix: String,
        time: u64,
    },
    Inspect,
    Open {
        open: bool,
        prefix: Option<String>,
    },
    Preview {
        recipe: Box<Recipe>,
        #[serde(default)]
        policy: ManualPolicy,
    },
    CancelPreview,
    Publish {
        recipe: Box<Recipe>,
        #[serde(default)]
        policy: ManualPolicy,
    },
    SelectOrgan {
        prefix: String,
        organ: Option<String>,
    },
    SetRoots {
        prefix: String,
        stem: String,
        settings: Option<roots::Settings>,
        #[serde(default)]
        preview: bool,
        #[serde(default)]
        policy: ManualPolicy,
    },
    SetAge {
        prefix: String,
        age_years: f64,
        #[serde(default)]
        vigor: Option<f64>,
        #[serde(default)]
        preview: bool,
        #[serde(default)]
        policy: ManualPolicy,
    },
    EditOrgan {
        prefix: String,
        organ: String,
        edit: OrganEdit,
    },
    Save {
        prefix: String,
        path: String,
    },
    Load {
        path: String,
        #[serde(default)]
        publish: bool,
    },
    Physics {
        prefix: String,
        action: dynamics::Action,
    },
}

pub fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn finite(v: &[f64]) -> bool {
    v.iter().all(|x| x.is_finite() && x.abs() <= 10000.)
}
impl Source {
    pub fn validate(&self) -> Result<()> {
        if !(1..=VERSION).contains(&self.version)
            || self.name.len() > 128
            || self.organs.is_empty()
            || self.organs.len() > MAX_ORGANS
        {
            return Err(invalid(
                "Invalid botanical source version/name/organ budget",
            ));
        }
        if self.development.is_some() && self.version < 3 {
            return Err(invalid("Development profiles require botanical source3"));
        }
        if let Some(p) = &self.development {
            p.validate()?;
        }
        let mut seen = BTreeSet::new();
        let mut forks: BTreeMap<&str, usize> = BTreeMap::new();
        for o in &self.organs {
            if o.life.is_some() && (self.development.is_none() || self.version < 3) {
                return Err(invalid(
                    "Organ chronology requires a source3 development profile",
                ));
            }
            if let Some(life) = &o.life {
                life.validate()?;
                if life.growth_stage != 1. && self.version < 9 {
                    return Err(invalid("Frond growth stages require botanical source9"));
                }
                if life.role == development::Role::Root
                    && (self.version < 5 || roots::system_for(self, &o.id).is_none())
                {
                    return Err(invalid("Root chronology requires a source5 root system"));
                }
            }
            if o.connection == Connection::Fork {
                let parent = o
                    .parent
                    .as_deref()
                    .ok_or_else(|| invalid("A fork requires a parent"))?;
                if self.version < 4
                    || o.insertion != 1.
                    || !matches!(o.shape, Shape::Axis { .. })
                    || !self
                        .organs
                        .iter()
                        .any(|p| p.id == parent && matches!(p.shape, Shape::Axis { .. }))
                {
                    return Err(invalid(
                        "Terminal forks require source4 woody axes at insertion1",
                    ));
                }
                *forks.entry(parent).or_default() += 1;
            }
            if o.id.is_empty()
                || o.id.len() > 512
                || !o.id.is_ascii()
                || !seen.insert(o.id.as_str())
                || o.parent
                    .as_deref()
                    .is_some_and(|p| !seen.contains(p) || p == o.id)
                || !(0. ..=1.).contains(&o.insertion)
                || o.path.len() < 2
                || o.path.len() > 33
                || o.path.iter().any(|p| !finite(p))
                || !finite(&o.normal)
                || DVec3::from_array(o.normal).length_squared() < 1e-12
                || !finite(&[o.birth, o.duration, o.density, o.twist])
                || o.birth < 0.
                || o.duration <= 0.
                || o.density <= 0.
                || o.young <= 0.
                || o.shear <= 0.
            {
                // Moduli have a separate finite range below, not coordinate bounds.
                if !(o.young > 0. && o.young.is_finite() && o.shear > 0. && o.shear.is_finite()) {
                    return Err(invalid("Invalid botanical material"));
                }
                return Err(invalid(format!(
                    "Invalid botanical organ or parent order: {}",
                    o.id
                )));
            }
            if !o.young.is_finite() || !o.shear.is_finite() || o.young > 1e14 || o.shear > 1e14 {
                return Err(invalid("Botanical moduli must be finite and at most1e14Pa"));
            }
            if !o.taper_power.is_finite() || !(0.1..=8.).contains(&o.taper_power) {
                return Err(invalid("Invalid botanical taper exponent"));
            }
            if o.path
                .windows(2)
                .any(|p| DVec3::from_array(p[1]).distance_squared(DVec3::from_array(p[0])) < 1e-16)
            {
                return Err(invalid("Botanical path has coincident controls"));
            }
            match &o.shape {
                Shape::Axis {
                    radius,
                    tip_ratio,
                    ellipticity,
                } if radius.is_finite()
                    && *radius > 0.
                    && *radius <= 10.
                    && tip_ratio.is_finite()
                    && *tip_ratio > 0.
                    && *tip_ratio <= 1.
                    && ellipticity.is_finite()
                    && *ellipticity > 0.
                    && *ellipticity <= 10. => {}
                Shape::Blade {
                    width,
                    thickness,
                    fold,
                    cup,
                    outline,
                } if finite(&[*width, *thickness, *fold, *cup])
                    && *width > 0.
                    && *thickness > 0. =>
                {
                    outline.validate()?
                }
                _ => return Err(invalid("Invalid botanical section/lamina")),
            }
        }
        if forks.values().any(|n| *n != 2) {
            return Err(invalid(
                "A terminal fork requires exactly two authored continuations",
            ));
        }
        if !self.root_systems.is_empty() && self.version < 5 {
            return Err(invalid("Root systems require source5"));
        }
        if self.root_systems.len() > 16 {
            return Err(invalid("At most16 root crowns"));
        }
        let mut stems = BTreeSet::new();
        for system in &self.root_systems {
            system.settings.validate()?;
            if system.settings.buttress.is_some() && self.version < 6 {
                return Err(invalid("Buttress sections require source6"));
            }
            if !stems.insert(&system.stem)
                || !self.organs.iter().any(|o| {
                    o.id == system.stem
                        && o.parent.is_none()
                        && matches!(o.shape, Shape::Axis { .. })
                })
            {
                return Err(invalid("Invalid or duplicate root stem"));
            }
            let n = self
                .organs
                .iter()
                .filter(|o| {
                    o.parent.as_deref() == Some(&system.stem) && o.connection == Connection::Basal
                })
                .count();
            if n != system.settings.count as usize {
                return Err(invalid("Root primary count differs from root system"));
            }
        }
        if !self.root_systems.is_empty() {
            let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
            for o in &self.organs {
                let root = if o.connection == Connection::Basal {
                    o.parent.as_deref()
                } else {
                    o.parent.as_deref().and_then(|p| owner.get(p).copied())
                };
                if let Some(stem) = root {
                    if !self
                        .root_systems
                        .iter()
                        .any(|r| r.stem == stem && r.contains(&o.id))
                        || !matches!(o.shape, Shape::Axis { .. })
                    {
                        return Err(invalid("Root namespace or axis ownership mismatch"));
                    }
                    owner.insert(&o.id, stem);
                } else if roots::system_for(self, &o.id).is_some() {
                    return Err(invalid("Reserved root ID has no basal root ancestor"));
                }
            }
        }
        for o in &self.organs {
            if o.connection == Connection::Basal
                && (self.version < 5
                    || o.insertion != 0.
                    || !matches!(o.shape, Shape::Axis { .. })
                    || !self
                        .root_systems
                        .iter()
                        .any(|s| o.parent.as_deref() == Some(&s.stem) && s.contains(&o.id)))
            {
                return Err(invalid(
                    "Basal roots require source5 root crown at insertion0",
                ));
            }
        }
        Ok(())
    }
}
impl Outline {
    fn validate(&self) -> Result<()> {
        let valid = match self {
            Self::Strip {
                base_power,
                tip_power,
            } => finite(&[*base_power, *tip_power]) && *base_power > 0. && *tip_power > 0.,
            Self::Palmate {
                tips,
                spread,
                depth,
                rounding,
                valley_rounding,
                lengths,
            } => {
                (3..=32).contains(tips)
                    && finite(&[*spread, *depth, *rounding, *valley_rounding])
                    && *spread > 0.
                    && *spread < std::f64::consts::PI
                    && (0. ..0.995).contains(depth)
                    && *rounding >= 0.2
                    && *rounding <= 4.
                    && *valley_rounding > 1.
                    && *valley_rounding <= 4.
                    && lengths.len() == *tips as usize
                    && lengths
                        .iter()
                        .all(|x| x.is_finite() && *x >= 0.05 && *x <= 2.)
            }
        };
        if valid {
            Ok(())
        } else {
            Err(invalid("Invalid botanical contour"))
        }
    }
}
impl Recipe {
    pub fn validate(&self) -> Result<()> {
        if let Some(plant) = &self.living {
            if self.version < 8
                || self.source.version < 8
                || plant.definition.prefix != self.prefix
                || !self.pruned.is_empty()
            {
                return Err(invalid(
                    "Living plants require matching recipe/source8 and semantic pruning",
                ));
            }
            plant.validate()?;
            if plant.recipe()?.source != self.source {
                return Err(invalid("Living source does not match its canonical state"));
            }
        }
        self.source.validate()?;
        if self.version < 4
            && self
                .source
                .organs
                .iter()
                .any(|o| o.connection == Connection::Fork)
        {
            return Err(invalid("Terminal forks require botanical recipe4"));
        }
        if self.version < 4 && self.settings.blade_samples != 12 {
            return Err(invalid("Blade resolution requires botanical recipe4"));
        }
        if self.version < 5 && !self.source.root_systems.is_empty() {
            return Err(invalid("Root systems require recipe5"));
        }
        if self.version < 6
            && self
                .source
                .root_systems
                .iter()
                .any(|r| r.settings.buttress.is_some())
        {
            return Err(invalid("Buttress sections require recipe6"));
        }
        if self.version < 7
            && self.source.version >= 7
            && self
                .source
                .root_systems
                .iter()
                .any(|r| r.settings.buttress.is_some())
        {
            return Err(invalid("Valley crowns require recipe7"));
        }
        let s = &self.settings;
        if self.source.version >= 9 && self.version < 9 {
            return Err(invalid("Frond growth source9 requires recipe9"));
        }
        if !s.frond_growth_variation.is_finite()
            || !(0. ..=1.).contains(&s.frond_growth_variation)
            || (s.frond_growth_variation != 0. && self.version < 9)
        {
            return Err(invalid("Frond growth variation must be0..1 and requires recipe9"));
        }
        if self.source.development.is_some() {
            if self.version < 3 {
                return Err(invalid("Development profiles require botanical recipe3"));
            }
            development::state(self)?;
        } else if s.age_years.is_some() {
            return Err(invalid(
                "Age in years requires a development profile in the source",
            ));
        }
        if !s.vigor.is_finite() || !(0.25..=2.).contains(&s.vigor) {
            return Err(invalid("Invalid botanical vigor"));
        }
        for pattern in [s.bark_pattern, s.leaf_pattern].into_iter().flatten() {
            pattern.validate()?;
        }
        if !(1..=VERSION).contains(&self.version)
            || !identifier(&self.prefix)
            || !finite(&[
                s.scale,
                s.crown_width,
                s.radius,
                s.leaf_length,
                s.leaf_width,
                s.leaf_density,
            ])
            || [
                s.scale,
                s.crown_width,
                s.radius,
                s.leaf_length,
                s.leaf_width,
            ]
            .iter()
            .any(|v| *v < 0.1 || *v > 5.)
            || !(0. ..=1.).contains(&s.leaf_density)
            || !(4..=64).contains(&s.axis_samples)
            || !(4..=32).contains(&s.blade_samples)
            || !(6..=16).contains(&s.sides)
            || s.position
                .iter()
                .any(|x| !x.is_finite() || x.abs() > 10000.)
            || s.bark_color
                .iter()
                .chain(&s.leaf_color)
                .any(|x| !x.is_finite() || !(0. ..=1.).contains(x))
            || self.edits.len() > MAX_ORGANS
            || self.pruned.len() > MAX_ORGANS
            || self.manual.len() > MAX_VERTICES
        {
            return Err(invalid("Invalid botanical recipe/settings/budget"));
        }
        let ids: BTreeSet<_> = self.source.organs.iter().map(|o| o.id.as_str()).collect();
        if self.pruned.iter().any(|v| !ids.contains(v.as_str())) {
            return Err(invalid("Unknown pruned botanical organ"));
        }
        for (id, e) in &self.edits {
            e.deformation.validate()?;
            if let Some(surface) = &e.wood_surface {
                if self.version < 10
                    || self.living.is_some()
                    || !self.source.organs.iter().any(|o| {
                        o.id == *id && matches!(o.shape, Shape::Axis { .. })
                    })
                {
                    return Err(invalid(
                        "Wood surface requires recipe10 and a nonliving woody axis",
                    ));
                }
                surface.validate()?;
            }
            if !ids.contains(id.as_str())
                || !finite(&e.offset)
                || !finite(&e.rotation)
                || [e.length, e.width, e.radius]
                    .iter()
                    .any(|x| !x.is_finite() || *x < 0.1 || *x > 5.)
            {
                return Err(invalid("Invalid botanical organ adjustment"));
            }
        }
        if self.manual.iter().any(|(k, d)| k.len() > 800 || !finite(d)) {
            return Err(invalid("Invalid botanical material offsets"));
        }
        Ok(())
    }
    pub fn all_parts(&self) -> Vec<String> {
        let mut ids = self.parts().to_vec();
        if self.living.is_some() {
            ids.extend(
                ["flowers", "green_fruit", "ripe_fruit"].map(|p| format!("{}_{}", self.prefix, p)),
            );
        }
        ids
    }
    pub fn parts(&self) -> [String; 2] {
        [
            format!("{}_wood", self.prefix),
            format!("{}_leaves", self.prefix),
        ]
    }
}

pub fn validate_scene(scene: &crate::model::Scene) -> Result<()> {
    if scene.botany.is_empty() {
        return Ok(());
    }
    if scene.version < 108 || scene.botany.len() > 64 {
        return Err(invalid(
            "Botanical sources require scene108, at most64 recipes",
        ));
    }
    let mut owned = BTreeSet::new();
    for recipe in &scene.botany {
        recipe.validate()?;
        if recipe.living.is_some() && scene.version < LIVING_SCENE_VERSION {
            return Err(invalid("Living plants require scene121"));
        }
        for id in recipe.all_parts() {
            if !owned.insert(id.clone())
                || !scene.meshes.assets.iter().any(|a| a.id == id)
                || !scene
                    .meshes
                    .instances
                    .iter()
                    .any(|i| i.id == id && i.asset == id)
            {
                return Err(invalid("Botanical bake parts missing or overlapping"));
            }
            if scene.characters.iter().any(|b| b.parts.contains(&id)) {
                return Err(invalid("Botanical and character ownership overlap"));
            }
        }
    }
    Ok(())
}

#[derive(Default)]
pub struct State {
    pub open: bool,
    pub selected: Option<String>,
    pub organ: Option<String>,
    pub revision: Option<u64>,
    pub preview: Option<Recipe>,
    pub overlay: Option<meshes::Document>,
    pub basis: Option<(Option<Recipe>, meshes::Document)>,
    pub physics: Option<dynamics::Session>,
}
