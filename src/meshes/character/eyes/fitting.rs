//! Versioned socket contact: legacy source8 skirt, source9 spherical globe.
use super::*;
mod spherical;
pub use spherical::SphericalFit;

pub const FIT_CONTROLS: &[(&str, &str, f32, f32, f32)] = &[
    (
        "globe_depth",
        "Profundidade do globo ocular",
        -0.008,
        0.008,
        0.,
    ),
    ("socket_fit", "Encaixe das pálpebras no globo", 0., 1., 1.),
];

/// Indices address the drawn part after its source-owned refinement. Rim is a
/// closed edge loop; lid is a finite, authored blend region, never the whole face.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EyeFit {
    pub part: String,
    pub rim: Vec<u32>,
    pub lid: Vec<WarpWeight>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spherical: Option<SphericalFit>,
}

impl Source {
    pub(super) fn validate_eye_fits(&self, params: &BTreeMap<&String, &Parameter>) -> Result<()> {
        let mut owned = BTreeSet::new();
        for eye in &self.eyes {
            let Some(fit) = &eye.fit else { continue };
            let body = self
                .parts
                .iter()
                .find(|p| p.name == fit.part)
                .ok_or_else(|| invalid("Unknown eyelid contact part"))?;
            let socket = self
                .parts
                .iter()
                .find(|p| p.name == eye.sclera)
                .and_then(|p| p.socket)
                .ok_or_else(|| invalid("Missing fitted eye socket"))?;
            if self.version < 8
                || body.socket.is_some()
                || self.joints.iter().any(|j| j.parent == Some(socket))
                || !(8..=128).contains(&fit.rim.len())
                || !(8..=4096).contains(&fit.lid.len())
            {
                return Err(invalid(
                    "Socket contact requires source8, a deformable lid and leaf eye joint",
                ));
            }
            let count = self.refined_vertex_count(body)?;
            let mut previous = None;
            let mut region = BTreeMap::new();
            for w in &fit.lid {
                if w.vertex as usize >= count
                    || previous.is_some_and(|i| i >= w.vertex)
                    || !w.weight.is_finite()
                    || !(0. ..=1.).contains(&w.weight)
                    || !owned.insert((&fit.part, w.vertex))
                {
                    return Err(invalid(
                        "Lid regions require disjoint, ordered, bounded drawn vertices",
                    ));
                }
                region.insert(w.vertex, w.weight);
                previous = Some(w.vertex);
            }
            let mut rim = BTreeSet::new();
            for &v in &fit.rim {
                if !rim.insert(v) || region.get(&v) != Some(&1.) {
                    return Err(invalid(
                        "Contact rim requires unique vertices with full lid weight",
                    ));
                }
            }
            self.validate_contact_rim(body, &fit.rim)?;
            if let Some(spherical) = &fit.spherical
                && (self.version < 9
                    || spherical.interior.len() > 4096
                    || spherical.interior.windows(2).any(|w| w[0] >= w[1])
                    || spherical
                        .interior
                        .iter()
                        .any(|i| rim.contains(i) || region.get(i) != Some(&1.)))
            {
                return Err(invalid(
                    "Spherical socket interior requires source9 and distinct full-weight non-rim lid vertices",
                ));
            }
            for &(name, _, min, max, default) in FIT_CONTROLS {
                let key = format!("{}_{}", eye.prefix, name);
                let p = params
                    .get(&key)
                    .ok_or_else(|| invalid("Missing eye fitting control"))?;
                if p.minimum < min || p.maximum > max || p.default != default {
                    return Err(invalid("Invalid eye fitting control bounds/default"));
                }
            }
        }
        Ok(())
    }

    pub(in crate::meshes::character) fn fit_eyes(
        &self,
        values: &BTreeMap<String, f32>,
        placement: &Placement,
        sockets: &BTreeMap<u8, Mat4>,
        document: &mut Document,
    ) -> Result<()> {
        for eye in &self.eyes {
            let Some(fit) = &eye.fit else { continue };
            let amount = values[&format!("{}_socket_fit", eye.prefix)];
            let depth = values[&format!("{}_globe_depth", eye.prefix)];
            if amount == 0. && depth == 0. {
                continue;
            }
            let eye_part = self.parts.iter().find(|p| p.name == eye.sclera).unwrap();
            let joint = eye_part.socket.unwrap();
            let socket = sockets[&joint];
            let inverse = socket.inverse();
            let id = format!("{}_{}", placement.prefix, fit.part);
            let body = document.assets.iter_mut().find(|a| a.id == id).unwrap();
            let g = Arc::make_mut(&mut body.geometry);
            let rim: Vec<_> = fit
                .rim
                .iter()
                .map(|&i| inverse.transform_point3(g.positions[i as usize].into()))
                .collect();
            let low = rim
                .iter()
                .fold(Vec3::splat(f32::INFINITY), |a, &b| a.min(b));
            let high = rim
                .iter()
                .fold(Vec3::splat(f32::NEG_INFINITY), |a, &b| a.max(b));
            let base = eye.center.at(&self.positions);
            let (final_center, fitted_radius) = if let Some(spherical) = &fit.spherical {
                let samples: Vec<_> = fit
                    .lid
                    .iter()
                    .map(|w| {
                        (
                            w.vertex,
                            w.weight,
                            inverse.transform_point3(g.positions[w.vertex as usize].into()),
                        )
                    })
                    .collect();
                let result =
                    spherical.wrap(&fit.rim, &rim, &samples, (base, eye.radius), amount, depth)?;
                for (i, p) in result.positions {
                    g.positions[i as usize] = socket.transform_point3(p).to_array();
                }
                (result.center, result.radius)
            } else {
                let mut center = (low + high) * 0.5;
                let radius = rim
                    .iter()
                    .map(|p| (p.truncate() - center.truncate()).length())
                    .fold(eye.radius, f32::max)
                    * 1.12;
                if !radius.is_finite() || radius > 0.04 {
                    return Err(invalid(
                        "Eye aperture exceeds bounded socket fit; reduce the orbital deformation",
                    ));
                }
                center.z = base.z;
                let fitted_center = base.lerp(center, amount);
                // Keep the authored skin exact at rest. A depth edit transports only
                // its finite lid band; fit closes the sclera behind the existing rim.
                if depth != 0. {
                    for w in &fit.lid {
                        let p = inverse.transform_point3(g.positions[w.vertex as usize].into());
                        g.positions[w.vertex as usize] = socket
                            .transform_point3(p + Vec3::Z * w.weight * depth)
                            .to_array();
                    }
                }
                (fitted_center + Vec3::Z * depth, eye.radius)
            };
            for name in [&eye.sclera, &eye.iris, &eye.cornea] {
                let part = self.parts.iter().find(|p| &p.name == name).unwrap();
                let id = format!("{}_{}", placement.prefix, name);
                let asset = document.assets.iter_mut().find(|a| a.id == id).unwrap();
                self.apply_eye_shape(
                    part,
                    values,
                    Some(socket),
                    asset,
                    Some((final_center, fitted_radius)),
                )?;
                if fit.spherical.is_none() && name == &eye.sclera && amount > 0. {
                    let iris_radius = values[&format!("{}_radius", eye.prefix)];
                    let shape = contact_sclera(
                        &rim,
                        final_center - Vec3::Z * depth,
                        eye.radius,
                        iris_radius,
                    )?;
                    let g = Arc::make_mut(&mut asset.geometry);
                    if g.positions.len() != shape.len() {
                        return Err(invalid(
                            "Scleral contact topology differs from ocular generator",
                        ));
                    }
                    for (old, p) in g.positions.iter_mut().zip(shape) {
                        let target = socket.transform_point3(p + Vec3::Z * depth);
                        *old = Vec3::from(*old).lerp(target, amount).to_array();
                    }
                }
            }
            // The eye's rotation pivot follows its new centre in every shared rig.
            let delta = socket.transform_vector3(final_center - base);
            for asset in &mut document.assets {
                let rig = Arc::make_mut(asset.rig.as_mut().unwrap());
                let globals = rig.bind_globals();
                let local_delta = rig.bones[joint as usize].parent.map_or(delta, |p| {
                    globals[p as usize].inverse().transform_vector3(delta)
                });
                let bone = &mut rig.bones[joint as usize];
                bone.translation = (Vec3::from(bone.translation) + local_delta).to_array();
            }
        }
        Ok(())
    }
}

/// A closed scleral surface with a source-owned contact skirt. Its front joins
/// the spherical iris limbus and finishes behind the lid rim; the rear stays
/// hidden in the socket. No skin projection or whole-face smoothing is needed.
fn contact_sclera(rim: &[Vec3], center: Vec3, radius: f32, iris: f32) -> Result<Vec<Vec3>> {
    let cross = |a: glam::Vec2, b: glam::Vec2| a.x * b.y - a.y * b.x;
    let z = -(radius * radius - iris * iris).sqrt();
    let rear = rim.iter().map(|p| p.z).fold(center.z + radius, f32::max) + 0.003;
    let mut border = Vec::with_capacity(80);
    for i in 0..80 {
        let angle = TAU * i as f32 / 80.;
        let direction = glam::Vec2::new(angle.cos(), angle.sin());
        let mut hit = None;
        for (a, b) in rim.iter().zip(rim.iter().cycle().skip(1)) {
            let a = *a - center;
            let b = *b - center;
            let edge = b.truncate() - a.truncate();
            let denominator = cross(direction, edge);
            if denominator.abs() < 1e-10 {
                continue;
            }
            let t = cross(a.truncate(), edge) / denominator;
            let u = cross(a.truncate(), direction) / denominator;
            if t > 0. && (0. ..1.).contains(&u) && hit.is_none_or(|(prior, _)| t < prior) {
                hit = Some((t, center.z + a.z + u * (b.z - a.z)));
            }
        }
        let (r, z) = hit.ok_or_else(|| {
            invalid("Eye contact requires an aperture surrounding its optical centre")
        })?;
        border.push((direction, (r + 0.0003).max(iris + 0.0003), z + 0.0004));
    }
    let mut points = Vec::with_capacity(1921);
    for j in 0..24 {
        for &(direction, outer, boundary) in &border {
            let (r, z) = if j <= 11 {
                let t = j as f32 / 11.;
                (
                    iris + (outer - iris) * t,
                    (center.z + z) * (1. - t) + boundary * t + 0.0003 * t * (1. - t),
                )
            } else {
                let a = (j - 11) as f32 / 13. * PI * 0.5;
                (outer * a.cos(), boundary + (rear - boundary) * a.sin())
            };
            points.push(Vec3::new(
                center.x + direction.x * r,
                center.y + direction.y * r,
                z,
            ));
        }
    }
    points.push(Vec3::new(center.x, center.y, rear));
    Ok(points)
}
