//! Seeded organ-local volumes built through the shared woody-surface evaluator.
//! Ratios use the source radius; zero amplitude preserves the baseline exactly.
use super::super::wood_surface::{Control, Relief, Section, Surface};
use super::*;
use std::f64::consts::{PI, TAU};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub bulge_ratio: f64,
    pub bulge_count: u8,
    pub cord_height_ratio: f64,
    pub valley_depth_ratio: f64,
    pub cord_width_ratio: f64,
    pub valley_width_ratio: f64,
    pub cord_count: u8,
    pub meander: f64,
    pub extent: f64,
    #[serde(default, skip_serializing_if = "is_false")]
    pub continuous: bool,
}
fn is_false(v: &bool) -> bool {
    !*v
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            bulge_ratio: 0.24,
            bulge_count: 5,
            cord_height_ratio: 0.18,
            valley_depth_ratio: 0.10,
            cord_width_ratio: 0.27,
            valley_width_ratio: 0.22,
            cord_count: 7,
            meander: 0.24,
            extent: 0.48,
            continuous: true,
        }
    }
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        if [
            self.bulge_ratio,
            self.cord_height_ratio,
            self.valley_depth_ratio,
        ]
        .iter()
        .any(|v| !v.is_finite() || !(0. ..=0.45).contains(v))
            || self.bulge_count > 8
            || !(3..=9).contains(&self.cord_count)
            || !self.cord_width_ratio.is_finite()
            || !(0.15..=0.5).contains(&self.cord_width_ratio)
            || !self.valley_width_ratio.is_finite()
            || !(0.1..=0.45).contains(&self.valley_width_ratio)
            || !self.meander.is_finite()
            || !(0. ..=0.8).contains(&self.meander)
            || !self.extent.is_finite()
            || !(0.3..=0.7).contains(&self.extent)
        {
            return Err(invalid("Invalid woody-volume profile"));
        }
        Ok(())
    }
    pub fn apply(&self, base: &Surface, radius: f64, seed: u64) -> Result<Surface> {
        self.validate()?;
        base.validate()?;
        if !radius.is_finite() || radius <= 0. {
            return Err(invalid("Invalid woody-volume radius"));
        }
        if (self.bulge_ratio == 0. || self.bulge_count == 0)
            && self.cord_height_ratio == 0.
            && self.valley_depth_ratio == 0.
        {
            return Ok(base.clone());
        }
        // Smoothly suppress added relief near the evaluator's 2cm radial floor.
        let scale = (radius - 0.05).max(0.).powi(2) / radius;
        let bump = |x: f64| (1. - x * x).max(0.).powi(2);
        let sections = (0..32)
            .map(|i| {
                let u = if i <= 16 {
                    i as f64 * 0.0125
                } else {
                    0.2 + 0.8 * (i - 16) as f64 / 15.
                };
                let radii = (0..32)
                    .map(|j| {
                        let a = j as f64 * TAU / 32.;
                        let mut r = base.radius(u, a).max(0.02);
                        for b in 0..self.bulge_count as u64 {
                            let center = 0.025 + 0.17 * tree::noise(seed, 1000 + b * 7);
                            let length = 0.022 + 0.025 * tree::noise(seed, 1001 + b * 7);
                            let angle = TAU * tree::noise(seed, 1002 + b * 7);
                            let angular = 0.35 + 0.4 * tree::noise(seed, 1003 + b * 7);
                            let d = (a - angle + PI).rem_euclid(TAU) - PI;
                            r += scale
                                * self.bulge_ratio
                                * bump((u - center) / length)
                                * bump(d / angular);
                        }
                        r
                    })
                    .collect();
                Section {
                    id: format!("volume{i}"),
                    u,
                    angle: 0.,
                    offset: base.offset(u),
                    radii,
                }
            })
            .collect();
        let mut relief = Vec::new();
        for i in 0..self.cord_count as u64 {
            let angle = TAU * (i as f64 + 0.2 * (tree::noise(seed, 2000 + i) - 0.5))
                / self.cord_count as f64;
            for valley in [false, true] {
                let path = [0., 0.12, 0.35, 0.7, 1.]
                    .into_iter()
                    .enumerate()
                    .map(|(k, t)| Control {
                        u: t * if self.continuous { 1. } else { self.extent },
                        angle: angle
                            + if valley {
                                PI / self.cord_count as f64
                            } else {
                                0.
                            }
                            + self.meander * (tree::noise(seed, 2100 + i * 11 + k as u64) - 0.5),
                    })
                    .collect();
                relief.push(Relief {
                    id: format!("{}{}", if valley { "valley" } else { "cord" }, i),
                    enabled: true,
                    path,
                    width: (scale
                        * if valley {
                            self.valley_width_ratio
                        } else {
                            self.cord_width_ratio
                        })
                    .max(0.02),
                    height: scale
                        * if valley {
                            -self.valley_depth_ratio
                        } else {
                            self.cord_height_ratio * (0.8 + 0.4 * tree::noise(seed, 2300 + i))
                        },
                    end_fade: if self.continuous { 0. } else { 0.035 },
                    follow_taper: self.continuous,
                });
            }
        }
        let result = Surface {
            sections,
            relief,
            sides: 96,
            samples: 128,
        };
        result.validate()?;
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_living_volume_mesh_is_closed_and_keeps_crown() {
        let mut d = super::super::sequoia_realistic::definition("volume".into(), 83);
        let mut p = Plant::new(d.clone(), 0).unwrap();
        p.advance(365000).unwrap();
        let before = p.bound_recipe().unwrap();
        let a = super::super::super::bake::compile(&before).unwrap();
        d.version = 8;
        d.wood_volume = Some(Profile::default());
        let mut p = Plant::new(d, 0).unwrap();
        p.advance(365000).unwrap();
        let after = p.bound_recipe().unwrap();
        assert_eq!(before.source, after.source);
        let b = super::super::super::bake::compile(&after).unwrap();
        let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
        let mut volume = 0.;

        for (tri, &owner) in b.document.assets[0]
            .geometry
            .triangles
            .iter()
            .zip(&b.triangles_by_organ[0])
        {
            if owner != 0 {
                continue;
            }
            for i in 0..3 {
                let x = tri[i];
                let y = tri[(i + 1) % 3];
                let e = edges.entry((x.min(y), x.max(y))).or_default();
                e.0 += 1;
                e.1 += if x < y { 1 } else { -1 };
            }
            let points = tri.map(|i| {
                glam::DVec3::from_array(
                    b.document.assets[0].geometry.positions[i as usize].map(f64::from),
                )
            });
            volume += points[0].dot(points[1].cross(points[2])) / 6.;
        }
        assert!(volume > 0. && edges.values().all(|&(count, sign)| count == 2 && sign == 0));
        let crown = |c: &super::super::super::bake::Compiled| {
            c.vertices[0]
                .iter()
                .zip(&c.document.assets[0].geometry.positions)
                .filter(|(v, _)| v.organ != 0)
                .map(|(_, p)| *p)
                .collect::<Vec<_>>()
        };
        assert_eq!(crown(&a), crown(&b));
        assert!(
            b.vertices[0].iter().filter(|v| v.organ == 0).count()
                > a.vertices[0].iter().filter(|v| v.organ == 0).count()
        );
        let restored: Plant = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(restored, p);
    }
    #[test]
    fn cords_continue_past_old_extent_and_taper_with_stem() {
        let base = super::super::sequoia_realistic::bole(3.664, 83);
        let p = Profile::default();
        let s = p.apply(&base, 3.664, 83).unwrap();
        let mut plain = s.clone();
        plain.relief.clear();
        let peak = |surface: &Surface, u: f64| {
            (0..768)
                .map(|j| {
                    let a = TAU * j as f64 / 768.;
                    (surface.radius(u, a) - plain.radius(u, a)).abs()
                })
                .fold(0_f64, f64::max)
        };
        for u in [0.3, 0.47, 0.48, 0.49, 0.6, 0.8, 0.9, 0.97] {
            assert!(peak(&s, u) > 0.001, "missing volume at {u}");
        }
        assert!(peak(&s, 0.9) < peak(&s, 0.3));
        assert!(peak(&s, 1.) < peak(&s, 0.97));
        let legacy = Profile {
            continuous: false,
            ..p
        };
        let old = legacy.apply(&base, 3.664, 83).unwrap();
        assert_eq!(peak(&old, 0.6), 0.);
        let mut json = serde_json::to_value(&legacy).unwrap();
        json.as_object_mut().unwrap().remove("continuous");
        let decoded: Profile = serde_json::from_value(json).unwrap();
        assert!(!decoded.continuous);
        assert_eq!(decoded.apply(&base, 3.664, 83).unwrap(), old);
    }
    #[test]
    fn bounded_seeded_volumes_and_zero_identity() {
        let p = Profile::default();
        for radius in [0.02, 0.046, 0.06, 1., 3.664, 4.1] {
            let base = super::super::sequoia_realistic::bole(radius, 83);
            let s = p
                .apply(&base, radius, 83)
                .unwrap_or_else(|e| panic!("radius {radius}: {e:?}"));
            assert_eq!(s, p.apply(&base, radius, 83).unwrap());
            assert!(
                s.sections.len() == 32 && s.relief.iter().any(|r| r.height < 0. || radius <= 0.05)
            );
        }
        let base = super::super::sequoia_realistic::bole(3.664, 83);
        let zero = Profile {
            bulge_ratio: 0.,
            cord_height_ratio: 0.,
            valley_depth_ratio: 0.,
            ..p.clone()
        };
        assert_eq!(zero.apply(&base, 3.664, 83).unwrap(), base);
        let a = p.apply(&base, 3.664, 83).unwrap();
        let b = p.apply(&base, 3.664, 84).unwrap();
        assert_ne!(a, b);
        let mut bad = p;
        bad.valley_depth_ratio = f64::NAN;
        assert!(bad.apply(&base, 3.664, 83).is_err());
    }
}
