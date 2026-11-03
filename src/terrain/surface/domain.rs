//! The only geometry-specific part of the common relief program.
use crate::{
    model::{Result, invalid},
    world_space::{CELL_METERS, Coordinates, Position},
};
use glam::DVec3;

#[derive(Debug, Clone, Copy)]
pub enum Domain {
    Plane {
        datum: Position,
    },
    Sphere {
        center: Position,
        radius_meters: f64,
    },
}

fn difference(a: Position, b: Position, limit: f64) -> Option<DVec3> {
    let a: Coordinates = a.into();
    let b: Coordinates = b.into();
    let mut out = [0.; 3];
    for (axis, value) in out.iter_mut().enumerate() {
        let cell = i128::from(a.cell[axis]) - i128::from(b.cell[axis]);
        if cell.abs() > (limit / CELL_METERS).ceil() as i128 + 1 {
            return None;
        }
        *value = cell as f64 * CELL_METERS + a.offset[axis] - b.offset[axis];
        if value.abs() > limit {
            return None;
        }
    }
    Some(DVec3::from_array(out))
}

#[derive(Clone, Copy)]
pub(super) enum Point {
    Plane(Position),
    Sphere(DVec3),
}

#[derive(Clone, Copy)]
pub(super) struct Anchor {
    point: Point,
    right: DVec3,
    forward: DVec3,
}

pub(super) struct Segment {
    start: Point,
    end: Point,
    tangent: DVec3,
    normal: DVec3,
    pub length: f64,
    pub prefix: f64,
    cone: (DVec3, f64),
}

impl Domain {
    pub fn validate(self) -> Result<()> {
        if let Self::Sphere { radius_meters, .. } = self
            && (!radius_meters.is_finite() || !(5. ..=5e11).contains(&radius_meters))
        {
            return Err(invalid("Surface sphere radius needs5..5e11 meters"));
        }
        Ok(())
    }
    pub(super) fn point(self, p: Position) -> Result<Point> {
        match self {
            Self::Plane { datum } => {
                let a: Coordinates = p.into();
                let b: Coordinates = datum.into();
                if a.cell[1] != b.cell[1] || a.offset[1] != b.offset[1] {
                    return Err(invalid(
                        "Surface plane anchors must share its vertical datum",
                    ));
                }
                Ok(Point::Plane(p))
            }
            Self::Sphere {
                center,
                radius_meters,
            } => {
                let d = difference(p, center, radius_meters * 2.)
                    .ok_or_else(|| invalid("Surface anchor is outside the reference sphere"))?;
                if (d.length() - radius_meters).abs() > (radius_meters * 1e-12).max(1e-4) {
                    return Err(invalid("Surface anchors must lie on the reference sphere"));
                }
                Ok(Point::Sphere(d.normalize()))
            }
        }
    }
    pub(super) fn anchor(self, position: Position) -> Result<Anchor> {
        let point = self.point(position)?;
        let (right, forward) = match point {
            Point::Plane(_) => (DVec3::X, DVec3::Z),
            Point::Sphere(up) => {
                // Fixed at the authored anchor, never selected per sample/face/chunk.
                let pole = if up.z.abs() < 0.9 { DVec3::Z } else { DVec3::X };
                let forward = (pole - up * pole.dot(up)).normalize();
                (up.cross(forward), forward)
            }
        };
        Ok(Anchor {
            point,
            right,
            forward,
        })
    }
    pub(super) fn check_support(self, meters: f64) -> Result<()> {
        if let Self::Sphere { radius_meters, .. } = self
            && meters >= radius_meters * std::f64::consts::FRAC_PI_2
        {
            return Err(invalid(
                "A local surface tool must fit inside a hemisphere; compose tools or use global fields",
            ));
        }
        Ok(())
    }
    pub(super) fn local(
        self,
        p: Point,
        a: Anchor,
        support: f64,
        min_cosine: f64,
    ) -> Option<[f64; 2]> {
        match (self, p, a.point) {
            (Self::Plane { .. }, Point::Plane(p), Point::Plane(a)) => {
                let d = difference(p, a, support)?;
                Some([d.x, d.z])
            }
            (Self::Sphere { radius_meters, .. }, Point::Sphere(p), Point::Sphere(a_point)) => {
                let cosine = a_point.dot(p).clamp(-1., 1.);
                if cosine + 1e-14 < min_cosine {
                    return None;
                }
                let tangent = p - a_point * cosine;
                let sine = tangent.length();
                if sine == 0. {
                    return Some([0., 0.]);
                }
                let log = tangent * (sine.atan2(cosine) * radius_meters / sine);
                Some([log.dot(a.right), log.dot(a.forward)])
            }
            _ => unreachable!("compiled surface domain"),
        }
    }
    pub(super) fn cosine(self, support: f64) -> f64 {
        match self {
            Self::Plane { .. } => -1.,
            Self::Sphere { radius_meters, .. } => (support / radius_meters).cos(),
        }
    }
    pub(super) fn segment(
        self,
        a: Position,
        b: Position,
        width: f64,
        prefix: f64,
    ) -> Result<Segment> {
        let start = self.point(a)?;
        let end = self.point(b)?;
        let (tangent, normal, length, cone) = match (start, end) {
            (Point::Plane(a), Point::Plane(b)) => {
                let d = difference(b, a, 1e12)
                    .ok_or_else(|| invalid("Surface curve segment too long"))?;
                let length = d.length();
                (d / length, DVec3::ZERO, length, (DVec3::ZERO, -1.))
            }
            (Point::Sphere(a), Point::Sphere(b)) => {
                let Self::Sphere {
                    radius_meters: r, ..
                } = self
                else {
                    unreachable!()
                };
                let cross = a.cross(b);
                let angle = cross.length().atan2(a.dot(b));
                if angle >= std::f64::consts::FRAC_PI_2 {
                    return Err(invalid(
                        "Split surface curve arcs longer than a quarter circumference",
                    ));
                }
                let normal = cross.normalize();
                (
                    normal.cross(a),
                    normal,
                    angle * r,
                    (
                        (a + b).normalize(),
                        (angle * 0.5 + width / r).min(std::f64::consts::PI).cos(),
                    ),
                )
            }
            _ => unreachable!(),
        };
        if !length.is_finite() || length < 1e-4 {
            return Err(invalid("Surface curve has a repeated/degenerate point"));
        }
        Ok(Segment {
            start,
            end,
            tangent,
            normal,
            length,
            prefix,
            cone,
        })
    }
    pub(super) fn segments(self, points: &[Position], width: f64) -> Result<Vec<Segment>> {
        let mut out = Vec::with_capacity(points.len().saturating_sub(1));
        let mut prefix = 0.;
        for pair in points.windows(2) {
            let s = self.segment(pair[0], pair[1], width, prefix)?;
            prefix += s.length;
            out.push(s);
        }
        Ok(out)
    }
    /// Distance, normalized along-position, signed cross-distance. End caps are continuous.
    pub(super) fn nearest(self, p: Point, s: &Segment, width: f64) -> Option<(f64, f64, f64)> {
        match (self, p, s.start, s.end) {
            (Self::Plane { .. }, Point::Plane(p), Point::Plane(a), _) => {
                let d = difference(p, a, s.length + width)?;
                let along = d.dot(s.tangent).clamp(0., s.length);
                let distance = (d - s.tangent * along).length();
                (distance < width).then_some((
                    distance,
                    along / s.length,
                    s.tangent.x * d.z - s.tangent.z * d.x,
                ))
            }
            (
                Self::Sphere {
                    radius_meters: r, ..
                },
                Point::Sphere(p),
                Point::Sphere(a),
                Point::Sphere(b),
            ) => {
                if p.dot(s.cone.0) + 1e-14 < s.cone.1 {
                    return None;
                }
                let theta = s.length / r;
                let angle = p.dot(s.tangent).atan2(p.dot(a));
                let along = if (0. ..=theta).contains(&angle) {
                    angle
                } else if p.dot(a) >= p.dot(b) {
                    0.
                } else {
                    theta
                };
                let closest = a * along.cos() + s.tangent * along.sin();
                let distance = p.cross(closest).length().atan2(p.dot(closest)) * r;
                let cross = -p.dot(s.normal).clamp(-1., 1.).asin() * r;
                (distance < width).then_some((distance, along / theta, cross))
            }
            _ => unreachable!(),
        }
    }
    pub(super) fn noise_position(self, p: Point) -> Position {
        match (self, p) {
            (Self::Plane { .. }, Point::Plane(p)) => {
                let mut coordinates: Coordinates = p.into();
                coordinates.cell[1] = 0;
                coordinates.offset[1] = 0.;
                Position::try_from(coordinates).expect("canonical plane noise coordinates")
            }
            (Self::Sphere { radius_meters, .. }, Point::Sphere(unit)) => {
                Position::normalize(Coordinates {
                    cell: [0; 3],
                    offset: (unit * radius_meters).to_array(),
                })
                .expect("validated planet-centric noise coordinates")
            }
            _ => unreachable!(),
        }
    }
    pub(super) fn unit_point(self, unit: DVec3) -> Point {
        Point::Sphere(unit)
    }
}
