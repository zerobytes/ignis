//! Solenoidal external airflow and passive rigid thin-surface aerodynamics.
use crate::model::{Result, Scene, invalid};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};
mod aero;
mod field;
pub use aero::{Mass, Panel, SurfacePoint, compile_surfaces, implicit_drag};
pub use field::{CompiledField, Field, FieldSnapshot, Gust, Jet, Turbulence, Vortex};

pub const VERSION: u16 = 1;
pub const SCENE_VERSION: u32 = 116;
pub const MAX_BODIES: usize = 256;
pub const MAX_PANELS: usize = 4096;
pub const MAX_WORK: usize = 4_000_000;
pub const MAX_COORDINATE: f64 = 1_000_000.;
fn version() -> u16 {
    VERSION
}
fn substeps() -> u8 {
    4
}
fn density() -> f64 {
    1.225
}
fn thickness() -> f64 {
    0.002
}
fn pressure() -> f64 {
    1.8
}
fn friction() -> [f64; 2] {
    [0.03, 0.03]
}
fn restitution() -> f32 {
    0.05
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    #[serde(default = "version")]
    pub version: u16,
    #[serde(default)]
    pub field: Field,
    #[serde(default = "density")]
    pub air_density: f64,
    #[serde(default = "substeps")]
    pub substeps: u8,
    #[serde(default)]
    pub bodies: Vec<Body>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: VERSION,
            field: Field::default(),
            air_density: density(),
            substeps: substeps(),
            bodies: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Body {
    pub instance: String,
    /// Total sheet mass; an optional point ballast adds its own mass.
    pub sheet_mass_kg: f64,
    #[serde(default)]
    pub ballast_mass_kg: f64,
    #[serde(default)]
    pub ballast_position: [f64; 3],
    #[serde(default = "thickness")]
    pub thickness_m: f64,
    #[serde(default = "pressure")]
    pub normal_drag: f64,
    #[serde(default = "friction")]
    pub tangent_drag: [f64; 2],
    #[serde(default)]
    pub velocity: [f32; 3],
    #[serde(default)]
    pub angular_velocity: [f32; 3],
    #[serde(default = "restitution")]
    pub restitution: f32,
    pub panels: Vec<Panel>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BodyState {
    pub id: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Inspect,
    Open {
        open: bool,
    },
    Replace {
        document: Arc<Document>,
    },
    Sample {
        positions: Vec<[f64; 3]>,
        time_seconds: f64,
    },
    Save {
        path: String,
    },
    Load {
        path: String,
    },
}
pub fn document_empty(d: &Arc<Document>) -> bool {
    d.is_empty()
}
impl Document {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
    pub fn validate(&self) -> Result<()> {
        self.field.validate()?;
        if self.version != VERSION
            || !(1..=16).contains(&self.substeps)
            || !self.air_density.is_finite()
            || !(0. ..=20.).contains(&self.air_density)
            || self.bodies.len() > MAX_BODIES
        {
            return Err(invalid("Wind version, density, substeps or body budget"));
        }
        let panels: usize = self.bodies.iter().map(|b| b.panels.len()).sum();
        if panels > MAX_PANELS
            || panels * 3 * usize::from(self.substeps) * self.field.sample_work() > MAX_WORK
        {
            return Err(invalid("Aerodynamic panel/sample work budget exceeded"));
        }
        let mut names = BTreeSet::new();
        for b in &self.bodies {
            if b.instance.is_empty()
                || b.instance.len() > 64
                || !names.insert(&b.instance)
                || b.panels.is_empty()
                || b.panels.len() > 256
                || !bounded(b.sheet_mass_kg, 1e-7, 10000.)
                || !bounded(b.ballast_mass_kg, 0., 10000.)
                || !bounded(b.thickness_m, 1e-5, 1.)
                || !bounded(b.normal_drag, 0., 10.)
                || b.tangent_drag.iter().any(|&c| !bounded(c, 0., 10.))
                || b.ballast_position
                    .iter()
                    .any(|&x| !bounded(x, -10000., 10000.))
                || b.velocity
                    .iter()
                    .chain(&b.angular_velocity)
                    .any(|&x| !x.is_finite() || x.abs() > 1000.)
                || !b.restitution.is_finite()
                || !(0. ..=1.).contains(&b.restitution)
            {
                return Err(invalid(
                    "Invalid aerodynamic body, identity, coefficients or budget",
                ));
            }
            compile_surfaces(b, 1.)?;
        }
        Ok(())
    }
    pub fn validate_scene(&self, scene: &Scene) -> Result<()> {
        self.validate()?;
        if !self.is_empty() && scene.version < SCENE_VERSION {
            return Err(invalid(format!(
                "Wind1 requires scene{SCENE_VERSION} or newer"
            )));
        }
        for b in &self.bodies {
            let i = scene
                .meshes
                .instances
                .iter()
                .find(|i| i.id == b.instance)
                .ok_or_else(|| invalid("Aerodynamic mesh instance not found"))?;
            let a = scene.meshes.asset(&i.asset)?;
            if i.playback.is_some()
                || i.attachment.is_some()
                || a.rig.is_some()
                || !a.clips.is_empty()
                || scene
                    .meshes
                    .instances
                    .iter()
                    .any(|c| c.attachment.as_ref().is_some_and(|a| a.parent == i.id))
                || scene.collision_meshes.contains(&i.id)
                || scene
                    .plants
                    .connections
                    .iter()
                    .any(|c| c.parent_instance == i.id || c.child_instance == i.id)
                || scene
                    .meshes
                    .objects
                    .iter()
                    .any(|o| o.members.contains(&i.id))
                || scene.botany.iter().any(|r| r.parts().contains(&i.id))
            {
                return Err(invalid(
                    "Free aerodynamics requires an independent static mesh, without static collision/object/botany ownership",
                ));
            }
            compile_surfaces(b, f64::from(i.scale))?;
        }
        Ok(())
    }
    pub fn work(&self) -> usize {
        self.bodies
            .iter()
            .map(|b| b.panels.len() * 3)
            .sum::<usize>()
            * usize::from(self.substeps)
            * self.field.sample_work()
    }
}
pub(crate) fn bounded(x: f64, lo: f64, hi: f64) -> bool {
    x.is_finite() && x >= lo && x <= hi
}
pub(crate) fn vec(v: [f64; 3], limit: f64) -> bool {
    v.iter().all(|&x| bounded(x, -limit, limit))
}
pub(crate) fn direction(v: [f64; 3]) -> bool {
    vec(v, 1.) && (DVec3::from_array(v).length_squared() - 1.).abs() < 1e-10
}
