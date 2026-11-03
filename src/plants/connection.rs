//! Regenerable disjoint plant pairs. Sources remain independent authoring documents.
use super::{Document, Entry, Evaluated, Sampling};
use crate::{
    mesh_bridge, mesh_junction_union, meshes,
    model::{Result, invalid},
};
use glam::{Mat4, Quat, Vec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
mod skin;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    Parent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub hub_offset: [f32; 3],
    pub hub_radius: f32,
    pub blend: f32,
    pub cell_size: f32,
    pub guard_length: f32,
    pub collar_length: f32,
    pub relaxation_steps: u8,
    pub material_offset: [f32; 3],
    #[serde(default)]
    pub seam_relaxation_steps: u8,
    #[serde(default)]
    pub seam_max_displacement: f32,
    #[serde(default)]
    pub material_relaxation_steps: u8,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Skin {
    pub iterations: u16,
    pub tolerance: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub id: String,
    pub parent_instance: String,
    pub child_instance: String,
    pub parent_sections: [String; 2],
    pub child_section: String,
    pub parent_root: String,
    pub child_root: String,
    pub appearance: Appearance,
    pub shape: Shape,
    pub skin: Skin,
}
#[derive(Debug, Serialize)]
pub struct ConnectionReport {
    pub id: String,
    pub parent_asset: String,
    pub child_asset: String,
    pub retained_parent_vertices: usize,
    pub retained_child_vertices: usize,
    pub junction: mesh_junction_union::Report,
    pub skin: skin::Report,
}

impl Document {
    fn pairs(&self, assets: &meshes::Document) -> Result<Vec<(usize, usize)>> {
        if self.connections.len() > 8 {
            return Err(invalid("At most8 disjoint plant connections"));
        }
        let mut used = BTreeSet::new();
        let mut names = BTreeSet::new();
        self.connections.iter().map(|c| {
            if !super::valid_id(&c.id) || !names.insert(&c.id) {
                return Err(invalid("Connection requires a unique stable ID"));
            }
            let mut pair = [0; 2];
            for (slot, name) in [&c.parent_instance, &c.child_instance].into_iter().enumerate() {
                let instance = assets.instances.iter().find(|i| &i.id == name)
                    .ok_or_else(|| invalid("Connection instance is missing"))?;
                let index = self.entries.iter().position(|e| e.definition.id == instance.asset)
                    .ok_or_else(|| invalid("Connection requires two live plants"))?;
                if !used.insert(index) {
                    return Err(invalid("Connections require disjoint plants; cycles, chains and shared members are not supported"));
                }
                if assets.instances.iter().filter(|i| i.asset == instance.asset).count() != 1 {
                    return Err(invalid("Connected plant assets require one instance each"));
                }
                pair[slot] = index;
            }
            Ok((pair[0], pair[1]))
        }).collect()
    }
    pub(super) fn evaluate_connections(
        &self,
        assets: &meshes::Document,
    ) -> Result<(Vec<Evaluated>, Vec<ConnectionReport>)> {
        let pairs = self.pairs(assets)?;
        let mut evaluated = self
            .entries
            .iter()
            .map(Entry::evaluate)
            .collect::<Result<Vec<_>>>()?;
        let mut reports = Vec::with_capacity(pairs.len());
        for (c, (parent, child)) in self.connections.iter().zip(pairs) {
            let (asset, report) = c.evaluate(
                &self.entries[parent],
                &self.entries[child],
                &evaluated[parent],
                &evaluated[child],
                assets,
            )?;
            evaluated[parent].asset = asset;
            reports.push(report);
        }
        Ok((evaluated, reports))
    }
    pub(crate) fn rebuild_connections(&self, assets: &mut meshes::Document) -> Result<()> {
        if self.connections.is_empty() {
            return Ok(());
        }
        for evaluated in self.evaluate_connections(assets)?.0 {
            assets.apply(meshes::Edit::PutAsset {
                asset: evaluated.asset,
            })?;
        }
        Ok(())
    }
    pub fn connection_reports(&self, assets: &meshes::Document) -> Result<Vec<ConnectionReport>> {
        Ok(self.evaluate_connections(assets)?.1)
    }
}

fn station(entry: &Entry, name: &str) -> Result<(usize, f64)> {
    let sections = entry
        .definition
        .sections
        .as_ref()
        .filter(|s| s.cross_section == super::CrossSection::Ellipse)
        .ok_or_else(|| invalid("Connection requires elliptical named sections"))?;
    let index = sections
        .stations
        .iter()
        .position(|s| s.id == name)
        .filter(|&i| i > 0 && i + 1 < sections.stations.len())
        .ok_or_else(|| invalid("Connection section must be a named interior station"))?;
    Ok((
        index,
        index as f64 / (sections.stations.len() - 1) as f64 - 0.5,
    ))
}

fn kept(g: &meshes::Geometry, removed: &BTreeSet<u32>) -> Vec<usize> {
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

/// Map the base's material cells into the retained plant prefix of an assembly.
fn base_map(entry: &Entry, evaluated: &Evaluated, sample: &Sampling) -> Result<Vec<Option<u32>>> {
    let mut map = vec![None; sample.material.len()];
    if let Some(assembly) = &entry.assembly {
        let (_, cut) = station(entry, &assembly.station)?;
        let removed = sample
            .geometry
            .triangles
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.iter()
                    .any(|&v| f64::from(sample.material[v as usize][1]) < cut - 1e-6)
            })
            .map(|(i, _)| i as u32)
            .collect();
        let offset = evaluated
            .report
            .assembly
            .as_ref()
            .ok_or_else(|| invalid("Missing assembly report"))?
            .retained_context_vertices;
        for (i, v) in kept(&sample.geometry, &removed).into_iter().enumerate() {
            map[v] = Some((offset + i) as u32);
        }
    } else {
        for (i, slot) in map.iter_mut().enumerate() {
            *slot = Some(i as u32);
        }
    }
    Ok(map)
}

fn rigid(weight: &meshes::Weight, bone: usize) -> bool {
    weight
        .influences()
        .all(|(j, w)| w == 0. || j as usize == bone)
}

impl Connection {
    fn evaluate(
        &self,
        parent: &Entry,
        child: &Entry,
        pa: &Evaluated,
        ca: &Evaluated,
        assets: &meshes::Document,
    ) -> Result<(meshes::Asset, ConnectionReport)> {
        if child.assembly.is_some() {
            return Err(invalid("Connected child must not have an assembly"));
        }
        if self.skin.iterations == 0
            || self.skin.iterations > 256
            || !self.skin.tolerance.is_finite()
            || !(1e-8..=1e-3).contains(&self.skin.tolerance)
        {
            return Err(invalid(
                "Connection skin requires1..256 iterations and tolerance1e-8..1e-3",
            ));
        }
        let pi = assets
            .instances
            .iter()
            .find(|i| i.id == self.parent_instance)
            .ok_or_else(|| invalid("Connection parent instance absent"))?;
        let ci = assets
            .instances
            .iter()
            .find(|i| i.id == self.child_instance)
            .ok_or_else(|| invalid("Connection child instance absent"))?;
        let attachment = ci
            .attachment
            .as_ref()
            .filter(|a| a.parent == pi.id)
            .ok_or_else(|| {
                invalid("Connection child must attach directly to its parent instance")
            })?;
        if ci.playback.is_some() || !ca.asset.clips.is_empty() {
            return Err(invalid(
                "Connected rigid child cannot have playback or clips",
            ));
        }
        if pa.asset.shading != ca.asset.shading {
            return Err(invalid("Connected plants require matching shading"));
        }
        let pr = pa
            .asset
            .rig
            .as_ref()
            .ok_or_else(|| invalid("Connection parent needs a rig"))?;
        let cr = ca
            .asset
            .rig
            .as_ref()
            .ok_or_else(|| invalid("Connection child needs a rig"))?;
        let root = pr
            .bones
            .iter()
            .position(|b| b.name == self.parent_root && b.parent.is_none())
            .ok_or_else(|| invalid("Connection parent root is missing"))?;
        let anchor = attachment
            .bone
            .as_ref()
            .and_then(|n| pr.bones.iter().position(|b| &b.name == n))
            .filter(|&i| i != root)
            .ok_or_else(|| {
                invalid("Connection requires a distinct named parent attachment bone")
            })?;
        let child_root = cr
            .bones
            .iter()
            .position(|b| b.name == self.child_root && b.parent.is_none())
            .ok_or_else(|| invalid("Connection child root is missing"))?;
        if cr.weights.iter().any(|w| !rigid(w, child_root)) {
            return Err(invalid(
                "Connection child body must be rigidly bound to its declared root",
            ));
        }
        let (lower, lo) = station(parent, &self.parent_sections[0])?;
        let (upper, hi) = station(parent, &self.parent_sections[1])?;
        if lower >= upper {
            return Err(invalid("Connection parent sections must be ordered"));
        }
        if let Some(a) = &parent.assembly
            && lower < station(parent, &a.station)?.0
        {
            return Err(invalid(
                "Connection cut must lie in the retained parent plant body",
            ));
        }
        let (_, cut) = station(child, &self.child_section)?;
        let ps = parent.definition.sample()?;
        let cs = child.definition.sample()?;
        let map = base_map(parent, pa, &ps)?;
        let lookup: BTreeMap<_, _> = pa
            .asset
            .geometry
            .triangles
            .iter()
            .enumerate()
            .map(|(i, t)| (*t, i as u32))
            .collect();
        let mut removed = BTreeSet::new();
        for t in &ps.geometry.triangles {
            let ys = t.map(|v| f64::from(ps.material[v as usize][1]));
            let min = ys.into_iter().fold(f64::INFINITY, f64::min);
            let max = ys.into_iter().fold(f64::NEG_INFINITY, f64::max);
            if min >= lo - 1e-6 && max <= hi + 1e-6 && max - min > 1e-6 {
                let mapped = t.map(|v| map[v as usize]);
                let [Some(a), Some(b), Some(c)] = mapped else {
                    return Err(invalid("Connection cut crosses removed parent cells"));
                };
                removed.insert(
                    *lookup
                        .get(&[a, b, c])
                        .ok_or_else(|| invalid("Connection parent cell is absent"))?,
                );
            }
        }
        let parent_ids = kept(&pa.asset.geometry, &removed);
        let parent_cut =
            mesh_bridge::remove_faces(&pa.asset, &removed.into_iter().collect::<Vec<_>>())?;
        let child_removed: BTreeSet<_> = cs
            .geometry
            .triangles
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.iter()
                    .any(|&v| f64::from(cs.material[v as usize][1]) < cut - 1e-6)
            })
            .map(|(i, _)| i as u32)
            .collect();
        let child_ids = kept(&ca.asset.geometry, &child_removed);
        let child_cut =
            mesh_bridge::remove_faces(&ca.asset, &child_removed.into_iter().collect::<Vec<_>>())?;
        let pb = mesh_bridge::boundaries(&parent_cut.geometry)?;
        let cb = mesh_bridge::boundaries(&child_cut.geometry)?;
        if pb.len() != 2 || cb.len() != 1 {
            return Err(invalid(
                "Connection cuts must expose two parent ports and one child port",
            ));
        }
        let ring = |at: f64| -> Result<Vec<u32>> {
            let ids = ps
                .material
                .iter()
                .enumerate()
                .filter(|(_, m)| (f64::from(m[1]) - at).abs() < 1e-6)
                .map(|(v, _)| map[v].ok_or_else(|| invalid("Connection station has been removed")))
                .collect::<Result<BTreeSet<_>>>()?;
            let mut compact = BTreeSet::new();
            for id in ids {
                compact.insert(
                    parent_ids
                        .binary_search(&(id as usize))
                        .map_err(|_| invalid("Connection port vertex absent"))?
                        as u32,
                );
            }
            pb.iter()
                .find(|b| b.vertices.iter().copied().collect::<BTreeSet<_>>() == compact)
                .map(|b| b.vertices.clone())
                .ok_or_else(|| invalid("Connection station does not match a real boundary"))
        };
        let n = parent_ids.len();
        if n + child_ids.len() >= meshes::MAX_VERTICES
            || parent_cut.geometry.triangles.len() + child_cut.geometry.triangles.len()
                >= meshes::MAX_TRIANGLES
        {
            return Err(invalid(
                "Connection retained geometry leaves no union budget",
            ));
        }
        let ports = [
            ring(lo)?,
            cb[0].vertices.iter().map(|v| *v + n as u32).collect(),
            ring(hi)?,
        ];
        let parent_weights = &parent_cut
            .rig
            .as_ref()
            .ok_or_else(|| invalid("Missing retained rig"))?
            .weights;
        if ports[0]
            .iter()
            .chain(&ports[2])
            .any(|&i| !rigid(&parent_weights[i as usize], root))
        {
            return Err(invalid(
                "Connection parent ports must be rigid at the declared root",
            ));
        }
        let transform = pr.bind_globals()[anchor]
            * Mat4::from_scale_rotation_translation(
                Vec3::splat(ci.scale),
                Quat::from_array(ci.rotation).normalize(),
                Vec3::from(ci.position),
            );
        let mut geometry = (*parent_cut.geometry).clone();
        geometry.positions.extend(
            child_cut
                .geometry
                .positions
                .iter()
                .map(|p| transform.transform_point3(Vec3::from(*p)).to_array()),
        );
        geometry.triangles.extend(
            child_cut
                .geometry
                .triangles
                .iter()
                .map(|t| t.map(|v| v + n as u32)),
        );
        match (
            &mut geometry.material_coordinates,
            &child_cut.geometry.material_coordinates,
        ) {
            (Some(a), Some(b)) => a.extend_from_slice(b),
            (None, None) => {}
            _ => {
                return Err(invalid(
                    "Connection requires matching material-channel presence",
                ));
            }
        }
        let center = |port: &[u32]| {
            port.iter()
                .map(|&v| Vec3::from(geometry.positions[v as usize]))
                .fold(Vec3::ZERO, |a, b| a + b)
                / port.len() as f32
        };
        let hub = (center(&ports[0]) + center(&ports[2])) * 0.5 + Vec3::from(self.shape.hub_offset);
        let options = mesh_junction_union::Options {
            hub: hub.to_array(),
            hub_radius: self.shape.hub_radius,
            blend: self.shape.blend,
            cell_size: self.shape.cell_size,
            guard_length: self.shape.guard_length,
            collar_length: self.shape.collar_length,
            relaxation_steps: self.shape.relaxation_steps,
            material_offset: self.shape.material_offset,
            seam_relaxation_steps: self.shape.seam_relaxation_steps,
            seam_max_displacement: self.shape.seam_max_displacement,
            material_relaxation_steps: self.shape.material_relaxation_steps,
        };
        let mut source = pa.asset.clone();
        source.geometry = Arc::new(geometry);
        source.rig = None;
        source.clips.clear();
        let fitted = mesh_junction_union::fit(&source, &ports, None, &options)?;
        let mut rig = (**pr).clone();
        rig.weights = parent_weights.clone();
        rig.weights.extend(std::iter::repeat_n(
            meshes::Weight {
                extra: None,
                joints: [anchor as u8, 0, 0, 0],
                weights: [1., 0., 0., 0.],
            },
            child_ids.len(),
        ));
        let (weights, skin_report) = skin::extend(
            &fitted.asset.geometry,
            &fitted.material_plan,
            &rig.weights,
            root,
            anchor,
            &self.skin,
        )?;
        rig.weights = weights;
        let mut out = fitted.asset;
        out.rig = Some(Arc::new(rig));
        out.clips = pa.asset.clips.clone();
        out.validate()?;
        let report = ConnectionReport {
            id: self.id.clone(),
            parent_asset: pa.asset.id.clone(),
            child_asset: ca.asset.id.clone(),
            retained_parent_vertices: n,
            retained_child_vertices: child_ids.len(),
            junction: fitted.report,
            skin: skin_report,
        };
        Ok((out, report))
    }
}
