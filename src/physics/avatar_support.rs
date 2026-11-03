//! Presentation adapters over the current physical snapshot. No independent movement solver.
use super::*;
use glam::{Vec2, Vec3};
impl Simulation {
    pub(crate) fn avatar_contact(
        &self,
        owner: &str,
        point: Vec3,
        up: f32,
        down: f32,
    ) -> Option<(Vec3, Vec3)> {
        let enabled = |_: ColliderHandle, c: &Collider| c.is_enabled();
        let query = self.broad.as_query_pipeline(
            self.narrow.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            QueryFilter::default()
                .exclude_rigid_body(*self.handles.get(owner)?)
                .exclude_sensors()
                .predicate(&enabled),
        );
        let start = point + Vec3::Y * up;
        let ray = Ray::new(point![start.x, start.y, start.z], vector![0., -1., 0.]);
        let (_, hit) = query.cast_ray_and_get_normal(&ray, up + down, false)?;
        let position = start - Vec3::Y * hit.time_of_impact;
        Some((position, Vec3::from_array(hit.normal.into())))
    }
    pub(crate) fn correct_avatar_pose(
        &self,
        id: &str,
        config: &crate::npc_presentation::Config,
        pose: &mut crate::npc_presentation::Pose,
    ) {
        if config.ik.is_empty() {
            return;
        }
        let state = &self.presentation.actors[id];
        use crate::npc_presentation::Phase;
        let airborne = self
            .character_controllers
            .get(id)
            .is_some_and(|c| c.airborne)
            || self.mobility.states.get(id).is_some_and(|m| {
                matches!(
                    m.motor_mode,
                    Some(
                        crate::mobility::Mode::Fly
                            | crate::mobility::Mode::Jump
                            | crate::mobility::Mode::Glide
                    )
                )
            });
        if !matches!(state.phase, Phase::Idle | Phase::Walk | Phase::Cast) || airborne {
            return;
        }
        let asset = &self.scene.meshes.assets[pose.asset as usize];
        let Some(rig) = &asset.rig else {
            return;
        };
        let Some(anchor) = self.position(id) else {
            return;
        };
        let world = crate::meshes::avatar_world(pose, anchor);
        let inverse = world.inverse();
        let mut local = crate::meshes::avatar_locals(asset, pose);
        for chain in &config.ik {
            let min_up = chain.max_slope_degrees.to_radians().cos();
            let globals = crate::avatar_ik::globals(rig, &local);
            let tip = world
                .transform_point3(globals[chain.bones[2] as usize].transform_point3(Vec3::ZERO));
            let Some((point, normal)) =
                self.avatar_contact(id, tip, chain.probe_up, chain.probe_down)
            else {
                continue;
            };
            if chain.support_radius > 0. {
                let r = chain.support_radius;
                let supported = [Vec3::X * r, -Vec3::X * r, Vec3::Z * r, -Vec3::Z * r]
                    .into_iter()
                    .all(|offset| {
                        self.avatar_contact(id, tip + offset, chain.probe_up, chain.probe_down)
                            .is_some_and(|(p, n)| {
                                n.y >= min_up
                                    && (normal.dot(p - point)).abs() <= chain.support_tolerance
                            })
                    });
                if !supported {
                    continue;
                }
            }
            let target = point + normal * chain.sole_offset;
            if normal.y < min_up || tip.distance(target) > chain.max_correction {
                continue;
            }
            if crate::avatar_ik::solve(
                rig,
                &mut local,
                chain,
                inverse.transform_point3(target),
                inverse.transform_vector3(normal).normalize(),
            ) {
                let solved = crate::avatar_ik::globals(rig, &local);
                let reached = world
                    .transform_point3(solved[chain.bones[2] as usize].transform_point3(Vec3::ZERO));
                if reached.distance(target) > 0.01 {
                    pose.ik_limited += 1;
                }
                for &bone in &chain.bones {
                    let (_, rotation, translation) =
                        local[bone as usize].to_scale_rotation_translation();
                    pose.joints[pose.joint_count as usize] = crate::avatar_ik::Joint {
                        bone,
                        translation: translation.to_array(),
                        rotation: rotation.to_array(),
                    };
                    pose.joint_count += 1;
                }
            }
        }
    }
    /// Apply animation distance before circulation/interrupt/KCC, never after collisions.
    pub(crate) fn apply_avatar_motion(&mut self) {
        if !self.avatar_motion_enabled {
            return;
        }
        let mut motions = std::mem::take(&mut self.presentation.motion);
        motions.retain(|id, m| self.reference_for_presentation(id) == Some((m.instance, m.life)));
        for (id, npc) in &self.npcs {
            let Some(controller) = self.character_controllers.get_mut(id) else {
                motions.remove(id);
                continue;
            };
            let entity = &self.scene.entities[controller.entity_index as usize];
            let Some(config) = entity.avatar.as_ref() else {
                motions.remove(id);
                continue;
            };
            let Some(root) = config.root_motion else {
                motions.remove(id);
                continue;
            };
            let slot = self
                .scene
                .reference(id)
                .and_then(|r| self.actors.actor_slot(r))
                .map(|a| a as u16);
            let Some(slot) = slot else {
                motions.remove(id);
                continue;
            };
            let instance = self.actors.reference(slot);
            let life = self.vitality.state.lives[slot as usize];
            let intended = Vec2::from(controller.requested_velocity);
            if self.vitality.dead(slot as usize)
                || !controller.active
                || controller.airborne
                || intended.length_squared() < 0.000001
            {
                motions.remove(id);
                continue;
            }
            let asset = self
                .scene
                .meshes
                .assets
                .iter()
                .find(|a| a.id == config.asset)
                .unwrap();
            let continuing = motions.get(id).filter(|m| {
                m.instance == instance
                    && m.life == life
                    && m.last_tick.checked_add(1) == Some(self.tick)
            });
            let (clip, started) = if let Some(m) = continuing {
                (m.clip, m.started)
            } else {
                let traits = npc
                    .mind_profile
                    .or_else(|| {
                        entity
                            .npc
                            .as_ref()
                            .and_then(|n| n.mind)
                            .map(|m| m.resolve(instance.instance))
                    })
                    .map_or([5000; 9], |p| p.traits);
                let variant = crate::npc_presentation::select_variant(
                    config,
                    crate::npc_presentation::Phase::Walk,
                    traits,
                    instance,
                    life,
                )
                .unwrap();
                let clip = asset
                    .clips
                    .iter()
                    .position(|c| Some(&c.id) == config.variants[variant as usize].clip.as_ref())
                    .unwrap() as u8;
                (clip, self.tick)
            };
            let command = crate::avatar_motion::velocity(
                root,
                asset,
                clip,
                self.tick - started,
                intended,
                config.scale,
            );
            controller.requested_velocity = command.to_array();
            let body = &mut self.bodies[self.handles[id]];
            let y = body.linvel().y;
            body.set_linvel(vector![command.x, y, command.y], true);
            if let Some(v) = self.velocities.get_mut(id) {
                *v = command.to_array();
            }
            let motion = crate::avatar_motion::Motion {
                instance,
                life,
                clip,
                started,
                last_tick: self.tick,
                intended: intended.to_array(),
                command: command.to_array(),
            };
            if let Some(m) = motions.get_mut(id) {
                *m = motion;
            } else {
                motions.insert(id.clone(), motion);
            }
        }
        self.presentation.motion = motions;
    }
}
