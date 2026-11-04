//! Shared coefficient lifetime. Evaluation never feeds control state.
use super::*;

const SCENARIOS: [&str; 4] = ["stable", "geometry_step", "geometry_pulse", "lighting_step"];
const POLICIES: [&str; 5] = ["plain", "per_query", "periodic", "notified", "watched"];
const STRIDE: usize = 4;
#[derive(Clone, Copy)]
struct Target {
    case: usize,
    scale: f64,
    revision: usize,
}
impl Target {
    fn value(self, p: [f64; 2]) -> Rgb {
        scale(f(self.case, p[0], p[1]), self.scale)
    }
}
fn target(scenario: usize, t: usize) -> Target {
    let changed = scenario > 0 && t >= 17 && (scenario != 2 || t < 29);
    Target {
        case: if scenario == 3 {
            3
        } else if changed {
            5
        } else {
            4
        },
        scale: if scenario == 3 && changed { 0.1 } else { 1.0 },
        revision: if scenario == 2 && t >= 29 {
            2
        } else {
            usize::from(changed)
        },
    }
}
fn point_seed(base: u64, scenario: usize, trial: usize, t: usize, stream: u64) -> u64 {
    seed(base, scenario + 10, trial * 256 + t, stream)
}
fn differs(a: Rgb, b: Rgb) -> bool {
    (0..3).any(|c| (a[c] - b[c]).abs() > 1e-9 * a[c].abs().max(b[c].abs()).max(1.0))
}
#[derive(Clone, Copy, Default)]
struct Query {
    value: Rgb,
    alpha: f64,
    train: usize,
    checks: usize,
    snapshots: usize,
    evaluations: usize,
    reason: usize,
    trained_revision: usize,
}
struct Scratch {
    train: Vec<[f64; 2]>,
    watch: Vec<[f64; 2]>,
    watch_values: Vec<Rgb>,
    y: Vec<[f64; 2]>,
    queries: Vec<Query>,
}
impl Scratch {
    fn new(budget: usize, pilot: usize, steps: usize) -> Self {
        let watchers = 8.min(budget / 8);
        Self {
            train: vec![[0.; 2]; pilot.min(budget / 4)],
            watch: vec![[0.; 2]; watchers],
            watch_values: vec![[0.; 3]; watchers],
            y: vec![[0.; 2]; budget],
            queries: vec![Query::default(); steps],
        }
    }
}
fn apply(t: Target, cache: &Cache, y: &[[f64; 2]], alpha: f64) -> Rgb {
    if t.scale == 1.0 {
        return pilot::weighted(t.case, cache, y, alpha);
    }
    let mut sum = [0.; 3];
    for &p in y {
        let value = t.value(p);
        sum = add(
            sum,
            if alpha == 0. {
                value
            } else {
                sub(value, scale(cache.at(p), alpha))
            },
        );
    }
    add(
        scale(cache.integral, alpha),
        scale(sum, 1. / y.len() as f64),
    )
}
// Policies 0/1/2/4 do not consult scene revision for control. It is recorded
// after the decision solely to evaluate response latency/staleness.
fn sequence(
    scenario: usize,
    policy: usize,
    base: u64,
    trial: usize,
    budget: usize,
    horizon: usize,
    cache: &Cache,
    s: &mut Scratch,
) -> Rgb {
    let mut last_refresh = None;
    let mut revision = 0usize;
    let mut alpha = 0.;
    let mut checksum = [0.; 3];
    for step in 0..s.queries.len() {
        let current = target(scenario, step);
        let mut q = Query::default();
        if policy != 0 && !cache.constant {
            let mut reason = if last_refresh.is_none() {
                1
            } else if policy == 1 {
                2
            } else if step - last_refresh.unwrap() >= horizon {
                3
            } else if policy == 3 && revision != current.revision {
                4
            } else {
                0
            };
            if reason == 0 && policy == 4 && horizon > STRIDE && step % STRIDE == 0 {
                for (p, old) in s.watch.iter().zip(&s.watch_values) {
                    q.checks += 1;
                    if differs(current.value(*p), *old) {
                        reason = 5;
                        break;
                    }
                }
            }
            if reason != 0 {
                fill(
                    &mut s.train,
                    point_seed(base, scenario, trial, step, 0x452821e638d01377),
                );
                let mut moments = pilot::Moments::default();
                for &p in &s.train {
                    moments.push(current.value(p), cache.at(p));
                }
                alpha = moments.alpha();
                q.train = s.train.len();
                if policy == 4 && horizon > STRIDE {
                    fill(
                        &mut s.watch,
                        point_seed(base, scenario, trial, step, 0xbe5466cf34e90c6c),
                    );
                    for (p, value) in s.watch.iter().zip(&mut s.watch_values) {
                        *value = current.value(*p);
                    }
                    q.snapshots = s.watch.len();
                }
                last_refresh = Some(step);
                revision = current.revision;
                q.reason = reason;
            }
        }
        q.alpha = alpha;
        q.trained_revision = revision;
        q.evaluations = budget - q.train - q.checks - q.snapshots;
        assert!(q.evaluations > 0);
        fill(
            &mut s.y[..q.evaluations],
            point_seed(base, scenario, trial, step, 0xc0ac29b7c97c50dd),
        );
        q.value = apply(current, cache, &s.y[..q.evaluations], alpha);
        checksum = add(checksum, q.value);
        s.queries[step] = q;
    }
    black_box(&s.queries);
    checksum
}

pub fn verify() {
    let cache = Cache::new(4, 8);
    let mut a = Scratch::new(64, 32, 48);
    let mut b = Scratch::new(64, 32, 48);
    for scenario in 0..4 {
        let cache = Cache::new(target(scenario, 0).case, 8);
        sequence(scenario, 1, 7, 2, 64, 1, &cache, &mut a);
        sequence(scenario, 2, 7, 2, 64, 1, &cache, &mut b);
        for (x, y) in a.queries.iter().zip(&b.queries) {
            assert_eq!(x.value, y.value);
        }
        for policy in 0..5 {
            sequence(scenario, policy, 7, 2, 64, 32, &cache, &mut a);
            for q in &a.queries {
                assert_eq!(q.train + q.checks + q.snapshots + q.evaluations, 64);
            }
            if policy == 3 && scenario > 0 {
                assert!(a.queries[17].reason > 0 && a.queries[17].trained_revision == 1);
                if scenario == 2 {
                    assert!(a.queries[29].reason > 0 && a.queries[29].trained_revision == 2);
                }
            }
            if scenario == 0 {
                assert!(!a.queries.iter().any(|q| q.reason == 5));
            }
        }
    }
    // TTL32 never revalidates the pulse while it exists (queries17..28).
    sequence(2, 2, 7, 2, 64, 32, &cache, &mut a);
    assert!(a.queries[17..29].iter().all(|q| q.reason == 0));
    assert_eq!(latency(2, 2, &a.queries, 1), (-1, 1));
    let before = target(3, 16).value([0.2, 0.3]);
    let after = target(3, 17).value([0.2, 0.3]);
    assert_eq!(scale(before, 0.1), after);
    // Paired probes have a real blind spot: an occluded direction can remain
    // identical while other directions change. Never certify the whole field.
    assert_eq!(f(4, 0., 0.), f(5, 0., 0.));
    assert!((0..64).any(|i| differs(
        f(4, (i % 8) as f64 / 8., (i / 8) as f64 / 8.),
        f(5, (i % 8) as f64 / 8., (i / 8) as f64 / 8.)
    )));
    assert_ne!(
        point_seed(1, 0, 0, 0, 0x452821e638d01377),
        point_seed(1, 0, 0, 0, 0xc0ac29b7c97c50dd)
    );
    println!(
        "verified: epoch budgets, H=1, notified changes, transient misses, light scaling, sentinel limits"
    );
}

fn references(out: &Path) -> Result<[Rgb; 6], Box<dyn std::error::Error>> {
    let mut result = [[0.; 3]; 6];
    let mut levels = [0; 6];
    for line in fs::read_to_string(out.join("references.csv"))?
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = line.split(',').collect();
        let case = CASES
            .iter()
            .position(|&n| n == fields[0])
            .ok_or("unknown reference")?;
        let grid: usize = fields[1].parse()?;
        if grid >= levels[case] {
            levels[case] = grid;
            result[case] = [fields[2].parse()?, fields[3].parse()?, fields[4].parse()?];
        }
    }
    assert!((3..6).all(|c| levels[c] > 0));
    Ok(result)
}
fn latency(scenario: usize, policy: usize, queries: &[Query], event: usize) -> (i32, usize) {
    if scenario == 0 || policy == 0 || (event == 2 && scenario != 2) {
        return (-1, 0);
    }
    let (start, end) = if event == 1 {
        (17, if scenario == 2 { 29 } else { queries.len() })
    } else {
        (29, queries.len())
    };
    for (t, q) in queries.iter().enumerate().take(end).skip(start) {
        if q.reason > 0 && q.trained_revision == event {
            return ((t - start) as i32, 0);
        }
    }
    (-1, 1)
}

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() != 10 {
        return Err(
            "epoch: output trials seed budgets horizons grid steps pilot blocks trace-trials"
                .into(),
        );
    }
    let out = Path::new(&args[0]);
    let trials: usize = args[1].parse()?;
    let base: u64 = args[2].parse()?;
    let budgets = list(&args[3], 4096);
    let horizons = list(&args[4], 128);
    let grid: usize = args[5].parse()?;
    let steps: usize = args[6].parse()?;
    let pilot: usize = args[7].parse()?;
    let blocks: usize = args[8].parse()?;
    let trace_trials: usize = args[9].parse()?;
    assert!((32..=512).contains(&trials) && budgets[0] >= 16 && (1..=64).contains(&grid));
    assert!(
        (32..=128).contains(&steps) && (4..=256).contains(&pilot) && (5..=31).contains(&blocks)
    );
    assert!(
        trace_trials <= 8
            && trials * budgets.iter().sum::<usize>() * horizons.len() * 4 * 5 * steps
                <= 150_000_000
    );
    let refs = references(out)?;
    let mut raw = String::from(
        "scenario,budget,horizon,trial,method,mse_r,mse_g,mse_b,error_r,error_g,error_b,early_mse,middle_mse,late_mse,train_calls,check_calls,snapshot_calls,evaluation_calls,refreshes,stale_queries,event1_latency,event1_missed,event2_latency,event2_missed\n",
    );
    let mut curves = String::from(
        "scenario,budget,horizon,method,step,mse_r,mse_g,mse_b,mean_alpha,train_calls,check_calls,snapshot_calls,evaluation_calls,refresh_rate\n",
    );
    let mut trace = String::from(
        "scenario,budget,horizon,trial,method,step,target_revision,trained_revision,alpha,reason,train,checks,snapshots,evaluations,r,g,b\n",
    );
    let mut timings = String::from("scenario,budget,horizon,method,block,ns_per_sequence,loops\n");
    for (scenario, name) in SCENARIOS.iter().enumerate() {
        let cache = Cache::new(target(scenario, 0).case, grid);
        for &budget in &budgets {
            for &horizon in &horizons {
                let mut scratch = Scratch::new(budget, pilot, steps);
                for (policy, method) in POLICIES.iter().enumerate() {
                    let mut curve = vec![[0.; 9]; steps];
                    for trial in 0..trials {
                        sequence(
                            scenario,
                            policy,
                            base,
                            trial,
                            budget,
                            horizon,
                            &cache,
                            &mut scratch,
                        );
                        let mut mse = [0.; 3];
                        let mut errors = [0.; 3];
                        let mut windows = [0.; 3];
                        let mut window_counts = [0usize; 3];
                        let mut calls = [0usize; 4];
                        let mut refreshes = 0;
                        let mut stale = 0;
                        for (t, q) in scratch.queries.iter().enumerate() {
                            let target = target(scenario, t);
                            let error = sub(q.value, scale(refs[target.case], target.scale));
                            let squared = error.map(|v| v * v);
                            mse = add(mse, squared);
                            errors = add(errors, error);
                            let window = if t < 17 {
                                0
                            } else if t < 29 {
                                1
                            } else {
                                2
                            };
                            windows[window] += squared.iter().sum::<f64>() / 3.;
                            window_counts[window] += 1;
                            calls[0] += q.train;
                            calls[1] += q.checks;
                            calls[2] += q.snapshots;
                            calls[3] += q.evaluations;
                            refreshes += usize::from(q.reason > 0);
                            stale +=
                                usize::from(policy != 0 && q.trained_revision != target.revision);
                            for (c, v) in squared.iter().enumerate() {
                                curve[t][c] += v / trials as f64;
                            }
                            let extra = [
                                q.alpha,
                                q.train as f64,
                                q.checks as f64,
                                q.snapshots as f64,
                                q.evaluations as f64,
                                f64::from(q.reason > 0),
                            ];
                            for (i, v) in extra.iter().enumerate() {
                                curve[t][i + 3] += v / trials as f64;
                            }
                            if trial < trace_trials
                                && budget == *budgets.last().unwrap()
                                && horizon == *horizons.last().unwrap()
                            {
                                write!(
                                    trace,
                                    "{name},{budget},{horizon},{trial},{method},{t},{},{},{:.17e},{},{},{},{},{}",
                                    target.revision,
                                    q.trained_revision,
                                    q.alpha,
                                    q.reason,
                                    q.train,
                                    q.checks,
                                    q.snapshots,
                                    q.evaluations
                                )?;
                                rgb_csv(&mut trace, q.value);
                                trace.push('\n');
                            }
                        }
                        assert_eq!(calls.iter().sum::<usize>(), budget * steps);
                        let (lag1, miss1) = latency(scenario, policy, &scratch.queries, 1);
                        let (lag2, miss2) = latency(scenario, policy, &scratch.queries, 2);
                        write!(raw, "{name},{budget},{horizon},{trial},{method}")?;
                        rgb_csv(&mut raw, scale(mse, 1. / steps as f64));
                        rgb_csv(&mut raw, scale(errors, 1. / steps as f64));
                        writeln!(
                            raw,
                            ",{:.17e},{:.17e},{:.17e},{},{},{},{},{},{},{lag1},{miss1},{lag2},{miss2}",
                            windows[0] / window_counts[0] as f64,
                            windows[1] / window_counts[1] as f64,
                            windows[2] / window_counts[2] as f64,
                            calls[0],
                            calls[1],
                            calls[2],
                            calls[3],
                            refreshes,
                            stale
                        )?;
                    }
                    for (t, row) in curve.iter().enumerate() {
                        write!(curves, "{name},{budget},{horizon},{method},{t}")?;
                        for v in row.iter().take(9) {
                            write!(curves, ",{v:.17e}")?;
                        }
                        curves.push('\n');
                    }
                    let mut trial = 0usize;
                    let measured = benchmark(
                        || {
                            let value = sequence(
                                black_box(scenario),
                                black_box(policy),
                                base,
                                trial,
                                budget,
                                horizon,
                                black_box(&cache),
                                &mut scratch,
                            );
                            trial += 1;
                            black_box(value)
                        },
                        blocks,
                    );
                    for (block, (ns, loops)) in measured.into_iter().enumerate() {
                        writeln!(
                            timings,
                            "{name},{budget},{horizon},{method},{block},{ns:.6},{loops}"
                        )?;
                    }
                }
            }
        }
    }
    fs::write(out.join("epoch_trials.csv"), raw)?;
    fs::write(out.join("epoch_curves.csv"), curves)?;
    fs::write(out.join("epoch_trace.csv"), trace)?;
    fs::write(out.join("epoch_timings.csv"), timings)?;
    Ok(())
}
