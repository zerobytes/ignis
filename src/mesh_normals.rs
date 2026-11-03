//! Effective pre-material normals shared by rendering and explicit diagnostics.
use crate::{
    meshes::{Document, Shading},
    model::{Result, invalid},
};
use glam::{DVec3, Vec3};
use serde::Serialize;

pub(crate) fn accumulate(triangles: &[[u32; 3]], positions: &[[f32; 3]], out: &mut [Vec3]) {
    out.fill(Vec3::ZERO);
    for triangle in triangles {
        let [p, q, r] = triangle.map(|index| Vec3::from(positions[index as usize]));
        let area = (q - p).cross(r - p);
        for &index in triangle {
            out[index as usize] += area;
        }
    }
    for normal in out {
        *normal = normal.normalize_or_zero();
    }
}
pub(crate) fn face(positions: [Vec3; 3]) -> (Vec3, bool) {
    let [a, b, c] = positions;
    match (b - a).cross(c - a).try_normalize() {
        Some(normal) => (normal, false),
        None => (Vec3::Y, true),
    }
}
pub(crate) fn corner(smooth: bool, sum: Vec3, face: Vec3) -> Vec3 {
    if smooth && sum != Vec3::ZERO {
        sum
    } else {
        face
    }
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sample {
    pub face: u32,
    pub vertices: [u32; 3],
    pub positions: [[f32; 3]; 3],
    pub geometric_normal: [f32; 3],
    pub corner_normals: [[f32; 3]; 3],
    pub corner_degrees: [f64; 3],
    pub maximum_degrees: f64,
    pub geometric_fallback: bool,
    pub smooth_fallback: [bool; 3],
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    pub space: &'static str,
    pub stage: &'static str,
    pub tick: u64,
    pub asset: String,
    pub shading: Shading,
    pub selected_faces: usize,
    pub world_area: f64,
    pub maximum_degrees: f64,
    pub maximum_face: u32,
    pub geometric_fallback_faces: usize,
    pub smooth_fallback_corners: usize,
    pub opposing_corners: usize,
    pub sample_limit: u8,
    pub samples: Vec<Sample>,
}
fn angle(a: Vec3, b: Vec3) -> f64 {
    if a == b {
        return 0.;
    }
    let a = a.as_dvec3();
    let b = b.as_dvec3();
    (a.dot(b) / (a.length() * b.length()))
        .clamp(-1., 1.)
        .acos()
        .to_degrees()
}
/// Read-only posed normals; face filtering never changes the smoothing neighborhood.
pub fn inspect(
    doc: &Document,
    instance: &str,
    tick: u64,
    faces: Option<&[u32]>,
    sample_limit: u8,
) -> Result<Report> {
    if tick > 36000 || sample_limit > 32 {
        return Err(invalid(
            "Normal inspection requires tick<=36000 and sample_limit<=32",
        ));
    }
    let i = doc
        .instances
        .iter()
        .find(|i| i.id == instance)
        .ok_or_else(|| invalid("Unknown mesh instance"))?;
    let asset = doc.asset(&i.asset)?;
    let triangles = &asset.geometry.triangles;
    let selected = faces
        .map(|f| {
            if f.is_empty() || f.len() > triangles.len() {
                return Err(invalid(
                    "Normal inspection faces must be nonempty within the triangle budget",
                ));
            }
            let mut selected = f.to_vec();
            selected.sort_unstable();
            if selected
                .last()
                .is_some_and(|&f| f as usize >= triangles.len())
                || selected.windows(2).any(|s| s[0] == s[1])
            {
                return Err(invalid(
                    "Normal inspection contains missing or duplicate faces",
                ));
            }
            Ok(selected)
        })
        .transpose()?;
    let pose = doc.pose(instance, tick)?;
    let smooth = asset.shading == Shading::Smooth;
    let mut sums = vec![Vec3::ZERO; pose.positions.len()];
    if smooth {
        accumulate(
            triangles,
            &pose.positions,
            &mut sums[..pose.positions.len()],
        );
    }
    let mut report = Report {
        space: "world",
        stage: "pre_material",
        tick,
        asset: asset.id.clone(),
        shading: asset.shading,
        selected_faces: selected.as_ref().map_or(triangles.len(), Vec::len),
        world_area: 0.,
        maximum_degrees: 0.,
        maximum_face: selected.as_ref().map_or(0, |s| s[0]),
        geometric_fallback_faces: 0,
        smooth_fallback_corners: 0,
        opposing_corners: 0,
        sample_limit,
        samples: Vec::with_capacity(sample_limit as usize),
    };
    for n in 0..report.selected_faces {
        let index = selected.as_ref().map_or(n as u32, |s| s[n]);
        let triangle = triangles[index as usize];
        let positions = triangle.map(|i| pose.positions[i as usize]);
        let (normal, fallback) = face(positions.map(Vec3::from));
        let normals = triangle.map(|i| corner(smooth, sums[i as usize], normal));
        let angles = normals.map(|c| angle(c, normal));
        let maximum = angles.into_iter().fold(0., f64::max);
        let fallbacks = triangle.map(|i| smooth && sums[i as usize] == Vec3::ZERO);
        let [a, b, c] = positions.map(|p| DVec3::from_array(p.map(f64::from)));
        report.world_area += (b - a).cross(c - a).length() * 0.5;
        report.geometric_fallback_faces += usize::from(fallback);
        report.smooth_fallback_corners += fallbacks.into_iter().filter(|f| *f).count();
        report.opposing_corners += angles.into_iter().filter(|a| *a > 90.).count();
        if maximum > report.maximum_degrees {
            report.maximum_degrees = maximum;
            report.maximum_face = index;
        }
        let limit = sample_limit as usize;
        if limit > 0 {
            let at = report.samples.partition_point(|s| {
                s.maximum_degrees > maximum || (s.maximum_degrees == maximum && s.face < index)
            });
            if at < limit {
                if report.samples.len() == limit {
                    report.samples.pop();
                }
                report.samples.insert(
                    at,
                    Sample {
                        face: index,
                        vertices: triangle,
                        positions,
                        geometric_normal: normal.to_array(),
                        corner_normals: normals.map(|n| n.to_array()),
                        corner_degrees: angles,
                        maximum_degrees: maximum,
                        geometric_fallback: fallback,
                        smooth_fallback: fallbacks,
                    },
                );
            }
        }
    }
    Ok(report)
}
