//! Critical publication boundary: malformed/stale sidecars must not replace a
//! usable package or alter source/physics/simulation. No UI flag mirror tests.
use ignis::{engine::Command, model::RoomRecipe, service::Service};
use serde_json::json;
#[test]
fn sidecar_publication_is_atomic_and_does_not_edit_the_scene() {
    let root = tempfile::tempdir().unwrap();
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let n = 12u32;
    let mut positions = vec![];
    let mut triangles = vec![];
    for z in 0..=n {
        for x in 0..=n {
            positions.push([x as f32 / 20., 1., z as f32 / 20.]);
        }
    }
    for z in 0..n {
        for x in 0..n {
            let a = z * (n + 1) + x;
            triangles.extend([[a, a + n + 1, a + 1], [a + 1, a + n + 1, a + n + 2]]);
        }
    }
    scene.meshes=serde_json::from_value(json!({"assets":[{"id":"surface","shading":"smooth","geometry":{"positions":positions,"triangles":triangles}}],"instances":[{"id":"surface","asset":"surface","position":[0.,2.,0.],"rotation":[0.,0.,0.,1.],"scale":1.,"color":[0.5,0.4,0.3]}]})).unwrap();
    let source = serde_json::to_value(&scene).unwrap();
    let mut service = Service::new(root.path(), scene).unwrap();
    let compiled = service
        .execute(
            Command::CompileMeshLods {
                path: "current.ignis-lod".into(),
            }
            .into(),
        )
        .unwrap()
        .data;
    assert_eq!(compiled["output_written"], true);
    let before = service
        .execute(Command::InspectMeshLods.into())
        .unwrap()
        .data;
    std::fs::write(root.path().join("bad.ignis-lod"), b"invalid").unwrap();
    assert!(
        service
            .execute(
                Command::LoadMeshLods {
                    path: "bad.ignis-lod".into()
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(
        service
            .execute(Command::InspectMeshLods.into())
            .unwrap()
            .data,
        before
    );
    assert!(
        service
            .execute(
                Command::CompileMeshLods {
                    path: "scene.json".into()
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(serde_json::to_value(&service.engine.scene).unwrap(), source);
    service
        .execute(
            Command::LoadMeshLods {
                path: "current.ignis-lod".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(
        service
            .execute(Command::InspectMeshLods.into())
            .unwrap()
            .data,
        before
    );
    assert_eq!(serde_json::to_value(&service.engine.scene).unwrap(), source);
    assert_eq!(service.engine.simulation.tick, 0);
}
