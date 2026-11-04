//! Persistent calendar cohorts; independent of rendering and mechanical stepping.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub period: u64,
    pub phase: u64,
    pub maturity: u64,
    pub flower_duration: u64,
    pub fruit_duration: u64,
    pub ripe_duration: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    Flower,
    GrowingFruit,
    RipeFruit,
    Dropped,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Individual {
    pub born: u64,
    pub time: u64,
    pub dead: bool,
    pub harvested: BTreeSet<u64>,
}
impl Schedule {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.period == 0 {
            return Err("zero production period");
        }
        self.flower_duration
            .checked_add(self.fruit_duration)
            .and_then(|x| x.checked_add(self.ripe_duration))
            .ok_or("duration overflow")?;
        Ok(())
    }
    pub(crate) fn start(&self, cohort: u64) -> Option<u64> {
        cohort.checked_mul(self.period)?.checked_add(self.phase)
    }
    pub fn stage(&self, plant: &Individual, cohort: u64) -> Option<Stage> {
        self.validate().ok()?;
        if plant.dead || plant.harvested.contains(&cohort) {
            return None;
        }
        let start = self.start(cohort)?;
        if start < plant.born.checked_add(self.maturity)? {
            return None;
        }
        let age = plant.time.checked_sub(start)?;
        Some(if age < self.flower_duration {
            Stage::Flower
        } else if age < self.flower_duration + self.fruit_duration {
            Stage::GrowingFruit
        } else if age < self.flower_duration + self.fruit_duration + self.ripe_duration {
            Stage::RipeFruit
        } else {
            Stage::Dropped
        })
    }
}
impl Individual {
    pub fn advance(&mut self, time: u64) -> Result<(), &'static str> {
        if time < self.time || time < self.born {
            return Err("time cannot rewind");
        }
        self.time = time;
        Ok(())
    }
    pub fn harvest(&mut self, schedule: &Schedule, cohort: u64) -> Result<(), &'static str> {
        if schedule.stage(self, cohort) != Some(Stage::RipeFruit) {
            return Err("fruit is not ripe and attached");
        }
        self.harvested.insert(cohort);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn harvest_survives_save_and_next_cycle() {
        let s = Schedule {
            period: 360,
            phase: 80,
            maturity: 0,
            flower_duration: 14,
            fruit_duration: 45,
            ripe_duration: 30,
        };
        let mut p = Individual {
            born: 0,
            time: 80,
            dead: false,
            harvested: BTreeSet::new(),
        };
        assert_eq!(s.stage(&p, 0), Some(Stage::Flower));
        p.advance(145).unwrap();
        p.harvest(&s, 0).unwrap();
        let mut p: Individual = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(s.stage(&p, 0), None);
        p.advance(505).unwrap();
        assert_eq!(s.stage(&p, 1), Some(Stage::RipeFruit));
        assert_eq!(s.stage(&p, 0), None);
        let before = p.time;
        assert!(p.advance(2).is_err());
        assert_eq!(p.time, before);
    }
    #[test]
    fn maturity_checked_at_birth_of_cohort() {
        let s = Schedule {
            period: 360,
            phase: 80,
            maturity: 100,
            flower_duration: 14,
            fruit_duration: 45,
            ripe_duration: 30,
        };
        let p = Individual {
            born: 0,
            time: 145,
            dead: false,
            harvested: BTreeSet::new(),
        };
        assert_eq!(s.stage(&p, 0), None);
    }
}
