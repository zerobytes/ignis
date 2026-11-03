//! Revision-scoped component references and bounded conforming linear refinement.
use crate::{
    meshes,
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub scope: String,
    pub revision: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Components {
    Vertices { indices: Vec<u32> },
    Edges { vertices: Vec<[u32; 2]> },
    Faces { indices: Vec<u32> },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub asset: String,
    pub reference: Reference,
    pub components: Components,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Edge {
    pub vertices: [u32; 2],
    pub faces: Vec<u32>,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Topology {
    pub vertices: usize,
    /// Sorted lexicographically; an edge is identified by its endpoint indices.
    pub edges: Vec<Edge>,
    /// Face IDs are indices into this array, scoped to the inspected revision.
    pub faces: Vec<[u32; 3]>,
    pub boundary_edges: usize,
    pub non_manifold_edges: usize,
    pub isolated_vertices: Vec<u32>,
}
fn edge(a: u32, b: u32) -> [u32; 2] {
    if a < b { [a, b] } else { [b, a] }
}
/// Material correspondence into the four ordered children produced by full refinement.
/// Boundary ties are deterministic; the caller supplies parent barycentric coordinates.
pub fn uniform_child(bary: [f32; 3]) -> (u32, [f32; 3]) {
    if bary[0] >= 0.5 {
        (0, [2. * bary[0] - 1., 2. * bary[1], 2. * bary[2]])
    } else if bary[1] >= 0.5 {
        (1, [2. * bary[0], 2. * bary[1] - 1., 2. * bary[2]])
    } else if bary[2] >= 0.5 {
        (2, [2. * bary[0], 2. * bary[1], 2. * bary[2] - 1.])
    } else {
        (3, [1. - 2. * bary[2], 1. - 2. * bary[0], 1. - 2. * bary[1]])
    }
}
impl Topology {
    pub fn inspect(geometry: &meshes::Geometry) -> Result<Self> {
        geometry.validate()?;
        let mut edges: BTreeMap<[u32; 2], Vec<u32>> = BTreeMap::new();
        let mut used = vec![false; geometry.positions.len()];
        for (index, &[a, b, c]) in geometry.triangles.iter().enumerate() {
            for i in [a, b, c] {
                used[i as usize] = true;
            }
            for key in [edge(a, b), edge(b, c), edge(c, a)] {
                edges.entry(key).or_default().push(index as u32);
            }
        }
        let edges: Vec<_> = edges
            .into_iter()
            .map(|(vertices, faces)| Edge { vertices, faces })
            .collect();
        Ok(Self {
            vertices: geometry.positions.len(),
            faces: geometry.triangles.clone(),
            boundary_edges: edges.iter().filter(|e| e.faces.len() == 1).count(),
            non_manifold_edges: edges.iter().filter(|e| e.faces.len() > 2).count(),
            isolated_vertices: used
                .iter()
                .enumerate()
                .filter_map(|(i, used)| (!used).then_some(i as u32))
                .collect(),
            edges,
        })
    }
    pub fn canonicalize(&self, components: &Components) -> Result<Components> {
        fn indices(values: &[u32], limit: usize) -> Result<Vec<u32>> {
            if values.is_empty() || values.len() > limit {
                return Err(invalid(
                    "Mesh selection must be nonempty within component budget",
                ));
            }
            let mut out = values.to_vec();
            out.sort_unstable();
            if out.last().is_some_and(|i| *i as usize >= limit)
                || out.windows(2).any(|w| w[0] == w[1])
            {
                return Err(invalid(
                    "Mesh selection has missing or duplicate components",
                ));
            }
            Ok(out)
        }
        Ok(match components {
            Components::Vertices { indices: values } => Components::Vertices {
                indices: indices(values, self.vertices)?,
            },
            Components::Faces { indices: values } => Components::Faces {
                indices: indices(values, self.faces.len())?,
            },
            Components::Edges { vertices } => {
                if vertices.is_empty() || vertices.len() > self.edges.len() {
                    return Err(invalid(
                        "Mesh edge selection must be nonempty within edge budget",
                    ));
                }
                let mut vertices: Vec<_> = vertices.iter().map(|&[a, b]| edge(a, b)).collect();
                vertices.sort_unstable();
                if vertices.windows(2).any(|w| w[0] == w[1])
                    || vertices
                        .iter()
                        .any(|e| self.edges.binary_search_by_key(e, |e| e.vertices).is_err())
                {
                    return Err(invalid("Mesh selection has missing or duplicate edges"));
                }
                Components::Edges { vertices }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Plan {
    split_edges: Vec<[u32; 2]>,
    triangles: Vec<[u32; 3]>,
    /// Parent face for every output face, in output order.
    parents: Vec<u32>,
    source_vertices: usize,
    source_triangles: Vec<[u32; 3]>,
}
impl Plan {
    pub fn split_edges(&self) -> &[[u32; 2]] {
        &self.split_edges
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }
    pub fn parents(&self) -> &[u32] {
        &self.parents
    }
    pub fn new(geometry: &meshes::Geometry, components: Option<&Components>) -> Result<Self> {
        let topology = Topology::inspect(geometry)?;
        let mut marked = BTreeSet::new();
        match components.map(|c| topology.canonicalize(c)).transpose()? {
            None => marked.extend(topology.edges.iter().map(|e| e.vertices)),
            Some(Components::Edges { vertices }) => marked.extend(vertices),
            Some(Components::Faces { indices }) => {
                for i in indices {
                    let [a, b, c] = topology.faces[i as usize];
                    marked.extend([edge(a, b), edge(b, c), edge(c, a)]);
                }
            }
            Some(Components::Vertices { indices }) => {
                for e in &topology.edges {
                    if e.vertices.iter().any(|i| indices.binary_search(i).is_ok()) {
                        marked.insert(e.vertices);
                    }
                }
            }
        }
        if marked.is_empty() {
            return Err(invalid("Mesh selection has no incident edges to subdivide"));
        }
        let vertices = geometry.positions.len() + marked.len();
        let triangles: usize = topology
            .edges
            .iter()
            .filter(|e| marked.contains(&e.vertices))
            .map(|e| e.faces.len())
            .sum::<usize>()
            + geometry.triangles.len();
        if vertices > meshes::MAX_VERTICES || triangles > meshes::MAX_TRIANGLES {
            return Err(invalid("Subdivision exceeds mesh vertex/triangle budget"));
        }
        let split_edges: Vec<_> = marked.into_iter().collect();
        let midpoint = |a, b| {
            split_edges
                .binary_search(&edge(a, b))
                .ok()
                .map(|i| (geometry.positions.len() + i) as u32)
        };
        let mut out = Vec::with_capacity(triangles);
        let mut parents = Vec::with_capacity(triangles);
        for (parent, triangle) in geometry.triangles.iter().enumerate() {
            let [a, b, c] = *triangle;
            let mids = [midpoint(a, b), midpoint(b, c), midpoint(c, a)];
            let mask = mids
                .iter()
                .enumerate()
                .fold(0, |m, (i, v)| m | ((v.is_some() as u8) << i));
            let start = out.len();
            match mask {
                0 => out.push(*triangle),
                7 => {
                    let [ab, bc, ca] = mids.map(|m| m.unwrap());
                    out.extend([[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]]);
                }
                1 | 2 | 4 => {
                    let r = if mask == 1 {
                        0
                    } else if mask == 2 {
                        1
                    } else {
                        2
                    };
                    let [a, b, c] = std::array::from_fn(|i| triangle[(r + i) % 3]);
                    let m = mids[r].unwrap();
                    out.extend([[a, m, c], [m, b, c]]);
                }
                _ => {
                    let r = if mask == 3 {
                        0
                    } else if mask == 6 {
                        1
                    } else {
                        2
                    };
                    let [a, b, c] = std::array::from_fn(|i| triangle[(r + i) % 3]);
                    let ab = mids[r].unwrap();
                    let bc = mids[(r + 1) % 3].unwrap();
                    out.extend([[b, bc, ab], [a, ab, c], [ab, bc, c]]);
                }
            }
            parents.resize(parents.len() + out.len() - start, parent as u32);
        }
        Ok(Self {
            split_edges,
            triangles: out,
            parents,
            source_vertices: geometry.positions.len(),
            source_triangles: topology.faces,
        })
    }
    pub fn vertex_count(&self) -> usize {
        self.source_vertices + self.split_edges.len()
    }
    fn check_source(&self, source: &meshes::Geometry) -> Result<()> {
        if source.positions.len() != self.source_vertices
            || source.triangles != self.source_triangles
        {
            return Err(invalid("Subdivision plan source topology changed"));
        }
        Ok(())
    }
    pub fn geometry(&self, source: &meshes::Geometry) -> Result<meshes::Geometry> {
        self.check_source(source)?;
        source.validate()?;
        let mut positions = Vec::with_capacity(self.vertex_count());
        positions.extend_from_slice(&source.positions);
        for &[a, b] in &self.split_edges {
            positions.push(std::array::from_fn(|i| {
                ((f64::from(source.positions[a as usize][i])
                    + f64::from(source.positions[b as usize][i]))
                    * 0.5) as f32
            }));
        }
        let out = meshes::Geometry {
            material_coordinates: source.material_coordinates.as_ref().map(|source| {
                let mut coordinates = Vec::with_capacity(self.vertex_count());
                coordinates.extend_from_slice(source);
                for &[a, b] in &self.split_edges {
                    coordinates.push(std::array::from_fn(|i| {
                        ((f64::from(source[a as usize][i]) + f64::from(source[b as usize][i]))
                            * 0.5) as f32
                    }));
                }
                coordinates
            }),
            positions,
            triangles: self.triangles.clone(),
        };
        out.validate()?;
        Ok(out)
    }
    pub fn asset(&self, source: &meshes::Asset) -> Result<meshes::Asset> {
        // Reject a reused plan before its indices can reach rig weights.
        self.check_source(&source.geometry)?;
        meshes::Document {
            objects: Default::default(),
            assets: vec![source.clone()],
            instances: vec![],
        }
        .validate()?;
        let rig = if let Some(rig) = &source.rig {
            let mut target = (**rig).clone();
            target.weights.reserve(self.split_edges.len());
            for &[a, b] in &self.split_edges {
                target
                    .weights
                    .push(crate::mesh_preservation::transfer_weight(
                        rig,
                        [a as usize, b as usize, a as usize],
                        [0.5, 0.5, 0.],
                    )?);
            }
            Some(Arc::new(target))
        } else {
            None
        };
        let result = meshes::Asset {
            id: source.id.clone(),
            geometry: Arc::new(self.geometry(&source.geometry)?),
            shading: source.shading,
            material: source.material.clone(),
            rig,
            clips: source.clips.clone(),
        };
        meshes::Document {
            objects: Default::default(),
            assets: vec![result.clone()],
            instances: vec![],
        }
        .validate()?;
        Ok(result)
    }
}
