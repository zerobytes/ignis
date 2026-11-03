use ignis::{
    engine::{Command, Engine},
    materials::{Detail, Material, Profile},
    meshes::Edit,
    model::RoomRecipe,
};
use serde_json::json;
use std::sync::Arc;
fn detail() -> Detail {
    serde_json::from_value(
        json!({"plane":"xz","tint":[0.3,0.4,0.2],"opacity":0.8,"bump":-0.003,
        "segments":[{"start":[0.25,0.25],"end":[0.75,0.75],"widths":[0.05,0.1]}]}),
    )
    .unwrap()
}
#[test]
fn legacy_is_omitted_and_new_fields_are_strict() {
    let d = detail();
    let old = serde_json::to_value(&d).unwrap();
    assert!(old.get("profile").is_none() && old.get("repeat").is_none());
    let mut v = old.clone();
    v["profile"] = json!("rounded");
    v["repeat"] = json!([1, 1]);
    let new: Detail = serde_json::from_value(v.clone()).unwrap();
    new.validate().unwrap();
    assert_eq!(new.profile, Profile::Rounded);
    assert_eq!(new.repeat, Some([1., 1.]));
    for profile in [json!("circle"), json!({"kind":"rounded"}), json!(1)] {
        v["profile"] = profile;
        assert!(serde_json::from_value::<Detail>(v.clone()).is_err());
    }
    assert_eq!(serde_json::to_value(d).unwrap(), old);
}
#[test]
fn capsule_clearance_and_period_bounds_reject_atomically() {
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::Grid {
                id: "sheet".into(),
                segments: 1,
                size: 1.,
            },
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Step { ticks: 8 }.into()).unwrap();
    let mut d = detail();
    d.repeat = Some([1., 1.]);
    d.profile = Profile::Rounded;
    for case in 0..8 {
        let mut bad = d.clone();
        match case {
            0 => bad.repeat = Some([f32::NAN, 1.]),
            1 => bad.repeat = Some([0., 1.]),
            2 => bad.repeat = Some([1., 10001.]),
            3 => bad.segments[0].start[0] = 0.01,
            4 => bad.segments[0].end[1] = 0.99,
            5 => bad.segments[0].widths[1] = 0.2,
            6 => bad.repeat = Some([0.0009, 1.]),
            _ => bad.segments[0].start[1] = -0.2,
        }
        let snapshot = e.snapshot();
        let scene = e.scene.clone();
        let edit = Edit::SetMaterial {
            asset: "sheet".into(),
            material: Material {
                detail: Some(Arc::new(bad)),
                ..Default::default()
            },
        };
        for c in [
            Command::PreviewMeshEdit { edit: edit.clone() },
            Command::EditMesh { edit },
        ] {
            assert!(e.execute(c.into()).is_err());
            assert_eq!(e.scene, scene);
            assert_eq!(e.snapshot(), snapshot);
        }
    }
    // Long diagonal and variable radii are allowed when their whole capsule,
    // including the AA clearance, remains in the convex period rectangle.
    d.validate().unwrap();
    let original = e.scene.clone();
    e.execute(
        Command::EditMesh {
            edit: Edit::SetMaterial {
                asset: "sheet".into(),
                material: Material {
                    detail: Some(Arc::new(d)),
                    ..Default::default()
                },
            },
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
}
#[test]
fn new_format_gate_covers_either_option_and_replay() {
    for repeat in [false, true] {
        let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
        e.execute(
            Command::EditMesh {
                edit: Edit::Grid {
                    id: "sheet".into(),
                    segments: 1,
                    size: 1.,
                },
            }
            .into(),
        )
        .unwrap();
        let mut d = detail();
        if repeat {
            d.repeat = Some([1., 1.]);
        } else {
            d.profile = Profile::Rounded;
        }
        e.execute(
            Command::EditMesh {
                edit: Edit::SetMaterial {
                    asset: "sheet".into(),
                    material: Material {
                        detail: Some(Arc::new(d)),
                        ..Default::default()
                    },
                },
            }
            .into(),
        )
        .unwrap();
        let mut old = e.scene.clone();
        old.version = 70;
        assert!(old.validate().unwrap_err().message.contains("version71"));
        assert_eq!(e.replay().version, ignis::engine::REPLAY_VERSION);
    }
}
