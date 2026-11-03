//! Seeded ocular authoring. Pigment is baked once, mip filtered by the common
//! material renderer; the socket transports geometry without resampling pigment.
use super::*;
use crate::materials::{
    self,
    recipe::{Fixed, Procedural, Recipe, Sample},
};
use std::{
    f32::consts::{PI, TAU},
    sync::Mutex,
};

/// Name, label, minimum, maximum, default. The configuration is shared by MCP/UI.
pub const CONTROLS: &[(&str, &str, f32, f32, f32)] = &[
    ("red", "Pigmento vermelho", 0., 1., 0.16),
    ("green", "Pigmento verde", 0., 1., 0.085),
    ("blue", "Pigmento azul", 0., 1., 0.025),
    ("inner_red", "Centro vermelho", 0., 1., 0.24),
    ("inner_green", "Centro verde", 0., 1., 0.12),
    ("inner_blue", "Centro azul", 0., 1., 0.025),
    ("inner_blend", "Pigmento central", 0., 1., 0.6),
    ("pupil", "Diâmetro relativo da pupila", 0.14, 0.65, 0.34),
    ("radius", "Raio da íris", 0.004, 0.007, 0.0056),
    ("limbal_width", "Largura do anel externo", 0., 0.18, 0.055),
    (
        "limbal_darkness",
        "Intensidade do anel externo",
        0.,
        1.,
        0.8,
    ),
    ("fibers", "Fibras radiais", 0., 1., 0.75),
    ("crypts", "Criptas e colarete", 0., 1., 0.65),
    ("rings", "Sulcos de contração", 0., 1., 0.45),
    ("relief", "Relevo da íris", 0., 0.00012, 0.000035),
    ("roughness", "Rugosidade da íris", 0.2, 1., 0.65),
    ("cornea_roughness", "Rugosidade da córnea", 0.2, 0.6, 0.2),
    ("depth", "Profundidade da íris", 0., 0.0012, 0.00035),
    ("sclera_redness", "Tonalidade da esclera", 0., 1., 0.15),
    ("seed", "Identidade do desenho", 0., 65535., 17.),
    ("quality", "Resolução do desenho", 0., 2., 1.),
];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Eye {
    pub sclera: String,
    pub iris: String,
    /// Reuses the old pupil part identity. The pupil is now in the iris map.
    pub cornea: String,
    pub center: Anchor,
    /// Parameter names are `<prefix>_<control>`.
    pub prefix: String,
    pub radius: f32,
    #[serde(skip)]
    cache: Arc<Mutex<Option<Cache>>>,
}
#[derive(Debug)]
struct Cache {
    key: Vec<u32>,
    material: Arc<Procedural>,
}
impl PartialEq for Eye {
    fn eq(&self, b: &Self) -> bool {
        self.sclera == b.sclera
            && self.iris == b.iris
            && self.cornea == b.cornea
            && self.center == b.center
            && self.prefix == b.prefix
            && self.radius == b.radius
    }
}
impl Eye {
    pub fn new(
        sclera: String,
        iris: String,
        cornea: String,
        center: Anchor,
        prefix: String,
        radius: f32,
    ) -> Self {
        Self {
            sclera,
            iris,
            cornea,
            center,
            prefix,
            radius,
            cache: Default::default(),
        }
    }
    fn material(&self, v: &[f32]) -> Result<Arc<Procedural>> {
        let key: Vec<_> = v
            .iter()
            .enumerate()
            .filter(|(i, _)| ![16, 17, 18].contains(i))
            .map(|(_, x)| x.to_bits())
            .collect();
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| invalid("Ocular cache poisoned"))?;
        if let Some(c) = cache.as_ref()
            && c.key == key
        {
            return Ok(c.material.clone());
        }
        let material = Arc::new(pigment(v));
        material.validate()?;
        *cache = Some(Cache {
            key,
            material: material.clone(),
        });
        Ok(material)
    }
}
impl Source {
    pub(super) fn validate_eyes(&self, params: &BTreeMap<&String, &Parameter>) -> Result<()> {
        if self.eyes.len() > 2 || (self.version < 7 && !self.eyes.is_empty()) {
            return Err(invalid(
                "Ocular authoring requires source7 and at most two eyes",
            ));
        }
        let mut owned = BTreeSet::new();
        let mut prefixes = BTreeSet::new();
        for eye in &self.eyes {
            super::super::id(&eye.prefix)?;
            if !prefixes.insert(&eye.prefix)
                || !eye.radius.is_finite()
                || !(0.009..=0.02).contains(&eye.radius)
                || eye.center.vertices.is_empty()
                || eye.center.vertices.len() > 32
                || eye
                    .center
                    .vertices
                    .iter()
                    .any(|&i| i as usize >= self.positions.len())
            {
                return Err(invalid("Invalid ocular center, radius or duplicate prefix"));
            }
            let mut socket = None;
            for name in [&eye.sclera, &eye.iris, &eye.cornea] {
                let part = self
                    .parts
                    .iter()
                    .find(|p| &p.name == name)
                    .ok_or_else(|| invalid("Unknown ocular part"))?;
                let joint = part
                    .socket
                    .ok_or_else(|| invalid("Ocular parts require a rigid socket"))?;
                if joint as usize >= self.joints.len()
                    || !owned.insert(name)
                    || socket.is_some_and(|j| j != joint)
                    || part.weights.is_empty()
                    || self.surfaces.iter().any(|s| &s.part == name)
                    || self.refinements.iter().any(|r| &r.part == name)
                {
                    return Err(invalid(
                        "Ocular parts require distinct ownership and one shared socket",
                    ));
                }
                socket = Some(joint);
                // Generated vertices keep this exact rigid weight. Never truncate an authored skin.
                if part.weights.iter().any(|w| {
                    w.influences().any(|(j, weight)| weight > 0. && j != joint)
                        || (w.influences().map(|(_, w)| w).sum::<f32>() - 1.).abs() > 1e-6
                }) {
                    return Err(invalid("Ocular weights must follow their socket rigidly"));
                }
            }
            for &(name, _, min, max, _) in CONTROLS {
                let key = format!("{}_{}", eye.prefix, name);
                let p = params
                    .get(&key)
                    .ok_or_else(|| invalid("Missing ocular parameter"))?;
                if p.minimum < min || p.maximum > max {
                    return Err(invalid("Ocular parameter exceeds supported bounds"));
                }
            }
        }
        Ok(())
    }
    pub(super) fn apply_eye(
        &self,
        part: &Part,
        values: &BTreeMap<String, f32>,
        socket: Option<Mat4>,
        asset: &mut Asset,
    ) -> Result<Option<[f32; 3]>> {
        let Some(eye) = self
            .eyes
            .iter()
            .find(|e| e.sclera == part.name || e.iris == part.name || e.cornea == part.name)
        else {
            return Ok(None);
        };
        let v: Vec<f32> = CONTROLS
            .iter()
            .map(|c| values[&format!("{}_{}", eye.prefix, c.0)])
            .collect();
        let r = v[8];
        let z = -(eye.radius * eye.radius - r * r).sqrt();
        let center = eye.center.at(&self.positions);
        let iris = part.name == eye.iris;
        let sclera = part.name == eye.sclera;
        let mut g = if sclera {
            // Open limbus, closed rear pole: the iris can be recessed without z fighting.
            lathe(24, 80, false, |t| {
                let theta = (r / eye.radius).asin() + (PI - (r / eye.radius).asin()) * t;
                [eye.radius * theta.sin(), -eye.radius * theta.cos()]
            })
        } else {
            lathe(12, 80, true, |t| {
                [
                    r * t,
                    if iris {
                        z + (v[17] + 0.00015) * (1. - t * t)
                    } else {
                        z - 0.0018 * (1. - t * t) - 0.000025
                    },
                ]
            })
        };
        for p in &mut g.positions {
            let rest = center + Vec3::from(*p);
            *p = socket.map_or(rest, |m| m.transform_point3(rest)).to_array();
        }
        if iris {
            asset.material = materials::Material {
                procedural: Some(eye.material(&v)?),
                ..Default::default()
            };
        } else if sclera {
            asset.material = materials::Material {
                reflectance: Some(materials::Reflectance {
                    roughness: 0.28,
                    f0: 0.025,
                    metallic: 0.,
                }),
                ..Default::default()
            };
        } else {
            asset.material = materials::Material {
                surface: Some(materials::Surface {
                    mode: materials::BlendMode::Blend,
                    glass: Some(materials::Glass {
                        ior: 1.376,
                        roughness: v[16],
                        thickness_m: 0.,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
        }
        if iris {
            // Local XY material domain stays attached to the eye through every pose.
            g.material_coordinates = Some(
                lathe(12, 80, true, |t| [r * t, 0.])
                    .positions
                    .iter()
                    .map(|p| [p[0] + r, p[1] + r, 0.])
                    .collect(),
            );
        }
        let count = g.positions.len();
        asset.geometry = Arc::new(g);
        Arc::make_mut(asset.rig.as_mut().unwrap()).weights = vec![part.weights[0].clone(); count];
        Ok(Some(if sclera {
            [0.82, 0.81 - v[18] * 0.1, 0.76 - v[18] * 0.1]
        } else {
            [1.; 3]
        }))
    }
}

/// A surface of revolution, forward normals toward -Z. `disk=false` ends at rear pole.
fn lathe(rings: usize, segments: usize, disk: bool, profile: impl Fn(f32) -> [f32; 2]) -> Geometry {
    let mut p = Vec::new();
    let mut triangles = Vec::new();
    let mut ring = |t: f32| {
        let [r, z] = profile(t);
        for i in 0..segments {
            let a = TAU * i as f32 / segments as f32;
            p.push([r * a.cos(), r * a.sin(), z]);
        }
    };
    for j in 0..rings {
        ring(if disk {
            (j + 1) as f32 / rings as f32
        } else {
            j as f32 / rings as f32
        });
    }
    for j in 0..rings - 1 {
        for i in 0..segments {
            let a = (j * segments + i) as u32;
            let b = (j * segments + (i + 1) % segments) as u32;
            let c = a + segments as u32;
            let d = b + segments as u32;
            triangles.extend([[a, d, c], [a, b, d]]);
        }
    }
    let pole = p.len() as u32;
    p.push([0., 0., profile(if disk { 0. } else { 1. })[1]]);
    for i in 0..segments {
        if disk {
            triangles.push([pole, ((i + 1) % segments) as u32, i as u32]);
        } else {
            let a = ((rings - 1) * segments + i) as u32;
            let b = ((rings - 1) * segments + (i + 1) % segments) as u32;
            triangles.push([a, b, pole]);
        }
    }
    Geometry {
        positions: p,
        triangles,
        material_coordinates: None,
    }
}
fn hash(i: i32, j: i32, seed: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(1597334677) ^ (j as u32).wrapping_mul(3812015801) ^ seed;
    h = (h ^ (h >> 16)).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(3266489917);
    h ^= h >> 16;
    (h & 16777215) as f32 / 16777215.
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a).max(f32::EPSILON)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn noise(u: f32, v: f32, n: i32, m: i32, seed: u32) -> f32 {
    let x = u * n as f32;
    let y = v * m as f32;
    let i = x.floor() as i32;
    let j = y.floor() as i32;
    let a = smooth(0., 1., x - x.floor());
    let b = smooth(0., 1., y - y.floor());
    let at = |di: i32, dj: i32| hash((i + di).rem_euclid(n), j + dj, seed);
    let lo = at(0, 0) + (at(1, 0) - at(0, 0)) * a;
    let hi = at(0, 1) + (at(1, 1) - at(0, 1)) * a;
    lo + (hi - lo) * b
}
fn texel(q: [f32; 2], v: &[f32]) -> Sample {
    let x = q[0] * 2. - 1.;
    let y = q[1] * 2. - 1.;
    let radius = x.hypot(y);
    let angle = y.atan2(x) / TAU + 0.5;
    let seed = v[19].round() as u32;
    let pupil = v[7];
    let t = ((radius - pupil) / (1. - pupil)).clamp(0., 1.);
    let broad = noise(angle, t, 31, 5, seed);
    let bend = noise(angle, t, 17, 4, seed + 7) - 0.5;
    let strand = noise(angle + 0.012 * bend, t, 173, 3, seed + 31);
    let fine = noise(angle + 0.006 * bend, t, 311, 7, seed + 73);
    let fibers = (0.6 * strand + 0.4 * fine - 0.5) * v[11];
    let collarette = 0.34 + 0.09 * (noise(angle, 0., 29, 1, seed + 47) - 0.5);
    let crypt = smooth(0.53, 0.8, noise(angle, t, 67, 13, seed + 19))
        * (1. - smooth(0.06, 0.24, (t - collarette).abs()))
        * v[12];
    let rings = (0.5 + 0.5 * (t * 65. + bend * 2.).sin())
        * smooth(0.5, 0.85, t)
        * smooth(0.3, 0.75, noise(angle, t, 13, 3, seed + 89))
        * v[13];
    let inner = (1. - smooth(collarette - 0.08, collarette + 0.1, t)) * v[6];
    let limbal = if v[9] > 0. {
        smooth(1. - v[9], 1., radius) * v[10]
    } else {
        0.
    };
    let ruff = (1. - smooth(pupil, pupil + 0.025 + 0.018 * broad, radius)) * 0.8;
    let brightness = (0.78 + 0.65 * broad + fibers * 1.15 - crypt * 0.8 - rings * 0.15).max(0.08)
        * (1. - limbal * 0.9)
        * (1. - ruff);
    let edge = smooth(pupil - 0.004, pupil + 0.004, radius);
    let rgb = std::array::from_fn::<_, 3, _>(|i| {
        let pigment = v[i] + (v[i + 3] - v[i]) * inner;
        (0.0015 + (pigment * brightness - 0.0015) * edge).clamp(0., 1.)
    });
    Sample {
        color_roughness: [rgb[0], rgb[1], rgb[2], v[15]],
        response: [
            0.,
            (0.45 + fibers * 0.3 - crypt * 0.4 - rings * 0.12).clamp(0., 1.),
            1.,
            1. - crypt * 0.25,
        ],
        emission: [0.; 4],
    }
}
fn pigment(v: &[f32]) -> Procedural {
    let resolution = [64u16, 128, 256][v[20].round() as usize];
    let n = usize::from(resolution);
    let mut samples = Vec::with_capacity((n * n * 4 - 1) / 3);
    for y in 0..n {
        for x in 0..n {
            let a = texel(
                [(x as f32 + 0.25) / n as f32, (y as f32 + 0.25) / n as f32],
                v,
            );
            let b = texel(
                [(x as f32 + 0.75) / n as f32, (y as f32 + 0.25) / n as f32],
                v,
            );
            let c = texel(
                [(x as f32 + 0.25) / n as f32, (y as f32 + 0.75) / n as f32],
                v,
            );
            let d = texel(
                [(x as f32 + 0.75) / n as f32, (y as f32 + 0.75) / n as f32],
                v,
            );
            samples.push(a.mix(b, 0.5).mix(c.mix(d, 0.5), 0.5));
        }
    }
    let mut size = n;
    let mut start = 0;
    while size > 1 {
        let next = size / 2;
        for y in 0..next {
            for x in 0..next {
                let i = start + 2 * y * size + 2 * x;
                samples.push(
                    samples[i]
                        .mix(samples[i + 1], 0.5)
                        .mix(samples[i + size].mix(samples[i + size + 1], 0.5), 0.5),
                );
            }
        }
        start += size * size;
        size = next;
    }
    Procedural {
        version: materials::recipe::VERSION,
        recipe: Recipe {
            tile_size: [v[8] * 2.; 2],
            height_m: v[14],
            plane: materials::Plane::Xy,
            ..Default::default()
        },
        fixed: Some(Fixed {
            resolution,
            samples: Arc::new(samples),
        }),
    }
}
