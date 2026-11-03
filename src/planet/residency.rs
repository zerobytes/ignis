//! Typed spherical addresses and camera-relative frames, shared by render/physics residency.
//! Planning does not claim that resources have been installed in either backend.
use super::{Document, Pose, mesh::Face};
use crate::model::{Result, invalid};
use glam::{DMat3, DQuat, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const FACES: [Face; 6] = [
    Face::PositiveX,
    Face::NegativeX,
    Face::PositiveY,
    Face::NegativeY,
    Face::PositiveZ,
    Face::NegativeZ,
];
pub const MAX_TILES: usize = 4096;

/// Address is independent of the resident origin; the source digest is a separate cache key.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Tile {
    pub face: Face,
    pub level: u8,
    pub xy: [u32; 2],
}
impl Tile {
    pub fn validate(self) -> Result<()> {
        if self.level > 24 || self.xy.iter().any(|&x| u64::from(x) >= 1u64 << self.level) {
            return Err(invalid(
                "Planet tile requires level0..24 and coordinates inside2^level",
            ));
        }
        Ok(())
    }
    pub fn direction(self, uv: [f64; 2]) -> DVec3 {
        let n = (1u64 << self.level) as f64;
        let [u, v] = std::array::from_fn(|i| 2. * (f64::from(self.xy[i]) + uv[i]) / n - 1.);
        match self.face {
            Face::PositiveX => DVec3::new(1., v, -u),
            Face::NegativeX => DVec3::new(-1., v, u),
            Face::PositiveY => DVec3::new(u, 1., -v),
            Face::NegativeY => DVec3::new(u, -1., v),
            Face::PositiveZ => DVec3::new(u, v, 1.),
            Face::NegativeZ => DVec3::new(-u, v, -1.),
        }
        .normalize()
    }
    pub fn children(self) -> [Self; 4] {
        std::array::from_fn(|i| Self {
            face: self.face,
            level: self.level + 1,
            xy: [
                self.xy[0] * 2 + (i as u32 & 1),
                self.xy[1] * 2 + (i as u32 >> 1),
            ],
        })
    }
    /// A positive cone enclosing the whole normalized cube-face rectangle.
    pub fn cone(self) -> (DVec3, f64) {
        let center = self.direction([0.5, 0.5]);
        let cosine = [[0., 0.], [1., 0.], [0., 1.], [1., 1.]]
            .into_iter()
            .map(|uv| center.dot(self.direction(uv)))
            .fold(1., f64::min);
        (center, cosine.clamp(-1., 1.).acos() + 64. * f64::EPSILON)
    }
    pub fn area(self, subdivisions: u16) -> super::mesh::Area {
        super::mesh::Area::Tile {
            face: self.face,
            level: self.level,
            tile: self.xy,
            subdivisions,
        }
    }
    pub fn label(self) -> String {
        let f = match self.face {
            Face::PositiveX => "px",
            Face::NegativeX => "nx",
            Face::PositiveY => "py",
            Face::NegativeY => "ny",
            Face::PositiveZ => "pz",
            Face::NegativeZ => "nz",
        };
        format!("{f}_{}_{}_{}", self.level, self.xy[0], self.xy[1])
    }
}

/// An orthonormal tangent frame. Absolute universe coordinates never enter f32.
#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub pose: Pose,
    pub radius: f64,
    pub basis: DMat3,
}
impl Anchor {
    pub fn new(document: &Document, pose: Pose) -> Result<Self> {
        let f = pose.frame()?;
        if !pose.altitude_meters.is_finite() || document.radius() + pose.altitude_meters <= 0. {
            return Err(invalid("Planet frame must lie outside the center"));
        }
        Ok(Self {
            pose: Pose {
                direction: f.up,
                forward: f.forward,
                ..pose
            },
            radius: document.radius(),
            basis: DMat3::from_cols(f.right.into(), f.up.into(), -DVec3::from(f.forward)),
        })
    }
    pub fn project(self, unit: DVec3, height: f64) -> DVec3 {
        let up = self.basis.y_axis;
        let delta = unit - up;
        DVec3::new(
            self.radius * delta.dot(self.basis.x_axis) + height * unit.dot(self.basis.x_axis),
            -self.radius * delta.length_squared() * 0.5 + height * unit.dot(up)
                - self.pose.altitude_meters,
            self.radius * delta.dot(self.basis.z_axis) + height * unit.dot(self.basis.z_axis),
        )
    }
    pub fn unproject(self, local: DVec3) -> (DVec3, f64) {
        let delta = self.basis * local + self.basis.y_axis * self.pose.altitude_meters;
        // The offset is local. Cancellation affects only height, never a huge global center.
        let p = self.basis.y_axis * self.radius + delta;
        let length = p.length();
        let height = (2. * self.radius * self.basis.y_axis.dot(delta) + delta.length_squared())
            / (length + self.radius);
        (p / length, height)
    }
    pub fn rotation_to(self, next: Self) -> DQuat {
        DQuat::from_mat3(&(next.basis.transpose() * self.basis)).normalize()
    }
    pub fn origin_in(self, next: Self) -> DVec3 {
        next.project(self.basis.y_axis, self.pose.altitude_meters)
    }
    pub fn point_in(self, next: Self, p: DVec3) -> DVec3 {
        next.basis.transpose() * self.basis * p + self.origin_in(next)
    }
    pub fn transported(self, document: &Document, unit: DVec3, height: f64) -> Result<Self> {
        let q = DQuat::from_rotation_arc(self.basis.y_axis, unit);
        Self::new(
            document,
            Pose {
                direction: unit.to_array(),
                forward: (q * -self.basis.z_axis).to_array(),
                altitude_meters: height,
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub pose: Pose,
    pub level: u8,
    pub visual_radius_meters: f64,
    pub physical_radius_meters: f64,
}
#[derive(Debug, Serialize)]
pub struct Plan {
    pub visual: Vec<Tile>,
    pub physical: Vec<Tile>,
    pub resources_installed: bool,
}
fn select(
    tile: Tile,
    level: u8,
    viewer: DVec3,
    angle: f64,
    out: &mut BTreeSet<Tile>,
) -> Result<()> {
    let (center, cone) = tile.cone();
    if angle + cone < std::f64::consts::PI
        && viewer.dot(center) + 128. * f64::EPSILON < (angle + cone).cos()
    {
        return Ok(());
    }
    if tile.level == level {
        if out.len() == MAX_TILES {
            return Err(invalid(
                "Planet residence request exceeds4096tiles; keep previous coverage",
            ));
        }
        out.insert(tile);
    } else {
        for child in tile.children() {
            select(child, level, viewer, angle, out)?;
        }
    }
    Ok(())
}
pub fn plan(document: &Document, request: &Request) -> Result<Plan> {
    document.validate()?;
    let anchor = Anchor::new(document, request.pose)?;
    if request.level > 24
        || !request.visual_radius_meters.is_finite()
        || !request.physical_radius_meters.is_finite()
        || request.physical_radius_meters <= 0.
        || request.visual_radius_meters < request.physical_radius_meters
    {
        return Err(invalid(
            "Planet residence needs level0..24 and finite visual radius>=physical radius>0",
        ));
    }
    let r = document.radius() + request.pose.altitude_meters;
    let mut sets = [BTreeSet::new(), BTreeSet::new()];
    for (distance, set) in [request.visual_radius_meters, request.physical_radius_meters]
        .into_iter()
        .zip(&mut sets)
    {
        // Any point within distance of r*viewer lies in this cone, regardless of relief.
        // If distance reaches the planet center, keep all directions conservatively.
        let angle = if distance >= r {
            std::f64::consts::PI
        } else {
            (distance / r).asin() + 128. * f64::EPSILON
        };
        for face in FACES {
            select(
                Tile {
                    face,
                    level: 0,
                    xy: [0, 0],
                },
                request.level,
                anchor.basis.y_axis,
                angle,
                set,
            )?;
        }
    }
    let [visual, physical] = sets;
    Ok(Plan {
        visual: visual.into_iter().collect(),
        physical: physical.into_iter().collect(),
        resources_installed: false,
    })
}
