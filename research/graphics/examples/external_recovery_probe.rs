#[allow(dead_code)]
#[path = "geometry_compiler_probe/compiler.rs"]
mod compiler;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/fixtures.rs"]
mod fixtures;
#[path = "external_recovery_probe/integrated.rs"]
mod integrated;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/render.rs"]
mod render;
use anyhow::{Result, ensure};
use std::{path::Path, time::Instant};
fn bytes(c: &compiler::Compiled) -> usize {
    c.proxy.vertices.len() * 24 + c.proxy.faces.len() * 12
}
fn cheaper(a: &compiler::Compiled, b: &compiler::Compiled) -> bool {
    a.proxy.faces.len() < b.proxy.faces.len() && bytes(a) <= bytes(b)
}
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "--build output-dir | --integrate output-dir"
    );
    let dir = Path::new(&args[1]);
    if args[0] == "--integrate" {
        return integrated::run(dir);
    }
    ensure!(args[0] == "--build", "unknown mode");
    let base = compiler::decode_mesh(&std::fs::read(dir.join("stool.igm"))?)?;
    for factor in [1, 2, 10, 100] {
        let mut input = fixtures::subdivide(&base, factor);
        fixtures::shuffle(&mut input, 0x456789 + factor as u64);
        let hash = compiler::hash(&compiler::encode_mesh(&input));
        let start = Instant::now();
        let legacy = compiler::compile(&input, hash)?;
        let (refit, _) = compiler::compile_traced(&input, hash, true)?;
        let old = if cheaper(&refit, &legacy) {
            refit
        } else {
            legacy
        };
        let old_ms = start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        let (candidate, diagnostics) = compiler::compile_borders(&input, hash, true, true)?;
        let new_ms = start.elapsed().as_secs_f64() * 1000.;
        let selected = cheaper(&candidate, &old);
        let new = if selected { &candidate } else { &old };
        for (prefix, c) in [("source", &old), ("stable", new)] {
            let b = compiler::encode(c);
            compiler::decode(&b)?;
            std::fs::write(dir.join(format!("{prefix}-{factor}.igc")), b)?;
        }
        let report = serde_json::json!([
         {"selected":false,"triangles":old.proxy.faces.len(),"gpu_bytes":bytes(&old),"file_bytes":compiler::encode(&old).len(),"compile_ms":old_ms},
         {"selected":selected,"triangles":candidate.proxy.faces.len(),"gpu_bytes":bytes(&candidate),"file_bytes":compiler::encode(&candidate).len(),"compile_ms":new_ms,"diagnostics":diagnostics,"source_triangles":input.faces.len(),"source_gpu_bytes":old.source.vertices.len()*24+old.source.faces.len()*12}
        ]);
        std::fs::write(
            dir.join(format!("build-{factor}.json")),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!(
            "{factor}: {} -> {} -> {}",
            input.faces.len(),
            old.proxy.faces.len(),
            new.proxy.faces.len()
        );
    }
    Ok(())
}
