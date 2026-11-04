//! Native command/state integration, without graphics or external preparation scripts.
use ignis::{
    botany::{
        self as b,
        growth::{Action, Definition},
        production::FruitId,
    },
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
use std::collections::BTreeMap;
fn op(s: &mut Service, operation: b::Operation) -> ignis::model::Result<ignis::service::Reply> {
    s.execute(
        Command::Botany {
            operation: Box::new(operation),
        }
        .into(),
    )
}
fn live(s: &mut Service, action: Action) -> ignis::model::Result<ignis::service::Reply> {
    op(
        s,
        b::Operation::Living {
            prefix: "living".into(),
            action,
        },
    )
}
fn count(s: &Service, part: &str) -> usize {
    s.engine
        .scene
        .meshes
        .asset(&format!("living_{part}"))
        .unwrap()
        .geometry
        .triangles
        .len()
}
fn state(s: &Service) -> &b::growth::Plant {
    s.engine.scene.botany[0].living.as_ref().unwrap()
}
#[test]
fn commands_grow_reproduce_prune_harvest_and_restore_canonical_scene() {
    let root = tempfile::tempdir().unwrap();
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.entities.clear();
    let mut s = Service::new(root.path(), scene).unwrap();
    op(
        &mut s,
        b::Operation::CreateLiving {
            definition: Definition::default(),
            born: 0,
        },
    )
    .unwrap();
    assert_eq!(s.engine.scene.version, b::LIVING_SCENE_VERSION);
    assert_eq!(s.engine.scene.botany[0].source.organs.len(), 1);
    assert_eq!(count(&s, "flowers"), 1);
    live(&mut s, Action::Advance { time: 40 }).unwrap();
    assert_eq!(s.engine.scene.botany[0].source.organs.len(), 13);
    live(&mut s, Action::Advance { time: 80 }).unwrap();
    assert!(count(&s, "flowers") > 100);
    live(&mut s, Action::Advance { time: 86 }).unwrap();
    assert!(count(&s, "green_fruit") > 100);
    live(&mut s, Action::Advance { time: 90 }).unwrap();
    assert!(count(&s, "ripe_fruit") > 100);
    let fruit = FruitId {
        site: "node0".into(),
        cohort: 0,
        slot: 0,
    };
    live(
        &mut s,
        Action::Harvest {
            fruit: fruit.clone(),
        },
    )
    .unwrap();
    live(
        &mut s,
        Action::Prune {
            site: "node1".into(),
        },
    )
    .unwrap();
    let before = s.engine.scene.clone();
    s.execute(
        Command::SaveReplay {
            path: "living.replay.json".into(),
        }
        .into(),
    )
    .unwrap();
    let result = s
        .execute(
            Command::RunReplay {
                path: "living.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(result.data["matched"], true);
    s.execute(Command::Undo.into()).unwrap();
    assert!(!state(&s).production.removed_sites.contains("node1"));
    s.execute(Command::Redo.into()).unwrap();
    assert_eq!(s.engine.scene, before);
    let revision = s.engine.revision;
    assert!(
        live(
            &mut s,
            Action::Harvest {
                fruit: fruit.clone()
            }
        )
        .is_err()
    );
    assert!(live(&mut s, Action::Advance { time: 10 }).is_err());
    assert_eq!(s.engine.scene, before);
    assert_eq!(s.engine.revision, revision);
    s.execute(
        Command::SaveScene {
            path: "living.scene.json".into(),
        }
        .into(),
    )
    .unwrap();
    live(&mut s, Action::Advance { time: 100 }).unwrap();
    s.execute(
        Command::LoadScene {
            path: "living.scene.json".into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.scene, before);
    s.execute(
        Command::SavePortableSave {
            path: "living.ignis-portable".into(),
        }
        .into(),
    )
    .unwrap();
    live(&mut s, Action::Advance { time: 180 }).unwrap();
    s.execute(
        Command::LoadPortableSave {
            path: "living.ignis-portable".into(),
            mode: None,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.scene, before);
    op(
        &mut s,
        b::Operation::Save {
            prefix: "living".into(),
            path: "plant.ignis-botany".into(),
        },
    )
    .unwrap();
    live(&mut s, Action::Advance { time: 180 }).unwrap();
    assert!(count(&s, "flowers") > 100);
    live(&mut s, Action::Advance { time: 190 }).unwrap();
    assert!(count(&s, "ripe_fruit") > 100);
    assert!(state(&s).production.harvested.contains(&fruit));
    assert!(state(&s).production.removed_sites.contains("node1"));
    assert!(
        !s.engine.scene.botany[0]
            .source
            .organs
            .iter()
            .any(|o| o.id == "node1")
    );
    let attached = state(&s)
        .production
        .attached(&state(&s).definition.schedule, 128)
        .unwrap();
    assert_eq!(attached.len(), 7);
    assert!(
        attached
            .iter()
            .any(|(id, _)| id.site == "node0" && id.cohort == 1)
    );
    op(
        &mut s,
        b::Operation::Load {
            path: "plant.ignis-botany".into(),
            publish: true,
        },
    )
    .unwrap();
    assert_eq!(s.engine.scene.botany[0], before.botany[0]);
    let mut legacy = s.engine.scene.clone();
    legacy.version = 120;
    assert!(legacy.validate().is_err());
    let recipe = s.engine.scene.botany[0].clone();
    let mut wrong = recipe.clone();
    wrong.version = 7;
    assert!(wrong.validate().is_err());
    let mut wrong = recipe.clone();
    wrong
        .living
        .as_mut()
        .unwrap()
        .production
        .site_births
        .insert("node0".into(), 0);
    assert!(wrong.validate().is_err());
    let mut wrong = recipe;
    std::sync::Arc::make_mut(&mut wrong.source).organs[0].path[1][1] += 1.;
    assert!(wrong.validate().is_err());
}
#[test]
fn produce_geometry_is_closed_oriented_and_manual_edits_are_not_lost() {
    let mut plant = b::growth::Plant::new(Definition::default(), 0).unwrap();
    plant.advance(80).unwrap();
    for time in [80, 86, 90] {
        plant.advance(time).unwrap();
        let compiled = b::bake::compile(&plant.bound_recipe().unwrap()).unwrap();
        for asset in &compiled.document.assets[2..] {
            if asset.geometry.triangles.len() == 1 {
                continue;
            }
            let mut edges = BTreeMap::new();
            let mut volume = 0.;
            for tri in &asset.geometry.triangles {
                for k in 0..3 {
                    *edges.entry((tri[k], tri[(k + 1) % 3])).or_insert(0) += 1;
                }
                let a = glam::DVec3::from_array(
                    asset.geometry.positions[tri[0] as usize].map(f64::from),
                );
                let b = glam::DVec3::from_array(
                    asset.geometry.positions[tri[1] as usize].map(f64::from),
                );
                let c = glam::DVec3::from_array(
                    asset.geometry.positions[tri[2] as usize].map(f64::from),
                );
                volume += a.dot(b.cross(c)) / 6.;
            }
            assert!(volume > 0., "{}", asset.id);
            assert!(
                edges
                    .iter()
                    .all(|((a, b), n)| *n == 1 && edges.get(&(*b, *a)) == Some(&1))
            );
        }
    }
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    op(
        &mut s,
        b::Operation::Publish {
            recipe: Box::new(plant.bound_recipe().unwrap()),
            policy: Default::default(),
        },
    )
    .unwrap();
    let mut asset = s
        .engine
        .scene
        .meshes
        .asset("living_ripe_fruit")
        .unwrap()
        .clone();
    std::sync::Arc::make_mut(&mut asset.geometry).positions[0][0] += 0.01;
    s.execute(
        Command::EditMesh {
            edit: ignis::meshes::Edit::PutAsset { asset },
        }
        .into(),
    )
    .unwrap();
    let before = s.engine.scene.clone();
    assert!(live(&mut s, Action::Advance { time: 100 }).is_err());
    assert_eq!(before, s.engine.scene);
}

#[test]
fn age_preview_is_reversible_and_preserves_saved_interventions() {
    let root = tempfile::tempdir().unwrap();
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.entities.clear();
    let mut s = Service::new(root.path(), scene).unwrap();
    op(
        &mut s,
        b::Operation::CreateLiving {
            definition: Definition::default(),
            born: 0,
        },
    )
    .unwrap();
    live(&mut s, Action::Advance { time: 90 }).unwrap();
    live(
        &mut s,
        Action::Harvest {
            fruit: FruitId {
                site: "node0".into(),
                cohort: 0,
                slot: 0,
            },
        },
    )
    .unwrap();
    live(
        &mut s,
        Action::Prune {
            site: "node1".into(),
        },
    )
    .unwrap();
    let saved = s.engine.scene.clone();
    let revision = s.engine.revision;
    let preview = |s: &mut Service, time| {
        op(
            s,
            b::Operation::PreviewLivingAge {
                prefix: "living".into(),
                time,
            },
        )
    };
    preview(&mut s, 40).unwrap();
    let young = s.botany.overlay.clone().unwrap();
    preview(&mut s, 190).unwrap();
    assert_ne!(s.botany.overlay.as_ref().unwrap(), &young);
    preview(&mut s, 40).unwrap();
    assert_eq!(s.botany.overlay.as_ref().unwrap(), &young);
    assert_eq!(s.engine.scene, saved);
    assert_eq!(s.engine.revision, revision);
    assert!(
        s.botany
            .preview
            .as_ref()
            .unwrap()
            .living
            .as_ref()
            .unwrap()
            .production
            .removed_sites
            .is_empty()
    );
    preview(&mut s, 190).unwrap();
    let ripe = s
        .botany
        .overlay
        .as_ref()
        .unwrap()
        .asset("living_ripe_fruit")
        .unwrap()
        .geometry
        .clone();
    let mut camera = s.engine.scene.camera.clone();
    camera.position[0] += 0.1;
    s.execute(Command::SetCamera { camera }.into()).unwrap();
    assert_eq!(
        s.botany
            .overlay
            .as_ref()
            .unwrap()
            .asset("living_ripe_fruit")
            .unwrap()
            .geometry,
        ripe
    );
    let overlay = s.botany.overlay.clone();
    assert!(preview(&mut s, b::growth::MAX_TIME + 1).is_err());
    assert_eq!(s.botany.overlay, overlay);
    op(&mut s, b::Operation::CancelPreview).unwrap();
    assert!(s.botany.overlay.is_none());
    assert_eq!(s.engine.scene.botany, saved.botany);
    assert_eq!(s.engine.scene.meshes, saved.meshes);
    assert!(state(&s).production.removed_sites.contains("node1"));
    assert_eq!(state(&s).production.harvested.len(), 1);
    assert_eq!(state(&s).production.plant.time, 90);
}
