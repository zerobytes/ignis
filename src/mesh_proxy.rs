//! Opt-in mesh-reduction laboratory. Material/physics/source remain canonical.
//! Meshoptimizer's error is an ESTIMATE; this is not an IGL surface/image certificate.
use crate::{
    meshes::{self, Asset, Geometry, Shading},
    render::RenderFrame,
};
use anyhow::{Context, Result, ensure};
use glam::{Mat4, Quat, Vec3};
use std::{collections::BTreeSet, sync::Arc};
const MAGIC: &[u8; 8] = b"IGPRX1\0\0";
mod sanitize;
const MAX_BYTES: usize = 256 * 1024 * 1024;
pub struct Level {
    pub error: f32,
    pub vertices: Vec<u32>,
    pub triangles: Vec<[u32; 3]>,
}
pub struct Entry {
    pub asset: String,
    pub hash: [u8; 32],
    pub levels: Vec<Level>,
}
pub struct Pack {
    pub entries: Vec<Entry>,
}
fn eligible(a: &Asset) -> bool {
    a.rig.is_none()
        && a.clips.is_empty()
        && a.shading == Shading::Smooth
        && a.geometry.material_coordinates.is_none()
        && a.material.detail.is_none()
        && a.material.procedural.is_none()
        && a.material.surface.is_none()
        && a.material.transmission == 0.
}
fn digest(a: &Asset) -> [u8; 32] {
    crate::distant::digest(a)
}
fn normals(a: &Asset) -> Vec<[f32; 3]> {
    let mut n = vec![Vec3::ZERO; a.geometry.positions.len()];
    crate::mesh_normals::accumulate(&a.geometry.triangles, &a.geometry.positions, &mut n);
    n.into_iter()
        .map(|n| {
            if n == Vec3::ZERO {
                Vec3::Y.to_array()
            } else {
                n.to_array()
            }
        })
        .collect()
}
fn components(g: &Geometry) -> (Vec<u32>, BTreeSet<u32>) {
    let p = sanitize::labels(g);
    let used = g.triangles.iter().map(|t| p[t[0] as usize]).collect();
    (p, used)
}
pub fn compile(doc: &meshes::Document) -> Result<Pack> {
    doc.validate().map_err(|e| anyhow::anyhow!(e.message))?;
    let mut entries = vec![];
    for a in &doc.assets {
        if !eligible(a) || a.geometry.triangles.len() < 64 {
            continue;
        }
        let g = &a.geometry;
        ensure!(
            g.positions.len() < u32::MAX as usize,
            "Proxy source exceeds u32"
        );
        let n = normals(a);
        let flat: Vec<_> = n.iter().flatten().copied().collect();
        let indices: Vec<_> = g.triangles.iter().flatten().copied().collect();
        let adapter = meshopt::VertexDataAdapter::new(bytemuck::cast_slice(&g.positions), 12, 0)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let (labels, source_parts) = components(g);
        let mut levels = vec![];
        for (ratio, error) in [
            (0.25, 0.003),
            (0.0625, 0.01),
            (0.015625, 0.03),
            (0.00390625, 0.1),
        ] {
            let mut estimated = 0.;
            let mut reduced = meshopt::simplify_with_attributes_and_locks(
                &indices,
                &adapter,
                &flat,
                &[0.025; 3],
                12,
                &vec![false; g.positions.len()],
                ((indices.len() as f64 * ratio) as usize / 3 * 3).max(3),
                error,
                meshopt::SimplifyOptions::ErrorAbsolute,
                Some(&mut estimated),
            );
            if reduced.is_empty()
                || reduced.len() * 10 >= indices.len() * 9
                || !estimated.is_finite()
            {
                continue;
            }
            reduced = reduced
                .chunks_exact(3)
                .filter(|t| {
                    let [a, b, c] = [t[0], t[1], t[2]].map(|i| Vec3::from(g.positions[i as usize]));
                    (b - a).cross(c - a).length_squared() > 0.
                })
                .flatten()
                .copied()
                .collect();
            let retained: BTreeSet<_> = reduced
                .chunks_exact(3)
                .map(|t| labels[t[0] as usize])
                .collect();
            // No component may disappear or join another. This does not certify
            // projected alpha/normal fidelity; image validation remains mandatory.
            if reduced.chunks_exact(3).any(|t| {
                t.iter()
                    .any(|i| labels[*i as usize] != labels[t[0] as usize])
            }) {
                continue;
            }
            if retained != source_parts {
                for t in &g.triangles {
                    if !retained.contains(&labels[t[0] as usize]) {
                        reduced.extend(t);
                    }
                }
            }
            if reduced.len() * 10 >= indices.len() * 9 {
                continue;
            }
            let mut remap = vec![u32::MAX; g.positions.len()];
            let mut vertices = vec![];
            let mut triangles = reduced
                .chunks_exact(3)
                .map(|t| {
                    std::array::from_fn(|j| {
                        let i = t[j];
                        if remap[i as usize] == u32::MAX {
                            remap[i as usize] = vertices.len() as u32;
                            vertices.push(i);
                        }
                        remap[i as usize]
                    })
                })
                .collect::<Vec<_>>();
            sanitize::repair(g, &labels, &mut vertices, &mut triangles)?;
            if levels
                .iter()
                .any(|l: &Level| l.triangles.len() <= triangles.len() && l.error <= estimated)
            {
                continue;
            }
            eprintln!(
                "proxy {}: {} -> {} tri, {} components, estimated {}m",
                a.id,
                g.triangles.len(),
                triangles.len(),
                source_parts.len(),
                estimated
            );
            levels.push(Level {
                error: estimated.max(0.000001),
                vertices,
                triangles,
            });
        }
        if !levels.is_empty() {
            entries.push(Entry {
                asset: a.id.clone(),
                hash: digest(a),
                levels,
            });
        }
    }
    Ok(Pack { entries })
}
impl Pack {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut b = MAGIC.to_vec();
        let word = |b: &mut Vec<u8>, v: u32| b.extend(v.to_le_bytes());
        word(&mut b, self.entries.len() as u32);
        for e in &self.entries {
            word(&mut b, e.asset.len() as u32);
            b.extend(e.asset.as_bytes());
            b.extend(e.hash);
            word(&mut b, e.levels.len() as u32);
            for l in &e.levels {
                word(&mut b, l.error.to_bits());
                word(&mut b, l.vertices.len() as u32);
                word(&mut b, l.triangles.len() as u32);
                for i in &l.vertices {
                    word(&mut b, *i);
                }
                for t in &l.triangles {
                    for i in t {
                        word(&mut b, *i);
                    }
                }
            }
        }
        ensure!(
            b.len() + 32 <= MAX_BYTES,
            "Proxy packet exceeds diagnostic resource budget"
        );
        let hash = ring::digest::digest(&ring::digest::SHA256, &b);
        b.extend(hash.as_ref());
        Ok(b)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        ensure!(
            (44..=MAX_BYTES).contains(&bytes.len()) && &bytes[..8] == MAGIC,
            "Invalid IGPRX1 size/header"
        );
        let (payload, seal) = bytes.split_at(bytes.len() - 32);
        ensure!(
            ring::digest::digest(&ring::digest::SHA256, payload).as_ref() == seal,
            "Proxy checksum mismatch"
        );
        struct Reader<'a> {
            b: &'a [u8],
            at: usize,
        }
        impl Reader<'_> {
            fn take(&mut self, n: usize) -> Result<&[u8]> {
                let end = self.at.checked_add(n).context("Proxy size overflow")?;
                let out = self.b.get(self.at..end).context("Truncated proxy")?;
                self.at = end;
                Ok(out)
            }
            fn word(&mut self) -> Result<u32> {
                Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
            }
        }
        let mut r = Reader { b: payload, at: 8 };
        let count = r.word()? as usize;
        ensure!(count <= payload.len() / 44, "Invalid proxy entry count");
        let mut entries = vec![];
        let mut names = BTreeSet::new();
        for _ in 0..count {
            let n = r.word()? as usize;
            ensure!((1..=80).contains(&n), "Invalid proxy name");
            let asset = std::str::from_utf8(r.take(n)?)?.to_owned();
            ensure!(names.insert(asset.clone()), "Duplicate proxy asset");
            let hash = r.take(32)?.try_into().unwrap();
            let count = r.word()?;
            ensure!((1..=4).contains(&count), "Invalid proxy level count");
            let mut levels = vec![];
            for _ in 0..count {
                let error = f32::from_bits(r.word()?);
                let nv = r.word()? as usize;
                let nt = r.word()? as usize;
                ensure!(
                    error.is_finite()
                        && error >= 0.
                        && nv >= 3
                        && nt > 0
                        && nv
                            .checked_mul(4)
                            .and_then(|n| nt.checked_mul(12).and_then(|t| n.checked_add(t)))
                            .is_some_and(|n| n <= payload.len() - r.at),
                    "Invalid proxy level bounds"
                );
                let mut vertices = Vec::with_capacity(nv);
                for _ in 0..nv {
                    vertices.push(r.word()?);
                }
                let mut triangles = Vec::with_capacity(nt);
                for _ in 0..nt {
                    let t = [r.word()?, r.word()?, r.word()?];
                    ensure!(t.iter().all(|v| (*v as usize) < nv), "Invalid proxy index");
                    triangles.push(t);
                }
                levels.push(Level {
                    error,
                    vertices,
                    triangles,
                });
            }
            entries.push(Entry {
                asset,
                hash,
                levels,
            });
        }
        ensure!(r.at == payload.len(), "Trailing proxy payload");
        Ok(Self { entries })
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"format":"IGPRX1","error_is_estimated_not_certified":true,"entries":self.entries.iter().map(|e|serde_json::json!({"asset":e.asset,"levels":e.levels.iter().map(|l|serde_json::json!({"triangles":l.triangles.len(),"vertices":l.vertices.len(),"estimated_error_meters":l.error})).collect::<Vec<_>>()})).collect::<Vec<_>>()})
    }
}

type ResolvedLevel = (Asset, Arc<Vec<[f32; 3]>>, f32);
struct Ready {
    source: Asset,
    levels: Vec<ResolvedLevel>,
    lo: Vec3,
    hi: Vec3,
}
fn in_view(vp: Mat4, center: Vec3, radius: f32) -> bool {
    let m = vp.transpose();
    [
        m.w_axis + m.x_axis,
        m.w_axis - m.x_axis,
        m.w_axis + m.y_axis,
        m.w_axis - m.y_axis,
        m.z_axis,
        m.w_axis - m.z_axis,
    ]
    .iter()
    .all(|p| p.dot(center.extend(1.)) + radius * p.truncate().length() >= 0.)
}
#[derive(Default, serde::Serialize)]
pub struct Stats {
    pub enabled: bool,
    pub selected_instances: usize,
    pub source_triangles: usize,
    pub selected_triangles: usize,
    pub maximum_estimated_error_px: f32,
    pub cpu_prepare_ms: f64,
}
pub(crate) struct State {
    pack: Arc<Pack>,
    ready: Vec<Ready>,
    source_assets: Vec<Asset>,
    doc: meshes::Document,
    pub frame: Option<Box<RenderFrame>>,
    pub stats: Stats,
    templates: Vec<crate::distant::selection::Draw>,
}
impl State {
    pub fn new(pack: Arc<Pack>) -> Self {
        Self {
            pack,
            ready: vec![],
            source_assets: vec![],
            doc: Default::default(),
            frame: None,
            stats: Default::default(),
            templates: vec![],
        }
    }
    pub fn prepare(&mut self, source: &RenderFrame, width: u32, height: u32) -> Result<bool> {
        let start = std::time::Instant::now();
        self.stats = Stats::default();
        if source.planet_air.is_some()
            || source.clip.is_some()
            || source.selection_mesh.is_some()
            || source.selection.is_some()
            || !source.meshes.proxy_compatible()
            || !source.particles.samples.is_empty()
        {
            return Ok(false);
        }
        let input = &source.meshes.source;
        if input.assets.len() != self.source_assets.len()
            || input.assets.iter().zip(&self.source_assets).any(|(a, b)| {
                a.id != b.id
                    || !Arc::ptr_eq(&a.geometry, &b.geometry)
                    || a.material != b.material
                    || a.shading != b.shading
            })
        {
            self.ready.clear();
            self.frame = None;
            self.templates.clear();
            self.source_assets = input.assets.clone();
            for (index, e) in self.pack.entries.iter().enumerate() {
                let Some(a) = input.assets.iter().find(|a| a.id == e.asset) else {
                    continue;
                };
                if !eligible(a) || digest(a) != e.hash {
                    continue;
                }
                let full_normals = normals(a);
                let (lo, hi) = crate::distant::bounds(a.geometry.positions.iter().copied());
                let mut levels = vec![];
                for (l, level) in e.levels.iter().enumerate() {
                    ensure!(
                        level.error.is_finite() && level.error >= 0.,
                        "Proxy error estimate invalid"
                    );
                    ensure!(
                        level
                            .vertices
                            .iter()
                            .all(|i| (*i as usize) < a.geometry.positions.len()),
                        "Proxy source index outside matching asset"
                    );
                    let mut positions = level
                        .vertices
                        .iter()
                        .map(|i| a.geometry.positions[*i as usize])
                        .collect::<Vec<_>>();
                    let mut n = level
                        .vertices
                        .iter()
                        .map(|i| full_normals[*i as usize])
                        .collect::<Vec<_>>();
                    positions.extend([lo.to_array(), hi.to_array()]);
                    n.extend([Vec3::Y.to_array(); 2]);
                    let mut asset = a.clone();
                    asset.id = format!("horizon_proxy_{index}_{l}");
                    ensure!(
                        !input.assets.iter().any(|a| a.id == asset.id),
                        "Proxy name collides with source"
                    );
                    asset.geometry = Arc::new(Geometry {
                        positions,
                        triangles: level.triangles.clone(),
                        material_coordinates: None,
                    });
                    levels.push((asset, Arc::new(n), level.error));
                }
                self.ready.push(Ready {
                    source: a.clone(),
                    levels,
                    lo,
                    hi,
                });
            }
            self.doc = input.clone();
            self.doc.assets.extend(
                self.ready
                    .iter()
                    .flat_map(|r| r.levels.iter().map(|l| l.0.clone())),
            );
            self.doc
                .validate()
                .map_err(|e| anyhow::anyhow!(e.message))?;
        }
        if self.ready.is_empty() {
            return Ok(false);
        }
        self.doc.instances.clone_from(&input.instances);
        self.doc.objects = input.objects.clone();
        let eye = Vec3::from(source.camera.position);
        let forward = (Vec3::from(source.camera.target) - eye).normalize();
        let focal = height as f32 / (2. * (source.camera.fov_degrees.to_radians() * 0.5).tan());
        let vp = Mat4::perspective_rh(
            source.camera.fov_degrees.to_radians(),
            width as f32 / height as f32,
            0.01,
            100000.,
        ) * Mat4::look_at_rh(eye, Vec3::from(source.camera.target), Vec3::Y);
        let shadow_vp = crate::render::shadow_view_projection(&source.light);
        let mut selected = 0;
        for instance in &mut self.doc.instances {
            let Some(r) = self.ready.iter().find(|r| r.source.id == instance.asset) else {
                continue;
            };
            let world = Mat4::from_scale_rotation_translation(
                Vec3::splat(instance.scale),
                Quat::from_array(instance.rotation),
                Vec3::from(instance.position),
            );
            let center = world.transform_point3((r.lo + r.hi) * 0.5);
            let radius = (r.hi - r.lo).length() * 0.5 * instance.scale;
            let z = (center - eye).dot(forward) - radius;
            let camera_visible = in_view(vp, center, radius);
            let shadow_visible = shadow_vp.is_some_and(|vp| in_view(vp, center, radius));
            if !camera_visible && !shadow_visible {
                continue;
            }
            let transverse = (center - eye).length() + radius;
            let mut choice = None;
            for (l, (_, _, error)) in r.levels.iter().enumerate() {
                let delta = error * instance.scale;
                if camera_visible && z <= delta + 0.05 {
                    continue;
                }
                let projected = if camera_visible {
                    focal * delta / (z - delta) * (1. + transverse / z)
                } else {
                    0.
                };
                let shadow = if shadow_visible {
                    source
                        .light
                        .shadow
                        .as_ref()
                        .map_or(0., |s| delta * s.resolution as f32 / (2. * s.extent))
                } else {
                    0.
                };
                if projected <= 0.35
                    && shadow <= 0.35
                    && choice.is_none_or(|old: usize| {
                        r.levels[l].0.geometry.triangles.len()
                            < r.levels[old].0.geometry.triangles.len()
                    })
                {
                    choice = Some(l);
                    self.stats.maximum_estimated_error_px =
                        self.stats.maximum_estimated_error_px.max(projected);
                }
            }
            if let Some(l) = choice {
                self.stats.source_triangles += r.source.geometry.triangles.len();
                self.stats.selected_triangles += r.levels[l].0.geometry.triangles.len();
                instance.asset.clone_from(&r.levels[l].0.id);
                selected += 1;
            }
        }
        if selected == 0 {
            return Ok(false);
        }
        let frame = self.frame.get_or_insert_with(|| {
            Box::new(RenderFrame {
                clip: None,
                planet_air: None,
                suppress_diagnostic_grid: source.suppress_diagnostic_grid,
                meshes: Default::default(),
                particles: Default::default(),
                distant_meshes: source.distant_meshes.clone(),
                mesh_lods: source.mesh_lods.clone(),
                mesh_lod_settings: source.mesh_lod_settings,
                camera: source.camera.clone(),
                light: source.light.clone(),
                boxes: vec![],
                player_primitives: Vec::new(),
                vision_guides: vec![],
                hearing_guides: vec![],
                noise_guides: vec![],
                route_guides: vec![],
                truncated_navigation_paths: 0,
                selection: None,
                selection_mesh: None,
            })
        });
        frame.camera = source.camera.clone();
        frame.light = source.light.clone();
        frame.suppress_diagnostic_grid = source.suppress_diagnostic_grid;
        frame.distant_meshes = source.distant_meshes.clone();
        frame.mesh_lods = source.mesh_lods.clone();
        frame.mesh_lod_settings = source.mesh_lod_settings;
        frame.boxes.clone_from(&source.boxes);
        frame
            .player_primitives
            .clone_from(&source.player_primitives);
        frame.vision_guides.clone_from(&source.vision_guides);
        frame.hearing_guides.clone_from(&source.hearing_guides);
        frame.noise_guides.clone_from(&source.noise_guides);
        frame.route_guides.clone_from(&source.route_guides);
        frame.truncated_navigation_paths = source.truncated_navigation_paths;
        let overrides = self
            .ready
            .iter()
            .flat_map(|r| {
                r.levels.iter().map(|(a, n, _)| {
                    (
                        self.doc.assets.iter().position(|b| a.id == b.id).unwrap(),
                        n.clone(),
                    )
                })
            })
            .collect::<Vec<_>>();
        if self.templates.is_empty() {
            self.templates = frame.meshes.prepare_proxy_catalog(&self.doc)?;
            frame.meshes.apply_proxy_normals(&overrides)?;
        }
        frame
            .meshes
            .set_proxy_instances(&self.doc, &self.templates)?;
        self.stats.enabled = true;
        self.stats.selected_instances = selected;
        self.stats.cpu_prepare_ms = start.elapsed().as_secs_f64() * 1000.;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> meshes::Document {
        serde_json::from_value(serde_json::json!({"assets":[{"id":"patch","shading":"smooth","geometry":{"positions":[[-1,0,-1],[1,0,-1],[-1,0,1],[1,0,1]],"triangles":[[0,2,1],[1,2,3]]}}],"instances":[{"id":"one","asset":"patch","position":[0,5,0],"rotation":[0,0,0,1],"scale":1,"color":[0.2,0.6,0.1]}]})).unwrap()
    }
    #[test]
    fn sealed_binary_rejects_corruption_truncation_and_invalid_indices() {
        let p = Pack {
            entries: vec![Entry {
                asset: "patch".into(),
                hash: [0; 32],
                levels: vec![Level {
                    error: 0.01,
                    vertices: vec![0, 1, 2],
                    triangles: vec![[0, 1, 2]],
                }],
            }],
        };
        let b = p.encode().unwrap();
        assert_eq!(
            Pack::decode(&b).unwrap().entries[0].levels[0].vertices,
            vec![0, 1, 2]
        );
        for n in [0, 7, 20, b.len() - 1] {
            assert!(Pack::decode(&b[..n]).is_err());
        }
        let mut bad = b.clone();
        bad[30] ^= 1;
        assert!(Pack::decode(&bad).is_err());
        let mut bad = b.clone();
        let end = bad.len() - 32;
        bad[end - 4..end].copy_from_slice(&3u32.to_le_bytes());
        let seal = ring::digest::digest(&ring::digest::SHA256, &bad[..end]);
        bad[end..].copy_from_slice(seal.as_ref());
        assert!(Pack::decode(&bad).is_err());
    }
    #[test]
    fn source_identity_fallback_and_camera_changes_reuse_geometry() {
        let mut doc = fixture();
        doc.assets[0].geometry = Arc::new(Geometry::grid(8, 1.).unwrap());
        let mut packet = compile(&doc).unwrap();
        for (i, l) in packet.entries[0].levels.iter_mut().enumerate() {
            l.error = 0.001 * 10f32.powi(i as i32);
        }
        let pack = Arc::new(packet);
        assert!(!pack.entries.is_empty());
        let mut scene = crate::model::RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap();
        scene.meshes = doc.clone();
        scene.camera.position = [0., 12., 30.];
        scene.camera.target = [0., 5., 0.];
        scene.light.shadow = None;
        let e = crate::engine::Engine::new(scene).unwrap();
        let mut frame = e.render_frame();
        let mut state = State::new(pack);
        assert!(state.prepare(&frame, 1100, 760).unwrap());
        assert!(state.stats.selected_instances > 0);
        assert_eq!(
            frame.meshes.source, doc,
            "Source frame must remain canonical"
        );
        let generation = state.frame.as_ref().unwrap().meshes.geometry_generation;
        frame.camera.position[0] += 1.;
        assert!(state.prepare(&frame, 1100, 760).unwrap());
        assert_eq!(
            state.frame.as_ref().unwrap().meshes.geometry_generation,
            generation,
            "Camera must not rebuild geometry"
        );
        let mut choices = BTreeSet::new();
        for z in [30., 200., 1000.] {
            frame.camera.position = [0., 12., z];
            assert!(state.prepare(&frame, 1100, 760).unwrap());
            choices.insert(
                state.frame.as_ref().unwrap().meshes.source.instances[0]
                    .asset
                    .clone(),
            );
            assert_eq!(
                state.frame.as_ref().unwrap().meshes.geometry_generation,
                generation,
                "LOD switches must retain resident geometry"
            );
        }
        assert!(choices.len() >= 2, "Exercise more than one level");
        let mut changed = doc.clone();
        Arc::make_mut(&mut changed.assets[0].geometry).positions[0][1] += 0.1;
        frame.meshes.update(&changed, 0);
        assert!(
            !state.prepare(&frame, 1100, 760).unwrap(),
            "Edited source must fall back"
        );
    }
    #[test]
    fn compiler_preserves_separate_surface_components() {
        let mut doc = fixture();
        let g = Geometry::grid(8, 1.).unwrap();
        let mut geometry = g.clone();
        let base = geometry.positions.len() as u32;
        geometry
            .positions
            .extend(g.positions.iter().map(|p| [p[0] + 20., p[1], p[2]]));
        geometry
            .triangles
            .extend(g.triangles.iter().map(|t| t.map(|i| i + base)));
        doc.assets[0].geometry = Arc::new(geometry);
        let pack = compile(&doc).unwrap();
        assert!(!pack.entries.is_empty());
        let (labels, parts) = components(&doc.assets[0].geometry);
        assert_eq!(parts.len(), 2);
        for l in &pack.entries[0].levels {
            let used = l
                .triangles
                .iter()
                .map(|t| labels[l.vertices[t[0] as usize] as usize])
                .collect::<BTreeSet<_>>();
            assert_eq!(parts, used);
        }
    }
}
