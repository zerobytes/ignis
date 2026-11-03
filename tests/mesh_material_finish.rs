use ignis::{
    mesh_material_finish::{self as finish, Affine, Options, Plan},
    meshes::{Asset, Geometry},
};
use std::sync::Arc;

fn reference() -> Affine {
    Affine {
        columns: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        offset: [0.; 3],
    }
}
fn at(r: &Affine, p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| {
        (f64::from(r.offset[i])
            + (0..3)
                .map(|j| f64::from(r.columns[j][i]) * f64::from(p[j]))
                .sum::<f64>()) as f32
    })
}
fn fixture(r: Affine) -> (Asset, Plan, Options) {
    let mut positions = vec![];
    for x in [-1., 0., 1., 2., 0.23, 0.62, 0.1, 0.9] {
        positions.extend([[x, 0., 0.], [x, 1., 0.]]);
    }
    let mut triangles = vec![];
    let mut quad = |a, b| triangles.extend([[a, b, a + 1], [a + 1, b, b + 1]]);
    quad(0, 2);
    quad(4, 6); // Retained source prefix.
    for (a, b) in [(2, 12), (12, 8), (8, 10), (10, 14), (14, 4)] {
        quad(a, b);
    }
    let mut material: Vec<_> = positions.iter().map(|&p| at(&r, p)).collect();
    let pairs = vec![[2, 12], [3, 13], [4, 14], [5, 15]];
    for &[s, t] in &pairs {
        material[t as usize] = material[s as usize];
    }
    let g = Geometry {
        positions,
        triangles,
        material_coordinates: Some(material),
    };
    let weights: Vec<_> = (0..16)
        .map(|_| serde_json::json!({"joints":[0,0,0,0],"weights":[1,0,0,0]}))
        .collect();
    let mut a:Asset=serde_json::from_value(serde_json::json!({"id":"patch","geometry":g,
      "shading":"smooth","rig":{"bones":[{"name":"root","parent":null,"translation":[0,0,0],"rotation":[0,0,0,1]}],"weights":weights},
      "clips":[{"id":"motion","duration_ticks":10,"tracks":[{"bone":0,"keys":[{"tick":0,"translation":[0,0,0],"rotation":[0,0,0,1]},{"tick":10,"translation":[0,0,0.5],"rotation":[0,0,0,1]}]}]}]})).unwrap();
    a.material = Default::default();
    (
        a,
        Plan {
            retained_vertices: 8,
            retained_faces: 4,
            iso_vertices: 4,
            pairs,
        },
        Options {
            reference: r,
            iterations: 64,
            tolerance: 1e-5,
            max_change: 10.,
            max_anisotropy: 10000.,
        },
    )
}
fn near(a: [f32; 3], b: [f32; 3]) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() < 1e-5, "{a:?} != {b:?}");
    }
}
#[test]
fn irregular_affine_identity_has_zero_rhs_and_preserves_correct_interior() {
    let (a, p, o) = fixture(reference());
    let before = a.clone();
    let f = finish::finish(&a, &p, &o).unwrap();
    assert_eq!(a, before);
    assert_eq!(f.report.iterations, 0);
    assert_eq!(f.report.residual_max, 0.);
    assert_eq!(f.report.changed_interior, 0);
    assert_eq!(f.report.changed_guards, 4);
    assert_eq!(f.report.before.guard.near_singular_faces, 4);
    assert_eq!(f.report.after.guard.near_singular_faces, 0);
    assert_eq!(f.asset.geometry.positions, a.geometry.positions);
    assert_eq!(f.asset.geometry.triangles, a.geometry.triangles);
    assert!(Arc::ptr_eq(
        f.asset.rig.as_ref().unwrap(),
        a.rig.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(&f.asset.clips[0], &a.clips[0]));
    assert_eq!(f.asset.material, a.material);
    assert_eq!(f.asset.shading, a.shading);
    assert_eq!(
        &f.asset.geometry.material_coordinates.as_ref().unwrap()[..8],
        &a.geometry.material_coordinates.as_ref().unwrap()[..8]
    );
    for i in 8..16 {
        assert_eq!(
            f.asset.geometry.material_coordinates.as_ref().unwrap()[i],
            a.geometry.positions[i]
        );
    }
    let mut reversed = p.clone();
    reversed.pairs.reverse();
    assert_eq!(
        serde_json::to_value(&f).unwrap(),
        serde_json::to_value(finish::finish(&a, &reversed, &o).unwrap()).unwrap()
    );
}
#[test]
fn affine_rotation_scale_shear_offset_and_final_geometry_are_respected() {
    let r = Affine {
        columns: [[2., 0., 0.5], [0.25, 3., 0.], [0., 0., 1.]],
        offset: [2., -1., 3.],
    };
    let (a, p, o) = fixture(r.clone());
    let f = finish::finish(&a, &p, &o).unwrap();
    assert_eq!(f.report.iterations, 0);
    assert_eq!(f.report.changed_interior, 0);
    for i in 8..16 {
        near(
            f.asset.geometry.material_coordinates.as_ref().unwrap()[i],
            at(&r, a.geometry.positions[i]),
        );
    }
    let (mut a, p, mut o) = fixture(reference());
    for q in &mut Arc::make_mut(&mut a.geometry).positions {
        q[0] *= 2.;
        q[1] *= 3.;
    }
    o.reference.columns = [[0.5, 0., 0.], [0., 1. / 3., 0.], [0., 0., 1.]];
    let f = finish::finish(&a, &p, &o).unwrap();
    near(
        f.asset.geometry.material_coordinates.as_ref().unwrap()[12],
        [0.1, 0., 0.],
    );
    assert_eq!(f.asset.geometry.positions, a.geometry.positions);
}
#[test]
fn different_authored_frames_stay_fixed_and_actual_residual_is_bounded() {
    let (mut a, p, o) = fixture(reference());
    let material = Arc::make_mut(&mut a.geometry)
        .material_coordinates
        .as_mut()
        .unwrap();
    for i in [4, 5, 6, 7, 14, 15] {
        material[i][0] += 3.;
        material[i][1] += 0.5;
    }
    let before = a.clone();
    let f = finish::finish(&a, &p, &o).unwrap();
    assert!(f.report.iterations > 0 && f.report.iterations <= 64);
    assert!(f.report.residual_max <= 1e-5);
    assert!(f.report.changed_interior > 0);
    assert!(f.report.max_change > 0.);
    assert_eq!(
        &f.asset.geometry.material_coordinates.as_ref().unwrap()[..8],
        &before.geometry.material_coordinates.as_ref().unwrap()[..8]
    );
    near(
        f.asset.geometry.material_coordinates.as_ref().unwrap()[14],
        [3.9, 0.5, 0.],
    );
    assert_eq!(a, before);
    let mut low = o.clone();
    low.iterations = 1;
    low.tolerance = 1e-8;
    assert!(
        finish::finish(&a, &p, &low)
            .unwrap_err()
            .message
            .contains("residual")
    );
}
#[test]
fn constant_or_rank_one_source_fields_do_not_gain_invented_directions() {
    for rank in 0..2 {
        let (mut a, p, o) = fixture(reference());
        for m in Arc::make_mut(&mut a.geometry)
            .material_coordinates
            .as_mut()
            .unwrap()
        {
            *m = if rank == 0 {
                [2., 2., 2.]
            } else {
                [0., m[1], 0.]
            };
        }
        let before = a.clone();
        let error = finish::finish(&a, &p, &o).unwrap_err();
        assert!(error.message.contains("source face") && error.message.contains("singular"));
        assert_eq!(a, before);
    }
}
#[test]
fn invalid_plan_options_envelope_and_quality_fail_without_mutation() {
    let (a, p, o) = fixture(reference());
    let before = a.clone();
    for case in 0..14 {
        let mut p = p.clone();
        let mut o = o.clone();
        match case {
            0 => p.pairs.pop().map(|_| ()).unwrap(),
            1 => p.pairs[1] = p.pairs[0],
            2 => p.pairs[0][0] = 99,
            3 => p.pairs[0][1] = 9,
            4 => p.retained_faces = 5,
            5 => p.retained_vertices = 7,
            6 => p.iso_vertices = 999,
            7 => o.iterations = 0,
            8 => o.iterations = 65,
            9 => o.reference.offset[0] = f32::NAN,
            10 => o.max_change = 1e-5,
            11 => o.max_anisotropy = 0.9,
            12 => o.tolerance = 0.,
            _ => p.pairs[0][0] = 0, // Boundary member, but not joined to this guard.
        }
        assert!(finish::finish(&a, &p, &o).is_err(), "case{case}");
        assert_eq!(a, before);
    }
    let mut missing = a.clone();
    Arc::make_mut(&mut missing.geometry).material_coordinates = None;
    assert!(finish::finish(&missing, &p, &o).is_err());
}
#[test]
fn a_free_or_unused_interior_component_is_rejected() {
    let (mut a, mut p, o) = fixture(reference());
    let g = Arc::make_mut(&mut a.geometry);
    g.positions.insert(12, [3., 3., 3.]);
    g.material_coordinates
        .as_mut()
        .unwrap()
        .insert(12, [3., 3., 3.]);
    for t in &mut g.triangles {
        for i in t {
            if *i >= 12 {
                *i += 1;
            }
        }
    }
    for pair in &mut p.pairs {
        pair[1] += 1;
    }
    p.iso_vertices += 1;
    let rig = Arc::make_mut(a.rig.as_mut().unwrap());
    rig.weights.insert(12, rig.weights[0].clone());
    ignis::meshes::Document {
        objects: Default::default(),
        assets: vec![a.clone()],
        instances: vec![],
    }
    .validate()
    .unwrap();
    assert!(
        finish::finish(&a, &p, &o)
            .unwrap_err()
            .message
            .contains("no Dirichlet anchor")
    );
}

#[test]
fn final_quality_and_numeric_guards_refuse_instead_of_fabricating_a_domain() {
    let r = Affine {
        columns: [[2., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        offset: [0.; 3],
    };
    let (a, p, mut o) = fixture(r);
    o.max_anisotropy = 1.1;
    assert!(
        finish::finish(&a, &p, &o)
            .unwrap_err()
            .message
            .contains("anisotropy cap")
    );
    let (mut a, p, o) = fixture(reference());
    for i in [12, 13] {
        Arc::make_mut(&mut a.geometry).positions[i][0] = 0.;
        Arc::make_mut(&mut a.geometry).positions[i][2] = 0.1;
    }
    let before = a.clone();
    assert!(
        finish::finish(&a, &p, &o)
            .unwrap_err()
            .message
            .contains("guard still has")
    );
    assert_eq!(a, before);
    let (mut a, p, o) = fixture(reference());
    for q in &mut Arc::make_mut(&mut a.geometry).positions {
        q[1] *= 1e-16;
    }
    assert!(
        finish::finish(&a, &p, &o)
            .unwrap_err()
            .message
            .contains("ill-conditioned")
    );
}
