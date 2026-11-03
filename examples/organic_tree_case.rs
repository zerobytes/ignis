//! Diagnose a local tangent probe of the shared tree's first primary fork.
//! This is not an exact surface cut and never moves the authored tree.
use glam::Vec3;
use ignis::mesh_organic::{Collar, Options, junction};

fn main() {
    // Source: tree authoring study17fb9ed, skeleton()/trunk(), first limb.
    let t = 0.34f32;
    let anchor = Vec3::new(
        0.18 * (t * 2.2).sin() + 0.1 * t,
        4.8 * t,
        0.15 * (t * 3.).sin(),
    );
    let trunk = Vec3::new(0.396 * (t * 2.2).cos() + 0.1, 4.8, 0.45 * (t * 3.).cos()).normalize();
    let end = Vec3::new(-1.55, 5.25, 0.7);
    let limb = ((end - anchor) * 0.28 + Vec3::new(0., 0.45, 0.)).normalize();
    let radius = 0.46 * (1. - t * 21. / 24.).powf(1.25) + 0.015;
    let normals = [-trunk, limb, trunk];
    let collars = std::array::from_fn(|i| {
        let normal = normals[i];
        let radial = (Vec3::Z - normal * normal.z).normalize();
        Collar {
            id: ["trunk_below", "limb_zero", "trunk_above"][i].into(),
            center: (anchor + normal * [0.3, 0.4, 0.3][i]).to_array(),
            normal: normal.to_array(),
            radial: radial.to_array(),
            radius: radius * [1., 0.7, 0.95][i],
            samples: 12,
        }
    });
    let result = junction(&collars, Options::default());
    let diagnostic = match result {
        Ok(out) => serde_json::json!({"supported":true,"report":out.report}),
        Err(error) => serde_json::json!({"supported":false,"reason":error.message}),
    };
    assert_eq!(
        diagnostic["supported"], false,
        "This close, acute local probe must not be silently reauthored"
    );
    let parameters:Vec<_>=collars.iter().map(|c|serde_json::json!({"id":c.id,"center":c.center,"normal":c.normal,"radial":c.radial,"radius":c.radius,"samples":c.samples})).collect();
    println!(
        "{}",
        serde_json::json!({"source":"tree-authoring-study17fb9ed first primary fork","probe":"local tangents; not an exact mesh cut","attachment_t":t,"anchor":anchor.to_array(),"divergence_degrees":trunk.dot(limb).clamp(-1.,1.).acos().to_degrees(),"collars":parameters,"diagnostic":diagnostic})
    );
}
