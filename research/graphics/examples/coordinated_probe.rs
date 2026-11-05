#[allow(dead_code)]
#[path = "irregular_lod_probe/compile.rs"]
mod compile;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/compiler.rs"]
mod compiler;
#[path = "coordinated_probe/experiment.rs"]
mod experiment;
#[allow(dead_code)]
#[path = "normal_field_probe/field.rs"]
mod field;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/fixtures.rs"]
mod fixtures;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/render.rs"]
mod render;
#[allow(dead_code)]
#[path = "irregular_lod_probe/select.rs"]
mod select;
use anyhow::{Result, ensure};
use std::{path::Path, time::Instant};
fn main() -> Result<()> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        a.len() >= 2,
        "--build base.igm output-dir factor | --run output-dir | --smoke output-dir"
    );
    if a[0] == "--build" {
        ensure!(a.len() == 4, "build arguments");
        let factor: usize = a[3].parse()?;
        ensure!([1, 2, 10, 100].contains(&factor), "factor");
        let dir = Path::new(&a[2]);
        std::fs::create_dir_all(dir)?;
        let base = compiler::decode_mesh(&std::fs::read(&a[1])?)?;
        let start = Instant::now();
        let mut input = fixtures::subdivide(&base, factor);
        fixtures::shuffle(&mut input, 0x456789 + factor as u64);
        let count = input.faces.len();
        let hash = compiler::hash(bytemuck::cast_slice(&input.vertices));
        let compiled = compiler::compile(&input, hash)?;
        drop(input);
        let recovered = compiled.proxy.faces.len();
        std::fs::write(
            dir.join(format!("source-{factor}.igc")),
            compiler::encode(&compiled),
        )?;
        let recovery_ms = start.elapsed().as_secs_f64() * 1000.;
        let mut reports = vec![];
        let small = compile::build(compiled.proxy.clone(), 64)?;
        reports.push(serde_json::json!({"name":"small_unmerged_control","regions":small.patches.len(),"minimum_triangles":small.patches.iter().map(|p|p.levels.last().unwrap().count as usize/3).sum::<usize>()}));
        drop(small);
        for coordinated in [false, true] {
            let start = Instant::now();
            let mut asset = if coordinated {
                compile::build_coordinated(compiled.proxy.clone(), 64)?
            } else {
                compile::build(compiled.proxy.clone(), 512)?
            };
            add_recovery_error(&mut asset, render::Bound::new(&compiled).error);
            let patches = asset.patches.len();
            let minimum: usize = asset
                .patches
                .iter()
                .map(|p| p.levels.last().unwrap().count as usize / 3)
                .sum();
            let name = if coordinated { "coordinated" } else { "old" };
            let field = field::build(asset, 64)?;
            let bytes = field::encode(&field);
            let check = field::decode(&bytes)?;
            ensure!(
                check.asset.source_count == field.asset.source_count,
                "roundtrip"
            );
            std::fs::write(dir.join(format!("{name}-{factor}.inf")), &bytes)?;
            reports.push(serde_json::json!({"name":name,"regions":patches,"minimum_triangles":minimum,"file_bytes":bytes.len(),"geometry_uv_bytes":field.asset.mesh.vertices.len()*32+field.asset.mesh.faces.len()*12,"texture_bytes":field.pixels.len(),"build_ms":start.elapsed().as_secs_f64()*1000.}));
        }
        let report = serde_json::json!({"factor":factor,"source_triangles":count,"recovered_triangles":recovered,"generation_recovery_write_ms":recovery_ms,"source_gpu_bytes":compiled.source.vertices.len()*24+compiled.source.faces.len()*12,"recovered_gpu_bytes":compiled.proxy.vertices.len()*24+compiled.proxy.faces.len()*12,"variants":reports});
        std::fs::write(
            dir.join(format!("build-{factor}.json")),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!("{report}");
    } else {
        ensure!(a[0] == "--run" || a[0] == "--smoke", "mode");
        experiment::run(Path::new(&a[1]), a[0] == "--smoke")?;
    }
    Ok(())
}

fn add_recovery_error(asset: &mut compile::Asset, error: f32) {
    for p in &mut asset.patches {
        for i in 0..3 {
            p.low[i] -= error;
            p.high[i] += error;
        }
        for l in p.levels.iter_mut().skip(1) {
            l.error = (l.error + error).next_up();
        }
    }
}
