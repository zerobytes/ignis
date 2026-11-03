//! Critical boundary: portable addresses must not lose identity of a location or overflow.
use ignis::{
    engine::{Command, Engine},
    model::RoomRecipe,
    world_space::{Coordinates, Position, Query},
};
use serde_json::json;
fn point(cell: [i64; 3], offset: [f64; 3]) -> Position {
    Position::normalize(Coordinates { cell, offset }).unwrap()
}
#[test]
fn canonical_global_addresses_preserve_precision_formats_and_stateless_queries() {
    // Independent integer quarter-meter oracle at both ends of the address space.
    for base in [
        i64::MIN,
        i64::MIN + 1,
        -9007199254740993,
        -1,
        0,
        1,
        9007199254740993,
        i64::MAX - 1,
        i64::MAX,
    ] {
        for quarters in [-4097_i128, -4096, -1, 0, 1, 4095, 4096, 4097] {
            let absolute = i128::from(base) * 4096 + quarters;
            let expected_cell = absolute.div_euclid(4096);
            let answer = Position::normalize(Coordinates {
                cell: [base, 0, 0],
                offset: [quarters as f64 / 4., 0., 0.],
            });
            if let Ok(cell) = i64::try_from(expected_cell) {
                let answer = answer.unwrap();
                let raw: Coordinates = answer.into();
                assert_eq!(raw.cell[0], cell);
                assert_eq!(raw.offset[0], absolute.rem_euclid(4096) as f64 / 4.);
                let text = serde_json::to_string(&answer).unwrap();
                assert_eq!(serde_json::from_str::<Position>(&text).unwrap(), answer);
                assert!(text.contains(&format!("\"{cell}\"")));
                let mut bytes = Vec::new();
                ciborium::into_writer(&answer, &mut bytes).unwrap();
                assert_eq!(
                    ciborium::from_reader::<Position, _>(bytes.as_slice()).unwrap(),
                    answer
                );
                let value: ciborium::Value = ciborium::from_reader(bytes.as_slice()).unwrap();
                let cells = value
                    .as_map()
                    .unwrap()
                    .iter()
                    .find(|(k, _)| k.as_text() == Some("cell"))
                    .unwrap()
                    .1
                    .as_array()
                    .unwrap();
                assert!(cells.iter().all(|v| v.as_integer().is_some()));
            } else {
                assert!(answer.is_err());
            }
        }
    }
    let origin = point(
        [i64::MAX - 2, i64::MIN + 2, 9007199254740993],
        [0.125, 0.25, 0.5],
    );
    let nearby = origin.translated([2.25, -0.5, 1. / 1024.]).unwrap();
    assert_eq!(
        nearby.project(origin, 8., 0.).unwrap(),
        [2.25, -0.5, 1. / 1024.]
    );
    let p = point([0; 3], [1. / 1024., 0., 0.]);
    let large = p.translated([2_f64.powi(40) + 0.25, 0., 0.]).unwrap();
    let raw: Coordinates = large.into();
    assert_eq!(raw.cell[0], 1_i64 << 30);
    assert_eq!(raw.offset[0], 0.25 + 1. / 1024.);
    let negative: Coordinates = point([0; 3], [-f64::from_bits(1), 0., 0.]).into();
    assert_eq!(negative.cell, [0; 3]); // underflow must never add a spurious cell
    let zero = point([0; 3], [0.; 3]);
    assert!(
        point([i64::MAX, 0, 0], [0.; 3])
            .project(point([i64::MIN, 0, 0], [0.; 3]), 8., 1.)
            .is_err()
    );
    assert!(
        point([i64::MAX, 0, 0], [1023.; 3])
            .translated([1., 0., 0.])
            .is_err()
    );
    assert!(
        point([0; 3], [0.1, 0., 0.])
            .project(zero, 1., 1e-12)
            .is_err()
    );
    assert!(point([0; 3], [0.1, 0., 0.]).project(zero, 1., 1e-6).is_ok());
    for offset in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        assert!(
            Position::normalize(Coordinates {
                cell: [0; 3],
                offset: [offset, 0., 0.]
            })
            .is_err()
        );
    }
    for cell in ["+1", "01", "-0", "9223372036854775808"] {
        assert!(
            serde_json::from_value::<Position>(json!({"cell":[cell,"0","0"],"offset":[0,0,0]}))
                .is_err()
        );
    }
    assert!(
        serde_json::from_value::<Position>(
            json!({"cell":[9007199254740993_i64,0,0],"offset":[0,0,0]})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<Position>(json!({"cell":["0","0","0"],"offset":[1024,0,0]}))
            .is_err()
    );
    let mut engine = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    let before = engine.snapshot();
    let command = Command::QueryWorldSpace {
        query: Query::Project {
            point: nearby,
            origin,
            radius_meters: 8.,
            max_error_meters: 0.,
        },
    };
    assert_eq!(
        engine.execute(command.into()).unwrap()["local"],
        json!([2.25, -0.5, 1. / 1024.])
    );
    let command = Command::QueryWorldSpace {
        query: Query::Project {
            point: nearby,
            origin: zero,
            radius_meters: 8.,
            max_error_meters: 0.,
        },
    };
    assert!(engine.execute(command.into()).is_err());
    assert_eq!(engine.snapshot(), before);
}
