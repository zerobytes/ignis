//! Source-owned, bounded authoring handles and rest-posture warps. No per-frame IO.
use super::*;
fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Handle {
    pub name: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub part: String,
    pub anchor: Anchor,
    /// Three scalar channels, in source-local metres, stored with all other parameters.
    pub parameters: [String; 3],
    pub limits: [f32; 3],
    pub mirror: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub midline: bool,
    pub frame_joint: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WarpWeight {
    pub vertex: u32,
    pub weight: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Warp {
    pub name: String,
    pub parameter: String,
    pub origin: Anchor,
    pub axis: [f32; 3],
    pub angle_degrees: f32,
    pub weights: Vec<WarpWeight>,
    /// Source3 rigid inserts preserve their prior twist through these final warps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rigid_sockets: Vec<u8>,
}
#[derive(Debug, Clone, Serialize)]
pub struct HandlePose {
    pub name: String,
    pub label: String,
    pub group: Option<String>,
    pub position: [f32; 3],
    pub basis: [[f32; 3]; 3],
    pub offset: [f32; 3],
    pub limits: [f32; 3],
    pub mirror: Option<String>,
    pub midline: bool,
}
impl Source {
    pub(super) fn validate_controls(
        &self,
        parameters: &BTreeMap<&String, &Parameter>,
        mut work: usize,
    ) -> Result<()> {
        let handle_budget = if self.version >= 8 {
            128
        } else if self.version >= 5 {
            80
        } else {
            64
        };
        if self.handles.len() > handle_budget || self.warps.len() > 16 {
            return Err(invalid(format!(
                "Character handles/warps exceed{handle_budget}/16 source budget"
            )));
        }
        let anchor_valid = |anchor: &Anchor| {
            !anchor.vertices.is_empty()
                && anchor.vertices.len() <= 32
                && anchor
                    .vertices
                    .iter()
                    .all(|&v| (v as usize) < self.positions.len())
        };
        let mut names = BTreeSet::new();
        let mut channels = BTreeSet::new();
        for h in &self.handles {
            super::super::id(&h.name)?;
            if !names.insert(&h.name)
                || h.midline && h.mirror.is_some()
                || h.label.is_empty()
                || h.label.len() > 128
                || h.group
                    .as_ref()
                    .is_some_and(|g| self.version < 3 || g.is_empty() || g.len() > 64)
                || !anchor_valid(&h.anchor)
                || h.frame_joint as usize >= self.joints.len()
                || !self.parts.iter().any(|p| {
                    p.name == h.part && h.anchor.vertices.iter().all(|v| p.vertices.contains(v))
                })
            {
                return Err(invalid(
                    "Invalid character handle identity, anchor, part or joint",
                ));
            }
            for (i, name) in h.parameters.iter().enumerate() {
                let p = parameters
                    .get(name)
                    .ok_or_else(|| invalid("Unknown character handle parameter"))?;
                if !channels.insert(name)
                    || !h.limits[i].is_finite()
                    || !(0.00001..=0.1).contains(&h.limits[i])
                    || p.minimum != -h.limits[i]
                    || p.maximum != h.limits[i]
                    || p.default != 0.
                {
                    return Err(invalid(
                        "Handle channels require distinct zero-centred parameters with matching metre limits",
                    ));
                }
            }
        }
        for h in &self.handles {
            if let Some(m) = &h.mirror {
                let other = self
                    .handles
                    .iter()
                    .find(|v| v.name == *m)
                    .ok_or_else(|| invalid("Unknown mirrored character handle"))?;
                if other.name == h.name
                    || other.mirror.as_deref() != Some(h.name.as_str())
                    || other.limits != h.limits
                {
                    return Err(invalid(
                        "Character mirror pairing must be reciprocal with equal limits",
                    ));
                }
            }
        }
        names.clear();
        let mut rigid_suffix = false;
        let mut rigid_joints = None;
        for w in &self.warps {
            super::super::id(&w.name)?;
            let p = parameters
                .get(&w.parameter)
                .ok_or_else(|| invalid("Unknown posture parameter"))?;
            if !names.insert(&w.name)
                || !anchor_valid(&w.origin)
                || !Vec3::from(w.axis).is_finite()
                || (Vec3::from(w.axis).length_squared() - 1.).abs() > 0.0001
                || !w.angle_degrees.is_finite()
                || w.angle_degrees.abs() > 45.
                || p.minimum < -1.
                || p.maximum > 1.
            {
                return Err(invalid("Invalid character rest-posture warp"));
            }
            let mut previous = None;
            for v in &w.weights {
                if v.vertex as usize >= self.positions.len()
                    || previous.is_some_and(|i| i >= v.vertex)
                    || !v.weight.is_finite()
                    || !(0.0..=1.).contains(&v.weight)
                {
                    return Err(invalid(
                        "Warp weights require ordered valid vertices and finite0..1 weights",
                    ));
                }
                previous = Some(v.vertex);
            }
            work = work.saturating_add(w.weights.len());
            if !w.rigid_sockets.is_empty() {
                rigid_suffix = true;
                if rigid_joints.is_some_and(|j| j != &w.rigid_sockets) {
                    return Err(invalid(
                        "Final rigid socket warps must transport the same insert set",
                    ));
                }
                rigid_joints = Some(&w.rigid_sockets);
                if self.version < 3
                    || w.rigid_sockets.len() > super::super::MAX_BONES
                    || w.rigid_sockets.windows(2).any(|j| j[0] >= j[1])
                {
                    return Err(invalid(
                        "Rigid socket transport requires source3 and ordered distinct joints",
                    ));
                }
                for &joint in &w.rigid_sockets {
                    let bone = self
                        .joints
                        .get(joint as usize)
                        .ok_or_else(|| invalid("Unknown rigid socket joint"))?;
                    if !self.parts.iter().any(|p| p.socket == Some(joint))
                        || bone
                            .head
                            .vertices
                            .iter()
                            .chain(&bone.tail.vertices)
                            .any(|v| {
                                w.weights
                                    .binary_search_by_key(v, |p| p.vertex)
                                    .ok()
                                    .is_none_or(|i| w.weights[i].weight != 1.)
                            })
                    {
                        return Err(invalid(
                            "Rigid socket head/tail anchors require full warp weight",
                        ));
                    }
                }
            } else if rigid_suffix {
                return Err(invalid(
                    "Rigid socket transport must be the final warp group",
                ));
            }
        }
        if work > MAX_DELTAS {
            return Err(invalid(
                "Combined character targets/warps exceed source work budget",
            ));
        }
        Ok(())
    }
    pub(super) fn apply_warps(
        &self,
        positions: &mut [[f32; 3]],
        values: &BTreeMap<String, f32>,
    ) -> Result<BTreeMap<u8, Quat>> {
        let mut rotations: BTreeMap<u8, Quat> = BTreeMap::new();
        for w in &self.warps {
            let angle = (w.angle_degrees * values[&w.parameter]).to_radians();
            if angle == 0. {
                continue;
            }
            let origin = w.origin.at(positions);
            if !w.rigid_sockets.is_empty() {
                let before = if w.rigid_sockets.iter().any(|j| !rotations.contains_key(j)) {
                    Some(self.bind(positions)?.1)
                } else {
                    None
                };
                let rotation = Quat::from_axis_angle(w.axis.into(), angle);
                for &joint in &w.rigid_sockets {
                    let previous = rotations.get(&joint).copied().unwrap_or_else(|| {
                        before.as_ref().unwrap()[joint as usize]
                            .to_scale_rotation_translation()
                            .1
                    });
                    rotations.insert(joint, (rotation * previous).normalize());
                }
            }
            for v in &w.weights {
                if v.weight == 0. {
                    continue;
                }
                let p = Vec3::from(positions[v.vertex as usize]);
                positions[v.vertex as usize] = (origin
                    + Quat::from_axis_angle(w.axis.into(), angle * v.weight) * (p - origin))
                    .to_array();
            }
        }
        Ok(rotations)
    }
    pub fn handle(&self, name: &str) -> Result<&Handle> {
        self.handles
            .iter()
            .find(|h| h.name == name)
            .ok_or_else(|| invalid("Character handle not found"))
    }
    pub fn handle_configuration(
        &self,
        configuration: &Configuration,
        name: &str,
        offset: [f32; 3],
        symmetry: bool,
    ) -> Result<Configuration> {
        let h = self.handle(name)?;
        if offset
            .iter()
            .zip(h.limits)
            .any(|(&v, limit)| !v.is_finite() || v.abs() > limit)
        {
            return Err(invalid(
                "Character handle offset exceeds its local metre limits",
            ));
        }
        let mut result = self.canonical_configuration(configuration)?;
        let offset = if symmetry && h.midline {
            [0., offset[1], offset[2]]
        } else {
            offset
        };
        result
            .parameters
            .extend(h.parameters.iter().cloned().zip(offset));
        if symmetry && let Some(mirror) = &h.mirror {
            let m = self.handle(mirror)?;
            result.parameters.extend(
                m.parameters
                    .iter()
                    .cloned()
                    .zip([-offset[0], offset[1], offset[2]]),
            );
        }
        self.canonical_configuration(&result)
    }
    /// Derived from the current bake, including posture, height and instance transform.
    pub(crate) fn handle_poses(
        &self,
        configuration: &Configuration,
        placement: &Placement,
        document: &Document,
    ) -> Result<Vec<HandlePose>> {
        if self.handles.is_empty() {
            return Ok(Vec::new());
        }
        let (_, rest) = self.bind(&self.positions)?;
        let values = self.values(configuration)?;
        let rotation = Quat::from_array(placement.rotation);
        let mut maps = BTreeMap::new();
        for p in &self.parts {
            if self.handles.iter().any(|h| h.part == p.name) {
                maps.insert(
                    &p.name,
                    p.vertices
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| (v, i))
                        .collect::<BTreeMap<_, _>>(),
                );
            }
        }
        let mut out = Vec::with_capacity(self.handles.len());
        for h in &self.handles {
            let a = document.asset(&format!("{}_{}", placement.prefix, h.part))?;
            let map = &maps[&h.part];
            let mut position = Vec3::ZERO;
            for v in &h.anchor.vertices {
                position += Vec3::from(
                    *a.geometry
                        .positions
                        .get(map[v])
                        .ok_or_else(|| invalid("Character handle bake topology changed"))?,
                );
            }
            position /= h.anchor.vertices.len() as f32;
            let rig = a
                .rig
                .as_ref()
                .ok_or_else(|| invalid("Character handle requires a fitted rig"))?;
            let current = rig.bind_globals();
            let frame = current[h.frame_joint as usize] * rest[h.frame_joint as usize].inverse();
            let basis = [Vec3::X, Vec3::Y, Vec3::Z].map(|v| {
                (rotation * frame.transform_vector3(v))
                    .normalize()
                    .to_array()
            });
            out.push(HandlePose {
                name: h.name.clone(),
                label: h.label.clone(),
                group: h.group.clone(),
                position: (Vec3::from(placement.position) + rotation * position * placement.scale)
                    .to_array(),
                basis,
                offset: std::array::from_fn(|i| values[&h.parameters[i]]),
                limits: h.limits,
                mirror: h.mirror.clone(),
                midline: h.midline,
            });
        }
        Ok(out)
    }
}
