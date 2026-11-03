//! Critical sculpt saturation and displacement regressions: a normal drag must
//! keep producing bounded geometry instead of losing the entire transaction.
use glam::Vec3;
use ignis::{
    engine::Command,
    mesh_tools::Primitive,
    meshes::{self, BrushKind, Stroke},
    model::RoomRecipe,
    service::Service,
};
use std::sync::Arc;
fn fixture() -> (tempfile::TempDir, Service) {
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(
        root.path(),
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    service
        .execute(
            Command::CreateEditorMesh {
                primitive: Primitive::Cube,
                resolution: 24,
                size: 2.,
            }
            .into(),
        )
        .unwrap();
    (root, service)
}
fn operation(s: &Service, stroke: Stroke, adaptive_detail: bool) -> meshes::Edit {
    meshes::Edit::SculptStroke {
        asset: "mesh_0".into(),
        reference: s.engine.mesh_reference(),
        stroke,
        adaptive_detail,
        detail: Default::default(),
    }
}
fn stroke(
    kind: BrushKind,
    radius: f32,
    strength: f32,
    center: [f32; 3],
    direction: [f32; 3],
) -> Stroke {
    Stroke {
        kind,
        radius,
        strength,
        centers: vec![center],
        direction,
    }
}
fn commit(s: &mut Service, edit: meshes::Edit) -> meshes::Asset {
    let before = s.engine.snapshot();
    let preview = s
        .execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
        .unwrap()
        .data;
    assert_eq!(
        s.engine.snapshot(),
        before,
        "preview must not consume the gesture"
    );
    s.execute(Command::EditMesh { edit }.into()).unwrap();
    assert_eq!(
        serde_json::to_value(&s.engine.scene.meshes).unwrap(),
        preview["meshes"]
    );
    assert_eq!(
        s.engine.snapshot()["history"]["undo"].as_u64(),
        Some(before["history"]["undo"].as_u64().unwrap() + 1)
    );
    let asset = s.engine.scene.meshes.assets[0].clone();
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(s.engine.snapshot()["scene"], before["scene"]);
    asset
}
#[test]
fn saturated_detail_keeps_painting_with_proportional_force_and_one_undo() {
    let (_root, mut s) = fixture();
    let mut base = s.engine.scene.meshes.assets[0].clone();
    base.geometry = Arc::new(meshes::Geometry::grid(63, 2.).unwrap());
    base = ignis::mesh_topology::Plan::new(&base.geometry, None)
        .unwrap()
        .asset(&base)
        .unwrap();
    assert_eq!(base.geometry.positions.len(), 16129);
    s.execute(
        Command::EditMesh {
            edit: meshes::Edit::PutAsset {
                asset: base.clone(),
            },
        }
        .into(),
    )
    .unwrap();
    let mut maxima = vec![];
    for force in [0.05, 0.1, 0.2, 0.4, 0.8] {
        let edit = operation(
            &s,
            stroke(
                BrushKind::Draw,
                0.005,
                force,
                [0.007, 0., 0.007],
                [0., 1., 0.],
            ),
            true,
        );
        let result = commit(&mut s, edit);
        result.geometry.validate().unwrap();
        assert!(result.geometry.positions.len() <= meshes::MAX_VERTICES);
        assert!(result.geometry.triangles.len() <= meshes::MAX_TRIANGLES);
        let max = result
            .geometry
            .positions
            .iter()
            .map(|p| p[1].abs())
            .fold(0f32, f32::max);
        assert!(max > 0., "full mesh silently swallowed pressure{force}");
        assert!(
            max <= 0.005 * 0.5 * force + 1e-7,
            "runaway relief at pressure{force}"
        );
        maxima.push(max);
    }
    for pair in maxima.windows(2) {
        assert!(
            (pair[1] / pair[0] - 2.).abs() < 0.001,
            "pressure discontinuity: {maxima:?}"
        );
    }
    println!("saturated pressure response {maxima:?}");
}
#[test]
fn volume_envelope_is_independent_of_overlap_and_capped_by_requested_radius() {
    let (_root, mut s) = fixture();
    let base = s.engine.scene.meshes.assets[0].clone();
    let mut outputs = vec![];
    for n in [1, 16, 128] {
        let mut brush = stroke(BrushKind::Draw, 0.4, 0.6, [0., 0., 1.], [0., 0., 1.]);
        brush.centers = vec![[0., 0., 1.]; n];
        let edit = operation(&s, brush, false);
        let result = commit(&mut s, edit);
        let max = result
            .geometry
            .positions
            .iter()
            .zip(&base.geometry.positions)
            .map(|(a, b)| Vec3::from(*a).distance(Vec3::from(*b)))
            .fold(0f32, f32::max);
        assert!((max - 0.12).abs() < 1e-5, "overlap inflated relief: {max}");
        outputs.push(result.geometry.positions.clone());
    }
    assert_eq!(outputs[0], outputs[1]);
    assert_eq!(outputs[1], outputs[2]);
    // Small-radius detail used to reject within4passes, even with free capacity.
    let tiny = operation(
        &s,
        stroke(
            BrushKind::Draw,
            0.001,
            0.6,
            [0.017, 0.023, 1.],
            [0., 0., 1.],
        ),
        true,
    );
    let result = commit(&mut s, tiny);
    assert_ne!(result.geometry.positions, base.geometry.positions);
    assert!(result.geometry.positions.len() <= meshes::MAX_VERTICES);
}
#[test]
fn grab_stops_before_first_fold_then_recovers_when_cursor_recedes() {
    let (_root, mut s) = fixture();
    let base = s.engine.scene.meshes.assets[0].clone();
    for sign in [-1., 1.] {
        for strength in [0.35, 1., 0.01] {
            let edit = operation(
                &s,
                stroke(
                    BrushKind::Grab,
                    0.241,
                    strength,
                    [0., 0., 1.],
                    [sign, 0., 0.],
                ),
                false,
            );
            let result = commit(&mut s, edit);
            let displacement = result
                .geometry
                .positions
                .iter()
                .zip(&base.geometry.positions)
                .map(|(a, b)| Vec3::from(*a).distance(Vec3::from(*b)))
                .fold(0f32, f32::max);
            assert!(displacement > 0.);
            if strength > 0.1 {
                assert!(displacement < strength * 0.9, "fold was not constrained");
            } else {
                assert!(
                    (displacement - strength).abs() < 1e-5,
                    "cursor recede did not recover"
                );
            }
            for f in &base.geometry.triangles {
                let [a, b, c] = f.map(|i| Vec3::from(base.geometry.positions[i as usize]));
                let [p, q, r] = f.map(|i| Vec3::from(result.geometry.positions[i as usize]));
                assert!((b - a).cross(c - a).dot((q - p).cross(r - p)) > 0.);
            }
        }
    }
}
#[test]
fn interactive_scope_and_invalid_batch_refuse_without_partial_publication() {
    let (_root, mut s) = fixture();
    let stale = operation(
        &s,
        stroke(BrushKind::Draw, 0.4, 0.6, [0., 0., 1.], [0., 0., 1.]),
        true,
    );
    s.execute(Command::Reset.into()).unwrap();
    let before = s.engine.snapshot();
    assert!(s.execute(Command::EditMesh { edit: stale }.into()).is_err());
    assert_eq!(s.engine.snapshot(), before);
    let mut bad = s.engine.scene.meshes.assets[0].clone();
    Arc::make_mut(&mut bad.geometry).triangles[0][0] = u32::MAX;
    let edit = operation(
        &s,
        stroke(BrushKind::Draw, 0.4, 0.6, [0., 0., 1.], [0., 0., 1.]),
        true,
    );
    assert!(
        s.execute(
            Command::EditScene {
                edits: vec![
                    ignis::engine::SceneEdit::Mesh {
                        edit: meshes::Edit::PutAsset { asset: bad }
                    },
                    ignis::engine::SceneEdit::Mesh { edit }
                ]
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(s.engine.snapshot(), before);
}

/// Persisted authoring parameters must not change legacy geometry or allow a
/// preview to publish a different topology than the confirmed transaction.
#[test]
fn detail_configuration_preserves_legacy_preview_and_atomic_commit() {
    use ignis::mesh_tools::SculptDetail;
    let (_root, mut s) = fixture();
    let brush = stroke(BrushKind::Draw, 0.4, 0.2, [0., 0., 1.], [0., 0., 1.]);
    let default = operation(&s, brush.clone(), true);
    let mut legacy = serde_json::to_value(&default).unwrap();
    legacy.as_object_mut().unwrap().remove("detail");
    let legacy: meshes::Edit = serde_json::from_value(legacy).unwrap();
    let original = s.engine.scene.meshes.assets[0].clone();
    let expected = commit(&mut s, legacy);
    let default = operation(&s, brush.clone(), true);
    assert_eq!(commit(&mut s, default), expected);

    let mut fine_vertices = 0;
    for detail in [
        SculptDetail::Brush { percent: 12.5 },
        SculptDetail::Constant { edge_length: 0.05 },
        SculptDetail::Constant { edge_length: 0.2 },
    ] {
        let mut edit = operation(&s, brush.clone(), true);
        if let meshes::Edit::SculptStroke { detail: value, .. } = &mut edit {
            *value = detail;
        }
        let result = commit(&mut s, edit);
        if detail == (SculptDetail::Constant { edge_length: 0.2 }) {
            assert!(result.geometry.positions.len() < fine_vertices);
        } else {
            assert_eq!(
                result, expected,
                "equivalent detail must produce identical geometry"
            );
            fine_vertices = result.geometry.positions.len();
        }
    }
    let fixed = operation(&s, brush.clone(), false);
    let fixed = commit(&mut s, fixed);
    assert_eq!(fixed.geometry.triangles, original.geometry.triangles);
    assert_eq!(
        fixed.geometry.positions.len(),
        original.geometry.positions.len()
    );

    for detail in [
        SculptDetail::Brush { percent: 0. },
        SculptDetail::Constant { edge_length: -1. },
    ] {
        let mut edit = operation(&s, brush.clone(), true);
        if let meshes::Edit::SculptStroke { detail: value, .. } = &mut edit {
            *value = detail;
        }
        let before = s.engine.snapshot();
        assert!(
            s.execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
                .is_err()
        );
        assert!(s.execute(Command::EditMesh { edit }.into()).is_err());
        assert_eq!(s.engine.snapshot(), before);
    }
}
