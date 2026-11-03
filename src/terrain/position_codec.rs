//! Serde's tagged-enum buffering loses is_human_readable on nested maps.
//! JSON keeps strict Position maps; binary uses48LE bytes without this ambiguity.
use super::*;
pub fn serialize<S: serde::Serializer>(p: &Position, s: S) -> std::result::Result<S::Ok, S::Error> {
    if s.is_human_readable() {
        return p.serialize(s);
    }
    let p: Coordinates = (*p).into();
    let mut bytes = [0u8; 48];
    for i in 0..3 {
        bytes[i * 8..i * 8 + 8].copy_from_slice(&p.cell[i].to_le_bytes());
        bytes[24 + i * 8..32 + i * 8].copy_from_slice(&p.offset[i].to_le_bytes());
    }
    s.serialize_bytes(&bytes)
}
pub fn deserialize<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Position, D::Error> {
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Position;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a canonical world position or48binary bytes")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            a: A,
        ) -> std::result::Result<Position, A::Error> {
            Position::deserialize(serde::de::value::MapAccessDeserializer::new(a))
        }
        fn visit_bytes<E: serde::de::Error>(self, b: &[u8]) -> std::result::Result<Position, E> {
            if b.len() != 48 {
                return Err(E::custom("Terrain position requires48bytes"));
            }
            let cell = std::array::from_fn(|i| {
                i64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap())
            });
            let offset = std::array::from_fn(|i| {
                f64::from_le_bytes(b[24 + i * 8..32 + i * 8].try_into().unwrap())
            });
            Position::try_from(Coordinates { cell, offset }).map_err(E::custom)
        }
        fn visit_byte_buf<E: serde::de::Error>(
            self,
            b: Vec<u8>,
        ) -> std::result::Result<Position, E> {
            self.visit_bytes(&b)
        }
    }
    d.deserialize_any(Visitor)
}
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct EncodedPosition(#[serde(with = "self")] Position);
pub fn serialize_points<S: serde::Serializer>(
    points: &[Position],
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = s.serialize_seq(Some(points.len()))?;
    for &point in points {
        seq.serialize_element(&EncodedPosition(point))?;
    }
    seq.end()
}
pub fn deserialize_points<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Position>, D::Error> {
    Ok(limited::<D, EncodedPosition, 64>(d)?
        .into_iter()
        .map(|p| p.0)
        .collect())
}
