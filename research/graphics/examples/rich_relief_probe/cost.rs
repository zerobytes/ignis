//! Offline/startup fit only. Runtime sees three work coefficients, never poses.
use anyhow::{Result, ensure};
use serde::Serialize;
#[derive(Serialize)]
pub struct Sample {
    pub saved_million: f64,
    pub extra_draws: f64,
    pub gain_ms: f64,
    pub noise_ms: f64,
}
#[derive(Serialize)]
pub struct Model {
    pub coefficients: [f64; 3],
    pub margin_ms: f64,
}
impl Model {
    pub fn fit(samples: &[Sample]) -> Result<Self> {
        ensure!(!samples.is_empty(), "empty calibration");
        ensure!(
            samples
                .iter()
                .all(|s| [s.saved_million, s.extra_draws, s.gain_ms, s.noise_ms]
                    .iter()
                    .all(|x| x.is_finite())
                    && s.noise_ms >= 0.
                    && s.extra_draws >= 0.),
            "invalid calibration"
        );
        let mut c = [0.; 3];
        // Nonnegative coordinate descent; increasing draws cannot improve cost.
        for _ in 0..2048 {
            for j in 0..3 {
                let (mut numerator, mut denominator) = (0., 0.);
                for s in samples {
                    let x = [s.saved_million, -s.extra_draws, -1.];
                    let residual =
                        s.gain_ms - (0..3).filter(|&k| k != j).map(|k| c[k] * x[k]).sum::<f64>();
                    numerator += x[j] * residual;
                    denominator += x[j] * x[j];
                }
                c[j] = if denominator > 0. {
                    (numerator / denominator).max(0.)
                } else {
                    0.
                };
            }
        }
        let mut model = Self {
            coefficients: c,
            margin_ms: 0.,
        };
        model.margin_ms = samples
            .iter()
            .map(|s| (model.gain(s.saved_million, s.extra_draws) - s.gain_ms).max(0.) + s.noise_ms)
            .fold(0., f64::max);
        model.validate()?;
        Ok(model)
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            self.coefficients
                .iter()
                .chain([&self.margin_ms])
                .all(|x| x.is_finite() && *x >= 0.),
            "invalid cost model"
        );
        Ok(())
    }
    pub fn gain(&self, saved_million: f64, extra_draws: f64) -> f64 {
        self.coefficients[0] * saved_million
            - self.coefficients[1] * extra_draws
            - self.coefficients[2]
    }
    pub fn choose(&self, previous: u32, local: u32, draws: usize) -> bool {
        local < previous
            && self.gain(
                (previous as f64 - local as f64) / 1e6,
                draws.saturating_sub(1) as f64,
            ) > self.margin_ms
    }
}
/// Lower triangle count bound for the existing projected-error selector.
/// Construct once from compiled certificates, never from measured validation poses.
pub struct Frontier {
    steps: Vec<(f32, u32)>,
    radius: f32,
}
impl Frontier {
    pub fn new(asset: &super::compile::Asset) -> Self {
        let mut counts = asset
            .patches
            .iter()
            .map(|p| p.levels[0].count / 3)
            .collect::<Vec<_>>();
        let mut events = asset
            .patches
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                p.levels
                    .iter()
                    .skip(1)
                    .map(move |l| (l.error, i, l.count / 3))
            })
            .collect::<Vec<_>>();
        events.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut total = counts.iter().sum();
        let mut steps = vec![(0., total)];
        for (error, i, count) in events {
            if count < counts[i] {
                total -= counts[i] - count;
                counts[i] = count;
                steps.push((error, total));
            }
        }
        let radius = asset
            .patches
            .iter()
            .map(|p| {
                glam::Vec3::from_array(p.low)
                    .abs()
                    .max(glam::Vec3::from_array(p.high).abs())
                    .length()
            })
            .fold(0., f32::max);
        Self { steps, radius }
    }
    pub fn minimum(&self, eye_distance: f32, height: u32, budget: f32) -> u32 {
        let focal = height as f32 / (2. * (std::f32::consts::FRAC_PI_4 / 2.).tan());
        // Far depth bounds every patch corner. Ignore the radius numerator:
        // admissible actual error implies error <= budget*far/(focal+budget).
        // Expand by roundoff to keep this an optimistic pruning bound.
        let threshold = (budget * (eye_distance + self.radius) / (focal + budget)) * 1.00001 + 1e-9;
        let end = self.steps.partition_point(|(e, _)| *e <= threshold);
        self.steps[end.saturating_sub(1)].1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn underestimated_gain_does_not_create_a_false_regression_reserve() -> Result<()> {
        // A benefit the simple work model cannot explain must not penalize
        // other predicted benefits. Overestimates and measurement noise still do.
        let samples = [(0.01, 0.2), (1., 0.3), (2., 0.6)].map(|(saved_million, gain_ms)| Sample {
            saved_million,
            gain_ms,
            extra_draws: 0.,
            noise_ms: 0.01,
        });
        let m = Model::fit(&samples)?;
        assert!(m.margin_ms >= 0.01);
        assert!(m.choose(1_000_000, 800_000, 1));
        assert!(!m.choose(1_000_000, 999_000, 1));
        Ok(())
    }
    #[test]
    fn calibration_recovers_cost_and_is_conservative_under_noise() -> Result<()> {
        let samples = [(0.1, 3.), (0.4, 2.), (0.8, 0.), (0.2, 18.)].map(|(t, d)| Sample {
            saved_million: t,
            extra_draws: d,
            gain_ms: 0.3 * t - 0.002 * d - 0.005,
            noise_ms: 0.001,
        });
        let m = Model::fit(&samples)?;
        for s in samples {
            assert!((m.gain(s.saved_million, s.extra_draws) - s.gain_ms).abs() < 1e-8);
        }
        assert!(m.choose(1_000_000, 400_000, 1));
        assert!(!m.choose(1_000_000, 999_000, 32));
        assert!(!m.choose(1_000_000, 1_000_000, 1));
        let mut last = f64::INFINITY;
        for d in 0..100 {
            let g = m.gain(0.3, d as f64);
            assert!(g <= last);
            last = g;
        }
        assert!(Model::fit(&[]).is_err());
        assert!(
            Model::fit(&[Sample {
                saved_million: f64::NAN,
                extra_draws: 0.,
                gain_ms: 0.,
                noise_ms: 0.
            }])
            .is_err()
        );
        Ok(())
    }
}
