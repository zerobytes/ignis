//! Camera/session and bounded global visual coverage. Physics never follows this camera.
use super::{
    Document, Pose, Relief, Sampler,
    mesh::Face,
    residency::{Anchor, FACES, Tile},
};
use crate::{
    meshes::{Asset, Geometry, Instance, Shading},
    model::{Result, invalid},
};
use glam::{DMat3, DVec3, Mat4};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap},
    sync::Arc,
};
mod topology;
const N: u32 = 32;
type Key = [i64; 3];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub distance_meters: f64,
    pub yaw_degrees: f64,
    pub tilt_degrees: f64,
    pub fov_degrees: f64,
    pub viewport: [u32; 2],
    pub max_error_pixels: f64,
    pub max_tiles: usize,
    /// Sampling per tile; canonical physical grid resolution stays unchanged.
    pub tile_subdivisions: u16,
    pub atmosphere_optimization: bool,
    /// GPU submission grouping, preserving source tiles and their sample grid.
    pub render_group_size: usize,
    /// Diagnostic oracle: false retains the original full sampling on seam changes.
    pub reuse_vertex_samples: bool,
    /// Diagnostic oracle for the former expanded CPU frame.
    pub canonical_cpu_stream: bool,
    pub atmosphere_height_meters: f64,
    /// Vertical optical depth of a homogeneous shell, not scattering/cloud simulation.
    pub atmosphere_optical_depth: Option<f64>,
    pub space_color: [f32; 3],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            distance_meters: 0.,
            yaw_degrees: 0.,
            tilt_degrees: 0.,
            fov_degrees: 60.,
            viewport: [1280, 800],
            max_error_pixels: 8.,
            max_tiles: 768,
            tile_subdivisions: N as u16,
            atmosphere_optimization: true,
            render_group_size: 8,
            reuse_vertex_samples: true,
            canonical_cpu_stream: true,
            atmosphere_height_meters: 8000.,
            atmosphere_optical_depth: None,
            space_color: [0.001, 0.002, 0.004],
        }
    }
}
impl Settings {
    pub fn validate(&self, doc: &Document) -> Result<()> {
        if !self.distance_meters.is_finite()
            || !(0. ..=1e18).contains(&self.distance_meters)
            || !self.yaw_degrees.is_finite()
            || !self.tilt_degrees.is_finite()
            || !(-85. ..=165.).contains(&self.tilt_degrees)
            || !self.fov_degrees.is_finite()
            || !(10. ..=120.).contains(&self.fov_degrees)
            || self.viewport.iter().any(|v| !(64..=16384).contains(v))
            || !self.max_error_pixels.is_finite()
            || !(0.5..=64.).contains(&self.max_error_pixels)
            || !(6..=1536).contains(&self.max_tiles)
            || ![32, 64].contains(&self.tile_subdivisions)
            || ![1, 8].contains(&self.render_group_size)
            || !self.atmosphere_height_meters.is_finite()
            || !(1. ..=doc.radius()).contains(&self.atmosphere_height_meters)
            || self
                .atmosphere_optical_depth
                .is_some_and(|v| !v.is_finite() || !(0. ..=8.).contains(&v))
            || self
                .space_color
                .iter()
                .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        {
            return Err(invalid(
                "Planet view needs distance0..1e18m, finite yaw/tilt[-85,165], FOV10..120, viewport64..16384, error0.5..64px,6..1536tiles and a finite atmosphere shell",
            ));
        }
        // Certify the whole orbit envelope, not just one lucky yaw near cell overflow.
        let extent = doc.radius() * 1.25 + self.distance_meters + 2000.;
        for a in 0..3 {
            for sign in [-1., 1.] {
                let mut d = [0.; 3];
                d[a] = sign * extent;
                doc.center
                    .translated(d)
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        Ok(())
    }
}
struct Source {
    grid: u32,
    operations: [super::Prepared; super::MAX_OPERATIONS],
    lower: f64,
    upper: f64,
}
impl Source {
    fn new(doc: &Document, grid: u32) -> Result<Self> {
        let sampler = Sampler::new(doc)?;
        let (lower, upper) = height_bounds(doc, DVec3::Y, std::f64::consts::PI);
        Ok(Self {
            grid,
            operations: sampler.operations,
            lower,
            upper,
        })
    }
    fn sampler<'a>(&self, doc: &'a Document) -> Sampler<'a> {
        Sampler {
            document: doc,
            operations: self.operations,
        }
    }
}
#[derive(Clone)]
pub(crate) struct View {
    pub eye: DVec3,    // actor anchor coordinates, metres
    pub basis: DMat3,  // view axes expressed in actor anchor coordinates
    pub center: DVec3, // planet center in view coordinates
    pub eye_radial: DVec3,
    pub altitude: f64,
    pub position: crate::world_space::Position,
    pub clip: crate::render::Clip,
    // Angular shadow of a sphere guaranteed to be inside the authored terrain.
    horizon: Option<(DVec3, f64)>,
}
impl View {
    fn new(
        doc: &Document,
        anchor: Anchor,
        settings: &Settings,
        eye_height: f64,
        outer: f64,
        fraction: f64,
        source: &Source,
    ) -> Result<Self> {
        let d = settings.distance_meters;
        let yaw = settings.yaw_degrees.to_radians();
        let tilt = settings.tilt_degrees.clamp(-85., 85.).to_radians();
        let tangent = DVec3::new(yaw.sin(), 0., -yaw.cos());
        let right = DVec3::new(yaw.cos(), 0., yaw.sin());
        let forward = tangent * tilt.cos() + DVec3::Y * tilt.sin();
        let eyes = DVec3::Y * eye_height;
        let mut eye = eyes + (desired_offset(doc, settings)) * fraction;
        let (unit, height) = anchor.unproject(eye);
        // Camera clearance only. Never move/snap the actor or use this as KCC support.
        if d > 125. {
            let surface = source.sampler(doc).height(unit) + 1.;
            if height < surface {
                eye = anchor.project(unit, surface);
            }
        }
        let focus = 1000. / (1. + d / 20.).powi(2);
        let center = DVec3::new(0., -(doc.radius() + anchor.pose.altitude_meters), 0.);
        let mix = smooth((d / doc.radius() - 0.02) / 1.98);
        let target = (eyes + forward * focus).lerp(center, mix);
        let f = (target - eye).normalize();
        let up = right.cross(f).normalize();
        let basis = DMat3::from_cols(right, up, -f);
        let view_center = basis.transpose() * (center - eye);
        let (unit, altitude) = anchor.unproject(eye);
        let actor_global = doc
            .center
            .translated(
                (anchor.basis.y_axis * (doc.radius() + anchor.pose.altitude_meters)).to_array(),
            )
            .map_err(|e| invalid(e.to_string()))?;
        let position = actor_global
            .translated((anchor.basis * eye).to_array())
            .map_err(|e| invalid(e.to_string()))?;
        let distance = view_center.length();
        let (lowest, highest) = (source.lower, source.upper);
        // Even the coarsest allowed chordal mesh encloses this sphere. The
        // numerical guard covers f32 vertex/placement roundoff, not a distance cut.
        let inner = (doc.radius() + lowest) * (2. * 2f64.sqrt() / f64::from(source.grid)).cos()
            - doc.radius() * 1e-5;
        let horizon = if distance > inner {
            Some((
                unit,
                (inner / distance).clamp(-1., 1.).acos()
                    + (inner / (doc.radius() + highest)).clamp(-1., 1.).acos(),
            ))
        } else {
            None
        };
        let aspect = f64::from(settings.viewport[0]) / f64::from(settings.viewport[1]);
        let tan = (settings.fov_degrees.to_radians() * 0.5).tan();
        let corner_cos = 1. / (1. + tan * tan * (1. + aspect * aspect)).sqrt();
        let near = ((distance - outer).max(0.) * corner_cos * 0.25).max(0.05);
        let far = (distance + outer).max(near * 2.) * 1.1 + 10.;
        let clip = crate::render::Clip {
            near: near as f32,
            far: far as f32,
            reverse_z: true,
        };
        clip.validate()?;
        Ok(Self {
            eye,
            basis,
            center: view_center,
            eye_radial: basis.transpose() * anchor.basis.transpose() * unit,
            altitude,
            position,
            clip,
            horizon,
        })
    }
}
pub(crate) fn desired_offset(doc: &Document, s: &Settings) -> DVec3 {
    let d = s.distance_meters;
    let yaw = s.yaw_degrees.to_radians();
    let tangent = DVec3::new(yaw.sin(), 0., -yaw.cos());
    let auto = (d / 100.).atan().min(80f64.to_radians());
    let minimum = (2. - 87. * smooth(d / doc.radius() - 2.)).to_radians();
    let elevation = (auto - s.tilt_degrees.to_radians()).clamp(minimum, 85f64.to_radians());
    -tangent * d * elevation.cos() + DVec3::Y * d * elevation.sin()
}
fn smooth(t: f64) -> f64 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
pub(crate) fn height_bounds(doc: &Document, unit: DVec3, angle: f64) -> (f64, f64) {
    let mut lo = doc.base_height_meters;
    let mut hi = lo;
    for op in &doc.operations {
        match *op {
            Relief::Noise {
                amplitude_meters, ..
            } => {
                lo -= amplitude_meters.abs();
                hi += amplitude_meters.abs();
            }
            Relief::Raise {
                direction,
                angular_radius_degrees,
                height_meters,
            } => {
                let theta = unit
                    .dot(DVec3::from(direction).normalize())
                    .clamp(-1., 1.)
                    .acos();
                let r = angular_radius_degrees.to_radians();
                let weight = |a: f64| {
                    if a >= r {
                        0.
                    } else {
                        (1. - (a / r).powi(2)).powi(2)
                    }
                };
                let a = height_meters * weight((theta - angle).max(0.));
                let b = height_meters * weight((theta + angle).min(std::f64::consts::PI));
                lo += a.min(b);
                hi += a.max(b);
            }
        }
    }
    (lo, hi)
}
fn error(doc: &Document, tile: Tile, max_level: u8, n: u32) -> f64 {
    if tile.level == max_level {
        return 0.;
    } // Canonical full-resolution grid of the physical source.
    let angle = 2. * 2f64.sqrt() / (f64::from(n) * (1u64 << tile.level) as f64);
    let diagonal = angle * doc.radius();
    let (c, cone) = tile.cone();
    let mut residual = 0.;
    let mut amplitude = doc.base_height_meters.abs();
    for op in &doc.operations {
        match *op {
            Relief::Noise {
                amplitude_meters,
                wavelength_meters,
                octaves,
                persistence,
                ..
            } => {
                let total = (0..octaves)
                    .map(|i| persistence.powi(i32::from(i)))
                    .sum::<f64>();
                for i in 0..octaves {
                    let a = amplitude_meters.abs() * persistence.powi(i32::from(i)) / total;
                    residual += (2. * a).min(
                        3. * 3f64.sqrt() * a / wavelength_meters
                            * 2f64.powi(i32::from(i))
                            * diagonal,
                    );
                }
                amplitude += amplitude_meters.abs();
            }
            Relief::Raise {
                direction,
                angular_radius_degrees,
                height_meters,
            } => {
                let theta = c
                    .dot(DVec3::from(direction).normalize())
                    .clamp(-1., 1.)
                    .acos();
                let r = angular_radius_degrees.to_radians();
                if theta - cone < r {
                    residual += height_meters.abs().min(
                        8. / (3. * 3f64.sqrt()) * height_meters.abs() / (r * doc.radius())
                            * diagonal,
                    );
                }
                amplitude += height_meters.abs();
            }
        }
    }
    residual + (doc.radius() + amplitude) * angle * angle * 0.5
}
/// Exact component extrema of a normalized cube rectangle occur at endpoints or zero.
fn global_bounds(doc: &Document, t: Tile) -> [DVec3; 8] {
    let n = (1u64 << t.level) as f64;
    let u = [
        2. * f64::from(t.xy[0]) / n - 1.,
        2. * f64::from(t.xy[0] + 1) / n - 1.,
    ];
    let v = [
        2. * f64::from(t.xy[1]) / n - 1.,
        2. * f64::from(t.xy[1] + 1) / n - 1.,
    ];
    let mut lo = DVec3::splat(f64::INFINITY);
    let mut hi = -lo;
    for x in [u[0], u[1], 0f64.clamp(u[0], u[1])] {
        for y in [v[0], v[1], 0f64.clamp(v[0], v[1])] {
            let p = face_direction(t.face, x, y);
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    let (unit, cone) = t.cone();
    let (h0, h1) = height_bounds(doc, unit, cone);
    let r0 = doc.radius() + h0;
    let r1 = doc.radius() + h1;
    let a = (lo * r0).min(lo * r1).min(hi * r0).min(hi * r1);
    let b = (lo * r0).max(lo * r1).max(hi * r0).max(hi * r1);
    std::array::from_fn(|i| {
        DVec3::new(
            if i & 1 == 0 { a.x } else { b.x },
            if i & 2 == 0 { a.y } else { b.y },
            if i & 4 == 0 { a.z } else { b.z },
        )
    })
}
fn face_direction(face: Face, u: f64, v: f64) -> DVec3 {
    let p = match face {
        Face::PositiveX => DVec3::new(1., v, -u),
        Face::NegativeX => DVec3::new(-1., v, u),
        Face::PositiveY => DVec3::new(u, 1., -v),
        Face::NegativeY => DVec3::new(u, -1., v),
        Face::PositiveZ => DVec3::new(u, v, 1.),
        Face::NegativeZ => DVec3::new(-u, v, -1.),
    };
    p.normalize()
}
fn projected(doc: &Document, t: Tile, max: u8, anchor: Anchor, view: &View, s: &Settings) -> f64 {
    if hidden(t, view) {
        return 0.;
    }
    projected_bounds(
        &global_bounds(doc, t),
        error(doc, t, max, grid_side(s, max)),
        anchor,
        view,
        s,
    )
}
fn hidden(t: Tile, v: &View) -> bool {
    let Some((eye, angle)) = v.horizon else {
        return false;
    };
    let (unit, cone) = t.cone();
    unit.dot(eye).clamp(-1., 1.).acos() - cone > angle + 1e-10
}
fn projected_bounds(
    bounds: &[DVec3; 8],
    error: f64,
    anchor: Anchor,
    view: &View,
    s: &Settings,
) -> f64 {
    let transform = view.basis.transpose() * anchor.basis.transpose();
    let offset = view.basis.transpose()
        * (-DVec3::Y * (anchor.radius + anchor.pose.altitude_meters) - view.eye);
    let b = bounds.map(|p| transform * p + offset);
    let tan = (s.fov_degrees.to_radians() * 0.5).tan();
    let aspect = f64::from(s.viewport[0]) / f64::from(s.viewport[1]);
    if b.iter().all(|p| p.z >= 0.)
        || b.iter().all(|p| p.x < p.z * tan * aspect)
        || b.iter().all(|p| p.x > -p.z * tan * aspect)
        || b.iter().all(|p| p.y < p.z * tan)
        || b.iter().all(|p| p.y > -p.z * tan)
    {
        return 0.;
    }
    let lo = b.iter().fold(DVec3::splat(f64::INFINITY), |a, b| a.min(*b));
    let hi = b
        .iter()
        .fold(DVec3::splat(f64::NEG_INFINITY), |a, b| a.max(*b));
    let nearest = DVec3::ZERO
        .clamp(lo, hi)
        .length()
        .max(f64::from(view.clip.near));
    // Perspective displacement grows toward viewport corners; Euclidean
    // distance alone underestimates it. Bound both projection derivatives.
    let corner_sec_squared = 1. + tan * tan * (1. + aspect * aspect);
    error * f64::from(s.viewport[1]) * corner_sec_squared / (2. * tan * nearest)
}
fn grid_side(s: &Settings, _max: u8) -> u32 {
    u32::from(s.tile_subdivisions)
}
pub(crate) struct ActorView {
    pub size: [f32; 3],
    pub color: [f32; 3],
    pub eye: f64,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceKey {
    tile: Tile,
    grid: u32,
    signature: Vec<(Key, u8)>,
}
#[derive(Clone, Copy)]
struct Priority {
    tile: Tile,
    error: f64,
}
impl PartialEq for Priority {
    fn eq(&self, b: &Self) -> bool {
        self.tile == b.tile && self.error == b.error
    }
}
impl Eq for Priority {}
impl PartialOrd for Priority {
    fn partial_cmp(&self, b: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(b))
    }
}
impl Ord for Priority {
    fn cmp(&self, b: &Self) -> std::cmp::Ordering {
        self.error.total_cmp(&b.error).then(self.tile.cmp(&b.tile))
    }
}
struct Resource {
    coordinates: Option<Arc<Vec<crate::meshes::SplitPosition>>>,
    keys: Vec<Key>,
    normal_levels: Vec<u8>,
    sample_hits: usize,
    asset: Asset,
    normals: Arc<Vec<[f32; 3]>>,
    cast_error: f64,
    bounds: [DVec3; 8],
    radial_upper: f64,
    error: f64,
}
struct Group {
    members: Vec<Arc<Resource>>,
    envelope: crate::atmosphere::transport::Envelope,
    material: crate::materials::Material,
}
pub(crate) struct Atlas {
    last_update_ms: f64,
    reused_samples: u64,
    groups: Vec<Group>,
    source: Source,
    scene_air: crate::atmosphere::Atmosphere,
    light: crate::model::Light,
    cheap_materials: usize,
    group_builds: u64,
    coordinates: Vec<Option<Arc<Vec<crate::meshes::SplitPosition>>>>,
    pub settings: Settings,
    pub meshes: crate::meshes::Document,
    pub normals: Vec<Arc<Vec<[f32; 3]>>>,
    pub placements: Vec<Mat4>,
    pub view: View,
    pub anchor: Anchor,
    resources: BTreeMap<ResourceKey, Arc<Resource>>,
    pub maximum_error: f64,
    pub limited: bool,
    pub builds: u64,
    pub plans: u64,
    pub retired: u64,
    pub max_level: u8,
    body: Arc<Resource>,
    half: f64,
    body_color: [f32; 3],
    show_body: bool,
    planned_anchor: Anchor,
    planned_eye: DVec3,
    planned_settings: Settings,
}
impl Atlas {
    pub fn new(
        config: &super::runtime::Config,
        settings: Settings,
        anchor: Anchor,
        actor: ActorView,
        light: &crate::model::Light,
        fraction: f64,
    ) -> Result<Self> {
        let ActorView { size, color, eye } = actor;
        let scene_air = light.atmosphere.unwrap_or(crate::atmosphere::Atmosphere {
            density: 0.,
            ..Default::default()
        });
        if !config.material.opaque_occluder() {
            return Err(invalid(
                "Planet global coverage requires opaque terrain; transparent terrain is not a closed world surface",
            ));
        }
        let doc = &config.document;
        settings.validate(doc)?;
        let effective = u32::from(config.level) + config.subdivisions.ilog2();
        let grid = u32::from(settings.tile_subdivisions).min(1 << effective.min(6));
        let mut settings = settings;
        settings.tile_subdivisions = grid as u16;
        let source = Source::new(doc, grid)?;
        let half = f64::from(size[1]) * 0.5;
        let h = source.upper;
        let outer = (doc.radius() + h)
            .max(doc.radius() + anchor.pose.altitude_meters + f64::from(size[1]) + 10.);
        let view = View::new(doc, anchor, &settings, eye, outer, fraction, &source)?;
        let body = Arc::new(topology::body(doc, anchor, size, color)?);
        let mut a = Self {
            last_update_ms: 0.,
            reused_samples: 0,
            source,
            groups: vec![],
            scene_air,
            light: light.clone(),
            cheap_materials: 0,
            group_builds: 0,
            coordinates: vec![],
            planned_settings: settings.clone(),
            planned_anchor: anchor,
            planned_eye: observer(anchor, &view),
            settings,
            meshes: Default::default(),
            normals: vec![],
            placements: vec![],
            view,
            anchor,
            resources: Default::default(),
            maximum_error: 0.,
            limited: false,
            builds: 0,
            plans: 0,
            retired: 0,
            max_level: (u32::from(config.level) + config.subdivisions.ilog2())
                .saturating_sub(grid.ilog2()) as u8,
            body,
            half,
            body_color: color,
            show_body: false,
        };
        a.update(doc, config, anchor, eye, fraction)?;
        Ok(a)
    }
    pub fn update(
        &mut self,
        doc: &Document,
        config: &super::runtime::Config,
        anchor: Anchor,
        eye: f64,
        fraction: f64,
    ) -> Result<()> {
        let started = std::time::Instant::now();
        let result = self.update_inner(doc, config, anchor, eye, fraction);
        if result.is_ok() {
            self.last_update_ms = started.elapsed().as_secs_f64() * 1000.;
        }
        result
    }
    fn update_inner(
        &mut self,
        doc: &Document,
        config: &super::runtime::Config,
        anchor: Anchor,
        eye: f64,
        fraction: f64,
    ) -> Result<()> {
        let h = self.source.upper;
        let outer = (doc.radius() + h)
            .max(doc.radius() + anchor.pose.altitude_meters + self.half * 2. + 10.);
        let view = View::new(
            doc,
            anchor,
            &self.settings,
            eye,
            outer,
            fraction,
            &self.source,
        )?;
        let show_body = (view.eye - DVec3::Y * eye).length() > 0.65;
        // Ordinary motion/looking changes placement only; amortize the expensive
        // source bake. Still recompute the exposed error bound for the current eye.
        let movement = anchor
            .origin_in(self.planned_anchor)
            .length()
            .max((observer(anchor, &view) - self.planned_eye).length());
        let mut threshold_settings = self.settings.clone();
        threshold_settings.distance_meters = self.planned_settings.distance_meters;
        threshold_settings.yaw_degrees = self.planned_settings.yaw_degrees;
        threshold_settings.tilt_degrees = self.planned_settings.tilt_degrees;
        let yaw_delta = ((self.settings.yaw_degrees - self.planned_settings.yaw_degrees + 180.)
            .rem_euclid(360.)
            - 180.)
            .abs();
        let angles = yaw_delta
            .max((self.settings.tilt_degrees - self.planned_settings.tilt_degrees).abs())
            .to_radians();
        let zoom = (self.settings.distance_meters - self.planned_settings.distance_meters).abs();
        if !self.resources.is_empty()
            && threshold_settings == self.planned_settings
            && movement < 32f64.max(self.settings.distance_meters * 0.01)
            && angles < 0.005
            && zoom < 0.005 * (1. + self.settings.distance_meters)
            && show_body == self.show_body
        {
            self.maximum_error = self
                .resources
                .iter()
                .filter(|(k, _)| !hidden(k.tile, &view))
                .map(|(_, r)| projected_bounds(&r.bounds, r.error, anchor, &view, &self.settings))
                .fold(0., f64::max);
            self.limited = self.maximum_error > self.settings.max_error_pixels;
            self.update_materials(config, anchor, &view);
            self.update_placements(anchor, &view);
            self.view = view;
            self.anchor = anchor;
            return Ok(());
        }
        let mut leaves: BTreeSet<_> = FACES
            .into_iter()
            .map(|face| Tile {
                face,
                level: 0,
                xy: [0, 0],
            })
            .collect();
        let mut queue = BinaryHeap::new();
        for &tile in &leaves {
            queue.push(Priority {
                tile,
                error: projected(doc, tile, self.max_level, anchor, &view, &self.settings),
            });
        }
        while let Some(Priority { tile, error }) = queue.pop() {
            if error <= self.settings.max_error_pixels * 0.8
                || leaves.len() + 3 > self.tile_budget()
            {
                break;
            }
            if tile.level >= self.max_level {
                continue;
            }
            leaves.remove(&tile);
            for tile in tile.children() {
                leaves.insert(tile);
                queue.push(Priority {
                    tile,
                    error: projected(doc, tile, self.max_level, anchor, &view, &self.settings),
                });
            }
        }
        let maximum = leaves
            .iter()
            .map(|t| projected(doc, *t, self.max_level, anchor, &view, &self.settings))
            .fold(0., f64::max);
        let registry = topology::Registry::new(&leaves, self.max_level, self.source.grid);
        let mut staged = BTreeMap::new();
        let mut born = 0;
        let mut count = 0;
        let mut reused_samples = 0;
        let previous: BTreeMap<_, _> = self.resources.iter().map(|(k, r)| (k.tile, r)).collect();
        for tile in leaves {
            let key = ResourceKey {
                tile,
                grid: self.source.grid,
                signature: registry.signature(tile),
            };
            let resource = if let Some(r) = self.resources.get(&key) {
                r.clone()
            } else {
                born += 1;
                let resource = topology::resource_reusing(
                    doc,
                    config,
                    &key,
                    &registry,
                    self.max_level,
                    self.settings
                        .reuse_vertex_samples
                        .then(|| previous.get(&tile).map(|r| r.as_ref()))
                        .flatten(),
                )?;
                reused_samples += resource.sample_hits as u64;
                Arc::new(resource)
            };
            count += resource.asset.geometry.positions.len();
            if count > 2_000_000 {
                return Err(invalid(
                    "Planet visual atlas exceeds2M vertices; previous coverage is retained",
                ));
            }
            staged.insert(key, resource);
        }
        let changed = staged.len() != self.resources.len()
            || staged
                .keys()
                .zip(self.resources.keys())
                .any(|(a, b)| a != b)
            || show_body != self.show_body
            || self.planned_settings.render_group_size != self.settings.render_group_size;
        let retired = self
            .resources
            .keys()
            .filter(|k| !staged.contains_key(*k))
            .count() as u64;
        if changed {
            let grouped = topology::groups(
                &staged,
                self.settings.render_group_size,
                &self.body,
                self,
                anchor,
                &view,
            )?;
            self.group_builds += grouped.built;
            self.meshes.assets = grouped.assets;
            self.coordinates = grouped.coordinates;
            self.normals = grouped.normals;
            self.groups = grouped.groups;
            self.meshes.instances = self
                .meshes
                .assets
                .iter()
                .filter(|a| a.id != "orbit_avatar" || show_body)
                .map(|a| Instance {
                    id: a.id.clone(),
                    asset: a.id.clone(),
                    position: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                    scale: 1.,
                    color: if a.id == "orbit_avatar" {
                        self.body_color()
                    } else {
                        config.color
                    },
                    playback: None,
                    attachment: None,
                })
                .collect();
        }
        // A render-only matrix bypasses no authoring validation: authored instances stay identity.
        self.show_body = show_body;
        self.planned_anchor = anchor;
        self.planned_eye = observer(anchor, &view);
        self.planned_settings = self.settings.clone();
        self.show_body = show_body;
        self.resources = staged;
        self.reused_samples += reused_samples;
        self.view = view;
        self.anchor = anchor;
        self.maximum_error = self
            .resources
            .iter()
            .filter(|(k, _)| !hidden(k.tile, &self.view))
            .map(|(_, r)| projected_bounds(&r.bounds, r.error, anchor, &self.view, &self.settings))
            .fold(maximum, f64::max);
        self.limited = self.maximum_error > self.settings.max_error_pixels;
        self.plans += 1;
        self.builds += born;
        self.retired += retired;
        self.update_materials(config, anchor, &self.view.clone());
        self.update_placements(anchor, &self.view.clone());
        Ok(())
    }
    fn update_placements(&mut self, _anchor: Anchor, view: &View) {
        self.placements.clear();
        self.placements.resize(self.groups.len(), Mat4::IDENTITY);
        if self.show_body {
            let mut body = Mat4::from_mat3(view.basis.transpose().as_mat3());
            body.w_axis = (view.basis.transpose() * (DVec3::Y * self.half - view.eye))
                .as_vec3()
                .extend(1.);
            self.placements.push(body);
        }
    }
    pub fn set_light(&mut self, light: &crate::model::Light) {
        self.scene_air = light.atmosphere.unwrap_or(crate::atmosphere::Atmosphere {
            density: 0.,
            ..Default::default()
        });
        self.light = light.clone();
    }
    fn update_materials(&mut self, config: &super::runtime::Config, anchor: Anchor, view: &View) {
        self.cheap_materials = 0;
        let air = self.scene_air;
        let permitted = self.settings.atmosphere_optimization
            && air.reduce_details
            && air.color_error > 0.
            && air.layers.is_none()
            && self.light.shadow.is_none()
            && self.light.environment.is_none_or(|e| e.intensity == 0.)
            && config.material.surface.is_none()
            && config.material.transmission == 0.
            && config
                .material
                .reflectance
                .is_none_or(|r| r.f0 == 0. && r.metallic == 0.);
        let r = config.document.radius();
        let eye = observer(anchor, view);
        let shift = crate::atmosphere::transport::correction_limit(r, view.altitude);
        // Bounds cover source vertices. Include common-frame f32 reconstruction;
        // the shader itself caps any optical-root displacement by `shift`.
        let guard = 256.
            * f64::from(f32::EPSILON)
            * (r + eye.length() + self.settings.atmosphere_height_meters);
        let upper =
            crate::distant::selection::bound(&config.material, glam::Vec3::from(config.color));
        let illumination =
            self.light.ambient + (1. - self.light.ambient) * self.light.direct_intensity;
        let contrast = f64::from(upper.max_element().max(1.) * illumination.max(1.) + 0.1) * 1.001;
        let medium = crate::atmosphere::transport::Medium::Sphere {
            radius: r + self.settings.atmosphere_height_meters,
        };
        let density = self.optical_depth() / self.settings.atmosphere_height_meters;
        for (group, asset) in self.groups.iter().zip(&mut self.meshes.assets) {
            let masked = permitted
                && density > 0.
                && 2.
                    * contrast
                    * medium.transmission_upper(density, eye, group.envelope, shift, guard)
                    <= f64::from(air.color_error) * 0.1;
            let material = if masked {
                crate::materials::Material::default()
            } else {
                group.material.clone()
            };
            if asset.material != material {
                asset.material = material;
            }
            self.cheap_materials += usize::from(masked);
        }
    }
    fn tile_budget(&self) -> usize {
        let n = self.source.grid as usize;
        // Interiors plus a global bound on added border points/fan centres.
        self.settings
            .max_tiles
            .min(1_999_000 / ((n + 1) * (n + 1) + 16 * n))
    }
    fn optical_depth(&self) -> f64 {
        self.settings
            .atmosphere_optical_depth
            .unwrap_or(f64::from(self.scene_air.density) * self.settings.atmosphere_height_meters)
    }
    fn body_color(&self) -> [f32; 3] {
        self.body_color
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"active":true,"session_only":true,"settings":self.settings,"eye_position":self.view.position,"camera_altitude_meters":self.view.altitude,
            "planet_center_view_meters":self.view.center.to_array(),"render_origin":"camera","render_units":"meters","clip":self.view.clip,"resident_tiles":self.resources.len(),"vertices":self.resources.values().map(|r|r.asset.geometry.positions.len()).sum::<usize>(),
            "triangles":self.resources.values().map(|r|r.asset.geometry.triangles.len()).sum::<usize>(),"geometry_builds":self.builds,"residence_plans":self.plans,"retired_tiles":self.retired,
            "last_update_ms":self.last_update_ms,"reused_vertex_samples":self.reused_samples,
            "max_source_cast_error_meters":self.resources.values().map(|r|r.cast_error).fold(0.,f64::max),"maximum_geometric_bound_pixels":self.maximum_error,"quality_limited":self.limited,
            "all_six_faces_covered":true,"conforming_edges":true,"shared_high_low_coordinates":true,"depth_contract":"reverse_z_finite","physical_grid_level":self.max_level+self.source.grid.ilog2() as u8,
            "atmosphere_model":"homogeneous_finite_shell","effective_density_per_meter":self.optical_depth()/self.settings.atmosphere_height_meters,"effective_optical_depth":self.optical_depth(),"effective_tile_budget":self.tile_budget(),"finite_atmospheric_materials":self.cheap_materials,"render_groups":self.groups.len(),"group_builds":self.group_builds,"legacy_atmospheric_selector":false,"gpu_occlusion":"fallback_custom_projection"})
    }
    pub fn apply(&self, frame: &mut crate::render::RenderFrame) {
        if self.settings.canonical_cpu_stream {
            frame
                .meshes
                .update_planet_canonical(&self.meshes, &self.normals, &self.coordinates);
        } else {
            frame.meshes.update(&self.meshes, 0);
        }
        frame
            .meshes
            .apply_smooth_normals(&self.normals)
            .expect("validated global visual normals");
        frame
            .meshes
            .apply_planet_coordinates(&self.coordinates)
            .expect("validated shared planetary coordinates");
        frame
            .meshes
            .apply_render_placements(&self.placements)
            .expect("validated camera-relative placements");
        frame.camera = crate::model::Camera {
            position: [0.; 3],
            target: [0., 0., -1.],
            fov_degrees: self.settings.fov_degrees as f32,
        };
        frame.clip = Some(self.view.clip);
        let radial = self.view.eye_radial.as_vec3().to_array();
        frame.planet_air = Some(crate::render::PlanetAir {
            radius: self.anchor.radius,
            altitude: self.view.altitude,
            radial,
            height: self.settings.atmosphere_height_meters,
            optical_depth: self.optical_depth(),
            space_color: self.settings.space_color,
            terrain_materials: self.groups.len() as u32,
            eye: crate::meshes::SplitPosition::new(
                self.anchor.basis
                    * (self.view.eye / self.anchor.radius
                        + DVec3::Y * (1. + self.anchor.pose.altitude_meters / self.anchor.radius)),
            ),
            axes: (self.view.basis.transpose() * self.anchor.basis.transpose())
                .as_mat3()
                .to_cols_array_2d(),
        });
        frame.light.direction = (self.view.basis.transpose()
            * DVec3::from(frame.light.direction.map(f64::from)))
        .as_vec3()
        .to_array();
        if let Some(a) = frame.light.atmosphere.as_mut() {
            a.reduce_details = false;
        }
        frame.distant_meshes = Default::default();
        frame.mesh_lod_settings.enabled = false;
        frame.truncated_navigation_paths = 0;
        frame.boxes.clear();
        frame.vision_guides.clear();
        frame.hearing_guides.clear();
        frame.noise_guides.clear();
        frame.route_guides.clear();
        frame.selection = None;
        frame.selection_mesh = None;
    }
}

fn observer(anchor: Anchor, view: &View) -> DVec3 {
    anchor.basis * (view.eye + DVec3::Y * (anchor.radius + anchor.pose.altitude_meters))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod precision_tests;
