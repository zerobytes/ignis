//! Seeded mineral structure, not a sampled photograph. CPU and WGSL use the same
//! periodic operators; high-resolution maps are an optional execution strategy.
use super::recipe::{Recipe, Sample, hash, noise};
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VERSION: u16 = 1;
pub const SCENE_VERSION: u32 = 115;
pub(crate) const WORDS: usize = 8;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Marble {
    pub version: u16,
    pub dark: [f32; 3],
    pub light: [f32; 3],
    pub calcite: [f32; 3],
    pub gold: [f32; 3],
    pub roughness: f32,
    pub cloud_contrast: f32,
    pub white_strength: f32,
    pub gold_strength: f32,
    pub network: u16,
    pub white_width: f32,
    pub warp: f32,
    pub banding: f32,
    pub main_vein: f32,
    pub fine_veins: f32,
    pub grain: f32,
    pub gold_metallic: f32,
    pub direction: [i8; 2],
    pub fold: f32,
    pub gold_width: f32,
    pub gold_flow: f32,
    pub breccia: f32,
    pub grain_frequency: u16,
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
impl Default for Marble {
    fn default() -> Self {
        Self {
            version: VERSION,
            dark: color(0x011a15),
            light: color(0x20513e),
            calcite: color(0xf2f0dc),
            gold: color(0xe0c06b),
            roughness: 0.22,
            cloud_contrast: 1.7,
            white_strength: 0.85,
            gold_strength: 0.,
            network: 4,
            white_width: 0.011,
            warp: 0.08,
            banding: 0.12,
            main_vein: 0.,
            fine_veins: 0.6,
            grain: 0.25,
            gold_metallic: 0.,
            direction: [1, 1],
            fold: 0.65,
            gold_width: 0.008,
            gold_flow: 0.,
            breccia: 0.6,
            grain_frequency: 768,
        }
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    #[default]
    VerdeBreccia,
    EmeraldWhiteGold,
    EmeraldGold,
    Carrara,
    Calacatta,
    Statuario,
    NeroMarquina,
    Portoro,
    Emperador,
    RossoLevanto,
    Rosa,
    Azul,
}
impl Family {
    pub const ALL: [Self; 12] = [
        Self::VerdeBreccia,
        Self::EmeraldWhiteGold,
        Self::EmeraldGold,
        Self::Carrara,
        Self::Calacatta,
        Self::Statuario,
        Self::NeroMarquina,
        Self::Portoro,
        Self::Emperador,
        Self::RossoLevanto,
        Self::Rosa,
        Self::Azul,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::VerdeBreccia => "Verde brechado",
            Self::EmeraldWhiteGold => "Esmeralda · branco e ouro",
            Self::EmeraldGold => "Esmeralda · ouro",
            Self::Carrara => "Carrara",
            Self::Calacatta => "Calacatta",
            Self::Statuario => "Statuario",
            Self::NeroMarquina => "Nero Marquina",
            Self::Portoro => "Portoro",
            Self::Emperador => "Emperador",
            Self::RossoLevanto => "Rosso Levanto",
            Self::Rosa => "Rosa",
            Self::Azul => "Azul",
        }
    }
    pub fn recipe(self) -> Recipe {
        let mut m = Marble::default();
        match self {
            Self::VerdeBreccia => {}
            Self::EmeraldWhiteGold => {
                m.dark = color(0x001c19);
                m.light = color(0x167a62);
                m.warp = 0.24;
                m.banding = 0.65;
                m.white_width = 0.1;
                m.calcite = color(0xf8fff8);
                m.main_vein = 1.;
                m.white_strength = 1.;
                m.fine_veins = 0.55;
                m.gold_strength = 0.85;
                m.gold_width = 0.006;
                m.gold_metallic = 0.8;
                m.breccia = 0.08;
            }
            Self::EmeraldGold => {
                m.dark = color(0x000e0e);
                m.light = color(0x238d7d);
                m.warp = 0.28;
                m.fold = 0.85;
                m.gold_flow = 0.85;
                m.banding = 0.8;
                m.white_width = 0.015;
                m.white_strength = 0.1;
                m.fine_veins = 0.3;
                m.gold_strength = 1.;
                m.gold_width = 0.09;
                m.gold_metallic = 0.85;
                m.breccia = 0.;
            }
            Self::Carrara => {
                m.dark = color(0xe0e2df);
                m.light = color(0xfafbf8);
                m.calcite = color(0x777e87);
                m.white_width = 0.006;
                m.white_strength = 0.35;
                m.banding = 0.15;
                m.main_vein = 0.18;
                m.warp = 0.32;
                m.breccia = 0.;
                m.grain = 0.12;
            }
            Self::Calacatta => {
                m.dark = color(0xe8e4d9);
                m.light = color(0xfffdf4);
                m.calcite = color(0x5d6266);
                m.network = 2;
                m.white_width = 0.014;
                m.white_strength = 0.65;
                m.main_vein = 0.8;
                m.gold_strength = 0.7;
                m.gold_width = 0.006;
                m.banding = 0.28;
                m.breccia = 0.;
                m.warp = 0.4;
            }
            Self::Statuario => {
                m.dark = color(0xdde0de);
                m.light = color(0xfafbf8);
                m.calcite = color(0x454d58);
                m.network = 2;
                m.white_width = 0.021;
                m.main_vein = 0.8;
                m.white_strength = 0.65;
                m.banding = 0.22;
                m.breccia = 0.;
            }
            Self::NeroMarquina => {
                m.dark = color(0x020607);
                m.light = color(0x232b2b);
                m.calcite = color(0xf3f2e9);
                m.white_width = 0.009;
                m.white_strength = 0.9;
                m.network = 3;
                m.main_vein = 0.3;
                m.breccia = 0.15;
            }
            Self::Portoro => {
                m.dark = color(0x030606);
                m.light = color(0x30312a);
                m.white_strength = 0.03;
                m.gold_strength = 1.;
                m.gold_width = 0.024;
                m.gold_flow = 0.45;
                m.gold_metallic = 0.75;
                m.network = 3;
                m.banding = 0.3;
                m.breccia = 0.1;
            }
            Self::Emperador => {
                m.dark = color(0x25180f);
                m.light = color(0x806247);
                m.calcite = color(0xd6c7a7);
                m.network = 5;
                m.white_width = 0.009;
                m.white_strength = 0.8;
                m.fine_veins = 0.85;
                m.breccia = 0.85;
                m.banding = 0.08;
            }
            Self::RossoLevanto => {
                m.dark = color(0x260a12);
                m.light = color(0x893748);
                m.calcite = color(0xf0dfcf);
                m.network = 4;
                m.white_width = 0.013;
                m.white_strength = 0.9;
                m.breccia = 0.65;
            }
            Self::Rosa => {
                m.dark = color(0xa4777b);
                m.light = color(0xf1d8cc);
                m.calcite = color(0xf9f2e7);
                m.white_strength = 0.7;
                m.banding = 0.4;
                m.network = 3;
                m.breccia = 0.3;
            }
            Self::Azul => {
                m.dark = color(0x092435);
                m.light = color(0x6a9ca9);
                m.calcite = color(0xe2e7dc);
                m.banding = 0.6;
                m.white_width = 0.019;
                m.main_vein = 0.3;
                m.breccia = 0.2;
            }
        }
        Recipe {
            seed: 931,
            marble: Some(m),
            height_m: 0.00004,
            ..Default::default()
        }
    }
}
impl Marble {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self
                .dark
                .iter()
                .chain(&self.light)
                .chain(&self.calcite)
                .chain(&self.gold)
                .any(|v| !v.is_finite() || !(0. ..=2.).contains(v))
            || !self.roughness.is_finite()
            || !(0.2..=1.).contains(&self.roughness)
            || !self.cloud_contrast.is_finite()
            || !(0.5..=3.).contains(&self.cloud_contrast)
            || !(1..=12).contains(&self.network)
            || !(128..=1024).contains(&self.grain_frequency)
            || self.direction == [0, 0]
            || self.direction.iter().any(|d| !(-4..=4).contains(d))
            || !self.fold.is_finite()
            || !(0. ..=4.).contains(&self.fold)
            || [self.white_width, self.gold_width]
                .iter()
                .any(|v| !v.is_finite() || !(0.001..=0.15).contains(v))
            || [
                self.white_strength,
                self.gold_strength,
                self.warp,
                self.banding,
                self.main_vein,
                self.fine_veins,
                self.grain,
                self.gold_metallic,
                self.breccia,
                self.gold_flow,
            ]
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        {
            return Err(invalid(
                "Invalid marble1 palette, density, grain, vein width, contrast, direction or surface controls",
            ));
        }
        Ok(())
    }
    pub(crate) fn packed(&self) -> [[f32; 4]; WORDS] {
        [
            [self.dark[0], self.dark[1], self.dark[2], self.roughness],
            [
                self.light[0],
                self.light[1],
                self.light[2],
                self.cloud_contrast,
            ],
            [
                self.calcite[0],
                self.calcite[1],
                self.calcite[2],
                self.white_strength,
            ],
            [self.gold[0], self.gold[1], self.gold[2], self.gold_strength],
            [
                f32::from(self.network),
                self.white_width,
                self.warp,
                self.banding,
            ],
            [
                self.main_vein,
                self.fine_veins,
                self.grain,
                self.gold_metallic,
            ],
            [
                f32::from(self.direction[0]),
                f32::from(self.direction[1]),
                self.fold,
                self.gold_width,
            ],
            [
                self.breccia,
                f32::from(self.grain_frequency),
                self.gold_flow,
                0.,
            ],
        ]
    }
    pub fn evaluate(&self, uv: [f32; 2], footprint: f32, seed: u32) -> Sample {
        let seed = |v| seed.wrapping_add(v);
        let a = fbm(uv, 2, seed(17), 5, footprint);
        let b = fbm(uv, 3, seed(53), 5, footprint);
        let q = [
            uv[0]
                + self.warp * (a - 0.5)
                + 0.004 * (filtered_noise(uv, 96, seed(71), footprint) - 0.5),
            uv[1]
                + self.warp * (b - 0.5)
                + 0.004 * (filtered_noise(uv, 96, seed(89), footprint) - 0.5),
        ];
        let fp = footprint * (1. + self.warp * 3.);
        let cloud = fbm(q, 4, seed(97), 5, fp);
        let small = fbm(q, 15, seed(131), 3, fp);
        let flow = fbm(uv, 2, seed(17), 3, footprint);
        let phase = uv[0] * f32::from(self.direction[0])
            + uv[1] * f32::from(self.direction[1])
            + self.fold * (flow - 0.5)
            + 0.015 * (small - 0.5);
        let bands = 0.5 + 0.5 * (std::f32::consts::TAU * phase).sin();
        let folded = 0.5 + 0.5 * (std::f32::consts::TAU * (phase * 3. + small * 0.7)).sin();
        let mut value = smooth(0.22, 0.76, cloud + 0.17 * (bands - 0.5));
        value = mix(
            value,
            smooth(0.1, 0.9, bands) * 0.65 + folded * 0.35,
            self.banding,
        );
        let strata = 0.5
            + 0.5
                * (std::f32::consts::TAU * (phase * 32. + small * 2.4)).sin()
                * (1. - smooth(0.25, 0.75, fp * 64.));
        let patches = fbm(uv, 27, seed(173), 5, footprint);
        value = (value + 0.6 * (patches - 0.5) + self.banding * 0.22 * (strata - 0.5))
            .clamp(0., 1.)
            .powf(self.cloud_contrast);
        let mut color = rgb_mix(self.dark, self.light, value);
        let primary = cell(q, i32::from(self.network), seed(211));
        let secondary = cell(q, 10, seed(307));
        let hairline = cell(q, 47, seed(401));
        let aa = (fp * f32::from(self.network) * 0.5).min(0.15);
        let width_variation = smooth(0.2, 0.8, small);
        let width = self.white_width * (0.15 + 2.8 * width_variation * width_variation);
        let network_width = width * mix(1., 0.12, self.main_vein);
        let network = 1.
            - smooth(
                (network_width * 0.85 - aa).max(0.),
                network_width * 1.05 + aa,
                primary[0],
            );
        let main_width = self.white_width * (0.65 + 0.7 * small);
        let line = (std::f32::consts::PI * phase).sin().abs();
        let main = 1.
            - smooth(
                (main_width * 0.9 - footprint).max(0.),
                main_width * 1.05 + footprint,
                line,
            );
        let fine = (1. - smooth(0.003 - fp * 5., 0.013 + fp * 5., secondary[0]))
            * smooth(0.32, 0.72, cloud + 0.25 * small);
        let hair = (1. - smooth(0.001 - fp * 23.5, 0.006 + fp * 23.5, hairline[0]))
            * smooth(0.42, 0.7, small)
            * 0.65;
        let fine_fade = 1. - smooth(0.001, 0.012, fp);
        let white = (network * mix(1., 0.65, self.main_vein))
            .max(main * self.main_vein)
            .max(fine.max(hair) * self.fine_veins * fine_fade)
            * self.white_strength;
        for c in &mut color {
            *c *= (1. + self.grain * 2. * (patches - 0.5))
                * mix(
                    1.,
                    0.35 + (secondary[1] * 0.35 + hairline[1] * 0.65) * 1.2,
                    self.breccia,
                );
        }
        let quartz = rgb_mix(
            std::array::from_fn(|i| self.calcite[i] * 0.45 + self.light[i] * 0.1),
            self.calcite,
            smooth(0.15, 0.85, fbm(uv, 40, seed(487), 3, footprint)),
        );
        let quartz = rgb_mix(quartz, self.calcite, main * 0.65);
        color = rgb_mix(
            color,
            self.light,
            self.breccia * smooth(0.77, 0.94, hairline[1]) * 0.45,
        );
        color = rgb_mix(color, quartz, white);
        let gw = self.gold_width * (0.15 + 2.8 * width_variation * width_variation);
        let gold_network_width = gw * mix(1., 0.12, self.gold_flow);
        let mut gold =
            (1. - smooth(
                (gold_network_width * 0.85 - aa).max(0.),
                gold_network_width * 1.05 + aa,
                primary[0],
            )) * self.gold_strength;
        let flow_width = self.gold_width * (0.65 + 0.7 * small);
        let flow_gold = 1.
            - smooth(
                (flow_width * 0.85 - footprint).max(0.),
                flow_width * 1.05 + footprint,
                line,
            );
        gold = gold.max(flow_gold * self.gold_flow * self.gold_strength);
        gold = gold.max(
            (1. - smooth(0.002 - fp * 5., 0.009 + fp * 5., secondary[0]))
                * smooth(0.34, 0.8, cloud)
                * self.gold_strength
                * 0.8
                * fine_fade,
        );
        if self.main_vein > 0. {
            let halo = (1. - smooth(main_width * 1.05, main_width * 1.25 + footprint, line))
                * (1. - main)
                * smooth(0.38, 0.62, small);
            gold = gold.max(halo * self.gold_strength);
        }
        let grain = crystal_grain(uv, i32::from(self.grain_frequency), seed(503), footprint);
        let gold_color = rgb_mix(
            self.gold.map(|c| c * 0.45),
            self.gold,
            smooth(0.15, 0.85, grain),
        );
        color = rgb_mix(color, gold_color, gold);
        let flecks = smooth(0.6, 0.94, grain) * self.grain * 0.22 * (1. - white) * (1. - gold);
        color = rgb_mix(color, rgb_mix(self.light, self.calcite, 0.25), flecks);
        color = color.map(|v| v.clamp(0., 2.));
        let roughness = mix(
            self.roughness + 0.025 * (grain - 0.5),
            0.27 + 0.05 * grain,
            gold,
        )
        .clamp(0.2, 1.);
        Sample {
            color_roughness: [color[0], color[1], color[2], roughness],
            response: [
                gold * self.gold_metallic,
                0.5 + 0.015 * (grain - 0.5) + 0.035 * white + 0.015 * gold,
                1.,
                1.,
            ],
            emission: [0.; 4],
        }
    }
}
fn rgb_mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| mix(a[i], b[i], t))
}
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn smooth(a: f32, b: f32, v: f32) -> f32 {
    let t = ((v - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn filtered_noise(uv: [f32; 2], n: i32, seed: u32, fp: f32) -> f32 {
    let coverage = 1. - smooth(0.25, 0.75, fp * n as f32);
    if coverage > 0. {
        mix(0.5, noise(uv, [n, n], seed), coverage)
    } else {
        0.5
    }
}
fn crystal_grain(uv: [f32; 2], n: i32, seed: u32, fp: f32) -> f32 {
    let coverage = 1. - smooth(1., 3., fp * n as f32);
    if coverage <= 0. {
        return 0.5;
    }
    let p = uv.map(|v| v * n as f32);
    if fp == 0. {
        return hash(p[0].floor() as i32, p[1].floor() as i32, [n, n], seed);
    }
    let offset = fp * n as f32 * 0.25;
    let mut sum = 0.;
    for y in [-offset, offset] {
        for x in [-offset, offset] {
            sum += hash(
                (p[0] + x).floor() as i32,
                (p[1] + y).floor() as i32,
                [n, n],
                seed,
            );
        }
    }
    mix(0.5, sum * 0.25, coverage)
}
fn fbm(uv: [f32; 2], n: i32, seed: u32, octaves: u8, fp: f32) -> f32 {
    let mut value = 0.;
    let mut total = 0.;
    let mut weight = 0.5;
    for i in 0..octaves {
        let q = if i % 2 == 0 {
            uv
        } else {
            [uv[0] + uv[1], uv[1] - uv[0]]
        };
        value += weight
            * filtered_noise(
                q,
                n * (1 << i),
                seed.wrapping_add(u32::from(i) * 1013),
                fp * if i % 2 == 0 {
                    1.
                } else {
                    std::f32::consts::SQRT_2
                },
            );
        total += weight;
        weight *= 0.5;
    }
    value / total
}
fn cell(uv: [f32; 2], n: i32, seed: u32) -> [f32; 2] {
    let p = uv.map(|v| v * n as f32);
    let base = p.map(|v| v.floor() as i32);
    let mut first = 100.;
    let mut second = 100.;
    let mut a = [0.; 2];
    let mut b = [0.; 2];
    let mut domain = 0.;
    for y in -1..=1 {
        for x in -1..=1 {
            let c = [base[0] + x, base[1] + y];
            let q = [
                c[0] as f32 + hash(c[0], c[1], [n, n], seed),
                c[1] as f32 + hash(c[0], c[1], [n, n], seed.wrapping_add(11)),
            ];
            let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2);
            if d < first {
                second = first;
                b = a;
                first = d;
                a = q;
                domain = hash(c[0], c[1], [n, n], seed.wrapping_add(912));
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
        domain,
    ]
}
