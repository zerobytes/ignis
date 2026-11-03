//! Pure local derivatives of an interpolated material domain on triangle surfaces.
use crate::{
    meshes::Geometry,
    model::{Result, invalid},
};
use glam::DVec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    #[default]
    Spatial,
    Xy,
    Xz,
    Yz,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    pub domain: Domain,
    pub scale: [f32; 3],
    pub sample_limit: u8,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            domain: Domain::Spatial,
            scale: [1.; 3],
            sample_limit: 8,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Extremum {
    pub face: u32,
    pub value: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sample {
    pub face: u32,
    pub local_area: f64,
    pub rates: [f64; 2],
    pub area_rate: f64,
    pub rank: u8,
    pub near_singular: bool,
    pub anisotropy: Option<f64>,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    pub options: Options,
    pub coordinates: &'static str,
    pub selected_faces: usize,
    pub local_area: f64,
    pub regular_area: f64,
    pub rank_zero_faces: usize,
    pub rank_one_faces: usize,
    pub near_singular_faces: usize,
    pub minimum_rate: Extremum,
    pub maximum_rate: Extremum,
    pub maximum_anisotropy: Option<Extremum>,
    pub area_weighted_mean_anisotropy: Option<f64>,
    pub samples: Vec<Sample>,
}
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
fn project(mut p: DVec3, o: &Options) -> DVec3 {
    p *= v(o.scale);
    match o.domain {
        Domain::Spatial => p,
        Domain::Xy => DVec3::new(p.x, p.y, 0.),
        Domain::Xz => DVec3::new(p.x, p.z, 0.),
        Domain::Yz => DVec3::new(p.y, p.z, 0.),
    }
}
pub(crate) fn sample(
    g: &Geometry,
    coordinates: &[[f32; 3]],
    face: u32,
    o: &Options,
) -> Result<Sample> {
    let t = g.triangles[face as usize];
    let [p, q, r] = t.map(|i| v(g.positions[i as usize]));
    let e = q - p;
    let f = r - p;
    let x = e.length();
    let twice_area = e.cross(f).length();
    let longest_squared = e
        .length_squared()
        .max(f.length_squared())
        .max((r - q).length_squared());
    if twice_area <= longest_squared * 1e-12 {
        return Err(invalid(format!(
            "Material-domain face {face} is too ill-conditioned for stable derivatives"
        )));
    }
    // Cross products avoid cancellation in |f|² - projection² on thin faces.
    let y = twice_area / x;
    if x == 0. || y == 0. || !y.is_finite() {
        return Err(invalid(
            "Material-domain inspection requires a nondegenerate local face",
        ));
    }
    let [a, b, c] = t.map(|i| v(coordinates[i as usize]));
    let u = project(b - a, o);
    let w = project(c - a, o);
    let j0 = u / x;
    let j1 = (w - j0 * f.dot(e / x)) / y;
    let aa = j0.length_squared();
    let bb = j0.dot(j1);
    let cc = j1.length_squared();
    let largest = ((aa + cc) + (aa - cc).hypot(2. * bb)) * 0.5;
    // Product of singular values from cross magnitude avoids cancellation
    // in the small eigenvalue of the Gram matrix.
    let area_rate = u.cross(w).length() / twice_area;
    let high = largest.sqrt();
    let low = if high > 0. { area_rate / high } else { 0. };
    let rank = if high == 0. {
        0
    } else if low == 0. {
        1
    } else {
        2
    };
    let near_singular = low <= high * 1e-10;
    // Do not rank or average numerically unbounded ratios as ordinary samples.
    let anisotropy = (!near_singular).then(|| high / low);
    if ![twice_area, area_rate, high, low]
        .iter()
        .all(|v| v.is_finite())
        || anisotropy.is_some_and(|v| !v.is_finite())
    {
        return Err(invalid("Material-domain derivatives are not finite"));
    }
    Ok(Sample {
        face,
        local_area: twice_area * 0.5,
        rates: [low, high],
        area_rate,
        rank,
        near_singular,
        anisotropy,
    })
}
fn worse(a: &Sample, b: &Sample) -> std::cmp::Ordering {
    b.near_singular
        .cmp(&a.near_singular)
        .then(a.rank.cmp(&b.rank))
        .then_with(|| {
            b.anisotropy
                .unwrap_or(0.)
                .total_cmp(&a.anisotropy.unwrap_or(0.))
        })
        .then(a.face.cmp(&b.face))
}
pub fn inspect(g: &Geometry, faces: Option<&[u32]>, options: &Options) -> Result<Report> {
    g.validate()?;
    if options.sample_limit > 32 || options.scale.iter().any(|v| !v.is_finite() || *v <= 0.) {
        return Err(invalid(
            "Material-domain inspection requires positive finite scale and sample_limit0..32",
        ));
    }
    let selected = faces
        .map(|faces| {
            if faces.is_empty() || faces.len() > g.triangles.len() {
                return Err(invalid(
                    "Material-domain faces must be nonempty within the triangle budget",
                ));
            }
            let mut out = faces.to_vec();
            out.sort_unstable();
            if out.last().is_some_and(|&i| i as usize >= g.triangles.len())
                || out.windows(2).any(|p| p[0] == p[1])
            {
                return Err(invalid(
                    "Material-domain faces contain missing or duplicate indices",
                ));
            }
            Ok(out)
        })
        .transpose()?;
    let count = selected.as_ref().map_or(g.triangles.len(), Vec::len);
    let coordinates = g.material_coordinates.as_deref().unwrap_or(&g.positions);
    let mut report = Report {
        options: options.clone(),
        coordinates: if g.material_coordinates.is_some() {
            "explicit"
        } else {
            "local_positions_fallback"
        },
        selected_faces: count,
        local_area: 0.,
        regular_area: 0.,
        rank_zero_faces: 0,
        rank_one_faces: 0,
        near_singular_faces: 0,
        minimum_rate: Extremum {
            face: 0,
            value: f64::INFINITY,
        },
        maximum_rate: Extremum {
            face: 0,
            value: -1.,
        },
        maximum_anisotropy: None,
        area_weighted_mean_anisotropy: None,
        samples: Vec::with_capacity(options.sample_limit as usize),
    };
    let mut weighted = 0.;
    for i in 0..count {
        let face = selected.as_ref().map_or(i as u32, |faces| faces[i]);
        let s = sample(g, coordinates, face, options)?;
        report.local_area += s.local_area;
        report.rank_zero_faces += usize::from(s.rank == 0);
        report.rank_one_faces += usize::from(s.rank == 1);
        report.near_singular_faces += usize::from(s.near_singular);
        if s.rates[0] < report.minimum_rate.value {
            report.minimum_rate = Extremum {
                face,
                value: s.rates[0],
            };
        }
        if s.rates[1] > report.maximum_rate.value {
            report.maximum_rate = Extremum {
                face,
                value: s.rates[1],
            };
        }
        if let Some(a) = s.anisotropy {
            report.regular_area += s.local_area;
            weighted += a * s.local_area;
            if report
                .maximum_anisotropy
                .as_ref()
                .is_none_or(|e| a > e.value)
            {
                report.maximum_anisotropy = Some(Extremum { face, value: a });
            }
        }
        let pos = report.samples.partition_point(|old| worse(old, &s).is_le());
        if pos < options.sample_limit as usize {
            // Remove before insert so a full topK never reallocates.
            if report.samples.len() == options.sample_limit as usize {
                report.samples.pop();
            }
            report.samples.insert(pos, s);
        }
    }
    report.area_weighted_mean_anisotropy =
        (report.regular_area > 0.).then(|| weighted / report.regular_area);
    if !report.local_area.is_finite() || !weighted.is_finite() {
        return Err(invalid("Material-domain summary is not finite"));
    }
    Ok(report)
}
