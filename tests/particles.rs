use ignis::{
    engine::{Command, Engine},
    materials::{BlendMode, Material, Surface},
    model::{Camera, RoomRecipe, Scene},
    particles::{
        self, ColorKey, Document, Emitter, Frame, Module, Operation, Placement, Preset, SizeKey,
        Source,
    },
    portable_save::PortableSave,
    render::Renderer,
    service::Service,
};
use std::{io::Cursor, sync::Arc};
fn placement(id: &str, source: &str) -> Placement {
    Placement {
        id: id.into(),
        source: source.into(),
        seed: 0,
        position: particles::origin(),
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        tint: [1.; 4],
        start_tick: 0,
        enabled: true,
    }
}
fn document(e: Emitter, looping: bool) -> Arc<Document> {
    Arc::new(Document {
        sources: vec![Arc::new(Source {
            version: 1,
            id: "fx".into(),
            seed: 42,
            duration_ticks: 60,
            looping,
            emitters: vec![e],
            fire: None,
        })],
        instances: vec![placement("instance", "fx")],
        ..Default::default()
    })
}
fn basic() -> Emitter {
    Emitter {
        rate_per_second: 0.,
        burst: 1,
        lifetime_ticks: [60, 60],
        speed: [0., 0.],
        spread_degrees: 0.,
        size: [1., 1.],
        ..Default::default()
    }
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    for e in &mut s.entities {
        e.visible = Some(false);
    }
    s.camera = Camera {
        position: [0., 0., 6.],
        target: [0.; 3],
        fov_degrees: 45.,
    };
    s.light.ambient = 1.;
    s.light.direct_intensity = 0.;
    s.light.atmosphere = None;
    s.light.environment = None;
    s
}
fn run(e: &mut Engine, op: Operation) -> serde_json::Value {
    e.execute(
        Command::Particles {
            operation: Box::new(op),
        }
        .into(),
    )
    .unwrap()
}
#[test]
fn analytic_motion_matches_independent_closed_forms_and_zero_drag_limit() {
    let p = [3., -4., 1.];
    let v = [2., 5., -1.];
    let a = [0.2, -9., 0.5];
    let t = 2.7;
    for d in [0., 1e-12, 0.2, 4.] {
        let (x, u) = particles::trajectory(p, v, a, d, t);
        for c in 0..3 {
            let (xp, vp) = if d < 1e-10 {
                (p[c] + v[c] * t + 0.5 * a[c] * t * t, v[c] + a[c] * t)
            } else {
                let terminal = a[c] / d;
                let decay = (-d * t).exp();
                (
                    p[c] + terminal * t + (v[c] - terminal) * (1. - decay) / d,
                    terminal + (v[c] - terminal) * decay,
                )
            };
            assert!((x[c] - xp).abs() < 1e-9);
            assert!((u[c] - vp).abs() < 1e-9);
        }
    }
}
#[test]
fn rate_and_burst_schedule_matches_integer_oracle_and_finite_drain() {
    let e = Emitter {
        rate_per_second: 17.123,
        burst: 3,
        delay_ticks: 7,
        lifetime_ticks: [19, 19],
        ..basic()
    };
    for looping in [false, true] {
        let d = document(e.clone(), looping);
        d.validate().unwrap();
        let mut f = Frame::default();
        for tick in [0, 7, 8, 19, 59, 60, 67, 130, 10000] {
            f.update(&d, tick);
            let mut count = 0;
            for born in 0..=tick {
                if tick - born >= 19 || born < 7 {
                    continue;
                }
                if (looping || born < 60) && born > 7 {
                    let before = (born - 8) * 17123 / 60000;
                    let after = (born - 7) * 17123 / 60000;
                    count += (after - before) as usize;
                }
                if (looping || born == 7) && (born - 7) % 60 == 0 {
                    count += 3;
                }
            }
            assert_eq!(f.samples.len(), count, "loop={looping},t={tick}");
        }
    }
}
#[test]
fn seek_seed_channels_and_warm_frame_are_stable_without_simulation_history() {
    let mut e = Source::preset(Preset::Sparks).emitters.remove(0);
    e.lifetime_ticks = [100, 100];
    let d = document(e, true);
    let mut f = Frame::default();
    f.update(&d, 78);
    let expected = f.samples.clone();
    let builds = f.stats.rebuilds;
    let ptr = f.samples.as_ptr();
    f.update(&d, 78);
    assert_eq!(f.stats.rebuilds, builds);
    assert_eq!(f.samples.as_ptr(), ptr);
    for t in [10000, 0, 78] {
        f.update(&d, t);
    }
    assert_eq!(f.samples, expected);
    let mut changed = (*d).clone();
    Arc::make_mut(&mut changed.sources[0]).emitters[0].color_min = [0.1; 4];
    f.update(&Arc::new(changed), 78);
    for (a, b) in f.samples.iter().zip(&expected) {
        assert_eq!(a.position, b.position);
        assert_eq!(a.velocity, b.velocity);
        assert_eq!(a.size, b.size);
    }
    let mut changed = (*d).clone();
    changed.instances[0].seed += 1;
    f.update(&Arc::new(changed), 78);
    assert!(
        f.samples
            .iter()
            .zip(&expected)
            .any(|(a, b)| a.velocity != b.velocity)
    );
}
#[test]
fn curves_shape_transform_and_projection_have_observable_boundaries() {
    let e = Emitter {
        speed: [2., 2.],
        direction: [0., 1., 0.],
        modules: vec![
            Module::Acceleration {
                value: [0., -2., 0.],
            },
            Module::ColorOverLife {
                keys: vec![
                    ColorKey {
                        time: 0.,
                        value: [1.; 4],
                    },
                    ColorKey {
                        time: 1.,
                        value: [0.; 4],
                    },
                ],
            },
            Module::SizeOverLife {
                keys: vec![
                    SizeKey {
                        time: 0.,
                        value: 1.,
                    },
                    SizeKey {
                        time: 1.,
                        value: 3.,
                    },
                ],
            },
        ],
        ..basic()
    };
    let mut d = (*document(e, false)).clone();
    d.instances[0].position = particles::origin().translated([3., 2., 0.]).unwrap();
    d.instances[0].scale = 2.;
    let d = Arc::new(d);
    let mut f = Frame::default();
    f.update(&d, 30);
    let p = &f.samples[0];
    assert_eq!(p.position, [3., 3.5, 0.]);
    assert_eq!(p.color, [0.5; 4]);
    assert_eq!(p.size, 4.);
    assert_eq!(p.velocity, [0., 2., 0.]);
    let mut far = (*d).clone();
    far.instances[0].position = particles::origin()
        .translated([2_000_000., 0., 0.])
        .unwrap();
    f.update(&Arc::new(far), 30);
    assert!(f.samples.is_empty());
    assert_eq!(f.stats.projection_drops, 1);
}
#[test]
fn adversarial_budget_is_bounded_at_u64_time_and_many_emitters() {
    let mut d = (*document(
        Emitter {
            rate_per_second: 100000.,
            burst: 65536,
            max_particles: 65536,
            lifetime_ticks: [3600, 3600],
            ..basic()
        },
        true,
    ))
    .clone();
    d.max_particles = 3;
    let source = Arc::make_mut(&mut d.sources[0]);
    source.emitters = (0..32)
        .map(|i| {
            let mut e = source.emitters[0].clone();
            e.id = format!("e_{i}");
            e
        })
        .collect();
    d.validate().unwrap();
    let d = Arc::new(d);
    let mut f = Frame::default();
    f.update(&d, u64::MAX);
    assert!(f.samples.len() <= 3);
    assert!(f.stats.candidates <= 48);
    assert!(f.stats.evaluated <= 48);
    assert!(f.stats.work_limited);
    assert!(f.stats.budget_drops > 0);
    let a = f.samples.clone();
    f.update(&d, 0);
    f.update(&d, u64::MAX);
    assert_eq!(a, f.samples);
}
#[test]
fn invalid_mutations_and_legacy_version_fail_atomically_and_undo_restores() {
    let mut e = Engine::new(scene()).unwrap();
    let before = e.snapshot();
    let mut bad = (*document(basic(), true)).clone();
    bad.instances[0].source = "missing".into();
    assert!(
        e.execute(
            Command::Particles {
                operation: Box::new(Operation::Replace {
                    document: Arc::new(bad)
                })
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    run(
        &mut e,
        Operation::Replace {
            document: document(basic(), true),
        },
    );
    assert!(e.scene.version >= particles::SCENE_VERSION);
    let mut legacy = e.scene.clone();
    legacy.version = particles::SCENE_VERSION - 1;
    assert!(legacy.validate().is_err());
    e.execute(Command::Undo.into()).unwrap();
    assert!(e.scene.particles.is_empty());
    e.execute(Command::Redo.into()).unwrap();
    assert!(!e.scene.particles.is_empty());
    let mut cases = vec![];
    let mut bad = basic();
    bad.rate_per_second = f32::NAN;
    cases.push(bad);
    let mut bad = basic();
    bad.rate_per_second = 0.0001;
    cases.push(bad);
    let mut bad = basic();
    bad.direction = [0.; 3];
    cases.push(bad);
    let mut bad = basic();
    bad.material.surface.as_mut().unwrap().mode = BlendMode::Opaque;
    cases.push(bad);
    for bad in cases {
        assert!(document(bad, true).validate().is_err());
    }
    let mut bad = basic();
    bad.modules = vec![Module::SizeOverLife {
        keys: vec![
            SizeKey {
                time: 0.,
                value: 1.,
            },
            SizeKey {
                time: 0.,
                value: 2.,
            },
        ],
    }];
    assert!(document(bad, true).validate().is_err());
}
#[test]
fn scene_replay_portable_and_public_content_reconstruct_the_same_cosmetic_frame() {
    let mut s = scene();
    s.particles = document(Source::preset(Preset::Smoke).emitters.remove(0), true);
    let mut e = Engine::new(s).unwrap();
    e.execute(Command::Step { ticks: 120 }.into()).unwrap();
    let expected = e.render_frame().particles.samples;
    assert!(!expected.is_empty());
    let replay = e.replay();
    let r = Engine::replay_at(&replay, None).unwrap();
    assert_eq!(r.render_frame().particles.samples, expected);
    let saved = PortableSave::capture(&e).unwrap().encode().unwrap();
    let r = Engine::from_portable_save(&saved, None).unwrap();
    assert_eq!(r.render_frame().particles.samples, expected);
    let content = ignis::network::content::Content::capture(&e, 7);
    let decoded = ignis::network::content::Content::decode(7, &content.encode().unwrap()).unwrap();
    let mut f = Frame::default();
    f.update(&decoded.particles, 120);
    assert_eq!(f.samples, expected);
    run(&mut e, Operation::Preview { tick: Some(0) });
    assert_ne!(e.render_frame().particles.samples, expected);
    let saved = PortableSave::capture(&e).unwrap().encode().unwrap();
    let r = Engine::from_portable_save(&saved, None).unwrap();
    assert_eq!(r.particle_preview, None);
    assert_eq!(r.render_frame().particles.samples, expected);
}
#[test]
fn source_file_round_trip_rejects_traversal_and_preserves_instance_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(root.path(), scene()).unwrap();
    let call = |s: &mut Service, o| {
        s.execute(
            Command::Particles {
                operation: Box::new(o),
            }
            .into(),
        )
    };
    call(
        &mut service,
        Operation::Preset {
            preset: Preset::Sparks,
            id: "fire".into(),
        },
    )
    .unwrap();
    let original = service.engine.scene.particles.sources[0].clone();
    call(
        &mut service,
        Operation::SaveSource {
            id: "fire".into(),
            path: "assets/fire.ignis-vfx".into(),
        },
    )
    .unwrap();
    assert!(
        call(
            &mut service,
            Operation::SaveSource {
                id: "fire".into(),
                path: "../escape.ignis-vfx".into(),
            },
        )
        .is_err()
    );
    call(
        &mut service,
        Operation::LoadSource {
            path: "assets/fire.ignis-vfx".into(),
        },
    )
    .unwrap();
    assert_eq!(service.engine.scene.particles.sources[0], original);
    assert_eq!(service.engine.scene.particles.instances.len(), 1);
}
#[test]
fn planetary_adapter_refuses_placements_before_changing_the_world() {
    let mut s = scene();
    s.particles = document(basic(), true);
    let mut e = Engine::new(s).unwrap();
    let config = ignis::planet::runtime::Config {
        document: ignis::planet::Document {
            version: 1,
            space_version: 1,
            id: "planet".into(),
            revision: 1,
            center: particles::origin(),
            diameter_meters: 10000.,
            surface_gravity_mps2: 9.81,
            base_height_meters: 0.,
            operations: vec![],
        },
        actor: "player".into(),
        spawn: ignis::planet::Pose {
            direction: [0., 1., 0.],
            forward: [0., 0., -1.],
            altitude_meters: 1.,
        },
        level: 4,
        subdivisions: 8,
        physical_radius_meters: 128.,
        color: [0.5; 3],
        material: Default::default(),
        compile_distant_meshes: false,
    };
    let before = e.snapshot();
    let result = e.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(config)),
        }
        .into(),
    );
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("particle placements")
    );
    assert_eq!(before, e.snapshot());
}
fn rgba(bytes: &[u8]) -> Vec<u8> {
    let mut r = png::Decoder::new(Cursor::new(bytes)).read_info().unwrap();
    let mut b = vec![0; r.output_buffer_size().unwrap()];
    let i = r.next_frame(&mut b).unwrap();
    b.truncate(i.buffer_size());
    b
}
#[test]
#[ignore = "requires native GPU"]
fn native_sprite_depth_tint_alpha_and_camera_only_upload_reuse() {
    let instance = wgpu::Instance::new(&Default::default());
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let e = Emitter {
        material: Material {
            surface: Some(Surface {
                mode: BlendMode::Additive,
                emissive: [1.; 3],
                ..Default::default()
            }),
            ..Default::default()
        },
        size: [2., 2.],
        softness: 0.,
        color_min: [0.25, 0.1, 0.05, 0.5],
        color_max: [0.25, 0.1, 0.05, 0.5],
        ..basic()
    };
    let mut s = scene();
    s.light.ambient = 0.;
    s.particles = document(e, false);
    let mut engine = Engine::new(s.clone()).unwrap();
    let mut frame = engine.render_frame();
    let image = rgba(&gpu.capture(&frame, 128, 128).unwrap());
    let center = &image[(64 * 128 + 64) * 4..(64 * 128 + 64) * 4 + 3];
    assert!(center[0] > center[1] && center[1] > center[2], "{center:?}");
    let srgb = |v: f32| {
        ((if v <= 0.0031308 {
            v * 12.92
        } else {
            1.055 * v.powf(1. / 2.4) - 0.055
        }) * 255.)
            .round() as u8
    };
    for (actual, linear) in
        center
            .iter()
            .zip([0.018 + 0.25 * 0.5, 0.027 + 0.1 * 0.5, 0.045 + 0.05 * 0.5])
    {
        assert!(actual.abs_diff(srgb(linear)) <= 2, "{center:?}");
    }
    assert_eq!(gpu.particle_diagnostics()["uploads"], 1);
    let builds = frame.particles.stats.rebuilds;
    engine.scene.camera.position = [2., 0., 6.];
    engine.update_render_frame(&mut frame);
    gpu.capture(&frame, 128, 128).unwrap();
    assert_eq!(frame.particles.stats.rebuilds, builds);
    assert_eq!(gpu.particle_diagnostics()["uploads"], 1);
    // Camera-facing quads must remain visible from the side too.
    engine.scene.camera.position = [6., 0., 0.];
    engine.update_render_frame(&mut frame);
    let side = rgba(&gpu.capture(&frame, 128, 128).unwrap());
    assert_eq!(&side[(64 * 128 + 64) * 4..(64 * 128 + 64) * 4 + 3], center);
    s.entities[0].visible = Some(true);
    s.entities[0].transform.position = [0., 0., 2.];
    s.entities[0].transform.size = [3., 3., 0.3];
    s.entities[0].color = [0.1, 0.2, 0.3];
    let mut occluded = Engine::new(s.clone()).unwrap().render_frame();
    let behind = gpu.capture(&occluded, 128, 128).unwrap();
    occluded.particles.clear();
    let without = gpu.capture(&occluded, 128, 128).unwrap();
    assert!(
        rgba(&behind)
            .iter()
            .zip(rgba(&without))
            .all(|(a, b)| a.abs_diff(b) <= 1)
    );
    assert_eq!(gpu.info()["transparency"]["active"], false);
    let mut blend = (*s.particles).clone();
    Arc::make_mut(&mut blend.sources[0]).emitters[0]
        .material
        .surface
        .as_mut()
        .unwrap()
        .mode = BlendMode::Blend;
    let mut visible = scene();
    visible.light.ambient = 0.;
    visible.particles = Arc::new(blend);
    let bytes = rgba(
        &gpu.capture(&Engine::new(visible).unwrap().render_frame(), 128, 128)
            .unwrap(),
    );
    let pixel = &bytes[(64 * 128 + 64) * 4..(64 * 128 + 64) * 4 + 3];
    for (actual, linear) in pixel.iter().zip([
        0.018 * 0.5 + 0.25 * 0.5,
        0.027 * 0.5 + 0.1 * 0.5,
        0.045 * 0.5 + 0.05 * 0.5,
    ]) {
        assert!(actual.abs_diff(srgb(linear)) <= 2, "{pixel:?}");
    }
}
