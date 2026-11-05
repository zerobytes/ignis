use anyhow::{Result, ensure};
use bytemuck::{Pod, Zeroable};
use glam::DVec3;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub p: [f32; 3],
    pub n: [f32; 3],
}
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<[u32; 4]>,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Certificate {
    pub low: [f32; 3],
    pub error: f32,
    pub high: [f32; 3],
    pub normal_error: f32,
    pub counts: [u32; 4],
}
pub struct Compiled {
    pub source: Mesh,
    pub proxy: Mesh,
    pub regions: Vec<Certificate>,
    pub source_hash: u64,
}
pub const POSITION_TOL: f64 = 0.00002;
pub const NORMAL_TOL: f64 = 0.00005;
pub const BORDER_TOL: f64 = 0.000001;
fn pos(v: Vertex) -> DVec3 {
    DVec3::from_array(v.p.map(f64::from))
}
fn normal(v: Vertex) -> DVec3 {
    DVec3::from_array(v.n.map(f64::from))
}
pub fn hash(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf29ce484222325, |h, v| {
        (h ^ *v as u64).wrapping_mul(0x100000001b3)
    })
}
pub fn validate(m: &Mesh) -> Result<()> {
    ensure!(!m.vertices.is_empty() && !m.faces.is_empty(), "empty mesh");
    ensure!(
        m.vertices
            .iter()
            .all(|v| v.p.iter().chain(v.n.iter()).all(|x| x.is_finite())
                && normal(*v).length() > 1e-8),
        "invalid vertex"
    );
    for t in &m.faces {
        ensure!(
            t[..3].iter().all(|i| (*i as usize) < m.vertices.len()) && t[3] < 3,
            "invalid index/material"
        );
        let [a, b, c]: [u32; 3] = t[..3].try_into().unwrap();
        let a = pos(m.vertices[a as usize]);
        let b = pos(m.vertices[b as usize]);
        let c = pos(m.vertices[c as usize]);
        ensure!(
            (b - a).cross(c - a).length_squared() > 1e-24,
            "degenerate face"
        );
    }
    Ok(())
}
fn mesh_bytes(m: &Mesh) -> Vec<u8> {
    let mut b = vec![];
    b.extend((m.vertices.len() as u32).to_le_bytes());
    b.extend((m.faces.len() as u32).to_le_bytes());
    for v in &m.vertices {
        for x in v.p.into_iter().chain(v.n) {
            b.extend(x.to_le_bytes());
        }
    }
    for f in &m.faces {
        for i in f {
            b.extend(i.to_le_bytes());
        }
    }
    b
}
fn read_mesh(b: &[u8], off: &mut usize) -> Result<Mesh> {
    let n = word(b, *off)? as usize;
    let t = word(b, *off + 4)? as usize;
    *off += 8;
    let end = off
        .checked_add(
            n.checked_mul(24)
                .and_then(|x| t.checked_mul(16).and_then(|y| x.checked_add(y)))
                .ok_or_else(|| anyhow::anyhow!("size overflow"))?,
        )
        .ok_or_else(|| anyhow::anyhow!("size overflow"))?;
    ensure!(end <= b.len(), "truncated mesh");
    let mut vertices = Vec::with_capacity(n);
    for _ in 0..n {
        let mut a = [0f32; 6];
        for v in &mut a {
            *v = f32::from_bits(word(b, *off)?);
            *off += 4;
        }
        vertices.push(Vertex {
            p: a[..3].try_into().unwrap(),
            n: a[3..].try_into().unwrap(),
        });
    }
    let mut faces = Vec::with_capacity(t);
    for _ in 0..t {
        let mut a = [0; 4];
        for v in &mut a {
            *v = word(b, *off)?;
            *off += 4;
        }
        faces.push(a);
    }
    let m = Mesh { vertices, faces };
    validate(&m)?;
    Ok(m)
}
fn word(b: &[u8], i: usize) -> Result<u32> {
    ensure!(i + 4 <= b.len(), "truncated word");
    Ok(u32::from_le_bytes(b[i..i + 4].try_into().unwrap()))
}
fn wrap(magic: &[u8; 4], payload: Vec<u8>) -> Vec<u8> {
    let mut b = magic.to_vec();
    b.extend(1u32.to_le_bytes());
    b.extend((payload.len() as u64).to_le_bytes());
    b.extend(hash(&payload).to_le_bytes());
    b.extend(payload);
    b
}
fn unwrap<'a>(b: &'a [u8], magic: &[u8; 4]) -> Result<&'a [u8]> {
    ensure!(
        b.len() >= 24 && &b[..4] == magic && word(b, 4)? == 1,
        "asset header"
    );
    ensure!(
        u64::from_le_bytes(b[8..16].try_into().unwrap()) == b.len() as u64 - 24
            && u64::from_le_bytes(b[16..24].try_into().unwrap()) == hash(&b[24..]),
        "asset length/checksum"
    );
    Ok(&b[24..])
}
pub fn encode_mesh(m: &Mesh) -> Vec<u8> {
    wrap(b"IGM1", mesh_bytes(m))
}
pub fn decode_mesh(b: &[u8]) -> Result<Mesh> {
    let b = unwrap(b, b"IGM1")?;
    let mut off = 0;
    let m = read_mesh(b, &mut off)?;
    ensure!(off == b.len(), "trailing source data");
    Ok(m)
}
pub fn encode(c: &Compiled) -> Vec<u8> {
    let mut b = c.source_hash.to_le_bytes().to_vec();
    let shared = c.regions.iter().all(|r| r.counts[3] == 0);
    b.extend(u32::from(shared).to_le_bytes());
    b.extend(mesh_bytes(&c.source));
    if !shared {
        b.extend(mesh_bytes(&c.proxy));
    }
    b.extend((c.regions.len() as u32).to_le_bytes());
    for r in &c.regions {
        for f in r
            .low
            .into_iter()
            .chain([r.error])
            .chain(r.high)
            .chain([r.normal_error])
        {
            b.extend(f.to_le_bytes());
        }
        for n in r.counts {
            b.extend(n.to_le_bytes());
        }
    }
    wrap(b"IGC1", b)
}
pub fn decode(b: &[u8]) -> Result<Compiled> {
    let b = unwrap(b, b"IGC1")?;
    ensure!(b.len() >= 12, "compiled header");
    let source_hash = u64::from_le_bytes(b[..8].try_into().unwrap());
    let shared = word(b, 8)?;
    ensure!(shared <= 1, "invalid shared geometry flag");
    let mut off = 12;
    let source = read_mesh(b, &mut off)?;
    let proxy = if shared == 1 {
        source.clone()
    } else {
        read_mesh(b, &mut off)?
    };
    let count = word(b, off)? as usize;
    off += 4;
    ensure!(
        count > 0 && off + count * 48 == b.len(),
        "certificate layout"
    );
    let mut regions = vec![];
    for _ in 0..count {
        let mut a = [0f32; 8];
        for v in &mut a {
            *v = f32::from_bits(word(b, off)?);
            off += 4;
        }
        let mut counts = [0; 4];
        for v in &mut counts {
            *v = word(b, off)?;
            off += 4;
        }
        ensure!(
            a.iter().all(|x| x.is_finite())
                && a[3] >= 0.
                && a[7] >= 0.
                && (0..3).all(|i| a[i] <= a[i + 4])
                && counts[0] > 0
                && counts[1] > 0
                && counts[2] < 3
                && counts[3] <= 1,
            "invalid certificate"
        );
        regions.push(Certificate {
            low: a[..3].try_into().unwrap(),
            error: a[3],
            high: a[4..7].try_into().unwrap(),
            normal_error: a[7],
            counts,
        });
    }
    ensure!(
        regions.iter().map(|r| r.counts[0] as usize).sum::<usize>() == source.faces.len()
            && regions.iter().map(|r| r.counts[1] as usize).sum::<usize>() == proxy.faces.len(),
        "certificate coverage"
    );
    ensure!(
        shared == 0 || regions.iter().all(|r| r.counts[3] == 0),
        "shared geometry certificate"
    );
    Ok(Compiled {
        source,
        proxy,
        regions,
        source_hash,
    })
}
#[derive(Clone, Copy)]
struct Plane {
    p: DVec3,
    e: DVec3,
    f: DVec3,
    n: DVec3,
    n0: DVec3,
    nu: DVec3,
    nv: DVec3,
    inv: f64,
}
impl Plane {
    fn new(m: &Mesh, t: [u32; 4]) -> Self {
        let a = m.vertices[t[0] as usize];
        let b = m.vertices[t[1] as usize];
        let c = m.vertices[t[2] as usize];
        let e = pos(b) - pos(a);
        let f = pos(c) - pos(a);
        let cross = e.cross(f);
        Self {
            p: pos(a),
            e,
            f,
            n: cross.normalize(),
            n0: normal(a),
            nu: normal(b) - normal(a),
            nv: normal(c) - normal(a),
            inv: 1. / cross.length_squared(),
        }
    }
    fn uv(&self, p: DVec3) -> [f64; 2] {
        let q = p - self.p;
        [
            (q.dot(self.e) * self.f.length_squared() - q.dot(self.f) * self.e.dot(self.f))
                * self.inv,
            (q.dot(self.f) * self.e.length_squared() - q.dot(self.e) * self.e.dot(self.f))
                * self.inv,
        ]
    }
    fn errors(&self, v: Vertex) -> [f64; 2] {
        let p = pos(v);
        let [u, w] = self.uv(p);
        [
            (p - self.p).dot(self.n).abs(),
            (normal(v) - (self.n0 + self.nu * u + self.nv * w)).length(),
        ]
    }
    fn accepts(&self, m: &Mesh, t: [u32; 4]) -> bool {
        let p = [t[0], t[1], t[2]].map(|i| pos(m.vertices[i as usize]));
        (p[1] - p[0]).cross(p[2] - p[0]).dot(self.n) > 0.
            && t[..3].iter().all(|&i| {
                let e = self.errors(m.vertices[i as usize]);
                e[0] <= POSITION_TOL && e[1] <= NORMAL_TOL
            })
    }
}
#[derive(Clone, Copy)]
struct Edge {
    key: u64,
    face: u32,
    side: u32,
}
fn key(a: u32, b: u32) -> u64 {
    ((a.min(b) as u64) << 32) | a.max(b) as u64
}
fn weld(m: &Mesh) -> Mesh {
    let bits = |v: Vertex| {
        let mut a = [0; 6];
        for (o, x) in a.iter_mut().zip(v.p.into_iter().chain(v.n)) {
            *o = if x == 0. { 0 } else { x.to_bits() };
        }
        a
    };
    let mut order = (0..m.vertices.len() as u32).collect::<Vec<_>>();
    order.sort_unstable_by_key(|&i| bits(m.vertices[i as usize]));
    let mut map = vec![0; m.vertices.len()];
    let mut vertices = vec![];
    let mut last = None;
    for i in order {
        let b = bits(m.vertices[i as usize]);
        if last != Some(b) {
            vertices.push(m.vertices[i as usize]);
            last = Some(b);
        }
        map[i as usize] = (vertices.len() - 1) as u32;
    }
    Mesh {
        vertices,
        faces: m
            .faces
            .iter()
            .map(|t| {
                [
                    map[t[0] as usize],
                    map[t[1] as usize],
                    map[t[2] as usize],
                    t[3],
                ]
            })
            .collect(),
    }
}
struct Region {
    faces: Vec<u32>,
    plane: Plane,
    material: u32,
}
fn segment_distance(p: DVec3, a: DVec3, b: DVec3) -> f64 {
    let d = b - a;
    (p - (a + d * ((p - a).dot(d) / d.length_squared()).clamp(0., 1.))).length()
}
fn triangulate_boundary(m: &Mesh, ring: &[u32], n: DVec3, material: u32) -> Option<Vec<[u32; 4]>> {
    let mut polygon = ring.to_vec();
    let mut output = vec![];
    while polygon.len() > 3 {
        let mut ear = None;
        for i in 0..polygon.len() {
            let ids = [
                polygon[(i + polygon.len() - 1) % polygon.len()],
                polygon[i],
                polygon[(i + 1) % polygon.len()],
            ];
            let [a, b, c] = ids.map(|v| pos(m.vertices[v as usize]));
            if (b - a).cross(c - a).dot(n) <= 1e-12 {
                continue;
            }
            let blocked = polygon.iter().filter(|v| !ids.contains(v)).any(|&v| {
                let p = pos(m.vertices[v as usize]);
                (b - a).cross(p - a).dot(n) >= -1e-16
                    && (c - b).cross(p - b).dot(n) >= -1e-16
                    && (a - c).cross(p - c).dot(n) >= -1e-16
            });
            if !blocked {
                ear = Some((i, [ids[0], ids[1], ids[2], material]));
                break;
            }
        }
        let (i, face) = ear?;
        output.push(face);
        polygon.remove(i);
    }
    if polygon.len() != 3 {
        return None;
    }
    let [a, b, c] = [polygon[0], polygon[1], polygon[2]].map(|v| pos(m.vertices[v as usize]));
    if (b - a).cross(c - a).dot(n) <= 1e-12 {
        return None;
    }
    output.push([polygon[0], polygon[1], polygon[2], material]);
    Some(output)
}
#[derive(Default, serde::Serialize)]
pub struct Diagnostics {
    pub refits: usize,
    pub shortcut_candidates: usize,
    pub shortcut_collapses: usize,
    pub topology_rejected: usize,
    pub no_reduction: usize,
    pub containment_rejected: usize,
    pub area_rejected: usize,
    pub triangulation_rejected: usize,
    pub restored_native: usize,
}
pub fn compile(input: &Mesh, source_hash: u64) -> Result<Compiled> {
    Ok(compile_traced(input, source_hash, false)?.0)
}
#[allow(dead_code)]
pub fn compile_traced(
    input: &Mesh,
    source_hash: u64,
    stable: bool,
) -> Result<(Compiled, Diagnostics)> {
    compile_borders(input, source_hash, stable, false)
}
#[allow(dead_code)]
pub fn compile_borders(
    input: &Mesh,
    source_hash: u64,
    stable: bool,
    merge_shortcuts: bool,
) -> Result<(Compiled, Diagnostics)> {
    let mut diagnostics = Diagnostics::default();
    validate(input)?;
    let mut source = weld(input);
    source.faces.sort_unstable_by_key(|t| {
        (
            t[3],
            *t[..3].iter().min().unwrap(),
            *t[..3].iter().max().unwrap(),
        )
    });
    let m = &source;
    let mut edges = Vec::with_capacity(m.faces.len() * 3);
    for (i, t) in m.faces.iter().enumerate() {
        for j in 0..3 {
            edges.push(Edge {
                key: key(t[j], t[(j + 1) % 3]),
                face: i as u32,
                side: j as u32,
            });
        }
    }
    edges.sort_unstable_by_key(|e| e.key);
    let mut adjacency = vec![[u32::MAX; 3]; m.faces.len()];
    let mut i = 0;
    while i < edges.len() {
        let mut end = i + 1;
        while end < edges.len() && edges[end].key == edges[i].key {
            end += 1;
        }
        if end == i + 2 {
            let a = edges[i];
            let b = edges[i + 1];
            let ta = m.faces[a.face as usize];
            let tb = m.faces[b.face as usize];
            if ta[a.side as usize] == tb[(b.side as usize + 1) % 3]
                && ta[(a.side as usize + 1) % 3] == tb[b.side as usize]
            {
                adjacency[a.face as usize][a.side as usize] = b.face;
                adjacency[b.face as usize][b.side as usize] = a.face;
            }
        }
        i = end;
    }
    let mut seeds = (0..m.faces.len() as u32).collect::<Vec<_>>();
    let area = |i: u32| {
        let t = m.faces[i as usize];
        (pos(m.vertices[t[1] as usize]) - pos(m.vertices[t[0] as usize]))
            .cross(pos(m.vertices[t[2] as usize]) - pos(m.vertices[t[0] as usize]))
            .length_squared()
    };
    seeds.sort_unstable_by(|&a, &b| {
        area(b).total_cmp(&area(a)).then_with(|| {
            let mut a = m.faces[a as usize];
            let mut b = m.faces[b as usize];
            a[..3].sort_unstable();
            b[..3].sort_unstable();
            a.cmp(&b)
        })
    });
    let mut labels = vec![u32::MAX; m.faces.len()];
    let mut regions = vec![];
    for seed in seeds {
        if labels[seed as usize] != u32::MAX {
            continue;
        }
        let mut plane = Plane::new(m, m.faces[seed as usize]);
        let mat = m.faces[seed as usize][3];
        let id = regions.len() as u32;
        let mut faces = vec![seed];
        labels[seed as usize] = id;
        let mut cursor = 0;
        while cursor < faces.len() {
            for next in adjacency[faces[cursor] as usize] {
                if next != u32::MAX
                    && labels[next as usize] == u32::MAX
                    && m.faces[next as usize][3] == mat
                    && plane.accepts(m, m.faces[next as usize])
                {
                    labels[next as usize] = id;
                    faces.push(next);
                    if stable && faces.len().is_power_of_two() {
                        let anchor = m.faces[seed as usize][0];
                        let vertices = faces
                            .iter()
                            .flat_map(|&f| m.faces[f as usize][..3].iter().copied());
                        let a = vertices
                            .clone()
                            .max_by(|&a, &b| {
                                (pos(m.vertices[a as usize]) - pos(m.vertices[anchor as usize]))
                                    .length_squared()
                                    .total_cmp(
                                        &(pos(m.vertices[b as usize])
                                            - pos(m.vertices[anchor as usize]))
                                        .length_squared(),
                                    )
                                    .then(a.cmp(&b))
                            })
                            .unwrap();
                        let b = vertices
                            .clone()
                            .max_by(|&a0, &b0| {
                                (pos(m.vertices[a0 as usize]) - pos(m.vertices[a as usize]))
                                    .length_squared()
                                    .total_cmp(
                                        &(pos(m.vertices[b0 as usize])
                                            - pos(m.vertices[a as usize]))
                                        .length_squared(),
                                    )
                                    .then(a0.cmp(&b0))
                            })
                            .unwrap();
                        let edge = pos(m.vertices[b as usize]) - pos(m.vertices[a as usize]);
                        let c = vertices
                            .max_by(|&a0, &b0| {
                                edge.cross(
                                    pos(m.vertices[a0 as usize]) - pos(m.vertices[a as usize]),
                                )
                                .length_squared()
                                .total_cmp(
                                    &edge
                                        .cross(
                                            pos(m.vertices[b0 as usize])
                                                - pos(m.vertices[a as usize]),
                                        )
                                        .length_squared(),
                                )
                                .then(a0.cmp(&b0))
                            })
                            .unwrap();
                        let mut fit = Plane::new(m, [a, b, c, mat]);
                        if fit.n.dot(plane.n) < 0. {
                            fit = Plane::new(m, [b, a, c, mat]);
                        }
                        if faces.iter().all(|&f| fit.accepts(m, m.faces[f as usize])) {
                            plane = fit;
                            diagnostics.refits += 1;
                        }
                    }
                }
            }
            cursor += 1;
        }
        regions.push(Region {
            faces,
            plane,
            material: mat,
        });
    }
    // Global border graph. Collapse an edge-chain vertex only once for both sides.
    let mut borders = vec![Vec::<(u32, u32)>::new(); regions.len()];
    let mut graph = vec![Vec::<(u32, f64)>::new(); m.vertices.len()];
    for e in &edges {
        let next = adjacency[e.face as usize][e.side as usize];
        if next == u32::MAX || labels[next as usize] != labels[e.face as usize] {
            let t = m.faces[e.face as usize];
            let a = t[e.side as usize];
            let b = t[(e.side as usize + 1) % 3];
            borders[labels[e.face as usize] as usize].push((a, b));
            if !graph[a as usize].iter().any(|x| x.0 == b) {
                graph[a as usize].push((b, 0.));
                graph[b as usize].push((a, 0.));
            }
        }
    }
    drop(edges);
    drop(adjacency);
    drop(labels);
    let mut keep = vec![true; m.vertices.len()];
    let mut heap = BinaryHeap::new();
    for (i, g) in graph.iter().enumerate() {
        if g.len() == 2 {
            heap.push(Reverse(i as u32));
        }
    }
    while let Some(Reverse(v)) = heap.pop() {
        if !keep[v as usize] || graph[v as usize].len() != 2 {
            continue;
        }
        let (a, ea) = graph[v as usize][0];
        let (b, eb) = graph[v as usize][1];
        if a == b {
            continue;
        }
        let err = ea.max(eb)
            + segment_distance(
                pos(m.vertices[v as usize]),
                pos(m.vertices[a as usize]),
                pos(m.vertices[b as usize]),
            );
        if err > BORDER_TOL {
            continue;
        }
        let shortcut = graph[a as usize].iter().any(|x| x.0 == b);
        if shortcut {
            diagnostics.shortcut_candidates += 1;
            if !merge_shortcuts {
                continue;
            }
            diagnostics.shortcut_collapses += 1;
        }
        keep[v as usize] = false;
        for (n, other) in [(a, b), (b, a)] {
            if shortcut {
                // Both paths represent the same segment. Keep its worst accumulated
                // error so a later collapse cannot discard the earlier bound.
                graph[n as usize].retain(|e| e.0 != v);
                let e = graph[n as usize].iter_mut().find(|e| e.0 == other).unwrap();
                e.1 = e.1.max(err);
            } else {
                let e = graph[n as usize].iter_mut().find(|e| e.0 == v).unwrap();
                *e = (other, err);
            }
            if graph[n as usize].len() == 2 {
                heap.push(Reverse(n));
            }
        }
    }
    let mut output = vec![];
    let mut certificates = vec![];
    let mut candidates = vec![];
    let mut rings = vec![];
    for (ri, r) in regions.iter().enumerate() {
        let mut low = DVec3::splat(f64::INFINITY);
        let mut high = DVec3::splat(f64::NEG_INFINITY);
        let mut errors = [0f64; 2];
        let mut verts = vec![];
        let mut projected_area = 0.;
        for &f in &r.faces {
            let t = m.faces[f as usize];
            for &v in &t[..3] {
                let p = pos(m.vertices[v as usize]);
                low = low.min(p);
                high = high.max(p);
                let e = r.plane.errors(m.vertices[v as usize]);
                errors[0] = errors[0].max(e[0]);
                errors[1] = errors[1].max(e[1]);
                verts.push(v);
            }
            projected_area += (pos(m.vertices[t[1] as usize]) - pos(m.vertices[t[0] as usize]))
                .cross(pos(m.vertices[t[2] as usize]) - pos(m.vertices[t[0] as usize]))
                .dot(r.plane.n)
                * 0.5;
        }
        verts.sort_unstable();
        verts.dedup();
        let border = &borders[ri];
        let mut next = HashMap::new();
        let mut valid = true;
        for &(a, b) in border {
            if next.insert(a, b).is_some() {
                valid = false;
            }
        }
        let mut ring = vec![];
        if let Some(&(start, _)) = border.first() {
            let mut v = start;
            for _ in 0..=border.len() {
                if ring.contains(&v) {
                    break;
                }
                ring.push(v);
                if let Some(&n) = next.get(&v) {
                    v = n;
                } else {
                    valid = false;
                    break;
                }
            }
            valid &= v == start && ring.len() == border.len();
        } else {
            valid = false;
        }
        // Disk Euler characteristic: internal edge count=(3F-B)/2.
        valid &= (3 * r.faces.len() - border.len()).is_multiple_of(2)
            && verts.len() as i64 - ((3 * r.faces.len() + border.len()) / 2) as i64
                + r.faces.len() as i64
                == 1;
        if !valid {
            diagnostics.topology_rejected += 1;
        }
        let coarse = ring
            .iter()
            .copied()
            .filter(|&i| keep[i as usize])
            .collect::<Vec<_>>();
        if valid && !(coarse.len() >= 3 && coarse.len() - 2 < r.faces.len()) {
            diagnostics.no_reduction += 1;
        }
        valid &= coarse.len() >= 3 && coarse.len() - 2 < r.faces.len();
        let mut area = 0.;
        if valid {
            let p0 = pos(m.vertices[coarse[0] as usize]);
            for i in 1..coarse.len() - 1 {
                area += (pos(m.vertices[coarse[i] as usize]) - p0)
                    .cross(pos(m.vertices[coarse[i + 1] as usize]) - p0)
                    .dot(r.plane.n)
                    * 0.5;
            }
            for i in 0..coarse.len() {
                let a = pos(m.vertices[coarse[i] as usize]);
                let b = pos(m.vertices[coarse[(i + 1) % coarse.len()] as usize]);
                let e = b - a;
                for &v in &verts {
                    if e.cross(pos(m.vertices[v as usize]) - a).dot(r.plane.n)
                        < -BORDER_TOL * e.length() * 1.1
                    {
                        valid = false;
                        diagnostics.containment_rejected += 1;
                        break;
                    }
                }
                if !valid {
                    break;
                }
            }
            let perimeter = ring
                .iter()
                .enumerate()
                .map(|(i, &a)| {
                    (pos(m.vertices[a as usize])
                        - pos(m.vertices[ring[(i + 1) % ring.len()] as usize]))
                    .length()
                })
                .sum::<f64>();
            if valid
                && !(area > 0.
                    && (area - projected_area).abs()
                        <= perimeter * BORDER_TOL * 2. + projected_area * 1e-9)
            {
                diagnostics.area_rejected += 1;
            }
            valid &= area > 0.
                && (area - projected_area).abs()
                    <= perimeter * BORDER_TOL * 2. + projected_area * 1e-9;
        }
        let triangulated = valid
            .then(|| triangulate_boundary(m, &coarse, r.plane.n, r.material))
            .flatten();
        if valid && triangulated.is_none() {
            diagnostics.triangulation_rejected += 1;
        }
        valid &= triangulated.is_some();
        let start = output.len();
        if let Some(faces) = triangulated {
            output.extend(faces);
        } else {
            output.extend(r.faces.iter().map(|&i| m.faces[i as usize]));
        }
        candidates.push((start, output.len(), valid));
        rings.push(ring);
        let extent = low.abs().max(high.abs()).max_element().max(1.);
        certificates.push(Certificate {
            low: low.as_vec3().to_array(),
            high: high.as_vec3().to_array(),
            error: if valid {
                ((2. * errors[0] + BORDER_TOL + 16. * f32::EPSILON as f64 * extent) as f32)
                    .next_up()
            } else {
                0.
            },
            normal_error: if valid {
                (2. * errors[1] as f32).next_up()
            } else {
                0.
            },
            counts: [
                r.faces.len() as u32,
                (output.len() - start) as u32,
                r.material,
                u32::from(valid),
            ],
        });
    }
    // Restore vertices only along boundaries touching a native region. Neighbors
    // triangulate their interiors with those boundary vertices still present.
    // If triangulation fails, retain that region and repeat the boundary constraint.
    loop {
        let mut forced = keep.clone();
        for (ri, border) in borders.iter().enumerate() {
            if !candidates[ri].2 {
                for &(v, _) in border {
                    forced[v as usize] = true;
                }
            }
        }
        let mut safe = Vec::with_capacity(output.len());
        let mut rejected = false;
        for (ri, r) in regions.iter().enumerate() {
            let (start, end, valid) = candidates[ri];
            let changed = rings[ri]
                .iter()
                .any(|&v| forced[v as usize] && !keep[v as usize]);
            let restored = if valid && changed {
                let ring = rings[ri]
                    .iter()
                    .copied()
                    .filter(|&v| forced[v as usize])
                    .collect::<Vec<_>>();
                triangulate_boundary(m, &ring, r.plane.n, r.material)
                    .filter(|faces| faces.len() < r.faces.len())
            } else {
                None
            };
            if valid && (!changed || restored.is_some()) {
                let faces = restored.as_deref().unwrap_or(&output[start..end]);
                certificates[ri].counts[1] = faces.len() as u32;
                safe.extend_from_slice(faces);
            } else {
                if valid {
                    candidates[ri].2 = false;
                    diagnostics.restored_native += 1;
                    rejected = true;
                }
                safe.extend(r.faces.iter().map(|&i| m.faces[i as usize]));
                certificates[ri].error = 0.;
                certificates[ri].normal_error = 0.;
                certificates[ri].counts[1] = r.faces.len() as u32;
                certificates[ri].counts[3] = 0;
            }
        }
        if !rejected {
            output = safe;
            break;
        }
    }
    let mut used = output
        .iter()
        .flat_map(|f| f[..3].iter().copied())
        .collect::<Vec<_>>();
    used.sort_unstable();
    used.dedup();
    let mut remap = vec![0; m.vertices.len()];
    let vertices = used
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            remap[v as usize] = i as u32;
            m.vertices[v as usize]
        })
        .collect();
    for f in &mut output {
        for i in &mut f[..3] {
            *i = remap[*i as usize];
        }
    }
    output.sort_unstable_by_key(|t| {
        (
            t[3],
            *t[..3].iter().min().unwrap(),
            *t[..3].iter().max().unwrap(),
        )
    });
    let proxy = Mesh {
        vertices,
        faces: output,
    };
    validate(&proxy)?;
    Ok((
        Compiled {
            source,
            proxy,
            regions: certificates,
            source_hash,
        },
        diagnostics,
    ))
}

#[cfg(test)]
mod normal_validation_tests {
    use super::*;
    #[test]
    fn interpolated_normals_can_be_short_but_not_zero() -> Result<()> {
        let mut mesh = Mesh {
            vertices: vec![
                Vertex {
                    p: [0., 0., 0.],
                    n: [0.05, 0.05, 0.],
                },
                Vertex {
                    p: [1., 0., 0.],
                    n: [1., 0., 0.],
                },
                Vertex {
                    p: [0., 1., 0.],
                    n: [-0.9, 0.1, 0.],
                },
            ],
            faces: vec![[0, 1, 2, 0]],
        };
        validate(&mesh)?;
        let decoded = decode_mesh(&encode_mesh(&mesh))?;
        assert_eq!(decoded.vertices[0].n, [0.05, 0.05, 0.]);
        mesh.vertices[0].n = [0.; 3];
        assert!(validate(&mesh).is_err());
        Ok(())
    }
}
