//! One Catmull-Clark step on the authored polygon cage, after shape evaluation.
//! Original vertex slots stay first so authoring anchors refer to the drawn surface.
use super::*;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Refinement {
    pub part: String,
    pub faces: Vec<Vec<u32>>,
    #[serde(skip)]
    cache: Arc<Mutex<Option<Arc<Plan>>>>,
}
impl PartialEq for Refinement {
    fn eq(&self, other: &Self) -> bool {
        self.part == other.part && self.faces == other.faces
    }
}
impl Refinement {
    pub fn new(part: String, faces: Vec<Vec<u32>>) -> Self {
        Self {
            part,
            faces,
            cache: Default::default(),
        }
    }
    fn plan(&self, part: &Part) -> Result<Arc<Plan>> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| invalid("Character refinement cache poisoned"))?;
        if let Some(plan) = cache.as_ref()
            && plan.faces == self.faces
            && plan.input_weights == part.weights
            && plan.input_triangles == part.triangles
            && plan.input_count == part.vertices.len()
        {
            return Ok(plan.clone());
        }
        let plan = Arc::new(Plan::build(self, part)?);
        *cache = Some(plan.clone());
        Ok(plan)
    }
}
#[derive(Debug)]
struct Plan {
    faces: Vec<Vec<u32>>,
    input_weights: Vec<Weight>,
    input_triangles: Vec<[u32; 3]>,
    input_count: usize,
    stencils: Vec<Vec<(u32, f64)>>,
    triangles: Vec<[u32; 3]>,
    weights: Vec<Weight>,
}
fn add(out: &mut BTreeMap<u32, f64>, stencil: impl IntoIterator<Item = (u32, f64)>, factor: f64) {
    for (v, w) in stencil {
        *out.entry(v).or_default() += w * factor;
    }
}
impl Plan {
    fn build(refinement: &Refinement, part: &Part) -> Result<Self> {
        let n = part.vertices.len();
        if refinement.faces.is_empty() || refinement.faces.len() > 100_000 || part.socket.is_some()
        {
            return Err(invalid(
                "Refinement requires a bounded deformable polygon cage",
            ));
        }
        let mut edges: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
        let mut direction = BTreeMap::new();
        let mut incident = vec![Vec::new(); n];
        let mut triangles = Vec::new();
        let mut face_stencils = Vec::new();
        for (f, face) in refinement.faces.iter().enumerate() {
            if !(3..=8).contains(&face.len())
                || face.iter().any(|&v| v as usize >= n)
                || face.iter().collect::<BTreeSet<_>>().len() != face.len()
            {
                return Err(invalid("Invalid refinement polygon"));
            }
            for i in 1..face.len() - 1 {
                triangles.push([face[0], face[i], face[i + 1]]);
            }
            for i in 0..face.len() {
                let (a, b) = (face[i], face[(i + 1) % face.len()]);
                let edge = (a.min(b), a.max(b));
                let forward = a < b;
                if direction
                    .insert(edge, forward)
                    .is_some_and(|prior| prior == forward)
                {
                    return Err(invalid("Refinement polygon winding is inconsistent"));
                }
                edges.entry(edge).or_default().push(f);
                incident[a as usize].push(f);
            }
            face_stencils.push(
                face.iter()
                    .map(|&v| (v, 1. / face.len() as f64))
                    .collect::<Vec<_>>(),
            );
        }
        if triangles != part.triangles
            || n + edges.len() + refinement.faces.len() > MAX_VERTICES
            || edges.values().any(|f| f.len() > 2)
        {
            return Err(invalid(
                "Refinement cage differs from part topology, is nonmanifold or exceeds vertex budget",
            ));
        }
        let edge_indices: BTreeMap<_, _> = edges
            .keys()
            .enumerate()
            .map(|(i, &edge)| (edge, (n + i) as u32))
            .collect();
        let mut neighbors = vec![Vec::new(); n];
        let mut boundary = vec![Vec::new(); n];
        for (&(a, b), faces) in &edges {
            neighbors[a as usize].push(b);
            neighbors[b as usize].push(a);
            if faces.len() == 1 {
                boundary[a as usize].push(b);
                boundary[b as usize].push(a);
            }
        }
        let mut stencils = Vec::with_capacity(n + edges.len() + refinement.faces.len());
        for i in 0..n {
            if let Some(&first) = incident[i].first() {
                let mut reached = BTreeSet::from([first]);
                let mut pending = vec![first];
                while let Some(f) = pending.pop() {
                    for &v in &neighbors[i] {
                        let edge = &edges[&((i as u32).min(v), (i as u32).max(v))];
                        if edge.contains(&f) {
                            for &other in edge {
                                if reached.insert(other) {
                                    pending.push(other);
                                }
                            }
                        }
                    }
                }
                if reached.len() != incident[i].len() {
                    return Err(invalid("Refinement vertex has disconnected fans"));
                }
            }
            let mut row = BTreeMap::new();
            if !boundary[i].is_empty() {
                if boundary[i].len() != 2 {
                    return Err(invalid("Refinement has a nonmanifold boundary"));
                }
                row.insert(i as u32, 0.75);
                for &v in &boundary[i] {
                    row.insert(v, 0.125);
                }
            } else if neighbors[i].is_empty() {
                row.insert(i as u32, 1.);
            } else {
                let k = neighbors[i].len();
                if k < 3 || incident[i].len() != k {
                    return Err(invalid("Refinement has a disconnected vertex fan"));
                }
                *row.entry(i as u32).or_default() += (k as f64 - 3.) / k as f64;
                for &f in &incident[i] {
                    add(
                        &mut row,
                        face_stencils[f].iter().copied(),
                        1. / (k * k) as f64,
                    );
                }
                for &v in &neighbors[i] {
                    add(&mut row, [(i as u32, 0.5), (v, 0.5)], 2. / (k * k) as f64);
                }
            }
            stencils.push(row.into_iter().collect());
        }
        for (&(a, b), faces) in &edges {
            let mut row = BTreeMap::new();
            let factor = if faces.len() == 1 { 0.5 } else { 0.25 };
            add(&mut row, [(a, 1.), (b, 1.)], factor);
            if faces.len() == 2 {
                for &f in faces {
                    add(&mut row, face_stencils[f].iter().copied(), 0.25);
                }
            }
            stencils.push(row.into_iter().collect());
        }
        stencils.extend(face_stencils);
        triangles.clear();
        for (f, face) in refinement.faces.iter().enumerate() {
            let center = (n + edges.len() + f) as u32;
            for i in 0..face.len() {
                let v = face[i];
                let next = face[(i + 1) % face.len()];
                let prev = face[(i + face.len() - 1) % face.len()];
                let a = edge_indices[&(v.min(next), v.max(next))];
                let b = edge_indices[&(v.min(prev), v.max(prev))];
                triangles.extend([[v, a, center], [v, center, b]]);
            }
        }
        let mut weights = Vec::with_capacity(stencils.len());
        for row in &stencils {
            let mut sums = [0.; super::super::MAX_BONES];
            for &(v, factor) in row {
                for (joint, weight) in part.weights[v as usize].influences() {
                    sums[joint as usize] += weight as f64 * factor;
                }
            }
            weights.push(Weight::from_sums(&sums)?);
        }
        Ok(Self {
            faces: refinement.faces.clone(),
            input_weights: part.weights.clone(),
            input_triangles: part.triangles.clone(),
            input_count: n,
            stencils,
            triangles,
            weights,
        })
    }
    fn interpolate(&self, input: &[[f32; 3]]) -> Vec<[f32; 3]> {
        self.stencils
            .iter()
            .map(|row| {
                let mut out = [0f64; 3];
                for &(v, w) in row {
                    for a in 0..3 {
                        out[a] += input[v as usize][a] as f64 * w;
                    }
                }
                out.map(|v| v as f32)
            })
            .collect()
    }
}
impl Source {
    pub(super) fn validate_refinements(&self) -> Result<()> {
        if self.version < 4 && !self.refinements.is_empty() {
            return Err(invalid("Polygon refinement requires character source4"));
        }
        let mut names = BTreeSet::new();
        let mut total = self.parts.iter().map(|p| p.vertices.len()).sum::<usize>();
        for r in &self.refinements {
            if !names.insert(&r.part) {
                return Err(invalid("Duplicate character refinement part"));
            }
            let part = self
                .parts
                .iter()
                .find(|p| p.name == r.part)
                .ok_or_else(|| invalid("Unknown character refinement part"))?;
            let plan = r.plan(part)?;
            total += plan.stencils.len() - part.vertices.len();
            if total > MAX_VERTICES {
                return Err(invalid("Refined character exceeds100000 vertices"));
            }
        }
        Ok(())
    }
    pub(super) fn refine(&self, part: &Part, asset: &mut Asset) -> Result<()> {
        if let Some(r) = self.refinements.iter().find(|r| r.part == part.name) {
            let plan = r.plan(part)?;
            asset.geometry = Arc::new(Geometry {
                positions: plan.interpolate(&asset.geometry.positions),
                triangles: plan.triangles.clone(),
                material_coordinates: asset
                    .geometry
                    .material_coordinates
                    .as_ref()
                    .map(|v| plan.interpolate(v)),
            });
            let rig = Arc::make_mut(
                asset
                    .rig
                    .as_mut()
                    .ok_or_else(|| invalid("Character refinement requires rig weights"))?,
            );
            rig.weights = plan.weights.clone();
        }
        Ok(())
    }
}
