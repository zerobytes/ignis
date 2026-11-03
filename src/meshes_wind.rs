//! Rigid pose overrides preserve authored data and static forest caches.
use super::*;
impl Frame {
    pub(crate) fn update_aerodynamic<'a>(
        &mut self,
        document: &Document,
        simulation: &crate::physics::Simulation,
        consumed: impl Iterator<Item = &'a str> + Clone,
    ) {
        let bodies = &simulation.wind.bodies;
        if !self
            .wind_residency
            .iter()
            .map(String::as_str)
            .eq(bodies.iter().map(|b| b.id.as_str()))
        {
            self.wind_residency.clear();
            self.wind_residency
                .extend(bodies.iter().map(|b| b.id.clone()));
            self.wind_draws.clear();
            self.tick = None;
        }
        self.update_filtered(document, simulation.tick, consumed);
        if bodies.is_empty() {
            return;
        }
        if self.wind_draws.len() != bodies.len()
            || bodies
                .iter()
                .zip(&self.wind_draws)
                .any(|(b, &index)| self.draws.get(index).is_none_or(|d| d.instance != b.id))
        {
            self.wind_draws.clear();
            self.wind_draws.extend(bodies.iter().map(|b| {
                self.draws
                    .iter()
                    .position(|d| d.instance == b.id)
                    .expect("validated independent aerodynamic draw")
            }));
        }
        let mut changed = false;
        for (b, &index) in bodies.iter().zip(&self.wind_draws) {
            let instance = if document
                .instances
                .get(b.instance)
                .is_some_and(|i| i.id == b.id)
            {
                &document.instances[b.instance]
            } else {
                document
                    .instances
                    .iter()
                    .find(|i| i.id == b.id)
                    .expect("validated aerodynamic overlay")
            };
            let (position, rotation) = simulation.aerodynamic_pose(b.handle);
            let world = Mat4::from_scale_rotation_translation(
                Vec3::splat(instance.scale),
                Quat::from_array(rotation),
                Vec3::from(position),
            );
            let draw = &mut self.draws[index];
            assert!(draw.shared, "aerodynamic geometry is resident");
            if draw.world != world {
                draw.world = world;
                changed = true;
            }
        }
        if changed {
            self.generation = next_frame_generation();
            // These bindings cannot belong to composite objects. Their motion
            // changes visibility/instance transforms, not geometry or object bounds.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{Command, Engine, SceneEdit},
        model::RoomRecipe,
    };
    use serde_json::json;
    #[test]
    fn aerodynamic_pose_overrides_keep_static_geometry_and_object_bounds() {
        let mut scene = RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let geometry =
            json!({"positions":[[0.,0.,0.],[1.,0.,0.],[0.,0.,1.]],"triangles":[[0,1,2]]});
        let placement = |id: &str, asset: &str, x: f32, y: f32| json!({"id":id,"asset":asset,"position":[x,y,0.],"rotation":[0.,0.,0.,1.],"scale":1.,"color":[0.5,0.6,0.1],"playback":null});
        scene.meshes=serde_json::from_value(json!({"assets":[{"id":"leaf-asset","geometry":geometry},{"id":"background","geometry":geometry}],
            "instances":[placement("leaf","leaf-asset",0.,2.),placement("back-a","background",4.,0.),placement("back-b","background",6.,0.)],
            "objects":[{"id":"static-object","members":["back-a","back-b"]}]})).unwrap();
        scene.wind=Arc::new(serde_json::from_value(json!({"bodies":[{"instance":"leaf","sheet_mass_kg":0.001,"panels":[{"vertices":[[0.,0.,0.],[1.,0.,0.],[0.,0.,1.]]}]}]})).unwrap());
        let mut engine = Engine::new(scene).unwrap();
        let mut frame = engine.render_frame();
        assert!(
            frame.meshes.vertices.is_empty(),
            "even a unique aerodynamic mesh stays resident"
        );
        let geometry_generation = frame.meshes.geometry_generation;
        let object_generation = frame.meshes.static_generation;
        let background: Vec<_> = frame
            .meshes
            .draws
            .iter()
            .filter(|d| d.instance.starts_with("back-"))
            .map(|d| d.world)
            .collect();
        engine.advance(2).unwrap();
        engine.update_render_frame(&mut frame);
        assert_eq!(frame.meshes.geometry_generation, geometry_generation);
        assert_eq!(frame.meshes.static_generation, object_generation);
        assert_eq!(frame.meshes.source, engine.scene.meshes);
        assert_eq!(
            frame
                .meshes
                .draws
                .iter()
                .filter(|d| d.instance.starts_with("back-"))
                .map(|d| d.world)
                .collect::<Vec<_>>(),
            background
        );
        assert!(
            frame
                .meshes
                .draws
                .iter()
                .find(|d| d.instance == "leaf")
                .unwrap()
                .world
                .w_axis
                .y
                < 2.
        );
        engine
            .execute(
                Command::EditScene {
                    edits: vec![SceneEdit::SetWind {
                        document: Arc::new(Default::default()),
                    }],
                }
                .into(),
            )
            .unwrap();
        engine.update_render_frame(&mut frame);
        assert_eq!(
            frame.meshes.vertices.len(),
            3,
            "removing the component restores the ordinary unique draw path"
        );
    }
}
