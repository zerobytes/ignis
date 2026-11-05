#[path = "geometry_compiler_probe/compiler.rs"]
mod compiler;
#[path = "geometry_compiler_probe/experiment.rs"]
mod experiment;
#[path = "geometry_compiler_probe/fixtures.rs"]
mod fixtures;
#[path = "geometry_compiler_probe/render.rs"]
mod render;
use anyhow::{Result, ensure};
use std::{path::Path, time::Instant};
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() >= 2,
        "--generate <dir> [size] | --compile <source> <output> | --run <dir>"
    );
    match args[0].as_str() {
        "--generate" => {
            let dir = Path::new(&args[1]);
            std::fs::create_dir_all(dir)?;
            let size = args.get(2).map(|s| s.parse()).transpose()?.unwrap_or(64);
            ensure!((4..=64).contains(&size), "fixture size must be 4..64");
            for (name, m) in [
                ("stone", fixtures::stone(size)),
                ("folded", fixtures::folded(size)),
            ] {
                for factor in [1, 2, 10, 100] {
                    let mut src = fixtures::subdivide(&m, factor);
                    fixtures::shuffle(&mut src, 0x123456 + factor as u64);
                    compiler::validate(&src)?;
                    let b = compiler::encode_mesh(&src);
                    std::fs::write(dir.join(format!("{name}-{factor}.igm")), &b)?;
                    println!(
                        "{name}-{factor}: {} triangles, {} bytes",
                        src.faces.len(),
                        b.len()
                    );
                }
            }
        }
        "--compile" => {
            ensure!(args.len() == 3, "--compile requires input and output paths");
            let b = std::fs::read(&args[1])?;
            let start = Instant::now();
            let mesh = compiler::decode_mesh(&b)?;
            let compiled = compiler::compile(&mesh, compiler::hash(&b))?;
            let bytes = compiler::encode(&compiled);
            let _ = compiler::decode(&bytes)?;
            std::fs::write(&args[2], &bytes)?;
            let report = serde_json::json!({"source_triangles":compiled.source.faces.len(),"proxy_triangles":compiled.proxy.faces.len(),"regions":compiled.regions.len(),"accepted_regions":compiled.regions.iter().filter(|r|r.counts[3]==1).count(),"compile_validate_write_ms":start.elapsed().as_secs_f64()*1000.,"source_bytes":b.len(),"compiled_bytes":bytes.len(),"proxy_geometry_bytes":compiled.proxy.vertices.len()*24+compiled.proxy.faces.len()*16,"certificates_bytes":compiled.regions.len()*48,"max_position_error":compiled.regions.iter().map(|r|r.error).fold(0f32,f32::max)});
            std::fs::write(
                format!("{}.json", args[2]),
                serde_json::to_vec_pretty(&report)?,
            )?;
            println!("{report}");
        }
        "--run" | "--smoke" | "--measure" => experiment::run(
            Path::new(&args[1]),
            args[0] == "--smoke",
            args[0] == "--measure",
        )?,
        _ => anyhow::bail!("unknown command"),
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blind_recovery_and_binary_integrity() -> Result<()> {
        let base = fixtures::stone(4);
        let mut input = fixtures::subdivide(&base, 100);
        fixtures::shuffle(&mut input, 91);
        let c = compiler::compile(&input, 13)?;
        assert!(
            c.proxy.faces.len() < input.faces.len() / 3,
            "{} -> {}",
            input.faces.len(),
            c.proxy.faces.len()
        );
        let bytes = compiler::encode(&c);
        let d = compiler::decode(&bytes)?;
        assert_eq!(d.proxy.faces.len(), c.proxy.faces.len());
        for cut in [0, 5, 24, bytes.len() - 1] {
            assert!(compiler::decode(&bytes[..cut]).is_err());
        }
        let mut corrupt = bytes;
        corrupt[39] ^= 4;
        assert!(compiler::decode(&corrupt).is_err());
        Ok(())
    }
    #[test]
    fn fold_materials_and_unreduced_input() -> Result<()> {
        let m = fixtures::folded(4);
        let c = compiler::compile(&m, 0)?;
        assert!(c.proxy.faces.len() <= m.faces.len());
        assert_eq!(c.source.faces.len(), m.faces.len());
        assert!(c.regions.iter().all(|r| r.counts[2] < 3));
        let m = fixtures::subdivide(&m, 10);
        let c = compiler::compile(&m, 0)?;
        assert!(c.proxy.faces.len() < m.faces.len() / 2);
        Ok(())
    }

    #[test]
    fn subdivision_counts_and_order_independence() -> Result<()> {
        let base = fixtures::stone(4);
        for factor in [1, 2, 10, 100] {
            let mut input = fixtures::subdivide(&base, factor);
            assert_eq!(input.faces.len(), base.faces.len() * factor);
            let a = compiler::compile(&input, 0)?;
            fixtures::shuffle(&mut input, 0xdeadbeef);
            let b = compiler::compile(&input, 0)?;
            assert_eq!(a.proxy.faces.len(), b.proxy.faces.len());
            assert_eq!(a.regions.len(), b.regions.len());
            assert_eq!(a.proxy.faces.len(), base.faces.len());
        }
        Ok(())
    }
    #[test]
    fn closed_surface_stays_closed() -> Result<()> {
        let c = compiler::compile(&fixtures::subdivide(&fixtures::stone(8), 100), 0)?;
        let mut edges = std::collections::HashMap::<(u32, u32), (u32, i32)>::new();
        for t in &c.proxy.faces {
            for i in 0..3 {
                let a = t[i];
                let b = t[(i + 1) % 3];
                let e = edges.entry((a.min(b), a.max(b))).or_default();
                e.0 += 1;
                e.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            edges
                .values()
                .all(|&(count, winding)| count == 2 && winding == 0)
        );
        Ok(())
    }
    #[test]
    fn local_native_seams_do_not_disable_whole_mesh() -> Result<()> {
        let base = fixtures::folded(64);
        let c = compiler::compile(&fixtures::subdivide(&base, 10), 0)?;
        assert!(c.proxy.faces.len() < c.source.faces.len() / 3);
        // Internal boundaries must remain paired; open borders must stay on square exterior.
        let mut edges = std::collections::HashMap::<(u32, u32), usize>::new();
        for t in &c.proxy.faces {
            for i in 0..3 {
                let (a, b) = (t[i], t[(i + 1) % 3]);
                *edges.entry((a.min(b), a.max(b))).or_default() += 1;
            }
        }
        for ((a, b), count) in edges {
            assert!(count <= 2);
            if count == 1 {
                let a = c.proxy.vertices[a as usize].p;
                let b = c.proxy.vertices[b as usize].p;
                assert!(
                    (0..2).any(|i| a[i].abs() == 1. && a[i] == b[i]),
                    "internal open edge {a:?} {b:?}"
                );
            }
        }
        Ok(())
    }
    fn center_mesh(position: f32, normal: [f32; 3]) -> compiler::Mesh {
        let v = |p, n| compiler::Vertex { p, n };
        compiler::Mesh {
            vertices: vec![
                v([-1., -1., 0.], [0., 0., 1.]),
                v([1., -1., 0.], [0., 0., 1.]),
                v([1., 1., 0.], [0., 0., 1.]),
                v([-1., 1., 0.], [0., 0., 1.]),
                v([0., 0., position], normal),
            ],
            faces: vec![[0, 1, 4, 0], [1, 2, 4, 0], [2, 3, 4, 0], [3, 0, 4, 0]],
        }
    }
    #[test]
    fn real_relief_normal_detail_and_materials_are_barriers() -> Result<()> {
        assert_eq!(
            compiler::compile(&center_mesh(0., [0., 0., 1.]), 0)?
                .proxy
                .faces
                .len(),
            2
        );
        assert_eq!(
            compiler::compile(&center_mesh(0.1, [0., 0., 1.]), 0)?
                .proxy
                .faces
                .len(),
            4
        );
        assert_eq!(
            compiler::compile(&center_mesh(0., [0.3, 0., 1.]), 0)?
                .proxy
                .faces
                .len(),
            4
        );
        let mut m = center_mesh(0., [0., 0., 1.]);
        for (i, f) in m.faces.iter_mut().enumerate() {
            f[3] = (i % 2) as u32;
        }
        assert_eq!(compiler::compile(&m, 0)?.proxy.faces.len(), 4);
        Ok(())
    }
    #[test]
    fn projection_and_invalid_input_fail_closed() -> Result<()> {
        let c = compiler::compile(&fixtures::subdivide(&fixtures::stone(4), 10), 0)?;
        let b = render::Bound::new(&c);
        let near = render::Pose {
            angle: 0.4,
            distance: 3.2,
            light: 0.,
        };
        let far = render::Pose {
            distance: 10.,
            ..near
        };
        assert!(b.pixels(near, 2234) > b.pixels(far, 2234));
        assert_eq!(b.pixels(near, 2234), 2. * b.pixels(near, 1117));
        assert!(b.choose(near, 2234));
        assert!(!b.choose(
            render::Pose {
                distance: 0.1,
                ..near
            },
            2234
        ));
        let mut m = center_mesh(0., [0., 0., 1.]);
        m.faces[0][0] = 999;
        assert!(compiler::compile(&m, 0).is_err());
        m.faces[0][0] = 0;
        m.vertices[0].p[0] = f32::NAN;
        assert!(compiler::compile(&m, 0).is_err());
        Ok(())
    }
}
