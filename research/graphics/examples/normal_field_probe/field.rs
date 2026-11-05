use super::{compile, compiler};
use anyhow::{Result, ensure};
use glam::{DVec2, DVec3};
use std::collections::HashMap;
pub struct Field {
    pub asset: compile::Asset,
    pub uv: Vec<[f32; 2]>,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
fn cross(a: DVec2, b: DVec2) -> f64 {
    a.x * b.y - a.y * b.x
}
// Exact inside interpolation; outside, closest point on one of the three edges.
fn sample(q: [DVec2; 3], n: [DVec3; 3], p: DVec2) -> (f64, DVec3) {
    let area = cross(q[1] - q[0], q[2] - q[0]);
    let b = cross(p - q[0], q[2] - q[0]) / area;
    let c = cross(q[1] - q[0], p - q[0]) / area;
    if b >= 0. && c >= 0. && b + c <= 1. {
        return (0., n[0] * (1. - b - c) + n[1] * b + n[2] * c);
    }
    let mut best = (f64::INFINITY, DVec3::ZERO);
    for i in 0..3 {
        let j = (i + 1) % 3;
        let e = q[j] - q[i];
        let t = ((p - q[i]).dot(e) / e.length_squared()).clamp(0., 1.);
        let d = (p - q[i] - e * t).length_squared();
        if d < best.0 {
            best = (d, n[i].lerp(n[j], t));
        }
    }
    best
}
fn packed(n: DVec3) -> [u8; 4] {
    let v = n
        .to_array()
        .map(|x| (x.clamp(-1., 1.) * 127.).round() as i8 as u8);
    [v[0], v[1], v[2], 127]
}
pub fn build(mut asset: compile::Asset, resolution: u32) -> Result<Field> {
    ensure!([32, 64, 128].contains(&resolution), "field resolution");
    let tiles = asset.patches.iter().filter(|p| p.levels.len() > 1).count() as u32;
    ensure!(tiles > 0, "no reducible patches");
    let stride = resolution + 2; // one texel gutter on all sides
    let columns = (tiles as f64).sqrt().ceil() as u32;
    let width = columns * stride;
    let height = tiles.div_ceil(columns) * stride;
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let old = asset.mesh.vertices.clone();
    let mut vertices = Vec::new();
    let mut uv = Vec::new();
    let mut native = HashMap::new();
    // Original range remains unchanged geometrically, with no atlas lookup.
    for t in &mut asset.mesh.faces[..asset.source_count as usize / 3] {
        for index in &mut t[..3] {
            let i = *native.entry(*index).or_insert_with(|| {
                let i = vertices.len() as u32;
                vertices.push(old[*index as usize]);
                uv.push([-1., -1.]);
                i
            });
            *index = i;
        }
    }
    let mut tile = 0;
    for patch in &asset.patches {
        if patch.levels.len() == 1 {
            continue;
        }
        let axis = DVec3::new(
            patch.cone[0] as f64,
            patch.cone[1] as f64,
            patch.cone[2] as f64,
        )
        .normalize();
        let u = axis.any_orthonormal_vector();
        let v = axis.cross(u);
        let xy = |p: [f32; 3]| {
            let p = DVec3::from_array(p.map(f64::from));
            DVec2::new(p.dot(u), p.dot(v))
        };
        let l = &patch.levels[0];
        let triangles = asset.mesh.faces[l.start as usize / 3..(l.start + l.count) as usize / 3]
            .iter()
            .map(|t| {
                let vs = [t[0], t[1], t[2]].map(|i| vertices[i as usize]);
                (
                    vs.map(|a| xy(a.p)),
                    vs.map(|a| DVec3::from_array(a.n.map(f64::from))),
                )
            })
            .collect::<Vec<_>>();
        let mut low = DVec2::splat(f64::INFINITY);
        let mut high = DVec2::splat(f64::NEG_INFINITY);
        for (q, _) in &triangles {
            for &p in q {
                low = low.min(p);
                high = high.max(p);
            }
        }
        let span = high - low;
        ensure!(span.min_element() > 0., "degenerate tile");
        let ox = (tile % columns) * stride;
        let oy = (tile / columns) * stride;
        for y in 0..stride {
            for x in 0..stride {
                let p = low
                    + span
                        * DVec2::new(
                            (x as f64 - 0.5) / resolution as f64,
                            (y as f64 - 0.5) / resolution as f64,
                        );
                let mut best = (f64::INFINITY, DVec3::ZERO);
                for &(q, n) in &triangles {
                    let s = sample(q, n, p);
                    if s.0 < best.0 {
                        best = s;
                    }
                    if best.0 == 0. {
                        break;
                    }
                }
                ensure!(best.1.is_finite(), "invalid field sample");
                let off = (((oy + y) * width + ox + x) * 4) as usize;
                pixels[off..off + 4].copy_from_slice(&packed(best.1));
            }
        }
        let mut remap = HashMap::new();
        for l in patch.levels.iter().skip(1) {
            for t in &mut asset.mesh.faces[l.start as usize / 3..(l.start + l.count) as usize / 3] {
                for index in &mut t[..3] {
                    let i = *remap.entry(*index).or_insert_with(|| {
                        let vert = old[*index as usize];
                        let st = (xy(vert.p) - low) / span;
                        let coord = [
                            ((ox + 1) as f64 + st.x * resolution as f64) / width as f64,
                            ((oy + 1) as f64 + st.y * resolution as f64) / height as f64,
                        ];
                        let i = vertices.len() as u32;
                        vertices.push(vert);
                        uv.push(coord.map(|x| x as f32));
                        i
                    });
                    *index = i;
                }
            }
        }
        tile += 1;
    }
    asset.mesh.vertices = vertices;
    compiler::validate(&asset.mesh)?;
    Ok(Field {
        asset,
        uv,
        pixels,
        width,
        height,
    })
}
pub fn encode(f: &Field) -> Vec<u8> {
    let a = compile::encode(&f.asset);
    let mut payload = Vec::new();
    payload.extend((a.len() as u64).to_le_bytes());
    payload.extend(f.width.to_le_bytes());
    payload.extend(f.height.to_le_bytes());
    payload.extend((f.uv.len() as u32).to_le_bytes());
    payload.extend(a);
    for uv in &f.uv {
        for x in uv {
            payload.extend(x.to_le_bytes());
        }
    }
    payload.extend(&f.pixels);
    let mut b = b"INF1".to_vec();
    b.extend(1u32.to_le_bytes());
    b.extend((payload.len() as u64).to_le_bytes());
    b.extend(compiler::hash(&payload).to_le_bytes());
    b.extend(payload);
    b
}
pub fn decode(b: &[u8]) -> Result<Field> {
    ensure!(b.len() >= 44 && &b[..4] == b"INF1", "field header");
    let u = |i| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    let q = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    ensure!(
        u(4) == 1 && q(8) == b.len() as u64 - 24 && q(16) == compiler::hash(&b[24..]),
        "field layout/checksum"
    );
    let size = q(24) as usize;
    let width = u(32);
    let height = u(36);
    let count = u(40) as usize;
    ensure!(
        width > 0 && height > 0 && width <= 4096 && height <= 4096 && size <= b.len() - 44,
        "field dimensions"
    );
    ensure!(
        44 + size + count * 8 + width as usize * height as usize * 4 == b.len(),
        "field size"
    );
    let asset = compile::decode(&b[44..44 + size])?;
    ensure!(count == asset.mesh.vertices.len(), "field vertex count");
    let mut uv = Vec::with_capacity(count);
    for raw in b[44 + size..44 + size + count * 8].chunks_exact(8) {
        let a =
            std::array::from_fn(|i| f32::from_le_bytes(raw[i * 4..i * 4 + 4].try_into().unwrap()));
        ensure!(
            a == [-1., -1.] || a.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
            "field UV"
        );
        uv.push(a);
    }
    let pixels = b[44 + size + count * 8..].to_vec();
    Ok(Field {
        asset,
        uv,
        pixels,
        width,
        height,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn affine_and_padding() {
        let q = [DVec2::ZERO, DVec2::X, DVec2::Y];
        let n = [DVec3::Z, DVec3::X, DVec3::Y];
        let s = sample(q, n, DVec2::new(0.25, 0.25));
        assert_eq!(s.0, 0.);
        assert!((s.1 - DVec3::new(0.25, 0.25, 0.5)).length() < 1e-12);
        assert_eq!(sample(q, n, DVec2::new(2., 0.)).1, DVec3::X);
        let p = packed(DVec3::new(-1., 0.5, 1.));
        assert_eq!(p, [129, 64, 127, 127]);
    }
    #[test]
    fn constant_field_uv_and_binary_integrity() -> Result<()> {
        let mesh = compiler::Mesh {
            vertices: [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]
                .map(|p| compiler::Vertex { p, n: [0., 0., 1.] })
                .to_vec(),
            faces: vec![[0, 1, 2, 0], [0, 2, 3, 0], [0, 1, 2, 0], [0, 2, 3, 0]],
        };
        let asset = compile::Asset {
            mesh,
            source_count: 6,
            patches: vec![compile::Patch {
                low: [0., 0., 0.],
                high: [1., 1., 0.],
                cone: [0., 0., 1., 0.],
                levels: vec![
                    compile::Level {
                        start: 0,
                        count: 6,
                        error: 0.,
                        normal: 0.,
                    },
                    compile::Level {
                        start: 6,
                        count: 6,
                        error: 0.,
                        normal: 0.,
                    },
                ],
            }],
        };
        let f = build(asset, 32)?;
        assert!(f.pixels.chunks_exact(4).all(|p| p == [0, 0, 127, 127]));
        assert_eq!(f.uv.iter().filter(|uv| **uv == [-1., -1.]).count(), 4);
        assert_eq!(f.uv.len(), 8);
        for t in &f.asset.mesh.faces[2..] {
            assert!(t[..3].iter().all(|&i| f.uv[i as usize][0] >= 0.));
        }
        let b = encode(&f);
        assert_eq!(encode(&decode(&b)?), b);
        for end in [0, 24, 43, b.len() - 1] {
            assert!(decode(&b[..end]).is_err());
        }
        let mut broken = b;
        let last = broken.len() - 1;
        broken[last] ^= 1;
        assert!(decode(&broken).is_err());
        Ok(())
    }
}
