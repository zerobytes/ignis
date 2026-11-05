#[path = "rich_relief_probe/build.rs"]
mod build;
#[allow(dead_code)]
#[path = "recovery_probe/certificates.rs"]
mod certificates;
#[path = "rich_relief_probe/compact.rs"]
mod compact;
#[allow(dead_code)]
#[path = "irregular_lod_probe/compile.rs"]
mod compile;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/compiler.rs"]
mod compiler;
#[path = "rich_relief_probe/cost.rs"]
mod cost;
#[path = "rich_relief_probe/experiment.rs"]
mod experiment;
#[path = "rich_relief_probe/material.rs"]
mod material;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/render.rs"]
mod render;
#[path = "rich_relief_probe/repeat.rs"]
mod repeat;
#[allow(dead_code)]
#[path = "irregular_lod_probe/select.rs"]
mod select;
#[path = "rich_relief_probe/tangent.rs"]
mod tangent;
use anyhow::{Result, ensure};
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "--build|--verify|--smoke|--run|--repeat|--filter-repeat|--texel-repeat|--inputs-repeat|--analytic-repeat|--analytic-run|--compact-build|--compact-run|--tangent-build|--tangent-run|--refine|--refined-smoke|--refined-run|--auto-run|--audit-auto output-dir"
    );
    let dir = std::path::Path::new(&args[1]);
    std::fs::create_dir_all(dir)?;
    match args[0].as_str() {
        "--build" => build::run(dir),
        "--verify" => build::verify_inputs(dir),
        "--smoke" => experiment::run(dir, true),
        "--run" => experiment::run(dir, false),
        "--repeat" => repeat::run(dir),
        "--filter-repeat" => repeat::filter(dir),
        "--texel-repeat" => repeat::texel(dir),
        "--inputs-repeat" => repeat::inputs(dir),
        "--analytic-repeat" => repeat::analytic(dir),
        "--analytic-run" => analytic::run(dir),
        "--compact-build" => compact::build(dir),
        "--compact-run" => compact::run(dir),
        "--tangent-build" => tangent::build(dir),
        "--tangent-run" => tangent::run(dir),
        "--refine" => build::refine(dir),
        "--auto-run" => experiment::run_auto(dir),
        "--audit-auto" => experiment::audit_auto(dir),
        "--refined-run" => experiment::run_refined(dir, false),
        "--refined-smoke" => experiment::run_refined(dir, true),
        _ => anyhow::bail!("unknown command"),
    }
}
#[path = "rich_relief_probe/analytic.rs"]
mod analytic;
