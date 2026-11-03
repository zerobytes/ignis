//! Engineering experiment, not an Ignis scene/controller or gameplay API.
//! Run: cargo run --locked --release --example locomotion_probe
use rapier3d::{
    control::{CharacterAutostep, CharacterLength, KinematicCharacterController},
    parry::query::contact,
    prelude::*,
};
use serde::Serialize;

const DT: f32 = ignis::model::FIXED_DT;
const TICKS: usize = 240;
const SPEED: f32 = 2.;
const HALF: [f32; 3] = [0.3, 0.8, 0.3];

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Motor {
    DynamicVelocity,
    Kinematic,
    KinematicWithImpulses,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Case {
    Flat,
    Ramp20Up,
    Ramp40Up,
    Ramp20Down,
    Step20,
    Step50,
    Step20LowCeiling,
    Wall,
    Ledge,
    PushCrate,
    ExternalImpulse,
}

const CASES: [Case; 11] = [
    Case::Flat,
    Case::Ramp20Up,
    Case::Ramp40Up,
    Case::Ramp20Down,
    Case::Step20,
    Case::Step50,
    Case::Step20LowCeiling,
    Case::Wall,
    Case::Ledge,
    Case::PushCrate,
    Case::ExternalImpulse,
];
const MOTORS: [Motor; 3] = [
    Motor::DynamicVelocity,
    Motor::Kinematic,
    Motor::KinematicWithImpulses,
];

#[derive(Default)]
struct World {
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad: BroadPhaseBvh,
    narrow: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse: ImpulseJointSet,
    multibody: MultibodyJointSet,
    ccd: CCDSolver,
}

impl World {
    fn step(&mut self) {
        self.pipeline.step(
            &vector![0., -9.81, 0.],
            &IntegrationParameters {
                dt: DT,
                ..Default::default()
            },
            &mut self.islands,
            &mut self.broad,
            &mut self.narrow,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse,
            &mut self.multibody,
            &mut self.ccd,
            &(),
            &(),
        );
    }

    fn fixed_box(&mut self, center: [f32; 3], half: [f32; 3]) {
        self.colliders.insert(
            ColliderBuilder::cuboid(half[0], half[1], half[2])
                .translation(center.into())
                .friction(0.7)
                .build(),
        );
    }

    fn ramp(&mut self, degrees: f32) -> f32 {
        let height = 4. * degrees.to_radians().tan();
        let vertices = [
            point![0., 0., -2.],
            point![0., 0., 2.],
            point![4., 0., -2.],
            point![4., 0., 2.],
            point![4., height, -2.],
            point![4., height, 2.],
        ];
        self.colliders.insert(
            ColliderBuilder::convex_hull(&vertices)
                .unwrap()
                .friction(0.7),
        );
        self.fixed_box([7., height / 2., 0.], [3., height / 2., 2.]);
        height
    }
}

#[derive(Debug, Serialize, PartialEq)]
struct Sample {
    tick: usize,
    center: [f32; 3],
    crate_center: Option<[f32; 3]>,
}

#[derive(Debug, Serialize, PartialEq)]
struct Outcome {
    motor: Motor,
    start: [f32; 3],
    end: [f32; 3],
    min_y: f32,
    max_y: f32,
    distance: f32,
    max_penetration: f32,
    grounded_ticks: Option<usize>,
    controller_collision_callbacks: usize,
    crate_displacement_x: Option<f32>,
    trajectory_hash: String,
    samples: Vec<Sample>,
}

fn run(case: Case, motor: Motor) -> Outcome {
    let mut w = World::default();
    let mut start = [-1.5, 0.81, 0.];
    let mut direction = 1.;
    w.fixed_box([0., -0.25, 0.], [20., 0.25, 4.]);
    match case {
        Case::Ramp20Up | Case::Ramp20Down => {
            let height = w.ramp(20.);
            if matches!(case, Case::Ramp20Down) {
                start = [5.5, height + 0.81, 0.];
                direction = -1.;
            }
        }
        Case::Ramp40Up => {
            w.ramp(40.);
        }
        Case::Step20 | Case::Step50 | Case::Step20LowCeiling => {
            let height = if matches!(case, Case::Step50) {
                0.5
            } else {
                0.2
            };
            w.fixed_box([4., height / 2., 0.], [4., height / 2., 2.]);
            if matches!(case, Case::Step20LowCeiling) {
                w.fixed_box([2., 1.95, 0.], [3., 0.2, 2.]);
            }
        }
        Case::Wall => w.fixed_box([0.25, 1.5, 0.], [0.25, 1.5, 2.]),
        Case::Ledge => {
            w.fixed_box([-2., 0.5, 0.], [3., 0.5, 2.]);
            start[1] += 1.;
        }
        Case::ExternalImpulse => direction = 0.,
        Case::Flat | Case::PushCrate => {}
    }
    let crate_handle = if matches!(case, Case::PushCrate) {
        let handle = w
            .bodies
            .insert(RigidBodyBuilder::dynamic().translation(vector![1., 0.5, 0.]));
        w.colliders.insert_with_parent(
            ColliderBuilder::cuboid(0.5, 0.5, 0.5).friction(0.7),
            handle,
            &mut w.bodies,
        );
        Some(handle)
    } else {
        None
    };
    let builder = if motor == Motor::DynamicVelocity {
        RigidBodyBuilder::dynamic()
            .lock_rotations()
            .ccd_enabled(true)
    } else {
        RigidBodyBuilder::kinematic_position_based()
    };
    let actor = w.bodies.insert(builder.translation(start.into()));
    let shape = SharedShape::cuboid(HALF[0], HALF[1], HALF[2]);
    let collider = w.colliders.insert_with_parent(
        ColliderBuilder::new(shape.clone())
            .friction(0.)
            .restitution(0.),
        actor,
        &mut w.bodies,
    );
    // Publish initial geometry to the broad phase before making any shape query.
    w.step();
    let controller = KinematicCharacterController {
        offset: CharacterLength::Absolute(0.01),
        max_slope_climb_angle: 30_f32.to_radians(),
        min_slope_slide_angle: 35_f32.to_radians(),
        autostep: Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(0.3),
            min_width: CharacterLength::Absolute(0.4),
            include_dynamic_bodies: false,
        }),
        snap_to_ground: Some(CharacterLength::Absolute(0.15)),
        ..Default::default()
    };
    let mut vertical_velocity = 0.;
    let mut grounded = 0;
    let mut collisions = Vec::with_capacity(32);
    let mut collision_count = 0;
    let mut max_penetration: f32 = 0.;
    let mut min_y = start[1];
    let mut max_y = start[1];
    let mut distance = 0.;
    let mut previous = *w.bodies[actor].translation();
    let mut hash = 14695981039346656037_u64;
    let mut samples = Vec::with_capacity(TICKS / 30);
    for tick in 0..TICKS {
        if motor == Motor::DynamicVelocity {
            let y = w.bodies[actor].linvel().y;
            w.bodies[actor].set_linvel(vector![SPEED * direction, y, 0.], true);
        } else {
            vertical_velocity -= 9.81 * DT;
            collisions.clear();
            let filter = QueryFilter::default().exclude_rigid_body(actor);
            let movement = controller.move_shape(
                DT,
                &w.broad.as_query_pipeline(
                    w.narrow.query_dispatcher(),
                    &w.bodies,
                    &w.colliders,
                    filter,
                ),
                shape.as_ref(),
                w.bodies[actor].position(),
                vector![SPEED * direction, vertical_velocity, 0.] * DT,
                |c| collisions.push(c),
            );
            grounded += usize::from(movement.grounded);
            if movement.grounded {
                vertical_velocity = 0.;
            }
            collision_count += collisions.len();
            if motor == Motor::KinematicWithImpulses {
                controller.solve_character_collision_impulses(
                    DT,
                    &mut w.broad.as_query_pipeline_mut(
                        w.narrow.query_dispatcher(),
                        &mut w.bodies,
                        &mut w.colliders,
                        filter,
                    ),
                    shape.as_ref(),
                    8. * HALF[0] * HALF[1] * HALF[2],
                    &collisions,
                );
            }
            let next = *w.bodies[actor].translation() + movement.translation;
            w.bodies[actor].set_next_kinematic_translation(next);
        }
        // An external event occurs after the movement command, before physics.
        // The next tick replaces horizontal velocity in the current dynamic policy.
        if matches!(case, Case::ExternalImpulse) && tick == 60 {
            w.bodies[actor].apply_impulse(vector![2., 0., 0.], true);
        }
        w.step();
        let p = *w.bodies[actor].translation();
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
        distance += (p - previous).norm();
        previous = p;
        let crate_center: Option<[f32; 3]> =
            crate_handle.map(|h| (*w.bodies[h].translation()).into());
        for value in p.iter().copied().chain(crate_center.into_iter().flatten()) {
            for byte in value.to_bits().to_le_bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(1099511628211);
            }
        }
        for (handle, other) in w.colliders.iter() {
            if handle == collider {
                continue;
            }
            if let Some(c) = contact(
                w.bodies[actor].position(),
                shape.as_ref(),
                other.position(),
                other.shape(),
                0.,
            )
            .unwrap()
            {
                max_penetration = max_penetration.max(-c.dist);
            }
        }
        if (tick + 1) % 30 == 0 {
            samples.push(Sample {
                tick: tick + 1,
                center: p.into(),
                crate_center,
            });
        }
    }
    Outcome {
        motor,
        start,
        end: (*w.bodies[actor].translation()).into(),
        min_y,
        max_y,
        distance,
        max_penetration,
        grounded_ticks: (motor != Motor::DynamicVelocity).then_some(grounded),
        controller_collision_callbacks: collision_count,
        crate_displacement_x: crate_handle.map(|h| w.bodies[h].translation().x - 1.),
        trajectory_hash: format!("{hash:016x}"),
        samples,
    }
}

#[derive(Serialize)]
struct Comparison {
    case: Case,
    outcomes: Vec<Outcome>,
    repeated_identically: bool,
}

fn validate(case: Case, o: &Outcome) {
    assert!(o.end.iter().all(|v| v.is_finite()), "{case:?}: {o:?}");
    assert!(o.max_penetration < 0.02, "{case:?}: {o:?}");
    let dynamic = o.motor == Motor::DynamicVelocity;
    let blocked = o.end[0] < -0.25 && o.end[0] > -0.4;
    let floor = (o.end[1] - HALF[1]).abs() < 0.03;
    let passed = match case {
        Case::Flat => o.end[0] > 6.2 && floor,
        Case::Ramp20Up => {
            o.end[0] > 4.5 && (o.end[1] - (HALF[1] + 4. * 20_f32.to_radians().tan())).abs() < 0.03
        }
        Case::Ramp40Up if dynamic => o.end[0] > 4.5,
        Case::Ramp40Up => blocked && floor,
        Case::Ramp20Down => o.end[0] < -2. && floor,
        Case::Step20 if !dynamic => o.end[0] > 6.2 && (o.end[1] - 1.).abs() < 0.03,
        Case::Step20 | Case::Step50 | Case::Step20LowCeiling | Case::Wall => blocked && floor,
        Case::Ledge => {
            o.end[0] > 6.2 && floor && (dynamic || o.grounded_ticks.unwrap() < TICKS - 10)
        }
        Case::PushCrate if o.motor == Motor::Kinematic => {
            o.end[0] < 0.3 && o.crate_displacement_x.unwrap().abs() < 0.01
        }
        Case::PushCrate => o.end[0] > 2. && o.crate_displacement_x.unwrap() > 1.,
        Case::ExternalImpulse if dynamic => {
            o.end[0] - o.start[0] > 0.02 && (o.samples[2].center[0] - o.end[0]).abs() < 0.001
        }
        Case::ExternalImpulse => (o.end[0] - o.start[0]).abs() < 0.001,
    };
    assert!(passed, "{case:?}: {o:?}");
}

fn comparisons() -> Vec<Comparison> {
    CASES
        .into_iter()
        .map(|case| {
            let outcomes: Vec<_> = MOTORS.into_iter().map(|motor| run(case, motor)).collect();
            let repeated_identically = outcomes.iter().all(|o| *o == run(case, o.motor));
            Comparison {
                case,
                outcomes,
                repeated_identically,
            }
        })
        .collect()
}

fn main() {
    let cases = comparisons();
    for c in &cases {
        for o in &c.outcomes {
            validate(c.case, o);
        }
    }
    let source_hash = include_bytes!("locomotion_probe.rs")
        .iter()
        .fold(14695981039346656037_u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(1099511628211)
        });
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "scope": "isolated_physics_experiment_not_engine_integration",
            "engine_build": env!("IGNIS_BUILD_ID"),
            "probe_source_hash": format!("{source_hash:016x}"),
            "dt": DT, "ticks": TICKS, "speed": SPEED, "actor_half_extents": HALF,
        "initial_physics_steps": 1,
            "max_slope_degrees": 30, "min_slide_degrees": 35,
            "step_height": 0.3, "step_min_width": 0.4, "snap": 0.15, "offset": 0.01,
            "cases": cases,
        }))
        .unwrap()
    );
    assert!(
        cases.iter().all(|c| c.repeated_identically),
        "non-repeatable trajectory"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_outcomes_and_repeatability() {
        for comparison in comparisons() {
            assert!(comparison.repeated_identically, "{:?}", comparison.case);
            for outcome in &comparison.outcomes {
                validate(comparison.case, outcome);
            }
        }
    }
}
