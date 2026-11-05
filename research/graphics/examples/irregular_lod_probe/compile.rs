use super::compiler::{self, Mesh, Vertex};
use anyhow::{Result, ensure};
use glam::{DVec2, DVec3, Vec3};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug)]
pub struct Level {
    pub start: u32,
    pub count: u32,
    pub error: f32,
    pub normal: f32,
}
#[derive(Clone, Debug)]
pub struct Patch {
    pub low: [f32; 3],
    pub high: [f32; 3],
    pub cone: [f32; 4],
    pub levels: Vec<Level>,
}
pub struct Asset {
    pub mesh: Mesh,
    pub source_count: u32,
    pub patches: Vec<Patch>,
}
fn p(v: Vertex) -> DVec3 {
    DVec3::from_array(v.p.map(f64::from))
}
fn n(v: Vertex) -> DVec3 {
    DVec3::from_array(v.n.map(f64::from))
}
fn edge(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}
#[derive(Clone, Copy)]
struct Frame {
    u: DVec3,
    v: DVec3,
    n: DVec3,
}
impl Frame {
    fn from_normal(n: DVec3) -> Self {
        let u = n.any_orthonormal_vector();
        Self {
            u,
            v: n.cross(u),
            n,
        }
    }
    fn xy(self, p: DVec3) -> DVec2 {
        DVec2::new(p.dot(self.u), p.dot(self.v))
    }
}
fn cross(a: DVec2, b: DVec2) -> f64 {
    a.x * b.y - a.y * b.x
}
#[derive(Clone)]
struct Tri {
    q: [DVec2; 3],
    z: [f64; 3],
    norm: [DVec3; 3],
    low: DVec2,
    high: DVec2,
    area: f64,
}
impl Tri {
    fn new(m: &Mesh, t: [u32; 4], f: Frame) -> Self {
        let a = [t[0], t[1], t[2]].map(|i| m.vertices[i as usize]);
        let q = a.map(|v| f.xy(p(v)));
        let area = cross(q[1] - q[0], q[2] - q[0]) * 0.5;
        Self {
            q,
            z: a.map(|v| p(v).dot(f.n)),
            norm: a.map(n),
            low: q[0].min(q[1]).min(q[2]),
            high: q[0].max(q[1]).max(q[2]),
            area,
        }
    }
    fn sample(&self, p: DVec2) -> (f64, DVec3) {
        let b = cross(p - self.q[0], self.q[2] - self.q[0]) / (2. * self.area);
        let c = cross(self.q[1] - self.q[0], p - self.q[0]) / (2. * self.area);
        let a = 1. - b - c;
        (
            self.z[0] * a + self.z[1] * b + self.z[2] * c,
            self.norm[0] * a + self.norm[1] * b + self.norm[2] * c,
        )
    }
}
fn intersection(a: &Tri, b: &Tri) -> Vec<DVec2> {
    if a.high.x < b.low.x || b.high.x < a.low.x || a.high.y < b.low.y || b.high.y < a.low.y {
        return vec![];
    }
    let mut poly = a.q.to_vec();
    for i in 0..3 {
        let origin = b.q[i];
        let e = b.q[(i + 1) % 3] - origin;
        let mut next = vec![];
        if poly.is_empty() {
            break;
        }
        let mut prev = *poly.last().unwrap();
        let mut dp = cross(e, prev - origin);
        for &cur in &poly {
            let dc = cross(e, cur - origin);
            if (dp >= 0.) != (dc >= 0.) {
                next.push(prev + (cur - prev) * (dp / (dp - dc)));
            }
            if dc >= 0. {
                next.push(cur);
            }
            prev = cur;
            dp = dc;
        }
        poly = next;
    }
    poly
}
fn area(poly: &[DVec2]) -> f64 {
    if poly.len() < 3 {
        return 0.;
    }
    (1..poly.len() - 1)
        .map(|i| cross(poly[i] - poly[0], poly[i + 1] - poly[0]) * 0.5)
        .sum::<f64>()
        .abs()
}
fn certify(src: &[Tri], dst: &[Tri]) -> Option<(f32, f32)> {
    if src.iter().chain(dst).any(|t| t.area <= 1e-14) {
        return None;
    }
    let mut coverage = vec![0.; dst.len()];
    let mut err = 0f64;
    let mut normal = 0f64;
    for a in src {
        let mut covered = 0.;
        for (j, b) in dst.iter().enumerate() {
            let poly = intersection(a, b);
            let ar = area(&poly);
            if ar <= 1e-18 {
                continue;
            }
            covered += ar;
            coverage[j] += ar;
            for q in poly {
                let (z, n) = a.sample(q);
                let (w, m) = b.sample(q);
                err = err.max((z - w).abs());
                normal = normal.max((n - m).length());
            }
        }
        if (covered - a.area).abs() > a.area * 1e-7 + 1e-13 {
            return None;
        }
    }
    if dst
        .iter()
        .zip(coverage)
        .any(|(t, a)| (t.area - a).abs() > t.area * 1e-7 + 1e-13)
    {
        return None;
    }
    Some((
        ((err + 2e-6) as f32).next_up(),
        ((normal + 2e-6) as f32).next_up(),
    ))
}
fn nonoverlap(tris: &[Tri]) -> bool {
    for (i, a) in tris.iter().enumerate() {
        if a.area <= 1e-14 {
            return false;
        }
        for b in &tris[i + 1..] {
            if area(&intersection(a, b)) > 1e-12 {
                return false;
            }
        }
    }
    true
}
fn simplify(
    mesh: &Mesh,
    faces: &mut Vec<[u32; 4]>,
    locked: &HashSet<u32>,
    frame: Frame,
    target: usize,
    budget: (f32, f32),
) {
    // Batch independent endpoint collapses. Rebuild connectivity only offline.
    for _ in 0..20 {
        if faces.len() <= target {
            break;
        }
        let mut incident = HashMap::<u32, Vec<usize>>::new();
        let mut edges = HashSet::new();
        for (i, t) in faces.iter().enumerate() {
            for j in 0..3 {
                incident.entry(t[j]).or_default().push(i);
                edges.insert(edge(t[j], t[(j + 1) % 3]));
            }
        }
        let mut choices = vec![];
        for (a, b) in edges {
            for (a, b) in [(a, b), (b, a)] {
                if locked.contains(&a) {
                    continue;
                }
                let pb = p(mesh.vertices[b as usize]);
                let mut cost = 0.;
                for &i in &incident[&a] {
                    let t = faces[i];
                    let [v, w, z] = [t[0], t[1], t[2]].map(|v| p(mesh.vertices[v as usize]));
                    let normal = (w - v).cross(z - v).normalize();
                    cost += (pb - v).dot(normal).powi(2);
                }
                cost += 0.001
                    * (n(mesh.vertices[a as usize]) - n(mesh.vertices[b as usize]))
                        .length_squared()
                    * (p(mesh.vertices[a as usize]) - pb).length_squared();
                choices.push((cost, a, b));
            }
        }
        choices
            .sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut touched = HashSet::new();
        let mut deleted = vec![false; faces.len()];
        let mut count = faces.len();
        let mut accepted = 0;
        for (_, a, b) in choices {
            if count <= target {
                break;
            }
            if touched.contains(&a) || touched.contains(&b) {
                continue;
            }
            let neighbors = |v| {
                incident[&v]
                    .iter()
                    .flat_map(|&i| faces[i][..3].to_vec())
                    .filter(|&x| x != v)
                    .collect::<HashSet<_>>()
            };
            let na = neighbors(a);
            let nb = neighbors(b);
            if na.iter().chain(&nb).any(|v| touched.contains(v)) {
                continue;
            }
            let common = na.intersection(&nb).copied().collect::<HashSet<_>>();
            let across = incident[&a]
                .iter()
                .filter_map(|&i| {
                    let t = faces[i];
                    if t[..3].contains(&b) {
                        t[..3].iter().copied().find(|&v| v != a && v != b)
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();
            if across.len() != 2 || common != across {
                continue;
            }
            let mut valid = true;
            for &i in &incident[&a] {
                let t = faces[i];
                if t[..3].contains(&b) {
                    continue;
                }
                let q = t.map(|v| v);
                let xyz = [q[0], q[1], q[2]]
                    .map(|v| p(mesh.vertices[if v == a { b } else { v } as usize]));
                let old = [t[0], t[1], t[2]].map(|v| p(mesh.vertices[v as usize]));
                let nn = (xyz[1] - xyz[0]).cross(xyz[2] - xyz[0]);
                let on = (old[1] - old[0]).cross(old[2] - old[0]);
                if nn.dot(frame.n) <= 1e-12 || nn.dot(on) <= 0. {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }
            let before = incident[&a]
                .iter()
                .map(|&i| Tri::new(mesh, faces[i], frame))
                .collect::<Vec<_>>();
            let after = incident[&a]
                .iter()
                .filter_map(|&i| {
                    let mut t = faces[i];
                    if t[..3].contains(&b) {
                        return None;
                    }
                    for v in &mut t[..3] {
                        if *v == a {
                            *v = b;
                        }
                    }
                    Some(Tri::new(mesh, t, frame))
                })
                .collect::<Vec<_>>();
            if !certify(&before, &after).is_some_and(|(e, n)| e <= budget.0 && n <= budget.1) {
                continue;
            }
            for &i in &incident[&a] {
                if faces[i][..3].contains(&b) {
                    deleted[i] = true;
                    count -= 1;
                } else {
                    for v in &mut faces[i][..3] {
                        if *v == a {
                            *v = b;
                        }
                    }
                }
            }
            touched.extend(na);
            touched.extend(nb);
            touched.insert(a);
            touched.insert(b);
            accepted += 1;
        }
        let mut i = 0;
        faces.retain(|_| {
            let keep = !deleted[i];
            i += 1;
            keep
        });
        if accepted == 0 {
            break;
        }
    }
}
pub fn build(source: Mesh, patch_size: usize) -> Result<Asset> {
    build_impl(source, patch_size, false)
}
#[allow(dead_code)]
pub fn build_coordinated(source: Mesh, patch_size: usize) -> Result<Asset> {
    build_impl(source, patch_size, true)
}
fn build_impl(source: Mesh, patch_size: usize, coordinated: bool) -> Result<Asset> {
    compiler::validate(&source)?;
    ensure!(
        source.faces.iter().all(|t| t[3] == 0),
        "diagnostic material0 required"
    );
    let mut ids = HashMap::new();
    let mut vertices = vec![];
    let mut sums = Vec::<DVec3>::new();
    let mut counts = Vec::<usize>::new();
    let mut remap = vec![];
    for &v in &source.vertices {
        let key = v.p.map(|v| if v == 0. { 0 } else { v.to_bits() });
        let id = *ids.entry(key).or_insert_with(|| {
            vertices.push(v);
            sums.push(DVec3::ZERO);
            counts.push(0);
            (vertices.len() - 1) as u32
        });
        sums[id as usize] += n(v);
        counts[id as usize] += 1;
        remap.push(id);
    }
    for (i, v) in vertices.iter_mut().enumerate() {
        v.n = (sums[i] / counts[i] as f64).as_vec3().to_array();
    }
    let topo = Mesh {
        vertices,
        faces: source
            .faces
            .iter()
            .map(|t| {
                [
                    remap[t[0] as usize],
                    remap[t[1] as usize],
                    remap[t[2] as usize],
                    0,
                ]
            })
            .collect(),
    };
    let mut edges = HashMap::<(u32, u32), Vec<usize>>::new();
    for (i, t) in topo.faces.iter().enumerate() {
        for j in 0..3 {
            edges.entry(edge(t[j], t[(j + 1) % 3])).or_default().push(i);
        }
    }
    let mut adj = vec![vec![]; topo.faces.len()];
    for v in edges.values() {
        if v.len() == 2 {
            adj[v[0]].push(v[1]);
            adj[v[1]].push(v[0]);
        }
    }
    for a in &mut adj {
        a.sort_unstable();
    }
    let normals = topo
        .faces
        .iter()
        .map(|t| {
            let [a, b, c] = [t[0], t[1], t[2]].map(|v| p(topo.vertices[v as usize]));
            (b - a).cross(c - a).normalize()
        })
        .collect::<Vec<_>>();
    let mut labels = vec![usize::MAX; topo.faces.len()];
    let mut groups = vec![];
    for seed in 0..topo.faces.len() {
        if labels[seed] != usize::MAX {
            continue;
        }
        let id = groups.len();
        let mut group = vec![seed];
        labels[seed] = id;
        let mut cursor = 0;
        while cursor < group.len() && group.len() < patch_size {
            for &next in &adj[group[cursor]] {
                if labels[next] == usize::MAX && normals[next].dot(normals[seed]) > 0.55 {
                    labels[next] = id;
                    group.push(next);
                    if group.len() == patch_size {
                        break;
                    }
                }
            }
            cursor += 1;
        }
        groups.push(group);
    }
    if coordinated {
        // Merge adjacent valid charts, then simplify their union. Former seams
        // become interior edges; only the new outer boundary remains locked.
        for _ in 0..8 {
            let mut pairs = HashSet::new();
            for (i, neighbors) in adj.iter().enumerate() {
                for &j in neighbors {
                    let a = labels[i];
                    let b = labels[j];
                    if a != b {
                        pairs.insert((a.min(b), a.max(b)));
                    }
                }
            }
            let mut pairs = pairs.into_iter().collect::<Vec<_>>();
            pairs.sort_unstable_by_key(|&(a, b)| (groups[a].len() + groups[b].len(), a, b));
            let mut used = HashSet::new();
            let mut changed = false;
            for (a, b) in pairs {
                if used.contains(&a)
                    || used.contains(&b)
                    || groups[a].len() + groups[b].len() > 1024
                {
                    continue;
                }
                let members = groups[a]
                    .iter()
                    .chain(&groups[b])
                    .copied()
                    .collect::<Vec<_>>();
                let axis = members
                    .iter()
                    .map(|&i| normals[i])
                    .sum::<DVec3>()
                    .normalize();
                if !axis.is_finite() || members.iter().any(|&i| normals[i].dot(axis) < 0.55) {
                    continue;
                }
                let frame = Frame::from_normal(axis);
                let tris = members
                    .iter()
                    .map(|&i| Tri::new(&source, source.faces[i], frame))
                    .collect::<Vec<_>>();
                if !nonoverlap(&tris) {
                    continue;
                }
                for &i in &groups[b] {
                    labels[i] = a;
                }
                let moved = std::mem::take(&mut groups[b]);
                groups[a].extend(moved);
                used.insert(a);
                used.insert(b);
                changed = true;
            }
            if !changed {
                break;
            }
        }
        groups.retain(|g| !g.is_empty());
        for (id, g) in groups.iter().enumerate() {
            for &i in g {
                labels[i] = id;
            }
        }
    }
    let mut global = source.vertices.clone();
    let offset = global.len() as u32;
    global.extend_from_slice(&topo.vertices);
    let mut faces = vec![];
    let mut patches = vec![];
    let mut reductions = vec![];
    for (ri, group) in groups.iter().enumerate() {
        let start = faces.len() as u32 * 3;
        faces.extend(group.iter().map(|&i| source.faces[i]));
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        let mut locked = HashSet::new();
        for &i in group {
            let t = topo.faces[i];
            for j in 0..3 {
                let v = Vec3::from_array(topo.vertices[t[j] as usize].p);
                low = low.min(v);
                high = high.max(v);
                let owners = &edges[&edge(t[j], t[(j + 1) % 3])];
                if owners.len() != 2 || owners.iter().any(|&x| labels[x] != ri) {
                    locked.insert(t[j]);
                    locked.insert(t[(j + 1) % 3]);
                }
            }
        }
        let frame =
            Frame::from_normal(group.iter().map(|&i| normals[i]).sum::<DVec3>().normalize());
        let original = group
            .iter()
            .map(|&i| Tri::new(&source, source.faces[i], frame))
            .collect::<Vec<_>>();
        let mut current = group.iter().map(|&i| topo.faces[i]).collect::<Vec<_>>();
        let mut variants = vec![];
        if nonoverlap(&original) {
            for (ratio, budget) in [
                (0.9, (0.0001, 0.001)),
                (0.65, (0.0005, 0.003)),
                (0.35, (0.002, 0.008)),
            ] {
                simplify(
                    &topo,
                    &mut current,
                    &locked,
                    frame,
                    (group.len() as f64 * ratio) as usize,
                    budget,
                );
                if current.len() < group.len() {
                    let projected = current
                        .iter()
                        .map(|&t| Tri::new(&topo, t, frame))
                        .collect::<Vec<_>>();
                    if let Some((error, normal)) = certify(&original, &projected) {
                        variants.push((current.clone(), error, normal));
                    } else {
                        break;
                    }
                }
            }
        }
        // Diagnostic level: geometric error is certified, normal error is recorded
        // but not constrained during construction. Runtime's normal gate still applies.
        if nonoverlap(&original) {
            let mut diagnostic = group.iter().map(|&i| topo.faces[i]).collect::<Vec<_>>();
            simplify(
                &topo,
                &mut diagnostic,
                &locked,
                frame,
                group.len() * 35 / 100,
                (0.002, 2.0),
            );
            if diagnostic.len() < group.len()
                && variants
                    .last()
                    .is_none_or(|(t, _, _)| diagnostic.len() < t.len())
            {
                let projected = diagnostic
                    .iter()
                    .map(|&t| Tri::new(&topo, t, frame))
                    .collect::<Vec<_>>();
                if let Some((error, normal)) = certify(&original, &projected) {
                    variants.push((diagnostic, error, normal));
                }
            }
        }
        let mut cosine = group
            .iter()
            .map(|&i| normals[i].dot(frame.n))
            .fold(1f64, f64::min);
        for (tris, _, _) in &variants {
            for t in tris {
                let [a, b, c] = [t[0], t[1], t[2]].map(|v| p(topo.vertices[v as usize]));
                cosine = cosine.min((b - a).cross(c - a).normalize().dot(frame.n));
            }
        }
        let sine = (1. - cosine.clamp(0., 1.).powi(2)).sqrt() as f32;
        patches.push(Patch {
            low: low.to_array(),
            high: high.to_array(),
            cone: [
                frame.n.x as f32,
                frame.n.y as f32,
                frame.n.z as f32,
                (sine + 1e-5).min(1.),
            ],
            levels: vec![Level {
                start,
                count: group.len() as u32 * 3,
                error: 0.,
                normal: 0.,
            }],
        });
        reductions.push(variants);
    }
    // All regions without a usable reduction become one fixed native range.
    let old_faces = faces;
    faces = vec![];
    let mut compact = vec![];
    let mut variants = vec![];
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for (i, p) in patches.iter().enumerate() {
        if reductions[i].is_empty() {
            let l = &p.levels[0];
            faces.extend_from_slice(
                &old_faces[l.start as usize / 3..(l.start + l.count) as usize / 3],
            );
            low = low.min(Vec3::from_array(p.low));
            high = high.max(Vec3::from_array(p.high));
        }
    }
    if !faces.is_empty() {
        compact.push(Patch {
            low: low.to_array(),
            high: high.to_array(),
            cone: [0., 0., 1., 1.],
            levels: vec![Level {
                start: 0,
                count: faces.len() as u32 * 3,
                error: 0.,
                normal: 0.,
            }],
        });
        variants.push(vec![]);
    }
    for (i, mut p) in patches.into_iter().enumerate() {
        if !reductions[i].is_empty() {
            let l = &mut p.levels[0];
            let start = faces.len() as u32 * 3;
            faces.extend_from_slice(
                &old_faces[l.start as usize / 3..(l.start + l.count) as usize / 3],
            );
            l.start = start;
            compact.push(p);
            variants.push(std::mem::take(&mut reductions[i]));
        }
    }
    let mut patches = compact;
    let reductions = variants;
    let source_count = faces.len() as u32 * 3;
    for level in 0..4 {
        for (i, variants) in reductions.iter().enumerate() {
            if let Some((tris, error, normal)) = variants.get(level) {
                let start = faces.len() as u32 * 3;
                faces.extend(
                    tris.iter()
                        .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset, 0]),
                );
                patches[i].levels.push(Level {
                    start,
                    count: tris.len() as u32 * 3,
                    error: *error,
                    normal: *normal,
                });
            }
        }
    }
    // Store only referenced vertices. Diagnostic levels share this compact buffer;
    // the position-welded working mesh is not retained wholesale at runtime.
    let mut remap = vec![u32::MAX; global.len()];
    let mut vertices = Vec::new();
    for face in &mut faces {
        for index in &mut face[..3] {
            let slot = &mut remap[*index as usize];
            if *slot == u32::MAX {
                *slot = vertices.len() as u32;
                vertices.push(global[*index as usize]);
            }
            *index = *slot;
        }
    }
    let asset = Asset {
        mesh: Mesh { vertices, faces },
        source_count,
        patches,
    };
    compiler::validate(&asset.mesh)?;
    Ok(asset)
}
pub fn encode(a: &Asset) -> Vec<u8> {
    let mesh = compiler::encode_mesh(&a.mesh);
    let mut b = (mesh.len() as u64).to_le_bytes().to_vec();
    b.extend(mesh);
    b.extend(a.source_count.to_le_bytes());
    b.extend((a.patches.len() as u32).to_le_bytes());
    for p in &a.patches {
        for x in p.low.into_iter().chain(p.high).chain(p.cone) {
            b.extend(x.to_le_bytes());
        }
        b.extend((p.levels.len() as u32).to_le_bytes());
        for l in &p.levels {
            b.extend(l.start.to_le_bytes());
            b.extend(l.count.to_le_bytes());
            b.extend(l.error.to_le_bytes());
            b.extend(l.normal.to_le_bytes());
        }
    }
    let mut out = b"IGL1".to_vec();
    out.extend(1u32.to_le_bytes());
    out.extend((b.len() as u64).to_le_bytes());
    out.extend(compiler::hash(&b).to_le_bytes());
    out.extend(b);
    out
}
pub fn decode(b: &[u8]) -> Result<Asset> {
    ensure!(b.len() >= 32 && &b[..4] == b"IGL1", "IGL header");
    let u = |i| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    let q = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    ensure!(
        u(4) == 1 && q(8) == b.len() as u64 - 24 && q(16) == compiler::hash(&b[24..]),
        "IGL checksum/layout"
    );
    let size = q(24) as usize;
    ensure!(
        size <= b.len() - 32 && size + 40 <= b.len(),
        "IGL mesh length"
    );
    let mesh = compiler::decode_mesh(&b[32..32 + size])?;
    let mut off = 32 + size;
    let source_count = u(off);
    let count = u(off + 4) as usize;
    off += 8;
    ensure!(
        count > 0 && count <= mesh.faces.len() && source_count as usize <= mesh.faces.len() * 3,
        "IGL counts"
    );
    let mut patches = vec![];
    let mut cursor = 0;
    for _ in 0..count {
        ensure!(off + 44 <= b.len(), "IGL patch truncated");
        let bounds: [f32; 6] = std::array::from_fn(|i| f32::from_bits(u(off + i * 4)));
        let cone: [f32; 4] = std::array::from_fn(|i| f32::from_bits(u(off + 24 + i * 4)));
        ensure!(
            cone.iter().all(|v| v.is_finite()) && (0.0..=1.0).contains(&cone[3]),
            "IGL cone"
        );
        let n = u(off + 40) as usize;
        off += 44;
        ensure!(
            (1..=5).contains(&n)
                && off + n * 16 <= b.len()
                && bounds.iter().all(|v| v.is_finite())
                && (0..3).all(|i| bounds[i] <= bounds[i + 3]),
            "IGL patch"
        );
        let mut levels = vec![];
        for i in 0..n {
            let l = Level {
                start: u(off),
                count: u(off + 4),
                error: f32::from_bits(u(off + 8)),
                normal: f32::from_bits(u(off + 12)),
            };
            off += 16;
            ensure!(
                l.count > 0
                    && l.start.is_multiple_of(3)
                    && l.count.is_multiple_of(3)
                    && l.start as usize + l.count as usize <= mesh.faces.len() * 3
                    && l.error.is_finite()
                    && l.error >= 0.
                    && l.normal.is_finite()
                    && l.normal >= 0.,
                "IGL level"
            );
            if i == 0 {
                ensure!(
                    l.start == cursor && l.error == 0. && l.normal == 0.,
                    "IGL source partition"
                );
                cursor += l.count;
            }
            levels.push(l);
        }
        patches.push(Patch {
            low: bounds[..3].try_into().unwrap(),
            high: bounds[3..].try_into().unwrap(),
            cone,
            levels,
        });
    }
    ensure!(
        off == b.len() && cursor == source_count,
        "IGL coverage/trailing"
    );
    Ok(Asset {
        mesh,
        source_count,
        patches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn grid(size: usize) -> Mesh {
        let mut vertices = vec![];
        let mut faces = vec![];
        for y in 0..=size {
            for x in 0..=size {
                let x = x as f32 / size as f32 * 2. - 1.;
                let y = y as f32 / size as f32 * 2. - 1.;
                vertices.push(Vertex {
                    p: [x, y, 0.],
                    n: [0.1 * x, 0.1 * y, 1.],
                });
            }
        }
        for y in 0..size {
            for x in 0..size {
                let a = (y * (size + 1) + x) as u32;
                let b = a + 1;
                let d = a + (size + 1) as u32;
                let c = d + 1;
                faces.extend([[a, b, c, 0], [a, c, d, 0]]);
            }
        }
        Mesh { vertices, faces }
    }
    #[test]
    fn exact_piecewise_certificate_detects_relief_and_normal_error() {
        let frame = Frame::from_normal(DVec3::Z);
        let mut m = grid(1);
        let a = m
            .faces
            .iter()
            .map(|&t| Tri::new(&m, t, frame))
            .collect::<Vec<_>>();
        let other = [[0, 1, 2, 0], [1, 3, 2, 0]].map(|t| Tri::new(&m, t, frame));
        let e = certify(&a, &other).unwrap();
        assert!(e.0 < 2.1e-6 && e.1 < 2.1e-6);
        m.vertices.push(Vertex {
            p: [0., 0., 0.1],
            n: [0., 0., 1.2],
        });
        let peak = [[0, 1, 4, 0], [1, 3, 4, 0], [3, 2, 4, 0], [2, 0, 4, 0]]
            .map(|t| Tri::new(&m, t, frame));
        let (e, n) = certify(&peak, &other).unwrap();
        assert!((0.1..0.10001).contains(&e));
        assert!((0.2..0.20001).contains(&n));
        assert!(certify(&peak, &other[..1]).is_none());
        assert!(!nonoverlap(&[a[0].clone(), a[0].clone()]));
    }
    #[test]
    fn bounds_and_binary_roundtrip() -> Result<()> {
        let a = build(grid(12), 64)?;
        assert!(a.patches.iter().any(|p| p.levels.len() > 1));
        let b = encode(&a);
        let loaded = decode(&b)?;
        assert_eq!(a.source_count, loaded.source_count);
        for i in [0, 12, 30, b.len() - 1] {
            assert!(decode(&b[..i]).is_err());
        }
        let mut broken = b;
        broken[45] ^= 1;
        assert!(decode(&broken).is_err());
        Ok(())
    }
    #[test]
    fn mixed_regions_keep_all_boundaries() -> Result<()> {
        let a = build(grid(16), 64)?;
        let mut edges = HashMap::<([u32; 3], [u32; 3]), usize>::new();
        for (i, p) in a.patches.iter().enumerate() {
            let l = if i % 2 == 0 {
                p.levels.last().unwrap()
            } else {
                &p.levels[0]
            };
            for t in &a.mesh.faces[l.start as usize / 3..(l.start + l.count) as usize / 3] {
                for j in 0..3 {
                    let v = a.mesh.vertices[t[j] as usize].p.map(f32::to_bits);
                    let w = a.mesh.vertices[t[(j + 1) % 3] as usize].p.map(f32::to_bits);
                    *edges
                        .entry(if v < w { (v, w) } else { (w, v) })
                        .or_default() += 1;
                }
            }
        }
        for ((a, b), count) in edges {
            assert!(count <= 2);
            if count == 1 {
                let a = a.map(f32::from_bits);
                let b = b.map(f32::from_bits);
                assert!(
                    (0..2).any(|i| a[i].abs() == 1. && a[i] == b[i]),
                    "internal seam {a:?} {b:?}"
                );
            }
        }
        Ok(())
    }
    #[test]
    fn selector_preserves_native_and_uses_physical_resolution() -> Result<()> {
        let a = build(grid(10), 128)?;
        let pose = super::super::render::Pose {
            angle: 0.4,
            distance: 4.,
            light: 0.,
        };
        let native = super::super::select::choose(&a, pose, 2234, 0., false);
        assert_eq!(native.ranges, vec![(0, a.source_count)]);
        let coarse = super::super::select::choose(&a, pose, 2234, 0.15, false);
        assert!(coarse.triangles < a.source_count / 3);
        let fine = super::super::select::choose(&a, pose, 22340000, 0.15, false);
        assert!(fine.triangles >= coarse.triangles);
        Ok(())
    }
    #[test]
    fn coordinated_unions_release_internal_seams() -> Result<()> {
        let source = grid(32);
        let old = build(source.clone(), 32)?;
        let new = build_coordinated(source, 32)?;
        let minimum = |a: &Asset| {
            a.patches
                .iter()
                .map(|p| p.levels.last().unwrap().count)
                .sum::<u32>()
        };
        assert!(new.patches.len() > 1);
        assert!(new.patches.len() < old.patches.len());
        assert!(minimum(&new) < minimum(&old));
        assert_eq!(new.source_count, old.source_count);
        let mut edges = HashMap::new();
        for (i, p) in new.patches.iter().enumerate() {
            let l = if i % 2 == 0 {
                p.levels.last().unwrap()
            } else {
                &p.levels[0]
            };
            for t in &new.mesh.faces[l.start as usize / 3..(l.start + l.count) as usize / 3] {
                for j in 0..3 {
                    let a = new.mesh.vertices[t[j] as usize].p.map(f32::to_bits);
                    let b = new.mesh.vertices[t[(j + 1) % 3] as usize]
                        .p
                        .map(f32::to_bits);
                    *edges
                        .entry(if a < b { (a, b) } else { (b, a) })
                        .or_insert(0) += 1;
                }
            }
        }
        for ((a, b), n) in edges {
            assert!(n <= 2);
            if n == 1 {
                let a = a.map(f32::from_bits);
                let b = b.map(f32::from_bits);
                assert!(
                    (0..2).any(|i| a[i].abs() == 1. && a[i] == b[i]),
                    "internal crack"
                );
            }
        }
        Ok(())
    }
}
