//! Shared surface composition, independent of the source of the geometry.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    #[default]
    Opaque,
    Mask,
    Blend,
    Additive,
}

/// An independently versioned material domain. Absence preserves legacy shading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub version: u16,
    #[serde(default)]
    pub mode: BlendMode,
    #[serde(default = "one")]
    pub opacity: f32,
    #[serde(default = "half")]
    pub cutoff: f32,
    #[serde(default)]
    pub emissive: [f32; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<Coverage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glass: Option<Glass>,
}
fn one() -> f32 {
    1.
}
fn half() -> f32 {
    0.5
}
impl Default for Surface {
    fn default() -> Self {
        Self {
            version: 1,
            mode: BlendMode::Opaque,
            opacity: 1.,
            cutoff: 0.5,
            emissive: [0.; 3],
            coverage: None,
            glass: None,
        }
    }
}

/// Scalar coverage in the existing material rest coordinates, separate from RGB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub frequency: [f32; 3],
    pub low: f32,
    pub high: f32,
    pub seed: u16,
    #[serde(default)]
    pub kind: super::PatternKind,
}
impl Default for Coverage {
    fn default() -> Self {
        Self {
            frequency: [4.; 3],
            low: 0.,
            high: 1.,
            seed: 1,
            kind: Default::default(),
        }
    }
}

/// Each geometric surface represents one thin sheet; no spatial ray refraction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Glass {
    pub ior: f32,
    pub roughness: f32,
    pub thickness_m: f32,
    pub attenuation_color: [f32; 3],
    pub attenuation_distance_m: f32,
}
impl Default for Glass {
    fn default() -> Self {
        Self {
            ior: 1.5,
            roughness: 0.2,
            thickness_m: 0.004,
            attenuation_color: [1.; 3],
            attenuation_distance_m: 1.,
        }
    }
}
impl Surface {
    pub fn transparent(&self) -> bool {
        matches!(self.mode, BlendMode::Blend | BlendMode::Additive)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !self.opacity.is_finite()
            || !(0. ..=1.).contains(&self.opacity)
            || !self.cutoff.is_finite()
            || !(0. ..=1.).contains(&self.cutoff)
            || self
                .emissive
                .iter()
                .any(|v| !v.is_finite() || !(0. ..=64.).contains(v))
        {
            return Err(invalid(
                "Surface1 requires opacity/cutoff0..1 and emissive0..64",
            ));
        }
        if self.mode == BlendMode::Opaque && (self.opacity != 1. || self.coverage.is_some()) {
            return Err(invalid(
                "Opaque surface requires full coverage; use mask or blend",
            ));
        }
        if let Some(c) = &self.coverage
            && (c
                .frequency
                .iter()
                .any(|v| !v.is_finite() || !(0.01..=2048.).contains(v))
                || [c.low, c.high]
                    .iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v)))
        {
            return Err(invalid(
                "Coverage requires frequency0.01..2048 and low/high0..1",
            ));
        }
        if let Some(g) = self.glass
            && (self.mode != BlendMode::Blend
                || !g.ior.is_finite()
                || !(1. ..=2.5).contains(&g.ior)
                || !g.roughness.is_finite()
                || !(0.2..=1.).contains(&g.roughness)
                || !g.thickness_m.is_finite()
                || !(0. ..=10.).contains(&g.thickness_m)
                || !g.attenuation_distance_m.is_finite()
                || !(0.0001..=10000.).contains(&g.attenuation_distance_m)
                || g.attenuation_color
                    .iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v)))
        {
            return Err(invalid(
                "Glass requires blend mode, IOR1..2.5, roughness0.2..1, thickness0..10m, attenuation color0..1/distance0.0001..10000m",
            ));
        }
        Ok(())
    }
}
