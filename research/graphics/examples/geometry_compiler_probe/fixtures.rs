use super::compiler::{Mesh, Vertex};
use glam::DVec3;
pub fn stone(size: usize) -> Mesh {
    let mut m = Mesh {
        vertices: vec![],
        faces: vec![],
    };
    for face in 0..6 {
        let base = m.vertices.len() as u32;
        for y in 0..=size {
            for x in 0..=size {
                let u = 2. * x as f64 / size as f64 - 1.;
                let v = 2. * y as f64 / size as f64 - 1.;
                let p = match face {
                    0 => DVec3::new(1., u, v),
                    1 => DVec3::new(-1., -u, v),
                    2 => DVec3::new(-u, 1., v),
                    3 => DVec3::new(u, -1., v),
                    4 => DVec3::new(u, v, 1.),
                    _ => DVec3::new(-u, v, -1.),
                }
                .normalize();
                let radius = 1.
                    + 0.085 * (p.x * 17. + p.y * 7.).sin() * (p.z * 13. - p.x * 5.).cos()
                    + 0.035 * (p.x * 41. + p.z * 29.).sin() * (p.y * 37.).cos();
                m.vertices.push(Vertex {
                    p: (p * radius).as_vec3().to_array(),
                    n: [0.; 3],
                });
            }
        }
        for y in 0..size {
            for x in 0..size {
                let a = base + (y * (size + 1) + x) as u32;
                let b = a + 1;
                let d = a + (size + 1) as u32;
                let c = d + 1;
                let mat = ((x / (size / 4).max(1) + y / (size / 4).max(1) + face) % 3) as u32;
                m.faces.extend([[a, b, c, mat], [a, c, d, mat]]);
            }
        }
    }
    // Smooth the authored normal field across cube-face seams using exact positions.
    let mut normals = std::collections::HashMap::<[u32; 3], DVec3>::new();
    for t in &m.faces {
        let [a, b, c] =
            [t[0], t[1], t[2]].map(|i| DVec3::from_array(m.vertices[i as usize].p.map(f64::from)));
        let n = (b - a).cross(c - a);
        for &i in &t[..3] {
            let k = m.vertices[i as usize]
                .p
                .map(|v| if v == 0. { 0 } else { v.to_bits() });
            *normals.entry(k).or_default() += n;
        }
    }
    for v in &mut m.vertices {
        v.n = normals[&v.p.map(|v| if v == 0. { 0 } else { v.to_bits() })]
            .normalize()
            .as_vec3()
            .to_array();
    }
    m
}
pub fn folded(size: usize) -> Mesh {
    let mut m = Mesh {
        vertices: vec![],
        faces: vec![],
    };
    for y in 0..=size {
        for x in 0..=size {
            let u = 2. * x as f64 / size as f64 - 1.;
            let v = 2. * y as f64 / size as f64 - 1.;
            let z = 0.35 * u.abs() + 0.08 * (u * 17.).sin() * (v * 19.).cos();
            m.vertices.push(Vertex {
                p: [u as f32, v as f32, z as f32],
                n: [0., 0., 1.],
            });
        }
    }
    for y in 0..size {
        for x in 0..size {
            let a = (y * (size + 1) + x) as u32;
            let b = a + 1;
            let d = a + (size + 1) as u32;
            let c = d + 1;
            let mat = (x * 3 / size).min(2) as u32;
            m.faces.extend([[a, b, c, mat], [a, c, d, mat]]);
        }
    }
    m
}
fn interpolate(m: &Mesh, t: [u32; 4], weights: [f64; 3]) -> Vertex {
    let mut terms = [(t[0], weights[0]), (t[1], weights[1]), (t[2], weights[2])];
    terms.sort_unstable_by_key(|t| t.0);
    let mut p = DVec3::ZERO;
    let mut n = DVec3::ZERO;
    for (i, w) in terms {
        p += DVec3::from_array(m.vertices[i as usize].p.map(f64::from)) * w;
        n += DVec3::from_array(m.vertices[i as usize].n.map(f64::from)) * w;
    }
    Vertex {
        p: p.as_vec3().to_array(),
        n: n.as_vec3().to_array(),
    }
}
pub fn subdivide(m: &Mesh, factor: usize) -> Mesh {
    if factor == 1 {
        return m.clone();
    }
    let mut out = Mesh {
        vertices: vec![],
        faces: vec![],
    };
    for (ti, &t) in m.faces.iter().enumerate() {
        let base = out.vertices.len() as u32;
        if factor == 2 {
            for &i in &t[..3] {
                out.vertices.push(m.vertices[i as usize]);
            }
            let (a, b, c) = if ti % 2 == 0 { (0, 2, 1) } else { (0, 1, 2) };
            let mut weights = [0.; 3];
            weights[a] = 0.5;
            weights[b] = 0.5;
            out.vertices.push(interpolate(m, t, weights));
            let mut faces = if ti % 2 == 0 {
                [[0, 1, 3, t[3]], [3, 1, 2, t[3]]]
            } else {
                [[0, 3, 2, t[3]], [3, 1, 2, t[3]]]
            };
            let _ = c;
            for f in &mut faces {
                for i in &mut f[..3] {
                    *i += base;
                }
            }
            out.faces.extend(faces);
            continue;
        }
        let mut n = (factor as f64).sqrt() as usize;
        if !(factor - n * n).is_multiple_of(2) {
            n -= 1;
        }
        let mut ids = vec![vec![]; n + 1];
        for (i, row) in ids.iter_mut().enumerate() {
            for j in 0..=n - i {
                row.push(out.vertices.len() as u32);
                out.vertices.push(interpolate(
                    m,
                    t,
                    [
                        (n - i - j) as f64 / n as f64,
                        i as f64 / n as f64,
                        j as f64 / n as f64,
                    ],
                ));
            }
        }
        let mut faces = vec![];
        for i in 0..n {
            for j in 0..n - i {
                faces.push([ids[i][j], ids[i + 1][j], ids[i][j + 1], t[3]]);
                if j + 1 < n - i {
                    faces.push([ids[i + 1][j], ids[i + 1][j + 1], ids[i][j + 1], t[3]]);
                }
            }
        }
        while faces.len() < factor {
            let f = faces.remove(0);
            let v = out.vertices.len() as u32;
            let mut p = DVec3::ZERO;
            let mut normal = DVec3::ZERO;
            for &i in &f[..3] {
                p += DVec3::from_array(out.vertices[i as usize].p.map(f64::from)) / 3.;
                normal += DVec3::from_array(out.vertices[i as usize].n.map(f64::from)) / 3.;
            }
            out.vertices.push(Vertex {
                p: p.as_vec3().to_array(),
                n: normal.as_vec3().to_array(),
            });
            faces.extend([
                [f[0], f[1], v, t[3]],
                [f[1], f[2], v, t[3]],
                [f[2], f[0], v, t[3]],
            ]);
        }
        out.faces.extend(faces);
    }
    out
}
fn random(seed: &mut u64) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed as usize
}
pub fn shuffle(m: &mut Mesh, mut seed: u64) {
    let mut ids = (0..m.vertices.len()).collect::<Vec<_>>();
    for i in (1..ids.len()).rev() {
        let j = random(&mut seed) % (i + 1);
        ids.swap(i, j);
    }
    let mut map = vec![0; m.vertices.len()];
    let vertices = ids
        .iter()
        .enumerate()
        .map(|(new, &old)| {
            map[old] = new as u32;
            m.vertices[old]
        })
        .collect();
    m.vertices = vertices;
    for t in &mut m.faces {
        for i in &mut t[..3] {
            *i = map[*i as usize];
        }
        t[..3].rotate_left(random(&mut seed) % 3);
    }
    for i in (1..m.faces.len()).rev() {
        let j = random(&mut seed) % (i + 1);
        m.faces.swap(i, j);
    }
}
