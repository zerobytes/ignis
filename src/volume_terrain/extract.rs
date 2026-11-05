use super::*;
const CORNERS: [[usize; 3]; 8] = [
    [0, 0, 0],
    [1, 0, 0],
    [1, 1, 0],
    [0, 1, 0],
    [0, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
    [0, 1, 1],
];
const EDGES: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

pub(super) fn compile(doc: &Document, grid: &Grid) -> Result<Compiled> {
    let sampler = Sampler::new(doc)?;
    let (n, count) = grid.dimensions()?;
    // Integer lattice plus shared origin; identical boundary coordinates in neighbouring
    // grids require callers to use the same origin/spacing lattice (no cumulative steps).
    let world = |p: [usize; 3]| {
        DVec3::from_array(std::array::from_fn(|i| {
            grid.origin[i] + p[i] as f64 * grid.step(i)
        }))
    };
    let mut values = Vec::with_capacity(count);
    for z in 0..n[2] {
        for y in 0..n[1] {
            for x in 0..n[0] {
                values.push(sampler.value(world([x, y, z])));
            }
        }
    }
    extract_values(doc, grid, n, values, "cpu_reference")
}
pub(super) fn extract_values(
    _doc: &Document,
    grid: &Grid,
    n: [usize; 3],
    values: Vec<f64>,
    generation: &str,
) -> Result<Compiled> {
    let count = n.iter().product::<usize>();
    if values.len() != count || values.iter().any(|v| !v.is_finite()) {
        return Err(invalid("Invalid density lattice"));
    }
    let index = |p: [usize; 3]| (p[2] * n[1] + p[1]) * n[0] + p[0];
    let mut edge_indices = vec![[u32::MAX; 3]; count];
    let mut corner_indices = vec![u32::MAX; count];
    let mut positions = Vec::<[f32; 3]>::new();
    let mut triangles = Vec::<[u32; 3]>::new();
    let mut active = 0usize;
    let mut degenerate = 0usize;
    for z in 0..n[2] - 1 {
        for y in 0..n[1] - 1 {
            for x in 0..n[0] - 1 {
                let cell = [x, y, z];
                let corners = CORNERS.map(|p| std::array::from_fn(|i| cell[i] + p[i]));
                let ids = corners.map(index);
                let ds = ids.map(|i| values[i]);
                let case = ds
                    .iter()
                    .enumerate()
                    .fold(0usize, |a, (i, &d)| a | ((d > 0.) as usize) << i);
                if case == 0 || case == 255 {
                    continue;
                }
                active += 1;
                let row = tables::TRIANGLES[case];
                let mut local = [u32::MAX; 12];
                for &edge in row.iter().take_while(|&&e| e >= 0) {
                    let edge = edge as usize;
                    if local[edge] != u32::MAX {
                        continue;
                    }
                    let [a, b] = EDGES[edge];
                    let axis = (0..3).find(|&i| corners[a][i] != corners[b][i]).unwrap();
                    let lower = std::array::from_fn(|i| corners[a][i].min(corners[b][i]));
                    let key = index(lower);
                    let mut vertex = edge_indices[key][axis];
                    if vertex == u32::MAX {
                        let t = ds[a] / (ds[a] - ds[b]);
                        let endpoint = if t == 0. {
                            Some(ids[a])
                        } else if t == 1. {
                            Some(ids[b])
                        } else {
                            None
                        };
                        vertex = endpoint.map_or(u32::MAX, |c| corner_indices[c]);
                        if vertex == u32::MAX {
                            if positions.len() >= MAX_VERTICES {
                                return Err(invalid(
                                    "Density mesh exceeds600000vertex budget; reduce grid or split publication",
                                ));
                            }
                            let pa = DVec3::from_array(std::array::from_fn(|i| {
                                corners[a][i] as f64 * grid.step(i)
                            }));
                            let pb = DVec3::from_array(std::array::from_fn(|i| {
                                corners[b][i] as f64 * grid.step(i)
                            }));
                            let p = pa + (pb - pa) * t;
                            vertex = positions.len() as u32;
                            positions.push(p.to_array().map(|v| v as f32));
                            if let Some(c) = endpoint {
                                corner_indices[c] = vertex;
                            }
                        }
                        edge_indices[key][axis] = vertex;
                    }
                    local[edge] = vertex;
                }
                for triple in row.chunks_exact(3) {
                    if triple[0] < 0 {
                        break;
                    }
                    let t = [
                        local[triple[0] as usize],
                        local[triple[2] as usize],
                        local[triple[1] as usize],
                    ];
                    let p = t.map(|i| glam::Vec3::from_array(positions[i as usize]));
                    if t[0] == t[1]
                        || t[1] == t[2]
                        || t[0] == t[2]
                        || (p[1] - p[0]).cross(p[2] - p[0]).length_squared() == 0.
                    {
                        degenerate += 1;
                        continue;
                    }
                    if triangles.len() >= MAX_TRIANGLES {
                        return Err(invalid("Density mesh exceeds1200000triangle budget"));
                    }
                    triangles.push(t);
                }
            }
        }
    }
    let report = serde_json::json!({"vertices":positions.len(),"triangles":triangles.len(),"samples":count,"active_cells":active,"degenerate_triangles_omitted":degenerate,"empty":triangles.is_empty(),"indexed":true,"volumetric":true,"generation":generation,"extraction":"marching_cubes_classic","grid":grid,"streaming":false,"gpu_generation":generation=="gpu_density_cpu_extraction","boundary":"open_at_grid_boundary","ambiguity_resolution":"classic_table_no_asymptotic_decider"});
    let geometry = crate::meshes::Geometry {
        positions,
        triangles,
        material_coordinates: None,
    };
    if !geometry.triangles.is_empty() {
        geometry.validate()?;
    }
    Ok(Compiled {
        geometry: Arc::new(geometry),
        report,
    })
}
