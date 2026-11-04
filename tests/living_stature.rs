use ignis::{
    botany::{
        self as b,
        growth::{Definition, Plant, Stature},
    },
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
fn diameter(r: &b::Recipe) -> f64 {
    if let b::Shape::Axis { radius, .. } = r.source.organs[0].shape {
        radius * 2.
    } else {
        panic!("missing trunk")
    }
}
#[test]
fn age_drives_configurable_stature_and_preserves_canonical_identity() {
    let stature = Stature {
        adult_diameter_m: 30.,
        seedling_diameter_m: 0.01,
        maturity_years: 40.,
    };
    let d = Definition {
        version: 3,
        stature: Some(stature.clone()),
        interval: 365,
        maximum_nodes: 10,
        ..Default::default()
    };
    let mut p = Plant::new(d, 0).unwrap();
    assert!((diameter(&p.recipe().unwrap()) - 0.01).abs() < 1e-12);
    let young = p.advance(365 * 5).unwrap();
    let adult = p.advance(365 * 40).unwrap();
    assert!(diameter(&adult) > diameter(&young));
    assert!((diameter(&adult) - 30.).abs() < 1e-10);
    assert!(adult.source.organs.iter().any(|o| o.id == "node0/b5/t3/l9"));
    assert_eq!(p.recipe().unwrap(), adult);
    let restored: Plant = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
    assert_eq!(restored.recipe().unwrap(), adult);
    let mut invalid = p.clone();
    invalid.definition.version = 2;
    assert!(invalid.validate().is_err());
    let before = p.clone();
    assert!(p.advance(1).is_err());
    assert_eq!(p, before);
}
#[test]
fn configuring_stature_and_bad_numeric_input_are_atomic() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let call = |s: &mut Service, operation| {
        s.execute(
            Command::Botany {
                operation: Box::new(operation),
            }
            .into(),
        )
    };
    call(
        &mut s,
        b::Operation::CreateLiving {
            definition: Definition {
                version: 2,
                interval: 365,
                ..Default::default()
            },
            born: 0,
        },
    )
    .unwrap();
    call(
        &mut s,
        b::Operation::Living {
            prefix: "living".into(),
            action: b::growth::Action::Advance { time: 365 * 20 },
        },
    )
    .unwrap();
    call(
        &mut s,
        b::Operation::Living {
            prefix: "living".into(),
            action: b::growth::Action::SetStature {
                stature: Stature {
                    adult_diameter_m: 5.,
                    seedling_diameter_m: 0.01,
                    maturity_years: 20.,
                },
            },
        },
    )
    .unwrap();
    assert!((diameter(&s.engine.scene.botany[0]) - 5.).abs() < 1e-9);
    let valid = s.engine.scene.clone();
    let revision = s.engine.revision;
    assert!(
        call(
            &mut s,
            b::Operation::Living {
                prefix: "living".into(),
                action: b::growth::Action::SetStature {
                    stature: Stature {
                        adult_diameter_m: f64::MAX,
                        seedling_diameter_m: 0.01,
                        maturity_years: 20.
                    }
                }
            }
        )
        .is_err()
    );
    assert_eq!(s.engine.scene, valid);
    assert_eq!(s.engine.revision, revision);
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        s.engine.scene.botany[0]
            .living
            .as_ref()
            .unwrap()
            .definition
            .version,
        2
    );
    s.execute(Command::Redo.into()).unwrap();
    assert_eq!(s.engine.scene, valid);
}
