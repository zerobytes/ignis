//! Native single-player surface residency. Immutable tile geometry, mutable local frames.
use super::{
    Document, Pose, Sampler, mesh,
    residency::{self, Anchor, Request, Tile},
};
use crate::{
    meshes::{Asset, Instance},
    model::{Light, Result, invalid},
    surfaces::Surface,
};
use glam::DVec3;
use rapier3d::{
    na::{Quaternion, UnitQuaternion},
    prelude::*,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub document: Document,
    pub actor: String,
    /// Absolute radial height of the actor center above the reference sphere.
    pub spawn: Pose,
    pub level: u8,
    pub subdivisions: u16,
    pub physical_radius_meters: f64,
    pub color: [f32; 3],
    #[serde(default)]
    pub material: crate::materials::Material,
    #[serde(default = "compile_default")]
    pub compile_distant_meshes: bool,
}
fn compile_default() -> bool {
    true
}
struct Resource {
    anchor: Anchor,
    asset: Asset,
    shape: Option<SharedShape>,
    pack: Option<crate::distant::Pack>,
    normals: Arc<Vec<[f32; 3]>>,
}
pub(crate) struct World {
    pub config: Config,
    pub anchor: Anchor,
    pub actor_pose: Pose,
    pub meshes: crate::meshes::Document,
    pub packs: Arc<Vec<crate::distant::Pack>>,
    pub normals: Vec<Arc<Vec<[f32; 3]>>>,
    pub surfaces: Vec<Surface>,
    cache: BTreeMap<Tile, Resource>,
    visual: Vec<Tile>,
    physical: Vec<Tile>,
    planned_at: Pose,
    pub visual_radius: f64,
    initial_up: DVec3,
    initial_forward: DVec3,
    principal_angle: f64,
    pub meridian_progress: f64,
    global_light: DVec3,
    pub rebases: u64,
    pub births: u64,
    pub retirements: u64,
    pub distance: f64,
    pub angular_path: f64,
    pub faces: BTreeSet<mesh::Face>,
}
fn transform(anchor: Anchor, next: Anchor) -> ([f32; 3], [f32; 4]) {
    (
        anchor.origin_in(next).as_vec3().to_array(),
        anchor.rotation_to(next).as_quat().to_array(),
    )
}
fn surface(id: String, resource: &Resource, anchor: Anchor) -> Surface {
    let (p, q) = transform(resource.anchor, anchor);
    Surface {
        id: id.clone(),
        asset: id,
        shape: resource
            .shape
            .as_ref()
            .expect("prepared physical tile")
            .clone(),
        pose: Isometry::from_parts(
            Translation::from(p),
            UnitQuaternion::new_normalize(Quaternion::new(q[3], q[0], q[1], q[2])),
        ),
    }
}
impl World {
    pub fn new(config: Config, light: &Light, half_height: f32) -> Result<Self> {
        config.document.validate()?;
        config.material.validate()?;
        light.validate()?;
        if config.level > 24
            || !config.subdivisions.is_power_of_two()
            || !(32..=128).contains(&config.subdivisions)
            || !config.physical_radius_meters.is_finite()
            || !(64. ..=2000.).contains(&config.physical_radius_meters)
            || config
                .color
                .iter()
                .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        {
            return Err(invalid(
                "Planet runtime requires level0..24, subdivisions32/64/128, physical radius64..2000m and RGB0..1",
            ));
        }
        // This first adapter only omits opaque terrain once it is indistinguishable
        // from the ACTUAL background, not merely once its material can be simplified.
        let air=light.atmosphere.ok_or_else(||invalid("Planet residency needs an atmosphere/background coverage certificate; clear-air fallback is not integrated"))?;
        if air.density <= 0.
            || air.color_error <= 0.
            || air.edge_softness.is_some_and(|e| e.active())
            || air.layers.is_some()
            || light.shadow.is_some()
            || light.environment.is_some_and(|e| e.intensity > 0.)
            || config.material.transmission > 0.
            || config
                .material
                .reflectance
                .is_some_and(|r| r.f0 > 0. || r.metallic > 0.)
        {
            return Err(invalid(
                "First planet residency adapter requires matching atmosphere/background (no layers), opaque diffuse terrain, no shadow or IBL; unsupported visibility is never silently clipped",
            ));
        }
        // Same diffuse kernel as the shader (no dielectric/metal/transmission/IBL).
        // A=B for this background. Both this terrain and any terrain behind it
        // obey the same upper bound: missing-surface error <= 2*C*T(d).
        let upper =
            crate::distant::selection::bound(&config.material, glam::Vec3::from(config.color));
        let illumination = light.ambient + (1. - light.ambient) * light.direct_intensity;
        let contrast = f64::from((upper.max_element() * illumination).max(1.)) * 1.001;
        let epsilon = f64::from(air.color_error) * 0.1;
        let visual_radius = (2. * contrast / epsilon).ln() / f64::from(air.density);
        if visual_radius > 27000. || visual_radius < config.physical_radius_meters + 128. {
            return Err(invalid(
                "Atmospheric coverage must fit the current30km projection and include physical prefetch; no finite clear-air cutoff is inferred",
            ));
        }
        let spawn_frame = config.spawn.frame()?;
        let actor_pose = Pose {
            direction: spawn_frame.up,
            forward: spawn_frame.forward,
            ..config.spawn
        };
        let spawn_unit = DVec3::from(spawn_frame.up);
        let surface_height = Sampler::new(&config.document)?.height(spawn_unit);
        if actor_pose.altitude_meters < surface_height + f64::from(half_height) {
            return Err(invalid(
                "Planet actor center must start above its source surface",
            ));
        }
        config
            .document
            .center
            .translated(
                (spawn_unit * (config.document.radius() + actor_pose.altitude_meters)).to_array(),
            )
            .map_err(|e| invalid(e.to_string()))?;
        let anchor = Anchor::new(
            &config.document,
            Pose {
                altitude_meters: actor_pose.altitude_meters - f64::from(half_height),
                ..actor_pose
            },
        )?;
        let global_light = anchor.basis * DVec3::from(light.direction.map(f64::from));
        let initial_up = anchor.basis.y_axis;
        let initial_forward = -anchor.basis.z_axis;
        let mut out = Self {
            config,
            anchor,
            actor_pose,
            meshes: Default::default(),
            packs: Default::default(),
            normals: vec![],
            surfaces: vec![],
            cache: BTreeMap::new(),
            visual: vec![],
            physical: vec![],
            planned_at: actor_pose,
            visual_radius,
            initial_up,
            initial_forward,
            principal_angle: 0.,
            meridian_progress: 0.,
            global_light,
            rebases: 0,
            births: 0,
            retirements: 0,
            distance: 0.,
            angular_path: 0.,
            faces: BTreeSet::new(),
        };
        out.prepare(true)?;
        Ok(out)
    }
    fn resource(&self, tile: Tile) -> Result<Resource> {
        let unit = tile.direction([0.5, 0.5]);
        let sampler = Sampler::new(&self.config.document)?;
        let h = sampler.height(unit);
        let heading = if unit.dot(DVec3::Z).abs() < 0.9 {
            DVec3::Z
        } else {
            DVec3::X
        };
        let pose = Pose {
            direction: unit.to_array(),
            forward: heading.to_array(),
            altitude_meters: h,
        };
        let anchor = Anchor::new(&self.config.document, pose)?;
        let compiled = mesh::compile(
            &self.config.document,
            &tile.area(self.config.subdivisions),
            &mesh::Projection::Surface { pose },
        )?;
        let mut geometry = Arc::try_unwrap(compiled.geometry).expect("unpublished planet geometry");
        // Canonical material domain is tied to the sphere, not the observing frame.
        // Keep it inside the existing authoring coordinate contract.
        let n = usize::from(self.config.subdivisions);
        let mut coordinates = Vec::with_capacity(geometry.positions.len());
        let mut normals = Vec::with_capacity(geometry.positions.len());
        for y in 0..=n {
            for x in 0..=n {
                let unit = tile.direction([x as f64 / n as f64, y as f64 / n as f64]);
                coordinates.push((unit * 4096.).as_vec3().to_array());
                let normal = continuous_normal(&sampler, self.config.document.radius(), unit);
                normals.push((anchor.basis.transpose() * normal).as_vec3().to_array());
            }
        }
        geometry.material_coordinates = Some(coordinates);
        geometry.validate()?;
        let mut material = self.config.material.clone();
        for p in [&mut material.pattern, &mut material.secondary_pattern]
            .into_iter()
            .flatten()
        {
            for f in &mut p.frequency {
                *f *= self.config.document.radius() as f32 / 4096.;
            }
        }
        material.validate()?;
        let asset = Asset {
            id: format!("planet_{}", tile.label()),
            geometry: Arc::new(geometry),
            shading: crate::meshes::Shading::Smooth,
            material,
            rig: None,
            clips: vec![],
        };
        let pack = if self.config.compile_distant_meshes {
            crate::distant::compile(&crate::meshes::Document {
                assets: vec![asset.clone()],
                instances: vec![],
                objects: Default::default(),
            })?
            .first()
            .cloned()
        } else {
            None
        };
        Ok(Resource {
            anchor,
            asset,
            shape: None,
            pack,
            normals: Arc::new(normals),
        })
    }
    pub fn prepare(&mut self, force: bool) -> Result<bool> {
        let r = self.config.document.radius() + self.actor_pose.altitude_meters;
        let unit = DVec3::from(self.actor_pose.direction);
        let old_unit = DVec3::from(self.planned_at.direction);
        let moved = (r * (unit - old_unit)
            + old_unit * (self.actor_pose.altitude_meters - self.planned_at.altitude_meters))
            .length();
        if !force && moved < 64. {
            return Ok(false);
        }
        let request = Request {
            pose: self.actor_pose,
            level: self.config.level,
            // Covers 64m since planning + next <=17m step + <=125m player
            // camera offset, body extent and numerical guard. Refresh uses 3D motion.
            visual_radius_meters: self.visual_radius + 512.,
            physical_radius_meters: self.config.physical_radius_meters + 64.,
        };
        let plan = residency::plan(&self.config.document, &request)?;
        let vertices = plan.visual.len() * (usize::from(self.config.subdivisions) + 1).pow(2);
        if vertices > 2_000_000 {
            return Err(invalid(
                "Planet residence exceeds the2M source-vertex memory budget; previous coverage is retained",
            ));
        }
        if !force && plan.visual == self.visual && plan.physical == self.physical {
            self.planned_at = self.actor_pose;
            return Ok(false);
        }
        let visual_changed = force || plan.visual != self.visual;
        let physical: BTreeSet<_> = plan.physical.iter().copied().collect();
        let mut staged = vec![];
        for &tile in &plan.visual {
            if !self.cache.contains_key(&tile) {
                let mut resource = self.resource(tile)?;
                if physical.contains(&tile) {
                    Self::cook(&mut resource)?;
                }
                staged.push((tile, resource));
            }
        }
        // Cooking an existing resource adds only a derivation, not live coverage.
        // All fallible work finishes before changing sets or retiring old resources.
        for tile in &plan.physical {
            if let Some(r) = self.cache.get_mut(tile) {
                Self::cook(r)?;
            }
        }
        self.births += staged.len() as u64;
        let before = self.cache.len();
        self.cache
            .retain(|tile, _| plan.visual.binary_search(tile).is_ok());
        self.retirements += (before - self.cache.len()) as u64;
        self.cache.extend(staged);
        self.visual = plan.visual;
        self.physical = plan.physical;
        self.planned_at = self.actor_pose;
        if visual_changed {
            self.normals = self
                .visual
                .iter()
                .map(|t| self.cache[t].normals.clone())
                .collect();
            self.meshes.assets = self
                .visual
                .iter()
                .map(|t| self.cache[t].asset.clone())
                .collect();
            self.meshes.instances = self
                .visual
                .iter()
                .map(|t| {
                    let r = &self.cache[t];
                    let (position, rotation) = transform(r.anchor, self.anchor);
                    Instance {
                        id: r.asset.id.clone(),
                        asset: r.asset.id.clone(),
                        position,
                        rotation,
                        scale: 1.,
                        color: self.config.color,
                        playback: None,
                        attachment: None,
                    }
                })
                .collect();
            self.packs = Arc::new(
                self.visual
                    .iter()
                    .filter_map(|t| self.cache[t].pack.clone())
                    .collect(),
            );
        }
        self.refresh_surfaces();
        Ok(true)
    }
    fn cook(resource: &mut Resource) -> Result<()> {
        if resource.shape.is_none() {
            let g = &resource.asset.geometry;
            resource.shape = Some(
                SharedShape::trimesh(
                    g.positions
                        .iter()
                        .map(|p| point![p[0], p[1], p[2]])
                        .collect(),
                    g.triangles.clone(),
                )
                .map_err(|e| invalid(format!("Planet collision compilation failed: {e}")))?,
            );
        }
        Ok(())
    }
    fn refresh_surfaces(&mut self) {
        if self.surfaces.len() == self.physical.len()
            && self
                .surfaces
                .iter()
                .zip(&self.physical)
                .all(|(s, t)| s.id == self.cache[t].asset.id)
        {
            for (s, t) in self.surfaces.iter_mut().zip(&self.physical) {
                let (p, q) = transform(self.cache[t].anchor, self.anchor);
                s.pose = Isometry::from_parts(
                    Translation::from(p),
                    UnitQuaternion::new_normalize(Quaternion::new(q[3], q[0], q[1], q[2])),
                );
            }
            return;
        }
        self.surfaces.clear();
        self.surfaces.extend(self.physical.iter().map(|t| {
            let r = &self.cache[t];
            surface(r.asset.id.clone(), r, self.anchor)
        }));
    }
    /// Called ONLY with the position actually produced by the collision controller.
    pub fn transport(&mut self, position: [f32; 3], half_height: f32) -> glam::DQuat {
        let old = self.anchor;
        let local = DVec3::from(position.map(f64::from));
        let (unit, height) = old.unproject(local);
        let previous = old.basis.y_axis;
        let principal = unit
            .dot(self.initial_forward)
            .atan2(unit.dot(self.initial_up));
        let delta = (principal - self.principal_angle + std::f64::consts::PI)
            .rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        self.meridian_progress += delta;
        self.principal_angle = principal;
        self.distance += (local - DVec3::new(0., f64::from(half_height), 0.)).length();
        self.angular_path += previous.cross(unit).length().atan2(previous.dot(unit));
        let next = old
            .transported(&self.config.document, unit, height - f64::from(half_height))
            .expect("finite physical position");
        let q = old.rotation_to(next);
        self.actor_pose = Pose {
            direction: unit.to_array(),
            forward: next.pose.forward,
            altitude_meters: height,
        };
        self.anchor = next;
        self.rebases += 1;
        for (tile, i) in self.visual.iter().zip(&mut self.meshes.instances) {
            (i.position, i.rotation) = transform(self.cache[tile].anchor, next);
        }
        self.refresh_surfaces();
        let axis = unit.abs().max_element();
        for face in residency::FACES {
            if (Tile {
                face,
                level: 0,
                xy: [0, 0],
            })
            .direction([0.5, 0.5])
            .dot(unit)
                >= axis - 1e-10
            {
                self.faces.insert(face);
            }
        }
        q
    }
    pub fn local_light(&self) -> [f32; 3] {
        (self.anchor.basis.transpose() * self.global_light)
            .as_vec3()
            .to_array()
    }
    pub fn inspect(&self) -> serde_json::Value {
        let unit = DVec3::from(self.actor_pose.direction);
        let position = self
            .config
            .document
            .center
            .translated(
                (unit * (self.config.document.radius() + self.actor_pose.altitude_meters))
                    .to_array(),
            )
            .expect("validated planet position");
        serde_json::json!({"active":true,"session_only":true,"pose":self.actor_pose,"position":position,"actor":self.config.actor,
            "diameter_meters":self.config.document.diameter_meters,"source_id":self.config.document.id,"source_revision":self.config.document.revision,
            "visual_radius_meters":self.visual_radius,"visual_tiles":self.visual.len(),"physical_tiles":self.physical.len(),
            "rebases":self.rebases,"tile_births":self.births,"tile_retirements":self.retirements,"actual_path_meters":self.distance,
            "angular_path_radians":self.angular_path,"distance_from_spawn_meters":self.config.document.radius()*unit.cross(self.initial_up).length().atan2(unit.dot(self.initial_up)),
            "initial_meridian_progress_radians":self.meridian_progress,
            "visited_faces":self.faces,"geometry_immutable_per_tile":true,"geometric_query_only":false})
    }
}

fn continuous_normal(sampler: &Sampler<'_>, radius: f64, unit: DVec3) -> DVec3 {
    if sampler.document.operations.is_empty() {
        return unit;
    }
    let reference = if unit.z.abs() < 0.9 {
        DVec3::Z
    } else {
        DVec3::X
    };
    let tangent = reference.cross(unit).normalize();
    let bitangent = unit.cross(tangent);
    let step = 0.25f64.min(radius * 0.0001);
    let gradient = |axis: DVec3| {
        (sampler.height((unit + axis * (step / radius)).normalize())
            - sampler.height((unit - axis * (step / radius)).normalize()))
            / (2. * step)
    };
    let factor = radius / (radius + sampler.height(unit));
    (unit - factor * (tangent * gradient(tangent) + bitangent * gradient(bitangent))).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> (crate::model::Scene, Config) {
        let mut scene = crate::model::RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap();
        scene.entities.retain(|e| e.id == "player");
        let p = &mut scene.entities[0];
        p.visible = Some(false);
        p.character_controller = Some(Default::default());
        p.footsteps = None;
        scene.light.atmosphere = Some(crate::atmosphere::Atmosphere {
            density: 0.01,
            sun_strength: 0.,
            ..Default::default()
        });
        let document:Document=serde_json::from_value(serde_json::json!({"version":1,"space_version":1,"id":"normals","revision":1,
            "center":{"cell":["0","0","0"],"offset":[0,0,0]},"diameter_meters":2000.,"surface_gravity_mps2":9.81,
            "base_height_meters":0.,"operations":[{"kind":"noise","seed":731,"wavelength_meters":300.,"amplitude_meters":5.,"octaves":3,"persistence":0.5}]})).unwrap();
        let height = Sampler::new(&document).unwrap().height(DVec3::Y);
        let config = Config {
            document,
            actor: "player".into(),
            spawn: Pose {
                direction: [0., 1., 0.],
                forward: [0., 0., -1.],
                altitude_meters: height + 0.91,
            },
            level: 2,
            subdivisions: 32,
            physical_radius_meters: 128.,
            color: [0.4, 0.6, 0.3],
            material: Default::default(),
            compile_distant_meshes: false,
        };
        (scene, config)
    }
    #[test]
    fn normals_match_across_every_resident_tile_and_face_seam() {
        let (scene, config) = source();
        let world = World::new(config, &scene.light, 0.8).unwrap();
        let mut normals = BTreeMap::new();
        let mut seams = 0;
        for (tile, r) in &world.cache {
            let n = i64::from(world.config.subdivisions);
            let denominator = n << tile.level;
            for y in 0..=n {
                for x in 0..=n {
                    let key = tile.face.key(
                        2 * (i64::from(tile.xy[0]) * n + x) - denominator,
                        2 * (i64::from(tile.xy[1]) * n + y) - denominator,
                        denominator,
                    );
                    let local = DVec3::from(r.normals[(y * (n + 1) + x) as usize].map(f64::from));
                    let global = r.anchor.basis * local;
                    if let Some(previous) = normals.insert(key, global) {
                        assert!(global.distance(previous) < 2e-7);
                        seams += 1;
                    }
                }
            }
        }
        assert!(seams > 1000);
    }
    #[test]
    #[ignore = "requires native GPU"]
    fn derived_normals_have_their_own_resident_payload_identity() {
        let (scene, config) = source();
        let mut e =
            crate::engine::Engine::new_with_mode(scene, crate::engine::ExecutionMode::Continuous)
                .unwrap();
        e.execute(
            crate::engine::Command::SetPlanetRuntime {
                config: Some(Box::new(config)),
            }
            .into(),
        )
        .unwrap();
        let world = e.simulation.planet.as_ref().unwrap();
        let slot = world
            .visual
            .iter()
            .position(|t| t.face == mesh::Face::PositiveY && t.xy == [1, 2])
            .unwrap();
        let mut frame = e.render_frame();
        frame.camera = crate::model::Camera {
            position: [0., 100., 140.],
            target: [0., 0., 0.],
            fov_degrees: 60.,
        };
        let gpu = wgpu::Instance::default();
        let mut renderer = pollster::block_on(crate::render::Renderer::new(&gpu, None)).unwrap();
        renderer.set_visibility_culling(false);
        renderer.set_occlusion_culling(false);
        let before = renderer.capture(&frame, 320, 240).unwrap();
        let uploads = renderer.info()["instancing"]["geometry_uploads"]
            .as_u64()
            .unwrap();
        let mut changed = world.normals.clone();
        let q = glam::Quat::from_rotation_x(0.8);
        changed[slot] = Arc::new(
            changed[slot]
                .iter()
                .map(|n| (q * glam::Vec3::from(*n)).to_array())
                .collect(),
        );
        frame.meshes.apply_smooth_normals(&changed).unwrap();
        let after = renderer.capture(&frame, 320, 240).unwrap();
        assert_eq!(
            renderer.info()["instancing"]["geometry_uploads"]
                .as_u64()
                .unwrap()
                - uploads,
            1
        );
        let a = crate::image_similarity::Image::decode(&before).unwrap();
        let b = crate::image_similarity::Image::decode(&after).unwrap();
        assert!(
            a.pixels
                .iter()
                .zip(&b.pixels)
                .filter(|(a, b)| a != b)
                .count()
                > 100,
            "normal edit must be visible"
        );
        renderer.set_geometry_indexing(false);
        renderer.set_geometry_indexing(true);
        let reference = renderer.capture(&frame, 320, 240).unwrap();
        assert_eq!(
            b.pixels,
            crate::image_similarity::Image::decode(&reference)
                .unwrap()
                .pixels
        );
    }

    #[test]
    #[ignore = "requires native GPU; atmospheric coverage versus a larger native residence"]
    fn distant_coverage_matches_a_larger_residence_at_faces_poles_and_exchange() {
        let gpu = wgpu::Instance::default();
        let mut renderer = pollster::block_on(crate::render::Renderer::new(&gpu, None)).unwrap();
        let mut exposed_pixels = 0;
        for (direction, fov, size) in [
            ([0., 1., 0.], 60., [320, 240]),
            (
                [
                    std::f64::consts::FRAC_1_SQRT_2,
                    std::f64::consts::FRAC_1_SQRT_2,
                    0.,
                ],
                90.,
                [480, 270],
            ),
            ([0., 0., 1.], 45., [320, 320]),
        ] {
            let (mut scene, mut config) = source();
            scene.light.atmosphere.as_mut().unwrap().density = 0.0008;
            config.document.diameter_meters = 12_742_000.;
            config.document.operations.clear();
            config.level = 12;
            config.spawn.direction = direction;
            config.spawn.forward = if direction[2] == 1. {
                [0., 1., 0.]
            } else {
                [0., 0., -1.]
            };
            config.spawn.altitude_meters = 1000.;
            config.compile_distant_meshes = true;
            let mut e = crate::engine::Engine::new_with_mode(
                scene,
                crate::engine::ExecutionMode::Continuous,
            )
            .unwrap();
            e.execute(
                crate::engine::Command::SetPlanetRuntime {
                    config: Some(Box::new(config)),
                }
                .into(),
            )
            .unwrap();
            for exchange in [false, true] {
                if exchange {
                    let world = e.simulation.planet.as_mut().unwrap();
                    world.transport([0., 0.8, -1800.], 0.8);
                    world.prepare(false).unwrap();
                }
                let mut frame = e.render_frame();
                // Same supported player-relative offset; the actor itself is airborne.
                frame.camera = crate::model::Camera {
                    position: [0., 2., 0.],
                    target: [0., 2., -15000.],
                    fov_degrees: fov,
                };
                renderer.set_visibility_culling(true);
                renderer.set_occlusion_culling(true);
                let fast = renderer.capture(&frame, size[0], size[1]).unwrap();
                renderer.set_visibility_culling(false);
                renderer.set_occlusion_culling(false);
                let oracle = renderer.capture(&frame, size[0], size[1]).unwrap();
                let decode = |b: &[u8]| crate::image_similarity::Image::decode(b).unwrap().pixels;
                assert_eq!(
                    decode(&fast),
                    decode(&oracle),
                    "culling must preserve RGB8 at residence exchange"
                );
                let world = e.simulation.planet.as_mut().unwrap();
                let radius = world.visual_radius;
                world.visual_radius = radius * 1.5;
                world.prepare(true).unwrap();
                let mut reference = e.render_frame();
                reference.camera = frame.camera.clone();
                reference.light.atmosphere.as_mut().unwrap().reduce_details = false;
                let expanded = renderer.capture(&reference, size[0], size[1]).unwrap();
                let a = decode(&fast);
                let b = decode(&expanded);
                let maximum = a
                    .iter()
                    .zip(&b)
                    .flat_map(|(a, b)| (0..3).map(move |c| a[c].abs_diff(b[c])))
                    .max()
                    .unwrap();
                assert!(
                    maximum <= 1,
                    "atmospheric omission/detail budget exceeded one RGB8 level: {maximum}"
                );
                // The comparison must contain genuinely missing distant ground,
                // not an empty/background-only annulus hidden by framing.
                frame.light.atmosphere = None;
                reference.light.atmosphere = None;
                let a = decode(&renderer.capture(&frame, size[0], size[1]).unwrap());
                let b = decode(&renderer.capture(&reference, size[0], size[1]).unwrap());
                let visible = a
                    .iter()
                    .zip(&b)
                    .filter(|(a, b)| (0..3).any(|c| a[c].abs_diff(b[c]) > 10))
                    .count();
                assert!(
                    visible > 100,
                    "distant coverage fixture must expose omitted ground without atmosphere: {visible}"
                );
                exposed_pixels += visible;
                let world = e.simulation.planet.as_mut().unwrap();
                world.visual_radius = radius;
                world.prepare(true).unwrap();
            }
        }
        eprintln!(
            "coverage:6 native poses,12 optimized/reference comparisons; exposed annulus pixels={exposed_pixels}"
        );
    }
}
