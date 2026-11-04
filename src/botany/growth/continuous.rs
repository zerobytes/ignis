//! Explicit grammar4: independent size curves and metric, renewing leaf nodes.
use super::*;
use glam::DVec3 as V;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub seedling_height_m: f64,
    pub height_limit_m: f64,
    pub height_reference_years: f64,
    pub seedling_diameter_m: f64,
    pub diameter_limit_m: f64,
    pub diameter_reference_years: f64,
    pub internode_m: f64,
    pub internode_days: u64,
    pub shoot_expansion_days: u64,
    pub leaf_turnover_days: u64,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            seedling_height_m: 0.6,
            height_limit_m: 8.5,
            height_reference_years: 90.,
            seedling_diameter_m: 0.025,
            diameter_limit_m: 0.28,
            diameter_reference_years: 250.,
            internode_m: 0.12,
            internode_days: 14,
            shoot_expansion_days: 120,
            leaf_turnover_days: 730,
        }
    }
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        let values = [
            self.seedling_height_m,
            self.height_limit_m,
            self.height_reference_years,
            self.seedling_diameter_m,
            self.diameter_limit_m,
            self.diameter_reference_years,
            self.internode_m,
        ];
        if values.iter().any(|x| !x.is_finite() || *x <= 0.)
            || self.height_limit_m < self.seedling_height_m
            || self.diameter_limit_m < self.seedling_diameter_m
            || !(1..=MAX_TIME).contains(&self.internode_days)
            || !(1..=MAX_TIME).contains(&self.shoot_expansion_days)
            || !(1..=MAX_TIME).contains(&self.leaf_turnover_days)
            || !self.diameter_limit_m.powi(2).is_finite()
            || self.seedling_diameter_m.powi(2) == 0.
            || self.height_reference_years / 19. == 0.
            || self.internode_m < 16. * f64::EPSILON * self.height_limit_m.max(10.)
        {
            return Err(invalid("Invalid independent growth profile"));
        }
        if self.diameter_limit_m > self.seedling_diameter_m
            && (!self.area_tau().is_finite() || self.area_tau() <= 0.)
        {
            return Err(invalid(
                "Diameter reference is unresolved at numeric precision",
            ));
        }
        Ok(())
    }
    fn fraction(age: u64, tau: f64) -> f64 {
        let a = age as f64 / 365.;
        a / (a + tau)
    }
    pub fn height(&self, age: u64) -> f64 {
        self.seedling_height_m
            + (self.height_limit_m - self.seedling_height_m)
                * Self::fraction(age, self.height_reference_years / 19.)
    }
    fn area_tau(&self) -> f64 {
        let d =
            self.seedling_diameter_m + 0.95 * (self.diameter_limit_m - self.seedling_diameter_m);
        let f = (d * d - self.seedling_diameter_m.powi(2))
            / (self.diameter_limit_m.powi(2) - self.seedling_diameter_m.powi(2));
        self.diameter_reference_years * (1. / f - 1.)
    }
    pub fn diameter(&self, age: u64) -> f64 {
        if self.seedling_diameter_m == self.diameter_limit_m {
            return self.seedling_diameter_m;
        }
        let d2 = self.seedling_diameter_m.powi(2)
            + (self.diameter_limit_m.powi(2) - self.seedling_diameter_m.powi(2))
                * Self::fraction(age, self.area_tau());
        d2.sqrt()
    }
    pub(super) fn apply(&self, r: &mut Recipe, plant: &Plant) -> Result<()> {
        let age = plant.production.plant.time - plant.production.plant.born;
        let original = r.source.organs.clone();
        let root = &original[0];
        let Shape::Axis { radius, .. } = root.shape else {
            return Err(invalid("Missing continuous trunk"));
        };
        let h = self.height(age) / root.path.last().unwrap()[1];
        let diameter = self.diameter(age) * 0.5 / radius;
        let crown = 1. + 0.35 * Self::fraction(age, self.height_reference_years / 19.);
        if ![h, diameter, crown]
            .iter()
            .all(|x| x.is_finite() && *x > 0.)
        {
            return Err(invalid("Unresolved continuous dimensions"));
        }
        let source = Arc::make_mut(&mut r.source);
        let mut lookup: BTreeMap<String, usize> = BTreeMap::new();
        for (i, old) in original.iter().enumerate() {
            let mut organ = old.clone();
            if i == 0 {
                for p in &mut organ.path {
                    p[0] *= crown;
                    p[1] *= h;
                    p[2] *= crown;
                }
            } else {
                let parent = old
                    .parent
                    .as_ref()
                    .ok_or_else(|| invalid("Missing continuous parent"))?;
                let pi = *lookup
                    .get(parent)
                    .ok_or_else(|| invalid("Unordered continuous parent"))?;
                let old_parent: Vec<_> = original[pi]
                    .path
                    .iter()
                    .copied()
                    .map(V::from_array)
                    .collect();
                let new_parent: Vec<_> = source.organs[pi]
                    .path
                    .iter()
                    .copied()
                    .map(V::from_array)
                    .collect();
                let origin = V::from_array(old.path[0]);
                let anchor = bake::curve(&new_parent, old.insertion).0
                    + (origin - bake::curve(&old_parent, old.insertion).0) * crown;
                for p in &mut organ.path {
                    *p = (anchor + (V::from_array(*p) - origin) * crown).to_array();
                }
            }
            if let Shape::Axis { radius, .. } = &mut organ.shape {
                *radius *= diameter;
                if i > 0 {
                    let floor = if organ.id.contains("/t") {
                        0.0022
                    } else if organ.id.contains("/b") {
                        0.003
                    } else {
                        0.004
                    };
                    *radius = radius.max(floor);
                }
            }
            lookup.insert(old.id.clone(), i);
            source.organs[i] = organ;
        }
        let axes = source.organs.clone();
        let foliage = plant
            .definition
            .foliage
            .as_ref()
            .ok_or_else(|| invalid("Continuous foliage profile missing"))?;
        for axis in &axes {
            self.leaves(&mut source.organs, axis, plant, foliage)?;
        }
        r.validate()
    }
    fn birth(&self, id: &str, d: &Definition) -> Result<u64> {
        if id == "stem" {
            return Ok(0);
        }
        let mut parts = id.split('/');
        let i = parts
            .next()
            .and_then(|p| p.strip_prefix("node"))
            .and_then(|n| n.parse::<u64>().ok())
            .ok_or_else(|| invalid("Invalid shoot identity"))?;
        let mut birth = (i + 1) * d.interval;
        if let Some(p) = parts.next() {
            let j = p
                .strip_prefix('b')
                .and_then(|n| n.parse::<u64>().ok())
                .ok_or_else(|| invalid("Invalid secondary identity"))?;
            birth += self.shoot_expansion_days + j * self.shoot_expansion_days / 3;
        }
        if let Some(p) = parts.next() {
            let k = p
                .strip_prefix('t')
                .and_then(|n| n.parse::<u64>().ok())
                .ok_or_else(|| invalid("Invalid twig identity"))?;
            birth += self.shoot_expansion_days + k * self.shoot_expansion_days / 4;
        }
        Ok(birth)
    }
    fn axis_length(&self, id: &str, d: &Definition, day: u64) -> Result<f64> {
        let crown = 1. + 0.35 * Self::fraction(day, self.height_reference_years / 19.);
        let points: Vec<V> = if id == "stem" {
            let t = (day as f64 / (d.interval * (d.maximum_nodes as u64 + 8)) as f64).clamp(0., 1.);
            let maturity = t * t * (3. - 2. * t);
            (0..=12)
                .map(|i| {
                    let u = i as f64 / 12.;
                    V::new(
                        0.16 * u * u * maturity * crown,
                        self.height(day) * u,
                        0.12 * (u * 3.).sin() * u * maturity * crown,
                    )
                })
                .collect()
        } else {
            let parts: Vec<_> = id.split('/').collect();
            let i = parts[0]
                .strip_prefix("node")
                .and_then(|p| p.parse::<u64>().ok())
                .ok_or_else(|| invalid("Invalid node identity"))?;
            let birth = self.birth(id, d)?;
            let t =
                (day.saturating_sub(birth) as f64 / self.shoot_expansion_days as f64).clamp(0., 1.);
            let g = t * t * (3. - 2. * t);
            let (dy, base, extension, lift) = if parts.len() == 1 {
                let envelope = 0.7
                    + 0.5
                        * (std::f64::consts::PI * (i as f64 + 1.) / (d.maximum_nodes as f64 + 1.))
                            .sin();
                (
                    0.02 + 0.28 * tree::noise(d.seed, i + 40),
                    0.25,
                    2.6 * envelope * (0.8 + 0.35 * tree::noise(d.seed, i + 55)),
                    0.2 + 0.9 * tree::noise(d.seed, i + 70),
                )
            } else {
                let j = parts[1]
                    .strip_prefix('b')
                    .and_then(|p| p.parse::<u64>().ok())
                    .ok_or_else(|| invalid("Invalid branch identity"))?;
                if parts.len() == 2 {
                    (
                        0.08 + 0.8 * tree::noise(d.seed, i * 100 + j + 900),
                        0.16,
                        1.15,
                        0.35,
                    )
                } else {
                    let k = parts[2]
                        .strip_prefix('t')
                        .and_then(|p| p.parse::<u64>().ok())
                        .ok_or_else(|| invalid("Invalid twig identity"))?;
                    (
                        -0.12 + tree::noise(d.seed, i * 1000 + j * 10 + k + 8000),
                        0.08,
                        0.62,
                        0.12,
                    )
                }
            };
            let length = base + extension * g;
            (0..=8)
                .map(|i| {
                    let u = i as f64 / 8.;
                    V::new(length * u, dy * length * u + lift * g * u * u, 0.) * crown
                })
                .collect()
        };
        let mut previous = bake::curve(&points, 0.).0;
        let mut length = 0.;
        for i in 1..=128 {
            let point = bake::curve(&points, i as f64 / 128.).0;
            length += point.distance(previous);
            previous = point;
        }
        Ok(length)
    }
    fn emergence(
        &self,
        id: &str,
        d: &Definition,
        axis_birth: u64,
        age: u64,
        distance: f64,
        cache: &mut BTreeMap<u64, f64>,
    ) -> Result<u64> {
        let mut lo = axis_birth;
        let mut hi = age;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let length = if let Some(v) = cache.get(&mid) {
                *v
            } else {
                let v = self.axis_length(id, d, mid)?;
                cache.insert(mid, v);
                v
            };
            if length + 1e-9 >= distance {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        Ok(lo)
    }
    fn leaves(
        &self,
        out: &mut Vec<Organ>,
        axis: &Organ,
        plant: &Plant,
        foliage: &foliage::Profile,
    ) -> Result<()> {
        let age = plant.production.plant.time - plant.production.plant.born;
        let birth = self.birth(&axis.id, &plant.definition)?;
        let points: Vec<_> = axis.path.iter().copied().map(V::from_array).collect();
        // Same spline kernel as native bake; table in metres, not control-point index.
        const STEPS: usize = 128;
        let samples: Vec<_> = (0..=STEPS)
            .map(|i| bake::curve(&points, i as f64 / STEPS as f64).0)
            .collect();
        let mut arc = vec![0.];
        for p in samples.windows(2) {
            arc.push(arc.last().unwrap() + p[0].distance(p[1]));
        }
        let length = *arc.last().unwrap();
        let start = 0.04;
        let margin = 0.02;
        let available = if length >= start + margin {
            (((length - start - margin) / self.internode_m).floor() as u64).saturating_add(1)
        } else {
            0
        };
        let initial = if axis.id == "stem" {
            (((self.seedling_height_m - start - margin).max(0.) / self.internode_m).floor() as u64)
                .saturating_add(1)
        } else {
            1
        };
        let temporal = initial.saturating_add(age.saturating_sub(birth) / self.internode_days);
        let count = available.min(temporal);
        let limit = if axis.id == "stem" {
            foliage.apical_leaves
        } else if axis.id.contains("/t") {
            10
        } else if axis.id.contains("/b") {
            foliage.secondary_leaves
        } else {
            foliage.primary_leaves
        };
        let capacity = (limit as u64).div_ceil(2);
        let first = if axis.id == "stem" {
            count.saturating_sub(capacity)
        } else {
            0
        };
        let mut length_cache = BTreeMap::new();
        for node in first..count.min(first.saturating_add(capacity)) {
            let scheduled = if node < initial {
                birth
            } else {
                birth.saturating_add((node - initial + 1).saturating_mul(self.internode_days))
            };
            let distance = start + node as f64 * self.internode_m;
            let node_birth = scheduled.max(self.emergence(
                &axis.id,
                &plant.definition,
                birth,
                age,
                distance + margin,
                &mut length_cache,
            )?);
            if node_birth > age {
                continue;
            }
            let j = arc
                .partition_point(|v| *v < distance)
                .saturating_sub(1)
                .min(STEPS - 1);
            let u = (j as f64 + (distance - arc[j]) / (arc[j + 1] - arc[j])) / STEPS as f64;
            let origin = bake::curve(&points, u).0;
            let cohort = (age - node_birth) / self.leaf_turnover_days;
            let new_birth = node_birth + cohort * self.leaf_turnover_days;
            let first_cohort = if cohort > 0 && age - new_birth < foliage.expansion_days {
                cohort - 1
            } else {
                cohort
            };
            for cycle in first_cohort..=cohort {
                let leaf_birth = node_birth + cycle * self.leaf_turnover_days;
                let x = (age - leaf_birth) as f64 / foliage.expansion_days as f64;
                let fraction = x.clamp(0., 1.);
                let fraction = fraction * fraction * (3. - 2. * fraction);
                let (length, width) = foliage.dimensions(fraction);
                for side in 0..2u64 {
                    if (node - first) * 2 + side >= limit as u64 {
                        continue;
                    }
                    let angle = node as f64 * 2.3999632297
                        + cycle as f64 * 1.7
                        + side as f64 * std::f64::consts::PI
                        + (plant.definition.seed % 1000) as f64 * 0.01;
                    let end = origin + V::new(angle.cos() * length, 0.06, angle.sin() * length);
                    let mut leaf = axis_leaf(
                        format!("{}/n{node}/c{cycle}/l{side}", axis.id),
                        &axis.id,
                        u,
                        origin,
                        end,
                        width,
                    );
                    leaf.normal = [angle.cos() * 0.3, 1., angle.sin() * 0.3];
                    out.push(leaf);
                }
            }
        }
        Ok(())
    }
}
fn axis_leaf(id: String, parent: &str, u: f64, origin: V, end: V, width: f64) -> Organ {
    let mut leaf = super::axis(
        id,
        Some(parent.into()),
        u,
        vec![origin.to_array(), end.to_array()],
        0.001,
    );
    leaf.shape = Shape::Blade {
        width,
        thickness: 0.0003,
        fold: 0.12,
        cup: 0.06,
        outline: Outline::Strip {
            base_power: 0.55,
            tip_power: 1.2,
        },
    };
    leaf
}
