//! Regional, depth-composited surface caches in a shared scene. Isolated research example.
use super::*;
use glam::Vec2;
use std::ops::Range;
#[path = "regional_build.rs"]
mod build;
#[path = "regional_efficiency.rs"]
mod efficiency;
#[path = "regional_scaling.rs"]
mod regional_scaling;
#[path = "regional_render.rs"]
mod render;
#[path = "triangle_scaling.rs"]
mod scaling;
#[path = "gpu_visibility.rs"]
mod visibility;
use build::*;
use render::*;
const OUT: &str = "artifacts/appearance-probe/regional-scene";
const RESOLUTIONS: [[u32; 2]; 2] = [[3456, 2234], [1728, 1117]];
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RegionGpu {
    center: [f32; 4],
    half: [f32; 4],
    meta: [u32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceGpu {
    position: [f32; 4],
    meta: [u32; 4],
}
#[derive(Clone, Serialize)]
struct Object {
    asset: usize,
    position: Vec3,
    scale: f32,
}
#[derive(Clone)]
struct Draw {
    indices: Range<u32>,
    instance: u32,
    count: u32,
}
#[derive(Clone)]
struct Camera {
    matrix: Mat4,
    inverse: Mat4,
    view: Mat4,
    eye: Vec3,
    size: [u32; 2],
    light: Vec3,
}
fn camera(t: f32, size: [u32; 2]) -> Camera {
    let eye = Vec3::new(4. * (t * std::f32::consts::PI).sin(), 7., 65. - 90. * t);
    let view = Mat4::look_at_rh(eye, eye + Vec3::new(-eye.x * 0.02, -2., 35.), Vec3::Y);
    let matrix = Mat4::perspective_infinite_reverse_rh(
        60f32.to_radians(),
        size[0] as f32 / size[1] as f32,
        0.1,
    ) * view;
    Camera {
        matrix,
        inverse: matrix.inverse(),
        view,
        eye,
        size,
        light: Vec3::new(-0.35, 0.8, -0.42).normalize(),
    }
}
fn height(x: f32, z: f32) -> f32 {
    0.18 * (x * 0.04).sin() * (z * 0.025).cos()
}
fn objects(assets: &[Asset]) -> Vec<Object> {
    let mut out = Vec::new();
    for (row, z) in [
        12., 22., 38., 65., 100., 150., 220., 320., 460., 660., 900., 1200.,
    ]
    .into_iter()
    .enumerate()
    {
        for col in 0..8 {
            let asset = (row * 8 + col) % 3;
            let seed = (row * 79 + col * 37 + 11) as f32;
            let x = ((col as f32 - 3.5) / 8. + 0.055 * seed.sin()) * z * 0.95;
            let scale = 0.75 + (seed * 0.71).sin().abs() * 1.25;
            let min = assets[asset]
                .vertices
                .iter()
                .map(|v| v.position[1])
                .fold(f32::INFINITY, f32::min);
            out.push(Object {
                asset,
                position: Vec3::new(x, height(x, z) - min * scale, z),
                scale,
            });
        }
    }
    out
}
fn ground() -> (Vec<Vertex>, Vec<u32>) {
    let mut vs = Vec::new();
    let mut ids = Vec::new();
    for z in 0..=128 {
        for x in 0..=128 {
            let px = -750. + x as f32 * 1500. / 128.;
            let pz = -50. + z as f32 * 1350. / 128.;
            let h = height(px, pz);
            let n = Vec3::new(
                -(height(px + 0.01, pz) - h) / 0.01,
                1.,
                -(height(px, pz + 0.01) - h) / 0.01,
            )
            .normalize();
            let c = 0.19 + 0.02 * (px * 0.08 + pz * 0.03).sin();
            vs.push(Vertex {
                position: [px, h, pz],
                normal: n.to_array(),
                color: [c * 0.8, c, c * 0.63],
                tile: 5,
            });
        }
    }
    for z in 0..128 {
        for x in 0..128 {
            let a = z * 129 + x;
            ids.extend([a, a + 129, a + 1, a + 1, a + 129, a + 130]);
        }
    }
    (vs, ids)
}
fn corners(min: Vec3, max: Vec3) -> impl Iterator<Item = Vec3> {
    [min.x, max.x].into_iter().flat_map(move |x| {
        [min.y, max.y]
            .into_iter()
            .flat_map(move |y| [min.z, max.z].into_iter().map(move |z| Vec3::new(x, y, z)))
    })
}
fn visible(cam: &Camera, min: Vec3, max: Vec3) -> bool {
    let points: [glam::Vec4; 8] = std::array::from_fn(|i| {
        cam.matrix
            * Vec3::new(
                if i & 1 == 0 { min.x } else { max.x },
                if i & 2 == 0 { min.y } else { max.y },
                if i & 4 == 0 { min.z } else { max.z },
            )
            .extend(1.)
    });
    !(points.iter().all(|p| p.x < -p.w)
        || points.iter().all(|p| p.x > p.w)
        || points.iter().all(|p| p.y < -p.w)
        || points.iter().all(|p| p.y > p.w)
        || points.iter().all(|p| p.z < 0.)
        || points.iter().all(|p| p.z > p.w))
}
fn bound(cam: &Camera, r: &Region, o: &Object) -> [f32; 2] {
    let mut depth = f32::INFINITY;
    let mut xy = Vec2::ZERO;
    for p in corners(r.min, r.max) {
        let p = cam.view.transform_point3(p * o.scale + o.position);
        depth = depth.min(-p.z);
        xy = xy.max(p.truncate().abs());
    }
    if depth <= 0.1 || !depth.is_finite() {
        return [f32::INFINITY; 2];
    }
    let scale = cam.size[1] as f32 / (2. * 30f32.to_radians().tan()) / depth
        * (1. + xy.length_squared() / depth.powi(2)).sqrt()
        * o.scale;
    [
        r.edge * scale,
        2f32.sqrt() * 4. * r.radius / SIDE as f32 * scale,
    ]
}
fn select(state: &mut bool, b: [f32; 2], count: u32, material: u32) {
    // Mean normals do not preserve narrow specular lobes; keep glossy source regions.
    if count < 24 || matches!(material, 2..=4) || b.iter().any(|x| !x.is_finite() || *x >= 1.) {
        *state = false;
    } else if b.iter().all(|x| *x < 0.75) {
        *state = true;
    }
}
fn face(direction: Vec3) -> u32 {
    let p = direction.abs();
    if p.x >= p.y && p.x >= p.z {
        if direction.x >= 0. { 0 } else { 1 }
    } else if p.y >= p.z {
        if direction.y >= 0. { 2 } else { 3 }
    } else if direction.z >= 0. {
        4
    } else {
        5
    }
}
#[derive(Default, Clone, Serialize)]
struct Counts {
    visible_objects: usize,
    mixed_objects: usize,
    geometry_regions: usize,
    summary_regions: usize,
    triangles: u64,
    draw_calls: usize,
    selection_us: f64,
    upload_bytes: usize,
    max_summary_bound: f32,
    per_object: Vec<ObjectCounts>,
}
#[derive(Clone, Serialize)]
struct ObjectCounts {
    id: usize,
    asset: usize,
    geometry_regions: usize,
    summary_regions: usize,
    max_summary_bound: f32,
    distance: f32,
}
#[derive(Clone)]
struct Plan {
    draws: Vec<Draw>,
    instances: Vec<InstanceGpu>,
    counts: Counts,
    proxies: Range<u32>,
    view_regions: Vec<[Vec3; 2]>,
}
impl Plan {
    fn new(cap: usize) -> Self {
        Self {
            proxies: 0..0,
            view_regions: Vec::with_capacity(512),
            draws: Vec::with_capacity(cap),
            instances: Vec::with_capacity(cap),
            counts: Counts {
                per_object: Vec::with_capacity(96),
                ..Counts::default()
            },
        }
    }
}
struct Scene {
    assets: Vec<Asset>,
    objects: Vec<Object>,
    regions: Vec<RegionGpu>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    ranges: Vec<Range<u32>>,
    region_offsets: Vec<usize>,
    ground: Range<u32>,
    cache: Vec<Texel>,
    asset_bounds: Vec<[Vec3; 2]>,
}
impl Scene {
    fn new(mut assets: Vec<Asset>) -> Self {
        let objects = objects(&assets);
        let (mut vertices, mut indices) = ground();
        let ground = 0..indices.len() as u32;
        let mut regions = Vec::new();
        let mut ranges = Vec::new();
        let mut region_offsets = Vec::new();
        let mut cache = Vec::new();
        for a in &mut assets {
            let start = indices.len() as u32;
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&a.vertices);
            indices.extend(a.indices.iter().map(|i| i + base));
            ranges.push(start..indices.len() as u32);
            region_offsets.push(regions.len());
            for (id, r) in a.regions.iter().enumerate() {
                regions.push(RegionGpu {
                    center: r.center.extend(r.radius).to_array(),
                    half: ((r.max - r.min) * 0.5).extend(0.).to_array(),
                    meta: [
                        (cache.len() + id * 6 * FACE_TEXELS) as u32,
                        r.material,
                        0,
                        0,
                    ],
                });
            }
            let mut payload = std::mem::take(&mut a.cache);
            cache.append(&mut payload);
        }
        let asset_bounds = assets
            .iter()
            .map(|a| {
                [
                    a.regions
                        .iter()
                        .map(|r| r.min)
                        .fold(Vec3::splat(f32::INFINITY), Vec3::min),
                    a.regions
                        .iter()
                        .map(|r| r.max)
                        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max),
                ]
            })
            .collect();
        Self {
            asset_bounds,
            assets,
            objects,
            regions,
            vertices,
            indices,
            ranges,
            region_offsets,
            ground,
            cache,
        }
    }
    fn states(&self) -> Vec<Vec<bool>> {
        self.objects
            .iter()
            .map(|o| vec![false; self.assets[o.asset].regions.len()])
            .collect()
    }
    fn plan(&self, cam: &Camera, adaptive: bool, states: &mut [Vec<bool>], plan: &mut Plan) {
        self.plan_variant(cam, adaptive, states, plan, false);
    }
    fn plan_variant(
        &self,
        cam: &Camera,
        adaptive: bool,
        states: &mut [Vec<bool>],
        plan: &mut Plan,
        fast: bool,
    ) {
        let begin = Instant::now();
        plan.draws.clear();
        if fast && adaptive {
            plan.view_regions.clear();
            let abs = glam::Mat3::from_cols(
                cam.view.x_axis.truncate().abs(),
                cam.view.y_axis.truncate().abs(),
                cam.view.z_axis.truncate().abs(),
            );
            for a in &self.assets {
                for r in &a.regions {
                    plan.view_regions.push([
                        cam.view.transform_vector3(r.center),
                        abs * ((r.max - r.min) * 0.5),
                    ]);
                }
            }
        }
        plan.instances.clear();
        let mut per_object = std::mem::take(&mut plan.counts.per_object);
        per_object.clear();
        plan.counts = Counts {
            per_object,
            ..Counts::default()
        };
        plan.instances.push(InstanceGpu {
            position: [0., 0., 0., 1.],
            meta: [0; 4],
        });
        for (i, o) in self.objects.iter().enumerate() {
            plan.instances.push(InstanceGpu {
                position: o.position.extend(o.scale).to_array(),
                meta: [0, 0, i as u32 + 1, 0],
            });
        }
        plan.draws.push(Draw {
            indices: self.ground.clone(),
            instance: 0,
            count: 1,
        });
        plan.counts.triangles = (self.ground.end - self.ground.start) as u64 / 3;
        for (id, o) in self.objects.iter().enumerate() {
            let asset = &self.assets[o.asset];
            let min = self.asset_bounds[o.asset][0] * o.scale + o.position;
            let max = self.asset_bounds[o.asset][1] * o.scale + o.position;
            if !visible(cam, min, max) {
                continue;
            }
            plan.counts.visible_objects += 1;
            if !adaptive {
                plan.draws.push(Draw {
                    indices: self.ranges[o.asset].clone(),
                    instance: id as u32 + 1,
                    count: 1,
                });
                plan.counts.triangles += asset.indices.len() as u64 / 3;
                plan.counts.geometry_regions += asset.regions.len();
                plan.counts.per_object.push(ObjectCounts {
                    id: id + 1,
                    asset: o.asset,
                    geometry_regions: asset.regions.len(),
                    summary_regions: 0,
                    max_summary_bound: 0.,
                    distance: cam.eye.distance(o.position),
                });
                continue;
            }
            let view_origin = cam.view.transform_point3(o.position);
            let mut full = 0;
            let mut summarized = 0;
            let start = self.ranges[o.asset].start;
            let mut run: Option<Range<u32>> = None;
            let mut object_max = 0f32;
            for (rid, r) in asset.regions.iter().enumerate() {
                let px = if fast {
                    efficiency::fast_bound(
                        cam,
                        r,
                        o,
                        plan.view_regions[self.region_offsets[o.asset] + rid],
                        view_origin,
                    )
                } else {
                    bound(cam, r, o)
                };
                if adaptive {
                    select(&mut states[id][rid], px, r.count, r.material);
                }
                let use_summary = adaptive && states[id][rid];
                if use_summary {
                    if let Some(range) = run.take() {
                        plan.draws.push(Draw {
                            indices: range,
                            instance: id as u32 + 1,
                            count: 1,
                        });
                    }
                    summarized += 1;
                    plan.counts.triangles += 2;
                    let q = px[0].max(px[1]);
                    object_max = object_max.max(q);
                    plan.counts.max_summary_bound = plan.counts.max_summary_bound.max(q);
                    let d = (cam.eye - o.position) / o.scale - r.center;
                    let span = (px[1] / (2f32.sqrt() * 2.)) * SIDE as f32;
                    let lod = if fast
                        && [
                            (std::f32::consts::FRAC_1_SQRT_2 * 0.25),
                            (std::f32::consts::FRAC_1_SQRT_2 * 0.5),
                            std::f32::consts::FRAC_1_SQRT_2,
                        ]
                        .iter()
                        .all(|v| (px[1] - v).abs() > 0.0001)
                    {
                        if px[1] < (std::f32::consts::FRAC_1_SQRT_2 * 0.25) {
                            4
                        } else if px[1] < (std::f32::consts::FRAC_1_SQRT_2 * 0.5) {
                            3
                        } else if px[1] < std::f32::consts::FRAC_1_SQRT_2 {
                            2
                        } else {
                            1
                        }
                    } else {
                        (SIDE as f32 / span.max(1.)).log2().floor().clamp(0., 4.) as u32
                    };
                    plan.instances.push(InstanceGpu {
                        position: o.position.extend(o.scale).to_array(),
                        meta: [
                            (self.region_offsets[o.asset] + rid) as u32,
                            face(d),
                            id as u32 + 1,
                            lod,
                        ],
                    });
                } else {
                    full += 1;
                    plan.counts.triangles += r.count as u64 / 3;
                    let range = start + r.first..start + r.first + r.count;
                    if let Some(current) = &mut run {
                        current.end = range.end;
                    } else {
                        run = Some(range);
                    }
                }
            }
            if let Some(range) = run {
                plan.draws.push(Draw {
                    indices: range,
                    instance: id as u32 + 1,
                    count: 1,
                });
            }
            plan.counts.geometry_regions += full;
            plan.counts.summary_regions += summarized;
            if full > 0 && summarized > 0 {
                plan.counts.mixed_objects += 1;
            }
            plan.counts.per_object.push(ObjectCounts {
                id: id + 1,
                asset: o.asset,
                geometry_regions: full,
                summary_regions: summarized,
                max_summary_bound: object_max,
                distance: cam.eye.distance(o.position),
            });
        }
        plan.proxies = self.objects.len() as u32 + 1..plan.instances.len() as u32;
        plan.counts.draw_calls =
            plan.draws.len() + usize::from(plan.instances.len() > self.objects.len() + 1);
        plan.counts.upload_bytes = plan.instances.len() * 32;
        plan.counts.selection_us = begin.elapsed().as_secs_f64() * 1e6;
    }
}
fn errors(reference: &[[f32; 4]], candidate: &[[f32; 4]], mask: &[[f32; 4]]) -> serde_json::Value {
    let mut sum = 0.;
    let mut n = 0;
    let mut all = 0.;
    let mut max = 0f64;
    let mut mismatch = 0;
    let mut objects = vec![(0f64, 0usize); 97];
    for ((r, c), m) in reference.iter().zip(candidate).zip(mask) {
        let e = (0..3).map(|k| f64::from(c[k] - r[k]).powi(2)).sum::<f64>();
        all += e;
        if m[3] > 0. || c[3] > 0. || r[3] > 0. {
            sum += e;
            n += 1;
            max = max.max(e.sqrt());
            if (m[3] - c[3]).abs() > 0.1 {
                mismatch += 1;
            }
        }
        let id = m[3].round() as usize;
        if id > 0 && id < objects.len() {
            objects[id].0 += e;
            objects[id].1 += 1;
        }
    }
    json!({"foreground_rmse":(sum/(3*n.max(1)) as f64).sqrt(),"frame_rmse":(all/(3*reference.len()) as f64).sqrt(),"max_rgb_error_length":max,"foreground_pixels":n,"id_mismatch_pixels":mismatch,"objects":objects.iter().enumerate().skip(1).filter(|(_,(_,n))|*n>0).map(|(id,(e,n))|json!({"id":id,"pixels":n,"rmse":(e/(3*n) as f64).sqrt()})).collect::<Vec<_>>()})
}
fn save(path: &Path, pixels: &[[f32; 4]], width: u32) -> Result<()> {
    let opaque = pixels
        .iter()
        .map(|p| [p[0], p[1], p[2], 1.])
        .collect::<Vec<_>>();
    metrics::png(path, &opaque, width)
}
fn visible_regions(map: &[[f32; 4]], full: &[[f32; 4]], size: [u32; 2]) -> serde_json::Value {
    let mut counts = vec![(0usize, 0usize, [u32::MAX, u32::MAX, 0, 0]); 97];
    for (i, (m, f)) in map.iter().zip(full).enumerate() {
        let id = m[3].round() as usize;
        if id > 0 && id < counts.len() {
            if m[0] > 0.5 {
                counts[id].1 += 1;
            } else {
                counts[id].0 += 1;
            }
        }
        let id = f[3].round() as usize;
        if id > 0 && id < counts.len() {
            let x = i as u32 % size[0];
            let y = i as u32 / size[0];
            let b = &mut counts[id].2;
            b[0] = b[0].min(x);
            b[1] = b[1].min(y);
            b[2] = b[2].max(x + 1);
            b[3] = b[3].max(y + 1);
        }
    }
    json!(
        counts
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, (g, s, b))| g + s > 0 && b[0] != u32::MAX)
            .map(|(id, (g, s, b))| json!({"id":id,"geometry_pixels":g,"summary_pixels":s,"bbox":b}))
            .collect::<Vec<_>>()
    )
}
pub fn run(mode: &str) -> Result<()> {
    if matches!(
        mode,
        "--fixed-tiled-field"
            | "--tune-tiled-field"
            | "--calibrate-tiled-field"
            | "--capture-tiled-field"
            | "--motion-tiled-field"
            | "--depth-tiled-field"
            | "--bench-tiled-field"
            | "--repeat-tiled-field"
            | "--audit-tiled-field"
            | "--smoke-tiled-field"
            | "--tune-hierarchical-field"
            | "--compile-hierarchical-field"
            | "--calibrate-hierarchical-field"
            | "--capture-hierarchical-field"
            | "--motion-hierarchical-field"
            | "--depth-hierarchical-field"
            | "--bench-hierarchical-field"
            | "--repeat-hierarchical-field"
            | "--audit-hierarchical-field"
            | "--smoke-hierarchical-field"
            | "--work-hierarchical-field"
            | "--audit-geometric-field"
            | "--motion-geometric-field"
            | "--depth-geometric-field"
            | "--compile-geometric-field"
            | "--capture-geometric-field"
            | "--calibrate-geometric-field"
            | "--bench-geometric-field"
            | "--repeat-geometric-field"
            | "--smoke-geometric-field"
            | "--compile-combined-symbol"
            | "--calibrate-combined-symbol"
            | "--capture-combined-symbol"
            | "--bench-combined-symbol"
            | "--repeat-combined-symbol"
            | "--smoke-combined-symbol"
            | "--bake-hybrid-scaling"
            | "--hybrid-scaling"
            | "--confirm-hybrid-scaling"
            | "--hybrid-transitions"
            | "--calibrate-cost-scaling"
            | "--validate-cost-scaling"
            | "--capture-cost-scaling"
            | "--inspect-cost-scaling"
            | "--repeat-cost-scaling"
            | "--ablate-scaling"
            | "--validate-ablation"
            | "--capture-ablation"
            | "--order-ablation"
            | "--repeat-ablation"
            | "--extend-domain-automatic-shading"
            | "--calibrate-automatic-shading"
            | "--validate-automatic-shading"
            | "--repeat-automatic-shading"
            | "--capture-automatic-shading"
            | "--isolate-near-plateau"
            | "--profile-near-plateau"
            | "--pixel-density"
            | "--pixel-density-smoke"
            | "--bake-near-plateau"
            | "--footprint-near-plateau"
            | "--bench-near-plateau"
            | "--repeat-near-plateau"
            | "--capture-near-plateau"
            | "--depth-ablation"
            | "--capture-depth-ablation"
            | "--repeat-depth-ablation"
            | "--bake-regional-scaling"
            | "--regional-scaling"
            | "--confirm-regional-scaling"
            | "--regional-transitions"
    ) {
        return regional_scaling::run(mode);
    }
    if matches!(
        mode,
        "--bake-triangle-scaling" | "--triangle-scaling" | "--confirm-triangle-scaling"
    ) {
        return scaling::run(mode);
    }
    if matches!(
        mode,
        "--bake-gpu-visibility"
            | "--gpu-visibility"
            | "--verify-gpu-visibility"
            | "--benchmark-gpu-visibility"
    ) {
        return visibility::run(mode);
    }
    if mode == "--regional-efficiency" {
        return efficiency::run();
    }
    fs::create_dir_all(OUT)?;
    let assets = assets(mode == "--bake-regional-scene")?;
    if mode == "--bake-regional-scene" {
        return Ok(());
    }
    let scene = Scene::new(assets);
    let mut renderer = pollster::block_on(Renderer::new(&scene))?;
    let cap = scene.objects.len() * scene.regions.len();
    let mut full = Plan::new(cap);
    let mut adaptive = Plan::new(cap);
    let mut rows = Vec::new();
    let mut checks = Vec::new();
    let root = Path::new(OUT);
    for (res, size) in RESOLUTIONS.into_iter().enumerate() {
        let mut states = scene.states();
        let sequence = (0usize..13).chain((0usize..13).rev()).collect::<Vec<_>>();
        for (frame, step) in sequence.into_iter().enumerate() {
            let t = step as f32 / 12.;
            let cam = camera(t, size);
            scene.plan(&cam, false, &mut states, &mut full);
            scene.plan(&cam, true, &mut states, &mut adaptive);
            ensure!(
                adaptive.counts.max_summary_bound < 1.,
                "selected visible detail"
            );
            // Repeated preparation uses the same stable hysteresis decision; no image work here.
            let mut cpu_full = Vec::new();
            let mut cpu_adaptive = Vec::new();
            for _ in 0..9 {
                scene.plan(&cam, false, &mut states, &mut full);
                cpu_full.push(full.counts.selection_us);
                scene.plan(&cam, true, &mut states, &mut adaptive);
                cpu_adaptive.push(adaptive.counts.selection_us);
            }
            let cpu_timing =
                json!({"full_us":stats(&mut cpu_full),"adaptive_us":stats(&mut cpu_adaptive)});
            let id = format!("r{res}_{frame:02}");
            let a = renderer.capture(&cam, &full, false, [0.; 2])?;
            let b = renderer.capture(&cam, &adaptive, false, [0.; 2])?;
            let map = renderer.capture(&cam, &adaptive, true, [0.; 2])?;
            let visibility = visible_regions(&map, &a, size);
            if frame == 0 {
                ensure!(
                    b == renderer.capture(&cam, &adaptive, false, [0.; 2])?,
                    "non deterministic regional capture"
                );
                let mut reverse = adaptive.clone();
                reverse.draws.reverse();
                reverse.instances[scene.objects.len() + 1..].reverse();
                let reordered = renderer.capture(&cam, &reverse, false, [0.; 2])?;
                let error = errors(&b, &reordered, &b);
                ensure!(
                    error["foreground_rmse"].as_f64().unwrap() < 0.0001,
                    "draw ordering changes depth composition: {error}"
                );
                checks.push(
                    json!({"resolution":res,"repeat_identical":true,"reverse_draw_error":error}),
                );
            }
            let reference = if step.is_multiple_of(3) {
                let mut sum = vec![[0.; 4]; a.len()];
                for jitter in [[-0.25, -0.25], [0.25, -0.25], [-0.25, 0.25], [0.25, 0.25]] {
                    let image = renderer.capture(&cam, &full, false, jitter)?;
                    for (s, p) in sum.iter_mut().zip(image) {
                        for k in 0..4 {
                            s[k] += p[k] * 0.25;
                        }
                    }
                }
                Some(sum)
            } else {
                None
            };
            let timings = if step.is_multiple_of(3) {
                Some(renderer.bench(&cam, &scene, &states, &full, &adaptive)?)
            } else {
                None
            };
            save(&root.join(format!("{id}_full.png")), &a, size[0])?;
            save(&root.join(format!("{id}_adaptive.png")), &b, size[0])?;
            save(&root.join(format!("{id}_map.png")), &map, size[0])?;
            if let Some(r) = &reference {
                save(&root.join(format!("{id}_reference.png")), r, size[0])?;
            }
            metrics::error_png(&root.join(format!("{id}_error.png")), &a, &b, size[0])?;
            let row = json!({"id":id,"resolution":res,"size":size,"frame":frame,"step":step,"t":t,"way":if frame<13{"away"}else{"return"},"eye":cam.eye,"full":full.counts,"adaptive":adaptive.counts,"error_vs_full":errors(&a,&b,&a),"reference_full_error":reference.as_ref().map(|r|errors(r,&a,&a)),"reference_adaptive_error":reference.as_ref().map(|r|errors(r,&b,&a)),"timings":timings,"cpu_planning":cpu_timing,"visible_regions":visibility});
            println!(
                "{id}: {} mixed objects, {} summaries, {} -> {} triangles, {:0.5} foreground RMSE",
                adaptive.counts.mixed_objects,
                adaptive.counts.summary_regions,
                full.counts.triangles,
                adaptive.counts.triangles,
                row["error_vs_full"]["foreground_rmse"].as_f64().unwrap()
            );
            rows.push(row);
        }
    }
    let report = json!({"schema":1,"adapter":renderer.gpu.adapter,"objects":scene.objects,"assets":scene.assets.iter().map(|a|json!({"name":a.name,"triangles":a.indices.len()/3,"regions":a.regions.len(),"geometry_bytes":a.vertices.len()*40+a.indices.len()*4,"cache_bytes":a.cache_bytes,"bake_seconds":a.bake_seconds})).collect::<Vec<_>>(),"cache_allocated_bytes":renderer.cache_bytes,"vertex_bytes":scene.vertices.len()*40,"index_bytes":scene.indices.len()*4,"region_bytes":scene.regions.len()*48,"reference_spp":4,"checks":checks,"rows":rows});
    fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn region() -> Region {
        Region {
            min: Vec3::splat(-0.1),
            max: Vec3::splat(0.1),
            center: Vec3::ZERO,
            radius: 0.12,
            edge: 0.02,
            first: 0,
            count: 300,
            material: 0,
        }
    }
    #[test]
    fn regional_selection_respects_physical_pixels_and_hysteresis() {
        let mut state = false;
        select(&mut state, [0.8, 0.8], 300, 0);
        assert!(!state);
        select(&mut state, [0.7, 0.6], 300, 0);
        assert!(state);
        select(&mut state, [0.9, 0.9], 300, 0);
        assert!(state);
        select(&mut state, [1., 0.6], 300, 0);
        assert!(!state);
        select(&mut state, [0.5, 0.5], 6, 0);
        assert!(!state);
        let o = Object {
            asset: 0,
            position: Vec3::new(0., 0., 200.),
            scale: 1.,
        };
        let a = bound(&camera(1., RESOLUTIONS[0]), &region(), &o);
        let b = bound(&camera(1., RESOLUTIONS[1]), &region(), &o);
        assert!((a[0] - 2. * b[0]).abs() < 1e-5);
    }
    #[test]
    fn different_regions_of_one_object_can_choose_different_representations() {
        let o = Object {
            asset: 0,
            position: Vec3::new(0., 0., 200.),
            scale: 1.,
        };
        let c = camera(1., RESOLUTIONS[0]);
        let small = region();
        let mut large = small.clone();
        large.edge = 1.;
        let (mut a, mut b) = (false, false);
        select(&mut a, bound(&c, &small, &o), 300, 0);
        select(&mut b, bound(&c, &large, &o), 300, 0);
        assert!(a && !b);
    }
    #[test]
    fn glossy_regions_remain_geometry_even_when_subpixel() {
        for material in 2..=4 {
            let mut state = true;
            select(&mut state, [0.1, 0.1], 300, material);
            assert!(!state);
        }
    }
    #[test]
    fn all_cube_faces_have_orthogonal_matching_bases() {
        for (i, [u, v, w]) in AXES.into_iter().enumerate() {
            assert_eq!(face(w), i as u32);
            assert!(u.dot(v).abs() < 1e-6 && v.dot(w).abs() < 1e-6 && w.dot(u).abs() < 1e-6);
        }
    }
    #[test]
    fn projection_bound_covers_actual_edges_and_reverse_depth_separates_far_surfaces() {
        let cam = camera(0.5, RESOLUTIONS[0]);
        let r = region();
        for distance in [10., 100., 1000.] {
            let o = Object {
                asset: 0,
                position: cam.eye + Vec3::Z * distance,
                scale: 1.,
            };
            let b = bound(&cam, &r, &o);
            let a = o.position;
            let c = a + Vec3::X * r.edge;
            let project = |p| {
                let n = cam.matrix.project_point3(p);
                Vec2::new(
                    n.x * cam.size[0] as f32 * 0.5,
                    n.y * cam.size[1] as f32 * 0.5,
                )
            };
            assert!(project(a).distance(project(c)) <= b[0] + 0.001);
        }
        let forward = cam.view.inverse().transform_vector3(Vec3::NEG_Z);
        let a = cam.matrix.project_point3(cam.eye + forward * 1000.).z;
        let b = cam.matrix.project_point3(cam.eye + forward * 1000.01).z;
        assert!(a > b && b > 0.);
    }
    #[test]
    fn integrated_foreground_includes_fractional_first_object_coverage() {
        let reference = [[1., 1., 1., 0.25]];
        let empty = [[0.; 4]];
        let e = errors(&reference, &empty, &empty);
        assert_eq!(e["foreground_pixels"], 1);
        assert_eq!(e["foreground_rmse"], 1.);
    }
    #[test]
    fn foreground_metrics_do_not_hide_small_objects_in_terrain() {
        let mut a = vec![[0., 0., 0., 0.]; 100];
        let mut b = a.clone();
        a[4] = [1., 1., 1., 1.];
        b[4] = [0., 0., 0., 1.];
        let e = errors(&a, &b, &a);
        assert_eq!(e["foreground_rmse"], 1.);
        assert_eq!(e["foreground_pixels"], 1);
        assert!(e["frame_rmse"].as_f64().unwrap() < 0.11);
    }
}
