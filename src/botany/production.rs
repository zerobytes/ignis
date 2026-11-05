//! Stable site/cohort production identities. No scene or renderer ownership yet.
use super::lifecycle::{Individual, Schedule, Stage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FruitId {
    pub site: String,
    pub cohort: u64,
    pub slot: u16,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Production {
    pub plant: Individual,
    pub sites: BTreeMap<String, u16>,
    #[serde(default)]
    pub site_births: BTreeMap<String, u64>,
    pub removed_sites: BTreeSet<String>,
    pub harvested: BTreeSet<FruitId>,
}
impl Production {
    pub fn stage(&self, schedule: &Schedule, id: &FruitId) -> Option<Stage> {
        if self.removed_sites.contains(&id.site)
            || self.harvested.contains(id)
            || id.slot >= *self.sites.get(&id.site)?
        {
            return None;
        }
        let start = schedule.start(id.cohort)?;
        if start
            < self
                .site_births
                .get(&id.site)
                .copied()
                .unwrap_or(self.plant.born)
        {
            return None;
        }
        schedule.stage(&self.plant, id.cohort)
    }
    pub fn harvest(&mut self, schedule: &Schedule, id: FruitId) -> Result<(), &'static str> {
        schedule.validate()?;
        if self.stage(schedule, &id) != Some(Stage::RipeFruit) {
            return Err("fruit is not ripe and attached");
        }
        self.harvested.insert(id);
        Ok(())
    }
    pub fn remove_site(&mut self, site: &str) -> Result<(), &'static str> {
        if !self.sites.contains_key(site) {
            return Err("unknown production site");
        }
        self.removed_sites.insert(site.to_owned());
        Ok(())
    }
    /// Enumerate all currently attached cohorts, including overlapping older periods.
    /// Refuse an excessive candidate interval before allocating its output.
    pub fn attached(
        &self,
        schedule: &Schedule,
        limit: usize,
    ) -> Result<Vec<(FruitId, Stage)>, &'static str> {
        schedule.validate()?;
        if self.plant.dead || self.plant.time < schedule.phase {
            return Ok(Vec::new());
        }
        let duration = schedule.flower_duration + schedule.fruit_duration + schedule.ripe_duration;
        if duration == 0 {
            return Ok(Vec::new());
        }
        let last = (self.plant.time - schedule.phase) / schedule.period;
        let first_time = self.plant.time.saturating_sub(duration - 1);
        let first = first_time
            .saturating_sub(schedule.phase)
            .div_ceil(schedule.period);
        if first > last {
            return Ok(Vec::new());
        }
        let slots = self
            .sites
            .iter()
            .filter(|(s, _)| !self.removed_sites.contains(*s))
            .try_fold(0usize, |a, (_, n)| a.checked_add(*n as usize))
            .ok_or("site capacity overflow")?;
        if slots == 0 {
            return Ok(Vec::new());
        }
        let count = last
            .checked_sub(first)
            .and_then(|v| v.checked_add(1))
            .ok_or("cohort capacity overflow")?;
        let count = usize::try_from(count).map_err(|_| "cohort capacity overflow")?;
        if count
            .checked_mul(slots)
            .ok_or("candidate capacity overflow")?
            > limit
        {
            return Err("production candidate budget exceeded");
        }
        let mut result = Vec::new();
        for cohort in first..=last {
            for (site, slots) in &self.sites {
                for slot in 0..*slots {
                    let id = FruitId {
                        site: site.clone(),
                        cohort,
                        slot,
                    };
                    if let Some(stage) = self.stage(schedule, &id)
                        && stage != Stage::Dropped
                    {
                        result.push((id, stage));
                    }
                }
            }
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn individual_harvest_pruning_overlap_and_roundtrip() {
        let s = Schedule {
            period: 10,
            phase: 0,
            maturity: 0,
            flower_duration: 2,
            fruit_duration: 2,
            ripe_duration: 12,
        };
        let mut p = Production {
            plant: Individual {
                born: 0,
                time: 14,
                dead: false,
                harvested: BTreeSet::new(),
            },
            sites: BTreeMap::from([("a".into(), 2), ("b".into(), 1)]),
            site_births: BTreeMap::new(),
            removed_sites: BTreeSet::new(),
            harvested: BTreeSet::new(),
        };
        assert_eq!(p.attached(&s, 6).unwrap().len(), 6);
        let id = FruitId {
            site: "a".into(),
            cohort: 0,
            slot: 0,
        };
        p.harvest(&s, id.clone()).unwrap();
        assert_eq!(p.attached(&s, 6).unwrap().len(), 5);
        p.remove_site("b").unwrap();
        let p: Production = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert!(p.stage(&s, &id).is_none());
        assert_eq!(p.attached(&s, 6).unwrap().len(), 3);
        assert!(p.attached(&s, 1).is_err());
    }
}
