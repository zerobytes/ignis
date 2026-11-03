//! Morphology helpers. Output is a normal editable profile/section document.
use super::*;
use std::f64::consts::{PI, TAU};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Preset {
    WaveCliff {
        height_meters: f64,
        depth_meters: f64,
        inset_meters: f64,
        lip_meters: f64,
        length_meters: f64,
        segments: u8,
    },
    Canyon {
        height_meters: f64,
        outer_width_meters: f64,
        floor_width_meters: f64,
        base_depth_meters: f64,
        length_meters: f64,
    },
    BentPeak {
        height_meters: f64,
        radius_meters: f64,
        lean: [f64; 2],
        levels: u8,
        sides: u8,
        seed: u32,
        roughness: f64,
    },
    Mesa {
        height_meters: f64,
        radius_meters: f64,
        foot_radius_meters: f64,
        foot_height_meters: f64,
        sides: u8,
        seed: u32,
        roughness: f64,
    },
    Hoodoo {
        height_meters: f64,
        radius_meters: f64,
        sides: u8,
        seed: u32,
        roughness: f64,
    },
    Arch {
        outer_radius_meters: f64,
        inner_radius_meters: f64,
        leg_height_meters: f64,
        depth_meters: f64,
        segments: u8,
    },
    Tunnel {
        radius_meters: f64,
        inner_ratio: f64,
        length_meters: f64,
        sides: u8,
        seed: u32,
        roughness: f64,
    },
}
fn dimension(v: f64) -> bool {
    v.is_finite() && (0.001..=1000.).contains(&v)
}
fn radial(sides: u8, seed: u32, roughness: f64) -> Result<Vec<[f64; 2]>> {
    if !(3..=64).contains(&sides) || !roughness.is_finite() || !(0. ..=0.35).contains(&roughness) {
        return Err(invalid(
            "Radial landform requires3..64sides and roughness0..0.35",
        ));
    }
    Ok((0..sides)
        .map(|i| {
            let mut h = u64::from(seed) ^ u64::from(i).wrapping_mul(0x9e3779b97f4a7c15);
            h = (h ^ (h >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            h = (h ^ (h >> 27)).wrapping_mul(0x94d049bb133111eb);
            h ^= h >> 31;
            let radius = 1. + roughness * (((h >> 11) as f64 / ((1u64 << 53) as f64)) * 2. - 1.);
            let a = TAU * f64::from(i) / f64::from(sides);
            [radius * a.cos(), radius * a.sin()]
        })
        .collect())
}
fn section(at: f64, scale: f64) -> Section {
    Section {
        at,
        offset: [0.; 2],
        scale,
    }
}
fn extrusion(id: String, profile: Vec<[f64; 2]>, length: f64) -> Piece {
    Piece {
        id,
        anchor: [0.; 3],
        axis: [0., 0., 1.],
        profile_x: [1., 0., 0.],
        profile,
        sections: vec![section(0., 1.), section(length, 1.)],
        inner_scale: None,
    }
}
pub(super) fn create(id: String, origin: Position, preset: Preset) -> Result<Document> {
    let mut p = match preset {
        Preset::WaveCliff {
            height_meters: h,
            depth_meters: d,
            inset_meters: inset,
            lip_meters: lip,
            length_meters: length,
            segments,
        } => {
            if ![h, d, length].into_iter().all(dimension)
                || !inset.is_finite()
                || !(0. ..d * 0.8).contains(&inset)
                || !lip.is_finite()
                || !(0. ..=1000.).contains(&lip)
                || !(2..=64).contains(&segments)
            {
                return Err(invalid("Invalid wave-cliff dimensions or resolution"));
            }
            let mut profile = Vec::with_capacity(usize::from(segments) + 3);
            profile.push([-d, 0.]);
            profile.push([0., 0.]);
            for i in 1..=segments {
                let t = f64::from(i) / f64::from(segments);
                profile.push([-inset * (PI * t).sin() + lip * t * t * t, h * t]);
            }
            profile.push([-d, h]);
            extrusion(id.clone(), profile, length)
        }
        Preset::Canyon {
            height_meters: h,
            outer_width_meters: w,
            floor_width_meters: floor,
            base_depth_meters: base,
            length_meters: length,
        } => {
            if ![h, w, floor, base, length].into_iter().all(dimension) || floor >= w {
                return Err(invalid(
                    "Canyon needs positive dimensions and floor narrower than outer width",
                ));
            }
            let a = w / 2.;
            let b = floor / 2.;
            extrusion(
                id.clone(),
                vec![
                    [-a, -base],
                    [a, -base],
                    [a, h],
                    [b, h],
                    [b, 0.],
                    [-b, 0.],
                    [-b, h],
                    [-a, h],
                ],
                length,
            )
        }
        Preset::BentPeak {
            height_meters: h,
            radius_meters: r,
            lean,
            levels,
            sides,
            seed,
            roughness,
        } => {
            if ![h, r].into_iter().all(dimension)
                || !finite(&lean, 1000.)
                || !(2..=96).contains(&levels)
            {
                return Err(invalid("Invalid bent-peak dimensions/resolution"));
            }
            let mut p = extrusion(id.clone(), radial(sides, seed, roughness)?, h);
            p.axis = [0., 1., 0.];
            p.sections = (0..=levels)
                .map(|i| {
                    let t = f64::from(i) / f64::from(levels);
                    Section {
                        at: h * t,
                        offset: lean.map(|v| v * t * t),
                        scale: r * (1. - t),
                    }
                })
                .collect();
            p
        }
        Preset::Mesa {
            height_meters: h,
            radius_meters: r,
            foot_radius_meters: foot,
            foot_height_meters: base,
            sides,
            seed,
            roughness,
        } => {
            if ![h, r, foot, base].into_iter().all(dimension) || foot < r || base >= h {
                return Err(invalid("Mesa needs a wider foot and foot height below top"));
            }
            let mut p = extrusion(id.clone(), radial(sides, seed, roughness)?, h);
            p.axis = [0., 1., 0.];
            p.sections = vec![section(0., foot), section(base, r), section(h, r)];
            p
        }
        Preset::Hoodoo {
            height_meters: h,
            radius_meters: r,
            sides,
            seed,
            roughness,
        } => {
            if ![h, r].into_iter().all(dimension) {
                return Err(invalid("Invalid hoodoo dimensions"));
            }
            let mut p = extrusion(id.clone(), radial(sides, seed, roughness)?, h);
            p.axis = [0., 1., 0.];
            p.sections = vec![
                section(0., r),
                section(h * 0.15, r * 0.65),
                section(h * 0.65, r * 0.5),
                section(h * 0.65, r * 1.3),
                section(h * 0.85, r * 1.3),
                section(h, r * 0.9),
            ];
            p
        }
        Preset::Arch {
            outer_radius_meters: r,
            inner_radius_meters: inner,
            leg_height_meters: leg,
            depth_meters: depth,
            segments,
        } => {
            if ![r, inner, leg, depth].into_iter().all(dimension)
                || inner >= r
                || !(4..=44).contains(&segments)
            {
                return Err(invalid("Invalid arch radii/leg/depth or segments4..44"));
            }
            let mut profile = Vec::with_capacity(usize::from(segments) * 2 + 6);
            profile.extend([[-r, 0.], [-inner, 0.], [-inner, leg]]);
            for i in 1..=segments {
                let a = PI * (1. - f64::from(i) / f64::from(segments));
                profile.push([inner * a.cos(), leg + inner * a.sin()]);
            }
            profile.extend([[inner, 0.], [r, 0.], [r, leg]]);
            for i in 1..=segments {
                let a = PI * f64::from(i) / f64::from(segments);
                profile.push([r * a.cos(), leg + r * a.sin()]);
            }
            extrusion(id.clone(), profile, depth)
        }
        Preset::Tunnel {
            radius_meters: r,
            inner_ratio: inner,
            length_meters: length,
            sides,
            seed,
            roughness,
        } => {
            if ![r, length].into_iter().all(dimension) {
                return Err(invalid("Invalid tunnel radius/length"));
            }
            let mut p = extrusion(id.clone(), radial(sides, seed, roughness)?, length);
            p.sections = vec![section(0., r), section(length, r)];
            p.inner_scale = Some(inner);
            p
        }
    };
    // Keep all sources explicit; a preset does not add a private evaluator.
    p.id = id.clone();
    let document = Document {
        version: VERSION,
        space_version: crate::world_space::VERSION,
        id,
        revision: 1,
        origin,
        pieces: vec![p],
    };
    document.validate()?;
    Ok(document)
}
