use super::*;
use crate::{
    engine::{Command, Engine, ExecutionMode},
    model::RoomRecipe,
    world_space::{Coordinates, Position},
};

pub(super) fn config() -> super::super::runtime::Config {
    let document = Document {
        version: 1,
        space_version: 1,
        id: "orbit_test".into(),
        revision: 1,
        center: Position::normalize(Coordinates {
            cell: [9_007_199_254_740_993, -9_007_199_254_740_993, 7],
            offset: [0.125, 0.25, 0.5],
        })
        .unwrap(),
        diameter_meters: 12_742_000.,
        surface_gravity_mps2: 9.81,
        base_height_meters: 0.,
        operations: vec![],
    };
    super::super::runtime::Config {
        document,
        actor: "player".into(),
        spawn: Pose {
            direction: [0., 1., 0.],
            forward: [0., 0., -1.],
            altitude_meters: 0.91,
        },
        level: 12,
        subdivisions: 32,
        physical_radius_meters: 128.,
        color: [0.3, 0.5, 0.2],
        material: Default::default(),
        compile_distant_meshes: false,
    }
}
fn anchor(c: &super::super::runtime::Config) -> Anchor {
    Anchor::new(
        &c.document,
        Pose {
            altitude_meters: 0.01,
            ..c.spawn
        },
    )
    .unwrap()
}
fn engine() -> Engine {
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene.entities.retain(|e| e.id == "player");
    let p = &mut scene.entities[0];
    p.character_controller = Some(Default::default());
    p.footsteps = None;
    p.transform.position = [0., 0.91, 0.];
    scene.light.atmosphere = Some(crate::atmosphere::Atmosphere {
        density: 0.02,
        sun_strength: 0.,
        ..Default::default()
    });
    let mut e = Engine::new_with_mode(scene, ExecutionMode::Continuous).unwrap();
    e.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(config())),
        }
        .into(),
    )
    .unwrap();
    e
}
fn atlas(c: &super::super::runtime::Config, s: Settings) -> Atlas {
    Atlas::new(
        c,
        s,
        anchor(c),
        ActorView {
            size: [0.6, 1.8, 0.6],
            color: [0.8, 0.6, 0.1],
            eye: 1.62,
        },
        &RoomRecipe::default().generate().unwrap().light,
        1.,
        Sampler::new(&c.document).unwrap(),
    )
    .unwrap()
}

#[test]
fn planet_orbit_conforming_closed_surface_across_levels_and_cube_corners() {
    let mut c = config();
    c.document.operations.push(Relief::Raise {
        direction: [1., 1., 1.],
        angular_radius_degrees: 60.,
        height_meters: 1800.,
    });
    let mut leaves: BTreeSet<_> = FACES
        .into_iter()
        .map(|face| Tile {
            face,
            level: 0,
            xy: [0, 0],
        })
        .collect();
    // Deliberately leave a three-level mismatch at a cube corner.
    let mut t = Tile {
        face: Face::PositiveY,
        level: 0,
        xy: [0, 0],
    };
    for _ in 0..3 {
        leaves.remove(&t);
        leaves.extend(t.children());
        t = t.children()[3];
    }
    let registry = topology::Registry::new(&leaves, 3, N);
    let mut edges: BTreeMap<(Key, Key), (usize, i32)> = BTreeMap::new();
    let mut seen: BTreeMap<Key, (crate::meshes::SplitPosition, DVec3)> = BTreeMap::new();
    for tile in leaves {
        let k = ResourceKey {
            tile,
            grid: N,
            signature: registry.signature(tile),
        };
        let r = topology::resource(&c.document, &c, &k, &registry, 3).unwrap();
        for (i, key) in r.keys.iter().enumerate() {
            let coordinate = r.coordinates.as_ref().unwrap()[i];
            let normal = DVec3::from(r.normals[i].map(f64::from));
            if let Some((p, n)) = seen.insert(*key, (coordinate, normal)) {
                assert_eq!(
                    p, coordinate,
                    "shared keys must reconstruct bit-identically on the GPU"
                );
                assert_eq!(n, normal, "shared normals must use one global frame");
            }
            let reconstructed = DVec3::from(coordinate.high.map(f64::from))
                + DVec3::from(coordinate.low.map(f64::from));
            let d = DVec3::from(key.map(|v| v as f64)).normalize();
            let expected =
                d * (1. + Sampler::new(&c.document).unwrap().height(d) / c.document.radius());
            assert!((reconstructed - expected).length() < 5e-15);
        }
        for tri in &r.asset.geometry.triangles {
            let p = tri.map(|i| DVec3::from(r.keys[i as usize].map(|v| v as f64)).normalize());
            assert!(
                (p[1] - p[0]).cross(p[2] - p[0]).dot(p[0]) > 0.,
                "inverted/degenerate face"
            );
            for i in 0..3 {
                let a = r.keys[tri[i] as usize];
                let b = r.keys[tri[(i + 1) % 3] as usize];
                let (key, sign) = if a < b { ((a, b), 1) } else { ((b, a), -1) };
                let x = edges.entry(key).or_default();
                x.0 += 1;
                x.1 += sign;
            }
        }
    }
    assert!(
        edges.values().all(|v| *v == (2, 0)),
        "open seam, T junction or duplicate face"
    );
}

#[test]
fn planet_orbit_reuses_samples_without_changing_stitched_payload() {
    let mut c = config();
    c.document.operations.push(Relief::Raise {
        direction: [1., 1., 0.],
        angular_radius_degrees: 60.,
        height_meters: 1000.,
    });
    let mut leaves: BTreeSet<_> = FACES
        .into_iter()
        .map(|face| Tile {
            face,
            level: 0,
            xy: [0, 0],
        })
        .collect();
    let tile = Tile {
        face: Face::PositiveY,
        level: 0,
        xy: [0, 0],
    };
    let registry = topology::Registry::new(&leaves, 3, N);
    let key = ResourceKey {
        tile,
        grid: N,
        signature: registry.signature(tile),
    };
    let old = topology::resource(&c.document, &c, &key, &registry, 3).unwrap();
    let neighbor = Tile {
        face: Face::PositiveX,
        level: 0,
        xy: [0, 0],
    };
    leaves.remove(&neighbor);
    leaves.extend(neighbor.children());
    let registry = topology::Registry::new(&leaves, 3, N);
    let key = ResourceKey {
        tile,
        grid: N,
        signature: registry.signature(tile),
    };
    let fresh = topology::resource(&c.document, &c, &key, &registry, 3).unwrap();
    let reused =
        topology::resource_reusing(&c.document, &c, &key, &registry, 3, Some(&old)).unwrap();
    assert!(reused.sample_hits > 900);
    assert_eq!(fresh.asset.geometry, reused.asset.geometry);
    assert_eq!(fresh.normals, reused.normals);
    assert_eq!(fresh.coordinates, reused.coordinates);
    assert_eq!(fresh.keys, reused.keys);
    assert_eq!(fresh.normal_levels, reused.normal_levels);
}

#[test]
fn planet_orbit_clip_and_basis_are_finite_at_surface_globe_and_astronomical_zoom() {
    let c = config();
    for direction in [[0., 1., 0.], [0., -1., 0.], [1., 0., 0.], [0., 0., 1.]] {
        let a = Anchor::new(
            &c.document,
            Pose {
                direction,
                forward: if direction[2] == 1. {
                    [1., 0., 0.]
                } else {
                    [0., 0., -1.]
                },
                altitude_meters: 0.01,
            },
        )
        .unwrap();
        for distance_meters in [0., 1., 1000., 100_000., c.document.radius() * 3., 1e18] {
            let s = Settings {
                distance_meters,
                ..Default::default()
            };
            let v = View::new(
                &c.document,
                a,
                &s,
                1.62,
                c.document.radius() + 10.,
                1.,
                &Source::new(&c.document, N, Sampler::new(&c.document).unwrap()).unwrap(),
            )
            .unwrap();
            assert!((v.basis.determinant() - 1.).abs() < 1e-12);
            assert!(v.eye.is_finite() && v.center.is_finite());
            v.clip.validate().unwrap();
            if distance_meters > c.document.radius() * 2. {
                let p = crate::picking::view_projection_clip(
                    &crate::model::Camera {
                        position: [0.; 3],
                        target: [0., 0., -1.],
                        fov_degrees: 60.,
                    },
                    1280,
                    800,
                    Some(v.clip),
                );
                let q = p * v.center.as_vec3().extend(1.);
                assert!((q.x / q.w).abs() < 1e-5 && (q.y / q.w).abs() < 1e-5);
                assert!(
                    (0. ..1.).contains(&(q.z / q.w)),
                    "globe clipped at {distance_meters}"
                );
            }
        }
    }
}

#[test]
fn planet_orbit_keeps_physics_scene_identity_and_rejects_invalid_settings_atomically() {
    let mut e = engine();
    let scene = serde_json::to_value(&e.scene).unwrap();
    let physics = e.inspect_planet_runtime();
    e.execute(
        Command::SetPlanetView {
            view: Some(Settings {
                distance_meters: 20_000_000.,
                yaw_degrees: 1e300,
                ..Default::default()
            }),
        }
        .into(),
    )
    .unwrap();
    assert!(
        e.inspect_planet_view()["settings"]["yaw_degrees"]
            .as_f64()
            .unwrap()
            < 360.
    );
    for z in [-1., 0.] {
        e.execute(
            Command::SetPlayerViewInput {
                input: crate::physics::Input {
                    x: 0.,
                    z,
                    ..Default::default()
                },
                control: None,
            }
            .into(),
        )
        .unwrap();
    }
    assert_eq!(e.inspect_planet_runtime(), physics);
    assert_eq!(serde_json::to_value(&e.scene).unwrap(), scene);
    let before = e.inspect_planet_view();
    let revision = e.view_revision;
    for view in [
        Settings {
            distance_meters: f64::NAN,
            ..Default::default()
        },
        Settings {
            max_tiles: 5,
            ..Default::default()
        },
        Settings {
            distance_meters: 1e19,
            ..Default::default()
        },
    ] {
        assert!(
            e.execute(Command::SetPlanetView { view: Some(view) }.into())
                .is_err()
        );
        assert_eq!(before, e.inspect_planet_view());
        assert_eq!(revision, e.view_revision);
    }
    assert!(
        e.execute(Command::ZoomPlanetView { factor: 0. }.into())
            .is_err()
    );
    assert!(
        e.execute(Command::SetPlanetRuntime { config: None }.into())
            .is_err()
    );
    e.execute(Command::SetPlanetView { view: None }.into())
        .unwrap();
    assert_eq!(e.inspect_planet_runtime(), physics);
    assert!(e.render_frame().clip.is_none());
}

#[test]
fn planet_orbit_warm_placement_does_not_rebuild_geometry_and_budget_retains_coverage() {
    let c = config();
    let mut a = atlas(
        &c,
        Settings {
            distance_meters: 20_000_000.,
            max_tiles: 6,
            ..Default::default()
        },
    );
    let builds = a.builds;
    let plans = a.plans;
    let assets: Vec<_> = a.meshes.assets.iter().map(|x| x.geometry.clone()).collect();
    let mut e = engine();
    e.execute(
        Command::SetPlanetView {
            view: Some(a.settings.clone()),
        }
        .into(),
    )
    .unwrap();
    let mut frame = e.render_frame();
    a.apply(&mut frame);
    let generation = frame.meshes.geometry_generation;
    a.settings.yaw_degrees += 0.1;
    a.update(&c.document, &c, anchor(&c), 1.62, 1.).unwrap();
    assert_eq!(builds, a.builds);
    assert_eq!(plans, a.plans);
    assert!(
        assets
            .iter()
            .zip(&a.meshes.assets)
            .all(|(g, a)| Arc::ptr_eq(g, &a.geometry))
    );
    a.apply(&mut frame);
    assert_eq!(frame.meshes.geometry_generation, generation);
    a.settings.yaw_degrees += 180.;
    a.update(&c.document, &c, anchor(&c), 1.62, 1.).unwrap();
    assert_eq!(
        a.plans,
        plans + 1,
        "half turn must not alias to a small angle"
    );
    a.settings.distance_meters = 0.;
    a.update(&c.document, &c, anchor(&c), 1.62, 1.).unwrap();
    assert_eq!(a.resources.len(), 6);
    assert!(a.limited);
    assert_eq!(a.inspect()["all_six_faces_covered"], true);
}

#[test]
fn planet_orbit_source_bounds_enclose_relief_not_only_samples() {
    let mut c = config();
    c.document.operations = vec![
        Relief::Noise {
            seed: 71,
            amplitude_meters: 400.,
            wavelength_meters: 23_000.,
            octaves: 4,
            persistence: 0.5,
        },
        Relief::Raise {
            direction: [0., 1., 0.],
            angular_radius_degrees: 0.3,
            height_meters: 2000.,
        },
    ];
    let sampler = Sampler::new(&c.document).unwrap();
    for face in FACES {
        for level in [0, 3, 12] {
            let t = Tile {
                face,
                level,
                xy: if level == 0 {
                    [0, 0]
                } else {
                    [1u32 << (level - 1); 2]
                },
            };
            let bounds = global_bounds(&c.document, t);
            let lo = bounds[0];
            let hi = bounds[7];
            for y in 0..=12 {
                for x in 0..=12 {
                    let d = t.direction([f64::from(x) / 12., f64::from(y) / 12.]);
                    let p = d * (c.document.radius() + sampler.height(d));
                    assert!((p - lo).min_element() > -1e-6 && (hi - p).min_element() > -1e-6);
                }
            }
        }
    }
}

#[test]
fn planet_orbit_frame_fits_narrow_viewport_and_fov_without_moving_player() {
    let mut e = engine();
    let physics = e.inspect_planet_runtime();
    e.execute(
        Command::SetPlanetView {
            view: Some(Settings {
                distance_meters: 20_000_000.,
                fov_degrees: 10.,
                viewport: [64, 800],
                max_tiles: 6,
                ..Default::default()
            }),
        }
        .into(),
    )
    .unwrap();
    let framed = e.execute(Command::FramePlanetView.into()).unwrap();
    let distance = glam::DVec3::from(
        serde_json::from_value::<[f64; 3]>(framed["planet_center_view_meters"].clone()).unwrap(),
    )
    .length();
    let radius = config().document.radius();
    let tangent = radius / (distance * distance - radius * radius).sqrt();
    assert!(tangent < 5f64.to_radians().tan() * (64. / 800.) * 0.9);
    assert_eq!(physics, e.inspect_planet_runtime());
}

#[test]
fn planet_orbit_inherits_density_and_finite_material_choice_keeps_geometry() {
    let mut c = config();
    c.material=serde_json::from_value(serde_json::json!({"pattern":{"frequency":[1,1,1],"low":[0.01,0.01,0.01],"high":[0.02,0.02,0.02],"bump":0,"warp":0,"octaves":1,"seed":7,"contrast":1}})).unwrap();
    let mut light = RoomRecipe::default().generate().unwrap().light;
    light.direct_intensity = 4.;
    light.atmosphere = Some(crate::atmosphere::Atmosphere {
        density: 0.02,
        sun_strength: 0.,
        ..Default::default()
    });
    let mut a = Atlas::new(
        &c,
        Settings {
            tile_subdivisions: 32,
            max_tiles: 192,
            ..Default::default()
        },
        anchor(&c),
        ActorView {
            size: [0.6, 1.8, 0.6],
            color: [1.; 3],
            eye: 1.62,
        },
        &light,
        1.,
        Sampler::new(&c.document).unwrap(),
    )
    .unwrap();
    assert_eq!(a.optical_depth(), f64::from(0.02f32) * 8000.);
    assert!(
        a.cheap_materials > 0,
        "fixture must exercise material substitution"
    );
    let geometry: Vec<_> = a.meshes.assets.iter().map(|a| a.geometry.clone()).collect();
    a.settings.atmosphere_optimization = false;
    a.update(&c.document, &c, anchor(&c), 1.62, 1.).unwrap();
    assert_eq!(a.cheap_materials, 0);
    for (old, new) in geometry.iter().zip(&a.meshes.assets) {
        assert!(Arc::ptr_eq(old, &new.geometry));
    }
    light.atmosphere.as_mut().unwrap().density = 0.001;
    a.set_light(&light);
    assert_eq!(a.optical_depth(), f64::from(0.001f32) * 8000.);
    a.settings.atmosphere_optical_depth = Some(0.3);
    assert_eq!(a.optical_depth(), 0.3);
}

#[test]
fn planet_orbit_submission_groups_retain_unchanged_payloads() {
    let c = config();
    let a = atlas(
        &c,
        Settings {
            max_tiles: 96,
            ..Default::default()
        },
    );
    let repeated = topology::groups(&a.resources, 8, &a.body, &a, a.anchor, &a.view).unwrap();
    assert_eq!(repeated.built, 0);
    for (before, after) in a.meshes.assets.iter().zip(&repeated.assets) {
        assert!(Arc::ptr_eq(&before.geometry, &after.geometry));
    }
    let mut partial = a.resources.clone();
    partial.pop_first();
    let next = topology::groups(&partial, 8, &a.body, &a, a.anchor, &a.view).unwrap();
    let retained = next
        .assets
        .iter()
        .filter(|after| {
            a.meshes
                .assets
                .iter()
                .any(|before| Arc::ptr_eq(&before.geometry, &after.geometry))
        })
        .count();
    assert!(retained > 1, "unaffected groups must retain their payloads");
    assert_eq!(
        next.assets
            .iter()
            .take(next.groups.len())
            .map(|a| a.geometry.triangles.len())
            .sum::<usize>(),
        partial
            .values()
            .map(|r| r.asset.geometry.triangles.len())
            .sum::<usize>()
    );
}

#[test]
fn planet_orbit_canonical_cpu_frame_preserves_payload_and_restores_expansion() {
    let c = config();
    let a = atlas(
        &c,
        Settings {
            distance_meters: 4.,
            max_tiles: 48,
            ..Default::default()
        },
    );
    let mut reference = crate::meshes::Frame::default();
    reference.update(&a.meshes, 0);
    reference.apply_smooth_normals(&a.normals).unwrap();
    reference.apply_planet_coordinates(&a.coordinates).unwrap();
    let mut current = crate::meshes::Frame::default();
    current.update_planet_canonical(&a.meshes, &a.normals, &a.coordinates);
    assert!(current.canonical_planet);
    assert_eq!(current.triangle_count(), reference.triangle_count());
    assert!(current.resident_vertex_count() * 10 < reference.resident_vertex_count());
    for (new, old) in current.draws.iter().zip(&reference.draws) {
        assert_eq!(new.instance, old.instance);
        assert_eq!(new.asset, old.asset);
        assert_eq!(new.range.len(), old.range.len());
        assert_eq!(new.world, old.world);
        assert_eq!(new.color, old.color);
        let expanded = current.expanded_planet_asset(new.asset);
        assert!(
            bytemuck::cast_slice::<_, u8>(&expanded)
                == bytemuck::cast_slice::<_, u8>(&reference.shared_vertices[old.range.clone()]),
            "expanded payload differs for asset {}",
            new.asset
        );
    }
    let generation = current.geometry_generation;
    current.update_planet_canonical(&a.meshes, &a.normals, &a.coordinates);
    assert_eq!(
        current.geometry_generation, generation,
        "warm source must remain cached"
    );
    let mut edited = a.meshes.clone();
    edited.assets[0].material.reflectance = Some(crate::materials::Reflectance {
        roughness: 0.7,
        f0: 0.04,
        metallic: 0.,
    });
    current.update_planet_canonical(&edited, &a.normals, &a.coordinates);
    assert_ne!(current.geometry_generation, generation);
    assert_eq!(current.materials[1], edited.assets[0].material);
    current.apply_render_placements(&a.placements).unwrap();
    let mut changed_normals = a.normals.clone();
    let n = &mut Arc::make_mut(&mut changed_normals[0])[0];
    *n = n.map(|x| -x);
    current.apply_smooth_normals(&changed_normals).unwrap();
    assert!(Arc::ptr_eq(
        current.normal_overrides[0].as_ref().unwrap(),
        &changed_normals[0]
    ));
    let mut changed_coordinates = a.coordinates.clone();
    Arc::make_mut(changed_coordinates[0].as_mut().unwrap())[0].low[0] += 1e-10;
    current
        .apply_planet_coordinates(&changed_coordinates)
        .unwrap();
    assert!(Arc::ptr_eq(
        current.planet_coordinates[0].as_ref().unwrap(),
        changed_coordinates[0].as_ref().unwrap()
    ));
    assert_eq!(
        current.draws.iter().map(|d| d.world).collect::<Vec<_>>(),
        a.placements
    );
    current.update(&a.meshes, 0);
    current.apply_smooth_normals(&a.normals).unwrap();
    current.apply_planet_coordinates(&a.coordinates).unwrap();
    assert!(!current.canonical_planet);
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&current.shared_vertices),
        bytemuck::cast_slice::<_, u8>(&reference.shared_vertices)
    );
    current.update_planet_canonical(&a.meshes, &a.normals, &a.coordinates);
    current.update(&crate::meshes::Document::default(), 0);
    assert!(current.draws.is_empty());
    assert_eq!(current.triangle_count(), 0);
    assert_eq!(current.materials.len(), 1);
    edited.assets[0].material.procedural = Some(Arc::new(
        crate::materials::recipe::Procedural::runtime(crate::materials::recipe::Recipe {
            unique_instances: true,
            ..Default::default()
        }),
    ));
    current.update_planet_canonical(&edited, &a.normals, &a.coordinates);
    assert!(
        !current.canonical_planet,
        "unique material recipes retain their derivation policy"
    );
    let draw = current.draws.iter().find(|d| d.asset == 0).unwrap();
    assert!(draw.material as usize > edited.assets.len());
    assert!(
        !current.materials[draw.material as usize]
            .procedural
            .as_ref()
            .unwrap()
            .recipe
            .unique_instances
    );
}

#[test]
fn planet_orbit_warm_gate_tracks_actual_collision_adjusted_eye() {
    let c = config();
    let mut a = atlas(
        &c,
        Settings {
            distance_meters: 125.,
            max_tiles: 48,
            ..Default::default()
        },
    );
    let plans = a.plans;
    a.update(&c.document, &c, anchor(&c), 1.62, 0.5).unwrap();
    assert!(
        a.plans > plans,
        "camera collision moved the eye without moving the actor or settings"
    );
}
