//! Small native growth grammar. Evaluates new organs from persistent individual time.
//! Scene ownership and automatic gameplay ticking remain separate integration work.
use super::{lifecycle::Schedule, production::Production, *};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub prefix: String,
    pub seed: u64,
    pub interval: u64,
    pub maximum_nodes: u16,
    pub internode_length: f64,
    pub schedule: Schedule,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plant {
    pub definition: Definition,
    pub production: Production,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.schedule.validate().map_err(invalid)?;
        if !identifier(&self.prefix)
            || self.prefix.len() > 20
            || self.interval == 0
            || !(1..=32).contains(&self.maximum_nodes)
            || !self.internode_length.is_finite()
            || !(0.3..=1.).contains(&self.internode_length)
        {
            return Err(invalid("Invalid living growth definition"));
        }
        Ok(())
    }
}
fn axis(
    id: String,
    parent: Option<String>,
    insertion: f64,
    path: Vec<[f64; 3]>,
    radius: f64,
) -> Organ {
    Organ {
        id,
        parent,
        insertion,
        path,
        normal: [0., 0., 1.],
        life: None,
        twist: 0.,
        shape: Shape::Axis {
            radius,
            tip_ratio: 0.6,
            ellipticity: 1.,
        },
        taper_power: 1.,
        connection: Connection::Graft,
        birth: 0.,
        duration: 1.,
        density: 650.,
        young: 1e8,
        shear: 2e7,
    }
}
impl Plant {
    pub fn recipe(&self) -> Result<Recipe> {
        let d = &self.definition;
        d.validate()?;
        let p = &self.production.plant;
        let age = p
            .time
            .checked_sub(p.born)
            .ok_or_else(|| invalid("Plant time precedes birth"))?;
        if !p.harvested.is_empty() {
            return Err(invalid("Living plants require individual fruit tombstones"));
        }
        let nodes = (age / d.interval).min(d.maximum_nodes as u64) as u16;
        let height =
            0.5 + (age as f64 / d.interval as f64).min(d.maximum_nodes as f64) * d.internode_length;
        let mut organs = vec![axis(
            "stem".into(),
            None,
            0.,
            vec![[0.; 3], [0., height, 0.]],
            0.025,
        )];
        for i in 0..nodes {
            let id = format!("node{i}");
            if self.production.removed_sites.contains(&id) {
                continue;
            }
            let birth = (i as u64 + 1)
                .checked_mul(d.interval)
                .ok_or_else(|| invalid("Bud birth overflow"))?;
            let u = ((age - birth) as f64 / d.interval as f64).clamp(0., 1.);
            let g = u * u * (3. - 2. * u);
            let y = 0.3 + i as f64 * d.internode_length;
            let angle = i as f64 * 2.3999632297 + (d.seed % 65536) as f64 * 0.0001;
            let (sin, cos) = angle.sin_cos();
            let length = 0.24 + 0.5 * g;
            let root = [0., y, 0.];
            let tip = [cos * length, y + 0.2 * length, sin * length];
            organs.push(axis(
                id.clone(),
                Some("stem".into()),
                y / height,
                vec![root, tip],
                0.004 + 0.003 * g,
            ));
            for side in [-1., 1.] {
                let leaf_id = format!("{id}/leaf{}", if side < 0. { 0 } else { 1 });
                let start = [tip[0] * 0.7, y + (tip[1] - y) * 0.7, tip[2] * 0.7];
                let l = 0.08 + 0.16 * g;
                let end = [
                    start[0] - sin * l * side,
                    start[1] + 0.04,
                    start[2] + cos * l * side,
                ];
                let mut leaf = axis(leaf_id, Some(id.clone()), 0.7, vec![start, end], 0.0012);
                leaf.shape = Shape::Blade {
                    width: 0.03 + 0.05 * g,
                    thickness: 0.0003,
                    fold: 0.05,
                    cup: 0.04,
                    outline: Outline::Strip {
                        base_power: 0.5,
                        tip_power: 1.,
                    },
                };
                leaf.normal = [cos, 0., sin];
                organs.push(leaf);
            }
        }
        let r = Recipe {
            version: VERSION,
            prefix: d.prefix.clone(),
            source: Arc::new(Source {
                version: VERSION,
                name: "Native growing specimen".into(),
                seed: d.seed,
                organs,
                development: None,
                root_systems: vec![],
            }),
            settings: Settings {
                axis_samples: 12,
                blade_samples: 4,
                ..Default::default()
            },
            edits: BTreeMap::new(),
            pruned: BTreeSet::new(),
            manual: BTreeMap::new(),
        };
        r.validate()?;
        Ok(r)
    }
    /// Build and validate a complete candidate before committing time or new sites.
    pub fn advance(&mut self, time: u64) -> Result<Recipe> {
        if self.production.plant.dead {
            return Err(invalid("Dead plants do not develop"));
        }
        let mut candidate = self.clone();
        candidate.production.plant.advance(time).map_err(invalid)?;
        let recipe = candidate.recipe()?;
        for organ in &recipe.source.organs {
            if organ.parent.as_deref() == Some("stem") {
                candidate
                    .production
                    .sites
                    .entry(organ.id.clone())
                    .or_insert(1);
            }
        }
        for i in 0..candidate.definition.maximum_nodes {
            let id = format!("node{i}");
            if candidate.production.sites.contains_key(&id) {
                let offset = (i as u64 + 1)
                    .checked_mul(candidate.definition.interval)
                    .ok_or_else(|| invalid("Site birth overflow"))?;
                let birth = candidate
                    .production
                    .plant
                    .born
                    .checked_add(offset)
                    .ok_or_else(|| invalid("Site birth overflow"))?;
                candidate.production.site_births.entry(id).or_insert(birth);
            }
        }
        bake::compile(&recipe)?;
        *self = candidate;
        Ok(recipe)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn specimen() -> Plant {
        Plant {
            definition: Definition {
                prefix: "living".into(),
                seed: 42,
                interval: 10,
                maximum_nodes: 8,
                internode_length: 0.45,
                schedule: Schedule {
                    period: 100,
                    phase: 80,
                    maturity: 0,
                    flower_duration: 5,
                    fruit_duration: 5,
                    ripe_duration: 10,
                },
            },
            production: Production {
                plant: lifecycle::Individual {
                    born: 0,
                    time: 0,
                    dead: false,
                    harvested: BTreeSet::new(),
                },
                sites: BTreeMap::new(),
                site_births: BTreeMap::new(),
                removed_sites: BTreeSet::new(),
                harvested: BTreeSet::new(),
            },
        }
    }
    #[test]
    fn native_growth_roundtrip_and_partitioned_advance() {
        let mut direct = specimen();
        let young = direct.advance(10).unwrap();
        let old = direct.advance(40).unwrap();
        assert!(old.source.organs.len() > young.source.organs.len());
        let mut steps = specimen();
        for t in [10, 20, 30, 40] {
            steps.advance(t).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&direct).unwrap(),
            serde_json::to_value(&steps).unwrap()
        );
        let mut restored: Plant =
            serde_json::from_str(&serde_json::to_string(&direct).unwrap()).unwrap();
        restored.production.remove_site("node0").unwrap();
        let r = restored.advance(50).unwrap();
        assert!(!r.source.organs.iter().any(|o| o.id.starts_with("node0")));
        assert!(r.source.organs.iter().any(|o| o.id == "node4"));
        let before = serde_json::to_value(&restored).unwrap();
        assert!(restored.advance(20).is_err());
        assert_eq!(before, serde_json::to_value(&restored).unwrap());
        let a = bake::compile(&restored.recipe().unwrap()).unwrap();
        assert!(
            a.document
                .assets
                .iter()
                .any(|a| a.geometry.triangles.len() > 10)
        );
    }
    #[test]
    fn growth_sites_do_not_retroactively_produce_and_harvest_persists() {
        let mut p = specimen();
        p.definition.schedule.phase = 0;
        p.definition.schedule.flower_duration = 5;
        p.definition.schedule.fruit_duration = 5;
        p.definition.schedule.ripe_duration = 90;
        p.advance(90).unwrap();
        let old = production::FruitId {
            site: "node0".into(),
            cohort: 0,
            slot: 0,
        };
        assert_eq!(p.production.stage(&p.definition.schedule, &old), None);
        p.advance(110).unwrap();
        let current = production::FruitId {
            site: "node0".into(),
            cohort: 1,
            slot: 0,
        };
        p.production
            .harvest(&p.definition.schedule, current.clone())
            .unwrap();
        let mut restored: Plant =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        restored.advance(210).unwrap();
        assert_eq!(
            restored
                .production
                .stage(&restored.definition.schedule, &current),
            None
        );
        let next = production::FruitId {
            cohort: 2,
            ..current
        };
        assert_eq!(
            restored
                .production
                .stage(&restored.definition.schedule, &next),
            Some(lifecycle::Stage::RipeFruit)
        );
    }
}
