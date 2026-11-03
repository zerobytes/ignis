//! Conservative representation planning. Readiness/bounds are caller-declared,
//! not proof that the renderer, atmosphere or streaming backend has installed them.
use super::{id_valid, limited};
use crate::{
    model::{Result, invalid},
    world_space::{Coordinates, Position},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_NODES: usize = 4096;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub origin: Position,
    pub size_meters: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Atmosphere {
    Disabled,
    /// A visibility model for the future backend, not an implemented fog shader.
    Uniform {
        extinction_per_meter: f64,
        maximum_contrast: f64,
        negligible_contrast: f64,
    },
}
impl Atmosphere {
    pub fn horizon(&self) -> Result<Option<f64>> {
        match self {
            Self::Disabled => Ok(None),
            Self::Uniform {
                extinction_per_meter,
                maximum_contrast,
                negligible_contrast,
            } => {
                if !extinction_per_meter.is_finite()
                    || !(0. ..=16.).contains(extinction_per_meter)
                    || !maximum_contrast.is_finite()
                    || !(0.000_000_01..=1_000_000.).contains(maximum_contrast)
                    || !negligible_contrast.is_finite()
                    || !(0.000_000_01..=0.1).contains(negligible_contrast)
                    || negligible_contrast > maximum_contrast
                {
                    return Err(invalid("Invalid atmospheric extinction or contrast bounds"));
                }
                if *extinction_per_meter == 0. {
                    Ok(None)
                } else {
                    Ok(Some(
                        (maximum_contrast / negligible_contrast).ln() / extinction_per_meter,
                    ))
                }
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub region: Region,
    pub center: Position,
    pub radius_meters: f64,
    /// Four exact XZ quadrants: (0,0),(half,0),(0,half),(half,half).
    pub children: Option<[String; 4]>,
    pub ready: bool,
    /// True only after a producer has verified border compatibility for this set.
    pub children_seams_ready: bool,
    /// Producer-supplied approximation certificate, independent of atmosphere.
    pub approximation_error_meters: f64,
}
fn nodes<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Node>, D::Error> {
    limited::<D, Node, MAX_NODES>(d)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub viewer: Position,
    pub atmosphere: Atmosphere,
    pub maximum_error_meters: f64,
    pub representation_budget: u16,
    #[serde(deserialize_with = "nodes")]
    pub nodes: Vec<Node>,
}
#[derive(Debug, Serialize)]
pub struct Plan {
    pub proposals_only: bool,
    pub renderer_binding_required: bool,
    pub coverage_scope: &'static str,
    pub horizon_meters: Option<f64>,
    pub selected: Vec<String>,
    pub requests: Vec<String>,
    pub omitted_by_atmosphere_model: Vec<String>,
    pub uncovered_roots: Vec<String>,
    pub quality_deferred: usize,
    pub ready_for_supplied_roots: bool,
    pub budget_exceeded_by_fallback: bool,
}
fn distance(a: Position, b: Position) -> f64 {
    let a: Coordinates = a.into();
    let b: Coordinates = b.into();
    (0..3)
        .map(|axis| {
            let d = (i128::from(a.cell[axis]) - i128::from(b.cell[axis])) as f64 * 1024.
                + a.offset[axis]
                - b.offset[axis];
            d * d
        })
        .sum::<f64>()
        .sqrt()
}
struct Planner<'a> {
    query: &'a Query,
    index: BTreeMap<&'a str, usize>,
    horizon: Option<f64>,
    requests: BTreeSet<&'a str>,
    omitted: BTreeSet<&'a str>,
    deferred: usize,
}
impl<'a> Planner<'a> {
    fn invisible(&self, n: &Node) -> bool {
        self.horizon.is_some_and(|h| {
            let d = distance(n.center, self.query.viewer);
            // Round conservatively: numerical uncertainty never shortens coverage.
            d - n.radius_meters - d.abs() * f64::EPSILON * 16. > h
        })
    }
    fn ready(&self, i: usize) -> bool {
        let n = &self.query.nodes[i];
        self.invisible(n)
            || n.ready
            || n.children.as_ref().is_some_and(|c| {
                n.children_seams_ready && c.iter().all(|id| self.ready(self.index[id.as_str()]))
            })
    }
    fn request(&mut self, i: usize) {
        let n = &self.query.nodes[i];
        if !n.ready && !self.invisible(n) {
            self.requests.insert(&n.id);
        }
    }
    fn select(&mut self, i: usize, allowance: usize, out: &mut Vec<String>) -> bool {
        let n = &self.query.nodes[i];
        if self.invisible(n) {
            self.omitted.insert(&n.id);
            return true;
        }
        let children = n
            .children
            .as_ref()
            .map(|c| c.each_ref().map(|id| self.index[id.as_str()]));
        let refine = n.approximation_error_meters > self.query.maximum_error_meters || !n.ready;
        if refine && let Some(children) = children {
            for &c in &children {
                self.request(c);
            }
            if n.children_seams_ready && children.iter().all(|&c| self.ready(c)) && allowance >= 4 {
                // Transactional selection: if a deeper choice is incomplete, keep
                // the known parent, never publish a partial replacement group.
                let start = out.len();
                let mut complete = true;
                for (j, &c) in children.iter().enumerate() {
                    let remaining = allowance
                        .saturating_sub(out.len() - start)
                        .saturating_sub(3 - j);
                    complete &= self.select(c, remaining, out);
                }
                if complete {
                    return true;
                }
                out.truncate(start);
            }
        }
        if n.ready {
            out.push(n.id.clone());
            self.deferred += usize::from(refine);
            true
        } else {
            self.requests.insert(&n.id);
            false
        }
    }
}
pub fn plan(query: Query) -> Result<Plan> {
    if query.nodes.len() > MAX_NODES
        || query.nodes.is_empty()
        || query.representation_budget == 0
        || usize::from(query.representation_budget) > MAX_NODES
        || !query.maximum_error_meters.is_finite()
        || !(0. ..=1_000_000.).contains(&query.maximum_error_meters)
    {
        return Err(invalid(
            "Invalid terrain coverage nodes, budget or error tolerance",
        ));
    }
    let horizon = query.atmosphere.horizon()?;
    let mut index = BTreeMap::new();
    for (i, n) in query.nodes.iter().enumerate() {
        if !id_valid(&n.id)
            || !n.region.size_meters.is_power_of_two()
            || n.region.size_meters > 1_048_576
            || !n.radius_meters.is_finite()
            || !(0.125..=1_048_576.).contains(&n.radius_meters)
            || !n.approximation_error_meters.is_finite()
            || !(0. ..=1_000_000.).contains(&n.approximation_error_meters)
            || index.insert(n.id.as_str(), i).is_some()
        {
            return Err(invalid("Invalid terrain coverage node or duplicate ID"));
        }
    }
    let mut parent = vec![None; query.nodes.len()];
    for (i, n) in query.nodes.iter().enumerate() {
        if let Some(children) = &n.children {
            if n.region.size_meters < 2 {
                return Err(invalid("Terrain leaf cannot subdivide below1m"));
            }
            let half = n.region.size_meters / 2;
            for (quadrant, id) in children.iter().enumerate() {
                let &j = index
                    .get(id.as_str())
                    .ok_or_else(|| invalid("Missing terrain coverage child"))?;
                if i == j || parent[j].replace(i).is_some() {
                    return Err(invalid(
                        "Coverage hierarchy repeats a child or has a self cycle",
                    ));
                }
                let c = &query.nodes[j];
                let expected = n
                    .region
                    .origin
                    .translated([
                        f64::from(half) * ((quadrant % 2) as f64),
                        0.,
                        f64::from(half) * ((quadrant / 2) as f64),
                    ])
                    .map_err(|e| invalid(e.to_string()))?;
                if c.region.size_meters != half || c.region.origin != expected {
                    return Err(invalid(
                        "Terrain children must exactly partition the parent footprint",
                    ));
                }
                // Bounds are deliberately conservative, not an inferred partition.
                if distance(c.center, n.center) + c.radius_meters > n.radius_meters + 1e-6 {
                    return Err(invalid("Coverage parent does not contain its child bounds"));
                }
            }
        }
    }
    for i in 0..query.nodes.len() {
        let mut at = Some(i);
        let mut depth = 0;
        while let Some(j) = at {
            depth += 1;
            if depth > 16 {
                return Err(invalid("Terrain coverage cycle or depth exceeds16"));
            }
            at = parent[j];
        }
    }
    let mut roots: Vec<_> = (0..query.nodes.len())
        .filter(|&i| parent[i].is_none())
        .collect();
    roots.sort_by_key(|&i| &query.nodes[i].id);
    let mut planner = Planner {
        query: &query,
        index,
        horizon,
        requests: BTreeSet::new(),
        omitted: BTreeSet::new(),
        deferred: 0,
    };
    let mut selected = Vec::new();
    let mut uncovered = Vec::new();
    for (j, &i) in roots.iter().enumerate() {
        let allowance = usize::from(query.representation_budget)
            .saturating_sub(selected.len())
            .saturating_sub(roots.len() - j - 1);
        if !planner.select(i, allowance, &mut selected) {
            uncovered.push(query.nodes[i].id.clone());
        }
    }
    Ok(Plan {
        proposals_only: true,
        renderer_binding_required: true,
        coverage_scope: "supplied_roots_and_declared_partition_only",
        horizon_meters: horizon,
        budget_exceeded_by_fallback: selected.len() > usize::from(query.representation_budget),
        selected,
        requests: planner.requests.into_iter().map(str::to_owned).collect(),
        omitted_by_atmosphere_model: planner.omitted.into_iter().map(str::to_owned).collect(),
        ready_for_supplied_roots: uncovered.is_empty(),
        uncovered_roots: uncovered,
        quality_deferred: planner.deferred,
    })
}
