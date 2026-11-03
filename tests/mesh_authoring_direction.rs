use glam::Vec3;
use ignis::{
    mesh_tools::{self, Primitive},
    meshes::{self, BrushKind},
};

#[test]
fn draw_paths_are_direction_independent_without_folded_faces() {
    for shape in [Primitive::Plane, Primitive::Sphere] {
        let base = mesh_tools::primitive(shape, 24, 2.).unwrap();
        let asset: meshes::Asset = serde_json::from_value(serde_json::json!({
            "id":"surface", "geometry":base, "shading":"smooth"
        }))
        .unwrap();
        for diagonal in [false, true] {
            for strength in [0.08, -0.08] {
                let centers: Vec<_> = (0..25)
                    .map(|i| {
                        let x = -0.6 + i as f32 * 0.05;
                        if shape == Primitive::Sphere {
                            let y = if diagonal { x * 0.5 } else { 0. };
                            [x, y, (1. - x * x - y * y).sqrt()]
                        } else {
                            [x, 0., if diagonal { x * 0.5 } else { 0. }]
                        }
                    })
                    .collect();
                let stroke = meshes::Stroke {
                    kind: BrushKind::Draw,
                    centers,
                    radius: 0.4,
                    strength,
                    direction: [0., 1., 0.],
                };
                let a = mesh_tools::sculpt_preview(&asset, &stroke).unwrap();
                let mut reversed = stroke.clone();
                reversed.centers.reverse();
                let b = mesh_tools::sculpt_preview(&asset, &reversed).unwrap();
                for (p, q) in a.geometry.positions.iter().zip(&b.geometry.positions) {
                    assert!(
                        Vec3::from(*p).distance(Vec3::from(*q)) < 0.000002,
                        "direction changed the shape"
                    );
                }
                for t in &base.triangles {
                    let [p, q, r] = t.map(|i| Vec3::from(a.geometry.positions[i as usize]));
                    let [u, v, w] = t.map(|i| Vec3::from(base.positions[i as usize]));
                    assert!(
                        (q - p)
                            .cross(r - p)
                            .normalize()
                            .dot((v - u).cross(w - u).normalize())
                            > 0.1,
                        "folded triangle"
                    );
                }
            }
        }
    }
}
