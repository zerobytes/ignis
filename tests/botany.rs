//! Critical source/bake atomicity, semantic identity and persistence contracts.
use ignis::{
    botany::{self as b, bake, dynamics},
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
fn fixture() -> b::Recipe {
    let axis = b::Organ {
        id: "stem".into(),
        parent: None,
        life: None,
        insertion: 0.,
        path: vec![[0.; 3], [0., 1., 0.]],
        normal: [1., 0., 0.],
        twist: 0.,
        taper_power: 1.,
        connection: Default::default(),
        shape: b::Shape::Axis {
            radius: 0.02,
            tip_ratio: 0.5,
            ellipticity: 1.,
        },
        birth: 0.,
        duration: 3.,
        density: 600.,
        young: 1e7,
        shear: 4e6,
    };
    let leaf = b::Organ {
        id: "leaf".into(),
        parent: Some("stem".into()),
        life: None,
        insertion: 0.7,
        path: vec![[0., 0.7, 0.], [0.3, 0.9, 0.]],
        normal: [0., 0., 1.],
        twist: 0.,
        taper_power: 1.,
        connection: Default::default(),
        shape: b::Shape::Blade {
            width: 0.12,
            thickness: 0.0003,
            fold: 0.1,
            cup: 0.1,
            outline: b::Outline::Strip {
                base_power: 0.5,
                tip_power: 1.,
            },
        },
        birth: 0.5,
        duration: 1.,
        density: 850.,
        young: 2e7,
        shear: 8e6,
    };
    b::Recipe {
        wood_details: None,
        version: 1,
        prefix: "plant".into(),
        source: Arc::new(b::Source {
            version: 1,
            name: "fixture".into(),
            seed: 42,
            organs: vec![axis, leaf],
            development: None,
            root_systems: vec![],
        }),
        settings: Default::default(),
        edits: BTreeMap::new(),
        pruned: BTreeSet::new(),
        manual: BTreeMap::new(),
            living: None,
    }
}
fn op(s: &mut Service, operation: b::Operation) -> ignis::model::Result<ignis::service::Reply> {
    s.execute(
        Command::Botany {
            operation: Box::new(operation),
        }
        .into(),
    )
}
#[test]
fn authoring_preserves_semantic_sculpt_and_rejects_loss_atomically() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let r = fixture();
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(r.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let scene = s.engine.scene.clone();
    let revision = s.engine.revision;
    let mut preview = r.clone();
    preview.settings.scale = 1.2;
    op(
        &mut s,
        b::Operation::Preview {
            recipe: Box::new(preview),
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(s.engine.scene, scene);
    assert_eq!(s.engine.revision, revision);
    op(&mut s, b::Operation::CancelPreview).unwrap();
    assert!(s.botany.overlay.is_none());
    let mut asset = s.engine.scene.meshes.asset("plant_leaves").unwrap().clone();
    Arc::make_mut(&mut asset.geometry).positions[12][2] += 0.015;
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        }
        .into(),
    )
    .unwrap();
    let mut altered = r.clone();
    altered.settings.leaf_length = 1.3;
    altered
        .edits
        .entry("stem".into())
        .or_default()
        .deformation
        .tip = [0.8, 0.4];
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(altered),
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(s.engine.scene.botany[0].manual.len(), 1);
    let clean = bake::compile(&{
        let mut r = s.engine.scene.botany[0].clone();
        r.manual.clear();
        r
    })
    .unwrap();
    assert_ne!(
        s.engine
            .scene
            .meshes
            .asset("plant_leaves")
            .unwrap()
            .geometry
            .positions[12],
        clean.document.assets[1].geometry.positions[12]
    );
    let before = s.engine.scene.clone();
    let revision = s.engine.revision;
    let mut loss = s.engine.scene.botany[0].clone();
    loss.settings.leaf_density = 0.;
    assert!(
        op(
            &mut s,
            b::Operation::Publish {
                recipe: Box::new(loss),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(s.engine.scene, before);
    assert_eq!(s.engine.revision, revision);
    op(
        &mut s,
        b::Operation::Save {
            prefix: "plant".into(),
            path: "plant.ignis-botany".into(),
        },
    )
    .unwrap();
    let saved: b::Recipe =
        serde_json::from_slice(&std::fs::read(root.path().join("plant.ignis-botany")).unwrap())
            .unwrap();
    assert_eq!(saved, s.engine.scene.botany[0]);
    let decoded: ignis::model::Scene =
        serde_json::from_slice(&serde_json::to_vec(&s.engine.scene).unwrap()).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded, s.engine.scene);
    s.execute(Command::Undo.into()).unwrap();
    assert!(s.engine.scene.botany[0].manual.is_empty());
    s.execute(Command::Redo.into()).unwrap();
    assert_eq!(s.engine.scene, before);
    let mut loss = s.engine.scene.botany[0].clone();
    loss.settings.leaf_density = 0.;
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(loss),
            policy: b::ManualPolicy::Discard,
        },
    )
    .unwrap();
    assert!(s.engine.scene.botany[0].manual.is_empty());
    let mut placeholder = s.engine.scene.meshes.asset("plant_leaves").unwrap().clone();
    Arc::make_mut(&mut placeholder.geometry).positions[0][0] += 0.03;
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset: placeholder },
        }
        .into(),
    )
    .unwrap();
    let recipe = s.engine.scene.botany[0].clone();
    let before = s.engine.scene.clone();
    assert!(
        op(
            &mut s,
            b::Operation::Publish {
                recipe: Box::new(recipe),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(s.engine.scene, before);
}
#[test]
fn exemplar_sources_compile_continuously_and_are_reproducible() {
    for id in ["palmeirinha", "platano", "eucalipto", "figueira"] {
        check_exemplar(id);
    }
}
// New preset exercises mixed woody forks and independent, flexible epiphyte bodies.
#[test]
fn figueira_rs_closed_skin_determinism_and_material_attachments() {
    check_exemplar("figueira_rs");
    let mut bare: b::Recipe =
        serde_json::from_slice(&std::fs::read("assets/botany/figueira_rs.ignis-botany").unwrap())
            .unwrap();
    bare.settings.leaf_density = 0.;
    bare.pruned.extend(
        bare.source
            .organs
            .iter()
            .filter(|o| o.id.contains("/epifita") && !o.id.contains("/fio"))
            .map(|o| o.id.clone()),
    );
    let c = bake::compile(&bare).unwrap();
    assert_closed_exemplar(&c, "figueira_rs/bare");
}
fn check_exemplar(id: &str) {
    let r: b::Recipe =
        serde_json::from_slice(&std::fs::read(format!("assets/botany/{id}.ignis-botany")).unwrap())
            .unwrap();
    let c = bake::compile(&r).unwrap_or_else(|e| panic!("{id}: {e}"));
    if id != "palmeirinha" {
        assert!(c.joints > 0);
    }
    c.document.validate().unwrap();
    let d = bake::compile(&r).unwrap();
    assert_eq!(c.document, d.document);
    if id == "figueira" || id == "figueira_rs" {
        let mut session = dynamics::Session::new(
            r.clone(),
            c.clone(),
            dynamics::Settings {
                age: 1.8,
                gravity: [0.; 3],
                ground: None,
                iterations: 8,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(session.inspect()["attachment_error_m"].as_f64().unwrap() < 1e-7);
        session.grow(2.).unwrap();
        session.step(2).unwrap();
        assert!(session.inspect()["attachment_error_m"].as_f64().unwrap() < 1e-4);
    }
    assert_closed_exemplar(&c, id);
}
fn assert_closed_exemplar(c: &bake::Compiled, id: &str) {
    // Continuous woody skin: every edge has two opposite incident faces.
    for (part, asset) in c.document.assets.iter().enumerate() {
        // Empty material parts use the existing microscopic ownership placeholder;
        // it is not botanical skin and has no material bindings or physics nodes.
        if c.vertices[part].is_empty() {
            assert_eq!(asset.geometry.positions.len(), 3);
            assert_eq!(asset.geometry.triangles.len(), 1);
            continue;
        }
        let mut edges: BTreeMap<(u32, u32), (u32, i32)> = BTreeMap::new();
        for t in &asset.geometry.triangles {
            for k in 0..3 {
                let a = t[k];
                let b = t[(k + 1) % 3];
                let entry = edges.entry((a.min(b), a.max(b))).or_default();
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            edges.values().all(|&(n, w)| n == 2 && w == 0),
            "{id}/{} mesh not closed/manifold: {:?}",
            asset.id,
            edges
                .iter()
                .filter(|(_, (n, w))| *n != 2 || *w != 0)
                .take(6)
                .map(|(&(a, b), v)| (
                    &c.vertices[part][a as usize].key,
                    &c.vertices[part][b as usize].key,
                    v
                ))
                .collect::<Vec<_>>()
        );
    }
}

// A grid ring almost coincident with a collar collapses in f32. Silently
// dropping its triangles used to leave the woody skin open after compilation.
#[test]
fn collar_near_longitudinal_sample_keeps_closed_skin() {
    let mut r = fixture();
    r.settings.axis_samples = 18;
    let source = Arc::make_mut(&mut r.source);
    let mut stem = source.organs[0].clone();
    stem.path = vec![[0.; 3], [0., 3., 0.]];
    stem.shape = b::Shape::Axis {
        radius: 3. * (1. / 6. - 1e-12) / 1.8,
        tip_ratio: 1.,
        ellipticity: 1.,
    };
    let mut child = stem.clone();
    child.id = "branch".into();
    child.parent = Some("stem".into());
    child.insertion = 0.5;
    child.path = vec![[0., 1.5, 0.], [1.5, 1.5, 0.]];
    child.normal = [0., 0., 1.];
    child.shape = b::Shape::Axis {
        radius: 0.05,
        tip_ratio: 0.5,
        ellipticity: 1.,
    };
    source.organs = vec![stem, child];
    let c = bake::compile(&r).unwrap();
    assert_closed_exemplar(&c, "near-collar");
    let stem_end = format!("stem@{:016x}/999", 1_f64.to_bits());
    assert!(c.vertices[0].iter().any(|v| v.key == stem_end));
    // A sculpt on the old redundant grid ring must be refused rather than lost.
    r.manual.insert(
        format!("stem@{:016x}/0", (1_f64 / 3.).to_bits()),
        [0.001, 0., 0.],
    );
    assert!(
        bake::compile(&r)
            .err()
            .unwrap()
            .message
            .contains("lose adjusted material point")
    );
}

#[test]
fn physical_growth_pruning_and_failure_preserve_identity_and_momentum() {
    let r = fixture();
    let c = bake::compile(&r).unwrap();
    let mut s = dynamics::Session::new(
        r,
        c,
        dynamics::Settings {
            age: 1.2,
            gravity: [0.; 3],
            wind: [3., 0., 0.],
            ground: None,
            ..Default::default()
        },
    )
    .unwrap();
    s.step(24).unwrap();
    let before = s.inspect();
    let momentum = before["momentum"].as_array().unwrap().clone();
    let nodes = before["nodes"].clone();
    s.grow(2.).unwrap();
    let grown = s.inspect();
    assert_eq!(grown["nodes"], nodes);
    assert!(grown["mass_kg"].as_f64().unwrap() > before["mass_kg"].as_f64().unwrap());
    for (a, b) in momentum.iter().zip(grown["momentum"].as_array().unwrap()) {
        assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10);
    }
    let previous = s.inspect();
    assert!(s.grow(0.).is_err());
    assert!(s.step(0).is_err());
    assert_eq!(s.inspect(), previous);
    s.prune("leaf").unwrap();
    let after = s.inspect();
    assert_eq!(after["nodes"], nodes);
    assert_eq!(after["removed_organs"], 1);
    assert!(after["removed_mass_kg"].as_f64().unwrap() > 0.);
    s.document().validate().unwrap();
}
#[test]
fn nonlinear_bending_ground_contact_remains_finite_without_angle_clamp() {
    let mut r = fixture();
    let src = Arc::make_mut(&mut r.source);
    src.organs.truncate(1);
    let o = &mut src.organs[0];
    o.path = vec![[0., 0.4, 0.], [1., 0.4, 0.]];
    o.normal = [0., 1., 0.];
    o.young = 1e5;
    o.shear = 4e4;
    let c = bake::compile(&r).unwrap();
    let mut s = dynamics::Session::new(
        r,
        c,
        dynamics::Settings {
            ground: Some(0.),
            iterations: 32,
            ..Default::default()
        },
    )
    .unwrap();
    s.step(240).unwrap();
    let report = s.inspect();
    println!("large bending: {report}");
    assert!(report["max_frame_rotation_rad"].as_f64().unwrap() > 0.65);
    assert!(report["contacts"].as_u64().unwrap() > 0);
    assert!(report["min_node_y"].as_f64().unwrap() >= 0.);
    s.document().validate().unwrap();
}
#[test]
fn crossing_branches_resolve_proxy_contact_with_accounted_external_impulses() {
    let mut r = fixture();
    let src = Arc::make_mut(&mut r.source);
    src.organs.truncate(1);
    let o = &mut src.organs[0];
    o.path = vec![[-0.5, 0.5, 0.], [0.5, 0.5, 0.]];
    o.normal = [0., 1., 0.];
    o.young = 1e5;
    o.shear = 4e4;
    let mut other = o.clone();
    other.id = "other".into();
    other.path = vec![[0., 0.5, -0.5], [0., 0.5, 0.5]];
    src.organs.push(other);
    let c = bake::compile(&r).unwrap();
    let mut s = dynamics::Session::new(
        r,
        c,
        dynamics::Settings {
            gravity: [0.; 3],
            ground: None,
            self_contact: true,
            iterations: 64,
            ..Default::default()
        },
    )
    .unwrap();
    s.step(120).unwrap();
    let report = s.inspect();
    println!("crossing contact: {report}");
    assert!(report["contacts"].as_u64().unwrap() > 0);
    assert!(report["max_proxy_overlap_m"].as_f64().unwrap() < 1e-4);
    for k in 0..3 {
        let p = report["momentum"][k].as_f64().unwrap();
        let external = report["force_impulse"][k].as_f64().unwrap()
            + report["solver_damping_impulse"][k].as_f64().unwrap();
        assert!((p - external).abs() < 1e-10);
    }
    s.document().validate().unwrap();
}
#[test]
fn physical_preview_tracks_camera_but_invalidates_on_source_edit_and_load() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(fixture()),
            policy: Default::default(),
        },
    )
    .unwrap();
    op(
        &mut s,
        b::Operation::Physics {
            prefix: "plant".into(),
            action: dynamics::Action::Start {
                settings: Default::default(),
            },
        },
    )
    .unwrap();
    let initial = s.botany.physics.as_ref().unwrap().document();
    let base = &s.botany.physics.as_ref().unwrap().compiled.document;
    for (a, b) in initial.assets.iter().zip(&base.assets) {
        for (a, b) in a.geometry.positions.iter().zip(&b.geometry.positions) {
            for k in 0..3 {
                assert!((a[k] - b[k]).abs() < 1e-6);
            }
        }
    }
    let mut camera = s.engine.scene.camera.clone();
    camera.position[0] += 1.;
    s.execute(Command::SetCamera { camera }.into()).unwrap();
    assert!(s.botany.physics.is_some());
    assert_eq!(s.botany.revision, Some(s.engine.revision));
    op(
        &mut s,
        b::Operation::Physics {
            prefix: "plant".into(),
            action: dynamics::Action::Play { playing: true },
        },
    )
    .unwrap();
    let before = s.botany.physics.as_ref().unwrap().inspect();
    assert!(s.advance(0).is_err());
    assert_eq!(s.botany.physics.as_ref().unwrap().inspect(), before);
    s.advance(1).unwrap();
    assert_eq!(s.botany.physics.as_ref().unwrap().tick, 2);
    let mut asset = s.engine.scene.meshes.asset("plant_wood").unwrap().clone();
    Arc::make_mut(&mut asset.geometry).positions[3][0] += 0.002;
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        }
        .into(),
    )
    .unwrap();
    assert!(s.botany.physics.is_none());
    assert!(s.botany.overlay.is_none());
}
#[test]
fn foliage_only_portable_save_restores_source_without_inventing_an_occupant() {
    let root = tempfile::tempdir().unwrap();
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.entities.clear();
    let mut s = Service::new(root.path(), scene).unwrap();
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(fixture()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let before = s.engine.scene.clone();
    s.execute(
        Command::SavePortableSave {
            path: "plant.ignis-portable".into(),
        }
        .into(),
    )
    .unwrap();
    s.execute(
        Command::LoadPortableSave {
            path: "plant.ignis-portable".into(),
            mode: None,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.scene, before);
    assert_eq!(s.engine.simulation.actors.actors.len(), 0);
}
#[test]
fn geometry_scale_and_blade_thickness_feed_the_same_physical_mass() {
    let r = fixture();
    let mass = |r: b::Recipe| {
        let c = bake::compile(&r).unwrap();
        dynamics::Session::new(r, c, Default::default())
            .unwrap()
            .mass()
    };
    let baseline = mass(r.clone());
    let mut stem = r.clone();
    stem.pruned.insert("leaf".into());
    let stem_mass = mass(stem);
    let mut thick = r.clone();
    thick.edits.insert(
        "leaf".into(),
        b::OrganEdit {
            radius: 2.,
            ..Default::default()
        },
    );
    assert!((mass(thick) - stem_mass - 2. * (baseline - stem_mass)).abs() < 1e-10);
    let mut scaled = r;
    scaled.settings.scale = 2.;
    assert!((mass(scaled) - 8. * baseline).abs() < 1e-10);
}
#[test]
fn contact_budget_failure_is_explicit_and_preserves_the_physical_state() {
    let mut r = fixture();
    let source = Arc::make_mut(&mut r.source);
    let axis = source.organs[0].clone();
    source.organs = (0..1000)
        .map(|i| {
            let mut o = axis.clone();
            o.id = format!("root{i}");
            o
        })
        .collect();
    let c = bake::compile(&r).unwrap();
    let mut s = dynamics::Session::new(
        r,
        c,
        dynamics::Settings {
            self_contact: true,
            gravity: [0.; 3],
            ground: None,
            ..Default::default()
        },
    )
    .unwrap();
    let before = s.inspect();
    assert_eq!(before["contact_probe_complete"], false);
    assert!(
        before["contact_probe_error"]
            .as_str()
            .unwrap()
            .contains("budget")
    );
    assert!(s.step(1).is_err());
    assert_eq!(s.inspect(), before);
}

// Protects the source-format and bake/physical-rest boundary: descendant identity,
// material edits and transactional publication must survive a change of rest curve.
#[test]
fn rest_curvature_roundtrip_descendants_and_physical_rest_share_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut r = fixture();
    Arc::make_mut(&mut r.source).organs[0].path = vec![[0.; 3], [0.1, 0.5, 0.2], [0.4, 1., 0.05]];
    let old = bake::compile(&r).unwrap();
    let mut dormant = r.clone();
    dormant
        .edits
        .entry("stem".into())
        .or_default()
        .deformation
        .frequency = 3.;
    assert_eq!(bake::compile(&dormant).unwrap().document, old.document);
    let legacy: b::Recipe = serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();
    assert_eq!(bake::compile(&legacy).unwrap().document, old.document);
    let mut curved = r.clone();
    curved.version = b::VERSION;
    curved.edits.entry("stem".into()).or_default().deformation = b::deformation::Deformation {
        base: [1.2, 0.2],
        tip: [-0.8, 0.6],
        twist: 0.8,
        sag: 0.4,
        irregularity: 0.15,
        ..Default::default()
    };
    let c = bake::compile(&curved).unwrap();
    assert!(c.poses[0].at(1.).p.distance(old.poses[0].at(1.).p) > 0.1);
    assert!(c.poses[1].at(0.).p.distance(old.poses[1].at(0.).p) > 0.05);
    assert!((c.poses[0].length - old.poses[0].length).abs() < 1e-10);
    for part in 0..2 {
        assert_eq!(
            c.vertices[part].iter().map(|v| &v.key).collect::<Vec<_>>(),
            old.vertices[part]
                .iter()
                .map(|v| &v.key)
                .collect::<Vec<_>>()
        );
    }
    let settings = dynamics::Settings {
        gravity: [0.; 3],
        ground: None,
        ..Default::default()
    };
    let mut physical = dynamics::Session::new(curved.clone(), c.clone(), settings.clone()).unwrap();
    let old_physical = dynamics::Session::new(r, old, settings).unwrap();
    assert_eq!(
        physical.inspect()["node_identity"],
        old_physical.inspect()["node_identity"]
    );
    assert!((physical.mass() - old_physical.mass()).abs() < 1e-10);
    let rendered = physical.document();
    for part in 0..2 {
        for (a, b) in rendered.assets[part]
            .geometry
            .positions
            .iter()
            .zip(&c.document.assets[part].geometry.positions)
        {
            assert!(glam::Vec3::from_array(*a).distance(glam::Vec3::from_array(*b)) < 1e-6);
        }
    }
    physical.step(2).unwrap();
    assert!(physical.inspect()["attachment_error_m"].as_f64().unwrap() < 1e-4);
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    op(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(curved.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    op(
        &mut service,
        b::Operation::Save {
            prefix: "plant".into(),
            path: "curved.ignis-botany".into(),
        },
    )
    .unwrap();
    let saved: b::Recipe =
        serde_json::from_slice(&std::fs::read(root.path().join("curved.ignis-botany")).unwrap())
            .unwrap();
    assert_eq!(saved, curved);
    let before = service.engine.scene.clone();
    let revision = service.engine.revision;
    let mut future = curved.clone();
    future.version = b::VERSION + 1;
    assert!(
        op(
            &mut service,
            b::Operation::Publish {
                recipe: Box::new(future),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, before);
    assert_eq!(service.engine.revision, revision);
    curved.edits.get_mut("stem").unwrap().deformation.base[0] = 11.;
    assert!(
        op(
            &mut service,
            b::Operation::Publish {
                recipe: Box::new(curved),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, before);
    assert_eq!(service.engine.revision, revision);
}

// Protects chronology format, semantic edits and the material state boundary.
#[test]
fn chronological_age_preserves_existing_identity_and_rejects_loss_atomically() {
    let root = tempfile::tempdir().unwrap();
    let mut r = fixture();
    r.version = b::VERSION;
    let source = Arc::make_mut(&mut r.source);
    source.version = b::VERSION;
    source.development = Some(Default::default());
    source.organs[1].life = Some(b::development::OrganLife {
        born_years: 4.,
        ..Default::default()
    });
    r.settings.age_years = Some(8.);
    let young = bake::compile(&r).unwrap();
    let mut adult = r.clone();
    adult.settings.age_years = Some(60.);
    let c = bake::compile(&adult).unwrap();
    assert!(c.poses[0].length > young.poses[0].length * 2.);
    assert!(c.poses[0].radius > young.poses[0].radius * 3.);
    assert!(c.poses[1].length < young.poses[1].length * 1.2);
    for part in 0..2 {
        assert_eq!(
            young.vertices[part]
                .iter()
                .map(|v| &v.key)
                .collect::<Vec<_>>(),
            c.vertices[part].iter().map(|v| &v.key).collect::<Vec<_>>()
        );
    }
    let settings = dynamics::Settings {
        gravity: [0.; 3],
        ground: None,
        ..Default::default()
    };
    let y = dynamics::Session::new(r.clone(), young, settings.clone()).unwrap();
    let a = dynamics::Session::new(adult.clone(), c.clone(), settings).unwrap();
    assert_eq!(y.inspect()["node_identity"], a.inspect()["node_identity"]);
    assert!(a.mass() > y.mass() * 8.);
    for part in 0..2 {
        for (p, q) in a.document().assets[part]
            .geometry
            .positions
            .iter()
            .zip(&c.document.assets[part].geometry.positions)
        {
            assert!(glam::Vec3::from_array(*p).distance(glam::Vec3::from_array(*q)) < 1e-6);
        }
    }
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    op(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(r),
            policy: Default::default(),
        },
    )
    .unwrap();
    let before_preview = service.engine.scene.clone();
    op(
        &mut service,
        b::Operation::SetAge {
            prefix: "plant".into(),
            age_years: 20.,
            vigor: None,
            preview: true,
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(service.engine.scene, before_preview);
    op(&mut service, b::Operation::CancelPreview).unwrap();
    let mut mesh = service
        .engine
        .scene
        .meshes
        .asset("plant_leaves")
        .unwrap()
        .clone();
    Arc::make_mut(&mut mesh.geometry).positions[12][2] += 0.01;
    service
        .execute(
            Command::EditMesh {
                edit: ignis::meshes::Edit::PutAsset { asset: mesh },
            }
            .into(),
        )
        .unwrap();
    op(
        &mut service,
        b::Operation::SetAge {
            prefix: "plant".into(),
            age_years: 60.,
            vigor: None,
            preview: false,
            policy: Default::default(),
        },
    )
    .unwrap();
    assert!(!service.engine.scene.botany[0].manual.is_empty());
    op(
        &mut service,
        b::Operation::Save {
            prefix: "plant".into(),
            path: "aged.ignis-botany".into(),
        },
    )
    .unwrap();
    let decoded: b::Recipe =
        serde_json::from_slice(&std::fs::read(root.path().join("aged.ignis-botany")).unwrap())
            .unwrap();
    assert_eq!(decoded, service.engine.scene.botany[0]);
    assert_eq!(
        bake::compile(&decoded).unwrap().document,
        bake::compile(&decoded).unwrap().document
    );
    let scene = service.engine.scene.clone();
    let revision = service.engine.revision;
    let mut younger = decoded.clone();
    younger.settings.age_years = Some(2.);
    assert!(
        op(
            &mut service,
            b::Operation::Publish {
                recipe: Box::new(younger),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, scene);
    assert_eq!(service.engine.revision, revision);
    let mut invalid = decoded;
    invalid.settings.age_years = Some(-1.);
    assert!(
        op(
            &mut service,
            b::Operation::Publish {
                recipe: Box::new(invalid),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, scene);
}

// Protects a new continuous-skin topology, its partial-prune boundary and format.
#[test]
fn terminal_forks_remain_closed_after_partial_pruning_and_roundtrip() {
    let mut r = fixture();
    r.version = b::VERSION;
    let source = Arc::make_mut(&mut r.source);
    source.version = b::VERSION;
    source.organs.truncate(1);
    source.organs[0].shape = b::Shape::Axis {
        radius: 0.06,
        tip_ratio: 0.76,
        ellipticity: 1.,
    };
    for (id, side) in [("left", -1.), ("right", 1.)] {
        let mut axis = source.organs[0].clone();
        axis.id = id.into();
        axis.parent = Some("stem".into());
        axis.insertion = 1.;
        axis.connection = b::Connection::Fork;
        axis.path = vec![[0., 1., 0.], [side * 0.7, 1.9, 0.1]];
        axis.normal = [0., 0., 1.];
        axis.shape = b::Shape::Axis {
            radius: 0.035,
            tip_ratio: 0.4,
            ellipticity: 1.,
        };
        source.organs.push(axis);
    }
    let bytes = serde_json::to_vec(&r).unwrap();
    let decoded: b::Recipe = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(r, decoded);
    let full = bake::compile(&r).unwrap();
    assert_eq!(full.joints, 1);
    let mut sculpt = r.clone();
    let interior = full.vertices[0]
        .iter()
        .find(|v| v.key.contains(":fork:"))
        .unwrap();
    sculpt.manual.insert(interior.key.clone(), [0., 0.001, 0.]);
    bake::compile(&sculpt).unwrap();
    sculpt.edits.entry("right".into()).or_default().length = 1.3;
    assert!(
        bake::compile(&sculpt).is_err(),
        "A changed peer collar must invalidate interior sculpt addresses"
    );
    let mut physical = dynamics::Session::new(
        r.clone(),
        full.clone(),
        dynamics::Settings {
            gravity: [0.; 3],
            ground: None,
            ..Default::default()
        },
    )
    .unwrap();
    physical.step(2).unwrap();
    assert!(physical.inspect()["attachment_error_m"].as_f64().unwrap() < 1e-4);
    for removed in [vec![], vec!["left"], vec!["right"], vec!["left", "right"]] {
        let mut cut = r.clone();
        cut.pruned.extend(removed.into_iter().map(str::to_owned));
        let c = bake::compile(&cut).unwrap();
        c.document.validate().unwrap();
        let mesh = &c.document.assets[0].geometry;
        let mut edges: BTreeMap<(u32, u32), (usize, i32)> = BTreeMap::new();
        let mut volume = 0.;
        for t in &mesh.triangles {
            let p = glam::DVec3::from_array(mesh.positions[t[0] as usize].map(f64::from));
            let q = glam::DVec3::from_array(mesh.positions[t[1] as usize].map(f64::from));
            let z = glam::DVec3::from_array(mesh.positions[t[2] as usize].map(f64::from));
            volume += p.dot(q.cross(z)) / 6.;
            for k in 0..3 {
                let a = t[k];
                let b = t[(k + 1) % 3];
                let e = edges.entry((a.min(b), a.max(b))).or_default();
                e.0 += 1;
                e.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(volume > 0.);
        assert!(edges.values().all(|&(n, w)| n == 2 && w == 0));
    }
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
    let rev = service.engine.revision;
    Arc::make_mut(&mut r.source).organs.pop();
    assert!(
        op(
            &mut service,
            b::Operation::Publish {
                recipe: Box::new(r),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, before);
    assert_eq!(service.engine.revision, rev);
}

// A tessellation change may remove sculpted material coordinates. Reject it
// atomically; legacy default sampling must keep its existing geometry.
#[test]
fn blade_resolution_preserves_legacy_geometry_and_protects_sculpt_addresses() {
    let legacy = fixture();
    let old = bake::compile(&legacy).unwrap();
    let mut recipe = legacy;
    recipe.version = b::VERSION;
    assert_eq!(old.document, bake::compile(&recipe).unwrap().document);
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    op(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(recipe.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let index = old.vertices[1]
        .iter()
        .position(|v| (v.u - 1. / 12.).abs() < 1e-10)
        .unwrap();
    let mut asset = service
        .engine
        .scene
        .meshes
        .asset("plant_leaves")
        .unwrap()
        .clone();
    Arc::make_mut(&mut asset.geometry).positions[index][2] += 0.002;
    service
        .execute(
            Command::EditMesh {
                edit: ignis::meshes::Edit::PutAsset { asset },
            }
            .into(),
        )
        .unwrap();
    let before = service.engine.scene.clone();
    let revision = service.engine.revision;
    recipe.settings.blade_samples = 4;
    assert!(
        op(
            &mut service,
            b::Operation::Publish {
                recipe: Box::new(recipe.clone()),
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, before);
    assert_eq!(service.engine.revision, revision);
    let coarse = bake::compile(&recipe).unwrap();
    assert!(coarse.vertices[1].len() < old.vertices[1].len());
    let decoded: b::Recipe = serde_json::from_slice(&serde_json::to_vec(&recipe).unwrap()).unwrap();
    assert_eq!(recipe, decoded);
    recipe.version = 3;
    assert!(recipe.validate().is_err());
}

// New basal topology must be closed through generation, partial pruning and
// removal; old source/geometry must not be rewritten to add a root crown.
#[test]
fn root_systems_preserve_legacy_source_and_closed_basal_skin() {
    let old = fixture();
    let baseline = bake::compile(&old).unwrap();
    for kind in [
        b::roots::Kind::Radial,
        b::roots::Kind::Taproot,
        b::roots::Kind::Fibrous,
    ] {
        let mut settings = b::roots::Settings {
            kind,
            count: if kind == b::roots::Kind::Taproot {
                1
            } else {
                5
            },
            orders: 1,
            ..Default::default()
        };
        if kind == b::roots::Kind::Taproot {
            settings.depth = 8.;
            settings.exposure = 0.;
        }
        if kind == b::roots::Kind::Fibrous {
            settings.radius_ratio = 0.12;
            settings.depth = 5.;
            settings.exposure = 0.;
        }
        let r = b::roots::configure(&old, "stem", Some(settings)).unwrap();
        assert_eq!(
            r.source.organs[..old.source.organs.len()],
            old.source.organs
        );
        let c = bake::compile(&r).unwrap_or_else(|e| panic!("{kind:?}: {e}"));
        assert_closed_exemplar(&c, "roots");
        assert_eq!(c.document, bake::compile(&r).unwrap().document);
        if kind == b::roots::Kind::Radial {
            let mut sloped = r.source.root_systems[0].settings.clone();
            sloped.soil_slope = [0.12, -0.08];
            let sloped = b::roots::configure(&old, "stem", Some(sloped)).unwrap();
            assert_closed_exemplar(&bake::compile(&sloped).unwrap(), "sloped-roots");
            assert_ne!(sloped.source.organs, r.source.organs);
            let mut unowned = r.clone();
            let source = Arc::make_mut(&mut unowned.source);
            let mut fake = source.organs[0].clone();
            fake.id = "stem/root-unowned".into();
            source.organs.push(fake);
            assert!(unowned.validate().is_err());
        }
        let roundtrip: b::Recipe =
            serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();
        assert_eq!(r, roundtrip);
        let mut partial = r.clone();
        partial.pruned.insert("stem/root0".into());
        assert_closed_exemplar(&bake::compile(&partial).unwrap(), "root-prune");
        let removed = b::roots::configure(&r, "stem", None).unwrap();
        assert_eq!(removed.source.organs, old.source.organs);
        assert_eq!(bake::compile(&removed).unwrap().document, baseline.document);
        let mut legacy = r;
        legacy.version = 4;
        assert!(bake::compile(&legacy).is_err());
    }
    let mut old = fixture();
    old.version = 3;
    let source = Arc::make_mut(&mut old.source);
    source.version = 3;
    source.development = Some(Default::default());
    let mut aged = b::roots::configure(
        &old,
        "stem",
        Some(b::roots::Settings {
            count: 5,
            orders: 1,
            ..Default::default()
        }),
    )
    .unwrap();
    for years in [8., 20., 80.] {
        aged.settings.age_years = Some(years);
        let c = bake::compile(&aged).unwrap_or_else(|e| {
            panic!(
                "root age {years}: {e}; {:?}",
                bake::poses(&aged)
                    .unwrap()
                    .iter()
                    .take(4)
                    .map(|p| (p.length, p.radius, p.parent))
                    .collect::<Vec<_>>()
            )
        });
        assert_closed_exemplar(&c, "root-age");
    }
}

// Root parameters use the same transaction as other botany operations; they
// cannot discard a sculpt or leave a partial scene when crown fitting fails.
#[test]
fn root_operations_are_atomic_and_root_anchors_preserve_mass() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let old = fixture();
    op(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(old),
            policy: Default::default(),
        },
    )
    .unwrap();
    let before = service.engine.scene.clone();
    let rev = service.engine.revision;
    let settings = b::roots::Settings {
        count: 5,
        orders: 1,
        ..Default::default()
    };
    op(
        &mut service,
        b::Operation::SetRoots {
            prefix: "plant".into(),
            stem: "stem".into(),
            settings: Some(settings.clone()),
            preview: true,
            policy: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(before, service.engine.scene);
    assert_eq!(rev, service.engine.revision);
    op(
        &mut service,
        b::Operation::SetRoots {
            prefix: "plant".into(),
            stem: "stem".into(),
            settings: Some(settings),
            preview: false,
            policy: Default::default(),
        },
    )
    .unwrap();
    let r = service.engine.scene.botany[0].clone();
    let c = bake::compile(&r).unwrap();
    let mut sim = dynamics::Session::new(
        r.clone(),
        c,
        dynamics::Settings {
            gravity: [0.; 3],
            wind: [0.5, 0., 0.],
            ..Default::default()
        },
    )
    .unwrap();
    let initial = sim.inspect();
    assert!(initial["root_anchor_nodes"].as_u64().unwrap() > 0);
    sim.step(4).unwrap();
    assert!(sim.inspect()["attachment_error_m"].as_f64().unwrap() < 1e-4);
    assert!(sim.inspect()["min_node_y"].as_f64().unwrap() < 0.);
    sim.prune("stem/root0").unwrap();
    let state = sim.inspect();
    assert!(
        (state["mass_kg"].as_f64().unwrap() + state["removed_mass_kg"].as_f64().unwrap()
            - state["initial_mass_kg"].as_f64().unwrap()
            - state["added_mass_kg"].as_f64().unwrap())
        .abs()
            < 1e-8
    );
    let mut sculpt = r.clone();
    let compiled = bake::compile(&sculpt).unwrap();
    let key = compiled.vertices[0]
        .iter()
        .find(|v| v.key.contains(":root-crown:"))
        .unwrap()
        .key
        .clone();
    sculpt.manual.insert(key, [0.001, 0., 0.]);
    op(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(sculpt),
            policy: Default::default(),
        },
    )
    .unwrap();
    let before = service.engine.scene.clone();
    let rev = service.engine.revision;
    let mut resampled = service.engine.scene.botany[0].clone();
    resampled.settings.sides = 10;
    assert!(
        bake::compile(&resampled)
            .err()
            .unwrap()
            .message
            .contains("lose adjusted material point")
    );
    let changed = b::roots::Settings {
        count: 6,
        orders: 1,
        ..Default::default()
    };
    assert!(
        op(
            &mut service,
            b::Operation::SetRoots {
                prefix: "plant".into(),
                stem: "stem".into(),
                settings: Some(changed),
                preview: false,
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(before, service.engine.scene);
    assert_eq!(rev, service.engine.revision);
}

#[test]
fn buttress_walls_have_closed_skin_distinct_crest_and_preserved_rest_state() {
    let old = fixture();
    let settings = b::roots::Settings {
        count: 5,
        orders: 1,
        spread: 12.,
        neck_height: 3.,
        depth: 1.5,
        buttress: Some(b::roots::Buttress {
            height: 2.3,
            meander: 0.12,
            ..Default::default()
        }),
        ..Default::default()
    };
    let r = b::roots::configure(&old, "stem", Some(settings.clone())).unwrap();
    assert_eq!(r.version, 7);
    assert_eq!(
        r.source.organs[..old.source.organs.len()],
        old.source.organs
    );
    let c = bake::compile(&r).unwrap();
    assert_closed_exemplar(&c, "buttress");
    assert_eq!(c.document, bake::compile(&r).unwrap().document);
    let root = c
        .poses
        .iter()
        .find(|p| r.source.organs[p.source].id == "stem/root0")
        .unwrap();
    let ring: Vec<_> = (0..24)
        .map(|i| {
            b::root_sections::point(root, &r.source, 0.4, std::f64::consts::TAU * i as f64 / 24.)
        })
        .collect();
    let height = ring.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max)
        - ring.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let side = bake::normal(root.at(0.4).t, glam::DVec3::Y).cross(root.at(0.4).t);
    let width = ring
        .iter()
        .map(|p| p.dot(side))
        .fold(f64::NEG_INFINITY, f64::max)
        - ring
            .iter()
            .map(|p| p.dot(side))
            .fold(f64::INFINITY, f64::min);
    assert!(
        height > 3. * width,
        "wall must be taller than its thickness"
    );
    let mut partial = r.clone();
    partial.pruned.insert("stem/root0/f0".into());
    assert_closed_exemplar(&bake::compile(&partial).unwrap(), "buttress-prune");
    let decoded: b::Recipe = serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();
    assert_eq!(decoded, r);
    let mut v5 = r.clone();
    v5.version = 5;
    assert!(v5.validate().is_err());
    let mut sim = dynamics::Session::new(
        r.clone(),
        c.clone(),
        dynamics::Settings {
            gravity: [0.; 3],
            wind: [0.2, 0., 0.],
            ..Default::default()
        },
    )
    .unwrap();
    let rest = sim.document();
    for (index, (a, b)) in rest.assets[0]
        .geometry
        .positions
        .iter()
        .zip(&c.document.assets[0].geometry.positions)
        .enumerate()
    {
        assert!(
            glam::Vec3::from_array(*a).distance(glam::Vec3::from_array(*b)) < 1e-5,
            "rest drift at {}: {a:?} vs {b:?}",
            c.vertices[0][index].key
        );
    }
    let initial = sim.inspect();
    sim.step(4).unwrap();
    sim.prune("stem/root0").unwrap();
    let state = sim.inspect();
    assert_eq!(initial["node_identity"], state["node_identity"]);
    assert!(state["attachment_error_m"].as_f64().unwrap() < 1e-4);
    assert!(
        (state["mass_kg"].as_f64().unwrap() + state["removed_mass_kg"].as_f64().unwrap()
            - initial["mass_kg"].as_f64().unwrap())
        .abs()
            < 1e-8
    );
    let mut sculpt = r.clone();
    let key = c.vertices[0]
        .iter()
        .find(|v| v.key.contains("wall-v1"))
        .unwrap()
        .key
        .clone();
    sculpt.manual.insert(key, [0.001, 0., 0.]);
    let mut tubular = settings;
    tubular.buttress = None;
    let changed = b::roots::configure(&sculpt, "stem", Some(tubular)).unwrap();
    assert!(bake::compile(&changed).is_err());
    let mut chronological = old.clone();
    chronological.version = 3;
    let source = Arc::make_mut(&mut chronological.source);
    source.version = 3;
    source.development = Some(Default::default());
    let mut aged = b::roots::configure(
        &chronological,
        "stem",
        Some(r.source.root_systems[0].settings.clone()),
    )
    .unwrap();
    for years in [8., 20., 80.] {
        aged.settings.age_years = Some(years);
        assert_closed_exemplar(
            &bake::compile(&aged).unwrap_or_else(|e| panic!("buttress-age {years}: {e}")),
            "buttress-age",
        );
    }
    let mut forked = old.clone();
    forked.version = 4;
    let source = Arc::make_mut(&mut forked.source);
    source.version = 4;
    for (index, side) in [-1., 1.].into_iter().enumerate() {
        let mut axis = source.organs[0].clone();
        axis.id = format!("branch{index}");
        axis.parent = Some("stem".into());
        axis.insertion = 1.;
        axis.connection = b::Connection::Fork;
        axis.path = vec![[0., 1., 0.], [side * 0.2, 1.2, 0.]];
        axis.normal = [0., 0., 1.];
        axis.shape = b::Shape::Axis {
            radius: 0.006,
            tip_ratio: 0.2,
            ellipticity: 1.,
        };
        source.organs.push(axis);
    }
    let mut forked = b::roots::configure(
        &forked,
        "stem",
        Some(r.source.root_systems[0].settings.clone()),
    )
    .unwrap();
    assert_closed_exemplar(&bake::compile(&forked).unwrap(), "fluted-stem-fork");
    forked.pruned.insert("branch0".into());
    assert_closed_exemplar(&bake::compile(&forked).unwrap(), "fluted-pruned-collars");
}

#[test]
fn valley_crown_versions_preserve_foliage_and_refuse_legacy_sculpt_loss() {
    let settings = b::roots::Settings {
        count: 5,
        orders: 1,
        spread: 12.,
        neck_height: 3.,
        depth: 1.5,
        buttress: Some(b::roots::Buttress {
            height: 2.3,
            meander: 0.12,
            ..Default::default()
        }),
        ..Default::default()
    };
    let current = b::roots::configure(&fixture(), "stem", Some(settings.clone())).unwrap();
    let mut legacy = current.clone();
    legacy.version = 6;
    Arc::make_mut(&mut legacy.source).version = 6;
    let old = bake::compile(&legacy).unwrap();
    let new = bake::compile(&current).unwrap();
    assert_closed_exemplar(&old, "legacy-crown");
    assert_closed_exemplar(&new, "valley-crown");
    assert_eq!(current.source.organs, legacy.source.organs);
    assert_eq!(
        new.document.assets[1].geometry,
        old.document.assets[1].geometry
    );
    assert_ne!(
        new.document.assets[0].geometry,
        old.document.assets[0].geometry
    );
    let mut invalid = current.clone();
    invalid.version = 6;
    assert!(invalid.validate().is_err());
    let key = old.vertices[0]
        .iter()
        .find(|v| v.key.contains("root-crown:"))
        .unwrap()
        .key
        .clone();
    legacy.manual.insert(key, [0.001, 0., 0.]);
    assert!(bake::compile(&legacy).is_ok());
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    op(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(legacy.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let before = service.engine.scene.clone();
    let revision = service.engine.revision;
    let result = op(
        &mut service,
        b::Operation::SetRoots {
            prefix: "plant".into(),
            stem: "stem".into(),
            settings: Some(settings),
            preview: false,
            policy: Default::default(),
        },
    );
    assert!(
        result
            .err()
            .unwrap()
            .message
            .contains("lose adjusted material point")
    );
    assert_eq!(service.engine.scene, before);
    assert_eq!(service.engine.revision, revision);
    let decoded: b::Recipe = serde_json::from_slice(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(
        bake::compile(&legacy).unwrap().document,
        bake::compile(&decoded).unwrap().document
    );
}

// Protect against losing generator textures when regenerating an editable plant,
// and against a preview that disagrees with publication or bypasses strict policies.
#[test]
fn generated_materials_survive_botanical_preview_publish_and_physical_overlay() {
    use ignis::materials::library::{Execution, Operation as MaterialOp, Preset};
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let r = fixture();
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(r.clone()),
            policy: b::ManualPolicy::Preserve,
        },
    )
    .unwrap();
    for (asset, execution) in [
        ("plant_wood", Execution::Fixed { resolution: 16 }),
        ("plant_leaves", Execution::Runtime),
    ] {
        s.execute(
            Command::Material {
                operation: Box::new(MaterialOp::Preset {
                    asset: asset.into(),
                    preset: Preset::Wood,
                    seed: Some(21),
                    unique_instances: false,
                    execution,
                }),
            }
            .into(),
        )
        .unwrap();
    }
    let materials = r
        .parts()
        .map(|id| s.engine.scene.meshes.asset(&id).unwrap().material.clone());
    let before = s.engine.scene.clone();
    let mut next = r.clone();
    next.settings.leaf_length = 1.2;
    op(
        &mut s,
        b::Operation::Preview {
            recipe: Box::new(next.clone()),
            policy: b::ManualPolicy::Preserve,
        },
    )
    .unwrap();
    assert_eq!(s.engine.scene, before);
    for (id, expected) in r.parts().iter().zip(&materials) {
        assert_eq!(
            &s.botany
                .overlay
                .as_ref()
                .unwrap()
                .asset(id)
                .unwrap()
                .material,
            expected
        );
    }
    op(&mut s, b::Operation::CancelPreview).unwrap();
    assert_eq!(s.engine.scene, before);
    assert!(
        op(
            &mut s,
            b::Operation::Publish {
                recipe: Box::new(next.clone()),
                policy: b::ManualPolicy::Reject
            }
        )
        .is_err()
    );
    assert_eq!(s.engine.scene, before);
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(next.clone()),
            policy: b::ManualPolicy::Preserve,
        },
    )
    .unwrap();
    for (id, expected) in r.parts().iter().zip(&materials) {
        assert_eq!(&s.engine.scene.meshes.asset(id).unwrap().material, expected);
    }
    op(
        &mut s,
        b::Operation::Save {
            prefix: r.prefix.clone(),
            path: "textured.ignis-botany".into(),
        },
    )
    .unwrap();
    op(
        &mut s,
        b::Operation::Physics {
            prefix: r.prefix.clone(),
            action: dynamics::Action::Start {
                settings: Default::default(),
            },
        },
    )
    .unwrap();
    op(
        &mut s,
        b::Operation::Physics {
            prefix: r.prefix.clone(),
            action: dynamics::Action::Step { ticks: 2 },
        },
    )
    .unwrap();
    for (id, expected) in r.parts().iter().zip(&materials) {
        assert_eq!(
            &s.botany
                .overlay
                .as_ref()
                .unwrap()
                .asset(id)
                .unwrap()
                .material,
            expected
        );
    }
    op(
        &mut s,
        b::Operation::Physics {
            prefix: r.prefix.clone(),
            action: dynamics::Action::Reset,
        },
    )
    .unwrap();
    let scene: ignis::model::Scene =
        serde_json::from_slice(&serde_json::to_vec(&s.engine.scene).unwrap()).unwrap();
    scene.validate().unwrap();
    assert_eq!(scene, s.engine.scene);
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(next),
            policy: b::ManualPolicy::Discard,
        },
    )
    .unwrap();
    assert!(
        s.engine
            .scene
            .meshes
            .asset("plant_wood")
            .unwrap()
            .material
            .procedural
            .is_none()
    );
}
