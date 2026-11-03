//! Canonical global addresses. Physics still owns a separate, local coordinate space.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VERSION: u16 = 1;
/// Addressing unit, independent of the size of streamed regions.
pub const CELL_METERS: f64 = 1024.;
pub const MAX_PROJECTION_RADIUS: f64 = 1_048_576.;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Coordinates {
    /// JSON uses decimal strings so JavaScript/MCP never rounds an i64 cell.
    #[serde(with = "cell_codec")]
    #[schemars(with = "[String; 3]")]
    pub cell: [i64; 3],
    pub offset: [f64; 3],
}

/// Deserialization accepts canonical positions only; explicit normalization is separate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "Coordinates", into = "Coordinates")]
#[schemars(with = "Coordinates")]
pub struct Position {
    cell: [i64; 3],
    offset: [f64; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpaceError(pub &'static str);
impl std::fmt::Display for SpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for SpaceError {}
type Result<T> = std::result::Result<T, SpaceError>;

// i128 intermediates allow subtracting opposite i64 cells without overflow.
// No float conversion of a global address is ever necessary.
fn split(offset: f64) -> Result<(i128, f64)> {
    if !offset.is_finite() {
        return Err(SpaceError("World offset/displacement must be finite"));
    }
    let quotient = (offset / CELL_METERS).floor();
    if quotient.abs() > (1_u128 << 64) as f64 {
        return Err(SpaceError(
            "World displacement exceeds the coordinate domain",
        ));
    }
    let mut carry = quotient as i128;
    let mut local = offset - quotient * CELL_METERS;
    // Division of tiny negative offsets can underflow to -0. Handle that case
    // before the rounded upper-bound case, so no spurious whole cell is added.
    if local < 0. {
        carry -= 1;
        local += CELL_METERS;
    }
    if local >= CELL_METERS {
        carry += 1;
        local -= CELL_METERS;
    }
    if local == 0. {
        local = 0.; // canonical positive zero
    }
    Ok((carry, local))
}

impl Position {
    pub fn normalize(raw: Coordinates) -> Result<Self> {
        let mut out = Self {
            cell: raw.cell,
            offset: [0.; 3],
        };
        for axis in 0..3 {
            let (carry, offset) = split(raw.offset[axis])?;
            out.cell[axis] = i64::try_from(i128::from(raw.cell[axis]) + carry)
                .map_err(|_| SpaceError("World cell overflow"))?;
            out.offset[axis] = offset;
        }
        Ok(out)
    }

    pub fn translated(self, delta: [f64; 3]) -> Result<Self> {
        let mut out = self;
        for (axis, component) in delta.into_iter().enumerate() {
            // Split the displacement FIRST. Adding a large delta to the local
            // fraction before splitting would destroy that fraction's precision.
            let (cells, fraction) = split(component)?;
            let (carry, local) = split(self.offset[axis] + fraction)?;
            out.cell[axis] = i64::try_from(i128::from(self.cell[axis]) + cells + carry)
                .map_err(|_| SpaceError("World cell overflow"))?;
            out.offset[axis] = local;
        }
        Ok(out)
    }

    pub fn project(self, origin: Self, radius: f64, max_error: f64) -> Result<[f32; 3]> {
        if !radius.is_finite()
            || radius <= 0.
            || radius > MAX_PROJECTION_RADIUS
            || !max_error.is_finite()
            || !(0. ..=1.).contains(&max_error)
        {
            return Err(SpaceError(
                "Projection requires radius (0,1048576]m and error [0,1]m",
            ));
        }
        let limit = (radius / CELL_METERS).ceil() as i128 + 1;
        let mut relative = [0.; 3];
        for (axis, value) in relative.iter_mut().enumerate() {
            let cells = i128::from(self.cell[axis]) - i128::from(origin.cell[axis]);
            if cells.abs() > limit {
                return Err(SpaceError("Position is outside this local projection"));
            }
            *value = cells as f64 * CELL_METERS + (self.offset[axis] - origin.offset[axis]);
        }
        if relative.iter().map(|v| v * v).sum::<f64>() > radius * radius {
            return Err(SpaceError("Position is outside this local projection"));
        }
        let projected = relative.map(|v| v as f32);
        if relative
            .iter()
            .zip(projected)
            .any(|(v, p)| (v - f64::from(p)).abs() > max_error)
        {
            return Err(SpaceError(
                "Local f32 projection exceeds the requested precision",
            ));
        }
        Ok(projected)
    }
}

impl TryFrom<Coordinates> for Position {
    type Error = SpaceError;
    fn try_from(raw: Coordinates) -> Result<Self> {
        if raw.offset.iter().any(|v| {
            !v.is_finite() || !(0. ..CELL_METERS).contains(v) || (*v == 0. && v.is_sign_negative())
        }) {
            return Err(SpaceError(
                "Canonical world offset must be finite in [0,1024), with positive zero",
            ));
        }
        Ok(Self {
            cell: raw.cell,
            offset: raw.offset,
        })
    }
}
impl From<Position> for Coordinates {
    fn from(value: Position) -> Self {
        Self {
            cell: value.cell,
            offset: value.offset,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    Normalize {
        point: Coordinates,
    },
    Translate {
        point: Position,
        delta: [f64; 3],
    },
    Project {
        point: Position,
        origin: Position,
        radius_meters: f64,
        max_error_meters: f64,
    },
}
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Answer {
    Position {
        version: u16,
        cell_meters: f64,
        point: Position,
    },
    Projection {
        version: u16,
        local: [f32; 3],
        origin: Position,
    },
}
pub fn query(query: Query) -> Result<Answer> {
    let point = match query {
        Query::Normalize { point } => Position::normalize(point)?,
        Query::Translate { point, delta } => point.translated(delta)?,
        Query::Project {
            point,
            origin,
            radius_meters,
            max_error_meters,
        } => {
            return Ok(Answer::Projection {
                version: VERSION,
                local: point.project(origin, radius_meters, max_error_meters)?,
                origin,
            });
        }
    };
    Ok(Answer::Position {
        version: VERSION,
        cell_meters: CELL_METERS,
        point,
    })
}

pub(crate) mod cell_codec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(
        cells: &[i64; 3],
        s: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            cells.map(|v| v.to_string()).serialize(s)
        } else {
            cells.serialize(s)
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<[i64; 3], D::Error> {
        if !d.is_human_readable() {
            return <[i64; 3]>::deserialize(d);
        }
        let strings = <[String; 3]>::deserialize(d)?;
        let mut cells = [0; 3];
        for (i, text) in strings.into_iter().enumerate() {
            // Avoid accepting aliases (+1, 001, -0) into a canonical save/address.
            let value: i64 = text.parse().map_err(serde::de::Error::custom)?;
            if value.to_string() != text {
                return Err(serde::de::Error::custom(
                    "World cell must use canonical signed decimal",
                ));
            }
            cells[i] = value;
        }
        Ok(cells)
    }
}
