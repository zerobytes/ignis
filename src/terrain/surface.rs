//! One editable relief program for every surface domain. Native slots, no JSON/heap per sample.
mod domain;
use crate::{
    model::{Result, invalid},
    world_space::Position,
};
pub use domain::Domain;
use domain::{Anchor, Point, Segment};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VERSION: u16 = 1;
pub const MAX_NODES: usize = 128;
const MAX_CURVE: usize = 64;
const MAX_KNOTS: usize = 32;
const MAX_VALUE: f64 = 1e12;
fn nodes<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Node>, D::Error> {
    super::limited::<D, Node, MAX_NODES>(d)
}
fn knots<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Knot>, D::Error> {
    super::limited::<D, Knot, MAX_KNOTS>(d)
}
fn points<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Position>, D::Error> {
    super::position_codec::deserialize_points(d)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub version: u16,
    #[serde(deserialize_with = "nodes")]
    pub nodes: Vec<Node>,
    pub output: u16,
    /// Checked enclosure, available to culling without recompiling the graph per tile.
    pub range_meters: [f64; 2],
}
/// Authoring draft: the shared compiler derives a conservative range for MCP/other callers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "nodes")]
    pub nodes: Vec<Node>,
    pub output: u16,
}
impl Draft {
    pub fn prepare(self) -> Result<Program> {
        Program::new(self.nodes, self.output)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Knot {
    pub x: f64,
    pub value: f64,
    pub derivative: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Node {
    /// Accumulated source height before this program. Its declaration is checked against the source.
    Input {
        minimum_meters: f64,
        maximum_meters: f64,
    },
    Constant {
        value: f64,
    },
    Add {
        a: u16,
        b: u16,
    },
    Subtract {
        a: u16,
        b: u16,
    },
    Multiply {
        a: u16,
        b: u16,
    },
    Minimum {
        a: u16,
        b: u16,
    },
    Maximum {
        a: u16,
        b: u16,
    },
    SmoothMinimum {
        a: u16,
        b: u16,
        transition_meters: f64,
    },
    SmoothMaximum {
        a: u16,
        b: u16,
        transition_meters: f64,
    },
    /// Mask is clamped to0..1. Scalars may represent altitude, masks or regional selectors.
    Mix {
        a: u16,
        b: u16,
        mask: u16,
    },
    Clamp {
        input: u16,
        minimum: f64,
        maximum: f64,
    },
    /// Cubic Hermite remapping, constant extrapolation outside the first/last knot.
    Remap {
        input: u16,
        #[serde(deserialize_with = "knots")]
        knots: Vec<Knot>,
    },
    Noise {
        seed: u32,
        period_meters: u32,
        octaves: u8,
        persistence: f64,
        amplitude_meters: f64,
        style: super::NoiseKind,
    },
    Raise {
        #[serde(with = "super::position_codec")]
        #[schemars(with = "Position")]
        center: Position,
        radius_meters: f64,
        amount_meters: f64,
    },
    Level {
        input: u16,
        #[serde(with = "super::position_codec")]
        #[schemars(with = "Position")]
        center: Position,
        radius_meters: f64,
        height_meters: f64,
        slope: [f64; 2],
        blend: f64,
    },
    Ridge {
        #[serde(with = "super::position_codec")]
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
    /// Polyline mountain/valley with metric width, signed height, asymmetric slopes and rounded crest.
    /// Branches are ordinary curves composed with Maximum/Minimum/Add and masks.
    Curve {
        #[serde(
            deserialize_with = "points",
            serialize_with = "super::position_codec::serialize_points"
        )]
        points: Vec<Position>,
        width_meters: f64,
        height_meters: f64,
        left_exponent: f64,
        right_exponent: f64,
        roundness: f64,
    },
    Channel {
        input: u16,
        #[serde(
            deserialize_with = "points",
            serialize_with = "super::position_codec::serialize_points"
        )]
        points: Vec<Position>,
        width_meters: f64,
        bed_start_meters: f64,
        bed_end_meters: f64,
        blend: f64,
    },
}

fn range(a: f64, b: f64) -> [f64; 2] {
    [a.min(b), a.max(b)]
}
fn union(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0].min(b[0]), a[1].max(b[1])]
}
fn finite(v: f64) -> bool {
    v.is_finite() && v.abs() <= MAX_VALUE
}
fn width(v: f64) -> bool {
    v.is_finite() && (0.125..=1e12).contains(&v)
}
fn fraction(v: f64) -> bool {
    v.is_finite() && (0. ..=1.).contains(&v)
}
fn exponent(v: f64) -> bool {
    v.is_finite() && (0.25..=8.).contains(&v)
}

impl Node {
    fn inputs(&self) -> ([u16; 3], usize) {
        match *self {
            Self::Add { a, b }
            | Self::Subtract { a, b }
            | Self::Multiply { a, b }
            | Self::Minimum { a, b }
            | Self::Maximum { a, b }
            | Self::SmoothMinimum { a, b, .. }
            | Self::SmoothMaximum { a, b, .. } => ([a, b, 0], 2),
            Self::Mix { a, b, mask } => ([a, b, mask], 3),
            Self::Clamp { input, .. }
            | Self::Remap { input, .. }
            | Self::Level { input, .. }
            | Self::Channel { input, .. } => ([input, 0, 0], 1),
            _ => ([0; 3], 0),
        }
    }
    fn enclosure(&self, prior: &[[f64; 2]]) -> Result<[f64; 2]> {
        let (refs, count) = self.inputs();
        if refs[..count].iter().any(|&n| usize::from(n) >= prior.len()) {
            return Err(invalid(
                "Surface nodes must reference earlier slots; cycles/forward references are rejected",
            ));
        }
        let at = |n: u16| prior[usize::from(n)];
        let r = match *self {
            Self::Input {
                minimum_meters,
                maximum_meters,
            } if finite(minimum_meters)
                && finite(maximum_meters)
                && minimum_meters <= maximum_meters =>
            {
                [minimum_meters, maximum_meters]
            }
            Self::Constant { value } if finite(value) => [value; 2],
            Self::Add { a, b } => [at(a)[0] + at(b)[0], at(a)[1] + at(b)[1]],
            Self::Subtract { a, b } => [at(a)[0] - at(b)[1], at(a)[1] - at(b)[0]],
            Self::Multiply { a, b } => {
                let a = at(a);
                let b = at(b);
                let v = [a[0] * b[0], a[0] * b[1], a[1] * b[0], a[1] * b[1]];
                [
                    v.into_iter().fold(f64::INFINITY, f64::min),
                    v.into_iter().fold(f64::NEG_INFINITY, f64::max),
                ]
            }
            Self::Minimum { a, b } => [at(a)[0].min(at(b)[0]), at(a)[1].min(at(b)[1])],
            Self::Maximum { a, b } => [at(a)[0].max(at(b)[0]), at(a)[1].max(at(b)[1])],
            Self::SmoothMinimum {
                a,
                b,
                transition_meters: k,
            } if width(k) => [at(a)[0].min(at(b)[0]) - k * 0.25, at(a)[1].min(at(b)[1])],
            Self::SmoothMaximum {
                a,
                b,
                transition_meters: k,
            } if width(k) => [at(a)[0].max(at(b)[0]), at(a)[1].max(at(b)[1]) + k * 0.25],
            Self::Mix { a, b, .. } => union(at(a), at(b)),
            Self::Clamp {
                input,
                minimum,
                maximum,
            } if finite(minimum) && finite(maximum) && minimum <= maximum => {
                at(input).map(|x| x.clamp(minimum, maximum))
            }
            Self::Remap { ref knots, .. } => {
                if !(2..=MAX_KNOTS).contains(&knots.len())
                    || knots
                        .iter()
                        .any(|k| !finite(k.x) || !finite(k.value) || !finite(k.derivative))
                    || knots.windows(2).any(|k| k[1].x - k[0].x < 1e-9)
                {
                    return Err(invalid(
                        "Surface remap needs2..32 finite Hermite knots in strictly increasing order",
                    ));
                }
                let mut bounds = [knots[0].value; 2];
                for pair in knots.windows(2) {
                    bounds = union(bounds, Polynomial::new(pair[0], pair[1]).bounds());
                }
                bounds
            }
            Self::Noise {
                period_meters,
                octaves,
                persistence,
                amplitude_meters,
                ..
            } if period_meters.is_power_of_two()
                && period_meters <= 1_073_741_824
                && (1..=12).contains(&octaves)
                && period_meters >> octaves.saturating_sub(1) > 0
                && fraction(persistence)
                && finite(amplitude_meters) =>
            {
                [-amplitude_meters.abs(), amplitude_meters.abs()]
            }
            Self::Raise {
                radius_meters,
                amount_meters,
                ..
            } if width(radius_meters) && finite(amount_meters) => range(0., amount_meters),
            Self::Level {
                input,
                radius_meters,
                height_meters,
                slope,
                blend,
                ..
            } if width(radius_meters)
                && finite(height_meters)
                && slope.iter().all(|v| finite(*v))
                && fraction(blend) =>
            {
                let delta = radius_meters * slope[0].hypot(slope[1]);
                union(at(input), [height_meters - delta, height_meters + delta])
            }
            Self::Ridge {
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
            } if azimuth_degrees.is_finite()
                && width(width_meters)
                && width(depth_meters)
                && finite(height_meters)
                && height_meters > 0.
                && finite(phase)
                && fraction(crest_wander)
                && crest_wander <= 0.5
                && exponent(slope_exponent)
                && finite(folds_meters)
                && folds_meters >= 0.
                && fraction(end_fade)
                && end_fade > 0. =>
            {
                [0., height_meters * 1.09 + folds_meters]
            }
            Self::Curve {
                ref points,
                width_meters,
                height_meters,
                left_exponent,
                right_exponent,
                roundness,
            } if (2..=MAX_CURVE).contains(&points.len())
                && width(width_meters)
                && finite(height_meters)
                && exponent(left_exponent)
                && exponent(right_exponent)
                && fraction(roundness) =>
            {
                range(0., height_meters)
            }
            Self::Channel {
                input,
                ref points,
                width_meters,
                bed_start_meters,
                bed_end_meters,
                blend,
            } if (2..=MAX_CURVE).contains(&points.len())
                && width(width_meters)
                && finite(bed_start_meters)
                && finite(bed_end_meters)
                && bed_end_meters <= bed_start_meters
                && fraction(blend) =>
            {
                [at(input)[0].min(bed_end_meters), at(input)[1]]
            }
            _ => return Err(invalid("Invalid common surface node parameters")),
        };
        if r.iter().any(|v| !finite(*v)) {
            return Err(invalid(
                "Surface intermediate interval overflow/range exceeded",
            ));
        }
        // Selection and constants do not round arithmetically; preserve exact boundary values.
        if matches!(
            self,
            Self::Constant { .. }
                | Self::Input { .. }
                | Self::Clamp { .. }
                | Self::Minimum { .. }
                | Self::Maximum { .. }
        ) {
            return Ok(r);
        }
        // Round outward; subsequent arithmetic must not shrink the culling certificate.
        let margin = (1. + r[0].abs().max(r[1].abs())) * 1e-12;
        Ok([(r[0] - margin).next_down(), (r[1] + margin).next_up()])
    }
}
#[derive(Clone, Copy)]
struct Polynomial {
    x: f64,
    width: f64,
    c: [f64; 4],
    enclosure: [f64; 2],
}
impl Polynomial {
    fn new(a: Knot, b: Knot) -> Self {
        let width = b.x - a.x;
        let m0 = a.derivative * width;
        let m1 = b.derivative * width;
        let controls = [a.value, a.value + m0 / 3., b.value - m1 / 3., b.value];
        let enclosure = [
            controls.into_iter().fold(f64::INFINITY, f64::min),
            controls.into_iter().fold(f64::NEG_INFINITY, f64::max),
        ];
        Self {
            enclosure,
            x: a.x,
            width,
            c: [
                a.value,
                m0,
                3. * (b.value - a.value) - 2. * m0 - m1,
                2. * (a.value - b.value) + m0 + m1,
            ],
        }
    }
    fn value(self, t: f64) -> f64 {
        ((self.c[3] * t + self.c[2]) * t + self.c[1]) * t + self.c[0]
    }
    fn bounds(self) -> [f64; 2] {
        // Bezier control hull encloses the entire Hermite segment, including overshoot.
        self.enclosure
    }
}
enum Geometry {
    None,
    Anchor {
        anchor: Anchor,
        support: f64,
        min_cosine: f64,
        orientation: (f64, f64),
    },
    Curve(Vec<Segment>),
    Spline(Vec<Polynomial>),
    Noise {
        amplitude: f64,
    },
}
#[derive(Clone)]
pub struct Compiled {
    program: std::sync::Arc<Program>,
    domain: Domain,
    geometry: std::sync::Arc<[Geometry]>,
    order: std::sync::Arc<[u16]>,
    has_noise: bool,
    input_bounds: Option<[f64; 2]>,
}
impl Program {
    pub fn new(nodes: Vec<Node>, output: u16) -> Result<Self> {
        let mut p = Self {
            version: VERSION,
            nodes,
            output,
            range_meters: [-1e6, 1e6],
        };
        p.range_meters = p.analyze()?;
        p.validate()?;
        Ok(p)
    }
    fn analyze(&self) -> Result<[f64; 2]> {
        if self.version != VERSION
            || self.nodes.is_empty()
            || self.nodes.len() > MAX_NODES
            || usize::from(self.output) >= self.nodes.len()
        {
            return Err(invalid(
                "Surface1 needs1..128 topologically ordered nodes and a valid output slot",
            ));
        }
        let mut bounds = [[0.; 2]; MAX_NODES];
        for (i, n) in self.nodes.iter().enumerate() {
            bounds[i] = n
                .enclosure(&bounds[..i])
                .map_err(|e| invalid(format!("Surface node{i}: {e}")))?;
        }
        Ok(bounds[usize::from(self.output)])
    }
    pub fn validate(&self) -> Result<()> {
        let actual = self.analyze()?;
        if self
            .range_meters
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1e6)
            || self.range_meters[0] > actual[0]
            || self.range_meters[1] < actual[1]
        {
            return Err(invalid(format!(
                "Surface range {:?} must enclose {:?} and fit±1000000m",
                self.range_meters, actual
            )));
        }
        Ok(())
    }
    pub fn validate_domain(&self, domain: Domain) -> Result<()> {
        self.validate()?;
        domain.validate()?;
        for node in &self.nodes {
            match *node {
                Node::Raise {
                    center,
                    radius_meters,
                    ..
                }
                | Node::Level {
                    center,
                    radius_meters,
                    ..
                } => {
                    domain.check_support(radius_meters)?;
                    domain.anchor(center)?;
                }
                Node::Ridge {
                    center,
                    width_meters,
                    depth_meters,
                    crest_wander,
                    ..
                } => {
                    domain.check_support(
                        (width_meters * 0.5).hypot(depth_meters * 0.5 * (1. + crest_wander)),
                    )?;
                    domain.anchor(center)?;
                }
                Node::Curve {
                    ref points,
                    width_meters,
                    ..
                }
                | Node::Channel {
                    ref points,
                    width_meters,
                    ..
                } => {
                    domain.check_support(width_meters)?;
                    for pair in points.windows(2) {
                        domain.segment(pair[0], pair[1], width_meters, 0.)?;
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
    pub fn validate_input(&self, incoming: [f64; 2]) -> Result<()> {
        if incoming.iter().any(|v| !v.is_finite()) || incoming[0] > incoming[1] {
            return Err(invalid("Invalid surface input interval"));
        }
        for (i, node) in self.nodes.iter().enumerate() {
            if let Node::Input {
                minimum_meters,
                maximum_meters,
            } = *node
                && (incoming[0] < minimum_meters || incoming[1] > maximum_meters)
            {
                return Err(invalid(format!(
                    "Surface node{i} input [{minimum_meters},{maximum_meters}] does not enclose source height {incoming:?}"
                )));
            }
        }
        Ok(())
    }
    pub fn work_per_point(&self) -> usize {
        self.nodes
            .iter()
            .map(|n| match n {
                Node::Noise { octaves, .. } => 8 * usize::from(*octaves),
                Node::Curve { points, .. } | Node::Channel { points, .. } => 32 * points.len(),
                Node::Remap { knots, .. } => knots.len() * 4,
                Node::Ridge { .. } => 32,
                _ => 8,
            })
            .sum()
    }
    pub fn compile(&self, domain: Domain) -> Result<Compiled> {
        self.validate_domain(domain)?;
        let mut active = [false; MAX_NODES];
        active[usize::from(self.output)] = true;
        for i in (0..self.nodes.len()).rev() {
            if active[i] {
                let (refs, n) = self.nodes[i].inputs();
                for &r in &refs[..n] {
                    active[usize::from(r)] = true;
                }
            }
        }
        let mut geometry = Vec::with_capacity(self.nodes.len());
        let mut has_noise = false;
        for (i, node) in self.nodes.iter().enumerate() {
            if !active[i] {
                geometry.push(Geometry::None);
                continue;
            }
            let g = match *node {
                Node::Raise {
                    center,
                    radius_meters,
                    ..
                }
                | Node::Level {
                    center,
                    radius_meters,
                    ..
                } => {
                    domain.check_support(radius_meters)?;
                    Geometry::Anchor {
                        anchor: domain.anchor(center)?,
                        support: radius_meters,
                        min_cosine: domain.cosine(radius_meters),
                        orientation: (0., 1.),
                    }
                }
                Node::Ridge {
                    center,
                    width_meters,
                    depth_meters,
                    crest_wander,
                    azimuth_degrees,
                    ..
                } => {
                    let support =
                        (width_meters * 0.5).hypot(depth_meters * 0.5 * (1. + crest_wander));
                    domain.check_support(support)?;
                    Geometry::Anchor {
                        anchor: domain.anchor(center)?,
                        support,
                        min_cosine: domain.cosine(support),
                        orientation: azimuth_degrees.to_radians().sin_cos(),
                    }
                }
                Node::Curve {
                    ref points,
                    width_meters,
                    ..
                }
                | Node::Channel {
                    ref points,
                    width_meters,
                    ..
                } => {
                    domain.check_support(width_meters)?;
                    Geometry::Curve(domain.segments(points, width_meters)?)
                }
                Node::Remap { ref knots, .. } => Geometry::Spline(
                    knots
                        .windows(2)
                        .map(|w| Polynomial::new(w[0], w[1]))
                        .collect(),
                ),
                Node::Noise {
                    octaves,
                    persistence,
                    amplitude_meters,
                    ..
                } => {
                    has_noise |= active[i];
                    let mut norm = 0.;
                    let mut weight = 1.;
                    for _ in 0..octaves {
                        norm += weight;
                        weight *= persistence;
                    }
                    Geometry::Noise {
                        amplitude: amplitude_meters / norm,
                    }
                }
                _ => Geometry::None,
            };
            geometry.push(g);
        }
        Ok(Compiled {
            program: std::sync::Arc::new(self.clone()),
            domain,
            geometry: geometry.into(),
            order: active[..self.nodes.len()]
                .iter()
                .enumerate()
                .filter_map(|(i, &yes)| yes.then_some(i as u16))
                .collect::<Vec<_>>()
                .into(),
            has_noise,
            input_bounds: self
                .nodes
                .iter()
                .filter_map(|n| {
                    if let Node::Input {
                        minimum_meters,
                        maximum_meters,
                    } = *n
                    {
                        Some([minimum_meters, maximum_meters])
                    } else {
                        None
                    }
                })
                .reduce(|a, b| [a[0].max(b[0]), a[1].min(b[1])]),
        })
    }
}
impl Compiled {
    pub fn sample(&self, p: Position) -> Result<f64> {
        self.sample_with_input(p, 0.)
    }
    pub fn sample_with_input(&self, p: Position, input: f64) -> Result<f64> {
        if !input.is_finite()
            || self
                .input_bounds
                .is_some_and(|b| input < b[0] || input > b[1])
        {
            return Err(invalid(
                "Surface sample input is outside the certified range",
            ));
        }
        Ok(self.evaluate(self.domain.point(p)?, input))
    }
    pub(crate) fn sample_unit_with_input(&self, unit: DVec3, input: f64) -> f64 {
        self.evaluate(self.domain.unit_point(unit), input)
    }
    fn evaluate(&self, p: Point, input: f64) -> f64 {
        let mut slots = [0.; MAX_NODES];
        let noise_position = self.has_noise.then(|| self.domain.noise_position(p));
        for &index in &*self.order {
            let i = usize::from(index);
            let n = &self.program.nodes[i];
            let at = |n: u16| slots[usize::from(n)];
            let local = || {
                let Geometry::Anchor {
                    anchor,
                    support,
                    min_cosine,
                    ..
                } = self.geometry[i]
                else {
                    unreachable!()
                };
                self.domain.local(p, anchor, support, min_cosine)
            };
            slots[i] = match *n {
                Node::Input { .. } => input,
                Node::Constant { value } => value,
                Node::Add { a, b } => at(a) + at(b),
                Node::Subtract { a, b } => at(a) - at(b),
                Node::Multiply { a, b } => at(a) * at(b),
                Node::Minimum { a, b } => at(a).min(at(b)),
                Node::Maximum { a, b } => at(a).max(at(b)),
                Node::SmoothMinimum {
                    a,
                    b,
                    transition_meters: k,
                } => at(a).min(at(b)) - smooth_union_offset(at(a), at(b), k),
                Node::SmoothMaximum {
                    a,
                    b,
                    transition_meters: k,
                } => at(a).max(at(b)) + smooth_union_offset(at(a), at(b), k),
                Node::Mix { a, b, mask } => super::mix(at(a), at(b), at(mask).clamp(0., 1.)),
                Node::Clamp {
                    input,
                    minimum,
                    maximum,
                } => at(input).clamp(minimum, maximum),
                Node::Remap { input, ref knots } => {
                    let x = at(input);
                    let Geometry::Spline(ref polynomials) = self.geometry[i] else {
                        unreachable!()
                    };
                    if x <= knots[0].x {
                        knots[0].value
                    } else if x >= knots.last().unwrap().x {
                        knots.last().unwrap().value
                    } else {
                        let j = knots.partition_point(|k| k.x <= x) - 1;
                        let q = polynomials[j];
                        q.value((x - q.x) / q.width)
                    }
                }
                Node::Noise {
                    seed,
                    period_meters,
                    octaves,
                    persistence,
                    style,
                    ..
                } => {
                    let Geometry::Noise { amplitude } = self.geometry[i] else {
                        unreachable!()
                    };
                    let mut sum = 0.;
                    let mut weight = 1.;
                    for octave in 0..octaves {
                        let n = noise3(
                            noise_position.unwrap(),
                            period_meters >> octave,
                            seed.wrapping_add(u32::from(octave).wrapping_mul(0x9e3779b9)),
                        );
                        sum += weight
                            * match style {
                                super::NoiseKind::Smooth => n,
                                super::NoiseKind::Ridged => 1. - 2. * n.abs(),
                            };
                        weight *= persistence;
                    }
                    amplitude * sum
                }
                Node::Raise {
                    radius_meters,
                    amount_meters,
                    ..
                } => local().map_or(0., |d| {
                    amount_meters * super::weight(d, radius_meters).clamp(0., 1.)
                }),
                Node::Level {
                    input,
                    radius_meters,
                    height_meters,
                    slope,
                    blend,
                    ..
                } => local().map_or(at(input), |d| {
                    super::mix(
                        at(input),
                        height_meters + slope[0] * d[0] + slope[1] * d[1],
                        (blend * super::weight(d, radius_meters)).clamp(0., 1.),
                    )
                }),
                Node::Ridge {
                    width_meters,
                    depth_meters,
                    height_meters,
                    phase,
                    crest_wander,
                    slope_exponent,
                    folds_meters,
                    end_fade,
                    ..
                } => local().map_or(0., |d| {
                    let Geometry::Anchor {
                        orientation: (sin, cos),
                        ..
                    } = self.geometry[i]
                    else {
                        unreachable!()
                    };
                    let u = 2. * (cos * d[0] + sin * d[1]) / width_meters;
                    let v = 2. * (-sin * d[0] + cos * d[1]) / depth_meters;
                    super::ridge_profile(
                        u,
                        v,
                        height_meters,
                        phase,
                        crest_wander,
                        slope_exponent,
                        folds_meters,
                        end_fade,
                    )
                    .unwrap_or(0.)
                }),
                Node::Curve {
                    width_meters,
                    height_meters,
                    left_exponent,
                    right_exponent,
                    roundness,
                    ..
                } => {
                    let Geometry::Curve(ref segments) = self.geometry[i] else {
                        unreachable!()
                    };
                    let mut best = 0f64;
                    for s in segments {
                        if let Some((d, _, cross)) = self.domain.nearest(p, s, width_meters) {
                            let r = (d / width_meters).min(1.);
                            let t =
                                super::mix(1. - r, 1. - super::smooth(r), roundness).clamp(0., 1.);
                            let side = if d > 1e-12 {
                                0.5 + 0.5 * (cross / d).clamp(-1., 1.)
                            } else {
                                0.5
                            };
                            best = best.max(super::mix(
                                t.powf(left_exponent),
                                t.powf(right_exponent),
                                side,
                            ));
                        }
                    }
                    height_meters * best
                }
                Node::Channel {
                    input,
                    width_meters,
                    bed_start_meters,
                    bed_end_meters,
                    blend,
                    ..
                } => {
                    let Geometry::Curve(ref segments) = self.geometry[i] else {
                        unreachable!()
                    };
                    let last = segments.last().unwrap();
                    let total = last.prefix + last.length;
                    let original = at(input);
                    let mut h = original;
                    for s in segments {
                        if let Some((d, t, _)) = self.domain.nearest(p, s, width_meters) {
                            let bed = super::mix(
                                bed_start_meters,
                                bed_end_meters,
                                (s.prefix + t * s.length) / total,
                            );
                            h = h.min(super::mix(
                                original,
                                original.min(bed),
                                (blend * (1. - super::smooth(d / width_meters))).clamp(0., 1.),
                            ));
                        }
                    }
                    h
                }
            };
        }
        slots[usize::from(self.program.output)]
    }
}
fn smooth_union_offset(a: f64, b: f64, k: f64) -> f64 {
    let h = (k - (a - b).abs()).max(0.) / k;
    h * h * k * 0.25
}
fn noise3(p: Position, period: u32, seed: u32) -> f64 {
    let (x, tx) = super::lattice(p, 0, period);
    let (y, ty) = super::lattice(p, 1, period);
    let (z, tz) = super::lattice(p, 2, period);
    let fold = |i: i128| super::hash(i as u64 ^ super::hash((i >> 64) as u64));
    let mut out = 0.;
    let q = [super::smooth(tx), super::smooth(ty), super::smooth(tz)];
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                let v = super::hash(
                    fold(x + dx)
                        ^ fold(y + dy).rotate_left(17)
                        ^ fold(z + dz).rotate_left(39)
                        ^ u64::from(seed),
                );
                let value = (v >> 11) as f64 * (1. / ((1u64 << 53) as f64)) * 2. - 1.;
                out += value
                    * if dx == 0 { 1. - q[0] } else { q[0] }
                    * if dy == 0 { 1. - q[1] } else { q[1] }
                    * if dz == 0 { 1. - q[2] } else { q[2] };
            }
        }
    }
    out
}

pub(crate) fn outward(b: [f64; 2]) -> [f64; 2] {
    let e = (1. + b[0].abs().max(b[1].abs())) * 1e-12;
    [(b[0] - e).next_down(), (b[1] + e).next_up()]
}
