//! Periodic, seeded local rock surfaces. Mineral albedo contains no lighting.
use super::{
    marble::{crystal_grain, fbm},
    recipe::{NormalFilter, Recipe, Sample, hash, noise},
};
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VERSION: u16 = 1;
pub const SCENE_VERSION: u32 = 117;
pub(crate) const WORDS: usize = 9;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rock {
    pub version: u16,
    pub dark: [f32; 3],
    pub light: [f32; 3],
    pub oxide: [f32; 3],
    pub mineral: [f32; 3],
    pub roughness: f32,
    pub contrast: f32,
    pub blocks: [u16; 2],
    pub joint_width: f32,
    pub joint_depth: f32,
    pub warp: f32,
    pub faceting: f32,
    pub layers: u16,
    pub stratification: f32,
    pub layer_warp: f32,
    pub layer_disorder: f32,
    pub grain: f32,
    pub grain_frequency: u16,
    pub oxidation: f32,
    pub inclusions: f32,
    pub pitting: f32,
    pub weathering: f32,
    pub roughness_variation: f32,
    pub detail_height: f32,
    pub secondary_joints: f32,
    pub erosion: f32,
    /// Integer directions preserve periodicity. X also shears the joint domain.
    pub direction: [i8; 2],
}
fn color(hex: u32) -> [f32; 3] {
    [16, 8, 0].map(|shift| {
        let c = ((hex >> shift) & 255) as f32 / 255.;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    })
}
impl Default for Rock {
    fn default() -> Self {
        Self {
            version: VERSION,
            dark: color(0x253038),
            light: color(0x89918e),
            oxide: color(0xa4754c),
            mineral: color(0xd3d1bc),
            roughness: 0.84,
            contrast: 1.25,
            blocks: [4, 5],
            joint_width: 0.035,
            joint_depth: 0.85,
            warp: 0.24,
            faceting: 0.8,
            layers: 24,
            stratification: 0.12,
            layer_warp: 0.28,
            layer_disorder: 0.7,
            grain: 0.5,
            grain_frequency: 768,
            oxidation: 0.15,
            inclusions: 0.25,
            pitting: 0.25,
            weathering: 0.45,
            roughness_variation: 0.18,
            detail_height: 0.065,
            secondary_joints: 0.6,
            erosion: 0.4,
            direction: [0, 1],
        }
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    #[default]
    Alpine,
    Fractured,
    Ochre,
    Sedimentary,
}
impl Family {
    pub const ALL: [Self; 4] = [
        Self::Alpine,
        Self::Fractured,
        Self::Ochre,
        Self::Sedimentary,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Alpine => "Rocha alpina",
            Self::Fractured => "Rocha escura fraturada",
            Self::Ochre => "Paredão ocre",
            Self::Sedimentary => "Estratos sedimentares",
        }
    }
    pub fn recipe(self) -> Recipe {
        let mut m = Rock::default();
        let depth = match self {
            Self::Alpine => {
                m.dark = color(0x59636a);
                m.light = color(0xb8bdbc);
                m.blocks = [3, 5];
                m.direction = [1, 2];
                m.faceting = 1.;
                m.warp = 0.06;
                m.stratification = 0.;
                m.joint_width = 0.018;
                m.oxidation = 0.;
                m.inclusions = 0.12;
                m.grain = 0.3;
                m.grain_frequency = 1024;
                m.weathering = 0.2;
                m.secondary_joints = 0.45;
                0.22
            }
            Self::Fractured => {
                m.dark = color(0x182125);
                m.light = color(0x737b76);
                m.joint_depth = 1.;
                m.warp = 0.1;
                m.stratification = 0.;
                m.faceting = 0.65;
                m.oxidation = 0.4;
                m.inclusions = 0.45;
                m.secondary_joints = 0.8;
                m.pitting = 0.45;
                0.13
            }
            Self::Ochre => {
                m.dark = color(0x514638);
                m.light = color(0xc0a67e);
                m.oxide = color(0xb97d49);
                m.blocks = [3, 6];
                m.direction = [1, 2];
                m.stratification = 0.08;
                m.warp = 0.12;
                m.layers = 14;
                m.layer_warp = 0.12;
                m.oxidation = 0.55;
                m.grain = 0.7;
                m.weathering = 0.7;
                m.erosion = 0.6;
                0.16
            }
            Self::Sedimentary => {
                m.dark = color(0x302e27);
                m.light = color(0xb7a084);
                m.oxide = color(0xc47a44);
                m.mineral = color(0xe3d7b6);
                m.blocks = [3, 2];
                m.warp = 0.15;
                m.secondary_joints = 0.18;
                m.joint_width = 0.016;
                m.joint_depth = 0.6;
                m.faceting = 0.25;
                m.layers = 56;
                m.stratification = 1.;
                m.layer_warp = 0.7;
                m.layer_disorder = 0.95;
                m.grain = 0.7;
                m.oxidation = 0.65;
                m.weathering = 0.65;
                m.erosion = 0.8;
                m.detail_height = 0.035;
                0.095
            }
        };
        Recipe {
            rock: Some(m),
            normal_filter: NormalFilter::Pixel,
            seed: 4271,
            height_m: depth,
            ..Default::default()
        }
    }
}
impl Rock {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self
                .dark
                .iter()
                .chain(&self.light)
                .chain(&self.oxide)
                .chain(&self.mineral)
                .any(|v| !v.is_finite() || !(0. ..=2.).contains(v))
            || !self.roughness.is_finite()
            || !(0.2..=1.).contains(&self.roughness)
            || !self.contrast.is_finite()
            || !(0.5..=3.).contains(&self.contrast)
            || self.blocks.iter().any(|v| !(1..=32).contains(v))
            || !(2..=128).contains(&self.layers)
            || !(128..=2048).contains(&self.grain_frequency)
            || !self.joint_width.is_finite()
            || !(0.002..=0.2).contains(&self.joint_width)
            || self.direction == [0, 0]
            || self.direction.iter().any(|v| !(-4..=4).contains(v))
            || [
                self.joint_depth,
                self.warp,
                self.faceting,
                self.stratification,
                self.layer_warp,
                self.layer_disorder,
                self.grain,
                self.oxidation,
                self.inclusions,
                self.pitting,
                self.weathering,
                self.roughness_variation,
                self.detail_height,
                self.secondary_joints,
                self.erosion,
            ]
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        {
            return Err(invalid(
                "Invalid rock1 palette, block/layer/grain frequency, direction, joint or finish controls",
            ));
        }
        Ok(())
    }
    pub(crate) fn packed(&self) -> [[f32; 4]; WORDS] {
        [
            [self.dark[0], self.dark[1], self.dark[2], self.roughness],
            [self.light[0], self.light[1], self.light[2], self.contrast],
            [self.oxide[0], self.oxide[1], self.oxide[2], self.oxidation],
            [
                self.mineral[0],
                self.mineral[1],
                self.mineral[2],
                self.inclusions,
            ],
            [
                f32::from(self.blocks[0]),
                f32::from(self.blocks[1]),
                self.joint_width,
                self.joint_depth,
            ],
            [
                self.warp,
                self.faceting,
                self.grain,
                f32::from(self.grain_frequency),
            ],
            [
                f32::from(self.layers),
                self.stratification,
                self.layer_warp,
                self.layer_disorder,
            ],
            [
                f32::from(self.direction[0]),
                f32::from(self.direction[1]),
                self.pitting,
                self.weathering,
            ],
            [
                self.roughness_variation,
                self.detail_height,
                self.secondary_joints,
                self.erosion,
            ],
        ]
    }
    pub fn evaluate(&self, uv: [f32; 2], footprint: f32, seed: u32) -> Sample {
        let sd = |v| seed.wrapping_add(v);
        let a = fbm(uv, 2, sd(31), 4, footprint);
        let b = fbm(uv, 3, sd(71), 4, footprint);
        let q = [
            uv[0]
                + f32::from(self.direction[0]) * uv[1]
                + self.warp * (a - 0.5)
                + 0.006 * (fbm(uv, 73, sd(89), 1, footprint) - 0.5),
            uv[1] + self.warp * (b - 0.5) + 0.006 * (fbm(uv, 73, sd(101), 1, footprint) - 0.5),
        ];
        let fp = footprint * (1. + self.warp * 3. + f32::from(self.direction[0]).abs());
        let n = self.blocks.map(i32::from);
        let primary = cell(q, n, sd(151));
        let secondary = cell(q, [n[0] * 3, n[1] * 2], sd(233));
        let coarse = fbm(q, 7, sd(317), 4, fp);
        let small = fbm(q, 27, sd(389), 4, fp);
        let grain = crystal_grain(uv, i32::from(self.grain_frequency), sd(431), footprint);
        let aa = (fp * n[0].max(n[1]) as f32 * 0.5).min(0.18);
        let width = self.joint_width * (0.25 + 2.5 * smooth(0.2, 0.8, small).powi(2));
        let main = 1. - smooth((width * 0.45 - aa).max(0.), width + aa, primary[0]);
        let sub =
            (1. - smooth(
                (width * 0.12 - aa * 2.).max(0.),
                width * 0.4 + aa * 2.,
                secondary[0],
            )) * smooth(0.27, 0.7, coarse)
                * self.secondary_joints;
        let joint = main.max(sub);
        let slope = [fract(primary[1] * 17.) - 0.5, fract(primary[1] * 43.) - 0.5];
        let plane =
            0.62 + 0.3 * (primary[1] - 0.5) + 1.5 * (primary[2] * slope[0] + primary[3] * slope[1]);
        let cut = 0.9 - 0.75 * (primary[2] * 0.7 - primary[3] * 0.8 + primary[1] - 0.5).abs();
        let bevel = smooth(width, width + 0.12 * self.weathering + 0.005, primary[0]);
        let face = mix(0.6, plane.min(cut).clamp(0.15, 0.95), self.faceting)
            * mix(1., 0.7 + 0.3 * bevel, self.weathering);
        let count = i32::from(self.layers);
        let mut phase = (uv[0] * f32::from(self.direction[0])
            + uv[1] * f32::from(self.direction[1]))
            * count as f32
            + self.layer_warp * count as f32 * 0.2 * (a - 0.5)
            + 0.12 * (small - 0.5);
        phase += self.layer_disorder
            * 1.6
            * (noise(
                [phase / count as f32, 0.37],
                [(count / 3).max(1), 1],
                sd(503),
            ) - 0.5);
        let layer = phase.floor() as i32;
        let t = fract(phase);
        let hardness = hash(layer, 0, [count, 1], sd(547));
        let rise = smooth(0.015, 0.08, t);
        let lip = 1. - smooth(0.68, 0.98, t);
        let shelf = rise * lip;
        let lamination_raw = (std::f32::consts::PI * (phase * 4. + small * 0.65))
            .sin()
            .abs();
        let direction_length =
            (f32::from(self.direction[0]).powi(2) + f32::from(self.direction[1]).powi(2)).sqrt();
        let lamination = mix(
            0.5,
            lamination_raw,
            1. - smooth(0.25, 0.75, footprint * count as f32 * 4. * direction_length),
        );
        let layer_height = 0.24 + 0.48 * hardness + 0.2 * shelf
            - 0.045 * (1. - lamination)
            - self.erosion * 0.2 * (1. - shelf);
        let pits = smooth(0.57, 0.78, small) * self.pitting;
        let height = (mix(face, layer_height, self.stratification * 0.9)
            - joint * self.joint_depth * 0.5
            - pits * 0.12
            + self.detail_height
                * ((small - 0.5) * 0.75 + (fbm(uv, 96, sd(467), 1, footprint) - 0.5) * 0.08))
            .clamp(0.02, 0.98);
        let mut value = mix(
            0.34 + 0.46 * coarse + 0.22 * (primary[1] - 0.5),
            0.18 + 0.68 * hardness + 0.12 * shelf,
            self.stratification,
        );
        value = (value
            + self.grain * 0.24 * (grain - 0.5)
            + 0.35 * (small - 0.5)
            + self.stratification * 0.06 * (lamination - 0.5))
            .clamp(0., 1.)
            .powf(self.contrast);
        let mut rgb = rgb_mix(self.dark, self.light, value);
        let rust = mix(
            smooth(0.46, 0.76, small) * (joint * 0.65 + smooth(0.68, 0.83, coarse) * 0.35),
            smooth(0.25, 0.72, hardness) * (0.55 + 0.45 * small),
            self.stratification,
        ) * self.oxidation;
        rgb = rgb_mix(rgb, self.oxide, rust);
        let crystals = smooth(0.55, 0.8, small) * smooth(0.59, 0.86, grain) * self.inclusions;
        let seams = (1. - smooth(0.008, 0.035, t))
            * smooth(0.82, 0.96, hardness)
            * self.stratification
            * self.inclusions;
        rgb = rgb_mix(rgb, self.mineral, crystals.max(seams));
        let roughness =
            (self.roughness + self.roughness_variation * (grain - 0.5) + 0.1 * pits + 0.06 * joint)
                .clamp(0.2, 1.);
        let ao = (1.
            - 0.5 * joint * self.joint_depth
            - 0.2 * (1. - shelf) * self.stratification
            - 0.12 * pits)
            .clamp(0.25, 1.);
        Sample {
            color_roughness: [rgb[0], rgb[1], rgb[2], roughness],
            response: [0., height, 1., ao],
            emission: [0.; 4],
        }
    }
}
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn rgb_mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| mix(a[i], b[i], t))
}
fn smooth(a: f32, b: f32, v: f32) -> f32 {
    let t = ((v - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn fract(v: f32) -> f32 {
    v - v.floor()
}
/// Nearest pair's bisector distance, stable domain ID and local face coordinates.
fn cell(uv: [f32; 2], n: [i32; 2], seed: u32) -> [f32; 4] {
    let p = [uv[0] * n[0] as f32, uv[1] * n[1] as f32];
    let base = p.map(|v| v.floor() as i32);
    let mut first = 100.;
    let mut second = 100.;
    let mut a = [0.; 2];
    let mut b = [0.; 2];
    let mut id = 0.;
    for y in -1..=1 {
        for x in -1..=1 {
            let c = [base[0] + x, base[1] + y];
            let q = [
                c[0] as f32 + 0.15 + 0.7 * hash(c[0], c[1], n, seed),
                c[1] as f32 + 0.15 + 0.7 * hash(c[0], c[1], n, seed.wrapping_add(11)),
            ];
            let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2);
            if d < first {
                second = first;
                b = a;
                first = d;
                a = q;
                id = hash(c[0], c[1], n, seed.wrapping_add(912));
            } else if d < second {
                second = d;
                b = q;
            }
        }
    }
    [
        0.5 * (second - first)
            / ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2))
                .sqrt()
                .max(0.00001),
        id,
        p[0] - a[0],
        p[1] - a[1],
    ]
}
