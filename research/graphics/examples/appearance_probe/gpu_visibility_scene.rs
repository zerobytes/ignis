//! Asset-local root/region bounds baked once, and current-frame test scenes.
use super::*;
pub(super) const WALL_HALVES: [Vec3; 4] = [
    Vec3::new(12., 6., 0.6),
    Vec3::new(12., 6., 0.6),
    Vec3::new(16., 5., 0.75),
    Vec3::new(10., 8., 0.6),
];
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Serialize, Deserialize)]
pub(super) struct Node {
    pub min: [f32; 4],
    pub max: [f32; 4],
    pub range: [u32; 4],
}
#[derive(Serialize, Deserialize)]
pub(super) struct Hierarchy {
    pub signature: u64,
    pub regions: usize,
    pub nodes: Vec<Node>,
}
fn signature(scene: &Scene) -> u64 {
    bytemuck::cast_slice::<Vertex, u8>(&scene.vertices)
        .iter()
        .chain(bytemuck::cast_slice::<u32, u8>(&scene.indices))
        .fold(0xcbf29ce484222325, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
        })
}
pub(super) fn hierarchy(scene: &Scene, bake: bool) -> Result<Hierarchy> {
    let path = Path::new(ROOT).join("hierarchy.json");
    if bake {
        let mut nodes = Vec::new();
        for (aid, a) in scene.assets.iter().enumerate() {
            for r in &a.regions {
                nodes.push(Node {
                    min: r.min.extend(0.).to_array(),
                    max: r.max.extend(0.).to_array(),
                    range: [scene.ranges[aid].start + r.first, r.count, aid as u32, 0],
                });
            }
        }
        let regions = nodes.len();
        for (aid, [min, max]) in scene.asset_bounds.iter().enumerate() {
            nodes.push(Node {
                min: min.extend(0.).to_array(),
                max: max.extend(0.).to_array(),
                range: [
                    scene.ranges[aid].start,
                    scene.ranges[aid].end - scene.ranges[aid].start,
                    aid as u32,
                    0,
                ],
            });
        }
        fs::write(
            &path,
            serde_json::to_vec_pretty(&Hierarchy {
                signature: signature(scene),
                regions,
                nodes,
            })?,
        )?;
    }
    let h: Hierarchy =
        serde_json::from_slice(&fs::read(&path).context("run --bake-gpu-visibility first")?)?;
    ensure!(
        h.signature == signature(scene)
            && h.regions == scene.regions.len()
            && h.nodes.len() == h.regions + 3,
        "stale hierarchy"
    );
    for n in &h.nodes {
        ensure!(
            n.min.iter().chain(&n.max).all(|x| x.is_finite())
                && n.range[0] + n.range[1] <= scene.indices.len() as u32
                && n.range[2] < 3,
            "invalid node"
        );
    }
    Ok(h)
}
fn cube(v: &mut Vec<Vertex>, indices: &mut Vec<u32>, half: Vec3, color: [f32; 3]) -> Range<u32> {
    let start = indices.len() as u32;
    for [u, w, n] in AXES {
        let base = v.len() as u32;
        for (x, y) in [(-1., -1.), (1., -1.), (-1., 1.), (1., 1.)] {
            v.push(Vertex {
                position: ((n + u * x + w * y) * half).to_array(),
                normal: n.to_array(),
                color,
                tile: 5,
            });
        }
        indices.extend([base, base + 2, base + 1, base + 2, base + 3, base + 1]);
    }
    start..indices.len() as u32
}
pub(super) struct World {
    pub scene: Scene,
    pub hierarchy: Hierarchy,
    pub walls: Vec<Range<u32>>,
    pub members: Vec<u32>,
}
impl World {
    pub fn new(mut scene: Scene, mut hierarchy: Hierarchy) -> Self {
        // This geometry-only experiment does not upload or retain appearance caches.
        scene.cache = Vec::new();
        for h in WALL_HALVES {
            hierarchy.nodes.push(Node {
                min: (-h).extend(0.).to_array(),
                max: h.extend(0.).to_array(),
                range: [0; 4],
            });
        }
        let walls = WALL_HALVES
            .into_iter()
            .enumerate()
            .map(|(i, h)| {
                cube(
                    &mut scene.vertices,
                    &mut scene.indices,
                    h,
                    [0.22 + i as f32 * 0.035, 0.25, 0.28],
                )
            })
            .collect();
        let members = (0..3)
            .flat_map(|a| {
                scene
                    .objects
                    .iter()
                    .enumerate()
                    .filter(move |(_, o)| o.asset == a)
                    .map(|(i, _)| i as u32 + 1)
            })
            .collect::<Vec<_>>();
        assert_eq!(members.len(), 96);
        Self {
            scene,
            hierarchy,
            walls,
            members,
        }
    }
    pub fn pose(&self, scenario: usize, t: f32, size: [u32; 2], p: &mut Pose) {
        let mut cam = camera(t, size);
        if scenario > 0 {
            let eye = Vec3::new(15. * (t * std::f32::consts::TAU).sin(), 4., -15. + 110. * t);
            let view = Mat4::look_at_rh(eye, eye + Vec3::new(-eye.x * 0.025, -0.5, 35.), Vec3::Y);
            let matrix = Mat4::perspective_infinite_reverse_rh(
                60f32.to_radians(),
                size[0] as f32 / size[1] as f32,
                0.1,
            ) * view;
            cam = Camera {
                eye,
                view,
                matrix,
                inverse: matrix.inverse(),
                size,
                light: cam.light,
            };
        }
        p.camera = cam;
        p.input.clear();
        p.obstacles.clear();
        p.baseline.clear();
        p.input.push(InstanceGpu {
            position: [0., 0., 0., 1.],
            meta: [0; 4],
        });
        for (id, o) in self.scene.objects.iter().enumerate() {
            let mut pos = o.position;
            // Rigid transform updates only; source mesh and local bounds remain valid.
            if id % 7 == 0 {
                pos.x += 8. * (t * std::f32::consts::TAU + id as f32 * 0.3).sin();
                pos.z += 3. * (t * std::f32::consts::TAU).cos();
            }
            p.input.push(InstanceGpu {
                position: pos.extend(o.scale).to_array(),
                meta: [o.asset as u32, 0, id as u32 + 1, 0],
            });
        }
        for (i, base) in [
            Vec3::new(-14., 6., 20.),
            Vec3::new(14., 6., 20.),
            Vec3::new(0., 5., 60.),
            Vec3::new(-25., 8., 120.),
        ]
        .into_iter()
        .enumerate()
        {
            let mut pos = base;
            if scenario == 2 && i == 2 {
                pos.x += 28. * (t * std::f32::consts::TAU).sin();
            }
            p.input.push(InstanceGpu {
                position: pos.extend(1.).to_array(),
                meta: [0, 0, 97 + i as u32, 0],
            });
        }
        p.obstacles.push(Draw {
            indices: self.scene.ground.clone(),
            instance: 0,
            count: 1,
        });
        if scenario > 0 {
            for (i, indices) in self.walls.iter().enumerate() {
                p.obstacles.push(Draw {
                    indices: indices.clone(),
                    instance: 97 + i as u32,
                    count: 1,
                });
            }
        }
        p.triangles = 0;
        for a in 0..3 {
            let first = p.input.len() as u32;
            let mut count = 0;
            for id in &self.members[a * 32..a * 32 + 32] {
                let inst = p.input[*id as usize];
                let pos = Vec3::from_array(inst.position[..3].try_into().unwrap());
                let scale = inst.position[3];
                let [min, max] = self.scene.asset_bounds[a];
                if visible(&p.camera, min * scale + pos, max * scale + pos) {
                    p.input.push(inst);
                    count += 1;
                }
            }
            p.baseline.push(Draw {
                indices: self.scene.ranges[a].clone(),
                instance: first,
                count,
            });
            p.triangles += (self.scene.ranges[a].end - self.scene.ranges[a].start) as u64 / 3
                * u64::from(count);
        }
        for d in &p.obstacles {
            p.triangles += (d.indices.end - d.indices.start) as u64 / 3;
        }
    }
}
pub(super) struct Pose {
    pub camera: Camera,
    pub input: Vec<InstanceGpu>,
    pub obstacles: Vec<Draw>,
    pub baseline: Vec<Draw>,
    pub triangles: u64,
}
impl Pose {
    pub fn new() -> Self {
        Self {
            camera: camera(0., RESOLUTIONS[0]),
            input: Vec::with_capacity(200),
            obstacles: Vec::with_capacity(5),
            baseline: Vec::with_capacity(3),
            triangles: 0,
        }
    }
}
