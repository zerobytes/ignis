//! Authored indexed meshes exercising the existing single-layer angular representation.
#[path = "physical_adaptive.rs"]
pub mod physical;
#[path = "regional_scene.rs"]
pub mod regional;
#[path = "rock_symbol.rs"]
pub mod symbol;
use super::*;
const ASSETS: &str = "artifacts/appearance-probe/assets";
const MATERIAL_SHADER: &str = include_str!("representative.wgsl");
type Image = Vec<[f32; 4]>;
fn decode(vertex_bytes: &[u8], index_bytes: &[u8]) -> Result<(Vec<Vertex>, Vec<u32>)> {
    ensure!(
        !vertex_bytes.is_empty() && vertex_bytes.len().is_multiple_of(40),
        "invalid vertex payload"
    );
    ensure!(
        !index_bytes.is_empty() && index_bytes.len().is_multiple_of(12),
        "invalid triangle payload"
    );
    ensure!(
        vertex_bytes.len() <= 40 * 1_000_000 && index_bytes.len() <= 12 * 1_000_000,
        "asset budget exceeded"
    );
    let vertices: Vec<Vertex> = vertex_bytes
        .chunks_exact(40)
        .map(bytemuck::pod_read_unaligned)
        .collect();
    let indices: Vec<u32> = index_bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    ensure!(
        indices.iter().all(|&i| (i as usize) < vertices.len()),
        "index outside mesh"
    );
    for v in &vertices {
        ensure!(
            v.position
                .iter()
                .chain(&v.normal)
                .chain(&v.color)
                .all(|x| x.is_finite()),
            "nonfinite vertex"
        );
        ensure!(
            Vec3::from_array(v.position).distance(CENTER) <= 1.23,
            "asset outside normalized bake domain"
        );
        ensure!(
            (Vec3::from_array(v.normal).length() - 1.).abs() < 0.01,
            "invalid normal"
        );
        ensure!(
            v.tile < 7 && v.color.iter().all(|&x| (0.0..=1.0).contains(&x)),
            "invalid material"
        );
    }
    for tri in indices.chunks_exact(3) {
        let a = vertices[tri[0] as usize].tile;
        ensure!(
            tri.iter().all(|&i| vertices[i as usize].tile == a),
            "mixed triangle material"
        );
    }
    Ok((vertices, indices))
}
fn pose(scene: &str, frame: usize, count: usize) -> Pose {
    let t = frame as f32 / (count - 1) as f32;
    let a = if scene == "seam" {
        358. + t * 4.
    } else {
        3.75 + t * 360.
    };
    Pose {
        camera: if scene == "light" { 43.75 } else { a },
        light: if scene == "camera" {
            67.25
        } else if scene == "seam" {
            a + 0.25
        } else {
            5.625 + t * 468.
        },
        distance: if scene == "near" {
            4.
        } else if scene == "zoom" {
            4. + 12. * (std::f32::consts::PI * t).sin()
        } else {
            12.
        },
    }
}
fn silhouette(reference: &Image, image: &Image) -> f64 {
    let coverage: f64 = reference.iter().map(|p| f64::from(p[3])).sum();
    reference
        .iter()
        .zip(image)
        .map(|(a, b)| f64::from((a[3] - b[3]).abs()))
        .sum::<f64>()
        / coverage.max(1.)
}
pub fn run(mode: &str) -> Result<()> {
    for name in ["rock", "chest", "tree"] {
        let root = Path::new(ASSETS).join(name);
        let source: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("asset.json"))?)?;
        let (vertices, indices) = decode(
            &fs::read(root.join("vertices.bin"))?,
            &fs::read(root.join("indices.bin"))?,
        )?;
        ensure!(
            source["triangles"].as_u64() == Some(indices.len() as u64 / 3),
            "source manifest mismatch"
        );
        let mut lab = pollster::block_on(Lab::new_source(&vertices, &indices, MATERIAL_SHADER))?;
        let packed_root = root.join("packed");
        if mode == "--bake-assets" {
            lab.bake_atlas_in(24, &vertices, &root)?;
            compressed::build_in(&root, &packed_root, &lab.source_signature(&vertices, 24))?;
            continue;
        }
        let original = lab.load_in(24, &vertices, &root)?;
        let (compact, _) = compressed::load_in(&lab, &vertices, &packed_root)?;
        let smoke = mode == "--assets-smoke";
        let mut rows = Vec::new();
        let mut convergence = Vec::new();
        let start = Instant::now();
        for scene in ["camera", "light", "both", "near", "zoom", "seam"] {
            let count = if scene == "seam" { 9 } else { 21 };
            let mut previous: Option<(Image, Image)> = None;
            for frame in 0..count {
                if smoke && ![0, count / 2, count - 1].contains(&frame) {
                    continue;
                }
                let p = pose(scene, frame, count);
                let reference = lab.capture(p, None, 0, 8)?;
                let full = lab.capture(p, None, 0, 4)?;
                let a = lab.capture(p, Some(&original), 1, 4)?;
                let b = lab.capture(p, Some(&compact), 1, 4)?;
                let native_full = lab.capture(p, None, 0, 1)?;
                let native_a = lab.capture(p, Some(&original), 1, 1)?;
                let native_b = lab.capture(p, Some(&compact), 1, 1)?;
                let fullscreen = lab.capture_options(p, Some(&compact), 1, 1, false, [0.; 2])?;
                ensure!(
                    native_b == fullscreen,
                    "proxy box clips {name}/{scene}/{frame}"
                );
                let delta = metrics::compare(&a, &b);
                // HDR peaks can amplify roundoff: report actual errors, reject gross packing faults.
                ensure!(
                    delta.rmse_surface < 0.002 && delta.alpha_rmse < 0.002,
                    "packing error {name}/{scene}/{frame}: {delta:?}"
                );
                let id = format!("{scene}_{frame:02}");
                for (method, pixels) in [
                    ("reference", &reference),
                    ("full", &full),
                    ("original", &a),
                    ("compact", &b),
                    ("native_full", &native_full),
                    ("native_original", &native_a),
                    ("native_compact", &native_b),
                ] {
                    metrics::png(&root.join(format!("{id}_{method}.png")), pixels, SIZE)?;
                }
                metrics::error_png(&root.join(format!("{id}_error.png")), &reference, &b, SIZE)?;
                metrics::error_png(&root.join(format!("{id}_packing.png")), &a, &b, SIZE)?;
                let timings = if smoke || frame % 5 == 0 || frame == count - 1 {
                    Some(lab.bench_choices_batched(
                        p,
                        [
                            (None, 0),
                            (Some(&original), 1),
                            (Some(&compact), 1),
                            (None, 0),
                        ],
                        32,
                    )?)
                } else {
                    None
                };
                if ["both", "near"].contains(&scene) && frame == 0 {
                    let fine = lab.capture(p, None, 0, 16)?;
                    convergence.push(
                        json!({"scene":scene,"64_to_256_spp":metrics::compare(&fine,&reference)}),
                    );
                }
                rows.push(json!({"id":id,"scene":scene,"frame":frame,"pose":p,"full_error":metrics::compare(&reference,&full),"original_error":metrics::compare(&reference,&a),"compact_error":metrics::compare(&reference,&b),"packing_error":delta,"native_full_error":metrics::compare(&reference,&native_full),"native_original_error":metrics::compare(&reference,&native_a),"native_compact_error":metrics::compare(&reference,&native_b),"silhouette_l1_relative":silhouette(&reference,&b),"native_silhouette_l1_relative":silhouette(&reference,&native_b),"temporal_residual":previous.as_ref().map(|(r,c)|metrics::temporal(r,c,&reference,&b)),"timings":timings}));
                previous = Some((reference, b));
                println!(
                    "{name}/{id}: compact RMSE {:.5}, native {:.5}",
                    rows.last().unwrap()["compact_error"]["rmse_surface"]
                        .as_f64()
                        .unwrap(),
                    rows.last().unwrap()["native_compact_error"]["rmse_surface"]
                        .as_f64()
                        .unwrap()
                );
            }
        }
        let p = pose("both", 0, 21);
        let a = lab.capture(p, Some(&compact), 1, 1)?;
        ensure!(
            a == lab.capture(p, Some(&compact), 1, 1)?,
            "nondeterministic capture"
        );
        ensure!(
            a == lab.capture(
                Pose {
                    camera: p.camera + 360.,
                    light: p.light + 360.,
                    ..p
                },
                Some(&compact),
                1,
                1
            )?,
            "angular wrap mismatch"
        );
        let changed = lab.capture(
            Pose {
                light: p.light + 75.,
                ..p
            },
            Some(&compact),
            1,
            1,
        )?;
        ensure!(
            a != changed && a == lab.capture(p, Some(&compact), 1, 1)?,
            "light change/return failure"
        );
        let result = json!({"schema":1,"asset":source,"adapter":lab.gpu.adapter,"geometry_bytes":std::mem::size_of_val(vertices.as_slice())+std::mem::size_of_val(indices.as_slice()),"original_bytes":original.manifest.bytes,"compact_bytes":compact.manifest.bytes,"original_load_ms":original.load_ms,"compact_load_ms":compact.load_ms,"bake_seconds":original.manifest.bake_seconds,"pack_seconds":compact.manifest.bake_seconds,"reference_spp":64,"candidate_spp":16,"timing_spp":1,"timing_batch":32,"timing_rounds":9,"size":SIZE,"seconds":start.elapsed().as_secs_f64(),"convergence":convergence,"checks":{"bounded_pairs":rows.len(),"repeat_wrap_light_return":true},"cases":rows});
        fs::write(
            root.join(if smoke { "smoke.json" } else { "report.json" }),
            serde_json::to_vec_pretty(&result)?,
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexed_source_rejects_invalid_materials_indices_and_nonfinite_data() {
        let v = Vertex {
            position: [0., 0.12, 0.],
            normal: [0., 1., 0.],
            color: [0.2; 3],
            tile: 0,
        };
        let valid = bytemuck::bytes_of(&v);
        let i = [0u32; 3];
        assert!(decode(valid, bytemuck::cast_slice(&i)).is_ok());
        assert!(decode(&valid[..39], bytemuck::cast_slice(&i)).is_err());
        assert!(decode(valid, bytemuck::cast_slice(&[1u32; 3])).is_err());
        let mut bad = v;
        bad.tile = 7;
        assert!(decode(bytemuck::bytes_of(&bad), bytemuck::cast_slice(&i)).is_err());
        bad = v;
        bad.position[0] = f32::NAN;
        assert!(decode(bytemuck::bytes_of(&bad), bytemuck::cast_slice(&i)).is_err());
        bad = v;
        bad.tile = 1;
        assert!(
            decode(
                bytemuck::cast_slice(&[v, bad]),
                bytemuck::cast_slice(&[0u32, 1, 0])
            )
            .is_err()
        );
    }
}
