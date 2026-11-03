//! Bounded native mesh authoring. Visual assets never imply physical/sensory geometry.
use crate::model::{Result, invalid};
use glam::{Mat4, Quat, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};
mod attachments;
use attachments::{Hierarchy, Prepared};
mod extraction;
pub use extraction::{ExtractionMode, Island, IslandTransform, Islands};

// Legacy per-operation modeling budgets; these are not scene/render quotas.
pub const MAX_VERTICES: usize = 16384;
pub const MAX_TRIANGLES: usize = 32768;
/// u8 joint indices address the entire 0..=255 domain; never truncate a rig.
pub const MAX_BONES: usize = 256;
pub const MAX_INFLUENCES: usize = 16;
pub mod character;
pub const MAX_CLIPS: usize = 16;
pub const MAX_KEYS: usize = 4096;
pub const MAX_TICK: u32 = 36000;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    #[serde(default, skip_serializing_if = "crate::mesh_objects::is_empty")]
    pub objects: Arc<Vec<crate::mesh_objects::Object>>,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub instances: Vec<Instance>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_coordinates: Option<Vec<[f32; 3]>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub id: String,
    pub geometry: Arc<Geometry>,
    #[serde(default)]
    pub shading: Shading,
    #[serde(default)]
    pub material: crate::materials::Material,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rig: Option<Arc<Rig>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clips: Vec<Arc<Clip>>,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Shading {
    #[default]
    Flat,
    /// Area-weighted posed normals, shared only by authored vertex index.
    Smooth,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bone {
    pub name: String,
    pub parent: Option<u8>,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Weight {
    pub joints: [u8; 4],
    pub weights: [f32; 4],
    /// Keep the compact legacy four channels; allocate overflow only when needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Box<[Influence]>>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Influence {
    pub joint: u8,
    pub weight: f32,
}
impl Weight {
    pub fn influences(&self) -> impl Iterator<Item = (u8, f32)> + '_ {
        self.joints
            .iter()
            .copied()
            .zip(self.weights.iter().copied())
            .chain(
                self.extra
                    .iter()
                    .flat_map(|v| v.iter().map(|v| (v.joint, v.weight))),
            )
    }
    /// Preserve every positive influence; reject invalid totals or excess channels.
    pub fn from_sums(sums: &[f64; MAX_BONES]) -> Result<Self> {
        if sums.iter().any(|v| !v.is_finite() || *v < 0.)
            || (sums.iter().sum::<f64>() - 1.).abs() > 0.0001
        {
            return Err(invalid(
                "Skin influence sums must be finite, nonnegative and total1",
            ));
        }
        let mut out = Self {
            joints: [0; 4],
            weights: [0.; 4],
            extra: None,
        };
        let mut extra = Vec::new();
        let mut count = 0;
        for (joint, &weight) in sums.iter().enumerate() {
            if weight > 0. {
                if count == MAX_INFLUENCES {
                    return Err(invalid(
                        "Skin transfer exceeds16 influences; explicit reduction required",
                    ));
                }
                if count < 4 {
                    out.joints[count] = joint as u8;
                    out.weights[count] = weight as f32;
                } else {
                    extra.push(Influence {
                        joint: joint as u8,
                        weight: weight as f32,
                    });
                }
                count += 1;
            }
        }
        if !extra.is_empty() {
            out.extra = Some(extra.into_boxed_slice());
        }
        Ok(out)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rig {
    pub bones: Vec<Bone>,
    pub weights: Vec<Weight>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Keyframe {
    pub tick: u32,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub bone: u8,
    pub keys: Vec<Keyframe>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub id: String,
    pub duration_ticks: u32,
    pub tracks: Vec<Track>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Playback {
    pub clip: String,
    pub offset_ticks: u32,
    pub looping: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Instance {
    pub id: String,
    pub asset: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
    pub color: [f32; 3],
    #[serde(default)]
    pub playback: Option<Playback>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Attachment>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub parent: String,
    /// None follows the instance origin; Some follows an asset-space posed bone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bone: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BrushKind {
    Draw,
    Grab,
    Smooth,
    Flatten,
    Pinch,
    Crease,
    Scrape,
    Fill,
    Relax,
}
impl BrushKind {
    pub const ALL: [Self; 9] = [
        Self::Draw,
        Self::Grab,
        Self::Smooth,
        Self::Flatten,
        Self::Pinch,
        Self::Crease,
        Self::Scrape,
        Self::Fill,
        Self::Relax,
    ];
    pub fn frozen_surface(self) -> bool {
        !matches!(self, Self::Grab | Self::Smooth)
    }
    pub fn invertible(self) -> bool {
        matches!(
            self,
            Self::Draw | Self::Grab | Self::Pinch | Self::Crease | Self::Scrape | Self::Fill
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Stroke {
    pub kind: BrushKind,
    pub centers: Vec<[f32; 3]>,
    pub radius: f32,
    /// Metres per sample for draw/grab; blend factor [0,1] for smooth.
    pub strength: f32,
    /// Local displacement direction for grab, normalized on use.
    pub direction: [f32; 3],
}
impl Stroke {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.centers.is_empty()
            || self.centers.len() > 128
            || !self.radius.is_finite()
            || !(0.001..=100.).contains(&self.radius)
            || !self.strength.is_finite()
            || !(-1.0..=1.0).contains(&self.strength)
            || (!self.kind.invertible() && self.strength < 0.)
        {
            return Err(invalid(
                "Stroke requires1..128 samples, radius0.001..100 and strength[-1,1]; non-invertible brushes require nonnegative strength",
            ));
        }
        finite(&self.direction)?;
        for c in &self.centers {
            finite(c)?;
        }
        let direction = Vec3::from(self.direction).normalize_or_zero();
        if self.kind == BrushKind::Grab && direction == Vec3::ZERO {
            return Err(invalid("Grab requires a nonzero direction"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    PutObject {
        object: crate::mesh_objects::Object,
    },
    RemoveObject {
        id: String,
    },
    AssignObject {
        instance: String,
        object: Option<String>,
    },
    ModelSelection {
        asset: String,
        reference: crate::mesh_topology::Reference,
        operation: crate::mesh_modeling::Operation,
    },
    TransformVertices {
        asset: String,
        reference: crate::mesh_topology::Reference,
        vertices: Vec<u32>,
        transform: IslandTransform,
    },
    TransformIslands {
        asset: String,
        reference: crate::mesh_topology::Reference,
        islands: Vec<u32>,
        transform: IslandTransform,
    },
    SmoothRegion {
        asset: String,
        reference: crate::mesh_topology::Reference,
        vertices: Vec<u32>,
        anchors: Vec<u32>,
        options: crate::mesh_surface::Options,
    },
    RemoveFaces {
        asset: String,
        reference: crate::mesh_topology::Reference,
        faces: Vec<u32>,
    },
    BridgeBoundaryLoops {
        asset: String,
        reference: crate::mesh_topology::Reference,
        loops: [Vec<u32>; 2],
        spans: u8,
        handles: [f32; 2],
    },
    CopyMaterialCoordinates {
        asset: String,
        source: String,
        #[serde(default)]
        per_island: bool,
    },
    FitMaterialCoordinates {
        asset: String,
        #[serde(default)]
        per_island: bool,
        bounds: [[f32; 3]; 2],
    },
    ExtractIslands {
        asset: String,
        id: String,
        reference: crate::mesh_topology::Reference,
        islands: Vec<u32>,
        #[serde(default)]
        mode: ExtractionMode,
    },
    SetMaterialCoordinates {
        asset: String,
        coordinates: Option<Vec<[f32; 3]>>,
    },
    SetMaterial {
        asset: String,
        material: crate::materials::Material,
    },
    SetShading {
        asset: String,
        shading: Shading,
    },
    Subdivide {
        asset: String,
        reference: crate::mesh_topology::Reference,
        /// None refines the entire asset; explicit components split incident edges.
        components: Option<crate::mesh_topology::Components>,
    },
    PutAsset {
        asset: Asset,
    },
    Panel {
        id: String,
        dimensions: [f32; 3],
        #[serde(default)]
        openings: Vec<crate::mesh_panel::Opening>,
    },
    Grid {
        id: String,
        segments: u16,
        size: f32,
    },
    Sweep {
        id: String,
        sweeps: Vec<crate::mesh_sweep::Sweep>,
        #[serde(default)]
        append: bool,
    },
    ChamferBoxes {
        id: String,
        boxes: Vec<crate::mesh_chamfer_boxes::BoxSpec>,
        #[serde(default)]
        append: bool,
    },
    RemoveAsset {
        id: String,
    },
    Sculpt {
        asset: String,
        stroke: Stroke,
    },
    /// Interactive Draw uses relative strength/envelope; Grab limits safe travel.
    /// Optional detail consumes available budget without rejecting the stroke.
    SculptStroke {
        asset: String,
        reference: crate::mesh_topology::Reference,
        stroke: Stroke,
        adaptive_detail: bool,
        #[serde(default)]
        detail: crate::mesh_tools::SculptDetail,
    },
    SculptDetailed {
        asset: String,
        reference: crate::mesh_topology::Reference,
        stroke: Stroke,
        max_edge: f32,
    },
    SetRig {
        asset: String,
        rig: Option<Arc<Rig>>,
    },
    RetargetClip {
        asset: String,
        source: String,
        clip: String,
        output: String,
        mappings: Vec<crate::retarget::Mapping>,
    },
    PutClip {
        asset: String,
        clip: Arc<Clip>,
    },
    RemoveClip {
        asset: String,
        clip: String,
    },
    PutInstance {
        instance: Instance,
    },
    RemoveInstance {
        id: String,
    },
}
impl Edit {
    pub(crate) fn component_reference(&self) -> Option<(&str, &crate::mesh_topology::Reference)> {
        match self {
            Self::SculptStroke {
                asset, reference, ..
            }
            | Self::SculptDetailed {
                asset, reference, ..
            }
            | Self::ModelSelection {
                asset, reference, ..
            }
            | Self::TransformVertices {
                asset, reference, ..
            }
            | Self::TransformIslands {
                asset, reference, ..
            }
            | Self::SmoothRegion {
                asset, reference, ..
            } => Some((asset, reference)),
            Self::RemoveFaces {
                asset, reference, ..
            }
            | Self::BridgeBoundaryLoops {
                asset, reference, ..
            }
            | Self::Subdivide {
                asset, reference, ..
            }
            | Self::ExtractIslands {
                asset, reference, ..
            } => Some((asset, reference)),
            _ => None,
        }
    }
    pub(crate) fn changed_assets(&self) -> [Option<&str>; 2] {
        let changed: Option<&str> = match self {
            Self::ExtractIslands {
                asset, id, mode, ..
            } => return [(*mode == ExtractionMode::Move).then_some(asset), Some(id)],
            Self::PutAsset { asset } => Some(&asset.id),
            Self::Panel { id, .. }
            | Self::Grid { id, .. }
            | Self::Sweep { id, .. }
            | Self::ChamferBoxes { id, .. }
            | Self::RemoveAsset { id } => Some(id),
            Self::SculptStroke { asset, .. }
            | Self::SculptDetailed { asset, .. }
            | Self::ModelSelection { asset, .. }
            | Self::TransformVertices { asset, .. }
            | Self::TransformIslands { asset, .. }
            | Self::RemoveFaces { asset, .. }
            | Self::SmoothRegion { asset, .. }
            | Self::BridgeBoundaryLoops { asset, .. }
            | Self::CopyMaterialCoordinates { asset, .. }
            | Self::FitMaterialCoordinates { asset, .. }
            | Self::Subdivide { asset, .. }
            | Self::SetShading { asset, .. }
            | Self::SetMaterial { asset, .. }
            | Self::SetMaterialCoordinates { asset, .. }
            | Self::Sculpt { asset, .. }
            | Self::SetRig { asset, .. }
            | Self::RetargetClip { asset, .. }
            | Self::PutClip { asset, .. }
            | Self::RemoveClip { asset, .. } => Some(asset),
            Self::PutInstance { .. }
            | Self::RemoveInstance { .. }
            | Self::PutObject { .. }
            | Self::RemoveObject { .. }
            | Self::AssignObject { .. } => None,
        };
        [changed, None]
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    pub instance: String,
    pub clip_tick: Option<u32>,
    /// World-space posed vertices; bone matrices below are asset-space globals.
    pub positions: Vec<[f32; 3]>,
    pub joints: Vec<[[f32; 4]; 4]>,
    pub world_transform: [[f32; 4]; 4],
}
fn id(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid(
            "Mesh identifiers require 1..64 ASCII letters/digits/_/-",
        ));
    }
    Ok(())
}
fn finite(p: &[f32]) -> Result<()> {
    if p.iter().any(|v| !v.is_finite() || v.abs() > 10000.) {
        return Err(invalid("Mesh coordinates must be finite within +/-10000"));
    }
    Ok(())
}
fn quaternion(q: [f32; 4]) -> Result<()> {
    finite(&q)?;
    if (Quat::from_array(q).length_squared() - 1.).abs() > 0.0001 {
        return Err(invalid("Mesh rotations must be unit XYZW quaternions"));
    }
    Ok(())
}
fn transform(t: [f32; 3], r: [f32; 4]) -> Mat4 {
    Mat4::from_rotation_translation(Quat::from_array(r).normalize(), Vec3::from(t))
}
impl Geometry {
    pub fn validate(&self) -> Result<()> {
        if let Some(coordinates) = &self.material_coordinates {
            if coordinates.len() != self.positions.len() {
                return Err(invalid(
                    "Material coordinates require one entry per mesh vertex",
                ));
            }
            for p in coordinates {
                finite(p)?;
            }
        }
        if self.positions.len() < 3 || self.triangles.is_empty() {
            return Err(invalid("Mesh requires at least3 vertices and1 triangle"));
        }
        if self.positions.len() > u32::MAX as usize || self.triangles.len() > u32::MAX as usize {
            return Err(invalid("Mesh indices exceed the u32 geometry format"));
        }
        for p in &self.positions {
            finite(p)?;
        }
        for &[a, b, c] in &self.triangles {
            let n = self.positions.len() as u32;
            if a >= n || b >= n || c >= n || a == b || a == c || b == c {
                return Err(invalid("Mesh triangle has invalid/repeated vertex indices"));
            }
            let [a, b, c] = [a, b, c].map(|i| Vec3::from(self.positions[i as usize]));
            if (b - a).cross(c - a).length_squared() <= 0. {
                return Err(invalid("Mesh triangles must have nonzero area"));
            }
        }
        Ok(())
    }
    pub fn grid(segments: u16, size: f32) -> Result<Self> {
        if !(1..=63).contains(&segments) || !size.is_finite() || !(0.01..=100.).contains(&size) {
            return Err(invalid(
                "Grid requires segments1..63 and size0.01..100 metres",
            ));
        }
        let n = segments as u32;
        let mut positions = Vec::with_capacity(((n + 1) * (n + 1)) as usize);
        let mut triangles = Vec::with_capacity((n * n * 2) as usize);
        for z in 0..=n {
            for x in 0..=n {
                positions.push([
                    (x as f32 / n as f32 - 0.5) * size,
                    0.,
                    (z as f32 / n as f32 - 0.5) * size,
                ]);
            }
        }
        for z in 0..n {
            for x in 0..n {
                let a = z * (n + 1) + x;
                triangles.extend([[a, a + n + 1, a + 1], [a + 1, a + n + 1, a + n + 2]]);
            }
        }
        let g = Self {
            material_coordinates: None,
            positions,
            triangles,
        };
        g.validate()?;
        Ok(g)
    }
    pub(crate) fn sculpt(&mut self, s: &Stroke) -> Result<()> {
        s.validate()?;
        let direction = Vec3::from(s.direction).normalize_or_zero();
        if !matches!(
            s.kind,
            BrushKind::Draw | BrushKind::Grab | BrushKind::Smooth
        ) {
            return crate::mesh_sculpt_shapes::apply(self, s);
        }
        if s.kind == BrushKind::Draw {
            return crate::mesh_sculpt_shapes::draw(self, s);
        }
        // Grab must not publish a folded patch. Keep its original positions for
        // atomic rejection; Smooth retains its existing sampling behavior.
        let original = (s.kind == BrushKind::Grab).then(|| self.positions.clone());
        // Each sample reads a frozen predecessor. Work is O(samples*(vertices+triangles)).
        let mut vectors = vec![Vec3::ZERO; self.positions.len()];
        let mut counts = vec![0u32; self.positions.len()];
        for center in &s.centers {
            vectors[..self.positions.len()].fill(Vec3::ZERO);
            counts[..self.positions.len()].fill(0);
            if s.kind != BrushKind::Grab {
                for &[a, b, c] in &self.triangles {
                    let [ia, ib, ic] = [a as usize, b as usize, c as usize];
                    let [a, b, c] = [ia, ib, ic].map(|i| Vec3::from(self.positions[i]));
                    if s.kind == BrushKind::Draw {
                        let normal = (b - a).cross(c - a);
                        for i in [ia, ib, ic] {
                            vectors[i] += normal;
                        }
                    } else {
                        for (i, sum) in [(ia, b + c), (ib, a + c), (ic, a + b)] {
                            vectors[i] += sum;
                            counts[i] += 2;
                        }
                    }
                }
            }
            for (i, p) in self.positions.iter_mut().enumerate() {
                let point = Vec3::from(*p);
                let d = point.distance(Vec3::from(*center)) / s.radius;
                if d >= 1. {
                    continue;
                }
                let falloff = (1. - d * d).powi(2);
                let delta = match s.kind {
                    BrushKind::Grab => direction,
                    BrushKind::Draw => vectors[i].normalize_or_zero(),
                    BrushKind::Smooth if counts[i] > 0 => vectors[i] / counts[i] as f32 - point,
                    BrushKind::Smooth => Vec3::ZERO,
                    _ => unreachable!("shaping brushes handled above"),
                };
                *p = (point + delta * (s.strength * falloff)).to_array();
            }
        }
        if let Some(original) = original {
            let result = crate::mesh_sculpt_shapes::check_orientation(
                &self.triangles,
                &original,
                &self.positions,
            )
            .and_then(|()| self.validate());
            if result.is_err() {
                self.positions = original;
            }
            result
        } else {
            self.validate()
        }
    }
}
impl Rig {
    fn validate(&self, vertices: usize) -> Result<()> {
        if self.bones.is_empty() || self.bones.len() > MAX_BONES || self.weights.len() != vertices {
            return Err(invalid(
                "Rig requires1..256 bones and one weight record per vertex (up to16 influences)",
            ));
        }
        let mut names = BTreeSet::new();
        for (i, b) in self.bones.iter().enumerate() {
            id(&b.name)?;
            if !names.insert(&b.name) || b.parent.is_some_and(|p| p as usize >= i) {
                return Err(invalid(
                    "Rig names must be unique; each parent must precede its child",
                ));
            }
            finite(&b.translation)?;
            quaternion(b.rotation)?;
        }
        for w in &self.weights {
            if w.extra
                .as_ref()
                .is_some_and(|v| v.is_empty() || v.len() > MAX_INFLUENCES - 4)
                || w.influences().any(|(j, v)| {
                    !v.is_finite() || !(0.0..=1.0).contains(&v) || j as usize >= self.bones.len()
                })
                || (w.influences().map(|(_, v)| v).sum::<f32>() - 1.).abs() > 0.0001
            {
                return Err(invalid(
                    "Rig weights require valid joints, nonnegative finite values summing to1",
                ));
            }
            let mut seen = [false; MAX_BONES];
            for (joint, weight) in w.influences() {
                if weight > 0. {
                    if seen[joint as usize] {
                        return Err(invalid("Duplicate nonzero joint influence"));
                    }
                    seen[joint as usize] = true;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn bind_globals(&self) -> [Mat4; MAX_BONES] {
        let mut out = [Mat4::IDENTITY; MAX_BONES];
        for (i, b) in self.bones.iter().enumerate() {
            let local = transform(b.translation, b.rotation);
            out[i] = b.parent.map_or(local, |p| out[p as usize] * local);
        }
        out
    }
}
impl Clip {
    fn validate(&self, rig: &Rig) -> Result<()> {
        id(&self.id)?;
        if !(1..=MAX_TICK).contains(&self.duration_ticks)
            || self.tracks.is_empty()
            || self.tracks.len() > rig.bones.len()
        {
            return Err(invalid(
                "Clip requires1..36000 duration ticks and nonempty tracks within rig size",
            ));
        }
        let mut bones = BTreeSet::new();
        let mut count = 0;
        for t in &self.tracks {
            count += t.keys.len();
            if t.bone as usize >= rig.bones.len()
                || !bones.insert(t.bone)
                || t.keys.is_empty()
                || count > MAX_KEYS
            {
                return Err(invalid(
                    "Clip tracks require unique valid bones and1..4096 total keys",
                ));
            }
            let mut previous = None;
            for k in &t.keys {
                if k.tick > self.duration_ticks || previous.is_some_and(|p| k.tick <= p) {
                    return Err(invalid(
                        "Keys must have strictly increasing ticks within clip duration",
                    ));
                }
                finite(&k.translation)?;
                quaternion(k.rotation)?;
                previous = Some(k.tick);
            }
        }
        Ok(())
    }
}
impl Asset {
    pub(crate) fn validate(&self) -> Result<()> {
        self.material.validate()?;
        id(&self.id)?;
        self.geometry.validate()?;
        if let Some(r) = &self.rig {
            r.validate(self.geometry.positions.len())?;
        }
        if self.clips.len() > MAX_CLIPS || (!self.clips.is_empty() && self.rig.is_none()) {
            return Err(invalid("Clips require a rig; maximum16 clips"));
        }
        let mut ids = BTreeSet::new();
        for c in &self.clips {
            if !ids.insert(&c.id) {
                return Err(invalid("Duplicate clip ID"));
            }
            c.validate(self.rig.as_ref().expect("rig checked"))?;
        }
        Ok(())
    }
}
impl Document {
    /// Preflight the index format before allocating replacement vertices/weights.
    pub(crate) fn check_replacement_counts(
        &self,
        id: &str,
        vertices: usize,
        triangles: usize,
    ) -> Result<()> {
        let _ = id;
        if vertices > u32::MAX as usize || triangles > u32::MAX as usize {
            return Err(invalid("Mesh indices exceed the u32 geometry format"));
        }
        Ok(())
    }
    pub(crate) fn exceeds_legacy_limits(&self) -> bool {
        self.assets.len() > 16
            || self.instances.len() > 32
            || self
                .assets
                .iter()
                .any(|a| a.geometry.positions.len() > 16384 || a.geometry.triangles.len() > 32768)
            || self
                .assets
                .iter()
                .map(|a| a.geometry.positions.len())
                .sum::<usize>()
                > 32768
            || self
                .assets
                .iter()
                .map(|a| a.geometry.triangles.len())
                .sum::<usize>()
                > 65536
            || self
                .instances
                .iter()
                .filter_map(|i| self.assets.iter().find(|a| a.id == i.asset))
                .map(|a| a.geometry.triangles.len())
                .sum::<usize>()
                > 131072
    }
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty() && self.instances.is_empty() && self.objects.is_empty()
    }
    pub fn validate(&self) -> Result<()> {
        if self.assets.len() >= u32::MAX as usize {
            return Err(invalid("Material indices exceed the u32 GPU format"));
        }
        let mut ids = BTreeSet::new();
        let (mut keys, mut clips) = (0, 0);
        for a in &self.assets {
            if !ids.insert(&a.id) {
                return Err(invalid("Duplicate mesh asset ID"));
            }
            a.validate()?;
            clips += a.clips.len();
            keys += a
                .clips
                .iter()
                .flat_map(|c| &c.tracks)
                .map(|t| t.keys.len())
                .sum::<usize>();
        }
        if clips > MAX_CLIPS || keys > MAX_KEYS {
            return Err(invalid(
                "Mesh document exceeds vertex/triangle/clip/key budget",
            ));
        }
        ids.clear();
        for i in &self.instances {
            id(&i.id)?;
            if !ids.insert(&i.id) {
                return Err(invalid("Duplicate mesh instance ID"));
            }
            let a = self.asset(&i.asset)?;
            finite(&i.position)?;
            quaternion(i.rotation)?;
            if !i.scale.is_finite()
                || !(0.001..=100.).contains(&i.scale)
                || i.color
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err(invalid(
                    "Instance requires positive uniform scale0.001..100 and linear RGB[0,1]",
                ));
            }
            if let Some(p) = &i.playback
                && (p.offset_ticks > MAX_TICK || !a.clips.iter().any(|c| c.id == p.clip))
            {
                return Err(invalid(
                    "Playback requires an existing clip and offset<=36000",
                ));
            }
        }
        Hierarchy::new(self)?;
        crate::mesh_objects::validate(self)?;
        Ok(())
    }
    pub fn asset(&self, name: &str) -> Result<&Asset> {
        self.assets
            .iter()
            .find(|a| a.id == name)
            .ok_or_else(|| invalid(format!("Mesh asset {name} not found")))
    }
    fn asset_mut(&mut self, name: &str) -> Result<&mut Asset> {
        self.assets
            .iter_mut()
            .find(|a| a.id == name)
            .ok_or_else(|| invalid(format!("Mesh asset {name} not found")))
    }
    pub(crate) fn model_selection(
        &mut self,
        asset: &str,
        operation: &crate::mesh_modeling::Operation,
    ) -> Result<crate::mesh_modeling::Report> {
        let source = self.asset(asset)?;
        let (source_v, source_t) = (
            source.geometry.positions.len(),
            source.geometry.triangles.len(),
        );
        self.check_replacement_counts(asset, source_v, source_t)?;
        let bound = |mut lo: usize, mut hi: usize, vertices: bool| {
            while lo < hi {
                let mid = lo + (hi - lo).div_ceil(2);
                let (v, t) = if vertices {
                    (mid, source_t)
                } else {
                    (source_v, mid)
                };
                if self.check_replacement_counts(asset, v, t).is_ok() {
                    lo = mid;
                } else {
                    hi = mid - 1;
                }
            }
            lo
        };
        let limits = crate::mesh_modeling::Limits {
            vertices: bound(source_v, MAX_VERTICES, true),
            triangles: bound(source_t, MAX_TRIANGLES, false),
        };
        let result = crate::mesh_modeling::evaluate_limited(source, operation, limits)?;
        self.check_replacement_counts(asset, result.report.vertices, result.report.triangles)?;
        *self.asset_mut(asset)? = result.asset;
        Ok(result.report)
    }
    /// Mutates only the engine's staged document. The engine validates the whole transaction before publishing.
    pub(crate) fn apply(&mut self, edit: Edit) -> Result<()> {
        match edit {
            Edit::ModelSelection {
                asset, operation, ..
            } => {
                self.model_selection(&asset, &operation)?;
            }
            Edit::TransformVertices {
                asset,
                vertices,
                transform,
                ..
            } => self.transform_vertices(&asset, &vertices, &transform)?,
            Edit::TransformIslands {
                asset,
                islands,
                transform,
                ..
            } => {
                self.transform_islands(&asset, &islands, &transform)?;
            }
            Edit::RemoveFaces { asset, faces, .. } => {
                let result = crate::mesh_bridge::remove_faces(self.asset(&asset)?, &faces)?;
                *self.asset_mut(&asset)? = result;
            }
            Edit::BridgeBoundaryLoops {
                asset,
                loops,
                spans,
                handles,
                ..
            } => {
                let source = self.asset(&asset)?;
                let (v, t) = crate::mesh_bridge::counts(source, &loops, spans)?;
                self.check_replacement_counts(&asset, v, t)?;
                let result =
                    crate::mesh_bridge::bridge(self.asset(&asset)?, &loops, spans, handles)?;
                *self.asset_mut(&asset)? = result;
            }
            Edit::CopyMaterialCoordinates {
                asset,
                source,
                per_island,
            } => self.copy_material_coordinates(&asset, &source, per_island)?,
            Edit::FitMaterialCoordinates {
                asset,
                per_island,
                bounds,
            } => self.fit_material_coordinates(&asset, per_island, bounds)?,
            Edit::ExtractIslands {
                asset,
                id,
                islands,
                mode,
                ..
            } => self.extract(&asset, &id, &islands, mode)?,
            Edit::SetMaterialCoordinates { asset, coordinates } => {
                Arc::make_mut(&mut self.asset_mut(&asset)?.geometry).material_coordinates =
                    coordinates;
            }
            Edit::SetMaterial { asset, material } => self.asset_mut(&asset)?.material = material,
            Edit::SetShading { asset, shading } => self.asset_mut(&asset)?.shading = shading,
            Edit::Subdivide {
                asset, components, ..
            } => {
                let source = self.asset(&asset)?;
                let plan = crate::mesh_topology::Plan::new(&source.geometry, components.as_ref())?;
                self.check_replacement_counts(&asset, plan.vertex_count(), plan.triangles().len())?;
                let subdivided = plan.asset(source)?;
                *self.asset_mut(&asset)? = subdivided;
            }
            Edit::PutAsset { asset } => {
                asset.validate()?;
                if let Some(a) = self.assets.iter_mut().find(|a| a.id == asset.id) {
                    *a = asset;
                } else {
                    self.assets.push(asset);
                }
            }
            Edit::Panel {
                id,
                dimensions,
                openings,
            } => self.apply(Edit::PutAsset {
                asset: Asset {
                    id,
                    geometry: Arc::new(crate::mesh_panel::build(dimensions, &openings)?),
                    shading: Shading::Flat,
                    material: Default::default(),
                    rig: None,
                    clips: vec![],
                },
            })?,
            Edit::Grid { id, segments, size } => self.apply(Edit::PutAsset {
                asset: Asset {
                    id,
                    geometry: Arc::new(Geometry::grid(segments, size)?),
                    shading: Shading::Flat,
                    material: Default::default(),
                    rig: None,
                    clips: vec![],
                },
            })?,
            Edit::Sweep { id, sweeps, append } => {
                let plan = crate::mesh_sweep::Plan::new(&sweeps)?;
                let source = if append { Some(self.asset(&id)?) } else { None };
                if source.is_some_and(|a| a.rig.is_some() || !a.clips.is_empty()) {
                    return Err(invalid(
                        "Sweep append requires an unrigged asset; author new skin explicitly",
                    ));
                }
                let previous_vertices = source.map_or(0, |a| a.geometry.positions.len());
                let previous_triangles = source.map_or(0, |a| a.geometry.triangles.len());
                self.check_replacement_counts(
                    &id,
                    previous_vertices + plan.vertices,
                    previous_triangles + plan.triangles,
                )?;
                let asset = if let Some(source) = source {
                    Asset {
                        geometry: Arc::new(plan.append_to((*source.geometry).clone())?),
                        ..source.clone()
                    }
                } else {
                    Asset {
                        id,
                        geometry: Arc::new(plan.generate()?),
                        shading: Shading::Flat,
                        material: Default::default(),
                        rig: None,
                        clips: vec![],
                    }
                };
                self.apply(Edit::PutAsset { asset })?;
            }
            Edit::ChamferBoxes { id, boxes, append } => {
                let plan = crate::mesh_chamfer_boxes::Plan::new(&boxes)?;
                let source = if append { Some(self.asset(&id)?) } else { None };
                if source.is_some_and(|a| a.rig.is_some() || !a.clips.is_empty()) {
                    return Err(invalid("Chamfer boxes append requires an unrigged asset"));
                }
                self.check_replacement_counts(
                    &id,
                    source.map_or(0, |a| a.geometry.positions.len()) + plan.vertices,
                    source.map_or(0, |a| a.geometry.triangles.len()) + plan.triangles,
                )?;
                let asset = if let Some(source) = source {
                    Asset {
                        geometry: Arc::new(plan.append_to((*source.geometry).clone())?),
                        ..source.clone()
                    }
                } else {
                    Asset {
                        id,
                        geometry: Arc::new(plan.generate()?),
                        shading: Shading::Flat,
                        material: Default::default(),
                        rig: None,
                        clips: vec![],
                    }
                };
                self.apply(Edit::PutAsset { asset })?;
            }
            Edit::RemoveAsset { id } => {
                let index = self
                    .assets
                    .iter()
                    .position(|a| a.id == id)
                    .ok_or_else(|| invalid("Mesh asset not found"))?;
                self.assets.remove(index);
            }
            Edit::SculptStroke {
                asset,
                stroke,
                adaptive_detail,
                detail,
                ..
            } => {
                let (result, _) = crate::mesh_sculpt_shapes::interactive(
                    self.asset(&asset)?,
                    &stroke,
                    adaptive_detail,
                    detail,
                    |v, t| self.check_replacement_counts(&asset, v, t),
                )?;
                *self.asset_mut(&asset)? = result;
            }
            Edit::SculptDetailed {
                asset,
                stroke,
                max_edge,
                ..
            } => {
                let result = crate::mesh_sculpt_shapes::detailed(
                    self.asset(&asset)?,
                    &stroke,
                    max_edge,
                    |v, t| self.check_replacement_counts(&asset, v, t),
                )?;
                *self.asset_mut(&asset)? = result;
            }
            Edit::Sculpt { asset, stroke } => {
                Arc::make_mut(&mut self.asset_mut(&asset)?.geometry).sculpt(&stroke)?
            }
            Edit::SmoothRegion {
                asset,
                vertices,
                anchors,
                options,
                ..
            } => {
                let filtered = crate::mesh_surface::smooth_region(
                    self.asset(&asset)?,
                    &vertices,
                    &anchors,
                    &options,
                )?;
                *self.asset_mut(&asset)? = filtered.asset;
            }
            Edit::SetRig { asset, rig } => self.asset_mut(&asset)?.rig = rig,
            Edit::RetargetClip {
                asset,
                source,
                clip,
                output,
                mappings,
            } => {
                let source = self
                    .assets
                    .iter()
                    .find(|a| a.id == source)
                    .ok_or_else(|| invalid("Unknown retarget source asset"))?;
                let target = self
                    .assets
                    .iter()
                    .find(|a| a.id == asset)
                    .ok_or_else(|| invalid("Unknown retarget target asset"))?;
                let clip = crate::retarget::bake(source, target, &clip, output, &mappings)?;
                self.apply(Edit::PutClip {
                    asset,
                    clip: Arc::new(clip),
                })?;
            }
            Edit::PutClip { asset, clip } => {
                let a = self.asset_mut(&asset)?;
                if let Some(c) = a.clips.iter_mut().find(|c| c.id == clip.id) {
                    *c = clip;
                } else {
                    a.clips.push(clip);
                }
            }
            Edit::RemoveClip { asset, clip } => {
                let a = self.asset_mut(&asset)?;
                let index = a
                    .clips
                    .iter()
                    .position(|c| c.id == clip)
                    .ok_or_else(|| invalid("Mesh clip not found"))?;
                a.clips.remove(index);
            }
            Edit::PutObject { object } => crate::mesh_objects::put(self, object)?,
            Edit::RemoveObject { id } => crate::mesh_objects::remove(self, &id)?,
            Edit::AssignObject { instance, object } => {
                crate::mesh_objects::assign(self, &instance, object)?
            }
            Edit::PutInstance { instance } => {
                if let Some(i) = self.instances.iter_mut().find(|i| i.id == instance.id) {
                    *i = instance;
                } else {
                    self.instances.push(instance);
                }
            }
            Edit::RemoveInstance { id } => {
                let index = self
                    .instances
                    .iter()
                    .position(|i| i.id == id)
                    .ok_or_else(|| invalid("Mesh instance not found"))?;
                self.instances.remove(index);
                crate::mesh_objects::forget(self, &id);
            }
        }
        Ok(())
    }
    pub fn pose(&self, instance: &str, tick: u64) -> Result<Pose> {
        let index = self
            .instances
            .iter()
            .position(|i| i.id == instance)
            .ok_or_else(|| invalid("Mesh instance not found"))?;
        let hierarchy = Hierarchy::new(self)?;
        let prepared = Prepared::new(self, &hierarchy, tick);
        self.prepared_pose(index, &prepared)
    }
    fn prepared_pose(&self, index: usize, prepared: &Prepared) -> Result<Pose> {
        let i = &self.instances[index];
        let a = self.asset(&i.asset)?;
        let mut positions = Vec::new();
        let skeleton = prepared.skeleton(index);
        evaluate(a, skeleton, prepared.worlds[index], &mut positions);
        Ok(Pose {
            instance: i.id.clone(),
            clip_tick: skeleton.clip_tick,
            positions: positions[..a.geometry.positions.len()].to_vec(),
            joints: skeleton.posed[..a.rig.as_ref().map_or(0, |r| r.bones.len())]
                .iter()
                .map(Mat4::to_cols_array_2d)
                .collect(),
            world_transform: prepared.worlds[index].to_cols_array_2d(),
        })
    }
    pub fn poses(&self, tick: u64) -> Vec<Pose> {
        let hierarchy = Hierarchy::new(self).expect("validated mesh hierarchy");
        let prepared = Prepared::new(self, &hierarchy, tick);
        self.instances
            .iter()
            .enumerate()
            .map(|(index, _)| {
                self.prepared_pose(index, &prepared)
                    .expect("validated mesh document")
            })
            .collect()
    }
    fn append_vertices(
        &self,
        tick: u64,
        hierarchy: &Hierarchy,
        projection_cache: &crate::mesh_projection::Cache,
        streams: (
            &mut Vec<crate::render::Vertex>,
            &mut Vec<crate::render::Vertex>,
            &mut Vec<crate::distant::selection::Draw>,
        ),
        (consumed, unique_static_residency): (&[String], bool),
    ) {
        let (out, shared_vertices, draws) = streams;
        let prepared = Prepared::new(self, hierarchy, tick);
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut counts = vec![0usize; self.assets.len()];
        let consumed: BTreeSet<_> = consumed.iter().map(String::as_str).collect();
        for (index, i) in self.instances.iter().enumerate() {
            if !consumed.contains(i.id.as_str()) && i.attachment.is_none() {
                counts[hierarchy.assets[index]] += 1;
            }
        }
        let mut ranges: Vec<Option<std::ops::Range<usize>>> = vec![None; self.assets.len()];
        for (index, i) in self.instances.iter().enumerate() {
            if consumed.contains(i.id.as_str()) {
                continue;
            }
            let slot = hierarchy.assets[index];
            let asset = &self.assets[slot];
            // Large static assets also benefit from resident local geometry when
            // used only once. Small unique draws retain the inexpensive legacy
            // path; this is a preparation heuristic, never an authoring limit.
            let shared = (counts[slot] > 1
                || (unique_static_residency && asset.geometry.triangles.len() >= 1024))
                && asset.rig.is_none()
                && asset.clips.is_empty()
                && i.playback.is_none()
                && i.attachment.is_none();
            if shared && let Some(range) = &ranges[slot] {
                draws.push(crate::distant::selection::Draw {
                    instance: i.id.clone(),
                    range: range.clone(),
                    asset: slot,
                    world: prepared.worlds[index],
                    color: i.color,
                    eligible: true,
                    shared,
                    repeated: counts[slot] > 1,
                });
                continue;
            }
            let out = if shared {
                &mut *shared_vertices
            } else {
                &mut *out
            };
            let start = out.len();
            evaluate(
                asset,
                prepared.skeleton(index),
                if shared {
                    Mat4::IDENTITY
                } else {
                    prepared.worlds[index]
                },
                &mut positions,
            );
            let smooth = asset.shading == Shading::Smooth;
            let projection_weights = projection_cache.get(&asset.geometry);
            normals.resize(asset.geometry.positions.len(), Vec3::ZERO);
            if smooth {
                crate::mesh_normals::accumulate(
                    &asset.geometry.triangles,
                    &positions[..asset.geometry.positions.len()],
                    &mut normals[..asset.geometry.positions.len()],
                );
            }
            for triangle in &asset.geometry.triangles {
                let [a, b, c] = triangle.map(|index| Vec3::from(positions[index as usize]));
                let face = crate::mesh_normals::face([a, b, c]).0;
                for (&index, p) in triangle.iter().zip([a, b, c]) {
                    let normal = crate::mesh_normals::corner(smooth, normals[index as usize], face);
                    out.push(crate::render::Vertex {
                        position: p.to_array(),
                        normal: normal.to_array(),
                        color: if shared { [1.; 3] } else { i.color },
                        rest_position: asset
                            .geometry
                            .material_coordinates
                            .as_ref()
                            .unwrap_or(&asset.geometry.positions)[index as usize],
                        material: (slot + 1) as u32,
                        material_projection_weights: projection_weights[index as usize],
                    });
                }
            }
            draws.push(crate::distant::selection::Draw {
                instance: i.id.clone(),
                range: start..out.len(),
                asset: slot,
                world: prepared.worlds[index],
                color: i.color,
                shared,
                repeated: counts[slot] > 1,
                eligible: asset.rig.is_none()
                    && asset.clips.is_empty()
                    && i.playback.is_none()
                    && i.attachment.is_none(),
            });
            if shared {
                ranges[slot] = Some(start..out.len());
            }
        }
    }
}
pub(crate) fn sample(track: &Track, tick: u32) -> Mat4 {
    let upper = track.keys.partition_point(|k| k.tick <= tick);
    if upper == 0 {
        let k = &track.keys[0];
        return transform(k.translation, k.rotation);
    }
    let a = &track.keys[upper - 1];
    if upper == track.keys.len() {
        return transform(a.translation, a.rotation);
    }
    let b = &track.keys[upper];
    let f = (tick - a.tick) as f32 / (b.tick - a.tick) as f32;
    Mat4::from_rotation_translation(
        Quat::from_array(a.rotation)
            .normalize()
            .slerp(Quat::from_array(b.rotation).normalize(), f)
            .normalize(),
        Vec3::from(a.translation).lerp(Vec3::from(b.translation), f),
    )
}
#[derive(Clone)]
struct Skeleton {
    clip_tick: Option<u32>,
    posed: Vec<Mat4>,
    skin: Vec<Mat4>,
}
fn skeleton(asset: &Asset, instance: &Instance, tick: u64) -> Skeleton {
    let playback = instance.playback.as_ref().map(|p| {
        let clip = asset
            .clips
            .iter()
            .find(|c| c.id == p.clip)
            .expect("validated clip");
        let elapsed = tick.saturating_add(p.offset_ticks as u64);
        let t = if p.looping {
            elapsed % clip.duration_ticks as u64
        } else {
            elapsed.min(clip.duration_ticks as u64)
        } as u32;
        (clip, t)
    });
    let count = asset.rig.as_ref().map_or(0, |r| r.bones.len());
    let mut posed = vec![Mat4::IDENTITY; count];
    let mut skin = vec![Mat4::IDENTITY; count];
    if let Some(r) = &asset.rig {
        let bind = r.bind_globals();
        for (i, b) in r.bones.iter().enumerate() {
            let local = playback
                .as_ref()
                .and_then(|(c, t)| {
                    c.tracks
                        .iter()
                        .find(|tr| tr.bone as usize == i)
                        .map(|tr| sample(tr, *t))
                })
                .unwrap_or_else(|| transform(b.translation, b.rotation));
            posed[i] = b.parent.map_or(local, |p| posed[p as usize] * local);
            skin[i] = posed[i] * bind[i].inverse();
        }
    }
    Skeleton {
        clip_tick: playback.map(|(_, t)| t),
        posed,
        skin,
    }
}
fn evaluate(asset: &Asset, skeleton: &Skeleton, world: Mat4, positions: &mut Vec<[f32; 3]>) {
    positions.resize(asset.geometry.positions.len(), [0.; 3]);
    for (index, p) in asset.geometry.positions.iter().enumerate() {
        let p = Vec3::from(*p);
        let local = asset.rig.as_ref().map_or(p, |r| {
            let w = &r.weights[index];
            let mut v = Vec3::ZERO;
            let sum = w.influences().map(|(_, w)| w).sum::<f32>();
            for (joint, weight) in w.influences() {
                if weight > 0. {
                    v += skeleton.skin[joint as usize].transform_point3(p) * (weight / sum);
                }
            }
            v
        });
        positions[index] = world.transform_point3(local).to_array();
    }
}

/// Reusable frame preparation; a paused/static document does not redo skinning.
fn next_frame_generation() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
#[derive(Default)]
pub struct Frame {
    /// Changes only when authored source/masks change, independent of avatar tails.
    pub static_generation: u64,
    pub(crate) generation: u64,
    /// Shared vertex payload/layout changes, independent of static placements.
    pub(crate) geometry_generation: u64,
    pub(crate) normal_overrides: Vec<Option<Arc<Vec<[f32; 3]>>>>,
    pub(crate) draws: Vec<crate::distant::selection::Draw>,
    base_vertices: usize,
    pub omitted_avatars: u32,
    pub drawn_avatars: Vec<u64>,
    pub(crate) source: Document,
    tick: Option<u64>,
    pub vertices: Vec<crate::render::Vertex>,
    pub(crate) shared_vertices: Vec<crate::render::Vertex>,
    pub materials: Vec<crate::materials::Material>,
    hierarchy: Hierarchy,
    projection_cache: crate::mesh_projection::Cache,
    consumed: Vec<String>,
    legacy_unique_static: bool,
}
impl Frame {
    /// Diagnostic oracle for the previous single-instance upload path. This
    /// changes derived render data only, never the document or physics.
    pub fn set_unique_static_residency(&mut self, enabled: bool) {
        if self.legacy_unique_static == enabled {
            self.legacy_unique_static = !enabled;
            self.tick = None;
            self.static_generation = next_frame_generation();
        }
    }
    pub fn triangle_count(&self) -> usize {
        self.vertices.len() / 3
            + self
                .draws
                .iter()
                .filter(|d| d.shared)
                .map(|d| d.range.len() / 3)
                .sum::<usize>()
    }
    pub fn resident_vertex_count(&self) -> usize {
        self.vertices.len() + self.shared_vertices.len()
    }
    pub(crate) fn draw_positions(
        &self,
        draw: &crate::distant::selection::Draw,
    ) -> impl Iterator<Item = [f32; 3]> + '_ {
        let shared = draw.shared;
        let world = draw.world;
        let vertices = if shared {
            &self.shared_vertices
        } else {
            &self.vertices
        };
        vertices[draw.range.clone()].iter().map(move |v| {
            if shared {
                world.transform_point3(v.position.into()).to_array()
            } else {
                v.position
            }
        })
    }
    pub fn update(&mut self, document: &Document, tick: u64) {
        self.update_filtered(document, tick, std::iter::empty());
    }
    pub fn update_filtered<'a>(
        &mut self,
        document: &Document,
        tick: u64,
        consumed: impl Iterator<Item = &'a str> + Clone,
    ) {
        if self.vertices.len() != self.base_vertices {
            self.generation = next_frame_generation();
        }
        self.vertices.truncate(self.base_vertices);
        self.omitted_avatars = 0;
        self.drawn_avatars.clear();
        let effective_tick = if document.instances.iter().any(|i| i.playback.is_some()) {
            tick
        } else {
            0
        };
        let mask_changed = !self
            .consumed
            .iter()
            .map(String::as_str)
            .eq(consumed.clone());
        // Immutable resident assets with stable membership: a rebase/color/pose edit
        // changes transforms, not evaluation, normals or the concatenated streams.
        if self.tick == Some(0)
            && effective_tick == 0
            && !mask_changed
            && self.vertices.is_empty()
            && self.draws.len() == document.instances.len()
            && self.draws.iter().all(|d| d.shared)
            && self.source.objects == document.objects
            && self.source.assets.len() == document.assets.len()
            && self
                .source
                .assets
                .iter()
                .zip(&document.assets)
                .all(|(a, b)| {
                    a.id == b.id
                        && Arc::ptr_eq(&a.geometry, &b.geometry)
                        && a.shading == b.shading
                        && a.material == b.material
                        && a.rig.is_none()
                        && b.rig.is_none()
                        && a.clips.is_empty()
                        && b.clips.is_empty()
                })
            && self.source.instances.len() == document.instances.len()
            && self
                .source
                .instances
                .iter()
                .zip(&document.instances)
                .all(|(a, b)| {
                    a.id == b.id
                        && a.asset == b.asset
                        && a.attachment.is_none()
                        && b.attachment.is_none()
                        && a.playback.is_none()
                        && b.playback.is_none()
                })
        {
            if self.source.instances != document.instances {
                for (draw, i) in self.draws.iter_mut().zip(&document.instances) {
                    draw.world =
                        transform(i.position, i.rotation) * Mat4::from_scale(Vec3::splat(i.scale));
                    draw.color = i.color;
                }
                self.source.instances.clone_from(&document.instances);
                self.generation = next_frame_generation();
                // Object bounds are world-space: a moved object must still invalidate them.
                self.static_generation = self.generation;
            }
            return;
        }
        let changed = self.source != *document;
        if self.tick == Some(effective_tick) && !changed && !mask_changed {
            return;
        }
        if mask_changed {
            self.consumed.clear();
            self.consumed.extend(consumed.map(str::to_owned));
        }
        self.generation = next_frame_generation();
        self.geometry_generation = self.generation;
        self.normal_overrides.clear();
        if changed || mask_changed {
            self.static_generation = self.generation;
        }
        self.vertices.clear();
        self.shared_vertices.clear();
        self.draws.clear();
        self.projection_cache.prepare(&document.assets);
        if changed {
            self.hierarchy = Hierarchy::new(document).expect("validated mesh hierarchy");
        }
        document.append_vertices(
            effective_tick,
            &self.hierarchy,
            &self.projection_cache,
            (
                &mut self.vertices,
                &mut self.shared_vertices,
                &mut self.draws,
            ),
            (&self.consumed, !self.legacy_unique_static),
        );
        if changed {
            self.source.clone_from(document);
            self.materials.clear();
            self.materials.push(Default::default());
            self.materials
                .extend(document.assets.iter().map(|a| a.material.clone()));
        }
        self.base_vertices = self.vertices.len();
        self.tick = Some(effective_tick);
    }
}

impl Frame {
    /// Derived immutable vertex normals, e.g. a continuous procedural surface.
    /// Kept separate from authored geometry and part of GPU payload identity.
    pub(crate) fn apply_smooth_normals(&mut self, normals: &[Arc<Vec<[f32; 3]>>]) -> Result<()> {
        if self.normal_overrides.len() == normals.len()
            && self
                .normal_overrides
                .iter()
                .zip(normals)
                .all(|(a, b)| a.as_ref().is_some_and(|a| Arc::ptr_eq(a, b)))
        {
            return Ok(());
        }
        if normals.len() != self.source.assets.len()
            || self.draws.iter().any(|d| !d.shared)
            || self.source.assets.iter().zip(normals).any(|(a, n)| {
                n.len() != a.geometry.positions.len()
                    || n.iter().any(|v| {
                        !Vec3::from(*v).is_finite()
                            || (Vec3::from(*v).length_squared() - 1.).abs() > 0.001
                    })
            })
        {
            return Err(invalid(
                "Derived normals require matching unit normals for shared static assets",
            ));
        }
        let mut done = vec![false; normals.len()];
        for draw in &self.draws {
            if done[draw.asset] {
                continue;
            }
            done[draw.asset] = true;
            for (v, index) in self.shared_vertices[draw.range.clone()].iter_mut().zip(
                self.source.assets[draw.asset]
                    .geometry
                    .triangles
                    .iter()
                    .flatten(),
            ) {
                v.normal = normals[draw.asset][*index as usize];
            }
        }
        self.normal_overrides = normals.iter().map(|n| Some(n.clone())).collect();
        self.generation = next_frame_generation();
        self.geometry_generation = self.generation;
        Ok(())
    }
    /// Append derived actor instances without enlarging the authored hierarchy.
    pub fn append_avatars(
        &mut self,
        document: &Document,
        poses: &[crate::npc_presentation::Pose],
        position: impl Fn(u64) -> Option<([f32; 3], [f32; 3])>,
    ) -> Result<()> {
        if !poses.is_empty() {
            self.generation = next_frame_generation();
        }
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        for p in poses {
            p.validate()?;
            let asset = document
                .assets
                .get(p.asset as usize)
                .ok_or_else(|| invalid("Avatar asset is absent in visual content"))?;
            p.validate_asset(asset)?;
            let Some((anchor, color)) = position(p.actor) else {
                return Err(invalid("Avatar actor absent"));
            };
            let world = avatar_world(p, anchor);
            let sk = avatar_skeleton(asset, p);
            evaluate(asset, &sk, world, &mut positions);
            let smooth = asset.shading == Shading::Smooth;
            normals.resize(asset.geometry.positions.len(), Vec3::ZERO);
            if smooth {
                crate::mesh_normals::accumulate(
                    &asset.geometry.triangles,
                    &positions[..asset.geometry.positions.len()],
                    &mut normals[..asset.geometry.positions.len()],
                );
            }
            let weights = self.projection_cache.get(&asset.geometry);
            for tri in &asset.geometry.triangles {
                let corners = tri.map(|i| Vec3::from(positions[i as usize]));
                let face = crate::mesh_normals::face(corners).0;
                for (index, point) in tri.iter().zip(corners) {
                    let index = *index as usize;
                    self.vertices.push(crate::render::Vertex {
                        position: point.to_array(),
                        normal: crate::mesh_normals::corner(smooth, normals[index], face)
                            .to_array(),
                        color,
                        rest_position: asset
                            .geometry
                            .material_coordinates
                            .as_ref()
                            .unwrap_or(&asset.geometry.positions)[index],
                        material: p.asset + 1,
                        material_projection_weights: weights[index],
                    });
                }
            }
            self.drawn_avatars.push(p.actor);
        }
        Ok(())
    }
}
pub(crate) fn avatar_world(p: &crate::npc_presentation::Pose, anchor: [f32; 3]) -> Mat4 {
    let rotation = Quat::from_rotation_y(-f32::from(p.yaw) * 0.01f32.to_radians())
        * Quat::from_rotation_x(f32::from(p.pitch) * 0.01f32.to_radians())
        * Quat::from_array(p.orientation).normalize();
    Mat4::from_scale_rotation_translation(
        Vec3::splat(p.scale),
        rotation,
        Vec3::from(anchor) + rotation * Vec3::from(p.offset),
    )
}
pub(crate) fn avatar_locals(asset: &Asset, p: &crate::npc_presentation::Pose) -> [Mat4; MAX_BONES] {
    let mut out = [Mat4::IDENTITY; MAX_BONES];
    if let Some(rig) = &asset.rig {
        let local = |bone: usize, clip: Option<u8>, tick: u32| {
            let value = clip
                .and_then(|i| {
                    asset.clips[i as usize]
                        .tracks
                        .iter()
                        .find(|t| t.bone as usize == bone)
                })
                .map_or_else(
                    || transform(rig.bones[bone].translation, rig.bones[bone].rotation),
                    |t| sample(t, tick),
                );
            if p.strip_root == Some(bone as u8) {
                let (_, r, mut t) = value.to_scale_rotation_translation();
                t.x = rig.bones[bone].translation[0];
                t.z = rig.bones[bone].translation[2];
                Mat4::from_rotation_translation(r, t)
            } else {
                value
            }
        };
        for (i, slot) in out.iter_mut().enumerate().take(rig.bones.len()) {
            let current = local(i, p.clip, p.clip_tick);
            *slot = if p.blend < 10000 {
                let before = local(i, p.previous_clip, p.previous_tick);
                let (_, ar, at) = before.to_scale_rotation_translation();
                let (_, br, bt) = current.to_scale_rotation_translation();
                let f = f32::from(p.blend) / 10000.;
                Mat4::from_rotation_translation(ar.slerp(br, f), at.lerp(bt, f))
            } else {
                current
            };
        }
        for j in &p.joints[..p.joint_count as usize] {
            out[j.bone as usize] = transform(j.translation, j.rotation);
        }
    }
    out
}
fn avatar_skeleton(asset: &Asset, p: &crate::npc_presentation::Pose) -> Skeleton {
    let count = asset.rig.as_ref().map_or(0, |r| r.bones.len());
    let mut out = Skeleton {
        clip_tick: p.clip.map(|_| p.clip_tick),
        posed: vec![Mat4::IDENTITY; count],
        skin: vec![Mat4::IDENTITY; count],
    };
    if let Some(rig) = &asset.rig {
        let bind = rig.bind_globals();
        out.posed
            .copy_from_slice(&crate::avatar_ik::globals(rig, &avatar_locals(asset, p))[..count]);
        for (i, mat) in bind.iter().enumerate().take(rig.bones.len()) {
            out.skin[i] = out.posed[i] * mat.inverse();
        }
    }
    out
}
