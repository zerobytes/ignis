//! Whole-object selection in physical pixels. Diagnostic visibility is never in the selector.
use super::*;
use glam::Vec2;
const OUT: &str = "artifacts/appearance-probe/adaptive-assets";
const WINDOW: u32 = 512;
const VIEWPORTS: [[u32; 2]; 2] = [[3456, 2234], [1728, 1117]];
const ENTER: f32 = 0.75;
const EXIT: f32 = 1.;
fn camera(p: Pose) -> Mat4 {
    Mat4::look_at_rh(
        CENTER + direction(p.camera, 55.) * p.distance,
        CENTER,
        Vec3::Y,
    )
}
pub(in crate::gpu::views) fn window_matrix(p: Pose, viewport: [u32; 2], size: u32) -> Mat4 {
    Mat4::from_scale(Vec3::new(
        viewport[0] as f32 / size as f32,
        viewport[1] as f32 / size as f32,
        1.,
    )) * Mat4::perspective_rh(
        45f32.to_radians(),
        viewport[0] as f32 / viewport[1] as f32,
        1.,
        100_000.,
    ) * camera(p)
}
fn pose_at(distance: f32) -> Pose {
    Pose {
        camera: 37.5,
        light: 67.25,
        distance,
    }
}
#[derive(Clone, Copy, Serialize)]
struct Bound {
    min: Vec3,
    max: Vec3,
    edge: f32,
}
impl Bound {
    fn build(vertices: &[Vertex], indices: &[u32]) -> Self {
        let mut b = Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
            edge: 0.,
        };
        for v in vertices {
            let p = Vec3::from_array(v.position);
            b.min = b.min.min(p);
            b.max = b.max.max(p);
        }
        for t in indices.chunks_exact(3) {
            for i in 0..3 {
                b.edge =
                    b.edge
                        .max(Vec3::from_array(vertices[t[i] as usize].position).distance(
                            Vec3::from_array(vertices[t[(i + 1) % 3] as usize].position),
                        ));
            }
        }
        b
    }
    fn pixels(self, p: Pose, viewport: [u32; 2]) -> [f32; 2] {
        let v = camera(p);
        let mut depth = f32::INFINITY;
        let mut xy = Vec2::ZERO;
        for x in [self.min.x, self.max.x] {
            for y in [self.min.y, self.max.y] {
                for z in [self.min.z, self.max.z] {
                    let q = v.transform_point3(Vec3::new(x, y, z));
                    depth = depth.min(-q.z);
                    xy = xy.max(q.truncate().abs());
                }
            }
        }
        if !depth.is_finite() || depth <= 1. || viewport.contains(&0) {
            return [f32::INFINITY; 2];
        }
        // Lipschitz bound of perspective projection over the convex bounding box.
        let scale = viewport[1] as f32 / (2. * (22.5f32.to_radians()).tan()) / depth
            * (1. + xy.length_squared() / depth.powi(2)).sqrt();
        // Two-texel bilinear support in the planar bake domain, not a bound on height discontinuities.
        [
            self.edge * scale,
            2. * 2f32.sqrt() * (2. * EXTENT / TEXELS as f32) * scale,
        ]
    }
    fn crossing(self, viewport: [u32; 2], threshold: f32) -> f32 {
        let (mut lo, mut hi) = (16., 90_000.);
        for _ in 0..40 {
            let mid = (lo + hi) * 0.5;
            if self
                .pixels(pose_at(mid), viewport)
                .into_iter()
                .fold(0., f32::max)
                > threshold
            {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        hi
    }
}
#[derive(Default)]
struct Selector {
    summary: bool,
}
impl Selector {
    fn update(&mut self, pixels: [f32; 2]) -> bool {
        if pixels.iter().any(|v| !v.is_finite() || *v >= EXIT) {
            self.summary = false;
        } else if pixels.iter().all(|v| *v < ENTER) {
            self.summary = true;
        }
        self.summary
    }
}
fn project(m: Mat4, p: Vec3, size: u32) -> Vec3 {
    let q = m * p.extend(1.);
    let n = q.truncate() / q.w;
    Vec3::new(
        (n.x * 0.5 + 0.5) * size as f32,
        (0.5 - n.y * 0.5) * size as f32,
        n.z,
    )
}
// Fixed 4x4 visibility samples per physical pixel, z-buffered. A sampled lower bound,
// NOT exact triangle overlap, submitted triangles/area, or work done by the selector.
fn density(vertices: &[Vertex], indices: &[u32], m: Mat4, size: u32) -> serde_json::Value {
    const S: usize = 4;
    let n = size as usize * S;
    let mut depth = vec![f32::INFINITY; n * n];
    let mut ids = vec![u32::MAX; n * n];
    let points: Vec<_> = vertices
        .iter()
        .map(|v| project(m, Vec3::from_array(v.position), n as u32))
        .collect();
    let mut edges = Vec::with_capacity(indices.len() / 3);
    for (id, t) in indices.chunks_exact(3).enumerate() {
        let [a, b, c] = [
            points[t[0] as usize],
            points[t[1] as usize],
            points[t[2] as usize],
        ];
        edges.push(
            a.truncate()
                .distance(b.truncate())
                .max(b.truncate().distance(c.truncate()))
                .max(c.truncate().distance(a.truncate()))
                / S as f32,
        );
        let ab = (b - a).truncate();
        let ac = (c - a).truncate();
        let det = ab.perp_dot(ac);
        if det.abs() < 1e-12 {
            continue;
        }
        let lo = a.min(b).min(c).truncate().floor().max(Vec2::ZERO);
        let hi = a.max(b).max(c).truncate().ceil().min(Vec2::splat(n as f32));
        for y in lo.y as usize..hi.y as usize {
            for x in lo.x as usize..hi.x as usize {
                let ap = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - a.truncate();
                let u = ap.perp_dot(ac) / det;
                let v = ab.perp_dot(ap) / det;
                if u >= 0. && v >= 0. && u + v <= 1. {
                    let z = a.z + u * (b.z - a.z) + v * (c.z - a.z);
                    let k = y * n + x;
                    if (0.0..=1.).contains(&z) && z < depth[k] {
                        depth[k] = z;
                        ids[k] = id as u32;
                    }
                }
            }
        }
    }
    let mut visible = vec![false; indices.len() / 3];
    let (mut covered, mut multiple, mut sum) = (0usize, 0usize, 0usize);
    for y in 0..size as usize {
        for x in 0..size as usize {
            let mut local = [u32::MAX; S * S];
            for sy in 0..S {
                for sx in 0..S {
                    local[sy * S + sx] = ids[(y * S + sy) * n + x * S + sx];
                }
            }
            local.sort_unstable();
            let mut last = u32::MAX;
            let mut count = 0;
            for id in local {
                if id != u32::MAX && id != last {
                    count += 1;
                    visible[id as usize] = true;
                    last = id;
                }
            }
            if count > 0 {
                covered += 1;
                sum += count;
                if count > 1 {
                    multiple += 1;
                }
            }
        }
    }
    json!({"samples_per_pixel":S*S,"covered_pixels":covered,"sampled_visible_triangles":visible.iter().filter(|v|**v).count(),"mean_ids_per_covered_pixel":sum as f64/covered.max(1) as f64,"fraction_multiple":multiple as f64/covered.max(1) as f64,"max_visible_edge_px":edges.iter().zip(&visible).filter(|(_,v)|**v).map(|(e,_)|*e).fold(0.,f32::max)})
}
pub fn run() -> Result<()> {
    for name in ["rock", "chest", "tree"] {
        let source = Path::new(ASSETS).join(name);
        let out = Path::new(OUT).join(name);
        fs::create_dir_all(&out)?;
        let (vertices, indices) = decode(
            &fs::read(source.join("vertices.bin"))?,
            &fs::read(source.join("indices.bin"))?,
        )?;
        let bound = Bound::build(&vertices, &indices);
        let mut lab = pollster::block_on(Lab::new_source(&vertices, &indices, MATERIAL_SHADER))?;
        lab.output_size = WINDOW;
        let (atlas, _) = compressed::load_in(&lab, &vertices, &source.join("packed"))?;
        let end = bound.crossing(VIEWPORTS[0], ENTER) * 1.5;
        let mut distances: Vec<_> = (0..13)
            .map(|i| 16. * (end / 16.).powf(i as f32 / 12.))
            .collect();
        for vp in VIEWPORTS {
            for t in [ENTER, EXIT] {
                for factor in [0.98, 1.02] {
                    distances.push(bound.crossing(vp, t) * factor);
                }
            }
        }
        distances.sort_by(f32::total_cmp);
        distances.dedup_by(|a, b| (*a - *b).abs() < 0.001);
        let mut rows = Vec::new();
        let mut convergence = Vec::new();
        for (resolution, viewport) in VIEWPORTS.into_iter().enumerate() {
            lab.physical_viewport = Some(viewport);
            let mut selector = Selector::default();
            for (way, steps) in [
                ("out", distances.clone()),
                ("in", distances.iter().rev().copied().collect()),
            ] {
                let mut previous: Option<(Image, Image)> = None;
                let mut prior_choice = None;
                for (frame, distance) in steps.into_iter().enumerate() {
                    let p = pose_at(distance);
                    let pixels = bound.pixels(p, viewport);
                    let summary = selector.update(pixels);
                    ensure!(
                        !summary || pixels.iter().all(|v| *v < EXIT),
                        "non-subpixel summary"
                    );
                    let id = format!("r{resolution}_{way}_{frame:02}");
                    let reference = lab.capture(p, None, 0, 4)?;
                    let full = lab.capture(p, None, 0, 1)?;
                    let forced = lab.capture(p, Some(&atlas), 1, 1)?;
                    let chosen = if summary { Some(&atlas) } else { None };
                    let adaptive = lab.capture(p, chosen, 1, 1)?;
                    ensure!(
                        adaptive == *if summary { &forced } else { &full },
                        "adaptive draw differs from chosen representation"
                    );
                    let switched = prior_choice.is_some_and(|s| s != summary);
                    let diagnostic = density(
                        &vertices,
                        &indices,
                        window_matrix(p, viewport, WINDOW),
                        WINDOW,
                    );
                    ensure!(
                        diagnostic["max_visible_edge_px"].as_f64().unwrap()
                            <= f64::from(pixels[0]) * 1.001 + 0.001,
                        "bound underestimates projected edge"
                    );
                    let timings = if frame == 0 || frame == distances.len() - 1 || switched {
                        Some(lab.bench_choices_batched(
                            p,
                            [(None, 0), (Some(&atlas), 1), (chosen, 1), (None, 0)],
                            32,
                        )?)
                    } else {
                        None
                    };
                    if way == "out" && (frame == 0 || switched) {
                        let fine = lab.capture(p, None, 0, 8)?;
                        convergence.push(
                            json!({"id":id,"16_to_64_spp":metrics::compare(&fine,&reference)}),
                        );
                    }
                    for (suffix, img) in [
                        ("reference", &reference),
                        ("full", &full),
                        ("forced", &forced),
                        ("adaptive", &adaptive),
                    ] {
                        metrics::png(&out.join(format!("{id}_{suffix}.png")), img, WINDOW)?;
                    }
                    metrics::error_png(
                        &out.join(format!("{id}_error.png")),
                        &reference,
                        &adaptive,
                        WINDOW,
                    )?;
                    rows.push(json!({"id":id,"resolution":resolution,"viewport":viewport,"way":way,"frame":frame,"pose":p,"summary":summary,"switched":switched,"triangle_bound_px":pixels[0],"filter_bound_px":pixels[1],"drawn_triangles":if summary {2}else{indices.len()/3},"density":diagnostic,"full_error":metrics::compare(&reference,&full),"forced_error":metrics::compare(&reference,&forced),"adaptive_error":metrics::compare(&reference,&adaptive),"replacement_delta":metrics::compare(&full,&forced),"temporal_residual":previous.as_ref().map(|(r,c)|metrics::temporal(r,c,&reference,&adaptive)),"timings":timings}));
                    previous = Some((reference, adaptive));
                    prior_choice = Some(summary);
                    println!(
                        "{name}/{id}: d={distance:.2}, triangle={:.3}px filter={:.3}px summary={summary}",
                        pixels[0], pixels[1]
                    );
                }
            }
        }
        fs::write(
            out.join("report.json"),
            serde_json::to_vec_pretty(
                &json!({"schema":1,"asset":name,"adapter":lab.gpu.adapter,"window":WINDOW,"viewports":VIEWPORTS,"enter_px":ENTER,"exit_px":EXIT,"reference_spp":16,"candidate_spp":1,"timing_batch":32,"bound":bound,"geometry_bytes":std::mem::size_of_val(vertices.as_slice())+std::mem::size_of_val(indices.as_slice()),"atlas_bytes":atlas.manifest.bytes,"triangles":indices.len()/3,"convergence":convergence,"cases":rows}),
            )?,
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn v(p: [f32; 3]) -> Vertex {
        Vertex {
            position: p,
            normal: [0., 0., 1.],
            color: [0.5; 3],
            tile: 0,
        }
    }
    #[test]
    fn crop_preserves_full_viewport_physical_coordinates() {
        let vp = VIEWPORTS[0];
        let p = pose_at(16.);
        let full = Mat4::perspective_rh(
            45f32.to_radians(),
            vp[0] as f32 / vp[1] as f32,
            1.,
            100_000.,
        ) * camera(p);
        for point in [CENTER, Vec3::new(0.4, 0.6, 0.2), Vec3::new(-1., 0., 0.)] {
            let n = full.project_point3(point);
            let physical = Vec2::new(n.x * vp[0] as f32 / 2., -n.y * vp[1] as f32 / 2.);
            let crop = project(window_matrix(p, vp, WINDOW), point, WINDOW).truncate()
                - Vec2::splat(WINDOW as f32 / 2.);
            assert!(physical.distance(crop) < 0.001);
        }
    }
    #[test]
    fn bound_covers_edges_and_scales_with_physical_resolution() {
        let vs = [v([-1., 0., 0.]), v([1., 0.4, 0.]), v([0., 1., 0.8])];
        let ids = [0, 1, 2];
        let b = Bound::build(&vs, &ids);
        for d in [3., 16., 64., 1000.] {
            for angle in [0., 37.5, 180.] {
                let p = Pose {
                    camera: angle,
                    ..pose_at(d)
                };
                let hi = b.pixels(p, VIEWPORTS[0]);
                let lo = b.pixels(p, VIEWPORTS[1]);
                assert!((hi[0] - lo[0] * 2.).abs() < 1e-4);
                for i in 0..3 {
                    let a = project(
                        window_matrix(p, VIEWPORTS[0], WINDOW),
                        Vec3::from_array(vs[i].position),
                        WINDOW,
                    );
                    let c = project(
                        window_matrix(p, VIEWPORTS[0], WINDOW),
                        Vec3::from_array(vs[(i + 1) % 3].position),
                        WINDOW,
                    );
                    assert!(a.truncate().distance(c.truncate()) <= hi[0] + 0.001);
                }
            }
        }
    }
    #[test]
    fn hysteresis_resize_and_invalid_projection_force_geometry() {
        let mut s = Selector::default();
        assert!(!s.update([0.9, 0.8]));
        assert!(s.update([0.7, 0.7]));
        assert!(s.update([0.9, 0.8]));
        assert!(!s.update([1., 0.8]));
        assert!(!s.update([0.6, 1.]));
        assert!(s.update([0.6, 0.4]));
        assert!(!s.update([1.2, 0.8]));
        assert!(!s.update([f32::NAN, 0.]));
        let b = Bound {
            min: Vec3::splat(-1.),
            max: Vec3::splat(1.),
            edge: 1.,
        };
        assert!(!s.update(b.pixels(pose_at(1.), VIEWPORTS[0])));
    }
    #[test]
    fn visibility_counts_front_surface_not_hidden_triangles() {
        let vs = [
            v([-0.8, -0.8, 0.2]),
            v([0.8, -0.8, 0.2]),
            v([0., 0.8, 0.2]),
            v([-0.8, -0.8, 0.8]),
            v([0.8, -0.8, 0.8]),
            v([0., 0.8, 0.8]),
        ];
        let d = density(&vs, &[0, 1, 2, 3, 4, 5], Mat4::IDENTITY, 8);
        assert_eq!(d["sampled_visible_triangles"], 1);
        assert_eq!(d["mean_ids_per_covered_pixel"], 1.);
        assert_eq!(d["fraction_multiple"], 0.);
    }
}
