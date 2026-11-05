//! Native spline/frame compiler; stable material addresses precede triangles.
use super::*;
use glam::{DMat3, DQuat};
use std::f64::consts::{PI, TAU};

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub p: DVec3,
    pub t: DVec3,
    pub n: DVec3,
}
impl Frame {
    pub fn b(self) -> DVec3 {
        self.t.cross(self.n)
    }
    pub fn vector(self, v: DVec3) -> DVec3 {
        self.t * v.x + self.n * v.y + self.b() * v.z
    }
    pub fn local(self, v: DVec3) -> DVec3 {
        DVec3::new(v.dot(self.t), v.dot(self.n), v.dot(self.b()))
    }
}
#[derive(Clone, Debug)]
pub struct Pose {
    pub source: usize,
    pub parent: Option<usize>,
    pub insertion: f64,
    pub samples: Vec<Frame>,
    pub length: f64,
    pub radius: f64,
    pub width: f64,
    pub flare: f64,
    pub stiffness: f64,
    pub section: Option<Arc<super::root_sections::Section>>,
    pub wood_surface: Option<Arc<super::wood_surface::Surface>>,
    pub surface_scale: f64,
    pub flutes: Option<Arc<super::root_sections::Flutes>>,
    pub patches: Vec<super::wood_details::Patch>,
}
impl Pose {
    pub fn at(&self, u: f64) -> Frame {
        let x = u.clamp(0., 1.) * (self.samples.len() - 1) as f64;
        let i = (x as usize).min(self.samples.len() - 2);
        let f = x - i as f64;
        let a = self.samples[i];
        let b = self.samples[i + 1];
        let t = a.t.lerp(b.t, f).normalize();
        let n = normal(t, a.n.lerp(b.n, f));
        Frame {
            p: a.p.lerp(b.p, f),
            t,
            n,
        }
    }
    /// Convert a metric collar offset into the actual spline sample parameter.
    /// Control-point parameter speed is generally nonuniform.
    pub(super) fn offset(&self, u: f64, distance: f64) -> f64 {
        let mut lengths = Vec::with_capacity(self.samples.len());
        lengths.push(0.);
        for pair in self.samples.windows(2) {
            lengths.push(lengths.last().unwrap() + pair[0].p.distance(pair[1].p));
        }
        let x = u * (lengths.len() - 1) as f64;
        let i = (x as usize).min(lengths.len() - 2);
        let target = lengths[i] + (lengths[i + 1] - lengths[i]) * (x - i as f64) + distance;
        if target <= 0. {
            return 0.;
        }
        if target >= *lengths.last().unwrap() {
            return 1.;
        }
        let j = lengths.partition_point(|v| *v < target).saturating_sub(1);
        (j as f64 + (target - lengths[j]) / (lengths[j + 1] - lengths[j]))
            / (lengths.len() - 1) as f64
    }
    pub fn radius_at(&self, source: &Source, u: f64) -> (f64, f64) {
        match source.organs[self.source].shape {
            Shape::Axis {
                tip_ratio,
                ellipticity,
                ..
            } => {
                let r = self.radius
                    * (tip_ratio
                        + (1. - tip_ratio) * (1. - u).powf(source.organs[self.source].taper_power));
                let r = r * (1. + self.flare * (1. - u).powi(6));
                (r, r * ellipticity)
            }
            Shape::Blade { .. } => (self.width / 2., self.radius),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Vertex {
    pub organ: usize,
    pub u: f64,
    pub local: DVec3,
    pub key: String,
}
#[derive(Clone)]
pub struct Compiled {
    pub document: meshes::Document,
    pub poses: Vec<Pose>,
    pub vertices: [Vec<Vertex>; 2],
    pub triangles_by_organ: [Vec<usize>; 2],
    pub joints: usize,
}
pub fn normal(t: DVec3, n: DVec3) -> DVec3 {
    let v = n - t * n.dot(t);
    if v.length_squared() > 1e-20 {
        v.normalize()
    } else {
        let a = if t.x.abs() < 0.7 { DVec3::X } else { DVec3::Z };
        (a - t * a.dot(t)).normalize()
    }
}
pub(super) fn curve(points: &[DVec3], u: f64) -> (DVec3, DVec3) {
    let x = u.clamp(0., 1.) * (points.len() - 1) as f64;
    let i = (x as usize).min(points.len() - 2);
    let t = x - i as f64;
    let a = points[i];
    let c = points[i + 1];
    let m0 = if i == 0 {
        c - a
    } else {
        (c - points[i - 1]) * 0.5
    };
    let m1 = if i + 2 == points.len() {
        c - a
    } else {
        (points[i + 2] - a) * 0.5
    };
    let p = a * (2. * t * t * t - 3. * t * t + 1.)
        + m0 * (t * t * t - 2. * t * t + t)
        + c * (-2. * t * t * t + 3. * t * t)
        + m1 * (t * t * t - t * t);
    let d = a * (6. * t * t - 6. * t)
        + m0 * (3. * t * t - 4. * t + 1.)
        + c * (-6. * t * t + 6. * t)
        + m1 * (3. * t * t - 2. * t);
    (p, d.normalize_or_zero())
}
fn frames(points: &[DVec3], n: DVec3, steps: usize, twist: f64) -> Result<Vec<Frame>> {
    let mut out: Vec<Frame> = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let (p, t) = curve(points, i as f64 / steps as f64);
        if !p.is_finite() || !t.is_normalized() {
            return Err(invalid("Botanical curve cusp or nonfinite position"));
        }
        let n = if let Some(last) = out.last() {
            let rotation = DQuat::from_rotation_arc(last.t, t);
            let moved = normal(t, rotation * last.n);
            DQuat::from_axis_angle(t, twist * p.distance(last.p)) * moved
        } else {
            normal(t, n)
        };
        out.push(Frame { p, t, n });
    }
    Ok(out)
}
fn hash(seed: u64, id: &str) -> f64 {
    let mut value = seed ^ 0xcbf29ce484222325;
    for b in id.bytes() {
        value ^= b as u64;
        value = value.wrapping_mul(0x100000001b3);
    }
    (value >> 11) as f64 / (1u64 << 53) as f64
}
pub fn poses(recipe: &Recipe) -> Result<Vec<Pose>> {
    poses_impl(recipe, true)
}
pub(super) fn history_poses(recipe: &Recipe) -> Result<Vec<Pose>> {
    poses_impl(recipe, false)
}
fn poses_impl(recipe: &Recipe, detailed_surfaces: bool) -> Result<Vec<Pose>> {
    recipe.validate()?;
    let s = &recipe.settings;
    let steps = s.axis_samples as usize;
    let development = super::development::state(recipe)?;
    let profile = recipe.source.development.as_ref();
    let mut depths: Vec<u32> = Vec::new();
    let mut frond_scales: Vec<f64> = Vec::new();
    let scale = DVec3::new(s.scale * s.crown_width, s.scale, s.scale * s.crown_width);
    let mut result: Vec<Pose> = Vec::new();
    let mut lookup: BTreeMap<String, usize> = BTreeMap::new();
    let mut raw: BTreeMap<String, Vec<Frame>> = BTreeMap::new();
    for (source, o) in recipe.source.organs.iter().enumerate() {
        if recipe.pruned.contains(&o.id)
            || super::wood_details::lost(recipe, &o.id)
            || o.parent.as_ref().is_some_and(|p| !lookup.contains_key(p))
        {
            continue;
        }
        let mut expansion = 1.;
        if let (Some(profile), Some(age)) = (profile, development) {
            let Some(g) = profile.expansion(o.life.as_ref(), age.physiological_years) else {
                continue;
            };
            expansion = g;
        }
        if matches!(o.shape, Shape::Blade { .. })
            && hash(recipe.source.seed, &o.id)
                >= s.leaf_density * development.map_or(1., |a| a.foliage_survival)
        {
            continue;
        }
        let edit = recipe.edits.get(&o.id).cloned().unwrap_or_default();
        let points: Vec<_> = o
            .path
            .iter()
            .map(|p| DVec3::from_array(*p) * scale)
            .collect();
        let initial = frames(&points, DVec3::from_array(o.normal), steps, o.twist)?;
        let mut transform = DMat3::IDENTITY;
        let mut origin = initial[0].p;
        let parent = o.parent.as_ref().map(|p| lookup[p]);
        if let Some(parent) = parent {
            let parent_id = o.parent.as_ref().unwrap();
            let old = &raw[parent_id];
            let x = o.insertion * steps as f64;
            let i = (x as usize).min(steps - 1);
            let f = x - i as f64;
            let t = old[i].t.lerp(old[i + 1].t, f).normalize();
            let n = normal(t, old[i].n.lerp(old[i + 1].n, f));
            let old_frame = Frame {
                p: old[i].p.lerp(old[i + 1].p, f),
                t,
                n,
            };
            let new = result[parent].at(o.insertion);
            transform = DMat3::from_cols(new.t, new.n, new.b())
                * DMat3::from_cols(old_frame.t, old_frame.n, old_frame.b()).transpose();
            if o.connection == Connection::Basal {
                transform = DMat3::IDENTITY;
            }
            origin = new.p + transform * (origin - old_frame.p);
            if matches!(o.shape, Shape::Blade { .. }) {
                let direction = transform * initial[0].t;
                let radial = direction - new.t * direction.dot(new.t);
                if radial.length_squared() > 1e-12 {
                    origin += radial.normalize()
                        * result[parent].radius_at(&recipe.source, o.insertion).0;
                }
            }
        }
        let depth = parent.map_or(0, |p| depths[p] + 1);
        let role = o
            .life
            .as_ref()
            .map_or(super::development::Role::Axis, |l| l.role);
        let inherited_frond_scale = parent.map_or(1., |p| frond_scales[p]);
        let frond_scale = if role == super::development::Role::Frond {
            let stage = o.life.as_ref().map_or(1., |life| life.growth_stage);
            inherited_frond_scale * (1. - s.frond_growth_variation * (1. - stage))
        } else {
            inherited_frond_scale
        };
        let mut age_scale = DVec3::ONE;
        let mut radial = 1.;
        let mut flare = 0.;
        let mut stiffness = 1.;
        let mut palm_radial = None;
        let root = super::roots::system_for(&recipe.source, &o.id).is_some();
        let crown = recipe.source.root_systems.iter().find(|r| r.stem == o.id);
        if let (Some(profile), Some(age)) = (profile, development) {
            age_scale = if matches!(o.shape, Shape::Blade { .. })
                || role == super::development::Role::Frond
            {
                DVec3::splat(age.leaf * expansion)
            } else {
                let power = profile.branch_length_decay.powi(depth as i32);
                DVec3::new(
                    age.crown.powf(power),
                    age.height.powf(power),
                    age.crown.powf(power),
                ) * expansion
            };
            if root {
                age_scale = DVec3::splat(age.crown * expansion);
            }
            if profile.model == super::development::Model::Palm && parent.is_none() {
                let local = (age.physiological_years
                    - o.life.as_ref().map_or(0., |l| l.born_years))
                .max(0.25);
                let local = profile.evaluate(local, 1.)?;
                age_scale = DVec3::new(age.crown, local.height, age.crown);
                palm_radial = Some(local.radial);
            }
            if parent.is_none() {
                origin *= DVec3::new(age.crown, 1., age.crown);
            }
            radial = if matches!(o.shape, Shape::Blade { .. }) {
                1.
            } else if role == super::development::Role::Frond {
                age.leaf * expansion
            } else {
                age.radial.powf(if age.radial < 1. {
                    1.
                } else {
                    profile.branch_thickening_decay.powi(depth as i32)
                })
            };
            if let Some(value) = palm_radial {
                radial = value;
            }
            if role == super::development::Role::AerialRoot {
                radial *= expansion;
            }
            if parent.is_none()
                && crown.is_none()
                && profile.model == super::development::Model::Woody
            {
                flare = profile.basal_flare * age.maturity;
            }
            stiffness = if matches!(o.shape, Shape::Blade { .. }) {
                1.
            } else {
                age.stiffness
            };
        }
        // One similarity scale for the whole frond and all its descendants.
        // Parent frames carry the attachment; local geometry uses this same scale.
        if frond_scale != 1. {
            age_scale *= frond_scale;
            radial *= frond_scale;
        }
        let rotation = DQuat::from_scaled_axis(DVec3::from_array(edit.rotation));
        let factor = edit.length
            * if matches!(o.shape, Shape::Blade { .. }) {
                s.leaf_length
            } else {
                1.
            };
        let mut final_points: Vec<_> = points
            .iter()
            .map(|p| {
                origin
                    + transform * (rotation * ((*p - points[0]) * age_scale) * factor)
                    + DVec3::from_array(edit.offset)
            })
            .collect();
        if development.is_some() && role == super::development::Role::AerialRoot {
            let ground = points.last().unwrap().y + edit.offset[1];
            let target = origin.y + (ground - origin.y) * expansion;
            let correction = target - final_points.last().unwrap().y;
            let count = final_points.len() - 1;
            for (i, p) in final_points.iter_mut().enumerate() {
                let u = i as f64 / count as f64;
                p.y += correction * u * u * (3. - 2. * u);
            }
        }
        if let Some(crown) = crown {
            let Shape::Axis { radius, .. } = o.shape else {
                unreachable!()
            };
            let lift = radius * crown.settings.neck_height * s.scale * age_scale.y;
            let base = final_points[0];
            let mut raised = Vec::new();
            raised.push(base);
            raised.extend(final_points.into_iter().map(|p| p + DVec3::Y * lift));
            let mut distances = vec![0.];
            for pair in raised.windows(2) {
                distances.push(distances.last().unwrap() + pair[0].distance(pair[1]));
            }
            let length = *distances.last().unwrap();
            let count = ((length / (lift / 4.).min(length / 32.)).ceil() as usize).clamp(16, 128);
            final_points = (0..=count)
                .map(|k| {
                    let d = length * k as f64 / count as f64;
                    let i = distances
                        .partition_point(|v| *v < d)
                        .saturating_sub(1)
                        .min(raised.len() - 2);
                    raised[i].lerp(
                        raised[i + 1],
                        (d - distances[i]) / (distances[i + 1] - distances[i]),
                    )
                })
                .collect();
        }
        super::wood_details::bend(
            recipe,
            o,
            &mut final_points,
            transform * (rotation * initial[0].n),
        );
        let steps = edit
            .wood_surface
            .as_ref()
            .map_or(steps, |s| s.samples as usize);
        let samples = frames(
            &final_points,
            transform * (rotation * initial[0].n),
            steps,
            o.twist,
        )?;
        let samples =
            super::deformation::apply(samples, &edit.deformation, recipe.source.seed, &o.id)?;
        let length = samples
            .windows(2)
            .map(|v| v[0].p.distance(v[1].p))
            .sum::<f64>();
        let (radius, width) = match o.shape {
            Shape::Axis { radius, .. } => (radius * s.scale * s.radius * edit.radius * radial, 0.),
            Shape::Blade {
                width, thickness, ..
            } => (
                thickness * s.scale * edit.radius * frond_scale / 2.,
                width
                    * s.scale
                    * s.leaf_width
                    * frond_scale
                    * edit.width
                    * development.map_or(1., |a| a.leaf * expansion),
            ),
        };
        depths.push(depth);
        frond_scales.push(frond_scale);
        raw.insert(o.id.clone(), initial);
        lookup.insert(o.id.clone(), result.len());
        let section = super::roots::system_for(&recipe.source, &o.id).and_then(|system| {
            system.settings.buttress.as_ref().map(|form| {
                let stem = &result[lookup[&system.stem]];
                Arc::new(super::root_sections::Section {
                    form: form.clone(),
                    base: stem.samples[0].p,
                    stem_radius: stem.radius,
                    variation: 1.
                        + system.settings.irregularity
                            * (2. * hash(recipe.source.seed, &o.id) - 1.),
                })
            })
        });
        result.push(Pose {
            source,
            parent,
            insertion: o.insertion,
            samples,
            length,
            radius,
            width,
            flare,
            stiffness,
            section,
            wood_surface: if !detailed_surfaces {
                None
            } else if recipe
                .living
                .as_ref()
                .is_some_and(|p| (6..=8).contains(&p.definition.version))
                && o.id == "stem"
            {
                let r = match o.shape {
                    Shape::Axis { radius, .. } => radius,
                    _ => unreachable!(),
                };
                let base = super::growth::sequoia_realistic::bole(r, recipe.source.seed);
                Some(Arc::new(
                    if let Some(profile) = recipe
                        .living
                        .as_ref()
                        .and_then(|p| p.definition.wood_volume.as_ref())
                    {
                        profile.apply(&base, r, recipe.source.seed)?
                    } else {
                        base
                    },
                ))
            } else {
                edit.wood_surface.clone().map(Arc::new)
            },
            surface_scale: match o.shape {
                Shape::Axis { radius: r, .. } => radius / r,
                _ => 1.,
            },
            flutes: None,
            patches: vec![],
        });
        if detailed_surfaces {
            super::wood_details::attach(recipe, result.last_mut().unwrap())?;
        }
    }
    let mut tree = Vec::with_capacity(result.len());
    for (i, p) in result.iter().enumerate() {
        tree.push(p.parent.map_or(i, |parent| tree[parent]));
    }
    for system in &recipe.source.root_systems {
        let Some(form) = system.settings.buttress.as_ref().filter(|f| f.fluting > 0.) else {
            continue;
        };
        let Some(stem) = result
            .iter()
            .position(|p| recipe.source.organs[p.source].id == system.stem)
        else {
            continue;
        };
        let angles = result
            .iter()
            .filter(|p| {
                p.parent == Some(stem)
                    && recipe.source.organs[p.source].connection == Connection::Basal
            })
            .map(|p| p.samples[0].t.z.atan2(p.samples[0].t.x))
            .collect();
        let field = Arc::new(super::root_sections::Flutes {
            form: form.clone(),
            base: result[stem].samples[0].p,
            radius: result[stem].radius,
            angles,
        });
        for (i, p) in result.iter_mut().enumerate() {
            if tree[i] == stem {
                p.flutes = Some(field.clone());
            }
        }
    }
    if result.is_empty() {
        return Err(invalid("Recipe excludes every botanical root"));
    }
    Ok(result)
}
struct PalmateContour {
    knots: Vec<(f64, f64)>,
    rounding: f64,
    valley_rounding: f64,
    extent: f64,
}
impl PalmateContour {
    fn new(outline: &Outline) -> Self {
        let Outline::Palmate {
            tips,
            spread,
            depth,
            rounding,
            valley_rounding,
            lengths,
        } = outline
        else {
            unreachable!()
        };
        let count = *tips as usize;
        let angles: Vec<_> = (0..count)
            .map(|i| -spread + 2. * spread * i as f64 / (count - 1) as f64)
            .collect();
        let h = 0.3;
        let mut knots = vec![(-PI, h), ((-PI + angles[0]) / 2., h * (1. - depth))];
        for i in 0..count {
            if i > 0 {
                knots.push((
                    (angles[i - 1] + angles[i]) / 2.,
                    lengths[i - 1].min(lengths[i]) * (1. - depth),
                ));
            }
            knots.push((angles[i], lengths[i]));
        }
        knots.push(((PI + angles[count - 1]) / 2., h * (1. - depth)));
        knots.push((PI, h));
        let mut result = Self {
            knots,
            rounding: *rounding,
            valley_rounding: *valley_rounding,
            extent: 1.,
        };
        result.extent = (0..=256)
            .map(|i| {
                let a = -PI + TAU * i as f64 / 256.;
                (a.sin() * result.radial(a)).abs()
            })
            .fold(0., f64::max);
        result
    }
    fn radial(&self, theta: f64) -> f64 {
        let (a, c) = self
            .knots
            .windows(2)
            .find(|k| theta <= k[1].0)
            .map(|k| (k[0], k[1]))
            .unwrap_or((
                self.knots[self.knots.len() - 2],
                self.knots[self.knots.len() - 1],
            ));
        let t = ((theta - a.0) / (c.0 - a.0)).clamp(0., 1.);
        let rise = |x: f64| {
            let n = x.powf(self.valley_rounding);
            n / (n + (1. - x).powf(self.rounding))
        };
        if c.1 >= a.1 {
            a.1 + (c.1 - a.1) * rise(t)
        } else {
            c.1 + (a.1 - c.1) * rise(1. - t)
        }
    }
    fn point(&self, r: f64, v: f64) -> (f64, f64) {
        let theta = PI * v;
        let radius = r * self.radial(theta);
        (
            theta.sin() * radius / (2. * self.extent),
            (0.3 + theta.cos() * radius) / 1.3,
        )
    }
}
fn blade_point(
    pose: &Pose,
    organ: &Organ,
    u: f64,
    v: f64,
    contour: Option<&PalmateContour>,
) -> (DVec3, f64) {
    let Shape::Blade {
        outline, fold, cup, ..
    } = &organ.shape
    else {
        unreachable!()
    };
    let (x, y) = match outline {
        Outline::Strip {
            base_power: a,
            tip_power: c,
        } => {
            let maximum = (a / (a + c)).powf(*a) * (c / (a + c)).powf(*c);
            (v * 0.5 * u.powf(*a) * (1. - u).powf(*c) / maximum, u)
        }
        Outline::Palmate { .. } => contour.unwrap().point(u, v),
    };
    let along = y.clamp(0., 1.);
    let f = pose.at(along);
    let x = x * pose.width;
    (
        f.p + f.t * ((y - along) * pose.length)
            + f.n * x
            + f.b() * (fold * x.abs() + cup * x * x / pose.width),
        along,
    )
}

struct Builder {
    positions: Vec<[f32; 3]>,
    triangles: Vec<[u32; 3]>,
    vertices: Vec<Vertex>,
    owners: Vec<usize>,
    keys: BTreeMap<String, u32>,
    flow_coordinates: BTreeMap<u32, [f32; 3]>,
}
impl Builder {
    fn new() -> Self {
        Self {
            positions: vec![],
            triangles: vec![],
            vertices: vec![],
            owners: vec![],
            keys: BTreeMap::new(),
            flow_coordinates: BTreeMap::new(),
        }
    }
    fn vertex(
        &mut self,
        organ: usize,
        pose: &Pose,
        u: f64,
        point: DVec3,
        key: String,
        recipe: &Recipe,
    ) -> Result<u32> {
        if let Some(index) = self.keys.get(&key) {
            return Ok(*index);
        }
        if self.positions.len() >= MAX_VERTICES
            || !point.is_finite()
            || point.abs().max_element() > 10000.
        {
            return Err(invalid("Botanical vertex budget/coordinates exceeded"));
        }
        let f = pose.at(u);
        let delta = recipe
            .manual
            .get(&key)
            .map(|v| f.vector(DVec3::from_array(*v)))
            .unwrap_or(DVec3::ZERO);
        let p = point + delta;
        let index = self.positions.len() as u32;
        self.positions.push(p.as_vec3().to_array());
        self.vertices.push(Vertex {
            organ,
            u,
            local: f.local(p - f.p),
            key: key.clone(),
        });
        self.keys.insert(key, index);
        Ok(index)
    }
    fn face(&mut self, a: u32, c: u32, d: u32, owner: usize) -> Result<()> {
        if self.triangles.len() >= MAX_TRIANGLES {
            return Err(invalid("Botanical triangle budget exceeded"));
        }
        let p = DVec3::from_array(self.positions[a as usize].map(f64::from));
        let q = DVec3::from_array(self.positions[c as usize].map(f64::from));
        let r = DVec3::from_array(self.positions[d as usize].map(f64::from));
        if (q - p).cross(r - p).length_squared() > 1e-24 {
            self.triangles.push([a, c, d]);
            self.owners.push(owner);
        }
        Ok(())
    }
}
fn surface_address(p: &Pose, recipe: &Recipe, u: f64, j: usize) -> String {
    let id = &recipe.source.organs[p.source].id;
    if let Some(key) = super::wood_details::feature_address(p, id, u, j) {
        return key;
    }
    if super::root_sections::shaped(p) {
        address(
            &format!("{id}:wall-v1:{}", super::root_sections::sides(p, recipe)),
            u,
            j,
        )
    } else {
        address(id, u, j)
    }
}
fn address(id: &str, u: f64, j: usize) -> String {
    format!("{id}@{:016x}/{j}", u.to_bits())
}
struct Joint {
    parent: usize,
    child: usize,
    lo: f64,
    hi: f64,
    start: f64,
    terminal: bool,
    peer: Option<(usize, f64)>,
    mesh: Option<crate::mesh_organic::MultiJunction>,
    basal: bool,
    root_ports: Vec<(usize, f64)>,
}
impl Joint {
    fn incoming(&self, organ: usize) -> Option<f64> {
        if self.basal {
            return if organ == self.parent {
                Some(self.lo)
            } else {
                self.root_ports
                    .iter()
                    .find(|(i, _)| *i == organ)
                    .map(|(_, u)| *u)
            };
        }
        if self.child == organ {
            Some(self.start)
        } else {
            self.peer.filter(|(i, _)| *i == organ).map(|(_, u)| u)
        }
    }
}
fn junctions(poses: &[Pose], recipe: &Recipe) -> Result<Vec<Joint>> {
    let mut result: Vec<Joint> = vec![];
    for (child, p) in poses.iter().enumerate() {
        let Some(parent) = p.parent else {
            continue;
        };
        if p.width > 0.
            || poses[parent].width > 0.
            || recipe.source.organs[p.source].connection != Connection::Graft
        {
            continue;
        }
        let parent_pose = &poses[parent];
        let r = parent_pose
            .radius_at(&recipe.source, p.insertion)
            .0
            .max(p.radius);
        let mut accepted = None;
        let mut reason = "Collar windows overlap a root or another branch".to_string();
        // Preserve existing valid joints, then search metric offsets on nonuniform curves.
        let legacy = [1.8, 2.5, 3.5, 5., 7.].into_iter().map(|f| (f, false, 1.));
        let metric = [1.8, 2.5, 3.5, 5., 7., 10., 14., 20., 28.]
            .into_iter()
            .flat_map(|f| {
                [0.6, 0.8, 1., 1.3, 1.7, 2.3]
                    .into_iter()
                    .map(move |ratio| (f, true, ratio))
            });
        for (factor, metric, child_ratio) in legacy.chain(metric) {
            let distance = r * factor;
            let lo = if metric {
                parent_pose.offset(p.insertion, -distance)
            } else {
                p.insertion - distance / parent_pose.length
            };
            let hi = if metric {
                parent_pose.offset(p.insertion, distance)
            } else {
                p.insertion + distance / parent_pose.length
            };
            let start = if metric {
                p.offset(0., distance * child_ratio)
            } else {
                distance / p.length
            };
            if lo <= 0.
                || hi >= 1.
                || start >= 0.8
                || result
                    .iter()
                    .any(|j| j.parent == parent && lo < j.hi && hi > j.lo)
                || result
                    .iter()
                    .any(|j| j.incoming(parent).is_some_and(|u| lo < u))
            {
                continue;
            }
            reason.clear();
            let collars = std::array::from_fn(|k| {
                let (pose, u, sign) = if k == 0 {
                    (parent_pose, lo, -1.)
                } else if k == 1 {
                    (parent_pose, hi, 1.)
                } else {
                    (p, start, 1.)
                };
                let f = pose.at(u);
                let (a, b) = pose.radius_at(&recipe.source, u);
                if (a - b).abs() > 1e-8 {
                    reason = "Continuous native junctions require circular woody sections".into();
                }
                crate::mesh_organic::Collar {
                    id: format!("port{k}"),
                    center: f.p.as_vec3().to_array(),
                    normal: (f.t * sign).as_vec3().to_array(),
                    radial: f.n.as_vec3().to_array(),
                    radius: a as f32,
                    samples: super::root_sections::sides(pose, recipe) as u8,
                }
            });
            if !reason.is_empty() {
                break;
            }
            match super::junction_flow::build(
                &collars,
                crate::mesh_organic::Options {
                    refinement_levels: super::junction_quality::level(
                        recipe,
                        [parent_pose.source, p.source],
                        1,
                    ),
                    relaxation_steps: 2,
                    ..Default::default()
                },
                parent_pose.at(p.insertion).p,
                super::junction_flow::strength(recipe, [parent_pose.source, p.source]),
            ) {
                Ok(mesh) => {
                    accepted = Some(Joint {
                        parent,
                        child,
                        lo,
                        hi,
                        start,
                        terminal: false,
                        peer: None,
                        mesh: Some(mesh.into()),
                        basal: false,
                        root_ports: vec![],
                    });
                    break;
                }
                Err(e) => reason = e.message,
            }
        }
        result.push(accepted.ok_or_else(||invalid(format!("Cannot fit continuous junction at {}: {}. Increase branch spacing/angle or reduce radii",recipe.source.organs[p.source].id,reason)))?);
    }
    // An ended parent has one incoming collar and two outgoing collars, rather
    // than the two parent collars of a lateral graft. A pruned sibling leaves
    // a two-port continuation; no intersecting/capped tubes are substituted.
    let mut forks: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, p) in poses.iter().enumerate() {
        if recipe.source.organs[p.source].connection == Connection::Fork {
            forks.entry(p.parent.unwrap()).or_default().push(i);
        }
    }
    for (parent, children) in forks {
        let parent_pose = &poses[parent];
        let child = children[0];
        let peer = children.get(1).copied();
        let r = parent_pose
            .radius_at(&recipe.source, 1.)
            .0
            .max(poses[child].radius)
            .max(peer.map_or(0., |i| poses[i].radius));
        let mut accepted = None;
        let mut reason = "Insufficient space between neighboring collars".to_string();
        'search: for factor in [1.5, 2., 2.8, 4., 6., 9., 13.] {
            for ratio in [0.7, 1., 1.5, 2., 3.] {
                let lo = parent_pose.offset(1., -r * factor);
                let start = poses[child].offset(0., r * factor * ratio);
                let peer_start = peer.map(|i| poses[i].offset(0., r * factor * ratio));
                if lo <= 0.
                    || start >= 0.8
                    || peer_start.is_some_and(|u| u >= 0.8)
                    || result.iter().any(|j| j.parent == child && start >= j.lo)
                    || peer
                        .zip(peer_start)
                        .is_some_and(|(i, u)| result.iter().any(|j| j.parent == i && u >= j.lo))
                    || result.iter().any(|j| j.parent == parent && lo < j.hi)
                    || result
                        .iter()
                        .any(|j| j.incoming(parent).is_some_and(|u| lo <= u))
                {
                    continue;
                }
                let mut mesh = None;
                if let Some(other) = peer {
                    let ports = [
                        (parent, lo, -1.),
                        (child, start, 1.),
                        (other, peer_start.unwrap(), 1.),
                    ];
                    let mut circular = true;
                    let collars = std::array::from_fn(|k| {
                        let (i, u, sign) = ports[k];
                        let p = &poses[i];
                        let f = p.at(u);
                        let (a, b) = p.radius_at(&recipe.source, u);
                        circular &= (a - b).abs() < 1e-8;
                        crate::mesh_organic::Collar {
                            id: format!("port{k}"),
                            center: f.p.as_vec3().to_array(),
                            normal: (f.t * sign).as_vec3().to_array(),
                            radial: f.n.as_vec3().to_array(),
                            radius: a as f32,
                            samples: super::root_sections::sides(p, recipe) as u8,
                        }
                    });
                    if !circular {
                        return Err(invalid("Terminal forks require circular woody sections"));
                    }
                    match super::junction_flow::build(
                        &collars,
                        crate::mesh_organic::Options {
                            refinement_levels: super::junction_quality::level(
                                recipe,
                                [parent_pose.source, poses[child].source, poses[other].source],
                                1,
                            ),
                            relaxation_steps: 2,
                            ..Default::default()
                        },
                        parent_pose.at(1.).p,
                        super::junction_flow::strength(
                            recipe,
                            [parent_pose.source, poses[child].source, poses[other].source],
                        ),
                    ) {
                        Ok(m) => mesh = Some(m.into()),
                        Err(e) => {
                            reason = e.message;
                            continue;
                        }
                    }
                }
                accepted = Some(Joint {
                    parent,
                    child,
                    lo,
                    hi: 1.,
                    start,
                    terminal: true,
                    peer: peer.zip(peer_start),
                    mesh,
                    basal: false,
                    root_ports: vec![],
                });
                break 'search;
            }
        }
        result.push(accepted.ok_or_else(|| {
            invalid(format!(
                "Cannot fit terminal fork at {}: {reason}",
                recipe.source.organs[parent_pose.source].id
            ))
        })?);
    }
    for system in &recipe.source.root_systems {
        let Some(parent) = poses
            .iter()
            .position(|p| recipe.source.organs[p.source].id == system.stem)
        else {
            continue;
        };
        let roots: Vec<_> = poses
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.parent == Some(parent)
                    && recipe.source.organs[p.source].connection == Connection::Basal
            })
            .map(|(i, _)| i)
            .collect();
        if roots.is_empty() {
            continue;
        }
        if super::root_sections::sides(&poses[parent], recipe)
            + roots
                .iter()
                .map(|i| super::root_sections::sides(&poses[*i], recipe))
                .sum::<usize>()
            > 160
        {
            return Err(invalid(
                "Root crown exceeds160 collar samples; lower sides or root count",
            ));
        }
        let p = &poses[parent];
        let r = p.radius;
        let mut accepted = None;
        let mut reason = "Insufficient crown collar spacing".to_string();
        let Shape::Axis {
            radius: source_radius,
            ..
        } = recipe.source.organs[p.source].shape
        else {
            unreachable!()
        };
        let height = super::development::state(recipe)?.map_or(1., |s| s.height);
        let preferred =
            source_radius * system.settings.neck_height * recipe.settings.scale * height / r;
        let preferred = preferred.max(
            system
                .settings
                .buttress
                .as_ref()
                .map_or(0., |f| f.height * 1.15),
        );
        let organic = recipe.source.version >= 7 && system.settings.buttress.is_some();
        let mut organic_reason = None;
        let mut quality = 0.;
        let mut heights = [
            preferred * 0.9,
            preferred * 0.75,
            preferred,
            0.7,
            1.,
            1.3,
            1.6,
            2.,
            2.5,
            3.,
        ]
        .to_vec();
        if organic {
            heights.extend([preferred * 1.15, preferred * 1.35, 4., 5., 6.]);
        }
        'crown: for up in heights {
            let u = p.offset(0., r * up);
            if u >= 0.8 || result.iter().any(|j| j.parent == parent && u >= j.lo) {
                continue;
            }
            for out in [1.15, 1.3, 1.6, 2., 2.5, 3., 3.8, 5., 7.] {
                let ports: Vec<_> = roots
                    .iter()
                    .map(|&i| (i, poses[i].offset(0., r * out)))
                    .collect();
                if ports
                    .iter()
                    .any(|(i, u)| *u >= 0.8 || result.iter().any(|j| j.parent == *i && *u >= j.lo))
                {
                    continue;
                }
                let mut collars = vec![];
                for (k, (i, v)) in std::iter::once((parent, u))
                    .chain(ports.iter().copied())
                    .enumerate()
                {
                    let q = &poses[i];
                    let f = q.at(v);
                    let (a, b) = q.radius_at(&recipe.source, v);
                    if (a - b).abs() > 1e-8 {
                        return Err(invalid("Basal crown requires circular root collars"));
                    }
                    collars.push(crate::mesh_organic::Collar {
                        id: format!("root{k}"),
                        center: f.p.as_vec3().to_array(),
                        normal: f.t.as_vec3().to_array(),
                        radial: f.n.as_vec3().to_array(),
                        radius: a as f32,
                        samples: super::root_sections::sides(q, recipe) as u8,
                    });
                }
                match crate::mesh_organic::junction_many(
                    &collars,
                    crate::mesh_organic::Options {
                        refinement_levels: super::junction_quality::level(
                            recipe,
                            std::iter::once(p.source).chain(roots.iter().map(|&i| poses[i].source)),
                            2,
                        ),
                        relaxation_steps: 2,
                        ..Default::default()
                    },
                ) {
                    Ok(mesh) => {
                        let mut candidate = Joint {
                            parent,
                            child: roots[0],
                            lo: u,
                            hi: u,
                            start: ports[0].1,
                            terminal: false,
                            peer: None,
                            mesh: Some(mesh),
                            basal: true,
                            root_ports: ports,
                        };
                        if organic {
                            match shape_joint(&mut candidate, poses, recipe) {
                                Ok(q) => {
                                    if q > quality {
                                        quality = q;
                                        accepted = Some(candidate);
                                    }
                                    if q >= 1. {
                                        break 'crown;
                                    }
                                    continue;
                                }
                                Err(e) => {
                                    organic_reason.get_or_insert(e.message.clone());
                                    reason = e.message;
                                    continue;
                                }
                            }
                        }
                        accepted = Some(candidate);
                        break 'crown;
                    }
                    Err(e) => reason = e.message,
                }
            }
        }
        result.push(accepted.ok_or_else(|| {
            invalid(format!(
                "Cannot fit root crown {}: {}",
                system.stem,
                organic_reason.unwrap_or(reason)
            ))
        })?);
    }
    for joint in &mut result {
        if joint.basal && recipe.source.version >= 7 && poses[joint.child].section.is_some() {
            continue;
        }
        shape_joint(joint, poses, recipe)?;
    }
    Ok(result)
}
fn shape_joint(joint: &mut Joint, poses: &[Pose], recipe: &Recipe) -> Result<f64> {
    let ports = joint_ports(joint);
    let fused = joint
        .mesh
        .as_ref()
        .is_some_and(|m| m.report.field.is_some());
    if !fused
        && !ports
            .iter()
            .any(|(i, _)| super::root_sections::shaped(&poses[*i]))
    {
        return Ok(1.);
    }
    if let Some(mesh) = &mut joint.mesh {
        let seed = if joint.basal && recipe.source.version >= 7 {
            Some(mesh.geometry.positions.clone())
        } else {
            None
        };
        let targets: Vec<Vec<_>> = ports
            .iter()
            .enumerate()
            .map(|(k, (i, u))| {
                let count = super::root_sections::sides(&poses[*i], recipe);
                (0..count)
                    .map(|slot| {
                        let angular = if k == 0 && !joint.basal {
                            (count - slot) % count
                        } else {
                            slot
                        };
                        super::root_sections::point(
                            &poses[*i],
                            &recipe.source,
                            *u,
                            TAU * angular as f64 / count as f64,
                        )
                    })
                    .collect()
            })
            .collect();
        let mut cuffs = vec![];
        if fused {
            let n = mesh.collar_vertices;
            for (k, boundary) in mesh.boundaries.iter().enumerate() {
                let (i, u) = ports[k];
                let pose = &poses[i];
                let count = boundary.vertices.len();
                for (slot, &vertex) in boundary.vertices.iter().enumerate() {
                    let outer =
                        DVec3::from_array(mesh.geometry.positions[vertex as usize].map(f64::from));
                    let inner_index = n + vertex as usize;
                    let inner =
                        DVec3::from_array(mesh.geometry.positions[inner_index].map(f64::from));
                    let guard = outer.distance(inner);
                    let inner_u = pose.offset(u, if k == 0 { guard } else { -guard });
                    let angular = if k == 0 { (count - slot) % count } else { slot };
                    cuffs.push((
                        inner_index,
                        super::root_sections::point(
                            pose,
                            &recipe.source,
                            inner_u,
                            TAU * angular as f64 / count as f64,
                        ),
                    ));
                }
            }
        }
        super::root_sections::deform_cuffs(mesh, &targets, &cuffs)?;
        if fused {
            let ends: Vec<_> = ports
                .iter()
                .map(|(i, _)| {
                    if *i == joint.parent {
                        if joint.terminal {
                            1.
                        } else {
                            poses[joint.child].insertion
                        }
                    } else {
                        0.
                    }
                })
                .collect();
            super::junction_flow::wood_mass(mesh, poses, &ports, &ends, recipe)?;
        }
        if let Some(seed) = seed {
            return super::root_crown::refine(mesh, &seed, poses, &ports, recipe);
        }
    }
    Ok(1.)
}
fn joint_ports(j: &Joint) -> Vec<(usize, f64)> {
    if j.basal {
        std::iter::once((j.parent, j.lo))
            .chain(j.root_ports.iter().copied())
            .collect()
    } else if j.terminal {
        std::iter::once((j.parent, j.lo))
            .chain(std::iter::once((j.child, j.start)))
            .chain(j.peer)
            .collect()
    } else {
        vec![(j.parent, j.lo), (j.parent, j.hi), (j.child, j.start)]
    }
}
fn tube(
    b: &mut Builder,
    pose: &Pose,
    index: usize,
    recipe: &Recipe,
    joints: &[Joint],
) -> Result<()> {
    let source = &recipe.source;
    let sides = super::root_sections::sides(pose, recipe);
    let steps = pose
        .wood_surface
        .as_ref()
        .map_or(recipe.settings.axis_samples as usize, |s| {
            s.samples as usize
        });
    let start = joints.iter().find_map(|j| j.incoming(index)).unwrap_or(0.);
    let end = joints
        .iter()
        .find(|j| j.parent == index && j.terminal)
        .map_or(1., |j| j.lo);
    let windows: Vec<_> = joints
        .iter()
        .filter(|j| j.parent == index && !j.basal)
        .map(|j| (j.lo, j.hi))
        .collect();
    // Keep exact collar parameters/addresses. A regular sample very close to
    // one can round to the same f32 ring: face() then drops collapsed triangles,
    // leaving two unstitched boundaries. Omit only that redundant grid sample.
    let scale = pose
        .samples
        .iter()
        .map(|f| f.p.abs().max_element() + pose.radius)
        .fold(1., f64::max);
    let clearance = 2. * f64::from(f32::EPSILON) * scale;
    let mut arc = Vec::with_capacity(pose.samples.len());
    arc.push(0.);
    for pair in pose.samples.windows(2) {
        arc.push(arc.last().unwrap() + pair[0].p.distance(pair[1].p));
    }
    let metric = |u: f64| {
        let x = u * (arc.len() - 1) as f64;
        let i = (x as usize).min(arc.len() - 2);
        arc[i] + (arc[i + 1] - arc[i]) * (x - i as f64)
    };
    let boundaries: Vec<_> = [start, end]
        .into_iter()
        .chain(windows.iter().flat_map(|(a, c)| [*a, *c]))
        .map(metric)
        .collect();
    let detail_coordinates = super::wood_details::coordinates(pose);
    let mut coordinates: Vec<_> = (0..=steps)
        .map(|i| i as f64 / steps as f64)
        .filter(|u| *u >= start && *u <= end && !windows.iter().any(|(a, c)| *u > *a && *u < *c))
        .filter(|u| {
            let s = metric(*u);
            !boundaries.iter().any(|b| (s - b).abs() < clearance)
                && !detail_coordinates
                    .iter()
                    .any(|b| (s - metric(*b)).abs() < clearance)
        })
        .collect();
    let mut previous_detail = None;
    for u in detail_coordinates {
        if u <= start || u >= end || windows.iter().any(|(a, c)| u > *a && u < *c) {
            continue;
        }
        let d = metric(u);
        if boundaries.iter().any(|b| (d - b).abs() < clearance)
            || previous_detail.is_some_and(|p: f64| (p - d).abs() < clearance)
        {
            continue;
        }
        coordinates.push(u);
        previous_detail = Some(d);
    }
    coordinates.push(start);
    coordinates.push(end);
    for (a, c) in &windows {
        coordinates.push(*a);
        if *c <= end {
            coordinates.push(*c);
        }
    }
    coordinates.sort_by(f64::total_cmp);
    coordinates.dedup();
    let mut rings = vec![];
    for &u in &coordinates {
        let mut ring = vec![];
        for j in 0..sides {
            let theta = TAU * j as f64 / sides as f64;
            ring.push(b.vertex(
                index,
                pose,
                u,
                super::root_sections::point(pose, source, u, theta),
                surface_address(pose, recipe, u, j),
                recipe,
            )?);
        }
        rings.push(ring);
    }
    for i in 0..rings.len() - 1 {
        let mid = (coordinates[i] + coordinates[i + 1]) * 0.5;
        if windows.iter().any(|(a, c)| mid > *a && mid < *c) {
            continue;
        }
        for j in 0..sides {
            let k = (j + 1) % sides;
            if recipe.version >= 13 {
                let a = DVec3::from_array(b.positions[rings[i][j] as usize].map(f64::from));
                let c = DVec3::from_array(b.positions[rings[i + 1][j] as usize].map(f64::from));
                if (c - a).dot(pose.at(mid).t) <= 0. {
                    return Err(invalid(format!(
                        "Woody sweep folds back at {}: smooth the axis bend or use a junction volume",
                        source.organs[pose.source].id
                    )));
                }
            }
            b.face(rings[i][j], rings[i][k], rings[i + 1][k], index)?;
            b.face(rings[i][j], rings[i + 1][k], rings[i + 1][j], index)?;
        }
    }
    for (i, flip) in [(0, true), (rings.len() - 1, false)] {
        if (flip && start > 0.) || (!flip && end < 1.) {
            continue;
        }
        let u = coordinates[i];
        let center = b.vertex(
            index,
            pose,
            u,
            super::root_sections::center(pose, source, u),
            surface_address(pose, recipe, u, 999),
            recipe,
        )?;
        for j in 0..sides {
            let k = (j + 1) % sides;
            let (a, c) = if flip {
                (rings[i][k], rings[i][j])
            } else {
                (rings[i][j], rings[i][k])
            };
            b.face(center, a, c, index)?;
        }
    }
    Ok(())
}
fn stitch(b: &mut Builder, j: &Joint, poses: &[Pose], recipe: &Recipe) -> Result<()> {
    let Some(mesh) = &j.mesh else {
        // Two-port skin after pruning one continuation. Align transported ring
        // phases by the shortest correspondence, retaining material addresses.
        let mut rings = vec![];
        for (organ, u) in [(j.parent, j.lo), (j.child, j.start)] {
            let p = &poses[organ];
            let sides = super::root_sections::sides(p, recipe);
            let mut ring = vec![];
            for k in 0..sides {
                let theta = TAU * k as f64 / sides as f64;
                ring.push(b.vertex(
                    organ,
                    p,
                    u,
                    super::root_sections::point(p, &recipe.source, u, theta),
                    surface_address(p, recipe, u, k),
                    recipe,
                )?);
            }
            rings.push(ring);
        }
        let sides = rings[0].len();
        if sides != rings[1].len() {
            return Err(invalid("Pruned root collar resolution mismatch"));
        }
        let shift = (0..sides)
            .min_by(|a, c| {
                let cost = |s: usize| {
                    (0..sides)
                        .map(|k| {
                            let p = glam::Vec3::from_array(b.positions[rings[0][k] as usize]);
                            p.distance_squared(glam::Vec3::from_array(
                                b.positions[rings[1][(k + s) % sides] as usize],
                            )) as f64
                        })
                        .sum::<f64>()
                };
                cost(*a).total_cmp(&cost(*c))
            })
            .unwrap();
        for k in 0..sides {
            let n = (k + 1) % sides;
            b.face(
                rings[0][k],
                rings[0][n],
                rings[1][(n + shift) % sides],
                j.parent,
            )?;
            b.face(
                rings[0][k],
                rings[1][(n + shift) % sides],
                rings[1][(k + shift) % sides],
                j.parent,
            )?;
        }
        return Ok(());
    };
    let mut map = vec![0; mesh.geometry.positions.len()];
    for k in 0..mesh.boundaries.len() {
        let (organ, u) = if j.basal {
            if k == 0 {
                (j.parent, j.lo)
            } else {
                j.root_ports[k - 1]
            }
        } else if k == 0 {
            (j.parent, j.lo)
        } else if j.terminal && k == 1 {
            (j.child, j.start)
        } else if j.terminal {
            j.peer.unwrap()
        } else if k == 1 {
            (j.parent, j.hi)
        } else {
            (j.child, j.start)
        };
        let pose = &poses[organ];
        let sides = mesh.boundaries[k].vertices.len();
        for (slot, &source) in mesh.boundaries[k].vertices.iter().enumerate() {
            let angular = if k == 0 && !j.basal {
                (sides - slot) % sides
            } else {
                slot
            };
            map[source as usize] = b.vertex(
                organ,
                pose,
                u,
                DVec3::from_array(mesh.geometry.positions[source as usize].map(f64::from)),
                surface_address(pose, recipe, u, angular),
                recipe,
            )?;
        }
    }
    // Interior points belong to this explicit joint domain. Changing a collar invalidates their addresses.
    let child_id = &recipe.source.organs[poses[j.child].source].id;
    let domain = if j.basal {
        let mut h = ring::digest::Context::new(&ring::digest::SHA256);
        h.update(b"root-crown-v1");
        h.update(&[recipe.settings.sides]);
        for tri in &mesh.geometry.triangles {
            for index in tri {
                h.update(&index.to_le_bytes());
            }
        }
        for (i, u) in &j.root_ports {
            let id = &recipe.source.organs[poses[*i].source].id;
            h.update(&(id.len() as u64).to_le_bytes());
            h.update(id.as_bytes());
            h.update(&u.to_bits().to_le_bytes());
        }
        h.update(&j.lo.to_bits().to_le_bytes());
        format!(
            "{}:root-crown:{}",
            recipe.source.organs[poses[j.parent].source].id,
            h.finish()
                .as_ref()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    } else if j.terminal {
        let (other, u) = j.peer.unwrap();
        format!(
            "{child_id}:fork:{:016x}:{:016x}:{}:{:016x}",
            j.lo.to_bits(),
            j.start.to_bits(),
            recipe.source.organs[poses[other].source].id,
            u.to_bits()
        )
    } else {
        format!(
            "{child_id}:joint:{:016x}:{:016x}",
            j.lo.to_bits(),
            j.start.to_bits()
        )
    };
    let flow = super::junction_flow::strength(
        recipe,
        joint_ports(j).iter().map(|(i, _)| poses[*i].source),
    )
    .is_some();
    let domain = if flow
        || joint_ports(j)
            .iter()
            .any(|(i, _)| super::root_sections::shaped(&poses[*i]))
    {
        let mut h = ring::digest::Context::new(&ring::digest::SHA256);
        for boundary in &mesh.boundaries {
            h.update(&(boundary.vertices.len() as u64).to_le_bytes());
        }
        for tri in &mesh.geometry.triangles {
            for index in tri {
                h.update(&index.to_le_bytes());
            }
        }
        let topology = h
            .finish()
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        format!(
            "{domain}:{}:{}:{topology}",
            if flow { "flow-v1" } else { "wall-v1" },
            recipe.settings.sides
        )
    } else {
        domain
    };
    let domain = if j.basal && recipe.source.version >= 7 && poses[j.child].section.is_some() {
        format!("{domain}:valley-v1")
    } else {
        domain
    };
    for (i, point) in mesh
        .geometry
        .positions
        .iter()
        .enumerate()
        .skip(mesh.collar_vertices)
    {
        let u = if j.basal { j.lo } else { (j.lo + j.hi) * 0.5 };
        map[i] = b.vertex(
            j.parent,
            &poses[j.parent],
            u,
            DVec3::from_array(point.map(f64::from)),
            format!("{domain}:{i}"),
            recipe,
        )?;
    }
    for &[a, c, d] in &mesh.geometry.triangles {
        b.face(map[a as usize], map[c as usize], map[d as usize], j.parent)?;
    }
    if flow {
        let mut coordinates = vec![DVec3::ZERO; mesh.geometry.positions.len()];
        for boundary in &mesh.boundaries {
            for &i in &boundary.vertices {
                coordinates[i as usize] = DVec3::from_array(
                    wood_coordinate(&b.vertices[map[i as usize] as usize], poses, recipe)
                        .map(f64::from),
                );
            }
        }
        let mut adjacent = vec![BTreeSet::new(); coordinates.len()];
        for &[a, c, d] in &mesh.geometry.triangles {
            for (i, k) in [(a, c), (c, d), (d, a)] {
                adjacent[i as usize].insert(k as usize);
                adjacent[k as usize].insert(i as usize);
            }
        }
        let mut next = coordinates.clone();
        for _ in 0..160 {
            for i in mesh.collar_vertices..coordinates.len() {
                next[i] = adjacent[i].iter().map(|&k| coordinates[k]).sum::<DVec3>()
                    / adjacent[i].len() as f64;
            }
            std::mem::swap(&mut coordinates, &mut next);
        }
        for (i, q) in coordinates.iter().enumerate().skip(mesh.collar_vertices) {
            b.flow_coordinates.insert(map[i], q.as_vec3().to_array());
        }
    }
    Ok(())
}
fn wood_coordinate(v: &Vertex, poses: &[Pose], recipe: &Recipe) -> [f32; 3] {
    let p = &poses[v.organ];
    let angle = v.local.z.atan2(v.local.y);
    if recipe.wood_details.is_some() {
        let a = p.wood_surface.as_ref().map_or(angle, |s| {
            let a = super::wood_details::grain_angle(s, &p.patches, v.u, angle);
            if recipe.version >= 13 {
                super::junction_flow::grain_angle(s, v.u, a)
            } else {
                a
            }
        }) - super::wood_details::root_profile(recipe, &recipe.source.organs[p.source].id)
            .map_or(0., |r| r.twist * v.u * p.length / recipe.settings.scale);
        [
            (a.sin() * p.radius) as f32,
            (v.u * p.length) as f32,
            (a.cos() * p.radius) as f32,
        ]
    } else {
        [
            (angle * p.radius) as f32,
            (v.u * p.length) as f32,
            v.local.z as f32,
        ]
    }
}
fn lamina(b: &mut Builder, pose: &Pose, index: usize, recipe: &Recipe) -> Result<()> {
    let organ = &recipe.source.organs[pose.source];
    let Shape::Blade { outline, .. } = &organ.shape else {
        unreachable!()
    };
    let (nu, nv) = if matches!(outline, Outline::Palmate { .. }) {
        (4, 48)
    } else {
        (recipe.settings.blade_samples as usize, 4)
    };
    let contour = matches!(outline, Outline::Palmate { .. }).then(|| PalmateContour::new(outline));
    let mut layers = vec![];
    for side in 0..2 {
        let mut grid = vec![];
        for i in 0..=nu {
            let raw = i as f64 / nu as f64;
            // A rounded beta tip has high curvature near its endpoints. Spend
            // the existing sample budget there instead of leaving a long spike.
            let u = if recipe.version >= 13
                && matches!(outline, Outline::Strip { tip_power, .. } if *tip_power <= 0.5)
            {
                (1. - (std::f64::consts::PI * raw).cos()) * 0.5
            } else {
                raw
            };
            let mut row = vec![];
            for j in 0..=nv {
                let v = if i == 0 || (i == nu && !matches!(outline, Outline::Palmate { .. })) {
                    0.
                } else if j == nv && matches!(outline, Outline::Palmate { .. }) {
                    -1.
                } else {
                    -1. + 2. * j as f64 / nv as f64
                };
                let (p, along) = blade_point(pose, organ, u, v, contour.as_ref());
                let f = pose.at(along);
                let offset = if side == 0 { 0.5 } else { -0.5 };
                let key = format!(
                    "{}:leaf:{side}:{:016x}:{:016x}",
                    organ.id,
                    u.to_bits(),
                    v.to_bits()
                );
                row.push(b.vertex(
                    index,
                    pose,
                    along,
                    p + f.b() * (2. * pose.radius * offset),
                    key,
                    recipe,
                )?);
            }
            grid.push(row);
        }
        for i in 0..nu {
            for j in 0..nv {
                let [a, c, d, e] = [
                    grid[i][j],
                    grid[i + 1][j],
                    grid[i + 1][j + 1],
                    grid[i][j + 1],
                ];
                if side == 0 {
                    b.face(a, c, d, index)?;
                    b.face(a, d, e, index)?;
                } else {
                    b.face(a, d, c, index)?;
                    b.face(a, e, d, index)?;
                }
            }
        }
        layers.push(grid);
    }
    let mut edge = |a: u32, c: u32, d: u32, e: u32| -> Result<()> {
        b.face(a, c, d, index)?;
        b.face(a, d, e, index)
    };
    if matches!(outline, Outline::Palmate { .. }) {
        for j in 0..nv {
            edge(
                layers[0][nu][j],
                layers[1][nu][j],
                layers[1][nu][j + 1],
                layers[0][nu][j + 1],
            )?;
        }
    } else {
        for i in 0..nu {
            edge(
                layers[0][i][0],
                layers[1][i][0],
                layers[1][i + 1][0],
                layers[0][i + 1][0],
            )?;
            edge(
                layers[0][i + 1][nv],
                layers[1][i + 1][nv],
                layers[1][i][nv],
                layers[0][i][nv],
            )?;
        }
    }
    Ok(())
}
pub fn compile(recipe: &Recipe) -> Result<Compiled> {
    let poses = poses(recipe)?;
    let age = super::development::state(recipe)?;
    let estimated = poses
        .iter()
        .map(|p| {
            if p.width == 0. {
                (p.wood_surface
                    .as_ref()
                    .map_or(recipe.settings.axis_samples as usize, |s| {
                        s.samples as usize
                    })
                    + 1
                    + p.patches.len() * 17)
                    * super::root_sections::sides(p, recipe)
                    + 2
            } else {
                if matches!(
                    recipe.source.organs[p.source].shape,
                    Shape::Blade {
                        outline: Outline::Palmate { .. },
                        ..
                    }
                ) {
                    386
                } else {
                    (recipe.settings.blade_samples as usize + 1) * 10 - 16
                }
            }
        })
        .sum::<usize>();
    if estimated > MAX_VERTICES {
        return Err(invalid("Botanical preflight exceeds vertex budget"));
    }
    let joints = junctions(&poses, recipe)?;
    let mut builders = [Builder::new(), Builder::new()];
    for (i, p) in poses.iter().enumerate() {
        if p.width == 0. {
            tube(&mut builders[0], p, i, recipe, &joints)?;
        } else {
            lamina(&mut builders[1], p, i, recipe)?;
        }
    }
    for joint in &joints {
        stitch(&mut builders[0], joint, &poses, recipe)?;
    }
    for builder in &mut builders {
        for (position, vertex) in builder.positions.iter_mut().zip(&mut builder.vertices) {
            let pose = &poses[vertex.organ];
            if let Some(field) = &pose.flutes {
                if !field.active(DVec3::from_array(position.map(f64::from)), 0.) {
                    continue;
                }
                let frame = pose.at(vertex.u);
                let edit = recipe
                    .manual
                    .get(&vertex.key)
                    .map_or(DVec3::ZERO, |v| frame.vector(DVec3::from_array(*v)));
                let point = field.warp(DVec3::from_array(position.map(f64::from)) - edit) + edit;
                *position = point.as_vec3().to_array();
                vertex.local = frame.local(point - frame.p);
            }
        }
    }
    if builders.iter().map(|b| b.positions.len()).sum::<usize>() > MAX_VERTICES
        || builders.iter().map(|b| b.triangles.len()).sum::<usize>() > MAX_TRIANGLES
    {
        return Err(invalid("Botanical total mesh budget exceeded"));
    }
    for key in recipe.manual.keys() {
        if !builders.iter().any(|b| b.keys.contains_key(key)) {
            return Err(invalid(format!(
                "Regeneration would lose adjusted material point: {key}"
            )));
        }
    }
    let mut document = meshes::Document::default();
    let mut vertices = [vec![], vec![]];
    let mut owners = [vec![], vec![]];
    for (i, b) in builders.into_iter().enumerate() {
        let id = recipe.parts()[i].clone();
        let geometry = meshes::Geometry {
            positions: b.positions,
            triangles: b.triangles,
            material_coordinates: if !b.flow_coordinates.is_empty()
                || recipe.wood_details.is_some()
                || recipe
                    .living
                    .as_ref()
                    .is_some_and(|p| (6..=8).contains(&p.definition.version))
            {
                Some(
                    b.vertices
                        .iter()
                        .enumerate()
                        .map(|(index, v)| {
                            let p = &poses[v.organ];
                            if i == 0 {
                                b.flow_coordinates
                                    .get(&(index as u32))
                                    .copied()
                                    .unwrap_or_else(|| wood_coordinate(v, &poses, recipe))
                            } else {
                                [(v.u * p.length) as f32, v.local.z as f32, v.local.y as f32]
                            }
                        })
                        .collect(),
                )
            } else {
                None
            },
        };
        // Empty foliage remains a tiny hidden placeholder so source ownership stays stable.
        let geometry = if geometry.positions.is_empty() {
            meshes::Geometry {
                positions: vec![[0.; 3], [0.00001, 0., 0.], [0., 0., 0.00001]],
                triangles: vec![[0, 1, 2]],
                material_coordinates: None,
            }
        } else {
            geometry
        };
        document.assets.push(meshes::Asset {
            id: id.clone(),
            geometry: Arc::new(geometry),
            shading: meshes::Shading::Smooth,
            material: crate::materials::Material {
                pattern: if i == 0 {
                    recipe.settings.bark_pattern.map(|mut p| {
                        if let (Some(age), Some(profile)) =
                            (age, recipe.source.development.as_ref())
                        {
                            p.bump *= (1. + profile.bark_gain * age.maturity) as f32;
                        }
                        p
                    })
                } else {
                    recipe.settings.leaf_pattern
                },
                reflectance: Some(crate::materials::Reflectance {
                    roughness: if i == 0 { 0.85 } else { 0.62 },
                    f0: 0.03,
                    metallic: 0.,
                }),
                ..Default::default()
            },
            rig: None,
            clips: vec![],
        });
        document.instances.push(meshes::Instance {
            id: id.clone(),
            asset: id,
            position: recipe.settings.position,
            rotation: [0., 0., 0., 1.],
            scale: 1.,
            color: if i == 0 {
                recipe.settings.bark_color
            } else {
                recipe.settings.leaf_color
            },
            playback: None,
            attachment: None,
        });
        vertices[i] = b.vertices;
        owners[i] = b.owners;
    }
    if recipe
        .living
        .as_ref()
        .is_some_and(|p| p.definition.version == 5)
    {
        let mut bark = crate::materials::library::Preset::Wood.recipe();
        bark.seed = recipe.source.seed as u32;
        bark.tile_size = [4., 60.];
        bark.height_m = 0.045;
        bark.base.color_roughness = [0.25, 0.12, 0.07, 0.95];
        bark.layers[0].frequency = [28, 2];
        bark.layers[0].target.color_roughness = [0.9, 0.58, 0.33, 0.88];
        bark.layers[1].target.color_roughness = [0.5, 0.29, 0.16, 0.95];
        document.assets[0].material = crate::materials::library::material(
            bark,
            crate::materials::library::Execution::Runtime,
        )?;
    }
    super::reproduction::append(recipe, &poses, &mut document)?;
    document.validate()?;
    Ok(Compiled {
        document,
        poses,
        vertices,
        triangles_by_organ: owners,
        joints: joints.len(),
    })
}
