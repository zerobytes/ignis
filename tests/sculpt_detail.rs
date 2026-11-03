//! Critical regressions: opposite diagonal aliasing, scoped topology, metadata and atomic refusal.
use glam::{Vec2, Vec3};
use ignis::{
    engine::Command,
    mesh_tools::Primitive,
    meshes::{self, BrushKind, Stroke},
    model::RoomRecipe,
    service::Service,
};
use std::sync::Arc;
fn service() -> (tempfile::TempDir, Service) {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(
        root.path(),
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    s.execute(
        Command::CreateEditorMesh {
            primitive: Primitive::Cube,
            resolution: 24,
            size: 2.,
        }
        .into(),
    )
    .unwrap();
    (root, s)
}
fn stroke(slope: f32, strength: f32) -> Stroke {
    let mut centers = vec![];
    let mut t = -0.6;
    while t <= 0.6 {
        centers.push([-1., t, t * slope]);
        t += 0.241 * 0.15 / 2f32.sqrt();
    }
    Stroke {
        kind: BrushKind::Draw,
        centers,
        radius: 0.241,
        strength,
        direction: [-1., 0., 0.],
    }
}
fn edit(s: &Service, stroke: Stroke) -> meshes::Edit {
    meshes::Edit::SculptDetailed {
        asset: s.engine.scene.meshes.assets[0].id.clone(),
        reference: s.engine.mesh_reference(),
        max_edge: stroke.radius / 8.,
        stroke,
    }
}
/// Sample actual piecewise-linear geometry along the ridge, not normals or shadow pixels.
fn ridge_variation(g: &meshes::Geometry, slope: f32) -> f32 {
    let faces: Vec<_> = g
        .triangles
        .iter()
        .filter(|f| f.iter().all(|i| g.positions[*i as usize][0] < -0.9999))
        .map(|f| f.map(|i| Vec3::from(g.positions[i as usize])))
        .collect();
    let mut heights = vec![];
    for i in 0..=240 {
        let t = -0.3 + i as f32 / 240. * 0.6;
        let point = Vec2::new(t, t * slope);
        let mut x = f32::INFINITY;
        for &[a, b, c] in &faces {
            let ab = Vec2::new(b.y - a.y, b.z - a.z);
            let ac = Vec2::new(c.y - a.y, c.z - a.z);
            let ap = point - Vec2::new(a.y, a.z);
            let den = ab.perp_dot(ac);
            if den.abs() < 1e-12 {
                continue;
            }
            let u = ap.perp_dot(ac) / den;
            let v = ab.perp_dot(ap) / den;
            if u >= -1e-5 && v >= -1e-5 && u + v <= 1.00001 {
                x = x.min(a.x + u * (b.x - a.x) + v * (c.x - a.x));
            }
        }
        assert!(x.is_finite());
        heights.push(-x - 1.);
    }
    let min = heights.iter().copied().fold(f32::INFINITY, f32::min);
    let max = heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mean = heights.iter().sum::<f32>() / heights.len() as f32;
    assert!(mean > 0.1);
    (max - min) / mean
}
#[test]
fn detailed_draw_removes_opposite_diagonal_teeth_before_painting_and_undo_is_one_transaction() {
    let (_root, mut s) = service();
    let base = s.engine.scene.clone();
    let raw = ignis::mesh_tools::sculpt_preview(&base.meshes.assets[0], &stroke(1., 0.04)).unwrap();
    assert!(
        ridge_variation(&raw.geometry, 1.) > 0.1,
        "fixture must reproduce the actual teeth"
    );
    for slope in [-1., 1.] {
        for strength in [0.04, 0.12] {
            let before = s.engine.snapshot();
            let e = edit(&s, stroke(slope, strength));
            let preview = s
                .execute(Command::PreviewMeshEdit { edit: e.clone() }.into())
                .unwrap()
                .data;
            assert_eq!(s.engine.snapshot(), before);
            s.execute(Command::EditMesh { edit: e }.into()).unwrap();
            assert_eq!(
                serde_json::to_value(&s.engine.scene.meshes).unwrap(),
                preview["meshes"]
            );
            let g = &s.engine.scene.meshes.assets[0].geometry;
            let variation = ridge_variation(g, slope);
            println!(
                "slope{slope} force{strength}:{}v/{}t ridge variation{variation}",
                g.positions.len(),
                g.triangles.len()
            );
            assert!(
                variation < 0.015,
                "visible diagonal teeth remain: {variation}"
            );
            assert!(g.positions.len() > base.meshes.assets[0].geometry.positions.len());
            assert_eq!(
                &g.positions[..base.meshes.assets[0].geometry.positions.len()]
                    .iter()
                    .filter(|p| p[0] > 0.)
                    .copied()
                    .collect::<Vec<_>>(),
                &base.meshes.assets[0]
                    .geometry
                    .positions
                    .iter()
                    .filter(|p| p[0] > 0.)
                    .copied()
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                s.engine.snapshot()["history"]["undo"].as_u64(),
                Some(before["history"]["undo"].as_u64().unwrap() + 1)
            );
            s.execute(Command::Undo.into()).unwrap();
            assert_eq!(s.engine.scene, base);
        }
    }
}
#[test]
fn detailed_sculpt_refuses_stale_scope_and_budget_without_partial_topology_or_history() {
    let (_root, mut s) = service();
    let stale = edit(&s, stroke(1., 0.04));
    s.execute(Command::Reset.into()).unwrap();
    let before = s.engine.snapshot();
    assert!(s.execute(Command::EditMesh { edit: stale }.into()).is_err());
    assert_eq!(s.engine.snapshot(), before);
    let instance = s.engine.scene.meshes.instances[0].clone();
    for i in 1..16 {
        let mut copy = instance.clone();
        copy.id = format!("copy{i}");
        s.execute(
            Command::EditMesh {
                edit: meshes::Edit::PutInstance { instance: copy },
            }
            .into(),
        )
        .unwrap();
    }
    let before = s.engine.snapshot();
    let e = edit(&s, stroke(1., 0.04));
    assert!(
        s.execute(Command::PreviewMeshEdit { edit: e.clone() }.into())
            .is_err()
    );
    assert!(s.execute(Command::EditMesh { edit: e }.into()).is_err());
    assert_eq!(s.engine.snapshot(), before);
}
#[test]
fn detail_preserves_material_rig_clips_and_interpolates_new_vertex_channels() {
    let (_root, mut s) = service();
    let mut a = s.engine.scene.meshes.assets[0].clone();
    Arc::make_mut(&mut a.geometry).material_coordinates = Some(
        a.geometry
            .positions
            .iter()
            .map(|p| [p[1] * 2. + 0.3, p[2] * 0.8 + 0.2, 0.])
            .collect(),
    );
    a.rig = Some(Arc::new(meshes::Rig {
        bones: vec![
            meshes::Bone {
                name: "root".into(),
                parent: None,
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            },
            meshes::Bone {
                name: "child".into(),
                parent: Some(0),
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            },
        ],
        weights: a
            .geometry
            .positions
            .iter()
            .map(|p| meshes::Weight {
                extra: None,
                joints: [0, 1, 0, 0],
                weights: [(p[2] + 1.) * 0.5, (1. - p[2]) * 0.5, 0., 0.],
            })
            .collect(),
    }));
    a.clips = vec![Arc::new(meshes::Clip {
        id: "idle".into(),
        duration_ticks: 60,
        tracks: vec![meshes::Track {
            bone: 0,
            keys: vec![meshes::Keyframe {
                tick: 0,
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            }],
        }],
    })];
    s.execute(
        Command::EditMesh {
            edit: meshes::Edit::PutAsset { asset: a.clone() },
        }
        .into(),
    )
    .unwrap();
    let e = edit(&s, stroke(1., 0.04));
    s.execute(Command::EditMesh { edit: e }.into()).unwrap();
    let b = &s.engine.scene.meshes.assets[0];
    assert_eq!(b.material, a.material);
    assert_eq!(b.shading, a.shading);
    assert_eq!(b.clips, a.clips);
    assert_eq!(b.rig.as_ref().unwrap().bones, a.rig.as_ref().unwrap().bones);
    assert_eq!(
        &b.rig.as_ref().unwrap().weights[..a.geometry.positions.len()],
        &a.rig.as_ref().unwrap().weights
    );
    for (i, p) in b.geometry.positions.iter().enumerate() {
        let uv = b.geometry.material_coordinates.as_ref().unwrap()[i];
        assert!((uv[0] - (p[1] * 2. + 0.3)).abs() < 1e-5);
        assert!((uv[1] - (p[2] * 0.8 + 0.2)).abs() < 1e-5);
        assert!((b.rig.as_ref().unwrap().weights[i].weights[0] - (p[2] + 1.) * 0.5).abs() < 1e-5);
    }
}

#[test]
fn detailed_sculpt_rejects_invalid_staged_indices_without_panicking_or_publishing() {
    let (_root, mut s) = service();
    let before = s.engine.snapshot();
    let mut asset = s.engine.scene.meshes.assets[0].clone();
    Arc::make_mut(&mut asset.geometry).triangles[0][0] = u32::MAX;
    let edits = vec![
        ignis::engine::SceneEdit::Mesh {
            edit: meshes::Edit::PutAsset { asset },
        },
        ignis::engine::SceneEdit::Mesh {
            edit: edit(&s, stroke(1., 0.04)),
        },
    ];
    assert!(s.execute(Command::EditScene { edits }.into()).is_err());
    assert_eq!(s.engine.snapshot(), before);
}
