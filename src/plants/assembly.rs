//! Rebuild a three-port graft from immutable context and live plant sources.
use super::{Definition, Sampling, graft};
use crate::{
    mesh_bridge, mesh_junction_union, mesh_material_finish, mesh_ring_loft, mesh_surface, meshes,
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Assembly {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_grafts: Vec<graft::ContextGraft>,
    pub context: Arc<meshes::Asset>,
    pub context_faces: Vec<u32>,
    /// Canonical context vertex IDs before cutting/compaction. Membership is exact;
    /// winding comes from source faces rather than spatial proximity.
    pub context_loops: [Vec<u32>; 2],
    pub station: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_bone: Option<String>,
    /// Material-only affine applied after preservation and the named section cut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plant_material_frame: Option<Arc<mesh_material_finish::Affine>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_finish: Option<Arc<mesh_material_finish::Options>>,
    pub options: mesh_junction_union::Options,
    /// References are into immutable context, never the evaluated composition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_lofts: Vec<mesh_ring_loft::Loft>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_surface: Option<ContextSurface>,
    /// Select the generated iso domain by provenance, leaving source and guards fixed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub junction_surface: Option<mesh_surface::Options>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextSurface {
    pub vertices: Vec<u32>,
    pub anchors: Vec<u32>,
    pub options: mesh_surface::Options,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_grafts: Vec<graft::Report>,
    pub retained_context_vertices: usize,
    pub retained_plant_vertices: usize,
    pub junction: mesh_junction_union::Report,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_lofts: Option<mesh_ring_loft::Report>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_surface: Option<mesh_surface::Report>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub junction_surface: Option<mesh_surface::Report>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_finish: Option<mesh_material_finish::Report>,
}
fn retained(g: &meshes::Geometry, removed: &[u32]) -> Vec<usize> {
    let removed: BTreeSet<_> = removed.iter().copied().collect();
    let mut used = vec![false; g.positions.len()];
    for (i, t) in g.triangles.iter().enumerate() {
        if !removed.contains(&(i as u32)) {
            for &v in t {
                used[v as usize] = true;
            }
        }
    }
    used.into_iter()
        .enumerate()
        .filter_map(|(i, yes)| yes.then_some(i))
        .collect()
}
impl Assembly {
    pub(super) fn evaluate(
        &self,
        plant: &meshes::Asset,
        definition: &Definition,
        sample: &Sampling,
    ) -> Result<(meshes::Asset, Report)> {
        self.context.validate()?;
        if self.context_grafts.len() > 8 {
            return Err(invalid("Assembly allows at most8context grafts"));
        }
        if self
            .context_loops
            .iter()
            .flatten()
            .any(|&i| i as usize >= self.context.geometry.positions.len())
        {
            return Err(invalid("Assembly port requires canonical context vertices"));
        }
        if self
            .context_loops
            .iter()
            .any(|p| !(4..=32).contains(&p.len()))
        {
            return Err(invalid("Assembly context ports require4..32 vertices"));
        }
        if self.context.id == plant.id
            || self.context.rig.is_some()
            || !self.context.clips.is_empty()
        {
            return Err(invalid(
                "Assembly context requires a distinct static canonical asset",
            ));
        }
        if self.context.material != plant.material || self.context.shading != plant.shading {
            return Err(invalid(
                "Assembly context and plant require identical appearance",
            ));
        }
        if self.context.geometry.material_coordinates.is_some()
            != plant.geometry.material_coordinates.is_some()
        {
            return Err(invalid(
                "Assembly requires matching material-domain presence",
            ));
        }
        let sections = definition
            .sections
            .as_ref()
            .ok_or_else(|| invalid("Assembly requires named sections"))?;
        let section = sections
            .stations
            .iter()
            .position(|s| s.id == self.station)
            .filter(|&i| i > 0 && i + 1 < sections.stations.len())
            .ok_or_else(|| invalid("Assembly must retain an interior named section"))?;
        // Material Y encodes interval identity, independently of physical at.
        let cut = section as f64 / (sections.stations.len() - 1) as f64 - 0.5;
        let faces: Vec<_> = plant
            .geometry
            .triangles
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.iter()
                    .any(|&i| f64::from(sample.material[i as usize][1]) < cut - 1e-6)
            })
            .map(|(i, _)| i as u32)
            .collect();
        // Cuts refer to canonical faces; lofts retain their source vertex IDs.
        // A cut through refined faces needs an explicit new domain, never a guessed map.
        let lofted = if self.context_lofts.is_empty() {
            None
        } else {
            Some(mesh_ring_loft::fit(&self.context, &self.context_lofts)?)
        };
        let prepared = lofted.as_ref().map_or(self.context.as_ref(), |f| &f.asset);
        let smoothed = if let Some(surface) = &self.context_surface {
            let count = self.context.geometry.positions.len();
            let selected: BTreeSet<_> = surface.vertices.iter().copied().collect();
            let anchors: BTreeSet<_> = surface.anchors.iter().copied().collect();
            if selected.is_empty()
                || selected.len() != surface.vertices.len()
                || anchors.len() != surface.anchors.len()
                || selected.iter().any(|&v| v as usize >= count)
                || !anchors.is_subset(&selected)
            {
                return Err(invalid(
                    "Context surface requires unique canonical vertices and subset anchors",
                ));
            }
            let mut selection = surface.vertices.clone();
            let mut pins = surface.anchors.clone();
            if let Some(loft) = &lofted {
                for (i, ends) in loft.origins.iter().enumerate() {
                    let vertex = (count + i) as u32;
                    if ends.iter().all(|v| selected.contains(v)) {
                        selection.push(vertex);
                    }
                    if ends.iter().all(|v| anchors.contains(v)) {
                        pins.push(vertex);
                    }
                }
            }
            let filtered =
                mesh_surface::smooth_region(prepared, &selection, &pins, &surface.options)
                    .map_err(|mut error| {
                        error.message.insert_str(0, "Context surface: ");
                        error
                    })?;
            if self.context_loops.iter().flatten().any(|&i| {
                filtered.asset.geometry.positions.get(i as usize)
                    != self.context.geometry.positions.get(i as usize)
            }) {
                return Err(invalid(
                    "Context finishing must preserve canonical junction ports",
                ));
            }
            Some(filtered)
        } else {
            None
        };
        let prepared = smoothed.as_ref().map_or(prepared, |f| &f.asset);
        let mut context_map: Vec<_> = (0..prepared.geometry.positions.len())
            .map(|i| Some(i as u32))
            .collect();
        let mut grafted = None;
        let mut graft_reports = Vec::with_capacity(self.context_grafts.len());
        for recipe in &self.context_grafts {
            let selected: BTreeSet<_> = recipe.context_vertices.iter().copied().collect();
            if selected.is_empty()
                || selected.len() != recipe.context_vertices.len()
                || selected
                    .iter()
                    .any(|&i| i as usize >= self.context.geometry.positions.len())
            {
                return Err(invalid(
                    "Graft mask requires unique canonical context vertices",
                ));
            }
            let mut selection = recipe.context_vertices.clone();
            if let Some(loft) = &lofted {
                for (i, ends) in loft.origins.iter().enumerate() {
                    if ends.iter().all(|v| selected.contains(v)) {
                        selection.push((self.context.geometry.positions.len() + i) as u32);
                    }
                }
            }
            // Ports must reference canonical context vertices, never generated iso IDs.
            if recipe
                .context_loops
                .iter()
                .flatten()
                .any(|&i| i as usize >= self.context.geometry.positions.len())
            {
                return Err(invalid("Graft port requires canonical context vertices"));
            }
            let selection = graft::remap(&selection, &context_map)?;
            let loops = [
                graft::remap(&recipe.context_loops[0], &context_map)?,
                graft::remap(&recipe.context_loops[1], &context_map)?,
            ];
            let current = grafted.as_ref().unwrap_or(prepared);
            let (asset, kept, report) = recipe.evaluate(current, &selection, &loops)?;
            for index in &mut context_map {
                if let Some(i) = index {
                    *index = kept[*i as usize];
                }
            }
            graft_reports.push(report);
            grafted = Some(asset);
        }
        let prepared = grafted.as_ref().unwrap_or(prepared);
        for &canonical in self.context_loops.iter().flatten() {
            let current = context_map[canonical as usize]
                .ok_or_else(|| invalid("Graft removed a main assembly port"))?;
            if prepared.geometry.positions[current as usize]
                != self.context.geometry.positions[canonical as usize]
            {
                return Err(invalid(
                    "Graft collar must preserve the main assembly ports",
                ));
            }
        }
        let triangles: BTreeMap<_, _> = prepared
            .geometry
            .triangles
            .iter()
            .enumerate()
            .map(|(i, t)| (*t, i as u32))
            .collect();
        let context_faces = self.context_faces.iter().map(|&f| {
            let t = self.context.geometry.triangles.get(f as usize).ok_or_else(|| invalid("Assembly cut face is absent"))?;
            let mapped = graft::remap(t, &context_map)?;
            triangles.get(&[mapped[0],mapped[1],mapped[2]]).copied().ok_or_else(||invalid("Assembly cut overlaps a context loft or graft; supply a distinct canonical domain"))
        }).collect::<Result<Vec<_>>>()?;
        let main_loops = [
            graft::remap(&self.context_loops[0], &context_map)?,
            graft::remap(&self.context_loops[1], &context_map)?,
        ];
        // Validate indices/duplicates before using the retained map.
        let context = mesh_bridge::remove_faces(prepared, &context_faces)?;
        let context_kept = retained(&prepared.geometry, &context_faces);
        let mut plant = mesh_bridge::remove_faces(plant, &faces)?;
        if let Some(frame) = &self.plant_material_frame {
            super::material::transform(&mut plant, frame)?;
        }
        let nv = context.geometry.positions.len() + plant.geometry.positions.len();
        let nt = context.geometry.triangles.len() + plant.geometry.triangles.len();
        if nv >= meshes::MAX_VERTICES || nt >= meshes::MAX_TRIANGLES {
            return Err(invalid(
                "Assembly retained geometry leaves no junction budget",
            ));
        }
        let context_boundaries = mesh_bridge::boundaries(&context.geometry)?;
        if context_boundaries.len() != 2 {
            return Err(invalid(
                "Assembly context must expose exactly two boundaries",
            ));
        }
        let mut ports: [Vec<u32>; 3] = [vec![], vec![], vec![]];
        for (slot, original) in [(0, &main_loops[0]), (2, &main_loops[1])] {
            let mapped = original
                .iter()
                .map(|i| {
                    context_kept
                        .binary_search(&(*i as usize))
                        .map(|j| j as u32)
                        .map_err(|_| invalid("Assembly context port references a removed vertex"))
                })
                .collect::<Result<Vec<_>>>()?;
            let set: BTreeSet<_> = mapped.iter().copied().collect();
            if set.len() != mapped.len() {
                return Err(invalid("Assembly context port contains repeated vertices"));
            }
            ports[slot] = context_boundaries
                .iter()
                .find(|b| b.vertices.iter().copied().collect::<BTreeSet<_>>() == set)
                .ok_or_else(|| invalid("Assembly context port must match its exact real boundary"))?
                .vertices
                .clone();
        }
        if ports[0] == ports[2] {
            return Err(invalid("Assembly context ports must be distinct"));
        }
        let mut plant_boundaries = mesh_bridge::boundaries(&plant.geometry)?;
        if plant_boundaries.len() != 1 {
            return Err(invalid(
                "Assembly plant cut must expose exactly one boundary",
            ));
        }
        let n = context.geometry.positions.len();
        ports[1] = plant_boundaries
            .remove(0)
            .vertices
            .into_iter()
            .map(|i| i + n as u32)
            .collect();
        let mut geometry = (*context.geometry).clone();
        geometry.positions.reserve(plant.geometry.positions.len());
        geometry
            .positions
            .extend_from_slice(&plant.geometry.positions);
        geometry.triangles.extend(
            plant
                .geometry
                .triangles
                .iter()
                .map(|t| t.map(|i| i + n as u32)),
        );
        if let (Some(a), Some(b)) = (
            &mut geometry.material_coordinates,
            &plant.geometry.material_coordinates,
        ) {
            a.extend_from_slice(b);
        }
        let mut composed = plant.clone();
        composed.geometry = Arc::new(geometry);
        if let Some(rig) = &plant.rig {
            let bone = self
                .context_bone
                .as_ref()
                .and_then(|name| {
                    rig.bones
                        .iter()
                        .position(|b| &b.name == name && b.parent.is_none())
                })
                .ok_or_else(|| invalid("Rigged assembly requires an explicit context root"))?;
            let mut rig = (**rig).clone();
            rig.weights = std::iter::repeat_n(
                meshes::Weight {
                    extra: None,
                    joints: [bone as u8; 4],
                    weights: [1., 0., 0., 0.],
                },
                n,
            )
            .chain(rig.weights)
            .collect();
            composed.rig = Some(Arc::new(rig));
        } else if self.context_bone.is_some() {
            return Err(invalid("Assembly context root requires a rig"));
        }
        let mut union_options = self.options.clone();
        if self.material_finish.is_some() {
            union_options.material_relaxation_steps = 0;
        }
        let fitted = mesh_junction_union::fit(
            &composed,
            &ports,
            self.context_bone.as_deref(),
            &union_options,
        )?;
        let finished = if let Some(options) = &self.junction_surface {
            let start = n + plant.geometry.positions.len();
            let vertices = (start..start + fitted.report.extracted_vertices)
                .map(|i| i as u32)
                .collect::<Vec<_>>();
            Some(
                mesh_surface::smooth_region(&fitted.asset, &vertices, &[], options).map_err(
                    |mut error| {
                        error.message.insert_str(0, "Junction surface: ");
                        error
                    },
                )?,
            )
        } else {
            None
        };
        let junction_surface = finished.as_ref().map(|f| f.report.clone());
        let asset = finished.map_or(fitted.asset, |f| f.asset);
        let material = self
            .material_finish
            .as_ref()
            .map(|options| mesh_material_finish::finish(&asset, &fitted.material_plan, options))
            .transpose()
            .map_err(|mut e| {
                e.message.insert_str(0, "Assembly material finish: ");
                e
            })?;
        let (asset, material_finish) =
            material.map_or((asset, None), |f| (f.asset, Some(f.report)));
        Ok((
            asset,
            Report {
                context_grafts: graft_reports,
                retained_context_vertices: n,
                retained_plant_vertices: plant.geometry.positions.len(),
                junction: fitted.report,
                context_lofts: lofted.map(|f| f.report),
                context_surface: smoothed.map(|f| f.report),
                junction_surface,
                material_finish,
            },
        ))
    }
}
