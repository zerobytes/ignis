//! Adversarial current-frame updates; intentionally uncull the CPU reference.
use super::*;

pub(super) fn run(world: &World, renderer: &mut Renderer) -> Result<()> {
    let mut rows = Vec::new();
    for size in RESOLUTIONS {
        for case in 0..8 {
            let mut p = Pose::new();
            world.pose(if case == 5 { 0 } else { 1 }, 0., size, &mut p);
            let (name, eye, direction) = match case {
                0 => ("inside_wall", Vec3::new(-14., 6., 20.), Vec3::Z),
                1 => ("wall_near_plane", Vec3::new(-14., 6., 19.35), Vec3::Z),
                2 => ("gap_edge", Vec3::new(-2., 4., 19.2), Vec3::Z),
                3 => ("inside_asset_bounds", Vec3::new(0., 4., 12.), Vec3::Z),
                4 => ("wall_present", Vec3::new(0., 4., 45.), Vec3::Z),
                5 => ("wall_removed_same_camera", Vec3::new(0., 4., 45.), Vec3::Z),
                6 => ("asset_behind_wall", Vec3::new(-14., 4., 10.), Vec3::Z),
                _ => ("asset_moved_in_front", Vec3::new(-14., 4., 10.), Vec3::Z),
            };
            let eye = if case == 3 {
                let inst = p.input[1];
                let a = inst.meta[0] as usize;
                let [lo, hi] = world.scene.asset_bounds[a];
                Vec3::from_array(inst.position[..3].try_into().unwrap())
                    + (lo + hi) * 0.5 * inst.position[3]
            } else {
                eye
            };
            if case >= 6 {
                p.input[1].position[0] = -14.;
                p.input[1].position[2] = if case == 6 { 35. } else { 14. };
            }
            p.camera.eye = eye;
            p.camera.view = Mat4::look_at_rh(eye, eye + direction, Vec3::Y);
            p.camera.matrix = Mat4::perspective_infinite_reverse_rh(
                60f32.to_radians(),
                size[0] as f32 / size[1] as f32,
                0.1,
            ) * p.camera.view;
            p.camera.inverse = p.camera.matrix.inverse();
            p.input.truncate(101);
            p.baseline.clear();
            p.triangles = p
                .obstacles
                .iter()
                .map(|d| (d.indices.end - d.indices.start) as u64 / 3)
                .sum();
            for a in 0..3 {
                let first = p.input.len() as u32;
                for id in &world.members[a * 32..a * 32 + 32] {
                    p.input.push(p.input[*id as usize]);
                }
                let indices = world.scene.ranges[a].clone();
                p.triangles += (indices.end - indices.start) as u64 / 3 * 32;
                p.baseline.push(Draw {
                    indices,
                    instance: first,
                    count: 32,
                });
            }
            let (reference, _) = renderer.capture(&p, 0)?;
            let mut checks = Vec::new();
            for mode in 1..6 {
                let (image, _) = renderer.capture(&p, mode)?;
                let error = compare(&reference, &image);
                if error["changed_pixels"] != 0 {
                    save(
                        &Path::new(ROOT).join("edge-failure-full.png"),
                        &reference,
                        size[0],
                    )?;
                    save(&Path::new(ROOT).join("edge-failure.png"), &image, size[0])?;
                }
                ensure!(
                    error["changed_pixels"] == 0,
                    "{name} {size:?} mode{mode}: {error}"
                );
                checks.push(error);
            }
            println!("{name} {size:?}: exact pixels in all five GPU selectors");
            rows.push(json!({"case":name,"size":size,"checks":checks}));
        }
    }
    fs::write(
        Path::new(ROOT).join("edge-checks.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}

/// Fill previously unmeasured intermediate poses, including maximum wall displacement.
pub(super) fn benchmark(world: &World, renderer: &mut Renderer) -> Result<()> {
    let path = Path::new(ROOT).join("report.json");
    let mut data: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    for row in data["rows"].as_array_mut().context("missing rows")? {
        let frame = row["frame"].as_u64().context("frame")? as usize;
        if [3, 9, 16, 22].contains(&frame) && row["timings"].is_null() {
            let size: [u32; 2] = serde_json::from_value(row["size"].clone())?;
            row["timings"] = renderer.bench(
                world,
                row["scenario_index"].as_u64().context("scenario")? as usize,
                frame,
                size,
            )?;
            println!("intermediate moving timing {}", row["id"]);
        }
    }
    fs::write(path, serde_json::to_vec_pretty(&data)?)?;
    Ok(())
}
