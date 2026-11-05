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
    pub bytes: u64,
    pub build_ms: f64,
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
    fs::create_dir_all(ROOT)?;
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
            let normals = source
                .iter()
                .map(|v| [v.n[0], v.n[1], v.n[2], 0.])
                .collect::<Vec<_>>();
            let root = Path::new(ROOT);
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
