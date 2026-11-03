//! Loose parts are connected by shared vertex indices, never spatial proximity.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IslandTransform {
    pub pivot: [f32; 3],
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMode {
    #[default]
    Move,
    Copy,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Island {
    /// Minimum original face index, scoped to the document inspection revision.
    pub id: u32,
    pub vertices: usize,
    pub triangles: usize,
    pub bounds: [[f32; 3]; 2],
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Islands {
    pub islands: Vec<Island>,
    pub isolated_vertices: Vec<u32>,
}
struct Partition {
    summary: Islands,
    vertices: Vec<Vec<u32>>,
    faces: Vec<Vec<u32>>,
}
impl Partition {
    fn new(geometry: &Geometry) -> Result<Self> {
        geometry.validate()?;
        let n = geometry.positions.len();
        let mut parents: Vec<usize> = (0..n).collect();
        fn root(parents: &mut [usize], mut i: usize) -> usize {
            while parents[i] != i {
                parents[i] = parents[parents[i]];
                i = parents[i];
            }
            i
        }
        for &[a, b, c] in &geometry.triangles {
            for (x, y) in [(a, b), (a, c)] {
                let x = root(&mut parents, x as usize);
                let y = root(&mut parents, y as usize);
                parents[x.max(y)] = x.min(y);
            }
        }
        let mut labels = vec![usize::MAX; n];
        let mut out = Self {
            summary: Islands {
                islands: vec![],
                isolated_vertices: vec![],
            },
            vertices: vec![],
            faces: vec![],
        };
        for (face, triangle) in geometry.triangles.iter().enumerate() {
            let r = root(&mut parents, triangle[0] as usize);
            if labels[r] == usize::MAX {
                labels[r] = out.faces.len();
                out.faces.push(vec![]);
                out.vertices.push(vec![]);
                out.summary.islands.push(Island {
                    id: face as u32,
                    vertices: 0,
                    triangles: 0,
                    bounds: [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]],
                });
            }
            out.faces[labels[r]].push(face as u32);
        }
        for (index, p) in geometry.positions.iter().enumerate() {
            let label = labels[root(&mut parents, index)];
            if label == usize::MAX {
                out.summary.isolated_vertices.push(index as u32);
                continue;
            }
            out.vertices[label].push(index as u32);
            let summary = &mut out.summary.islands[label];
            for (axis, &v) in p.iter().enumerate() {
                summary.bounds[0][axis] = summary.bounds[0][axis].min(v);
                summary.bounds[1][axis] = summary.bounds[1][axis].max(v);
            }
        }
        for (i, s) in out.summary.islands.iter_mut().enumerate() {
            s.vertices = out.vertices[i].len();
            s.triangles = out.faces[i].len();
        }
        Ok(out)
    }
}

fn subset(source: &Asset, id: &str, vertices: &[u32], faces: &[u32]) -> Asset {
    let mut indices = vec![u32::MAX; source.geometry.positions.len()];
    for (new, &old) in vertices.iter().enumerate() {
        indices[old as usize] = new as u32;
    }
    let geometry = Geometry {
        positions: vertices
            .iter()
            .map(|&v| source.geometry.positions[v as usize])
            .collect(),
        triangles: faces
            .iter()
            .map(|&f| source.geometry.triangles[f as usize].map(|v| indices[v as usize]))
            .collect(),
        material_coordinates: source
            .geometry
            .material_coordinates
            .as_ref()
            .map(|coords| vertices.iter().map(|&v| coords[v as usize]).collect()),
    };
    Asset {
        id: id.into(),
        geometry: Arc::new(geometry),
        shading: source.shading,
        material: source.material.clone(),
        rig: source.rig.as_ref().map(|rig| {
            Arc::new(Rig {
                bones: rig.bones.clone(),
                weights: vertices
                    .iter()
                    .map(|&v| rig.weights[v as usize].clone())
                    .collect(),
            })
        }),
        clips: source.clips.clone(),
    }
}

impl Document {
    pub(super) fn transform_islands(
        &mut self,
        asset: &str,
        selected: &[u32],
        transform: &IslandTransform,
    ) -> Result<()> {
        finite(&transform.pivot)?;
        finite(&transform.translation)?;
        quaternion(transform.rotation)?;
        if !transform.scale.is_finite() || !(0.001..=100.).contains(&transform.scale) {
            return Err(invalid(
                "Island transform requires positive uniform scale0.001..100",
            ));
        }
        let source = &self.asset(asset)?.geometry;
        let partition = Partition::new(source)?;
        if selected.is_empty() || selected.len() > partition.summary.islands.len() {
            return Err(invalid(
                "Transform requires nonempty island selection within budget",
            ));
        }
        let mut selected = selected.to_vec();
        selected.sort_unstable();
        if selected.windows(2).any(|w| w[0] == w[1]) {
            return Err(invalid("Duplicate transform island"));
        }
        let mut marked = vec![false; source.positions.len()];
        for id in selected {
            let index = partition
                .summary
                .islands
                .binary_search_by_key(&id, |s| s.id)
                .map_err(|_| invalid("Transform island not found; inspect again"))?;
            for &v in &partition.vertices[index] {
                marked[v as usize] = true;
            }
        }
        self.transform_marked(asset, &marked, transform)
    }
    pub(super) fn transform_vertices(
        &mut self,
        asset: &str,
        vertices: &[u32],
        transform: &IslandTransform,
    ) -> Result<()> {
        let source = &self.asset(asset)?.geometry;
        if vertices.is_empty() || vertices.len() > source.positions.len() {
            return Err(invalid("Select1..vertex_count vertices"));
        }
        let mut marked = vec![false; source.positions.len()];
        for &i in vertices {
            let p = marked
                .get_mut(i as usize)
                .ok_or_else(|| invalid("Vertex index outside mesh"))?;
            if *p {
                return Err(invalid("Duplicate vertex index"));
            }
            *p = true;
        }
        self.transform_marked(asset, &marked, transform)
    }
    fn transform_marked(
        &mut self,
        asset: &str,
        marked: &[bool],
        transform: &IslandTransform,
    ) -> Result<()> {
        use glam::{DQuat, DVec3};
        finite(&transform.pivot)?;
        finite(&transform.translation)?;
        quaternion(transform.rotation)?;
        if !transform.scale.is_finite() || !(0.001..=100.).contains(&transform.scale) {
            return Err(invalid("Transform scale requires0.001..100"));
        }
        let source = &self.asset(asset)?.geometry;
        let pivot = DVec3::from_array(transform.pivot.map(f64::from));
        let translation = DVec3::from_array(transform.translation.map(f64::from));
        let rotation = DQuat::from_array(transform.rotation.map(f64::from)).normalize();
        let mut positions = source.positions.clone();
        for (index, p) in positions.iter_mut().enumerate() {
            if marked[index] {
                let old = DVec3::from_array(p.map(f64::from));
                *p =
                    (pivot + translation + rotation * ((old - pivot) * f64::from(transform.scale)))
                        .as_vec3()
                        .to_array();
                finite(p)?;
            }
        }
        for triangle in &source.triangles {
            if !triangle.iter().any(|i| marked[*i as usize]) {
                continue;
            }
            let normal = |points: &[[f32; 3]]| {
                let [a, b, c] =
                    triangle.map(|i| DVec3::from_array(points[i as usize].map(f64::from)));
                (b - a).cross(c - a)
            };
            let expected = if triangle.iter().all(|i| marked[*i as usize]) {
                rotation * normal(&source.positions)
            } else {
                normal(&source.positions)
            };
            if normal(&positions).dot(expected) <= 0. {
                return Err(invalid(
                    "Island transform collapses or reverses a triangle at f32 precision",
                ));
            }
        }
        let geometry = Geometry {
            positions,
            triangles: source.triangles.clone(),
            material_coordinates: Some(
                source
                    .material_coordinates
                    .as_ref()
                    .unwrap_or(&source.positions)
                    .clone(),
            ),
        };
        geometry.validate()?;
        self.asset_mut(asset)?.geometry = Arc::new(geometry);
        Ok(())
    }

    pub fn islands(&self, asset: &str) -> Result<Islands> {
        Ok(Partition::new(&self.asset(asset)?.geometry)?.summary)
    }
    pub(super) fn extract(
        &mut self,
        asset: &str,
        destination: &str,
        selected: &[u32],
        mode: ExtractionMode,
    ) -> Result<()> {
        id(destination)?;
        if self.assets.iter().any(|a| a.id == destination) {
            return Err(invalid("Extraction destination asset already exists"));
        }
        let source = self.asset(asset)?;
        let partition = Partition::new(&source.geometry)?;
        if selected.is_empty() || selected.len() > partition.summary.islands.len() {
            return Err(invalid(
                "Extraction requires nonempty island selection within budget",
            ));
        }
        let mut selected = selected.to_vec();
        selected.sort_unstable();
        if selected.windows(2).any(|s| s[0] == s[1]) {
            return Err(invalid("Duplicate extraction island"));
        }
        let mut marked_vertices = vec![false; source.geometry.positions.len()];
        let mut marked_faces = vec![false; source.geometry.triangles.len()];
        for id in selected {
            let index = partition
                .summary
                .islands
                .binary_search_by_key(&id, |s| s.id)
                .map_err(|_| invalid("Extraction island not found; inspect again"))?;
            for &v in &partition.vertices[index] {
                marked_vertices[v as usize] = true;
            }
            for &f in &partition.faces[index] {
                marked_faces[f as usize] = true;
            }
        }
        let split = |marked: &[bool], take: bool| -> Vec<u32> {
            marked
                .iter()
                .enumerate()
                .filter_map(|(i, &value)| (value == take).then_some(i as u32))
                .collect()
        };
        let vertices = split(&marked_vertices, true);
        let faces = split(&marked_faces, true);
        if mode == ExtractionMode::Move && faces.len() == source.geometry.triangles.len() {
            return Err(invalid(
                "Extraction Move must leave at least one source face; use Copy or explicit asset removal",
            ));
        }
        // Payloads/weights are copied only after exact global storage/clip preflight.
        let clips: usize = self.assets.iter().map(|a| a.clips.len()).sum();
        let keys = |a: &Asset| {
            a.clips
                .iter()
                .flat_map(|c| &c.tracks)
                .map(|t| t.keys.len())
                .sum::<usize>()
        };
        let total_k: usize = self.assets.iter().map(keys).sum();
        let copy = mode == ExtractionMode::Copy;
        if clips + source.clips.len() > MAX_CLIPS || total_k + keys(source) > MAX_KEYS {
            return Err(invalid("Extraction exceeds mesh storage/clip/key budget"));
        }
        let extracted = subset(source, destination, &vertices, &faces);
        let remaining = if copy {
            None
        } else {
            Some(subset(
                source,
                asset,
                &split(&marked_vertices, false),
                &split(&marked_faces, false),
            ))
        };
        if let Some(remaining) = remaining {
            *self.asset_mut(asset)? = remaining;
        }
        self.assets.push(extracted);
        Ok(())
    }
}

impl Document {
    pub(super) fn fit_material_coordinates(
        &mut self,
        asset: &str,
        per_island: bool,
        bounds: [[f32; 3]; 2],
    ) -> Result<()> {
        if bounds
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 10000.)
            || (0..3).any(|axis| bounds[0][axis] > bounds[1][axis])
        {
            return Err(invalid(
                "Material fit requires finite ordered bounds within±10000",
            ));
        }
        let source = &self.asset(asset)?.geometry;
        let input = source
            .material_coordinates
            .as_ref()
            .unwrap_or(&source.positions);
        let groups = if per_island {
            Partition::new(source)?.vertices
        } else {
            vec![(0..input.len() as u32).collect()]
        };
        // Unreferenced vertices need no individual allocation or AABB pass.
        let centre = std::array::from_fn(|axis| {
            ((f64::from(bounds[0][axis]) + f64::from(bounds[1][axis])) * 0.5) as f32
        });
        let mut result = vec![centre; input.len()];
        for group in groups {
            let mut low = [f64::INFINITY; 3];
            let mut high = [f64::NEG_INFINITY; 3];
            for &v in &group {
                for axis in 0..3 {
                    low[axis] = low[axis].min(f64::from(input[v as usize][axis]));
                    high[axis] = high[axis].max(f64::from(input[v as usize][axis]));
                }
            }
            for &v in &group {
                result[v as usize] = std::array::from_fn(|axis| {
                    let t = if high[axis] == low[axis] {
                        0.5
                    } else {
                        (f64::from(input[v as usize][axis]) - low[axis]) / (high[axis] - low[axis])
                    };
                    (f64::from(bounds[0][axis])
                        + t * (f64::from(bounds[1][axis]) - f64::from(bounds[0][axis])))
                        as f32
                });
            }
        }
        Arc::make_mut(&mut self.asset_mut(asset)?.geometry).material_coordinates = Some(result);
        Ok(())
    }
}

impl Document {
    pub(super) fn copy_material_coordinates(
        &mut self,
        asset: &str,
        source: &str,
        per_island: bool,
    ) -> Result<()> {
        let template = &self.asset(source)?.geometry;
        let coordinates = template
            .material_coordinates
            .as_ref()
            .ok_or_else(|| invalid("Material coordinate template needs an explicit channel"))?;
        let target = &self.asset(asset)?.geometry;
        let groups = if per_island {
            let origin = Partition::new(template)?;
            let destination = Partition::new(target)?;
            if origin.faces.len() != 1
                || !origin.summary.isolated_vertices.is_empty()
                || !destination.summary.isolated_vertices.is_empty()
            {
                return Err(invalid(
                    "Per-island coordinate copy requires one template island and no isolated vertices",
                ));
            }
            let mut remap = vec![u32::MAX; target.positions.len()];
            for (vertices, faces) in destination.vertices.iter().zip(&destination.faces) {
                if vertices.len() != template.positions.len()
                    || faces.len() != template.triangles.len()
                {
                    return Err(invalid("Material coordinate template topology mismatch"));
                }
                for (local, &vertex) in vertices.iter().enumerate() {
                    remap[vertex as usize] = local as u32;
                }
                if faces
                    .iter()
                    .zip(&template.triangles)
                    .any(|(&face, triangle)| {
                        target.triangles[face as usize].map(|v| remap[v as usize]) != *triangle
                    })
                {
                    return Err(invalid(
                        "Material coordinate template triangle order mismatch",
                    ));
                }
            }
            destination.vertices
        } else {
            if target.positions.len() != template.positions.len()
                || target.triangles != template.triangles
            {
                return Err(invalid("Material coordinate template topology mismatch"));
            }
            vec![(0..target.positions.len() as u32).collect()]
        };
        // All correspondence checks precede allocation/copy of the new channel.
        let mut result = vec![[0.; 3]; target.positions.len()];
        for vertices in groups {
            for (local, vertex) in vertices.into_iter().enumerate() {
                result[vertex as usize] = coordinates[local];
            }
        }
        Arc::make_mut(&mut self.asset_mut(asset)?.geometry).material_coordinates = Some(result);
        Ok(())
    }
}
