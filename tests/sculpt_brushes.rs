use glam::Vec3;
use ignis::{
    editor::SculptSettings,
    engine::Command,
    mesh_tools,
    meshes::{Asset, BrushKind, Stroke},
    model::RoomRecipe,
    service::Service,
};
use std::sync::Arc;
const SHAPES: [BrushKind; 6] = [
    BrushKind::Flatten,
    BrushKind::Pinch,
    BrushKind::Crease,
    BrushKind::Scrape,
    BrushKind::Fill,
    BrushKind::Relax,
];
fn surface() -> Asset {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: mesh_tools::Primitive::Plane,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )
    .unwrap();
    let mut a = s.engine.scene.meshes.assets[0].clone();
    for p in &mut Arc::make_mut(&mut a.geometry).positions {
        p[1] = 0.12 * (p[0] * 7.).sin() * (p[2] * 4.).cos();
        p[0] += 0.012 * (p[2] * 19. + p[0] * 11.).sin();
    }
    a.geometry.validate().unwrap();
    a
}
fn stroke(kind: BrushKind) -> Stroke {
    Stroke {
        kind,
        centers: vec![[-0.15, -0.1, 0.], [0., 0., 0.], [0.15, 0.1, 0.]],
        radius: 0.65,
        strength: 0.25,
        direction: [0., 1., 0.],
    }
}
fn distance(a: &Asset, b: &Asset) -> f32 {
    a.geometry
        .positions
        .iter()
        .zip(&b.geometry.positions)
        .map(|(p, q)| Vec3::from(*p).distance(Vec3::from(*q)))
        .fold(0., f32::max)
}
#[test]
fn six_kernels_are_distinct_local_deformations_and_reverse_path_independent() {
    let a = surface();
    let mut outputs = vec![];
    for kind in SHAPES {
        let s = stroke(kind);
        let result = mesh_tools::sculpt_preview(&a, &s).unwrap();
        assert!(distance(&a, &result) > 1e-4, "{kind:?} did nothing");
        assert!(
            distance(&a, &result) < 0.2,
            "{kind:?} escaped the local surface"
        );
        assert_eq!(result.geometry.triangles, a.geometry.triangles);
        assert_eq!(result.material, a.material);
        assert_eq!(result.rig, a.rig);
        for (p, q) in a.geometry.positions.iter().zip(&result.geometry.positions) {
            if s.centers
                .iter()
                .all(|c| Vec3::from(*p).distance(Vec3::from(*c)) >= s.radius)
            {
                assert_eq!(p, q, "{kind:?} affected outside radius");
            }
        }
        let mut reverse = s.clone();
        reverse.centers.reverse();
        let reversed = mesh_tools::sculpt_preview(&a, &reverse).unwrap();
        assert!(
            distance(&result, &reversed) < 2e-6,
            "{kind:?} depends on path direction"
        );
        for face in &a.geometry.triangles {
            let [p, q, r] = face.map(|i| Vec3::from(a.geometry.positions[i as usize]));
            let [a, b, c] = face.map(|i| Vec3::from(result.geometry.positions[i as usize]));
            assert!((q - p).cross(r - p).dot((b - a).cross(c - a)) > 0.);
        }
        outputs.push(result);
    }
    for (i, a) in outputs.iter().enumerate() {
        for b in &outputs[i + 1..] {
            assert!(distance(a, b) > 1e-4, "two brushes are aliases");
        }
    }
}
#[test]
fn scrape_fill_are_one_sided_and_ctrl_swaps_them_pinch_and_crease_really_invert() {
    let a = surface();
    let positive = mesh_tools::sculpt_preview(&a, &stroke(BrushKind::Scrape)).unwrap();
    let mut inverse = stroke(BrushKind::Fill);
    inverse.strength *= -1.;
    let swapped = mesh_tools::sculpt_preview(&a, &inverse).unwrap();
    assert_eq!(positive.geometry, swapped.geometry);
    for kind in [
        BrushKind::Pinch,
        BrushKind::Crease,
        BrushKind::Scrape,
        BrushKind::Fill,
    ] {
        let positive = mesh_tools::sculpt_preview(&a, &stroke(kind)).unwrap();
        let mut s = stroke(kind);
        s.strength *= -1.;
        let negative = mesh_tools::sculpt_preview(&a, &s).unwrap();
        assert!(
            distance(&positive, &negative) > 1e-4,
            "{kind:?} inversion did nothing"
        );
    }
    for kind in [BrushKind::Flatten, BrushKind::Relax, BrushKind::Smooth] {
        let mut s = stroke(kind);
        s.strength = -0.1;
        assert!(mesh_tools::sculpt_preview(&a, &s).is_err());
    }
}
#[test]
fn radius_strength_and_zero_are_respected_and_errors_do_not_modify_the_source() {
    let a = surface();
    let frozen = a.clone();
    for kind in SHAPES {
        let mut s = stroke(kind);
        s.strength = 0.;
        assert_eq!(
            mesh_tools::sculpt_preview(&a, &s).unwrap().geometry,
            a.geometry
        );
        s.strength = 0.1;
        let low = mesh_tools::sculpt_preview(&a, &s).unwrap();
        s.strength = 0.5;
        let high = mesh_tools::sculpt_preview(&a, &s).unwrap();
        assert!(distance(&a, &high) > distance(&a, &low));
        s.strength = 0.25;
        s.radius = 0.35;
        let narrow = mesh_tools::sculpt_preview(&a, &s).unwrap();
        s.radius = 0.8;
        let wide = mesh_tools::sculpt_preview(&a, &s).unwrap();
        let changed = |b: &Asset| {
            a.geometry
                .positions
                .iter()
                .zip(&b.geometry.positions)
                .filter(|(p, q)| p != q)
                .count()
        };
        assert!(changed(&wide) > changed(&narrow));
        s.radius = f32::NAN;
        assert!(mesh_tools::sculpt_preview(&a, &s).is_err());
        assert_eq!(a, frozen);
    }
}
#[test]
fn old_layout_strengths_load_and_nine_brush_preferences_stay_independent() {
    let old = serde_json::json!({"brush":"draw","radius":0.4,"draw_strength":0.04,"grab_strength":0.7,"smooth_strength":0.5});
    let mut settings: SculptSettings = serde_json::from_value(old).unwrap();
    settings.validate().unwrap();
    assert!(
        !settings.draw_strength_is_relative,
        "old layouts retain their unit marker for native migration"
    );
    for (i, kind) in BrushKind::ALL.into_iter().enumerate() {
        *settings.strength_mut(kind) = 0.1 + i as f32 * 0.07;
    }
    for (i, kind) in BrushKind::ALL.into_iter().enumerate() {
        assert_eq!(settings.strength(kind), 0.1 + i as f32 * 0.07);
    }
    let saved = serde_json::to_string(&settings).unwrap();
    assert_eq!(
        serde_json::from_str::<SculptSettings>(&saved).unwrap(),
        settings
    );
}
#[test]
fn every_shape_uses_public_edit_preview_commit_undo_and_preserves_asset_metadata() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let a = surface();
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset: a.clone() },
        }
        .into(),
    )
    .unwrap();
    for kind in SHAPES {
        let before = s.engine.snapshot();
        let stroke = stroke(kind);
        let expected = mesh_tools::sculpt_preview(&a, &stroke).unwrap();
        let edit = ignis::meshes::Edit::Sculpt {
            asset: a.id.clone(),
            stroke,
        };
        s.execute(Command::EditMesh { edit }.into()).unwrap();
        assert_eq!(s.engine.scene.meshes.asset(&a.id).unwrap(), &expected);
        assert_eq!(
            s.engine.snapshot()["history"]["undo"].as_u64(),
            Some(before["history"]["undo"].as_u64().unwrap() + 1)
        );
        s.execute(Command::Undo.into()).unwrap();
        assert_eq!(s.engine.snapshot()["scene"], before["scene"]);
    }
}

// Critical: a single grab on a new mesh must not publish inverted faces or history.
#[test]
fn first_grab_cannot_fold_a_clean_cube_or_partially_mutate_engine() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: mesh_tools::Primitive::Cube,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )
    .unwrap();
    let a = s.engine.scene.meshes.assets[0].clone();
    for sign in [-1., 1.] {
        let stroke = Stroke {
            kind: BrushKind::Grab,
            centers: vec![[0., 0., 1.]],
            radius: 0.241,
            strength: 0.35,
            direction: [sign, 0., 0.],
        };
        assert!(
            mesh_tools::sculpt_preview(&a, &stroke).is_err(),
            "first Grab must refuse a folded surface"
        );
        let before = s.engine.snapshot();
        assert!(
            s.execute(
                Command::EditMesh {
                    edit: ignis::meshes::Edit::Sculpt {
                        asset: a.id.clone(),
                        stroke
                    }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(s.engine.snapshot(), before);
    }
    let stroke = Stroke {
        kind: BrushKind::Grab,
        centers: vec![[0., 0., 1.]],
        radius: 0.241,
        strength: 0.03,
        direction: [1., 0., 0.],
    };
    let before = s.engine.scene.clone();
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::Sculpt {
                asset: a.id,
                stroke,
            },
        }
        .into(),
    )
    .unwrap();
    assert_ne!(s.engine.scene, before);
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(s.engine.scene, before);
}

// Critical: a Draw dab must raise a coherent patch instead of diverging with creased vertex normals.
#[test]
fn draw_dab_has_one_area_direction_on_an_uneven_surface() {
    let a = surface();
    let stroke = Stroke {
        kind: BrushKind::Draw,
        centers: vec![[0., 0., 0.]],
        radius: 0.65,
        strength: 0.04,
        direction: [0., 1., 0.],
    };
    let b = mesh_tools::sculpt_preview(&a, &stroke).unwrap();
    let directions: Vec<_> = a
        .geometry
        .positions
        .iter()
        .zip(&b.geometry.positions)
        .filter_map(|(a, b)| {
            let d = Vec3::from(*b) - Vec3::from(*a);
            (d.length() > 0.0001).then(|| d.normalize())
        })
        .collect();
    assert!(directions.len() > 20);
    assert!(
        directions.iter().all(|d| d.dot(directions[0]) > 0.9999),
        "Draw diverged into independent vertex normals"
    );
}
