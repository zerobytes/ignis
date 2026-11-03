//! Select BEFORE uploads/draws; cache plans so a stationary source is not re-expanded.
use super::{Level, Pack, bounds};
use crate::{
    materials::Material,
    meshes::Geometry,
    render::{RenderFrame, Vertex},
};
use glam::{Mat4, Vec3};
use std::{ops::Range, sync::Arc};
#[derive(Clone)]
pub(crate) struct Draw {
    pub range: Range<usize>,
    pub instance: String,
    pub asset: usize,
    pub world: Mat4,
    pub color: [f32; 3],
    pub eligible: bool,
    pub shared: bool,
    /// Whether the asset is actually repeated, rather than resident only for size.
    pub repeated: bool,
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CheapVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
struct Cached {
    geometry: Arc<Geometry>,
    material: Material,
    data: Option<Arc<Vec<u8>>>,
    levels: Vec<Level>,
    valid: bool,
    upper: Vec3,
    lo: Vec3,
    hi: Vec3,
}
#[derive(Clone, PartialEq)]
pub(crate) struct Choice {
    pub visible: u8,
    pub level: Option<usize>,
    pub color: Option<[f32; 3]>,
    pub geometric: Option<usize>,
}
#[derive(Default)]
pub(crate) struct Selection {
    pub precise_projection: bool,
    pub lods: crate::mesh_lods::selection::Selection,
    cache: Vec<Cached>,
    pub plan: Vec<Choice>,
    scratch: Vec<Choice>,
    pub draw_ranges: Vec<(bool, Range<u32>)>,
    pub tail: Range<u32>,
    tail_visibility: u8,
    generation: Option<u64>,
    pub active: bool,
    pub full: Vec<Vertex>,
    pub cheap: Vec<CheapVertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<Material>,
    pub stats: Stats,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Stats {
    pub source_triangles: usize,
    pub detailed_triangles: usize,
    pub cheap_triangles: usize,
    pub substituted_instances: usize,
    pub geometric_lod_instances: usize,
    pub material_slots_used: usize,
    pub selected_vertex_bytes: usize,
    pub allocated_vertex_bytes: usize,
    pub selected_index_bytes: usize,
    pub allocated_index_bytes: usize,
    pub selected_geometry_bytes: usize,
    pub allocated_geometry_bytes: usize,
    pub compiled_bytes: usize,
    pub max_projected_error_px: f32,
    pub max_color_error_bound: f32,
}
impl Selection {
    pub fn level(&self, asset: usize, level: usize) -> &Level {
        &self.cache[asset].levels[level]
    }
    fn passthrough_stats(&mut self, frame: &RenderFrame) {
        self.stats = Stats {
            source_triangles: frame.meshes.triangle_count(),
            detailed_triangles: frame.meshes.vertices.len() / 3,
            selected_vertex_bytes: frame.meshes.vertices.len() * 64,
            selected_geometry_bytes: frame.meshes.vertices.len() * 64,
            material_slots_used: frame.meshes.materials.len(),
            compiled_bytes: frame.distant_meshes.iter().map(|p| p.data.len()).sum(),
            ..Default::default()
        };
    }
    pub fn update(
        &mut self,
        frame: &RenderFrame,
        height: u32,
        visibility: &crate::render_visibility::Visibility,
    ) {
        let lod_changed = self.lods.update(frame, height);
        let enabled = frame.light.atmosphere.is_some_and(|a| {
            a.reduce_details && (a.density > 0. || a.layers.is_some()) && a.color_error > 0.
        }) && !frame.distant_meshes.is_empty()
            && frame.light.shadow.is_none()
            && frame.light.environment.is_none_or(|e| e.intensity == 0.);
        let source = &frame.meshes.source;
        let mut cache_changed = lod_changed || self.cache.len() != source.assets.len();
        self.cache.truncate(source.assets.len());
        for (i, a) in source.assets.iter().enumerate() {
            let pack = frame.distant_meshes.iter().find(|p| p.asset == a.id);
            let data = pack.map(|p| p.data.clone());
            let same = self.cache.get(i).is_some_and(|c| {
                Arc::ptr_eq(&c.geometry, &a.geometry)
                    && c.material == a.material
                    && match (&c.data, &data) {
                        (None, None) => true,
                        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                        _ => false,
                    }
            });
            if same {
                continue;
            }
            cache_changed = true;
            let valid = pack.is_some_and(|p| p.matches(a));
            let levels = if valid {
                pack.and_then(|p| p.levels().ok()).unwrap_or_default()
            } else {
                Vec::new()
            };
            let (lo, hi) = bounds(a.geometry.positions.iter().copied());
            let c = Cached {
                geometry: a.geometry.clone(),
                material: a.material.clone(),
                data,
                levels,
                valid,
                upper: if valid {
                    pack.unwrap().upper()
                } else {
                    Vec3::ZERO
                },
                lo,
                hi,
            };
            if i < self.cache.len() {
                self.cache[i] = c;
            } else {
                self.cache.push(c);
            }
        }
        let air = frame.light.atmosphere.filter(|a| {
            enabled
                && a.reduce_details
                && (a.density > 0. || a.layers.is_some())
                && a.color_error > 0.
        });
        let eligible_light = frame.light.shadow.is_none()
            && frame.light.environment.is_none_or(|e| e.intensity == 0.);
        let eye = Vec3::from(frame.camera.position);
        let forward = (Vec3::from(frame.camera.target) - eye).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward);
        let focal = height as f32 / (2. * (frame.camera.fov_degrees.to_radians() * 0.5).tan());
        let mut plan = std::mem::take(&mut self.scratch);
        plan.clear();
        let mut max_error = 0f32;
        let mut max_color = 0f32;
        for (n, draw) in frame.meshes.draws.iter().enumerate() {
            let c = &self.cache[draw.asset];
            let mut choice = Choice {
                visible: visibility.masks[n],
                level: None,
                color: None,
                geometric: None,
            };
            if let Some(air) = air.filter(|_| {
                choice.visible & crate::render_visibility::CAMERA != 0
                    && eligible_light
                    && draw.eligible
                    && c.valid
                    && frame.selection_mesh.as_ref() != Some(&draw.instance)
            }) {
                let a = &source.assets[draw.asset];
                if a.material
                    .reflectance
                    .is_none_or(|r| r.f0 == 0. && r.metallic == 0.)
                {
                    let center = draw.world.transform_point3((c.lo + c.hi) * 0.5);
                    let scale = draw.world.x_axis.truncate().length();
                    let radius = (c.hi - c.lo).length() * 0.5 * scale;
                    let distance = (center - eye).length();
                    // The source is inside this oriented box. A wide mountain must not
                    // pretend its side-to-side radius extends toward the camera.
                    let local_eye = draw.world.inverse().transform_point3(eye);
                    let padding = (distance + radius).max(1.) * 0.00001;
                    let nearest = ((local_eye - local_eye.clamp(c.lo, c.hi)).length() * scale
                        - padding)
                        .max(0.);
                    let farthest = (local_eye - c.lo)
                        .abs()
                        .max((local_eye - c.hi).abs())
                        .length()
                        * scale
                        + padding;
                    let half = (c.hi - c.lo) * 0.5;
                    let depth_extent = draw.world.x_axis.truncate().dot(forward).abs() * half.x
                        + draw.world.y_axis.truncate().dot(forward).abs() * half.y
                        + draw.world.z_axis.truncate().dot(forward).abs() * half.z;
                    // Perspective division amplifies depth error by x/z and y/z,
                    // not by the full camera distance. Bound the entire oriented
                    // source box in the camera's transverse plane.
                    let transverse_bound = if !self.precise_projection {
                        distance + radius
                    } else {
                        let extent = |axis: Vec3| {
                            (center - eye).dot(axis).abs()
                                + draw.world.x_axis.truncate().dot(axis).abs() * half.x
                                + draw.world.y_axis.truncate().dot(axis).abs() * half.y
                                + draw.world.z_axis.truncate().dot(axis).abs() * half.z
                                + padding
                        };
                        extent(right).hypot(extent(up)).min(distance + radius)
                    };
                    let transmission = (-air.density * nearest).exp();
                    // Source diffuse lighting is nonnegative and bounded by tint * gain;
                    // transmission/back-light <=1 cannot increase its maximum above this.
                    let upper = c.upper
                        * Vec3::from(draw.color)
                        * (frame.light.ambient
                            + (1. - frame.light.ambient) * frame.light.direct_intensity);
                    // Include the renderer's diagnostic floor grid upper contribution.
                    let upper = upper + Vec3::new(0.055, 0.075, 0.08) * 0.65;
                    let remaining = air.layers.map_or(1., |l| l.remaining(nearest));
                    let error = transmission * remaining * upper.max_element() * 0.5;
                    if error <= air.color_error * 0.8 {
                        choice.color = Some((upper * 0.5).to_array());
                        let mut selected_color_error = error;
                        let mut selected_projected_error = 0.;
                        let z = (center - eye).dot(forward) - depth_extent - padding;
                        // Screen-space feathering classifies depth discontinuities. A
                        // displaced proxy can change that classification even when its
                        // projected position error is small. Keep the original depth
                        // surface; the bounded material substitution remains valid.
                        let levels = if air.edge_softness.is_some_and(|s| s.active()) {
                            &[][..]
                        } else {
                            &c.levels[..]
                        };
                        for (i, l) in levels.iter().enumerate() {
                            let delta = l.error * scale;
                            let (transverse, depth_error) = if l.vertical {
                                let world_y = draw.world.y_axis.truncate().normalize();
                                (
                                    delta
                                        * (world_y.dot(right).powi(2) + world_y.dot(up).powi(2))
                                            .sqrt(),
                                    delta * world_y.dot(forward).abs(),
                                )
                            } else {
                                (delta, delta)
                            };
                            if z <= depth_error + 0.05 {
                                continue;
                            }
                            // Bound perspective division over the entire source box.
                            let projected = focal / (z - depth_error)
                                * (transverse + depth_error * transverse_bound / z);
                            let midpoint = upper * 0.5;
                            let base = air.layers.map_or(midpoint, |l| l.mountain_color.into());
                            let air_difference = [midpoint, base]
                                .into_iter()
                                .map(|c| {
                                    (c - Vec3::from(air.color))
                                        .abs()
                                        .max((c - Vec3::from(air.sun_color)).abs())
                                        .max_element()
                                })
                                .fold(0., f32::max);
                            let shifted_error = if l.error == 0. {
                                error
                            } else {
                                error
                                    + (transmission - (-air.density * farthest).exp())
                                        * air_difference
                                    + air.layers.map_or(0., |l| {
                                        (l.remaining(nearest) - l.remaining(farthest))
                                            * (midpoint - Vec3::from(l.mountain_color))
                                                .abs()
                                                .max_element()
                                    })
                            };
                            // Reserve20% for roundoff/transition headroom. No unattenuated silhouette shortcut.
                            if shifted_error.is_finite()
                                && shifted_error <= air.color_error * 0.8
                                && projected <= air.silhouette_error_px * 0.8
                                && choice.level.is_none_or(|old| {
                                    l.triangles.len() < c.levels[old].triangles.len()
                                })
                            {
                                choice.level = Some(i);
                                selected_color_error = shifted_error;
                                selected_projected_error = projected;
                            }
                        }
                        max_color = max_color.max(selected_color_error);
                        max_error = max_error.max(selected_projected_error);
                    }
                }
            }
            if choice.color.is_none() && choice.visible & crate::render_visibility::CAMERA != 0 {
                choice.geometric = self.lods.choose(frame, draw);
            }
            plan.push(choice);
        }
        self.stats.max_projected_error_px = max_error;
        self.stats.max_color_error_bound = max_color;
        if !cache_changed
            && self.generation == Some(frame.meshes.generation)
            && self.plan == plan
            && self.tail_visibility == visibility.tail
        {
            self.scratch = plan;
            return;
        }
        self.active = plan
            .iter()
            .any(|c| c.color.is_some() || c.geometric.is_some() || c.visible == 0)
            || visibility.tail == 0;
        self.tail_visibility = visibility.tail;
        self.generation = Some(frame.meshes.generation);
        self.scratch = std::mem::replace(&mut self.plan, plan);
        self.draw_ranges.clear();
        self.tail = 0..0;
        self.full.clear();
        self.cheap.clear();
        self.indices.clear();
        if !self.active {
            self.passthrough_stats(frame);
            return;
        }
        let mut substituted = 0;
        let mut reduced = 0;
        let mut end = 0;
        for (draw, choice) in frame.meshes.draws.iter().zip(&self.plan) {
            if draw.shared {
                self.draw_ranges.push((false, 0..0));
                continue;
            }
            end = draw.range.end;
            if choice.visible == 0 {
                self.draw_ranges.push((false, 0..0));
                continue;
            }
            let cheap = choice.color.is_some();
            let start = if cheap {
                self.indices.len()
            } else {
                self.full.len()
            };
            if let Some(color) = choice.color {
                substituted += 1;
                let cached = &self.cache[draw.asset];
                // Reuse authored/compiled indices; no runtime deduplication or hashing.
                let base = self.cheap.len() as u32;
                let positions = if let Some(level) = choice.level {
                    reduced += 1;
                    let l = &cached.levels[level];
                    self.indices
                        .extend(l.triangles.iter().flatten().map(|&i| base + u32::from(i)));
                    &l.positions
                } else {
                    self.indices.extend(
                        cached
                            .geometry
                            .triangles
                            .iter()
                            .flatten()
                            .map(|&i| base + i),
                    );
                    &cached.geometry.positions
                };
                self.cheap.extend(positions.iter().map(|p| CheapVertex {
                    position: draw.world.transform_point3((*p).into()).to_array(),
                    color,
                }));
            } else {
                self.full
                    .extend_from_slice(&frame.meshes.vertices[draw.range.clone()]);
            }
            let stop = if cheap {
                self.indices.len()
            } else {
                self.full.len()
            };
            self.draw_ranges.push((cheap, start as u32..stop as u32));
        }
        // Animated derived avatars are never substituted by this static compiler.
        let start = self.full.len();
        if visibility.tail != 0 {
            self.full.extend_from_slice(&frame.meshes.vertices[end..]);
        }
        self.tail = start as u32..self.full.len() as u32;
        let mut used = vec![false; frame.meshes.materials.len()];
        if let Some(first) = used.first_mut() {
            *first = true;
        }
        for (draw, choice) in frame.meshes.draws.iter().zip(&self.plan) {
            if draw.shared && choice.visible != 0 && choice.color.is_none() {
                used[draw.asset + 1] = true;
            }
        }
        for v in &self.full {
            used[v.material as usize] = true;
        }
        self.materials.clear();
        self.materials
            .extend(frame.meshes.materials.iter().enumerate().map(|(i, m)| {
                if used[i] {
                    m.clone()
                } else {
                    Material::default()
                }
            }));
        self.stats.source_triangles = frame.meshes.triangle_count();
        self.stats.detailed_triangles = self.full.len() / 3;
        self.stats.cheap_triangles = self.indices.len() / 3;
        self.stats.substituted_instances = substituted;
        self.stats.geometric_lod_instances = reduced;
        self.stats.material_slots_used = used.into_iter().filter(|v| *v).count();
        self.stats.selected_vertex_bytes = self.full.len() * 64 + self.cheap.len() * 24;
        self.stats.selected_index_bytes = self.indices.len() * 4;
        self.stats.selected_geometry_bytes =
            self.stats.selected_vertex_bytes + self.stats.selected_index_bytes;
        self.stats.compiled_bytes = frame
            .distant_meshes
            .iter()
            .map(|p: &Pack| p.data.len())
            .sum();
    }
}
pub(crate) fn bound(m: &Material, color: Vec3) -> Vec3 {
    let mut upper = color;
    for p in [m.pattern, m.secondary_pattern].into_iter().flatten() {
        upper *= Vec3::from(p.low).max(Vec3::from(p.high));
    }
    if let Some(d) = &m.detail {
        upper *= Vec3::ONE.max(Vec3::ONE.lerp(d.tint.into(), d.opacity));
    }
    upper
}
