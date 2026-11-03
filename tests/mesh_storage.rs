use ignis::{
    engine::{Command, Engine},
    meshes::{self, Asset, Geometry},
    model::RoomRecipe,
};
use std::sync::Arc;
fn torus() -> Geometry {
    let (nx, ny) = (256_u32, 64_u32);
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    for x in 0..nx {
        let a = x as f32 * std::f32::consts::TAU / nx as f32;
        for y in 0..ny {
            let b = y as f32 * std::f32::consts::TAU / ny as f32;
            let r = 2. + 0.5 * b.cos();
            positions.push([r * a.cos(), 0.5 * b.sin(), r * a.sin()]);
        }
    }
    for x in 0..nx {
        for y in 0..ny {
            let a = x * ny + y;
            let b = ((x + 1) % nx) * ny + y;
            let c = x * ny + (y + 1) % ny;
            let d = ((x + 1) % nx) * ny + (y + 1) % ny;
            triangles.extend([[a, c, b], [b, c, d]]);
        }
    }
    Geometry {
        positions,
        triangles,
        material_coordinates: None,
    }
}
fn engine() -> Engine {
    Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap()
}
fn put(
    e: &mut Engine,
    id: &str,
    g: Arc<Geometry>,
) -> Result<serde_json::Value, ignis::model::Error> {
    e.execute(
        Command::EditMesh {
            edit: meshes::Edit::PutAsset {
                asset: Asset {
                    id: id.into(),
                    geometry: g,
                    material: Default::default(),
                    shading: meshes::Shading::Smooth,
                    rig: None,
                    clips: vec![],
                },
            },
        }
        .into(),
    )
}
#[test]
fn full_closed_assets_grow_beyond_legacy_storage_and_preserve_version_and_undo() {
    let g = Arc::new(torus());
    assert_eq!(g.positions.len(), 16384);
    assert_eq!(g.triangles.len(), 32768);
    let topo = ignis::mesh_topology::Topology::inspect(&g).unwrap();
    assert_eq!(topo.boundary_edges, 0);
    assert_eq!(topo.non_manifold_edges, 0);
    assert_eq!(
        topo.vertices as isize - topo.edges.len() as isize + topo.faces.len() as isize,
        0
    );
    let mut e = engine();
    for i in 0..2 {
        put(&mut e, &format!("torus{i}"), g.clone()).unwrap();
    }
    e.scene.validate().unwrap();
    let scene = e.scene.clone();
    let mut old = scene.clone();
    old.version = 66;
    assert!(old.validate().unwrap_err().message.contains("version67"));
    put(&mut e, "additional", g).unwrap();
    assert_eq!(e.scene.meshes.assets.len(), 3);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, scene);
    let parsed = serde_json::from_slice(&serde_json::to_vec(&scene).unwrap()).unwrap();
    assert_eq!(Engine::new(parsed).unwrap().scene, scene);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene.meshes.assets.len(), 1);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, scene);
    for case in 0..2 {
        let mut g = torus();
        if case == 0 {
            g.positions.push([0., 0., 0.]);
        } else {
            g.triangles.push(g.triangles[0]);
        }
        g.validate().unwrap();
    }
}
#[test]
fn legacy_source_cannot_hide_extended_storage_behind_a_coarse_evaluated_mesh() {
    let mut e = engine();
    let mut d: ignis::plants::Definition = serde_json::from_value(serde_json::json!({"id":"plant","profile":{"id":"profile","width":1,"height":2},"depth":1,"segments":[27,27,27],"sampling_tolerance":0.001})).unwrap();
    for segments in [[27, 27, 27], [1, 1, 1]] {
        d.segments = segments;
        e.execute(
            Command::EditPlant {
                edit: ignis::plants::Edit::Put {
                    definition: d.clone(),
                    detail_policy: ignis::mesh_preservation::DisplacementPolicy::Physical,
                    animation_translation_policy:
                        ignis::mesh_preservation::DisplacementPolicy::Physical,
                },
            }
            .into(),
        )
        .unwrap();
    }
    assert_eq!(e.scene.meshes.assets[0].geometry.positions.len(), 24);
    assert!(e.scene.plants.entries[0].source.geometry.positions.len() > 4096);
    e.scene.validate().unwrap();
    let mut old = e.scene.clone();
    old.version = 65;
    assert!(old.validate().unwrap_err().message.contains("version66"));
}
