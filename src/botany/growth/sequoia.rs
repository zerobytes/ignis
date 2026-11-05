//! Grammar5: authored giant sequoia architecture. Sprays aggregate scale-bearing
//! branchlets; they are not individual, oversized botanical leaves. No cones yet.
use super::*;
use glam::DVec3 as V;
use tree::noise;

pub fn definition(prefix: String, seed: u64) -> Definition {
    Definition {
        version: 5,
        prefix,
        seed,
        interval: 365,
        maximum_nodes: 16,
        internode_length: 0.6,
        schedule: Schedule {
            period: 365,
            phase: 0,
            maturity: 0,
            flower_duration: 0,
            fruit_duration: 0,
            ripe_duration: 0,
        },
        continuous: Some(continuous::Profile {
            seedling_height_m: 0.6,
            height_limit_m: 85.,
            height_reference_years: 1200.,
            seedling_diameter_m: 0.04,
            diameter_limit_m: 8.5,
            diameter_reference_years: 3000.,
            internode_m: 0.12,
            internode_days: 14,
            shoot_expansion_days: 365,
            leaf_turnover_days: 1095,
        }),
        foliage: Some(foliage::Profile {
            length_m: 0.45,
            width_m: 0.14,
            ..Default::default()
        }),
        stature: None,
        wood_volume: None,
    }
}
/// Cubic rational area growth avoids giving a seedling a metre-wide trunk.
/// The reference age is still the age at 95% of the diameter increment.
pub fn diameter(p: &continuous::Profile, age: u64) -> f64 {
    let (d0, d1) = (p.seedling_diameter_m, p.diameter_limit_m);
    if d0 == d1 {
        return d0;
    }
    let d95 = d0 + 0.95 * (d1 - d0);
    let f95 = (d95 * d95 - d0 * d0) / (d1 * d1 - d0 * d0);
    let tau = p.diameter_reference_years * (f95.powf(-1. / 3.) - 1.);
    let a = age as f64 / 365.;
    (d0 * d0 + (d1 * d1 - d0 * d0) * (a / (a + tau)).powi(3)).sqrt()
}
fn point(path: &[[f64; 3]], u: f64) -> V {
    let points: Vec<_> = path.iter().copied().map(V::from_array).collect();
    super::super::bake::curve(&points, u).0
}
fn path(start: V, angle: f64, length: f64, rise: f64, bend: f64) -> Vec<[f64; 3]> {
    (0..=8)
        .map(|i| {
            let u = i as f64 / 8.;
            (start
                + V::new(angle.cos(), 0., angle.sin()) * length * u
                + V::Y * (rise * u + bend * u * u))
                .to_array()
        })
        .collect()
}
fn woody(id: String, parent: String, at: f64, path: Vec<[f64; 3]>, radius: f64) -> Organ {
    let mut o = axis(id, Some(parent), at, path, radius);
    o.taper_power = 1.2;
    // Fine branchlets retain separate closed collars; large limbs are grafted below.
    o.connection = Connection::Sheath;
    if let Shape::Axis { tip_ratio, .. } = &mut o.shape {
        *tip_ratio = 0.055;
    }
    o
}
fn sprays(
    organs: &mut Vec<Organ>,
    parent: &str,
    path: &[[f64; 3]],
    count: u16,
    scale: f64,
    profile: &foliage::Profile,
    seed: u64,
) {
    let direction = V::from_array(*path.last().unwrap()) - V::from_array(path[0]);
    let angle = direction.z.atan2(direction.x);
    for i in 0..count {
        let at = 0.15 + 0.83 * i as f64 / (count - 1).max(1) as f64;
        let start = point(path, at);
        let a = angle + i as f64 * 2.3999632297;
        let length = profile.length_m * scale * (0.65 + 0.5 * noise(seed, i as u64));
        // Three narrow radial blades approximate a volumetric scale-leaf spray.
        for side in 0..3 {
            let b = a + side as f64 * std::f64::consts::TAU / 3.;
            let end = start + V::new(b.cos() * length, length * 0.55, b.sin() * length);
            let mut o = axis(
                format!("{parent}/spray{i}/{side}"),
                Some(parent.into()),
                at,
                vec![start.to_array(), end.to_array()],
                0.001,
            );
            o.connection = Connection::Sheath;
            o.shape = Shape::Blade {
                width: profile.width_m * scale,
                thickness: 0.001 * scale,
                fold: 0.35,
                cup: 0.08,
                outline: Outline::Strip {
                    base_power: 0.35,
                    tip_power: 1.5,
                },
            };
            o.normal = [b.cos(), 0.6, b.sin()];
            organs.push(o);
        }
    }
}
pub(super) fn recipe(plant: &Plant) -> Result<Recipe> {
    let d = &plant.definition;
    let p = d
        .continuous
        .as_ref()
        .ok_or_else(|| invalid("Missing sequoia growth profile"))?;
    let foliage = d
        .foliage
        .as_ref()
        .ok_or_else(|| invalid("Missing sequoia sprays"))?;
    let age = plant.production.plant.time - plant.production.plant.born;
    let years = age as f64 / 365.;
    let height = p.height(age);
    let radius = diameter(p, age) * 0.5;
    let adult = years / (years + 100.);
    let trunk: Vec<_> = (0..=32)
        .map(|i| {
            let u = i as f64 / 32.;
            [
                height * 0.005 * u * u * (noise(d.seed, 400) - 0.5),
                height * u,
                height * 0.004 * (u * 4.).sin() * u * adult,
            ]
        })
        .collect();
    let mut stem = axis("stem".into(), None, 0., trunk.clone(), radius);
    stem.taper_power = 0.85;
    if let Shape::Axis { tip_ratio, .. } = &mut stem.shape {
        *tip_ratio = 0.018;
    }
    let mut organs = vec![stem];
    let spray_scale = (height / 10.).clamp(0.3, 2.4);
    // Photosynthetic apex exists even at birth, independent of primary buds.
    let apex = path(
        point(&trunk, 0.94),
        noise(d.seed, 99) * 6.,
        height * 0.025,
        height * 0.045,
        0.,
    );
    let mut top = woody(
        "leader".into(),
        "stem".into(),
        0.94,
        apex.clone(),
        (radius * 0.012).max(0.002),
    );
    top.connection = Connection::Sheath;
    organs.push(top);
    sprays(
        &mut organs,
        "leader",
        &apex,
        foliage.apical_leaves,
        spray_scale,
        foliage,
        d.seed,
    );
    let nodes = (age / d.interval).min(d.maximum_nodes as u64);
    let lower = (d.maximum_nodes / 3).max(1) as u64;
    for i in 0..nodes {
        let site = format!("node{i}");
        if plant.production.removed_sites.contains(&site) || (i < lower && years >= 300.) {
            continue;
        }
        let at = 0.18 + 0.74 * i as f64 / (d.maximum_nodes - 1).max(1) as f64;
        let g = ((age - (i + 1) * d.interval) as f64 / p.shoot_expansion_days as f64).clamp(0., 1.);
        let survival = if i < lower {
            ((300. - years) / 150.).clamp(0., 1.)
        } else {
            1.
        };
        let mut support = Vec::new();
        for arm in 0..3u64 {
            let id = if arm == 0 {
                site.clone()
            } else {
                format!("{site}/arm{arm}")
            };
            let a = i as f64 * 2.3999632297
                + arm as f64 * std::f64::consts::TAU / 3.
                + noise(d.seed, i * 3 + arm) * 0.5;
            let length = height
                * (1. - at).powf(0.7 - 0.3 * adult)
                * (0.25 + adult * 0.02)
                * (0.85 + 0.25 * noise(d.seed, i * 3 + arm + 30))
                * (0.15 + 0.85 * g);
            let start = if arm == 0 {
                point(&trunk, at)
            } else {
                point(&support, 0.04)
            };
            let branch = path(
                start,
                a,
                length,
                length * (-0.08 + adult * 0.28),
                length * (0.1 + adult * 0.2),
            );
            let mut limb = woody(
                id.clone(),
                if arm == 0 {
                    "stem".into()
                } else {
                    site.clone()
                },
                if arm == 0 { at } else { 0.04 },
                branch.clone(),
                (radius * (1. - at) * 0.19).max(0.003),
            );
            // Sparse structural grafts avoid overlapping giant-trunk collar domains.
            // The other limbs/branchlets use explicit separate closed sheaths.
            if arm == 0 && i % 4 == 0 {
                limb.connection = Connection::Graft;
            }
            organs.push(limb);
            if arm == 0 {
                support = branch.clone();
            }
            sprays(
                &mut organs,
                &id,
                &branch,
                foliage.primary_leaves,
                spray_scale * survival.max(0.05),
                foliage,
                d.seed + i * 3 + arm,
            );
            if g < 0.5 {
                continue;
            }
            for j in 0..8u64 {
                let u = 0.2 + j as f64 * 0.105;
                let b = a + if j % 2 == 0 { 0.85 } else { -0.85 };
                let l = length
                    * (0.28 + 0.12 * noise(d.seed, i * 100 + j + arm * 31))
                    * (1. - u * 0.45);
                let secondary = path(
                    point(&branch, u),
                    b,
                    l,
                    l * 0.05,
                    l * (0.6 + 0.4 * noise(d.seed, i * 200 + j)),
                );
                let sid = format!("{id}/b{j}");
                organs.push(woody(
                    sid.clone(),
                    id.clone(),
                    u,
                    secondary.clone(),
                    (radius * (1. - at) * 0.025).max(0.002),
                ));
                sprays(
                    &mut organs,
                    &sid,
                    &secondary,
                    foliage.secondary_leaves,
                    spray_scale * survival.max(0.05),
                    foliage,
                    d.seed + j + arm * 100 + 100,
                );
                for k in 0..4u64 {
                    let v = 0.25 + k as f64 * 0.22;
                    let c = b + if k % 2 == 0 { 0.8 } else { -0.8 };
                    let tl = l * (0.45 + 0.18 * noise(d.seed, i * 1000 + j * 10 + k));
                    let twig = path(point(&secondary, v), c, tl, tl * 0.1, tl * 0.5);
                    let tid = format!("{sid}/t{k}");
                    organs.push(woody(
                        tid.clone(),
                        sid.clone(),
                        v,
                        twig.clone(),
                        (radius * 0.002).max(0.0015),
                    ));
                    sprays(
                        &mut organs,
                        &tid,
                        &twig,
                        foliage.secondary_leaves,
                        spray_scale * survival.max(0.05),
                        foliage,
                        d.seed + i * 1000 + j * 10 + k + arm * 137,
                    );
                }
            }
        }
    }
    let r = Recipe {
        version: 8,
        prefix: d.prefix.clone(),
        source: Arc::new(Source {
            version: 8,
            name: "Sequoia-gigante · exemplar vivo".into(),
            seed: d.seed,
            organs,
            development: None,
            root_systems: vec![],
        }),
        settings: Settings {
            axis_samples: 16,
            blade_samples: 4,
            sides: 12,
            bark_color: [0.48, 0.19, 0.085],
            leaf_color: [0.12, 0.27, 0.105],
            ..Default::default()
        },
        edits: BTreeMap::new(),
        pruned: BTreeSet::new(),
        manual: BTreeMap::new(),
        living: None,
    };
    r.validate()?;
    Ok(r)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequoia_age_pruning_roundtrip_and_bake() {
        for years in [0, 1, 5, 20, 100, 300, 1000, 3000] {
            let mut plant = Plant::new(definition("sequoia".into(), 83), 0).unwrap();
            let r = plant
                .advance(years * 365)
                .unwrap_or_else(|e| panic!("age {years}: {e:?}"));
            assert!(
                r.source
                    .organs
                    .iter()
                    .any(|o| matches!(o.shape, Shape::Blade { .. }))
            );
            assert!(
                plant
                    .production
                    .attached(&plant.definition.schedule, MAX_ATTACHED)
                    .unwrap()
                    .is_empty()
            );
            let restored: Plant =
                serde_json::from_str(&serde_json::to_string(&plant).unwrap()).unwrap();
            assert_eq!(r, restored.recipe().unwrap());
        }
        let mut p = Plant::new(definition("sequoia".into(), 83), 0).unwrap();
        p.advance(1000 * 365).unwrap();
        p.production.remove_site("node8").unwrap();
        let r = p.advance(1001 * 365).unwrap();
        assert!(
            !r.source
                .organs
                .iter()
                .any(|o| o.id == "node8" || o.id.starts_with("node8/"))
        );
        let c = p.definition.continuous.as_ref().unwrap();
        assert!(diameter(c, 365) < 0.06);
        assert!((diameter(c, 3000 * 365) - (0.04 + 0.95 * (8.5 - 0.04))).abs() < 1e-10);
        assert!(diameter(c, 4000 * 365) > diameter(c, 3000 * 365));
    }
}
