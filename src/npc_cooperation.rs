//! Explicit orders are receipts addressed to an individual life, not shared perception.
use crate::{
    identity::Reference,
    model::{Result, invalid},
    npc_mind::Profile,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub group: u16,
    pub role: u16,
    pub formation_offset: [f32; 3],
    pub reserve_support: bool,
    pub support_lease_ticks: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            group: 1,
            role: 0,
            formation_offset: [0.; 3],
            reserve_support: false,
            support_lease_ticks: 120,
        }
    }
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if self.group == 0
            || !(1..=3600).contains(&self.support_lease_ticks)
            || self
                .formation_offset
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 32.)
        {
            return Err(invalid(
                "Cooperation requires group>0 and formation offsets within +/-32m",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Formation {
    Offsets,
    Line,
    Wedge,
    Circle,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Order {
    Move {
        position: [f32; 3],
        arrival_distance: f32,
        for_ticks: u32,
        formation: Formation,
        spacing: f32,
        facing_degrees: f32,
    },
    Hold {
        for_ticks: u32,
    },
    Cancel,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Intent {
    Move {
        position: [f32; 3],
        arrival_distance: f32,
        for_ticks: u32,
    },
    Hold {
        for_ticks: u32,
    },
    Cancel,
}
impl Intent {
    pub fn validate(self) -> Result<()> {
        let duration = match self {
            Self::Move {
                position,
                arrival_distance,
                for_ticks,
            } => {
                if position.iter().any(|v| !v.is_finite() || v.abs() > 10000.)
                    || !arrival_distance.is_finite()
                    || !(0.1..=2.).contains(&arrival_distance)
                {
                    return Err(invalid(
                        "Order requires finite XYZ point and arrival [0.1,2]m",
                    ));
                }
                for_ticks
            }
            Self::Hold { for_ticks } => for_ticks,
            Self::Cancel => return Ok(()),
        };
        if !(1..=36000).contains(&duration) {
            return Err(invalid("Order duration must be [1,36000]ticks"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Frozen {
    pub intent: Intent,
    pub ordinal: u16,
    pub members: u16,
    pub role: u16,
}
impl Frozen {
    pub fn validate(self) -> Result<()> {
        if self.members == 0
            || self.members as usize > crate::targeting::MAX_ACTORS
            || self.ordinal >= self.members
        {
            return Err(invalid("Invalid frozen formation slot"));
        }
        self.intent.validate()
    }
}
impl Order {
    pub fn freeze(self, config: Config, ordinal: usize, members: usize) -> Result<Frozen> {
        if members == 0 || members > crate::targeting::MAX_ACTORS || ordinal >= members {
            return Err(invalid("Invalid formation membership"));
        }
        let intent = match self {
            Self::Cancel => Intent::Cancel,
            Self::Hold { for_ticks } => Intent::Hold { for_ticks },
            Self::Move {
                position,
                arrival_distance,
                for_ticks,
                formation,
                spacing,
                facing_degrees,
            } => {
                if !spacing.is_finite()
                    || !(0.1..=8.).contains(&spacing)
                    || !facing_degrees.is_finite()
                    || facing_degrees.abs() > 360.
                {
                    return Err(invalid(
                        "Formation needs spacing [0.1,8]m and facing within +/-360deg",
                    ));
                }
                let mut offset = glam::Vec3::from(config.formation_offset);
                match formation {
                    Formation::Offsets => {}
                    Formation::Line => {
                        offset.x += (ordinal as f32 - (members - 1) as f32 * 0.5) * spacing
                    }
                    Formation::Wedge => {
                        let row = ordinal.div_ceil(2) as f32;
                        offset.x += if ordinal % 2 == 1 {
                            -row * spacing
                        } else {
                            row * spacing
                        };
                        offset.z -= row * spacing;
                    }
                    Formation::Circle => {
                        let angle = ordinal as f32 * std::f32::consts::TAU / members as f32;
                        offset += glam::Vec3::new(angle.cos(), 0., angle.sin()) * spacing;
                    }
                }
                let point = glam::Vec3::from(position)
                    + glam::Quat::from_rotation_y(facing_degrees.to_radians()) * offset;
                Intent::Move {
                    position: point.into(),
                    arrival_distance,
                    for_ticks,
                }
            }
        };
        intent.validate()?;
        Ok(Frozen {
            intent,
            ordinal: ordinal as u16,
            members: members as u16,
            role: config.role,
        })
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Reacting,
    Moving,
    Arrived,
    Holding,
    Expired,
    Cancelled,
    Suspended,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportLease {
    pub group: u16,
    pub actor: u16,
    pub source: Reference,
    pub source_life: u64,
    pub target: u16,
    pub target_instance: Reference,
    pub target_life: u64,
    pub execution: u64,
    pub expires_at: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportDecision {
    pub group: u16,
    pub source: Reference,
    pub target: Reference,
    pub target_life: u64,
    pub reserved_by: Reference,
    pub allowed: bool,
    pub score: i32,
}
#[derive(Clone, Copy)]
pub(crate) struct SupportCandidate {
    pub group: u16,
    pub source: u16,
    pub source_instance: Reference,
    pub target: u16,
    pub target_instance: Reference,
    pub target_life: u64,
    pub score: i32,
    pub existing: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub request: u64,
    pub instance: Reference,
    pub life: u64,
    pub order: Frozen,
    pub issued_tick: u64,
    pub starts_at: u64,
    pub expires_at: u64,
    pub phase: Phase,
}
impl State {
    pub fn new(
        request: u64,
        instance: Reference,
        life: u64,
        order: Frozen,
        tick: u64,
        profile: Option<Profile>,
    ) -> Self {
        let duration = match order.intent {
            Intent::Move { for_ticks, .. } | Intent::Hold { for_ticks } => u64::from(for_ticks),
            Intent::Cancel => 0,
        };
        let delay = profile.map_or(0, |p| {
            (p.think_interval() - 1) * u64::from(10_000 - p.traits[8]) / 10_000
        });
        Self {
            request,
            instance,
            life,
            order,
            issued_tick: tick,
            starts_at: tick.saturating_add(delay),
            expires_at: tick.saturating_add(duration),
            phase: if matches!(order.intent, Intent::Cancel) {
                Phase::Cancelled
            } else {
                Phase::Reacting
            },
        }
    }
}
