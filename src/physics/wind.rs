//! Aerodynamic bodies share the ordinary contact world, not the actor catalog.
use super::*;
use crate::{
    model::{Result, invalid},
    wind as w,
};
use glam::{DMat3, DQuat, DVec3};

pub(crate) struct AeroBody {
    pub instance: usize,
    pub id: String,
    pub handle: RigidBodyHandle,
    mass: w::Mass,
    inverse_local_inertia: DMat3,
    samples: Vec<w::SurfacePoint>,
    air: Vec<DVec3>,
    coefficients: [f64; 3],
    force: DVec3,
    torque: DVec3,
}
#[derive(Default)]
pub(crate) struct Runtime {
    pub field: w::CompiledField,
    pub bodies: Vec<AeroBody>,
    pub samples_per_substep: usize,
    kinematics: Vec<(RigidBodyHandle, Isometry<f32>, Isometry<f32>)>,
    load_tick: Option<u64>,
}
pub(crate) use crate::wind::BodyState as Primary;
impl Simulation {
    pub(crate) fn prepare_wind_kinematics(&mut self, substeps: u8) {
        self.wind.kinematics.clear();
        for b in &mut self.wind.bodies {
            b.force = DVec3::ZERO;
            b.torque = DVec3::ZERO;
        }
        self.wind.load_tick = Some(self.tick + 1);
        if substeps == 1 {
            return;
        }
        for (h, b) in self.bodies.iter() {
            if b.is_kinematic() {
                self.wind
                    .kinematics
                    .push((h, *b.position(), *b.next_position()));
            }
        }
    }
    pub(crate) fn advance_wind_kinematics(&mut self, substep: u8, total: u8) {
        let t = f32::from(substep + 1) / f32::from(total);
        for (h, start, end) in &self.wind.kinematics {
            let position = Isometry::from_parts(
                Translation::from(start.translation.vector * (1. - t) + end.translation.vector * t),
                start.rotation.slerp(&end.rotation, t),
            );
            self.bodies[*h].set_next_kinematic_position(position);
        }
    }
    pub(crate) fn aerodynamic_pose(&self, handle: RigidBodyHandle) -> ([f32; 3], [f32; 4]) {
        let b = &self.bodies[handle];
        ((*b.translation()).into(), b.rotation().coords.into())
    }
    pub(crate) fn insert_aerodynamics(&mut self) {
        let document = &self.scene.wind;
        self.wind = Runtime {
            field: w::CompiledField::new(&document.field).expect("validated wind"),
            ..Default::default()
        };
        for config in &document.bodies {
            let index = self
                .scene
                .meshes
                .instances
                .iter()
                .position(|i| i.id == config.instance)
                .expect("validated binding");
            let instance = &self.scene.meshes.instances[index];
            let (mass, samples, hull) =
                w::compile_surfaces(config, f64::from(instance.scale)).expect("validated surfaces");
            let inertia = rapier3d::na::Matrix3::from_column_slice(
                &mass.inertia.to_cols_array().map(|x| x as f32),
            );
            let properties = MassProperties::with_inertia_matrix(
                point![
                    mass.center.x as f32,
                    mass.center.y as f32,
                    mass.center.z as f32
                ],
                mass.mass as f32,
                inertia,
            );
            let q = instance.rotation;
            let position = Isometry::from_parts(
                Translation::from(Vector::from(instance.position)),
                rapier3d::na::UnitQuaternion::new_normalize(rapier3d::na::Quaternion::new(
                    q[3], q[0], q[1], q[2],
                )),
            );
            let handle = self.bodies.insert(
                RigidBodyBuilder::dynamic()
                    .pose(position)
                    .additional_mass_properties(properties)
                    .linvel(Vector::from(config.velocity))
                    .angvel(Vector::from(config.angular_velocity))
                    .ccd_enabled(true)
                    .can_sleep(false)
                    .gyroscopic_forces_enabled(true),
            );
            let points: Vec<_> = hull.into_iter().map(Point::from).collect();
            let collider = ColliderBuilder::convex_hull(&points)
                .expect("positive thickness hull")
                .density(0.)
                .friction(0.6)
                .restitution(config.restitution)
                .build();
            let c = self
                .colliders
                .insert_with_parent(collider, handle, &mut self.bodies);
            self.bodies[handle].recompute_mass_properties_from_colliders(&self.colliders);
            self.broad.set_aabb(
                &IntegrationParameters::default(),
                c,
                self.colliders[c].compute_aabb(),
            );
            self.wind.samples_per_substep += samples.len();
            let air = vec![DVec3::ZERO; samples.len()];
            self.wind.bodies.push(AeroBody {
                instance: index,
                id: instance.id.clone(),
                handle,
                mass,
                inverse_local_inertia: mass.inertia.inverse(),
                samples,
                air,
                coefficients: [
                    config.normal_drag,
                    config.tangent_drag[0],
                    config.tangent_drag[1],
                ],
                force: DVec3::ZERO,
                torque: DVec3::ZERO,
            });
        }
    }
    pub(crate) fn apply_aerodynamics(&mut self, seconds: f64, dt: f64) {
        let density = self.scene.wind.air_density;
        let field = self.wind.field.at_time(seconds);
        for compiled in &mut self.wind.bodies {
            let body = &mut self.bodies[compiled.handle];
            let q = DQuat::from_array(<[f32; 4]>::from(body.rotation().coords).map(f64::from))
                .normalize();
            let rotation = DMat3::from_quat(q);
            let inverse_inertia = rotation * compiled.inverse_local_inertia * rotation.transpose();
            let origin = DVec3::from_array(<[f32; 3]>::from(*body.translation()).map(f64::from));
            let mut velocity = DVec3::from_array(<[f32; 3]>::from(*body.linvel()).map(f64::from));
            let mut omega = DVec3::from_array(<[f32; 3]>::from(*body.angvel()).map(f64::from));
            let before_velocity = velocity;
            let before_omega = omega;
            for (s, air) in compiled.samples.iter().zip(&mut compiled.air) {
                *air = field.sample(origin + rotation * s.position);
            }
            // Palindromic rank-one splitting. Both sweeps use the same physical
            // half step, point air and orientation; no random displacement.
            for reverse in [false, true] {
                for j in 0..compiled.samples.len() {
                    let index = if reverse {
                        compiled.samples.len() - 1 - j
                    } else {
                        j
                    };
                    let s = &compiled.samples[index];
                    let offset = rotation * (s.position - compiled.mass.center);
                    let air = compiled.air[index];
                    for k in 0..3 {
                        let k = if reverse { 2 - k } else { k };
                        w::implicit_drag(
                            &mut velocity,
                            &mut omega,
                            1. / compiled.mass.mass,
                            inverse_inertia,
                            offset,
                            rotation * s.axes[k],
                            air,
                            0.25 * density * s.area * compiled.coefficients[k] * dt,
                        );
                    }
                }
            }
            body.set_linvel(Vector::from(velocity.as_vec3().to_array()), true);
            body.set_angvel(Vector::from(omega.as_vec3().to_array()), true);
            compiled.force +=
                (velocity - before_velocity) * (compiled.mass.mass / f64::from(FIXED_DT));
            compiled.torque += (rotation * compiled.mass.inertia * rotation.transpose())
                * (omega - before_omega)
                / f64::from(FIXED_DT);
        }
    }
    pub(crate) fn wind_handles(&self) -> Vec<RigidBodyHandle> {
        self.wind.bodies.iter().map(|b| b.handle).collect()
    }
    pub(crate) fn wind_primaries(&self) -> Vec<Primary> {
        self.wind
            .bodies
            .iter()
            .map(|a| {
                let b = &self.bodies[a.handle];
                Primary {
                    id: a.id.clone(),
                    position: (*b.translation()).into(),
                    rotation: b.rotation().coords.into(),
                    velocity: (*b.linvel()).into(),
                    angular_velocity: (*b.angvel()).into(),
                }
            })
            .collect()
    }
    pub(crate) fn restore_wind_primaries(&mut self, values: Vec<Primary>) -> Result<()> {
        if values.len() != self.wind.bodies.len() {
            return Err(invalid("Saved aerodynamic body coverage"));
        }
        for (value, compiled) in values.into_iter().zip(&self.wind.bodies) {
            if value.id != compiled.id
                || value
                    .position
                    .iter()
                    .chain(&value.rotation)
                    .chain(&value.velocity)
                    .chain(&value.angular_velocity)
                    .any(|x| !x.is_finite())
                || (glam::Quat::from_array(value.rotation).length_squared() - 1.).abs() > 1e-4
            {
                return Err(invalid("Saved aerodynamic body identity or finite state"));
            }
            let b = &mut self.bodies[compiled.handle];
            let q = value.rotation;
            b.set_position(
                Isometry::from_parts(
                    Translation::from(Vector::from(value.position)),
                    // Preserve validated canonical quaternion bits on restore.
                    rapier3d::na::UnitQuaternion::new_unchecked(rapier3d::na::Quaternion::new(
                        q[3], q[0], q[1], q[2],
                    )),
                ),
                true,
            );
            b.set_linvel(Vector::from(value.velocity), true);
            b.set_angvel(Vector::from(value.angular_velocity), true);
        }
        // Publish the restored collision poses before any query or first tick.
        self.bodies
            .propagate_modified_body_positions_to_colliders(&mut self.colliders);
        Ok(())
    }
    pub fn inspect_wind(&self) -> serde_json::Value {
        let loads:Vec<_>=self.wind.bodies.iter().map(|b|serde_json::json!({"instance":b.id,"force_newtons":b.force.to_array(),"torque_newton_meters":b.torque.to_array()})).collect();
        serde_json::json!({"tick":self.tick,"document":self.scene.wind,"bodies":self.wind_primaries(),
            "aerodynamic_loads_last_tick":(self.wind.load_tick==Some(self.tick)).then_some(loads),
            "samples_per_substep":self.wind.samples_per_substep,"sample_work_per_tick":self.scene.wind.work(),
            "aerodynamics":"passive_quasi_steady_thin_surfaces","fluid_feedback":false,"physics_precision":"f32","field_precision":"f64"})
    }
}

pub(crate) fn validate_saved_handles(
    author: &Scene,
    handles: &[RigidBodyHandle],
    primaries: &[Primary],
    bodies: &RigidBodySet,
    colliders: &ColliderSet,
) -> Result<std::collections::HashSet<ColliderHandle>> {
    if handles.len() != author.wind.bodies.len() || primaries.len() != handles.len() {
        return Err(invalid("Checkpoint aerodynamic handle coverage"));
    }
    let mut seen = std::collections::HashSet::new();
    for ((config, &handle), primary) in author.wind.bodies.iter().zip(handles).zip(primaries) {
        let b = bodies
            .get(handle)
            .ok_or_else(|| invalid("Checkpoint aerodynamic body missing"))?;
        if primary.id != config.instance
            || primary.position != <[f32; 3]>::from(*b.translation())
            || primary.rotation != <[f32; 4]>::from(b.rotation().coords)
            || primary.velocity != <[f32; 3]>::from(*b.linvel())
            || primary.angular_velocity != <[f32; 3]>::from(*b.angvel())
        {
            return Err(invalid(
                "Checkpoint aerodynamic primaries disagree with the native backend",
            ));
        }
        let instance = author
            .meshes
            .instances
            .iter()
            .find(|i| i.id == config.instance)
            .unwrap();
        let (mass, _, hull) = w::compile_surfaces(config, f64::from(instance.scale))?;
        let points: Vec<_> = hull.into_iter().map(Point::from).collect();
        let expected = SharedShape::convex_hull(&points)
            .ok_or_else(|| invalid("Checkpoint aerodynamic hull"))?;
        if !b.is_dynamic()
            || b.colliders().len() != 1
            || !b.is_enabled()
            || !b.is_ccd_enabled()
            || !b.gyroscopic_forces_enabled()
            || b.translation()
                .iter()
                .chain(b.linvel().iter())
                .chain(b.angvel().iter())
                .any(|x| !x.is_finite())
            || (f64::from(b.mass()) - mass.mass).abs() > mass.mass * 1e-4
        {
            return Err(invalid("Checkpoint aerodynamic body state/mass"));
        }
        let c = colliders
            .get(b.colliders()[0])
            .ok_or_else(|| invalid("Checkpoint aerodynamic collider"))?;
        let expected = expected.as_convex_polyhedron().unwrap();
        let actual_mass = &b.mass_properties().local_mprops;
        let actual_inertia = actual_mass.reconstruct_inertia_matrix();
        let expected_inertia = rapier3d::na::Matrix3::from_column_slice(
            &mass.inertia.to_cols_array().map(|x| x as f32),
        );
        let expected_center = Point::from(mass.center.as_vec3().to_array());
        if (actual_mass.local_com - expected_center).norm() > 1e-5
            || (actual_inertia - expected_inertia).norm() > expected_inertia.norm() * 1e-4
            || b.mass_properties().flags != LockedAxes::empty()
            || b.gravity_scale() != 1.
            || (b.rotation().quaternion().norm_squared() - 1.).abs() > 1e-4
        {
            return Err(invalid(
                "Checkpoint aerodynamic inertia, center, rotation or constraints",
            ));
        }
        if !seen.insert(b.colliders()[0])
            || c.parent() != Some(handle)
            || c.is_sensor()
            || !c.is_enabled()
            || c.position_wrt_parent() != Some(&Isometry::identity())
            || c.position() != b.position()
            || c.friction() != 0.6
            || c.restitution() != config.restitution
            || c.shape()
                .as_convex_polyhedron()
                .is_none_or(|s| s.points() != expected.points())
        {
            return Err(invalid("Checkpoint aerodynamic collision shape"));
        }
    }
    Ok(seen)
}
