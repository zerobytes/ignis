use super::*;
use crate::terrain::surface::{Knot, Node, Program};

#[test]
fn planet_orbit_surface_program_certifies_unsampled_peaks_and_all_cube_faces() {
    let mut c = super::tests::config();
    c.document.version = 2;
    let program = Program::new(
        vec![
            Node::Noise {
                seed: 91,
                period_meters: 4096,
                octaves: 4,
                persistence: 0.5,
                amplitude_meters: 1.,
                style: crate::terrain::NoiseKind::Smooth,
            },
            Node::Remap {
                input: 0,
                knots: vec![
                    Knot {
                        x: -1.,
                        value: 0.,
                        derivative: 2000.,
                    },
                    Knot {
                        x: 1.,
                        value: 0.,
                        derivative: -2000.,
                    },
                ],
            },
            Node::Raise {
                center: c
                    .document
                    .center
                    .translated([0., c.document.radius(), 0.])
                    .unwrap(),
                radius_meters: 100.,
                amount_meters: 800.,
            },
            Node::Add { a: 1, b: 2 },
        ],
        3,
    )
    .unwrap();
    c.document.operations = vec![Relief::Surface {
        program: Box::new(program),
    }];
    let sampler = Sampler::new(&c.document).unwrap();
    for face in FACES {
        for level in [0, 3, 12] {
            let t = Tile {
                face,
                level,
                xy: if level == 0 {
                    [0, 0]
                } else {
                    [1u32 << (level - 1); 2]
                },
            };
            let bounds = global_bounds(&c.document, t);
            let lo = bounds[0];
            let hi = bounds[7];
            for y in 0..=12 {
                for x in 0..=12 {
                    let d = t.direction([f64::from(x) / 12., f64::from(y) / 12.]);
                    let p = d * (c.document.radius() + sampler.height(d));
                    assert!((p - lo).min_element() > -1e-6 && (hi - p).min_element() > -1e-6);
                }
            }
            assert!(error(&c.document, t, 24, 32) > 0.);
        }
    }
    // A very narrow authored peak cannot disappear because tile corners missed it.
    let t = Tile {
        face: Face::PositiveY,
        level: 12,
        xy: [2048, 2048],
    };
    let d = DVec3::Y;
    let peak = d * (c.document.radius() + sampler.height(d));
    let bounds = global_bounds(&c.document, t);
    assert!((peak - bounds[0]).min_element() > -1e-6 && (bounds[7] - peak).min_element() > -1e-6);
    assert!(error(&c.document, t, 24, 32) > 800.);
}
