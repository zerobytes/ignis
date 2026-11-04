//! Derived closed demonstration geometry for living reproductive cohorts.
use super::*;
use glam::DVec3;

fn ellipsoid(g: &mut meshes::Geometry, center: DVec3, radii: DVec3) {
    const N: usize = 12;
    const M: usize = 8;
    let base = g.positions.len() as u32;
    g.positions
        .push((center + DVec3::Y * radii.y).as_vec3().to_array());
    for j in 1..M {
        let theta = std::f64::consts::PI * j as f64 / M as f64;
        for k in 0..N {
            let phi = std::f64::consts::TAU * k as f64 / N as f64;
            g.positions.push(
                (center
                    + radii
                        * DVec3::new(
                            theta.sin() * phi.cos(),
                            theta.cos(),
                            theta.sin() * phi.sin(),
                        ))
                .as_vec3()
                .to_array(),
            );
        }
    }
    let bottom = g.positions.len() as u32;
    g.positions
        .push((center - DVec3::Y * radii.y).as_vec3().to_array());
    for k in 0..N {
        let a = base + 1 + k as u32;
        let b = base + 1 + ((k + 1) % N) as u32;
        g.triangles.push([base, b, a]);
        for j in 0..M - 2 {
            let a = base + 1 + (j * N + k) as u32;
            let b = base + 1 + (j * N + (k + 1) % N) as u32;
            let c = a + N as u32;
            let d = b + N as u32;
            g.triangles.extend([[a, b, c], [b, d, c]]);
        }
        let a = base + 1 + ((M - 2) * N + k) as u32;
        let b = base + 1 + ((M - 2) * N + (k + 1) % N) as u32;
        g.triangles.push([a, b, bottom]);
    }
}

pub fn append(
    recipe: &Recipe,
    poses: &[bake::Pose],
    document: &mut meshes::Document,
) -> Result<()> {
    let Some(plant) = &recipe.living else {
        return Ok(());
    };
    let mut geometry: [meshes::Geometry; 3] = std::array::from_fn(|_| meshes::Geometry {
        positions: vec![],
        triangles: vec![],
        material_coordinates: None,
    });
    for (id, stage) in plant
        .production
        .attached(&plant.definition.schedule, growth::MAX_ATTACHED)
        .map_err(invalid)?
    {
        let pose = poses
            .iter()
            .find(|p| recipe.source.organs[p.source].id == id.site)
            .ok_or_else(|| invalid("Production support missing"))?;
        let center = pose.at(1.).p;
        let scale = recipe.settings.scale;
        match stage {
            lifecycle::Stage::Flower => {
                for petal in 0..5 {
                    let a = petal as f64 * std::f64::consts::TAU / 5.;
                    let offset = DVec3::new(a.cos() * 0.055, 0., a.sin() * 0.055) * scale;
                    ellipsoid(
                        &mut geometry[0],
                        center + offset,
                        DVec3::new(0.052, 0.018, 0.052) * scale,
                    );
                }
                ellipsoid(
                    &mut geometry[0],
                    center + DVec3::Y * 0.012 * scale,
                    DVec3::splat(0.025) * scale,
                );
            }
            lifecycle::Stage::GrowingFruit => {
                let start = plant
                    .definition
                    .schedule
                    .start(id.cohort)
                    .ok_or_else(|| invalid("Cohort overflow"))?
                    + plant.definition.schedule.flower_duration;
                let fraction = (plant.production.plant.time - start) as f64
                    / plant.definition.schedule.fruit_duration.max(1) as f64;
                let radius = (0.025 + 0.06 * fraction) * scale;
                ellipsoid(
                    &mut geometry[1],
                    center - DVec3::Y * radius * 0.65,
                    DVec3::new(radius, radius * 1.1, radius),
                );
            }
            lifecycle::Stage::RipeFruit => ellipsoid(
                &mut geometry[2],
                center - DVec3::Y * 0.055 * scale,
                DVec3::new(0.085, 0.094, 0.085) * scale,
            ),
            lifecycle::Stage::Dropped => {}
        }
    }
    for (index, (suffix, color)) in [
        ("flowers", [0.95, 0.70, 0.30]),
        ("green_fruit", [0.12, 0.38, 0.035]),
        ("ripe_fruit", [0.72, 0.045, 0.015]),
    ]
    .into_iter()
    .enumerate()
    {
        let id = format!("{}_{}", recipe.prefix, suffix);
        let mut g = std::mem::replace(
            &mut geometry[index],
            meshes::Geometry {
                positions: vec![],
                triangles: vec![],
                material_coordinates: None,
            },
        );
        if g.positions.is_empty() {
            g.positions = vec![[0.; 3], [0.00001, 0., 0.], [0., 0., 0.00001]];
            g.triangles = vec![[0, 1, 2]];
        }
        document.assets.push(meshes::Asset {
            id: id.clone(),
            geometry: Arc::new(g),
            shading: meshes::Shading::Smooth,
            material: Default::default(),
            rig: None,
            clips: vec![],
        });
        document.instances.push(meshes::Instance {
            id: id.clone(),
            asset: id,
            position: recipe.settings.position,
            rotation: [0., 0., 0., 1.],
            scale: 1.,
            color,
            playback: None,
            attachment: None,
        });
    }
    Ok(())
}
