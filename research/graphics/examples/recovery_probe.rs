#[path = "recovery_probe/certificates.rs"]
mod certificates;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/compiler.rs"]
mod compiler;
#[path = "recovery_probe/experiment.rs"]
mod experiment;
#[path = "recovery_probe/rays.rs"]
mod rays;
#[allow(dead_code)]
#[path = "geometry_compiler_probe/render.rs"]
mod render;
use anyhow::{Result, ensure};
use std::{path::Path, time::Instant};
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "input.igc output-dir factor | --run old-dir output-dir"
    );
    if args[0] == "--certificates" {
        return certificates::build_directory(Path::new(&args[1]), Path::new(&args[2]));
    }
    if args[0] == "--ray" {
        return rays::run(Path::new(&args[1]), Path::new(&args[2]));
    }
    if args[0] == "--run" {
        return experiment::run(Path::new(&args[1]), Path::new(&args[2]));
    }
    let borders = std::env::var_os("RECOVERY_BORDERS").is_some();
    let dir = Path::new(&args[1]);
    std::fs::create_dir_all(dir)?;
    let old = compiler::decode(&std::fs::read(&args[0])?)?;
    let mut reports = vec![];
    for stable in [false, true] {
        let start = Instant::now();
        let (c, diagnostics) = if borders && !stable {
            (
                compiler::decode(&compiler::encode(&old))?,
                compiler::Diagnostics::default(),
            )
        } else {
            compiler::compile_borders(&old.source, old.source_hash, stable, borders)?
        };
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.;
        let compile_ms = if borders && !stable { 0. } else { elapsed_ms };
        let baseline_copy_ms = if borders && !stable { elapsed_ms } else { 0. };
        if !stable {
            ensure!(
                compiler::encode(&c) == compiler::encode(&old),
                "legacy changed"
            );
        }
        let bytes = compiler::encode(&c);
        compiler::decode(&bytes)?;
        let selected = stable
            && c.proxy.faces.len() < old.proxy.faces.len()
            && c.proxy.vertices.len() * 24 + c.proxy.faces.len() * 12
                <= old.proxy.vertices.len() * 24 + old.proxy.faces.len() * 12;
        if stable {
            std::fs::write(
                dir.join(format!("stable-{}.igc", args[2])),
                if selected {
                    bytes.clone()
                } else {
                    compiler::encode(&old)
                },
            )?;
        }
        reports.push(serde_json::json!({"stable":stable,"baseline_reused":borders && !stable,"selected":selected,"source_triangles":c.source.faces.len(),"triangles":c.proxy.faces.len(),"regions":c.regions.len(),"native_faces":c.regions.iter().filter(|r|r.counts[3]==0).map(|r|r.counts[0] as usize).sum::<usize>(),"gpu_bytes":c.proxy.vertices.len()*24+c.proxy.faces.len()*12,"file_bytes":bytes.len(),"compile_ms":compile_ms,"baseline_copy_ms":baseline_copy_ms,"diagnostics":diagnostics}));
    }
    std::fs::write(
        dir.join(format!("build-{}.json", args[2])),
        serde_json::to_vec_pretty(&reports)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&reports)?);
    Ok(())
}

#[cfg(test)]
#[path = "geometry_compiler_probe/fixtures.rs"]
mod fixtures;
#[cfg(test)]
mod tests {
    use super::*;
    fn triangle_keys(
        m: &compiler::Mesh,
    ) -> std::collections::BTreeSet<([([u32; 3], [u32; 3]); 3], u32)> {
        m.faces
            .iter()
            .map(|t| {
                let mut corners = [t[0], t[1], t[2]].map(|i| {
                    let v = m.vertices[i as usize];
                    (
                        v.p.map(|x| if x == 0. { 0 } else { x.to_bits() }),
                        v.n.map(|x| if x == 0. { 0 } else { x.to_bits() }),
                    )
                });
                let first = (0..3).min_by_key(|&i| corners[i]).unwrap();
                corners.rotate_left(first);
                (corners, t[3])
            })
            .collect()
    }
    #[test]
    fn asymmetric_borders_recover_surface_and_materials() -> Result<()> {
        for mut base in [fixtures::stone(4), fixtures::folded(8)] {
            fixtures::shuffle(&mut base, 119);
            for factor in [1, 2, 10, 100] {
                let mut input = fixtures::subdivide(&base, factor);
                let (a, stats) = compiler::compile_borders(&input, 0, true, true)?;
                eprintln!(
                    "base={} factor={factor} triangles={} shortcuts={}",
                    base.faces.len(),
                    a.proxy.faces.len(),
                    stats.shortcut_collapses
                );
                assert_eq!(a.proxy.faces.len(), base.faces.len());
                assert_eq!(triangle_keys(&a.proxy), triangle_keys(&base));
                fixtures::shuffle(&mut input, 771);
                let (b, _) = compiler::compile_borders(&input, 0, true, true)?;
                assert_eq!(triangle_keys(&a.proxy), triangle_keys(&b.proxy));
                compiler::decode(&compiler::encode(&a))?;
            }
        }
        Ok(())
    }
    #[test]
    fn asymmetric_border_keeps_real_displacement_and_normal_detail() -> Result<()> {
        let base = compiler::Mesh {
            vertices: [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.2]]
                .map(|p| compiler::Vertex { p, n: [0., 0., 1.] })
                .to_vec(),
            faces: vec![[0, 1, 2, 0], [2, 3, 0, 1]],
        };
        for normal_detail in [false, true] {
            let mut input = fixtures::subdivide(&base, 2);
            let mut detail = None;
            for v in &mut input.vertices {
                if v.p == [0., 0., 0.] {
                    if normal_detail {
                        v.n = [0.3, 0., 1.]
                    } else {
                        v.p[2] = 0.02;
                    }
                    detail = Some(*v);
                }
            }
            let detail = detail.unwrap();
            let (c, _) = compiler::compile_borders(&input, 0, true, true)?;
            assert!(
                c.proxy
                    .vertices
                    .iter()
                    .any(|v| v.p == detail.p && v.n == detail.n)
            );
            assert!(c.proxy.faces.iter().any(|t| t[3] == 0));
            assert!(c.proxy.faces.iter().any(|t| t[3] == 1));
            assert!(c.regions.iter().any(|r| r.counts[3] == 0));
        }
        Ok(())
    }
    #[test]
    fn shortcut_merge_preserves_closed_oriented_surface() -> Result<()> {
        let mut base = fixtures::stone(4);
        fixtures::shuffle(&mut base, 119);
        let input = fixtures::subdivide(&base, 2);
        let (old, _) = compiler::compile_traced(&input, 0, true)?;
        let (new, stats) = compiler::compile_borders(&input, 0, true, true)?;
        assert!(stats.shortcut_collapses > 0);
        assert!(new.proxy.faces.len() < old.proxy.faces.len());
        let mut edges = std::collections::HashMap::new();
        for t in &new.proxy.faces {
            for j in 0..3 {
                let x = t[j];
                let y = t[(j + 1) % 3];
                let e = edges.entry((x.min(y), x.max(y))).or_insert((0, 0));
                e.0 += 1;
                e.1 += if x < y { 1 } else { -1 };
            }
        }
        assert!(edges.values().all(|&e| e == (2, 0)));
        Ok(())
    }
    #[test]
    fn refit_preserves_detail_materials_and_order() -> Result<()> {
        let base = fixtures::stone(4);
        let input = fixtures::subdivide(&base, 100);
        let (a, stats) = compiler::compile_traced(&input, 0, true)?;
        assert!(stats.refits > 0);
        assert_eq!(a.proxy.faces.len(), base.faces.len());
        let mut shuffled = input;
        fixtures::shuffle(&mut shuffled, 331);
        let (b, _) = compiler::compile_traced(&shuffled, 0, true)?;
        assert_eq!(a.regions.len(), b.regions.len());
        assert_eq!(a.proxy.faces.len(), b.proxy.faces.len());
        assert!(a.regions.iter().all(|r| r.normal_error <= 0.000101));
        let mut edges = std::collections::HashMap::new();
        for t in &a.proxy.faces {
            for j in 0..3 {
                let x = t[j];
                let y = t[(j + 1) % 3];
                let e = edges.entry((x.min(y), x.max(y))).or_insert((0, 0));
                e.0 += 1;
                e.1 += if x < y { 1 } else { -1 };
            }
        }
        assert!(edges.values().all(|&e| e == (2, 0)));
        let mut creased = fixtures::folded(4);
        for (i, t) in creased.faces.iter_mut().enumerate() {
            t[3] = (i % 3) as u32;
        }
        let (c, _) = compiler::compile_traced(&creased, 0, true)?;
        assert_eq!(c.proxy.faces.len(), creased.faces.len());
        Ok(())
    }
    #[test]
    fn non_affine_center_remains_visible_geometry() -> Result<()> {
        for (height, normal) in [(0.1, [0., 0., 1.]), (0., [0.3, 0., 1.])] {
            let base = compiler::Mesh {
                vertices: vec![
                    compiler::Vertex {
                        p: [-1., -1., 0.],
                        n: [0., 0., 1.],
                    },
                    compiler::Vertex {
                        p: [1., -1., 0.],
                        n: [0., 0., 1.],
                    },
                    compiler::Vertex {
                        p: [1., 1., 0.],
                        n: [0., 0., 1.],
                    },
                    compiler::Vertex {
                        p: [-1., 1., 0.],
                        n: [0., 0., 1.],
                    },
                    compiler::Vertex {
                        p: [0., 0., height],
                        n: normal,
                    },
                ],
                faces: vec![[0, 1, 4, 0], [1, 2, 4, 0], [2, 3, 4, 0], [3, 0, 4, 0]],
            };
            let input = fixtures::subdivide(&base, 100);
            let (c, _) = compiler::compile_borders(&input, 0, true, true)?;
            assert!(c.proxy.faces.len() >= 4);
            assert!(
                c.proxy
                    .vertices
                    .iter()
                    .any(|v| v.p == [0., 0., height] && v.n == normal)
            );
        }
        Ok(())
    }
    #[test]
    fn refit_recovers_small_rounded_triangle() -> Result<()> {
        let base = compiler::Mesh {
            vertices: vec![
                compiler::Vertex {
                    p: [0.91234, 0.83456, 0.72345],
                    n: [0.01, 0.2, 0.95],
                },
                compiler::Vertex {
                    p: [0.91534, 0.83556, 0.72445],
                    n: [0.8, 0.1, 0.3],
                },
                compiler::Vertex {
                    p: [0.91334, 0.83496, 0.72365],
                    n: [0.1, 0.9, 0.2],
                },
            ],
            faces: vec![[0, 1, 2, 0]],
        };
        let mut m = base;
        let origin = m.vertices[0].p;
        for v in &mut m.vertices {
            for (i, x) in v.p.iter_mut().enumerate() {
                *x = origin[i] + (*x - origin[i]) * 8.;
            }
        }
        let input = fixtures::subdivide(&m, 100);
        let (old, _) = compiler::compile_traced(&input, 0, false)?;
        let (new, stats) = compiler::compile_traced(&input, 0, true)?;
        assert!(old.proxy.faces.len() > 90);
        assert_eq!(new.proxy.faces.len(), 1);
        assert!(stats.refits > 0);
        assert!(new.regions.iter().all(|r| r.normal_error <= 0.000101));
        Ok(())
    }
}
