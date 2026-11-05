use ignis::{
    engine::{Command, Engine, SceneEdit},
    mesh_slope::Style,
    meshes::{Asset, Edit, Geometry, Instance, Shading},
    model::RoomRecipe,
};
use std::sync::Arc;
#[test]
fn disjoint_partition_preserves_triangles_and_undo() {
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    let geometry = Arc::new(Geometry {
        positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., 1.], [0., 1., 0.]],
        triangles: vec![[0, 2, 1], [0, 1, 3], [0, 3, 2]],
        material_coordinates: None,
    });
    let a = Asset {
        id: "source".into(),
        geometry: geometry.clone(),
        shading: Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    };
    let i = Instance {
        id: "source".into(),
        asset: "source".into(),
        position: [10., 0., 0.],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [1.; 3],
        playback: None,
        attachment: None,
    };
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Mesh {
                    edit: Edit::PutAsset { asset: a },
                },
                SceneEdit::Mesh {
                    edit: Edit::PutInstance { instance: i },
                },
            ],
        }
        .into(),
    )
    .unwrap();
    let before = serde_json::to_value(&e.scene).unwrap();
    let style = Style {
        instance: "source".into(),
        wall_asset: "walls".into(),
        top_asset: "tops".into(),
        top_instance: "tops".into(),
        min_up: 0.8,
        chamfer: None,
        terrace_lift: None,
        walls: Default::default(),
        top: Default::default(),
        wall_color: [1.; 3],
        top_color: [0., 1., 0.],
    };
    let report = e
        .execute(
            Command::StyleMeshBySlope {
                style: Box::new(style),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(report["partition"]["source_triangles"], 3);
    assert_eq!(report["partition"]["top_triangles"], 1);
    assert_eq!(report["partition"]["wall_triangles"], 2);
    assert_eq!(
        e.scene
            .meshes
            .assets
            .iter()
            .map(|a| a.geometry.triangles.len())
            .sum::<usize>(),
        3
    );
    for a in &e.scene.meshes.assets {
        assert!(a.geometry.material_coordinates.is_some());
    }
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(before, serde_json::to_value(&e.scene).unwrap());
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene.meshes.instances.len(), 2);
}

#[test]
fn lifting_changes_band_separation_without_stretching_local_relief() {
    let g = Geometry {
        positions: vec![[0., 30., 0.], [1., 31., 0.], [0., 90., 1.]],
        triangles: vec![[0, 1, 2]],
        material_coordinates: None,
    };
    let c = ignis::mesh_slope::TerraceLift {
        spacing: 60.,
        factor: 20.,
        phase: 0.,
    };
    let out = ignis::mesh_slope::lift(&g, &c, 0.).unwrap();
    assert_eq!(out.positions[1][1] - out.positions[0][1], 1.);
    assert_eq!(out.positions[2][1] - out.positions[0][1], 1200.);
    assert_eq!(out.triangles, g.triangles);
    for (a, b) in out.positions.iter().zip(&g.positions) {
        assert_eq!((a[0], a[2]), (b[0], b[2]));
    }
}
