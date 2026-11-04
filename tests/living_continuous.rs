use ignis::{
    botany::{
        self as b,
        growth::{Action, Definition, Plant, continuous::Profile, foliage},
    },
    engine::Command,
    model::RoomRecipe,
    service::Service,
};
fn specimen() -> Plant {
    Plant::new(
        Definition {
            version: 4,
            interval: 365,
            maximum_nodes: 10,
            internode_length: 0.55,
            continuous: Some(Profile::default()),
            foliage: Some(foliage::Profile::default()),
            ..Default::default()
        },
        0,
    )
    .unwrap()
}
#[test]
fn independent_curves_continue_after_reference_and_bake() {
    let p = Profile::default();
    p.validate().unwrap();
    assert_eq!(p.height(0), p.seedling_height_m);
    assert!((p.diameter(0) - p.seedling_diameter_m).abs() < 1e-12);
    assert!(
        (p.height((p.height_reference_years * 365.) as u64)
            - (p.seedling_height_m + 0.95 * (p.height_limit_m - p.seedling_height_m)))
            .abs()
            < 1e-10
    );
    assert!(
        (p.diameter((p.diameter_reference_years * 365.) as u64)
            - (p.seedling_diameter_m + 0.95 * (p.diameter_limit_m - p.seedling_diameter_m)))
            .abs()
            < 1e-10
    );
    let mut plant = specimen();
    let mut previous = (0., 0.);
    for day in [
        0,
        30,
        180,
        365,
        400,
        500,
        730,
        5 * 365,
        10 * 365,
        30 * 365,
        100 * 365,
        250 * 365,
        500 * 365,
        1000 * 365,
    ] {
        let r = plant.advance(day).unwrap();
        let b::Shape::Axis { radius, .. } = r.source.organs[0].shape else {
            panic!()
        };
        let height = r.source.organs[0].path.last().unwrap()[1];
        assert!((height - p.height(day)).abs() < 1e-9);
        assert!((radius * 2. - p.diameter(day)).abs() < 1e-9);
        assert!(height > previous.0 && radius * 2. > previous.1);
        previous = (height, radius * 2.);
        assert!(
            r.source
                .organs
                .iter()
                .any(|o| matches!(o.shape, b::Shape::Blade { .. }))
        );
    }
    let before = plant.recipe().unwrap();
    let mut changed = plant.clone();
    changed
        .definition
        .continuous
        .as_mut()
        .unwrap()
        .diameter_limit_m *= 0.8;
    let after = changed.recipe().unwrap();
    assert_eq!(before.source.organs[0].path, after.source.organs[0].path);
    let restored: Plant = serde_json::from_slice(&serde_json::to_vec(&plant).unwrap()).unwrap();
    assert_eq!(before, restored.recipe().unwrap());
}
#[test]
fn nodes_are_spaced_and_leaves_renew_without_losing_pruning() {
    let mut p = specimen();
    let mut counts = Vec::new();
    for day in [365, 380, 395, 425, 485] {
        let r = p.advance(day).unwrap();
        let leaves: Vec<_> = r
            .source
            .organs
            .iter()
            .filter(|o| {
                o.parent.as_deref() == Some("node0") && matches!(o.shape, b::Shape::Blade { .. })
            })
            .collect();
        let mut nodes = std::collections::BTreeMap::new();
        for leaf in &leaves {
            let index = leaf
                .id
                .split('/')
                .nth(1)
                .unwrap()
                .trim_start_matches('n')
                .parse::<u64>()
                .unwrap();
            nodes
                .entry(index)
                .or_insert(glam::DVec3::from_array(leaf.path[0]));
        }
        for pair in nodes.values().collect::<Vec<_>>().windows(2) {
            assert!(pair[0].distance(*pair[1]) >= 0.95 * Profile::default().internode_m);
        }
        counts.push(leaves.len());
        for leaf in &leaves {
            assert!(leaf.id.starts_with("node0/n"));
        }
        if day == 365 {
            assert_eq!(leaves.len(), 2);
        }
    }
    assert!(counts[4] > counts[0]);
    let a = p.advance(20 * 365).unwrap();
    let b = p.advance(22 * 365).unwrap();
    assert_ne!(
        a.source
            .organs
            .iter()
            .filter(|o| matches!(o.shape, ignis::botany::Shape::Blade { .. }))
            .map(|o| &o.id)
            .collect::<Vec<_>>(),
        b.source
            .organs
            .iter()
            .filter(|o| matches!(o.shape, ignis::botany::Shape::Blade { .. }))
            .map(|o| &o.id)
            .collect::<Vec<_>>()
    );
    p.production.remove_site("node0").unwrap();
    let pruned = p.advance(23 * 365).unwrap();
    assert!(
        !pruned
            .source
            .organs
            .iter()
            .any(|o| o.id == "node0" || o.id.starts_with("node0/"))
    );
}
#[test]
fn native_upgrade_is_atomic_and_old_definitions_are_unchanged() {
    let root = tempfile::tempdir().unwrap();
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.entities.clear();
    let mut s = Service::new(root.path(), scene).unwrap();
    let cmd = |s: &mut Service, op| {
        s.execute(
            Command::Botany {
                operation: Box::new(op),
            }
            .into(),
        )
    };
    cmd(
        &mut s,
        b::Operation::CreateLiving {
            definition: Definition {
                version: 2,
                interval: 365,
                maximum_nodes: 10,
                ..Default::default()
            },
            born: 0,
        },
    )
    .unwrap();
    cmd(
        &mut s,
        b::Operation::Living {
            prefix: "living".into(),
            action: Action::Advance { time: 365 * 5 },
        },
    )
    .unwrap();
    cmd(
        &mut s,
        b::Operation::Living {
            prefix: "living".into(),
            action: Action::SetContinuous {
                profile: Profile::default(),
            },
        },
    )
    .unwrap();
    assert_eq!(
        s.engine.scene.botany[0]
            .living
            .as_ref()
            .unwrap()
            .definition
            .version,
        4
    );
    let before = s.engine.scene.clone();
    let bad = Profile {internode_m:f64::NAN,..Default::default()};
    assert!(
        cmd(
            &mut s,
            b::Operation::Living {
                prefix: "living".into(),
                action: Action::SetContinuous { profile: bad }
            }
        )
        .is_err()
    );
    assert_eq!(s.engine.scene, before);
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        s.engine.scene.botany[0]
            .living
            .as_ref()
            .unwrap()
            .definition
            .version,
        2
    );
    s.execute(Command::Redo.into()).unwrap();
    assert_eq!(s.engine.scene, before);
    assert!(
        !serde_json::to_string(&Definition::default())
            .unwrap()
            .contains("continuous")
    );
}
