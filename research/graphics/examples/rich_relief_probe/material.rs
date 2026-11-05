use super::compiler;
use anyhow::{Result, ensure};
use glam::Vec3;
pub fn noise(x: f64, y: f64) -> f64 {
    fn hash(x: i64, y: i64) -> f64 {
        let mut v = (x as u64).wrapping_mul(0x517cc1b727220a95)
            ^ (y as u64).wrapping_mul(0x9e3779b97f4a7c15);
        v ^= v >> 30;
        v = v.wrapping_mul(0xbf58476d1ce4e5b9);
        v ^= v >> 27;
        v = v.wrapping_mul(0x94d049bb133111eb);
        v ^= v >> 31;
        (v >> 11) as f64 / (1u64 << 53) as f64
    }
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let mut a = x - x.floor();
    let mut b = y - y.floor();
    a = a * a * (3. - 2. * a);
    b = b * b * (3. - 2. * b);
    let lo = hash(ix, iy) * (1. - a) + hash(ix + 1, iy) * a;
    let hi = hash(ix, iy + 1) * (1. - a) + hash(ix + 1, iy + 1) * a;
    lo * (1. - b) + hi * b
}
pub fn height(x: f64, y: f64) -> f64 {
    let radial = (1. - (x * x + y * y) * 0.48).max(0.);
    let broad =
        0.11 * (noise(x * 3. + 7., y * 3. + 3.) - 0.5) + 0.06 * (noise(x * 8., y * 8.) - 0.5);
    let ridges = (x * 13. + y * 7. + 3. * noise(x * 3., y * 3.)).sin();
    let crack = ((x * 17. + 2. * (y * 5.).sin()).sin().abs() / 0.13).min(1.);
    let rough = (noise(x * 47. + 2., y * 47.) - 0.5) * 0.005
        + (noise(x * 113., y * 113. + 8.) - 0.5) * 0.0015;
    let detail_mask = (noise(x * 2. + 11., y * 2. + 9.) * 1.6 - 0.3).clamp(0., 1.);
    -0.18
        + radial
            * (0.34 + broad + 0.025 * ridges - 0.027 * (1. - crack).powi(2) + rough * detail_mask)
}
fn color(x: f64, y: f64) -> [f32; 4] {
    let macro_noise = noise(x * 3. + 5., y * 3. - 7.);
    let warpx = x + 0.12 * noise(x * 5., y * 5.);
    let warpy = y + 0.1 * noise(x * 4. + 3., y * 4.);
    let band = (warpx * 19. + warpy * 11. + 4. * noise(x * 3., y * 3.)).sin() * 0.5 + 0.5;
    let copper = Vec3::new(0.48, 0.13, 0.035);
    let ochre = Vec3::new(0.55, 0.32, 0.095);
    let turquoise = Vec3::new(0.018, 0.34, 0.28);
    let slate = Vec3::new(0.12, 0.07, 0.22);
    let mut c = if macro_noise > 0.54 {
        copper.lerp(ochre, band as f32)
    } else {
        slate.lerp(turquoise, band as f32)
    };
    let vein = ((warpx * 31. + warpy * 9. + noise(x * 8., y * 8.) * 4.)
        .sin()
        .abs()
        / 0.09)
        .min(1.);
    c = c.lerp(
        Vec3::new(0.79, 0.72, 0.54),
        (1. - vein).powi(2) as f32 * 0.9,
    );
    let moss = ((noise(x * 7. + 40., y * 7.) - 0.6) * 6.).clamp(0., 1.);
    c = c.lerp(Vec3::new(0.16, 0.28, 0.025), moss as f32);
    let fleck = noise(x * 203. + 2., y * 203.) as f32;
    c *= 0.76 + 0.35 * fleck;
    let grain = noise(x * 513., y * 513. + 22.) as f32;
    c += Vec3::splat((grain - 0.5) * 0.025);
    [
        c.x,
        c.y,
        c.z,
        (0.35 + 0.52 * moss as f32 + 0.14 * fleck).clamp(0.2, 0.95),
    ]
}
pub struct Map {
    pub size: u32,
    pub levels: Vec<Vec<u8>>,
}
fn mipmaps(size: u32, pixels: Vec<u8>) -> Map {
    let mut levels = vec![pixels];
    let mut n = size as usize;
    while n > 1 {
        let prev = levels.last().unwrap();
        let mut next = vec![0u8; n * n];
        for y in 0..n / 2 {
            for x in 0..n / 2 {
                for c in 0..4 {
                    let sum: u32 = (0..2)
                        .flat_map(|dy| {
                            (0..2)
                                .map(move |dx| prev[((y * 2 + dy) * n + x * 2 + dx) * 4 + c] as u32)
                        })
                        .sum();
                    next[(y * (n / 2) + x) * 4 + c] = ((sum + 2) / 4) as u8;
                }
            }
        }
        levels.push(next);
        n /= 2;
    }
    Map { size, levels }
}
pub fn build_color(size: u32) -> Map {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let a = (x as f64 + 0.5) / size as f64 * 2. - 1.;
            let b = (y as f64 + 0.5) / size as f64 * 2. - 1.;
            pixels.extend(color(a, b).map(|c| (c.clamp(0., 1.) * 255.).round() as u8));
        }
    }
    mipmaps(size, pixels)
}
pub fn build_normal(size: u32, mesh: &compiler::Mesh, grid: usize) -> Map {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let u = (x as f32 + 0.5) / size as f32 * grid as f32;
            let v = (y as f32 + 0.5) / size as f32 * grid as f32;
            let ix = (u.floor() as usize).min(grid - 1);
            let iy = (v.floor() as usize).min(grid - 1);
            let fx = u - ix as f32;
            let fy = v - iy as f32;
            let a = Vec3::from_array(mesh.vertices[iy * (grid + 1) + ix].n);
            let b = Vec3::from_array(mesh.vertices[iy * (grid + 1) + ix + 1].n);
            let c = Vec3::from_array(mesh.vertices[(iy + 1) * (grid + 1) + ix + 1].n);
            let d = Vec3::from_array(mesh.vertices[(iy + 1) * (grid + 1) + ix].n);
            let n = if fx >= fy {
                a * (1. - fx) + b * (fx - fy) + c * fy
            } else {
                a * (1. - fy) + c * fx + d * (fy - fx)
            };
            let n = n.normalize();
            let ambient = (0.75 + 0.25 * n.z).clamp(0., 1.);
            pixels.extend(
                [n.x * 0.5 + 0.5, n.y * 0.5 + 0.5, n.z * 0.5 + 0.5, ambient]
                    .map(|c| (c.clamp(0., 1.) * 255.).round() as u8),
            );
        }
    }
    mipmaps(size, pixels)
}
pub fn encode(m: &Map) -> Vec<u8> {
    let payload = m.levels.concat();
    let mut b = b"IRM1".to_vec();
    b.extend(1u32.to_le_bytes());
    b.extend(m.size.to_le_bytes());
    b.extend((m.levels.len() as u32).to_le_bytes());
    b.extend(compiler::hash(&payload).to_le_bytes());
    b.extend(payload);
    b
}
pub fn decode(b: &[u8]) -> Result<Map> {
    ensure!(b.len() >= 24 && &b[..4] == b"IRM1", "map header");
    let u = |i| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    let size = u(8);
    ensure!(
        u(4) == 1
            && size.is_power_of_two()
            && (2..=4096).contains(&size)
            && u(12) == size.ilog2() + 1,
        "map version/dimensions"
    );
    ensure!(
        u64::from_le_bytes(b[16..24].try_into().unwrap()) == compiler::hash(&b[24..]),
        "map checksum"
    );
    let mut n = size;
    let mut off = 24;
    let mut levels = vec![];
    loop {
        let len = (n * n * 4) as usize;
        ensure!(off + len <= b.len(), "map truncated");
        levels.push(b[off..off + len].to_vec());
        off += len;
        if n == 1 {
            break;
        }
        n /= 2;
    }
    ensure!(off == b.len(), "map trailing");
    Ok(Map { size, levels })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_roundtrip_linear_average_and_corruption() -> Result<()> {
        let m = mipmaps(
            2,
            vec![
                0, 0, 0, 0, 100, 100, 100, 100, 200, 200, 200, 200, 100, 100, 100, 100,
            ],
        );
        assert_eq!(m.levels[1], vec![100; 4]);
        let bytes = encode(&m);
        assert_eq!(encode(&decode(&bytes)?), bytes);
        let mut b = bytes.clone();
        *b.last_mut().unwrap() ^= 1;
        assert!(decode(&b).is_err());
        assert!(decode(&bytes[..bytes.len() - 1]).is_err());
        Ok(())
    }
}
