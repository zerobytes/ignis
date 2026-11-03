//! Critical untrusted binary boundary: reject stale/corrupt packets before publication.
use super::*;
use crate::mesh_objects::Object;
use std::sync::Arc;
fn fixture() -> crate::meshes::Document {
    serde_json::from_value(serde_json::json!({
        "assets":[{"id":"part","geometry":{"positions":[[-1,0,0],[1,0,0],[0,2,0]],"triangles":[[0,1,2]]}}],
        "instances":[{"id":"one","asset":"part","position":[0,1,0],"rotation":[0,0,0,1],"scale":1,"color":[0.2,0.6,0.1]}],
        "objects":[{"id":"object","members":["one"]}]
    })).unwrap()
}
#[test]
fn packet_rejects_corruption_truncation_stale_sources_and_invalid_symbols() {
    let mut doc = fixture();
    let object = Object {
        id: "object".into(),
        members: vec!["one".into()],
    };
    let pack = compile(&doc, &object, 8).unwrap();
    let bytes = pack.encode();
    let restored = Pack::decode(&bytes, &doc).unwrap();
    assert_eq!(restored.triangles, pack.triangles);
    assert_eq!(restored.source_triangles, 1);
    for n in [0, 7, 20, 64, bytes.len() - 1] {
        assert!(Pack::decode(&bytes[..n], &doc).is_err());
    }
    let mut corrupt = bytes.clone();
    let at = corrupt.len() / 2;
    corrupt[at] ^= 1;
    assert!(Pack::decode(&corrupt, &doc).is_err());
    // Resealed malicious palette references are still invalid.
    let mut invalid = bytes.clone();
    let payload_end = invalid.len() - 32;
    invalid[payload_end - 4..payload_end].copy_from_slice(&u32::MAX.to_le_bytes());
    let seal = ring::digest::digest(&ring::digest::SHA256, &invalid[..payload_end]);
    invalid[payload_end..].copy_from_slice(seal.as_ref());
    assert!(Pack::decode(&invalid, &doc).is_err());
    // A resealed packet must not create traversal cycles or shrink coverage bounds.
    let mut invalid = Pack::decode(&bytes, &doc).unwrap();
    invalid.nodes[0][3] = 0;
    assert!(Pack::decode(&invalid.encode(), &doc).is_err());
    let mut invalid = Pack::decode(&bytes, &doc).unwrap();
    invalid.nodes[0][0] = 0.0f32.to_bits();
    assert!(Pack::decode(&invalid.encode(), &doc).is_err());
    Arc::make_mut(&mut doc.assets[0].geometry).positions[0][0] += 0.1;
    assert!(Pack::decode(&bytes, &doc).is_err());
    let mut moved = fixture();
    moved.instances[0].position = [5., 6., 7.];
    moved.instances[0].color = [0.8, 0.1, 0.3];
    assert!(
        Pack::decode(&bytes, &moved).is_ok(),
        "Pose/tint are runtime inputs, not baked source"
    );
    moved.assets[0].material.transmission = 0.1;
    assert!(Pack::decode(&bytes, &moved).is_err());
}

#[test]
#[ignore = "requires native GPU; atomic installation/source fallback integration"]
fn failed_packet_install_and_source_mismatch_preserve_native_render() {
    use crate::{engine::Engine, model::RoomRecipe, render::Renderer};
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut doc = fixture();
    doc.assets[0].geometry = Arc::new(crate::meshes::Geometry::grid(16, 1.).unwrap());
    doc.instances[0].position = [0., 5., 0.];
    let mut duplicate = doc.instances[0].clone();
    duplicate.id = "offscreen".into();
    duplicate.position = [1000., 5., 0.];
    doc.instances.push(duplicate);
    scene.light.shadow = None;
    scene.light.environment = None;
    scene.light.atmosphere = None;
    for entity in &mut scene.entities {
        entity.visible = Some(false);
    }
    scene.meshes = doc.clone();
    scene.camera.position = [0., 8., 12.];
    scene.camera.target = [0., 5., 0.];
    let mut frame = Engine::new(scene).unwrap().render_frame();
    let pack = Arc::new(compile(&doc, &doc.objects[0], 8).unwrap());
    let instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&instance, None)).unwrap();
    let original = renderer.capture(&frame, 320, 240).unwrap();
    renderer.set_collective_packet(Some(pack.clone())).unwrap();
    renderer.set_collective_selection(0., false).unwrap();
    renderer.set_collective_density(0.25).unwrap();
    renderer.set_collective_enabled(true);
    let selected = renderer.capture(&frame, 320, 240).unwrap();
    assert_eq!(renderer.info()["collective_lod"]["selected_objects"], 1);
    let reference = crate::image_similarity::Image::decode(&original).unwrap();
    let actual = crate::image_similarity::Image::decode(&selected).unwrap();
    assert!(
        reference
            .pixels
            .iter()
            .zip(&actual.pixels)
            .all(|(a, b)| a.iter().zip(b).all(|(a, b)| a.abs_diff(*b) <= 2))
    );
    renderer.set_collective_raster_grid(256).unwrap();
    let raster =
        crate::image_similarity::Image::decode(&renderer.capture(&frame, 320, 240).unwrap())
            .unwrap();
    assert!(
        reference
            .pixels
            .iter()
            .zip(&raster.pixels)
            .all(|(a, b)| a.iter().zip(b).all(|(a, b)| a.abs_diff(*b) <= 2))
    );
    // A later group can veto the whole plan. It must not leave the earlier
    // group's CAMERA mask cleared and thereby remove valid source geometry.
    let mut guarded = doc.clone();
    let mut near = guarded.instances[0].clone();
    near.id = "near".into();
    near.position = [0., 8.05, 12.];
    guarded.instances.push(near);
    Arc::make_mut(&mut guarded.objects).push(Object {
        id: "near-object".into(),
        members: vec!["near".into()],
    });
    frame.meshes.update(&guarded, 0);
    renderer.set_collective_enabled(false);
    let native_guarded = renderer.capture(&frame, 320, 240).unwrap();
    renderer.set_collective_selection(0., true).unwrap();
    renderer.set_collective_enabled(true);
    assert_eq!(renderer.capture(&frame, 320, 240).unwrap(), native_guarded);
    assert_eq!(renderer.info()["collective_lod"]["near_guard"], true);
    assert_eq!(renderer.info()["collective_lod"]["selected_objects"], 0);
    frame.meshes.update(&doc, 0);
    renderer.set_collective_selection(0., false).unwrap();
    renderer.set_collective_density(4096.).unwrap();
    renderer.set_collective_enabled(true);
    assert_eq!(renderer.capture(&frame, 320, 240).unwrap(), original);
    assert_eq!(renderer.info()["collective_lod"]["selected_objects"], 0);
    let mut broken = Pack::decode(&pack.encode(), &doc).unwrap();
    broken.nodes.clear();
    assert!(
        renderer
            .set_collective_packet(Some(Arc::new(broken)))
            .is_err()
    );
    assert_eq!(renderer.capture(&frame, 320, 240).unwrap(), original);
    Arc::make_mut(&mut doc.assets[0].geometry).positions[0][1] += 0.3;
    frame.meshes.update(&doc, 0);
    renderer.set_collective_enabled(false);
    let changed = renderer.capture(&frame, 320, 240).unwrap();
    renderer.set_collective_selection(0., false).unwrap();
    renderer.set_collective_density(0.25).unwrap();
    renderer.set_collective_enabled(true);
    assert_eq!(renderer.capture(&frame, 320, 240).unwrap(), changed);
    assert_eq!(renderer.info()["collective_lod"]["selected_objects"], 0);
}
