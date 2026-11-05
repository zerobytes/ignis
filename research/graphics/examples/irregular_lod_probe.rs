#[path = "irregular_lod_probe/compile.rs"]
mod compile;
#[path = "geometry_compiler_probe/compiler.rs"]
mod compiler;
#[path = "irregular_lod_probe/experiment.rs"]
mod experiment;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/render.rs"]
mod render;
#[path = "irregular_lod_probe/select.rs"]
mod select;
use anyhow::{Result, ensure};
use std::{path::Path, time::Instant};
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() >= 3,
        "--build input.igm output.igl | --run input.igl output-dir"
    );
    match args[0].as_str() {
        "--build" => {
            let bytes = std::fs::read(&args[1])?;
            let source = compiler::decode_mesh(&bytes)?;
            let start = Instant::now();
            let baseline = compiler::compile(&source, compiler::hash(&bytes))?;
            let _ = compiler::decode(&compiler::encode(&baseline))?;
            let old = baseline.proxy.faces.len();
            let source_count = source.faces.len();
            drop(baseline);
            let asset = compile::build(source, 512)?;
            let bytes = compile::encode(&asset);
            let loaded = compile::decode(&bytes)?;
            ensure!(loaded.source_count == asset.source_count, "roundtrip");
            std::fs::write(&args[2], &bytes)?;
            let report = serde_json::json!({"source_triangles":source_count,"coplanar_triangles":old,"patches":asset.patches.len(),"levels":asset.patches.iter().map(|p|p.levels.len()).sum::<usize>(),"minimum_triangles":asset.patches.iter().map(|p|p.levels.last().unwrap().count as usize/3).sum::<usize>(),"file_bytes":bytes.len(),"gpu_geometry_bytes":asset.mesh.vertices.len()*24+asset.mesh.faces.len()*12,"build_validate_write_ms":start.elapsed().as_secs_f64()*1000.,"regions":asset.patches.iter().map(|p|p.levels.iter().map(|l|serde_json::json!({"triangles":l.count/3,"error":l.error,"normal":l.normal})).collect::<Vec<_>>()).collect::<Vec<_>>()});
            std::fs::write(
                Path::new(&args[2]).with_extension("json"),
                serde_json::to_vec_pretty(&report)?,
            )?;
            println!(
                "source {source_count}, coplanar {old}, patches {}, minimum {}, {:.2}s",
                asset.patches.len(),
                report["minimum_triangles"],
                start.elapsed().as_secs_f64()
            );
        }
        "--run" | "--smoke" => experiment::run(
            Path::new(&args[1]),
            Path::new(&args[2]),
            args[0] == "--smoke",
        )?,
        _ => anyhow::bail!("unknown command"),
    }
    Ok(())
}
