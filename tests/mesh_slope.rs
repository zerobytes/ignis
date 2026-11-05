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
        wall_rock: None,
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
        random_heights: None,
    };
    let out = ignis::mesh_slope::lift(&g, &c, 0.).unwrap();
    assert_eq!(out.positions[1][1] - out.positions[0][1], 1.);
    assert_eq!(out.positions[2][1] - out.positions[0][1], 1200.);
    assert_eq!(out.triangles, g.triangles);
    for (a, b) in out.positions.iter().zip(&g.positions) {
        assert_eq!((a[0], a[2]), (b[0], b[2]));
    }
}

#[test]
fn random_levels_are_bounded_seeded_and_continuous_at_band_boundaries() {
    use ignis::mesh_slope::{RandomHeights, TerraceLift, lift};
    let g = Geometry {
        positions: (-4..=4)
            .map(|i| [0., i as f32 * 60., 0.])
            .chain([[1., 0., 0.]])
            .collect(),
        triangles: vec![[0, 1, 9]],
        material_coordinates: None,
    };
    let mut c = TerraceLift {
        spacing: 60.,
        factor: 1.,
        phase: 0.,
        random_heights: Some(RandomHeights {
            min: 3.,
            max: 15.,
            seed: 2718,
        }),
    };
    let a = lift(&g, &c, 0.).unwrap();
    let b = lift(&g, &c, 0.).unwrap();
    assert_eq!(a.positions, b.positions);
    for p in a.positions[..9].windows(2) {
        let d = p[1][1] - p[0][1];
        assert!((3. - 0.00001..=15. + 0.00001).contains(&d));
    }
    c.random_heights.as_mut().unwrap().seed = 42;
    assert_ne!(a.positions, lift(&g, &c, 0.).unwrap().positions);
    c.random_heights.as_mut().unwrap().min = 16.;
    assert!(lift(&g, &c, 0.).is_err());
}
#[test]
fn wall_relief_moves_interior_geometry_and_keeps_open_borders() {
    let positions = (0..3)
        .flat_map(|y| (0..3).map(move |x| [x as f32, y as f32, 0.]))
        .collect();
    let mut triangles = vec![];
    for y in 0..2 {
        for x in 0..2 {
            let a = y * 3 + x;
            triangles.extend([[a, a + 1, a + 4], [a, a + 4, a + 3]]);
        }
    }
    let g = Geometry {
        positions,
        triangles,
        material_coordinates: None,
    };
    let c = ignis::mesh_wall_rock::Config {
        amplitude: 0.65,
        stratum_height: 0.9,
        joint_width: 24.,
        seed: 2718,
    };
    let a = ignis::mesh_wall_rock::apply(&g, &g, &c, [0.; 3]).unwrap();
    assert_ne!(a.positions[4], g.positions[4]);
    for i in [0, 1, 2, 3, 5, 6, 7, 8] {
        assert_eq!(a.positions[i], g.positions[i]);
    }
    assert_eq!(a.triangles, g.triangles);
    assert_eq!(
        a.positions,
        ignis::mesh_wall_rock::apply(&g, &g, &c, [0.; 3])
            .unwrap()
            .positions
    );
}
