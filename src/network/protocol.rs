use crate::{
    abilities::{Aim, Outcome},
    physics::Input,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use subtle::ConstantTimeEq;
pub const VERSION: u16 = 5;
pub const OPERATIONS: usize = 16;
pub const RESULTS: usize = 64;
pub const INPUT_TTL_MS: u64 = 250;
pub const RESUME_MS: u64 = 10_000;
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret([u8; 32]);
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}
impl Secret {
    pub(crate) fn new() -> crate::model::Result<Self> {
        let mut bytes = [0; 32];
        getrandom::fill(&mut bytes)
            .map_err(|_| crate::model::invalid("Session entropy unavailable"))?;
        Ok(Self(bytes))
    }
    pub(crate) fn matches(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub actor: u64,
    pub life: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Movement {
    pub sequence: u64,
    pub life: u64,
    pub input: Input,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Intent {
    Activate { slot: u8, target: Option<Target> },
    Aim { slot: u8, aim: Aim },
    Cancel { execution: u64 },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub id: u64,
    pub life: u64,
    pub intent: Intent,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Status {
    Received,
    Queued,
    Started {
        execution: Option<u64>,
    },
    Resolved {
        outcome: Outcome,
        execution: Option<u64>,
    },
    Rejected {
        reason: Rejection,
    },
}
impl Status {
    pub fn terminal(&self) -> bool {
        matches!(self, Self::Resolved { .. } | Self::Rejected { .. })
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    UnknownSession,
    OldConnection,
    StaleLife,
    Sequence,
    Capacity,
    ExpiredResult,
    Target,
    InvalidOperation,
    Unavailable,
    SlowConsumer,
    WorldChanged,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationResult {
    pub id: u64,
    pub tick: u64,
    pub status: Status,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    pub slot: u8,
    pub id: String,
    pub selection: crate::abilities::Selection,
}
fn alive_default() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Welcome {
    pub version: u16,
    pub world: u64,
    pub session: u64,
    pub connection: u64,
    pub actor: Target,
    pub secret: Secret,
    pub last_operation: u64,
    pub results: Vec<OperationResult>,
    #[serde(default)]
    pub grants: Vec<Grant>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerView {
    pub actor: Target,
    pub tick: u64,
    pub input_ack: u64,
    pub position: [f32; 3],
    pub resources: Vec<(u8, i64, i64)>,
    pub ready_at: [u64; 16],
    #[serde(default = "alive_default")]
    pub alive: bool,
    #[serde(default)]
    pub cast_execution: Option<u64>,
}
