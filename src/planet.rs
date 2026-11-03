//! Finite spherical world source. Global f64 geometry is independent of resident simulation.
use crate::{
    model::{Result, invalid},
    world_space::Position,
};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub mod mesh;
pub mod orbit;
pub mod residency;
pub mod runtime;
pub mod view;
pub const VERSION: u16 = 1;
pub const MAX_OPERATIONS: usize = 32;
pub const MAX_WORK: usize = 16_000_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u16,
    pub space_version: u16,
    pub id: String,
    pub revision: u32,
    #[serde(with = "crate::terrain::position_codec")]
    #[schemars(with = "Position")]
    pub center: Position,
    /// Diameter of the reference sphere, not the enclosing relief or an orbital parameter.
    pub diameter_meters: f64,
    pub surface_gravity_mps2: f64,
    pub base_height_meters: f64,
    #[serde(deserialize_with = "operations")]
    pub operations: Vec<Relief>,
}
fn operations<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Relief>, D::Error> {
    crate::terrain::limited::<D, Relief, MAX_OPERATIONS>(d)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Relief {
    Noise {
        seed: u32,
        wavelength_meters: f64,
        amplitude_meters: f64,
        octaves: u8,
        persistence: f64,
    },
    Raise {
        direction: [f64; 3],
        angular_radius_degrees: f64,
        height_meters: f64,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    pub direction: [f64; 3],
    /// Tangent heading transported with travel; no longitude or polar singularity.
    pub forward: [f64; 3],
    /// Height above the reference sphere, not automatic terrain following.
    pub altitude_meters: f64,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Frame {
    pub right: [f64; 3],
    pub up: [f64; 3],
    pub forward: [f64; 3],
}
fn direction(v: [f64; 3]) -> Result<DVec3> {
    let v = DVec3::from_array(v);
    if !v.is_finite() || !(0.5..=2.).contains(&v.length()) {
        return Err(invalid(
            "Planet direction requires a finite near-unit vector (length0.5..2)",
        ));
    }
    Ok(v.normalize())
}
impl Pose {
    pub fn frame(self) -> Result<Frame> {
        let up = direction(self.direction)?;
        let f = direction(self.forward)?;
        let f = f - up * f.dot(up);
        if f.length() < 1e-6 || !self.altitude_meters.is_finite() {
            return Err(invalid(
                "Planet heading must have a nonzero tangent; altitude must be finite",
            ));
        }
        let forward = f.normalize();
        Ok(Frame {
            right: forward.cross(up).to_array(),
            up: up.to_array(),
            forward: forward.to_array(),
        })
    }
}
impl Document {
    pub fn radius(&self) -> f64 {
        self.diameter_meters * 0.5
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.space_version != crate::world_space::VERSION
            || self.id.is_empty()
            || self.id.len() > 120
            || self.revision == 0
            || !self.diameter_meters.is_finite()
            || !(10. ..=1e12).contains(&self.diameter_meters)
            || !self.surface_gravity_mps2.is_finite()
            || !(0. ..=1000.).contains(&self.surface_gravity_mps2)
            || !self.base_height_meters.is_finite()
            || self.operations.len() > MAX_OPERATIONS
        {
            return Err(invalid(
                "Planet1 needs id1..120bytes/revision>0, diameter10..1e12m, gravity0..1000m/s² and <=32operations",
            ));
        }
        let mut bound = self.base_height_meters.abs();
        for op in &self.operations {
            match *op {
                Relief::Noise {
                    wavelength_meters,
                    amplitude_meters,
                    octaves,
                    persistence,
                    ..
                } => {
                    if !wavelength_meters.is_finite()
                        || wavelength_meters < self.radius() / 1e6
                        || wavelength_meters > self.diameter_meters * 4.
                        || !amplitude_meters.is_finite()
                        || !(1..=8).contains(&octaves)
                        || !persistence.is_finite()
                        || !(0. ..=1.).contains(&persistence)
                    {
                        return Err(invalid(
                            "Planet noise requires wavelength[R/1e6,4D], finite amplitude, octaves1..8, persistence0..1",
                        ));
                    }
                    bound += amplitude_meters.abs();
                }
                Relief::Raise {
                    direction: d,
                    angular_radius_degrees,
                    height_meters,
                } => {
                    direction(d)?;
                    if !angular_radius_degrees.is_finite()
                        || !(0.001..=180.).contains(&angular_radius_degrees)
                        || !height_meters.is_finite()
                    {
                        return Err(invalid(
                            "Planet raise requires finite height and angular radius0.001..180degrees",
                        ));
                    }
                    bound += height_meters.abs();
                }
            }
        }
        if bound > self.radius() * 0.25 {
            return Err(invalid(
                "Planet relief absolute bound exceeds25% of reference radius",
            ));
        }
        // Reject an address too near a cell overflow before producing partial geometry or files.
        let extent = self.radius() + bound;
        for axis in 0..3 {
            for sign in [-1., 1.] {
                let mut delta = [0.; 3];
                delta[axis] = sign * extent;
                self.center
                    .translated(delta)
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        Ok(())
    }
    pub fn budget(&self, count: usize) -> Result<()> {
        self.validate()?;
        let work = 1 + self
            .operations
            .iter()
            .map(|o| match o {
                Relief::Noise { octaves, .. } => usize::from(*octaves) * 8,
                Relief::Raise { .. } => 1,
            })
            .sum::<usize>();
        if count.saturating_mul(work) > MAX_WORK {
            return Err(invalid("Planet sample/work budget exceeded"));
        }
        Ok(())
    }
    pub fn travel(&self, pose: Pose, distance_meters: f64) -> Result<Pose> {
        self.validate()?;
        let f = pose.frame()?;
        if !distance_meters.is_finite()
            || distance_meters.abs() > self.radius() * std::f64::consts::TAU * 1e6
            || pose.altitude_meters <= -self.radius()
        {
            return Err(invalid(
                "Planet travel requires finite distance up to1million revolutions and altitude>-R",
            ));
        }
        let up = DVec3::from(f.up);
        let forward = DVec3::from(f.forward);
        // Distance is measured on the reference sphere. Preserve heading by parallel transport.
        let (s, c) = (distance_meters / self.radius())
            .rem_euclid(std::f64::consts::TAU)
            .sin_cos();
        Ok(Pose {
            direction: (up * c + forward * s).normalize().to_array(),
            forward: (forward * c - up * s).normalize().to_array(),
            altitude_meters: pose.altitude_meters,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Sample {
    pub direction: [f64; 3],
    pub height_meters: f64,
    pub radial_distance_meters: f64,
    pub position: Position,
    pub gravity_mps2: [f64; 3],
}
/// Validated source view with bounded inline coefficients; no per-sample allocation or I/O.
#[derive(Clone, Copy)]
enum Prepared {
    Empty,
    Noise {
        seed: u32,
        frequency: f64,
        amplitude: f64,
        octaves: u8,
        persistence: f64,
    },
    Raise {
        direction: DVec3,
        cosine: f64,
        inverse_angle: f64,
        height: f64,
    },
}
pub struct Sampler<'a> {
    document: &'a Document,
    operations: [Prepared; MAX_OPERATIONS],
}
impl<'a> Sampler<'a> {
    pub fn new(document: &'a Document) -> Result<Self> {
        document.validate()?;
        let mut operations = [Prepared::Empty; MAX_OPERATIONS];
        for (target, op) in operations.iter_mut().zip(&document.operations) {
            *target = match *op {
                Relief::Noise {
                    seed,
                    wavelength_meters,
                    amplitude_meters,
                    octaves,
                    persistence,
                } => {
                    let (mut weight, mut total) = (1., 0.);
                    for _ in 0..octaves {
                        total += weight;
                        weight *= persistence;
                    }
                    Prepared::Noise {
                        seed,
                        frequency: document.radius() / wavelength_meters,
                        amplitude: amplitude_meters / total,
                        octaves,
                        persistence,
                    }
                }
                Relief::Raise {
                    direction,
                    angular_radius_degrees,
                    height_meters,
                } => Prepared::Raise {
                    direction: DVec3::from(direction).normalize(),
                    cosine: angular_radius_degrees.to_radians().cos(),
                    inverse_angle: 1. / angular_radius_degrees.to_radians(),
                    height: height_meters,
                },
            };
        }
        Ok(Self {
            document,
            operations,
        })
    }
    pub(crate) fn height(&self, unit: DVec3) -> f64 {
        let mut h = self.document.base_height_meters;
        for op in &self.operations[..self.document.operations.len()] {
            match *op {
                Prepared::Noise {
                    seed,
                    frequency,
                    amplitude,
                    octaves,
                    persistence,
                } => {
                    let mut p = unit * frequency;
                    let (mut sum, mut weight) = (0., 1.);
                    for octave in 0..octaves {
                        sum += noise(p, seed.wrapping_add(u32::from(octave))) * weight;
                        weight *= persistence;
                        p *= 2.;
                    }
                    h += amplitude * sum;
                }
                Prepared::Raise {
                    direction,
                    cosine,
                    inverse_angle,
                    height,
                } => {
                    let dot = unit.dot(direction).clamp(-1., 1.);
                    if dot > cosine {
                        let t = dot.acos() * inverse_angle;
                        h += height * (1. - t * t).powi(2);
                    }
                }
                Prepared::Empty => {}
            }
        }
        h
    }
    pub fn sample(&self, d: [f64; 3]) -> Result<Sample> {
        let up = direction(d)?;
        let height = self.height(up);
        let radius = self.document.radius() + height;
        let position = self
            .document
            .center
            .translated((up * radius).to_array())
            .map_err(|e| invalid(e.to_string()))?;
        let g =
            -up * self.document.surface_gravity_mps2 * (self.document.radius() / radius).powi(2);
        Ok(Sample {
            direction: up.to_array(),
            height_meters: height,
            radial_distance_meters: radius,
            position,
            gravity_mps2: g.to_array(),
        })
    }
}
// 3D value noise on planet-centric Cartesian direction: charts/tiles never change its seed.
fn noise(p: DVec3, seed: u32) -> f64 {
    let c = p.floor();
    let q = p - c;
    let q = q * q * q * (q * (q * 6. - DVec3::splat(15.)) + DVec3::splat(10.));
    let mut out = 0.;
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let a = (c.x as i64).wrapping_add(x);
                let b = (c.y as i64).wrapping_add(y);
                let c = (c.z as i64).wrapping_add(z);
                let mut v = (a as u64).wrapping_mul(0x9e3779b97f4a7c15)
                    ^ (b as u64).wrapping_mul(0xbf58476d1ce4e5b9)
                    ^ (c as u64).wrapping_mul(0x94d049bb133111eb)
                    ^ u64::from(seed);
                v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
                v ^= v >> 31;
                let val = (v >> 11) as f64 / (1u64 << 53) as f64 * 2. - 1.;
                out += val
                    * if x == 0 { 1. - q.x } else { q.x }
                    * if y == 0 { 1. - q.y } else { q.y }
                    * if z == 0 { 1. - q.z } else { q.z };
            }
        }
    }
    out
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    Inspect {
        document: Document,
    },
    Sample {
        document: Document,
        #[serde(deserialize_with = "sample_directions")]
        directions: Vec<[f64; 3]>,
    },
    Travel {
        document: Document,
        pose: Pose,
        distance_meters: f64,
    },
    Horizon {
        document: Document,
        altitude_meters: f64,
    },
    InspectMesh {
        document: Document,
        area: mesh::Area,
        frame: mesh::Projection,
    },
    PlanResidence {
        document: Document,
        request: residency::Request,
    },
}
fn sample_directions<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<[f64; 3]>, D::Error> {
    crate::terrain::limited::<D, [f64; 3], 4096>(d)
}
pub fn query(q: Query) -> Result<Value> {
    match q {
        Query::Inspect { document } => {
            document.validate()?;
            Ok(
                json!({"radius_meters":document.radius(),"circumference_meters":std::f64::consts::PI*document.diameter_meters,"surface_area_square_meters":std::f64::consts::PI*document.diameter_meters.powi(2),"document":document,"runtime_world_mode":false,"streaming":false}),
            )
        }
        Query::Sample {
            document,
            directions,
        } => {
            if directions.len() > 4096 {
                return Err(invalid("Planet sample requires at most4096directions"));
            }
            document.budget(directions.len())?;
            let sampler = Sampler::new(&document)?;
            let samples = directions
                .into_iter()
                .map(|d| sampler.sample(d))
                .collect::<Result<Vec<_>>>()?;
            Ok(json!({"samples":samples,"gravity_applied_to_runtime":false}))
        }
        Query::Travel {
            document,
            pose,
            distance_meters,
        } => {
            let pose = document.travel(pose, distance_meters)?;
            Ok(
                json!({"pose":pose,"frame":pose.frame()?,"geometric_only":true,"collision_checked":false,"reference_surface_distance_meters":distance_meters}),
            )
        }
        Query::Horizon {
            document,
            altitude_meters: h,
        } => {
            document.validate()?;
            if !h.is_finite() || !(0. ..=document.diameter_meters * 100.).contains(&h) {
                return Err(invalid("Horizon altitude needs0..100D meters"));
            }
            let r = document.radius();
            let tangent = (h * (2. * r + h)).sqrt();
            let angle = tangent.atan2(r);
            Ok(
                json!({"line_of_sight_meters":tangent,"surface_arc_meters":r*angle,"dip_degrees":angle.to_degrees(),"reference_sphere_only":true,"refraction":false,"terrain_occlusion_checked":false}),
            )
        }
        Query::InspectMesh {
            document,
            area,
            frame,
        } => Ok(mesh::compile(&document, &area, &frame)?.report),
        Query::PlanResidence { document, request } => {
            Ok(serde_json::to_value(residency::plan(&document, &request)?)
                .expect("typed planet residence plan"))
        }
    }
}
pub fn encode(d: &Document) -> Result<Vec<u8>> {
    d.validate()?;
    crate::terrain::encode_asset(b"IGNISPLN", VERSION, d)
}
pub fn decode(bytes: &[u8]) -> Result<Document> {
    let d: Document = crate::terrain::decode_asset(bytes, b"IGNISPLN", VERSION)?;
    d.validate()?;
    Ok(d)
}
