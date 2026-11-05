use super::*;
const MAGIC: u32 = u32::from_le_bytes(*b"IRK1");
const HEADER: usize = 17;
#[derive(Clone)]
pub(super) struct Asset {
    pub size: u32,
    pub views: u32,
    pub lights: u32,
    pub codes: u32,
    pub extent: f32,
    pub center: Vec3,
    pub camera_elevation: f32,
    pub light_elevation: f32,
    pub directory: Vec<u32>,
    pub maps: Vec<u32>,
    pub responses: Vec<u32>,
}
impl Asset {
    pub fn bytes(&self) -> Vec<u8> {
        let mut h = vec![
            MAGIC,
            1,
            self.size,
            self.views,
            self.lights,
            self.codes,
            self.extent.to_bits(),
            self.center.x.to_bits(),
            self.center.y.to_bits(),
            self.center.z.to_bits(),
            self.camera_elevation.to_bits(),
            self.light_elevation.to_bits(),
            self.directory.len() as u32,
            self.maps.len() as u32,
            self.responses.len() as u32,
        ];
        let payload: Vec<u8> = self
            .directory
            .iter()
            .chain(&self.maps)
            .chain(&self.responses)
            .flat_map(|x| x.to_le_bytes())
            .collect();
        let mut checked: Vec<u8> = h.iter().flat_map(|x| x.to_le_bytes()).collect();
        checked.extend(&payload);
        let checksum = hash(&checked);
        h.extend([checksum as u32, (checksum >> 32) as u32]);
        let mut bytes: Vec<u8> = h.iter().flat_map(|x| x.to_le_bytes()).collect();
        bytes.extend(payload);
        bytes
    }
    pub fn load(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() >= HEADER * 4
                && bytes.len() <= 64 * 1024 * 1024
                && bytes.len().is_multiple_of(4),
            "binary length/budget"
        );
        let words: Vec<u32> = bytes
            .chunks_exact(4)
            .map(|x| u32::from_le_bytes(x.try_into().unwrap()))
            .collect();
        let h = &words[..HEADER];
        ensure!(h[0] == MAGIC && h[1] == 1, "magic/version");
        ensure!(
            [128, 256].contains(&h[2])
                && [16, 32].contains(&h[3])
                && [16, 32].contains(&h[4])
                && [256, 512].contains(&h[5]),
            "binary dimensions"
        );
        let f = |i: usize| f32::from_bits(h[i]);
        ensure!(
            (6..12).all(|i| f(i).is_finite())
                && (0.1..=4.).contains(&f(6))
                && f(10) == 35.
                && f(11) == 55.,
            "binary domain"
        );
        ensure!(
            h[12] == h[3] * (h[2] / 8).pow(2)
                && h[13].is_multiple_of(64)
                && h[14] == h[3] * h[4] * (h[5] - 1),
            "binary section sizes"
        );
        let total = HEADER as u64 + u64::from(h[12]) + u64::from(h[13]) + u64::from(h[14]);
        ensure!(total == words.len() as u64, "binary section length");
        let mut checked = bytes[..15 * 4].to_vec();
        checked.extend(&bytes[HEADER * 4..]);
        ensure!(
            hash(&checked) == u64::from(h[15]) | (u64::from(h[16]) << 32),
            "binary checksum"
        );
        let directory = words[HEADER..HEADER + h[12] as usize].to_vec();
        let start = HEADER + h[12] as usize;
        let maps = words[start..start + h[13] as usize].to_vec();
        let responses = words[start + h[13] as usize..].to_vec();
        // One directory entry per stored tile: no unreachable data or duplicate aliases.
        let mut used = vec![false; maps.len() / 64];
        for &p in &directory {
            if p != u32::MAX {
                ensure!(
                    p.is_multiple_of(64) && (p as usize) + 64 <= maps.len(),
                    "tile offset"
                );
                ensure!(!used[p as usize / 64], "duplicate tile");
                used[p as usize / 64] = true;
            }
        }
        ensure!(used.iter().all(|x| *x), "unreachable tile");
        ensure!(maps.iter().all(|w| (w & 65535) < h[5]), "response code");
        ensure!(responses.iter().all(|w| w >> 30 == 3), "response coverage");
        Ok(Self {
            size: h[2],
            views: h[3],
            lights: h[4],
            codes: h[5],
            extent: f(6),
            center: Vec3::new(f(7), f(8), f(9)),
            camera_elevation: f(10),
            light_elevation: f(11),
            directory,
            maps,
            responses,
        })
    }
    pub fn payload(&self) -> usize {
        4 * (self.directory.len() + self.maps.len() + self.responses.len())
    }
}
#[derive(Clone, Copy)]
struct Sample {
    normal: Vec3,
    color: Vec3,
    pixel: usize,
}
fn split(samples: &mut [Sample], count: usize, output: &mut Vec<(Vec3, Vec3)>, ids: &mut [u16]) {
    if count == 1 {
        let mut n = Vec3::ZERO;
        let mut c = Vec3::ZERO;
        for s in samples.iter() {
            n += s.normal;
            c += s.color;
        }
        let id = output.len() + 1;
        output.push((n.normalize_or_zero(), c / samples.len() as f32));
        for s in samples {
            ids[s.pixel] = id as u16;
        }
        return;
    }
    let value = |s: &Sample, k: usize| {
        if k < 3 {
            s.normal[k]
        } else {
            s.color[k - 3] * 4.
        }
    };
    let mut ranges = [0f32; 6];
    for (k, r) in ranges.iter_mut().enumerate() {
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for s in samples.iter() {
            let v = value(s, k);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        *r = hi - lo;
    }
    let axis = (0..6)
        .max_by(|&a, &b| ranges[a].total_cmp(&ranges[b]))
        .unwrap();
    samples.sort_unstable_by(|a, b| {
        value(a, axis)
            .total_cmp(&value(b, axis))
            .then(a.pixel.cmp(&b.pixel))
    });
    let left = count / 2;
    let at = samples.len() * left / count;
    let (a, b) = samples.split_at_mut(at);
    split(a, left, output, ids);
    split(b, count - left, output, ids);
}
fn material(n: Vec3, c: Vec3, v: Vec3, l: Vec3) -> Vec3 {
    let n = n.normalize();
    let h = (l + v).normalize_or_zero();
    let nv = n.dot(v).max(0.0001);
    let nl = n.dot(l).max(0.);
    let nh = n.dot(h).max(0.);
    let roughness = 0.88f32;
    let a2 = roughness.powi(4);
    let denom = nh * nh * (a2 - 1.) + 1.;
    let d = a2 / (std::f32::consts::PI * denom * denom);
    let k = (roughness + 1.).powi(2) / 8.;
    let visibility = (nv / (nv * (1. - k) + k)) * (nl / (nl * (1. - k) + k));
    let f = Vec3::splat(0.04) + Vec3::splat(0.96) * (1. - h.dot(v).max(0.)).powi(5);
    let spec = d * visibility * f / (4. * nv * nl).max(0.0001);
    let diffuse = (Vec3::ONE - f) * c / std::f32::consts::PI;
    c * 0.03 + (diffuse + spec) * nl * std::f32::consts::PI
}
fn pack_color(c: Vec3) -> Result<u32> {
    ensure!(
        c.is_finite() && c.min_element() >= 0. && c.max_element() <= 2.,
        "response outside coding range"
    );
    let q = (c * (1023. / 2.)).round().as_uvec3();
    Ok(q.x | (q.y << 10) | (q.z << 20) | (3 << 30))
}
pub(super) fn append_view(
    asset: &mut Asset,
    view: u32,
    normal: &Image,
    color: &Image,
) -> Result<f32> {
    let mut samples = Vec::new();
    let mut ids = vec![0u16; normal.len()];
    for (pixel, (n, c)) in normal.iter().zip(color).enumerate() {
        if c[3] > 0. {
            samples.push(Sample {
                normal: Vec3::from_slice(n),
                color: Vec3::from_slice(c),
                pixel,
            });
        }
    }
    ensure!(
        samples.len() >= asset.codes as usize,
        "insufficient occupied texels"
    );
    let mut dictionary = Vec::new();
    split(
        &mut samples,
        asset.codes as usize - 1,
        &mut dictionary,
        &mut ids,
    );
    let mut max_error = 0f32;
    for s in &samples {
        let (n, c) = dictionary[ids[s.pixel] as usize - 1];
        max_error = max_error.max((n - s.normal).length().max((c - s.color).length()));
    }
    let tiles = asset.size / 8;
    for y in 0..tiles {
        for x in 0..tiles {
            let mut block = [0u32; 64];
            for by in 0..8 {
                for bx in 0..8 {
                    let p = ((y * 8 + by) * asset.size + x * 8 + bx) as usize;
                    let id = ids[p];
                    if id != 0 {
                        let h = normal[p][3] - 2.;
                        ensure!(h.abs() <= asset.extent, "depth outside extent");
                        let q = ((h / asset.extent * 0.5 + 0.5) * 65535.).round() as u32;
                        block[(by * 8 + bx) as usize] = u32::from(id) | (q << 16);
                    }
                }
            }
            if block.iter().all(|x| *x == 0) {
                asset.directory.push(u32::MAX);
            } else {
                asset.directory.push(asset.maps.len() as u32);
                asset.maps.extend(block);
            }
        }
    }
    let v = direction(
        view as f32 * 360. / asset.views as f32,
        asset.camera_elevation,
    );
    for light in 0..asset.lights {
        let l = direction(
            light as f32 * 360. / asset.lights as f32,
            asset.light_elevation,
        );
        for &(n, c) in &dictionary {
            asset.responses.push(pack_color(material(n, c, v, l))?);
        }
    }
    Ok(max_error)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Asset {
        Asset {
            size: 128,
            views: 16,
            lights: 16,
            codes: 256,
            extent: 1.3,
            center: CENTER,
            camera_elevation: 35.,
            light_elevation: 55.,
            directory: vec![u32::MAX; 16 * 16 * 16],
            maps: vec![],
            responses: vec![3 << 30; 16 * 16 * 255],
        }
    }
    #[test]
    fn binary_roundtrip_and_corruption() {
        let a = fixture();
        let b = a.bytes();
        assert_eq!(Asset::load(&b).unwrap().bytes(), b);
        for n in [0, 1, 67, b.len() - 1] {
            assert!(Asset::load(&b[..n]).is_err());
        }
        for i in [0, 4, 24, 60, b.len() - 1] {
            let mut x = b.clone();
            x[i] ^= 1;
            assert!(Asset::load(&x).is_err());
        }
    }
    #[test]
    fn semantic_validation_after_checksum() {
        let mut a = fixture();
        a.directory[0] = 0;
        assert!(Asset::load(&a.bytes()).is_err());
        a.maps = vec![1; 64];
        assert!(Asset::load(&a.bytes()).is_ok());
        a.maps[0] = 256;
        assert!(Asset::load(&a.bytes()).is_err());
        a.maps[0] = 1;
        a.directory[1] = 0;
        assert!(Asset::load(&a.bytes()).is_err());
        a.directory[1] = u32::MAX;
        a.maps.extend([0; 64]);
        assert!(Asset::load(&a.bytes()).is_err());
    }
    #[test]
    fn quantization_and_material_limits() {
        for x in [0., 0.001, 0.5, 1., 2.] {
            let p = pack_color(Vec3::splat(x)).unwrap();
            assert!(((p & 1023) as f32 * 2. / 1023. - x).abs() <= 1. / 1023. + 1e-6);
            assert_eq!(p >> 30, 3);
        }
        assert!(pack_color(Vec3::splat(2.1)).is_err());
        for a in 0..360 {
            assert!(
                material(
                    Vec3::Y,
                    Vec3::splat(0.4),
                    direction(a as f32, 35.),
                    direction(a as f32 * 2., 55.)
                )
                .is_finite()
            );
        }
    }
}
