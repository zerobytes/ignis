//! Structured profiles and bounded section lofts in an explicit material volume.
use crate::{
    mesh_preservation::{self, DisplacementPolicy},
    meshes,
};
use crate::{
    meshes::{Geometry, MAX_TRIANGLES, MAX_VERTICES},
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
mod assembly;
mod connection;
mod graft;
mod material;
pub use assembly::{Assembly, ContextSurface};
pub use connection::{Connection, ConnectionReport};
pub use graft::{CollarSurface, ContextGraft};
mod curves;
use curves::{Curves, values_at};
mod trajectory;
use trajectory::Compiled as CompiledTrajectory;
pub use trajectory::{Report as TrajectoryReport, Trajectory};

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub definition: Definition,
    pub reference: Definition,
    pub source: Arc<meshes::Asset>,
    pub detail_policy: DisplacementPolicy,
    pub animation_translation_policy: DisplacementPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<Assembly>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    Put {
        definition: Definition,
        detail_policy: DisplacementPolicy,
        animation_translation_policy: DisplacementPolicy,
    },
    /// Add an axis explicitly while retaining the historical source/reference.
    AdoptTrajectory {
        definition: Definition,
        detail_policy: DisplacementPolicy,
        animation_translation_policy: DisplacementPolicy,
    },
    SetAssembly {
        id: String,
        assembly: Option<Assembly>,
    },
    PutConnection {
        connection: Box<Connection>,
    },
    RemoveConnection {
        id: String,
    },
    /// Keep the current evaluated asset, explicitly ending parametric regeneration.
    Detach {
        id: String,
    },
}

pub fn default_sampling_tolerance() -> f32 {
    1. / 1024.
}

#[derive(Debug, Clone, Serialize)]
pub struct PlantReport {
    #[serde(flatten)]
    pub preservation: mesh_preservation::Report,
    pub sampled_reconstruction_error: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trajectory: Option<TrajectoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assembly: Option<assembly::Report>,
}

pub struct Evaluated {
    pub asset: meshes::Asset,
    pub report: PlantReport,
}

impl Entry {
    pub fn evaluate(&self) -> Result<Evaluated> {
        self.reference.validate()?;
        self.definition.validate()?;
        if self.reference.id != self.definition.id
            || self.source.id != self.definition.id
            || self.reference.profile.id != self.definition.profile.id
            || !self.reference.compatible_sections(&self.definition)
            || (self.reference.trajectory.is_some()
                && !self.reference.compatible_trajectory(&self.definition))
        {
            return Err(invalid(
                "Plant source/profile reference is missing or changed; detach explicitly before replacing its identity",
            ));
        }
        let reference = self.reference.sample()?;
        let target = self.definition.sample()?;
        let mut correspondences = Vec::with_capacity(target.material.len());
        for (&p, &face_index) in target.material.iter().zip(&target.surface) {
            let (triangle, barycentric) = reference.correspondence(face_index as usize, p);
            correspondences.push(mesh_preservation::Correspondence {
                triangle,
                barycentric,
            });
        }
        let input = mesh_preservation::Evaluation {
            source: &self.source,
            reference_base: &reference.geometry,
            target_base: &target.geometry,
            reference_dimensions: self.reference.dimensions(),
            target_dimensions: self.definition.dimensions(),
            correspondences: &correspondences,
            detail_policy: self.detail_policy,
            animation_translation_policy: self.animation_translation_policy,
        };
        let evaluated =
            if self.definition.sections.is_some() || self.definition.trajectory.is_some() {
                let origins = self.mapped_bone_origins()?;
                mesh_preservation::evaluate_material(
                    &input,
                    &mesh_preservation::MaterialDomain {
                        reference: &reference.material,
                        target: &target.material,
                        bone_origins: &origins,
                    },
                )?
            } else {
                mesh_preservation::evaluate(&input)?
            };
        let error = self.sampled_error(&reference, &target, &evaluated.asset);
        if error > f64::from(self.definition.sampling_tolerance) {
            return Err(invalid(format!(
                "Plant resolution insufficient: sampled error {error}m exceeds {}m",
                self.definition.sampling_tolerance
            )));
        }
        let (asset, assembly) = if let Some(recipe) = &self.assembly {
            let (asset, report) = recipe.evaluate(&evaluated.asset, &self.definition, &target)?;
            (asset, Some(report))
        } else {
            (evaluated.asset, None)
        };
        Ok(Evaluated {
            asset,
            report: PlantReport {
                preservation: evaluated.report,
                sampled_reconstruction_error: error,
                trajectory: target.trajectory_report,
                assembly,
            },
        })
    }
}

impl Entry {
    fn mapped_bone_origins(&self) -> Result<Vec<[f32; 3]>> {
        use glam::{DQuat, DVec3};
        meshes::Document {
            objects: Default::default(),
            assets: vec![(*self.source).clone()],
            instances: vec![],
        }
        .validate()?;
        let Some(rig) = &self.source.rig else {
            return Ok(vec![]);
        };
        let mut positions = [DVec3::ZERO; meshes::MAX_BONES];
        let mut rotations = [DQuat::IDENTITY; meshes::MAX_BONES];
        let mut out = Vec::with_capacity(rig.bones.len());
        for (i, bone) in rig.bones.iter().enumerate() {
            let local = DVec3::from_array(bone.translation.map(f64::from));
            let rotation = DQuat::from_array(bone.rotation.map(f64::from)).normalize();
            if let Some(p) = bone.parent {
                positions[i] = positions[p as usize] + rotations[p as usize] * local;
                rotations[i] = rotations[p as usize] * rotation;
            } else {
                positions[i] = local;
                rotations[i] = rotation;
            }
            let material = self.reference.material_at(positions[i].to_array())?;
            out.push(self.definition.position_at(material).map(|v| v as f32));
        }
        Ok(out)
    }
    pub fn sampled_error(
        &self,
        reference: &Sampling,
        target: &Sampling,
        asset: &meshes::Asset,
    ) -> f64 {
        use glam::DVec3;
        let from = DVec3::from_array(self.reference.dimensions().map(f64::from));
        let to = DVec3::from_array(self.definition.dimensions().map(f64::from));
        let mut maximum: f64 = 0.;
        for (i, &face_index) in reference.surface.iter().enumerate() {
            let (triangle, bary) =
                target.correspondence(face_index as usize, reference.material[i]);
            let indices = target.geometry.triangles[triangle as usize];
            let sampled = indices
                .into_iter()
                .zip(bary)
                .fold(DVec3::ZERO, |sum, (index, w)| {
                    sum + DVec3::from_array(asset.geometry.positions[index as usize].map(f64::from))
                        * f64::from(w)
                });
            let original = DVec3::from_array(self.source.geometry.positions[i].map(f64::from));
            let base = DVec3::from_array(reference.geometry.positions[i].map(f64::from));
            let mut detail = original - base;
            if self.detail_policy == DisplacementPolicy::Proportional {
                detail *= to / from;
            }
            let target_base = indices
                .into_iter()
                .zip(bary)
                .fold(DVec3::ZERO, |sum, (index, w)| {
                    sum + DVec3::from_array(
                        target.geometry.positions[index as usize].map(f64::from),
                    ) * f64::from(w)
                });
            let expected =
                if self.definition.sections.is_some() || self.definition.trajectory.is_some() {
                    target_base + detail
                } else {
                    DVec3::from_array(reference.material[i].map(f64::from)) * to + detail
                };
            maximum = maximum.max(sampled.distance(expected));
        }
        maximum
    }
}

impl Document {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.connections.is_empty()
    }
    pub fn entry(&self, id: &str) -> Result<&Entry> {
        self.entries
            .iter()
            .find(|e| e.definition.id == id)
            .ok_or_else(|| invalid(format!("Plant {id} not found")))
    }
    pub fn validate(&self, assets: &meshes::Document) -> Result<()> {
        let mut ids = std::collections::BTreeSet::new();
        let mut canonical = meshes::Document::default();
        for e in &self.entries {
            if !ids.insert(&e.definition.id) {
                return Err(invalid("Duplicate plant ID"));
            }
            canonical.assets.push((*e.source).clone());
            if let Some(a) = &e.assembly {
                canonical.assets.push((*a.context).clone());
                canonical
                    .assets
                    .extend(a.context_grafts.iter().map(|g| (*g.source).clone()));
            }
        }
        canonical.validate()?;
        for (e, evaluated) in self
            .entries
            .iter()
            .zip(self.evaluate_connections(assets)?.0)
        {
            if assets.asset(&e.definition.id)? != &evaluated.asset {
                return Err(invalid(format!(
                    "Plant {} evaluated asset disagrees with canonical source",
                    e.definition.id
                )));
            }
        }
        Ok(())
    }
    pub(crate) fn apply(&mut self, edit: Edit, assets: &mut meshes::Document) -> Result<()> {
        match edit {
            Edit::Put {
                definition,
                detail_policy,
                animation_translation_policy,
            } => {
                definition.validate()?;
                let entry = if let Some(old) = self
                    .entries
                    .iter_mut()
                    .find(|e| e.definition.id == definition.id)
                {
                    if !old.definition.compatible_trajectory(&definition) {
                        return Err(invalid(
                            "Changing trajectory identity requires explicit adoption or detach",
                        ));
                    }
                    old.definition = definition;
                    old.detail_policy = detail_policy;
                    old.animation_translation_policy = animation_translation_policy;
                    old
                } else {
                    if assets.assets.iter().any(|a| a.id == definition.id) {
                        return Err(invalid("Plant ID already belongs to an independent mesh"));
                    }
                    let source = meshes::Asset {
                        id: definition.id.clone(),
                        geometry: Arc::new(definition.sample()?.geometry),
                        shading: meshes::Shading::Flat,
                        material: Default::default(),
                        rig: None,
                        clips: vec![],
                    };
                    self.entries.push(Entry {
                        reference: definition.clone(),
                        definition,
                        source: Arc::new(source),
                        detail_policy,
                        animation_translation_policy,
                        assembly: None,
                    });
                    self.entries.last_mut().expect("inserted plant")
                };
                let evaluated = entry.evaluate()?;
                assets.apply(meshes::Edit::PutAsset {
                    asset: evaluated.asset,
                })?;
            }
            Edit::AdoptTrajectory {
                definition,
                detail_policy,
                animation_translation_policy,
            } => {
                definition.validate()?;
                let old = self
                    .entries
                    .iter_mut()
                    .find(|e| e.definition.id == definition.id)
                    .ok_or_else(|| invalid("Trajectory adoption requires an existing plant"))?;
                if old.definition.trajectory.is_some()
                    || old.reference.trajectory.is_some()
                    || definition.trajectory.is_none()
                {
                    return Err(invalid(
                        "Trajectory adoption requires an unbound axis and a new trajectory",
                    ));
                }
                old.definition = definition;
                old.detail_policy = detail_policy;
                old.animation_translation_policy = animation_translation_policy;
                assets.apply(meshes::Edit::PutAsset {
                    asset: old.evaluate()?.asset,
                })?;
            }
            Edit::SetAssembly { id, assembly } => {
                let entry = self
                    .entries
                    .iter_mut()
                    .find(|e| e.definition.id == id)
                    .ok_or_else(|| invalid("Assembly requires an existing plant"))?;
                entry.assembly = assembly;
                assets.apply(meshes::Edit::PutAsset {
                    asset: entry.evaluate()?.asset,
                })?;
            }
            Edit::Detach { id } => {
                let index = self
                    .entries
                    .iter()
                    .position(|e| e.definition.id == id)
                    .ok_or_else(|| invalid("Plant not found"))?;
                self.entries.remove(index);
            }
            Edit::PutConnection { connection } => {
                if !valid_id(&connection.id) {
                    return Err(invalid("Connection requires a stable ID"));
                }
                if let Some(old) = self.connections.iter_mut().find(|c| c.id == connection.id) {
                    *old = *connection;
                } else {
                    self.connections.push(*connection);
                }
            }
            Edit::RemoveConnection { id } => {
                let index = self
                    .connections
                    .iter()
                    .position(|c| c.id == id)
                    .ok_or_else(|| invalid("Connection not found"))?;
                let old = self.connections.remove(index);
                let instance = assets
                    .instances
                    .iter()
                    .find(|i| i.id == old.parent_instance)
                    .ok_or_else(|| invalid("Connection parent instance is absent"))?;
                let evaluated = self.entry(&instance.asset)?.evaluate()?;
                assets.apply(meshes::Edit::PutAsset {
                    asset: evaluated.asset,
                })?;
            }
        }
        Ok(())
    }
    pub(crate) fn apply_mesh(
        &mut self,
        edit: meshes::Edit,
        assets: &mut meshes::Document,
    ) -> Result<()> {
        self.apply_mesh_report(edit, assets).map(|_| ())
    }
    pub(crate) fn apply_mesh_report(
        &mut self,
        edit: meshes::Edit,
        assets: &mut meshes::Document,
    ) -> Result<Option<crate::mesh_modeling::Report>> {
        if let meshes::Edit::SculptStroke { asset, .. }
        | meshes::Edit::SculptDetailed { asset, .. }
        | meshes::Edit::ModelSelection { asset, .. } = &edit
            && self.connections.iter().any(|c| {
                assets.instances.iter().any(|i| {
                    i.asset == *asset && (i.id == c.parent_instance || i.id == c.child_instance)
                })
            })
        {
            return Err(invalid(
                "Remove plant connections explicitly before topology modeling",
            ));
        }
        if let meshes::Edit::SculptStroke { asset, .. }
        | meshes::Edit::SculptDetailed { asset, .. }
        | meshes::Edit::ModelSelection { asset, .. }
        | meshes::Edit::TransformVertices { asset, .. }
        | meshes::Edit::TransformIslands { asset, .. }
        | meshes::Edit::RemoveFaces { asset, .. }
        | meshes::Edit::BridgeBoundaryLoops { asset, .. } = &edit
            && self.entries.iter().any(|e| e.definition.id == *asset)
        {
            return Err(invalid(
                "Detach plant explicitly before local cuts, boundary bridges or island transforms",
            ));
        }
        if let meshes::Edit::ExtractIslands {
            asset,
            mode: meshes::ExtractionMode::Move,
            ..
        } = &edit
            && self.entries.iter().any(|e| e.definition.id == *asset)
        {
            return Err(invalid(
                "Detach plant explicitly before moving its mesh islands",
            ));
        }
        if let meshes::Edit::Subdivide {
            asset, components, ..
        } = &edit
            && let Some(entry) = self.entries.iter().find(|e| e.definition.id == *asset)
        {
            if components.is_some() {
                return Err(invalid(
                    "Linked plants support uniform subdivision only; detach explicitly before local topology edits",
                ));
            }
            let mut definition = entry.definition.clone();
            definition.subdivision_levels = definition
                .subdivision_levels
                .checked_add(1)
                .ok_or_else(|| invalid("Plant subdivision level overflow"))?;
            definition.validate()?;
            let (vertices, triangles) = definition.counts()?;
            assets.check_replacement_counts(asset, vertices, triangles)?;
            return self
                .apply(
                    Edit::Put {
                        definition,
                        detail_policy: entry.detail_policy,
                        animation_translation_policy: entry.animation_translation_policy,
                    },
                    assets,
                )
                .map(|()| None);
        }
        // Appearance metadata must not rebase the canonical geometric source.
        if let meshes::Edit::SetMaterial { asset, material } = &edit
            && let Some(entry) = self.entries.iter_mut().find(|e| e.definition.id == *asset)
        {
            Arc::make_mut(&mut entry.source).material = material.clone();
            if let Some(a) = &mut entry.assembly {
                Arc::make_mut(&mut a.context).material = material.clone();
                for graft in &mut a.context_grafts {
                    Arc::make_mut(&mut graft.source).material = material.clone();
                }
            }
        }
        if let meshes::Edit::SetShading { asset, shading } = &edit
            && let Some(entry) = self.entries.iter_mut().find(|e| e.definition.id == *asset)
        {
            Arc::make_mut(&mut entry.source).shading = *shading;
            if let Some(a) = &mut entry.assembly {
                Arc::make_mut(&mut a.context).shading = *shading;
                for graft in &mut a.context_grafts {
                    Arc::make_mut(&mut graft.source).shading = *shading;
                }
            }
        }
        let edited = match &edit {
            meshes::Edit::Sculpt { asset, .. }
            | meshes::Edit::SmoothRegion { asset, .. }
            | meshes::Edit::SetMaterialCoordinates { asset, .. }
            | meshes::Edit::FitMaterialCoordinates { asset, .. }
            | meshes::Edit::CopyMaterialCoordinates { asset, .. }
            | meshes::Edit::SetRig { asset, .. }
            | meshes::Edit::RetargetClip { asset, .. }
            | meshes::Edit::PutClip { asset, .. }
            | meshes::Edit::RemoveClip { asset, .. } => Some(asset.clone()),
            meshes::Edit::PutAsset { asset }
                if self.entries.iter().any(|e| e.definition.id == asset.id) =>
            {
                return Err(invalid("Detach plant before replacing its mesh source"));
            }
            meshes::Edit::Panel { id, .. }
            | meshes::Edit::Grid { id, .. }
            | meshes::Edit::Sweep { id, .. }
            | meshes::Edit::ChamferBoxes { id, .. }
            | meshes::Edit::RemoveAsset { id }
                if self.entries.iter().any(|e| e.definition.id == *id) =>
            {
                return Err(invalid(
                    "Detach plant before replacing/removing its mesh source",
                ));
            }
            _ => None,
        };
        if edited.as_ref().is_some_and(|id| {
            self.entries.iter().any(|e| {
                e.definition.id == *id
                    && (e.assembly.is_some()
                        || self.connections.iter().any(|c| {
                            assets.instances.iter().any(|i| {
                                i.asset == *id
                                    && (i.id == c.parent_instance || i.id == c.child_instance)
                            })
                        }))
            })
        }) {
            return Err(invalid(
                "Remove assembly/connection explicitly before rebasing its geometry, rig or clips",
            ));
        }
        let report = if let meshes::Edit::ModelSelection {
            asset, operation, ..
        } = edit
        {
            Some(assets.model_selection(&asset, &operation)?)
        } else {
            assets.apply(edit)?;
            None
        };
        if let Some(entry) =
            edited.and_then(|id| self.entries.iter_mut().find(|e| e.definition.id == id))
        {
            // Explicit direct edits establish a new authoring source; regeneration never does.
            entry.reference = entry.definition.clone();
            entry.source = Arc::new(assets.asset(&entry.definition.id)?.clone());
            let evaluated = entry.evaluate()?;
            assets.apply(meshes::Edit::PutAsset {
                asset: evaluated.asset,
            })?;
        }
        Ok(report)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rectangle {
    pub id: String,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub profile: Rectangle,
    pub depth: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trajectory: Option<Trajectory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<Sections>,
    /// Segments along local X/Y/Z, independent of dimensions.
    pub segments: [u32; 3],
    /// Uniform linear refinement after the six base grids; canonical source remains separate.
    #[serde(default, skip_serializing_if = "no_subdivisions")]
    pub subdivision_levels: u8,
    /// Maximum sampled reconstruction error at canonical vertices, in meters.
    #[serde(default = "default_sampling_tolerance")]
    pub sampling_tolerance: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrossSection {
    Rectangle,
    Diamond,
    Ellipse,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Station {
    pub id: String,
    /// Longitudinal position, normalized from0 to1 along local Y.
    pub at: f32,
    /// Fractions of the global profile width and depth, both strictly positive.
    pub width: f32,
    pub depth: f32,
    /// Center in the parallel XZ section plane, in local meters.
    #[serde(default, skip_serializing_if = "zero_offset")]
    pub offset: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sections {
    pub id: String,
    pub cross_section: CrossSection,
    #[serde(default, skip_serializing_if = "SectionInterpolation::is_linear")]
    pub interpolation: SectionInterpolation,
    pub stations: Vec<Station>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SectionInterpolation {
    #[default]
    Linear,
    Pchip,
}
impl SectionInterpolation {
    fn is_linear(&self) -> bool {
        *self == Self::Linear
    }
}

#[derive(Debug, Serialize)]
pub struct Sampling {
    pub geometry: Geometry,
    pub material: Vec<[f32; 3]>,
    /// Canonical surface identity for EVERY vertex, including appended midpoints.
    pub surface: Vec<u8>,
    pub subdivision_levels: u8,
    pub trajectory_report: Option<TrajectoryReport>,
    /// Base triangle-grid offsets, ordered axis X/Y/Z then negative/positive.
    /// Ellipse grids share vertices; `first_vertex` is contiguous only for legacy shapes.
    /// Use `material` and `surface` for complete vertex provenance.
    pub faces: [Face; 6],
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Face {
    pub first_vertex: usize,
    pub first_triangle: usize,
    pub u_segments: u32,
    pub v_segments: u32,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}
fn no_subdivisions(levels: &u8) -> bool {
    *levels == 0
}
fn zero_offset(offset: &[f32; 2]) -> bool {
    *offset == [0.; 2]
}

/// Square-to-disk radial map and its exact inverse. The material domain stays
/// piecewise linear, so correspondence never interpolates across an angle seam.
fn disk(x: f64, z: f64, inverse: bool) -> (f64, f64) {
    let maximum = x.abs().max(z.abs());
    if maximum == 0. {
        return (0., 0.);
    }
    let scale = if inverse {
        x.hypot(z) / maximum
    } else {
        maximum / x.hypot(z)
    };
    (x * scale, z * scale)
}

impl Definition {
    fn compatible_trajectory(&self, target: &Self) -> bool {
        match (&self.trajectory, &target.trajectory) {
            (None, None) => true,
            (Some(a), Some(b)) => a.id() == b.id(),
            _ => false,
        }
    }
    fn section_radius_bound(&self) -> f64 {
        let (width, depth, offset) = if let Some(sections) = &self.sections {
            sections
                .stations
                .iter()
                .fold((0.0_f64, 0.0_f64, [0.0_f64; 2]), |(w, d, o), s| {
                    (
                        w.max(f64::from(s.width)),
                        d.max(f64::from(s.depth)),
                        [
                            o[0].max(f64::from(s.offset[0]).abs()),
                            o[1].max(f64::from(s.offset[1]).abs()),
                        ],
                    )
                })
        } else {
            (1., 1., [0.; 2])
        };
        (f64::from(self.profile.width) * width * 0.5 + offset[0])
            .hypot(f64::from(self.depth) * depth * 0.5 + offset[1])
    }
    pub fn requires_smooth_format(&self) -> bool {
        self.sections
            .as_ref()
            .is_some_and(|s| s.interpolation == SectionInterpolation::Pchip)
    }
    pub fn requires_curved_format(&self) -> bool {
        self.sections.as_ref().is_some_and(|s| {
            s.cross_section == CrossSection::Ellipse
                || s.stations.iter().any(|s| !zero_offset(&s.offset))
        })
    }
    fn shared_surface(&self) -> bool {
        self.sections
            .as_ref()
            .is_some_and(|s| s.cross_section == CrossSection::Ellipse)
    }
    fn compatible_sections(&self, target: &Self) -> bool {
        match (&self.sections, &target.sections) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.id == b.id
                    && a.cross_section == b.cross_section
                    && a.stations
                        .iter()
                        .map(|s| &s.id)
                        .eq(b.stations.iter().map(|s| &s.id))
            }
            _ => false,
        }
    }

    fn intervals(&self) -> usize {
        self.sections
            .as_ref()
            .map_or(1, |s| s.stations.len().saturating_sub(1))
    }

    /// Forward volume map. The material Y intervals refer to station identities,
    /// independently of their physical spacing.
    pub fn position_at(&self, material: [f64; 3]) -> [f64; 3] {
        let curves = self.sections.as_ref().and_then(Curves::new);
        let trajectory = self
            .trajectory
            .as_ref()
            .map(|t| t.compile(f64::from(self.profile.height)));
        self.position_with(material, curves.as_ref(), trajectory.as_ref())
    }

    fn position_with(
        &self,
        material: [f64; 3],
        curves: Option<&Curves>,
        trajectory: Option<&CompiledTrajectory>,
    ) -> [f64; 3] {
        let [mut x, y, mut z] = material;
        let [w, h, d] = self.dimensions().map(f64::from);
        let mut at = y + 0.5;
        let mut width = 1.;
        let mut depth = 1.;
        let mut offset = [0.; 2];
        if let Some(sections) = &self.sections {
            let t = ((y + 0.5) * self.intervals() as f64).clamp(0., self.intervals() as f64);
            let i = (t.floor() as usize).min(self.intervals() - 1);
            let a = &sections.stations[i];
            let b = &sections.stations[i + 1];
            let f = t - i as f64;
            at = f64::from(a.at) * (1. - f) + f64::from(b.at) * f;
            [width, depth, offset[0], offset[1]] = values_at(sections, curves, i, f);
            if sections.cross_section == CrossSection::Diamond {
                (x, z) = ((x + z) * 0.5, (z - x) * 0.5);
            } else if sections.cross_section == CrossSection::Ellipse {
                (x, z) = disk(x, z, false);
            }
        }
        if let Some(trajectory) = trajectory {
            return trajectory
                .frame(at)
                .position(offset[0] + x * w * width, offset[1] + z * d * depth);
        }
        [
            offset[0] + x * w * width,
            (at - 0.5) * h,
            offset[1] + z * d * depth,
        ]
    }

    /// Inverse on the valid solid, including its boundary. No nearest projection.
    pub fn material_at(&self, position: [f64; 3]) -> Result<[f64; 3]> {
        let [w, h, d] = self.dimensions().map(f64::from);
        let [mut x, mut y, mut z] = if let Some(trajectory) = &self.trajectory {
            let [x, at, z] = trajectory.compile(h).inverse(position)?;
            [x / w, at - 0.5, z / d]
        } else {
            [position[0] / w, position[1] / h, position[2] / d]
        };
        if let Some(sections) = &self.sections {
            let at = y + 0.5;
            let i = sections
                .stations
                .windows(2)
                .position(|pair| at <= f64::from(pair[1].at))
                .unwrap_or(self.intervals() - 1);
            let a = &sections.stations[i];
            let b = &sections.stations[i + 1];
            let f = (at - f64::from(a.at)) / (f64::from(b.at) - f64::from(a.at));
            let curves = Curves::new(sections);
            let [width, depth, offset_x, offset_z] = values_at(sections, curves.as_ref(), i, f);
            let offset = [offset_x, offset_z];
            x -= offset[0] / w;
            z -= offset[1] / d;
            x /= width;
            z /= depth;
            if sections.cross_section == CrossSection::Diamond {
                (x, z) = (x - z, x + z);
            } else if sections.cross_section == CrossSection::Ellipse {
                (x, z) = disk(x, z, true);
            }
            y = (i as f64 + f) / self.intervals() as f64 - 0.5;
        }
        let material = [x, y, z];
        if material
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 0.5 + mesh_preservation::MAPPING_TOLERANCE)
        {
            return Err(invalid(
                "Plant bind origin lies outside the reference section volume",
            ));
        }
        Ok(material.map(|v| v.clamp(-0.5, 0.5)))
    }
    pub fn dimensions(&self) -> [f32; 3] {
        [self.profile.width, self.profile.height, self.depth]
    }

    pub fn validate(&self) -> Result<()> {
        if !valid_id(&self.id) || !valid_id(&self.profile.id) {
            return Err(invalid(
                "Plant/profile IDs require1..64 ASCII alphanumeric,_,- characters",
            ));
        }
        if self
            .dimensions()
            .iter()
            .any(|d| !d.is_finite() || !(0.01..=100.).contains(d))
        {
            return Err(invalid("Plant dimensions require finite0.01..100 meters"));
        }
        if !self.sampling_tolerance.is_finite()
            || !(0.000001..=1.).contains(&self.sampling_tolerance)
        {
            return Err(invalid(
                "Plant sampling tolerance requires finite0.000001..1 meters",
            ));
        }
        if let Some(s) = &self.sections {
            if !valid_id(&s.id) || !(2..=16).contains(&s.stations.len()) {
                return Err(invalid("Section domain requires an ID and2..16 stations"));
            }
            let mut ids = std::collections::BTreeSet::new();
            for station in &s.stations {
                if !valid_id(&station.id)
                    || !ids.insert(&station.id)
                    || !station.at.is_finite()
                    || !(0.0..=1.).contains(&station.at)
                    || [station.width, station.depth]
                        .iter()
                        .any(|v| !v.is_finite() || !(0.001..=1.).contains(v))
                    || station
                        .offset
                        .iter()
                        .any(|v| !v.is_finite() || v.abs() > 100.)
                {
                    return Err(invalid(
                        "Sections require unique IDs, finite positions0..1, width/depth fractions0.001..1 and offsets within100m",
                    ));
                }
            }
            if s.stations[0].at != 0.
                || s.stations.last().expect("stations").at != 1.
                || s.stations.windows(2).any(|p| p[1].at - p[0].at < 0.001)
            {
                return Err(invalid(
                    "Section stations must span0..1 in increasing order with spacing at least0.001",
                ));
            }
        }
        if let Some(trajectory) = &self.trajectory {
            trajectory.validate(f64::from(self.profile.height), self.section_radius_bound())?;
        }
        self.counts()?;
        Ok(())
    }
    /// Exact counts for material face grids, shared for ellipses; preflight every level.
    pub fn counts(&self) -> Result<(usize, usize)> {
        if self.segments.iter().any(|n| !(1..=63).contains(n)) {
            return Err(invalid("Plant segments require1..63 on each axis"));
        }
        let [x, y, z] = self.segments.map(|n| n as usize);
        let y = y * self.intervals();
        if self.subdivision_levels > 6 {
            return Err(invalid(
                "Plant subdivision levels require0..6 within mesh budgets",
            ));
        }
        let mut triangles = 4 * (x * y + x * z + y * z);
        let mut vertices = if self.shared_surface() {
            2 * (x * y + x * z + y * z) + 2
        } else {
            2 * ((x + 1) * (y + 1) + (x + 1) * (z + 1) + (y + 1) * (z + 1))
        };
        let mut edges = if self.shared_surface() {
            triangles * 3 / 2
        } else {
            6 * (x * y + x * z + y * z) + 4 * (x + y + z)
        };
        for level in 0..=self.subdivision_levels {
            if vertices > MAX_VERTICES || triangles > MAX_TRIANGLES {
                return Err(invalid(
                    "Plant sampling exceeds mesh vertex/triangle budget",
                ));
            }
            if level < self.subdivision_levels {
                vertices += edges;
                edges = 2 * edges + 3 * triangles;
                triangles *= 4;
            }
        }
        Ok((vertices, triangles))
    }

    pub fn sample(&self) -> Result<Sampling> {
        self.validate()?;
        // Coefficients are bounded and compiled once for all vertices.
        let curves = self.sections.as_ref().and_then(Curves::new);
        let trajectory = self
            .trajectory
            .as_ref()
            .map(|t| t.compile(f64::from(self.profile.height)));
        let [x, y, z] = self.segments.map(|n| n as usize);
        let y = y * self.intervals();
        let segments = [x as u32, y as u32, z as u32];
        let shared = self.shared_surface();
        let count = if shared {
            2 * (x * y + x * z + y * z) + 2
        } else {
            2 * ((x + 1) * (y + 1) + (x + 1) * (z + 1) + (y + 1) * (z + 1))
        };
        let mut out = Sampling {
            geometry: Geometry {
                material_coordinates: None,
                positions: Vec::with_capacity(count),
                triangles: Vec::with_capacity(4 * (x * y + x * z + y * z)),
            },
            material: Vec::with_capacity(count),
            surface: Vec::with_capacity(count),
            subdivision_levels: 0,
            trajectory_report: None,
            faces: [Face::default(); 6],
        };
        let mut vertices = std::collections::BTreeMap::new();
        for axis in 0..3 {
            let u = (axis + 1) % 3;
            let v = (axis + 2) % 3;
            let nu = segments[u];
            let nv = segments[v];
            for side in 0..2 {
                let face = Face {
                    first_vertex: out.material.len(),
                    first_triangle: out.geometry.triangles.len(),
                    u_segments: nu,
                    v_segments: nv,
                };
                out.faces[axis * 2 + side] = face;
                let mut grid = Vec::with_capacity(((nu + 1) * (nv + 1)) as usize);
                for j in 0..=nv {
                    for i in 0..=nu {
                        let mut p = [0.; 3];
                        p[axis] = if side == 0 { -0.5 } else { 0.5 };
                        p[u] = i as f32 / nu as f32 - 0.5;
                        p[v] = j as f32 / nv as f32 - 0.5;
                        let mut key = [0; 3];
                        key[axis] = if side == 0 { 0 } else { segments[axis] };
                        key[u] = i;
                        key[v] = j;
                        if shared && let Some(&index) = vertices.get(&key) {
                            grid.push(index);
                            continue;
                        }
                        let index = out.material.len() as u32;
                        grid.push(index);
                        if shared {
                            vertices.insert(key, index);
                        }
                        out.material.push(p);
                        out.surface.push((axis * 2 + side) as u8);
                        out.geometry.positions.push(
                            self.position_with(
                                p.map(f64::from),
                                curves.as_ref(),
                                trajectory.as_ref(),
                            )
                            .map(|v| v as f32),
                        );
                    }
                }
                for j in 0..nv {
                    for i in 0..nu {
                        let a = (j * (nu + 1) + i) as usize;
                        let b = a + 1;
                        let c = a + nu as usize + 1;
                        let d = c + 1;
                        let [a, b, c, d] = [a, b, c, d].map(|i| grid[i]);
                        for mut tri in [[a, b, d], [a, d, c]] {
                            if side == 0 {
                                tri.swap(1, 2);
                            }
                            out.geometry.triangles.push(tri);
                        }
                    }
                }
            }
        }
        for _ in 0..self.subdivision_levels {
            let plan = crate::mesh_topology::Plan::new(&out.geometry, None)?;
            for &[a, b] in plan.split_edges() {
                out.material.push(std::array::from_fn(|i| {
                    ((f64::from(out.material[a as usize][i])
                        + f64::from(out.material[b as usize][i]))
                        * 0.5) as f32
                }));
                let surface = if shared {
                    (0..6)
                        .find(|&face| {
                            let axis = face / 2;
                            let side = if face % 2 == 0 { -0.5 } else { 0.5 };
                            out.material[a as usize][axis] == side
                                && out.material[b as usize][axis] == side
                        })
                        .expect("edge belongs to a material face") as u8
                } else {
                    out.surface[a as usize]
                };
                out.surface.push(surface);
            }
            out.geometry = plan.geometry(&out.geometry)?;
            out.subdivision_levels += 1;
        }
        if shared {
            out.geometry.validate()?;
        }
        if let Some(trajectory) = &trajectory {
            let mut error = 0.0_f64;
            let mut probes = 0;
            for tri in &out.geometry.triangles {
                if [-0.5, 0.5]
                    .into_iter()
                    .any(|at| tri.iter().all(|&i| out.material[i as usize][1] == at))
                {
                    continue;
                }
                for bary in [[0.5, 0.5, 0.], [0., 0.5, 0.5], [0.5, 0., 0.5], [1. / 3.; 3]] {
                    let material = std::array::from_fn(|k| {
                        tri.iter()
                            .zip(bary)
                            .map(|(&i, w)| f64::from(out.material[i as usize][k]) * w)
                            .sum()
                    });
                    let linear = glam::DVec3::from_array(std::array::from_fn(|k| {
                        tri.iter()
                            .zip(bary)
                            .map(|(&i, w)| f64::from(out.geometry.positions[i as usize][k]) * w)
                            .sum()
                    }));
                    let analytic = glam::DVec3::from_array(self.position_with(
                        material,
                        curves.as_ref(),
                        Some(trajectory),
                    ));
                    error = error.max(linear.distance(analytic));
                    probes += 1;
                }
            }
            if error > f64::from(trajectory.tolerance) {
                return Err(invalid(format!(
                    "Trajectory sampling insufficient: sampled surface error {error}m exceeds {}m; increase base segments",
                    trajectory.tolerance
                )));
            }
            out.trajectory_report = Some(TrajectoryReport {
                inverse_derivative_margin: trajectory.margin(self.section_radius_bound()),
                sampled_surface_error: error,
                surface_tolerance: trajectory.tolerance,
                surface_probes: probes,
            });
        }
        Ok(out)
    }
}

impl Sampling {
    /// Locate a target material coordinate on the same canonical face.
    /// Returns source indices and barycentric weights; never searches by proximity.
    pub fn correspondence(&self, face_index: usize, p: [f32; 3]) -> (u32, [f32; 3]) {
        let face = self.faces[face_index];
        let axis = face_index / 2;
        let u =
            ((p[(axis + 1) % 3] + 0.5) * face.u_segments as f32).clamp(0., face.u_segments as f32);
        let v =
            ((p[(axis + 2) % 3] + 0.5) * face.v_segments as f32).clamp(0., face.v_segments as f32);
        let i = (u.floor() as u32).min(face.u_segments - 1);
        let j = (v.floor() as u32).min(face.v_segments - 1);
        let fu = u - i as f32;
        let fv = v - j as f32;
        let second = fv > fu;
        let tri =
            face.first_triangle + ((j * face.u_segments + i) * 2) as usize + usize::from(second);
        let mut bary = if second {
            [1. - fv, fu, fv - fu]
        } else {
            [1. - fu, fu - fv, fv]
        };
        if face_index.is_multiple_of(2) {
            bary.swap(1, 2);
        }
        let mut tri = tri as u32;
        for _ in 0..self.subdivision_levels {
            let (child, local) = crate::mesh_topology::uniform_child(bary);
            bary = local;
            tri = tri * 4 + child;
        }
        (tri, bary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    fn definition(segments: [u32; 3]) -> Definition {
        Definition {
            trajectory: None,
            id: "limb".into(),
            profile: Rectangle {
                id: "outline".into(),
                width: 2.,
                height: 4.,
            },
            depth: 0.5,
            segments,
            sections: None,
            subdivision_levels: 0,
            sampling_tolerance: default_sampling_tolerance(),
        }
    }
    #[test]
    fn surface_has_outward_winding_and_expected_budgets() {
        let s = definition([2, 4, 1]).sample().unwrap();
        assert_eq!(s.material.len(), 62);
        assert_eq!(s.geometry.triangles.len(), 56);
        for t in &s.geometry.triangles {
            let [a, b, c] = t.map(|i| Vec3::from(s.geometry.positions[i as usize]));
            assert!((b - a).cross(c - a).dot((a + b + c) / 3.) > 0.);
        }
    }
    #[test]
    fn retessellation_correspondences_reproduce_material_surface() {
        let source = definition([2, 3, 1]).sample().unwrap();
        let target = definition([5, 7, 4]).sample().unwrap();
        for face in 0..6 {
            let f = target.faces[face];
            let count = ((f.u_segments + 1) * (f.v_segments + 1)) as usize;
            for p in &target.material[f.first_vertex..f.first_vertex + count] {
                let (triangle, w) = source.correspondence(face, *p);
                let indices = source.geometry.triangles[triangle as usize];
                let q = indices.into_iter().zip(w).fold(Vec3::ZERO, |sum, (i, w)| {
                    sum + Vec3::from(source.material[i as usize]) * w
                });
                assert!(q.distance(Vec3::from(*p)) < 1e-6);
                assert!(w.into_iter().all(|w| w >= -1e-6));
            }
        }
    }
    #[test]
    fn invalid_dimensions_ids_and_sampling_reject_before_generation() {
        for dimensions in [[0., 1., 1.], [f32::NAN, 1., 1.], [1., 1., 101.]] {
            let mut d = definition([1, 1, 1]);
            d.profile.width = dimensions[0];
            d.profile.height = dimensions[1];
            d.depth = dimensions[2];
            assert!(d.sample().is_err());
        }
        for segments in [[u32::MAX, 1, 1], [63; 3], [0, 1, 1]] {
            assert!(definition(segments).sample().is_err());
        }
        let mut d = definition([1; 3]);
        d.profile.id = "../bad".into();
        assert!(d.sample().is_err());
    }
}
