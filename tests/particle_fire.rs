use ignis::{
    engine::{Command, Engine},
    materials::recipe::sample_count,
    model::{Camera, RoomRecipe, Scene},
    particles::{
        self, Document, Frame, Module, Operation, Placement, Preset, Source,
        fire::{Kind, Settings},
    },
    portable_save::PortableSave,
    render::Renderer,
    service::Service,
};
use std::{io::Cursor, sync::Arc};
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
    s.light.ambient = 0.15;
    s.light.direct_intensity = 0.3;
    s.camera = Camera {
        position: [0., 2., 7.],
        target: [0., 1., 0.],
        fov_degrees: 42.,
    };
    s
}
fn doc(source: Source) -> Arc<Document> {
    Arc::new(Document {
        sources: vec![Arc::new(source)],
        instances: vec![Placement {
            id: "display".into(),
            source: "fire".into(),
            seed: 0,
            position: particles::origin(),
            rotation: [0., 0., 0., 1.],
            scale: 1.,
            tint: [1.; 4],
            start_tick: 0,
            enabled: true,
        }],
        ..Default::default()
    })
}
fn run(e: &mut Engine, settings: Settings) {
    e.execute(
        Command::Particles {
            operation: Box::new(Operation::Fire {
                id: "fire".into(),
                settings,
            }),
        }
        .into(),
    )
    .unwrap();
}
#[test]
fn all_families_have_bounded_distinct_emission_and_mapped_silhouettes() {
    let mut silhouettes = vec![];
    for kind in Kind::ALL {
        let settings = Settings::for_kind(kind);
        let source = Source::fire("fire".into(), settings).unwrap();
        source.validate().unwrap();
        let wire = serde_json::to_vec(&source).unwrap();
        assert!(wire.len() < 1024);
        assert!(
            !serde_json::from_slice::<serde_json::Value>(&wire)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("emitters")
        );
        assert_eq!(serde_json::from_slice::<Source>(&wire).unwrap(), source);
        assert_eq!(source.version, 2);
        assert_eq!(source.fire.as_ref().unwrap().settings, settings);
        let f = source.emitters[0]
            .material
            .procedural
            .as_ref()
            .unwrap()
            .fixed
            .as_ref()
            .unwrap();
        assert_eq!(f.resolution, 32);
        assert_eq!(f.samples.len(), sample_count(32));
        assert!(f.samples[..1024].iter().any(|s| s.response[2] > 0.3));
        assert!(f.samples[..32].iter().all(|s| s.response[2] == 0.));
        assert!(f.samples[992..1024].iter().all(|s| s.response[2] == 0.));
        for i in 0..32 {
            assert_eq!(f.samples[i * 32].response[2], 0.);
            assert_eq!(f.samples[i * 32 + 31].response[2], 0.);
        }
        let mean = f.samples[..1024]
            .iter()
            .map(|s| s.response[2] as f64)
            .sum::<f64>()
            / 1024.;
        assert!((f.samples.last().unwrap().response[2] as f64 - mean).abs() < 1e-6);
        silhouettes.push(
            f.samples[..1024]
                .iter()
                .map(|s| s.response[2])
                .collect::<Vec<_>>(),
        );
        let d = doc(source);
        let mut frame = Frame::default();
        frame.update(&d, 120);
        assert!(!frame.samples.is_empty());
        assert!(frame.samples.len() < 4096);
        assert!(
            frame
                .samples
                .iter()
                .all(|p| p.position.iter().all(|v| v.is_finite()))
        );
    }
    for i in 0..silhouettes.len() {
        for j in 0..i {
            assert_ne!(silhouettes[i], silhouettes[j], "families{i}/{j}");
        }
    }
}
#[test]
fn extinction_intensity_seed_and_seek_are_observable_and_stable() {
    let mut settings = Settings {
        intensity: 0.,
        ..Default::default()
    };
    let mut f = Frame::default();
    f.update(&doc(Source::fire("fire".into(), settings).unwrap()), 300);
    assert!(f.samples.is_empty());
    settings.intensity = 0.4;
    let low = Source::fire("fire".into(), settings).unwrap();
    settings.intensity = 1.6;
    let high = Source::fire("fire".into(), settings).unwrap();
    assert!((high.emitters[0].rate_per_second / low.emitters[0].rate_per_second - 4.).abs() < 1e-5);
    let d = doc(high);
    f.update(&d, 150);
    let saved = f.samples.clone();
    let n = f.stats.rebuilds;
    f.update(&d, 150);
    assert_eq!(f.stats.rebuilds, n);
    f.update(&d, 90000);
    f.update(&d, 150);
    assert_eq!(saved, f.samples);
    settings.seed += 1;
    f.update(&doc(Source::fire("fire".into(), settings).unwrap()), 150);
    assert_ne!(saved, f.samples);
}
#[test]
fn offsets_require_source2_and_legacy_sources_keep_their_authored_data() {
    let mut s = Source::preset(Preset::Smoke);
    let original = s.clone();
    let raw = serde_json::to_vec(&s).unwrap();
    assert!(!String::from_utf8(raw.clone()).unwrap().contains("\"fire\""));
    assert_eq!(serde_json::from_slice::<Source>(&raw).unwrap(), original);
    s.emitters[0].rate_per_second = 0.;
    s.emitters[0].burst = 1;
    s.emitters[0].speed = [0.; 2];
    s.emitters[0].shape = particles::Shape::Point;
    s.emitters[0].modules = vec![Module::Offset {
        value: [3., 4., 5.],
    }];
    s.id = "fire".into();
    assert!(s.validate().is_err());
    s.version = 2;
    s.validate().unwrap();
    let mut f = Frame::default();
    f.update(&doc(s), 0);
    assert_eq!(f.samples[0].position, [3., 4., 5.]);
}
#[test]
fn invalid_recipe_or_derived_edit_is_atomic_and_placements_survive_updates() {
    let mut engine = Engine::new(scene()).unwrap();
    run(&mut engine, Settings::default());
    let mut changed = (*engine.scene.particles).clone();
    changed.instances[0].position = particles::origin().translated([2., 0., 1.]).unwrap();
    engine
        .execute(
            Command::Particles {
                operation: Box::new(Operation::Replace {
                    document: Arc::new(changed),
                }),
            }
            .into(),
        )
        .unwrap();
    let placed = engine.scene.particles.instances.clone();
    let original = engine.snapshot();
    let invalid = [
        Settings {
            intensity: f32::NAN,
            ..Default::default()
        },
        Settings {
            intensity: 3.1,
            ..Default::default()
        },
        Settings {
            height: 0.,
            ..Default::default()
        },
        Settings {
            width: 6.,
            ..Default::default()
        },
        Settings {
            turbulence: 2.1,
            ..Default::default()
        },
    ];
    for settings in invalid {
        assert!(
            engine
                .execute(
                    Command::Particles {
                        operation: Box::new(Operation::Fire {
                            id: "fire".into(),
                            settings
                        })
                    }
                    .into()
                )
                .is_err()
        );
        assert_eq!(engine.snapshot(), original);
    }
    let mut corrupt = (*engine.scene.particles.sources[0]).clone();
    corrupt.emitters[0].rate_per_second += 1.;
    assert!(
        engine
            .execute(
                Command::Particles {
                    operation: Box::new(Operation::PutSource {
                        source: Arc::new(corrupt)
                    })
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(engine.snapshot(), original);
    run(
        &mut engine,
        Settings {
            intensity: 2.,
            ..Default::default()
        },
    );
    assert_eq!(engine.scene.particles.instances, placed);
    engine.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        engine.scene.particles.sources[0]
            .fire
            .as_ref()
            .unwrap()
            .settings
            .intensity,
        1.
    );
    let mut legacy = engine.scene.clone();
    legacy.version = particles::FIRE_SCENE_VERSION - 1;
    assert!(legacy.validate().is_err());
}
#[test]
fn source_files_and_world_continuation_preserve_fire_recipes_and_samples() {
    let tmp = tempfile::tempdir().unwrap();
    let mut service = Service::new(tmp.path(), scene()).unwrap();
    run(&mut service.engine, Settings::for_kind(Kind::Arcane));
    let source = service.engine.scene.particles.sources[0].clone();
    service
        .execute(
            Command::Particles {
                operation: Box::new(Operation::SaveSource {
                    id: "fire".into(),
                    path: "fire.ignis-vfx".into(),
                }),
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::Particles {
                operation: Box::new(Operation::LoadSource {
                    path: "fire.ignis-vfx".into(),
                }),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene.particles.sources[0], source);
    service
        .engine
        .execute(Command::Step { ticks: 90 }.into())
        .unwrap();
    let expected = service.engine.render_frame().particles.samples;
    assert!(!expected.is_empty());
    let replay = Engine::replay_at(&service.engine.replay(), None).unwrap();
    assert_eq!(replay.render_frame().particles.samples, expected);
    let bytes = PortableSave::capture(&service.engine)
        .unwrap()
        .encode()
        .unwrap();
    let restored = Engine::from_portable_save(&bytes, None).unwrap();
    assert_eq!(restored.render_frame().particles.samples, expected);
    let c = ignis::network::content::Content::capture(&service.engine, 17);
    let c = ignis::network::content::Content::decode(17, &c.encode().unwrap()).unwrap();
    let mut f = Frame::default();
    f.update(&c.particles, 90);
    assert_eq!(f.samples, expected);
}
#[test]
fn detaching_is_explicit_preserves_emitters_and_refuses_destructive_regeneration() {
    let mut e = Engine::new(scene()).unwrap();
    run(&mut e, Settings::default());
    let emitters = e.scene.particles.sources[0].emitters.clone();
    e.execute(
        Command::Particles {
            operation: Box::new(Operation::DetachFire { id: "fire".into() }),
        }
        .into(),
    )
    .unwrap();
    assert!(e.scene.particles.sources[0].fire.is_none());
    assert_eq!(e.scene.particles.sources[0].emitters, emitters);
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::Particles {
                operation: Box::new(Operation::Fire {
                    id: "fire".into(),
                    settings: Settings::default()
                })
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    e.execute(Command::Undo.into()).unwrap();
    assert!(e.scene.particles.sources[0].fire.is_some());
}
fn rgba(b: &[u8]) -> Vec<u8> {
    let mut p = png::Decoder::new(Cursor::new(b)).read_info().unwrap();
    let mut out = vec![0; p.output_buffer_size().unwrap()];
    let i = p.next_frame(&mut out).unwrap();
    out.truncate(i.buffer_size());
    out
}
#[test]
#[ignore = "requires native GPU"]
fn native_fire_is_visible_varied_and_keeps_warm_uploads_stable() {
    let instance = wgpu::Instance::new(&Default::default());
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let mut images = vec![];
    for kind in [Kind::Campfire, Kind::BlueJet, Kind::Arcane] {
        let mut e = Engine::new(scene()).unwrap();
        run(&mut e, Settings::for_kind(kind));
        e.execute(
            Command::Particles {
                operation: Box::new(Operation::Preview { tick: Some(100) }),
            }
            .into(),
        )
        .unwrap();
        let mut frame = e.render_frame();
        let lit = rgba(&gpu.capture(&frame, 256, 256).unwrap());
        let uploads = gpu.particle_diagnostics()["uploads"].clone();
        e.update_render_frame(&mut frame);
        assert_eq!(rgba(&gpu.capture(&frame, 256, 256).unwrap()), lit);
        assert_eq!(gpu.particle_diagnostics()["uploads"], uploads);
        run(
            &mut e,
            Settings {
                kind,
                intensity: 0.,
                ..Settings::for_kind(kind)
            },
        );
        let dark = rgba(&gpu.capture(&e.render_frame(), 256, 256).unwrap());
        assert!(
            lit.chunks_exact(4)
                .zip(dark.chunks_exact(4))
                .filter(|(a, b)| a[..3] != b[..3])
                .count()
                > 100
        );
        images.push(lit);
    }
    assert_ne!(images[0], images[1]);
    assert_ne!(images[1], images[2]);
}
