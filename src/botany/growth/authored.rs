//! Native chronology for an existing authored adult graph; no replacement crown grammar.
use super::*;
use glam::DVec3 as V;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub source: Arc<Source>,
    pub settings: Settings,
    pub reference_years: f64,
    pub seedling_height_m: f64,
    pub height_time_years: f64,
    pub crown_time_years: f64,
    pub radial_time_years: f64,
    pub order_delay_days: u64,
    pub expansion_days: u64,
    /// Native organ IDs, relative days since the individual's birth.
    pub sites: BTreeMap<String, u64>,
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        self.source.validate()?;
        if self.source.development.is_some()
            || !self.source.root_systems.is_empty()
            || self
                .source
                .organs
                .iter()
                .filter(|o| o.parent.is_none())
                .count()
                != 1
            || self.sites.is_empty()
            || self.sites.len() > 32
            || self.expansion_days == 0
            || self.expansion_days > MAX_TIME
            || self.order_delay_days > MAX_TIME
            || [
                self.reference_years,
                self.seedling_height_m,
                self.height_time_years,
                self.crown_time_years,
                self.radial_time_years,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.)
            || self.reference_years > MAX_TIME as f64 / 365.
            || self.sites.values().any(|t| *t == 0 || *t > MAX_TIME)
        {
            return Err(invalid("Invalid authored living profile"));
        }
        let root = self
            .source
            .organs
            .iter()
            .find(|o| o.parent.is_none())
            .unwrap();
        if !matches!(root.shape, Shape::Axis { .. })
            || root.path.last().unwrap()[1] - root.path[0][1] <= self.seedling_height_m
        {
            return Err(invalid(
                "Authored living profile requires an upright woody root axis",
            ));
        }
        for id in self.sites.keys() {
            if !self.source.organs.iter().any(|o| {
                o.id == *id
                    && o.parent.as_deref() == Some(root.id.as_str())
                    && matches!(o.shape, Shape::Axis { .. })
            }) {
                return Err(invalid(
                    "Authored production site must be a primary woody branch",
                ));
            }
        }
        // Reuse normal setting and shape admission, without recursive living data.
        let r = Recipe {
            version: VERSION,
            prefix: "authored_guard".into(),
            source: self.source.clone(),
            settings: self.settings.clone(),
            living: None,
            wood_details: None,
            edits: BTreeMap::new(),
            pruned: BTreeSet::new(),
            manual: BTreeMap::new(),
        };
        r.validate()
    }
    pub(super) fn births(&self, born: u64, time: u64) -> BTreeMap<String, u64> {
        self.sites
            .iter()
            .filter_map(|(id, t)| {
                born.checked_add(*t)
                    .filter(|t| *t <= time)
                    .map(|t| (id.clone(), t))
            })
            .collect()
    }
}
fn ratio(t: f64, reference: f64, tau: f64, start: f64) -> f64 {
    start + (1. - start) * (-(-t / tau).exp_m1()) / (-(-reference / tau).exp_m1())
}
fn progress(age: u64, birth: u64, duration: u64) -> f64 {
    let t = (age.saturating_sub(birth) as f64 / duration as f64).clamp(0., 1.);
    0.5 + 0.5 * t * t * (3. - 2. * t)
}
pub(super) fn recipe(plant: &Plant) -> Result<Recipe> {
    let d = &plant.definition;
    let p = d.authored.as_ref().unwrap();
    let age = plant.production.plant.time - plant.production.plant.born;
    let years = age as f64 / 365.;
    let root = &p.source.organs[0];
    let origin = V::from_array(root.path[0]);
    let height = root.path.last().unwrap()[1] - root.path[0][1];
    let h = ratio(
        years,
        p.reference_years,
        p.height_time_years,
        p.seedling_height_m / height,
    );
    let c = ratio(years, p.reference_years, p.crown_time_years, 0.08);
    let radial = ratio(years, p.reference_years, p.radial_time_years, 0.06);
    let mut organs: Vec<Organ> = vec![];
    let mut indices = BTreeMap::new();
    let mut times: BTreeMap<String, u64> = BTreeMap::new();
    let mut branches: BTreeMap<String, Option<String>> = BTreeMap::new();
    for o in &p.source.organs {
        let (mut birth, site) = if let Some(parent) = &o.parent {
            if !indices.contains_key(parent) {
                continue;
            }
            if let Some(t) = p.sites.get(&o.id) {
                (*t, Some(o.id.clone()))
            } else {
                let delay = if matches!(o.shape, Shape::Blade { .. }) {
                    20
                } else {
                    p.order_delay_days
                };
                (
                    times[parent].saturating_add(delay),
                    branches[parent].clone(),
                )
            }
        } else {
            (0, None)
        };
        // The existing organic junction kernel needs millimetric collars. Delay fine
        // axes until their radius is representable instead of thickening them or
        // weakening that kernel's admission guard. Sibling forks share this date.
        if o.parent.is_some()
            && matches!(o.connection, Connection::Graft | Connection::Fork)
            && let Shape::Axis { radius, .. } = o.shape
        {
            let required = radius.min(0.0011) / radius;
            let f =
                ((required - 0.06) / 0.94) * (-(-p.reference_years / p.radial_time_years).exp_m1());
            let ready = if f <= 0. {
                0
            } else if f >= 1. {
                MAX_TIME + 1
            } else {
                (-p.radial_time_years * (-f).ln_1p() * 365.).ceil() as u64
            };
            birth = birth.max(ready);
        }
        if age < birth
            || site
                .as_ref()
                .is_some_and(|s| plant.production.removed_sites.contains(s))
        {
            continue;
        }
        let g = progress(age, birth, p.expansion_days);
        let mut next = o.clone();
        if let Some(parent) = &o.parent {
            let source_parent = p.source.organs.iter().find(|q| q.id == *parent).unwrap();
            let parent_index: usize = indices[parent];
            let actual = &organs[parent_index];
            let old = super::super::bake::curve(
                &source_parent
                    .path
                    .iter()
                    .copied()
                    .map(V::from_array)
                    .collect::<Vec<_>>(),
                o.insertion,
            )
            .0;
            let new = super::super::bake::curve(
                &actual
                    .path
                    .iter()
                    .copied()
                    .map(V::from_array)
                    .collect::<Vec<_>>(),
                o.insertion,
            )
            .0;
            let woody = matches!(o.shape, Shape::Axis { .. });
            let scale = if woody {
                V::new(c, h, c) * g
            } else {
                V::splat((0.6 + 0.4 * (years / p.reference_years).min(1.)) * g)
            };
            next.path = o
                .path
                .iter()
                .map(|q| (new + (V::from_array(*q) - old) * scale).to_array())
                .collect();
        } else {
            next.path = o
                .path
                .iter()
                .map(|q| (origin + (V::from_array(*q) - origin) * V::new(c, h, c)).to_array())
                .collect();
        }
        match &mut next.shape {
            Shape::Axis { radius, .. } => *radius *= radial,
            Shape::Blade {
                width, thickness, ..
            } => {
                *width *= (0.6 + 0.4 * (years / p.reference_years).min(1.)) * g;
                *thickness *= g;
            }
        }
        next.life = None;
        indices.insert(o.id.clone(), organs.len());
        times.insert(o.id.clone(), birth);
        branches.insert(o.id.clone(), site);
        organs.push(next);
    }
    // Leaves accompany the first stem and growing branch frontiers, before the adult sprays emerge.
    let frontiers: Vec<_> = organs
        .iter()
        .filter(|o| {
            matches!(o.shape, Shape::Axis { .. })
                && ((o.parent.is_none() && years < p.reference_years)
                    || !organs
                        .iter()
                        .any(|q| q.parent.as_deref() == Some(o.id.as_str())))
        })
        .cloned()
        .collect();
    for host in frontiers {
        for k in 0..6 {
            let u = if host.parent.is_none() {
                0.8 + 0.03 * k as f64
            } else {
                0.45 + 0.08 * k as f64
            };
            let anchor = super::super::bake::curve(
                &host
                    .path
                    .iter()
                    .copied()
                    .map(V::from_array)
                    .collect::<Vec<_>>(),
                u,
            )
            .0;
            let angle = k as f64 * 2.399963 + (d.seed % 1024) as f64;
            let tip = anchor + V::new(angle.cos(), 0.35, angle.sin()) * 0.12;
            let mut leaf = axis(
                format!("{}/juvenile{k}", host.id),
                Some(host.id.clone()),
                u,
                vec![anchor.to_array(), tip.to_array()],
                0.001,
            );
            leaf.shape = Shape::Blade {
                width: 0.075,
                thickness: 0.0003,
                fold: 0.055,
                cup: 0.055,
                outline: Outline::Strip {
                    base_power: 0.48,
                    tip_power: 1.15,
                },
            };
            leaf.normal = [-angle.sin(), 1., angle.cos()];
            organs.push(leaf);
        }
    }
    let r = Recipe {
        version: VERSION,
        prefix: d.prefix.clone(),
        source: Arc::new(Source {
            version: p.source.version.max(8),
            name: format!("{} · living", p.source.name),
            seed: p.source.seed,
            organs,
            development: None,
            root_systems: vec![],
        }),
        settings: p.settings.clone(),
        living: None,
        wood_details: None,
        edits: BTreeMap::new(),
        pruned: BTreeSet::new(),
        manual: BTreeMap::new(),
    };
    r.validate()?;
    Ok(r)
}

/// Small command payload; the adult source remains in native authoring memory.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Timing {
    pub reference_years: f64,
    pub seedling_height_m: f64,
    pub height_time_years: f64,
    pub crown_time_years: f64,
    pub radial_time_years: f64,
    pub branch_interval_days: u64,
    pub order_delay_days: u64,
    pub expansion_days: u64,
}
impl Default for Timing {
    fn default() -> Self {
        Self {
            reference_years: 20.,
            seedling_height_m: 0.55,
            height_time_years: 35.,
            crown_time_years: 35.,
            radial_time_years: 35.,
            branch_interval_days: 200,
            order_delay_days: 120,
            expansion_days: 730,
        }
    }
}
pub fn adopt(old: &Recipe, t: &Timing, born: u64) -> Result<Plant> {
    if old.living.is_some()
        || !old.edits.is_empty()
        || !old.pruned.is_empty()
        || !old.manual.is_empty()
        || old.wood_details.is_some()
    {
        return Err(invalid(
            "Adoption requires an unmodified static source; retain edited source separately",
        ));
    }
    if t.branch_interval_days == 0 {
        return Err(invalid("Branch interval must be positive"));
    }
    let root = old
        .source
        .organs
        .iter()
        .find(|o| o.parent.is_none())
        .ok_or_else(|| invalid("Missing source root"))?;
    let mut sites = BTreeMap::new();
    for o in old.source.organs.iter().filter(|o| {
        o.parent.as_deref() == Some(root.id.as_str()) && matches!(o.shape, Shape::Axis { .. })
    }) {
        let day = (sites.len() as u64 + 1)
            .checked_mul(t.branch_interval_days)
            .filter(|n| *n <= MAX_TIME)
            .ok_or_else(|| invalid("Authored branch chronology exceeds time budget"))?;
        sites.insert(o.id.clone(), day);
    }
    let definition = Definition {
        version: 9,
        prefix: old.prefix.clone(),
        seed: old.source.seed,
        maximum_nodes: sites.len() as u16,
        interval: t.branch_interval_days,
        schedule: Schedule {
            period: 365,
            phase: 0,
            maturity: 0,
            flower_duration: 0,
            fruit_duration: 0,
            ripe_duration: 0,
        },
        authored: Some(Arc::new(Profile {
            source: old.source.clone(),
            settings: old.settings.clone(),
            reference_years: t.reference_years,
            seedling_height_m: t.seedling_height_m,
            height_time_years: t.height_time_years,
            crown_time_years: t.crown_time_years,
            radial_time_years: t.radial_time_years,
            order_delay_days: t.order_delay_days,
            expansion_days: t.expansion_days,
            sites,
        })),
        ..Default::default()
    };
    Plant::new(definition, born)
}
