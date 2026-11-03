//! Recognize a rectangular height field by geometry/topology, never subdivision history.
//! Certify the vertical distance on the exact overlay of both piecewise-linear surfaces.
use super::Level;
use crate::meshes::Geometry;
use glam::DVec3;
use std::collections::BTreeSet;
pub(super) fn compile(g: &Geometry) -> Option<Vec<Level>> {
    compile_kind(g, false)
}
pub(super) fn compile_with_planar(g: &Geometry) -> Option<Vec<Level>> {
    compile_kind(g, true)
}
fn compile_kind(g: &Geometry, planar: bool) -> Option<Vec<Level>> {
    let coordinates = |axis: usize| {
        let mut v: Vec<_> = g.positions.iter().map(|p| p[axis]).collect();
        v.sort_by(f32::total_cmp);
        v.dedup();
        v
    };
    let xs = coordinates(0);
    let zs = coordinates(2);
    if xs.len() < 3 || zs.len() < 3 || xs.len().checked_mul(zs.len())? != g.positions.len() {
        return None;
    }
    let (nx, nz) = (xs.len(), zs.len());
    let mut grid = vec![usize::MAX; nx * nz];
    for (i, p) in g.positions.iter().enumerate() {
        let x = xs
            .binary_search_by(|x| x.partial_cmp(&p[0]).unwrap())
            .ok()?;
        let z = zs
            .binary_search_by(|z| z.partial_cmp(&p[2]).unwrap())
            .ok()?;
        if grid[z * nx + x] != usize::MAX {
            return None;
        }
        grid[z * nx + x] = i;
    }
    if g.triangles.len() != 2 * (nx - 1) * (nz - 1) {
        return None;
    }
    let sorted = |mut t: [u32; 3]| {
        t.sort_unstable();
        t
    };
    let triangles: BTreeSet<_> = g.triangles.iter().copied().map(sorted).collect();
    for z in 0..nz - 1 {
        for x in 0..nx - 1 {
            let [a, b, c, d] = [
                grid[z * nx + x],
                grid[z * nx + x + 1],
                grid[(z + 1) * nx + x],
                grid[(z + 1) * nx + x + 1],
            ]
            .map(|i| i as u32);
            if !triangles.contains(&sorted([a, c, b])) || !triangles.contains(&sorted([b, c, d])) {
                return None;
            }
        }
    }
    let mut out = Vec::new();
    if planar {
        let exact = planar_regions(g, &grid, nx, nz);
        // Do not ship almost a second copy of a curved mesh to save a handful
        // of triangles. Require the added packed level to pay for itself in
        // selected GPU payload for even one instance (24B vertices/u32 indices).
        let source_bytes = g.positions.len() * 24 + g.triangles.len() * 12;
        let selected_bytes = exact.positions.len() * 24 + exact.triangles.len() * 12;
        let packed_bytes = 16 + exact.positions.len() * 12 + exact.triangles.len() * 6;
        if packed_bytes <= source_bytes.saturating_sub(selected_bytes) {
            out.push(exact);
        }
    }
    for step in [2usize, 4, 8, 16, 32] {
        let indices = |n: usize| {
            let mut v: Vec<_> = (0..n - 1).step_by(step).collect();
            v.push(n - 1);
            v
        };
        let ix = indices(nx);
        let iz = indices(nz);
        let mut positions = Vec::with_capacity(ix.len() * iz.len());
        for &z in &iz {
            for &x in &ix {
                positions.push(g.positions[grid[z * nx + x]]);
            }
        }
        let mut faces = Vec::new();
        let mut error = 0f64;
        for cz in 0..iz.len() - 1 {
            for cx in 0..ix.len() - 1 {
                let a = (cz * ix.len() + cx) as u16;
                let b = a + 1;
                let c = a + ix.len() as u16;
                let d = c + 1;
                faces.extend([[a, c, b], [b, c, d]]);
                let [pa, pb, pc, pd] =
                    [a, b, c, d].map(|i| DVec3::from_array(positions[i as usize].map(f64::from)));
                let coords =
                    |p: DVec3| ((p.x - pa.x) / (pb.x - pa.x), (p.z - pa.z) / (pc.z - pa.z));
                let side = |p: DVec3| {
                    let (u, v) = coords(p);
                    u + v - 1.
                };
                let coarse = |p: DVec3| {
                    let (u, v) = coords(p);
                    if u + v <= 1. {
                        pa.y + (pb.y - pa.y) * u + (pc.y - pa.y) * v
                    } else {
                        pd.y + (pc.y - pd.y) * (1. - u) + (pb.y - pd.y) * (1. - v)
                    }
                };
                for z in iz[cz]..iz[cz + 1] {
                    for x in ix[cx]..ix[cx + 1] {
                        let [a, b, c, d] = [
                            grid[z * nx + x],
                            grid[z * nx + x + 1],
                            grid[(z + 1) * nx + x],
                            grid[(z + 1) * nx + x + 1],
                        ];
                        for t in [[a, c, b], [b, c, d]] {
                            let p = t.map(|i| DVec3::from_array(g.positions[i].map(f64::from)));
                            for v in p {
                                error = error.max((v.y - coarse(v)).abs());
                            }
                            // The only new overlay vertices are intersections of a fine edge
                            // with the coarse diagonal. Coarse cell borders coincide with grid edges.
                            for (a, b) in [(p[0], p[1]), (p[1], p[2]), (p[2], p[0])] {
                                let (sa, sb) = (side(a), side(b));
                                if sa * sb < 0. {
                                    let v = a.lerp(b, sa / (sa - sb));
                                    error = error.max((v.y - coarse(v)).abs());
                                }
                            }
                        }
                    }
                }
            }
        }
        let magnitude = g.positions.iter().map(|p| p[1].abs()).fold(1., f32::max);
        out.push(Level {
            positions,
            triangles: faces,
            error: (error as f32 + 16. * f32::EPSILON * magnitude).next_up(),
            vertical: true,
        });
    }
    Some(out)
}

// Re-triangulate ONLY exactly horizontal source patches. All perimeter samples stay,
// so adjacent curved/full patches share every edge and the covered surface is unchanged.
fn planar_regions(g: &Geometry, grid: &[usize], nx: usize, nz: usize) -> Level {
    let mut faces: Vec<[u16; 3]> = Vec::new();
    for z0 in (0..nz - 1).step_by(4) {
        for x0 in (0..nx - 1).step_by(4) {
            let x1 = (x0 + 4).min(nx - 1);
            let z1 = (z0 + 4).min(nz - 1);
            let id = |x: usize, z: usize| grid[z * nx + x] as u16;
            let h = g.positions[id(x0, z0) as usize][1];
            let flat = x1 - x0 >= 3
                && z1 - z0 >= 3
                && (z0..=z1).all(|z| (x0..=x1).all(|x| g.positions[id(x, z) as usize][1] == h));
            if flat {
                let mut edge = Vec::new();
                for z in z0..z1 {
                    edge.push(id(x0, z));
                }
                for x in x0..x1 {
                    edge.push(id(x, z1));
                }
                for z in (z0 + 1..=z1).rev() {
                    edge.push(id(x1, z));
                }
                for x in (x0 + 1..=x1).rev() {
                    edge.push(id(x, z0));
                }
                let center = id((x0 + x1) / 2, (z0 + z1) / 2);
                for i in 0..edge.len() {
                    faces.push([center, edge[i], edge[(i + 1) % edge.len()]]);
                }
            } else {
                for z in z0..z1 {
                    for x in x0..x1 {
                        let [a, b, c, d] = [id(x, z), id(x + 1, z), id(x, z + 1), id(x + 1, z + 1)];
                        faces.extend([[a, c, b], [b, c, d]]);
                    }
                }
            }
        }
    }
    let mut positions = Vec::new();
    let mut mapping = vec![u16::MAX; g.positions.len()];
    for t in &mut faces {
        for i in t {
            let old = *i as usize;
            if mapping[old] == u16::MAX {
                mapping[old] = positions.len() as u16;
                positions.push(g.positions[old]);
            }
            *i = mapping[old];
        }
    }
    Level {
        positions,
        triangles: faces,
        error: 0.,
        vertical: true,
    }
}
