//! Session camera/visuals only: no actor, scene, replay or physics mutations.
use super::*;
use crate::planet::orbit::{Atlas, Settings};
impl Engine {
    pub fn inspect_planet_view(&self) -> Value {
        self.planet_view
            .as_ref()
            .map_or(json!({"active":false}), Atlas::inspect)
    }
    /// Current orbital settings, or defaults fitted to the active planet/player.
    pub fn planet_view_settings(&self) -> Result<Settings> {
        let world = self
            .simulation
            .planet
            .as_ref()
            .ok_or_else(|| invalid("Planet view requires an active planet runtime"))?;
        if let Some(v) = &self.planet_view {
            return Ok(v.settings.clone());
        }
        let player = self
            .player_view_config()
            .ok_or_else(|| invalid("Planet view requires a player camera"))?;
        Ok(Settings {
            distance_meters: if player.mode == crate::player_view::Mode::ThirdPerson {
                f64::from(player.third_person.distance_meters)
            } else {
                0.
            },
            yaw_degrees: f64::from(player.yaw_degrees),
            tilt_degrees: f64::from(player.pitch_degrees),
            fov_degrees: f64::from(player.fov_degrees),
            atmosphere_height_meters: 8000f64.min(world.config.document.radius()),
            ..Default::default()
        })
    }
    fn planet_camera_fraction(&self, settings: &Settings) -> Result<f64> {
        let world = self
            .simulation
            .planet
            .as_ref()
            .ok_or_else(|| invalid("Planet runtime required"))?;
        if settings.distance_meters > 125. {
            return Ok(1.);
        }
        let eye = self
            .player_view
            .as_ref()
            .ok_or_else(|| invalid("Player view required"))?
            .config
            .eye_height;
        let pivot = glam::Vec3::Y * eye;
        let desired = pivot
            + crate::planet::orbit::desired_offset(&world.config.document, settings).as_vec3();
        let (fraction, _) = self.simulation.camera_fraction(
            &world.config.actor,
            pivot.to_array(),
            desired.to_array(),
            0.15,
        );
        Ok(f64::from(
            (fraction - 0.01 / (settings.distance_meters as f32).max(0.01)).clamp(0., 1.),
        ))
    }
    pub(super) fn set_planet_view(&mut self, settings: Option<Settings>) -> Result<()> {
        let Some(mut settings) = settings else {
            self.planet_view = None;
            self.view_revision += 1;
            return Ok(());
        };
        // Wrap before the f32 locomotion adapter consumes yaw; a finite f64 angle
        // can otherwise overflow that adapter even though camera sin/cos is valid.
        settings.yaw_degrees = settings.yaw_degrees.rem_euclid(360.);
        settings.validate(
            &self
                .simulation
                .planet
                .as_ref()
                .ok_or_else(|| invalid("Planet runtime required"))?
                .config
                .document,
        )?;
        let fraction = self.planet_camera_fraction(&settings)?;
        let world = self
            .simulation
            .planet
            .as_ref()
            .ok_or_else(|| invalid("Planet runtime required"))?;
        let eye = f64::from(self.player_view.as_ref().unwrap().config.eye_height);
        if let Some(view) = &mut self.planet_view {
            let old = std::mem::replace(&mut view.settings, settings);
            if let Err(e) = view.update(
                &world.config.document,
                &world.config,
                world.anchor,
                eye,
                fraction,
            ) {
                view.settings = old;
                return Err(e);
            }
        } else {
            let entity = &self.scene.entities[0];
            self.planet_view = Some(Atlas::new(
                &world.config,
                settings,
                world.anchor,
                entity.transform.size,
                entity.color,
                eye,
                fraction,
            )?);
        }
        self.view_revision += 1;
        Ok(())
    }
    pub(crate) fn prepare_planet_view(&mut self) -> Result<()> {
        if let Some(settings) = self.planet_view.as_ref().map(|v| v.settings.clone()) {
            let fraction = self.planet_camera_fraction(&settings)?;
            let world = self.simulation.planet.as_ref().expect("planet view source");
            let eye = f64::from(self.player_view.as_ref().unwrap().config.eye_height);
            self.planet_view.as_mut().unwrap().update(
                &world.config.document,
                &world.config,
                world.anchor,
                eye,
                fraction,
            )?;
        }
        Ok(())
    }
    pub(crate) fn prepare_planet_viewport(&mut self, width: u32, height: u32) -> Result<()> {
        if let Some(view) = &self.planet_view
            && view.settings.viewport != [width, height]
        {
            let mut settings = view.settings.clone();
            settings.viewport = [width, height];
            self.set_planet_view(Some(settings))?;
        }
        Ok(())
    }
}
