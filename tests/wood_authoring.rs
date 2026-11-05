use ignis::{
    botany::{self as b, bake},
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
use std::collections::BTreeMap;
fn specimen() -> b::Recipe {
    serde_json::from_str(include_str!("../assets/botany/wood-authoring.ignis-botany")).unwrap()
}
fn op(s: &mut Service, operation: b::Operation) -> ignis::model::Result<ignis::service::Reply> {
    s.execute(
        Command::Botany {
            operation: Box::new(operation),
        }
        .into(),
    )
}
fn closed(r: &b::Recipe) {
    let compiled = bake::compile(r).unwrap();
    let g = &compiled.document.assets[0].geometry;
    let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
    let mut volume = 0f64;
    for &[a, b, c] in &g.triangles {
        for (a, b) in [(a, b), (b, c), (c, a)] {
            let e = edges.entry((a.min(b), a.max(b))).or_default();
            e.0 += 1;
            e.1 += if a < b { 1 } else { -1 };
        }
        let p = glam::DVec3::from_array(g.positions[a as usize].map(f64::from));
        let q = glam::DVec3::from_array(g.positions[b as usize].map(f64::from));
        let t = glam::DVec3::from_array(g.positions[c as usize].map(f64::from));
        assert!((q - p).cross(t - p).length() > 1e-10);
        volume += p.dot(q.cross(t)) / 6.;
    }
    assert!(
        edges.values().all(|&(n, w)| n == 2 && w == 0),
        "unclosed woody skin"
    );
    assert!(volume > 1.);
}
#[test]
fn wood_sections_relief_and_junctions_are_closed_and_roundtrip() {
    let r = specimen();
    r.validate().unwrap();
    closed(&r);
    let encoded = serde_json::to_vec(&r).unwrap();
    let saved: b::Recipe = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        bake::compile(&r).unwrap().document,
        bake::compile(&saved).unwrap().document
    );
    let mut disabled = r.clone();
    let surface = disabled
        .edits
        .get_mut("trunk")
        .unwrap()
        .wood_surface
        .as_mut()
        .unwrap();
    let baseline = surface.radius(0.3, 0.26);
    surface.relief[0].enabled = false;
    assert!((surface.radius(0.3, 0.26) - baseline).abs() > 0.1);
    closed(&disabled);
    let mut old = r;
    old.version = 9;
    assert!(old.validate().is_err());
}
#[test]
fn wood_editor_transactions_preserve_last_valid_state() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let r = specimen();
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(r.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let published = s.engine.scene.clone();
    let revision = s.engine.revision;
    let mut bad = r.clone();
    bad.edits
        .get_mut("trunk")
        .unwrap()
        .wood_surface
        .as_mut()
        .unwrap()
        .relief[0]
        .height = -9.;
    assert!(
        op(
            &mut s,
            b::Operation::Publish {
                recipe: Box::new(bad),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(revision, s.engine.revision);
    assert_eq!(s.engine.scene, published);
    let mut next = r;
    next.edits
        .get_mut("trunk")
        .unwrap()
        .wood_surface
        .as_mut()
        .unwrap()
        .sections[1]
        .offset[0] += 0.05;
    op(
        &mut s,
        b::Operation::Preview {
            recipe: Box::new(next.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(s.engine.scene, published);
    op(&mut s, b::Operation::CancelPreview).unwrap();
    assert_eq!(s.engine.scene, published);
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(next),
            policy: Default::default(),
        },
    )
    .unwrap();
    let changed = s.engine.scene.clone();
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(s.engine.scene, published);
    s.execute(Command::Redo.into()).unwrap();
    assert_eq!(s.engine.scene, changed);
}

#[test]
fn rounded_root_crown_uses_same_surface_and_remains_closed() {
    let r = specimen();
    let roots = b::roots::Settings {
        count: 4,
        seed: 91,
        spread: 10.,
        depth: 0.8,
        radius_ratio: 0.5,
        neck_height: 1.25,
        exposure: 1.,
        orders: 0,
        irregularity: 0.13,
        ..Default::default()
    };
    let rooted = b::roots::configure(&r, "trunk", Some(roots)).unwrap();
    closed(&rooted);
}

#[test]
fn shaped_wood_preserves_material_and_sculpt_and_refuses_resolution_loss() {
    use ignis::materials::library::{Execution, Operation as MaterialOp, Preset};
    use std::sync::Arc;
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let r = specimen();
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(r),
            policy: Default::default(),
        },
    )
    .unwrap();
    s.execute(
        Command::Material {
            operation: Box::new(MaterialOp::Preset {
                asset: "woodlab_wood".into(),
                preset: Preset::Wood,
                seed: Some(71),
                unique_instances: false,
                execution: Execution::Fixed { resolution: 16 },
            }),
        }
        .into(),
    )
    .unwrap();
    let material = s
        .engine
        .scene
        .meshes
        .asset("woodlab_wood")
        .unwrap()
        .material
        .clone();
    let mut asset = s.engine.scene.meshes.asset("woodlab_wood").unwrap().clone();
    Arc::make_mut(&mut asset.geometry).positions[31][0] += 0.001;
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        }
        .into(),
    )
    .unwrap();
    let mut next = s.engine.scene.botany[0].clone();
    next.edits
        .get_mut("trunk")
        .unwrap()
        .wood_surface
        .as_mut()
        .unwrap()
        .relief[0]
        .height += 0.01;
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(next),
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(s.engine.scene.botany[0].manual.len(), 1);
    assert_eq!(
        s.engine
            .scene
            .meshes
            .asset("woodlab_wood")
            .unwrap()
            .material,
        material
    );
    let before = s.engine.scene.clone();
    let revision = s.engine.revision;
    let mut bad = s.engine.scene.botany[0].clone();
    bad.edits
        .get_mut("trunk")
        .unwrap()
        .wood_surface
        .as_mut()
        .unwrap()
        .sides = 24;
    assert!(
        op(
            &mut s,
            b::Operation::Publish {
                recipe: Box::new(bad),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(s.engine.scene, before);
    assert_eq!(s.engine.revision, revision);
}

#[test]
fn wide_wood_has_no_fixed_artistic_radius_ceiling() {
    let mut r = specimen();
    for o in &mut std::sync::Arc::make_mut(&mut r.source).organs {
        for p in &mut o.path {
            for x in p {
                *x *= 100.;
            }
        }
        if let b::Shape::Axis { radius, .. } = &mut o.shape {
            *radius *= 100.;
        }
    }
    let surface = r
        .edits
        .get_mut("trunk")
        .unwrap()
        .wood_surface
        .as_mut()
        .unwrap();
    for section in &mut surface.sections {
        for x in &mut section.radii {
            *x *= 100.;
        }
        for x in &mut section.offset {
            *x *= 100.;
        }
    }
    for relief in &mut surface.relief {
        relief.width *= 100.;
        relief.height *= 100.;
    }
    r.settings.radius = 8.;
    r.validate().unwrap();
    r.settings.radius = 1.;
    closed(&r);
    // Numeric/coordinate validity remains distinct from an artistic diameter ceiling.
    r.edits
        .get_mut("trunk")
        .unwrap()
        .wood_surface
        .as_mut()
        .unwrap()
        .sections[0]
        .radii[0] = f64::INFINITY;
    assert!(r.validate().is_err());
}

#[test]
fn junction_refinement_is_local_versioned_and_transactional() {
    let r = specimen();
    let base = bake::compile(&r).unwrap();
    let mut same = r.clone();
    same.version = 11;
    same.edits.get_mut("trunk").unwrap().junction_refinement = Some(1);
    assert_eq!(bake::compile(&same).unwrap().document, base.document);
    let mut detailed = same.clone();
    detailed.edits.get_mut("trunk").unwrap().junction_refinement = Some(2);
    closed(&detailed);
    let next = bake::compile(&detailed).unwrap();
    assert!(
        next.document.assets[0].geometry.positions.len()
            > base.document.assets[0].geometry.positions.len()
    );
    let body = |c: &bake::Compiled| {
        c.vertices[0]
            .iter()
            .zip(&c.document.assets[0].geometry.positions)
            .filter(|(v, _)| v.key.contains(":wall") && v.key.contains('@'))
            .map(|(v, p)| (v.key.clone(), *p))
            .collect::<BTreeMap<_, _>>()
    };
    let original = body(&base);
    assert!(!original.is_empty());
    assert_eq!(original, body(&next));
    let encoded = serde_json::to_vec(&detailed).unwrap();
    let round: b::Recipe = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(bake::compile(&round).unwrap().document, next.document);
    let mut invalid = detailed.clone();
    invalid.version = 10;
    assert!(invalid.validate().is_err());
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    op(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(r.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let before = service.engine.scene.clone();
    op(
        &mut service,
        b::Operation::Preview {
            recipe: Box::new(detailed.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(service.engine.scene, before);
    op(&mut service, b::Operation::CancelPreview).unwrap();
    assert_eq!(service.engine.scene, before);
    op(
        &mut service,
        b::Operation::EditOrgan {
            prefix: r.prefix.clone(),
            organ: "trunk".into(),
            edit: detailed.edits["trunk"].clone(),
        },
    )
    .unwrap();
    let after = service.engine.scene.clone();
    assert_eq!(after.botany[0].version, b::VERSION);
    let mut bad = detailed.edits["trunk"].clone();
    bad.junction_refinement = Some(4);
    assert!(
        op(
            &mut service,
            b::Operation::EditOrgan {
                prefix: r.prefix.clone(),
                organ: "trunk".into(),
                edit: bad
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, after);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, before);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, after);
}
