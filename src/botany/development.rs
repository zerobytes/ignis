//! Chronological authoring snapshots, independent of the mechanical expansion clock.
use super::*;
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Model {
    #[default]
    Woody,
    Palm,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Axis,
    Frond,
    AerialRoot,
    Root,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct OrganLife {
    pub born_years: f64,
    pub expansion_years: f64,
    pub role: Role,
    /// Authored fraction of developed size for a complete leaf, not a biological age.
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub growth_stage: f64,
}
impl Default for OrganLife {
    fn default() -> Self {
        Self {
            born_years: 0.,
            expansion_years: 2.,
            role: Role::Axis,
            growth_stage: 1.,
        }
    }
}
impl OrganLife {
    pub fn validate(&self) -> Result<()> {
        if !self.growth_stage.is_finite()
            || !(0.1..=1.).contains(&self.growth_stage)
            || (self.role != Role::Frond && self.growth_stage != 1.)
            || !self.born_years.is_finite()
            || !(0. ..=2000.).contains(&self.born_years)
            || !self.expansion_years.is_finite()
            || !(0.1..=200.).contains(&self.expansion_years)
        {
            return Err(invalid("Invalid botanical organ chronology"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    pub model: Model,
    pub reference_age_years: f64,
    pub height_time_years: f64,
    pub crown_time_years: f64,
    pub radial_time_years: f64,
    pub palm_establishment_years: f64,
    pub branch_thickening_decay: f64,
    pub branch_length_decay: f64,
    pub leaf_maturity_years: f64,
    pub senescence_start_years: f64,
    pub senescence_time_years: f64,
    pub senescent_foliage_loss: f64,
    pub basal_flare: f64,
    pub bark_gain: f64,
    pub stiffness_gain: f64,
    /// Visible cohorts emerge at this fraction of their reference length.
    pub emergence_fraction: f64,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            model: Model::Woody,
            reference_age_years: 8.,
            height_time_years: 25.,
            crown_time_years: 14.,
            radial_time_years: 45.,
            palm_establishment_years: 4.,
            branch_thickening_decay: 0.6,
            branch_length_decay: 0.72,
            leaf_maturity_years: 4.,
            senescence_start_years: 120.,
            senescence_time_years: 40.,
            senescent_foliage_loss: 0.5,
            basal_flare: 0.5,
            bark_gain: 1.5,
            stiffness_gain: 0.5,
            emergence_fraction: 0.5,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct State {
    pub age_years: f64,
    pub physiological_years: f64,
    pub height: f64,
    pub crown: f64,
    pub radial: f64,
    pub leaf: f64,
    pub maturity: f64,
    pub senescence: f64,
    pub foliage_survival: f64,
    pub stiffness: f64,
}
fn growth(t: f64, tau: f64) -> f64 {
    -(-t / tau).exp_m1()
}
fn smooth(v: f64) -> f64 {
    let t = v.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        for t in [
            self.reference_age_years,
            self.height_time_years,
            self.crown_time_years,
            self.radial_time_years,
            self.palm_establishment_years,
            self.leaf_maturity_years,
            self.senescence_time_years,
        ] {
            if !t.is_finite() || !(0.1..=500.).contains(&t) {
                return Err(invalid("Invalid botanical development time"));
            }
        }
        if !self.branch_length_decay.is_finite() || !(0. ..=1.).contains(&self.branch_length_decay)
        {
            return Err(invalid("Invalid botanical branch length decay"));
        }
        if !self.senescence_start_years.is_finite()
            || !(1. ..=2000.).contains(&self.senescence_start_years)
            || !self.branch_thickening_decay.is_finite()
            || !(0. ..=1.).contains(&self.branch_thickening_decay)
            || !self.senescent_foliage_loss.is_finite()
            || !(0. ..=0.95).contains(&self.senescent_foliage_loss)
            || !self.emergence_fraction.is_finite()
            || !(0.1..=1.).contains(&self.emergence_fraction)
            || [self.basal_flare, self.bark_gain, self.stiffness_gain]
                .iter()
                .any(|v| !v.is_finite() || !(0. ..=4.).contains(v))
        {
            return Err(invalid("Invalid botanical development profile"));
        }
        Ok(())
    }
    pub fn evaluate(&self, age: f64, vigor: f64) -> Result<State> {
        self.validate()?;
        if !age.is_finite()
            || !(0.25..=2000.).contains(&age)
            || !vigor.is_finite()
            || !(0.25..=2.).contains(&vigor)
        {
            return Err(invalid("Age must be0.25..2000years and vigor0.25..2"));
        }
        let t = age * vigor;
        let reference = self.reference_age_years;
        let ratio = |tau| growth(t, tau) / growth(reference, tau);
        let radial = if self.model == Model::Palm {
            smooth(t / self.palm_establishment_years)
                / smooth(reference / self.palm_establishment_years)
        } else {
            ratio(self.radial_time_years)
        };
        let maturity = growth(t, 20.);
        let senescence = growth(
            (t - self.senescence_start_years).max(0.),
            self.senescence_time_years,
        );
        let leaf = (0.55 + 0.45 * growth(t, self.leaf_maturity_years))
            / (0.55 + 0.45 * growth(reference, self.leaf_maturity_years));
        let result = State {
            age_years: age,
            physiological_years: t,
            height: ratio(self.height_time_years),
            crown: ratio(self.crown_time_years),
            radial,
            leaf,
            maturity,
            senescence,
            foliage_survival: 1. - self.senescent_foliage_loss * senescence,
            stiffness: (1. + self.stiffness_gain * maturity)
                / (1. + self.stiffness_gain * growth(reference, 20.)),
        };
        if [
            result.height,
            result.crown,
            result.radial,
            result.leaf,
            result.stiffness,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v <= 0. || *v > 20.)
        {
            return Err(invalid(
                "Botanical development ratio exceeds20; revise profile/reference age",
            ));
        }
        Ok(result)
    }
    pub fn expansion(&self, life: Option<&OrganLife>, t: f64) -> Option<f64> {
        let Some(life) = life else {
            return Some(1.);
        };
        if t < life.born_years {
            return None;
        }
        Some(
            self.emergence_fraction
                + (1. - self.emergence_fraction)
                    * smooth((t - life.born_years) / life.expansion_years),
        )
    }
}
pub fn state(recipe: &Recipe) -> Result<Option<State>> {
    recipe
        .source
        .development
        .as_ref()
        .map(|p| {
            p.evaluate(
                recipe.settings.age_years.unwrap_or(p.reference_age_years),
                recipe.settings.vigor,
            )
        })
        .transpose()
}
