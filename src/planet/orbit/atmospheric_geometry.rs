//! View-dependent interior derivatives; the physical/source atlas is immutable.
//! Each reduced disk keeps its complete perimeter and proves equal ray coverage.
use super::*;
use crate::atmosphere::transport::{Envelope, Medium, correction_limit};
const SIDE: usize = 4;
struct Patch {
    source_triangles: Vec<u32>,
    fan: Vec<[u32; 3]>,
    fan_built: bool,
    center: Key,
    bounds: [DVec3; 8],
    eye: DVec3,
    radius: f64,
    facing: bool,
}
struct Candidate {
    source: Arc<Resource>,
    patches: Vec<Patch>,
    owners: Vec<u8>,
    mask: u16,
    reduced: Option<Arc<Resource>>,
    rejected_eye: Option<DVec3>,
}
#[derive(Default)]
pub(super) struct Cache {
    entries: BTreeMap<ResourceKey, Candidate>,
    builds: u64,
    tests: u64,
}
impl Cache {
    pub fn prepare(
        &mut self,
        resource: (&ResourceKey, &Arc<Resource>),
        config: &super::super::runtime::Config,
        settings: &Settings,
        light: &crate::model::Light,
        upper: f64,
        camera: (Anchor, &View),
    ) {
        let _ = self.select(
            std::iter::once(resource),
            config,
            settings,
            light,
            upper,
            camera,
        );
    }
    pub fn invalidate_rejections(&mut self) {
        for c in self.entries.values_mut() {
            c.rejected_eye = None;
        }
    }
    pub fn retain(&mut self, resources: &BTreeMap<ResourceKey, Arc<Resource>>) {
        self.entries
            .retain(|k, c| resources.get(k).is_some_and(|r| Arc::ptr_eq(r, &c.source)));
    }
    pub fn retain_pair(
        &mut self,
        current: &BTreeMap<ResourceKey, Arc<Resource>>,
        pending: &BTreeMap<ResourceKey, Arc<Resource>>,
    ) {
        self.entries.retain(|k, c| {
            [current, pending]
                .iter()
                .any(|m| m.get(k).is_some_and(|r| Arc::ptr_eq(r, &c.source)))
        });
    }
    pub fn select<'a>(
        &mut self,
        resources: impl Iterator<Item = (&'a ResourceKey, &'a Arc<Resource>)>,
        config: &super::super::runtime::Config,
        settings: &Settings,
        light: &crate::model::Light,
        upper: f64,
        camera: (Anchor, &View),
    ) -> Vec<(&'a ResourceKey, Arc<Resource>)> {
        let (anchor, view) = camera;
        let mut selected: Vec<_> = resources.map(|(k, r)| (k, r.clone())).collect();
        let Some(air) = light.atmosphere else {
            return selected;
        };
        if !settings.atmosphere_geometry
            || !settings.atmosphere_optimization
            || !air.reduce_details
            || air.color_error <= 0.
            || air.layers.is_some()
            || air.edge_softness.is_some()
            || light.shadow.is_some()
            || light.environment.is_some_and(|e| e.intensity != 0.)
            || config.material.surface.is_some()
            || config.material.detail.is_some()
            || config.material.procedural.is_some()
            || config.material.transmission != 0.
            || config
                .material
                .reflectance
                .is_some_and(|r| r.f0 != 0. || r.metallic != 0.)
        {
            return selected;
        }
        let r = config.document.radius();
        let eye = observer(anchor, view);
        let shift = correction_limit(r, view.altitude);
        let guard =
            256. * f64::from(f32::EPSILON) * (r + eye.length() + settings.atmosphere_height_meters);
        let density = settings
            .atmosphere_optical_depth
            .unwrap_or(f64::from(air.density) * settings.atmosphere_height_meters)
            / settings.atmosphere_height_meters;
        if density <= 0. {
            return selected;
        }
        let medium = Medium::Sphere {
            radius: r + settings.atmosphere_height_meters,
        };
        let surface =
            crate::distant::selection::bound(&config.material, glam::Vec3::from(config.color));
        let illumination = light.ambient + (1. - light.ambient) * light.direct_intensity;
        let contrast =
            f64::from(surface.max_element().max(1.) * illumination.max(1.) + 0.1) * 1.001;
        let tan = (settings.fov_degrees.to_radians() * 0.5).tan();
        let aspect = f64::from(settings.viewport[0]) / f64::from(settings.viewport[1]);
        let corner_cos = 1. / (1. + tan * tan * (1. + aspect * aspect)).sqrt();
        for (key, selected_source) in &mut selected {
            let source = &*selected_source;
            // Do not clone the large seam signature on every warm lookup.
            if !self.entries.contains_key(*key) {
                self.entries.insert((*key).clone(), partition(source));
            }
            let candidate = self.entries.get_mut(*key).unwrap();
            if !Arc::ptr_eq(source, &candidate.source) {
                *candidate = partition(source);
            }
            // Keeping the original source is always safe. Rejected candidates
            // retry after motion, or immediately after an explicit setting/light
            // change; looking alone need not repeat thousands of failed proofs.
            if candidate
                .rejected_eye
                .is_some_and(|old| (eye / r - old).length() * r < 32.)
            {
                continue;
            }
            let mut mask = 0;
            for (i, patch) in candidate.patches.iter_mut().enumerate() {
                if patch.source_triangles.is_empty() {
                    continue;
                }
                // Use the GLOBAL radial upper bound: different terrain may be
                // exposed behind this disk when its interior depth changes.
                let envelope = Envelope {
                    corners: patch.bounds.map(|p| p * r),
                    radius_upper: r + upper,
                };
                // Ray coverage also needs to survive clipping. Within the
                // viewport, depth >= distance*corner_cos; keep both disks well
                // beyond near. The global clip already encloses every far point.
                let nearest = (eye - eye.clamp(envelope.corners[0], envelope.corners[7])).length();
                if nearest * corner_cos <= f64::from(view.clip.near) * 2. {
                    continue;
                }
                if 2. * contrast * medium.transmission_upper(density, eye, envelope, shift, guard)
                    > f64::from(air.color_error) * 0.05
                {
                    continue;
                }
                if (eye / r - patch.eye).length() >= patch.radius {
                    self.tests += 1;
                    let triangles = patch
                        .source_triangles
                        .iter()
                        .map(|&i| source.asset.geometry.triangles[i as usize]);
                    let (facing, radius) = clearance(source, triangles, eye / r);
                    patch.eye = eye / r;
                    patch.radius = radius;
                    patch.facing = facing;
                }
                if patch.facing && !patch.fan_built {
                    patch.fan_built = true;
                    prepare_fan(source, patch);
                }
                if patch.facing && !patch.fan.is_empty() && fan_covers_disk(source, patch, eye / r)
                {
                    mask |= 1 << i;
                }
            }
            if mask == 0 {
                candidate.rejected_eye = Some(eye / r);
                continue;
            }
            candidate.rejected_eye = None;
            if mask != candidate.mask || candidate.reduced.is_none() {
                candidate.reduced = Some(Arc::new(reduce(candidate, mask)));
                candidate.mask = mask;
                self.builds += 1;
            }
            *selected_source = candidate.reduced.as_ref().unwrap().clone();
        }
        selected
    }
    pub fn inspect(
        &self,
        source: &BTreeMap<ResourceKey, Arc<Resource>>,
        selected: &BTreeMap<ResourceKey, Arc<Resource>>,
    ) -> serde_json::Value {
        serde_json::json!({"reduced_tiles":selected.iter().filter(|(k,r)|source.get(*k).is_some_and(|s|!Arc::ptr_eq(s,r))).count(),
            "selected_triangles":selected.values().map(|r|r.asset.geometry.triangles.len()).sum::<usize>(),
            "selected_vertices":selected.values().map(|r|r.asset.geometry.positions.len()).sum::<usize>(),
            "derivative_builds":self.builds,"source_projection_tests":self.tests,"cached_tiles":self.entries.len(),
            "boundary":"exact_source_polygon","coverage":"forward_disk_winding_one","physical_source_changed":false})
    }
}
fn point(r: &Resource, i: usize) -> DVec3 {
    let p = r.coordinates.as_ref().unwrap()[i];
    DVec3::from(p.high.map(f64::from)) + DVec3::from(p.low.map(f64::from))
}
fn clearance(r: &Resource, triangles: impl Iterator<Item = [u32; 3]>, eye: DVec3) -> (bool, f64) {
    let mut nearest = f64::INFINITY;
    for t in triangles {
        let [a, b, c] = t.map(|i| point(r, i as usize));
        let n = (b - a).cross(c - a);
        let length = n.length();
        if length <= 1e-30 {
            return (false, 0.);
        }
        let d = n.dot(eye - a) / length;
        let margin = 4096. * f64::EPSILON;
        if d <= margin {
            return (false, (margin - d).max(0.) * 0.5);
        }
        nearest = nearest.min(d - margin);
    }
    (true, nearest * 0.5)
}
fn partition(source: &Arc<Resource>) -> Candidate {
    let lo: Key = source.keys.iter().fold([i64::MAX; 3], |a, b| {
        std::array::from_fn(|i| a[i].min(b[i]))
    });
    let hi: Key = source.keys.iter().fold([i64::MIN; 3], |a, b| {
        std::array::from_fn(|i| a[i].max(b[i]))
    });
    let axes: Vec<_> = (0..3).filter(|&i| lo[i] != hi[i]).collect();
    let mut patches: Vec<_> = (0..SIDE * SIDE)
        .map(|_| Patch {
            source_triangles: vec![],
            fan: vec![],
            fan_built: false,
            center: [0; 3],
            bounds: [DVec3::ZERO; 8],
            eye: DVec3::ZERO,
            radius: -1.,
            facing: false,
        })
        .collect();
    let mut owners = vec![];
    for (index, t) in source.asset.geometry.triangles.iter().enumerate() {
        let cell = |axis| {
            let sum: i64 = t.iter().map(|&i| source.keys[i as usize][axis]).sum();
            (((sum - 3 * lo[axis]) * SIDE as i64 / (3 * (hi[axis] - lo[axis]))) as usize)
                .min(SIDE - 1)
        };
        let owner = cell(axes[0]) + SIDE * cell(axes[1]);
        owners.push(owner as u8);
        patches[owner].source_triangles.push(index as u32);
    }
    for (i, patch) in patches.iter_mut().enumerate() {
        let mut center = lo;
        for (axis, offset) in [(axes[0], i % SIDE), (axes[1], i / SIDE)] {
            center[axis] =
                lo[axis] + (hi[axis] - lo[axis]) * (2 * offset as i64 + 1) / (2 * SIDE as i64);
        }
        patch.center = center;
        let mut min = DVec3::splat(f64::INFINITY);
        let mut max = -min;
        for &i in &patch.source_triangles {
            let t = source.asset.geometry.triangles[i as usize];
            for a in t {
                let p = point(source, a as usize);
                min = min.min(p);
                max = max.max(p);
            }
        }
        patch.bounds = std::array::from_fn(|i| {
            DVec3::new(
                if i & 1 == 0 { min.x } else { max.x },
                if i & 2 == 0 { min.y } else { max.y },
                if i & 4 == 0 { min.z } else { max.z },
            )
        });
    }
    Candidate {
        source: source.clone(),
        patches,
        owners,
        mask: 0,
        reduced: None,
        rejected_eye: None,
    }
}
fn prepare_fan(source: &Resource, patch: &mut Patch) {
    let Some(center) = source
        .keys
        .iter()
        .position(|k| *k == patch.center)
        .map(|i| i as u32)
    else {
        return;
    };
    let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for &index in &patch.source_triangles {
        let t = source.asset.geometry.triangles[index as usize];
        for e in 0..3 {
            let (a, b) = (t[e], t[(e + 1) % 3]);
            if !edges.remove(&(b, a)) {
                edges.insert((a, b));
            }
        }
    }
    let next: BTreeMap<_, _> = edges.iter().copied().collect();
    if next.len() != edges.len() || edges.is_empty() {
        return;
    }
    let start = edges.first().unwrap().0;
    let mut at = start;
    for _ in 0..edges.len() {
        let Some(&b) = next.get(&at) else {
            break;
        };
        patch.fan.push([center, at, b]);
        at = b;
    }
    if at != start
        || patch.fan.len() != edges.len()
        || patch.fan.len() >= patch.source_triangles.len()
    {
        patch.fan.clear();
    }
}
fn fan_covers_disk(r: &Resource, patch: &Patch, eye: DVec3) -> bool {
    if !clearance(r, patch.fan.iter().copied(), eye).0 {
        return false;
    }
    let center = point(r, patch.fan[0][0] as usize);
    // A virtual camera points at the disk: coverage equivalence is in ray space
    // and therefore remains valid when the real camera rotates without moving.
    let forward = (center - eye).normalize();
    let reference = if forward.z.abs() < 0.9 {
        DVec3::Z
    } else {
        DVec3::X
    };
    let right = forward.cross(reference).normalize();
    let up = right.cross(forward);
    if patch
        .bounds
        .iter()
        .any(|p| (*p - eye).dot(forward) <= 1e-12)
    {
        return false;
    }
    let project = |i| {
        let p = point(r, i) - eye;
        glam::DVec2::new(p.dot(right), p.dot(up)) / p.dot(forward)
    };
    let origin = project(patch.fan[0][0] as usize);
    let mut angle = 0.;
    for t in &patch.fan {
        let a = project(t[1] as usize) - origin;
        let b = project(t[2] as usize) - origin;
        angle += a.perp_dot(b).atan2(a.dot(b));
    }
    (angle - std::f64::consts::TAU).abs() < 1e-8
}
fn reduce(candidate: &Candidate, mask: u16) -> Resource {
    let source = &candidate.source;
    let old = &source.asset.geometry;
    let mut emitted = [false; SIDE * SIDE];
    let mut kept = vec![];
    let mut remap = vec![u32::MAX; source.keys.len()];
    let mut triangles = Vec::with_capacity(old.triangles.len());
    let mut emit = |t: [u32; 3]| {
        triangles.push(t.map(|i| {
            let slot = &mut remap[i as usize];
            if *slot == u32::MAX {
                *slot = kept.len() as u32;
                kept.push(i as usize);
            }
            *slot
        }));
    };
    for (i, &owner) in candidate.owners.iter().enumerate() {
        let owner = usize::from(owner);
        if mask & (1 << owner) != 0 {
            if !emitted[owner] {
                for &t in &candidate.patches[owner].fan {
                    emit(t);
                }
                emitted[owner] = true;
            }
        } else {
            emit(old.triangles[i]);
        }
    }
    let geometry = Geometry {
        positions: kept.iter().map(|&i| old.positions[i]).collect(),
        triangles,
        material_coordinates: Some(
            kept.iter()
                .map(|&i| old.material_coordinates.as_ref().unwrap()[i])
                .collect(),
        ),
    };
    let mut asset = source.asset.clone();
    asset.geometry = Arc::new(geometry);
    Resource {
        coordinates: Some(Arc::new(
            kept.iter()
                .map(|&i| source.coordinates.as_ref().unwrap()[i])
                .collect(),
        )),
        keys: kept.iter().map(|&i| source.keys[i]).collect(),
        normal_levels: kept.iter().map(|&i| source.normal_levels[i]).collect(),
        sample_hits: 0,
        asset,
        normals: Arc::new(kept.iter().map(|&i| source.normals[i]).collect()),
        cast_error: source.cast_error,
        coordinate_error: source.coordinate_error,
        bounds: source.bounds,
        radial_upper: source.radial_upper,
        error: source.error,
    }
}
