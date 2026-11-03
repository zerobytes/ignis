//! Pure image-space diagnostics. Correspondences are supplied hypotheses, not inferred matches.
pub use crate::mesh_screen::ProjectionStatus as Status;
use crate::{
    mesh_screen::Projection,
    model::{Camera, Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_LANDMARKS: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Landmark {
    pub id: String,
    pub world: [f32; 3],
    pub reference_pixel: Option<[f32; 2]>,
}

#[derive(Debug, Serialize)]
pub struct Sample<'a> {
    pub id: &'a str,
    pub world: [f32; 3],
    pub status: Status,
    pub pixel: Option<[f32; 2]>,
    pub reference_pixel: Option<[f32; 2]>,
    /// Predicted minus supplied reference, in pixels. Offscreen XY points are included.
    pub residual: Option<[f64; 2]>,
    pub distance_pixels: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct Report<'a> {
    pub camera: &'a Camera,
    pub width: u32,
    pub height: u32,
    pub requested: usize,
    pub projectable: usize,
    pub in_viewport: usize,
    pub correspondences: usize,
    pub compared: usize,
    pub excluded: usize,
    /// Euclidean pixel RMSE over projectable correspondences only; null if none.
    pub rmse_pixels: Option<f64>,
    pub maximum_pixels: Option<f64>,
    pub samples: Vec<Sample<'a>>,
}

pub fn inspect<'a>(
    camera: &'a Camera,
    width: u32,
    height: u32,
    landmarks: &'a [Landmark],
) -> Result<Report<'a>> {
    if landmarks.is_empty() || landmarks.len() > MAX_LANDMARKS {
        return Err(invalid("Projection requires1..128 landmarks"));
    }
    let projection = Projection::new(camera, width, height)?;
    // Validate the complete request before allocating output or projecting anything.
    for (i, point) in landmarks.iter().enumerate() {
        if point.id.is_empty()
            || point.id.len() > 64
            || !point
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        {
            return Err(invalid(
                "Landmark ID requires1..64 ASCII letters, digits, -_.",
            ));
        }
        if landmarks[..i].iter().any(|p| p.id == point.id) {
            return Err(invalid("Landmark IDs must be unique"));
        }
        if point
            .world
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 10_000.)
        {
            return Err(invalid(
                "Landmark world coordinates must be finite within +/-10000m",
            ));
        }
        if point.reference_pixel.is_some_and(|p| {
            !p[0].is_finite()
                || !p[1].is_finite()
                || p[0] < 0.
                || p[1] < 0.
                || p[0] >= width as f32
                || p[1] >= height as f32
        }) {
            return Err(invalid(
                "Reference pixel must be finite and inside the viewport",
            ));
        }
    }
    let mut report = Report {
        camera,
        width,
        height,
        requested: landmarks.len(),
        projectable: 0,
        in_viewport: 0,
        correspondences: 0,
        compared: 0,
        excluded: 0,
        rmse_pixels: None,
        maximum_pixels: None,
        samples: Vec::with_capacity(landmarks.len()),
    };
    let mut squared_sum = 0.;
    for point in landmarks {
        let (status, pixel) = projection.project_classified(point.world.into());
        let pixel = pixel.map(|v| v.to_array());
        report.projectable += usize::from(pixel.is_some());
        report.in_viewport += usize::from(status == Status::InViewport);
        report.correspondences += usize::from(point.reference_pixel.is_some());
        let residual = pixel
            .zip(point.reference_pixel)
            .map(|(p, r)| [p[0] as f64 - r[0] as f64, p[1] as f64 - r[1] as f64]);
        let distance_pixels = residual.map(|r| r[0].hypot(r[1]));
        if let Some(distance) = distance_pixels {
            report.compared += 1;
            squared_sum += distance * distance;
            report.maximum_pixels = Some(report.maximum_pixels.unwrap_or(0.).max(distance));
        }
        report.samples.push(Sample {
            id: &point.id,
            world: point.world,
            status,
            pixel,
            reference_pixel: point.reference_pixel,
            residual,
            distance_pixels,
        });
    }
    report.excluded = report.correspondences - report.compared;
    if report.compared > 0 {
        report.rmse_pixels = Some((squared_sum / report.compared as f64).sqrt());
    }
    Ok(report)
}
