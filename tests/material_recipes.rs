use ignis::{
    engine::{Command, Engine, Request},
    materials::{
        Material,
        library::{self, Execution, Operation, Preset},
        recipe::{Field, Layer, Procedural, Recipe, Relief, Sample},
    },
    meshes::{Asset, Geometry, Instance, Shading},
    model::{Camera, RoomRecipe, Scene},
    render::Renderer,
    service::Service,
};
use std::sync::Arc;
fn source() -> Scene {
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
        position: [0., 0., 3.],
        target: [0., 0., 0.],
        fov_degrees: 45.,
    };
    s.light.ambient = 1.;
    s.meshes.assets = vec![Asset {
        id: "panel".into(),
        geometry: Arc::new(Geometry {
            positions: vec![[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            material_coordinates: None,
        }),
        material: Material::default(),
        shading: Shading::Smooth,
        rig: None,
        clips: vec![],
    }];
    s.meshes.instances = vec![Instance {
        id: "panel-one".into(),
        asset: "panel".into(),
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [1.; 3],
        attachment: None,
        playback: None,
    }];
    s
}
fn command(op: Operation) -> Request {
    Command::Material {
        operation: Box::new(op),
    }
    .into()
}
fn recipe(field: Field) -> Recipe {
    Recipe {
        base: Sample {
            color_roughness: [0.1, 0.2, 0.3, 0.7],
            ..Default::default()
        },
        layers: vec![Layer {
            field,
            frequency: [4, 4],
            octaves: 1,
            target: Sample {
                color_roughness: [0.8, 0.6, 0.2, 0.3],
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
#[test]
fn periodic_fields_fixed_maps_and_seeds_are_reproducible() {
    for field in [
        Field::Noise,
        Field::Ridged,
        Field::Cells,
        Field::Bricks,
        Field::Waves,
    ] {
        let r = recipe(field);
        let p = Procedural::runtime(r.clone());
        p.validate().unwrap();
        let fixed = p.bake(64).unwrap();
        assert_eq!(fixed, p.bake(64).unwrap());
        for y in 0..11 {
            for x in 0..11 {
                let uv = [x as f32 / 16., y as f32 / 16.];
                assert_eq!(p.sample(uv, 0.), p.sample([uv[0] + 1., uv[1] - 2.], 0.));
                assert_eq!(
                    fixed.sample(uv, 0.),
                    fixed.sample([uv[0] + 1., uv[1] - 2.], 0.)
                );
            }
        }
        let mut changed = fixed.clone();
        changed.recipe.seed += 1;
        assert_eq!(
            fixed.sample([0.32, 0.41], 0.),
            changed.sample([0.32, 0.41], 0.)
        );
        let far = fixed.sample([0.12, 0.8], 100.);
        assert_eq!(far, fixed.sample([0.9, 0.1], 100.));
    }
    let mut p = Procedural::runtime(recipe(Field::Noise));
    p.recipe.unique_instances = true;
    p.recipe.seed = u32::MAX;
    assert_eq!(p.instance("stone-2"), p.instance("stone-2"));
    assert_ne!(p.instance("stone-1"), p.instance("stone-2"));
    let m = library::material(p.recipe.clone(), Execution::Fixed { resolution: 32 }).unwrap();
    assert_eq!(m, library::decode(&library::encode(&m).unwrap()).unwrap());
    let mut invalid = (*m.procedural.unwrap()).clone();
    Arc::make_mut(&mut invalid.fixed.as_mut().unwrap().samples).pop();
    assert!(invalid.validate().is_err());
}
#[test]
fn material_library_application_is_atomic_versioned_and_reuses_geometry() {
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(root.path(), source()).unwrap();
    let geometry = service.engine.scene.meshes.assets[0].geometry.clone();
    service
        .execute(command(Operation::Preset {
            asset: "panel".into(),
            preset: Preset::Stone,
            seed: Some(43),
            unique_instances: true,
            execution: Execution::Runtime,
        }))
        .unwrap();
    assert!(service.engine.scene.version >= 112);
    assert!(Arc::ptr_eq(
        &geometry,
        &service.engine.scene.meshes.assets[0].geometry
    ));
    let frame = service.engine.render_frame();
    assert_eq!(frame.meshes.materials.len(), 3);
    let before = service.engine.scene.clone();
    let revision = service.engine.revision;
    let mut bad = recipe(Field::Noise);
    bad.tile_size[0] = f32::NAN;
    assert!(
        service
            .execute(command(Operation::Generate {
                asset: "panel".into(),
                recipe: Box::new(bad),
                execution: Execution::Runtime
            }))
            .is_err()
    );
    assert_eq!(before, service.engine.scene);
    service
        .execute(command(Operation::SetExecution {
            asset: "panel".into(),
            execution: Execution::Fixed { resolution: 32 },
        }))
        .unwrap();
    service
        .execute(command(Operation::Save {
            asset: "panel".into(),
            path: "materials/stone.ignis-material".into(),
        }))
        .unwrap();
    let saved = service.engine.scene.meshes.assets[0].material.clone();
    service
        .execute(command(Operation::SetExecution {
            asset: "panel".into(),
            execution: Execution::Runtime,
        }))
        .unwrap();
    service
        .execute(command(Operation::Load {
            path: "materials/stone.ignis-material".into(),
            asset: Some("panel".into()),
        }))
        .unwrap();
    assert_eq!(saved, service.engine.scene.meshes.assets[0].material);
    let stable = service.engine.scene.clone();
    let mut request = command(Operation::SetExecution {
        asset: "panel".into(),
        execution: Execution::Runtime,
    });
    request.expected_revision = Some(revision);
    assert!(service.execute(request).is_err());
    assert_eq!(stable, service.engine.scene);
    service.execute(Command::Undo.into()).unwrap();
    assert!(
        service.engine.scene.meshes.assets[0]
            .material
            .procedural
            .as_ref()
            .unwrap()
            .fixed
            .is_none()
    );
    let mut legacy = before;
    legacy.version = 111;
    assert!(Engine::new(legacy).is_err());
    let data = ignis::network::content::Content::capture(&service.engine, 1);
    let bytes = data.encode().unwrap();
    assert_eq!(
        data.meshes,
        ignis::network::content::Content::decode(1, &bytes)
            .unwrap()
            .meshes
    );
}
fn rgba(bytes: &[u8]) -> Vec<u8> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut out = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut out).unwrap();
    out.truncate(info.buffer_size());
    out
}
fn capture(gpu: &mut Renderer, s: &Scene) -> Vec<u8> {
    rgba(
        &gpu.capture(&Engine::new(s.clone()).unwrap().render_frame(), 256, 256)
            .unwrap(),
    )
}
#[test]
#[ignore = "requires native GPU"]
fn native_fixed_and_runtime_fields_match_and_warm_frames_reuse_uploads() {
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let mut s = source();
    for field in [
        Field::Noise,
        Field::Ridged,
        Field::Cells,
        Field::Bricks,
        Field::Waves,
    ] {
        let mut r = recipe(field);
        r.seed = u32::MAX;
        s.meshes.assets[0].material = library::material(r.clone(), Execution::Runtime).unwrap();
        let live = capture(&mut gpu, &s);
        s.meshes.assets[0].material =
            library::material(r, Execution::Fixed { resolution: 256 }).unwrap();
        let fixed = capture(&mut gpu, &s);
        let error = live
            .iter()
            .zip(&fixed)
            .map(|(a, b)| f64::from(a.abs_diff(*b)))
            .sum::<f64>()
            / live.len() as f64;
        assert!(error < 2.0, "{field:?}: average RGB8 error {error}");
        let uploads = gpu.info()["procedural_materials"]["uploads"].clone();
        assert_eq!(fixed, capture(&mut gpu, &s));
        assert_eq!(uploads, gpu.info()["procedural_materials"]["uploads"]);
    }
    s.meshes.assets[0].material =
        library::decode(&library::encode(&s.meshes.assets[0].material).unwrap()).unwrap();
    let reloaded = capture(&mut gpu, &s);
    let uploads = gpu.info()["procedural_materials"]["uploads"].clone();
    assert_eq!(reloaded, capture(&mut gpu, &s));
    assert_eq!(uploads, gpu.info()["procedural_materials"]["uploads"]);
    let mut r = recipe(Field::Noise);
    r.unique_instances = true;
    s.meshes.assets[0].material = library::material(r, Execution::Runtime).unwrap();
    s.meshes.instances[0].position[0] = -0.6;
    s.meshes.instances[0].scale = 0.5;
    let mut second = s.meshes.instances[0].clone();
    second.id = "panel-two".into();
    second.position[0] = 0.6;
    s.meshes.instances.push(second);
    let before = capture(&mut gpu, &s);
    s.meshes.instances.reverse();
    assert_eq!(before, capture(&mut gpu, &s));
    let frame = Engine::new(s.clone()).unwrap().render_frame();
    assert_ne!(
        frame.meshes.materials[2]
            .procedural
            .as_ref()
            .unwrap()
            .recipe
            .seed,
        frame.meshes.materials[3]
            .procedural
            .as_ref()
            .unwrap()
            .recipe
            .seed
    );
    let mut uniform = s.clone();
    let p = Arc::make_mut(
        uniform.meshes.assets[0]
            .material
            .procedural
            .as_mut()
            .unwrap(),
    );
    p.recipe.unique_instances = false;
    assert_ne!(before, capture(&mut gpu, &uniform));
}
#[test]
#[ignore = "requires native GPU"]
fn parallax_and_self_shadow_are_live_light_dependent_and_optional() {
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let mut s = source();
    s.camera.position = [1.4, 0.2, 2.6];
    s.light.ambient = 0.1;
    s.light.direction = [-1., -0.15, -0.25];
    let mut r = Preset::Ceramic.recipe();
    r.height_m = 0.12;
    r.view_steps = 32;
    r.shadow_steps = 24;
    r.tile_size = [2., 2.];
    s.meshes.assets[0].material =
        library::material(r.clone(), Execution::Fixed { resolution: 256 }).unwrap();
    let shadow = capture(&mut gpu, &s);
    r.shadow_steps = 0;
    s.meshes.assets[0].material =
        library::material(r.clone(), Execution::Fixed { resolution: 256 }).unwrap();
    let no_shadow = capture(&mut gpu, &s);
    assert_ne!(shadow, no_shadow);
    r.relief = Relief::Bump;
    s.meshes.assets[0].material =
        library::material(r, Execution::Fixed { resolution: 256 }).unwrap();
    assert_ne!(no_shadow, capture(&mut gpu, &s));
}

#[test]
fn explicit_displacement_preserves_source_rest_domain_and_undo() {
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(root.path(), source()).unwrap();
    service
        .execute(command(Operation::Preset {
            asset: "panel".into(),
            preset: Preset::Stone,
            seed: Some(19),
            unique_instances: true,
            execution: Execution::Runtime,
        }))
        .unwrap();
    let before = service.engine.scene.clone();
    let reference = service.engine.mesh_reference();
    service
        .execute(command(Operation::BakeDisplacement {
            instance: "panel-one".into(),
            id: "relief".into(),
            reference: reference.clone(),
        }))
        .unwrap();
    assert_eq!(
        service.engine.scene.meshes.assets[0],
        before.meshes.assets[0]
    );
    let generated = service.engine.scene.meshes.asset("relief").unwrap();
    assert_ne!(
        generated.geometry.positions,
        before.meshes.assets[0].geometry.positions
    );
    assert_eq!(
        generated.geometry.material_coordinates.as_ref().unwrap(),
        &before.meshes.assets[0].geometry.positions
    );
    assert_eq!(
        generated
            .material
            .procedural
            .as_ref()
            .unwrap()
            .recipe
            .height_m,
        0.
    );
    assert_eq!(service.engine.scene.meshes.instances[0].asset, "relief");
    assert!(
        service
            .execute(command(Operation::BakeDisplacement {
                instance: "panel-one".into(),
                id: "stale".into(),
                reference
            }))
            .is_err()
    );
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(before, service.engine.scene);
}

#[test]
#[ignore = "requires native GPU"]
fn recipe_opacity_and_emission_use_the_shared_transparent_composition() {
    let instance = wgpu::Instance::default();
    let mut gpu = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let mut s = source();
    let mut back = s.meshes.assets[0].clone();
    back.id = "back".into();
    s.meshes.assets.push(back);
    let mut back = s.meshes.instances[0].clone();
    back.id = "back".into();
    back.asset = "back".into();
    back.position[2] = -0.2;
    back.color = [0.1, 0.2, 0.3];
    s.meshes.instances.push(back);
    let recipe = Recipe {
        base: Sample {
            color_roughness: [0.8, 0.1, 0.2, 0.7],
            response: [0., 0.5, 0.25, 1.],
            emission: [0.1, 0.2, 0., 0.],
        },
        ..Default::default()
    };
    for mode in [Execution::Runtime, Execution::Fixed { resolution: 16 }] {
        s.meshes.assets[0].material = library::material(recipe.clone(), mode).unwrap();
        let pixels = capture(&mut gpu, &s);
        let center = &pixels[(128 * 256 + 128) * 4..(128 * 256 + 128) * 4 + 3];
        for (actual, linear) in center.iter().zip([0.3f32, 0.225, 0.275]) {
            let expected = (if linear <= 0.0031308 {
                12.92 * linear
            } else {
                1.055 * linear.powf(1. / 2.4) - 0.055
            } * 255.)
                .round() as u8;
            assert!(actual.abs_diff(expected) <= 2, "{center:?}");
        }
    }
}
