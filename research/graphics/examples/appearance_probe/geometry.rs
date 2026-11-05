use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use std::time::Instant;

pub const TILES: usize = 8;
pub const TILE_COUNT: usize = TILES * TILES;
pub const MAX_BINS: usize = 128;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    pub tile: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct Lobe {
    pub normal_area: [f32; 4],
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Default)]
pub(super) struct Accumulator {
    normal: [f64; 3],
    color: [f64; 3],
    area: f64,
}

impl Accumulator {
    pub(super) fn add(&mut self, normal: Vec3, color: Vec3, area: f64) {
        for i in 0..3 {
            self.normal[i] += f64::from(normal[i]) * area;
            self.color[i] += f64::from(color[i]) * area;
        }
        self.area += area;
    }

    pub(super) fn merge(&mut self, other: Self) {
        for i in 0..3 {
            self.normal[i] += other.normal[i];
            self.color[i] += other.color[i];
        }
        self.area += other.area;
    }

    pub(super) fn finish(self) -> Lobe {
        if self.area == 0.0 {
            return Lobe::default();
        }
        let normal = Vec3::from_array(self.normal.map(|v| (v / self.area) as f32));
        Lobe {
            normal_area: [normal.x, normal.y, normal.z, self.area as f32],
            color: [
                (self.color[0] / self.area) as f32,
                (self.color[1] / self.area) as f32,
                (self.color[2] / self.area) as f32,
                0.0,
            ],
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct TileBounds {
    pub min: Vec3,
    pub max: Vec3,
    pub cell_diameter: f32,
}

pub struct Mesh {
    pub bounds: [TileBounds; TILE_COUNT],
    pub vertices: Vec<Vertex>,
    pub proxy: Vec<Vertex>,
    pub lobes32: Vec<Lobe>,
    pub lobes128: Vec<Lobe>,
    pub generation_ms: f64,
    pub aggregation_ms: f64,
}

fn hash(x: u32, z: u32) -> f32 {
    let mut v = x.wrapping_mul(0x9e3779b9) ^ z.wrapping_mul(0x85ebca6b) ^ 0x1234abcd;
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb352d);
    v ^= v >> 15;
    (v & 0xffff) as f32 / 65535.0
}

pub(super) fn bin(n: Vec3, bins: usize) -> usize {
    let azimuth = (n.z.atan2(n.x) + std::f32::consts::PI) / std::f32::consts::TAU;
    let az = ((azimuth * 8.0) as usize).min(7);
    let levels = bins / 8;
    let elevation = n.y.clamp(0.0, 1.0).acos() / std::f32::consts::FRAC_PI_2;
    az * levels + ((elevation * levels as f32) as usize).min(levels - 1)
}

pub fn generate(n: usize) -> Mesh {
    assert!((TILES..=512).contains(&n) && n.is_multiple_of(TILES));
    let start = Instant::now();
    let mut vertices = Vec::with_capacity(n * n * 12);
    let cell = 2.0 / n as f32;
    let mut bounds = [TileBounds::default(); TILE_COUNT];
    let per_tile = n / TILES;
    let tile_side = 2.0 / TILES as f32;
    for (tile_id, bound) in bounds.iter_mut().enumerate() {
        let tx = tile_id % TILES;
        let tz = tile_id / TILES;
        bound.min = Vec3::new(
            -1.0 + tx as f32 * tile_side,
            0.0,
            -1.0 + tz as f32 * tile_side,
        );
        bound.max = bound.min + Vec3::new(tile_side, 0.0, tile_side);
        for z in tz * per_tile..(tz + 1) * per_tile {
            for x in tx * per_tile..(tx + 1) * per_tile {
                let x0 = -1.0 + x as f32 * cell;
                let z0 = -1.0 + z as f32 * cell;
                let slope = 0.15 + 1.25 * hash(x as u32, z as u32);
                let top = Vec3::new(x0 + cell * 0.5, slope * cell * 0.5, z0 + cell * 0.5);
                bound.max.y = bound.max.y.max(top.y);
                let corners = [
                    Vec3::new(x0, 0.0, z0),
                    Vec3::new(x0, 0.0, z0 + cell),
                    Vec3::new(x0 + cell, 0.0, z0 + cell),
                    Vec3::new(x0 + cell, 0.0, z0),
                ];
                let tile = (z * TILES / n) * TILES + x * TILES / n;
                // A fixed macro pattern plus slope-correlated material tests joint statistics.
                let checker = ((x * TILES / n) + (z * TILES / n)) % 2;
                let base = if checker == 0 {
                    Vec3::new(0.65, 0.22, 0.055)
                } else {
                    Vec3::new(0.07, 0.32, 0.55)
                };
                let color = base * (0.65 + 0.35 * slope / 1.4);
                for face in 0..4 {
                    let points = [corners[face], corners[(face + 1) % 4], top];
                    let normal = (points[1] - points[0])
                        .cross(points[2] - points[0])
                        .normalize();
                    for position in points {
                        vertices.push(Vertex {
                            position: position.to_array(),
                            normal: normal.to_array(),
                            color: color.to_array(),
                            tile: tile as u32,
                        });
                    }
                }
            }
        }
        bound.cell_diameter = (2.0 * cell * cell + bound.max.y * bound.max.y).sqrt();
    }
    let generation_ms = start.elapsed().as_secs_f64() * 1000.0;
    let aggregation_start = Instant::now();
    let mut a32 = vec![Accumulator::default(); TILE_COUNT * 32];
    let mut a128 = vec![Accumulator::default(); TILE_COUNT * MAX_BINS];
    let mut means = vec![Accumulator::default(); TILE_COUNT];
    for tri in vertices.chunks_exact(3) {
        let p = tri.map_positions();
        let area = f64::from((p[1] - p[0]).cross(p[2] - p[0]).length()) * 0.5;
        let normal = Vec3::from_array(tri[0].normal);
        let color = Vec3::from_array(tri[0].color);
        let tile = tri[0].tile as usize;
        a32[tile * 32 + bin(normal, 32)].add(normal, color, area);
        a128[tile * MAX_BINS + bin(normal, MAX_BINS)].add(normal, color, area);
        means[tile].add(normal, color, area);
    }
    let lobes32 = a32.into_iter().map(Accumulator::finish).collect();
    let lobes128 = a128.into_iter().map(Accumulator::finish).collect();
    let mut proxy = Vec::with_capacity(TILE_COUNT * 6);
    for z in 0..TILES {
        for x in 0..TILES {
            let tile = z * TILES + x;
            let m = means[tile].finish();
            let x0 = -1.0 + x as f32 * 2.0 / TILES as f32;
            let z0 = -1.0 + z as f32 * 2.0 / TILES as f32;
            let side = 2.0 / TILES as f32;
            for (dx, dz) in [(0., 0.), (0., 1.), (1., 1.), (0., 0.), (1., 1.), (1., 0.)] {
                proxy.push(Vertex {
                    position: [x0 + dx * side, 0.0, z0 + dz * side],
                    normal: [0., 1., 0.],
                    color: [m.color[0], m.color[1], m.color[2]],
                    tile: tile as u32 | (1 << 31),
                });
            }
        }
    }
    Mesh {
        bounds,
        vertices,
        proxy,
        lobes32,
        lobes128,
        generation_ms,
        aggregation_ms: aggregation_start.elapsed().as_secs_f64() * 1000.0,
    }
}

trait TrianglePositions {
    fn map_positions(&self) -> [Vec3; 3];
}
impl TrianglePositions for [Vertex] {
    fn map_positions(&self) -> [Vec3; 3] {
        [
            Vec3::from_array(self[0].position),
            Vec3::from_array(self[1].position),
            Vec3::from_array(self[2].position),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diffuse_moment_identity_and_normalization_counterexample() {
        let normals = [Vec3::new(0.6, 0.8, 0.0), Vec3::new(-0.6, 0.8, 0.0)];
        let light = Vec3::new(0.1, 1.0, 0.1).normalize();
        let explicit = normals.iter().map(|n| 0.5 * n.dot(light)).sum::<f32>();
        let moment = (normals[0] + normals[1]) * 0.5;
        assert!((explicit - moment.dot(light)).abs() < 1e-6);
        assert!((explicit - moment.normalize().dot(light)).abs() > 0.1);
        let side_light = Vec3::X;
        let explicit_clamped = normals
            .iter()
            .map(|n| 0.5 * n.dot(side_light).max(0.0))
            .sum::<f32>();
        assert!((explicit_clamped - moment.dot(side_light).max(0.0)).abs() > 0.2);
    }
    #[test]
    fn projected_area_and_material_are_conserved() {
        let mesh = generate(16);
        for lobes in [&mesh.lobes32, &mesh.lobes128] {
            // Retained first moment preserves total and signed projected area.
            let area: f32 = lobes.iter().map(|x| x.normal_area[3]).sum();
            let exact: f32 = mesh
                .vertices
                .chunks_exact(3)
                .map(|t| {
                    let p = t.map_positions();
                    (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5
                })
                .sum();
            assert!((area - exact).abs() < 1e-4);
            let projected: f32 = lobes
                .iter()
                .map(|b| b.normal_area[1] * b.normal_area[3])
                .sum();
            assert!((projected - 4.0).abs() < 1e-5);
            for channel in 0..3 {
                let aggregate: f32 = lobes
                    .iter()
                    .map(|b| b.color[channel] * b.normal_area[3])
                    .sum();
                let explicit: f32 = mesh
                    .vertices
                    .chunks_exact(3)
                    .map(|t| {
                        let p = t.map_positions();
                        (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5 * t[0].color[channel]
                    })
                    .sum();
                assert!((aggregate - explicit).abs() < 1e-4);
            }
        }
        for t in mesh.vertices.chunks_exact(3) {
            assert!(t[0].normal[1] > 0.5);
        }
        let a: f32 = mesh
            .vertices
            .chunks_exact(3)
            .map(|t| {
                let p = t.map_positions();
                (p[1] - p[0]).cross(p[2] - p[0]).y * 0.5
            })
            .sum();
        assert!((a - 4.0).abs() < 1e-5);
        assert_eq!(mesh.proxy.len(), TILE_COUNT * 6);
        let second = generate(16);
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&mesh.lobes32),
            bytemuck::cast_slice::<_, u8>(&second.lobes32)
        );
    }
}
