//! Bounded shape-preserving cubic section channels in physical longitudinal Y.
use super::{SectionInterpolation, Sections, Station};

pub(super) struct Curves {
    /// Interval -> width/depth/offsetX/offsetZ -> descending powers of fraction.
    coefficients: [[[f64; 4]; 4]; 15],
}

fn channels(s: &Station) -> [f64; 4] {
    [s.width, s.depth, s.offset[0], s.offset[1]].map(f64::from)
}

fn end_slope(h: f64, adjacent_h: f64, delta: f64, adjacent_delta: f64) -> f64 {
    let slope = ((2. * h + adjacent_h) * delta - h * adjacent_delta) / (h + adjacent_h);
    if slope.signum() != delta.signum() {
        0.
    } else if delta.signum() != adjacent_delta.signum() && slope.abs() > 3. * delta.abs() {
        3. * delta
    } else {
        slope
    }
}

impl Curves {
    pub(super) fn new(sections: &Sections) -> Option<Self> {
        if sections.interpolation == SectionInterpolation::Linear {
            return None;
        }
        // Definition::validate precedes sampling/evaluation. Public point maps
        // retain their existing contract of requiring a valid definition.
        assert!((2..=16).contains(&sections.stations.len()));
        let intervals = sections.stations.len() - 1;
        let mut spans = [0.; 15];
        let mut secants = [[0.; 4]; 15];
        let mut slopes = [[0.; 4]; 16];
        for i in 0..intervals {
            let a = &sections.stations[i];
            let b = &sections.stations[i + 1];
            spans[i] = f64::from(b.at) - f64::from(a.at);
            let (a, b) = (channels(a), channels(b));
            for k in 0..4 {
                secants[i][k] = (b[k] - a[k]) / spans[i];
            }
        }
        for k in 0..4 {
            if intervals == 1 {
                slopes[0][k] = secants[0][k];
                slopes[1][k] = secants[0][k];
                continue;
            }
            slopes[0][k] = end_slope(spans[0], spans[1], secants[0][k], secants[1][k]);
            let last = intervals - 1;
            slopes[intervals][k] = end_slope(
                spans[last],
                spans[last - 1],
                secants[last][k],
                secants[last - 1][k],
            );
            for i in 1..intervals {
                let before = secants[i - 1][k];
                let after = secants[i][k];
                if before == 0. || after == 0. || before.signum() != after.signum() {
                    slopes[i][k] = 0.;
                } else {
                    let w1 = 2. * spans[i] + spans[i - 1];
                    let w2 = spans[i] + 2. * spans[i - 1];
                    slopes[i][k] = (w1 + w2) / (w1 / before + w2 / after);
                }
            }
        }
        let mut coefficients = [[[0.; 4]; 4]; 15];
        for i in 0..intervals {
            let a = channels(&sections.stations[i]);
            let b = channels(&sections.stations[i + 1]);
            for k in 0..4 {
                let delta = b[k] - a[k];
                let first = spans[i] * slopes[i][k];
                let last = spans[i] * slopes[i + 1][k];
                coefficients[i][k] = [
                    -2. * delta + first + last,
                    3. * delta - 2. * first - last,
                    first,
                    a[k],
                ];
            }
        }
        Some(Self { coefficients })
    }
}

pub(super) fn values_at(
    sections: &Sections,
    curves: Option<&Curves>,
    interval: usize,
    fraction: f64,
) -> [f64; 4] {
    let a = channels(&sections.stations[interval]);
    let b = channels(&sections.stations[interval + 1]);
    let Some(curves) = curves else {
        // Preserve the exact legacy operation order, including nonuniform stations.
        return std::array::from_fn(|k| a[k] * (1. - fraction) + b[k] * fraction);
    };
    if fraction == 0. {
        return a;
    }
    if fraction == 1. {
        return b;
    }
    curves.coefficients[interval]
        .map(|[a, b, c, d]| ((a * fraction + b) * fraction + c) * fraction + d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plants::CrossSection;

    fn drawing(at: &[f32], offsets: &[f32]) -> Sections {
        Sections {
            id: "sections".into(),
            cross_section: CrossSection::Ellipse,
            interpolation: SectionInterpolation::Pchip,
            stations: at
                .iter()
                .zip(offsets)
                .enumerate()
                .map(|(i, (&at, &x))| Station {
                    id: format!("node{i}"),
                    at,
                    width: 0.5,
                    depth: 0.5,
                    offset: [x, -x],
                })
                .collect(),
        }
    }

    #[test]
    fn nonuniform_harmonic_derivatives_match_independent_rational_values() {
        // Three-point scalar data (x,y)=(0,0),(1,1),(3,2). Normalizing x
        // preserves the values. Tangents in original x:7/6,9/13,1/6.
        let s = drawing(&[0., 1. / 3., 1.], &[0., 1., 2.]);
        let c = Curves::new(&s).unwrap();
        assert!((values_at(&s, Some(&c), 0, 0.5)[2] - 349. / 624.).abs() < 2e-8);
        assert!((values_at(&s, Some(&c), 1, 0.5)[2] - 509. / 312.).abs() < 2e-8);
    }

    #[test]
    fn extrema_plateaus_endpoint_limiter_and_two_nodes_remain_bounded() {
        for (at, offsets) in [
            (vec![0., 1.], vec![-100., 100.]),
            (
                vec![0., 0.001, 0.25, 0.9, 1.],
                vec![0., 99., 100., -100., -100.],
            ),
            (vec![0., 0.1, 0.5, 1.], vec![1., 1., 1., 1.]),
            (vec![0., 0.01, 0.5, 1.], vec![0., 0.001, 10., 0.]),
        ] {
            let mut s = drawing(&at, &offsets);
            for (i, station) in s.stations.iter_mut().enumerate() {
                station.width = if i % 2 == 0 { 0.001 } else { 1. };
                station.depth = if i % 2 == 0 { 1. } else { 0.001 };
            }
            let c = Curves::new(&s).unwrap();
            for i in 0..at.len() - 1 {
                let a = channels(&s.stations[i]);
                let b = channels(&s.stations[i + 1]);
                assert_eq!(values_at(&s, Some(&c), i, 0.), a);
                assert_eq!(values_at(&s, Some(&c), i, 1.), b);
                for j in 0..=200 {
                    let v = values_at(&s, Some(&c), i, f64::from(j) / 200.);
                    for k in 0..4 {
                        assert!(v[k] >= a[k].min(b[k]) - 1e-12 && v[k] <= a[k].max(b[k]) + 1e-12);
                    }
                    assert!(v[0] > 0. && v[1] > 0.);
                }
            }
        }
    }

    #[test]
    fn first_derivative_is_continuous_in_physical_y_with_uneven_stations() {
        let s = drawing(&[0., 0.08, 0.25, 0.7, 1.], &[0., 0.4, -0.2, -0.15, 0.8]);
        let c = Curves::new(&s).unwrap();
        for i in 1..s.stations.len() - 1 {
            let h0 = f64::from(s.stations[i].at) - f64::from(s.stations[i - 1].at);
            let h1 = f64::from(s.stations[i + 1].at) - f64::from(s.stations[i].at);
            for k in 0..4 {
                let [a, b, d, _] = c.coefficients[i - 1][k];
                let left = (3. * a + 2. * b + d) / h0;
                let right = c.coefficients[i][k][2] / h1;
                assert!((left - right).abs() < 1e-12);
                let step = h0.min(h1) * 1e-6;
                let center = values_at(&s, Some(&c), i, 0.)[k];
                let l = (center - values_at(&s, Some(&c), i - 1, 1. - step / h0)[k]) / step;
                let r = (values_at(&s, Some(&c), i, step / h1)[k] - center) / step;
                assert!((l - r).abs() < 0.0002);
            }
        }
    }
}
