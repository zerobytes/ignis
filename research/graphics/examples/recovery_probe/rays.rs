use super::{
    compiler::{self, Mesh},
    render::Pose,
};
use anyhow::Result;
use glam::{DVec3, Mat4, Vec3};
use std::path::Path;
fn trace(mesh: &Mesh, pose: Pose, width: u32, height: u32, x: u32, y: u32) -> serde_json::Value {
    let matrix = Mat4::perspective_infinite_reverse_rh(
        std::f32::consts::FRAC_PI_4,
        width as f32 / height as f32,
        0.1,
    ) * Mat4::look_at_rh(pose.eye(), Vec3::ZERO, Vec3::Z);
    let matrix = matrix.as_dmat4();
    let ndc = DVec3::new(
        2. * (x as f64 + 0.5) / width as f64 - 1.,
        1. - 2. * (y as f64 + 0.5) / height as f64,
        1.,
    );
    let near = matrix.inverse().project_point3(ndc);
    let origin = pose.eye().as_dvec3();
    let direction = (near - origin).normalize();
    let mut closest = f64::INFINITY;
    let mut hit = serde_json::Value::Null;
    for (id, t) in mesh.faces.iter().enumerate() {
        let [a, b, c] = [t[0], t[1], t[2]]
            .map(|i| DVec3::from_array(mesh.vertices[i as usize].p.map(f64::from)));
        let e = b - a;
        let f = c - a;
        let h = direction.cross(f);
        let det = e.dot(h);
        if det.abs() < 1e-20 {
            continue;
        }
        let q = origin - a;
        let u = q.dot(h) / det;
        if !(0. ..=1.).contains(&u) {
            continue;
        }
        let cross = q.cross(e);
        let v = direction.dot(cross) / det;
        if v < 0. || u + v > 1. {
            continue;
        }
        let distance = f.dot(cross) / det;
        if distance <= 0. || distance >= closest {
            continue;
        }
        closest = distance;
        let n = [t[0], t[1], t[2]]
            .map(|i| DVec3::from_array(mesh.vertices[i as usize].n.map(f64::from)));
        let raw = n[0] * (1. - u - v) + n[1] * u + n[2] * v;
        let sign = if e.cross(f).dot(direction) < 0. {
            1.
        } else {
            -1.
        };
        let normal = raw.normalize() * sign;
        let p = origin + direction * distance;
        let l = Vec3::new(pose.light.cos(), pose.light.sin(), 0.6)
            .normalize()
            .as_dvec3();
        let eye = (origin - p).normalize();
        let half = (l + eye).normalize();
        let detail = 0.92 + 0.08 * (p.x * 67. + p.z * 23.).sin() * (p.y * 59. - p.z * 17.).sin();
        let color = DVec3::new(0.32, 0.26, 0.19) * detail * (0.15 + 0.85 * normal.dot(l).max(0.))
            + DVec3::splat(0.12 * normal.dot(half).max(0.).powf(24.));
        let screen = [a, b, c].map(|p| {
            matrix.project_point3(p) * DVec3::new(width as f64 / 2., height as f64 / 2., 1.)
        });
        let area = (screen[1] - screen[0]).cross(screen[2] - screen[0]).z.abs() / 2.;
        hit = serde_json::json!({"face":id,"distance":distance,"position":p.to_array(),"normal_length":raw.length(),"normal":normal.to_array(),"color":color.to_array(),"barycentric":[1.-u-v,u,v],"projected_area":area});
    }
    hit
}
pub fn run(old_dir: &Path, dir: &Path) -> Result<()> {
    let base = compiler::decode(&std::fs::read(old_dir.join("source-1.igc"))?)?.source;
    let mut rows = vec![];
    for (factor, frame, x, y) in [
        (2, 4, 865, 565),
        (10, 1, 855, 551),
        (100, 1, 855, 551),
        (100, 2, 867, 544),
    ] {
        let source = compiler::decode(&std::fs::read(
            old_dir.join(format!("source-{factor}.igc")),
        )?)?
        .source;
        let phase = frame as f32 * std::f32::consts::TAU / 8.;
        let pose = Pose {
            angle: phase,
            distance: 64.,
            light: phase * 1.7 + 0.3,
        };
        rows.push(serde_json::json!({"factor":factor,"frame":frame,"pixel":[x,y],"base":trace(&base,pose,1728,1117,x,y),"dense":trace(&source,pose,1728,1117,x,y)}));
    }
    std::fs::write(
        dir.join("ray-diagnostics.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(())
}
