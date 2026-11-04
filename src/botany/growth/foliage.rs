//! Optional early foliage, independent of woody-axis expansion speed.
use super::*;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub expansion_days: u64,
    pub apical_leaves: u16,
    pub primary_leaves: u16,
    pub secondary_leaves: u16,
    pub length_m: f64,
    pub width_m: f64,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            expansion_days: 30,
            apical_leaves: 16,
            primary_leaves: 24,
            secondary_leaves: 8,
            length_m: 0.25,
            width_m: 0.085,
        }
    }
}
impl Profile {
    pub(super) fn validate(&self) -> Result<()> {
        if !(1..=MAX_TIME).contains(&self.expansion_days)
            || !(2..=64).contains(&self.apical_leaves)
            || !(2..=64).contains(&self.primary_leaves)
            || !(2..=32).contains(&self.secondary_leaves)
            || !self.length_m.is_finite()
            || !(0.02..=1.).contains(&self.length_m)
            || !self.width_m.is_finite()
            || !(0.005..=0.5).contains(&self.width_m)
            || self.width_m > self.length_m
        {
            return Err(invalid("Invalid early foliage profile"));
        }
        Ok(())
    }
    pub(super) fn dimensions(&self, fraction: f64) -> (f64, f64) {
        let scale = 0.45 + 0.55 * fraction;
        (self.length_m * scale, self.width_m * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seedling_and_every_branch_have_leaves_before_wood_is_mature() {
        let profile = Profile::default();
        let d = Definition {
            version: 2,
            interval: 365,
            maximum_nodes: 10,
            foliage: Some(profile.clone()),
            ..Default::default()
        };
        let mut plant = Plant::new(d, 0).unwrap();
        let mut previous_leaves = 0;
        for day in [0, 1, 30, 180, 365, 395, 5 * 365, 10 * 365, 20 * 365] {
            let recipe = plant.advance(day).unwrap();
            let organs = &recipe.source.organs;
            let leaves = organs
                .iter()
                .filter(|o| matches!(o.shape, Shape::Blade { .. }))
                .count();
            assert!(leaves >= profile.apical_leaves as usize && leaves >= previous_leaves);
            for woody in organs
                .iter()
                .filter(|o| matches!(o.shape, Shape::Axis { .. }))
            {
                assert!(
                    organs.iter().any(|o| o.parent.as_deref() == Some(&woody.id)
                        && matches!(o.shape, Shape::Blade { .. })),
                    "{day}: {} lacks leaves",
                    woody.id
                );
            }
            previous_leaves = leaves;
            if day == 395 {
                let leaf = organs.iter().find(|o| o.id == "node0/leaf0").unwrap();
                let Shape::Blade { width, .. } = leaf.shape else {
                    panic!()
                };
                assert!((width - profile.width_m).abs() < 1e-12);
                let branch = organs.iter().find(|o| o.id == "node0").unwrap();
                let length: f64 = branch
                    .path
                    .windows(2)
                    .map(|p| {
                        (glam::DVec3::from_array(p[1]) - glam::DVec3::from_array(p[0])).length()
                    })
                    .sum();
                assert!(
                    length < 0.5,
                    "Leaf expansion must not wait for full woody extension"
                );
            }
        }
        let restored: Plant = serde_json::from_slice(&serde_json::to_vec(&plant).unwrap()).unwrap();
        assert_eq!(plant.recipe().unwrap(), restored.recipe().unwrap());
        plant.production.remove_site("node0").unwrap();
        let pruned = plant.advance(21 * 365).unwrap();
        assert!(
            !pruned
                .source
                .organs
                .iter()
                .any(|o| o.id == "node0" || o.id.starts_with("node0/"))
        );
        assert!(
            pruned
                .source
                .organs
                .iter()
                .any(|o| o.id.starts_with("stem/young_leaf"))
        );
    }
    #[test]
    fn early_foliage_is_explicit_and_invalid_profiles_are_rejected() {
        let mut d = Definition {
            version: 2,
            foliage: Some(Profile::default()),
            ..Default::default()
        };
        d.validate().unwrap();
        d.foliage.as_mut().unwrap().expansion_days = 0;
        assert!(d.validate().is_err());
        d.foliage = Some(Profile::default());
        d.version = 1;
        assert!(d.validate().is_err());
        d.version = 2;
        d.foliage = None;
        let p = Plant::new(d, 0).unwrap();
        assert_eq!(p.recipe().unwrap().source.organs.len(), 1);
        assert!(!serde_json::to_string(&p).unwrap().contains("foliage"));
    }
}
