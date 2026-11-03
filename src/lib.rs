pub mod abilities;
pub mod ability_policy;
pub mod actor_set;
pub mod botany;
mod character_cast;
pub mod combat_spatial;
pub mod communication;
pub mod controls;
pub mod debug_routes;
pub mod editor;
pub mod editor_modeling;
pub mod editor_transform;
pub mod editor_ui;
pub mod editor_window;
pub mod engine;
pub mod footsteps;
pub mod interactions;
pub mod mcp;
pub mod mesh_screen;
pub mod mesh_surface;
pub mod source_pool;

pub mod landmark_projection;
pub mod mesh_chamfer_boxes;
pub mod mesh_topology;
pub mod model;
pub mod navigation;
pub mod network;
pub mod noise;
pub mod patrol;
pub mod perception;
pub mod physics;
pub mod picking;
pub mod player_view;
pub mod projectiles;
pub mod recognition;
pub mod render;
mod render_transparency;
pub mod scenario;
pub mod service;
pub mod sound_attention;
pub mod targeting;
mod transaction;
pub mod trigger;
pub mod vitality;
pub mod wind;
pub mod window;

pub mod objectives;

pub mod territory;

pub mod nav_recovery;

pub mod passages;

pub mod mobility;

pub mod flight;
pub mod walk_support;

pub mod periodic;

pub mod channel;

pub mod attributes;

pub mod damage;

pub mod area;

pub mod persistent_area;

pub mod identity;
pub mod image_similarity;

#[cfg(test)]
mod identity_bindings_tests;

pub mod origins;
pub mod runtime_world;

#[cfg(test)]
mod world_origins_tests;

pub mod population;

pub mod spawn_region;
pub mod spawn_support;
pub mod template_selection;
pub mod templates;

pub mod generator_activation;
pub mod generator_control;
pub mod generators;
pub mod geometry;

pub mod waves;

pub mod materials;
pub mod mesh_modeling;
pub mod mesh_organic;
pub mod mesh_preservation;
mod mesh_sculpt_shapes;
pub mod meshes;
pub mod plants;

pub mod mesh_bridge;

pub mod mesh_junction_union;
pub mod mesh_normals;
pub mod mesh_ring_loft;
pub mod mesh_sweep;

pub mod mesh_material_domain;
pub mod mesh_material_finish;
pub mod mesh_projection;

pub mod mesh_panel;

pub mod project;
pub mod project_browser;

pub mod mesh_tools;

pub mod character_controller;

pub mod ground_navigation;

pub mod environment;
mod environment_gpu;

pub mod avatar_ik;
pub mod avatar_motion;
pub mod npc_circulation;
pub mod npc_cooperation;
pub mod npc_mind;
pub mod npc_presentation;
pub mod npc_routines;
pub mod npc_tactics;

pub mod npc_navigation_generation;

pub mod checkpoint;
pub mod portable_save;

pub mod retarget;

pub mod audio_device;

pub mod planet;
pub mod world_space;

pub mod landforms;
pub mod terrain;
pub mod world_regions;

pub mod atmosphere;

pub mod distant;

mod render_geometry;
mod render_instances;
mod render_outline;
mod render_visibility;

mod atmosphere_edges;
pub mod viewport_performance;

mod render_occlusion;
pub mod surfaces;

pub mod character_editor;
pub mod mesh_lods;
pub mod mesh_objects;

pub mod collective_lod;

pub mod particles;

mod render_particles;
