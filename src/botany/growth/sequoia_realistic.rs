//! Grammar6: irregular, billowing mature giant-sequoia crown and derived fluted bole.
//! Foliage blades are finite aggregate branchlet proxies, not individual scales.
use super::*;
use glam::DVec3 as V;
use std::f64::consts::TAU;
use tree::noise;

pub fn definition(prefix: String, seed: u64) -> Definition {
    let mut d = sequoia::definition(prefix, seed);
    d.version = 6;
    d.maximum_nodes = 22;
    d.foliage.as_mut().unwrap().length_m = 0.35;
    d.foliage.as_mut().unwrap().width_m = 0.24;
    d
}
fn point(p: &[[f64; 3]], u: f64) -> V {
    super::super::bake::curve(&p.iter().copied().map(V::from_array).collect::<Vec<_>>(), u).0
}
fn axis_path(start: V, a: f64, l: f64, rise: f64, seed: u64) -> Vec<[f64; 3]> {
    let dir = V::new(a.cos(), 0., a.sin());
    let side = V::new(-a.sin(), 0., a.cos());
    (0..=8)
        .map(|i| {
            let u = i as f64 / 8.;
            (start
                + dir * l * u
                + side * l * (noise(seed, 51) - 0.5) * 0.3 * (u * 3.).sin() * u
                + V::Y * l * (rise * u - 0.45 * u * (1. - u) + 0.25 * u * u * u))
                .to_array()
        })
        .collect()
}
fn wood(id: String, parent: Option<String>, at: f64, p: Vec<[f64; 3]>, r: f64) -> Organ {
    let mut o = axis(id, parent, at, p, r);
    o.connection = Connection::Sheath;
    o.taper_power = 0.8;
    if let Shape::Axis { tip_ratio, .. } = &mut o.shape {
        *tip_ratio = 0.045;
    }
    o
}
fn foliage(
    organs: &mut Vec<Organ>,
    parent: &str,
    p: &[[f64; 3]],
    size: f64,
    seed: u64,
    profile: &foliage::Profile,
    count: u16,
) {
    // Spiralling small sprays form rounded clusters at different heights/depths.
    for k in 0..count as u64 {
        let u = 0.15 + 0.83 * (k as f64 + noise(seed, k) * 0.5) / count as f64;
        let root = point(p, u);
        let a = k as f64 * 2.399963 + noise(seed, k + 100) * 1.4;
        let l = size * (profile.length_m / 0.35) * (0.55 + noise(seed, k + 200) * 0.7);
        let dir = V::new(a.cos(), noise(seed, k + 300) * 1.3 - 0.4, a.sin()).normalize();
        let tip = root + dir * l;
        let mut o = wood(
            format!("{parent}/s{k}"),
            Some(parent.into()),
            u,
            vec![root.to_array(), tip.to_array()],
            0.001,
        );
        o.normal = [a.sin(), 1., -a.cos()];
        o.shape = Shape::Blade {
            width: l * profile.width_m / profile.length_m,
            thickness: 0.001,
            fold: 0.55,
            cup: 0.12,
            outline: Outline::Palmate {
                tips: 13,
                spread: 1.3,
                depth: 0.55,
                rounding: 0.7,
                valley_rounding: 2.2,
                lengths: (0..13)
                    .map(|j| {
                        let t = (j as f64 - 6.) / 6.;
                        0.35 + 0.65 * (1. - t * t).max(0.).sqrt()
                    })
                    .collect(),
            },
        };
        organs.push(o);
    }
}
/// Generated organ-local geometry, not an editable overlay. Re-evaluates with age.
pub fn bole(radius: f64, seed: u64) -> super::super::wood_surface::Surface {
    use super::super::wood_surface::{Section, Surface};
    let sections = [0., 0.025, 0.07, 0.18, 0.38, 0.58, 0.75, 0.88, 1.]
        .iter()
        .enumerate()
        .map(|(i, &u)| {
            let profile = (1. - u * 0.75_f64).max(0.02) * (1. + 0.34 * (-u * 28.).exp());
            let profile = profile
                * if u > 0.75 {
                    (1. - (u - 0.75) / 0.25) * 0.94 + 0.06
                } else {
                    1.
                };
            let radii = (0..32)
                .map(|j| {
                    let a = j as f64 * TAU / 32.;
                    (radius
                        * profile
                        * (1.
                            + (0.035 + 0.10 * (-u * 9.).exp())
                                * (a * 7. + 0.4 * (u * 5.).sin()).sin()
                            + 0.025 * (a * 11. + u * 2.).sin()
                            + 0.025 * (noise(seed, j) - 0.5)))
                        .max(0.02)
                })
                .collect();
            Section {
                id: format!("section{i}"),
                u,
                angle: 0.,
                offset: [0.; 2],
                radii,
            }
        })
        .collect();
    Surface {
        sections,
        relief: vec![],
        sides: 32,
        samples: 128,
    }
}
pub(super) fn recipe(plant: &Plant) -> Result<Recipe> {
    let d = &plant.definition;
    let p = d.continuous.as_ref().unwrap();
    let age = plant.production.plant.time - plant.production.plant.born;
    let years = age as f64 / 365.;
    let h = p.height(age);
    let r = sequoia::diameter(p, age) * 0.5;
    let tuft_count = if years > 300. { 5 } else { 2 };
    let mature = (years / (years + 100.)).clamp(0., 1.);
    let trunk: Vec<_> = (0..=32)
        .map(|i| {
            let u = i as f64 / 32.;
            [
                h * 0.014 * u * u * (u * 4.).sin(),
                h * u,
                h * 0.009 * u * u * (u * 5.).sin(),
            ]
        })
        .collect();
    let mut organs = vec![wood("stem".into(), None, 0., trunk.clone(), r)];
    // The juvenile leader remains photosynthetic at birth.
    let apex = axis_path(point(&trunk, 0.94), 0., h * 0.015, 1., d.seed + 99);
    organs.push(wood(
        "leader".into(),
        Some("stem".into()),
        0.94,
        apex.clone(),
        (r * 0.01).max(0.002),
    ));
    foliage(
        &mut organs,
        "leader",
        &apex,
        (h / 20.).clamp(0.08, 0.65),
        d.seed,
        d.foliage.as_ref().unwrap(),
        tuft_count,
    );
    let nodes = (age / d.interval).min(d.maximum_nodes as u64);
    let lower = d.maximum_nodes as u64 / 2;
    for i in 0..nodes {
        let id = format!("node{i}");
        if plant.production.removed_sites.contains(&id) || (years > 300. && i < lower) {
            continue;
        }
        let q = i as f64 / (d.maximum_nodes - 1).max(1) as f64;
        let at = ((0.08 + 0.87 * q) * (1. - mature)
            + (0.28 + 0.69 * q) * mature
            + 0.015 * (noise(d.seed, i + 31) - 0.5))
            .clamp(0.05, 0.97);
        let a = i as f64 * 2.399963 + noise(d.seed, i + 32) * 1.7;
        let juvenile = (1. - at).powf(0.65) * 0.22;
        let dome = (1. - ((at - 0.74) / 0.31).powi(2)).max(0.03).sqrt() * 0.14;
        let l =
            h * (juvenile * (1. - mature) + dome * mature) * (0.72 + 0.5 * noise(d.seed, i + 33));
        let branch = axis_path(
            point(&trunk, at),
            a,
            l,
            -0.04 + noise(d.seed, i + 34) * 0.24,
            d.seed + i,
        );
        organs.push(wood(
            id.clone(),
            Some("stem".into()),
            at,
            branch.clone(),
            (r * 0.16 * (1. - at).sqrt()).max(0.003),
        ));
        let g = ((age - (i + 1) * d.interval) as f64 / p.shoot_expansion_days as f64).clamp(0., 1.);
        if g < 0.5 {
            foliage(
                &mut organs,
                &id,
                &branch,
                0.15,
                d.seed + i,
                d.foliage.as_ref().unwrap(),
                tuft_count,
            );
            continue;
        }
        for j in 0..12u64 {
            let u = 0.18 + 0.77 * (j as f64 + noise(d.seed, i * 101 + j) * 0.6) / 12.;
            let b = a + (noise(d.seed, i * 103 + j) - 0.5) * 3.6;
            let sl = l * (0.26 + noise(d.seed, i * 107 + j) * 0.28);
            let sp = axis_path(
                point(&branch, u),
                b,
                sl,
                noise(d.seed, i * 109 + j) * 1.0 - 0.25,
                d.seed + i * 100 + j,
            );
            let sid = format!("{id}/b{j}");
            organs.push(wood(
                sid.clone(),
                Some(id.clone()),
                u,
                sp.clone(),
                (r * 0.018).max(0.002),
            ));
            for k in 0..8u64 {
                let v = 0.18 + 0.78 * (k as f64 + noise(d.seed, j * 137 + k) * 0.6) / 8.;
                let c = b + (noise(d.seed, i * 1009 + j * 11 + k) - 0.5) * 4.5;
                let tl = sl * (0.38 + noise(d.seed, i * 1013 + j * 11 + k) * 0.34);
                let tp = axis_path(
                    point(&sp, v),
                    c,
                    tl,
                    noise(d.seed, i * 1021 + j * 11 + k) * 1.5 - 0.6,
                    d.seed + i * 1000 + j * 10 + k,
                );
                let tid = format!("{sid}/t{k}");
                organs.push(wood(
                    tid.clone(),
                    Some(sid.clone()),
                    v,
                    tp.clone(),
                    (r * 0.003).max(0.0015),
                ));
                foliage(
                    &mut organs,
                    &tid,
                    &tp,
                    (h / 35.).clamp(0.12, 1.8),
                    d.seed + i * 10000 + j * 100 + k,
                    d.foliage.as_ref().unwrap(),
                    tuft_count,
                );
            }
        }
    }
    let recipe = Recipe {
        version: 8,
        prefix: d.prefix.clone(),
        source: Arc::new(Source {
            version: 8,
            name: "Sequoia-gigante · copa madura irregular".into(),
            seed: d.seed,
            organs,
            development: None,
            root_systems: vec![],
        }),
        settings: Settings {
            axis_samples: 12,
            blade_samples: 4,
            sides: 8,
            bark_color: [1.; 3],
            leaf_color: [1.; 3],
            ..Default::default()
        },
        edits: BTreeMap::new(),
        pruned: BTreeSet::new(),
        manual: BTreeMap::new(),
        living: None,
    };
    recipe.validate()?;
    Ok(recipe)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_realistic_sequoia_age_and_pruning() {
        for y in [0, 1, 20, 299, 1000, 3000] {
            let mut p = Plant::new(definition("real".into(), 83), 0).unwrap();
            p.advance(y * 365).unwrap();
            super::super::super::bake::compile(&p.bound_recipe().unwrap()).unwrap();
        }
        let mut p = Plant::new(definition("real".into(), 83), 0).unwrap();
        p.advance(1000 * 365).unwrap();
        p.production.remove_site("node14").unwrap();
        let r = p.advance(1001 * 365).unwrap();
        assert!(!r.source.organs.iter().any(|o| o.id.starts_with("node14/")));
    }
}
