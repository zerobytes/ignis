//! Critical source compatibility, continuity and large publication atomicity.
use ignis::{
    engine::{Command, Engine},
    model::RoomRecipe,
    terrain::{
        self, Base, Document, Operation, Patch, Sampler,
        region::{Publication, Region},
    },
    world_space::{Coordinates, Position},
};
fn p(x: f64, z: f64) -> Position {
    Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [x, 0., z],
    })
    .unwrap()
}
fn doc() -> Document {
    Document {
        version: 2,
        space_version: 1,
        id: "ridge".into(),
        revision: 1,
        datum: p(0., 0.),
        base: Base::Constant { height_meters: 0. },
        operations: vec![Operation::Ridge {
            center: p(0., 0.),
            azimuth_degrees: 0.,
            width_meters: 4000.,
            depth_meters: 1600.,
            height_meters: 850.,
            phase: 0.3,
            crest_wander: 0.13,
            slope_exponent: 1.15,
            folds_meters: 48.,
            end_fade: 0.15,
        }],
        water_level_meters: None,
    }
}
#[test]
fn ridge_seams_global_translation_region_indices_and_v1_v2_codec() {
    let d = doc();
    let sampler = Sampler::new(&d).unwrap();
    for (u, v) in [(0., 0.), (0.3, 0.1), (-0.5, -0.2), (0.75, 0.4)] {
        let crest = 0.13_f64 * (u * 6. + 0.3_f64).sin();
        let envelope = (1. - (v - crest).abs()).max(0.).powf(1.15);
        let profile = 0.76
            + 0.18 * (u * 5. + 0.3_f64).sin()
            + 0.10 * (u * 13. - 0.3_f64).sin()
            + 0.05 * (u * 31. + 0.3_f64).sin();
        let folds = 48. * (u * 47. + v * 19. + 0.3_f64).sin() * (v * 17. - u * 11.).sin();
        let expected = ((850. * profile + folds) * envelope).max(0.);
        assert!((sampler.sample(p(u * 2000., v * 800.)).unwrap().unwrap() - expected).abs() < 1e-9);
    }
    assert_eq!(sampler.sample(p(2000., 0.)).unwrap(), Some(0.));
    assert_eq!(sampler.sample(p(2001., 0.)).unwrap(), Some(0.));
    assert!(sampler.sample(p(1999.999, 0.)).unwrap().unwrap() < 1e-10);
    let huge = Position::normalize(Coordinates {
        cell: [9_007_199_254_740_993, 0, -9_007_199_254_740_993],
        offset: [1010., 0., 1010.],
    })
    .unwrap();
    let mut shifted = d.clone();
    shifted.datum = huge;
    if let Operation::Ridge { center, .. } = &mut shifted.operations[0] {
        *center = huge;
    }
    let local = Sampler::new(&shifted).unwrap();
    for delta in [[0., 0., 0.], [777., 0., -81.], [-2001., 0., 100.]] {
        assert_eq!(
            sampler.sample(p(delta[0], delta[2])).unwrap(),
            local.sample(huge.translated(delta).unwrap()).unwrap()
        );
    }
    let a = terrain::bake(
        &d,
        Patch {
            origin: p(-16., 0.),
            cells: [4, 4],
            spacing_meters: 4.,
        },
    )
    .unwrap();
    let b = terrain::bake(
        &d,
        Patch {
            origin: p(0., 0.),
            cells: [4, 4],
            spacing_meters: 4.,
        },
    )
    .unwrap();
    for z in 0..5 {
        assert_eq!(a.heights[z * 5 + 4].to_bits(), b.heights[z * 5].to_bits());
    }
    let region = terrain::region::compile(
        &d,
        Region {
            origin: p(-16., 0.),
            cells: [8, 4],
            spacing_meters: 4.,
            axis_offsets_meters: None,
        },
    )
    .unwrap();
    assert_eq!(region.geometry.positions.len(), 45);
    assert_eq!(region.geometry.triangles.len(), 64);
    for z in 0..5 {
        assert_eq!(
            region.geometry.positions[z * 9 + 4][1].to_bits(),
            a.heights[z * 5 + 4].to_bits()
        );
    }
    let bytes = terrain::encode(&d).unwrap();
    assert_eq!(&bytes[8..10], &2u16.to_le_bytes());
    assert_eq!(terrain::decode(&bytes).unwrap(), d);
    let mut stale = d.clone();
    stale.version = 1;
    assert!(stale.validate().is_err());
    let mut tampered = bytes;
    tampered[8..10].copy_from_slice(&1u16.to_le_bytes());
    assert!(terrain::decode(&tampered).is_err());
    let old = include_bytes!("../examples/terrain-tools.ignis-terrain");
    assert_eq!(
        terrain::encode(&terrain::decode(old).unwrap()).unwrap(),
        old
    );
    assert!(
        terrain::region::compile(
            &d,
            Region {
                origin: p(0., 0.),
                cells: [u16::MAX; 2],
                spacing_meters: 4.,
                axis_offsets_meters: None,
            }
        )
        .is_err()
    );
}
#[test]
fn regional_publication_beyond_modeling_budget_is_atomic_and_undoable() {
    let mut d = doc();
    d.operations.clear();
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    let before = e.snapshot();
    let mut pub_ = Publication {
        document: d,
        region: Region {
            origin: p(-64., -64.),
            cells: [128, 128],
            spacing_meters: 1.,
            axis_offsets_meters: None,
        },
        frame_origin: p(0., 0.),
        asset: "ground".into(),
        instance: "ground".into(),
        color: [0.3, 0.5, 0.2],
        material: Default::default(),
        collision: Some(true),
        world_material_coordinates: false,
    };
    pub_.instance.clear();
    assert!(
        e.execute(
            Command::PublishTerrainRegion {
                publication: Box::new(pub_.clone())
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    pub_.instance = "ground".into();
    let result = e
        .execute(
            Command::PublishTerrainRegion {
                publication: Box::new(pub_),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(result["visual_only"], false);
    assert_eq!(e.scene.meshes.assets[0].geometry.positions.len(), 16641);
    assert_eq!(e.scene.collision_meshes, ["ground"]);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.snapshot()["scene"], before["scene"]);
}

/// A malformed axis must not publish folded geometry; adjoining grids must share
/// the same physical edge even when cell widths change away from the player.
#[test]
fn explicit_axes_validate_before_allocation_and_preserve_shared_edges() {
    use terrain::region::Axes;
    let d = doc();
    let mut left = Region {
        origin: p(-64., -32.),
        cells: [4, 3],
        spacing_meters: 4.,
        axis_offsets_meters: Some(Axes {
            x: vec![0., 32., 56., 60., 64.],
            z: vec![0., 4., 8., 64.],
        }),
    };
    let l = terrain::region::compile(&d, left.clone()).unwrap();
    let mut right = left.clone();
    right.origin = p(0., -32.);
    right.axis_offsets_meters.as_mut().unwrap().x = vec![0., 4., 8., 32., 64.];
    let r = terrain::region::compile(&d, right).unwrap();
    let sampler = Sampler::new(&d).unwrap();
    for row in 0..4 {
        let a = l.geometry.positions[row * 5 + 4];
        let b = r.geometry.positions[row * 5];
        assert_eq!(a[1].to_bits(), b[1].to_bits());
        assert_eq!(a[2], b[2]);
        assert_eq!(a[0] - 64., b[0]);
        assert_eq!(
            a[1],
            sampler
                .sample(p(0., f64::from(a[2]) - 32.))
                .unwrap()
                .unwrap() as f32
        );
    }
    assert_eq!(l.geometry.positions.len(), 20);
    assert_eq!(l.geometry.triangles.len(), 24);
    assert_eq!(l.bounds[1][0], 64.);
    for bad in [
        vec![0., 1.],
        vec![0., 4., 4., 60., 64.],
        vec![0., 4., f64::NAN, 60., 64.],
        vec![1., 4., 8., 60., 64.],
        vec![0., 4., 8., 60., 10001.],
    ] {
        left.axis_offsets_meters.as_mut().unwrap().x = bad;
        assert!(terrain::region::compile(&d, left.clone()).is_err());
    }
    let over = serde_json::json!({"x":vec![0.; terrain::region::MAX_AXIS_POINTS+1],"z":[0.,4.]});
    assert!(serde_json::from_value::<Axes>(over).is_err());
    left.origin = p(-10000., -10000.);
    left.axis_offsets_meters.as_mut().unwrap().x = vec![0., 32., 56., 60., 64.];
    let mut publication = Publication {
        document: d,
        region: left,
        frame_origin: p(0., 0.),
        asset: "diagonal_ground".into(),
        instance: "diagonal_ground".into(),
        color: [0.3, 0.5, 0.2],
        material: Default::default(),
        collision: Some(true),
        world_material_coordinates: true,
    };
    let (asset, instance, _) = terrain::region::publication(publication.clone()).unwrap();
    assert_eq!(instance.position, [-10000., 0., -10000.]);
    assert_eq!(
        asset.geometry.material_coordinates.as_ref().unwrap()[0],
        [-10000., 0., -10000.]
    );
    publication.region.origin = p(10001., 0.);
    assert!(terrain::region::publication(publication).is_err());
}
