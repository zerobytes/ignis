//! Bounded overlap of source-owned proportional XYZ displacement fields.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlendField {
    pub parameters: [String; 3],
    pub targets: [String; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FieldBlend {
    /// A 0..1 channel that interpolates the original and proportional fields.
    pub strength: String,
    pub fields: Vec<BlendField>,
}

impl Source {
    pub(super) fn validate_blends(&self, parameters: &BTreeMap<&String, &Parameter>) -> Result<()> {
        if self.field_blends.is_empty() {
            return Ok(());
        }
        if self.version < 10 || self.field_blends.len() > 4 {
            return Err(invalid(
                "Proportional field blending requires source10; maximum4 groups",
            ));
        }
        let targets: BTreeMap<_, _> = self.targets.iter().map(|t| (&t.name, t)).collect();
        let mut used = BTreeSet::new();
        let mut channels = BTreeSet::new();
        for blend in &self.field_blends {
            let p = parameters
                .get(&blend.strength)
                .ok_or_else(|| invalid("Unknown field blend strength"))?;
            if p.minimum != 0.
                || p.maximum != 1.
                || blend.fields.is_empty()
                || blend.fields.len() > 128
            {
                return Err(invalid(
                    "Field blend requires a0..1 strength and1..128 XYZ fields",
                ));
            }
            for field in &blend.fields {
                let mut weights: Option<Vec<(u32, f32)>> = None;
                for axis in 0..3 {
                    let p = parameters
                        .get(&field.parameters[axis])
                        .ok_or_else(|| invalid("Unknown proportional field channel"))?;
                    if !channels.insert(&field.parameters[axis])
                        || p.minimum != -p.maximum
                        || p.maximum <= 0.
                        || p.default != 0.
                    {
                        return Err(invalid(
                            "Proportional XYZ fields require distinct zero-centred channels",
                        ));
                    }
                    let t = targets
                        .get(&field.targets[axis])
                        .ok_or_else(|| invalid("Unknown proportional field target"))?;
                    if !used.insert(&t.name)
                        || t.gain != 1.
                        || t.factors
                            != [
                                Factor {
                                    parameter: p.name.clone(),
                                    curve: vec![[-p.maximum, -1.], [0., 0.], [p.maximum, 1.]],
                                },
                                Factor {
                                    parameter: blend.strength.clone(),
                                    curve: vec![[0., 0.], [1., 1.]],
                                },
                            ]
                    {
                        return Err(invalid(
                            "Proportional targets require unique linear XYZ and strength factors",
                        ));
                    }
                    let mut current = Vec::with_capacity(t.deltas.len());
                    for d in &t.deltas {
                        let w = d.offset[axis] / p.maximum;
                        if !(0.0..=2.0).contains(&w)
                            || d.offset
                                .iter()
                                .enumerate()
                                .any(|(a, &v)| a != axis && v != 0.)
                        {
                            return Err(invalid(
                                "Proportional target weights require0..2 and one XYZ axis",
                            ));
                        }
                        current.push((d.vertex, w));
                    }
                    if let Some(weights) = &weights {
                        if weights.len() != current.len()
                            || weights
                                .iter()
                                .zip(&current)
                                .any(|(&(i, w), &(j, v))| i != j || (w - v).abs() > 0.000002)
                        {
                            return Err(invalid("Proportional XYZ target supports must match"));
                        }
                    } else {
                        weights = Some(current);
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn apply_blends(
        &self,
        positions: &mut [[f32; 3]],
        values: &BTreeMap<String, f32>,
        gains: &[f32],
    ) {
        if self.field_blends.is_empty() {
            return;
        }
        let targets: BTreeMap<_, _> = self
            .targets
            .iter()
            .enumerate()
            .map(|(i, t)| (&t.name, i))
            .collect();
        let parameters: BTreeMap<_, _> = self.parameters.iter().map(|p| (&p.name, p)).collect();
        for blend in &self.field_blends {
            if values[&blend.strength] == 0. {
                continue;
            }
            let mut displacement = vec![Vec3::ZERO; positions.len()];
            let mut support = vec![0f32; positions.len()];
            for field in &blend.fields {
                // Zero offsets exert no constraint. Activation is continuous;
                // moving a new point slightly cannot abruptly pin its neighbors.
                let activation = field
                    .parameters
                    .iter()
                    .map(|p| values[p].abs() / parameters[p].maximum)
                    .fold(0., f32::max);
                if activation == 0. {
                    continue;
                }
                let t = &self.targets[targets[&field.targets[0]]];
                let limit = parameters[&field.parameters[0]].maximum;
                for d in &t.deltas {
                    support[d.vertex as usize] += activation * d.offset[0] / limit;
                }
                for target in &field.targets {
                    let index = targets[target];
                    let gain = gains[index];
                    if gain == 0. {
                        continue;
                    }
                    for d in &self.targets[index].deltas {
                        displacement[d.vertex as usize] += Vec3::from(d.offset) * gain;
                    }
                }
            }
            for ((p, delta), weight) in positions.iter_mut().zip(displacement).zip(support) {
                if delta != Vec3::ZERO {
                    *p = (Vec3::from(*p) + delta / weight.max(1.)).to_array();
                }
            }
        }
    }
}
