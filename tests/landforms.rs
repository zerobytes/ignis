//! Material risks: a visually plausible mesh must not seal openings or lose authored data.
use ignis::{
    landforms::{self, Document, Edit, Piece, Query, Ray, Section},
    world_space::{Coordinates, Position},
};
use serde_json::json;

fn origin() -> Position {
    Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [0.; 3],
    })
    .unwrap()
}
fn preset(value: serde_json::Value) -> Document {
    let q: Query = serde_json::from_value(
        json!({"kind":"preset","id":"form","origin":origin(),"preset":value}),
    )
    .unwrap();
    serde_json::from_value(landforms::query(q).unwrap()["document"].clone()).unwrap()
}
fn ray(o: [f64; 3], d: [f64; 3]) -> Ray {
    Ray {
        origin: o,
        direction: d,
        max_distance_meters: 100.,
    }
}
fn surface(document: &Document, ray: Ray) -> Vec<landforms::Hit> {
    landforms::compile(document)
        .unwrap()
        .intersections(&ray)
        .unwrap()
}
fn closed(document: &Document) {
    let c = landforms::compile(document).unwrap();
    let t = ignis::mesh_topology::Topology::inspect(&c.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);
    assert!(t.isolated_vertices.is_empty());
    assert!(
        c.parts
            .iter()
            .all(|p| p.closed_oriented && p.signed_volume_cubic_meters > 0.)
    );
}

#[test]
fn actual_overhangs_vertical_cliffs_caps_and_cavities_survive_native_triangulation() {
    let wave = preset(
        json!({"kind":"wave_cliff","height_meters":14,"depth_meters":8,"inset_meters":2,"lip_meters":4,"length_meters":10,"segments":24}),
    );
    closed(&wave);
    let h = surface(&wave, ray([2., 40., 5.], [0., -1., 0.]));
    assert_eq!(h.len(), 2);
    assert_eq!(h[0].point[1], 14.);
    assert!(h[1].point[1] > 1. && h[1].point[1] < 14.);
    let canyon = preset(
        json!({"kind":"canyon","height_meters":10,"outer_width_meters":20,"floor_width_meters":6,"base_depth_meters":2,"length_meters":12}),
    );
    closed(&canyon);
    let floor = surface(&canyon, ray([0., 30., 6.], [0., -1., 0.]));
    assert_eq!(floor.len(), 2);
    assert_eq!(floor[0].point[1], 0.);
    let wall = surface(&canyon, ray([0., 5., 6.], [1., 0., 0.]));
    assert_eq!(wall.len(), 2);
    assert_eq!(wall[0].point[0], 3.);
    assert!(wall[0].normal[0] < -0.99);
    let top = surface(&canyon, ray([8., 30., 6.], [0., -1., 0.]));
    assert_eq!(top[0].point[1], 10.);
    assert!(top[0].normal[1] > 0.99);
    let peak = preset(
        json!({"kind":"bent_peak","height_meters":14,"radius_meters":4,"lean":[10,0],"levels":24,"sides":24,"seed":4,"roughness":0}),
    );
    closed(&peak);
    let h = surface(&peak, ray([8., 30., 0.], [0., -1., 0.]));
    assert_eq!(h.len(), 2);
    assert!(h.iter().all(|h| h.point[1] > 10.));
    let arch = preset(
        json!({"kind":"arch","outer_radius_meters":6,"inner_radius_meters":3,"leg_height_meters":3,"depth_meters":4,"segments":20}),
    );
    closed(&arch);
    assert!(surface(&arch, ray([0., 1., -2.], [0., 0., 1.])).is_empty());
    assert_eq!(surface(&arch, ray([0., 7., -2.], [0., 0., 1.])).len(), 2);
    let tunnel = preset(
        json!({"kind":"tunnel","radius_meters":4,"inner_ratio":0.6,"length_meters":10,"sides":32,"seed":7,"roughness":0}),
    );
    closed(&tunnel);
    assert!(surface(&tunnel, ray([0., 0., -2.], [0., 0., 1.])).is_empty());
    assert_eq!(surface(&tunnel, ray([3.5, 0., -2.], [0., 0., 1.])).len(), 2);
    assert_eq!(surface(&tunnel, ray([10., 0., 5.], [-1., 0., 0.])).len(), 4);
    let hoodoo = preset(
        json!({"kind":"hoodoo","height_meters":12,"radius_meters":3,"sides":16,"seed":9,"roughness":0}),
    );
    closed(&hoodoo);
    let h = surface(&hoodoo, ray([3., 20., 0.], [0., -1., 0.]));
    assert!(
        h.iter()
            .any(|h| (h.point[1] - 7.8).abs() < 1e-5 && h.normal[1] < -0.99)
    );
    let mesa = preset(
        json!({"kind":"mesa","height_meters":10,"radius_meters":4,"foot_radius_meters":6,"foot_height_meters":3,"sides":8,"seed":7,"roughness":0.2}),
    );
    closed(&mesa);
    let h = surface(&mesa, ray([0., 30., 0.], [0., -1., 0.]));
    assert_eq!(h[0].point[1], 10.);
}

#[test]
fn manual_authorship_codec_and_late_errors_preserve_source_and_format_boundaries() {
    let mut d = Document {
        domain: None,
        version: 1,
        space_version: 1,
        id: "authored".into(),
        revision: 1,
        origin: origin(),
        pieces: vec![Piece {
            id: "manual".into(),
            anchor: [0.; 3],
            axis: [0., 0., 1.],
            profile_x: [1., 0., 0.],
            profile: vec![
                [-4., -1.],
                [4., -1.],
                [4., 6.],
                [2., 6.],
                [2., 0.],
                [-2., 0.],
                [-2., 6.],
                [-4., 6.],
            ],
            sections: vec![
                Section {
                    radial_scales: Vec::new(),
                    at: 0.,
                    offset: [0.; 2],
                    scale: 1.,
                },
                Section {
                    radial_scales: Vec::new(),
                    at: 5.,
                    offset: [0.; 2],
                    scale: 1.,
                },
            ],
            inner_scale: None,
        }],
    };
    d.origin = Position::normalize(Coordinates {
        cell: [9_007_199_254_740_993, 0, i64::MIN],
        offset: [1.5, 0., 0.],
    })
    .unwrap();
    closed(&d);
    let bytes = landforms::encode(&d).unwrap();
    assert_eq!(landforms::decode(&bytes).unwrap(), d);
    assert_eq!(
        serde_json::from_str::<Document>(&serde_json::to_string(&d).unwrap()).unwrap(),
        d
    );
    let mut damaged = bytes.clone();
    *damaged.last_mut().unwrap() ^= 1;
    assert!(landforms::decode(&damaged).is_err());
    assert!(ignis::terrain::decode(&bytes).is_err());
    let old_terrain = include_bytes!("../examples/terrain-tools.ignis-terrain");
    let old = ignis::terrain::decode(old_terrain).unwrap();
    assert_eq!(ignis::terrain::encode(&old).unwrap(), old_terrain);
    let old = d.clone();
    assert!(
        landforms::edit(
            d.clone(),
            1,
            vec![
                Edit::SetSections {
                    id: "manual".into(),
                    sections: vec![
                        Section {
                            radial_scales: Vec::new(),
                            at: 0.,
                            offset: [0.; 2],
                            scale: 1.
                        },
                        Section {
                            radial_scales: Vec::new(),
                            at: 8.,
                            offset: [1., 0.],
                            scale: 1.
                        }
                    ]
                },
                Edit::Remove {
                    id: "missing".into()
                }
            ]
        )
        .is_err()
    );
    assert_eq!(d, old);
    let edited = landforms::edit(
        d.clone(),
        1,
        vec![Edit::SetSections {
            id: "manual".into(),
            sections: vec![
                Section {
                    radial_scales: Vec::new(),
                    at: 0.,
                    offset: [0.; 2],
                    scale: 1.,
                },
                Section {
                    radial_scales: Vec::new(),
                    at: 8.,
                    offset: [1., 0.],
                    scale: 1.,
                },
            ],
        }],
    )
    .unwrap();
    assert_eq!(edited.revision, 2);
    closed(&edited);
    assert_ne!(
        landforms::compile(&d).unwrap().source_sha256,
        landforms::compile(&edited).unwrap().source_sha256
    );
    assert!(
        landforms::edit(
            d.clone(),
            0,
            vec![Edit::Remove {
                id: "manual".into()
            }]
        )
        .is_err()
    );
    let mut invalid = d.clone();
    invalid.pieces[0].profile = vec![[-1., -1.], [1., 1.], [-1., 1.], [1., -1.]];
    assert!(landforms::compile(&invalid).is_err());
    let mut invalid = d.clone();
    invalid.pieces[0].inner_scale = Some(0.5);
    assert!(landforms::compile(&invalid).is_err());
    let mut invalid = d.clone();
    invalid.pieces[0].sections.insert(
        1,
        Section {
            radial_scales: Vec::new(),
            at: 0.,
            offset: [0.; 2],
            scale: 0.5,
        },
    );
    assert!(landforms::compile(&invalid).is_err());
    let mut invalid = d.clone();
    invalid.pieces[0].sections[0].scale = 0.;
    assert!(landforms::compile(&invalid).is_err());
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["origin"]["cell"] = json!([0, 0, 0]);
    assert!(serde_json::from_value::<Document>(raw).is_err());
    // Bound returned data as well as arithmetic work. A compact zigzag loft can
    // otherwise produce a large response without exceeding the triangle budget.
    let mut dense = d.clone();
    let mut piece = d.pieces[0].clone();
    piece.profile = vec![[-1., -1.], [1., -1.], [0., 1.]];
    piece.sections = (0..128)
        .map(|i| Section {
            radial_scales: Vec::new(),
            at: f64::from(i),
            offset: [if i % 2 == 0 { -5. } else { 5. }, 0.],
            scale: 1.,
        })
        .collect();
    dense.pieces = (0..32)
        .map(|i| {
            let mut p = piece.clone();
            p.id = format!("piece_{i}");
            p
        })
        .collect();
    let mut ray = ray([0., 0., -1.], [0., 0., 1.]);
    ray.max_distance_meters = 200.;
    assert!(
        landforms::compile(&dense)
            .unwrap()
            .intersections(&ray)
            .unwrap_err()
            .message
            .contains("budget")
    );
    dense.pieces.truncate(1);
    let result = landforms::query(Query::Raycast {
        document: dense,
        rays: vec![ray; 17],
    });
    assert!(
        result.is_err(),
        "Dense ray batch should reject without materializing an oversized response"
    );
    if let Err(error) = result {
        assert!(error.message.contains("budget"));
    }
}
