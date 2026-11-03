//! Bounded procedural surface recipes. No gameplay geometry or vendor assets.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Material {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<Pattern>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_pattern: Option<Pattern>,
    #[serde(default)]
    pub transmission: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Arc<Detail>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reflectance: Option<Reflectance>,
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
            && self.secondary_pattern == other.secondary_pattern
            && self.transmission == other.transmission
            && self.reflectance == other.reflectance
            && match (&self.detail, &other.detail) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b) || a == b,
                _ => false,
            }
    }
}

/// Direct-light response. Zero F0 disables the dielectric lobe; metal remains independent.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Reflectance {
    pub roughness: f32,
    pub f0: f32,
    /// Blend dielectric/metal BRDFs. Base color is the metal reflectance at normal incidence.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub metallic: f32,
}
fn is_zero(v: &f32) -> bool {
    *v == 0.
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    /// Cycles per local rest-coordinate unit; instance scaling scales the pattern.
    pub frequency: [f32; 3],
    /// Multipliers of the instance base color, not absolute radiance.
    pub low: [f32; 3],
    pub high: [f32; 3],
    /// Normal perturbation amplitude in world metres; never changes geometry.
    pub bump: f32,
    pub warp: f32,
    pub octaves: u8,
    pub seed: u16,
    pub contrast: f32,
    #[serde(default)]
    pub kind: PatternKind,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    #[default]
    Noise,
    Ridged,
    Cellular,
}

pub const MAX_SEGMENTS: usize = 32;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Plane {
    Xy,
    #[default]
    Xz,
    Yz,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub start: [f32; 2],
    pub end: [f32; 2],
    /// Radii in material-coordinate units, not world metres.
    pub widths: [f32; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Triplanar {
    #[serde(default = "default_sharpness")]
    pub sharpness: f32,
}
fn default_sharpness() -> f32 {
    2.
}
impl Default for Triplanar {
    fn default() -> Self {
        Self {
            sharpness: default_sharpness(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    #[default]
    Band,
    Rounded,
}
impl Profile {
    fn is_band(&self) -> bool {
        *self == Self::Band
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Detail {
    #[serde(default, skip_serializing_if = "Profile::is_band")]
    pub profile: Profile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triplanar: Option<Triplanar>,
    #[serde(default)]
    pub plane: Plane,
    pub tint: [f32; 3],
    pub opacity: f32,
    /// Signed normal perturbation in world metres; no geometry displacement.
    pub bump: f32,
    pub segments: Vec<Segment>,
}
impl Detail {
    pub fn validate(&self) -> Result<()> {
        if let Some(period) = self.repeat {
            if period
                .iter()
                .any(|p| !p.is_finite() || !(0.001..=10000.).contains(p))
            {
                return Err(invalid(
                    "Detail repeat periods must be finite within0.001..10000",
                ));
            }
            for s in &self.segments {
                for (point, radius) in [(s.start, s.widths[0]), (s.end, s.widths[1])] {
                    for axis in 0..2 {
                        let margin = 2. * f64::from(radius);
                        if f64::from(point[axis]) - margin < 0.
                            || f64::from(point[axis]) + margin > f64::from(period[axis])
                        {
                            return Err(invalid(
                                "Repeated detail endpoints need2radius clearance inside their period cell",
                            ));
                        }
                    }
                }
            }
        }
        if self
            .triplanar
            .as_ref()
            .is_some_and(|p| !p.sharpness.is_finite() || !(1. ..=8.).contains(&p.sharpness))
        {
            return Err(invalid("Triplanar sharpness must be finite within1..8"));
        }
        if self.segments.is_empty()
            || self.segments.len() > MAX_SEGMENTS
            || self
                .tint
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=2.0).contains(v))
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
            || !self.bump.is_finite()
            || !(-0.05..=0.05).contains(&self.bump)
        {
            return Err(invalid(
                "Detail requires1..32 segments, tint0..2, opacity0..1 and bump±0.05m",
            ));
        }
        for s in &self.segments {
            if s.start
                .iter()
                .chain(s.end.iter())
                .any(|v| !v.is_finite() || v.abs() > 10000.)
                || s.widths
                    .iter()
                    .any(|v| !v.is_finite() || !(0.000001..=10.).contains(v))
                || (f64::from(s.end[0]) - f64::from(s.start[0]))
                    .hypot(f64::from(s.end[1]) - f64::from(s.start[1]))
                    < 0.000001
            {
                return Err(invalid(
                    "Detail segment requires finite coordinates±10000, radii0.000001..10 and nonzero length",
                ));
            }
        }
        Ok(())
    }
}

impl Material {
    pub fn validate(&self) -> Result<()> {
        if let Some(r) = self.reflectance
            && (!r.roughness.is_finite()
                || !(0.2..=1.).contains(&r.roughness)
                || !r.f0.is_finite()
                || !(0.0..=0.16).contains(&r.f0))
        {
            return Err(invalid(
                "Reflectance requires roughness0.2..1 and f0=0..0.16",
            ));
        }
        if let Some(r) = self.reflectance {
            if !r.metallic.is_finite() || !(0. ..=1.).contains(&r.metallic) {
                return Err(invalid("Reflectance metallic requires finite0..1"));
            }
            if r.metallic > 0. && self.transmission > 0. {
                return Err(invalid(
                    "Metallic response is incompatible with transmission",
                ));
            }
        }
        if let Some(detail) = &self.detail {
            detail.validate()?;
        }
        if !self.transmission.is_finite() || !(0.0..=1.0).contains(&self.transmission) {
            return Err(invalid("Material transmission requires0..1"));
        }
        for p in [self.pattern.as_ref(), self.secondary_pattern.as_ref()]
            .into_iter()
            .flatten()
        {
            p.validate()?;
        }
        if self
            .pattern
            .iter()
            .chain(self.secondary_pattern.iter())
            .map(|p| f64::from(p.bump))
            .sum::<f64>()
            > f64::from(0.05f32)
        {
            return Err(invalid("Combined pattern bump must not exceed0.05m"));
        }
        Ok(())
    }
}

impl Pattern {
    pub fn validate(&self) -> Result<()> {
        if self
            .frequency
            .iter()
            .any(|v| !v.is_finite() || !(0.01..=2048.).contains(v))
            || self
                .low
                .iter()
                .chain(self.high.iter())
                .any(|v| !v.is_finite() || !(0.0..=2.0).contains(v))
            || !self.bump.is_finite()
            || !(0.0..=0.05).contains(&self.bump)
            || !self.warp.is_finite()
            || !(0.0..=2.0).contains(&self.warp)
            || !(1..=4).contains(&self.octaves)
            || !self.contrast.is_finite()
            || !(0.1..=4.0).contains(&self.contrast)
        {
            return Err(invalid(
                "Pattern requires frequency0.01..2048, tints0..2, bump0..0.05m, warp0..2, octaves1..4 and contrast0.1..4",
            ));
        }
        Ok(())
    }
}

// GPU-only slots are derived from the frame and never stored as asset identity.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub(crate) struct GpuPattern {
    frequency: [f32; 4],
    low: [f32; 4],
    high: [f32; 4],
    control: [f32; 4],
    response: [f32; 4],
}
impl From<&Pattern> for GpuPattern {
    fn from(p: &Pattern) -> Self {
        Self {
            frequency: [p.frequency[0], p.frequency[1], p.frequency[2], 1.],
            low: [p.low[0], p.low[1], p.low[2], 0.],
            high: [p.high[0], p.high[1], p.high[2], 0.],
            control: [p.bump, p.warp, p.octaves as f32, p.seed as f32],
            response: [
                0.,
                match p.kind {
                    PatternKind::Noise => 0.,
                    PatternKind::Ridged => 1.,
                    PatternKind::Cellular => 2.,
                },
                p.contrast,
                0.,
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub(crate) struct GpuMaterial {
    pub frequency: [f32; 4],
    pub low: [f32; 4],
    pub high: [f32; 4],
    pub control: [f32; 4],
    pub response: [f32; 4],
    pub detail_tint: [f32; 4],
    pub detail_control: [f32; 4],
    pub detail_domain: [f32; 4],
    pub segments: [[[f32; 4]; 2]; MAX_SEGMENTS],
    secondary: GpuPattern,
}
impl From<&Material> for GpuMaterial {
    fn from(material: &Material) -> Self {
        let mut out = Self {
            frequency: [1., 1., 1., 0.],
            low: [1.; 4],
            high: [1., 1., 1., 0.],
            control: [0.; 4],
            response: [material.transmission, 0., 1., 0.],
            detail_tint: [1., 1., 1., 0.],
            detail_control: [0.; 4],
            detail_domain: [0.; 4],
            segments: [[[0.; 4]; 2]; MAX_SEGMENTS],
            secondary: material
                .secondary_pattern
                .as_ref()
                .map_or_else(Default::default, GpuPattern::from),
        };
        if let Some(p) = &material.pattern {
            let p = GpuPattern::from(p);
            out.frequency = p.frequency;
            out.low = p.low;
            out.high = p.high;
            out.control = p.control;
            out.response[1] = p.response[1];
            out.response[2] = p.response[2];
        }
        if let Some(r) = material.reflectance {
            out.low[3] = r.roughness;
            out.response[3] = r.f0;
            out.high[3] = r.metallic;
        }
        if let Some(detail) = &material.detail {
            let period = detail.repeat.unwrap_or([0.; 2]);
            out.detail_domain = [
                period[0],
                period[1],
                if detail.profile == Profile::Rounded {
                    1.
                } else {
                    0.
                },
                0.,
            ];
            out.detail_tint = [
                detail.tint[0],
                detail.tint[1],
                detail.tint[2],
                detail.opacity,
            ];
            out.detail_control = [
                detail.bump,
                match detail.plane {
                    Plane::Xy => 0.,
                    Plane::Xz => 1.,
                    Plane::Yz => 2.,
                },
                detail.segments.len() as f32,
                detail.triplanar.as_ref().map_or(0., |p| p.sharpness),
            ];
            for (slot, s) in out.segments.iter_mut().zip(&detail.segments) {
                *slot = [
                    [s.start[0], s.start[1], s.widths[0], 0.],
                    [s.end[0], s.end[1], s.widths[1], 0.],
                ];
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_detail_table_matches_uniform_layout_and_zero_count_legacy() {
        assert_eq!(std::mem::size_of::<GpuMaterial>(), 1232);
        let old: Material = serde_json::from_str(r#"{"transmission":0.3}"#).unwrap();
        let gpu = GpuMaterial::from(&old);
        assert_eq!(gpu.detail_control, [0.; 4]);
        assert!(!serde_json::to_string(&old).unwrap().contains("detail"));
        assert!(!serde_json::to_string(&old).unwrap().contains("reflectance"));
        assert_eq!(gpu.response[3], 0.);
        assert_eq!(gpu.high[3], 0.);
        assert_eq!(gpu.detail_domain, [0.; 4]);
        assert_eq!(std::mem::offset_of!(GpuMaterial, detail_domain), 112);
        assert_eq!(std::mem::offset_of!(GpuMaterial, segments), 128);
    }

    #[test]
    fn secondary_layout_encoding_and_legacy_bytes() {
        assert_eq!(std::mem::size_of::<GpuPattern>(), 80);
        assert_eq!(std::mem::offset_of!(GpuMaterial, secondary), 1152);
        let mut a: Material = serde_json::from_str(r#"{"pattern":{"frequency":[2,3,4],"low":[0.2,0.3,0.4],"high":[1,1.2,1.3],"bump":0.002,"warp":0.3,"octaves":3,"seed":71,"contrast":1.2,"kind":"cellular"},"transmission":0.3,"reflectance":{"roughness":0.7,"f0":0.08}}"#).unwrap();
        let before = GpuMaterial::from(&a);
        assert_eq!(before.secondary, GpuPattern::default());
        a.secondary_pattern = a.pattern;
        let layered = GpuMaterial::from(&a);
        assert_eq!(
            &bytemuck::bytes_of(&before)[..1152],
            &bytemuck::bytes_of(&layered)[..1152]
        );
        assert_eq!(layered.secondary.frequency, before.frequency);
        assert_eq!(layered.secondary.low[..3], before.low[..3]);
        assert_eq!(layered.secondary.high, before.high);
        assert_eq!(layered.secondary.control, before.control);
        assert_eq!(layered.secondary.response, [0., 2., 1.2, 0.]);
        a.pattern = None;
        assert_eq!(GpuMaterial::from(&a).secondary, layered.secondary);
    }

    #[test]
    fn metallic_reuses_only_high_w_without_changing_secondary_or_other_layout() {
        let mut a: Material = serde_json::from_str(r#"{"pattern":{"frequency":[2,3,4],"low":[0.2,0.3,0.4],"high":[1,1.2,1.3],"bump":0.002,"warp":0.3,"octaves":3,"seed":71,"contrast":1.2},"reflectance":{"roughness":0.7,"f0":0.08}}"#).unwrap();
        a.secondary_pattern = a.pattern;
        let mut expected = GpuMaterial::from(&a);
        let before = a.clone();
        a.reflectance.as_mut().unwrap().metallic = 0.85;
        expected.high[3] = 0.85;
        assert_eq!(GpuMaterial::from(&a), expected);
        assert_ne!(a, before);
        assert_eq!(std::mem::size_of::<GpuMaterial>(), 1232);
    }

    #[test]
    fn projection_uses_only_reserved_detail_scalar_and_changes_material_equality() {
        let mut a: Material = serde_json::from_str(r#"{"detail":{"tint":[0.2,0.3,0.4],"opacity":0.7,"bump":0.002,"segments":[{"start":[0,0],"end":[1,1],"widths":[0.1,0.01]}]}}"#).unwrap();
        let old = a.clone();
        let mut expected = GpuMaterial::from(&old);
        Arc::make_mut(a.detail.as_mut().unwrap()).triplanar = Some(Default::default());
        expected.detail_control[3] = 2.;
        assert_eq!(GpuMaterial::from(&a), expected);
        assert_ne!(old, a);
        Arc::make_mut(a.detail.as_mut().unwrap()).triplanar = None;
        assert_eq!(old, a);
        let d = Arc::make_mut(a.detail.as_mut().unwrap());
        d.profile = Profile::Rounded;
        d.repeat = Some([2., 3.]);
        assert_ne!(old, a);
        assert_eq!(GpuMaterial::from(&a).detail_domain, [2., 3., 1., 0.]);
    }

    #[test]
    fn reflectance_reuses_unused_scalars_and_changes_semantic_equality() {
        let mut a: Material = serde_json::from_str(r#"{"pattern":{"frequency":[2,3,4],"low":[0.2,0.3,0.4],"high":[1,1,1],"bump":0,"warp":0,"octaves":1,"seed":0,"contrast":1},"reflectance":{"roughness":0.5,"f0":0.04}}"#).unwrap();
        let gpu = GpuMaterial::from(&a);
        assert_eq!(gpu.low, [0.2, 0.3, 0.4, 0.5]);
        assert_eq!(gpu.response[3], 0.04);
        let old = a.clone();
        a.reflectance.as_mut().unwrap().roughness = 0.8;
        assert_ne!(old, a);
    }
}
