//! Offline character authoring: sparse, parameter-driven shapes and fitted bind rigs.
//! The source stays separate from a baked Scene; authoring recipes can link the bake.
use super::{Asset, Bone, Document, Geometry, Instance, Rig, Shading, Weight};
use crate::model::{Result, invalid};
use glam::{Mat4, Quat, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod controls;
pub use controls::{Handle, HandlePose, Warp, WarpWeight};
mod surface;
pub use surface::Surface;

mod refinement;
pub use refinement::Refinement;
pub mod eyes;
pub use eyes::Eye;

pub const VERSION: u16 = 9;
pub const MAX_PARAMETERS: usize = 512;
pub const MAX_BYTES: usize = 96 * 1024 * 1024;
pub const LEGACY_MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_VERTICES: usize = 100_000;
const MAX_DELTAS: usize = 4_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub name: String,
    pub minimum: f32,
    pub maximum: f32,
    pub default: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Factor {
    pub parameter: String,
    /// Piecewise linear response, strictly increasing abscissae; endpoint hold.
    pub curve: Vec<[f32; 2]>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Delta {
    pub vertex: u32,
    pub offset: [f32; 3],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub name: String,
    pub gain: f32,
    /// Products express combination shapes, e.g. muscle x fat x breast volume.
    pub factors: Vec<Factor>,
    pub deltas: Vec<Delta>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub vertices: Vec<u32>,
}
impl Anchor {
    fn at(&self, positions: &[[f32; 3]]) -> Vec3 {
        self.vertices
            .iter()
            .map(|&i| Vec3::from(positions[i as usize]))
            .sum::<Vec3>()
            / self.vertices.len() as f32
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Joint {
    pub name: String,
    pub parent: Option<u8>,
    pub head: Anchor,
    pub tail: Anchor,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub name: String,
    /// Stable source indices; helper vertices can fit joints without being drawn.
    pub vertices: Vec<u32>,
    pub triangles: Vec<[u32; 3]>,
    pub weights: Vec<Weight>,
    /// A rigid insert follows this fitted joint (eyes, teeth, accessories).
    #[serde(default)]
    pub socket: Option<u8>,
    pub color: [f32; 3],
    #[serde(default)]
    pub material: crate::materials::Material,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub version: u16,
    pub id: String,
    pub provenance: String,
    pub positions: Vec<[f32; 3]>,
    pub parameters: Vec<Parameter>,
    pub targets: Vec<Target>,
    pub joints: Vec<Joint>,
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handles: Vec<Handle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warps: Vec<Warp>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaces: Vec<Surface>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refinements: Vec<Refinement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub eyes: Vec<Eye>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(default)]
    pub parameters: BTreeMap<String, f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub prefix: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
}
impl Placement {
    pub fn validate(&self) -> Result<()> {
        super::id(&self.prefix)?;
        super::finite(&self.position)?;
        super::quaternion(self.rotation)?;
        if !self.scale.is_finite() || !(0.001..=100.).contains(&self.scale) {
            return Err(invalid("Character scale requires0.001..100"));
        }
        Ok(())
    }
}
impl Source {
    pub fn validate(&self) -> Result<()> {
        super::id(&self.id)?;
        if !(1..=VERSION).contains(&self.version)
            || !(3..=MAX_VERTICES).contains(&self.positions.len())
            || self.parameters.len()
                > if self.version == 1 {
                    128
                } else if self.version < 6 {
                    256
                } else {
                    MAX_PARAMETERS
                }
            || self.version == 1 && (!self.handles.is_empty() || !self.warps.is_empty())
            || self.version < 3 && !self.surfaces.is_empty()
            || self.targets.len() > if self.version >= 8 { 1024 } else { 512 }
            || self.joints.is_empty()
            || self.joints.len() > super::MAX_BONES
            || self.parts.is_empty()
            || self.parts.len() > 32
            || self.provenance.len() > 8192
        {
            return Err(invalid(
                "Invalid character source version or authoring budgets",
            ));
        }
        for p in &self.positions {
            super::finite(p)?;
        }
        let mut parameters = BTreeMap::new();
        for p in &self.parameters {
            super::id(&p.name)?;
            if !p.minimum.is_finite()
                || !p.maximum.is_finite()
                || !p.default.is_finite()
                || p.minimum >= p.maximum
                || !(p.minimum..=p.maximum).contains(&p.default)
                || parameters.insert(&p.name, p).is_some()
            {
                return Err(invalid("Invalid or duplicate character parameter"));
            }
        }
        let mut names = BTreeSet::new();
        let mut deltas = 0usize;
        for target in &self.targets {
            super::id(&target.name)?;
            if !names.insert(&target.name)
                || !target.gain.is_finite()
                || target.gain.abs() > 16.
                || target.factors.len() > 8
            {
                return Err(invalid("Invalid or duplicate character target"));
            }
            deltas = deltas.saturating_add(target.deltas.len());
            if deltas > MAX_DELTAS {
                return Err(invalid("Character source exceeds sparse delta work budget"));
            }
            for f in &target.factors {
                let p = parameters
                    .get(&f.parameter)
                    .ok_or_else(|| invalid("Unknown character factor parameter"))?;
                if !(2..=16).contains(&f.curve.len())
                    || f.curve.windows(2).any(|v| v[0][0] >= v[1][0])
                    || f.curve.iter().any(|v| {
                        !v[0].is_finite()
                            || !v[1].is_finite()
                            || !(if self.version == 1 { 0. } else { -1. }..=1.).contains(&v[1])
                    })
                    || f.curve[0][0] > p.minimum
                    || f.curve.last().unwrap()[0] < p.maximum
                {
                    return Err(invalid(
                        "Character response curves must cover the parameter range and have ordered finite knots",
                    ));
                }
            }
            let mut previous = None;
            for d in &target.deltas {
                if d.vertex as usize >= self.positions.len()
                    || previous.is_some_and(|v| v >= d.vertex)
                {
                    return Err(invalid(
                        "Character target indices must be sorted, distinct and in range",
                    ));
                }
                super::finite(&d.offset)?;
                previous = Some(d.vertex);
            }
        }
        self.validate_controls(&parameters, deltas)?;
        self.validate_surfaces(&parameters)?;
        names.clear();
        for (i, j) in self.joints.iter().enumerate() {
            super::id(&j.name)?;
            if !names.insert(&j.name) || j.parent.is_some_and(|p| p as usize >= i) {
                return Err(invalid(
                    "Character joint parents must precede children with unique names",
                ));
            }
            for anchor in [&j.head, &j.tail] {
                if anchor.vertices.is_empty()
                    || anchor.vertices.len() > 32
                    || anchor
                        .vertices
                        .iter()
                        .any(|&v| v as usize >= self.positions.len())
                {
                    return Err(invalid(
                        "Character joint anchors require1..32 valid source vertices",
                    ));
                }
            }
        }
        let (bones, _) = self.bind(&self.positions)?;
        names.clear();
        let mut total = 0usize;
        for part in &self.parts {
            super::id(&part.name)?;
            total = total.saturating_add(part.vertices.len());
            if !names.insert(&part.name)
                || total > MAX_VERTICES
                || part
                    .vertices
                    .iter()
                    .any(|&v| v as usize >= self.positions.len())
                || part.socket.is_some_and(|v| v as usize >= bones.len())
                || part
                    .color
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err(invalid("Invalid character part, socket or vertex budget"));
            }
            Document {
                objects: Default::default(),
                assets: vec![self.asset(
                    part,
                    part.name.clone(),
                    &self.positions,
                    bones.clone(),
                    None,
                )?],
                instances: vec![],
            }
            .validate()?;
        }
        self.validate_refinements()?;
        self.validate_eyes(&parameters)?;
        Ok(())
    }
    fn values(&self, configuration: &Configuration) -> Result<BTreeMap<String, f32>> {
        let mut values: BTreeMap<_, _> = self
            .parameters
            .iter()
            .map(|p| (p.name.clone(), p.default))
            .collect();
        for (name, &v) in &configuration.parameters {
            let p = self
                .parameters
                .iter()
                .find(|p| p.name == *name)
                .ok_or_else(|| invalid(format!("Unknown character parameter: {name}")))?;
            if !v.is_finite() || !(p.minimum..=p.maximum).contains(&v) {
                return Err(invalid(format!("Character parameter out of range: {name}")));
            }
            values.insert(name.clone(), v);
        }
        Ok(values)
    }
    fn bind(&self, positions: &[[f32; 3]]) -> Result<(Vec<Bone>, Vec<Mat4>)> {
        let mut bones = Vec::with_capacity(self.joints.len());
        let mut global: Vec<Mat4> = Vec::with_capacity(self.joints.len());
        for j in &self.joints {
            let head = j.head.at(positions);
            let direction = j.tail.at(positions) - head;
            if !direction.is_finite() || direction.length() < 0.00001 {
                return Err(invalid("Character shape collapses a joint axis"));
            }
            let world = Mat4::from_rotation_translation(
                Quat::from_rotation_arc(Vec3::Y, direction.normalize()),
                head,
            );
            let local = j
                .parent
                .map_or(world, |p| global[p as usize].inverse() * world);
            let (_, rotation, translation) = local.to_scale_rotation_translation();
            bones.push(Bone {
                name: j.name.clone(),
                parent: j.parent,
                translation: translation.to_array(),
                rotation: rotation.normalize().to_array(),
            });
            global.push(world);
        }
        Ok((bones, global))
    }
    fn asset(
        &self,
        part: &Part,
        id: String,
        positions: &[[f32; 3]],
        bones: Vec<Bone>,
        socket: Option<Mat4>,
    ) -> Result<Asset> {
        let geometry = Geometry {
            positions: part
                .vertices
                .iter()
                .map(|&i| {
                    socket.map_or(positions[i as usize], |m| {
                        m.transform_point3(Vec3::from(self.positions[i as usize]))
                            .to_array()
                    })
                })
                .collect(),
            triangles: part.triangles.clone(),
            material_coordinates: None,
        };
        Ok(Asset {
            id,
            geometry: Arc::new(geometry),
            rig: Some(Arc::new(Rig {
                bones,
                weights: part.weights.clone(),
            })),
            shading: Shading::Smooth,
            material: part.material.clone(),
            clips: vec![],
        })
    }
    pub fn compile(
        &self,
        configuration: &Configuration,
        placement: &Placement,
    ) -> Result<(Document, serde_json::Value)> {
        self.validate()?;
        self.compile_cached(configuration, placement)
    }
    pub fn canonical_configuration(&self, configuration: &Configuration) -> Result<Configuration> {
        Ok(Configuration {
            parameters: self.values(configuration)?,
        })
    }
    /// Only for the project service's already decoded and validated source snapshot.
    pub(crate) fn compile_cached(
        &self,
        configuration: &Configuration,
        placement: &Placement,
    ) -> Result<(Document, serde_json::Value)> {
        placement.validate()?;
        let values = self.values(configuration)?;
        let mut positions = self.positions.clone();
        let mut active = 0;
        for t in &self.targets {
            let gain = t.factors.iter().fold(t.gain, |gain, f| {
                gain * response(&f.curve, values[&f.parameter])
            });
            if gain == 0. {
                continue;
            }
            active += 1;
            for d in &t.deltas {
                for (value, offset) in positions[d.vertex as usize].iter_mut().zip(d.offset) {
                    *value += gain * offset;
                }
            }
        }
        let socket_rotations = self.apply_warps(&mut positions, &values)?;
        let (bones, bind) = self.bind(&positions)?;
        let (_, base_bind) = self.bind(&self.positions)?;
        let mut document = Document::default();
        let mut sockets = BTreeMap::new();
        for part in &self.parts {
            let id = format!("{}_{}", placement.prefix, part.name);
            let socket = part.socket.map(|j| {
                let fitted = socket_rotations.get(&j).map_or(bind[j as usize], |q| {
                    Mat4::from_rotation_translation(
                        *q,
                        bind[j as usize].transform_point3(Vec3::ZERO),
                    )
                });
                fitted * base_bind[j as usize].inverse()
            });
            let mut asset = self.asset(part, id.clone(), &positions, bones.clone(), socket)?;
            if let (Some(joint), Some(matrix)) = (part.socket, socket) {
                sockets.insert(joint, matrix);
            }
            self.apply_surface(part, &values, &mut asset);
            self.refine(part, &mut asset)?;
            let eye_color = self.apply_eye(part, &values, socket, &mut asset)?;
            document.assets.push(asset);
            document.instances.push(Instance {
                id: id.clone(),
                asset: id,
                position: placement.position,
                rotation: placement.rotation,
                scale: placement.scale,
                color: eye_color.unwrap_or(part.color),
                playback: None,
                attachment: None,
            });
        }
        self.fit_eyes(&values, placement, &sockets, &mut document)?;
        if !self.eyes.is_empty()
            && document
                .assets
                .iter()
                .map(|a| a.geometry.positions.len())
                .sum::<usize>()
                > MAX_VERTICES
        {
            return Err(invalid("Compiled character exceeds the vertex budget"));
        }
        document.validate()?;
        let report = serde_json::json!({"format":self.version,"source":self.id,"parameters":values,"active_targets":active,"handles":self.handles.len(),"posture_warps":self.warps.len(),"surfaces":self.surfaces.len(),"eyes":self.eyes,
            "parts":self.parts.len(),"bones":bones.len(),"vertices":document.assets.iter().map(|a|a.geometry.positions.len()).sum::<usize>(),
            "triangles":document.assets.iter().map(|a|a.geometry.triangles.len()).sum::<usize>(),"source_embedded_in_scene":false,
            "secondary_motion":false,"pose_correctives":false});
        Ok((document, report))
    }
    pub fn inspect(&self) -> Result<serde_json::Value> {
        self.validate()?;
        Ok(
            serde_json::json!({"format":self.version,"id":self.id,"provenance":self.provenance,"parameters":self.parameters,"handles":self.handles,"posture_warps":self.warps.len(),"surfaces":self.surfaces.len(),"eyes":self.eyes,
            "targets":self.targets.len(),"source_vertices":self.positions.len(),"bones":self.joints.len(),
            "parts":self.parts.iter().map(|p|serde_json::json!({"name":p.name,"vertices":p.vertices.len(),"triangles":p.triangles.len()})).collect::<Vec<_>>(),"world_changed":false}),
        )
    }
}
fn response(curve: &[[f32; 2]], value: f32) -> f32 {
    if value <= curve[0][0] {
        return curve[0][1];
    }
    for w in curve.windows(2) {
        if value <= w[1][0] {
            return w[0][1] + (w[1][1] - w[0][1]) * ((value - w[0][0]) / (w[1][0] - w[0][0]));
        }
    }
    curve.last().unwrap()[1]
}
pub fn decode(bytes: &[u8]) -> Result<Source> {
    if bytes.len() > MAX_BYTES {
        return Err(invalid("Character source exceeds96MiB"));
    }
    let source: Source = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    if source.version < 3 && bytes.len() > LEGACY_MAX_BYTES {
        return Err(invalid("Legacy character source exceeds64MiB"));
    }
    source.validate()?;
    Ok(source)
}
