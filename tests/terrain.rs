//! Critical risks: authoring loss/codec, global seams and atomic representation changes.
use ignis::{
    terrain::{self, Base, Document, Edit, Operation, Patch},
    world_space::{Coordinates, Position},
};
use serde_json::json;

fn pos(x: f64, z: f64) -> Position {
    Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [x, 0., z],
    })
    .unwrap()
}
fn document() -> Document {
    Document {
        version: 1,
        space_version: 1,
        id: "world".into(),
        revision: 1,
        datum: pos(0., 0.),
        base: Base::Constant { height_meters: 0. },
        operations: vec![],
        water_level_meters: Some(0.),
    }
}
fn patch(origin: Position, cells: [u16; 2], spacing_meters: f64) -> Patch {
    Patch {
        origin,
        cells,
        spacing_meters,
    }
}

#[test]
fn canonical_authoring_global_seams_formats_and_failed_edits_preserve_source() {
    let mut d = document();
    d.operations.push(Operation::Noise {
        seed: u32::MAX,
        period_meters: 4096,
        octaves: 10,
        persistence: 0.5,
        amplitude_meters: 20.,
        style: terrain::NoiseKind::Smooth,
    });
    // Above2^53, negative cells and boundaries all use the same global lattice.
    let a = Position::normalize(Coordinates {
        cell: [9_007_199_254_740_993, 0, -9_007_199_254_740_993],
        offset: [1020., 0., 0.],
    })
    .unwrap();
    let left = terrain::bake(&d, patch(a, [4, 4], 1.)).unwrap();
    let b = a.translated([4., 0., 0.]).unwrap();
    let right = terrain::bake(&d, patch(b, [4, 4], 1.)).unwrap();
    for z in 0..5 {
        assert_eq!(
            left.heights[z * 5 + 4].to_bits(),
            right.heights[z * 5].to_bits()
        );
    }
    let reverse = terrain::bake(&d, patch(a, [4, 4], 1.)).unwrap();
    assert_eq!(reverse.heights, left.heights);
    let boundary = terrain::query(terrain::Query::Sample {
        document: d.clone(),
        points: vec![a, b],
    })
    .unwrap();
    assert_eq!(
        boundary["samples"][0]["height_meters"].as_f64().unwrap() as f32,
        left.heights[0]
    );
    let bytes = terrain::encode(&d).unwrap();
    assert_eq!(terrain::decode(&bytes).unwrap(), d);
    let mut spatial = d.clone();
    spatial.operations.extend([
        Operation::Raise {
            center: a,
            radius_meters: 8.,
            amount_meters: 2.,
        },
        Operation::Level {
            center: pos(-4., -5.),
            radius_meters: 8.,
            height_meters: 2.,
            slope: [0.1, 0.2],
            blend: 0.5,
        },
        Operation::Channel {
            points: vec![pos(-5., -5.), pos(5., 5.)],
            width_meters: 2.,
            bed_start_meters: 1.,
            bed_end_meters: -1.,
            blend: 1.,
        },
    ]);
    assert_eq!(
        terrain::decode(&terrain::encode(&spatial).unwrap()).unwrap(),
        spatial
    );
    assert_eq!(
        serde_json::from_str::<Document>(&serde_json::to_string(&spatial).unwrap()).unwrap(),
        spatial
    );
    let text = serde_json::to_string(&d).unwrap();
    assert_eq!(serde_json::from_str::<Document>(&text).unwrap(), d);
    let mut broken = bytes.clone();
    *broken.last_mut().unwrap() ^= 1;
    assert!(terrain::decode(&broken).is_err());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(terrain::decode(&trailing).is_err());
    let old = d.clone();
    assert!(
        terrain::edit(
            d.clone(),
            1,
            vec![
                Edit::Append {
                    operation: Operation::Raise {
                        center: a,
                        radius_meters: 10.,
                        amount_meters: 1.
                    }
                },
                Edit::Remove { index: 63 }
            ]
        )
        .is_err()
    );
    assert_eq!(d, old);
    assert!(
        terrain::edit(
            d.clone(),
            2,
            vec![Edit::SetWater {
                height_meters: None
            }]
        )
        .is_err()
    );
    let edited = terrain::edit(
        d.clone(),
        1,
        vec![
            Edit::SetWater {
                height_meters: None,
            },
            Edit::SetBase {
                base: Base::Samples {
                    patch: patch(a, [4, 4], 1.),
                    heights: left.heights,
                },
            },
            Edit::Remove { index: 0 },
        ],
    )
    .unwrap();
    assert_eq!(edited.revision, 2);
    assert_eq!(
        terrain::decode(&terrain::encode(&edited).unwrap()).unwrap(),
        edited
    );
    let samples = terrain::query(terrain::Query::Sample {
        document: edited.clone(),
        points: vec![a, a.translated([-1., 0., 0.]).unwrap()],
    })
    .unwrap();
    assert!(!samples["samples"][0]["height_meters"].is_null());
    assert!(samples["samples"][1]["height_meters"].is_null());
    let patched = terrain::edit(
        edited.clone(),
        2,
        vec![Edit::PatchSamples {
            start: [0, 0],
            size: [1, 1],
            heights: vec![80.],
        }],
    )
    .unwrap();
    assert_eq!(patched.revision, 3);
    assert_eq!(
        terrain::query(terrain::Query::Sample {
            document: patched,
            points: vec![a]
        })
        .unwrap()["samples"][0]["height_meters"],
        80.
    );
    let invalid = json!({"version":1,"space_version":1,"id":"x","revision":1,"datum":pos(0.,0.),"base":{"kind":"samples","patch":patch(pos(0.,0.),[1,1],1.),"heights":vec![0.;terrain::MAX_SAMPLES+1]},"operations":[]});
    assert!(serde_json::from_value::<Document>(invalid).is_err());
    let edge = Position::normalize(Coordinates {
        cell: [i64::MAX, 0, 0],
        offset: [1023., 0., 0.],
    })
    .unwrap();
    assert!(terrain::bake(&d, patch(edge, [2, 2], 1.)).is_err());
}

#[test]
fn sampled_surface_and_compilation_share_triangles_and_conservative_sampled_error() {
    let mut d = document();
    d.base = Base::Samples {
        patch: patch(pos(0., 0.), [2, 2], 1.),
        heights: vec![0., 0., 0., 0., 8., 0., 0., 0., 0.],
    };
    // Center peak must not disappear without an explicit nonzero certificate.
    let fine = terrain::compile(&d, patch(pos(0., 0.), [2, 2], 1.), 1).unwrap();
    let coarse = terrain::compile(&d, patch(pos(0., 0.), [2, 2], 1.), 2).unwrap();
    assert_eq!(fine.sampled_error_meters, 0.);
    assert_eq!(coarse.sampled_error_meters, 8.);
    assert_eq!(coarse.bounds[1][1], 8.); // bounds include the source, not only coarse vertices
    let sample = terrain::query(terrain::Query::Sample {
        document: d.clone(),
        points: vec![pos(0.75, 0.75)],
    })
    .unwrap();
    assert_eq!(sample["samples"][0]["height_meters"], 4.);
    for t in &fine.geometry.triangles {
        let [a, b, c] = t.map(|i| glam::Vec3::from(fine.geometry.positions[i as usize]));
        assert!((b - a).cross(c - a).y > 0.);
    }
    d.base = Base::Constant { height_meters: 10. };
    d.operations = vec![Operation::Channel {
        points: vec![pos(0., 0.), pos(10., 0.)],
        width_meters: 2.,
        bed_start_meters: 4.,
        bed_end_meters: 0.,
        blend: 1.,
    }];
    let sample = terrain::query(terrain::Query::Sample {
        document: d.clone(),
        points: vec![pos(0., 0.), pos(5., 0.), pos(10., 0.), pos(5., 3.)],
    })
    .unwrap();
    assert_eq!(
        sample["samples"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["height_meters"].as_f64().unwrap())
            .collect::<Vec<_>>(),
        vec![4., 2., 0., 10.]
    );
    // Freezing a procedural patch produces ordinary editable samples.
    let frozen = terrain::query(terrain::Query::Bake {
        document: d,
        patch: patch(pos(0., 0.), [10, 4], 1.),
    })
    .unwrap();
    let frozen: Document = serde_json::from_value(frozen["document"].clone()).unwrap();
    assert!(matches!(frozen.base, Base::Samples { .. }));
    assert!(frozen.operations.is_empty());
}

#[test]
fn coverage_replacement_is_atomic_and_clear_air_has_no_finite_cutoff() {
    use terrain::coverage::*;
    let parent = Node {
        id: "parent".into(),
        region: Region {
            origin: pos(-16., -16.),
            size_meters: 32,
        },
        center: pos(0., 0.),
        radius_meters: 100.,
        children: Some(["a", "b", "c", "d"].map(str::to_owned)),
        ready: true,
        children_seams_ready: true,
        approximation_error_meters: 10.,
    };
    let mut nodes = vec![parent];
    for (i, id) in ["a", "b", "c", "d"].into_iter().enumerate() {
        nodes.push(Node {
            id: id.into(),
            region: Region {
                origin: pos(-16. + 16. * ((i % 2) as f64), -16. + 16. * ((i / 2) as f64)),
                size_meters: 16,
            },
            center: pos(-8. + 16. * ((i % 2) as f64), -8. + 16. * ((i / 2) as f64)),
            radius_meters: 12.,
            children: None,
            ready: true,
            children_seams_ready: false,
            approximation_error_meters: 0.,
        });
    }
    let mut q = Query {
        viewer: pos(0., 0.),
        atmosphere: Atmosphere::Disabled,
        maximum_error_meters: 1.,
        representation_budget: 4,
        nodes,
    };
    q.nodes[4].ready = false;
    let p = plan(q.clone()).unwrap();
    assert_eq!(p.selected, vec!["parent"]);
    assert_eq!(p.requests, vec!["d"]);
    assert!(p.ready_for_supplied_roots);
    assert_eq!(p.horizon_meters, None);
    q.nodes[4].ready = true;
    let mut overlap = q.clone();
    overlap.nodes[4].region = overlap.nodes[1].region.clone();
    assert!(plan(overlap).is_err());
    let p = plan(q.clone()).unwrap();
    assert_eq!(p.selected, vec!["a", "b", "c", "d"]);
    q.representation_budget = 1;
    let p = plan(q.clone()).unwrap();
    assert_eq!(p.selected, vec!["parent"]);
    assert_eq!(p.quality_deferred, 1);
    q.nodes[0].ready = false;
    assert!(!plan(q.clone()).unwrap().ready_for_supplied_roots);
    q.nodes[0].ready = true;
    q.nodes[0].children_seams_ready = false;
    q.representation_budget = 4;
    assert_eq!(plan(q.clone()).unwrap().selected, vec!["parent"]);
    q.atmosphere = Atmosphere::Uniform {
        extinction_per_meter: 0.01,
        maximum_contrast: 1.,
        negligible_contrast: 0.01,
    };
    q.viewer = pos(1000., 0.);
    let p = plan(q.clone()).unwrap();
    assert!(p.selected.is_empty());
    assert_eq!(p.omitted_by_atmosphere_model, vec!["parent"]);
    assert!((p.horizon_meters.unwrap() - 460.51701859880916).abs() < 1e-9);
    q.atmosphere = Atmosphere::Uniform {
        extinction_per_meter: 0.,
        maximum_contrast: 1.,
        negligible_contrast: 0.01,
    };
    assert_eq!(plan(q.clone()).unwrap().selected, vec!["parent"]);
    q.nodes[4].children = Some(["parent", "a", "b", "c"].map(str::to_owned));
    assert!(plan(q).is_err());
}
