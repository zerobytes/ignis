//! Optical-footprint experiment. No automatic policy or reference shader change.
use super::{certificates, compile, compiler, experiment, material, render::Gpu};
use anyhow::{Result, ensure};
use std::path::Path;

const MODES: [&str; 5] = [
    "source",
    "previous",
    "analytic-previous",
    "regional",
    "analytic-regional",
];

pub fn run(dir: &Path) -> Result<()> {
    super::build::verify_inputs(dir)?;
    let mut asset = compile::decode(&std::fs::read(dir.join("refined.igl"))?)?;
    let package = std::fs::read(dir.join("previous.igc"))?;
    let prior = compiler::decode(&package)?;
    let hierarchy = certificates::Hierarchy::load(
        &std::fs::read(dir.join("previous.ich"))?,
        &prior,
        compiler::hash(&package),
    )?;
    let start = asset.mesh.faces.len() as u32 * 3;
    let count = prior.proxy.faces.len() as u32 * 3;
    let offset = asset.mesh.vertices.len() as u32;
    asset.mesh.vertices.extend_from_slice(&prior.proxy.vertices);
    asset.mesh.faces.extend(
        prior
            .proxy
            .faces
            .iter()
            .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset, 0]),
    );
    let full = vec![(0, asset.source_count)];
    let color = material::decode(&std::fs::read(dir.join("color.irm"))?)?;
    let normal = material::decode(&std::fs::read(dir.join("normal.irm"))?)?;
    let texture_bytes: usize = color
        .levels
        .iter()
        .chain(&normal.levels)
        .map(Vec::len)
        .sum();
    let mut rows = vec![];
    let mut returns = vec![];
    let mut timings = vec![];
    let mut adapters = vec![];
    for (width, height) in [(1728, 1117), (3456, 2234)] {
        let mut gpu = pollster::block_on(Gpu::new(width, height))?;
        gpu.install_rich_material(color.size, &color.levels, normal.size, &normal.levels);
        let original = gpu.rich_variant("implicit");
        let analytic = gpu.rich_variant("analytic-lod");
        let mesh = gpu.upload_rich(&asset.mesh);
        adapters.push(gpu.adapter.clone());
        let choose = |mode: usize, p| match mode {
            0 => full.clone(),
            1 | 2 => {
                if hierarchy.decide(p, height).proxy {
                    vec![(start, count)]
                } else {
                    full.clone()
                }
            }
            _ => experiment::regional(&asset, p, height, 0.05).ranges,
        };
        for case in 0..5 {
            let steps = if case == 4 { 32 } else { 8 };
            let mut first = [0; 5];
            for frame in 0..=steps {
                let p = experiment::pose(case, frame, steps);
                // Install only pipelines/bindings; textures remain shared and resident.
                gpu.select_rich_variant(&original);
                let source = gpu.capture_selected(&mesh, p, Some(&full))?;
                let previous = gpu.capture_selected(&mesh, p, Some(&choose(1, p)))?;
                let local = experiment::regional(&asset, p, height, 0.05);
                let regional = gpu.capture_selected(&mesh, p, Some(&local.ranges))?;
                gpu.select_rich_variant(&analytic);
                let analytic_previous = gpu.capture_selected(&mesh, p, Some(&choose(2, p)))?;
                let analytic_regional = gpu.capture_selected(&mesh, p, Some(&local.ranges))?;
                let pixels = [
                    &source,
                    &previous,
                    &analytic_previous,
                    &regional,
                    &analytic_regional,
                ];
                let hashes = pixels.map(|p| compiler::hash(bytemuck::cast_slice(p)));
                if frame == 0 {
                    first = hashes;
                }
                if frame == steps {
                    for (index, (mode, hash)) in MODES.iter().zip(hashes).enumerate() {
                        returns.push(serde_json::json!({"width":width,"case":case,"mode":mode,"pass":first[index]==hash,"first_hash":first[index].to_string(),"return_hash":hash.to_string()}));
                    }
                }
                rows.push(serde_json::json!({"width":width,"height":height,"case":case,"frame":frame,"distance":p.distance,
                    "previous":experiment::quality(&source,&previous),
                    "regional":experiment::quality(&source,&regional),
                    "analytic-previous":experiment::quality(&source,&analytic_previous),
                    "analytic-regional":experiment::quality(&source,&analytic_regional),
                    "material_previous":experiment::quality(&previous,&analytic_previous),
                    "material_regional":experiment::quality(&regional,&analytic_regional),
                    "selection":local,"previous_triangles":choose(1,p).iter().map(|r| r.1/3).sum::<u32>()}));
                for (mode, pixels) in MODES.iter().zip(pixels) {
                    experiment::png(
                        &dir.join(format!("analytic-{width}-{case}-{frame}-{mode}.png")),
                        pixels,
                        width,
                        height,
                    )?;
                }
            }
            let poses = (0..32)
                .map(|i| experiment::pose(case, i, 32))
                .collect::<Vec<_>>();
            for round in 0..8 {
                let order = if round % 2 == 0 {
                    [0, 1, 2, 3, 4, 4, 3, 2, 1, 0]
                } else {
                    [4, 3, 2, 1, 0, 0, 1, 2, 3, 4]
                };
                for (slot, mode) in order.into_iter().enumerate() {
                    gpu.select_rich_variant(if mode == 2 || mode == 4 {
                        &analytic
                    } else {
                        &original
                    });
                    let _ = gpu.batch_selected(&mesh, &poses, |p| choose(mode, p))?;
                    let timing = gpu.batch_selected(&mesh, &poses, |p| choose(mode, p))?;
                    timings.push(serde_json::json!({"width":width,"case":case,"round":round,"slot":slot,"mode":MODES[mode],"timing":timing}));
                }
            }
            println!("analytic {width} case{case} complete");
        }
    }
    let result = serde_json::json!({"sampling":"analytic-lod-normal-field","automatic":false,"adapters":adapters,"texture_bytes":texture_bytes,"geometry_bytes":asset.mesh.vertices.len()*12+asset.mesh.faces.len()*12,"source_triangles":asset.source_count/3,"quality":rows,"returns":returns,"timings":timings});
    std::fs::write(
        dir.join("analytic-results.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    ensure!(
        returns.iter().all(|r| r["pass"] == true),
        "return gate failed; analytic data retained, no promotion"
    );
    ensure!(
        rows.iter()
            .all(|r| r["analytic-previous"]["pass"] == true
                && r["analytic-regional"]["pass"] == true),
        "analytic image gate failed; diagnostic data retained, no promotion"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use glam::{DMat4, DVec2, DVec3};
    #[test]
    fn tangent_jacobian_matches_independent_ray_plane_intersections() {
        // Closed form inversion is checked against finite differences of rays,
        // not against a second implementation of the same Jacobian.
        for width in [1728., 3456.] {
            let height = width * 1117. / 1728.;
            for distance in [2.7, 6., 16., 64.] {
                for i in 0..16 {
                    let angle = i as f64 * std::f64::consts::TAU / 16.;
                    let eye = DVec3::new(angle.cos() * 0.85, angle.sin() * 0.85, 0.53).normalize()
                        * distance;
                    let matrix = DMat4::perspective_infinite_reverse_rh(
                        std::f64::consts::FRAC_PI_4,
                        width / height,
                        0.1,
                    ) * DMat4::look_at_rh(eye, DVec3::ZERO, DVec3::Z);
                    let p = DVec3::new(0.21, -0.13, 0.2);
                    for n in [
                        DVec3::Z,
                        DVec3::new(0.7, -0.2, 0.3),
                        DVec3::new(-0.2, 0.6, 0.8),
                    ] {
                        let clip = matrix * p.extend(1.);
                        let u = matrix * DVec3::new(n.z, 0., -n.x).extend(0.);
                        let v = matrix * DVec3::new(0., n.z, -n.y).extend(0.);
                        let ndc = clip.truncate().truncate() / clip.w;
                        let scale = DVec2::new(width, -height) * 0.5 / clip.w;
                        let a = (u.truncate().truncate() - ndc * u.w) * scale;
                        let b = (v.truncate().truncate() - ndc * v.w) * scale;
                        let det = a.perp_dot(b);
                        let dx = DVec2::new(b.y, -a.y) * (0.5 * n.z / det);
                        let dy = DVec2::new(-b.x, a.x) * (0.5 * n.z / det);
                        let inverse = matrix.inverse();
                        let ray_plane = |delta: DVec2| {
                            let uv = ndc + delta * DVec2::new(2. / width, -2. / height);
                            let unprojected = inverse * uv.extend(1.).extend(1.);
                            let direction = unprojected.truncate() / unprojected.w - eye;
                            let hit = eye + direction * (n.dot(p - eye) / n.dot(direction));
                            hit.truncate() * 0.5
                        };
                        for (axis, expected) in [(DVec2::X, dx), (DVec2::Y, dy)] {
                            let delta = axis * 0.001;
                            let observed = (ray_plane(delta) - ray_plane(-delta)) / 0.002;
                            assert!(
                                (observed - expected).length() < expected.length() * 1e-5 + 1e-9
                            );
                        }
                    }
                }
            }
        }
    }
}
