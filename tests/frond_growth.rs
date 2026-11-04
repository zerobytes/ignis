use ignis::{
    botany::{self as b, bake},
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
use std::sync::Arc;

fn fixture() -> b::Recipe {
    let mut r: b::Recipe =
        serde_json::from_slice(&std::fs::read("assets/botany/palmeirinha.ignis-botany").unwrap())
            .unwrap();
    Arc::make_mut(&mut r.source).organs.retain(|o| {
        o.id == "caule0"
            || o.id == "caule0/raquis0"
            || o.id == "caule0/raquis4"
            || o.id == "caule0/raquis0/foliolo10_1"
            || o.id == "caule0/raquis4/foliolo10_1"
    });
    r
}

#[test]
fn growth_scales_complete_fronds_and_descendants_preserving_curve_and_legacy() {
    let mut r = fixture();
    r.settings.frond_growth_variation = 0.;
    let original = bake::compile(&r).unwrap();
    let mut legacy = r.clone();
    legacy.version = 3;
    let source = Arc::make_mut(&mut legacy.source);
    source.version = 3;
    for o in &mut source.organs {
        o.life.as_mut().unwrap().growth_stage = 1.;
    }
    let old = bake::compile(&legacy).unwrap();
    assert_eq!(original.document, old.document);
    assert!(
        !serde_json::to_value(&legacy).unwrap()["settings"]
            .as_object()
            .unwrap()
            .contains_key("frond_growth_variation")
    );
    r.settings.frond_growth_variation = 1.;
    let scaled = bake::compile(&r).unwrap();
    for (before, after) in original.poses.iter().zip(&scaled.poses) {
        let id = &r.source.organs[after.source].id;
        let factor = if id.starts_with("caule0/raquis0") {
            0.30
        } else {
            1.
        };
        assert!((after.length / before.length - factor).abs() < 1e-8, "{id}");
        assert!((after.radius / before.radius - factor).abs() < 1e-8, "{id}");
        if before.width != 0. {
            assert!((after.width / before.width - factor).abs() < 1e-8, "{id}");
        }
        for (a, z) in before.samples.iter().zip(&after.samples) {
            assert!(a.t.distance(z.t) < 1e-8, "Curve angle changed: {id}");
        }
        if let Some(parent) = after.parent
            && matches!(r.source.organs[after.source].shape, b::Shape::Axis { .. })
        {
            let insertion = r.source.organs[after.source].insertion;
            assert!(
                after.samples[0]
                    .p
                    .distance(scaled.poses[parent].at(insertion).p)
                    < 1e-7
            );
        }
    }
    let json = serde_json::to_vec(&r).unwrap();
    let restored: b::Recipe = serde_json::from_slice(&json).unwrap();
    assert_eq!(bake::compile(&restored).unwrap().document, scaled.document);
}

#[test]
fn growth_preview_is_atomic_and_invalid_versions_or_stages_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let mut r = fixture();
    r.settings.frond_growth_variation = 0.;
    let execute = |s: &mut Service, operation| {
        s.execute(
            Command::Botany {
                operation: Box::new(operation),
            }
            .into(),
        )
    };
    execute(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(r.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let scene = service.engine.scene.clone();
    let revision = service.engine.revision;
    r.settings.frond_growth_variation = 1.;
    execute(
        &mut service,
        b::Operation::Preview {
            recipe: Box::new(r.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(service.engine.scene, scene);
    assert_eq!(service.engine.revision, revision);
    execute(&mut service, b::Operation::CancelPreview).unwrap();
    assert!(service.botany.overlay.is_none());
    for value in [-0.01, 1.01, f64::NAN] {
        let mut invalid = r.clone();
        invalid.settings.frond_growth_variation = value;
        assert!(
            execute(
                &mut service,
                b::Operation::Publish {
                    recipe: Box::new(invalid),
                    policy: Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(service.engine.scene, scene);
        assert_eq!(service.engine.revision, revision);
    }
    let mut invalid = r.clone();
    invalid.version = 8;
    assert!(invalid.validate().is_err());
    let mut invalid = r.clone();
    Arc::make_mut(&mut invalid.source).version = 8;
    assert!(invalid.validate().is_err());
    let mut invalid = r.clone();
    Arc::make_mut(&mut invalid.source).organs[1]
        .life
        .as_mut()
        .unwrap()
        .growth_stage = 0.;
    assert!(invalid.validate().is_err());
    let mut invalid = r.clone();
    Arc::make_mut(&mut invalid.source).organs[0]
        .life
        .as_mut()
        .unwrap()
        .growth_stage = 0.5;
    assert!(invalid.validate().is_err());
}
