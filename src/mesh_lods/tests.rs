use super::*;
fn fixture() -> Meshes {
    let n = 12;
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
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
    // A separate small island must survive even if it has almost no area.
    let a = positions.len() as u32;
    positions.extend([[2., 1., 0.], [2.01, 1., 0.], [2., 1., 0.01]]);
    triangles.push([a, a + 2, a + 1]);
    Meshes {
        objects: Default::default(),
        assets: vec![Asset {
            id: "surface".into(),
            geometry: Arc::new(Geometry {
                positions,
                triangles,
                material_coordinates: None,
            }),
            shading: Shading::Smooth,
            material: Default::default(),
            rig: None,
            clips: vec![],
        }],
        instances: vec![],
    }
}
// Critical boundary: corrupted/stale/unproven data must never become a GPU source,
// and simplification must not erase disconnected coverage to claim a lower cost.
#[test]
fn binary_certificate_rejects_corruption_stale_source_and_lost_island() {
    let mesh = fixture();
    let compiled = compile(&mesh).unwrap();
    assert!(!compiled.packs.is_empty());
    let encoded = compiled.encode();
    assert_eq!(encoded.len(), compiled.byte_len());
    let loaded = Document::decode(&encoded, &mesh).unwrap();
    assert!(loaded.packs[0].matches(&mesh.assets[0]));
    let mut bad = encoded.clone();
    bad[20] ^= 1;
    assert!(Document::decode(&bad, &mesh).is_err());
    let mut stale = mesh.clone();
    Arc::make_mut(&mut stale.assets[0].geometry).positions[0][1] += 0.1;
    assert!(Document::decode(&encoded, &stale).is_err());
    assert!(loaded.filtered(&stale).packs.is_empty());
    let mut lost = compiled.clone();
    let p = Arc::make_mut(&mut lost.packs[0]);
    let last = (mesh.assets[0].geometry.triangles.len() - 1) * 3;
    for l in &mut p.levels {
        l.triangles
            .retain(|t| !t.iter().any(|&i| l.corners[i as usize] as usize >= last));
    }
    assert!(Document::decode(&lost.encode(), &mesh).is_err());
    let mut unproven = compiled.clone();
    let p = Arc::make_mut(&mut unproven.packs[0]);
    for l in &mut p.levels {
        l.error = 0.;
        l.normal_error = 0.;
    }
    assert!(Document::decode(&unproven.encode(), &mesh).is_err());
    assert!(Document::decode(&encoded[..encoded.len() - 1], &mesh).is_err());
    let mut tail = encoded[..encoded.len() - 32].to_vec();
    tail.push(0);
    let seal = ring::digest::digest(&ring::digest::SHA256, &tail);
    tail.extend(seal.as_ref());
    assert!(Document::decode(&tail, &mesh).is_err());
}
