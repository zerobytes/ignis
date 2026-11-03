//! The existing KCC remains the sole movement authority; only its resident frame changes.
use super::*;
use crate::model::{Result, invalid};

impl Simulation {
    pub(crate) fn enable_planet(
        &mut self,
        config: crate::planet::runtime::Config,
        light: &crate::model::Light,
    ) -> Result<()> {
        let entity = self
            .scene
            .entities
            .iter()
            .find(|e| e.id == config.actor)
            .ok_or_else(|| invalid("Planet player missing"))?;
        let half = entity.transform.size[1] * 0.5;
        let candidate = crate::planet::runtime::World::new(config, light, half)?;
        let actor = candidate.config.actor.clone();
        let body = &mut self.bodies[self.handles[&actor]];
        body.set_position(Isometry::translation(0., half, 0.), true);
        body.set_next_kinematic_position(*body.position());
        body.set_linvel(Vector::zeros(), true);
        self.planet = Some(Box::new(candidate));
        self.sync_planet_surfaces();
        self.publish_planet_aabbs();
        Ok(())
    }
    pub(crate) fn prepare_planet_step(&mut self) -> Result<()> {
        if let Some(world) = self.planet.as_mut()
            && world.prepare(false)?
        {
            self.sync_planet_surfaces();
            self.publish_planet_aabbs();
        }
        Ok(())
    }
    fn sync_planet_surfaces(&mut self) {
        let world = self.planet.as_ref().expect("planet enabled");
        self.surface_bodies.retain(|id, &mut handle| {
            if world.surfaces.iter().any(|s| s.id == *id) {
                return true;
            }
            self.bodies.remove(
                handle,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse,
                &mut self.multibody,
                true,
            );
            false
        });
        for surface in &world.surfaces {
            let handle = if let Some(&h) = self.surface_bodies.get(&surface.id) {
                h
            } else {
                let h = self
                    .bodies
                    .insert(RigidBodyBuilder::fixed().pose(surface.pose));
                self.colliders.insert_with_parent(
                    ColliderBuilder::new(surface.shape.clone())
                        .friction(0.7)
                        .restitution(0.),
                    h,
                    &mut self.bodies,
                );
                self.surface_bodies.insert(surface.id.clone(), h);
                h
            };
            let body = &mut self.bodies[handle];
            body.set_position(surface.pose, false);
            self.colliders[body.colliders()[0]].set_position(surface.pose);
        }
        if self.physical_surfaces.len() == world.surfaces.len()
            && self
                .physical_surfaces
                .iter()
                .zip(&world.surfaces)
                .all(|(a, b)| a.id == b.id)
        {
            for (a, b) in self.physical_surfaces.iter_mut().zip(&world.surfaces) {
                a.pose = b.pose;
            }
        } else {
            self.physical_surfaces.clone_from(&world.surfaces);
        }
    }
    fn publish_planet_aabbs(&mut self) {
        let params = IntegrationParameters {
            dt: FIXED_DT,
            ..Default::default()
        };
        for (h, c) in self.colliders.iter() {
            if c.is_enabled() {
                self.broad.set_aabb(&params, h, c.compute_aabb());
            }
        }
    }
    pub(crate) fn transport_planet_after_step(&mut self) {
        let Some(world) = self.planet.as_mut() else {
            return;
        };
        let entity = self
            .scene
            .entities
            .iter()
            .find(|e| e.id == world.config.actor)
            .expect("planet player");
        let id = &entity.id;
        let half = entity.transform.size[1] * 0.5;
        let body = &mut self.bodies[self.handles[id]];
        let q = world.transport((*body.translation()).into(), half);
        let velocity = (q * glam::DVec3::new(
            f64::from(body.linvel().x),
            f64::from(body.linvel().y),
            f64::from(body.linvel().z),
        ))
        .as_vec3();
        body.set_position(Isometry::translation(0., half, 0.), true);
        body.set_next_kinematic_position(*body.position());
        body.set_linvel(vector![velocity.x, velocity.y, velocity.z], true);
        // Re-express the previous intentions too; they are observations, not an extra move.
        if let Some(state) = self.character_controllers.get_mut(id) {
            for value in [
                &mut state.desired_translation,
                &mut state.corrected_translation,
            ] {
                *value = (q * glam::DVec3::from(value.map(f64::from)))
                    .as_vec3()
                    .to_array();
            }
        }
        self.sync_planet_surfaces();
        self.publish_planet_aabbs();
    }
}
