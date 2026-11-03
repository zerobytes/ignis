//! Aligned convex primitives. Wedges share one solid across physical and semantic queries.
use crate::model::{BodyKind, Entity, Result, invalid};
use glam::{Vec2, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Rise {
    PositiveX,
    NegativeX,
    PositiveZ,
    NegativeZ,
}
impl Rise {
    pub fn axis_sign(self) -> (usize, f32) {
        match self {
            Self::PositiveX => (0, 1.),
            Self::NegativeX => (0, -1.),
            Self::PositiveZ => (2, 1.),
            Self::NegativeZ => (2, -1.),
        }
    }
}
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Geometry {
    #[default]
    Box,
    Wedge {
        rise: Rise,
    },
}
impl Geometry {
    pub fn is_box(&self) -> bool {
        *self == Self::Box
    }
    pub fn validate(self, e: &Entity, version: u32) -> Result<()> {
        if !self.is_box()
            && (version < 75
                || !matches!(e.body, None | Some(BodyKind::Fixed))
                || e.trigger.is_some())
        {
            return Err(invalid(
                "Wedge requires scene75 and fixed/visual entity without trigger; actors remain boxes",
            ));
        }
        Ok(())
    }
    pub fn vertices(self, size: [f32; 3]) -> [Vec3; 6] {
        let Self::Wedge { rise } = self else {
            unreachable!("Wedge vertices only");
        };
        let (axis, sign) = rise.axis_sign();
        let other = 2 - axis;
        let h = Vec3::from(size) * 0.5;
        [
            (-1., -1., -1.),
            (1., -1., -1.),
            (1., 1., -1.),
            (-1., -1., 1.),
            (1., -1., 1.),
            (1., 1., 1.),
        ]
        .map(|(u, y, v)| {
            let mut p = Vec3::ZERO;
            p[axis] = u * h[axis] * sign;
            p.y = y * h.y;
            p[other] = v * h[other];
            p
        })
    }
}
pub const TRIANGLES: [[usize; 3]; 8] = [
    [0, 2, 1],
    [3, 4, 5],
    [0, 1, 4],
    [0, 4, 3],
    [1, 2, 5],
    [1, 5, 4],
    [0, 3, 5],
    [0, 5, 2],
];
pub const EDGES: [[usize; 2]; 9] = [
    [0, 1],
    [1, 2],
    [2, 0],
    [3, 4],
    [4, 5],
    [5, 3],
    [0, 3],
    [1, 4],
    [2, 5],
];

#[derive(Clone, Copy)]
pub struct Volume {
    pub geometry: Geometry,
    pub center: Vec3,
    pub size: Vec3,
}
impl Volume {
    pub fn entity(e: &Entity, center: Vec3) -> Self {
        Self {
            geometry: e.geometry,
            center,
            size: e.transform.size.into(),
        }
    }
    /// Closed (contact) or open (positive interior) intersection with a swept axis-aligned box.
    /// Callers preserve their established parallel threshold and contact policy.
    pub fn cast(
        self,
        start: Vec3,
        delta: Vec3,
        padding: Vec3,
        interior: bool,
        epsilon: f32,
    ) -> Option<f32> {
        let h = self.size * 0.5 + padding;
        let mut enter = 0_f32;
        let mut leave = 1_f32;
        for axis in 0..3 {
            let min = self.center[axis] - h[axis];
            let max = self.center[axis] + h[axis];
            if delta[axis] == 0. || delta[axis].abs() < epsilon {
                if if interior {
                    start[axis] <= min || start[axis] >= max
                } else {
                    start[axis] < min || start[axis] > max
                } {
                    return None;
                }
            } else {
                let a = (min - start[axis]) / delta[axis];
                let b = (max - start[axis]) / delta[axis];
                enter = enter.max(a.min(b));
                leave = leave.min(a.max(b));
                if if interior {
                    enter >= leave
                } else {
                    enter > leave
                } {
                    return None;
                }
            }
        }
        if let Geometry::Wedge { rise } = self.geometry {
            let (axis, sign) = rise.axis_sign();
            let k = self.size.y / self.size[axis];
            let mut n = Vec3::Y;
            n[axis] = -sign * k;
            let bound = padding.y + k * padding[axis];
            let distance = n.dot(start - self.center) - bound;
            let velocity = n.dot(delta);
            if velocity == 0. || velocity.abs() < epsilon {
                if if interior {
                    distance >= 0.
                } else {
                    distance > 0.
                } {
                    return None;
                }
            } else {
                let t = -distance / velocity;
                if velocity < 0. {
                    enter = enter.max(t);
                } else {
                    leave = leave.min(t);
                }
                if if interior {
                    enter >= leave
                } else {
                    enter > leave
                } {
                    return None;
                }
            }
        }
        Some(enter)
    }
    pub fn top(self, x: f32, z: f32) -> Option<f32> {
        let h = self.size * 0.5;
        if (x - self.center.x).abs() > h.x || (z - self.center.z).abs() > h.z {
            return None;
        }
        Some(match self.geometry {
            Geometry::Box => self.center.y + h.y,
            Geometry::Wedge { rise } => {
                let (axis, sign) = rise.axis_sign();
                self.center.y
                    + sign * self.size.y / self.size[axis] * ([x, 0., z][axis] - self.center[axis])
            }
        })
    }
    /// XZ projection of the solid intersected with an open vertical slab (planar actor body).
    pub fn horizontal_bounds(self, min_y: f32, max_y: f32) -> Option<(Vec2, Vec2)> {
        let h = self.size * 0.5;
        let mut min = self.center - h;
        let mut max = self.center + h;
        if max.y <= min_y || min.y >= max_y {
            return None;
        }
        if let Geometry::Wedge { rise } = self.geometry {
            let (axis, sign) = rise.axis_sign();
            let cut =
                self.center[axis] + sign * (min_y - self.center.y) * self.size[axis] / self.size.y;
            if sign > 0. {
                min[axis] = min[axis].max(cut);
            } else {
                max[axis] = max[axis].min(cut);
            }
        }
        Some((Vec2::new(min.x, min.z), Vec2::new(max.x, max.z)))
    }
}
