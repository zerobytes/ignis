//! Independent training/validation pilots; no evaluation samples enter fitting.
use super::*;

const TRAIN_STREAM: u64 = 0xa4093822299f31d0;
const VALIDATION_STREAM: u64 = 0x082efa98ec4e6c89;
const EVALUATION_STREAM: u64 = 0x243f6a8885a308d3;
const METHODS: [&str; 4] = ["plain", "fixed", "learned", "gated"];

#[derive(Default)]
struct Moments {
    n: usize,
    mean_f: Rgb,
    mean_g: Rgb,
    m2_f: Rgb,
    m2_g: Rgb,
    covariance: Rgb,
}
impl Moments {
    fn push(&mut self, f: Rgb, g: Rgb) {
        self.n += 1;
        for c in 0..3 {
            let df = f[c] - self.mean_f[c];
            let dg = g[c] - self.mean_g[c];
            self.mean_f[c] += df / self.n as f64;
            self.mean_g[c] += dg / self.n as f64;
            self.m2_f[c] += df * (f[c] - self.mean_f[c]);
            self.m2_g[c] += dg * (g[c] - self.mean_g[c]);
            self.covariance[c] += df * (g[c] - self.mean_g[c]);
        }
    }
    fn alpha(&self) -> f64 {
        let var_g: f64 = self.m2_g.iter().sum();
        if var_g <= 1e-20 {
            return 0.0;
        }
        let alpha = self.covariance.iter().sum::<f64>() / var_g;
        if alpha.is_finite() {
            alpha.clamp(0.0, 2.0)
        } else {
            0.0
        }
    }
    fn variance_ratio(&self, alpha: f64) -> Option<f64> {
        let vf: f64 = self.m2_f.iter().sum();
        if vf <= 1e-20 {
            return None;
        }
        let vg: f64 = self.m2_g.iter().sum();
        let cov: f64 = self.covariance.iter().sum();
        Some((vf - 2.0 * alpha * cov + alpha * alpha * vg).max(0.0) / vf)
    }
}
fn moments(case: usize, cache: &Cache, points: &[[f64; 2]]) -> Moments {
    let mut result = Moments::default();
    for &p in points {
        result.push(f(case, p[0], p[1]), cache.at(p));
    }
    result
}

#[derive(Clone, Copy)]
struct Decision {
    alpha: f64,
    train: usize,
    validation: usize,
    evaluation: usize,
    validation_ratio: f64,
    reason: &'static str,
}
fn decide(
    case: usize,
    cache: &Cache,
    budget: usize,
    method: usize,
    base: u64,
    trial: usize,
    train: &mut [[f64; 2]],
    validation: &mut [[f64; 2]],
) -> Decision {
    let mut result = Decision {
        alpha: 0.0,
        train: 0,
        validation: 0,
        evaluation: budget,
        validation_ratio: 0.0,
        reason: "plain",
    };
    if method == 0 {
        return result;
    }
    if method == 1 {
        result.alpha = 1.0;
        result.reason = "fixed";
        return result;
    }
    // This is an exact property of the complete cache, not a guess from the
    // pilot. A constant control variate cannot reduce variance.
    if cache.constant {
        result.reason = "constant_cache";
        return result;
    }
    fill(train, seed(base, case, trial, TRAIN_STREAM));
    result.train = train.len();
    result.evaluation -= result.train;
    result.alpha = moments(case, cache, train).alpha();
    result.reason = "learned";
    if result.alpha == 0.0 {
        result.reason = "zero_coefficient";
        return result;
    }
    if method == 2 {
        return result;
    }
    fill(validation, seed(base, case, trial, VALIDATION_STREAM));
    result.validation = validation.len();
    result.evaluation -= result.validation;
    let ratio = moments(case, cache, validation).variance_ratio(result.alpha);
    result.validation_ratio = ratio.unwrap_or(0.0);
    let threshold = 0.9 * result.evaluation as f64 / budget as f64;
    if ratio.is_some_and(|r| r < threshold) {
        result.reason = "accepted";
    } else {
        result.alpha = 0.0;
        result.reason = if ratio.is_none() {
            "zero_validation_variance"
        } else {
            "validation_rejected"
        };
    }
    result
}
fn weighted(case: usize, cache: &Cache, points: &[[f64; 2]], alpha: f64) -> Rgb {
    if alpha == 0.0 {
        return estimate(case, 0, cache, points, &[]);
    }
    if alpha == 1.0 {
        return estimate(case, 1, cache, points, &[]);
    }
    let mut total = [0.0; 3];
    for &p in points {
        total = add(total, sub(f(case, p[0], p[1]), scale(cache.at(p), alpha)));
    }
    add(
        scale(cache.integral, alpha),
        scale(total, 1.0 / points.len() as f64),
    )
}

pub fn verify() {
    let mut m = Moments::default();
    for i in 0..64 {
        let x = (i as f64 + 0.5) / 64.0;
        m.push(
            [0.1 + 0.05 * x, 0.2 + 0.02 * x, 0.3 + 0.03 * x],
            [0.1 + 5.0 * x, 0.2 + 2.0 * x, 0.3 + 3.0 * x],
        );
    }
    assert!(
        (m.alpha() - 0.01).abs() < 1e-12,
        "centered pilot must recover coefficient scale"
    );
    let cache = Cache::new(0, 8);
    let mut points = vec![[0.0; 2]; 64];
    fill(&mut points, 77);
    assert_eq!(
        weighted(0, &cache, &points, 0.0),
        estimate(0, 0, &cache, &points, &[])
    );
    assert_eq!(
        weighted(0, &cache, &points, 1.0),
        estimate(0, 1, &cache, &points, &[])
    );
    let mut train = vec![[0.0; 2]; 8];
    let mut validation = train.clone();
    for case in 0..6 {
        let cache = Cache::new(case, 8);
        for method in 0..4 {
            let d = decide(case, &cache, 64, method, 5, 7, &mut train, &mut validation);
            assert_eq!(d.train + d.validation + d.evaluation, 64);
            assert!((0.0..=2.0).contains(&d.alpha) && d.evaluation > 0);
            if case == 1 && method >= 2 {
                assert_eq!(d.reason, "constant_cache");
                assert_eq!((d.train, d.validation, d.evaluation), (0, 0, 64));
            }
        }
    }
    // Given the same pilot, changing final Y affects neither decision nor a.
    let changed = Cache::new(5, 8);
    let d = decide(5, &changed, 64, 3, 5, 7, &mut train, &mut validation);
    let _ = weighted(5, &changed, &points, d.alpha);
    fill(&mut points, 88);
    let repeated = decide(5, &changed, 64, 3, 5, 7, &mut train, &mut validation);
    assert_eq!((d.alpha, d.reason), (repeated.alpha, repeated.reason));
    let train_seed = seed(5, 0, 7, TRAIN_STREAM);
    assert_ne!(train_seed, seed(5, 0, 7, VALIDATION_STREAM));
    assert_ne!(train_seed, seed(5, 0, 7, EVALUATION_STREAM));
    println!(
        "verified: pilot scale, zero/unit coefficient, budget/fallback, frozen decision, separate streams"
    );
}

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() != 7 {
        return Err("pilot: output trials seed budgets grids pilot-size timing-blocks".into());
    }
    let out = Path::new(&args[0]);
    let trials: usize = args[1].parse()?;
    let base: u64 = args[2].parse()?;
    let budgets = list(&args[3], 4096);
    let grids = list(&args[4], 64);
    let requested: usize = args[5].parse()?;
    let blocks: usize = args[6].parse()?;
    assert!((32..=2048).contains(&trials) && budgets[0] >= 16);
    assert!((4..=256).contains(&requested) && (5..=31).contains(&blocks));
    assert!(trials * budgets.iter().sum::<usize>() * grids.len() * 6 <= 100_000_000);
    let mut raw = String::from(
        "case,grid,budget,trial,method,alpha,train,validation,evaluation,validation_ratio,reason,r,g,b\n",
    );
    let mut timings = String::from("case,grid,budget,method,phase,block,ns_per_estimate,loops\n");
    for (case, name) in CASES.iter().enumerate() {
        for &grid in &grids {
            let cache = Cache::new(case, grid);
            for &budget in &budgets {
                let p = requested.min(budget / 4);
                let mut train = vec![[0.0; 2]; p];
                let mut validation = train.clone();
                let mut y = vec![[0.0; 2]; budget];
                for trial in 0..trials {
                    fill(&mut y, seed(base, case, trial, EVALUATION_STREAM));
                    for (method, method_name) in METHODS.iter().enumerate() {
                        let d = decide(
                            case,
                            &cache,
                            budget,
                            method,
                            base,
                            trial,
                            &mut train,
                            &mut validation,
                        );
                        assert_eq!(d.train + d.validation + d.evaluation, budget);
                        let value = weighted(case, &cache, &y[..d.evaluation], d.alpha);
                        write!(
                            raw,
                            "{name},{grid},{budget},{trial},{method_name},{:.17e},{},{},{},{:.17e},{}",
                            d.alpha,
                            d.train,
                            d.validation,
                            d.evaluation,
                            d.validation_ratio,
                            d.reason
                        )?;
                        rgb_csv(&mut raw, value);
                        raw.push('\n');
                    }
                }
                for offset in 0..4 {
                    let method = (offset + case + grid + budget) % 4;
                    let method_name = METHODS[method];
                    for phase in ["decision", "total"] {
                        let mut trial = 0usize;
                        let measured = benchmark(
                            || {
                                let d = decide(
                                    black_box(case),
                                    black_box(&cache),
                                    budget,
                                    black_box(method),
                                    base,
                                    trial,
                                    &mut train,
                                    &mut validation,
                                );
                                trial += 1;
                                if phase == "decision" {
                                    black_box([d.alpha, d.evaluation as f64, d.validation_ratio])
                                } else {
                                    fill(
                                        &mut y[..d.evaluation],
                                        seed(base, case, trial - 1, EVALUATION_STREAM),
                                    );
                                    weighted(
                                        black_box(case),
                                        black_box(&cache),
                                        black_box(&y[..d.evaluation]),
                                        d.alpha,
                                    )
                                }
                            },
                            blocks,
                        );
                        for (block, (ns, loops)) in measured.into_iter().enumerate() {
                            writeln!(
                                timings,
                                "{name},{grid},{budget},{method_name},{phase},{block},{ns:.6},{loops}"
                            )?;
                        }
                    }
                }
            }
        }
    }
    fs::write(out.join("pilot_trials.csv"), raw)?;
    fs::write(out.join("pilot_timings.csv"), timings)?;
    Ok(())
}
