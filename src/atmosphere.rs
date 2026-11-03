//! Scene-authored aerial perspective and quality budgets; selection lives in distant.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Atmosphere {
    /// Optional depth-guided edge finish; absent/zero uses the original direct path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_softness: Option<EdgeSoftness>,
    /// Optional artistic layers: background and low-frequency surface are independent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layers: Option<Layers>,
    /// Extinction per metre, finite0..1. Zero with layers absent preserves the legacy surface/background.
    pub density: f32,
    #[serde(default)]
    pub reduce_details: bool,
    #[serde(default = "default_pixels")]
    pub silhouette_error_px: f32,
    #[serde(default = "default_color_error")]
    pub color_error: f32,
    /// Linear RGB of distant air, each channel0..1.
    pub color: [f32; 3],
    /// Linear RGB toward the directional light, each channel0..1.
    pub sun_color: [f32; 3],
    /// Strength0..1 of the directional lobe, multiplied by direct light intensity.
    pub sun_strength: f32,
}

fn default_pixels() -> f32 {
    0.5
}
fn default_color_error() -> f32 {
    0.002
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            edge_softness: None,
            layers: None,
            density: 0.01,
            reduce_details: true,
            silhouette_error_px: default_pixels(),
            color_error: default_color_error(),
            color: [0.38, 0.55, 0.72],
            sun_color: [1., 0.78, 0.52],
            sun_strength: 0.35,
        }
    }
}

impl Atmosphere {
    pub fn validate(&self) -> Result<()> {
        if let Some(edges) = self.edge_softness {
            edges.validate()?;
        }
        if let Some(layers) = self.layers {
            layers.validate()?;
        }
        if !self.silhouette_error_px.is_finite()
            || !(0. ..=2.).contains(&self.silhouette_error_px)
            || !self.color_error.is_finite()
            || !(0. ..=0.05).contains(&self.color_error)
        {
            return Err(invalid(
                "Atmosphere tolerances require pixels0..2 and linear color0..0.05",
            ));
        }
        if self
            .color
            .iter()
            .chain(&self.sun_color)
            .chain([&self.density, &self.sun_strength])
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        {
            return Err(invalid(
                "Atmosphere requires finite density0..1/m, RGB0..1 and sun_strength0..1",
            ));
        }
        Ok(())
    }
}

/// Scene-authored appearance, not an inference of physical humidity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Layers {
    pub sky_color: [f32; 3],
    pub mountain_color: [f32; 3],
    pub detail_start: f32,
    pub detail_end: f32,
    pub influence: f32,
}
impl Default for Layers {
    fn default() -> Self {
        Self {
            sky_color: [0.72, 0.80, 0.9],
            mountain_color: [0.015, 0.04, 0.075],
            detail_start: 800.,
            detail_end: 4000.,
            influence: 1.,
        }
    }
}
impl Layers {
    pub fn remaining(&self, distance: f32) -> f32 {
        let t =
            ((distance - self.detail_start) / (self.detail_end - self.detail_start)).clamp(0., 1.);
        1. - self.influence * t * t * (3. - 2. * t)
    }
    pub fn validate(&self) -> Result<()> {
        if self
            .sky_color
            .iter()
            .chain(&self.mountain_color)
            .chain([&self.influence])
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            || !self.detail_start.is_finite()
            || !self.detail_end.is_finite()
            || self.detail_start < 0.
            || self.detail_end <= self.detail_start
            || self.detail_end > 100_000.
        {
            return Err(invalid(
                "Atmospheric layers require RGB/influence0..1 and 0<=start<end<=100000m",
            ));
        }
        Ok(())
    }
}

/// Artistic screen-space feathering, independent of geometry error tolerances.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EdgeSoftness {
    pub strength: f32,
    pub radius_px: u32,
    pub start: f32,
    pub end: f32,
}
impl Default for EdgeSoftness {
    fn default() -> Self {
        Self {
            strength: 0.5,
            radius_px: 6,
            start: 2000.,
            end: 10000.,
        }
    }
}
impl EdgeSoftness {
    pub fn active(self) -> bool {
        self.strength > 0. && self.radius_px > 0
    }
    pub fn validate(self) -> Result<()> {
        if !self.strength.is_finite()
            || !(0. ..=1.).contains(&self.strength)
            || self.radius_px > 12
            || !self.start.is_finite()
            || !self.end.is_finite()
            || self.start < 0.
            || self.end <= self.start
            || self.end > 100000.
        {
            return Err(invalid(
                "Edge softness requires strength0..1, radius0..12 physical pixels, 0<=start<end<=100000m",
            ));
        }
        Ok(())
    }
}
