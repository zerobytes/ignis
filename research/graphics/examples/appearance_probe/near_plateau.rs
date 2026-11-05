//! Near-camera diagnosis and lossless geometry locality experiment.
use super::*;
const OUTPUT: &str = "artifacts/appearance-probe/near-plateau";
const SOURCE: &str = "artifacts/appearance-probe/hybrid-scaling";
const CODES: [u32; 6] = [0, 95, 85, 105, 115, 135];
const DISTANCES: [f32; 4] = [10., 14., 28., 224.];
fn asset(f: usize, d: usize) -> Result<Asset> {
    Ok(serde_json::from_slice(&fs::read(
        Path::new(SOURCE).join(format!("f{f}_d{d}.json")),
    )?)?)
}
fn ranges(a: &Asset) -> Vec<(usize, usize)> {
    let mut ranges = a
        .levels
        .iter()
        .chain(&a.sealed)
        .chain(a.regions.iter().flat_map(|r| &r.levels))
        .map(|l| (l.first as usize, l.count as usize))
        .collect::<Vec<_>>();
    ranges.sort_unstable();
    ranges.dedup();
    ranges
}
fn morton(x: u32, y: u32) -> u32 {
    fn spread(mut x: u32) -> u32 {
        x &= 0xffff;
        x = (x | x << 8) & 0x00ff00ff;
        x = (x | x << 4) & 0x0f0f0f0f;
        x = (x | x << 2) & 0x33333333;
        (x | x << 1) & 0x55555555
    }
    spread(x) | spread(y) << 1
}
// Each draw interval owns a compact vertex window. Sort only within that interval;
// no triangle is inserted, removed, or reversed. Selection offsets remain unchanged.
fn pack(
    v: &[build::V],
    indices: &[u32],
    ranges: &[(usize, usize)],
    spatial: bool,
) -> Result<(Vec<[f32; 3]>, Vec<u32>)> {
    ensure!(
        !ranges.is_empty()
            && ranges[0].0 == 0
            && ranges.last().is_some_and(|&(f, c)| f + c == indices.len()),
        "range coverage"
    );
    ensure!(
        ranges.windows(2).all(|w| w[0].0 + w[0].1 == w[1].0),
        "range gap/overlap"
    );
    ensure!(
        indices.iter().all(|&i| (i as usize) < v.len()),
        "source index bounds"
    );
    let mut positions = Vec::new();
    let mut out = Vec::with_capacity(indices.len());
    let mut mapping = vec![u32::MAX; v.len()];
    let mut generation = vec![usize::MAX; v.len()];
    for (epoch, &(first, count)) in ranges.iter().enumerate() {
        ensure!(count % 3 == 0, "triangle alignment");
        let mut triangles = indices[first..first + count]
            .chunks_exact(3)
            .map(|t| [t[0], t[1], t[2]])
            .collect::<Vec<_>>();
        if spatial {
            triangles.sort_unstable_by_key(|t| {
                let p = t.map(|i| v[i as usize].p);
                let x = ((p[0][0] + p[1][0] + p[2][0]) / 36. + 0.5).clamp(0., 1.);
                let y = ((p[0][2] + p[1][2] + p[2][2]) / 36. + 0.5).clamp(0., 1.);
                (morton((x * 65535.) as u32, (y * 65535.) as u32), *t)
            });
        }
        for t in triangles {
            for i in t {
                let id = i as usize;
                if generation[id] != epoch {
                    generation[id] = epoch;
                    mapping[id] = u32::try_from(positions.len())?;
                    positions.push(v[id].p);
                }
                out.push(mapping[id]);
            }
        }
    }
    Ok((positions, out))
}
fn bake() -> Result<()> {
    let mut rows = Vec::new();
    for family in 0..2 {
        for density in 0..4 {
            let a = asset(family, density)?;
            let vb = fs::read(Path::new(SOURCE).join(format!("{}.vertices", a.id)))?;
            let ib = fs::read(Path::new(SOURCE).join(format!("{}.indices", a.id)))?;
            let v = bytemuck::cast_slice::<u8, build::V>(&vb);
            let indices = bytemuck::cast_slice::<u8, u32>(&ib);
            let ranges = ranges(&a);
            for (name, spatial) in [("packed", false), ("spatial", true)] {
                let start = Instant::now();
                let (p, i) = pack(v, indices, &ranges, spatial)?;
                fs::write(
                    Path::new(OUTPUT).join(format!("{}.{}.positions", a.id, name)),
                    bytemuck::cast_slice(&p),
                )?;
                fs::write(
                    Path::new(OUTPUT).join(format!("{}.{}.indices", a.id, name)),
                    bytemuck::cast_slice(&i),
                )?;
                rows.push(json!({"id":a.id,"variant":name,"source_vertex_bytes":vb.len(),"source_index_bytes":ib.len(),"positions_bytes":p.len()*12,"index_bytes":i.len()*4,"build_ms":start.elapsed().as_secs_f64()*1000.,"ranges":ranges.len()}));
            }
            println!("packed {}", a.id);
        }
    }
    fs::write(
        Path::new(OUTPUT).join("bake.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}
pub(super) fn run(mode: &str) -> Result<()> {
    fs::create_dir_all(OUTPUT)?;
    if mode == "--bake-near-plateau" {
        return bake();
    }
    if mode == "--footprint-near-plateau" {
        return footprints();
    }
    let diagnostic = mode == "--profile-near-plateau" || mode == "--isolate-near-plateau";
    let mut renderer = pollster::block_on(Renderer::new_automatic())?;
    if !diagnostic {
        automatic::load(&mut renderer)?;
    }
    if mode == "--capture-near-plateau" {
        return capture(&mut renderer);
    }
    let mut rows = Vec::new();
    for family in 0..2 {
        let assets = (0..4)
            .map(|d| asset(family, d))
            .collect::<Result<Vec<_>>>()?;
        let uploaded = assets
            .iter()
            .map(|a| {
                let mut u = renderer.upload_from(a, SOURCE)?;
                if !diagnostic {
                    renderer.upload_packed(&mut u, a, OUTPUT)?;
                }
                Ok(u)
            })
            .collect::<Result<Vec<_>>>()?;
        let inputs = uploaded.iter().zip(&assets).collect::<Vec<_>>();
        for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
            renderer.resize(size);
            let distances: &[f32] = if diagnostic {
                &[14.]
            } else if mode == "--repeat-near-plateau" {
                &[14., 224.]
            } else {
                &DISTANCES
            };
            for &distance in distances {
                let cam = render::camera_at(distance, size);
                let timings = if mode == "--profile-near-plateau" {
                    renderer.profile_stages(&inputs, distance, size)?
                } else {
                    renderer.bench_cost(
                        &inputs,
                        distance,
                        size,
                        if diagnostic { &[85, 185, 285] } else { &CODES },
                    )?
                };
                rows.push(json!({"family":family,"resolution":resolution,"size":size,"distance":distance,"triangles":assets.iter().map(|a|render::selection(a,&cam,5).2).collect::<Vec<_>>(),"codes":if diagnostic {vec![85,185,285]} else {CODES.to_vec()},"samples":timings,"adapter":renderer.gpu.adapter}));
                let name = match mode {
                    "--isolate-near-plateau" => "isolation",
                    "--profile-near-plateau" => "stages",
                    "--repeat-near-plateau" => "repeat",
                    _ => "validation",
                };
                fs::write(
                    Path::new(OUTPUT).join(format!("{name}.json")),
                    serde_json::to_vec_pretty(&rows)?,
                )?;
                println!("{name} family{family} resolution{resolution} distance{distance}");
            }
        }
    }
    Ok(())
}
fn explain(a: &Asset, cam: &Camera) -> serde_json::Value {
    let (selection, strategy, visited) = render::hybrid_choice(a, cam, true);
    let global = a.sealed.iter().map(|l| json!({"stride":l.stride,"triangles":l.count/3,"bound_px":render::precise_error(cam,l.position_error,a.bounds)})).collect::<Vec<_>>();
    let regions = a.regions.iter().map(|r| {
        let i=render::selected(&r.levels,cam,r.bounds,false);let l=&r.levels[i];
        json!({"stride":l.stride,"triangles":l.count/3,"bound_px":render::precise_error(cam,l.position_error,r.bounds),"next_coarser_bound_px":i.checked_sub(1).map(|j|render::precise_error(cam,r.levels[j].position_error,r.bounds))})
    }).collect::<Vec<_>>();
    json!({"strategy":strategy,"visited":visited,"draws":selection.1,"selected_triangles":selection.2,"normal_texels":(a.grid[0]+1)*(a.grid[1]+1),"global":global,"regional":regions})
}
fn capture(renderer: &mut Renderer) -> Result<()> {
    let mut rows = Vec::new();
    for family in 0..2 {
        for density in 0..4 {
            let a = asset(family, density)?;
            let mut u = renderer.upload_from(&a, SOURCE)?;
            renderer.upload_packed(&mut u, &a, OUTPUT)?;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                let mut previous = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
                for frame in 0..7 {
                    let distance = DISTANCES[frame.min(6 - frame)];
                    let cam = render::camera_at(distance, size);
                    let mut images: Vec<Vec<[f32; 4]>> = Vec::new();
                    let mut checks = Vec::new();
                    for (i, code) in CODES.into_iter().enumerate() {
                        let (image, _) = renderer.capture(&u, &a, &cam, code)?;
                        if i > 0 {
                            let (q, r) = quality(&images[0], &image, &previous[i - 1]);
                            previous[i - 1] = r;
                            checks.push(q);
                        }
                        images.push(image);
                    }
                    let equality = [
                        images[2] == images[3],
                        images[2] == images[4],
                        images[1] == images[5],
                    ];
                    let id = format!("f{family}_d{density}_r{resolution}_{frame}");
                    for (i, image) in images.iter().enumerate() {
                        save(
                            &Path::new(OUTPUT).join(format!("{id}_{i}.png")),
                            image,
                            size[0],
                        )?;
                    }
                    rows.push(json!({"id":id,"family":family,"density":density,"resolution":resolution,"size":size,"frame":frame,"distance":distance,"selection":explain(&a,&cam),"checks":checks,"covered_pixels":images[0].iter().filter(|p|p[3]>0.).count(),"exact_rgba32f":equality,"triangles":render::selection(&a,&cam,5).2}));
                }
            }
            fs::write(
                Path::new(OUTPUT).join("quality.json"),
                serde_json::to_vec_pretty(&rows)?,
            )?;
            println!("quality {}", a.id);
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packing_preserves_triangle_positions_winding_ranges_and_bounds() {
        let v = (0..9)
            .map(|i| build::V {
                p: [(i % 3) as f32, i as f32 * 0.01, (i / 3) as f32],
                n: [0., 1., 0.],
            })
            .collect::<Vec<_>>();
        let source = vec![0, 3, 1, 1, 3, 4, 4, 7, 5, 5, 7, 8];
        let ranges = [(0, 6), (6, 6)];
        for spatial in [false, true] {
            let (p, idx) = pack(&v, &source, &ranges, spatial).unwrap();
            assert_eq!(idx.len(), source.len());
            for (first, count) in ranges {
                let mut a = source[first..first + count]
                    .chunks(3)
                    .map(|t| {
                        t.iter()
                            .flat_map(|&i| v[i as usize].p.map(f32::to_bits))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let mut b = idx[first..first + count]
                    .chunks(3)
                    .map(|t| {
                        t.iter()
                            .flat_map(|&i| p[i as usize].map(f32::to_bits))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                a.sort();
                b.sort();
                assert_eq!(a, b);
            }
        }
        assert!(pack(&v, &source, &[(0, 3), (6, 6)], true).is_err());
        assert!(pack(&v, &[99, 0, 1], &[(0, 3)], false).is_err());
        assert_eq!(morton(0, 0), 0);
        assert_eq!(morton(1, 1), 3);
    }
}

// Offline diagnostic of source triangle footprints, not visibility or a frame selector.
fn footprints() -> Result<()> {
    let mut rows = Vec::new();
    for family in 0..2 {
        for density in 0..4 {
            let a = asset(family, density)?;
            let vb = fs::read(Path::new(SOURCE).join(format!("{}.vertices", a.id)))?;
            let ib = fs::read(Path::new(SOURCE).join(format!("{}.indices", a.id)))?;
            let vertices = bytemuck::cast_slice::<u8, build::V>(&vb);
            let indices = bytemuck::cast_slice::<u8, u32>(&ib);
            let l = a.levels.last().unwrap();
            let indices = &indices[l.first as usize..(l.first + l.count) as usize];
            let max_index = *indices.iter().max().unwrap() as usize;
            for (resolution, size) in RESOLUTIONS.into_iter().enumerate() {
                for distance in DISTANCES {
                    let cam = render::camera_at(distance, size);
                    let projected = vertices[..=max_index]
                        .iter()
                        .map(|v| {
                            let p = cam.matrix * Vec3::from_array(v.p).extend(1.);
                            [p.x / p.w, p.y / p.w, p.z / p.w, p.w]
                        })
                        .collect::<Vec<_>>();
                    let mut inside = 0u64;
                    let mut outside = 0u64;
                    let mut crossing = 0u64;
                    let mut behind = 0u64;
                    let mut bins = [0u64; 4];
                    let mut area_sum = 0f64;
                    for t in indices.chunks_exact(3) {
                        let p = [
                            projected[t[0] as usize],
                            projected[t[1] as usize],
                            projected[t[2] as usize],
                        ];
                        if p.iter()
                            .any(|v| v[3] <= 0. || v.iter().any(|x| !x.is_finite()))
                        {
                            behind += 1;
                            continue;
                        }
                        // Common outside clip halfspace: conservatively outside; mixed cases
                        // are explicitly excluded rather than counting unclipped area as pixels.
                        let out = (0..3).any(|axis| {
                            p.iter().all(|v| v[axis] < if axis == 2 { 0. } else { -1. })
                                || p.iter().all(|v| v[axis] > 1.)
                        });
                        if out {
                            outside += 1;
                            continue;
                        }
                        if p.iter()
                            .any(|v| v[0].abs() > 1. || v[1].abs() > 1. || v[2] < 0. || v[2] > 1.)
                        {
                            crossing += 1;
                            continue;
                        }
                        inside += 1;
                        let x1 = (p[1][0] as f64 - p[0][0] as f64) * size[0] as f64 / 2.;
                        let y1 = (p[1][1] as f64 - p[0][1] as f64) * size[1] as f64 / 2.;
                        let x2 = (p[2][0] as f64 - p[0][0] as f64) * size[0] as f64 / 2.;
                        let y2 = (p[2][1] as f64 - p[0][1] as f64) * size[1] as f64 / 2.;
                        let area = (x1 * y2 - y1 * x2).abs() / 2.;
                        area_sum += area;
                        bins[if area < 0.25 {
                            0
                        } else if area < 1. {
                            1
                        } else if area < 4. {
                            2
                        } else {
                            3
                        }] += 1;
                    }
                    ensure!(
                        inside + outside + crossing + behind == l.count as u64 / 3,
                        "footprint accounting"
                    );
                    ensure!(bins.iter().sum::<u64>() == inside, "footprint bins");
                    rows.push(json!({"family":family,"density":density,"resolution":resolution,"size":size,"distance":distance,"source_triangles":l.count/3,"fully_inside":inside,"outside_common_halfspace":outside,"clip_crossing":crossing,"behind_or_invalid":behind,"projected_area_bins_px2":["<0.25","0.25..1","1..4",">=4"],"counts":bins,"mean_area_px2":area_sum/inside.max(1) as f64,"occlusion_tested":false}));
                }
            }
        }
    }
    fs::write(
        Path::new(OUTPUT).join("footprints.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}
