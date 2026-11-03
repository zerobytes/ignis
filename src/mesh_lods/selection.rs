//! Optional geometry-only selection. Surface correspondence is not an image proof.
use super::{Document, Pack};
use crate::{distant::selection::Draw, render::RenderFrame};
use glam::Vec3;
use std::sync::Arc;
#[derive(Default)]
pub(crate) struct Selection {
    document: Option<Arc<Document>>,
    source: Option<Arc<crate::meshes::Geometry>>,
    packs: Vec<Option<Arc<Pack>>>,
    bounds: Vec<(Vec3, Vec3)>,
    pub stats: Stats,
    view: View,
}
#[derive(Default, Clone, Copy)]
struct View {
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    focal: f32,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Stats {
    pub enabled: bool,
    pub selected_instances: usize,
    pub source_triangles: usize,
    pub selected_triangles: usize,
    pub max_projected_error_px: f32,
    pub max_normal_color_bound: f32,
    pub compiled_bytes: usize,
}
impl Selection {
    pub fn update(&mut self, frame: &RenderFrame, height: u32) -> bool {
        self.stats = Stats {
            enabled: frame.mesh_lod_settings.enabled,
            compiled_bytes: if frame.mesh_lods.packs.is_empty() {
                0
            } else {
                frame.mesh_lods.byte_len()
            },
            ..Default::default()
        };
        if !frame.mesh_lod_settings.enabled || frame.mesh_lods.packs.is_empty() {
            return false;
        }
        let eye = Vec3::from(frame.camera.position);
        let forward = (Vec3::from(frame.camera.target) - eye).normalize();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        self.view = View {
            eye,
            forward,
            right,
            up: right.cross(forward),
            focal: height as f32 / (2. * (frame.camera.fov_degrees.to_radians() * 0.5).tan()),
        };
        // A source mesh generation includes transforms; immutable asset identity is
        // checked only once per asset, never by hashing each instance or pixel.
        let changed = self
            .document
            .as_ref()
            .is_none_or(|d| !Arc::ptr_eq(d, &frame.mesh_lods))
            || self.packs.len() != frame.meshes.source.assets.len()
            || self
                .packs
                .iter()
                .zip(&frame.meshes.source.assets)
                .any(|(p, a)| p.as_ref().is_some_and(|p| !p.matches(a)))
            || self
                .source
                .as_ref()
                .zip(frame.meshes.source.assets.first())
                .is_some_and(|(g, a)| !Arc::ptr_eq(g, &a.geometry));
        if changed {
            self.document = Some(frame.mesh_lods.clone());
            self.source = frame
                .meshes
                .source
                .assets
                .first()
                .map(|a| a.geometry.clone());
            self.packs = frame
                .meshes
                .source
                .assets
                .iter()
                .map(|a| frame.mesh_lods.packs.iter().find(|p| p.matches(a)).cloned())
                .collect();
            self.bounds = frame
                .meshes
                .source
                .assets
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    if self.packs[i].is_some() {
                        crate::distant::bounds(a.geometry.positions.iter().copied())
                    } else {
                        (Vec3::ZERO, Vec3::ZERO)
                    }
                })
                .collect();
        }
        changed
    }
    pub fn choose(&mut self, frame: &RenderFrame, draw: &Draw) -> Option<usize> {
        let settings = frame.mesh_lod_settings;
        if !settings.enabled
            || frame.mesh_lods.packs.is_empty()
            || !draw.shared
            || !draw.eligible
            || frame.selection_mesh.as_ref() == Some(&draw.instance)
            || frame.light.shadow.is_some()
            || frame.light.environment.is_some_and(|e| e.intensity > 0.)
            || frame
                .light
                .atmosphere
                .is_some_and(|a| a.edge_softness.is_some_and(|s| s.active()))
        {
            return None;
        }
        let pack = self.packs[draw.asset].as_ref()?;
        let (lo, hi) = self.bounds[draw.asset];
        let View {
            eye,
            forward,
            right,
            up,
            focal,
        } = self.view;
        let center = draw.world.transform_point3((lo + hi) * 0.5);
        let half = (hi - lo) * 0.5;
        let scale = draw.world.x_axis.truncate().length();
        let extent = |axis: Vec3| {
            draw.world.x_axis.truncate().dot(axis).abs() * half.x
                + draw.world.y_axis.truncate().dot(axis).abs() * half.y
                + draw.world.z_axis.truncate().dot(axis).abs() * half.z
        };
        let radius = half.length() * scale;
        let pad = (center.distance(eye) + radius).max(1.) * 0.00001;
        let z = (center - eye).dot(forward) - extent(forward) - pad;
        // Diagnostic floor has a discontinuous world-space grid. Never simplify a
        // source intersecting it; its appearance has no smooth normal-only bound.
        if center.y - extent(Vec3::Y) <= 0.02 && center.y + extent(Vec3::Y) >= -0.02 {
            return None;
        }
        let transverse = ((center - eye).dot(right).abs() + extent(right) + pad)
            .hypot((center - eye).dot(up).abs() + extent(up) + pad);
        let upper =
            crate::distant::selection::bound(&pack.material, Vec3::from(draw.color)).max_element();
        let light = (1. - frame.light.ambient) * frame.light.direct_intensity;
        let shade_max = upper * (frame.light.ambient + light);
        let mut best = None;
        let mut selected_error = 0.;
        let mut selected_color = 0.;
        for (i, l) in pack.levels.iter().enumerate() {
            let delta = l.error * scale;
            if z <= delta + 0.05 {
                continue;
            }
            let projected = focal / (z - delta) * delta * (1. + transverse / z);
            let mut normal_color = upper * light * l.normal_error;
            if let Some(air) = frame.light.atmosphere {
                let local_eye = draw.world.inverse().transform_point3(eye);
                let near =
                    ((local_eye - local_eye.clamp(lo, hi)).length() * scale - pad - delta).max(0.);
                let t = (-air.density * near).exp();
                let remain = air.layers.map_or(1., |l| l.remaining(near));
                let base = air.layers.map_or(Vec3::ZERO, |l| l.mountain_color.into());
                let maximum = shade_max.max(base.max_element()).max(1.);
                // Corresponding points are at most delta apart, so their radial
                // distances differ by at most delta, not the whole asset's depth.
                // Smoothstep is1.5-Lipschitz over its normalized interval.
                // Directional sun color uses an8-Lipschitz lobe.
                normal_color = normal_color * t * remain
                    + t * -(-air.density * delta).exp_m1() * maximum
                    + air.layers.map_or(0., |l| {
                        if near > l.detail_end {
                            0.
                        } else {
                            (1.5 * l.influence * delta / (l.detail_end - l.detail_start))
                                .min(l.influence)
                                * t
                                * shade_max.max(base.max_element())
                        }
                    })
                    + (8. * air.sun_strength * frame.light.direct_intensity * 2. * delta
                        / near.max(0.00001))
                    .min(1.);
            }
            if projected <= settings.error_px * 0.8
                && normal_color <= settings.color_error * 0.8
                && best
                    .is_none_or(|old: usize| l.triangles.len() < pack.levels[old].triangles.len())
            {
                best = Some(i);
                selected_error = projected;
                selected_color = normal_color;
            }
        }
        if let Some(i) = best {
            self.stats.selected_instances += 1;
            self.stats.source_triangles += pack.geometry.triangles.len();
            self.stats.selected_triangles += pack.levels[i].triangles.len();
            self.stats.max_projected_error_px =
                self.stats.max_projected_error_px.max(selected_error);
            self.stats.max_normal_color_bound =
                self.stats.max_normal_color_bound.max(selected_color);
        }
        best
    }
}
