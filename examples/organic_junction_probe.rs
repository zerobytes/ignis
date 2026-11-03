//! Emit a connected fork and an overlapping-tube control for native MCP inspection.
use glam::Vec3;
use ignis::{
    mesh_organic::{Collar, Fusion, Options, fused_junction, junction},
    mesh_topology::Topology,
    meshes::Geometry,
};

fn radial_degree(geometry: &Geometry) -> f64 {
    let mut angle = 0.;
    for &tri in &geometry.triangles {
        let [a, b, c] =
            tri.map(|i| glam::DVec3::from_array(geometry.positions[i as usize].map(f64::from)));
        let determinant = a.dot(b.cross(c));
        assert!(
            determinant > 0.,
            "The closed probe must be radially oriented"
        );
        angle += 2.
            * determinant.atan2(
                a.length() * b.length() * c.length()
                    + a.dot(b) * c.length()
                    + b.dot(c) * a.length()
                    + c.dot(a) * b.length(),
            );
    }
    let degree = angle / (4. * std::f64::consts::PI);
    assert!(
        (degree - 1.).abs() < 1e-5,
        "The probe must cover radial directions exactly once"
    );
    degree
}

fn extend(
    geometry: &mut Geometry,
    start: &[u32],
    center: Vec3,
    normal: Vec3,
    length: f32,
    taper: f32,
) {
    let mut end = Vec::new();
    for &i in start {
        end.push(geometry.positions.len() as u32);
        let radial = Vec3::from(geometry.positions[i as usize]) - center;
        geometry
            .positions
            .push((center + normal * length + radial * taper).to_array());
    }
    for j in 0..start.len() {
        let k = (j + 1) % start.len();
        geometry
            .triangles
            .extend([[start[j], start[k], end[k]], [start[j], end[k], end[j]]]);
    }
    for j in 1..end.len() - 1 {
        geometry.triangles.push([end[0], end[j], end[j + 1]]);
    }
}
fn main() {
    let s = 0.5f32.sqrt();
    let normals = [[0., -1., 0.], [-s, s, 0.], [s, s, 0.]];
    let collars = std::array::from_fn(|i| Collar {
        id: format!("collar{i}"),
        center: normals[i].map(|x| x * 1.2),
        normal: normals[i],
        radial: [0., 0., 1.],
        radius: [0.45, 0.35, 0.3][i],
        samples: 12,
    });
    let patch = junction(
        &collars,
        Options {
            refinement_levels: 2,
            ..Default::default()
        },
    )
    .expect("admissible fork");
    let mut connected = patch.geometry.clone();
    for (i, b) in patch.boundaries.iter().enumerate() {
        extend(
            &mut connected,
            &b.vertices,
            Vec3::from(collars[i].center),
            Vec3::from(collars[i].normal),
            [1.0, 1.4, 1.7][i],
            [1.15, 0.6, 0.45][i],
        );
    }
    let topology = Topology::inspect(&connected).unwrap();
    assert_eq!(topology.boundary_edges, 0);
    assert_eq!(topology.non_manifold_edges, 0);
    assert_eq!(
        topology.vertices as i64 - topology.edges.len() as i64 + topology.faces.len() as i64,
        2
    );
    let connected_degree = radial_degree(&connected);
    let fitted = fused_junction(
        &collars,
        Options {
            refinement_levels: 2,
            ..Default::default()
        },
        Fusion::default(),
    )
    .expect("admissible fitted fork");
    let mut fused = fitted.geometry.clone();
    for (i, b) in fitted.boundaries.iter().enumerate() {
        extend(
            &mut fused,
            &b.vertices,
            Vec3::from(collars[i].center),
            Vec3::from(collars[i].normal),
            [1.0, 1.4, 1.7][i],
            [1.15, 0.6, 0.45][i],
        );
    }
    let ft = Topology::inspect(&fused).unwrap();
    assert_eq!(ft.boundary_edges, 0);
    assert_eq!(ft.non_manifold_edges, 0);
    assert_eq!(
        ft.vertices as i64 - ft.edges.len() as i64 + ft.faces.len() as i64,
        2
    );
    let fused_degree = radial_degree(&fused);
    let mut overlap = Geometry {
        material_coordinates: None,
        positions: vec![],
        triangles: vec![],
    };
    for (i, c) in collars.iter().enumerate() {
        let center = -Vec3::from(c.normal) * 0.25;
        let radial = Vec3::from(c.radial);
        let normal = Vec3::from(c.normal);
        let other = normal.cross(radial);
        let ring: Vec<_> = (0..c.samples)
            .map(|j| {
                let a = std::f32::consts::TAU * f32::from(j) / f32::from(c.samples);
                let index = overlap.positions.len() as u32;
                overlap
                    .positions
                    .push((center + (radial * a.cos() + other * a.sin()) * c.radius).to_array());
                index
            })
            .collect();
        for j in 1..ring.len() - 1 {
            overlap.triangles.push([ring[0], ring[j + 1], ring[j]]);
        }
        extend(
            &mut overlap,
            &ring,
            center,
            normal,
            1.45 + [1.0, 1.4, 1.7][i],
            [1.15, 0.6, 0.45][i],
        );
    }
    overlap.validate().unwrap();
    println!(
        "{}",
        serde_json::json!({"fused":fused,"connected":connected,"overlap":overlap,"junction":fitted.report,"topology_only":patch.report,"closed_boundary_edges":ft.boundary_edges,"connected_radial_degree":connected_degree,"fused_radial_degree":fused_degree})
    );
}
