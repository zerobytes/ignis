//! Bounded procedural distant probe and deterministic CPU pre-integration.
use crate::model::{Result, invalid};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const SPECULAR_SIZE: u32 = 64;
pub const MIP_COUNT: u32 = 7;
pub const DIFFUSE_SIZE: u32 = 8;
pub const LUT_SIZE: u32 = 64;
const PROBE_SAMPLES: u32 = 128;
const LUT_SAMPLES: u32 = 512;
pub const TEXTURE_BYTES: usize = 148_984;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    /// Linear RGB radiance, not an sRGB image or a background color.
    pub sky: [f32; 3],
    pub horizon: [f32; 3],
    pub ground: [f32; 3],
    /// Direction of the sky pole. Finite components within -1..1, norm >=0.01.
    pub axis: [f32; 3],
    /// Independent radiance multiplier; excluded from the prefilter cache key.
    pub intensity: f32,
}
impl Environment {
    pub fn validate(&self) -> Result<()> {
        if self
            .sky
            .iter()
            .chain(&self.horizon)
            .chain(&self.ground)
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            || self
                .axis
                .iter()
                .any(|v| !v.is_finite() || !(-1. ..=1.).contains(v))
            || Vec3::from(self.axis).length() < 0.01
            || !self.intensity.is_finite()
            || !(0. ..=4.).contains(&self.intensity)
        {
            return Err(invalid(
                "Environment requires linear RGB0..1, axis±1/norm>=0.01 and intensity0..4",
            ));
        }
        Ok(())
    }
    pub(crate) fn key(&self) -> Key {
        Key {
            sky: self.sky.into(),
            horizon: self.horizon.into(),
            ground: self.ground.into(),
            axis: Vec3::from(self.axis).normalize(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Key {
    sky: Vec3,
    horizon: Vec3,
    ground: Vec3,
    axis: Vec3,
}
impl Key {
    fn radiance(self, direction: Vec3) -> Vec3 {
        let h = direction.dot(self.axis).clamp(-1., 1.);
        self.horizon
            .lerp(if h >= 0. { self.sky } else { self.ground }, h.abs())
    }
}
fn sample(i: u32, count: u32) -> (f32, f32) {
    (
        i as f32 / count as f32,
        i.reverse_bits() as f32 * 2.328_306_4e-10,
    )
}
fn ggx_samples(roughness: f32, count: u32) -> Vec<Vec3> {
    let a2 = roughness.powi(4);
    (0..count)
        .map(|i| {
            let (x, y) = sample(i, count);
            let phi = std::f32::consts::TAU * x;
            let c = ((1. - y) / (1. + (a2 - 1.) * y)).sqrt();
            let s = (1. - c * c).max(0.).sqrt();
            Vec3::new(s * phi.cos(), s * phi.sin(), c)
        })
        .collect()
}
fn cosine_samples() -> Vec<Vec3> {
    (0..PROBE_SAMPLES)
        .map(|i| {
            let (x, y) = sample(i, PROBE_SAMPLES);
            let phi = std::f32::consts::TAU * x;
            let r = y.sqrt();
            Vec3::new(r * phi.cos(), r * phi.sin(), (1. - y).sqrt())
        })
        .collect()
}
fn basis(n: Vec3) -> (Vec3, Vec3) {
    let up = if n.z.abs() < 0.999 { Vec3::Z } else { Vec3::X };
    let tangent = up.cross(n).normalize();
    (tangent, n.cross(tangent))
}
fn world(n: Vec3, t: Vec3, b: Vec3, p: Vec3) -> Vec3 {
    t * p.x + b * p.y + n * p.z
}
fn face_direction(face: u32, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1., -v, -u),
        1 => Vec3::new(-1., -v, u),
        2 => Vec3::new(u, 1., v),
        3 => Vec3::new(u, -1., -v),
        4 => Vec3::new(u, -v, 1.),
        _ => Vec3::new(-u, -v, -1.),
    }
    .normalize()
}
fn cube(size: u32, mut evaluate: impl FnMut(Vec3) -> Vec3) -> Vec<u8> {
    let mut out = Vec::with_capacity((size * size * 6 * 4) as usize);
    for face in 0..6 {
        for y in 0..size {
            for x in 0..size {
                let u = 2. * (x as f32 + 0.5) / size as f32 - 1.;
                let v = 2. * (y as f32 + 0.5) / size as f32 - 1.;
                let c = evaluate(face_direction(face, u, v));
                out.extend([unit_byte(c.x), unit_byte(c.y), unit_byte(c.z), 255]);
            }
        }
    }
    out
}
fn unit_byte(x: f32) -> u8 {
    (x.clamp(0., 1.) * 255.).round() as u8
}
pub(crate) struct Baked {
    pub specular: Vec<Vec<u8>>,
    pub diffuse: Vec<u8>,
}
pub(crate) fn bake(key: Key) -> Baked {
    let mut specular = Vec::with_capacity(MIP_COUNT as usize);
    for level in 0..MIP_COUNT {
        let size = SPECULAR_SIZE >> level;
        if level == 0 {
            specular.push(cube(size, |n| key.radiance(n)));
            continue;
        }
        let roughness = level as f32 / (MIP_COUNT - 1) as f32;
        // Direction/weight in the v=n local frame is invariant across texels.
        let a2 = roughness.powi(4);
        let directions: Vec<_> = ggx_samples(roughness, PROBE_SAMPLES)
            .into_iter()
            .filter_map(|h| {
                let l = 2. * h.z * h - Vec3::Z;
                if l.z <= 0. {
                    return None;
                }
                let v = 0.5 / (l.z + (l.z * l.z * (1. - a2) + a2).sqrt());
                Some((l, v * l.z))
            })
            .collect();
        let weight: f32 = directions.iter().map(|(_, w)| w).sum();
        specular.push(cube(size, |n| {
            let (t, b) = basis(n);
            directions
                .iter()
                .map(|(l, w)| key.radiance(world(n, t, b, *l)) * *w)
                .sum::<Vec3>()
                / weight
        }));
    }
    let samples = cosine_samples();
    let diffuse = cube(DIFFUSE_SIZE, |n| {
        let (t, b) = basis(n);
        samples
            .iter()
            .map(|l| key.radiance(world(n, t, b, *l)))
            .sum::<Vec3>()
            / PROBE_SAMPLES as f32
    });
    Baked { specular, diffuse }
}
fn integrate_dfg(no_v: f32, roughness: f32, samples: &[Vec3]) -> [f32; 2] {
    let view = Vec3::new((1. - no_v * no_v).sqrt(), 0., no_v);
    let a2 = roughness.powi(4);
    let mut ab = [0.; 2];
    for h in samples {
        let vo_h = view.dot(*h).max(0.);
        let l = 2. * vo_h * *h - view;
        let no_l = l.z.max(0.);
        if no_l <= 0. {
            continue;
        }
        let visibility = 0.5
            / (no_l * (no_v * no_v * (1. - a2) + a2).sqrt()
                + no_v * (no_l * no_l * (1. - a2) + a2).sqrt())
            .max(0.000001);
        let g_vis = 4. * no_l * visibility * vo_h / h.z.max(0.000001);
        let fc = (1. - vo_h).powi(5);
        ab[0] += (1. - fc) * g_vis;
        ab[1] += fc * g_vis;
    }
    ab.map(|v| v / samples.len() as f32)
}
fn pack(ab: [f32; 2]) -> [u8; 4] {
    let [a, b] = ab.map(|v| (v.clamp(0., 1.) * 65535.).round() as u16);
    [(a >> 8) as u8, a as u8, (b >> 8) as u8, b as u8]
}
pub(crate) fn bake_lut() -> Vec<u8> {
    let mut out = Vec::with_capacity((LUT_SIZE * LUT_SIZE * 4) as usize);
    for y in 0..LUT_SIZE {
        let r = 0.2 + 0.8 * y as f32 / (LUT_SIZE - 1) as f32;
        let h = ggx_samples(r, LUT_SAMPLES);
        for x in 0..LUT_SIZE {
            let no_v = 0.001 + 0.999 * x as f32 / (LUT_SIZE - 1) as f32;
            out.extend(pack(integrate_dfg(no_v, r, &h)));
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    fn env() -> Environment {
        Environment {
            sky: [0.8, 0.9, 1.],
            horizon: [0.5, 0.5, 0.5],
            ground: [0.1, 0.2, 0.3],
            axis: [0., 1., 0.],
            intensity: 1.,
        }
    }
    #[test]
    fn bounded_parameters_and_intensity_independent_normalized_cache_key() {
        let e = env();
        e.validate().unwrap();
        let mut other = e;
        other.intensity = 4.;
        other.axis = [0., 0.5, 0.];
        assert_eq!(e.key(), other.key());
        other.sky[0] = 0.7;
        assert_ne!(e.key(), other.key());
        for i in 0..6 {
            let mut bad = e;
            match i {
                0 => bad.sky[0] = f32::NAN,
                1 => bad.ground[1] = 1.01,
                2 => bad.horizon[0] = -0.01,
                3 => bad.axis = [0.; 3],
                4 => bad.axis[1] = 1.01,
                _ => bad.intensity = 4.01,
            };
            assert!(bad.validate().is_err());
        }
        assert_eq!(e.key().radiance(Vec3::Y), Vec3::from(e.sky));
        assert_eq!(e.key().radiance(-Vec3::Y), Vec3::from(e.ground));
        assert_eq!(e.key().radiance(Vec3::X), Vec3::from(e.horizon));
        for (i, axis) in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z]
            .into_iter()
            .enumerate()
        {
            assert_eq!(face_direction(i as u32, 0., 0.), axis);
        }
    }
    #[test]
    fn constant_environment_preserves_radiance_and_all_mip_sizes_deterministically() {
        let mut e = env();
        e.sky = [0.2, 0.4, 0.6];
        e.horizon = e.sky;
        e.ground = e.sky;
        let a = bake(e.key());
        let b = bake(e.key());
        assert_eq!(a.specular, b.specular);
        assert_eq!(a.diffuse, b.diffuse);
        let mut size = 0;
        for (level, bytes) in a.specular.iter().enumerate() {
            assert_eq!(
                bytes.len(),
                (SPECULAR_SIZE as usize >> level).pow(2) * 6 * 4
            );
            size += bytes.len();
            for c in bytes.chunks_exact(4) {
                assert_eq!(c, [51, 102, 153, 255]);
            }
        }
        for c in a.diffuse.chunks_exact(4) {
            assert_eq!(c, [51, 102, 153, 255]);
        }
        assert_eq!(
            size + a.diffuse.len() + (LUT_SIZE * LUT_SIZE * 4) as usize,
            TEXTURE_BYTES
        );
    }
    #[test]
    fn integrated_white_metal_matches_the_analytic_rough_normal_limit() {
        let ab = integrate_dfg(1., 1., &ggx_samples(1., LUT_SAMPLES));
        assert!((ab[0] + ab[1] - (1. - 2_f32.ln())).abs() < 0.002);
        let smooth = integrate_dfg(1., 0.2, &ggx_samples(0.2, LUT_SAMPLES));
        assert!((smooth[0] + smooth[1] - 1.).abs() < 0.003);
        let lut = bake_lut();
        assert_eq!(lut.len(), (LUT_SIZE * LUT_SIZE * 4) as usize);
        for (index, c) in lut.chunks_exact(4).enumerate() {
            let a = u16::from_be_bytes([c[0], c[1]]) as f32 / 65535.;
            let b = u16::from_be_bytes([c[2], c[3]]) as f32 / 65535.;
            assert!(a + b <= 1.03, "{index}: {a}+{b}");
        }
        for ab in [[0., 0.], [1., 1.], [0.12345, 0.67891]] {
            let p = pack(ab);
            for i in 0..2 {
                let v = u16::from_be_bytes([p[i * 2], p[i * 2 + 1]]) as f32 / 65535.;
                assert!((v - ab[i]).abs() <= 0.5 / 65535. + f32::EPSILON);
            }
        }
    }
}
