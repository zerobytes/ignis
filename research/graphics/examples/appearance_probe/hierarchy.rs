//! Spatial summaries with a strict final-pixel containment gate.
use super::{
    geometry::{self, Accumulator, Lobe, Mesh, TileBounds, Vertex},
    gpu::{Case, FOV_Y, NEAR},
};
use glam::{Mat4, Vec2, Vec3};
use std::time::Instant;

pub const BINS: usize = 32;
pub struct Node {
    pub x: usize,
    pub z: usize,
    pub width: usize,
    pub bounds: TileBounds,
    pub children: Option<[usize; 4]>,
    pub full_start: u32,
}
pub struct Tree {
    pub n: usize,
    pub nodes: Vec<Node>,
    pub lobes: Vec<Lobe>,
    pub proxy: Vec<Vertex>,
    pub build_ms: f64,
}
impl Tree {
    pub fn build(mesh: &Mesh, n: usize) -> Self {
        assert!(n.is_power_of_two() && (8..=128).contains(&n));
        assert_eq!(mesh.vertices.len(), n * n * 12);
        let start = Instant::now();
        let count = (4 * n * n - 1) / 3;
        let mut tree = Self {
            n,
            nodes: Vec::with_capacity(count),
            lobes: Vec::with_capacity(count * BINS),
            proxy: Vec::with_capacity(count * 6),
            build_ms: 0.0,
        };
        tree.node(mesh, 0, 0, n);
        for (id, node) in tree.nodes.iter().enumerate() {
            for (dx, dz) in [(0., 0.), (0., 1.), (1., 1.), (0., 0.), (1., 1.), (1., 0.)] {
                let p =
                    node.bounds.min + Vec3::new(dx, 0., dz) * (node.bounds.max - node.bounds.min);
                tree.proxy.push(Vertex {
                    position: p.to_array(),
                    normal: [0., 1., 0.],
                    color: [0.; 3],
                    tile: id as u32 | (1 << 31),
                });
            }
        }
        tree.build_ms = start.elapsed().as_secs_f64() * 1000.0;
        tree
    }
    fn node(
        &mut self,
        mesh: &Mesh,
        x: usize,
        z: usize,
        width: usize,
    ) -> (usize, [Accumulator; BINS]) {
        let id = self.nodes.len();
        let cell = 2.0 / self.n as f32;
        let min = Vec3::new(-1.0 + x as f32 * cell, 0., -1.0 + z as f32 * cell);
        let per_tile = self.n / geometry::TILES;
        let tile = (z / per_tile) * geometry::TILES + x / per_tile;
        let full_start =
            ((tile * per_tile * per_tile + (z % per_tile) * per_tile + x % per_tile) * 12) as u32;
        self.nodes.push(Node {
            x,
            z,
            width,
            bounds: TileBounds {
                min,
                max: min + Vec3::new(width as f32 * cell, 0., width as f32 * cell),
                cell_diameter: 0.,
            },
            children: None,
            full_start,
        });
        self.lobes.resize((id + 1) * BINS, Lobe::default());
        let mut bins = [Accumulator::default(); BINS];
        let mut height = 0.0_f32;
        if width == 1 {
            for tri in mesh.vertices[full_start as usize..full_start as usize + 12].chunks_exact(3)
            {
                let a = Vec3::from_array(tri[0].position);
                let b = Vec3::from_array(tri[1].position);
                let c = Vec3::from_array(tri[2].position);
                height = height.max(a.y.max(b.y).max(c.y));
                let normal = Vec3::from_array(tri[0].normal);
                bins[geometry::bin(normal, BINS)].add(
                    normal,
                    Vec3::from_array(tri[0].color),
                    f64::from((b - a).cross(c - a).length()) * 0.5,
                );
            }
        } else {
            let half = width / 2;
            let mut children = [0; 4];
            for (i, (dx, dz)) in [(0, 0), (half, 0), (0, half), (half, half)]
                .into_iter()
                .enumerate()
            {
                let (child, accum) = self.node(mesh, x + dx, z + dz, half);
                children[i] = child;
                height = height.max(self.nodes[child].bounds.max.y);
                for (bin, other) in bins.iter_mut().zip(accum) {
                    bin.merge(other);
                }
            }
            self.nodes[id].children = Some(children);
        }
        self.nodes[id].bounds.max.y = height;
        self.nodes[id].bounds.cell_diameter = (self.nodes[id].bounds.max - min).length();
        for (out, bin) in self.lobes[id * BINS..(id + 1) * BINS].iter_mut().zip(bins) {
            *out = bin.finish();
        }
        (id, bins)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Projection {
    pub min: Vec2,
    pub max: Vec2,
    pub pixel: Option<[u32; 2]>,
}
impl Projection {
    pub fn span(self) -> f32 {
        (self.max - self.min).max_element()
    }
}
pub fn project(bounds: TileBounds, view: Mat4, size: u32) -> Projection {
    project_with_focal(
        bounds,
        view,
        size,
        size as f32 / (2.0 * (FOV_Y * 0.5).tan()),
    )
}

pub(super) fn project_with_focal(
    bounds: TileBounds,
    view: Mat4,
    size: u32,
    focal: f32,
) -> Projection {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for x in [bounds.min.x, bounds.max.x] {
        for y in [bounds.min.y, bounds.max.y] {
            for z in [bounds.min.z, bounds.max.z] {
                let p = view.transform_point3(Vec3::new(x, y, z));
                if !p.is_finite() || -p.z <= NEAR {
                    return Projection {
                        min: Vec2::ZERO,
                        max: Vec2::splat(f32::INFINITY),
                        pixel: None,
                    };
                }
                let screen = Vec2::splat(size as f32 * 0.5) + Vec2::new(p.x, -p.y) * (focal / -p.z);
                min = min.min(screen);
                max = max.max(screen);
            }
        }
    }
    // Inflate for f32 rounding; never use a tolerance to accept a boundary crossing.
    let margin = size as f32 * f32::EPSILON * 4.0;
    min -= Vec2::splat(margin);
    max += Vec2::splat(margin);
    let inside = min.cmpge(Vec2::ZERO).all() && max.cmplt(Vec2::splat(size as f32)).all();
    let pixel = (inside && min.floor() == max.floor())
        .then_some([min.x.max(0.) as u32, min.y.max(0.) as u32]);
    Projection { min, max, pixel }
}

pub struct Selector {
    summarized: Vec<bool>,
}
pub struct Plan {
    pub full: Vec<u32>,
    pub proxy: Vec<u32>,
    pub summaries: Vec<usize>,
    pub leaves: Vec<usize>,
    pub visited: usize,
    pub max_span: f32,
}
impl Plan {
    pub fn new(tree: &Tree) -> Self {
        Self {
            full: Vec::with_capacity(tree.n * tree.n * 12),
            proxy: Vec::with_capacity(tree.n * tree.n * 6),
            summaries: Vec::with_capacity(tree.n * tree.n),
            leaves: Vec::with_capacity(tree.n * tree.n),
            visited: 0,
            max_span: 0.,
        }
    }
    pub fn triangles(&self) -> usize {
        (self.full.len() + self.proxy.len()) / 3
    }
    pub fn levels(&self, tree: &Tree) -> [usize; 8] {
        let mut counts = [0; 8];
        for &id in &self.summaries {
            counts[tree.nodes[id].width.ilog2() as usize] += 1;
        }
        counts
    }
}
impl Selector {
    pub fn new(tree: &Tree) -> Self {
        Self {
            summarized: vec![false; tree.nodes.len()],
        }
    }
    pub fn update(&mut self, tree: &Tree, case: Case, size: u32, plan: &mut Plan) {
        plan.full.clear();
        plan.proxy.clear();
        plan.summaries.clear();
        plan.leaves.clear();
        plan.visited = 0;
        plan.max_span = 0.;
        self.visit(
            tree,
            0,
            case.view_matrix(),
            size,
            size as f32 / (2.0 * (FOV_Y * 0.5).tan()),
            plan,
        );
    }
    fn visit(
        &mut self,
        tree: &Tree,
        id: usize,
        view: Mat4,
        size: u32,
        focal: f32,
        plan: &mut Plan,
    ) {
        let node = &tree.nodes[id];
        let projection = project_with_focal(node.bounds, view, size, focal);
        plan.visited += 1;
        let summarized = projection.pixel.is_some()
            && projection.span() < if self.summarized[id] { 1.0 } else { 0.75 };
        self.summarized[id] = summarized;
        if summarized {
            plan.summaries.push(id);
            plan.proxy.extend(id as u32 * 6..id as u32 * 6 + 6);
            plan.max_span = plan.max_span.max(projection.span());
        } else if let Some(children) = node.children {
            for child in children {
                self.visit(tree, child, view, size, focal, plan);
            }
        } else {
            plan.leaves.push(id);
            plan.full.extend(node.full_start..node.full_start + 12);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn camera(distance: f32, shift: f32) -> Case {
        Case {
            distance,
            shift,
            tilt: 35.,
            light_angle: 25.,
            exponent: 96.,
            specular: 2.,
        }
    }
    #[test]
    fn hierarchy_conserves_area_normal_material_and_covers_source() {
        let mesh = geometry::generate(16);
        let tree = Tree::build(&mesh, 16);
        assert_eq!(tree.nodes.len(), 341);
        for (id, node) in tree.nodes.iter().enumerate() {
            if let Some(children) = node.children {
                for channel in 0..7 {
                    let moment = |bins: &[Lobe]| {
                        bins.iter()
                            .map(|b| {
                                if channel < 3 {
                                    b.normal_area[channel] * b.normal_area[3]
                                } else if channel == 3 {
                                    b.normal_area[3]
                                } else {
                                    b.color[channel - 4] * b.normal_area[3]
                                }
                            })
                            .sum::<f32>()
                    };
                    let parent = moment(&tree.lobes[id * BINS..(id + 1) * BINS]);
                    let sum = children
                        .iter()
                        .map(|&c| moment(&tree.lobes[c * BINS..(c + 1) * BINS]))
                        .sum::<f32>();
                    assert!((sum - parent).abs() < 2e-5);
                }
            } else {
                let points =
                    &mesh.vertices[node.full_start as usize..node.full_start as usize + 12];
                assert!(points.iter().all(|p| {
                    Vec3::from_array(p.position).cmpge(node.bounds.min).all()
                        && Vec3::from_array(p.position).cmple(node.bounds.max).all()
                }));
            }
        }
    }
    #[test]
    fn subpixel_extent_is_not_same_pixel_and_near_plane_is_rejected() {
        let case = Case {
            tilt: 0.,
            ..camera(10., 0.)
        };
        let bound = TileBounds {
            min: Vec3::new(-0.005, 0., -0.005),
            max: Vec3::new(0.005, 0.001, 0.005),
            cell_diameter: 0.,
        };
        let crossing = project(bound, case.view_matrix(), 256);
        assert!(crossing.span() < 1.0 && crossing.pixel.is_none());
        let shifted = TileBounds {
            min: bound.min + Vec3::new(0.015, 0., 0.015),
            max: bound.max + Vec3::new(0.015, 0., 0.015),
            ..bound
        };
        assert!(project(shifted, case.view_matrix(), 256).pixel.is_some());
        assert!(
            project(
                bound,
                Case {
                    distance: 0.05,
                    ..case
                }
                .view_matrix(),
                256
            )
            .pixel
            .is_none()
        );
    }
    #[test]
    fn selected_nodes_partition_cells_and_stay_in_one_pixel_on_return_and_resize() {
        let mesh = geometry::generate(32);
        let tree = Tree::build(&mesh, 32);
        let mut selector = Selector::new(&tree);
        let mut plan = Plan::new(&tree);
        let mut widths = std::collections::BTreeSet::new();
        for size in [256, 512] {
            for d in [2.5, 8., 16., 32., 64., 32., 16., 8., 2.5] {
                let case = camera(d, 0.023);
                selector.update(&tree, case, size, &mut plan);
                let mut hits = vec![0; tree.n * tree.n];
                for &id in plan.summaries.iter().chain(&plan.leaves) {
                    let node = &tree.nodes[id];
                    for z in node.z..node.z + node.width {
                        for x in node.x..node.x + node.width {
                            hits[z * tree.n + x] += 1;
                        }
                    }
                }
                assert!(hits.iter().all(|h| *h == 1));
                for &id in &plan.summaries {
                    let node = &tree.nodes[id];
                    widths.insert(node.width);
                    let p = project(node.bounds, case.view_matrix(), size);
                    assert!(p.pixel.is_some() && p.span() < 1.0);
                }
                if d == 2.5 {
                    assert!(plan.summaries.is_empty());
                    assert_eq!(plan.full.len(), mesh.vertices.len());
                }
            }
        }
        assert!(widths.len() >= 2);
    }
}
