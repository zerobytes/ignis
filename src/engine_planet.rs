use super::*;
impl Engine {
    pub fn planet_active(&self) -> bool {
        self.simulation.planet.is_some()
    }
    pub fn inspect_planet_runtime(&self) -> Value {
        if let Some(world) = &self.simulation.planet {
            let mut report = world.inspect();
            report["tick"] = json!(self.simulation.tick);
            report["controller"] =
                json!(self.simulation.character_controllers[&world.config.actor]);
            report
        } else {
            json!({"active":false})
        }
    }
    pub(super) fn enable_planet_runtime(
        &mut self,
        config: crate::planet::runtime::Config,
    ) -> Result<()> {
        let e = self
            .scene
            .entities
            .iter()
            .find(|e| e.id == config.actor)
            .ok_or_else(|| invalid("Planet runtime player missing"))?;
        if self.recording()
            || self.playing
            || self.simulation.tick != 0
            || self.scene.entities.len() != 1
            || e.body != Some(BodyKind::Player)
            || e.character_controller.is_none()
            || !e.geometry.is_box()
            || e.abilities.is_some()
            || e.vitality.is_some()
            || e.avatar.is_some()
            || e.footsteps.is_some()
            || e.trigger.is_some()
            || e.transform.size.iter().any(|v| *v > 10.)
            || !self.scene.meshes.is_empty()
            || !self.scene.npc_templates.is_empty()
            || !self.scene.npc_generators.is_empty()
            || !self.scene.passages.is_empty()
            || self.scene.navigation_generation.is_some()
            || !self.scene.interactions.is_empty()
            || !self.controls.state.bindings.is_empty()
            || self.input != Input::default()
            || !self.noises.is_empty()
            || !self.ability_inputs.is_empty()
            || !self.population_inputs.is_empty()
        {
            return Err(invalid(
                "First planet runtime requires paused tick0 continuous execution, one small unbound Player/KCC and no authored meshes/NPCs/gameplay components; other systems have not migrated to transported coordinates",
            ));
        }
        let gravity = config.document.surface_gravity_mps2;
        if (f64::from(e.character_controller.unwrap().gravity) - gravity).abs() > 1e-5 {
            return Err(invalid(
                "Player controller gravity must match planet surface gravity",
            ));
        }
        let actor = config.actor.clone();
        let eye = (e.transform.size[1] * 0.9).min(1.65);
        self.simulation.enable_planet(config, &self.scene.light)?;
        if self.player_view.is_none() {
            self.execute(
                Command::SetPlayerView {
                    view: Some(crate::player_view::Config {
                        actor,
                        yaw_degrees: 0.,
                        pitch_degrees: 0.,
                        eye_height: eye,
                        fov_degrees: 60.,
                        mode: Default::default(),
                        third_person: Default::default(),
                    }),
                }
                .into(),
            )?;
        }
        Ok(())
    }
}
