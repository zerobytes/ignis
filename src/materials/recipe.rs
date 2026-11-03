//! Deterministic, periodic surface fields shared by fixed baking and live materials.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const VERSION: u16 = 1;
pub const MAX_LAYERS: usize = 8;
pub const MAX_FILE_BYTES: usize = 32 * 1024 * 1024;

#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    JsonSchema,
    PartialEq,
    bytemuck::Pod,
    bytemuck::Zeroable,
)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    /// Linear base-color multiplier and perceptual roughness.
    pub color_roughness: [f32; 4],
    /// Metalness, normalized height, opacity, indirect occlusion.
    pub response: [f32; 4],
    /// Linear emission RGB and thin-surface diffuse transmission.
    pub emission: [f32; 4],
}
impl Default for Sample {
    fn default() -> Self {
        Self {
            color_roughness: [1., 1., 1., 0.7],
            response: [0., 0.5, 1., 1.],
            emission: [0.; 4],
        }
    }
}
impl Sample {
    fn valid(&self) -> bool {
        self.color_roughness[..3]
            .iter()
            .all(|v| v.is_finite() && (0. ..=2.).contains(v))
            && self.color_roughness[3].is_finite()
            && (0.2..=1.).contains(&self.color_roughness[3])
            && self
                .response
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v))
            && self.emission[..3]
                .iter()
                .all(|v| v.is_finite() && (0. ..=64.).contains(v))
            && self.emission[3].is_finite()
            && (0. ..=1.).contains(&self.emission[3])
            && (self.response[0] == 0. || self.emission[3] == 0.)
    }
    pub fn mix(self, b: Self, t: f32) -> Self {
        fn blend(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
            std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
        }
        Self {
            color_roughness: blend(self.color_roughness, b.color_roughness, t),
            response: blend(self.response, b.response, t),
            emission: blend(self.emission, b.emission, t),
        }
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    #[default]
    Noise,
    Ridged,
    Cells,
    Bricks,
    Waves,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub field: Field,
    pub frequency: [u16; 2],
    pub seed: u32,
    pub octaves: u8,
    pub warp: f32,
    pub threshold: [f32; 2],
    pub strength: f32,
    pub invert: bool,
    pub target: Sample,
}
impl Default for Layer {
    fn default() -> Self {
        Self {
            field: Field::Noise,
            frequency: [8, 8],
            seed: 0,
            octaves: 3,
            warp: 0.,
            threshold: [0., 1.],
            strength: 1.,
            invert: false,
            target: Sample::default(),
        }
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Relief {
    #[default]
    Bump,
    Parallax,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub seed: u32,
    /// Stable instance-ID variation is evaluated only in runtime mode.
    pub unique_instances: bool,
    pub plane: super::Plane,
    /// Repeat size in rest-material units; identity instance units are metres.
    pub tile_size: [f32; 2],
    pub base: Sample,
    pub layers: Vec<Layer>,
    pub height_m: f32,
    pub relief: Relief,
    pub view_steps: u8,
    pub shadow_steps: u8,
}
impl Default for Recipe {
    fn default() -> Self {
        Self {
            seed: 1,
            unique_instances: false,
            plane: super::Plane::Xy,
            tile_size: [1.; 2],
            base: Sample::default(),
            layers: vec![],
            height_m: 0.,
            relief: Relief::Bump,
            view_steps: 16,
            shadow_steps: 8,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Fixed {
    pub resolution: u16,
    /// Complete box-filtered mip chain, largest first, texel centers, wrap repeat.
    pub samples: Arc<Vec<Sample>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Procedural {
    pub version: u16,
    pub recipe: Recipe,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed: Option<Fixed>,
}
impl Procedural {
    pub fn runtime(recipe: Recipe) -> Self {
        Self {
            version: VERSION,
            recipe,
            fixed: None,
        }
    }
    pub fn validate(&self) -> Result<()> {
        let r = &self.recipe;
        if self.version != VERSION
            || r.layers.len() > MAX_LAYERS
            || !r.base.valid()
            || r.tile_size
                .iter()
                .any(|v| !v.is_finite() || !(0.001..=10000.).contains(v))
            || !r.height_m.is_finite()
            || !(0. ..=0.25).contains(&r.height_m)
            || !(8..=48).contains(&r.view_steps)
            || r.shadow_steps > 24
        {
            return Err(invalid(
                "Material recipe1 requires <=8 layers, valid channels, tile0.001..10000, height0..0.25m, view steps8..48 and shadow steps0..24",
            ));
        }
        for l in &r.layers {
            if !l.target.valid()
                || l.frequency.iter().any(|f| !(1..=64).contains(f))
                || !(1..=4).contains(&l.octaves)
                || !l.warp.is_finite()
                || !(0. ..=1.).contains(&l.warp)
                || l.threshold
                    .iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
                || l.threshold[0] >= l.threshold[1]
                || !l.strength.is_finite()
                || !(0. ..=1.).contains(&l.strength)
            {
                return Err(invalid(
                    "Invalid procedural layer: frequency1..64, octaves1..4, ordered mask0..1, warp/strength0..1",
                ));
            }
        }
        let metal = r.base.response[0] > 0. || r.layers.iter().any(|l| l.target.response[0] > 0.);
        let transmission =
            r.base.emission[3] > 0. || r.layers.iter().any(|l| l.target.emission[3] > 0.);
        if metal && transmission {
            return Err(invalid(
                "Procedural metal and thin transmission cannot be mixed",
            ));
        }
        if let Some(f) = &self.fixed
            && (!matches!(f.resolution, 16 | 32 | 64 | 128 | 256)
                || f.samples.len() != sample_count(f.resolution)
                || f.samples.iter().any(|s| !s.valid())
                || (f.samples.iter().any(|s| s.response[0] > 0.)
                    && f.samples.iter().any(|s| s.emission[3] > 0.)))
        {
            return Err(invalid(
                "Fixed material requires resolution16/32/64/128/256 and a complete finite mip chain",
            ));
        }

        Ok(())
    }
    pub fn translucent(&self) -> bool {
        self.fixed.as_ref().map_or_else(
            || {
                self.recipe.base.response[2] < 1.
                    || self
                        .recipe
                        .layers
                        .iter()
                        .any(|l| l.strength > 0. && l.target.response[2] < 1.)
            },
            |f| f.samples.iter().any(|s| s.response[2] < 1.),
        )
    }
    pub fn bake(&self, resolution: u16) -> Result<Self> {
        self.validate()?;
        if !matches!(resolution, 16 | 32 | 64 | 128 | 256) {
            return Err(invalid("Bake resolution must be16/32/64/128/256"));
        }
        let n = usize::from(resolution);
        let mut samples = Vec::with_capacity(sample_count(resolution));
        for y in 0..n {
            for x in 0..n {
                // Four strata suppress fine-field aliasing in the first mip.
                let mut s = Sample::zeroed_value();
                for dy in [0.25, 0.75] {
                    for dx in [0.25, 0.75] {
                        let v = self
                            .recipe
                            .evaluate([(x as f32 + dx) / n as f32, (y as f32 + dy) / n as f32], 0.);
                        s.add_scaled(v, 0.25);
                    }
                }
                samples.push(s);
            }
        }
        let mut size = n;
        let mut start = 0;
        while size > 1 {
            let next = size / 2;
            for y in 0..next {
                for x in 0..next {
                    let at = start + 2 * y * size + 2 * x;
                    let a = samples[at].mix(samples[at + 1], 0.5);
                    let b = samples[at + size].mix(samples[at + size + 1], 0.5);
                    samples.push(a.mix(b, 0.5));
                }
            }
            start += size * size;
            size = next;
        }
        Ok(Self {
            version: VERSION,
            recipe: self.recipe.clone(),
            fixed: Some(Fixed {
                resolution,
                samples: Arc::new(samples),
            }),
        })
    }
    pub fn sample(&self, uv: [f32; 2], footprint: f32) -> Sample {
        if let Some(f) = &self.fixed {
            f.sample(uv, footprint)
        } else {
            self.recipe.evaluate(uv, footprint)
        }
    }
    pub fn instance(&self, id: &str) -> Option<Self> {
        if self.fixed.is_some() || !self.recipe.unique_instances {
            return None;
        }
        let mut out = self.clone();
        let mut seed = self.recipe.seed ^ 2166136261;
        for b in id.bytes() {
            seed = (seed ^ u32::from(b)).wrapping_mul(16777619);
        }
        out.recipe.seed = seed;
        out.recipe.unique_instances = false;
        Some(out)
    }
}
impl Sample {
    fn zeroed_value() -> Self {
        Self {
            color_roughness: [0.; 4],
            response: [0.; 4],
            emission: [0.; 4],
        }
    }
    fn add_scaled(&mut self, b: Self, scale: f32) {
        for i in 0..4 {
            self.color_roughness[i] += b.color_roughness[i] * scale;
            self.response[i] += b.response[i] * scale;
            self.emission[i] += b.emission[i] * scale;
        }
    }
}
pub fn sample_count(mut size: u16) -> usize {
    let mut count = 0;
    while size > 0 {
        count += usize::from(size).pow(2);
        size /= 2;
    }
    count
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn hash(x: i32, y: i32, n: [i32; 2], seed: u32) -> f32 {
    let x = x.rem_euclid(n[0]) as u32;
    let y = y.rem_euclid(n[1]) as u32;
    let mut h = x.wrapping_mul(1597334677) ^ y.wrapping_mul(3812015801) ^ seed;
    h = (h ^ (h >> 16)).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(3266489917);
    h ^= h >> 16;
    (h & 16777215) as f32 / 16777215.
}
fn noise(uv: [f32; 2], n: [i32; 2], seed: u32) -> f32 {
    let p = [uv[0] * n[0] as f32, uv[1] * n[1] as f32];
    let b = p.map(|p| p.floor() as i32);
    let f = [p[0] - p[0].floor(), p[1] - p[1].floor()].map(|v| v * v * (3. - 2. * v));
    mix(
        mix(
            hash(b[0], b[1], n, seed),
            hash(b[0] + 1, b[1], n, seed),
            f[0],
        ),
        mix(
            hash(b[0], b[1] + 1, n, seed),
            hash(b[0] + 1, b[1] + 1, n, seed),
            f[0],
        ),
        f[1],
    )
}
fn field(uv: [f32; 2], n: [i32; 2], seed: u32, kind: Field, aa: f32) -> f32 {
    if kind == Field::Noise {
        return noise(uv, n, seed);
    }
    if kind == Field::Ridged {
        return 1. - (2. * noise(uv, n, seed) - 1.).abs();
    }
    if kind == Field::Waves {
        return 0.5
            + 0.5
                * (std::f32::consts::TAU * (uv[0] * n[0] as f32 + noise(uv, [2, 2], seed))).sin();
    }
    let mut p = [uv[0] * n[0] as f32, uv[1] * n[1] as f32];
    if kind == Field::Bricks {
        p[0] +=
            0.5 * (p[1].floor() as i32).rem_euclid(2) as f32 * if n[1] % 2 == 0 { 1. } else { 0. };
        let b = p.map(|v| v.floor() as i32);
        let f = p.map(|v| v - v.floor());
        let edge = f[0].min(1. - f[0]).min(f[1].min(1. - f[1]));
        return smooth(0.035 - aa, 0.065 + aa, edge) * (0.7 + 0.3 * hash(b[0], b[1], n, seed));
    }
    let b = p.map(|v| v.floor() as i32);
    let mut first = 100f32;
    let mut second = 100f32;
    for y in -1..=1 {
        for x in -1..=1 {
            let c = [b[0] + x, b[1] + y];
            let dx = p[0] - c[0] as f32 - hash(c[0], c[1], n, seed);
            let dy = p[1] - c[1] as f32 - hash(c[0], c[1], n, seed.wrapping_add(11));
            let d = (dx * dx + dy * dy).sqrt();
            if d < first {
                second = first;
                first = d;
            } else if d < second {
                second = d;
            }
        }
    }
    smooth(0.02 - aa, 0.12 + aa, second - first)
}
impl Recipe {
    pub fn evaluate(&self, uv: [f32; 2], footprint: f32) -> Sample {
        let uv = uv.map(|v| v - v.floor());
        let mut sample = self.base;
        for l in &self.layers {
            let seed = self.seed.wrapping_add(l.seed);
            let mut q = uv;
            if l.warp > 0. {
                q[0] += l.warp * (noise(uv, [2, 2], seed.wrapping_add(11)) - 0.5);
                q[1] += l.warp * (noise(uv, [2, 2], seed.wrapping_add(23)) - 0.5);
            }
            let mut value = 0.;
            let mut total = 0.;
            let mut weight = 0.5;
            let mut n = l.frequency.map(i32::from);
            for octave in 0..l.octaves {
                let fp = footprint * n[0].max(n[1]) as f32 * (1. + l.warp * 4.);
                let coverage = 1. - smooth(0.25, 0.75, fp);
                let v = if coverage > 0. {
                    field(
                        q,
                        n,
                        seed.wrapping_add(u32::from(octave) * 1013),
                        l.field,
                        fp * 0.5,
                    )
                } else {
                    0.5
                };
                value += weight * mix(0.5, v, coverage);
                total += weight;
                weight *= 0.5;
                n = n.map(|v| v * 2);
            }
            let mut mask = smooth(l.threshold[0], l.threshold[1], value / total);
            if l.invert {
                mask = 1. - mask;
            }
            sample = sample.mix(l.target, mask * l.strength);
        }
        sample
    }
    pub fn coordinate(&self, rest: [f32; 3]) -> [f32; 2] {
        let q = match self.plane {
            super::Plane::Xy => [rest[0], rest[1]],
            super::Plane::Xz => [rest[0], rest[2]],
            super::Plane::Yz => [rest[1], rest[2]],
        };
        [q[0] / self.tile_size[0], q[1] / self.tile_size[1]]
    }
}
impl Fixed {
    fn level(&self, uv: [f32; 2], level: u32) -> Sample {
        let mut n = usize::from(self.resolution);
        let mut start = 0;
        for _ in 0..level {
            start += n * n;
            n /= 2;
        }
        let p = uv.map(|v| (v - v.floor()) * n as f32 - 0.5);
        let b = p.map(|v| v.floor() as i32);
        let f = p.map(|v| v - v.floor());
        let at = |x: i32, y: i32| {
            self.samples
                [start + y.rem_euclid(n as i32) as usize * n + x.rem_euclid(n as i32) as usize]
        };
        at(b[0], b[1])
            .mix(at(b[0] + 1, b[1]), f[0])
            .mix(at(b[0], b[1] + 1).mix(at(b[0] + 1, b[1] + 1), f[0]), f[1])
    }
    pub fn sample(&self, uv: [f32; 2], footprint: f32) -> Sample {
        let max = self.resolution.ilog2();
        let lod = (footprint * f32::from(self.resolution))
            .max(1.)
            .log2()
            .clamp(0., max as f32);
        let level = lod.floor() as u32;
        self.level(uv, level)
            .mix(self.level(uv, (level + 1).min(max)), lod - level as f32)
    }
}

/// Packed vec4 words: domain, relief, base sample, program metadata, layer records
/// or fixed mip samples. Offsets are transient renderer state, never asset IDs.
pub(crate) fn storage_bytes(source: &[super::Material]) -> u64 {
    let mut seen = std::collections::BTreeSet::new();
    let mut bytes = 0u64;
    for m in source {
        if let Some(p) = &m.procedural
            && seen.insert(Arc::as_ptr(p) as usize)
        {
            bytes = bytes.saturating_add(96).saturating_add(
                p.fixed
                    .as_ref()
                    .map_or((p.recipe.layers.len() * 96) as u64, |f| {
                        (f.samples.len() * 48) as u64
                    }),
            );
        }
    }
    bytes.max(16)
}
pub(crate) fn pack(source: &[super::Material]) -> (Vec<super::GpuMaterial>, Vec<[f32; 4]>) {
    let mut data = Vec::new();
    let mut table = Vec::with_capacity(source.len());
    let mut offsets = std::collections::BTreeMap::new();
    for material in source {
        let mut gpu = super::GpuMaterial::from(material);
        if let Some(p) = &material.procedural {
            let key = Arc::as_ptr(p) as usize;
            let offset = *offsets.entry(key).or_insert_with(|| {
                let offset = data.len() as u32;
                let r = &p.recipe;
                data.push([
                    r.tile_size[0],
                    r.tile_size[1],
                    match r.plane {
                        super::Plane::Xy => 0.,
                        super::Plane::Xz => 1.,
                        super::Plane::Yz => 2.,
                    },
                    u8::from(r.relief == Relief::Parallax) as f32,
                ]);
                data.push([
                    r.height_m,
                    f32::from(r.view_steps),
                    f32::from(r.shadow_steps),
                    0.,
                ]);
                data.extend([r.base.color_roughness, r.base.response, r.base.emission]);
                data.push([
                    f32::from((r.seed & 65535) as u16),
                    r.layers.len() as f32,
                    f32::from((r.seed >> 16) as u16),
                    0.,
                ]);
                if let Some(f) = &p.fixed {
                    for s in f.samples.iter() {
                        data.extend([s.color_roughness, s.response, s.emission]);
                    }
                } else {
                    for l in &r.layers {
                        data.extend([
                            [
                                match l.field {
                                    Field::Noise => 0.,
                                    Field::Ridged => 1.,
                                    Field::Cells => 2.,
                                    Field::Bricks => 3.,
                                    Field::Waves => 4.,
                                },
                                f32::from(l.frequency[0]),
                                f32::from(l.frequency[1]),
                                f32::from((l.seed & 65535) as u16),
                            ],
                            [
                                f32::from(l.octaves),
                                l.warp,
                                f32::from((l.seed >> 16) as u16),
                                0.,
                            ],
                            [
                                l.threshold[0],
                                l.threshold[1],
                                l.strength,
                                u8::from(l.invert) as f32,
                            ],
                            l.target.color_roughness,
                            l.target.response,
                            l.target.emission,
                        ]);
                    }
                }
                offset
            });
            gpu.procedural = [
                if p.fixed.is_some() { 2 } else { 1 },
                offset,
                p.fixed.as_ref().map_or(0, |f| u32::from(f.resolution)),
                0,
            ];
        }
        table.push(gpu);
    }
    if data.is_empty() {
        data.push([0.; 4]);
    }
    (table, data)
}
