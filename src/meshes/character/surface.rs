//! Parameter-driven surface relief uses the existing material renderer and rest coordinates.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub part: String,
    pub factors: Vec<Factor>,
    pub coordinates: Vec<[f32; 3]>,
    pub detail: crate::materials::Detail,
}
impl Source {
    pub(super) fn validate_surfaces(
        &self,
        parameters: &BTreeMap<&String, &Parameter>,
    ) -> Result<()> {
        if self.surfaces.len() > 32 {
            return Err(invalid("Character surface budget exceeds32 parts"));
        }
        let mut parts = BTreeSet::new();
        for s in &self.surfaces {
            let part = self
                .parts
                .iter()
                .find(|p| p.name == s.part)
                .ok_or_else(|| invalid("Unknown character surface part"))?;
            if !parts.insert(&s.part)
                || part.socket.is_some()
                || part.material.detail.is_some()
                || s.coordinates.len() != part.vertices.len()
                || s.factors.is_empty()
                || s.factors.len() > 8
            {
                return Err(invalid(
                    "Character surface requires one skinned part, matching coordinates and1..8 factors",
                ));
            }
            s.detail.validate()?;
            for coordinate in &s.coordinates {
                super::super::finite(coordinate)?;
            }
            for f in &s.factors {
                let p = parameters
                    .get(&f.parameter)
                    .ok_or_else(|| invalid("Unknown surface parameter"))?;
                if !(2..=16).contains(&f.curve.len())
                    || f.curve.windows(2).any(|v| v[0][0] >= v[1][0])
                    || f.curve.iter().any(|v| {
                        !v[0].is_finite() || !v[1].is_finite() || !(0. ..=1.).contains(&v[1])
                    })
                    || f.curve[0][0] > p.minimum
                    || f.curve.last().unwrap()[0] < p.maximum
                {
                    return Err(invalid(
                        "Surface curves require finite ordered knots covering the parameter and responses0..1",
                    ));
                }
            }
        }
        Ok(())
    }
    pub(super) fn apply_surface(
        &self,
        part: &Part,
        values: &BTreeMap<String, f32>,
        asset: &mut Asset,
    ) {
        if let Some(s) = self.surfaces.iter().find(|s| s.part == part.name) {
            let gain = s
                .factors
                .iter()
                .fold(1., |v, f| v * response(&f.curve, values[&f.parameter]));
            if gain != 0. {
                let mut detail = s.detail.clone();
                detail.bump *= gain;
                detail.opacity *= gain;
                asset.material.detail = Some(Arc::new(detail));
                Arc::make_mut(&mut asset.geometry).material_coordinates =
                    Some(s.coordinates.clone());
            }
        }
    }
}
