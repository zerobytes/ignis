//! Species stature driven by the same canonical age in game days.
use super::*;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Stature {
    pub adult_diameter_m: f64,
    pub seedling_diameter_m: f64,
    pub maturity_years: f64,
}
impl Default for Stature {
    fn default() -> Self {
        Self {
            adult_diameter_m: 0.2,
            seedling_diameter_m: 0.01,
            maturity_years: 30.,
        }
    }
}
impl Stature {
    pub fn validate(&self) -> Result<()> {
        if !self.adult_diameter_m.is_finite()
            || self.adult_diameter_m <= 0.
            || !self.seedling_diameter_m.is_finite()
            || self.seedling_diameter_m <= 0.
            || self.seedling_diameter_m > self.adult_diameter_m
            || !self.maturity_years.is_finite()
            || self.maturity_years <= 0.
        {
            return Err(invalid(
                "Stature requires finite positive diameters and maturity; seedling cannot exceed adult",
            ));
        }
        Ok(())
    }
    pub fn diameter(&self, age_days: u64) -> f64 {
        let t = (age_days as f64 / 365. / self.maturity_years).clamp(0., 1.);
        let g = t * t * (3. - 2. * t);
        self.seedling_diameter_m * (1. - g) + self.adult_diameter_m * g
    }
    pub(super) fn apply(&self, recipe: &mut Recipe, age: u64) -> Result<()> {
        self.validate()?;
        let Shape::Axis { radius, .. } = recipe.source.organs[0].shape else {
            return Err(invalid("Stature requires a woody root axis"));
        };
        let factor = (self.diameter(age) * 0.5) / radius;
        if !factor.is_finite() || factor <= 0. {
            return Err(invalid("Stature scale is unresolved at numeric precision"));
        }
        // Preserve the species' structural proportions and collar geometry. Independent
        // bole/crown proportions belong to the species grammar, not this size trait.
        for organ in &mut Arc::make_mut(&mut recipe.source).organs {
            if let Shape::Axis { radius, .. } = &mut organ.shape {
                for p in &mut organ.path {
                    for v in p {
                        *v *= factor;
                    }
                }
                *radius *= factor;
            } else {
                // Leaves keep their own age-dependent physical size, while their
                // insertion follows the enlarged supporting axis.
                let origin = DVec3::from_array(organ.path[0]);
                for p in &mut organ.path {
                    *p = (origin * factor + (DVec3::from_array(*p) - origin)).to_array();
                }
            }
        }
        recipe.validate()
    }
}
