//! Critical planning boundary: no lost occupancy/pins, duplicate admissions or changed world identity.
#[path = "../src/world_regions.rs"]
pub mod world_regions;
#[path = "../src/world_space.rs"]
pub mod world_space;
use serde_json::{Value, json};
use world_regions::*;
use world_space::{Coordinates, Position};
fn point(cell: [i64; 3]) -> Position {
    Position::normalize(Coordinates {
        cell,
        offset: [0.; 3],
    })
    .unwrap()
}
fn manifest() -> Manifest {
    Manifest {
        version: 1,
        space_version: 1,
        world_id: "test-world".into(),
        revision: 1,
        seed: u64::MAX,
        generator: Generator {
            id: "terrain".into(),
            version: 3,
        },
        region_size_meters: 1024,
        budget: Budget {
            occupied: 64,
            requests: 64,
            probes: 65536,
        },
    }
}
fn interest(id: &str, cell: [i64; 3]) -> Interest {
    Interest {
        id: id.into(),
        position: point(cell),
        priority: 0,
        load_radius: [0; 3],
        retain_radius: [1; 3],
    }
}
fn query() -> Query {
    Query {
        manifest: manifest(),
        interests: vec![interest("player", [0; 3])],
        observed: vec![],
    }
}
fn observed(x: i64, phase: Phase, pins: Vec<Pin>) -> Observed {
    Observed {
        key: Key {
            cell: [x, 0, 0],
            offset_meters: [0; 3],
        },
        phase,
        pins,
    }
}
fn value(q: Query) -> Value {
    serde_json::to_value(plan(q).unwrap()).unwrap()
}
#[test]
fn planning_is_deterministic_bounded_and_never_reclaims_unreleased_or_pinned_regions() {
    let q = query();
    let v = value(q.clone());
    assert!(v["proposals_only"].as_bool().unwrap());
    assert_eq!(v["loads"][0]["key"]["cell"], json!(["0", "0", "0"]));
    // Golden cross-language FNV1a seed contract, not a region identity.
    assert_eq!(v["loads"][0]["seed"], json!("15003469448002236603"));
    let mut changed = q.clone();
    changed.manifest.revision = 2;
    changed.manifest.budget.requests = 1;
    assert_eq!(value(changed)["loads"][0]["seed"], v["loads"][0]["seed"]);
    let mut changed = q.clone();
    changed.manifest.generator.version += 1;
    assert_ne!(value(changed)["loads"][0]["seed"], v["loads"][0]["seed"]);
    let mut bytes = Vec::new();
    ciborium::into_writer(&q, &mut bytes).unwrap();
    let restored: Query = ciborium::from_reader(bytes.as_slice()).unwrap();
    assert_eq!(value(restored), v);
    let text = serde_json::to_string(&q).unwrap();
    assert!(text.contains("18446744073709551615"));
    assert_eq!(value(serde_json::from_str(&text).unwrap()), v);
    let mut negative = q.clone();
    negative.manifest.region_size_meters = 2048;
    negative.interests[0].position = point([-1, 0, 0]);
    let result = value(negative);
    assert_eq!(result["loads"][0]["key"]["cell"], json!(["-2", "0", "0"]));
    assert_eq!(
        result["loads"][0]["origin"]["cell"],
        json!(["-2", "0", "0"])
    );
    let mut fine = q.clone();
    fine.manifest.region_size_meters = 64;
    fine.interests[0].position = Position::normalize(Coordinates {
        cell: [9007199254740993, 0, 0],
        offset: [100., 0., 0.],
    })
    .unwrap();
    fine.interests[0].load_radius = [1, 0, 0];
    let fine = value(fine);
    assert_eq!(fine["region_meters"], 64.);
    assert_eq!(fine["loads"][0]["key"]["offset_meters"], json!([64, 0, 0]));
    assert_eq!(
        fine["loads"][0]["key"]["cell"],
        json!(["9007199254740993", "0", "0"])
    );
    let mut boundary = q.clone();
    boundary.manifest.region_size_meters = 64;
    boundary.interests[0].position = point([1, 0, 0]);
    boundary.interests[0].load_radius = [1, 0, 0];
    let boundary = value(boundary);
    assert!(
        boundary["loads"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"]["cell"] == json!(["0", "0", "0"])
                && r["key"]["offset_meters"] == json!([960, 0, 0]))
    );
    let mut crowd = q.clone();
    crowd.manifest.budget.probes = 9;
    crowd.interests = (0..1000)
        .map(|i| {
            let mut x = interest(&format!("player-{i:04}"), [9007199254740993, 0, 0]);
            x.load_radius = [1, 0, 1];
            x.retain_radius = [2, 0, 2];
            x
        })
        .collect();
    let original = value(crowd.clone());
    assert_eq!(original["totals"]["groups"], 1);
    assert_eq!(original["totals"]["probes"], 9);
    assert_eq!(original["loads"].as_array().unwrap().len(), 9);
    crowd.interests.reverse();
    assert_eq!(value(crowd.clone()), original);
    crowd.manifest.budget.probes = 8;
    assert!(plan(crowd).is_err());
    let mut priority = q.clone();
    priority.manifest.budget.occupied = 1;
    priority.manifest.budget.requests = 1;
    priority.interests[0].priority = i16::MIN;
    let mut high = interest("far", [i64::MAX, 0, 0]);
    high.priority = i16::MAX;
    priority.interests.push(high);
    let selected = value(priority.clone());
    assert_eq!(selected["loads"][0]["interest"], "far");
    priority.interests.reverse();
    assert_eq!(value(priority), selected);
    let mut capacity = q.clone();
    capacity.manifest.budget.occupied = 2;
    capacity.observed = vec![
        observed(-20, Phase::Resident, vec![]),
        observed(-30, Phase::Requested, vec![]),
    ];
    let held = value(capacity.clone());
    assert_eq!(held["loads"], json!([]));
    assert_eq!(held["unload_candidates"].as_array().unwrap().len(), 1);
    assert_eq!(held["cancel_candidates"].as_array().unwrap().len(), 1);
    assert_eq!(held["totals"]["deferred_capacity"], 1);
    capacity.observed.remove(0);
    assert_eq!(
        value(capacity.clone())["loads"].as_array().unwrap().len(),
        1
    );
    for phase in [
        Phase::Requested,
        Phase::Prepared,
        Phase::Resident,
        Phase::Retiring,
    ] {
        let mut pending = q.clone();
        pending.observed = vec![observed(0, phase, vec![])];
        assert_eq!(value(pending)["loads"], json!([]));
    }
    let mut retained = q.clone();
    retained.interests[0].retain_radius = [2, 0, 0];
    retained.observed = vec![observed(2, Phase::Resident, vec![])];
    assert_eq!(value(retained.clone())["unload_candidates"], json!([]));
    retained.interests[0].position = point([5, 0, 0]);
    assert_eq!(
        value(retained.clone())["unload_candidates"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for pin in [
        Pin::Control,
        Pin::Actor,
        Pin::Delivery,
        Pin::Origin,
        Pin::Contact,
        Pin::DirtyState,
    ] {
        retained.observed[0].pins = vec![pin];
        let r = value(retained.clone());
        assert_eq!(r["unload_candidates"], json!([]));
        assert_eq!(r["totals"]["held_by_pins"], 1);
    }
    retained.observed[0].pins = vec![Pin::Control, Pin::Control];
    assert!(plan(retained).is_err());
    let mut edge = q.clone();
    edge.interests[0].position = point([i64::MIN, 0, 0]);
    edge.interests[0].load_radius = [1, 0, 0];
    let edge = value(edge);
    assert_eq!(edge["loads"].as_array().unwrap().len(), 2);
    assert_eq!(edge["totals"]["coordinate_domain_clipped"], 1);
    let mut invalid_key = q.clone();
    invalid_key.manifest.region_size_meters = 64;
    invalid_key.observed = vec![observed(0, Phase::Resident, vec![])];
    invalid_key.observed[0].key.offset_meters = [1, 0, 0];
    assert!(plan(invalid_key).is_err());
    let mut duplicate_key = q.clone();
    duplicate_key.observed = vec![
        observed(0, Phase::Resident, vec![]),
        observed(0, Phase::Retiring, vec![]),
    ];
    assert!(plan(duplicate_key).is_err());
    let mut duplicate = q.clone();
    duplicate.interests.push(duplicate.interests[0].clone());
    assert!(plan(duplicate).is_err());
    let mut many = q.clone();
    many.interests = vec![q.interests[0].clone(); 1025];
    assert!(serde_json::from_value::<Query>(serde_json::to_value(many).unwrap()).is_err());
    let mut bad = q.clone();
    bad.manifest.seed = 1;
    let mut j = serde_json::to_value(bad).unwrap();
    j["manifest"]["seed"] = json!(9007199254740993_i64);
    assert!(serde_json::from_value::<Query>(j).is_err());
}
