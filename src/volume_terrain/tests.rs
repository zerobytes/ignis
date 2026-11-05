use super::*;
fn plane() -> Document {
    Document {
        version: 1,
        seed: 19,
        base: Base::Plane { height: 0.25 },
        noise_domain_scale: [1.; 3],
        vertical_strength: 1.,
        octaves: vec![],
        warp: None,
        terraces: None,
        shelf: None,
        floor: None,
    }
}
#[test]
fn plane_sign_normal_and_exact_extraction() {
    let d = plane();
    let s = Sampler::new(&d).unwrap();
    assert_eq!(s.sample([0., -1., 0.]).unwrap(), 1.25);
    assert_eq!(s.normal([0.; 3], 0.001).unwrap(), [0., 1., 0.]);
    let c = compile(
        &d,
        &Grid {
            origin: [-2.; 3],
            cells: [4; 3],
            spacing: 1.,
            vertical_spacing: None,
        },
    )
    .unwrap();
    assert_eq!(c.geometry.positions.len(), 25);
    assert_eq!(c.geometry.triangles.len(), 32);
    for p in &c.geometry.positions {
        assert_eq!(p[1], 2.25);
    }
    for t in &c.geometry.triangles {
        let p = t.map(|i| DVec3::from_array(c.geometry.positions[i as usize].map(f64::from)));
        assert!((p[1] - p[0]).cross(p[2] - p[0]).y > 0.);
    }
}
#[test]
fn sphere_is_closed_and_outward() {
    let mut d = plane();
    d.base = Base::Sphere {
        center: [0.; 3],
        radius: 1.13,
    };
    let c = compile(
        &d,
        &Grid {
            origin: [-2.; 3],
            cells: [16; 3],
            spacing: 0.25,
            vertical_spacing: None,
        },
    )
    .unwrap();
    let mut edges = std::collections::BTreeMap::new();
    for t in &c.geometry.triangles {
        let p = t.map(|i| {
            DVec3::from_array(c.geometry.positions[i as usize].map(f64::from)) - DVec3::splat(2.)
        });
        assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(p[0] + p[1] + p[2]) > 0.);
        for [a, b] in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
            let key = if a < b { [a, b] } else { [b, a] };
            *edges.entry(key).or_insert(0) += 1;
        }
    }
    assert!(edges.values().all(|&n| n == 2));
}
#[test]
fn original_shader_snap_and_overhang_allow_multiple_vertical_crossings() {
    let mut d = plane();
    d.base = Base::Plane { height: 0. };
    d.terraces = Some(Terraces {
        spacing: 8.,
        strength: 3.,
        overhang: 20.,
        sharpness: 16.,
        warp: 0.,
        phase: 0.,
    });
    let s = Sampler::new(&d).unwrap();
    let mut crossings = 0;
    let mut previous = s.sample([0., -8., 0.]).unwrap() > 0.;
    for i in 1..=1600 {
        let positive = s.sample([0., -8. + i as f64 * 0.01, 0.]).unwrap() > 0.;
        if positive != previous {
            crossings += 1;
        }
        previous = positive;
    }
    assert!(crossings >= 3, "{crossings}");
}
#[test]
fn seed_determinism_roundtrip_validation_and_budget() {
    let mut d = plane();
    d.octaves.push(Octave {
        frequency: 0.1,
        amplitude: 4.,
        rotation_degrees: [13., 27., 19.],
    });
    let p = [1.3, 2.1, -9.7];
    let a = Sampler::new(&d).unwrap().sample(p).unwrap();
    let bytes = encode(&d).unwrap();
    let loaded = decode(&bytes).unwrap();
    assert_eq!(a, Sampler::new(&loaded).unwrap().sample(p).unwrap());
    d.seed += 1;
    assert_ne!(a, Sampler::new(&d).unwrap().sample(p).unwrap());
    assert!(
        compile(
            &d,
            &Grid {
                origin: [0.; 3],
                cells: [200; 3],
                spacing: 1.,
                vertical_spacing: None,
            }
        )
        .is_err()
    );
    assert!(Sampler::new(&d).unwrap().sample([f64::NAN; 3]).is_err());
    let mut extra = bytes;
    extra.push(0);
    assert!(decode(&extra).is_err());
    d.terraces = Some(Terraces {
        spacing: 0.,
        strength: 1.,
        overhang: 0.,
        sharpness: 16.,
        warp: 0.,
        phase: 0.,
    });
    assert!(d.validate().is_err());
}
#[test]
fn integer_plane_has_no_degeneracy_and_adjacent_boundaries_match() {
    let mut d = plane();
    d.base = Base::Plane { height: 0. };
    let a = compile(
        &d,
        &Grid {
            origin: [0., -2., 0.],
            cells: [4; 3],
            spacing: 1.,
            vertical_spacing: None,
        },
    )
    .unwrap();
    let b = compile(
        &d,
        &Grid {
            origin: [4., -2., 0.],
            cells: [4; 3],
            spacing: 1.,
            vertical_spacing: None,
        },
    )
    .unwrap();
    let edge_a: Vec<_> = a
        .geometry
        .positions
        .iter()
        .filter(|p| p[0] == 4.)
        .map(|p| [p[1], p[2]])
        .collect();
    let edge_b: Vec<_> = b
        .geometry
        .positions
        .iter()
        .filter(|p| p[0] == 0.)
        .map(|p| [p[1], p[2]])
        .collect();
    assert_eq!(edge_a, edge_b);
}

#[test]
fn gpu_density_matches_reference_for_all_operations_and_radial_base() {
    let instance = wgpu::Instance::new(&Default::default());
    let r = pollster::block_on(crate::render::Renderer::new(&instance, None))
        .expect("Headless GPU required for density backend test");
    let gpu = super::gpu::Gpu::new(&r.device);
    let grid = Grid {
        origin: [-4.; 3],
        cells: [16; 3],
        spacing: 0.5,
        vertical_spacing: Some(0.25),
    };
    let mut d = plane();
    d.octaves = vec![
        Octave {
            frequency: 0.17,
            amplitude: 3.,
            rotation_degrees: [13., 27., 19.],
        },
        Octave {
            frequency: 0.47,
            amplitude: 0.4,
            rotation_degrees: [31., 7., 23.],
        },
    ];
    d.version = 2;
    d.noise_domain_scale = [0.5, 0.8, 0.7];
    d.warp = Some(Warp {
        frequency: 0.13,
        amplitude: 1.7,
    });
    d.terraces = Some(Terraces {
        spacing: 3.7,
        strength: 2.1,
        overhang: 1.3,
        sharpness: 16.,
        warp: 0.27,
        phase: 0.13,
    });
    d.shelf = Some(Shelf {
        altitude: 0.7,
        half_thickness: 0.8,
        strength: 3.,
        blend: 0.6,
    });
    d.floor = Some(Floor {
        altitude: -2.,
        transition: 1.,
        strength: 4.,
    });
    for base in [
        Base::Plane { height: 0.25 },
        Base::Sphere {
            center: [0., -1., 0.],
            radius: 2.7,
        },
    ] {
        d.base = base;
        let (n, values) = gpu.sample(&r.device, &r.queue, &d, &grid).unwrap();
        let sampler = Sampler::new(&d).unwrap();
        let mut max_error = 0f64;
        for z in 0..n[2] {
            for y in 0..n[1] {
                for x in 0..n[0] {
                    let p = std::array::from_fn(|i| {
                        grid.origin[i] + [x, y, z][i] as f64 * grid.step(i)
                    });
                    let cpu = sampler.sample(p).unwrap();
                    let error = (cpu - values[(z * n[1] + y) * n[0] + x]).abs();
                    max_error = max_error.max(error);
                }
            }
        }
        assert!(
            max_error < 0.001,
            "GPU/reference maximum density difference {max_error}"
        );
        let c = gpu.compile(&r.device, &r.queue, &d, &grid).unwrap();
        assert!(c.report["gpu_generation"].as_bool().unwrap());
    }
}

#[test]
fn empty_density_blocks_are_inspectable_without_invalid_mesh_publication() {
    let d = plane();
    let grid = Grid {
        origin: [0., 20., 0.],
        cells: [4; 3],
        spacing: 1.,
        vertical_spacing: None,
    };
    let c = compile(&d, &grid).unwrap();
    assert_eq!(c.report["empty"], true);
    assert!(c.geometry.triangles.is_empty());
    assert!(
        publication(Publication {
            document: d,
            grid,
            asset: "empty".into(),
            instance: "empty".into(),
            position: [0.; 3],
            color: [1.; 3],
            material: Default::default(),
            collision: None
        })
        .is_err()
    );
}

#[test]
fn horizontal_and_vertical_field_scales_preserve_plane_density_under_mapping() {
    let mut a = plane();
    a.base = Base::Plane { height: 0. };
    a.octaves = vec![Octave {
        frequency: 0.03,
        amplitude: 7.,
        rotation_degrees: [19., 37., 11.],
    }];
    a.warp = Some(Warp {
        frequency: 0.05,
        amplitude: 2.,
    });
    a.terraces = Some(Terraces {
        spacing: 8.,
        strength: 4.,
        overhang: 1.6,
        sharpness: 16.,
        warp: 0.15,
        phase: 0.,
    });
    a.floor = Some(Floor {
        altitude: -30.,
        transition: 3.,
        strength: 40.,
    });
    let mut b = a.clone();
    b.version = 2;
    b.noise_domain_scale = [0.02, 0.125, 0.02];
    b.vertical_strength *= 0.125;
    b.terraces.as_mut().unwrap().spacing /= 0.125;
    b.floor.as_mut().unwrap().altitude /= 0.125;
    b.floor.as_mut().unwrap().transition /= 0.125;
    let sa = Sampler::new(&a).unwrap();
    let sb = Sampler::new(&b).unwrap();
    for x in -8..8 {
        for y in -8..8 {
            let p = [x as f64 * 3.1, y as f64 * 0.71, 7.3];
            let q = std::array::from_fn(|i| p[i] / b.noise_domain_scale[i]);
            assert!((sa.sample(p).unwrap() - sb.sample(q).unwrap()).abs() < 1e-10);
        }
    }
    b.version = 1;
    assert!(b.validate().is_err());
    b.version = 2;
    b.noise_domain_scale[0] = 0.;
    assert!(b.validate().is_err());
    let bytes = encode(&a).unwrap();
    let loaded = decode(&bytes).unwrap();
    assert_eq!(bytes, encode(&loaded).unwrap());
    assert!(
        serde_json::to_value(&a)
            .unwrap()
            .get("noise_domain_scale")
            .is_none()
    );
}

#[test]
fn anisotropic_vertical_grid_plane_extraction() {
    let d = plane();
    let c = compile(
        &d,
        &Grid {
            origin: [-4., -2., -4.],
            cells: [4, 16, 4],
            spacing: 2.,
            vertical_spacing: Some(0.25),
        },
    )
    .unwrap();
    assert_eq!(c.geometry.positions.len(), 25);
    for p in &c.geometry.positions {
        assert_eq!(p[1], 2.25);
    }
}
