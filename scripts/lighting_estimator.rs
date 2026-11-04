//! Standalone CPU estimator laboratory. Compile with rustc, without the engine.
use std::{fmt::Write as _, fs, hint::black_box, path::Path, time::Instant};

type Rgb = [f64; 3];
#[path = "lighting_estimator_pilot.rs"]
mod pilot;
const CASES: [&str; 6] = [
    "smooth_polynomial",
    "missed_emitter",
    "stale_analytic",
    "hemisphere_open",
    "hemisphere_occluded",
    "hemisphere_changed",
];
const ALBEDO: Rgb = [0.7, 0.55, 0.35];

fn add(a: Rgb, b: Rgb) -> Rgb {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Rgb, b: Rgb) -> Rgb {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: Rgb, s: f64) -> Rgb {
    a.map(|v| v * s)
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((z ^ (z >> 31)) >> 11) as f64 * (1.0 / ((1u64 << 53) as f64))
    }
}
fn seed(base: u64, case: usize, trial: usize, stream: u64) -> u64 {
    base ^ (case as u64).wrapping_mul(0xd6e8feb86659fd93)
        ^ (trial as u64).wrapping_mul(0xa0761d6478bd642f)
        ^ stream
}
fn fill(points: &mut [[f64; 2]], seed: u64) {
    let mut rng = Rng(seed);
    for p in points {
        *p = [rng.next(), rng.next()];
    }
}

fn direction(u: f64, v: f64) -> Rgb {
    let phi = std::f64::consts::TAU * v;
    let r = u.sqrt();
    [r * phi.cos(), (1.0 - u).sqrt(), r * phi.sin()]
}
fn hits_box(d: Rgb, lo: Rgb, hi: Rgb) -> bool {
    let mut near: f64 = 0.0;
    let mut far = f64::INFINITY;
    for i in 0..3 {
        if d[i].abs() < 1e-15 {
            if lo[i] > 0.0 || hi[i] < 0.0 {
                return false;
            }
        } else {
            let a = lo[i] / d[i];
            let b = hi[i] / d[i];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
        }
    }
    far >= near && far > 0.0
}
fn transport(case: usize, u: f64, v: f64) -> Rgb {
    let d = direction(u, v);
    let shift = if case == 5 { 0.65 } else { 0.0 };
    let blocked = case != 3
        && (hits_box(d, [-0.9 + shift, 0.65, -0.65], [0.15 + shift, 1.1, 0.35])
            || hits_box(d, [0.25, 0.35, -1.0], [0.35, 1.6, -0.2])
            || hits_box(d, [-0.4, 0.2, 0.8], [0.8, 0.5, 1.0]));
    if blocked {
        return [0.0; 3];
    }
    let l = [0.6, 0.7, -(0.15f64).sqrt()];
    let sun = (0..3).map(|i| l[i] * d[i]).sum::<f64>().max(0.0).powi(48);
    let sky = [0.2 + 0.7 * d[1], 0.3 + 0.8 * d[1], 0.5 + d[1]];
    let bright = [4.0, 1.5, 0.4];
    std::array::from_fn(|i| ALBEDO[i] * (sky[i] + bright[i] * sun))
}
fn f(case: usize, u: f64, v: f64) -> Rgb {
    match case {
        0 => [
            0.1 + 0.8 * u * u + 0.3 * v * v,
            0.2 + 0.7 * u * v + 0.25 * u * u,
            0.3 + 0.3 * u + 0.4 * v * v * v,
        ],
        1 => {
            let hit = (0.501..0.503).contains(&u) && (0.101..0.301).contains(&v);
            add(
                [0.1, 0.08, 0.04],
                scale([120.0, 45.0, 10.0], f64::from(hit)),
            )
        }
        2 => [0.1 + 0.05 * u, 0.08 + 0.04 * v, 0.04 + 0.03 * u],
        3..=5 => transport(case, u, v),
        _ => unreachable!(),
    }
}
fn cache_source(case: usize, u: f64, v: f64) -> Rgb {
    match case {
        2 => [0.1 + 5.0 * u, 0.08 + 4.0 * v, 0.04 + 3.0 * u],
        5 => transport(4, u, v),
        _ => f(case, u, v),
    }
}
fn analytic(case: usize) -> Option<Rgb> {
    match case {
        0 => Some([0.1 + 1.1 / 3.0, 0.2 + 0.7 / 4.0 + 0.25 / 3.0, 0.55]),
        1 => Some(add([0.1, 0.08, 0.04], scale([120.0, 45.0, 10.0], 0.0004))),
        2 => Some([0.125, 0.1, 0.055]),
        _ => None,
    }
}

struct Cache {
    side: usize,
    values: Vec<Rgb>,
    integral: Rgb,
    constant: bool,
}
impl Cache {
    fn new(case: usize, side: usize) -> Self {
        let mut values = Vec::with_capacity(side * side);
        let mut total = [0.0; 3];
        let mut constant = true;
        for y in 0..side {
            for x in 0..side {
                let value = cache_source(
                    case,
                    (x as f64 + 0.5) / side as f64,
                    (y as f64 + 0.5) / side as f64,
                );
                if let Some(first) = values.first() {
                    constant &= *first == value;
                }
                values.push(value);
                total = add(total, value);
            }
        }
        Self {
            side,
            values,
            integral: scale(total, 1.0 / (side * side) as f64),
            constant,
        }
    }
    fn at(&self, p: [f64; 2]) -> Rgb {
        self.values
            [(p[1] * self.side as f64) as usize * self.side + (p[0] * self.side as f64) as usize]
    }
}
fn midpoint(case: usize, side: usize) -> Rgb {
    let mut total = [0.0; 3];
    for y in 0..side {
        for x in 0..side {
            total = add(
                total,
                f(
                    case,
                    (x as f64 + 0.5) / side as f64,
                    (y as f64 + 0.5) / side as f64,
                ),
            );
        }
    }
    scale(total, 1.0 / (side * side) as f64)
}
// The X and Y batches are independent; plain and corrected methods share Y
// for paired comparisons. No fitted weights or clamps use evaluation samples.
fn estimate(case: usize, method: usize, cache: &Cache, y: &[[f64; 2]], x: &[[f64; 2]]) -> Rgb {
    let mut total = [0.0; 3];
    for &p in y {
        let value = f(case, p[0], p[1]);
        total = add(
            total,
            if method == 0 {
                value
            } else {
                sub(value, cache.at(p))
            },
        );
    }
    let residual = scale(total, 1.0 / y.len() as f64);
    if method == 0 {
        return residual;
    }
    let g = if method == 1 {
        cache.integral
    } else {
        let mut sum = [0.0; 3];
        for &p in x {
            sum = add(sum, cache.at(p));
        }
        scale(sum, 1.0 / x.len() as f64)
    };
    add(g, residual)
}

fn rgb_csv(out: &mut String, a: Rgb) {
    for v in a {
        write!(out, ",{v:.17e}").unwrap();
    }
}
fn benchmark(mut work: impl FnMut() -> Rgb, blocks: usize) -> Vec<(f64, usize)> {
    for _ in 0..4 {
        black_box(work());
    }
    let mut loops = 1usize;
    loop {
        let start = Instant::now();
        for _ in 0..loops {
            black_box(work());
        }
        if start.elapsed().as_secs_f64() >= 0.001 || loops >= 65536 {
            break;
        }
        loops *= 2;
    }
    (0..blocks)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..loops {
                black_box(work());
            }
            (start.elapsed().as_secs_f64() * 1e9 / loops as f64, loops)
        })
        .collect()
}

fn verify() {
    let mut a = Rng(1);
    let mut b = Rng(1);
    for _ in 0..1000 {
        let x = a.next();
        assert_eq!(x, b.next());
        assert!((0.0..1.0).contains(&x));
    }
    assert!(hits_box(
        [0.0, 1.0, 0.0],
        [-1.0, 1.0, -1.0],
        [1.0, 2.0, 1.0]
    ));
    assert!(!hits_box(
        [0.0, 1.0, 0.0],
        [1.0, 1.0, -1.0],
        [2.0, 2.0, 1.0]
    ));
    assert!(!hits_box(
        [0.0, -1.0, 0.0],
        [-1.0, 1.0, -1.0],
        [1.0, 2.0, 1.0]
    ));
    // Cosine hemisphere/PDF normalization: E[omega_y] = 2/3, unit albedo,
    // constant environment produces exactly unit outgoing radiance.
    let mut cos_mean = 0.0;
    for i in 0..10000 {
        let d = direction((i as f64 + 0.5) / 10000.0, 0.37);
        assert!((d.iter().map(|x| x * x).sum::<f64>() - 1.0).abs() < 1e-12);
        cos_mean += d[1] / 10000.0;
    }
    assert!((cos_mean - 2.0 / 3.0).abs() < 1e-6);
    // Integrate every cache cell with equal area. The approximation integral
    // must equal the stored exact G, including the stale cache.
    for case in 0..6 {
        let cache = Cache::new(case, 8);
        let mut sum = [0.0; 3];
        for y in 0..8 {
            for x in 0..8 {
                sum = add(
                    sum,
                    cache.at([(x as f64 + 0.5) / 8.0, (y as f64 + 0.5) / 8.0]),
                );
            }
        }
        assert_eq!(scale(sum, 1.0 / 64.0), cache.integral);
    }
    // Constant missed cache must not fabricate a bright feature or suppress
    // a negative correction. A stale predictor remains an unbiased but poor CV.
    let cache = Cache::new(1, 8);
    assert!(
        sub(cache.integral, [0.1, 0.08, 0.04])
            .iter()
            .all(|x| x.abs() < 1e-14)
    );
    assert_eq!(f(1, 0.502, 0.2), [120.1, 45.08, 10.04]);
    let points: Vec<_> = (0..64)
        .map(|i| [(i as f64 + 0.5) / 64.0, (i as f64 + 0.5) / 64.0])
        .collect();
    let stale = Cache::new(2, 8);
    let result = estimate(2, 1, &stale, &points, &[]);
    assert!(
        sub(result, analytic(2).unwrap())
            .iter()
            .all(|x| x.abs() < 1e-12)
    );
    // Subtracting g and adding its *own* integral gives the same expectation
    // even for an intentionally incorrect predictor (no ground truth leakage).
    assert!(stale.integral[0] > analytic(2).unwrap()[0] * 10.0);
    println!(
        "verified: RNG/domain, box visibility, cosine map, cache measure, missed feature, signed stale residual"
    );
}

fn list(text: &str, max: usize) -> Vec<usize> {
    let values: Vec<usize> = text
        .split(',')
        .map(|s| s.parse().expect("integer list"))
        .collect();
    assert!(!values.is_empty() && values.len() <= 8);
    assert!(values.iter().all(|&v| v > 0 && v <= max));
    assert!(values.windows(2).all(|p| p[0] < p[1]));
    values
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--verify") {
        verify();
        pilot::verify();
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("--pilot") {
        return pilot::run(&args[2..]);
    }
    if args.len() != 8 {
        return Err(
            "expected: output trials seed sample-list cache-list reference-grid timing-blocks"
                .into(),
        );
    }
    let out = Path::new(&args[1]);
    let trials: usize = args[2].parse()?;
    let base_seed: u64 = args[3].parse()?;
    let counts = list(&args[4], 4096);
    let grids = list(&args[5], 64);
    let reference: usize = args[6].parse()?;
    let blocks: usize = args[7].parse()?;
    assert!((32..=2048).contains(&trials));
    assert!((128..=2048).contains(&reference) && reference.is_power_of_two());
    assert!((5..=31).contains(&blocks));
    let work: usize = trials * counts.iter().sum::<usize>() * grids.len() * CASES.len();
    assert!(work <= 100_000_000, "evaluation budget exceeded");
    fs::create_dir_all(out)?;
    let mut refs = String::from("case,grid,r,g,b\n");
    let mut caches = String::from("case,grid,r,g,b,payload_bytes,source_calls\n");
    let mut raw = String::from(
        "case,grid,n,trial,plain_r,plain_g,plain_b,corrected_exact_r,corrected_exact_g,corrected_exact_b,two_level_r,two_level_g,two_level_b\n",
    );
    let mut times = String::from("case,grid,n,method,block,ns_per_estimate,loops\n");
    let mut maps = String::from("case,grid,x,y,f_r,f_g,f_b,g_r,g_g,g_b\n");
    for (case, name) in CASES.iter().enumerate() {
        if let Some(value) = analytic(case) {
            write!(refs, "{name},0")?;
            rgb_csv(&mut refs, value);
            refs.push('\n');
        } else {
            for side in [reference / 4, reference / 2, reference] {
                write!(refs, "{name},{side}")?;
                rgb_csv(&mut refs, midpoint(case, side));
                refs.push('\n');
            }
        }
        for &grid in &grids {
            let cache = Cache::new(case, grid);
            if grid == if grids.contains(&8) { 8 } else { grids[0] } {
                for y in 0..64 {
                    for x in 0..64 {
                        let p = [(x as f64 + 0.5) / 64.0, (y as f64 + 0.5) / 64.0];
                        write!(maps, "{name},{grid},{x},{y}")?;
                        rgb_csv(&mut maps, f(case, p[0], p[1]));
                        rgb_csv(&mut maps, cache.at(p));
                        maps.push('\n');
                    }
                }
            }
            write!(caches, "{name},{grid}")?;
            rgb_csv(&mut caches, cache.integral);
            writeln!(
                caches,
                ",{},{}",
                cache.values.len() * std::mem::size_of::<Rgb>(),
                grid * grid
            )?;
            for (block, (ns, loops)) in benchmark(
                || {
                    let cache = black_box(Cache::new(black_box(case), black_box(grid)));
                    black_box(cache.integral)
                },
                blocks,
            )
            .into_iter()
            .enumerate()
            {
                writeln!(times, "{name},{grid},0,cache_build,{block},{ns:.6},{loops}")?;
            }
            for &n in &counts {
                let mut y = vec![[0.0; 2]; n];
                let mut x = vec![[0.0; 2]; 4 * n];
                for trial in 0..trials {
                    fill(&mut y, seed(base_seed, case, trial, 0x243f6a8885a308d3));
                    fill(&mut x, seed(base_seed, case, trial, 0x13198a2e03707344));
                    write!(raw, "{name},{grid},{n},{trial}")?;
                    for method in 0..3 {
                        rgb_csv(&mut raw, estimate(case, method, &cache, &y, &x));
                    }
                    raw.push('\n');
                }
                for (name, include_x) in
                    [("input_generation_n", false), ("input_generation_5n", true)]
                {
                    let mut iteration = 0usize;
                    let measured = benchmark(
                        || {
                            fill(
                                black_box(&mut y),
                                seed(base_seed, case, iteration, 0x243f6a8885a308d3),
                            );
                            if include_x {
                                fill(
                                    black_box(&mut x),
                                    seed(base_seed, case, iteration, 0x13198a2e03707344),
                                );
                            }
                            iteration += 1;
                            black_box([y[0][0], y[n - 1][1], x[0][0]])
                        },
                        blocks,
                    );
                    for (block, (ns, loops)) in measured.into_iter().enumerate() {
                        writeln!(
                            times,
                            "{},{grid},{n},{name},{block},{ns:.6},{loops}",
                            CASES[case]
                        )?;
                    }
                }
                // Rotate execution order; wall times include PRNG/input filling
                // into already allocated scratch. No CSV/string work is timed.
                for offset in 0..3 {
                    let method = (offset + case + grid + n) % 3;
                    let method_name = ["plain", "corrected_exact", "two_level"][method];
                    let mut iteration = 0usize;
                    let measured = benchmark(
                        || {
                            fill(&mut y, seed(base_seed, case, iteration, 0x243f6a8885a308d3));
                            if method == 2 {
                                fill(&mut x, seed(base_seed, case, iteration, 0x13198a2e03707344));
                            }
                            iteration += 1;
                            estimate(
                                black_box(case),
                                black_box(method),
                                black_box(&cache),
                                black_box(&y),
                                black_box(&x),
                            )
                        },
                        blocks,
                    );
                    for (block, (ns, loops)) in measured.into_iter().enumerate() {
                        writeln!(
                            times,
                            "{name},{grid},{n},{method_name},{block},{ns:.6},{loops}"
                        )?;
                    }
                }
            }
        }
    }
    // Files are written only after numerical evaluation/timing, into the
    // runner's private pending directory. The runner publishes manifest last.
    fs::write(out.join("references.csv"), refs)?;
    fs::write(out.join("caches.csv"), caches)?;
    fs::write(out.join("trials.csv"), raw)?;
    fs::write(out.join("timings.csv"), times)?;
    fs::write(out.join("maps.csv"), maps)?;
    Ok(())
}
