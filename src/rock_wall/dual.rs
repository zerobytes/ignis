//! Uniform-grid Dual Contouring experiment, Hermite intersections + bounded QEF.
//! Positive density is solid. No octree simplification or manifold guarantee.
use crate::model::{Result, invalid};
use glam::DVec3;

const C: [[usize; 3]; 8] = [
    [0, 0, 0],
    [1, 0, 0],
    [1, 1, 0],
    [0, 1, 0],
    [0, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
    [0, 1, 1],
];
const E: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

// Solve a small SPD system with pivoting; unused dimensions are identity rows.
fn solve(mut a: [[f64; 4]; 3]) -> Option<DVec3> {
    for i in 0..3 {
        let k = (i..3).max_by(|&j, &k| a[j][i].abs().total_cmp(&a[k][i].abs()))?;
        a.swap(i, k);
        let d = a[i][i];
        if d.abs() < 1e-14 {
            return None;
        }
        for value in &mut a[i][i..] {
            *value /= d;
        }
        let pivot = a[i];
        for (k, row) in a.iter_mut().enumerate() {
            if k != i {
                let s = row[i];
                for (value, &source) in row[i..].iter_mut().zip(&pivot[i..]) {
                    *value -= s * source;
                }
            }
        }
    }
    Some(DVec3::new(a[0][3], a[1][3], a[2][3]))
}

// Active-set box-constrained least squares. Work in cell-relative metres, with
// tiny mass-point regularization for underdetermined planar/edge cells.
fn vertex(h: &[(DVec3, DVec3)], lo: DVec3, size: f64) -> DVec3 {
    let mass = h.iter().map(|(p, _)| *p - lo).sum::<DVec3>() / h.len() as f64;
    let mut ata = [[0.; 3]; 3];
    let mut atb = [0.; 3];
    for &(p, n) in h {
        let b = n.dot(p - lo);
        for i in 0..3 {
            atb[i] += n[i] * b;
            for j in 0..3 {
                ata[i][j] += n[i] * n[j];
            }
        }
    }
    let reg = 1e-8 * h.len() as f64;
    for i in 0..3 {
        ata[i][i] += reg;
        atb[i] += reg * mass[i];
    }
    let mut best = mass.clamp(DVec3::ZERO, DVec3::splat(size));
    let mut error = f64::INFINITY;
    for code in 0..27 {
        let mut c = code;
        let mut free = [false; 3];
        let mut fixed = DVec3::ZERO;
        for i in 0..3 {
            let mode = c % 3;
            c /= 3;
            free[i] = mode == 0;
            fixed[i] = if mode == 2 { size } else { 0. };
        }
        let mut a = [[0.; 4]; 3];
        for i in 0..3 {
            if free[i] {
                a[i][3] = atb[i];
                for j in 0..3 {
                    if free[j] {
                        a[i][j] = ata[i][j];
                    } else {
                        a[i][3] -= ata[i][j] * fixed[j];
                    }
                }
            } else {
                a[i][i] = 1.;
                a[i][3] = fixed[i];
            }
        }
        let Some(p) = solve(a) else { continue };
        if !p.is_finite() || p.min_element() < -1e-9 || p.max_element() > size + 1e-9 {
            continue;
        }
        let p = p.clamp(DVec3::ZERO, DVec3::splat(size));
        let e = h
            .iter()
            .map(|(q, n)| n.dot(p - (*q - lo)).powi(2))
            .sum::<f64>()
            + reg * p.distance_squared(mass);
        if e < error {
            error = e;
            best = p;
        }
    }
    lo + best
}

pub(super) fn extract(
    field: impl Fn(DVec3) -> (f64, DVec3),
    origin: DVec3,
    cells: [usize; 3],
    spacing: f64,
) -> Result<(crate::meshes::Geometry, serde_json::Value)> {
    let n = cells.map(|v| v + 1);
    let count = n.iter().product::<usize>();
    if count > crate::volume_terrain::MAX_POINTS {
        return Err(invalid("Dual contour sample budget exceeded"));
    }
    let index = |p: [usize; 3]| (p[2] * n[1] + p[1]) * n[0] + p[0];
    let world =
        |p: [usize; 3]| origin + DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64) * spacing;
    let mut values = Vec::with_capacity(count);
    for z in 0..n[2] {
        for y in 0..n[1] {
            for x in 0..n[0] {
                let v = field(world([x, y, z])).0;
                if !v.is_finite() {
                    return Err(invalid("Nonfinite density"));
                }
                values.push(v);
            }
        }
    }
    // Cache each lattice edge's Hermite data once, even when four cells share it.
    let mut edges = vec![[u32::MAX; 3]; count];
    let mut edge_samples = Vec::new();
    let mut edge_count = 0;
    for z in 0..n[2] {
        for y in 0..n[1] {
            for x in 0..n[0] {
                let a = [x, y, z];
                for axis in 0..3 {
                    if a[axis] == cells[axis] {
                        continue;
                    }
                    let mut b = a;
                    b[axis] += 1;
                    if (values[index(a)] > 0.) == (values[index(b)] > 0.) {
                        continue;
                    }
                    let mut low = world(a);
                    let mut high = world(b);
                    let sign = values[index(a)] > 0.;
                    for _ in 0..28 {
                        let mid = (low + high) * 0.5;
                        if (field(mid).0 > 0.) == sign {
                            low = mid;
                        } else {
                            high = mid;
                        }
                    }
                    let p = (low + high) * 0.5;
                    let normal = field(p).1;
                    if !normal.is_finite() || normal.length_squared() < 1e-20 {
                        return Err(invalid("Undefined Hermite normal"));
                    }
                    edges[index(a)][axis] = edge_samples.len() as u32;
                    edge_samples.push((p, normal.normalize()));
                    edge_count += 1;
                }
            }
        }
    }
    let mut ids = vec![u32::MAX; count];
    let mut positions = vec![];
    let mut hermite = Vec::with_capacity(12);
    for z in 0..cells[2] {
        for y in 0..cells[1] {
            for x in 0..cells[0] {
                let cell = [x, y, z];
                hermite.clear();
                for [a, b] in E {
                    let pa: [usize; 3] = std::array::from_fn(|i| cell[i] + C[a][i]);
                    let pb: [usize; 3] = std::array::from_fn(|i| cell[i] + C[b][i]);
                    let axis = (0..3).find(|&i| pa[i] != pb[i]).unwrap();
                    let start = std::array::from_fn(|i| pa[i].min(pb[i]));
                    let edge = edges[index(start)][axis];
                    if edge != u32::MAX {
                        hermite.push(edge_samples[edge as usize]);
                    }
                }
                if hermite.is_empty() {
                    continue;
                }
                if positions.len() >= crate::volume_terrain::MAX_VERTICES {
                    return Err(invalid("Dual contour vertex budget exceeded"));
                }
                let p = vertex(&hermite, world(cell), spacing);
                ids[index(cell)] = positions.len() as u32;
                positions.push(p.to_array().map(|v| v as f32));
            }
        }
    }
    let mut triangles = vec![];
    let mut degenerate = 0;
    for z in 0..n[2] {
        for y in 0..n[1] {
            for x in 0..n[0] {
                let p = [x, y, z];
                for (axis, &edge) in edges[index(p)].iter().enumerate() {
                    if edge == u32::MAX {
                        continue;
                    }
                    let u = (axis + 1) % 3;
                    let v = (axis + 2) % 3;
                    if p[u] == 0 || p[v] == 0 || p[u] >= cells[u] || p[v] >= cells[v] {
                        return Err(invalid(
                            "Surface touches dual contour boundary; pad the grid",
                        ));
                    }
                    let mut a = p;
                    a[u] -= 1;
                    a[v] -= 1;
                    let mut b = a;
                    b[u] += 1;
                    let mut c = b;
                    c[v] += 1;
                    let mut d = a;
                    d[v] += 1;
                    let mut quad = [a, b, c, d].map(|q| ids[index(q)]);
                    if quad.contains(&u32::MAX) {
                        return Err(invalid("Missing dual contour cell"));
                    }
                    // u cross v points +axis. Positive-to-negative along edge is outward.
                    if values[index(p)] <= 0. {
                        quad.reverse();
                    }
                    // Prefer shorter diagonal; field topology is unchanged by triangulation.
                    let q = quad.map(|i| DVec3::from_array(positions[i as usize].map(f64::from)));
                    let ts = if q[0].distance_squared(q[2]) <= q[1].distance_squared(q[3]) {
                        [[quad[0], quad[1], quad[2]], [quad[0], quad[2], quad[3]]]
                    } else {
                        [[quad[0], quad[1], quad[3]], [quad[1], quad[2], quad[3]]]
                    };
                    for t in ts {
                        let ps = t.map(|i| DVec3::from_array(positions[i as usize].map(f64::from)));
                        if (ps[1] - ps[0]).cross(ps[2] - ps[0]).length_squared() < 1e-20 {
                            degenerate += 1;
                            continue;
                        }
                        triangles.push(t);
                    }
                    if triangles.len() > crate::volume_terrain::MAX_TRIANGLES {
                        return Err(invalid("Dual contour triangle budget exceeded"));
                    }
                }
            }
        }
    }
    let report = serde_json::json!({"extraction":"dual_contouring_uniform_qef","samples":count,"hermite_edges":edge_count,"vertices":positions.len(),"triangles":triangles.len(),"degenerate_triangles_omitted":degenerate,"spacing_meters":spacing,"octree":false,"manifold_guarantee":false,"qef":"box_constrained_regularized_active_set","grid_cells":cells});
    let geometry = crate::meshes::Geometry {
        positions,
        triangles,
        material_coordinates: None,
    };
    geometry.validate()?;
    Ok((geometry, report))
}
