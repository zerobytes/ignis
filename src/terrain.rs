//! Editable height surfaces. Authoring and generation share the same bounded kernel.
//! No renderer, physics, streaming or scene ownership is hidden in a surface query.
use crate::{
    model::{Result, invalid},
    world_space::{CELL_METERS, Coordinates, Position},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod coverage;
pub(crate) mod position_codec;
pub mod region;
pub mod surface;
pub const VERSION: u16 = 3;
pub const MAX_SAMPLES: usize = 66_049;
pub const MAX_OPERATIONS: usize = 64;
pub const MAX_WORK: usize = 16_000_000;
pub const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_HEIGHT: f64 = 1_000_000.;

/// Reusable bounded decoder, including untrusted CBOR arrays without a size hint.
pub(crate) fn limited<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct Visitor<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Visitor<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "an array with at most {N} entries")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            if a.size_hint().is_some_and(|n| n > N) {
                return Err(serde::de::Error::custom("Terrain array exceeds its limit"));
            }
            let mut out = Vec::with_capacity(a.size_hint().unwrap_or(0).min(N));
            while let Some(v) = a.next_element()? {
                if out.len() == N {
                    return Err(serde::de::Error::custom("Terrain array exceeds its limit"));
                }
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Visitor::<T, N>(std::marker::PhantomData))
}
fn heights<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<f32>, D::Error> {
    limited::<D, f32, MAX_SAMPLES>(d)
}
fn operations<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Operation>, D::Error> {
    limited::<D, Operation, MAX_OPERATIONS>(d)
}
fn points<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Position>, D::Error> {
    position_codec::deserialize_points(d)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    #[serde(with = "position_codec")]
    #[schemars(with = "Position")]
    pub origin: Position,
    /// Number of cells, not vertices. Samples are row-major, X varying fastest.
    pub cells: [u16; 2],
    pub spacing_meters: f64,
}
impl Patch {
    pub fn validate(&self) -> Result<usize> {
        let count = (usize::from(self.cells[0]) + 1) * (usize::from(self.cells[1]) + 1);
        if self.cells.contains(&0)
            || self.cells.iter().any(|&n| n > 256)
            || count > MAX_SAMPLES
            || !self.spacing_meters.is_finite()
            || !(0.125..=4096.).contains(&self.spacing_meters)
        {
            return Err(invalid(
                "Terrain patch needs1..256cells/axis, spacing0.125..4096m and at most66049samples",
            ));
        }
        self.point(self.cells[0], self.cells[1])?;
        Ok(count)
    }
    fn point(&self, x: u16, z: u16) -> Result<Position> {
        self.origin
            .translated([
                f64::from(x) * self.spacing_meters,
                0.,
                f64::from(z) * self.spacing_meters,
            ])
            .map_err(|e| invalid(e.to_string()))
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Base {
    Constant {
        height_meters: f64,
    },
    Samples {
        patch: Patch,
        #[serde(deserialize_with = "heights")]
        heights: Vec<f32>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NoiseKind {
    Smooth,
    Ridged,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    /// Compact oriented mountain chain; shape data belongs equally to authoring/generation.
    Ridge {
        #[serde(with = "position_codec")]
        #[schemars(with = "Position")]
        center: Position,
        azimuth_degrees: f64,
        width_meters: f64,
        depth_meters: f64,
        height_meters: f64,
        phase: f64,
        crest_wander: f64,
        slope_exponent: f64,
        folds_meters: f64,
        end_fade: f64,
    },
    /// A signed hill/depression; compact support, zero at the boundary.
    Raise {
        #[serde(with = "position_codec")]
        #[schemars(with = "Position")]
        center: Position,
        radius_meters: f64,
        amount_meters: f64,
    },
    /// Blend toward a plane, allowing both plateaus and ramps.
    Level {
        #[serde(with = "position_codec")]
        #[schemars(with = "Position")]
        center: Position,
        radius_meters: f64,
        height_meters: f64,
        slope: [f64; 2],
        blend: f64,
    },
    Noise {
        seed: u32,
        period_meters: u32,
        octaves: u8,
        persistence: f64,
        amplitude_meters: f64,
        style: NoiseKind,
    },
    /// Excavate toward an explicit descending bed. Hydrology/flow are separate.
    Channel {
        #[serde(
            deserialize_with = "points",
            serialize_with = "position_codec::serialize_points"
        )]
        points: Vec<Position>,
        width_meters: f64,
        bed_start_meters: f64,
        bed_end_meters: f64,
        blend: f64,
    },
    /// Common native field program, identical for planar and spherical surfaces.
    Surface { program: Box<surface::Program> },
    Clamp {
        minimum_meters: f64,
        maximum_meters: f64,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u16,
    pub space_version: u16,
    pub id: String,
    pub revision: u32,
    /// Vertical datum only. X/Z do not limit the extent of an unbounded field.
    #[serde(with = "position_codec")]
    #[schemars(with = "Position")]
    pub datum: Position,
    pub base: Base,
    #[serde(default, deserialize_with = "operations")]
    pub operations: Vec<Operation>,
    /// Geography metadata; not a water mesh, material or fluid simulation.
    #[serde(default)]
    pub water_level_meters: Option<f64>,
}
fn finite_height(v: f64) -> bool {
    v.is_finite() && v.abs() <= MAX_HEIGHT
}
fn radius(v: f64) -> bool {
    v.is_finite() && (0.125..=1_048_576.).contains(&v)
}
fn blend(v: f64) -> bool {
    v.is_finite() && (0. ..=1.).contains(&v)
}
pub(crate) fn id_valid(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-.".contains(&c))
}

/// Small local differences only: never round an absolute global coordinate.
fn horizontal(a: Position, b: Position, limit: f64) -> Option<[f64; 2]> {
    let a: Coordinates = a.into();
    let b: Coordinates = b.into();
    let mut out = [0.; 2];
    for (j, axis) in [0, 2].into_iter().enumerate() {
        let cells = i128::from(a.cell[axis]) - i128::from(b.cell[axis]);
        if cells.abs() > (limit / CELL_METERS).ceil() as i128 + 1 {
            return None;
        }
        out[j] = cells as f64 * CELL_METERS + (a.offset[axis] - b.offset[axis]);
        if out[j].abs() > limit {
            return None;
        }
    }
    Some(out)
}
fn same_datum(a: Position, b: Position) -> bool {
    let a: Coordinates = a.into();
    let b: Coordinates = b.into();
    a.cell[1] == b.cell[1] && a.offset[1] == b.offset[1]
}
impl Document {
    pub fn validate(&self) -> Result<()> {
        if !(1..=VERSION).contains(&self.version)
            || self.space_version != crate::world_space::VERSION
            || !id_valid(&self.id)
            || self.revision == 0
            || self.operations.len() > MAX_OPERATIONS
            || self.water_level_meters.is_some_and(|v| !finite_height(v))
        {
            return Err(invalid(
                "Invalid terrain version, space, ID, revision, operation budget or water level",
            ));
        }
        match &self.base {
            Base::Constant { height_meters } if !finite_height(*height_meters) => {
                return Err(invalid("Invalid terrain base height"));
            }
            Base::Samples { patch, heights }
                if patch.validate()? != heights.len()
                    || !same_datum(patch.origin, self.datum)
                    || heights.iter().any(|&h| !finite_height(f64::from(h))) =>
            {
                return Err(invalid("Terrain sample count, datum or height is invalid"));
            }
            _ => (),
        }
        let mut height_bounds = if self.version < 3 {
            [0.; 2]
        } else {
            match &self.base {
                Base::Constant { height_meters } => [*height_meters; 2],
                Base::Samples { heights, .. } => {
                    let b = [
                        heights.iter().copied().fold(f32::INFINITY, f32::min) as f64,
                        heights.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64,
                    ];
                    if b[0] == b[1] { b } else { surface::outward(b) }
                }
            }
        };
        for op in &self.operations {
            let valid = match op {
                Operation::Ridge {
                    azimuth_degrees,
                    width_meters,
                    depth_meters,
                    height_meters,
                    phase,
                    crest_wander,
                    slope_exponent,
                    folds_meters,
                    end_fade,
                    ..
                } => {
                    self.version >= 2
                        && azimuth_degrees.is_finite()
                        && azimuth_degrees.abs() <= 360.
                        && radius(*width_meters)
                        && radius(*depth_meters)
                        && finite_height(*height_meters)
                        && *height_meters > 0.
                        && phase.is_finite()
                        && (0. ..=10000.).contains(phase)
                        && crest_wander.is_finite()
                        && (0. ..=0.5).contains(crest_wander)
                        && slope_exponent.is_finite()
                        && (0.25..=4.).contains(slope_exponent)
                        && folds_meters.is_finite()
                        && (0. ..=1000.).contains(folds_meters)
                        && end_fade.is_finite()
                        && (0.01..=1.).contains(end_fade)
                }
                Operation::Raise {
                    radius_meters,
                    amount_meters,
                    ..
                } => radius(*radius_meters) && finite_height(*amount_meters),
                Operation::Level {
                    radius_meters,
                    height_meters,
                    slope,
                    blend: b,
                    ..
                } => {
                    radius(*radius_meters)
                        && finite_height(*height_meters)
                        && slope.iter().all(|v| v.is_finite() && v.abs() <= 100.)
                        && blend(*b)
                }
                Operation::Noise {
                    period_meters,
                    octaves,
                    persistence,
                    amplitude_meters,
                    ..
                } => {
                    period_meters.is_power_of_two()
                        && *period_meters <= 1_048_576
                        && (1..=12).contains(octaves)
                        && (*period_meters >> (octaves.saturating_sub(1))) > 0
                        && blend(*persistence)
                        && finite_height(*amplitude_meters)
                }
                Operation::Channel {
                    points,
                    width_meters,
                    bed_start_meters,
                    bed_end_meters,
                    blend: b,
                } => {
                    if !(2..=64).contains(&points.len())
                        || !radius(*width_meters)
                        || !finite_height(*bed_start_meters)
                        || !finite_height(*bed_end_meters)
                        || bed_end_meters > bed_start_meters
                        || !blend(*b)
                    {
                        false
                    } else {
                        points.windows(2).all(|w| {
                            horizontal(w[1], w[0], 1_048_576.)
                                .is_some_and(|d| d[0] * d[0] + d[1] * d[1] > 0.)
                        })
                    }
                }
                Operation::Surface { program } => {
                    if self.version < 3 {
                        false
                    } else {
                        program.validate_domain(surface::Domain::Plane { datum: self.datum })?;
                        program.validate_input(height_bounds)?;
                        true
                    }
                }
                Operation::Clamp {
                    minimum_meters,
                    maximum_meters,
                } => {
                    finite_height(*minimum_meters)
                        && finite_height(*maximum_meters)
                        && minimum_meters <= maximum_meters
                }
            };
            if !valid {
                return Err(invalid("Invalid terrain operation parameters"));
            }
            if self.version >= 3 {
                height_bounds = operation_bounds(op, height_bounds);
            }
        }
        Ok(())
    }
    fn work_per_point(&self) -> usize {
        1 + self
            .operations
            .iter()
            .map(|op| match op {
                Operation::Noise { octaves, .. } => 4 * usize::from(*octaves),
                Operation::Ridge { .. } => 16,
                Operation::Surface { program } => program.work_per_point(),
                Operation::Channel { points, .. } => points.len() * 2,
                _ => 1,
            })
            .sum::<usize>()
    }
    fn budget(&self, points: usize) -> Result<()> {
        if points > MAX_SAMPLES || self.work_per_point().saturating_mul(points) > MAX_WORK {
            return Err(invalid("Terrain sampling exceeds point/work budget"));
        }
        Ok(())
    }
}
fn operation_bounds(op: &Operation, h: [f64; 2]) -> [f64; 2] {
    let bounds = match op {
        Operation::Surface { program } => return program.range_meters,
        Operation::Raise {
            amount_meters: a, ..
        } => [h[0] + a.min(0.), h[1] + a.max(0.)],
        Operation::Noise {
            amplitude_meters: a,
            ..
        } => [h[0] - a.abs(), h[1] + a.abs()],
        Operation::Ridge {
            height_meters,
            folds_meters,
            ..
        } => [h[0], h[1] + height_meters * 1.09 + folds_meters],
        Operation::Level {
            radius_meters,
            height_meters,
            slope,
            ..
        } => {
            let d = radius_meters * slope[0].hypot(slope[1]);
            [h[0].min(height_meters - d), h[1].max(height_meters + d)]
        }
        Operation::Channel { bed_end_meters, .. } => [h[0].min(*bed_end_meters), h[1]],
        Operation::Clamp {
            minimum_meters,
            maximum_meters,
        } => return h.map(|v| v.clamp(*minimum_meters, *maximum_meters)),
    };
    surface::outward(bounds)
}
fn smooth(t: f64) -> f64 {
    t * t * t * (t * (t * 6. - 15.) + 10.)
}
fn weight(d: [f64; 2], r: f64) -> f64 {
    1. - smooth((d[0].hypot(d[1]) / r).min(1.))
}
#[allow(clippy::too_many_arguments)]
fn ridge_profile(
    u: f64,
    v: f64,
    height_meters: f64,
    phase: f64,
    crest_wander: f64,
    slope_exponent: f64,
    folds_meters: f64,
    end_fade: f64,
) -> Option<f64> {
    if u.abs() < 1. {
        let crest = crest_wander * (u * 6. + phase).sin();
        let t = (1. - (v - crest).abs()).max(0.);
        if t > 0. {
            let profile = 0.76
                + 0.18 * (u * 5. + phase).sin()
                + 0.10 * (u * 13. - phase).sin()
                + 0.05 * (u * 31. + phase).sin();
            let folds =
                folds_meters * (u * 47. + v * 19. + phase).sin() * (v * 17. - u * 11.).sin();
            return Some(
                (height_meters * profile + folds).max(0.)
                    * t.powf(slope_exponent)
                    * smooth(((1. - u.abs()) / end_fade).min(1.)),
            );
        }
    }
    None
}
fn mix(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
fn hash(mut v: u64) -> u64 {
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
    v ^ (v >> 31)
}
fn lattice(p: Position, axis: usize, period: u32) -> (i128, f64) {
    let p: Coordinates = p.into();
    let meters = i128::from(p.cell[axis]) * 1024;
    let period_i = i128::from(period);
    let offset_cell = (p.offset[axis] / f64::from(period)).floor();
    (
        meters.div_euclid(period_i) + offset_cell as i128,
        (meters.rem_euclid(period_i) as f64 + p.offset[axis] - offset_cell * f64::from(period))
            / f64::from(period),
    )
}
fn random(x: i128, z: i128, seed: u32) -> f64 {
    let fold = |i: i128| hash(i as u64 ^ hash((i >> 64) as u64));
    let n = hash(fold(x) ^ fold(z).rotate_left(27) ^ u64::from(seed));
    ((n >> 11) as f64 * (1. / ((1u64 << 53) as f64))) * 2. - 1.
}
fn noise(p: Position, period: u32, seed: u32) -> f64 {
    let (x, tx) = lattice(p, 0, period);
    let (z, tz) = lattice(p, 2, period);
    mix(
        mix(random(x, z, seed), random(x + 1, z, seed), smooth(tx)),
        mix(
            random(x, z + 1, seed),
            random(x + 1, z + 1, seed),
            smooth(tx),
        ),
        smooth(tz),
    )
}
// Shared diagonal is b--c. Interpolation is the exact triangulated surface.
fn triangle(a: f64, b: f64, c: f64, d: f64, x: f64, z: f64) -> f64 {
    if x + z <= 1. {
        a + (b - a) * x + (c - a) * z
    } else {
        d + (c - d) * (1. - x) + (b - d) * (1. - z)
    }
}
fn sample_grid(patch: &Patch, heights: &[f32], p: Position) -> Option<f64> {
    let extent = patch.cells.map(|n| f64::from(n) * patch.spacing_meters);
    let d = horizontal(p, patch.origin, extent[0].max(extent[1]))?;
    if d[0] < 0. || d[1] < 0. || d[0] > extent[0] || d[1] > extent[1] {
        return None;
    }
    let x = d[0] / patch.spacing_meters;
    let z = d[1] / patch.spacing_meters;
    let ix = (x.floor() as usize).min(usize::from(patch.cells[0]) - 1);
    let iz = (z.floor() as usize).min(usize::from(patch.cells[1]) - 1);
    let w = usize::from(patch.cells[0]) + 1;
    let i = iz * w + ix;
    Some(triangle(
        f64::from(heights[i]),
        f64::from(heights[i + 1]),
        f64::from(heights[i + w]),
        f64::from(heights[i + w + 1]),
        x - ix as f64,
        z - iz as f64,
    ))
}
struct Segment {
    start: Position,
    delta: [f64; 2],
    length: f64,
    prefix: f64,
}
pub struct Sampler<'a> {
    doc: &'a Document,
    channels: Vec<Vec<Segment>>,
    orientations: Vec<(f64, f64)>,
    programs: Vec<surface::Compiled>,
    program_slots: [u8; MAX_OPERATIONS],
}
impl<'a> Sampler<'a> {
    pub fn new(doc: &'a Document) -> Result<Self> {
        doc.validate()?;
        let channels = doc
            .operations
            .iter()
            .map(|op| {
                let mut segments = Vec::new();
                if let Operation::Channel { points, .. } = op {
                    let mut prefix = 0.;
                    for w in points.windows(2) {
                        let delta = horizontal(w[1], w[0], 1_048_576.).expect("validated channel");
                        let length = delta[0].hypot(delta[1]);
                        segments.push(Segment {
                            start: w[0],
                            delta,
                            length,
                            prefix,
                        });
                        prefix += length;
                    }
                }
                segments
            })
            .collect();
        let orientations = doc
            .operations
            .iter()
            .map(|op| match op {
                Operation::Ridge {
                    azimuth_degrees, ..
                } => azimuth_degrees.to_radians().sin_cos(),
                _ => (0., 1.),
            })
            .collect();
        let mut programs = Vec::new();
        let mut program_slots = [u8::MAX; MAX_OPERATIONS];
        for (i, op) in doc.operations.iter().enumerate() {
            if let Operation::Surface { program } = op {
                program_slots[i] = programs.len() as u8;
                programs.push(program.compile(surface::Domain::Plane { datum: doc.datum })?);
            }
        }
        Ok(Self {
            doc,
            channels,
            orientations,
            programs,
            program_slots,
        })
    }
    pub fn sample(&self, p: Position) -> Result<Option<f64>> {
        let mut h = match &self.doc.base {
            Base::Constant { height_meters } => *height_meters,
            Base::Samples { patch, heights } => match sample_grid(patch, heights, p) {
                Some(h) => h,
                None => return Ok(None),
            },
        };
        for (i, op) in self.doc.operations.iter().enumerate() {
            match op {
                Operation::Ridge {
                    center,
                    width_meters,
                    depth_meters,
                    height_meters,
                    phase,
                    crest_wander,
                    slope_exponent,
                    folds_meters,
                    end_fade,
                    ..
                } => {
                    let range =
                        (width_meters * 0.5).hypot(depth_meters * 0.5 * (1. + crest_wander));
                    if let Some(d) = horizontal(p, *center, range) {
                        let (sin, cos) = self.orientations[i];
                        let u = 2. * (cos * d[0] + sin * d[1]) / width_meters;
                        let v = 2. * (-sin * d[0] + cos * d[1]) / depth_meters;
                        if let Some(delta) = ridge_profile(
                            u,
                            v,
                            *height_meters,
                            *phase,
                            *crest_wander,
                            *slope_exponent,
                            *folds_meters,
                            *end_fade,
                        ) {
                            h += delta;
                        }
                    }
                }
                Operation::Raise {
                    center,
                    radius_meters,
                    amount_meters,
                } => {
                    if let Some(d) = horizontal(p, *center, *radius_meters) {
                        h += amount_meters * weight(d, *radius_meters)
                    }
                }
                Operation::Level {
                    center,
                    radius_meters,
                    height_meters,
                    slope,
                    blend,
                    ..
                } => {
                    if let Some(d) = horizontal(p, *center, *radius_meters) {
                        h = mix(
                            h,
                            height_meters + slope[0] * d[0] + slope[1] * d[1],
                            blend * weight(d, *radius_meters),
                        )
                    }
                }
                Operation::Noise {
                    seed,
                    period_meters,
                    octaves,
                    persistence,
                    amplitude_meters,
                    style,
                } => {
                    let mut amplitude = 1.;
                    let mut total = 0.;
                    let mut norm = 0.;
                    for octave in 0..*octaves {
                        let n = noise(
                            p,
                            *period_meters >> octave,
                            seed.wrapping_add(u32::from(octave).wrapping_mul(0x9e3779b9)),
                        );
                        total += amplitude
                            * match style {
                                NoiseKind::Smooth => n,
                                NoiseKind::Ridged => 1. - 2. * n.abs(),
                            };
                        norm += amplitude;
                        amplitude *= persistence;
                    }
                    h += amplitude_meters * (total / norm);
                }
                Operation::Channel {
                    width_meters,
                    bed_start_meters,
                    bed_end_meters,
                    blend,
                    ..
                } => {
                    let segments = &self.channels[i];
                    let last = segments.last().expect("validated channel");
                    let total = last.prefix + last.length;
                    let mut best: Option<(f64, f64)> = None;
                    for s in segments {
                        if let Some(d) = horizontal(p, s.start, s.length + width_meters) {
                            let t = ((d[0] * s.delta[0] + d[1] * s.delta[1])
                                / (s.length * s.length))
                                .clamp(0., 1.);
                            let distance = (d[0] - t * s.delta[0]).hypot(d[1] - t * s.delta[1]);
                            if distance <= *width_meters
                                && best.is_none_or(|(old, _)| distance < old)
                            {
                                best = Some((distance, (s.prefix + t * s.length) / total));
                            }
                        }
                    }
                    if let Some((distance, t)) = best {
                        let bed = mix(*bed_start_meters, *bed_end_meters, t);
                        h = mix(
                            h,
                            h.min(bed),
                            blend * (1. - smooth(distance / width_meters)),
                        );
                    }
                }
                Operation::Surface { .. } => {
                    h = self.programs[usize::from(self.program_slots[i])].sample_with_input(p, h)?
                }
                Operation::Clamp {
                    minimum_meters,
                    maximum_meters,
                } => h = h.clamp(*minimum_meters, *maximum_meters),
            }
            if !finite_height(h) {
                return Err(invalid(
                    "Terrain result exceeds finite height range; no partial result published",
                ));
            }
        }
        Ok(Some(h))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    Append {
        operation: Operation,
    },
    Replace {
        index: u8,
        operation: Operation,
    },
    Remove {
        index: u8,
    },
    SetBase {
        base: Base,
    },
    SetWater {
        height_meters: Option<f64>,
    },
    PatchSamples {
        start: [u16; 2],
        size: [u16; 2],
        #[serde(deserialize_with = "heights")]
        heights: Vec<f32>,
    },
}
pub fn edit(mut document: Document, expected_revision: u32, edits: Vec<Edit>) -> Result<Document> {
    document.validate()?;
    if document.revision != expected_revision || edits.is_empty() || edits.len() > 64 {
        return Err(invalid("Terrain revision conflict or invalid edit batch"));
    }
    let next = document
        .revision
        .checked_add(1)
        .ok_or_else(|| invalid("Terrain revision exhausted"))?;
    for edit in edits {
        match edit {
            Edit::Append { operation } => document.operations.push(operation),
            Edit::Replace { index, operation } => {
                *document
                    .operations
                    .get_mut(usize::from(index))
                    .ok_or_else(|| invalid("Missing terrain operation"))? = operation
            }
            Edit::Remove { index } => {
                if usize::from(index) >= document.operations.len() {
                    return Err(invalid("Missing terrain operation"));
                }
                document.operations.remove(usize::from(index));
            }
            Edit::SetBase { base } => document.base = base,
            Edit::SetWater { height_meters } => document.water_level_meters = height_meters,
            Edit::PatchSamples {
                start,
                size,
                heights,
            } => {
                let Base::Samples {
                    patch,
                    heights: target,
                } = &mut document.base
                else {
                    return Err(invalid("Sample patch requires a sampled base"));
                };
                if size.contains(&0)
                    || heights.len() != usize::from(size[0]) * usize::from(size[1])
                    || (0..2).any(|a| {
                        u32::from(start[a]) + u32::from(size[a]) > u32::from(patch.cells[a]) + 1
                    })
                {
                    return Err(invalid("Sample patch exceeds the source grid"));
                }
                let width = usize::from(patch.cells[0]) + 1;
                for z in 0..usize::from(size[1]) {
                    let at = (usize::from(start[1]) + z) * width + usize::from(start[0]);
                    let from = z * usize::from(size[0]);
                    target[at..at + usize::from(size[0])]
                        .copy_from_slice(&heights[from..from + usize::from(size[0])]);
                }
            }
        }
        document.validate()?;
    }
    document.revision = next;
    Ok(document)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub patch: Patch,
    #[serde(deserialize_with = "heights")]
    pub heights: Vec<f32>,
}
pub fn bake(document: &Document, patch: Patch) -> Result<Grid> {
    let count = patch.validate()?;
    document.budget(count)?;
    if !same_datum(document.datum, patch.origin) {
        return Err(invalid(
            "Terrain patch and source must share the vertical datum",
        ));
    }
    let prepared = Sampler::new(document)?;
    let mut heights = Vec::with_capacity(count);
    for z in 0..=patch.cells[1] {
        for x in 0..=patch.cells[0] {
            let h = prepared
                .sample(patch.point(x, z)?)?
                .ok_or_else(|| invalid("Terrain patch extends outside the fixed source"))?;
            heights.push(h as f32);
        }
    }
    Ok(Grid { patch, heights })
}
#[derive(Debug, Clone, Serialize)]
pub struct Compiled {
    pub version: u16,
    pub source_id: String,
    pub source_revision: u32,
    pub source_sha256: String,
    pub patch: Patch,
    pub origin: Position,
    pub geometry: std::sync::Arc<crate::meshes::Geometry>,
    /// Against the sampled full-resolution field only; NOT analytic/noise error.
    pub sampled_error_meters: f64,
    pub stride: u16,
    pub mixed_stride_edges_require_stitching: bool,
    pub bounds: [[f32; 3]; 2],
}
pub fn compile(document: &Document, patch: Patch, stride: u16) -> Result<Compiled> {
    if stride == 0 || !stride.is_power_of_two() || patch.cells.iter().any(|n| n % stride != 0) {
        return Err(invalid(
            "Terrain stride must divide both patch dimensions and be a power of two",
        ));
    }
    let cells = patch.cells.map(|n| n / stride);
    let vertices = (usize::from(cells[0]) + 1) * (usize::from(cells[1]) + 1);
    let faces = usize::from(cells[0]) * usize::from(cells[1]) * 2;
    if vertices > crate::meshes::MAX_VERTICES || faces > crate::meshes::MAX_TRIANGLES {
        return Err(invalid(
            "Compiled terrain exceeds native mesh budget; request a smaller patch",
        ));
    }
    let grid = bake(document, patch.clone())?;
    let width = usize::from(patch.cells[0]) + 1;
    let mut positions = Vec::with_capacity(vertices);
    let mut triangles = Vec::with_capacity(faces);
    let minimum = grid.heights.iter().copied().fold(f32::INFINITY, f32::min);
    let maximum = grid
        .heights
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    for z in 0..=cells[1] {
        for x in 0..=cells[0] {
            let h = grid.heights[usize::from(z * stride) * width + usize::from(x * stride)];
            positions.push([
                (f64::from(x * stride) * patch.spacing_meters) as f32,
                h,
                (f64::from(z * stride) * patch.spacing_meters) as f32,
            ]);
        }
    }
    let w = u32::from(cells[0]) + 1;
    for z in 0..u32::from(cells[1]) {
        for x in 0..u32::from(cells[0]) {
            let a = z * w + x;
            triangles.extend([[a, a + w, a + 1], [a + 1, a + w, a + w + 1]]);
        }
    }
    let coarse = Patch {
        spacing_meters: patch.spacing_meters * f64::from(stride),
        cells,
        ..patch.clone()
    };
    let coarse_heights: Vec<_> = positions.iter().map(|p| p[1]).collect();
    let mut error = 0f64;
    for z in 0..=patch.cells[1] {
        for x in 0..=patch.cells[0] {
            let h = sample_grid(&coarse, &coarse_heights, patch.point(x, z)?)
                .ok_or_else(|| invalid("Terrain approximation outside its patch"))?;
            error = error
                .max((f64::from(grid.heights[usize::from(z) * width + usize::from(x)]) - h).abs());
        }
    }
    let geometry = crate::meshes::Geometry {
        positions,
        triangles,
        material_coordinates: None,
    };
    geometry.validate()?;
    Ok(Compiled {
        version: document.version,
        source_id: document.id.clone(),
        source_revision: document.revision,
        source_sha256: source_digest(document)?,
        patch: patch.clone(),
        origin: patch.origin,
        geometry: std::sync::Arc::new(geometry),
        sampled_error_meters: error,
        stride,
        mixed_stride_edges_require_stitching: stride > 1,
        bounds: [
            [0., minimum, 0.],
            [
                (f64::from(patch.cells[0]) * patch.spacing_meters) as f32,
                maximum,
                (f64::from(patch.cells[1]) * patch.spacing_meters) as f32,
            ],
        ],
    })
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    PrepareSurface {
        draft: surface::Draft,
    },
    InspectRegion {
        document: Document,
        region: region::Region,
    },
    Inspect {
        document: Document,
    },
    Sample {
        document: Document,
        #[serde(deserialize_with = "sample_points")]
        points: Vec<Position>,
    },
    Edit {
        document: Document,
        expected_revision: u32,
        #[serde(deserialize_with = "edit_list")]
        edits: Vec<Edit>,
    },
    Bake {
        document: Document,
        patch: Patch,
    },
    Compile {
        document: Document,
        patch: Patch,
        stride: u16,
    },
}
fn sample_points<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Position>, D::Error> {
    limited::<D, Position, MAX_SAMPLES>(d)
}
fn edit_list<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Edit>, D::Error> {
    limited::<D, Edit, 64>(d)
}
pub fn query(query: Query) -> Result<serde_json::Value> {
    match query {
        Query::PrepareSurface { draft } => Ok(serde_json::json!({"program":draft.prepare()?})),
        Query::InspectRegion { document, region } => {
            Ok(region::compile(&document, region)?.report())
        }
        Query::Inspect { document } => {
            document.validate()?;
            Ok(
                serde_json::json!({"document":document,"limits":{"samples":MAX_SAMPLES,"operations":MAX_OPERATIONS,"work":MAX_WORK},"collision":false,"streaming":false}),
            )
        }
        Query::Sample { document, points } => {
            document.budget(points.len())?;
            let prepared = Sampler::new(&document)?;
            let mut out = Vec::with_capacity(points.len());
            for point in points {
                let h = prepared.sample(point)?;
                out.push(serde_json::json!({"point":point,"height_meters":h,"water_depth_meters":h.zip(document.water_level_meters).map(|(h,w)|(w-h).max(0.))}));
            }
            Ok(serde_json::json!({"samples":out,"source_revision":document.revision}))
        }
        Query::Edit {
            document,
            expected_revision,
            edits,
        } => Ok(serde_json::json!({"document":edit(document,expected_revision,edits)?})),
        Query::Bake { document, patch } => {
            let grid = bake(&document, patch)?;
            let frozen = Document {
                revision: document
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| invalid("Terrain revision exhausted"))?,
                base: Base::Samples {
                    patch: grid.patch,
                    heights: grid.heights,
                },
                operations: vec![],
                ..document
            };
            Ok(serde_json::json!({"document":frozen,"analytic_source_frozen":true}))
        }
        Query::Compile {
            document,
            patch,
            stride,
        } => serde_json::to_value(compile(&document, patch, stride)?)
            .map_err(|e| invalid(e.to_string())),
    }
}

pub(crate) fn source_digest(document: &impl Serialize) -> Result<String> {
    struct DigestWriter(ring::digest::Context);
    impl std::io::Write for DigestWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = DigestWriter(ring::digest::Context::new(&ring::digest::SHA256));
    ciborium::ser::into_writer(document, &mut writer).map_err(|e| invalid(e.to_string()))?;
    use std::fmt::Write;
    let mut hex = String::with_capacity(64);
    for byte in writer.0.finish().as_ref() {
        write!(&mut hex, "{byte:02x}").expect("writing to String");
    }
    Ok(hex)
}

/// Authoring envelope: typed CBOR, SHA256, bounded size/depth and exact EOF.
pub fn encode(document: &Document) -> Result<Vec<u8>> {
    document.validate()?;
    encode_asset(b"IGNISTER", document.version, document)
}
pub(crate) fn encode_asset(
    magic: &[u8; 8],
    version: u16,
    document: &impl Serialize,
) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    ciborium::ser::into_writer(document, &mut payload).map_err(|e| invalid(e.to_string()))?;
    if payload.len() > MAX_BYTES - 50 {
        return Err(invalid("Terrain file exceeds8MiB"));
    }
    let digest = ring::digest::digest(&ring::digest::SHA256, &payload);
    let mut out = Vec::with_capacity(50 + payload.len());
    out.extend_from_slice(magic);
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(digest.as_ref());
    out.extend_from_slice(&payload);
    Ok(out)
}
pub fn decode(bytes: &[u8]) -> Result<Document> {
    let version = bytes
        .get(8..10)
        .map(|v| u16::from_le_bytes(v.try_into().unwrap()))
        .ok_or_else(|| invalid("Invalid terrain header"))?;
    if !(1..=VERSION).contains(&version) {
        return Err(invalid("Unsupported terrain format"));
    }
    let doc: Document = decode_asset(bytes, b"IGNISTER", version)?;
    if doc.version != version {
        return Err(invalid("Terrain envelope/document version mismatch"));
    }
    doc.validate()?;
    Ok(doc)
}
pub(crate) fn decode_asset<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    magic: &[u8; 8],
    version: u16,
) -> Result<T> {
    if bytes.len() < 50
        || bytes.len() > MAX_BYTES
        || &bytes[..8] != magic
        || u16::from_le_bytes(bytes[8..10].try_into().unwrap()) != version
        || u64::from_le_bytes(bytes[10..18].try_into().unwrap()) != (bytes.len() - 50) as u64
    {
        return Err(invalid("Invalid terrain envelope/version/length"));
    }
    let payload = &bytes[50..];
    let digest = ring::digest::digest(&ring::digest::SHA256, payload);
    if digest.as_ref() != &bytes[18..50] {
        return Err(invalid("Terrain checksum mismatch"));
    }
    let mut reader = std::io::Cursor::new(payload);
    let doc: T = ciborium::de::from_reader_with_recursion_limit(&mut reader, 64)
        .map_err(|e| invalid(e.to_string()))?;
    if reader.position() != payload.len() as u64 {
        return Err(invalid("Trailing terrain payload"));
    }
    Ok(doc)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    /// None preserves an existing binding; true/false explicitly enable/disable.
    #[serde(default)]
    pub collision: Option<bool>,
    pub document: Document,
    pub patch: Patch,
    pub stride: u16,
    pub asset: String,
    pub instance: String,
    /// Explicit projection only, not a change of the live world's origin.
    pub frame_origin: Position,
    pub color: [f32; 3],
    #[serde(default)]
    pub material: crate::materials::Material,
}
pub fn publication(
    p: Publication,
) -> Result<(crate::meshes::Asset, crate::meshes::Instance, Compiled)> {
    let compiled = compile(&p.document, p.patch, p.stride)?;
    let local = compiled
        .origin
        .project(p.frame_origin, 10_000., 0.001)
        .map_err(|e| invalid(e.to_string()))?;
    let asset = crate::meshes::Asset {
        id: p.asset.clone(),
        geometry: std::sync::Arc::clone(&compiled.geometry),
        shading: crate::meshes::Shading::Smooth,
        material: p.material,
        rig: None,
        clips: vec![],
    };
    let instance = crate::meshes::Instance {
        id: p.instance,
        asset: p.asset,
        position: local,
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: p.color,
        playback: None,
        attachment: None,
    };
    Ok((asset, instance, compiled))
}
