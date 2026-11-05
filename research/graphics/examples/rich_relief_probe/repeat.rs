use super::{
    compile, compiler, material,
    render::{Gpu, Pose},
};
use anyhow::Result;
use std::path::Path;

pub fn run(dir: &Path) -> Result<()> {
    run_kind(dir, 0)
}
pub fn filter(dir: &Path) -> Result<()> {
    run_kind(dir, 1)
}
pub fn texel(dir: &Path) -> Result<()> {
    run_kind(dir, 2)
}
pub fn inputs(dir: &Path) -> Result<()> {
    run_kind(dir, 3)
}
pub fn analytic(dir: &Path) -> Result<()> {
    run_kind(dir, 4)
}
fn run_kind(dir: &Path, mode: u8) -> Result<()> {
    super::build::verify_inputs(dir)?;
    let mut asset = compile::decode(&std::fs::read(dir.join("regional.igl"))?)?;
    let prior = compiler::decode(&std::fs::read(dir.join("previous.igc"))?)?;
    let offset = asset.mesh.vertices.len() as u32;
    asset.mesh.vertices.extend_from_slice(&prior.proxy.vertices);
    asset.mesh.faces.extend(
        prior
            .proxy
            .faces
            .iter()
            .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset, 0]),
    );
    let full = [(0, asset.source_count)];
    let color = material::decode(&std::fs::read(dir.join("color.irm"))?)?;
    let normal = material::decode(&std::fs::read(dir.join("normal.irm"))?)?;
    let mut rows = vec![];
    for (width, height) in [(1728, 1117), (3456, 2234)] {
        let mut gpu = pollster::block_on(Gpu::new(width, height))?;
        let mesh = gpu.upload_rich(&asset.mesh);
        let mut originals = std::collections::HashMap::new();
        let variants = if mode == 4 {
            [
                "implicit",
                "analytic-lod",
                "analytic-manual",
                "analytic-manual",
                "analytic-lod",
                "implicit",
            ]
        } else if mode == 3 {
            [
                "implicit-inputs",
                "explicit-lod-inputs",
                "manual-trilinear-inputs",
                "manual-trilinear-inputs",
                "explicit-lod-inputs",
                "implicit-inputs",
            ]
        } else if mode == 2 {
            [
                "implicit",
                "explicit-lod",
                "manual-trilinear",
                "manual-trilinear",
                "explicit-lod",
                "implicit",
            ]
        } else if mode == 1 {
            [
                "implicit",
                "explicit-lod",
                "lod",
                "uv",
                "explicit-lod",
                "implicit",
            ]
        } else {
            ["implicit", "fine", "level0", "uv", "fine", "implicit"]
        };
        for (round, variant) in variants.into_iter().enumerate() {
            gpu.install_rich_variant(
                color.size,
                &color.levels,
                normal.size,
                &normal.levels,
                variant,
            );
            for (case, pose) in [
                Pose {
                    angle: 0.,
                    distance: 2.7,
                    light: 0.3,
                },
                Pose {
                    angle: 2.75,
                    distance: 43.,
                    light: 4.97,
                },
            ]
            .into_iter()
            .enumerate()
            {
                let _ = gpu.capture_selected(&mesh, pose, Some(&full))?;
                let reference = gpu.capture_selected(&mesh, pose, Some(&full))?;
                let reference_inputs = if mode == 3 {
                    Some(gpu.rich_derivatives()?)
                } else {
                    None
                };
                if round == 0 {
                    originals.insert(case, reference.clone());
                }
                let comparison = super::experiment::quality(&originals[&case], &reference);
                let mut changed = 0;
                let mut max_pixels = 0;
                let mut max_error = 0f32;
                let mut locations = std::collections::BTreeSet::new();
                let mut input_changes = 0;
                let mut color_changes_same_inputs = 0;
                let mut evidence = vec![];
                for i in 0..32 {
                    let other = Pose {
                        angle: i as f32 * 0.2,
                        distance: 6.,
                        light: i as f32 * 0.3,
                    };
                    let _ =
                        gpu.capture_region(&mesh, other, [0, 0, 1, 1], [0., 0.], Some(&full))?;
                    let pixels = gpu.capture_selected(&mesh, pose, Some(&full))?;
                    let inputs = if mode == 3 {
                        Some(gpu.rich_derivatives()?)
                    } else {
                        None
                    };
                    if let (Some(a), Some(b)) = (&reference_inputs, &inputs) {
                        input_changes += a.iter().zip(b).filter(|(a, b)| a != b).count();
                    }
                    let mut count = 0;
                    for (index, (a, b)) in pixels.iter().zip(&reference).enumerate() {
                        if a != b {
                            if let (Some(ri), Some(pi)) = (&reference_inputs, &inputs) {
                                color_changes_same_inputs += usize::from(ri[index] == pi[index]);
                                if evidence.len() < 8 {
                                    evidence.push(serde_json::json!({"iteration":i,"pixel":[index as u32%width,index as u32/width],"before":a,"after":b,"derivatives_before":ri[index],"derivatives_after":pi[index]}));
                                }
                            }
                            count += 1;
                            locations.insert((index as u32 % width, index as u32 / width));
                            for c in 0..4 {
                                max_error = max_error.max((a[c] - b[c]).abs());
                            }
                        }
                    }
                    changed += usize::from(count > 0);
                    max_pixels = max_pixels.max(count);
                }
                let row = serde_json::json!({"width":width,"case":case,"round":round,"variant":variant,"comparison_to_implicit":comparison,"captures":32,"changed_captures":changed,"max_pixels":max_pixels,"max_channel_error":max_error,"changed_locations":locations,"derivative_pixel_changes":input_changes,"color_changes_with_same_derivatives":color_changes_same_inputs,"evidence":evidence});
                println!("{row}");
                rows.push(row);
            }
        }
    }
    std::fs::write(
        dir.join(if mode == 4 {
            "analytic-repeat.json"
        } else if mode == 3 {
            "inputs-repeat.json"
        } else if mode == 2 {
            "texel-repeat.json"
        } else if mode == 1 {
            "filter-repeat.json"
        } else {
            "repeat.json"
        }),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    if mode >= 2 {
        anyhow::ensure!(
            rows.iter().all(|r| r["changed_captures"] == 0),
            "repeatability failed; diagnostic saved, no promotion"
        );
        anyhow::ensure!(
            rows.iter()
                .all(|r| r["comparison_to_implicit"]["pass"] == true),
            "material comparison failed; diagnostic saved, no promotion"
        );
    }
    Ok(())
}
