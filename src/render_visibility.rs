//! Conservative, per-view rejection before mesh uploads and draw submission.
//! Bounds cover the actual pose and every compiled proxy; no asset semantics.
use crate::render::{RenderBox, RenderFrame};
use glam::{Mat4, Vec3};
use std::{ops::Range, sync::Arc};
pub(crate) const CAMERA: u8 = 1;
pub(crate) const SHADOW: u8 = 2;
fn absolute(m: Mat4) -> Mat4 {
    Mat4::from_cols(
        m.x_axis.abs(),
        m.y_axis.abs(),
        m.z_axis.abs(),
        m.w_axis.abs(),
    )
}
#[derive(Clone, Copy)]
struct Bounds {
    lo: Vec3,
    hi: Vec3,
}
impl Bounds {
    fn points(points: impl Iterator<Item = [f32; 3]>) -> Self {
        let (lo, hi) = crate::distant::bounds(points);
        Self { lo, hi }
    }
    fn visible(self, view: Mat4, world: Mat4) -> bool {
        let clip = view * world;
        let magnitude = (absolute(view) * absolute(world)).transpose();
        // WebGPU homogeneous volume: -w<=x,y<=w, 0<=z<=w. Support
        // of an AABB against each plane avoids perspective division/near-plane holes.
        if !self.lo.is_finite() || !self.hi.is_finite() {
            return true;
        }
        let m = clip.transpose();
        let center = (self.lo + self.hi) * 0.5;
        let half = (self.hi - self.lo) * 0.5;
        let planes = [
            m.w_axis + m.x_axis,
            m.w_axis - m.x_axis,
            m.w_axis + m.y_axis,
            m.w_axis - m.y_axis,
            m.z_axis,
            m.w_axis - m.z_axis,
        ];
        let errors = [
            magnitude.w_axis + magnitude.x_axis,
            magnitude.w_axis + magnitude.x_axis,
            magnitude.w_axis + magnitude.y_axis,
            magnitude.w_axis + magnitude.y_axis,
            magnitude.z_axis,
            magnitude.w_axis + magnitude.z_axis,
        ];
        for (p, e) in planes.into_iter().zip(errors) {
            let n = p.truncate();
            let support = n.dot(center) + p.w + n.abs().dot(half);
            // Fail open for invalid math; roundoff margin includes world translation.
            let margin = 64. * f32::EPSILON * (e.truncate().dot(center.abs() + half) + e.w).max(1.);
            if support.is_finite() && support < -margin {
                return false;
            }
        }
        true
    }
}
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum Relation {
    Outside,
    Partial,
    Inside,
}
struct ObjectFrustum {
    planes: [glam::Vec4; 6],
    errors: [glam::Vec4; 6],
}
impl ObjectFrustum {
    fn new(view: Mat4) -> Self {
        let m = view.transpose();
        let e = absolute(view).transpose();
        Self {
            planes: [
                m.w_axis + m.x_axis,
                m.w_axis - m.x_axis,
                m.w_axis + m.y_axis,
                m.w_axis - m.y_axis,
                m.z_axis,
                m.w_axis - m.z_axis,
            ],
            errors: [
                e.w_axis + e.x_axis,
                e.w_axis + e.x_axis,
                e.w_axis + e.y_axis,
                e.w_axis + e.y_axis,
                e.z_axis,
                e.w_axis + e.z_axis,
            ],
        }
    }
    fn relation(&self, b: Bounds) -> Relation {
        if !b.lo.is_finite() || !b.hi.is_finite() {
            return Relation::Partial;
        }
        let c = (b.lo + b.hi) * 0.5;
        let half = (b.hi - b.lo) * 0.5;
        let mut result = Relation::Inside;
        for (p, e) in self.planes.iter().zip(&self.errors) {
            let n = p.truncate();
            let center = n.dot(c) + p.w;
            let radius = n.abs().dot(half);
            let margin = 64. * f32::EPSILON * (e.truncate().dot(c.abs() + half) + e.w).max(1.);
            if (center + radius).is_finite() && center + radius < -margin {
                return Relation::Outside;
            }
            if !(center - radius).is_finite() || center - radius <= margin {
                result = Relation::Partial;
            }
        }
        result
    }
}
impl Bounds {
    fn world(self, m: Mat4) -> Option<Self> {
        let c = (self.lo + self.hi) * 0.5;
        let half = (self.hi - self.lo) * 0.5;
        let center = m.transform_point3(c);
        let radius = m.x_axis.truncate().abs() * half.x
            + m.y_axis.truncate().abs() * half.y
            + m.z_axis.truncate().abs() * half.z;
        // Enclose both bound construction and the shader's world transform,
        // including cancellation of large local coordinates and translation.
        let error = (absolute(m) * (c.abs() + half).extend(1.))
            .truncate()
            .max(Vec3::ONE)
            * (64. * f32::EPSILON);
        let b = Self {
            lo: center - radius - error,
            hi: center + radius + error,
        };
        (b.lo.is_finite() && b.hi.is_finite()).then_some(b)
    }
}
#[derive(Clone, Copy)]
struct Rect {
    lo: glam::Vec2,
    hi: glam::Vec2,
    nearest: f32,
}
impl Rect {
    fn project(b: Bounds, view: Mat4, world: Mat4, padding: glam::Vec2) -> Self {
        let clip = view * world;
        let magnitude = absolute(view) * absolute(world);
        let full = Self {
            lo: glam::Vec2::splat(f32::NEG_INFINITY),
            hi: glam::Vec2::splat(f32::INFINITY),
            nearest: 1.,
        };
        let mut rect = Self {
            lo: glam::Vec2::splat(f32::INFINITY),
            hi: glam::Vec2::splat(f32::NEG_INFINITY),
            nearest: 0.,
        };
        for x in [b.lo.x, b.hi.x] {
            for y in [b.lo.y, b.hi.y] {
                for z in [b.lo.z, b.hi.z] {
                    let p = clip * Vec3::new(x, y, z).extend(1.);
                    if !p.is_finite() || p.w <= 0.0001 || p.z < 0. {
                        return full;
                    }
                    let error =
                        (magnitude * Vec3::new(x, y, z).abs().extend(1.)) * (64. * f32::EPSILON);
                    if p.w <= error.w || p.z <= error.z {
                        return full;
                    }
                    rect.nearest = rect.nearest.max(0.05001 / (p.w - error.w));
                    for sign in [-1., 1.] {
                        let denominator = p.w + sign * error.w;
                        rect.lo = rect
                            .lo
                            .min(glam::Vec2::new(p.x - error.x, p.y - error.y) / denominator);
                        rect.hi = rect
                            .hi
                            .max(glam::Vec2::new(p.x + error.x, p.y + error.y) / denominator);
                    }
                }
            }
        }
        rect.lo -= padding;
        rect.hi += padding;
        rect
    }
    fn separate(self, other: Self) -> bool {
        self.hi.x < other.lo.x
            || other.hi.x < self.lo.x
            || self.hi.y < other.lo.y
            || other.hi.y < self.lo.y
    }
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Stats {
    pub enabled: bool,
    pub object_groups: usize,
    pub object_camera_active: bool,
    pub object_shadow_active: bool,
    pub object_camera_tests_saved: usize,
    pub object_shadow_tests_saved: usize,
    pub object_cache_bytes: usize,
    pub mesh_instances: usize,
    pub camera_mesh_instances: usize,
    pub shadow_mesh_instances: usize,
    pub rejected_mesh_instances: usize,
    pub rejected_source_triangles: usize,
    pub boxes: usize,
    pub camera_boxes: usize,
    pub shadow_boxes: usize,
    pub rejected_boxes: usize,
}
#[derive(Default)]
pub(crate) struct Visibility {
    pub objects_disabled: bool,
    object_generation: Option<u64>,
    object_bounds: Vec<Bounds>,
    object_sizes: Vec<usize>,
    object_membership: Vec<u32>,
    object_relations: Vec<[Relation; 2]>,
    viewport: glam::Vec2,
    generation: Option<u64>,
    local_geometry_generation: Option<u64>,
    packs: Option<Arc<Vec<crate::distant::Pack>>>,
    bounds: Vec<Bounds>,
    tail_bounds: Option<Bounds>,
    pub masks: Vec<u8>,
    camera_rects: Vec<Rect>,
    shadow_rects: Vec<Rect>,
    pub tail: u8,
    pub boxes: Vec<RenderBox>,
    pub box_camera: Vec<Range<u32>>,
    pub box_shadow: Vec<Range<u32>>,
    pub stats: Stats,
}
fn append_range(ranges: &mut Vec<Range<u32>>, range: Range<u32>) {
    if let Some(last) = ranges.last_mut()
        && last.end == range.start
    {
        last.end = range.end;
    } else {
        ranges.push(range);
    }
}
impl Visibility {
    fn prepare_objects(&mut self, frame: &RenderFrame) {
        self.object_bounds.clear();
        self.object_sizes.clear();
        self.object_membership.clear();
        if frame.meshes.source.objects.is_empty() {
            return;
        }
        self.object_membership
            .resize(frame.meshes.draws.len(), u32::MAX);
        let lookup: std::collections::HashMap<_, _> = frame
            .meshes
            .draws
            .iter()
            .enumerate()
            .map(|(n, d)| (d.instance.as_str(), n))
            .collect();
        for object in frame.meshes.source.objects.iter() {
            if object.members.len() < 2
                || !object.members.iter().all(|m| {
                    lookup
                        .get(m.as_str())
                        .is_some_and(|&n| frame.meshes.draws[n].eligible)
                })
            {
                continue;
            }
            let mut bound: Option<Bounds> = None;
            let mut valid = true;
            for member in &object.members {
                let n = lookup[member.as_str()];
                let draw = &frame.meshes.draws[n];
                let b = if draw.shared {
                    self.bounds[n].world(draw.world)
                } else {
                    Some(self.bounds[n])
                };
                let Some(b) = b else {
                    valid = false;
                    break;
                };
                bound = Some(bound.map_or(b, |old| Bounds {
                    lo: old.lo.min(b.lo),
                    hi: old.hi.max(b.hi),
                }));
            }
            if !valid {
                continue;
            }
            let Ok(index) = u32::try_from(self.object_bounds.len()) else {
                continue;
            };
            if index == u32::MAX {
                continue;
            }
            if let Some(b) = bound {
                self.object_bounds.push(b);
                self.object_sizes.push(object.members.len());
                for m in &object.members {
                    self.object_membership[lookup[m.as_str()]] = index;
                }
            }
        }
    }
    fn use_objects(&self, view: &ObjectFrustum) -> bool {
        // Cost prediction only, never a visibility decision. Probe across the
        // authored order before paying for every group. Partial groups still
        // require the old per-part test, so decline this path unless enough
        // part tests disappear to cover group queries plus a safety margin.
        // The probe bounds work, not the number of objects a scene can contain.
        let stride = self.object_bounds.len().div_ceil(64).max(1);
        let mut groups = 0usize;
        let mut saved = 0usize;
        for n in (0..self.object_bounds.len()).step_by(stride) {
            groups += 1;
            if view.relation(self.object_bounds[n]) != Relation::Partial {
                saved += self.object_sizes[n];
            }
        }
        saved as f64 > groups as f64 * 1.25
    }
    pub fn occlusion_bound(&self, n: usize) -> crate::render_occlusion::Bound {
        let r = self.camera_rects[n];
        let lo = glam::Vec2::new(r.lo.x * 0.5 + 0.5, 0.5 - r.hi.y * 0.5) * self.viewport;
        let hi = glam::Vec2::new(r.hi.x * 0.5 + 0.5, 0.5 - r.lo.y * 0.5) * self.viewport;
        crate::render_occlusion::Bound {
            rect: [lo.x.floor(), lo.y.floor(), hi.x.ceil(), hi.y.ceil()],
            nearest: r.nearest,
            pad: [0.; 3],
        }
    }
    /// Conservative longest projected AABB edge; remove only the explicit
    /// grouping guard, retaining projection roundoff. Near-plane crossings are large.
    pub fn size_bucket(&self, n: usize) -> usize {
        if !self.stats.enabled {
            return 5;
        }
        let r = self.camera_rects[n];
        let extent = ((r.hi - r.lo) * self.viewport * 0.5 - glam::Vec2::splat(4.)).max_element();
        [1., 2., 4., 16., 64.]
            .iter()
            .position(|&bound| extent <= bound)
            .unwrap_or(5)
    }
    /// Reordering is safe only when neither view has shared raster coverage.
    pub fn independent(&self, a: usize, b: usize) -> bool {
        let common = self.masks[a] & self.masks[b];
        (common & CAMERA == 0 || self.camera_rects[a].separate(self.camera_rects[b]))
            && (common & SHADOW == 0 || self.shadow_rects[a].separate(self.shadow_rects[b]))
    }

    pub fn update(
        &mut self,
        frame: &RenderFrame,
        vp: Mat4,
        width: u32,
        height: u32,
        enabled: bool,
    ) {
        self.viewport = glam::Vec2::new(width as f32, height as f32);
        let packs_changed = self
            .packs
            .as_ref()
            .is_none_or(|p| !Arc::ptr_eq(p, &frame.distant_meshes));
        let changed = self.generation != Some(frame.meshes.generation)
            || frame.meshes.generation == 0
            || self
                .packs
                .as_ref()
                .is_none_or(|p| !Arc::ptr_eq(p, &frame.distant_meshes));
        if changed {
            let reuse_locals = self.local_geometry_generation
                == Some(frame.meshes.geometry_generation)
                && !packs_changed
                && frame.meshes.vertices.is_empty()
                && self.bounds.len() == frame.meshes.draws.len()
                && frame.meshes.draws.iter().all(|d| d.shared);
            if !reuse_locals {
                self.bounds.clear();
                let locals: Vec<_> = frame
                    .meshes
                    .source
                    .assets
                    .iter()
                    .map(|a| {
                        let mut b = Bounds::points(a.geometry.positions.iter().copied());
                        if let Some(pack) = frame.distant_meshes.iter().find(|p| p.asset == a.id)
                            && pack.matches(a)
                            && let Ok(levels) = pack.levels()
                        {
                            for l in levels {
                                let q = Bounds::points(l.positions.into_iter());
                                b.lo = b.lo.min(q.lo);
                                b.hi = b.hi.max(q.hi);
                            }
                        }
                        b
                    })
                    .collect();
                let mut end = 0;
                for draw in &frame.meshes.draws {
                    self.bounds.push(if draw.shared {
                        locals[draw.asset]
                    } else {
                        end = draw.range.end;
                        let mut b = Bounds::points(
                            frame.meshes.vertices[draw.range.clone()]
                                .iter()
                                .map(|v| v.position),
                        );
                        if draw.eligible {
                            let local = locals[draw.asset];
                            for x in [local.lo.x, local.hi.x] {
                                for y in [local.lo.y, local.hi.y] {
                                    for z in [local.lo.z, local.hi.z] {
                                        let p = draw.world.transform_point3(Vec3::new(x, y, z));
                                        b.lo = b.lo.min(p);
                                        b.hi = b.hi.max(p);
                                    }
                                }
                            }
                        }
                        b
                    });
                }
                self.tail_bounds = (end < frame.meshes.vertices.len()).then(|| {
                    Bounds::points(frame.meshes.vertices[end..].iter().map(|v| v.position))
                });
                self.local_geometry_generation = Some(frame.meshes.geometry_generation);
            }
            if self.object_generation != Some(frame.meshes.static_generation)
                || frame.meshes.static_generation == 0
                || packs_changed
            {
                self.prepare_objects(frame);
                self.object_generation = Some(frame.meshes.static_generation);
            }
            self.generation = Some(frame.meshes.generation);
            self.packs = Some(frame.distant_meshes.clone());
        }
        let shadow = crate::render::shadow_view_projection(&frame.light);
        let classify = |b: Bounds, world: Mat4| {
            let camera = !enabled || b.visible(vp, world);
            let light = shadow.is_some_and(|s| !enabled || b.visible(s, world));
            (u8::from(camera) * CAMERA) | (u8::from(light) * SHADOW)
        };
        self.stats = Stats {
            enabled,
            mesh_instances: frame.meshes.draws.len(),
            boxes: frame.boxes.len(),
            ..Default::default()
        };
        self.object_relations.clear();
        if enabled && !self.objects_disabled {
            let camera = ObjectFrustum::new(vp);
            let light = shadow.map(ObjectFrustum::new);
            let camera_active = self.use_objects(&camera);
            let shadow_active = light.as_ref().is_some_and(|f| self.use_objects(f));
            if camera_active || shadow_active {
                self.object_relations
                    .extend(self.object_bounds.iter().map(|&b| {
                        [
                            if camera_active {
                                camera.relation(b)
                            } else {
                                Relation::Partial
                            },
                            if shadow_active {
                                light.as_ref().unwrap().relation(b)
                            } else {
                                Relation::Partial
                            },
                        ]
                    }));
            }
            self.stats.object_groups = self.object_bounds.len();
            self.stats.object_camera_active = camera_active;
            self.stats.object_shadow_active = shadow_active;
        }
        self.stats.object_cache_bytes = self.object_bounds.capacity()
            * std::mem::size_of::<Bounds>()
            + self.object_sizes.capacity() * std::mem::size_of::<usize>()
            + self.object_membership.capacity() * 4
            + self.object_relations.capacity() * 2;
        self.masks.clear();
        self.camera_rects.clear();
        self.shadow_rects.clear();
        for (n, (draw, b)) in frame.meshes.draws.iter().zip(&self.bounds).enumerate() {
            let world = if draw.shared {
                draw.world
            } else {
                Mat4::IDENTITY
            };
            let relation = self
                .object_membership
                .get(n)
                .and_then(|&g| self.object_relations.get(g as usize));
            let camera = if let Some(r) = relation
                && r[0] != Relation::Partial
            {
                self.stats.object_camera_tests_saved += 1;
                r[0] == Relation::Inside
            } else {
                !enabled || b.visible(vp, world)
            };
            let light = shadow.is_some_and(|view| {
                if let Some(r) = relation
                    && r[1] != Relation::Partial
                {
                    self.stats.object_shadow_tests_saved += 1;
                    r[1] == Relation::Inside
                } else {
                    !enabled || b.visible(view, world)
                }
            });
            let mask = (u8::from(camera) * CAMERA) | (u8::from(light) * SHADOW);
            self.masks.push(mask);
            let world = if draw.shared {
                draw.world
            } else {
                Mat4::IDENTITY
            };
            self.camera_rects.push(if enabled && mask & CAMERA != 0 {
                Rect::project(
                    *b,
                    vp,
                    world,
                    glam::Vec2::new(4. / width as f32, 4. / height as f32),
                )
            } else {
                Rect {
                    lo: glam::Vec2::ZERO,
                    hi: glam::Vec2::ZERO,
                    nearest: 1.,
                }
            });
            self.shadow_rects.push(if enabled && mask & SHADOW != 0 {
                Rect::project(
                    *b,
                    shadow.unwrap(),
                    world,
                    glam::Vec2::splat(4. / frame.light.shadow.as_ref().unwrap().resolution as f32),
                )
            } else {
                Rect {
                    lo: glam::Vec2::ZERO,
                    hi: glam::Vec2::ZERO,
                    nearest: 1.,
                }
            });

            self.stats.camera_mesh_instances += usize::from(mask & CAMERA != 0);
            self.stats.shadow_mesh_instances += usize::from(mask & SHADOW != 0);
            if mask == 0 {
                self.stats.rejected_mesh_instances += 1;
                self.stats.rejected_source_triangles += draw.range.len() / 3;
            }
        }
        self.tail = self.tail_bounds.map_or(0, |b| classify(b, Mat4::IDENTITY));
        self.boxes.clear();
        self.box_camera.clear();
        self.box_shadow.clear();
        for item in &frame.boxes {
            let center = Vec3::from(item.position);
            let half = Vec3::from(item.size) * 0.5;
            let mask = classify(
                Bounds {
                    lo: center - half,
                    hi: center + half,
                },
                Mat4::IDENTITY,
            );
            if mask == 0 {
                self.stats.rejected_boxes += 1;
                continue;
            }
            let start = self.boxes.len() as u32 * 36;
            self.boxes.push(*item);
            if mask & CAMERA != 0 {
                self.stats.camera_boxes += 1;
                append_range(&mut self.box_camera, start..start + 36);
            }
            if mask & SHADOW != 0 {
                self.stats.shadow_boxes += 1;
                append_range(&mut self.box_shadow, start..start + 36);
            }
        }
    }
}
