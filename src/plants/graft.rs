//! Explicit static side branches in a context. No spatial correspondence or skin inference.
use crate::{
    mesh_bridge, mesh_junction_union, mesh_material_finish, mesh_surface, meshes,
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextGraft {
    pub source: Arc<meshes::Asset>,
    /// Explicit translation from immutable source coordinates into context coordinates.
    pub source_translation: [f32; 3],
    /// Applied to evaluated source material only; canonical source remains unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_material_frame: Option<Arc<mesh_material_finish::Affine>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_finish: Option<Arc<mesh_material_finish::Options>>,
    pub source_faces: Vec<u32>,
    pub source_loop: Vec<u32>,
    /// Canonical context selection. All vertices of a removed face must be selected.
    pub context_vertices: Vec<u32>,
    pub context_loops: [Vec<u32>; 2],
    pub options: mesh_junction_union::Options,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<mesh_surface::Options>,
    /// Intentional post-union position edits on retained and generated collar points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collar_surface: Option<CollarSurface>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollarSurface {
    /// Ordinals into the two context ports followed by the source port.
    pub ports: Vec<u8>,
    pub rings: u8,
    pub fixed_rings: u8,
    pub options: mesh_surface::Options,
}
impl CollarSurface {
    fn validate(&self) -> Result<()> {
        let mut ports = [false; 3];
        if self.ports.is_empty()
            || self.ports.len() > 3
            || self.ports.iter().any(|&i| {
                if i >= 3 || ports[i as usize] {
                    true
                } else {
                    ports[i as usize] = true;
                    false
                }
            })
            || !(1..=16).contains(&self.rings)
            || self.fixed_rings == 0
            || self.fixed_rings > self.rings
        {
            return Err(invalid(
                "Collar surface requires unique ports0..2 and1..16rings with1..rings fixed layers",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct CollarReport {
    pub filter: mesh_surface::Report,
    pub moved_retained_context: usize,
    pub moved_retained_source: usize,
    pub moved_generated: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub retained_context_vertices: usize,
    pub retained_source_vertices: usize,
    pub junction: mesh_junction_union::Report,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface: Option<mesh_surface::Report>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collar_surface: Option<CollarReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_finish: Option<mesh_material_finish::Report>,
}
// Maps original index -> compacted index, retaining exact source order.
pub(super) fn kept_map(g: &meshes::Geometry, faces: &[u32]) -> Vec<Option<u32>> {
    let removed: BTreeSet<_> = faces.iter().copied().collect();
    let mut used = vec![false; g.positions.len()];
    for (i, t) in g.triangles.iter().enumerate() {
        if !removed.contains(&(i as u32)) {
            for &v in t {
                used[v as usize] = true;
            }
        }
    }
    let mut next = 0;
    used.into_iter()
        .map(|used| {
            used.then(|| {
                let i = next;
                next += 1;
                i
            })
        })
        .collect()
}
pub(super) fn remap(ids: &[u32], map: &[Option<u32>]) -> Result<Vec<u32>> {
    ids.iter()
        .map(|&i| {
            map.get(i as usize).copied().flatten().ok_or_else(|| {
                invalid("Graft references an absent or previously removed context vertex")
            })
        })
        .collect()
}
fn port(asset: &meshes::Asset, ids: &[u32]) -> Result<Vec<u32>> {
    let set: BTreeSet<_> = ids.iter().copied().collect();
    if !(4..=32).contains(&ids.len()) || set.len() != ids.len() {
        return Err(invalid("Graft requires unique4..32vertex ports"));
    }
    mesh_bridge::boundaries(&asset.geometry)?
        .into_iter()
        .find(|b| b.vertices.iter().copied().collect::<BTreeSet<_>>() == set)
        .map(|b| b.vertices)
        .ok_or_else(|| invalid("Graft port must match its exact real boundary"))
}
impl ContextGraft {
    pub(super) fn evaluate(
        &self,
        context: &meshes::Asset,
        selection: &[u32],
        loops: &[Vec<u32>; 2],
    ) -> Result<(meshes::Asset, Vec<Option<u32>>, Report)> {
        if let Some(collar) = &self.collar_surface {
            collar.validate()?;
        }
        self.source.validate()?;
        if self
            .source_translation
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 10000.)
        {
            return Err(invalid("Graft source translation outside finite bounds"));
        }
        if self.source.id == context.id
            || context.rig.is_some()
            || self.source.rig.is_some()
            || !context.clips.is_empty()
            || !self.source.clips.is_empty()
        {
            return Err(invalid(
                "Context graft requires distinct static sources; skin is not inferred",
            ));
        }
        if context.material != self.source.material
            || context.shading != self.source.shading
            || context.geometry.material_coordinates.is_some()
                != self.source.geometry.material_coordinates.is_some()
        {
            return Err(invalid(
                "Context graft requires matching appearance and material-domain presence",
            ));
        }
        let mask: BTreeSet<_> = selection.iter().copied().collect();
        if mask.is_empty()
            || mask.len() != selection.len()
            || mask
                .iter()
                .any(|&i| i as usize >= context.geometry.positions.len())
        {
            return Err(invalid("Graft cut requires unique context vertices"));
        }
        let faces: Vec<_> = context
            .geometry
            .triangles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.iter().all(|i| mask.contains(i)))
            .map(|(i, _)| i as u32)
            .collect();
        let cut = mesh_bridge::remove_faces(context, &faces)?;
        let context_map = kept_map(&context.geometry, &faces);
        let mut branch = mesh_bridge::remove_faces(&self.source, &self.source_faces)?;
        if let Some(frame) = &self.source_material_frame {
            super::material::transform(&mut branch, frame)?;
        }
        for p in &mut Arc::make_mut(&mut branch.geometry).positions {
            for (x, delta) in p.iter_mut().zip(self.source_translation) {
                *x += delta;
            }
        }
        let source_map = kept_map(&self.source.geometry, &self.source_faces);
        let a = port(&cut, &remap(&loops[0], &context_map)?)?;
        let b = port(&cut, &remap(&loops[1], &context_map)?)?;
        let c = port(&branch, &remap(&self.source_loop, &source_map)?)?;
        let n = cut.geometry.positions.len();
        let m = branch.geometry.positions.len();
        if n + m >= meshes::MAX_VERTICES
            || cut.geometry.triangles.len() + branch.geometry.triangles.len()
                >= meshes::MAX_TRIANGLES
        {
            return Err(invalid("Graft retained sources leave no junction budget"));
        }
        let mut g = (*cut.geometry).clone();
        g.positions.reserve(m);
        g.positions.extend_from_slice(&branch.geometry.positions);
        g.triangles.reserve(branch.geometry.triangles.len());
        g.triangles.extend(
            branch
                .geometry
                .triangles
                .iter()
                .map(|t| t.map(|i| i + n as u32)),
        );
        if let (Some(a), Some(b)) = (
            &mut g.material_coordinates,
            &branch.geometry.material_coordinates,
        ) {
            a.extend_from_slice(b);
        }
        let mut joined = cut;
        joined.geometry = Arc::new(g);
        let ports = [a, b, c.into_iter().map(|i| i + n as u32).collect()];
        let mut union_options = self.options.clone();
        if self.material_finish.is_some() {
            union_options.material_relaxation_steps = 0;
        }
        let fitted = mesh_junction_union::fit(&joined, &ports, None, &union_options)?;
        let finished = self
            .surface
            .as_ref()
            .map(|options| {
                let vertices = (n + m..n + m + fitted.report.extracted_vertices)
                    .map(|i| i as u32)
                    .collect::<Vec<_>>();
                mesh_surface::smooth_region(&fitted.asset, &vertices, &[], options)
            })
            .transpose()?;
        let surface = finished.as_ref().map(|f| f.report.clone());
        let base = finished.as_ref().map_or(&fitted.asset, |f| &f.asset);
        let collar = self
            .collar_surface
            .as_ref()
            .map(|options| {
                let seeds = options
                    .ports
                    .iter()
                    .flat_map(|&i| ports[i as usize].iter().copied())
                    .collect::<Vec<_>>();
                let mut region = mesh_surface::ring_region(
                    &base.geometry,
                    &seeds,
                    options.rings,
                    options.fixed_rings,
                )?;
                // Unselected real ports and their first tangent guards remain exact.
                let mut guard = n + m + fitted.report.extracted_vertices;
                for (i, port) in ports.iter().enumerate() {
                    if !options.ports.contains(&(i as u8)) {
                        for point in port
                            .iter()
                            .copied()
                            .chain((guard..guard + port.len()).map(|i| i as u32))
                        {
                            if region.vertices.binary_search(&point).is_ok() {
                                region.anchors.push(point);
                            }
                        }
                    }
                    guard += port.len();
                }
                region.anchors.sort_unstable();
                region.anchors.dedup();
                mesh_surface::smooth_region(
                    base,
                    &region.vertices,
                    &region.anchors,
                    &options.options,
                )
            })
            .transpose()
            .map_err(|mut error| {
                error.message.insert_str(0, "Graft collar surface: ");
                error
            })?;
        let collar_surface = collar.as_ref().map(|f| {
            let moved = |range: std::ops::Range<usize>| {
                range
                    .filter(|&i| base.geometry.positions[i] != f.asset.geometry.positions[i])
                    .count()
            };
            CollarReport {
                filter: f.report.clone(),
                moved_retained_context: moved(0..n),
                moved_retained_source: moved(n..n + m),
                moved_generated: moved(n + m..base.geometry.positions.len()),
            }
        });
        let asset = if let Some(collar) = collar {
            collar.asset
        } else {
            finished.map_or(fitted.asset, |f| f.asset)
        };
        let material = self
            .material_finish
            .as_ref()
            .map(|options| mesh_material_finish::finish(&asset, &fitted.material_plan, options))
            .transpose()
            .map_err(|mut e| {
                e.message.insert_str(0, "Graft material finish: ");
                e
            })?;
        let (asset, material_finish) =
            material.map_or((asset, None), |f| (f.asset, Some(f.report)));
        Ok((
            asset,
            context_map,
            Report {
                retained_context_vertices: n,
                retained_source_vertices: m,
                junction: fitted.report,
                surface,
                collar_surface,
                material_finish,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reached_unselected_port_and_its_guards_are_pinned() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../examples/mesh-junction-union.input.json"
        ))
        .unwrap();
        let mut original: meshes::Asset = serde_json::from_value(f["source"].clone()).unwrap();
        original.rig = None;
        original.clips.clear();
        let ports: [Vec<u32>; 3] = serde_json::from_value(f["ports"].clone()).unwrap();
        let mut adjacent = vec![vec![]; original.geometry.positions.len()];
        for t in &original.geometry.triangles {
            for &a in t {
                for &b in t {
                    adjacent[a as usize].push(b);
                }
            }
        }
        let mut child = vec![false; adjacent.len()];
        let mut queue = vec![ports[1][0]];
        child[queue[0] as usize] = true;
        while let Some(i) = queue.pop() {
            for &j in &adjacent[i as usize] {
                if !child[j as usize] {
                    child[j as usize] = true;
                    queue.push(j);
                }
            }
        }
        let subset = |take_child: bool, id: &str| {
            let mut map = vec![None; child.len()];
            let mut g = meshes::Geometry {
                positions: vec![],
                triangles: vec![],
                material_coordinates: Some(vec![]),
            };
            for (i, &take) in child.iter().enumerate() {
                if take == take_child {
                    map[i] = Some(g.positions.len() as u32);
                    g.positions.push(original.geometry.positions[i]);
                    g.material_coordinates
                        .as_mut()
                        .unwrap()
                        .push(original.geometry.material_coordinates.as_ref().unwrap()[i]);
                }
            }
            g.triangles = original
                .geometry
                .triangles
                .iter()
                .filter(|t| child[t[0] as usize] == take_child)
                .map(|t| t.map(|i| map[i as usize].unwrap()))
                .collect();
            let mut a = original.clone();
            a.id = id.into();
            a.geometry = Arc::new(g);
            (a, map)
        };
        let (mut context, cm) = subset(false, "parent");
        let (mut source, sm) = subset(true, "child");
        let loops = [
            remap(&ports[0], &cm).unwrap(),
            remap(&ports[2], &cm).unwrap(),
        ];
        let source_loop = remap(&ports[1], &sm).unwrap();
        let cap = |a: &mut meshes::Asset, lp: &[u32]| {
            let g = Arc::make_mut(&mut a.geometry);
            let first = g.triangles.len();
            for j in 1..lp.len() - 1 {
                g.triangles.push([lp[0], lp[j + 1], lp[j]]);
            }
            (first..g.triangles.len())
                .map(|i| i as u32)
                .collect::<Vec<_>>()
        };
        cap(&mut context, &loops[0]);
        cap(&mut context, &loops[1]);
        let source_faces = cap(&mut source, &source_loop);
        let mask = loops.iter().flatten().copied().collect::<Vec<_>>();
        let mut graft = ContextGraft {
            source: Arc::new(source),
            source_translation: [0.; 3],
            source_material_frame: None,
            material_finish: None,
            source_faces,
            source_loop: source_loop.clone(),
            context_vertices: mask.clone(),
            context_loops: loops.clone(),
            options: serde_json::from_value(f["options"].clone()).unwrap(),
            surface: None,
            collar_surface: None,
        };
        let (base, _, report) = graft.evaluate(&context, &mask, &loops).unwrap();
        let n = report.retained_context_vertices;
        let m = report.retained_source_vertices;
        let seeds = loops.iter().flatten().copied().collect::<Vec<_>>();
        let region = mesh_surface::ring_region(&base.geometry, &seeds, 10, 1).unwrap();
        let reached = source_loop
            .iter()
            .map(|&i| i + n as u32)
            .filter(|i| {
                region.vertices.binary_search(i).is_ok() && region.anchors.binary_search(i).is_err()
            })
            .collect::<Vec<_>>();
        assert!(!reached.is_empty());
        graft.collar_surface = Some(CollarSurface {
            ports: vec![0, 1],
            rings: 10,
            fixed_rings: 1,
            options: mesh_surface::Options {
                iterations: 1,
                strength: 0.01,
                max_displacement: 0.0001,
            },
        });
        let (after, _, report_after) = graft.evaluate(&context, &mask, &loops).unwrap();
        assert!(report_after.collar_surface.unwrap().filter.moved_vertices > 0);
        let guard = n + m + report.junction.extracted_vertices + loops[0].len() + loops[1].len();
        for i in source_loop
            .iter()
            .map(|&i| i as usize + n)
            .chain(guard..guard + source_loop.len())
        {
            assert_eq!(after.geometry.positions[i], base.geometry.positions[i]);
        }
    }
}
