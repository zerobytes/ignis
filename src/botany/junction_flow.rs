//! Opt-in implicit woody junctions with protected collars and transition cuffs.
//! Uses the same bounded field operator exposed by mesh authoring.
use super::*;
use crate::mesh_organic::{Collar, Fusion, Junction, Options};

pub(super) fn strength(recipe: &Recipe, organs: impl IntoIterator<Item = usize>) -> Option<f64> {
    organs
        .into_iter()
        .filter_map(|i| recipe.edits.get(&recipe.source.organs[i].id)?.junction_flow)
        .max_by(f64::total_cmp)
        .filter(|v| *v > 0.)
}

/// Material fibres follow the authored positive cords instead of the axis alone.
pub(super) fn grain_angle(surface: &wood_surface::Surface, u: f64, angle: f64) -> f64 {
    let radius = surface.radius(u, angle);
    let mut sum = 0.;
    let mut drift = 0.;
    for cord in surface
        .relief
        .iter()
        .filter(|r| r.enabled && r.height > 0. && u >= r.path[0].u && u <= r.path.last().unwrap().u)
    {
        let i = cord
            .path
            .partition_point(|p| p.u < u)
            .saturating_sub(1)
            .min(cord.path.len() - 2);
        let (a, b) = (&cord.path[i], &cord.path[i + 1]);
        let t = ((u - a.u) / (b.u - a.u)).clamp(0., 1.);
        let t = t * t * (3. - 2. * t);
        let direction = a.angle * (1. - t) + b.angle * t;
        let distance = 2. * radius * ((angle - direction) * 0.5).sin();
        let weight = (-(distance / (cord.width * 2.)).powi(2)).exp();
        drift += weight * (direction - cord.path[0].angle);
        sum += weight;
    }
    if sum > 1e-12 {
        angle - drift / sum
    } else {
        angle
    }
}

pub(super) fn build(
    collars: &[Collar; 3],
    options: Options,
    center: DVec3,
    flow: Option<f64>,
) -> Result<Junction> {
    let Some(flow) = flow else {
        return crate::mesh_organic::junction(collars, options);
    };
    let depth = collars
        .iter()
        .map(|c| {
            DVec3::from_array(c.normal.map(f64::from))
                .normalize()
                .dot(DVec3::from_array(c.center.map(f64::from)) - center)
        })
        .fold(f64::INFINITY, f64::min);
    let radius = collars
        .iter()
        .map(|c| f64::from(c.radius))
        .fold(f64::INFINITY, f64::min);
    let guard = (radius * 0.10).min(depth * 0.08);
    if !depth.is_finite() || depth <= 0.003 || guard <= 0.001 {
        return Err(invalid(
            "Woody junction flow has no resolved interior support domain",
        ));
    }
    let mut error = invalid("Woody junction flow could not fit its support domain");
    for ratio in [1.3, 1.0, 0.8, 0.65] {
        let core = (radius * ratio).min(depth * 0.55);
        match crate::mesh_organic::fused_junction(
            collars,
            Options {
                refinement_levels: options.refinement_levels.max(2),
                ..options
            },
            Fusion {
                center: center.as_vec3().to_array(),
                center_radius: core as f32,
                blend_radius: (core * (0.15 + 0.35 * flow)) as f32,
                guard_length: guard as f32,
                transition_lengths: [((depth - guard) * 0.20) as f32; 3],
                max_displacement: (depth * 4. + radius * 4.) as f32,
                tolerance: 0.0005,
                max_volume_loss: 0.75,
                max_face_rotation_degrees: 85.,
            },
        ) {
            Ok(mesh) => return Ok(mesh),
            Err(e) => error = e,
        }
    }
    Err(error)
}

pub fn editor(ui: &mut egui::Ui, edit: &mut OrganEdit) {
    let mut enabled = edit.junction_flow.is_some();
    if ui
        .checkbox(&mut enabled, "Fusão volumétrica suave nas junções")
        .changed()
    {
        edit.junction_flow = enabled.then_some(0.6);
    }
    if let Some(flow) = &mut edit.junction_flow {
        ui.add(egui::Slider::new(flow, 0.01..=1.).text("Mistura entre braços"));
        ui.label("Preserva colares e cria faixas de transição; encaixes sem espaço são recusados. Prévia e aplicação usam os botões abaixo.");
    }
}

/// Restore the actual swept wood mass between cuffs. The collar constructor's
/// small central sphere alone can produce a visibly pinched waist.
pub(super) fn wood_mass(
    mesh: &mut crate::mesh_organic::MultiJunction,
    poses: &[bake::Pose],
    ports: &[(usize, f64)],
    ends: &[f64],
    recipe: &Recipe,
) -> Result<()> {
    struct Guide {
        organ: usize,
        end: f64,
        samples: Vec<(f64, DVec3)>,
        arc: Vec<f64>,
    }
    impl Guide {
        fn offset(&self, u: f64, distance: f64) -> f64 {
            let x = u * (self.arc.len() - 1) as f64;
            let i = (x as usize).min(self.arc.len() - 2);
            let target =
                (self.arc[i] + (self.arc[i + 1] - self.arc[i]) * (x - i as f64) + distance)
                    .clamp(0., *self.arc.last().unwrap());
            let j = self
                .arc
                .partition_point(|v| *v < target)
                .saturating_sub(1)
                .min(self.arc.len() - 2);
            (j as f64 + (target - self.arc[j]) / (self.arc[j + 1] - self.arc[j]))
                / (self.arc.len() - 1) as f64
        }
    }
    let guides: Vec<_> = ports
        .iter()
        .zip(ends)
        .map(|(&(organ, u), &end)| Guide {
            organ,
            end,
            arc: {
                let mut arc = vec![0.];
                for w in poses[organ].samples.windows(2) {
                    arc.push(arc.last().unwrap() + w[0].p.distance(w[1].p));
                }
                arc
            },
            samples: (0..=16)
                .map(|i| {
                    let v = u + (end - u) * i as f64 / 16.;
                    (v, poses[organ].at(v).p)
                })
                .collect(),
        })
        .collect();
    let center = guides
        .iter()
        .map(|g| poses[g.organ].at(g.end).p)
        .sum::<DVec3>()
        / guides.len() as f64;
    let normals: Vec<_> = ports
        .iter()
        .enumerate()
        .map(|(k, &(i, u))| poses[i].at(u).t * if k == 0 { -1. } else { 1. })
        .collect();
    let collar: Vec<_> = ports.iter().map(|&(i, u)| poses[i].at(u).p).collect();
    let radius = ports
        .iter()
        .map(|&(i, u)| poses[i].radius_at(&recipe.source, u).0)
        .fold(f64::INFINITY, f64::min);
    let section = |organ: usize, u: f64, p: DVec3, radial_only: bool| {
        let pose = &poses[organ];
        let f = pose.at(u);
        let origin = root_sections::center(pose, &recipe.source, u);
        let d = p - origin;
        let angle = d.dot(f.b()).atan2(d.dot(f.n));
        let r = root_sections::point(pose, &recipe.source, u, angle).distance(origin);
        (if radial_only {
            (d - f.t * d.dot(f.t)).length()
        } else {
            d.length()
        }) - r
    };
    let field = |p: DVec3| {
        let mut value = f64::INFINITY;
        for g in &guides {
            let mut closest = (f64::INFINITY, 0.);
            for w in g.samples.windows(2) {
                let axis = w[1].1 - w[0].1;
                let t = ((p - w[0].1).dot(axis) / axis.length_squared().max(1e-24)).clamp(0., 1.);
                let d = p.distance_squared(w[0].1 + axis * t);
                if d < closest.0 {
                    closest = (d, w[0].0 + (w[1].0 - w[0].0) * t);
                }
            }
            let distance = section(g.organ, closest.1, p, false);
            let k = radius * 0.35;
            if value.is_finite() {
                let h = (0.5 + 0.5 * (distance - value) / k).clamp(0., 1.);
                value = distance * (1. - h) + value * h - k * h * (1. - h);
            } else {
                value = distance;
            }
        }
        // Match the actual taper/cords near each protected cuff, rather than a
        // circular cylinder whose radial profile could leave a visible step.
        for (k, &(i, u)) in ports.iter().enumerate() {
            let distance = -(p - collar[k]).dot(normals[k]);
            let width = radius * 0.65;
            let t = (distance / width).clamp(0., 1.);
            let t = t * t * t * (10. + t * (-15. + 6. * t));
            let v = guides[k].offset(u, if k == 0 { distance } else { -distance });
            value = section(i, v, p, true) * (1. - t) + value * t;
        }
        value
    };
    if !field(center).is_finite() || field(center) >= 0. {
        return Err(invalid(
            "Woody flow requires a shared interior volume between arms",
        ));
    }
    let old: Vec<_> = mesh
        .geometry
        .positions
        .iter()
        .map(|p| DVec3::from_array(p.map(f64::from)))
        .collect();
    let fixed = 2 * mesh.collar_vertices;
    let mut target = old.clone();
    for (i, &p) in old.iter().enumerate().skip(fixed) {
        let ray = (p - center).normalize_or_zero();
        if ray.length_squared() < 0.5 {
            continue;
        }
        let mut high = f64::INFINITY;
        for k in 0..ports.len() {
            let denominator = normals[k].dot(ray);
            if denominator > 1e-10 {
                high = high.min(normals[k].dot(collar[k] - center) / denominator - 1e-6);
            }
        }
        if !high.is_finite() || high <= 0. || field(center + ray * high) < 0. {
            continue;
        }
        let mut low = 0.;
        for _ in 0..32 {
            let mid = (low + high) * 0.5;
            if field(center + ray * mid) > 0. {
                high = mid;
            } else {
                low = mid;
            }
        }
        target[i] = center + ray * ((low + high) * 0.5);
    }
    for blend in [1., 0.75, 0.5, 0.25, 0.125, 0.0625] {
        let candidate: Vec<_> = old
            .iter()
            .zip(&target)
            .map(|(p, q)| p.lerp(*q, blend).as_vec3().as_dvec3())
            .collect();
        let valid = mesh.geometry.triangles.iter().all(|t| {
            let [a, b, c] = t.map(|i| old[i as usize]);
            let n = (b - a).cross(c - a);
            (1..=8).all(|step| {
                let [p, q, r] =
                    t.map(|i| old[i as usize].lerp(candidate[i as usize], step as f64 / 8.));
                let m = (q - p).cross(r - p);
                m.is_finite() && m.length_squared() > 1e-24 && n.dot(m) > 0.
            })
        });
        if valid {
            for (p, q) in mesh.geometry.positions.iter_mut().zip(candidate) {
                *p = q.as_vec3().to_array();
            }
            return Ok(());
        }
    }
    Err(invalid(
        "Woody mass field cannot fit without reversing a junction face",
    ))
}
