//! A seeded broadleaf tree grammar, retaining the persistent node/site identities.
use super::*;
use glam::DVec3 as V;

fn progress(age: u64, birth: u64, duration: u64) -> f64 {
    let u = (age.saturating_sub(birth) as f64 / duration as f64).clamp(0., 1.);
    u * u * (3. - 2. * u)
}
fn point(path: &[[f64; 3]], u: f64) -> V {
    let x = u * (path.len() - 1) as f64;
    let i = (x.floor() as usize).min(path.len() - 2);
    V::from_array(path[i]).lerp(V::from_array(path[i + 1]), x - i as f64)
}
fn curve(start: V, direction: V, length: f64, lift: f64) -> Vec<[f64; 3]> {
    (0..=8)
        .map(|i| {
            let u = i as f64 / 8.;
            (start + direction * length * u + V::Y * lift * u * u).to_array()
        })
        .collect()
}
pub(super) fn noise(seed: u64, i: u64) -> f64 {
    let mut n = seed.wrapping_add(i.wrapping_mul(0x9e3779b97f4a7c15));
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((n ^ (n >> 31)) >> 11) as f64 / (1u64 << 53) as f64
}
fn young_leaves(
    organs: &mut Vec<Organ>,
    parent: &str,
    path: &[[f64; 3]],
    count: u16,
    angle: f64,
    growth: f64,
    profile: &foliage::Profile,
) {
    let (length, width) = profile.dimensions(growth);
    for l in 0..count {
        let at = 0.45 + 0.52 * l as f64 / (count - 1) as f64;
        let a = angle + l as f64 * 2.3999632297;
        let start = point(path, at);
        let end = start + V::new(a.cos() * length, 0.06, a.sin() * length);
        let mut leaf = axis(
            format!("{parent}/young_leaf{l}"),
            Some(parent.into()),
            at,
            vec![start.to_array(), end.to_array()],
            0.001,
        );
        leaf.shape = Shape::Blade {
            width,
            thickness: 0.0003,
            fold: 0.12,
            cup: 0.06,
            outline: Outline::Strip {
                base_power: 0.55,
                tip_power: 1.2,
            },
        };
        leaf.normal = [a.cos() * 0.3, 1., a.sin() * 0.3];
        organs.push(leaf);
    }
}
pub(super) fn recipe(plant: &Plant) -> Result<Recipe> {
    let d = &plant.definition;
    let age = plant.production.plant.time - plant.production.plant.born;
    let nodes = (age / d.interval).min(d.maximum_nodes as u64);
    let maturity = progress(age, 0, d.interval * (d.maximum_nodes as u64 + 8));
    let height =
        0.6 + (age as f64 / d.interval as f64).min(d.maximum_nodes as f64) * d.internode_length;
    let trunk: Vec<_> = (0..=12)
        .map(|i| {
            let u = i as f64 / 12.;
            [
                0.16 * u * u * maturity,
                height * u,
                0.12 * (u * 3.).sin() * u * maturity,
            ]
        })
        .collect();
    let mut organs = vec![axis(
        "stem".into(),
        None,
        0.,
        trunk.clone(),
        0.025 + 0.075 * maturity,
    )];
    organs[0].taper_power = 0.7;
    if let Shape::Axis { tip_ratio, .. } = &mut organs[0].shape {
        *tip_ratio = 0.03;
    }
    if let Some(profile) = &d.foliage && d.version<4 {
        young_leaves(
            &mut organs,
            "stem",
            &trunk,
            profile.apical_leaves,
            noise(d.seed, 71000) * 6.,
            progress(age, 0, profile.expansion_days),
            profile,
        );
    }
    for i in 0..nodes {
        let id = format!("node{i}");
        if plant.production.removed_sites.contains(&id) {
            continue;
        }
        let birth = (i + 1) * d.interval;
        let shoot_days=d.continuous.as_ref().map_or(d.interval*4,|p|p.shoot_expansion_days);
        let g = progress(age, birth, shoot_days);
        let y = 1.0 + i as f64 * d.internode_length * 0.74;
        let insertion = if d.version==4 {0.18+0.64*i as f64/d.maximum_nodes as f64}else{(y / height).min(0.95)};
        let angle = i as f64 * 2.3999632297 + noise(d.seed, i) * 0.7;
        let dir = V::new(
            angle.cos(),
            0.02 + 0.28 * noise(d.seed, i + 40),
            angle.sin(),
        );
        let envelope = 0.7
            + 0.5 * (std::f64::consts::PI * (i as f64 + 1.) / (d.maximum_nodes as f64 + 1.)).sin();
        let length = 0.25 + 2.6 * envelope * (0.8 + 0.35 * noise(d.seed, i + 55)) * g;
        let path = curve(
            point(&trunk, insertion),
            dir,
            length,
            (0.2 + 0.9 * noise(d.seed, i + 70)) * g,
        );
        organs.push(axis(
            id.clone(),
            Some("stem".into()),
            insertion,
            path.clone(),
            if d.version==4 {(0.025+0.075*maturity)*(0.1+0.2*g)}else{0.006+0.04*g*(0.5+maturity*0.5)},
        ));
        // Persistent juvenile foliage: early crowns are visible before later orders emerge.
        let leaf_count = if d.version==4 {0}else{d.foliage.as_ref().map_or(16, |p| p.primary_leaves as u64)};
        for l in 0..leaf_count {
            let at = if d.foliage.is_some() {
                0.15 + l as f64 * 0.8 / (leaf_count - 1) as f64
            } else {
                0.25 + l as f64 * 0.045
            };
            let a = angle + l as f64 * 2.3999632297;
            let start = point(&path, at);
            let (len, width) = d
                .foliage
                .as_ref()
                .map(|p| p.dimensions(progress(age, birth, p.expansion_days)))
                .unwrap_or((0.07 + 0.18 * g, 0.025 + 0.06 * g));
            let end = start + V::new(a.cos() * len, 0.04, a.sin() * len);
            let mut leaf = axis(
                format!("{id}/leaf{l}"),
                Some(id.clone()),
                at,
                vec![start.to_array(), end.to_array()],
                0.001,
            );
            leaf.shape = Shape::Blade {
                width,
                thickness: 0.0003,
                fold: 0.12,
                cup: 0.06,
                outline: Outline::Strip {
                    base_power: 0.55,
                    tip_power: 1.2,
                },
            };
            leaf.normal = [a.cos() * 0.3, 1., a.sin() * 0.3];
            organs.push(leaf);
        }
        if age < birth + d.continuous.as_ref().map_or(d.interval,|p|p.shoot_expansion_days) {
            continue;
        }
        for j in 0..6u64 {
            let secondary_span=d.continuous.as_ref().map_or(d.interval,|p|p.shoot_expansion_days);
            let secondary_birth = birth + d.continuous.as_ref().map_or(d.interval*4,|p|p.shoot_expansion_days) + j * secondary_span / 3;
            if age < secondary_birth {
                continue;
            }
            let sg = progress(age, secondary_birth, d.continuous.as_ref().map_or(d.interval*3,|p|p.shoot_expansion_days));
            let at = 0.28 + j as f64 * 0.115;
            let a = angle + if j % 2 == 0 { 0.85 } else { -0.85 };
            let direction = V::new(
                a.cos(),
                0.08 + 0.8 * noise(d.seed, i * 100 + j + 900),
                a.sin(),
            );
            let branch = curve(point(&path, at), direction, 0.16 + 1.15 * sg, 0.35 * sg);
            let sid = format!("{id}/b{j}");
            organs.push(axis(
                sid.clone(),
                Some(id.clone()),
                at,
                branch.clone(),
                if d.version==4 {(0.025+0.075*maturity)*(0.03+0.04*sg)}else{0.002+0.009*sg},
            ));
            if let Some(profile) = &d.foliage && d.version<4 {
                young_leaves(
                    &mut organs,
                    &sid,
                    &branch,
                    profile.secondary_leaves,
                    a,
                    progress(age, secondary_birth, profile.expansion_days),
                    profile,
                );
            }
            for k in 0..4u64 {
                let twig_birth = secondary_birth + d.continuous.as_ref().map_or(d.interval*3,|p|p.shoot_expansion_days) + k * secondary_span / 4;
                if age < twig_birth {
                    continue;
                }
                let tg = progress(age, twig_birth, d.continuous.as_ref().map_or(d.interval*2,|p|p.shoot_expansion_days));
                let at = 0.35 + k as f64 * 0.18;
                let a = a + if k % 2 == 0 { 0.75 } else { -0.75 };
                let td = V::new(
                    a.cos(),
                    -0.12 + 1.0 * noise(d.seed, i * 1000 + j * 10 + k + 8000),
                    a.sin(),
                );
                let twig = curve(point(&branch, at), td, 0.08 + 0.62 * tg, 0.12 * tg);
                let tid = format!("{sid}/t{k}");
                organs.push(axis(
                    tid.clone(),
                    Some(sid.clone()),
                    at,
                    twig.clone(),
                    if d.version==4 {(0.025+0.075*maturity)*(0.01+0.02*tg)}else{0.002+0.002*tg},
                ));
                for l in 0..if d.version==4 {0}else{10u64} {
                    let at = 0.22 + l as f64 * 0.077;
                    let leaf_angle = a
                        + l as f64 * 2.3999632297
                        + noise(d.seed, i * 1000 + j * 100 + k * 10 + l);
                    let start = point(&twig, at);
                    let (len, width) = d
                        .foliage
                        .as_ref()
                        .map(|p| p.dimensions(progress(age, twig_birth, p.expansion_days)))
                        .unwrap_or((0.08 + 0.19 * tg, 0.025 + 0.065 * tg));
                    let end = start
                        + V::new(
                            leaf_angle.cos() * len,
                            0.04 + len * 0.2,
                            leaf_angle.sin() * len,
                        );
                    let mut leaf = axis(
                        format!("{tid}/l{l}"),
                        Some(tid.clone()),
                        at,
                        vec![start.to_array(), end.to_array()],
                        0.001,
                    );
                    leaf.shape = Shape::Blade {
                        width,
                        thickness: 0.0003,
                        fold: 0.12,
                        cup: 0.06,
                        outline: Outline::Strip {
                            base_power: 0.55,
                            tip_power: 1.2,
                        },
                    };
                    leaf.normal = [leaf_angle.cos() * 0.3, 1., leaf_angle.sin() * 0.3];
                    organs.push(leaf);
                }
            }
        }
    }
    let r = Recipe {
        version: 8,
        prefix: d.prefix.clone(),
        source: Arc::new(Source {
            version: 8,
            name: "Árvore viva · copa ampla".into(),
            seed: d.seed,
            organs,
            development: None,
            root_systems: vec![],
        }),
        settings: Settings {
            axis_samples: 12,
            blade_samples: 6,
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
    fn broadleaf_tree_grows_bakes_and_preserves_interventions() {
        let definition = Definition {
            version: 2,
            interval: 365,
            maximum_nodes: 10,
            ..Default::default()
        };
        let mut plant = Plant::new(definition, 0).unwrap();
        let young = plant.advance(365 * 3).unwrap();
        for year in [5, 8, 10, 12, 15] {
            plant.advance(365 * year).unwrap();
        }
        let adult = plant.advance(365 * 20).unwrap();
        assert!(adult.source.organs.len() > 2500);
        assert!(adult.source.organs.len() > young.source.organs.len());
        assert!(adult.source.organs.iter().any(|o| o.id == "node0/b5/t3/l9"));
        let restored: Plant =
            serde_json::from_str(&serde_json::to_string(&plant).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(restored.recipe().unwrap()).unwrap(),
            serde_json::to_value(&adult).unwrap()
        );
        plant.production.remove_site("node0").unwrap();
        let pruned = plant.advance(365 * 21).unwrap();
        assert!(
            !pruned
                .source
        wood_details: None,
                .organs
                .iter()
                .any(|o| o.id == "node0" || o.id.starts_with("node0/"))
        );
    }
}
