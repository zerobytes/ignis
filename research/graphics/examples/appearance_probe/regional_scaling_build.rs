use super::*;
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct V {
    pub p: [f32; 3],
    pub n: [f32; 3],
}
#[derive(Serialize, Deserialize)]
pub(super) struct Level {
    pub first: u32,
    pub count: u32,
    pub position_error: f32,
    pub normal_error: f32,
    pub stride: u32,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Asset {
    pub id: String,
    pub family: usize,
    pub factor: u32,
    pub grid: [u32; 2],
    pub levels: Vec<Level>,
    #[serde(default)]
    pub sealed: Vec<Level>,
    #[serde(default)]
    pub regional_floor: u32,
    pub regions: Vec<Region>,
    pub bounds: [[f32; 3]; 2],
    pub bytes: u64,
    pub build_ms: f64,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Region {
    #[serde(default)]
    pub floor: u32,
    pub bounds: [[f32; 3]; 2],
    pub levels: Vec<Level>,
}
pub(super) const GRIDS: [[u32; 2]; 4] = [[128, 128], [128, 256], [256, 640], [1280, 1280]];
pub(super) fn vertex(u: f32, v: f32, family: usize, density: usize) -> V {
    let (mut h, mut du, mut dv) = (0., 0., 0.);
    if family == 1 {
        let bands = [
            (0.55, 1., 1.),
            (0.09, 12., 10.),
            (0.012, 40., 36.),
            (0.002, 128., 120.),
            (0.00025, 320., 300.),
        ];
        for &(amp, fu, fv) in &bands[..density + 2] {
            let a = fu * std::f32::consts::PI;
            let b = fv * std::f32::consts::PI;
            h += amp * (a * u).sin() * (b * v).sin();
            du += amp * a * (a * u).cos() * (b * v).sin();
            dv += amp * b * (a * u).sin() * (b * v).cos();
        }
    }
    V {
        p: [(u - 0.5) * 12., h, (v - 0.5) * 12.],
        n: Vec3::new(-du / 12., 1., -dv / 12.).normalize().to_array(),
    }
}
// Nested diagonal triangulation, not bilinear interpolation across a saddle.
pub(super) fn interpolate(source: &[V], grid: [u32; 2], stride: u32, x: u32, y: u32) -> V {
    let [nx, ny] = grid;
    let ax = (x / stride).min(nx / stride - 1) * stride;
    let ay = (y / stride).min(ny / stride - 1) * stride;
    let u = (x - ax) as f32 / stride as f32;
    let v = (y - ay) as f32 / stride as f32;
    let ids = if u + v <= 1. {
        [
            (ax, ay, 1. - u - v),
            (ax + stride, ay, u),
            (ax, ay + stride, v),
        ]
    } else {
        [
            (ax + stride, ay + stride, u + v - 1.),
            (ax, ay + stride, 1. - u),
            (ax + stride, ay, 1. - v),
        ]
    };
    let mut p = Vec3::ZERO;
    let mut n = Vec3::ZERO;
    for (ix, iy, w) in ids {
        let a = source[(iy * (nx + 1) + ix) as usize];
        p += Vec3::from_array(a.p) * w;
        n += Vec3::from_array(a.n) * w;
    }
    V {
        p: p.to_array(),
        n: n.to_array(),
    }
}
pub(super) fn bake() -> Result<()> {
    bake_at(ROOT, false)
}
pub(super) fn bake_at(output: &str, hybrid: bool) -> Result<()> {
    fs::create_dir_all(output)?;
    for family in 0..2 {
        for (density, grid) in GRIDS.into_iter().enumerate() {
            let start = Instant::now();
            let [nx, ny] = grid;
            let id = format!("f{family}_d{density}");
            let source = (0..=ny)
                .flat_map(|y| {
                    (0..=nx).map(move |x| {
                        vertex(x as f32 / nx as f32, y as f32 / ny as f32, family, density)
                    })
                })
                .collect::<Vec<_>>();
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            let mut levels = Vec::new();
            for power in (0..=7).rev() {
                let stride = 1 << power;
                if nx / stride < 1 || ny / stride < 1 {
                    continue;
                }
                let sx = nx / stride;
                let sy = ny / stride;
                let base = vertices.len() as u32;
                for y in 0..=sy {
                    for x in 0..=sx {
                        vertices.push(source[((y * stride) * (nx + 1) + x * stride) as usize]);
                    }
                }
                let first = indices.len() as u32;
                for y in 0..sy {
                    for x in 0..sx {
                        let a = base + y * (sx + 1) + x;
                        indices.extend([a, a + sx + 1, a + 1, a + 1, a + sx + 1, a + sx + 2]);
                    }
                }
                let (mut pe, mut ne) = (0f32, 0f32);
                if stride > 1 {
                    for y in 0..=ny {
                        for x in 0..=nx {
                            let a = source[(y * (nx + 1) + x) as usize];
                            let b = interpolate(&source, grid, stride, x, y);
                            pe = pe.max((a.p[1] - b.p[1]).abs());
                            ne = ne.max(Vec3::from_array(a.n).distance(Vec3::from_array(b.n)));
                        }
                    }
                }
                levels.push(Level {
                    first,
                    count: indices.len() as u32 - first,
                    position_error: pe,
                    normal_error: ne,
                    stride,
                });
            }
            let bounds = bounds(&source);
            let source_base = vertices.len() as u32;
            let regions = regions(&source, grid, &mut vertices, &mut indices);
            let regional_floor = regions
                .iter()
                .map(|r| r.levels.iter().map(|l| l.count / 3).min().unwrap())
                .sum();
            let sealed = if hybrid {
                partition(
                    &source,
                    grid,
                    &mut vertices,
                    &mut indices,
                    1,
                    Some(source_base),
                )
                .remove(0)
                .levels
            } else {
                Vec::new()
            };
            let normals = source
                .iter()
                .map(|v| [v.n[0], v.n[1], v.n[2], 0.])
                .collect::<Vec<_>>();
            let root = Path::new(output);
            fs::write(
                root.join(format!("{id}.vertices")),
                bytemuck::cast_slice(&vertices),
            )?;
            fs::write(
                root.join(format!("{id}.indices")),
                bytemuck::cast_slice(&indices),
            )?;
            fs::write(
                root.join(format!("{id}.normals")),
                bytemuck::cast_slice(&normals),
            )?;
            let asset = Asset {
                id: id.clone(),
                family,
                factor: [1, 2, 10, 100][density],
                grid,
                levels,
                regions,
                sealed,
                regional_floor,
                bounds,
                bytes: (vertices.len() * 24 + indices.len() * 4 + normals.len() * 16) as u64,
                build_ms: start.elapsed().as_secs_f64() * 1000.,
            };
            fs::write(
                root.join(format!("{id}.json")),
                serde_json::to_vec_pretty(&asset)?,
            )?;
            println!(
                "baked {id}: {} triangles, {:.1}MiB, {:.1}ms",
                nx * ny * 2,
                asset.bytes as f64 / 1048576.,
                asset.build_ms
            );
        }
    }
    Ok(())
}

fn bounds(v: &[V]) -> [[f32; 3]; 2] {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for v in v {
        lo = lo.min(Vec3::from_array(v.p));
        hi = hi.max(Vec3::from_array(v.p));
    }
    [lo.to_array(), hi.to_array()]
}
// All patch boundaries retain the identical source polyline. A boundary fan
// differs from its original coarse triangle by at most its boundary correction.
// Use signed intervals per original triangle: (source-coarse) - (fan-coarse).
// Extrema of the first occur at nested source vertices, of the second at fan
// vertices. This is at most 2*coarse_error, tighter than doubling a patch maximum.
fn regions(
    source: &[V],
    grid: [u32; 2],
    vertices: &mut Vec<V>,
    indices: &mut Vec<u32>,
) -> Vec<Region> {
    partition(source, grid, vertices, indices, 8, None)
}
fn partition(
    source: &[V],
    grid: [u32; 2],
    vertices: &mut Vec<V>,
    indices: &mut Vec<u32>,
    divisions: u32,
    reuse: Option<u32>,
) -> Vec<Region> {
    let [nx, ny] = grid;
    let (w, h) = (nx / divisions, ny / divisions);
    let index_base = indices.len();
    let source_base = reuse.unwrap_or(vertices.len() as u32);
    if reuse.is_none() {
        vertices.extend_from_slice(source);
    }
    let index = |x: u32, y: u32| source_base + y * (nx + 1) + x;
    let mut result = Vec::new();
    for py in 0..divisions {
        for px in 0..divisions {
            let (x0, y0) = (px * w, py * h);
            let (x1, y1) = (x0 + w, y0 + h);
            let mut local = Vec::new();
            for y in y0..=y1 {
                for x in x0..=x1 {
                    local.push(source[(y * (nx + 1) + x) as usize]);
                }
            }
            let mut levels = Vec::new();
            for power in (0..=7).rev() {
                let s = 1 << power;
                if w % s != 0 || h % s != 0 {
                    continue;
                }
                let first = indices.len() as u32;
                let mut error = 0f32;
                for y in (y0..y1).step_by(s as usize) {
                    for x in (x0..x1).step_by(s as usize) {
                        for (half, tri) in [
                            [(x, y), (x, y + s), (x + s, y)],
                            [(x + s, y), (x, y + s), (x + s, y + s)],
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            let mut polygon = Vec::with_capacity((s * 2 + 3) as usize);
                            for k in 0..3 {
                                let (ax, ay) = tri[k];
                                let (bx, by) = tri[(k + 1) % 3];
                                polygon.push(index(ax, ay));
                                let boundary = (ax == bx && (ax == x0 || ax == x1))
                                    || (ay == by && (ay == y0 || ay == y1));
                                if boundary {
                                    for t in 1..s {
                                        let ix = (ax as i32
                                            + (bx as i32 - ax as i32) * t as i32 / s as i32)
                                            as u32;
                                        let iy = (ay as i32
                                            + (by as i32 - ay as i32) * t as i32 / s as i32)
                                            as u32;
                                        polygon.push(index(ix, iy));
                                    }
                                }
                            }
                            if s > 1 {
                                let deviation = |ix, iy| {
                                    source[(iy * (nx + 1) + ix) as usize].p[1]
                                        - interpolate(source, grid, s, ix, iy).p[1]
                                };
                                let (mut low, mut high) = (0f32, 0f32);
                                for iy in y..=y + s {
                                    for ix in x..=x + s {
                                        let sum = ix - x + iy - y;
                                        if (half == 0 && sum <= s) || (half == 1 && sum >= s) {
                                            let e = deviation(ix, iy);
                                            low = low.min(e);
                                            high = high.max(e);
                                        }
                                    }
                                }
                                let (mut cmin, mut cmax) = (0f32, 0f32);
                                for &v in &polygon {
                                    let k = v - source_base;
                                    let e = deviation(k % (nx + 1), k / (nx + 1));
                                    cmin = cmin.min(e);
                                    cmax = cmax.max(e);
                                }
                                error = error.max((low - cmax).abs()).max((high - cmin).abs());
                            }
                            if polygon.len() == 3 {
                                indices.extend(&polygon);
                            } else {
                                let mut c = V {
                                    p: [0.; 3],
                                    n: [0.; 3],
                                };
                                for (tx, ty) in tri {
                                    let v = source[(ty * (nx + 1) + tx) as usize];
                                    for j in 0..3 {
                                        c.p[j] += v.p[j] / 3.;
                                        c.n[j] += v.n[j] / 3.;
                                    }
                                }
                                let center = vertices.len() as u32;
                                vertices.push(c);
                                for k in 0..polygon.len() {
                                    indices.extend([
                                        center,
                                        polygon[k],
                                        polygon[(k + 1) % polygon.len()],
                                    ]);
                                }
                            }
                        }
                    }
                }
                levels.push(Level {
                    first,
                    count: indices.len() as u32 - first,
                    position_error: error,
                    normal_error: 0.,
                    stride: s,
                });
            }
            result.push(Region {
                floor: levels.iter().map(|l| l.count / 3).min().unwrap(),
                bounds: bounds(&local),
                levels,
            });
        }
    }
    let old = indices[index_base..].to_vec();
    indices.truncate(index_base);
    for level in 0..result[0].levels.len() {
        for region in &mut result {
            let l = &mut region.levels[level];
            let start = l.first as usize - index_base;
            l.first = indices.len() as u32;
            indices.extend_from_slice(&old[start..start + l.count as usize]);
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn every_level_covers_patch_and_preserves_source_boundary() {
        let grid = [32, 64];
        let source = (0..=64)
            .flat_map(|y| (0..=32).map(move |x| vertex(x as f32 / 32., y as f32 / 64., 1, 3)))
            .collect::<Vec<_>>();
        for (divisions, reuse) in [(1, false), (8, false), (1, true)] {
            let mut vertices = if reuse { source.clone() } else { Vec::new() };
            let mut indices = Vec::new();
            let regions = partition(
                &source,
                grid,
                &mut vertices,
                &mut indices,
                divisions,
                if reuse { Some(0) } else { None },
            );
            assert_eq!(regions.len(), (divisions * divisions) as usize);
            for r in &regions {
                for level in &r.levels {
                    let mut edges = BTreeMap::new();
                    let mut area = 0f32;
                    for t in indices[level.first as usize..(level.first + level.count) as usize]
                        .chunks_exact(3)
                    {
                        let [a, b, c] = [
                            vertices[t[0] as usize],
                            vertices[t[1] as usize],
                            vertices[t[2] as usize],
                        ];
                        let det = (b.p[0] - a.p[0]) * (c.p[2] - a.p[2])
                            - (b.p[2] - a.p[2]) * (c.p[0] - a.p[0]);
                        assert!(det < 0.);
                        for i in 0..=5 {
                            for j in 0..=5 - i {
                                let u = i as f32 / 5.;
                                let v = j as f32 / 5.;
                                let p = Vec3::from_array(a.p) * (1. - u - v)
                                    + Vec3::from_array(b.p) * u
                                    + Vec3::from_array(c.p) * v;
                                let gx = ((p.x / 12. + 0.5) * 32.).clamp(0., 32.);
                                let gy = ((p.z / 12. + 0.5) * 64.).clamp(0., 64.);
                                let ix = (gx.floor() as u32).min(31);
                                let iy = (gy.floor() as u32).min(63);
                                let (fx, fy) = (gx - ix as f32, gy - iy as f32);
                                let h = |x, y| source[(y * 33 + x) as usize].p[1];
                                let height = if fx + fy <= 1. {
                                    h(ix, iy) * (1. - fx - fy)
                                        + h(ix + 1, iy) * fx
                                        + h(ix, iy + 1) * fy
                                } else {
                                    h(ix + 1, iy + 1) * (fx + fy - 1.)
                                        + h(ix, iy + 1) * (1. - fx)
                                        + h(ix + 1, iy) * (1. - fy)
                                };
                                assert!((height - p.y).abs() <= level.position_error + 0.00001);
                            }
                        }
                        area += det.abs() * 0.5;
                        for k in 0..3 {
                            let (a, b) = (t[k], t[(k + 1) % 3]);
                            *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                        }
                    }
                    assert!((area - 144. / (divisions * divisions) as f32).abs() < 0.0001);
                    let mut boundary = 0;
                    for ((a, b), count) in edges {
                        assert!(count == 1 || count == 2);
                        if count == 1 {
                            boundary += 1;
                            assert!((a as usize) < source.len() && (b as usize) < source.len());
                            assert!(a.abs_diff(b) == 1 || a.abs_diff(b) == 33);
                            let (a, b) = (vertices[a as usize], vertices[b as usize]);
                            assert!(
                                (a.p[0] == b.p[0]
                                    && (a.p[0] == r.bounds[0][0] || a.p[0] == r.bounds[1][0]))
                                    || (a.p[2] == b.p[2]
                                        && (a.p[2] == r.bounds[0][2] || a.p[2] == r.bounds[1][2]))
                            );
                        }
                    }
                    assert_eq!(boundary, 2 * (32 + 64) / divisions);
                    assert!(level.position_error.is_finite());
                }
            }
        }
    }
}
