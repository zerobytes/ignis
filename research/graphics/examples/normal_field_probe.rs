#[allow(dead_code)]
#[path = "irregular_lod_probe/compile.rs"]
mod compile;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/compiler.rs"]
mod compiler;
#[path = "normal_field_probe/experiment.rs"]
mod experiment;
#[path = "normal_field_probe/field.rs"]
mod field;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/render.rs"]
mod render;
#[allow(dead_code)]
#[path = "irregular_lod_probe/select.rs"]
mod select;
use anyhow::{Result, ensure};
fn main() -> Result<()> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        a.len() >= 3,
        "--build input.igl output.inf resolution | --run output-dir input.inf..."
    );
    if a[0] == "--build" {
        ensure!(a.len() == 4, "build requires input, output and resolution");
        let start = std::time::Instant::now();
        let src = compile::decode(&std::fs::read(&a[1])?)?;
        let f = field::build(src, a[3].parse()?)?;
        let b = field::encode(&f);
        let check = field::decode(&b)?;
        ensure!(field::encode(&check) == b, "field roundtrip");
        std::fs::write(&a[2], b)?;
        println!(
            "atlas {}x{}, {} vertices, {:.3}s",
            f.width,
            f.height,
            f.uv.len(),
            start.elapsed().as_secs_f64()
        );
    } else {
        ensure!(a[0] == "--run" || a[0] == "--smoke", "unknown command");
        experiment::run(std::path::Path::new(&a[1]), &a[2..], a[0] == "--smoke")?;
    }
    Ok(())
}
