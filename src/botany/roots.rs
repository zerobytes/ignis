//! Seeded, authored root axes and a shared basal crown; no terrain/soil simulation.
use super::*;
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    Radial,
    Taproot,
    Fibrous,
}
/// Optional wall/rounded-crest cross section. Absence is the exact v5 tube.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Buttress {
    pub height: f64,
    pub reach: f64,
    pub wall_width: f64,
    pub crest_width: f64,
    pub meander: f64,
    pub fluting: f64,
}
impl Default for Buttress {
    fn default() -> Self {
        Self {
            height: 3.,
            reach: 6.,
            wall_width: 0.25,
            crest_width: 0.5,
            meander: 0.2,
            fluting: 0.8,
        }
    }
}
impl Buttress {
    fn validate(&self) -> Result<()> {
        if !finite(&[
            self.height,
            self.reach,
            self.wall_width,
            self.crest_width,
            self.meander,
            self.fluting,
        ]) || !(0. ..=8.).contains(&self.height)
            || !(1. ..=30.).contains(&self.reach)
            || !(0.08..=1.).contains(&self.wall_width)
            || !(0.08..=1.5).contains(&self.crest_width)
            || !(0. ..=0.35).contains(&self.meander)
            || !(0. ..=2.).contains(&self.fluting)
        {
            return Err(invalid("Invalid bounded buttress profile"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttress: Option<Buttress>,
    pub kind: Kind,
    pub count: u8,
    pub seed: u64,
    /// Lengths are multiples of the unedited source stem radius.
    pub spread: f64,
    pub depth: f64,
    pub radius_ratio: f64,
    pub neck_height: f64,
    pub exposure: f64,
    pub soil_height: f64,
    pub soil_slope: [f64; 2],
    pub orders: u8,
    pub length_ratio: f64,
    pub branch_angle: f64,
    pub irregularity: f64,
    /// Explicit distal lattice anchors; not a constitutive model of soil.
    pub anchored_fraction: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            buttress: None,
            kind: Kind::Radial,
            count: 7,
            seed: 0,
            spread: 8.,
            depth: 0.8,
            radius_ratio: 0.48,
            neck_height: 1.25,
            exposure: 0.7,
            soil_height: 0.,
            soil_slope: [0.; 2],
            orders: 2,
            length_ratio: 0.65,
            branch_angle: 0.60,
            irregularity: 0.12,
            anchored_fraction: 0.3,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if let Some(form) = &self.buttress {
            form.validate()?;
            if self.kind != Kind::Radial {
                return Err(invalid("Buttresses require radial roots"));
            }
        }
        if !(1..=12).contains(&self.count)
            || self.orders > 3
            || !finite(&[
                self.spread,
                self.depth,
                self.radius_ratio,
                self.neck_height,
                self.exposure,
                self.soil_height,
                self.soil_slope[0],
                self.soil_slope[1],
                self.length_ratio,
                self.branch_angle,
                self.irregularity,
                self.anchored_fraction,
            ])
            || !(2. ..=40.).contains(&self.spread)
            || !(0.1..=40.).contains(&self.depth)
            || !(0.03..=0.9).contains(&self.radius_ratio)
            || !(0.5..=5.).contains(&self.neck_height)
            || !(0. ..=1.).contains(&self.exposure)
            || !(0.3..=0.85).contains(&self.length_ratio)
            || !(0.2..=1.2).contains(&self.branch_angle)
            || !(0. ..=0.35).contains(&self.irregularity)
            || !(0. ..=1.).contains(&self.anchored_fraction)
            || self.soil_slope.iter().any(|s| s.abs() > 1.)
            || (self.kind == Kind::Taproot && self.count != 1)
        {
            return Err(invalid("Invalid bounded root system settings"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub stem: String,
    pub settings: Settings,
}
impl System {
    pub fn contains(&self, id: &str) -> bool {
        id.strip_prefix(&self.stem)
            .is_some_and(|tail| tail.starts_with("/root"))
    }
}
pub fn system_for<'a>(source: &'a Source, id: &str) -> Option<&'a System> {
    source.root_systems.iter().find(|s| s.contains(id))
}
fn unit(v: DVec3) -> DVec3 {
    v.try_normalize().unwrap_or(DVec3::X)
}
fn random(seed: u64, id: &str, salt: u64) -> f64 {
    let mut h = seed ^ salt;
    for x in id.bytes() {
        h = (h ^ u64::from(x)).wrapping_mul(1099511628211);
    }
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    (h >> 11) as f64 / (1u64 << 53) as f64
}
pub fn configure(recipe: &Recipe, stem: &str, settings: Option<Settings>) -> Result<Recipe> {
    configure_with_policy(recipe, stem, settings, ManualPolicy::Preserve)
}
pub fn configure_with_policy(
    recipe: &Recipe,
    stem: &str,
    settings: Option<Settings>,
    policy: ManualPolicy,
) -> Result<Recipe> {
    if let Some(s) = &settings {
        s.validate()?;
    }
    let mut next = recipe.clone();
    let version = if settings.as_ref().is_some_and(|s| s.buttress.is_some()) {
        7
    } else {
        5
    };
    next.version = next.version.max(version);
    let source = Arc::make_mut(&mut next.source);
    source.version = source.version.max(version);
    let _index = source
        .organs
        .iter()
        .position(|o| o.id == stem && o.parent.is_none() && matches!(o.shape, Shape::Axis { .. }))
        .ok_or_else(|| invalid("Root crown requires an authored top-level axis"))?;
    let old = source.root_systems.iter().find(|s| s.stem == stem).cloned();
    if let Some(old) = old {
        let mut removed = BTreeSet::new();
        for o in &source.organs {
            if (o.parent.as_deref() == Some(&old.stem) && o.connection == Connection::Basal)
                || o.parent.as_ref().is_some_and(|p| removed.contains(p))
            {
                removed.insert(o.id.clone());
            }
        }
        source.organs.retain(|o| !removed.contains(&o.id));
        source.root_systems.retain(|s| s.stem != stem);
    }
    if let Some(settings) = settings {
        if source.root_systems.len() >= 16 {
            return Err(invalid("At most16 root crowns per source"));
        }
        let base = source.organs.iter().find(|o| o.id == stem).unwrap().clone();
        let Shape::Axis { radius, .. } = base.shape else {
            unreachable!()
        };
        let system = System {
            stem: stem.into(),
            settings,
        };
        if source.organs.iter().any(|o| system.contains(&o.id)) {
            return Err(invalid("Root IDs collide with existing authored organs"));
        }
        let center = DVec3::from_array(base.path[0]);
        let s = &system.settings;
        let seed = source.seed ^ s.seed;
        let total = s.count as usize * ((1usize << (s.orders + 1)) - 1);
        if source.organs.len() + total > MAX_ORGANS {
            return Err(invalid("Root generation exceeds organ budget"));
        }
        struct Grow<'a> {
            out: &'a mut Vec<Organ>,
            s: &'a Settings,
            seed: u64,
            center: DVec3,
            r: f64,
            chronology: bool,
        }
        struct Bud {
            id: String,
            parent: String,
            start: DVec3,
            az: f64,
            length: f64,
            radius: f64,
            remaining: u8,
            progress: f64,
            level: u8,
        }
        impl Grow<'_> {
            fn grow(&mut self, bud: Bud) {
                let Bud {
                    id,
                    parent,
                    start,
                    az,
                    length,
                    radius,
                    remaining,
                    progress,
                    level,
                } = bud;
                let end_progress = if remaining == 0 {
                    1.
                } else {
                    progress + (1. - progress) * 0.5
                };
                let mut path = Vec::with_capacity(9);
                let jitter = self.s.irregularity * (random(self.seed, &id, 4) - 0.5) * 2.;
                for k in 0..=8 {
                    let u = k as f64 / 8.;
                    let a = az + jitter * u;
                    let p = start + DVec3::new(a.cos(), 0., a.sin()) * (length * u);
                    let p = if let Some(form) = &self.s.buttress {
                        let phase = random(self.seed, &id, 19) * std::f64::consts::TAU;
                        let delay = if level == 0 { 0.55 } else { 0.20 };
                        let q = ((u - delay) / (1. - delay)).clamp(0., 1.);
                        let wave = (std::f64::consts::PI * q).sin().powi(2) * phase.sin();
                        p + DVec3::new(-az.sin(), 0., az.cos()) * (length * form.meander * wave)
                    } else {
                        p
                    };
                    let q = progress + (end_progress - progress) * u;
                    let soil = self.center.y
                        + self.s.soil_height
                        + self.s.soil_slope[0] * (p.x - self.center.x)
                        + self.s.soil_slope[1] * (p.z - self.center.z);
                    let expose = if self.s.kind == Kind::Radial && self.s.buttress.is_none() {
                        self.s.exposure * radius * (1. - (-q * 8.).exp())
                    } else {
                        0.
                    };
                    let target = soil + expose
                        - self.r
                            * self.s.depth
                            * if self.s.kind == Kind::Taproot {
                                q
                            } else {
                                q * q
                            };
                    let target0 = self.center.y
                        + self.s.soil_height
                        + self.s.soil_slope[0] * (start.x - self.center.x)
                        + self.s.soil_slope[1] * (start.z - self.center.z)
                        + if self.s.kind == Kind::Radial && self.s.buttress.is_none() {
                            self.s.exposure * radius * (1. - (-progress * 8.).exp())
                        } else {
                            0.
                        }
                        - self.r
                            * self.s.depth
                            * if self.s.kind == Kind::Taproot {
                                progress
                            } else {
                                progress * progress
                            };
                    let mut p = p;
                    p.y = target + (start.y - target0) * (1. - u);
                    if k == 0 {
                        p = start;
                    }
                    path.push(p.to_array());
                }
                if self.s.kind == Kind::Taproot && level == 0 {
                    for (k, p) in path.iter_mut().enumerate() {
                        let u = k as f64 / 8.;
                        p[0] = start.x + length * 0.04 * (az + u).sin() * u;
                        p[2] = start.z + length * 0.04 * (az + u).cos() * u;
                    }
                }
                let end = DVec3::from_array(*path.last().unwrap());
                let direction = unit(DVec3::from_array(path[1]) - start);
                let normal = super::bake::normal(direction, DVec3::Y);
                self.out.push(Organ {
                    id: id.clone(),
                    parent: Some(parent),
                    insertion: if level == 0 { 0. } else { 1. },
                    path,
                    normal: normal.to_array(),
                    twist: 0.,
                    shape: Shape::Axis {
                        radius: radius.max(0.0012),
                        tip_ratio: if remaining > 0 { 0.8 } else { 0.08 },
                        ellipticity: 1.,
                    },
                    taper_power: 1.2,
                    connection: if level == 0 {
                        Connection::Basal
                    } else {
                        Connection::Fork
                    },
                    birth: 0.,
                    duration: 2.,
                    density: 650.,
                    young: 3e9,
                    shear: 6e8,
                    life: self.chronology.then_some(development::OrganLife {
                        born_years: level as f64 * 1.2,
                        expansion_years: 2.,
                        role: development::Role::Root,
                    }),
                });
                if remaining > 0 {
                    for j in 0..2 {
                        let turn = self.s.branch_angle * (if j == 0 { -1. } else { 1. });
                        self.grow(Bud {
                            id: format!("{id}/f{j}"),
                            parent: id.clone(),
                            start: end,
                            az: az + turn,
                            length: length * self.s.length_ratio,
                            radius: radius * 0.8 * 0.5_f64.powf(1. / 2.3),
                            remaining: remaining - 1,
                            progress: end_progress,
                            level: level + 1,
                        });
                    }
                }
            }
        }
        let mut grow = Grow {
            out: &mut source.organs,
            s,
            seed,
            center,
            r: radius,
            chronology: source.development.is_some(),
        };
        for i in 0..s.count {
            let id = format!("{stem}/root{i}");
            let az = std::f64::consts::TAU
                * (i as f64 + 2. * s.irregularity * (random(seed, &id, 1) - 0.5))
                / s.count as f64;
            let len =
                radius * s.spread * (1. + s.irregularity * (2. * random(seed, &id, 2) - 1.) * 0.8);
            let rad = radius
                * s.radius_ratio
                * (1. + s.irregularity * (2. * random(seed, &id, 3) - 1.) * 1.5);
            grow.grow(Bud {
                id,
                parent: stem.into(),
                start: center,
                az,
                length: len * 0.55,
                radius: rad,
                remaining: s.orders,
                progress: 0.,
                level: 0,
            });
        }
        source.root_systems.push(system);
    }
    if policy == ManualPolicy::Discard {
        next.edits
            .retain(|id, _| source.organs.iter().any(|o| &o.id == id));
        next.pruned
            .retain(|id| source.organs.iter().any(|o| &o.id == id));
        next.manual.clear();
    }
    next.validate()?;
    Ok(next)
}
