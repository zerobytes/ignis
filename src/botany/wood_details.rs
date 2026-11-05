//! Authored root morphology and persistent branch-loss history. No fracture/soil solver.
use super::bake::Pose;
use super::wood_surface::{Control, Relief, Section, Surface};
use super::*;
#[path = "wood_details_ui.rs"]
pub mod ui;
use std::f64::consts::{PI, TAU};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct RootProfile {
    pub bulges: u8,
    pub swelling: f64,
    pub flattening: f64,
    pub bend: f64,
    pub twist: f64,
    pub grain: f64,
}
impl Default for RootProfile {
    fn default() -> Self {
        Self {
            bulges: 9,
            swelling: 0.38,
            flattening: 0.24,
            bend: 0.32,
            twist: 0.35,
            grain: 0.12,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    Pruning,
    Breakage,
    Historical,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scar {
    #[serde(default = "enabled")]
    pub visible: bool,
    pub id: String,
    pub parent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_organs: Vec<String>,
    /// Source metres along the parent, not a fraction of its growing length.
    pub distance_m: f64,
    pub angle: f64,
    pub radius_m: f64,
    pub raised_m: f64,
    pub depth_m: f64,
    pub year: f64,
    pub closing_years: f64,
    pub cause: Cause,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct NaturalLoss {
    pub first_year: f64,
    pub interval_years: f64,
    pub maximum_events: u8,
}
impl Default for NaturalLoss {
    fn default() -> Self {
        Self {
            first_year: 25.,
            interval_years: 18.,
            maximum_events: 8,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Details {
    pub seed: u64,
    pub age_years: f64,
    pub healing_years: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natural_loss: Option<NaturalLoss>,
    pub scars: Vec<Scar>,
}
impl Default for Details {
    fn default() -> Self {
        Self {
            seed: 42,
            age_years: 50.,
            healing_years: 20.,
            roots: None,
            natural_loss: None,
            scars: vec![],
        }
    }
}
fn finite_values(v: &[f64]) -> bool {
    v.iter().all(|v| v.is_finite())
}
fn organ_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 512 && s.is_ascii()
}
fn enabled() -> bool {
    true
}
fn unit(seed: u64, id: &str, k: u64) -> f64 {
    let mut v = seed ^ k.wrapping_mul(0x9e3779b97f4a7c15);
    for b in id.bytes() {
        v = (v ^ b as u64).wrapping_mul(0x100000001b3);
    }
    v ^= v >> 30;
    v = v.wrapping_mul(0xbf58476d1ce4e5b9);
    v ^= v >> 27;
    (v >> 11) as f64 / (1_u64 << 53) as f64
}
fn bump(x: f64) -> f64 {
    (1. - x * x).max(0.).powi(2)
}
pub fn age(recipe: &Recipe) -> f64 {
    if let Some(p) = &recipe.living {
        (p.production.plant.time - p.production.plant.born) as f64 / 365.
    } else if let Some(p) = &recipe.source.development {
        recipe.settings.age_years.unwrap_or(p.reference_age_years)
    } else {
        recipe.wood_details.as_ref().map_or(50., |d| d.age_years)
    }
}

impl Details {
    pub fn validate(&self, recipe: &Recipe) -> Result<()> {
        if recipe.version < 12
            || !finite_values(&[self.age_years, self.healing_years])
            || self.age_years < 0.
            || self.age_years > growth::MAX_TIME as f64 / 365.
            || self.healing_years <= 0.
            || self.scars.len() > 64
        {
            return Err(invalid(
                "Wood history requires recipe12, finite age/healing and at most64 scars",
            ));
        }
        if let Some(r) = &self.roots
            && (r.bulges > 16
                || !finite_values(&[r.swelling, r.flattening, r.bend, r.twist, r.grain])
                || !(0. ..=0.65).contains(&r.swelling)
                || !(0. ..=0.45).contains(&r.flattening)
                || !(0. ..=1.).contains(&r.bend)
                || r.twist.abs() > 2.
                || !(0. ..=0.3).contains(&r.grain))
        {
            return Err(invalid("Invalid root morphology profile"));
        }
        if let Some(n) = &self.natural_loss
            && (!finite_values(&[n.first_year, n.interval_years])
                || n.first_year < 0.
                || n.interval_years < 1.
                || n.maximum_events > 32)
        {
            return Err(invalid("Invalid bounded natural branch-loss schedule"));
        }
        let mut ids = BTreeSet::new();
        for s in &self.scars {
            if !identifier(&s.id)
                || !ids.insert(&s.id)
                || !organ_id(&s.parent)
                || s.branch.as_ref().is_some_and(|b| !organ_id(b))
                || !finite_values(&[
                    s.distance_m,
                    s.angle,
                    s.radius_m,
                    s.raised_m,
                    s.depth_m,
                    s.year,
                    s.closing_years,
                ])
                || s.distance_m < 0.
                || s.radius_m < 0.001
                || s.raised_m < 0.
                || s.depth_m < 0.
                || s.year < 0.
                || s.closing_years <= 0.
                || s.angle.abs() > 100.
                || s.raised_m > s.radius_m * 2.
                || s.depth_m > s.radius_m * 1.5
                || s.removed_organs.len() > MAX_ORGANS
                || s.removed_organs.iter().any(|id| !organ_id(id))
            {
                return Err(invalid(format!("Invalid or duplicate woody scar {}", s.id)));
            }
            if s.branch.is_none()
                && s.year <= age(recipe)
                && !self
                    .scars
                    .iter()
                    .any(|loss| loss.year <= age(recipe) && loss.removed_organs.contains(&s.parent))
                && !recipe
                    .source
                    .organs
                    .iter()
                    .any(|o| o.id == s.parent && matches!(o.shape, Shape::Axis { .. }))
            {
                return Err(invalid(format!("Unknown woody scar parent {}", s.parent)));
            }
        }
        Ok(())
    }
}
pub fn lost(recipe: &Recipe, id: &str) -> bool {
    recipe.wood_details.as_ref().is_some_and(|d| {
        d.scars
            .iter()
            .any(|s| s.year <= age(recipe) && s.branch.as_deref() == Some(id))
    })
}
pub fn root_profile<'a>(recipe: &'a Recipe, id: &str) -> Option<&'a RootProfile> {
    recipe
        .wood_details
        .as_ref()?
        .roots
        .as_ref()
        .filter(|_| roots::system_for(&recipe.source, id).is_some())
}
/// Deform the rest curve before child attachment; endpoints remain fixed.
pub fn bend(recipe: &Recipe, organ: &Organ, points: &mut Vec<DVec3>, normal: DVec3) {
    let Some(r) = root_profile(recipe, &organ.id) else {
        return;
    };
    let Shape::Axis { radius, .. } = organ.shape else {
        return;
    };
    if points.len() < 17 {
        let original = points.clone();
        *points = (0..=16)
            .map(|i| bake::curve(&original, i as f64 / 16.).0)
            .collect();
    }
    let seed = recipe.wood_details.as_ref().unwrap().seed;
    let phase = TAU * unit(seed, &organ.id, 1);
    let tangent = (points[points.len() - 1] - points[0]).normalize_or_zero();
    let n = (normal - tangent * normal.dot(tangent)).normalize_or_zero();
    let b = tangent.cross(n);
    let count = points.len() - 1;
    for (i, p) in points.iter_mut().enumerate() {
        let u = i as f64 / count as f64;
        let envelope = 16. * u * u * (1. - u).powi(2);
        *p += radius
            * recipe.settings.scale
            * r.bend
            * envelope
            * (n * (u * 5.3 + phase).sin() + b * 0.55 * (u * 8.1 - phase).sin());
    }
}
fn circular(organ: &Organ, sides: u8, samples: u16) -> Surface {
    let Shape::Axis {
        radius,
        tip_ratio,
        ellipticity,
    } = organ.shape
    else {
        unreachable!()
    };
    let sections = (0..32)
        .map(|i| {
            let u = i as f64 / 31.;
            let r = radius * (tip_ratio + (1. - tip_ratio) * (1. - u).powf(organ.taper_power));
            let e = ellipticity;
            Section {
                id: format!("detail{i}"),
                u,
                angle: 0.,
                offset: [0.; 2],
                radii: (0..sides)
                    .map(|j| {
                        let a = TAU * j as f64 / sides as f64;
                        (r / (a.cos().powi(2) + a.sin().powi(2) / (e * e)).sqrt()).max(0.02)
                    })
                    .collect(),
            }
        })
        .collect();
    Surface {
        sections,
        relief: vec![],
        sides,
        samples,
    }
}
fn roots_surface(
    base: &Surface,
    organ: &Organ,
    r: &RootProfile,
    seed: u64,
    length: f64,
) -> Surface {
    let Shape::Axis { radius, .. } = organ.shape else {
        unreachable!()
    };
    let strength = ((radius - 0.05).max(0.) / radius).min(1.);
    let mut result = base.clone();
    result.samples = result.samples.max(96);
    result.sections = (0..32)
        .map(|i| {
            let u = i as f64 / 31.;
            let fade = bump((u - 0.5) / 0.5);
            let a0 = TAU * unit(seed, &organ.id, 10) + r.twist * u * length;
            let radii = (0..base.sides.min(32))
                .map(|j| {
                    let a = TAU * j as f64 / base.sides.min(32) as f64;
                    let mut gain = 1.
                        + strength
                            * fade
                            * r.flattening
                            * (2. * (a - a0)).cos()
                            * (0.5 + 0.5 * (u * 4.7 + a0).sin());
                    for k in 0..r.bulges as u64 {
                        let at = 0.08 + 0.84 * unit(seed, &organ.id, 20 + k * 5);
                        let len = 0.06 + 0.17 * unit(seed, &organ.id, 21 + k * 5);
                        let angle = TAU * unit(seed, &organ.id, 22 + k * 5);
                        let width = 0.65 + 0.8 * unit(seed, &organ.id, 23 + k * 5);
                        let da = (a - angle + PI).rem_euclid(TAU) - PI;
                        gain += strength
                            * r.swelling
                            * bump((u - at) / len)
                            * bump(da / width)
                            * (0.5 + unit(seed, &organ.id, 24 + k * 5));
                    }
                    (base.radius(u, a) * gain).max(0.02)
                })
                .collect();
            Section {
                id: format!("rootform{i}"),
                u,
                angle: 0.,
                offset: base.offset(u),
                radii,
            }
        })
        .collect();
    for k in 0..3_u64 {
        let start = 0.03 + 0.25 * unit(seed, &organ.id, 201 + k);
        let end = 0.65 + 0.32 * unit(seed, &organ.id, 211 + k);
        let a = TAU * unit(seed, &organ.id, 221 + k);
        result.relief.push(Relief {
            id: format!("rootgrain{k}"),
            enabled: true,
            path: (0..5)
                .map(|i| {
                    let t = i as f64 / 4.;
                    Control {
                        u: start + (end - start) * t,
                        angle: a + 0.55 * (t * 3. + k as f64).sin(),
                    }
                })
                .collect(),
            width: (radius * 0.8).max(0.02),
            height: radius * r.grain * strength * (if k == 2 { -0.5 } else { 1. }),
            end_fade: 0.06,
            follow_taper: true,
        });
    }
    result
}
#[derive(Clone, Debug)]
pub struct Patch {
    visible: bool,
    id: String,
    u: f64,
    half_u: f64,
    angle: f64,
    width: f64,
    raised: f64,
    depth: f64,
    closure: f64,
    phase: f64,
}
fn patch_coordinates(p: &Patch, u: f64, a: f64, base: f64) -> (f64, f64, f64) {
    let x = (u - p.u) / p.half_u;
    let y = base * (a - p.angle).sin() / p.width;
    let transverse = 2. * base * ((a - p.angle) * 0.5).sin() / p.width;
    let q = (x * x + transverse * transverse).sqrt();
    let theta = y.atan2(x);
    (x, y, q / (1. + 0.10 * (theta * 3. + p.phase).sin()))
}
pub fn grain_angle(surface: &Surface, patches: &[Patch], u: f64, a: f64) -> f64 {
    if patches.is_empty() {
        return a;
    }
    let base = surface.radius(u, a);
    let mut angle = a;
    for p in patches.iter().filter(|p| p.visible) {
        let (x, y, q) = patch_coordinates(p, u, a, base);
        angle += 0.45 * x * y / (0.25 + y.abs()) * (-1.3 * q * q).exp();
    }
    angle
}
pub fn marked_radius(surface: &Surface, patches: &[Patch], u: f64, a: f64) -> f64 {
    if patches.is_empty() {
        return surface.radius(u, a);
    }
    let base = surface.radius(u, a);
    let angle = grain_angle(surface, patches, u, a);
    let mut radius = surface.radius(u, angle);
    for p in patches.iter().filter(|p| p.visible) {
        let (_, _, q) = patch_coordinates(p, u, a, base);
        let rim = (-((q - 0.85) / 0.28).powi(2)).exp() * bump(q / 1.8);
        let cavity = bump(q / 0.70);
        radius += p.raised * (0.35 + 0.65 * (PI * p.closure).sin()) * rim
            - p.depth * (1. - 0.88 * p.closure) * cavity;
    }
    radius
}
pub fn point(surface: &Surface, p: &Pose, u: f64, a: f64) -> DVec3 {
    let f = p.at(u);
    let offset = surface.offset(u);
    let r = marked_radius(surface, &p.patches, u, a) * p.surface_scale;
    f.p + (f.n * offset[0] + f.b() * offset[1]) * p.surface_scale
        + (f.n * a.cos() + f.b() * a.sin()) * r
}
/// Derived geometry only. The canonical event log remains unchanged by evaluation.
pub fn attach(recipe: &Recipe, p: &mut Pose) -> Result<()> {
    let Some(d) = &recipe.wood_details else {
        return Ok(());
    };
    let organ = &recipe.source.organs[p.source];
    let Shape::Axis { .. } = organ.shape else {
        return Ok(());
    };
    let scars: Vec<_> = d
        .scars
        .iter()
        .filter(|s| s.parent == organ.id && s.year <= age(recipe))
        .collect();
    let root = root_profile(recipe, &organ.id);
    if root.is_none() && scars.is_empty() {
        return Ok(());
    }
    if root.is_some() && p.section.is_some() {
        return Err(invalid(
            "Irregular rounded roots require a root system without buttress walls",
        ));
    }
    let mut surface = p.wood_surface.as_ref().map_or_else(
        || circular(organ, recipe.settings.sides.max(12), 96),
        |s| (**s).clone(),
    );
    if !scars.is_empty() {
        let incident = recipe.source.organs.iter().any(|o| {
            !lost(recipe, &o.id)
                && o.parent.is_some()
                && (o.id == organ.id || o.parent.as_deref() == Some(organ.id.as_str()))
                && matches!(
                    o.connection,
                    Connection::Graft | Connection::Fork | Connection::Basal
                )
        });
        if !incident {
            surface.sides = 96;
        }
        surface.samples = surface.samples.max(128);
    }
    if let Some(r) = root {
        surface = roots_surface(&surface, organ, r, d.seed, p.length / recipe.settings.scale);
    }
    let source_length = p.length / recipe.settings.scale;
    for s in scars {
        if s.distance_m > source_length * (1. + 1e-9) {
            continue;
        }
        let closure = 1. - (-(age(recipe) - s.year) / s.closing_years).exp();
        p.patches.push(Patch {
            visible: s.visible,
            id: s.id.clone(),
            u: p.offset(0., s.distance_m * recipe.settings.scale),
            half_u: (s.radius_m / source_length).max(32. * f64::EPSILON),
            angle: s.angle,
            width: s.radius_m,
            raised: s.raised_m,
            depth: s.depth_m,
            closure,
            phase: TAU * unit(d.seed, &s.id, 0),
        });
    }
    surface.validate()?;
    for i in 0..=surface.samples as usize * 2 {
        for j in 0..surface.sides as usize * 2 {
            let radius = marked_radius(
                &surface,
                &p.patches,
                i as f64 / (surface.samples as f64 * 2.),
                TAU * j as f64 / (surface.sides as f64 * 2.),
            );
            if !radius.is_finite() || radius < 0.02 * (1. - 16. * f64::EPSILON) {
                return Err(invalid(format!(
                    "Woody detail collapses radius at {}",
                    organ.id
                )));
            }
        }
    }
    p.wood_surface = Some(Arc::new(surface));
    Ok(())
}
pub fn record_loss(
    recipe: &mut Recipe,
    branch: &str,
    cause: Cause,
    id: Option<String>,
) -> Result<()> {
    let index = recipe
        .source
        .organs
        .iter()
        .position(|o| o.id == branch)
        .ok_or_else(|| invalid("Unknown lost branch"))?;
    let o = &recipe.source.organs[index];
    let parent = o
        .parent
        .clone()
        .ok_or_else(|| invalid("Cannot lose the root stem"))?;
    let Shape::Axis { .. } = o.shape else {
        return Err(invalid("Only woody branches can leave a scar"));
    };
    if roots::system_for(&recipe.source, branch).is_some() || lost(recipe, branch) {
        return Err(invalid("Branch already lost or belongs to the root system"));
    }
    let poses = bake::history_poses(recipe)?;
    let child = poses
        .iter()
        .find(|p| p.source == index)
        .ok_or_else(|| invalid("Branch is not currently visible"))?;
    let parent_pose = child
        .parent
        .map(|i| &poses[i])
        .ok_or_else(|| invalid("Branch has no current woody parent"))?;
    let frame = parent_pose.at(o.insertion);
    let dir = child.samples[0].t;
    let distance = parent_pose
        .samples
        .windows(2)
        .enumerate()
        .map(|(i, p)| {
            let lo = i as f64 / (parent_pose.samples.len() - 1) as f64;
            let hi = (i + 1) as f64 / (parent_pose.samples.len() - 1) as f64;
            p[0].p.distance(p[1].p) * ((o.insertion - lo) / (hi - lo)).clamp(0., 1.)
        })
        .sum::<f64>()
        / recipe.settings.scale;
    let mut removed = BTreeSet::from([branch.to_owned()]);
    for node in &recipe.source.organs {
        if node.parent.as_ref().is_some_and(|p| removed.contains(p)) {
            removed.insert(node.id.clone());
        }
    }
    let year = age(recipe);
    let d = recipe.wood_details.get_or_insert_with(|| Details {
        age_years: year,
        ..Default::default()
    });
    let scar_id = id.unwrap_or_else(|| {
        (0..1000)
            .map(|i| format!("loss{i}"))
            .find(|id| !d.scars.iter().any(|s| s.id == *id))
            .unwrap()
    });
    let r = (child.radius / recipe.settings.scale).max(0.001);
    let depth = (r * 0.25).min((parent_pose.radius / recipe.settings.scale - 0.025).max(0.));
    d.scars.push(Scar {
        visible: true,
        id: scar_id,
        parent,
        branch: Some(branch.into()),
        removed_organs: removed.into_iter().collect(),
        distance_m: distance,
        angle: dir.dot(frame.b()).atan2(dir.dot(frame.n)),
        radius_m: r,
        raised_m: r * if cause == Cause::Breakage { 0.65 } else { 0.4 },
        depth_m: depth,
        year,
        closing_years: d.healing_years,
        cause,
    });
    recipe.version = recipe.version.max(12);
    recipe.validate()
}
/// A site is a woody attachment; its dependent arms leave with that attachment.
pub fn record_site_loss(recipe: &mut Recipe, site: &str, cause: Cause) -> Result<()> {
    if !lost(recipe, site) {
        record_loss(recipe, site, cause, None)?;
    }
    Ok(())
}

fn bound(old: &Recipe, plant: &growth::Plant, details: &Details) -> Result<Recipe> {
    let mut r = plant.bound_recipe()?;
    r.settings = old.settings.clone();
    r.edits = old.edits.clone();
    r.version = old.version.max(12);
    r.wood_details = Some(details.clone());
    Ok(r)
}
/// Advance through explicit and existing native loss dates in chronological order.
pub fn advance(old: &Recipe, plant: &mut growth::Plant, time: u64) -> Result<Option<Details>> {
    let Some(mut d) = old.wood_details.clone() else {
        plant.advance(time)?;
        return Ok(None);
    };
    let previous = plant.production.plant.time;
    let born = plant.production.plant.born;
    let mut dates = vec![];
    if let Some(schedule) = &d.natural_loss {
        for i in 0..schedule.maximum_events as u64 {
            let year = schedule.first_year
                + schedule.interval_years * (i as f64 + 0.35 * unit(d.seed, "loss-schedule", i));
            dates.push((born.saturating_add((year * 365.).round() as u64), Some(i)));
        }
    }
    // Record the frozen legacy maturity rule rather than silently erasing branches.
    let maturity = born + 300 * 365 + u64::from(plant.definition.version >= 6);
    if (5..=8).contains(&plant.definition.version) {
        dates.push((maturity, None));
    }
    dates.sort();
    for (day, kind) in dates {
        if day <= previous || day > time {
            continue;
        }
        if let Some(i) = kind {
            let id = format!("natural{i}");
            if d.scars.iter().any(|s| s.id == id) {
                continue;
            }
            plant.advance(day)?;
            let mut r = bound(old, plant, &d)?;
            let candidates: Vec<_> = r
                .source
                .organs
                .iter()
                .filter(|o| {
                    o.parent.as_ref().is_some_and(|p| {
                        r.source
                            .organs
                            .iter()
                            .any(|q| q.id == *p && q.parent.is_none())
                    }) && matches!(o.shape, Shape::Axis { .. })
                        && plant.production.sites.contains_key(&o.id)
                        && roots::system_for(&r.source, &o.id).is_none()
                        && !lost(&r, &o.id)
                })
                .map(|o| o.id.clone())
                .collect();
            if candidates.is_empty() {
                continue;
            }
            let branch = &candidates
                [(unit(d.seed, "branch", i) * candidates.len() as f64) as usize % candidates.len()];
            record_loss(&mut r, branch, Cause::Breakage, Some(id))?;
            d = r.wood_details.unwrap();
            plant.production.remove_site(branch).map_err(invalid)?;
        } else {
            plant.advance(day - 1)?;
            let mut r = bound(old, plant, &d)?;
            let count = if plant.definition.version == 5 {
                (plant.definition.maximum_nodes / 3).max(1)
            } else {
                plant.definition.maximum_nodes / 2
            };
            let mut removed = vec![];
            for i in 0..count {
                let site = format!("node{i}");
                if !r.source.organs.iter().any(|o| o.id == site) {
                    continue;
                }
                let before = r.wood_details.as_ref().unwrap().scars.len();
                record_site_loss(&mut r, &site, Cause::Breakage)?;
                for event in &mut r.wood_details.as_mut().unwrap().scars[before..] {
                    event.year = (day - born) as f64 / 365.;
                }
                removed.push(site);
            }
            d = r.wood_details.unwrap();
            for site in removed {
                plant.production.remove_site(&site).map_err(invalid)?;
            }
            plant.advance(day)?;
        }
    }
    plant.advance(time)?;
    Ok(Some(d))
}

/// Disabling a visual layer must not resurrect a historically removed branch.
pub fn preserve_losses(old: Option<&Details>, next: Option<&Details>) -> Result<()> {
    if let Some(old) = old {
        for event in old.scars.iter().filter(|e| e.branch.is_some()) {
            if !next.is_some_and(|d| {
                d.scars.iter().any(|n| {
                    n.id == event.id
                        && n.branch == event.branch
                        && n.parent == event.parent
                        && n.year == event.year
                        && n.distance_m == event.distance_m
                        && n.removed_organs == event.removed_organs
                })
            }) {
                return Err(invalid(
                    "A branch-loss event cannot be removed or retimed by a visual edit; undo the intervention instead",
                ));
            }
        }
    }
    Ok(())
}

pub fn coordinates(p: &Pose) -> Vec<f64> {
    let mut values: Vec<_> = p
        .patches
        .iter()
        .flat_map(|s| (-8..=8).map(move |i| s.u + s.half_u * i as f64 * 0.225))
        .filter(|u| *u > 0. && *u < 1.)
        .collect();
    values.sort_by(f64::total_cmp);
    values.dedup();
    values
}
pub fn feature_address(p: &Pose, organ: &str, u: f64, j: usize) -> Option<String> {
    for s in &p.patches {
        for i in -8..=8 {
            if (s.u + s.half_u * i as f64 * 0.225).to_bits() == u.to_bits() {
                return Some(format!("{organ}:scar-v1:{}:{i}/{j}", s.id));
            }
        }
    }
    None
}
