use glam::{DMat3, DVec3};
use ignis::{
    engine::{Command, Engine, SceneEdit},
    model::{RoomRecipe, Scene},
    wind::*,
};
use serde_json::json;
use std::sync::Arc;

fn sheet() -> Body {
    serde_json::from_value(json!({"instance":"leaf","sheet_mass_kg":0.0004,
        "panels":[{"vertices":[[-0.12,0.,0.],[0.,0.,0.06],[0.12,0.,0.]]},
                  {"vertices":[[-0.12,0.,0.],[0.12,0.,0.],[0.,0.,-0.06]]}]}))
    .unwrap()
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.meshes=serde_json::from_value(json!({"assets":[{"id":"blade","geometry":{"positions":[[-0.12,0.,0.],[0.,0.,0.06],[0.12,0.,0.],[0.,0.,-0.06]],"triangles":[[0,1,2],[0,2,3]]}}],
        "instances":[{"id":"leaf","asset":"blade","position":[0.,2.,0.],"rotation":[0.,0.,0.,1.],"scale":1.,"color":[0.4,0.7,0.1],"playback":null}]})).unwrap();
    s.wind = Arc::new(Document {
        bodies: vec![sheet()],
        ..Default::default()
    });
    s
}
fn field() -> Field {
    Field {
        mean: [0.4, 0., 0.],
        gusts: vec![Gust {
            amplitude: [0.3, 0.1, -0.2],
            period_seconds: 5.,
            phase_radians: 0.,
        }],
        turbulence: Some(Turbulence {
            seed: 93,
            modes: 24,
            rms_speed: 1.5,
            min_wavelength: 0.8,
            max_wavelength: 10.,
            advection: [1., 0., 0.],
        }),
        jets: vec![Jet {
            origin: [0.; 3],
            direction: [1., 0., 0.],
            transverse: [0., 0., 1.],
            speed: 2.,
            width: 1.,
        }],
        vortices: vec![Vortex {
            center: [0.; 3],
            axis: [0., 1., 0.],
            velocity: [0.2, 0., 0.],
            radius: 3.,
            half_height: 2.,
            peak_swirl_speed: 4.,
            updraft_speed: 2.,
            start_seconds: 0.,
            duration_seconds: Some(10.),
            fade_seconds: 1.,
        }],
    }
}

#[test]
fn analytic_field_is_solenoidal_continuous_and_deterministic() {
    let f = CompiledField::new(&field()).unwrap();
    let other = CompiledField::new(&field()).unwrap();
    let mut max_divergence: f64 = 0.;
    for t in [0., 0.1, 1., 4.999, 9.999, 10., 11.] {
        for i in 0..80 {
            let p = DVec3::new(
                (i as f64 * 1.77).sin() * 4.,
                (i as f64 * 0.37).cos() * 3.,
                (i as f64 * 0.91).sin() * 4.,
            );
            assert_eq!(f.sample(p, t), other.sample(p, t));
            let h = 1e-5;
            let mut div = 0.;
            for axis in 0..3 {
                let mut d = DVec3::ZERO;
                d[axis] = h;
                div += (f.sample(p + d, t)[axis] - f.sample(p - d, t)[axis]) / (2. * h);
            }
            max_divergence = max_divergence.max(div.abs());
        }
    }
    assert!(max_divergence < 2e-7, "divergence={max_divergence}");
    let mut isolated = field();
    isolated.mean = [0.; 3];
    isolated.gusts.clear();
    isolated.turbulence = None;
    isolated.jets.clear();
    let f = CompiledField::new(&isolated).unwrap();
    let center = DVec3::new(1., 0., 0.); // Advected center at t5.
    let boundary = center + DVec3::X * 3.;
    assert_eq!(f.sample(boundary, 5.), DVec3::ZERO);
    assert!(f.sample(boundary - DVec3::X * 1e-5, 5.).length() < 1e-8);
    assert!(f.sample(center, 1e-6).length() < 1e-10);
    let boundary = center + DVec3::new(2.4, 1.2, 0.);
    let h = 1e-4;
    let divergence = (0..3)
        .map(|axis| {
            let mut d = DVec3::ZERO;
            d[axis] = h;
            (f.sample(boundary + d, 5.)[axis] - f.sample(boundary - d, 5.)[axis]) / (2. * h)
        })
        .sum::<f64>();
    assert!(
        divergence.abs() < 1e-12,
        "compact support boundary: {divergence}"
    );
    isolated.vortices[0].updraft_speed = 0.;
    let f = CompiledField::new(&isolated).unwrap();
    let peak_radius = 3. * (1. / 3f64.sqrt()).sqrt();
    assert!((f.sample(center + DVec3::X * peak_radius, 5.).length() - 4.).abs() < 1e-12);
}

#[test]
fn lamina_mass_center_and_full_inertia_match_rectangle_integrals() {
    let mut b = sheet();
    b.sheet_mass_kg = 3.;
    b.panels = vec![
        Panel {
            vertices: [[-1., -2., 0.], [1., -2., 0.], [1., 2., 0.]],
        },
        Panel {
            vertices: [[-1., -2., 0.], [1., 2., 0.], [-1., 2., 0.]],
        },
    ];
    let (m, points, _) = compile_surfaces(&b, 1.).unwrap();
    assert!((m.mass - 3.).abs() < 1e-12 && m.center.length() < 1e-12);
    let dz = 3. * b.thickness_m.powi(2) / 12.;
    assert!((m.inertia.x_axis.x - 4. - dz).abs() < 1e-12);
    assert!((m.inertia.y_axis.y - 1. - dz).abs() < 1e-12);
    assert!((m.inertia.z_axis.z - 5.).abs() < 1e-12);
    assert!((points.iter().map(|p| p.area).sum::<f64>() - 8.).abs() < 1e-12);
    let (scaled, _, _) = compile_surfaces(&b, 3.).unwrap();
    assert!((scaled.inertia - m.inertia * 9.).abs_diff_eq(DMat3::ZERO, 1e-11));
    let shift = DVec3::new(1000., -300., 800.);
    for panel in &mut b.panels {
        for p in &mut panel.vertices {
            *p = (DVec3::from_array(*p) + shift).to_array();
        }
    }
    let (translated, _, _) = compile_surfaces(&b, 1.).unwrap();
    assert!((translated.center - shift).length() < 1e-12);
    assert!(translated.inertia.abs_diff_eq(m.inertia, 1e-12));
}

#[test]
fn implicit_point_drag_dissipates_energy_and_conserves_its_impulse_torque() {
    for dt in [1e-6, 0.01, 1., 1000.] {
        let mut v = DVec3::new(8., -2., 3.);
        let mut w = DVec3::new(2., 1., -1.);
        let before_v = v;
        let before_w = w;
        let inertia = DMat3::from_diagonal(DVec3::new(0.1, 0.2, 0.3));
        let offset = DVec3::new(0.2, 0., 0.3);
        let direction = DVec3::new(1., 2., 3.).normalize();
        let energy = |v: DVec3, w: DVec3| 0.5 * v.length_squared() + 0.5 * w.dot(inertia * w);
        let before = energy(v, w);
        let j = implicit_drag(
            &mut v,
            &mut w,
            1.,
            inertia.inverse(),
            offset,
            direction,
            DVec3::ZERO,
            dt,
        );
        assert!(energy(v, w) <= before + 1e-12);
        assert!((v - before_v - direction * j).length() < 1e-12);
        assert!((inertia * (w - before_w) - offset.cross(direction * j)).length() < 1e-12);
        let u0 = (-before_v - before_w.cross(offset)).dot(direction);
        let u1 = (-v - w.cross(offset)).dot(direction);
        assert!(u0 * u1 >= -1e-12 && u1.abs() <= u0.abs() + 1e-12);
    }
}

#[test]
fn implicit_drag_step_refinement_converges_to_exact_scalar_solution() {
    let evaluate = |n| {
        let mut v = DVec3::X * 10.;
        let mut w = DVec3::ZERO;
        for _ in 0..n {
            implicit_drag(
                &mut v,
                &mut w,
                1.,
                DMat3::IDENTITY,
                DVec3::ZERO,
                DVec3::X,
                DVec3::ZERO,
                1. / n as f64,
            );
        }
        (v.x - 10. / 11.).abs()
    };
    let a = evaluate(10);
    let b = evaluate(20);
    let c = evaluate(40);
    assert!(b < a * 0.65 && c < b * 0.65, "{a} {b} {c}");
}

#[test]
fn free_mesh_moves_rotates_collides_replays_and_restores_saves() {
    let mut upward = scene();
    Arc::make_mut(&mut upward.wind).field.mean = [0., 4., 0.];
    let mut lifted = Engine::new(upward).unwrap();
    lifted.advance(120).unwrap();
    let lift = lifted.replay().expected_wind;
    assert!(
        lift[0].position[1] > 4. && lift[0].velocity[1] > 1.,
        "updraft must physically carry the free mesh: {lift:?}"
    );
    let mut s = scene();
    let mut d = (*s.wind).clone();
    d.field.mean = [2., 0.5, 0.];
    d.bodies[0].ballast_mass_kg = 0.00003;
    d.bodies[0].ballast_position = [-0.09, 0., 0.];
    d.bodies[0].angular_velocity = [0., 0., 2.];
    s.wind = Arc::new(d);
    let mut e = Engine::new(s).unwrap();
    e.advance(120).unwrap();
    let state = e.replay().expected_wind;
    assert!(state[0].position[0] > 0.05 && state[0].rotation != [0., 0., 0., 1.]);
    assert_eq!(
        Engine::replay_at(&e.replay(), None)
            .unwrap()
            .replay()
            .expected_wind,
        state
    );
    assert_eq!(
        Engine::verify_replay(&e.replay()).unwrap()["wind_matched"],
        true
    );
    let native = ignis::checkpoint::Checkpoint::capture(&e)
        .unwrap()
        .encode()
        .unwrap();
    let portable = ignis::portable_save::PortableSave::capture(&e)
        .unwrap()
        .encode()
        .unwrap();
    let mut n = Engine::from_checkpoint(&native, None).unwrap();
    let mut p = Engine::from_portable_save(&portable, None).unwrap();
    assert_eq!(state, n.replay().expected_wind);
    assert_eq!(state, p.replay().expected_wind);
    e.advance(30).unwrap();
    n.advance(30).unwrap();
    p.advance(30).unwrap();
    assert_eq!(e.replay().expected_wind, n.replay().expected_wind);
    let a = &e.replay().expected_wind[0];
    let b = &p.replay().expected_wind[0];
    assert!(
        (DVec3::from_array(a.position.map(f64::from))
            - DVec3::from_array(b.position.map(f64::from)))
        .length()
            < 0.02
    );
    let mut calm = Engine::new(scene()).unwrap();
    calm.advance(600).unwrap();
    let state = calm.replay().expected_wind;
    assert!(
        state[0].position[1] >= -0.01 && state[0].position[1] < 0.03,
        "{state:?}"
    );
}

#[test]
fn invalid_wind_edit_is_atomic_and_undo_restores_physics() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(20).unwrap();
    let before = serde_json::to_value(e.replay().expected_wind).unwrap();
    let revision = e.revision;
    let mut d = (*e.scene.wind).clone();
    d.bodies[0].panels[0].vertices = [[0.; 3]; 3];
    assert!(
        e.execute(
            Command::Wind {
                operation: Box::new(Operation::Replace {
                    document: Arc::new(d)
                })
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.revision, revision);
    assert_eq!(
        serde_json::to_value(e.replay().expected_wind).unwrap(),
        before
    );
    e.execute(
        Command::EditScene {
            edits: vec![SceneEdit::SetWind {
                document: Arc::new(Document::default()),
            }],
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.replay().expected_wind.len(), 1);
    assert_eq!(e.simulation.tick, 0);
    let mut bad = e.replay();
    bad.expected_wind[0].position[0] += 1.;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["matched"], false);
}

// Substepping the shared contact world must not multiply an existing KCC target.
#[test]
fn aerodynamic_substeps_preserve_kinematic_motion_cadence() {
    let travel = |substeps: Option<u8>| {
        let mut source = scene();
        source.meshes.instances[0].position = [100., 2., 0.];
        source
            .entities
            .iter_mut()
            .find(|e| e.id == "player")
            .unwrap()
            .character_controller = Some(Default::default());
        if let Some(substeps) = substeps {
            Arc::make_mut(&mut source.wind).substeps = substeps;
        } else {
            source.wind = Default::default();
        }
        let mut engine = Engine::new(source).unwrap();
        engine.advance(30).unwrap();
        let start = engine.simulation.positions()["player"];
        engine
            .execute(
                Command::SetInput {
                    input: ignis::physics::Input {
                        x: 1.,
                        ..Default::default()
                    },
                }
                .into(),
            )
            .unwrap();
        engine.advance(30).unwrap();
        let end = engine.simulation.positions()["player"];
        DVec3::from_array(end.map(f64::from)) - DVec3::from_array(start.map(f64::from))
    };
    let ordinary = travel(None);
    assert!(ordinary.x > 1.);
    for substeps in [1, 4, 16] {
        let aerodynamic = travel(Some(substeps));
        assert!(
            (aerodynamic - ordinary).length() < 1e-3,
            "{substeps}: {aerodynamic} vs {ordinary}"
        );
    }
}
