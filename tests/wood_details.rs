use ignis::{
    botany::{self as b, wood_details as d},
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
use std::collections::BTreeMap;
fn roots() -> b::Recipe {
    serde_json::from_str(include_str!(
        "../assets/botany/wood-authoring-with-roots.ignis-botany"
    ))
    .unwrap()
}
fn living() -> b::Recipe {
    let mut p = b::growth::Plant::new(
        b::growth::sequoia_realistic::definition("history".into(), 83),
        0,
    )
    .unwrap();
    p.advance(365 * 40).unwrap();
    p.bound_recipe().unwrap()
}
fn closed(c: &b::bake::Compiled) {
    let g = &c.document.assets[0].geometry;
    let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
    let mut volume = 0.;
    for &[a, b, c] in &g.triangles {
        for (a, b) in [(a, b), (b, c), (c, a)] {
            let e = edges.entry((a.min(b), a.max(b))).or_default();
            e.0 += 1;
            e.1 += if a < b { 1 } else { -1 };
        }
        let p = glam::DVec3::from_array(g.positions[a as usize].map(f64::from));
        let q = glam::DVec3::from_array(g.positions[b as usize].map(f64::from));
        let r = glam::DVec3::from_array(g.positions[c as usize].map(f64::from));
        assert!((q - p).cross(r - p).length_squared() > 0.);
        volume += p.dot(q.cross(r)) / 6.;
    }
    assert!(volume > 0.);
    assert!(edges.values().all(|&(n, s)| n == 2 && s == 0));
}
fn call(s: &mut Service, op: b::Operation) -> ignis::model::Result<ignis::service::Reply> {
    s.execute(
        Command::Botany {
            operation: Box::new(op),
        }
        .into(),
    )
}
#[test]
fn irregular_roots_are_seeded_closed_and_keep_the_source() {
    let old = roots();
    let baseline = b::bake::compile(&old).unwrap();
    closed(&baseline);
    let mut r = old.clone();
    r.version = 12;
    r.wood_details = Some(d::Details {
        roots: Some(Default::default()),
        ..Default::default()
    });
    let c = b::bake::compile(&r).unwrap();
    closed(&c);
    assert_eq!(r.source, old.source);
    assert_ne!(c.document, baseline.document);
    assert_eq!(c.document, b::bake::compile(&r).unwrap().document);
    let encoded = serde_json::to_vec(&r).unwrap();
    let restored: b::Recipe = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(c.document, b::bake::compile(&restored).unwrap().document);
    r.wood_details.as_mut().unwrap().seed += 1;
    assert_ne!(c.document, b::bake::compile(&r).unwrap().document);
    r.wood_details = None;
    assert_eq!(baseline.document, b::bake::compile(&r).unwrap().document);
}
#[test]
fn loss_is_persistent_scar_ages_and_stays_at_metric_distance() {
    let mut r = living();
    r.settings.leaf_density = 0.;
    r.version = 12;
    r.wood_details = Some(Default::default());
    d::record_loss(&mut r, "node14", d::Cause::Breakage, None).unwrap();
    let event = r.wood_details.as_ref().unwrap().scars[0].clone();
    assert_eq!(event.year, 40.);
    assert!(event.distance_m > 0.);
    let fresh = b::bake::compile(&r).unwrap();
    closed(&fresh);
    assert!(
        !fresh
            .poses
            .iter()
            .any(|p| r.source.organs[p.source].id == "node14")
    );
    let mut plant = r.living.clone().unwrap();
    plant.advance(60 * 365).unwrap();
    let mut old = plant.bound_recipe().unwrap();
    old.version = 12;
    old.settings = r.settings.clone();
    old.wood_details = r.wood_details.clone();
    let aged = b::bake::compile(&old).unwrap();
    closed(&aged);
    for c in [&fresh, &aged] {
        let v = c.vertices[0]
            .iter()
            .find(|v| v.key.contains(":scar-v1:loss0:0/"))
            .unwrap();
        let p = &c.poses[v.organ];
        let x = v.u * (p.samples.len() - 1) as f64;
        let i = (x as usize).min(p.samples.len() - 2);
        let metric = p
            .samples
            .windows(2)
            .take(i)
            .map(|f| f[0].p.distance(f[1].p))
            .sum::<f64>()
            + p.samples[i].p.distance(p.samples[i + 1].p) * (x - i as f64);
        assert!((metric - event.distance_m).abs() < 1e-8);
        let surface = p.wood_surface.as_ref().unwrap();
        for a in [-3., -1., 0., 1., 3.] {
            let b = a + std::f64::consts::TAU;
            assert!(
                (d::marked_radius(surface, &p.patches, v.u, a)
                    - d::marked_radius(surface, &p.patches, v.u, b))
                .abs()
                    < 1e-10
            );
            assert!(
                (d::grain_angle(surface, &p.patches, v.u, b)
                    - d::grain_angle(surface, &p.patches, v.u, a)
                    - std::f64::consts::TAU)
                    .abs()
                    < 1e-10
            );
        }
    }
    assert_eq!(old.wood_details.as_ref().unwrap().scars[0], event);
    assert!(aged.vertices[0].iter().any(|v| v.key.contains(":scar-v1:")));
    assert!(
        !aged
            .poses
            .iter()
            .any(|p| old.source.organs[p.source].id == "node14")
    );
    assert_ne!(fresh.document, aged.document);
    let mut hidden = old.clone();
    hidden.wood_details.as_mut().unwrap().scars[0].visible = false;
    d::preserve_losses(old.wood_details.as_ref(), hidden.wood_details.as_ref()).unwrap();
    let hidden_model = b::bake::compile(&hidden).unwrap();
    assert!(
        !hidden_model
            .poses
            .iter()
            .any(|p| hidden.source.organs[p.source].id == "node14")
    );
    assert_eq!(
        aged.vertices[0].iter().map(|v| &v.key).collect::<Vec<_>>(),
        hidden_model.vertices[0]
            .iter()
            .map(|v| &v.key)
            .collect::<Vec<_>>()
    );
    let round: b::Recipe = serde_json::from_slice(&serde_json::to_vec(&old).unwrap()).unwrap();
    assert_eq!(aged.document, b::bake::compile(&round).unwrap().document);
    let mut p = b::growth::Plant::new(r.living.as_ref().unwrap().definition.clone(), 0).unwrap();
    p.advance(39 * 365).unwrap();
    let mut before = p.bound_recipe().unwrap();
    before.version = 12;
    before.settings = r.settings;
    before.wood_details = r.wood_details.clone();
    assert!(
        b::bake::poses(&before)
            .unwrap()
            .iter()
            .any(|p| before.source.organs[p.source].id == "node14")
    );
    assert!(d::preserve_losses(old.wood_details.as_ref(), None).is_err());
    assert!(d::record_loss(&mut old, "node14", d::Cause::Pruning, None).is_err());
}
#[test]
fn natural_history_is_independent_of_time_step_and_transactions_are_atomic() {
    let mut initial = living();
    initial.version = 12;
    initial.settings.leaf_density = 0.;
    initial.wood_details = Some(d::Details {
        natural_loss: Some(d::NaturalLoss {
            first_year: 41.,
            interval_years: 3.,
            maximum_events: 3,
        }),
        ..Default::default()
    });
    let mut big = initial.living.clone().unwrap();
    let history = d::advance(&initial, &mut big, 55 * 365).unwrap();
    assert_eq!(history.as_ref().unwrap().scars.len(), 3);
    let mut small = initial.clone();
    for year in 41..=55 {
        let mut p = small.living.clone().unwrap();
        let details = d::advance(&small, &mut p, year * 365).unwrap();
        let mut next = p.bound_recipe().unwrap();
        next.version = 12;
        next.settings = small.settings.clone();
        next.wood_details = details;
        small = next;
    }
    assert_eq!(small.living.as_ref().unwrap(), &big);
    assert_eq!(small.wood_details, history);
    let root = tempfile::tempdir().unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    call(
        &mut service,
        b::Operation::Publish {
            recipe: Box::new(initial.clone()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let baseline = service.engine.scene.clone();
    call(
        &mut service,
        b::Operation::PreviewLivingAge {
            prefix: "history".into(),
            time: 55 * 365,
        },
    )
    .unwrap();
    assert_eq!(service.engine.scene, baseline);
    assert_eq!(
        service.botany.preview.as_ref().unwrap().wood_details,
        history
    );
    let future = service.botany.overlay.clone().unwrap();
    call(&mut service, b::Operation::CancelPreview).unwrap();
    call(
        &mut service,
        b::Operation::Living {
            prefix: "history".into(),
            action: b::growth::Action::Advance { time: 55 * 365 },
        },
    )
    .unwrap();
    assert_eq!(service.engine.scene.meshes, future);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, baseline);

    let mut bad = initial.wood_details.clone().unwrap();
    bad.roots = Some(d::RootProfile {
        swelling: f64::NAN,
        ..Default::default()
    });
    assert!(
        call(
            &mut service,
            b::Operation::SetWoodDetails {
                prefix: "history".into(),
                details: Some(bad),
                preview: false,
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, baseline);
    call(
        &mut service,
        b::Operation::EditOrgan {
            prefix: "history".into(),
            organ: "node14".into(),
            edit: b::OrganEdit {
                radius: 1.1,
                ..Default::default()
            },
        },
    )
    .unwrap();
    let baseline = service.engine.scene.clone();
    call(
        &mut service,
        b::Operation::LoseBranch {
            prefix: "history".into(),
            organ: "node14".into(),
            cause: d::Cause::Pruning,
            policy: Default::default(),
        },
    )
    .unwrap();
    let changed = service.engine.scene.clone();
    assert_eq!(
        changed.botany[0].wood_details.as_ref().unwrap().scars.len(),
        1
    );
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, baseline);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, changed);
    assert!(
        call(
            &mut service,
            b::Operation::SetWoodDetails {
                prefix: "history".into(),
                details: None,
                preview: false,
                policy: Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(service.engine.scene, changed);
}

#[test]
fn existing_maturity_losses_are_recorded_with_their_subtrees() {
    let mut p = b::growth::Plant::new(
        b::growth::sequoia_realistic::definition("maturity".into(), 83),
        0,
    )
    .unwrap();
    p.advance(299 * 365).unwrap();
    let mut old = p.bound_recipe().unwrap();
    old.version = 12;
    old.settings.leaf_density = 0.;
    old.wood_details = Some(Default::default());
    let mut big = p.clone();
    let all = d::advance(&old, &mut big, 301 * 365).unwrap().unwrap();
    assert_eq!(all.scars.len(), 11);
    assert!(
        all.scars
            .iter()
            .all(|s| s.year == (300 * 365 + 1) as f64 / 365.
                && s.removed_organs.contains(s.branch.as_ref().unwrap()))
    );
    let details = d::advance(&old, &mut p, 300 * 365).unwrap();
    assert!(details.as_ref().unwrap().scars.is_empty());
    let mut middle = p.bound_recipe().unwrap();
    middle.version = 12;
    middle.settings = old.settings;
    middle.wood_details = details;
    let split = d::advance(&middle, &mut p, 301 * 365).unwrap().unwrap();
    assert_eq!(split, all);
    assert_eq!(p, big);
    let mut simple =
        b::growth::Plant::new(b::growth::sequoia::definition("grouped".into(), 13), 0).unwrap();
    simple.advance(30 * 365).unwrap();
    let mut r = simple.bound_recipe().unwrap();
    r.settings.leaf_density = 0.;
    r.version = 12;
    r.wood_details = Some(Default::default());
    d::record_site_loss(&mut r, "node0", d::Cause::Pruning).unwrap();
    assert_eq!(r.wood_details.as_ref().unwrap().scars.len(), 1);
    assert!(
        r.wood_details.as_ref().unwrap().scars[0]
            .removed_organs
            .contains(&"node0/arm1".into())
    );
    assert!(
        r.wood_details.as_ref().unwrap().scars[0]
            .removed_organs
            .contains(&"node0/arm2".into())
    );
}
