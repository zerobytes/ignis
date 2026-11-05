use anyhow::Result;
use serde::Serialize;
use std::{fs::File, io::BufWriter, path::Path};

#[derive(Serialize, Debug)]
pub struct Error {
    pub rmse_surface: f64,
    pub rmse_interior: f64,
    pub rmse_boundary: f64,
    pub max_abs: f64,
    pub alpha_rmse: f64,
    pub surface_pixels: usize,
}
pub fn compare(reference: &[[f32; 4]], candidate: &[[f32; 4]]) -> Error {
    assert_eq!(reference.len(), candidate.len());
    let mut sums = [0.0_f64; 3];
    let mut counts = [0_usize; 3];
    let mut alpha = 0.0;
    let mut maximum = 0.0_f64;
    for (r, c) in reference.iter().zip(candidate) {
        if r[3] <= 0.0 && c[3] <= 0.0 {
            continue;
        }
        let region = if r[3] > 0.999 && c[3] > 0.999 { 1 } else { 2 };
        for k in 0..3 {
            let e = f64::from(c[k] - r[k]);
            sums[0] += e * e;
            sums[region] += e * e;
            maximum = maximum.max(e.abs());
        }
        counts[0] += 1;
        counts[region] += 1;
        alpha += f64::from(c[3] - r[3]).powi(2);
    }
    let rmse = |i: usize| (sums[i] / (counts[i].max(1) * 3) as f64).sqrt();
    Error {
        rmse_surface: rmse(0),
        rmse_interior: rmse(1),
        rmse_boundary: rmse(2),
        max_abs: maximum,
        alpha_rmse: (alpha / counts[0].max(1) as f64).sqrt(),
        surface_pixels: counts[0],
    }
}

// Difference of residuals at screen coordinates; includes silhouette motion.
// It is NOT optical-flow-aligned perceptual flicker or a temporal reconstruction test.
pub fn temporal(
    previous_ref: &[[f32; 4]],
    previous: &[[f32; 4]],
    reference: &[[f32; 4]],
    candidate: &[[f32; 4]],
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0;
    for i in 0..reference.len() {
        if reference[i][3]
            .max(previous_ref[i][3])
            .max(candidate[i][3])
            .max(previous[i][3])
            <= 0.0
        {
            continue;
        }
        for c in 0..3 {
            let delta = f64::from(
                (candidate[i][c] - reference[i][c]) - (previous[i][c] - previous_ref[i][c]),
            );
            sum += delta * delta;
            count += 1;
        }
    }
    (sum / count.max(1) as f64).sqrt()
}
fn display(x: f32) -> u8 {
    let x = x.max(0.0) / (1.0 + x.max(0.0));
    let srgb = if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (srgb * 255.0).round() as u8
}
pub fn png(path: &Path, pixels: &[[f32; 4]], width: u32) -> Result<()> {
    let mut bytes = Vec::with_capacity(pixels.len() * 3);
    for p in pixels {
        for value in &p[..3] {
            bytes.push(display(*value + 0.018 * (1.0 - p[3])));
        }
    }
    let mut encoder = png::Encoder::new(
        BufWriter::new(File::create(path)?),
        width,
        (pixels.len() / width as usize) as u32,
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&bytes)?;
    Ok(())
}
pub fn error_png(
    path: &Path,
    reference: &[[f32; 4]],
    candidate: &[[f32; 4]],
    width: u32,
) -> Result<()> {
    let pixels = reference
        .iter()
        .zip(candidate)
        .map(|(r, c)| {
            [
                (c[0] - r[0]).abs() * 8.0,
                (c[1] - r[1]).abs() * 8.0,
                (c[2] - r[2]).abs() * 8.0,
                1.0,
            ]
        })
        .collect::<Vec<_>>();
    png(path, &pixels, width)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metrics_do_not_hide_errors_in_background_or_cancellation() {
        let a = [[1., 0., 0., 1.], [0., 0., 0., 0.]];
        let b = [[0., 1., 0., 1.], [0., 0., 0., 0.]];
        assert!((compare(&a, &b).rmse_surface - (2.0_f64 / 3.0).sqrt()).abs() < 1e-10);
        assert_eq!(compare(&a, &a).rmse_surface, 0.0);
        assert_eq!(temporal(&a, &b, &a, &b), 0.0);
        assert!(temporal(&a, &a, &a, &b) > 0.8);
        let missing = [[0., 0., 0., 0.]; 2];
        assert_eq!(compare(&a, &missing).alpha_rmse, 1.0);
    }
}
