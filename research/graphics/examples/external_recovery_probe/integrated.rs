use super::{
    compiler,
    render::{Gpu, Pose},
};
use anyhow::{Result, ensure};
use std::path::Path;
fn offsets(n: usize) -> Vec<[f32; 2]> {
    (0..n)
        .flat_map(|y| {
            (0..n).map(move |x| {
                [
                    (x as f32 + 0.5) / n as f32 - 0.5,
                    (y as f32 + 0.5) / n as f32 - 0.5,
                ]
            })
        })
        .collect()
}
fn metrics(a: &[[f32; 4]], b: &[[f32; 4]]) -> serde_json::Value {
    let mut errors = vec![];
    let mut sum = 0.;
    let mut alpha = 0.;
    let mut count = 0;
    for (a, b) in a.iter().zip(b) {
        if a[3] > 0. || b[3] > 0. {
            count += 1;
            alpha += (a[3] - b[3]).abs() as f64;
            for i in 0..3 {
                let e = (a[i] - b[i]).abs();
                errors.push(e);
                sum += (e as f64).powi(2);
            }
        }
    }
    errors.sort_unstable_by(f32::total_cmp);
    serde_json::json!({"rmse":(sum/(count*3).max(1) as f64).sqrt(),"p99":errors.get(errors.len()*99/100).copied().unwrap_or(0.),"max":errors.last().copied().unwrap_or(0.),"alpha_mae":alpha/count.max(1) as f64,"covered":count})
}
fn png(path: &Path, pixels: &[[f32; 4]], size: u32) -> Result<()> {
    let bytes = pixels
        .iter()
        .flat_map(|p| {
            p[..3].iter().map(|&v| {
                let s = if v <= 0.0031308 {
                    v * 12.92
                } else {
                    1.055 * v.max(0.).powf(1. / 2.4) - 0.055
                };
                (s.clamp(0., 1.) * 255.).round() as u8
            })
        })
        .collect::<Vec<_>>();
    let mut encoder = png::Encoder::new(std::fs::File::create(path)?, size, size);
    encoder.set_color(png::ColorType::Rgb);
    encoder.write_header()?.write_image_data(&bytes)?;
    Ok(())
}
pub fn run(dir: &Path) -> Result<()> {
    let low_gpu = pollster::block_on(Gpu::new(1728, 1117))?;
    let high_gpu = pollster::block_on(Gpu::new(3456, 2234))?;
    let mut rows = vec![];
    let stone = dir.parent().unwrap().join("border-recovery");
    for (asset, folder, cases) in [
        (
            "stone",
            stone,
            vec![(1728, 2, 4), (1728, 10, 1), (1728, 100, 1), (1728, 100, 2)],
        ),
        (
            "stool",
            dir.to_path_buf(),
            vec![
                (1728, 2, 0),
                (1728, 10, 2),
                (1728, 100, 4),
                (1728, 100, 6),
                (1728, 100, 5),
                (3456, 100, 7),
            ],
        ),
    ] {
        let base = compiler::decode(&std::fs::read(folder.join("stable-1.igc"))?)?.source;
        for (width, factor, frame) in cases {
            let (gpu, rect) = if width == 1728 {
                (&low_gpu, [832, 526, 64, 64])
            } else {
                (&high_gpu, [1664, 1053, 128, 128])
            };
            let size = rect[2] as usize;
            let base_gpu = gpu.upload(&base);
            let c = compiler::decode(&std::fs::read(folder.join(format!("stable-{factor}.igc")))?)?;
            let source = gpu.upload(&c.source);
            let chosen = gpu.upload(&c.proxy);
            let phase = frame as f32 * std::f32::consts::TAU / 8.;
            let pose = Pose {
                angle: phase,
                distance: 64.,
                light: phase * 1.7 + 0.3,
            };
            let mut previous: Option<Vec<Vec<[f32; 4]>>> = None;
            for n in [1, 2, 4, 8] {
                let mut images = vec![];
                for (mode, buffer) in [
                    ("base", &base_gpu),
                    ("source", &source),
                    ("chosen", &chosen),
                ] {
                    let mut sum = vec![[0f32; 4]; size * size];
                    for offset in offsets(n) {
                        let pixels = gpu.capture_region(buffer, pose, rect, offset, None)?;
                        ensure!(
                            pixels.iter().enumerate().all(|(i, p)| (i % size != 0
                                && i % size != size - 1
                                && i / size != 0
                                && i / size != size - 1)
                                || p[3] == 0.),
                            "ROI clips asset"
                        );
                        for (s, p) in sum.iter_mut().zip(pixels) {
                            for k in 0..4 {
                                s[k] += p[k] / (n * n) as f32;
                            }
                        }
                    }
                    if n == 1 && mode == "base" {
                        let full = gpu.capture(buffer, pose)?;
                        for y in 0..size {
                            for x in 0..size {
                                ensure!(
                                    full[(y + rect[1] as usize) * width + x + rect[0] as usize]
                                        == sum[y * size + x],
                                    "ROI differs from unjittered full viewport"
                                );
                            }
                        }
                    }
                    if n == 1 || n == 8 {
                        png(
                            &dir.join(format!(
                                "integrated-{asset}-{factor}-{frame}-{n}-{mode}.png"
                            )),
                            &sum,
                            rect[2],
                        )?;
                    }
                    images.push(sum);
                }
                let convergence = previous.as_ref().map(|p| {
                    p.iter()
                        .zip(&images)
                        .map(|(a, b)| metrics(a, b))
                        .collect::<Vec<_>>()
                });
                rows.push(serde_json::json!({"asset":asset,"width":width,"roi":rect,"factor":factor,"frame":frame,"samples":n*n,"source_base":metrics(&images[1],&images[0]),"chosen_base":metrics(&images[2],&images[0]),"chosen_source":metrics(&images[2],&images[1]),"convergence_base_source_chosen":convergence}));
                previous = Some(images);
            }
            println!("integrated {asset} {factor} frame{frame}");
        }
    }
    std::fs::write(
        dir.join("integrated.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"viewports":[[1728,1117],[3456,2234]],"adapter":low_gpu.adapter,"rows":rows,"note":"Finite stratified linear RGBA averages. 64 samples are not an exact integral; convergence reported separately. No timing claim."}),
        )?,
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stratification_and_fractional_coverage() {
        for n in [1, 2, 4, 8] {
            let o = offsets(n);
            assert_eq!(o.len(), n * n);
            for axis in 0..2 {
                assert_eq!(o.iter().map(|v| v[axis]).sum::<f32>(), 0.);
                assert!(o.iter().all(|v| v[axis].abs() < 0.5));
            }
        }
        let a = [[0.5, 0.5, 0.5, 0.5]];
        let b = [[0., 0., 0., 0.]];
        let m = metrics(&a, &b);
        assert_eq!(m["rmse"], 0.5);
        assert_eq!(m["alpha_mae"], 0.5);
        assert_eq!(metrics(&a, &a)["rmse"], 0.);
    }
}
