//! Static geometry is uploaded once per asset/level. Instance and geometry pages
//! follow device buffer limits; there is no authored object-count ceiling.
use crate::{distant::selection::Selection, render::RenderFrame};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    sync::Arc,
};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
struct Instance {
    world: [[f32; 4]; 4],
    color: [f32; 3],
    material: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CompactVertex {
    position: [f32; 3],
    normal: [f32; 3],
    rest: [f32; 3],
    weights: [f32; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MinimalVertex {
    position: [f32; 3],
    normal: [f32; 3],
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    source: usize,
    smooth: bool,
    normal: usize,
    representation: usize,
    level: Option<usize>,
    cheap: bool,
    full_material_slot: usize,
    layout: u8,
    indexed: bool,
}
#[derive(Default)]
struct Representations {
    normals: Option<Arc<Vec<[f32; 3]>>>,
    atmosphere: Option<Arc<Vec<u8>>>,
    lod: Option<Arc<crate::mesh_lods::Pack>>,
}
struct Geometry {
    // Own the identity behind pointer keys: allocator address reuse cannot alias
    // an old payload while its GPU entry survives.
    _source: Arc<crate::meshes::Geometry>,
    _normals: Option<Arc<Vec<[f32; 3]>>>,
    _atmosphere: Option<Arc<Vec<u8>>>,
    _lod: Option<Arc<crate::mesh_lods::Pack>>,
    pages: Vec<crate::render_geometry::Page>,
    bytes: usize,
    index_bytes: usize,
    triangles: usize,
    compact: bool,
    minimal: bool,
    indexing_allowed: bool,
}
#[derive(Clone, PartialEq)]
struct Batch {
    key: Key,
    asset: usize,
    mask: u8,
    instances: Range<usize>,
}
enum Command {
    Shared(usize),
    Legacy(bool, Range<u32>, u8),
}
fn legacy(commands: &mut Vec<Command>, cheap: bool, range: Range<u32>, mask: u8) {
    if range.is_empty() || mask == 0 {
        return;
    }
    if let Some(Command::Legacy(last_cheap, last, last_mask)) = commands.last_mut()
        && *last_mask == mask
        && *last_cheap == cheap
        && last.end == range.start
    {
        last.end = range.end;
    } else {
        commands.push(Command::Legacy(cheap, range, mask));
    }
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Footprint {
    pub instances: usize,
    pub source_triangles: usize,
    pub drawn_triangles: usize,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Stats {
    pub instances: usize,
    pub occluder_work_budget_triangles: usize,
    pub adaptive_occluder_budget: bool,
    pub minimal_instances: usize,
    pub minimal_triangles: usize,
    pub detailed_triangles: usize,
    pub cheap_triangles: usize,
    pub selected_geometry_bytes: usize,
    pub allocated_geometry_bytes: usize,
    pub selected_index_bytes: usize,
    pub allocated_index_bytes: usize,
    pub selected_vertices: usize,
    pub selected_indices: usize,
    /// Camera bounds: <=1, <=2, <=4, <=16, <=64, >64 pixels (or unknown).
    pub camera_footprint: [Footprint; 6],
    pub instance_bytes: usize,
    pub allocated_instance_bytes: u64,
    pub draw_calls: usize,
    pub reordered_instances: usize,
    pub geometry_uploads: u64,
    pub instance_uploads: u64,
}
pub(crate) struct Gpu {
    occluder_pipelines: [wgpu::RenderPipeline; 6],
    pub occlusion: crate::render_occlusion::Gpu,
    occluders: Vec<(usize, usize)>,
    legacy_occluders: Vec<(bool, Range<u32>)>,
    occlusion_bounds: Vec<crate::render_occlusion::Bound>,
    occlusion_jobs: Vec<crate::render_occlusion::Job>,
    empty_occlusion_input: Option<wgpu::Buffer>,
    source_ids: Vec<usize>,
    full: wgpu::RenderPipeline,
    full_compact: wgpu::RenderPipeline,
    full_minimal: wgpu::RenderPipeline,
    cheap: wgpu::RenderPipeline,
    shadow: wgpu::RenderPipeline,
    shadow_compact: wgpu::RenderPipeline,
    shadow_minimal: wgpu::RenderPipeline,
    geometry: BTreeMap<Key, Geometry>,
    generation: Option<u64>,
    representations: Vec<Representations>,
    packs: Option<std::sync::Arc<Vec<crate::distant::Pack>>>,
    lods: Option<std::sync::Arc<crate::mesh_lods::Document>>,
    batches: Vec<Batch>,
    commands: Vec<Command>,
    order: Vec<usize>,
    instances: Vec<Instance>,
    scratch: Vec<Instance>,
    instance_pages: Vec<wgpu::Buffer>,
    per_page: usize,
    indexing: bool,
    compact_vertices: bool,
    minimal_vertices: bool,
    adaptive_occluders: bool,
    pub stats: Stats,
}
impl Gpu {
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        shadow_layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
    ) -> Self {
        let attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Float32x3,4=>Uint32,5=>Float32x3];
        let compact_attrs =
            wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,3=>Float32x3,5=>Float32x3];
        let transforms = wgpu::vertex_attr_array![6=>Float32x4,7=>Float32x4,8=>Float32x4,9=>Float32x4,10=>Float32x3,11=>Uint32];
        let pipeline = |cheap: bool, shadow: bool, compact: bool, minimal: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if shadow {
                    "shared static shadow"
                } else if cheap {
                    "shared atmospheric geometry"
                } else {
                    "shared static geometry"
                }),
                layout: Some(if shadow { shadow_layout } else { layout }),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some(if shadow {
                        "vs_shared_shadow"
                    } else if cheap {
                        "vs_shared_distant"
                    } else if minimal {
                        "vs_shared_minimal"
                    } else if compact {
                        "vs_shared_compact"
                    } else {
                        "vs_shared"
                    }),
                    compilation_options: Default::default(),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: if cheap {
                                12
                            } else if minimal {
                                24
                            } else if compact {
                                48
                            } else {
                                64
                            },
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: if cheap {
                                &attrs[..1]
                            } else if minimal {
                                &attrs[..2]
                            } else if compact {
                                &compact_attrs
                            } else {
                                &attrs
                            },
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Instance>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &transforms,
                        },
                    ],
                },
                fragment: (!shadow).then_some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some(if cheap { "fs_distant" } else { "fs_main" }),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: if minimal {
                            &[("uniform_material", 1.0)]
                        } else {
                            &[]
                        },
                        ..Default::default()
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: if shadow {
                        wgpu::DepthBiasState {
                            constant: 1,
                            slope_scale: 1.,
                            clamp: 0.,
                        }
                    } else {
                        Default::default()
                    },
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let occluder_pipelines = [
            (64, true),
            (48, true),
            (12, true),
            (64, false),
            (24, false),
            (24, true),
        ]
        .map(|(stride, shared)| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("current reverse-Z occluder"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some(if shared {
                        "vs_shared_occlusion"
                    } else {
                        "vs_occlusion"
                    }),
                    compilation_options: Default::default(),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: stride,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &attrs[..1],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: 80,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &transforms,
                        },
                    ][..if shared { 2 } else { 1 }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some("fs_occlusion"),
                    compilation_options: Default::default(),
                    targets: &[],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        });
        Self {
            occluder_pipelines,
            occlusion: crate::render_occlusion::Gpu::new(device),
            occluders: Vec::new(),
            legacy_occluders: Vec::new(),
            occlusion_bounds: Vec::new(),
            occlusion_jobs: Vec::new(),
            empty_occlusion_input: None,
            source_ids: Vec::new(),
            full: pipeline(false, false, false, false),
            full_compact: pipeline(false, false, true, false),
            full_minimal: pipeline(false, false, true, true),
            cheap: pipeline(true, false, false, false),
            shadow: pipeline(false, true, false, false),
            shadow_compact: pipeline(false, true, true, false),
            shadow_minimal: pipeline(false, true, true, true),
            geometry: BTreeMap::new(),
            generation: None,
            representations: Vec::new(),
            packs: None,
            lods: None,
            batches: Vec::new(),
            commands: Vec::new(),
            order: Vec::new(),
            instances: Vec::new(),
            scratch: Vec::new(),
            instance_pages: Vec::new(),
            per_page: (device.limits().max_buffer_size as usize / std::mem::size_of::<Instance>())
                .min(u32::MAX as usize),
            indexing: true,
            compact_vertices: true,
            minimal_vertices: true,
            adaptive_occluders: true,
            stats: Stats::default(),
        }
    }
    pub fn set_fine_occlusion(&mut self, enabled: bool) {
        self.occlusion.fine_enabled = enabled;
    }
    pub fn set_adaptive_occluders(&mut self, enabled: bool) {
        self.adaptive_occluders = enabled;
    }
    pub fn set_minimal_vertices(&mut self, enabled: bool) {
        if self.minimal_vertices != enabled {
            self.geometry.clear();
            self.minimal_vertices = enabled;
        }
    }
    pub fn set_indexing(&mut self, enabled: bool) {
        if self.indexing != enabled {
            self.geometry.clear();
            self.indexing = enabled;
        }
    }
    pub fn set_compact_vertices(&mut self, enabled: bool) {
        if self.compact_vertices != enabled {
            self.geometry.clear();
            self.compact_vertices = enabled;
        }
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &RenderFrame,
        selected: &Selection,
        visibility: &crate::render_visibility::Visibility,
    ) -> Result<()> {
        let lod_changed = self
            .lods
            .as_ref()
            .is_none_or(|d| !std::sync::Arc::ptr_eq(d, &frame.mesh_lods));
        let pack_changed = lod_changed
            || self
                .packs
                .as_ref()
                .is_none_or(|p| !std::sync::Arc::ptr_eq(p, &frame.distant_meshes));
        if self.generation != Some(frame.meshes.geometry_generation) || pack_changed {
            let mut live = BTreeSet::new();
            self.representations.clear();
            for (slot, asset) in frame.meshes.source.assets.iter().enumerate() {
                let source = Arc::as_ptr(&asset.geometry) as usize;
                let smooth = asset.shading == crate::meshes::Shading::Smooth;
                let normals = frame.meshes.normal_overrides.get(slot).cloned().flatten();
                let normal = normals.as_ref().map_or(0, |n| Arc::as_ptr(n) as usize);
                live.insert((source, smooth, normal, 0, 0));
                live.insert((source, smooth, normal, 0, slot + 1));
                let atmosphere = frame
                    .distant_meshes
                    .iter()
                    .find(|p| p.asset == asset.id && p.matches(asset))
                    .map(|p| p.data.clone());
                let lod = frame
                    .mesh_lods
                    .packs
                    .iter()
                    .find(|p| p.matches(asset))
                    .cloned();
                if let Some(p) = &atmosphere {
                    live.insert((source, smooth, normal, Arc::as_ptr(p) as usize, 0));
                    live.insert((source, smooth, normal, Arc::as_ptr(p) as usize, slot + 1));
                }
                if let Some(p) = &lod {
                    live.insert((source, smooth, normal, Arc::as_ptr(p) as usize, 0));
                    live.insert((source, smooth, normal, Arc::as_ptr(p) as usize, slot + 1));
                }
                self.representations.push(Representations {
                    atmosphere,
                    lod,
                    normals,
                });
            }
            // Catalog/list changes retire only obsolete sources/representations.
            // Device resources referenced by submitted work retain wgpu ownership.
            self.geometry.retain(|key, _| {
                live.contains(&(
                    key.source,
                    key.smooth,
                    key.normal,
                    key.representation,
                    key.full_material_slot,
                ))
            });
            self.packs = Some(frame.distant_meshes.clone());
            self.lods = Some(frame.mesh_lods.clone());
            self.generation = Some(frame.meshes.geometry_generation);
        }
        let geometry_uploads = self.stats.geometry_uploads;
        let instance_uploads = self.stats.instance_uploads;
        self.stats = Stats {
            geometry_uploads,
            instance_uploads,
            ..Default::default()
        };
        let mut instances = std::mem::take(&mut self.scratch);
        instances.clear();
        self.source_ids.clear();
        let mut batches = std::mem::take(&mut self.batches);
        batches.clear();
        self.commands.clear();
        let mut end = 0;
        let mut used = BTreeSet::new();
        self.order.clear();
        let key_for = |n: usize| {
            let draw = &frame.meshes.draws[n];
            let c = selected.plan.get(n);
            (
                draw.asset,
                c.and_then(|c| {
                    if c.color.is_some() {
                        c.level
                    } else {
                        c.geometric
                    }
                }),
                c.is_some_and(|c| c.color.is_some()),
                visibility.masks[n],
            )
        };
        for (n, draw) in frame.meshes.draws.iter().enumerate() {
            if draw.shared && visibility.masks[n] == 0 {
                continue;
            }
            // Only commute disjoint shared draws, across at most32 neighbours.
            // Legacy draws are barriers; stable order retains all depth ties.
            let mut at = self.order.len();
            if visibility.stats.enabled && draw.shared && visibility.masks[n] != 0 {
                for p in (self.order.len().saturating_sub(32)..self.order.len()).rev() {
                    let previous = self.order[p];
                    if !frame.meshes.draws[previous].shared {
                        break;
                    }
                    if key_for(previous) == key_for(n) {
                        at = p + 1;
                        break;
                    }
                    if !visibility.independent(n, previous) {
                        break;
                    }
                }
            }
            self.stats.reordered_instances += usize::from(at < self.order.len());
            self.order.insert(at, n);
        }
        for &n in &self.order {
            let draw = &frame.meshes.draws[n];
            let mask = visibility.masks[n];
            if !draw.shared {
                end = draw.range.end;
                let (cheap, range) = if selected.active {
                    selected.draw_ranges[n].clone()
                } else {
                    (false, draw.range.start as u32..draw.range.end as u32)
                };
                legacy(&mut self.commands, cheap, range, mask);
                continue;
            }
            if mask == 0 {
                continue;
            }
            let choice = selected.plan.get(n);
            let color = choice.and_then(|c| c.color);
            let level = choice.and_then(|c| {
                if c.color.is_some() {
                    c.level
                } else {
                    c.geometric
                }
            });
            // A material-only edit can leave all vertex bytes unchanged.
            // Re-evaluate this policy independently of geometry invalidation.
            let material = &frame.meshes.source.assets[draw.asset].material;
            // Promoting unique surfaces must not also introduce indexed raster
            // interpolation: a curved floor exposed RGB8=10 differences on Metal.
            // Keep their original corner stream. Repeated assets retain the
            // existing indexing policy; constant atmospheric fills can index.
            let indexed = self.indexing
                && (color.is_some()
                    || (draw.repeated
                        && material.pattern.is_none()
                        && material.secondary_pattern.is_none()
                        && material.detail.is_none()));
            let minimal_material = self.minimal_vertices
                && material.pattern.is_none()
                && material.secondary_pattern.is_none()
                && material.detail.is_none();
            let asset = &frame.meshes.source.assets[draw.asset];
            let representations = &self.representations[draw.asset];
            let representation = if level.is_none() {
                0
            } else if color.is_some() {
                representations
                    .atmosphere
                    .as_ref()
                    .map_or(0, |p| Arc::as_ptr(p) as usize)
            } else {
                representations
                    .lod
                    .as_ref()
                    .map_or(0, |p| Arc::as_ptr(p) as usize)
            };
            // Shared static streams are white and use the owning asset's material.
            // Compact/minimal layouts remap material via Instance; full64 embeds it.
            let key = Key {
                source: Arc::as_ptr(&asset.geometry) as usize,
                smooth: asset.shading == crate::meshes::Shading::Smooth,
                normal: representations
                    .normals
                    .as_ref()
                    .map_or(0, |n| Arc::as_ptr(n) as usize),
                representation,
                level,
                cheap: color.is_some(),
                full_material_slot: if color.is_none() && !self.compact_vertices {
                    draw.asset + 1
                } else {
                    0
                },
                layout: if color.is_some() {
                    0
                } else if !self.compact_vertices {
                    3
                } else if minimal_material {
                    1
                } else {
                    2
                },
                indexed,
            };
            if self.geometry.get(&key).is_some_and(|g| {
                g.indexing_allowed != indexed || g.minimal != (minimal_material && g.compact)
            }) {
                self.geometry.remove(&key);
            }
            if let std::collections::btree_map::Entry::Vacant(entry) = self.geometry.entry(key) {
                let lod_vertices: Vec<crate::render::Vertex>;
                let mut compiled_indices = None;
                let source_vertices = if let Some(level) = level.filter(|_| color.is_none()) {
                    let asset = &frame.meshes.source.assets[draw.asset];
                    let pack = frame
                        .mesh_lods
                        .packs
                        .iter()
                        .find(|p| p.matches(asset))
                        .expect("selected matching LOD");
                    let l = &pack.levels[level];
                    if indexed
                        && l.corners.len() * 64 <= device.limits().max_buffer_size as usize
                        && (l.triangles.len() * 6).next_multiple_of(4)
                            <= device.limits().max_buffer_size as usize
                    {
                        compiled_indices = Some(l.triangles.as_slice());
                        lod_vertices = l
                            .corners
                            .iter()
                            .map(|&c| frame.meshes.shared_vertices[draw.range.start + c as usize])
                            .collect();
                    } else {
                        // Hardware-sized fallback follows the existing paginated path.
                        lod_vertices = l
                            .triangles
                            .iter()
                            .flatten()
                            .map(|&i| {
                                frame.meshes.shared_vertices
                                    [draw.range.start + l.corners[i as usize] as usize]
                            })
                            .collect();
                    }
                    lod_vertices.as_slice()
                } else {
                    &frame.meshes.shared_vertices[draw.range.clone()]
                };
                let cheap_positions: Vec<[f32; 3]>;
                let compact = color.is_none()
                    && self.compact_vertices
                    && source_vertices
                        .iter()
                        .all(|v| v.color == [1.; 3] && v.material as usize == draw.asset + 1);
                let minimal = compact && minimal_material;
                let minimal_data: Vec<MinimalVertex>;
                let compact_data: Vec<CompactVertex>;
                let bytes = if color.is_some() {
                    if let Some(level) = level {
                        let level = selected.level(draw.asset, level);
                        cheap_positions = level
                            .triangles
                            .iter()
                            .flatten()
                            .map(|&i| level.positions[i as usize])
                            .collect();
                    } else {
                        cheap_positions = frame.meshes.shared_vertices[draw.range.clone()]
                            .iter()
                            .map(|v| v.position)
                            .collect();
                    }
                    bytemuck::cast_slice(&cheap_positions)
                } else if minimal {
                    minimal_data = source_vertices
                        .iter()
                        .map(|v| MinimalVertex {
                            position: v.position,
                            normal: v.normal,
                        })
                        .collect();
                    bytemuck::cast_slice(&minimal_data)
                } else if compact {
                    compact_data = source_vertices
                        .iter()
                        .map(|v| CompactVertex {
                            position: v.position,
                            normal: v.normal,
                            rest: v.rest_position,
                            weights: v.material_projection_weights,
                        })
                        .collect();
                    bytemuck::cast_slice(&compact_data)
                } else {
                    bytemuck::cast_slice(source_vertices)
                };
                let stride = if color.is_some() {
                    12
                } else if minimal {
                    24
                } else if compact {
                    48
                } else {
                    64
                };
                anyhow::ensure!(
                    device.limits().max_buffer_size as usize >= stride * 3,
                    "GPU cannot allocate one triangle"
                );
                // Detailed procedural surfaces showed a first/repeated-draw
                // discrepancy on Metal with indexing. Retain their stream until
                // interpolation/derivative stability is demonstrated; atmospheric
                // substitution no longer evaluates that material.
                let pages = if let Some(indices) = compiled_indices {
                    vec![crate::render_geometry::compiled(
                        device,
                        bytes,
                        indices,
                        source_vertices.len(),
                    )]
                } else if color.is_some() {
                    crate::render_geometry::upload::<3>(device, bytes, indexed)
                } else if minimal {
                    crate::render_geometry::upload::<6>(device, bytes, indexed)
                } else if compact {
                    crate::render_geometry::upload::<12>(device, bytes, indexed)
                } else {
                    crate::render_geometry::upload::<16>(device, bytes, indexed)
                };
                let allocated_bytes = pages.iter().map(|p| p.bytes()).sum();
                let index_bytes = pages
                    .iter()
                    .filter_map(|p| p.indices.as_ref())
                    .map(|b| b.buffer.size() as usize)
                    .sum();
                entry.insert(Geometry {
                    _source: asset.geometry.clone(),
                    _normals: representations.normals.clone(),
                    _atmosphere: if key.cheap && key.level.is_some() {
                        representations.atmosphere.clone()
                    } else {
                        None
                    },
                    _lod: if !key.cheap && key.level.is_some() {
                        representations.lod.clone()
                    } else {
                        None
                    },
                    pages,
                    bytes: allocated_bytes,
                    index_bytes,
                    triangles: compiled_indices.map_or(bytes.len() / stride / 3, |i| i.len()),
                    compact,
                    minimal,
                    indexing_allowed: indexed,
                });
                self.stats.geometry_uploads += 1;
            }
            let geometry = &self.geometry[&key];
            if geometry.minimal {
                self.stats.minimal_instances += 1;
                self.stats.minimal_triangles += geometry.triangles;
            }
            if mask & crate::render_visibility::CAMERA != 0 {
                let bucket = &mut self.stats.camera_footprint[visibility.size_bucket(n)];
                bucket.instances += 1;
                bucket.source_triangles += draw.range.len() / 3;
                bucket.drawn_triangles += geometry.triangles;
            }
            self.stats.selected_vertices +=
                geometry.pages.iter().map(|p| p.vertex_count).sum::<usize>();
            self.stats.selected_indices += geometry
                .pages
                .iter()
                .filter(|p| p.indices.is_some())
                .map(|p| p.count as usize)
                .sum::<usize>();
            if color.is_some() {
                self.stats.cheap_triangles += geometry.triangles;
            } else {
                self.stats.detailed_triangles += geometry.triangles;
            }
            used.insert(key);
            let start = instances.len();
            self.source_ids.push(n);
            instances.push(Instance {
                world: draw.world.to_cols_array_2d(),
                color: color.unwrap_or(draw.color),
                material: (draw.asset + 1) as u32,
            });
            if let Some(last) = batches.last_mut()
                && last.mask == mask
                && last.key == key
                && last.asset == draw.asset
                && matches!(self.commands.last(), Some(Command::Shared(_)))
            {
                last.instances.end += 1;
            } else {
                self.commands.push(Command::Shared(batches.len()));
                batches.push(Batch {
                    key,
                    asset: draw.asset,
                    mask,
                    instances: start..start + 1,
                });
            }
        }
        legacy(
            &mut self.commands,
            false,
            if selected.active {
                selected.tail.clone()
            } else {
                end as u32..frame.meshes.vertices.len() as u32
            },
            visibility.tail,
        );
        if instances != self.instances {
            let page_count = instances.len().div_ceil(self.per_page);
            self.instance_pages.truncate(page_count);
            for (i, page) in instances.chunks(self.per_page).enumerate() {
                let bytes = bytemuck::cast_slice(page);
                if self
                    .instance_pages
                    .get(i)
                    .is_none_or(|b| b.size() < bytes.len() as u64)
                {
                    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("static instance page"),
                        size: bytes.len() as u64,
                        usage: wgpu::BufferUsages::VERTEX
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::STORAGE,
                        mapped_at_creation: false,
                    });
                    if i == self.instance_pages.len() {
                        self.instance_pages.push(buffer);
                    } else {
                        self.instance_pages[i] = buffer;
                    }
                }
                queue.write_buffer(&self.instance_pages[i], 0, bytes);
            }
            self.stats.instance_uploads += 1;
        }
        self.stats.instances = instances.len();
        self.stats.instance_bytes = instances.len() * std::mem::size_of::<Instance>();
        self.stats.allocated_instance_bytes =
            self.instance_pages.iter().map(wgpu::Buffer::size).sum();
        self.stats.selected_geometry_bytes = used.iter().map(|key| self.geometry[key].bytes).sum();
        self.stats.allocated_geometry_bytes = self.geometry.values().map(|g| g.bytes).sum();
        self.stats.selected_index_bytes =
            used.iter().map(|key| self.geometry[key].index_bytes).sum();
        self.stats.allocated_index_bytes = self.geometry.values().map(|g| g.index_bytes).sum();
        for batch in &batches {
            let pages = self.geometry[&batch.key].pages.len();
            self.stats.draw_calls += if pages > 1 {
                // Device-sized geometry pages must retain instance-major order.
                batch.instances.len() * pages
            } else {
                ((batch.instances.end - 1) / self.per_page - batch.instances.start / self.per_page
                    + 1)
                    * pages
            };
        }
        self.scratch = std::mem::replace(&mut self.instances, instances);
        self.batches = batches;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_occlusion(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &RenderFrame,
        selected: &Selection,
        visibility: &crate::render_visibility::Visibility,
        width: u32,
        height: u32,
        enabled: bool,
        extra_work: usize,
    ) {
        self.occlusion.reset();
        self.occluders.clear();
        self.legacy_occluders.clear();
        self.occlusion_bounds.clear();
        self.occlusion_jobs.clear();
        // Spend at most1/64 of candidate instance geometry on depth seeds when
        // the fixed budget is too small. A large seed must cost <=1/256 of that
        // work. Both paths draw CURRENT opaque geometry exactly; only the
        // profitability policy changes, never coverage/depth or main geometry.
        let work = self
            .stats
            .detailed_triangles
            .saturating_add(self.stats.cheap_triangles)
            .saturating_add(extra_work);
        let adaptive = self.adaptive_occluders && work > 262_144 * 64;
        let budget = if adaptive {
            262_144.max(work / 64)
        } else {
            262_144
        };
        self.stats.occluder_work_budget_triangles = budget;
        self.stats.adaptive_occluder_budget = adaptive;
        let limits = device.limits();
        let maximum = limits.max_storage_buffer_binding_size as usize;
        if !enabled
            || work < 1_000_000
            || self.instance_pages.len() > 1
            || (self.instance_pages.is_empty() && extra_work == 0)
            || self
                .instance_pages
                .first()
                .is_some_and(|page| page.size() as usize > maximum)
            || self.instances.len().next_power_of_two().saturating_mul(80) > maximum
            || self.batches.len().next_power_of_two().saturating_mul(32) > maximum
            || self.batches.len() > limits.max_compute_workgroups_per_dimension as usize
            || self
                .batches
                .iter()
                .any(|b| self.geometry[&b.key].pages.len() != 1)
        {
            return;
        }
        self.occlusion_bounds.extend(
            self.source_ids
                .iter()
                .map(|&n| visibility.occlusion_bound(n)),
        );
        let mut triangles = 0;
        let area = (width as f32) * (height as f32);
        for (n, draw) in frame.meshes.draws.iter().enumerate() {
            if draw.shared
                || visibility.masks[n] & crate::render_visibility::CAMERA == 0
                || frame.meshes.source.assets[draw.asset].material.transmission > 0.
            {
                continue;
            }
            let (cheap, range) = if selected.active {
                selected.draw_ranges[n].clone()
            } else {
                (false, draw.range.start as u32..draw.range.end as u32)
            };
            let count = range.len() / 3;
            let r = visibility.occlusion_bound(n).rect;
            let w = (r[2].min(width as f32) - r[0].max(0.)).max(0.);
            let h = (r[3].min(height as f32) - r[1].max(0.)).max(0.);
            if w * h >= area / 64.
                && w.max(h) >= 128.
                && (count <= ((w * h) as usize) / 2 || (adaptive && count <= work / 256))
                && triangles + count <= budget
                && self.legacy_occluders.len() < 128
            {
                self.legacy_occluders.push((cheap, range));
                triangles += count;
            }
        }
        for (b, batch) in self.batches.iter().enumerate() {
            let geometry = &self.geometry[&batch.key];
            self.occlusion_jobs.push(crate::render_occlusion::Job {
                start: batch.instances.start as u32,
                count: if batch.mask & crate::render_visibility::CAMERA != 0 {
                    batch.instances.len() as u32
                } else {
                    0
                },
                vertices: geometry.pages[0].count,
                indexed: u32::from(geometry.pages[0].indices.is_some()),
            });
            if (!batch.key.cheap && batch.key.level.is_some())
                || batch.mask & crate::render_visibility::CAMERA == 0
                || frame.meshes.source.assets[batch.asset]
                    .material
                    .transmission
                    > 0.
            {
                continue;
            }
            for i in batch.instances.clone() {
                let bounds = &self.occlusion_bounds[i];
                let r = bounds.rect;
                let w = (r[2].min(width as f32) - r[0].max(0.)).max(0.);
                let h = (r[3].min(height as f32) - r[1].max(0.)).max(0.);
                // A work budget for occluders, never a bound on authored objects.
                if w * h >= area / 64.
                    && w.max(h) >= 128.
                    && (geometry.triangles <= ((w * h) as usize) / 2
                        || (adaptive && geometry.triangles <= work / 256))
                    && triangles + geometry.triangles <= budget
                    && self.occluders.len() + self.legacy_occluders.len() < 128
                {
                    self.occluders.push((b, i));
                    triangles += geometry.triangles;
                    self.occlusion_bounds[i].nearest = 1.; // Occluders always keep their main draw.
                }
            }
        }
        if self.occluders.is_empty() && self.legacy_occluders.is_empty() {
            return;
        }
        if self.instance_pages.is_empty() && self.empty_occlusion_input.is_none() {
            self.empty_occlusion_input = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("empty native input for collective depth seeds"),
                size: 80,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            }));
        }
        let input = self
            .instance_pages
            .first()
            .or(self.empty_occlusion_input.as_ref())
            .unwrap();
        self.occlusion.prepare(
            device,
            queue,
            input,
            &self.occlusion_bounds,
            &self.occlusion_jobs,
            width,
            height,
            self.occluders.len() + self.legacy_occluders.len(),
            triangles,
        );
    }
    pub fn encode_occlusion(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        globals: &wgpu::BindGroup,
        vertices: &wgpu::Buffer,
        cheap_vertices: &wgpu::Buffer,
        indices: &wgpu::Buffer,
    ) {
        if !self.occlusion.stats.active {
            return;
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("current opaque occluders"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.occlusion.depth(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_bind_group(0, globals, &[]);
            for (cheap, range) in &self.legacy_occluders {
                pass.set_pipeline(&self.occluder_pipelines[if *cheap { 4 } else { 3 }]);
                if *cheap {
                    pass.set_vertex_buffer(0, cheap_vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(range.clone(), 0, 0..1);
                } else {
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.draw(range.clone(), 0..1);
                }
            }
            if let Some(input) = self.instance_pages.first() {
                pass.set_vertex_buffer(1, input.slice(..));
            }
            for &(b, i) in &self.occluders {
                let batch = &self.batches[b];
                let geometry = &self.geometry[&batch.key];
                let page = &geometry.pages[0];
                pass.set_pipeline(
                    &self.occluder_pipelines[if batch.key.cheap {
                        2
                    } else if geometry.minimal {
                        5
                    } else if geometry.compact {
                        1
                    } else {
                        0
                    }],
                );
                pass.set_vertex_buffer(0, page.vertices.slice(..));
                if let Some(index) = &page.indices {
                    pass.set_index_buffer(index.buffer.slice(..), index.format);
                    pass.draw_indexed(0..page.count, 0, i as u32..i as u32 + 1);
                } else {
                    pass.draw(0..page.count, i as u32..i as u32 + 1);
                }
            }
        }
        self.occlusion.encode(encoder);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        shadow: bool,
        full: &'a wgpu::RenderPipeline,
        cheap: &'a wgpu::RenderPipeline,
        vertices: &'a wgpu::Buffer,
        cheap_vertices: &'a wgpu::Buffer,
        indices: &'a wgpu::Buffer,
    ) {
        let view_mask = if shadow {
            crate::render_visibility::SHADOW
        } else {
            crate::render_visibility::CAMERA
        };
        for command in &self.commands {
            let batch = match command {
                Command::Shared(index) => &self.batches[*index],
                Command::Legacy(is_cheap, range, mask) => {
                    if mask & view_mask == 0 {
                        continue;
                    }
                    if *is_cheap && !shadow {
                        pass.set_pipeline(cheap);
                        pass.set_vertex_buffer(0, cheap_vertices.slice(..));
                        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(range.clone(), 0, 0..1);
                    } else {
                        pass.set_pipeline(full);
                        pass.set_vertex_buffer(0, vertices.slice(..));
                        pass.draw(range.clone(), 0..1);
                    }
                    continue;
                }
            };
            if batch.mask & view_mask == 0 {
                continue;
            }
            let compact = self.geometry[&batch.key].compact;
            let minimal = self.geometry[&batch.key].minimal;
            pass.set_pipeline(if shadow && minimal {
                &self.shadow_minimal
            } else if shadow && compact {
                &self.shadow_compact
            } else if shadow {
                &self.shadow
            } else if batch.key.cheap {
                &self.cheap
            } else if minimal {
                &self.full_minimal
            } else if compact {
                &self.full_compact
            } else {
                &self.full
            });
            let pages = &self.geometry[&batch.key].pages;
            if !shadow && self.occlusion.stats.active {
                let Command::Shared(index) = command else {
                    unreachable!()
                };
                let page = &pages[0];
                pass.set_vertex_buffer(
                    1,
                    self.occlusion
                        .output()
                        .slice(batch.instances.start as u64 * 80..batch.instances.end as u64 * 80),
                );
                pass.set_vertex_buffer(0, page.vertices.slice(..));
                if let Some(indices) = &page.indices {
                    pass.set_index_buffer(indices.buffer.slice(..), indices.format);
                    pass.draw_indexed_indirect(self.occlusion.args(), *index as u64 * 32);
                } else {
                    pass.draw_indirect(self.occlusion.args(), *index as u64 * 32);
                }
                continue;
            }
            let mut start = batch.instances.start;
            while start < batch.instances.end {
                let instance_page = start / self.per_page;
                let base = instance_page * self.per_page;
                // Serialise only device-sized multi-page assets. Drawing every
                // instance's first page before their second pages changes depth ties.
                let end = if pages.len() > 1 {
                    start + 1
                } else {
                    batch.instances.end.min(base + self.per_page)
                };
                pass.set_vertex_buffer(1, self.instance_pages[instance_page].slice(..));
                for page in pages {
                    pass.set_vertex_buffer(0, page.vertices.slice(..));
                    let instances = (start - base) as u32..(end - base) as u32;
                    if let Some(indices) = &page.indices {
                        pass.set_index_buffer(indices.buffer.slice(..), indices.format);
                        pass.draw_indexed(0..page.count, 0, instances);
                    } else {
                        pass.draw(0..page.count, instances);
                    }
                }
                start = end;
            }
        }
    }
}
