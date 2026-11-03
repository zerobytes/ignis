//! Bounded placement on the first fixed box top below a search origin.
use crate::{
    model::{BodyKind, Result, invalid},
    physics::Simulation,
};
use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    pub max_drop: f32,
}
impl Projection {
    pub fn validate(self) -> Result<()> {
        if !self.max_drop.is_finite() || self.max_drop <= 0. || self.max_drop > 20000. {
            return Err(invalid(
                "Support max_drop must be finite, positive and at most20000",
            ));
        }
        Ok(())
    }
}
pub fn validate_position(position: [f32; 3]) -> Result<()> {
    if position.iter().any(|p| !p.is_finite() || p.abs() > 10000.) {
        return Err(invalid(
            "Support search position must be finite within +/-10000",
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Supported,
    NoSurface,
    InsufficientSupport,
    SupportLimit,
    Occupied,
    OutOfBounds,
    SlopedSurface,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub status: Status,
    pub surface: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh_surface: Option<u32>,
    pub feet: Option<[f32; 3]>,
    pub position: Option<[f32; 3]>,
}
impl Report {
    pub fn accepted(self) -> Option<[f32; 3]> {
        if self.status == Status::Supported {
            self.position
        } else {
            None
        }
    }
    pub(crate) fn rejection(self) -> crate::population::Outcome {
        if self.status == Status::Occupied {
            crate::population::Outcome::Occupied
        } else {
            crate::population::Outcome::UnsupportedPosition
        }
    }
}
impl Simulation {
    /// Internal callers have validated template/origin/projection at the command or scene boundary.
    /// This is geometry only: no slot, target, mobility or passage capacity reservation.
    pub(crate) fn query_spawn_support(
        &self,
        template: u8,
        origin: [f32; 3],
        projection: Projection,
    ) -> Report {
        let mut best: Option<(usize, f32)> = None;
        for (index, e) in self.definitions().entities.iter().enumerate() {
            if e.body != Some(BodyKind::Fixed) {
                continue;
            }
            let Some(top) = crate::geometry::Volume::entity(e, e.transform.position.into())
                .top(origin[0], origin[2])
            else {
                continue;
            };
            if top > origin[1] || origin[1] - top > projection.max_drop {
                continue;
            }
            if best.is_none_or(|(old, y)| {
                top > y || top == y && e.id < self.definitions().entities[old].id
            }) {
                best = Some((index, top));
            }
        }
        let prototype = &self.definitions().npc_templates[template as usize];
        let mesh = crate::surfaces::project(
            &self.physical_surfaces,
            prototype,
            origin.into(),
            projection.max_drop,
        );
        let Ok(mesh) = mesh else {
            return Report {
                status: Status::SupportLimit,
                surface: None,
                mesh_surface: None,
                feet: None,
                position: None,
            };
        };
        if let Some((id, top)) = mesh.filter(|(_, y)| best.is_none_or(|(_, old)| *y > old)) {
            let offset = prototype.character_controller.map_or(0., |c| c.offset);
            let feet = Vec3::new(origin[0], top + offset, origin[2]);
            let position = feet + Vec3::Y * (prototype.transform.size[1] * 0.5);
            let status = if position
                .to_array()
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 10000.)
            {
                Status::OutOfBounds
            } else {
                match crate::surfaces::footprint(
                    self.definitions(),
                    &self.physical_surfaces,
                    prototype,
                    feet,
                ) {
                    crate::surfaces::Support::Limit => Status::SupportLimit,
                    crate::surfaces::Support::Gap => Status::InsufficientSupport,
                    crate::surfaces::Support::Supported => {
                        if self.spawn_position_occupied(template, position) {
                            Status::Occupied
                        } else {
                            Status::Supported
                        }
                    }
                }
            };
            return Report {
                status,
                surface: None,
                mesh_surface: Some(
                    self.definitions()
                        .collision_meshes
                        .iter()
                        .position(|s| s == &id)
                        .unwrap() as u32,
                ),
                feet: Some(feet.to_array()),
                position: Some(position.to_array()),
            };
        }
        let Some((index, top)) = best else {
            return Report {
                status: Status::NoSurface,
                surface: None,
                mesh_surface: None,
                feet: None,
                position: None,
            };
        };
        let prototype = &self.definitions().npc_templates[template as usize];
        let feet = Vec3::new(origin[0], top, origin[2]);
        let position = feet + Vec3::Y * (prototype.transform.size[1] * 0.5);
        let status = if !self.definitions().entities[index].geometry.is_box() {
            Status::SlopedSurface
        } else if position
            .to_array()
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 10000.)
        {
            Status::OutOfBounds
        } else {
            match crate::walk_support::query(self.definitions(), prototype, feet, feet) {
                crate::walk_support::Support::ComplexityLimit => Status::SupportLimit,
                crate::walk_support::Support::Continuous => {
                    if self.spawn_position_occupied(template, position) {
                        Status::Occupied
                    } else {
                        Status::Supported
                    }
                }
                _ => Status::InsufficientSupport,
            }
        };
        Report {
            status,
            surface: Some(index as u16),
            mesh_surface: None,
            feet: Some(feet.to_array()),
            position: Some(position.to_array()),
        }
    }
}
