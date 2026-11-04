//! Insert shared edges on authored crease planes, without removing either side.
use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreasePlane {
    pub point: [f64; 3],
    pub normal: [f64; 3],
}
pub(super) fn planes<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<CreasePlane>, D::Error> {
    crate::terrain::limited::<D, CreasePlane, 24>(d)
}
impl CreasePlane {
    pub(super) fn validate(&self) -> Result<()> {
        if !finite(&self.point, 10000.)
            || !finite(&self.normal, 10000.)
            || DVec3::from_array(self.normal).try_normalize().is_none()
        {
            return Err(invalid(
                "Crease plane needs finite local coordinates and a nonzero normal",
            ));
        }
        Ok(())
    }
}
pub(super) fn apply(g: &mut Geometry, planes: &[CreasePlane]) -> Result<f64> {
    let mut work = 0usize;
    let mut rounding = 0f64;
    for plane in planes {
        let point = DVec3::from_array(plane.point);
        let normal = DVec3::from_array(plane.normal).normalize();
        work = work.saturating_add(g.positions.len() + g.triangles.len() * 6);
        if work > MAX_WORK {
            return Err(invalid("Crease tessellation exceeds work budget"));
        }
        let distances: Vec<_> = g
            .positions
            .iter()
            .map(|p| (DVec3::from_array(p.map(f64::from)) - point).dot(normal))
            .collect();
        let extent = g
            .positions
            .iter()
            .flatten()
            .map(|v| v.abs())
            .fold(1f32, f32::max) as f64;
        // Stay above local f32 rounding; points already on a cut retain their IDs.
        let epsilon = extent * f64::from(f32::EPSILON) * 4.;
        let mut intersections = BTreeMap::<[u32; 2], u32>::new();
        let source = std::mem::take(&mut g.triangles);
        let mut output = Vec::with_capacity((source.len() * 2).min(meshes::MAX_TRIANGLES));
        for face in source {
            let d = face.map(|i| distances[i as usize]);
            if !d.iter().any(|&v| v > epsilon) || !d.iter().any(|&v| v < -epsilon) {
                if output.len() == meshes::MAX_TRIANGLES {
                    return Err(invalid("Crease triangle budget exceeded"));
                }
                output.push(face);
                continue;
            }
            for sign in [1., -1.] {
                let mut polygon = [0u32; 4];
                let mut length = 0;
                for j in 0..3 {
                    let a = face[j];
                    let b = face[(j + 1) % 3];
                    let da = distances[a as usize];
                    let db = distances[b as usize];
                    if da * sign >= -epsilon {
                        polygon[length] = a;
                        length += 1;
                    }
                    if (da > epsilon && db < -epsilon) || (da < -epsilon && db > epsilon) {
                        let key = [a.min(b), a.max(b)];
                        let id = if let Some(&id) = intersections.get(&key) {
                            id
                        } else {
                            if g.positions.len() == meshes::MAX_VERTICES {
                                return Err(invalid("Crease vertex budget exceeded"));
                            }
                            let p = DVec3::from_array(g.positions[key[0] as usize].map(f64::from));
                            let q = DVec3::from_array(g.positions[key[1] as usize].map(f64::from));
                            let t = distances[key[0] as usize]
                                / (distances[key[0] as usize] - distances[key[1] as usize]);
                            let id = g.positions.len() as u32;
                            let value = p.lerp(q, t);
                            let rounded = value.as_vec3();
                            rounding = rounding.max(value.distance(rounded.as_dvec3()));
                            g.positions.push(rounded.to_array());
                            intersections.insert(key, id);
                            id
                        };
                        polygon[length] = id;
                        length += 1;
                    }
                }
                for j in 1..length - 1 {
                    if output.len() == meshes::MAX_TRIANGLES {
                        return Err(invalid("Crease triangle budget exceeded"));
                    }
                    output.push([polygon[0], polygon[j], polygon[j + 1]]);
                }
            }
        }
        g.triangles = output;
    }
    g.validate()?;
    Ok(rounding)
}
