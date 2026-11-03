//! Bounded authoring of convex profiles along open regular cubic chains.
use crate::{
    meshes::{Geometry, MAX_TRIANGLES, MAX_VERTICES},
    model::{Result, invalid},
};
use glam::{DQuat, DVec2, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub at: f32,
    pub scale: [f32; 2],
}
/// Scalar legacy sampling or one independently authored count per cubic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Spans {
    Uniform(u16),
    PerCurve(Vec<u16>),
}
impl From<u16> for Spans {
    fn from(value: u16) -> Self {
        Self::Uniform(value)
    }
}
impl Spans {
    fn validate(&self, curves: usize) -> Result<()> {
        let valid = match self {
            Self::Uniform(n) => (1..=128).contains(n),
            Self::PerCurve(ns) => ns.len() == curves && ns.iter().all(|n| (1..=128).contains(n)),
        };
        if !valid {
            return Err(invalid(
                "Sweep spans must be1..128 or one count1..128 per curve",
            ));
        }
        Ok(())
    }
    fn get(&self, curve: usize) -> u16 {
        match self {
            Self::Uniform(n) => *n,
            Self::PerCurve(ns) => ns[curve],
        }
    }
    fn intervals(&self, curves: usize) -> usize {
        match self {
            Self::Uniform(n) => curves * usize::from(*n),
            Self::PerCurve(ns) => ns.iter().map(|&n| usize::from(n)).sum(),
        }
    }
    fn uniform(&self) -> Option<u16> {
        match self {
            Self::Uniform(n) => Some(*n),
            Self::PerCurve(ns) => ns.first().copied().filter(|n| ns.iter().all(|x| x == n)),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sweep {
    pub profile: Vec<[f32; 2]>,
    pub curves: Vec<[[f32; 3]; 4]>,
    pub spans: Spans,
    pub profile_x: [f32; 3],
    #[serde(default = "unit_sections")]
    pub sections: Vec<Section>,
    #[serde(default = "yes")]
    pub capped: bool,
    pub centerline_tolerance: f32,
}
fn yes() -> bool {
    true
}
fn unit_sections() -> Vec<Section> {
    vec![
        Section {
            at: 0.,
            scale: [1.; 2],
        },
        Section {
            at: 1.,
            scale: [1.; 2],
        },
    ]
}
fn v(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}
fn p2(p: [f32; 2]) -> DVec2 {
    DVec2::from_array(p.map(f64::from))
}
fn finite(p: &[f32], limit: f32) -> bool {
    p.iter().all(|x| x.is_finite() && x.abs() <= limit)
}
fn position(p: [DVec3; 4], t: f64) -> DVec3 {
    let s = 1. - t;
    p[0] * s.powi(3) + p[1] * (3. * s * s * t) + p[2] * (3. * s * t * t) + p[3] * t.powi(3)
}
fn derivative(p: [DVec3; 4], t: f64) -> DVec3 {
    let s = 1. - t;
    (p[1] - p[0]) * (3. * s * s) + (p[2] - p[1]) * (6. * s * t) + (p[3] - p[2]) * (3. * t * t)
}
/// Conservative lower bound on the derivative norm, with bounded subdivision.
fn regular(q: [DVec3; 3], depth: u8) -> bool {
    let lo = q[0].min(q[1]).min(q[2]);
    let hi = q[0].max(q[1]).max(q[2]);
    let nearest = lo.max(DVec3::ZERO) + hi.min(DVec3::ZERO);
    if nearest.length_squared() > 1e-14 {
        return true;
    }
    if depth == 12 {
        return false;
    }
    let a = (q[0] + q[1]) * 0.5;
    let b = (q[1] + q[2]) * 0.5;
    let mid = (a + b) * 0.5;
    regular([q[0], a, mid], depth + 1) && regular([mid, b, q[2]], depth + 1)
}
fn segment_distance(p: DVec3, a: DVec3, b: DVec3) -> f64 {
    let d = b - a;
    let fraction = ((p - a).dot(d) / d.length_squared()).clamp(0., 1.);
    (p - a - d * fraction).length()
}
fn curve_bound(curve: &[[f32; 3]; 4], spans: u16) -> f64 {
    let h = 1. / f64::from(spans);
    let mut bound = 0f64;
    let p = curve.map(v);
    for i in 0..spans {
        let t = f64::from(i) * h;
        let a = position(p, t);
        let b = position(p, t + h);
        if (b - a).length_squared() < 1e-18 {
            return f64::INFINITY;
        }
        let q1 = a + derivative(p, t) * (h / 3.);
        let q2 = b - derivative(p, t + h) * (h / 3.);
        bound = bound
            .max(segment_distance(q1, a, b))
            .max(segment_distance(q2, a, b));
    }
    bound
}
#[derive(Debug, Serialize)]
pub struct PiecePlan {
    pub requested_spans: Spans,
    pub spans: Spans,
    pub vertices: usize,
    pub triangles: usize,
    pub centerline_deviation_bound: f64,
    pub curve_centerline_bounds: Vec<f64>,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub pieces: Vec<PiecePlan>,
    pub vertices: usize,
    pub triangles: usize,
    pub surface_validated: bool,
}
pub fn plan(sweeps: &[Sweep]) -> Result<Report> {
    counts(sweeps)?;
    let mut report = Report {
        pieces: Vec::with_capacity(sweeps.len()),
        vertices: 0,
        triangles: 0,
        surface_validated: false,
    };
    for (index, s) in sweeps.iter().enumerate() {
        let tolerance = f64::from(s.centerline_tolerance);
        let refine = |curve: &[[f32; 3]; 4], mut n: u16| -> Result<(u16, f64)> {
            loop {
                let bound = curve_bound(curve, n);
                if bound <= tolerance {
                    return Ok((n, bound));
                }
                if n == 128 {
                    return Err(invalid(format!(
                        "Sweep piece{index}: centerline tolerance exceeds maximum128 spans"
                    )));
                }
                n = (n * 2).min(128);
            }
        };
        let (spans, bounds) = match &s.spans {
            Spans::Uniform(requested) => {
                let mut n = *requested;
                loop {
                    let bounds: Vec<_> = s.curves.iter().map(|c| curve_bound(c, n)).collect();
                    if bounds.iter().all(|&b| b <= tolerance) {
                        break (Spans::Uniform(n), bounds);
                    }
                    if n == 128 {
                        return Err(invalid(format!(
                            "Sweep piece{index}: centerline tolerance exceeds maximum128 spans"
                        )));
                    }
                    n = (n * 2).min(128);
                }
            }
            Spans::PerCurve(requested) => {
                let mut counts = Vec::with_capacity(s.curves.len());
                let mut bounds = Vec::with_capacity(s.curves.len());
                for (curve, &n) in s.curves.iter().zip(requested) {
                    let (n, b) = refine(curve, n)?;
                    counts.push(n);
                    bounds.push(b);
                }
                (Spans::PerCurve(counts), bounds)
            }
        };
        let bound = bounds.iter().copied().fold(0., f64::max);
        let rings = spans.intervals(s.curves.len()) + 1;
        let sides = s.profile.len();
        let vertices = rings * sides;
        let triangles = 2 * (rings - 1) * sides + usize::from(s.capped) * 2 * (sides - 2);
        report.vertices += vertices;
        report.triangles += triangles;
        if report.vertices > MAX_VERTICES || report.triangles > MAX_TRIANGLES {
            return Err(invalid("Sweep planned resolution exceeds mesh budget"));
        }
        report.pieces.push(PiecePlan {
            requested_spans: s.spans.clone(),
            spans,
            vertices,
            triangles,
            centerline_deviation_bound: bound,
            curve_centerline_bounds: bounds,
        });
    }
    Ok(report)
}
impl Sweep {
    fn validate(&self) -> Result<(usize, usize)> {
        let n = self.profile.len();
        if !(3..=32).contains(&n)
            || !(1..=16).contains(&self.curves.len())
            || !(2..=32).contains(&self.sections.len())
            || !self.centerline_tolerance.is_finite()
            || !(0.000001..=10.).contains(&self.centerline_tolerance)
            || !finite(&self.profile_x, 10000.)
            || self.profile.iter().any(|p| !finite(p, 100.))
        {
            return Err(invalid(
                "Sweep requires profile3..32, curves1..16, spans1..128, sections2..32 and finite tolerance0.000001..10m",
            ));
        }
        self.spans.validate(self.curves.len())?;
        // Every other point must be strictly left of every oriented edge.
        // This also excludes repeated vertices, self-crossings and star ordering.
        for i in 0..n {
            let a = p2(self.profile[i]);
            let edge = p2(self.profile[(i + 1) % n]) - a;
            if edge.perp_dot(-a) <= 1e-12 {
                return Err(invalid(
                    "Sweep profile must be CCW, strictly convex and contain the origin",
                ));
            }
            for j in 0..n {
                if j != i && j != (i + 1) % n && edge.perp_dot(p2(self.profile[j]) - a) <= 1e-12 {
                    return Err(invalid(
                        "Sweep profile must be strictly convex with distinct ordered vertices",
                    ));
                }
            }
        }
        if self.sections.first().unwrap().at != 0. || self.sections.last().unwrap().at != 1. {
            return Err(invalid("Sweep sections must include endpoints0 and1"));
        }
        for (i, s) in self.sections.iter().enumerate() {
            if !s.at.is_finite()
                || !(0. ..=1.).contains(&s.at)
                || s.scale
                    .iter()
                    .any(|x| !x.is_finite() || !(0.001..=100.).contains(x))
                || i > 0 && self.sections[i - 1].at >= s.at
            {
                return Err(invalid(
                    "Sweep sections require increasing positions0..1 and scales0.001..100",
                ));
            }
        }
        if self.curves[0][0] == self.curves.last().unwrap()[3] {
            return Err(invalid(
                "Sweep requires an open chain; closed-path frames and caps need explicit authoring",
            ));
        }
        for (i, curve) in self.curves.iter().enumerate() {
            if curve.iter().any(|p| !finite(p, 10000.)) {
                return Err(invalid("Sweep controls must be finite within10000m"));
            }
            let p = curve.map(v);
            if !regular(
                [(p[1] - p[0]) * 3., (p[2] - p[1]) * 3., (p[3] - p[2]) * 3.],
                0,
            ) {
                return Err(invalid(
                    "Sweep derivative is singular or regularity could not be certified",
                ));
            }
            if i > 0 {
                let previous = self.curves[i - 1].map(v);
                if self.curves[i - 1][3] != curve[0]
                    || derivative(previous, 1.)
                        .normalize()
                        .dot(derivative(p, 0.).normalize())
                        < 1. - 1e-6
                {
                    return Err(invalid(
                        "Sweep curves require shared endpoints and matching tangent directions; split intentional corners into pieces",
                    ));
                }
            }
        }
        let tangent = derivative(self.curves[0].map(v), 0.).normalize();
        let axis = v(self.profile_x);
        if (axis - tangent * axis.dot(tangent)).length_squared() < 1e-14 {
            return Err(invalid(
                "Sweep profile_x must not be parallel to the initial tangent",
            ));
        }
        let rings = self.spans.intervals(self.curves.len()) + 1;
        Ok((
            rings * n,
            2 * (rings - 1) * n + usize::from(self.capped) * 2 * (n - 2),
        ))
    }
}

pub fn counts(sweeps: &[Sweep]) -> Result<(usize, usize)> {
    if !(1..=256).contains(&sweeps.len()) {
        return Err(invalid("Sweep requires1..256 pieces"));
    }
    let (mut vertices, mut triangles) = (0, 0);
    for (index, s) in sweeps.iter().enumerate() {
        let (v, t) = s
            .validate()
            .map_err(|error| invalid(format!("Sweep piece{index}: {}", error.message)))?;
        vertices += v;
        triangles += t;
        if vertices > MAX_VERTICES || triangles > MAX_TRIANGLES {
            return Err(invalid("Sweep exceeds mesh vertex/triangle budget"));
        }
    }
    Ok((vertices, triangles))
}

pub(crate) struct Plan<'a> {
    sweeps: &'a [Sweep],
    pub vertices: usize,
    pub triangles: usize,
}
impl<'a> Plan<'a> {
    pub(crate) fn new(sweeps: &'a [Sweep]) -> Result<Self> {
        let (vertices, triangles) = counts(sweeps)?;
        for (index, s) in sweeps.iter().enumerate() {
            let bound = s
                .curves
                .iter()
                .enumerate()
                .map(|(i, c)| curve_bound(c, s.spans.get(i)))
                .fold(0., f64::max);
            if bound > f64::from(s.centerline_tolerance) {
                return Err(invalid(format!(
                    "Sweep piece{index}: centerline bound{bound:.6}m exceeds tolerance{}m; increase spans or use plan_mesh_sweep",
                    s.centerline_tolerance
                )));
            }
        }
        Ok(Self {
            sweeps,
            vertices,
            triangles,
        })
    }
    pub(crate) fn generate(self) -> Result<Geometry> {
        self.append_to(Geometry {
            positions: Vec::new(),
            triangles: Vec::new(),
            material_coordinates: Some(Vec::new()),
        })
    }
    pub(crate) fn append_to(self, mut geometry: Geometry) -> Result<Geometry> {
        geometry.positions.reserve(self.vertices);
        geometry.triangles.reserve(self.triangles);
        let coordinates = geometry
            .material_coordinates
            .get_or_insert_with(|| geometry.positions.clone());
        coordinates.reserve(self.vertices);
        for (index, sweep) in self.sweeps.iter().enumerate() {
            append(&mut geometry, sweep)
                .map_err(|error| invalid(format!("Sweep piece{index}: {}", error.message)))?;
        }
        geometry.validate()?;
        Ok(geometry)
    }
}
/// A sampling constructor rather than a persistent parametric source.
pub fn generate(sweeps: &[Sweep]) -> Result<Geometry> {
    Plan::new(sweeps)?.generate()
}
struct Ring {
    point: DVec3,
    tangent: DVec3,
    x: DVec3,
    y: DVec3,
    scale: DVec2,
}
fn append(g: &mut Geometry, s: &Sweep) -> Result<()> {
    let sides = s.profile.len();
    let start = g.positions.len() as u32;
    let steps = s.spans.intervals(s.curves.len());
    let uniform = s.spans.uniform();
    let mut curve_cursor = 0;
    let mut curve_start = 0;
    let mut previous: Option<Ring> = None;
    let mut distance = 0.;
    let mut station = 0;
    for ring in 0..=steps {
        let (curve_index, t, at) = if let Some(spans) = uniform {
            // Keep scalar arithmetic, including rounding, byte-identical.
            let i = (ring / usize::from(spans)).min(s.curves.len() - 1);
            let t = (ring - i * usize::from(spans)) as f64 / f64::from(spans);
            (i, t, ring as f64 / steps as f64)
        } else {
            while curve_cursor + 1 < s.curves.len()
                && ring >= curve_start + usize::from(s.spans.get(curve_cursor))
            {
                curve_start += usize::from(s.spans.get(curve_cursor));
                curve_cursor += 1;
            }
            let t = (ring - curve_start) as f64 / f64::from(s.spans.get(curve_cursor));
            (
                curve_cursor,
                t,
                (curve_cursor as f64 + t) / s.curves.len() as f64,
            )
        };
        let p = s.curves[curve_index].map(v);
        let point = position(p, t);
        let tangent = derivative(p, t).normalize();
        let mut x = if let Some(prev) = &previous {
            if prev.tangent.dot(tangent) < -1. + 1e-8 {
                return Err(invalid(
                    "Sweep sampled frame reverses; increase spans or split the path",
                ));
            }
            DQuat::from_rotation_arc(prev.tangent, tangent) * prev.x
        } else {
            v(s.profile_x)
        };
        x -= tangent * x.dot(tangent);
        if x.length_squared() < 1e-14 {
            return Err(invalid(
                "Sweep profile_x must not be parallel to the initial tangent",
            ));
        }
        x = x.normalize();
        let y = tangent.cross(x);
        while station + 2 < s.sections.len() && at > f64::from(s.sections[station + 1].at) {
            station += 1;
        }
        let a = &s.sections[station];
        let b = &s.sections[station + 1];
        let fraction = (at - f64::from(a.at)) / (f64::from(b.at) - f64::from(a.at));
        let scale = p2(a.scale).lerp(p2(b.scale), fraction);
        if let Some(prev) = &previous {
            let length = (point - prev.point).length();
            if length < 1e-9 {
                return Err(invalid("Sweep adjacent sampled centers must be distinct"));
            }
            distance += length;
        }
        let current = Ring {
            point,
            tangent,
            x,
            y,
            scale,
        };
        for profile in &s.profile {
            let uv = p2(*profile) * scale;
            let position = point + x * uv.x + y * uv.y;
            g.positions.push(position.as_vec3().to_array());
            g.material_coordinates.as_mut().unwrap().push([
                uv.x as f32,
                distance as f32,
                uv.y as f32,
            ]);
        }
        if let Some(prev) = &previous {
            for j in 0..sides {
                let a = start + ((ring - 1) * sides + j) as u32;
                let b = start + ((ring - 1) * sides + (j + 1) % sides) as u32;
                let c = a + sides as u32;
                let d = b + sides as u32;
                let edge = p2(s.profile[(j + 1) % sides]) - p2(s.profile[j]);
                let outward = |r: &Ring| r.x * (edge.y / r.scale.x) - r.y * (edge.x / r.scale.y);
                let normal = outward(prev) + outward(&current);
                for face in [[a, b, c], [b, d, c]] {
                    let [a, b, c] = face.map(|i| v(g.positions[i as usize]));
                    if (b - a).cross(c - a).dot(normal) <= 1e-16 {
                        return Err(invalid(
                            "Sweep side folds or degenerates; reduce section, increase spans or split path",
                        ));
                    }
                    g.triangles.push(face);
                }
            }
        }
        previous = Some(current);
    }
    if s.capped {
        let last = start + (steps * sides) as u32;
        for j in 1..sides as u32 - 1 {
            g.triangles.extend([
                [start, start + j + 1, start + j],
                [last, last + j, last + j + 1],
            ]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_topology::Topology;

    fn straight() -> Sweep {
        Sweep {
            profile: vec![[-0.1, -0.2], [0.1, -0.2], [0.1, 0.2], [-0.1, 0.2]],
            curves: vec![[[0., 0., 0.], [0., 1., 0.], [0., 2., 0.], [0., 3., 0.]]],
            spans: 8.into(),
            profile_x: [1., 0., 0.],
            sections: unit_sections(),
            capped: true,
            centerline_tolerance: 0.001,
        }
    }
    #[test]
    fn straight_section_caps_material_and_winding() {
        let s = straight();
        let g = generate(&[s]).unwrap();
        assert_eq!((g.positions.len(), g.triangles.len()), (36, 68));
        assert_eq!(g.positions[0], [-0.1, 0., 0.2]);
        assert_eq!(g.positions[32], [-0.1, 3., 0.2]);
        assert_eq!(
            g.material_coordinates.as_ref().unwrap()[32],
            [-0.1, 3., -0.2]
        );
        let t = Topology::inspect(&g).unwrap();
        assert_eq!(t.boundary_edges, 0);
        assert_eq!(t.non_manifold_edges, 0);
        let volume: f64 = g
            .triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| v(g.positions[i as usize]));
                a.dot(b.cross(c)) / 6.
            })
            .sum();
        assert!((volume - 0.24).abs() < 1e-7);
    }
    #[test]
    fn open_caps_and_disjoint_pieces() {
        let mut s = straight();
        s.capped = false;
        let g = generate(&[s.clone(), s]).unwrap();
        assert_eq!((g.positions.len(), g.triangles.len()), (72, 128));
        assert_eq!(Topology::inspect(&g).unwrap().boundary_edges, 16);
    }
    #[test]
    fn regular_3d_curve_transports_asymmetric_profile_and_scales() {
        let mut s = straight();
        s.curves = vec![[[0., 0., 0.], [0., 1., 1.], [1., 2., 1.], [1., 3., 0.]]];
        s.spans = 32.into();
        s.centerline_tolerance = 0.01;
        s.sections[1].scale = [2., 0.5];
        let g = generate(&[s.clone()]).unwrap();
        assert_eq!(g, generate(&[s]).unwrap());
        let end = v(g.positions[g.positions.len() - 4]);
        assert!(
            ((end - DVec3::new(1., 3., 0.)).length() - (0.2f64.powi(2) + 0.1f64.powi(2)).sqrt())
                .abs()
                < 1e-6
        );
        assert_eq!(Topology::inspect(&g).unwrap().boundary_edges, 0);
    }
    #[test]
    fn joined_chain_has_one_shared_ring_and_linear_section_times() {
        let mut s = straight();
        s.curves
            .push([[0., 3., 0.], [0., 4., 0.], [0., 5., 0.], [0., 6., 0.]]);
        s.sections.insert(
            1,
            Section {
                at: 0.5,
                scale: [2., 2.],
            },
        );
        let g = generate(&[s]).unwrap();
        assert_eq!(g.positions.len(), 68);
        assert_eq!(g.positions[32], [-0.2, 3., 0.4]);
        assert_eq!(Topology::inspect(&g).unwrap().non_manifold_edges, 0);
    }
    #[test]
    fn planner_refines_with_a_certified_bound_but_does_not_certify_surface() {
        let mut s = straight();
        s.curves = vec![[[0., 0., 0.], [0., 1., 1.], [1., 2., 1.], [1., 3., 0.]]];
        s.spans = 1.into();
        let report = plan(&[s.clone()]).unwrap();
        assert!(!report.surface_validated);
        let chosen = &report.pieces[0];
        assert!(chosen.spans.get(0) > s.spans.get(0));
        assert!(chosen.centerline_deviation_bound <= f64::from(s.centerline_tolerance));
        s.spans = chosen.spans.clone();
        let g = generate(&[s.clone()]).unwrap();
        assert_eq!(g.positions.len(), report.vertices);
        let p = s.curves[0].map(v);
        let n = f64::from(s.spans.get(0));
        for i in 0..=1000 {
            let t = f64::from(i) / 1000.;
            let lo = (t * n).floor().min(n - 1.) / n;
            assert!(
                segment_distance(position(p, t), position(p, lo), position(p, lo + 1. / n))
                    <= chosen.centerline_deviation_bound + 1e-10
            );
        }
        s.profile = vec![[-20., -20.], [20., -20.], [20., 20.], [-20., 20.]];
        assert!(plan(&[s.clone()]).is_ok());
        assert!(generate(&[s]).is_err());
    }
    #[test]
    fn closed_regular_chain_is_not_silently_capped_twice() {
        let mut s = straight();
        let k = 0.5522848;
        s.profile_x = [0., 0., 1.];
        s.curves = vec![
            [[1., 0., 0.], [1., k, 0.], [k, 1., 0.], [0., 1., 0.]],
            [[0., 1., 0.], [-k, 1., 0.], [-1., k, 0.], [-1., 0., 0.]],
            [[-1., 0., 0.], [-1., -k, 0.], [-k, -1., 0.], [0., -1., 0.]],
            [[0., -1., 0.], [k, -1., 0.], [1., -k, 0.], [1., 0., 0.]],
        ];
        assert!(plan(&[s.clone()]).is_err());
        assert!(generate(&[s]).is_err());
    }
    #[test]
    fn rejects_profile_controls_sections_frames_sampling_and_budgets() {
        let mut bad = Vec::new();
        let base = straight();
        let mut s = base.clone();
        s.profile.reverse();
        bad.push(s);
        let mut s = base.clone();
        s.profile[1] = s.profile[0];
        bad.push(s);
        let mut s = base.clone();
        s.profile[0] = [0., 0.];
        bad.push(s);
        let mut s = base.clone();
        s.sections[1].at = 0.;
        bad.push(s);
        let mut s = base.clone();
        s.sections[1].scale = [0., 1.];
        bad.push(s);
        let mut s = base.clone();
        s.profile_x = [0., 1., 0.];
        bad.push(s);
        let mut s = base.clone();
        s.curves[0][1] = s.curves[0][0];
        bad.push(s);
        let mut s = base.clone();
        s.curves[0][2] = [0., -1., 0.];
        bad.push(s);
        let mut s = base.clone();
        s.curves[0][1] = [f32::NAN, 1., 0.];
        bad.push(s);
        let mut s = base.clone();
        s.spans = 0.into();
        bad.push(s);
        let mut s = base.clone();
        s.curves
            .push([[0., 4., 0.], [0., 5., 0.], [0., 6., 0.], [0., 7., 0.]]);
        bad.push(s);
        let mut s = base.clone();
        s.curves = vec![[[0., 0., 0.], [0., 1., 1.], [1., 2., 1.], [1., 3., 0.]]];
        s.spans = 1.into();
        bad.push(s);
        for s in bad {
            assert!(generate(&[s]).is_err());
        }
        assert!(generate(&[]).is_err());
        assert!(counts(&vec![base.clone(); 256]).is_ok());
        assert!(counts(&vec![base; 257]).is_err());
        let mut huge = straight();
        huge.spans = 128.into();
        huge.curves = (0..16)
            .map(|i| {
                let y = i as f32 * 3.;
                [
                    [0., y, 0.],
                    [0., y + 1., 0.],
                    [0., y + 2., 0.],
                    [0., y + 3., 0.],
                ]
            })
            .collect();
        assert!(counts(&[huge.clone(), huge.clone(), huge]).is_err());
    }
    fn two_curves() -> Sweep {
        let mut s = straight();
        s.curves
            .push([[0., 3., 0.], [0., 4., 0.], [0., 5., 0.], [0., 6., 0.]]);
        s
    }
    #[test]
    fn scalar_and_uniform_array_are_exactly_equal_including_section_rounding() {
        let mut s = two_curves();
        s.sections.insert(
            1,
            Section {
                at: 0.37,
                scale: [1.2, 0.7],
            },
        );
        let scalar = generate(&[s.clone()]).unwrap();
        s.spans = Spans::PerCurve(vec![8, 8]);
        assert_eq!(scalar, generate(&[s.clone()]).unwrap());
        let p = plan(&[s]).unwrap();
        assert_eq!(p.pieces[0].spans, Spans::PerCurve(vec![8, 8]));
        assert_eq!(p.pieces[0].curve_centerline_bounds.len(), 2);
    }
    #[test]
    fn unequal_sampling_shares_knots_and_preserves_chain_section_parameter() {
        let mut s = two_curves();
        s.spans = Spans::PerCurve(vec![1, 8]);
        s.sections.insert(
            1,
            Section {
                at: 0.5,
                scale: [2., 2.],
            },
        );
        let g = generate(&[s]).unwrap();
        assert_eq!((g.positions.len(), g.triangles.len()), (40, 76));
        // The first knot is ring1, but station0.5 still belongs to the knot.
        assert_eq!(g.positions[4], [-0.2, 3., 0.4]);
        assert!((v(g.positions[20]) - DVec3::new(-0.15, 4.5, 0.3)).length() < 1e-7);
        let topology = Topology::inspect(&g).unwrap();
        assert_eq!(topology.boundary_edges, 0);
        assert_eq!(topology.non_manifold_edges, 0);
        assert_eq!(g.material_coordinates.as_ref().unwrap()[4][1], 3.);
    }
    #[test]
    fn explicit_planner_refines_only_curved_segments_with_dense_error_check() {
        let mut s = two_curves();
        s.curves[1] = [[0., 3., 0.], [0., 4., 0.], [1., 5., 1.], [1., 6., 1.]];
        s.spans = Spans::PerCurve(vec![1, 1]);
        let report = plan(&[s.clone()]).unwrap();
        let chosen = &report.pieces[0];
        assert_eq!(chosen.spans.get(0), 1);
        assert!(chosen.spans.get(1) > 1);
        assert!(
            chosen
                .curve_centerline_bounds
                .iter()
                .all(|b| *b <= f64::from(s.centerline_tolerance))
        );
        for (i, curve) in s.curves.iter().enumerate() {
            let n = f64::from(chosen.spans.get(i));
            let p = curve.map(v);
            for k in 0..=1000 {
                let t = f64::from(k) / 1000.;
                let lo = (t * n).floor().min(n - 1.) / n;
                assert!(
                    segment_distance(position(p, t), position(p, lo), position(p, lo + 1. / n))
                        <= chosen.curve_centerline_bounds[i] + 1e-10
                );
            }
        }
        s.spans = chosen.spans.clone();
        assert_eq!(
            generate(&[s.clone()]).unwrap().positions.len(),
            report.vertices
        );
        let mut scalar = s.clone();
        scalar.spans = Spans::Uniform(1);
        let uniform = plan(&[scalar]).unwrap();
        assert!(uniform.vertices > report.vertices);
        assert_eq!(uniform.pieces[0].spans.get(0), chosen.spans.get(1));
    }
    #[test]
    fn per_curve_counts_and_input_shapes_are_strict_and_bounded() {
        let base = two_curves();
        for spans in [vec![], vec![1], vec![1, 0], vec![1, 129], vec![1, 1, 1]] {
            let mut s = base.clone();
            s.spans = Spans::PerCurve(spans);
            assert!(counts(&[s.clone()]).is_err());
            assert!(plan(&[s.clone()]).is_err());
            assert!(generate(&[s]).is_err());
        }
        for raw in [
            serde_json::json!(-1),
            serde_json::json!([1, 1.5]),
            serde_json::json!("8"),
            serde_json::json!(null),
        ] {
            assert!(serde_json::from_value::<Spans>(raw).is_err());
        }
        let mut huge = straight();
        huge.curves = (0..16)
            .map(|i| {
                let y = i as f32 * 3.;
                [
                    [0., y, 0.],
                    [0., y + 1., 0.],
                    [0., y + 2., 0.],
                    [0., y + 3., 0.],
                ]
            })
            .collect();
        huge.spans = Spans::PerCurve(vec![128; 16]);
        huge.profile = vec![[-0.1, -0.1], [0.1, -0.1], [0.1, 0.1], [-0.1, 0.1]];
        assert!(counts(&[huge.clone(), huge.clone(), huge]).is_err());
    }
}
